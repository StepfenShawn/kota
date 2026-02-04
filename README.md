<a href="https://docs.rs/kota/latest/kota"><img src="https://img.shields.io/badge/docs-API Reference-dca282.svg" /></a> 
<a href="https://crates.io/crates/kota"><img src="https://img.shields.io/crates/v/kota.svg?color=dca282" /></a>  &nbsp;
# Kota
A lightweight, highly extensible ai code agent, built in Rust:    

![kota_screenshot](assert/screenshort.jpg)

# Why Kota?

Kota is designed with the philosophy of **vim** - lightweight, highly extensible, and simple to configure. Just as vim became the go-to editor for developers who value efficiency and customization, Kota aims to be the AI code agent that gets out of your way while giving you full control.

**Core Principles:**

- **Lightweight** - Minimal dependencies, fast startup, low resource usage. Built in Rust for performance and reliability.
- **Highly Extensible** - Skills system, custom tools, hooks, and plugin architecture. Extend Kota to fit your workflow, not the other way around.
- **Simple Configuration** - Plain text config files, straightforward API. No complex setup, no bloated frameworks.
- **Practical** - Focus on real development tasks: code review, refactoring, debugging, documentation. Tools that actually help you code.

Whether you use it as a CLI tool or integrate it as a library into your own projects, Kota gives you the power to build AI-assisted workflows that match your needs - without the overhead.

## Comparison with Other Code Agents

| Feature | Kota | Cursor | GitHub Copilot | Claude Code | Codex |
|---------|------|--------|----------------|-------------|-------|
| **Language** | Rust | Electron/TS | TypeScript | Proprietary | Rust |
| **Extensibility** | ✅ Lua config + Custom tools | Limited | Plugin API | ❌ No | API only |
| **Library Usage** | ✅ Rust crate | ❌ No | ❌ No | ❌ No | ❌ No |
| **Skills System** | ✅ Built-in | ❌ No | ❌ No | ✅ Built-in | ❌ No |
| **Custom Commands** | ✅ Lua functions | ❌ No | ❌ No | ❌ No | ❌ No |
| **Multi-Model** | ✅ Multiple | ✅ Multiple | ❌ OpenAI only | ❌ Claude only | ❌ OpenAI only |
| **Configuration** | Simple Lua files | GUI settings | GUI settings | Web UI | API config |
| **Learning Curve** | Low (vim-like) | Medium | Low | Low | High |

## Setup

### Configuration

Kota uses Lua-based configuration (inspired by Neovim). Create a `.kota/config.lua` file in your project root:

```lua
kota.setup({
  model = "deepseek-chat",
  api_key = os.getenv("API_KEY"),
  api_base = "https://api.deepseek.com/v1",
  temperature = 0.7,
  
  -- Custom commands with parameter support
  commands = {
    -- Simple string commands
    ["fix"] = "analyze and fix the current file",
    
    -- Function commands with parameters
    ["test"] = function(args)
      local file = args.file or "current file"
      return "run tests for " .. file
    end,
    
    ["review"] = function(args)
      local file = args.file or "current file"
      local aspect = args.aspect or "general quality"
      return string.format("review %s focusing on %s", file, aspect)
    end,
  },
})
```

Set your API key as an environment variable:

```bash
# Linux/Mac
export API_KEY="your-api-key-here"

# Windows PowerShell
$env:API_KEY="your-api-key-here"
```

See [Lua Configuration Guide](guides/lua_configuration.md) for more details.

### .kota Directory Structure

Kota uses a `.kota` directory in your project root for all configuration and extensions:

```
.kota/
├── config.lua         # Main configuration file
├── mcps/              # Model Context Protocol servers (future)
├── prompts/           # Custom prompt templates
├── skills/            # Custom skills definitions
│   ├── code-review/
│   │   └── SKILL.md
│   ├── debug/
│   │   └── SKILL.md
│   ├── refactor/
│   │   └── SKILL.md
│   └── rust-expert/
│       └── SKILL.md
├── tools/             # Custom tool implementations
├── ui/                # Custom UI components (future)
└── workflows/         # Workflow definitions (future)
```

**Directory Purposes:**

- `config.lua` - Main Lua configuration file for model settings, commands, tools, and hooks
- `skills/` - Custom skill definitions with SKILL.md files describing specialized behaviors
- `tools/` - Custom tool implementations to extend Kota's capabilities
- `prompts/` - Reusable prompt templates for common tasks
- `mcps/` - MCP server configurations (planned feature)
- `workflows/` - Automated workflow definitions (planned feature)
- `ui/` - Custom UI components and themes (planned feature)

## Installation

