//! A program that is already ground, as an outside grounder writes one:
//! facts, an annotated disjunction, negation and a cycle, every rule
//! without variables. The grounder takes each rule's atoms as lookups
//! (`src/ground/engine.rs`, `src/ground/schedule.rs`); the answers are
//! worked out by hand, and the same program written with variables gives
//! them too.
use crate::common;

use tddlog::Options;

/// The answers, by hand. `reach(a)` holds where `sa` does or `sb` and
/// `e(b,a)` do: 1 - 0.1 (1 - 0.8 * 0.5) = 0.94. `reach(b)` where `sb` does or
/// `sa` and `e(a,b)` do: 1 - 0.2 (1 - 0.9 * 0.6) = 0.908. `reach(c)` is
/// `reach(b)` and `e(b,c)`: 0.908 * 0.7 = 0.6356. `ok` is `reach(c)` without
/// the red colour, independent of it: 0.6356 * 0.7 = 0.44492; `unreached`
/// is not `reach(c)`: 0.3644. The disjunction's two heads exclude each
/// other, so `both` is impossible; `plain` reads the certain fact `link`
/// and not the blue colour: 0.5.
const EXPECTED: [(&str, f64); 8] = [
    ("reach(a)", 0.94),
    ("reach(b)", 0.908),
    ("reach(c)", 0.6356),
    ("ok", 0.44492),
    ("unreached", 0.3644),
    ("red", 0.3),
    ("both", 0.0),
    ("plain", 0.5),
];

const QUERIES: &str = "
    query(reach(a)). query(reach(b)). query(reach(c)). query(ok).
    query(unreached). query(red). query(both). query(plain).";

fn assert_answers(source: &str) {
    let run = common::run(source, &Options::default()).unwrap();
    common::assert_answers(
        source,
        &run.results.expect("consistent evidence"),
        &EXPECTED,
        1e-9,
    );
}

#[test]
fn a_ground_program_answers_as_written_with_variables() {
    let ground = "
        0.9::sa. 0.8::sb.
        0.6::e(a,b). 0.5::e(b,a). 0.7::e(b,c).
        0.3::col(red); 0.5::col(blue).
        link.
        reach(a) :- sa.
        reach(b) :- sb.
        reach(b) :- e(a,b), reach(a).
        reach(a) :- e(b,a), reach(b).
        reach(c) :- e(b,c), reach(b).
        ok :- reach(c), \\+ col(red).
        unreached :- \\+ reach(c).
        red :- col(red).
        both :- col(red), col(blue).
        plain :- link, \\+ col(blue).";
    assert_answers(&format!("{ground}{QUERIES}"));
    let written = "
        0.9::start(a). 0.8::start(b).
        0.6::e(a,b). 0.5::e(b,a). 0.7::e(b,c).
        0.3::col(red); 0.5::col(blue).
        link.
        reach(X) :- start(X).
        reach(Y) :- e(X,Y), reach(X).
        ok :- reach(c), \\+ col(red).
        unreached :- \\+ reach(c).
        red :- col(red).
        both :- col(red), col(blue).
        plain :- link, \\+ col(blue).";
    assert_answers(&format!("{written}{QUERIES}"));
}
