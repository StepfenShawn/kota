use anyhow::{Ok, Result};
use rig::{
    agent::Agent,
    client::{CompletionClient, ProviderClient},
    providers::{
        anthropic, cohere,
        deepseek::{self, DEEPSEEK_CHAT},
        ollama, openai,
    },
    streaming::StreamingPrompt,
};

use super::context::ContextManager;
use super::plan::PlanManager;
use super::runtime::{dyn_tools_loader::LuaToolLoader, ToolRegistry};
use super::skills::SkillManager;
use super::tools::{
    WrappedCreateDirectoryTool, WrappedDeleteFileTool, WrappedEditFileTool,
    WrappedExecuteBashCommandTool, WrappedGrepSearchTool, WrappedReadFileTool,
    WrappedScanCodebaseTool, WrappedUpdatePlanTool, WrappedWriteFileTool,
};

macro_rules! build_agent {
    ($client_expr:expr, $model_name:expr, $preamble:expr, $dyn_tools:expr, $variant:ident) => {{
        let client = $client_expr?;
        let agent = client
            .agent($model_name)
            .preamble(&$preamble)
            .max_tokens(4096)
            .tools($dyn_tools)
            .build();
        AgentType::$variant(agent)
    }};
}

macro_rules! impl_stream_chat {
    ($agent:expr, $input:expr, $hook:expr, $history:expr) => {
        $agent
            .stream_prompt($input)
            .with_hook($hook)
            .multi_turn(20)
            .with_history($history)
            .await
    };
}

/// Supported LLM providers
#[derive(Debug, Clone)]
pub enum Provider {
    /// OpenAI (GPT-4, GPT-3.5, etc.)
    OpenAI,
    /// Anthropic Claude models
    Anthropic,
    /// Cohere models
    Cohere,
    /// DeepSeek models
    DeepSeek,
    /// Local Ollama models
    Ollama,
}

/// Agent enum to handle different provider types
///
/// This enum wraps agents from different LLM providers, allowing you to work
/// with them through a unified interface.
pub enum AgentType {
    /// OpenAI agent
    OpenAI(Agent<openai::responses_api::ResponsesCompletionModel>),
    /// Anthropic Claude agent
    Anthropic(Agent<anthropic::completion::CompletionModel>),
    /// Cohere agent
    Cohere(Agent<cohere::CompletionModel>),
    /// DeepSeek agent
    DeepSeek(Agent<deepseek::CompletionModel>),
    /// Ollama local agent
    Ollama(Agent<ollama::CompletionModel>),
}

/// Complete agent instance with context and skill management
///
/// This struct combines the agent with its context manager and skill manager,
/// providing a complete solution for building AI assistants.
pub struct AgentInstance {
    pub agent: AgentType,
    pub context: Option<ContextManager>,
    pub skill_manager: Option<SkillManager>,
    pub tool_registry: ToolRegistry,
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
}

impl AgentInstance {
    /// Stream chat with the agent
    ///
    /// # Arguments
    ///
    /// * `input` - The user input message
    /// * `hook` - Session hook for tracking
    /// * `history` - Conversation history
    ///
    /// # Returns
    ///
    /// Returns a completion response after streaming to stdout
    pub async fn stream_chat<H>(
        &self,
        input: &str,
        hook: H,
        history: Vec<rig::completion::Message>,
    ) -> Result<rig::agent::FinalResponse>
    where
        H: rig::agent::StreamingPromptHook<openai::responses_api::ResponsesCompletionModel>
            + Clone
            + 'static,
        H: rig::agent::StreamingPromptHook<anthropic::completion::CompletionModel>
            + Clone
            + 'static,
        H: rig::agent::StreamingPromptHook<cohere::CompletionModel> + Clone + 'static,
        H: rig::agent::StreamingPromptHook<deepseek::CompletionModel> + Clone + 'static,
        H: rig::agent::StreamingPromptHook<ollama::CompletionModel> + Clone + 'static,
    {
        match &self.agent {
            AgentType::OpenAI(agent) => {
                let mut stream = impl_stream_chat!(agent, input, hook.clone(), history.clone());
                rig::agent::stream_to_stdout(&mut stream)
                    .await
                    .map_err(|e| anyhow::anyhow!(e))
            }
            AgentType::Anthropic(agent) => {
                let mut stream = impl_stream_chat!(agent, input, hook.clone(), history.clone());
                rig::agent::stream_to_stdout(&mut stream)
                    .await
                    .map_err(|e| anyhow::anyhow!(e))
            }
            AgentType::Cohere(agent) => {
                let mut stream = impl_stream_chat!(agent, input, hook.clone(), history.clone());
                rig::agent::stream_to_stdout(&mut stream)
                    .await
                    .map_err(|e| anyhow::anyhow!(e))
            }
            AgentType::DeepSeek(agent) => {
                let mut stream = impl_stream_chat!(agent, input, hook.clone(), history.clone());
                rig::agent::stream_to_stdout(&mut stream)
                    .await
                    .map_err(|e| anyhow::anyhow!(e))
            }
            AgentType::Ollama(agent) => {
                let mut stream = impl_stream_chat!(agent, input, hook.clone(), history.clone());
                rig::agent::stream_to_stdout(&mut stream)
                    .await
                    .map_err(|e| anyhow::anyhow!(e))
            }
        }
    }

