//Attack detection for Engine
// % centralize offset generator with one function
// % its a can be done but isnt a has to be done
// % then less problems if square120 + direction is negative for example
// % not possible to cast it to usize.

use crate::board::mailbox120::{
    BISHOP_DIRECTIONS, KNIGHT_DIRECTIONS, QUEEN_DIRECTIONS as KING_QUEEN_DIRECTIONS,
    ROOK_DIRECTIONS, is_on_board,
};
use crate::position::position::PieceKind;
use crate::position::{Cell, Color, Position, Square};

//main function, detects if given square is attacked
pub fn is_square_attacked(position: &Position, square: Square, by_color: Color) -> bool {
    let square120 = square.as_usize();
    debug_assert!(is_on_board(square120));
    attacked_by_pawn(position, square120, by_color)
        || attacked_by_knight(position, square120, by_color)
        || attacked_by_sliders(position, square120, by_color)
        || attacked_by_king(position, square120, by_color)
}

pub fn is_in_check(position: &Position, color: Color) -> bool {
    let cached_king_sq120 = position.king_sq[color.idx()] as usize;

    debug_assert!(
        is_on_board(cached_king_sq120),
        "cached_king_sq120 invalid: color={:?}, cached_king_sq120={}, fen={}",
        color, cached_king_sq120, position.to_fen()
    );

    let king_square = Square::new(cached_king_sq120 as u8);
    let enemy = color.opposite();
    is_square_attacked(position, king_square, enemy)
}

//checks if a pawn of given color attacks given square
//pawns attack diagonally
fn attacked_by_pawn(position: &Position, square120: usize, by_color: Color) -> bool {
    let pawn_attack_offsets = match by_color {
        Color::White => [-9, -11],
        Color::Black => [9, 11],
    };

    for &offset in &pawn_attack_offsets {
        let attacker_square = square120 as i32 + offset as i32;
        if attacker_square < 0 {
            continue;
        }
        let attacker_square = attacker_square as usize;

        if !is_on_board(attacker_square) {
            continue;
        }
        if let Cell::Piece(piece) = &position.board[attacker_square] {
            if piece.color == by_color && matches!(piece.kind, PieceKind::Pawn) {
                return true;
            }
        }
    }
    false
}

//checks if knight of given color attacks given square
fn attacked_by_knight(position: &Position, square120: usize, by_color: Color) -> bool {
    for &offset in &KNIGHT_DIRECTIONS {
        let attacker_square = square120 as i32 + offset as i32;
        if attacker_square < 0 {
            continue;
        }
        let attacker_square = attacker_square as usize;

        if !is_on_board(attacker_square) {
            continue;
        }
        if let Cell::Piece(piece) = &position.board[attacker_square] {
            if piece.color == by_color && matches!(piece.kind, PieceKind::Knight) {
                return true;
            }
        }
    }
    false
}

//checks if Slider of given color attacks given square
fn attacked_by_sliders(position: &Position, square120: usize, by_color: Color) -> bool {
    //checks ROOK
    for &direction in &ROOK_DIRECTIONS {
        if check_sliding_attack(position, square120, direction, by_color, true) {
            return true;
        }
    }

    //check Bishop
    for &direction in &BISHOP_DIRECTIONS {
        if check_sliding_attack(position, square120, direction, by_color, false) {
            return true;
        }
    }

    false
}

//helperfunction
fn check_sliding_attack(
    position: &Position,
    square120: usize,
    direction: i8,
    by_color: Color,
    is_rook_direction: bool, // true -> ROOK/QUEEN, false -> BISHOP/QUEEN
) -> bool {
    let dir_32 = direction as i32;
    let mut current = (square120 as i32 + dir_32);

    loop {
        //while(true) till break or return

        if current < 0 {
            break;
        }
        let current_usize = current as usize;

        if !is_on_board(current_usize) {
            break;
        }

        if let Cell::Piece(piece) = &position.board[current_usize] {
            //get out wrong colors
            if piece.color != by_color {
                return false;
            }

            //right color but which piecekind
            return match (is_rook_direction, &piece.kind) {
                (true, PieceKind::Rook) => true,
                (true, PieceKind::Queen) => true,
                (false, PieceKind::Bishop) => true,
                (false, PieceKind::Queen) => true,
                _ => false,
            };
        }

        //next
        current += dir_32;
    }
    false
}

fn attacked_by_king(position: &Position, square120: usize, by_color: Color) -> bool {
    for &offset in &KING_QUEEN_DIRECTIONS {
        let attacker_square = square120 as i32 + offset as i32;
        if attacker_square < 0 {
            continue;
        }
        let attacker_square = attacker_square as usize;

        if !is_on_board(attacker_square) {
            continue;
        }

        if let Cell::Piece(piece) = &position.board[attacker_square] {
            if piece.color == by_color && matches!(piece.kind, PieceKind::King) {
                return true;
            }
        }
    }
    false
}

