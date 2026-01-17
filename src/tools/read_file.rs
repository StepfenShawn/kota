use super::FileToolError;
use colored::*;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct ReadFileArgs {
    pub file_path: String,
}

#[derive(Serialize, Debug)]
pub struct ReadFileOutput {
    pub content: String,
    pub file_path: String,
    pub size_bytes: u64,
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize, Serialize)]
pub struct ReadFileTool;

impl Tool for ReadFileTool {
    const NAME: &'static str = "read_file";

    type Error = FileToolError;
    type Args = ReadFileArgs;
    type Output = ReadFileOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "read_file".to_string(),
            description: "Read the contents of a file from the filesystem. Supports text files and returns the content as a string.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "The path to the file to read (relative or absolute). Examples: 'README.md', 'src/main.rs', '/path/to/file.txt'"
                    }
                },
                "required": ["file_path"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let file_path = &args.file_path;
        let path = Path::new(file_path);

        // Check if file exists
        if !path.exists() {
            return Err(FileToolError::FileNotFound(file_path.clone()));
        }

        // Check if it's actually a file (not a directory)
        if !path.is_file() {
            return Err(FileToolError::NotAFile(file_path.clone()));
        }

        // Try to read the file
        match fs::read_to_string(file_path) {
            Ok(content) => {
                // Get file metadata for size
                let metadata = fs::metadata(file_path)?;
                let size_bytes = metadata.len();

                Ok(ReadFileOutput {
                    content,
                    file_path: file_path.clone(),
                    size_bytes,
                    success: true,
                    message: format!(
                        "Successfully read {} bytes from '{}'",
                        size_bytes, file_path
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
// 在工具调用前后显示信息
#[derive(Deserialize, Serialize)]
pub struct WrappedReadFileTool {
    inner: ReadFileTool,
}

impl WrappedReadFileTool {
    pub fn new() -> Self {
        Self {
            inner: ReadFileTool,
        }
    }
}

impl Tool for WrappedReadFileTool {
    const NAME: &'static str = "read_file";

    type Error = FileToolError;
    type Args = <ReadFileTool as Tool>::Args;
    type Output = <ReadFileTool as Tool>::Output;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!();
        println!("{} {}({})", "●".bright_green(), "Read", args.file_path);

        let result = self.inner.call(args).await;

        match &result {
            Ok(output) => {
                // 对于读取文件，显示行数和预览
                let line_count = output.content.lines().count();
                let first_line = output.content.lines().next().unwrap_or("");
                let preview = if first_line.len() > 50 {
                    format!("{}...", &first_line[..50])
                } else {
                    first_line.to_string()
                };
                println!(
                    "  └─ {}| {} ... +{} lines",
                    "1".dimmed(),
                    preview.dimmed(),
                    line_count
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
