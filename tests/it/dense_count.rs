//! A dense Markov network, every pair of its variables joined by a factor:
//! the evidence's large levels are tables the count passes read as grids
//! (`eval::grid`), and the marginals are those of a sum over every joint
//! value.
use crate::common;

use common::{Lcg, assert_answers};
use tddlog::Projection;
use tddlog::Strategy;

/// `n` variables of `d` values each, as ProbLog writes a Markov network:
/// each variable an annotated disjunction over its values whose
/// probabilities sum to 0.8, and each pair of variables a factor, an atom
/// `f(ok)` observed true that each joint value of the pair makes true with
/// its own probability. A variable left without a value makes its factors
/// false, so the evidence conditions on a value for each. Queried: each
/// variable's first and last values.
struct Network {
    n: usize,
    d: usize,
    prior: Vec<Vec<f64>>,
    /// `factor[f][a * d + b]` for the `f`-th pair `(i, j)`, `i < j`, in
    /// order, `i` at `a` and `j` at `b`.
    factor: Vec<Vec<f64>>,
}

impl Network {
    fn new(n: usize, d: usize) -> Network {
        let mut rng = Lcg(12345);
        let prior = (0..n)
            .map(|_| {
                let p: Vec<f64> = (0..d).map(|_| 0.5 + rng.next()).collect();
                let total: f64 = p.iter().sum::<f64>() * 1.25;
                p.iter().map(|x| x / total).collect()
            })
            .collect();
        let factor = (0..n * (n - 1) / 2)
            .map(|_| (0..d * d).map(|_| 0.05 + 0.9 * rng.next()).collect())
            .collect();
        Network {
            n,
            d,
            prior,
            factor,
        }
    }

    fn pairs(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        (0..self.n).flat_map(move |i| (i + 1..self.n).map(move |j| (i, j)))
    }

    fn program(&self) -> String {
        let mut s = String::new();
        for (i, p) in self.prior.iter().enumerate() {
            let choices: Vec<String> = p
                .iter()
                .enumerate()
                .map(|(a, x)| format!("{x:?}::x{i}(v{a})"))
                .collect();
            s += &format!("{}.\n", choices.join("; "));
        }
        for (f, (i, j)) in self.pairs().enumerate() {
            for a in 0..self.d {
                for b in 0..self.d {
                    s += &format!(
                        "{:?}::f{f}(ok) :- x{i}(v{a}), x{j}(v{b}).\n",
                        self.factor[f][a * self.d + b]
                    );
                }
            }
            s += &format!("evidence(f{f}(ok)).\n");
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
        let pairs: Vec<(usize, usize)> = self.pairs().collect();
        let mut by_value = vec![vec![0.0; d]; n];
        let mut total = 0.0;
        let mut x = vec![0usize; n];
        for mut k in 0..d.pow(n as u32) {
            for v in x.iter_mut() {
                *v = k % d;
                k /= d;
            }
            let mut w: f64 = (0..n).map(|i| self.prior[i][x[i]]).product();
            for (f, &(i, j)) in pairs.iter().enumerate() {
                w *= self.factor[f][x[i] * d + x[j]];
            }
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

/// Five variables of twelve values: the table over all five holds 248 832
/// pairs, which the count reads as a grid, its offsets a list; four of
/// sixteen: one of 65 536. The pass up takes those pairs through the
/// grid's routes, through its vector kernels where the processor has them,
/// and the passes down take them into each child: on the first table
/// through the kernels too, whose outputs are adjacent slots; on the
/// second through the scalar loops, whose outputs lie 256 slots apart in
/// a column that no copy relays, since its offsets are a list.
#[test]
fn dense_networks_count_their_tables_as_grids() {
    for (n, d) in [(5, 12), (4, 16)] {
        let net = Network::new(n, d);
        let run = common::run(
            &net.program(),
            &common::options(Strategy::Auto, Projection::default()),
        )
        .unwrap();
        let what = format!("{n} variables of {d} values");
        let grids = run.query_sizes.grid_pairs;
        assert!(grids >= 1 << 16, "{what}: {grids} pairs counted as grids");
        let routes = run.query_sizes.routes;
        assert!(
            routes.up[..3].iter().sum::<usize>() >= grids
                && routes.down[..3].iter().sum::<usize>() > 0,
            "{what}: {routes:?}"
        );
        #[cfg(target_arch = "x86_64")]
        if std::arch::is_x86_feature_detected!("avx2") {
            assert!(
                routes.up[0] >= 1 << 16 && (d == 16 || routes.down[0] >= 1 << 16),
                "{what}: {routes:?}"
            );
        }
        assert_answers(
            &what,
            &run.results.expect("the evidence is consistent"),
            &net.marginals(),
            1e-12,
        );
    }
}
