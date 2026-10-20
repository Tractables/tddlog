# Baseline inference engines

The reference systems tddlog is checked and compared against, how to
install each, and how `scripts/bench.py` runs it. Nothing is installed
system-wide. `tmp/` is a directory of the checkout that git ignores,
where `bench.py` and `problog_suite.py` look for ProbLog and its tests
by default; `$PREFIX` is a directory outside the checkout with room for
builds:

- venv `tmp/venv` (`problog==2.3.0 PySDD==1.0.6 pyeda==0.29.0`, aspmc)
- ProbLog's sources `tmp/baselines/problog-src`, Scallop's
  `$PREFIX/scallop-src`
- Ace 3.0 `$PREFIX/ace_v3.0_linux86` (section 7)
- Merlin `$PREFIX/merlin/build/merlin` (section 8)
- Ganak and sharpSAT-TD, static binaries outside the checkout (section 9)
- Julia 1.11 with Alea.jl and its package depot outside the checkout
  (section 5)
- Racket 8.16 with Roulette outside the checkout (section 11)
- the network, database and model-checking baselines of sections 20 to 22 under `$HOME/local/code`

| system | exact | input | `bench.py run` option |
|---|---|---|---|
| ProbLog 2.3.0 | yes (`ddnnf`, `sdd`, `fsdd`, `bdd`) | programs | `--problog`, `--kbs` |
| PITA / cplint (SWI-Prolog 9.2.9) | yes | programs, through `problog2pita.py` | `--pita`, `--pita-packs` |
| Scallop | no (top-k proofs) | ground programs, through `ground2scallop.py` | `--scallop`, `--scallop-k` |
| Dice | yes | Dice programs and networks | `--dice` |
| Alea.jl (Julia 1.11.9) | yes | Dice programs and networks, through `dice2alea.py` | `--alea` |
| aspmc 1.1.1 | yes | programs without evidence | `--aspmc` |
| Ace 3.0 | yes | networks | `--ace`, `--ace-variants`, `--ace-heap`, `--ace-java` |
| Merlin 1.7.0 | yes (`bte`, `cte`) | UAI files | `--merlin` |
| pyAgrum (aGrUM) | yes (junction tree, Shafer-Shenoy on Markov networks) | UAI files | `--agrum` |
| pgmpy | yes (variable elimination) | UAI files | `--pgmpy` |
| libDAI 0.3.2 | yes (junction tree, min-fill) | UAI files | `--libdai` |
| TensorInference.jl | yes (tensor network contraction, `treesa`, `greedy`) | UAI files | `--tensorinference` |
| ProvSQL 1.13.0 (PostgreSQL 16.4) | yes (its own method choice, tree decomposition) | the TPC-H programs, as SQL | `--provsql` |
| Storm 1.14 (stormpy) | yes (`sparse`, `exact`, `dd`) | the weather factory, as a PRISM model | `--storm`, `--storm-variants`, `--weather-models` |
| PRISM 4.8.1 | yes (`sparse`, `hybrid`) | the weather factory | `--prism`, `--prism-variants`, `--prism-heap` |
| Ganak | yes, exact rationals | weighted CNFs | `--ganak` |
| sharpSAT-TD | yes, 53-bit MPFR floats | weighted CNFs | `--sharpsat-td`, `--sharpsat-td-decot` |
| Roulette (Racket 8.16) | yes | its own programs (`--suite roulette`) | `--roulette`, `--roulette-variants` |
| Geni (gennifer) | yes, f32 | its own `.gir` programs (`--suite geni`) | `--geni`, `--geni-benchmarks` |
| noDice | yes | Dice programs without `nflip` | `--nodice` |
| Psi (ef8a9e5) | yes, symbolic | Dice programs, through `dice2psi.py`, and its linear regression | `--psi` |
| Pluck | yes | its four sequence models (`--suite ppl`, family `pluck`) | `--pluck` |
| Genfer | yes | its population model and the Dice programs it ships | `--genfer`, `--genfer-benchmarks` |
| SPPL 2.0.4 | yes, symbolic | its Markov switching model | `--sppl` |
| Prodigy, PLIA, Slice, Hakaru | | left out, section 12 | |

`--exact` adds the exact references written for the suites:
`scripts/bn_exact.py` (a clique tree over the rows `gen_bn.py` writes, the
reference for the networks, where ProbLog finishes only sachs),
`lit_exact.py`, `tpch_exact.py`, `reliability_exact.py` and
`gen_roulette.py --reference-of`. A baseline
whose option is empty is not run:

```
python3 scripts/bench.py run --pita $PREFIX/swipl/bin/swipl \
    --pita-packs $PREFIX/swipl-packs --aspmc tmp/venv/bin/python
```

Candidates considered for the network, TPC-H and model checking suites
that are not run (rule 15), each with its reason (section 20 has the
installed ones):

| candidate | reason left out |
|---|---|
| toulbar2 | not an exact marginal or partition function engine here: it answers MAP and its counting-like modes are approximate |
| SamIam | a closed-source GUI application with no batch interface |
| UAI 2022 competition entries | the solvers of the competition's MAR and PR tracks are not released as installable packages (the tuning set's instances are run, section 8) |
| gRain (R) | the junction tree again, on discrete Bayesian networks only; libDAI and pyAgrum cover it and R is not installed on the boxes |
| daoopt | answers MAP (AND/OR branch and bound), not marginals or Z |
| DPMC, Ganak, sharpSAT-TD, d4 as counters | weighted model counters, not network or database methods; Ganak and sharpSAT-TD run on the UAI PR sets (section 9), DPMC needs a CNF encoding of the network that is a second compile route (rule 10) |
| MayBMS, Trio, MystiQ, SPROUT, dissociation | no maintained release builds on a current system, or no probabilistic-query implementation to download; ProvSQL is the maintained exact system |
| Tessa | exact model checker for the weather factory in JAX; needs Python 3.12 with its own stormpy build and was not installed in the time of this run |

`bench.py table` takes the first of `--ref` that finished as the reference
answer and the first of `--pr-ref` for log10 Z; `--approx` (Scallop by
default) gets a column of its own.

---

## 1. ProbLog 2

<https://github.com/ML-KULeuven/problog>. The venv's wheel is what runs;
a clone at `tmp/baselines/problog-src` gives the system tests.

### Install

```
python3 -m venv tmp/venv
tmp/venv/bin/pip install problog PySDD pyeda
export PATH=$PWD/tmp/venv/bin:$PATH
```

The wheel bundles `dsharp` and `maxsatz`, so the default backend needs no
setup. `PySDD` enables `sdd`, `sddx` and `fsdd`; `pyeda` enables `bdd`
(not the `dd` package).

### Backends

`problog -k <backend> file.pl` prints the marginals; bench.py runs that
for each backend in `--kbs` (by default `ddnnf,sdd,fsdd`), as the systems
`problog:<backend>`.

- `ddnnf` (the default): ground, break cycles, Clark's completion to a
  CNF, compile with dsharp.
- `sdd`, `sddx`: the ground formula compiled to an SDD.
- `fsdd`: Tp-compilation (`problog/forward.py`, `ForwardSDD`), from the
  possibly cyclic ground formula, without cycle breaking or CNF. The
  ProbLog implementation of the method tddlog follows
  ([papers/tp-compilation.md](papers/tp-compilation.md)).
- `fbdd` crashes upstream (`'_ForwardBDD' object has no attribute
  'init_varcount'`); use `fsdd`.

A program that is not stratified is refused at grounding
(`NegativeCycle`); there is no well-founded fallback. `problog ground
--format cnf --break-cycles -v` prints the DIMACS CNF with the atom names
as comments; `--format internal` prints the ground node table.

### Gotchas

- ProbLog 2.3.0's `-k ddnnf` answers `tests/lit/pedigree.pl` wrongly (by
  0.078) when asked all twelve queries at once and right when asked one at
  a time. The repository's checks against ProbLog ask one query at a time.
- Without PySDD the system tests print "No SDD support" and test only
  `ddnnf`.

### The system tests

`tmp/baselines/problog-src/test/` holds 243 `.pl` files, 109 of them core
marginals; `tests/problog/` keeps copies (Apache 2.0). Expected results
live in comments, in three forms:

1. A block after `%Expected outcome:`, each following `% ` line
   `<query term> <probability>`, ended by the first line not starting
   with `%`.
2. Inline: `query(q_001). % outcome: 1`.
3. Expected errors: `% ERROR <ExceptionClassName>`, e.g.
   `InconsistentEvidenceError`.

