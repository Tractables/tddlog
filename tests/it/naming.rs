//! Naming the atoms that cross a narrow cut of `program`'s order
//! (src/vtree/naming.rs) and settling which cuts keep their names
//! (src/tp/settle.rs): on a Markov chain unrolled over days, on a
//! link-failure grid and on a coupon collector, the answers and P(e) under
//! `program`, `program:fanin` and `auto` equal an exact forward pass over
//! the model's states, and `program` names atoms. The copies' weights are
//! one, or one half on each side where a cut is expanded, so P(e) counts
//! no copy. The coupon collector's cuts are expanded, its readers holding
//! less than the joint of its links; so are those of a collector that
//! draws two different coupons at a time (noDice's coupon), whose copies
//! of one draw's coupon a clause table defines, under one link that the
//! program observes false. And on cuts of Roulette's insurance and
//! hailfinder networks whose links read copies of their own cut or that a
//! tie stands on, the answers equal ProbLog's.

use crate::common;

use tddlog::{Kind, Options, Projection, Strategy};

/// The strategies each program is run under.
const STRATEGIES: [Strategy; 3] = [
    Strategy::Trial(Kind::Program { fan_in: false }),
    Strategy::Trial(Kind::Program { fan_in: true }),
    Strategy::Auto,
];

/// The weather factory's shape (GeNI's weather, smaller): the weather
/// `w_t` is a chain of its own, each of `factories` factories strikes on
/// day `t` with a probability that depends on whether it struck the day
/// before and on the weather, and `done_t` holds once all of them struck
/// on one day up to `t`. Each day states its facts before its rules, and
/// the atoms are propositional, as GeNI's are: the grounder numbers the
/// atoms of one predicate together, and `program` lays the facts out in
/// that order.
struct Weather {
    factories: usize,
    days: usize,
}

/// The probabilities: the weather stays fine 0.7 and turns fine 0.2; a
/// factory strikes on the first day 0.1, again 0.6, and newly 0.3 on a
/// fine day and 0.1 on another.
const FINE: f64 = 0.5;
const STAYS_FINE: f64 = 0.7;
const TURNS_FINE: f64 = 0.2;
const FIRST: f64 = 0.1;
const AGAIN: f64 = 0.6;
const NEW_FINE: f64 = 0.3;
const NEW_OTHER: f64 = 0.1;

