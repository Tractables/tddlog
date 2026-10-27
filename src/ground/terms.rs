//! Compound terms, list predicates and findall/3 lowered to the literals
//! the rest of the grounder reads.
//!
//! The grounder reads atoms whose arguments are variables or ground terms,
//! and three literals beside Datalog's: a unification `V = f(A1,...,An)`
//! whose arguments are variables or ground terms, which constructs `V` or
//! takes it apart ([`super::schedule`]); a call of a list predicate
//! ([`Builtin`]); and a [`Collect`], a findall. This pass writes a program
//! so:
//!
//! - each compound term with variables in a head, an atom or a comparison
//!   of terms becomes a fresh variable `V` and a unification `V = f(..)` of
//!   the body, its own compound arguments in turn; a unification of two
//!   compound terms is taken apart, or makes its clause false;
//! - a call of a list predicate the program does not define becomes a
//!   [`Literal::Builtin`];
//! - `findall(T, G, R)` becomes an auxiliary predicate `'$fa<k>'` whose
//!   tuples are the solutions of `G`, one per substitution of its
//!   variables: the variables it shares with the clause, the value of `T`
//!   and its other variables, a clause per alternative of `G`; and a
//!   [`Collect`] that lists the template values of the tuples with the
//!   shared variables' values;
//! - a negated atom with variables of its own, which no other literal and
//!   no head reads (`\+ p(X, _)`, true where `p(X, Y)` holds for no `Y`),
//!   becomes the negation of an auxiliary predicate `'$nt<k>'` over its
//!   other variables, defined by the atom.
//!
//! Before any of it, a query, an observation or a call of a predicate that
//! has no clauses is refused where the queries and the evidence reach it,
//! as ProbLog refuses it ([`check_defined`]).
//!
//! The predicates a findall reads, its cone, are counted: the engine lists
//! the solutions in the order of resolution from their derivations, and
//! refuses a solution with more than one derivation, which ProbLog lists
//! once per proof ([`super::engine`]). The cone's clauses get groups of
//! their own, one per clause and alternative, in the order of the source.
//! A cone that is recursive, reads the findall's own clause or has a
//! disjunction after another literal (whose alternatives would be listed in
//! another order than resolution's) is refused.

use std::borrow::Cow;
use std::sync::Arc;

use rustc_hash::{FxHashMap, FxHashSet};

use super::engine::GroundError;
use crate::syntax::{
    Atom, Builtin, CONS, Clause, CmpOp, Collect, Compound, Findall, Literal, Name, Program, Term,
};

/// The prefix of the auxiliary predicates of findalls.
pub(crate) const FINDALL_PREFIX: &str = "$fa";

/// The prefix of the auxiliary predicates of negations with variables of
/// their own.
const NEGATION_PREFIX: &str = "$nt";

/// A predicate, `(name, arity)`.
pub(crate) type Pred = (Name, usize);

