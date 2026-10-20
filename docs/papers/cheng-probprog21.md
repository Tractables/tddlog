# Cheng, Millstein, Van den Broeck, Holtzen, "flip-hoisting: Exploiting Repeated Parameters in Discrete Probabilistic Programs" (PROBPROG 2021)

Implementer's note. Source: https://starai.cs.ucla.edu/papers/ChengPROBPROG21.pdf.
The paper generalizes the parameter sharing of Bayesian network encodings
(Chavira and Darwiche 2008) to a program transformation on Dice programs.
tddlog keeps its local form as switch sharing (`src/ground/share.rs`,
narrowed to the tables where it pays), beside row classes and clause tables
([design.md](../design.md)).

## The transformation (Sec. 3)

`hoist(p, i, j)` replaces two `flip θ` with the same θ by references to one
new flip. It is unsound when some execution reaches both flips: hoisting
then couples two independent coins.

- **Theorem 1 (path redundancy).** Hoisting flips i and j is sound if
  their path conditions (the conditions on flips under which execution
  reaches each) are inconsistent. `share.rs` uses it with the rows'
  contexts as the path conditions.
- **Local hoisting (Prop. 2).** Two flips of one parameter in disjoint
  branches of the same `if` are always hoistable: parameter sharing within
  a table, checked syntactically.
- **Global hoisting.** A dataflow analysis of aliasing facts (variable =
  flip) and constraint facts (flip assignments implied by conjunctive
  guards), intersected at joins; two flips are hoistable if their
  constraint facts disagree on some flip. Polynomial, and finds sharing
  across tables, but adds little on the networks (emdec6g 712 to 678
  flips, tcc4e 545 to 541).

## Implementation in Dice (Sec. 4)

- **Sequential encoding** of `discrete(p1, ..., pk)`: flip `p1`, then
  `p2/(1−p1)`, and so on (Sang, Beame and Kautz 2005). Which parameters
  appear depends on the value order; ordering the values by how often
  each parameter occurs in the program keeps the common ones.
- **Order preservation (Theorem 3).** If hoisting keeps the BDD variable
  order, `|hoist(p,i,j)| ≤ |p|`: it only relabels two variables to one.

## Results (Sec. 5)

Flips with the default encoding, local hoisting, and the sequential
encoding with local hoisting: alarm 504, 201, 133; andes 1 084, 328, 328;
hailfinder 2 155, 1 149, 871; munin 25 625, 19 148, 16 487; pathfinder
28 703, 1 897, 1 652; pigs 2 066, 1 178, 586; water 3 113, 1 813, 1 783;
win95pts 350, 125, 125. Hoisting shrinks every BDD (pathfinder 61 157
nodes to 16 153), but the sequential encoding sometimes gives fewer flips
and larger BDDs (alarm, hailfinder, munin). Dice with hoisting beats
ProbLog on 21 networks and loses on 1; barley, mildew and diabetes did not
finish in 20 minutes in any setting.
