//! The UAI format: `BAYES` or `MARKOV`, the variables' domain sizes, the
//! functions' scopes, then each function's table, the scope's last
//! variable changing fastest. The variables are named `x0`, `x1`, ... and
//! their values `0`, `1`, ... (written `v_0`, `v_1`, ...).
//!
//! A `BAYES` function is the table of its scope's last variable given the
//! others. A `MARKOV` network that is a Bayesian network as written, each
//! function a conditional table of one variable of its scope, each
//! variable the child of one function, and the child relation acyclic (the
//! UAI suites' Promedus and Pedigree), is read as that network
//! ([`conditional_tables`]): its partition function is one, and its offset
//! zero. Any other `MARKOV` network's variables get uniform priors, times
//! their unary factors normalized, and every other factor `f` becomes a
//! variable `fJ` of one value `ok` whose row for each assignment of its
//! scope holds the factor's entry divided by its largest, observed. In
//! each assignment the program's weight is then the product of the
//! factors divided by each prior's sum and each factor's largest entry,
//! and by the entry of each factor with one distinct entry, which is left
//! out: so `log10 Z = log10 P(e) + c`, where the offset `c` is the sum of
//! the base-10 logarithms of all of these ([`partition_offset`]).
//!
//! For a program that asks nothing and observes nothing, a `MARKOV`
//! network whose variables all have the same `k` values and whose
//! functions are all Potts tables over two variables (one entry on the
//! diagonal, one off it), such as a graph colouring, is invariant under
//! any permutation of the values: the program breaks that symmetry
//! ([`colour_symmetry`]), which leaves `Z` a `k`-th of the network's, and
//! the offset holds `log10 k` besides.

use super::rows::fsum;
use super::{Network, Read};

/// A UAI file's parts: whether it is `MARKOV`, the domain sizes, the
/// functions' scopes and their tables.
struct File {
    markov: bool,
    sizes: Vec<usize>,
    scopes: Vec<Vec<usize>>,
    tables: Vec<Vec<f64>>,
}

