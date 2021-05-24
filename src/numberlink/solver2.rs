use super::super::{Grid, D, LP, P};
use super::*;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Edge {
    Undecided,
    Line,
    Blank,
}

enum History {
    AnotherEnd(i32, i32),
    Edge(LP),
    Inconsistent(bool),
    OpenEndCount(i32, i32),
    NumberEnd(i32, (i32, i32)),
    Checkpoint,
}

struct SolverField {
    another_end: Grid<i32>,        // height * width
    has_clue: Grid<bool>,          // height * width
    unused: Grid<bool>,            // height * width
    down_left: Grid<bool>,         // height * width
    down_right: Grid<bool>,        // height * width
    left_clue_distance: Grid<i32>, // height * width
    edge: Grid<Edge>,              // (2 * height - 1) * (2 * width - 1)
    inconsistent: bool,
    disallow_unused_cell: bool,
    history: Vec<History>,

    // for cut-based pruning
    undecided_count: Vec<i32>,   // width - 1
    open_end_count: Vec<i32>,    // width
    number_end: Vec<(i32, i32)>, // max clue
}

const CLOSED_END: i32 = -1;

impl SolverField {
    fn new(problem: &Grid<Clue>, disallow_unused_cell: bool) -> SolverField {
        let height = problem.height();
        let width = problem.width();
        let mut another_end = Grid::new(height, width, 0);
        let mut edge = Grid::new(height * 2 - 1, width * 2 - 1, Edge::Undecided);
        let mut has_clue = Grid::new(height, width, false);
        let mut unused = Grid::new(height, width, false);
        let mut max_clue = 0;
        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                let c = problem[pos];
                if c == UNUSED {
                    has_clue[pos] = true;
                    unused[pos] = true;

                    if y > 0 {
                        edge[LP::of_vertex(pos) + D(-1, 0)] = Edge::Blank;
                    }
                    if x > 0 {
                        edge[LP::of_vertex(pos) + D(0, -1)] = Edge::Blank;
                    }
                    if y < height - 1 {
                        edge[LP::of_vertex(pos) + D(1, 0)] = Edge::Blank;
                    }
                    if x < width - 1 {
                        edge[LP::of_vertex(pos) + D(0, 1)] = Edge::Blank;
                    }
                } else if c == NO_CLUE {
                    let id = another_end.index_p(pos) as i32;
                    another_end[pos] = id;
                } else {
                    max_clue = ::std::cmp::max(max_clue, c.0);
                    another_end[pos] = -(c.0 + 1);
                    has_clue[pos] = true;
                }
            }
        }
        let mut down_left = Grid::new(height, width, false);
        let mut down_right = Grid::new(height, width, false);
        for y in 0..height {
            let y = height - 1 - y;
            for x in 0..width {
                if y != height - 1 {
                    let pos = P(y, x);
                    if x > 0 && (down_left[pos + D(1, -1)] || has_clue[pos + D(1, -1)]) {
                        down_left[pos] = true;
                    }
                    if x < width - 1 && (down_right[pos + D(1, 1)] || has_clue[pos + D(1, 1)]) {
                        down_right[pos] = true;
                    }
                }
            }
        }
        let mut left_clue_distance = Grid::new(height, width, 0);
        for y in 0..height {
            let mut d = width;
            for x in 0..width {
                d += 1;
                let pos = P(y, x);
                left_clue_distance[pos] = d;
                if has_clue[pos] {
                    d = 0;
                }
            }
        }
        let undecided_count = vec![height; (width - 1) as usize];
        let open_end_count = vec![0; width as usize];
        let mut number_end = vec![(-1, -1); (max_clue + 1) as usize];
        for y in 0..height {
            for x in 0..width {
                let Clue(c) = problem[P(y, x)];
                if c > 0 {
                    let c = c as usize;
                    if number_end[c].0 == -1 {
                        number_end[c].0 = x;
                    } else {
                        number_end[c].1 = x;
                    }
                }
            }
        }
        let mut ret = SolverField {
            another_end,
            has_clue,
            unused,
            down_left,
            down_right,
            left_clue_distance,
            edge,
            inconsistent: false,
            disallow_unused_cell,
            history: Vec::new(),
            undecided_count,
            open_end_count,
            number_end,
        };
        if disallow_unused_cell {
            for y in 0..height {
                for x in 0..width {
                    ret.inspect(P(y, x));
                }
            }
        }
        ret
    }
    fn get_edge(&self, pos: LP) -> Edge {
        if self.edge.is_valid_lp(pos) {
            self.edge[pos]
        } else {
            Edge::Blank
        }
    }
    fn height(&self) -> i32 {
        self.another_end.height()
    }
    fn width(&self) -> i32 {
        self.another_end.width()
    }
    fn get_line_placement(&self) -> LinePlacement {
        let height = self.height();
        let width = self.width();
        let mut ret = LinePlacement::new(height, width);
        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if y != height - 1 && self.get_edge(LP::of_vertex(pos) + D(1, 0)) == Edge::Line {
                    ret.set_down(pos, true);
                }
                if x != width - 1 && self.get_edge(LP::of_vertex(pos) + D(0, 1)) == Edge::Line {
                    ret.set_right(pos, true);
                }
            }
        }
        ret
    }

    fn set_inconsistent(&mut self) -> bool {
        self.history.push(History::Inconsistent(self.inconsistent));
        self.inconsistent = true;
        return true;
    }
    fn update_another_end(&mut self, id: i32, value: i32) {
        self.history
            .push(History::AnotherEnd(id, self.another_end[id as usize]));
        self.another_end[id as usize] = value;
    }
    fn update_open_end_count(&mut self, x1: i32, x2: i32, sgn: i32) {
        if x1 < x2 {
            self.open_end_count[x1 as usize] += sgn;
            self.open_end_count[x2 as usize] -= sgn;
            self.history.push(History::OpenEndCount(x1, -sgn));
            self.history.push(History::OpenEndCount(x2, sgn));
        } else if x1 > x2 {
            self.open_end_count[x2 as usize] += sgn;
            self.open_end_count[x1 as usize] -= sgn;
            self.history.push(History::OpenEndCount(x2, -sgn));
            self.history.push(History::OpenEndCount(x1, sgn));
        }
    }
    fn update_number_end(&mut self, n: i32, before: i32, after: i32) {
        self.history
            .push(History::NumberEnd(n, self.number_end[n as usize]));
        let n = n as usize;
        if self.number_end[n].0 == before {
            self.number_end[n].0 = after;
        } else {
            self.number_end[n].1 = after;
        }
    }
    fn close_number_end(&mut self, n: i32) {
        self.history
            .push(History::NumberEnd(n, self.number_end[n as usize]));
        self.number_end[n as usize] = (-1, -1);
    }
    /// Add an checkpoint.
    fn add_checkpoint(&mut self) {
        self.history.push(History::Checkpoint);
    }
    /// Rollback until the last checkpoint.
    fn rollback(&mut self) {
        while let Some(entry) = self.history.pop() {
            match entry {
                History::AnotherEnd(id, val) => self.another_end[id as usize] = val,
                History::Edge(cd) => {
                    self.edge[cd] = Edge::Undecided;
                    let LP(_, x) = cd;
                    if x % 2 == 1 {
                        self.undecided_count[(x / 2) as usize] += 1;
                    }
                }
                History::Inconsistent(ic) => self.inconsistent = ic,
                History::OpenEndCount(x, app) => self.open_end_count[x as usize] += app,
                History::NumberEnd(n, v) => self.number_end[n as usize] = v,
                History::Checkpoint => break,
            }
        }
    }
    /// Decide edge `cd`.
    /// `cd` must be in universal-coordination.
    fn decide_edge(&mut self, pos: LP, state: Edge) -> bool {
        let current_state = self.get_edge(pos);
        if current_state != Edge::Undecided {
            if current_state != state {
                return self.set_inconsistent();
            }
            return false;
        }

        let LP(y, x) = pos;

        // update endpoints or detect inconsistency
        let end1;
        let end2;
        if y % 2 == 0 {
            end1 = P(y / 2, x / 2);
            end2 = P(y / 2, x / 2 + 1);
        } else {
            end1 = P(y / 2, x / 2);
            end2 = P(y / 2 + 1, x / 2);
        }
        let end1_id = self.another_end.index_p(end1) as i32;
        let end2_id = self.another_end.index_p(end2) as i32;

        if state == Edge::Line {
            let another_end1_id = self.another_end[end1];
            let another_end2_id = self.another_end[end2];

            // connecting closed ends / closing single chain
            if another_end1_id == CLOSED_END
                || another_end2_id == CLOSED_END
                || another_end1_id == end2_id
            {
                return self.set_inconsistent();
            }
            match (another_end1_id < 0, another_end2_id < 0) {
                (true, true) => {
                    if another_end1_id == another_end2_id {
                        self.close_number_end(-another_end1_id - 1);
                        self.update_another_end(end1_id, CLOSED_END);
                        self.update_another_end(end2_id, CLOSED_END);
                    } else {
                        return self.set_inconsistent();
                    }
                }
                (false, true) => {
                    let ae1_x = self.another_end.p(another_end1_id as usize).x();
                    self.update_open_end_count(ae1_x, end1.1, -1);
                    self.update_number_end(-another_end2_id - 1, end2.1, ae1_x);
                    if end1_id != another_end1_id {
                        self.update_another_end(end1_id, CLOSED_END);
                    }
                    self.update_another_end(another_end1_id, another_end2_id);
                    self.update_another_end(end2_id, CLOSED_END);
                }
                (true, false) => {
                    let ae2_x = self.another_end.p(another_end2_id as usize).x();
                    self.update_open_end_count(ae2_x, end2.1, -1);
                    self.update_number_end(-another_end1_id - 1, end1.1, ae2_x);
                    if end2_id != another_end2_id {
                        self.update_another_end(end2_id, CLOSED_END);
                    }
                    self.update_another_end(another_end2_id, another_end1_id);
                    self.update_another_end(end1_id, CLOSED_END);
                }
                (false, false) => {
                    let ae1_x = self.another_end.p(another_end1_id as usize).x();
                    let ae2_x = self.another_end.p(another_end2_id as usize).x();
                    self.update_open_end_count(ae1_x, end1.1, -1);
                    self.update_open_end_count(ae2_x, end2.1, -1);
                    self.update_open_end_count(ae1_x, ae2_x, 1);
                    if end1_id != another_end1_id {
                        self.update_another_end(end1_id, CLOSED_END);
                    }
                    self.update_another_end(another_end1_id, another_end2_id);
                    if end2_id != another_end2_id {
                        self.update_another_end(end2_id, CLOSED_END);
                    }
                    self.update_another_end(another_end2_id, another_end1_id);
                }
            }
        }

        // update edge state
        self.history.push(History::Edge(pos));
        self.edge[pos] = state;
        if x % 2 == 1 {
            self.undecided_count[(x / 2) as usize] -= 1;
        }

        // ensure canonical form
        if state == Edge::Line {
            if y % 2 == 0 {
                if !self.down_right[P(y / 2, x / 2)] && self.get_edge(pos + D(1, -1)) == Edge::Line
                {
                    return self.set_inconsistent();
                }
                if !self.down_left[P(y / 2, x / 2 + 1)]
                    && self.get_edge(pos + D(1, 1)) == Edge::Line
                {
                    return self.set_inconsistent();
                }

                if self.get_edge(pos + D(-2, 0)) == Edge::Line {
                    if self.decide_edge(pos + D(-1, -1), Edge::Blank) {
                        return true;
                    }
                    if self.decide_edge(pos + D(-1, 1), Edge::Blank) {
                        return true;
                    }
                } else if self.get_edge(pos + D(-1, -1)) == Edge::Line {
                    if self.decide_edge(pos + D(-2, 0), Edge::Blank) {
                        return true;
                    }
                    if self.decide_edge(pos + D(-1, 1), Edge::Blank) {
                        return true;
                    }
                } else if self.get_edge(pos + D(-1, 1)) == Edge::Line {
                    if self.decide_edge(pos + D(-2, 0), Edge::Blank) {
                        return true;
                    }
                    if self.decide_edge(pos + D(-1, -1), Edge::Blank) {
                        return true;
                    }
                }

                if self.get_edge(pos + D(2, 0)) == Edge::Line {
                    if self.decide_edge(pos + D(1, -1), Edge::Blank) {
                        return true;
                    }
                    if self.decide_edge(pos + D(1, 1), Edge::Blank) {
                        return true;
                    }
                } else if self.get_edge(pos + D(1, -1)) == Edge::Line {
                    if self.decide_edge(pos + D(2, 0), Edge::Blank) {
                        return true;
                    }
                    if self.decide_edge(pos + D(1, 1), Edge::Blank) {
                        return true;
                    }

                    // yielding L-chain
                    if !self.has_clue[P(y / 2 + 1, x / 2 + 1)] {
                        if self.decide_edge(pos + D(2, 2), Edge::Line) {
                            return true;
                        }
                        if self.decide_edge(pos + D(3, 1), Edge::Line) {
                            return true;
                        }
                    }
                } else if self.get_edge(pos + D(1, 1)) == Edge::Line {
                    if self.decide_edge(pos + D(2, 0), Edge::Blank) {
                        return true;
                    }
                    if self.decide_edge(pos + D(1, -1), Edge::Blank) {
                        return true;
                    }

                    // yielding L-chain
                    if !self.has_clue[P(y / 2 + 1, x / 2)] {
                        if self.decide_edge(pos + D(2, -2), Edge::Line) {
                            return true;
                        }
                        if self.decide_edge(pos + D(3, -1), Edge::Line) {
                            return true;
                        }
                    }
                }
            } else {
                if !self.down_left[P(y / 2, x / 2)] && self.get_edge(pos + D(-1, -1)) == Edge::Line
                {
                    return self.set_inconsistent();
                }
                if !self.down_right[P(y / 2, x / 2)] && self.get_edge(pos + D(-1, 1)) == Edge::Line
                {
                    return self.set_inconsistent();
                }

                if self.get_edge(pos + D(0, -2)) == Edge::Line {
                    if self.decide_edge(pos + D(-1, -1), Edge::Blank) {
                        return true;
                    }
                    if self.decide_edge(pos + D(1, -1), Edge::Blank) {
                        return true;
                    }
                } else if self.get_edge(pos + D(-1, -1)) == Edge::Line {
                    if self.decide_edge(pos + D(0, -2), Edge::Blank) {
                        return true;
                    }
                    if self.decide_edge(pos + D(1, -1), Edge::Blank) {
                        return true;
                    }

                    // yielding L-chain
                    if !self.has_clue[P(y / 2 + 1, x / 2 - 1)] {
                        if self.decide_edge(pos + D(1, -3), Edge::Line) {
                            return true;
                        }
                        if self.decide_edge(pos + D(2, -2), Edge::Line) {
                            return true;
                        }
                    }
                } else if self.get_edge(pos + D(1, -1)) == Edge::Line {
                    if self.decide_edge(pos + D(0, -2), Edge::Blank) {
                        return true;
                    }
                    if self.decide_edge(pos + D(-1, -1), Edge::Blank) {
                        return true;
                    }
                }

                if self.get_edge(pos + D(0, 2)) == Edge::Line {
                    if self.decide_edge(pos + D(-1, 1), Edge::Blank) {
                        return true;
                    }
                    if self.decide_edge(pos + D(1, 1), Edge::Blank) {
                        return true;
                    }
                } else if self.get_edge(pos + D(-1, 1)) == Edge::Line {
                    if self.decide_edge(pos + D(0, 2), Edge::Blank) {
                        return true;
                    }
                    if self.decide_edge(pos + D(1, 1), Edge::Blank) {
                        return true;
                    }

                    // yielding L-chain
                    if !self.has_clue[P(y / 2 + 1, x / 2 + 1)] {
                        if self.decide_edge(pos + D(1, 3), Edge::Line) {
                            return true;
                        }
                        if self.decide_edge(pos + D(2, 2), Edge::Line) {
                            return true;
                        }
                    }
                } else if self.get_edge(pos + D(1, 1)) == Edge::Line {
                    if self.decide_edge(pos + D(0, 2), Edge::Blank) {
                        return true;
                    }
                    if self.decide_edge(pos + D(-1, 1), Edge::Blank) {
                        return true;
                    }
                }
            }
        }

        // check incident vertices
        if self.inspect(end1) {
            return true;
        }
        if self.inspect(end2) {
            return true;
        }

        return false;
    }

    /// Inspect vertex `cd`.
    /// `cd` must be in vertex-coordination.
    fn inspect(&mut self, pos: P) -> bool {
        if self.unused[pos] {
            return false;
        }

        //let (Y(y), X(x)) = cd;

        let mut n_line = if self.has_clue[pos] { 1 } else { 0 };
        let mut n_undecided = 0;
        for &d in &FOUR_NEIGHBOURS {
            match self.get_edge(LP::of_vertex(pos) + d) {
                Edge::Undecided => n_undecided += 1,
                Edge::Line => n_line += 1,
                Edge::Blank => (),
            }
        }

        let another_end = self.another_end[pos];
        if another_end < -1 {
            for &d in &FOUR_NEIGHBOURS {
                let pos2 = pos + d;
                if self.another_end.is_valid_p(pos2) {
                    let another_end2 = self.another_end[pos2];
                    if another_end2 < -1 {
                        if self.decide_edge(
                            LP::of_vertex(pos) + d,
                            if another_end == another_end2 {
                                Edge::Line
                            } else {
                                Edge::Blank
                            },
                        ) {
                            return true;
                        }
                    }
                }
            }
        }

        if n_line >= 3 {
            return self.set_inconsistent();
        }
        if n_line == 2 {
            for &d in &FOUR_NEIGHBOURS {
                let pos2 = LP::of_vertex(pos) + d;
                if self.get_edge(pos2) == Edge::Undecided {
                    if self.decide_edge(pos2, Edge::Blank) {
                        return true;
                    }
                }
            }
        } else if n_line == 1 {
            if n_undecided == 1 {
                for &d in &FOUR_NEIGHBOURS {
                    let pos2 = LP::of_vertex(pos) + d;
                    if self.get_edge(pos2) == Edge::Undecided {
                        if self.decide_edge(pos2, Edge::Line) {
                            return true;
                        }
                    }
                }
            } else if n_undecided == 0 {
                return self.set_inconsistent();
            }
        } else if n_line == 0 && self.disallow_unused_cell {
            if n_undecided < 2 {
                return self.set_inconsistent();
            } else if n_undecided == 2 {
                for &d in &FOUR_NEIGHBOURS {
                    let pos2 = LP::of_vertex(pos) + d;
                    if self.get_edge(pos2) == Edge::Undecided {
                        if self.decide_edge(pos2, Edge::Line) {
                            return true;
                        }
                    }
                }
            }
        }
        /*
        if n_line == 0 && n_undecided == 2 && self.get_edge((Y(y * 2 + 1), X(x * 2))) == Edge::Undecided {
            if self.get_edge((Y(y * 2), X(x * 2 - 1))) == Edge::Undecided && (!self.down_left[(Y(y), X(x))] || self.get_edge((Y(y * 2 + 2), X(x * 2 - 1))) == Edge::Line || self.get_edge((Y(y * 2 + 1), X(x * 2 - 2))) == Edge::Line) {
                if self.decide_edge((Y(y * 2 + 1), X(x * 2)), Edge::Blank) { return true; }
            }
            if self.get_edge((Y(y * 2), X(x * 2 + 1))) == Edge::Undecided && (!self.down_right[(Y(y), X(x))] || self.get_edge((Y(y * 2 + 2), X(x * 2 + 1))) == Edge::Line || self.get_edge((Y(y * 2 + 1), X(x * 2 + 2))) == Edge::Line) {
                if self.decide_edge((Y(y * 2 + 1), X(x * 2)), Edge::Blank) { return true; }
            }
        }
        */
        false
    }
}
impl fmt::Debug for SolverField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let height = self.height();
        let width = self.width();

        // TODO: problems with more than 34 lines
        let trans = [
            '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H',
            'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y',
        ];

        for y in 0..(2 * height - 1) {
            for x in 0..(2 * width - 1) {
                match (y % 2, x % 2) {
                    (0, 0) => write!(
                        f,
                        "{}",
                        match self.another_end[P(y / 2, x / 2)] {
                            n @ -100...-2 => trans[((-n) - 2) as usize],
                            _ => '+',
                        }
                    )?,
                    (0, 1) => write!(
                        f,
                        "{}",
                        match self.get_edge(LP(y, x)) {
                            Edge::Undecided => ' ',
                            Edge::Line => '-',
                            Edge::Blank => 'x',
                        }
                    )?,
                    (1, 0) => write!(
                        f,
                        "{}",
                        match self.get_edge(LP(y, x)) {
                            Edge::Undecided => ' ',
                            Edge::Line => '|',
                            Edge::Blank => 'x',
                        }
                    )?,
                    (1, 1) => write!(f, " ")?,
                    _ => unreachable!(),
                }
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}
struct AnswerInfo {
    answers: Vec<LinePlacement>,
    limit: Option<usize>,
    terminate_on_not_fully_filled: bool,
    found_not_fully_filled: bool,
}

pub fn solve2(
    problem: &Grid<Clue>,
    limit: Option<usize>,
    disallow_unused_cell: bool,
    terminate_on_not_fully_filled: bool,
) -> AnswerDetail {
    let mut solver_field = SolverField::new(problem, disallow_unused_cell);
    let mut answer_info = AnswerInfo {
        answers: Vec::new(),
        limit,
        terminate_on_not_fully_filled,
        found_not_fully_filled: false,
    };
    let mut n_steps = 0u64;

    search(0, 0, &mut solver_field, &mut answer_info, &mut n_steps, 0);

    let fully_checked = if let Some(limit) = limit {
        limit == answer_info.answers.len()
    } else {
        true
    };

    AnswerDetail {
        answers: answer_info.answers,
        fully_checked,
        found_not_fully_filled: answer_info.found_not_fully_filled,
        n_steps,
    }
}
fn prune_cut(field: &SolverField) -> bool {
    let width = field.width();
    let mut accsum = vec![0; width as usize];

    for x in 0..width {
        accsum[x as usize] = -field.open_end_count[x as usize];
    }
    for n in 0..field.number_end.len() {
        let (a, b) = field.number_end[n];
        if a != -1 {
            if a < b {
                accsum[a as usize] += 1;
                accsum[b as usize] -= 1;
            } else {
                accsum[b as usize] += 1;
                accsum[a as usize] -= 1;
            }
        }
    }

    for i in 1..(width as usize) {
        accsum[i] += accsum[i - 1];
    }
    for x in 0..(width - 1) {
        if field.undecided_count[x as usize] < accsum[x as usize] {
            return true;
        }
    }
    false
}
fn search(
    y: i32,
    x: i32,
    field: &mut SolverField,
    answer_info: &mut AnswerInfo,
    n_steps: &mut u64,
    line_chain: i32,
) -> bool {
    let mut y = y;
    let mut x = x;
    let mut line_chain = line_chain;
    if x == field.width() {
        y += 1;
        x = 0;
        line_chain = 0;
    }
    while y < field.height()
        && field.get_edge(LP(y * 2 + 1, x * 2)) != Edge::Undecided
        && field.get_edge(LP(y * 2, x * 2 + 1)) != Edge::Undecided
    {
        if x == field.width() - 1 {
            y += 1;
            x = 0;
        } else {
            x += 1;
            if y > 0 {
                if field.get_edge(LP(y * 2, x * 2 - 1)) == Edge::Line {
                    if field.get_edge(LP(y * 2 - 2, x * 2 - 1)) == Edge::Line {
                        line_chain = -field.width();
                    } else {
                        line_chain += 1;
                    }
                } else {
                    line_chain = 0;
                }
            }
            if line_chain > 0 && field.get_edge(LP(y * 2 - 1, x * 2)) == Edge::Line {
                if field.get_edge(LP(y * 2 - 1, (x - line_chain) * 2)) == Edge::Line
                    && field.left_clue_distance[P(y - 1, x)] >= line_chain
                {
                    return false;
                }
            }
        }
    }
    *n_steps += 1;

    if y == field.height() {
        // answer found
        answer_info.answers.push(field.get_line_placement());
        if answer_info.terminate_on_not_fully_filled {
            let mut full = true;
            for y in 0..field.height() {
                for x in 0..field.width() {
                    if !field.unused[P(y, x)]
                        && field.get_edge(LP(y * 2 - 1, x * 2)) == Edge::Blank
                        && field.get_edge(LP(y * 2 + 1, x * 2)) == Edge::Blank
                        && field.get_edge(LP(y * 2, x * 2 - 1)) == Edge::Blank
                        && field.get_edge(LP(y * 2, x * 2 + 1)) == Edge::Blank
                    {
                        full = false;
                    }
                }
            }
            if !full {
                answer_info.found_not_fully_filled = true;
                return true;
            }
        }
        if let Some(lim) = answer_info.limit {
            if answer_info.answers.len() >= lim {
                return true;
            }
        }
        return false;
    }

    let degree_common = if field.has_clue[P(y, x)] { 1 } else { 0 }
        + if field.get_edge(LP(y * 2, x * 2 - 1)) == Edge::Line {
            1
        } else {
            0
        }
        + if field.get_edge(LP(y * 2, x * 2 + 1)) == Edge::Line {
            1
        } else {
            0
        }
        + if field.get_edge(LP(y * 2 - 1, x * 2)) == Edge::Line {
            1
        } else {
            0
        }
        + if field.get_edge(LP(y * 2 + 1, x * 2)) == Edge::Line {
            1
        } else {
            0
        };

    for mask in 0..4 {
        let mask = 3 - mask;
        let right = (mask & 1) != 0;
        let down = (mask & 2) != 0;

        if right && field.get_edge(LP(y * 2, x * 2 + 1)) != Edge::Undecided {
            continue;
        }
        if down && field.get_edge(LP(y * 2 + 1, x * 2)) != Edge::Undecided {
            continue;
        }

        let degree = degree_common + if right { 1 } else { 0 } + if down { 1 } else { 0 };
        if degree != 0 && degree != 2 {
            continue;
        }

        let right_effective = right || (field.get_edge(LP(y * 2, x * 2 + 1)) == Edge::Line);
        let down_effective = down || (field.get_edge(LP(y * 2 + 1, x * 2)) == Edge::Line);
        if right_effective && down_effective {
            if !field.down_right[P(y, x)] {
                continue;
            }
        }
        if right_effective && field.get_edge(LP(y * 2 - 1, x * 2 + 2)) == Edge::Line {
            if line_chain > 0
                && field.get_edge(LP(y * 2 - 2, x * 2 + 1)) == Edge::Blank
                && field.get_edge(LP(y * 2 - 1, (x - line_chain) * 2)) == Edge::Line
                && field.left_clue_distance[P(y - 1, x + 1)] >= line_chain + 1
            {
                continue;
            }
        }
        field.add_checkpoint();
        let mut inconsistent = false;

        inconsistent |= field.decide_edge(
            LP(y * 2, x * 2 + 1),
            if right { Edge::Line } else { Edge::Blank },
        );
        if !inconsistent {
            inconsistent |= field.decide_edge(
                LP(y * 2 + 1, x * 2),
                if down { Edge::Line } else { Edge::Blank },
            );
        }
        if !inconsistent {
            inconsistent |= prune_cut(field);
        }
        if !inconsistent {
            let line_chain2 = if right_effective {
                if field.get_edge(LP(y * 2 - 2, x * 2 + 1)) == Edge::Line {
                    -field.width()
                } else {
                    line_chain + 1
                }
            } else {
                0
            };
            if search(y, x + 1, field, answer_info, n_steps, line_chain2) {
                return true;
            }
        }
        field.rollback();
    }
    return false;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_solver_unused_cells() {
        let problem_base = [
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [1, 2, -1, 2, 1],
            [-1, -1, -1, -1, -1],
        ];
        let mut problem = Grid::new(
            problem_base.len() as i32,
            problem_base[0].len() as i32,
            NO_CLUE,
        );
        for y in 0..problem_base.len() {
            for x in 0..problem_base[0].len() {
                problem[P(y as i32, x as i32)] = Clue(problem_base[y][x]);
            }
        }

        let ans = solve2(&problem, None, false, false);
        assert_eq!(ans.len(), 1);
    }
}