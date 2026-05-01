//! # Kota - A lightweight ai code agent in Rust.
//!
//! ## Modules
//!
//! - `kota_code`: Core AI code agent functionality with LLM integration, file operations,
//!   code analysis, and task management.
//! - `kota_cli`: Command-line interface components
//!

pub mod kota_cli;
pub mod kota_code;

// Re-export commonly used types for convenience
pub use kota_code::{
    AgentBuilder, AgentInstance, ChatResponse, CommandDef, ContextManager, KotaConfig, McpClient,
    McpManager, Message, OpenAiClient, Plan, PlanManager, SerializableMessage, SessionIdHook,
    SessionMetadata, Skill, SkillManager, Task, TaskStatus, Tool, ToolDefinition, ToolDyn,
    ToolRegistry, Usage, create_agent,
};

// Re-export CLI components for testing
pub use kota_cli::{CommandRegistry, parse_command_input};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::kota_code::prelude::*;
}
