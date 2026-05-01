//! OpenAI-compatible LLM layer.

pub mod client;
pub mod tool;
pub mod types;

pub use client::OpenAiClient;
pub use tool::{BoxFuture, Tool, ToolDefinition, ToolDyn, ToolError};
pub use types::{ChatResponse, Choice, CompletionResponse, FunctionCall, Message, ToolCall, Usage};
