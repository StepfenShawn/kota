use std::time::Duration;

use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::tool::ToolDefinition;
use super::types::{Choice, CompletionResponse, FunctionCall, Message, ToolCall, Usage};

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [Message],
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<ToolSpec<'a>>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<StreamOptions>,
}

#[derive(Serialize)]
struct StreamOptions {
    include_usage: bool,
}

#[derive(Serialize)]
struct ToolSpec<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    function: &'a ToolDefinition,
}

#[derive(Deserialize)]
struct StreamChunk {
    #[serde(default)]
    choices: Vec<StreamChoice>,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct StreamChoice {
    #[serde(default)]
    delta: StreamDelta,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Deserialize, Default)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<StreamToolCall>>,
}

#[derive(Deserialize)]
struct StreamToolCall {
    #[serde(default)]
    index: usize,
    #[serde(default)]
    id: Option<String>,
    #[serde(default, rename = "type")]
    call_type: Option<String>,
    #[serde(default)]
    function: Option<StreamFunctionCall>,
}

#[derive(Deserialize)]
struct StreamFunctionCall {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

#[derive(Default)]
struct ToolCallAccumulator {
    id: String,
    call_type: String,
    name: String,
    arguments: String,
}

/// Minimal client for OpenAI-compatible `/chat/completions` endpoints.
pub struct OpenAiClient {
    http: Client,
    api_key: String,
    base_url: String,
    model: String,
    temperature: Option<f64>,
}

impl OpenAiClient {
    pub fn new(
        api_key: String,
        model: String,
        base_url: String,
        temperature: Option<f64>,
    ) -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            http,
            api_key,
            base_url: base_url.trim_end_matches('/').to_string(),
            model,
            temperature,
        })
    }

    fn body<'a>(
        &'a self,
        messages: &'a [Message],
        tools: &'a [ToolDefinition],
        stream: bool,
        include_usage: bool,
    ) -> ChatRequest<'a> {
        ChatRequest {
            model: &self.model,
            messages,
            temperature: self.temperature,
            tools: tools
                .iter()
                .map(|definition| ToolSpec {
                    kind: "function",
                    function: definition,
                })
                .collect(),
            stream,
            stream_options: include_usage.then_some(StreamOptions {
                include_usage: true,
            }),
        }
    }

    fn endpoint(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }

    fn request(&self, url: &str, body: &ChatRequest<'_>) -> reqwest::RequestBuilder {
        let request = self.http.post(url).json(body);
        if self.api_key.is_empty() {
            request
        } else {
            request.bearer_auth(&self.api_key)
        }
    }

    /// Non-streaming completion.
    pub async fn completion(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<CompletionResponse> {
        let url = self.endpoint();
        let body = self.body(messages, tools, false, false);

        let response = self
            .request(&url, &body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to '{}': {}", url, e))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("OpenAI API error ({}): {}", status, body));
        }

        response
            .json::<CompletionResponse>()
            .await
            .map_err(|e| anyhow!("Failed to parse response from '{}': {}", url, e))
    }

    /// Streaming completion using SSE. `on_token` is called for every content delta.
    pub async fn stream_completion<F>(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        mut on_token: F,
    ) -> Result<CompletionResponse>
    where
        F: FnMut(&str),
    {
        let url = self.endpoint();

        let response = self
            .request(&url, &self.body(messages, tools, true, true))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to '{}': {}", url, e))?;

        let mut response = if response.status().is_success() {
            response
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();

            // Some OpenAI-compatible servers reject `stream_options`. Retry once
            // without it so streaming still works.
            if status == reqwest::StatusCode::BAD_REQUEST {
                let retry = self
                    .request(&url, &self.body(messages, tools, true, false))
                    .send()
                    .await
                    .map_err(|e| anyhow!("Failed to send request to '{}': {}", url, e))?;

                if retry.status().is_success() {
                    retry
                } else {
                    let status = retry.status();
                    let text = retry.text().await.unwrap_or_default();
                    return Err(anyhow!("OpenAI API error ({}): {}", status, text));
                }
            } else {
                return Err(anyhow!("OpenAI API error ({}): {}", status, text));
            }
        };

        let mut buffer = String::new();
        let mut content = String::new();
        let mut tool_calls: Vec<ToolCallAccumulator> = Vec::new();
        let mut usage = Usage::default();
        let mut finish_reason = None;

        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| anyhow!("Failed to read stream from '{}': {}", url, e))?
        {
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(newline) = buffer.find('\n') {
                let line = buffer[..newline].trim_end_matches('\r').to_string();
                buffer.drain(..=newline);

                let Some(payload) = line.strip_prefix("data:") else {
                    continue;
                };
                let payload = payload.trim();
                if payload.is_empty() {
                    continue;
                }
                if payload == "[DONE]" {
                    buffer.clear();
                    break;
                }

                let parsed: StreamChunk = match serde_json::from_str(payload) {
                    Ok(parsed) => parsed,
                    Err(_) => continue,
                };

                if let Some(chunk_usage) = parsed.usage {
                    usage = chunk_usage;
                }

                for choice in parsed.choices {
                    if let Some(reason) = choice.finish_reason {
                        finish_reason = Some(reason);
                    }

                    if let Some(delta_content) = choice.delta.content
                        && !delta_content.is_empty()
                    {
                        content.push_str(&delta_content);
                        on_token(&delta_content);
                    }

                    for delta_call in choice.delta.tool_calls.unwrap_or_default() {
                        if tool_calls.len() <= delta_call.index {
                            tool_calls
                                .resize_with(delta_call.index + 1, ToolCallAccumulator::default);
                        }
                        let accumulator = &mut tool_calls[delta_call.index];

                        if let Some(id) = delta_call.id {
                            accumulator.id.push_str(&id);
                        }
                        if let Some(call_type) = delta_call.call_type {
                            accumulator.call_type.push_str(&call_type);
                        }
                        if let Some(function) = delta_call.function {
                            if let Some(name) = function.name {
                                accumulator.name.push_str(&name);
                            }
                            if let Some(arguments) = function.arguments {
                                accumulator.arguments.push_str(&arguments);
                            }
                        }
                    }
                }
            }
        }

        let message = if tool_calls.is_empty() {
            Message::assistant(content)
        } else {
            let calls = tool_calls
                .into_iter()
                .map(|accumulator| ToolCall {
                    id: accumulator.id,
                    call_type: if accumulator.call_type.is_empty() {
                        "function".to_string()
                    } else {
                        accumulator.call_type
                    },
                    function: FunctionCall {
                        name: accumulator.name,
                        arguments: accumulator.arguments,
                    },
                })
                .collect();
            Message::assistant_with_tool_calls((!content.is_empty()).then_some(content), calls)
        };

        Ok(CompletionResponse {
            choices: vec![Choice {
                message,
                finish_reason,
            }],
            usage,
        })
    }
}
