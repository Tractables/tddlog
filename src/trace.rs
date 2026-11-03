//! A trace of a run ([`Trace`]): each stage the run went through, with what
//! it took in and gave out, in summary. `tddlog --trace FILE` writes it as
//! JSON, and the browser build hands the same JSON to the playground, whose
//! pages draw it step by step.
//!
//! The trace is an observer of [`fn@crate::solve`]'s [`Progress`] events:
//! [`Trace::observe`] turns each into stages, and [`Trace::answered`] or
//! [`Trace::failed`] ends it. It is asked for with [`Options::trace`], under
//! which the compile records each component it compiles
//! ([`crate::tp::CompileTrace`]); without it the compile records nothing.
//! The stages, in the order a run reports them, each a JSON object with
//! its name in `stage`:
//!
//! - `read`: the input's format and size, its text, and the program it
//!   was read as where that is not its own text (a network's, a Dice
//!   program's, an LPAD's, a program given arguments).
//! - `parse`: the program's clauses by kind, its queries and observations.
//! - `ground`: the ground program, its size as the grounder's front end
//!   wrote it and after the passes, the passes that changed it, its
//!   components, and its text.
//! - `note`: a line on the run as a whole (the split of the queries).
//! - `compiling`: the program of a part, as it is handed to the compile.
//! - `rewrite`: the program as the layout that answered compiled it:
//!   without its linear recursion, read through indicators.
//! - `layout`: the vtree (as a tree a page can draw), the layout it came
//!   from and `auto`'s trials.
//! - `compile`: each component in the order it compiled, with the pairs of
//!   each formula an update made and the operations it ran, and the
//!   totals.
//! - `evidence`: the conjunction of the evidence and the sum-outs.
//! - `queries`: a part's answers.
//! - `answers` or `error`: the run's answers in the program's order and
//!   `P(e)`, or why it failed.
//!
//! The lists are bounded so that a large program's trace stays small: a
//! text keeps its first [`LINES`] lines, the compile its first
//! [`tp::trace::COMPONENTS`] components, and the vtree its top
//! [`TREE_LEAVES`] subtrees; what is left out is counted.

use std::fmt::{self, Write};

use crate::ground::{AtomId, GroundProgram, deps};
use crate::input::{Format, Input};
use crate::solve::{Options, Progress, Solution};
use crate::syntax::Program;
use crate::{Error, Run, Strategy, tp};

/// The lines of a text a trace keeps.
pub const LINES: usize = 200;
/// The subtrees a trace's vtree is drawn down to: past them, a subtree is
/// given by its number of leaves.
pub const TREE_LEAVES: usize = 64;
/// The trace's format, which a page checks before it draws one.
pub const VERSION: u32 = 1;

/// A trace of a run, built from its [`Progress`] events.
pub struct Trace {
    stages: Vec<String>,
    /// The part whose events come next, from 0.
    part: usize,
    strategy: Strategy,
    evidence_probability: bool,
    skip_levels: bool,
}

impl Trace {
    /// The trace of a run under `options`, of `input`, read from `data` in
    /// `format`; `options` must ask for it ([`Options::trace`]), so that
    /// the compile records its components.
    pub fn new(
        format: Format,
        data: &[u8],
        input: &Input,
        options: &Options,
    ) -> Result<Trace, Error> {
        if !options.trace {
            return Err(Error::Options(
                "a trace needs the compile's record of its components: Options::trace".into(),
            ));
        }
        let source = String::from_utf8_lossy(data);
        let mut stage = format!(
            "{{\"stage\":\"read\",\"format\":{},\"bytes\":{},\"source\":{}",
            string(&format.to_string()),
            data.len(),
            text_json(&source)
        );
        match &input.program {
            std::borrow::Cow::Owned(program) => {
                let _ = write!(stage, ",\"program\":{}", text_json(program));
            }
            std::borrow::Cow::Borrowed(_) => stage.push_str(",\"program\":null"),
        }
        stage.push_str(",\"notes\":");
        stage.push_str(&strings(&input.left_out));
        stage.push('}');
        Ok(Trace {
            stages: vec![stage],
            part: 0,
            strategy: options.strategy,
            evidence_probability: options.evidence_probability,
            skip_levels: options.skip_levels,
        })
    }

