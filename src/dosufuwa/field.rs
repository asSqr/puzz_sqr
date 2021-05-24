use super::super::{Grid, D, P};
use super::Cell;

#[derive(Clone)]
struct Area {
    num_cand: usize,
    xor_cand: usize,
}

#[derive(Clone)]
pub struct Field {
    cell: Grid<Cell>,
    area_id: Grid<usize>,
    maybe_balloon: Grid<bool>,
    maybe_iron: Grid<bool>,
    area_cells: Vec<Vec<P>>,
    areas_balloon: Vec<Area>,
    areas_iron: Vec<Area>,
    num_decided: i32,
    inconsistent: bool,
}

impl Field {
    pub fn new(is_black: &Grid<bool>, areas: &Vec<Vec<P>>) -> Field {
        let height = is_black.height();
        let width = is_black.width();
        let mut area_id = Grid::new(height, width, !0);
        let mut areas_balloon_iron = vec![];
        for i in 0..areas.len() {
            let mut xor_cand = 0;
            for &p in &areas[i] {
                area_id[p] = i;
                xor_cand ^= area_id.index_p(p);
            }
            areas_balloon_iron.push(Area {
                num_cand: areas[i].len(),
                xor_cand,
            });
        }
        let mut cell = Grid::new(height, width, Cell::Undecided);
        let mut maybe_balloon_iron = Grid::new(height, width, true);
        let mut num_decided = 0;
        for y in 0..height {
            for x in 0..width {
                if is_black[P(y, x)] {
                    cell[P(y, x)] = Cell::Black;
                    maybe_balloon_iron[P(y, x)] = false;
                    num_decided += 1;
                }
            }
        }
        Field {
            cell,
            area_id,
            maybe_balloon: maybe_balloon_iron.clone(),
            maybe_iron: maybe_balloon_iron.clone(),
            area_cells: areas.clone(),
            areas_balloon: areas_balloon_iron.clone(),
            areas_iron: areas_balloon_iron.clone(),
            num_decided,
            inconsistent: false,
        }
    }

    pub fn height(&self) -> i32 {
        self.cell.height()
    }
    pub fn width(&self) -> i32 {
        self.cell.width()
    }
    pub fn inconsistent(&self) -> bool {
        self.inconsistent
    }
    pub fn set_inconsistent(&mut self) {
        self.inconsistent = true;
    }
    pub fn fully_solved(&self) -> bool {
        self.num_decided == self.height() * self.width()
    }
    pub fn num_decided(&self) -> i32 {
        self.num_decided
    }

    fn inspect_area_balloon(&mut self, id: usize) {
        let area = &self.areas_balloon[id];
        if area.num_cand == 0 {
            self.set_inconsistent();
            return;
        } else if area.num_cand == 1 {
            self.decide_balloon(self.cell.p(area.xor_cand));
        }
    }
    pub fn decide_balloon(&mut self, pos: P) {
        let cell = self.cell[pos];
        if cell == Cell::Balloon {
            return;
        }
        if cell != Cell::Undecided {
            self.set_inconsistent();
            return;
        }
        self.cell[pos] = Cell::Balloon;
        self.num_decided += 1;
        self.decide_no_iron(pos);
        let area_id = self.area_id[pos];
        for i in 0..self.area_cells[area_id].len() {
            let p = self.area_cells[area_id][i];
            if p != pos {
                self.decide_no_balloon(p);
            }
        }
        let P(y, x) = pos;
        if 0 < y && self.cell[P(y - 1, x)] != Cell::Black {
            self.decide_balloon(P(y - 1, x));
        }
    }
    pub fn decide_no_balloon(&mut self, pos: P) {
        let cell = self.cell[pos];
        if cell == Cell::Balloon {
            return;
        }
        if !self.maybe_balloon[pos] {
            return;
        }
        self.maybe_balloon[pos] = false;
        if !self.maybe_iron[pos] {
            self.cell[pos] = Cell::Empty;
            self.num_decided += 1;
        }
        let area_id = self.area_id[pos];
        self.areas_balloon[area_id].num_cand -= 1;
        self.areas_balloon[area_id].xor_cand ^= self.cell.index_p(pos);
        self.inspect_area_balloon(area_id);
        let P(y, x) = pos;
        if y < self.height() - 1 && self.cell[P(y + 1, x)] != Cell::Black {
            self.decide_no_balloon(P(y + 1, x));
        }
    }

