//! Tables split on a parent of many values ([`tddlog::indicators`]), whose
//! parts read the parent's copy of their value alone and go side by side
//! in the `families` vtree, against answers computed by enumeration.
use crate::common;

use common::Rows;
use tddlog::{Kind, Strategy};

/// p with n values, r(p), s(p, r) and t(p, r, s), as `fault`, f97, f78 and
/// f3 of bn_pathfinder, and `leaves` tables c0(p), c1(p), … read by p
/// alone and by nothing, one unit, of which the first `observed` are
/// observed `yes`. Returns the program and the exact P(q | e) of each
/// query: r(a) and the values of t.
fn hub(n: usize, leaves: usize, observed: usize) -> (String, Vec<(String, f64)>) {
    let weight: Vec<f64> = (0..n).map(|i| (i % 5 + 1) as f64).collect();
    let total: f64 = weight.iter().sum();
    let prior: Vec<f64> = weight.iter().map(|w| w / total).collect();
    let two = [[0.9, 0.1], [0.3, 0.7]];
    let three = [[0.2, 0.3, 0.5], [0.5, 0.3, 0.2], [0.1, 0.1, 0.8]];
    let yes = [0.125, 0.25, 0.5, 0.75, 0.875];
    let (rv, sv) = (["a", "b"], ["x", "y", "z"]);
    let mut source = prior
        .iter()
        .enumerate()
        .map(|(i, q)| format!("{q}::p(v{i})"))
        .collect::<Vec<_>>()
        .join("; ")
        + ".\n";
    let mut k = 0;
    let mut r_of = vec![[0.0; 2]; n];
    for (i, d) in r_of.iter_mut().enumerate() {
        *d = two[k % 2];
        k += 1;
        source += &format!("{}::r(a); {}::r(b) :- p(v{i}).\n", d[0], d[1]);
    }
    let mut s_of = vec![[[0.0; 3]; 2]; n];
    for (i, rows) in s_of.iter_mut().enumerate() {
        for (j, d) in rows.iter_mut().enumerate() {
            *d = three[k % 3];
            k += 1;
            source += &format!(
                "{}::s(x); {}::s(y); {}::s(z) :- p(v{i}), r({}).\n",
                d[0], d[1], d[2], rv[j]
            );
        }
    }
    let mut t_of = vec![[[[0.0; 3]; 3]; 2]; n];
    for (i, rows) in t_of.iter_mut().enumerate() {
        for (j, row) in rows.iter_mut().enumerate() {
            for (l, d) in row.iter_mut().enumerate() {
                *d = three[k % 3];
                k += 1;
                source += &format!(
                    "{}::t(x); {}::t(y); {}::t(z) :- p(v{i}), r({}), s({}).\n",
                    d[0], d[1], d[2], rv[j], sv[l]
                );
            }
        }
    }
    let mut likelihood = vec![1.0; n];
    for c in 0..leaves {
        for (i, like) in likelihood.iter_mut().enumerate() {
            let q = yes[(i * 3 + c) % yes.len()];
            source += &format!("{q}::c{c}(yes); {}::c{c}(no) :- p(v{i}).\n", 1.0 - q);
            if c < observed {
                *like *= q;
            }
        }
        if c < observed {
            source += &format!("evidence(c{c}(yes)).\n");
        }
    }
    let w: Vec<f64> = (0..n).map(|i| prior[i] * likelihood[i]).collect();
    let evidence: f64 = w.iter().sum();
    let mut expected = vec![(
        "r(a)".to_string(),
        (0..n).map(|i| w[i] * r_of[i][0]).sum::<f64>() / evidence,
    )];
    for (m, v) in sv.iter().enumerate() {
        let mut p = 0.0;
        for i in 0..n {
            for j in 0..2 {
                for l in 0..3 {
                    p += w[i] * r_of[i][j] * s_of[i][j][l] * t_of[i][j][l][m];
                }
            }
        }
        expected.push((format!("t({v})"), p / evidence));
    }
    for (q, _) in &expected {
        source += &format!("query({q}).\n");
    }
    (source, expected)
}

