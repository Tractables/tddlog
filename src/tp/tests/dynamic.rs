//! A dynamic vtree changes how the formulas are represented, never what they
//! are: every example program answers the same with a search at every step
//! of its compile as on the vtree the search starts from, which
//! `tests/it/examples.rs` checks against ProbLog.

use super::*;
use crate::ground::deps::Deps;
use crate::ground::{GroundOptions, ground};
use crate::query::{QueryResult, query_probabilities};
use crate::syntax::parse_program;
use crate::vtree::{self, Kind};

fn answers(
    gp: &GroundProgram,
    formulation: Formulation,
    first_pairs: Option<usize>,
) -> (Option<Vec<QueryResult>>, Option<SearchStats>) {
    let deps = Deps::of(gp);
    let (vtree::Layout { vtree, order, .. }, _) =
        vtree::build(gp, &deps, Kind::Dynamic).expect("builds");
    vtree.context().run(|engine| {
        let backend = TddBackend::new(engine, vtree.clone());
        let var_of = fact_variables(gp, &order);
        let mut compilation =
            Compilation::new(gp, &deps, var_of, formulation, &backend, Projection::Off)
                .expect("maps every fact");
        if let Some(first) = first_pairs {
            compilation = compilation.dynamic(first);
        }
        compilation.run(gp, &backend, None).expect("compiles");
        let compiled = compilation.finish(&backend).expect("finishes");
        let (results, _) = query_probabilities(gp, &backend, &compiled).expect("answers");
        (results, compiled.stats.search)
    })
}

#[test]
fn every_example_answers_the_same_on_a_vtree_searched_at_every_step() {
    let mut kept = 0;
    for entry in std::fs::read_dir("examples").expect("the examples directory") {
        let path = entry.expect("an entry").path();
        // unsound.pl has no answer to compare; tests/it/examples.rs checks that
        // it is refused.
        if path.extension().is_none_or(|e| e != "pl") || path.ends_with("unsound.pl") {
            continue;
        }
        let program =
            parse_program(&std::fs::read_to_string(&path).expect("reads")).expect("parses");
        let gp = ground(&program, &GroundOptions::default()).expect("grounds");
        for formulation in [Formulation::Dual, Formulation::Primal] {
            let (fixed, none) = answers(&gp, formulation, None);
            assert!(
                none.is_none(),
                "{path:?}: a static compile reports no search"
            );
            let (moved, search) = answers(&gp, formulation, Some(0));
            kept += search
                .expect("a dynamic compile reports its search")
                .accepts;
            let (fixed, moved) = (
                fixed.expect("evidence possible"),
                moved.expect("evidence possible"),
            );
            assert_eq!(fixed.len(), moved.len());
            for (f, m) in fixed.iter().zip(&moved) {
                assert_eq!((&f.atom, f.positive), (&m.atom, m.positive), "{path:?}");
                assert!(
                    (f.probability - m.probability).abs() < 1e-9,
                    "{path:?} {formulation:?} {}: {} against {}",
                    f.atom,
                    f.probability,
                    m.probability
                );
            }
        }
    }
    assert!(kept > 0, "no example kept a move");
}
