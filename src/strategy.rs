//! What a run's strategy decides. A [`Strategy`] is `auto`'s trials or one
//! layout, a [`Kind`]; a kind fixes, beside the vtree `vtree::build` lays
//! out, the grounder's options, the rewrite of the program it compiles,
//! the formulation and the projection, and [`Kind::prepare`] builds its
//! program and layout.

use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use crate::ground::deps::Deps;
use crate::ground::{self, GroundOptions, GroundProgram};
use crate::tp::{Formulation, Projection};
use crate::vtree::{self, Kind};
use crate::{Error, indicators};

/// How a run lays out its vtree.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Strategy {
    /// Trials of layouts over rewrites of the program, the first to finish
    /// its compile, evidence and queries answering (`src/auto/`). No static
    /// score of a vtree measured so far predicts which one a program wants,
    /// so `auto` finds out by compiling, under bounds that grow (see
    /// `docs/design.md`).
    #[default]
    Auto,
    /// One layout, compiled without bounds.
    Trial(Kind),
}

impl Strategy {
    /// The grounder's `options` for a run under this strategy
    /// ([`Kind::ground_options`]; `auto`'s are `options`).
    pub(crate) fn ground_options(self, options: GroundOptions) -> GroundOptions {
        match self {
            Strategy::Auto => options,
            Strategy::Trial(kind) => kind.ground_options(options),
        }
    }
}

impl FromStr for Strategy {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "auto" => Ok(Strategy::Auto),
            _ => s.parse().map(Strategy::Trial),
        }
    }
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Strategy::Auto => f.write_str("auto"),
            Strategy::Trial(kind) => kind.fmt(f),
        }
    }
}

impl Kind {
    /// The grounder's `options` for a run under this layout. The layouts
    /// that read the program's text order, `program` and `decision`, read
    /// the program with its views in place ([`GroundOptions::views`]): a
    /// view inlined into a reader moves the view's facts to the reader.
    /// `decision` then loses which of the facts a reader decides between
    /// comes first (the weekend circuit at 12 bits and 1 024 pieces: 360
    /// facts outside the tree, a vtree 360 deep against 26, and a compile
    /// out of 16 GB; [`ground::guarded`] inlines them after the
    /// conditioning). `program`'s cuts widen: a grid's boundary chains,
    /// views over facts, are read by every later row, and the widest cut
    /// of geni_link_16 grows from 17 to 30 atoms, so that the naming finds
    /// 21 cuts in place of 67 and the compile a formula of 113 M pairs in
    /// place of 3 022 (`vtree::naming`). `program:fanin` reads its order
    /// off the rules, and the inlined program serves it better: with the
    /// views in place the circuits it wins took 2 901 s against 2 184
    /// (bb_pi_w18 4.4 times as long).
    pub(crate) fn ground_options(self, options: GroundOptions) -> GroundOptions {
        match self {
            Kind::Decision | Kind::Program { fan_in: false } => GroundOptions {
                views: false,
                ..options
            },
            _ => options,
        }
    }

    /// Whether this layout reads its program through indicators: every
    /// atom another component reads, for `families`, which lays them out,
    /// as `auto`'s families trials do; none for the others.
    pub fn indicators(self) -> bool {
        matches!(self, Kind::Families { .. })
    }

