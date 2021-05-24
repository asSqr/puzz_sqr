use super::super::{FiniteSearchQueue, Grid, Symmetry, D, LP, P};
use super::*;

extern crate rand;

use rand::Rng;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Undecided,
    Line,
    Blank,
}

#[derive(Clone)]
pub struct AnswerField {
    height: i32,
    width: i32,
    chain_union: Grid<usize>,      // height * width
    chain_connectivity: Grid<i32>, // height * width
    chain_length: Grid<i32>,       // height * width
    field: Grid<Edge>,             // (2 * height - 1) * (2 * width - 1)
    seed_idx: Grid<i32>,
    seeds: Vec<LP>,
    seed_count: usize,
    endpoint_constraint: Grid<Endpoint>,
    endpoints: i32,
    endpoint_forced_cells: i32,
    chain_threshold: i32,
    forbid_adjacent_clue: bool,
    symmetry: Symmetry,
    invalid: bool,
    search_queue: FiniteSearchQueue,
}

#[derive(PartialEq, Eq)]
enum Cnt<T> {
    None,
    One(T),
    Many,
}

impl AnswerField {
    pub fn new(height: i32, width: i32, opt: &GeneratorOption) -> AnswerField {
        let mut ret = AnswerField {
            height,
            width,
            chain_union: Grid::new(height, width, 0),
            chain_connectivity: Grid::new(height, width, -1),
            chain_length: Grid::new(height, width, 0),
            field: Grid::new(2 * height - 1, 2 * width - 1, Edge::Undecided),
            seed_idx: Grid::new(2 * height - 1, 2 * width - 1, -1),
            seeds: vec![LP(0, 0); (height * width) as usize],
            seed_count: 0,
            endpoint_constraint: match opt.endpoint_constraint {
                Some(ep) => ep.clone(),
                None => Grid::new(height, width, Endpoint::Any),
            },
            endpoints: 0,
            endpoint_forced_cells: 0,
            chain_threshold: opt.chain_threshold,
            forbid_adjacent_clue: opt.forbid_adjacent_clue,
            symmetry: opt.symmetry,
            invalid: false,
            search_queue: FiniteSearchQueue::new((height * width) as usize),
        };

        for idx in 0..((height * width) as usize) {
            ret.chain_union[idx] = idx;
            if ret.endpoint_constraint[idx] == Endpoint::Forced {
                ret.endpoint_forced_cells += 1;
            }
        }

        ret.seeds[0] = LP(0, 0);
        ret.seeds[1] = LP(0, 2 * width - 2);
        ret.seeds[2] = LP(2 * height - 2, 0);
        ret.seeds[3] = LP(2 * height - 2, 2 * width - 2);
        ret.seed_count = 4;
        ret.seed_idx[LP(0, 0)] = 0;
        ret.seed_idx[LP(0, 2 * width - 2)] = 1;
        ret.seed_idx[LP(2 * height - 2, 0)] = 2;
        ret.seed_idx[LP(2 * height - 2, 2 * width - 2)] = 3;
        ret
    }

    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn width(&self) -> i32 {
        self.width
    }
    pub fn is_invalid(&self) -> bool {
        self.invalid
    }
    pub fn set_invalid(&mut self) {
        self.invalid = true;
    }
    pub fn endpoint_forced_cells(&self) -> i32 {
        self.endpoint_forced_cells
    }

    pub fn get_edge(&self, pos: LP) -> Edge {
        if self.field.is_valid_lp(pos) {
            self.field[pos]
        } else {
            Edge::Blank
        }
    }

    pub fn get_endpoint_constraint(&self, pos: P) -> Endpoint {
        self.endpoint_constraint[pos]
    }

    /// Counts the number of (Line, Undecided) around `cd`
    pub fn count_neighbor(&self, pos: LP) -> (i32, i32) {
        let mut line = 0;
        let mut undecided = 0;
        for &d in &FOUR_NEIGHBOURS {
            let e = self.get_edge(pos + d);
            if e == Edge::Line {
                line += 1;
            } else if e == Edge::Undecided {
                undecided += 1;
            }
        }
        (line, undecided)
    }

