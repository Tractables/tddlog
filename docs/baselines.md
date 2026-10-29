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
| Dice | yes | Dice programs and networks; other programs through `ground2net.py` | `--dice` |
| Alea.jl (Julia 1.11.9) | yes | Dice programs and networks, through `dice2alea.py` | `--alea` |
| aspmc 1.1.1 | yes | programs, the evidence asked as queries | `--aspmc` |
| Ace 3.0 | yes | networks; other programs through `ground2net.py` (section 23) | `--ace`, `--ace-variants`, `--ace-heap`, `--ace-java` |
| Merlin 1.7.0 | yes (`bte`, `cte`) | UAI files; other programs through `ground2net.py` | `--merlin` |
| pyAgrum (aGrUM) | yes (junction tree, Shafer-Shenoy on Markov networks) | UAI files | `--agrum` |
| pgmpy | yes (variable elimination) | UAI files | `--pgmpy` |
| libDAI 0.3.2 | yes (junction tree, min-fill) | UAI files | `--libdai` |
| TensorInference.jl | yes (tensor network contraction, `treesa`, `greedy`) | UAI files | `--tensorinference` |
| ProvSQL 1.13.0 (PostgreSQL 16.4) | yes (its own method choice, tree decomposition) | the TPC-H programs and the literature suite's pdb, noisy-or, liftsep and rgraph, as SQL | `--provsql` |
| PSouffle (3fb0fd06) | yes (derivation graph, BDDs) | the programs of the logic program and database suites, through `problog2souffle.py` (section 26) | `--psouffle` |
| vProbLog | yes, at the fixpoint | not run, section 27 | |
| Storm 1.14 (stormpy) | yes (`sparse`, `exact`, `dd`) | the weather factory as Rubicon's PRISM model; every other Dice program as a Markov chain, through `dice2prism.py` | `--storm`, `--storm-variants`, `--weather-models` |
| PRISM 4.8.1 | yes (`sparse`, `hybrid`) | the same as Storm | `--prism`, `--prism-variants`, `--prism-heap` |
| Ganak | yes, exact rationals | weighted CNFs | `--ganak` |
| sharpSAT-TD | yes, 53-bit MPFR floats | weighted CNFs | `--sharpsat-td`, `--sharpsat-td-decot` |
| Roulette (Racket 8.16) | yes | its own programs (`--suite roulette`); Dice programs and circuits through `dice2roulette.py` | `--roulette`, `--roulette-variants` |
| Geni (gennifer) | yes, f32 | its own `.gir` programs (`--suite geni`) and the other Dice programs, through `dice2geni.py` | `--geni`, `--geni-benchmarks` |
| noDice | yes | Dice programs without `nflip`, the circuits' included | `--nodice` |
| Psi (ef8a9e5) | yes, symbolic | Dice programs, the circuits' included, through `dice2psi.py`, and its linear regression | `--psi` |
| Pluck | yes | its four sequence models (`--suite ppl`, family `pluck`) and the other Dice programs, through `dice2pluck.py` | `--pluck` |
| Genfer | yes | its population model, the Dice programs it ships, and the other Dice programs, through `dice2genfer.py` | `--genfer`, `--genfer-benchmarks` |
| SPPL 2.0.4 | yes, symbolic | its Markov switching model and the Dice programs, through `dice2sppl.py` | `--sppl` |
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