/// `program` lowered, and the predicates of the cones of its findalls;
/// the program itself, borrowed, where it has nothing to lower.
pub(crate) fn lower(program: &Program) -> Result<(Cow<'_, Program>, FxHashSet<Pred>), GroundError> {
    // A compound term with variables; a call that may be of a built-in,
    // unless the program defines its predicate.
    let compound = |t: &Term| matches!(t, Term::Compound(_)) && !t.is_ground();
    let lowers = |l: &Literal| match l {
        Literal::Pos(a) | Literal::Neg(a) => {
            a.args.iter().any(compound) || Builtin::of(&a.name, a.args.len()).is_some()
        }
        Literal::Cmp(op, a, b) => {
            !op.arithmetic() && (matches!(a, Term::Compound(_)) || matches!(b, Term::Compound(_)))
        }
        Literal::Findall(_) | Literal::Builtin(..) | Literal::Collect(_) => true,
        Literal::Is(..) => false,
    };
    let defined = by_head(program);
    check_defined(program, &defined)?;
    if !program.clauses.iter().any(|c| {
        c.heads.iter().any(|h| h.atom.args.iter().any(compound))
            || c.body.iter().any(lowers)
            || projects(c.heads.iter().map(|h| &h.atom), &c.body)
    }) && !program
        .query_rules
        .iter()
        .any(|r| r.body.iter().any(lowers) || projects(std::iter::once(&r.pattern), &r.body))
    {
        return Ok((Cow::Borrowed(program), FxHashSet::default()));
    }
    let mut l = Lowering {
        defined: &defined,
        fresh: 0,
        aux: 0,
        out: Vec::with_capacity(program.clauses.len()),
        cone_roots: Vec::new(),
    };
    let mut lowered = program.clone();
    spread(&mut lowered);
    // The clauses in order, each followed by the auxiliary clauses of its
    // findalls.
    let mut work: Vec<Clause> = std::mem::take(&mut lowered.clauses);
    work.reverse();
    l.run(&mut work)?;
    let mut query_rules = Vec::with_capacity(lowered.query_rules.len());
    for mut r in std::mem::take(&mut lowered.query_rules) {
        let mut extra = Vec::new();
        l.project(&mut r.body, &atom_vars([&r.pattern]), &mut extra);
        let Some(body) = l.body(std::mem::take(&mut r.body), &[], &mut extra)? else {
            continue;
        };
        r.body = body;
        query_rules.push(r);
        // The auxiliary clauses of a query rule's findalls.
        extra.reverse();
        l.run(&mut extra)?;
    }
    lowered.clauses = std::mem::take(&mut l.out);
    lowered.query_rules = query_rules;
    let cone = cone(&mut lowered, &l.cone_roots)?;
    Ok((Cow::Owned(lowered), cone))
}

/// The list predicate `a` calls, where the program defines none of its name
/// and arity.
fn builtin(a: &Atom, defined: &ByHead) -> Option<Builtin> {
    let (name, arity) = a.signature();
    Builtin::of(name, arity).filter(|_| !defined.contains_key(&(name, arity)))
}

/// The predicates the program's clauses define, by their heads, each with
/// those of its clauses that have a body.
type ByHead<'p> = FxHashMap<(&'p str, usize), Vec<&'p Clause>>;

/// [`ByHead`] of `program`.
fn by_head(program: &Program) -> ByHead<'_> {
    // The clauses of a predicate mostly come one after another: a run of
    // them is looked up once.
    let mut runs: Vec<((&str, usize), Vec<&Clause>)> = Vec::new();
    for c in &program.clauses {
        for h in &c.heads {
            let sig = h.atom.signature();
            if runs.last().is_none_or(|(s, _)| *s != sig) {
                runs.push((sig, Vec::new()));
            }
            if !c.body.is_empty() {
                runs.last_mut().expect("a run").1.push(c);
            }
        }
    }
    let mut out = ByHead::default();
    for (sig, rules) in runs {
        out.entry(sig).or_default().extend(rules);
    }
    out
}

