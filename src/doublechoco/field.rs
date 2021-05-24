use super::*;
use std::cell::Cell;
use crate::common::{Grid, D, FOUR_NEIGHBOURS, LP, P};

#[derive(Clone)]
pub struct Field {
    color: Grid<Color>,
    clue: Grid<Clue>,
    border: Grid<Border>,
    size_low: Grid<i32>,
    size_high: Grid<i32>,
    frozen: Grid<bool>,
    num_decided_borders: i32,
    inconsistent: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum CellAffinity {
    Undecided,
    Same,
    Different,
}
impl CellAffinity {
    fn same(a: CellAffinity, b: CellAffinity) -> bool {
        match (a, b) {
            (CellAffinity::Same, CellAffinity::Same) => true,
            _ => false,
        }
    }
    fn opposite(a: CellAffinity, b: CellAffinity) -> bool {
        match (a, b) {
            (CellAffinity::Same, CellAffinity::Different)
            | (CellAffinity::Different, CellAffinity::Same) => true,
            _ => false,
        }
    }
}

impl Field {
    pub fn new(color: &Grid<Color>, clue: &Grid<Clue>) -> Field {
        let height = color.height();
        let width = color.width();
        assert_eq!(height, clue.height());
        assert_eq!(width, clue.width());

        let mut size_low = Grid::new(height, width, 1);
        let mut size_high = Grid::new(height, width, height * width / 2);
        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if clue[pos] != NO_CLUE {
                    size_low[pos] = clue[pos];
                    size_high[pos] = clue[pos];
                }
            }
        }
        Field {
            color: color.clone(),
            clue: clue.clone(),
            border: Grid::new(height * 2 - 1, width * 2 - 1, Border::Undecided),
            size_low,
            size_high,
            frozen: Grid::new(height, width, false),
            num_decided_borders: 0,
            inconsistent: false,
        }
    }
    pub fn height(&self) -> i32 {
        self.color.height()
    }
    pub fn width(&self) -> i32 {
        self.color.width()
    }
    pub fn inconsistent(&self) -> bool {
        self.inconsistent
    }
    pub fn set_inconsistent(&mut self) {
        self.inconsistent = true;
    }
    pub fn border(&self, pos: LP) -> Border {
        self.border[pos]
    }
    pub fn decide_border(&mut self, pos: LP, border: Border) {
        if self.border[pos] != Border::Undecided {
            if self.border[pos] != border {
                self.set_inconsistent();
            }
            return;
        }
        self.border[pos] = border;
        self.num_decided_borders += 1;

        let LP(y, x) = pos;
        self.ensure_no_broken_border(P((y + 1) / 2, (x + 1) / 2));
        self.ensure_no_broken_border(P((y + 2) / 2, (x + 2) / 2));
    }
    pub fn ensure_no_broken_border(&mut self, pos: P) {
        let height = self.height();
        let width = self.width();
        let P(y, x) = pos;

        if !(0 < y && y < height && 0 < x && x < width) {
            return;
        }

        let mut num_undecided = 0;
        let mut num_line = 0;

        for &d in &FOUR_NEIGHBOURS {
            match self.border[LP(y * 2 - 1, x * 2 - 1) + d] {
                Border::Undecided => num_undecided += 1,
                Border::Line => num_line += 1,
                Border::Blank => (),
            }
        }

        if num_line == 1 {
            if num_undecided == 0 {
                self.set_inconsistent();
                return;
            } else if num_undecided == 1 {
                for &d in &FOUR_NEIGHBOURS {
                    if self.border[LP(y * 2 - 1, x * 2 - 1) + d] == Border::Undecided {
                        self.decide_border(LP(y * 2 - 1, x * 2 - 1) + d, Border::Line);
                    }
                }
            }
        } else if num_line == 0 && num_undecided == 1 {
            for &d in &FOUR_NEIGHBOURS {
                if self.border[LP(y * 2 - 1, x * 2 - 1) + d] == Border::Undecided {
                    self.decide_border(LP(y * 2 - 1, x * 2 - 1) + d, Border::Blank);
                }
            }
        }
    }
    fn check_connected_components_dfs(&self, pos: P, visited: &mut Grid<bool>) -> i32 {
        if !self.color.is_valid_p(pos) || visited[pos] {
            return 0;
        }
        visited[pos] = true;
        let mut ret = match self.color[pos] {
            Color::Black => 1,
            Color::White => -1,
        };
        for &d in &FOUR_NEIGHBOURS {
            if self.color.is_valid_p(pos + d) && self.border[LP::of_vertex(pos) + d] != Border::Line
            {
                ret += self.check_connected_components_dfs(pos + d, visited);
            }
        }
        ret
    }
    fn check_connected_components(&mut self) {
        let height = self.height();
        let width = self.width();
        let mut visited = Grid::new(height, width, false);

        for y in 0..height {
            for x in 0..width {
                if !visited[P(y, x)] {
                    if self.check_connected_components_dfs(P(y, x), &mut visited) != 0 {
                        self.set_inconsistent();
                        return;
                    }
                }
            }
        }
    }
    fn expand_block_size_dfs1(
        &self,
        pos: P,
        visited: &mut Grid<bool>,
        group: &mut Vec<P>,
        affinity: &Grid<CellAffinity>,
        base_color: Color,
    ) -> i32 {
        let P(y, x) = pos;
        if !(0 <= y && y < self.height() && 0 <= x && x < self.width())
            || visited[pos]
            || affinity[pos] == CellAffinity::Different
        {
            return 0;
        }
        visited[pos] = true;
        let mut ret = 0;
        if self.color[pos] == base_color {
            group.push(pos);
            ret = 1;
        }
        for &d in &FOUR_NEIGHBOURS {
            let pos2 = pos + d;
            if self.clue.is_valid_p(pos2)
                && self.border[LP::of_vertex(pos) + d] != Border::Line
                && !(self.color[pos] != base_color && self.color[pos2] == base_color)
            {
                ret += self.expand_block_size_dfs1(pos2, visited, group, affinity, base_color);
            }
        }
        ret
    }
    fn expand_block_size_dfs2(
        &self,
        pos: P,
        current_depth: i32,
        depths: &mut Grid<i32>,
        ans: &mut Grid<i32>,
        affinity: &Grid<CellAffinity>,
        base_color: Color,
    ) -> (i32, i32) {
        depths[pos] = current_depth;
        let mut lowlink = self.height() * self.width();
        let mut descendant_size = 1;
        ans[pos] = -1;
        for &d in &FOUR_NEIGHBOURS {
            let pos2 = pos + d;
            if self.clue.is_valid_p(pos2)
                && self.border[LP::of_vertex(pos) + d] != Border::Line
                && affinity[pos2] != CellAffinity::Different
                && self.color[pos2] == base_color
            {
                if depths[pos2] == -1 {
                    let (l, d) = self.expand_block_size_dfs2(
                        pos2,
                        current_depth + 1,
                        depths,
                        ans,
                        affinity,
                        base_color,
                    );
                    lowlink = lowlink.min(l);
                    if l >= current_depth {
                        ans[pos] -= d;
                    }
                    descendant_size += d;
                } else {
                    lowlink = lowlink.min(depths[pos2]);
                }
            }
        }
        (lowlink, descendant_size)
    }
    fn expand_block_size(&self, base: P, affinity: &mut Grid<CellAffinity>) -> bool {
        let height = self.height();
        let width = self.width();

        let mut size = NO_CLUE;
        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if affinity[pos] == CellAffinity::Same {
                    if self.clue[pos] != NO_CLUE {
                        if size != NO_CLUE && size != self.clue[pos] {
                            return true;
                        } else {
                            size = self.clue[pos];
                        }
                    }
                }
            }
        }
        let mut visited = Grid::new(height, width, false);
        let mut group = vec![];

        self.expand_block_size_dfs1(base, &mut visited, &mut group, affinity, self.color[base]);

        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if !visited[pos] {
                    affinity[pos] = CellAffinity::Different;
                }
            }
        }

        if size == NO_CLUE {
            return false;
        }
        let mut depths = Grid::new(height, width, -1);
        let mut ans = Grid::new(height, width, 0);
        let (_, g) = self.expand_block_size_dfs2(
            base,
            0,
            &mut depths,
            &mut ans,
            &affinity,
            self.color[base],
        );
        for cand in group {
            if g + ans[cand] < size {
                match affinity[cand] {
                    CellAffinity::Different => return true,
                    CellAffinity::Undecided => affinity[cand] = CellAffinity::Same,
                    _ => (),
                }
            }
        }
        false
    }
    fn find_companion(&self, base: P, affinity: &mut Grid<CellAffinity>) -> bool {
        let height = self.height();
        let width = self.width();
        let base_color = self.color[base];

        let mut same_group = vec![];
        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if affinity[pos] == CellAffinity::Same && self.color[pos] == base_color {
                    same_group.push(pos);
                }
            }
        }
        if same_group.len() == 0 {
            return true;
        }

        let mut is_closed = true;
        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if self.color[pos] == base_color && affinity[pos] == CellAffinity::Undecided {
                    is_closed = false;
                }
            }
        }

        fn rotate_group(group: &Vec<P>, mode: i32) -> Vec<P> {
            let trans_y = (mode & 4) != 0;
            let trans_x = (mode & 2) != 0;
            let flip = (mode & 1) != 0;
            let mut ret = vec![];
            for &P(y, x) in group {
                let y = if trans_y { -y } else { y };
                let x = if trans_x { -x } else { x };
                ret.push(if flip { P(x, y) } else { P(y, x) });
            }
            ret
        }

        let limit = height.max(width);
        let mut num_cand = Grid::new(height, width, 0);
        let mut num_cand_total = 0;

        for rotate_mode in 0..8 {
            let group = rotate_group(&same_group, rotate_mode);

            let mut y_lo = limit;
            let mut x_lo = limit;
            let mut y_hi = -limit;
            let mut x_hi = -limit;

            for &P(y, x) in &group {
                y_lo = y_lo.min(y);
                x_lo = x_lo.min(x);
                y_hi = y_hi.max(y);
                x_hi = x_hi.max(x);
            }

            // 0 <= dy + y_lo <= dy + y_hi < height
            for dy in (-y_lo)..(height - y_hi) {
                for dx in (-x_lo)..(width - x_hi) {
                    let d = D(dy, dx);
                    let mut isok = true;
                    for &pos in &group {
                        if affinity[pos + d] == CellAffinity::Different
                            || self.color[pos + d] == base_color
                        {
                            isok = false;
                            break;
                        }
                    }
                    if isok && is_closed {
                        // attached?
                        isok = false;
                        for &pos in &group {
                            for &d2 in &FOUR_NEIGHBOURS {
                                let pos2 = pos + d + d2;
                                if self.color.is_valid_p(pos2)
                                    && self.color[pos2] == base_color
                                    && affinity[pos2] == CellAffinity::Same
                                {
                                    isok = true;
                                }
                            }
                        }
                    }
                    if isok {
                        num_cand_total += 1;
                        for &pos in &group {
                            num_cand[pos + d] += 1;
                        }
                    }
                }
            }
        }
        if num_cand_total == 0 {
            return true;
        }
        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if self.color[pos] != base_color {
                    if num_cand[pos] == 0 {
                        if is_closed {
                            affinity[pos] = CellAffinity::Different;
                        }
                    } else if num_cand[pos] == num_cand_total {
                        affinity[pos] = CellAffinity::Same;
                    }
                }
            }
        }

        false
    }
    fn set_initial_affinity(&self, pos: P, affinity: &mut Grid<CellAffinity>) {
        let P(y, x) = pos;
        if !self.color.is_valid_p(pos) || affinity[pos] != CellAffinity::Undecided {
            return;
        }
        affinity[pos] = CellAffinity::Same;
        for &d in &FOUR_NEIGHBOURS {
            if !self.color.is_valid_p(pos + d) {
                continue;
            }
            match self.border[LP::of_vertex(pos) + d] {
                Border::Line => {
                    affinity[pos + d] = CellAffinity::Different;
                }
                Border::Blank => self.set_initial_affinity(pos + d, affinity),
                _ => (),
            }
        }
    }
    fn inspect(&mut self, base: P) {
        if self.frozen[base] {
            return;
        }
        let height = self.height();
        let width = self.width();

        let mut affinity = Grid::new(height, width, CellAffinity::Undecided);
        self.set_initial_affinity(base, &mut affinity);

        let mut size_low = self.size_low[base];
        let mut size_high = self.size_high[base];
        let clue = self.clue[base];
        if clue != NO_CLUE {
            for y in 0..height {
                for x in 0..width {
                    let pos = P(y, x);
                    if (self.clue[pos] != NO_CLUE && self.clue[pos] != clue)
                        || size_high < self.size_low[pos]
                        || self.size_high[pos] < size_low
                    {
                        affinity[pos] = CellAffinity::Different;
                    }
                }
            }
        }

        if self.expand_block_size(base, &mut affinity) {
            self.set_inconsistent();
            return;
        }
        if self.find_companion(base, &mut affinity) {
            self.set_inconsistent();
            return;
        }

        let mut unit_clue = NO_CLUE;
        let mut unit_size = 0;
        let mut max_size = 0;

        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if affinity[pos] != CellAffinity::Different && self.color[pos] == self.color[base] {
                    max_size += 1;
                }
                if affinity[pos] != CellAffinity::Same {
                    continue;
                }
                size_low = size_low.max(self.size_low[pos]);
                size_high = size_high.min(self.size_high[pos]);
                if self.clue[pos] != NO_CLUE {
                    if unit_clue == NO_CLUE {
                        unit_clue = self.clue[pos];
                    } else if unit_clue != self.clue[pos] {
                        self.set_inconsistent();
                        return;
                    }
                }
                if self.color[pos] == self.color[base] {
                    unit_size += 1;
                }
            }
        }
        size_high = size_high.min(max_size);
        if size_low > size_high {
            self.set_inconsistent();
            return;
        }
        size_low = size_low.max(unit_size);
        if unit_clue != NO_CLUE {
            if unit_size > unit_clue {
                self.set_inconsistent();
                return;
            } else if unit_size == unit_clue {
                for y in 0..height {
                    for x in 0..width {
                        let pos = P(y, x);
                        if self.color[pos] == self.color[base]
                            && affinity[pos] == CellAffinity::Undecided
                        {
                            affinity[pos] = CellAffinity::Different;
                        }
                    }
                }
            }
        }
        let mut is_frozen = true;
        for y in 0..height {
            for x in 0..width {
                if affinity[P(y, x)] == CellAffinity::Undecided {
                    is_frozen = false;
                }
            }
        }
        for y in 0..height {
            for x in 0..width {
                let aff = affinity[P(y, x)];
                if aff == CellAffinity::Same {
                    self.size_low[P(y, x)] = self.size_low[P(y, x)].max(size_low);
                    self.size_high[P(y, x)] = self.size_high[P(y, x)].max(size_high);
                    if self.size_low[P(y, x)] > self.size_high[P(y, x)] {
                        self.set_inconsistent();
                        return;
                    }
                    if is_frozen {
                        self.frozen[P(y, x)] = true;
                    }
                }
                if y < height - 1 {
                    let aff2 = affinity[P(y + 1, x)];
                    if CellAffinity::opposite(aff, aff2) {
                        self.decide_border(LP(y * 2 + 1, x * 2), Border::Line);
                    } else if CellAffinity::same(aff, aff2) {
                        self.decide_border(LP(y * 2 + 1, x * 2), Border::Blank);
                    }
                }
                if x < width - 1 {
                    let aff2 = affinity[P(y, x + 1)];
                    if CellAffinity::opposite(aff, aff2) {
                        self.decide_border(LP(y * 2, x * 2 + 1), Border::Line);
                    } else if CellAffinity::same(aff, aff2) {
                        self.decide_border(LP(y * 2, x * 2 + 1), Border::Blank);
                    }
                }
            }
        }
    }
    pub fn solve(&mut self) {
        let height = self.height();
        let width = self.width();
        loop {
            let last_num_decided = self.num_decided_borders;

            self.check_connected_components();
            if self.inconsistent() {
                return;
            }
            for y in 0..height {
                for x in 0..width {
                    self.inspect(P(y, x));
                    if self.inconsistent() {
                        return;
                    }
                }
            }

            if last_num_decided == self.num_decided_borders {
                break;
            }
        }
    }
    pub fn trial_and_error(&mut self, depth: i32) {
        let height = self.height();
        let width = self.width();

        if depth == 0 {
            self.solve();
            return;
        }
        self.trial_and_error(depth - 1);

        loop {
            let mut updated = false;
            for y in 0..(height * 2 - 1) {
                for x in 0..(width * 2 - 1) {
                    if y % 2 == x % 2 {
                        continue;
                    }
                    let pos = LP(y, x);
                    if self.border[pos] != Border::Undecided {
                        continue;
                    }
                    {
                        let mut field_line = self.clone();
                        field_line.decide_border(pos, Border::Line);
                        field_line.trial_and_error(depth - 1);

                        if field_line.inconsistent() {
                            updated = true;
                            self.decide_border(pos, Border::Blank);
                            self.trial_and_error(depth - 1);
                        }
                    }
                    {
                        let mut field_blank = self.clone();
                        field_blank.decide_border(pos, Border::Blank);
                        field_blank.trial_and_error(depth - 1);

                        if field_blank.inconsistent() {
                            updated = true;
                            self.decide_border(pos, Border::Line);
                            self.trial_and_error(depth - 1);
                        }
                    }
                }
            }
            if !updated {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}