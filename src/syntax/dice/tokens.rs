//! The tokens of a Dice program ([`tokenize`]): identifiers, numbers,
//! operators and places `$N`, each with its line, the comments skipped.
//! The program's arguments take the places they fill ([`super::fill`]),
//! and the parser ([`super::syntax`]) reads the tokens, where a place
//! left without an argument stands only for an `iterate`'s count.

use super::Res;
use crate::syntax::lpad::space;
use crate::syntax::{ParseError, fail};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Tk {
    Id,
    Num,
    Op,
    /// A place `$N` that no argument filled: the count of an `iterate`.
    Place,
    Eof,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Token<'a> {
    pub(super) kind: Tk,
    pub(super) text: &'a str,
    pub(super) line: usize,
}

/// The operators of more than one character, longest first where one is
/// a prefix of another; then the single characters.
const LONG_OPS: [&str; 11] = [
    "<=>", "==", "!=", "<=", ">=", "<<", ">>", "&&", "||", "::", "->",
];
const SHORT_OPS: &str = "-+*/%<>^(){}[],=!:";

/// The tokens of `text`: identifiers `[A-Za-z_][A-Za-z0-9_']*`, numbers,
/// operators, places `$N`; `//` comments to the end of the line and
/// `(* *)` comments (not nested) are skipped.
pub(super) fn tokenize(text: &str) -> Res<Vec<Token<'_>>> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let (mut pos, mut line) = (0, 1);
    loop {
        while let Some(c) = text[pos..].chars().next().filter(|&c| space(c)) {
            line += usize::from(c == '\n');
            pos += c.len_utf8();
        }
        if pos == b.len() {
            break;
        }
        let rest = &text[pos..];
        if rest.starts_with("//") {
            pos += rest.find('\n').unwrap_or(rest.len());
            continue;
        }
        if rest.starts_with("(*")
            && let Some(end) = rest[2..].find("*)")
        {
            line += rest[..end + 4].matches('\n').count();
            pos += end + 4;
            continue;
        }
        let digits = |from: usize| b[from..].iter().take_while(|c| c.is_ascii_digit()).count();
        let len = if b[pos].is_ascii_alphabetic() || b[pos] == b'_' {
            1 + b[pos + 1..]
                .iter()
                .take_while(|&&c| c.is_ascii_alphanumeric() || c == b'_' || c == b'\'')
                .count()
        } else if b[pos].is_ascii_digit() || b[pos] == b'.' {
            // `[0-9]*\.[0-9]+` or else `[0-9]+\.?[0-9]*`, then an exponent.
            let lead = digits(pos);
            let mut i = pos + lead;
            if b.get(i) == Some(&b'.') && digits(i + 1) > 0 {
                i += 1 + digits(i + 1);
            } else if lead > 0 {
                if b.get(i) == Some(&b'.') {
                    i += 1 + digits(i + 1);
                }
            } else {
                return Err(cannot(text, pos, line));
            }
            if matches!(b.get(i), Some(b'e' | b'E')) {
                let sign = usize::from(matches!(b.get(i + 1), Some(b'+' | b'-')));
                let d = digits(i + 1 + sign);
                if d > 0 {
                    i += 1 + sign + d;
                }
            }
            i - pos
        } else if b[pos] == b'$' && digits(pos + 1) > 0 {
            1 + digits(pos + 1)
        } else if let Some(op) = LONG_OPS.iter().find(|op| rest.starts_with(**op)) {
            op.len()
        } else if SHORT_OPS.as_bytes().contains(&b[pos]) {
            1
        } else {
            return Err(cannot(text, pos, line));
        };
        let kind = if b[pos].is_ascii_alphabetic() || b[pos] == b'_' {
            Tk::Id
        } else if b[pos].is_ascii_digit() || b[pos] == b'.' {
            Tk::Num
        } else if b[pos] == b'$' {
            Tk::Place
        } else {
            Tk::Op
        };
        out.push(Token {
            kind,
            text: &text[pos..pos + len],
            line,
        });
        pos += len;
    }
    Ok(out)
}

fn cannot(text: &str, pos: usize, line: usize) -> ParseError {
    let at: String = text[pos..].chars().take(30).collect();
    fail(line, format!("cannot read a Dice token at `{at}`"))
}

/// A place without an argument where the program needs a number: a place
/// waits for its argument only as an `iterate`'s count.
pub(super) fn misplaced(t: Token<'_>) -> ParseError {
    fail(
        t.line,
        format!(
            "`{}` has no argument: a place other than an iterate's count needs one",
            t.text
        ),
    )
}
