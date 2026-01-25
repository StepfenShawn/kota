use anyhow::Result;
use rig::{
    agent::Agent,
    client::CompletionClient,
    providers::{
        anthropic, cohere,
        deepseek::{self, DEEPSEEK_CHAT},
        ollama, openai,
    },
    streaming::StreamingPrompt,
};

use super::context::ContextManager;
use super::plan::PlanManager;
use super::skills::SkillManager;
use super::tools::{
    WrappedCreateDirectoryTool, WrappedDeleteFileTool, WrappedEditFileTool,
    WrappedExecuteBashCommandTool, WrappedGrepSearchTool, WrappedReadFileTool,
    WrappedScanCodebaseTool, WrappedUpdatePlanTool, WrappedWriteFileTool,
};

macro_rules! build_agent {
    ($client_expr:expr, $model_name:expr, $preamble:expr, $tools:expr, $variant:ident) => {{
        let client = $client_expr?;
        let agent = client
            .agent($model_name)
            .preamble($preamble)
            .max_tokens(4096)
            .tool($tools.read_file)
            .tool($tools.write_file)
            .tool($tools.edit_file)
            .tool($tools.delete_file)
            .tool($tools.execute_bash)
            .tool($tools.scan_codebase)
            .tool($tools.make_dir)
            .tool($tools.grep_find)
            .tool($tools.update_plan)
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
    /// ```rust,no_run
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
        use super::hooks::SessionIdHook;
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
        let history = self
            .context()
            .map(|c| c.get_messages().to_vec())
            .unwrap_or_default();

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
/// ```rust,no_run
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
    pub fn build(self) -> Result<AgentInstance> {
        let tools = self.create_tools();
        let preamble = self.get_preamble();

        let agent = match self.provider {
            Provider::OpenAI => {
                build_agent!(
                    openai::Client::new(&self.api_key),
                    &self.model_name,
                    preamble,
                    tools,
                    OpenAI
                )
            }
            Provider::Anthropic => {
                build_agent!(
                    anthropic::Client::new(&self.api_key),
                    &self.model_name,
                    preamble,
                    tools,
                    Anthropic
                )
            }
            Provider::Cohere => {
                build_agent!(
                    cohere::Client::new(&self.api_key),
                    &self.model_name,
                    preamble,
                    tools,
                    Cohere
                )
            }
            Provider::DeepSeek => {
                build_agent!(
                    deepseek::Client::new(&self.api_key),
                    DEEPSEEK_CHAT,
                    preamble,
                    tools,
                    DeepSeek
                )
            }
            Provider::Ollama => {
                build_agent!(
                    ollama::Client::new(rig::client::Nothing),
                    &self.model_name,
                    preamble,
                    tools,
                    Ollama
                )
            }
        };

        Ok(AgentInstance {
            agent,
            context: self.context,
            skill_manager: self.skill_manager,
        })
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
            "ollama" | "local" => Ok(Provider::Ollama),

            _ => Err(anyhow::anyhow!(
                "Unknown model: {}. Please specify a supported model.",
                model_name
            )),
        }
    }

    fn create_tools(&self) -> AgentTools {
        AgentTools {
            read_file: WrappedReadFileTool::new(),
            write_file: WrappedWriteFileTool::new(),
            edit_file: WrappedEditFileTool::new(),
            delete_file: WrappedDeleteFileTool::new(),
            execute_bash: WrappedExecuteBashCommandTool::new(),
            scan_codebase: WrappedScanCodebaseTool::new(),
            make_dir: WrappedCreateDirectoryTool::new(),
            grep_find: WrappedGrepSearchTool::new(),
            update_plan: WrappedUpdatePlanTool::new(self.plan_manager.clone()),
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

struct AgentTools {
    read_file: WrappedReadFileTool,
    write_file: WrappedWriteFileTool,
    edit_file: WrappedEditFileTool,
    delete_file: WrappedDeleteFileTool,
    execute_bash: WrappedExecuteBashCommandTool,
    scan_codebase: WrappedScanCodebaseTool,
    make_dir: WrappedCreateDirectoryTool,
    grep_find: WrappedGrepSearchTool,
    update_plan: WrappedUpdatePlanTool,
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
/// ```rust,no_run
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
