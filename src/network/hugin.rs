//! Hugin's `.net` format: `node X { states = ("a" "b"); }` and
//! `potential (X | P1 P2) { data = (((...))); }`, the data nested with the
//! first parent outermost and the values of X innermost. A comment runs
//! from `%` to the end of the line, outside quotes (munin's states include
//! "<12%"). A node without states, a decision or utility node of an
//! influence diagram, is not a variable. Read as `gen_bn.py`'s regular
//! expressions read it.

use super::Network;
use super::scan::{fields, names_before_brace, numbers, occurrences, skip_space, space, strip};

pub(super) fn parse(text: &str) -> Result<Network, String> {
    let text = without_comments(text);
    let mut network = Network::default();
    for (name, body) in blocks(&text, "node") {
        if let Some(values) = states(body) {
            network.declare(name, values)?;
        }
    }
    let mut pos = 0;
    while let Some(at) = occurrences(&text, pos, "potential", true).next() {
        match potential(&text, at + "potential".len()) {
            Some((head, body, end)) => {
                read_potential(&mut network, head, body)?;
                pos = end;
            }
            None => pos = at + 1,
        }
    }
    Ok(network)
}

/// `text` without its comments, from `%` to the end of the line, where
/// the `%` is not inside a quoted string: gen_bn.py's
/// `re.sub(r'("[^"]*")|%.*', ...)`.
fn without_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(k) = rest.find(['"', '%']) {
        out.push_str(&rest[..k]);
        rest = &rest[k..];
        if rest.starts_with('"') {
            match rest[1..].find('"') {
                Some(close) => {
                    out.push_str(&rest[..close + 2]);
                    rest = &rest[close + 2..];
                }
                None => {
                    out.push('"');
                    rest = &rest[1..];
                }
            }
        } else {
            rest = &rest[rest.find('\n').unwrap_or(rest.len())..];
        }
    }
    out.push_str(rest);
    out
}

/// The name and the body of each match of `\bword\s+(\S+)\s*\{(.*?)\}` in
/// `text`, in order, as Python's `re.finditer` finds them.
fn blocks<'a>(text: &'a str, word: &'a str) -> Vec<(&'a str, &'a str)> {
    let mut out = Vec::new();
    let mut pos = 0;
    while let Some(at) = occurrences(text, pos, word, true).next() {
        let after = at + word.len();
        let start = skip_space(text, after);
        let mut matched = None;
        if start > after {
            for (name_end, brace) in names_before_brace(text, start) {
                if let Some(close) = text[brace + 1..].find('}') {
                    let close = brace + 1 + close;
                    matched = Some((&text[start..name_end], &text[brace + 1..close], close + 1));
                    break;
                }
            }
        }
        match matched {
            Some((name, body, end)) => {
                out.push((name, body));
                pos = end;
            }
            None => pos = at + 1,
        }
    }
    out
}

/// The values of a node's body: the quoted strings of the first match of
/// `states\s*=\s*\(([^)]*)\)`, or `None` without one.
fn states(body: &str) -> Option<Vec<String>> {
    for (at, _) in body.match_indices("states") {
        let i = skip_space(body, at + "states".len());
        let Some(rest) = body[i..].strip_prefix('=') else {
            continue;
        };
        let j = skip_space(rest, 0);
        let Some(rest) = rest[j..].strip_prefix('(') else {
            continue;
        };
        let Some(close) = rest.find(')') else {
            continue;
        };
        let list = &rest[..close];
        let mut values = Vec::new();
        let mut quoted = list;
        while let Some(open) = quoted.find('"') {
            let Some(end) = quoted[open + 1..].find('"') else {
                break;
            };
            values.push(quoted[open + 1..open + 1 + end].to_string());
            quoted = &quoted[open + 1 + end + 1..];
        }
        return Some(values);
    }
    None
}

/// The head and body of a match of `potential\s*\(([^)]*)\)\s*\{(.*?)\}`
/// from `i`, just after the word, and where the match ends.
fn potential(text: &str, i: usize) -> Option<(&str, &str, usize)> {
    let i = skip_space(text, i);
    let rest = text[i..].strip_prefix('(')?;
    let close = rest.find(')')?;
    let head = &rest[..close];
    let j = skip_space(text, i + 1 + close + 1);
    let body_start = j + text[j..].strip_prefix('{').map(|_| 1)?;
    let end = body_start + text[body_start..].find('}')?;
    Some((head, &text[body_start..end], end + 1))
}

/// Sets a variable's table from a potential's head, `X | P1 P2`, and body.
fn read_potential(network: &mut Network, head: &str, body: &str) -> Result<(), String> {
    let mut parts = head.split('|');
    let name = strip(parts.next().unwrap_or(""));
    let v = network.variable(name)?;
    let parents = match parts.next() {
        Some(p) => fields(p)
            .map(|p| network.variable(p))
            .collect::<Result<Vec<_>, _>>()?,
        None => Vec::new(),
    };
    let data = data(body).ok_or_else(|| format!("{name}: a potential without data"))?;
    network.set_product_table(v, parents, numbers(data)?)
}

/// What the first match of `\bdata\s*=\s*(.*?);` in a potential's body
/// holds.
fn data(body: &str) -> Option<&str> {
    occurrences(body, 0, "data", true).find_map(|at| {
        let i = skip_space(body, at + "data".len());
        let rest = body[i..].strip_prefix('=')?;
        let rest = &rest[rest.find(|c| !space(c)).unwrap_or(rest.len())..];
        rest.find(';').map(|end| &rest[..end])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments_go_but_percent_signs_in_quotes_stay() {
        assert_eq!(
            without_comments("a \"<12%\" b % c\nd \"e"),
            "a \"<12%\" b \nd \"e"
        );
    }

    #[test]
    fn a_network_reads_as_its_nodes_and_potentials() {
        let text = "net { node_size = (80 40); }\n\
            node A { label = \"a\"; states = (\"yes\" \"no\"); }\n\
            decision D { states = (\"go\"); }\n\
            node B\n{\n  states = (\"lo\" \"mid\" \"hi\");\n}\n\
            potential (B | A) { data = ((0.1 0.2 0.7) % row yes\n (1 0 0)); }\n\
            potential ( A ) { data = (0.25 0.75); }\n";
        let network = parse(text).unwrap();
        assert_eq!(network.names, ["A", "B"]);
        assert_eq!(network.values[1], ["lo", "mid", "hi"]);
        let b = network.tables[1].as_ref().unwrap();
        assert_eq!(b.parents, [0]);
        assert_eq!(b.keys, [0, 1]);
        assert_eq!(b.probs, [0.1, 0.2, 0.7, 1.0, 0.0, 0.0]);
        assert!(
            parse("node A { states = (\"y\"); }\npotential (A) { data = (0.5 0.5); }").is_err()
        );
        assert!(parse("node A { states = (\"y\"); }\npotential (A | Z) { data = (1); }").is_err());
    }
}
