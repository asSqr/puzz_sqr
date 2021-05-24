use std::io::BufRead;

use crate::common::{Grid, P};
use crate::io::{next_valid_line, ReadError};

use super::*;

pub fn read_penciloid_problem<T: BufRead>(reader: &mut T) -> Result<Grid<Clue>, ReadError> {
    let mut buffer = String::new();

    let height;
    let width;

    {
        next_valid_line(reader, &mut buffer)?;
        let mut header = buffer.split(' ');
        height = header
            .next()
            .ok_or(ReadError::InvalidFormat)?
            .trim()
            .parse::<i32>()
            .map_err(|_| ReadError::InvalidValue)?;
        width = header
            .next()
            .ok_or(ReadError::InvalidFormat)?
            .trim()
            .parse::<i32>()
            .map_err(|_| ReadError::InvalidValue)?;
    }

    let mut ret = Grid::new(height, width, NO_CLUE);

    for y in 0..height {
        next_valid_line(reader, &mut buffer)?;
        let mut row_iter = buffer.chars();

        for x in 0..width {
            let c = row_iter.next().ok_or(ReadError::InvalidFormat)?;
            match c {
                '0' | '1' | '2' | '3' => ret[P(y, x)] = Clue((c as u8 - '0' as u8) as i32),
                _ => (),
            }
        }
    }

    Ok(ret)
}