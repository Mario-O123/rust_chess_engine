use std::io::{self, Write};

use rust_chess_engine::board::mailbox120::{QUEEN_DIRECTIONS, square120_from_file_rank};
use rust_chess_engine::movegen::{Move, generate_legal_moves_in_place};
use rust_chess_engine::position::{Cell, Color, PieceKind, Position, Game, GameStatus};
use rust_chess_engine::evaluation::{Evaluator, ClassicalEval};
use rust_chess_engine::search::{SearchLimits, Searcher};

fn main() { /* 

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
                max_depth: 5,
                max_nodes: None,
                max_time_ms: Some(250),
            },
        }
    }

    pub fn print_position(&self) {
        let pos = self.game.position();
        print_board(pos);
        println!("FEN: {}", pos.to_fen());
        println!("Status: {}", format_status(self.game.status()));
    }

    pub fn game_over_message(&self) -> Option<String> {
        match self.game.status() {
            GameStatus::Ongoing => None,
            other => Some(format!("Game over: {}", format_status(other))),
        }
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
        (Color::White, PieceKind::Pawn) => 'P',
        (Color::White, PieceKind::Knight) => 'N',
        (Color::White, PieceKind::Bishop) => 'B',
        (Color::White, PieceKind::Rook) => 'R',
        (Color::White, PieceKind::Queen) => 'Q',
        (Color::White, PieceKind::King) => 'K',

        (Color::Black, PieceKind::Pawn) => 'p',
        (Color::Black, PieceKind::Knight) => 'n',
        (Color::Black, PieceKind::Bishop) => 'b',
        (Color::Black, PieceKind::Rook) => 'r',
        (Color::Black, PieceKind::Queen) => 'q',
        (Color::Black, PieceKind::King) => 'k',
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
            max_time_ms: Some(250),
        }
    }
    #[test]
    fn empty_go_tokens_returns_defaults() {
        let limits = parse_go_limits(&[], default_limits());
        assert_eq!(limits.max_depth, 5);
        assert_eq!(limits.max_time_ms, Some(250));
        assert_eq!(limits.max_nodes, None);
    }

    #[test]
    fn depth_overrides_default() {
        let limits = parse_go_limits(&["depth", "7"], default_limits());
        assert_eq!(limits.max_depth, 7);
        assert_eq!(limits.max_time_ms, Some(250));
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
        assert_eq!(limits.max_time_ms, Some(1000));
        assert_eq!(limits.max_nodes, Some(200000));
    }

    #[test]
    fn unknown_tokens_and_invalid_values_are_ignored() {
        let limits = parse_go_limits(
            &["random", "123", "depth", "abc", "time", "xyz", "nodes", "-1"],
            default_limits(),
        );
        
        assert_eq!(limits.max_depth, 5);
        assert_eq!(limits.max_time_ms, Some(250));
        assert_eq!(limits.max_nodes, None);
    }
}






