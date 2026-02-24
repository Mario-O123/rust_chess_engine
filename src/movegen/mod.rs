// === Public API ===
pub mod attack;
pub mod legal_move_filter;
pub mod r#move;
pub mod perft;
pub mod pseudo_legal_movegen;

// === Internal helpers ===
mod pawn;
mod piece;

// === Re-exports for clean imports ===
pub use legal_move_filter::filter_legal_moves;
pub use r#move::{Move, MoveType, PromotionPiece};
pub use perft::perft;
pub use pseudo_legal_movegen::{generate_pseudo_legal_moves, generate_pseudo_legal_moves_in_place};

pub use attack::is_in_check;

use crate::position::{self, Cell, Position};

#[inline]
fn is_capture(position: &Position, mv: Move) -> bool {
    mv.is_en_passant() || matches!(position.board[mv.to_sq()], Cell::Piece(_))
}

pub fn generate_legal_moves_in_place(pos: &mut Position, out: &mut Vec<Move>) {
    out.clear();
    let stm = pos.player_to_move;

    let pseudo = generate_pseudo_legal_moves(pos);
    for mv in pseudo {
        let undo = pos.make_move_with_undo(mv);
        let legal = !is_in_check(pos, stm);
        pos.undo_move(undo);

        if legal {
            out.push(mv);
        }
    }
}

pub fn generate_legal_captures_in_place(pos: &mut Position, out: &mut Vec<Move>) {
    out.clear();
    let stm = pos.player_to_move;

    let pseudo = generate_pseudo_legal_moves(pos);
    for mv in pseudo {
        if !is_capture(pos, mv) {
            continue;
        }
        let undo = pos.make_move_with_undo(mv);
        let legal = !is_in_check(pos, stm);
        pos.undo_move(undo);

        if legal {
            out.push(mv);
        }
    }
}
