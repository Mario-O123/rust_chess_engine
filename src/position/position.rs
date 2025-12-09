pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Color {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

pub enum Piecekind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

pub struct Piece {
    pub color: Color,
    pub kind: Piecekind,
}

pub type Square = u8;

pub struct Position {
    pub board: [Option<Piece>; 120],
    pub player_to_move: Color,
    pub en_passant_square: Option<Square>,
    pub castling_rights: u8, // 4 bits: White 0-0 (1), White 0-0-0 (2), Black 0-0 (4), Black 0-0-0 (8)
    pub zobrist: u64,
    pub half_move_clock: u8,
}

impl Position {
    fn empty() {
        // TO-DO!
    }

    fn piece_at() {
        // TO-DO!
    }

    fn set_board() {
        // TO-DO!
    }

    fn get_board() {
        // TO-DO!
    }

    fn set_castling_rights() {
        // TO-DO!
    }

    fn check_en_passant() {
        // TO-DO!
    }

    fn set_half_move_clock() {
        // TO-DO!
    }

    fn create_zobrist() {
        // TO-DO!
    }

    fn update_zobrist() {
        // TO-DO!
    }
}