    /// Returns all neighbors whose state is `Undecided` around `cd`
    pub fn undecided_neighbors(&self, pos: LP) -> Vec<LP> {
        let mut ret = vec![];
        for &d in &FOUR_NEIGHBOURS {
            let pos2 = pos + d;
            let e = self.get_edge(pos2);
            if e == Edge::Undecided {
                ret.push(pos2);
            }
        }
        ret
    }

    fn undecided_neighbors_summary(&self, pos: LP) -> Cnt<LP> {
        let mut ret = Cnt::None;
        for &d in &FOUR_NEIGHBOURS {
            let pos2 = pos + d;
            let e = self.get_edge(pos2);
            if e == Edge::Undecided {
                ret = match ret {
                    Cnt::None => Cnt::One(pos2),
                    _ => return Cnt::Many,
                }
            }
        }
        ret
    }

    /// Returns whether vertex `cd` is a seed
    pub fn is_seed(&self, pos: LP) -> bool {
        let nb = self.count_neighbor(pos);
        nb == (0, 2) || (nb.0 == 1 && nb.1 > 0)
    }

    /// Copy `src` into this `AnswerField`.
    /// the shape of these `AnswerField`s must match.
    pub fn copy_from(&mut self, src: &AnswerField) {
        self.chain_union.copy_from(&src.chain_union);
        self.chain_connectivity.copy_from(&src.chain_connectivity);
        self.chain_length.copy_from(&src.chain_length);
        self.field.copy_from(&src.field);
        self.seed_idx.copy_from(&src.seed_idx);

        self.seeds[0..src.seed_count].copy_from_slice(&src.seeds[0..src.seed_count]);
        self.seed_count = src.seed_count;

        self.endpoint_constraint.copy_from(&src.endpoint_constraint);
        self.endpoints = src.endpoints;
        self.endpoint_forced_cells = src.endpoint_forced_cells;
        self.chain_threshold = src.chain_threshold;
        self.forbid_adjacent_clue = src.forbid_adjacent_clue;
        self.symmetry = src.symmetry;
        self.invalid = src.invalid;
    }

    /// Returns the representative node of the union containing `x` in `chain_connectivity`.
    /// Performs path compression to reduce complexity.
    fn root_mut(&mut self, x: usize) -> usize {
        if self.chain_connectivity[x] < 0 {
            x as usize
        } else {
            let parent = self.chain_connectivity[x] as usize;
            let ret = self.root_mut(parent);
            self.chain_connectivity[x] = ret as i32;
            ret
        }
    }

    /// Returns the representative node of the union containing `x` in `chain_connectivity`.
    fn root(&self, x: usize) -> usize {
        if self.chain_connectivity[x] < 0 {
            x as usize
        } else {
            let parent = self.chain_connectivity[x] as usize;
            self.root(parent)
        }
    }

    pub fn root_from_coord(&self, pos: P) -> usize {
        self.root(self.chain_connectivity.index_p(pos))
    }

    /// Join `x` and `y` in `chain_connectivity`
    fn join(&mut self, x: usize, y: usize) {
        let x = self.root_mut(x);
        let y = self.root_mut(y);
        if x != y {
            if self.chain_connectivity[x] < self.chain_connectivity[y] {
                self.chain_connectivity[x] += self.chain_connectivity[y];
                self.chain_connectivity[y] = x as i32;
            } else {
                self.chain_connectivity[y] += self.chain_connectivity[x];
                self.chain_connectivity[x] = y as i32;
            }
        }
    }

    /// Returns whether there is at least one seed
    pub fn has_seed(&self) -> bool {
        self.seed_count != 0
    }

    /// Returns a random seed using `rng`
    pub fn random_seed<R: Rng>(&self, rng: &mut R) -> LP {
        let idx = rng.gen_range(0, self.seed_count);
        self.seeds[idx]
    }

