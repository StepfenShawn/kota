use super::FileToolError;
use colored::*;
use regex::Regex;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use walkdir::WalkDir;

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

#[derive(Deserialize, Serialize)]
pub struct GrepSearchTool;

impl GrepSearchTool {
    fn is_text_file(path: &Path) -> bool {
        // Check file extension for common text file types
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            matches!(
                ext.as_str(),
                "rs" | "py"
                    | "js"
                    | "ts"
                    | "jsx"
                    | "tsx"
                    | "html"
                    | "css"
                    | "scss"
                    | "sass"
                    | "json"
                    | "xml"
                    | "yaml"
                    | "yml"
                    | "toml"
                    | "md"
                    | "txt"
                    | "csv"
                    | "sql"
                    | "sh"
                    | "bash"
                    | "zsh"
                    | "fish"
                    | "ps1"
                    | "bat"
                    | "cmd"
                    | "c"
                    | "cpp"
                    | "cc"
                    | "cxx"
                    | "h"
                    | "hpp"
                    | "hxx"
                    | "java"
                    | "kt"
                    | "scala"
                    | "go"
                    | "php"
                    | "rb"
                    | "swift"
                    | "dart"
                    | "r"
                    | "m"
                    | "mm"
                    | "pl"
                    | "pm"
                    | "lua"
                    | "vim"
                    | "el"
                    | "clj"
                    | "cljs"
                    | "hs"
                    | "ml"
                    | "fs"
                    | "fsx"
                    | "ex"
                    | "exs"
                    | "erl"
                    | "hrl"
                    | "nim"
                    | "cr"
                    | "d"
                    | "zig"
                    | "v"
                    | "jl"
                    | "rkt"
                    | "scm"
                    | "ss"
                    | "lisp"
                    | "lsp"
                    | "cl"
                    | "asd"
                    | "pro"
                    | "prolog"
                    | "dockerfile"
                    | "makefile"
                    | "mk"
                    | "cmake"
                    | "gradle"
                    | "sbt"
                    | "pom"
                    | "gemfile"
                    | "rakefile"
                    | "podfile"
                    | "cartfile"
                    | "brewfile"
                    | "vagrantfile"
                    | "gitignore"
                    | "gitattributes"
                    | "editorconfig"
                    | "eslintrc"
                    | "prettierrc"
                    | "babelrc"
                    | "tsconfig"
                    | "jsconfig"
                    | "webpack"
                    | "rollup"
                    | "vite"
                    | "package"
                    | "lock"
                    | "sum"
                    | "mod"
                    | "ini"
                    | "cfg"
                    | "conf"
                    | "config"
                    | "properties"
                    | "env"
                    | "example"
                    | "sample"
                    | "template"
                    | "stub"
                    | "proto"
                    | "graphql"
                    | "gql"
                    | "schema"
                    | "prisma"
                    | "ddl"
                    | "dml"
                    | "hql"
                    | "cql"
                    | "psql"
                    | "mysql"
                    | "sqlite"
                    | "db"
                    | "log"
                    | "out"
                    | "err"
                    | "trace"
                    | "debug"
                    | "info"
                    | "warn"
                    | "error"
                    | "fatal"
            )
        } else {
            // Check for files without extensions that are commonly text files
            if let Some(file_name) = path.file_name() {
                let name = file_name.to_string_lossy().to_lowercase();
                matches!(
                    name.as_str(),
                    "readme"
                        | "license"
                        | "changelog"
                        | "authors"
                        | "contributors"
                        | "makefile"
                        | "dockerfile"
                        | "vagrantfile"
                        | "gemfile"
                        | "rakefile"
                        | "podfile"
                        | "cartfile"
                        | "brewfile"
                        | "procfile"
                        | "justfile"
                        | "taskfile"
                        | "buildfile"
                        | "gradlew"
                        | "mvnw"
                )
            } else {
                false
            }
        }
    }

    fn should_skip_directory(dir_name: &str) -> bool {
        matches!(
            dir_name,
            "target"
                | "node_modules"
                | "__pycache__"
                | ".git"
                | ".svn"
                | ".hg"
                | ".bzr"
                | "dist"
                | "build"
                | "out"
                | "bin"
                | "obj"
                | ".vscode"
                | ".idea"
                | ".vs"
                | "coverage"
                | ".nyc_output"
                | ".pytest_cache"
                | ".tox"
                | "venv"
                | "env"
                | ".env"
                | "vendor"
                | "deps"
                | "_build"
                | ".elixir_ls"
                | ".mix"
                | "tmp"
                | "temp"
                | "cache"
                | ".cache"
                | "logs"
                | "log"
                | ".DS_Store"
                | "Thumbs.db"
        )
    }

    fn search_in_file(
        &self,
        file_path: &Path,
        regex: &Regex,
        max_results: usize,
        current_matches: usize,
    ) -> Result<Vec<SearchMatch>, FileToolError> {
        if current_matches >= max_results {
            return Ok(Vec::new());
        }

        let content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(_) => return Ok(Vec::new()), // Skip files that can't be read as text
        };

        let mut matches = Vec::new();
        let file_path_str = file_path.to_string_lossy().to_string();

        for (line_number, line) in content.lines().enumerate() {
            if current_matches + matches.len() >= max_results {
                break;
            }

            if let Some(mat) = regex.find(line) {
                matches.push(SearchMatch {
                    file_path: file_path_str.clone(),
                    line_number: line_number + 1, // 1-indexed line numbers
                    line_content: line.to_string(),
                    match_start: mat.start(),
                    match_end: mat.end(),
                });
            }
        }

        Ok(matches)
    }
}

