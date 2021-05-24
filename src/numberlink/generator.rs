use super::super::{Grid, Symmetry, D, LP, P};
use super::*;

extern crate rand;

use rand::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endpoint {
    Any,
    Forced,
    Prohibited,
}

pub struct GeneratorOption<'a> {
    pub chain_threshold: i32,
    pub endpoint_constraint: Option<&'a Grid<Endpoint>>,
    pub forbid_adjacent_clue: bool,
    pub symmetry: Symmetry,
    pub clue_limit: Option<i32>,
    pub prioritized_extension: bool,
}

pub fn generate_endpoint_constraint<R: Rng>(
    height: i32,
    width: i32,
    empty_width: i32,
    corner_constraint: Option<(i32, i32)>,
    symmetry: Symmetry,
    rng: &mut R,
) -> Grid<Endpoint> {
    let mut ret = Grid::new(height, width, Endpoint::Any);
    for d in 0..empty_width {
        for y in 0..height {
            ret[P(y, d)] = Endpoint::Prohibited;
            ret[P(y, width - 1 - d)] = Endpoint::Prohibited;
        }
        for x in 0..width {
            ret[P(d, x)] = Endpoint::Prohibited;
            ret[P(height - 1 - d, x)] = Endpoint::Prohibited;
        }
    }
    if let Some((lo, hi)) = corner_constraint {
        // upper left, upper right, lower left, lower right
        let mut corner_positions = [-1, -1, -1, -1];
        for i in 0..4 {
            if corner_positions[i] != -1 {
                continue;
            }
            corner_positions[i] = rng.gen_range(lo, hi + 1);
            if symmetry.tetrad || symmetry.vertical || (symmetry.dyad && symmetry.horizontal) {
                corner_positions[i ^ 1] = corner_positions[i];
            }
            if symmetry.tetrad || symmetry.horizontal || (symmetry.dyad && symmetry.vertical) {
                corner_positions[i ^ 2] = corner_positions[i];
            }
            if symmetry.dyad || symmetry.tetrad || (symmetry.vertical && symmetry.horizontal) {
                corner_positions[i ^ 3] = corner_positions[i];
            }
        }
        for i in 0..4 {
            let y = if (i & 2) == 0 {
                corner_positions[i]
            } else {
                height - 1 - corner_positions[i]
            };
            let x = if (i & 1) == 0 {
                corner_positions[i]
            } else {
                width - 1 - corner_positions[i]
            };
            ret[P(y, x)] = Endpoint::Forced;
        }
    }
    ret
}

/// A type for an update of `AnswerField`.
/// - `Corner(e, f)`: both `e` and `f` must be `Line` to make a corner.
/// - `Endpoint(e, f)`: `e` must be `Line` but `f` must be `Blank` to make an endpoint.
/// - `Extend(e)`: `e` must be `Line` to extend an existing chain.
#[derive(Clone, Copy)]
enum FieldUpdate {
    Corner(LP, LP),
    Endpoint(LP, LP),
    Extend(LP),
}

pub struct PlacementGenerator {
    pool: Vec<AnswerField>,
    active_fields: Vec<AnswerField>,
    next_fields: Vec<AnswerField>,
    height: i32,
    width: i32,
    beam_width: usize,
}

