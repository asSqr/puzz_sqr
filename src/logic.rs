use std::collections::{HashMap, HashSet};
use varisat::{CnfFormula, ExtendFormula};
use varisat::solver::Solver;
use varisat::{Var, Lit};

type Field = Vec<Vec<usize>>;
type P = (usize, usize);
type Arc = (P, P);
type Sol = Vec<Arc>;

pub fn solve(url: String) -> Option<(Field, Sol)> {
    let field = parse_url(url);

    if let None = field {
        return None;
    }

    let field = field.unwrap();

    let sol = solve_numberlink(&field);

    if let None = sol {
        return None;
    }

    
    Some((field.clone(), sol.unwrap()))
}

fn solve_numberlink(field: &Field) -> Option<Sol> {
    if field.len() == 0 || field[0].len() == 0 {
        return None;
    }

    let width = field[0].len();
    let height = field.len();

    let (s, t, b) = parse_field(&field).unwrap_or((vec![], vec![], vec![]));

    if s.len() == 0 || s.len() != t.len() || s.len()+t.len()+b.len() != width*height {
        return None;
    }

    let mut vs: Vec<P> = vec![];

    for i in 0..height {
        for j in 0..width {
            vs.push((i, j));
        }
    }

    let arcs: Vec<Arc> = gen_arcs(width, height);

    let mut formula = CnfFormula::new();

    let mut mp: HashMap<Arc, usize> = HashMap::new();

    /*for (i, arc) in arcs.clone().into_iter().enumerate() {
        println!("arc {}: {:?}", i, arc);
    }*/

    /* "Solving Nubmerlink by a SAT-based Constraint Solver" (https://ipsj.ixsq.nii.ac.jp/ej/index.php?action=pages_view_main&active_action=repository_action_common_download&item_id=102780&item_no=1&attribute_id=1&file_no=1&page_id=13&block_id=8) */
    for (i, (u, v)) in arcs.clone().into_iter().enumerate() {
        mp.insert((u, v), i);
    }

    let length = arcs.len();
    let mut m = s.len();
    let mut mb = 0;

    while m > 0 {
        m >>= 1;
        mb += 1;
    }

    let mut bmp: HashMap<P, Vec<usize>> = HashMap::new();

    // vs*log(n)
    // 自然数変数を bit ごとに分解
    for (index, (i,j)) in vs.clone().into_iter().enumerate() {
        for b in 0..mb {
            let x = Var::from_index(length+index*mb+b+1);

            if !bmp.contains_key(&(i,j)) {
                bmp.insert((i,j), vec![length+index*mb+b+1]);
            } else {
                bmp.get_mut(&(i,j)).unwrap().push(length+index*mb+b+1);
            }

            let mut num: usize = field[i][j];

            if num == 0 {
                continue;
            }

            num -= 1;

            // (11)
            formula.add_clause(&[Lit::from_var(x, (num>>b&1) != 0)]);
        }
    }

    for (u, v) in arcs.clone().into_iter() {
        let x = Var::from_index(mp[&(u, v)]+1);

        // (12)
        // !(x and num_u != num_v)
        // !x or f_u == f_v
        for lits in mk_clause_impl(&x, &bmp[&u], &bmp[&v]) {
            formula.add_clause(lits.as_slice());
        }

        let y = Var::from_index(mp[&(v, u)]+1);

        // (2)
        formula.add_clause(&[x.negative(), y.negative()]);
    }

    for u in vs {
        let adjs: &Vec<P> = &adj(u, width, height);

        if s.contains(&u) {
            // (3)
            {
                let mut vars: Vec<Var> = vec![];    
                for v in adjs {
                    vars.push(Var::from_index(mp[&(u, *v)]+1));
                }

                for lits in mk_clause_eq1(vars) {
                    formula.add_clause(lits.as_slice());
                }
            }    
            
            // (4)
            {
                for v in adjs {
                    formula.add_clause(&[Var::from_index(mp[&(*v, u)]+1).negative()]);
                }
            }
        }

        if t.contains(&u) {
            // (5)
            {
                for v in adjs {
                    formula.add_clause(&[Var::from_index(mp[&(u, *v)]+1).negative()]);
                }
            }
            
            // (6)
            {
                let mut vars: Vec<Var> = vec![];    
                for v in adjs {
                    vars.push(Var::from_index(mp[&(*v, u)]+1));
                }

                for lits in mk_clause_eq1(vars) {
                    formula.add_clause(lits.as_slice());
                }
            }
        }

        if b.contains(&u) {
            // (8) (9)
            {
                let mut varss: Vec<Vec<Var>> = vec![];
                let mut vars1: Vec<Var> = vec![];
                let mut vars2: Vec<Var> = vec![];    
                for v in adjs {
                    vars1.push(Var::from_index(mp[&(u, *v)]+1));
                    vars2.push(Var::from_index(mp[&(*v, u)]+1));
                }

                varss.push(vars1);
                varss.push(vars2);

                /*for i in 0..2 {
                    println!("----------");

                    for var in &varss[i] {
                        println!("{}", var.index());
                    }

                    println!("----------");
                }*/

                for lits in mk_clause_d(varss) {
                    formula.add_clause(lits.as_slice());
                }
            }
        }
    }

    /*for lits in formula.iter() {
        let mut flag = false;

        for lit in lits {
            flag = flag || lit.var().index() == 8;
        }

        flag = true;

        if flag {
            println!("---------");

            for lit in lits {
                println!("{} [{}]", lit.var().index(), if lit.is_positive() { "True" } else { "False" });
            }

            println!("---------");
        }
    }*/

    let mut solver = Solver::new();

    solver.add_formula(&formula);

    solver.solve().unwrap();

    let model = solver.model();

    /*let mut index = 1;

    let mut prev_formula = CnfFormula::new();

    loop {
        solver = Solver::new();

        solver.add_formula(&formula);

        solution = solver.solve().unwrap();
        model = solver.model();

        match &model {
            Some(lits) => {
                let mut cnt = 0;
                let mut flag = false;

                for (idx, lit) in lits.iter().enumerate() {
                    if lit.var().index() >= arcs.len()+1 {
                        continue;
                    }

                    if lit.is_positive() {
                        cnt += 1;
                    }

                    if !flag && idx >= index && lit.is_negative() {
                        flag = true;

                        index = idx;
                    }
                }

                if cnt == width*height-s.len() {
                    break;
                }

                prev_formula = CnfFormula::new();

                for lits in formula.iter() {
                    prev_formula.add_clause(lits.clone());
                }

                println!("index: {}", index);

                formula.add_clause(&[Lit::from_var(Var::from_index(index), true)]);

                index += 1;

                if index >= lits.len() {
                    return None;
                }
            },
            None => {
                formula = CnfFormula::new();

                for lits in prev_formula.iter() {
                    formula.add_clause(lits.clone());
                }
            }
        }
    }*/
    
    //println!("Solution: {}", solution);

    let mut sol: Vec<Arc> = vec![];

    match model {
        Some(lits) => {
            //println!("{:?}", lits);

            for lit in &lits {
                //println!("{} [{}]", lit.var().index(), if lit.is_positive() { "True" } else { "False" });

                if lit.is_positive() {
                    let index = lit.var().index();

                    if index >= arcs.len() {
                        continue;
                    }

                    //let arc = arcs[index-1];
                    //println!("{:?}", arc);

                    sol.push(arcs[index-1]);
                }
            }

            /*if cnt != width*height-s.len() {
                println!("Kansei Solution!!!");
            }*/
        },
        None => {
            //println!("No Solution");

            return None;
        }
    }

    Some(sol)
}

