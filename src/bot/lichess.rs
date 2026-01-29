use reqwest::Client;
use serde::Deserialize;
use futures::StreamExt;
use anyhow::Result;
use super::{UciEngineHandle, BotConfig};

pub const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, Deserialize)]
struct Event {
    #[serde(rename = "type")]
    event_type: String,
    game: Option<GameInfo>,
    challenge: Option<ChallengeInfo>,
}

#[derive(Debug, Deserialize)]
struct GameInfo {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ChallengeInfo {
    id: String,
}

#[derive(Debug, Deserialize)]
struct GameFull {
    #[serde(rename = "type")]
    event_type: String,
    id: String,
    white: Option<Player>,
    black: Option<Player>,
    #[serde(rename = "initialFen")]
    initial_fen: String,
    state: GameState,
}

#[derive(Debug, Deserialize)]
struct Player {
    id: String,
}

#[derive(Debug, Deserialize)]
struct GameState {
    #[serde(rename = "type")]
    event_type: String,
    moves: String,
    status: String,
}

pub struct LichessBot {
    client: Client,
    config: BotConfig,
    engine: UciEngineHandle,
    bot_id: String,
}

impl LichessBot {
    pub async fn new(config: BotConfig) -> Result<Self> {
        // Create client with User-Agent (required by Lichess)
        let client = Client::builder()
            .user_agent("rust-chess-bot/0.1.0")
            .build()?;
        
        let engine = UciEngineHandle::new(&config.engine_path)?;
        
        // Get bot account info
        let response = client
            .get("https://lichess.org/api/account")
            .header("Authorization", format!("Bearer {}", config.lichess_token))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to authenticate with Lichess: {}", response.status()));
        }
        
        let account: serde_json::Value = response.json().await?;
        let bot_id = account["id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("No bot id in account info"))?
            .to_string();
        
        println!("Logged in as: {}", account["username"]);
        
        Ok(Self { client, config, engine, bot_id })
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("Bot running, waiting for games...");
        
        let url = "https://lichess.org/api/stream/event";
        let response = self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.config.lichess_token))
            .send()
            .await?;

        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk);
            
            for line in text.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                
                if let Ok(event) = serde_json::from_str::<Event>(line) {
                    match event.event_type.as_str() {
                        "challenge" => {
                            if let Some(challenge) = event.challenge {
                                println!("Challenge received: {}", challenge.id);
                                self.accept_challenge(&challenge.id).await?;
                            }
                        }
                        "gameStart" => {
                            if let Some(game) = event.game {
                                println!("Game started: {}", game.id);
                                if let Err(e) = self.play_game(&game.id).await {
                                    eprintln!("Error playing game {}: {}", game.id, e);
                                }
                            }
                        }
                        _ => {
                            println!("📨 Event: {}", event.event_type);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn accept_challenge(&self, challenge_id: &str) -> Result<()> {
        let url = format!("https://lichess.org/api/challenge/{}/accept", challenge_id);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.lichess_token))
            .send()
            .await?;
        
        if response.status().is_success() {
            println!("Challenge accepted: {}", challenge_id);
        } else {
            println!("Failed to accept challenge: {}", response.text().await?);
        }
        
        Ok(())
    }

    async fn play_game(&mut self, game_id: &str) -> Result<()> {
        let url = format!("https://lichess.org/api/bot/game/stream/{}", game_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.lichess_token))
            .send()
            .await?;

        let mut stream = response.bytes_stream();
        let mut my_color: Option<String> = None;
        let mut initial_fen = STARTPOS_FEN.to_string();
        let mut last_move_count: usize = 0;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk);
            
            for line in text.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                
                // Try GameFull first (initial game state)
                if let Ok(game_full) = serde_json::from_str::<GameFull>(line) {
                    // Fix: Handle startpos/- in initialFen
                    initial_fen = if game_full.initial_fen == "-" || game_full.initial_fen == "startpos" {
                        STARTPOS_FEN.to_string()
                    } else {
                        game_full.initial_fen
                    };
                    
                    // Determine my color
                    if let Some(white) = game_full.white {
                        if white.id == self.bot_id {
                            my_color = Some("white".to_string());
                            println!("🎮 Playing as White");
                        }
                    }
                    if let Some(black) = game_full.black {
                        if black.id == self.bot_id {
                            my_color = Some("black".to_string());
                            println!("🎮 Playing as Black");
                        }
                    }
                    
                    self.handle_state(&game_full.state, &my_color, &initial_fen, game_id, &mut last_move_count).await?;
                    continue;
                }
                
                // Try GameState (position updates)
                if let Ok(state) = serde_json::from_str::<GameState>(line) {
                    // Handle game end states
                    match state.status.as_str() {
                        "started" => {
                            self.handle_state(&state, &my_color, &initial_fen, game_id, &mut last_move_count).await?;
                        }
                        "mate" => {
                            println!("Checkmate!");
                            break;
                        }
                        "resign" => {
                            println!("Resignation");
                            break;
                        }
                        "aborted" => {
                            println!("Game aborted");
                            break;
                        }
                        "timeout" => {
                            println!("Timeout");
                            break;
                        }
                        "draw" => {
                            println!("Draw");
                            break;
                        }
                        status => {
                            println!("Game ended: {}", status);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_state(
        &mut self, 
        state: &GameState, 
        my_color: &Option<String>, 
        fen: &str, 
        game_id: &str,
        last_move_count: &mut usize,
    ) -> Result<()> {
        let move_count = if state.moves.is_empty() { 
            0 
        } else { 
            state.moves.split_whitespace().count() 
        };
        
        // Fix: Prevent duplicate moves
        if move_count <= *last_move_count {
            return Ok(());
        }
        *last_move_count = move_count;
        
        let is_my_turn = match my_color.as_deref() {
            Some("white") => move_count % 2 == 0,
            Some("black") => move_count % 2 == 1,
            _ => {
                println!("Color not determined yet");
                return Ok(());
            }
        };
        
        if is_my_turn {
            println!("My turn (move {}), thinking...", move_count + 1);
            
            // Fix: Use spawn_blocking for blocking UCI call
            let fen = fen.to_string();
            let moves = state.moves.clone();
            let time = self.config.movetime_ms;
            let engine = self.engine.clone();
            
            let best_move = tokio::task::spawn_blocking(move || {
                engine.get_best_move(&fen, &moves, time)
            })
            .await??;
            
            self.make_move(game_id, &best_move).await?;
        } else {
            println!("Opponent's turn (move {})", move_count + 1);
        }
        
        Ok(())
    }

    async fn make_move(&self, game_id: &str, mv: &str) -> Result<()> {
        let url = format!("https://lichess.org/api/bot/game/{}/move/{}", game_id, mv);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.lichess_token))
            .send()
            .await?;

        if response.status().is_success() {
            println!("Played: {}", mv);
        } else {
            let error = response.text().await?;
            println!("Failed to play {}: {}", mv, error);
            return Err(anyhow::anyhow!("Move failed: {}", error));
        }

        Ok(())
    }
}

impl std::fmt::Debug for LichessBot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LichessBot")
            .field("bot_id", &self.bot_id)
            .finish()
    }
}