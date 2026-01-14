// Library interface for Kota
// This exposes internal modules for testing and potential library usage

pub mod agent;
pub mod context;
pub mod hooks;
pub mod tools;

// Re-export commonly used types for convenience
pub use agent::{create_agent, AgentType};
pub use context::{ContextManager, SerializableMessage};
pub use hooks::SessionIdHook;
pub use tools::FileToolError;