fn parse_field(field :&Field) -> Option<(Vec<P>, Vec<P>, Vec<P>)> {
    let mut cnt = vec![0; 100];
    let mut ends = vec![vec![]; 2];
    let mut b = vec![];
    
    for (i, line) in field.clone().into_iter().enumerate() {
        for (j, p) in line.clone().into_iter().enumerate() {
            if p > 0 {
                if cnt[p] >= 2 {
                    return None;
                }

                ends[cnt[p]].push((i, j));
                cnt[p] += 1;
            } else {
                b.push((i, j));
            }
        }
    }

    Some((ends[0].clone(), ends[1].clone(), b))
}

fn mk_clause_impl(x: &Var, fu: &Vec<usize>, fv: &Vec<usize>) -> Vec<Vec<Lit>> {
    let mut res: Vec<Vec<Lit>> = vec![];
    
    for (i, fuidx) in fu.clone().into_iter().enumerate() {
        let fvidx = fv[i].clone();
        let fui = Var::from_index(fuidx);
        let fvi = Var::from_index(fvidx);

        res.push(vec![x.negative(), fui.negative(), fvi.positive()]);
        res.push(vec![x.negative(), fui.positive(), fvi.negative()]);
    }

    res
}

fn popcount(bit: usize) -> usize {
    let mut ret = 0;
    let mut b = bit;

    while b > 0 {
        if b&1 != 0 {
            ret += 1;
        }

        b >>= 1;
    }

    ret
}

