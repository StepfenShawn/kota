use anyhow::Result;
use colored::*;
use rig::agent::{CancelSignal, StreamingPromptHook};
use rig::completion::{CompletionModel, GetTokenUsage, Message};
use rig::message::{AssistantContent, UserContent};

/// Session-aware hook that logs tool calls and completions with session context
#[derive(Clone)]
pub struct SessionIdHook {
    pub session_id: String,
    pub enable_logging: bool,
}

impl SessionIdHook {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            enable_logging: true,
        }
    }

    pub fn with_logging(mut self, enable: bool) -> Self {
        self.enable_logging = enable;
        self
    }

    fn log(&self, message: &str) -> Result<()> {
        if self.enable_logging {
            // Use println! instead of raw_println! since hooks are called during streaming
            println!(
                "{} [Session {}] {}",
                "🔍".bright_blue(),
                self.session_id.bright_cyan(),
                message
            );
        }
        Ok(())
    }
}

impl<M: CompletionModel> StreamingPromptHook<M> for SessionIdHook {
    async fn on_tool_call(
        &self,
        tool_name: &str,
        tool_call_id: Option<String>,
        args: &str,
        _cancel_sig: CancelSignal,
    ) {
        let call_id = tool_call_id.unwrap_or_else(|| "<no call ID>".to_string());
        let message = format!(
            "🔧 Calling tool: {} (ID: {}) with args: {}",
            tool_name.bright_green(),
            call_id.dimmed(),
            args.bright_white()
        );
        let _ = self.log(&message);
    }

    async fn on_tool_result(
        &self,
        tool_name: &str,
        _tool_call_id: Option<String>,
        args: &str,
        result: &str,
        _cancel_sig: CancelSignal,
    ) {
        // Truncate long results for readability
        let truncated_result = if result.chars().count() > 200 {
            let truncated: String = result.chars().take(200).collect();
            format!("{}...", truncated)
        } else {
            result.to_string()
        };

        let message = format!(
            "✅ Tool {} completed (args: {}) → {}",
            tool_name.bright_green(),
            args.dimmed(),
            truncated_result.bright_white()
        );
        let _ = self.log(&message);
    }

    async fn on_completion_call(
        &self,
        prompt: &Message,
        history: &[Message],
        _cancel_sig: CancelSignal,
    ) {
        let prompt_text = match prompt {
            Message::User { content } => content
                .iter()
                .filter_map(|c| {
                    if let UserContent::Text(text_content) = c {
                        Some(text_content.text.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
            Message::Assistant { content, .. } => content
                .iter()
                .filter_map(|c| {
                    if let AssistantContent::Text(text_content) = c {
                        Some(text_content.text.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
        };

        let truncated_prompt = if prompt_text.len() > 100 {
            format!("{}...", &prompt_text[..100])
        } else {
            prompt_text
        };

        let message = format!(
            "📤 Sending prompt (history: {} messages): {}",
            history.len().to_string().bright_yellow(),
            truncated_prompt.bright_white()
        );
        let _ = self.log(&message);
    }

    async fn on_text_delta(
        &self,
        text_delta: &str,
        _aggregated_text: &str,
        _cancel_sig: CancelSignal,
    ) {
        if self.enable_logging && !text_delta.trim().is_empty() {
            let message = format!("� Text delta: {}", text_delta.bright_white());
            let _ = self.log(&message);
        }
    }

    async fn on_tool_call_delta(
        &self,
        tool_call_id: &str,
        tool_name: Option<&str>,
        tool_call_delta: &str,
        _cancel_sig: CancelSignal,
    ) {
        if self.enable_logging {
            let name = tool_name.unwrap_or("unknown");
            let message = format!(
                "🔧 Tool call delta: {} (ID: {}) → {}",
                name.bright_green(),
                tool_call_id.dimmed(),
                tool_call_delta.bright_white()
            );
            let _ = self.log(&message);
        }
    }

    async fn on_stream_completion_response_finish(
        &self,
        _prompt: &Message,
        _response: &M::StreamingResponse,
        _cancel_sig: CancelSignal,
    ) {
        let message = format!(
            "📥 Stream completed: {}",
            _response.token_usage().unwrap().total_tokens
        );
        let _ = self.log(&message);
    }
}
