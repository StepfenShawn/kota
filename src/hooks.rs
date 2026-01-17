use colored::*;
use rig::agent::{CancelSignal, StreamingPromptHook};
use rig::completion::CompletionModel;
use rig::completion::Message;
use serde_json;

/// Session-aware hook that logs tool calls and completions with session context
#[derive(Clone)]
pub struct SessionIdHook {
    pub session_id: String,
}

impl SessionIdHook {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
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
    }

    async fn on_tool_result(
        &self,
        tool_name: &str,
        _tool_call_id: Option<String>,
        _args: &str,
        result: &str,
        _cancel_sig: CancelSignal,
    ) {
    }

    async fn on_completion_call(
        &self,
        _prompt: &Message,
        _history: &[Message],
        _cancel_sig: CancelSignal,
    ) {
    }

    async fn on_text_delta(
        &self,
        _text_delta: &str,
        _aggregated_text: &str,
        _cancel_sig: CancelSignal,
    ) {
    }

    async fn on_tool_call_delta(
        &self,
        _tool_call_id: &str,
        _tool_name: Option<&str>,
        _tool_call_delta: &str,
        _cancel_sig: CancelSignal,
    ) {
    }

    async fn on_stream_completion_response_finish(
        &self,
        _prompt: &Message,
        _response: &M::StreamingResponse,
        _cancel_sig: CancelSignal,
    ) {
    }
}