fn mk_clause_eq1(vars: Vec<Var>) -> Vec<Vec<Lit>> {
    let mut res: Vec<Vec<Lit>> = vec![];
    let n = vars.len();

    for bit in 0..(1<<n) {
        if 1+popcount(bit) == n {
            continue;
        }

        let mut lits: Vec<Lit> = vec![];

        for i in 0..n {
            lits.push(Lit::from_var(vars[i], (bit>>i&1) != 0));
        }

        res.push(lits);
    }

    res
}

fn mk_clause_d(varss: Vec<Vec<Var>>) -> Vec<Vec<Lit>> {
    let mut res: Vec<Vec<Lit>> = vec![];
    let n = varss[0].len();

    /*for ri in 0..2 {
        for rj in 0..2 {
            let mut lits: Vec<Lit> = vec![];

            for i in 0..n {
                lits.push(Lit::from_var(varss[ri][i], false));
            }

            res.push(lits);
        }
    }*/

    for r in 0..2 {
        for i in 0..(1<<n) {
            if 1+popcount(i) == n {
                continue;
            }

            let mut lits = vec![];

            for j in 0..n {
                lits.push(Lit::from_var(varss[r][j], (i>>j&1) != 0));
            }

            res.push(lits);
        }
    }

    res
}

fn adj(p: P, width: usize, height: usize) -> Vec<P> {
    let dx: Vec<i32> = vec![1, 0, -1, 0];
    let dy: Vec<i32> = vec![0, 1, 0, -1];

    let mut st = HashSet::new();
    let mut res = vec![];

    for d in 0..4 {
        let ni = (p.0 as i32 + dy[d]) as usize;
        let nj = (p.1 as i32 + dx[d]) as usize;

        if ni < height && nj < width && !st.contains(&(ni, nj)) {
            res.push((ni, nj));
            st.insert((ni, nj));
        }
    }

    res
}

fn gen_arcs(width: usize, height: usize) -> Vec<Arc> {
    let mut res: Vec<Arc> = vec![];

    for i in 0..height {
        for j in 0..width {
            let u = (i, j);
            let adjs = adj(u, width, height);

            for v in adjs {
                res.push((u, v));
            }
        }
    }

    res
}

pub fn parse_url(url: String) -> Option<Field> {
    let splitter = '/';
    let params: Vec<String> = url.split(splitter).map(|s| s.to_string()).collect();
    let length = params.len();

    if length < 3 {
        return None;
    }

    let width = params[length-3].parse().unwrap_or(0);
    let height = params[length-2].parse().unwrap_or(0);

    if width <= 0 || height <= 0 {
        return None;
    }

    let field_code = params[length-1].clone();

    if !is_valid_code(&field_code) {
        return None;
    }

    decode_field(width, height, field_code)
}

fn is_valid_code(code: &String) -> bool {
    return code.chars().all(|ch| char::is_alphanumeric(ch) || ch == '-');
}

fn decode_field(width: usize, height: usize, code: String) -> Option<Field> {
    let list: &Vec<char> = &code.chars().collect();
    let mut index: usize = 0;
    let mut i: usize = 0;
    let mut j: usize = 0;

    let mut res: Field = vec![vec![0; width as usize]; height as usize];

    while index < list.len() {
        while let Some(num) = get_num(&mut index, list) {
            res[i as usize][j as usize] = num;

            j += 1;

            if j >= width {
                j = 0;
                i += 1;
            }

            if index >= list.len() {
                break;
            }
        }

        consume(&mut index, &mut i, &mut j, width, list);
    }

    Some(res)
}

fn get_num(index: &mut usize, list: &Vec<char>) -> Option<usize> {
    let ch = list[*index as usize];

    if ch == '-' {
        *index += 1;
        let mut res = 0;

        while *index < list.len() && list[*index].is_digit(16) {
            res *= 16;
            res += list[*index].to_digit(16).unwrap();
            *index += 1;
        }

        return Some(res as usize);
    } else if ch.is_digit(16) {
        *index += 1;

        return match ch.to_digit(16) {
            Some(num) => Some(num as usize),
            None => None,
        };
    } else {
        return None;
    }
}

fn consume(index: &mut usize, i: &mut usize, j: &mut usize, width: usize, list: &Vec<char>) {
    let length = list.len();

    while *index < length && !(list[*index as usize]).is_digit(16) {
        let ch = list[*index as usize];
        let value = (ch as i32) - ('f' as i32);

        if value <= 0 {
            return;
        }

        let value: usize = value as usize;

        *j += value;

        if *j >= width {
            *i += *j/width;
            *j %= width;
        }

        *index += 1;
    }    
}