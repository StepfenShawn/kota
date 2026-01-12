# Kota
A lightweight AI code agent in Rust.

## Setup

### Environment Configuration

Before running Kota, you need to create a `.env` file with your API configuration:

1. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```

2. Edit the `.env` file and configure your API settings:
   ```env
   API_KEY=your-api-key-here
   MODEL_NAME=deepseek-chat
   ```

   - `API_KEY`: Your LLM provider API key
   - `MODEL_NAME`: The model to use (e.g., deepseek-chat, gpt-4o)

### Running Kota

```bash
cargo run -r
```

## Available Tools

Kota comes with a comprehensive set of file system and development tools:

### File Operations
- **`read_file`** - Read the contents of a file from the filesystem
- **`write_file`** - Write content to a file, creating it if it doesn't exist or overwriting completely
- **`edit_file`** - Apply unified diff patches to files for targeted changes
- **`delete_file`** - Delete a file from the filesystem

### Directory Operations
- **`create_directory`** - Create directories and all necessary parent directories
- **`scan_codebase`** - Scan and display the structure of a codebase directory tree

### Search Operations
- **`grep_search`** - Search for text patterns in files using regular expressions with recursive directory traversal

### System Operations
- **`execute_bash_command`** - Execute bash/cmd commands and return output (use with caution)

Each tool provides detailed feedback during execution and handles common error cases like permission issues and missing files.