impl PlacementGenerator {
    pub fn new(height: i32, width: i32) -> PlacementGenerator {
        let template = AnswerField::new(
            height,
            width,
            &GeneratorOption {
                chain_threshold: 1,
                endpoint_constraint: None,
                forbid_adjacent_clue: false,
                symmetry: Symmetry::none(),
                clue_limit: None,
                prioritized_extension: false,
            },
        );
        let beam_width = 100;
        PlacementGenerator {
            pool: vec![template; beam_width * 2 + 1],
            active_fields: Vec::with_capacity(beam_width),
            next_fields: Vec::with_capacity(beam_width),
            height,
            width,
            beam_width,
        }
    }
    pub fn generate<R: Rng>(
        &mut self,
        opt: &GeneratorOption,
        rng: &mut R,
    ) -> Option<LinePlacement> {
        let beam_width = self.beam_width;
        let height = self.height;
        let width = self.width;
        let fields = &mut self.active_fields;

        let symmetry = Symmetry {
            dyad: opt.symmetry.dyad || opt.symmetry.tetrad,
            tetrad: opt.symmetry.tetrad && (height == width),
            ..opt.symmetry
        };
        let mut endpoint_constraint = match opt.endpoint_constraint {
            Some(e) => e.clone(),
            None => Grid::new(height, width, Endpoint::Any),
        };
        if symmetry.dyad && height % 2 == 1 && width % 2 == 1 {
            endpoint_constraint[P(height / 2, width / 2)] = Endpoint::Prohibited;
        }
        if opt.forbid_adjacent_clue {
            if symmetry.horizontal && height % 2 == 0 {
                for x in 0..width {
                    endpoint_constraint[P(height / 2, x)] = Endpoint::Prohibited;
                    endpoint_constraint[P(height / 2 + 1, x)] = Endpoint::Prohibited;
                }
            }
            if symmetry.horizontal && width % 2 == 0 {
                for y in 0..height {
                    endpoint_constraint[P(y, width / 2)] = Endpoint::Prohibited;
                    endpoint_constraint[P(y, width / 2 + 1)] = Endpoint::Prohibited;
                }
            }
            if symmetry.dyad {
                endpoint_constraint[P(height / 2, width / 2)] = Endpoint::Prohibited;
                endpoint_constraint[P(height / 2, (width - 1) / 2)] = Endpoint::Prohibited;
                endpoint_constraint[P((height - 1) / 2, width / 2)] = Endpoint::Prohibited;
                endpoint_constraint[P((height - 1) / 2, (width - 1) / 2)] = Endpoint::Prohibited;
            }
        }
        let opt = GeneratorOption {
            endpoint_constraint: Some(&endpoint_constraint),
            symmetry,
            ..*opt
        };

        let template = AnswerField::new(height, width, &opt);

        let mut field_base = self.pool.pop().unwrap();
        field_base.copy_from(&template);

        field_base.inspect_all();

        fields.push(field_base);

        loop {
            if fields.len() == 0 {
                break;
            }

            let fields_next = &mut self.next_fields;
            'outer: for _ in 0..(5 * fields.len()) {
                if fields_next.len() >= beam_width || fields.len() == 0 {
                    break;
                }

                let id = rng.gen_range(0, fields.len());

                if fields[id].is_invalid() || !fields[id].has_seed() {
                    self.pool.push(fields.swap_remove(id));
                    continue;
                }

                let mut field = self.pool.pop().unwrap();
                field.copy_from(&fields[id]);

                if !field.has_seed() {
                    continue;
                }
                let cd = if opt.prioritized_extension {
                    field.best_seed(5, rng)
                } else {
                    field.random_seed(rng)
                };

                let update = PlacementGenerator::choose_update(&field, cd, rng);
                PlacementGenerator::apply_update(&mut field, update);
                PlacementGenerator::check_invalidity(&mut field, &opt);

                if field.is_invalid() {
                    self.pool.push(field);
                    PlacementGenerator::deny_update(&mut fields[id], cd, update);
                    PlacementGenerator::check_invalidity(&mut fields[id], &opt);
                    if fields[id].is_invalid() {
                        self.pool.push(fields.swap_remove(id));
                    }
                    continue;
                }

                if !field.has_seed() {
                    if !check_answer_validity(&field) {
                        self.pool.push(field);
                        continue 'outer;
                    }

                    let line_placement = field.as_line_placement();

                    self.pool.push(field);
                    // release used fields
                    for used in fields.drain(0..) {
                        self.pool.push(used);
                    }
                    for used in fields_next.drain(0..) {
                        self.pool.push(used);
                    }

                    return Some(line_placement);
                }

                fields_next.push(field);
            }

            // release old fields
            for old in fields.drain(0..) {
                self.pool.push(old);
            }

            ::std::mem::swap(fields, fields_next);
        }
        None
    }
    pub fn generate_and_test<R: Rng>(
        &mut self,
        opt: &GeneratorOption,
        rng: &mut R,
    ) -> Option<Grid<Clue>> {
        if let Some(placement) = self.generate(opt, rng) {
            if uniqueness_pretest(&placement) {
                let problem = extract_problem(&placement, rng);
                let ans = solve2(&problem, Some(2), false, true);
                if ans.len() == 1 && !ans.found_not_fully_filled {
                    return Some(problem);
                }
            }
        }
        None
    }
    fn check_invalidity(field: &mut AnswerField, opt: &GeneratorOption) {
        if field.is_invalid() {
            return;
        }
        if let Some(limit) = opt.clue_limit {
            limit_clue_number(field, limit);
            if field.is_invalid() {
                return;
            }
        }
        if is_entangled(field) {
            field.set_invalid();
            return;
        }
        // TODO: better check for other symmetry types?
        if opt.symmetry.dyad && check_symmetry(field) {
            field.set_invalid();
        }
    }
    fn choose_update<R: Rng>(field: &AnswerField, pos: LP, rng: &mut R) -> FieldUpdate {
        let pos_vtx = pos.as_vertex();
        let nbs = field.undecided_neighbors(pos);

        if field.count_neighbor(pos) == (0, 2) {
            let constraint = field.get_endpoint_constraint(pos_vtx);

            if constraint != Endpoint::Forced && rng.gen::<f64>() < 0.9f64 {
                FieldUpdate::Corner(nbs[0], nbs[1])
            } else {
                let i = rng.gen_range(0, 2);
                FieldUpdate::Endpoint(nbs[i], nbs[1 - i])
            }
        } else {
            let i = rng.gen_range(0, nbs.len());
            FieldUpdate::Extend(nbs[i])
        }
    }
    fn apply_update(field: &mut AnswerField, update: FieldUpdate) {
        match update {
            FieldUpdate::Corner(e, f) => {
                field.decide(e, Edge::Line);
                field.decide(f, Edge::Line);
            }
            FieldUpdate::Endpoint(e, f) => {
                field.decide(e, Edge::Line);
                field.decide(f, Edge::Blank);
            }
            FieldUpdate::Extend(e) => field.decide(e, Edge::Line),
        }
    }
    fn deny_update(field: &mut AnswerField, pos: LP, update: FieldUpdate) {
        match update {
            FieldUpdate::Corner(_, _) => {
                field.update_endpoint_constraint(pos.as_vertex(), Endpoint::Forced);
            }
            FieldUpdate::Endpoint(e, _) => field.decide(e, Edge::Blank),
            FieldUpdate::Extend(e) => field.decide(e, Edge::Blank),
        }
    }
}

