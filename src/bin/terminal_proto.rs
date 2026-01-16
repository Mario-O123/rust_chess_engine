use std::io::{self, Write};

use rust_chess_engine::board::mailbox120::square120_from_file_rank;
use rust_chess_engine::movegen::{Move, filter_legal_moves, generate_pseudo_legal_moves};
use rust_chess_engine::position::{Cell, Color, PieceKind, Position};

fn main() {
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
    }
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
