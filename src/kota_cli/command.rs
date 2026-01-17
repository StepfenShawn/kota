use crate::agent::AgentType;
use crate::context::{ContextManager, SerializableMessage};
use crate::hooks::SessionIdHook;
use anyhow::Result;
use colored::*;
use rig::agent::stream_to_stdout;
use rig::completion::Message;
use rig::streaming::StreamingPrompt;

use super::KotaCli;

impl KotaCli {
    pub async fn handle_command(&mut self, input: &str) -> Result<bool> {
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
            "/history" => {
                self.show_history()?;
            }
            _ if input.starts_with("/load ") => {
                let session_id = input.strip_prefix("/load ").unwrap_or("").trim();
                if session_id.is_empty() {
                    self.list_sessions()?;
                } else {
                    self.load_session(session_id)?;
                }
            }
            _ if input.starts_with("/delete ") => {
                let session_id = input.strip_prefix("/delete ").unwrap_or("").trim();
                if !session_id.is_empty() {
                    self.delete_session(session_id)?;
                } else {
                    println!("{} Usage: /delete <session_id>", "❌".red());
                }
            }
            _ if input.starts_with('/') => {
                println!("{} Unknown command: {}", "❌".red(), input);
                println!("{} Type /help for available commands", "💡".bright_blue());
            }
            _ => {
                // 添加用户消息到上下文
                self.context.add_message(Message::user(input));

                println!("{}", "🧠 Thinking...".yellow());
                println!("{}", "● kota:".blue());

                // 创建会话钩子
                let hook = SessionIdHook::new(self.context.session_id().to_string());

                let response_result = match &self.agent {
                    AgentType::OpenAI(agent) => {
                        let mut stream = agent
                            .stream_prompt(input)
                            .with_hook(hook.clone())
                            .multi_turn(20)
                            .await;
                        stream_to_stdout(&mut stream).await
                    }
                    AgentType::Anthropic(agent) => {
                        let mut stream = agent
                            .stream_prompt(input)
                            .with_hook(hook.clone())
                            .multi_turn(20)
                            .await;
                        stream_to_stdout(&mut stream).await
                    }
                    AgentType::Cohere(agent) => {
                        let mut stream = agent
                            .stream_prompt(input)
                            .with_hook(hook.clone())
                            .multi_turn(20)
                            .await;
                        stream_to_stdout(&mut stream).await
                    }
                    AgentType::DeepSeek(agent) => {
                        let mut stream = agent
                            .stream_prompt(input)
                            .with_hook(hook.clone())
                            .multi_turn(20)
                            .await;
                        stream_to_stdout(&mut stream).await
                    }
                    AgentType::Ollama(agent) => {
                        let mut stream = agent
                            .stream_prompt(input)
                            .with_hook(hook.clone())
                            .multi_turn(20)
                            .await;
                        stream_to_stdout(&mut stream).await
                    }
                };

                println!();

                match response_result {
                    Ok(resp) => {
                        // 获取响应内容并添加到上下文
                        let response_content = resp.response();
                        self.context
                            .add_message(Message::assistant(response_content));

                        // 自动保存上下文（包含用户消息和助手响应）
                        if let Err(e) = self.context.save() {
                            println!("{} Failed to save context: {}", "⚠️".yellow(), e);
                        }

                        println!(
                            "{} Total tokens used: {}",
                            "📊".bright_blue(),
                            resp.usage().total_tokens
                        );
                    }
                    Err(e) => {
                        println!("{} Failed to get AI response: {}", "❌".red(), e);
                        println!(
                            "{} Please check your API key and network connection",
                            "💡".bright_blue()
                        );
                    }
                }
            }
        }
        println!(); // 添加空行分隔
        Ok(true)
    }

    fn show_config(&self) -> Result<()> {
        println!("{}", "⚙️  Current Configuration:".bright_cyan());
        println!("  {} {}", "API Base:".bright_white(), self.api_base);
        println!("  {} {}", "Model:".bright_white(), self.model_name);
        println!(
            "  {} {}",
            "API Key:".bright_white(),
            "*".repeat(self.api_key.len().min(8))
        );
        println!();
        Ok(())
    }

    fn show_help(&self) -> Result<()> {
        println!("{}", "📚 Available Commands:".bright_cyan());
        println!();
        println!("  {} - Exit the application", "/quit".bright_green());
        println!(
            "  {} - Show current model configuration",
            "/config".bright_green()
        );
        println!(
            "  {} - Show conversation history",
            "/history".bright_green()
        );
        println!(
            "  {} - List all sessions or load specific session",
            "/load [session_id]".bright_green()
        );
        println!(
            "  {} - Delete a specific session",
            "/delete <session_id>".bright_green()
        );
        println!("  {} - Show this help message", "/help".bright_green());
        println!();
        println!(
            "{}",
            "💡 You can also type any message to chat with the AI!".bright_white()
        );
        println!();
        Ok(())
    }

    fn show_history(&self) -> Result<()> {
        let messages = self.context.get_messages();
        if messages.is_empty() {
            println!(
                "{} No conversation history in current session",
                "📝".bright_blue()
            );
            println!(
                "  Current session: {}",
                self.context.session_id().bright_white()
            );
        } else {
            println!(
                "{} Conversation History (Session: {})",
                "📝".bright_blue(),
                self.context.session_id().bright_white()
            );
            println!();

            for (i, message) in messages.iter().enumerate() {
                let serializable = SerializableMessage::from(message);
                let role_color = match serializable.role.as_str() {
                    "user" => "👤 User".bright_cyan(),
                    "assistant" => "🤖 Assistant".bright_green(),
                    _ => "❓ Unknown".bright_yellow(),
                };

                println!("{}. {}", (i + 1).to_string().bright_white(), role_color);

                // 限制显示长度，避免输出过长
                let content = if serializable.content.chars().count() > 200 {
                    format!(
                        "{}...",
                        serializable.content.chars().take(200).collect::<String>()
                    )
                } else {
                    serializable.content
                };

                // 缩进显示内容
                for line in content.lines() {
                    println!("   {}", line);
                }
                println!();
            }

            println!("{} Total messages: {}", "📊".bright_blue(), messages.len());
        }
        println!();
        Ok(())
    }

    fn list_sessions(&self) -> Result<()> {
        match self.context.list_sessions() {
            Ok(sessions) => {
                if sessions.is_empty() {
                    println!("{} No saved sessions found", "📁".bright_blue());
                } else {
                    println!("{} Available Sessions:", "📁".bright_blue());
                    println!();

                    for (i, session) in sessions.iter().enumerate() {
                        let current_marker = if session.session_id == self.context.session_id() {
                            " (current)".bright_green()
                        } else {
                            "".normal()
                        };

                        println!(
                            "{}. {} - {} messages{}",
                            (i + 1).to_string().bright_white(),
                            session.session_id.bright_cyan(),
                            session.message_count.to_string().bright_yellow(),
                            current_marker
                        );
                        println!("   Last updated: {}", session.last_updated.dimmed());
                    }

                    println!();
                    println!(
                        "{} Use '/load <session_id>' to load a session",
                        "💡".bright_blue()
                    );
                }
            }
            Err(e) => {
                println!("{} Failed to list sessions: {}", "❌".red(), e);
            }
        }
        println!();
        Ok(())
    }

    fn load_session(&mut self, session_id: &str) -> Result<()> {
        // 保存当前会话
        if let Err(e) = self.context.save() {
            println!(
                "{} Warning: Failed to save current session: {}",
                "⚠️".yellow(),
                e
            );
        }

        // 切换到新会话
        self.context.switch_session(session_id.to_string());

        match self.context.load() {
            Ok(true) => {
                println!(
                    "{} Successfully loaded session: {}",
                    "✅".bright_green(),
                    session_id.bright_cyan()
                );
                println!(
                    "   Messages loaded: {}",
                    self.context
                        .get_messages()
                        .len()
                        .to_string()
                        .bright_yellow()
                );
            }
            Ok(false) => {
                println!(
                    "{} Session '{}' not found, created new session",
                    "📝".bright_blue(),
                    session_id.bright_cyan()
                );
            }
            Err(e) => {
                println!(
                    "{} Failed to load session '{}': {}",
                    "❌".red(),
                    session_id.bright_cyan(),
                    e
                );
            }
        }
        println!();
        Ok(())
    }

    fn delete_session(&mut self, session_id: &str) -> Result<()> {
        if session_id == self.context.session_id() {
            println!("{} Cannot delete current active session", "❌".red());
            println!("   Switch to another session first using '/load <session_id>'",);
            return Ok(());
        }

        // 创建临时上下文管理器来删除指定会话
        let temp_context = ContextManager::new("./.chat_sessions", session_id.to_string())?;

        match temp_context.delete_session() {
            Ok(true) => {
                println!(
                    "{} Successfully deleted session: {}",
                    "✅".bright_green(),
                    session_id.bright_cyan()
                );
            }
            Ok(false) => {
                println!(
                    "{} Session '{}' not found",
                    "❌".red(),
                    session_id.bright_cyan()
                );
            }
            Err(e) => {
                println!(
                    "{} Failed to delete session '{}': {}",
                    "❌".red(),
                    session_id.bright_cyan(),
                    e
                );
            }
        }
        println!();
        Ok(())
    }
}