/// Refuses what ProbLog stops on with its "No clauses found": a query or an
/// observation of a predicate that no clause defines, and a call of one
/// that is not a built-in tddlog evaluates, in a clause of a predicate the
/// queries and the evidence reach, or in a query rule. tddlog would read
/// such a predicate as false: a misspelt name, or a built-in of ProbLog's
/// it does not evaluate (`write/1`, `call/1`, `select/3`), would give a
/// wrong probability. ProbLog stops where its resolution reaches the call,
/// so it also answers a program whose call comes after a literal that
/// never holds, or in a clause whose head unifies with no call; tddlog
/// refuses those too.
fn check_defined(program: &Program, defined: &ByHead) -> Result<(), GroundError> {
    let mut reached: FxHashSet<(&str, usize)> = FxHashSet::default();
    let mut stack: Vec<(&str, usize)> = Vec::new();
    let asked = [("query", &program.queries), ("evidence", &program.evidence)]
        .into_iter()
        .flat_map(|(what, atoms)| atoms.iter().map(move |(a, positive)| (what, a, *positive)))
        .chain(
            program
                .query_rules
                .iter()
                .map(|r| ("query", &r.pattern, r.positive)),
        );
    for (what, a, positive) in asked {
        let sig = a.signature();
        if !defined.contains_key(&sig) {
            let sign = if positive { "" } else { "\\+" };
            let body = match Builtin::of(sig.0, sig.1) {
                Some(_) => " (tddlog evaluates this built-in in a clause's body only)",
                None => "",
            };
            return Err(GroundError::NoClauses(format!(
                "{}/{}, asked in {what}({sign}{a}){body}",
                sig.0, sig.1
            )));
        }
        if reached.insert(sig) {
            stack.push(sig);
        }
    }
    let called = |a: &Atom, at: &dyn std::fmt::Display| {
        let (name, arity) = a.signature();
        GroundError::NoClauses(format!(
            "{name}/{arity}, called in {at}, nor is it a built-in tddlog evaluates"
        ))
    };
    for r in &program.query_rules {
        if let Some(a) = first_undefined(&r.body, defined, &mut reached, &mut stack) {
            return Err(called(a, &format!("a query rule for {}", r.pattern)));
        }
    }
    while let Some(sig) = stack.pop() {
        for c in defined.get(&sig).into_iter().flatten() {
            if let Some(a) = first_undefined(&c.body, defined, &mut reached, &mut stack) {
                return Err(called(a, c));
            }
        }
    }
    Ok(())
}

/// The first atom `body` calls whose predicate has no clauses and is not a
/// built-in tddlog evaluates; each predicate with clauses that it calls is
/// pushed on `stack` the first time it is `reached`.
fn first_undefined<'p>(
    body: &'p [Literal],
    defined: &ByHead<'p>,
    reached: &mut FxHashSet<(&'p str, usize)>,
    stack: &mut Vec<(&'p str, usize)>,
) -> Option<&'p Atom> {
    let mut missing = None;
    calls(body, &mut |a| {
        let sig = a.signature();
        if defined.contains_key(&sig) {
            if reached.insert(sig) {
                stack.push(sig);
            }
        } else if missing.is_none() && Builtin::of(sig.0, sig.1).is_none() {
            missing = Some(a);
        }
    });
    missing
}

/// Spreads each argument of a predicate that is, in every head and every
/// call, a compound of one name and arity other than a list cell into the
/// compound's arguments: `s(L, c(N, NN))` is read as `s(L, N, NN)`. A
/// demand then binds the parts its call binds, a count passed in where
/// another is passed out, which a demand of the whole compound does not, so
/// the predicate is ground for the counts its calls give, as ProbLog's
/// resolution reaches them, rather than for every count. A predicate that
/// is asked keeps its arguments, as does one whose spread arity names
/// another predicate or a built-in. Repeated while a spread argument holds
/// another such compound.
fn spread(program: &mut Program) {
    loop {
        let plan = spread_plan(program);
        if plan.is_empty() {
            return;
        }
        let respread = |a: &mut Atom| {
            if let Some(widths) = plan.get(&(a.name.clone(), a.args.len())) {
                let mut args = Vec::with_capacity(a.args.len());
                for (t, width) in a.args.iter().zip(widths) {
                    match (width, t.as_compound()) {
                        (Some(_), Some((_, parts))) => args.extend(parts.iter().cloned()),
                        _ => args.push(t.clone()),
                    }
                }
                *a = Atom::new(a.name.clone(), args);
            }
        };
        for c in &mut program.clauses {
            for h in &mut c.heads {
                respread(&mut h.atom);
            }
            calls_mut(&mut c.body, &respread);
        }
        for r in &mut program.query_rules {
            calls_mut(&mut r.body, &respread);
        }
    }
}

