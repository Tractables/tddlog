//! Sets of many values read in binary ([`tddlog::indicators`]): a reader
//! reads a value as the cube of its code over copies of the code's bits.
//! Small networks against answers computed by enumerating their worlds,
//! with ProbLog's reading of a row whose values do not sum to one: the
//! rest of its mass goes to no value, and a child of a variable with no
//! value has none either.
use crate::common;

use common::{Rows, assert_answers};
use tddlog::{Kind, Strategy};

/// A discrete variable: its values, its parents, and a row per assignment
/// of the parents (the first parent's value most significant), each a
/// probability per value.
struct Var {
    name: &'static str,
    values: usize,
    parents: Vec<usize>,
    rows: Vec<Vec<f64>>,
}

/// Probabilities from a fixed sequence, so that the networks are the same
/// on every run.
struct Draw {
    state: u64,
}

impl Draw {
    fn next(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state >> 33
    }

    /// A row of `values` probabilities summing to `mass`, about one in
    /// `zeros` of them zero, none if `zeros` is 0.
    fn row(&mut self, values: usize, zeros: u64, mass: f64) -> Vec<f64> {
        let mut weights: Vec<f64> = (0..values)
            .map(|_| {
                if zeros > 0 && self.next().is_multiple_of(zeros) {
                    0.0
                } else {
                    (self.next() % 97 + 1) as f64
                }
            })
            .collect();
        if weights.iter().all(|&w| w == 0.0) {
            weights[0] = 1.0;
        }
        let total: f64 = weights.iter().sum();
        weights.iter().map(|w| w / total * mass).collect()
    }
}

struct Network {
    vars: Vec<Var>,
}

impl Network {
    /// A variable of `values` values reading `parents`, its rows drawn
    /// from `draw`, each summing to the mass `mass` gives its index. A
    /// root's row has no zero, so that each of its values is an atom of
    /// the program.
    fn add(
        &mut self,
        draw: &mut Draw,
        name: &'static str,
        values: usize,
        parents: &[usize],
        mass: impl Fn(usize) -> f64,
    ) -> usize {
        let count: usize = parents.iter().map(|&p| self.vars[p].values).product();
        let zeros = if parents.is_empty() { 0 } else { 4 };
        let rows = (0..count)
            .map(|r| draw.row(values, zeros, mass(r)))
            .collect();
        self.vars.push(Var {
            name,
            values,
            parents: parents.to_vec(),
            rows,
        });
        self.vars.len() - 1
    }

    fn atom(&self, v: usize, value: usize) -> String {
        format!("{}(v{value})", self.vars[v].name)
    }

    /// The program: an annotated disjunction per row, its zero heads left
    /// out, and the evidence and queries.
    fn source(&self, evidence: &[(usize, usize, bool)], queries: &[usize]) -> String {
        let mut source = String::new();
        for (v, var) in self.vars.iter().enumerate() {
            for (r, row) in var.rows.iter().enumerate() {
                let heads: Vec<String> = row
                    .iter()
                    .enumerate()
                    .filter(|&(_, &p)| p > 0.0)
                    .map(|(i, p)| format!("{p}::{}", self.atom(v, i)))
                    .collect();
                source += &heads.join("; ");
                let mut rest = r;
                let mut body = Vec::new();
                for &p in var.parents.iter().rev() {
                    body.push(self.atom(p, rest % self.vars[p].values));
                    rest /= self.vars[p].values;
                }
                if !body.is_empty() {
                    body.reverse();
                    source += &format!(" :- {}", body.join(", "));
                }
                source += ".\n";
            }
        }
        for &(v, value, holds) in evidence {
            source += &format!("evidence({},{holds}).\n", self.atom(v, value));
        }
        for &v in queries {
            for i in 0..self.vars[v].values {
                source += &format!("query({}).\n", self.atom(v, i));
            }
        }
        source
    }

