


fn legal_move_filter() -> Vec<Move> {

check_for_pins();

king_in_check_filter();

move_in_check();

castling_prohibited()

}