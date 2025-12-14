use crate::board::mailbox120::SQUARE120_TO_SQUARE64;
use once_cell::sync::Lazy;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

const BOARD64: usize = 64;
const BOARD120: usize = 120;

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

    pub fn idx(&self) -> u8 {
        match self {
            Color::White => 0,
            Color::Black => 1,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Cell {
    Piece(Piece),
    Offboard,
    Empty,
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

impl PieceKind {
    pub fn idx(self) -> u8 {
        match self {
            PieceKind::Pawn => 0,
            PieceKind::Knight => 1,
            PieceKind::Bishop => 2,
            PieceKind::Rook => 3,
            PieceKind::Queen => 4,
            PieceKind::King => 5,
        }
    }
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

// zobrist_values: [[[Squares]; Piecekinds]; Colors]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Zobrist {
    pub zobrist_values: [[[u64; BOARD64]; 6]; 2],
    pub zobrist_side_to_move: u64,
    pub zobrist_castling: [u64; 4],
    pub zobrist_enpassant: [u64; 8],
}

impl Zobrist {
    // Same Seed generates same random u64 Numbers everytime function is called
    pub fn init_zobrist() -> Self {
        const SEED: u64 = 42;
        let mut rng = StdRng::seed_from_u64(SEED);
        let mut zobrist_values = [[[0u64; BOARD64]; 6]; 2];

        for i in 0..2 {
            for j in 0..6 {
                for k in 0..BOARD64 {
                    zobrist_values[i][j][k] = rng.r#gen();
                }
            }
        }

        let zobrist_side_to_move = rng.r#gen();

        let mut zobrist_castling = [0u64; 4];
        for i in 0..4 {
            zobrist_castling[i] = rng.r#gen();
        }

        let mut zobrist_enpassant = [0u64; 8];
        for i in 0..8 {
            zobrist_enpassant[i] = rng.r#gen();
        }

        Self {
            zobrist_values,
            zobrist_side_to_move,
            zobrist_castling,
            zobrist_enpassant,
        }
    }
}

// Blocks Memory for Zobrist. init_zobrist is called, when needed
pub static ZOBRIST: Lazy<Zobrist> = Lazy::new(|| Zobrist::init_zobrist());

// castling_rights uses 4 bits: White 0-0 (0b0001), White 0-0-0 (0b0010),
// Black 0-0 (0b0100), Black 0-0-0 (0b1000)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Position {
    pub board: [Cell; BOARD120],
    pub player_to_move: Color,
    pub en_passant_square: Option<Square>,
    pub castling_rights: u8,
    pub zobrist: u64,
    pub half_move_clock: u16,
    pub move_counter: u16,
}

impl Position {
    fn empty() -> Self {
        Self {
            board: Self::init_empty_board(),
            player_to_move: Color::White,
            en_passant_square: None,
            castling_rights: 0,
            zobrist: 0,
            half_move_clock: 0,
            move_counter: 1,
        }
    }

    fn starting_position() -> Self {
        let mut pos = Self::empty();
        pos.board = Self::init_board();
        pos.castling_rights = 0b1111;
        pos.zobrist = Self::compute_zobrist(&pos);
        pos
    }

    fn init_empty_board() -> [Cell; BOARD120] {
        let mut board = [Cell::Offboard; BOARD120];

        for rank in 0..8 {
            let start = 22 + rank * 10;

            for file in 0..8 {
                board[start + file] = Cell::Empty;
            }
        }
        board
    }

    fn init_board() -> [Cell; BOARD120] {
        let mut board = Self::init_empty_board();

        const BACK_RANK: [PieceKind; 8] = [
            PieceKind::Rook,
            PieceKind::Knight,
            PieceKind::Bishop,
            PieceKind::Queen,
            PieceKind::King,
            PieceKind::Bishop,
            PieceKind::Knight,
            PieceKind::Rook,
        ];

        const A1: usize = 22;
        const A2: usize = 32;
        const A7: usize = 82;
        const A8: usize = 92;
        const STRIDE: usize = 8;

        for i in A2..A2 + STRIDE {
            board[i] = Cell::Piece(Piece::new(Color::White, PieceKind::Pawn));
        }
        for (i, kind) in BACK_RANK.iter().enumerate() {
            board[i + A1] = Cell::Piece(Piece::new(Color::White, *kind));
        }
        for i in A7..A7 + STRIDE {
            board[i] = Cell::Piece(Piece::new(Color::Black, PieceKind::Pawn));
        }
        for (i, kind) in BACK_RANK.iter().enumerate() {
            board[i + A8] = Cell::Piece(Piece::new(Color::Black, *kind));
        }
        board
    }

    // Maybe this belongs to movegen
    fn piece_at(&self, square: Square) -> Option<Piece> {
        match self.board[square as usize] {
            Cell::Piece(piece) => Some(piece),
            Cell::Empty => None,
            Cell::Offboard => None,
        }
    }

    // Maybe this belongs to movegen
    fn king_square(&self, color: Color) -> Square {
        for (i, maybe_piece) in self.board.iter().enumerate() {
            if let Cell::Piece(piece) = maybe_piece {
                if piece.color == color && piece.kind == PieceKind::King {
                    return i as u8;
                }
            }
        }
        unreachable!("no king of {:?} found on the board", color);
    }

    // Maybe this belongs to movegen
    fn set_piece(&mut self, piece: Piece, square: Square) {
        self.board[square as usize] = Cell::Piece(piece);
    }

    // computes a hash-value for every Piece on the Board, Player to Move, Castling rights
    // and en-passant-row. So every Boardstate has a unique hash-value
    fn compute_zobrist(&self) -> u64 {
        let mut zobrist: u64 = 0;

        // Hashvalue for every Piece on the Board
        for (sq120, cell) in self.board.iter().enumerate() {
            let sq64 = SQUARE120_TO_SQUARE64[sq120];
            if sq64 < 0 {
                continue;
            }

            if let Cell::Piece(piece) = cell {
                zobrist ^= ZOBRIST.zobrist_values[piece.color.idx() as usize]
                    [piece.kind.idx() as usize][sq64 as usize];
            }
        }

        // Hashvalue for player turn
        if self.player_to_move == Color::Black {
            zobrist ^= ZOBRIST.zobrist_side_to_move
        }

        const CASTLING_POSSIBILITIES: [u8; 4] = [
            0b0001, // White 0-0
            0b0010, // White 0-0-0
            0b0100, // Black 0-0
            0b1000, // Black 0-0-0
        ];

        // Hashvalue for castling possibilities
        for (i, value) in CASTLING_POSSIBILITIES.iter().enumerate() {
            if Self::bitmask(self.castling_rights, *value) {
                zobrist ^= ZOBRIST.zobrist_castling[i];
            }
        }

        // Hashvalue for en passant
        if let Some(ep_sq120) = self.en_passant_square {
            let sq64 = SQUARE120_TO_SQUARE64[ep_sq120 as usize];
            if sq64 >= 0 {
                let file = (sq64 as usize) % 8;
                zobrist ^= ZOBRIST.zobrist_enpassant[file];
            }
        }

        zobrist
    }

    // Helperfunktion that checks if a single bit in mask is also set in to_check
    fn bitmask(to_check: u8, mask: u8) -> bool {
        to_check & mask != 0
    }
}
