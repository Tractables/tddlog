//! Dense Bayesian networks whose evidence holds implicit levels: complete
//! products the conjunction keeps as the description of their pairs, which
//! the count passes read over the pairs they generate, these levels being
//! too small to repay reading one as a grid (`eval::grid`, whose unit tests
//! check a grid read off a description against those passes). The
//! marginals are those of a sum over every joint value.
use crate::common;

use common::{Lcg, assert_answers};
use tddlog::Projection;
use tddlog::Strategy;

/// `n` variables of `d` values each, as ProbLog writes a Bayesian network:
/// each variable an annotated disjunction over its values per joint value
/// of its parents, its probabilities summing to 1, so that every variable
/// takes a value. Observed: some variables at a value. Queried: each
/// variable's first and last values.
struct Network {
    n: usize,
    d: usize,
    parents: Vec<Vec<usize>>,
    /// `cpt[i][row][a]`, variable `i` at `a` given its parents' joint value
    /// `row`, the first parent's value its fastest digit.
    cpt: Vec<Vec<Vec<f64>>>,
    evidence: Vec<(usize, usize)>,
}

impl Network {
    fn new(
        d: usize,
        parents: Vec<Vec<usize>>,
        evidence: Vec<(usize, usize)>,
        seed: u64,
    ) -> Network {
        let mut rng = Lcg(seed);
        let cpt = parents
            .iter()
            .map(|ps| {
                (0..d.pow(ps.len() as u32))
                    .map(|_| {
                        let p: Vec<f64> = (0..d).map(|_| 0.5 + rng.next()).collect();
                        let total: f64 = p.iter().sum();
                        p.iter().map(|x| x / total).collect()
                    })
                    .collect()
            })
            .collect();
        Network {
            n: parents.len(),
            d,
            parents,
            cpt,
            evidence,
        }
    }

    /// A `rows` by `cols` grid, each variable a child of the one left of it
    /// and the one above it.
    fn grid(rows: usize, cols: usize, d: usize, evidence: Vec<(usize, usize)>) -> Network {
        let at = |r: usize, c: usize| r * cols + c;
        let parents = (0..rows)
            .flat_map(|r| (0..cols).map(move |c| (r, c)))
            .map(|(r, c)| {
                let mut ps = Vec::new();
                if c > 0 {
                    ps.push(at(r, c - 1));
                }
                if r > 0 {
                    ps.push(at(r - 1, c));
                }
                ps
            })
            .collect();
        Network::new(d, parents, evidence, 12345)
    }

    /// A chain of `n` variables, each a child of the one before.
    fn chain(n: usize, d: usize, evidence: Vec<(usize, usize)>) -> Network {
        Network::new(
            d,
            (0..n)
                .map(|i| if i == 0 { vec![] } else { vec![i - 1] })
                .collect(),
            evidence,
            54321,
        )
    }

    /// The row of variable `i`'s table that the joint value `x` selects.
    fn row(&self, i: usize, x: &[usize]) -> usize {
        self.parents[i]
            .iter()
            .rev()
            .fold(0, |row, &p| row * self.d + x[p])
    }

    fn program(&self) -> String {
        let mut s = String::new();
        for i in 0..self.n {
            for (row, p) in self.cpt[i].iter().enumerate() {
                let choices: Vec<String> = p
                    .iter()
                    .enumerate()
                    .map(|(a, x)| format!("{x:?}::x{i}(v{a})"))
                    .collect();
                let mut k = row;
                let body: Vec<String> = self.parents[i]
                    .iter()
                    .map(|&q| {
                        let a = k % self.d;
                        k /= self.d;
                        format!("x{q}(v{a})")
                    })
                    .collect();
                let rule = if body.is_empty() {
                    String::new()
                } else {
                    format!(" :- {}", body.join(", "))
                };
                s += &format!("{}{rule}.\n", choices.join("; "));
            }
        }
        for &(i, a) in &self.evidence {
            s += &format!("evidence(x{i}(v{a})).\n");
        }
        for (i, a) in self.queries() {
            s += &format!("query(x{i}(v{a})).\n");
        }
        s
    }

    fn queries(&self) -> Vec<(usize, usize)> {
        (0..self.n)
            .flat_map(|i| [(i, 0), (i, self.d - 1)])
            .collect()
    }

    /// The marginals of the queries given the evidence, summed over every
    /// joint value of the variables.
    fn marginals(&self) -> Vec<(String, f64)> {
        let (n, d) = (self.n, self.d);
        let mut by_value = vec![vec![0.0; d]; n];
        let mut total = 0.0;
        let mut x = vec![0usize; n];
        for mut k in 0..d.pow(n as u32) {
            for v in x.iter_mut() {
                *v = k % d;
                k /= d;
            }
            if self.evidence.iter().any(|&(i, a)| x[i] != a) {
                continue;
            }
            let w: f64 = (0..n).map(|i| self.cpt[i][self.row(i, &x)][x[i]]).product();
            total += w;
            for i in 0..n {
                by_value[i][x[i]] += w;
            }
        }
        self.queries()
            .into_iter()
            .map(|(i, a)| (format!("x{i}(v{a})"), by_value[i][a] / total))
            .collect()
    }
}

/// A 3 by 3 grid of 3-valued variables and a chain of five 15-valued ones,
/// each with and without an observed value: the answers are the marginals
/// summed over every joint value, with the evidence's implicit levels
/// counted over their pairs.
#[test]
fn dense_networks_with_implicit_levels_count_their_marginals() {
    let networks = [
        ("3x3 grid of 3 values", Network::grid(3, 3, 3, vec![])),
        (
            "3x3 grid of 3 values, the last observed",
            Network::grid(3, 3, 3, vec![(8, 2)]),
        ),
        ("chain of 5 of 15 values", Network::chain(5, 15, vec![])),
        (
            "chain of 5 of 15 values, one observed",
            Network::chain(5, 15, vec![(3, 2)]),
        ),
    ];
    for (what, net) in networks {
        let run = common::run(
            &net.program(),
            &common::options(Strategy::Auto, Projection::default()),
        )
        .unwrap();
        let q = &run.query_sizes;
        assert!(
            q.evidence_implicit > 0,
            "{what}: no implicit level in the evidence of {} pairs",
            q.evidence
        );
        assert_eq!(
            q.grid_pairs, 0,
            "{what}: a level this small counted as a grid"
        );
        assert_answers(
            what,
            &run.results.expect("the evidence is consistent"),
            &net.marginals(),
            1e-12,
        );
    }
}
