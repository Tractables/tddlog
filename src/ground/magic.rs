//! Magic-set rewrite: demand-driven grounding.
//!
//! Bottom-up grounding derives every possible atom of every predicate. For
//! programs that only make sense with bindings flowing from the query (a time
//! index counting down from the query's argument, a probabilistic fact
//! `0.5::b(X).` over an unbounded domain) that is wrong or does not terminate.
//!
//! The rewrite adds, for every predicate `p` and binding pattern `α` reached
//! from the queries, a *magic* predicate `m_p^α` over the bound arguments,
//! seeded by the query and evidence atoms. Every rule for `p` gets a copy
//! guarded by `m_p^α(bound head args)`, and every body atom `q` in that copy
//! yields a magic rule `m_q^β(bound args of q) :- m_p^α(...), <preceding
//! literals>`, following the left-to-right scheduling of
//! [`super::schedule`], of those literals the ones that bear on the bound
//! arguments of `q`. The atoms themselves are not renamed, so the copies for
//! different patterns derive into the same predicate; every atom derived is in
//! the original model, and every atom the queries depend on is derived. A
//! given predicate, of ground facts or of rules that read only given
//! predicates, is not demanded: its rules are read as they are, and a
//! negation of it filters the demand as a positive literal does. A
//! rule that cannot use some of a pattern's bindings, a head variable its
//! body reads only as the value of an `is`, is copied under the pattern
//! without them instead, whose magic predicate the first one's feeds. A
//! predicate that the rewrite finds demanded both with every argument
//! free and with some bound is demanded free only, in a second rewrite.
//!
//! Magic predicates only control grounding: the grounder drops them from the
//! ground rules it emits (see `NormProgram::control`). Their truth in the
//! possible-world union means "demanded in some world", which is exactly the
//! relevance condition.
//!
//! The rewrite can make a stratified program non-stratified (demand for a
//! lower-stratum atom may pass through a recursive predicate of a higher
//! stratum). The caller checks stratification and falls back to the plain
//! program in that case.

use super::normalize::{NormProgram, Sig};
use super::schedule::{Order, literal_vars, schedule_body};
use crate::syntax::{Atom, Clause, Literal, Name, Term};
use rustc_hash::{FxHashMap, FxHashSet};

pub(crate) const MAGIC_PREFIX: &str = "$m";

/// Binding pattern: one flag per argument, `true` = bound.
type Adornment = Vec<bool>;

/// The predicates, by name and arity, and the patterns they are demanded in.
type Demands = FxHashSet<(Name, usize, Adornment)>;

/// `$m_<name>_<pattern>`, written into one string of its size rather
/// than formatted.
fn magic_name(name: &str, adornment: &Adornment) -> Name {
    let mut text = String::with_capacity(MAGIC_PREFIX.len() + name.len() + adornment.len() + 2);
    text.push_str(MAGIC_PREFIX);
    text.push('_');
    text.push_str(name);
    text.push('_');
    text.extend(adornment.iter().map(|b| if *b { 'b' } else { 'f' }));
    text.into()
}

fn magic_atom(atom: &Atom, adornment: &Adornment) -> Atom {
    Atom::new(
        magic_name(&atom.name, adornment),
        bound_args(atom, adornment),
    )
}

/// The arguments of `atom` that `adornment` binds.
fn bound_args(atom: &Atom, adornment: &Adornment) -> Vec<Term> {
    atom.args
        .iter()
        .zip(adornment)
        .filter(|(_, b)| **b)
        .map(|(t, _)| t.clone())
        .collect()
}

fn adornment_of(atom: &Atom, bound: &FxHashSet<String>) -> Adornment {
    atom.args
        .iter()
        .map(|t| match t {
            Term::Var(v) => bound.contains(v),
            // A query's pattern is bound where it is ground.
            t => t.is_ground(),
        })
        .collect()
}

