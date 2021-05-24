use super::super::{Grid, Symmetry, D, LP, P};
use super::*;
use crate::grid_loop::{Edge, GridLoop, GridLoopField};

use rand::Rng;
use crate::common::FOUR_NEIGHBOURS;

pub fn generate<R: Rng>(
    has_clue: &Grid<bool>,
    dic: &Dictionary,
    rng: &mut R,
) -> Option<Grid<Clue>> {
    let height = has_clue.height();
    let width = has_clue.width();
    let max_step = height * width * 10;

    let mut current_problem = Grid::new(height, width, NO_CLUE);
    let mut prev_score = 0;
    let temperature = 5.0f64;

    let mut unplaced_clues = 0;
    for y in 0..height {
        for x in 0..width {
            if has_clue[P(y, x)] {
                unplaced_clues += 1;
            }
        }
    }

    let mut last_field = Field::new(&current_problem, dic);

    for _ in 0..max_step {
        let mut pos_cand = vec![];
        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if has_clue[pos]
                    && (current_problem[pos] == NO_CLUE || has_undecided_nearby(&last_field, pos))
                {
                    pos_cand.push(pos);
                }
            }
        }

        rng.shuffle(&mut pos_cand);

        let mut interpos_common = last_field.clone();
        let mut pos_with_clue = vec![];
        let mut pos_with_clue_idx = 0;

        for &pos in &pos_cand {
            if current_problem[pos] != NO_CLUE {
                pos_with_clue.push(pos);
            }
        }

        let mut updated = false;
        for &pos in &pos_cand {
            let prev_clue = current_problem[pos];
            let is_zero_ok = !has_zero_nearby(&current_problem, pos);

            let mut new_clue_cand = vec![];
            for c in (if is_zero_ok { 0 } else { 1 })..4 {
                let c = Clue(c);
                if c != prev_clue {
                    new_clue_cand.push(c);
                }
            }

            rng.shuffle(&mut new_clue_cand);

            let mut common;
            if prev_clue == NO_CLUE {
                common = last_field.clone();
            } else {
                if pos_with_clue_idx % 2 == 0 {
                    if pos_with_clue_idx == pos_with_clue.len() - 1 {
                        current_problem[pos] = NO_CLUE;
                        common = Field::new(&current_problem, dic);
                        common.check_all_cell();
                    } else {
                        let c1 = current_problem[pos_with_clue[pos_with_clue_idx]];
                        current_problem[pos_with_clue[pos_with_clue_idx]] = NO_CLUE;
                        let c2 = current_problem[pos_with_clue[pos_with_clue_idx + 1]];
                        current_problem[pos_with_clue[pos_with_clue_idx + 1]] = NO_CLUE;

                        interpos_common = Field::new(&current_problem, dic);
                        interpos_common.check_all_cell();

                        current_problem[pos_with_clue[pos_with_clue_idx]] = c1;
                        current_problem[pos_with_clue[pos_with_clue_idx + 1]] = c2;

                        common = interpos_common.clone();
                        common.add_clue(pos_with_clue[pos_with_clue_idx + 1], c2);
                    }
                } else {
                    common = interpos_common.clone();
                    common.add_clue(
                        pos_with_clue[pos_with_clue_idx - 1],
                        current_problem[pos_with_clue[pos_with_clue_idx - 1]],
                    );
                }
                pos_with_clue_idx += 1;
            }

            for &c in &new_clue_cand {
                current_problem[pos] = c;

                let mut field = common.clone();
                field.add_clue(pos, c);

                if field.inconsistent() {
                    continue;
                }

                let current_score = field.grid_loop().num_decided_edges()
                    - count_prohibited_patterns(has_clue, &field, &current_problem) * 10;

                if prev_score >= current_score {
                    if !(rng.gen::<f64>()
                        < ((current_score - prev_score) as f64 / temperature).exp())
                    {
                        continue;
                    }
                }

                let mut field_inout_test = field.clone();
                GridLoop::apply_inout_rule(&mut field_inout_test);
                GridLoop::check_connectability(&mut field_inout_test);
                if field_inout_test.inconsistent() {
                    continue;
                }

                updated = true;
                prev_score = current_score;
                if prev_clue == NO_CLUE {
                    unplaced_clues -= 1;
                }

                if field.fully_solved() && unplaced_clues == 0 {
                    return Some(current_problem);
                }

                last_field = field;
                break;
            }

            if updated {
                break;
            } else {
                current_problem[pos] = prev_clue;
            }
        }
    }

    None
}

