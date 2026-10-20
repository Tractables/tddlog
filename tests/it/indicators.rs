//! Reading derived atoms through indicators changes the decomposition, not
//! the probabilities.
use crate::common;

use common::{Rows, assert_probabilities};
use tddlog::{Kind, Strategy};

#[test]
fn a_chain_of_units_read_through_copies_keeps_its_conditional_probabilities() {
    // Forty units, each reading the one before through a copy of its own,
    // with an early one observed false. A predicate per link: a recursive
    // `a(I) :- a(I-1)` is ground from its bound ends, where each query is a
    // flat disjunction of facts (`ground::linear`) with nothing to cut.
    let mut source = String::from("0.1::f0. a0 :- f0.\n");
    for i in 1..40 {
        source.push_str(&format!("0.1::f{i}. a{i} :- f{i}. a{i} :- a{}.\n", i - 1));
    }
    source.push_str("evidence(a1,false). query(a19). query(a39).\n");
    let all = common::ground(&source, Rows::Classes, true);
    // The evidence fixes a0 and a1 (src/ground/condition.rs): 38 units are
    // left, a2 to a39.
    assert!(
        all.encoding.indicators.len() >= 37,
        "{} copies",
        all.encoding.indicators.len()
    );
    let expected = [18, 38].map(|n| 1.0 - 0.9_f64.powi(n));
    for strategy in [
        Strategy::Trial(Kind::Families { search: true }),
        Strategy::Trial(Kind::Closed),
    ] {
        let run = tddlog::solve_ground(
            all.clone(),
            &common::options(strategy, tddlog::Projection::default()),
        )
        .unwrap();
        assert_probabilities(
            &format!("{strategy:?}"),
            &run.results.unwrap(),
            &expected,
            1e-12,
        );
    }
}

/// The copies of `atom` in `gp`.
fn copies_of(gp: &tddlog::GroundProgram, atom: &str) -> usize {
    gp.encoding
        .indicators
        .iter()
        .filter(|i| gp.atoms[i.atom as usize].to_string() == atom)
        .count()
}

#[test]
fn each_reader_of_a_shared_definition_reads_a_copy_of_its_own() {
    let source = "0.5::a. 0.5::b. 0.5::c. 0.5::d. 0.5::e. 0.5::f.
                  shared :- a,b. l :- c,e. r :- d,f.
                  left :- shared,l. right :- shared,r.
                  query(left). query(right).";
    // `shared`, one rule over facts, would be inlined into its readers.
    let all = common::ground_as_written(source, Rows::Classes, true);
    assert_eq!(copies_of(&all, "shared"), 2);
    let run = tddlog::solve_ground(
        all,
        &common::options(Strategy::Trial(Kind::Closed), tddlog::Projection::default()),
    )
    .unwrap();
    assert_probabilities("closed", &run.results.unwrap(), &[0.0625; 2], 1e-12);
    // Seven readers, one of them observed false, which fixes no atom.
    let mut source = String::from("0.5::a. 0.5::b. shared :- a,b.\n");
    for i in 0..7 {
        source.push_str(&format!(
            "0.5::c({i}). 0.5::d({i}). local({i}) :- c({i}),d({i}).
             out({i}) :- shared,local({i}). query(out({i})).\n"
        ));
    }
    source.push_str("evidence(out(0),false).\n");
    let all = common::ground_as_written(&source, Rows::Classes, true);
    assert_eq!(copies_of(&all, "shared"), 7);
    let run = tddlog::solve_ground(
        all,
        &common::options(
            Strategy::Trial(Kind::Families { search: true }),
            tddlog::Projection::default(),
        ),
    )
    .unwrap();
    // P(out(i), not out(0)) = 1/4 * 1/4 * 3/4 over P(not out(0)) = 15/16.
    let expected: Vec<f64> = (0..7).map(|i| if i == 0 { 0.0 } else { 0.05 }).collect();
    assert_probabilities("families", &run.results.unwrap(), &expected, 1e-12);
}

