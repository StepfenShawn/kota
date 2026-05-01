use kota::KotaConfig;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_lua_config_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".kota");
    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.lua");
    let config_content = r#"
kota.setup({
  model = "gpt-4o",
  api_key = "test-api-key-123",
  temperature = 0.8,
  
  tools = {
    enabled = { "read_file", "write_file" },
    disabled = { "delete_file" },
  },
})
"#;

    fs::write(&config_path, config_content).unwrap();

    let config = KotaConfig::from_lua_file(&config_path).unwrap();

    assert_eq!(config.model, "gpt-4o");
    assert_eq!(config.api_key, "test-api-key-123");
    assert_eq!(config.temperature, Some(0.8));
    assert_eq!(config.enabled_tools, vec!["read_file", "write_file"]);
    assert_eq!(config.disabled_tools, vec!["delete_file"]);
}

#[test]
fn test_lua_config_with_env_var() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".kota");
    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.lua");
    let config_content = r#"
kota.setup({
  model = "deepseek-chat",
  api_key = os.getenv("TEST_API_KEY"),
  temperature = 0.7,
})
"#;

    fs::write(&config_path, config_content).unwrap();

    // Set environment variable
    unsafe {
        std::env::set_var("TEST_API_KEY", "env-api-key-456");
    }

    let config = KotaConfig::from_lua_file(&config_path).unwrap();

    assert_eq!(config.model, "deepseek-chat");
    assert_eq!(config.api_key, "env-api-key-456");
    assert_eq!(config.temperature, Some(0.7));

    // Clean up env var
    unsafe {
        std::env::remove_var("TEST_API_KEY");
    }
}

#[test]
fn test_missing_config_file() {
    let temp_dir = TempDir::new().unwrap();

    // Change to temp directory (no config file)
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = KotaConfig::load();

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Configuration file not found")
    );

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}