/// A chain p0 → p1 → … → p{k-1} of variables of n values, each table
/// over the one before alone, as bn_mildew's dm_1 to dm_4, and a leaf of
/// p0 observed `yes`; with `mapped` values or more, p{k} maps the values
/// of p{k-1} onto them, value m to m mod `mapped`, as `udbytte` reads
/// `dm_4`. Returns the program and the exact P(q | e) of each value of the
/// last.
fn chain(n: usize, k: usize, mapped: usize) -> (String, Vec<(String, f64)>) {
    let row = |seed: usize| -> Vec<f64> {
        let w: Vec<f64> = (0..n)
            .map(|m| ((seed * 7 + m * 3) % 5 + 1) as f64)
            .collect();
        let total: f64 = w.iter().sum();
        w.iter().map(|x| x / total).collect()
    };
    let yes = [0.125, 0.25, 0.5, 0.75, 0.875];
    let prior = row(0);
    let mut source = prior
        .iter()
        .enumerate()
        .map(|(i, q)| format!("{q}::p0(v{i})"))
        .collect::<Vec<_>>()
        .join("; ")
        + ".\n";
    let mut dist: Vec<f64> = (0..n).map(|i| prior[i] * yes[i % yes.len()]).collect();
    for i in 0..n {
        let q = yes[i % yes.len()];
        source += &format!("{q}::leaf(yes); {}::leaf(no) :- p0(v{i}).\n", 1.0 - q);
    }
    source += "evidence(leaf(yes)).\n";
    for j in 1..k {
        let mut next = vec![0.0; n];
        for (i, d) in dist.iter().enumerate() {
            let r = row(j * n + i);
            source += &(r
                .iter()
                .enumerate()
                .map(|(m, q)| format!("{q}::p{j}(v{m})"))
                .collect::<Vec<_>>()
                .join("; ")
                + &format!(" :- p{}(v{i}).\n", j - 1));
            for (m, q) in r.iter().enumerate() {
                next[m] += d * q;
            }
        }
        dist = next;
    }
    let evidence: f64 = dist.iter().sum();
    let expected: Vec<(String, f64)> = if mapped == 0 {
        dist.iter()
            .enumerate()
            .map(|(m, d)| (format!("p{}(v{m})", k - 1), d / evidence))
            .collect()
    } else {
        for m in 0..n {
            source += &format!("p{k}(y{}) :- p{}(v{m}).\n", m % mapped, k - 1);
        }
        (0..mapped)
            .map(|y| {
                (
                    format!("p{k}(y{y})"),
                    dist.iter().skip(y).step_by(mapped).sum::<f64>() / evidence,
                )
            })
            .collect()
    };
    for (q, _) in &expected {
        source += &format!("query({q}).\n");
    }
    (source, expected)
}

/// Parents a, b and d of `na`, `nb` and `nd` values and their child c(a,
/// b, d) of eight, which a leaf observed `yes` reads: rows enough, of seven
/// switches each, that the table's parts are split again on another
/// parent. Returns the program and the exact P(q | e) of each value of a
/// and of c.
fn grid(na: usize, nb: usize, nd: usize) -> (String, Vec<(String, f64)>) {
    let dist = |n: usize, seed: usize| -> Vec<f64> {
        let w: Vec<f64> = (0..n)
            .map(|m| ((seed * 7 + m * 3) % 5 + 1) as f64)
            .collect();
        let total: f64 = w.iter().sum();
        w.iter().map(|x| x / total).collect()
    };
    let yes = [0.125, 0.25, 0.5, 0.75, 0.875];
    let cv = 8;
    let mut source = String::new();
    let (pa, pb, pd) = (dist(na, 1), dist(nb, 2), dist(nd, 3));
    for (name, prior) in [("a", &pa), ("b", &pb), ("d", &pd)] {
        source += &(prior
            .iter()
            .enumerate()
            .map(|(i, q)| format!("{q}::{name}(v{i})"))
            .collect::<Vec<_>>()
            .join("; ")
            + ".\n");
    }
    let (mut wa, mut wc) = (vec![0.0; na], vec![0.0; cv]);
    for i in 0..na {
        for (j, &qb) in pb.iter().enumerate() {
            for (l, &qd) in pd.iter().enumerate() {
                let row = dist(cv, (i * nb + j) * nd + l + 4);
                source += &(row
                    .iter()
                    .enumerate()
                    .map(|(m, q)| format!("{q}::c(w{m})"))
                    .collect::<Vec<_>>()
                    .join("; ")
                    + &format!(" :- a(v{i}), b(v{j}), d(v{l}).\n"));
                for (m, q) in row.iter().enumerate() {
                    let w = pa[i] * qb * qd * q * yes[m % yes.len()];
                    wa[i] += w;
                    wc[m] += w;
                }
            }
        }
    }
    for m in 0..cv {
        let q = yes[m % yes.len()];
        source += &format!("{q}::e(yes); {}::e(no) :- c(w{m}).\n", 1.0 - q);
    }
    source += "evidence(e(yes)).\n";
    let evidence: f64 = wc.iter().sum();
    let mut expected: Vec<(String, f64)> = wa
        .iter()
        .enumerate()
        .map(|(i, w)| (format!("a(v{i})"), w / evidence))
        .collect();
    expected.extend(
        wc.iter()
            .enumerate()
            .map(|(m, w)| (format!("c(w{m})"), w / evidence)),
    );
    for (q, _) in &expected {
        source += &format!("query({q}).\n");
    }
    (source, expected)
}

