mod dictionary;
mod field;
mod generator;
mod io;

pub use self::dictionary::*;
pub use self::field::*;
pub use self::generator::*;
pub use self::io::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clue(pub i32);
const NO_CLUE: Clue = Clue(-1);