# Alea.jl benchmark programs

The benchmark programs of two papers on Alea.jl (Tractables/Alea.jl, the
Julia successor of Dice, called Dice.jl in the first) and the Dice.jl
programs of a third that compares with it, each a Julia file
in the form `scripts/alea_bench.jl` reads: it defines `model()`, which
returns what an `@alea begin ... end` block returns, and `QUERY` (`:pr`
or `:expectation`), with `IGNORE_ERRORS = true` where the program ran
`pr(...; ignore_errors = true)`. Each file names its source; arith,
hybit and bitblast are under the Apache License 2.0 (the repository's
LICENSE), pluck under CC-BY-4.0 (the artifact's record).

## arith: Cao et al., UAI 2023

William X. Cao, Poorva Garg, Ryan Tjoa, Steven Holtzen, Todd Millstein,
Guy Van den Broeck, "Scaling Integer Arithmetic in Probabilistic
Programs", UAI 2023 (PMLR 216, pp. 260-270, arXiv 2307.13837). The
programs of the paper's Table 1, from `experiments/dicejl/` of the
`arithmetic` branch of Tractables/Alea.jl at commit
7e9ff23517086eb4518845e50218bf18b4d9f2a5, and gcd-small, -medium and
-large from its `examples/gcd_8.jl`, `gcd_10.jl` and `gcd_12.jl` (the
paper's gcd on 8, 10 and 12 bits). The port is mechanical: `using Dice`
becomes `using Alea`, `@dice` `@alea`, the BenchmarkTools harness
(`function fun() ... end`, `@benchmark`, `println`) is dropped, the block
`c = @dice begin ... end` becomes `model() = @alea begin ... end`, and
`pr(c)` becomes `QUERY = :pr`. caesar_huge, disease_large and plankton
are in the directory but not in the table. The directory's
`disease_*_modified.jl` variants are left out.

A program that returns a tuple (luhn, ranking, linext) is answered here
one component at a time; the paper's `pr(c)` computes the tuple's joint
distribution.

## hybit: Garg et al., PLDI 2024

Poorva Garg, Steven Holtzen, Guy Van den Broeck, Todd Millstein, "Bit
Blasting Probabilistic Programs", PLDI 2024 (PACMPL 8, article 182, DOI
10.1145/3656412, arXiv 2312.05706); artifact Zenodo
10.5281/zenodo.10901544 and branch `hybit` of Tractables/Alea.jl. The
programs of the paper's Table 2, at the bits and pieces of its HyBit
column, from `hybrid/benchmarks/` of Tractables/Alea.jl at commit
cf5e5b8d6d34e63ae9ab614baf242d0d3520aa8b (the `hybit` branch's
`benchmarks/` with Dice.jl's names changed to Alea's), except
conjugate_gaussians, whose form for the table (15 bits after the binary
point, 16 pieces, an expectation) only the `hybit` branch has (commit
96b69a58909cb873f7c285977cafe3d0aece6bb3), run with the observations 1
and 2 of the table's runs rather than the file's 8 and 9 (the file's
header gives the evidence). The programs' bodies are
unchanged but for the changes each file's header states: the parameters
the source reads from the command line are constants, a type defined
inside the block moved out of it, and pi.jl defines the `round` it calls,
which Alea does not have. A program that returns a Boolean is queried
for its probability (`:pr`), one that returns a number for its
expectation (`:expectation`), the quantities whose error the paper
reports.

## pluck: Bowers et al., PLDI 2025

Maddy Bowers, Alexander K. Lew, Joshua B. Tenenbaum, Armando
Solar-Lezama, Vikash K. Mansinghka, "Stochastic Lazy Knowledge
Compilation for Inference in Discrete Probabilistic Programs", PLDI 2025
(PACMPL 9, article 222, DOI 10.1145/3729325); artifact Zenodo
10.5281/zenodo.15051140 (CC-BY-4.0). The Dice.jl baselines of the
sequence models of the paper's Table 1 (HMM of 50 steps, PCFG of 23
terminals, String Editing of 4 to 5 characters, Sorted List Generation
of 8 elements), from `PluckArtifact.jl/src/table1/sequence_models/dice/`.
Its other rows (networks, diamond, ladder) are the Dice paper's. The
port is mechanical: Dice.jl's `Dice.List`, `Dice.Nil` and `Dice.Cons`
become Alea's `List`, `Nil` and `Cons`, `@dice_ite` `@alea_ite`, the
`add_benchmark!` harness is dropped, and the `pr(E)` of the program's
function becomes `E`, which `model()` returns. The programs use Alea's
inductive types (`@inductive`, `match`, `Nat`) with fuel, as in Dice.jl.

## bitblast: the bit-blasting paper's Section 2

The three programs of Section 2 of the bit-blasting paper (above),
from `hybrid/benchmarks/` of Tractables/Alea.jl at cf5e5b8: `genes.jl`
(`or/or.jl`, Figures 1 and 2: the gene expression program, T genes),
`multimodal.jl` (`multimodal/multimodal.jl`, Figures 3 and 4: the
Yao-Vehtari-Gelman model) and `conjugate_ex3.jl`
(`conjugate_gaussians/conjugate_gaussians.jl`, Figures 5 and 6: conjugate
Gaussians observed at 8 and 9). The sizes they read from the command line
are constants; the distributions are built inside the `@alea` block, the
plotting and results files dropped. The paper plots the posterior of the
last two, which are queried here for their expectation. `genes.jl` has
the setting of the runs behind Figure 2 (`or/results_T.txt`): 20 bits
after the binary point and 1024 pieces, error 4.8e-7 at every T. Section
2.1's text speaks of 25 bits (16 after the point), 4096 pieces and 53K
flips; 16 bits and 1024 pieces give 53 588 flips at 5 genes. `bench.py
--suite bitblast` runs these and the hybit programs on ladders of size,
setting a program's constant with `alea_bench.jl --set`.

`laplace.jl` is the one other program of the paper's repository with
observations that the tables leave out: a mixture of two Laplace
distributions observed under normal noise
(`hybrid/benchmarks/laplace_scaling/laplace_scaling.jl` at cf5e5b8, its
header says what changed). Its `hmm` and `hmm_slicstan` are left out:
`hmm` observes only the last step, whose state does not depend on the
queried parameter, and `hmm_slicstan` calls its observations as
arguments of `ifelse`, so that every branch's is made. The paper's
Figure 13 times spacex, weekend, altermu2 and addFun_max over their
pieces; `bench.py --suite bitblast` runs those ladders too (`bb_NAME_p`).
The artifact's other experiments are left out too: each builds one
distribution (Figure 12's normal of 8 pieces at up to 15 bits, the gamma
densities' observations at up to 14 bits), with no program around it.

## Programs of other papers, written in Alea.jl

Programs from other papers on exact and hybrid inference, written as
Alea.jl programs in the form above, each in a directory named after its
source with a README that cites it: `fairsquare` (FairSquare's decision
trees, as SPPL solves them), `sppl` (SPPL's hierarchical Markov switching
model), `psi` (R2's and PSI's linear regression), `wmi` (the path and
star of weighted model integration), `gubpi` (the pedestrian) and
`genfer` (Genfer's population model, its Poisson counts truncated).
`bench.py --suite bitblast` runs each on a ladder of its precision or
size.
