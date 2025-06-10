use rand::Rng;

/// Rolls a standard 6-sided dice
pub fn roll_dice() -> u8 {
    let mut rng = rand::thread_rng();
    rng.gen_range(1..=6)
}

pub(crate) fn get_global_board_index(player_index: usize, piece_pos: i32) -> Option<u8> {
    if piece_pos == 0 || piece_pos > 51 {
        return None; // at home or in home column, not on main board
    }
    let entry = [0, 13, 26, 39][player_index];
    Some(((entry as i32 + piece_pos - 1) % 52) as u8)
}
