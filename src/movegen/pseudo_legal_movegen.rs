// const sliding: [bool;5] = [false, true, true, true, false]; //knight bishop rook queen king 

use crate::position::{Position, Cell, PieceKind};
use crate::board::mailbox120::{
    BISHOP_DIRECTIONS, BOARD_SIZE, KING_DIRECTIONS, KNIGHT_DIRECTIONS, QUEEN_DIRECTIONS, ROOK_DIRECTIONS, is_on_board
};
use crate::movegen::{Move, piece, pawn};


pub fn generate_pseudo_legal_moves(position : &Position) -> Vec<Move> {
    let mut move_list = Vec::new();

    for square120 in 0..BOARD_SIZE {
        if !is_on_board(square120) {
            continue;
        } 

        let Cell::Piece(piece) = position.board[square120] else {continue;};
        if piece.color != position.player_to_move {continue;} 
    
        match piece.kind {
            PieceKind::Knight => { 
                piece::gen_jumping_moves(position, &mut move_list, square120, &KNIGHT_DIRECTIONS);
            }PieceKind::Bishop => {
                piece::gen_sliding_moves(position, &mut move_list, square120, &BISHOP_DIRECTIONS);
            }PieceKind::Pawn => {
                pawn::gen_pawn_moves(position, &mut move_list, square120);
            }PieceKind::Rook => {
                piece::gen_sliding_moves(position, &mut move_list, square120, &ROOK_DIRECTIONS);
            }PieceKind::Queen => {
                piece::gen_sliding_moves(position, &mut move_list, square120, &QUEEN_DIRECTIONS);
            }PieceKind::King => {
                piece::gen_jumping_moves(position, &mut move_list, square120, &KING_DIRECTIONS);
                piece::gen_castling_moves(position, &mut move_list, square120);
            }
        }
    }
    move_list
}


