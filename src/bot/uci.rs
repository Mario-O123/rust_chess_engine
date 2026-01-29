use std::process::{Command, Stdio, Child, ChildStdin, ChildStdout};
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use anyhow::Result;

/// Internal UCI engine that handles the actual communication
struct UciEngine {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl UciEngine {
    fn new(engine_path: &str) -> Result<Self> {
        let mut process = Command::new(engine_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdin = process.stdin.take().unwrap();
        let stdout = BufReader::new(process.stdout.take().unwrap());

        let mut engine = Self { process, stdin, stdout };
        
        // Initialize UCI
        engine.send("uci")?;
        engine.wait_for("uciok")?;
        engine.send("isready")?;
        engine.wait_for("readyok")?;
        
        println!("UCI Engine initialized");
        
        Ok(engine)
    }

    fn send(&mut self, cmd: &str) -> Result<()> {
        writeln!(self.stdin, "{}", cmd)?;
        self.stdin.flush()?;
        println!("→ UCI: {}", cmd);
        Ok(())
    }

    fn wait_for(&mut self, expected: &str) -> Result<String> {
        let mut line = String::new();
        loop {
            line.clear();
            self.stdout.read_line(&mut line)?;
            let trimmed = line.trim();
            println!("← UCI: {}", trimmed);
            if trimmed.contains(expected) {
                return Ok(trimmed.to_string());
            }
        }
    }

    fn get_best_move(&mut self, fen: &str, moves: &str, time_ms: u64) -> Result<String> {
        // Set position
        if moves.is_empty() {
            self.send(&format!("position fen {}", fen))?;
        } else {
            self.send(&format!("position fen {} moves {}", fen, moves))?;
        }

        // Search
        self.send(&format!("go movetime {}", time_ms))?;

        // Get result
        let response = self.wait_for("bestmove")?;
        
        // Parse "bestmove e2e4" -> "e2e4"
        let parts: Vec<&str> = response.split_whitespace().collect();
        if parts.len() >= 2 {
            Ok(parts[1].to_string())
        } else {
            Err(anyhow::anyhow!("Invalid bestmove response"))
        }
    }
}

impl Drop for UciEngine {
    fn drop(&mut self) {
        let _ = self.send("quit");
        let _ = self.process.wait();
    }
}

/// Thread-safe handle to UCI engine
/// Can be cloned and used across async tasks
#[derive(Clone)]
pub struct UciEngineHandle {
    inner: Arc<Mutex<UciEngine>>,
}

impl UciEngineHandle {
    pub fn new(engine_path: &str) -> Result<Self> {
        let engine = UciEngine::new(engine_path)?;
        Ok(Self { 
            inner: Arc::new(Mutex::new(engine)) 
        })
    }
    
    /// Get best move for position (blocking call - use with spawn_blocking!)
    pub fn get_best_move(&self, fen: &str, moves: &str, time_ms: u64) -> Result<String> {
        let mut engine = self.inner.lock()
            .map_err(|e| anyhow::anyhow!("Engine lock poisoned: {}", e))?;
        engine.get_best_move(fen, moves, time_ms)
    }
}

impl std::fmt::Debug for UciEngineHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UciEngineHandle")
            .finish_non_exhaustive()
    }
}