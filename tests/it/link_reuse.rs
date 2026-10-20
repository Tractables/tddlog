//! An atom that holds where a fact agrees with a derived atom reuses the
//! formula of the first such atom for that derived atom, with the fact
//! renamed (src/tp/compile.rs, `equivalence`). The renaming holds only
//! where the fact is outside the derived atom's cone: here `tmp` agrees
//! `d` with `v`, which is `\+d`, so `tmp` is false, and `link`, which
//! agrees `a` with `v`, is not. Reused, `link` was false under every
//! projected strategy, and the evidence on it inconsistent. `v` is
//! queried, so that the grounder keeps it rather than merging it into
//! `\+d` (`ground/aig.rs`). Answers computed by hand over `d` and `a`.

use crate::common;

use common::assert_answers;
use tddlog::Projection;
use tddlog::{Kind, Strategy};

const RULES: &str = "0.6::d.
0.5::a.
v :- \\+d.
tmp :- d, v.
tmp :- \\+d, \\+v.
link :- v, a.
link :- \\+v, \\+a.
";

fn check(tail: &str, expected: &[(&str, f64)]) {
    let source = format!("{RULES}{tail}");
    for strategy in [
        Strategy::Trial(Kind::Closed),
        Strategy::Auto,
        Strategy::Trial(Kind::Td),
        Strategy::Trial(Kind::Dynamic),
        Strategy::Trial(Kind::Families { search: false }),
        Strategy::Trial(Kind::Families { search: true }),
    ] {
        // A dynamic vtree is searched unprojected.
        let projections: &[Projection] = if strategy == Strategy::Trial(Kind::Dynamic) {
            &[Projection::Off]
        } else {
            &[
                Projection::Off,
                Projection::default(),
                Projection::Cones { max_share: 1.0 },
            ]
        };
        for &projection in projections {
            let what = format!("{tail} ({strategy:?}, {projection})");
            let run = common::run(&source, &common::options(strategy, projection))
                .unwrap_or_else(|e| panic!("{what}: {e}"));
            assert_answers(
                &what,
                &run.results
                    .unwrap_or_else(|| panic!("{what}: the evidence is consistent")),
                expected,
                1e-12,
            );
        }
    }
}

#[test]
fn a_link_whose_fact_is_in_the_cone_of_its_atom_is_built_from_its_rules() {
    // link holds where a agrees with v, that is where a differs from d.
    check(
        "query(v). query(tmp). query(link).",
        &[("v", 0.4), ("tmp", 0.0), ("link", 0.5)],
    );
    // Given link: P(a, not d) / P(link) and P(d, not a) / P(link), and v
    // where a holds.
    check(
        "evidence(link, true). query(v). query(tmp). query(a). query(d).",
        &[("v", 0.4), ("tmp", 0.0), ("a", 0.4), ("d", 0.6)],
    );
}
