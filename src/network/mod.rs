//! Discrete Bayesian and Markov networks, read as programs.
//!
//! A network file is read as a ProbLog program, which `tddlog --program`
//! prints, and that program is what tddlog answers. It is the program the
//! benchmarks' Python script `gen_bn.py` (on the baselines branch), which
//! this reader replaced, wrote, byte for byte: every variable `X` with
//! parents `P1..Pm` becomes one annotated disjunction per row of its
//! conditional table,
//!
//! ```text
//! p1::x(v1); ...; pk::x(vk) :- p1(u1), ..., pm(um).
//! ```
//!
//! so the program is ground and every rule is a row. Names are lowercased
//! and every other character but a letter, a digit or `_` becomes `_`, with
//! `v_` before a name that does not start with a letter; a clash is an
//! error. Zero-probability heads are dropped, a row with one certain value
//! is a plain rule, and a row that sums to one up to the rounding of its
//! decimals is divided by its sum (`rows.rs`). The rules come in the order
//! of a depth-first walk from each variable in turn to its parents.
//!
//! The formats, by the file's extension ([`Format::of_path`]): BIF
//! (`.bif`), Hugin (`.net`, `.hugin`), GeNIe (`.xdsl`, its `cpt` nodes),
//! and the UAI format (`.uai`), whose variables are named `x0`, `x1`, ...
//! and their values `v_0`, `v_1`, .... A UAI `MARKOV` network's variables
//! get uniform priors, times their unary factors normalized, and every
//! other factor `f` becomes a variable `fJ` of one value `ok`, observed,
//! whose row for each assignment of its scope holds the factor's entry
//! divided by its largest: `P(e)` of the program is then the partition
//! function given the evidence divided by `10^c`, the offset
//! [`NetworkProgram::log10_offset`] gives and the program's first line
//! states (`uai.rs`).
//!
//! The evidence is an Ace `.inst` file or, for a UAI network, a UAI
//! `.evid` file. The queries are every value of every variable, or of the
//! variables without children, or of the variables named, the observed
//! ones left out; or none, the program for `P(e)` alone, which breaks the
//! colour symmetry of a Markov network without evidence where it has one.

mod bif;
mod hugin;
pub(crate) mod rows;
mod scan;
mod uai;
mod xdsl;
mod xml;

use std::collections::HashMap;
use std::fmt::Write;

/// A network file's format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// The Bayesian Interchange Format, `.bif`.
    Bif,
    /// Hugin's `.net` (also `.hugin`).
    Hugin,
    /// GeNIe's `.xdsl`.
    Xdsl,
    /// The UAI inference competitions' `.uai`, `BAYES` or `MARKOV`.
    Uai,
}

impl Format {
    /// The format of the file at `path`, by its extension; `None` for any
    /// other file, a program.
    pub fn of_path(path: &str) -> Option<Format> {
        let extension = std::path::Path::new(path).extension()?.to_str()?;
        Some(match extension {
            "bif" => Format::Bif,
            "net" | "hugin" => Format::Hugin,
            "xdsl" => Format::Xdsl,
            "uai" => Format::Uai,
            _ => return None,
        })
    }
}

/// Which variables the program queries, each at every value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Queries {
    /// Every variable.
    All,
    /// The variables without children.
    Leaves,
    /// The variables named, as the file names them.
    Named(Vec<String>),
    /// None: the program for `P(e)` alone.
    None,
}

impl std::str::FromStr for Queries {
    type Err = String;

    /// `all`, `leaves`, or variables' names separated by commas.
    fn from_str(s: &str) -> Result<Queries, String> {
        Ok(match s {
            "all" => Queries::All,
            "leaves" => Queries::Leaves,
            _ if s.is_empty() => return Err("no variable to query".into()),
            _ => Queries::Named(s.split(',').map(str::to_string).collect()),
        })
    }
}

