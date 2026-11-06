use super::*;

use num_rational::BigRational;
use num_traits::ToPrimitive;

use super::syntax::{Expr, Kind};
use super::values::{bits_for, discrete_probs};
use crate::network::rows::fsum;

fn tree(text: &str) -> String {
    fn show(e: &Expr<'_>) -> String {
        match &e.kind {
            Kind::Bin(op, l, r) => format!("({} {op} {})", show(l), show(r)),
            Kind::Not(x) => format!("!{}", show(x)),
            Kind::Var(x) => x.to_string(),
            _ => "?".into(),
        }
    }
    let toks = tokenize(text).unwrap();
    show(&Parser { toks, i: 0 }.expr(0).unwrap())
}

/// `!` binds looser than `==` and `<`, tighter than `&&` (Dice's
/// `lib/Parser.mly`); `==` shares the level of `+` and `-`.
#[test]
fn precedence_is_dices() {
    assert_eq!(tree("!a == b && c < d + e"), "(!(a == b) && (c < (d + e)))");
    assert_eq!(tree("a || b && c"), "(a || (b && c))");
    assert_eq!(tree("a - b - c"), "((a - b) - c)");
    // `::` associates to the right, between the comparisons and `+`.
    assert_eq!(tree("a < b :: c :: d + e"), "(a < (b :: (c :: (d + e))))");
}

/// A list is its length, 4 bits, and ten elements as nested pairs;
/// `head` and `tail` of a list that may be empty observe that it is
/// not, and a list of known length needs no observation.
#[test]
fn a_list_is_its_length_and_ten_elements() {
    // The length, the constant 1, is queried at 0 and 1.
    let text = read("let xs = [flip 0.5, true] in tail xs").unwrap();
    let queries = text.lines().filter(|l| l.starts_with("query(")).count();
    assert_eq!(queries, 2 + 10, "{text}");
    assert!(!text.contains("evidence("), "{text}");
    let text = read("let xs = if flip 0.5 then [true] else [] : list(bool) in head xs").unwrap();
    assert_eq!(text.matches("evidence(").count(), 1, "{text}");
}

/// A chain of tens of thousands of `let`s, its flips bound first and
/// read last, as in Roulette's programs written in Dice: each variable
/// is found at once, and the bindings are dropped without a recursion
/// as deep as the chain, which overflowed a test's stack.
#[test]
fn a_long_chain_of_lets_is_read() {
    let n = 50_000;
    let mut text = String::new();
    for i in (1..=n).rev() {
        text += &format!("let a{i} = flip 0.5 in\n");
    }
    text += "let s0 = true in\n";
    for i in 1..=n {
        text += &format!("let s{i} = s{} && a{i} in\n", i - 1);
    }
    text += &format!("s{n}\n");
    let program = read(&text).unwrap();
    let flips = program.lines().filter(|l| l.starts_with("0.5::")).count();
    assert_eq!(flips, n);
    assert_eq!(
        program.lines().filter(|l| l.starts_with("query(")).count(),
        1
    );
}

#[test]
fn tokens_are_the_scripts() {
    let toks = tokenize("let x' = flip .5 in (* a\ncomment *) x' <=> 1.e2 // end").unwrap();
    let texts: Vec<&str> = toks.iter().map(|t| t.text).collect();
    assert_eq!(
        texts,
        ["let", "x'", "=", "flip", ".5", "in", "x'", "<=>", "1.e2"]
    );
    assert_eq!(toks[6].line, 2);
    assert!(tokenize("a # b").is_err());
    let toks = tokenize("iterate(f, x, $12)").unwrap();
    assert!(toks[6].kind == Tk::Place && toks[6].text == "$12");
}

/// A loop's count is a fact that the recursion over the steps reads,
/// and the state after them is read at it, so that a place for it,
/// `steps($1)` and `s($1)`, makes one program of every count.
#[test]
fn a_count_is_a_fact() {
    let program =
        read("fun f(x: bool) { if flip 0.5 then !x else x }\niterate(f, true, $1)").unwrap();
    for line in [
        "steps($1).",
        "step(0) :- steps(N), N > 0.",
        "step(T1) :- step(T), steps(N), T1 is T + 1, T1 < N.",
    ] {
        assert!(program.lines().any(|l| l == line), "{line}: {program}");
    }
    // The result, the state after the loop, read at the count.
    assert!(
        program
            .lines()
            .any(|l| l.starts_with("result") && l.ends_with(" :- f($1).")),
        "{program}"
    );
    assert_eq!(program.matches('$').count(), 2, "{program}");
}