/// The predicates [`spread`] spreads, each with the arity of the compound
/// at each argument it spreads (`None` at one it keeps).
fn spread_plan(program: &Program) -> FxHashMap<Pred, Vec<Option<usize>>> {
    // Per predicate, per argument: the name and arity of the compound every
    // occurrence so far has there, `None` once one differs.
    let mut shapes: FxHashMap<Pred, Vec<Option<(Name, usize)>>> = FxHashMap::default();
    let mut note = |a: &Atom| {
        let shape = |t: &Term| {
            t.as_compound()
                .filter(|(name, _)| name.as_ref() != CONS)
                .map(|(name, parts)| (name.clone(), parts.len()))
        };
        let seen = shapes
            .entry((a.name.clone(), a.args.len()))
            .or_insert_with(|| a.args.iter().map(shape).collect());
        for (s, t) in seen.iter_mut().zip(a.args.iter()) {
            if s.is_some() && *s != shape(t) {
                *s = None;
            }
        }
    };
    let mut headed = FxHashSet::default();
    for c in &program.clauses {
        for h in &c.heads {
            note(&h.atom);
            headed.insert((h.atom.name.clone(), h.atom.args.len()));
        }
        calls(&c.body, &mut note);
    }
    for r in &program.query_rules {
        calls(&r.body, &mut note);
    }
    let asked: FxHashSet<Pred> = program
        .queries
        .iter()
        .chain(&program.evidence)
        .map(|(a, _)| a)
        .chain(program.query_rules.iter().map(|r| &r.pattern))
        .map(|a| (a.name.clone(), a.args.len()))
        .collect();
    let mut taken: FxHashSet<Pred> = shapes
        .keys()
        .cloned()
        .chain(asked.iter().cloned())
        .collect();
    let mut preds: Vec<&Pred> = shapes.keys().collect();
    preds.sort();
    let mut plan = FxHashMap::default();
    for pred in preds {
        let at = &shapes[pred];
        if !headed.contains(pred) || asked.contains(pred) || at.iter().all(Option::is_none) {
            continue;
        }
        let arity = at.iter().map(|s| s.as_ref().map_or(1, |(_, k)| *k)).sum();
        let (name, old) = pred;
        if arity != *old
            && (Builtin::of(name, arity).is_some() || !taken.insert((name.clone(), arity)))
        {
            continue;
        }
        plan.insert(
            pred.clone(),
            at.iter().map(|s| s.as_ref().map(|(_, k)| *k)).collect(),
        );
    }
    plan
}

/// Each atom `body` calls, in findalls' goals too.
fn calls<'a>(body: &'a [Literal], f: &mut impl FnMut(&'a Atom)) {
    for l in body {
        match l {
            Literal::Pos(a) | Literal::Neg(a) => f(a),
            Literal::Findall(fa) => {
                for conj in &fa.goal {
                    calls(conj, f);
                }
            }
            _ => {}
        }
    }
}

/// [`calls`], each atom changed by `f`.
fn calls_mut(body: &mut [Literal], f: &impl Fn(&mut Atom)) {
    for l in body {
        match l {
            Literal::Pos(a) | Literal::Neg(a) => f(a),
            Literal::Findall(fa) => {
                for conj in &mut fa.goal {
                    calls_mut(conj, f);
                }
            }
            _ => {}
        }
    }
}

struct Lowering<'a, 'p> {
    defined: &'a ByHead<'p>,
    /// Fresh variables made, and auxiliary predicates.
    fresh: usize,
    aux: usize,
    out: Vec<Clause>,
    /// The auxiliary predicates of the findalls, the roots of the cones.
    cone_roots: Vec<Pred>,
}

impl Lowering<'_, '_> {
    /// The clauses of `work`, a stack, lowered onto `out`, each followed by
    /// the auxiliary clauses of its findalls in their order.
    fn run(&mut self, work: &mut Vec<Clause>) -> Result<(), GroundError> {
        while let Some(clause) = work.pop() {
            let mut aux = Vec::new();
            if let Some(c) = self.clause(clause, &mut aux)? {
                self.out.push(c);
            }
            work.extend(aux.into_iter().rev());
        }
        Ok(())
    }

    fn var(&mut self) -> Term {
        self.fresh += 1;
        Term::Var(format!("$V{}", self.fresh))
    }

