use crate::movegen::Move;
use crate::movegen::attack::is_in_check;
use crate::position::Position;

pub fn filter_legal_moves(position: &Position, moves: &[Move]) -> Vec<Move> {
    let mut legal_moves = Vec::new();

    let mut pos = position.clone();

    for mv in moves.iter().copied() {
        let undo = pos.make_move_with_undo(mv);

        let moved_color = pos.player_to_move.opposite();

        // Check if own King is in check
        if !is_in_check(&pos, moved_color) {
            legal_moves.push(mv);
        }

        pos.undo_move(undo);
    }

    legal_moves
}
