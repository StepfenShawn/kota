use anyhow::{Ok, Result};
use colored::Colorize;
use dotenv::dotenv;
use kota::{ContextManager, SkillManager};
use names::Generator;
use std::env;

mod kota_cli;

use kota_cli::KotaCli;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let api_key = env::var("API_KEY").expect("API_KEY must be set in .env file");
    let api_base = env::var("API_BASE").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let model_name = env::var("MODEL_NAME").unwrap_or_else(|_| "gpt-4o".to_string());

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

    let context = ContextManager::new("./.chat_sessions", session_id)?.with_max_messages(100);
    let skill_manager = SkillManager::new();
    let mut cli = KotaCli::new(api_key, api_base, model_name, context, skill_manager)?;
    cli.run().await?;

    Ok(())
}
