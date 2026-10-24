# tddlog: the baselines branch

<p align="center">
  <img src="https://raw.githubusercontent.com/Tractables/tddlog/main/docs/logo.svg"
       alt="tddlog" width="340">
</p>

This branch is tddlog's main branch plus the code and programs that
reproduce the comparisons published in main's
[README](https://github.com/Tractables/tddlog#results) and in
[docs/benchmarks.md](docs/benchmarks.md). It is one commit on top of
main. tddlog needs none of it: build and use tddlog from main, whose
README this file replaces here.

## What the branch adds

- `scripts/bench.py`, the benchmark driver. `gen` writes a suite's
  instances, `run` runs every system named on its command line on them
  and appends a JSON line a run, `table` prints the tables
  (docs/benchmarks.md, Setup).
- The scripts that feed a baseline: `problog2pita.py` (a ProbLog program
  as a cplint LPAD, for PITA), `ground2scallop.py` (a ground program, as
  `tddlog --ground` prints it, for Scallop), `dice2alea.py` (a Dice
  program as a Julia script for Alea.jl), `alea_bench.jl` (runs an
  Alea.jl program, or writes the circuit Alea builds for it as a ProbLog
  program and as Dice programs), `gen_geni.py` and `gen_roulette.py` (the
  programs of Geni's and Roulette's evaluations for tddlog, ProbLog and
  Dice, and their exact answers), `dice2problog_onehot.py` and
  `tddlog-onehot.sh` (the one-hot arm), `dice_scale.py` and
  `bench_one.py` (Dice's loops scaled), and `scripts/samplers/` (Stan,
  WebPPL and Psi).
- Copies of the baselines' benchmark programs: `data/alea` (Alea.jl's,
  the bit-blasting paper's and Pluck's programs), `data/hybit` (the
  bit-blasting artifact's Stan, WebPPL and Psi programs), `data/geni`,
  `data/nodice` and `data/roulette`. Each directory's README gives the
  source, commit and license; NOTICE lists them.
- `benchmarks/`: the results page's data on each benchmark
  (`results/benchmarks/` on the site), a JSON file per family of
  instances: the files each system read, as
  `bench.py` wrote and handed them (a file larger than 1 MB by its first
  64 kB, with its size and SHA-256), how each was made, the command each
  system ran, and every repetition's time, answers and sizes. The files
  of a benchmark written from data the repository does not ship (Ace's
  and Biomine's archives, the UAI sets, the bnlearn networks) are shown
  by their size, SHA-256 and where the data is downloaded.
- `docs/baselines.md`: how to install each baseline and how `bench.py`
  runs it, cited below by section.
- The tests of the above: `tests/test_bench.py`, `test_alea_bench.py`,
  `test_dice2alea.py`, `test_problog2pita.py`, `test_gen_geni.py`,
  `test_gen_roulette.py`, `test_nodice.py` (noDice's programs through the
  Dice front end, against Dice's answers) and `test_counters.py` (the
  weighted CNFs of `scripts/uai_wcnf.py` against Ganak and sharpSAT-TD),
  and `tests/alea/*.jl`.

The front ends (`scripts/dice2problog.py`, `scripts/lpad2problog.py`),
the generators of tddlog's own suites, the exact references
(`bn_exact.py`, `lit_exact.py`, `tpch_exact.py`, `reliability_exact.py`)
and the Dice paper's programs (`data/dice`) are on main.

## The baselines

| system | version | runs on | fed by | `bench.py run` options | baselines.md |
|---|---|---|---|---|---|
| ProbLog | 2.3.0, PySDD 1.0.6, pyeda 0.29.0 | the program suites and Geni's programs; a reference on the others | the program itself | `--problog`, `--kbs ddnnf,sdd,fsdd` | 1 |
| PITA | the cplint pack on SWI-Prolog 9.2.9 | the program suites | `problog2pita.py` | `--pita`, `--pita-packs` | 2 |
| Scallop | scallop-core 0.2.5, top-k proofs (approximate) | the program suites | `ground2scallop.py` | `--scallop`, `--scallop-k 10,1000` | 3 |
| Dice | 1.0, the binary of the sholtzen/dice image | `data/dice`, the networks, `data/nodice`, the circuits of `data/alea`, Geni's and Roulette's programs | the program, or `gen_bn.py --format dice`, `alea_bench.jl export`, `gen_geni.py`, `gen_roulette.py` | `--dice` | 4 |
| Alea.jl | cf5e5b8 on Julia 1.11.9, CUDD 3.0.0 | the Dice programs above; the programs of `data/alea` | `dice2alea.py`; `alea_bench.jl run` | `--alea` | 5 |
| aspmc | 1.1.1, sharpSAT-TD | the program suites; with evidence, the evidence asked as queries | the program itself | `--aspmc` | 6 |
| Ace | 3.0 | the networks | the network file, with the evidence of `gen_bn.py --inst` | `--ace`, `--ace-variants auto,forceC2d`, `--ace-heap` | 7 |
| Merlin | 1.7.0, built at ad37783 | the networks, the UAI sets | `gen_bn.py --uai` | `--merlin` | 8 |
| Ganak, sharpSAT-TD | a static Ganak; sharpSAT-TD at 0c234c1 | the UAI PR sets, a second reference for log10 Z, not timed | `gen --cnf` (`uai_wcnf.py`) | `--ganak`, `--sharpsat-td`, `--sharpsat-td-decot 1,10` | 9 |
| Roulette | camoy/roulette 54d6185 from the artifact (Zenodo 10.5281/zenodo.15253254) on Racket 8.16 | `data/roulette` and the artifact's networks | the artifact's programs | `--roulette`, `--roulette-variants rsdd,nocomplement` | 11 |

The exact references written for this repository run with `--exact`:
`bn_exact.py` (a clique tree) on the networks, `lit_exact.py`,
`tpch_exact.py`, `reliability_exact.py` on the Biomine suite, and the
dynamic programs of `gen_geni.py` and `gen_roulette.py`. They check
answers and are not compared on time.

## Data not in the repository

Four suites read archives whose licenses keep them out of the tree:

- Ace's suite: `benchmarks_v1.0.zip`, distributed with Ace 3.0
  (non-commercial research use), unpacked, its `benchmarks_v1.0`
  passed as `--ace-benchmarks`.
- The UAI suites: `MAR.zip` and `PR.zip` of the UAI 2022 competition's
  tuning set
  (<https://www.ics.uci.edu/~dechter/uaicompetition/2022/TuningBenchmarks/>)
  and `PR.zip` of its final set
  (<https://www.ics.uci.edu/~dechter/uaicompetition/2022/FinalBenchmarks/>),
  each unpacked and passed as `--uai-benchmarks`.
- The Biomine suite: `human_3_oct_2018.zip` of
  <https://biomine.ijs.si/downloads/> (academic and non-commercial use),
  its directory passed as `--biomine-benchmarks`.
- Roulette's networks: the `benchmark/` directory of Roulette's artifact
  (Zenodo 10.5281/zenodo.15253254), passed as `--roulette-benchmarks`.

The standard suite's Bayesian networks (`data/bn`) and the large suite's
(`data/bn-large`), of the bnlearn repository, are not in the repository
either, as no license for redistributing them is established:
`python3 scripts/bench.py fetch` downloads each file `data/fetch.txt`
lists into its place and checks its SHA-256, and `gen` stops, naming it,
while one is missing. The tests do not read them.

## Running

The published runs used tddlog linked statically:

```
RUSTFLAGS='-C target-feature=+crt-static -C relocation-model=static' \
  cargo build --release --target x86_64-unknown-linux-gnu
```

and passed the binary as `--tddlog`. Every run is single-threaded. The
release runs ran on lab machines in parallel lanes, one physical core a
lane (`run --cpus`), each process under an address-space limit (`ulimit
-v`) of its lane's share of memory, the systems of an instance back to
back in one lane; timings compare only within one run
(docs/benchmarks.md, Setup). The commands below are bash; the
upper-case words and variables are paths to fill in, and these arrays
name the baselines:

```
PROGRAMS=(--kbs ddnnf,sdd,fsdd --problog "$PROBLOG" --pita "$SWIPL" --pita-packs "$PITA_PACKS"
          --aspmc "$ASPMC_PYTHON" --scallop "$SCLI" --scallop-k 10,1000)
NETWORKS=(--ace "$ACE_DIR" --ace-variants auto,forceC2d --ace-heap "$ACE_HEAP" --merlin "$MERLIN")
COUNTERS=(--ganak "$GANAK" --sharpsat-td "$SHARPSAT_TD" --sharpsat-td-decot 1,10)
JULIA="julia --startup-file=no --threads=1 --gcthreads=1 --project=$ALEA_JL {file}"
DICE_ALEA=(--kbs '' --dice "prlimit --stack=unlimited $DICE {file}" --alea "$JULIA")
RACKET="env PLTUSERHOME=$PREFIX/home PLTADDONDIR=$PREFIX/addon $PREFIX/racket/bin/racket"
```

`bench.py run` appends to `--out` (by default `results.jsonl`), and
`bench.py table FILE` prints the tables of a file. `--rotate R` below is
repetition R, which rotates the order of the systems on an instance.

### The README's table and docs/benchmarks.md's current results

The standard, literature, Ace's and large suites and the UAI suites, in
one job:

```
bench.py gen --dir B
bench.py gen --suite lit --dir L
bench.py gen --suite large --dir G
bench.py gen --suite ace --ace-benchmarks ACE_BENCH --dir A
bench.py gen --suite uai --uai-benchmarks MAR_TUNE --dir U
bench.py gen --suite uai --uai-benchmarks PR_FINAL --dir UF --cnf --pr
bench.py gen --suite uai --uai-benchmarks PR_TUNE --dir UT --cnf --pr

# the standard suite: the programs, then the networks (bn, bnev)
bench.py run --dir B --families graph,grid,smokers,uwcse,cora,cplint,dice --timeout 300 "${PROGRAMS[@]}"
bench.py run --dir B --families bn,bnev --exact --timeout 300 "${PROGRAMS[@]}" "${NETWORKS[@]}"
# the literature suite
bench.py run --suite lit --dir L --exact --timeout 300 "${PROGRAMS[@]}"
# the large networks and Ace's suite
bench.py run --suite large --dir G --exact --timeout 600 --kbs '' "${NETWORKS[@]}"
bench.py run --suite ace --ace-benchmarks ACE_BENCH --dir A --exact --timeout 600 --kbs '' "${NETWORKS[@]}"
# the UAI MAR tuning set: the marginals, then the partition function of the
# same networks (--pr), aspmc asked P(e) there; Ace in a lane's process with
# half its memory as heap, in log space (Xlog) on the partition function
UAI=(--suite uai --uai-benchmarks MAR_TUNE --dir U --kbs ddnnf,sdd,fsdd --problog "$PROBLOG"
     --pita "$SWIPL" --pita-packs "$PITA_PACKS" --ace "$ACE_DIR" --ace-heap "$ACE_HEAP"
     --merlin "$MERLIN" --exact --timeout 300 --no-skip)
bench.py run "${UAI[@]}" --ace-variants auto,forceC2d
bench.py run "${UAI[@]}" --ace-variants Xlog,forceC2d --aspmc "$ASPMC_PYTHON" --pr
# the UAI PR sets: the partition function
bench.py run --suite uai --uai-benchmarks PR_FINAL --dir UF --pr --kbs '' --merlin "$MERLIN" --exact \
  "${COUNTERS[@]}" --timeout 300 --no-skip
bench.py run --suite uai --uai-benchmarks PR_TUNE --dir UT --pr --kbs '' --merlin "$MERLIN" --exact \
  "${COUNTERS[@]}" --timeout 300 --no-skip
```

### TPC-H and Biomine

```
bench.py gen --suite tpch --dir T
bench.py gen --suite biomine --biomine-benchmarks BIOMINE --dir M
bench.py run --suite tpch --dir T --exact --timeout 600 "${PROGRAMS[@]}"
bench.py run --suite biomine --biomine-benchmarks BIOMINE --dir M --exact --timeout 600 "${PROGRAMS[@]}"
```

### Dice, Alea.jl and the programs of their papers

Each suite has a pass of the references first (ProbLog's d-DNNF, or the
exact answers), then the timed repetitions, tddlog and the baselines on
an instance back to back:

```
# data/dice's programs and the networks, and the loop programs
bench.py gen --dir B --families dice,bn
bench.py gen --suite indicators --families dice --dir LOOPS
bench.py run --dir B --families dice,bn --no-skip --strategies '' --kbs ddnnf --problog "$PROBLOG" \
  --exact --timeout 300
bench.py run --suite indicators --families dice --dir LOOPS --no-skip --strategies '' --kbs ddnnf \
  --problog "$PROBLOG" --timeout 300
bench.py run --dir B --families dice,bn --no-skip "${DICE_ALEA[@]}" --rotate R --timeout 300
bench.py run --suite indicators --families dice --dir LOOPS --no-skip "${DICE_ALEA[@]}" --rotate R --timeout 600

# the ppl suite: the circuits of data/alea's arith, hybit and pluck
# programs, which Alea.jl writes at gen, and noDice's programs
bench.py gen --suite ppl --dir P --alea "$JULIA"
bench.py run --suite ppl --dir P --no-skip --strategies '' --kbs ddnnf --problog "$PROBLOG" --timeout 300
bench.py run --suite ppl --dir P --no-skip "${DICE_ALEA[@]}" --rotate R --timeout 600

# the bit-blasting paper's programs: tddlog's three arms and Alea.jl
bench.py gen --suite bitblast --dir BB --alea "$JULIA"
bench.py run --suite bitblast --dir BB --strategies auto,program,decision --kbs '' --alea "$JULIA" \
  --rotate R --timeout 600

# Geni's programs
bench.py gen --suite geni --dir GN
bench.py run --suite geni --dir GN --strategies '' --kbs '' --exact --timeout 600
bench.py run --suite geni --dir GN --kbs ddnnf --problog "$PROBLOG" \
  --dice "prlimit --stack=unlimited $DICE {file}" --alea "$JULIA" --rotate R --timeout 600

# Roulette's programs and networks
bench.py gen --suite roulette --roulette-benchmarks RL_BENCH --dir RL
bench.py run --suite roulette --roulette-benchmarks RL_BENCH --dir RL --strategies '' --kbs '' --exact \
  --timeout 600
bench.py run --suite roulette --roulette-benchmarks RL_BENCH --dir RL --strategies auto,program,decision \
  "${DICE_ALEA[@]}" --roulette "$RACKET {file}" \
  --roulette-variants rsdd,nocomplement --rotate R --timeout 600
```

### The one-hot arm, Dice's scaled loops and the samplers

The tables on Dice's programs have a column for tddlog on the programs'
one-hot translation (variant `auto+onehot`): `dice2problog_onehot.py` is
`dice2problog.py` as it was before it held integers in bits. Each Dice
program is translated into a directory `$ONEHOT` under its instance's
name (the name `bench.py gen` gives it), and `tddlog-onehot.sh` runs
tddlog on that file in place of the one `bench.py` names:

```
export ONEHOT=DIR TDDLOG=PATH
python3 scripts/dice2problog_onehot.py data/dice/PROGRAM.dice > "$ONEHOT/INSTANCE.pl"
bench.py run ... --strategies auto,auto+onehot --tddlog-arm auto+onehot=scripts/tddlog-onehot.sh
```

The table of Dice's loops scaled to more steps: `dice_scale.py` sets the
count of a program's `iterate` call, and `bench_one.py` runs one system
on one program as `bench.py run` would, appending its record to a file.
The four systems ran back to back in one lane, their order rotated by
program and repetition, 600 s each, and ProbLog's d-DNNF once as the
reference (300 s):

```
for k in 100 400 1600 6400; do
  python3 scripts/dice_scale.py data/dice/diamond.dice $k S/dice_diamond_k$k.dice
done   # and ladder at the same counts, pmc1 at 10 20 40 75, pmc2 at 10 20 51,
       # caesar-iter at 100 1000 3000, simpleCaesar at 100 300 1000
python3 scripts/dice2problog.py S/NAME.dice > S/NAME.pl
python3 scripts/dice2problog_onehot.py S/NAME.dice > "$ONEHOT/NAME.pl"
DICE="prlimit --stack=unlimited $DICE" ALEA="$JULIA" PROBLOG=... TDDLOG=... \
  python3 scripts/bench_one.py scripts SYSTEM S/NAME REP CPU 600 scaled.jsonl
# SYSTEM: tddlog, onehot, dice, alea; problog for the reference
```

The bit-blasting section's samplers (Stan, WebPPL) and Psi ran on the
bit-blasting artifact's own programs, copied in `data/hybit`, through
`scripts/samplers/` (docs/baselines.md, section 10): three install
scripts, the artifact's runners `stan.sh` and `webppl.sh`, and
`samplers.py`, which prints the runs as commands for itself, one a
processor:

```
scripts/samplers/install_stan.sh; scripts/samplers/install_webppl.sh; scripts/samplers/install_psi.sh
export RUN_STAN=scripts/samplers/stan.sh RUN_WEBPPL=scripts/samplers/webppl.sh \
  PROGS=data/hybit STAN_HOME=... WEBPPL_HOME=... PSI=... STAN_WORK=DIR
samplers.py prepare     # `compile MODEL` lines, run before the others
samplers.py jobs 1,2,3 samplers.jsonl
samplers.py psijobs psi.jsonl
```

That section's Dice and ProbLog columns came from the commands above at
an earlier tddlog.

## Testing the branch

The branch's tests run as main's do:

```
cargo test --profile quick
python3 -m unittest discover -s tests -p 'test_*.py'
```

The Python tests need numpy. The tests that run a baseline are skipped
unless it is installed: Alea.jl with `TDDLOG_ALEA` set to the Julia
command, Ganak and sharpSAT-TD with `GANAK` and `SHARPSAT_TD` set to their
binaries.
