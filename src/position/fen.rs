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

    InvalidKingCount {color: Color, found: usize},
}

impl Position {

    pub fn from_fen(fen_string: &str) -> Result<Self, FenError> {
        let fields: Vec<&str> = fen_string.split_whitespace().collect();
        if fields.len() != 6 {
            return Err(FenError::InvalidFieldCount {found: fields.len()});
        }
        let mut pos = Position::empty();

        parse_piece_placement(fields[0], &mut pos)?;
        pos.player_to_move =parse_active_color(fields[1])?;
        pos.castling_rights = parse_castling(fields[2])?;
        pos.en_passant_square = parse_en_passant(fields[3])?;
        pos.half_move_clock = parse_halfmove_clock(fields[4])?;
        pos.move_counter = parse_fullmove_counter(fields[5])?;

        //sanity-check: we have exactly 1 white and black king, this way we don't have to rely on the panic in compute_king_sq()
        let white_king_count = pos.find_pieces(Color::White, PieceKind::King).len();
        if white_king_count != 1 {
            return Err(FenError::InvalidKingCount {
                color: Color::White,
                found: white_king_count,
            });
        }

        let black_king_count = pos.find_pieces(Color::Black, PieceKind::King).len();
        if black_king_count != 1 {
            return Err(FenError::InvalidKingCount {
                color: Color::Black,
                found: black_king_count,
            });
        }

        pos.king_sq = pos.compute_king_sq();
        pos.piece_counter = pos.compute_piece_counter();
        pos.zobrist = pos.compute_zobrist();

        Ok(pos)
    }

    
    pub fn to_fen(&self) -> String {
        let board = piece_placement_to_string(self);
        let active_color = active_color_to_string(self.player_to_move);
        let castling = castling_to_string(self.castling_rights);
        let en_passant = en_passant_to_string(self.en_passant_square);

        format!("{} {} {} {} {} {}", board, active_color, castling, en_passant, self.half_move_clock, self.move_counter)
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
            '0'..='9' => { //numbers mean: "skip this many empty fields in a rank"
                let n = (ch as u8 - b'0') as usize;

                if !(1..=8).contains(&n) {
                    return Err(FenError::InvalidBoardFormat);
                }

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

//helper for parsing the second FEN-field (strict parsing, don't accept uppercase)
fn parse_active_color(field: &str) -> Result<Color, FenError> {
    match field {
        "w" => Ok(Color::White),
        "b" => Ok(Color::Black),
        _ => Err(FenError::InvalidCurrentColor),
    }
}

//helper for parsing the third FEN-field (tolerant parsing, accept different ordering)
fn parse_castling(field: &str) -> Result<u8, FenError> {
    if field == "-" {
        return Ok(0);
    }
    //Maybe add here another if: "0" returns Ok(0)
    if field.contains('-') {
        return Err(FenError::InvalidCastling);
    }

    let mut rights: u8 = 0;
    for ch in field.chars() {
        let bit = match ch {
            'K' => 0b0001,
            'Q' => 0b0010,
            'k' => 0b0100,
            'q' => 0b1000,
            _ => return Err(FenError::InvalidCastling),
        };

        //bitwise addition on rights resulting in 1, means the bit was already updated (read before)
        if rights & bit != 0 {
        return Err(FenError::InvalidCastling);
        }
        //update the bit in the rights, only if the corresponding character is read the first time
        rights |= bit;
    }

    Ok(rights)
}

fn parse_en_passant(field: &str) -> Result<Option<Square>, FenError> {
    if field == "-" {
        return Ok(None);
    }
    let square120 = square120_from_string(field).ok_or(FenError::InvalidEnPassant)?;

    if !is_on_board(square120) {
        return Err(FenError::InvalidEnPassant);
    }

    let square_string = square120_to_string(square120).ok_or(FenError::InvalidEnPassant)?;
    let rank_byte = square_string.as_bytes()[1];

    if rank_byte != b'3' && rank_byte != b'6' {
        return Err(FenError::InvalidEnPassant);
    }

    Ok(Some(Square::new(square120 as u8)))
}

fn parse_halfmove_clock(field: &str) -> Result<u16, FenError> {
    let parsed = field.parse::<u16>();

    match parsed {
        Ok(value) => Ok(value),
        Err(_) => Err(FenError::InvalidHalfmove),
    }
}

fn parse_fullmove_counter(field: &str) -> Result<u16, FenError> {
    let parsed = field.parse::<u16>();

    match parsed {
        Ok(value) =>
        if value < 1 {
            Err(FenError::InvalidFullmove)
        }
        else {
            Ok(value)
        }
        Err(_) => Err(FenError::InvalidFullmove),
    }
}



//helpers for to_fen()

fn piece_placement_to_string(pos: &Position) -> String {
    let mut ranks: Vec<String> = Vec::with_capacity(8);

    for rank in (0..8).rev() {
        ranks.push(encode_rank(pos, rank));
    }

    ranks.join("/")
}

//use as helper for piece_placement_to_string, for each specific rank
fn encode_rank(pos: &Position, rank: usize) -> String {
    let mut output_string = String::new();
    let mut empty_squares: u8 = 0;

    for file in 0..8 {
        let square120 = square120_from_file_rank(file, rank);

        match pos.board[square120] {
            Cell::Empty => { empty_squares += 1; }
            Cell::Piece(piece) => {
                if empty_squares > 0 {
                    output_string.push(std::char::from_digit(empty_squares as u32, 10).unwrap());
                    empty_squares = 0;
                }
                output_string.push(piece_to_fen_char(piece));
            }
            Cell::Offboard => { unreachable!("square120_from_file_square returned offboard square") } //maybe re-think this handling because of crashing?
        }
    }

    if empty_squares > 0 {
            output_string.push(std::char::from_digit(empty_squares as u32, 10).unwrap());
    }

    output_string
}

//use as helper for encode_rank()
//since we changed Piece in position.rs, our piece_to_char() in conversion.rs is of no use now
fn piece_to_fen_char(piece: Piece) -> char {
    let piece_char = match piece.kind {
            PieceKind::Pawn => 'p',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::Rook => 'r',
            PieceKind::Queen => 'q',
            PieceKind::King => 'k',
    };

    match piece.color {
            Color::White => piece_char.to_ascii_uppercase(),
            Color::Black => piece_char,
    }
}

fn active_color_to_string(color: Color) -> &'static str {
    match color {
        Color::White => "w",
        Color::Black => "b",
    }
}

fn castling_to_string(rights: u8) -> String {
    if rights == 0 {
        return "-".to_string();
    }

    let mut s = String::new();
    if rights & 0b0001 != 0 {s.push('K');}
    if rights & 0b0010 != 0 {s.push('Q');}
    if rights & 0b0100 != 0 {s.push('k');}
    if rights & 0b1000 != 0 {s.push('q');}

    //extra check
    if s.is_empty() {"-".to_string()} else {s}
}

fn en_passant_to_string(ep_target: Option<Square>) -> String {
    match ep_target {
        None => "-".to_string(),
        Some(square) => square120_to_string(square.as_usize()).unwrap_or_else(|| "-".to_string()),
    }
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
        assert_eq!(err, FenError::InvalidBoardFormat);
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

    #[test]
    fn parse_active_color_accepts_w_and_b() {
        assert_eq!(parse_active_color("w").unwrap(), Color::White);
        assert_eq!(parse_active_color("b").unwrap(), Color::Black);
    }

    #[test]
    fn parse_active_color_rejects_invalid() {
        assert_eq!(parse_active_color("c").unwrap_err(), FenError::InvalidCurrentColor);
    }

    #[test]
    fn parse_castling_dash_means_no_rights() {
        assert_eq!(parse_castling("-").unwrap(), 0);
    }

    #[test]
    fn parse_castling_all_rights() {
        assert_eq!(parse_castling("KQkq").unwrap(), 0b1111);
    }

    #[test]
    fn parse_castling_partiel_rights() {
        assert_eq!(parse_castling("Kq").unwrap(), 0b1001)
    }
    
    #[test]
    fn parse_castling_rejects_invalid_character() {
        assert_eq!(parse_castling("Kx").unwrap_err(), FenError::InvalidCastling);
    }

    #[test]
    fn parse_castling_rights_rejects_dash_with_other_chars() {
        assert_eq!(parse_castling("K-").unwrap_err(), FenError::InvalidCastling);
    }

    #[test]
    fn parse_castling_rejects_duplicafes() {
        assert_eq!(parse_castling("KKq").unwrap_err(), FenError::InvalidCastling);
    }
    
    #[test]
    fn parse_castling_accepts_any_rights_order() {
        assert_eq!(parse_castling("qK").unwrap(), 0b1001);
    }

    #[test]
    fn parse_en_passant_dash_is_none() {
        assert_eq!(parse_en_passant("-").unwrap(), None);
    }

    #[test]
    fn parse_en_passant_accepts_valid_square() {
        let en_passant = parse_en_passant("e3").unwrap();
        assert!(en_passant.is_some());
    }

    #[test]
    fn parse_en_passant_rejects_invalid_square() {
        let error = parse_en_passant("x9").unwrap_err();
        assert_eq!(error, FenError::InvalidEnPassant);
    }

    #[test]
    fn parse_en_passant_rejects_wrong_ranks() {
        let error = parse_en_passant("e4").unwrap_err();
        assert_eq!(error, FenError::InvalidEnPassant);
    }

    #[test]
    fn parse_halfmove_clock_accepts_zero_and_above() {
        assert_eq!(parse_halfmove_clock("0").unwrap(), 0);
        assert_eq!(parse_halfmove_clock("1").unwrap(), 1);
    }

    #[test]
    fn parse_halfmove_clock_rejects_invalid_input() {
        assert_eq!(parse_halfmove_clock("-1").unwrap_err(), FenError::InvalidHalfmove);
        assert_eq!(parse_halfmove_clock("abc").unwrap_err(), FenError::InvalidHalfmove);
    }

    #[test]
    fn parse_fullmove_counter_accepts_one_and_above() {
        assert_eq!(parse_fullmove_counter("1").unwrap(), 1);
        assert_eq!(parse_fullmove_counter("2").unwrap(), 2);
    }

    #[test]
    fn parse_fullmove_counter_rejects_zero_and_invalid() {
        assert_eq!(parse_fullmove_counter("0").unwrap_err(), FenError::InvalidFullmove);
        assert_eq!(parse_fullmove_counter("x").unwrap_err(), FenError::InvalidFullmove);
    }

    #[test]
    fn from_fen_parses_startposition() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let pos = Position::from_fen(fen).expect("startposition fen should parse");

        let a8 = square120_from_file_rank(0, 7);
        assert_eq!(pos.board[a8], Cell::Piece(Piece {color: Color::Black, kind: PieceKind::Rook}));

        let e1 = square120_from_file_rank(4, 0);
        assert_eq!(pos.board[e1], Cell::Piece(Piece {color: Color::White, kind: PieceKind::King}));
    
        assert_eq!(pos.player_to_move, Color::White);
        assert_eq!(pos.castling_rights, 0b1111);
        assert_eq!(pos.en_passant_square, None);
        assert_eq!(pos.half_move_clock, 0);
        assert_eq!(pos.move_counter, 1);
    }

    #[test]
    fn from_fen_rejects_wrong_field_count() {
        let fen = "8/8/8/8/8/8/8/8 1 - - 0";
        let error = Position::from_fen(fen).unwrap_err();
        assert_eq!(error, FenError::InvalidFieldCount {found: 5});
    }

    #[test]
    fn from_fen_rejects_fullmove_zero() {
        let fen = "8/8/8/8/8/8/8/8 w - - 0 0";
        let error = Position::from_fen(fen).unwrap_err();
        assert_eq!(error, FenError::InvalidFullmove);
    }

    #[test]
    fn from_fen_rejects_missing_king() {
        let fen = "8/8/8/8/8/8/8/4K3 w - - 0 1";
        let error = Position::from_fen(fen).unwrap_err();
        assert_eq!(error, FenError::InvalidKingCount {color: Color::Black, found: 0});
    }

    #[test]
    fn from_fen_rejects_two_kings() {
        let fen = "8/8/8/8/8/8/8/3KK3 w - - 0 1";
        let error = Position::from_fen(fen).unwrap_err();
        assert_eq!(error, FenError::InvalidKingCount {color: Color::White, found: 2});
    }

    //testing to_fen() helpers
    #[test]
    fn enocode_rank_empty_rank_is_8() {
        let pos = Position::empty();
        assert_eq!(encode_rank(&pos, 7), "8");
    }

    fn encode_rank_startposition_rank8_is_correct() {
        let pos = Position::starting_position();
        assert_eq!(encode_rank(&pos, 7), "rnbqkbnr");
    }
    
    #[test]
    fn ecode_rank_single_piece_with_gaps() {
        let mut pos = Position::empty();
        let d8 = square120_from_file_rank(3, 7);
        pos.board[d8] = Cell::Piece(Piece {color: Color::Black, kind: PieceKind::Queen});
        assert_eq!(encode_rank(&pos, 7), "3q4");
    }

    #[test]
    fn encode_rank_flushes_trailing_empty_squares() {
        let mut pos = Position::empty();
        let a8 = square120_from_file_rank(0, 7);
        pos.board[a8] = Cell::Piece(Piece {color: Color::Black, kind: PieceKind::Rook});
        assert_eq!(encode_rank(&pos, 7), "r7");
    }

    #[test]
    fn piece_placement_to_string_empty_board_is_all_8s() {
        let pos = Position::empty();
        assert_eq!(piece_placement_to_string(&pos), "8/8/8/8/8/8/8/8");
    }
    #[test]
    fn piece_placement_to_string_startpos_matches_expected() {
        let pos = Position::starting_position();
        assert_eq!(piece_placement_to_string(&pos), "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
    }

    #[test]
    fn piece_placement_to_string_single_piece() {
        let mut pos = Position::empty();
        let d8 = square120_from_file_rank(3, 7);
        pos.board[d8] = Cell::Piece(Piece {color: Color::Black, kind: PieceKind::Queen});
        assert_eq!(piece_placement_to_string(&pos), "3q4/8/8/8/8/8/8/8");
    }


    #[test]
    fn piece_to_fen_char_uppercase_and_lowercase_works() {
        let white_char = piece_to_fen_char(Piece {color: Color::White, kind: PieceKind::Knight});
        assert_eq!(white_char, 'N');

        let black_char = piece_to_fen_char(Piece {color: Color::Black, kind: PieceKind::Queen});
        assert_eq!(black_char, 'q');
    }

    #[test]
    fn castling_to_string_zero_is_dash() {
        assert_eq!(castling_to_string(0), "-".to_string());
    }

    #[test]
    fn castling_to_string_all_rights_is_KQkq() {
        assert_eq!(castling_to_string(0b1111), "KQkq".to_string());
    }
    #[test]
    fn castling_to_string_canonical_order() {
        assert_eq!(castling_to_string(0b0101), "Kk".to_string());
        assert_eq!(castling_to_string(0b1001), "Kq".to_string());
        assert_eq!(castling_to_string(0b0110), "Qk".to_string());
    }

    #[test]
    fn en_passant_to_string_none_and_some_work() {
        assert_eq!(en_passant_to_string(None), "-".to_string());
        let square120 = square120_from_string("e3").unwrap();
        let ep = Some(Square::new(square120 as u8));
        assert_eq!(en_passant_to_string(ep), "e3".to_string());
    }

    //testing to_fen

    #[test]
    fn to_fen_startpos_matches_expected() {
        let pos = Position::starting_position();
        assert_eq!(pos.to_fen(), "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    }
    
    #[test]
    fn to_fen_empty_position_matches_expected() {
        let pos = Position::empty();
        assert_eq!(pos.to_fen(), "8/8/8/8/8/8/8/8 w - - 0 1");
    }

    #[test]
    fn to_fen_black_to_move() {
        let mut pos = Position::empty();
        pos.player_to_move = Color::Black;
        assert_eq!(pos.to_fen(), "8/8/8/8/8/8/8/8 b - - 0 1");
    }

    #[test]
    fn to_fen_gives_castling_rights_in_canonic_order() {
        let mut pos = Position::empty();
        pos.castling_rights = 0b1001;
        assert_eq!(pos.to_fen(), "8/8/8/8/8/8/8/8 w Kq - 0 1");
    }

    #[test]
    fn to_fen_includes_en_passant_square() {
        let mut pos = Position::empty();

        let square120 = square120_from_string("e3").unwrap();
        pos.en_passant_square = Some(Square::new(square120 as u8));

        assert_eq!(pos.to_fen(), "8/8/8/8/8/8/8/8 w - e3 0 1");
    }

    #[test]
    fn to_fen_halfmove_and_fullmove_fields_are_used() {
        let mut pos = Position::empty();
        pos.half_move_clock = 6;
        pos.move_counter = 40;
        assert_eq!(pos.to_fen(), "8/8/8/8/8/8/8/8 w - - 6 40");
    }
    


}




