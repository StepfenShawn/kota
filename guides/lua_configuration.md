# Lua Configuration

Kota uses Lua-based configuration, inspired by Neovim's configuration system. This provides a flexible and programmable way to configure your AI code agent.

## Configuration File

Create a `.kota/config.lua` file in your project root:

```lua
kota.setup({
  model = "deepseek-flash",
  api_key = os.getenv("API_KEY"),
  api_base = "https://api.deepseek.com/v1",
  temperature = 1.0,
  
  tools = {
    enabled = { "grep_find", "write_file", "read_file" },
    disabled = { "delete_file" },
  },
  
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
  
  hooks = {
    before_execute = function(tool, args)
      -- Custom logic before tool execution
    end,
    after_execute = function(result)
      -- Custom logic after tool execution
    end,
  },
})
```

## Configuration Options

### Basic Settings

- **model** (string): The LLM model to use
  - Examples: `"gpt-4o"`, `"deepseek-flash"`
  
- **api_key** (string): Your API key for the LLM provider
  - Can use `os.getenv("API_KEY")` to read from environment variables
  
- **temperature** (number, optional): Temperature for LLM responses
  - Default: `1.0`

### Tools Configuration

Control which tools are available to the agent:

```lua
tools = {
  enabled = { "grep_find", "write_file", "read_file" },
  disabled = { "delete_file" },
}
```

Available tools:
- `read_file` - Read file contents
- `write_file` - Write to files
- `edit_file` - Apply patches to files
- `delete_file` - Delete files
- `make_dir` - Create directories
- `scan_codebase` - Scan directory structure
- `grep_find` - Search for patterns
- `exec_cmd` - Execute shell commands
- `update_plan` - Manage task plans

### Commands

Define custom command shortcuts with parameter support:

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
  
  ["refactor"] = function(args)
    local file = args.file or args["1"] or "current file"
    local pattern = args.pattern or args["2"] or "improve code structure"
    return string.format("refactor %s to %s", file, pattern)
  end,
}
```

#### Using Custom Commands

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

#### Parameter Types

- **Named parameters**: `key=value` format, accessed as `args.key`
- **Positional parameters**: Space-separated values, accessed as `args["1"]`, `args["2"]`, etc.
- **Default values**: Use `or` operator for fallback values

### Hooks (Coming Soon)

Add custom logic before and after tool execution:

```lua
hooks = {
  before_execute = function(tool, args)
    print("Executing tool: " .. tool)
  end,
  after_execute = function(result)
    print("Tool execution completed")
  end,
}
```

## Environment Variables

The Lua configuration can read environment variables using `os.getenv()`:

```lua
kota.setup({
  api_key = os.getenv("API_KEY"),
})
```

Set your environment variables:

```bash
# Linux/Mac
export API_KEY="your-api-key-here"

# Windows PowerShell
$env:API_KEY="your-api-key-here"
```

## Benefits of Lua Configuration

1. **Programmable**: Use Lua logic for dynamic configuration
2. **Type-safe**: Lua provides better structure than plain text
3. **Extensible**: Easy to add custom tools and hooks
4. **Familiar**: Similar to Neovim's configuration system
5. **Flexible**: Supports both static values and environment variables

## Future Enhancements

- Custom tool definitions in Lua
- Plugin system for extending functionality
- Configuration hot-reloading
- Project-specific vs global configuration
- Configuration validation and error reporting
