use std::io::BufRead;

use super::*;
use crate::common::Grid;
use crate::io::{read_grid, ReadError};

pub fn read_penciloid_problem<T: BufRead>(reader: &mut T) -> Result<Grid<Clue>, ReadError> {
    read_grid(
        reader,
        |token: &str| {
            if token == "." {
                Ok(NO_CLUE)
            } else {
                let n = token.parse::<i32>().map_err(|_| ReadError::InvalidValue)?;
                if n <= 0 {
                    Err(ReadError::InvalidValue)
                } else {
                    Ok(Clue(n))
                }
            }
        },
        NO_CLUE,
    )
}