### As a CLI Tool
```bash
cargo install kota
```
Then start it:
```bash
kota
```

### As a Library
Add Kota to your `Cargo.toml`:
```toml
[dependencies]
kota = "0.1.3"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
```

## Usage as a Library

Kota can be used as a library to build your own AI code agents. Here's a quick example:

```rust
use kota::kota_code::{AgentBuilder, ContextManager};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create context manager for conversation history
    let context = ContextManager::new(".chat_sessions", "my-session".to_string())?;

    // Create a code agent instance with context
    let mut agent = AgentBuilder::new(
        "your-api-key".to_string(),
        "gpt-4".to_string()
    )?
    .with_context(context)
    .build()?;

    // Simple chat with automatic context management
    let response = agent.chat("Hello! Can you help me with Rust?").await?;
    println!("Tokens used: {}", response.usage().total_tokens);

    // Continue conversation (context is automatically maintained)
    let response = agent.chat("Can you give me an example?").await?;
    println!("Tokens used: {}", response.usage().total_tokens);

    Ok(())
}
```

### Library Features

- **Agent Builder**: Create customized AI code agents with different LLM providers
- **Agent Instance**: Unified structure containing agent, context manager, and skill manager
- **Context Management**: Persistent conversation history with session support
- **Plan Management**: Structured task execution with dependencies
- **Skills System**: Specialized agent behaviors for different tasks
- **Tool Registry**: Extensible tool system for custom functionality
- **Custom Commands**: Define parameterized commands in Lua configuration
- **Built-in Tools**: File operations, code scanning, grep search, and more

## Supported Models

Kota supports various LLM providers:

- **OpenAI Compatible** - (e.g., gpt-4o, gpt-4o-mini, gpt-4-turbo, gpt-3.5-turbo...)
- **DeepSeek** - (e.g., deepseek-chat, deepseek-coder...)
- **Anthropic Claude** (e.g., claude-3-5-sonnet, claude-4-opus...)

### Other Providers
- Any OpenAI-compatible API endpoint can be configured
- Local models via Ollama or similar services

## CLI Features

Kota provides a powerful command-line interface with built-in commands, custom command support, and intelligent tab completion.

### Interactive Commands

Kota provides an interactive CLI with the following commands:

- `/quit` or `/exit` - Exit the application
- `/config` - Show current model configuration
- `/help` - Show available commands
- `/history` - Show conversation history
- `/skills` - List all available skills
- `/skill <name>` - Activate a specific skill
- `/skill-off` - Deactivate current skill
- `/load <session_id>` - Load specific session
- `/sessions` - List all sessions
- `/delete <session_id>` - Delete a specific session

### Custom Commands

Kota supports custom commands defined in your Lua configuration with parameter support:

```bash
# Simple commands
/fix

# Named parameters
/test file=main.rs
/review file=src/lib.rs aspect=security

# Positional parameters
/refactor main.rs "use builder pattern"

# Mixed parameters
/review main.rs aspect=performance
```

Custom commands are defined in your `.kota/config.lua`:

```lua
commands = {
  -- Simple string commands
  ["fix"] = "analyze and fix the current file",
  
  -- Function commands with parameters
  ["test"] = function(args)
    local file = args.file or "current file"
    return "run tests for " .. file
  end,
  
  ["review"] = function(args)
    local file = args.file or "current file"
    local aspect = args.aspect or "general quality"
    return string.format("review %s focusing on %s", file, aspect)
  end,
}
```

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
| **Plan Mode** | `update_plan` | Manage structured execution plans with tasks, dependencies, and status tracking (similar to Claude Code) |

Each tool provides detailed feedback during execution and handles common error cases like permission issues and missing files.

## Skills System

Kota now includes a powerful Skills system similar to Claude's skills, allowing you to specialize the AI assistant for specific tasks.

### Built-in Skills

| Skill | Description | Available Tools |
|-------|-------------|-----------------|
| **code_review** | Code quality analysis and review | read_file, scan_codebase, grep_search |
| **refactor** | Code refactoring and optimization | read_file, edit_file, write_file |
| **debug** | Problem diagnosis and debugging | read_file, execute_bash, grep_search |
| **documentation** | Documentation writing and improvement | read_file, write_file, scan_codebase |

### Using Skills

```bash
# List all available skills
❯ /skills

# Activate a skill
❯ /skill code_review

# Use the skill
❯ Please review src/main.rs for code quality issues

# Deactivate skill
❯ /skill-off
```

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
- Plugin system for custom tools
- Enhanced session management
- Multi-project workspace support
- Integration with popular IDEs and editors

## API Documentation

Full API documentation is available at [docs.rs/kota](https://docs.rs/kota).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

