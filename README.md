# tddlog

<p align="center">
  <a href="https://tractables.github.io/tddlog/"><img
     src="https://img.shields.io/badge/run-in%20the%20browser-blue" alt="Run in the browser"></a>
  <a href="https://tractables.github.io/tddlog/guide/"><img
     src="https://img.shields.io/badge/guide-worked%20examples-blue" alt="Guide"></a>
  <a href="https://tractables.github.io/tddlog/results/"><img
     src="https://img.shields.io/badge/results-against%20the%20baselines-blue" alt="Results"></a>
  <a href="https://pypi.org/project/tddlog/"><img
     src="https://img.shields.io/pypi/v/tddlog.svg" alt="PyPI"></a>
  <a href="https://github.com/Tractables/tddlog/releases/latest"><img
     src="https://img.shields.io/github/v/release/Tractables/tddlog" alt="GitHub release"></a>
  <a href="https://github.com/Tractables/tddlog/actions/workflows/ci.yml"><img
     src="https://github.com/Tractables/tddlog/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://tractables.github.io/tddlog/api/tddlog/"><img
     src="https://img.shields.io/badge/docs-API-blue" alt="API documentation"></a>
</p>

tddlog computes exact marginal probabilities for probabilistic logic
programs in the Datalog fragment of ProbLog. It compiles the program by
Tp-compilation (Vlasselaer et al., IJCAI 2015 and IJAR 2016; Bogaerts and
Van den Broeck, ICLP 2015, for negation through recursion) into tree
decision diagrams, using the [tididi](https://github.com/Tractables/tididi)
library, and counts the queries on the diagrams. The playground at
<https://tractables.github.io/tddlog/> runs it in the browser, the
[guide](https://tractables.github.io/tddlog/guide/) walks through the
language and the command line with worked examples, and the
[results page](https://tractables.github.io/tddlog/results/) compares it
with ProbLog, PITA, aspmc, Ace, Merlin, Dice, Alea.jl and Roulette.

## Results

The release run: tddlog 0.1.0, linked statically, against every
baseline, each suite's systems in one job (three jobs on two lab
machines (36 and 20 physical cores, 503 GB each), 2026-10-19 to
2026-10-20), each run single-threaded and pinned to a core of its own,
the systems' runs side by side in parallel lanes, so the systems compare
within each suite. The baselines are ProbLog 2.3.0 (d-DNNF, SDD and
forward SDD compilation), PITA and aspmc on the program suites, Ace 3.0
(its default route and c2d forced) and Merlin 1.7.0 (bucket-tree
elimination in C++) on the networks, with pyAgrum 3.2.1 (junction tree),
pgmpy 1.1.2 (variable elimination), libDAI 0.3.2 (junction tree) and
TensorInference.jl 0.6.3 (tensor network contraction, Julia 1.11.9)
beside them, and ProvSQL 1.13.0 on TPC-H.

| suite | instances | tddlog finishes | faster than ProbLog, PITA, aspmc and Ace | faster than Merlin | faster than the other network tools (pyAgrum, pgmpy, libDAI and TensorInference.jl), on the TPC-H row ProvSQL |
|---|---:|---:|---:|---:|---:|
| standard: graphs, grids, smokers, UW-CSE, Cora, networks, Dice programs (300 s) | 84 | 82 | 60 of 75, 15 of them under 0.1 s for both | 8 of the 34 networks, 24 of them under 0.1 s for both | 8 of the 34 instances a tool answers, 24 of them under 0.1 s for both |
| literature: mnist_sum, pedigree, sickness, noisy-or, mastermind, pdb (300 s) | 39 | 39 | 23 of 24, 1 of them under 0.1 s for both | | |
| Ace's benchmark networks (600 s) | 46 | 46 | 46 of 46 | 13 of the 25 Merlin answers, 8 of them under 0.1 s for both | 14 of the 25 instances a tool answers, 8 of them under 0.1 s for both |
| large networks: barley, diabetes, mildew, munin1 to munin4 (600 s) | 14 | 14 | 14 of 14 | 6 of 14 | 5 of the 14 instances a tool answers |
| UAI 2022 MAR tuning set: Markov and Bayesian networks (300 s) | 120 | 120 | | 101 of the 114 Merlin answers, 13 of them under 0.1 s for both | 107 of the 120 instances a tool answers, 13 of them under 0.1 s for both |
| TPC-H: the queries and their views at scale factors 0.001 to 0.1 (600 s) | 208 | 205 | 171 of 172 (ProbLog, PITA and aspmc only) | | 34 of the 196 ProvSQL answers, 29 of them under 0.1 s for both |

The last column counts tddlog against the fastest of pyAgrum, pgmpy,
libDAI and TensorInference.jl on each network instance that at least one
of them answers, and against ProvSQL on the TPC-H row.

An instance that tddlog and the baseline both finish in under 0.1 s
counts neither as faster nor as slower: a few milliseconds of start-up
are not compared.

Leaving aside the instances both finish in under 0.1 s, tddlog is faster
than the program systems wherever one of them finished, by a median of
33 times on the standard suite, 36 on the literature suite, 63 on Ace's
networks and 10 on the large networks; the narrowest margins are uwcse2
(2.1 times ProbLog's SDDs), bn_mildew (2.7), ace_munin4 (8.6) and
mastermind6_4_8 (14 times ProbLog's SDDs). It finishes 22 instances no
exact system finishes, g30_100 among them; the two it does not finish
(grid25 and grid30) none finishes.

Merlin is the strongest baseline on graphical models. On the standard
suite's 34 networks the two both take under 0.1 s on 24; of the other 10
Merlin is faster on 2, by 3.9 to 4.4 times on pathfinder (0.07 s against
0.30); tddlog is faster on 8, among them link (0.30 s against 29.1 with
evidence), munin, water and pigs. It takes 3.9 s over the 34 against
Merlin's 64.5. On the large networks Merlin is faster on barley, mildew,
munin2 and munin3 with and without evidence, by 1.05 to 1.5 times
(bn_barley 1.57 s against 1.82), and tddlog on diabetes, munin1 and
munin4 with and without evidence (bn_munin1 0.94 s against 26.4). Merlin
answers 25 of Ace's 46 networks, the larger blockmap and fs networks
running out of memory or time, and is faster on 4 of them
(ace_pathfinder 0.06 s against 0.18), the two both taking under 0.1 s on
8 more.

pyAgrum, pgmpy, libDAI and TensorInference.jl run the same 214 networks
from the same files (TensorInference.jl's two contraction orders count
as one tool, the faster of them). pyAgrum answers 180; tddlog is faster
on 168 of the 180 that both answer in 0.1 s or more and slower on 11
(bn_mildew 0.31 s against 1.64, bn_mildew_ev 0.45 s against 1.56 and
bn_barley 0.77 s against 1.82). pgmpy answers 102; tddlog is faster on
101 of the 102 that both answer in 0.1 s or more and slower on 1
(bn_mildew 1.43 s against 1.64). libDAI answers 193; tddlog is faster on
140 of the 148 that both answer in 0.1 s or more and slower on 8
(bn_mildew 0.44 s against 1.64, bn_mildew_ev 0.45 s against 1.56 and
bn_pathfinder 0.08 s against 0.31). TensorInference.jl answers 185;
tddlog is faster on 185 of the 185 that both answer in 0.1 s or more.
Merlin is faster than pyAgrum on 91 of the 180 instances, pgmpy on 100
of the 102 instances, libDAI on 109 of the 140 instances and
TensorInference.jl on 158 of the 179 instances, counting those that both
take 0.1 s or more for.

On the UAI suite tddlog answers all 120 and Merlin 114. The two both
take under 0.1 s on 13 of the 114 Merlin answers; tddlog is faster on
all 101 others (332.4 s against 3 895 over all 114): all three CSP, all
six DBN, all 65 ObjectDetection, the two Grids Merlin answers, all three
Pedigree, all 16 Promedus and all six Segmentation; Merlin prints `-nan`
for Alchemy_11 and five of the eight Grids.

Every answer agrees with its reference (ProbLog, Ace or Merlin, or an
exact solver written for this repository: `scripts/lit_exact.py` on the
literature suite, `scripts/bn_exact.py` on the networks): to 4.9e-9 on
the standard suite (ProbLog prints eight digits), 2.7e-13 on the
literature suite, 6.3e-15 on Ace's networks, 4.9e-11 on the large
networks and 9.1e-14 on the 111 UAI networks the reference answers (the
other six agree with Merlin's six printed digits); no other system
answers Grids_16, Grids_17 and Grids_18, so they are unchecked. The
other network tools' answers against the same references: pyAgrum 180
marginal runs to 2.0e-7 and 108 partition functions to 5.6e-9 in log10;
pgmpy 102 marginal runs to 1.2e-10 and 103 partition functions to 2.2e-7
in log10; libDAI 190 marginal runs to 5.0e-7 and 120 partition functions
to 2.2e-7 in log10; TensorInference.jl 281 marginal runs to 5.0e-7 and
176 partition functions to 2.2e-7 in log10 (the tools print six digits,
as Merlin does). ProvSQL's answers on TPC-H agree with the reference's
to 1.0e-12 on 196 instances. The graphs no exact system finishes agree
with estimates from sampled graphs (`scripts/sample_paths.py`). The
tables and the setup are in [docs/benchmarks.md](docs/benchmarks.md),
which also compares tddlog with Dice, Alea.jl, Roulette, Geni, noDice,
Psi, Genfer, SPPL, Pluck, Storm and PRISM on the programs of their
papers; the [results page](https://tractables.github.io/tddlog/results/)
plots every suite. The code that runs the baselines and the copies of
their programs are not part of the release: they are on the repository's
[`baselines` branch][baselines], whose README says, for each baseline,
which version ran on which programs and which script reproduces each
published number.

## Building

tddlog is a Rust crate (edition 2024, Rust 1.88 or newer). It depends on
two libraries from their public repositories (`Cargo.toml`): tididi, the
TDD library, and goatd, the tree-decomposition library. goatd's build
script compiles a vendored C++ solver, so building needs a C++ compiler
(gcc 12 or newer, or a recent clang; `GOATD_CXX` names another).

```
git clone https://github.com/Tractables/tddlog
cd tddlog
cargo build --release
./target/release/tddlog examples/alarm.pl
burglary:       0.357143
earthquake:     0.714286
calls(mary):    0.400000
```

`cargo build --profile quick` builds without link-time optimization; its
answers are the same as the release build's.

The scripts need Python 3, and numpy for the exact references and the
sampled check (`bn_exact.py`, `lit_exact.py`, `tpch_exact.py`,
`sample_paths.py`). Their tests:

```
python3 -m unittest discover -s tests -p 'test_*.py'
```

## Supported language

A Datalog fragment of ProbLog: probabilistic facts and rules, annotated
disjunctions, negation as failure (also through recursion, where every
possible world has a well-founded model), comparisons and integer
arithmetic on bound terms, `query/1` and `evidence/1,2`. There are no
function symbols, lists, cut, `findall`, `use_module` directives or
non-ground atoms, and rules must be range restricted
(docs/design.md, Language). Of ProbLog 2.3's 109 system tests,
`scripts/problog_suite.py` passes 47, reports 7 expected errors as
errors and 55 as unsupported.

## Usage

```
tddlog [options] <program.pl>
```

prints one `atom:<tab>probability` line per query, like ProbLog.

- `--vtree <strategy>`: the vtree over the probabilistic facts.
  - `auto` (default): trials of the layouts below over rewrites of the
    program, each compiled with its evidence and queries under bounds on
    diagram size and work that double until one finishes.
  - `td`: right-linear over a fact order from a tree decomposition of the
    program's interaction graph; fails where no elimination of the graph
    finishes within its budget.
  - `closed`: components in an order found by local search, each on a
    right-linear spine, bushy where its parts share nothing.
  - `families`, `families:0`: a tree decomposition of the program read
    through indicators (`families:0` without the decomposer's search);
    fails for a program that reads no atom through an indicator.
  - `dynamic`, `dynamic:minfill`: right-linear, changed during the compile
    by a search of rotations over every live diagram.
  - `program`: right-linear in the order the program states its facts, as
    a BDD package building a bit-blasted circuit would.
  - `program:fanin`: right-linear in the fan-in order of BDD variable
    ordering, interleaved over the evidence and the queries as Fujii,
    Ootomo and Hori interleave it: a depth-first walk of the rules from
    each, the deepest first, places each fact as it meets it, and a fact
    an earlier walk placed marks where the next goes, so that the bits
    of the operands an adder or a comparator combines, and those of an
    operand several operations read, come out side by side. Under both
    orders the derived atoms that cross a narrow cut, as a hidden Markov
    model's state between two steps, are named, read through variables
    of their own, where the compiled diagrams show that the names pay.
  - `decision`: the tree of the decisions the grounder read facts as
    guards in, its contexts and the atoms read outside it named, as the
    program orders name theirs; fails for a program with none.

  Every strategy but `auto` is one of `auto`'s trials, under the name
  its `trial:` lines in `--stats` print; docs/design.md describes each.
- `--ground`: print the relevant ground program, as ProbLog, and exit;
  under a strategy other than `auto`, as that strategy compiles it, and
  with `--evidence-probability`, the program `P(e)` is computed on.
- `--evidence-probability`: after the query lines, print the probability
  of the evidence as `P(e):<tab>p<tab>log10 l`, `p` in scientific
  notation and `l` its base-10 logarithm, exact below the least float
  (`0` and `-inf` for inconsistent evidence, `1` and `0` for none). The
  evidence no query depends on is then compiled rather than dropped.
- `--no-queries`: with `--evidence-probability`, leave out the program's
  queries and print the `P(e)` line alone. The program is then ground for
  its evidence alone: in a Bayesian network, only the variables that are
  ancestors of an observed one are compiled.
- `--stats`: print compilation statistics to stderr; a run that fails prints
  the ground program's line and `auto`'s trials so far before the error.
- `--precision <n>`: digits after the decimal point (default 6), of each
  probability and of `P(e)`'s mantissa and logarithm. A probability under
  1e-4 that is not zero is written in scientific notation, as ProbLog
  writes it, with `<n>` digits after the point of its mantissa.
- `-h`, `--help`: the option text.
- `-V`, `--version`: the version of tddlog.

An option given twice, or one that `--ground` would leave unused, is an
error, and so is an unknown one: tddlog exits with status 2 without
reading the program. `--no-queries` without `--evidence-probability`
asks for nothing and exits with status 2 too.

### Bayesian and Markov networks

`scripts/gen_bn.py` writes a discrete network as a ground program:
a Bayesian network in BIF, Hugin (`.net`, `.hugin`), GeNIe (`.xdsl`) or
the UAI format (`BAYES`), and a Markov network in the UAI format
(`MARKOV`), each factor an observed atom.

```
python3 scripts/gen_bn.py examples/asia.net > asia.pl
python3 scripts/gen_bn.py tests/regress/markov.uai --evid tests/regress/markov.uai.evid > markov.pl
./target/release/tddlog markov.pl
x0(v_0):        0.181818
x0(v_1):        0.818182
x1(v_0):        0.151515
x1(v_1):        0.848485
```

The program queries every value of every unobserved variable (`--query`
narrows that), and its answers are the network's marginals given the
evidence. A UAI file's variable i is `x<i>` and its value u `v_<u>`. The
script's docstring has the encoding and the options.

For a Markov network the program's first line is
`% log10 Z - log10 P(e) = c`, and `--evidence-probability` then gives the
partition function given the evidence, the UAI PR task:
`log10 Z = l + c`. With markov.uai's evidence above, c is
`2.9822712330395684` and tddlog prints
`P(e):<tab>1.718750e-1<tab>log10 -0.764787`, so `log10 Z = 2.217484`,
`Z = 165`. `--no-queries` leaves out the marginals, which the PR task
does not ask for. docs/benchmarks.md compares this with Merlin's PR
task on the UAI 2022 sets.

## Use as a library

From Rust, `tddlog::solve` takes a program's text and `tddlog::Options`, the
command line's options, and returns the answers to the program's queries and,
where asked for, `P(e)`. The crate documentation runs this example as a
doctest:

```rust
let program = "
    0.1::burglary.
    0.2::earthquake.
    0.7::hears_alarm(john).
    alarm :- burglary.
    alarm :- earthquake.
    calls(X) :- alarm, hears_alarm(X).
    evidence(calls(john), true).
    query(burglary).
";
let options = tddlog::Options { evidence_probability: true, ..tddlog::Options::default() };
let solution = tddlog::solve(program, &options, &mut |_| {})?;
// `answers` is `None` where the evidence is inconsistent.
for answer in solution.answers.as_deref().unwrap_or_default() {
    println!("{answer}: {}", answer.probability);
}
```

Two programs in `examples/` use the library: `solve.rs` answers a
program file and reports `P(e)` and the layout that answered
(`cargo run --release --example solve -- examples/alarm.pl`), and
`sprinkler.rs` writes a Bayesian network as a program from Rust data and
checks its posteriors against a sum over the network's worlds
(`cargo run --release --example sprinkler`).

The same call is wrapped for Python and C:

- [bindings/python](bindings/python/README.md): the `tddlog` package,
  `pip install tddlog`; `tddlog.solve(program)` returns the answers as a
  dict from query to probability.
- [bindings/c](bindings/c/README.md): a C library with a CMake package and
  a pkg-config file; `tddlog_solve` returns an opaque solution read through
  accessors.

## How it works

1. Parse the program (`src/syntax`).
2. Ground it (`src/ground`): magic sets from the queries and evidence,
   right-linear recursion from its bound end, stratified semi-naive
   bottom-up evaluation, simplification, the gates reduced as an
   and-inverter graph (equal gates merged by structural hashing, and
   gates that random simulation and a SAT check prove equal in function
   merged too), exclusive sets, row classes, and conditioning on what
   the evidence implies.
3. Write each recursive component whose rules read at most one of its
   atoms, positively, without recursion (`src/ground/eliminate.rs`):
   such a component is reachability, and eliminating its nodes one by
   one, as Floyd and Warshall's closure does, leaves an acyclic program
   with the same least model in every world. Every strategy compiles
   the rewrite.
4. `auto` (`src/auto/`) picks, trial by trial, an indicator rewrite
   (`src/indicators`), which reads derived atoms through variables of
   their own and ties tables to them by clauses, and a vtree
   (`src/vtree`).
5. Compile each derived atom to a TDD (`src/tp`), strongly connected
   component by component, in the negated form that suits TDD
   conjunction, each component on the vtree projected to its ancestor
   cone.
6. Conjoin the evidence bottom up along the vtree and count every query
   against it (`src/query`, `src/eval`).

[docs/design.md](docs/design.md) describes each stage and the decisions
behind it.

## Repository layout

- `src/`: the crate; docs/design.md maps each module and describes each
  stage.
- `tests/`: `it/`, one crate of differential tests against ProbLog and
  exact sums, of the command line (`cli.rs`) and of the helpers they
  share (`common/`); the programs and answers they read (`problog/`,
  copies of ProbLog's system tests, among them); and `test_*.py`, the
  scripts' tests.
- `examples/`: small programs with known answers, and two Rust
  programs that use the library.
- `bindings/wasm/`: the crate built for WebAssembly, and the site at
  tractables.github.io/tddlog: the playground, the guide, the results page
  and the API documentation (its README).
- `bindings/python/` and `bindings/c/`: the Python package and the C
  library, each a crate of its own over `tddlog::solve`, with its tests
  and examples (their READMEs).
- `scripts/`:
  - `gen_*.py`, the generators of the benchmark families;
  - converters: `lpad2problog.py` (cplint's LPADs), `dice2problog.py`
    (Dice programs) and `uai_wcnf.py` (a network as a weighted CNF);
  - exact references where ProbLog does not finish: `bn_exact.py`,
    `lit_exact.py`, `reliability_exact.py`, `tpch_exact.py`, and the
    sampled check of the graphs, `sample_paths.py`;
  - checks against ProbLog: `compare.py` (tddlog beside ProbLog on a
    program), `problog_suite.py` (its system tests) and
    `fuzz_problog.py` (random programs).
- `data/`: the inputs of the generators and the translators: cplint
  programs (`cplint`) and Dice programs (`dice`), each directory's README
  naming its source and license. The benchmarks' Bayesian networks, from
  the bnlearn repository, are not in the repository, since no license for
  them is established: `data/fetch.txt` names where each is downloaded
  and its SHA-256 (`data/bn/README.md`). `examples/asia.net` and
  `examples/cancer.net` are two small networks written out from their
  publications.
- `docs/`: design, benchmarks, paper notes. CONTRIBUTING.md lists the
  checks a change runs.

The [`baselines` branch][baselines] adds what reproduces the published
comparisons: the benchmark driver, the scripts that run or feed each
baseline, the baselines' benchmark programs and docs/baselines.md, how
to install and run each baseline.

## Validation against ProbLog

- `cargo test` runs `tests/it/examples.rs`, which checks `examples/*.pl`
  against ProbLog's answers under `auto`, `closed`, `dynamic` and
  `families`, and `tests/it/problog_suite.rs`, which does the same for
  ProbLog's system tests in `tests/problog/` under `auto`, `closed`,
  `dynamic`, `program` and `families`. The other tests check one mechanism
  each against an exact sum or a hand count; `td` is checked against
  ProbLog's `P(e)` in `tests/it/evidence_probability.rs`.
- `python3 -m unittest discover -s tests -p 'test_*.py'` tests the
  scripts: `bn_exact.py` and `sample_paths.py` against hand counts, the
  generators, the Dice translator against an exact evaluation of the
  Dice program, and the weighted CNF of a UAI network against
  enumeration.
- `scripts/problog_suite.py --dir PROBLOG/test` runs the 109 system tests
  of a ProbLog 2.3.0 checkout through the binary and classifies pass,
  fail, unsupported and expected error.
- `scripts/compare.py` runs a program through tddlog and ProbLog and
  diffs the probabilities; `scripts/fuzz_problog.py` does it for random
  programs in the supported fragment.
- The benchmark driver (on the `baselines` branch) checks every
  benchmark answer against the suite's exact reference where it has one
  (`scripts/lit_exact.py`, `tpch_exact.py`, `reliability_exact.py`),
  else the first ProbLog compiler that finished, else
  `scripts/bn_exact.py` on the networks, else Ace.
- `tddlog --ground` prints the ground program as ProbLog, which ProbLog
  accepts, so grounding can be checked on its own.
- The bindings' tests (`pytest bindings/python/tests` against an installed
  wheel, `python bindings/c/check.py`) check their answers against
  ProbLog's on a dozen of its system tests in `tests/problog/`.

## License

Apache-2.0 (LICENSE). The copies of third-party programs and data in
`tests/problog` and `data/` keep their own licenses (NOTICE). The binaries
carry third-party components compiled in, whose notices are in
THIRD-PARTY.md.

[baselines]: https://github.com/Tractables/tddlog/tree/baselines
