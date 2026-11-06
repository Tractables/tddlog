//! ProbLog's `subquery/2` and `subquery/3`. `subquery(Goal, P, Evidence)`
//! binds `P` to the probability of an instance of `Goal` given the
//! observations of the list `Evidence` (none for `subquery/2`), an
//! inference of its own over the program, and holds once for each instance
//! of `Goal` the program can derive, as a certain fact. ProbLog makes the
//! call where its grounding of the program reaches it.
//!
//! [`answered`] makes every call before the program is ground, with the one
//! solver ([`super::solve_parsed`]) on the program with `Goal` as its query
//! and `Evidence` as its observations, and the program reads the answers as
//! facts. In a clause that the queries and the evidence reach, the call
//! `subquery(G, P, E)` is replaced by `'$sqK'(G, E, P)`, and each answer of
//! the call is a fact `'$sqK'(g, e, p)`: `g` the instance, `e` the
//! evidence and `p` the probability. The calls made are the instances of
//! `'$sqdK'(S) :- L1, ..., Ln.` that the grounding of the program finds
//! for that query, `L1` to `Ln` the literals before the call and `S` the
//! variables of `G` and `E` they bind. One call's answers can let the
//! literals before another hold, so the calls are found again until no new
//! one is made; a call with no answers yet fails.
//!
//! A variable of `Goal` that the literals before the call leave free is
//! left free in the call, also where the clause's caller binds it: the
//! call then answers every instance the program can derive, and the clause
//! keeps the caller's. ProbLog asks for the caller's instance alone and,
//! where the program cannot derive it, binds `P` to 0 where this call has
//! no answer. Where `P` is read as a probability the two agree.
//!
//! Refused: `subquery/4` and `/5` (a semiring and an evaluator), `P` bound
//! before the call, a goal that is not an atom, evidence that is not a list
//! of atoms and negated atoms, ground once the literals before the call
//! bind it, a call in a negation, a findall's goal or a query rule, calls
//! nested more than [`NESTING`] deep, and evidence of probability zero,
//! which ProbLog stops on too.

use rustc_hash::{FxHashMap, FxHashSet};

use super::{Options, SplitBounds, solve_parsed};
use crate::Error;
use crate::ground::terms::{ANSWERS, calls};
use crate::ground::{self, GroundError, GroundOptions};
use crate::syntax::{Atom, CONS, Clause, Compound, F64, HeadChoice, Literal, NIL, Program, Term};

/// How deep a call's goal may make calls of its own.
const NESTING: usize = 16;

/// The predicates that call a goal they are given, which may reach any
/// predicate.
const CALLERS: &[&str] = &[
    "call",
    "maplist",
    "foldl",
    "include",
    "exclude",
    "partition",
    "aggregate",
];

/// A call of `subquery/2,3` in a clause's body.
struct Call {
    /// The clause and the place in its body.
    at: (usize, usize),
    goal: Term,
    evidence: Term,
    /// The variables of the goal and the evidence that the literals before
    /// the call bind.
    inputs: Vec<Term>,
    /// The values of the inputs of the calls made.
    made: FxHashSet<Vec<Term>>,
    /// The facts of their answers.
    facts: Vec<Clause>,
}

fn unsupported(what: String) -> Error {
    Error::Ground(GroundError::Unsupported(what))
}