    /// Convenient method to chat with automatic context management
    ///
    /// This method handles the complete chat flow:
    /// - Creates session hook automatically
    /// - Retrieves conversation history from context
    /// - Adds user message to context
    /// - Streams the chat response
    /// - Saves assistant response to context
    /// - Auto-saves context to disk
    ///
    /// # Arguments
    ///
    /// * `input` - The user input message
    ///
    /// # Returns
    ///
    /// Returns a completion response with usage information
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use kota::kota_code::{AgentBuilder, ContextManager};
    /// use anyhow::Result;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let context = ContextManager::new(".chat_sessions", "my-session".to_string())?;
    ///     let mut agent = AgentBuilder::new("api-key".to_string(), "gpt-4".to_string())?
    ///         .with_context(context)
    ///         .build()?;
    ///
    ///     let response = agent.chat("Hello, how are you?").await?;
    ///     println!("Tokens used: {}", response.usage().total_tokens);
    ///     Ok(())
    /// }
    /// ```
    pub async fn chat(&mut self, input: &str) -> Result<rig::agent::FinalResponse> {
        use super::runtime::SessionIdHook;
        use rig::completion::Message;

        // 添加用户消息到上下文
        if let Some(context) = self.context_mut() {
            context.add_message(Message::user(input));
        }

        // 创建会话钩子
        let session_id = self
            .context()
            .map(|c| c.session_id().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let hook = SessionIdHook::new(session_id);

        // 获取历史消息
        let mut history = self
            .context()
            .map(|c| c.get_messages().to_vec())
            .unwrap_or_default();

        // 如果有激活的 skill，在历史消息前添加 skill instructions
        if let Some(skill_manager) = &self.skill_manager {
            if let Some(skill) = skill_manager.get_active_skill() {
                let skill_prompt = format!(
                    "[ACTIVE SKILL: {}]\n{}\n\n{}",
                    skill.name, skill.description, skill.instructions
                );
                history.insert(0, Message::user(&skill_prompt));
            }
        }

        // 执行流式聊天
        let response = self.stream_chat(input, hook, history).await?;

        // 保存助手响应到上下文
        if let Some(context) = self.context_mut() {
            let response_content = response.response();
            context.add_message(Message::assistant(response_content));

            // 自动保存上下文
            context.save()?;
        }

        Ok(response)
    }
}

/// Builder for creating AI agents with custom configuration
///
/// # Example
///
/// ```rust,ignore
/// use kota::{AgentBuilder, PlanManager};
/// use anyhow::Result;
///
/// fn main() -> Result<()> {
///     let plan_manager = PlanManager::new();
///     let agent = AgentBuilder::new("api-key".to_string(), "gpt-4".to_string())?
///         .with_plan_manager(plan_manager)
///         .build()?;
///     Ok(())
/// }
/// ```
pub struct AgentBuilder {
    provider: Provider,
    api_key: String,
    model_name: String,
    plan_manager: PlanManager,
    context: Option<ContextManager>,
    skill_manager: Option<SkillManager>,
    tool_registry: ToolRegistry,
}

impl AgentBuilder {
    /// Create a new agent builder
    ///
    /// # Arguments
    ///
    /// * `api_key` - API key for the LLM provider
    /// * `model_name` - Model name (e.g., "gpt-4", "claude-3-5-sonnet", "deepseek-chat")
    ///
    /// # Returns
    ///
    /// Returns a builder that can be configured and built into an agent
    pub fn new(api_key: String, model_name: String) -> Result<Self> {
        let provider = Self::get_provider_from_model(&model_name)?;
        Ok(Self {
            provider,
            api_key,
            model_name,
            plan_manager: PlanManager::new(),
            context: None,
            skill_manager: None,
            tool_registry: ToolRegistry::new(),
        })
    }

