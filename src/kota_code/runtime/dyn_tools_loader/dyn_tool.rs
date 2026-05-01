use crate::kota_code::llm::{BoxFuture, ToolDefinition, ToolDyn, ToolError};
use mlua::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DynToolError {
    #[error("Input is invalid: {0}")]
    InvalidInput(String),
}

/// Lua tool that wraps a Lua function and makes it callable as an LLM tool
#[derive(Clone)]
pub struct LuaDynTool {
    name: String,
    description: String,
    parameters: JsonValue,
    lua_code: Arc<Vec<u8>>, // Lua function bytecode
}

impl LuaDynTool {
    pub fn new(
        name: String,
        description: String,
        parameters: JsonValue,
        lua_code: Vec<u8>,
    ) -> Self {
        Self {
            name,
            description,
            parameters,
            lua_code: Arc::new(lua_code),
        }
    }

    pub fn tool_name(&self) -> &str {
        &self.name
    }
}

#[derive(Deserialize)]
pub struct LuaToolArgs {
    #[serde(flatten)]
    pub args: JsonValue,
}

#[derive(Serialize)]
pub struct LuaToolOutput {
    pub result: JsonValue,
}

impl ToolDyn for LuaDynTool {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn definition<'a>(&'a self, _prompt: String) -> BoxFuture<'a, ToolDefinition> {
        Box::pin(async move {
            ToolDefinition {
                name: self.name.clone(),
                description: self.description.clone(),
                parameters: self.parameters.clone(),
            }
        })
    }

    fn call<'a>(&'a self, args: String) -> BoxFuture<'a, Result<String, ToolError>> {
        Box::pin(async move {
            let lua = Lua::new();

            // Load the Lua function from bytecode
            let func: LuaFunction = lua.load(&*self.lua_code).into_function().map_err(|e| {
                ToolError::ToolCallError(format!("Failed to load Lua bytecode: {}", e).into())
            })?;

            // Parse JSON args
            let json_args: JsonValue = serde_json::from_str(&args).map_err(|e| {
                ToolError::ToolCallError(format!("Failed to parse args: {}", e).into())
            })?;

            // Convert JSON args to Lua value
            let lua_args = json_to_lua(&lua, &json_args).map_err(|e| {
                ToolError::ToolCallError(format!("Failed to convert args to Lua: {}", e).into())
            })?;

            // Call the Lua function
            let result: LuaValue = func.call(lua_args).map_err(|e| {
                ToolError::ToolCallError(format!("Lua function call failed: {}", e).into())
            })?;

            // Convert Lua result back to JSON
            let json_result = lua_to_json(&result).map_err(|e| {
                ToolError::ToolCallError(
                    format!("Failed to convert Lua result to JSON: {}", e).into(),
                )
            })?;

            // Return as JSON string
            serde_json::to_string(&json_result).map_err(|e| {
                ToolError::ToolCallError(format!("Failed to serialize result: {}", e).into())
            })
        })
    }
}

// Helper: Convert JSON to Lua value
fn json_to_lua<'lua>(lua: &'lua Lua, json: &JsonValue) -> LuaResult<LuaValue<'lua>> {
    match json {
        JsonValue::Null => Ok(LuaValue::Nil),
        JsonValue::Bool(b) => Ok(LuaValue::Boolean(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(LuaValue::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(LuaValue::Number(f))
            } else {
                Ok(LuaValue::Nil)
            }
        }
        JsonValue::String(s) => Ok(LuaValue::String(lua.create_string(s)?)),
        JsonValue::Array(arr) => {
            let table = lua.create_table()?;
            for (i, val) in arr.iter().enumerate() {
                table.set(i + 1, json_to_lua(lua, val)?)?;
            }
            Ok(LuaValue::Table(table))
        }
        JsonValue::Object(obj) => {
            let table = lua.create_table()?;
            for (key, val) in obj.iter() {
                table.set(key.as_str(), json_to_lua(lua, val)?)?;
            }
            Ok(LuaValue::Table(table))
        }
    }
}

// Helper: Convert Lua value to JSON
fn lua_to_json(value: &LuaValue) -> LuaResult<JsonValue> {
    match value {
        LuaValue::Nil => Ok(JsonValue::Null),
        LuaValue::Boolean(b) => Ok(JsonValue::Bool(*b)),
        LuaValue::Integer(i) => Ok(JsonValue::Number((*i).into())),
        LuaValue::Number(n) => {
            if let Some(num) = serde_json::Number::from_f64(*n) {
                Ok(JsonValue::Number(num))
            } else {
                Ok(JsonValue::Null)
            }
        }
        LuaValue::String(s) => Ok(JsonValue::String(s.to_str()?.to_string())),
        LuaValue::Table(table) => {
            // Check if it's an array or object
            let mut is_array = true;
            let mut max_index = 0;

            for pair in table.clone().pairs::<LuaValue, LuaValue>() {
                let (key, _) = pair?;
                if let LuaValue::Integer(i) = key {
                    if i > 0 {
                        max_index = max_index.max(i);
                    } else {
                        is_array = false;
                        break;
                    }
                } else {
                    is_array = false;
                    break;
                }
            }

            if is_array && max_index > 0 {
                let mut arr = Vec::new();
                for i in 1..=max_index {
                    let val: LuaValue = table.get(i)?;
                    arr.push(lua_to_json(&val)?);
                }
                Ok(JsonValue::Array(arr))
            } else {
                let mut obj = serde_json::Map::new();
                for pair in table.clone().pairs::<LuaValue, LuaValue>() {
                    let (key, val) = pair?;
                    let key_str = match key {
                        LuaValue::String(s) => s.to_str()?.to_string(),
                        LuaValue::Integer(i) => i.to_string(),
                        _ => continue,
                    };
                    obj.insert(key_str, lua_to_json(&val)?);
                }
                Ok(JsonValue::Object(obj))
            }
        }
        _ => Ok(JsonValue::Null),
    }
}
