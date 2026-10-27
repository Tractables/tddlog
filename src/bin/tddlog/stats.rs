//! The `--stats` lines on a run: the ground program, the indicators, the
//! compile, the projection, the vtree searches' notes, and the evidence and
//! queries.

use tddlog::{GroundProgram, Projection, Strategy};

use super::Args;

/// The `--stats` lines: the ground program, the indicators, the compile,
/// the projection, the vtree searches' notes, and the evidence and queries.
pub(super) fn print_stats(args: &Args, run: &tddlog::Run, ground_secs: f64) {
    let g = &run.ground;
    eprintln!("{}", ground_line(g, ground_secs));
    // `program` names the atoms that cross a narrow cut of its order.
    let policy = if run.kind.indicators() || g.encoding.is_empty() {
        run.kind.indicators_name()
    } else {
        "named"
    };
    let rewrite_secs = run.rewrite_seconds;
    let mut indicated: Vec<_> = g.encoding.indicators.iter().map(|i| i.atom).collect();
    indicated.sort_unstable();
    indicated.dedup();
    eprintln!(
        "indicators: {policy}, {} copies of {} atoms, {rewrite_secs:.3}s",
        g.encoding.indicators.len(),
        indicated.len()
    );
    if run.kind.indicators() {
        eprintln!(
            "tables: {} of {} links hold clauses",
            tddlog::indicators::clause_links(g).len(),
            g.encoding.links().len()
        );
    }
    let s = &run.stats;
    let strategy = match args.strategy {
        Strategy::Trial(_) => run.kind.to_string(),
        Strategy::Auto => format!(
            "{} (from {}: {} attempts, {:.3}s discarded)",
            run.kind, args.strategy, run.attempts, run.wasted_seconds
        ),
    };
    // A dynamic layout, also as auto's trial, compiles unprojected.
    let projection = run.kind.projection(Projection::default());
    eprintln!(
        "compile ({}, projection {}): vtree {} over {} vars in {:.3}s; {} components ({} cyclic), {} atom updates, {} iterations (max {}), largest formula {} pairs (final {}), {:.3}s",
        run.kind.formulation(),
        projection,
        strategy,
        run.vtree.num_leaves(),
        run.vtree_seconds,
        s.components,
        s.cyclic_components,
        s.atom_updates,
        s.iterations,
        s.max_iterations,
        s.max_pairs,
        s.final_max_pairs,
        s.seconds
    );
    let [n_and, n_or, n_min, n_eq, n_neg] = s.op_counts;
    eprintln!(
        "compile operations: and {n_and}, or {n_or}, negate {n_neg}, minimize {n_min}, equivalent {n_eq} of {} tests; {} formulas taken by their last reader",
        s.convergence_tests, s.taken
    );
    eprintln!(
        "compile negations: {}; {} of the compile's {} work units",
        negations_by_source(&s.negations),
        s.negations.total().work_units,
        s.work_units
    );
    let m = &s.clause_memo;
    eprintln!(
        "compile clause memo: {} of {} clause loops copied, {} kept loops of another shape passed over, {} kept ({} pairs); {} literals left out, {} loops held their sibling groups; {} built level by level ({} from their models), {} fell back, {} kept after the watch refused them ({} taken again); {} bodies by the or-route ({} of them after a product given up); {} readers' links tied to another's copies",
        m.copies,
        m.lookups,
        m.passed,
        m.kept,
        m.pairs,
        m.left_out,
        m.seeded,
        m.built,
        m.from_models,
        m.fell_back,
        m.refused,
        m.retaken,
        m.or_routes,
        m.given_up,
        s.tied
    );
    if let Some(sr) = &s.search {
        eprintln!(
            "compile search: {} searches ({} gained under a tenth), {} probes, {} kept, {} pairs to {}, {} work units, {} restarts ({} kept)",
            sr.searches,
            sr.weak,
            sr.probes,
            sr.accepts,
            sr.pairs_before,
            sr.pairs_after,
            sr.work_units,
            sr.restarts,
            sr.restarts_kept
        );
    }
    if let Some(pr) = &s.projection {
        let cone_cut = match projection {
            Projection::Cones { max_share } => {
                ((max_share * f64::from(pr.facts)).ceil() as u32).max(1)
            }
            Projection::Off => 0,
        };
        if pr.projected == 0 {
            eprintln!(
                "projection: every one of the {} components has a cone of {} facts or more, so all of them kept the global vtree",
                s.components, cone_cut
            );
        } else {
            eprintln!(
                "projection: {} of {} components on {} projected vtrees, largest kept cone {} of {} facts, {} narrowed to their support",
                pr.projected, s.components, pr.scopes, pr.largest_kept, pr.facts, pr.narrowed
            );
        }
    }
    for note in &run.vtree_notes {
        eprintln!("{note}");
    }
    let q = &run.query_sizes;
    eprintln!(
        "evidence and queries: {:.3}s, evidence {} pairs, largest formula {} pairs; walk {:.3}s, counting the evidence {:.3}s, {} query products {:.3}s, {} queries counted in the compile, {} through a template, {} recounts, {} pairs counted as grids",
        run.query_seconds,
        q.evidence,
        q.largest,
        q.walk_seconds,
        q.count_seconds,
        q.products,
        q.product_seconds,
        s.queries_counted,
        s.templated,
        q.recounts,
        q.grid_pairs
    );
    if q.names_kept + q.names_expanded > 0 {
        eprintln!(
            "names: kept at {} cuts, expanded at {}, {} work units",
            q.names_kept, q.names_expanded, q.names_work
        );
    }
    let r = q.routes;
    if r != tddlog::eval::Routes::default() {
        eprintln!(
            "count routes, pairs up/down: grid kernels {}/{}, grid loops in floats {}/{}, grid loops in Counts {}/{}, grid walks in floats {}/{}, grid walks in Counts {}/{}, pairs in floats {}/{}, pairs in Counts {}/{}",
            r.up[0],
            r.down[0],
            r.up[1],
            r.down[1],
            r.up[2],
            r.down[2],
            r.up[3],
            r.down[3],
            r.up[4],
            r.down[4],
            r.up[5],
            r.down[5],
            r.up[6],
            r.down[6]
        );
    }
    if q.work_units > 0 {
        eprintln!(
            "query negations: {}; {} of the query phase's {} work units",
            negations_by_source(&q.negations),
            q.negations.total().work_units,
            q.work_units
        );
    } else if q.negations.total().calls > 0 {
        eprintln!("query negations: {}", negations_by_source(&q.negations));
    }
    eprintln!("{}", implicit_line(q));
}

