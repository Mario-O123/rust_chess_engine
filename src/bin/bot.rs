use anyhow::Result;

use rust_chess_engine::bot::{BotConfig, LichessBot};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Lichess Bot...");
    dotenv::dotenv().ok();

    // Load configuration from environment
    let config = BotConfig::from_env()?;

    println!("Configuration:");
    println!("Engine path: {}", config.engine_path);
    println!("Move time: {}ms", config.movetime_ms);

    // Initialize bot
    let mut bot = LichessBot::new(config).await?;

    // Run bot (blocks until error or Ctrl+C)
    bot.run().await?;

    Ok(())
}