    fn complexity(&self, pos: LP) -> i32 {
        let LP(y, x) = pos;
        let ret = if y > 0 {
            4 - self.count_neighbor(pos + D(-2, 0)).1
        } else {
            0
        } + if x > 0 {
            4 - self.count_neighbor(pos + D(0, -2)).1
        } else {
            0
        } + if y < self.height * 2 - 2 {
            4 - self.count_neighbor(pos + D(2, 0)).1
        } else {
            0
        } + if x < self.width * 2 - 2 {
            4 - self.count_neighbor(pos + D(0, 2)).1
        } else {
            0
        };

        ret
    }

    /// Returns a seed with largest complexity among `k` samples
    pub fn best_seed<R: Rng>(&self, k: i32, rng: &mut R) -> LP {
        let mut seed = self.random_seed(rng);
        let mut complexity = self.complexity(seed);

        for _ in 1..k {
            let seed_cand = self.random_seed(rng);
            let complexity_cand = self.complexity(seed_cand);

            if complexity < complexity_cand {
                seed = seed_cand;
                complexity = complexity_cand;
            }
        }

        seed
    }

    /// Update `endpoint_constraint[cd]`.
    /// `cd` must be in vertex-coordinate.
    pub fn update_endpoint_constraint(&mut self, pos: P, constraint: Endpoint) {
        if !self.search_queue.is_started() {
            self.search_queue.start();
            self.update_endpoint_constraint_int(pos, constraint);
            self.queue_pop_all();
            self.search_queue.finish();
        } else {
            self.update_endpoint_constraint_int(pos, constraint);
        }
    }

    fn update_endpoint_constraint_int(&mut self, pos: P, constraint: Endpoint) {
        if self.endpoint_constraint[pos] == Endpoint::Any {
            self.endpoint_constraint[pos] = constraint;
            if constraint == Endpoint::Forced {
                self.endpoint_forced_cells += 1;
            }
            self.inspect(LP::of_vertex(pos));
        } else if self.endpoint_constraint[pos] != constraint {
            self.invalid = true;
        }
    }

    fn queue_pop_all(&mut self) {
        while !self.search_queue.empty() && !self.invalid {
            let idx = self.search_queue.pop();
            let pos = self.chain_connectivity.p(idx);
            self.inspect_int(LP::of_vertex(pos));
        }
        self.search_queue.clear();
    }

    pub fn decide(&mut self, pos: LP, state: Edge) {
        if !self.search_queue.is_started() {
            self.search_queue.start();
            self.decide_int(pos, state);
            self.queue_pop_all();
            self.search_queue.finish();
        } else {
            self.decide_int(pos, state);
        }
    }