    /// `t` with its compound terms with variables replaced by fresh
    /// variables, whose unifications with the terms, taken one level at a
    /// time, are pushed on `out`.
    fn flatten(&mut self, t: &Term, out: &mut Vec<Literal>) -> Term {
        match t.as_compound() {
            Some((name, args)) if !t.is_ground() => {
                let args: Vec<Term> = args.iter().map(|a| self.flatten(a, out)).collect();
                let v = self.var();
                out.push(Literal::Cmp(
                    CmpOp::Eq,
                    v.clone(),
                    Compound::term(name.clone(), args),
                ));
                v
            }
            _ => t.clone(),
        }
    }

    /// The unification of `a` and `b`, pushed on `out` as unifications of
    /// a flat term with a flat term or with a compound of flat arguments;
    /// false where it cannot hold.
    fn unify(&mut self, a: &Term, b: &Term, out: &mut Vec<Literal>) -> bool {
        match (a.as_compound(), b.as_compound()) {
            _ if a.is_ground()
                && b.is_ground()
                && (a.as_compound().is_some() || b.as_compound().is_some()) =>
            {
                a == b
            }
            (Some((f, xs)), Some((g, ys))) => {
                f == g
                    && xs.len() == ys.len()
                    && xs.iter().zip(ys).all(|(x, y)| self.unify(x, y, out))
            }
            (None, Some(_)) | (Some(_), None) => {
                let (flat, c) = if a.as_compound().is_none() {
                    (a, b)
                } else {
                    (b, a)
                };
                if !flat.is_var() {
                    return false;
                }
                let (name, args) = c.as_compound().expect("a compound");
                let args: Vec<Term> = args.iter().map(|x| self.flatten(x, out)).collect();
                out.push(Literal::Cmp(
                    CmpOp::Eq,
                    flat.clone(),
                    Compound::term(name.clone(), args),
                ));
                true
            }
            (None, None) => {
                out.push(Literal::Cmp(CmpOp::Eq, a.clone(), b.clone()));
                true
            }
        }
    }

    /// The atom `a` with its compound arguments flattened onto `out`.
    fn flat_atom(&mut self, a: Atom, out: &mut Vec<Literal>) -> Atom {
        if a.args.iter().all(|t| t.is_ground()) {
            return a;
        }
        let args: Vec<Term> = a.args.iter().map(|t| self.flatten(t, out)).collect();
        Atom::new(a.name, args)
    }

    /// Each negated atom of `body` with variables of its own, read by no
    /// other literal and not in `outside`, replaced by the negation of an
    /// auxiliary predicate over its other variables, whose clause, the atom,
    /// is pushed on `work`: ProbLog calls the atom with those variables
    /// unbound, and the grounder reads a negation whose variables are bound.
    fn project(&mut self, body: &mut [Literal], outside: &[String], work: &mut Vec<Clause>) {
        for i in 0..body.len() {
            let own = locals(body, i, outside);
            let Literal::Neg(a) = &mut body[i] else {
                continue;
            };
            if own.is_empty() {
                continue;
            }
            let mut vs = Vec::new();
            a.vars_into(&mut vs);
            vs.retain(|v| !own.contains(v));
            self.aux += 1;
            let head = Atom::new(
                format!("{NEGATION_PREFIX}{}", self.aux),
                vs.into_iter().map(Term::Var).collect::<Vec<_>>(),
            );
            let atom = std::mem::replace(a, head.clone());
            work.push(Clause::rule(head, vec![Literal::Pos(atom)], u32::MAX));
        }
    }

