use std::io::{self, Write};
use std::time::Instant;

use rust_chess_engine::board::mailbox120::{QUEEN_DIRECTIONS, square120_from_file_rank};
use rust_chess_engine::movegen::{Move, generate_legal_moves_in_place};
use rust_chess_engine::position::{Cell, Color, PieceKind, Position, Game, GameStatus};
use rust_chess_engine::evaluation::{Evaluator, ClassicalEval};
use rust_chess_engine::search::{SearchLimits, Searcher};

fn main() {
    let  mut cli = EngineCli::new();

    println!("terminal_promo — commands: help | eval | go [depth N|time MS|nodes N] | undo | undo2 | new | engine on/off | quit");
    loop {
        cli.print_position();

        if let Some(msg) = cli.game_over_message() {
            println!("{msg}");
        }

        print!("{:?}> ", cli.game.position().player_to_move);
        if io::stdout().flush().is_err() {
            eprintln!("stdput flush failed");
            break;
        }

        let mut line = String::new();
        let bytes = match io::stdin().read_line(&mut line) {
            Ok(n) => n,
            Err(_) => break,
        };
        if bytes == 0 {
            break;
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if cli.handle_line(input) {
            break;
        }
    }

    
    
    /* 
    let mut pos = Position::starting_position();

    loop {
        print_board(&pos);
        println!("FEN: {}", pos.to_fen());

        print!("{:?}> ", pos.player_to_move);
        if io::stdout().flush().is_err() {
            eprintln!("stdout flush failed");
            break;
        }

        let mut line = String::new();
        let bytes = match io::stdin().read_line(&mut line) {
            Ok(n) => n,
            Err(_) => break,
        };
        if bytes == 0 {
            break;
        }
        let input = line.trim();

        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            break;
        }

        //legal moves
        let legal = legal_moves(&pos);
        if legal.is_empty() {
            println!("Game over (keine legalen Züge).");
            break;
        }

        //User move: UCI -> matching legal move
        let user_mv = match find_legal_move_from_uci(input, &legal) {
            Some(mv) => mv,
            None => {
                println!("Illegal: {input}");
                continue;
            }
        };

        pos.make_move(user_mv);

        //Engine move: simply first legal move
        let legal2 = legal_moves(&pos);
        if legal2.is_empty() {
            print_board(&pos);
            println!("You have won, game over for the oponent (no legal moves for oponent).");
            break;
        }

        let engine_mv = legal2[0];
        println!("Engine: {}", engine_mv.to_uci());
        pos.make_move(engine_mv);
    } */
}

pub struct EngineCli {
    game: Game,
    searcher: Searcher<ClassicalEval>,
    eval_view: ClassicalEval,
    legal_buf: Vec<Move>,
    engine_enabled: bool,
    default_limits: SearchLimits,
}

impl EngineCli {
    pub fn new() -> Self {
        let eval_for_search = ClassicalEval::new();
        Self {
            game: Game::new(),
            searcher: Searcher::new(eval_for_search),
            eval_view: ClassicalEval::new(),
            legal_buf: Vec::new(),
            engine_enabled: true,
            default_limits: SearchLimits {
                max_depth: 7,
                max_nodes: None,
                max_time_ms: Some(2000),
            },
        }
    }

    pub fn print_position(&self) {
        let pos = self.game.position();
        print_board(pos);

        println!();

        println!("FEN: {}", pos.to_fen());
        println!("Status: {}", format_status(self.game.status()));
    }

    pub fn game_over_message(&self) -> Option<String> {
        match self.game.status() {
            GameStatus::Ongoing => None,
            other => Some(format!("Game over: {}", format_status(other))),
        }
    }

    fn play_engine_move(&mut self, limits: SearchLimits) {
        let root_side_to_move = self.game.position().player_to_move;

        let requested_depth = limits.max_depth;
        let requested_time_ms = limits.max_time_ms;
        let requested_nodes = limits.max_nodes;

        let t0 = Instant::now();
        let result = {
            let (searcher, game) = (&mut self.searcher, &mut self.game);
            searcher.search(game.position_mut(), limits)
        };

        let score_side_to_move_cp = result.score_cp;
        let score_white_cp = if root_side_to_move == Color::White {
            score_side_to_move_cp
        } else {
            -score_side_to_move_cp
        };

        let elapsed_ms = t0.elapsed().as_millis() as u64;
        let reached_depth = result.depth;
        let stopped_by = if reached_depth >= requested_depth {
            "depth"
        } else if let Some(ms) = requested_time_ms {
            if elapsed_ms >= ms {"time"} else {"unknown"}
        } else if let Some(n) = requested_nodes {
            if result.nodes >= n {"nodes"} else {"unknown"}
        } else {
            "unknown"
        };

        if result.best_move.is_null() {
            println!("Engine({:?}) found no move: score(side_to_move)={}cp | score(white)={}cp | depth={}/{} | nodes={} | elapsed={}ms | stop={}",
            root_side_to_move, score_side_to_move_cp, score_white_cp, reached_depth, requested_depth, result.nodes, elapsed_ms, stopped_by);
            return;
        }

        println!(
            "Engine({:?}): bestmove {} | score(stm)={}cp | score(white)={}cp | depth={}/{} | nodes={} | elapsed={}ms | stop={} | limits(time={:?}ms, nodes={:?})",
            root_side_to_move,
            result.best_move.to_uci(),
            score_side_to_move_cp,
            score_white_cp,
            reached_depth,
            requested_depth,
            result.nodes,
            elapsed_ms,
            stopped_by,
            requested_time_ms,
            requested_nodes,
        );

        self.game.try_play_move(result.best_move);
    }

    pub fn handle_line(&mut self, input: &str) -> bool {
        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts[0].to_ascii_lowercase();

        match  cmd.as_str() {
            "quit" | "exit" => return true,

            "help" => {
                println!();
                println!("Commands:");
                println!("  help");
                println!("  quit/exit");
                println!("  new                            (new game)");
                println!("  undo                           (undo 1 ply");
                println!("  undo2                          (undo 2 plies");
                println!("  eval                           (classical eval, from White perspective");
                println!("  go [depth N| time MS|noes N]   (engine plays one move noew)");
                println!("  engine on/off                  (toggle auto-engine reply after your move");
                return false;
            }

            "new" => {
                self.game = Game::new();
                return false;
            }

            "undo" => {
                if !self.game.undo() {
                    println!("Nothing to undo");
                }
                return false;
            }

            "undo2" => {
                let _ = self.game.undo();
                let _ = self.game.undo();
                return false;
            }

            "engine" => {
                if parts.len() >= 2 {
                    match parts[1].to_ascii_lowercase().as_str() {
                        "on" => self.engine_enabled = true,
                        "off" => self.engine_enabled = false,
                        _ => println!("usage: engine on|off"),

                    }
                } else {
                    println!("engine is {}", if self.engine_enabled {" on "} else { "off" });
                }
                return false;
            }

            "eval" => {
                let score = self.eval_view.evaluate(self.game.position());
                println!("Eval (White+) {} cp", score);
                return false;
            }

            "go" => {
                if self.game.status() != GameStatus::Ongoing {
                    println!("Game is over; use 'new' or 'undo'.");
                    return false;
                }
                let limits = parse_go_limits(&parts[1..], self.default_limits);
                self.play_engine_move(limits);
                return false;
            }

            _ => {}
        }

        //default: interpret as uci move
        if self.game.status() != GameStatus::Ongoing {
            println!("Game is over; use 'new' or 'undo'.");
            return false;
        }

        //generate legal moves
        let pos = self.game.position_mut();
        generate_legal_moves_in_place(pos, &mut self.legal_buf);

        if self.legal_buf.is_empty() {
            println!("No legal moves.");
            return false;
        }

        //uci -> legal move
        let user_mv = match find_legal_move_from_uci(input, &self.legal_buf) {
            Some(mv) => mv,
            None => {
                println!("Illegal: {input}");
                return false;
            }
        };

        self.game.try_play_move(user_mv);

        //when engine active: search answer-move and play
        if self.engine_enabled && self.game.status() == GameStatus::Ongoing {
            self.play_engine_move(self.default_limits);
        }

        false
    }
 


}

fn format_status(status: GameStatus) -> String {
        match status {
            GameStatus::Ongoing => "Ongoing".to_string(),
            GameStatus::Checkmate {winner} => format!("Chechmate (winner: {:?})", winner),
            GameStatus::Stalemate => "Stalemate".to_string(),
            GameStatus::DrawRepetition => "Draw by repetition".to_string(),
            GameStatus::DrawInsufficientMaterial => "Draw (insufficient material".to_string(),
            GameStatus::Draw50Moves => "Draw (50-move rule)".to_string(),
        }
    }

    fn parse_go_limits(go_tokens: &[&str], default_limits: SearchLimits) -> SearchLimits {
        let mut effective_limits = default_limits;
        let mut token_index = 0;

        while token_index < go_tokens.len() {
            let keyword = go_tokens[token_index].to_ascii_lowercase();

            match keyword.as_str() {
                "depth" if token_index + 1 < go_tokens.len() => {
                    let depth_token = go_tokens[token_index + 1];
                    if let Ok(parsed_depth) = depth_token.parse::<u8>() {
                        effective_limits.max_depth = parsed_depth.max(1);
                    }
                    token_index += 2;
                }
                "time" if token_index + 1 < go_tokens.len() => {
                    let time_as_token = go_tokens[token_index + 1];
                    if let Ok(parsed_time_ms) = time_as_token.parse::<u64>() {
                        effective_limits.max_time_ms = Some(parsed_time_ms);
                    }
                    token_index += 2;
                }
                "nodes" if token_index + 1 < go_tokens.len() => {
                    let nodes_token = go_tokens[token_index + 1];
                    if let Ok(parsed_nodes) = nodes_token.parse::<u64>() {
                        effective_limits.max_nodes = Some(parsed_nodes);
                    }
                    token_index += 2;
                }
                //unknown token:
                _ => {
                    token_index += 1;
                }
            }
        }
        effective_limits
    }

fn print_board(pos: &Position) {
    println!();
    for rank in (0..8).rev() {
        print!("{} ", rank + 1);
        for file in 0..8 {
            let sq = square120_from_file_rank(file, rank);
            let ch = match pos.board[sq] {
                Cell::Empty => '.',
                Cell::Offboard => '?',
                Cell::Piece(p) => piece_to_char(p.color, p.kind),
            };
            print!(" {}", ch);
        }
        println!();
    }
    println!("\n   a b c d e f g h");
}

fn piece_to_char(color: Color, kind: PieceKind) -> char {
    match (color, kind) {
        (Color::White, PieceKind::Pawn) => '♟',
        (Color::White, PieceKind::Knight) => '♞',
        (Color::White, PieceKind::Bishop) => '♝',
        (Color::White, PieceKind::Rook) => '♜',
        (Color::White, PieceKind::Queen) => '♛',
        (Color::White, PieceKind::King) => '♚',

        (Color::Black, PieceKind::Pawn) => '♙',
        (Color::Black, PieceKind::Knight) => '♘',
        (Color::Black, PieceKind::Bishop) => '♗',
        (Color::Black, PieceKind::Rook) => '♖',
        (Color::Black, PieceKind::Queen) => '♕',
        (Color::Black, PieceKind::King) => '♔',
    }
}


fn find_legal_move_from_uci(input: &str, legal: &[Move]) -> Option<Move> {
    let key = Move::from_uci(input)?;
    legal.iter().copied().find(|m| {
        m.from == key.from && m.to == key.to && m.promotion_piece() == key.promotion_piece()
    })
}

#[cfg(test)]
mod terminal_promo_cli_tests {
    use super::parse_go_limits;
    use rust_chess_engine::search::SearchLimits;

    fn default_limits() -> SearchLimits {
        SearchLimits {
            max_depth: 5,
            max_nodes: None,
            max_time_ms: None,
        }
    }
    #[test]
    fn empty_go_tokens_returns_defaults() {
        let limits = parse_go_limits(&[], default_limits());
        assert_eq!(limits.max_depth, 7);
        assert_eq!(limits.max_time_ms, Some(2000));
        assert_eq!(limits.max_nodes, None);
    }

    #[test]
    fn depth_overrides_default() {
        let limits = parse_go_limits(&["depth", "7"], default_limits());
        assert_eq!(limits.max_depth, 7);
        assert_eq!(limits.max_time_ms, Some(2000));
        assert_eq!(limits.max_nodes, None);
    }

    #[test]
    fn depth_zero_is_raised_to_one() {
        let limits = parse_go_limits(&["depth", "0"], default_limits());
        assert_eq!(limits.max_depth, 1);
    }

    #[test]
    fn time_and_nodes_override_defaults() {
        let limits = parse_go_limits(&["time", "1000", "nodes", "200000"], default_limits());
        assert_eq!(limits.max_depth, 5);
        assert_eq!(limits.max_time_ms, Some(2000));
        assert_eq!(limits.max_nodes, Some(200000));
    }

    #[test]
    fn unknown_tokens_and_invalid_values_are_ignored() {
        let limits = parse_go_limits(
            &["random", "123", "depth", "abc", "time", "xyz", "nodes", "-1"],
            default_limits(),
        );
        
        assert_eq!(limits.max_depth, 7);
        assert_eq!(limits.max_time_ms, Some(2000));
        assert_eq!(limits.max_nodes, None);
    }
}

#[cfg(test)]
mod terminal_promo_handle_line_tests {
    use super::EngineCli;

    #[test]
    fn new_resets_the_game() {
        let mut cli = EngineCli::new();

        cli.handle_line("engine off");
        cli.handle_line("e2e4");

        let fen_after_move = cli.game.position().to_fen();
        assert_ne!(fen_after_move, EngineCli::new().game.position().to_fen());

        cli.handle_line("new");
        let fen_after_new = cli.game.position().to_fen();

        let fen_start = EngineCli::new().game.position().to_fen();
        assert_eq!(fen_after_new, fen_start);
    }

    #[test]
    fn engine_toggle_off_on_does_not_quit() {
        let mut cli = EngineCli::new();
        assert!(!cli.handle_line("engine off"));
        assert!(!cli.handle_line("engine on"));
    }

    #[test]
    fn legal_user_move_changes_position_when_engine_off() {
        let mut cli = EngineCli::new();
        cli.handle_line("engine off");

        let fen_before = cli.game.position().to_fen();
        let side_before = cli.game.position().player_to_move;

        assert!(!cli.handle_line("e2e4"));

        let fen_after = cli.game.position().to_fen();
        let side_after = cli.game.position().player_to_move;

        assert_ne!(fen_after, fen_before);
        assert_ne!(side_after, side_before);
    }

    #[test]
    fn illegal_user_move_does_not_change_position() {
        let mut cli = EngineCli::new();
        cli.handle_line("engine off");

        let fen_before = cli.game.position().to_fen();
        let side_before = cli.game.position().player_to_move;

        assert!(!cli.handle_line("e2e1"));

        let fen_after = cli.game.position().to_fen();
        let side_after = cli.game.position().player_to_move;

        assert_eq!(fen_after, fen_before);
        assert_eq!(side_after, side_before);
    }
}





