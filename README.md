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

tddlog is an exact inference engine for probabilistic Datalog:
probabilistic facts and annotated disjunctions, and recursive rules with
negation, comparisons and arithmetic, over atoms whose arguments are
constants. Front ends reduce richer inputs to this intermediate
language: ProbLog programs with function symbols and lists, Bayesian
and Markov networks, Dice programs and LPADs. The
engine grounds the program, simplifies it, compiles it bottom up into
tree decision diagrams (TDDs) with the
[tididi](https://github.com/Tractables/tididi) library, and reads every
query's probability off the diagrams. It is a Rust library and
command-line tool, with Python, C and WebAssembly bindings, and it
[runs in the browser](https://tractables.github.io/tddlog/).

- [Front ends](#front-ends): the languages and formats tddlog reads, and
  how each reduces to probabilistic Datalog.
- [Inside the engine](#inside-the-engine): the techniques the inference
  rests on.
- [Performance](#performance): tddlog against every exact inference
  system we could find, by type of program.

<p align="center">
  <img src="docs/images/overview.svg"
       alt="Answers and total time of tddlog and of the virtual best of each family of exact inference systems, by type of program">
</p>

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
examples that open in the [playground]: the [reachability example]
queries paths in a graph with cycles, the [negation example] the winning
positions of a game defined by negation through recursion, the
[Bayesian network example] a network's marginals, and the
[binary integers example] the sum of two random integers.
[examples/](examples/) lists every example program by what it shows.

## Front ends

Every input is reduced to a probabilistic Datalog program before
inference. `tddlog --ground` prints the ground program that results,
which ProbLog also reads, so each reduction can be checked on its own.

| Input | Read by | Reduced to probabilistic Datalog as |
|---|---|---|
| ProbLog programs | tddlog | function symbols and lists ground to constants, with ProbLog's list predicates and `findall/3` over deterministic goals |
| Bayesian networks in BIF, Hugin, GeNIe or UAI, with evidence in Ace's `.inst` or UAI's `.evid` | tddlog, by the file's extension | one annotated disjunction per row of each table |
| Markov networks in UAI | tddlog | one observed atom per factor; `--evidence-probability` prints the partition function |
| Dice programs | [`scripts/dice2problog.py`](scripts/dice2problog.py) | each integer as its bits, each operation as rules over them |
| LPADs written for cplint and PITA | tddlog, where the file holds `:- begin_lpad.` | the same clauses in ProbLog's syntax, with the database clauses they call and the file's example queries |

```text
$ tddlog examples/asia.net --query leaves
$ python3 scripts/dice2problog.py tests/dice/arith.dice > arith.pl && tddlog arith.pl
```

`tddlog --program examples/asia.net` prints the program a network is
read as, and `tddlog --program tests/lpad/coin.cplint.pl` an LPAD's. As a library, [`tddlog::solve`] takes a program's text and the
command line's options and returns the answers, and the
[Python](bindings/python/README.md) (`tddlog.solve(program).answers`),
[C](bindings/c/README.md) and [WebAssembly](bindings/wasm/README.md)
bindings wrap the same call; a front end of one's own writes the text,
as [examples/sprinkler.rs](examples/sprinkler.rs) does from Rust data.

Not yet read: cut, if-then-else, `call/N`, `assert`, strings and
probabilities computed in a rule's body in ProbLog, and `findall/3` over
a probabilistic or recursive goal; lists, `/`, `%`, `nth_bit` and
`sample` in Dice; network files through the bindings, which take a
program's text (`tddlog --program` writes it).

## Inside the engine

The engine answers a ground program exactly, in four stages.
[docs/design.md](docs/design.md) describes each technique and the cases
measured behind it.

**Grounding and simplification**

- **Relevant grounding.** Magic sets and semi-naive bottom-up evaluation
  ground only what the queries and evidence depend on (Fierens et al.,
  TPLP 2015). Transitive closures ground from their bound end, and
  linear recursion is solved without iteration by Gaussian elimination
  (Tarjan, 1981).
- **And-inverter graph.** Acyclic rules are hashed into an and-inverter
  graph, so a gate the program writes many times is built once: of a
  12-bit circuit's 6 635 gates, 4 941 merge. Gates equal to another
  gate, to its negation or to a constant are then found by random
  simulation, proved by SAT and merged (fraiging, Mishchenko et al.,
  2005).
- **Conditioning on the evidence.** Unit propagation, SAT backbones and
  equivalences fix and merge facts, and tables are renormalized to the
  evidence, before anything is compiled.
- **Table structure.** The rows of one distribution share a class atom,
  rows that exclude each other share switches, and a choice read under
  exclusive contexts becomes one fact per context.

**Compilation**

- **Tp-compilation.** Each derived atom's TDD is built bottom up by the
  immediate-consequence operator on diagrams (Vlasselaer et al., IJCAI
  2015), one strongly connected component at a time and each shared atom
  once. Negation through recursion takes the well-founded fixpoint by
  approximation fixpoint theory (Bogaerts and Van den Broeck, ICLP 2015),
  which answers sound programs that ProbLog rejects.
- **Names where depth needs them.** An atom read across a narrow cut of
  the variable order is read through a named copy tied to its definition,
  as in a Tseitin encoding. After the compile, each name is kept or
  composed away, whichever gives the smaller diagram.
- **Projected scopes.** Each component compiles over the part of the
  vtree its facts lie in, since a TDD has a node on every level of its
  vtree.

**Vtrees**

- **Layouts from the program.** A vtree, the layout of the diagrams'
  variables, is read from tree decompositions computed with
  [goatd](https://github.com/Tractables/goatd), from the rules'
  dependency order, from the order the program states its facts in, or
  from the fan-in order of its rules.
- **Decision vtrees for circuits.** In a bit-blasted program, an atom
  read under a fact is read conditioned on it, a decision list becomes a
  balanced tree, and these guards give the vtree.
- **Layouts as trials.** By default, several layouts run in turns under
  doubling budgets of engine work, one of them a vtree searched by
  rotations during the compile, and the first to finish answers. The
  budgets count pairs and units of engine work, not seconds, so the same
  layout wins on every machine.

**Queries and evidence**

- **Every marginal in one pass.** The evidence is conjoined bottom up
  along the vtree, and every query is read off one backward pass.
  Probabilities below a double's range, as a thousand observations give,
  keep an exponent of their own instead of underflowing.
- **Summing out.** Where only the probability of the evidence or a
  partition function is asked, each join sums out what no later join
  reads.

## Performance

tddlog was run against every exact inference system we could find. The
results are partitioned by type of program, in six categories, and each
instance counts in one category, whatever language its source was
written in. Every system given some instance of a category is measured
on all of the category's instances: it is given each one it can take,
through a translation where it cannot read the original, and an
instance it does not answer, cannot express or was not given counts as
unanswered. Each run is single-threaded, each suite with its own time
limit (300 to 1200 seconds), and every answer is checked against an
exact reference.

Total time is the measure: a system's total over all of a category's
instances, a run that does not answer counting as its time limit and a
time under 0.2 s as 0.2 s. A system run in several forms (ProbLog's
three compilers, Ace's two modes) counts as its fastest form on each
instance, and the virtual best of several systems is the fastest of
them on each instance. The figure above compares tddlog with the
virtual best of each family of systems. The cactus plots count the
instances each system answers within each time limit, for the systems
given at least 20 of the category's instances, and the scatter plots put
tddlog's time against the fastest other system's, instance by instance.
The [results page](https://tractables.github.io/tddlog/results/) has
every instance and time, each category with every system, a page for
each system (its version and settings, how it was given each program,
its plots against tddlog and its paper), and for each benchmark the
program as its authors wrote it, as each system was given it, and every
run;
[docs/baselines.md](https://github.com/Tractables/tddlog/blob/baselines/docs/baselines.md)
says how each system was given each suite and what it was not given.

### Probabilistic logic programs

<p align="center">
  <img src="docs/images/results-logic-programs.svg"
       alt="Probabilistic logic programs answered within each time limit, and tddlog's time against the fastest other system's">
</p>

ProbLog programs: the standard suite, the literature suite's families
and Biomine.

- tddlog answers **120 of 129** in 1.2 h. The best other system,
  Alea.jl, answers 65 in 5.9 h, and the virtual best of all other
  systems 92 in 3.9 h.
- On Biomine, ProbLog's own benchmark, tddlog answers all 10 queries and
  ProbLog 4.
- Where others win: pgmpy answers grid25, a path query across a 25 by 25
  grid of probabilistic edges, in 144 s on the program's ground network,
  and tddlog runs out of time.

### Probabilistic database queries

<p align="center">
  <img src="docs/images/results-database-queries.svg"
       alt="Probabilistic database queries answered within each time limit, and tddlog's time against the fastest other system's">
</p>

Boolean queries over tuple-independent databases: the TPC-H queries and
the literature suite's pdb family.

- tddlog answers **308 of 320** in 2.3 h. The best other system,
  ProvSQL, answers 284 in 5.2 h, and the virtual best of all other
  systems 298 in 3.7 h.
- Where others win: ProvSQL is fast on the easy TPC-H queries, its own
  benchmarks, and faster than tddlog on 107 of them.

### Bayesian networks

<p align="center">
  <img src="docs/images/results-bayesian-networks.svg"
       alt="Bayesian networks answered within each time limit, and tddlog's time against the fastest other system's">
</p>

bnlearn's networks, Ace's, the large networks, the noisy-or networks,
the networks of the Dice, Roulette and noDice papers, and the UAI 2022
networks whose factors are conditional probability tables, the Promedus
diagnosis and or_chain networks among them.

- tddlog answers **291 of 292** in 7 min. The best other system,
  Ace, answers 272 in 2.6 h, and the virtual best of all other
  systems 286 in 1.1 h.

### Markov networks

<p align="center">
  <img src="docs/images/results-markov-networks.svg"
       alt="Markov networks answered within each time limit, and tddlog's time against the fastest other system's">
</p>

The other UAI 2022 networks, whose factors are not conditional
probability tables over an acyclic graph: object detection,
segmentation and grid networks, protein networks and random networks.

- tddlog answers **215 of 263** in 4.6 h. The best other system,
  TensorInference.jl, answers 187 in 9.1 h, and the virtual best of all other
  systems 198 in 6.6 h.
- Where others win: 15 partition functions of the UAI 2022 final set, 14
  of them protein networks, run out of time or memory.
  TensorInference.jl answers each in 45 to 85 s.

### Probabilistic functional programs

<p align="center">
  <img src="docs/images/results-functional-programs.svg"
       alt="Probabilistic functional programs answered within each time limit, and tddlog's time against the fastest other system's">
</p>

Programs over discrete values with conditionals and loops: the Dice
paper's programs and loops, Roulette's, Geni's, noDice's and Pluck's
programs, and Cao et al.'s integer programs.

- tddlog answers **222 of 258** in 7.1 h. The best other system,
  Storm, answers 174 in 16 h, and the virtual best of all other
  systems 216 in 8.8 h.
- Where others win: PRISM and Storm answer Geni's three largest router
  networks, which tddlog does not, by value iteration over decision
  diagrams in double precision; their exact engines time out from 18 to
  20 routers.

### Bit-blasted programs

<p align="center">
  <img src="docs/images/results-bit-blasted.svg"
       alt="Bit-blasted programs answered within each time limit, and tddlog's time against the fastest other system's">
</p>

Programs over integers written in Alea.jl, which compiles each to a
Boolean circuit. Alea.jl runs its own program; every other system is
given the circuit.

- tddlog answers **218 of 250** in 8.0 h. The best other system,
  Alea.jl, answers 137 in 21 h, and the virtual best of all other
  systems 179 in 14 h.
- Where others win: Alea.jl on 65 circuits, six of which tddlog does not
  answer, and Roulette on 34; Ace, pyAgrum and pgmpy on two clinical
  trial circuits that tddlog does not answer, and Merlin on one of them;
  Genfer on the population models, which it answers in milliseconds; Psi
  on two linear regressions it solves symbolically.

### Each system on its own benchmarks

| System | Its benchmarks | Instances | tddlog | The system |
|---|---|--:|--:|--:|
| Ace | Ace's own benchmarks (Chavira and Darwiche) | 46 | 46 in 18 s | 46 in 8 min |
| ProbLog | The Biomine network of ProbLog's first paper (De Raedt et al., IJCAI 2007) | 10 | 10 in 3 min | 4 in 1.0 h |
| PITA | cplint's blood type example, which PITA ships | 1 | 1 in 0.2 s | 1 in 0.5 s |
| ProvSQL | The TPC-H queries, ProvSQL's own evaluation | 295 | 289 in 1.6 h | 275 in 3.9 h |
| Dice | The Dice paper's programs and networks (OOPSLA 2020) and its repository's loops | 69 | 67 in 41 min | 39 in 5.3 h |
| Alea.jl | The bit-blasting paper's programs (PLDI 2024), Alea.jl's own | 250 | 218 in 8.0 h | 137 in 21 h |
| Alea.jl | Cao et al.'s integer programs (UAI 2023), on Alea.jl's arithmetic branch | 28 | 25 in 35 min | 25 in 33 min |
| Roulette | The Roulette paper's programs and networks (PLDI 2025), from its artifact | 114 | 95 in 3.5 h | 44 in 12 h |
| Geni | The programs of Geni's evaluation (ICFP 2025) | 60 | 49 in 2.1 h | 39 in 4.2 h |
| noDice | noDice's programs (OOPSLA 2026) | 14 | 13 in 11 min | 10 in 55 min |
| Pluck | Pluck's four sequence models (PLDI 2025) | 4 | 4 in 2 min | 1 in 30 min |
| Genfer | Genfer's population model (NeurIPS 2023) | 12 | 12 in 8 min | 12 in 4.3 s |
| Genfer | The Dice programs Genfer's repository ships | 7 | 7 in 1.4 s | 7 in 1.4 s |
| Psi | Psi's R2 linear regression, as Psi's own program | 8 | 6 in 27 min | 2 in 1.0 h |
| SPPL | SPPL's hierarchical Markov switching model (PLDI 2021) | 8 | 8 in 7 min | 0 in 1.3 h |
| PSouffle | The Android taint analyses of PSouffle's evaluation (FMCAD 2026) | 15 | 12 in 16 min | 10 in 35 min |

### Setup

The release run compared tddlog with ProbLog 2.3.0, PITA, aspmc, Ace
3.0, Merlin 1.7.0, pyAgrum 3.2.1, pgmpy 1.1.2, libDAI 0.3.2,
TensorInference.jl 0.6.3, ProvSQL 1.13.0, Dice, Alea.jl, Roulette and
the further systems, on lab and cloud machines. A suite's systems ran in
the same job, side by side in parallel lanes, so times compare within a
suite, and each comparison pairs a baseline with tddlog run on the same
machine in the same lanes. tddlog's rows come from a rerun of its current version on the
same machines, with the same memory per run; the suites that had run on
cloud machines were rerun in full on lab machines, every system side by
side. Dice's loops are given to ProbLog, PITA and aspmc both unrolled and
as recursion, and each is scored on the form it does better on. The
references are ProbLog, Ace, Merlin and Alea.jl where they
answer, Ganak's exact model count, and exact solvers written for this
repository. [docs/benchmarks.md](https://github.com/Tractables/tddlog/blob/baselines/docs/benchmarks.md) on the `baselines` branch has the machines,
versions and every table; the
[`baselines` branch](https://github.com/Tractables/tddlog/tree/baselines)
has the code that runs each system and the copies of their programs.

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
[reachability example]: https://tractables.github.io/tddlog/guide/#reachability
[negation example]: https://tractables.github.io/tddlog/guide/#negation
[Bayesian network example]: https://tractables.github.io/tddlog/guide/#bayesian-network
[binary integers example]: https://tractables.github.io/tddlog/guide/#binary-integers
[`tddlog::solve`]: https://tractables.github.io/tddlog/api/tddlog/fn.solve.html