fn read_file(text: &str) -> Result<File, String> {
    let mut tokens = text.split_whitespace();
    let mut take = || tokens.next().ok_or("UAI file ends early");
    let kind = take()?.to_ascii_uppercase();
    let markov = match kind.as_str() {
        "BAYES" => false,
        "MARKOV" => true,
        _ => return Err(format!("UAI file of kind {kind}: BAYES or MARKOV")),
    };
    let int = |t: &str| {
        t.parse::<usize>()
            .map_err(|_| format!("UAI file: `{t}` is not a count"))
    };
    let n = int(take()?)?;
    let sizes = (0..n)
        .map(|_| int(take()?))
        .collect::<Result<Vec<_>, _>>()?;
    let functions = int(take()?)?;
    let mut scopes = Vec::with_capacity(functions);
    for _ in 0..functions {
        let k = int(take()?)?;
        let scope = (0..k)
            .map(|_| int(take()?))
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(v) = scope.iter().find(|&&v| v >= n) {
            return Err(format!(
                "UAI scope names variable {v}, beyond the {n} of the file"
            ));
        }
        scopes.push(scope);
    }
    let mut tables = Vec::with_capacity(functions);
    for scope in &scopes {
        let entries = int(take()?)?;
        let expected: usize = scope.iter().map(|&v| sizes[v]).product();
        if entries != expected {
            return Err(format!(
                "UAI table of {entries} entries for a scope of {expected}"
            ));
        }
        let table = (0..entries)
            .map(|_| {
                let t = take()?;
                t.parse::<f64>()
                    .map_err(|_| format!("UAI file: `{t}` is not a number"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        tables.push(table);
    }
    if tokens.next().is_some() {
        return Err("UAI file has tokens after its tables".into());
    }
    Ok(File {
        markov,
        sizes,
        scopes,
        tables,
    })
}

/// The network of a UAI file, what it observes besides the evidence and
/// its offset; with `colours`, for a program that asks nothing and
/// observes nothing, a Markov network's colour symmetry broken where it
/// has one.
pub(super) fn parse(text: &str, colours: bool) -> Result<Read, String> {
    let File {
        markov,
        sizes,
        mut scopes,
        mut tables,
    } = read_file(text)?;
    let mut network = Network::default();
    for (i, &k) in sizes.iter().enumerate() {
        network.declare(&format!("x{i}"), (0..k).map(|u| u.to_string()).collect())?;
    }
    if !markov {
        for (scope, table) in scopes.into_iter().zip(tables) {
            let (&child, parents) = scope
                .split_last()
                .ok_or("UAI BAYES function of an empty scope")?;
            if network.tables[child].is_some() {
                return Err(format!("UAI BAYES file with two tables for x{child}"));
            }
            network.set_product_table(child, parents.to_vec(), table)?;
        }
        if let Some(v) = network.tables.iter().position(Option::is_none) {
            return Err(format!("UAI BAYES file without a table for x{v}"));
        }
        return Ok(Read::of(network));
    }
    if let Some(conditional) = conditional_tables(&sizes, &scopes, &tables) {
        for (v, (scope, table)) in conditional.into_iter().enumerate() {
            network.set_product_table(v, scope[..scope.len() - 1].to_vec(), table)?;
        }
        return Ok(Read {
            network,
            forced: Vec::new(),
            log10_offset: Some(0.0),
        });
    }
    let n = sizes.len();
    let mut prior: Vec<Vec<f64>> = sizes.iter().map(|&k| vec![1.0; k]).collect();
    let mut factors = Vec::new();
    // The constant factors, which the program leaves out: their entries
    // are factors of Z that P(e) does not hold.
    let mut constants = Vec::new();
    if colours && let Some((s, t)) = colour_symmetry(&sizes, &scopes, &tables) {
        // The network's Z is k times that of the functions so broken.
        (scopes, tables) = (s, t);
        constants.push(sizes[0] as f64);
    }
    for (scope, table) in scopes.into_iter().zip(tables) {
        if table.iter().any(|&w| w < 0.0) {
            return Err("UAI factor with a negative entry".into());
        }
        if scope.len() == 1 {
            for (a, b) in prior[scope[0]].iter_mut().zip(&table) {
                *a *= b;
            }
        } else if table.iter().any(|&w| w != table[0]) {
            factors.push((scope, table));
        } else if table[0] == 0.0 {
            return Err("UAI factor that is zero everywhere".into());
        } else {
            constants.push(table[0]);
        }
    }
    let mut logs = Vec::with_capacity(n + factors.len() + constants.len());
    for (i, weights) in prior.into_iter().enumerate() {
        let total = fsum(&weights);
        if total == 0.0 {
            return Err(format!("x{i}: its unary factors are zero everywhere"));
        }
        logs.push(total.log10());
        network.set_product_table(i, Vec::new(), weights.iter().map(|w| w / total).collect())?;
    }
    let mut forced = Vec::with_capacity(factors.len());
    for (j, (scope, table)) in factors.into_iter().enumerate() {
        let top = table.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        logs.push(top.log10());
        let f = network.declare(&format!("f{j}"), vec!["ok".into()])?;
        network.set_product_table(f, scope, table.iter().map(|w| w / top).collect())?;
        forced.push(f);
    }
    logs.extend(constants.iter().map(|c| c.log10()));
    Ok(Read {
        network,
        forced,
        log10_offset: Some(partition_offset(&logs)),
    })
}

/// `log10 Z - log10 P(e)` of a Markov network's program, the sum of the
/// base-10 logarithms of each prior's sum, each factor's largest entry and
/// each constant factor's entry (`logs`): in each assignment the program's
/// weight is the product of the factors divided by all of these, so
/// `P(e) · 10^offset` is `Z` given the same evidence.
fn partition_offset(logs: &[f64]) -> f64 {
    fsum(logs)
}

/// A Markov network that is a Bayesian network as written: each function
/// a conditional table of one variable of its scope (its child: for each
/// assignment of the others the entries over the child's values sum to one
/// within 1e-9), each variable the child of one function, and the child
/// relation acyclic. Returns each variable's scope, its child last, and its
/// table in that order; `None` for another network. A function that is a
/// table of two of its variables, an identity say, takes the child a
/// matching of functions to variables gives it.
#[allow(clippy::type_complexity)]
fn conditional_tables(
    sizes: &[usize],
    scopes: &[Vec<usize>],
    tables: &[Vec<f64>],
) -> Option<Vec<(Vec<usize>, Vec<f64>)>> {
    let n = sizes.len();
    if scopes.len() != n {
        return None;
    }
    let strides_of = |scope: &[usize]| -> Vec<usize> {
        (0..scope.len())
            .map(|i| scope[i + 1..].iter().map(|&u| sizes[u]).product())
            .collect()
    };
    let mut candidates: Vec<Vec<usize>> = Vec::with_capacity(n);
    for (scope, table) in scopes.iter().zip(tables) {
        let strides = strides_of(scope);
        let found: Vec<usize> = (0..scope.len())
            .filter(|&i| {
                let (k, stride) = (sizes[scope[i]], strides[i]);
                (0..table.len()).filter(|r| r / stride % k == 0).all(|r| {
                    let row: Vec<f64> = (0..k).map(|x| table[r + stride * x]).collect();
                    (fsum(&row) - 1.0).abs() <= 1e-9
                })
            })
            .collect();
        if found.is_empty() {
            return None;
        }
        candidates.push(found);
    }
    // Kuhn's augmenting paths, functions to the variables they may be the
    // table of; almost every function has one candidate.
    let mut owner: Vec<Option<usize>> = vec![None; n];
    for j in 0..n {
        let mut seen = vec![false; n];
        let mut path = vec![usize::MAX; n];
        let mut stack = vec![j];
        let mut augmented = false;
        while let Some(f) = stack.pop() {
            let mut free = None;
            for &i in &candidates[f] {
                let v = scopes[f][i];
                if seen[v] {
                    continue;
                }
                seen[v] = true;
                path[v] = f;
                match owner[v] {
                    None => {
                        free = Some(v);
                        break;
                    }
                    Some(o) => stack.push(o),
                }
            }
            if let Some(mut v) = free {
                loop {
                    let f = path[v];
                    let previous = candidates[f]
                        .iter()
                        .map(|&i| scopes[f][i])
                        .find(|&u| owner[u] == Some(f));
                    owner[v] = Some(f);
                    match previous {
                        Some(u) => v = u,
                        None => break,
                    }
                }
                augmented = true;
                break;
            }
        }
        if !augmented {
            return None;
        }
    }
    let mut out = Vec::with_capacity(n);
    for (v, f) in owner.iter().enumerate() {
        let f = f.expect("a perfect matching");
        let (scope, table) = (&scopes[f], &tables[f]);
        let at = scope
            .iter()
            .position(|&u| u == v)
            .expect("v is in its scope");
        let order: Vec<usize> = (0..scope.len()).filter(|&i| i != at).chain([at]).collect();
        let strides = strides_of(scope);
        let mut reordered = Vec::with_capacity(table.len());
        let mut key = vec![0usize; order.len()];
        for _ in 0..table.len() {
            reordered.push(
                table[key
                    .iter()
                    .zip(&order)
                    .map(|(a, &i)| a * strides[i])
                    .sum::<usize>()],
            );
            for j in (0..key.len()).rev() {
                key[j] += 1;
                if key[j] < sizes[scope[order[j]]] {
                    break;
                }
                key[j] = 0;
            }
        }
        out.push((
            order.iter().map(|&i| scope[i]).collect::<Vec<_>>(),
            reordered,
        ));
    }
    // The child relation is acyclic: a depth-first walk to the parents
    // meets no variable on its own path.
    let mut state = vec![0u8; n]; // 0 unseen, 1 on the path, 2 done
    for root in 0..n {
        if state[root] != 0 {
            continue;
        }
        state[root] = 1;
        let mut stack = vec![(root, 0usize)];
        while let Some((v, next)) = stack.last_mut() {
            let parents = &out[*v].0[..out[*v].0.len() - 1];
            if *next == parents.len() {
                state[*v] = 2;
                stack.pop();
                continue;
            }
            let u = parents[*next];
            *next += 1;
            match state[u] {
                1 => return None,
                0 => {
                    state[u] = 1;
                    stack.push((u, 0));
                }
                _ => {}
            }
        }
    }
    Some(out)
}

/// The colour symmetry of a Markov network, broken. In a network whose
/// variables all have the same k >= 2 values and whose functions are all
/// over two variables and Potts tables (one entry on the diagonal, one off
/// it), any permutation of the values maps a world to one of the same
/// weight. So without evidence Z is k times its value with v1, a variable
/// of largest degree, at 0; given that, the values 1..k-1 are
/// interchangeable, so v2, a neighbour of v1 of largest degree, takes 0
/// with weight 1 and 1 with weight k - 1 and no other value; given v1 and
/// v2, v3 (a common neighbour of largest degree, else one of v2's other
/// neighbours, if any) takes each value used so far with weight 1 and the
/// first unused one with the number of unused values as its weight.
/// Returns the functions with v1's and v2's unary tables added and the
/// table of v3 given v2 where there is a v3, whose Z is the network's
/// divided by k; `None` for another network.
#[allow(clippy::type_complexity)]
fn colour_symmetry(
    sizes: &[usize],
    scopes: &[Vec<usize>],
    tables: &[Vec<f64>],
) -> Option<(Vec<Vec<usize>>, Vec<Vec<f64>>)> {
    let k = *sizes.first()?;
    if k < 2 || sizes.iter().any(|&c| c != k) || scopes.is_empty() {
        return None;
    }
    let mut neighbours: Vec<std::collections::BTreeSet<usize>> =
        vec![Default::default(); sizes.len()];
    for (scope, table) in scopes.iter().zip(tables) {
        if scope.len() != 2 || scope[0] == scope[1] {
            return None;
        }
        let at = |i: usize, j: usize| table[i * k + j];
        let diagonal = at(0, 0);
        let off = at(0, 1);
        for i in 0..k {
            for j in 0..k {
                if at(i, j) != if i == j { diagonal } else { off } {
                    return None;
                }
            }
        }
        neighbours[scope[0]].insert(scope[1]);
        neighbours[scope[1]].insert(scope[0]);
    }
    // The largest degree, the least variable among equals.
    let largest = |vs: &mut dyn Iterator<Item = usize>| {
        vs.max_by_key(|&v| (neighbours[v].len(), std::cmp::Reverse(v)))
    };
    let v1 = largest(&mut (0..sizes.len()))?;
    let v2 = largest(&mut neighbours[v1].iter().copied())?;
    let common: Vec<usize> = neighbours[v1]
        .intersection(&neighbours[v2])
        .copied()
        .collect();
    let others: Vec<usize> = if common.is_empty() {
        neighbours[v2]
            .iter()
            .copied()
            .filter(|&v| v != v1)
            .collect()
    } else {
        common
    };
    let mut scopes = scopes.to_vec();
    let mut tables = tables.to_vec();
    scopes.extend([vec![v1], vec![v2]]);
    let mut first = vec![0.0; k];
    first[0] = 1.0;
    let mut second = vec![0.0; k];
    (second[0], second[1]) = (1.0, (k - 1) as f64);
    tables.extend([first, second]);
    if let Some(v3) = largest(&mut others.into_iter()) {
        let mut table = Vec::with_capacity(k * k);
        for c2 in 0..k {
            let mut row = vec![0.0; k];
            if c2 <= 1 {
                let used = c2 + 1; // v1 = 0 and v2 = c2
                row[..used].fill(1.0);
                if used < k {
                    row[used] = (k - used) as f64;
                }
            }
            table.extend(row);
        }
        scopes.push(vec![v2, v3]);
        tables.push(table);
    }
    Some((scopes, tables))
}

/// Whether a UAI evidence file observes a variable.
pub(super) fn observes(text: &str) -> Result<bool, String> {
    Ok(match text.split_whitespace().next() {
        None => false,
        Some(t) => {
            t.parse::<i64>()
                .map_err(|_| format!("evidence: `{t}` is not a count"))?
                != 0
        }
    })
}

/// The observations of a UAI evidence file, `N i1 u1 ... iN uN`, a
/// variable and a value index each, in the network's order.
pub(super) fn evid(text: &str, network: &Network) -> Result<Vec<(usize, usize)>, String> {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    let count = tokens.first().and_then(|t| t.parse::<usize>().ok());
    if count.is_none_or(|c| tokens.len() != 1 + 2 * c) {
        return Err("evidence: not a UAI evidence file of one sample".into());
    }
    let mut given: Vec<Option<usize>> = vec![None; network.names.len()];
    for pair in tokens[1..].chunks(2) {
        let v = network.variable(&format!("x{}", pair[0]))?;
        let u = network.value(v, pair[1])?;
        if given[v].replace(u).is_some() {
            return Err(format!("evidence: x{} observed twice", pair[0]));
        }
    }
    Ok(given
        .into_iter()
        .enumerate()
        .filter_map(|(v, u)| Some((v, u?)))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bayes_function_is_its_last_variables_table() {
        let read = parse("BAYES 2 2 2 2 1 0 2 0 1 2 0.2 0.8 4 0.1 0.9 0.6 0.4", false).unwrap();
        assert_eq!(read.log10_offset, None);
        let t = read.network.tables[1].as_ref().unwrap();
        assert_eq!(t.parents, [0]);
        assert_eq!(t.probs, [0.1, 0.9, 0.6, 0.4]);
    }

    #[test]
    fn a_markov_file_of_conditional_tables_is_the_network() {
        // x1's table given x0 is written with x1 first in its scope.
        let read = parse(
            "MARKOV 2 2 2 2 1 0 2 1 0 2 0.2 0.8 4 0.1 0.6 0.9 0.4",
            false,
        )
        .unwrap();
        assert_eq!(read.log10_offset, Some(0.0));
        assert!(read.forced.is_empty());
        let t = read.network.tables[1].as_ref().unwrap();
        assert_eq!(t.parents, [0]);
        assert_eq!(t.probs, [0.1, 0.9, 0.6, 0.4]);
    }

    #[test]
    fn a_markov_factor_is_an_observed_variable_of_its_rows_over_its_largest() {
        let read = parse("MARKOV 2 2 2 3 1 0 2 0 1 0 2 1 3 4 2 1 1 4 1 7", false).unwrap();
        assert_eq!(read.forced, [2]);
        let f = read.network.tables[2].as_ref().unwrap();
        assert_eq!(f.parents, [0, 1]);
        assert_eq!(f.probs, [0.5, 0.25, 0.25, 1.0]);
        assert_eq!(read.network.tables[0].as_ref().unwrap().probs, [0.25, 0.75]);
        let c = read.log10_offset.unwrap();
        // x0's prior sums to 4, x1's to 2, the factor's largest entry is 4,
        // and the constant factor 7 (of the empty scope) is left out.
        let expected = 4f64.log10() + 2f64.log10() + 4f64.log10() + 7f64.log10();
        assert!((c - expected).abs() < 1e-15);
    }

    #[test]
    fn a_potts_network_without_evidence_has_its_symmetry_broken() {
        // A triangle coloured with three colours.
        let potts = "MARKOV 3 3 3 3 3 2 0 1 2 1 2 2 0 2 \
            9 1 2 2 2 1 2 2 2 1 9 1 2 2 2 1 2 2 2 1 9 1 2 2 2 1 2 2 2 1";
        let plain = parse(potts, false).unwrap();
        let broken = parse(potts, true).unwrap();
        assert_eq!(plain.forced.len(), 3);
        assert_eq!(broken.forced.len(), 4);
        assert_eq!(
            broken.network.tables[0].as_ref().unwrap().probs,
            [1.0, 0.0, 0.0]
        );
        let offset = broken.log10_offset.unwrap() - plain.log10_offset.unwrap();
        // v1 = x0 at 0 (its prior sums to 1, not 3), v2 = x1 (to 3, as
        // before), the table of x2 given x1 (largest entry 2), and the
        // constant k = 3.
        let expected = -3f64.log10() + 2f64.log10() + 3f64.log10();
        assert!((offset - expected).abs() < 1e-12);
    }

    #[test]
    fn evidence_names_variables_by_index() {
        let read = parse(
            "BAYES 2 2 3 2 1 0 2 0 1 2 0.2 0.8 6 0.1 0.2 0.7 0.3 0.3 0.4",
            false,
        )
        .unwrap();
        assert_eq!(evid("2 1 2 0 1", &read.network).unwrap(), [(0, 1), (1, 2)]);
        assert!(evid("1 1 3", &read.network).is_err());
        assert!(evid("2 1 2", &read.network).is_err());
        assert!(observes("0").is_ok_and(|o| !o));
        assert!(observes("1 0 1").unwrap());
    }
}