/// An evidence file's contents.
#[derive(Clone, Copy, Debug)]
pub enum Evidence<'a> {
    /// Ace's `.inst`: `<inst id="X" value="v"/>` under `<instantiation>`.
    Inst(&'a [u8]),
    /// A UAI evidence file, `.evid`: `N i1 u1 ... iN uN`.
    Uai(&'a [u8]),
}

impl<'a> Evidence<'a> {
    /// The evidence of the file at `path`, whose contents are `data`, by
    /// its extension.
    pub fn of_path(path: &str, data: &'a [u8]) -> Result<Evidence<'a>, String> {
        match std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
        {
            Some("inst") => Ok(Evidence::Inst(data)),
            Some("evid") => Ok(Evidence::Uai(data)),
            _ => Err(format!(
                "{path}: evidence is an Ace .inst file or a UAI .evid file"
            )),
        }
    }
}

/// How a network is read.
#[derive(Clone, Debug)]
pub struct ReadOptions<'a> {
    pub queries: Queries,
    pub evidence: Option<Evidence<'a>>,
    /// Divide every row by its sum, for a file that writes its tables in
    /// percent or as weights, as Ace's `-Xnormalize` does on reading.
    pub rescale: bool,
}

/// A network's program.
#[derive(Clone, Debug)]
pub struct NetworkProgram {
    /// The program's text.
    pub program: String,
    /// For a UAI `MARKOV` network, `log10 Z - log10 P(e)`: the base-10
    /// logarithm of the partition function given the evidence is the
    /// program's `log10 P(e)` plus this.
    pub log10_offset: Option<f64>,
}

/// The program of the network file whose contents are `data`, in
/// `format`, read under `options`.
pub fn read(data: &[u8], format: Format, options: &ReadOptions) -> Result<NetworkProgram, String> {
    if let (Some(Evidence::Uai(_)), false) = (options.evidence, format == Format::Uai) {
        return Err("UAI evidence (.evid) is for a .uai network".into());
    }
    let read = match format {
        Format::Bif => Read::of(bif::parse(&text(data))?),
        Format::Hugin => Read::of(hugin::parse(&text(data))?),
        Format::Xdsl => Read::of(xdsl::parse(data)?),
        Format::Uai => {
            // A program that asks nothing and observes nothing breaks a
            // Markov network's colour symmetry where it has one.
            let observes = match options.evidence {
                None => false,
                Some(Evidence::Inst(_)) => true,
                Some(Evidence::Uai(e)) => uai::observes(&text(e))?,
            };
            uai::parse(&text(data), options.queries == Queries::None && !observes)?
        }
    };
    let Read {
        mut network,
        forced,
        log10_offset,
    } = read;
    if options.rescale {
        network.rescale()?;
    }
    let mut evidence: Vec<(usize, usize)> = forced.into_iter().map(|f| (f, 0)).collect();
    match options.evidence {
        None => {}
        Some(Evidence::Inst(e)) => evidence.extend(xdsl::inst(e, &network)?),
        Some(Evidence::Uai(e)) => evidence.extend(uai::evid(&text(e), &network)?),
    }
    let mut program = String::new();
    if let Some(c) = log10_offset {
        program.push_str("% log10 Z - log10 P(e) = ");
        rows::write_float(&mut program, c);
        program.push('\n');
    }
    network.write(&mut program, &options.queries, &evidence)?;
    Ok(NetworkProgram {
        program,
        log10_offset,
    })
}

/// A file's text: UTF-8, or else ISO-8859-1, which Ace's water, pigs and
/// mildew `.hugin` files are written in.
fn text(data: &[u8]) -> std::borrow::Cow<'_, str> {
    match std::str::from_utf8(data) {
        Ok(s) => s.into(),
        Err(_) => data
            .iter()
            .map(|&b| char::from(b))
            .collect::<String>()
            .into(),
    }
}

/// A network as read, with what its program observes besides the evidence
/// (a Markov network's factors, each at its one value) and its offset.
struct Read {
    network: Network,
    forced: Vec<usize>,
    log10_offset: Option<f64>,
}

impl Read {
    fn of(network: Network) -> Read {
        Read {
            network,
            forced: Vec::new(),
            log10_offset: None,
        }
    }
}

/// A discrete network: its variables in the file's order, their values
/// and conditional tables.
#[derive(Debug, Default)]
struct Network {
    names: Vec<String>,
    values: Vec<Vec<String>>,
    tables: Vec<Option<Table>>,
    index: HashMap<String, usize>,
}

/// A variable's conditional table: its parents, and its rows, each an
/// assignment of the parents (one value index per parent, `keys` flat)
/// and the probabilities of the variable's values (`probs` flat).
#[derive(Debug)]
struct Table {
    parents: Vec<usize>,
    keys: Vec<u32>,
    probs: Vec<f64>,
}

impl Network {
    /// Declares a variable with its values; a second declaration of a name
    /// is an error.
    fn declare(&mut self, name: &str, values: Vec<String>) -> Result<usize, String> {
        let v = self.names.len();
        if self.index.insert(name.to_string(), v).is_some() {
            return Err(format!("{name} is declared twice"));
        }
        self.names.push(name.to_string());
        self.values.push(values);
        self.tables.push(None);
        Ok(v)
    }

    fn variable(&self, name: &str) -> Result<usize, String> {
        self.index
            .get(name)
            .copied()
            .ok_or_else(|| format!("no variable {name}"))
    }

