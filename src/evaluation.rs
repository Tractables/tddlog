//! A program's compile and query phase on one vtree ([`Evaluation`]),
//! which a run of one layout calls once and `auto` once per attempt of a
//! trial.

use crate::ground::{self, GroundProgram};
use crate::tp::{self, CompileStats, Projection};
use crate::vtree::Kind;
use crate::{QueryResult, eval, query};

/// What a finished query phase gives: the answers, `None` where the
/// evidence is inconsistent, and what the phase built.
pub(crate) type Answers = (Option<Vec<QueryResult>>, query::QuerySizes);

/// A program's compile and query phase on one vtree. A call that stops,
/// at its watch or at the engine's limits, keeps what it built: the
/// components finished, or the evidence conjunctions and answers
/// completed, and the next call goes on from there. A run of one layout
/// makes one call without a watch (`crate::solve`), `auto` one per attempt
/// of a trial (`src/auto/`), without a watch only for a trial left alone,
/// which an error retires. A call without a watch is the last: the last
/// reads of the formulas in its compile and its evidence walk take them
/// rather than copying them, and it cannot go on after an error.
#[derive(Default)]
pub(crate) struct Evaluation {
    /// The compile under way, between its start and its finish.
    pub(crate) compilation: Option<tp::Compilation>,
    pub(crate) compiled: Option<tp::Compiled>,
    query: Option<query::QueryEvaluation>,
    /// Seconds in the query phase, over every call.
    pub(crate) query_seconds: f64,
    /// Work units in the query phase, over every call since it last
    /// began ([`Self::walk_again`]), but for what settling spent within its
    /// budget ([`Self::settled_work`]): the work `auto` charges a trial's
    /// turns for it.
    pub(crate) query_work: u64,
    /// Whether the query phase started over with a walk that puts nothing
    /// off ([`Self::walk_again`]).
    walked_again: bool,
}

impl Evaluation {
    /// Whether the compile has finished, and only the query phase is left.
    pub(crate) fn is_compiled(&self) -> bool {
        self.compiled.is_some()
    }

    /// The share of its phase finished, as the steps done and all of
    /// them: the compile's components, or the query phase's joins and
    /// queries ([`query::QueryEvaluation::progress`]). `None` before either
    /// has started.
    pub(crate) fn progress(&self) -> Option<(usize, usize)> {
        match (&self.compilation, &self.query) {
            (Some(c), _) => Some(c.progress()),
            (None, Some(q)) => Some(q.progress()),
            (None, None) => None,
        }
    }

    /// The components the compile had finished, and all of them, where its
    /// last run passed half the trial's target ([`tp::Horizon`]); `None`
    /// where it did not, or the compile has finished.
    pub(crate) fn marked(&self) -> Option<(usize, usize)> {
        self.compilation.as_ref().and_then(|c| c.marked)
    }

    /// The work units that settling the names of `program` and `decision`
    /// has spent within its budget, over every call, which no turn is
    /// charged for (`tp::settle`, `Settled::free`).
    pub(crate) fn settled_work(&self) -> u64 {
        self.compiled
            .as_ref()
            .map_or(0, |c| c.for_queries.settled.free(c.stats.work_units))
    }

    /// `P(e)` of `gp`, whose query phase has answered
    /// ([`crate::Run::evidence_probability`]).
    pub(crate) fn evidence_probability(&self, gp: &GroundProgram) -> Option<eval::Count> {
        let mass = self
            .query
            .as_ref()
            .expect("the query phase has answered")
            .evidence_mass();
        (gp.stats.detached == 0).then(|| (gp.weight * mass).normalized())
    }

