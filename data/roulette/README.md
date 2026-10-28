# Roulette benchmark drivers

Drivers that run the scaling programs of Roulette's evaluation and print
their answers in full, for `bench.py run --suite roulette --roulette CMD`
(docs/baselines.md, section 11), and the observations of its caesar
program, which `scripts/gen_roulette.py` reads.

## Source and license

Cameron Moy, Jack Czenszak, John M. Li, Jacob Marshall and Steven
Holtzen, "Roulette: A Language for Expressive, Exact, and Efficient
Discrete Probabilistic Programming", PLDI 2025 (PACMPL 9, article 231,
DOI 10.1145/3729334). The programs are from the paper's artifact (Zenodo
10.5281/zenodo.15253254, CC-BY-4.0), a Docker image whose Roulette is a
checkout of camoy/roulette at commit
54d6185f0b5143bdaabbd9ed35bb22c9dcaddc60 (Apache License 2.0), in its
`roulette/benchmark/` directory. The files here are under CC-BY-4.0, with
the attribution above.

## Files

- `motivating.rkt`, `diamond.rkt`, `ladder.rkt`, `caesar.rkt`: copies of
  `benchmark/modular/NAME.rkt`. Only the head comment and the last line
  change: the last line keeps the artifact's `(time (query ...))` and
  prints the answers with `print.rkt`'s `emit`, one `roulette_q ATOM P`
  a line, P in full (Roulette prints six digits), under the atom names
  of gen_roulette.py's ProbLog program. `caesar.rkt` also takes
  `--no-complement`, which queries with `(query e #f)` (below), and
  wraps its print loop in `(void ...)`.
- `network.rkt`, `hardware.rkt`: drivers that require the artifact's
  `benchmark/scaling/network.rkt` and `hardware.rkt` (their `rsdd`
  submodules, as the artifact's `-m rsdd` runs them) and print the
  distribution: `m(K,V)` and `m(K,16)` for a lost message; `res(T)` and
  `res(4)` for a run out of fuel.
- `bayes.rkt NAME`: the network NAME of the artifact's `benchmark/main.rkt`
  (its .bif file, queried variable and variable order, read from
  main.rkt) under `benchmark/bayes.rkt`, its distribution printed as
  `roulette_v VALUE P`.
- `bif.rkt BIF QUERIES`: a network of another suite as the artifact runs
  its own (bench.py's Roulette variant `bayes`): `benchmark/bayes.rkt`'s
  `main` on the BIF file `scripts/gen_bn.py` writes of it
  (`bif_programs`), once for each queried variable, with the variable and
  its ancestors evaluated in the order of the lets of its Dice program;
  its distributions printed as `roulette_b VARIABLE VALUE P`.
- `print.rkt`: `emit`.
- `caesar_obs.rkt N` prints the first N observations of the caesar
  program, `(random 0 26 rng)` from a generator seeded with `(random-seed
  0)`, as the benchmark draws them; `caesar_obs.txt` holds the first
  100000 (Racket 8.16), one a line.

The hmm programs run the artifact's own `benchmark/hmm.rkt` (`racket
hmm.rkt roulette QUERY N`), which prints its answer on its last line.

## Two findings in the artifact

- Roulette's `query` (the `#:complement-of` option of
  `example/disrupt/rsdd.rkt`) computes one value's probability as one
  minus the others'. Once the evidence's probability falls below about
  1e-16 that value is lost: on caesar with 25 letters key(0) prints 0
  against 0.519, with 30 letters 0 against 0.99991; with 5 letters it is
  off by 1.5e-6 relative. `(query e #f)` gives every value to 1e-13.
  hmm.rkt already queries with #f ("TODO: shouldn't need to disable
  optimization").
- `benchmark/gen-caesar.rkt`, which writes the Dice programs of the
  caesar comparison, is byte for byte `gen-motivating.rkt`, so the
  artifact's Dice caesar numbers are those of the motivating program.
  Its scribble source of the Dice caesar program uses `int(5)`, which
  wraps at 32, where caesar.rkt adds 32-bit vectors.
