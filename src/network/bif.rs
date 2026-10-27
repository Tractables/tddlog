//! The Bayesian Interchange Format: `variable X { type discrete [ 2 ] {
//! a, b }; }` and `probability ( X | P1, P2 ) { (u1, u2) 0.1, 0.9; ... }`,
//! a root's row written `table 0.3, 0.7;`. A comment runs from `//` to the
//! end of the line. Read as `gen_bn.py`'s regular expressions read it; a
//! row of the same parents' values twice keeps the first's place and
//! the second's numbers.

use std::collections::HashMap;

use super::scan::{names_before_brace, occurrences, probability, skip_space, space, strip};
use super::{Network, Table};

pub(super) fn parse(text: &str) -> Result<Network, String> {
    let text = without_comments(text);
    let mut network = Network::default();
    let mut pos = 0;
    while let Some(at) = occurrences(&text, pos, "variable", false).next() {
        match variable(&text, at + "variable".len()) {
            Some((name, size, values, end)) => {
                let values: Vec<String> = values.split(',').map(|v| strip(v).to_string()).collect();
                if size.parse() != Ok(values.len()) {
                    return Err(format!(
                        "{name}: {size} values declared, {} listed",
                        values.len()
                    ));
                }
                network.declare(name, values)?;
                pos = end;
            }
            None => pos = at + 1,
        }
    }
    let mut pos = 0;
    while let Some(at) = occurrences(&text, pos, "probability", false).next() {
        match block(&text, at + "probability".len()) {
            Some((head, body, end)) => {
                read_probability(&mut network, head, body)?;
                pos = end;
            }
            None => pos = at + 1,
        }
    }
    Ok(network)
}

/// `text` without its comments, from `//` to the end of the line.
fn without_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(k) = rest.find("//") {
        out.push_str(&rest[..k]);
        rest = &rest[k..];
        rest = &rest[rest.find('\n').unwrap_or(rest.len())..];
    }
    out.push_str(rest);
    out
}

/// The name, the declared number of values, the values' list and the end
/// of a match of `variable\s+(\S+)\s*\{\s*type\s+discrete\s*\[\s*(\d+)\s*\]
/// \s*\{([^}]*)\}\s*;\s*\}` from `i`, just after the word.
fn variable(text: &str, i: usize) -> Option<(&str, &str, &str, usize)> {
    let start = skip_space(text, i);
    if start == i {
        return None;
    }
    names_before_brace(text, start)
        .into_iter()
        .find_map(|(name_end, brace)| {
            let (size, values, end) = declaration(text, brace + 1)?;
            Some((&text[start..name_end], size, values, end))
        })
}

/// `\s*type\s+discrete\s*\[\s*(\d+)\s*\]\s*\{([^}]*)\}\s*;\s*\}` at `i`.
fn declaration(text: &str, i: usize) -> Option<(&str, &str, usize)> {
    let i = skip_space(text, i);
    let i = i + text[i..].strip_prefix("type").map(|_| 4)?;
    let j = skip_space(text, i);
    if j == i {
        return None;
    }
    let j = j + text[j..].strip_prefix("discrete").map(|_| 8)?;
    let j = skip_space(text, j);
    let j = skip_space(text, j + text[j..].strip_prefix('[').map(|_| 1)?);
    let digits = text[j..]
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len() - j);
    if digits == 0 {
        return None;
    }
    let size = &text[j..j + digits];
    let k = skip_space(text, j + digits);
    let k = skip_space(text, k + text[k..].strip_prefix(']').map(|_| 1)?);
    let k = k + text[k..].strip_prefix('{').map(|_| 1)?;
    let close = k + text[k..].find('}')?;
    let m = skip_space(text, close + 1);
    let m = skip_space(text, m + text[m..].strip_prefix(';').map(|_| 1)?);
    let end = m + text[m..].strip_prefix('}').map(|_| 1)?;
    Some((size, &text[k..close], end))
}