    /// P(q | e) for every value of the variables `queries`, in order, by
    /// enumerating every assignment of values or none to the variables,
    /// which are in topological order.
    fn exact(&self, evidence: &[(usize, usize, bool)], queries: &[usize]) -> Vec<(String, f64)> {
        let n = self.vars.len();
        let mut state = vec![0usize; n];
        let mut joint = vec![vec![0.0; 0]; n];
        for (v, var) in self.vars.iter().enumerate() {
            joint[v] = vec![0.0; var.values];
        }
        let mut total = 0.0;
        // `state[v] == values` is none.
        fn walk(
            net: &Network,
            v: usize,
            p: f64,
            state: &mut Vec<usize>,
            evidence: &[(usize, usize, bool)],
            joint: &mut [Vec<f64>],
            total: &mut f64,
        ) {
            if p == 0.0 {
                return;
            }
            if v == net.vars.len() {
                if evidence
                    .iter()
                    .all(|&(e, value, holds)| (state[e] == value) == holds)
                {
                    *total += p;
                    for (u, &s) in state.iter().enumerate() {
                        if s < net.vars[u].values {
                            joint[u][s] += p;
                        }
                    }
                }
                return;
            }
            let var = &net.vars[v];
            let mut row = 0;
            let mut unset = false;
            for &q in &var.parents {
                unset |= state[q] == net.vars[q].values;
                row = row * net.vars[q].values + state[q].min(net.vars[q].values - 1);
            }
            if unset {
                state[v] = var.values;
                walk(net, v + 1, p, state, evidence, joint, total);
                return;
            }
            let probs = &var.rows[row];
            for (i, &q) in probs.iter().enumerate() {
                state[v] = i;
                walk(net, v + 1, p * q, state, evidence, joint, total);
            }
            let rest = 1.0 - probs.iter().sum::<f64>();
            if rest > 1e-12 {
                state[v] = var.values;
                walk(net, v + 1, p * rest, state, evidence, joint, total);
            }
        }
        walk(self, 0, 1.0, &mut state, evidence, &mut joint, &mut total);
        queries
            .iter()
            .flat_map(|&v| (0..self.vars[v].values).map(move |i| (v, i)))
            .map(|(v, i)| (self.atom(v, i), joint[v][i] / total))
            .collect()
    }

    /// The answers on the `families` vtrees, with the rows of a table as
    /// classes and apart, and under `auto`, against the enumeration; and
    /// how many sets are read in binary.
    fn check(&self, what: &str, evidence: &[(usize, usize, bool)], queries: &[usize]) -> usize {
        let source = self.source(evidence, queries);
        let expected = self.exact(evidence, queries);
        let all = common::ground(&source, Rows::Classes, true);
        let codes = all.encoding.codes.len();
        for strategy in [
            Strategy::Trial(Kind::Families { search: true }),
            Strategy::Trial(Kind::Families { search: false }),
        ] {
            let run = tddlog::solve_ground(
                all.clone(),
                &common::options(strategy, tddlog::Projection::default()),
            )
            .unwrap();
            assert_answers(
                &format!("{what} ({strategy})"),
                &run.results.expect("consistent evidence"),
                &expected,
                1e-12,
            );
        }
        let apart = common::ground(&source, Rows::Apart, true);
        assert_eq!(
            apart.encoding.codes.len(),
            codes,
            "{what}: the same sets in binary with the rows apart"
        );
        let run = tddlog::solve_ground(
            apart,
            &common::options(
                Strategy::Trial(Kind::Families { search: true }),
                tddlog::Projection::default(),
            ),
        )
        .unwrap();
        assert_answers(
            &format!("{what} (rows apart)"),
            &run.results.expect("consistent evidence"),
            &expected,
            1e-12,
        );
        let run = common::run(
            &source,
            &common::options(Strategy::Auto, tddlog::Projection::default()),
        )
        .unwrap();
        assert_answers(
            &format!("{what} (auto)"),
            &run.results.expect("consistent evidence"),
            &expected,
            1e-12,
        );
        codes
    }
}

// A table over two parents of many values is split on one, each part a
// table over the other ([`tddlog::indicators`]): the parts of `y` below, one
// per value of `x`, read `f` in binary.

#[test]
fn a_parent_of_many_values_read_in_binary() {
    let mut draw = Draw { state: 1 };
    let mut net = Network { vars: Vec::new() };
    let x = net.add(&mut draw, "x", 40, &[], |_| 1.0);
    let f = net.add(&mut draw, "f", 33, &[], |_| 1.0);
    let y = net.add(&mut draw, "y", 3, &[f, x], |_| 1.0);
    assert_eq!(
        net.check("plain", &[(y, 1, true)], &[f, y]),
        1,
        "f is read in binary"
    );
    assert_eq!(
        net.check("no evidence", &[], &[f, y]),
        1,
        "f is read in binary"
    );
}

