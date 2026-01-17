use super::FileToolError;
use colored::*;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::fs;
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
        println!();
        println!("{} {}({})", "●".bright_green(), "Write", args.file_path);

        // Store line count before moving args
        let line_count = args.content.lines().count();

        let result = self.inner.call(args).await;

        match &result {
            Ok(output) => {
                println!(
                    "  └─ {} bytes written, {} lines",
                    output.bytes_written.to_string().dimmed(),
                    line_count.to_string().dimmed()
                );
            }
            Err(e) => {
                println!("  └─ {}", format!("Error: {}", e).red());
            }
        }
        println!();
        result
    }
}
