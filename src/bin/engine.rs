use std::io::{self, BufRead, Write};

use rust_chess_engine::evaluation::ClassicalEval;
use rust_chess_engine::movegen::{Move, filter_legal_moves, generate_pseudo_legal_moves};
use rust_chess_engine::position::Position;
use rust_chess_engine::search::{SearchLimits, Searcher};

const MOVETIME: u64 = 2000;
const MAXDEPTH: u8 = 12;

fn main() {
    let stdin = io::stdin();
    let mut out = io::stdout();
    let mut pos = Position::starting_position();
    // Here we can also change to neural evaluation
    let mut searcher = Searcher::new(ClassicalEval::new());

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
                    let movetime_ms = parse_movetime_ms(line).unwrap_or(MOVETIME);

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

fn handle_position(line: &str, pos: &mut Position) {
    let mut parts = line.split_whitespace();
    let _ = parts.next();

    match parts.next() {
        Some("startpos") => {
            *pos = Position::starting_position();
        }
        Some("fen") => return,
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

fn parse_movetime_ms(line: &str) -> Option<u64> {
    let mut it = line.split_whitespace();
    while let Some(tok) = it.next() {
        if tok == "movetime" {
            return it.next()?.parse::<u64>().ok();
        }
    }
    None
}

fn legal_moves(pos: &Position) -> Vec<Move> {
    let pseudo = generate_pseudo_legal_moves(pos);
    filter_legal_moves(pos, &pseudo)
}

fn find_legal_move_from_uci(input: &str, legal: &[Move]) -> Option<Move> {
    let key = Move::from_uci(input)?;
    legal.iter().copied().find(|m| {
        m.from == key.from && m.to == key.to && m.promotion_piece() == key.promotion_piece()
    })
}