    /// [`Kind::indicators`] as trial lines and `--stats` print it: `all`
    /// or `off`.
    pub fn indicators_name(self) -> &'static str {
        if self.indicators() { "all" } else { "off" }
    }

    /// The formulation this layout compiles under: the primal under a
    /// dynamic vtree, the dual under the others. On the random graphs,
    /// whose one trial is dynamic, the search ends on smaller vtrees under
    /// the primal, and g20_60 runs about 1.7 times faster. No other program
    /// moved, so the dual, which the indicators and clause tables were tuned
    /// under, stays for the rest.
    pub fn formulation(self) -> Formulation {
        if self.is_dynamic() {
            Formulation::Primal
        } else {
            Formulation::Dual
        }
    }

    /// The projection this layout compiles under: `asked`, but off for a
    /// dynamic vtree, whose search changes the one vtree every formula
    /// lives on.
    pub fn projection(self, asked: Projection) -> Projection {
        if self.is_dynamic() {
            Projection::Off
        } else {
            asked
        }
    }

    /// The program this layout compiles, from `gp` as grounded under the
    /// layout's options ([`crate::Options::ground_options`]): written
    /// without its linear recursion (`Kind::eliminated`), then rewritten
    /// for the layout (`Kind::rewrite_eliminated`). `auto` picks its trials
    /// from the program as grounded, whose recursive components say which
    /// layouts can matter, eliminates once for all of them, and rewrites
    /// each trial's program from that. `td`'s layout may add separator
    /// copies to the rewrite, and those of `program` and `decision` their
    /// names (`Prepared::lay_out`).
    pub fn rewrite(self, gp: GroundProgram) -> GroundProgram {
        self.rewrite_eliminated(Kind::eliminated(gp))
    }

    /// The first step of every layout's rewrite: `gp` written without its
    /// linear recursion ([`ground::without_linear_recursion`]). Every
    /// layout took less work on the rewrite of the twelve benchmark
    /// programs with linear recursion counted (sickness18_20 under
    /// `families:0` 3.41 M units against 170 k, under `td` 3.66 G against
    /// 203 k; s10 under `closed` 105 k against 21 k). A program that
    /// already reads atoms through indicators is a rewrite, made before
    /// its indicators, and is compiled as it is, under any layout and
    /// under `auto`: a test of a mechanism of the indicators builds one.
    pub(crate) fn eliminated(gp: GroundProgram) -> GroundProgram {
        if gp.encoding.is_empty() {
            ground::without_linear_recursion(gp)
        } else {
            gp
        }
    }

    /// The rest of this layout's rewrite ([`Kind::rewrite`]) of `gp`,
    /// written without its linear recursion: for `decision` its atoms
    /// conditioned on their guards ([`ground::guarded`]), and for
    /// `families` read through indicators ([`indicators::introduce`]). A
    /// program that already reads atoms through indicators is returned as
    /// it is ([`Kind::eliminated`]).
    pub(crate) fn rewrite_eliminated(self, gp: GroundProgram) -> GroundProgram {
        if !gp.encoding.is_empty() {
            return gp;
        }
        let gp = if self == Kind::Decision {
            ground::guarded(gp)
        } else {
            gp
        };
        if self.indicators() {
            indicators::introduce(&gp)
        } else {
            gp
        }
    }

    /// This layout's program ([`Kind::rewrite`]) of `gp`, as grounded, and
    /// its layout ([`Prepared::lay_out`]).
    pub(crate) fn prepare(self, gp: GroundProgram) -> Result<Prepared, Error> {
        let start = Instant::now();
        let ground = self.rewrite(gp);
        Prepared::lay_out(ground, None, self, start.elapsed().as_secs_f64())
    }
}

/// A layout's program and vtree, ready to compile: the program as the
/// layout rewrote it ([`Kind::rewrite`]) and, under `td`, `program` and
/// `decision`, added copies to it, its rules by head and components, the
/// layout over it, and the seconds the rewrite and the layout took.
pub(crate) struct Prepared {
    pub(crate) ground: GroundProgram,
    pub(crate) deps: Arc<Deps>,
    pub(crate) layout: vtree::Layout,
    pub(crate) rewrite_seconds: f64,
    pub(crate) layout_seconds: f64,
}

impl Prepared {
    /// `ground`, the program `kind` compiles ([`Kind::rewrite`]), with its
    /// rules by head and components `deps` where they are known, laid out
    /// by `kind` ([`vtree::build`]); the rewrite took `rewrite_seconds`.
    /// Where every rule reads facts alone, `td`'s fold copies separator
    /// facts; where `program`'s order has narrow cuts, it names the atoms
    /// crossing them; and where atoms outside `decision`'s tree read it,
    /// `decision` names the tree's contexts and the atoms they read. The
    /// program is then `ground` with the copies.
    pub(crate) fn lay_out(
        ground: GroundProgram,
        deps: Option<Arc<Deps>>,
        kind: Kind,
        rewrite_seconds: f64,
    ) -> Result<Prepared, Error> {
        let start = Instant::now();
        let deps = deps.unwrap_or_else(|| Arc::new(Deps::of(&ground)));
        let (layout, copied) = vtree::build(&ground, &deps, kind).map_err(Error::Vtree)?;
        let (ground, deps) = match copied {
            Some(copied) => {
                let deps = Arc::new(Deps::of(&copied));
                (copied, deps)
            }
            None => (ground, deps),
        };
        Ok(Prepared {
            ground,
            deps,
            layout,
            rewrite_seconds,
            layout_seconds: start.elapsed().as_secs_f64(),
        })
    }

    /// The pairs its layout is estimated to hold ([`vtree::Estimate`]).
    pub(crate) fn pairs(&self) -> Option<f64> {
        self.layout.estimate.map(|e| e.pairs)
    }
}