impl Weather {
    fn program(&self, extra: &str) -> String {
        let n = self.factories;
        let mut src = format!("{FINE}::w_0.\n");
        for i in 0..n {
            src += &format!("{FIRST}::s_{i}_0.\n");
        }
        src += &format!(
            "all_0 :- {}.\ndone_0 :- all_0.\n",
            (0..n)
                .map(|i| format!("s_{i}_0"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        for t in 1..self.days {
            let p = t - 1;
            src += &format!(
                "{STAYS_FINE}::a_{t}.\n{TURNS_FINE}::b_{t}.\nw_{t} :- w_{p}, a_{t}.\nw_{t} :- \\+w_{p}, b_{t}.\n"
            );
            for i in 0..n {
                src += &format!(
                    "{AGAIN}::c_{i}_{t}.\n{NEW_FINE}::d_{i}_{t}.\n{NEW_OTHER}::e_{i}_{t}.\n\
                     s_{i}_{t} :- s_{i}_{p}, c_{i}_{t}.\n\
                     s_{i}_{t} :- \\+s_{i}_{p}, w_{t}, d_{i}_{t}.\n\
                     s_{i}_{t} :- \\+s_{i}_{p}, \\+w_{t}, e_{i}_{t}.\n"
                );
            }
            src += &format!(
                "all_{t} :- {}.\n",
                (0..n)
                    .map(|i| format!("s_{i}_{t}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            src += &format!("done_{t} :- done_{p}.\ndone_{t} :- all_{t}.\n");
        }
        src + extra
    }

    /// `(P(done_last ∧ w_observed = fine), P(w_observed = fine))` by a
    /// forward pass over the weather, the strikes and `done`; with
    /// `observed` `None`, `(P(done_last), 1)`.
    fn exact(&self, observed: Option<(usize, bool)>) -> (f64, f64) {
        let n = self.factories;
        let states = 1usize << (n + 2);
        // A state: bit 0 the weather, bits 1..=n the strikes, bit n+1 done.
        let done_bit = 1 << (n + 1);
        let all = ((1 << n) - 1) << 1;
        let mut dist = vec![0.0; states];
        for s in 0..1usize << n {
            let mut p = 1.0;
            for i in 0..n {
                p *= if s >> i & 1 == 1 { FIRST } else { 1.0 - FIRST };
            }
            for w in 0..2usize {
                let x = w | s << 1;
                let x = if x & all == all { x | done_bit } else { x };
                dist[x] += p * if w == 1 { FINE } else { 1.0 - FINE };
            }
        }
        let keep = |dist: &mut Vec<f64>, t: usize| {
            if let Some((day, fine)) = observed
                && day == t
            {
                for (x, p) in dist.iter_mut().enumerate() {
                    if (x & 1 == 1) != fine {
                        *p = 0.0;
                    }
                }
            }
        };
        keep(&mut dist, 0);
        for t in 1..self.days {
            let mut next = vec![0.0; states];
            for (x, &p) in dist.iter().enumerate().filter(|(_, p)| **p > 0.0) {
                let fine = x & 1 == 1;
                let pw = if fine { STAYS_FINE } else { TURNS_FINE };
                for w in 0..2usize {
                    let ps = p * if w == 1 { pw } else { 1.0 - pw };
                    let mut strikes = vec![(0usize, 1.0)];
                    for i in 0..n {
                        let before = x >> (i + 1) & 1 == 1;
                        let q = if before {
                            AGAIN
                        } else if w == 1 {
                            NEW_FINE
                        } else {
                            NEW_OTHER
                        };
                        strikes = strikes
                            .into_iter()
                            .flat_map(|(s, r)| [(s | 1 << i, r * q), (s, r * (1.0 - q))])
                            .collect();
                    }
                    for &(s, r) in &strikes {
                        let y = w | s << 1 | (x & done_bit);
                        let y = if y & all == all { y | done_bit } else { y };
                        next[y] += ps * r;
                    }
                }
            }
            dist = next;
            keep(&mut dist, t);
        }
        let total: f64 = dist.iter().sum();
        let done: f64 = dist
            .iter()
            .enumerate()
            .filter(|(x, _)| x & done_bit != 0)
            .map(|(_, p)| p)
            .sum();
        (done, total)
    }
}

/// A link-failure grid of `side` routers a side (Fierens et al.'s grid,
/// GeNI's link family): the top-left router is reached, and each other one
/// is reached when its left, upper or upper-left neighbour is and the link
/// from it, working with probability 0.5, works. The routers are stated
/// row by row, each with its links before its rules.
fn grid(side: usize, extra: &str) -> String {
    let mut src = String::from("c_0_0.\n");
    for i in 0..side {
        for j in 0..side {
            for (di, dj, name) in [(0, 1, "l"), (1, 0, "u"), (1, 1, "d")] {
                if i >= di && j >= dj && (i, j) != (0, 0) {
                    src += &format!(
                        "0.5::{name}_{i}_{j}.\nc_{i}_{j} :- c_{}_{}, {name}_{i}_{j}.\n",
                        i - di,
                        j - dj
                    );
                }
            }
        }
    }
    src + extra
}

/// P(the bottom-right router is reached) by a pass over the rows, the
/// reached routers of the row before its state, and every assignment of
/// a row's links.
fn grid_exact(side: usize) -> f64 {
    let mut dist = vec![0.0; 1 << side];
    for links in 0..1usize << side {
        // Row 0: each router after the first reached from its left one.
        let mut row = 1;
        for j in 1..side {
            if row >> (j - 1) & 1 == 1 && links >> j & 1 == 1 {
                row |= 1 << j;
            }
        }
        dist[row] += 0.5f64.powi(side as i32);
    }
    for _ in 1..side {
        let mut next = vec![0.0; 1 << side];
        for (prev, &p) in dist.iter().enumerate().filter(|(_, p)| **p > 0.0) {
            for links in 0..1usize << (3 * side) {
                let link = |j: usize, k: usize| links >> (3 * j + k) & 1 == 1;
                let mut row = 0;
                for j in 0..side {
                    let left = j > 0 && row >> (j - 1) & 1 == 1 && link(j, 0);
                    let up = prev >> j & 1 == 1 && link(j, 1);
                    let diag = j > 0 && prev >> (j - 1) & 1 == 1 && link(j, 2);
                    if left || up || diag {
                        row |= 1 << j;
                    }
                }
                next[row] += p * 0.5f64.powi(3 * side as i32);
            }
        }
        dist = next;
    }
    dist.iter()
        .enumerate()
        .filter(|(x, _)| x >> (side - 1) & 1 == 1)
        .map(|(_, p)| p)
        .sum()
}

/// A coupon collector (noDice's coupon, smaller): each of `draws` draws
/// is one of four types, each with probability 0.25; `g_i_t` holds once
/// type `i` was drawn by draw `t`, and `all` once every type was by the
/// last draw. Each draw states its choice before its rules.
fn coupon(draws: usize, extra: &str) -> String {
    let mut src = String::new();
    for t in 0..draws {
        src += &(0..4)
            .map(|i| format!("0.25::d_{i}_{t}"))
            .collect::<Vec<_>>()
            .join("; ");
        src += ".\n";
        for i in 0..4 {
            src += &format!("g_{i}_{t} :- d_{i}_{t}.\n");
            if t > 0 {
                src += &format!("g_{i}_{t} :- g_{i}_{}.\n", t - 1);
            }
        }
    }
    let last = draws - 1;
    src += &format!(
        "all :- {}.\n",
        (0..4)
            .map(|i| format!("g_{i}_{last}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    src + extra
}

/// `(P(all ∧ g_0_observed), P(g_0_observed))` by a forward pass over the
/// set of types drawn; with `observed` `None`, `(P(all), 1)`.
fn coupon_exact(draws: usize, observed: Option<usize>) -> (f64, f64) {
    let mut dist = [0.0; 16];
    dist[0] = 1.0;
    for t in 0..draws {
        let mut next = [0.0; 16];
        for (x, &p) in dist.iter().enumerate() {
            for i in 0..4 {
                next[x | 1 << i] += p * 0.25;
            }
        }
        dist = next;
        if observed == Some(t) {
            for (x, p) in dist.iter_mut().enumerate() {
                if x & 1 == 0 {
                    *p = 0.0;
                }
            }
        }
    }
    (dist[15], dist.iter().sum())
}

/// Asserts the answer to the one query of `source`, and its P(e), under
/// each strategy; whether `program` named atoms, and whether it expanded
/// a cut's names.
fn check(what: &str, source: &str, want: f64, want_evidence: f64) -> (bool, bool) {
    let (mut named, mut expanded) = (false, false);
    for strategy in STRATEGIES {
        let options = common::options(strategy, Projection::default());
        let run =
            common::run(source, &options).unwrap_or_else(|e| panic!("{what} {strategy:?}: {e}"));
        let got = run.results.as_ref().expect("consistent evidence")[0].probability;
        assert!(
            (got - want).abs() < 1e-12,
            "{what} {strategy:?}: {got} against {want}"
        );
        if strategy == Strategy::Trial(Kind::Program { fan_in: false }) {
            named = !run.ground.encoding.is_empty();
            expanded = run.query_sizes.names_expanded > 0;
        }
        let options = Options {
            evidence_probability: true,
            ..options
        };
        let run =
            common::run(source, &options).unwrap_or_else(|e| panic!("{what} {strategy:?}: {e}"));
        let p = run.evidence_probability.expect("P(e)").to_f64();
        assert!(
            (p - want_evidence).abs() < 1e-12 * want_evidence.max(1e-300),
            "{what} {strategy:?}: P(e) = {p} against {want_evidence}"
        );
    }
    (named, expanded)
}

#[test]
fn weather_answers_and_evidence_are_the_forward_pass() {
    for (factories, days) in [(2, 4), (3, 6), (3, 9)] {
        let weather = Weather { factories, days };
        let last = days - 1;
        let what = format!("weather {factories} {days}");
        let (done, _) = weather.exact(None);
        // The query observed: P(e) is its probability.
        let (named, _) = check(
            &what,
            &weather.program(&format!("query(done_{last}).\n")),
            done,
            1.0,
        );
        let (fine_and_done, _) = weather.exact(Some((1, true)));
        check(
            &what,
            &weather.program(&format!("evidence(done_{last}).\nquery(w_1).\n")),
            fine_and_done / done,
            done,
        );
        // The weather of day 2 observed foul.
        let (both, e) = weather.exact(Some((2, false)));
        check(
            &what,
            &weather.program(&format!("evidence(w_2, false).\nquery(done_{last}).\n")),
            both / e,
            e,
        );
        assert!(
            named || days < 9,
            "{what}: `program` names the days' states"
        );
    }
}

#[test]
fn grid_answers_and_evidence_are_the_forward_pass() {
    for side in [3, 4, 5] {
        let want = grid_exact(side);
        let last = side - 1;
        let what = format!("grid {side}");
        let (named, _) = check(
            &what,
            &grid(side, &format!("query(c_{last}_{last}).\n")),
            want,
            1.0,
        );
        check(
            &what,
            &grid(
                side,
                &format!("evidence(c_{last}_{last}).\nquery(c_{last}_{last}).\n"),
            ),
            1.0,
            want,
        );
        assert!(
            named || side < 5,
            "{what}: `program` names the rows' routers"
        );
    }
}

#[test]
fn coupon_answers_and_evidence_are_the_forward_pass() {
    for draws in [4, 6, 8] {
        let what = format!("coupon {draws}");
        let (all, _) = coupon_exact(draws, None);
        let (named, expanded) = check(&what, &coupon(draws, "query(all).\n"), all, 1.0);
        // A type drawn by the middle draw observed.
        let middle = draws / 2;
        let (both, e) = coupon_exact(draws, Some(middle));
        check(
            &what,
            &coupon(draws, &format!("evidence(g_0_{middle}).\nquery(all).\n")),
            both / e,
            e,
        );
        assert!(
            named || draws < 8,
            "{what}: `program` names the draws' sets"
        );
        assert!(
            expanded || draws < 8,
            "{what}: the readers hold less than the joint of the links"
        );
    }
}

/// A coupon collector that draws two coupons at a time and observes them
/// different (noDice's coupon, smaller): at each of `steps` steps two
/// draws `a_t` and `b_t` are each one of `types` types, uniformly, `obs_t`
/// holds where they differ, `g_i_t` once type `i` was drawn by step `t`,
/// and `all` once every type was by the last step.
fn pairs(types: usize, steps: usize, extra: &str) -> String {
    let mut src = String::new();
    for t in 0..steps {
        for d in ["a", "b"] {
            src += &(0..types)
                .map(|i| format!("{}::{d}_{t}({i})", 1.0 / types as f64))
                .collect::<Vec<_>>()
                .join("; ");
            src += ".\n";
        }
        for i in 0..types {
            src += &format!("same_{t} :- a_{t}({i}), b_{t}({i}).\n");
        }
        src += &format!("obs_{t} :- \\+same_{t}.\n");
        for i in 0..types {
            src += &format!("g_{i}_{t} :- a_{t}({i}).\ng_{i}_{t} :- b_{t}({i}).\n");
            if t > 0 {
                src += &format!("g_{i}_{t} :- g_{i}_{}.\n", t - 1);
            }
        }
    }
    let last = steps - 1;
    src += &format!(
        "all :- {}.\n",
        (0..types)
            .map(|i| format!("g_{i}_{last}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    src + extra
}

/// `(P(all ∧ e), P(e))` for [`pairs`] with every step's `obs_t` observed,
/// by a forward pass over the set of types drawn.
fn pairs_exact(types: usize, steps: usize) -> (f64, f64) {
    let mut dist = vec![0.0; 1 << types];
    dist[0] = 1.0;
    let p = 1.0 / (types * types) as f64;
    for _ in 0..steps {
        let mut next = vec![0.0; 1 << types];
        for (x, &q) in dist.iter().enumerate() {
            for i in 0..types {
                for j in (0..types).filter(|&j| j != i) {
                    next[x | 1 << i | 1 << j] += q * p;
                }
            }
        }
        dist = next;
    }
    (dist[(1 << types) - 1], dist.iter().sum())
}

#[test]
fn paired_coupon_answers_and_evidence_are_the_forward_pass() {
    for (types, steps) in [(4, 3), (4, 5), (6, 5)] {
        let what = format!("paired coupon {types} {steps}");
        let observed: String = (0..steps)
            .map(|t| format!("evidence(obs_{t}).\n"))
            .collect();
        let (all, e) = pairs_exact(types, steps);
        let (named, expanded) = check(
            &what,
            &pairs(types, steps, &format!("{observed}query(all).\n")),
            all / e,
            e,
        );
        assert!(
            named && expanded,
            "{what}: `program` names the steps' sets and expands a cut"
        );
    }
}

/// Roulette's insurance network as a circuit (converted through ABC),
/// cut down to the rules that keep `program` wrong before the fix.
/// `othercar_c` and `othercar_c_1` partition `socioecon_b`, a clause table
/// that the segment of `antitheft_g_2` reads through copies tied by one
/// link. An atom of the table's own segment that reads `othercar_c`
/// (`riskaversion_b_1`, through `riskaversion_g_1`) reads that first
/// reader's copy instead, so the link of the copy of `riskaversion_b_1`,
/// which the segment of `antitheft_g_2` reads too, reads a copy of its own
/// cut. Expanding that cut composed the readers with the clause link
/// before `riskaversion_b_1`'s link, which brought the copy back with
/// nothing to tie it: `program` answered 0.917065.
const INSURANCE_CUT: &str = "\
0.2::age_f1.
socioecon_b :- age_f1.
othercar_c :- \\+socioecon_b.
othercar_c_1 :- socioecon_b.
0.9::riskaversion_then_f0.
riskaversion_then_b0 :- riskaversion_then_f0.
riskaversion_g_1 :- \\+othercar_c, age_f1.
riskaversion_g_2 :- riskaversion_g_1.
0.95::riskaversion_then_1_f1.
riskaversion_b :- riskaversion_then_b0.
riskaversion_b_1 :- riskaversion_g_2, riskaversion_then_1_f1.
antitheft_g_2 :- riskaversion_b_1.
0.9::antitheft_then_then_6_f0.
0.7::antitheft_then_then_7_f0.
antitheft_then_b_2 :- antitheft_then_then_7_f0.
antitheft_then_b_2 :- antitheft_then_then_6_f0.
antitheft_g_3 :- riskaversion_b_1, riskaversion_b.
antitheft_g_4 :- antitheft_g_3, othercar_c.
antitheft_b :- antitheft_then_b_2, antitheft_g_2.
homebase_then_b_5 :- othercar_c_1.
0.2::homebase_then_f1.
0.75::homebase_then_f0_1.
homebase_then_b0 :- homebase_then_f0_1.
homebase_b :- antitheft_g_4, homebase_then_b0.
homebase_b_1 :- antitheft_g_4, homebase_then_f1.
homebase_b_1 :- homebase_then_b_5, antitheft_g_2.
theft_then_c :- \\+homebase_b_1.
theft_g_1 :- antitheft_b, \\+theft_then_c.
theft_g_3 :- theft_g_1.
theft_g_4 :- theft_g_3, \\+homebase_b.
0.9::theft_then_then_12_f0.
theft_then_b_3 :- theft_then_then_12_f0.
theft_b :- theft_g_4, theft_then_b_3.
thiscarcost_then_then_b :- \\+theft_b.
thiscarcost_then_b :- thiscarcost_then_then_b.
thiscarcost_b :- thiscarcost_then_b.
propcost_then_c :- \\+thiscarcost_b.
propcost_then_b :- propcost_then_c.
propcost_b :- propcost_then_b.
propcost(0) :- \\+propcost_b.
query(propcost(0)).
";

/// `propcost(0)` is `\+theft_b`, and `theft_b` holds exactly where
/// `antitheft_then_b_2` (1 - 0.3·0.1), `age_f1`, `riskaversion_then_1_f1`
/// and `theft_then_then_12_f0` do (`antitheft_g_4` never holds, since it
/// reads `othercar_c` and `age_f1` together). ProbLog 2.3.0 (`-k sdd`):
/// 0.83413.
#[test]
fn a_link_that_reads_a_copy_of_its_own_cut() {
    let want = 1.0 - 0.97 * 0.2 * 0.95 * 0.9;
    let (named, _) = check("insurance's cut", INSURANCE_CUT, want, 1.0);
    assert!(named, "`program` names the atoms that cross its cuts");
}

/// Roulette's insurance network, cut down to the rules that keep `program`
/// wrong before the fix. `ruggedauto`'s table is split on `makemodel`
/// (`indicators`' table splits), and its parts for `economy` and
/// `superluxury` read `vehicleyear(older)` through copies of their own in
/// one segment: the second copy's link ties it to the first and reads the
/// part's copy of `vehicleyear(current)`, copies of its own cut.
const INSURANCE_SPLIT: &str = "\
0.2::age(adolescent); 0.6::age(adult); 0.2::age(senior).
0.4::socioecon(prole); 0.4::socioecon(middle); 0.19::socioecon(uppermiddle); 0.01::socioecon(wealthy) :- age(adolescent).
0.4::socioecon(prole); 0.4::socioecon(middle); 0.19::socioecon(uppermiddle); 0.01::socioecon(wealthy) :- age(adult).
0.01::riskaversion(psychopath); 0.09::riskaversion(adventurous); 0.4::riskaversion(normal); 0.5::riskaversion(cautious) :- age(senior).
0.9::vehicleyear(current); 0.1::vehicleyear(older) :- riskaversion(cautious).
0.3::makemodel(sportscar); 0.01::makemodel(economy); 0.09::makemodel(familysedan); 0.4::makemodel(luxury); 0.2::makemodel(superluxury) :- riskaversion(normal).
drivquality(excellent) :- riskaversion(cautious).
0.995::accident(none); 0.003::accident(mild); 0.001::accident(moderate); 0.001::accident(severe) :- drivquality(excellent).
0.95::ruggedauto(eggshell); 0.04::ruggedauto(football); 0.01::ruggedauto(tank) :- makemodel(sportscar).
0.5::ruggedauto(eggshell); 0.5::ruggedauto(football) :- makemodel(economy).
0.2::ruggedauto(eggshell); 0.6::ruggedauto(football); 0.2::ruggedauto(tank) :- makemodel(familysedan).
0.1::ruggedauto(eggshell); 0.6::ruggedauto(football); 0.3::ruggedauto(tank) :- makemodel(luxury).
0.05::ruggedauto(eggshell); 0.55::ruggedauto(football); 0.4::ruggedauto(tank) :- makemodel(superluxury), vehicleyear(current).
0.95::ruggedauto(eggshell); 0.04::ruggedauto(football); 0.01::ruggedauto(tank) :- makemodel(sportscar).
0.9::ruggedauto(eggshell); 0.1::ruggedauto(football) :- makemodel(economy), vehicleyear(older).
0.05::ruggedauto(eggshell); 0.55::ruggedauto(football); 0.4::ruggedauto(tank) :- makemodel(familysedan).
0.1::ruggedauto(eggshell); 0.6::ruggedauto(football); 0.3::ruggedauto(tank) :- makemodel(luxury).
0.05::ruggedauto(eggshell); 0.55::ruggedauto(football); 0.4::ruggedauto(tank) :- makemodel(superluxury), vehicleyear(older).
0.001::thiscardam(none); 0.9::thiscardam(mild); 0.098::thiscardam(moderate); 0.001::thiscardam(severe) :- ruggedauto(eggshell).
0.999997::homebase(secure); 1e-06::homebase(city); 1e-06::homebase(suburb); 1e-06::homebase(rural) :- socioecon(uppermiddle).
0.95::homebase(secure); 1e-06::homebase(city); 1e-06::homebase(suburb); 0.049998::homebase(rural) :- riskaversion(adventurous).
0.85::homebase(secure); 1e-06::homebase(city); 0.001::homebase(suburb); 0.148999::homebase(rural) :- riskaversion(normal).
0.0002::theft(true); 0.9998::theft(false) :- homebase(rural).
0.005::thiscarcost(thousand); 0.005::thiscarcost(tenthou); 0.99::thiscarcost(hundredthou) :- theft(false).
0.98::thiscarcost(thousand); 0.01::thiscarcost(tenthou); 0.01::thiscarcost(hundredthou) :- thiscardam(mild).
0.003::thiscarcost(thousand); 0.003::thiscarcost(tenthou); 0.044::thiscarcost(hundredthou); 0.95::thiscarcost(million) :- thiscardam(moderate).
1e-06::thiscarcost(thousand); 1e-06::thiscarcost(tenthou); 0.029998::thiscarcost(hundredthou); 0.97::thiscarcost(million) :- thiscardam(severe).
0.1::othercarcost(thousand); 0.5::othercarcost(tenthou); 0.39994::othercarcost(hundredthou); 6e-05::othercarcost(million) :- ruggedauto(football).
0.4::othercarcost(thousand); 0.3::othercarcost(tenthou); 0.29996::othercarcost(hundredthou); 4e-05::othercarcost(million) :- accident(moderate).
0.005::othercarcost(thousand); 0.55::othercarcost(tenthou); 0.4449::othercarcost(hundredthou); 0.0001::othercarcost(million) :- accident(severe), ruggedauto(tank).
0.7::propcost(thousand); 0.3::propcost(tenthou) :- thiscarcost(thousand).
propcost(million) :- thiscarcost(hundredthou).
propcost(million) :- othercarcost(thousand).
";

/// ProbLog 2.3.0 (`-k sdd`, one query at a time) answers 0.019884864,
/// 0.0085220844, 0 and 0.016076784; `program` answered 0.019029, 0.008155,
/// 0 and 0.015402 before the fix.
#[test]
fn a_split_tables_ties_read_copies_of_their_own_cut() {
    let queries = ["thousand", "tenthou", "hundredthou", "million"];
    let source: String = queries.iter().fold(INSURANCE_SPLIT.to_string(), |src, q| {
        src + &format!("query(propcost({q})).\n")
    });
    let want: Vec<(String, f64)> = queries
        .iter()
        .map(|q| format!("propcost({q})"))
        .zip([0.019884864, 0.0085220844, 0.0, 0.016076784])
        .collect();
    for strategy in STRATEGIES {
        let options = common::options(strategy, Projection::default());
        let run = common::run(&source, &options)
            .unwrap_or_else(|e| panic!("insurance's split {strategy:?}: {e}"));
        common::assert_answers(
            &format!("insurance's split {strategy:?}"),
            run.results.as_ref().expect("consistent evidence"),
            &want,
            1e-8,
        );
        if strategy == Strategy::Trial(Kind::Program { fan_in: false }) {
            assert!(!run.ground.encoding.is_empty(), "`program` names atoms");
        }
    }
}

/// Roulette's hailfinder network, cut down to the rules that keep
/// `program` wrong once a link's own cut is composed in order. The
/// segments of `scenrelamcin(ab)` and `capinscen(morethanave)` read the
/// clause table `scenario` through copies of their own, and the second
/// reader's link is tied to the first reader's copies (`tp::tie`): the
/// evidence holds the tie, which reads copies of the first reader's cut.
/// Expanding that cut released them, and the tie then held the second
/// reader's copies equal to free variables.
const HAILFINDER_TIE: &str = "\
0.254098::date(may15_jun14); 0.131148::date(jun15_jul1); 0.106557::date(jul2_jul15); 0.213115::date(jul16_aug10); 0.07377::date(aug11_aug20); 0.221312::date(aug20_sep15).
0.1::scenario(a); 0.16::scenario(b); 0.1::scenario(c); 0.08::scenario(d); 0.08::scenario(e); 0.01::scenario(f); 0.08::scenario(g); 0.1::scenario(h); 0.09::scenario(i); 0.03::scenario(j); 0.17::scenario(k) :- date(may15_jun14).
0.05::scenario(a); 0.16::scenario(b); 0.09::scenario(c); 0.09::scenario(d); 0.12::scenario(e); 0.02::scenario(f); 0.13::scenario(g); 0.06::scenario(h); 0.07::scenario(i); 0.11::scenario(j); 0.1::scenario(k) :- date(jun15_jul1).
0.04::scenario(a); 0.13::scenario(b); 0.1::scenario(c); 0.08::scenario(d); 0.15::scenario(e); 0.03::scenario(f); 0.14::scenario(g); 0.04::scenario(h); 0.06::scenario(i); 0.15::scenario(j); 0.08::scenario(k) :- date(jul2_jul15).
0.04::scenario(a); 0.13::scenario(b); 0.09::scenario(c); 0.07::scenario(d); 0.2::scenario(e); 0.08::scenario(f); 0.06::scenario(g); 0.05::scenario(h); 0.07::scenario(i); 0.13::scenario(j); 0.08::scenario(k) :- date(jul16_aug10).
0.04::scenario(a); 0.11::scenario(b); 0.1::scenario(c); 0.07::scenario(d); 0.17::scenario(e); 0.05::scenario(f); 0.1::scenario(g); 0.05::scenario(h); 0.07::scenario(i); 0.14::scenario(j); 0.1::scenario(k) :- date(aug11_aug20).
0.05::scenario(a); 0.11::scenario(b); 0.1::scenario(c); 0.08::scenario(d); 0.11::scenario(e); 0.02::scenario(f); 0.11::scenario(g); 0.06::scenario(h); 0.08::scenario(i); 0.11::scenario(j); 0.17::scenario(k) :- date(aug20_sep15).
scenrelamcin(ab) :- scenario(a).
scenrelamcin(ab) :- scenario(b).
scenrelamcin(cthruk) :- scenario(c).
scenrelamcin(cthruk) :- scenario(d).
scenrelamcin(cthruk) :- scenario(e).
scenrelamcin(cthruk) :- scenario(f).
scenrelamcin(cthruk) :- scenario(g).
scenrelamcin(cthruk) :- scenario(h).
scenrelamcin(cthruk) :- scenario(i).
scenrelamcin(cthruk) :- scenario(j).
scenrelamcin(cthruk) :- scenario(k).
0.15::morningcin(none); 0.57::morningcin(partinhibit); 0.2::morningcin(stifling); 0.08::morningcin(totalinhibit).
0.3::amcininscen(lessthanave); 0.6::amcininscen(average); 0.1::amcininscen(morethanave) :- morningcin(partinhibit).
0.01::amcininscen(lessthanave); 0.4::amcininscen(average); 0.59::amcininscen(morethanave) :- morningcin(stifling).
0.1::amcininscen(average); 0.9::amcininscen(morethanave) :- scenrelamcin(ab).
0.03::amcininscen(average); 0.97::amcininscen(morethanave) :- scenrelamcin(cthruk).
capinscen(morethanave) :- amcininscen(morethanave).
0.3::amdewptcalpl(instability); 0.25::amdewptcalpl(neutral); 0.45::amdewptcalpl(stability).
0.6::aminswliscen(lessunstable); 0.3::aminswliscen(average); 0.1::aminswliscen(moreunstable) :- amdewptcalpl(instability).
0.3::aminswliscen(lessunstable); 0.55::aminswliscen(average); 0.15::aminswliscen(moreunstable) :- amdewptcalpl(instability).
0.3::aminswliscen(lessunstable); 0.3::aminswliscen(average); 0.4::aminswliscen(moreunstable) :- amdewptcalpl(instability).
0.15::aminswliscen(lessunstable); 0.3::aminswliscen(average); 0.55::aminswliscen(moreunstable) :- amdewptcalpl(instability).
0.07::aminswliscen(lessunstable); 0.38::aminswliscen(average); 0.55::aminswliscen(moreunstable) :- amdewptcalpl(instability).
0.03::aminswliscen(lessunstable); 0.17::aminswliscen(average); 0.8::aminswliscen(moreunstable) :- amdewptcalpl(instability).
0.75::aminswliscen(lessunstable); 0.2::aminswliscen(average); 0.05::aminswliscen(moreunstable) :- amdewptcalpl(stability).
0.6::aminswliscen(lessunstable); 0.3::aminswliscen(average); 0.1::aminswliscen(moreunstable) :- amdewptcalpl(stability).
0.5::aminswliscen(lessunstable); 0.43::aminswliscen(average); 0.07::aminswliscen(moreunstable) :- amdewptcalpl(stability).
0.3::aminswliscen(lessunstable); 0.5::aminswliscen(average); 0.2::aminswliscen(moreunstable) :- amdewptcalpl(stability).
0.5::aminswliscen(lessunstable); 0.4::aminswliscen(average); 0.1::aminswliscen(moreunstable) :- amdewptcalpl(stability).
0.45::aminswliscen(lessunstable); 0.4::aminswliscen(average); 0.15::aminswliscen(moreunstable) :- amdewptcalpl(stability).
0.35::aminswliscen(lessunstable); 0.35::aminswliscen(average); 0.3::aminswliscen(moreunstable) :- amdewptcalpl(stability).
0.25::aminswliscen(lessunstable); 0.4::aminswliscen(average); 0.35::aminswliscen(moreunstable) :- amdewptcalpl(stability).
0.04::aminswliscen(lessunstable); 0.16::aminswliscen(average); 0.8::aminswliscen(moreunstable) :- amdewptcalpl(stability).
0.1::aminswliscen(lessunstable); 0.25::aminswliscen(average); 0.65::aminswliscen(moreunstable) :- amdewptcalpl(stability).
inssclinscen(moreunstable) :- aminswliscen(moreunstable).
scenrel3_4(acefk) :- scenario(a).
scenrel3_4(b) :- scenario(b).
scenrel3_4(acefk) :- scenario(c).
scenrel3_4(d) :- scenario(d).
scenrel3_4(acefk) :- scenario(e).
scenrel3_4(acefk) :- scenario(f).
scenrel3_4(gj) :- scenario(g).
scenrel3_4(hi) :- scenario(h).
scenrel3_4(hi) :- scenario(i).
scnrelplfcst(j) :- scenario(j).
scnrelplfcst(k) :- scenario(k).
0.5::plainsfcst(xnil); 0.22::plainsfcst(sig); 0.28::plainsfcst(svr) :- capinscen(morethanave), scnrelplfcst(j).
0.3::plainsfcst(xnil); 0.28::plainsfcst(sig); 0.42::plainsfcst(svr) :- inssclinscen(moreunstable).
0.88::plainsfcst(xnil); 0.1::plainsfcst(sig); 0.02::plainsfcst(svr) :- scnrelplfcst(k).
0.99::n34starfcst(xnil); 0.01::n34starfcst(sig) :- scenrel3_4(hi).
0.04::n34starfcst(xnil); 0.94::n34starfcst(sig); 0.02::n34starfcst(svr) :- plainsfcst(sig).
0.01::n34starfcst(xnil); 0.89::n34starfcst(sig); 0.1::n34starfcst(svr) :- plainsfcst(sig).
0.01::n34starfcst(xnil); 0.05::n34starfcst(sig); 0.94::n34starfcst(svr) :- scenrel3_4(acefk).
0.03::n34starfcst(sig); 0.97::n34starfcst(svr) :- scenrel3_4(b).
0.01::n34starfcst(sig); 0.99::n34starfcst(svr) :- scenrel3_4(d).
0.01::n34starfcst(xnil); 0.04::n34starfcst(sig); 0.95::n34starfcst(svr) :- scenrel3_4(gj), plainsfcst(svr).
r5fcst(svr) :- n34starfcst(sig).
";

/// ProbLog 2.3.0 (`-k sdd`) answers 0.2510229974258016 for
/// `r5fcst(svr)`; no rule makes the other two values. `program` answered 0.247402 before the fix.
#[test]
fn a_cut_whose_link_a_tie_stands_on() {
    let source =
        format!("{HAILFINDER_TIE}query(r5fcst(xnil)).\nquery(r5fcst(sig)).\nquery(r5fcst(svr)).\n");
    let want = [
        ("r5fcst(xnil)", 0.0),
        ("r5fcst(sig)", 0.0),
        ("r5fcst(svr)", 0.2510229974258016),
    ];
    for strategy in STRATEGIES {
        let options = common::options(strategy, Projection::default());
        let run = common::run(&source, &options)
            .unwrap_or_else(|e| panic!("hailfinder's tie {strategy:?}: {e}"));
        common::assert_answers(
            &format!("hailfinder's tie {strategy:?}"),
            run.results.as_ref().expect("consistent evidence"),
            &want,
            1e-9,
        );
    }
}