/// The run of `source` on the `families` vtree under every indicator,
/// its answers checked against `expected`, and its largest compiled
/// formula, largest formula of the query phase and evidence, in pairs.
fn run(source: &str, expected: &[(String, f64)], what: &str) -> (usize, usize, usize) {
    let gp = common::ground(source, Rows::Classes, true);
    let strategy = Strategy::Trial(Kind::Families { search: true });
    let run = tddlog::solve_ground(
        gp,
        &common::options(strategy, tddlog::Projection::default()),
    )
    .unwrap_or_else(|e| panic!("{what}: {e}"));
    common::assert_answers(what, &run.results.unwrap(), expected, 1e-9);
    (
        run.stats.max_pairs,
        run.query_sizes.largest,
        run.query_sizes.evidence,
    )
}

#[test]
fn a_table_split_on_a_parent_of_many_values_keeps_its_answers() {
    // Below sixteen values the parts read the other values' copies
    // negated; from sixteen on, their own alone, and so do the leaves,
    // one unit of several tables, where one of them is observed but not
    // where three are.
    for n in [9, 20, 40] {
        for observed in [1, 3] {
            let (source, expected) = hub(n, 4, observed);
            run(
                &source,
                &expected,
                &format!("{n} values, {observed} observed"),
            );
        }
    }
}

#[test]
fn the_parts_of_a_split_table_go_side_by_side() {
    // Side by side, a node over some of the parts has a class for each
    // part under it; on a spine, one for each part still to come: the
    // query phase's largest formula took 24.6 k pairs at 32 values and
    // 94.4 k at 64 so, against 6.3 k and 15.6 k. The compile's largest
    // formula holds the tables' clauses, which grow with their rows (1.2 k
    // pairs to 4.5 k), and is not compared.
    let sizes: Vec<(usize, usize, usize)> = [32, 64]
        .into_iter()
        .map(|n| {
            let (source, expected) = hub(n, 0, 0);
            run(&source, &expected, &format!("{n} values"))
        })
        .collect();
    let (small, large) = (sizes[0], sizes[1]);
    assert!(
        large.1 < 3 * small.1 && large.2 < 3 * small.2,
        "32 values to 64: {small:?} to {large:?}"
    );
}

#[test]
fn observations_of_tables_that_share_a_unit_do_not_multiply() {
    // The leaves are one unit, their tables read through the same copies
    // of p. Read alone, a part's copy would leave the observations free
    // of each other until p's links reach them, a class for every subset
    // of them satisfied at every node over the parts: the query phase's
    // largest formula took 77 k pairs with 6 observed and 303 M with 12
    // so, against 8.3 k and 13 k.
    let sizes: Vec<(usize, usize, usize)> = [6, 12]
        .into_iter()
        .map(|observed| {
            let (source, expected) = hub(20, 12, observed);
            run(&source, &expected, &format!("{observed} observed"))
        })
        .collect();
    let (few, many) = (sizes[0], sizes[1]);
    assert!(
        many.1 < 3 * few.1 && many.2 < 3 * few.2,
        "6 observed to 12: {few:?} to {many:?}"
    );
}

