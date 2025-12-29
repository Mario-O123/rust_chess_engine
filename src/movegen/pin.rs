//Pin detection and X-Ray Attacks
//pinned if piece cant move without having own king in check

use crate::position::{Position, Color, PieceKind, Cell};
use crate::board::mailbox120::{
    is_on_board,
    ROOK_DIRECTIONS,
    BISHOP_DIRECTIONS,
};


//Pin representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pin {
    pub pinned_square: usize,
    pub pinner_square: usize,
    pub king_square: usize,
    pub direction: i8,
}

//finds all pins for given color 
//returns vec of all pins
pub fn find_all_pins(position: &Position, color: Color) -> Vec<Pin> {
    let mut pins = Vec::new();

    let king_square = match find_king(position, color) {
        Some(square) => square,
        None =>return pins,
    };

    let enemy_color = color.opposite();

    //checks Rook
    for &direction in &ROOK_DIRECTIONS {
        if let Some(pin) = check_pin_in_direction (
            position,
            king_square,
            direction,
            color,
            enemy_color,
            true
        )  {
            pins.push(pin);
        }       
        
    }

    //checks Bishops
    for &direction in &BISHOP_DIRECTIONS {
        if let Some(pin) = check_pin_in_direction (
            position,
            king_square,
            direction,
            color,
            enemy_color,
            false
        )  {
            pins.push(pin);
        }   
    }

    pins
}

//true if a piece is pinned
pub fn is_piece_pinned(position: &Position, square120: usize, color: Color) -> bool {
    let pins = find_all_pins(position, color);
    pins.into_iter().any(|pin| pin.pinned_square == square120)
}

//returns pin if theres a pinned piece
pub fn get_pin(position: &Position, square120: usize, color: Color) -> Option<Pin> {
    let pins = find_all_pins(position, color);
    pins.into_iter().find(|pin| pin.pinned_square == square120)
}

//checks if move is legal if there is pinned piece
pub fn is_move_legal_if_pinned(pin: &Pin, from: usize, to: usize) -> bool {
    if to == pin.pinned_square {
        return true;
    }

    is_on_pin_line(from, to, pin.king_square, pin.direction)
}

//Helperfunction: checks if square on pin-line
fn is_on_pin_line(from: usize, to: usize, king_square: usize, pin_direction: i8) -> bool {
    // checks if from->to same direction as pin direction
    let diff = (to as i32 - from as i32) as i32;
    if diff == 0 {
        return false;
    }
    
    // Normalize both 
    let pin_dir_norm = normalize_direction(pin_direction);
    let move_dir_norm = normalize_direction(diff);
    
    pin_dir_norm == move_dir_norm || pin_dir_norm == -move_dir_norm
}

//Normalizing direction
fn normalize_direction(dir: i32) -> i8 {
    if dir == 0 {
        return 0;
    }

    let sign = dir.signum();
    let abs = dir.abs();

    //Horizontal +-1
    //Vertical +- 10
    //diagonal +-9 , +-11

    if abs % 1 == 0 && abs < 10 {
        return sign as i8;
    } else if abs % 10 == 0 {
        return (10 * sign) as i8; // Vertikal (normalisiert auf ±1 in Richtung)
    } else if abs % 9 == 0 {
        return (9 * sign) as i8; // Diagonal /
    } else if abs % 11 == 0 {
        return (11 * sign) as i8; // Diagonal \
    }
    
    return 0;
}

//checks if theres a pin i specific direction
fn check_pin_in_direction(
    position: &Position,
    king_square: usize,
    direction: i8,
    friendly_color: Color,
    enemy_color: Color,
    is_rook_direction: bool, // true = Rook/Queen, false = Bishop/Queen
) -> Option<Pin> {
    let mut current = (king_square as i32 + direction as i32) as usize;
    let mut potential_pinned: Option<usize> = None;

    loop {

        if !is_on_board(current) {
            break;
        }

        if let Cell::Piece(piece) = &position.board[current] {
            
            //own piece
            if piece.color == friendly_color {
                if potential_pinned.is_some() {
                    return None;
                }
                potential_pinned = Some(current);
                current = (current as i32 + direction as i32) as usize;
                continue;
            }

            //opp piece
            if piece.color == enemy_color {
                //is it a slider?
                let can_pin = match (is_rook_direction, &piece.kind) {
                    (true, PieceKind::Rook) => true,
                        (true, PieceKind::Queen) => true,
                        (false, PieceKind::Bishop) => true,
                        (false, PieceKind::Queen) => true,
                        _ => false,
                };

                if can_pin && potential_pinned.is_some() {
                    return Some(Pin {
                        pinned_square: potential_pinned.unwrap(),
                        pinner_square: current,
                        king_square,
                        direction,
                    });
                }
                return None;
            }
        }
    current = (current as i32 + direction as i32) as usize;
    }
    None
}

//xray attack = find figure that would put king to check 
//if they werent blocked by other piece
// KING----PAWN----queen -> queen would be a xray attacker
pub fn find_xray_attackers(
    position: &Position,
    target_square: usize,
    by_color: Color,
) -> Vec<usize> {
    let mut xray_attackers = Vec::new();

    //check ROOK
    for &direction in &ROOK_DIRECTIONS {
        if let Some(attacker) = find_xray_in_direction(
            position,
            target_square,
            direction,
            by_color,
            true,
        ) {
            xray_attackers.push(attacker);
        }
    }

    //check BISHOP
    for &direction in &BISHOP_DIRECTIONS {
        if let Some(attacker) = find_xray_in_direction(
            position,
            target_square,
            direction,
            by_color,
            false,
        ) {
            xray_attackers.push(attacker);
        }
    }

    xray_attackers
}

fn find_xray_in_direction(
    position: &Position,
    target_square: usize,
    direction: i8,
    by_color: Color,
    is_rook_direction: bool,
) -> Option<usize> {
    let mut current = (target_square as i32 + direction as i32) as usize;
    let mut blocker_found = false;
    
    loop {
        if !is_on_board(current) {
            break;
        }

        if let Cell::Piece(piece) = position.board[current] {
            
            if !blocker_found {
                //first piece could be blocker
                blocker_found = true;
                current = (current as i32 + direction as i32) as usize;
                continue;
            } else {
                //second piece could be blocker 
                if piece.color == by_color {
                    let can_xray = match (is_rook_direction, &piece.kind) {
                        (true, PieceKind::Rook) => true,
                        (true, PieceKind::Queen) => true,
                        (false, PieceKind::Bishop) => true,
                        (false, PieceKind::Queen) => true,
                        _ => false,
                    };

                    if can_xray {
                        return Some(current);
                    }
                }
                return None;

            }
        }
        current = (current as i32 + direction as i32) as usize;
    }
    None
}



