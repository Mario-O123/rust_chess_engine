// Token: lip_j8eqUXZYF9lpT9Yjqdss
pub mod lichess;
pub mod uci;
pub mod config;

#[cfg(test)]
mod test;

pub use lichess::LichessBot;
pub use uci::UciEngineHandle;
pub use config::BotConfig;