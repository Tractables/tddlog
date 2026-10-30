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
/// release build. The layout reads the program's order: one from the
/// interaction graph would expand each body into a clique of 20000 atoms,
/// 2 * 10^8 edges.
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
