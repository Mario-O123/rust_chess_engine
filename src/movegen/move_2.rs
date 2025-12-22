//Move structure + Flags
pub struct Move_2{
    from: u8,
    to: u8,
    promotion_piece: u8,
    flags: u8,
} //flags for promotion, castle , en passant , 


impl Move_2 {
    pub const none_flag:u8 = 0;
    pub const en_passant_flag:u8 = 1;
    pub const castling_flag:u8 = 2;
    pub const double_move_pawn_flag:u8 = 3;
    
    pub const promotion_none:u8 = 0;
    pub const promotion_knight:u8 = 1;
    pub const promotion_bishop:u8 = 2;
    pub const promotion_rook:u8 = 3;
    pub const promotion_queen:u8 = 4;

    pub fn new(from:u8, to:u8, promotion_piece:u8, flags:u8) -> Self{
        Move_2 {from, to, promotion_piece, flags}
    }


}