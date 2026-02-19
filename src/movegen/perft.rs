//Perft
use crate::movegen::{is_in_check, generate_pseudo_legal_moves_in_place, Move};
use crate::position::Position;

pub fn perft(position: &Position, depth: u32) -> u64 {
    let mut pos = position.clone();
    let mut buf: Vec<Move> = Vec::with_capacity(256);
    perft_mut(&mut pos, depth, &mut buf)
}

pub fn perft_mut(pos: &mut Position, depth: u32, buf: &mut Vec<Move>) -> u64 {
    if depth == 0 {
        return 1;
    }

    let side_to_move = pos.player_to_move;

    buf.clear();
    generate_pseudo_legal_moves_in_place(pos, buf);

    let mut nodes: u64 = 0;

    let moves: Vec<Move> = buf.clone();
    for mv in moves {
        let undo = pos.make_move_with_undo(mv);

        if !is_in_check(pos, side_to_move) {
            nodes += perft_mut(pos, depth -1, buf);
        }

        pos.undo_move(undo);
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
    #[test]
    #[ignore]
    fn startpos_depth_five() {
        let pos = Position::starting_position();
        assert_eq!(perft(&pos, 5), 4_865_609);
    }
    #[test]
    #[ignore]
    fn kiwipete_depth_four() {
        let pos = Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();
        assert_eq!(perft(&pos, 4), 4_085_603);
    }
}
