# tddlog

<p align="center">
  <a href="https://tractables.github.io/tddlog/"><img
     src="https://img.shields.io/badge/run-in%20the%20browser-blue" alt="Run in the browser"></a>
  <a href="https://tractables.github.io/tddlog/guide/"><img
     src="https://img.shields.io/badge/guide-worked%20examples-blue" alt="Guide"></a>
  <a href="https://pypi.org/project/tddlog/"><img
     src="https://img.shields.io/pypi/v/tddlog.svg" alt="PyPI"></a>
  <a href="https://github.com/Tractables/tddlog/releases/latest"><img
     src="https://img.shields.io/github/v/release/Tractables/tddlog" alt="GitHub release"></a>
  <a href="https://github.com/Tractables/tddlog/actions/workflows/ci.yml"><img
     src="https://github.com/Tractables/tddlog/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://tractables.github.io/tddlog/api/tddlog/"><img
     src="https://img.shields.io/badge/docs-API-blue" alt="API documentation"></a>
</p>

**Exact probabilistic inference by compilation into tree decision diagrams.**

tddlog computes exact probabilities for probabilistic logic programs
written in ProbLog, for Bayesian and Markov networks, and for the
discrete programs that probabilistic programming languages compile to.
It grounds a program, compiles it bottom up into tree decision diagrams
(TDDs) with the [tididi](https://github.com/Tractables/tididi) library,
and reads every query's probability off the diagrams. It is a Rust
library and command-line tool, with Python, C and WebAssembly bindings,
and it [runs in the browser](https://tractables.github.io/tddlog/).

<p align="center">
  <img src="docs/images/overview.svg"
       alt="Total time of tddlog and of each other exact inference system over the same instances, by kind of problem">
</p>

## Getting started

Install the command-line tool with Cargo, or the Python package with pip:

```sh
cargo install --git https://github.com/Tractables/tddlog tddlog
pip install tddlog
```

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

The [guide](https://tractables.github.io/tddlog/guide/) walks through the
language and the options with worked examples, and the
[playground](https://tractables.github.io/tddlog/) runs them in the browser.

## Performance

tddlog was run against every exact inference system we could find, on
five kinds of problems. Each kind is one set of instances, the programs
written in its languages, and every system of the kind is measured on
all of them: it is given each one, through a translation where it cannot
read the original, and an instance it is not given, cannot express or
does not answer counts as unanswered. ProbLog, PITA and aspmc are on
every kind, since tddlog reads ProbLog and they are given the program
tddlog reads. Each run is single-threaded, with a limit of 300 or 600
seconds, and every answer is checked against an exact reference.

"N× faster" compares total times over the instances some system of the
kind answers, the same instances for every system: a run that does not
answer counts as its time limit, and a time under 0.2 s counts as 0.2 s.
The number in brackets is the same over the instances both answer. The
cactus plots count the instances each system answers within each time
limit, and the scatter plots put tddlog's time against the fastest other
system's, instance by instance. The
[results page](https://tractables.github.io/tddlog/results/) has every
instance and time,
[docs/baselines.md](https://github.com/Tractables/tddlog/blob/baselines/docs/baselines.md)
how each system was given each suite and what it was not given, and each
[benchmark's page](https://tractables.github.io/tddlog/results/benchmarks/)
the program as its authors wrote it and as each system was given it.

### Probabilistic logic programs

<p align="center">
  <img src="docs/images/results-programs.svg"
       alt="Probabilistic logic programs answered within each time limit, and tddlog's time against the fastest other system's">
</p>

- **49× faster than ProbLog** (71× where both answer), **79× faster
  than PITA** and **146× faster than aspmc**.
- tddlog answers 303 of the 307 programs, ProbLog 236.
- On Biomine, ProbLog's own benchmark, tddlog answers all 10 queries and
  ProbLog 4.

### Bayesian and Markov networks

<p align="center">
  <img src="docs/images/results-networks.svg"
       alt="Networks answered within each time limit, and tddlog's time against the fastest other system's">
</p>

- **11× faster than Ace** (49× where both answer) and **9.1× faster
  than Merlin** (8.8×). Against TensorInference.jl, libDAI, pyAgrum and
  pgmpy, 10× to 15× faster.
- tddlog answers 604 of the 652 networks, TensorInference.jl 459,
  libDAI 452 and Merlin 443.
- Where tddlog loses: 15 partition functions of the UAI 2022 final set,
  14 of them protein networks, run out of time or memory.
  TensorInference.jl answers each in 45 to 85 s.

### Probabilistic programs

<p align="center">
  <img src="docs/images/results-ppl.svg"
       alt="Probabilistic programs answered within each time limit, and tddlog's time against the fastest other system's">
</p>

- **8.8× faster than Alea.jl**, **14× faster than Roulette** and **16×
  faster than Dice**, and 22× to 34× faster than Geni, Genfer, noDice,
  SPPL, Pluck and Psi.
- **9.4× faster than Storm** and **15× faster than PRISM**, the
  probabilistic model checkers.
- tddlog answers 222 of the 232 programs, Storm 176 and Alea.jl 175.
- Where others win: PRISM and Storm answer Geni's three largest router
  networks, which tddlog does not, by value iteration over decision
  diagrams in double precision; their exact engines time out from 18 to
  20 routers.

### Bit-blasted circuits

<p align="center">
  <img src="docs/images/results-circuits.svg"
       alt="Bit-blasted circuits answered within each time limit, and tddlog's time against the fastest other system's">
</p>

Programs over integers written in Alea.jl, which compiles each to a
Boolean circuit. Alea.jl runs its own program; every other system is
given the circuit.

- **4.1× faster than Alea.jl** (1.7× where both answer) and **5.7×
  faster than Roulette**.
- tddlog answers 201 of the 224 circuits, Alea.jl 135.
- Where others win: Alea.jl on 65 circuits, six of which tddlog does not
  answer, and Roulette on 34; Genfer on the population models, which it
  answers in milliseconds; Psi on two linear regressions it solves
  symbolically.

### Probabilistic databases

<p align="center">
  <img src="docs/images/results-databases.svg"
       alt="TPC-H queries answered within each time limit, and tddlog's time against ProvSQL's">
</p>

- ProvSQL is fast on the easy TPC-H queries, its own benchmarks, and
  faster than tddlog on 107 of them.
- tddlog answers more of the hard ones, 206 of the 208 against
  ProvSQL's 196, and is **10× faster in total** (1.3× where both
  answer).

### Setup

The release run compared tddlog with ProbLog 2.3.0, PITA, aspmc, Ace
3.0, Merlin 1.7.0, pyAgrum 3.2.1, pgmpy 1.1.2, libDAI 0.3.2,
TensorInference.jl 0.6.3, ProvSQL 1.13.0, Dice, Alea.jl, Roulette and
the further systems, on lab and cloud machines. A suite's systems ran in
the same job, side by side in parallel lanes, so times compare within a
suite. tddlog's rows come from a rerun of its current version on the
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

## Using tddlog

**The language** is the Datalog fragment of ProbLog: probabilistic facts
and rules, annotated disjunctions, negation as failure (also through
recursion), comparisons and integer arithmetic, `query/1` and
`evidence/1,2`. Function symbols, lists, cut and `findall` are not
supported.

**The command line** is `tddlog [options] program.pl`.
`--evidence-probability` also prints the probability of the evidence,
`--ground` prints the relevant ground program, `--stats` reports the
compilation, and `--vtree` chooses how the diagrams order the program's
variables (the default, `auto`, tries several layouts under growing
budgets). `tddlog --help` lists every option.

**Bayesian and Markov networks** in the BIF, Hugin, GeNIe and UAI formats
become programs through `scripts/gen_bn.py`; the answers are the
marginals given the evidence:

```
python3 scripts/gen_bn.py tests/regress/markov.uai --evid tests/regress/markov.uai.evid > markov.pl
./target/release/tddlog markov.pl
x0(v_0):        0.181818
x0(v_1):        0.818182
x1(v_0):        0.151515
x1(v_1):        0.848485
```

**As a library**, `tddlog::solve` takes a program's text and the command
line's options:

```rust
let options = tddlog::Options { evidence_probability: true, ..tddlog::Options::default() };
let solution = tddlog::solve(program, &options, &mut |_| {})?;
for answer in solution.answers.as_deref().unwrap_or_default() {
    println!("{answer}: {}", answer.probability);
}
```

The same call is wrapped for [Python](bindings/python/README.md)
(`tddlog.solve(program).answers`), [C](bindings/c/README.md) and
[WebAssembly](bindings/wasm/README.md). The
[API documentation](https://tractables.github.io/tddlog/api/tddlog/)
covers the rest.

## How it works

tddlog grounds the part of the program the queries and evidence depend
on, simplifies it as an and-inverter graph, and compiles each derived
atom into a TDD, one strongly connected component at a time, by
Tp-compilation (Vlasselaer et al., IJCAI 2015; Bogaerts and Van den
Broeck, ICLP 2015, for negation through recursion). The layout of the
diagrams' variables, their vtree, comes from tree decompositions of the
program computed with [goatd](https://github.com/Tractables/goatd). The
evidence is conjoined bottom up along the vtree and each query is
counted against it. [docs/design.md](docs/design.md) describes each
stage.

## Building from source

tddlog needs Rust 1.88 or newer and a C++ compiler (gcc 12 or newer, or a
recent clang) for goatd's vendored solver:

```
git clone https://github.com/Tractables/tddlog
cd tddlog
cargo build --release
./target/release/tddlog examples/alarm.pl
burglary:       0.357143
earthquake:     0.714286
calls(mary):    0.400000
```

`cargo test` checks tddlog against ProbLog's answers on its examples and
on ProbLog's own system tests. [CONTRIBUTING.md](CONTRIBUTING.md) lists
the checks a change runs.

## Citing

tddlog has no accompanying paper yet, so cite the software:

```bibtex
@misc{tddlog,
  author       = {Van den Broeck, Guy},
  title        = {tddlog: Exact Probabilistic Inference by Compilation into Tree Decision Diagrams},
  year         = {2026},
  howpublished = {\url{https://github.com/Tractables/tddlog}},
  note         = {Rust library and command-line tool, version 0.1.0}
}
```

## License

Apache-2.0 ([LICENSE](LICENSE)). The copies of third-party programs and
data in `tests/problog` and `data/` keep their own licenses
([NOTICE](NOTICE)); the components compiled into the binaries are listed
in [THIRD-PARTY.md](THIRD-PARTY.md).
