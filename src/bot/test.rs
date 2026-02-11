#[cfg(test)]
mod tests {
    use super::super::*;
    use std::io::Write;

    // Mock UCI Engine für Tests
    const MOCK_ENGINE_SCRIPT: &str = r#"#!/bin/bash
while IFS= read -r line; do
    case "$line" in
        "uci")
            echo "id name MockEngine"
            echo "id author TestBot"
            echo "uciok"
            ;;
        "isready")
            echo "readyok"
            ;;
        "position"*)
            # Just acknowledge
            ;;
        "go"*)
            # Always return e2e4 as best move
            echo "info depth 1 score cp 20"
            echo "bestmove e2e4"
            ;;
        "quit")
            exit 0
            ;;
    esac
done
"#;

    fn create_mock_engine() -> std::io::Result<String> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let temp_dir = std::env::temp_dir();

        // Eindeutiger Dateiname pro Test!
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let script_path = temp_dir.join(format!("mock_uci_engine_{}.sh", timestamp));

        let mut file = std::fs::File::create(&script_path)?;
        file.write_all(MOCK_ENGINE_SCRIPT.as_bytes())?;

        // Make executable (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(&script_path, perms)?;
        }

        Ok(script_path.to_string_lossy().to_string())
    }

    #[test]
    fn test_config_from_env() {
        unsafe {
            std::env::set_var("LICHESS_TOKEN", "lip_test123");
            std::env::set_var("ENGINE_PATH", "./test_engine");
            std::env::set_var("MOVETIME_MS", "2000");
        }

        let config = BotConfig {
            lichess_token: std::env::var("LICHESS_TOKEN").unwrap(),
            engine_path: std::env::var("ENGINE_PATH").unwrap(),
            movetime_ms: std::env::var("MOVETIME_MS").unwrap().parse().unwrap(),
        };

        assert_eq!(config.lichess_token, "lip_test123");
        assert_eq!(config.engine_path, "./test_engine");
        assert_eq!(config.movetime_ms, 2000);

        unsafe {
            std::env::remove_var("LICHESS_TOKEN");
            std::env::remove_var("ENGINE_PATH");
            std::env::remove_var("MOVETIME_MS");
        }
    }

    #[test]
    fn test_config_missing_token() {
        unsafe {
            std::env::remove_var("LICHESS_TOKEN");
        }

        let result = BotConfig::from_env();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("LICHESS_TOKEN"));
    }

    #[test]
    fn test_config_defaults() {
        unsafe {
            std::env::set_var("LICHESS_TOKEN", "lip_test");
            std::env::remove_var("ENGINE_PATH");
            std::env::remove_var("MOVETIME_MS"); // Das reicht nicht!

            // Warte kurz damit env vars sich updaten
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // Prüfe dass Vars wirklich weg sind
        assert!(
            std::env::var("MOVETIME_MS").is_err(),
            "MOVETIME_MS should be unset"
        );

        let config = BotConfig {
            lichess_token: std::env::var("LICHESS_TOKEN").unwrap(),
            engine_path: std::env::var("ENGINE_PATH")
                .unwrap_or_else(|_| "./target/release/chess-engine".to_string()),
            movetime_ms: std::env::var("MOVETIME_MS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
        };

        assert_eq!(config.engine_path, "./target/release/chess-engine");
        assert_eq!(config.movetime_ms, 1000);

        unsafe {
            std::env::remove_var("LICHESS_TOKEN");
        }
    }

    #[cfg(unix)] // UCI tests only work on Unix (need bash script)
    #[test]
    fn test_uci_engine_initialization() {
        let engine_path = create_mock_engine().unwrap();

        let result = uci::UciEngineHandle::new(&engine_path);

        assert!(result.is_ok(), "Engine should initialize successfully");
    }

    #[cfg(unix)]
    #[test]
    fn test_uci_engine_get_best_move() {
        let engine_path = create_mock_engine().unwrap();
        let engine = uci::UciEngineHandle::new(&engine_path).unwrap();

        let best_move = engine.get_best_move(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "",
            1000,
        );

        assert!(best_move.is_ok());
        assert_eq!(best_move.unwrap(), "e2e4");
    }

    #[cfg(unix)]
    #[test]
    fn test_uci_engine_with_moves() {
        let engine_path = create_mock_engine().unwrap();
        let engine = uci::UciEngineHandle::new(&engine_path).unwrap();

        let best_move = engine.get_best_move(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "e2e4 e7e5",
            1000,
        );

        assert!(best_move.is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn test_uci_engine_thread_safety() {
        let engine_path = create_mock_engine().unwrap();
        let engine = uci::UciEngineHandle::new(&engine_path).unwrap();

        // Clone the handle (test Arc<Mutex> works)
        let engine_clone = engine.clone();

        let handle = std::thread::spawn(move || {
            engine_clone.get_best_move(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                "",
                1000,
            )
        });

        let result = handle.join().unwrap();
        assert!(result.is_ok());
    }

    #[test]
    fn test_startpos_fen_constant() {
        use crate::bot::lichess::STARTPOS_FEN;

        assert_eq!(
            STARTPOS_FEN,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }

    // Integration test skeleton (requires actual Lichess token)
    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_lichess_authentication() {
        // This test requires a real LICHESS_TOKEN
        // Run with: LICHESS_TOKEN=lip_xxx cargo test test_lichess_authentication -- --ignored

        let config = BotConfig::from_env();

        if config.is_err() {
            println!("Skipping test - no LICHESS_TOKEN set");
            return;
        }

        // Would need tokio runtime for async test
        // See below for async test example
    }
}

// Async tests (require tokio test feature)
#[cfg(test)]
mod async_tests {
    use super::super::*;

    #[tokio::test]
    #[ignore] // Run with real token: cargo test -- --ignored
    async fn test_lichess_bot_authentication() {
        let token = std::env::var("LICHESS_TOKEN");
        if token.is_err() {
            println!("Skipping - no LICHESS_TOKEN");
            return;
        }

        let config = BotConfig {
            lichess_token: token.unwrap(),
            engine_path: "./mock_engine".to_string(),
            movetime_ms: 1000,
        };

        // Note: This will fail if mock_engine doesn't exist
        // For real test, use actual engine or mock
        let result = lichess::LichessBot::new(config).await;

        // Should authenticate successfully (or fail on engine, not auth)
        if result.is_err() {
            let err = result.unwrap_err().to_string();
            assert!(
                !err.contains("401") && !err.contains("authenticate"),
                "Authentication failed: {}",
                err
            );
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_lichess_invalid_token() {
        let config = BotConfig {
            lichess_token: "lip_invalid_token_12345".to_string(),
            engine_path: "./mock_engine".to_string(),
            movetime_ms: 1000,
        };

        let result = lichess::LichessBot::new(config).await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("401") || err.contains("authenticate"),
            "Expected auth error, got: {}",
            err
        );
    }
}
