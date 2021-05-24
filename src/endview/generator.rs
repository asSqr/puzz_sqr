use super::super::P;
use super::*;

use rand::Rng;

pub fn generate<R: Rng>(size: i32, n_alpha: i32, rng: &mut R) -> Option<Problem> {
    let mut current_problem = Problem::new(size, n_alpha);
    let mut prev_score = (size * size * (n_alpha + 1)) as f64;

    let max_step = size * size * 20;
    let temperature = 6.0;

    for _ in 0..max_step {
        let mut update_cand = vec![];
        for &loc in &[ClueLoc::Top, ClueLoc::Bottom, ClueLoc::Left, ClueLoc::Right] {
            for i in 0..size {
                for nxt in -1..n_alpha {
                    if current_problem.get_clue(loc, i) != Clue(nxt) {
                        update_cand.push((loc, i, Clue(nxt)));
                    }
                }
            }
        }

        rng.shuffle(&mut update_cand);

        for &(loc, i, nxt) in &update_cand {
            let current_clue = current_problem.get_clue(loc, i);
            current_problem.set_clue(loc, i, nxt);

            let mut field = Field::from_problem(&current_problem);
            field.trial_and_error();

            let keep_update;
            let current_score;
            if field.inconsistent() {
                current_score = -1f64;
                keep_update = false;
            } else {
                current_score = compute_score(&current_problem, &field);
                keep_update = prev_score > current_score
                    || rng.gen::<f64>() < ((prev_score - current_score) / temperature).exp();
            }

            if keep_update {
                if field.is_solved() {
                    let mut field = Field::from_problem(&current_problem);
                    let mut isok = true;
                    field.apply_methods();
                    for y in 0..size {
                        for x in 0..size {
                            if field.get_value(P(y, x)).0 >= 0 {
                                isok = false;
                            }
                        }
                    }
                    if isok {
                        return Some(current_problem);
                    }
                }
                prev_score = current_score;
                break;
            } else {
                current_problem.set_clue(loc, i, current_clue);
            }
        }
    }
    None
}

fn compute_score(problem: &Problem, field: &Field) -> f64 {
    field.total_cands() as f64 + count_clues(problem) as f64 * 12.0f64
}

fn count_clues(problem: &Problem) -> i32 {
    let mut n_clues = 0;
    for &loc in &[ClueLoc::Top, ClueLoc::Bottom, ClueLoc::Left, ClueLoc::Right] {
        for i in 0..problem.size() {
            if problem.get_clue(loc, i) != NO_CLUE {
                n_clues += 1;
            }
        }
    }
    n_clues
}