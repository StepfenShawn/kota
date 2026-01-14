use crate::agent::AgentType;
use crate::context::ContextManager;
use anyhow::Result;
use colored::*;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{self, Clear, ClearType},
};
use std::io;

mod command;
mod render;
pub mod utils;

pub struct KotaCli {
    pub api_key: String,
    pub api_base: String,
    pub model_name: String,
    pub agent: AgentType,
    pub context: ContextManager,
    pub debug_logging: bool,
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
            debug_logging: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.show_welcome()?;
        self.show_tips()?;

        // 启用原始模式
        terminal::enable_raw_mode()?;

        let result = self.run_input_loop().await;

        // 恢复正常模式
        terminal::disable_raw_mode()?;

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
        let mut input_buffer = String::new();
        let mut cursor_pos = 0; // 光标在输入缓冲区中的位置

        // 绘制初始输入框
        self.draw_input_box(&input_buffer, cursor_pos)?;

        loop {
            if let Event::Key(key_event) = event::read()? {
                // 只处理按键按下事件，忽略按键释放事件
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }

                match key_event {
                    KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        break;
                    }
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    } => {
                        if !input_buffer.trim().is_empty() {
                            // 安全地移动到输入框下方处理命令
                            let mut stdout = io::stdout();
                            let (_, terminal_height) = terminal::size()?;
                            let (_, current_row) = cursor::position()?;

                            // 检查是否有足够空间向下移动，如果没有则滚动或清屏
                            if current_row + 4 >= terminal_height {
                                // 空间不足，清屏并重新开始
                                execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                            } else {
                                // 有足够空间，正常向下移动
                                execute!(stdout, cursor::MoveDown(3), cursor::MoveToColumn(0))?;
                            }

                            let should_continue = self.handle_command(&input_buffer).await?;
                            if !should_continue {
                                break;
                            }
                            input_buffer.clear();
                            cursor_pos = 0; // 重置光标位置

                            // 重新绘制输入框进行下一轮对话
                            self.draw_input_box(&input_buffer, cursor_pos)?;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Backspace,
                        ..
                    } => {
                        if cursor_pos > 0 {
                            // 删除光标前的字符
                            let chars: Vec<char> = input_buffer.chars().collect();
                            input_buffer = chars[..cursor_pos - 1].iter().collect::<String>()
                                + &chars[cursor_pos..].iter().collect::<String>();
                            cursor_pos -= 1;
                            self.redraw_input_line(&input_buffer, cursor_pos)?;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Delete,
                        ..
                    } => {
                        if cursor_pos < input_buffer.chars().count() {
                            // 删除光标后的字符
                            let chars: Vec<char> = input_buffer.chars().collect();
                            input_buffer = chars[..cursor_pos].iter().collect::<String>()
                                + &chars[cursor_pos + 1..].iter().collect::<String>();
                            self.redraw_input_line(&input_buffer, cursor_pos)?;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Left,
                        ..
                    } => {
                        if cursor_pos > 0 {
                            cursor_pos -= 1;
                            self.update_cursor_position(&input_buffer, cursor_pos)?;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Right,
                        ..
                    } => {
                        if cursor_pos < input_buffer.chars().count() {
                            cursor_pos += 1;
                            self.update_cursor_position(&input_buffer, cursor_pos)?;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Home,
                        ..
                    } => {
                        cursor_pos = 0;
                        self.update_cursor_position(&input_buffer, cursor_pos)?;
                    }
                    KeyEvent {
                        code: KeyCode::End, ..
                    } => {
                        cursor_pos = input_buffer.chars().count();
                        self.update_cursor_position(&input_buffer, cursor_pos)?;
                    }
                    KeyEvent {
                        code: KeyCode::Char(c),
                        ..
                    } => {
                        // 在光标位置插入字符
                        let chars: Vec<char> = input_buffer.chars().collect();
                        input_buffer = chars[..cursor_pos].iter().collect::<String>()
                            + &c.to_string()
                            + &chars[cursor_pos..].iter().collect::<String>();
                        cursor_pos += 1;
                        self.redraw_input_line(&input_buffer, cursor_pos)?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
