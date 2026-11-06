//! Dice programs (Holtzen, Van den Broeck and Millstein, OOPSLA 2020),
//! read as the ProbLog program this module writes for them ([`read`]),
//! which `tddlog --program` prints and the parser then reads.
//!
//! The program is evaluated symbolically: every `let` binds a value that
//! is a Boolean literal, an integer, or a tuple of values. An integer of
//! type `int(w)` is `w` bits, as in Dice, each a literal. Every `flip`
//! becomes a fresh probabilistic fact; a `discrete` and a `uniform` become
//! flips bit by bit as Dice draws them (`Translator::choice`); Boolean
//! operators and conditionals become rules over fresh atoms. Integer
//! operations work on the bits: `+` and `-` an adder, `*` shifted
//! additions, `==` a conjunction, `<` a comparator, a conditional a choice
//! of bits. The values an integer can take are tracked where their pairs
//! are few, so that a comparison every pair decides is a constant. An
//! `observe` inside a conditional branch holds only when the branch is
//! taken, so it becomes an evidence atom that is true whenever the path
//! condition is false.
//!
//! A chain `if c0 then e0 else if c1 then e1 ... else en` is merged once:
//! branch i is guarded by one atom, that c_i holds and no earlier
//! condition did, and values equal in several branches are merged under
//! the disjunction of their guards. Within a branch, a variable the guard
//! compares with a constant takes only the values the guard allows (under
//! a disjunction, the values either side allows): a constant where it
//! allows one, and the bits on which the values agree are constants, so
//! that a loop over a guarded-command step carries the values its state
//! reaches rather than every value of its type. Operations on the same
//! operands are made once. Functions are inlined at each call.
//!
//! A loop `iterate(f, init, k)` is written as a ProbLog user writes one,
//! recursion over a step index that the grounder unrolls: the count a
//! fact `steps(k)`, f's body once, its atoms indexed by the step `T`
//! (`step(T)`, 0 to k - 1), each flip a probabilistic fact of its step,
//! and the state before step `T` the atoms `s(T)` of the state's Booleans
//! and bits, `s(0)` the initial state's and `s(T + 1)` the body's at step
//! `T`; the state after the loop is read at the count, `s(k)`; the
//! body's observations hold at every step, `failed` where one does not,
//! and the evidence is that it is false (`Translator::iterate`). An
//! iterate inside a loop's step is expanded as Dice expands every iterate
//! (its `lib/Passes.ml`): k calls of f in a row, within the step.
//!
//! A place `$1`, `$2`, ... stands for an integer of the program (a width,
//! a value, a bound, a shift or a count), which the program's arguments
//! give ([`read_with`]), so that one program serves every size. Without
//! arguments, a place may stand only for an iterate's count, which the
//! program written then holds in its fact `steps($1)` and where it reads
//! the state after the loop, `s($1)`, so that one ProbLog program serves
//! every count, its arguments filling its places as they fill any ProbLog
//! program's. A function that calls itself is
//! inlined up to Dice's recursion limit, 10: a call past it is
//! unreachable, an observation of false on its path, and returns its
//! declared type's default. The program's return value is queried: every
//! value of an integer (the conjunction of its bits), every component of a
//! tuple.
//!
//! A list is held as Dice holds one (its `lib/Passes.ml`): a pair of its
//! length, an integer of 4 bits, and exactly 10 elements as
//! nested pairs, those past the length its type's default value (false,
//! zero). `x :: xs` keeps the first 9 elements of `xs`, its length capped
//! at 10; `head` and `tail` of an empty list observe false on their path.
//! `binomial(w, n, p)` is the sum of `n` flips `p`, each added in a
//! conditional as Dice adds it, and `nth_bit(int(w, i), x)` is bit `i` of
//! `x` counted from the most significant.
//!
//! Supported: `let`, `if`-`then`-`else`, `flip`, `discrete`, `uniform`,
//! `binomial`, `int(w, k)`, `true`, `false`, `&&`, `||`, `!`, `^`, `<=>`,
//! `==`, `!=`, `<`, `<=`, `>`, `>=`, `+`, `-`, `*`, `<<` and `>>` by a
//! constant, `nth_bit`, tuples with `fst` and `snd`, lists with `[...]`,
//! `[] : list(t)`, `::`, `head`, `tail` and `length`, `observe`, `fun`
//! definitions and calls, `iterate`, recursion. Operator precedence is
//! Dice's (its `lib/Parser.mly`). Refused: `/`, `%`, `sample`, integers of
//! more than 63 bits, and a `uniform` over more than 2^24 values.
//!
//! The program is the one the Python script `dice2problog.py` (on the
//! baselines branch) wrote before this reader replaced it, byte for byte,
//! but for a loop's count, which the script wrote as the bounds of the
//! recursion and of the reads after it: its atoms are named in the same
//! order from the same hints, and its probabilities are written as
//! Python's `repr` writes a float.