#[test]
fn rows_that_leave_a_variable_without_a_value() {
    // f's row, and every third row of y, leave mass to no value: f's code
    // is then 0.
    let mut draw = Draw { state: 2 };
    let mut net = Network { vars: Vec::new() };
    let x = net.add(&mut draw, "x", 40, &[], |_| 1.0);
    let f = net.add(&mut draw, "f", 37, &[], |_| 0.875);
    let y = net.add(&mut draw, "y", 4, &[f, x], |r| {
        if r % 3 == 0 { 0.75 } else { 1.0 }
    });
    let w = net.add(&mut draw, "w", 2, &[y], |_| 1.0);
    assert_eq!(
        net.check("unset", &[(w, 0, true)], &[f, y]),
        1,
        "f is read in binary"
    );
    assert_eq!(
        net.check("unset, observed", &[(f, 4, false), (w, 1, true)], &[f, y]),
        1,
        "f is read in binary"
    );
}

#[test]
fn evidence_on_a_variable_read_in_binary() {
    let mut draw = Draw { state: 3 };
    let mut net = Network { vars: Vec::new() };
    let x = net.add(&mut draw, "x", 36, &[], |_| 1.0);
    let f = net.add(&mut draw, "f", 35, &[], |_| 1.0);
    let y = net.add(&mut draw, "y", 3, &[f, x], |_| 1.0);
    // Observed true, f is fixed (src/ground/condition.rs), and nothing
    // reads it.
    assert_eq!(net.check("observed true", &[(f, 7, true)], &[x, y]), 0);
    // Observed false at two values, f's row is renormalized over the 33
    // left (src/ground/condition.rs), still read in binary.
    assert_eq!(
        net.check(
            "observed false",
            &[(f, 7, false), (f, 3, false), (y, 1, true)],
            &[f, y]
        ),
        1
    );
}

#[test]
fn a_chain_of_variables_of_many_values() {
    // u reads x, its rows leaving it without a value now and then, and y
    // reads both: u's table whole reads x in binary, and y's parts, one per
    // value of x, read u in binary.
    let mut draw = Draw { state: 4 };
    let mut net = Network { vars: Vec::new() };
    let x = net.add(&mut draw, "x", 40, &[], |_| 1.0);
    let u = net.add(
        &mut draw,
        "u",
        33,
        &[x],
        |r| if r % 5 == 0 { 0.9 } else { 1.0 },
    );
    let y = net.add(&mut draw, "y", 3, &[u, x], |_| 1.0);
    assert_eq!(
        net.check("chain", &[(y, 2, true)], &[u, y]),
        2,
        "x and u are read in binary"
    );
}

#[test]
fn several_readers_of_one_variable() {
    // b reads f in binary in its parts on x; a's rows, each a distribution
    // of its own, read f and x in binary through a class atom each; c reads
    // f through a copy per value, its parts on f reading theirs alone.
    let mut draw = Draw { state: 5 };
    let mut net = Network { vars: Vec::new() };
    let x = net.add(&mut draw, "x", 40, &[], |_| 1.0);
    let f = net.add(&mut draw, "f", 34, &[], |_| 1.0);
    let a = net.add(&mut draw, "a", 2, &[f, x], |_| 1.0);
    let b = net.add(&mut draw, "b", 3, &[f, x], |_| 1.0);
    let c = net.add(&mut draw, "c", 2, &[a, b, f], |_| 1.0);
    // Evidence on a would fix it (src/ground/condition.rs), and c would
    // read its value.
    assert_eq!(
        net.check("readers", &[(c, 0, true)], &[f, b]),
        2,
        "f and x are read in binary"
    );
}

