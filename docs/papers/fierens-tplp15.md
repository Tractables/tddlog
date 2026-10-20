# Fierens et al., "Inference and Learning in Probabilistic Logic Programs using Weighted Boolean Formulas" (TPLP 2015)

Implementer's note. Source: D. Fierens, G. Van den Broeck, J. Renkens, D. Shterionov, B. Gutmann,
I. Thon, G. Janssens, L. De Raedt, TPLP 15(3):358-401, 2015. The ProbLog2 paper: program ->
relevant ground program -> weighted CNF -> WMC by knowledge compilation (d-DNNF or BDD ->
arithmetic circuit), plus EM parameter learning. tddlog takes its semantics, the relevant ground
program and the weights from here, and replaces the CNF and loop breaking by Tp-compilation
([tp-compilation.md](tp-compilation.md)).

Notation: `L` program, `Lg` relevant ground program, `C` total choice, `Q` queries, `E = e`
evidence. Claims marked **[NOT IN PAPER]** are additions an implementer needs.

---

## 1. Syntax and semantics (Sec. 3)

- A program is a set of ground probabilistic facts `p::f` and a logic program of normal rules
  (negation allowed in bodies) and plain facts.
- **Intensional probabilistic facts** `p::f(X1..Xn) :- body.`, whose body calls
  non-probabilistic facts only, are expanded to ground facts before anything else:
  `0.7::hears_alarm(X) :- person(X).` becomes one fact per person.
- **Probabilistic atoms and derived atoms are disjoint**: no atom is both a probabilistic fact
  and a rule head. In particular evidence cannot be added as a rule for a probabilistic atom.
- **Rules are range-restricted** (every head variable in a positive body literal), so SLD
  grounding yields ground rules. The Herbrand base is assumed finite.
- **Annotated disjunctions** are not in the paper. **[NOT IN PAPER]** The standard reduction:
  `p1::h1 ; ... ; pk::hk :- body.` becomes facts `q_i::c_i(Vars)` with the stick-breaking
  probabilities `q1 = p1`, `q2 = p2/(1-p1)`, `q3 = p3/(1-p1-p2)`, ..., and rules
  `hi :- body, c_i, \+c_1, ..., \+c_{i-1}`. The outcomes are mutually exclusive; never encode
  them as independent facts.

Distribution semantics: each probabilistic fact is an independent atomic choice; a total choice
`C` has `P(C) = Π_{f∈C} p_f Π_{f∉C} (1-p_f)`, and its world is the **well-founded model** of
`C ∪ R` (the least Herbrand model without negation). Total choices and models are in bijection.

**Soundness requirement** (Sec. 3.2): every total choice must give a two-valued well-founded
model; otherwise the program is not a valid ProbLog program. Locally stratified rules suffice
(footnote 4). Positive recursion (`smokes(X) :- smokes(Y), influences(Y,X)`) is fine; recursion
through negation (`a :- \+a.`) is not.

Negation is negation as failure under the closed-world assumption: the rules of `a` are its
definition, `a` holds iff some body holds. FOL implication alone is wrong (`{a <- b}` has three
models, the program `{a :- b}` one).

---

## 2. Inference tasks (Sec. 4)

- **EVID** `P(E = e)`; **MARG** `P(Q | E = e)` for each query; **MPE** the most probable
  assignment of the unobserved atoms.
- Evidence may be on any atom, probabilistic or derived (the feature ProbLog2 claims over
  PRISM, ProbLog1 and PITA).
- `P(Q | E=e) = P(Q ∧ E=e) / P(E=e)`; all of them come from one compiled circuit. With evidence
  on derived atoms `P(E=e)` can be 0; handle that before dividing **[NOT IN PAPER]**.

---

## 3. The pipeline (Sec. 5 and 6)

### 3.1 Relevant ground program (Sec. 5.1)

- The dependency set of a ground atom is the set of atoms in some proof of it. An atom is
  relevant if it is in the dependency set of `Q ∪ E`; a rule is relevant if all its atoms are.
- Ground by tabled SLD resolution from `Q ∪ E` (evidence seeds the grounding as queries do;
  tabling stops loops on cyclic rules). The relevant rules are the ground rules met.
- A rule is **inactive** if its body has a literal false in the evidence; inactive rules are
  dropped, which can make probabilistic facts irrelevant.
- A body literal **true** in the evidence must be kept (footnote 9): it may be part of a
  positive loop.
- The definition of a **false** evidence atom must be kept (Example 6): its head's value
  constrains its bodies.
- Theorem 1: `L` and `Lg` give the same `P(Q | E=e)`. Pruning removed only about 17% of the
  rules but up to 90% of the CNF clauses in the experiments, by breaking loops.

### 3.2 Rules to a Boolean formula (Sec. 5.2)

- Acyclic rules: Clark's completion, `a <-> B_1 ∨ ... ∨ B_k` per derived atom. An atom with no
  rule is `a <-> false` and must be forced false, or the count doubles.
- Rules of one head are disjoined in one definition; overlapping bodies are allowed (no PRISM
  exclusiveness), so `alarm` from both `burglary` and `earthquake` is not double counted.
