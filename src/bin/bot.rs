use anyhow::Result;

use rust_chess_engine::bot::{BotConfig, LichessBot};

//main function to run with lichess 
#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Lichess Bot...");
    dotenv::dotenv().ok();

    // Load configuration from env
    let config = BotConfig::from_env()?;

    println!("Configuration:");
    println!("Engine path: {}", config.engine_path);
    println!("Move time: {}ms", config.movetime_ms);

    
    let mut bot = LichessBot::new(config).await?;
    bot.run().await?;

    Ok(())
}