    /// Compile `gp`, whose rules by head and components are `deps`, its
    /// facts' variables in `order`, on `backend` under `kind`'s
    /// formulation and projection ([`Kind::formulation`],
    /// [`Kind::projection`] of `projection`), then answer its queries, each
    /// phase going on from where the last call stopped. `watch` bounds the
    /// pairs of the formulas ([`tp::Compilation::run`]) and of what the
    /// query phase builds, and `horizon` where the compile stops on its
    /// projected finish ([`tp::Horizon`]). A dynamic layout searches first at `first_pairs`
    /// live pairs; the vtree the search reached is `backend`'s, and the
    /// next call must build on it.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn run(
        &mut self,
        gp: &GroundProgram,
        deps: &ground::deps::Deps,
        order: &[ground::AtomId],
        backend: &tp::TddBackend,
        kind: Kind,
        projection: Projection,
        first_pairs: usize,
        watch: Option<usize>,
        horizon: Option<tp::Horizon>,
    ) -> Result<Answers, tp::CompileError> {
        if self.compiled.is_none() {
            if self.compilation.is_none() {
                let var_of = tp::fact_variables(gp, order);
                let compilation = tp::Compilation::new(
                    gp,
                    deps,
                    var_of,
                    kind.formulation(),
                    backend,
                    kind.projection(projection),
                )?;
                self.compilation = Some(if kind.is_dynamic() {
                    compilation.dynamic(first_pairs)
                } else {
                    compilation
                });
            }
            let compiling = self.compilation.as_mut().unwrap();
            compiling.horizon = horizon;
            compiling.run(gp, backend, watch)?;
            self.compiled = Some(compiling.finish(backend)?);
            self.compilation = None;
        }
        let start = std::time::Instant::now();
        let work = backend.work_units();
        let settled = self.settled_work();
        let result = self.query_phase(gp, backend, watch, settled);
        self.query_seconds += start.elapsed().as_secs_f64();
        self.query_work +=
            (backend.work_units() - work).saturating_sub(self.settled_work() - settled);
        let (results, mut sizes) = result?;
        // Under a watch the phase may stop and go on in a later call, and
        // its work units are not added up across calls.
        if watch.is_none() {
            sizes.work_units = backend.work_units() - work;
        }
        Ok((results, sizes))
    }

    /// The query phase of [`Self::run`], its compile finished, from where
    /// the last call left it. The layout's names are settled once, in the
    /// phase's work (`tp::settle`), and the run's stop moves later by what
    /// settling spent within its budget, `settled` before the call, which
    /// `auto` does not charge to the trial's turn.
    fn query_phase(
        &mut self,
        gp: &GroundProgram,
        backend: &tp::TddBackend,
        watch: Option<usize>,
        settled: u64,
    ) -> Result<Answers, tp::CompileError> {
        if self.query.is_none() {
            tp::settle(gp, backend, self.compiled.as_mut().unwrap())?;
            backend.kept();
        }
        let _later = backend.stop_later(self.settled_work() - settled);
        let compiled = self.compiled.as_mut().unwrap();
        let query = self.query.get_or_insert_with(|| {
            let query = query::QueryEvaluation::new(gp, backend, compiled);
            if self.walked_again {
                query.putting_nothing_off()
            } else {
                query
            }
        });
        if watch.is_none() {
            query.take_last_reads(gp, compiled);
        }
        query.run(gp, backend, compiled, watch)
    }

    /// Starts the query phase over, once, with an evidence walk that puts
    /// nothing off, where its walk sums out, has put items off and holds
    /// no formula taken from the compile (`query::walk`'s module doc):
    /// `auto` does so for a trial whose walk was refused memory or outgrew
    /// an index. What the walk built is dropped. Whether it did.
    pub(crate) fn walk_again(&mut self) -> bool {
        if !self.query.as_ref().is_some_and(|q| q.walks_again()) {
            return false;
        }
        self.query = None;
        self.query_work = 0;
        self.walked_again = true;
        true
    }

    /// The finished compile's statistics.
    pub(crate) fn into_stats(self) -> CompileStats {
        self.compiled.expect("the compile finished").stats
    }
}

#[cfg(test)]
mod tests {
    use super::Evaluation;
    use crate::GroundOptions;
    use crate::tp::{CompileError, Projection, TddBackend};
    use crate::vtree::Kind;

    #[test]
    fn a_summing_walk_starts_over_putting_nothing_off() {
        // Two naive Bayes models of 40 features, each with its first half
        // observed and no query, so that only P(e) is wanted and the walk
        // sums out. On `closed`'s right-linear layout it puts observations
        // along the spine off. Stopped by the watch once it has, as `auto`
        // stops it where it is refused memory, it starts over putting
        // nothing off, once, and answers the same P(e), hand computed from
        // the program's rounded weights: each model's P(e) is 0.5 times
        // the product of the observed p_i plus 0.5 times that of the q_i,
        // and the models are independent.
        let k = 40;
        let weight = |x: f64| format!("{x:.4}").parse::<f64>().unwrap();
        let p = |i: usize| weight(0.1 + 0.8 * (i as f64 / k as f64));
        let q = |i: usize| weight(0.9 - 0.8 * (i as f64 / k as f64));
        let mut bayes = String::new();
        for m in ["a", "b"] {
            bayes.push_str(&format!("0.5::cls({m},v1); 0.5::cls({m},v2).\n"));
            for i in 1..=k / 2 {
                bayes.push_str(&format!(
                    "{}::obs({m},{i}) :- cls({m},v1).\n{}::obs({m},{i}) :- cls({m},v2).\n",
                    p(i),
                    q(i)
                ));
                bayes.push_str(&format!("evidence(obs({m},{i}), true).\n"));
            }
        }
        let (v1, v2) = (1..=k / 2).fold((0.5, 0.5), |(a, b), i| (a * p(i), b * q(i)));
        let want = 2.0 * (v1 + v2).log10();
        let options = GroundOptions {
            detach: false,
            ..GroundOptions::default()
        };
        let gp = crate::ground::ground(&crate::syntax::parse_program(&bayes).unwrap(), &options)
            .unwrap();
        let deps = crate::ground::deps::Deps::of(&gp);
        let (layout, _) = crate::vtree::build(&gp, &deps, Kind::Closed).unwrap();
        let tree = layout.vtree.clone();
        let mut evaluation = Evaluation::default();
        let run = |evaluation: &mut Evaluation, watch| {
            tree.context().run(|engine| {
                let backend = TddBackend::new(engine, tree.clone());
                evaluation
                    .run(
                        &gp,
                        &deps,
                        &layout.order,
                        &backend,
                        Kind::Closed,
                        Projection::default(),
                        1,
                        watch,
                        None,
                    )
                    .map(|_| ())
            })
        };
        let mut watch = 1;
        loop {
            match run(&mut evaluation, Some(watch)) {
                Err(CompileError::Oversize { .. } | CompileError::QueryOversize { .. }) => {}
                other => panic!("at a watch of {watch} pairs: {other:?}"),
            }
            if evaluation.walk_again() {
                break;
            }
            watch *= 2;
        }
        run(&mut evaluation, None).unwrap();
        assert!(!evaluation.walk_again(), "the walk put items off again");
        let got = evaluation
            .evidence_probability(&gp)
            .expect("nothing detached")
            .log10();
        assert!((got - want).abs() < 1e-9, "log10 P(e) {got} against {want}");
    }
}
