use anyhow::{Context, Result};
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParams, InitializeResult, Tool},
    service::{RoleClient, RunningService},
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use serde_json::Value;
use std::borrow::Cow;
use tokio::process::Command;

/// MCP client connection wrapper
pub struct McpClient {
    /// The running service for communicating with the MCP server
    service: RunningService<RoleClient, ()>,
    /// Server information
    server_info: InitializeResult,
    /// Cached list of available tools
    tools: Vec<Tool>,
}

impl McpClient {
    /// Create a new MCP client by connecting to a server via child process
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute (e.g., "uvx", "npx")
    /// * `args` - Arguments for the command
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = McpClient::new("uvx", vec!["mcp-server-git"]).await?;
    /// ```
    pub async fn new(command: &str, args: Vec<String>) -> Result<Self> {
        // Create command
        let cmd = Command::new(command);

        // Configure the command with arguments
        let transport = TokioChildProcess::new(cmd.configure(|c| {
            for arg in args {
                c.arg(arg);
            }
        }))
        .context("Failed to create child process transport")?;

        // Connect to the server
        let service = ().serve(transport).await.context("Failed to connect to MCP server")?;

        // Get server information
        let server_info = service
            .peer_info()
            .cloned()
            .context("Failed to get server info")?;

        // List available tools
        let tools_result = service
            .list_tools(Default::default())
            .await
            .context("Failed to list tools from MCP server")?;

        Ok(Self {
            service,
            server_info,
            tools: tools_result.tools,
        })
    }

    /// Get server information
    pub fn server_info(&self) -> &InitializeResult {
        &self.server_info
    }

    /// Get list of available tools
    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }

    /// Call a tool on the MCP server
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Name of the tool to call
    /// * `arguments` - JSON arguments for the tool
    ///
    /// # Returns
    ///
    /// Returns the result from the tool execution
    pub async fn call_tool(&self, tool_name: &str, arguments: Option<Value>) -> Result<Value> {
        let result = self
            .service
            .call_tool(CallToolRequestParams {
                meta: None,
                name: Cow::Owned(tool_name.to_string()),
                arguments: arguments.and_then(|v| v.as_object().cloned()),
                task: None,
            })
            .await
            .context(format!("Failed to call tool: {}", tool_name))?;

        Ok(serde_json::to_value(result)?)
    }

    /// Refresh the list of available tools
    pub async fn refresh_tools(&mut self) -> Result<()> {
        let tools_result = self
            .service
            .list_tools(Default::default())
            .await
            .context("Failed to refresh tools list")?;

        self.tools = tools_result.tools;
        Ok(())
    }

    /// Close the connection to the MCP server
    pub async fn close(self) -> Result<()> {
        // The service will be dropped and cleaned up automatically
        self.service
            .cancel()
            .await
            .context("Failed to close the MCP server")?;
        Ok(())
    }
}