/// `program` with every call of `subquery/2,3` that its queries and
/// evidence reach answered, `depth` calls deep.
pub(super) fn answered(
    program: Program,
    options: &Options,
    depth: usize,
) -> Result<Program, Error> {
    // The arities of `subquery` that the program defines itself.
    let defined: Vec<usize> = program
        .clauses
        .iter()
        .flat_map(|c| &c.heads)
        .filter(|h| &*h.atom.name == "subquery")
        .map(|h| h.atom.args.len())
        .collect();
    let builtin = |a: &Atom| {
        &*a.name == "subquery"
            && (2..=5).contains(&a.args.len())
            && !defined.contains(&a.args.len())
    };
    let mut any = false;
    for c in &program.clauses {
        calls(&c.body, &mut |a| any |= builtin(a));
    }
    if !any {
        return Ok(program);
    }
    for r in &program.query_rules {
        let mut found = false;
        calls(&r.body, &mut |a| found |= builtin(a));
        if found {
            return Err(unsupported(format!(
                "subquery/3 in a query rule, for {}",
                r.pattern
            )));
        }
    }
    let reached = reached(&program);
    let reach = |h: &HeadChoice| {
        reached
            .as_ref()
            .is_none_or(|r| r.contains(&h.atom.signature()))
    };
    let mut working = program.clone();
    let mut made: Vec<Call> = Vec::new();
    for (ci, clause) in working.clauses.iter_mut().enumerate() {
        if !clause.heads.iter().any(reach) {
            continue;
        }
        let (mut at_top, mut all) = (0, 0);
        calls(&clause.body, &mut |a| all += usize::from(builtin(a)));
        for bi in 0..clause.body.len() {
            let Literal::Pos(a) = &clause.body[bi] else {
                continue;
            };
            if !builtin(a) {
                continue;
            }
            at_top += 1;
            let call = clause.body[bi].to_string();
            if a.args.len() > 3 {
                return Err(unsupported(format!(
                    "{call}: a semiring or an evaluator of its own"
                )));
            }
            let (goal, prob) = (a.args[0].clone(), a.args[1].clone());
            let evidence = a
                .args
                .get(2)
                .cloned()
                .unwrap_or_else(|| Term::Const(NIL.into()));
            let mut before = Vec::new();
            for l in &clause.body[..bi] {
                l.vars_into(&mut before);
            }
            if !matches!(&prob, Term::Var(v) if !before.contains(v)) {
                return Err(unsupported(format!(
                    "{call}: its probability is bound before the call"
                )));
            }
            if evidence.vars().iter().any(|v| !before.contains(v)) {
                return Err(unsupported(format!(
                    "{call}: its evidence has a variable that no literal before the call binds"
                )));
            }
            let mut asked = goal.vars();
            evidence.vars_into(&mut asked);
            let inputs = asked
                .into_iter()
                .filter(|v| before.contains(v))
                .map(Term::Var)
                .collect();
            clause.body[bi] = Literal::Pos(Atom::new(
                format!("{ANSWERS}{}", made.len() + 1),
                vec![goal.clone(), evidence.clone(), prob],
            ));
            made.push(Call {
                at: (ci, bi),
                goal,
                evidence,
                inputs,
                made: FxHashSet::default(),
                facts: Vec::new(),
            });
        }
        if at_top != all {
            return Err(unsupported(format!(
                "subquery/3 in a negation or in a findall's goal, in {clause}"
            )));
        }
    }
    if made.is_empty() {
        return Ok(working);
    }
    if depth >= NESTING {
        return Err(unsupported(format!(
            "subquery/3 calls nested more than {NESTING} deep"
        )));
    }
    // The demands' clauses take the groups after the program's, and the
    // answers' facts those after them.
    let first = working.clauses.iter().map(|c| c.group).max().unwrap_or(0) + 1;
    let mut group = first + made.len() as u32;
    let opts = GroundOptions {
        observe_query: false,
        ..options.ground_options()
    };
    loop {
        let mut demand = with_answers(&working, &made);
        demand.queries.clear();
        demand.evidence.clear();
        demand.query_rules.clear();
        for (k, c) in made.iter().enumerate() {
            let head = Atom::new(format!("$sqd{}", k + 1), c.inputs.clone());
            let body = demand.clauses[c.at.0].body[..c.at.1].to_vec();
            demand
                .clauses
                .push(Clause::rule(head.clone(), body, first + k as u32));
            demand.queries.push((head, true));
        }
        let ground = ground::ground(&demand, &opts)?;
        let mut new = false;
        for (instance, id, _) in &ground.queries {
            let k: usize = instance.name["$sqd".len()..]
                .parse()
                .expect("a demand's number");
            let call = &mut made[k - 1];
            let values = instance.args.to_vec();
            if id.is_none() || !call.made.insert(values.clone()) {
                continue;
            }
            new = true;
            let (evidence, results) = answers(&program, call, &values, options, depth)?;
            for (answer, p) in results {
                let fact = Atom::new(format!("{ANSWERS}{k}"), vec![answer, evidence.clone(), p]);
                call.facts.push(Clause::rule(fact, Vec::new(), group));
                group += 1;
            }
        }
        if !new {
            return Ok(with_answers(&working, &made));
        }
    }
}

