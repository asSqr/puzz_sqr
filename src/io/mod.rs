use std::error;
use std::fmt::{self, Display};
use std::io::{self, BufRead};

use crate::common::{Grid, P};

/// The type for errors occurring in reading puzrs data.
#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    InvalidFormat,
    InvalidValue,
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        match *self {
            ReadError::Io(ref err) => Display::fmt(err, f),
            _ => write!(f, "{}", self.description()),
        }
    }
}

impl error::Error for ReadError {
    fn description(&self) -> &str {
        match *self {
            ReadError::Io(ref err) => err.description(),
            ReadError::InvalidFormat => "invalid format",
            ReadError::InvalidValue => "invalid value",
        }
    }
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> ReadError {
        ReadError::Io(err)
    }
}

fn is_comment(s: &String) -> bool {
    s.chars().next().unwrap() == '%'
}

pub fn next_valid_line(reader: &mut BufRead, buf: &mut String) -> io::Result<usize> {
    loop {
        buf.clear();
        let len = reader.read_line(buf)?;

        if len == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, ""));
        }

        if !buf.trim().is_empty() && !is_comment(buf) {
            return Ok(len);
        }
    }
}

pub fn read_grid<R, F, T>(reader: &mut R, converter: F, default: T) -> Result<Grid<T>, ReadError>
where
    R: BufRead,
    F: Fn(&str) -> Result<T, ReadError>,
    T: Clone,
{
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

    let mut ret = Grid::new(height, width, default);

    for y in 0..height {
        next_valid_line(reader, &mut buffer)?;
        let mut row = buffer.trim_end().split(' ');

        for x in 0..width {
            let elem = row.next().ok_or(ReadError::InvalidFormat)?;
            let converted_elem = converter(elem.as_ref())?;

            ret[P(y, x)] = converted_elem;
        }
    }

    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_grid() {
        let mut src = "
3 4
% this line is a comment!
1 2 3 4
x y z w
p q r s
"
        .as_bytes();
        let grid = read_grid(&mut src, |s| Ok(s.to_string()), String::new()).unwrap();

        assert_eq!(grid.height(), 3);
        assert_eq!(grid.width(), 4);
        assert_eq!(grid[P(1, 2)], "z".to_string());
    }
}