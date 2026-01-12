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

        //EP square is only valid for the immediate newxt Move
        self.en_passant_square = None;

        //EP square gets deleted in with every turn, only set when there is a DoublePawnPush
        let mut did_capture = false;
        let mut captured_piece: Option<Piece> = None;

        //En passant
        if mv.is_en_passant() {
            //captured square becomes empty
            let captured_sq = if moving_piece.color == Color::White {
                debug_assert!(
                    to >= 10,
                    "en passant: to too small for white capture square"
                );
                to - 10
            } else {
                debug_assert!(
                    to + 10 < 120,
                    "en passant: to too large for black capture square"
                );
                to + 10
            };

            //check that an enemy pawn is captured
            debug_assert!(
                matches!(self.board[captured_sq], Cell::Piece(Piece {color, kind: PieceKind::Pawn}) if color != moving_piece.color),
                "en passant: captured piece is not an enemy pawn"
            );

            if let Cell::Piece(p) = self.board[captured_sq] {
                captured_piece = Some(p);
            }
            self.board[captured_sq] = Cell::Empty;
            did_capture = true;

            self.board[from] = Cell::Empty;
            self.board[to] = Cell::Piece(moving_piece);
        }
        //castling
        else if mv.is_castling() {
            //king move
            self.board[from] = Cell::Empty;
            self.board[to] = Cell::Piece(moving_piece);

            //rook move with distance +/-2
            let king_shift = to as i32 - from as i32;
            if king_shift == 2 {
                //kingside
                let rook_from = from + 3;
                let rook_to = from + 1;
                self.board[rook_to] = self.board[rook_from];
                self.board[rook_from] = Cell::Empty;
            } else if king_shift == -2 {
                //queenside
                let rook_from = from - 4;
                let rook_to = from - 1;
                self.board[rook_to] = self.board[rook_from];
                self.board[rook_from] = Cell::Empty;
            }
        }
        //Normal, Promotion, DoublePawnPush
        else {
            if let Cell::Piece(p) = self.board[to] {
                did_capture = true;
                captured_piece = Some(p);
            }

            //remove captured piece = normal capture
            self.board[to] = Cell::Empty;
            self.board[from] = Cell::Empty;

            //check if it's a promotion
            if mv.is_promotion() {
                let promo_kind = match mv.promotion_piece() {
                    Some(p) => p.to_piece_kind(),
                    None => panic!("promotion move must carry promotion piece"), //or Queen as fallback maybe?
                };

                self.board[to] = Cell::Piece(Piece {
                    color: moving_piece.color,
                    kind: promo_kind,
                });
            } else {
                self.board[to] = Cell::Piece((moving_piece));
            }

            //if double pawn push, set the EP target
            if mv.is_double_pawn_push() && moving_piece.kind == PieceKind::Pawn {
                let ep_sq = if moving_piece.color == Color::White {
                    from as i32 + 10
                } else {
                    from as i32 - 10
                };
                debug_assert!((0..120).contains(&ep_sq));
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

        self.piece_counter = self.compute_piece_counter();
        self.zobrist = self.compute_zobrist();
    }
}

#[cfg(test)]
mod make_move_tests {
    use super::*;
    use crate::movegen::{Move, PromotionPiece};

    const WK: u8 = 0b0001;
    const WQ: u8 = 0b0010;
    const BK: u8 = 0b0100;
    const BQ: u8 = 0b1000;

    //helpers for testing

    //turns chess field like e2 into our mailbox120 index
    fn sq_str(s: &str) -> usize {
        crate::board::conversion::square120_from_string(s).unwrap()
    }

    //puts a specific piece on a field
    fn put(pos: &mut Position, sq: &str, color: Color, kind: PieceKind) {
        pos.board[sq_str(sq)] = Cell::Piece(Piece { color, kind });
    }

    //builds a specified safe test position
    fn with_kings(mut pos: Position) -> Position {
        put(&mut pos, "e1", Color::White, PieceKind::King);
        put(&mut pos, "e8", Color::Black, PieceKind::King);

        pos.king_sq = pos.compute_king_sq();
        pos.piece_counter = pos.compute_piece_counter();
        pos.zobrist = pos.compute_zobrist();
        pos
    }

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
