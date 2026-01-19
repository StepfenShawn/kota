use anyhow::Result;
use rig::{
    agent::Agent,
    client::CompletionClient,
    providers::{
        anthropic, cohere,
        deepseek::{self, DEEPSEEK_CHAT},
        ollama, openai,
    },
};

use crate::tools::{
    WrappedCreateDirectoryTool, WrappedDeleteFileTool, WrappedEditFileTool,
    WrappedExecuteBashCommandTool, WrappedGrepSearchTool, WrappedReadFileTool,
    WrappedScanCodebaseTool, WrappedWriteFileTool,
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
            .build();
        Ok(AgentType::$variant(agent))
    }};
}

#[derive(Debug, Clone)]
pub enum Provider {
    OpenAI,
    Anthropic,
    Cohere,
    DeepSeek,
    Ollama,
}

// Agent enum to handle different provider types
pub enum AgentType {
    OpenAI(Agent<openai::responses_api::ResponsesCompletionModel>),
    Anthropic(Agent<anthropic::completion::CompletionModel>),
    Cohere(Agent<cohere::CompletionModel>),
    DeepSeek(Agent<deepseek::CompletionModel>),
    Ollama(Agent<ollama::CompletionModel>),
}

pub struct AgentBuilder {
    provider: Provider,
    api_key: String,
    model_name: String,
}

impl AgentBuilder {
    pub fn new(api_key: String, model_name: String) -> Result<Self> {
        let provider = Self::get_provider_from_model(&model_name)?;
        Ok(Self {
            provider,
            api_key,
            model_name,
        })
    }

    pub fn build(self) -> Result<AgentType> {
        let tools = self.create_tools();
        let preamble = self.get_preamble();

        match self.provider {
            Provider::OpenAI => {
                build_agent!(
                    openai::Client::new(&self.api_key),
                    &self.model_name,
                    &preamble,
                    tools,
                    OpenAI
                )
            }
            Provider::Anthropic => {
                build_agent!(
                    anthropic::Client::new(&self.api_key),
                    &self.model_name,
                    &preamble,
                    tools,
                    Anthropic
                )
            }
            Provider::Cohere => {
                build_agent!(
                    cohere::Client::new(&self.api_key),
                    &self.model_name,
                    &preamble,
                    tools,
                    Cohere
                )
            }
            Provider::DeepSeek => {
                build_agent!(
                    deepseek::Client::new(&self.api_key),
                    DEEPSEEK_CHAT,
                    &preamble,
                    tools,
                    DeepSeek
                )
            }
            Provider::Ollama => {
                build_agent!(
                    ollama::Client::new(rig::client::Nothing),
                    &self.model_name,
                    &preamble,
                    tools,
                    Ollama
                )
            }
        }
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
        }
    }

    fn get_preamble(&self) -> String {
        r#"
        Your name is Kato. You are a helpful AI code assistant with comprehensive file system and command execution access. 
        You can read, write, edit (with patches), and delete files, execute bash commands, scan codebase structures,  search text in the codebase and create directories. 
        Use the edit_file tool for making small, targeted changes to existing files - it's more efficient than rewriting entire files.
        Please provide clear and concise responses and be careful when modifying files or executing commands."#.to_string()
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
}

// Convenience function for creating an agent
pub fn create_agent(api_key: String, model_name: String) -> Result<AgentType> {
    AgentBuilder::new(api_key, model_name)?.build()
}