#[test]
fn tables_as_clauses_keep_the_rows_a_reader_of_some_values_reads() {
    // x is a table read by y, which reads every value, and by z, which
    // reads x(a) and x(b) and, through v, is a unit of its own. y's copies
    // are tied to x by clauses; z's copies are tied to x(a) and x(b), which
    // keep their rows, where x(c) reads y's copy.
    let source = "0.2::x(a); 0.3::x(b); 0.5::x(c).
                  0.6::v(a); 0.3::v(b); 0.1::v(c).
                  0.9::y(t); 0.1::y(f) :- x(a).
                  0.4::y(t); 0.6::y(f) :- x(b).
                  0.7::y(t); 0.3::y(f) :- x(c).
                  0.8::z(t); 0.2::z(f) :- x(a), v(a).
                  0.3::z(t); 0.7::z(f) :- x(b).
                  evidence(y(t)). query(x(a)). query(x(b)). query(z(t)). query(v(b)).";
    let clauses = common::ground(source, Rows::Classes, true);
    let atom = |name: &str| {
        clauses
            .atoms
            .iter()
            .position(|a| a.to_string() == name)
            .unwrap() as u32
    };
    let defined_by_copy = |k: u32| {
        let rules: Vec<_> = clauses.rules.iter().filter(|r| r.head == k).collect();
        rules.len() == 1
            && rules[0].body.len() == 1
            && clauses
                .encoding
                .indicators
                .iter()
                .any(|i| i.var == rules[0].body[0].atom())
    };
    assert!(
        clauses
            .evidence
            .iter()
            .any(|(a, _, value)| !value && a.to_string().starts_with("'$c'")),
        "x's clauses tie it to y"
    );
    assert!(
        !defined_by_copy(atom("x(a)")) && !defined_by_copy(atom("x(b)")),
        "z reads them through links"
    );
    assert!(defined_by_copy(atom("x(c)")));
    // P(y(t)) = 0.2·0.9 + 0.3·0.4 + 0.5·0.7 = 0.65, and
    // P(z(t), y(t)) = 0.2·0.9·0.6·0.8 + 0.3·0.4·0.3.
    let expected = [0.18 / 0.65, 0.12 / 0.65, (0.0864 + 0.036) / 0.65, 0.3];
    for strategy in [
        Strategy::Trial(Kind::Families { search: true }),
        Strategy::Trial(Kind::Closed),
    ] {
        let run = tddlog::solve_ground(
            clauses.clone(),
            &common::options(strategy, tddlog::Projection::default()),
        )
        .unwrap();
        assert_probabilities(
            &format!("{strategy:?}"),
            &run.results.unwrap(),
            &expected,
            1e-12,
        );
    }
}

