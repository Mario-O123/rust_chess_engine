use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

impl Piece {
    const fn new(color: Color, kind: PieceKind) -> Self {
        Self { color, kind }
    }
}

pub type Square = u8;

// zobrist_values: [[[Squares] Piecekinds] Colors]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Zobrist {
    pub zobrist_values: [[[u64; 64]; 6]; 2],
    pub zobrist_side_to_move: u64,
    pub zobrist_castling: [u64; 4],
    pub zobrist_enpassant_row: [u64; 8],
}

impl Zobrist {
    // Same Seed generates same random u64 Numbers everytime function is called
    pub fn new_zobrist() -> Self {
        const SEED: u64 = 42;
        let mut rng = StdRng::seed_from_u64(SEED);
        let mut zobrist_values = [[[0u64; 64]; 6]; 2];

        for i in 0..2 {
            for j in 0..6 {
                for k in 0..64 {
                    zobrist_values[i][j][k] = rng.r#gen();
                }
            }
        }

        let zobrist_side_to_move = rng.r#gen();

        let mut zobrist_castling = [0u64; 4];
        for i in 0..4 {
            zobrist_castling[i] = rng.r#gen();
        }

        let mut zobrist_enpassant_row = [0u64; 8];
        for i in 0..8 {
            zobrist_enpassant_row[i] = rng.r#gen();
        }

        Self {
            zobrist_values: zobrist_values,
            zobrist_side_to_move: zobrist_side_to_move,
            zobrist_castling: zobrist_castling,
            zobrist_enpassant_row: zobrist_enpassant_row,
        }
    }
}

pub const START_BOARD: [Option<Piece>; 64] = [
    Some(Piece::new(Color::Black, PieceKind::Rook)),
    Some(Piece::new(Color::Black, PieceKind::Knight)),
    Some(Piece::new(Color::Black, PieceKind::Bishop)),
    Some(Piece::new(Color::Black, PieceKind::Queen)),
    Some(Piece::new(Color::Black, PieceKind::King)),
    Some(Piece::new(Color::Black, PieceKind::Bishop)),
    Some(Piece::new(Color::Black, PieceKind::Knight)),
    Some(Piece::new(Color::Black, PieceKind::Rook)),
    Some(Piece::new(Color::Black, PieceKind::Pawn)),
    Some(Piece::new(Color::Black, PieceKind::Pawn)),
    Some(Piece::new(Color::Black, PieceKind::Pawn)),
    Some(Piece::new(Color::Black, PieceKind::Pawn)),
    Some(Piece::new(Color::Black, PieceKind::Pawn)),
    Some(Piece::new(Color::Black, PieceKind::Pawn)),
    Some(Piece::new(Color::Black, PieceKind::Pawn)),
    Some(Piece::new(Color::Black, PieceKind::Pawn)),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(Piece::new(Color::White, PieceKind::Pawn)),
    Some(Piece::new(Color::White, PieceKind::Pawn)),
    Some(Piece::new(Color::White, PieceKind::Pawn)),
    Some(Piece::new(Color::White, PieceKind::Pawn)),
    Some(Piece::new(Color::White, PieceKind::Pawn)),
    Some(Piece::new(Color::White, PieceKind::Pawn)),
    Some(Piece::new(Color::White, PieceKind::Pawn)),
    Some(Piece::new(Color::White, PieceKind::Pawn)),
    Some(Piece::new(Color::White, PieceKind::Rook)),
    Some(Piece::new(Color::White, PieceKind::Knight)),
    Some(Piece::new(Color::White, PieceKind::Bishop)),
    Some(Piece::new(Color::White, PieceKind::Queen)),
    Some(Piece::new(Color::White, PieceKind::King)),
    Some(Piece::new(Color::White, PieceKind::Bishop)),
    Some(Piece::new(Color::White, PieceKind::Knight)),
    Some(Piece::new(Color::White, PieceKind::Rook)),
];

// castling_rights uses 4 bits: White 0-0 (0b0001), White 0-0-0 (0b0010),
// Black 0-0 (0b0100), Black 0-0-0 (0b1000)
pub struct Position {
    pub board: [Option<Piece>; 64],
    pub player_to_move: Color,
    pub en_passant_square: Option<Square>,
    pub castling_rights: u8,
    pub zobrist: u64,
    pub half_move_clock: u8,
    pub move_counter: u16,
    pub zobrist_object: Zobrist,
}

impl Position {
    fn empty() -> Self {
        Self {
            board: [None; 64],
            player_to_move: Color::White,
            en_passant_square: None,
            castling_rights: 0,
            zobrist: 0,
            half_move_clock: 0,
            move_counter: 0,
            zobrist_object: Zobrist::new_zobrist(),
        }
    }

    fn starting_position(&self) -> Self {
        let mut pos = Self::empty();
        pos.board = START_BOARD;
        pos.castling_rights = 0b1111;
        pos.zobrist = Self::compute_zobrist(&self);
        pos
    }

    fn piece_at() {
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

    fn compute_zobrist(&self) -> u64 {
        let mut zobrist: u64 = 0;
        for (square, maybe_piece) in self.board.iter().enumerate() {
            if let Some(piece) = maybe_piece {
                zobrist ^= self.zobrist_object.zobrist_values[piece.color as usize]
                    [piece.kind as usize][square];
            }
        }

        if self.player_to_move == Color::White {
            zobrist ^= self.zobrist_object.zobrist_side_to_move
        }

        // castling fehlt
        // en passant fehlt

        zobrist
    }
}
