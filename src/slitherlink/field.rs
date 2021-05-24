use super::super::{Grid, D, LP, P};
use super::*;
use crate::grid_loop::{Edge, GridLoop, GridLoopField};
use crate::common::FOUR_NEIGHBOURS;

#[derive(Clone)]
pub struct Field<'a> {
    grid_loop: GridLoop,
    clue: Grid<Clue>,
    dic: &'a Dictionary,
}
impl<'a> Field<'a> {
    pub fn new(clue: &Grid<Clue>, dic: &'a Dictionary) -> Field<'a> {
        let grid_loop = GridLoop::new(clue.height(), clue.width());

        Field {
            grid_loop: grid_loop,
            clue: clue.clone(),
            dic: dic,
        }
    }
    pub fn height(&self) -> i32 {
        self.clue.height()
    }
    pub fn width(&self) -> i32 {
        self.clue.width()
    }
    pub fn inconsistent(&self) -> bool {
        self.grid_loop.inconsistent()
    }
    pub fn set_inconsistent(&mut self) {
        self.grid_loop.set_inconsistent()
    }
    pub fn fully_solved(&self) -> bool {
        self.grid_loop.fully_solved()
    }
    pub fn check_all_cell(&mut self) {
        let height = self.height();
        let width = self.width();
        let mut handle = GridLoop::get_handle(self);
        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                let clue = handle.get_clue(pos);
                if clue != NO_CLUE {
                    handle.inspect_technique(LP::of_cell(pos));
                    GridLoop::check(&mut *handle, LP::of_cell(pos));
                }
            }
        }
    }
    pub fn get_clue(&self, pos: P) -> Clue {
        self.clue[pos]
    }
    pub fn add_clue(&mut self, pos: P, clue: Clue) {
        if self.clue[pos] != NO_CLUE {
            if self.clue[pos] != clue {
                self.grid_loop.set_inconsistent();
            }
        } else {
            self.clue[pos] = clue;

            let mut handle = GridLoop::get_handle(self);
            handle.inspect_technique(LP::of_cell(pos));
            GridLoop::check(&mut *handle, LP::of_cell(pos));
        }
    }
    pub fn get_edge(&self, pos: LP) -> Edge {
        self.grid_loop.get_edge(pos)
    }
    pub fn get_edge_safe(&self, pos: LP) -> Edge {
        self.grid_loop.get_edge_safe(pos)
    }

    fn inspect_technique(&mut self, pos: LP) {
        if pos.is_cell() {
            let cell_pos = P(pos.0 / 2, pos.1 / 2);
            let clue = self.clue[cell_pos];
            if clue == Clue(0) {
                for &d in &FOUR_NEIGHBOURS {
                    GridLoop::decide_edge(self, pos + d, Edge::Blank);
                }
            }
            if clue == Clue(3) {
                // adjacent 3
                //for d in 0..4 {
                //    let (Y(dy), X(dx)) = neighbor[d];
                for &d in &FOUR_NEIGHBOURS {
                    let cell2 = cell_pos + d;
                    if self.clue.is_valid_p(cell2) && self.clue[cell2] == Clue(3) {
                        // Deriberately ignoring the possible small loop encircling the two 3's
                        GridLoop::decide_edge(self, pos - d, Edge::Line);
                        GridLoop::decide_edge(self, pos + d, Edge::Line);
                        GridLoop::decide_edge(self, pos + d * 3, Edge::Line);
                        GridLoop::decide_edge(
                            self,
                            pos + d + d.rotate_clockwise() * 2,
                            Edge::Blank,
                        );
                        GridLoop::decide_edge(
                            self,
                            pos + d - d.rotate_clockwise() * 2,
                            Edge::Blank,
                        );
                    }
                }

                // diagonal 3
                for &d in &FOUR_NEIGHBOURS {
                    let dr = d.rotate_clockwise();
                    let cell2 = cell_pos + d + dr;
                    if self.clue.is_valid_p(cell2) && self.clue[cell2] == Clue(3) {
                        GridLoop::decide_edge(self, pos - d, Edge::Line);
                        GridLoop::decide_edge(self, pos - dr, Edge::Line);
                        GridLoop::decide_edge(self, pos + d * 2 + dr * 3, Edge::Line);
                        GridLoop::decide_edge(self, pos + d * 3 + dr * 2, Edge::Line);
                    }
                }
            }
        }
    }
}
impl<'a> GridLoopField for Field<'a> {
    fn grid_loop(&mut self) -> &mut GridLoop {
        &mut self.grid_loop
    }
    fn check_neighborhood(&mut self, pos: LP) {
        if pos.0 % 2 == 1 {
            GridLoop::check(self, pos + D(-1, 0));
            GridLoop::check(self, pos + D(1, 0));

            GridLoop::check(self, pos + D(0, -1));
            GridLoop::check(self, pos + D(0, 1));
            GridLoop::check(self, pos + D(-2, -1));
            GridLoop::check(self, pos + D(-2, 1));
            GridLoop::check(self, pos + D(2, -1));
            GridLoop::check(self, pos + D(2, 1));
        } else {
            GridLoop::check(self, pos + D(0, -1));
            GridLoop::check(self, pos + D(0, 1));

            GridLoop::check(self, pos + D(-1, 0));
            GridLoop::check(self, pos + D(1, 0));
            GridLoop::check(self, pos + D(-1, -2));
            GridLoop::check(self, pos + D(1, -2));
            GridLoop::check(self, pos + D(-1, 2));
            GridLoop::check(self, pos + D(1, 2));
        }
    }
    fn inspect(&mut self, pos: LP) {
        if pos.is_cell() {
            let clue = self.clue[pos.as_cell()];
            if clue == NO_CLUE || clue == Clue(0) {
                return;
            }

            let mut neighbors_code = 0;
            let mut pow3 = 1;
            for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                let d = DICTIONARY_EDGE_OFFSET[i];
                neighbors_code += pow3
                    * match self.grid_loop.get_edge_safe(pos + d) {
                        Edge::Undecided => 0,
                        Edge::Line => 1,
                        Edge::Blank => 2,
                    };
                pow3 *= 3;
            }

            let res = self.dic.consult_raw(clue, neighbors_code);
            if res == DICTIONARY_INCONSISTENT {
                self.grid_loop.set_inconsistent();
                return;
            }
            let mut res = res;
            while res != 0 {
                let ix = res.trailing_zeros();
                let i = ix / 2;
                let d = DICTIONARY_EDGE_OFFSET[i as usize];
                GridLoop::decide_edge(
                    self,
                    pos + d,
                    if ix % 2 == 0 { Edge::Line } else { Edge::Blank },
                );
                res ^= 1u32 << ix;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common;

    fn run_problem_test(dic: &Dictionary, input: &[&str], fully_solved: bool) {
        let height = (input.len() / 2) as i32;
        let width = (input[0].len() / 2) as i32;

        let mut clue = Grid::new(height, width, NO_CLUE);
        for y in 0..height {
            let mut row_iter = input[(y * 2 + 1) as usize].chars();

            for x in 0..width {
                row_iter.next();
                let c = row_iter.next().unwrap();
                if '0' <= c && c <= '3' {
                    clue[P(y, x)] = Clue(((c as u8) - ('0' as u8)) as i32);
                }
            }
        }

        let mut field = Field::new(&clue, dic);
        field.check_all_cell();

        assert_eq!(field.inconsistent(), false);
        assert_eq!(field.fully_solved(), fully_solved);

        for y in 0..(input.len() as i32) {
            let mut row_iter = input[y as usize].chars();

            for x in 0..(input[0].len() as i32) {
                let ch = row_iter.next().unwrap();
                let pos = LP(y, x);

                if !pos.is_edge() {
                    continue;
                }

                let expected_edge = match ch {
                    '|' | '-' => Edge::Line,
                    'x' => Edge::Blank,
                    _ => Edge::Undecided,
                };

                assert_eq!(
                    field.get_edge(pos),
                    expected_edge,
                    "Comparing at y={}, x={}",
                    y,
                    x
                );
            }
        }
    }

    #[test]
    fn test_problem() {
        let dic = Dictionary::complete();

        run_problem_test(
            &dic,
            &[
                "+x+-+-+ +",
                "x | x    ",
                "+x+-+x+ +",
                "x0x3|    ",
                "+x+-+x+ +",
                "x | x    ",
                "+x+-+-+ +",
            ],
            false,
        );
        run_problem_test(
            &dic,
            &[
                "+x+-+x+x+",
                "x |3| x x",
                "+x+x+-+-+",
                "x | x x3|",
                "+x+-+x+-+",
                "x0x2| | x",
                "+x+x+-+x+",
            ],
            true,
        );
        run_problem_test(
            &dic,
            &[
                "+-+-+-+-+",
                "|3x x x |",
                "+-+ +-+x+",
                "x     | |",
                "+x+ +x+-+",
                "x x x0x1x",
                "+x+x+x+x+",
            ],
            false,
        );
        run_problem_test(
            &dic,
            &[
                "+ +-+-+x+",
                " 2  x2| x",
                "+ +x+x+-+",
                "| x x0x |",
                "+ + +x+ +",
                "         ",
                "+ + + + +",
            ],
            false,
        );
        run_problem_test(
            &dic,
            &[
                "+ +-+ +x+",
                "   3   1x",
                "+x+-+x+ +",
                "   3  |  ",
                "+ +-+ + +",
                "         ",
                "+ + + + +",
            ],
            false,
        );
        run_problem_test(
            &dic,
            &[
                "+-+-+ + +",
                "|2x      ",
                "+x+-+ + +",
                "| |3     ",
                "+ + + + +",
                "     3| x",
                "+ + +-+x+",
            ],
            false,
        );
    }
}