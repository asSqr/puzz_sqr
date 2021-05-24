use super::super::D;
use super::Clue;
use crate::grid_loop::Edge;

pub const DICTIONARY_NEIGHBOR_SIZE: usize = 12;
pub const DICTIONARY_EDGE_OFFSET: [D; DICTIONARY_NEIGHBOR_SIZE] = [
    D(-2, -1),
    D(-2, 1),
    D(-1, -2),
    D(-1, 0),
    D(-1, 2),
    D(0, -1),
    D(0, 1),
    D(1, -2),
    D(1, 0),
    D(1, 2),
    D(2, -1),
    D(2, 1),
];

const DICTIONARY_NEIGHBOR_PATTERN_COUNT: usize = 531441; // 3^12
const DICTIONARY_SIZE: usize = DICTIONARY_NEIGHBOR_PATTERN_COUNT * 4;
pub const DICTIONARY_INCONSISTENT: u32 = 0xffffffff;

pub struct Dictionary {
    dic: Vec<u32>,
}
impl Dictionary {
    pub fn complete() -> Dictionary {
        let mut dic = vec![0u32; DICTIONARY_SIZE];
        for clue in 0..4 {
            let ofs = clue * DICTIONARY_NEIGHBOR_PATTERN_COUNT;
            for pat_id in 0..DICTIONARY_NEIGHBOR_PATTERN_COUNT {
                let pat_id = DICTIONARY_NEIGHBOR_PATTERN_COUNT - 1 - pat_id;
                let mut pat = Dictionary::id_to_pattern(pat_id);
                let mut undecided_pos = None;
                for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                    if pat[i] == Edge::Undecided {
                        undecided_pos = Some(i);
                        break;
                    }
                }
                match undecided_pos {
                    Some(p) => {
                        pat[p] = Edge::Line;
                        let base1 = Dictionary::pattern_to_id(pat);
                        pat[p] = Edge::Blank;
                        let base2 = Dictionary::pattern_to_id(pat);

                        dic[ofs + pat_id] = dic[ofs + base1] & dic[ofs + base2];
                    }
                    None => {
                        if Dictionary::is_valid_vertex(pat, 0, 2, 3, 5)
                            && Dictionary::is_valid_vertex(pat, 1, 3, 4, 6)
                            && Dictionary::is_valid_vertex(pat, 5, 7, 8, 10)
                            && Dictionary::is_valid_vertex(pat, 6, 8, 9, 11)
                            && Dictionary::count_lines(pat, 3, 5, 6, 8) == clue
                        {
                            let mut pat_id_bin = 0u32;
                            for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                                pat_id_bin |= match pat[i] {
                                    Edge::Line => 1,
                                    Edge::Blank => 2,
                                    Edge::Undecided => unreachable!(),
                                } << (2 * i);
                            }
                            dic[ofs + pat_id] = pat_id_bin;
                        } else {
                            dic[ofs + pat_id] = DICTIONARY_INCONSISTENT;
                        }
                    }
                }
            }
            for pat_id in 0..DICTIONARY_NEIGHBOR_PATTERN_COUNT {
                let pat_id = DICTIONARY_NEIGHBOR_PATTERN_COUNT - 1 - pat_id;
                let mut pat = Dictionary::id_to_pattern(pat_id);

                let mut pat_id_bin = 0u32;
                for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                    pat_id_bin |= match pat[i] {
                        Edge::Undecided => 0,
                        Edge::Line => 1,
                        Edge::Blank => 2,
                    } << (2 * i);
                }

                if dic[ofs + pat_id] != DICTIONARY_INCONSISTENT {
                    dic[ofs + pat_id] &= !pat_id_bin;
                }
            }
        }
        Dictionary { dic }
    }
    pub fn consult_raw(&self, Clue(c): Clue, neighbor_code: u32) -> u32 {
        self.dic[c as usize * DICTIONARY_NEIGHBOR_PATTERN_COUNT + neighbor_code as usize]
    }
    pub fn consult(&self, Clue(c): Clue, neighbor: &mut [Edge; DICTIONARY_NEIGHBOR_SIZE]) -> bool {
        let id = Dictionary::pattern_to_id(*neighbor);
        let dic_val = self.dic[c as usize * DICTIONARY_NEIGHBOR_PATTERN_COUNT + id];

        if dic_val == DICTIONARY_INCONSISTENT {
            true
        } else {
            for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                neighbor[i] = match (dic_val >> (2 * i)) & 3 {
                    1 => Edge::Line,
                    2 => Edge::Blank,
                    _ => Edge::Undecided,
                }
            }
            false
        }
    }
    fn count_lines(
        pat: [Edge; DICTIONARY_NEIGHBOR_SIZE],
        p1: usize,
        p2: usize,
        p3: usize,
        p4: usize,
    ) -> usize {
        (if pat[p1] == Edge::Line { 1 } else { 0 })
            + (if pat[p2] == Edge::Line { 1 } else { 0 })
            + (if pat[p3] == Edge::Line { 1 } else { 0 })
            + (if pat[p4] == Edge::Line { 1 } else { 0 })
    }
    fn is_valid_vertex(
        pat: [Edge; DICTIONARY_NEIGHBOR_SIZE],
        p1: usize,
        p2: usize,
        p3: usize,
        p4: usize,
    ) -> bool {
        let cnt = Dictionary::count_lines(pat, p1, p2, p3, p4);
        cnt == 0 || cnt == 2
    }
    fn id_to_pattern(pat_id: usize) -> [Edge; DICTIONARY_NEIGHBOR_SIZE] {
        let mut pat = [Edge::Undecided; DICTIONARY_NEIGHBOR_SIZE];
        {
            let mut tmp = pat_id;
            for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                pat[i] = match tmp % 3 {
                    0 => Edge::Undecided,
                    1 => Edge::Line,
                    2 => Edge::Blank,
                    _ => unreachable!(),
                };
                tmp /= 3;
            }
        }
        pat
    }
    fn pattern_to_id(pat: [Edge; DICTIONARY_NEIGHBOR_SIZE]) -> usize {
        let mut ret = 0;
        let mut coef = 1;
        for i in 0..DICTIONARY_NEIGHBOR_SIZE {
            ret += coef
                * match pat[i] {
                    Edge::Undecided => 0,
                    Edge::Line => 1,
                    Edge::Blank => 2,
                };
            coef *= 3;
        }
        ret
    }
}