    fn value(&self, v: usize, value: &str) -> Result<usize, String> {
        self.values[v]
            .iter()
            .position(|u| u == value)
            .ok_or_else(|| format!("{} has no value {value}", self.names[v]))
    }

    /// Every assignment of `parents`, the last changing fastest, flat.
    fn assignments(&self, parents: &[usize]) -> Vec<u32> {
        let sizes: Vec<usize> = parents.iter().map(|&p| self.values[p].len()).collect();
        let count: usize = sizes.iter().product();
        let mut keys = Vec::with_capacity(count * parents.len());
        let mut key = vec![0u32; parents.len()];
        for _ in 0..count {
            keys.extend_from_slice(&key);
            for j in (0..key.len()).rev() {
                key[j] += 1;
                if (key[j] as usize) < sizes[j] {
                    break;
                }
                key[j] = 0;
            }
        }
        keys
    }

    /// Sets `v`'s table to `probs`, one row per assignment of `parents` in
    /// order, the last parent changing fastest.
    fn set_product_table(
        &mut self,
        v: usize,
        parents: Vec<usize>,
        probs: Vec<f64>,
    ) -> Result<(), String> {
        let keys = self.assignments(&parents);
        let rows = keys.len().checked_div(parents.len()).unwrap_or(1);
        let k = self.values[v].len();
        if probs.len() != rows * k {
            return Err(format!(
                "{}: a table of {} numbers for {rows} rows of {k} values",
                self.names[v],
                probs.len()
            ));
        }
        self.tables[v] = Some(Table {
            parents,
            keys,
            probs,
        });
        Ok(())
    }

    /// The table of each variable; an error for a variable without one.
    fn tables(&self) -> Result<Vec<&Table>, String> {
        self.tables
            .iter()
            .zip(&self.names)
            .map(|(t, name)| t.as_ref().ok_or_else(|| format!("{name} has no table")))
            .collect()
    }

    /// Every row divided by its sum (`ReadOptions::rescale`); a row of
    /// zeros is an error.
    fn rescale(&mut self) -> Result<(), String> {
        for (v, table) in self.tables.iter_mut().enumerate() {
            let Some(table) = table else { continue };
            let k = self.values[v].len();
            for row in table.probs.chunks_mut(k) {
                let total = rows::fsum(row);
                if total.is_nan() || total <= 0.0 {
                    return Err(format!(
                        "{}: a row of zeros cannot be rescaled",
                        self.names[v]
                    ));
                }
                for p in row {
                    *p /= total;
                }
            }
        }
        Ok(())
    }

    /// Each variable after its parents, depth first from each variable in
    /// turn, the parents in their table's order. A stack, not recursion: a
    /// chain of ancestors can be as long as the network.
    fn topological(&self, tables: &[&Table]) -> Vec<usize> {
        let mut done = vec![false; self.names.len()];
        let mut out = Vec::with_capacity(self.names.len());
        for root in 0..self.names.len() {
            if done[root] {
                continue;
            }
            done[root] = true;
            let mut stack = vec![(root, 0)];
            while let Some((v, next)) = stack.last_mut() {
                let parents = &tables[*v].parents;
                while *next < parents.len() && done[parents[*next]] {
                    *next += 1;
                }
                if *next == parents.len() {
                    out.push(*v);
                    stack.pop();
                } else {
                    let p = parents[*next];
                    done[p] = true;
                    stack.push((p, 0));
                }
            }
        }
        out
    }

    /// The variables `queries` asks, observed or not.
    fn queried(&self, tables: &[&Table], queries: &Queries) -> Result<Vec<usize>, String> {
        Ok(match queries {
            Queries::All => (0..self.names.len()).collect(),
            Queries::Leaves => {
                let mut parent = vec![false; self.names.len()];
                for t in tables {
                    for &p in &t.parents {
                        parent[p] = true;
                    }
                }
                (0..self.names.len()).filter(|&v| !parent[v]).collect()
            }
            Queries::Named(names) => names
                .iter()
                .map(|n| self.variable(n))
                .collect::<Result<_, _>>()?,
            Queries::None => Vec::new(),
        })
    }