/// The part of `adornment` that `rule` can use: a bound head variable that
/// the body reads only as the value an `is` computes binds nothing the body
/// reads before that test, and a demand for each of its values grounds the
/// body again for each: mnist_sum96's queries `sum(S)` demand
/// `partial(96, S)` for each sum, and its rule
/// `partial(K, S) :- next(J, K), partial(J, S0), digit(K, D), S is S0 + D`
/// would scan every sum of the level before and the ten digits for each,
/// per sum demanded. `None` where that is all of `adornment`.
fn usable(rule: &Clause, adornment: &Adornment) -> Option<Adornment> {
    let head = &rule.heads[0].atom;
    if rule.heads.len() > 1 {
        return None;
    }
    let read = |v: &String| {
        rule.body.iter().any(|l| match l {
            Literal::Is(Term::Var(w), e) if w == v => {
                let mut vs = Vec::new();
                e.vars_into(&mut vs);
                vs.contains(v)
            }
            other => literal_vars(other).contains(v),
        })
    };
    // A variable the body does not mention at all, as the `_` of
    // `walk([], _, _)`, only the demand binds.
    let mentioned = |v: &String| rule.body.iter().any(|l| literal_vars(l).contains(v));
    let keeps = |i: usize, t: &Term| match t {
        Term::Var(v) => {
            read(v) || !mentioned(v) || head.args.iter().enumerate().any(|(j, u)| j != i && u == t)
        }
        _ => true,
    };
    // A head of constants, as a tuple's, keeps all: asked first, without
    // the copy of the adornment per rule.
    if head
        .args
        .iter()
        .zip(adornment)
        .enumerate()
        .all(|(i, (t, &b))| !b || keeps(i, t))
    {
        return None;
    }
    Some(
        head.args
            .iter()
            .zip(adornment)
            .enumerate()
            .map(|(i, (t, &b))| b && keeps(i, t))
            .collect(),
    )
}

/// Whether no rule of `norm` has a variable. Queries and evidence may still
/// be patterns; the grounder resolves those against the possible atoms.
pub(crate) fn is_ground(norm: &NormProgram) -> bool {
    norm.query_drivers.is_empty() && norm.rules.iter().all(ground_clause)
}

/// Whether neither the heads nor the body of `c` has a variable.
pub(super) fn ground_clause(c: &Clause) -> bool {
    c.heads.iter().all(|h| h.atom.is_ground()) && c.body.iter().all(|l| literal_vars(l).is_empty())
}

/// The predicates the rewrite reads as they are, without a demand: those
/// whose clauses are all ground facts, and those whose rules read only
/// given predicates and bind their heads without a demand. Their atoms are
/// all known before any demand is, and a demand would only copy them under
/// guards for the grounder to check against every new demand: on
/// PSouffle's side-channel analyses, 83 571 facts against the demands of
/// each of 1 350 semi-naive rounds. A negation of a given predicate joins
/// the demand's prefix: cplint's left-corner grammar tries a goal's third
/// clause only where `\+ lc(G, B, rule(G, [B|_]))` holds, which is nowhere.
fn given<'a>(
    norm: &'a NormProgram,
    rules_of: &FxHashMap<Sig<'a>, Vec<usize>>,
) -> FxHashSet<Sig<'a>> {
    let reads_given = |rule: &Clause, given: &FxHashSet<Sig>| {
        let [head] = &rule.heads[..] else {
            return false;
        };
        let literals = rule.body.iter().all(|l| match l {
            Literal::Pos(a) | Literal::Neg(a) => given.contains(&a.signature()),
            Literal::Cmp(..) | Literal::Is(..) | Literal::Builtin(..) => true,
            Literal::Findall(_) | Literal::Collect(_) => false,
        });
        let mut bound = FxHashSet::default();
        literals
            && !head.negated
            && schedule_body(&rule.body, &mut bound, |_| false, Order::Source).is_ok()
            && literal_vars(&Literal::Pos(head.atom.clone()))
                .iter()
                .all(|v| bound.contains(v))
    };
    let switches: FxHashSet<Sig> = norm.switches.keys().map(|(n, k)| (&**n, *k)).collect();
    let mut given: FxHashSet<Sig> = FxHashSet::default();
    loop {
        let before = given.len();
        for (&sig, rules) in rules_of {
            if !given.contains(&sig)
                && !switches.contains(&sig)
                && rules.iter().all(|&ri| reads_given(&norm.rules[ri], &given))
            {
                given.insert(sig);
            }
        }
        if given.len() == before {
            return given;
        }
    }
}