/// The `--stats` line on implicit levels: the pairs held as their
/// description, those a prune kept described and those a renumbering or a
/// change left stored; the evidence's described pairs and those its count
/// read as grids.
fn implicit_line(q: &tddlog::query::QuerySizes) -> String {
    format!(
        "implicit levels: {} pairs described, {} kept described by prunes, {} stored by renumberings and changes; evidence {} of {} pairs, {} counted as grids",
        tididi::diagram::described(),
        tididi::diagram::redescribed(),
        tididi::diagram::stored_moved(),
        q.evidence_implicit,
        q.evidence,
        q.grid_pairs
    )
}

/// The `--stats` line on the ground program.
pub(super) fn ground_line(g: &GroundProgram, secs: f64) -> String {
    let mut line = format!(
        "ground: {} atoms ({} facts, {} derived), {} rules, {secs:.3}s",
        g.num_atoms(),
        g.num_facts(),
        g.num_derived(),
        g.rules.len()
    );
    if !g.stats.from_bound_end.is_empty() {
        line.push_str(&format!(
            "; {} grounded from the bound end",
            g.stats.from_bound_end.join(", ")
        ));
    }
    if g.stats.levels > 0 {
        line.push_str(&format!(
            "; grounded level by level, {} keys, {} atoms proved constant",
            g.stats.levels, g.stats.constants
        ));
    }
    if g.stats.noisy_or > 0 {
        line.push_str(&format!(
            "; {} tables written as noisy-ors",
            g.stats.noisy_or
        ));
    }
    if g.stats.inlined > 0 {
        line.push_str(&format!("; {} views inlined", g.stats.inlined));
    }
    if g.stats.chained > 0 {
        line.push_str(&format!("; {} atoms of chains inlined", g.stats.chained));
    }
    if g.stats.merged > 0 {
        line.push_str(&format!(
            "; {} atoms merged into equal ones",
            g.stats.merged
        ));
    }
    let s = g.stats.swept;
    if s.calls > 0 {
        line.push_str(&format!(
            "; the sweep proved {} gates equal to earlier ones or constant in {} calls ({} told apart, {} left open, {} steps)",
            s.proved, s.calls, s.refuted, s.open, s.steps
        ));
    }
    if g.stats.contexts > 0 {
        line.push_str(&format!(
            "; {} facts read in place of choices, one per context",
            g.stats.contexts
        ));
    }
    if g.stats.guarded > 0 {
        line.push_str(&format!(
            "; {} atoms conditioned on their guards",
            g.stats.guarded
        ));
    }
    if g.stats.stepped > 0 {
        line.push_str(&format!(
            "; {} atoms composing chains of steps",
            g.stats.stepped
        ));
    }
    let c = g.stats.conditioned;
    if c.facts + c.atoms + c.merged + c.into > 0 {
        line.push_str(&format!(
            "; the evidence fixed {} facts and {} derived atoms, and made {} facts copies of {}",
            c.facts, c.atoms, c.merged, c.into
        ));
    }
    if let Some(b) = c.backbones {
        let s = b.search;
        line.push_str(&format!(
            "; the SAT search fixed {} facts more, {} backbones and {} equalities proved in {} calls ({} conflicts, {} propagation steps), {} candidates left open",
            b.fixed, s.backbones, s.equivalences, s.calls, s.conflicts, s.ticks, s.open
        ));
    }
    if c.gates > 0 {
        line.push_str(&format!(
            "; recovered {} gates from the clauses, read nowhere else and dropped",
            c.gates
        ));
    }
    if c.renormalized > 0 {
        line.push_str(&format!(
            "; {} rows renormalized over the heads the evidence leaves",
            c.renormalized
        ));
    }
    if let Some(d) = g.stats.defined {
        let s = d.support;
        line.push_str(&format!(
            "; the independent support left out {} of {} facts of probability 1/2 in {} calls ({} undecided, {} untried, {} conflicts, {} propagation steps), {} eliminated by resolution, {} clauses to {}",
            s.defined, d.candidates, s.tried, s.unknown, s.untried, s.conflicts, s.ticks, d.eliminated, d.clauses, d.left
        ));
        for (i, when) in [
            (d.implied, "before the support"),
            (d.implied_after, "after the elimination"),
        ] {
            if i.tried > 0 {
                line.push_str(&format!(
                    ", {} of {} clauses tried {when} implied by the others ({} searched, {} unsearched, {} passed over as joined by others, {} conflicts, {} propagation steps)",
                    i.implied, i.tried, i.searched, i.untried, i.joined, i.conflicts, i.ticks
                ));
            }
        }
    }
    if g.stats.detached > 0 {
        line.push_str(&format!(
            "; {} observations no query depends on dropped",
            g.stats.detached
        ));
    }
    line
}

/// The sources that negated anything: calls, operand pairs and work units.
fn negations_by_source(n: &tddlog::tp::Negations) -> String {
    let sources = tddlog::tp::NegationSource::ALL
        .iter()
        .filter_map(|&source| {
            let c = n.of(source);
            (c.calls > 0).then(|| {
                format!(
                    "{} {} ({} pairs, {} work)",
                    source.name(),
                    c.calls,
                    c.pairs,
                    c.work_units
                )
            })
        })
        .collect::<Vec<_>>();
    if sources.is_empty() {
        "none".to_string()
    } else {
        sources.join(", ")
    }
}