fn is_entangled(field: &AnswerField) -> bool {
    let height = field.height();
    let width = field.width();

    let mut entangled_pairs = vec![];

    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            if field.get_endpoint_constraint(P(y, x)) == Endpoint::Forced {
                let pos = LP(y * 2, x * 2);
                for &d in &FOUR_NEIGHBOURS {
                    if field.get_edge(pos + d) != Edge::Line {
                        continue;
                    }
                    let dr = d.rotate_clockwise();
                    if field.get_edge(pos + dr * 2 - d) == Edge::Line
                        && field.get_edge(pos - dr * 2 - d) == Edge::Line
                        && field.get_edge(pos + dr - d * 2) == Edge::Line
                        && field.get_edge(pos - dr - d * 2) == Edge::Line
                        && (field.get_edge(pos + dr * 2 + d) == Edge::Line
                            || field.get_edge(pos + dr + d * 2) == Edge::Line)
                        && (field.get_edge(pos - dr * 2 + d) == Edge::Line
                            || field.get_edge(pos - dr + d * 2) == Edge::Line)
                    {
                        let u = field.root_from_coord(P(y, x));
                        let v = field.root_from_coord(P(y, x) - d);
                        if u < v {
                            entangled_pairs.push((u, v));
                        } else {
                            entangled_pairs.push((v, u));
                        }
                    }
                }
            }
        }
    }

    entangled_pairs.sort();

    for i in 1..entangled_pairs.len() {
        if entangled_pairs[i - 1] == entangled_pairs[i] {
            return true;
        }
    }

    false
}