    /// Records what `progress` reports.
    pub fn observe(&mut self, progress: &Progress<'_>) {
        match progress {
            Progress::Parsed { seconds, program } => {
                let stage = parse_json(program, *seconds);
                self.stages.push(stage);
            }
            Progress::Grounded { seconds, ground } => {
                let stage = ground_json(ground, *seconds);
                self.stages.push(stage);
            }
            Progress::Note(line) => {
                self.stages
                    .push(format!("{{\"stage\":\"note\",\"text\":{}}}", string(line)));
            }
            Progress::Compiling(ground) => {
                let stage = format!(
                    "{{\"stage\":\"compiling\",\"part\":{},{}}}",
                    self.part,
                    sizes_json(ground)
                );
                self.stages.push(stage);
            }
            Progress::Answered(run) => {
                self.answered_part(run);
                self.part += 1;
            }
        }
    }

    /// The stages of a part's run: the program it compiled, its layout, its
    /// compile, its evidence and its answers.
    fn answered_part(&mut self, run: &Run) {
        let part = self.part;
        let g = &*run.ground;
        let mut indicated: Vec<AtomId> = g.encoding.indicators.iter().map(|i| i.atom).collect();
        indicated.sort_unstable();
        indicated.dedup();
        let deps = deps::Deps::of(g);
        self.stages.push(format!(
            "{{\"stage\":\"rewrite\",\"part\":{part},\"kind\":{},{},\"components\":{},\"indicators\":{{\"copies\":{},\"atoms\":{},\"links\":{}}},\"weight\":{},\"seconds\":{},\"program\":{}}}",
            string(&run.kind.to_string()),
            sizes_json(g),
            components_json(g, &deps),
            g.encoding.indicators.len(),
            indicated.len(),
            g.encoding.links().len(),
            number(g.weight.to_f64()),
            number(run.rewrite_seconds),
            display_json(g)
        ));
        let names = |v: &tididi::vtree::VarId| -> String {
            run.trace
                .as_ref()
                .and_then(|t| t.variables.get(v.0 as usize - 1))
                .filter(|&&a| a != AtomId::MAX)
                .map_or_else(|| format!("v{}", v.0), |&a| g.atoms[a as usize].to_string())
        };
        self.stages.push(format!(
            "{{\"stage\":\"layout\",\"part\":{part},\"kind\":{},\"strategy\":{},\"attempts\":{},\"seconds\":{},\"discardedSeconds\":{},\"notes\":{},\"vtree\":{}}}",
            string(&run.kind.to_string()),
            string(&self.strategy.to_string()),
            run.attempts,
            number(run.vtree_seconds),
            number(run.wasted_seconds),
            strings(&run.vtree_notes),
            vtree_json(&run.vtree, &names)
        ));
        self.stages.push(compile_json(part, run));
        let q = &run.query_sizes;
        let s = &run.stats;
        self.stages.push(format!(
            "{{\"stage\":\"evidence\",\"part\":{part},\"observations\":{},\"queries\":{},\"pairs\":{},\"largest\":{},\"products\":{},\"countedInCompile\":{},\"templated\":{},\"summingJoins\":{},\"weightedJoins\":{},\"namesKept\":{},\"namesExpanded\":{},\"workUnits\":{},\"seconds\":{},\"probability\":{}}}",
            g.evidence.len(),
            g.num_queries(),
            q.evidence,
            q.largest,
            q.products,
            s.queries_counted,
            s.templated,
            q.summing_joins,
            q.weighted_joins,
            q.names_kept,
            q.names_expanded,
            q.work_units,
            number(run.query_seconds),
            run.evidence_probability
                .map_or_else(|| "null".to_string(), |p| number(p.to_f64()))
        ));
        self.stages.push(format!(
            "{{\"stage\":\"queries\",\"part\":{part},\"answers\":{}}}",
            run.results
                .as_deref()
                .map_or_else(|| "null".to_string(), answers_json)
        ));
    }