/// The literals of `prefix`, positions in `body`, that bear on the values
/// of `args`: those that share a variable with them, or with a literal
/// that does, in their order. Another literal only asks that some values
/// of its own satisfy it, which a demand need not ask: on PSouffle's
/// side-channel analyses the demand of `equal_assign` with no argument
/// bound was derived again from each of the 437 million pairs of an
/// `xor_assign_right` and a `share` before it, 20 s for each of eight
/// rules.
fn bearing(body: &[Literal], prefix: &[usize], args: &[Term]) -> Vec<usize> {
    let mut reached: FxHashSet<String> = FxHashSet::default();
    for t in args {
        let mut vs = Vec::new();
        t.vars_into(&mut vs);
        reached.extend(vs);
    }
    let vars: Vec<Vec<String>> = prefix.iter().map(|&j| literal_vars(&body[j])).collect();
    let mut kept = vec![false; prefix.len()];
    let mut grew = true;
    while grew {
        grew = false;
        for (k, vs) in vars.iter().enumerate() {
            if !kept[k] && vs.iter().any(|v| reached.contains(v)) {
                kept[k] = true;
                grew = true;
                reached.extend(vs.iter().cloned());
            }
        }
    }
    prefix
        .iter()
        .zip(kept)
        .filter(|(_, k)| *k)
        .map(|(&j, _)| j)
        .collect()
}

/// Rewrite `norm`. Returns the rewritten program, or `None` if some rule
/// cannot be scheduled even with the head bindings (then the plain program
/// will report the same problem with a better message).
///
/// A predicate demanded with every argument free derives every atom it
/// can, and a demand of it with some bound grounds its rules again for
/// each call, with a tuple of its magic predicate per call. Where the rules
/// of such a demand read the predicate's own atoms before the bound head
/// argument (`rand(Y) :- rand(X), assign(Y, X)` asked for `rand(c)`), a
/// call of `assign` follows for every pair: on PSouffle's side-channel
/// analyses, 17 million tuples of the magic predicates of `assign` and the
/// relations it reads, where the program has 50 atoms that matter. A
/// predicate demanded both free and with some argument bound is therefore
/// rewritten again with every demand of it read as its free demand.
pub(crate) fn rewrite(norm: &NormProgram) -> Option<NormProgram> {
    let (out, done) = demand(norm, &FxHashSet::default())?;
    // Whether each predicate is demanded free, and with some argument bound.
    let mut patterns: FxHashMap<(&Name, usize), (bool, bool)> = FxHashMap::default();
    for (name, arity, adornment) in &done {
        let (free, bound) = patterns.entry((name, *arity)).or_default();
        if adornment.iter().any(|b| *b) {
            *bound = true;
        } else {
            *free = true;
        }
    }
    let free: FxHashSet<(Name, usize)> = patterns
        .into_iter()
        .filter(|(_, (free, bound))| *free && *bound)
        .map(|((name, arity), _)| (name.clone(), arity))
        .collect();
    if free.is_empty() {
        return Some(out);
    }
    demand(norm, &free).map(|(out, _)| out)
}

