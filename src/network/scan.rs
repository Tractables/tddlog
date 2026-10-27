//! The text conventions the Hugin and BIF readers share with the regular
//! expressions of `gen_bn.py`, which read the same files: white
//! space and word characters as Python's `re` and `str.split` take them.

/// White space as Python's `\s` and `str.split()` take it: Unicode's, and
/// the four ASCII separators U+001C to U+001F.
pub(super) fn space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// A character of a word, for `\b`: a letter, a digit or `_`.
pub(super) fn word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The fields of `s` between white space, as Python's `s.split()`.
pub(super) fn fields(s: &str) -> impl Iterator<Item = &str> {
    s.split(space).filter(|f| !f.is_empty())
}

/// `s` without white space at either end, as Python's `s.strip()`.
pub(super) fn strip(s: &str) -> &str {
    s.trim_matches(space)
}

/// The index of the first character at or after `i` that is not white
/// space.
pub(super) fn skip_space(s: &str, i: usize) -> usize {
    s[i..].find(|c| !space(c)).map_or(s.len(), |k| i + k)
}

/// Where each occurrence of `word` in `s` from `from` on starts that a
/// regular expression starting `\bword` matches at: not preceded by a word
/// character when `bounded`.
pub(super) fn occurrences<'a>(
    s: &'a str,
    from: usize,
    word_: &'a str,
    bounded: bool,
) -> impl Iterator<Item = usize> + 'a {
    s[from..]
        .match_indices(word_)
        .map(move |(i, _)| from + i)
        .filter(move |&i| !bounded || !s[..i].chars().next_back().is_some_and(word))
}

/// The ways `(\S+)\s*\{` matches at `i` in `s`, in the order a
/// backtracking match tries them: the end of the name and the place of the
/// brace, first the whole run of non-space characters followed by a brace
/// after optional white space, then each shorter run that a brace inside
/// it ends, longest first.
pub(super) fn names_before_brace(s: &str, i: usize) -> Vec<(usize, usize)> {
    let run_end = s[i..].find(space).map_or(s.len(), |k| i + k);
    if run_end == i {
        return Vec::new();
    }
    let mut ways = Vec::new();
    let after = skip_space(s, run_end);
    if s[after..].starts_with('{') {
        ways.push((run_end, after));
    }
    for (k, _) in s[i..run_end].rmatch_indices('{') {
        if k > 0 {
            ways.push((i + k, i + k));
        }
    }
    ways
}

/// The numbers of a Hugin table's data: each match of gen_bn.py's
/// `[-+]?(?:[0-9]+\.?[0-9]*|\.[0-9]+)(?:[eE][-+]?[0-9]+)?`, read as a
/// double.
pub(super) fn numbers(s: &str) -> Result<Vec<f64>, String> {
    let b = s.as_bytes();
    let digits = |mut i: usize| {
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        i
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let start = i;
        let j = if b[i] == b'-' || b[i] == b'+' {
            i + 1
        } else {
            i
        };
        let mut end = None;
        if j < b.len() && b[j].is_ascii_digit() {
            let mut k = digits(j);
            if k < b.len() && b[k] == b'.' {
                k = digits(k + 1);
            }
            end = Some(k);
        } else if j + 1 < b.len() && b[j] == b'.' && b[j + 1].is_ascii_digit() {
            end = Some(digits(j + 1));
        }
        let Some(mut k) = end else {
            i += 1;
            continue;
        };
        if k < b.len() && (b[k] == b'e' || b[k] == b'E') {
            let e = if k + 1 < b.len() && (b[k + 1] == b'-' || b[k + 1] == b'+') {
                k + 2
            } else {
                k + 1
            };
            if e < b.len() && b[e].is_ascii_digit() {
                k = digits(e);
            }
        }
        out.push(probability(&s[start..k])?);
        i = k;
    }
    Ok(out)
}

/// A number of a table, as Python's `float` reads it.
pub(super) fn probability(s: &str) -> Result<f64, String> {
    s.parse().map_err(|_| format!("`{s}` is not a number"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_read_as_gen_bn_py_finds_them() {
        assert_eq!(
            numbers("((0.5 .5\n1e-3), -2.5E+1 7. 1e x+3 1.2.3)").unwrap(),
            [0.5, 0.5, 1e-3, -25.0, 7.0, 1.0, 3.0, 1.2, 0.3]
        );
    }

    #[test]
    fn a_name_before_a_brace_is_the_longest_that_matches() {
        let s = "node a{b {";
        assert_eq!(names_before_brace(s, 5), [(8, 9), (6, 6)]);
        let s = "node a{b{c";
        assert_eq!(names_before_brace(s, 5), [(8, 8), (6, 6)]);
        assert_eq!(
            occurrences("node xnode _node\tnode", 0, "node", true).collect::<Vec<_>>(),
            [0, 17]
        );
    }
}
