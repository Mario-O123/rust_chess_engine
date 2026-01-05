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
    DoublePawnPush = 4
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
