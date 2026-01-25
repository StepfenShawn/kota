//! # Kota Code - AI Code Agent Library
//!
//! Kota Code is a lightweight AI code agent library that provides a comprehensive set of tools
//! for building AI-powered code assistants. It supports multiple LLM providers and comes
//! with built-in file operations, code analysis, and task management capabilities.
//!
//! ## Quick Start
//!
//! ```rust,no_run
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
//! - **Multiple LLM Providers**: OpenAI, Anthropic, DeepSeek, Cohere, Ollama
//! - **File Operations**: Read, write, edit, delete files with built-in tools
//! - **Code Analysis**: Scan codebase, grep search, pattern matching
//! - **Task Management**: Plan mode with task dependencies and status tracking
//! - **Skills System**: Specialized agent behaviors (code review, refactoring, debugging, documentation)
//! - **Context Management**: Persistent conversation history with session support
//! - **Extensible**: Easy to add custom tools and behaviors

// Core modules
pub mod agent;
pub mod context;
pub mod hooks;
pub mod plan;
pub mod skills;
pub mod tools;

// Re-export commonly used types for convenience
pub use agent::{create_agent, AgentBuilder, AgentInstance, AgentType, Provider};
pub use context::{ContextManager, SerializableMessage, SessionMetadata};
pub use hooks::SessionIdHook;
pub use plan::{Plan, PlanManager, Task, TaskStatus};
pub use skills::{Skill, SkillManager};
pub use tools::{
    FileToolError, WrappedCreateDirectoryTool, WrappedDeleteFileTool, WrappedEditFileTool,
    WrappedExecuteBashCommandTool, WrappedGrepSearchTool, WrappedReadFileTool,
    WrappedScanCodebaseTool, WrappedUpdatePlanTool, WrappedWriteFileTool,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use super::agent::{create_agent, AgentBuilder, AgentInstance, AgentType, Provider};
    pub use super::context::{ContextManager, SerializableMessage, SessionMetadata};
    pub use super::hooks::SessionIdHook;
    pub use super::plan::{Plan, PlanManager, Task, TaskStatus};
    pub use super::skills::{Skill, SkillManager};
    pub use super::tools::FileToolError;
}