fn has_undecided_nearby(field: &Field, pos: P) -> bool {
    let lp = LP::of_cell(pos);

    let neighbor_size: i32 = 7;
    for dy in -neighbor_size..(neighbor_size + 1) {
        let dx_max = neighbor_size - dy.abs();
        for dx in -dx_max..(dx_max + 1) {
            if (dy & 1) != (dx & 1) {
                if field.get_edge_safe(lp + D(dy, dx)) == Edge::Undecided {
                    return true;
                }
            }
        }
    }
    false
}

fn has_zero_nearby(problem: &Grid<Clue>, pos: P) -> bool {
    for dy in -1..2 {
        for dx in -1..2 {
            let pos2 = pos + D(dy, dx);
            if problem.is_valid_p(pos2) && problem[pos2] == Clue(0) {
                return true;
            }
        }
    }
    false
}
fn count_prohibited_patterns(has_clue: &Grid<bool>, field: &Field, problem: &Grid<Clue>) -> i32 {
    let mut ret = 0;
    for y in 0..has_clue.height() {
        for x in 0..has_clue.width() {
            let pos = P(y, x);
            let pos_lp = LP::of_cell(pos);
            if has_clue[pos] && field.get_clue(pos) == NO_CLUE && has_zero_nearby(problem, pos) {
                if field.get_edge(pos_lp + D(-1, 0)) == Edge::Blank
                    && field.get_edge(pos_lp + D(1, 0)) == Edge::Blank
                    && field.get_edge(pos_lp + D(0, -1)) == Edge::Blank
                    && field.get_edge(pos_lp + D(0, 1)) == Edge::Blank
                {
                    ret += 1;
                    continue;
                }
            }
            if y > 0 && field.get_clue(pos + D(-1, 0)) != NO_CLUE {
                continue;
            }
            if x > 0 && field.get_clue(pos + D(0, -1)) != NO_CLUE {
                continue;
            }
            if y < has_clue.height() - 1 && field.get_clue(pos + D(1, 0)) != NO_CLUE {
                continue;
            }
            if x < has_clue.width() - 1 && field.get_clue(pos + D(0, 1)) != NO_CLUE {
                continue;
            }

            if field.get_clue(pos) == Clue(2) {
                if field.get_edge_safe(pos_lp + D(2, 1)) == Edge::Blank
                    && field.get_edge_safe(pos_lp + D(1, 2)) == Edge::Blank
                    && field.get_edge_safe(pos_lp + D(-2, -1)) == Edge::Blank
                    && field.get_edge_safe(pos_lp + D(-1, -2)) == Edge::Blank
                {
                    ret += 1;
                    continue;
                }
                if field.get_edge_safe(pos_lp + D(-2, 1)) == Edge::Blank
                    && field.get_edge_safe(pos_lp + D(-1, 2)) == Edge::Blank
                    && field.get_edge_safe(pos_lp + D(2, -1)) == Edge::Blank
                    && field.get_edge_safe(pos_lp + D(1, -2)) == Edge::Blank
                {
                    ret += 1;
                    continue;
                }
            } else if field.get_clue(pos) == NO_CLUE {
                let mut n_in = 0;
                let mut n_blank = 0;

                for &d in &FOUR_NEIGHBOURS {
                    let dr = d.rotate_clockwise();
                    let edge1 = field.get_edge_safe(pos_lp + d * 2 + dr);
                    let edge2 = field.get_edge_safe(pos_lp + d + dr * 2);
                    match (edge1, edge2) {
                        (Edge::Blank, Edge::Blank) => n_blank += 1,
                        (Edge::Blank, Edge::Line) | (Edge::Line, Edge::Blank) => n_in += 1,
                        _ => (),
                    }
                }
                if n_in >= 1 && n_blank >= 2 {
                    ret += 1;
                }
            }
        }
    }
    ret
}

