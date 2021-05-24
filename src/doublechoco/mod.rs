mod field;

pub use self::field::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Color {
    Black,
    White,
}
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Border {
    Undecided,
    Line,
    Blank,
}
pub type Clue = i32;
pub const NO_CLUE: Clue = 0;