// Display only: turns the machine's state into something readable.
// None of this affects how the emulator runs.

use crate::memory::Row;

// Converts an instruction into readable text, e.g. "load row 24".
// The Baby itself never did this.
// Positions 1 to 5 hold the row number. Positions 14 to 16 hold the instruction number.
pub fn as_text(instruction: Row) -> String {
    let which_row = instruction & 0b11111;
    let which_instruction = (instruction >> 13) & 0b111;
    match which_instruction {
        0 => format!("jump row {}", which_row),
        1 => format!("jump forward row {}", which_row),
        2 => format!("load row {}", which_row),
        3 => format!("store row {}", which_row),
        4 | 5 => format!("subtract row {}", which_row),
        // These two don't use a row, so it isn't shown.
        6 => String::from("skip if negative"),
        _ => String::from("stop"),
    }
}