#[test]
fn a_table_of_many_values_that_another_reads_keeps_its_negations() {
    // p1 is split on p0, and p2 either is split on p1 or maps p1's values
    // onto four of its own, all of sixteen values. Read alone, p1's values
    // would be tied to p2's copies before anything says that p1's copies
    // of p0 exclude each other: the evidence and query phase passed 8 GB
    // so, either way, where with the negations its largest formula took
    // 66 k pairs. A table that nothing reads reads its parent's
    // value alone.
    for (k, mapped) in [(3, 0), (2, 4)] {
        let (source, expected) = chain(16, k, mapped);
        let gp = common::ground(&source, Rows::Classes, true);
        let copies: std::collections::HashSet<u32> =
            gp.encoding.indicators.iter().map(|i| i.var).collect();
        // A table's rows are its rules, or, tied to a reader by clauses, the
        // rules of the reader's link `'$c'(value, reader)`; a clause also
        // negates the reader's copies of the table's own values.
        let negates = |table: &str| {
            let link = format!("'$c'('{table}");
            let own = format!("'$v'('{table}");
            gp.rules
                .iter()
                .filter(|r| {
                    let head = gp.atoms[r.head as usize].to_string();
                    head.starts_with(table) || head.starts_with(&link)
                })
                .any(|r| {
                    r.body.iter().any(|l| {
                        !l.is_positive()
                            && copies.contains(&l.atom())
                            && !gp.atoms[l.atom() as usize].to_string().starts_with(&own)
                    })
                })
        };
        assert!(negates("p1("), "p1 reads p0's values alone");
        if mapped == 0 {
            assert!(!negates("p2("), "p2 reads p1's other values negated");
        }
        run(
            &source,
            &expected,
            &format!("{k} tables, map onto {mapped}"),
        );
    }
}

#[test]
fn the_parts_of_a_split_table_are_split_again() {
    // c is split on b, and each of b's parts on d. With 16 values, b's
    // parts read their value alone. c's rows repeat five distributions, and
    // the split is of its rows as written, without row classes; the answers
    // are checked with them as well.
    for (na, nb, nd) in [(4, 8, 8), (4, 16, 8)] {
        let (source, expected) = grid(na, nb, nd);
        let gp = common::ground(&source, Rows::Apart, true);
        // A copy's name holds the atom, the unit and the value of each
        // split parent on the way to the part.
        assert!(
            gp.atoms
                .iter()
                .any(|a| &*a.name == "$v" && a.args.len() == 4),
            "{na}, {nb}, {nd}: c's parts are not split again"
        );
        run(&source, &expected, &format!("{na}, {nb}, {nd}"));
    }
}

#[test]
fn the_parts_of_a_part_go_side_by_side() {
    // c is split on b and each of b's parts on d; the parts of a part all
    // read the copy of b's value they were split on, and go side by side
    // above it. With d's values doubled from 8 to 16, the evidence grew
    // 2.09 times and the query phase's work 2.09 times; with the parts of
    // a part on a spine, 2.38 and 2.56 times.
    let sizes: Vec<(usize, u64)> = [8, 16]
        .into_iter()
        .map(|nd| {
            let (source, expected) = grid(4, 16, nd);
            let gp = common::ground(&source, Rows::Apart, true);
            assert!(
                gp.atoms
                    .iter()
                    .any(|a| &*a.name == "$v" && a.args.len() == 4),
                "{nd}: c's parts are not split again"
            );
            let strategy = Strategy::Trial(Kind::Families { search: true });
            let run = tddlog::solve_ground(
                gp,
                &common::options(strategy, tddlog::Projection::default()),
            )
            .unwrap();
            common::assert_answers(
                &format!("d of {nd}"),
                &run.results.unwrap(),
                &expected,
                1e-9,
            );
            (run.query_sizes.evidence, run.query_sizes.work_units)
        })
        .collect();
    let (few, many) = (sizes[0], sizes[1]);
    assert!(
        4 * many.0 < 9 * few.0 && 10 * many.1 < 23 * few.1,
        "d of 8 to 16: {few:?} to {many:?}"
    );
}