/// The rewrite of `norm` ([`rewrite`]), in which a demand of a predicate of
/// `free` with some argument bound is its demand with none, and the
/// predicates and patterns it demands.
fn demand(norm: &NormProgram, free: &FxHashSet<(Name, usize)>) -> Option<(NormProgram, Demands)> {
    let rules_of = norm.rules_by_sig();
    let is_switch = |a: &Atom| norm.switches.contains_key(&(a.name.clone(), a.args.len()));
    let given = given(norm, &rules_of);
    // An atom of a predicate with rules, not given, which a demand reaches.
    let derived = |a: &Atom| {
        let sig = a.signature();
        rules_of.contains_key(&sig) && !given.contains(&sig) && !is_switch(a)
    };
    // The given predicates reached, whose rules are in `out`.
    let mut read: FxHashSet<(Name, usize)> = FxHashSet::default();

    let mut out = NormProgram {
        switches: norm.switches.clone(),
        queries: norm.queries.clone(),
        evidence: norm.evidence.clone(),
        query_drivers: norm.query_drivers.clone(),
        counted: norm.counted.clone(),
        ..Default::default()
    };
    let mut control: FxHashSet<(Name, usize)> = FxHashSet::default();
    let mut done = Demands::default();
    let mut work: Vec<(Atom, Adornment)> = Vec::new();
    let adorn = |atom: &Atom, bound: &FxHashSet<String>| {
        if free.contains(&(atom.name.clone(), atom.args.len())) {
            vec![false; atom.args.len()]
        } else {
            adornment_of(atom, bound)
        }
    };

    // Seeds: query and evidence atoms, with constants bound; query-rule
    // drivers with nothing bound.
    let seed = |atom: &Atom, out: &mut NormProgram, work: &mut Vec<(Atom, Adornment)>| {
        let adornment = adorn(atom, &FxHashSet::default());
        if derived(atom) {
            out.rules
                .push(Clause::rule(magic_atom(atom, &adornment), vec![], u32::MAX));
        }
        work.push((atom.clone(), adornment));
    };
    for (q, _) in &norm.queries {
        seed(q, &mut out, &mut work);
    }
    for (e, _) in &norm.evidence {
        seed(e, &mut out, &mut work);
    }
    for (driver, pattern, _) in &norm.query_drivers {
        seed(driver, &mut out, &mut work);
        // Every true driver instance demands the pattern instance it names:
        // m_pattern(bound args) :- driver(vars). The pattern's variables are
        // the driver's arguments, so all of them are bound.
        let bound: FxHashSet<String> = literal_vars(&Literal::Pos(driver.clone()))
            .into_iter()
            .collect();
        let sub = adorn(pattern, &bound);
        if derived(pattern) {
            out.rules.push(Clause::rule(
                magic_atom(pattern, &sub),
                vec![Literal::Pos(driver.clone())],
                u32::MAX,
            ));
        }
        work.push((pattern.clone(), sub));
    }

    while let Some((atom, adornment)) = work.pop() {
        let sig = atom.signature();
        if given.contains(&sig) {
            if read.insert((atom.name.clone(), sig.1)) {
                for &ri in &rules_of[&sig] {
                    let rule = &norm.rules[ri];
                    for l in &rule.body {
                        if let Literal::Pos(a) | Literal::Neg(a) = l {
                            work.push((a.clone(), Vec::new()));
                        }
                    }
                    out.rules.push(rule.clone());
                }
            }
            continue;
        }
        if !derived(&atom) {
            continue;
        }
        if !done.insert((atom.name.clone(), sig.1, adornment.clone())) {
            continue;
        }
        // The demand's magic predicate, named once for all the rules it
        // guards, rather than the name made, the adornment copied and the
        // literals of the prefix copied per rule: TPC-H's tuples are each a
        // rule of one predicate.
        let magic = magic_name(sig.0, &adornment);
        control.insert((magic.clone(), adornment.iter().filter(|b| **b).count()));
        for &ri in &rules_of[&sig] {
            let rule = &norm.rules[ri];
            let head = &rule.heads[0].atom;
            // A rule that cannot use some of the demand's bindings is
            // grounded under the demand without them, which this demand
            // feeds.
            if let Some(kept) = usable(rule, &adornment) {
                out.rules.push(Clause::rule(
                    magic_atom(head, &kept),
                    vec![Literal::Pos(magic_atom(head, &adornment))],
                    u32::MAX,
                ));
                work.push((head.clone(), kept));
                continue;
            }
            // Head variables bound by the demand.
            let mut bound: FxHashSet<String> = FxHashSet::default();
            for (t, b) in head.args.iter().zip(&adornment) {
                if let Term::Var(v) = t
                    && *b
                {
                    bound.insert(v.clone());
                }
            }
            let guard = Literal::Pos(Atom::new(magic.clone(), bound_args(head, &adornment)));
            let ordered = schedule_body(
                &rule.body,
                &mut bound.clone(),
                |i| matches!(&rule.body[i], Literal::Pos(a) | Literal::Neg(a) if is_switch(a)),
                Order::Source,
            )
            .ok()?;
            // Walk the schedule, emitting magic rules for body atoms with rules.
            let mut bound_so_far = bound.clone();
            // After the guard, the body literals that bind: the positions
            // in the body, the literals copied only into a magic rule.
            let mut prefix: Vec<usize> = Vec::new();
            for (&i, lit) in ordered.iter().map(|i| (i, &rule.body[*i])) {
                // A findall demands its goal's solutions for the values of
                // the variables it shares.
                let read;
                let body_atom = match lit {
                    Literal::Pos(a) | Literal::Neg(a) => Some(a),
                    Literal::Collect(c) => {
                        read = c.pattern();
                        Some(&read)
                    }
                    _ => None,
                };
                if let Some(a) = body_atom {
                    if derived(a) {
                        let sub = adorn(a, &bound_so_far);
                        let bearing = bearing(&rule.body, &prefix, &bound_args(a, &sub));
                        let mut body = Vec::with_capacity(1 + bearing.len());
                        body.push(guard.clone());
                        body.extend(bearing.iter().map(|&j| rule.body[j].clone()));
                        out.rules
                            .push(Clause::rule(magic_atom(a, &sub), body, u32::MAX));
                        work.push((a.clone(), sub));
                    } else if given.contains(&a.signature()) {
                        work.push((a.clone(), Vec::new()));
                    }
                }
                // Positive literals and deterministic filters join the prefix,
                // and so do negations of given predicates, whose atoms are
                // all known before any demand is; other negative literals do
                // not (they would add negative dependencies to the magic
                // predicates) and switches bind nothing.
                match lit {
                    Literal::Neg(a) if !given.contains(&a.signature()) => {}
                    Literal::Pos(a) if is_switch(a) => {}
                    _ => prefix.push(i),
                }
                for v in literal_vars(lit) {
                    bound_so_far.insert(v);
                }
            }
            let mut body = Vec::with_capacity(1 + ordered.len());
            body.push(guard);
            // A counted rule keeps its literals in the order of the source,
            // by which its derivations are ordered.
            if norm.counted.contains(&(head.name.clone(), head.args.len())) {
                body.extend(rule.body.iter().cloned());
            } else {
                body.extend(ordered.iter().map(|&i| rule.body[i].clone()));
            }
            out.rules.push(Clause {
                heads: rule.heads.clone(),
                body,
                group: rule.group,
            });
        }
    }
    out.control = control;
    Some((out, done))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::normalize::normalize;
    use crate::syntax::parse_program;

    #[test]
    fn countdown_gets_demand_rules() {
        let p =
            parse_program("0.6::w(T) :- T > 0, P is T - 1, w(P).\nw(0).\nquery(w(3)).\n").unwrap();
        let n = normalize(&p).unwrap();
        let m = rewrite(&n).unwrap();
        let text: Vec<String> = m.rules.iter().map(|r| r.to_string()).collect();
        assert!(text.contains(&"'$m_w_b'(3).".to_string()), "{text:?}");
        assert!(
            text.contains(&"'$m_w_b'(P) :- '$m_w_b'(T), T>0, P is T-1.".to_string()),
            "{text:?}"
        );
        assert!(
            text.iter()
                .any(|t| t.starts_with("w(T) :- '$m_w_b'(T), T>0, P is T-1,")),
            "{text:?}"
        );
        assert!(m.control.contains(&("$m_w_b".into(), 1)));
    }

    /// A predicate asked with its argument free, whose rule reads its own
    /// atoms before the bound head argument, is demanded free only: its
    /// demand under a bound argument would call `a` for every pair of a
    /// bound head and an atom of `r`. Its free demand reads none of the
    /// literals before it, which bind nothing it is asked with.
    #[test]
    fn a_predicate_demanded_free_is_demanded_free_only() {
        let p = parse_program(
            "0.5::r(1).\na(2,1).\na(3,2).\nb(4,3).\n\
             r(Y) :- r(X), a(Y,X).\ns(Z) :- b(Z,X), r(X).\nquery(r(_)).\nquery(s(_)).\n",
        )
        .unwrap();
        let n = normalize(&p).unwrap();
        let m = rewrite(&n).unwrap();
        let text: Vec<String> = m.rules.iter().map(|r| r.to_string()).collect();
        assert!(
            !text
                .iter()
                .any(|t| t.contains("$m_r_b") || t.contains("$m_a_bb")),
            "{text:?}"
        );
        assert!(
            text.iter().any(|t| t == "'$m_r_f' :- '$m_s_f'."),
            "{text:?}"
        );
        assert!(
            text.iter()
                .any(|t| t.starts_with("r(Y) :- '$m_r_f', r(X), a(Y,X)")),
            "{text:?}"
        );
    }

    /// A predicate of ground facts is read as it is, not demanded: its facts
    /// are in the rewritten program once, unguarded, and only the rules of
    /// the derived predicates are copied under their demands.
    #[test]
    fn facts_are_read_as_they_are() {
        let p = parse_program(
            "e(1,2).\ne(2,3).\ne(4,5).\n0.5::p(X,Y) :- e(X,Y).\n\
             p(X,Z) :- e(X,Y), p(Y,Z).\nquery(p(1,_)).\n",
        )
        .unwrap();
        let n = normalize(&p).unwrap();
        let m = rewrite(&n).unwrap();
        let text: Vec<String> = m.rules.iter().map(|r| r.to_string()).collect();
        assert!(!text.iter().any(|t| t.contains("$m_e")), "{text:?}");
        for fact in ["e(1,2).", "e(2,3).", "e(4,5)."] {
            assert_eq!(text.iter().filter(|t| *t == fact).count(), 1, "{text:?}");
        }
        assert!(
            text.iter()
                .any(|t| t.starts_with("p(X,Z) :- '$m_p_bf'(X), e(X,Y), p(Y,Z)")),
            "{text:?}"
        );
    }

    /// A predicate whose rules read only facts is given too, and its
    /// negation filters the demand: the magic rule of `q` reads it, so `q`
    /// is demanded only where `nt` fails.
    #[test]
    fn a_negation_of_a_given_predicate_filters_the_demand() {
        let p = parse_program(
            "lc(s,a,r(a)).\nlc(s,b,r(c)).\nnt(G,B) :- lc(G,B,R), R = r(B).\n\
             0.5::q(X,N) :- lc(s,X,_), N = 1.\n\
             p(X,N) :- lc(s,X,_), \\+ nt(s,X), q(X,N).\nquery(p(b,_)).\n",
        )
        .unwrap();
        let n = normalize(&p).unwrap();
        let m = rewrite(&n).unwrap();
        let text: Vec<String> = m.rules.iter().map(|r| r.to_string()).collect();
        assert!(
            !text
                .iter()
                .any(|t| t.contains("$m_nt") || t.contains("$m_lc")),
            "{text:?}"
        );
        assert!(
            text.iter().any(
                |t| t.starts_with("'$m_q_bf'(X) :- '$m_p_bf'(X), ") && t.contains("\\+nt(s,X)")
            ),
            "{text:?}"
        );
    }

    /// A demand reads the literals before it that bear on its bound
    /// arguments: `c` is demanded where `a` holds, whatever `b` holds, and
    /// `b`, with no argument bound, wherever `h` is
    /// (tests/regress/magic_bearing.pl, by hand).
    #[test]
    fn a_demand_reads_the_literals_that_bear_on_it() {
        let p = parse_program(
            "0.5::a(1).\n0.4::d(5).\nb(7) :- d(9).\n0.6::c(1,3).\n\
             h(X) :- a(X), b(Y), c(X,Z).\nquery(h(1)).\n",
        )
        .unwrap();
        let n = normalize(&p).unwrap();
        let m = rewrite(&n).unwrap();
        let text: Vec<String> = m.rules.iter().map(|r| r.to_string()).collect();
        assert!(
            text.contains(&"'$m_c_bf'(X) :- '$m_h_b'(X), a(X).".to_string()),
            "{text:?}"
        );
        assert!(
            text.contains(&"'$m_b_f' :- '$m_h_b'(X).".to_string()),
            "{text:?}"
        );
    }

    /// A demand that binds an argument the body reads only as the value of
    /// an `is` grounds that rule under the demand without it, which the
    /// demand feeds; a rule that reads the argument keeps it.
    #[test]
    fn a_binding_read_only_by_is_is_dropped() {
        let p = parse_program(
            "0.5::d(1,0); 0.5::d(1,1).\n0.5::d(2,0); 0.5::d(2,1).\nnext(1,2).\n\
             s(1,S) :- d(1,S).\ns(K,S) :- next(J,K), s(J,S0), d(K,D), S is S0+D.\nquery(s(2,1)).\n",
        )
        .unwrap();
        let n = normalize(&p).unwrap();
        let m = rewrite(&n).unwrap();
        let text: Vec<String> = m.rules.iter().map(|r| r.to_string()).collect();
        assert!(
            text.contains(&"'$m_s_bf'(K) :- '$m_s_bb'(K,S).".to_string()),
            "{text:?}"
        );
        assert!(
            text.iter()
                .any(|t| t.starts_with("s(K,S) :- '$m_s_bf'(K),")),
            "{text:?}"
        );
        assert!(
            !text
                .iter()
                .any(|t| t.starts_with("s(K,S) :- '$m_s_bb'(K,S),")),
            "{text:?}"
        );
        assert!(
            text.iter()
                .any(|t| t.starts_with("s(1,S) :- '$m_s_bb'(1,S),")),
            "{text:?}"
        );
    }
}
