use super::super::{Grid, P};
use super::*;

#[derive(Clone)]
pub struct Field {
    size: i32,
    n_alpha: i32,
    cand: Grid<Cand>,
    value: Grid<Value>,
    clue_front: Vec<Clue>,
    clue_back: Vec<Clue>,
    total_cands: i32,
    inconsistent: bool,
}

impl Field {
    pub fn new(size: i32, n_alpha: i32) -> Field {
        assert!(n_alpha >= 2);
        Field {
            size,
            n_alpha,
            cand: Grid::new(size, size, Cand((1u32 << n_alpha) - 1)),
            value: Grid::new(size, size, UNDECIDED),
            clue_front: vec![NO_CLUE; (2 * size) as usize],
            clue_back: vec![NO_CLUE; (2 * size) as usize],
            total_cands: size * size * (n_alpha + 1),
            inconsistent: false,
        }
    }
    pub fn from_problem(problem: &Problem) -> Field {
        let size = problem.size();
        let mut ret = Field::new(size, problem.n_alpha());
        for &loc in &[ClueLoc::Left, ClueLoc::Right, ClueLoc::Top, ClueLoc::Bottom] {
            for i in 0..size {
                let clue = problem.get_clue(loc, i);
                if clue != NO_CLUE {
                    ret.set_clue(loc, i, clue);
                }
            }
        }
        ret
    }
    pub fn get_value(&self, cell: P) -> Value {
        self.value[cell]
    }
    pub fn inconsistent(&self) -> bool {
        self.inconsistent
    }
    pub fn total_cands(&self) -> i32 {
        self.total_cands
    }
    pub fn is_solved(&self) -> bool {
        self.total_cands == self.size * self.n_alpha
    }
    pub fn get_clue(&self, loc: ClueLoc, idx: i32) -> Clue {
        match loc {
            ClueLoc::Left => self.clue_front[idx as usize],
            ClueLoc::Right => self.clue_back[idx as usize],
            ClueLoc::Top => self.clue_front[(idx + self.size) as usize],
            ClueLoc::Bottom => self.clue_back[(idx + self.size) as usize],
        }
    }
    pub fn set_clue(&mut self, loc: ClueLoc, idx: i32, clue: Clue) {
        let current = self.get_clue(loc, idx);
        if current != NO_CLUE {
            if current != clue {
                self.inconsistent = true;
            }
            return;
        }
        let size = self.size;
        match loc {
            ClueLoc::Left => self.clue_front[idx as usize] = clue,
            ClueLoc::Right => self.clue_back[idx as usize] = clue,
            ClueLoc::Top => self.clue_front[(idx + size) as usize] = clue,
            ClueLoc::Bottom => self.clue_back[(idx + size) as usize] = clue,
        }
        if loc == ClueLoc::Left || loc == ClueLoc::Right {
            self.inspect_row(idx);
        } else {
            self.inspect_row(idx + size);
        }
    }
    pub fn decide(&mut self, cell: P, val: Value) {
        let current = self.value[cell];
        if current.0 >= 0 && val == SOME {
            return;
        }
        if current != UNDECIDED && !(current == SOME && val != EMPTY) {
            if current != val {
                self.inconsistent = true;
            }
            return;
        }
        if current == UNDECIDED {
            if val == UNDECIDED {
                return;
            }
            self.total_cands -= 1;
        }

        self.value[cell] = val;

        if val == EMPTY {
            self.limit_cand(cell, Cand(0));
        } else if val == SOME {
            self.inspect_cell(cell);
        } else if val != UNDECIDED {
            self.limit_cand(cell, Cand::singleton(val.0));

            let P(y, x) = cell;
            let limit = !Cand::singleton(val.0);
            for y2 in 0..self.size {
                if y != y2 {
                    self.limit_cand(P(y2, x), limit);
                }
            }
            for x2 in 0..self.size {
                if x != x2 {
                    self.limit_cand(P(y, x2), limit);
                }
            }
        }
    }
    pub fn apply_methods(&mut self) {
        loop {
            let current_cands = self.total_cands();

            self.hidden_candidate();
            if self.inconsistent() {
                return;
            }
            self.fishy_method();
            if self.inconsistent() {
                return;
            }

            if self.total_cands() == current_cands {
                break;
            }
        }
    }
    pub fn trial_and_error(&mut self) {
        loop {
            self.apply_methods();
            if self.inconsistent() {
                break;
            }
            if !self.trial_and_error_step() {
                break;
            }
        }
    }
    fn trial_and_error_step(&mut self) -> bool {
        let size = self.size;
        let n_alpha = self.n_alpha;

        let mut is_update = false;
        for y in 0..size {
            for x in 0..size {
                let pos = P(y, x);
                let val = self.get_value(pos);
                if !(val == UNDECIDED || val == SOME) {
                    continue;
                }

                let cand = self.cand[pos];
                let mut valid_cands = vec![];

                for i in 0..n_alpha {
                    if !cand.is_set(i) {
                        continue;
                    }

                    let mut field_cloned = self.clone();
                    field_cloned.decide(pos, Value(i));

                    if !field_cloned.inconsistent() {
                        valid_cands.push((Value(i), field_cloned));
                    }
                }
                if val != SOME {
                    let mut field_cloned = self.clone();
                    field_cloned.decide(pos, EMPTY);
                    if !field_cloned.inconsistent() {
                        valid_cands.push((EMPTY, field_cloned));
                    }
                }

                if valid_cands.len() == 0 {
                    self.inconsistent = true;
                    return false;
                }
                if valid_cands.len() == 1 {
                    let only_cand = valid_cands.pop().unwrap();
                    *self = only_cand.1;
                    is_update = true;
                }
            }
        }
        is_update
    }
    /// Returns `pos`-th cell of group `gid`.
    fn group(&self, gid: i32, pos: i32) -> P {
        if gid < self.size {
            P(gid, pos)
        } else {
            P(pos, gid - self.size)
        }
    }
    /// Returns `pos`-th cell of group `gid` with reversed indexing of cells when `dir` is `true`.
    fn directed_group(&self, gid: i32, pos: i32, dir: bool) -> P {
        self.group(gid, if dir { self.size - pos - 1 } else { pos })
    }
    fn limit_cand(&mut self, cell: P, lim: Cand) {
        let current_cand = self.cand[cell];

        if (current_cand & lim) == current_cand {
            return;
        }

        let new_cand = current_cand & lim;
        self.cand[cell] = new_cand;
        self.total_cands -= (current_cand.0.count_ones() - new_cand.0.count_ones()) as i32;

        if self.cand[cell] == Cand(0) {
            self.decide(cell, EMPTY);
        }
        self.inspect_cell(cell);
    }
    fn inspect_cell(&mut self, cell: P) {
        if self.value[cell] == SOME && self.cand[cell].count_set_cands() == 1 {
            let val = Value(self.cand[cell].smallest_set_cand());
            self.decide(cell, val);
        }
        let size = self.size;
        let P(y, x) = cell;
        self.inspect_row(y);
        self.inspect_row(x + size);
    }
    fn inspect_row(&mut self, group: i32) {
        let size = self.size;
        let n_alpha = self.n_alpha;

        let mut n_some = 0;
        let mut n_empty = 0;

        for i in 0..size {
            let v = self.value[self.group(group, i)];
            if v == EMPTY {
                n_empty += 1;
            } else if v != UNDECIDED {
                n_some += 1;
            }
        }

        if n_some == n_alpha {
            for i in 0..size {
                let c = self.group(group, i);
                if self.value[c] == UNDECIDED {
                    self.decide(c, EMPTY);
                }
            }
        }
        if n_empty == size - n_alpha {
            for i in 0..size {
                let c = self.group(group, i);
                if self.value[c] == UNDECIDED {
                    self.decide(c, SOME);
                }
            }
        }

        for a in 0..n_alpha {
            let mut loc = -1;
            for i in 0..size {
                if self.cand[self.group(group, i)].is_set(a) {
                    if loc == -1 {
                        loc = i;
                    } else {
                        loc = -2;
                        break;
                    }
                }
            }
            if loc == -1 {
                self.inconsistent = true;
                return;
            } else if loc != -2 {
                let c = self.group(group, loc);
                self.decide(c, Value(a));
            }
        }
        for &dir in [true, false].iter() {
            let clue = if !dir {
                self.clue_front[group as usize]
            } else {
                self.clue_back[group as usize]
            };
            if clue == NO_CLUE {
                continue;
            }

            let mut first_nonempty_id = -1;
            for i in 0..size {
                let v = self.value[self.directed_group(group, i, dir)];
                if v != EMPTY {
                    first_nonempty_id = i;
                    break;
                }
            }
            if first_nonempty_id == -1 {
                self.inconsistent = true;
                return;
            }
            let first_nonempty_cell = self.directed_group(group, first_nonempty_id, dir);
            self.limit_cand(first_nonempty_cell, Cand::singleton(clue.0));

            let mut first_diff_id = -1;
            for i in 0..size {
                let v = self.value[self.directed_group(group, i, dir)];
                if v == SOME && !self.cand[self.directed_group(group, i, dir)].is_set(clue.0) {
                    first_diff_id = i;
                }
            }
            if first_diff_id != -1 {
                for i in (first_diff_id + 1)..size {
                    let c = self.directed_group(group, i, dir);
                    self.limit_cand(c, !Cand::singleton(clue.0));
                }
            }

            let mut n_back_diff = n_alpha - 1;
            for i in 0..size {
                let c = self.directed_group(group, i, !dir);
                let v = self.value[c];
                if v != EMPTY {
                    self.limit_cand(c, !Cand::singleton(clue.0));
                    n_back_diff -= 1;
                    if n_back_diff == 0 {
                        break;
                    }
                }
            }

            let mut mask = Cand((1u32 << n_alpha) - 1) & !Cand::singleton(clue.0);
            for i in 0..size {
                let c = self.directed_group(group, i, !dir);
                self.limit_cand(c, !Cand::singleton(clue.0));
                mask &= !self.cand[c];
                if mask == Cand(0) {
                    break;
                }
            }
        }
    }
    /// Apply *hidden candidate method* to the field.
    /// Note: runs in O*(2^(n_alpha)) and works only if n_alpha < 32.
    pub fn hidden_candidate(&mut self) {
        let size = self.size;
        let n_alpha = self.n_alpha;
        for bits in 1..(1u32 << n_alpha) {
            let mask = Cand(bits);
            for g in 0..(2 * size) {
                let mut n_match = 0;
                for i in 0..size {
                    if self.cand[self.group(g, i)] & mask != Cand(0) {
                        n_match += 1;
                    }
                }
                if n_match < bits.count_ones() {
                    self.inconsistent = true;
                    return;
                } else if n_match == bits.count_ones() {
                    for i in 0..size {
                        let c = self.group(g, i);
                        if self.cand[c] & mask != Cand(0) {
                            self.decide(c, SOME);
                            self.limit_cand(c, mask);
                        }
                    }
                }
            }
        }
    }
    /// Apply *fishy method* (like *X-wing* and *Sword fish* in Sudoku) to the field.
    /// Note: runs in O*(2^size) and works only if size < 32.
    pub fn fishy_method(&mut self) {
        let size = self.size;
        for i in 0..self.n_alpha {
            let mut masks = vec![];
            for y in 0..size {
                let mut mask = 0u32;
                for x in 0..size {
                    let pos = P(y, x);
                    if self.cand[pos].is_set(i) {
                        mask |= 1u32 << x;
                    }
                }
                masks.push(mask);
            }
            for bits in 1..(1u32 << size) {
                let mut ors = 0u32;
                for y in 0..size {
                    if (bits & (1u32 << y)) != 0 {
                        ors |= masks[y as usize];
                    }
                }
                if bits.count_ones() > ors.count_ones() {
                    self.inconsistent = true;
                    return;
                } else if bits.count_ones() == ors.count_ones() {
                    for y in 0..size {
                        if (bits & (1u32 << y)) == 0 {
                            masks[y as usize] &= !ors;
                        }
                    }
                }
            }
            for y in 0..size {
                for x in 0..size {
                    if (masks[y as usize] & (1u32 << x)) == 0 {
                        self.limit_cand(P(y, x), !Cand::singleton(i));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduction() {
        {
            // a symbol shouldn't occur more than once in a row / a column
            let mut field = Field::new(5, 3);

            assert_eq!(field.cand[P(0, 0)], Cand(7));
            assert_eq!(field.cand[P(0, 1)], Cand(7));
            assert_eq!(field.cand[P(1, 1)], Cand(7));

            field.decide(P(0, 0), Value(0));

            assert_eq!(field.inconsistent, false);
            assert_eq!(field.cand[P(0, 0)], Cand(1));
            assert_eq!(field.cand[P(0, 1)], Cand(6));
            assert_eq!(field.cand[P(1, 0)], Cand(6));
            assert_eq!(field.cand[P(1, 1)], Cand(7));
        }
        {
            // there must be exactly `n_alpha` symbols in a row / a column
            let mut field = Field::new(5, 3);

            field.decide(P(1, 0), SOME);
            field.decide(P(1, 1), SOME);
            field.decide(P(1, 2), SOME);

            assert_eq!(field.inconsistent, false);
            assert_eq!(field.value[P(1, 3)], EMPTY);
        }
        {
            // there must be exactly `n_alpha` symbols in a row / a column
            let mut field = Field::new(5, 3);

            field.decide(P(3, 2), EMPTY);
            field.decide(P(4, 2), EMPTY);

            assert_eq!(field.inconsistent, false);
            assert_eq!(field.value[P(1, 2)], SOME);
        }
        {
            let mut field = Field::new(5, 3);

            field.limit_cand(P(0, 2), Cand(5));
            field.limit_cand(P(2, 2), Cand(5));
            field.limit_cand(P(3, 2), Cand(5));
            field.limit_cand(P(4, 2), Cand(5));

            assert_eq!(field.inconsistent, false);
            assert_eq!(field.value[P(1, 2)], Value(1));
            assert_eq!(field.cand[P(1, 3)], Cand(5));
        }
    }

    #[test]
    fn test_clue() {
        {
            let mut field = Field::new(5, 3);

            field.set_clue(ClueLoc::Left, 0, Clue(0));

            assert_eq!(field.cand[P(0, 0)], Cand(1));
            assert_eq!(field.cand[P(0, 3)], Cand(6));
            assert_eq!(field.cand[P(0, 4)], Cand(6));
        }
    }

    #[test]
    fn test_problem() {
        {
            let mut problem = Problem::new(5, 3);
            problem.set_clue(ClueLoc::Top, 1, Clue(1));

            let field = Field::from_problem(&problem);

            assert_eq!(field.cand[P(0, 1)], Cand(2));
            assert_eq!(field.cand[P(4, 1)], Cand(5));
        }
    }
}