/// Extract a problem from `placement`.
/// Clue numbers are randomly assigned using `rng`.
pub fn extract_problem<R: Rng>(placement: &LinePlacement, rng: &mut R) -> Grid<Clue> {
    let height = placement.height();
    let width = placement.width();
    let groups = match placement.extract_chain_groups() {
        Some(groups) => groups,
        None => panic!(),
    };

    let mut max_id = 0;
    for y in 0..height {
        for x in 0..width {
            max_id = ::std::cmp::max(max_id, groups[P(y, x)]);
        }
    }

    let mut shuffler = vec![0; (max_id + 1) as usize];
    for i in 0..(max_id + 1) {
        shuffler[i as usize] = i;
    }
    rng.shuffle(&mut shuffler);

    let mut ret = Grid::new(height, width, NO_CLUE);
    for y in 0..height {
        for x in 0..width {
            let pos = P(y, x);
            if placement.is_endpoint(pos) {
                ret[pos] = Clue(1 + shuffler[groups[pos] as usize]);
            }
        }
    }

    ret
}

/// Check whether the problem obtained from `placement` *may have* unique solution.
/// If `false` is returned, the problem is guaranteed to have several solutions.
/// However, even if `true` is returned, it is still possible that the problem has several solutions.
pub fn uniqueness_pretest(placement: &LinePlacement) -> bool {
    let height = placement.height();
    let width = placement.width();
    let ids = match placement.extract_chain_groups() {
        Some(ids) => ids,
        None => return false,
    };

    if !uniqueness_pretest_horizontal(&ids) {
        return false;
    }
    if height == width {
        let mut ids_fliped = Grid::new(width, height, -1);
        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                ids_fliped[pos] = ids[pos];
            }
        }

        if !uniqueness_pretest_horizontal(&ids_fliped) {
            return false;
        }
    }

    true
}
fn uniqueness_pretest_horizontal(ids: &Grid<i32>) -> bool {
    let height = ids.height();
    let width = ids.width();

    let mut max_id = 0;
    for y in 0..height {
        for x in 0..width {
            max_id = ::std::cmp::max(max_id, ids[P(y, x)]);
        }
    }

    let mut positions = vec![vec![]; (max_id + 1) as usize];
    for y in 0..height {
        for x in 0..width {
            let pos = P(y, x);
            positions[ids[pos] as usize].push(pos);
        }
    }

    for mode in 0..2 {
        let mut checked = vec![false; (max_id + 1) as usize];
        let mut screen_problem = Grid::new(height, width, UNUSED);
        let mut used_cells = 0;
        for x in 0..width {
            let x = if mode == 0 { x } else { width - 1 - x };
            for y in 0..height {
                let pos = P(y, x);
                let i = ids[pos];
                if !checked[i as usize] {
                    for &loc in &positions[i as usize] {
                        let P(y, x) = loc;
                        let is_endpoint = 1
                            == (if y > 0 && ids[loc] == ids[loc + D(-1, 0)] {
                                1
                            } else {
                                0
                            } + if x > 0 && ids[loc] == ids[loc + D(0, -1)] {
                                1
                            } else {
                                0
                            } + if y < height - 1 && ids[loc] == ids[loc + D(1, 0)] {
                                1
                            } else {
                                0
                            } + if x < width - 1 && ids[loc] == ids[loc + D(0, 1)] {
                                1
                            } else {
                                0
                            });
                        screen_problem[loc] = if is_endpoint { Clue(i + 1) } else { NO_CLUE };
                    }
                    checked[i as usize] = true;
                    used_cells += positions[i as usize].len() as i32;
                }
            }
            if used_cells >= height * width / 2 {
                break;
            }
        }

        let ans = solve2(&screen_problem, Some(2), false, true);
        if ans.len() >= 2 || ans.found_not_fully_filled {
            return false;
        }
    }
    return true;
}

