//! # Kota Code - AI Code Agent Library
//!
//! Kota Code is a lightweight AI code agent library that provides a comprehensive set of tools
//! for building AI-powered code assistants. It supports multiple LLM providers and comes
//! with built-in file operations, code analysis, and task management capabilities.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use kota::kota_code::{AgentBuilder, ContextManager, SkillManager};
//! use anyhow::Result;

//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Create context and skill managers
//!     let context = ContextManager::new(".chat_sessions", "my-session".to_string())?;
//!     let skill_manager = SkillManager::new();

//!     // Create an agent instance with context and skills
//!     let agent_instance = AgentBuilder::new(
//!         "your-api-key".to_string(),
//!         "gpt-4".to_string()
//!     )?
//!     .with_context(context)
//!     .with_skill_manager(skill_manager)
//!     .build()?;

//!     // Access the agent, context, and skill manager
//!     // agent_instance.agent
//!     // agent_instance.context()
//!     // agent_instance.skill_manager()

//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - **OpenAI-Compatible API**: Works with OpenAI, DeepSeek, Ollama, and any OpenAI-format endpoint
//! - **File Operations**: Read, write, edit, delete files with built-in tools
//! - **Code Analysis**: Scan codebase, grep search, pattern matching
//! - **Task Management**: Plan mode with task dependencies and status tracking
//! - **Skills System**: Specialized agent behaviors (code review, refactoring, debugging, documentation)
//! - **Context Management**: Persistent conversation history with session support
//! - **Extensible**: Easy to add custom tools and behaviors

// Core modules
pub mod agent;
pub mod context;
pub mod llm;
pub mod mcp;
pub mod plan;
pub mod runtime;
pub mod skills;
pub mod tools;

// Re-export commonly used types for convenience
pub use agent::{AgentBuilder, AgentInstance, create_agent};
pub use context::{ContextManager, SerializableMessage, SessionMetadata};
pub use llm::{ChatResponse, Message, OpenAiClient, Tool, ToolDefinition, ToolDyn, Usage};
pub use mcp::{McpManager, client::McpClient};
pub use plan::{Plan, PlanManager, Task, TaskStatus};
pub use runtime::{CommandDef, KotaConfig, SessionIdHook, ToolRegistry};
pub use skills::{Skill, SkillManager};
pub use tools::{
    FileToolError, WrappedCreateDirectoryTool, WrappedDeleteFileTool, WrappedEditFileTool,
    WrappedExecuteBashCommandTool, WrappedGrepSearchTool, WrappedReadFileTool,
    WrappedScanCodebaseTool, WrappedUpdatePlanTool, WrappedWriteFileTool,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use super::agent::{AgentBuilder, AgentInstance, create_agent};
    pub use super::context::{ContextManager, SerializableMessage, SessionMetadata};
    pub use super::llm::{
        ChatResponse, Message, OpenAiClient, Tool, ToolDefinition, ToolDyn, Usage,
    };
    pub use super::mcp::{McpManager, client::McpClient};
    pub use super::plan::{Plan, PlanManager, Task, TaskStatus};
    pub use super::runtime::{CommandDef, KotaConfig, SessionIdHook, ToolRegistry};
    pub use super::skills::{Skill, SkillManager};
    pub use super::tools::FileToolError;
}
