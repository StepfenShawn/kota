//! # Kota - A lightweight ai code agent in Rust.
//!
//! ## Modules
//!
//! - `kota_code`: Core AI code agent functionality with LLM integration, file operations,
//!   code analysis, and task management.
//!

pub mod kota_code;

// Re-export commonly used types for convenience
pub use kota_code::{
    create_agent, AgentBuilder, AgentInstance, AgentType, ContextManager, Plan, PlanManager,
    Provider, SerializableMessage, SessionIdHook, SessionMetadata, Skill, SkillManager, Task,
    TaskStatus,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::kota_code::prelude::*;
}