    /// Ends the trace with the run's answers, in the order of the program's
    /// queries, and `P(e)` where it was asked for, of `input`.
    pub fn answered(&mut self, solution: &Solution, input: &Input) {
        let evidence = match input.evidence_probability(solution) {
            Some((p, log10)) if self.evidence_probability => format!(
                "{{\"probability\":{},\"log10\":{},\"z\":{}}}",
                number(p),
                number(log10),
                input.log10_offset.is_some()
            ),
            _ => "null".to_string(),
        };
        self.stages.push(format!(
            "{{\"stage\":\"answers\",\"answers\":{},\"evidence\":{evidence}}}",
            solution
                .answers
                .as_deref()
                .map_or_else(|| "null".to_string(), answers_json)
        ));
    }

    /// Ends the trace with the error that ended the run.
    pub fn failed(&mut self, error: &Error) {
        let notes = match error {
            Error::Trials { notes, .. } => strings(notes),
            _ => "[]".to_string(),
        };
        self.stages.push(format!(
            "{{\"stage\":\"error\",\"message\":{},\"notes\":{notes}}}",
            string(&error.to_string())
        ));
    }
}

/// The trace as JSON: `{"trace": VERSION, "tddlog": version, "strategy",
/// "evidenceProbability", "skipLevels", "stages": [...]}`.
impl fmt::Display for Trace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"trace\":{VERSION},\"tddlog\":{},\"strategy\":{},\"evidenceProbability\":{},\"skipLevels\":{},\"stages\":[",
            string(env!("CARGO_PKG_VERSION")),
            string(&self.strategy.to_string()),
            self.evidence_probability,
            self.skip_levels
        )?;
        for (i, stage) in self.stages.iter().enumerate() {
            if i > 0 {
                f.write_str(",\n")?;
            }
            f.write_str(stage)?;
        }
        f.write_str("]}\n")
    }
}

/// The program's clauses by kind: probabilistic facts, annotated
/// disjunctions (of two heads or more), probabilistic rules, facts and
/// rules; its queries and observations.
fn parse_json(program: &Program, seconds: f64) -> String {
    let (mut facts, mut ads, mut prob_rules, mut certain, mut rules) = (0, 0, 0, 0, 0);
    for c in &program.clauses {
        let probabilistic = c.heads.iter().any(|h| h.prob.is_some());
        match (c.heads.len() > 1, probabilistic, c.body.is_empty()) {
            (true, _, _) => ads += 1,
            (false, true, true) => facts += 1,
            (false, true, false) => prob_rules += 1,
            (false, false, true) => certain += 1,
            (false, false, false) => rules += 1,
        }
    }
    format!(
        "{{\"stage\":\"parse\",\"clauses\":{},\"probabilisticFacts\":{facts},\"annotatedDisjunctions\":{ads},\"probabilisticRules\":{prob_rules},\"facts\":{certain},\"rules\":{rules},\"queries\":{},\"evidence\":{},\"seconds\":{}}}",
        program.clauses.len(),
        program.queries.len() + program.query_rules.len(),
        program.evidence.len(),
        number(seconds)
    )
}

