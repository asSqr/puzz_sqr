use std::cmp::min;
use std::default::Default;
use std::ops::{Add, Sub};

const TERMINAL: usize = ::std::usize::MAX;

pub struct GraphSeparation<T>
where
    T: Add<T, Output = T> + Sub<T, Output = T> + Default + Copy,
{
    n: usize,
    m: usize,

    top: Vec<usize>,
    dest: Vec<usize>,
    next_edge: Vec<usize>,
    value: Vec<T>,

    ord: Vec<usize>,
    lowlink: Vec<usize>,
    root: Vec<usize>,
    dfs_edge: Vec<bool>,
}

impl<T> GraphSeparation<T>
where
    T: Add<T, Output = T> + Sub<T, Output = T> + Default + Copy,
{
    pub fn new(n: usize, max_edges: usize) -> GraphSeparation<T> {
        GraphSeparation {
            n,
            m: 0usize,

            top: vec![TERMINAL; n],
            dest: vec![TERMINAL; 2 * max_edges],
            next_edge: vec![TERMINAL; 2 * max_edges],
            value: vec![T::default(); n],

            ord: vec![TERMINAL; n],
            lowlink: vec![TERMINAL; n],
            root: vec![TERMINAL; n],
            dfs_edge: vec![false; 2 * max_edges],
        }
    }
    pub fn add_edge(&mut self, u: usize, v: usize) {
        self.next_edge[self.m] = self.top[u];
        self.dest[self.m] = v;
        self.top[u] = self.m;

        self.next_edge[self.m + 1] = self.top[v];
        self.dest[self.m + 1] = u;
        self.top[v] = self.m + 1;

        self.m += 2;
    }
    pub fn set_weight(&mut self, i: usize, v: T) {
        self.value[i] = v;
    }
    pub fn build(&mut self) {
        let mut counter = 0usize;
        for i in 0..self.n {
            if self.ord[i] == TERMINAL {
                self.visit(i, TERMINAL, i, &mut counter);
            }
        }
    }
    fn visit(&mut self, u: usize, parent: usize, root: usize, counter: &mut usize) {
        self.ord[u] = *counter;
        self.lowlink[u] = *counter;
        self.root[u] = root;
        *counter += 1;

        let mut e = self.top[u];
        while e != TERMINAL {
            let v = self.dest[e];

            if self.ord[v] == TERMINAL {
                // unvisited
                self.visit(v, u, root, counter);
                self.lowlink[u] = min(self.lowlink[u], self.lowlink[v]);
                self.value[u] = self.value[u] + self.value[v];
                self.dfs_edge[e] = true;
            } else if v != parent {
                self.lowlink[u] = min(self.lowlink[u], self.ord[v]);
            }

            e = self.next_edge[e];
        }
    }
    pub fn union_root(&self, u: usize) -> usize {
        self.root[u]
    }
    pub fn separate(&self, sep: usize) -> Vec<T> {
        let mut ret = vec![];

        if self.root[sep] == sep {
            // root
            let mut e = self.top[sep];
            while e != TERMINAL {
                let v = self.dest[e];

                if self.dfs_edge[e] {
                    ret.push(self.value[v]);
                }

                e = self.next_edge[e];
            }
        } else {
            let mut root_side = self.value[self.root[sep]] - self.value[sep];
            let mut e = self.top[sep];
            while e != TERMINAL {
                let v = self.dest[e];

                if self.dfs_edge[e] {
                    if self.lowlink[v] < self.ord[sep] {
                        root_side = root_side + self.value[v];
                    } else {
                        ret.push(self.value[v]);
                    }
                }

                e = self.next_edge[e];
            }
            ret.push(root_side);
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connectivity_naive(n: usize, edges: &[(usize, usize)], sep: usize) -> Vec<i32> {
        let mut visited = vec![false; n];

        fn visit(
            n: usize,
            edges: &[(usize, usize)],
            visited: &mut [bool],
            sep: usize,
            u: usize,
        ) -> i32 {
            if u == sep || visited[u] {
                0
            } else {
                visited[u] = true;
                let mut ret = 1 << (u as i32);
                for &(p, q) in edges {
                    if p == u {
                        ret += visit(n, edges, visited, sep, q);
                    } else if q == u {
                        ret += visit(n, edges, visited, sep, p);
                    }
                }
                ret
            }
        }

        let mut ret = vec![];
        for &(p, q) in edges {
            if p == sep && !visited[q] {
                ret.push(visit(n, edges, &mut visited, sep, q));
            } else if q == sep && !visited[p] {
                ret.push(visit(n, edges, &mut visited, sep, p));
            }
        }
        ret
    }

    #[test]
    fn test_connectivity() {
        {
            let n = 11;
            let edges = [
                (0, 1),
                (0, 3),
                (1, 3),
                (2, 4),
                (2, 6),
                (3, 4),
                (3, 5),
                (4, 6),
                (4, 7),
                (5, 7),
                (8, 9),
                (9, 10),
            ];
            let m = edges.len();
            let mut graph = GraphSeparation::<i32>::new(n, m);
            for &(u, v) in &edges {
                graph.add_edge(u, v);
            }
            for i in 0..n {
                graph.set_weight(i, 1 << (i as i32));
            }

            graph.build();

            for sep in 0..n {
                let mut expected = connectivity_naive(n, &edges, sep);
                let mut returned = graph.separate(sep);
                expected.sort();
                returned.sort();
                assert_eq!(expected, returned, "vertex #{}", sep);
            }
        }
    }
}