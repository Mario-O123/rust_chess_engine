//! Minimal UCI driver for `rust_chess_engine`.
//!
//! This binary speaks a small subset of the UCI protocol via stdin/stdout and
//! connects it to our `Searcher` implementation.
//!
//! Evaluation:
//! - Default: classical evaluation
//!     (build example: `cargo run --release --bin bot`).
//! - With feature neural-eval: loads a neural model and uses NeuralEval
//!     (build example: `cargo run --release --features neural-eval --bin bot`).

use std::io::{self, BufRead, Write};

use rust_chess_engine::evaluation::ClassicalEval;
use rust_chess_engine::movegen::{Move, filter_legal_moves, generate_pseudo_legal_moves};
use rust_chess_engine::position::Position;
use rust_chess_engine::search::{SearchLimits, Searcher};

/// Default thinking time (ms) used if `go movetime <ms>` is missing.
const DEFAULT_MOVETIME: u64 = 2000;

/// Depth limit for the search, regardless of given time.
const MAXDEPTH: u8 = 12;

fn main() {
    let stdin = io::stdin();
    let mut out = io::stdout();
    let mut pos = Position::starting_position();

    // Starts on default with classical evaluation
    // To use neural evaluation use "cargo run --release --features neural-eval --bin bot"
    let mut searcher = make_searcher();

    // UCI requires immediate flushing; many GUIs expect line-buffered responses.
    let mut send = |s: &str| {
        writeln!(out, "{s}").unwrap();
        out.flush().unwrap();
    };

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match line {
            "uci" => {
                send("id name RustEngine 1.0");
                send("id author Mario Orsolic, Emil Sitka, Julien Kriebel, Noah Schuller");
                send("uciok");
            }
            "isready" => send("readyok"),
            "ucinewgame" => pos = Position::starting_position(),
            "quit" => break,
            _ => {
                if line.starts_with("position ") {
                    handle_position(line, &mut pos);
                } else if line.starts_with("go") {
                    let movetime_ms = parse_movetime_ms(line).unwrap_or(DEFAULT_MOVETIME);

                    let limits = SearchLimits {
                        max_depth: MAXDEPTH,
                        max_nodes: None,
                        max_time_ms: Some(movetime_ms),
                    };

                    let result = searcher.search(&mut pos, limits);

                    if result.best_move.is_null() {
                        send("bestmove 0000");
                    } else {
                        send(&format!("bestmove {}", result.best_move.to_uci()));
                    }
                }
            }
        }
    }
}

/// Parses a UCI `position ...` command and updates `pos`.
///
/// Supported formats:
/// - `position startpos [moves ...]`
/// - `position fen <6 fen fields> [moves ...]`
///
/// After the base position is set, any following moves are applied in order.
/// If an invalid FEN or an illegal move is encountered, this function returns
/// early and keeps the partially updated state (current behavior).
fn handle_position(line: &str, pos: &mut Position) {
    let mut parts = line.split_whitespace();
    let _ = parts.next();

    match parts.next() {
        Some("startpos") => {
            *pos = Position::starting_position();
        }
        Some("fen") => {
            let fen_fields: Vec<&str> = parts.by_ref().take(6).collect();
            if fen_fields.len() != 6 {
                return;
            }
            let fen = fen_fields.join(" ");

            let Ok(parsed) = Position::from_fen(&fen) else {
                return;
            };

            *pos = parsed;
        }
        _ => return,
    }

    let mut saw_moves = false;
    for tok in parts {
        if tok == "moves" {
            saw_moves = true;
            continue;
        }
        if !saw_moves {
            continue;
        }

        let legal = legal_moves(pos);
        let Some(mv) = find_legal_move_from_uci(tok, &legal) else {
            return;
        };
        pos.make_move(mv);
    }
}

/// Extracts `movetime <ms>` from a UCI `go ...` command.
fn parse_movetime_ms(line: &str) -> Option<u64> {
    let mut it = line.split_whitespace();
    while let Some(tok) = it.next() {
        if tok == "movetime" {
            return it.next()?.parse::<u64>().ok();
        }
    }
    None
}

/// Creates a `Searcher` using the neural evaluation (feature-gated).
#[cfg(feature = "neural-eval")]
fn make_searcher() -> Searcher<NeuralEval> {
    let model_path = "src/trainer_rust/models/mlp_checkpoint_3.json";
    let eval = NeuralEval::load(model_path).expect("failed to load model");
    Searcher::new(eval)
}

/// Creates a `Searcher` using the classical evaluation (default).
#[cfg(not(feature = "neural-eval"))]
fn make_searcher() -> Searcher<ClassicalEval> {
    Searcher::new(ClassicalEval::new())
}

/// Generates all legal moves for the current position.
fn legal_moves(pos: &Position) -> Vec<Move> {
    let pseudo = generate_pseudo_legal_moves(pos);
    filter_legal_moves(pos, &pseudo)
}

/// Finds the matching legal move for a given UCI move string (e.g. `e2e4`, `e7e8q`).
///
/// We parse the UCI move into a key move and then match by from/to and promotion.
/// This ignores any additional internal move flags and ensures the move is legal
fn find_legal_move_from_uci(input: &str, legal: &[Move]) -> Option<Move> {
    let key = Move::from_uci(input)?;
    legal.iter().copied().find(|m| {
        m.from == key.from && m.to == key.to && m.promotion_piece() == key.promotion_piece()
    })
}
