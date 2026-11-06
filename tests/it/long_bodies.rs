//! Rules with bodies of tens of thousands of literals, as a bit-blasted
//! program's test of a wide number's value has, one literal per bit. The
//! grounder walks such a body without a stack frame for each literal.
//! Answers worked out by hand.
use crate::common;

use tddlog::Options;

/// `q` reads a chain of 20000 facts through as many variables, each
/// literal binding the next; `r` reads the negations of 20000 facts. Each
/// holds where every one of its 20000 independent literals does, with
/// probability 0.9999^20000. A walk with a stack frame for each literal
/// overflowed a stack of 2 MB, a test thread's, from 5000 literals on in a
/// release build. Every layout reads each body as a chain of links
/// (`vtree::chained`): one from the interaction graph would otherwise
/// expand it into a clique of 20000 atoms, 2 * 10^8 edges.
#[test]
fn a_body_of_20000_literals() {
    const N: usize = 20000;
    let mut source = String::new();
    for k in 0..N {
        source += &format!("0.9999::b({k},{}).\n0.0001::c({k}).\n", k + 1);
    }
    let chain: Vec<String> = (0..N)
        .map(|k| match k {
            0 => "b(0,X1)".to_string(),
            _ => format!("b(X{k},X{})", k + 1),
        })
        .collect();
    let negations: Vec<String> = (0..N).map(|k| format!("\\+c({k})")).collect();
    source += &format!("q :- {}.\n", chain.join(", "));
    source += &format!("r :- {}.\n", negations.join(", "));
    source += "query(q).\nquery(r).\n";
    let options = Options {
        strategy: "program".parse().unwrap(),
        ..Options::default()
    };
    let run = common::run(&source, &options).unwrap();
    let results = run.results.expect("no evidence");
    let expected = 0.9999f64.powi(N as i32);
    let got: Vec<(String, f64)> = results
        .iter()
        .map(|r| (r.to_string(), r.probability))
        .collect();
    assert_eq!(
        got.iter().map(|(a, _)| a.as_str()).collect::<Vec<_>>(),
        ["q", "r"]
    );
    for (a, p) in &got {
        assert!(
            (p - expected).abs() < 1e-9 * expected,
            "{a}: {p} against {expected}"
        );
    }
}

/// Three conjunctions of the same 30 derived atoms, each with a fact of its
/// own last, and their disjunction, under every layout. The chains of the
/// three bodies share their links (`vtree::chained`). Each `d(k)` holds
/// with probability 0.9 * 0.8, so `q(i)` with 0.72^30 * 0.5 and `any` with
/// 0.72^30 * (1 - 0.5^3).
#[test]
fn bodies_that_start_alike() {
    let mut source = String::new();
    for k in 1..=30 {
        source += &format!("0.9::f({k}). 0.8::e({k}). d({k}) :- f({k}), e({k}).\n");
    }
    let reads: Vec<String> = (1..=30).map(|k| format!("d({k})")).collect();
    for i in 1..=3 {
        source += &format!(
            "0.5::g({i}).\nq({i}) :- {}, g({i}).\nany :- q({i}).\n",
            reads.join(", ")
        );
    }
    source += "query(q(1)).\nquery(any).\n";
    let all = 0.72f64.powi(30);
    let expected = [("q(1)", all * 0.5), ("any", all * (1.0 - 0.125))];
    for layout in [
        "program",
        "program:fanin",
        "families:0",
        "closed",
        "td",
        "auto",
    ] {
        let options = Options {
            strategy: layout.parse().unwrap(),
            ..Options::default()
        };
        let run = common::run(&source, &options).unwrap();
        let results = run.results.expect("no evidence");
        let got: Vec<(String, f64)> = results
            .iter()
            .map(|r| (r.to_string(), r.probability))
            .collect();
        assert_eq!(got.len(), 2, "{layout}: {got:?}");
        for ((a, p), (b, q)) in got.iter().zip(expected) {
            assert_eq!(a, b, "{layout}");
            assert!((p - q).abs() < 1e-12 * q, "{layout}: {a}: {p} against {q}");
        }
    }
}