/// With arguments, a place stands for any integer the program reads,
/// and the program is the one written with the integers in their
/// places.
#[test]
fn the_arguments_take_every_place() {
    let arguments = |a: &[&str]| a.iter().map(|a| a.to_string()).collect::<Vec<_>>();
    let placed = "fun f(x: int($1)) { x + uniform($1, 0, $2) }\n\
                  let y = iterate(f, int($1, $3), $4) in nth_bit(int($1, $5), y << $5)";
    let literal = "fun f(x: int(3)) { x + uniform(3, 0, 5) }\n\
                   let y = iterate(f, int(3, 2), 2) in nth_bit(int(3, 1), y << 1)";
    assert_eq!(
        read_with(placed, &arguments(&["3", "5", "2", " 2", "1"])).unwrap(),
        read(literal).unwrap()
    );
    let error = |a: &[&str]| read_with(placed, &arguments(a)).unwrap_err().to_string();
    assert_eq!(
        error(&["3", "5", "2", "2"]),
        "parse error: line 2: `$5` has no argument: the program is given 4"
    );
    assert_eq!(
        error(&["3", "5", "2", "2", "1", "7"]),
        "options error: argument 6, `7`, is given, and the program has no `$6`"
    );
    assert_eq!(
        error(&["3", "0.5", "2", "2", "1"]),
        "options error: argument 2, `0.5`, is not a natural number, which is all a Dice \
         program's places hold"
    );
}

/// An iterate inside a loop's step is expanded, as Dice expands it:
/// one loop, the outer, whose step holds the inner calls in a row
/// (their answers: tests/it/examples.rs).
#[test]
fn an_iterate_inside_a_loop_is_expanded() {
    let program = read(
        "fun g(x: int(3)) { x + int(3, 1) }\n\
         fun f(x: int(3)) { if flip 0.5 then iterate(g, x, 2) else x }\n\
         iterate(f, int(3, 0), 3)",
    )
    .unwrap();
    let counts: Vec<&str> = program.lines().filter(|l| l.starts_with("steps")).collect();
    assert_eq!(counts, ["steps(3)."], "{program}");
}

/// Dice accepts a list within 1e-4 of one and normalizes it (each bit
/// a ratio of sums); noDice's networks print thirds as 0.333333.
#[test]
fn a_discrete_off_one_by_rounding_is_normalized() {
    for ps in [&[0.333333, 0.333333, 0.333333][..], &[0.5, 0.50005]] {
        let got = discrete_probs(ps);
        assert!((fsum(&got) - 1.0).abs() <= 1e-15);
        assert!((got[0] / got[got.len() - 1] - ps[0] / ps[ps.len() - 1]).abs() <= 1e-15);
    }
    assert_eq!(discrete_probs(&[0.25, 0.75]), [0.25, 0.75]);
    assert_eq!(discrete_probs(&[0.25, 0.7]), [0.25, 0.7]);
}

#[test]
fn bits_for_counts_dices_digits() {
    let got: Vec<u64> = [1, 2, 3, 4, 5, 8, 9].iter().map(|&n| bits_for(n)).collect();
    assert_eq!(got, [1, 1, 2, 2, 3, 3, 4]);
}

/// The probabilities of the flips of a program's facts.
fn facts(text: &str) -> Vec<f64> {
    let mut ps: Vec<f64> = text
        .lines()
        .filter_map(|l| l.split_once("::"))
        .map(|(p, _)| p.parse().unwrap())
        .collect();
    ps.sort_by(f64::total_cmp);
    ps
}

/// A discrete is drawn bit by bit from the most significant, as Dice
/// draws it: bit 1 with the mass of 2 and 3; bit 0 with that of 1
/// among 0 and 1, and of 3 among 2 and 3, each ratio of the masses as
/// written, exactly. A uniform over a power of two takes one flip a
/// bit. No atom stands for one value but the queries.
#[test]
fn an_integer_is_its_bits() {
    let text = read("let x = discrete(0.1, 0.2, 0.3, 0.4) in x").unwrap();
    let exact = |x: f64| BigRational::from_float(x).unwrap();
    let p: Vec<BigRational> = [0.1, 0.2, 0.3, 0.4].into_iter().map(exact).collect();
    let ratio = |a: BigRational, b: BigRational| (a / b).to_f64().unwrap();
    let mut want = vec![
        ratio(&p[2] + &p[3], &p[0] + &p[1] + &p[2] + &p[3]),
        ratio(p[1].clone(), &p[0] + &p[1]),
        ratio(p[3].clone(), &p[2] + &p[3]),
    ];
    want.sort_by(f64::total_cmp);
    assert_eq!(facts(&text), want, "{text}");
    let text =
        read("let x = uniform(3, 0, 8) in let y = uniform(3, 0, 8) in x + y == int(3, 5)").unwrap();
    assert_eq!(facts(&text), [0.5; 6]);
    assert!(
        !text.lines().any(|l| {
            let head = l.split([' ', '.']).next().unwrap();
            head.ends_with(')') && !l.starts_with("query(")
        }),
        "{text}"
    );
}

