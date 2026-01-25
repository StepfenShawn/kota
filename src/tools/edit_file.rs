use super::FileToolError;
use colored::*;
use patch_apply::{apply, Patch};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct EditFileArgs {
    pub file_path: String,
    pub patch: String,
}

#[derive(Serialize, Debug)]
pub struct EditFileOutput {
    pub file_path: String,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize, Serialize, Default)]
pub struct EditFileTool;

impl Tool for EditFileTool {
    const NAME: &'static str = "edit_file";

    type Error = FileToolError;
    type Args = EditFileArgs;
    type Output = EditFileOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "edit_file".to_string(),
            description: "Apply a unified diff patch to a file. This is efficient for making small, targeted changes to existing files without rewriting the entire content.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "The path to the file to edit (relative or absolute). The file must exist. Examples: 'src/main.rs', 'README.md'"
                    },
                    "patch": {
                        "type": "string",
                        "description": "A unified diff patch string in standard format. CRITICAL REQUIREMENTS:\n1. MUST start with '--- a/filename' header line\n2. MUST have '+++ b/filename' as second line\n3. MUST have hunk header: '@@ -old_start,old_count +new_start,new_count @@'\n4. Context lines MUST start with ' ' (space)\n5. Removed lines MUST start with '-'\n6. Added lines MUST start with '+'\n7. EVERY line (including the last) MUST end with '\\n'\n\nExample:\n--- a/main.rs\n+++ b/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"Hello, world!\");\n     // existing code\n }\n"
                    }
                },
                "required": ["file_path", "patch"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let file_path = &args.file_path;
        let patch_str = &args.patch;
        let path = Path::new(file_path);

        // Check if file exists
        if !path.exists() {
            return Err(FileToolError::FileNotFound(file_path.clone()));
        }

        // Check if it's actually a file (not a directory)
        if !path.is_file() {
            return Err(FileToolError::NotAFile(file_path.clone()));
        }

        // Read the current file content
        let current_content = fs::read_to_string(file_path)?;

        // Ensure patch_str ends with a newline
        let patch_str_normalized = if !patch_str.ends_with('\n') {
            format!("{}\n", patch_str)
        } else {
            patch_str.to_string()
        };

        // Parse the patch
        let patch = Patch::from_single(&patch_str_normalized).map_err(|e| {
            FileToolError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse patch: {}", e),
            ))
        })?;

        // Apply the patch using patch_apply::apply
        let patched_content = apply(current_content, patch);

        // Calculate statistics
        let original_lines: Vec<&str> = args.patch.lines().collect();
        let mut lines_added = 0usize;
        let mut lines_removed = 0usize;

        for line in original_lines {
            if line.starts_with('+') && !line.starts_with("+++") {
                lines_added += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                lines_removed += 1;
            }
        }

        // Write the modified content back to the file
        match fs::write(file_path, &patched_content) {
            Ok(()) => Ok(EditFileOutput {
                file_path: file_path.clone(),
                lines_added,
                lines_removed,
                success: true,
                message: format!(
                    "Successfully applied patch to '{}': +{} lines, -{} lines",
                    file_path, lines_added, lines_removed
                ),
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
pub struct WrappedEditFileTool {
    inner: EditFileTool,
}

impl WrappedEditFileTool {
    pub fn new() -> Self {
        Self {
            inner: EditFileTool,
        }
    }
}

impl Tool for WrappedEditFileTool {
    const NAME: &'static str = "edit_file";

    type Error = FileToolError;
    type Args = <EditFileTool as Tool>::Args;
    type Output = <EditFileTool as Tool>::Output;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("\n{} Edit({})", "●".bright_green(), args.file_path);

        let result = self.inner.call(args).await;

        match &result {
            Ok(output) => {
                println!(
                    "  └─ {} (+{} lines, -{} lines)",
                    format!("Patched '{}'", output.file_path).dimmed(),
                    output.lines_added.to_string().green(),
                    output.lines_removed.to_string().red()
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
