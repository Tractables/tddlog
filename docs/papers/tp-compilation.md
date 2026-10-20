# T_P-compilation — implementer's notes

Sources:

- **IJCAI15** — Vlasselaer, Van den Broeck, Kimmig, Meert, De Raedt, *Anytime Inference in
  Probabilistic Logic Programs with T_P-Compilation*, IJCAI 2015.
- **IJAR16** — same authors, *T_P-Compilation for Inference in Probabilistic Logic Programs*,
  Int. J. Approximate Reasoning 2016 (the journal version; IJCAI15's §4.3 becomes §5.1).
- **ICLP15** — Bogaerts & Van den Broeck, *Knowledge Compilation of Logic Programs Using
  Approximation Fixpoint Theory*, ICLP/TPLP 2015 (negation through recursion).
- **TPLP15** — Fierens et al., *Inference and Learning in PLP using Weighted Boolean Formulas*,
  to which both T_P papers delegate the relevant ground program and the weights
  ([fierens-tplp15.md](fierens-tplp15.md)).

Notation: `F` = ground probabilistic facts, `R` = rules, `A` = atoms of the relevant ground
program, `λ_a` = Boolean formula of atom `a` over `F`, `Λ_a` = its diagram (SDD in the papers, TDD
here). `src/tp/` implements §1 and §2; `src/tp/aft.rs` implements §2.2.

---

## 1. The Tc_P operator

### 1.1 What is computed

A total choice `C ⊆ F` fixes every fact, independently with probability `p_i`; `C ∪ R` has a
unique least / canonical / well-founded model. `Pr(q)` is the sum over the choices whose model
holds `q` (IJAR16 Eq. 1), i.e. `WMC(λ_q)` with `w(f) = p`, `w(¬f) = 1 − p`. Since every variable
has `w(x) + w(¬x) = 1`, a variable outside a formula's support contributes a factor 1 and no
normalisation is needed; a variable weighted 1 on both sides would break that.

Assumptions both papers state:

1. No probabilistic fact unifies with a rule head (facts and derived atoms are disjoint). A
   program that violates it is renamed: `p :: f'` and `f :- f'`.
2. `F` is ground (intensional facts `p::f(X) :- body` are expanded first).
3. Finite support: the queries depend on finitely many facts, so the fixpoint is reached.
4. Rules are range-restricted.

### 1.2 Parameterized interpretation and the operator (IJAR16 Defs. 3–5)

A parameterized interpretation is a set of pairs `(a, λ_a)`. `I⁰ = ∅` means every atom, every
probabilistic fact included, maps to `⊥`, with an implicit `(true, ⊤)`. Then

```
λ'_a = a                                                   if a ∈ F   (the variable, pinned)
λ'_a = ⋁_{(a :- b_1,...,b_n) ∈ P} (λ_{b_1} ∧ ... ∧ λ_{b_n})  if a ∈ A \ F
```

An atom without rules is the empty disjunction `⊥`; an empty body is `⊤`. Formulas are over `F`
only: no Tseitin variables, no variables for derived atoms. That is the method's point: the CNF
route on a complete 10-node graph has 26 995 variables and 109 899 clauses, Tc_P 90 edge variables.

**The fixpoint test is semantic**: `I` is a fixpoint iff `λ_a ≡ λ'_a` for every atom. With a
canonical diagram (for a fixed vtree) equivalence is pointer equality, so canonicity is
load-bearing, not a convenience.

### 1.3 Correctness (IJAR16 Thm. 1, Cor. 1, Thm. 2)

- After every iteration `i`, `C |= λ^i_a → C ∪ R |= a`, so `WMC(λ^i_a) ≤ Pr(a)`: an anytime
  lower bound.
- At the fixpoint `C ∪ R |= a ↔ C |= λ^∞_a`, for every atom at once: one fixpoint answers all
  queries.
- Conditional probabilities are defined at the fixpoint only:
  `Pr(q | e) = WMC(λ_q ∧ λ_e) / WMC(λ_e)` with `λ_e` the conjunction of the evidence atoms'
  formulas, negated for negative evidence. The ratio of two intermediate lower bounds bounds
  nothing.

### 1.4 Operations needed

Conjoin, disjoin, negate (with negation only), the equivalence test (free under canonicity), WMC,
and size. `condition` only for the papers' dynamic models; `forget`/`exists` never. The papers do
not say so, but every implementation wants semi-naive evaluation: recompute a rule's conjunction
only when one of its body formulas changed.

### 1.5 Running example (IJAR16 Examples 2, 4, 8, 9)

```prolog
0.4 :: edge(b,a).   0.3 :: edge(b,c).
0.8 :: edge(a,c).   0.9 :: edge(c,a).
path(X,Y) :- edge(X,Y).
path(X,Y) :- edge(X,Z), path(Z,Y).
```

`λ²_{p(b,c)} ≡ e(b,c)`, `λ³_{p(b,c)} ≡ e(b,c) ∨ (e(b,a) ∧ e(a,c))`, `λ³_{p(c,c)} ≡ e(c,a) ∧
e(a,c)`; the operator keeps adding longer acyclic edge sequences until a fixpoint. The plain `T_P`
trace: `ΔI¹` the four edges, `ΔI²` `p(b,a), p(b,c), p(a,c), p(c,a)`, `ΔI³` `p(a,a), p(c,c)`,
`ΔI⁴ = ∅`.

---

## 2. Negation

### 2.1 Stratified programs (IJAR16 §2.2, §3.2)

`P` is stratified if its rules partition into `P_1 ∪ ... ∪ P_m` with positive dependencies within
or below a stratum and negative dependencies strictly below. Compute `I_1 = Tc^∞_{P_1}(∅)`,
`I_2 = Tc^∞_{P_2}(I_1)`, ..., and take `λ_¬a = ¬λ_a`.

`λ_¬a = ¬λ_a` is sound only once `λ_a` has converged, i.e. `a`'s stratum is closed. Negating a
formula still growing turns the lower bound into an over-approximation. Strata are per rule, not
per predicate: in the sprinkler example `wetGrass` has rules in two strata.

### 2.2 General programs: approximation fixpoint theory (ICLP15)

Each defined atom carries a pair `(A_t, A_p)` of formulas over `F`, a lower bound ("certainly
true") and an upper bound ("possibly true"), with `A_t |= A_p`; start at `(⊥, ⊤)`. The
three-valued symbolic evaluation (ICLP15 Def. 4.4):

```
p^S     = (p, p)               p a probabilistic fact
p^S     = (A_t(p), A_p(p))      p defined
(ψ ∧ ξ)^S = (ψ_t ∧ ξ_t, ψ_p ∧ ξ_p)
(ψ ∨ ξ)^S = (ψ_t ∨ ξ_t, ψ_p ∨ ξ_p)
(¬ψ)^S  = (¬ψ_p, ¬ψ_t)          -- the swap: negate the other bound
```

`Ψ_P(S)(p)` evaluates `p`'s rule bodies this way (Fitting's operator, lifted). The
parametrised well-founded model is reached by refinements:

- **application refinement**: `S ← Ψ_P(S)`, repeated while it changes (cheap, taken first);
- **unfoundedness refinement**, when no application changes anything: recompute the upper bound
  as a least fixpoint from `⊥`, with the lower bound frozen (`U ← Ψ_P((S_t, U))_2` until stable),
  and set `S_p ← U`. Terminal when it changes nothing either.

```
CompileWFM(P):
  S_t[q] ← ⊥ ; S_p[q] ← ⊤  for every defined q
  loop:
    S' ← Ψ_P(S); if S' ≠ S: S ← S'; continue
    U ← ⊥; repeat U ← Ψ_P((S_t, U))_2 until stable
    if U ≡ S_p: return (S_t, S_p)
    S_p ← U
```

There are at most `|Σ_d|` consecutive application refinements and `|Σ_d|` unfoundedness
refinements, so `O(|Σ_d|²)` applications of `Ψ_P` bound the loop. ICLP15 uses Fitting's
approximator only; the ultimate approximation is cited, not developed.

For a sound ProbLog program (a two-valued well-founded model for every total choice, TPLP15 §3.2)
the result is exact, `A_t = A_p`. Where some choice has a three-valued model the pair stays apart
and there is no single `λ_a`. At any point `WMC(A_t(q)) ≤ Pr(q) ≤ WMC(A_p(q))` (ICLP15 Prop. 5.2).

| class | method |
|---|---|
| definite | plain least fixpoint |
| stratified | per-stratum fixpoints, `λ_¬a = ¬λ_a` |
| general, two-valued WFM for every choice | AFT pair, exact |
| general, three-valued WFM for some choice | AFT pair, not exact |
| stable models | not supported |

### 2.3 Test cases

- **Sprinkler** (IJAR16 Ex. 3, 5), two strata: `0.7::sprinklerOn. 0.2::cloudy. rain :- cloudy.
  wetGrass :- rain. sprinkler :- \+cloudy, sprinklerOn. wetGrass :- sprinkler.`
  `Pr(rain) = 0.2`, `Pr(sprinkler) = 0.56`, `Pr(wetGrass) = 0.76`.
- **Gear wheels** (ICLP15 Ex. 4.13; `tests/it/examples.rs` `gearwheels`), not stratified:
  `turns_1(T) ↔ turns_2(T)` loops, and `turns_i(1)` from `turns_i(0) ∧ ¬button_i(0)` or
  `¬turns_i(0) ∧ button_i(0)`. `(⊥, ⊤)` is already a fixpoint of `Ψ_P`, so an implementation with
  application refinements alone never moves; the unfoundedness refinement sets `turns_i(0)` to `⊥`.
  Result: `turns_i(0) = ⊥`, `turns_i(1) = button_1(0) ∨ button_2(0)`.
- **Not exact** (ICLP15 Ex. 4.15): `a ← ¬b. b ← ¬a. c ← ¬b. c ← e. d ← a ∧ ¬c.` with parameter
  `e`: `S(a) = S(b) = (⊥, ⊤)`, `S(c) = (e, ⊤)`, `S(d) = (⊥, ¬e)`.
- **Smokers** (ICLP15 Ex. 4.12), three people: `smokes(X) ← stress(X). smokes(X) ← fr(X,Y) ∧
  smokes(Y).` reaches its fixpoint in three applications.

---

## 3. Algorithms in the papers

- **Naive**: every atom updated from the previous interpretation until nothing changes.
- **One atom at a time** (IJAR16 §4.1): select an atom, apply the operator to its rules alone with
  the current formulas. Fair selection is the only convergence requirement; with negation,
  select within the current stratum.
- **Selection heuristic** (IJAR16 §4.2), for anytime use: the atom maximising the probability
  gained over the relative size growth, weighted by `φ_a`, the atom's least depth in the SLD tree
  of the queries. It compiles a candidate for every atom per step and keeps one, which leaves
  dead nodes in a shared manager. tddlog compiles to the fixpoint and does not use it.
- **Upper bound** (IJAR16 §4.3), definite programs only: a second fixpoint with the facts of SLD
  depth below `d` treated as certain; valid only at that fixpoint. ICLP15's pair gives two-sided
  bounds with negation as well.
- **Program updates** (IJAR16 §5.1), definite only: adding clauses restarts from the old fixpoint;
  removing clauses with heads `H` conjoins every formula with `⋀ ¬λ_h` and restarts.
- **Dynamic models** (IJAR16 §5.2): a first-order Markov transition model compiled once, then a
  forward message over the interface atoms per time step, using `condition`. The Mastermind and
  Sickness domains come from here.

### Evidence, queries and grounding

- All atoms are computed together; the queries only select the relevant ground program.
- Grounding is backward: tabled SLD resolution from `Q ∪ E` gives the relevant ground program
  (TPLP15 §5.1, Thm. 1: same `P(Q | E = e)`). Rules with a body literal false in the evidence are
  inactive and can be dropped; a body literal true in the evidence must be kept.
- All formulas live in one shared manager over `F`; SDD apply is polynomial per node pair, and
  compressed SDDs are canonical.

---

## 4. Vtrees

None of the three papers says anything about vtree choice or dynamic minimization. What follows
from them:

1. The variables are the probabilistic facts of the relevant ground program and nothing else, so
   the vtree can be built once, over `F`, before the compile.
2. One manager holds every atom's formula and they share structure, so the vtree must be good for
   all of them jointly, not for one query.
3. The only structural claim (IJAR16 §2.4): SDDs have size bounds in treewidth, tighter than the
   pathwidth bounds of OBDDs. A vtree from a low-width decomposition of the ground program's
   structure over `F` is what makes that bound apply.
4. Sizes in the papers are edges of the d-DNNF and 3 × edges of the SDD.
5. Under dynamic minimization, compare sizes under the same vtree, and minimize between
   iterations only.

---

## 5. Benchmarks and small cases

The papers' domains: Alzheimer (`path/2` on a gene network, 5 to 40 nodes), Smokers (power-law
friendship graphs, 5 to 30 people), Genes (500 connection queries), WebKB (class of a page from
the 100 most frequent words), and the dynamic Mastermind and Sickness. The Smokers program (TPLP15
§9.1):

