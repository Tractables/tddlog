# tddlog

<p align="center">
  <img src="https://raw.githubusercontent.com/Tractables/tddlog/main/docs/logo.svg"
       alt="tddlog" width="340">
  <br>
  <a href="https://github.com/Tractables/tddlog/actions/workflows/ci.yml"><img
     src="https://github.com/Tractables/tddlog/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://tractables.github.io/tddlog/"><img
     src="https://img.shields.io/badge/run-in%20the%20browser-blue" alt="Run in the browser"></a>
  <a href="https://tractables.github.io/tddlog/guide/"><img
     src="https://img.shields.io/badge/docs-guide-blue" alt="Guide"></a>
  <a href="https://tractables.github.io/tddlog/api/tddlog/"><img
     src="https://img.shields.io/badge/docs-Rust-blue" alt="Rust API"></a>
  <a href="https://tractables.github.io/tddlog/python/"><img
     src="https://img.shields.io/badge/docs-Python-blue" alt="Python API"></a>
  <a href="https://tractables.github.io/tddlog/c/"><img
     src="https://img.shields.io/badge/docs-C-blue" alt="C API"></a>
  <a href="https://tractables.github.io/tddlog/results/"><img
     src="https://img.shields.io/badge/results-benchmarks-blue" alt="Benchmark results"></a>
</p>

**Exact probabilistic inference by compilation into tree decision diagrams.**

