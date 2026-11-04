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

They run on neuppl/roulette's main (commit
60f77cb13839cccafd018f4a8281758355492483, the packages `roulette-lib` and
`roulette`, Apache License 2.0), whose `#lang roulette/example/disrupt` is
the artifact's language with two differences the drivers follow: `query`
takes the body alone (the artifact's took a second argument, whether to
take one value's probability as one minus the others') and divides each
value's probability with the observations by their sum; and the language
runs each expression of a module as a query of its own at the module's
level, printing its distribution, so the drivers' printing is a
definition (`print.rkt`'s `statement`).

- `motivating.rkt`, `diamond.rkt`, `ladder.rkt`, `caesar.rkt`: copies of
  `benchmark/modular/NAME.rkt`. Only the head comment and the last line
  change: the last line keeps the artifact's `(time (query ...))` and
  prints the answers with `print.rkt`'s `emit`, one `roulette_q ATOM P
  VALUE` a line, P in full (Roulette prints six digits) and VALUE the
  program's value, under the atom names of gen_roulette.py's ProbLog
  program. `caesar.rkt` queries as `(query (main SIZE))` and seeds its
  generator in a statement.
- `scaling/network.rkt`, `scaling/hardware.rkt`: the artifact's
  `benchmark/scaling/` programs in `#lang roulette/example/disrupt` (the
  artifact's `roulette/benchmark/disrupt` wrote each once for each engine
  and chose one, `-m rsdd`); the rest unchanged.
- `network.rkt`, `hardware.rkt`: drivers that require them and print the
  distribution: `m(K,V)` and `m(K,16)` for a lost message; `res(T)` and
  `res(4)` for a run out of fuel.
- `hardware-bitwidth.rkt BW`: `hardware.rkt` on the artifact's bit-width
  ladder (`-p (BW 2 3)`), printing `res(T)` and, for a run out of fuel,
  `res(L)`, L one past the largest total printed, as gen_roulette.py
  labels it. The artifact's third ladder (`-p (2 n 3)`, n from 5 to 20)
  is not run: at bit width 2 `(int n)` saturates at 3, so every n of it
  is the same program.
- `hmm.rkt QUERY N`: the Roulette implementation of the artifact's
  `benchmark/hmm.rkt` (its model, query patterns and parameters; the Dice
  and dynamic programming implementations run other systems), queried as
  `(query e)` where the artifact wrote `(with-observe (query e #f))`, its
  answer printed as `roulette_q q P #t`.
- `bn.rkt NETWORK ASKS`: a Bayesian network in Roulette's own BN
  language, the package's `roulette/example/bn` (its reader and its
  `make-run`, the artifact's `benchmark/bayes.rkt`), on a BIF file, asked
  what the file ASKS asks: the variable order, the observations and the
  queried variables, or under the PR task the observations whose
  conjunction's probability is P(e). The network is evaluated once at the
  module's level, the observations observed there, and each query asked
  as the language asks a module-level expression (`display-pmf`), its
  distribution printed in full, `roulette_b VARIABLE VALUE P` (P(e) as
  `roulette_e P`). bench.py runs it on every Bayesian network
  (`roulette_bif`): on one of the artifact's nine, the artifact's own BIF
  file (`benchmark/bayesian-networks/NAME.bif`) with main.rkt's queried
  variable and variable order (`networks.rktd`); on another, the BIF file
  `scripts/gen_bn.py` writes of the network (`bif_programs`: the same
  variables and tables, renamed as words of the BN language's lexer) with
  the instance's observations and queries, the variables in the order of
  the lets of its Dice program. A Markov network, whose factors BIF does
  not hold, runs the Dice program `dice2roulette.py` translates.
- `print.rkt`: `emit` and `statement`.
- `caesar_obs.rkt N` prints the first N observations of the caesar
  program, `(random 0 26 rng)` from a generator seeded with `(random-seed
  0)`, as the benchmark draws them; `caesar_obs.txt` holds the first
  100000 (Racket 8.16), one a line.

## Two findings in the artifact

- The artifact's `query` (the `#:complement-of` option of
  `example/disrupt/rsdd.rkt`) computes one value's probability as one
  minus the others'. Once the evidence's probability falls below about
  1e-16 that value is lost: on caesar with 25 letters key(0) prints 0
  against 0.519, with 30 letters 0 against 0.99991; with 5 letters it is
  off by 1.5e-6 relative. `(query e #f)` gives every value to 1e-13.
  hmm.rkt already queries with #f ("TODO: shouldn't need to disable
  optimization"). neuppl/roulette's `query` has no such shortcut.
- `benchmark/gen-caesar.rkt`, which writes the Dice programs of the
  caesar comparison, is byte for byte `gen-motivating.rkt`, so the
  artifact's Dice caesar numbers are those of the motivating program.
  Its scribble source of the Dice caesar program uses `int(5)`, which
  wraps at 32, where caesar.rkt adds 32-bit vectors.
