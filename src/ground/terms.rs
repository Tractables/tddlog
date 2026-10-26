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
//!   shared variables' values.
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
    Atom, Builtin, Clause, CmpOp, Collect, Compound, Findall, Literal, Name, Program, Term,
};

/// The prefix of the auxiliary predicates of findalls.
pub(crate) const FINDALL_PREFIX: &str = "$fa";

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
            a.args.iter().any(compound)
                || Builtin::of(&a.name, a.args.len()).is_some()
                || unevaluated(&a.name, a.args.len())
        }
        Literal::Cmp(op, a, b) => {
            !op.arithmetic() && (matches!(a, Term::Compound(_)) || matches!(b, Term::Compound(_)))
        }
        Literal::Findall(_) | Literal::Builtin(..) | Literal::Collect(_) => true,
        Literal::Is(..) => false,
    };
    if !program.clauses.iter().any(|c| {
        c.heads.iter().any(|h| h.atom.args.iter().any(compound))
            || c.body.iter().any(lowers)
    }) && !program.query_rules.iter().any(|r| r.body.iter().any(lowers))
    {
        return Ok((Cow::Borrowed(program), FxHashSet::default()));
    }
    let defined: FxHashSet<(&str, usize)> = program
        .clauses
        .iter()
        .flat_map(|c| c.heads.iter().map(|h| h.atom.signature()))
        .collect();
    let mut l = Lowering {
        defined: &defined,
        fresh: 0,
        aux: 0,
        out: Vec::with_capacity(program.clauses.len()),
        cone_roots: Vec::new(),
    };
    let mut lowered = program.clone();
    // The clauses in order, each followed by the auxiliary clauses of its
    // findalls.
    let mut work: Vec<Clause> = std::mem::take(&mut lowered.clauses);
    work.reverse();
    l.run(&mut work)?;
    let mut query_rules = Vec::with_capacity(lowered.query_rules.len());
    for mut r in std::mem::take(&mut lowered.query_rules) {
        let mut extra = Vec::new();
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
fn builtin(a: &Atom, defined: &FxHashSet<(&str, usize)>) -> Option<Builtin> {
    let (name, arity) = a.signature();
    Builtin::of(name, arity).filter(|_| !defined.contains(&(name, arity)))
}

/// Whether `name/arity` is one of ProbLog's built-ins or of its
/// library(lists) that tddlog does not evaluate.
fn unevaluated(name: &str, arity: usize) -> bool {
    matches!(
        (name, arity),
        ("call" | "call_nc" | "try_call", 1..=9)
            | ("nl", 0)
            | (
                "once" | "callable" | "atom" | "atomic" | "compound" | "float" | "integer"
                    | "number" | "var" | "nonvar" | "ground" | "is_list" | "is_set" | "rational"
                    | "simple" | "primitive" | "dbreference" | "unknown" | "possible"
                    | "cmd_args" | "consult" | "use_module" | "unzip",
                1
            )
            | (
                "=.." | "succ" | "forall" | "atom_number" | "clause" | "subquery" | "module"
                    | "numbervars" | "varnumbers" | "subsumes_term" | "subsumes_chk"
                    | "create_scope" | "call_in_scope" | "find_scope" | "nocache" | "use_module"
                    | "append" | "flatten" | "groupby" | "head" | "list_to_set" | "max_member"
                    | "min_member" | "permutation" | "prefix" | "proper_length" | "same_length"
                    | "subset" | "memberchk_search",
                2
            )
            | (
                "arg" | "functor" | "plus" | "compare" | "all" | "all_or_none" | "numbervars"
                    | "sample_uniform1" | "clause" | "subquery" | "subquery_in_scope" | "delete"
                    | "flatten" | "intersection" | "make_list" | "max_member" | "min_member"
                    | "nextto" | "numlist" | "select" | "selectchk" | "subtract" | "union" | "unzip"
                    | "zip" | "max_list" | "min_list" | "reverse" | "sum_list",
                3
            )
            | (
                "subquery_in_scope" | "select" | "selectchk" | "select_uniform"
                    | "select_weighted" | "groupby",
                4
            )
            | ("subquery" | "select_weighted" | "sub_list", 5)
            | ("subquery_in_scope" | "sub_list" | "sw", 6)
    )
}

/// Refuses a call of one of ProbLog's built-ins that tddlog does not
/// evaluate (call/1, plus/3, select/3, ...), where the program defines no
/// predicate of its name and arity: tddlog would read it as a predicate
/// without clauses, false, where ProbLog evaluates it.
fn evaluated(a: &Atom, defined: &FxHashSet<(&str, usize)>) -> Result<(), GroundError> {
    let (name, arity) = a.signature();
    if unevaluated(name, arity) && !defined.contains(&(name, arity)) {
        return Err(GroundError::Unsupported(format!(
            "{name}/{arity} is a built-in of ProbLog's that tddlog does not evaluate, in {a}"
        )));
    }
    Ok(())
}

struct Lowering<'a, 'p> {
    defined: &'a FxHashSet<(&'p str, usize)>,
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
            _ if a.is_ground() && b.is_ground() && (a.as_compound().is_some() || b.as_compound().is_some()) => {
                a == b
            }
            (Some((f, xs)), Some((g, ys))) => {
                f == g
                    && xs.len() == ys.len()
                    && xs.iter().zip(ys).all(|(x, y)| self.unify(x, y, out))
            }
            (None, Some(_)) | (Some(_), None) => {
                let (flat, c) = if a.as_compound().is_none() { (a, b) } else { (b, a) };
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

    /// `clause` lowered, with the auxiliary clauses of its findalls pushed
    /// on `work`; `None` where it cannot hold.
    fn clause(&mut self, mut clause: Clause, work: &mut Vec<Clause>) -> Result<Option<Clause>, GroundError> {
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
            if let Literal::Pos(a) | Literal::Neg(a) = &l {
                evaluated(a, self.defined)?;
            }
            match l {
                Literal::Pos(a) => match builtin(&a, self.defined) {
                    Some(b) => {
                        let args: Vec<Term> = a.args.iter().map(|t| self.flatten(t, &mut out)).collect();
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
                    let pattern = matches!(collect.result, Term::Compound(_))
                        && !collect.result.is_ground();
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
                return refuse(format!("the template's {v} is not bound by every alternative"));
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
    let cone: FxHashSet<Pred> = (0..n).filter(|&p| reached[p]).map(|p| names[p].clone()).collect();
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

/// The variables `l` binds where it holds, added to `out`: a findall's are
/// its result's, the others of its template and goal staying unbound.
fn binds(l: &Literal, out: &mut Vec<String>) {
    match l {
        Literal::Findall(f) => f.result.vars_into(out),
        _ => l.vars_into(out),
    }
}