```prolog
0.2::stress(P)        :- person(P).
0.3::influences(P1,P2):- friend(P1,P2).
0.1::cancer_spont(P)  :- person(P).
0.3::cancer_smoke(P)  :- person(P).
smokes(X) :- stress(X).
smokes(X) :- smokes(Y), influences(Y,X).
cancer(P) :- cancer_spont(P).
cancer(P) :- smokes(P), cancer_smoke(P).
```

Exact cases, checkable by enumeration over `2^|F|`:

- Graph, IJAR16 probabilities: `path(a,a)` 0.720, `(a,c)` 0.800, `(b,a)` 0.562, `(b,c)` 0.524,
  `(c,a)` 0.900, `(c,c)` 0.720.
- Graph, IJCAI15 probabilities (`e(b,c)` 0.5, `e(c,a)` 0.7): `(a,a)` 0.560, `(a,c)` 0.800,
  `(b,a)` 0.610, `(b,c)` 0.660, `(c,a)` 0.700, `(c,c)` 0.560.
- Alarm (TPLP15 Ex. 1–5): `0.1::burglary. 0.2::earthquake. 0.7::hears_alarm(X) :- person(X).`
  with evidence `calls(john)`: `Pr(calls(john)) = 0.196`, `Pr(burglary | calls(john)) = 0.357`.
  The relevant ground program drops `mary`.
- Probabilistic grid (TPLP15 §9.1): 16 × 16 nodes, edges right, down and diagonal, each 0.5,
  queries `path(n_i_i, n_16_16)`.

---

## 6. Gotchas

1. The fixpoint test is semantic, per atom, per iteration; canonicity makes it pointer equality.
2. `I⁰` maps everything to `⊥`, and probabilistic facts are pinned to their variable (never `⊤`);
   an empty body is `⊤`.
3. Never negate a formula that has not converged. In a program that is not stratified, carry the
   AFT pair and negate across it.
4. `(⊥, ⊤)` is a `Ψ_P` fixpoint on programs with positive loops: the unfoundedness refinement is
   mandatory.
5. Conditional probabilities exist at the fixpoint only.
6. No atom may be both probabilistic and derived.
7. Vtree policy is not in the papers; it is tddlog's own design.