    /// Set a custom plan manager for task management
    ///
    /// # Arguments
    ///
    /// * `manager` - A PlanManager instance for managing tasks and plans
    pub fn with_plan_manager(mut self, manager: PlanManager) -> Self {
        self.plan_manager = manager;
        self
    }

    /// Set a context manager for conversation history
    ///
    /// # Arguments
    ///
    /// * `context` - A ContextManager instance for managing conversation history
    pub fn with_context(mut self, context: ContextManager) -> Self {
        self.context = Some(context);
        self
    }

    /// Set a skill manager for specialized agent behaviors
    ///
    /// # Arguments
    ///
    /// * `skill_manager` - A SkillManager instance for managing agent skills
    pub fn with_skill_manager(mut self, skill_manager: SkillManager) -> Self {
        self.skill_manager = Some(skill_manager);
        self
    }

    /// Build the agent with the configured settings
    ///
    /// # Returns
    ///
    /// Returns an AgentInstance that includes the agent, context manager, and skill manager
    pub fn build(mut self) -> Result<AgentInstance> {
        // Register built-in tools first
        self.register_builtin_tools();

        // Load Lua tools from .kota/tools/mod.lua
        let lua_tools = LuaToolLoader::load_tools().unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load Lua tools: {}", e);
            Vec::new()
        });

        // Register Lua tools to the registry
        for tool in lua_tools {
            self.tool_registry.add(Box::new(tool));
        }

        let preamble = self.get_preamble();

        let agent = match self.provider {
            Provider::OpenAI => {
                build_agent!(
                    openai::Client::new(&self.api_key),
                    &self.model_name,
                    preamble,
                    self.tool_registry.take_all(),
                    OpenAI
                )
            }
            Provider::Anthropic => {
                build_agent!(
                    anthropic::Client::new(&self.api_key),
                    &self.model_name,
                    preamble,
                    self.tool_registry.take_all(),
                    Anthropic
                )
            }
            Provider::Cohere => {
                build_agent!(
                    cohere::Client::new(&self.api_key),
                    &self.model_name,
                    preamble,
                    self.tool_registry.take_all(),
                    Cohere
                )
            }
            Provider::DeepSeek => {
                build_agent!(
                    deepseek::Client::new(&self.api_key),
                    DEEPSEEK_CHAT,
                    preamble,
                    self.tool_registry.take_all(),
                    DeepSeek
                )
            }
            Provider::Ollama => {
                build_agent!(
                    ollama::Client::new(rig::client::Nothing),
                    self.model_name.strip_prefix("ollama-").unwrap(),
                    preamble,
                    self.tool_registry.take_all(),
                    Ollama
                )
            }
        };

        Ok(AgentInstance {
            agent,
            context: self.context,
            skill_manager: self.skill_manager,
            tool_registry: self.tool_registry,
        })
    }

    /// Register all built-in tools to the tool registry
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

    fn get_provider_from_model(model_name: &str) -> Result<Provider> {
        match model_name.to_lowercase().as_str() {
            // OpenAI models
            name if name.starts_with("gpt-") || name.starts_with("o1-") => Ok(Provider::OpenAI),

            // Anthropic models
            name if name.starts_with("claude-") => Ok(Provider::Anthropic),

            // Cohere models
            name if name.starts_with("command-") => Ok(Provider::Cohere),

            // DeepSeek models
            name if name.starts_with("deepseek-") => Ok(Provider::DeepSeek),
            
            // Models by ollama
            name if name.starts_with("ollama-") => Ok(Provider::Ollama),

            _ => Err(anyhow::anyhow!(
                "Unknown model: {}. Please specify a supported model.",
                model_name
            )),
        }
    }

    fn get_preamble(&self) -> &str {
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

/// Convenience function for creating an agent with default settings
///
/// # Arguments
///
/// * `api_key` - API key for the LLM provider
/// * `model_name` - Model name (e.g., "gpt-4", "claude-3-5-sonnet", "deepseek-chat")
///
/// # Example
///
/// ```rust,ignore
/// use kota::create_agent;
/// use anyhow::Result;
///
/// fn main() -> Result<()> {
///     let instance = create_agent("api-key".to_string(), "gpt-4".to_string())?;
///     Ok(())
/// }
/// ```
pub fn create_agent(api_key: String, model_name: String) -> Result<AgentInstance> {
    AgentBuilder::new(api_key, model_name)?.build()
}
