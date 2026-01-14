use anyhow::Result;
use colored::*;
use crossterm::{
    cursor, execute,
    terminal::{self, Clear, ClearType},
};
use std::io::{self, Write};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::raw_println;

use super::KotaCli;

impl KotaCli {
    pub fn draw_input_box(&self, input: &str, cursor_pos: usize) -> Result<()> {
        let mut stdout = io::stdout();
        let (terminal_width, terminal_height) = crossterm::terminal::size()?;
        let box_width = (terminal_width as usize).min(80); // 适应终端宽度

        // 检查是否有足够的垂直空间绘制输入框（需要5行）
        let (_, current_row) = cursor::position()?;
        if current_row + 5 >= terminal_height {
            // 空间不足，向下滚动5行来腾出空间
            // 移动到终端底部
            execute!(stdout, cursor::MoveTo(0, terminal_height))?;
            // 插入5个空行，这会让现有内容向下滚动
            for _ in 0..5 {
                println!();
            }
            // 移动到底部准备绘制输入框（预留5行空间）
            execute!(stdout, cursor::MoveTo(0, terminal_height - 5))?;
        }

        // 清除可能存在的旧内容
        execute!(stdout, cursor::MoveToColumn(0))?;
        for _ in 0..5 {
            execute!(stdout, Clear(ClearType::CurrentLine))?;
            if cursor::position()?.1 < terminal_height - 1 {
                execute!(stdout, cursor::MoveDown(1))?;
            }
        }

        // 回到起始位置
        for _ in 0..5 {
            if cursor::position()?.1 > 0 {
                execute!(stdout, cursor::MoveUp(1))?;
            }
        }

        // 绘制输入框顶部
        execute!(stdout, cursor::MoveToColumn(0))?;
        print!("{}", "┌".dimmed());
        for _ in 0..box_width.saturating_sub(2) {
            print!("{}", "─".dimmed());
        }
        println!("{}", "┐".dimmed());

        // 绘制输入内容行
        execute!(stdout, cursor::MoveToColumn(0))?;
        print!("{}", "│".dimmed());
        print!(" {}", "❯".bright_green());

        if input.is_empty() {
            let placeholder = "Type your message...";
            print!(" {}", placeholder.dimmed());
            let used_chars = 4 + Self::display_width(placeholder);
            let remaining = box_width.saturating_sub(used_chars + 1);
            for _ in 0..remaining {
                print!(" ");
            }
        } else {
            // 处理输入内容可能超出终端宽度的情况
            let max_input_width = box_width.saturating_sub(5); // "│ ❯ " + "│"
            let display_input = if Self::display_width(input) > max_input_width {
                // 如果输入太长，只显示末尾部分
                Self::truncate_from_end(input, max_input_width)
            } else {
                input.to_string()
            };

            print!(" {}", display_input);
            let used_chars = 4 + display_input.chars().count();
            let remaining = box_width.saturating_sub(used_chars + 1);
            for _ in 0..remaining {
                print!(" ");
            }
        }
        println!("{}", "│".dimmed());

        // 绘制输入框底部
        execute!(stdout, cursor::MoveToColumn(0))?;
        print!("{}", "└".dimmed());
        for _ in 0..box_width.saturating_sub(2) {
            print!("{}", "─".dimmed());
        }
        println!("{}", "┘".dimmed());

        // 绘制提示信息
        execute!(stdout, cursor::MoveToColumn(0))?;
        let tip_text = "ctrl+c to exit, /help to help";
        if tip_text.len() <= terminal_width as usize {
            println!("{}", tip_text.dimmed());
        } else {
            println!("{}", "? for shortcuts, ctrl+c to exit".dimmed());
        }

        // 将光标定位到输入框内的正确位置
        execute!(stdout, cursor::MoveUp(3))?; // 回到输入行
        let cursor_position = if input.is_empty() {
            4 // "│ ❯ " 后面，准备输入
        } else {
            let max_input_width = box_width.saturating_sub(5);
            let chars: Vec<char> = input.chars().collect();
            let text_before_cursor = chars[..cursor_pos.min(chars.len())]
                .iter()
                .collect::<String>();

            if Self::display_width(input) > max_input_width {
                // 如果输入太长，需要计算滚动偏移
                let display_input = Self::truncate_from_end(input, max_input_width);
                let display_chars: Vec<char> = display_input.chars().collect();
                let cursor_in_display = if cursor_pos >= chars.len() - display_chars.len() {
                    cursor_pos - (chars.len() - display_chars.len())
                } else {
                    0
                };
                4 + Self::display_width(
                    &display_chars[..cursor_in_display.min(display_chars.len())]
                        .iter()
                        .collect::<String>(),
                )
            } else {
                4 + Self::display_width(&text_before_cursor) // "│ ❯ " + 光标前的内容宽度
            }
        };
        execute!(
            stdout,
            cursor::MoveToColumn(cursor_position.min(terminal_width as usize - 1) as u16)
        )?;

        stdout.flush()?;
        Ok(())
    }