    /// Writes the program: the rules in topological order, then the
    /// observations `evidence` (a variable and a value index each), then
    /// the queries, each line ending in a newline.
    fn write(
        &self,
        out: &mut String,
        queries: &Queries,
        evidence: &[(usize, usize)],
    ) -> Result<(), String> {
        let tables = self.tables()?;
        let atoms = self.atoms()?;
        let start = out.len();
        let mut row = Vec::new();
        for v in self.topological(&tables) {
            let t = tables[v];
            let (k, m) = (self.values[v].len(), t.parents.len());
            for (r, probs) in t.probs.chunks(k).enumerate() {
                row.clear();
                row.extend_from_slice(probs);
                let key = &t.keys[r * m..(r + 1) * m];
                rows::normalized(&mut row).map_err(|e| {
                    let at: Vec<&str> = key
                        .iter()
                        .zip(&t.parents)
                        .map(|(&u, &p)| self.values[p][u as usize].as_str())
                        .collect();
                    format!("{} given ({}): {e}", self.names[v], at.join(", "))
                })?;
                let heads: Vec<(f64, usize)> = row
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| **p > 0.0)
                    .map(|(u, &p)| (p, u))
                    .collect();
                match heads[..] {
                    // No value under this row: no rule.
                    [] => continue,
                    [(p, u)] if (p - 1.0).abs() < 1e-9 => out.push_str(&atoms[v][u]),
                    _ => {
                        for (j, &(p, u)) in heads.iter().enumerate() {
                            if j > 0 {
                                out.push_str("; ");
                            }
                            rows::write_float(out, p);
                            out.push_str("::");
                            out.push_str(&atoms[v][u]);
                        }
                    }
                }
                for (j, (&u, &p)) in key.iter().zip(&t.parents).enumerate() {
                    out.push_str(if j == 0 { " :- " } else { ", " });
                    out.push_str(&atoms[p][u as usize]);
                }
                out.push_str(".\n");
            }
        }
        let mut observed = vec![false; self.names.len()];
        for &(v, u) in evidence {
            observed[v] = true;
            writeln!(out, "evidence({}).", atoms[v][u]).expect("a String takes any text");
        }
        if *queries != Queries::None {
            let queried: Vec<usize> = self
                .queried(&tables, queries)?
                .into_iter()
                .filter(|&v| !observed[v])
                .collect();
            if queried.is_empty() {
                return Err("every queried variable is observed, nothing is left to query".into());
            }
            for v in queried {
                for atom in &atoms[v] {
                    writeln!(out, "query({atom}).").expect("a String takes any text");
                }
            }
        }
        if out.len() == start {
            out.push('\n');
        }
        Ok(())
    }

    /// The atom `name(value)` of each value of each variable, by
    /// [`ident`]; an error where two variables, or two values of one,
    /// write alike.
    fn atoms(&self) -> Result<Vec<Vec<String>>, String> {
        let mut seen: HashMap<String, usize> = HashMap::with_capacity(self.names.len());
        let mut atoms = Vec::with_capacity(self.names.len());
        for (v, name) in self.names.iter().enumerate() {
            let n = ident(name);
            if let Some(&w) = seen.get(&n) {
                return Err(format!("name clash: {name} and {} -> {n}", self.names[w]));
            }
            let mut values: Vec<String> = Vec::with_capacity(self.values[v].len());
            for u in &self.values[v] {
                let atom = format!("{n}({})", ident(u));
                if values.contains(&atom) {
                    return Err(format!("value clash in {name}: {:?}", self.values[v]));
                }
                values.push(atom);
            }
            seen.insert(n, v);
            atoms.push(values);
        }
        Ok(atoms)
    }
}

/// A name as the program writes it: lowercased, every character but
/// `a-z`, `0-9` and `_` written `_`, and `v_` before a name that does not
/// start with a letter.
fn ident(s: &str) -> String {
    let t: String = s
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if t.starts_with(|c: char| c.is_ascii_lowercase()) {
        t
    } else {
        format!("v_{t}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_lowercased_and_its_other_characters_underscores() {
        assert_eq!(ident("Smoker"), "smoker");
        assert_eq!(ident("<12%"), "v__12_");
        assert_eq!(ident("0"), "v_0");
        assert_eq!(ident("_a"), "v__a");
        assert_eq!(ident("x-ray Ä"), "x_ray__");
        assert_eq!(ident(""), "v_");
    }

    #[test]
    fn a_format_and_its_evidence_go_by_extension() {
        assert_eq!(Format::of_path("a/b.net"), Some(Format::Hugin));
        assert_eq!(Format::of_path("b.hugin"), Some(Format::Hugin));
        assert_eq!(Format::of_path("b.bif"), Some(Format::Bif));
        assert_eq!(Format::of_path("b.xdsl"), Some(Format::Xdsl));
        assert_eq!(Format::of_path("b.uai"), Some(Format::Uai));
        assert_eq!(Format::of_path("b.pl"), None);
        assert_eq!(Format::of_path("b.net.gz"), None);
        assert!(Evidence::of_path("e.txt", b"").is_err());
        assert_eq!(
            "a,B".parse(),
            Ok(Queries::Named(vec!["a".into(), "B".into()]))
        );
    }
}
