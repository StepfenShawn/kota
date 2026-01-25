use super::FileToolError;
use colored::*;
use ignore::WalkBuilder;
use regex::Regex;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Deserialize)]
pub struct GrepSearchArgs {
    pub root_path: String,
    pub query: String,
    pub max_results: Option<usize>,
}

#[derive(Serialize, Clone, Debug)]
pub struct SearchMatch {
    pub file_path: String,
    pub line_number: usize,
    pub line_content: String,
    pub match_start: usize,
    pub match_end: usize,
}

#[derive(Serialize, Debug)]
pub struct GrepSearchOutput {
    pub root_path: String,
    pub query: String,
    pub matches: Vec<SearchMatch>,
    pub total_matches: usize,
    pub files_searched: usize,
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize, Serialize, Default)]
pub struct GrepSearchTool;

impl Tool for GrepSearchTool {
    const NAME: &'static str = "grep_find";

    type Error = FileToolError;
    type Args = GrepSearchArgs;
    type Output = GrepSearchOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "grep_find".to_string(),
            description:
                "Search for text patterns in files using regex. Respects .gitignore automatically."
                    .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "root_path": {"type": "string", "description": "Root directory to search"},
                    "query": {"type": "string", "description": "Regex pattern to search for"},
                    "max_results": {"type": "integer", "description": "Max matches (default: 100)", "default": 100}
                },
                "required": ["root_path", "query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let max_results = args.max_results.unwrap_or(100);

        let regex = Regex::new(&args.query)
            .map_err(|e| FileToolError::InvalidInput(format!("Invalid regex: {}", e)))?;

        let mut matches = Vec::new();
        let mut files_searched = 0;

        // Use ignore crate for smart file walking (respects .gitignore, etc.)
        for result in WalkBuilder::new(&args.root_path)
            .hidden(false) // Include hidden files
            .git_ignore(true) // Respect .gitignore
            .build()
        {
            if matches.len() >= max_results {
                break;
            }

            let entry = match result {
                Ok(entry) => entry,
                Err(_) => continue, // Skip entries we can't access
            };

            if entry.file_type().is_some_and(|ft| ft.is_file()) {
                files_searched += 1;

                if let Ok(content) = fs::read_to_string(entry.path()) {
                    for (line_num, line) in content.lines().enumerate() {
                        if let Some(mat) = regex.find(line) {
                            matches.push(SearchMatch {
                                file_path: entry.path().to_string_lossy().to_string(),
                                line_number: line_num + 1,
                                line_content: line.to_string(),
                                match_start: mat.start(),
                                match_end: mat.end(),
                            });

                            if matches.len() >= max_results {
                                break;
                            }
                        }
                    }
                }
            }
        }

        let message = format!(
            "Found {} matches in {} files",
            matches.len(),
            files_searched
        );

        Ok(GrepSearchOutput {
            root_path: args.root_path,
            query: args.query,
            total_matches: matches.len(),
            matches,
            files_searched,
            success: true,
            message,
        })
    }
}

// Wrapper with visual feedback
#[derive(Deserialize, Serialize, Default)]
pub struct WrappedGrepSearchTool {
    inner: GrepSearchTool,
}

impl WrappedGrepSearchTool {
    pub fn new() -> Self {
        Self {
            inner: GrepSearchTool,
        }
    }
}

impl Tool for WrappedGrepSearchTool {
    const NAME: &'static str = "grep_find";
    type Error = FileToolError;
    type Args = <GrepSearchTool as Tool>::Args;
    type Output = <GrepSearchTool as Tool>::Output;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("\n{} Search({})", "●".bright_green(), args.query);

        let result = self.inner.call(args).await;

        match &result {
            Ok(output) => {
                if output.total_matches > 0 {
                    let preview = &output.matches[0].line_content;
                    let preview = if preview.len() > 50 {
                        format!("{}...", &preview[..50])
                    } else {
                        preview.clone()
                    };
                    println!(
                        "  └─ {} ... +{} matches",
                        preview.dimmed(),
                        output.total_matches
                    );
                } else {
                    println!("  └─ {}", "No matches found".dimmed());
                }
            }
            Err(e) => println!("  └─ {}", format!("Error: {}", e).red()),
        }

        result
    }
}
