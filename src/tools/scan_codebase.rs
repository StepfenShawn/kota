use super::FileToolError;
use colored::*;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct ScanCodebaseArgs {
    pub root_path: String,
}

#[derive(Serialize, Debug)]
pub struct ScanCodebaseOutput {
    pub root_path: String,
    pub structure: String,
    pub total_files: usize,
    pub total_directories: usize,
}

#[derive(Deserialize, Serialize, Default)]
pub struct ScanCodebaseTool;

impl ScanCodebaseTool {
    fn scan_directory(
        path: &Path,
        prefix: &str,
        max_depth: usize,
        current_depth: usize,
    ) -> Result<(String, usize, usize), FileToolError> {
        if current_depth > max_depth {
            return Ok((String::new(), 0, 0));
        }

        let mut result = String::new();
        let mut file_count = 0;
        let mut dir_count = 0;

        let entries = fs::read_dir(path)?;
        let mut entries: Vec<_> = entries.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by(|a, b| {
            let a_is_dir = a.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let b_is_dir = b.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        for (i, entry) in entries.iter().enumerate() {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Skip hidden files and common ignore patterns
            if file_name_str.starts_with('.')
                || file_name_str == "target"
                || file_name_str == "node_modules"
                || file_name_str == "__pycache__"
            {
                continue;
            }

            let is_last = i == entries.len() - 1;
            let current_prefix = if is_last { "└── " } else { "├── " };
            let next_prefix = if is_last { "    " } else { "│   " };

            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                result.push_str(&format!("{}{}{}\n", prefix, current_prefix, file_name_str));
                dir_count += 1;

                let (sub_result, sub_files, sub_dirs) = Self::scan_directory(
                    &entry.path(),
                    &format!("{}{}", prefix, next_prefix),
                    max_depth,
                    current_depth + 1,
                )?;
                result.push_str(&sub_result);
                file_count += sub_files;
                dir_count += sub_dirs;
            } else {
                result.push_str(&format!("{}{}{}\n", prefix, current_prefix, file_name_str));
                file_count += 1;
            }
        }

        Ok((result, file_count, dir_count))
    }
}

impl Tool for ScanCodebaseTool {
    const NAME: &'static str = "scan_codebase";

    type Error = FileToolError;
    type Args = ScanCodebaseArgs;
    type Output = ScanCodebaseOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "scan_codebase".to_string(),
            description: "Scan and display the structure of a codebase directory tree. Shows files and directories in a tree format.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "root_path": {
                        "type": "string",
                        "description": "The root directory path to scan. Examples: '.', 'src', '/path/to/project'"
                    }
                },
                "required": ["root_path"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let root_path = &args.root_path;
        let path = Path::new(root_path);

        if !path.exists() {
            return Err(FileToolError::FileNotFound(root_path.clone()));
        }

        if !path.is_dir() {
            return Err(FileToolError::NotAFile(format!(
                "Path '{}' is not a directory",
                root_path
            )));
        }

        let mut structure = format!(
            "{}\n",
            path.file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new(root_path))
                .to_string_lossy()
        );
        let (tree_result, file_count, dir_count) = Self::scan_directory(path, "", 5, 0)?;
        structure.push_str(&tree_result);

        Ok(ScanCodebaseOutput {
            root_path: root_path.clone(),
            structure,
            total_files: file_count,
            total_directories: dir_count,
        })
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct WrappedScanCodebaseTool {
    inner: ScanCodebaseTool,
}

impl WrappedScanCodebaseTool {
    pub fn new() -> Self {
        Self {
            inner: ScanCodebaseTool,
        }
    }
}

impl Tool for WrappedScanCodebaseTool {
    const NAME: &'static str = "scan_codebase";

    type Error = FileToolError;
    type Args = <ScanCodebaseTool as Tool>::Args;
    type Output = <ScanCodebaseTool as Tool>::Output;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("\n{} Scan({})", "●".bright_green(), args.root_path);

        let result = self.inner.call(args).await;

        match &result {
            Ok(output) => {
                println!(
                    "  └─ {} files, {} directories",
                    output.total_files.to_string().dimmed(),
                    output.total_directories.to_string().dimmed()
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
