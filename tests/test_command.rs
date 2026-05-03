use mlua::prelude::*;
use std::collections::HashMap;

// Import the types we need from the kota crate
use kota::{CommandDef, CommandRegistry, KotaConfig, parse_command_input};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_input() {
        // No args
        let (name, args) = parse_command_input("test").unwrap();
        assert_eq!(name, "test");
        assert!(args.is_empty());

        // Positional args
        let (name, args) = parse_command_input("test arg1 arg2").unwrap();
        assert_eq!(name, "test");
        assert_eq!(args.get("1"), Some(&"arg1".to_string()));
        assert_eq!(args.get("2"), Some(&"arg2".to_string()));

        // Named args
        let (name, args) = parse_command_input("test file=main.rs aspect=security").unwrap();
        assert_eq!(name, "test");
        assert_eq!(args.get("file"), Some(&"main.rs".to_string()));
        assert_eq!(args.get("aspect"), Some(&"security".to_string()));

        // Mixed args
        let (name, args) = parse_command_input("test arg1 key=value arg2").unwrap();
        assert_eq!(name, "test");
        assert_eq!(args.get("1"), Some(&"arg1".to_string()));
        assert_eq!(args.get("key"), Some(&"value".to_string()));
        assert_eq!(args.get("2"), Some(&"arg2".to_string()));
    }

    #[test]
    fn test_string_command() {
        let mut config = KotaConfig::default();
        config.commands.insert(
            "fix".to_string(),
            CommandDef::String("analyze and fix the current file".to_string()),
        );

        let registry = CommandRegistry::new(&config).unwrap();
        let result = registry.execute("fix", HashMap::new()).unwrap();
        assert_eq!(result, "analyze and fix the current file");
    }

    #[test]
    fn test_function_command() {
        let lua = Lua::new();
        let func: LuaFunction = lua
            .load(
                r#"
            function(args)
                local file = args.file or "current file"
                return "run tests for " .. file
            end
        "#,
            )
            .eval()
            .unwrap();

        let bytecode = func.dump(false);

        let mut config = KotaConfig::default();
        config
            .commands
            .insert("test".to_string(), CommandDef::Function(bytecode));

        let registry = CommandRegistry::new(&config).unwrap();

        // Without args
        let result = registry.execute("test", HashMap::new()).unwrap();
        assert_eq!(result, "run tests for current file");

        // With args
        let mut args = HashMap::new();
        args.insert("file".to_string(), "main.rs".to_string());
        let result = registry.execute("test", args).unwrap();
        assert_eq!(result, "run tests for main.rs");
    }
}