    fn inspect_area_iron(&mut self, id: usize) {
        let area = &self.areas_iron[id];
        if area.num_cand == 0 {
            self.set_inconsistent();
            return;
        } else if area.num_cand == 1 {
            self.decide_iron(self.cell.p(area.xor_cand));
        }
    }
    pub fn decide_iron(&mut self, pos: P) {
        let cell = self.cell[pos];
        if cell == Cell::Iron {
            return;
        }
        if cell != Cell::Undecided {
            self.set_inconsistent();
            return;
        }
        self.cell[pos] = Cell::Iron;
        self.num_decided += 1;
        self.decide_no_balloon(pos);

        let area_id = self.area_id[pos];
        for i in 0..self.area_cells[area_id].len() {
            let p = self.area_cells[area_id][i];
            if p != pos {
                self.decide_no_iron(p);
            }
        }
        let P(y, x) = pos;
        if y < self.height() - 1 && self.cell[P(y + 1, x)] != Cell::Black {
            self.decide_iron(P(y + 1, x));
        }
    }
    pub fn decide_no_iron(&mut self, pos: P) {
        let cell = self.cell[pos];
        if cell == Cell::Iron {
            return;
        }
        if !self.maybe_iron[pos] {
            return;
        }
        self.maybe_iron[pos] = false;
        if !self.maybe_balloon[pos] {
            self.cell[pos] = Cell::Empty;
            self.num_decided += 1;
        }
        let area_id = self.area_id[pos];
        self.areas_iron[area_id].num_cand -= 1;
        self.areas_iron[area_id].xor_cand ^= self.cell.index_p(pos);
        self.inspect_area_iron(area_id);
        let P(y, x) = pos;
        if 0 < y && self.cell[P(y - 1, x)] != Cell::Black {
            self.decide_no_iron(P(y - 1, x));
        }
    }
    pub fn inspect_initial(&mut self) {
        let height = self.height();
        let width = self.width();

        for y in 0..height {
            for x in 0..width {
                let mut y2 = y + 1;
                while y2 < height && self.cell[P(y2, x)] != Cell::Black {
                    if self.area_id[P(y, x)] == self.area_id[P(y2, x)] {
                        self.decide_no_iron(P(y, x));
                        self.decide_no_balloon(P(y2, x));
                    }
                    y2 += 1;
                }
            }
        }
    }
    pub fn solve(&mut self) {
        // do nothing
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
            for y in 0..height {
                for x in 0..width {
                    let pos = P(y, x);
                    if self.cell[pos] != Cell::Undecided {
                        continue;
                    }
                    if self.maybe_balloon[pos] {
                        {
                            let mut field_balloon = self.clone();
                            field_balloon.decide_balloon(pos);
                            field_balloon.trial_and_error(depth - 1);

                            if field_balloon.inconsistent() {
                                updated = true;
                                self.decide_no_balloon(pos);
                                self.trial_and_error(depth - 1);
                            }
                        }
                        {
                            let mut field_no_balloon = self.clone();
                            field_no_balloon.decide_no_balloon(pos);
                            field_no_balloon.trial_and_error(depth - 1);

                            if field_no_balloon.inconsistent() {
                                updated = true;
                                self.decide_balloon(pos);
                                self.trial_and_error(depth - 1);
                            }
                        }
                    }
                    if self.maybe_iron[pos] {
                        {
                            let mut field_iron = self.clone();
                            field_iron.decide_iron(pos);
                            field_iron.trial_and_error(depth - 1);

                            if field_iron.inconsistent() {
                                updated = true;
                                self.decide_no_iron(pos);
                                self.trial_and_error(depth - 1);
                            }
                        }
                        {
                            let mut field_no_iron = self.clone();
                            field_no_iron.decide_no_iron(pos);
                            field_no_iron.trial_and_error(depth - 1);

                            if field_no_iron.inconsistent() {
                                updated = true;
                                self.decide_iron(pos);
                                self.trial_and_error(depth - 1);
                            }
                        }
                    }
                    if self.inconsistent() {
                        return;
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

    fn to_areas<CellId: AsRef<[Row]>, Row: AsRef<[i32]>>(cell_id: &CellId) -> Vec<Vec<P>> {
        let mut ret = vec![];
        for (y, row) in cell_id.as_ref().iter().enumerate() {
            for (x, &id) in row.as_ref().iter().enumerate() {
                if id >= 0 {
                    while ret.len() as i32 <= id {
                        ret.push(vec![]);
                    }
                    ret[id as usize].push(P(y as i32, x as i32));
                }
            }
        }
        ret
    }
    fn extract_is_black(height: i32, width: i32, areas: &Vec<Vec<P>>) -> Grid<bool> {
        let mut ret = Grid::new(height, width, true);
        for area in areas {
            for &p in area {
                ret[p] = false;
            }
        }
        ret
    }

    #[test]
    fn test_dosufuwa_simple() {
        {
            // https://puzsq.jp/main/puzzle_play.php?pid=10159
            let height = 8;
            let width = 8;
            let cell_id = [
                [0, 0, 0, 1, 1, -1, 2, 2],
                [3, -1, 1, 1, 1, 1, 2, 2],
                [3, 4, 4, 4, 4, 2, 2, 2],
                [3, -1, 5, -1, 4, 6, 6, 7],
                [3, 5, 5, 5, -1, 6, -1, 7],
                [3, 5, 8, 8, 9, 9, 7, 7],
                [3, 5, 8, 10, 9, 9, -1, 7],
                [3, 3, -1, 10, 10, 11, 11, 11],
            ];
            let areas = to_areas(&cell_id);
            let is_black = extract_is_black(height, width, &areas);

            let mut field = Field::new(&is_black, &areas);
            field.inspect_initial();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.fully_solved(), true);
        }
        {
            // https://puzsq.jp/main/puzzle_play.php?pid=10182
            let height = 8;
            let width = 8;
            let cell_id = [
                [0, -1, 1, 1, 1, -1, 2, -1],
                [0, 0, 0, 0, 2, 2, 2, 2],
                [3, 3, 3, 4, 5, 5, 5, 5],
                [-1, 3, -1, 4, 4, 6, -1, 5],
                [7, 7, 7, -1, 6, 6, 8, 5],
                [9, 9, 9, 10, -1, 6, 8, 5],
                [11, 11, 10, 10, 10, 10, 8, 8],
                [-1, 11, 11, -1, 8, 8, 8, -1],
            ];
            let areas = to_areas(&cell_id);
            let is_black = extract_is_black(height, width, &areas);

            let mut field = Field::new(&is_black, &areas);
            field.inspect_initial();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.fully_solved(), false);

            field.trial_and_error(1);
            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.fully_solved(), true);
        }
    }
}