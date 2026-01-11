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
use crate::position::{Cell, Color, Position};

//main function, detects if given square is attacked
pub fn is_square_attacked(position: &Position, square120: usize, by_color: Color) -> bool {
    attacked_by_pawn(position, square120, by_color)
        || attacked_by_knight(position, square120, by_color)
        || attacked_by_sliders(position, square120, by_color)
        || attacked_by_king(position, square120, by_color)
}

pub fn is_in_check(position: &Position, color: Color) -> bool {
    let king_square = position.king_sq[color.idx()] as usize;
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
        if check_sliding_attack(position, square120, direction, &by_color, true) {
            return true;
        }
    }

    //check Bishop
    for &direction in &BISHOP_DIRECTIONS {
        if check_sliding_attack(position, square120, direction, &by_color, false) {
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
    by_color: &Color,
    is_rook_direction: bool, // true -> ROOK/QUEEN, false -> BISHOP/QUEEN
) -> bool {
    let mut current = (square120 as i32 + direction as i32) as usize;

    loop {
        //while(true) till break or return

        if !is_on_board(current) {
            break;
        }

        if let Cell::Piece(piece) = &position.board[current] {
            //get out wrong colors
            if piece.color != *by_color {
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
        current = (current as i32 + direction as i32) as usize;
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
            find_sliding_attacker(position, square120, direction, &by_color, false)
        {
            attackers.push(attacker);
        }
    }

    for &direction in &ROOK_DIRECTIONS {
        if let Some(attacker) =
            find_sliding_attacker(position, square120, direction, &by_color, true)
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
    by_color: &Color,
    is_rook_direction: bool, // true -> ROOK/QUEEN, false -> BISHOP/QUEEN
) -> Option<usize> {
    let mut current = (square120 as i32 + direction as i32) as usize;

    loop {
        //while(true) till break or return

        if !is_on_board(current) {
            break;
        }

        if let Cell::Piece(piece) = &position.board[current] {
            //get out wrong colors
            if piece.color != *by_color {
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

            return if is_attacker { Some(current) } else { None };
        }

        //next
        current = (current as i32 + direction as i32) as usize;
    }

    None
}