#[test]
fn a_distribution_that_sums_to_one_up_to_rounding_is_a_table() {
    // x's probabilities, given to nine decimals, sum to 1 - 1e-9. Read as
    // a full distribution, divided by their sum, x is a table and so is y,
    // which reads every value of x, and both are tied to their readers by
    // clauses. With a residual of no value, neither would be.
    let source = "0.2::x(a); 0.3::x(b); 0.499999999::x(c).
                  0.9::y(t); 0.1::y(f) :- x(a).
                  0.4::y(t); 0.6::y(f) :- x(b).
                  0.7::y(t); 0.3::y(f) :- x(c).
                  0.8::z(t); 0.2::z(f) :- y(t).
                  0.3::z(t); 0.7::z(f) :- y(f).
                  evidence(z(t)). query(x(a)). query(y(t)).";
    let clauses = common::ground(source, Rows::Classes, true);
    for value in ["x(a)", "x(b)", "x(c)", "y(t)", "y(f)"] {
        let links: Vec<&str> = clauses
            .encoding
            .indicators
            .iter()
            .filter(|i| clauses.atoms[i.atom as usize].to_string() == value)
            .map(|i| &*clauses.atoms[i.link as usize].name)
            .collect();
        assert!(
            !links.is_empty() && links.iter().all(|&l| l == "$c"),
            "{value}: {links:?}"
        );
    }
    let total = 0.999999999;
    let x = [0.2 / total, 0.3 / total, 0.499999999 / total];
    let y_given_x = [0.9, 0.4, 0.7];
    let y: f64 = x.iter().zip(y_given_x).map(|(p, q)| p * q).sum();
    let z = 0.8 * y + 0.3 * (1.0 - y);
    let expected = [x[0] * (0.9 * 0.8 + 0.1 * 0.3) / z, 0.8 * y / z];
    for strategy in [
        Strategy::Trial(Kind::Families { search: true }),
        Strategy::Trial(Kind::Closed),
    ] {
        let run = tddlog::solve_ground(
            clauses.clone(),
            &common::options(strategy, tddlog::Projection::default()),
        )
        .unwrap();
        assert_probabilities(
            &format!("{strategy:?}"),
            &run.results.unwrap(),
            &expected,
            1e-12,
        );
    }
}

/// y reads x's 48 values and q's 48, 2 304 rows of `values` values, the
/// first with a probability of its own per row and the rest alike; z reads
/// y's first value against the others and w reads z, observed. Asserts that y's
/// copies link to `y_link` and z's to the clauses of z's table, and that the
/// answers are the network's.
fn many_rules(values: usize, y_link: &str) {
    let mut source = String::new();
    source.push_str(
        &(0..48)
            .map(|i| format!("{}::x(v{i})", 1.0 / 48.0))
            .collect::<Vec<_>>()
            .join("; "),
    );
    source.push_str(".\n");
    source.push_str(
        &(0..48)
            .map(|j| format!("{}::q(u{j})", 1.0 / 48.0))
            .collect::<Vec<_>>()
            .join("; "),
    );
    source.push_str(".\n");
    let mut y = 0.0;
    for i in 0..48 {
        for j in 0..48 {
            let p = ((3 * i + 5 * j + i * j) % 7 + 1) as f64 / 8.0;
            y += p / (48.0 * 48.0);
            let rest = (1..values)
                .map(|k| format!("{}::y(k{k})", (1.0 - p) / (values - 1) as f64))
                .collect::<Vec<_>>()
                .join("; ");
            source.push_str(&format!("{p}::y(k0); {rest} :- x(v{i}), q(u{j}).\n"));
        }
    }
    source.push_str("0.3::z(t); 0.7::z(f) :- y(k0).\n");
    for k in 1..values {
        source.push_str(&format!("0.6::z(t); 0.4::z(f) :- y(k{k}).\n"));
    }
    source.push_str("0.2::w(t); 0.8::w(f) :- z(t).\n0.5::w(t); 0.5::w(f) :- z(f).\n");
    // Evidence on z would fix z, and w would read its value
    // (src/ground/condition.rs).
    source.push_str("evidence(w(t)). query(y(k0)). query(z(t)).\n");
    // Row classes would read y's seven distributions through a class atom.
    let clauses = common::ground(&source, Rows::Apart, true);
    let link_of = |value: &str| -> Vec<String> {
        clauses
            .encoding
            .indicators
            .iter()
            .filter(|i| clauses.atoms[i.atom as usize].to_string() == value)
            .map(|i| clauses.atoms[i.link as usize].name.to_string())
            .collect()
    };
    for value in (0..values).map(|k| format!("y(k{k})")) {
        let links = link_of(&value);
        assert!(
            !links.is_empty() && links.iter().all(|l| l == y_link),
            "{value}: {links:?}"
        );
    }
    for value in ["z(t)", "z(f)"] {
        let links = link_of(value);
        assert!(
            !links.is_empty() && links.iter().all(|l| l == "$c"),
            "{value}: {links:?}"
        );
    }
    let z = 0.3 * y + 0.6 * (1.0 - y);
    let w = 0.2 * z + 0.5 * (1.0 - z);
    let expected = [y * (0.3 * 0.2 + 0.7 * 0.5) / w, 0.2 * z / w];
    // `closed` is left out: with y of two values held as rules or as
    // clauses it did not finish in two minutes (with links to the rules
    // alone its evidence was 8.8 M pairs).
    for strategy in [
        Strategy::Trial(Kind::Families { search: true }),
        Strategy::Trial(Kind::Families { search: false }),
    ] {
        let run = tddlog::solve_ground(
            clauses.clone(),
            &common::options(strategy, tddlog::Projection::default()),
        )
        .unwrap();
        assert_probabilities(
            &format!("{values} values, {strategy:?}"),
            &run.results.unwrap(),
            &expected,
            1e-12,
        );
    }
}

