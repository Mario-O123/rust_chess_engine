// const sliding: [bool;5] = [false, true, true, true, false]; //knight bishop rook queen king

use crate::board::mailbox120::{
    BISHOP_DIRECTIONS, BOARD_SIZE, KNIGHT_DIRECTIONS, QUEEN_DIRECTIONS as KING_QUEEN_DIRECTIONS,
    ROOK_DIRECTIONS, is_on_board,
};
use crate::movegen::Move;
use crate::movegen::{pawn, piece};
use crate::position::position::PieceKind;
use crate::position::{Cell, Position};

pub fn generate_pseudo_legal_moves(position: &Position) -> Vec<Move> {
    let mut move_list = Vec::new();
    generate_pseudo_legal_moves_in_place(position, &mut move_list);
    move_list
}

pub fn generate_pseudo_legal_moves_in_place(position: &Position, move_list: &mut Vec<Move>) {
    move_list.clear();

    for square120 in 0..BOARD_SIZE {
        if !is_on_board(square120) {
            continue;
        }

        let Cell::Piece(piece) = position.board[square120] else {
            continue;
        };
        if piece.color != position.player_to_move {
            continue;
        }

        match piece.kind {
            PieceKind::Knight => {
                piece::gen_jumping_moves(position, move_list, square120, &KNIGHT_DIRECTIONS);
            }
            PieceKind::Bishop => {
                piece::gen_sliding_moves(position, move_list, square120, &BISHOP_DIRECTIONS);
            }
            PieceKind::Pawn => {
                pawn::gen_pawn_moves(position, move_list, square120);
            }
            PieceKind::Rook => {
                piece::gen_sliding_moves(position, move_list, square120, &ROOK_DIRECTIONS);
            }
            PieceKind::Queen => {
                piece::gen_sliding_moves(position, move_list, square120, &KING_QUEEN_DIRECTIONS);
            }
            PieceKind::King => {
                piece::gen_jumping_moves(position, move_list, square120, &KING_QUEEN_DIRECTIONS);
                piece::gen_castling_moves(position, move_list, square120);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let pos = Position::starting_position();

        let pseudo = generate_pseudo_legal_moves(&pos);
        eprintln!("pseudo count: {}", pseudo.len());
        for m in &pseudo {
            eprintln!("{:?}", m);
        }
    }
}
