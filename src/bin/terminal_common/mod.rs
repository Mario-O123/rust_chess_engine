//! Shared terminal-REPL for terminal_proto_classical and terminal_proto_neural
//! both binaries use the same UI, we use:
//!
//! -identical inüut syntax (UCI moves, commands)
//! -identical output format (FEN, search info, scores)
//! -identical default search budget (depth/time/nodes)
//!
//! input format: UCI moves (eg. e2e4), promotion (eg g7g8q)
//! commands:
//! -help
//! -new
//! -undo/undo2
//! -engine on/off (toggle outomatic engine reply after your move)
//! -eval (static evaluation of the current position)
//! -go [depth N| time MS| nodes N] (search noew and play one engine move)
//! -quit/exit
//!
//! use of "go ..." command: primarily to help with debugging as comparison tool
//! it lets us control the search budget deterministically:
//! -go depth N: comparable depth-limited runs
//! -go nodes N: comparable work budget as max limit of nodes to be visited in search
//! -go time MS: time budget for how long the search is allowed to take
//! by setting and playing with the different combinations for these parameters in search,
//! we can adjust the limits of our engine and compare classical and neural evaluators to see
//! if they perform differently with the same settings.
//!
//! the engine prints "go :"
//! -bestmove
//! -score in centipawns as "score(stm)" (side to move) and score(White) (White perspective as default)
//! -reached depth, nodes, elapsed ms, stop reason
//!
//!

use std::io::{self, Write};
use std::time::Instant;

use rust_chess_engine::board::mailbox120::{QUEEN_DIRECTIONS, square120_from_file_rank};
use rust_chess_engine::evaluation::Evaluator;
use rust_chess_engine::movegen::{Move, generate_legal_moves_in_place};
use rust_chess_engine::position::{Cell, Color, Game, GameStatus, PieceKind, Position};
use rust_chess_engine::search::{SearchLimits, Searcher};

pub(crate) fn format_status(status: GameStatus) -> String {
    match status {
        GameStatus::Ongoing => "Ongoing".to_string(),
        GameStatus::Checkmate { winner } => format!("Chechmate (winner: {:?})", winner),
        GameStatus::Stalemate => "Stalemate".to_string(),
        GameStatus::DrawRepetition => "Draw by repetition".to_string(),
        GameStatus::DrawInsufficientMaterial => "Draw (insufficient material".to_string(),
        GameStatus::Draw50Moves => "Draw (50-move rule)".to_string(),
    }
}

///parse tokens after "go ..." and selectively override the defaults
///unknown tokens or invalid numeric values are ignored
pub(crate) fn parse_go_limits(go_tokens: &[&str], default_limits: SearchLimits) -> SearchLimits {
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

pub(crate) fn print_board(pos: &Position) {
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

///matches a unicode chess piece to the (Color, PieceKind), the chess symbols were copied from the internet
pub(crate) fn piece_to_char(color: Color, kind: PieceKind) -> char {
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

///finds a legal move by matching a UCI move string or promotion
pub(crate) fn find_legal_move_from_uci(input: &str, legal: &[Move]) -> Option<Move> {
    let key = Move::from_uci(input)?;
    legal.iter().copied().find(|m| {
        m.from == key.from && m.to == key.to && m.promotion_piece() == key.promotion_piece()
    })
}

///CLI state: holds the Game, Searcher and UI settings
///"E" is a generic [`Evaluator`] so the same REPL can run with classical or neural evaluation
pub(crate) struct EngineCli<E: Evaluator> {
    pub(crate) game: Game,
    pub(crate) searcher: Searcher<E>,
    pub(crate) eval_view: E,
    pub(crate) legal_buf: Vec<Move>,
    pub(crate) engine_enabled: bool,
    pub(crate) default_limits: SearchLimits,
}

impl<E: Evaluator> EngineCli<E> {
    ///create a new REPL session state with a fresh game and default search limits
    pub(crate) fn new(eval_for_search: E, eval_view: E) -> Self {
        Self {
            game: Game::new(),
            searcher: Searcher::new(eval_for_search),
            eval_view,
            legal_buf: Vec::new(),
            engine_enabled: true,
            default_limits: SearchLimits {
                max_depth: 7,
                max_nodes: None,
                max_time_ms: Some(2000),
            },
        }
    }

    pub(crate) fn print_position(&self) {
        let pos = self.game.position();
        print_board(pos);

        println!();

        println!("FEN: {}", pos.to_fen());
        println!("Status: {}", format_status(self.game.status()));
    }

    pub(crate) fn game_over_message(&self) -> Option<String> {
        match self.game.status() {
            GameStatus::Ongoing => None,
            other => Some(format!("Game over: {}", format_status(other))),
        }
    }

    ///run a search and play the resulting best move
    ///the output (for debug purposes) will include:
    /// -score from side to move and white perspective
    /// -nodes, time, reached depth, stop reason
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
            if elapsed_ms >= ms { "time" } else { "unknown" }
        } else if let Some(n) = requested_nodes {
            if result.nodes >= n {
                "nodes"
            } else {
                "unknown"
            }
        } else {
            "unknown"
        };

        if result.best_move.is_null() {
            println!(
                "Engine({:?}) found no move: score(side_to_move)={}cp | score(white)={}cp | depth={}/{} | nodes={} | elapsed={}ms | stop={}",
                root_side_to_move,
                score_side_to_move_cp,
                score_white_cp,
                reached_depth,
                requested_depth,
                result.nodes,
                elapsed_ms,
                stopped_by
            );
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

    ///handle one input line
    ///returns:
    /// -true-> exit the REPL (quit/exit)
    /// -false-> keep running
    pub(crate) fn handle_line(&mut self, input: &str) -> bool {
        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts[0].to_ascii_lowercase();

        match cmd.as_str() {
            "quit" | "exit" => return true,

            "help" => {
                println!();
                println!("Commands:");
                println!("  help");
                println!("  quit/exit");
                println!("  new                            (new game)");
                println!("  undo                           (undo 1 ply)");
                println!("  undo2                          (undo 2 plies)");
                println!(
                    "  eval                           (classical eval, from White perspective)"
                );
                println!("  go [depth N| time MS| nodes N] (engine plays one move now)");
                println!(
                    "  engine on/off                  (toggle auto-engine reply after your move)"
                );
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
                    println!(
                        "engine is {}",
                        if self.engine_enabled { " on " } else { "off" }
                    );
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

///start the REPL, called by both binaries
pub(crate) fn run_repl<E: Evaluator>(eval_for_search: E, eval_view: E) {
    let mut cli = EngineCli::new(eval_for_search, eval_view);

    println!();
    println!(
        "terminal_proto — commands: help | eval | go [depth N|time MS|nodes N] | undo | undo2 | new | engine on/off | quit"
    );
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
}
