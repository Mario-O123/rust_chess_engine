use crate::board::mailbox120::BOARD_SIZE as BOARD120;
use crate::board::mailbox120::SQUARE120_TO_SQUARE64;
use once_cell::sync::Lazy;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

const BOARD64: usize = 64;
const BOARD_LENGTH: usize = 8;

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

    pub fn idx(&self) -> usize {
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
    pub fn idx(self) -> usize {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Square(u8);

impl Square {
    pub fn new(square: u8) -> Self {
        debug_assert!(square < 120, "Only 0-119 are valid values for Square");
        Self(square)
    }

    pub const fn get(self) -> u8 {
        self.0
    }

    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

// zobrist_values: [[[Squares]; Piecekinds]; Colors]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Zobrist {
    pub zobrist_values: [[[u64; BOARD64]; 6]; 2],
    pub zobrist_side_to_move: u64,
    pub zobrist_castling: [u64; 4],
    pub zobrist_enpassant: [u64; BOARD_LENGTH],
}

impl Zobrist {
    // Same Seed generates same random u64 Numbers everytime function is called
    pub fn init_zobrist() -> Self {
        const SEED: u64 = 42;
        let mut rng = StdRng::seed_from_u64(SEED);
        let mut zobrist_values = [[[0u64; BOARD64]; 6]; 2];

        for colors in zobrist_values.iter_mut() {
            for piecekinds in colors.iter_mut() {
                for square in piecekinds.iter_mut() {
                    *square = rng.r#gen();
                }
            }
        }

        let zobrist_side_to_move = rng.r#gen();

        let mut zobrist_castling = [0u64; 4];
        for castling_possibility in zobrist_castling.iter_mut() {
            *castling_possibility = rng.r#gen();
        }

        let mut zobrist_enpassant = [0u64; BOARD_LENGTH];
        for ep_possibility in zobrist_enpassant.iter_mut() {
            *ep_possibility = rng.r#gen();
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
pub static ZOBRIST: Lazy<Zobrist> = Lazy::new(Zobrist::init_zobrist);

// castling_rights uses 4 bits: White 0-0 (0b0001), White 0-0-0 (0b0010),
// Black 0-0 (0b0100), Black 0-0-0 (0b1000)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Position {
    pub board: [Cell; BOARD120],
    pub player_to_move: Color,
    pub en_passant_square: Option<Square>,
    pub castling_rights: u8,
    pub zobrist: u64,
    pub half_move_clock: u16,
    pub move_counter: u16,
    pub king_sq: [u8; 2],
    pub piece_counter: [u8; 12],
}

impl Position {
    pub fn empty() -> Self {
        Self {
            board: Self::init_empty_board(),
            player_to_move: Color::White,
            en_passant_square: None,
            castling_rights: 0,
            zobrist: 0,
            half_move_clock: 0,
            move_counter: 1,
            king_sq: [0; 2],
            piece_counter: [0; 12],
        }
    }

    pub fn starting_position() -> Self {
        let mut pos = Self::empty();
        pos.board = Self::init_board();
        pos.castling_rights = 0b1111;
        pos.zobrist = pos.compute_zobrist();
        pos.king_sq = pos.compute_king_sq();
        pos.piece_counter = pos.compute_piece_counter();

        pos
    }

    fn init_empty_board() -> [Cell; BOARD120] {
        let mut board = [Cell::Offboard; BOARD120];
        const A1: usize = 21;

        for rank in 0..BOARD_LENGTH {
            let start = A1 + rank * 10;

            for file in 0..BOARD_LENGTH {
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

        const A1: usize = 21;
        const A2: usize = 31;
        const A7: usize = 81;
        const A8: usize = 91;

        for square in board[A2..A2 + BOARD_LENGTH].iter_mut() {
            *square = Cell::Piece(Piece::new(Color::White, PieceKind::Pawn));
        }

        for (i, kind) in BACK_RANK.iter().enumerate() {
            board[i + A1] = Cell::Piece(Piece::new(Color::White, *kind));
        }

        for square in board[A7..A7 + BOARD_LENGTH].iter_mut() {
            *square = Cell::Piece(Piece::new(Color::Black, PieceKind::Pawn));
        }

        for (i, kind) in BACK_RANK.iter().enumerate() {
            board[i + A8] = Cell::Piece(Piece::new(Color::Black, *kind));
        }
        board
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        match self.board[square.as_usize()] {
            Cell::Piece(piece) => Some(piece),
            Cell::Empty => None,
            Cell::Offboard => None,
        }
    }

    // returns a vec with squares of all pieces with same color and Piecekind
    // on the board
    pub fn find_pieces(&self, color: Color, kind: PieceKind) -> Vec<Square> {
        let mut pieces_found: Vec<Square> = Vec::new();
        for (i, maybe_piece) in self.board.iter().enumerate() {
            if let Cell::Piece(piece) = maybe_piece
                && piece.color == color
                && piece.kind == kind
            {
                pieces_found.push(Square::new(i as u8))
            }
        }
        pieces_found
    }

    pub fn find_single_piece(&self, color: Color, kind: PieceKind) -> Option<Square> {
        for (i, maybe_piece) in self.board.iter().enumerate() {
            if let Cell::Piece(piece) = maybe_piece
                && piece.color == color
                && piece.kind == kind
            {
                return Some(Square::new(i as u8));
            }
        }
        None
    }

    pub fn compute_king_sq(&self) -> [u8; 2] {
        // checks if there is exactly 1 black and 1 white king on the board
        debug_assert!(self.find_pieces(Color::White, PieceKind::King).len() == 1);
        debug_assert!(self.find_pieces(Color::Black, PieceKind::King).len() == 1);

        let white = self
            .find_single_piece(Color::White, PieceKind::King)
            .expect("White king is missing");

        let black = self
            .find_single_piece(Color::Black, PieceKind::King)
            .expect("Black king is missing");

        [white.get(), black.get()]
    }

    pub fn compute_piece_counter(&self) -> [u8; 12] {
        let mut all_pieces: [u8; 12] = [0; 12];

        for cell in self.board.iter() {
            if let Cell::Piece(piece) = cell {
                if piece.color == Color::White {
                    all_pieces[piece.kind.idx()] += 1;
                } else if piece.color == Color::Black {
                    all_pieces[piece.kind.idx() + 6] += 1;
                }
            }
        }
        all_pieces
    }

    // computes a hash-value for every Piece on the Board, Player to Move, Castling rights
    // and en-passant-row. So every Boardstate has a unique hash-value
    pub fn compute_zobrist(&self) -> u64 {
        let mut zobrist: u64 = 0;

        // Hashvalue for every Piece on the Board
        for (sq120, cell) in self.board.iter().enumerate() {
            let sq64 = SQUARE120_TO_SQUARE64[sq120];
            if sq64 < 0 {
                continue;
            }

            if let Cell::Piece(piece) = cell {
                zobrist ^=
                    ZOBRIST.zobrist_values[piece.color.idx()][piece.kind.idx()][sq64 as usize];
            }
        }

        // Hashvalue for player turn. If it´s white´s turn the hashvalue is 0.
        if self.player_to_move == Color::Black {
            zobrist ^= ZOBRIST.zobrist_side_to_move;
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
            let sq64 = SQUARE120_TO_SQUARE64[ep_sq120.as_usize()];
            if sq64 >= 0 {
                let file = (sq64 as usize) % BOARD_LENGTH;
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