- **Positive loops break Clark's completion** (Example 8): on two people with
  `influences(p1,p2)`, `influences(p2,p1)` and no stress, the completion admits both `smokes`
  true, a self-supporting loop the least model excludes. ProbLog2 removes loops by a rule-based
  rewrite (Janhunen 2004; fast, larger formulas) or a proof-based one (Mantadelis & Janssens
  2010; smaller formulas, intractable on large domains). Example 9's proof-based result:
  `smokes(p2)`'s support from `smokes(p1)` is replaced by `stress(p1)`, the derivation of
  `smokes(p1)` that does not go through `smokes(p2)`.
- **Faithfulness** (footnote 11): an auxiliary atom (loop breaking, Tseitin) must be determined
  by the original atoms and weighted 1 on both sides; two settings of an auxiliary for one
  setting of the originals multiply the count.

Tp-compilation needs no loop breaking: the least fixpoint excludes self-supporting loops by
construction. What stays relevant is "no rule means false" and faithfulness.

### 3.3 Evidence and weights (Sec. 5.3)

- `φ = φ_r ∧ φ_e`, `φ_e` one unit clause per evidence atom.
- `w(f) = p`, `w(¬f) = 1 - p` for a probabilistic fact; every other literal weighs 1 on both
  sides. Theorem 2: the models of `φ` are the models of `Lg` under the evidence, with their
  probabilities as weights.

### 3.4 Inference (Sec. 6)

`P(E=e)` is the weighted model count of `φ`. ProbLog2 compiles `φ` to a smooth d-DNNF (c2d or
DSHARP; BDDs lost in every experiment; SDDs are not in the paper), turns it into an arithmetic
circuit with an indicator per literal, and gets every `P(X ∧ E=e)` from one upward and one
downward pass (Darwiche 2009). **Smoothness is required** for that (Appendix C): on a non-smooth
circuit, Alarm with `earthquake=true` added evaluates to 0.196 instead of 0.14. Asserting the
evidence before compiling is optional given indicators, but cheaper.

The circuit structure is independent of the parameters, so EM (LFI-ProbLog, Sec. 7) compiles
once per training example and reuses it across iterations.

---

## 4. Test cases

### Alarm (Example 1)

```prolog
0.1::burglary.
0.2::earthquake.
0.7::hears_alarm(X) :- person(X).
person(mary).
person(john).
alarm :- burglary.
alarm :- earthquake.
calls(X) :- alarm, hears_alarm(X).
```

Four probabilistic atoms, 16 total choices. With evidence `calls(john)=true`:
`P(calls(john)) = 0.196`, `P(burglary ∧ calls(john)) = 0.07`, `P(burglary | calls(john)) =
0.357`, `P(earthquake ∧ calls(john)) = 0.14`; the MPE world is `{earthquake, hears_alarm(john),
hears_alarm(mary)}` (0.0882). The relevant ground program for `Q = {burglary}` drops `mary`
entirely, and `φ` has three models weighing 0.014 + 0.056 + 0.126 = 0.196.

### Smokers, three people (Example 6)

```prolog
0.2::stress(P) :- person(P).
0.3::influences(P1,P2) :- friend(P1,P2).
person(p1). person(p2). person(p3).
friend(p1,p2). friend(p1,p3). friend(p2,p1). friend(p3,p1).
smokes(X) :- stress(X).
smokes(X) :- smokes(Y), influences(Y,X).
```

With `E = {smokes(p2)=true, smokes(p3)=false}`, `Q = {smokes(p1)}`: the rule
`smokes(p1) :- smokes(p3), influences(p3,p1)` is inactive and `influences(p3,p1)` irrelevant,
but both rules of `smokes(p3)` stay.

### Simplified Smokers, two people (Examples 8, 9)

`0.2::stress(p1). 0.2::stress(p2). 0.3::influences(p2,p1). 0.3::influences(p1,p2).` with the
two `smokes` rules per person. The choice `{¬stress(p1), ¬stress(p2), influences(p1,p2),
influences(p2,p1)}` must give both `smokes` false. **[NOT IN PAPER]** `P(smokes(p1)) = 0.2 +
0.8·0.2·0.3 = 0.248`, and symmetrically for `p2`.

### Benchmarks (Sec. 9.1)

- Smokers with cancer (`cancer_spont`, `cancer_smoke`), the program in
  [tp-compilation.md](tp-compilation.md) §5, on power-law friendship graphs; half the `smokes`
  and `cancer` atoms as evidence.
- Probabilistic grid: 16 × 16, edges right, down and diagonal, each 0.5, queries
  `path(n_i_i, n_16_16)`. ProbLog2 reached distance 10, ProbLog1 distance 6.
- WebKB: `has_class(P,C)` from `word_class` and from linked pages' classes, one intensional
  probabilistic fact per (word, class) and (class, class) pair.

---

## 5. What the paper does not give

Annotated disjunctions, SDDs, the loop-breaking transformations themselves (delegated to
Janhunen and to Mantadelis & Janssens), infinite Herbrand bases, Tp-compilation, and numeric
answers for any program but Alarm.