#[test]
fn a_table_of_many_rules_keeps_links_to_its_formulas() {
    // 27 648 rules, more than a table tied by clauses may have, and more
    // cells than one may have past that.
    many_rules(12, "$d");
}

#[test]
fn a_table_of_many_rules_of_few_values_is_tied_by_clauses() {
    // 4 608 rules, more than the limit of rules, but 9 216 cells.
    many_rules(2, "$c");
}

#[test]
fn the_readers_of_one_table_read_it_through_copies_of_their_own() {
    // y, z and w each read x's three values through copies of their own,
    // tied to x's rows by clauses; the answers must be those of the
    // network.
    let source = "0.3::p. 0.4::fq. 0.2::gq. 0.7::fr. 0.1::gr.
                  q :- fq. q :- gq. r :- fr. r :- gr.
                  0.2::x(a); 0.5::x(b); 0.3::x(c) :- p.
                  0.6::x(a); 0.1::x(b); 0.3::x(c) :- \\+p.
                  0.9::y(1); 0.1::y(2) :- x(a), q.  0.4::y(1); 0.6::y(2) :- x(b), q.
                  0.1::y(1); 0.9::y(2) :- x(c), q.  0.3::y(1); 0.7::y(2) :- \\+q.
                  0.2::z(1); 0.8::z(2) :- x(a), r.  0.7::z(1); 0.3::z(2) :- x(b), r.
                  0.5::z(1); 0.5::z(2) :- x(c), r.  0.6::z(1); 0.4::z(2) :- \\+r.
                  0.5::w(1); 0.5::w(2) :- x(a).  0.3::w(1); 0.7::w(2) :- x(b).
                  0.8::w(1); 0.2::w(2) :- x(c).
                  evidence(y(1), true).
                  query(z(1)). query(w(2)). query(x(a)). query(p).";
    let rewritten = common::ground(source, Rows::Classes, true);
    for value in ["x(a)", "x(b)", "x(c)"] {
        let links = rewritten
            .encoding
            .indicators
            .iter()
            .filter(|i| rewritten.atoms[i.atom as usize].to_string() == value)
            .count();
        assert_eq!(links, 3, "{value}: one copy per reader");
    }
    // x with p, and y(1), z(1) and w(2) given x.
    let (q, r) = (1.0 - 0.6 * 0.8, 1.0 - 0.3 * 0.9);
    let x = [
        [0.3 * 0.2, 0.3 * 0.5, 0.3 * 0.3],
        [0.7 * 0.6, 0.7 * 0.1, 0.7 * 0.3],
    ];
    let y = [0.9, 0.4, 0.1].map(|l| q * l + (1.0 - q) * 0.3);
    let z = [0.2, 0.7, 0.5].map(|m| r * m + (1.0 - r) * 0.6);
    let w = [0.5, 0.7, 0.2];
    let joint = |p: usize, v: usize| x[p][v] * y[v];
    let e: f64 = (0..2)
        .flat_map(|p| (0..3).map(move |v| (p, v)))
        .map(|(p, v)| joint(p, v))
        .sum();
    let given = |f: &dyn Fn(usize, usize) -> f64| {
        (0..2)
            .flat_map(|p| (0..3).map(move |v| (p, v)))
            .map(|(p, v)| joint(p, v) * f(p, v))
            .sum::<f64>()
            / e
    };
    let expected = [
        given(&|_, v| z[v]),
        given(&|_, v| w[v]),
        given(&|_, v| f64::from(u8::from(v == 0))),
        given(&|p, _| f64::from(u8::from(p == 0))),
    ];
    for strategy in [
        Strategy::Trial(Kind::Families { search: false }),
        Strategy::Trial(Kind::Families { search: true }),
    ] {
        let run = tddlog::solve_ground(
            rewritten.clone(),
            &common::options(strategy, tddlog::Projection::default()),
        )
        .unwrap();
        assert_probabilities(
            &format!("{strategy:?}"),
            &run.results.unwrap(),
            &expected,
            1e-12,
        );
    }
}

