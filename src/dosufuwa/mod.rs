mod field;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Undecided,
    Black,
    Empty,
    Balloon,
    Iron,
}