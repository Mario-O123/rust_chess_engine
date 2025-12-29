use crate::position::Position;
use crate::movegen::Move;
use crate::movegen::attack::is_in_check;


fn filter_legal_moves(position: &Position , moves : &[Move]) -> Vec<Move> {

   let mut legal_moves = Vec::new();

    for mv in moves.iter().copied() {
        let mut new_pos = position.clone();
        new_pos.make_move(mv);

        // Check if own King is in check
        if !is_in_check(&new_pos, position.player_to_move) {
            legal_moves.push(mv);
        }
    }

    legal_moves
}