pub fn find_king(position: &Position, color: Color) -> Option<usize> {
    for square120 in 21..=98 {
        if !is_on_board(square120) {
            continue;
        }
        if let Cell::Piece(piece) = &position.board[square120] {
            if piece.color == color && matches!(piece.kind, PieceKind::King) {
                return Some(square120);
            }
        }
    }
    None
}

//gibt square120 indizes der angreifer wieder
pub fn attackers_of_square(position: &Position, square120: usize, by_color: Color) -> Vec<usize> {
    let mut attackers = Vec::new();

    //Pawn
    let pawn_attack_offsets = match by_color {
        Color::White => [-9, -11],
        Color::Black => [9, 11],
    };

    for &offset in &pawn_attack_offsets {
        let attacker_square = square120 as i32 + offset as i32;
        if attacker_square < 0 {
            continue;
        }
        let attacker_square = attacker_square as usize;
        if is_on_board(attacker_square) {
            if let Cell::Piece(piece) = &position.board[attacker_square] {
                if piece.color == by_color && matches!(piece.kind, PieceKind::Pawn) {
                    attackers.push(attacker_square);
                }
            }
        }
    }

    //Knight
    for &offset in &KNIGHT_DIRECTIONS {
        let attacker_square = square120 as i32 + offset as i32;
        if attacker_square < 0 {
            continue;
        }
        let attacker_square = attacker_square as usize;

        if is_on_board(attacker_square) {
            if let Cell::Piece(piece) = &position.board[attacker_square] {
                if piece.color == by_color && matches!(piece.kind, PieceKind::Knight) {
                    attackers.push(attacker_square);
                }
            }
        }
    }

    //Slider ROOK;BISHOP;QUEEN
    for &direction in &BISHOP_DIRECTIONS {
        if let Some(attacker) =
            find_sliding_attacker(position, square120, direction, by_color, false)
        {
            attackers.push(attacker);
        }
    }

    for &direction in &ROOK_DIRECTIONS {
        if let Some(attacker) =
            find_sliding_attacker(position, square120, direction, by_color, true)
        {
            attackers.push(attacker);
        }
    }

    //King

    for &offset in &KING_QUEEN_DIRECTIONS {
        let attacker_square = square120 as i32 + offset as i32;
        if attacker_square < 0 {
            continue;
        }
        let attacker_square = attacker_square as usize;

        if is_on_board(attacker_square) {
            if let Cell::Piece(piece) = &position.board[attacker_square] {
                if piece.color == by_color && matches!(piece.kind, PieceKind::King) {
                    attackers.push(attacker_square)
                }
            }
        }
    }

    attackers
}

