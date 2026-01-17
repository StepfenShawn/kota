use crate::agent::AgentType;
use crate::context::ContextManager;
use anyhow::Result;
use colored::*;
use names::Generator;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::Editor;
use rustyline::{Context, Helper};
use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashSet;

mod command;
mod render;

// 自定义补全器
struct KotaHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    commands: HashSet<String>,
}

impl Default for KotaHelper {
    fn default() -> Self {
        let mut commands = HashSet::new();
        commands.insert("/quit".to_string());
        commands.insert("/exit".to_string());
        commands.insert("/config".to_string());
        commands.insert("/help".to_string());
        commands.insert("/history".to_string());
        commands.insert("/load".to_string());
        commands.insert("/sessions".to_string());
        commands.insert("/delete".to_string());

        Self {
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            validator: MatchingBracketValidator::new(),
            hinter: HistoryHinter {},
            commands,
        }
    }
}

impl Completer for KotaHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        if line.starts_with('/') {
            let input = &line[..pos];
            let mut matches = Vec::new();

            for command in &self.commands {
                if command.starts_with(input) {
                    matches.push(Pair {
                        display: command.clone(),
                        replacement: command.clone(),
                    });
                }
            }

            // 按字母顺序排序
            matches.sort_by(|a, b| a.display.cmp(&b.display));

            Ok((0, matches))
        } else {
            Ok((pos, vec![]))
        }
    }
}

impl Hinter for KotaHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for KotaHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(prompt)
        } else {
            Owned(prompt.to_string())
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(format!("{}", hint.dimmed()))
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        // 高亮显示命令
        if line.starts_with('/') {
            if let Some(space_pos) = line.find(' ') {
                let command = &line[..space_pos];
                let rest = &line[space_pos..];
                if self.commands.contains(command) {
                    return Owned(format!("{}{}", command.bright_green(), rest));
                }
            } else if self.commands.contains(line) {
                return Owned(line.bright_green().to_string());
            }
        }

        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
        self.highlighter.highlight_char(line, pos, forced)
    }
}

impl Validator for KotaHelper {
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

impl Helper for KotaHelper {}
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
    pub context: ContextManager,
}

impl KotaCli {
    pub fn new(
        api_key: String,
        api_base: String,
        model_name: String,
        agent: AgentType,
    ) -> Result<Self> {
        let session_id = {
            let mut generator = Generator::default();
            generator
                .next()
                .unwrap_or_else(|| "unknown-session".to_string())
        };

        println!(
            "{} {}",
            "🎯 Session ID:".bright_cyan(),
            session_id.bright_yellow()
        );

        // 创建上下文管理器，使用随机生成的session_id
        let context = ContextManager::new("./.chat_sessions", session_id)?.with_max_messages(100);

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
