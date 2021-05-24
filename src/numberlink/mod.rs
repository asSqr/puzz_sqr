use std::ops::Index;

mod generator;
mod generator_field;
mod io;
mod solver2;

pub use self::generator::*;
use self::generator_field::*;
pub use self::io::*;
pub use self::solver2::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Clue(pub i32);

pub const NO_CLUE: Clue = Clue(0);
pub const UNUSED: Clue = Clue(-1);

use super::{Grid, D, LP, P};
use crate::common::FOUR_NEIGHBOURS;

#[derive(Clone)]
pub struct LinePlacement {
    right: Grid<bool>,
    down: Grid<bool>,
}

impl LinePlacement {
    pub fn new(height: i32, width: i32) -> LinePlacement {
        LinePlacement {
            right: Grid::new(height, width - 1, false),
            down: Grid::new(height - 1, width, false),
        }
    }
    pub fn height(&self) -> i32 {
        self.right.height()
    }
    pub fn width(&self) -> i32 {
        self.down.width()
    }
    pub fn right(&self, pos: P) -> bool {
        self.right.is_valid_p(pos) && self.right[pos]
    }
    pub fn set_right(&mut self, pos: P, e: bool) {
        self.right[pos] = e;
    }
    pub fn down(&self, pos: P) -> bool {
        self.down.is_valid_p(pos) && self.down[pos]
    }
    pub fn set_down(&mut self, pos: P, e: bool) {
        self.down[pos] = e;
    }
    pub fn get(&self, pos: LP) -> bool {
        let LP(y, x) = pos;
        match (y % 2, x % 2) {
            (0, 1) => self.right(P(y / 2, x / 2)),
            (1, 0) => self.down(P(y / 2, x / 2)),
            _ => panic!(),
        }
    }
    pub fn get_checked(&self, pos: LP) -> bool {
        let LP(y, x) = pos;
        if 0 <= y && y < self.height() * 2 - 1 && 0 <= x && x < self.width() * 2 - 1 {
            self.get(pos)
        } else {
            false
        }
    }
    pub fn isolated(&self, pos: P) -> bool {
        !(self.right(pos + D(0, -1))
            || self.right(pos)
            || self.down(pos + D(-1, 0))
            || self.down(pos))
    }
    pub fn is_endpoint(&self, pos: P) -> bool {
        let mut n_lines = 0;
        let pos_vtx = LP::of_vertex(pos);
        for &d in &FOUR_NEIGHBOURS {
            if self.get_checked(pos_vtx + d) {
                n_lines += 1;
            }
        }
        n_lines == 1
    }
    pub fn extract_chain_groups(&self) -> Option<Grid<i32>> {
        let height = self.height();
        let width = self.width();
        let mut ids = Grid::new(height, width, -1);
        let mut last_id = 0;

        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if self.is_endpoint(pos) && ids[pos] == -1 {
                    // traverse chain
                    let mut l = P(-1, -1);
                    let mut c = pos;

                    'traverse: loop {
                        ids[c] = last_id;
                        for &d in &FOUR_NEIGHBOURS {
                            if c + d != l && self.get_checked(LP::of_vertex(c) + d) {
                                l = c;
                                c = c + d;
                                continue 'traverse;
                            }
                        }
                        break;
                    }
                    last_id += 1;
                }
            }
        }

        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if ids[pos] == -1 {
                    return None;
                }
                if y < height - 1 && (ids[pos] == ids[pos + D(1, 0)]) != self.down(pos) {
                    return None;
                }
                if x < width - 1 && (ids[pos] == ids[pos + D(0, 1)]) != self.right(pos) {
                    return None;
                }
            }
        }

        Some(ids)
    }
}

pub struct AnswerDetail {
    pub answers: Vec<LinePlacement>,
    pub fully_checked: bool,
    pub found_not_fully_filled: bool,
    pub n_steps: u64,
}
impl AnswerDetail {
    pub fn len(&self) -> usize {
        self.answers.len()
    }
}
impl Index<usize> for AnswerDetail {
    type Output = LinePlacement;
    fn index(&self, idx: usize) -> &LinePlacement {
        &self.answers[idx]
    }
}