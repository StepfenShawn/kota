use std::future::Future;
use std::pin::Pin;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Future returned by the object-safe [`ToolDyn`] trait.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Definition of a tool exposed to the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Error returned when a dynamic tool invocation fails.
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Tool call error: {0}")]
    ToolCallError(Box<dyn std::error::Error + Send + Sync>),
}

/// Statically dispatched tool trait.
///
/// Implementations keep their strongly typed arguments and output while the
/// blanket [`ToolDyn`] implementation adapts them to JSON for the model.
pub trait Tool: Send + Sync {
    const NAME: &'static str;

    type Error: std::error::Error + Send + Sync + 'static;
    type Args: DeserializeOwned + Send;
    type Output: Serialize + Send;

    fn definition(&self, prompt: String) -> impl Future<Output = ToolDefinition> + Send;

    fn call(
        &self,
        args: Self::Args,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;
}

/// Object-safe tool trait used by [`crate::kota_code::runtime::ToolRegistry`].
pub trait ToolDyn: Send + Sync {
    fn name(&self) -> String;

    fn definition<'a>(&'a self, prompt: String) -> BoxFuture<'a, ToolDefinition>;

    fn call<'a>(&'a self, args: String) -> BoxFuture<'a, Result<String, ToolError>>;
}

impl<T> ToolDyn for T
where
    T: Tool + 'static,
{
    fn name(&self) -> String {
        T::NAME.to_string()
    }

    fn definition<'a>(&'a self, prompt: String) -> BoxFuture<'a, ToolDefinition> {
        Box::pin(Tool::definition(self, prompt))
    }

    fn call<'a>(&'a self, args: String) -> BoxFuture<'a, Result<String, ToolError>> {
        Box::pin(async move {
            let parsed: T::Args =
                serde_json::from_str(&args).map_err(|e| ToolError::ToolCallError(Box::new(e)))?;
            let output = Tool::call(self, parsed)
                .await
                .map_err(|e| ToolError::ToolCallError(Box::new(e)))?;
            serde_json::to_string(&output).map_err(|e| ToolError::ToolCallError(Box::new(e)))
        })
    }
}
