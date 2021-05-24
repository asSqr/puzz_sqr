use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct P(pub i32, pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LP(pub i32, pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct D(pub i32, pub i32);

pub const FOUR_NEIGHBOURS: [D; 4] = [D(-1, 0), D(0, -1), D(1, 0), D(0, 1)];

impl LP {
    pub fn of_cell(pos: P) -> LP {
        LP(pos.0 * 2 + 1, pos.1 * 2 + 1)
    }
    pub fn of_vertex(pos: P) -> LP {
        LP(pos.0 * 2, pos.1 * 2)
    }
    pub fn is_edge(self) -> bool {
        self.0 % 2 != self.1 % 2
    }
    pub fn is_vertex(self) -> bool {
        self.0 % 2 == 0 && self.1 % 2 == 0
    }
    pub fn is_cell(self) -> bool {
        self.0 % 2 == 1 && self.1 % 2 == 1
    }
    pub fn as_vertex(self) -> P {
        P(self.0 / 2, self.1 / 2)
    }
    pub fn as_cell(self) -> P {
        P(self.0 / 2, self.1 / 2)
    }
    pub fn y(self) -> i32 {
        self.0
    }
    pub fn x(self) -> i32 {
        self.1
    }
}
impl P {
    pub fn y(self) -> i32 {
        self.0
    }
    pub fn x(self) -> i32 {
        self.1
    }
}
impl D {
    pub fn rotate_clockwise(self) -> D {
        D(self.1, -self.0)
    }
    pub fn rotate_counterclockwise(self) -> D {
        D(-self.1, self.0)
    }
}
impl Add<D> for P {
    type Output = P;
    fn add(self, rhs: D) -> P {
        P(self.0 + rhs.0, self.1 + rhs.1)
    }
}
impl Sub<D> for P {
    type Output = P;
    fn sub(self, rhs: D) -> P {
        P(self.0 - rhs.0, self.1 - rhs.1)
    }
}
impl Sub<P> for P {
    type Output = D;
    fn sub(self, rhs: P) -> D {
        D(self.0 - rhs.0, self.1 - rhs.1)
    }
}
impl Add<D> for LP {
    type Output = LP;
    fn add(self, rhs: D) -> LP {
        LP(self.0 + rhs.0, self.1 + rhs.1)
    }
}
impl Sub<D> for LP {
    type Output = LP;
    fn sub(self, rhs: D) -> LP {
        LP(self.0 - rhs.0, self.1 - rhs.1)
    }
}
impl Add<D> for D {
    type Output = D;
    fn add(self, rhs: D) -> D {
        D(self.0 + rhs.0, self.1 + rhs.1)
    }
}
impl Sub<D> for D {
    type Output = D;
    fn sub(self, rhs: D) -> D {
        D(self.0 - rhs.0, self.1 - rhs.1)
    }
}
impl Mul<i32> for D {
    type Output = D;
    fn mul(self, rhs: i32) -> D {
        D(self.0 * rhs, self.1 * rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positions() {
        assert_eq!(P(1, 2) + D(3, 0), P(4, 2));
        assert_eq!(P(1, 2) - D(3, 0), P(-2, 2));
        assert_eq!(LP(1, 2) + D(3, 0), LP(4, 2));
        assert_eq!(LP(1, 2) - D(3, 0), LP(-2, 2));
        assert_eq!(D(1, 2) + D(3, 0), D(4, 2));
        assert_eq!(D(1, 2) - D(3, 0), D(-2, 2));
        assert_eq!(D(1, 2) * 4, D(4, 8));

        assert_eq!(D(2, 1).rotate_clockwise(), D(1, -2));
        assert_eq!(D(2, 1).rotate_counterclockwise(), D(-1, 2));

        assert_eq!(LP::of_cell(P(1, 2)), LP(3, 5));
        assert_eq!(LP::of_vertex(P(1, 2)), LP(2, 4));
    }
}