#[test]
fn tables_as_clauses_take_a_table_split_on_a_parent() {
    // y reads x's 64 values and q's 4, and is split on x: the rows for each
    // value of x read q through copies of their own. The product of the
    // inputs' states, 64·4^64, overflows the measure sum, so the rows are
    // checked part by part, and y is tied to z by clauses.
    let q = [("a", 0.125), ("b", 0.25), ("c", 0.25), ("d", 0.375)];
    let mut source = String::from("0.3::z(t); 0.7::z(f) :- y(t).\n0.6::z(t); 0.4::z(f) :- y(f).\n");
    source.push_str(
        &(0..64)
            .map(|i| format!("0.015625::x(v{i})"))
            .collect::<Vec<_>>()
            .join("; "),
    );
    source.push_str(".\n0.125::q(a); 0.25::q(b); 0.25::q(c); 0.375::q(d).\n");
    // P(y(t) | q(j)), averaged over x's values.
    let mut mean = [0.0; 4];
    for i in 0..64 {
        for (j, (value, _)) in q.iter().enumerate() {
            let p = ((3 * i + 5 * j + i * j) % 7 + 1) as f64 / 8.0;
            mean[j] += p / 64.0;
            source.push_str(&format!(
                "{p}::y(t); {}::y(f) :- x(v{i}), q({value}).\n",
                1.0 - p
            ));
        }
    }
    source.push_str("evidence(z(t)). query(q(a)). query(y(t)).\n");
    let clauses = common::ground(&source, Rows::Classes, true);
    let atom = |name: &str| {
        clauses
            .atoms
            .iter()
            .position(|a| a.to_string() == name)
            .unwrap() as u32
    };
    let defined_by_copy = |k: u32| {
        let rules: Vec<_> = clauses.rules.iter().filter(|r| r.head == k).collect();
        rules.len() == 1
            && rules[0].body.len() == 1
            && clauses
                .encoding
                .indicators
                .iter()
                .any(|i| i.var == rules[0].body[0].atom())
    };
    assert!(
        defined_by_copy(atom("y(t)")) && defined_by_copy(atom("y(f)")),
        "y's clauses tie it to z"
    );
    let y: f64 = q.iter().zip(mean).map(|((_, p), m)| p * m).sum();
    let z = 0.3 * y + 0.6 * (1.0 - y);
    let expected = [
        0.125 * (0.3 * mean[0] + 0.6 * (1.0 - mean[0])) / z,
        0.3 * y / z,
    ];
    for strategy in [
        Strategy::Trial(Kind::Families { search: true }),
        Strategy::Trial(Kind::Closed),
    ] {
        let run = tddlog::solve_ground(
            clauses.clone(),
            &common::options(strategy, tddlog::Projection::default()),
        )
        .unwrap();
        assert_probabilities(
            &format!("{strategy:?}"),
            &run.results.unwrap(),
            &expected,
            1e-12,
        );
    }
}