#[test]
fn a_variable_read_in_binary_whose_rows_read_parents_of_few_values() {
    // u's bits are derived from u's rows, which read x and z through copies
    // with the other values' copies negated; the bits' clause loops hold
    // the at-most-one of each parent's copies and leave the negations out
    // ([`tddlog::indicators`]). Its rows leave u without a value now and
    // then.
    let mut draw = Draw { state: 6 };
    let mut net = Network { vars: Vec::new() };
    let x = net.add(&mut draw, "x", 3, &[], |_| 1.0);
    let z = net.add(&mut draw, "z", 5, &[], |_| 1.0);
    let u = net.add(&mut draw, "u", 33, &[x, z], |r| {
        if r % 4 == 0 { 0.9 } else { 1.0 }
    });
    let v = net.add(&mut draw, "v", 40, &[], |_| 1.0);
    // y's table is split on v, and its parts read u in binary.
    let y = net.add(&mut draw, "y", 3, &[u, v], |_| 1.0);
    assert_eq!(
        net.check("siblings", &[(y, 1, true)], &[x, u]),
        1,
        "u is read in binary"
    );
    assert_eq!(
        net.check("siblings, no evidence", &[], &[z, y]),
        1,
        "u is read in binary"
    );
    assert_eq!(
        net.check(
            "siblings, u observed",
            &[(u, 4, false), (y, 0, true)],
            &[x, z]
        ),
        1,
        "u is read in binary"
    );
    let source = net.source(&[(y, 1, true)], &[x, u]);
    let all = common::ground(&source, Rows::Classes, true);
    let run = tddlog::solve_ground(
        all,
        &common::options(
            Strategy::Trial(Kind::Families { search: true }),
            tddlog::Projection::default(),
        ),
    )
    .unwrap();
    assert!(
        run.stats.clause_memo.seeded > 0,
        "no loop held its sibling groups"
    );
}

/// The sum of digits as in the literature's mnist_sum, a digit per row
/// of `rows`: `partial(k,S)` is the sum of the first k digits, and `sum`
/// that of all of them. The sum of k digits has 9k + 1 values, and from
/// four digits on it is read in binary by the next.
fn running_sum(rows: &[Vec<f64>]) -> String {
    let digits = rows.len();
    let mut source = String::new();
    for (k, row) in rows.iter().enumerate() {
        let heads: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(d, p)| format!("{p}::digit({},{d})", k + 1))
            .collect();
        source += &format!("{}.\n", heads.join("; "));
    }
    for k in 1..digits {
        source += &format!("next({k},{}).\n", k + 1);
    }
    source += "partial(1,S) :- digit(1,S).\n";
    source += "partial(K,S) :- next(J,K), partial(J,S0), digit(K,D), S is S0+D.\n";
    source += &format!("sum(S) :- partial({digits},S).\n");
    source
}

/// The distribution of the sum of digits drawn from `rows`, convolved
/// digit by digit.
fn convolved(rows: &[Vec<f64>]) -> Vec<f64> {
    let mut sum = vec![1.0];
    for row in rows {
        let mut next = vec![0.0; sum.len() + 9];
        for (s, &p) in sum.iter().enumerate() {
            for (d, &q) in row.iter().enumerate() {
                next[s + d] += p * q;
            }
        }
        sum = next;
    }
    sum
}

