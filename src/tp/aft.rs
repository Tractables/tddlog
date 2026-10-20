//! Well-founded compilation of a recursive component with internal negation
//! (Bogaerts & Van den Broeck, ICLP 2015).
//!
//! Each atom carries a pair `(t, p)` of formulas: `t` entails the atom is
//! true, `p` that it is not false. Bodies are evaluated three-valued, with
//! negation swapping the pair: `(¬b)^S = (¬p_b, ¬t_b)`. Starting from `(⊥, ⊤)`
//! the component is refined by applying the operator until it stabilises, then
//! by an *unfoundedness refinement*: the upper bound is recomputed as a least
//! fixpoint from `⊥` with the lower bound frozen. Alternating the two until
//! neither changes anything yields the parametrised well-founded model. For a
//! sound ProbLog program (two-valued well-founded model in every world) the
//! two bounds coincide and give the atom's formula.

use tididi::Tdd;
use tididi::vtree::VarId;

use super::{CompileError, Reduce, TddBackend, VtreeScope};
use crate::ground::{AtomId, AtomKind, GLit, GroundProgram, RulesByHead};

/// Compile the atoms of `scc` on `scope`. `lambda_of` returns the final
/// formula of a derived atom outside the component, already on `scope`.
/// Returns each atom's formula in the order of `scc`.
///
/// One component, one scope: every atom of a cycle sees the same facts, so
/// nothing inside this fixpoint is embedded. Every negation goes through
/// `negate`, which counts it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_component(
    gp: &GroundProgram,
    by_head: &RulesByHead,
    var_of: &[Option<VarId>],
    scc: &[AtomId],
    scope: &VtreeScope,
    lambda_of: &mut dyn FnMut(AtomId) -> Result<Tdd, CompileError>,
    negate: &dyn Fn(Tdd) -> Result<Tdd, CompileError>,
    updates: &mut usize,
    backend: &TddBackend,
) -> Result<Vec<Tdd>, CompileError> {
    let n = gp.num_atoms();
    let mut index = vec![usize::MAX; n];
    for (i, &a) in scc.iter().enumerate() {
        index[a as usize] = i;
    }
    // Formulas of literals over atoms outside the component are fixed.
    let mut outside: Vec<Option<Tdd>> = (0..n).map(|_| None).collect();
    for &a in scc {
        for &ri in &by_head[a as usize] {
            for l in &gp.rules[ri].body {
                let b = l.atom() as usize;
                if gp.kinds[b] == AtomKind::Derived
                    && index[b] == usize::MAX
                    && outside[b].is_none()
                {
                    outside[b] = Some(lambda_of(l.atom())?);
                }
            }
        }
    }
    let lit_pair = |l: GLit, lower: &[Tdd], upper: &[Tdd]| -> Result<(Tdd, Tdd), CompileError> {
        let b = l.atom() as usize;
        let (t, p) = match gp.kinds[b] {
            AtomKind::Fact(_) => {
                let v = var_of[b].unwrap();
                let f = backend.literal(scope, tididi::Literal::pos(v))?;
                (backend.copy(&f)?, f)
            }
            AtomKind::Derived if index[b] != usize::MAX => (
                backend.copy(&lower[index[b]])?,
                backend.copy(&upper[index[b]])?,
            ),
            AtomKind::Derived => {
                let f = outside[b]
                    .as_ref()
                    .expect("an atom outside the component has its formula");
                (backend.copy(f)?, backend.copy(f)?)
            }
        };
        if l.is_positive() {
            Ok((t, p))
        } else {
            Ok((negate(p)?, negate(t)?))
        }
    };
    // One application of the operator to atom `a`, both components.
    let apply = |a: AtomId, lower: &[Tdd], upper: &[Tdd]| -> Result<(Tdd, Tdd), CompileError> {
        let mut t_acc = backend.zero(scope);
        let mut p_acc = backend.zero(scope);
        for body in by_head[a as usize]
            .iter()
            .flat_map(|&ri| gp.rules[ri].bodies())
        {
            let mut bt = backend.one(scope);
            let mut bp = backend.one(scope);
            for &l in body.iter() {
                let (vt, vp) = lit_pair(l, lower, upper)?;
                bt = backend.and(bt, vt)?;
                bp = backend.and(bp, vp)?;
                if backend.is_zero(&bp) {
                    break;
                }
            }
            t_acc = backend.or(t_acc, bt)?;
            p_acc = backend.or(p_acc, bp)?;
        }
        backend.minimize(&mut t_acc, Reduce::Between)?;
        backend.minimize(&mut p_acc, Reduce::Between)?;
        Ok((t_acc, p_acc))
    };

    let k = scc.len();
    let mut lower: Vec<Tdd> = (0..k).map(|_| backend.zero(scope)).collect();
    let mut upper: Vec<Tdd> = (0..k).map(|_| backend.one(scope)).collect();
    loop {
        // Application refinements, one atom at a time, until stable.
        loop {
            let mut changed = false;
            for (i, &a) in scc.iter().enumerate() {
                let (t, p) = apply(a, &lower, &upper)?;
                *updates += 1;
                if !backend.equivalent(&t, &lower[i])? {
                    lower[i] = t;
                    changed = true;
                }
                if !backend.equivalent(&p, &upper[i])? {
                    upper[i] = p;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        // Unfoundedness refinement: least fixpoint of the upper component
        // from ⊥ with the lower component frozen.
        let mut u: Vec<Tdd> = (0..k).map(|_| backend.zero(scope)).collect();
        loop {
            let mut changed = false;
            for (i, &a) in scc.iter().enumerate() {
                let (_, p) = apply(a, &lower, &u)?;
                *updates += 1;
                if !backend.equivalent(&p, &u[i])? {
                    u[i] = p;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        let mut refined = false;
        for i in 0..k {
            if !backend.equivalent(&u[i], &upper[i])? {
                refined = true;
            }
        }
        if !refined {
            break;
        }
        upper = u;
    }
    for i in 0..k {
        if !backend.equivalent(&lower[i], &upper[i])? {
            return Err(CompileError::Unsound(gp.atoms[scc[i] as usize].clone()));
        }
    }
    Ok(lower)
}
