<a href="https://docs.rs/kota/latest/kota"><img src="https://img.shields.io/badge/docs-API Reference-dca282.svg" /></a> 
<a href="https://crates.io/crates/kota"><img src="https://img.shields.io/crates/v/kota.svg?color=dca282" /></a>  &nbsp;
# Kota
A lightweight AI code agent in Rust:  

![kota_screenshot](assert/screenshort.jpg)

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
   - `MODEL_NAME`: The model to use (see supported models below)

## Install
```
cargo install kota
```
and then start it by:  
```
kota
```
enjoy it!

## Supported Models

Kota supports various LLM providers:

- **OpenAI Compatible** - (e.g., gpt-4o, gpt-4o-mini, gpt-4-turbo, gpt-3.5-turbo...)
- **DeepSeek** - (e.g., deepseek-chat, deepseek-coder...)
- **Anthropic Claude** (e.g., claude-3-5-sonnet, claude-4-opus...)

### Other Providers
- Any OpenAI-compatible API endpoint can be configured
- Local models via Ollama or similar services

## CLI Features

### Interactive Commands

Kota provides an interactive CLI with the following commands:

- `/quit` or `/exit` - Exit the application
- `/config` - Show current model configuration
- `/help` - Show available commands
- `/history` - Show conversation history
- `/load <session_id>` - Load specific session
- `/list` - List all sessions
- `/delete <session_id>` - Delete a specific session

### Tab Completion

Kota supports intelligent tab completion for commands:

- Type partial commands (e.g., `/h`) and press **Tab** to auto-complete
- Commands are highlighted in green when recognized
- Use **Ctrl+C** to exit or **Ctrl+D** for EOF

**Example usage:**
```
❯ /h<Tab>        # Completes to /help
❯ /hi<Tab>       # Completes to /history
```

## Current architecture of Kota
![architecture](assert/architecture.png)

## Available Tools

Kota comes with a comprehensive set of file system and development tools:

| Category | Tool | Description |
|----------|------|-------------|
| **File Operations** | `read_file` | Read the contents of a file from the filesystem |
| | `write_file` | Write content to a file, creating it if it doesn't exist or overwriting completely |
| | `edit_file` | Apply unified diff patches to files for targeted changes |
| | `delete_file` | Delete a file from the filesystem |
| **Directory Operations** | `make_dir` | Create directories and all necessary parent directories |
| | `scan_codebase` | Scan and display the structure of a codebase directory tree |
| **Search Operations** | `grep_find` | Search for text patterns in files using regular expressions with recursive directory traversal |
| **System Operations** | `exec_cmd` | Execute bash/cmd commands and return output (use with caution) |

Each tool provides detailed feedback during execution and handles common error cases like permission issues and missing files.

## Roadmap & TODO

### Upcoming Features

1. **MCP (Model Context Protocol) Support**
   - Integration with MCP servers for extended functionality
   - Support for custom MCP tools and resources
   - Enhanced context management through MCP

2. **Claude Skills Integration**
   - Implement Claude-like skills system for specialized capabilities
   - Modular skill architecture for extensibility
   - Built-in skills for common development tasks

### Future Enhancements
- Plan mode for structured task execution
- Plugin system for custom tools
- Enhanced session management
- Multi-project workspace support
- Integration with popular IDEs and editors

