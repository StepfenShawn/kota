// Library interface for Kota
// This exposes internal modules for testing and potential library usage

pub mod agent;
pub mod tools;

// Re-export commonly used types for convenience
pub use agent::{AgentType, create_agent};
pub use tools::FileToolError;