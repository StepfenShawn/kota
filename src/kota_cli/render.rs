use anyhow::Result;
use colored::*;

use super::KotaCli;

impl KotaCli {
    pub fn show_welcome(&self) -> Result<()> {
        println!("{}", "✨ Welcome to Kota CLI! 0.1.1".bright_green());
        println!(
            "{} {}",
            "cwd:".dimmed(),
            std::env::current_dir().unwrap().display()
        );
        println!();
        Ok(())
    }

    pub fn show_tips(&self) -> Result<()> {
        println!("{}", "Tips for getting started:".bright_white());
        println!();
        println!(
            "{} Ask questions, edit files, or run commands.",
            "1.".bright_white()
        );
        println!("{} Be specific for the best results.", "2.".bright_white());
        println!("{} Type /help for more information.", "3.".bright_white());
        println!();
        println!("{}", "ctrl+c to exit, /help for commands".dimmed());
        println!();
        Ok(())
    }
}
