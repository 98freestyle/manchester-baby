// The Baby's RAM: 32 rows of 32 bits.

// Number of rows in the RAM.
pub const ROWS: usize = 32;

// One row: 32 bits. i32 is a 32 bit number that can also be negative.
pub type Row = i32;

// The whole RAM: 32 rows.
pub type Memory = [Row; ROWS];

// Creates empty RAM, with every bit off.
pub fn empty() -> Memory {
    [0; ROWS]
}
