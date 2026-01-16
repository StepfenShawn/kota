use anyhow::Result;
use colored::*;
use rig::agent::{CancelSignal, StreamingPromptHook};
use rig::completion::CompletionModel;
use rig::completion::Message;
use serde_json;

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
        _tool_call_id: Option<String>,
        args: &str,
        _cancel_sig: CancelSignal,
    ) {
        // 解析参数以获取简洁的显示
        let display_args = if let Ok(json) = serde_json::from_str::<serde_json::Value>(args) {
            // 尝试提取常见的参数
            if let Some(path) = json.get("path").and_then(|v| v.as_str()) {
                path.to_string()
            } else if let Some(pattern) = json.get("pattern").and_then(|v| v.as_str()) {
                pattern.to_string()
            } else if let Some(command) = json.get("command").and_then(|v| v.as_str()) {
                command.to_string()
            } else {
                args.to_string()
            }
        } else {
            args.to_string()
        };

        // 简洁的工具调用显示格式
        println!("{} {}({})", "●".bright_green(), tool_name, display_args);
    }

    async fn on_tool_result(
        &self,
        tool_name: &str,
        _tool_call_id: Option<String>,
        _args: &str,
        result: &str,
        _cancel_sig: CancelSignal,
    ) {
        // 解析结果以获取简洁的显示
        let display_result = if tool_name == "read_file" || tool_name == "Read" {
            // 对于读取文件，显示行数
            let line_count = result.lines().count();
            let first_line = result.lines().next().unwrap_or("");
            let preview = if first_line.len() > 50 {
                format!("{}...", &first_line[..50])
            } else {
                first_line.to_string()
            };
            format!("  └─ 1| {} ... +{} lines", preview.dimmed(), line_count)
        } else if tool_name == "grep_search" || tool_name == "Glob" {
            // 对于搜索，显示匹配数
            let match_count = result.lines().count();
            let first_match = result.lines().next().unwrap_or("");
            format!("  └─ {} ... +{} lines", first_match.dimmed(), match_count)
        } else {
            // 其他工具，简单截断
            let truncated = if result.chars().count() > 100 {
                format!("{}...", result.chars().take(100).collect::<String>())
            } else {
                result.to_string()
            };
            format!("  └─ {}", truncated.dimmed())
        };

        println!("{}", display_result);
    }

    async fn on_completion_call(
        &self,
        _prompt: &Message,
        _history: &[Message],
        _cancel_sig: CancelSignal,
    ) {
        // 不显示完成调用的详细信息，保持界面简洁
    }

    async fn on_text_delta(
        &self,
        _text_delta: &str,
        _aggregated_text: &str,
        _cancel_sig: CancelSignal,
    ) {
        // 不显示文本增量，保持界面简洁
    }

    async fn on_tool_call_delta(
        &self,
        _tool_call_id: &str,
        _tool_name: Option<&str>,
        _tool_call_delta: &str,
        _cancel_sig: CancelSignal,
    ) {
        // 不显示工具调用增量，保持界面简洁
    }

    async fn on_stream_completion_response_finish(
        &self,
        _prompt: &Message,
        _response: &M::StreamingResponse,
        _cancel_sig: CancelSignal,
    ) {
        // 不显示流完成信息，保持界面简洁
    }
}
