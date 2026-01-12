use super::FileToolError;
use colored::*;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

#[derive(Deserialize)]
pub struct CreateDirectoryArgs {
    pub dir_path: String,
}

#[derive(Serialize, Debug)]
pub struct CreateDirectoryOutput {
    pub dir_path: String,
    pub success: bool,
    pub message: String,
    pub created_parents: bool,
}

#[derive(Deserialize, Serialize)]
pub struct CreateDirectoryTool;

impl Tool for CreateDirectoryTool {
    const NAME: &'static str = "create_directory";

    type Error = FileToolError;
    type Args = CreateDirectoryArgs;
    type Output = CreateDirectoryOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "create_directory".to_string(),
            description: "Create a directory and all necessary parent directories. If the directory already exists, the operation succeeds.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "dir_path": {
                        "type": "string",
                        "description": "The path of the directory to create (relative or absolute). Examples: 'new_folder', 'src/components', '/path/to/new/dir'"
                    }
                },
                "required": ["dir_path"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let dir_path = &args.dir_path;
        let path = Path::new(dir_path);

        // Check if directory already exists
        if path.exists() {
            if path.is_dir() {
                return Ok(CreateDirectoryOutput {
                    dir_path: dir_path.clone(),
                    success: true,
                    message: format!("Directory '{}' already exists", dir_path),
                    created_parents: false,
                });
            } else {
                return Err(FileToolError::NotAFile(format!(
                    "Path '{}' exists but is not a directory",
                    dir_path
                )));
            }
        }

        // Check if we need to create parent directories
        let needs_parents = path.parent().map_or(false, |parent| !parent.exists());

        // Create the directory and all parent directories
        match fs::create_dir_all(dir_path) {
            Ok(()) => Ok(CreateDirectoryOutput {
                dir_path: dir_path.clone(),
                success: true,
                message: if needs_parents {
                    format!(
                        "Successfully created directory '{}' and parent directories",
                        dir_path
                    )
                } else {
                    format!("Successfully created directory '{}'", dir_path)
                },
                created_parents: needs_parents,
            }),
            Err(e) => match e.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    Err(FileToolError::PermissionDenied(dir_path.clone()))
                }
                _ => Err(FileToolError::Io(e)),
            },
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct WrappedCreateDirectoryTool {
    inner: CreateDirectoryTool,
}

impl WrappedCreateDirectoryTool {
    pub fn new() -> Self {
        Self {
            inner: CreateDirectoryTool,
        }
    }
}

impl Tool for WrappedCreateDirectoryTool {
    const NAME: &'static str = "create_directory";

    type Error = FileToolError;
    type Args = <CreateDirectoryTool as Tool>::Args;
    type Output = <CreateDirectoryTool as Tool>::Output;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 显示工具调用开始
        println!(
            "\n{} {} {}",
            "🔧".bright_blue(),
            "Tool:".bright_white(),
            format!("Creating directory '{}'", args.dir_path).cyan()
        );
        io::stdout().flush().unwrap();

        // 调用实际工具
        let result = self.inner.call(args).await;

        // 显示工具调用结果
        match &result {
            Ok(output) => {
                println!("{} {}", "✅".bright_green(), "Done.".bright_green());
                if output.created_parents {
                    println!(
                        "{}",
                        "📁 Created parent directories as needed.".bright_blue()
                    );
                }
            }
            Err(e) => {
                println!(
                    "{} {} {}",
                    "❌".bright_red(),
                    "Error:".bright_red(),
                    e.to_string().red()
                );
            }
        }
        println!(); // 添加空行
        io::stdout().flush().unwrap();

        result
    }
}