    /// `clause` lowered, with the auxiliary clauses of its findalls and
    /// negations pushed on `work`; `None` where it cannot hold.
    fn clause(
        &mut self,
        mut clause: Clause,
        work: &mut Vec<Clause>,
    ) -> Result<Option<Clause>, GroundError> {
        let outside = atom_vars(clause.heads.iter().map(|h| &h.atom));
        self.project(&mut clause.body, &outside, work);
        // The heads' compounds are taken apart before the body, under a
        // demand that binds them, and built after it otherwise: the
        // schedule places each unification where it can run.
        let mut front = Vec::new();
        for h in &mut clause.heads {
            let atom = std::mem::take(&mut h.atom);
            h.atom = self.flat_atom(atom, &mut front);
        }
        let head_vars: Vec<String> = {
            let mut vs = Vec::new();
            for h in &clause.heads {
                h.atom.vars_into(&mut vs);
            }
            vs
        };
        let Some(body) = self.body(std::mem::take(&mut clause.body), &head_vars, work)? else {
            return Ok(None);
        };
        front.extend(body);
        clause.body = front;
        Ok(Some(clause))
    }

    /// `body` lowered, `None` where it cannot hold; the auxiliary clauses of
    /// its findalls pushed on `work`. `head` holds the variables of the
    /// clause's heads.
    fn body(
        &mut self,
        body: Vec<Literal>,
        head: &[String],
        work: &mut Vec<Clause>,
    ) -> Result<Option<Vec<Literal>>, GroundError> {
        let mut out = Vec::with_capacity(body.len());
        let n = body.len();
        // The variables of the literals before each.
        let mut before: Vec<String> = head.to_vec();
        let vars: Vec<Vec<String>> = body
            .iter()
            .map(|l| {
                let mut vs = Vec::new();
                binds(l, &mut vs);
                vs
            })
            .collect();
        for (i, l) in body.into_iter().enumerate() {
            match l {
                Literal::Pos(a) => match builtin(&a, self.defined) {
                    Some(b) => {
                        let args: Vec<Term> =
                            a.args.iter().map(|t| self.flatten(t, &mut out)).collect();
                        out.push(Literal::Builtin(b, args, false));
                    }
                    None => {
                        let a = self.flat_atom(a, &mut out);
                        out.push(Literal::Pos(a));
                    }
                },
                Literal::Neg(a) => {
                    let mut pre = Vec::new();
                    let lit = match builtin(&a, self.defined) {
                        Some(b) => {
                            let args = a.args.iter().map(|t| self.flatten(t, &mut pre)).collect();
                            Literal::Builtin(b, args, true)
                        }
                        None => Literal::Neg(self.flat_atom(a, &mut pre)),
                    };
                    out.extend(pre);
                    out.push(lit);
                }
                Literal::Cmp(CmpOp::Eq, a, b) => {
                    if !self.unify(&a, &b, &mut out) {
                        return Ok(None);
                    }
                }
                Literal::Cmp(op, a, b) if !op.arithmetic() => {
                    let a = self.flatten(&a, &mut out);
                    let b = self.flatten(&b, &mut out);
                    out.push(Literal::Cmp(op, a, b));
                }
                Literal::Findall(fa) => {
                    let after: Vec<String> = vars[i + 1..n].iter().flatten().cloned().collect();
                    let mut collect = self.findall(*fa, &before, &after, work)?;
                    // A result pattern is matched against the list after it.
                    let pattern =
                        matches!(collect.result, Term::Compound(_)) && !collect.result.is_ground();
                    let result = if pattern {
                        std::mem::replace(&mut collect.result, self.var())
                    } else {
                        collect.result.clone()
                    };
                    let r = collect.result.clone();
                    out.push(Literal::Collect(Box::new(collect)));
                    if pattern && !self.unify(&r, &result, &mut out) {
                        return Ok(None);
                    }
                }
                other => out.push(other),
            }
            for v in &vars[i] {
                if !before.contains(v) {
                    before.push(v.clone());
                }
            }
        }
        Ok(Some(out))
    }

