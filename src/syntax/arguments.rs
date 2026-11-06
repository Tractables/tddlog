//! A program's arguments ([`bind`]): the values that follow the program's
//! file on the command line, `tddlog walk.pl 100 0.3`, take the places
//! `$1`, `$2`, ... of its text, as a shell script's arguments do, so that
//! one program serves every size, probability, query and observation it
//! is run with.

use std::borrow::Cow;

use super::{ParseError, SYMBOL_CHARS, lexer, parser};
use crate::Error;

/// `text`, a ProbLog program or an LPAD, with each place `$N` (`$` and
/// digits) outside its comments and quoted atoms replaced by the term
/// `arguments[N - 1]`: as it is written, or in parentheses where an
/// operator is at its top (`1/3` as `(1/3)`), and apart from a neighbour
/// it would otherwise run into (`5-$1` of `-3` as `5- -3`). Without
/// arguments, `text` itself, in which a place is a parse error.
///
/// The errors: an argument that is not a term, an argument that no place
/// takes, and a place without an argument.
pub(crate) fn bind<'a>(text: &'a str, arguments: &[String]) -> Result<Cow<'a, str>, Error> {
    if arguments.is_empty() {
        return Ok(Cow::Borrowed(text));
    }
    let terms = arguments
        .iter()
        .enumerate()
        .map(|(k, argument)| {
            let argument = argument.trim();
            match parser::argument(argument) {
                Ok(true) => Ok(Cow::Borrowed(argument)),
                Ok(false) => Ok(Cow::Owned(format!("({argument})"))),
                Err(e) => Err(Error::Options(format!(
                    "argument {}, `{argument}`, is not a term: {}",
                    k + 1,
                    e.msg
                ))),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let places = match text.contains('$') {
        true => lexer::places(text).map_err(|e| Error::Parse(e.into()))?,
        false => Vec::new(),
    };
    let mut taken = vec![false; terms.len()];
    let mut out = String::with_capacity(text.len());
    let mut next = 0;
    for place in places {
        let name = &text[place.at.clone()];
        let k = name[1..]
            .parse::<usize>()
            .ok()
            .filter(|k| (1..=terms.len()).contains(k))
            .ok_or_else(|| {
                Error::Parse(ParseError {
                    line: place.line,
                    msg: format!(
                        "`{name}` has no argument: the program is given {}",
                        terms.len()
                    ),
                })
            })?;
        taken[k - 1] = true;
        let term = &terms[k - 1];
        out.push_str(&text[next..place.at.start]);
        if joins(out.as_bytes().last(), term.as_bytes().first()) {
            out.push(' ');
        }
        out.push_str(term);
        next = place.at.end;
        if joins(term.as_bytes().last(), text.as_bytes().get(next)) {
            out.push(' ');
        }
    }
    if let Some(k) = taken.iter().position(|&t| !t) {
        return Err(Error::Options(format!(
            "argument {}, `{}`, is given, and the program has no `${}`",
            k + 1,
            arguments[k].trim(),
            k + 1
        )));
    }
    out.push_str(&text[next..]);
    Ok(Cow::Owned(out))
}

/// Whether the characters that end and start with the bytes `a` and `b`,
/// side by side, would be read as one token: two characters of names and
/// numbers, or two symbol characters, as Prolog reads `-` and `-3` as
/// `--` and `3`.
fn joins(a: Option<&u8>, b: Option<&u8>) -> bool {
    let word = |c: u8| c.is_ascii_alphanumeric() || c == b'_' || c >= 0x80;
    let symbol = |c: u8| SYMBOL_CHARS.as_bytes().contains(&c);
    match (a, b) {
        (Some(&a), Some(&b)) => (word(a) && word(b)) || (symbol(a) && symbol(b)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bound(text: &str, arguments: &[&str]) -> Result<String, Error> {
        let arguments: Vec<String> = arguments.iter().map(|a| a.to_string()).collect();
        bind(text, &arguments).map(Cow::into_owned)
    }

    #[test]
    fn each_place_takes_its_argument_as_a_term() {
        assert_eq!(
            bound(
                "$2::e(I) :- between(1, $1, I).\nquery(p($3)). % $9\nq('$1', \"$2\").\n",
                &["10", "0.3", " [a, f(b)] "]
            )
            .unwrap(),
            "0.3::e(I) :- between(1, 10, I).\nquery(p([a, f(b)])). % $9\nq('$1', \"$2\").\n"
        );
        // An operator at the top in parentheses, and a sign or a name kept
        // apart from its neighbour.
        assert_eq!(
            bound("X is 5-$1, Y is $2*2, Z=$1, a($3$3).", &["-3", "1+2", "b"]).unwrap(),
            "X is 5- -3, Y is (1+2)*2, Z= -3, a(b b)."
        );
        assert_eq!(bound("a($1).", &["X"]).unwrap(), "a(X).");
    }

    #[test]
    fn a_program_without_arguments_is_its_own_text() {
        let text = "a($1).";
        assert!(matches!(bind(text, &[]), Ok(Cow::Borrowed(t)) if t == text));
        let e = crate::syntax::parse_program(text).unwrap_err();
        assert_eq!(e.to_string(), "line 1: `$1` has no argument");
    }

    #[test]
    fn a_place_without_an_argument_an_argument_without_a_place_and_one_not_a_term_fail() {
        let error =
            |text: &str, arguments: &[&str]| bound(text, arguments).unwrap_err().to_string();
        assert_eq!(
            error("a.\nb($1, $3).", &["1", "2"]),
            "parse error: line 2: `$3` has no argument: the program is given 2"
        );
        assert_eq!(
            error("a($0).", &["1"]),
            "parse error: line 1: `$0` has no argument: the program is given 1"
        );
        assert_eq!(
            error("a($1). % $2", &["1", "2"]),
            "options error: argument 2, `2`, is given, and the program has no `$2`"
        );
        assert_eq!(
            error("a.", &["1"]),
            "options error: argument 1, `1`, is given, and the program has no `$1`"
        );
        for (argument, why) in [
            ("f(", "unexpected end of input"),
            ("a b", "unexpected name `b`"),
            ("", "unexpected end of input"),
            ("a.", "unexpected `.`"),
            ("$2", "`$2` has no argument"),
        ] {
            assert_eq!(
                error("a($1).", &[argument]),
                format!("options error: argument 1, `{argument}`, is not a term: {why}")
            );
        }
    }
}
