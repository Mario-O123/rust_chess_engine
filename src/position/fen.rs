use crate::board::conversion::{square120_from_string, square120_to_string};
use crate::board::mailbox120::{is_on_board, square120_from_file_rank};
use crate::position::{Cell, Color, Piece, Position, Square, PieceKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenError {
    InvalidFieldCount {found: usize},

    //board errors
    InvalidBoardFormat,
    InvalidOverallRankCount {found: usize},
    InvalidFileSumInOneRank {rank: usize, sum: usize},
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
        //fen_rank 0..=7 semantically equals fen order of rank 8... rank 1
        parse_one_rank(fen_rank, rank_string, pos)?;
    }

    Ok(())
}


fn parse_one_rank(fen_rank: usize, rank_string: &str, pos: &mut Position) -> Result<(), FenError> {
    let rank = 7 - fen_rank; //this way, the first fen_rank we get from enumerate(), which is semantically 8, gets turned into 7 internally
    
    let mut file_cursor: usize = 0; //cursor for the files, value of 8 means "rank is full"

    for ch in rank_string.chars() {
        match ch {
            '1'..='8' => { //numbers mean: "skip this many empty fields in a rank"
                let n = (ch as u8 - b'0') as usize;
                file_cursor += n; //adjust the file_cursor to jump forther in the rank

                if file_cursor > 8 { //the rank only ever contains 8 files
                    return Err(FenError::InvalidFileSumInOneRank {rank: fen_rank, sum: file_cursor});
                }
            }
            _ => { //otherwise match a character
                let piece = fen_char_to_piece(ch).ok_or(FenError::InvalidPieceChar {c: ch})?;

                if file_cursor >= 8 { // have to check if file_cursor is surpassed the maximum of 8 fields, if a number comes before a char in the rank_string
                    return Err(FenError::InvalidFileSumInOneRank {rank: fen_rank, sum: file_cursor});
                }

                let square120 = square120_from_file_rank(file_cursor, rank);
                pos.board[square120] = Cell::Piece(piece);
                file_cursor += 1; //increment normally after adding a char as piece
            }
        }
    
    } //in the end, the rank has to have 8 fields
    if file_cursor != 8 {
        return Err(FenError::InvalidFileSumInOneRank {rank: fen_rank, sum: file_cursor});
    }

    Ok(())
}

//helper for conversion of a char into a valid piece, think about maybe moving helper to board/conversion.rs
fn fen_char_to_piece(ch: char) -> Option<Piece> {
    let color = if ch.is_ascii_uppercase() {
        Color::White
    } else {
        Color::Black
    };

    let up = ch.to_ascii_uppercase();
    let kind = match up {
        'P' => PieceKind::Pawn,
        'N' => PieceKind::Knight,
        'B' => PieceKind::Bishop,
        'R' => PieceKind::Rook,
        'Q' => PieceKind::Queen,
        'K' => PieceKind::King,
        _ => return None,
    };

    Some(Piece {color, kind})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::mailbox120::square120_from_file_rank;

    #[test]
    fn parse_one_rank_places_correct_pieces_on_rank8() { //not exactly all 8 pieces are tested here
        let mut pos = Position::empty();

        parse_one_rank(0, "rnbqkbnr", &mut pos).expect("valid rank string should parse");
        
        let a8 = square120_from_file_rank(0, 7); //a8 should now be black rook
        assert_eq!(pos.board[a8], Cell::Piece(Piece {color: Color::Black, kind: PieceKind::Rook}));

        let d8 = square120_from_file_rank(3, 7); //d8 should now be a black queen
        assert_eq!(pos.board[d8], Cell::Piece(Piece {color: Color::Black, kind: PieceKind::Queen}));

        let e8 = square120_from_file_rank(4, 7); //e8 should now be a black king
        assert_eq!(pos.board[e8], Cell::Piece(Piece {color: Color::Black, kind: PieceKind::King}));
    }

    #[test]
    fn parse_one_rank_parses_digits_and_pieces_in_between() {
        let mut pos = Position::empty();

        //3p4 should be parsed as "3 empty cells, then 1 black pawn, ten 4 empty cells = 8 cells in total"
        parse_one_rank(0, "3p4", &mut pos).expect("valid rank string containing digits and characters, should pars");

        //pawn should land on file=3 (3 internally, 4 semantically), meaning file d, rank 8
        let d8 = square120_from_file_rank(3, 7);
        assert_eq!(pos.board[d8], Cell::Piece(Piece {color: Color::Black, kind: PieceKind::Pawn}));
    }

    #[test]
    fn parse_one_rank_rejects_too_large_file_sum() {
        let mut pos = Position::empty();
        let err = parse_one_rank(0, "9", &mut pos).unwrap_err();
        assert_eq!(err, FenError::InvalidFileSumInOneRank {rank: 0, sum: 9})
    }

    #[test]
    fn parse_one_rank_rejects_too_small_file_sum() {
        let mut pos = Position::empty();
        let err = parse_one_rank(0, "7", &mut pos).unwrap_err();
        assert_eq!(err, FenError::InvalidFileSumInOneRank {rank: 0, sum: 7});
    }

    #[test]
    fn parse_one_rank_rejects_invalid_piece_char() {
        let mut pos = Position::empty();
        let err = parse_one_rank(0, "7X", &mut pos).unwrap_err();
        assert_eq!(err, FenError::InvalidPieceChar {c: 'X'});
    }

    #[test]
    fn parse_piece_placement_startposition_can_be_set() { //only check a few pieces, after setting starting position
        let mut pos = Position::empty();

        parse_piece_placement("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", &mut pos).expect("Fen String is accepted as valid and interpretet as starting piece placement");
        
        let a8 = square120_from_file_rank(0, 7); //want to check if a8 (rook black) was parsed correctly
        assert_eq!(pos.board[a8], Cell::Piece(Piece {color: Color::Black, kind: PieceKind::Rook}));

        let e1 = square120_from_file_rank(4, 0); //want to check if e1 (king white) was parsed correctly
        assert_eq!(pos.board[e1], Cell::Piece(Piece {color: Color::White, kind: PieceKind::King}));
    }

    #[test]
    fn parse_piece_placement_rejects_wrong_rank_count() {
        let mut pos = Position::empty();
        let err = parse_piece_placement("8/8/8/8/8/8/8", &mut pos).unwrap_err();
        assert_eq!(err, FenError::InvalidOverallRankCount {found: 7});
    }

    #[test]
    fn parse_piece_placement_empty_board_can_be_parsed() {
        let mut pos = Position::empty();
        parse_piece_placement("8/8/8/8/8/8/8/8", &mut pos).expect("This FEN-Position is interpreted as an empty board");

        for file in 0..8 {
            for rank in 0..8 {
                let square120 = square120_from_file_rank(file, rank);
                assert_eq!(pos.board[square120], Cell::Empty);
            }
        }    
    }

    #[test]
    fn fen_char_to_piece_uppercase_is_white() {
        let piece = fen_char_to_piece('N').expect("N should map to a piece");
        assert_eq!(piece, Piece {color: Color::White, kind: PieceKind::Knight});
    }

    #[test]
    fn fen_char_to_piece_invalid_char_returns_none() {
        assert!(fen_char_to_piece('X').is_none());
    }



}




