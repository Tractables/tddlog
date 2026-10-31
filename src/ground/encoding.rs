//! What a program rewritten by [`crate::indicators`] records of the
//! variables it reads derived atoms through ([`Encoding`]), and the sizes of
//! exclusive sets from which the rewrite reads them otherwise, which the
//! grounder's passes that shape a program for it read too.

use rustc_hash::{FxHashMap, FxHashSet};

use super::AtomId;

/// How the program reads derived atoms through variables of their own
/// ([`crate::indicators`]); empty for a program as grounded.
#[derive(Clone, Debug, Default)]
pub struct Encoding {
    /// The derived atoms that other components read through a variable of
    /// their own, one entry per copy.
    pub indicators: Vec<Indicator>,
    /// The sets of values that other components read in binary.
    pub codes: Vec<Code>,
    /// Groups of copies of which the links admit at most one true: the
    /// copies of one exclusive set's atoms that one unit and part read,
    /// each ascending.
    pub siblings: Vec<Vec<AtomId>>,
    /// For names along an order ([`crate::vtree`]'s `program`), the segment
    /// that reads each copy, one entry per indicator; for `decision`'s
    /// names, a segment for each node of its tree with the names of the
    /// atoms read outside the tree above it, and `usize::MAX` for a
    /// context's copy, which is in none; empty for the families'
    /// indicators.
    pub(crate) segments: Vec<usize>,
}

/// A derived atom read through an indicator variable: `var` is a fact atom
/// of weight one on both sides that the atom's readers read in its place,
/// and `link` a derived atom true exactly when `var` agrees with the atom,
/// or with another indicator of it, which the program observes true; or,
/// for a table as clauses ([`crate::indicators::introduce`]), true exactly
/// when a clause of the table fails, which the program observes false.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Indicator {
    /// The derived atom.
    pub atom: AtomId,
    /// The fact its readers read in its place.
    pub var: AtomId,
    /// The derived atom that ties `var` to it.
    pub link: AtomId,
}

/// An exclusive set of derived atoms read in binary: value `values[i]` has
/// the code `i + 1`, 0 being none of them, and `bits[j]` is a derived atom
/// true exactly when the code has bit j, which readers read through
/// indicators of their own in place of the values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Code {
    pub(crate) values: Vec<AtomId>,
    pub(crate) bits: Vec<AtomId>,
}

impl Encoding {
    /// Whether the program reads no atom through an indicator, as a program
    /// as grounded does.
    pub fn is_empty(&self) -> bool {
        self.indicators.is_empty()
    }

    /// The indicators' links, each once.
    pub fn links(&self) -> FxHashSet<AtomId> {
        self.indicators.iter().map(|ind| ind.link).collect()
    }

    /// Per atom read through indicators, the variable of its first one.
    pub(crate) fn first_copies(&self) -> FxHashMap<AtomId, AtomId> {
        let mut first = FxHashMap::default();
        for ind in &self.indicators {
            first.entry(ind.atom).or_insert(ind.var);
        }
        first
    }
}

/// The values from which a table split on a parent reads, in each part,
/// the parent's copy of the part's value alone. The other values' copies
/// negated tie every part to every copy, so the `families` layout keeps
/// the copies below all the parts, and the parts on a spine above them:
/// each node of the spine has a class for every part still to come. Read
/// alone, a part's copy is its own and the parts go side by side, each
/// with its copy ([`crate::vtree`]). The negations are what tells a
/// conjunction of the walk that no two values hold at once before the
/// parent's links reach it, and without them a parent of few values costs
/// the walk more (bn_hailfinder).
///
/// Only a unit of which at most one table is read from outside, by an
/// observation or through another unit's copies, reads its values so.
/// Where several are, their observations and their readers' links are as
/// many formulas over the same copies, which the walk conjoins before the
/// links of the copies: each pair of values that could hold together
/// leaves a class per subset of those formulas satisfied.
///
/// Nor does a table whose parts share a switch, one fact read under
/// contexts that exclude each other (as switch sharing makes
/// them): the rows of two parts read it under contexts that no longer
/// exclude each other, and the parts fall into one block above every copy.
///
/// Nor does a table of this many values or more that another unit reads
/// through copies of its own. The readers' links to its values meet before
/// anything says that its own parts' copies exclude each other, and leave
/// a class for each set of its values that could hold together
/// (bn_mildew's `dm_2`, `dm_3` and `dm_4` are each split on the one
/// before, and `udbytte` reads `dm_4`).
///
/// Nor does a part split again (`indicators::table_splits`): only the
/// parts of a whole unit read their value alone; below another split,
/// reading alone ran bn_pathfinder out of memory.
///
/// The row classes (`ground::classes`) read it too, to keep the tables
/// whose classes the rewrite would read so.
pub(crate) const OWN_VALUE_FROM: usize = 16;

/// The fewest values of a set read in binary (`indicators::Bits`). From 32
/// values, bits cut the work of bn_mildew's and bn_pathfinder's compile
/// and query phase and leave bn_diabetes and bn_barley as they are. From
/// fewer they cost bn_diabetes and bn_barley work: the row classes of each
/// of bn_diabetes's `cho_t` ([`crate::ground::classes`]), 31 of them, are
/// read by one unit, whose link as clauses is one factor that the evidence
/// walk joins more cheaply than the five links of the bits. The
/// conditioning's renormalization reads it too: a table whose contexts
/// read such a set keeps its masses in its own unit (`ground::condition`,
/// `renormalize`).
pub(crate) const BITS_FROM: usize = 32;
