//Knight/Bishop/Queen/King

//hier parameter square color offsets und der vector

use crate::movegen::Move;
use crate::movegen::attack::is_square_attacked;
use crate::position::{Cell, Color, Piece, PieceKind, Position};

pub fn gen_sliding_moves(
    position: &Position,
    moves: &mut Vec<Move>,
    square: usize,
    piece_offsets: &[i8],
) {
    for offset in piece_offsets {
        let mut target = square as i32 + *offset as i32;

        if target < 0 {
            continue;
        }
        while target >= 0 && target < 120 && position.board[target as usize] != Cell::Offboard {
            let target_usize = target as usize;
            if position.board[target as usize] == Cell::Empty {
                // push move
                moves.push(Move::new(square as usize, target as usize));
                target += *offset as i32;
            } else if matches!((position.board[target], position.board[square]), (Cell::Piece(slider_target), Cell::Piece(slider_square))
                                 if slider_target.color == slider_square.color.opposite())
            {
                //push move with capture
                moves.push(Move::new(square as usize, target as usize));

                break;
            } else {
                break;
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
    for offset in piece_offsets {
        if (square as i32 + *offset as i32) < 0 {
            continue;
        }
        if position.board[(square as i32 + *offset as i32) as usize] == Cell::Empty {
            //push move to vector
            moves.push(Move::new(
                square as usize,
                (square as i32 + *offset as i32) as usize,
            ));
        } else if matches!((position.board[(square as i32 + *offset as i32)as usize ], position.board[square]), (Cell::Piece(jumper_target), Cell::Piece(jumper_square))
            if jumper_target.color == jumper_square.color.opposite())
        {
            //push move to vector with capture flag
            moves.push(Move::new(
                square as usize,
                (square as i32 + *offset as i32) as usize,
            ));
        } else {
        }
    }
}

// color castling rights from gamestate and if between king and rook is none

// dsquare indexes are magic numbers should be changed if wrong or hard to understand

pub fn gen_castling_moves(position: &Position, moves: &mut Vec<Move>, king_from: usize) {
    match position.board[king_from] {
        Cell::Piece(Piece {
            color: Color::White,
            kind: PieceKind::King,
        }) => {
            if position.castling_rights & 0b0010 != 0
                && position.board[92] == Cell::Empty
                && position.board[93] == Cell::Empty
                && position.board[94] == Cell::Empty
                && matches!(
                    position.board[91],
                    Cell::Piece(Piece {
                        kind: PieceKind::Rook,
                        color: Color::White
                    })
                )
                && is_square_attacked(position, king_from, Color::Black) == false
                && is_square_attacked(position, 94, Color::Black) == false
                && is_square_attacked(position, 93, Color::Black) == false
                && is_square_attacked(position, 92, Color::Black) == false
                && is_square_attacked(position, 91, Color::Black) == false
            {
                moves.push(Move::new_castling(king_from, 93));
            }
            if position.castling_rights & 0b0001 != 0
                && position.board[96] == Cell::Empty
                && position.board[97] == Cell::Empty
                && matches!(
                    position.board[98],
                    Cell::Piece(Piece {
                        kind: PieceKind::Rook,
                        color: Color::White
                    })
                )
                && is_square_attacked(position, king_from, Color::Black) == false
                && is_square_attacked(position, 96, Color::Black) == false
                && is_square_attacked(position, 98, Color::Black) == false
                && is_square_attacked(position, 97, Color::Black) == false
            {
                moves.push(Move::new_castling(king_from, 97));
            }
        }
        Cell::Piece(Piece {
            color: Color::Black,
            kind: PieceKind::King,
        }) => {
            if position.castling_rights & 0b1000 != 0
                && position.board[22] == Cell::Empty
                && position.board[23] == Cell::Empty
                && position.board[24] == Cell::Empty
                && matches!(
                    position.board[21],
                    Cell::Piece(Piece {
                        kind: PieceKind::Rook,
                        color: Color::Black
                    })
                )
                && is_square_attacked(position, king_from, Color::White) == false
                && is_square_attacked(position, 24, Color::White) == false
                && is_square_attacked(position, 23, Color::White) == false
                && is_square_attacked(position, 22, Color::White) == false
                && is_square_attacked(position, 21, Color::White) == false
            {
                moves.push(Move::new_castling(king_from, 23));
            }
            if position.castling_rights & 0b0100 != 0
                && position.board[26] == Cell::Empty
                && position.board[27] == Cell::Empty
                && matches!(
                    position.board[28],
                    Cell::Piece(Piece {
                        kind: PieceKind::Rook,
                        color: Color::Black
                    })
                )
                && is_square_attacked(position, king_from, Color::White) == false
                && is_square_attacked(position, 26, Color::White) == false
                && is_square_attacked(position, 28, Color::White) == false
                && is_square_attacked(position, 27, Color::White) == false
            {
                moves.push(Move::new_castling(king_from, 27));
            }
        }
    }
}
