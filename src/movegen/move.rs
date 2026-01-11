//Move structure + Flags

use crate::position::position::PieceKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub move_type: MoveType,
    pub promotion: Option<PromotionPiece>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromotionPiece {
    Knight,
    Bishop,
    Rook,
    Queen,
}

//Move Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MoveType {
    Normal = 0,    // 0b00 Normal moves
    Promotion = 1, //0b01 Pawn Promotion
    EnPassant = 2, // 0b10 En Passent capture
    Castling = 3,  // 0b11 Castling
    DoublePawnPush = 4,
}

impl PromotionPiece {
    //convert to piecekind
    pub fn to_piece_kind(&self) -> PieceKind {
        match self {
            PromotionPiece::Knight => PieceKind::Knight,
            PromotionPiece::Bishop => PieceKind::Bishop,
            PromotionPiece::Rook => PieceKind::Rook,
            PromotionPiece::Queen => PieceKind::Queen,
        }
    }

    //convert from piecekind (promotion only)
    pub fn from_piece_kind(kind: PieceKind) -> Option<Self> {
        match kind {
            PieceKind::Knight => Some(PromotionPiece::Knight),
            PieceKind::Bishop => Some(PromotionPiece::Bishop),
            PieceKind::Rook => Some(PromotionPiece::Rook),
            PieceKind::Queen => Some(PromotionPiece::Queen),
            _ => None,
        }
    }

    pub fn to_uci_char(self) -> char {
        match self {
            PromotionPiece::Knight => 'n',
            PromotionPiece::Bishop => 'b',
            PromotionPiece::Rook => 'r',
            PromotionPiece::Queen => 'q',
        }
    }
}

impl Move {
    //creates NULL-Move
    pub const NULL: Self = Self {
        from: 0,
        to: 0,
        move_type: MoveType::Normal,
        promotion: None,
    };

    #[inline]
    pub fn is_null(&self) -> bool {
        self.from == 0 && self.to == 0
    }

    #[inline]
    pub fn from_sq(&self) -> usize {
        self.from as usize
    }

    #[inline]
    pub fn to_sq(&self) -> usize {
        self.to as usize
    }

    #[inline]
    pub fn move_type(&self) -> MoveType {
        self.move_type
    }

    //creates normal move
    pub fn new(from: usize, to: usize) -> Self {
        debug_assert!(from < 120 && to < 120, "Out of bounds");

        Self {
            from: from as u8,
            to: to as u8,
            move_type: MoveType::Normal,
            promotion: None,
        }
    }

    //creates pawn promotion
    pub fn new_promotion(from: usize, to: usize, promotion: PromotionPiece) -> Self {
        debug_assert!(from < 120 && to < 120, "Out of bounds");

        Move {
            from: from as u8,
            to: to as u8,
            move_type: MoveType::Promotion,
            promotion: Some(promotion),
        }
    }

    // creates new en passant
    pub fn new_en_passant(from: usize, to: usize) -> Self {
        debug_assert!(from < 120 && to < 120, "Out of bounds");

        Self {
            from: from as u8,
            to: to as u8,
            move_type: MoveType::EnPassant,
            promotion: None,
        }
    }

    pub fn new_pawn_double(from: usize, to: usize) -> Self {
        debug_assert!(from < 120 && to < 120, "Out of bounds");

        Self {
            from: from as u8,
            to: to as u8,
            move_type: MoveType::DoublePawnPush, //new type
            promotion: None,
        }
    }

    pub fn new_castling(from: usize, to: usize) -> Self {
        debug_assert!(from < 120 && to < 120, "Out of bounds");

        Self {
            from: from as u8,
            to: to as u8,
            move_type: MoveType::Castling,
            promotion: None,
        }
    }

    //gets the promotion
    pub fn promotion_piece(&self) -> Option<PromotionPiece> {
        if self.move_type() != MoveType::Promotion {
            return None;
        }

        self.promotion
    }

    // Checks if this is a promotion
    #[inline]
    pub fn is_promotion(&self) -> bool {
        self.move_type() == MoveType::Promotion
    }
    // Checks if this is en passant
    #[inline]
    pub fn is_en_passant(&self) -> bool {
        self.move_type() == MoveType::EnPassant
    }
    // Checks if this is castling
    #[inline]
    pub fn is_castling(&self) -> bool {
        self.move_type() == MoveType::Castling
    }
    #[inline]
    pub fn is_double_pawn_push(&self) -> bool {
        self.move_type == MoveType::DoublePawnPush
    }

    //converst Move to UCI format
    pub fn to_uci(&self) -> String {
        use crate::board::conversion::square120_to_string;

        if self.is_null() {
            return "0000".to_string();
        }

        let mut s = format!(
            "{}{}",
            square120_to_string(self.from_sq()).unwrap(),
            square120_to_string(self.to_sq()).unwrap(),
        );

        if let Some(p) = self.promotion {
            s.push(p.to_uci_char())
        }
        s
    }