//helper function to help detecting sliding attacker
fn find_sliding_attacker(
    position: &Position,
    square120: usize,
    direction: i8,
    by_color: Color,
    is_rook_direction: bool, // true -> ROOK/QUEEN, false -> BISHOP/QUEEN
) -> Option<usize> {
    let dir_32 = direction as i32;
    let mut current = square120 as i32 + dir_32;

    loop {
        //while(true) till break or return

        if current < 0 {
            break;
        }
        let current_usize = current as usize;

        if !is_on_board(current_usize) {
            break;
        }

        if let Cell::Piece(piece) = &position.board[current_usize] {
            //get out wrong colors
            if piece.color != by_color {
                return None;
            }

            //right color but which piecekind
            let is_attacker = match (is_rook_direction, &piece.kind) {
                (true, PieceKind::Rook) => true,
                (true, PieceKind::Queen) => true,
                (false, PieceKind::Bishop) => true,
                (false, PieceKind::Queen) => true,
                _ => false,
            };

            return if is_attacker {
                Some(current_usize)
            } else {
                None
            };
        }

        //next
        current += dir_32;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::position::PieceKind;
    use crate::position::{Cell, Color, Position};

    // Helperfunction a1 = sq(0,0); h8 = sq(7,7);
    fn sq(file: i32, rank: i32) -> Square {
        Square::new((21 + file + rank * 10) as u8)
    }

    // Puts a piece on a sq
    fn put(pos: &mut Position, s: Square, color: Color, kind: PieceKind) {
        pos.board[s.as_usize()] = Cell::Piece(crate::position::Piece { color, kind });
    }

    #[test]
    fn pawn_attacks_diagonally_white() {
        let mut pos = Position::empty();

        let d4 = sq(3, 3);
        let e5 = sq(4, 4);

        put(&mut pos, d4, Color::White, PieceKind::Pawn);

        assert!(is_square_attacked(&pos, e5, Color::White));
        assert!(!is_square_attacked(&pos, e5, Color::Black));
    }

    #[test]
    fn pawn_attacks_diagonally_black() {
        let mut pos = Position::empty();

        let a7 = sq(0, 6);
        let b6 = sq(1, 5);

        put(&mut pos, a7, Color::Black, PieceKind::Pawn);

        assert!(is_square_attacked(&pos, b6, Color::Black));
        assert!(!is_square_attacked(&pos, b6, Color::White));
    }

    #[test]
    fn knight_attacks() {
        let mut pos = Position::empty();

        let f3 = sq(5, 2);
        let e5 = sq(4, 4);

        put(&mut pos, f3, Color::White, PieceKind::Knight);

        assert!(is_square_attacked(&pos, e5, Color::White));
    }

    #[test]
    fn slider_attack_blocked_by_piece() {
        let mut pos = Position::empty();

        let d1 = sq(3, 0);
        let d3 = sq(3, 2);
        let d8 = sq(3, 7);
        let b1 = sq(1, 0);
        let e4 = sq(4, 3);

        put(&mut pos, d1, Color::White, PieceKind::Rook);
        put(&mut pos, b1, Color::White, PieceKind::Queen);
        put(&mut pos, d3, Color::Black, PieceKind::Pawn);

        assert!(!is_square_attacked(&pos, d8, Color::White));
        assert!(!is_square_attacked(&pos, e4, Color::White));
        assert!(is_square_attacked(&pos, d3, Color::White));
        assert!(!is_square_attacked(&pos, d3, Color::Black));
    }

    #[test]
    fn slider_attack_unblocked() {
        let mut pos = Position::empty();

        let a1 = sq(0, 0);
        let a4 = sq(0, 3);

        let b1 = sq(1, 0);
        let b2 = sq(1, 1);
        let c2 = sq(2, 1);

        let e1 = sq(4, 0);
        let h4 = sq(7, 3);

        put(&mut pos, a1, Color::Black, PieceKind::Rook);
        put(&mut pos, b1, Color::Black, PieceKind::Queen);
        put(&mut pos, e1, Color::Black, PieceKind::Bishop);

        assert!(is_square_attacked(&pos, a4, Color::Black));
        assert!(!is_square_attacked(&pos, a4, Color::White));

        assert!(is_square_attacked(&pos, b2, Color::Black));
        assert!(is_square_attacked(&pos, c2, Color::Black));
        assert!(!is_square_attacked(&pos, b2, Color::White));
        assert!(!is_square_attacked(&pos, c2, Color::White));

        assert!(is_square_attacked(&pos, h4, Color::Black));
        assert!(!is_square_attacked(&pos, h4, Color::White));
    }

    #[test]
    fn king_attacks() {
        let mut pos = Position::empty();

        let a1 = sq(0, 0);
        let b2 = sq(1, 1);
        let c2 = sq(2, 1);
        let c4 = sq(2, 3);

        put(&mut pos, b2, Color::White, PieceKind::King);

        assert!(is_square_attacked(&pos, a1, Color::White));
        assert!(is_square_attacked(&pos, c2, Color::White));
        assert!(!is_square_attacked(&pos, c4, Color::White));
        assert!(!is_square_attacked(&pos, a1, Color::Black));
        assert!(!is_square_attacked(&pos, c2, Color::Black));
    }

    #[test]
    fn attackers_of_square_returns_all_attackers() {
        let mut pos = Position::empty();

        let target = sq(4, 4); // e5

        let d4 = sq(3, 3); // Pawn attacks e5
        let f3 = sq(5, 2); // Knight attacks e5

        put(&mut pos, d4, Color::White, PieceKind::Pawn);
        put(&mut pos, f3, Color::White, PieceKind::Knight);

        let mut attackers = attackers_of_square(&pos, target.as_usize(), Color::White);
        attackers.sort_unstable();

        let mut expected = vec![d4.as_usize(), f3.as_usize()];
        expected.sort_unstable();

        assert_eq!(attackers, expected);
    }

    #[test]
    fn is_in_check_rook_file() {
        let mut pos = Position::empty();

        let e8 = sq(4, 7);
        let e1 = sq(4, 0);
        let a1 = sq(0, 0); // irgendwo für den weißen König

        put(&mut pos, e8, Color::Black, PieceKind::King);
        put(&mut pos, e1, Color::White, PieceKind::Rook);
        put(&mut pos, a1, Color::White, PieceKind::King);

        pos.king_sq[Color::Black.idx()] = e8.get();
        pos.king_sq[Color::White.idx()] = a1.get();

        assert!(is_in_check(&pos, Color::Black));
        assert!(!is_in_check(&pos, Color::White));
    }

    #[test]
    fn find_king_finds_correct_square() {
        let mut pos = Position::empty();

        let g2 = sq(6, 1);
        put(&mut pos, g2, Color::White, PieceKind::King);

        assert_eq!(find_king(&pos, Color::White), Some(g2.as_usize()));
        assert_eq!(find_king(&pos, Color::Black), None);
    }

    #[test]
    fn no_panic_on_edge_squares() {
        let pos = Position::empty();

        let a1 = sq(0, 0);
        let h8 = sq(7, 7);

        assert!(!is_square_attacked(&pos, a1, Color::White));
        assert!(!is_square_attacked(&pos, a1, Color::Black));
        assert!(!is_square_attacked(&pos, h8, Color::White));
        assert!(!is_square_attacked(&pos, h8, Color::Black));
    }
}