    fn decide_int(&mut self, pos: LP, state: Edge) {
        let current = self.field[pos];
        if current != Edge::Undecided {
            if current != state {
                self.invalid = true;
            }
            return;
        }
        self.field[pos] = state;

        let LP(y, x) = pos;

        // update chain information
        if state == Edge::Line {
            let end1 = P(y / 2, x / 2);
            let end2 = P((y + 1) / 2, (x + 1) / 2);

            let end1_id = self.chain_union.index_p(end1);
            let end2_id = self.chain_union.index_p(end2);
            let another_end1_id = self.chain_union[end1_id];
            let another_end2_id = self.chain_union[end2_id];

            if another_end1_id == end2_id {
                // invalid: a self-loop will be formed
                self.invalid = true;
                return;
            }

            let new_length = self.chain_length[end1_id] + self.chain_length[end2_id] + 1;

            self.chain_union[another_end1_id] = another_end2_id;
            self.chain_union[another_end2_id] = another_end1_id;
            self.chain_length[another_end1_id] = new_length;
            self.chain_length[another_end2_id] = new_length;

            self.join(another_end1_id, another_end2_id);
            self.root_mut(another_end1_id);
            self.root_mut(another_end2_id);

            if new_length < self.chain_threshold {
                let pos = self.chain_union.p(another_end1_id);
                self.extend_chain(pos);
            }
        }

        // check incident vertices
        if y % 2 == 1 {
            if self.count_neighbor(pos + D(-1, 0)) == (1, 0) {
                self.endpoints += 1;
            }
            if self.count_neighbor(pos + D(1, 0)) == (1, 0) {
                self.endpoints += 1;
            }
            self.inspect(pos + D(-1, 0));
            self.inspect(pos + D(1, 0));
        } else {
            if self.count_neighbor(pos + D(0, -1)) == (1, 0) {
                self.endpoints += 1;
            }
            if self.count_neighbor(pos + D(0, 1)) == (1, 0) {
                self.endpoints += 1;
            }
            self.inspect(pos + D(0, -1));
            self.inspect(pos + D(0, 1));
        }

        // check for canonization rule
        if state == Edge::Line {
            if y % 2 == 1 {
                let related = [pos + D(0, -2), pos + D(-1, -1), pos + D(1, -1)];
                for i in 0..3 {
                    if self.get_edge(related[i]) == Edge::Line {
                        self.decide(related[(i + 1) % 3], Edge::Blank);
                        self.decide(related[(i + 2) % 3], Edge::Blank);
                    }
                }
                let related = [pos + D(0, 2), pos + D(-1, 1), pos + D(1, 1)];
                for i in 0..3 {
                    if self.get_edge(related[i]) == Edge::Line {
                        self.decide(related[(i + 1) % 3], Edge::Blank);
                        self.decide(related[(i + 2) % 3], Edge::Blank);
                    }
                }
            } else {
                let related = [pos + D(-2, 0), pos + D(-1, -1), pos + D(-1, 1)];
                for i in 0..3 {
                    if self.get_edge(related[i]) == Edge::Line {
                        self.decide(related[(i + 1) % 3], Edge::Blank);
                        self.decide(related[(i + 2) % 3], Edge::Blank);
                    }
                }
                let related = [pos + D(2, 0), pos + D(1, -1), pos + D(1, 1)];
                for i in 0..3 {
                    if self.get_edge(related[i]) == Edge::Line {
                        self.decide(related[(i + 1) % 3], Edge::Blank);
                        self.decide(related[(i + 2) % 3], Edge::Blank);
                    }
                }
            }
        }
    }

    /// Inspect all vertices
    pub fn inspect_all(&mut self) {
        assert_eq!(self.search_queue.is_started(), false);

        self.search_queue.start();

        let height = self.height;
        let width = self.width;

        for y in 0..height {
            for x in 0..width {
                self.inspect(LP(y * 2, x * 2));
            }
        }

        self.queue_pop_all();
        self.search_queue.finish();
    }

    /// Inspect vertex (y, x)
    fn inspect(&mut self, pos: LP) {
        assert_eq!(self.search_queue.is_started(), true);

        self.search_queue
            .push(self.chain_connectivity.index_p(pos.as_vertex()));
    }