pub fn generate_placement<R: Rng>(
    height: i32,
    width: i32,
    num_clues: i32,
    symmetry: Symmetry,
    rng: &mut R,
) -> Grid<bool> {
    let mut num_clues = num_clues;
    let mut symmetry = symmetry;

    symmetry.dyad |= symmetry.tetrad;
    symmetry.tetrad &= height == width;

    let mut grp_ids = Grid::new(height, width, false);

    let mut clue_positions: Vec<Vec<P>> = vec![];

    for y in 0..height {
        for x in 0..width {
            if !grp_ids[P(y, x)] {
                let mut sto = vec![];
                update_grp(y, x, symmetry, &mut grp_ids, &mut sto);
                clue_positions.push(sto);
            }
        }
    }

    let mut ret = Grid::new(height, width, false);
    while clue_positions.len() > 0 && num_clues > 0 {
        let mut scores = vec![];
        let mut scores_total = 0.0f64;

        for pos in &clue_positions {
            let p = pos[0];
            let mut score_base = 0.0f64;

            for dy in -2..3 {
                for dx in -2..3 {
                    let cd2 = p + D(dy, dx);
                    if ret.is_valid_p(cd2) && ret[cd2] {
                        let dist = dy.abs() + dx.abs();
                        score_base += 5.0f64 - (dist as f64);
                        if dist == 1 {
                            score_base += 2.0f64;
                        }
                    }
                }
            }

            let pos_score = 64.0f64 * 2.0f64.powf((16.0f64 - score_base) / 2.0f64) + 4.0f64;
            scores.push(pos_score);
            scores_total += pos_score;
        }

        let mut thresh = rng.gen_range(0.0f64, scores_total);
        for i in 0..clue_positions.len() {
            if thresh < scores[i] {
                for &c in &(clue_positions[i]) {
                    ret[c] = true;
                    num_clues -= 1;
                }
                clue_positions.swap_remove(i);
                break;
            } else {
                thresh -= scores[i];
            }
        }
    }

    ret
}

fn update_grp(y: i32, x: i32, symmetry: Symmetry, grp_ids: &mut Grid<bool>, sto: &mut Vec<P>) {
    if grp_ids[P(y, x)] {
        return;
    }
    grp_ids[P(y, x)] = true;
    sto.push(P(y, x));

    if symmetry.tetrad {
        update_grp(grp_ids.height() - 1 - x, y, symmetry, grp_ids, sto);
    } else if symmetry.dyad {
        update_grp(
            grp_ids.height() - 1 - y,
            grp_ids.width() - 1 - x,
            symmetry,
            grp_ids,
            sto,
        );
    }
    if symmetry.horizontal {
        update_grp(grp_ids.height() - 1 - y, x, symmetry, grp_ids, sto);
    }
    if symmetry.vertical {
        update_grp(y, grp_ids.width() - 1 - x, symmetry, grp_ids, sto);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand;

    fn run_placement_test<R: Rng>(placement: Vec<Vec<bool>>, dic: &Dictionary, rng: &mut R) {
        let placement = ::common::vec_to_grid(&placement);
        let mut succeeded = false;

        for _ in 0..10 {
            let problem = generate(&placement, dic, rng);

            if let Some(problem) = problem {
                succeeded = true;

                assert_eq!(problem.height(), placement.height());
                assert_eq!(problem.width(), placement.width());

                for y in 0..placement.height() {
                    for x in 0..placement.width() {
                        let clue = problem[P(y, x)];
                        assert_eq!(placement[P(y, x)], clue != NO_CLUE);

                        if clue == Clue(0) {
                            for dy in -1..2 {
                                for dx in -1..2 {
                                    let y2 = y + dy;
                                    let x2 = x + dx;

                                    if 0 <= y2
                                        && y2 < placement.height()
                                        && 0 <= x2
                                        && x2 < placement.width()
                                        && (dy, dx) != (0, 0)
                                    {
                                        assert!(problem[P(y2, x2)] != Clue(0));
                                    }
                                }
                            }
                        }
                    }
                }

                let mut field = Field::new(&problem, &dic);
                field.check_all_cell();
                assert!(!field.inconsistent());
                assert!(field.fully_solved());

                break;
            }
        }

        assert!(succeeded);
    }

    #[test]
    fn test_generator() {
        let mut rng = rand::thread_rng();
        let dic = Dictionary::complete();

        run_placement_test(
            vec![
                vec![true, true, true, true, true],
                vec![true, false, false, false, true],
                vec![true, false, false, false, true],
                vec![true, false, false, false, true],
                vec![true, true, true, true, true],
            ],
            &dic,
            &mut rng,
        );

        run_placement_test(
            vec![
                vec![true, false, true, true, true],
                vec![false, false, false, false, true],
                vec![true, false, false, false, true],
                vec![true, false, false, false, false],
                vec![true, true, true, false, true],
            ],
            &dic,
            &mut rng,
        );
    }
}