/// The ground program: its sizes, as the front end ground it and now, the
/// passes that changed it, its components and its text.
fn ground_json(g: &GroundProgram, seconds: f64) -> String {
    let s = &g.stats;
    let mut passes = Vec::new();
    let mut pass = |name: &str, count: usize| {
        if count > 0 {
            passes.push(format!("{{\"pass\":{},\"count\":{count}}}", string(name)));
        }
    };
    pass("bound end", s.from_bound_end.len());
    pass("levels", s.levels);
    pass("constants", s.constants);
    pass("noisy-or", s.noisy_or);
    pass("values", s.values);
    pass("unfolded", s.unfolded);
    pass("views", s.inlined - s.chained.min(s.inlined));
    pass("chains", s.chained);
    pass("merged", s.merged);
    pass(
        "swept",
        usize::try_from(s.swept.proved).unwrap_or(usize::MAX),
    );
    pass("contexts", s.contexts);
    pass("guarded", s.guarded);
    pass("stepped", s.stepped);
    let c = &s.conditioned;
    pass("evidence facts", c.facts);
    pass("evidence atoms", c.atoms);
    pass("evidence copies", c.merged);
    pass("backbones", c.backbones.map_or(0, |b| b.fixed));
    pass("gates", c.gates);
    pass("renormalized", c.renormalized);
    pass("defined", s.defined.map_or(0, |d| d.eliminated));
    pass("detached", s.detached);
    let (atoms, rules) = s.as_ground;
    format!(
        "{{\"stage\":\"ground\",\"asGround\":{{\"atoms\":{atoms},\"rules\":{rules}}},{},\"boundEnd\":{},\"passes\":[{}],\"components\":{},\"weight\":{},\"seconds\":{},\"program\":{}}}",
        sizes_json(g),
        strings(&s.from_bound_end),
        passes.join(","),
        components_json(g, &deps::Deps::of(g)),
        number(g.weight.to_f64()),
        number(seconds),
        display_json(g)
    )
}

/// A ground program's atoms, facts, derived atoms, rules, queries and
/// observations, as fields of an object.
fn sizes_json(g: &GroundProgram) -> String {
    format!(
        "\"atoms\":{},\"facts\":{},\"derived\":{},\"rules\":{},\"queries\":{},\"observations\":{}",
        g.num_atoms(),
        g.num_facts(),
        g.num_derived(),
        g.rules.len(),
        g.num_queries(),
        g.evidence.len()
    )
}

/// The components of a ground program's derived atoms: how many, how many
/// are cyclic, and the atoms of the largest.
fn components_json(g: &GroundProgram, deps: &deps::Deps) -> String {
    let cyclic = deps
        .sccs
        .iter()
        .filter(|scc| deps::is_cyclic(g, &deps.by_head, scc))
        .count();
    let largest = deps.sccs.iter().map(Vec::len).max().unwrap_or(0);
    format!(
        "{{\"count\":{},\"cyclic\":{cyclic},\"largest\":{largest}}}",
        deps.sccs.len()
    )
}

/// The compile of a part: each component in the order it compiled, those
/// past the first [`tp::trace::COMPONENTS`] together, and the totals.
fn compile_json(part: usize, run: &Run) -> String {
    let g = &*run.ground;
    let s = &run.stats;
    let name = |a: AtomId| string(&g.atoms[a as usize].to_string());
    let mut out = format!(
        "{{\"stage\":\"compile\",\"part\":{part},\"formulation\":{},\"projection\":{},\"components\":[",
        string(&run.kind.formulation().to_string()),
        string(&run.kind.projection(tp::Projection::default()).to_string())
    );
    let trace = run.trace.as_deref();
    for (i, c) in trace.map_or(&[][..], |t| &t.components).iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let atoms: Vec<String> = c.atoms.iter().map(|&a| name(a)).collect();
        let sizes: Vec<String> = c
            .sizes
            .iter()
            .map(|&(a, pairs, round)| {
                format!(
                    "[{},{},{round}]",
                    name(a),
                    pairs.map_or_else(|| "null".to_string(), |p| p.to_string())
                )
            })
            .collect();
        let _ = write!(
            out,
            "{{\"atoms\":[{}],\"size\":{},\"cyclic\":{},\"negation\":{},\"rounds\":{},\"updates\":{},\"largest\":{},\"sizes\":[{}],\"operations\":{},\"workUnits\":{}}}",
            atoms.join(","),
            c.size,
            c.cyclic,
            c.negation,
            c.rounds,
            c.updates,
            c.largest,
            sizes.join(","),
            operations_json(&c.operations),
            c.work_units
        );
    }
    let r = trace.map(|t| t.rest).unwrap_or_default();
    let _ = write!(
        out,
        "],\"rest\":{{\"components\":{},\"cyclic\":{},\"updates\":{},\"largest\":{},\"operations\":{},\"workUnits\":{}}},\"totals\":{{\"components\":{},\"cyclic\":{},\"updates\":{},\"iterations\":{},\"largest\":{},\"final\":{},\"operations\":{},\"workUnits\":{}}},\"seconds\":{}}}",
        r.components,
        r.cyclic,
        r.updates,
        r.largest,
        operations_json(&r.operations),
        r.work_units,
        s.components,
        s.cyclic_components,
        s.atom_updates,
        s.iterations,
        s.max_pairs,
        s.final_max_pairs,
        operations_json(&s.op_counts),
        s.work_units,
        number(s.seconds)
    );
    out
}

