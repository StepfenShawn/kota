use anyhow::Result;
use colored::*;
use kota::kota_code::agent::{AgentBuilder, AgentInstance};
use kota::kota_code::context::ContextManager;
use kota::kota_code::skills::SkillManager;
use rustyline::error::ReadlineError;
use rustyline::Editor;

mod command;
mod render;
mod tab;

use tab::KotaHelper;

const LOGO: &str = r#"

██╗░░██╗░█████╗░████████╗░█████╗░
██║░██╔╝██╔══██╗╚══██╔══╝██╔══██╗
█████═╝░██║░░██║░░░██║░░░███████║
██╔═██╗░██║░░██║░░░██║░░░██╔══██║
██║░╚██╗╚█████╔╝░░░██║░░░██║░░██║
╚═╝░░╚═╝░╚════╝░░░░╚═╝░░░╚═╝░░╚═╝
"#;

pub struct KotaCli {
    pub agent_instance: AgentInstance,
    pub api_base: String,
    pub model_name: String,
    pub api_key: String,
}

impl KotaCli {
    pub fn new(
        api_key: String,
        api_base: String,
        model_name: String,
        context: ContextManager,
        skill_manager: SkillManager,
    ) -> Result<Self> {
        let agent_instance = AgentBuilder::new(api_key.clone(), model_name.clone())?
            .with_context(context)
            .with_skill_manager(skill_manager)
            .build()?;

        Ok(Self {
            agent_instance,
            api_base,
            model_name,
            api_key,
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
        let mut rl = Editor::new()?;
        rl.set_helper(Some(KotaHelper::default()));

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