/// The head and body of a match of `\s*\(([^)]*)\)\s*\{([^}]*)\}` at `i`,
/// and where it ends.
fn block(text: &str, i: usize) -> Option<(&str, &str, usize)> {
    let i = skip_space(text, i);
    let rest = text[i..].strip_prefix('(')?;
    let close = rest.find(')')?;
    let j = skip_space(text, i + 1 + close + 1);
    let k = j + text[j..].strip_prefix('{').map(|_| 1)?;
    let end = k + text[k..].find('}')?;
    Some((&rest[..close], &text[k..end], end + 1))
}

/// Sets a variable's table from a `probability` block: its head, `X |
/// P1, P2`, and its rows.
fn read_probability(network: &mut Network, head: &str, body: &str) -> Result<(), String> {
    let mut parts = head.split('|').map(strip);
    let name = parts.next().unwrap_or("");
    let v = network.variable(name)?;
    let parents = match parts.next() {
        Some(p) if !p.is_empty() => p
            .split(',')
            .map(|p| network.variable(strip(p)))
            .collect::<Result<Vec<_>, _>>()?,
        _ => Vec::new(),
    };
    let k = network.values[v].len();
    let mut table = Table {
        parents,
        keys: Vec::new(),
        probs: Vec::new(),
    };
    let mut place: HashMap<Vec<u32>, usize> = HashMap::new();
    for line in strip(body).split(';').map(strip).filter(|l| !l.is_empty()) {
        let (key, numbers) = if let Some(numbers) = line.strip_prefix("table") {
            if !table.parents.is_empty() {
                return Err(format!("{name}: a `table` row for a variable with parents"));
            }
            (Vec::new(), numbers)
        } else {
            let row = line
                .strip_prefix('(')
                .ok_or_else(|| format!("{name}: a row that is not `(values) numbers`: {line}"))?;
            let close = row
                .find(')')
                .ok_or_else(|| format!("{name}: a row without its `)`: {line}"))?;
            let values: Vec<&str> = row[..close].split(',').map(strip).collect();
            if values.len() != table.parents.len() {
                return Err(format!(
                    "{name}: a row of {} parents' values: {line}",
                    values.len()
                ));
            }
            let key = values
                .iter()
                .zip(&table.parents)
                .map(|(u, &p)| network.value(p, u).map(|u| u as u32))
                .collect::<Result<Vec<_>, _>>()?;
            (key, &row[close + 1..])
        };
        let probs = numbers
            .split(|c: char| c == ',' || space(c))
            .filter(|f| !f.is_empty())
            .map(probability)
            .collect::<Result<Vec<_>, _>>()?;
        if probs.len() != k {
            return Err(format!(
                "{name}: a row of {} numbers for {k} values: {line}",
                probs.len()
            ));
        }
        match place.get(&key) {
            Some(&r) => table.probs[r * k..(r + 1) * k].copy_from_slice(&probs),
            None => {
                place.insert(key.clone(), place.len());
                table.keys.extend(key);
                table.probs.extend(probs);
            }
        }
    }
    network.tables[v] = Some(table);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_network_reads_as_its_variables_and_rows() {
        let text = "network unknown {\n}\n\
            variable Smoker { // a comment\n  type discrete [ 2 ] { yes, no };\n}\n\
            variable Cancer {\n  type discrete [ 2 ] { true, false };\n}\n\
            probability ( Smoker ) {\n  table 0.3, 0.7;\n}\n\
            probability ( Cancer | Smoker ) {\n  (no) 0.01, 0.99;\n  (yes) 0.1, 0.9;\n  (no) 0.02, 0.98;\n}\n";
        let network = parse(text).unwrap();
        assert_eq!(network.names, ["Smoker", "Cancer"]);
        assert_eq!(network.values[0], ["yes", "no"]);
        let cancer = network.tables[1].as_ref().unwrap();
        assert_eq!(cancer.keys, [1, 0]);
        assert_eq!(cancer.probs, [0.02, 0.98, 0.1, 0.9]);
        assert!(parse("variable A { type discrete [ 3 ] { a, b }; }").is_err());
    }
}
