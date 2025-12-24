//Tests
use crate::{movegen::{
    lega_move_filter::filter_legal_moves,
    pseudo_legal_movegen::generate_pseudo_legal_moves
    },
    movegen::Move,
    position::Position,
};

pub fn perft(position: &Position, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let pseudo_moves = generate_pseudo_legal_moves(position, position.last_move);
    let legal_moves = filter_legal_moves(position, &pseudo_moves);

    let mut nodes = 0;

    for mv in legal_moves {
        let mut new_position =  position.clone();
        new_position.make_mmove(mv);

        nodes += perft(&new_position, depth - 1);
    }

    nodes
}