/// The operations by kind, in the order the compile counts them.
fn operations_json(ops: &[usize; 5]) -> String {
    let [and, or, minimize, equivalent, negate] = ops;
    format!(
        "{{\"and\":{and},\"or\":{or},\"minimize\":{minimize},\"equivalent\":{equivalent},\"negate\":{negate}}}"
    )
}

/// The vtree as a tree a page can draw: an inner node as the array of its
/// two children, a leaf as its fact's name (`names`), and a subtree below
/// the top [`TREE_LEAVES`] as `{"leaves": n}`; with its variables and
/// depth.
fn vtree_json(vtree: &tididi::Vtree, names: &dyn Fn(&tididi::vtree::VarId) -> String) -> String {
    use tididi::vtree::VtreeIdx;
    // The leaves and the depth below each node, children before parents.
    let n = vtree.num_nodes();
    let (mut leaves, mut depth) = (vec![0usize; n], vec![0usize; n]);
    for idx in vtree.bottomup() {
        let i = idx.idx();
        if vtree.node(idx).is_leaf() {
            leaves[i] = 1;
        } else {
            let (l, r) = vtree.children(idx);
            leaves[i] = leaves[l.idx()] + leaves[r.idx()];
            depth[i] = 1 + depth[l.idx()].max(depth[r.idx()]);
        }
    }
    // The subtrees drawn: from the root, the one with the most leaves is
    // opened until there are TREE_LEAVES of them.
    let root = vtree.root();
    let mut open = vec![false; n];
    let mut frontier = vec![root];
    while frontier.len() < TREE_LEAVES {
        let Some(k) = (0..frontier.len())
            .filter(|&k| !vtree.node(frontier[k]).is_leaf())
            .max_by_key(|&k| (leaves[frontier[k].idx()], std::cmp::Reverse(k)))
        else {
            break;
        };
        let idx = frontier.swap_remove(k);
        open[idx.idx()] = true;
        let (l, r) = vtree.children(idx);
        frontier.extend([l, r]);
    }
    fn draw(
        vtree: &tididi::Vtree,
        idx: VtreeIdx,
        open: &[bool],
        leaves: &[usize],
        names: &dyn Fn(&tididi::vtree::VarId) -> String,
        out: &mut String,
    ) {
        if vtree.node(idx).is_leaf() {
            out.push_str(&string(&names(&vtree.leaf_var(idx))));
        } else if open[idx.idx()] {
            let (l, r) = vtree.children(idx);
            out.push('[');
            draw(vtree, l, open, leaves, names, out);
            out.push(',');
            draw(vtree, r, open, leaves, names, out);
            out.push(']');
        } else {
            let _ = write!(out, "{{\"leaves\":{}}}", leaves[idx.idx()]);
        }
    }
    let mut tree = String::new();
    draw(vtree, root, &open, &leaves, names, &mut tree);
    format!(
        "{{\"variables\":{},\"depth\":{},\"tree\":{tree}}}",
        vtree.num_leaves(),
        depth[root.idx()]
    )
}