    /// The [`Collect`] of `fa`, its auxiliary clauses pushed on `work`.
    /// `before` holds the variables of the heads and of the literals before
    /// it, `after` those of the literals after it.
    fn findall(
        &mut self,
        fa: Findall,
        before: &[String],
        after: &[String],
        work: &mut Vec<Clause>,
    ) -> Result<Collect, GroundError> {
        let text: Arc<str> = Literal::Findall(Box::new(fa.clone())).to_string().into();
        let refuse = |why: String| Err(GroundError::Refused(format!("{text}: {why}")));
        if fa.interleaved {
            return refuse(
                "a disjunction after another literal of the goal, whose solutions ProbLog lists in another order"
                    .into(),
            );
        }
        // The goal's variables, and those its literals bind: a nested
        // findall's other variables stay its own.
        let (mut goal_vars, mut bound) = (Vec::new(), Vec::new());
        for l in fa.goal.iter().flatten() {
            l.vars_into(&mut goal_vars);
            binds(l, &mut bound);
        }
        let mut template_vars = Vec::new();
        fa.template.vars_into(&mut template_vars);
        let outside = |v: &String| before.contains(v) || after.contains(v);
        let mut shared: Vec<String> = Vec::new();
        for v in goal_vars.iter().chain(&template_vars) {
            if outside(v) && !shared.contains(v) {
                shared.push(v.clone());
            }
        }
        if let Some(v) = shared.iter().find(|v| !before.contains(v)) {
            return refuse(format!(
                "{v} is bound after the findall, which ProbLog calls with it unbound"
            ));
        }
        if let Some(v) = template_vars
            .iter()
            .find(|v| !bound.contains(v) && !shared.contains(v))
        {
            return refuse(format!("the template's {v} is not bound by the goal"));
        }
        let locals: Vec<String> = bound
            .iter()
            .filter(|v| !shared.contains(v) && !template_vars.contains(v))
            .cloned()
            .collect();
        self.aux += 1;
        let aux: Name = format!("{FINDALL_PREFIX}{}", self.aux).into();
        let mut head_args: Vec<Term> = shared.iter().cloned().map(Term::Var).collect();
        head_args.push(fa.template.clone());
        head_args.extend(locals.iter().cloned().map(Term::Var));
        let arity = head_args.len();
        let head = Atom::new(aux.clone(), head_args);
        let none: Term = Term::Const("$none".into());
        for alt in fa.goal {
            let mut vs = Vec::new();
            for l in &alt {
                binds(l, &mut vs);
            }
            if let Some(v) = template_vars
                .iter()
                .find(|v| !vs.contains(v) && !shared.contains(v))
            {
                return refuse(format!(
                    "the template's {v} is not bound by every alternative"
                ));
            }
            // A variable of another alternative is no variable of this one.
            let mut body = alt;
            for v in locals.iter().filter(|v| !vs.contains(v)) {
                body.push(Literal::Cmp(CmpOp::Eq, Term::Var(v.clone()), none.clone()));
            }
            work.push(Clause::rule(head.clone(), body, 0));
        }
        self.cone_roots.push((aux.clone(), arity));
        Ok(Collect {
            aux,
            arity,
            shared: shared.into_iter().map(Term::Var).collect(),
            result: fa.result,
            text,
        })
    }
}

