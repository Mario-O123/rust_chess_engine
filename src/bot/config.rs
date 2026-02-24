use anyhow::Result;

#[derive(Debug, Clone)]
pub struct BotConfig {
    pub lichess_token: String,
    pub engine_path: String,
    pub movetime_ms: u64,
}

impl BotConfig {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let lichess_token = std::env::var("LICHESS_TOKEN")
            .map_err(|_| anyhow::anyhow!("LICHESS_TOKEN environment variable not set"))?;

        let engine_path = std::env::var("ENGINE_PATH")
            .unwrap_or_else(|_| "./target/release/chess-engine".to_string());

        let movetime_ms = std::env::var("MOVETIME_MS")
            .unwrap_or_else(|_| "1000".to_string())
            .parse()
            .unwrap_or(1000);

        Ok(Self {
            lichess_token,
            engine_path,
            movetime_ms,
        })
    }
}
