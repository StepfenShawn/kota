use colored::*;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Result, anyhow};

use crate::kota_code::llm::{ChatResponse, Message, OpenAiClient, Usage};

use super::context::ContextManager;
use super::plan::PlanManager;
use super::runtime::{ToolRegistry, dyn_tools_loader::LuaToolLoader};
use super::skills::SkillManager;
use super::tools::{
    WrappedCreateDirectoryTool, WrappedDeleteFileTool, WrappedEditFileTool,
    WrappedExecuteBashCommandTool, WrappedGrepSearchTool, WrappedReadFileTool,
    WrappedScanCodebaseTool, WrappedUpdatePlanTool, WrappedWriteFileTool,
};

const MAX_TOOL_TURNS: usize = 20;
const DEFAULT_API_BASE: &str = "https://api.openai.com/v1";

/// Complete agent instance with context and skill management.
pub struct AgentInstance {
    pub client: OpenAiClient,
    pub context: Option<ContextManager>,
    pub skill_manager: Option<SkillManager>,
    pub tool_registry: ToolRegistry,
    preamble: String,
}

impl AgentInstance {
    /// Get the context manager
    pub fn context(&self) -> Option<&ContextManager> {
        self.context.as_ref()
    }

    /// Get the skill manager
    pub fn skill_manager(&self) -> Option<&SkillManager> {
        self.skill_manager.as_ref()
    }

    /// Get mutable context manager
    pub fn context_mut(&mut self) -> Option<&mut ContextManager> {
        self.context.as_mut()
    }

    /// Get mutable skill manager
    pub fn skill_manager_mut(&mut self) -> Option<&mut SkillManager> {
        self.skill_manager.as_mut()
    }

    /// Get the tool registry
    pub fn tool_registry(&self) -> &ToolRegistry {
        &self.tool_registry
    }

    /// Get mutable tool registry
    pub fn tool_registry_mut(&mut self) -> &mut ToolRegistry {
        &mut self.tool_registry
    }

    /// Send a message to the model, handling tool calls until a final answer is produced.
    pub async fn chat(&mut self, input: &str) -> Result<ChatResponse> {
        if let Some(context) = self.context_mut() {
            context.add_message(Message::user(input));
        }

        let mut working: Vec<Message> = Vec::new();
        working.push(Message::system(self.preamble.clone()));

        if let Some(skill_manager) = &self.skill_manager
            && let Some(skill) = skill_manager.get_active_skill()
        {
            working.push(Message::system(format!(
                "[ACTIVE SKILL: {}]\n{}\n\n{}",
                skill.name, skill.description, skill.instructions
            )));
        }

        if let Some(context) = &self.context {
            working.extend(context.get_messages().iter().cloned());
        }

        let mut tool_definitions = Vec::with_capacity(self.tool_registry.len());
        for tool in self.tool_registry.get_all() {
            tool_definitions.push(tool.definition(String::new()).await);
        }

        let mut usage = Usage::default();
        let mut final_content = None;

        for _ in 0..MAX_TOOL_TURNS {
            let thinking = Arc::new(AtomicBool::new(false));
            
            let response = self
                .client
                .stream_completion(
                    &working,
                    &tool_definitions,
                    |token| {
                        // TODO: move to kota_cli?
                        thinking.swap(false, Ordering::Relaxed);
                        print!("{}", token);
                        let _ = std::io::stdout().flush();
                    },
                    |token| {
                        // TODO: move to kota_cli?
                        if !thinking.swap(true, Ordering::Relaxed) {
                            println!("{} ", "🧠 Thinking...".yellow());
                        }
                        print!("{}", token.dimmed());
                        let _ = std::io::stdout().flush();
                    },
                )
                .await?;
            usage.add(response.usage);

            let choice = response
                .choices
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("No choices returned by the model"))?;
            let message = choice.message;
            let tool_calls = message.tool_calls.clone().unwrap_or_default();

            if tool_calls.is_empty() {
                final_content = Some(message.content.clone().unwrap_or_default());
                working.push(message);
                break;
            }

            working.push(message);
            for call in tool_calls {
                let output = self
                    .execute_tool(&call.function.name, &call.function.arguments)
                    .await;
                working.push(Message::tool(call.id, output));
            }
        }

        let content = final_content
            .ok_or_else(|| anyhow!("Exceeded maximum of {} tool turns", MAX_TOOL_TURNS))?;

        if let Some(context) = self.context_mut() {
            context.add_message(Message::assistant(content.clone()));
            context.save()?;
        }