/// The predicates the findalls of `program` read, from `roots`, checked and
/// their clauses given groups of their own in the order of the source.
fn cone(program: &mut Program, roots: &[Pred]) -> Result<FxHashSet<Pred>, GroundError> {
    if roots.is_empty() {
        return Ok(FxHashSet::default());
    }
    let mut ids: FxHashMap<Pred, usize> = FxHashMap::default();
    let id = |p: Pred, ids: &mut FxHashMap<Pred, usize>| {
        let n = ids.len();
        *ids.entry(p).or_insert(n)
    };
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for c in &program.clauses {
        for h in &c.heads {
            let from = id((h.atom.name.clone(), h.atom.args.len()), &mut ids);
            for l in &c.body {
                let to = match l {
                    Literal::Pos(a) | Literal::Neg(a) => (a.name.clone(), a.args.len()),
                    Literal::Collect(k) => (k.aux.clone(), k.arity),
                    _ => continue,
                };
                let to = id(to, &mut ids);
                edges.push((from, to));
            }
        }
    }
    let n = ids.len();
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); n];
    for &(a, b) in &edges {
        out[a].push(b);
    }
    let mut reached = vec![false; n];
    let mut stack: Vec<usize> = roots.iter().map(|r| ids[r]).collect();
    while let Some(p) = stack.pop() {
        if !std::mem::replace(&mut reached[p], true) {
            stack.extend(out[p].iter().copied());
        }
    }
    let names: Vec<&Pred> = {
        let mut v = vec![None; n];
        for (p, &i) in &ids {
            v[i] = Some(p);
        }
        v.into_iter().map(|p| p.expect("a predicate")).collect()
    };
    let mut cyclic: Option<usize> = None;
    super::deps::tarjan(
        n,
        0..n,
        |v, (k, _)| {
            let w = *out[v].get(*k)?;
            *k += 1;
            Some(w)
        },
        |comp| {
            if cyclic.is_none()
                && comp.iter().any(|&p| reached[p])
                && (comp.len() > 1 || out[comp[0]].contains(&comp[0]))
            {
                cyclic = Some(comp[0]);
            }
        },
    );
    if let Some(p) = cyclic {
        let (name, arity) = names[p];
        return Err(GroundError::Refused(format!(
            "findall over a recursive goal: {name}/{arity} is recursive or reads the findall's own clause"
        )));
    }
    let cone: FxHashSet<Pred> = (0..n)
        .filter(|&p| reached[p])
        .map(|p| names[p].clone())
        .collect();
    let mut next = program.clauses.iter().map(|c| c.group).max().unwrap_or(0) + 1;
    for c in &mut program.clauses {
        let sig = (c.heads[0].atom.name.clone(), c.heads[0].atom.args.len());
        if !cone.contains(&sig) {
            continue;
        }
        // An auxiliary clause is the findall's, whose goal is checked, and
        // has no group of its own yet.
        if !sig.0.starts_with(FINDALL_PREFIX) && program.interleaved.contains(&c.group) {
            return Err(GroundError::Refused(format!(
                "findall over {}/{}, whose clause has a disjunction after another literal, which ProbLog lists in another order",
                sig.0, sig.1
            )));
        }
        if c.heads.iter().all(|h| h.prob.is_none()) {
            c.group = next;
            next += 1;
        }
    }
    Ok(cone)
}

/// The variables of `atoms`.
fn atom_vars<'a>(atoms: impl IntoIterator<Item = &'a Atom>) -> Vec<String> {
    let mut vs = Vec::new();
    for a in atoms {
        a.vars_into(&mut vs);
    }
    vs
}

/// The variables of `body[i]`, where it is a negated atom, that neither
/// `outside` nor another literal of `body` reads.
fn locals(body: &[Literal], i: usize, outside: &[String]) -> Vec<String> {
    let mut vs = Vec::new();
    if let Literal::Neg(a) = &body[i] {
        a.vars_into(&mut vs);
    }
    if vs.is_empty() {
        return vs;
    }
    let mut elsewhere = outside.to_vec();
    for (j, l) in body.iter().enumerate() {
        if j != i {
            l.vars_into(&mut elsewhere);
        }
    }
    vs.retain(|v| !elsewhere.contains(v));
    vs
}

/// Whether a negated atom of `body` has variables of its own, which
/// neither `heads` nor another literal reads ([`Lowering::project`]).
fn projects<'a>(heads: impl IntoIterator<Item = &'a Atom>, body: &[Literal]) -> bool {
    let negated =
        |l: &Literal| matches!(l, Literal::Neg(a) if a.args.iter().any(|t| !t.is_ground()));
    if !body.iter().any(negated) {
        return false;
    }
    let outside = atom_vars(heads);
    (0..body.len()).any(|i| !locals(body, i, &outside).is_empty())
}

/// The variables `l` binds where it holds, added to `out`: a findall's are
/// its result's, the others of its template and goal staying unbound.
fn binds(l: &Literal, out: &mut Vec<String>) {
    match l {
        Literal::Findall(f) => f.result.vars_into(out),
        _ => l.vars_into(out),
    }
}
