//Move structure + Flags

use crate::position::PieceKind;

/// Bit Layout:
/// - Bits 0-6:   from square (7 bits, supports 0-127, we use 0-119)
/// - Bits 7-13:  to square (7 bits, supports 0-127, we use 0-119)
/// - Bits 14-15: move type (2 bits, 4 different types)
/// 
/// Total: 16 bits = 2 bytes per move
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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


//Bitmasks
const FROM_MASK: u16 = 0b0000_0000_0111_1111;  // Bits 0-6
const TO_MASK: u16 = 0b0011_1111_1000_0000;    // Bits 7-13
const TYPE_MASK: u16 = 0b1100_0000_0000_0000;  // Bits 14-15

const TO_SHIFT: u16 = 7;
const TYPE_SHIFT: u16 = 14;

//Move Types
#[derive(Debug, Clone, Copy, PartialEq, Eq,)]
#[repr(u8)]
pub enum MoveType {
    Normal = 0, // 0b00 Normal moves
    Promotion = 1, //0b01 Pawn Promotion
    EnPassant = 2, // 0b10 En Passent capture
    Castling = 3, // 0b11 Castling 
}

impl MoveType {
    fn from_u8(value:u8) -> Self {
        match value {
            0 => MoveType::Normal,
            1 => MoveType::Promotion,
            2 => MoveType::EnPassant,
            3 => MoveType::Castling,
            _ => unreachable!("Invalid move type: {}", value),
        }
    }
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
}


impl Move {
    //creates NULL-Move
    pub const NULL: Move = Move {data: 0};

    #[inline]
    pub fn from(&self) -> usize {
        (self.data & FROM_MASK) as usize
    }

    //creates normal move
    pub fn new(from: usize, to: usize) -> Self {
        debug_assert!(from < 120 && from > 0, "from square out of bounds");
        debug_assert!(to < 120 && to > 0, "to square out of bounds");

        Move {
            from: from as u8,
            to: to as u8,
            move_type: MoveType::Normal,
            promotion: None,
        }
    }
    
    //creates pawn promotion
    pub fn new_promotion(from: usize, to: usize, promotion: PromotionPiece) -> Self {
        debug_assert!(from < 120 && from > 0, "from square out of bounds");
        debug_assert!(to < 120 && to > 0, "to square out of bounds");
        
        Move {
            from: from as u8,
            to: to as u8,
            move_type: MoveType::Promotion,
            promotion: Some(p),
        }
    }
    
    // creates new en passant 
    pub fn new_en_passant(from: usize, to: usize) -> Self {
        debug_assert!(from < 120 && from > 0, "from square out of bounds");
        debug_assert!(to < 120 && to > 0, "to square out of bounds");
        
        let data = (from as u16)
            | ((to as u16) << TO_SHIFT)
            | ((MoveType::EnPassant as u16) << TYPE_SHIFT);
        
        Move { data }
    }
    //creates new castling move
    pub fn new_castling(from: usize, to: usize) ->  Self {
        debug_assert!(from < 120, "from square out of bounds");
        debug_assert!(to < 120, "to square out of bounds");
        
        let data = (from as u16)
            | ((to as u16) << TO_SHIFT)
            | ((MoveType::Castling as u16) << TYPE_SHIFT);
        
        Move { data }
    }
    
    
    //gets the from square120
    #[inline]
    pub fn to(&self) -> usize {
        ((self.data & TO_MASK) >> TO_SHIFT) as usize & 0b0111_11111
    }
    //gets the move type
    #[inline]
    pub fn move_type(&self) -> MoveType {
        let type_bits = ((self.data & TYPE_MASK) >> TYPE_SHIFT) as u8;
        MoveType::from_u8(type_bits)
    }
    //gets the promotion
    pub fn promotion_piece(&self) -> Option<PromotionPiece> {
        if self.move_type() != MoveType::Promotion {
            return None;
        }

        self.promotion
    }

    //checks if move is null
    #[inline]
    pub fn is_null(&self) -> bool {
        self.data == 0
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

    //converst Move to UCI format
    pub fn to_uci(&self) -> String {
        use crate::board::conversion::square120_to_string;

        if self.is_null() {
            return "0000".to_string();
        }

        let from_str = square120_to_string(self.from())
            .expect("valid from square");
        let to_str = square120_to_string(self.to())
            .expect("valid to square");

        format!("{}{}", from_str, to_str)
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


