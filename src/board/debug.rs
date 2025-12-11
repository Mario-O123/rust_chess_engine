//Helperfunctions to print and check the Board

use crate::position::{Position, Piece, Color, Piecekind};
use crate::Mailbox120::{SQUARE64_TO_SQUARE120, SQUARE120_TO_SQUARE64, OFFBOARD};


//Convert Piece to Char
//White Uppercase, Black Lowercase
#[inline]
pub fn piece_to_char(piece: &Piece) -> char {
    match (&piece.color, &piece.kind) {
        (Color::White, Piecekind::Pawn)   => 'P',
        (Color::White, Piecekind::Knight) => 'N',
        (Color::White, Piecekind::Bishop) => 'B',
        (Color::White, Piecekind::Rook)   => 'R',
        (Color::White, Piecekind::Queen)  => 'Q',
        (Color::White, Piecekind::King)   => 'K',
        (Color::Black, Piecekind::Pawn)   => 'p',
        (Color::Black, Piecekind::Knight) => 'n',
        (Color::Black, Piecekind::Bishop) => 'b',
        (Color::Black, Piecekind::Rook)   => 'r',
        (Color::Black, Piecekind::Queen)  => 'q',
        (Color::Black, Piecekind::King)   => 'k',
    }
}

//UTF-8 figures for fancy Output
#[inline]
pub fn piece_to_char_unicode(piece: &Piece) -> char {
    match (&piece.color, &piece.kind) {
        (Color::White, Piecekind::Pawn)   => '♙',
        (Color::White, Piecekind::Knight) => '♘',
        (Color::White, Piecekind::Bishop) => '♗',
        (Color::White, Piecekind::Rook)   => '♖',
        (Color::White, Piecekind::Queen)  => '♕',
        (Color::White, Piecekind::King)   => '♔',
        (Color::Black, Piecekind::Pawn)   => '♟',
        (Color::Black, Piecekind::Knight) => '♞',
        (Color::Black, Piecekind::Bishop) => '♝',
        (Color::Black, Piecekind::Rook)   => '♜',
        (Color::Black, Piecekind::Queen)  => '♛',
        (Color::Black, Piecekind::King)   => '♚',
    }
}

//extract File and Rank out ouf 120er Index
pub fn file_rank_from_square120(square120: usize) -> (u8, u8) {
    let adjusted = square120 - 21;
    let file = (adjusted % 10) as u8;
    let rank = (adjusted / 10) as u8;
    (file, rank)
}

//Algebraic notation of a 64 Field
#[inline]
pub fn square_to_sring(square64: u8) -> String {
    let square120 = SQUARE64_TO_SQUARE120[square64 as usize];
    let (file, rank) = file_rank_from_square120(square120);
    let file_char = (b'a' + file) as char;
    let rank_char = (b'1' + rank) as char;
    format!("{}{}", file_char, rank_char)
}

//returns board as ASCII (64er)
pub fn print_board(pos: &Position) {
    println!("----------------- BOARD -----------------");
    for rank in (0..8).rev() {
        print!("{} ", rank + 1);
        for file in 0..8 {
            let square64 = rank * 8 + file;
            let square120 = SQUARE64_TO_SQUARE120[square64];
            let c = match &pos.board[square120] {
                Some(piece) => piece_to_char(piece),
                None => '.',
            };
            print!("{} ", c);
        }
        println!();
    }
    println!("   a b c d e f g h");
    println!("side:      {:?}", pos.player_to_move);
    println!("castling:  {}", castling_to_string(pos.castling_rights));
    println!("enpassant: {:?}", pos.en_passant_square.map(square_to_string));
    println!("halfmoves: {}", pos.half_move_clock);
    println!("-----------------------------------------");
}

//returns board as Unicode figures
pub fn print_board_unicode(pos: &Position) {

}

//returns Castling-Rights of Black and White (KQkq-format) 
fn castling_to_string(rights: u8) {

}

//convert board to FEN-String
pub fn board_to_fen(pos: &Position) -> String {

}

//returns Mailbox120 Matrix
pub fn print_mailbox120(pos: &Position) {

} 

//Checks consistency between 120er und 64er
pub fn debug_sanity_check(pos: &Position) {

}

//Helperfunction: Checks if two pieces the same
fn pieces_equal(p1: &Piece, p2: &Piece) -> bool {

}

//OneLiner for Board 
pub fn board_compact(pos: &Position) -> String {

}

//return move information
pub fn print_move_debug(from: u8, to: u8, piece: Option<&Piece>, captured: Option<&Piece>) {

}