/// Answers as `[{"query", "probability"}]`.
fn answers_json(answers: &[crate::QueryResult]) -> String {
    let items: Vec<String> = answers
        .iter()
        .map(|r| {
            format!(
                "{{\"query\":{},\"probability\":{}}}",
                string(&r.to_string()),
                number(r.probability)
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

/// A text as `{"text", "lines", "shown"}`: its first [`LINES`] lines, and
/// how many it has.
fn text_json(text: &str) -> String {
    let lines = text.lines().count();
    let shown: Vec<&str> = text.lines().take(LINES).collect();
    format!(
        "{{\"text\":{},\"lines\":{lines},\"shown\":{}}}",
        string(&shown.join("\n")),
        shown.len()
    )
}

/// What `value` prints as `{"text", "lines", "shown"}`, written only up to
/// its first [`LINES`] lines: `lines` is then `null`, the number not known.
fn display_json(value: &dyn fmt::Display) -> String {
    /// A writer that refuses a line past [`LINES`].
    struct Bounded {
        text: String,
        lines: usize,
        full: bool,
    }
    impl Write for Bounded {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            for piece in s.split_inclusive('\n') {
                if self.lines == LINES {
                    self.full = true;
                    return Err(fmt::Error);
                }
                self.text.push_str(piece);
                if piece.ends_with('\n') {
                    self.lines += 1;
                }
            }
            Ok(())
        }
    }
    let mut out = Bounded {
        text: String::new(),
        lines: 0,
        full: false,
    };
    let _ = write!(out, "{value}");
    let text = out.text.trim_end_matches('\n');
    let lines = if out.full {
        "null".to_string()
    } else {
        text.lines().count().to_string()
    };
    format!(
        "{{\"text\":{},\"lines\":{lines},\"shown\":{}}}",
        string(text),
        text.lines().count()
    )
}

/// Strings as a JSON array.
fn strings(items: &[String]) -> String {
    let items: Vec<String> = items.iter().map(|s| string(s)).collect();
    format!("[{}]", items.join(","))
}

/// A JSON number; `null` for a value JSON cannot hold, as the logarithm of
/// a zero `P(e)`. The browser build's result is written with it too.
pub fn number(x: f64) -> String {
    if x.is_finite() {
        format!("{x:?}")
    } else {
        "null".to_string()
    }
}

/// A JSON string. The browser build's result is written with it too.
pub fn string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_and_numbers_are_json() {
        assert_eq!(string("a\"b\\c\n\u{1}"), "\"a\\\"b\\\\c\\n\\u0001\"");
        assert_eq!(number(f64::NEG_INFINITY), "null");
        assert_eq!(number(f64::NAN), "null");
        assert_eq!(number(1e-300), "1e-300");
        assert_eq!(number(0.25), "0.25");
    }

    #[test]
    fn a_text_keeps_its_first_lines_and_counts_them_all() {
        let text: String = (0..LINES + 5).map(|i| format!("l{i}\n")).collect();
        let last = format!("l{}", LINES - 1);
        let read = text_json(&text);
        assert!(read.ends_with(&format!("\"lines\":{},\"shown\":{LINES}}}", LINES + 5)));
        assert!(read.contains(&format!("{last}\"")), "{read}");
        // What a value prints is written only up to the lines kept, so the
        // number of lines past them is not known.
        let shown = display_json(&text);
        assert!(
            shown.ends_with(&format!("\"lines\":null,\"shown\":{LINES}}}")),
            "{shown}"
        );
        assert!(shown.contains(&format!("{last}\"")), "{shown}");
        let short = display_json(&"a\nb\n");
        assert_eq!(short, "{\"text\":\"a\\nb\",\"lines\":2,\"shown\":2}");
    }
}
