use crate::agent::AgentType;
use crate::context::ContextManager;
use anyhow::Result;
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

mod command;
mod render;
const LOGO: &str = r#"

██╗░░██╗░█████╗░████████╗░█████╗░
██║░██╔╝██╔══██╗╚══██╔══╝██╔══██╗
█████═╝░██║░░██║░░░██║░░░███████║
██╔═██╗░██║░░██║░░░██║░░░██╔══██║
██║░╚██╗╚█████╔╝░░░██║░░░██║░░██║
╚═╝░░╚═╝░╚════╝░░░░╚═╝░░░╚═╝░░╚═╝
"#;

pub struct KotaCli {
    pub api_key: String,
    pub api_base: String,
    pub model_name: String,
    pub agent: AgentType,
    pub context: ContextManager
}

impl KotaCli {
    pub fn new(
        api_key: String,
        api_base: String,
        model_name: String,
        agent: AgentType,
    ) -> Result<Self> {
        // 创建默认的上下文管理器
        let context =
            ContextManager::new("./.chat_sessions", "default".to_string())?.with_max_messages(100);

        Ok(Self {
            api_key,
            api_base,
            model_name,
            agent,
            context,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("{}", LOGO);
        self.show_welcome()?;
        self.show_tips()?;

        let result = self.run_input_loop().await;

        match result {
            Ok(_) => println!("\n{}", "👋 Goodbye!".bright_cyan()),
            Err(e) => {
                println!("\n{} {}", "❌ Error:".red(), e);
                return Err(e);
            }
        }

        Ok(())
    }

    async fn run_input_loop(&mut self) -> Result<()> {
        let mut rl = DefaultEditor::new()?;

        loop {
            self.print_separator()?;
            let readline = rl.readline("❯ ");

            match readline {
                Ok(line) => {
                    let input = line.trim();
                    if input.is_empty() {
                        continue;
                    }

                    // 添加到历史记录
                    let _ = rl.add_history_entry(input);

                    // 显示分隔线
                    self.print_separator()?;

                    // 处理命令
                    let should_continue = self.handle_command(input).await?;
                    if !should_continue {
                        break;
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl-C
                    println!("{}", "^C".dimmed());
                    break;
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl-D
                    break;
                }
                Err(err) => {
                    println!("{} {:?}", "Error:".red(), err);
                    break;
                }
            }
        }

        Ok(())
    }

    fn print_separator(&self) -> Result<()> {
        let width = 80; // 默认宽度
        let separator = "-".repeat(width);
        println!("{}", separator.dimmed());
        Ok(())
    }
}