/// Check whether `field` is valid.
/// A field is considered invalid if it contains a self-touching line.
fn check_answer_validity(field: &AnswerField) -> bool {
    let height = field.height();
    let width = field.width();
    let mut ids = Grid::new(height, width, -1);
    let mut id = 1;
    for y in 0..height {
        for x in 0..width {
            let pos = P(y, x);
            if ids[pos] == -1 {
                fill_line_id(pos, &field, &mut ids, id);
                id += 1;
            }
        }
    }

    let mut end_count = vec![0; id as usize];
    for y in 0..height {
        for x in 0..width {
            if field.count_neighbor(LP(y * 2, x * 2)) == (1, 0) {
                end_count[ids[P(y, x)] as usize] += 1;
            }
        }
    }
    for i in 1..id {
        if end_count[i as usize] != 2 {
            return false;
        }
    }

    for y in 0..(2 * height - 1) {
        for x in 0..(2 * width - 1) {
            if y % 2 == 1 && x % 2 == 0 {
                if (ids[P(y / 2, x / 2)] == ids[P(y / 2 + 1, x / 2)])
                    != (field.get_edge(LP(y, x)) == Edge::Line)
                {
                    return false;
                }
            } else if y % 2 == 0 && x % 2 == 1 {
                if (ids[P(y / 2, x / 2)] == ids[P(y / 2, x / 2 + 1)])
                    != (field.get_edge(LP(y, x)) == Edge::Line)
                {
                    return false;
                }
            }
        }
    }

    true
}
/// Returns true if the line placements in `field` is too *symmetry*
fn check_symmetry(field: &AnswerField) -> bool {
    let mut n_equal = 0i32;
    let mut n_diff = 0i32;
    let height = field.height();
    let width = field.width();

    for y in 0..(2 * height - 1) {
        for x in 0..(2 * width - 1) {
            if y % 2 != x % 2 {
                let e1 = field.get_edge(LP(y, x));
                let e2 = field.get_edge(LP(2 * height - 2 - y, 2 * width - 2 - x));

                if e1 == Edge::Undecided && e2 == Edge::Undecided {
                    continue;
                }
                if e1 == e2 {
                    n_equal += 1;
                } else {
                    n_diff += 1;
                }
            }
        }
    }

    n_equal as f64 >= (n_equal + n_diff) as f64 * 0.85 + 4.0f64
}
fn limit_clue_number(field: &mut AnswerField, limit: i32) {
    let limit = limit * 2;

    if field.endpoint_forced_cells() > limit {
        field.set_invalid();
    } else {
        if field.endpoint_forced_cells() == limit {
            field.forbid_further_endpoint();
        }
        if field.endpoint_forced_cells() > limit {
            field.set_invalid();
        }
    }
}

fn fill_line_id(pos: P, field: &AnswerField, ids: &mut Grid<i32>, id: i32) {
    if ids[pos] != -1 {
        return;
    }
    ids[pos] = id;

    for &d in &FOUR_NEIGHBOURS {
        if field.get_edge(LP::of_vertex(pos) + d) == Edge::Line {
            fill_line_id(pos + d, field, ids, id);
        }
    }
}