        Ok(ChatResponse { content, usage })
    }

    async fn execute_tool(&self, name: &str, args: &str) -> String {
        match self
            .tool_registry
            .get_all()
            .iter()
            .find(|tool| tool.name() == name)
        {
            Some(tool) => match tool.call(args.to_string()).await {
                Ok(output) => output,
                Err(e) => format!("Error executing tool '{}': {}", name, e),
            },
            None => format!("Error: tool '{}' not found", name),
        }
    }
}

/// Builder for creating AI agents with custom configuration.
pub struct AgentBuilder {
    api_key: String,
    model_name: String,
    api_base: String,
    temperature: Option<f64>,
    plan_manager: PlanManager,
    context: Option<ContextManager>,
    skill_manager: Option<SkillManager>,
    tool_registry: ToolRegistry,
}

impl AgentBuilder {
    /// Create a new agent builder.
    ///
    /// * `api_key` - API key for the OpenAI-compatible endpoint
    /// * `model_name` - Model name (e.g. "gpt-4o", "deepseek-chat")
    pub fn new(api_key: String, model_name: String) -> Result<Self> {
        Ok(Self {
            api_key,
            model_name,
            api_base: DEFAULT_API_BASE.to_string(),
            temperature: Some(0.7),
            plan_manager: PlanManager::new(),
            context: None,
            skill_manager: None,
            tool_registry: ToolRegistry::new(),
        })
    }

    /// Set the base URL of the OpenAI-compatible API.
    pub fn with_api_base(mut self, api_base: impl Into<String>) -> Self {
        self.api_base = api_base.into();
        self
    }

    /// Set the sampling temperature.
    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set a custom plan manager for task management
    pub fn with_plan_manager(mut self, manager: PlanManager) -> Self {
        self.plan_manager = manager;
        self
    }

    /// Set a context manager for conversation history
    pub fn with_context(mut self, context: ContextManager) -> Self {
        self.context = Some(context);
        self
    }

    /// Set a skill manager for specialized agent behaviors
    pub fn with_skill_manager(mut self, skill_manager: SkillManager) -> Self {
        self.skill_manager = Some(skill_manager);
        self
    }

    /// Build the agent with the configured settings.
    pub fn build(mut self) -> Result<AgentInstance> {
        self.register_builtin_tools();

        let lua_tools = LuaToolLoader::load_tools().unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load Lua tools: {}", e);
            Vec::new()
        });

        for tool in lua_tools {
            self.tool_registry.add(Box::new(tool));
        }

        let preamble = Self::preamble().to_string();
        let client = OpenAiClient::new(
            self.api_key,
            self.model_name,
            self.api_base,
            self.temperature,
        )?;

        Ok(AgentInstance {
            client,
            context: self.context,
            skill_manager: self.skill_manager,
            tool_registry: self.tool_registry,
            preamble,
        })
    }

    fn register_builtin_tools(&mut self) {
        self.tool_registry.add(Box::new(WrappedReadFileTool::new()));
        self.tool_registry
            .add(Box::new(WrappedWriteFileTool::new()));
        self.tool_registry.add(Box::new(WrappedEditFileTool::new()));
        self.tool_registry
            .add(Box::new(WrappedDeleteFileTool::new()));
        self.tool_registry
            .add(Box::new(WrappedExecuteBashCommandTool::new()));
        self.tool_registry
            .add(Box::new(WrappedScanCodebaseTool::new()));
        self.tool_registry
            .add(Box::new(WrappedCreateDirectoryTool::new()));
        self.tool_registry
            .add(Box::new(WrappedGrepSearchTool::new()));
        self.tool_registry.add(Box::new(WrappedUpdatePlanTool::new(
            self.plan_manager.clone(),
        )));
    }

    fn preamble() -> &'static str {
        r#"
        Your name is Kato. You are a helpful AI code assistant with comprehensive file system and command execution access. 
        You can read, write, edit (with patches), and delete files, execute bash commands, scan codebase structures, search text in the codebase and create directories. 
        Use the edit_file tool for making small, targeted changes to existing files - it's more efficient than rewriting entire files.
        
        You also have access to Plan Mode via the update_plan tool. Use it to:
        - Create structured execution plans for complex tasks
        - Break down work into manageable tasks with dependencies
        - Track progress and update task status (pending, in_progress, completed, blocked)
        - Show current plan and identify next available tasks
        
        Please provide clear and concise responses and be careful when modifying files or executing commands."#
    }
}

/// Convenience function for creating an agent with default settings.
pub fn create_agent(api_key: String, model_name: String) -> Result<AgentInstance> {
    AgentBuilder::new(api_key, model_name)?.build()
}
