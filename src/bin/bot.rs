//! Entry point for running the engine as a Lichess bot.
//!
//! This binary connects to the Lichess API, listens for incoming games,
//! and delegates move calculation to the configured engine.
//!
//! Environment variables (loaded via `.env` or system environment):
//! - `LICHESS_TOKEN` (required): API token for the Lichess bot account
//! - `ENGINE_PATH` (optional): Path to the UCI engine binary
//! - `MOVETIME_MS` (optional): Thinking time per move in milliseconds
//!
//! Example `.env`:
//! LICHESS_TOKEN=lip_xxxxxxxxx
//! ENGINE_PATH=./target/release/engine
//! MOVETIME_MS=2000
//!
//! Run example:
//! cargo run --release --bin bot


use anyhow::Result;

use rust_chess_engine::bot::{BotConfig, LichessBot};

/// Async main function starting the Lichess bot.
///
/// Responsibilities:
/// 1. Load environment variables (via `.env`).
/// 2. Parse configuration using `BotConfig::from_env`.
/// 3. Initialize the `LichessBot`.
/// 4. Start the bot event loop (`run`). 
#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Lichess Bot...");
    dotenv::dotenv().ok();

    // Load configuration from env
    let config = BotConfig::from_env()?;

    println!("Configuration:");
    println!("Engine path: {}", config.engine_path);
    println!("Move time: {}ms", config.movetime_ms);

    // Initialize bot (connects to Lichess and starts engine).
    let mut bot = LichessBot::new(config).await?;
    
    // Starts main bot loop to handle events and play games.
    bot.run().await?;

    Ok(())
}
