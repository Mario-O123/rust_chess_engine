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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_one() {
        let pos = Position::starting_position();
        let legal_moves = perft(&pos, 1);
        assert_eq!(legal_moves, 20);
    }

    #[test]
    fn depth_two() {
        let pos = Position::starting_position();
        let legal_moves = perft(&pos, 2);
        assert_eq!(legal_moves, 400);
    }

    #[test]
    fn depth_three() {
        let pos = Position::starting_position();
        let legal_moves = perft(&pos, 3);
        assert_eq!(legal_moves, 8902);
    }

    #[test]
    fn depth_four() {
        let pos = Position::starting_position();
        let legal_moves = perft(&pos, 4);
        assert_eq!(legal_moves, 197281);
    }
}