/// `working` with the facts of the answers of the calls `made`.
fn with_answers(working: &Program, made: &[Call]) -> Program {
    let mut program = working.clone();
    for c in made {
        program.clauses.extend(c.facts.iter().cloned());
    }
    program
}

/// The evidence of `call` at its inputs' `values`, and its answers: each
/// instance of its goal with its probability, `program` solved under
/// `options` with the goal as its query and the evidence observed.
fn answers(
    program: &Program,
    call: &Call,
    values: &[Term],
    options: &Options,
    depth: usize,
) -> Result<(Term, Vec<(Term, Term)>), Error> {
    let binding: FxHashMap<&Term, &Term> = call.inputs.iter().zip(values).collect();
    let goal = call.goal.substituted(&binding);
    let evidence = call.evidence.substituted(&binding);
    let what = || format!("subquery({goal}, _, {evidence})");
    let Some(query) = callable(&goal) else {
        return Err(unsupported(format!("{}: its goal is not an atom", what())));
    };
    let mut observed = Vec::new();
    let mut list = &evidence;
    while let Some((_, [item, tail])) = list.as_compound().filter(|(n, _)| &***n == CONS) {
        let (atom, value) = match item.as_compound() {
            Some((name, [negated])) if &**name == "\\+" => (callable(negated), false),
            _ => (callable(item), true),
        };
        match atom {
            Some(a) if a.is_ground() => observed.push((a, value)),
            _ => {
                return Err(unsupported(format!(
                    "{}: its evidence {item} is not a ground atom or a negated one",
                    what()
                )));
            }
        }
        list = tail;
    }
    if !matches!(list, Term::Const(n) if **n == *NIL) {
        return Err(unsupported(format!(
            "{}: its evidence is not a list",
            what()
        )));
    }
    let nested_options = Options {
        queries: true,
        evidence_probability: false,
        trace: false,
        ..*options
    };
    let nested = Program {
        queries: vec![(query, true)],
        evidence: observed,
        query_rules: Vec::new(),
        ..program.clone()
    };
    let nested = answered(nested, &nested_options, depth + 1)?;
    let solution = solve_parsed(
        nested,
        &nested_options,
        &SplitBounds::default(),
        &mut |_| {},
    )?;
    let Some(results) = solution.answers else {
        return Err(unsupported(format!(
            "{}: its evidence has probability zero, which ProbLog stops on too",
            what()
        )));
    };
    let results = results
        .into_iter()
        .map(|r| {
            let args = Vec::from(r.atom.args);
            let answer = if args.is_empty() {
                Term::Const(r.atom.name)
            } else {
                Compound::term(r.atom.name, args)
            };
            (answer, Term::Float(F64::rounded(r.probability)))
        })
        .collect();
    Ok((evidence, results))
}

/// The atom a goal names: a symbol or a compound that is not a construct
/// of goals.
fn callable(t: &Term) -> Option<Atom> {
    match t {
        Term::Const(n) if **n != *NIL => Some(Atom::new(n.clone(), Vec::new())),
        Term::Compound(c) if !matches!(&*c.name, "," | ";" | "\\+" | "->" | CONS) => {
            Some(Atom::new(c.name.clone(), c.args.to_vec()))
        }
        _ => None,
    }
}

/// The predicates that the queries and the evidence reach through the
/// clauses' bodies; `None` where one of them is in [`CALLERS`].
fn reached(program: &Program) -> Option<FxHashSet<(&str, usize)>> {
    let mut next: Vec<(&str, usize)> = program
        .queries
        .iter()
        .chain(&program.evidence)
        .map(|(a, _)| a.signature())
        .chain(program.query_rules.iter().map(|r| r.pattern.signature()))
        .collect();
    for r in &program.query_rules {
        calls(&r.body, &mut |a| next.push(a.signature()));
    }
    let mut seen: FxHashSet<(&str, usize)> = FxHashSet::default();
    let mut stack = Vec::new();
    loop {
        for sig in next.drain(..) {
            if seen.insert(sig) {
                stack.push(sig);
            }
        }
        let Some(sig) = stack.pop() else {
            return Some(seen);
        };
        if CALLERS.contains(&sig.0) {
            return None;
        }
        for c in &program.clauses {
            if c.heads.iter().any(|h| h.atom.signature() == sig) {
                calls(&c.body, &mut |a| next.push(a.signature()));
            }
        }
    }
}
