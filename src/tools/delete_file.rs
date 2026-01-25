use super::FileToolError;
use colored::*;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct DeleteFileArgs {
    pub file_path: String,
}

#[derive(Serialize, Debug)]
pub struct DeleteFileOutput {
    pub file_path: String,
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize, Serialize, Default)]
pub struct DeleteFileTool;

impl Tool for DeleteFileTool {
    const NAME: &'static str = "delete_file";

    type Error = FileToolError;
    type Args = DeleteFileArgs;
    type Output = DeleteFileOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "delete_file".to_string(),
            description: "Delete a file from the filesystem. The file must exist and be a regular file (not a directory).".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "The path to the file to delete (relative or absolute). Examples: 'temp.txt', 'src/old_file.rs', '/path/to/file.txt'"
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

        // Try to delete the file
        match fs::remove_file(file_path) {
            Ok(()) => Ok(DeleteFileOutput {
                file_path: file_path.clone(),
                success: true,
                message: format!("Successfully deleted file '{}'", file_path),
            }),
            Err(e) => match e.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    Err(FileToolError::PermissionDenied(file_path.clone()))
                }
                _ => Err(FileToolError::Io(e)),
            },
        }
    }
}
#[derive(Deserialize, Serialize, Default)]
pub struct WrappedDeleteFileTool {
    inner: DeleteFileTool,
}

impl WrappedDeleteFileTool {
    pub fn new() -> Self {
        Self {
            inner: DeleteFileTool,
        }
    }
}

impl Tool for WrappedDeleteFileTool {
    const NAME: &'static str = "delete_file";

    type Error = FileToolError;
    type Args = <DeleteFileTool as Tool>::Args;
    type Output = <DeleteFileTool as Tool>::Output;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("\n{} Delete({})", "●".bright_green(), args.file_path);

        let result = self.inner.call(args).await;

        match &result {
            Ok(_output) => {
                println!("  └─ {}", "File deleted".dimmed());
            }
            Err(e) => {
                println!("  └─ {}", format!("Error: {}", e).red());
            }
        }
        println!();
        result
    }
}