mod syntax;
mod tokens;
mod translate;
mod values;

use super::{ParseError, fail};
use syntax::Parser;
use tokens::{Tk, Token, tokenize};
use translate::Translator;

/// The ProbLog program the Dice program `source` is read as. The error
/// names the line of the token or expression that the program cannot be
/// read at: a syntax error, an unbound variable, a type error, or a
/// construct the reader does not support.
pub fn read(source: &str) -> Result<String, ParseError> {
    translate(tokenize(source)?)
}

/// [`read`] of `source` with its places `$1`, `$2`, ... the integers
/// `arguments`, each digits, which take the places before the program is
/// read. Without arguments, it is [`read`]. The errors are [`read`]'s, an
/// argument that is not a natural number or that no place takes, and a
/// place without an argument.
pub fn read_with(source: &str, arguments: &[String]) -> Result<String, crate::Error> {
    let mut tokens = tokenize(source)?;
    fill(&mut tokens, arguments)?;
    Ok(translate(tokens)?)
}

fn translate(tokens: Vec<Token<'_>>) -> Result<String, ParseError> {
    let program = Parser { toks: tokens, i: 0 }.program()?;
    Translator::new(&program).translate(&program.body)
}

/// `tokens` with each place `$N` the number `arguments[N - 1]`, where any
/// are given.
fn fill<'a>(tokens: &mut [Token<'a>], arguments: &'a [String]) -> Result<(), crate::Error> {
    if arguments.is_empty() {
        return Ok(());
    }
    let arguments: Vec<&str> = arguments.iter().map(|a| a.trim()).collect();
    let natural = |a: &str| !a.is_empty() && a.bytes().all(|c| c.is_ascii_digit());
    if let Some((k, a)) = arguments.iter().enumerate().find(|(_, a)| !natural(a)) {
        return Err(crate::Error::Options(format!(
            "argument {}, `{a}`, is not a natural number, which is all a Dice program's places hold",
            k + 1
        )));
    }
    let mut taken = vec![false; arguments.len()];
    for t in tokens.iter_mut().filter(|t| t.kind == Tk::Place) {
        let k = t.text[1..]
            .parse::<usize>()
            .ok()
            .filter(|k| (1..=arguments.len()).contains(k))
            .ok_or_else(|| {
                fail(
                    t.line,
                    format!(
                        "`{}` has no argument: the program is given {}",
                        t.text,
                        arguments.len()
                    ),
                )
            })?;
        taken[k - 1] = true;
        *t = Token {
            kind: Tk::Num,
            text: arguments[k - 1],
            line: t.line,
        };
    }
    if let Some(k) = taken.iter().position(|&t| !t) {
        return Err(crate::Error::Options(format!(
            "argument {}, `{}`, is given, and the program has no `${}`",
            k + 1,
            arguments[k],
            k + 1
        )));
    }
    Ok(())
}

/// Dice's recursion limit (its `lib/Passes.ml`): a function that calls
/// itself is inlined this deep, and the call past it observes false.
const RECURSION_LIMIT: usize = 10;

/// The elements a list holds (Dice's `max_list_length`, its recursion
/// limit): a longer literal is an error, and `::` on a full list drops its
/// last element.
const LIST_LENGTH: usize = RECURSION_LIMIT;

/// The most pairs of values whose results an integer operation lists, to
/// know the values the result can take.
const SUPPORT_PAIRS: usize = 1 << 16;

/// The widest integer: its values are `u64`s.
const MAX_WIDTH: u64 = 63;

/// The widest `uniform` drawn bit by bit: its masses are listed.
const MAX_UNIFORM_WIDTH: u64 = 24;

/// The suffix of an atom of a loop's step.
const STEP: &str = "(T)";

type Res<T> = Result<T, ParseError>;

#[cfg(test)]
mod tests;
