use crate::board::conversion::{square120_from_string, square120_to_string};
use crate::board::mailbox120::{is_on_board, square120_from_file_rank};
use crate::position::{Cell, Color, Piece, Position, Square, PieceKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenError {
    InvalidFieldCount {found: usize},

    //board errors
    InvalidBoardFormat,
    InvalidOverallRankCount {found: usize},
    InvalidRankSumInOneRank {rank: usize, sum: usize},
    InvalidPieceChar {c: char},

    InvalidCurrentColor,
    InvalidCastling,
    InvalidEnPassant,
    InvalidHalfmove,
    InvalidFullmove,

    InvalidKingCount {color: Color, found: u8},
}

impl Position {
    pub fn from_fen(fen_string: &str) -> Result<Self, FenError> {
        let fields: Vec<&str> = fen_string.split_whitespace().collect();
        if fields.len() != 6 {
            return Err(FenError::InvalidFieldCount {found: fields.len()});
        }
        let mut pos = Position::empty();

        parse_piece_placement(fields[0], &mut pos)?;

    }
}

fn parse_piece_placement(board_fields: &str, pos: &mut Position) -> Result<(), FenError> {
    let ranks: Vec<&str> = board_fields.split("/").collect();
    if ranks.len() != 8 {
        return Err(FenError::InvalidOverallRankCount {found: ranks.len()});
    }
    for (fen_rank, rank_string) in ranks.iter().enumerate() {
        let rank = 7 - fen_rank;
        let mut file: usize = 0; //maybe not "file"

        for c in rank_string.chars() {
            if c.is_ascii_digit() {
                let n = (c as u8 - b'0') as usize;
                if !(1..=8).contains(&n) {
                    return Err(FenError::InvalidBoardFormat);
                }
                file += n;
                if file > 8 {
                    return Err(FenError::InvalidRankSumInOneRank {rank: fen_rank, sum: file})
                }
            }
            continue;
        let piece = fen_char_to_piece(c).ok_or(FenError::InvalidPieceChar {c})?;
        if file >= 8 {
            return Err(FenError::InvalidRankSumInOneRank {rank: fen_rank, sum: file });
        }
        let square120 = square120_from_file_rank(file, rank);
        pos.board[square120] = Cell::Piece(piece);
        file += 1;
        }
        if file != 8 {
            return Err(FenError::InvalidRankSumInOneRank { rank: fen_rank, sum: file });
        }        
    }
    Ok(())
}