    //creates a move from a UCI format
    pub fn from_uci(uci: &str) -> Option<Self> {
        use crate::board::conversion::square120_from_string;

        if uci == "0000" {
            return Some(Move::NULL);
        }

        if uci.len() != 4 && uci.len() != 5 {
            return None;
        }

        let from = square120_from_string(&uci[0..2])?;
        let to = square120_from_string(&uci[2..4])?;

        if uci.len() == 5 {
            let promo = match uci.chars().nth(4)? {
                'n' | 'N' => PromotionPiece::Knight,
                'b' | 'B' => PromotionPiece::Bishop,
                'r' | 'R' => PromotionPiece::Rook,
                'q' | 'Q' => PromotionPiece::Queen,
                _ => return None,
            };

            return Some(Move::new_promotion(from, to, promo));
        }

        Some(Move::new(from, to))
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_uci())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_move_basics() {
        let m = Move::NULL;
        assert!(m.is_null());
        assert_eq!(m.to_uci(), "0000");
        assert_eq!(format!("{}", m), "0000");

        let parsed = Move::from_uci("0000").expect("should parse null move");
        assert_eq!(parsed, Move::NULL);
        assert!(parsed.is_null());
    }

    #[test]
    fn promotion_move_construction_and_helpers() {
        let m = Move::new_promotion(21, 31, PromotionPiece::Queen);
        assert_eq!(m.move_type(), MoveType::Promotion);
        assert!(m.is_promotion());
        assert_eq!(m.promotion_piece(), Some(PromotionPiece::Queen));

        let n = Move::new(21, 31);
        assert_eq!(n.promotion_piece(), None);
    }

    #[test]
    fn en_passant_construction_and_flag() {
        let m = Move::new_en_passant(21, 31);
        assert_eq!(m.move_type(), MoveType::EnPassant);
        assert!(m.is_en_passant());
        assert!(!m.is_promotion());
        assert!(!m.is_castling());
        assert!(!m.is_double_pawn_push());
        assert_eq!(m.promotion_piece(), None);
    }

    #[test]
    fn castling_construction_and_flag() {
        let m = Move::new_castling(21, 31);
        assert_eq!(m.move_type(), MoveType::Castling);
        assert!(m.is_castling());
        assert!(!m.is_promotion());
        assert!(!m.is_en_passant());
        assert!(!m.is_double_pawn_push());
        assert_eq!(m.promotion_piece(), None);
    }

    #[test]
    fn double_pawn_push_construction_and_flag() {
        let m = Move::new_pawn_double(21, 41);
        assert_eq!(m.move_type(), MoveType::DoublePawnPush);
        assert!(m.is_double_pawn_push());
        assert!(!m.is_promotion());
        assert!(!m.is_en_passant());
        assert!(!m.is_castling());
        assert_eq!(m.promotion_piece(), None);
    }

    #[test]
    fn promotion_piece_to_uci_char() {
        assert_eq!(PromotionPiece::Knight.to_uci_char(), 'n');
        assert_eq!(PromotionPiece::Bishop.to_uci_char(), 'b');
        assert_eq!(PromotionPiece::Rook.to_uci_char(), 'r');
        assert_eq!(PromotionPiece::Queen.to_uci_char(), 'q');
    }

    #[test]
    fn from_uci_rejects_bad_lengths() {
        assert_eq!(Move::from_uci(""), None);
        assert_eq!(Move::from_uci("e2e"), None);
        assert_eq!(Move::from_uci("e2e4qq"), None);
    }

    #[test]
    fn from_uci_rejects_bad_promo_char() {
        assert_eq!(Move::from_uci("e7e8x"), None);
        assert_eq!(Move::from_uci("e7e8?"), None);
    }

    #[test]
    fn from_uci_parses_normal_and_promotion_case_insensitive() {
        // Normal
        let m = Move::from_uci("e2e4").expect("should parse normal uci");
        assert_eq!(m.move_type(), MoveType::Normal);
        assert!(!m.is_promotion());

        // Promotion lower
        let p = Move::from_uci("e7e8q").expect("should parse promotion uci");
        assert_eq!(p.move_type(), MoveType::Promotion);
        assert_eq!(p.promotion_piece(), Some(PromotionPiece::Queen));

        // Promotion upper
        let p2 = Move::from_uci("e7e8Q").expect("should parse promotion uci uppercase");
        assert_eq!(p2.move_type(), MoveType::Promotion);
        assert_eq!(p2.promotion_piece(), Some(PromotionPiece::Queen));
    }

    #[test]
    fn uci_roundtrip_normal() {
        let original = "b1c3";
        let m = Move::from_uci(original).unwrap();
        let back = m.to_uci();
        assert_eq!(back, original);
    }

    #[test]
    fn uci_roundtrip_promotion() {
        let original = "a7a8r";
        let m = Move::from_uci(original).unwrap();
        assert!(m.is_promotion());
        assert_eq!(m.promotion_piece(), Some(PromotionPiece::Rook));
        assert_eq!(m.to_uci(), original);
    }
}
