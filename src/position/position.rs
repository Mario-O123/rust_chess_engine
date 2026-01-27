use super::state::Undo;
pub use crate::board::mailbox120::BOARD_SIZE as BOARD120;
use crate::board::mailbox120::SQUARE120_TO_SQUARE64;
use crate::movegen::Move;
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

    //Attention: works only on legal moves
    pub fn make_move(&mut self, mv: Move) {
        const WK: u8 = 0b0001;
        const WQ: u8 = 0b0010;
        const BK: u8 = 0b0100;
        const BQ: u8 = 0b1000;

        //squares useful for handling castling rights
        const A1: usize = 21;
        const H1: usize = 28;
        const A8: usize = 91;
        const H8: usize = 98;

        let from = mv.from_sq();
        let to = mv.to_sq();

        let moving_piece = match self.board[from] {
            Cell::Piece(p) => p,
            _ => {
                debug_assert!(false, "make_move: from-square has no piece");
                return;
            }
        };

        //never move onto offboard
        debug_assert!(
            self.board[to] != Cell::Offboard,
            "make_move: to-square is offboard"
        );
        //only side-to-move may move
        debug_assert!(
            moving_piece.color == self.player_to_move,
            "make_move: wrong side moved"
        );

        //castling snapshot + old EP hash remove
        let old_castling = self.castling_rights;
        if let Some(old_ep) = self.en_passant_square {
            self.zobrist ^= Self::zob_ep(old_ep);
        }

        //EP square is only valid for the immediate newxt Move
        self.en_passant_square = None;

        //EP square gets deleted in with every turn, only set when there is a DoublePawnPush
        let mut did_capture = false;
        let mut captured_piece: Option<Piece> = None;

        //En passant
        if mv.is_en_passant() {
            //captured square becomes empty
            let captured_sq = if moving_piece.color == Color::White {
                to - 10
            } else {
                to + 10
            };

            //check that an enemy pawn is captured
            debug_assert!(
                matches!(self.board[captured_sq], Cell::Piece(Piece {color, kind: PieceKind::Pawn}) if color != moving_piece.color),
                "en passant: captured piece is not an enemy pawn"
            );

            let captured_pawn = match self.board[captured_sq] {
                Cell::Piece(p) => p,
                _ => {
                    debug_assert!(false);
                    return;
                }
            };

            captured_piece = Some(captured_pawn);
            did_capture = true;

            //incremental zobrist + piece counter
            self.zobrist ^= Self::zob_piece(moving_piece, from);
            self.zobrist ^= Self::zob_piece(captured_pawn, captured_sq);
            self.zobrist ^= Self::zob_piece(moving_piece, to);

            let ci = Self::pc_idx(captured_pawn);
            debug_assert!(self.piece_counter[ci] > 0);
            self.piece_counter[ci] -= 1;

            self.board[captured_sq] = Cell::Empty;
            self.board[from] = Cell::Empty;
            self.board[to] = Cell::Piece(moving_piece);
        }
        //castling
        else if mv.is_castling() {
            //zobrist king move
            self.zobrist ^= Self::zob_piece(moving_piece, from);
            self.zobrist ^= Self::zob_piece(moving_piece, to);

            //king move
            self.board[from] = Cell::Empty;
            self.board[to] = Cell::Piece(moving_piece);

            //rook move with distance +/-2
            let king_shift = to as i32 - from as i32;
            if king_shift == 2 {
                //kingside
                let rook_from = from + 3;
                let rook_to = from + 1;

                let rook_piece = match self.board[rook_from] {
                    Cell::Piece(p) => p,
                    _ => {
                        debug_assert!(false, "castling: rook missing on rook_from");
                        //fallback prohibits a half turn in release build
                        Piece {color: moving_piece.color, kind: PieceKind::Rook}
                    }
                };

                debug_assert!(rook_piece.kind == PieceKind::Rook && rook_piece.color == moving_piece.color, "castling: wrong rook on rook_from");

                self.zobrist ^= Self::zob_piece(rook_piece, rook_from);
                self.zobrist ^= Self::zob_piece(rook_piece, rook_to);

                self.board[rook_to] = self.board[rook_from];
                self.board[rook_from] = Cell::Empty;
            } else if king_shift == -2 {
                //queenside
                let rook_from = from - 4;
                let rook_to = from - 1;

                let rook_piece = match self.board[rook_from] {
                    Cell::Piece(p) => p,
                    _ => {
                        debug_assert!(false, "castling: rook missing on rook_from");
                        Piece {color: moving_piece.color, kind: PieceKind::Rook}
                    }
                };

                debug_assert!(rook_piece.kind == PieceKind::Rook && rook_piece.color == moving_piece.color, "castling: wrong rook on rook_from");

                self.zobrist ^= Self::zob_piece(rook_piece, rook_from);
                self.zobrist ^= Self::zob_piece(rook_piece, rook_to);

                self.board[rook_to] = self.board[rook_from];
                self.board[rook_from] = Cell::Empty;
            }
        }
        //Normal, Promotion, DoublePawnPush
        else {
            //moving piece leaves from
            self.zobrist ^= Self::zob_piece(moving_piece, from);

            if let Cell::Piece(p) = self.board[to] {
                did_capture = true;
                captured_piece = Some(p);

                //remove captured from has + decrease counter
                self.zobrist ^= Self::zob_piece(p, to);
                let captured_idx = Self::pc_idx(p);
                debug_assert!(self.piece_counter[captured_idx] > 0);
                self.piece_counter[captured_idx] -= 1;
            }

            //remove captured piece = normal capture
            self.board[to] = Cell::Empty;
            self.board[from] = Cell::Empty;

            //check if it's a promotion
            if mv.is_promotion() {
                let promo_kind = match mv.promotion_piece() {
                    Some(p) => p.to_piece_kind(),
                    None => {
                        debug_assert!(false, "promotion move must carry promotion piece");
                        return; //or fallback Queen
                    }
                };

                let promoted = Piece {
                    color: moving_piece.color,
                    kind: promo_kind,
                };

                self.board[to] = Cell::Piece(promoted);

                //hash promoted piece + counters pawn promo++
                self.zobrist ^= Self::zob_piece(promoted, to);

                let pawn = Piece {
                    color: moving_piece.color,
                    kind: PieceKind::Pawn,
                };
                let pawn_idx = Self::pc_idx(pawn);
                debug_assert!(self.piece_counter[pawn_idx] > 0);
                self.piece_counter[pawn_idx] -= 1;

                let queen_idx = Self::pc_idx(promoted);
                self.piece_counter[queen_idx] = self.piece_counter[queen_idx].saturating_add(1);
            } else {
                self.board[to] = Cell::Piece(moving_piece);

                //moving piece arrives
                self.zobrist ^= Self::zob_piece(moving_piece, to);
            }

            //if double pawn push, set the EP target
            if mv.is_double_pawn_push() && moving_piece.kind == PieceKind::Pawn {
                let ep_sq = if moving_piece.color == Color::White {
                    from as i32 + 10
                } else {
                    from as i32 - 10
                };
                debug_assert!((0..120).contains(&ep_sq));
                let ep_sq = ep_sq as usize;
                debug_assert!(self.board[ep_sq] != Cell::Offboard);
                debug_assert!(SQUARE120_TO_SQUARE64[ep_sq] >= 0);

                self.en_passant_square = Some(Square::new(ep_sq as u8));
            }

            //update castling rights: if rook moved from starting position, take right away
            if moving_piece.kind == PieceKind::Rook {
                match (moving_piece.color, from) {
                    (Color::White, A1) => self.castling_rights &= !WQ,
                    (Color::White, H1) => self.castling_rights &= !WK,
                    (Color::Black, A8) => self.castling_rights &= !BQ,
                    (Color::Black, H8) => self.castling_rights &= !BK,
                    _ => {}
                }
            }

            //update castling rights: if rook on starting position was captured, take right away
            if let Some(cap) = captured_piece {
                if cap.kind == PieceKind::Rook {
                    match (cap.color, to) {
                        (Color::White, A1) => self.castling_rights &= !WQ,
                        (Color::White, H1) => self.castling_rights &= !WK,
                        (Color::Black, A8) => self.castling_rights &= !BQ,
                        (Color::Black, H8) => self.castling_rights &= !BK,
                        _ => {}
                    }
                }
            }
        }

        //update king cache (relevant for normal and castling)
        //update castling rights: if king moves, take away rights
        if moving_piece.kind == PieceKind::King {
            self.king_sq[moving_piece.color.idx()] = to as u8;

            match moving_piece.color {
                Color::White => self.castling_rights &= !(WK | WQ),
                Color::Black => self.castling_rights &= !(BK | BQ),
            }
        }

        //castling has delta + add new EP hash
        Self::xor_castling_delta(&mut self.zobrist, old_castling, self.castling_rights);
        if let Some(ep) = self.en_passant_square {
            self.zobrist ^= Self::zob_ep(ep);
        }

        //update halfmove clock, fullmove counter
        if moving_piece.kind == PieceKind::Pawn || did_capture {
            self.half_move_clock = 0;
        } else {
            self.half_move_clock = self.half_move_clock.saturating_add(1);
        }

        if self.player_to_move == Color::Black {
            self.move_counter = self.move_counter.saturating_add(1);
        }

        //side to move
        self.player_to_move = self.player_to_move.opposite();
        self.zobrist ^= ZOBRIST.zobrist_side_to_move;

        debug_assert_eq!(self.zobrist, self.compute_zobrist());
        debug_assert_eq!(self.piece_counter, self.compute_piece_counter());
        debug_assert_eq!(self.king_sq, self.compute_king_sq());

        /*removed
        self.piece_counter = self.compute_piece_counter();
        self.zobrist = self.compute_zobrist(); */
    }

    //helpers for make_move

    //we could also use square120_to_square64 but we would have more overhead in the hotpath, look into it
    #[inline]
    fn sq64(sq120: usize) -> usize {
        let s = SQUARE120_TO_SQUARE64[sq120];
        debug_assert!(s >= 0);
        s as usize
    }

    #[inline]
    fn zob_piece(piece: Piece, sq120: usize) -> u64 {
        let s64 = Self::sq64(sq120);
        ZOBRIST.zobrist_values[piece.color.idx()][piece.kind.idx()][s64]
    }

    #[inline]
    fn zob_ep(ep_sq120: Square) -> u64 {
        let s64 = Self::sq64(ep_sq120.as_usize());
        let file = s64 % BOARD_LENGTH;
        ZOBRIST.zobrist_enpassant[file]
    }

    #[inline]
    fn pc_idx(piece: Piece) -> usize {
        piece.kind.idx() + piece.color.idx() * 6
    }

    #[inline]
    fn xor_castling_delta(hash: &mut u64, old: u8, new: u8) {
        const WK: u8 = 0b0001;
        const WQ: u8 = 0b0010;
        const BK: u8 = 0b0100;
        const BQ: u8 = 0b1000;
        let flags = [WK, WQ, BK, BQ];
        for (i, f) in flags.iter().enumerate() {
            if (old ^ new) & f != 0 {
                *hash ^= ZOBRIST.zobrist_castling[i]
            }
        }
    }

    pub fn make_move_with_undo(&mut self, mv: Move) -> Undo {
        let from = mv.from_sq();
        let to = mv.to_sq();

        let moving_piece = match self.board[from] {
            Cell::Piece(p) => p,
            _ => {
                debug_assert!(false, "make_move_with_undo: from-square has no piece");
                return Undo {
                    mv,
                    moving_piece: Piece {
                        color: self.player_to_move,
                        kind: PieceKind::Pawn,
                    },
                    captured: None,
                    captured_sq: None,
                    rook_from: None,
                    rook_to: None,
                    prev_player_to_move: self.player_to_move,
                    prev_ep_sq: self.en_passant_square,
                    prev_castling: self.castling_rights,
                    prev_zobrist: self.zobrist,
                    prev_hm_clock: self.half_move_clock,
                    prev_move_counter: self.move_counter,
                    prev_king_sq: self.king_sq,
                    prev_piece_counter: self.piece_counter,
                };
            }
        };

        //learn capture info before make_move
        let (captured, captured_sq) = if mv.is_en_passant() {
            let cap_sq = if moving_piece.color == Color::White {
                to - 10
            } else {
                to + 10
            };
            let cap = match self.board[cap_sq] {
                Cell::Piece(p) => Some(p),
                _ => None,
            };
            (cap, Some(cap_sq))
        } else {
            match self.board[to] {
                Cell::Piece(p) => (Some(p), None),
                _ => (None, None),
            }
        };

        //castling rook squares, if castling
        let (rook_from, rook_to) = if mv.is_castling() {
            let shift = to as i32 - from as i32;
            if shift == 2 {
                (Some(from + 3), Some(from + 1)) //kingside
            } else if shift == -2 {
                (Some(from - 4), Some(from - 1)) //queenside
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        //snapshot of the old state
        let undo = Undo {
            mv,
            moving_piece,
            captured,
            captured_sq,
            rook_from,
            rook_to,

            prev_player_to_move: self.player_to_move,
            prev_ep_sq: self.en_passant_square,
            prev_castling: self.castling_rights,
            prev_zobrist: self.zobrist,
            prev_hm_clock: self.half_move_clock,
            prev_move_counter: self.move_counter,
            prev_king_sq: self.king_sq,
            prev_piece_counter: self.piece_counter,
        };

        //apply move
        self.make_move(mv);

        undo
    }

    pub fn undo_move(&mut self, undo: Undo) {
        let from = undo.mv.from_sq();
        let to = undo.mv.to_sq();

        //revert board
        if undo.mv.is_castling() {
            //revert king
            self.board[from] = Cell::Piece(undo.moving_piece);
            self.board[to] = Cell::Empty;

            let (rf, rt) = match (undo.rook_from, undo.rook_to) {
                (Some(rf), Some(rt)) => (rf, rt),
                _ => {
                    debug_assert!(false, "undo_move: rook_from/rook_to missing for castling");
                    return;
                }
            };

            //revert rook of rook_to to rook_from
            let rook_piece = match self.board[rt] {
                Cell::Piece(p) => p,
                _ => {
                    debug_assert!(false, "undo_move: rook missing on rook_to");
                    return;
                }
            };

            self.board[rt] = Cell::Empty;
            self.board[rf] = Cell::Piece(rook_piece);
        } else if undo.mv.is_en_passant() {
            //revert pawn
            self.board[from] = Cell::Piece(undo.moving_piece);
            self.board[to] = Cell::Empty;

            let cap_sq = match undo.captured_sq {
                Some(sq) => sq,
                None => {
                    debug_assert!(false, "undo_move: captured_sq missind for en-passant");
                    return;
                }
            };

            let cap = match undo.captured {
                Some(p) => p,
                None => {
                    debug_assert!(false, "undo_move: cpatured piece missing for en-passant");
                    return;
                }
            };

            self.board[cap_sq] = Cell::Piece(cap);
        } else {
            //normal, capture, promotion
            //from gets original moving_piece (pawn in promotion)
            self.board[from] = Cell::Piece(undo.moving_piece);

            //to gets captured or Empty
            self.board[to] = match undo.captured {
                Some(p) => Cell::Piece(p),
                None => Cell::Empty,
            };
        }

        //restore state
        self.player_to_move = undo.prev_player_to_move;
        self.en_passant_square = undo.prev_ep_sq;
        self.castling_rights = undo.prev_castling;
        self.zobrist = undo.prev_zobrist;
        self.half_move_clock = undo.prev_hm_clock;
        self.move_counter = undo.prev_move_counter;
        self.king_sq = undo.prev_king_sq;
        self.piece_counter = undo.prev_piece_counter;

        debug_assert_eq!(self.zobrist, self.compute_zobrist());
        debug_assert_eq!(self.piece_counter, self.compute_piece_counter());
        debug_assert_eq!(self.king_sq, self.compute_king_sq());
    }
}

#[cfg(test)]
pub(super) mod test_util {
    use super::*;

    //helpers for testing

    //turns chess field like e2 into our mailbox120 index
    pub(super) fn sq_str(s: &str) -> usize {
        crate::board::conversion::square120_from_string(s).unwrap()
    }

    //puts a specific piece on a field
    pub(super) fn put(pos: &mut Position, sq: &str, color: Color, kind: PieceKind) {
        pos.board[sq_str(sq)] = Cell::Piece(Piece { color, kind });
    }

    //builds a specified safe test position
    pub(super) fn with_kings(mut pos: Position) -> Position {
        put(&mut pos, "e1", Color::White, PieceKind::King);
        put(&mut pos, "e8", Color::Black, PieceKind::King);

        pos.king_sq = pos.compute_king_sq();
        pos.piece_counter = pos.compute_piece_counter();
        pos.zobrist = pos.compute_zobrist();
        pos
    }
}

#[cfg(test)]
mod make_move_tests {
    use super::test_util::*;
    use super::*;
    use crate::movegen::{Move, PromotionPiece};

    const WK: u8 = 0b0001;
    const WQ: u8 = 0b0010;
    const BK: u8 = 0b0100;
    const BQ: u8 = 0b1000;

    #[test]
    fn normal_move_updates_board_turn_and_halfmove() {
        let mut pos = Position::starting_position();

        let mv = Move::new(sq_str("g1"), sq_str("f3"));
        pos.make_move(mv);

        assert_eq!(pos.board[sq_str("g1")], Cell::Empty);
        assert_eq!(
            pos.board[sq_str("f3")],
            Cell::Piece(Piece {
                color: Color::White,
                kind: PieceKind::Knight
            })
        );

        assert_eq!(pos.player_to_move, Color::Black);
        assert_eq!(pos.en_passant_square, None);
        assert_eq!(pos.half_move_clock, 1);
        assert_eq!(pos.move_counter, 1);
    }

    #[test]
    fn double_pawn_push_sets_en_passant_square() {
        let mut pos = Position::starting_position();

        let mv = Move::new_pawn_double(sq_str("e2"), sq_str("e4"));
        pos.make_move(mv);

        assert_eq!(pos.board[sq_str("e2")], Cell::Empty);
        assert_eq!(
            pos.board[sq_str("e4")],
            Cell::Piece(Piece {
                color: Color::White,
                kind: PieceKind::Pawn
            })
        );

        //EP target it e3
        let e3 = Square::new(sq_str("e3") as u8);
        assert_eq!(pos.en_passant_square, Some(e3));

        assert_eq!(pos.half_move_clock, 0); //pawn move resets
        assert_eq!(pos.player_to_move, Color::Black);
        assert_eq!(pos.move_counter, 1);
    }

    #[test]
    fn en_passant_capture_removes_captured_pawn() {
        //create situation where White can caüture and go to en_passant_square
        let mut pos = with_kings(Position::empty());
        put(&mut pos, "e5", Color::White, PieceKind::Pawn);
        put(&mut pos, "d5", Color::Black, PieceKind::Pawn);
        pos.en_passant_square = Some(Square::new(sq_str("d6") as u8));
        pos.player_to_move = Color::White;

        //refresh derived data
        pos.king_sq = pos.compute_king_sq();
        pos.piece_counter = pos.compute_piece_counter();
        pos.zobrist = pos.compute_zobrist();

        let mv = Move::new_en_passant(sq_str("e5"), sq_str("d6"));
        pos.make_move(mv);

        assert_eq!(pos.board[sq_str("e5")], Cell::Empty);
        assert_eq!(
            pos.board[sq_str("d6")],
            Cell::Piece(Piece {
                color: Color::White,
                kind: PieceKind::Pawn
            })
        );
        assert_eq!(pos.board[sq_str("d5")], Cell::Empty);

        assert_eq!(pos.en_passant_square, None);
        assert_eq!(pos.half_move_clock, 0);
        assert_eq!(pos.player_to_move, Color::Black);
    }

    #[test]
    fn promotion_replaces_pawn_with_promoted_piece() {
        let mut pos = with_kings(Position::empty());

        put(&mut pos, "a7", Color::White, PieceKind::Pawn);
        pos.player_to_move = Color::White;

        pos.king_sq = pos.compute_king_sq();
        pos.piece_counter = pos.compute_piece_counter();
        pos.zobrist = pos.compute_zobrist();

        let mv = Move::new_promotion(sq_str("a7"), sq_str("a8"), PromotionPiece::Queen);
        pos.make_move(mv);

        assert_eq!(pos.board[sq_str("a7")], Cell::Empty);
        assert_eq!(
            pos.board[sq_str("a8")],
            Cell::Piece(Piece {
                color: Color::White,
                kind: PieceKind::Queen
            })
        );

        assert_eq!(pos.half_move_clock, 0);
        assert_eq!(pos.player_to_move, Color::Black);
    }

    #[test]
    fn castling_kingside_moves_rook_and_clears_rights() {
        let mut pos = with_kings(Position::empty());

        //White rook at h1, f1 and g1 are emtpy
        put(&mut pos, "h1", Color::White, PieceKind::Rook);
        pos.castling_rights = WK | WQ;
        pos.player_to_move = Color::White;

        //check if make_move updates rights correctly after the king castles

        let mv = Move::new_castling(sq_str("e1"), sq_str("g1"));
        pos.make_move(mv);

        assert_eq!(pos.board[sq_str("e1")], Cell::Empty);
        assert_eq!(pos.board[sq_str("h1")], Cell::Empty);
        assert_eq!(
            pos.board[sq_str("g1")],
            Cell::Piece(Piece {
                color: Color::White,
                kind: PieceKind::King
            })
        );
        assert_eq!(
            pos.board[sq_str("f1")],
            Cell::Piece(Piece {
                color: Color::White,
                kind: PieceKind::Rook
            })
        );

        assert_eq!(pos.castling_rights & (WK | WQ), 0);
        assert_eq!(pos.king_sq[Color::White.idx()] as usize, sq_str("g1"));
        assert_eq!(pos.player_to_move, Color::Black);
    }

    #[test]
    fn rook_move_clears_correct_castling_rights() {
        let mut pos = with_kings(Position::empty());

        //White rook a1 moves -> should only clear WQ
        put(&mut pos, "a1", Color::White, PieceKind::Rook);
        pos.castling_rights = WK | WQ;
        pos.player_to_move = Color::White;

        pos.king_sq = pos.compute_king_sq();
        pos.piece_counter = pos.compute_piece_counter();
        pos.zobrist = pos.compute_zobrist();

        let mv = Move::new(sq_str("a1"), sq_str("a2"));
        pos.make_move(mv);

        assert_eq!(pos.castling_rights & WQ, 0);
        assert_ne!(pos.castling_rights & WK, 0);
    }
}

#[cfg(test)]
mod undo_tests {
    use super::test_util::*;
    use super::*;
    use crate::movegen::{Move, PromotionPiece};

    const WK: u8 = 0b0001;
    const WQ: u8 = 0b0010;

    #[test]
    fn normal_move_roundtrip_restores_position_and_undo_metadata() {
        let mut pos = Position::starting_position();
        let before = pos.clone();

        let mv = Move::new(sq_str("g1"), sq_str("f3"));
        let undo = pos.make_move_with_undo(mv);

        //make_move_with_undo metadata
        assert_eq!(undo.prev_player_to_move, before.player_to_move);
        assert_eq!(undo.prev_ep_sq, before.en_passant_square);
        assert_eq!(undo.prev_castling, before.castling_rights);
        assert_eq!(undo.prev_zobrist, before.zobrist);
        assert_eq!(undo.prev_piece_counter, before.piece_counter);
        assert_eq!(undo.captured, None);
        assert_eq!(undo.captured_sq, None);
        assert_eq!(undo.rook_from, None);
        assert_eq!(undo.rook_to, None);

        pos.undo_move(undo);

        //undo_move should restore everything
        assert_eq!(pos, before);
    }

    #[test]
    fn capture_roundtrip_restores_position_and_sets_captured_piece() {
        let mut pos = with_kings(Position::empty());
        put(&mut pos, "e4", Color::White, PieceKind::Queen);
        put(&mut pos, "e5", Color::Black, PieceKind::Knight);
        pos.player_to_move = Color::White;

        //refresh
        pos.king_sq = pos.compute_king_sq();
        pos.piece_counter = pos.compute_piece_counter();
        pos.zobrist = pos.compute_zobrist();

        let before = pos.clone();

        let mv = Move::new(sq_str("e4"), sq_str("e5"));
        let undo = pos.make_move_with_undo(mv);

        assert_eq!(
            undo.captured,
            Some(Piece {
                color: Color::Black,
                kind: PieceKind::Knight
            })
        );
        assert_eq!(undo.captured_sq, None);

        pos.undo_move(undo);
        assert_eq!(pos, before);
    }

    #[test]
    fn castling_roundtrip_restores_position_and_sets_rook_squares() {
        let mut pos = with_kings(Position::empty());

        //white kingside castling setup
        put(&mut pos, "h1", Color::White, PieceKind::Rook);
        pos.castling_rights = WK | WQ;
        pos.player_to_move = Color::White;

        //refresh
        pos.king_sq = pos.compute_king_sq();
        pos.piece_counter = pos.compute_piece_counter();
        pos.zobrist = pos.compute_zobrist();

        let before = pos.clone();

        let mv = Move::new_castling(sq_str("e1"), sq_str("g1"));
        let undo = pos.make_move_with_undo(mv);

        // rook squares should be present in Undo
        assert_eq!(undo.rook_from, Some(sq_str("h1")));
        assert_eq!(undo.rook_to, Some(sq_str("f1")));
        assert_eq!(undo.captured, None);
        assert_eq!(undo.captured_sq, None);

        pos.undo_move(undo);
        assert_eq!(pos, before);
    }

    #[test]
    fn promotion_roundtrip_restores_pawn_and_state() {
        let mut pos = with_kings(Position::empty());

        put(&mut pos, "a7", Color::White, PieceKind::Pawn);
        pos.player_to_move = Color::White;

        //refresh
        pos.king_sq = pos.compute_king_sq();
        pos.piece_counter = pos.compute_piece_counter();
        pos.zobrist = pos.compute_zobrist();

        let before = pos.clone();

        let mv = Move::new_promotion(sq_str("a7"), sq_str("a8"), PromotionPiece::Queen);
        let undo = pos.make_move_with_undo(mv);

        //moving_piece should be the original pawn = important for undo of promotions
        assert_eq!(
            undo.moving_piece,
            Piece {
                color: Color::White,
                kind: PieceKind::Pawn
            }
        );

        pos.undo_move(undo);
        assert_eq!(pos, before);
    }
}