/// Parents of `dims` values each and their child c of `cv` values, each
/// row of c a distribution of its own, and a leaf observed `yes` that reads
/// c. Returns the program and the exact P(q | e) of each value of the
/// first parent and of c.
fn spines(dims: &[usize], cv: usize) -> (String, Vec<(String, f64)>) {
    // The last value's weight against the first's grows with the seed.
    let dist = |n: usize, seed: usize| -> Vec<f64> {
        let w: Vec<f64> = (0..n)
            .map(|m| (1 + seed * (m + 1) + m * m) as f64)
            .collect();
        let total: f64 = w.iter().sum();
        w.iter().map(|x| x / total).collect()
    };
    let yes = [0.125, 0.25, 0.5, 0.75, 0.875];
    let names: Vec<String> = (0..dims.len()).map(|i| format!("a{i}")).collect();
    let priors: Vec<Vec<f64>> = dims
        .iter()
        .enumerate()
        .map(|(i, &n)| dist(n, i + 1))
        .collect();
    let mut source = String::new();
    for (name, prior) in names.iter().zip(&priors) {
        source += &(prior
            .iter()
            .enumerate()
            .map(|(v, q)| format!("{q}::{name}(v{v})"))
            .collect::<Vec<_>>()
            .join("; ")
            + ".\n");
    }
    let rows: usize = dims.iter().product();
    let (mut wa, mut wc) = (vec![0.0; dims[0]], vec![0.0; cv]);
    for row in 0..rows {
        // The row's value of each parent, the last parent's fastest.
        let mut values = vec![0; dims.len()];
        let mut rest = row;
        for (i, &n) in dims.iter().enumerate().rev() {
            values[i] = rest % n;
            rest /= n;
        }
        let context = names
            .iter()
            .zip(&values)
            .map(|(name, v)| format!("{name}(v{v})"))
            .collect::<Vec<_>>()
            .join(", ");
        let prior: f64 = values.iter().zip(&priors).map(|(&v, p)| p[v]).product();
        let d = dist(cv, row + dims.len() + 1);
        source += &(d
            .iter()
            .enumerate()
            .map(|(m, q)| format!("{q}::c(w{m})"))
            .collect::<Vec<_>>()
            .join("; ")
            + &format!(" :- {context}.\n"));
        for (m, q) in d.iter().enumerate() {
            let w = prior * q * yes[m % yes.len()];
            wa[values[0]] += w;
            wc[m] += w;
        }
    }
    for m in 0..cv {
        let q = yes[m % yes.len()];
        source += &format!("{q}::e(yes); {}::e(no) :- c(w{m}).\n", 1.0 - q);
    }
    source += "evidence(e(yes)).\n";
    let evidence: f64 = wc.iter().sum();
    let mut expected: Vec<(String, f64)> = wa
        .iter()
        .enumerate()
        .map(|(v, w)| (format!("a0(v{v})"), w / evidence))
        .collect();
    expected.extend(
        wc.iter()
            .enumerate()
            .map(|(m, w)| (format!("c(w{m})"), w / evidence)),
    );
    for (q, _) in &expected {
        source += &format!("query({q}).\n");
    }
    (source, expected)
}

#[test]
fn row_classes_select_the_rows_of_a_long_spine() {
    // c's 48 rows over three parents, of 3, 4 and 4 values, each a
    // distribution of its own: split on a parent of 4 values, a part keeps
    // 12 rows on a spine of their switches. Rewritten, a class atom per row
    // selects it, the table over the classes is split on them and its parts
    // go side by side. c's 144 rows over two parents of 12 values are not
    // rewritten.
    for (dims, rewritten) in [(vec![3, 4, 4], true), (vec![12, 12], false)] {
        let (source, expected) = spines(&dims, 3);
        let mut sizes = Vec::new();
        for rows in [Rows::Apart, Rows::Classes] {
            let gp = common::ground(&source, rows, true);
            let classes = gp.atoms.iter().any(|a| a.name.starts_with("$c_c"));
            assert_eq!(
                classes,
                rewritten && rows == Rows::Classes,
                "{dims:?}, {rows:?}: c's class atoms"
            );
            let strategy = Strategy::Trial(Kind::Families { search: true });
            let run = tddlog::solve_ground(
                gp,
                &common::options(strategy, tddlog::Projection::default()),
            )
            .unwrap();
            common::assert_answers(
                &format!("{dims:?}, {rows:?}"),
                &run.results.unwrap(),
                &expected,
                1e-9,
            );
            sizes.push(run.query_sizes.evidence);
        }
        if rewritten {
            assert!(
                3 * sizes[1] < sizes[0],
                "{dims:?}: the evidence apart and with classes, {sizes:?}"
            );
        }
    }
}
