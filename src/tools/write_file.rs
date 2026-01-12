use super::FileToolError;
use colored::*;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

#[derive(Deserialize)]
pub struct WriteFileArgs {
    pub file_path: String,
    pub content: String,
}

#[derive(Serialize, Debug)]
pub struct WriteFileOutput {
    pub file_path: String,
    pub bytes_written: u64,
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize, Serialize)]
pub struct WriteFileTool;

impl Tool for WriteFileTool {
    const NAME: &'static str = "write_file";

    type Error = FileToolError;
    type Args = WriteFileArgs;
    type Output = WriteFileOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "write_file".to_string(),
            description: "Write content to a file, creating it if it doesn't exist or overwriting it completely if it does. Creates parent directories if needed.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "The path to the file to write (relative or absolute). Examples: 'output.txt', 'src/new_file.rs', '/path/to/file.txt'"
                    },
                    "content": {
                        "type": "string",
                        "description": "The content to write to the file. This will completely replace any existing content."
                    }
                },
                "required": ["file_path", "content"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let file_path = &args.file_path;
        let content = &args.content;
        let path = Path::new(file_path);

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // Write the content to the file
        match fs::write(file_path, content) {
            Ok(()) => {
                let bytes_written = content.len() as u64;
                Ok(WriteFileOutput {
                    file_path: file_path.clone(),
                    bytes_written,
                    success: true,
                    message: format!(
                        "Successfully wrote {} bytes to '{}'",
                        bytes_written, file_path
                    ),
                })
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    Err(FileToolError::PermissionDenied(file_path.clone()))
                }
                _ => Err(FileToolError::Io(e)),
            },
        }
    }
}
#[derive(Deserialize, Serialize)]
pub struct WrappedWriteFileTool {
    inner: WriteFileTool,
}

impl WrappedWriteFileTool {
    pub fn new() -> Self {
        Self {
            inner: WriteFileTool,
        }
    }
}

impl Tool for WrappedWriteFileTool {
    const NAME: &'static str = "write_file";

    type Error = FileToolError;
    type Args = <WriteFileTool as Tool>::Args;
    type Output = <WriteFileTool as Tool>::Output;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 显示工具调用开始
        println!(
            "\n{} {} {}",
            "🔧".bright_blue(),
            "Tool:".bright_white(),
            format!("Writing to file '{}'", args.file_path).cyan()
        );
        io::stdout().flush().unwrap();

        // 调用实际工具
        let result = self.inner.call(args).await;

        // 显示工具调用结果
        match &result {
            Ok(_output) => {
                println!("{} {}", "✅".bright_green(), "Done.".bright_green());
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
        println!();
        io::stdout().flush().unwrap();

        result
    }
}
