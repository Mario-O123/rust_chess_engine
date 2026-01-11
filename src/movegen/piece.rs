//Knight/Bishop/Queen/King

//hier parameter square color offsets und der vector

use crate::movegen::Move;
use crate::movegen::attack::is_square_attacked;
use crate::position::position::PieceKind;
use crate::position::{Cell, Color, Piece, Position, Square};

pub fn gen_sliding_moves(
    position: &Position,
    moves: &mut Vec<Move>,
    square: usize,
    piece_offsets: &[i8],
) {
    let Cell::Piece(moving_piece) = position.board[square] else {
        return;
    }; // early return added 

    for offset in piece_offsets {
        let mut target = square as i32 + *offset as i32;

        if target < 0 {
            continue;
        }

        while target >= 0 && target < 120 {
            let target_usize = target as usize;

            match position.board[target_usize] {
                Cell::Offboard => break,

                Cell::Empty => {
                    moves.push(Move::new(square, target_usize));
                    target += *offset as i32;
                }

                Cell::Piece(target_piece) => {
                    if target_piece.color != moving_piece.color {
                        moves.push(Move::new(square, target_usize));
                    }
                    break;
                }
            }
        }
    }
}

//maybe could save lines here by checking for if not offboard instead of Cell::Empty and Piece
pub fn gen_jumping_moves(
    position: &Position,
    moves: &mut Vec<Move>,
    square: usize,
    piece_offsets: &[i8],
) {
    let Cell::Piece(moving_piece) = position.board[square] else {
        return;
    };

    for offset in piece_offsets {
        let target = square as i32 + *offset as i32;

        if target < 0 || target >= 120 {
            continue;
        }

        let target_usize = target as usize;

        match position.board[target_usize] {
            Cell::Offboard => continue,

            Cell::Empty => {
                moves.push(Move::new(square, target_usize));
            }

            Cell::Piece(target_piece) => {
                if target_piece.color != moving_piece.color {
                    moves.push(Move::new(square, target_usize));
                }
            }
        }
    }
}

// color castling rights from gamestate and if between king and rook is none

// dsquare indexes are magic numbers should be changed if wrong or hard to understand

pub fn gen_castling_moves(position: &Position, moves: &mut Vec<Move>, king_from: usize) {
    let king_from_sq = Square::new(king_from as u8);

    match position.board[king_from] {
        Cell::Piece(Piece {
            color: Color::White,
            kind: PieceKind::King,
        }) => {
            if position.castling_rights & 0b0010 != 0
                && position.board[22] == Cell::Empty
                && position.board[23] == Cell::Empty
                && position.board[24] == Cell::Empty
                && matches!(
                    position.board[21],
                    Cell::Piece(Piece {
                        kind: PieceKind::Rook,
                        color: Color::White
                    })
                )
                && !is_square_attacked(position, king_from_sq, Color::Black)
                && !is_square_attacked(position, Square::new(24), Color::Black)
                && !is_square_attacked(position, Square::new(23), Color::Black)
            {
                moves.push(Move::new_castling(king_from, 23));
            }
            if position.castling_rights & 0b0001 != 0
                && position.board[26] == Cell::Empty
                && position.board[27] == Cell::Empty
                && matches!(
                    position.board[28],
                    Cell::Piece(Piece {
                        kind: PieceKind::Rook,
                        color: Color::White
                    })
                )
                && !is_square_attacked(position, king_from_sq, Color::Black)
                && !is_square_attacked(position, Square::new(26), Color::Black)
                && !is_square_attacked(position, Square::new(27), Color::Black)
            {
                moves.push(Move::new_castling(king_from, 27));
            }
        }
        Cell::Piece(Piece {
            color: Color::Black,
            kind: PieceKind::King,
        }) => {
            if position.castling_rights & 0b1000 != 0
                && position.board[92] == Cell::Empty
                && position.board[93] == Cell::Empty
                && position.board[94] == Cell::Empty
                && matches!(
                    position.board[91],
                    Cell::Piece(Piece {
                        kind: PieceKind::Rook,
                        color: Color::Black
                    })
                )
                && !is_square_attacked(position, king_from_sq, Color::White)
                && !is_square_attacked(position, Square::new(9), Color::White)
                && !is_square_attacked(position, Square::new(93), Color::White)
            {
                moves.push(Move::new_castling(king_from, 93));
            }
            if position.castling_rights & 0b0100 != 0
                && position.board[96] == Cell::Empty
                && position.board[97] == Cell::Empty
                && matches!(
                    position.board[98],
                    Cell::Piece(Piece {
                        kind: PieceKind::Rook,
                        color: Color::Black
                    })
                )
                && !is_square_attacked(position, king_from_sq, Color::White)
                && !is_square_attacked(position, Square::new(96), Color::White)
                && !is_square_attacked(position, Square::new(97), Color::White)
            {
                moves.push(Move::new_castling(king_from, 97));
            }
        }

        _ => {}
    }
}
