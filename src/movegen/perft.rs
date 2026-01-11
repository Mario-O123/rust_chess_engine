//Perft
use crate::movegen::{filter_legal_moves, generate_pseudo_legal_moves};
use crate::position::Position;

pub fn perft(position: &Position, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let pseudo_moves = generate_pseudo_legal_moves(position);
    let legal_moves = filter_legal_moves(position, &pseudo_moves);

    let mut nodes = 0;

    for mv in legal_moves {
        let mut new_position = position.clone();
        new_position.make_move(mv);

        nodes += perft(&new_position, depth - 1);
    }

    nodes
}
