


fn legal_move_filter(position: &Position , moves : &[Move]) -> Vec<Move> {

   let mut legal_moves = Vec::new();

    for &pseudo_move in moves {
        let mut new_pos = position.clone();
        new_pos.make_move(pseudo_move);

        // Check if own King is in check
        if !is_in_check(&new_pos, position.player_to_move) {
            legal_moves.push(pseudo_move);
        }
    }

    legal_moves
}

