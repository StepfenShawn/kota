use crate::agent::AgentType;
use crate::raw_println;
use crate::kota_cli::utils::with_normal_mode_async;
use anyhow::Result;
use colored::*;
use rig::agent::stream_to_stdout;
use rig::streaming::StreamingPrompt;

use super::KotaCli;

impl KotaCli {
    pub async fn handle_command(&self, input: &str) -> Result<bool> {
        match input {
            "/quit" | "/exit" => {
                return Ok(false);
            }
            "/config" => {
                self.show_config()?;
            }
            "/help" => {
                self.show_help()?;
            }
            _ if input.starts_with('/') => {
                raw_println!("{} Unknown command: {}", "❌".red(), input)?;
                raw_println!("{} Type /help for available commands", "💡".bright_blue())?;
            }
            _ => {
                raw_println!("{}", "🧠 Thinking...".yellow())?;

                raw_println!("{}", "🤖 kota:".green())?;
                
                // 使用正常模式处理流输出，避免换行问题
                let response_result = with_normal_mode_async(|| async {
                    match &self.agent {
                        AgentType::OpenAI(agent) => {
                            let mut stream = agent.stream_prompt(input).multi_turn(20).await;
                            stream_to_stdout(&mut stream).await
                        }
                        AgentType::Anthropic(agent) => {
                            let mut stream = agent.stream_prompt(input).multi_turn(20).await;
                            stream_to_stdout(&mut stream).await
                        }
                        AgentType::Cohere(agent) => {
                            let mut stream = agent.stream_prompt(input).multi_turn(20).await;
                            stream_to_stdout(&mut stream).await
                        }
                        AgentType::DeepSeek(agent) => {
                            let mut stream = agent.stream_prompt(input).multi_turn(20).await;
                            stream_to_stdout(&mut stream).await
                        }
                        AgentType::Ollama(agent) => {
                            let mut stream = agent.stream_prompt(input).multi_turn(20).await;
                            stream_to_stdout(&mut stream).await
                        }
                    }
                }).await;
                
                raw_println!()?;

                match response_result {
                    Ok(resp) => {
                        raw_println!(
                            "{} Total tokens used: {}",
                            "📊".bright_blue(),
                            resp.usage().total_tokens
                        )?;
                    }
                    Err(e) => {
                        raw_println!("{} Failed to get AI response: {}", "❌".red(), e)?;
                        raw_println!(
                            "{} Please check your API key and network connection",
                            "💡".bright_blue()
                        )?;
                    }
                }
            }
        }
        raw_println!()?; // 添加空行分隔
        Ok(true)
    }

    fn show_config(&self) -> Result<()> {
        raw_println!("{}", "⚙️  Current Configuration:".bright_cyan())?;
        raw_println!("  {} {}", "API Base:".bright_white(), self.api_base)?;
        raw_println!("  {} {}", "Model:".bright_white(), self.model_name)?;
        raw_println!(
            "  {} {}",
            "API Key:".bright_white(),
            "*".repeat(self.api_key.len().min(8))
        )?;
        raw_println!()?;
        Ok(())
    }

    fn show_help(&self) -> Result<()> {
        raw_println!("{}", "📚 Available Commands:".bright_cyan())?;
        raw_println!()?;
        raw_println!("  {} - Exit the application", "/quit".bright_green())?;
        raw_println!(
            "  {} - Show current model configuration",
            "/config".bright_green()
        )?;
        raw_println!("  {} - Show this help message", "/help".bright_green())?;
        raw_println!()?;
        raw_println!(
            "{}",
            "💡 You can also type any message to chat with the AI!".bright_white()
        )?;
        raw_println!()?;
        Ok(())
    }
}
