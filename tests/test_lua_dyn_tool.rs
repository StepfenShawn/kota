use kota::kota_code::llm::ToolDyn;
use kota::kota_code::runtime::dyn_tools_loader::{LuaToolLoader, dyn_tool::LuaDynTool};
use serde_json::json;
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_lua_dyn_tool_basic_call() {
    // Create a simple Lua function that adds two numbers
    let lua = mlua::Lua::new();

    // Create the function and dump it to bytecode
    let func: mlua::Function = lua
        .load(
            r#"
            return function(args)
                return { result = args.a + args.b }
            end
        "#,
        )
        .eval()
        .unwrap();

    let bytecode = func.dump(false);

    let tool = LuaDynTool::new(
        "test_add".to_string(),
        "Add two numbers".to_string(),
        json!({
            "type": "object",
            "properties": {
                "a": { "type": "number" },
                "b": { "type": "number" }
            }
        }),
        bytecode,
    );

    // Test tool definition
    let definition = tool.definition("".to_string()).await;
    assert_eq!(definition.name, "test_add");
    assert_eq!(definition.description, "Add two numbers");

    // Test tool call via ToolDyn
    let args_json = json!({ "a": 5, "b": 3 });
    let args_str = serde_json::to_string(&args_json).unwrap();

    let result_str = tool.call(args_str).await.unwrap();
    let result: serde_json::Value = serde_json::from_str(&result_str).unwrap();
    assert_eq!(result["result"], 8);
}

#[tokio::test]
async fn test_lua_dyn_tool_string_manipulation() {
    let lua = mlua::Lua::new();
    let func: mlua::Function = lua
        .load(
            r#"
            return function(args)
                local text = args.text
                local operation = args.operation
                
                if operation == "uppercase" then
                    return { result = string.upper(text) }
                elseif operation == "lowercase" then
                    return { result = string.lower(text) }
                elseif operation == "reverse" then
                    return { result = string.reverse(text) }
                else
                    return { error = "Unknown operation" }
                end
            end
        "#,
        )
        .eval()
        .unwrap();

    let bytecode = func.dump(false);

    let tool = LuaDynTool::new(
        "string_transform".to_string(),
        "Transform string".to_string(),
        json!({
            "type": "object",
            "properties": {
                "text": { "type": "string" },
                "operation": { "type": "string" }
            }
        }),
        bytecode,
    );

    // Test uppercase
    let args_json = json!({ "text": "hello", "operation": "uppercase" });
    let result_str = tool
        .call(serde_json::to_string(&args_json).unwrap())
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result_str).unwrap();
    assert_eq!(result["result"], "HELLO");

    // Test lowercase
    let args_json = json!({ "text": "WORLD", "operation": "lowercase" });
    let result_str = tool
        .call(serde_json::to_string(&args_json).unwrap())
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result_str).unwrap();
    assert_eq!(result["result"], "world");
}

#[tokio::test]
async fn test_lua_dyn_tool_array_handling() {
    let lua = mlua::Lua::new();
    let func: mlua::Function = lua
        .load(
            r#"
            return function(args)
                local numbers = args.numbers
                local sum = 0
                for i = 1, #numbers do
                    sum = sum + numbers[i]
                end
                return { sum = sum, count = #numbers }
            end
        "#,
        )
        .eval()
        .unwrap();

    let bytecode = func.dump(false);

    let tool = LuaDynTool::new(
        "array_sum".to_string(),
        "Sum array of numbers".to_string(),
        json!({
            "type": "object",
            "properties": {
                "numbers": {
                    "type": "array",
                    "items": { "type": "number" }
                }
            }
        }),
        bytecode,
    );

    let args_json = json!({ "numbers": [1, 2, 3, 4, 5] });
    let result_str = tool
        .call(serde_json::to_string(&args_json).unwrap())
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result_str).unwrap();

    assert_eq!(result["sum"], 15);
    assert_eq!(result["count"], 5);
}

#[tokio::test]
async fn test_lua_dyn_tool_error_handling() {
    let lua = mlua::Lua::new();
    let func: mlua::Function = lua
        .load(
            r#"
            return function(args)
                if args.value == 0 then
                    error("Value cannot be zero")
                end
                return { result = 100 / args.value }
            end
        "#,
        )
        .eval()
        .unwrap();

    let bytecode = func.dump(false);

    let tool = LuaDynTool::new(
        "divide".to_string(),
        "Divide 100 by value".to_string(),
        json!({
            "type": "object",
            "properties": {
                "value": { "type": "number" }
            }
        }),
        bytecode,
    );

    // Test error case
    let args_json = json!({ "value": 0 });
    let result = tool.call(serde_json::to_string(&args_json).unwrap()).await;
    assert!(result.is_err());
}

#[test]
fn test_lua_tool_loader_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // No .kota/tools directory
    let tools = LuaToolLoader::load_tools().unwrap();
    assert_eq!(tools.len(), 0);

    std::env::set_current_dir(original_dir).unwrap();
}

#[tokio::test]
async fn test_loaded_tool_execution() {
    let temp_dir = TempDir::new().unwrap();
    let tools_dir = temp_dir.path().join(".kota").join("tools");
    fs::create_dir_all(&tools_dir).unwrap();

    let tools_file = tools_dir.join("init.lua");
    let tools_content = r#"
kota.register_tool({
    name = "multiply",
    description = "Multiply two numbers",
    parameters = {
        type = "object",
        properties = {
            x = { type = "number" },
            y = { type = "number" }
        },
        required = { "x", "y" }
    },
    entry = function(args)
        return { product = args.x * args.y }
    end
})
"#;

    fs::write(&tools_file, tools_content).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = LuaToolLoader::load_tools();
    if let Err(e) = &result {
        eprintln!("Error loading tools: {}", e);
    }
    let tools = result.unwrap();
    assert_eq!(tools.len(), 1);

    let tool = &tools[0];
    let args_json = json!({ "x": 7, "y": 6 });
    let result_str = tool
        .call(serde_json::to_string(&args_json).unwrap())
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result_str).unwrap();

    assert_eq!(result["product"], 42);

    std::env::set_current_dir(original_dir).unwrap();
}

#[tokio::test]
async fn test_lua_dyn_tool_complex_object() {
    let lua = mlua::Lua::new();
    let func: mlua::Function = lua
        .load(
            r#"
            return function(args)
                local user = args.user
                return {
                    greeting = "Hello, " .. user.name,
                    age_next_year = user.age + 1,
                    is_adult = user.age >= 18
                }
            end
        "#,
        )
        .eval()
        .unwrap();

    let bytecode = func.dump(false);

    let tool = LuaDynTool::new(
        "user_info".to_string(),
        "Process user information".to_string(),
        json!({
            "type": "object",
            "properties": {
                "user": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "age": { "type": "number" }
                    }
                }
            }
        }),
        bytecode,
    );

    let args_json = json!({"user": {
                "name": "Alice",
                "age": 25
            } });
    let result_str = tool
        .call(serde_json::to_string(&args_json).unwrap())
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result_str).unwrap();

    assert_eq!(result["greeting"], "Hello, Alice");
    assert_eq!(result["age_next_year"], 26);
    assert_eq!(result["is_adult"], true);
}