impl Tool for GrepSearchTool {
    const NAME: &'static str = "grep_search";

    type Error = FileToolError;
    type Args = GrepSearchArgs;
    type Output = GrepSearchOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "grep_search".to_string(),
            description: "Search for text patterns in files within a directory tree using regular expressions. Recursively searches through text files and returns matching lines with context.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "root_path": {
                        "type": "string",
                        "description": "The root directory path to search in. Examples: '.', 'src', '/path/to/project'"
                    },
                    "query": {
                        "type": "string",
                        "description": "The search pattern (regular expression). Examples: 'function', 'TODO|FIXME', 'async fn \\w+', 'import.*from'"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of matches to return (default: 100). Use smaller values for broad searches.",
                        "default": 100
                    }
                },
                "required": ["root_path", "query"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let root_path = &args.root_path;
        let query = &args.query;
        let max_results = args.max_results.unwrap_or(100);

        let path = Path::new(root_path);
        if !path.exists() {
            return Err(FileToolError::FileNotFound(root_path.clone()));
        }

        // Compile the regex pattern
        let regex = match Regex::new(query) {
            Ok(regex) => regex,
            Err(e) => {
                return Ok(GrepSearchOutput {
                    root_path: root_path.clone(),
                    query: query.clone(),
                    matches: Vec::new(),
                    total_matches: 0,
                    files_searched: 0,
                    success: false,
                    message: format!("Invalid regex pattern: {}", e),
                });
            }
        };

        let mut all_matches = Vec::new();
        let mut files_searched = 0;

        // Walk through the directory tree
        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    if let Some(dir_name) = e.file_name().to_str() {
                        !Self::should_skip_directory(dir_name) && !dir_name.starts_with('.')
                    } else {
                        false
                    }
                } else {
                    true
                }
            })
        {
            if all_matches.len() >= max_results {
                break;
            }

            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue, // Skip entries we can't access
            };

            if entry.file_type().is_file() && Self::is_text_file(entry.path()) {
                files_searched += 1;
                match self.search_in_file(entry.path(), &regex, max_results, all_matches.len()) {
                    Ok(mut matches) => {
                        all_matches.append(&mut matches);
                    }
                    Err(_) => continue, // Skip files that cause errors
                }
            }
        }

        let total_matches = all_matches.len();
        let success = true;
        let message = if total_matches == 0 {
            format!(
                "No matches found for '{}' in {} files",
                query, files_searched
            )
        } else if total_matches >= max_results {
            format!(
                "Found {} matches (limit reached) for '{}' in {} files",
                total_matches, query, files_searched
            )
        } else {
            format!(
                "Found {} matches for '{}' in {} files",
                total_matches, query, files_searched
            )
        };

        Ok(GrepSearchOutput {
            root_path: root_path.clone(),
            query: query.clone(),
            matches: all_matches,
            total_matches,
            files_searched,
            success,
            message,
        })
    }
}

// Wrapper tool with visual feedback
#[derive(Deserialize, Serialize)]
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
    const NAME: &'static str = "grep_search";

    type Error = FileToolError;
    type Args = <GrepSearchTool as Tool>::Args;
    type Output = <GrepSearchTool as Tool>::Output;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!(
            "\n{} {} {}",
            "🔧".bright_blue(),
            "Tool:".bright_white(),
            format!("Searching for '{}' in '{}'", args.query, args.root_path).cyan()
        );
        io::stdout().flush().unwrap();

        let result = self.inner.call(args).await;

        match &result {
            Ok(output) => {
                println!(
                    "{} {}",
                    "✅".bright_green(),
                    "Search completed.".bright_green()
                );
                println!("{}", output.message.bright_white());

                if !output.matches.is_empty() {
                    println!("\n{}", "🔍 Search Results:".bright_white());
                    for (i, search_match) in output.matches.iter().enumerate() {
                        if i >= 20 {
                            // Limit console output to first 20 matches
                            println!(
                                "  {} (showing first 20 of {} matches)",
                                "...".dimmed(),
                                output.total_matches
                            );
                            break;
                        }

                        println!(
                            "  {}:{}:{} {}",
                            search_match.file_path.bright_cyan(),
                            search_match.line_number.to_string().bright_yellow(),
                            search_match.match_start.to_string().dimmed(),
                            search_match.line_content.trim()
                        );
                    }
                }

                println!(
                    "\n{} {} matches in {} files",
                    "📊".bright_blue(),
                    output.total_matches.to_string().bright_cyan(),
                    output.files_searched.to_string().bright_cyan()
                );
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
        println!(); // Add empty line
        io::stdout().flush().unwrap();

        result
    }
}