tddlog's time includes the conversion of a program from its original
language into the file tddlog reads: the ProbLog program of a Dice or
noDice program (and `dice2problog_onehot.py`'s, which tddlog's one-hot
arm reads), of a network and of a cplint LPAD, which `tddlog --program`
writes (`dice2problog.py`, `gen_bn.py` and `lpad2problog.py` wrote the
same text before tddlog's readers replaced them). The circuit Alea.jl
writes of a bit-blasted program is not charged: it is the benchmark's
input, and tddlog and every system that reads it or a translation of it
start from the same circuit (the owner, 2026-10-28), as Alea.jl starts
from its own program. Each conversion is timed once an
instance on one machine, apart from the runs (matched to the file the
runs read by its SHA-256), as the median of three runs. A conversion is
timed as the system it comes from is: a Python converter's process, or
`tddlog --program`'s. The processing
(`rel/charge.py`) adds its
seconds to the record of tddlog and of every system whose input is made
from the converted file, directly or through a translator of its own: for
example the ProbLog family, PITA, aspmc, Scallop and PSouffle; Ace, Merlin
and the network tools on the
ground networks of a converted program; and the Dice family on the Dice
programs written from it. A run that passes
the limit with it counts as a timeout. A system that reads the
benchmark's original program pays nothing (Dice and the systems that
translate a Dice program themselves, Alea.jl on its own programs, Ace,
Merlin and the network tools on a network's own file), and neither does
one whose input another converter writes for one baseline (`gen_bn.py`'s
Dice programs, Hugin and UAI files). A translator written only to hand a
program to one baseline (`problog2pita.py`, `problog2souffle.py`,
`dice2geni.py`, `dice2genfer.py`, `dice2psi.py`, `dice2alea.py`,
`dice2roulette.py`, the Hugin and UAI writers) is not timed: its start-up
on every call is the harness's, not the system's work.

Where tddlog reads the original itself, its time includes the reading
and it pays nothing (a record that names the file it read, `network` or
`original`: `bench.py`'s `network_args` and `original`, and
`bench_one.py` on the scaled loops), and the systems that read the
program `tddlog --program` writes of it pay that command's time. tddlog
has read the networks so since the rerun of 2026-10-27, whose conversion
times are `tddlog --program`'s, and the harness gives it the Dice and
noDice programs and cplint's LPADs so from the same day. The records
made before keep their charge: every record on a Dice, noDice or cplint
program to date, where tddlog read the script's program and paid its
time (`dice2problog.py`'s or `lpad2problog.py`'s), as the systems that
read ProbLog did, and on the networks every record before that rerun,
where tddlog paid `gen_bn.py`'s.

Every command a run times has the same memory, tddlog's and the
baselines' alike. `bench.py run --mem-cap SIZE` (the lane's share of the
box's memory) runs each command in a cgroup of its own
(`scripts/memcap.py`) whose `memory.max` is SIZE, without swap, the whole
process tree killed at the cap; the run starts under `memcap.py delegate
--`, a user scope with `Delegate=yes`, which the lab boxes' user managers
allow. A command the cap killed is a memout, and so is one whose error
says that an allocation failed; any other error is an error
(`bench.py`'s `memout`, which stops on a text that names memory and is
neither, so that a new text is read and classed). Each record keeps
`maxrss_kb`, the largest resident set of one of its processes, and under
the cap `mem_peak_kb`, the tree's peak as the cgroup counts it (its
`memory.peak` on Linux 6.0 and later, else `memory.current` read every
0.2 s), and `mem_cap_kb`. The systems' own memory options follow the
cap: Ace's heap is 0.8 of it (the JVM's own memory and c2d's beside it),
PRISM's heap and its decision diagrams' memory a third each, Julia's
`--heap-size-hint` 0.8 (Alea.jl, Pluck, TensorInference.jl; Julia's
collector otherwise aims at a share of the machine's memory), and
sharpSAT-TD's cache half of it less 500 MB. ProvSQL's server starts
outside the commands, and only its client is capped. tddlog bounds its
own allocations by the address-space limit; under the cap alone the cap
stops it instead, a memout either way.

The runs before 2026-10-28 capped each process's address space instead,
the lane's share under `ulimit -v`. That limit counts reservations, not
memory: a JVM's heap, Julia's and OCaml's reserved pools and the new
block of a reallocation count against it before any of it is used, so it
refused allocations below the memory the processes held, and Ace's heap
was half of it. Their records keep no memory figure.

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
- `-k fsdd` stops with `InconsistentEvidenceError` on programs whose
  evidence is consistent: bn_sachs_ev and mastermind4_4_8, which `ddnnf`
  and `sdd` answer, and 19 UAI networks that tddlog and the reference
  answer. The error comes from a logical check, the evidence's SDD found
  false (`problog/forward.py`), not from a probability; the check also
  runs while the forward compile is still building that SDD
  (`ForwardEvaluator.node_updated`). The results count these runs as
  errors.

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

`tests/it/problog_suite.rs` and `tests/problog_suite.py` (default `--dir`
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

The pack server's cplint 4.5.0 comes with bddem 4.3.1 (`pita.pl` md5
4b2d4a11a51f63b3b1dcddbd86b9e22b), and that pair works. The GitHub tags
do not replace it where the pack server cannot be reached (it answered 404
on 2026-10-21): cplint v4.5.0's `pita.pl` there calls `init_test/1`, which
no bddem tag defines, so every query fails with `Unknown procedure:
pita:init_test/1`, with bddem rebuilt from its source or not. Those
machines take a copy of a working pack directory instead. Check an install
with one query (`a:0.5.` answers `prob(a, P)` with 0.5). The runs whose
PITA failed in this way are not in the results; they were run again with
the working packs.

### How `bench.py` runs it

`scripts/problog2pita.py` writes `prog.lpad.pl` from the ProbLog program,
and one swipl process consults it and asks `prob/2` (or `prob/3` with
evidence) for every query:

```
swipl -q -g "attach_packs('$PREFIX/swipl-packs',[duplicate(replace)])" \
      -g "consult('prog.lpad.pl'), main" -t halt
```

Under a memory limit (`ulimit -v`) a `-g "set_prolog_flag(table_space,
LIMIT)"` before the consult gives PITA's tables the run's whole limit:
SWI's default table space of 1 GB stopped PITA with `Not enough
resources: private_table_space` on the larger grammars of the terms
families (tf1), before the memory limit that binds the other systems.
That error counts as running out of memory (`ran_out`, the skip rule).

The translation has to:

- table every predicate that can reach itself (without it the smokers
  programs loop);
- expand `query(Q) :- Body.` by copying the facts the body reads out of
  the LPAD block under a prefixed name, since PITA rewrites every
  predicate inside the block;
- turn `evidence/1,2` into the second argument of `prob/3`;
- rename the atoms PITA keeps for itself (the predicates its user module
  defines, its exports, and `db/1`) with a prefix, which the driver strips
  from the queries it prints. PITA adds two arguments to every predicate,
  so a program's `v/1` is PITA's own `v/3`: on Roulette's HMM programs,
  whose observations are `v/1`, PITA answered 0 at every length. It reads
  `db(G)` in a body as a call of `G` in plain Prolog: on Roulette's caesar
  programs, whose bits are `db/2`, it printed nothing. PITA ran again on
  these 36 programs (Roulette's HMM programs and its caesar programs
  below 300 letters) with the renaming, and those records replace the
  first ones. On the HMM programs it answers 100 steps (300 for the
  smoothed marginals, 600 for the marginal likelihood) and runs out of
  time at the next length; its answer on rl_hmm_filtering_marginal_100 is
  1.43, above one, where the smoothing programs of that length come out
  right. The answer is PITA's own, not a misreading of its output:
  asked apart (2026-10-22), it gives P(q, e) = 2.3e-17 and P(e) =
  2.8e-35, a joint above the probability of its own evidence, and
  `prob/3` the 1.43 the harness reads. The cplint pack it ran with
  reports on loading that bddem does not define `create_json_util/5`,
  which pita.pl exports. On rl_caesar_25 it
  prints `Undefined: probability of evidence 0.` for every query, the
  evidence's probability computed as 0, so the record has no answer and
  its error shows the consult warning, the last line on stderr; the
  longer caesar programs are not run after that failure;
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

PITA loses a probability far below one: bddem evaluates a complemented
else-edge of CUDD's diagrams as `1.0 - pf` (`Prob` in `bddem.c`), in
doubles, so a part of P(e) below about 1e-16 that it reaches through such
an edge vanishes. A cut-down written by hand shows it: n facts `0.5::a_i`,
evidence that they are all false or all true (P(e) = 2^-(n-1)), and the
query `a1` (1/2). At n = 45 PITA answers 1/2; at n = 60 it gives
P(e) = 2^-60, the all-false half lost, and P(a1|e) = 1, on
`problog2pita.py`'s translation and on the LPAD written by hand alike.
The Ace programs `ace_fs_07` and `ace_mastermind_03_08_03` have evidence
that unlikely. On `ace_fs_07` PITA's P(e) is right under the first 700 of
its 1120 observations (3.4e-13) and reads 1.8e-15 under all of them, where
every query answers 0 (ProbLog and tddlog 0.0237 on the worst); on
`ace_mastermind_03_08_03` (P(e) = 2.6e-13) it answers 0.49979 where the
others answer 0.5. On the arithmetic programs' `arith_caesar_small` (P(e) =
4.8e-15 as PITA computes it) PITA's probabilities of a query and the
evidence together come out as multiples of 2^-53, the spacing of doubles
below one, and its answers are off by up to 0.031. These wrong answers are
PITA's, not the translation's.

tddlog reads cplint's example LPADs itself, and `tddlog --program`
converts them to ProbLog (as `scripts/lpad2problog.py` did), with the
`?- prob(Q, P)` queries of their `/** <examples> */` block; 28 of the
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

For a `dice` instance it runs the source program; for a network one
program per queried variable, written by `gen_bn.py --format dice --query
VAR` with the instance's observations (a returned tuple of all variables
would print the joint).

### Programs

- `data/dice/`: the eight programs of the paper's comparison with Psi and
  WebPPL, read by tddlog and translated by `tddlog --program` (as by
  `scripts/dice2problog.py`; all agree with Dice),
  and the loop programs (caesar-iter, simpleCaesar, pmc1, pmc2,
  recursion from Dice's repository; diamond and ladder from dicebench's
  generators), which use integers, `iterate` and recursion. `bench.py
  gen --suite indicators --families dice` adds the loop programs.
- `gen_bn.py --format dice` writes a network as nested conditionals over
  `discrete`s, the encoding of Dice's `bngen`, with the program's
  observations: each observed variable followed by `observe (X ==
  int(w, k))`, and a Markov network's factor, which the ProbLog program
  writes as a variable of one value observed true, as a `flip` of the
  factor's entry over its largest under the conditionals on its scope,
  observed. The program defines only the variables it returns or
  observes and their ancestors; the others sum out. So every network of
  the standard, large, Ace and UAI suites, with its evidence, is a Dice
  program, and the systems that read Dice programs (Dice, Alea.jl,
  Roulette, noDice, Psi, Genfer, SPPL and Geni) are given them. For the
  PR task (`bench.py run --pr`) the program of P(e) observes nothing and
  returns the conjunction of the observations, the factors' flips among
  them (`gen_bn.dice_programs` with `pe`): its probability of true is
  P(e) of the network the ProbLog program has, and log10 Z is its
  logarithm plus the offset the ProbLog program states on its first line,
  as for tddlog. Where a system computes in floating point, an answer
  that underflows (P(e) is as low as 1e-618 on the UAI PR set) is checked
  against the reference like any other and fails.
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
    julia --project=Alea.jl alea_bench.jl export [--warm-below S] [--set NAME=VALUE ...] PROGRAM.jl PREFIX

`--set NAME=VALUE` runs the program with its one line `const NAME = ...`
reading `const NAME = VALUE`: a size, such as the bits after the binary
point of its fixed-point numbers (`--suite bitblast`, below).

`export` writes PREFIX.pl, a ProbLog program (a flip `P::fN.`, numbered
in creation order, an `&` one rule, an `|` two, `!` a negated literal,
each observation `evidence/2`, each answer a query defined by its
literals), which main's front end `scripts/alea2problog.jl` writes (a
user's Alea.jl program in the same form, without `--set`; alea_bench.jl
includes it and runs the program it loads), and PREFIX__dJ.dice, Dice programs over the same circuit with
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

The bitblast suite's programs, like the ppl suite's `arith_` and
`pluck_` ones, exist only as Alea.jl source, over its `DistUInt` and
`DistFix` numbers. Every system other than Alea.jl, tddlog included, is
given the circuit Alea.jl builds for a program: PREFIX.pl, or the
PREFIX__dJ.dice programs (a `let` per flip and per gate) that Dice runs
and the other systems' translators read. Those Dice programs are the
circuit in Dice's syntax, not a model written in Dice, and
`dice2problog.py` on them gives the same circuit back, so these
instances are translated for every system but Alea.jl (section 24),
apart from the three models that Genfer, SPPL and Psi run as their own
programs (section 12). A model written in another system's language
would need a translator from Alea.jl's source, which does not exist yet.

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
from stderr. A ground query it does not print is an atom its grounder
removed: one the program states as a fact answers 1 (`bench_pe :- true.`,
P(e) on the seven PR networks with no observations, where aspmc prints
only the program's overall weight), and any other answers 0, since no
rule derives it (water's values of probability 0). The processing reads
the records of runs before bench.py did so (bn_water_ev, 11 of 14 queries)
the same way: of aspmc alone, a query left out where the reference answers
exactly 0 counts as answered 0. Any other
system's record that leaves out a query the reference answers counts as
wrong (`missing`).

Its ProbLog frontend does not condition on `evidence/1,2`: it reads them
as facts and returns unconditional probabilities with exit code zero.
bench.py therefore gives it a program with evidence with the evidence
asked as queries (`aspmc_conditioned`): the observations and queries
leave the program, `bench_e :- e1, ..., en.` is asked (a negative
observation `\+ a` in the conjunction), and every `query(Q) :- B.` (B
optional) becomes `bench_qe_Q :- B, Q, e1, ..., en.`, asked for the same
solutions of B. The answer to Q is P(bench_qe_Q) / P(bench_e), which is
P(Q | e) exactly, as for Scallop (section 3); the evidence is
inconsistent where P(bench_e) is 0. aspmc compiles the program once with
the extra queries, so its time includes what conditioning costs it. It
computes in doubles, and on the UAI networks, where P(e) is as low as
1e-618, its answers are checked against the reference.
The release run's harness, before `aspmc_conditioned`, refused aspmc on
a program with evidence (an error, "aspmc ProbLog frontend does not
condition on evidence"), as did the reruns of its skips at its commit.
Such a refusal says nothing of aspmc, which takes the program: a later
run of the current harness replaces each (aspmc_std, aspmc_lit, bnx),
neither a table nor a page shows it (bench.py's `stale_refusal`), and
one that no later run replaces stops the processing.
Its grammar reads a probability as `1`, `1.0...` or `0.` and digits, not
in exponent notation, in which the network programs write a small one
(`1e-05`): on the UAI MAR set the parser refused 95 of the 120 networks'
programs, nearly all for it. bench.py writes such a probability out in positional
notation, the same double (`aspmc_numbers`). The grammar has no `is`
either, which the Dice loops' step rules use (`T1 is T + 1`): bench.py
writes it as clingo's `=` (`aspmc_is`). It still rejects some
programs with a parser error (bn_insurance); check the coverage on the
installation a run uses.

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
- **The heap**, `--ace-heap` (8g by default, 0.8 of `--mem-cap` under
  the cap), decides the route: Ace
  compiles tabularly when the entries of the best of six min-fill orders
  fit in the free heap, through c2d otherwise. With 8g every network of
  data/bn and data/bn-large but munin1 goes tabular. Compare routes, not
  only times, and give a box's runs one heap.
- **Randomness.** Min-fill and dtree ties are broken at random from the
  clock unless `-Xseed N` (variant `Xseed=N`).
- **The thread stack**, `--ace-stack` (the JVM's `-Xss`; by default the
  JVM's own). Ace recurses over the network, and on the deep ones the
  default stack overflowed (`StackOverflowError`): the Ace suite's
  blockmaps from 15_01 and fs_28, nine blockmaps of the UAI PR set in log
  space, the TPC-H programs' and Roulette's diamond, ladder and
  motivating chains through their ground programs, digitRecognition and
  bb_wmi_path_n1024. With `--ace-stack 512m` (2026-10-22) Ace ran those
  again in the variant that had overflowed, from the instance that had
  failed on: no run overflowed, and every answer agrees with tddlog's
  (the nine PR networks' log10 Z to 1e-15). What stops it now is the
  time limit, c2d (exit status non-zero, on TPC-H q1 and q18 at size 5
  and q15 at 100, and on Roulette's chains at 10000 and 30000), the heap
  (rl_ladder_300000) and ground2net.py's limit on a network's size (the
  chains of 1000000 steps); its c2d route prints `NaN` on rl_diamond_3000
  and rl_ladder_3000, as on the scaled diamond (log space below).

The record's `seconds` cover both JVM runs; `ace_method`, the circuit's
size and Ace's own timings are kept beside them. The answers are read
from `Pr(X,e) = [...]` through `gen_bn.py`'s names.

### Variants

`--ace-variants`: `auto` gives no option; any other variant names options
without the dash, joined by `+`, with `=` before an argument
(`forceC2d+dtHypergraph=3`). `AceCompile` accepts `-forceC2d |
-forceTabular`, `-d02 | -sbk05 | -cd05 | -cd06`, `-dtBnOrder |
-dtClauseMinfill | -dtHypergraph <count>`, `-noEclause`, `-Xlog` (log
space, UAI networks below), `-e <file>`; the readme's `-noTabular` and
`-dtBnMinfill` are `-forceC2d` and `-dtBnOrder`.
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
observed `ok`, so Ace compiles the network tddlog compiles. Ace runs in
the narrow lanes of the uai tables, with half a lane's memory as heap.
Where that heap ran out (ObjectDetection, two DBNs), normal space
underflowed (csp_13) or the run timed out, Ace ran again with `Xlog`
in every other lane, with the lane's memory less 4 GB (25 GB) as heap,
and each table takes a network's fastest answer over Ace's routes. With
that heap it answers csp_13 and dbn_12. On every ObjectDetection network
(48 MAR, 46 PR) it now times out at 300 s instead of running out of heap.

**The PR task.** Under `run --pr` Ace compiles and evaluates as for the
marginals, with the `.inst` files of a plain `gen` (`gen --pr` writes
none, and `--ace` on such a directory is refused). Its record holds log10
Z from the P(e) it prints, as tddlog's from its own, beside its marginals.
AceEvaluate prints P(e) in full on its standard output and to six digits
in the `.marginals` file; bench.py takes the former. On these networks
P(e) goes down to 1e-618 (Grids_18), below a double's range, where Ace in
normal space prints `Pr(e) = 0.0`: bench.py reads that as inconsistent
evidence, and the check against the reference (section 8, the PR task)
makes the record a failed run.

**Log space.** `AceCompile -Xlog`, an option of its code the readme leaves
out, writes the circuit's space into the `.lmap` file (`LOG_E`), and
AceEvaluate then computes in natural logarithms
(`CalculatorLogE`, "to prevent underflow") and writes `log_e(Pr(e))` and
`log_e(Pr(X,e))`, which bench.py reads (variant `Xlog`). Checked on seven
networks of the MAR set (2026-10-20): on the tabular route
the log-space answers are the normal-space ones, at about the same cost
(Promedus_31, Segmentation_11, Alchemy_11, Grids_14). Its c2d route
answers wrongly: `log_e(Pr(e)) = -∞` on Promedus_31 and Segmentation_11,
whose P(e) is 1.6e-2 and 2.0e-52, and a NullPointerException in the
evaluator on Alchemy_11 and Grids_14 to Grids_18. On the PR sets it
printed P(e) = 0 for three wcsp networks (myciel5g_3, myciel5g_4,
queen5_5_4) that have no reference; their Z is positive, since every
entry of their tables is 1 or 1e-4 (soft colouring constraints, as in
queen5_5_3, whose log10 Z is -113.03). An `Xlog` run whose compile reports
`Method : c2dCompilation` is therefore a failed run: bench.py does not
evaluate the circuit. The uai_pr table
therefore runs Ace as `Xlog` (its default route, in log space) and
`forceC2d` (in normal space). The uai table runs `auto` and `forceC2d`,
as every other network suite does, since log space changes the times
(Grids_14's evaluation took 19 s against 12 s); the uai_pr table's `Xlog`
records show the networks it would answer besides. Of the networks whose
P(e) underflows a double, Grids_17 and Grids_18, `-Xlog` helps only where
Ace's tabular route compiles them: with a 10 GB heap its default route chose
c2d in log space (and failed), and in normal space did not compile any of
Grids_16 to Grids_18 in 300 s.

The networks of the programs (the ground route, section 23) underflow
too: Ace's evaluator stops with `UnderflowException` on Roulette's
caesar, hardware and HMM programs, on the circuits clinicalTrial1_w14,
normal_mixture, markov, population and wmi_star from a size on, on
digitRecognition and on the scaled caesar-iter and simpleCaesar loops,
in both normal-space variants. `Xlog` ran on those from the instance
that had failed (2026-10-22): it answers 29 of Roulette's 33, all four
loops, digitRecognition and 11 of the 16 circuits, each as tddlog does
(clinicalTrial1_w14, where tddlog ran out of its lane's memory, as
pyAgrum and pgmpy do). Where Ace's default route chose c2d
(normal_mixture at 9 bits, rl_hardware_10000), log space is refused or
c2d failed. c2d's `NaN` answers (forceC2d on bb_wmi_star, on
bb_clinicalTrial1, on the scaled diamond and on Roulette's diamond and
ladder at 3000) have no such option: c2d in log space answers wrongly,
and bench.py refuses that run.

**uai08_pe.** The package's `uai08_pe` script (`mark.reason.apps.BnUai08
-z NET EVID SEED`, Pr(e) for the UAI 2008 evaluation) reads a UAI file
and its evidence file (under the `.uai.evid` name; another extension is
refused, and a leading sample count is not read). It prints `P(e)`,
`log_e(P(e))` and `log_10(P(e))`, right on Promedus_31, a Bayesian
network. It reads a Markov network as a Bayesian one too: `P(e) =
Infinity` on Grids_14, and no answer in 300 s on Grids_16 to Grids_18.
bench.py does not use it.

### BIF networks

Ace 3.0's jar leaves out the classes of its BIF reader: on a `.bif` file
it stops with `NoClassDefFoundError: BayesianNetworks/DiscreteFunction`.
Roulette's nine networks are BIF files, and bench.py gives Ace each of
them as a Hugin file written the same way (`write_hugin`).

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

ProbLog, PITA and aspmc print no P(e). Under `--pr` bench.py writes
`NAME.pe.pl` beside each program (`pe_program`): its queries and
observations dropped, and `bench_pe :- e1, ..., en.` with
`query(bench_pe).` added, the conjunction of the program's observations.
P(bench_pe) is P(e), and `log10_z` is its logarithm plus the program's
constant. aspmc is given this program as it stands, since it has no
evidence. Ace prints P(e) beside its marginals; under `--pr` its record
holds both (section 7).

On the uai suite, MAR and PR alike, `bench.py run` checks every baseline's
record against the instance's reference once its systems have run
(`check_against_reference`), and the processing checks every suite's
records with the same function (`answer_right`): an answer is right
when it is within 1e-6 of the reference and within 1e-3 of it
relatively, each beyond the last digit either of the two prints (Merlin
six decimals, noDice six with its last unrounded, Geni seven, ProbLog and
PSouffle eight significant digits, the network tools' drivers twelve),
or when it is within 1e-9 of it: differences under 1e-9 count as
floating-point noise. The reference is
the exact record, else Alea.jl's, ProbLog's, Merlin's or Ace's (whose
answers of 0 or NaN, its underflow in doubles, are never a reference),
and tddlog's only where none of them answers (the page then says checked
against tddlog only); the reference itself is checked against none. log10 Z is
checked against the clique tree's (else Merlin's PR task, Ganak's or
tddlog's) to 1e-6 times max(1, |log10 Z|). A record off by more is an
error with the difference in its message, a failed run as a
crash is: a system that computes in doubles can underflow on these
networks, whose P(e) goes down to 1e-618 (Grids_18 of the MAR set).

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
  default 1), with a cache of C 10^6 bytes (half the memory cap, or
  else of the address-space limit, less 500), and reads `c s log10-estimate`.

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
An Alea.jl program (`--suite ppl` and `--suite bitblast`) reaches Roulette
as the Dice programs of its circuit, one at a time, as Dice runs them: a
`bool` program answers its atom, and a `joint` program's N Booleans are
asked as one int (`dice2roulette.py --joint`, the first Boolean the most
significant bit), the value U answering PRED(U). The seconds of the
programs add up. A program that reaches no network or Dice program of its
own goes through its ground program (section 23).
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
| Geni (gennifer), ICFP 2025 | yes (generating functions, f32) | github.com/geni-icfp25-ae/geni-icfp25-ae, MIT | run on `--suite geni` and, translated, on every Dice program: `--geni` |
| Genfer, NeurIPS 2023 | yes (generating functions; rational arithmetic where the program allows) | github.com/fzaiser/genfer, MIT | run on its own programs and, translated, on every other Dice program: `--genfer` |
| noDice, OOPSLA 2026 | yes | Zenodo 10.5281/zenodo.18739789, code Apache-2.0 | run: `--nodice` |
| Psi | yes (symbolic, imperative) | eth-sri/psi, Boost | run on Dice programs and on its own linear regression: `--psi` |
| SPPL, PLDI 2021 | yes (symbolic, sum-product) | PyPI `sppl` 2.0.4, Apache-2.0 | run on its own Markov switching model and, translated, on every Dice program: `--sppl` |
| Pluck, PLDI 2025 | yes | Zenodo 10.5281/zenodo.15051140, Julia and Rust | run on its sequence models and, translated, on every Dice program: `--pluck` |
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

### Dice programs in the other systems' languages

Every system runs on every suite whose programs it can be given. The
instances that have a Dice program behind them (`dice_source`: the dice
paper's programs, the networks as Dice programs, the loops, noDice's
programs, Geni's and Roulette's generators') and the circuits of the
Alea.jl programs (the ppl suite's arith, hybit and pluck families and the
bitblast suite, whose Dice programs `alea_bench.jl export` writes and Dice
runs, NAME.dicemap) reach Genfer, SPPL and Geni through a translator each: `scripts/dice2genfer.py` (SGCL),
`scripts/dice2sppl.py` (SPPL) and `scripts/dice2geni.py` (Geni's IR), over
one lowering, `scripts/dice_ir.py`. Psi reaches the same programs through
`scripts/dice2psi.py` (its own translator, one run a component, `--joint`
for a circuit's joint lines), Pluck through `scripts/dice2pluck.py` (over
the same lowering, one Julia run for all the components, by
`scripts/pluck_dice.jl`), Storm and PRISM through `scripts/dice2prism.py`
(a Markov chain a component, section 22), and noDice reads them as they are, as Dice does. It inlines the functions (recursion is
refused), resolves tuples, folds constants, and gives each variable the
values it can take; `iterate` stays a loop. A system's own program is run
where its paper has one (Genfer's seven Dice programs, Geni's suite);
everything else goes through the translation. As for Psi, a program is
asked one component of its value at a time (Genfer, Geni) or compiled
once and asked each value of each component (SPPL): the table's single
marginals, no joint, as the other systems are asked. Where a system is
asked one component a run (Genfer, Geni, Psi), the record's seconds are
its runs', and the translations, one translator process a component, are
bounded by a time limit of their own, the suite's (bench.py's
by_component): a program of 30 000 queries (smokers s30000) translates
for hours. Past the limit the record is the translator's error, "the
translation of N components passed the time limit", and the instance
counts as unanswered by that system. A circuit's program is asked
what Dice is asked of it: a `bool` program its probability of true, a
`joint` one, whose value is N Booleans, the joint of the N
(`--joint`: the int(N) whose bits they are, the first the most
significant; SPPL, which has no sum of two variables, is asked the
probability of each assignment of the N instead).

- Genfer: a Boolean is 0 or 1, an int(w) a number below 2^w; + and a
  product with a constant are affine assignments reduced modulo 2^w, and
  -, a product of two variables and >> are switches on one operand's
  values. Genfer's time grows exponentially with the number of variables,
  so values live in registers, reused once dead (set to 0, which
  marginalizes them). Run with `--limit 2^w`; floating point (not
  `--rational`).
  Genfer runs on a thread with a fixed 32 MB stack, which long programs
  (caesar-iter, Roulette's ladder) overflow.
- SPPL: a variable is defined once, by a sample or a transform of one
  variable; an operation of two variables is a switch on the values of
  one, an observation inside a branch a variable conditioned on after the
  outermost `if` (a `condition` inside a branch would renormalize it).
  `scripts/sppl_dice.py` compiles the program and asks `prob` for each
  value.
  SPPL's compiler writes the program as Python, nesting a call for each
  `elif`, and Python refuses more than 200 nested parentheses: a switch
  on more values does not compile (bb_laplace_w13's joint, written as a
  sum, did not, which is why a joint is asked by assignments).
- Geni: its interpreter answers Booleans and tuples of them, so an int(w)
  is w bits, as in Dice and Geni's own surface language: `discrete` a
  chain of Bernoullis, + and - ripple-carry adders, * shifted partial
  products, < the carry of a - b. Geni does not normalize by the
  observations, so the masses it prints are divided by their sum.
  It prints seven decimals: where the observations are less probable
  than that, every mass prints as 0, which bench.py records as an
  error with that reason.
- Pluck: a lazy functional language over algebraic data, whose natural
  numbers are unary, so an int(w) is w Booleans, as for Geni (the
  artifact's own Bayesian networks write a value as a constructor of its
  own type, or as Pluck's IntDist, which has no arithmetic). `discrete`
  draws the bits one after the other, a flip given the bits drawn before
  it, as Pluck's own `discrete` expands into a tree of flips. The program
  is one function, `dice_model`, which returns the conjunction of the
  observations and the components; a query `cK` per component
  (`Posterior` given the observations, `Marginal` without), all in one
  file and one run. An operation read more than once is a `let` binding,
  which Pluck evaluates once; `iterate` is a recursion over Pluck's
  natural numbers through its Y combinator, as the artifact's diamond and
  ladder networks iterate, its state the loop's bits in nested `Pair`s
  (Pluck's `let` copies its environment at each binding, so an unrolled
  loop would be quadratic). Pluck's evaluator recurses as deep as the
  program nests, so `pluck_dice.jl` runs it in a task with a 1 GB stack.

A program the target language cannot express is refused by its
translator with exit code 3 and the reason, which `bench.py` records as
`unsupported` (`n/a` in a table, unanswered in a cactus plot): lists, `/`,
`%`, `::` and recursion, for all of them.

Left out, with the reason:

- Pluck's artifact has its own versions of the dice paper's small
  networks (two coins, evidence1, evidence2, grass, murder mystery,
  noisy-or, burglary), diamond and ladder (`src/table1/bayesian_networks`,
  `network_verification`); Pluck runs the suites' Dice programs of these
  through `dice2pluck.py` instead, as the other systems are given them.

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
difference to the reference and applies no tolerance; read 1e-6 as agreement for Geni. One
exception is open: on Roulette's Munin network (rl_bn_munin, the marginal of
L_SUR_CV_CA) Geni's answers differ from the reference by up to 3e-4, while
Genfer, SPPL and tddlog agree with it and Geni agrees with it on the other
networks it answers. Geni rebuilt with `Precision = f64` prints the same answers,
and every `discrete` of the Dice program sums to one, so neither rounding
nor a table short of one explains the difference; whether Geni or the
program `dice2geni.py` writes is at fault is not known. On a network whose
evidence has a small probability the seven decimals are too few: on
uai_promedus_24, P(e) = 1.37e-6, Geni prints `Pr(1) = 0.0000013`, and its
log10 Z differs from the reference by 0.024 and its marginals (each over
P(e)) by 0.034, while Dice, Alea.jl, Roulette, Genfer, SPPL, Pluck, Storm
and PRISM agree with it to 1e-12 on the same programs. Such answers are
wrong answers of Geni's own and count as failures. A Boolean's two answers are `Pr(1)` (true) and
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
On the two smallest programs of the Geni weather factory
(`geni_weather_h1`, `geni_weather_n2`) noDice exits in 0.03 s without
printing anything, and the cause is not known. The larger weather
programs run out of time or memory. Both count as unanswered.
Its six decimals are too few for a network whose evidence has a small
probability: on uai_promedus_24 under `--pr`, P(e) = 1.37e-6, noDice
prints 0.000001 for the program Dice answers 1.37e-6 on, and its log10 Z
differs from the reference by 0.138. Such answers are wrong answers of
noDice's own and count as failures.

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
timeout, counted as a failure. A marginals run reads the query
variables' indices from a file (`@FILE`). On the largest networks of
the final PR set the list is longer than Linux allows for one argument
(128 KB, `E2BIG`).

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
built in). It is the TPC-H suite's database baseline, and runs on every
other family whose programs are queries over tuple-independent tables:
the literature suite's `pdb`, `noisyor`, `liftsep` and `rgraph`.

The installation (in a prefix,
`$HOME/local/code/provsql` in the release run) builds libuuid (util-linux 2.39.3, for
`uuid-ossp`), PostgreSQL 16.4 and static Boost 1.74 (serialization) from
source and ProvSQL v1.13.0 as an extension, and ends with a smoke test.

`scripts/tpch2provsql.py` translates a program of `gen_tpch.py`,
`gen_pdb.py`, `gen_noisyor.py`, `gen_liftsep.py` or `gen_rgraph.py`
(facts, probabilistic facts, non-recursive conjunctive rules with
comparisons, queries and evidence on ground atoms) to SQL: a table per
fact predicate (`c1`.. and `prob`), the
data loaded by `COPY`, `add_provenance` and `set_prob`, a view per rule
predicate (a `UNION` of `SELECT DISTINCT` per clause, in dependency order),
and per query `SELECT probability_evaluate(provenance()) FROM q`. A ground
atom `p(a, b)` is a Boolean view of the rows of `p` with those columns.
Evidence is the view `pq_ev`, the product of its true atoms' views
`EXCEPT` the union of its false atoms' (ProvSQL's monus, a negation in
the Boolean semiring), and a query under evidence is its atom's view times
the evidence's: its answer is P(q and e) / P(e), two evaluations within
the instance's time limit. `bench.py run --provsql PREFIX` starts a
private cluster per instance on a unix socket (`fsync` off, one
connection), then times the load (`psql -f load.sql`: the tables and their
provenance, which is parsing the input as tddlog's reading of its fact
files is; the owner, 2026-10-27) and the queries through `psql`. The
record's seconds are the load's and the queries', within the time limit
together (a load past it is a timeout); the load's are the record's
`load_seconds`. The start of the server is not timed. The postmaster
runs in a session of its own (and under `--mem-cap` in a cgroup of its
own) with `TMPDIR` in the instance's directory, and is killed with
everything it started after each instance: ProvSQL's backend starts a
compiler (d4) that a timeout does not stop, and seven of them filled a lab
box's root filesystem in the first run (the record is an error if any
process or /tmp/provsql* directory is left, and the run stops with under
2 GB free on `/`). Tests:
`tests/test_tpch2provsql.py` (the translation, and with `PROVSQL_PREFIX`
set the answers against `tpch_exact.py` and `lit_exact.py`).

The other families of the program suites are not given to ProvSQL: its
queries are non-recursive SQL over tuple-independent tables, and the
translation has no counterpart for a recursive rule (`graph`, `grid`,
`sickness`, `smokers`, `biomine`, `pedigree`), a probabilistic rule or an
annotated disjunction, which would need a fresh tuple per grounding
(`uwcse`, `cora`, `cplint`, `bn`, `mnist_sum`, `mastermind`), or a Dice or
Alea.jl program (the standard suite's `dice`, the ppl, dice, roulette,
bitblast and geni suites).

Dry run: seven TPC-H instances (q4, q7v, q10, q12, q15v, q18, q21) agree
with `tpch_exact.py` to 5e-14; ProvSQL takes 0.02 to 2.6 s against 0.2 to
7.9 s for the exact reference. In the run of the literature suite's
`pdb` and `noisyor` programs, all 11 instances agree with `lit_exact.py`
to 1.2e-15.

## 22. Storm and PRISM

Probabilistic model checkers, given every instance that has Dice
programs, on two routes.

The weather factory: Rubicon's `weatherfactoryN.prism` models (GPL-3,
fetched by the installer and not shipped), with the property
`P=? [F<=H "allStrike"]` for the instance (`geni_weather_nN_hH`): the
program's Markov chain as Rubicon's authors wrote it in PRISM.

Every other instance with Dice programs, the ones Genfer, SPPL and Geni
are given (section 12: `dice_source` and the circuits' dicemaps; the
logic programs through their ground programs' Dice programs, section 23):
`scripts/dice2prism.py` writes a discrete-time Markov chain in the PRISM
language, which both read, one model a component of the program's value
(`--joint` for a circuit's joint lines), over dice_ir.py's lowering
(functions inlined, tuples resolved, constants folded). A discrete program
without unbounded loops is such a chain:

- One module, a program counter `pc`, and the program's values as its
  variables: a Boolean a `bool`, an int(w) a number in [0..2^w-1] whose
  +, - and * are taken modulo 2^w as Dice's (`mod`, unless no values in
  the operands' ranges wrap: PRISM checks every update in every state
  its guard admits, reachable or not), << a product and >> the `floor`
  of a quotient.
- A `flip`, `discrete` or `uniform` is a command's probabilistic choice,
  and the pure operations after it are computed in the same command, the
  outcome substituted and folded in each update, up to the next choice:
  the chain has a step only where something random happens (and where an
  expression would grow beyond 64 operations, whose operands are then
  stored). A condition that the outcome decides sends each outcome to its
  side; before the choice, a condition on the state splits the command, a
  guard per case (a network's conditional table is a command per row; at
  most 256 commands a step), and after it such a condition starts the next
  step, so that the rows of two tables do not multiply.
- `observe e` sends the states where e is false to an absorbing fail
  state; the end is an absorbing state labelled "done" that keeps only
  the value. The answer to value v is P(F "done" & r=v) / P(F "done"): a
  property per value the component takes (for a Boolean only true),
  named `vN`, and `obs` for P(F "done") where something is observed (PRISM
  refuses a property named as a label), all checked on one build of the
  model. `iterate(f, x, k)` is the inlined body as a loop with a counter:
  the states of the unrolled program in k times fewer commands (Storm's
  explicit builder evaluates every command's guard in each state).
- The state count is what the checkers' time grows with, so a value is in
  the state only while it is live: a variable is reset to false or 0
  after its last read, the variables are registers shared by values whose
  live ranges do not meet, a sample is taken just before its first reader
  and an observation checked right after what it reads is computed.
- The probabilities are exact: a decimal as the shortest one that reads
  back as the program's double, a discrete's outcomes summing to one in
  rationals (the largest takes up the rounding), a uniform's as fractions.
- Refused (exit 3, `unsupported`): what every translator of section 12
  refuses (lists, `/`, `%`, `::`, recursion), and an int wider than 30
  bits or a product beyond 2^31, past PRISM's 32-bit integers.

The installation (in a prefix, `$HOME/local/code/modelcheckers` in the
release run): stormpy 1.14.0 in a venv, PRISM 4.8.1 (sha256 checked;
needs Java), and the weather models.

`bench.py run --storm PY --prism DIR [--weather-models DIR]`: Storm through
`scripts/storm_check.py`, the model built once for all its properties,
with one Sylvan thread, in three variants (`sparse`: the explicit model in
doubles, `exact`: in rationals, `dd`: the model as BDDs, doubles). The
explicit engines solve unbounded reachability with Storm's topological
solver, which on an acyclic chain computes each state's value once from
its successors' (Storm's default for doubles, GMRES to 1e-6, would not
be exact); `dd` iterates to a relative precision of 1e-15, with Sylvan's
tables given half of the process's address space where it is limited
(its default of 4 GB filled on circuits). PRISM with `-sparse` or
`-hybrid` (`-exact` refuses bounded until), `-javamaxmem` (`--prism-heap`,
4g by default, a third of `--mem-cap` under the cap; the release runs
give it a third of the lane's memory,
as 4g ran out parsing the larger networks' chains) and on the chains
`-cuddmaxmem` (the same), `-epsilon 1e-15` and no cap on the iterations:
on an acyclic chain they reach the exact values after as many as the
chain is long, and stop. In doubles, a long chain's probability of its
observations can underflow to 0 (the long hidden Markov models and
loops): that is the engine's failure, which Storm's `exact` does not
have. The answers are `result X` and `Result: X`; the time is the
process's, the components' runs added (PRISM's JVM starts each), with
the checkers' own building and checking in `inproc_seconds` (`storm_inference_seconds` on
the weather factory). The translation is not timed, as no translator
written for one baseline is (the introduction); its seconds over all the engines of an instance are bounded by the time
limit, past which the instance is a timeout. Tests:
`tests/test_modelcheckers.py` against `gen_geni.py weather N H
--reference`, `tests/test_dice2prism.py` (every program of data/dice and
data/nodice translates or is refused; with STORM_PYTHON and PRISM_DIR
each engine's answers, read as bench.py reads them, against the exact
distribution: observations, integer operations, a function, `iterate`
and a joint).

Dry run: weather factory (seven instances): Storm
(all three engines) and PRISM `sparse` agree with the dynamic program to
1.5e-15 where they finish; Storm `sparse` is the fastest (0.4 to 8 s, a
timeout at 17 factories and at 500 days), `exact` and `dd` take 7 to
104 s, PRISM `sparse` 11 to 13 s (JVM start-up), PRISM `hybrid` times out
everywhere at 120 s, and Storm `dd` runs out of BDD nodes at 17 factories.

Geni's router grids (`geni_router_N`): the engines that solve the whole
chain, Storm `exact` and `sparse` and PRISM `sparse`, time out from N =
18 (Storm) and N = 20 (PRISM) at 600 s. The answers at larger N do not
come from such solves. At N = 24 PRISM reports 8,925,478,909 states and
"Switching to MTBDD engine, as number of states is too large for Sparse
engine", after which its Jacobi iteration takes 0.04 s; it answers N = 24
to 40 that way. Storm `dd` answers every N up to 40 with the same kind of
computation: value iteration in doubles over decision diagrams. It stays
small and fast only because every probability in these programs is within
6e-7 of 0.5. The answer is 0.50000045 at every N from 4 to 40, within the
tolerance of the reference, and it counts as an answer, as any system's
does.

Past 2^63 reachable states PRISM has no engine that answers. `-sparse`
and `-hybrid` switch to MTBDD past 2^31 states, as above. The MTBDD
engine's unbounded until (`PrismMTBDD.ProbUntil`, line 413) reads the
reachable states from the model's offset-labelled diagram (ODD), which
PRISM builds only up to 2^63 states (`ODDUtils.BuildODD` returns null
past it), and PRISM stops with a `NullPointerException` and "Error:
Caught unhandled exception, aborting". The explicit engine indexes
states by Java ints. 109 records of the coverage runs end that way, on
Markov chains of 1.8e19 to 2.8e70 states: Geni's routers from N = 56,
TPC-H queries, the long hidden Markov models, the circuits and the UAI
networks' chains. They used 0.3 to 1.4 GB of resident memory, and eight
of them, run again under the memory cap, gave the same exception. Nor does
forcing an engine help on those eight: with `-mtbdd` each stops in the same
exception (4 to 288 s), and with `-explicit` five run out of Java's heap
while PRISM enumerates the states and three reach their time limit. Their text names
memory only in PRISM's header ("Memory limits: cudd=..."), so
`bench.memout` classes them as errors, not memouts. They are
unanswered, and they do not justify skipping the larger sizes of their
programs (rule 15), so those larger sizes are run.

## 23. Programs as networks and Dice programs (`ground2net.py`)

Every system tries every problem (rule 15), so the systems that read
networks (Ace, Merlin, pyAgrum, pgmpy, libDAI, TensorInference.jl) are
given every program that is not a network, and the systems that read
Dice programs (Dice, Alea.jl, Roulette, noDice, Psi, Genfer, SPPL,
Geni, Pluck, and Storm and PRISM through `dice2prism.py`, section 22) every
program that has no Dice program of its own and is not a
circuit: the standard suite's graph, grid, smokers, UW-CSE, Cora and
cplint families, the literature, Biomine and TPC-H suites. bench.py
runs `tddlog --ground` on the program, as for Scallop (section 3), and
`scripts/ground2net.py` writes the ground program as a Bayesian network
of Boolean variables (UAI and its evidence file for Merlin and the UAI
systems, Hugin and `.inst` for Ace) or as Dice programs (one program
returning every query as a tuple for Alea.jl, Roulette, Psi, Genfer,
SPPL, Geni, Storm and PRISM; one program per query, listed in a `.dicemap`, for Dice
and noDice, as for the circuits). Both steps are timed, and their
seconds (`ground_s` in the record) are added to the system's. The
answers are those of the ground program's queries, which are the ground
atoms tddlog answers.

The ground program's probabilities are on independent facts: `--ground`
has turned annotated disjunctions and probabilistic rules into switch
facts and plain rules. Each fact is a root; each derived atom is the
disjunction of its rules' bodies and each body the conjunction of its
literals, built as an and-inverter graph with constants folded and
equal conjunctions shared. In a network, a conjunction of more than two
signals is a chain of conjunctions of two, so that every table is
deterministic over at most three variables; in Dice it is one `&&`.
The weight `--ground` factors out of the facts it simplifies is a
constant factor of every world and cancels in every marginal asked.
A fact of probability p is a row of two doubles whose decimals sum to
one (`fact_row`): written as 1 - p and p, the two can print as decimals
that sum to 1.00000000000000026, which `gen_bn.py`'s check of a row
refuses. That refusal stopped the network tools on uwcse2 and on six
circuit families (bb_GPA, genes, genes5, markov, population, weekend)
until 2026-10-22, and the skip rule left each family's larger
instances untried.

Recursion. A recursive component (atoms that depend on each other) is
unrolled into stages: stage 0 has the component's atoms false, stage s
applies the rules once to stage s - 1, with the atoms outside the
component at their final values. In every world each stage before the
least fixpoint adds an atom, so the stage numbered by the component's
size is the least model and the translation is exact; it stops earlier
where a stage's formulas are the previous stage's. A component with
negation inside it (negation through recursion) has a well-founded
model rather than a least one; tddlog computes it (`src/tp/aft.rs`,
the alternating fixpoint), and ground2net.py refuses such a component
(exit status 3, recorded `unsupported`) rather than unroll the
alternating fixpoint; the records name the instances it leaves out. It also
refuses a network of more than two million conjunctions, which the
unrolling of a large component (Biomine's paths) can reach; such an
instance counts as unanswered.

Under `--pr` there is no ground route: the PR task asks Z of a network,
which a ground program does not state.

Tests: `tests/test_ground2net.py` checks the network's marginals
(bn_exact.py), the tuple program's and every per-query program's
(test_dice2problog.py's enumeration) against the program's own, found
by enumerating the worlds of its facts with the well-founded model of
each, on an acyclic program with negation, constants and evidence and
on a cycle that needs all of its stages; and bench.py's ground route on
the files the script writes.

## 24. What each system is compared on

The results are partitioned by type of program, six categories:
probabilistic logic programs, probabilistic database queries, Bayesian
networks, Markov networks, probabilistic functional programs and
bit-blasted programs. Each instance counts in one category, whatever
language its source was written in, and a copy of an instance (the same
original files and the same program, by SHA-256) counts once. A UAI 2022 network
is a Bayesian or a Markov network by its structure: every file of those
sets has the header MARKOV, and `scripts/uai_types.py` calls a network
Bayesian when its factors are conditional probability tables over an
acyclic graph, writing `data/uai_types.tsv`, which the processing reads.
Every system given some instance of a category is compared on all of
the category's instances, as the README's figures and the results
page's plots draw them, and an instance a system does not answer counts
as unanswered, whatever the reason. This section documents how each
system was given each suite: the program it read, in its own language or
as a translation. Each system's version, settings, the instances it is not
given and why, and its paper and link are in `data/systems.json`, from
this document and the systems' own repositories, which the results page
shows on the system's page beside these rows.

For each system, an instance is its own, native to it, translated for
it, or not given. An instance is the system's **own** if it comes from
the system's own papers, benchmarks or distribution. It is **native** if
the program the system read is in the system's own language, as the
benchmark's authors or its generator wrote it, and **translated** if a
translator wrote that program from a program in another language: line
for line (`problog2pita.py`, or the ProbLog program of a Dice program, a
cplint LPAD or a network, which `tddlog --program` writes and
`dice2problog.py`, `lpad2problog.py` and `gen_bn.py` wrote before), or doing some of the system's work for it (grounding,
unrolling, bit-blasting); a network that a converter wrote in another
network format for a tool that does not read the original's (`gen_bn.py`
writes a Hugin file of a UAI or BIF network for Ace, and a UAI file of a
Hugin or BIF network for the other network tools) is translated too.
The class is decided by the files the system's commands read, as the run
captured them, the same way in the results and on the benchmark pages:
a page marks each program it shows as the original (or a copy of it,
byte for byte or decompressed), native, a conversion of format or a
translation, the rows below give each instance its class, and a check
of the results compares every record's class with its page. A page
tells a conversion as one: its tab says that the original network was
converted to the format, and its warning that the file might make the
comparison unfair says so of a conversion. An own instance is so by
where it comes from, whatever file the system read, and is not compared
with its page. A converted program's page names the converter that the
runs which read it ran: the script in the runs before tddlog read the
original itself, `tddlog --program`, which writes the same program, in
the runs since (each run's capture records which); the rows below name
both. tddlog's input is classified the same way: it reads the ProbLog
programs and the networks natively, and the Dice programs and cplint's
LPADs in the runs since it reads them itself, which were given to it
before as `dice2problog.py`'s and `lpad2problog.py`'s translations; it is
given translations of the Roulette and Geni programs (`gen_roulette.py`,
`gen_geni.py`) and of the bit-blasted and integer programs (the circuit
Alea.jl builds). Each system's page on the results page
evaluates it separately on the instances native to it, its own among
them, and on those translated for it.

The ProbLog family is scored on the dice suite's 21 scaled loops in the
form of the ProbLog program on which it does better, so that no system
loses answers to the form chosen for it: ProbLog's d-DNNF, PITA and
aspmc ran on both forms, the recursion and the loops unrolled step by
step, in the same lanes of one job (three repetitions, tddlog on the
recursion beside them), and each system takes the form on
which it answers more of the loops, on a tie the one with the smaller
total time, for the whole suite. ProbLog is scored on the unrolled form
(15 answered against 10 on the recursion), aspmc on the unrolled form
(16 on each, 3412 s against 3949 s) and PITA on the recursion (10 on
each, 6739 s against 6752 s); tddlog stays on the recursion. An
answer that fails the answer check against tddlog's counts as
unanswered. Both forms are translations of the Dice program. Each
loop's page shows both forms and which systems are scored on each.

The first row that matches an instance's system, suite and name (a
regular expression the name starts with) classifies it; the processes
of Alea.jl and Roulette (`alea-process`, `roulette-process`, their times
with the start-up of Julia and Racket) are not systems of their own in
the categories. On an instance joined with its copies, a system's class
is the first of own, native and translated among the copies it was given.
The uncertain cells were decided as follows: Storm's and PRISM's
weather factory is native, not own (Rubicon's PRISM model, not theirs);
pgmpy has no own networks (bnlearn's networks are R's package's); a
network converted to another network format for a network tool (BIF or
UAI to Hugin for Ace, Hugin or BIF to UAI for the others) is translated,
and Ace's copy of a Hugin file, the same file with a `+` in a number
dropped where its reader refuses one (`bench.py`'s `hugin_for_ace`),
native; the observations `gen_bn.py --inst` writes of a network's evidence are the
generator's, native to Ace and tddlog, and a conversion where the
original has its own (a UAI network's `.evid`); the Dice family's
systems are given every network as a Dice program that `gen_bn.py`
writes from it, a translation; Ace reads the Dice paper's networks as
Hugin files, natively, and the Roulette paper's BIF files converted to
Hugin, and the other network tools read both converted to UAI files;
Alea.jl is given
Pluck's sequence models as a port of the Dice.jl versions in Pluck's
artifact, a translation, Pluck's own programs being the models (the
original on their pages); Dice and noDice read each other's programs as
they are; the blood type example and the other cplint programs reach
the ProbLog family through `lpad2problog.py` (through `tddlog --program`
in the runs since tddlog reads LPADs itself), which makes the blood type
example, the one program that is only that conversion, a translation,
its original on its page cplint's file; Cora's program is native,
written in ProbLog by `gen_cora.py` from cplint's dataset (its
generator, the dataset shown as its source), and so is each taint app's,
which `gen_taint.py` writes from PSouffle's ProbLog program with two
constructs respelled as ProbLog reads them alike; aspmc's form of a
ProbLog program (`bench.py`'s `run_aspmc`: the evidence asked as queries,
the same model) has the class of the program it is written from.

| systems | suites | instances | class | reason |
|---|---|---|---|---|
| ace | ace | all | own | Ace's own benchmarks (Chavira and Darwiche) |
| problog | biomine | all | own | the Biomine network of ProbLog's first paper (De Raedt et al., IJCAI 2007) |
| pita | std | `bloodtype$` | own | cplint's blood type example, which PITA ships |
| provsql | tpch | all | own | the TPC-H queries, ProvSQL's own evaluation |
| dice | dice | all | own | the Dice paper's programs and networks (OOPSLA 2020) and its repository's loops |
| alea | bitblast | all | own | the bit-blasting paper's programs (PLDI 2024), Alea.jl's own |
| alea | ppl | `arith_` | own | Cao et al.'s integer programs (UAI 2023), on Alea.jl's arithmetic branch |
| roulette | roulette | all | own | the Roulette paper's programs and networks (PLDI 2025), from its artifact |
| geni | geni | all | own | the programs of Geni's evaluation (ICFP 2025) |
| nodice | ppl | `nodice_` | own | noDice's programs (OOPSLA 2026) |
| pluck | ppl | `pluck_` | own | Pluck's four sequence models (PLDI 2025) |
| genfer | bitblast | `bb_population_` | own | Genfer's population model (NeurIPS 2023) |
| genfer | dice | `dice_(twocoins\|evidence\|grass\|noisyOr\|alarm\|murderMystery)` | own | the Dice programs Genfer's repository ships |
| psi | bitblast | `bb_linreg_` | own | Psi's R2 linear regression, as Psi's own program |
| sppl | bitblast | `bb_markov_` | own | SPPL's hierarchical Markov switching model (PLDI 2021) |
| psouffle | lit | `taint_` | own | the Android taint analyses of PSouffle's evaluation (FMCAD 2026) |
| tddlog | std_bn, ace, large, uai, uai_pr, uaipr, uaitpr | all | native | the network itself, read by tddlog's network reader (BIF, Hugin, GeNIe, UAI) |
| tddlog | roulette | `rl_bn_` | native | the Roulette paper's Bayesian networks, read by tddlog's network reader |
| ace | std_bn, large | all | native | a Hugin network, Ace's own input (a + in a number dropped where Ace's reader refuses one) |
| ace | dice | `bn_` | native | the Dice paper's Bayesian networks, as Hugin networks |
| ace | uai, uai_pr, uaipr, uaitpr | all | translated | the UAI network converted to a Hugin file (gen_bn.py), as Ace reads a UAI file only as a Bayesian network |
| ace | roulette | `rl_bn_` | translated | the Roulette paper's BIF networks converted to Hugin files (gen_bn.py), as Ace reads no BIF file |
| merlin, agrum, pgmpy, libdai, tensorinference | uai, uai_pr, uaipr, uaitpr | all | native | a UAI network, the tool's own input |
| merlin, agrum, pgmpy, libdai, tensorinference | std_bn, ace, large | all | translated | the network converted to a UAI file (gen_bn.py --uai) |
| merlin, agrum, pgmpy, libdai, tensorinference | dice, roulette | `(rl_)?bn_` | translated | the Dice and Roulette papers' Bayesian networks converted to UAI files (gen_bn.py --uai) |
| ace, merlin, agrum, pgmpy, libdai, tensorinference | std, lit, tpch, biomine, dice, ppl, bitblast, roulette, geni | all | translated | a network of Boolean variables built from the ground program (ground2net.py) |
| problog, pita, aspmc | std_bn, ace, large, uai, uai_pr, uaipr, uaitpr | all | translated | a ProbLog program written from the network, one annotated disjunction per row of a table (gen_bn.py, or tddlog --program, the same program, in the runs since tddlog reads networks itself) |
| tddlog, problog, pita, aspmc | dice | `bn_` | translated | a ProbLog program written from the network, one annotated disjunction per row of a table (gen_bn.py, or tddlog --program, the same program, in the runs since tddlog reads networks itself) |
| alea, roulette, dice, geni, nodice, psi, genfer, sppl, pluck, storm, prism | std_bn, ace, large, uai, uai_pr, uaipr, uaitpr | all | translated | a Dice program of nested conditionals over discretes written from the network, as the Dice paper writes networks (gen_bn.py), translated line for line for the systems that read another language |
| tddlog, problog, pita, aspmc | std | `bloodtype$` | translated | cplint's LPAD translated by lpad2problog.py, or tddlog --program, the same program, in the runs since tddlog reads LPADs itself |
| pita | std, lit, tpch, biomine | all | translated | the ProbLog program translated by problog2pita.py |
| tddlog, problog, pita, aspmc | std | `dice_` | translated | the Dice paper's program translated line for line by dice2problog.py, or tddlog --program, the same program, in the runs since tddlog reads Dice programs itself |
| tddlog, problog, pita, aspmc | std, lit, tpch, biomine | all | native | a ProbLog program |
| provsql | lit | all | translated | tuple-independent tables and non-recursive views (tpch2provsql.py) |
| psouffle | std, lit, tpch, biomine | all | translated | a Soufflé program written rule for rule from the ProbLog program (problog2souffle.py) |
| alea, roulette, dice, geni, nodice, psi, genfer, sppl, pluck, storm, prism | std, lit, tpch, biomine | all | translated | Dice programs of the ground program (ground2net.py) |
| dice | ppl | `nodice_` | native | noDice's programs, which are Dice programs |
| nodice | dice | `dice_` | native | the Dice paper's programs, which noDice reads as they are |
| alea | ppl | `pluck_` | translated | the Dice.jl versions of Pluck's sequence models in the paper's artifact, ported to Alea.jl (Pluck's own programs are the models) |
| tddlog, problog, pita, aspmc, alea, roulette, dice, geni, nodice, psi, genfer, sppl, pluck, storm, prism | ppl, bitblast | `(bb\|arith\|pluck)_` | translated | the Boolean circuit Alea.jl builds for a program that exists only as Alea.jl source, as ProbLog (alea2problog.jl) or as Dice programs of its gates |
| problog, aspmc | dice | `dice_(caesar-iter\|simpleCaesar\|pmc\|diamond\|ladder)` | translated | the loops unrolled step by step, the form of its ProbLog program on which the system answers more of them |
| tddlog, problog, pita, aspmc | dice | `dice_(caesar-iter\|simpleCaesar\|pmc\|diamond\|ladder)` | translated | the program's loops written as recursion over a step index, which the system's grounder unrolls, by dice2problog.py, or tddlog --program, the same program, in the runs since tddlog reads Dice programs itself |
| tddlog, problog, pita, aspmc | roulette | all | translated | the paper's model written as a ProbLog program (gen_roulette.py) |
| tddlog, problog, pita, aspmc | dice, ppl, bitblast, roulette, geni | all | translated | a loop-free Dice program translated line for line by dice2problog.py, or tddlog --program, the same program, in the runs since tddlog reads Dice programs itself |
| storm, prism | geni | `geni_weather_` | native | Rubicon's PRISM model of the weather factory (CAV 2021) |
| alea, roulette, dice, geni, nodice, psi, genfer, sppl, pluck, storm, prism | dice, ppl, bitblast, roulette, geni | all | translated | a Dice program, translated into the system's language where it reads another (dice2alea.py, dice2roulette.py, dice2prism.py and the like), its loops kept as loops |

## 25. What this release does not run

The coverage runs closed on 2026-10-22, when the results were
finalized: what they had not reached by then is not run in this
release, and the results page says so under each plot ("was not run on
N of the problems in this release"). An instance that a system skipped
by the skip rule, after it failed on a smaller instance of the same
family, was run and counts as unanswered. By system and suite:

- ProbLog, PITA and aspmc run on every program tddlog reads in the five
  program suites (dice, ppl, bitblast, Roulette's and Geni's), each on
  the ProbLog file tddlog reads.
- The standard suite's dice family (8) and networks (`std_bn`) take the
  dice suite's records of the Dice family, the network tools and Storm
  and PRISM: `bench.py` writes the same files for both (the same
  SHA-256), and each record is compared with its own job's tddlog.
- Pluck: the UAI PR sets, for lack of time, and 4 of the ppl suite's 43
  programs. Every other program reaches it through `dice2pluck.py`: the
  dice suite's 21 scaled loops, the circuits, the networks (std_bn, ace,
  large and the UAI MAR set, through `gen_bn.py`'s Dice programs) and the
  logic programs (std, lit, tpch and biomine, through their ground
  programs' Dice programs; 2026-10-22).
- Storm and PRISM: the UAI PR sets, for lack of time. The logic programs
  reach them through their ground programs' Dice programs (section 22;
  std, lit, tpch and biomine, 2026-10-22).
- Ace: what stopped on Java's thread stack or underflowed ran again with
  `--ace-stack` or in log space (section 7, the thread stack and log
  space).
- ProvSQL: section 21.
- The Dice family on taint, through the apps' ground programs' Dice
  programs. Geni, Genfer and Psi read the program through
  `by_component`, which translates the whole Dice program (several MB)
  once for each component of its value, one a query (thousands on
  taint). In the run of 2026-10-25 it charged only the runs to the
  instance's limit, and the jobs on 8 apps (andors_trail, and_roc,
  app_018, app_kQm, ginger_master, noisy_sounds, tilt_mazes, tom_droid)
  were stopped after 7.5 hours. The translators the stop ended left
  records with no message, Genfer's on andors_trail and app_018 and
  Geni's on ginger_master, noisy_sounds, tilt_mazes and tom_droid,
  which the processing leaves out.
  `by_component` now charges its translations to the limit and stops
  at it (2026-10-26), and a run of 2026-10-26 ran the
  family again on the 13 apps whose grounding finishes: those 8 and
  angulo, app_ca7, barcode, savage_knife and yaaic, whose earlier records predate the bound. Its records replace the earlier ones on those apps. Every
  system of the family failed on every app: Dice, Alea.jl and noDice
  ran out of 300 s; Geni's and Genfer's translations passed the limit
  on 12 apps (by_component's bound or the translator's own) and the
  systems ran out of time on the 13th; Psi and SPPL ran out of time on
  10 apps and their translations passed the limit on 3; Pluck ran out
  of time on 10, out of memory on 1 and its translation passed the
  limit on 2; Roulette's translator failed or passed the limit on 16 of
  its 26 runs (two forms) and its `raco make` ran out of time on 10.
  On tuio_droid and video_game the grounding (`tddlog --ground`) ran
  out of 300 s, so the family, the network systems and the model
  checkers were handed no program there: those records count as
  unanswered and need no captured program.
- The Dice family on Ace's three largest `fs` networks (`ace_fs_25`,
  `ace_fs_28`, `ace_fs_29`): not given. The family ran on them, and
  every run failed: the time limit (Dice, Alea.jl, Geni, Psi, Pluck),
  an allocation failure (Roulette, noDice), a stack overflow (Genfer)
  or an error (SPPL). A record is published with the files its command
  read, and the capture writes those files again:
  through `by_component` that is one translation of the network's Dice
  program for each of its 1275 to 1711 components and each of Dice,
  Geni, Psi and Genfer, files of up to 36 MB each. The capture had not
  finished after 15 hours and was stopped (2026-10-26). The processing
  leaves the family's records on the three networks out, in the table of the run that wrote them
  and in the earlier coverage table whose skips they replaced. Pluck's
  runs on the ground programs' Dice programs stand.

Failures that come from a lane's limits rather than from the system,
under the address-space limit of the runs before 2026-10-28 (the memory
paragraphs before section 1):
Roulette, in the narrow lanes (55 GB of address space) of the UAI
networks' tables of the Dice family, fails on all 112 networks it was
given there when it allocates its table (`memory allocation of
12884901888 bytes failed` on 111); the other Dice systems time out, run
out of memory or crash (Psi, exit -11) on them as well. Pluck's failures
on the networks and the logic programs are all RSDD's allocations that
the lane's address space refuses: 3.2 or 8.7 GB in lanes of 17 GB (all
120 networks of the UAI MAR set, within 40 to 80 s), 34 GB for std_bn,
ace and large; on the circuits, 8.7 or 17.4 GB in lanes of 25 GB.

## 26. PSouffle

PSouffle (Li, Xia, Adnan and Wang, "PSoufflé: Scaling Exact
Probabilistic Logic Inference for Program Analysis", FMCAD 2026,
<https://doi.org/10.34727/2026/isbn.978-3-85448-093-8_42>; its online
mode is Li, Chen, Dillig and Wang, "Incremental Inference for
Probabilistic Datalog", CAV 2026;
<https://github.com/psouffle-lang/psouffle>, UPL-1.0, its evaluation
directory MIT) is an exact probabilistic Datalog compiler and runtime
built on Soufflé: the compiled program evaluates the rules semi-naively,
records each rule application in a derivation graph, prunes the graph to
the queries and the evidence, and compiles its components into BDDs
(CUDD) for weighted model counting. Its README says so ("exact
probabilistic Datalog"), and approximate inference is outside the
repository. The literature suite's `taint` family is its paper's
evaluation, so it is that family's own system (rule 15); it runs on every
other program of the logic program and database suites that its
translation can express.

PSouffle is built from its repository at 3fb0fd06, with CUDD at fc30c18f,
the commit its build pins, as `souffle` with 32-bit words.

`scripts/problog2souffle.py PROGRAM OUTDIR` writes the Soufflé program
(`compute.dl`), its input relations (`input/R.facts`, a row a line, and
`input/R.prob`, each row's probability) and `names.tsv`, which maps the
Soufflé relations back to ProbLog's predicates. An argument position is
a Soufflé number where every constant that reaches it, through the
variables the clauses share, is an integer, and a symbol otherwise.

- A ground fact is a row of an input relation; a tuple given twice is one
  row of probability 1 - (1 - p)(1 - q), as ProbLog's two independent
  facts are.
- A predicate with facts and rules reads its facts from a relation of its
  own (`R__f`, with `R(x..) :- R__f(x..)`): PSouffle keeps only the input
  probability of a tuple that is both an input and derived, with or
  without its rewrites (a probe on 2026-10-25:
  `r(3)` answered 0.2 where ProbLog answers 0.48).
- A rule keeps its probability, which PSouffle draws once per ground
  instance of the rule, as ProbLog does (probed: `0.5::h(X) :- b(X,_)`
  over two rows of `b` gives 0.75 in both). The anonymous variables of a
  probabilistic clause are named, so that each instance is drawn on its
  own; `\+ A` is `!A`, the comparisons and `is` are Soufflé's.
- An annotated disjunction chooses its heads in order: the choice of the
  k-th head has probability p_k / (1 - p_1 - ... - p_(k-1)), and the k-th
  head holds where its choice does and none of the earlier ones do. The
  annotated disjunctions of facts are rows of input relations, one pair
  of relations per shape; one with a body draws its choices over the
  clause's variables.
- `query(A)` and `evidence(A, true|false)` are PSouffle's own; a query
  with variables is written with `_`. PSouffle names the code of a query
  or of evidence after its relation, so a relation takes one of each (a
  second one does not compile): a second ground atom of a relation is
  asked through a relation of no arguments of its own (`q__0() :- p(1,
  2).`), and several queries with variables of one relation are one
  query of every tuple of it. A query with a body over certain facts
  (smokers' `query(smokes(X)) :- person(X).`) is its ground queries.
  `queries.txt` lists the queries as tddlog prints their atoms.

It refuses (exit 3, `unsupported`) lists, function symbols, a disjunction
in a body, division, a non-ground fact, a negation through recursion, a
variable no positive literal binds and arithmetic on symbols. An
annotated disjunction inside a recursion (pedigree's and bloodtype's
inheritance) is refused too: the k-th head negates the earlier choices,
whose rules read the recursion, and Soufflé stratifies negation by
relation only.

`bench.py run --psouffle SOUFFLE` translates the instance, compiles the
program with `souffle --full-only compute.dl -o compute` and runs
`./compute --jobs=1 -F input -D out` within the instance's limit,
reading `out/facts.prob`. The translation is the harness's and is not
timed. The compile is timed apart from the run, as the paper times
inference and not the compile of the analysis: every record after it
keeps the compile's seconds (`compile_s`, at most 3600 s), and the
compiled program is reused by every instance whose `compute.dl` is the
same, whose record gets that compile's seconds (before 2026-10-28 the
harness gave such a record 0 and a failed run none). The processing
charges the compile to the record (`rel/charge.py`, final_all.sh's
`PSOUFFLE_COMPILE`, charged by default while the owner decides): the
system's own work from the program to the answer, as ProvSQL pays for
loading its tables. A reused compile is charged its seconds from the
record of the same run that compiled the text (the `compute.dl` its
captured call reads); an answer or error whose compile and run pass the
limit, and a compile stopped at its 3600 s, become timeouts. On the
published tables this turns 10 of PSouffle's 276 answers into timeouts
(std's s1000, cora10, cora50 and cora500; the literature's
taint_app_ca7, taint_yaaic, eventcalc4_250, mastermind32_5_8,
mastermind64_5_8 and sickness45_40) and its 10 compile timeouts (std 8,
the literature 2) into timeouts. The run is PSouffle's default full inference, which the paper's
evaluation runs as one of its two engines (`souffle_no_rewrite` in
`evaluation/full/FMCAD.py`); `--jobs=1` keeps it in its lane. The other,
`--rewrite` (the graph rewrites), loses derivations at 3fb0fd06 and is
not used: on probes of 2026-10-25 it answers
0.3 or 0.4 for a tuple that two rules derive from 0.4 and 0.3 facts
(0.58 without it), 0.5 for `q() :- a(_)` over two 0.5 rows (0.75), and
0.2995 for TPC-H's q4 through views (tpch_q4v_1; 0.41734 without it, as
the exact reference). A ground query that PSouffle does not list has
probability 0. PSouffle prints probabilities with eight decimals, so its
answers differ from the references by up to 5e-9.

On taint, PSouffle's own runner (`evaluation/full/FMCAD.py`) runs the
analysis as five stage programs (`evaluation/full/taint/programs/`), each
stage's output tuples read by the next as facts of probability 1.0: a
different distribution from the merged program. The family's instances
are the merged program its evaluation gives ProbLog
(`compute.problog.dl`, which `gen_taint.py` reads), and PSouffle runs on
the same program as tddlog and ProbLog, through the translation.

The side-channel family (`sidechannel_p1` to `sidechannel_p20`) is
PSouffle's 19 Souffle programs, and PSouffle runs on them as they are:
`gen_sidechannel.py --souffle` writes each case's `compute.souffle.dl` and
its input relations beside the ProbLog translation, and `run_psouffle`
reads that instead of translating the program back. The two readings
differ where a relation is both an input relation and derived, as
KEY_SENSITIVE, KEY_IND and RAND are in every case. ProbLog's semantics,
which the translation keeps, has such a tuple hold where its input fact
holds or a rule derives it; PSouffle keeps only the input probability of
the tuple (the ps0 probe above). Where that gives a different answer from
ProbLog's, the answer counts as wrong (rule 15), as on any other program.

PSouffle answers some recursive programs wrongly at 3fb0fd06: on

```
0.5::e(1,2). 0.5::e(2,1). 0.5::e(2,3).
path(X,Y) :- e(X,Y).
path(X,Y) :- e(X,Z), path(Z,Y).
```

it answers path(1,3) 0.375 and path(2,3) 0.625, where ProbLog and tddlog
answer 0.25 and 0.5: the extra 0.125 is the world with e(1,2) and e(2,1)
but not e(2,3), in which the cycle between path(1,3) and path(2,3)
supports itself. Its answer under evidence on the same cycle is wrong in
the same way. Taint's rules are recursive through a dozen relations, so
an answer of PSouffle that differs from tddlog's is not taken as either
one's error by itself; a wrong answer counts as a failure (rule 15).

In the timed run of 2026-10-25, against tddlog and the
references: it agrees within 5e-9 on every program it answered of the
lit suite but taint's yaaic (section 28: eventcalc4_250, liftsep 3 to 8,
rgraph 6 to 12, 10 taint apps and 32 programs of the older families), of
TPC-H (175) and of the standard suite (30); it answers wrongly the
standard suite's graph programs g10_25 to g18_50 (8, by up to 0.71) and
all 10 Biomine programs (by up to 0.62), whose graphs have cycles.
Elsewhere it stops on CUDD's allocation (`bddOr returned NULL`,
`Cudd_bddAnd failed`), runs past the limit, or is refused by the
translation (pedigree's 14 programs and bloodtype: their annotated
disjunctions inside a recursion).

Not given: the networks (`gen_bn.py`'s programs, std_bn and the network
suites) and the probabilistic functional programs, for lack of time in
this release; the translation could write them (an annotated disjunction
per row of a table), but each row would be a clause of its own in the
compiled C++ program. Tests: `tests/test_problog2souffle.py` (the
translation, and with `PSOUFFLE` set PSouffle's answers on hand-computed
programs and on small eventcalc, liftsep and rgraph programs against
`lit_exact.py`).

## 27. vProbLog

vProbLog (Tsamoura, Gutiérrez-Basulto and Kimmig, "Beyond the Grounding
Bottleneck: Datalog Techniques for Inference in Probabilistic Logic
Programs", AAAI 2020, technical report arXiv:1911.07750;
<https://github.com/jjjxia/Vproblog>, built on VLog, no licence stated)
computes the TcP operator of Vlasselaer et al. with semi-naive evaluation
and magic sets on a non-ground program, each derived atom's formula an
SDD. Run to its fixpoint (`--maxDepth` unbounded, the default) its
answers are exact, and it has an implementation, so rule 15 makes it a
baseline. It is not run in this release:

- Its language is ProbLog's positive, function-free programs with
  probabilities on facts: "We focus on programs without negation for
  simplicity" (Section 1), and the rules carry no probabilities
  (Section 2). taint and eventcalc negate in their rules, and taint's
  rules are probabilistic. PSouffle's evaluation ran it on taint's five
  stages with a patched VLog that draws a variable per grounding of a
  probabilistic rule (`evaluation/full/scripts/transform_to_vproblog_native.py`),
  which is not the released system.
- Its input is the paper's rewriting of the program into Datalog over
  formulas (the `t_`, `agg_`, `tt_`, `d_` and `req_` predicates, an
  external `equiv` over SDDs), with an `edb.conf` and the facts as CSV
  files: a translator of its own, which this release does not have.

The negation-free families whose probabilities are on facts only (graph,
pdb, liftsep, rgraph, Biomine) are what it could be given once that
translator is written.

## 28. What checks the taint family

Taint has no exact reference of its own: ProbLog (`-k ddnnf`, `-k sdd`)
runs out of 1200 s on angulo, the smallest app, and `lit_exact.py` has no
method for it. tddlog's answers on taint are checked against PSouffle's
(rule 2), the paper's own system, on each app where both finish: an app
is checked where every answer of the two passes the answer check.

In that run 10 of the 15 apps are checked: and_roc, andors_trail,
app_018, app_ca7, app_kQm, barcode, ginger_master, noisy_sounds,
tilt_mazes and tom_droid, every answer (56,894 to 191,605 an app) within
5e-9. On yaaic 11 of 157,963 answers differ by up to 1e-3, PSouffle's the
higher (reachablecm(0,2574) 0.998999 against tddlog's 0.998001, which is
0.999 squared), as on its positive cycles; yaaic is not checked. On
savage_knife PSouffle ran past 300 s; on angulo, tuio_droid and
video_game tddlog did (angulo answers in 314 s untimed), and PSouffle
stopped on CUDD's allocation or ran past 300 s too.

The run of 2026-10-26 ran tddlog again beside the Dice family, at
the build with the successor grounding (1b835f25), and its answers give
the same check: the same 10 apps within 5e-9 of PSouffle's, yaaic's 11
answers up to 1e-3 apart, savage_knife answered with no PSouffle answer
to check, and angulo past 300 s. Another run ran tddlog at that build on
tuio_droid and video_game: both ran past 300 s.

A check of 2026-10-26 settled yaaic. ProbLog (`-k ddnnf`) and PITA, with the
driver that prints every instance of a query (section 2), were each given the
app one ground query at a time, on the 11 queries where PSouffle and
tddlog differ, and both print tddlog's answers: reachablet(669) 0.999 and
reachablecm(0,2574) 0.998001, where PSouffle has 0.999997 and 0.998999.
These 11 answers are yaaic's reference (shown on its page as
`exact:by-query`): tddlog is checked against them, and
PSouffle's answers on yaaic count as wrong.