/// Under `x == 1 || x == 3`, x takes the values 1 and 3 alone, whose
/// bit 0 is set: in that branch bit 0 is the constant true, so the
/// result's bit 0, true in the other branch too, is constant, and the
/// values with bit 0 clear have no rule.
#[test]
fn a_disjunctive_guard_restricts_its_branch() {
    let text = read(
        "let x = discrete(0.1, 0.2, 0.3, 0.4) in\n\
         if x == int(2, 1) || x == int(2, 3) then x else int(2, 1)",
    )
    .unwrap();
    let heads: Vec<&str> = text
        .lines()
        .filter(|l| !l.starts_with("query("))
        .map(|l| l.split(" :- ").next().unwrap().trim_end_matches('.'))
        .collect();
    let queries: Vec<&str> = text
        .lines()
        .filter_map(|l| l.strip_prefix("query(")?.strip_suffix(")."))
        .collect();
    let defined: Vec<bool> = queries.iter().map(|q| heads.contains(q)).collect();
    assert_eq!(defined, [false, true, false, true], "{text}");
}

#[test]
fn what_the_reader_does_not_support_is_refused() {
    for (program, error) in [
        ("int(3, 4) / int(3, 2)", "operator / is not supported"),
        ("int(3, 4) % int(3, 2)", "operator % is not supported"),
        ("sample flip 0.5", "`sample` is not supported"),
        ("[flip 0.5, int(2, 1)]", "a list of elements of types"),
        ("int(2, 1) :: [true]", "a list of elements of types"),
        (
            "[true, true, true, true, true, true, true, true, true, true, true]",
            "hold at most 10",
        ),
        ("[] : bool", "an empty list has a list type"),
        ("head (true, false)", "a list is expected"),
        ("nth_bit(int(3, 1), int(2, 1))", "x has 2 bits"),
        ("nth_bit(int(2, 2), int(2, 1))", "bit 2 of 2"),
        ("binomial(2, 4, 0.5)", "4 out of range for 4 values"),
        ("int(2, 1) + int(3, 1)", "integers of 4 and 8 values"),
        ("int(2, 4)", "out of range for 4 values"),
        ("int(64, 4)", "more than 63 bits"),
        ("let x = flip 0.5 in y", "unbound variable y"),
        ("f(flip 0.5)", "unknown function f"),
        (
            "fun f(a: bool) { a } f(true, false)",
            "1 parameters, 2 arguments",
        ),
        ("uniform(2, 3, 3)", "empty or out of range"),
        ("flip 0.5 + int(2, 1)", "an integer is expected"),
        ("flip $1", "`$1` has no argument: a place other than"),
        ("int(3, $1)", "`$1` has no argument: a place other than"),
        (
            "let x = $1 in x",
            "`$1` has no argument: a place other than",
        ),
        (
            "fun g(x: bool) { !x }\nfun f(x: bool) { iterate(g, x, $1) }\niterate(f, true, 2)",
            "an iterate inside a loop needs its count `$1` given as an argument",
        ),
        (
            "fun g(x: bool) { int(2, 1) }\nfun f(x: bool) { iterate(g, x, 2) }\niterate(f, true, 2)",
            "g: the loop step returns a value of another type than its state",
        ),
        ("let x = flip 0.5 in $x", "cannot read a Dice token at `$x"),
    ] {
        let e = read(program).expect_err(program);
        assert!(e.msg.contains(error), "{program}: {e}");
    }
}

#[test]
fn projections_take_the_longest_expression_as_dices_do() {
    // Dice's parser shifts after `fst`, `snd`, `head`, `tail` and
    // `length`: `fst s == x` is `fst (s == x)` and `snd s << 1` is
    // `snd (s << 1)`, which Dice refuses as type errors, as here.
    for (program, error) in [
        (
            "let s = (int(2, 1), int(2, 1)) in fst s == int(2, 1)",
            "== on a tuple",
        ),
        (
            "let s = (int(2, 1), int(2, 1)) in snd s << 1",
            "an integer is expected",
        ),
    ] {
        let e = read(program).expect_err(program);
        assert!(e.msg.contains(error), "{program}: {e}");
    }
    for program in [
        "let s = (int(2, 1), int(2, 1)) in (fst s) == int(2, 1)",
        "let s = (int(2, 1), int(2, 1)) in (snd s) << 1",
        "let s = (flip 0.5, int(2, 1)) in (fst s, snd s)",
        "let s = (int(2, 1), (true, int(2, 3))) in if fst (snd s) then snd (snd s) else fst s",
    ] {
        read(program).expect(program);
    }
}
