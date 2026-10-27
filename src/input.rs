//! Every input tddlog reads, as the program it answers ([`read`]): a
//! ProbLog program, or an LPAD written for cplint ([`syntax::lpad`]); a
//! Dice program ([`syntax::dice`]); and a Bayesian or Markov network file
//! ([`network`]) with its evidence file. The command line names an input's
//! format by the file's extension ([`Format::of_path`]), and the bindings
//! by its name ([`Format`]'s `FromStr`).

use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

use crate::network::{self, Evidence, Queries, ReadOptions};
use crate::syntax::{self, ParseError};
use crate::{Error, Solution};

/// The languages and file formats tddlog reads; ProbLog by default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Format {
    /// A ProbLog program, or an LPAD written for cplint where the text
    /// holds the directive `:- begin_lpad.` ([`syntax::lpad`]).
    #[default]
    Problog,
    /// A Dice program ([`syntax::dice`]).
    Dice,
    /// A network file ([`network`]).
    Network(network::Format),
}

/// Each format's name, as [`Format`]'s `FromStr` takes it.
const NAMES: [(&str, Format); 6] = [
    ("problog", Format::Problog),
    ("dice", Format::Dice),
    ("bif", Format::Network(network::Format::Bif)),
    ("hugin", Format::Network(network::Format::Hugin)),
    ("xdsl", Format::Network(network::Format::Xdsl)),
    ("uai", Format::Network(network::Format::Uai)),
];

impl Format {
    /// The format of the file at `path`, by its extension: `.dice` a Dice
    /// program, a network's ([`network::Format::of_path`]), and a ProbLog
    /// program for any other.
    pub fn of_path(path: &str) -> Format {
        if path.ends_with(".dice") {
            return Format::Dice;
        }
        network::Format::of_path(path).map_or(Format::Problog, Format::Network)
    }

    /// The contents `data` of an evidence file, as a network in this format
    /// reads them where nothing else says which file it is: a UAI `.evid`
    /// file for a UAI network, an Ace `.inst` file for any other. [`read`]
    /// refuses evidence for an input that is not a network.
    pub fn evidence(self, data: &[u8]) -> Evidence<'_> {
        match self {
            Format::Network(network::Format::Uai) => Evidence::Uai(data),
            _ => Evidence::Inst(data),
        }
    }
}

impl FromStr for Format {
    type Err = String;

    /// `problog`, `dice`, or a network's format: `bif`, `hugin` (`.net`),
    /// `xdsl` or `uai`.
    fn from_str(s: &str) -> Result<Format, String> {
        NAMES
            .iter()
            .find(|(name, _)| *name == s)
            .map(|&(_, format)| format)
            .ok_or_else(|| format!("unknown format `{s}`: problog, dice, bif, hugin, xdsl or uai"))
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (name, _) = NAMES
            .iter()
            .find(|(_, format)| format == self)
            .expect("named");
        f.write_str(name)
    }
}

/// An input read as the program tddlog answers.
#[derive(Clone, Debug)]
pub struct Input<'a> {
    /// The program [`crate::solve`] takes: a ProbLog program's own text,
    /// borrowed, or the program another input is read as, which
    /// `tddlog --program` prints.
    pub program: Cow<'a, str>,
    /// For a UAI `MARKOV` network, `log10 Z - log10 P(e)`
    /// ([`network::NetworkProgram::log10_offset`]).
    pub log10_offset: Option<f64>,
    /// The example queries of an LPAD that its program leaves out, each
    /// with why ([`syntax::lpad::Lpad::left_out`]).
    pub left_out: Vec<String>,
}

impl Input<'_> {
    /// `P(e)` of the input, from its program's in `solution`, where it was
    /// asked for: a float (zero under the least, infinite over the
    /// largest) and its base-10 logarithm, exact beyond a float's range.
    /// For a Markov network, it is the partition function given the
    /// evidence, `Z`, the program's `P(e)` times 10 to
    /// [`Input::log10_offset`].
    pub fn evidence_probability(&self, solution: &Solution) -> Option<(f64, f64)> {
        let p = solution.evidence_probability?;
        Some(match self.log10_offset {
            None => (p.to_f64(), p.log10()),
            Some(c) => (p.to_f64() * 10f64.powf(c), p.log10() + c),
        })
    }
}

/// Read `data`, an input in `format`, as the program tddlog answers.
///
/// A ProbLog program is its own text, which must be UTF-8; an LPAD, a
/// Dice program and a network are read as the programs their modules
/// write. `options` say how a network is read: its queries, its evidence
/// and whether its rows are rescaled. For any other input they must be
/// the defaults, but for the queries, which may be left out
/// ([`Queries::None`]) as [`crate::Options::queries`] leaves them out.
pub fn read<'a>(data: &'a [u8], format: Format, options: &ReadOptions) -> Result<Input<'a>, Error> {
    let input = |program, left_out| Input {
        program,
        log10_offset: None,
        left_out,
    };
    let network = match format {
        Format::Network(network) => network,
        Format::Problog | Format::Dice => {
            let given = [
                (options.evidence.is_some(), "an evidence file"),
                (options.rescale, "rescaling the rows"),
                (
                    matches!(options.queries, Queries::Leaves | Queries::Named(_)),
                    "choosing the variables queried",
                ),
            ];
            if let Some((_, what)) = given.iter().find(|(given, _)| *given) {
                return Err(Error::Options(format!(
                    "{what} is for a network (bif, hugin, xdsl or uai), and the input is {format}"
                )));
            }
            let text = utf8(data)?;
            return Ok(match format {
                Format::Dice => input(Cow::Owned(syntax::dice::read(text)?), Vec::new()),
                _ => match syntax::lpad::read(text).transpose()? {
                    None => input(Cow::Borrowed(text), Vec::new()),
                    // `--program` prints it as a file, with its last line's
                    // newline.
                    Some(lpad) => input(Cow::Owned(lpad.program + "\n"), lpad.left_out),
                },
            });
        }
    };
    let read = network::read(data, network, options).map_err(Error::Network)?;
    Ok(Input {
        program: Cow::Owned(read.program),
        log10_offset: read.log10_offset,
        left_out: Vec::new(),
    })
}

/// `data` as text, failing at the line of its first byte that is not
/// UTF-8.
fn utf8(data: &[u8]) -> Result<&str, ParseError> {
    std::str::from_utf8(data).map_err(|e| ParseError {
        line: 1 + data[..e.valid_up_to()]
            .iter()
            .filter(|&&b| b == b'\n')
            .count(),
        msg: "the text is not UTF-8".into(),
    })
}