tddlog answers probabilistic Datalog programs exactly: probabilistic
facts, annotated disjunctions, and recursive rules with negation,
comparisons and arithmetic. [Front ends](#front-ends) reduce ProbLog
programs with function symbols and lists, Bayesian and Markov networks,
Dice programs and LPADs to it. tddlog grounds the program, simplifies
it, compiles it bottom up into tree decision diagrams (TDDs) with the
[tididi](https://github.com/Tractables/tididi) library, and reads every
query's probability off the diagrams. It is a Rust library and
command-line tool with Python, C and WebAssembly bindings, and it
[runs in the browser](https://tractables.github.io/tddlog/).

<p align="center">
  <img src="docs/images/overview.svg"
       alt="Each type of program against the exact inference systems native to it: their answers, their total times and tddlog's, and tddlog's speed-up over each">
</p>

On every type of program, tddlog answers more instances than each
exact inference system native to it, in less total time
([Performance](#performance)).

## Install

- **Command line:** `cargo install --locked --git https://github.com/Tractables/tddlog tddlog`.
- **Rust:** `tddlog = { git = "https://github.com/Tractables/tddlog" }` under
  `[dependencies]`. See the [Rust API].
- **Python:** build the package from a checkout. See the
  [Python README](bindings/python/README.md).
- **C and C++:** build the CMake package from a checkout. See the
  [C README](bindings/c/README.md).
- **Browser:** the [playground] runs tddlog in the tab, with nothing to install.

A build from source needs Rust 1.88 or newer and a C++ compiler (gcc 12
or newer, or a recent clang) for the tree-decomposition solver that goatd
vendors.

## Getting started

A program states probabilistic facts, rules, evidence and queries.
This is `examples/alarm.pl`:

```prolog
0.1::burglary.
0.2::earthquake.
0.7::hears_alarm(john).
0.4::hears_alarm(mary).
alarm :- burglary.
alarm :- earthquake.
calls(X) :- alarm, hears_alarm(X).
evidence(calls(john), true).
query(burglary).
query(earthquake).
query(calls(mary)).
```

tddlog prints the probability of each query given the evidence:

```text
$ tddlog examples/alarm.pl
burglary:       0.357143
earthquake:     0.714286
calls(mary):    0.400000
```

Each probabilistic fact `p::f` holds with probability p, independently
of the others, and the rules derive the other atoms. The alarm goes off
with probability 1 − 0.9 × 0.8 = 0.28, so John calls with probability
0.28 × 0.7 = 0.196, and P(burglary | John calls) = 0.1 × 0.7 / 0.196 =
0.357143.

The [guide] describes the language and the command line, with worked
examples that open in the [playground].
[examples/](examples/) lists every example program by what it shows.

## Front ends

Every input is reduced to a probabilistic Datalog program before
inference:

| Input | Read by | Reduced to probabilistic Datalog as |
|---|---|---|
| ProbLog programs | tddlog | function symbols and lists ground to constants, with ProbLog's list predicates, `call/N`, `findall/3`, `all/3` and `forall/2` over probabilistic goals, and `library(aggregate)`'s aggregates |
| Bayesian networks in BIF, Hugin, GeNIe or UAI, with evidence in Ace's `.inst` or UAI's `.evid` | tddlog, by the file's extension | one annotated disjunction per row of each table |
| Markov networks in UAI | tddlog | one observed atom per factor; `--evidence-probability` prints the partition function |
| Dice programs | tddlog, by the extension `.dice` | each integer as its bits, each operation as rules over them, a loop as recursion over a step index |
| Alea.jl programs (`model()` and `QUERY`) | [`scripts/alea2problog.jl`](scripts/alea2problog.jl), in Julia with Alea.jl | the Boolean circuit Alea.jl builds: each flip a probabilistic fact, each gate a rule, each observation evidence |
| Roulette programs (`#lang roulette/example/disrupt`) | [`scripts/roulette2problog.rkt`](scripts/roulette2problog.rkt), in Racket with Roulette; with `--tddlog`, the program runs with tddlog answering its queries | the Boolean circuit Roulette builds: each flip a probabilistic fact, each gate a rule, each observation evidence |
| LPADs written for cplint and PITA | tddlog, where the file holds `:- begin_lpad.` | the same clauses in ProbLog's syntax, with the database clauses they call and the file's example queries; under PITA's depth bound, each predicate with the depth as an argument |

```text
$ tddlog examples/asia.net --query leaves
$ tddlog tests/dice/arith.dice
$ tddlog --program tests/lpad/coin.cplint.pl
```

`--program` prints the program an input is read as, and `--ground` the
ground program after it, which ProbLog also reads, so each reduction can
be checked on its own. As a library, [`tddlog::read`] reads an input in
any of these formats as a program and [`tddlog::solve`] answers it; the
[Python](bindings/python/README.md) (`tddlog.solve(text, format="dice")`),
[C](bindings/c/README.md) and [WebAssembly](bindings/wasm/README.md)
bindings take the text with its format's name. A front end of one's own
writes the program's text, as [examples/sprinkler.rs](examples/sprinkler.rs)
does from Rust data.

Not yet read: cut, if-then-else, `assert`, strings, `library(collect)`
and a findall in a cycle of calls in ProbLog; `/`, `%` and `sample` in
Dice.

## Inside the engine

[docs/design.md](docs/design.md) describes each stage and the cases
measured behind it.

**Grounding and simplification**

- **Relevant grounding.** Magic sets and semi-naive evaluation ground
  only what the queries and the evidence depend on, ProbLog's relevant
  ground program (Fierens et al., TPLP 2015). Transitive closures ground
  from their bound end, and linear recursion is solved without iteration
  by Gaussian elimination (Tarjan, 1981).
- **And-inverter graph.** Acyclic rules are hashed into an and-inverter
  graph, so a gate the program writes many times is built once (of a
  12-bit circuit's 6 635 gates, 4 941 merge). Gates equal to another
  gate, to its negation or to a constant are found by random simulation,
  proved by SAT and merged (fraiging, Mishchenko et al., 2005).
- **Conditioning.** Unit propagation, SAT backbones and equivalences fix
  and merge facts under the evidence, and tables are renormalized to it.
- **Table structure.** The rows of one distribution share a class atom,
  rows that exclude each other share switches, and a choice read under
  exclusive contexts becomes one fact per context.

**Compilation**

- **Tp-compilation.** Each derived atom's TDD is built bottom up by the
  immediate-consequence operator on diagrams (Vlasselaer et al., IJCAI
  2015), one strongly connected component at a time, each shared atom
  once, and each component over the part of the vtree its facts lie in.
  Negation through recursion takes the well-founded fixpoint by
  approximation fixpoint theory (Bogaerts and Van den Broeck, ICLP 2015),
  which answers sound programs that ProbLog rejects.
- **Names where depth needs them.** An atom read across a narrow cut of
  the variable order is read through a named copy tied to its
  definition, as in a Tseitin encoding. After the compile, each name is
  kept or composed away, whichever gives the smaller diagram.

**Vtrees**

- **Layouts from the program.** A vtree, the layout of the diagrams'
  variables, is read from tree decompositions computed with
  [goatd](https://github.com/Tractables/goatd), from the rules'
  dependency order, from the order the program states its facts in, or
  from its rules' fan-in. In a bit-blasted program, an atom read under a
  fact is read conditioned on it, a decision list becomes a balanced
  tree, and these guards give the vtree.
- **Layouts as trials.** By default, several layouts run in turns under
  doubling budgets of engine work, one of them a vtree searched by
  rotations during the compile, and the first to finish answers. The
  budgets count pairs and units of work, not seconds, so the same layout
  wins on every machine.

**Queries and evidence**

- **Every marginal in one pass.** The evidence is conjoined bottom up
  along the vtree, and every query is read off one backward pass.
  Probabilities below a double's range keep an exponent of their own
  instead of underflowing.
- **Summing out.** Where only the probability of the evidence or a
  partition function is asked, each join sums out what no later join
  reads.

## Performance

tddlog was compared with every exact inference system we could find, on
six types of program, each against the systems that natively handle it,
and on [all programs](#all-programs) together, against every system.
Every answer is checked against an exact reference. Each table gives a
system's answers, its total time and tddlog's over the instances it is
given and either answers, and how many times faster tddlog is. The
[results page](https://tractables.github.io/tddlog/results/) has every
instance, time and program, and a page for each system.

<details>
<summary>How the systems were run and compared</summary>

- Each instance counts in one type, whatever language its source was
  written in. A system is given every instance it can take, through a
  translation where it cannot read the original (an LPAD for ProbLog, a
  Dice program for Roulette).
- Runs are single-threaded, under each suite's time limit (300 to 1200
  seconds). A suite's systems ran side by side on the same machines, and
  tddlog's current version in the same lanes, with the same memory per
  run.
- **Answers** count over all of a type's instances. **Total times** count
  a run that does not answer as twice its time limit (PAR-2, as in the
  SAT and model counting competitions) and a time under 0.2 s as 0.2 s.
- **tddlog faster** is the ratio of the total times on the instances both
  answer, leaving out those both answer in under 0.2 s. An instance only
  the other system answers is added with tddlog at its time limit, and
  one only tddlog answers with the other system at its limit, where that
  limit over tddlog's time is at least the ratio: the other system's
  timeouts can only raise the speed-up. With fewer than 20 instances both
  answer there is no speed-up.
- A system run in several forms (ProbLog's three compilers, Ace's two
  modes) counts as its fastest form on each instance.
- The plots count the instances each system answers within each time
  limit, and put tddlog's time against the fastest other system's,
  instance by instance.
- The [`baselines` branch](https://github.com/Tractables/tddlog/tree/baselines)
  has the code that runs each system,
  [how each was given each suite](https://github.com/Tractables/tddlog/blob/baselines/docs/baselines.md),
  and the [machines, versions and tables](https://github.com/Tractables/tddlog/blob/baselines/docs/benchmarks.md).

</details>

### Probabilistic logic programs

<p align="center">
  <img src="docs/images/results-logic-programs.svg"
       alt="Probabilistic logic programs answered within each time limit, and tddlog's time against the fastest native system's">
</p>

tddlog answers **331 of 356** ProbLog programs, of 25 families: paths
in random graphs and grids, Biomine's paths, smokers, UW-CSE, Cora's
citation matching, blood types and pedigrees, sickness, Mastermind,
MNIST addition and handwritten formulas, Prob-EC's event calculus, taint
analyses of Android apps, probabilistic grammars, hidden Markov models,
and the programs of the ProbLog tutorial and of ProbLog's and cplint's
examples.

| System | Answers | Its total time | tddlog's total time | tddlog faster |
|---|--:|--:|--:|--:|
| ProbLog | 251 | 15 h | 47 min | 29× |
| PITA | 212 | 21 h | 27 min | 169× |
| aspmc | 209 | 22 h | 27 min | 93× |
| Scallop | 148 | 31 h | 26 min | too few in common |
| PSouffle | 105 | 41 h | 27 min | 665× |

- On Biomine, ProbLog's own benchmark, tddlog answers all 10 queries and
  ProbLog 4.
- Where others win: ProbLog answers 22 of its tutorial's 115 programs
  that tddlog does not, each in under 2 s. Nineteen of them used what
  tddlog did not read yet when they ran, and on three it ran out of time
  or memory.

### Probabilistic database queries

<p align="center">
  <img src="docs/images/results-database-queries.svg"
       alt="Probabilistic database queries answered within each time limit, and tddlog's time against the fastest native system's">
</p>

tddlog answers **308 of 320** Boolean queries over tuple-independent
databases: 16 TPC-H queries, the queries h0 and h1, the safe queries QW
and Q9, and paths of two edges in random graphs.

| System | Answers | Its total time | tddlog's total time | tddlog faster |
|---|--:|--:|--:|--:|
| ProvSQL | 268 | 16 h | 45 min | 20× |
| ProbLog | 204 | 39 h | 45 min | 72× |
| PSouffle | 189 | 41 h | 45 min | 94× |
| PITA | 123 | 62 h | 45 min | 223× |
| Scallop | 30 | 90 h | 45 min | too few in common |
| aspmc | 16 | 97 h | 45 min | too few in common |

- Where others win: ProbLog on the paths of two edges in random graphs
  of 14 and 17 nodes, in 4.8 s and 39 s against tddlog's 18 s and 182 s.

### Bayesian networks

<p align="center">
  <img src="docs/images/results-bayesian-networks.svg"
       alt="Bayesian networks answered within each time limit, and tddlog's time against the fastest native system's">
</p>

tddlog answers **609 of 636** networks: bnlearn's, the large ones and
the Roulette and noDice papers' versions among them, Ace's, the noisy-or
networks, and the networks of the UAI 2022 competition and of Ihler's
collection whose factors are conditional probability tables
(diagnosis, pedigree, BN, blockmap, Mastermind and fs networks, and the
classic networks).

| System | Answers | Its total time | tddlog's total time | tddlog faster |
|---|--:|--:|--:|--:|
| Ace | 416 | 34 h | 1.3 h | 23× |
| TensorInference.jl | 323 | 56 h | 1.5 h | 34× |
| Merlin | 293 | 57 h | 1.5 h | 34× |
| libDAI | 291 | 58 h | 1.5 h | 33× |
| pyAgrum | 263 | 63 h | 1.5 h | 37× |
| 14 others: PITA, pgmpy, aspmc, Alea.jl, Roulette, Pluck, ProbLog, Dice, Genfer, SPPL, noDice, Scallop, Psi, Geni | 43 to 238 | over 19 h each | 2 min to 1.5 h | at least 42× |

- Where others win: the networks named BN of Ihler's collection. tddlog
  runs out of time on seven that pyAgrum or Merlin answers in 0.3 to 2.1 s,
  and on 20 more Merlin, pyAgrum or TensorInference.jl is at least twice
  as fast, up to 0.3 s against tddlog's 204 s.

PSouffle was given 11 of the networks and answers 6.

### Markov networks

<p align="center">
  <img src="docs/images/results-markov-networks.svg"
       alt="Markov networks answered within each time limit, and tddlog's time against the fastest native system's">
</p>

tddlog answers **243 of 290** networks, those of the UAI 2022
competition and of Ihler's collection whose factors are not conditional
probability tables over an acyclic graph: object detection,
segmentation, grid, DBN, CSP and Alchemy networks, protein side chains,
the rus networks, weighted CSPs, and pedigree and diagnosis networks.

| System | Answers | Its total time | tddlog's total time | tddlog faster |
|---|--:|--:|--:|--:|
| TensorInference.jl | 198 | 9.3 h | 2.6 h | 2.5× |
| libDAI | 180 | 13 h | 1.4 h | 16× |
| Merlin | 171 | 13 h | 1.6 h | 11× |
| pyAgrum | 170 | 13 h | 1.4 h | 13× |
| pgmpy | 117 | 22 h | 1.4 h | 28× |

- Where others win: six partition functions of the UAI 2022 final set,
  five of them protein networks, and Ihler's pedigree50 run out of time
  or memory. TensorInference.jl answers the six in 12 to 45 s and
  pedigree50 in 133 s.

### Probabilistic functional programs

<p align="center">
  <img src="docs/images/results-functional-programs.svg"
       alt="Probabilistic functional programs answered within each time limit, and tddlog's time against the fastest native system's">
</p>

tddlog answers **279 of 315** programs over discrete values with
conditionals and loops, of 17 families: the Caesar cipher, diamond and
ladder networks, Geni's router grids with failing links or routers and
its weather factory, hidden Markov models and Pluck's sequence models, a
Markov chain, a lossy network, contract signing, a perturbed-memory
interpreter, Cao et al.'s integer programs, the coupon collector, 3-SAT,
the bounded retransmission protocol and the Dice paper's small
programs.

| System | Answers | Its total time | tddlog's total time | tddlog faster |
|---|--:|--:|--:|--:|
| Alea.jl | 209 | 25 h | 1.2 h | 12× |
| Roulette | 171 | 37 h | 1.2 h | 18× |
| Dice | 171 | 38 h | 1.2 h | 36× |
| Pluck | 123 | 53 h | 1.2 h | 39× |
| Geni | 91 | 63 h | 1.2 h | 70× |
| 4 others: Genfer, noDice, SPPL, Psi | 8 to 83 | over 65 h each | 1.2 h | at least 408× |

### Bit-blasted programs

<p align="center">
  <img src="docs/images/results-bit-blasted.svg"
       alt="Bit-blasted programs answered within each time limit, and tddlog's time against the fastest native system's">
</p>

tddlog answers **224 of 257** programs over integers written in
Alea.jl, which compiles each to a Boolean circuit. Alea.jl runs its own
program; every other system is given the circuit.

| System | Answers | Its total time | tddlog's total time | tddlog faster |
|---|--:|--:|--:|--:|
| Alea.jl | 139 | 32 h | 3.7 h | 6.6× |
| Roulette | 98 | 43 h | 2.1 h | 20× |
| Pluck | 54 | 58 h | 2.1 h | 31× |
| Dice | 19 | 69 h | 2.1 h | too few in common |
| Genfer | 12 | 71 h | 2.1 h | too few in common |
| 4 others: noDice, Psi, SPPL, Geni | 0 to 7 | over 72 h each | 2.1 h to 2.7 h | too few in common |

- Where others win: Alea.jl on 59 circuits, five of which tddlog does not
  answer, and Roulette on 31; Genfer on the population models, which it
  answers in milliseconds; Psi on two linear regressions it solves
  symbolically.

### All programs

<p align="center">
  <img src="docs/images/results-all-programs.svg"
       alt="All programs answered within each time limit, and tddlog's time against the fastest other system's">
</p>

tddlog answers **1994 of 2174**: the six types together, every system
on every program it is given, natively or through a translation.

| System | Answers | Its total time | tddlog's total time | tddlog faster |
|---|--:|--:|--:|--:|
| Ace | 1070 | 179 h | 7.1 h | 28× |
| libDAI | 913 | 227 h | 7.4 h | 36× |
| TensorInference.jl | 890 | 243 h | 8.4 h | 30× |
| pyAgrum | 866 | 233 h | 7.4 h | 35× |
| Merlin | 862 | 238 h | 7.6 h | 30× |
| 18 others: ProbLog, PITA, pgmpy, Alea.jl, aspmc, Roulette, Storm, Pluck, Dice, PRISM, PSouffle, ProvSQL, Scallop, noDice, Genfer, Geni, SPPL, Psi | 88 to 752 | over 16 h each | 45 min to 7.6 h | at least 20× |

- Where others win: pgmpy answers grid25, a logic program's path query
  across a 25 by 25 grid of probabilistic edges, in 144 s on the
  program's ground network, and tddlog runs out of time. PRISM and Storm
  answer Geni's four largest router networks (PRISM alone the largest,
  48 routers), which tddlog does not, by
  value iteration over decision diagrams in double precision; their
  exact engines time out from 18 to 20 routers. Ace, pyAgrum and pgmpy
  answer two clinical trial circuits that tddlog does not, and Merlin
  one of them.

## Building and testing

From a checkout, `cargo build --release` builds `target/release/tddlog`,
and `cargo test` checks it against ProbLog's answers on the examples and
on ProbLog's own system tests. [CONTRIBUTING.md](.github/CONTRIBUTING.md) lists
the checks a change runs.

## Citing

tddlog has no paper yet: cite the software, as [CITATION.cff](CITATION.cff)
describes it (GitHub's "Cite this repository"). TDDs were introduced in
[*A Canonical Generalization of OBDD*](https://arxiv.org/abs/2604.05537).

## Acknowledgements

This material is based upon work supported by the Defense Advanced Research
Projects Agency (DARPA) under Agreement No. HR0011269E224.

## License

Apache-2.0 ([LICENSE](LICENSE)). The copies of third-party programs in
`tests/problog` and `tests/dice` keep their own licenses
([NOTICE](NOTICE)); [THIRD-PARTY.md](THIRD-PARTY.md) lists the components
compiled into the binaries.

[guide]: https://tractables.github.io/tddlog/guide/
[playground]: https://tractables.github.io/tddlog/
[Rust API]: https://tractables.github.io/tddlog/api/tddlog/
[`tddlog::solve`]: https://tractables.github.io/tddlog/api/tddlog/fn.solve.html
[`tddlog::read`]: https://tractables.github.io/tddlog/api/tddlog/fn.read.html
