use super::FileToolError;
use colored::*;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Deserialize)]
pub struct ExecuteBashCommandArgs {
    pub command: String,
}

#[derive(Serialize, Debug)]
pub struct ExecuteBashCommandOutput {
    pub command: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct ExecuteBashCommandTool;

impl Tool for ExecuteBashCommandTool {
    const NAME: &'static str = "exec_cmd";

    type Error = FileToolError;
    type Args = ExecuteBashCommandArgs;
    type Output = ExecuteBashCommandOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "exec_cmd".to_string(),
            description: "Execute a bash command and return the output. Use with caution as this can modify the system.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The bash command to execute. Examples: 'ls -la', 'git status', 'cargo build'"
                    }
                },
                "required": ["command"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let command = &args.command;

        // Execute the command using cmd on Windows or bash on Unix
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", command]).output()
        } else {
            Command::new("bash").args(["-c", command]).output()
        };

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let success = output.status.success();
                let exit_code = output.status.code();

                Ok(ExecuteBashCommandOutput {
                    command: command.clone(),
                    success,
                    stdout,
                    stderr,
                    exit_code,
                })
            }
            Err(e) => Err(FileToolError::Io(e)),
        }
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct WrappedExecuteBashCommandTool {
    inner: ExecuteBashCommandTool,
}

impl WrappedExecuteBashCommandTool {
    pub fn new() -> Self {
        Self {
            inner: ExecuteBashCommandTool,
        }
    }
}

impl Tool for WrappedExecuteBashCommandTool {
    const NAME: &'static str = "exec_cmd";

    type Error = FileToolError;
    type Args = <ExecuteBashCommandTool as Tool>::Args;
    type Output = <ExecuteBashCommandTool as Tool>::Output;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("\n{} Exec({})", "●".bright_green(), args.command);

        let result = self.inner.call(args).await;

        match &result {
            Ok(output) => {
                if output.success {
                    let stdout_lines = output.stdout.lines().count();
                    if stdout_lines > 0 {
                        println!(
                            "  └─ {} ... +{} lines output",
                            "Command succeeded".dimmed(),
                            stdout_lines
                        );
                    } else {
                        println!("  └─ {}", "Command succeeded".dimmed());
                    }
                } else {
                    let stderr_lines = output.stderr.lines().count();
                    println!(
                        "  └─ {} (exit: {})",
                        format!("Command failed, {} lines stderr", stderr_lines).red(),
                        output.exit_code.unwrap_or(-1)
                    );
                }
            }
            Err(e) => {
                println!("  └─ {}", format!("Error: {}", e).red());
            }
        }
        println!();
        result
    }
}