    pub fn redraw_input_line(&self, input: &str, cursor_pos: usize) -> Result<()> {
        let mut stdout = io::stdout();
        let (terminal_width, _) = terminal::size()?;
        let box_width = (terminal_width as usize).min(80); // 与 draw_input_box 保持一致

        // 移动到输入行
        execute!(stdout, cursor::MoveToColumn(0))?;

        // 清除输入行
        execute!(stdout, Clear(ClearType::CurrentLine))?;

        // 重新绘制输入行
        print!("{}", "│".dimmed());
        print!(" {}", "❯".bright_green());

        let is_empty = input.is_empty();

        if is_empty {
            let placeholder = "Type your message...";
            print!(" {}", placeholder.dimmed());
            let used_chars = 4 + Self::display_width(placeholder);
            let remaining = box_width.saturating_sub(used_chars + 1);
            for _ in 0..remaining {
                print!(" ");
            }
        } else {
            // 处理输入内容可能超出终端宽度的情况
            let max_input_width = box_width.saturating_sub(5); // "│ ❯ " + "│"
            let display_input = if Self::display_width(input) > max_input_width {
                // 如果输入太长，只显示末尾部分
                Self::truncate_from_end(input, max_input_width)
            } else {
                input.to_string()
            };

            print!(" {}", display_input);
            let used_chars = 4 + Self::display_width(&display_input);
            let remaining = box_width.saturating_sub(used_chars + 1);
            for _ in 0..remaining {
                print!(" ");
            }
        }
        print!("{}", "│".dimmed());

        // 将光标定位到输入位置
        let cursor_position = if is_empty {
            4 // "│ ❯ " 后面
        } else {
            let max_input_width = box_width.saturating_sub(5);
            let chars: Vec<char> = input.chars().collect();
            let text_before_cursor = chars[..cursor_pos.min(chars.len())]
                .iter()
                .collect::<String>();

            if Self::display_width(input) > max_input_width {
                // 如果输入太长，需要计算滚动偏移
                let display_input = Self::truncate_from_end(input, max_input_width);
                let display_chars: Vec<char> = display_input.chars().collect();
                let cursor_in_display = if cursor_pos >= chars.len() - display_chars.len() {
                    cursor_pos - (chars.len() - display_chars.len())
                } else {
                    0
                };
                4 + Self::display_width(
                    &display_chars[..cursor_in_display.min(display_chars.len())]
                        .iter()
                        .collect::<String>(),
                )
            } else {
                4 + Self::display_width(&text_before_cursor) // "│ ❯ " + 光标前的内容宽度
            }
        };
        execute!(
            stdout,
            cursor::MoveToColumn(cursor_position.min(terminal_width as usize - 1) as u16)
        )?;

        stdout.flush()?;
        Ok(())
    }

    pub fn update_cursor_position(&self, input: &str, cursor_pos: usize) -> Result<()> {
        let mut stdout = io::stdout();
        let (terminal_width, _) = terminal::size()?;
        let box_width = (terminal_width as usize).min(80);

        let cursor_position = if input.is_empty() {
            4 // "│ ❯ " 后面
        } else {
            let max_input_width = box_width.saturating_sub(5);
            let chars: Vec<char> = input.chars().collect();
            let text_before_cursor = chars[..cursor_pos.min(chars.len())]
                .iter()
                .collect::<String>();

            if Self::display_width(input) > max_input_width {
                // 如果输入太长，需要计算滚动偏移
                let display_input = Self::truncate_from_end(input, max_input_width);
                let display_chars: Vec<char> = display_input.chars().collect();
                let cursor_in_display = if cursor_pos >= chars.len() - display_chars.len() {
                    cursor_pos - (chars.len() - display_chars.len())
                } else {
                    0
                };
                4 + Self::display_width(
                    &display_chars[..cursor_in_display.min(display_chars.len())]
                        .iter()
                        .collect::<String>(),
                )
            } else {
                4 + Self::display_width(&text_before_cursor) // "│ ❯ " + 光标前的内容宽度
            }
        };

        execute!(
            stdout,
            cursor::MoveToColumn(cursor_position.min(terminal_width as usize - 1) as u16)
        )?;
        stdout.flush()?;
        Ok(())
    }

    pub fn show_welcome(&self) -> Result<()> {
        raw_println!("{}", "✨ Welcome to Kota CLI! 0.1.1".bright_green())?;
        raw_println!(
            "{} {}",
            "cwd:".dimmed(),
            std::env::current_dir().unwrap().display()
        )?;
        raw_println!()?;
        Ok(())
    }

    pub fn show_tips(&self) -> Result<()> {
        raw_println!("{}", "Tips for getting started:".bright_white())?;
        raw_println!()?;
        raw_println!(
            "{} Ask questions, edit files, or run commands.",
            "1.".bright_white()
        )?;
        raw_println!("{} Be specific for the best results.", "2.".bright_white())?;
        raw_println!("{} Type /help for more information.", "3.".bright_white())?;
        raw_println!()?;
        raw_println!(
            "{}",
            "? for shortcuts, ctrl+c to exit, ctrl+f to add images".dimmed()
        )?;
        raw_println!()?;
        Ok(())
    }

    // 获取字符串的显示宽度
    pub fn display_width(s: &str) -> usize {
        s.width()
    }

    // 从字符串末尾截取指定显示宽度
    pub fn truncate_from_end(s: &str, max_width: usize) -> String {
        let chars: Vec<char> = s.chars().collect();
        let mut result = String::new();
        let mut current_width = 0;

        for &ch in chars.iter().rev() {
            let char_width = ch.width().unwrap_or(0);
            if current_width + char_width > max_width {
                break;
            }
            result.insert(0, ch);
            current_width += char_width;
        }
        result
    }
}