    fn inspect_int(&mut self, pos: LP) {
        let (line, undecided) = self.count_neighbor(pos);
        if line == 0 {
            if undecided == 0 {
                self.invalid = true;
                return;
            }
            if undecided == 1 {
                for &d in &FOUR_NEIGHBOURS {
                    let e = self.get_edge(pos + d);
                    if e == Edge::Undecided {
                        self.decide(pos + d, Edge::Line);
                    }
                }
            }
        } else if line == 2 {
            for &d in &FOUR_NEIGHBOURS {
                let e = self.get_edge(pos + d);
                if e == Edge::Undecided {
                    self.decide(pos + d, Edge::Blank);
                }
            }
        } else if line == 1 {
            // avoid too short chains
            if self.chain_length[pos.as_vertex()] < self.chain_threshold {
                self.extend_chain(pos.as_vertex());

                let a = self.chain_union.p(self.chain_union[pos.as_vertex()]);
                if self.count_neighbor(LP::of_vertex(a)) == (1, 0) {
                    let minimum_len = self.chain_threshold - self.chain_length[pos.as_vertex()];
                    for &d in &FOUR_NEIGHBOURS {
                        if self.get_edge(pos + d) == Edge::Undecided {
                            let a = self.chain_union.p(self.chain_union[pos.as_vertex() + d]);
                            if self.count_neighbor(LP::of_vertex(a)) == (1, 0)
                                && self.chain_length[pos.as_vertex() + d] < minimum_len
                            {
                                self.decide(pos + d, Edge::Blank);
                            }
                        }
                    }
                }
            }
        } else if line >= 3 {
            self.invalid = true;
            return;
        }

        if line == 1 && undecided == 0 {
            if self.get_endpoint_constraint(pos.as_vertex()) == Endpoint::Prohibited {
                self.invalid = true;
                return;
            }
            if self.endpoint_constraint[pos.as_vertex()] == Endpoint::Any {
                self.endpoint_constraint[pos.as_vertex()] = Endpoint::Forced;
                self.endpoint_forced_cells += 1;
            }
        }
        if line == 2 {
            if self.get_endpoint_constraint(pos.as_vertex()) == Endpoint::Forced {
                self.invalid = true;
                return;
            }
            if self.endpoint_constraint[pos.as_vertex()] == Endpoint::Any {
                self.endpoint_constraint[pos.as_vertex()] = Endpoint::Prohibited;
            }
        }

        if self.forbid_adjacent_clue
            && (self.get_endpoint_constraint(pos.as_vertex()) == Endpoint::Forced
                || (line == 1 && undecided == 0))
        {
            let LP(y, x) = pos;
            for dy in -1..2 {
                for dx in -1..2 {
                    if dy == 0 && dx == 0 {
                        continue;
                    }
                    if y / 2 + dy < 0
                        || y / 2 + dy >= self.height
                        || x / 2 + dx < 0
                        || x / 2 + dx >= self.width
                    {
                        continue;
                    }
                    self.update_endpoint_constraint(
                        P(y / 2 + dy, x / 2 + dx),
                        Endpoint::Prohibited,
                    );
                }
            }
        }
        if self.forbid_adjacent_clue && line + undecided == 2 {
            for &d in &FOUR_NEIGHBOURS {
                if self.get_edge(pos + d) != Edge::Blank {
                    let nb = self.count_neighbor(pos + d * 2);
                    if nb.0 + nb.1 == 2 {
                        self.decide(pos + d, Edge::Line);
                    }
                }
            }
        }

        let con = self.get_endpoint_constraint(pos.as_vertex());
        if con != Endpoint::Any {
            let height = self.height;
            let width = self.width;
            let LP(y, x) = pos;
            if self.symmetry.tetrad {
                self.update_endpoint_constraint(P(x / 2, width - 1 - y / 2), con);
            } else if self.symmetry.dyad {
                self.update_endpoint_constraint(P(height - 1 - y / 2, width - 1 - x / 2), con);
            }
            if self.symmetry.horizontal {
                self.update_endpoint_constraint(P(height - 1 - y / 2, x / 2), con);
            }
            if self.symmetry.vertical {
                self.update_endpoint_constraint(P(y / 2, width - 1 - x / 2), con);
            }
        }

        match self.get_endpoint_constraint(pos.as_vertex()) {
            Endpoint::Any => (),
            Endpoint::Forced => {
                if line == 1 {
                    for &d in &FOUR_NEIGHBOURS {
                        let e = self.get_edge(pos + d);
                        if e == Edge::Undecided {
                            self.decide(pos + d, Edge::Blank);
                        }
                    }
                } else if line >= 2 {
                    self.invalid = true;
                }
            }
            Endpoint::Prohibited => {
                if line == 1 {
                    if undecided == 0 {
                        self.invalid = true;
                        return;
                    } else if undecided == 1 {
                        for &d in &FOUR_NEIGHBOURS {
                            let e = self.get_edge(pos + d);
                            if e == Edge::Undecided {
                                self.decide(pos + d, Edge::Line);
                            }
                        }
                    }
                } else if line == 0 && undecided == 2 {
                    for &d in &FOUR_NEIGHBOURS {
                        let e = self.get_edge(pos + d);
                        if e == Edge::Undecided {
                            self.decide(pos + d, Edge::Line);
                        }
                    }
                }
            }
        }

        let is_seed = self.is_seed(pos);
        let seed_idx = self.seed_idx[pos];

        if seed_idx != -1 && !is_seed {
            // (y, x) is no longer a seed
            let moved = self.seeds[self.seed_count - 1];
            self.seed_idx[moved] = seed_idx;
            self.seeds[seed_idx as usize] = moved;
            self.seed_count -= 1;
            self.seed_idx[pos] = -1;
        } else if seed_idx == -1 && is_seed {
            // (y, x) is now a seed
            self.seed_idx[pos] = self.seed_count as i32;
            self.seeds[self.seed_count] = pos;
            self.seed_count += 1;
        }
    }

