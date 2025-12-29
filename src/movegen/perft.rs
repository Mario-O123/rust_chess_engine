//Perft
use crate::movegen::{
    legal_move_filter::legal_move_filter,
    pseudo_legal_movegen::generate_pseudo_legal_moves,
    Move,
};
use crate::position::Position;

pub fn perft(position: &Position, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let pseudo_moves = generate_pseudo_legal_moves(position, position.last_move);
    let legal_moves = legal_move_filter(position, &pseudo_moves);

    let mut nodes = 0;

    for mv in legal_moves {
        let mut new_position =  position.clone();
        new_position.make_move(&mv);

        nodes += perft(&new_position, depth - 1);
    }

    nodes
}