`tests/it/problog_suite.rs` and `scripts/problog_suite.py` (default `--dir`
the clone's `test/`) run tddlog on them. The upstream suite itself:
`cd tmp/baselines/problog-src && ../../venv/bin/python -m pytest
problog/test/test_system.py -q` (111 tests, 1643 subtests).

---

## 2. PITA / cplint

PITA (Probabilistic Inference with Tabling and Answer subsumption) is part
of cplint by Fabrizio Riguzzi (<https://friguzzi.github.io/cplint/>). It
transforms the program, tables it with answer subsumption and compiles
BDDs with CUDD.

### Install

SWI-Prolog from source, without root, with the packs beside it:

```
cd $PREFIX/src
curl -sSLO https://www.swi-prolog.org/download/stable/src/swipl-9.2.9.tar.gz
tar xf swipl-9.2.9.tar.gz && cd swipl-9.2.9
cmake -S . -B build -DCMAKE_INSTALL_PREFIX=$PREFIX/swipl \
      -DCMAKE_BUILD_TYPE=Release -DSWIPL_PACKAGES_X=OFF \
      -DSWIPL_PACKAGES_JAVA=OFF -DSWIPL_PACKAGES_QT=OFF -DSWIPL_PACKAGES_TERM=OFF
cmake --build build -j 6 && cmake --install build
$PREFIX/swipl/bin/swipl -g "pack_install(cplint, [interactive(false), \
    package_directory('$PREFIX/swipl-packs')])" -t halt
```

Only `libgmp` and a C++ compiler are needed; `pack_install` needs the
`http` and `ssl` packages (built above) and pulls `bddem`, `auc` and
`matrix`. bddem ships a prebuilt library, older than cplint's `pita.pl`:
consulting prints `bddem:create_json_util/5 is not defined`, harmless
(`prob/2` never calls it).

### How `bench.py` runs it

`scripts/problog2pita.py` writes `prog.lpad.pl` from the ProbLog program,
and one swipl process consults it and asks `prob/2` (or `prob/3` with
evidence) for every query:

```
swipl -q -g "attach_packs('$PREFIX/swipl-packs',[duplicate(replace)])" \
      -g "consult('prog.lpad.pl'), main" -t halt
```

The translation has to:

- table every predicate that can reach itself (without it the smokers
  programs loop);
- expand `query(Q) :- Body.` by copying the facts the body reads out of
  the LPAD block under a prefixed name, since PITA rewrites every
  predicate inside the block;
- turn `evidence/1,2` into the second argument of `prob/3`;
- write a fact of probability one as a plain fact: PITA reads `a:1.` as
  false (a rule `a:1 :- Body.` is read right).

Syntax against ProbLog: postfix annotations (`a:0.6 ; b:0.4 :- Body.`),
the probabilistic part between `:- begin_lpad.` and `:- end_lpad.` after
`:- use_module(library(pita)).` and `:- pita.`, probabilities may be
fractions, queries are goals (`prob(Q, P)`, `prob(Q, E, P)`).

PITA answers one query at a time and shares nothing between queries; that
is the system, not the harness. Its answers agree with ProbLog's to
ProbLog's printed precision (4.9e-9) on the graph, grid, smokers, uwcse,
cora, bloodtype, network and Dice families.

`scripts/lpad2problog.py` converts cplint's example LPADs to ProbLog, with
the `?- prob(Q, P)` queries of their `/** <examples> */` block; 28 of the
29 in tddlog's fragment agree with ProbLog to 1e-6. `data/cplint/`
(Artistic License 2.0) keeps `bloodtype.pl`, `uwcse3s.pl` and the Cora
theory with fold m1 of cplint_datasets (`cora_m1.pl`, input of
`scripts/gen_cora.py`).

---

## 3. Scallop

<https://github.com/scallop-lang/scallop> (clone `$PREFIX/scallop-src`,
`scallop-core` 0.2.5). A Datalog with provenance semirings, in Rust.

### Build

It needs nightly Rust (`#![feature(min_specialization)]`); current
nightly builds it.

```
cd $PREFIX/scallop-src
CARGO_TARGET_DIR=$PREFIX/cargo-target cargo +nightly build --release --bin scli
```

`scli file.scl -p <provenance> -k <k>` prints one line per relation,
`name: {tag::tuple, ...}`.

### Not exact

Scallop offers no exact probabilistic inference out of the box:

- `topkproofs` keeps the k most probable proofs of each tuple
  (`retain_top_k`) and computes an exact WMC over that subset of the DNF:
  a lower bound, exact only once k reaches the number of proofs on the
  way. Negation goes through `cnf2dnf_k`, also capped at k. The default k
  is 3. Five independent facts deriving one head (true answer 0.96875)
  give 0.5, 0.75, 0.875 at k = 1, 2, 3.
- `proofs` keeps every proof but applies no WMC; `prob_proofs`, the exact
  one, is dead code with no CLI name.
- `minmaxprob` and `addmultprob` are approximations.

A probability on a rule is a fresh random variable per derivation: `0.95
:: alarm() :- earthquake() or burglary()` gives `1-(1-0.05*0.95)(1-0.90*0.95)
= 0.8619`, where ProbLog's shared rule gives 0.85975.

### How `bench.py` runs it

`scripts/ground2scallop.py` translates the ground program `tddlog
--ground` prints, where annotated disjunctions and rule probabilities are
already independent switch facts, so the per-derivation semantics never
arises. `bench.py run --scallop <scli> --scallop-k 3,1000` runs one
variant `topk<k>` per k (by default 1000), and for a program with evidence
divides `q<i>e` by `e`, since Scallop does not condition. The seconds
include the grounding.

The error reaches zero only on the smallest programs; from bn_asia up,
the k that would make it exact runs out of time first (a conjunction of
two k-proof DNFs forms k² products before it truncates). Scallop is in the
harness as the approximation it is (`--approx`), not as an exact baseline.

---

## 4. Dice

Dice (Holtzen, Van den Broeck, Millstein, OOPSLA 2020;
<https://github.com/SHoltzen/dice>) is a functional probabilistic language
(`flip`, `discrete`, `if`, `observe`, tuples, functions) compiled to a BDD
in the order of the program text.

### Run

The docker image is the simplest install (`docker pull sholtzen/dice`);
the OCaml build needs opam. The image's binary
(`/home/opam/.opam/4.09.0/bin/dice`, `dice -version` 1.0) needs only
glibc and runs outside the container once copied out (`docker create`,
then `docker cp -L`). `dice FILE` prints the distribution of the returned expression (`-show-size`
adds the BDD size). `bench.py run --dice` takes a command template with
`{file}`, `{dir}` and `{base}`:

    --dice "docker run --rm -v {dir}:/b sholtzen/dice dice /b/{base}"
    --dice "/path/to/dice {file}"

For a `dice` instance it runs the source program; for a `bn` instance one
program per queried variable, written by `gen_bn.py --format dice --query
VAR` (a returned tuple of all variables would print the joint).

### Programs

- `data/dice/`: the eight programs of the paper's comparison with Psi and
  WebPPL, translated by `scripts/dice2problog.py` (all agree with Dice),
  and the loop programs (caesar-iter, simpleCaesar, pmc1, pmc2,
  recursion from Dice's repository; diamond and ladder from dicebench's
  generators), which use integers, `iterate` and recursion. `bench.py
  gen --suite indicators --families dice` adds the loop programs.
- `gen_bn.py --format dice` writes a network as nested conditionals over
  `discrete`s, the encoding of Dice's `bngen`.
- Dice 1.0 prints an all-zero distribution whenever the evidence has
  probability below 1e-6: a program that observes a flip of probability
  9e-7 prints 0 for every value, one of 2e-6 the posterior (checked on
  the image's binary). The repository's driver compared the evidence's
  weight with zero to within 1e-6 (`Util.within_epsilon`) before
  dividing from June 2020 to August 2021 (80a6ecc to e13ddab; the
  paper's tag, oopsla2020, has no such test); later drivers test for
  exactly zero (b3ab324) and divide in log space (651987e).
  digitRecognition (P(e) = 9.6e-241, a double), caesar-iter (2.0e-904)
  and simpleCaesar (4.0e-1215) fall under the cutoff, as do the circuits
  of the Caesar programs of `--suite ppl` from five observed characters
  on; ProbLog, Alea and tddlog give the posterior.
- Dice 1.0 also drops small values of a `discrete`. It encodes the
  value in bits, each a flip whose probability is the mass of the
  values with that bit set, among those that agree on the bits before
  it, over the mass of all of those; where that numerator is below 1e-4
  the flip is 0 (`gen_discrete` in lib/Passes.ml, whose
  `within_epsilon` is 1e-4 from b664d44 on). A value other than 0 of
  probability below 1e-4 then has probability 0, unless a value with
  the same bit set makes up the mass, and its mass goes to the values
  with that bit clear: `discrete(0.5, 0.00005, 0.49995)` prints 0.50005,
  0 and 0.49995, `discrete(0.00005, 0.99995)` itself (checked on the
  image's binary). A flip keeps its probability. Of data/bn's networks,
  insurance (106 entries), andes (106), munin (216) and pathfinder
  (906) have probabilities below 1e-4; Dice's marginals are off on
  insurance and pathfinder (benchmarks.md, "Dice and Alea.jl"), right on andes to
  4.7e-13, and it times out on munin.
- Dice accepts a `discrete` whose list sums to one within 1e-4 and
  normalizes it; dice2problog.py (and dice2alea.py through it) divides
  such a list by its sum, as noDice's networks need (thirds printed as
  0.333333).
- Dice 1.0 recurses through a program's nested `let`s on the system
  stack (8 MB by default) and stops with `Stack overflow` on the
  largest of the circuits `alea_bench.jl` writes, a `let` per gate: on
  the seven of 118 560 to 532 827 gates (benchmarks.md, "The programs of
  the Dice and Alea.jl papers"). Under `ulimit -s unlimited` it gets past
  that point; bench.py runs it with the stack it inherits, and the
  bit-blasting runs under `prlimit --stack=unlimited`.
- `data/nodice/`: noDice's purely probabilistic programs (data/nodice/
  README.md), in `--suite ppl` (below).
- A tuple of Booleans returned prints as a joint table; bench.py reads it
  as one marginal per query.

---

## 5. Alea.jl

Alea.jl (<https://github.com/Tractables/Alea.jl>, Apache License 2.0) is
the Julia successor of Dice: a library whose distributions (`flip`,
`discrete`, `DistUInt{W}` for unsigned integers of W bits, tuples) are
Julia values that build an expression graph, which `pr` compiles to a
CUDD BDD to return each query's distribution given the evidence. The
flips order the BDD's variables in the order they are created.

### Install

Alea asks for Julia 1.11 (1.11.9 from julialang-s3, checked against the
SHA-256 of julialang's `versions.json`). The repository has no
Manifest.toml, and its Project.toml takes CUDD.jl and IRTools.jl from
forks through a `[sources]` section, which Pkg honours only in the active
project, so Alea runs as its own project, as its README does:

```
git clone https://github.com/Tractables/Alea.jl && git -C Alea.jl checkout cf5e5b8
export JULIA_DEPOT_PATH=$PREFIX/depot PYTHON=/usr/bin/python3
julia --project=Alea.jl -e 'using Pkg; Pkg.resolve(); Pkg.instantiate(); Pkg.precompile()'
julia --project=Alea.jl -e 'using Alea; println(pr(flip(0.7)))'
```

`PYTHON` is the interpreter PyCall builds against, for Alea's SymPy
dependency; it needs `sympy` (1.9 was used). Julia takes 1 GB and the
depot 2.1 GB, so both go to a local disk. The Manifest Pkg resolved has
357 packages (CUDD.jl 0.3.0 from rtjoa/CUDD.jl `m1compat`, CUDD_jll 3.0.0, IRTools.jl
0.4.14 from PoorvaGarg/IRTools.jl `loop-break-patch`,
DirectedAcyclicGraphs 0.2.0, Distributions 0.25.131, SymPy 2.3.3,
PyCall 1.96.4). `using Alea` takes 2.1 s.

### Translation

`scripts/dice2alea.py program.dice > program.jl` writes a Julia script
from a Dice program, read with dice2problog.py's parser; it supports what
dice2problog.py supports (no lists, `/`, `%`, `nth_bit` or `sample`):

- `flip p`, `discrete(ps)` and `uniform(w, lo, hi)` are Alea's, an
  `int(w)` a `DistUInt{w}`, `==` and `<=>` `prob_equals`. Every flip is
  created where the program has it, so the BDD's order is Dice's.
- `if c then a else b` evaluates both branches and merges them with
  `ifelse`; a branch whose condition is constant is not evaluated.
- `observe e` conjoins `!path | e` to the evidence, `path` the
  conjunction of the conditions of the branches around it, which is what
  Alea's `@alea` macro does with an observation. The macro is not used:
  under it `+`, `-` and `*` raise an error on overflow, where outside it
  they wrap modulo 2^w as Dice's do. `>>` is a helper that shifts the
  bits, which Alea does not have.
- A function is a Julia function of the path and its arguments,
  `iterate(f, x, k)` a loop of k calls, and a function that calls itself
  observes false past Dice's recursion limit, as in dice2problog.py.
- A `discrete` row that sums to just over one by rounding in the source
  (digitRecognition's prior, 1.000004) is divided by its sum, as for
  ProbLog: Alea's `discrete` asserts that the sum is one.
- The translation is long straight-line code (a network's tables,
  digitRecognition's 7 830 observations), on which Julia's optimizer
  spends more time than running it takes. The script is therefore a
  module whose own functions Julia interprets
  (`Base.Experimental.@compiler_options compile=min optimize=0
  infer=false`), while Alea's methods compile as usual. In a probe on a
  lab machine beside other jobs this gave the same answers and cut
  the in-process time of bn_pathfinder from 191 s (184 s of it
  compilation) to 10 s, of bn_water from 24 s to 1.7 s, of
  digitRecognition from 16 s to 2.1 s, of bn_munin from 296 s to 190 s
  and of pmc2 at 10 steps from 4.0 s to 3.3 s; `optimize=0` alone gave
  111 s, 15 s, 8.6 s, 245 s and 3.5 s. On diamond, caesar-iter and
  geometric, whose functions are short, the three differ by less than
  4 percent. A long body is also split into functions of about 400
  statements, since lowering takes time that grows faster than a
  function's length.

The program's value is queried by one call of `pr`, one distribution per
component of a returned tuple; the script prints `alea K VALUE P` for
every value of the K-th component. The work runs on a task with a 1 GiB
stack, since Alea's compilation and probability computation recurse as
deep as a loop is long.

### How `bench.py` runs it

`bench.py run --alea` takes a command template with `{file}`, `{dir}` and
`{base}`, run on the translated script (the translation is not timed):

    --alea "julia --startup-file=no --threads=1 --gcthreads=1 --project=Alea.jl {file}"

It runs on the `dice` and `bn` families: a `dice` instance's source
program, and for a `bn` instance one program that returns the tuple of
the queried variables (`gen_bn.py --format dice --query leaves`), all
answered by the one `pr`. A Boolean component answers one query, an
integer one the queries of one predicate, value k the k-th.

The script loads Alea and runs a small program through `pr` first, so
that Julia compiles Alea's common methods, then times building the
program and running `pr` with `@timed` and prints `alea_seconds` and
`alea_compile_seconds`. bench.py records the wall time of the Julia
process as `seconds`, as for every system, and the two printed times as
`inproc_seconds` and `jit_seconds`; `bench.py table` prints both times,
`alea:cudd` (wall) and `alea:cudd in-process`. The wall time includes
Julia's start, `using Alea`, the warm-up and the parsing of the script;
the in-process time includes Julia's compilation of the Alea methods the
program calls (`jit_seconds`).

### Alea.jl programs (`alea_bench.jl`)

The Alea.jl programs of `--suite ppl` (data/alea/README.md) run as they
are in Alea, and tddlog, ProbLog and Dice run the Boolean circuit Alea
builds for them, which `scripts/alea_bench.jl` writes. Alea represents
every Boolean of a program as a DAG of flips, `&`, `|` and `!`, and a
number as a vector of such Booleans; `pr` compiles the queried bits and
the conjunction of the observations into one CUDD BDD whose variables are
the flips in the order they were created. The circuit is therefore the
formula Alea compiles, and the comparison is between compilers of one
formula.

A program file defines `model()`, returning what an `@alea` block
returns, and `QUERY`: `:pr` answers a Boolean (`qK`, the K-th component
of the value) or the distribution of a number over the worlds of its bits
that are not constant (`qK(U)`, at most 16 bits), `:expectation` the
probability of each bit of a number (`eK(I)`), from which the expectation
is a linear sum, as Alea's `expectation` computes it. A returned tuple is
answered one component at a time, where Alea's `pr(c)` of a tuple gives
the joint distribution. `IGNORE_ERRORS = true` stands for `pr(c;
ignore_errors = true)`; the circuit never has the error conditions, and a
program run without it stops in Alea where an error has positive
probability.

    julia --project=Alea.jl alea_bench.jl run [--warm-below S] [--set NAME=VALUE ...] PROGRAM.jl [PREFIX]
    julia --project=Alea.jl alea_bench.jl export [--set NAME=VALUE ...] PROGRAM.jl PREFIX

`--set NAME=VALUE` runs the program with its one line `const NAME = ...`
reading `const NAME = VALUE`: a size, such as the bits after the binary
point of its fixed-point numbers (`--suite bitblast`, below).

`export` writes PREFIX.pl, a ProbLog program (a flip `P::fN.`, numbered
in creation order, an `&` one rule, an `|` two, `!` a negated literal,
each observation `evidence/2`, each answer a query defined by its
literals), and PREFIX__dJ.dice, Dice programs over the same circuit with
the flips first in Alea's order (so that Dice's BDD has Alea's variable
order), one per Boolean or `:pr` number (returning its bits as nested
pairs, printed as a joint table) or per expectation bit, listed in
PREFIX.dicemap. The rules come in a depth-first post-order from the
answers' literals and then the observations, every gate after the gates
it reads. tddlog's formulas depend on that order: on some programs the
same gates in another order give formulas many times smaller or larger
(benchmarks.md, "The programs of the Dice and Alea.jl papers"). `run`
answers with Alea's `pr` and prints `alea_q ATOM
P`, then `alea_seconds` (building the program and `pr`) and
`alea_compile_seconds` (Julia's compilation within it). The `@alea` macro
runs the program under an IRTools dynamo, so Julia compiles Alea's
methods anew for every program, which can take most of the first run
(benchmarks.md, "The programs of the Dice and Alea.jl papers"). `run`
then builds and answers the program a
second time in the process and prints `alea_warm_seconds`, the time
without compilation, which is what the papers report (the median of
BenchmarkTools' runs). `--warm-below S` skips the second run when the
first took S seconds or more; bench.py passes a third of its timeout.
Where the warm run took less than S/2, `run` compiles the same roots a
third time, in the same order and without reordering, and prints the
sizes of the BDDs in nodes (`Cudd_DagSize`): `alea_bdd_evidence` (the
conjunction of the observations), `alea_bdd_query_max` (the largest of a
queried bit), `alea_bdd_joint_max` (the largest conjunction of a queried
bit with the evidence) and `alea_bdd_peak` (`Cudd_ReadPeakNodeCount`,
the most nodes the manager held). bench.py records them as
`bdd_evidence`, `bdd_query_max`, `bdd_joint_max` and `bdd_peak`.

`bench.py gen --suite ppl --alea CMD` writes the circuits (`CMD` as for
`run --alea`); `bench.py run` gives tddlog and ProbLog the ProbLog
program, Dice the Dice programs (`dice:bdd`, the seconds summed), and
Alea the program itself (`alea:cudd`, with `inproc_seconds`,
`jit_seconds` and `warm_seconds`; `bench.py table` adds the column
`alea:cudd warm`). tests/test_alea_bench.py checks the circuit's answers
against Alea's on two small programs where `TDDLOG_ALEA` names Julia with
Alea (a command template like `--alea`).

### Benchmark programs of the Dice and Alea.jl papers

`bench.py --suite ppl` holds the benchmark programs of the papers on
Dice and Alea.jl, and of later papers that compare with them, that are
exact inference on a discrete (or bit-blasted) program: 62 instances in
four families. The survey of 2026-10-11:

| paper | artifact (license) | programs | here |
|---|---|---|---|
| Dice, OOPSLA 2020 | SHoltzen/dice (Apache-2.0) | Table 1's eight against Psi; digit recognition; Fig. 10's Caesar, diamond and ladder generators; Tables 2 and 3's networks | `data/dice`, the `bn` family (before this survey) |
| Cao et al., "Scaling Integer Arithmetic in Probabilistic Programs", UAI 2023 | Tractables/Alea.jl branch `arithmetic` 7e9ff23 (Apache-2.0) | Table 1's 25 programs (book, tugofwar, caesar, ranking, radar, Floyd-Warshall, linext, triangle, gcd, disease, luhn), integers of 2 to 20 bits | `arith`: all 25 and caesar_huge, disease_large, plankton (`data/alea/arith`) |
| Garg et al., "Bit Blasting Probabilistic Programs", PLDI 2024 | Alea.jl branch `hybit` 96b69a5 (Apache-2.0); Zenodo 10.5281/zenodo.10901544 (CC-BY-4.0) | Table 2's 19 programs, continuous distributions as fixed-point numbers of 8 to 25 bits and 16 to 4 096 pieces | `hybit`: all 19 at the bits and pieces of the paper's HyBit column (`data/alea/hybit`) |
| Bowers et al., "Stochastic Lazy Knowledge Compilation for Inference in Discrete Probabilistic Programs" (Pluck), PLDI 2025 | Zenodo 10.5281/zenodo.15051140 (CC-BY-4.0) | Table 1 against Dice.jl: the Dice paper's networks, diamond and ladder; four sequence models (HMM, PCFG, string editing, sorted lists) on inductive types | `pluck`: the four sequence models' Dice.jl programs ported to Alea.jl (`data/alea/pluck`) |
| Gürtler and Kaminski, noDice, OOPSLA 2026 | Zenodo 10.5281/zenodo.18739789 (CC-BY-4.0, code Apache-2.0) | Table 1's Dice programs, some with nondeterminism (`nflip`) | `nodice`: the 11 without `nflip` (`data/nodice`) |

The `arith`, `hybit` and `pluck` programs are Alea.jl programs: Alea
runs them, and tddlog, ProbLog and Dice run the circuit Alea builds for
them (below). The `nodice` ones are Dice programs, translated by
`dice2problog.py`. Left out:

- Dice's Fig. 10a (Caesar with errors) is written in the paper's
  one-hot integers (`int(26)`, whose `+` is mod 26), which Dice 1.0 and
  dice2problog.py do not have; Table 1's Coin Bias is not in the
  repository's history. Cao et al.'s Fig. 5 microbenchmarks (a < b,
  a == b, E[a + b]) are plotted, not tabulated, and the directory's
  `disease_*_modified` variants are not in its Table 1.
- Roulette (Moy et al., PLDI 2025) runs the Dice paper's programs; the
  ones it adds (a Markov chain, an HMM) duplicate the tree's.
- PLIA (De Smet and Zuidberg Dos Martires, NeurIPS 2024) reuses Cao et
  al.'s microbenchmarks and luhn.
- Slice (OOPSLA 2026, Zenodo 10.5281/zenodo.21780754, MIT) cites the
  bit-blasting paper; its programs (eight baselines, decision-tree
  fairness programs) are in Slice's own language with continuous
  distributions and become Dice programs only through Slice's OCaml
  discretizer, which the artifact does not run ahead of time. Open.
- noDice's other programs choose nondeterministically (`nflip`), which
  no system here has.

`bench.py --suite bitblast` runs the bit-blasting paper's programs on
ladders of size: each Table 2 program (`data/alea/hybit`) at the paper's
precision and at 2, 4 and 6 bits below and above it, the gene expression
program of the paper's Figures 1 and 2 from 5 to 100 genes (the paper's
5 to 50) and with 5 genes by precision, and the Yao-Vehtari-Gelman model
and the conjugate Gaussians of its Figures 3 and 5 by precision
(`data/alea/bitblast`): 154 instances in 23 families, a family a program
and ladder, named by the width of its numbers in bits (Table 2's column
`Bit`) or by its genes. Table 2 reports each
system's error against the ground truth within 20 minutes, not HyBit's
time. The paper's other systems are samplers (WebPPL's rejection, MCMC
and SMC; Stan's HMC on programs whose discrete variables SlicStan
marginalizes), discretizers (AQUA, GuBPI) and the exact symbolic Psi
(its Table 3).

---

## 6. aspmc

<https://github.com/raki123/aspmc> (Eiter, Hecher and Kiesel). `pip
install aspmc` into the venv; it pulls clingo and ships the `sharpsat-td`
and `flow-cutter` binaries it needs. It reads ProbLog (`-m problog`),
breaks cycles by Tp-unfolding, does a Clark completion guided by a tree
decomposition and counts the CNF with a compiling sharpSAT-TD: the
published system closest to tddlog's aim.

```
tmp/venv/bin/python -m aspmc.main -m problog -c -v result prog.pl
```

It writes `out.cnf` into the working directory, so bench.py gives each
instance a directory of its own and reads `[RESULT] aspmc: <query>: <p>`
from stderr.

Its ProbLog frontend does not condition on `evidence/1,2`: it reads them
as facts and returns unconditional probabilities with exit code zero.
bench.py therefore refuses a program with evidence before running aspmc.
It rejects some programs with a parser error (bn_insurance); check the
coverage on the installation a run uses.

---

## 7. Ace

Ace 3.0 (Chavira and Darwiche; <http://reasoning.cs.ucla.edu/ace/>)
compiles a Bayesian network into an arithmetic circuit, by tabular
variable elimination or by encoding it as CNF and compiling with c2d,
and answers every marginal with one upward and one downward pass. It runs
on the network file each `gen_bn.py` instance is written from. Ace and its
benchmarks are licensed for non-commercial research and educational use
only, so neither is in this or any other repository.

### Install

1. Java 8 or later (checked with OpenJDK 21).
2. The 32-bit loader `/lib/ld-linux.so.2`: `c2d_linux` is a 32-bit
   binary. Without it only the tabular route works.
3. `tar xzf ace_v3.0_linux86.tar.gz` and pass that directory as `--ace`
   (it holds `ace.jar`, `inflib.jar`, `jdom.jar`, `c2d_linux`,
   `libcallsmile.so`). bench.py calls `java` itself (`--ace-java`), not
   the `compile` and `evaluate` scripts.
4. For `--suite ace`, unzip `benchmarks_v1.0.zip` (directories
   `2004-pgm`, `2005-ijcai`, `2006-ijar`, each network with its `.inst`
   evidence files) and pass `benchmarks_v1.0` as `--ace-benchmarks` to
   `gen` and `run`.
5. Check: `bench.py run --ace DIR --strategies '' --kbs '' --only
   bn_asia_ev --ace-variants auto,forceC2d` runs `tabularCompilation` and
   then `c2dCompilation`.

### How `bench.py` runs it

```
java -Xmx8g -Djava.util.Arrays.useLegacyMergeSort=true -Djava.io.tmpdir=D -DACEC2D=ACE/c2d_linux \
     -classpath ACE/ace.jar:ACE/inflib.jar:ACE/jdom.jar \
     edu.ucla.belief.ace.AceCompile -Xnormalize [options] [-e x.inst] x.net
java ... edu.ucla.belief.ace.AceEvaluate x.net x.inst
```

in a scratch directory `D` of the run's own.

- **The network** is the file `gen_bn.py` read. Ace's lexer refuses a `+`
  in a number (`0.000000e+00`), so the copy drops every `+` before a digit
  or a point outside quotes and comments (`hugin_for_ace`), and
  `gen_bn.py` reads the same copy.
- **`-Xnormalize`**, an option the readme leaves out, divides every row by
  its sum, as tddlog does with a row within 1e-8 of one; without it Ace's
  marginals differ by up to 7e-8 (pathfinder). `gen_bn.py --rescale`
  divides the rows of the Ace suite's networks written in percent.
- **The evidence** is chosen in `gen_bn.py` and written beside the program
  as an `.inst` file (`--inst`). Ace evaluates with it, and compiles with
  it (`-e`) when it is not empty, as tddlog compiles for one evidence set.
  Ace's evidence-free compile is not run.
- **The sort.** Ace's unit resolution sorts with a comparator that is not
  a total order, which Java 7 and later refuse; the system property
  restores the legacy merge sort.
- **The heap**, `--ace-heap` (8g by default), decides the route: Ace
  compiles tabularly when the entries of the best of six min-fill orders
  fit in the free heap, through c2d otherwise. With 8g every network of
  data/bn and data/bn-large but munin1 goes tabular. Compare routes, not
  only times, and give a box's runs one heap.
- **Randomness.** Min-fill and dtree ties are broken at random from the
  clock unless `-Xseed N` (variant `Xseed=N`).

The record's `seconds` cover both JVM runs; `ace_method`, the circuit's
size and Ace's own timings are kept beside them. The answers are read
from `Pr(X,e) = [...]` through `gen_bn.py`'s names.

### Variants

`--ace-variants`: `auto` gives no option; any other variant names options
without the dash, joined by `+`, with `=` before an argument
(`forceC2d+dtHypergraph=3`). `AceCompile` accepts `-forceC2d |
-forceTabular`, `-d02 | -sbk05 | -cd05 | -cd06`, `-dtBnOrder |
-dtClauseMinfill | -dtHypergraph <count>`, `-noEclause`, `-e <file>`; the
readme's `-noTabular` and `-dtBnMinfill` are `-forceC2d` and `-dtBnOrder`.
The readme's settings for its published results:

| readme | networks | variant |
|---|---|---|
| `-noTabular -d02 -dtHypergraph 3` | `2006-ijar` | `forceC2d+d02+dtHypergraph=3` |
| `-noTabular -cd05 -dtClauseMinfill` | munin1 to munin4 | `forceC2d+cd05+dtClauseMinfill` |
| `-noTabular -cd05 -dtBnMinfill` | other IJCAI 2005 networks | `forceC2d+cd05+dtBnOrder` |
| `-noTabular -cd06 -dtBnMinfill` | SAT 2006 | `forceC2d+cd06+dtBnOrder` |

**sbk05 with evidence is wrong**: compiled with `-e`, bn_alarm_ev's
circuit gives P(e) = 0.01503 against 0.02740. bench.py records an sbk05
variant on an instance with evidence as an error.

Ace and tddlog agree to 4.6e-15 on the bn and bn_ev networks and to
1.5e-15 on the Ace suite (checked 2026-10-01). On sachs and insurance both
differ from `bn_exact.py` by up to 1.2e-10, where rows short of one by
less than 1e-9 are left short by the clique tree.

### What does not convert

- tcc4f.obfuscated: noisy-max nodes, which `gen_bn.py` refuses.
- The `2005-ijcai` copies of blockmap_05_03 and mastermind_03_08_03 are
  `2004-pgm`'s files; the suite takes them once.
- `mildew-0007.inst` and `-0008.inst` declare UTF-8 and hold Latin-1
  bytes; the suite uses `-0006`.

### UAI networks

Ace's UAI reader takes every file as a Bayesian network and prints `NaN`
marginals on a Markov network. On `--suite uai` bench.py gives Ace the
network `gen_bn.py` makes of the file, written as a Hugin file
(`write_hugin`): each factor a variable of two values, `ok` and `not_ok`,
observed `ok`, so Ace compiles the network tddlog compiles. `--pr` with
`--ace` is refused: Ace prints P(e) to six digits in a double, which the
PR set's networks underflow.

---

## 8. Merlin

Merlin (Radu Marinescu, BSD 3-clause; <https://github.com/radum2275/merlin>)
answers PR, MAR, MAP and MMAP on Bayesian and Markov networks in the UAI
format. Its exact MAR routes are bucket-tree (`-a bte`) and clique-tree
(`-a cte`) elimination along a min-fill order. It is the graphical-model
baseline of the UAI suite.

### Build

```
git clone https://github.com/radum2275/merlin $PREFIX/merlin
cd $PREFIX/merlin && mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release && cmake --build . -j 8
```

It needs CMake and boost (`program_options`, `thread`); built at ad37783.
A static build (`-DBoost_USE_STATIC_LIBS=ON
-DCMAKE_EXE_LINKER_FLAGS=-static`) runs on machines without boost.

### How `bench.py` runs it

`bench.py run --merlin BIN` runs `merlin -f NAME.uai -e NAME.uai.evid -a
bte -t MAR -O uai -o OUT` per network instance and reads `OUT.MAR`: a
`STATUS` line (`false` when the evidence is inconsistent or underflows)
and the marginals, to six decimals, so they differ from the others' by up
to 5e-7. The record keeps the induced width (`merlin_width`). `gen_bn.py`
names variable i of a UAI file `x<i>` and its value u `v_<u>`. A network in
another format is read from the UAI copy `bench.py gen` writes beside the
program (`gen_bn.py --uai`). `--merlin` without a network among the
instances is refused.

### The PR task

`bench.py run --suite uai --pr` compares the partition function given the
evidence (`--pr` is for the uai suite). With `--merlin` it
runs Merlin again per instance as `merlin:bte-pr`, `-t PR`, and reads
`OUT.PR`: the natural logarithm of Z to six decimals, recorded as
`log10_z` (up to 2.2e-7 off). Merlin multiplies in doubles and takes the
logarithm at the bucket tree's roots: the status is `false` where the
exponential underflows, the logarithm is `-inf` for inconsistent evidence
or an underflowed message (read as zero), `inf` for an overflow (an
error). Without evidence Merlin leaves out a factor of empty scope
(tests/regress/markov_pr.uai: Z = 199.47 against 797.88); no instance of
the UAI sets has one. tddlog runs as `s+evidence-probability` and the
clique tree with `--evidence-probability`, each giving `log10_z` as the
`log10 P(e)` it prints plus the constant `gen_bn.py` writes on the
program's first line, `% log10 Z - log10 P(e) = c`. `bench.py gen --pr`
writes the programs for this task alone (`gen_bn.py --query none`): no
queries and no `.inst` files, and on a Markov network without evidence
whose tables are all Potts tables over two variables of k values (the
final PR set's queen5_5_3, queen5_5_4, myciel5g_3 and myciel5g_4) the
colour symmetry broken, which divides Z by k and adds log10 k to c
(`gen_bn.colour_symmetry`). Programs written so answer `run --pr` alone;
their marginals are not the network's.

### Known failures

Merlin multiplies in floating point, not in logarithms: on Alchemy_11,
Grids_13 and Grids_14 the partition function overflows a double and it
writes `PR inf` and `-nan` marginals with a true status, which bench.py
counts as an error. Elsewhere its marginals agree with tddlog's to 5e-7
(checked 2026-10-04).

---

## 9. Ganak and sharpSAT-TD

Two model counters give a second reference for log10 Z on the UAI
networks under `--pr`: the weighted model count of the network's CNF
given the evidence. They check answers and are not compared on time.

- Ganak (Sharma, Roy, Soos and Meel, IJCAI 2019; MIT;
  <https://github.com/meelgroup/ganak>), a static binary: `--ganak BIN`
  runs `BIN --mode 1 --prob 0 NAME.cnf`, weights in exact rationals and
  the probabilistic hashing of its cache off, and reads `c s exact arb
  frac N/D`.
- sharpSAT-TD (Korhonen and Järvisalo, CP 2021; MIT;
  <https://github.com/Laakeri/sharpsat-td>), a static build at 0c234c1
  with `flow_cutter_pace17` in the same directory: `--sharpsat-td BIN`
  runs `BIN -WE -decot S -decow 100 -tmpdir DIR -cs C NAME.cnf` in 53-bit
  MPFR floats, after S seconds of FlowCutter (`--sharpsat-td-decot`, by
  default 1), with a cache of C 10^6 bytes (half the address-space
  limit, less 500), and reads `c s log10-estimate`.

`bench.py gen --cnf` writes the CNF beside each program (`gen_bn.py
--cnf`, `scripts/uai_wcnf.py`) from the UAI file and its evidence, by the
encoding of Darwiche (KR 2002) and Chavira and Darwiche (AIJ 2008):

- a variable of two values is one Boolean variable, true at value 1; a
  variable of k > 2 values is k indicators with one clause that one holds
  and k(k-1)/2 that no two do; a variable of one value has none;
- each entry other than 0 and 1 is a parameter variable equivalent to the
  conjunction of its assignment's indicators, weighted by the entry, its
  negation by 1; an entry of 0 is a clause ruling its assignment out, an
  entry of 1 needs nothing;
- each observation is a unit clause on its indicator.

The weighted count is then Z given the evidence (P(e) for a Bayesian
network). Both literals of a parameter get a weight line, since the
format leaves 1 - w to the other literal of a weight given alone.
`tests/test_uai_wcnf.py` checks the count against enumeration, and with
`GANAK` or `SHARPSAT_TD` set to a binary, that counter's count as well.

---

## 10. The bit-blasting paper's other systems

The bit-blasting paper (Garg, Holtzen, Van den Broeck and Millstein,
PLDI 2024) compares HyBit, Alea.jl's bit blasting, with samplers
(WebPPL's rejection, MCMC and SMC; Stan's HMC), discretizers (AQUA,
GuBPI) and the exact symbolic Psi, each on its own program of the same
model, from the artifact (Alea.jl branch `hybit` at 96b69a5,
`baselines/`). These programs are continuous: the samplers estimate the
continuous model's answer and Psi computes it in closed form, where
tddlog, Dice and Alea answer the bit-blasted program exactly. They are
compared by their error against the ground truth of the paper's
notebook (`benchmarks/plotting.ipynb`), not by agreement with Alea.
`scripts/samplers/` installs Stan, WebPPL and Psi and runs them on the
artifact's programs, which `data/hybit` copies (its README); the
baselines branch's README gives the commands.

### Psi

eth-sri/psi at ef8a9e5 (2025-03-13, Boost Software License 1.0) with
its submodules tgehr/ast e343546, tgehr/sym a0d42b7 and tgehr/util
0d44034, built with LDC 1.32.0 (the version Psi's build script pins) by
one `ldmd2 -O -release -inline -boundscheck=off` call; Psi's own tests
pass (389 passed, 239 skipped, no regressions). The install checks the
LDC tarball's MD5 against
Psi's `dependencies-release.sh` and its SHA-256 against the first
download's. Psi prints a density or distribution in closed form, with
integrals it could not do left in it; the paper simplified those with
Mathematica, which is not here. With 300 s each:

- In closed form: weekend (P = 0.374206175427), GPA (85/139),
  coinBias (5/12), clinicalTrial2 (2/7), the gene expression program
  with 5 and 10 genes (1/2), and, after a rewrite, altermu2 (E[mu1] =
  0.155061748300) and conjugate_gaussians (E = 1).
- With integrals left: pi, spacex, tug_of_war, trueskill, addFun_max
  (1/√π by quadrature of its one integral).
- Out of time: zeroone, clinicalTrial1, normal_mixture, the gene
  expression program with 20 and 50 genes.

The rewrite: altermu2, conjugate_gaussians and normal_mixture observe a
Gaussian's value with `observe(gauss(m, v) == y)`, which this Psi treats
as an event of probability zero (`Pr[error] = 1`); `cobserve(gauss(m, v),
y)` conditions on the density, as the paper's runs must have. The
artifact has no Psi program for addFun_sum, clickGraph, and the programs
of Figures 3 and 5 (multimodal, conjugate_ex3).

### Stan

CmdStan 2.32.2 (stan-dev/cmdstan, BSD 3-clause, released 2023-05-15),
the release tarball checked against the SHA-256 of the first download
(e8174f74ec14dfc05dc2d2c1360e0098fcb93fdfe4a81b7527bf5a2fab3db40a; the
publisher gives none for this release). The artifact's models were
built with CmdStan 2.28.2.
2.32.2 is the last release series that compiles both the old array
syntax (`int y[N];`) of 12 of the artifact's .stan files and the
`array[N] int` of four others; 2.33 rejects the old one. A run is one
chain with 100 warm-up iterations, as in the artifact's gene expression
runs, and N samples, from 10^3 up to 10^7, ten times more each run,
until a run fails or takes more than 60 s (the next would pass 600 s);
above 10^6 it keeps every (N/10^6)-th draw. Three seeds. Not run:
`GPA.stan`, a change-point model unrelated to GPA, with no data in the
artifact (Table 2 marks Stan unsupported on GPA). The gene expression
program's models above 30 genes declare arrays larger than a 32-bit
`int` and do not compile. Stan's answer for pi is P(1), whose
complement is the ground truth's P(even).

`weekend.stan` answers another model: it divides each hours density by
P(hours >= 0) (0.89435 and 0.69146, the normal CDF at 5/4 and 1/2), not
by P(0 <= hours <= 8), the truncation of the HyBit and Psi programs.
Its model's answer is 0.330742 by hand, which Stan prints at every
sample count (the model has no parameter left; the program sums the
weekend flag out); the ground truth is 0.374206. Table 2 marks Stan
unsupported on weekend. Stan's zeroone model is HyBit's (a correctly
classified point weighs e times a misclassified one), but its density
is a step function, and HMC's estimates of w1 and w2 stay off by 0.7 to
2.9 at every sample count. On the multimodal model Stan's chain stays
in the mode of mu1 near -5 (E[mu1] = 35/9).

### WebPPL

WebPPL 0.9.15 and webppl-timeit 0.3.0 (npm, MIT), installed with `npm
ci` from a lock file that pins all 45 packages by version and SHA-512
on node 18 (the system's, or v18.20.4
from nodejs.org, checked against its SHASUMS256.txt). The artifact's
programs run with `--m METHOD --s N` for rejection sampling, MCMC (its
default kernel) and SMC (N particles), N from 10^2 up to 10^7, ten
times more each run, under the same stopping rule as Stan, three seeds.
Not run: `GPA2.wppl` and `conjugate_gaussians2`, which plot.

Three of the artifact's WebPPL programs answer another model, each
checked by hand against what the estimates settle on:

- `weekend.wppl` leaves out the truncation of the hours to [0, 8]: its
  answer is 0.389946, not 0.374206.
- `clinicalTrial2.wppl` maps a function of two arguments over a list of
  pairs, so its first argument is a pair, never equal to 1, and every
  observation reads as a failure: its answer is 1/7, not 2/7.
- `zeroone.wppl` and `zeroone2.wppl` condition on hard constraints
  (`x*w1 + w2 < 0` where the label is 1) in place of HyBit's penalty of
  1/e for a misclassified point, with the opposite sign. No weights
  meet all of them, and `zeroone.wppl` (E[w1]) fails or times out.
  `zeroone2.wppl` (E[w2]) maps a function of two arguments over the
  data with one list, so its label is undefined and every point gets
  the same constraint: its answer is 5.3348, not 3.6888. The paper's
  notebook reads E[w2] from `zeroone2.wppl`, as here.

### AQUA and GuBPI

AQUA (uiuc-arc/AQUA at 527eea9, 2022-06-04; the repository states no
license) builds with Maven 3.9.9 on OpenJDK 21, its pom's open versions
pinned to what they resolved to (junit 4.13.2, commons-math3 3.6.1,
renjin 3.5-beta76). AQUA at this commit fixes its grid at 61 splits and
ignores the second argument, so the artifact's `run.sh`, which loops
over grids of 2^0 to 2^20, runs one analysis 21 times. The artifact's
AQUA inputs (`baselines/aqua`) do not give Table 2's quantities: pi,
weekend, spacex, addFun_sum and the gene expression program crash AQUA;
coinBias's template has no observation and its Stan program answers
another quantity; GPA's observation is not applied (AQUA gives the
prior's 0.25); conjugate_gaussians's Stan program writes `target +=
normal(y[n], 1)`, a draw rather than a log density, so the data are
ignored and the answer is the prior mean 0, as in the artifact's own
results files; tug of war and clickGraph do not finish in 300 s. Only
altermu2, normal_mixture, zeroone and the multimodal model give numbers.
AQUA is therefore not timed here.

GuBPI (gubpi-tool/gubpi at 47e9a21, MIT) builds with the .NET SDK
7.0.410. Its repository has programs for coinBias, the sum of two
Gaussians and a tug of war with another query, and none for the other
models of Table 2; the artifact has none. GuBPI prints bounds on a
histogram of the posterior, not a point answer. It is not timed here.

Mathematica, with which the paper simplified Psi's integrals, is not
available.

## 11. Roulette

Roulette (Moy, Czenszak, Li, Marshall and Holtzen, PLDI 2025, DOI
10.1145/3729334) is a discrete probabilistic language embedded in Racket
through Rosette's symbolic evaluation, which compiles a program's
distribution to a BDD (rsdd) in the order its flips are created. The
paper compares it with Dice on Markov chains, networks of diamonds and
ladders, a Caesar cipher, a hidden Markov model with five queries, nine
Bayesian networks and two programs Dice cannot write (a lossy network
and a perturbed-memory interpreter).

### Install

From the paper's artifact (Zenodo 10.5281/zenodo.15253254, CC-BY-4.0, a
2.6 GB Docker image, md5 99bd2e7417aa52092396009e922cb5ec), whose
Roulette is camoy/roulette at 54d6185f0b5143bdaabbd9ed35bb22c9dcaddc60
(Apache License 2.0; the repository is not public, and the public
neuppl/roulette head 60f77cb does not compile on Racket 8.13). The paper
ran Racket 8.13 with Rosette 4.1; the image has Racket 8.16, which runs
the artifact's source. The install puts everything under PREFIX:

- Racket 8.16 minimal (`racket-minimal-8.16-x86_64-linux-cs.sh`, SHA-256
  5439d606ddbac8cbc72e7d9ec690e6fee97e49d439f9f0d2b1f19a3b4dce5101);
- the image's layers that hold the Roulette source and `librsdd.so`
  (built there from camoy/rsdd branch `more-ffi`, MIT-style license),
  cut into a 36 MB tarball, so a machine that has the tarball skips the
  2.6 GB download and needs no Rust;
- rosette, memo and mischief from the 8.16 release catalog, and the
  Roulette packages linked from the source, with `PLTUSERHOME`,
  `PLTADDONDIR` and `TMPDIR` under PREFIX. The image's compiled files
  do not load on this Racket and are rebuilt by `raco setup`.

### How `bench.py` runs it

`--suite roulette` holds the programs (gen_roulette.py writes their
ProbLog and Dice versions, data/roulette/README.md the drivers) on
ladders of size, and the nine networks with the variable the artifact
queries. `--roulette CMD` runs Roulette, CMD a template for Racket with
`{file}`, e.g.

    --roulette "env PLTUSERHOME=$PREFIX/home PLTADDONDIR=$PREFIX/addon $PREFIX/racket/bin/racket {file}"

on the artifact's program for the instance: a driver of data/roulette/,
or for the hmm queries the artifact's `benchmark/hmm.rkt` under
`--roulette-benchmarks` (the artifact's `benchmark/` directory, which
also holds the networks' .bif files). The time is the process's;
Roulette's own `(time ...)` of building and querying the program, without
Racket's start and loading, is kept as `inproc_seconds`. A value Roulette
leaves out of its distribution answers 0.

`--roulette-variants rsdd,nocomplement` adds caesar queried with `(query
e #f)`. Roulette's default `query` takes one value's probability as one
minus the others', which loses that value once the evidence's
probability is below about 1e-16 (caesar from 25 letters on prints 0 for
the most likely key); `nocomplement` gives every value to 1e-13.

The reference is `--exact` (`exact:dp`): `gen_roulette.py --reference-of`
computes each program's answers by a dynamic program over the model's
states, in logarithms where they fall below the doubles (the hmm
marginal likelihood from about 915 steps, the caesar evidence). The
references agree with an exact enumeration of the Dice programs and with
Roulette (tests/test_gen_roulette.py), and tddlog's answers on small
programs with them (tests/it/examples.rs, `mod roulette`).

### Other suites' programs

`--roulette` also runs on the Dice programs of the other suites (the
Dice paper's programs and the standard networks, `--suite standard`
families dice and bn; the loops, `--suite indicators`; noDice's, `--suite
ppl`; GeNI's, `--suite geni`): scripts/dice2roulette.py translates the
instance's Dice program into Roulette's language, which `raco make`
compiles, both untimed, before the program runs. The translation holds
an int(w) as a list of w Booleans, least significant bit first, draws
`discrete` as Dice does, creates the flips in Dice's order, and queries
each component of the program's value with `(query e #f)` under
`--roulette-variants nocomplement` or `(query e)` otherwise; the answers
map to the instance's queries as Alea.jl's do. The program's statements
are forms of the module, not a function's body: Racket CS interprets a
large module's forms, where it took longer than 120 s to compile a
function of the size of munin's or pathfinder's.
tests/test_dice2roulette.py checks the translation, and with
`ROULETTE_RACKET` set (the command that runs Racket, without `{file}`)
Roulette's answers on small programs against an exact enumeration.

---

## 12. Program baselines of the survey

A suite taken from a paper runs that paper's system, and every other exact
system that accepts its programs runs on it too. The survey of exact
systems for probabilistic programs, 2026-10-19:

| system | exact | available | decision |
|---|---|---|---|
| Geni (gennifer), ICFP 2025 | yes (generating functions, f32) | github.com/geni-icfp25-ae/geni-icfp25-ae, MIT | run on `--suite geni`: `--geni` |
| Genfer, NeurIPS 2023 | yes (generating functions; rational arithmetic where the program allows) | github.com/fzaiser/genfer, MIT | run on its own programs: `--genfer` |
| noDice, OOPSLA 2026 | yes | Zenodo 10.5281/zenodo.18739789, code Apache-2.0 | run: `--nodice` |
| Psi | yes (symbolic, imperative) | eth-sri/psi, Boost | run on Dice programs and on its own linear regression: `--psi` |
| SPPL, PLDI 2021 | yes (symbolic, sum-product) | PyPI `sppl` 2.0.4, Apache-2.0 | run on its own Markov switching model: `--sppl` |
| Pluck, PLDI 2025 | yes | Zenodo 10.5281/zenodo.15051140, Julia and Rust | run on the sequence models: `--pluck` |
| Prodigy | yes (generating functions) | github.com/LKlinke/Prodigy, Apache-2.0 | not run, see below |
| PLIA, NeurIPS 2024 | convolution by FFT in the log domain; exactness not stated | the paper (arXiv 2410.12389) links no code | left out: no implementation found |
| Slice, OOPSLA 2026 | yes (discretization of continuous programs into Dice, Roulette or Storm programs) | the paper (arXiv 2608.16093) links no artifact | left out: no implementation found |
| Hakaru | yes (symbolic) | github.com/hakaru-dev/hakaru, BSD-3 | left out: its `simplify` transformation, which reduces a program to a closed form, requires Maple (hakaru-dev.github.io), which is proprietary |

Prodigy is exact and public. Its repository's pyproject.toml needs Python 3.11, GiNaC with CLN, poetry, Z3 and a git dependency
(`pygin`, on git.rwth-aachen.de); the Ubuntu 22.04 box checked has none of GiNaC, CLN, poetry or Python 3.11, and the
Dice programs it can run are the seven that Genfer also runs (Genfer's
repository ships them as `.pgcl`, `benchmarks/neurips2023/exact`). It is not
run.

Where a system's own paper has programs in a suite, it runs those, and the
time is the process's:

| system | its programs | suite, instances |
|---|---|---|
| Geni | the link, router and weather-factory programs | `--suite geni` |
| Genfer | the population model; the Dice programs of its comparison with Dice, Prodigy and PSI | `bb_population_sN` of `--suite bitblast`; twocoins, evidence1, evidence2, grass, noisyOr, alarm and murderMystery of `--suite standard` |
| SPPL | the hierarchical Markov switching model | `bb_markov_nN` of `--suite bitblast` |
| Psi | the R2 linear regression | `bb_linreg_wW` of `--suite bitblast` |
| Pluck | the four sequence models | `pluck_*` of `--suite ppl` |

### Geni

`gennifer` from geni-icfp25-ae/geni-icfp25-ae at
9e5ffb25548258849ce58105feef395514c8df94 (MIT). The artifact's Cargo
manifest depends on a private repository (Genfer, in a university's git
server) through an embedded access token, for the `--taylor` option only.
No credential is used here: delete the `genfer = ...` line of
`Cargo.toml`, delete the `genfer` package and its entry in gennifer's
dependency list from `Cargo.lock`, and apply `data/geni/geni-no-genfer.patch`
(it removes the option and the two files that use it, `src/genfun_conversion.rs` and
`src/bin/bench-taylor.rs`). Then `cargo build --release --bin gennifer`
(a Rust toolchain) and copy `target/release/gennifer`.
The same commands as shell, in a clone of the artifact:

    git checkout --detach 9e5ffb25548258849ce58105feef395514c8df94
    sed -i '/^genfer = /d' Cargo.toml
    python3 - <<'EOF'
    t = open('Cargo.lock').read().split('\n\n')
    t = [b for b in t if not b.startswith('[[package]]\nname = "genfer"')]
    open('Cargo.lock', 'w').write('\n\n'.join(t).replace(' "genfer",\n', ''))
    EOF
    git apply data/geni/geni-no-genfer.patch
    cargo build --release --bin gennifer

It runs `gennifer -l -i FILE`; answers are f32, printed with 7 decimals
(`Pr(1) = 0.1234567`), so they differ from the exact reference by up to
5e-8 on the instances tried. `bench.py table` prints the maximum
difference to the reference and applies no tolerance; read 1e-6 as agreement for Geni. A Boolean's two answers are `Pr(1)` (true) and
`Pr(0)`. `scripts/gen_geni.py --gir` writes the `.gir` program of an
instance (link, router and weather-factory families; the weather programs
equal the artifact's files byte for byte); the weather query is Pr(0) (the
artifact's), the grids' Pr(1). `--geni-benchmarks DIR` points at the
artifact's `bench/ICFP`. On the small instances (link 4 and 8, router 4
and 8, weather N = 2 and 7, H = 1 and 2) Geni agrees with the exact
reference (`exact:dp`) to 5e-8 in 0.00 to 0.27 s. The artifact's weather
factory with 10 factories and 1000 days takes 206 s and 8 GB.

### Genfer

github.com/fzaiser/genfer at d132c76 (MIT), the commit the population
program of `data/alea/genfer` is taken from:

    git clone https://github.com/fzaiser/genfer.git
    git -C genfer checkout --detach d132c76
    cd genfer && cargo build --release        # target/release/genfer

`--genfer 'GENFER/target/release/genfer {file}'` with
`--genfer-benchmarks GENFER/benchmarks/neurips2023`. Genfer reads SGCL
(`x ~ Poisson(51.4); observe 45 ~ Binomial(x, 0.2); return x`) and prints
the posterior's moments and, with `--limit L`, its masses p(n)/Z for n
below L.

- The Dice programs of the standard suite that Genfer's repository has
  as programs of its own (`dice_twocoins`, `dice_evidence1`, `dice_evidence2`,
  `dice_grass`, `dice_noisyOr`, `dice_alarm`, `dice_murderMystery`) run as
  its shipped `.sgcl` files with `--rational`, the expected value of the
  returned Boolean being the instance's one query; the answers are exact
  rationals (509/719 for grass) and agree with ProbLog's `ddnnf` to the
  latter's print precision (3.3e-9 at most), in 0.00 to 0.03 s.
- The population model of Winner and Sheldon (`bb_population_sN`) is run
  from `scripts/gen_genfer.py population N`, which writes Genfer's own
  `population.sgcl` for N seasons (N = 4 is the file; it repeats the
  four seasons' rates and counts in turn). The Poisson rates are not
  rational, so it runs in floating point (`--rational` prints "not a
  rational"). `--limit 1024` prints the masses of 0 to 1023, the range of
  the bit-blasted program, and the bit queries of the circuit's ProbLog
  program, `eK(J)`, are sums of them; the instance's expectation is
  also Genfer's `E`. Agreement with Alea.jl's bit probabilities on the
  circuit: 1.4e-14 (one season) and 2.3e-14 (two); with the reference
  means of `data/alea/genfer/README.md`: E = 241.91923199999994 (1
  season), 194.27522836978991 (4) and 194.56199475284632 (16), the
  README's to its digits. Genfer takes 0.006 to 0.015 s for one, two and four
  seasons; Alea.jl takes 141 s for one season
  and 323 s for two, and runs out of time at four.
  `--bounds --precision 128` bounds the rounding error of its results with
  interval arithmetic: on one season its interval for E is 5e-33 wide.

### noDice

The artifact's `nDice.exe` (OCaml, with a Rust rsdd library). Artifact:
Zenodo 10.5281/zenodo.18739789 (`nodice artefact.zip`, md5
fa025c6957b72ce632519245c0d56113), unpacked to `src/`. Build with opam
2.1.6 (the release binary of github.com/ocaml/opam), `OPAMNODEPEXTS=1`,
an opam repository that is the snapshot of 2025-02-28 (ocaml/opam-repository
at 4b26c3abed41d15919cbdf2774982771682c5c0a, `opam init --bare
--disable-sandboxing` on its directory), OCaml 4.14.2 (`opam switch create
4.14.2`), then in `src/`:

    opam install . --deps-only
    dune build ./bin/nDice.exe ./bin/dice.exe

The system libraries libffi and libgmp (headers) and a Rust toolchain are
needed; 3.2 GB with the build. `_build/default/bin/nDice.exe
resources/example.dice` prints `true 0.348837`. It reads the Dice syntax and prints a table of values and
probabilities to 6 digits, which `bench.py` reads as `run_dice` reads
Dice's output (`--nodice CMD`, `{file}`); the programs are those of the
Dice-program instances, the answers keyed as the instance's queries.
Dry run on the standard Dice programs and noDice's own: the answers agree
with ProbLog's `ddnnf` to 7.5e-7 (the print precision) on every
instance both finish: twocoins, evidence1, evidence2, grass, alarm,
noisyOr, murderMystery, survey and coupon2-10-obs (4.2 s; ProbLog times
out at 120 s); nodice_alarm runs out of time at 120 s, as Psi does.

### Psi

Section 10's Psi (ef8a9e5; `scripts/samplers/install_psi.sh`) on Dice
programs, through `scripts/dice2psi.py` (Boolean as a 0/1 number,
integers modulo 2^w, `categorical`, `uniformInt`, `observe`, `for` for
iterate; no recursion, `/`, `%`, lists). The probabilities of a
`categorical` are written as rationals that sum to 1 exactly: with
floating point entries that do not (k/36 for k = 1 to 8), Psi prints
`Pr[error] = 1`. A tuple is built only where it is passed to a function,
a loop or out of a branch. Psi answers one expression, so
the program runs once per component of its value and the times add up.
Dry run against ProbLog: agrees on twocoins, evidence1, evidence2, grass,
alarm, noisyOr (1.3 s) and survey (2 s); errors on murderMystery (a
function returning a tuple) and coupon2-10-obs; times out at 120 s on
nodice_alarm. This Psi build dumps core on murderMystery and on diamond
(a loop of a hundred calls of a function), which translate correctly.
On the Geni
programs it times out on the connectivity grids and segfaults on the
weather factory (tuple-valued function arguments); Psi also segfaults on
`&&` or `||` with a constant operand, which the translator removes.
Psi's own symbolic simplification is the cost: a hundred Boolean
variables in a DAG are out of its reach.

Its own program, R2's linear regression (`bb_linreg_wW`): a program of
`scripts/gen_psi_linreg.py` (`cobserve(gauss(a * x + b, 1), y)` for each of
the ten observations, the data of `data/alea/psi/linear_regression.jl`),
run once for the slope and once for the intercept with `psi
--expectation`; the answers are exact rationals, keyed `E(a)` and `E(b)`:
-0.7789866 and 2.6277944 in 0.21 s, against the conjugate posterior's
(-0.77899, 2.62779) of `data/alea/psi/README.md`. They are not checked
against the circuit's bit queries (the circuit is the discretized model,
whose means differ by its discretization), so `table` shows no
difference for them.

### SPPL

`pip install sppl==2.0.4` (PyPI, Apache-2.0; the repository's
e537b0c, from which `data/alea/sppl` takes the model, is six commits past
the tag v2.0.4), in a Python 3.10 venv: the package's dependencies
(numpy 1.22, scipy 1.8, sympy 1.6) do not build on Python 3.12, so on a
box with that Python the venv is made with `uv venv --python 3.10`. The
`setup.py`'s own pin of sympy 1.10.1 lacks the `limit` that sppl imports from
`sympy.calculus.util`, so the versions pip resolves are used. `--sppl
'SPPL/venv/bin/python -I {file}'` runs `scripts/sppl_markov.py N`, the
model of `examples/hierarchcial-markov-switching.ipynb` for N steps run as
the notebook does (compile, `constrain` on the observations, `prob`), with
the observations of `data/alea/sppl/markov_switching.jl` (the same
generator, with scipy's quantiles). It answers the continuous model
(normal emissions as densities), the circuit the model whose emissions
are rounded to a grid, so the answers differ by that discretization: at 10 steps by 6.7e-3 at most
from Alea.jl's. Times: 0.19 s at 5 steps, 0.73 s at 20 and 4.6 s at 100
(Alea.jl on the circuit: 87 s at 10).

### Pluck

The paper's artifact, Zenodo 10.5281/zenodo.15051140 (`pluck-artifact.zip`,
md5 b0932f3c7fd62ebfba4c7eb772cbccac, CC-BY-4.0), unpacked to `src/`; its
Julia package `PluckArtifact.jl/Pluck.jl` and the Rust BDD library rsdd it
calls (`src/RSDD/rsdd`, built with `cargo build --release --features=ffi`
after the line `rand = "0.8.5"` of its `Cargo.toml` gets the feature
`small_rng`). It runs on the Julia 1.11.9 of section 5 with its own depot:
`JULIA_DEPOT_PATH=DEPOT julia --project=PluckArtifact.jl/Pluck.jl -e 'using
Pkg; Pkg.instantiate()'`. The artifact's `programs/simple_example.pluck`
prints 0.2935138 for `num-greater-than-five` (true). `scripts/pluck_bench.jl` runs the
artifact's own program of a model (`src/table1/sequence_models/pluck/MODEL.jl`)
through `bdd_forward`, as its harness does.

Dry run against Alea.jl on the circuits of the pluck family (ProbLog does not finish
any of them in 300 s): hmm agrees to 5.7e-27, pcfg and string_editing to
3.0e-8 (Pluck's print precision), in 4.8 to 5.5 s of which 3.2 to 3.6 s are
Julia loading Pluck (0.01 to 0.03 s on a second run); Alea.jl takes 10.5,
42 and 128 s. On sorted_list Pluck stops with `type Closure has no field
constructor` in its own code (`likelihood/bdd.jl`), which `pluck_bench.jl`
reaches through the artifact's program as its harness includes it; the
cause is not known (the driver may differ from the artifact's own for
this model), and Alea.jl takes 78 s on it.

---

## 20. pyAgrum, pgmpy, libDAI and TensorInference.jl

Four exact engines for Bayesian and Markov networks beside Ace and Merlin,
run on the standard, large, Ace and UAI MAR suites and on the UAI PR sets.
All four read the UAI file and its `.evid` that `gen_bn.py --uai` writes
(`bench.py gen` writes them beside the program), so every system sees the
same model and evidence, and write Merlin's output format (`PR`, ln Z, `MAR`
with the marginals of the program's query atoms, rule 19) plus a trailing
`inference_seconds` line. `bench.py` reuses the Merlin parsers.

### Install

The installation (in a prefix,
`$HOME/local/code/bn-baselines` in the release run): a venv with `pyagrum`,
`pgmpy` and numpy; libDAI 0.3.2 from its release tarball, built with `make
lib` and linked into `$PREFIX/bin/libdai_uai` from
`scripts/libdai_uai.cpp` (needs libgmp); TensorInference.jl on the Julia
that `install_alea.sh` installs, in a depot of its own. The versions are
printed by the installer and by `bn_baselines_sample.sh`.

### How `bench.py` runs them

`bench.py run --agrum PY --pgmpy PY --libdai BIN --tensorinference
'JULIA --startup-file=no --threads=1 --project=DIR'` runs, per network
instance, one command per system through `timed()` (pinned to the lane's
core, single-threaded: numpy, BLAS and Julia threads are set to one):

- `agrum` (`scripts/bn_uai_run.py`): the evidence is sliced out of the
  tables first. Bayesian networks run `LazyPropagation`, Markov networks
  `ShaferShenoyMRFInference` (`evidenceProbability` is Z); aGrUM allows one
  factor per variable set, so factors of the same scope are multiplied.
- `pgmpy` (same driver): `VariableElimination`; on a Markov network
  (`DiscreteMarkovNetwork`) a query returns unnormalized values whose sum
  is Z. The marginals are asked one variable at a time.
- `libdai` (`scripts/libdai_uai.cpp`): `JTree` with HUGIN updates and a
  min-fill order, `logZ()` for the PR task.
- `tensorinference` (`scripts/tensorinference_uai.jl`): `TensorNetworkModel`
  with the optimizer `TreeSA` (variant `treesa`, the package's default) or
  `GreedyMethod` (variant `greedy`), `marginals` and `log_probability`.

The tables carry one column per system (variant). The PR task
(`--suite uai --pr`) gives each system's `log10_z` from the same drivers in
PR mode. The time is the process's wall time; `inference_seconds` goes to
the record's stats to separate start-up (Python, Julia's package load)
from the work. A model whose factors overflow memory is an error or a
timeout, counted as a failure.

### Checks

`tests/test_bn_baselines.py` compares each driver with brute-force
enumeration on small Bayesian and Markov files, with and without
evidence, for PR, MAR and a variable subset (env `BN_PYTHON`,
`BN_LIBDAI`, `BN_TENSORINFERENCE`). `scripts/bn_baselines_sample.sh`
runs a sample of each suite in lanes against the reference and Merlin,
and `scripts/sample_check.py` and `sample_times.py` list differences over
1e-9, errors, and each system's geometric mean time against Merlin's.

### Dry run (a lab machine, 2026-10-19)

Each suite's sample ran in four pinned lanes with a 120 s limit. Versions:
pyAgrum 3.2.1, pgmpy 1.1.2, numpy 2.2.6, libDAI 0.3.2.

| system | answers against the reference | time against Merlin's (geometric mean over the instances both finished at 0.1 s or more, the PR runs) |
|---|---|---|
| libDAI | marginals and log10 Z agree (to Merlin's 6 digits) | 3.0x (slowest 4.8x), 1 timeout in 13 |
| pyAgrum | agree | 2.8x (slowest 26x, a 3.6 s run against 0.1 s); fails where Merlin does (Alchemy_11 overflows a double) |
| pgmpy | agree | 2.1x on log10 Z (143x slowest), 8 of 13 marginals runs time out |
| TensorInference.jl `treesa` | agree | 10x (a fixed 25 to 40 s of Julia compile and contraction-order search on every model, 3000x on the smallest); no failures but one timeout |
| TensorInference.jl `greedy` | agree where it finishes | 43x; runs out of memory on the object-detection networks |

Merlin finished the PR task 1.5 to 3x faster than the clique tree there.
The slowest systems (pgmpy and TensorInference.jl) fit a run with the
300 s limit but not a 600 s run's budget on the large networks; libDAI and
pyAgrum are within a small factor of Merlin.

## 21. ProvSQL

ProvSQL (Senellart et al., MIT; <https://github.com/PierreSenellart/provsql>)
is a PostgreSQL extension that tracks provenance and evaluates the
probability of a query's result tuples; `probability_evaluate` picks
its method by cost (independent tuples, a tree decomposition built in,
external knowledge compilers where installed; none is, so it uses what is
built in). It is the TPC-H suite's database baseline.

The installation (in a prefix,
`$HOME/local/code/provsql` in the release run) builds libuuid (util-linux 2.39.3, for
`uuid-ossp`), PostgreSQL 16.4 and static Boost 1.74 (serialization) from
source and ProvSQL v1.13.0 as an extension, and ends with a smoke test.

`scripts/tpch2provsql.py` translates a `tpch` program (facts, probabilistic
facts, rules) to SQL: a table per fact predicate (`c1`.. and `prob`), the
data loaded by `COPY`, `add_provenance` and `set_prob`, a view per rule
predicate (a `UNION` of `SELECT DISTINCT` per clause, in dependency order),
and per query `SELECT probability_evaluate(provenance()) FROM q`. `bench.py
run --provsql PREFIX` starts a private cluster per instance on a unix
socket (`fsync` off, one connection), loads the data untimed (a database
holds its data) and times the query through `psql`; the record's seconds
are the queries'. Tests: `tests/test_tpch2provsql.py` (the translation, and
with `PROVSQL_PREFIX` set the answers against `tpch_exact.py`).

Dry run: seven TPC-H instances (q4, q7v, q10, q12, q15v, q18, q21) agree
with `tpch_exact.py` to 5e-14; ProvSQL takes 0.02 to 2.6 s against 0.2 to
7.9 s for the exact reference.

## 22. Storm and PRISM

Probabilistic model checkers on the weather factory (Rubicon's
`weatherfactoryN.prism` models, GPL-3, fetched by the installer and not
shipped): the property is `P=? [F<=H "allStrike"]` for the program's
instance (`geni_weather_nN_hH`). Not part of the programs' systems: the
model is the program's Markov chain written in PRISM.

The installation (in a prefix,
`$HOME/local/code/modelcheckers` in the release run): stormpy 1.14.0 in a venv, PRISM 4.8.1
(sha256 checked; needs Java), and the models.

`bench.py run --suite geni --storm PY --prism DIR --weather-models DIR`:
Storm through `scripts/storm_weather.py` with one Sylvan thread, in
three variants (`sparse`: floating-point sparse engine, `exact`: rational
sparse model, `dd`: symbolic BDD engine); PRISM with `-sparse` or
`-hybrid` (`-exact` refuses bounded until) and `-javamaxmem`
(`--prism-heap`, 4g). The answer is `result X`; the time is the process's,
with Storm's `inference_seconds` in the stats. Tests:
`tests/test_modelcheckers.py` against `gen_geni.py weather N H --reference`.

Dry run: weather factory (seven instances): Storm
(all three engines) and PRISM `sparse` agree with the dynamic program to
1.5e-15 where they finish; Storm `sparse` is the fastest (0.4 to 8 s, a
timeout at 17 factories and at 500 days), `exact` and `dd` take 7 to
104 s, PRISM `sparse` 11 to 13 s (JVM start-up), PRISM `hybrid` times out
everywhere at 120 s, and Storm `dd` runs out of BDD nodes at 17 factories.