    /// Extend the chain one of whose endpoint is `(y, x)`
    fn extend_chain(&mut self, pos: P) {
        let end1_id = self.chain_union.index_p(pos);
        let end2_id = self.chain_union[end1_id];

        let end1 = LP::of_vertex(pos);
        let end2_vertex = self.chain_union.p(end2_id);
        let end2 = LP::of_vertex(end2_vertex);

        let end1_undecided = self.undecided_neighbors_summary(end1);
        let end2_undecided = self.undecided_neighbors_summary(end2);

        if end1_undecided == Cnt::None {
            let con = self.endpoint_constraint[end2_vertex];
            match con {
                Endpoint::Forced => {
                    self.invalid = true;
                    return;
                }
                Endpoint::Any => {
                    self.endpoint_constraint[end2_vertex] = Endpoint::Prohibited;
                    self.inspect(end2);
                }
                Endpoint::Prohibited => (),
            }
        }
        if end2_undecided == Cnt::None {
            let con = self.endpoint_constraint[pos];
            match con {
                Endpoint::Forced => {
                    self.invalid = true;
                    return;
                }
                Endpoint::Any => {
                    self.endpoint_constraint[pos] = Endpoint::Prohibited;
                    self.inspect(end1);
                }
                Endpoint::Prohibited => (),
            }
        }
        match (end1_undecided, end2_undecided) {
            (Cnt::None, Cnt::None) => {
                self.invalid = true;
                return;
            }
            (Cnt::None, Cnt::One(e)) | (Cnt::One(e), Cnt::None) => self.decide(e, Edge::Line),
            _ => (),
        }
    }

    pub fn forbid_further_endpoint(&mut self) {
        // this function should not be called from internal functions
        assert_eq!(self.search_queue.is_started(), false);

        self.search_queue.start();
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = P(y, x);
                if self.endpoint_constraint[pos] == Endpoint::Any {
                    self.update_endpoint_constraint(pos, Endpoint::Prohibited);
                }
            }
        }
        self.queue_pop_all();
        self.search_queue.finish();
    }

    /// Convert into `LinePlacement`
    pub fn as_line_placement(&self) -> LinePlacement {
        let height = self.height;
        let width = self.width;
        let mut ret = LinePlacement::new(height, width);

        for y in 0..height {
            for x in 0..width {
                if y < height - 1 {
                    if self.get_edge(LP(y * 2 + 1, x * 2)) == Edge::Line {
                        ret.set_down(P(y, x), true);
                    }
                }
                if x < width - 1 {
                    if self.get_edge(LP(y * 2, x * 2 + 1)) == Edge::Line {
                        ret.set_right(P(y, x), true);
                    }
                }
            }
        }

        ret
    }
}

impl fmt::Debug for AnswerField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let height = self.height;
        let width = self.width;

        for y in 0..(2 * height - 1) {
            for x in 0..(2 * width - 1) {
                match (y % 2, x % 2) {
                    (0, 0) => write!(
                        f,
                        "{}",
                        match self.endpoint_constraint[P(y / 2, x / 2)] {
                            Endpoint::Any => '#',
                            Endpoint::Forced => '*',
                            Endpoint::Prohibited => '+',
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