#[test]
fn a_running_sum_read_in_binary_merges_its_bits_rows() {
    // The sum of seven digits, as in the literature's mnist_sum: the sum of
    // k digits has 9k + 1 values, and from four digits on it is read in
    // binary by the next. A bit's rules are the rows of the values whose
    // code has the bit, and two that differ only in one bit of the code of
    // the sum they read are one rule without it ([`tddlog::indicators`]).
    let digits = 7;
    let mut draw = Draw { state: 7 };
    let rows: Vec<Vec<f64>> = (0..digits).map(|_| draw.row(10, 0, 1.0)).collect();
    let mut source = running_sum(&rows);
    let top = 9 * digits;
    for s in 0..=top {
        source += &format!("query(sum({s})).\n");
    }
    let expected: Vec<(String, f64)> = convolved(&rows)
        .iter()
        .enumerate()
        .map(|(s, &p)| (format!("sum({s})"), p))
        .collect();
    let all = common::ground(&source, Rows::Classes, true);
    assert!(!all.encoding.codes.is_empty(), "no sum is read in binary");
    // Unmerged, each row of a value would be a rule of every bit its code
    // has, at least one rule per row: the sum of k digits has a row per
    // value of the sum of k - 1 and digit. A bit is named by its set's
    // first value, `'$b'('partial(k,0)',j)`. The sums of four digits and
    // more are read in binary, by the next sum and by `sum`, and the rows
    // of those of five and more read a sum in binary.
    let mut bit_rules = std::collections::BTreeMap::new();
    for r in &all.rules {
        let head = all.atoms[r.head as usize].to_string();
        if let Some(rest) = head.strip_prefix("'$b'('partial(") {
            *bit_rules
                .entry(rest.split(',').next().unwrap().parse::<usize>().unwrap())
                .or_insert(0) += 1;
        }
    }
    assert_eq!(
        bit_rules.keys().copied().collect::<Vec<_>>(),
        [4, 5, 6, 7],
        "the sums read in binary"
    );
    for k in [5, 6, 7] {
        let rows = 10 * (9 * (k - 1) + 1);
        assert!(
            bit_rules[&k] < rows,
            "the sum of {k} digits: {} rules of bits against {rows} rows",
            bit_rules[&k]
        );
    }
    for strategy in [
        Strategy::Trial(Kind::Families { search: true }),
        Strategy::Trial(Kind::Families { search: false }),
    ] {
        let run = tddlog::solve_ground(
            all.clone(),
            &common::options(strategy, tddlog::Projection::default()),
        )
        .unwrap();
        assert_answers(
            &format!("running sum ({strategy})"),
            &run.results.expect("no evidence"),
            &expected,
            1e-12,
        );
    }
    let run = common::run(
        &source,
        &common::options(Strategy::Auto, tddlog::Projection::default()),
    )
    .unwrap();
    assert_answers(
        "running sum (auto)",
        &run.results.expect("no evidence"),
        &expected,
        1e-12,
    );
}

#[test]
fn queried_values_of_a_set_read_in_binary() {
    // The sum of five digits is read in binary by the sum of six, and its
    // own rows read the sum of four in binary. Queried, its values are the
    // cubes of their codes over a reader's copies of its bits
    // ([`tddlog::indicators`]); with their own rules kept, bn_mildew with
    // dm_4's values queried ran out of memory in the compile.
    let mut draw = Draw { state: 8 };
    let rows: Vec<Vec<f64>> = (0..7).map(|_| draw.row(10, 0, 1.0)).collect();
    let mut source = running_sum(&rows);
    for s in 0..=45 {
        source += &format!("query(partial(5,{s})).\n");
    }
    source += "query(sum(30)).\n";
    let mut expected: Vec<(String, f64)> = convolved(&rows[..5])
        .iter()
        .enumerate()
        .map(|(s, &p)| (format!("partial(5,{s})"), p))
        .collect();
    expected.push(("sum(30)".into(), convolved(&rows)[30]));
    let all = common::ground(&source, Rows::Classes, true);
    let copies: std::collections::HashMap<u32, String> = all
        .encoding
        .indicators
        .iter()
        .map(|i| (i.var, all.atoms[i.atom as usize].to_string()))
        .collect();
    for s in 0..=45 {
        let name = format!("partial(5,{s})");
        let rules: Vec<_> = all
            .rules
            .iter()
            .filter(|r| all.atoms[r.head as usize].to_string() == name)
            .collect();
        assert_eq!(rules.len(), 1, "{name}: one rule");
        assert_eq!(
            rules[0].body.len(),
            6,
            "{name}: a literal per bit of 46 values' codes"
        );
        for l in &rules[0].body {
            let copied = copies
                .get(&l.atom())
                .unwrap_or_else(|| panic!("{name} reads an atom that is no copy"));
            assert!(
                copied.starts_with("'$b'('partial(5,0)',"),
                "{name} reads a copy of {copied}"
            );
        }
    }
    for strategy in [
        Strategy::Trial(Kind::Families { search: true }),
        Strategy::Trial(Kind::Families { search: false }),
    ] {
        let run = tddlog::solve_ground(
            all.clone(),
            &common::options(strategy, tddlog::Projection::default()),
        )
        .unwrap();
        assert_answers(
            &format!("queried sum ({strategy})"),
            &run.results.expect("no evidence"),
            &expected,
            1e-12,
        );
    }
    let run = common::run(
        &source,
        &common::options(Strategy::Auto, tddlog::Projection::default()),
    )
    .unwrap();
    assert_answers(
        "queried sum (auto)",
        &run.results.expect("no evidence"),
        &expected,
        1e-12,
    );
}