/// A table's rows read the value of each parent. The link that states at
/// most one of a parent's copies holds takes the rows as clauses without
/// the other values' copies negated, which no other rule reads, and the
/// answers are ProbLog's: 2.3.0 prints x(a): 0.19238095, x(c): 0.52619048,
/// w(b): 0.24428571, y(p): 0.57785714, y(r): 0.27714286, which enumerating
/// the 27 worlds gives exactly as 101/525, 221/420, 171/700, 809/1400 and
/// 97/350.
#[test]
fn tables_as_clauses_read_a_parent_s_value_without_the_other_copies() {
    let source = "0.2::x(a); 0.3::x(b); 0.5::x(c).
                  0.6::w(a); 0.3::w(b); 0.1::w(c).
                  0.1::y(p); 0.3::y(q); 0.6::y(r) :- x(a), w(a).
                  0.5::y(p); 0.2::y(q); 0.3::y(r) :- x(a), w(b).
                  0.7::y(p); 0.2::y(q); 0.1::y(r) :- x(a), w(c).
                  0.2::y(p); 0.2::y(q); 0.6::y(r) :- x(b), w(a).
                  0.4::y(p); 0.4::y(q); 0.2::y(r) :- x(b), w(b).
                  0.3::y(p); 0.6::y(q); 0.1::y(r) :- x(b), w(c).
                  0.8::y(p); 0.1::y(q); 0.1::y(r) :- x(c), w(a).
                  0.1::y(p); 0.8::y(q); 0.1::y(r) :- x(c), w(b).
                  0.25::y(p); 0.25::y(q); 0.5::y(r) :- x(c), w(c).
                  0.9::z(t); 0.1::z(f) :- y(p).
                  0.3::z(t); 0.7::z(f) :- y(q).
                  0.6::z(t); 0.4::z(f) :- y(r).
                  evidence(z(t)).
                  query(x(a)). query(x(c)). query(w(b)). query(y(p)). query(y(r)).";
    let clauses = common::ground(source, Rows::Classes, true);
    assert!(
        clauses
            .evidence
            .iter()
            .any(|(a, _, value)| !value && a.to_string().starts_with("'$c'")),
        "y's clauses tie it to z"
    );
    let parent_copies: Vec<u32> = clauses
        .encoding
        .indicators
        .iter()
        .filter(|i| {
            ["x(", "w("]
                .iter()
                .any(|p| clauses.atoms[i.atom as usize].to_string().starts_with(p))
        })
        .map(|i| i.var)
        .collect();
    let rows: Vec<_> = clauses
        .rules
        .iter()
        .filter(|r| {
            clauses.atoms[r.head as usize]
                .to_string()
                .starts_with("'$c'")
                && r.body
                    .iter()
                    .any(|l| l.is_positive() && parent_copies.contains(&l.atom()))
        })
        .collect();
    assert!(!rows.is_empty(), "the links take y's rows");
    assert!(
        rows.iter().all(|r| r
            .body
            .iter()
            .all(|l| l.is_positive() || !parent_copies.contains(&l.atom()))),
        "a row reads no parent's copy negated"
    );
    let expected = [
        101.0 / 525.0,
        221.0 / 420.0,
        171.0 / 700.0,
        809.0 / 1400.0,
        97.0 / 350.0,
    ];
    for strategy in [
        Strategy::Trial(Kind::Families { search: true }),
        Strategy::Trial(Kind::Closed),
    ] {
        let run = tddlog::solve_ground(
            clauses.clone(),
            &common::options(strategy, tddlog::Projection::default()),
        )
        .unwrap();
        assert_probabilities(
            &format!("{strategy:?}"),
            &run.results.unwrap(),
            &expected,
            1e-12,
        );
    }
}
