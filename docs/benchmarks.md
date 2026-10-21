# Benchmarks

tddlog against exact baselines: the results, the suites, and how the
instances and systems are set up. The code that made these tables is
not on main: it is on the repository's [`baselines` branch][baselines],
whose README says which version of each baseline ran on which programs
and which script reproduces each table. That branch holds the benchmark
driver `scripts/bench.py`, the scripts that run or feed a baseline
(`alea_bench.jl`, `dice2alea.py`, `problog2pita.py`, `ground2scallop.py`,
`gen_geni.py`, `gen_roulette.py`), the harnesses of the one-hot arm
(`dice2problog_onehot.py`), the scaled loops (`dice_scale.py`,
`bench_one.py`) and the samplers (`scripts/samplers/`), the programs of
Alea.jl's, Geni's, noDice's and Roulette's evaluations and the
bit-blasting artifact's (`data/alea`, `data/geni`, `data/nodice`,
`data/roulette`, `data/hybit`) and docs/baselines.md, how to install and
run each baseline, cited below as baselines.md.

The [results page](https://tractables.github.io/tddlog/results/) draws
every suite below as a plot of the instances each system answers within
a time limit, with a table of every instance's times; it reads the same
records as these tables.

## Current results: tddlog 0.1.0

tddlog 0.1.0 `auto` (tididi e4d60911, goatd e237c1a), linked statically,
and every baseline, each suite's systems in one job (six jobs on four
lab machines (36 physical cores and 503 GB; 20 physical cores and 503
GB; 16 physical cores and 251 GB; 16 physical cores and 125 GB),
2026-10-19 to 2026-10-21); further systems in later jobs, each beside
that job's tddlog, in parallel lanes (Ace in 12 wide lanes of 88 GB,
half of it its Java heap) (Setup). The baselines: ProbLog 2.3.0 with
d-DNNF, SDD and forward SDD compilation, PITA's BDDs and aspmc with
sharpSAT-TD on the program suites and Ace's networks, the large networks
and the UAI networks; Ace 3.0 (default route and c2d forced, and in log
space, `-Xlog`, on the UAI networks), Merlin 1.7.0's bucket-tree
elimination (`-a bte`), pyAgrum's junction tree, pgmpy's variable
elimination, libDAI's junction tree and TensorInference.jl's tensor
network contraction on the networks, and ProvSQL on TPC-H; Dice,
Alea.jl, Roulette, noDice, Psi, Genfer, SPPL and Geni, the exact systems
that read Dice programs, on the standard suite's networks, Ace's
networks, the large networks and the UAI networks and with Ace, Merlin
and the network tools on the ground programs of the standard suite, the
literature suite, TPC-H and Biomine (the ground route). Seconds are wall
time, parsing and grounding included; the limit is 300 s a run on the
standard, literature and UAI suites and 600 s on Ace's and the large
networks.

| suite | instances | tddlog finishes (total s) | faster than ProbLog, PITA, aspmc and Ace | Merlin faster | faster than pyAgrum, pgmpy, libDAI and TensorInference.jl (ProvSQL on TPC-H) | finished where no exact system did |
|---|---:|---:|---:|---:|---:|---:|
| standard | 84 | 82 (136.5) | 52 of 52 (26 under 0.2 s) | 2 of 8 (26 under 0.2 s) | 6 of 8 (26 under 0.2 s) | 2 |
| literature | 39 | 39 (10.7) | 20 of 20 (5 under 0.2 s) |  |  | 9 |
| Ace's networks | 46 | 46 (9.9) | 46 of 46 | 2 of 11 (14 under 0.2 s) | 12 of 13 (12 under 0.2 s) | 0 |
| large networks | 14 | 14 (13.4) | 14 of 14 | 8 of 14 | 5 of 14 | 0 |
| TPC-H | 208 | 205 (366.0) | 171 of 172 |  | 32 of 149 (47 under 0.2 s) | 7 |

An instance that tddlog and the fastest baseline both finish in under
0.2 s counts neither as faster nor as slower: differences of a few
milliseconds of start-up are not compared.

**Standard suite** (random graphs, grids, smokers, UW-CSE, Cora, blood
types, Bayesian networks with and without evidence, the Dice programs).
No exact system finishes grid30, which tddlog does not finish either,
nor g25_80 and g30_100, which it does (g30_100 in 122.4 s, most of the
suite's total). Against the fastest of ProbLog, PITA, aspmc and Ace on
each instance, in total time: Of the 84 instances both are given, tddlog
answers 82 and ProbLog, PITA, aspmc or Ace 78. Over the 82 that either
answers, a run that does not answer counted at the 300 s limit, tddlog
takes 136.6 s in all and the fastest of them 1 453 s; over the 78 both
answer, 5.20 s against 253.1. tddlog alone answers g20_60, g25_80,
g30_100 and grid20. Neither answers grid25 and grid30. Merlin answers
all 34 networks; the two both take under 0.2 s on 26, and of the other 8
Merlin is faster on 2: 3.9 to 4.4 times on pathfinder (bn_pathfinder_ev
0.068 s against 0.301). tddlog is faster on 6, among them bn_link_ev
(0.30 s against 29.1) and bn_munin_ev (0.78 s against 2.20), and takes
3.9 s over the 34 against Merlin's 64.5.

**Literature suite.** tddlog finishes all 39, 9 of them where no exact
baseline finishes (mnist_sum96, pedigree5_10, pedigree6_8 to
pedigree32_6 and sickness22_40). Against the fastest of ProbLog, PITA
and aspmc on each instance, in total time: Of the 39 instances both are
given, tddlog answers 39 and ProbLog, PITA or aspmc 25. Over the 39 that
either answers, a run that does not answer counted at the 300 s limit,
tddlog takes 10.7 s in all and the fastest of them 4 630 s; over the 25
both answer, 0.49 s against 429.7. tddlog alone answers mnist_sum16,
pedigree4_6, mnist_sum32, mnist_sum64, mnist_sum96 and 9 more.

**Ace's networks and the large networks.** Against Ace in total time, on
Ace's networks: Both are given 46 instances and answer all of them. Over
the 46 both answer, tddlog takes 9.92 s in all and Ace 725.3 s. On the
large networks: Both are given 14 instances and answer all of them. Over
the 14 both answer, tddlog takes 13.4 s in all and Ace 134.7 s. Merlin
runs out of memory or time on 21 of Ace's networks (ace_blockmap_10_01
to ace_blockmap_22_03 and ace_fs_07 to ace_fs_29); of the other 25 it is
faster on 2 (ace_munin3 0.20 s against 0.31, ace_diabetes 0.17 against
0.24); the two both take under 0.2 s on 14 more. On the large networks
Merlin is faster on barley, mildew, munin2 and munin3 with and without
evidence (bn_barley 1.57 s against 1.82); tddlog on diabetes, munin1 and
munin4 with and without evidence (bn_munin1 0.94 s against 26.4).

**The other network tools.** pyAgrum, pgmpy, libDAI and
TensorInference.jl read the same UAI files as Merlin and run in the same
lanes, single-threaded; TensorInference.jl with two contraction orders,
`treesa` and `greedy`, of which the faster counts. Their answers go
through the same checks as Merlin's (below), and each has columns in the
tables. On the standard suite's networks (34 instances) pyAgrum answers
34, pgmpy answers 30, libDAI answers 34, TensorInference.jl answers 34,
Merlin 34, tddlog 34. Of the 8 that tddlog and a tool both answer in
0.2 s or more, tddlog is faster than the fastest tool on 6, and takes
3.4 s over them against the fastest tool's 10.2; a tool is faster on 2
(bn_pathfinder libDAI 0.08 s against 0.31 and bn_pathfinder_ev libDAI
0.07 s against 0.30). Both take under 0.2 s on 26 more. On Ace's
networks (46 instances) pyAgrum answers 25, pgmpy answers 12, libDAI
answers 25, TensorInference.jl answers 22, Merlin 25, tddlog 46. Of the
13 that tddlog and a tool both answer in 0.2 s or more, tddlog is faster
than the fastest tool on 12, and takes 1.8 s over them against the
fastest tool's 11.2; a tool is faster on 1 (ace_diabetes libDAI 0.16 s
against 0.24). Both take under 0.2 s on 12 more. tddlog alone answers
21. On the large networks (14 instances) pyAgrum answers 14, pgmpy
answers 6, libDAI answers 14, TensorInference.jl answers 14, Merlin 14,
tddlog 14. Of the 14 that tddlog and a tool both answer in 0.2 s or
more, tddlog is faster than the fastest tool on 5, and takes 13.4 s over
them against the fastest tool's 29.4; a tool is faster on 9 (bn_mildew
pyAgrum 0.31 s against 1.64, bn_mildew_ev libDAI 0.45 s against 1.56 and
bn_barley pyAgrum 0.77 s against 1.82). On the UAI MAR tuning set (120
instances) pyAgrum answers 107, pgmpy answers 53, libDAI answers 117,
TensorInference.jl answers 113, Merlin 113, tddlog 120. Of the 101 that
tddlog and a tool both answer in 0.2 s or more, tddlog is faster than
the fastest tool on 99, and takes 295.6 s over them against the fastest
tool's 2 458; a tool is faster on 2 (uai_csp_13 pyAgrum 7.23 s against
8.14 and uai_segmentation_12 libDAI 0.29 s against 0.42). Both take
under 0.2 s on 18 more. tddlog alone answers 1. On the partition
function of the MAR tuning set's networks (120 instances) pyAgrum
answers 108, pgmpy answers 104, libDAI answers 120, TensorInference.jl
answers 115, Merlin 114, tddlog 120. Of the 103 that tddlog and a tool
both answer in 0.2 s or more, tddlog is faster than the fastest tool on
78, and takes 217.2 s over them against the fastest tool's 870.2; a tool
is faster on 25 (uai_objectdetection_25 pgmpy 1.94 s against 6.30,
uai_objectdetection_69 pgmpy 2.56 s against 4.86 and
uai_objectdetection_37 pgmpy 2.62 s against 4.87). Both take under 0.2 s
on 17 more. On the final PR set (197 instances) pyAgrum answers 14,
pgmpy answers 6, libDAI answers 15, TensorInference.jl answers 46,
Merlin 28, tddlog 145. Of the 41 that tddlog and a tool both answer in
0.2 s or more, tddlog is faster than the fastest tool on 41, and takes
43.6 s over them against the fastest tool's 3 988. tddlog alone answers
73, a tool alone 19. On the PR tuning set (121 instances) pyAgrum
answers 108, pgmpy answers 104, libDAI answers 121, TensorInference.jl
answers 115, Merlin 115, tddlog 121. Of the 103 that tddlog and a tool
both answer in 0.2 s or more, tddlog is faster than the fastest tool on
82, and takes 285.2 s over them against the fastest tool's 1 276; a tool
is faster on 21 (uai_objectdetection_69 pgmpy 2.98 s against 6.93,
uai_objectdetection_65 pgmpy 2.98 s against 6.74 and
uai_objectdetection_73 pgmpy 2.95 s against 6.71). Both take under 0.2 s
on 18 more.

**The Dice family and the ground route.** Dice, Alea.jl, Roulette,
noDice, Psi, Genfer, SPPL and Geni, the exact systems that read Dice
programs, ran in a later job, each instance beside that job's tddlog,
which their rows under Against each baseline compare with: on the
standard suite's networks, Ace's networks, the large networks and the
UAI networks as the Dice programs of `scripts/gen_bn.py` and with Ace,
Merlin and the other network tools on the standard suite, the literature
suite, TPC-H and Biomine, on the network and the Dice program that
`scripts/ground2net.py` writes of each program's ground program.
Alea.jl's and Roulette's times leave out the start-up and compilation of
Julia and Racket, as on the page of the Dice family's suites. A
baseline's answer off the reference by more than 1e-6 counts as a
failure (`wrong`): Geni on bn_child_ev; Dice on bn_insurance,
bn_insurance_ev and bn_pathfinder; PITA on ace_fs_07,
ace_mastermind_03_08_03, uai_mastermind_05_08_03_0002,
uai_or_chain_12_fg, uai_or_chain_15_fg and 8 more; Roulette on
ace_fs_07, ace_fs_10, ace_mastermind_03_08_03, ace_mastermind_03_08_04,
ace_mastermind_03_08_05 and 4 more; ProbLog on uai_promedus_24; aspmc on
uai_promedus_13, uai_promedus_15, uai_promedus_16, uai_promedus_19,
uai_promedus_24 and 8 more; Ace on uai_mastermind_04_08_04_0012,
uai_mastermind_04_08_04_0015 and uai_queen5_5_3_wcsp.

**Validation.** Every answer is compared with a reference (Setup). The
largest absolute differences: 4.9e-9 over 5 766 answers on 78 standard
instances (ProbLog prints eight digits), 2.7e-13 on the 39 literature
instances, 6.3e-15 over 25 967 answers on 46 of Ace's 46 networks,
4.9e-11 on the 14 large networks. g20_60 to g30_100 and grid20 have no
exact reference; `scripts/sample_paths.py` estimates their answers from
a million sampled graphs (200 000 for the grids), and each of the 76
answers lies within 2.7 standard errors of its estimate (checked with an
earlier version's answers, from which this run's differ by at most
3.5e-14).

The other network tools' answers against the same references: pyAgrum
214 marginal runs to 2.0e-7 and 230 partition functions to 2.1e-7 in
log10; pgmpy 129 marginal runs to 4.7e-9 and 214 partition functions to
2.2e-7 in log10; libDAI 224 marginal runs to 5.0e-7 and 256 partition
functions to 2.2e-7 in log10; TensorInference.jl 341 marginal runs to
5.0e-7 and 380 partition functions to 2.2e-7 in log10 (the tools print
six digits, as Merlin does).

In the tables, ratio is the fastest exact baseline's time over tddlog's,
Merlin, pyAgrum, pgmpy, libDAI and TensorInference.jl among them on the
networks; `>300` is a timeout; `-` means no exact baseline finished;
`skip` is a run that the baseline job left out after a timeout on a
smaller instance of the family. Scallop's top-k proofs
(`scallop:topk10`) are approximate and listed for their time only.

### Standard suite (84 instances, 300 s)

| instance | tddlog 0.1.0 | fastest exact baseline | ratio | scallop:topk10 | merlin:bte | agrum:jtree | pgmpy:ve | libdai:jtree | tensorinference:treesa | tensorinference:greedy |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
| uwcse2 | 0.07 | 0.15 (problog:sdd) | 0.8 | 0.05 | - | - | - | - | - | - |
| dice_twocoins | 0.00 | 0.07 (problog:sdd) | 0.4 | 0.01 | - | - | - | - | - | - |
| s6 | 0.00 | 0.09 (problog:sdd) | 0.4 | 0.01 | - | - | - | - | - | - |
| dice_evidence1 | 0.00 | 0.07 (problog:fsdd) | 0.4 | 0.01 | - | - | - | - | - | - |
| bloodtype | 0.01 | 0.27 (problog:ddnnf) | 1.4 | >300 | - | - | - | - | - | - |
| grid4 | 0.01 | 0.09 (problog:sdd) | 0.4 | 0.94 | - | - | - | - | - | - |
| dice_evidence2 | 0.00 | 0.07 (problog:sdd) | 0.4 | 0.01 | - | - | - | - | - | - |
| dice_grass | 0.01 | 0.08 (problog:sdd) | 0.4 | 0.01 | - | - | - | - | - | - |
| dice_alarm | 0.00 | 0.07 (problog:fsdd) | 0.4 | 0.01 | - | - | - | - | - | - |
| dice_noisyOr | 0.01 | 0.08 (problog:fsdd) | 0.4 | 0.02 | - | - | - | - | - | - |
| s10 | 0.01 | 0.11 (problog:sdd) | 0.5 | 0.04 | - | - | - | - | - | - |
| dice_murderMystery | 0.00 | 0.07 (problog:sdd) | 0.4 | 0.01 | - | - | - | - | - | - |
| g10_25 | 0.01 | 0.14 (problog:sdd) | 0.7 | 0.09 | - | - | - | - | - | - |
| uwcse3 | 0.01 | 0.20 (problog:sdd) | 1.0 | 0.07 | - | - | - | - | - | - |
| uwcse4 | 0.01 | 0.28 (problog:sdd) | 1.4 | 0.24 | - | - | - | - | - | - |
| dice_digitRecognition | 0.26 | 7.36 (problog:sdd) | 28.8 | err | - | - | - | - | - | - |
| g11_29 | 0.01 | 0.17 (problog:sdd) | 0.9 | 0.09 | - | - | - | - | - | - |
| cora10 | 0.02 | 0.68 (pita:bdd) | 3.4 | 0.06 | - | - | - | - | - | - |
| grid5 | 0.01 | 0.11 (problog:sdd) | 0.5 | 0.24 | - | - | - | - | - | - |
| s15 | 0.01 | 0.12 (problog:sdd) | 0.6 | 0.01 | - | - | - | - | - | - |
| grid6 | 0.01 | 0.18 (problog:sdd) | 0.9 | 0.10 | - | - | - | - | - | - |
| s20 | 0.01 | 0.13 (problog:fsdd) | 0.7 | 0.02 | - | - | - | - | - | - |
| cora50 | 0.03 | 0.81 (pita:bdd) | 4.1 | 0.24 | - | - | - | - | - | - |
| s50 | 0.01 | 0.21 (problog:sdd) | 1.0 | 0.03 | - | - | - | - | - | - |
| s100 | 0.01 | 0.33 (problog:sdd) | 1.7 | 0.04 | - | - | - | - | - | - |
| s200 | 0.02 | 0.53 (pita:bdd) | 2.7 | 0.07 | - | - | - | - | - | - |
| grid7 | 0.01 | 0.51 (pita:bdd) | 2.5 | 1.44 | - | - | - | - | - | - |
| cora200 | 0.06 | 1.44 (pita:bdd) | 7.2 | 2.10 | - | - | - | - | - | - |
| cora500 | 0.10 | 2.65 (pita:bdd) | 13.2 | 10.38 | - | - | - | - | - | - |
| uwcse5 | 0.01 | 0.35 (problog:sdd) | 1.8 | 0.78 | - | - | - | - | - | - |
| uwcse6 | 0.01 | 0.64 (problog:sdd) | 3.2 | 0.26 | - | - | - | - | - | - |
| uwcse8 | 0.01 | 2.81 (problog:sdd) | 14.1 | 0.33 | - | - | - | - | - | - |
| g12_32 | 0.01 | 0.19 (problog:sdd) | 1.0 | 0.35 | - | - | - | - | - | - |
| uwcse10 | 0.01 | 7.04 (pita:bdd) | 35.2 | 0.24 | - | - | - | - | - | - |
| grid8 | 0.01 | 0.67 (pita:bdd) | 3.4 | 2.17 | - | - | - | - | - | - |
| g13_35 | 0.01 | 0.24 (problog:sdd) | 1.2 | 0.32 | - | - | - | - | - | - |
| g14_37 | 0.01 | 0.22 (problog:sdd) | 1.1 | 0.15 | - | - | - | - | - | - |
| g15_40 | 0.03 | 0.71 (problog:sdd) | 3.6 | 0.16 | - | - | - | - | - | - |
| grid10 | 0.02 | - |  | 22.71 | - | - | - | - | - | - |
| g16_44 | 0.01 | 0.30 (problog:sdd) | 1.5 | 0.36 | - | - | - | - | - | - |
| g18_50 | 0.03 | 0.73 (problog:sdd) | 3.7 | 0.47 | - | - | - | - | - | - |
| g20_56 | 0.20 | 89.42 (problog:sdd) | 447.1 | 0.61 | - | - | - | - | - | - |
| grid12 | 0.03 | - |  | 123.10 | - | - | - | - | - | - |
| g20_60 | 1.37 | - |  | 2.15 | - | - | - | - | - | - |
| grid15 | 0.14 | - |  | 5.39 | - | - | - | - | - | - |
| g25_80 | 4.79 | - |  | 4.99 | - | - | - | - | - | - |
| g30_100 | 122.40 | - |  | 15.05 | - | - | - | - | - | - |
| grid20 | 2.80 | - |  | >300 | - | - | - | - | - | - |
| grid25 | >300 | - |  | skip | - | - | - | - | - | - |
| grid30 | skip | - |  | skip | - | - | - | - | - | - |
| bn_cancer_ev | 0.00 | 0.00 (libdai:jtree) | 0.0 | 0.03 | 0.00 | 0.10 | 1.34 | 0.00 | 15.32 | 13.78 |
| bn_cancer | 0.00 | 0.00 (libdai:jtree) | 0.0 | 0.02 | 0.00 | 0.10 | 1.30 | 0.00 | 15.32 | 13.59 |
| bn_earthquake_ev | 0.00 | 0.00 (merlin:bte) | 0.0 | 0.01 | 0.00 | 0.10 | 1.29 | 0.00 | 15.57 | 13.78 |
| bn_earthquake | 0.00 | 0.00 (merlin:bte) | 0.0 | 0.02 | 0.00 | 0.11 | 1.31 | 0.00 | 15.18 | 13.60 |
| bn_asia_ev | 0.00 | 0.00 (merlin:bte) | 0.0 | 0.08 | 0.00 | 0.10 | 1.31 | 0.00 | 18.26 | 16.29 |
| bn_asia | 0.01 | 0.00 (merlin:bte) | 0.0 | 0.06 | 0.00 | 0.10 | 1.31 | 0.00 | 17.78 | 16.03 |
| bn_survey_ev | 0.00 | 0.00 (merlin:bte) | 0.0 | 0.09 | 0.00 | 0.10 | 1.32 | 0.00 | 17.74 | 16.03 |
| bn_survey | 0.00 | 0.00 (merlin:bte) | 0.0 | 0.13 | 0.00 | 0.10 | 1.30 | 0.00 | 17.24 | 15.83 |
| bn_sachs_ev | 0.01 | 0.00 (libdai:jtree) | 0.0 | >300 | 0.00 | 0.11 | 1.32 | 0.00 | 19.66 | 17.65 |
| bn_sachs | 0.01 | 0.00 (merlin:bte) | 0.0 | 189.03 | 0.00 | 0.11 | 1.27 | 0.00 | 19.17 | 17.03 |
| bn_child | 0.01 | 0.01 (libdai:jtree) | 0.0 | >300 | 0.01 | 0.10 | 1.28 | 0.01 | 20.00 | 17.68 |
| bn_child_ev | 0.01 | 0.00 (libdai:jtree) | 0.0 | skip | 0.01 | 0.11 | 1.29 | 0.00 | 19.77 | 17.06 |
| bn_insurance_ev | 0.02 | 0.01 (libdai:jtree) | 0.0 | skip | 0.01 | 0.12 | 1.28 | 0.01 | 26.68 | 23.52 |
| bn_insurance | 0.02 | 0.01 (merlin:bte) | 0.0 | skip | 0.01 | 0.11 | 1.29 | 0.01 | 26.27 | 23.15 |
| bn_water_ev | 0.04 | 0.24 (agrum:jtree) | 1.2 | skip | 0.36 | 0.24 | 1.38 | 0.32 | 34.88 | 29.66 |
| bn_alarm_ev | 0.01 | 0.01 (libdai:jtree) | 0.0 | skip | 0.01 | 0.12 | 1.36 | 0.01 | 25.49 | 21.17 |
| bn_hailfinder_ev | 0.04 | 0.01 (libdai:jtree) | 0.0 | skip | 0.01 | 0.12 | 1.37 | 0.01 | 27.93 | 22.55 |
| bn_hepar2_ev | 0.04 | 0.01 (libdai:jtree) | 0.0 | skip | 0.02 | 0.13 | 1.47 | 0.01 | 35.04 | 27.35 |
| bn_win95pts_ev | 0.01 | 0.01 (libdai:jtree) | 0.0 | skip | 0.01 | 0.12 | 1.38 | 0.01 | 37.74 | 29.35 |
| bn_pathfinder_ev | 0.30 | 0.07 (merlin:bte) | 0.2 | skip | 0.07 | 0.17 | 1.99 | 0.07 | 35.51 | 26.26 |
| bn_andes_ev | 0.04 | 0.07 (libdai:jtree) | 0.3 | skip | 0.09 | 0.14 | 1.88 | 0.07 | 74.70 | 48.91 |
| bn_pigs_ev | 0.07 | 0.14 (merlin:bte) | 0.7 | skip | 0.14 | 0.19 | 37.13 | 0.14 | 86.57 | 49.17 |
| bn_water | 0.03 | 0.31 (agrum:jtree) | 1.5 | skip | 0.34 | 0.31 | 1.48 | 0.32 | 33.41 | 28.02 |
| bn_alarm | 0.01 | 0.01 (merlin:bte) | 0.0 | skip | 0.01 | 0.11 | 1.33 | 0.01 | 24.41 | 19.99 |
| bn_hailfinder | 0.05 | 0.01 (libdai:jtree) | 0.1 | skip | 0.01 | 0.12 | 1.32 | 0.01 | 26.30 | 21.13 |
| bn_hepar2 | 0.04 | 0.01 (libdai:jtree) | 0.1 | skip | 0.02 | 0.12 | 1.56 | 0.01 | 33.62 | 26.26 |
| bn_link_ev | 0.30 | 3.62 (libdai:jtree) | 12.2 | skip | 29.06 | 45.61 | >300 | 3.62 | 150.24 | >300 |
| bn_win95pts | 0.01 | 0.01 (libdai:jtree) | 0.0 | skip | 0.01 | 0.12 | 1.41 | 0.01 | 37.58 | 29.23 |
| bn_pathfinder | 0.31 | 0.08 (libdai:jtree) | 0.2 | skip | 0.08 | 0.20 | 3.45 | 0.08 | 34.41 | 25.70 |
| bn_andes | 0.04 | 0.08 (libdai:jtree) | 0.4 | skip | 0.08 | 0.14 | 2.86 | 0.08 | 72.82 | 46.95 |
| bn_pigs | 0.08 | 0.14 (merlin:bte) | 0.7 | skip | 0.14 | 0.19 | 28.61 | 0.15 | 84.40 | 48.40 |
| bn_link | 0.85 | 4.07 (libdai:jtree) | 4.8 | skip | 29.55 | 47.89 | >300 | 4.07 | 147.01 | >300 |
| bn_munin_ev | 0.78 | 0.89 (agrum:jtree) | 1.1 | skip | 2.20 | 0.89 | skip | 3.33 | 172.04 | skip |
| bn_munin | 0.80 | 0.90 (agrum:jtree) | 1.1 | skip | 2.19 | 0.90 | skip | 3.35 | 163.82 | skip |

### Literature suite (39 instances, 300 s)

| instance | tddlog 0.1.0 | fastest exact baseline | ratio | scallop:topk10 |
|---|---:|---|---:|---:|
| noisyor6_9 | 0.00 | 0.10 (problog:sdd) | 0.5 | 0.06 |
| pdb10 | 0.01 | 0.10 (problog:sdd) | 0.5 | 0.15 |
| mnist_sum4 | 0.01 | 1.85 (pita:bdd) | 9.2 | >300 |
| mastermind4_4_8 | 0.02 | 0.43 (problog:sdd) | 2.1 | 109.03 |
| pdb15 | 0.01 | 0.16 (problog:sdd) | 0.8 | 0.23 |
| mastermind6_4_8 | 0.04 | 0.61 (problog:sdd) | 3.0 | 161.58 |
| sickness6_10 | 0.01 | 0.59 (pita:bdd) | 3.0 | 19.63 |
| pedigree4_4 | 0.03 | - |  | 143.64 |
| noisyor8_12 | 0.01 | 0.15 (problog:sdd) | 0.7 | 0.42 |
| noisyor10_15 | 0.01 | 0.21 (problog:ddnnf) | 1.1 | 8.94 |
| pdb20 | 0.01 | 0.13 (problog:sdd) | 0.7 | 0.16 |
| mastermind8_4_8 | 0.04 | 0.75 (problog:sdd) | 3.7 | 215.68 |
| noisyor15_22 | 0.01 | 0.88 (problog:ddnnf) | 4.4 | 225.58 |
| pdb30 | 0.01 | 0.35 (problog:sdd) | 1.7 | 0.51 |
| pdb40 | 0.01 | 2.99 (aspmc:sharpsat-td) | 15.0 | 0.40 |
| mnist_sum8 | 0.03 | 33.26 (pita:bdd) | 166.3 | skip |
| mastermind10_4_8 | 0.05 | 0.92 (problog:sdd) | 4.6 | 267.93 |
| noisyor20_30 | 0.01 | 7.88 (problog:ddnnf) | 39.4 | >300 |
| sickness10_10 | 0.01 | 2.30 (pita:bdd) | 11.5 | 273.82 |
| noisyor30_45 | 0.01 | 55.65 (pita:bdd) | 278.2 | skip |
| mnist_sum16 | 0.08 | - |  | skip |
| pedigree4_6 | 0.04 | - |  | >300 |
| sickness14_10 | 0.01 | 5.87 (pita:bdd) | 29.4 | >300 |
| sickness18_20 | 0.02 | 204.43 (pita:bdd) | 1022.1 | skip |
| mastermind4_5_8 | 0.04 | 0.60 (problog:sdd) | 3.0 | 128.15 |
| mnist_sum32 | 0.25 | - |  | skip |
| mnist_sum64 | 0.83 | - |  | skip |
| mnist_sum96 | 1.82 | - |  | skip |
| pedigree6_6 | 0.06 | - |  | skip |
| pedigree10_6 | 0.12 | - |  | skip |
| pedigree5_10 | 0.47 | - |  | skip |
| pedigree6_8 | 0.09 | - |  | skip |
| pedigree16_6 | 0.28 | - |  | skip |
| pedigree32_6 | 1.13 | - |  | skip |
| pedigree6_10 | 2.91 | - |  | skip |
| sickness22_40 | 0.04 | - |  | skip |
| pedigree7_10 | 2.12 | - |  | skip |
| mastermind6_5_8 | 0.04 | 1.11 (problog:sdd) | 5.6 | 187.30 |
| mastermind8_5_8 | 0.06 | 1.84 (problog:sdd) | 9.2 | 250.94 |

### Ace's suite (46 instances, 600 s)

| instance | tddlog 0.1.0 | fastest exact baseline | ratio | merlin:bte | agrum:jtree | pgmpy:ve | libdai:jtree | tensorinference:treesa | tensorinference:greedy |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| ace_water | 0.01 | 0.02 (libdai:jtree) | 0.1 | 0.17 | 0.13 | 1.48 | 0.02 | 37.07 | 32.33 |
| ace_fs_01 | 0.00 | 0.00 (libdai:jtree) | 0.0 | 0.09 | 0.14 | 1.46 | 0.00 | 18.81 | 16.68 |
| ace_mastermind_03_08_03 | 0.02 | 0.34 (merlin:bte) | 1.7 | 0.34 | 0.50 | >600 | 0.97 | 283.35 | >600 |
| ace_blockmap_05_01 | 0.01 | 0.80 (merlin:bte) | 4.0 | 0.80 | 1.95 | err | 1.20 | 205.85 | err |
| ace_fs_04 | 0.01 | 0.01 (libdai:jtree) | 0.1 | 0.02 | 0.17 | 1.85 | 0.01 | 69.61 | 47.52 |
| ace_mildew | 0.12 | 0.22 (agrum:jtree) | 1.1 | 1.58 | 0.22 | 1.54 | 0.44 | 25.76 | 20.37 |
| ace_munin1 | 0.09 | 0.16 (merlin:bte) | 0.8 | 0.16 | 0.18 | 12.62 | 0.17 | 65.83 | err |
| ace_fs_07 | 0.03 | 2.40 (ace:auto) | 12.0 | err | err | >600 | err | err | >600 |
| ace_alarm | 0.01 | 0.01 (libdai:jtree) | 0.0 | 0.01 | 0.12 | 1.40 | 0.01 | 26.47 | 22.20 |
| ace_hailfinder | 0.03 | 0.01 (libdai:jtree) | 0.0 | 0.01 | 0.12 | 1.43 | 0.01 | 29.02 | 23.80 |
| ace_munin2 | 0.12 | 0.08 (merlin:bte) | 0.4 | 0.08 | 0.29 | 7.41 | 0.09 | 183.28 | skip |
| ace_pathfinder | 0.18 | 0.06 (libdai:jtree) | 0.3 | 0.06 | 0.23 | 1.81 | 0.06 | 38.19 | 28.78 |
| ace_students_03_02 | 0.03 | 0.04 (merlin:bte) | 0.2 | 0.04 | 0.17 | 1.55 | 0.06 | 126.84 | 64.16 |
| ace_munin3 | 0.31 | 0.20 (merlin:bte) | 0.6 | 0.20 | 0.38 | err | 0.38 | 175.74 | skip |
| ace_diabetes | 0.24 | 0.16 (libdai:jtree) | 0.7 | 0.17 | 0.24 | 1.52 | 0.16 | 79.17 | 45.88 |
| ace_pigs | 0.06 | 0.04 (merlin:bte) | 0.2 | 0.04 | 0.18 | 3.94 | 0.04 | 91.27 | 51.91 |
| ace_blockmap_05_02 | 0.01 | 0.05 (merlin:bte) | 0.2 | 0.05 | 0.24 | skip | 0.06 | 243.69 | skip |
| ace_munin4 | 0.77 | 0.94 (agrum:jtree) | 1.2 | 2.33 | 0.94 | skip | 3.49 | 180.72 | skip |
| ace_blockmap_05_03 | 0.03 | 0.07 (merlin:bte) | 0.4 | 0.07 | 0.29 | skip | 0.15 | 300.90 | skip |
| ace_blockmap_10_01 | 0.16 | 4.04 (ace:auto) | 20.2 | err | err | skip | err | >600 | skip |
| ace_mastermind_04_08_03 | 0.03 | 1.27 (agrum:jtree) | 6.3 | 1.33 | 1.27 | skip | 2.31 | 346.60 | skip |
| ace_fs_10 | 0.08 | 4.06 (ace:forceC2d) | 20.3 | skip | skip | skip | skip | skip | skip |
| ace_fs_13 | 0.14 | 7.70 (ace:forceC2d) | 38.5 | skip | skip | skip | skip | skip | skip |
| ace_mastermind_05_08_03 | 0.03 | 0.10 (merlin:bte) | 0.5 | 0.10 | 0.49 | skip | 0.25 | 409.88 | skip |
| ace_fs_16 | 0.25 | 12.86 (ace:forceC2d) | 52.0 | skip | skip | skip | skip | skip | skip |
| ace_fs_19 | 0.41 | 19.70 (ace:auto) | 48.1 | skip | skip | skip | skip | skip | skip |
| ace_fs_22 | 0.62 | 35.40 (ace:auto) | 57.1 | skip | skip | skip | skip | skip | skip |
| ace_fs_25 | 0.92 | 58.63 (ace:auto) | 63.7 | skip | skip | skip | skip | skip | skip |
| ace_fs_28 | 1.28 | 105.74 (ace:auto) | 82.6 | skip | skip | skip | skip | skip | skip |
| ace_mastermind_06_08_03 | 0.04 | 1.53 (agrum:jtree) | 7.7 | 2.45 | 1.53 | skip | 3.15 | 477.61 | skip |
| ace_blockmap_10_02 | 0.06 | 4.42 (ace:auto) | 22.1 | skip | skip | skip | skip | skip | skip |
| ace_blockmap_10_03 | 0.18 | 7.87 (ace:forceC2d) | 39.4 | skip | skip | skip | skip | skip | skip |
| ace_fs_29 | 1.47 | 125.26 (ace:auto) | 85.4 | skip | skip | skip | skip | skip | skip |
| ace_blockmap_15_01 | 0.10 | 11.48 (ace:forceC2d) | 57.4 | skip | skip | skip | skip | skip | skip |
| ace_blockmap_15_02 | 0.08 | 12.07 (ace:forceC2d) | 60.4 | skip | skip | skip | skip | skip | skip |
| ace_blockmap_15_03 | 0.38 | 19.42 (ace:forceC2d) | 51.7 | skip | skip | skip | skip | skip | skip |
| ace_blockmap_20_01 | 0.18 | 28.32 (ace:forceC2d) | 141.6 | skip | skip | skip | skip | skip | skip |
| ace_blockmap_20_02 | 0.20 | 31.18 (ace:forceC2d) | 154.3 | skip | skip | skip | skip | skip | skip |
| ace_blockmap_20_03 | 0.21 | 32.61 (ace:forceC2d) | 155.3 | skip | skip | skip | skip | skip | skip |
| ace_blockmap_22_01 | 0.26 | 44.21 (ace:forceC2d) | 170.0 | skip | skip | skip | skip | skip | skip |
| ace_blockmap_22_02 | 0.27 | 47.82 (ace:forceC2d) | 179.1 | skip | skip | skip | skip | skip | skip |
| ace_blockmap_22_03 | 0.28 | 49.45 (ace:forceC2d) | 175.4 | skip | skip | skip | skip | skip | skip |
| ace_mastermind_03_08_04 | 0.02 | 0.14 (merlin:bte) | 0.7 | 0.14 | 0.76 | skip | 0.35 | 529.57 | skip |
| ace_mastermind_04_08_04 | 0.05 | 0.39 (merlin:bte) | 2.0 | 0.39 | 1.18 | skip | 1.72 | >600 | skip |
| ace_mastermind_10_08_03 | 0.06 | 0.55 (merlin:bte) | 2.7 | 0.55 | 1.14 | skip | 1.01 | skip | skip |
| ace_mastermind_03_08_05 | 0.08 | 0.62 (merlin:bte) | 3.1 | 0.62 | 2.18 | skip | 3.00 | skip | skip |

### Large networks (14 instances, 600 s)

| instance | tddlog 0.1.0 | fastest exact baseline | ratio | merlin:bte | agrum:jtree | pgmpy:ve | libdai:jtree | tensorinference:treesa | tensorinference:greedy |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| bn_barley_ev | 1.51 | 0.52 (agrum:jtree) | 0.3 | 1.41 | 0.52 | 1.80 | 1.50 | 31.98 | 25.31 |
| bn_barley | 1.82 | 0.77 (agrum:jtree) | 0.4 | 1.57 | 0.77 | 6.63 | 1.85 | 31.68 | 24.78 |
| bn_diabetes_ev | 0.78 | 0.26 (agrum:jtree) | 0.3 | 0.86 | 0.26 | 1.49 | 0.94 | 74.82 | err |
| bn_diabetes | 0.78 | 0.28 (agrum:jtree) | 0.4 | 0.89 | 0.28 | 1.54 | 0.96 | 75.32 | err |
| bn_mildew_ev | 1.56 | 0.45 (libdai:jtree) | 0.3 | 1.47 | 0.45 | 1.59 | 0.45 | 24.13 | skip |
| bn_mildew | 1.64 | 0.31 (agrum:jtree) | 0.2 | 1.46 | 0.31 | 1.43 | 0.44 | 23.49 | skip |
| bn_munin1 | 0.94 | 11.01 (agrum:jtree) | 11.7 | 26.36 | 11.01 | err | 50.61 | 55.91 | skip |
| bn_munin1_ev | 0.84 | 12.28 (agrum:jtree) | 14.5 | 25.01 | 12.28 | >600 | 44.01 | 57.13 | skip |
| bn_munin2 | 0.38 | 0.36 (merlin:bte) | 0.9 | 0.36 | 0.44 | skip | 0.63 | 167.70 | skip |
| bn_munin3 | 0.46 | 0.43 (merlin:bte) | 1.0 | 0.43 | 0.45 | skip | 0.76 | 154.67 | skip |
| bn_munin4 | 0.89 | 0.98 (agrum:jtree) | 1.1 | 2.29 | 0.98 | skip | 3.16 | 169.50 | skip |
| bn_munin2_ev | 0.53 | 0.35 (merlin:bte) | 0.7 | 0.35 | 0.41 | skip | 0.58 | 169.64 | skip |
| bn_munin3_ev | 0.47 | 0.41 (agrum:jtree) | 0.9 | 0.41 | 0.41 | skip | 0.71 | 155.66 | skip |
| bn_munin4_ev | 0.82 | 0.83 (agrum:jtree) | 1.0 | 2.21 | 0.83 | skip | 3.20 | 168.46 | skip |

### Against each baseline

For each baseline, in total time as the README compares two systems: the
instances given to both, how many each answers, each one's total time
over those either answers (a run that does not answer counted at the
suite's time limit), the two totals over those both answer, and the
instances on which tddlog is slower (both under 0.2 s counted for
neither). TensorInference.jl's row is its faster contraction order on
each instance; its two orders follow. A system that ran in a later job
is compared with that job's tddlog, so tddlog's counts and totals differ
between such rows.

**Standard suite: networks**

| baseline | given | baseline answers | tddlog answers | baseline s | tddlog s | both answer | baseline s, both | tddlog s, both | tddlog slower |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Ace | 34 | 34 | 34 | 107.9 | 3.9 | 34 | 107.9 | 3.9 | 0 |
| Merlin | 34 | 34 | 34 | 64.5 | 3.9 | 34 | 64.5 | 3.9 | 2 |
| pyAgrum | 34 | 34 | 34 | 99.3 | 3.9 | 34 | 99.3 | 3.9 | 2 |
| pgmpy | 34 | 30 | 34 | 1 308 | 3.9 | 30 | 108.2 | 1.2 | 0 |
| libDAI | 34 | 34 | 34 | 15.7 | 3.9 | 34 | 15.7 | 3.9 | 2 |
| TensorInference.jl | 34 | 34 | 34 | 1 368 | 3.9 | 34 | 1 368 | 3.9 | 0 |
| TensorInference.jl treesa | 34 | 34 | 34 | 1 602 | 3.9 | 34 | 1 602 | 3.9 | 0 |
| TensorInference.jl greedy | 34 | 30 | 34 | 1 935 | 3.9 | 30 | 735.1 | 1.2 | 0 |
| ProbLog | 34 | 12 | 34 | 6 604 | 3.9 | 12 | 3.6 | 0.1 | 0 |
| PITA | 34 | 12 | 34 | 6 607 | 3.9 | 12 | 7.4 | 0.1 | 0 |
| aspmc | 34 | 18 | 34 | 5 764 | 4.5 | 18 | 964.5 | 0.3 | 0 |
| Dice | 34 | 17 | 34 | 5 273 | 4.5 | 17 | 173.4 | 0.2 | 0 |
| Alea.jl | 34 | 26 | 34 | 2 436 | 4.5 | 26 | 35.9 | 1.2 | 0 |
| Roulette | 34 | 21 | 34 | 3 988 | 4.5 | 21 | 88.0 | 0.7 | 0 |
| noDice | 34 | 18 | 34 | 5 080 | 4.5 | 18 | 279.5 | 0.3 | 0 |
| Psi | 34 | 8 | 34 | 7 828 | 4.5 | 8 | 27.6 | 0.0 | 0 |
| Genfer | 34 | 19 | 34 | 4 855 | 4.5 | 19 | 354.8 | 0.3 | 0 |
| SPPL | 34 | 12 | 34 | 6 788 | 4.5 | 12 | 188.2 | 0.1 | 0 |
| Geni | 34 | 18 | 34 | 4 918 | 4.5 | 18 | 117.7 | 0.3 | 0 |
| Storm | 17 | 12 | 17 | 1 655 | 1.3 | 12 | 155.2 | 0.2 | 0 |
| PRISM | 17 | 12 | 17 | 1 645 | 1.3 | 12 | 144.7 | 0.2 | 0 |

**Ace's networks**

| baseline | given | baseline answers | tddlog answers | baseline s | tddlog s | both answer | baseline s, both | tddlog s, both | tddlog slower |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Ace | 46 | 46 | 46 | 725.3 | 9.9 | 46 | 725.3 | 9.9 | 0 |
| Merlin | 46 | 25 | 46 | 12 612 | 9.9 | 25 | 11.8 | 2.4 | 2 |
| pyAgrum | 46 | 25 | 46 | 12 615 | 9.9 | 25 | 15.0 | 2.4 | 0 |
| pgmpy | 46 | 12 | 46 | 20 438 | 9.9 | 12 | 38.0 | 0.9 | 0 |
| libDAI | 46 | 25 | 46 | 12 619 | 9.9 | 25 | 19.1 | 2.4 | 1 |
| TensorInference.jl | 46 | 22 | 46 | 18 157 | 9.9 | 22 | 3 757 | 2.2 | 0 |
| TensorInference.jl treesa | 46 | 22 | 46 | 18 345 | 9.9 | 22 | 3 945 | 2.2 | 0 |
| TensorInference.jl greedy | 46 | 10 | 46 | 21 954 | 9.9 | 10 | 353.6 | 0.7 | 0 |
| ProbLog | 46 | 9 | 46 | 22 255 | 11.5 | 9 | 55.3 | 0.2 | 0 |
| PITA | 46 | 13 | 46 | 20 826 | 11.5 | 13 | 1 026 | 1.2 | 0 |
| aspmc | 46 | 7 | 46 | 23 864 | 11.4 | 7 | 464.2 | 0.1 | 0 |
| Dice | 46 | 2 | 46 | 26 597 | 11.5 | 2 | 197.4 | 0.2 | 0 |
| Alea.jl | 46 | 28 | 46 | 10 871 | 11.5 | 28 | 71.1 | 3.3 | 0 |
| Roulette | 46 | 28 | 46 | 10 958 | 11.5 | 28 | 158.2 | 3.2 | 0 |
| noDice | 46 | 8 | 46 | 23 310 | 11.5 | 8 | 509.7 | 0.3 | 0 |
| Psi | 46 | 1 | 46 | 27 050 | 11.5 | 1 | 50.2 | 0.0 | 0 |
| Genfer | 46 | 2 | 46 | 26 405 | 11.5 | 2 | 5.2 | 0.2 | 0 |
| SPPL | 46 | 18 | 46 | 20 309 | 11.5 | 18 | 3 509 | 1.0 | 0 |
| Geni | 46 | 3 | 46 | 25 890 | 11.5 | 3 | 89.6 | 0.0 | 0 |
| Storm | 46 | 6 | 46 | 24 319 | 9.0 | 6 | 319.5 | 0.1 | 0 |
| PRISM | 46 | 8 | 46 | 23 594 | 9.0 | 8 | 794.4 | 0.2 | 0 |

**Large networks**

| baseline | given | baseline answers | tddlog answers | baseline s | tddlog s | both answer | baseline s, both | tddlog s, both | tddlog slower |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Ace | 14 | 14 | 14 | 134.7 | 13.4 | 14 | 134.7 | 13.4 | 0 |
| Merlin | 14 | 14 | 14 | 65.1 | 13.4 | 14 | 65.1 | 13.4 | 8 |
| pyAgrum | 14 | 14 | 14 | 29.4 | 13.4 | 14 | 29.4 | 13.4 | 9 |
| pgmpy | 14 | 6 | 14 | 4 814 | 13.4 | 6 | 14.5 | 8.1 | 1 |
| libDAI | 14 | 14 | 14 | 109.8 | 13.4 | 14 | 109.8 | 13.4 | 3 |
| TensorInference.jl | 14 | 14 | 14 | 1 346 | 13.4 | 14 | 1 346 | 13.4 | 0 |
| TensorInference.jl treesa | 14 | 14 | 14 | 1 360 | 13.4 | 14 | 1 360 | 13.4 | 0 |
| TensorInference.jl greedy | 14 | 2 | 14 | 7 250 | 13.4 | 2 | 50.1 | 3.3 | 0 |
| ProbLog | 14 | 0 | 14 | 8 400 | 15.3 | 0 | 0.0 | 0.0 | 0 |
| PITA | 14 | 0 | 14 | 8 400 | 15.3 | 0 | 0.0 | 0.0 | 0 |
| aspmc | 14 | 0 | 14 | 8 400 | 15.3 | 0 | 0.0 | 0.0 | 0 |
| Dice | 14 | 0 | 14 | 8 400 | 15.3 | 0 | 0.0 | 0.0 | 0 |
| Alea.jl | 14 | 0 | 14 | 8 400 | 15.3 | 0 | 0.0 | 0.0 | 0 |
| Roulette | 14 | 0 | 14 | 8 400 | 15.3 | 0 | 0.0 | 0.0 | 0 |
| noDice | 14 | 0 | 14 | 8 400 | 15.3 | 0 | 0.0 | 0.0 | 0 |
| Psi | 14 | 0 | 14 | 8 400 | 15.3 | 0 | 0.0 | 0.0 | 0 |
| Genfer | 14 | 0 | 14 | 8 400 | 15.3 | 0 | 0.0 | 0.0 | 0 |
| SPPL | 14 | 0 | 14 | 8 400 | 15.3 | 0 | 0.0 | 0.0 | 0 |
| Geni | 14 | 0 | 14 | 8 400 | 15.3 | 0 | 0.0 | 0.0 | 0 |
| Storm | 14 | 0 | 14 | 8 400 | 9.8 | 0 | 0.0 | 0.0 | 0 |
| PRISM | 14 | 0 | 14 | 8 400 | 9.8 | 0 | 0.0 | 0.0 | 0 |

**Standard suite: programs**

| baseline | given | baseline answers | tddlog answers | baseline s | tddlog s | both answer | baseline s, both | tddlog s, both | tddlog slower |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| ProbLog | 50 | 41 | 48 | 2 297 | 132.6 | 41 | 197.1 | 1.0 | 0 |
| PITA | 50 | 40 | 48 | 2 564 | 132.6 | 40 | 164.1 | 0.8 | 0 |
| aspmc | 50 | 26 | 48 | 6 722 | 132.6 | 26 | 122.4 | 0.3 | 0 |
| Ace | 42 | 23 | 41 | 5 553 | 337.8 | 23 | 153.4 | 0.6 | 0 |
| Merlin | 42 | 21 | 41 | 6 016 | 337.8 | 21 | 15.8 | 0.6 | 0 |
| pyAgrum | 42 | 21 | 41 | 6 014 | 337.8 | 21 | 13.6 | 0.6 | 0 |
| pgmpy | 42 | 22 | 41 | 6 014 | 337.8 | 22 | 314.1 | 187.4 | 1 |
| libDAI | 42 | 21 | 41 | 6 013 | 337.8 | 21 | 12.7 | 0.6 | 0 |
| TensorInference.jl | 42 | 21 | 41 | 7 043 | 337.8 | 21 | 1 043 | 3.6 | 0 |
| TensorInference.jl treesa | 42 | 19 | 41 | 8 090 | 337.8 | 19 | 1 490 | 0.4 | 0 |
| TensorInference.jl greedy | 42 | 21 | 41 | 7 043 | 337.8 | 21 | 1 043 | 3.6 | 0 |
| Dice | 42 | 27 | 41 | 4 432 | 337.8 | 27 | 232.5 | 0.5 | 0 |
| Alea.jl | 42 | 34 | 41 | 2 207 | 337.8 | 34 | 107.0 | 2.4 | 0 |
| Roulette | 42 | 36 | 41 | 1 664 | 337.8 | 36 | 163.9 | 2.6 | 0 |
| noDice | 42 | 22 | 41 | 5 835 | 337.8 | 22 | 134.6 | 0.4 | 0 |
| Psi | 42 | 3 | 41 | 11 629 | 337.8 | 3 | 228.6 | 0.0 | 0 |
| Genfer | 42 | 13 | 41 | 8 643 | 337.8 | 13 | 242.8 | 0.3 | 0 |
| SPPL | 42 | 5 | 41 | 10 969 | 337.8 | 5 | 168.7 | 0.0 | 0 |
| Geni | 42 | 11 | 41 | 9 098 | 337.8 | 11 | 98.4 | 0.3 | 0 |

**Literature suite**

| baseline | given | baseline answers | tddlog answers | baseline s | tddlog s | both answer | baseline s, both | tddlog s, both | tddlog slower |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| ProbLog | 39 | 19 | 39 | 6 117 | 10.7 | 19 | 117.1 | 0.4 | 0 |
| PITA | 39 | 24 | 39 | 4 919 | 10.7 | 24 | 418.9 | 0.4 | 0 |
| aspmc | 39 | 14 | 39 | 7 854 | 12.4 | 14 | 353.8 | 0.2 | 0 |
| ProvSQL | 11 | 11 | 11 | 19.2 | 0.1 | 11 | 19.2 | 0.1 | 0 |
| Ace | 39 | 25 | 39 | 4 419 | 12.5 | 25 | 218.7 | 0.6 | 0 |
| Merlin | 39 | 14 | 39 | 7 690 | 12.5 | 14 | 190.3 | 0.2 | 0 |
| pyAgrum | 39 | 13 | 39 | 7 900 | 12.5 | 13 | 99.8 | 0.2 | 0 |
| pgmpy | 39 | 8 | 39 | 9 334 | 12.5 | 8 | 34.0 | 0.1 | 0 |
| libDAI | 39 | 15 | 39 | 7 514 | 12.5 | 15 | 314.1 | 0.2 | 0 |
| TensorInference.jl | 39 | 13 | 39 | 8 947 | 12.5 | 13 | 1 147 | 0.1 | 0 |
| TensorInference.jl treesa | 39 | 13 | 39 | 9 439 | 12.5 | 13 | 1 639 | 0.1 | 0 |
| TensorInference.jl greedy | 39 | 11 | 39 | 9 040 | 12.5 | 11 | 640.4 | 0.1 | 0 |
| Dice | 39 | 10 | 39 | 9 260 | 12.5 | 10 | 560.0 | 0.1 | 0 |
| Alea.jl | 39 | 19 | 39 | 6 045 | 12.5 | 19 | 45.1 | 0.8 | 0 |
| Roulette | 39 | 20 | 39 | 5 878 | 12.5 | 20 | 177.5 | 1.8 | 0 |
| noDice | 39 | 6 | 39 | 10 107 | 12.5 | 6 | 207.1 | 0.1 | 0 |
| Psi | 39 | 0 | 39 | 11 700 | 12.5 | 0 | 0.0 | 0.0 | 0 |
| Genfer | 39 | 0 | 39 | 11 700 | 12.5 | 0 | 0.0 | 0.0 | 0 |
| SPPL | 39 | 5 | 39 | 10 640 | 12.5 | 5 | 440.5 | 0.0 | 0 |
| Geni | 39 | 0 | 39 | 11 700 | 12.5 | 0 | 0.0 | 0.0 | 0 |

## The literature suite

Six families from published evaluations, each grown on a random
structure from a seed (`--suite lit`, 39 instances). The generators are
written for this repository; the only copied material is cplint's blood
type rules (`data/cplint`, Artistic License 2.0).

- **mnist_sum** `mnist_sum<n>` (`gen_mnist_sum.py`): DeepProbLog's
  addition task (Manhaeve et al., NeurIPS 2018) grown to n digits as in
  Scallop (Li, Huang, Naik, PLDI 2023): each digit a ten-valued annotated
  disjunction, the sum built by partial sums, all 9n+1 values queried.
- **pedigree** `pedigree<g>_<w>` (`gen_pedigree.py`): cplint's blood type
  rules (Meert, Struyf, Blockeel, ILP 2009; Riguzzi and Swift, TPLP 2011)
  on a loopy pedigree of g generations of w people, a third of the blood
  types before the last generation observed, the last generation's
  queried.
- **sickness** `sickness<n>_<t>` (`gen_sickness.py`): the dynamic domain of
  Vlasselaer et al. (IJAR 2016, Example 10): n people on a power-law
  friendship graph over t steps, cyclic within a step and chained across
  steps, a third of the earlier sick atoms observed, the last step
  queried.
- **noisyor** `noisyor<d>_<f>` (`gen_noisyor.py`): the BN2O networks of the
  UAI 2008 evaluation, after QMR-DT: d diseases, f findings with three
  noisy-or causes each and a leak, every finding observed, every disease
  queried.
- **mastermind** `mastermind<c>_<p>_<r>` (`gen_mastermind.py`): the game of
  Chavira, Darwiche, Jaeger (IJAR 2006) as the program of Vlasselaer et
  al. (IJAR 2016, Table 3): p pegs in c colours, the black and white pegs
  of r random guesses observed, every peg's colour queried.
- **pdb** `pdb<n>` (`gen_pdb.py`): the Boolean queries h0 and h1 (#P-hard)
  and a safe one of Dalvi and Suciu (VLDB J. 2007) over a
  tuple-independent database of n constants with two random S tuples
  each.

The reference, `scripts/lit_exact.py` (`exact:lit`), shares nothing with
tddlog: convolution (mnist_sum), clique-tree calibration (pedigree,
noisyor), forward filtering over the 2^n states of a step (sickness),
enumeration of the codes the feedback admits (mastermind), variable
elimination of a pairwise model (pdb). It agrees with ProbLog on a small
instance of each family (`tests/lit/`, `tests/test_lit.py`), on which
`tests/examples.rs` checks tddlog, and bounds the instance sizes (the
pedigree clique tree does not fit in 30 GB from pedigree8_8 on).
ProbLog 2.3.0's `-k ddnnf` answers `tests/lit/pedigree.pl` wrongly (by
0.078) when asked all twelve queries at once and right when asked one at
a time, so every check against ProbLog asks one query at a time.

Considered and left out: aspmc's growing-head and growing-negated-body
programs (trivial answers), Latin squares and sudoku (sizes 3 to 5 only),
the caesar cipher and HMMs (the shapes of `dice_digitRecognition` and
mnist_sum), CLUTRR kinship (an invented composition table), multi-digit
addition (grounding grows as 10^n).

## The large networks

The seven large networks of the flip-hoisting evaluation (Cheng et al.,
PROBPROG 2021: barley, diabetes, mildew, munin1 to munin4, with and
without evidence; `--suite large`). No ProbLog compiler finishes them, so
the reference is the clique tree (`scripts/bn_exact.py`). Dice with flip
hoisting did not finish barley, diabetes or mildew in 20 minutes.

## Ace's suite

The 46 networks of Ace's benchmarks (Chavira and Darwiche) that
`gen_bn.py` reads, each with one of the archive's evidence sets
(`--suite ace`, Setup). The archive is licensed for non-commercial
research use and is not in the tree.

## The UAI suite

The 120 instances of the UAI 2022 inference competition's MAR tuning set
(`--suite uai`; MAR.zip of
<https://www.ics.uci.edu/~dechter/uaicompetition/2022/TuningBenchmarks/>,
passed as `--uai-benchmarks`): Markov networks (Alchemy, CSP, DBN, Grids,
ObjectDetection, Segmentation) and Bayesian networks (Pedigree,
Promedus), each with one evidence file. `gen_bn.py` writes a Markov
network's factors as observed variables, so the program asks for the
marginals given the evidence. The baselines are Merlin (`--merlin`,
docs/baselines.md section 8) and pyAgrum, pgmpy, libDAI and
TensorInference.jl (`--agrum`, `--pgmpy`, `--libdai`, `--tensorinference`,
section 20), which read the UAI file; Ace (`--ace`, section 7), on the
network `gen_bn.py` makes of the file written as a Hugin file; and
ProbLog, PITA and aspmc (sections 1, 2 and 6) on the program tddlog
reads. The answers are checked against the clique tree
(`scripts/bn_exact.py`) and Merlin. The tuning set's PR.zip (121)
and the competition's final sets, MAR.zip (88) and PR.zip (197) of
<https://www.ics.uci.edu/~dechter/uaicompetition/2022/FinalBenchmarks/>,
read the same way; `uai_family` takes the family from the network's name
(mastermind, or_chain, blockmap, rus, protein side chains by PDB code,
weighted CSPs, BN, fs, pedigree). For the partition function (`run
--pr`), tddlog prints P(e) alone (`--evidence-probability --no-queries`)
and Merlin runs its PR task (`-t PR`), as do the other four tools on the
same files.

### Results: the marginals

The MAR task, in one job on one lab machine (16 physical cores and 125
GB), 2026-10-20 to 2026-10-21 (300 s a run). tddlog faster than Merlin
counts the instances where tddlog is faster than Merlin or answers
alone, not those both finish in under 0.2 s; seconds are each one's
total over the instances either answers, a run that does not answer
counted at the limit.

| family | instances | tddlog | Merlin | tddlog faster than Merlin | tddlog s | Merlin s |
|---|---:|---:|---:|---:|---:|---:|
| Alchemy | 1 | 1 | 0 | 1 | 0.2 | 300.0 |
| CSP | 3 | 3 | 3 | 2 | 18.5 | 63.8 |
| DBN | 6 | 6 | 6 | 6 | 12.5 | 136.9 |
| Grids | 8 | 8 | 2 | 7 | 17.6 | 1 804 |
| ObjectDetection | 65 | 65 | 65 | 65 | 249.1 | 3 157 |
| Pedigree | 3 | 3 | 3 | 3 | 0.2 | 5.8 |
| Promedus | 28 | 28 | 28 | 15 | 1.8 | 504.9 |
| Segmentation | 6 | 6 | 6 | 5 | 1.3 | 4.0 |
| all | 120 | 120 | 113 | 104 | 301.3 | 5 977 |

Merlin prints `-nan` where the partition function overflows a double
(Alchemy_11, Grids_13, 14, 17) and fails on Grids_15, 16 and 18. The two
both take under 0.2 s on 15 of Merlin's 113 answers; tddlog is faster on
97 of the other 98 (283.7 s against 3 877 over all 113): every DBN,
ObjectDetection and Pedigree network, CSP_11 and 13, Grids_11, 15 of the
15 Promedus and 5 of the 6 Segmentation; Merlin is faster on
Segmentation_12 (339 ms against 423). tddlog's marginals agree with the
reference's (Setup) to 9.1e-14 on 111 networks and with Merlin's six
printed digits on the other 5. Ace, Alea.jl, aspmc, Dice, Genfer, Geni,
noDice, PRISM, Psi, Roulette, SPPL and Storm also ran in a later job,
each instance beside that job's tddlog, which their times are compared
with. ProbLog's forward SDD compile (`fsdd`) stops with
`InconsistentEvidenceError` on 19 networks whose evidence is consistent
(CSP_12, DBN_11, 12, 13, Grids_11, 12, 13, 14, 15, 16, 17, 18,
Pedigree_13, Segmentation_11, 12, 13, 14, 15, 16; tddlog answers them).
The error comes from a logical check, the evidence's SDD found false,
not from a probability; the check also runs while the compile is still
building that SDD (`ForwardEvaluator.node_updated`), and it stops so on
smaller programs that ProbLog's other compiles answer (bn_sachs_ev,
mastermind4_4_8). These runs count as errors.

Against each baseline (columns as in "Against each baseline"):

| baseline | given | baseline answers | tddlog answers | baseline s | tddlog s | both answer | baseline s, both | tddlog s, both | tddlog slower |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Ace | 120 | 66 | 120 | 18 800 | 294.6 | 66 | 2 600 | 58.7 | 0 |
| Merlin | 120 | 113 | 120 | 5 977 | 301.3 | 113 | 3 877 | 283.7 | 1 |
| pyAgrum | 120 | 107 | 120 | 6 218 | 301.3 | 107 | 2 318 | 283.1 | 1 |
| pgmpy | 120 | 53 | 120 | 23 848 | 301.3 | 53 | 3 748 | 110.4 | 0 |
| libDAI | 120 | 117 | 120 | 6 645 | 301.3 | 117 | 5 745 | 286.4 | 1 |
| TensorInference.jl | 120 | 113 | 120 | 10 416 | 301.3 | 113 | 8 316 | 284.1 | 0 |
| TensorInference.jl treesa | 120 | 111 | 120 | 13 345 | 301.3 | 111 | 10 645 | 275.1 | 0 |
| TensorInference.jl greedy | 120 | 59 | 120 | 23 647 | 301.3 | 59 | 5 347 | 61.8 | 0 |
| ProbLog | 120 | 0 | 120 | 36 000 | 301.3 | 0 | 0.0 | 0.0 | 0 |
| PITA | 120 | 1 | 120 | 35 894 | 301.3 | 1 | 193.8 | 0.0 | 0 |
| aspmc | 120 | 5 | 120 | 35 079 | 364.3 | 5 | 578.9 | 0.4 | 0 |
| Dice | 120 | 0 | 120 | 36 000 | 364.3 | 0 | 0.0 | 0.0 | 0 |
| Alea.jl | 120 | 0 | 120 | 36 000 | 364.3 | 0 | 0.0 | 0.0 | 0 |
| Roulette | 120 | 0 | 120 | 36 000 | 364.3 | 0 | 0.0 | 0.0 | 0 |
| noDice | 120 | 0 | 120 | 36 000 | 364.3 | 0 | 0.0 | 0.0 | 0 |
| Psi | 120 | 0 | 120 | 36 000 | 364.3 | 0 | 0.0 | 0.0 | 0 |
| Genfer | 120 | 0 | 120 | 36 000 | 364.3 | 0 | 0.0 | 0.0 | 0 |
| SPPL | 120 | 1 | 120 | 35 792 | 364.3 | 1 | 91.8 | 0.0 | 0 |
| Geni | 120 | 0 | 120 | 36 000 | 364.3 | 0 | 0.0 | 0.0 | 0 |
| Storm | 120 | 1 | 120 | 35 750 | 256.5 | 1 | 50.2 | 0.0 | 0 |
| PRISM | 120 | 0 | 120 | 36 000 | 256.5 | 0 | 0.0 | 0.0 | 0 |

The PR task on the same 120 networks (`--pr`, one job on one lab machine
(16 physical cores and 251 GB), 2026-10-20 to 2026-10-21): tddlog's
log10 Z agrees with the reference's to 5.7e-14 on 107 instances and with
Merlin's to 2.2e-7 on 114. tddlog answers all 120 in 217.5 s, Merlin's
`-t PR` 114 in 1 247. Summed by family where both answer, Merlin is
slower on every family (ObjectDetection 984.5 against 180.7, Promedus
130.9 against 2.9). Ace, Alea.jl, aspmc, Dice, Genfer, Geni, noDice,
Psi, Roulette and SPPL also ran in a later job, each instance beside
that job's tddlog, which their times are compared with.

Against each baseline (columns as in "Against each baseline"):

| baseline | given | baseline answers | tddlog answers | baseline s | tddlog s | both answer | baseline s, both | tddlog s, both | tddlog slower |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Ace | 120 | 70 | 120 | 17 880 | 217.5 | 70 | 2 880 | 59.3 | 0 |
| Merlin | 120 | 114 | 120 | 3 047 | 217.5 | 114 | 1 247 | 207.3 | 9 |
| pyAgrum | 120 | 108 | 120 | 5 812 | 217.5 | 108 | 2 212 | 206.3 | 4 |
| pgmpy | 120 | 104 | 120 | 6 377 | 217.5 | 104 | 1 577 | 182.8 | 21 |
| libDAI | 120 | 120 | 120 | 4 564 | 217.5 | 120 | 4 564 | 217.5 | 3 |
| TensorInference.jl | 120 | 115 | 120 | 5 159 | 217.5 | 115 | 3 659 | 209.9 | 0 |
| TensorInference.jl treesa | 120 | 115 | 120 | 7 605 | 217.5 | 115 | 6 105 | 209.9 | 0 |
| TensorInference.jl greedy | 120 | 60 | 120 | 19 769 | 217.5 | 60 | 1 769 | 52.8 | 0 |
| ProbLog | 120 | 0 | 120 | 36 000 | 217.5 | 0 | 0.0 | 0.0 | 0 |
| PITA | 120 | 25 | 120 | 29 131 | 217.5 | 25 | 631.4 | 2.9 | 0 |
| aspmc | 120 | 18 | 120 | 31 189 | 328.9 | 18 | 588.5 | 5.8 | 0 |
| Dice | 120 | 0 | 120 | 36 000 | 311.3 | 0 | 0.0 | 0.0 | 0 |
| Alea.jl | 120 | 0 | 120 | 36 000 | 311.3 | 0 | 0.0 | 0.0 | 0 |
| Roulette | 120 | 0 | 120 | 36 000 | 311.3 | 0 | 0.0 | 0.0 | 0 |
| noDice | 120 | 0 | 120 | 36 000 | 311.3 | 0 | 0.0 | 0.0 | 0 |
| Psi | 120 | 0 | 120 | 36 000 | 311.3 | 0 | 0.0 | 0.0 | 0 |
| Genfer | 120 | 0 | 120 | 36 000 | 311.3 | 0 | 0.0 | 0.0 | 0 |
| SPPL | 120 | 3 | 120 | 35 348 | 311.3 | 3 | 247.7 | 0.0 | 0 |
| Geni | 120 | 0 | 120 | 36 000 | 311.3 | 0 | 0.0 | 0.0 | 0 |

### Results: the partition function

The UAI 2022 PR sets, final (197) and tuning (121): tddlog's
`--pr` run against Merlin's `-t PR`, 300 s a run, in the same job as
the current results above. tddlog fastest counts the instances where
tddlog is faster than Merlin or the only one to answer, not those both
finish in under 0.2 s; seconds are each one's total over the instances
either answers, a run that does not answer counted at the 300 s limit.

The final set:

| family | instances | tddlog | Merlin | tddlog fastest | tddlog alone | none | tddlog s | Merlin s |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| blockmap | 14 | 14 | 4 | 14 | 10 | 0 | 3.6 | 3 096 |
| BN | 5 | 5 | 1 | 5 | 4 | 0 | 0.9 | 1 251 |
| fs | 1 | 1 | 0 | 1 | 1 | 0 | 0.0 | 300.0 |
| mastermind | 67 | 67 | 19 | 67 | 48 | 0 | 83.0 | 14 744 |
| or_chain | 25 | 25 | 2 | 25 | 23 | 0 | 1.0 | 7 087 |
| pedigree | 1 | 1 | 0 | 1 | 1 | 0 | 73.1 | 300.0 |
| protein | 19 | 1 | 0 | 1 | 1 | 18 | 10.3 | 300.0 |
| rus | 60 | 30 | 0 | 30 | 30 | 30 | 2 504 | 9 000 |
| wcsp | 5 | 1 | 2 | 1 | 0 | 3 | 311.7 | 177.8 |
| all | 197 | 145 | 28 | 145 | 118 | 51 | 2 988 | 36 255 |

Ace, Alea.jl, aspmc, Dice, exact, Genfer, Geni, libDAI, Merlin, noDice,
pgmpy, PITA, ProbLog, Psi, pyAgrum, Roulette, SPPL and
TensorInference.jl also ran in a later job, each instance beside that
job's tddlog, which their times are compared with.

Against each baseline (the partition function; columns as in "Against
each baseline"):

| baseline | given | baseline answers | tddlog answers | baseline s | tddlog s | both answer | baseline s, both | tddlog s, both | tddlog slower |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Ace | 197 | 87 | 114 | 9 329 | 168.8 | 87 | 1 229 | 53.1 | 0 |
| Merlin | 197 | 28 | 145 | 36 255 | 2 985 | 27 | 785.3 | 14.3 | 0 |
| pyAgrum | 197 | 14 | 114 | 31 129 | 168.8 | 14 | 1 129 | 1.5 | 0 |
| pgmpy | 197 | 6 | 114 | 33 177 | 168.8 | 6 | 777.4 | 1.3 | 0 |
| libDAI | 197 | 15 | 114 | 30 309 | 168.8 | 15 | 609.1 | 1.7 | 0 |
| TensorInference.jl | 197 | 46 | 114 | 30 514 | 5 869 | 27 | 3 589 | 41.9 | 0 |
| TensorInference.jl treesa | 197 | 46 | 114 | 30 514 | 5 869 | 27 | 3 589 | 41.9 | 0 |
| TensorInference.jl greedy | 197 | 0 | 114 | 34 200 | 168.8 | 0 | 0.0 | 0.0 | 0 |
| ProbLog | 197 | 20 | 114 | 28 588 | 164.1 | 20 | 387.8 | 2.0 | 0 |
| PITA | 197 | 43 | 114 | 22 864 | 164.1 | 43 | 1 564 | 4.0 | 0 |
| aspmc | 197 | 73 | 114 | 16 124 | 185.0 | 73 | 3 824 | 76.6 | 0 |
| Dice | 197 | 1 | 114 | 33 900 | 164.1 | 1 | 0.1 | 0.0 | 0 |
| Alea.jl | 197 | 16 | 114 | 29 428 | 164.1 | 16 | 27.7 | 2.2 | 0 |
| Roulette | 197 | 14 | 114 | 30 041 | 164.1 | 14 | 41.2 | 1.9 | 0 |
| noDice | 197 | 1 | 114 | 33 900 | 164.1 | 1 | 0.0 | 0.0 | 0 |
| Psi | 197 | 1 | 114 | 33 900 | 164.1 | 1 | 0.0 | 0.0 | 0 |
| Genfer | 197 | 1 | 114 | 33 900 | 164.1 | 1 | 0.0 | 0.0 | 0 |
| SPPL | 197 | 1 | 114 | 33 901 | 164.1 | 1 | 0.7 | 0.0 | 0 |
| Geni | 197 | 0 | 114 | 34 200 | 164.1 | 0 | 0.0 | 0.0 | 0 |

Of tddlog's 52 misses, 19 are timeouts (17 of protein, 2 of rus) and 33
memory refusals (28 of rus, 4 of wcsp, 1 of protein); Merlin answers
bwt3ac_wcsp of them. Merlin answers 28, and tddlog is faster on each it
also answers. tddlog's log10 Z agrees within 2.2e-7 with another exact
answer wherever one exists: Merlin's, the reference's (Setup) or a
weighted model count of the network's CNF by Ganak or sharpSAT-TD (110
of its 145 answers); the other 35 (30 of rus, 3 of BN, 1 of pedigree, 1
of protein) tddlog alone answers.

The tuning set:

| family | instances | tddlog | Merlin | tddlog fastest | tddlog alone | none | tddlog s | Merlin s |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Alchemy | 1 | 1 | 0 | 1 | 1 | 0 | 0.2 | 300.0 |
| CSP | 3 | 3 | 3 | 2 | 0 | 0 | 5.6 | 17.7 |
| DBN | 6 | 6 | 6 | 6 | 0 | 0 | 7.7 | 43.9 |
| grid10x10.f | 1 | 1 | 1 | 0 | 0 | 0 | 0.1 | 0.0 |
| Grids | 8 | 8 | 3 | 7 | 5 | 0 | 12.2 | 1 574 |
| ObjectDetection | 65 | 65 | 65 | 56 | 0 | 0 | 346.7 | 1 204 |
| Pedigree | 3 | 3 | 3 | 2 | 0 | 0 | 0.1 | 1.5 |
| Promedus | 28 | 28 | 28 | 14 | 0 | 0 | 0.5 | 147.9 |
| Segmentation | 6 | 6 | 6 | 3 | 0 | 0 | 1.1 | 1.3 |
| all | 121 | 121 | 115 | 91 | 6 | 0 | 374.2 | 3 291 |

Ace, Alea.jl, aspmc, Dice, exact, Genfer, Geni, libDAI, Merlin, noDice,
pgmpy, PITA, ProbLog, Psi, pyAgrum, Roulette, SPPL and
TensorInference.jl also ran in a later job, each instance beside that
job's tddlog, which their times are compared with.

| baseline | given | baseline answers | tddlog answers | baseline s | tddlog s | both answer | baseline s, both | tddlog s, both | tddlog slower |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Ace | 121 | 67 | 121 | 19 535 | 285.5 | 67 | 3 335 | 49.8 | 0 |
| Merlin | 121 | 115 | 121 | 3 291 | 285.5 | 115 | 1 491 | 278.3 | 8 |
| pyAgrum | 121 | 108 | 121 | 6 543 | 285.5 | 108 | 2 643 | 276.0 | 0 |
| pgmpy | 121 | 104 | 121 | 6 371 | 285.5 | 104 | 1 271 | 246.9 | 20 |
| libDAI | 121 | 121 | 121 | 6 279 | 285.5 | 121 | 6 279 | 285.5 | 1 |
| TensorInference.jl | 121 | 115 | 121 | 6 575 | 285.5 | 115 | 4 775 | 279.7 | 0 |
| TensorInference.jl treesa | 121 | 115 | 121 | 10 453 | 285.5 | 115 | 8 653 | 279.7 | 0 |
| TensorInference.jl greedy | 121 | 61 | 121 | 20 326 | 285.5 | 61 | 2 326 | 65.0 | 0 |
| ProbLog | 121 | 0 | 121 | 36 300 | 209.9 | 0 | 0.0 | 0.0 | 0 |
| PITA | 121 | 25 | 121 | 29 475 | 209.9 | 25 | 674.9 | 0.4 | 0 |
| aspmc | 121 | 10 | 121 | 34 110 | 215.2 | 10 | 809.5 | 0.6 | 0 |
| Dice | 121 | 0 | 121 | 36 300 | 209.9 | 0 | 0.0 | 0.0 | 0 |
| Alea.jl | 121 | 0 | 121 | 36 300 | 209.9 | 0 | 0.0 | 0.0 | 0 |
| Roulette | 121 | 0 | 121 | 36 300 | 209.9 | 0 | 0.0 | 0.0 | 0 |
| noDice | 121 | 0 | 121 | 36 300 | 209.9 | 0 | 0.0 | 0.0 | 0 |
| Psi | 121 | 0 | 121 | 36 300 | 209.9 | 0 | 0.0 | 0.0 | 0 |
| Genfer | 121 | 0 | 121 | 36 300 | 209.9 | 0 | 0.0 | 0.0 | 0 |
| SPPL | 121 | 1 | 121 | 36 011 | 209.9 | 1 | 10.9 | 0.0 | 0 |
| Geni | 121 | 0 | 121 | 36 300 | 209.9 | 0 | 0.0 | 0.0 | 0 |

tddlog answers all 121 in 374.2 s, Merlin 115 in 1 491 s. Merlin is
faster on 10 of 95 instances that both answer in 0.2 s or more (9
ObjectDetection, 1 Segmentation); the two both take under 0.2 s on 20.
log10 Z agrees with the reference's to 5.7e-14 on 113 instances and with
Merlin's to 2.2e-7 on 115, and with a weighted model count by Ganak or
sharpSAT-TD to 7.1e-15 on 40. uai_grids_18, uai_grids_17 and
uai_grids_16 only tddlog answers.

## The TPC-H suite

The probabilistic-database workload of SPROUT (Olteanu, Huang, Koch, ICDE
2009) and dissociation (Gatterbauer and Suciu, VLDB J. 2017): TPC-H with
every tuple an independent probabilistic fact and the TPC-H queries as
Boolean queries that keep their joins and selections (`--suite tpch`,
208 instances). `scripts/gen_tpch.py` writes the data after the schema
and value distributions of the TPC-H specification (clause 4.2.3), since
TPC's dbgen may not be redistributed. Each query is a family,
`tpch_<query>_<1000 SF>`, at scale factors 0.001 to 0.1 from the
smallest at which its selections leave a tuple (q19 is left out); the
program holds every tuple of the relations the query reads. Each query
is also written through views (`--views`, `tpch_<query>v_<1000 SF>`),
over the same lineage; the grounder inlines a view over facts alone read
once or of one rule (design.md, Grounding, step 6).

q1 and q6 are a disjunction of lineitem tuples, q4, q12 and q14 to q17
hierarchical, q3, q10, q11 and q18 hierarchical under the keys: all
read-once on TPC-H data. q2 and q20 (supplier - partsupp - part, the
query h0, #P-hard) leave a sparse bipartite graph, q7 and q21 (supplier -
lineitem - orders) one that grows dense (618 and 6 181 clauses at SF
0.1, one component). Tuple probabilities are drawn from [0.05, 0.95],
those of lineitem and partsupp scaled by min(1, 100/n).

The reference, `scripts/tpch_exact.py` (`exact:tpch`), evaluates each
query's joins and selections in Python, query by query, into its lineage
and computes its probability by d-tree decomposition with variable
elimination on the rest. It answers every instance and agrees with
ProbLog on `tests/lit/tpch.pl` (`tests/test_tpch.py`, which also checks
the views' lineage) and on the smallest instance of each query ProbLog
answers. Every query reads facts alone, so `auto` tries `td` first.

ProvSQL 1.13.0 (`--provsql`; Senellart et al., a PostgreSQL 16.4
extension that tracks provenance and evaluates a result tuple's
probability by a method of its own choice) runs the same instances as SQL
(`scripts/tpch2provsql.py`): a table per fact predicate, a view per rule
predicate, the data loaded before the clock starts as a database holds its
data, and the seconds those of the queries through `psql`.

### Results

tddlog 0.1.0 `auto`, ProbLog's three compilations, PITA, aspmc and
Scallop (top 10 and top 1 000 proofs, approximate), ProvSQL 1.13.0 on
PostgreSQL 16.4 (its own choice of method; the seconds of its queries,
the data loaded before), in a second job, on a lab machine with 20
physical cores and 503 GB of memory, on 2026-10-19, in 18 lanes of 55 GB
of address space, 600 s a run.

tddlog answers 205 of the 208 (366.0 s in all), each within 1.0e-14 of
`exact:tpch`; it misses q7v, q21v and q21 at SF 0.1. Against the fastest
of ProbLog, PITA and aspmc on each instance, in total time: Of the 208
instances both are given, tddlog answers 205 and ProbLog, PITA or aspmc
172. Over the 205 that either answers, a run that does not answer
counted at the 600 s limit, tddlog takes 366.0 s in all and the fastest
of them 33 811 s; over the 172 both answer, 181.2 s against 14 011.
tddlog is slower on tpch_q11v_5 (1.92 s against ProbLog's 1.74). tddlog
alone answers tpch_q10v_5, tpch_q7v_50, tpch_q7_50, tpch_q10_5,
tpch_q11_50 and 28 more. Neither answers tpch_q7v_100, tpch_q21v_100 and
tpch_q21_100. By query, flat and views together (seconds in total time,
as in "Against each baseline": over the instances either answers, a run
that does not answer counted at 600 s; largest scale factor answered,
times 1 000):

| query | instances | tddlog | ProbLog, PITA or aspmc | tddlog s | ProbLog, PITA or aspmc s | largest SF, tddlog / others | ProvSQL | tddlog s | ProvSQL s |
|---|---:|---:|---:|---:|---:|---|---:|---:|---:|
| q1 | 14 | 14 | 12 | 34.0 | 2 305 | 100 / 50 | 14 | 34.0 | 37.3 |
| q2 | 12 | 12 | 12 | 1.8 | 68.5 | 100 / 100 | 12 | 1.8 | 2.2 |
| q3 | 14 | 14 | 14 | 18.9 | 1 349 | 100 / 100 | 14 | 18.9 | 6.4 |
| q4 | 14 | 14 | 14 | 23.3 | 1 466 | 100 / 100 | 14 | 23.3 | 7.6 |
| q6 | 14 | 14 | 13 | 17.5 | 2 139 | 100 / 100 | 14 | 17.5 | 1.5 |
| q7 | 12 | 11 | 8 | 62.0 | 2 009 | 100 / 20 | 8 | 62.0 | 1 806 |
| q10 | 14 | 14 | 4 | 21.8 | 6 043 | 100 / 2 | 14 | 21.8 | 11.4 |
| q11 | 10 | 10 | 6 | 7.7 | 2 451 | 100 / 20 | 7 | 7.7 | 1 801 |
| q12 | 14 | 14 | 10 | 28.2 | 3 123 | 100 / 50 | 14 | 28.2 | 2.4 |
| q14 | 14 | 14 | 10 | 17.6 | 3 522 | 100 / 100 | 14 | 17.6 | 2.6 |
| q15 | 14 | 14 | 14 | 18.7 | 1 959 | 100 / 100 | 14 | 18.7 | 6.0 |
| q16 | 14 | 14 | 14 | 6.1 | 700.5 | 100 / 100 | 14 | 6.1 | 2.1 |
| q17 | 10 | 10 | 9 | 17.7 | 1 847 | 100 / 100 | 10 | 17.7 | 1.4 |
| q18 | 14 | 14 | 13 | 50.6 | 2 278 | 100 / 100 | 14 | 50.6 | 273.5 |
| q20 | 12 | 12 | 12 | 2.7 | 95.4 | 100 / 100 | 12 | 2.7 | 0.3 |
| q21 | 12 | 10 | 7 | 37.3 | 2 457 | 50 / 20 | 7 | 37.3 | 1 804 |

ProvSQL runs 201 of the 208 instances and answers 196 (5 timeout
otherwise), each within 1.0e-12 of `exact:tpch` (196 compared). Against
ProvSQL in total time: Of the 208 instances both are given, tddlog
answers 205 and ProvSQL 196. Over the 205 that either answers, a run
that does not answer counted at the 600 s limit, tddlog takes 366.0 s in
all and ProvSQL 5 766 s; over the 196 both answer, 269.9 s against
366.4. tddlog is slower on tpch_q12v_100 (6.87 s against ProvSQL's
0.56), tpch_q12_100 (6.45 s against ProvSQL's 0.54), tpch_q17_100
(4.68 s against ProvSQL's 0.14), tpch_q4_100 (6.30 s against ProvSQL's
1.80), tpch_q14v_100 (4.84 s against ProvSQL's 0.35) and 112 more.
tddlog alone answers tpch_q11v_20, tpch_q7v_50, tpch_q7_50,
tpch_q11v_50, tpch_q11v_100 and 4 more. Neither answers tpch_q7v_100,
tpch_q21v_100 and tpch_q21_100.

Scallop's top-10 answers 167 instances, off by up to 0.92 (q1v at SF
0.002).

## The Biomine suite

ProbLog's first workload (De Raedt, Kimmig, Toivonen, IJCAI 2007) and the
Alzheimer domain of the Tp-compilation paper (Vlasselaer et al., IJAR
2016): the probability that a gene is connected to Alzheimer disease in
Biomine, a network of biological entities with probabilistic links
(Sevon et al., DILS 2006); real data, and an undirected cyclic path
program. `--suite biomine` reads the human Biomine graph of 2018
(`human_3_oct_2018.zip` of <https://biomine.ijs.si/downloads/>, academic
and non-commercial use, kept out of the tree; `--biomine-benchmarks`).
`scripts/gen_biomine.py` takes the union of the K most probable simple
paths between Alzheimer disease and each of four genes (APP, APBB2,
BCHE, PCSK2), by Yen's algorithm, and queries `path(disease, gene)` for
each. Ten instances `biomine<K>` (each loads the 647 MB graph, about
1.8 GB resident):

| K | 1 | 2 | 4 | 8 | 12 | 16 | 24 | 32 | 48 | 64 |
|---|---|---|---|---|---|---|---|---|---|---|
| nodes | 12 | 20 | 27 | 58 | 76 | 95 | 130 | 177 | 240 | 295 |
| edges | 11 | 23 | 37 | 83 | 115 | 148 | 211 | 290 | 414 | 526 |

The reference, `scripts/reliability_exact.py` (`exact:reliability`), is
the two-terminal reliability of the undirected graph: series, parallel
and degree-one reductions (Satyanarayana and Wood, 1985), then a dynamic
program over the partitions of the edge frontier (Hardy, Lucet and
Limnios, 2007). It agrees with enumeration on random graphs
(`tests/test_biomine.py`) and with ProbLog on
`tests/lit/biomine_toy.bmg` and biomine1, 2 and 4, and answers K = 64
but not 96 in 1 200 s, so the suite stops at 64. tddlog's largest
formula (`--stats`, `auto`, machine independent):

| K | 1 | 2 | 4 | 8 | 12 | 16 | 24 | 32 | 48 | 64 |
|---|---|---|---|---|---|---|---|---|---|---|
| largest formula (pairs) | 6 | 63 | 199 | 1 466 | 3 396 | 30 538 | 29 147 | 3 235 827 | 976 103 | 27.9 M |

### Results

The same job as the TPC-H results above, 600 s a run. Seconds, the
fastest of ProbLog's three compilations; Scallop's top-10 is approximate,
and its largest error is given:

| K | 1 | 2 | 4 | 8 | 12 | 16 | 24 | 32 | 48 | 64 |
|---|---|---|---|---|---|---|---|---|---|---|
| tddlog | 0.02 | 0.01 | 0.01 | 0.02 | 0.05 | 0.11 | 0.17 | 3.63 | 2.37 | 174.5 |
| ProbLog | 0.13 | 0.20 | 0.42 | 73.0 | >600 |  |  |  |  |  |
| PITA | 0.52 | 0.50 | 0.50 | 4.73 | >600 |  |  |  |  |  |
| aspmc | 2.78 | 7.88 | >600 |  |  |  |  |  |  |  |
| Scallop top-10 | 0.02 | 0.04 | 0.04 | 0.11 | 0.17 | 0.27 | 0.48 | 0.82 | 1.54 | 2.44 |
| Scallop's error | 6e-17 | 0.12 | 0.32 | 0.41 | 0.46 | 0.48 | 0.49 | 0.50 | 0.51 | 0.51 |

tddlog answers every K, within 3.4e-12 of the reference; ProbLog answers
to K = 8 (within 4.6e-9, its eight printed digits), PITA to K = 8
(within 3.3e-16) and aspmc to K = 2.

## Dice and Alea.jl

Dice (Holtzen, Van den Broeck and Millstein, OOPSLA 2020) compiles a
functional probabilistic program to a BDD in the order of its text
(baselines.md, section 4). It runs on the standard suite's `dice` and
`bn` families (a network as one Dice program per queried variable, the
encoding of Dice's `bngen`; Dice returns one variable's distribution,
and a tuple of all of them would print the joint) and on the loop
programs (`--suite indicators --families dice`), which use integers,
`iterate` and recursion. Alea.jl, its Julia successor, runs on the same
programs translated by `dice2alea.py` (baselines.md, section 5), a
network as one program that returns all its queried variables.

### Results: tddlog 0.1.0, Dice and Alea.jl

tddlog 0.1.0 `auto` (tididi e4d60911), linked statically, Dice 1.0 with
an unlimited stack and Alea.jl at revision cf5e5b8 (Julia 1.11.9, CUDD
3.0.0) and, beside them, Roulette in one job on a cloud machine (96
physical cores, 1488 GB) on 2026-10-19, in 94 lanes, each process
limited to 15 GB of address space. The four systems run on a program
back to back in one lane, in an order rotated by program and repetition
(`bench.py run --rotate`); three repetitions, 300 s each on the networks
and the paper's programs and 600 s on the loops. Seconds are the median
of the repetitions, a failed repetition counted as longer than any time;
`(k/3)` marks a program answered in k repetitions, `>T` a timeout and
`oom` an allocation that failed inside the process. Roulette (the
artifact's, 54d6185, on Racket 8.16) runs `dice2roulette.py`'s
translation of the Dice program, variant `rsdd`, compiled by `raco make`
before it runs, untimed, as the artifact's programs are. Its columns:
**Roulette**, the wall time of its process, which includes Racket's
start and loading; **in Roulette**, what it reports for building and
querying the program.

Alea has three columns:

- **Alea**: the wall time of the Julia process, as every other column is
  the wall time of its system's process. It includes Julia's start,
  `using Alea`, a warm-up program and parsing the script: 8.48 s in the
  median over the runs, from 7.99 to 18.6 s.
- **in-process**: what the script reports for building the program and
  running `pr`.
- **JIT**: Julia's compilation within the in-process time, mostly of
  Alea's methods for the program's types; the translated program's own
  functions are interpreted (baselines.md, section 5).

Answers. Answers were checked in a pass of their own, with ProbLog's
d-DNNF and, on the networks, the clique tree as references, then Alea's
where neither answers. tddlog agrees with the reference on every program
it answers; its largest relative differences are 4.0e-10 from the clique
tree (bn_sachs), 2.2e-8 from ProbLog, which prints eight digits
(dice_simpleCaesar_k300) and 1.8e-14 from Alea (dice_ladder_k1600). Alea
agrees with it on every program Alea finishes; its largest are 4.0e-10
from the clique tree (bn_sachs) and 2.2e-8 from ProbLog, which prints
eight digits (dice_simpleCaesar_k300). Alea computes probabilities in
log space and gives tiny posteriors: down to 2.7e-152 on
digitRecognition; tddlog counts an answer that small on the complement's
formula (design.md, Queries) and prints it in scientific notation, and
agrees with Alea's to a relative 7.5e-12. Dice prints 0 for every answer
on caesar-iter, digitRecognition and simpleCaesar (9 programs with the
scaled ones). Dice prints zeros wherever the evidence has probability
below 1e-6 (baselines.md, section 4). By tddlog's
`--evidence-probability` the evidence has probability 2.0e-904 on
caesar-iter, 9.6e-241 on digitRecognition, 4.0e-1215 on simpleCaesar,
2.0e-31 and 2.3e-302 on caesar-iter at 100 and 1000 steps and 1.6e-123
and 4.2e-366 on simpleCaesar at 100 and 300 steps (simpleCaesar at 1000
steps and caesar-iter at 3000 steps the same as at their own sizes);
caesar-iter, simpleCaesar and simpleCaesar at 300 steps lie below the
smallest normal double, 2.2e-308. Dice's marginals on bn_pathfinder and
bn_insurance are off by 4.5e-5 (relative 8.7e-3) and 7.4e-6 (relative
4.4e-4): Dice drops most values of a `discrete` below 1e-4
(baselines.md, section 4). ProbLog's d-DNNF fails on 9 of the 17
networks (alarm, andes, hailfinder, insurance, link, munin, pathfinder,
pigs and water); there the clique tree is the reference.

The networks (Dice's time is the sum of its programs, one per queried
variable; Alea answers every variable from one program):

| instance | tddlog | Dice | Alea | in-process | JIT | Roulette | in Roulette |
|---|---:|---:|---:|---:|---:|---:|---:|
| bn_alarm | 0.01 | 0.23 | 9.46 | 1.02 | 0.95 | >300 | >300 |
| bn_andes | 0.03 | 245.07 | oom | oom | oom | oom | oom |
| bn_child | 0.007 | 0.14 | 9.43 | 1.03 | 1.01 | 0.40 | 0.03 |
| bn_hailfinder | 0.03 | 2.48 | 11.55 | 2.89 | 2.08 | >300 | >300 |
| bn_hepar2 | 0.02 | 1.77 | 9.80 | 1.29 | 1.04 | 0.80 | 0.45 |
| bn_insurance | 0.02 | 1.17 (wrong) | 11.57 | 3.03 | 1.88 | 1.92 | 1.54 |
| bn_link | 0.73 | oom | oom | oom | oom | 39.91 | 38.86 |
| bn_munin | 0.56 | >300 | oom | oom | oom | >300 | >300 |
| bn_pathfinder | 0.24 | 119.45 (wrong) | 28.21 | 9.67 | 3.72 | 13.43 | 13.03 |
| bn_pigs | 0.05 | 26.22 | 11.41 | 2.16 | 1.89 | 40.15 | 39.80 |
| bn_sachs | 0.006 | 0.07 | 9.69 | 1.37 | 1.35 | 0.37 | 0.02 |
| bn_water | 0.02 | 125.70 | 11.30 | 1.68 | 1.51 | 0.52 | 0.18 |
| bn_win95pts | 0.01 | 0.52 | 9.82 | 1.16 | 1.14 | 0.40 | 0.04 |

On the four networks that every system answers in under a second (asia,
cancer, earthquake and survey) and the paper's seven other small
programs, tddlog takes 0.002 to 0.003 s, Dice 0.01 to 0.03 s and Alea
8.15 to 8.97 s, of which 0.04 to 0.61 s in the process, nearly all of it
JIT. tddlog is the fastest on every network (against Alea without its
compilation). Alea's time in the process is below Dice's on hepar2,
pathfinder, pigs and water, and its wall time on pathfinder, pigs and
water.

On digitRecognition tddlog takes 0.16 s, Dice 0.17 s (all zero) and Alea
10.63 s, 1.88 s in the process (1.49 s JIT).

The loop programs at their own sizes (bench.py's `indicators` suite) and
scaled by `dice_scale.py` (`zero`: Dice printed 0 for every answer, the
evidence below its cutoff of 1e-6):

| program | steps | tddlog | Dice | Alea | in-process | JIT | Roulette | in Roulette |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| diamond | 100 | 0.01 | 0.02 | 8.26 | 0.08 | 0.07 | 0.36 | 0.02 |
| diamond | 400 | 0.09 | 0.02 | 10.23 | 0.23 | 0.09 | 0.88 | 0.41 |
| diamond | 1 600 | 0.20 | 0.07 | 12.35 | 2.76 | 0.09 | 8.01 | 7.48 |
| diamond | 6 400 | 0.51 | 0.19 | oom | oom | oom | oom | oom |
| ladder | 100 | 0.03 | 0.02 | 9.28 | 1.11 | 1.09 | 0.38 | 0.04 |
| ladder | 400 | 0.18 | 0.03 | 11.72 | 1.71 | 1.29 | 1.15 | 0.68 |
| ladder | 1 600 | 0.41 | 0.10 | 18.12 | 8.10 | 1.32 | 12.53 | 11.92 |
| ladder | 6 400 | 1.24 | 0.41 | oom | oom | oom | oom | oom |
| caesar-iter | 100 | 0.02 | 0.03 (zero) | 11.25 | 1.19 | 1.17 | 0.51 | 0.05 |
| caesar-iter | 1 000 | 0.29 | 0.10 (zero) | 12.40 | 2.35 | 1.18 | 3.36 | 2.87 |
| caesar-iter | 3 000 | 0.82 | 0.23 (zero) | 19.88 | 11.62 | 1.02 | 23.64 (zero) | 22.83 |
| simpleCaesar | 100 | 0.21 | 0.13 (zero) | 11.78 | 1.77 | 1.46 | 1.07 | 0.60 |
| simpleCaesar | 300 | 0.84 | 0.32 (zero) | 13.99 | 3.96 | 1.45 | 5.99 (zero) | 5.46 |
| simpleCaesar | 1 000 | 5.42 | 0.83 (zero) | oom | oom | oom | oom | oom |
| pmc1 | 10 | 0.003 | >600 | 14.76 | 6.27 | 6.25 | 0.44 | 0.02 |
| pmc1 | 20 | 0.003 | >600 | 14.70 | 6.22 | 6.20 | 0.38 | 0.02 |
| pmc1 | 40 | 0.003 | >600 | 14.97 | 6.41 | 6.37 | 0.39 | 0.03 |
| pmc1 | 75 | 0.41 | >600 | 29.52 | 21.18 | 16.06 | 5.40 | 5.07 |
| pmc2 | 10 | 0.003 | 97.86 | 11.79 | 3.24 | 3.23 | 0.36 | 0.004 |
| pmc2 | 20 | 0.003 | >600 | 11.54 | 3.24 | 3.23 | 0.37 | 0.01 |
| pmc2 | 51 | 0.07 | >600 | 17.39 | 9.40 | 8.16 | 1.42 | 1.10 |

On diamond Dice is faster than tddlog from 400 steps on (0.19 s against
0.51 at 6 400 steps). Alea answers diamond up to 1 600 steps (oom at 6
400). On ladder Dice is faster than tddlog from 100 steps on (0.41 s
against 1.24 at 6 400 steps). Alea answers ladder up to 1 600 steps (oom
at 6 400). On pmc1 and pmc2 Dice fails on 6 of the 7 sizes; Alea answers
7, in 3.24 to 21.18 s in the process, and tddlog 7, in 0.003 to 0.41 s.

Roulette against the others on the Dice programs: the programs each
system answers right (an answer of the reference's, no zeros, none off
by more than 1e-6), and in total time, over the programs given to both
that either answers right, the system's seconds in all / tddlog's, a run
that does not answer right counted at the time limit (300 s, 600 s on
the loops; tddlog's and Dice's time is the process's, Alea's and
Roulette's their own clock, without Julia's and Racket's start):

| programs | n | tddlog | Dice | Alea | Roulette |
|---|---:|---:|---:|---:|---:|
| the paper's programs and the networks | 25 | 25 | 20; 1902 / 1.92 | 22; 930 / 1.92 | 21; 1295 / 1.92 |
| the loops | 6 | 6 | 2; 2400 / 6.76 | 5; 643 / 6.76 | 4; 1206 / 6.76 |
| the loops scaled | 21 | 21 | 9; 7299 / 11.8 | 18; 1893 / 11.8 | 16; 3031 / 11.8 |

Roulette answers 41 of the 52 instances it ran and fails on 5 where it
runs out of memory (andes, simpleCaesar, diamond_k6400, ladder_k6400 and
simpleCaesar_k1000); 3 where it runs out of time (alarm, hailfinder and
munin); 3 where it prints zeros (caesar-iter, caesar-iter_k3000 and
simpleCaesar_k300).

tddlog holds a Dice program's integers in bits, as Dice and Alea do
(`dice2problog.py`); an integer of n values takes about log2 n atoms.
The same binary ran on the translation that held them one-hot, an atom
per value, in the same lanes: on the 46 programs of noDice and data/dice
that both answer, one-hot takes 15.1 s in all against 21.8 s in bits,
and their answers agree to a relative 1.1e-13. Where either takes 0.1 s
or more and the two differ by a quarter or more:

| program | bits | one-hot | bits pairs | one-hot pairs |
|---|---:|---:|---:|---:|
| dice_caesar-iter | 0.82 | 0.21 | 77 126 | 84 024 |
| dice_caesar-iter_k1000 | 0.29 | 0.08 | 25 976 | 28 024 |
| dice_caesar-iter_k3000 | 0.89 | 0.23 | 77 126 | 84 024 |
| dice_pmc1 | 0.41 | 0.21 | 72 | 147 |
| dice_pmc1_k75 | 0.51 | 0.25 | 72 | 147 |
| dice_simpleCaesar | 5.42 | 0.19 | 59 033 | 161 074 |
| dice_simpleCaesar_k100 | 0.21 | 0.02 | 6 476 | 17 074 |
| dice_simpleCaesar_k1000 | 6.30 | 0.20 | 59 033 | 161 074 |
| dice_simpleCaesar_k300 | 0.84 | 0.07 | 17 733 | 49 074 |
| nodice_coupon2-10-obs | 0.07 | 0.20 | 549 920 | 1 004 262 |
| nodice_coupon2-11-obs | 0.14 | 0.35 | 1 104 865 | 2 823 851 |
| nodice_coupon2-12-obs | 0.19 | 1.91 | 3 542 710 | 7 219 794 |
| nodice_coupon2-13-obs | 0.41 | 2.20 | 10 183 550 | 18 090 785 |
| nodice_coupon2-14-obs | 0.81 | 4.47 | 21 118 853 | 12 014 254 |
| nodice_water | 0.78 | 0.56 | 1 098 264 | 1 106 853 |

The repetitions differ by more than half in eight places where one takes
a second or more, most on dice_pmc1_k75 (Roulette 5.01 to 8.48 s),
dice_noisyOr (Alea 8.18 to 13.7 s), dice_evidence1 (Alea 8.15 to 13.7 s)
and dice_murderMystery (Alea 8.15 to 13.7 s).

Considered with TPC-H and Biomine and left out: WebKB on its real pages
(no theory comes with the folds), NELL (extraction scores, safe
queries), probabilistic LUBM (invented probabilities, no exact
evaluation), reachability on SNAP networks with random probabilities.

## The programs of the Dice and Alea.jl papers

`bench.py --suite ppl` (baselines.md, section 5, "Benchmark programs of
the Dice and Alea.jl papers") holds the programs of four papers: Cao et
al.'s integer arithmetic programs (`arith`, 28), the bit-blasting
paper's Table 2 (`hybit`, 19), Pluck's Dice.jl sequence models (`pluck`,
4), all Alea.jl programs, and noDice's purely probabilistic Dice
programs (`nodice`, 11). Alea runs an Alea.jl program itself; tddlog,
ProbLog and Dice run the Boolean circuit Alea builds for it, which
`alea_bench.jl` writes (a ProbLog program, and a Dice program per
Boolean answer or per bit of an expectation, the seconds summed). The
`nodice` programs are Dice programs, translated by `dice2problog.py` for
tddlog and ProbLog and by `dice2alea.py` for Alea.

### Results: tddlog 0.1.0, Dice and Alea.jl

tddlog 0.1.0 `auto` (tididi e4d60911), linked statically, Dice 1.0 with
an unlimited stack and Alea.jl at revision cf5e5b8 (Julia 1.11.9, CUDD
3.0.0) in one job on a cloud machine (96 physical cores, 1488 GB) on
2026-10-19, in 60 lanes, each process limited to 24 GB of address space.
The three systems run on an instance back to back in one lane, in an
order rotated by instance and repetition; three repetitions, 600 s each.
ProbLog's d-DNNF ran in a pass of its own (300 s) as a second reference.
Seconds are the median of the repetitions, a failed repetition counted
as longer than any time; `(k/3)` marks an instance answered in k
repetitions, `>T` a timeout, `oom` an allocation that failed inside the
process, `killed` a SIGTERM from outside the driver and `stack` a stack
overflow. The columns:

- **paper**: the time the paper reports for its own system, on its own
  hardware: Cao et al.'s Table 1 (the Dice.jl binary), Pluck's Table 1
  (Dice.jl), noDice's Table 1 (Dice, Apple M2 Pro). The bit-blasting
  paper reports errors, not times.
- **Alea**: the wall time of the Julia process. **in-proc**: what the
  process reports for building the program and running `pr` the first
  time; **JIT**: Julia's compilation within it; **warm**: the same done
  a second time in the process, run when the first took less than 200 s.
  The warm time is Alea without compilation, the time the papers report.
- **tddlog Δ**, **Dice Δ**: the largest absolute and the largest
  relative difference of an answer from the reference (`absolute /
  relative`): Alea's answers, else ProbLog's. `(P)`: against ProbLog's
  eight printed digits. `zero`: every answer printed 0 though the
  evidence has a probability a double holds; `underflow`: every answer
  printed 0 and the evidence is below the smallest normal double.

The arithmetic programs of Cao et al.:

| instance | paper | tddlog | Dice | Alea | in-proc | JIT | warm | ProbLog | tddlog Δ | Dice Δ |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| book | 5.3 | 12.77 | >600 | 58.54 | 46.64 | 45.61 | 1.01 | >300 | 1e-16 / 1e-15 |  |
| tugofwar | 0.106 | 0.20 | 305.85 | 77.50 | 67.03 | 66.96 | 0.04 | >300 | 3e-16 / 3e-16 | 4e-13 / 5e-13 |
| caesar_small | 0.041 | 0.08 | 0.59 | 79.62 | 68.93 | 68.87 | 0.03 | 7.15 | 1e-15 / 3e-14 | zero |
| caesar_medium | 0.239 | 0.76 | 15.23 | 80.45 | 69.42 | 69.23 | 0.17 | 40.52 | 9e-15 / 3e-13 | zero |
| caesar_large | 0.556 | 0.73 | 63.48 | 80.17 | 69.11 | 68.68 | 0.43 | 91.66 | 1e-29 / 6e-13 | zero |
| caesar_huge |  | 1.66 | 398.27 | 85.53 | 70.33 | 67.90 | 1.99 | >300 | 3e-69 / 3e-12 | underflow |
| ranking_small | 0.007 | 0.01 | 0.73 | 40.95 | 30.32 | 30.31 | 0.007 | 33.93 | 3e-16 / 1e-15 | 5e-13 / 2e-12 |
| ranking_medium | 0.022 | 0.02 | 203.44 | 40.85 | 30.22 | 30.20 | 0.01 | 147.36 | 6e-16 / 1e-14 | 4e-13 / 2e-12 |
| ranking_large | 0.048 | 0.07 | >600 | 38.48 | 28.55 | 28.51 | 0.03 | 235.60 | 2e-15 / 3e-14 |  |
| radar | 0.034 | 0.04 | 0.24 | 55.55 | 44.83 | 44.78 | 0.03 | 32.65 | 3e-16 / 2e-15 | 5e-13 / 3e-12 |
| fw_small | 0.009 | 0.005 | 0.02 | 85.76 | 75.06 | 75.02 | 0.01 | 0.11 | 0 | 0 |
| fw_medium | 0.515 | 0.03 | >600 | 85.95 | 75.12 | 74.69 | 0.45 | 15.79 | 6e-17 / 3e-14 |  |
| fw_large | 3.41 | 0.52 | >600 | 93.05 | 77.99 | 75.47 | 2.53 | >300 | 6e-17 / 9e-15 |  |
| linext_small | 0.003 | 0.005 | 0.09 | 38.94 | 28.31 | 28.30 | 0.005 | 0.15 | 2e-16 / 4e-16 | 2e-16 / 4e-16 |
| linext_medium | 0.007 | 0.008 | 0.25 | 40.74 | 30.10 | 30.08 | 0.007 | 24.26 | 6e-16 / 2e-15 | 3e-13 / 3e-12 |
| linext_large | 0.072 | 0.01 | 0.95 | 40.53 | 29.89 | 29.87 | 0.02 | >300 | 4e-16 / 2e-15 | 4e-13 / 2e-12 |
| triangle_small | 0.086 | 0.24 | 93.81 | 51.17 | 40.66 | 40.60 | 0.04 | >300 | 3e-16 / 5e-15 | 2e-13 / 7e-13 |
| triangle_medium | 0.455 | 0.57 | 459.28 | 50.16 | 39.82 | 39.65 | 0.16 | >300 | 2e-15 / 3e-14 | 4e-13 / 2e-12 |
| triangle_large | 17.4 | 25.92 | >600 | 78.66 | 48.67 | 37.92 | 10.66 | >300 | 5e-15 / 8e-14 |  |
| gcd_small | 2.88 | 11.40 | >600 | 20.84 | 5.87 | 3.48 | 2.44 | >300 | 2e-16 / 4e-16 |  |
| gcd_medium | 104 | 159.16 | >600 | 257.73 | 81.63 | 3.42 | 80.21 | >300 | 2e-16 / 4e-16 |  |
| gcd_large | >3600 | >600 | >600 | oom | oom | oom | oom | oom |  |  |
| disease_small | 7.91 | 11.68 | oom | 186.70 | 84.51 | 32.88 | 53.37 | >300 | 1e-15 / 6e-13 |  |
| disease_medium | 764 | >600 | oom | oom | oom | oom | oom | oom |  |  |
| disease_large |  | >600 | oom | oom | oom | oom | oom | oom |  |  |
| luhn_small | 0.039 | 0.01 | 0.26 | 63.66 | 52.97 | 52.93 | 0.01 | 2.77 | 2e-16 / 1e-15 | 2e-13 / 2e-12 |
| luhn_medium | 4.58 | 0.13 | 4.78 | 63.70 | 53.08 | 53.01 | 0.03 | >300 | 2e-15 / 1e-14 | 4e-13 / 2e-12 |
| plankton |  | 0.32 | oom | 43.38 | 33.05 | 32.94 | 0.10 | >300 | 1e-16 / 4e-14 |  |

The programs of the bit-blasting paper's Table 2, at its precision
(conjugate_gaussians on the artifact file's observations 8 and 9;
`data/alea/hybit/conjugate_gaussians.jl` now observes Table 2's 1 and
2):

| instance | tddlog | Dice | Alea | in-proc | JIT | warm | ProbLog | tddlog Δ | Dice Δ |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| pi | 1.49 | >600 | 162.09 | 105.80 | 82.10 | 23.81 | >300 | 6e-17 / 1e-16 |  |
| weekend | 20.45 | 393.45 | 85.96 | 52.92 | 41.86 | 11.39 | oom | 8e-16 / 2e-15 | zero |
| spacex | oom | >600 | 283.54 | 147.65 | 28.61 | 125.63 | >300 |  |  |
| GPA | 23.63 | 41.50 | 507.65 | 497.58 | 486.46 |  | oom | 1e-16 / 2e-16 | 9e-14 / 2e-13 |
| tug_of_war | 0.54 | >600 | oom | oom | oom | oom | >300 | unchecked |  |
| altermu2 | oom | >600 | oom | oom | oom | oom | >300 |  |  |
| conjugate_gaussians | 0.88 | >600 | oom | oom | oom | oom | >300 | unchecked |  |
| normal_mixture_theta | 29.98 | >600 | oom | oom | oom | oom | >300 | unchecked |  |
| normal_mixture_mu1 | 134.92 | >600 | oom | oom | oom | oom | >300 | unchecked |  |
| normal_mixture_mu2 | 162.22 | >600 | oom | oom | oom | oom | >300 | unchecked |  |
| zeroone_w1 | 0.32 | >600 | 83.10 | 71.09 | 70.12 | 0.95 | >300 | 4e-15 / 8e-15 |  |
| zeroone_w2 | 1.25 | >600 | 109.42 | 81.45 | 69.98 | 12.61 | >300 | 1e-13 / 2e-13 |  |
| coinBias | >600 | >600 | oom | oom | oom | oom | oom |  |  |
| addFun_sum | 0.70 | >600 | oom | oom | oom | oom | >300 | unchecked |  |
| clickGraph | 0.32 | >600 | oom | oom | oom | oom | 5.53 | 5e-9 / 9e-9 (P) |  |
| trueskill | 180.86 | >600 | 269.44 | 104.75 | 29.59 | 73.74 | >300 | 3e-16 / 7e-16 |  |
| clinicalTrial1 | 1.66 | >600 | oom | oom | oom | oom | 21.47 | 6e-10 / 4e-9 (P) |  |
| clinicalTrial2 | 0.02 | >600 | oom | oom | oom | oom | 22.09 | 5e-9 / 2e-8 (P) |  |
| addFun_max | 37.46 | >600 | oom | oom | oom | oom | >300 | unchecked |  |

Pluck's sequence models:

| instance | paper | tddlog | Dice | Alea | in-proc | JIT | warm | ProbLog | tddlog Δ | Dice Δ |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| hmm | 0.00276 | 0.02 | >600 | 10.84 | 0.63 | 0.62 | 0.008 | >300 | 6e-27 / 3e-14 |  |
| pcfg | 8.83 | 100.78 | >600 | 42.07 | 11.88 | 2.85 | 9.37 | >300 | 9e-23 / 2e-15 |  |
| string_editing | 61.3 | 0.06 | 1.66 | 166.62 | 53.70 | 2.05 | 51.74 | >300 | 2e-26 / 7e-15 | 2e-24 / 1e-12 |
| sorted_list | 21.9 | 7.72 | 44.44 | 92.02 | 27.95 | 1.35 | 27.34 | >300 | 2e-25 / 2e-14 | 4e-24 / 5e-13 |

noDice's programs:

| instance | paper | tddlog | Dice | Alea | in-proc | JIT | Roulette | in Roulette | ProbLog | tddlog Δ | Dice Δ | Roulette Δ |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| coupon2-10-obs | 0.721 | 0.07 | 0.35 | 9.61 | 1.32 | 1.13 | 0.82 | 0.46 | >300 | 2e-16 / 6e-16 | 3e-13 / 1e-12 | 1e-15 / 4e-15 |
| coupon2-11-obs | 1.96 | 0.14 | 0.93 | 9.95 | 1.74 | 1.14 | 3.25 | 2.91 | >300 | 6e-15 / 3e-14 | 2e-13 / 7e-13 | 4e-16 / 2e-15 |
| coupon2-12-obs | 5.8 | 0.19 | 2.34 | 10.56 | 2.51 | 1.09 | 27.01 | 26.66 | >300 | 7e-16 / 4e-15 | 2e-13 / 1e-12 | 2e-15 / 8e-15 |
| coupon2-13-obs | 20.7 | 0.41 | 8.48 | 14.07 | 5.73 | 1.14 | 335.35 | 334.96 | >300 | 5e-16 / 3e-15 | 1e-13 / 7e-13 | 3e-15 / 2e-14 |
| coupon2-14-obs | 63.4 | 0.81 | 33.83 | 30.72 | 22.60 | 1.17 | >600 | >600 | >300 | 2e-16 / 2e-15 | 2e-14 / 2e-13 |  |
| survey | 0.001 | 0.004 | 0.02 | 8.74 | 0.49 | 0.49 | 0.36 | 0.002 | 0.08 | 6e-17 / 4e-16 | 1e-16 / 2e-16 | 8e-17 / 5e-16 |
| hepar2 | 0.099 | 0.06 | 0.06 | 9.34 | 0.93 | 0.79 | 0.57 | 0.21 | 0.75 | 3e-17 / 2e-16 | 3e-14 / 2e-13 | 1e-16 / 1e-16 |
| insurance | 0.58 | 0.09 | 0.24 | 10.82 | 2.29 | 1.46 | 1.40 | 1.03 | 48.55 | 2e-16 / 2e-15 | 7e-6 / 4e-4 | 2e-16 / 1e-15 |
| water | 9.24 | 0.78 | 199.72 | 10.88 | 1.56 | 1.43 | 0.49 | 0.17 | 8.16 | 1e-16 / 7e-16 | 5e-13 / 1e-12 | 3e-17 / 4e-16 |
| alarm | 160 | 0.05 | 0.15 | 9.51 | 1.14 | 0.77 | 260.03 | 259.66 | 1.95 | 1e-16 / 3e-16 | 3e-13 / 8e-13 | 3e-16 / 2e-15 |
| 3sat-0 | 19.5 | 0.45 | 8.44 | 9.36 | 1.28 | 0.35 | 3.22 | 2.80 | >300 | 1e-22 / 1e-15 | 1e-19 / 1e-12 | 1e-22 / 1e-15 |

Roulette runs the Dice program of each noDice instance, translated; the
table of the answers right and of the total times, as for the Dice
programs (600 s a run):

| programs | n | tddlog | Dice | Alea | Roulette |
|---|---:|---:|---:|---:|---:|
| noDice's programs | 11 | 11 | 10; 854 / 3.06 | 11; 41.6 / 3.06 | 10; 1229 / 3.06 |

Roulette answers 10 of the 11 instances it ran and fails on 1 where it
runs out of time (coupon2-14-obs). Of the instances where tddlog or
Roulette takes 0.2 s or more, Roulette is faster than tddlog (in its own
clock, tddlog's process) on 1: water (0.17 s against 0.78).

Answers. Of the 62 instances tddlog answers 56, Dice 31, Alea 47 and
ProbLog's d-DNNF 20. tddlog answers ten where Alea fails (hybit
tug_of_war, hybit conjugate_gaussians, hybit normal_mixture_theta, hybit
normal_mixture_mu1, hybit normal_mixture_mu2, hybit addFun_sum, hybit
clickGraph, hybit clinicalTrial1, hybit clinicalTrial2 and hybit
addFun_max). Alea answers one that tddlog does not (hybit spacex).
tddlog answers every instance Dice answers.

tddlog agrees with the reference on every instance both answer, to a
relative 3.1e-12 at most (on arith caesar_huge, absolute 3.5e-69). Where
Alea fails, ProbLog agrees with tddlog to its eight printed digits on
hybit clickGraph (5e-9), hybit clinicalTrial1 (6e-10) and hybit
clinicalTrial2 (5e-9). tddlog alone answers hybit tug_of_war, hybit
conjugate_gaussians, hybit normal_mixture_theta, hybit
normal_mixture_mu1, hybit normal_mixture_mu2, hybit addFun_sum and hybit
addFun_max (unchecked). Dice agrees to a relative 2.7e-12 or less
wherever it prints nonzero answers, but for nodice insurance. Dice
prints 0 for every answer on arith caesar_small, arith caesar_medium,
arith caesar_large, arith caesar_huge and hybit weekend. Their evidence
has probability 4.6e-15 (small), 2.3e-69 (medium), 4.5e-132 (large),
1.1e-323 (huge) and 1.1e-7 (weekend), from tddlog's
`--evidence-probability`; Dice prints zeros wherever the evidence has
probability below 1e-6 (baselines.md, section 4), so they are marked
`zero` where a double holds the evidence and `underflow` where it is
below the smallest normal double. On insurance Dice is off by 7.4e-6
(relative 4.4e-4): its `discrete` drops the network's table entries
below 1e-4.

Failures. Of the 558 timed runs 63 ended on a failed allocation: 6 of
tddlog's, 12 of Dice's and 45 of Alea's.

Times. Against Dice (its zero answers counted as answers), over the 56
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, tddlog takes 935 s in all and Dice 17283 s;
over the 31 both answer, 59.4 s against 2283 s. tddlog is faster on 26
of the 31 both answer and Dice on 0, both under 0.2 s or level on the
other 5. tddlog alone answers 25, Dice alone 0. Against Alea's wall
time, over the 57 instances given to both that either answers, a run
that does not answer counted at the 600 s limit, tddlog takes 1535 s in
all and Alea 9785 s; over the 46 both answer, 566 s against 3501 s;
against its first time in the process, over the 57 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, tddlog takes 1535 s in all and Alea 8479 s; over the 46 both
answer, 566 s against 2331 s. Without Julia's compilation (the warm
time, or the first time in the process less its JIT where there was no
warm run), over the 57 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, tddlog takes 1535 s
in all and Alea 6532 s; over the 46 both answer, 566 s against 407 s.
tddlog is faster on 16 of the 46 both answer and Alea on 15, both under
0.2 s or level on the other 15. tddlog alone answers 10, Alea alone 1.
Alea's largest leads are on hybit trueskill (73.7 s against 181), pluck
pcfg (9.37 s against 101), arith gcd_medium (80.2 s against 159), arith
triangle_large (10.7 s against 25.9) and hybit GPA (11.1 s against
23.6).

Of the 18 instances where tddlog takes 1 s or more, `auto`'s attempts
before the one that answers (vtrees it tries and drops, and attempts
stopped at a smaller work limit) take half the time or more on 2: on
Alea's largest leads they take 33 of 181 s on hybit trueskill, 5.72 of
101 s on pluck pcfg, 22 of 159 s on arith gcd_medium, 9.69 of 25.9 s on
arith triangle_large and 0 of 23.6 s on hybit GPA, and without them
tddlog would take 148 s on hybit trueskill, 95.1 s on pluck pcfg, 137 s
on arith gcd_medium, 16.2 s on arith triangle_large and 23.6 s on hybit
GPA against Alea's 73.7, 9.37, 80.2, 10.7 and 11.1.

Alea's first run is mostly Julia compiling Alea's methods for the
program's types: 486 of 498 s on hybit GPA, 28.3 to 75.5 s on the
arithmetic programs but the gcd ones. Its times without compilation on
the arithmetic programs run from 0.0072 of Cao et al.'s Dice.jl times
(arith luhn_medium 0.033 s against 4.58) to 6.7 times them (arith
disease_small 53.4 s against 7.91). Dice's times here and in noDice's
Table 1 differ by large factors both ways on the same programs (nodice
water 200 s against 9.24, nodice alarm 0.15 s against 160).

## The bit-blasting paper's programs

`bench.py --suite bitblast` holds the programs of the bit-blasting paper
(Garg, Holtzen, Van den Broeck and Millstein, PLDI 2024) on ladders of
size, as circuits that `alea_bench.jl` writes from Alea.jl programs:
each program of its Table 2 (`data/alea/hybit`) at the paper's precision
and at 2, 4 and 6 bits after the point below and above it; the gene
expression program of its Section 2.1 (`data/alea/bitblast/ genes.jl`)
at Figure 2's precision from 5 to 100 genes (the paper's 5 to 50), and
with 5 genes by precision (`bb_genes5`); and the multimodal model of
Figure 3 and the conjugate Gaussians of Figure 5 by precision. An
instance is `bb_NAME_wW`, W the width of the program's numbers in bits
(Table 2's column `Bit`), or `bb_genes_tT` with T genes: 154 instances
in 23 families. Alea's own bit-count assertions reject two of them
(`bb_normal_mixture_theta_w7`, `bb_multimodal_w9`), which leaves 152.
The paper's other systems run on the artifact's own programs of the same
models (baselines.md, section 10).

### Results: tddlog 0.1.0 and Alea.jl, with Dice and ProbLog from an earlier run

tddlog 0.1.0 (tididi e4d60911), linked statically, in three arms:
`auto`; `--vtree program` (`program`), a right-linear vtree over the
facts in the reverse of the order the program states them, so that the
diagram reads the circuit's flips in the order Alea made them; and
`--vtree decision` (`decision`), which grounds the program again with
each derived atom conditioned on the selector flips that guard it and
places each selector above the flips it decides between (`n/a` where the
grounder reads no flip as a guard: clickGraph, clinicalTrial1,
clinicalTrial2 and pi). Alea.jl at revision cf5e5b8 on Julia 1.11.9 with
CUDD 3.0.0, Distributions.jl 0.25.131 and StatsFuns 2.2.1. One job on a
cloud machine (96 physical cores, 1488 GB) on 2026-10-19, in 45 lanes of
33 GB a process, with the earlier run's rotation, 600 s and family skip
rule; the four run on an instance back to back in one lane, three
repetitions. Dice and ProbLog were not rerun: their columns are the
earlier run's (below; three repetitions for Dice, ProbLog's d-DNNF in a
pass of its own), a run of their own, and their times are not compared
with the others' here. The page's plots leave them out.

Seconds are the median of the repetitions, a failed repetition counted
as longer than any time; `(k/n)` marks an instance answered in k of n
repetitions. Alea's time is its warm time (the program built and
answered a second time in the process, without Julia's compilation, as
the papers time it; the first run's time less its compilation where the
first took more than 200 s), and `Alea process` the wall time of the
Julia process. `oom` is an allocation that failed inside the process,
`killed` a SIGTERM from earlyoom, the machine's out-of-memory guard,
when its free memory ran low, `zero` every answer printed 0 though a
double holds the answers, `-` not run after a failure on a smaller size.
`ref` names the reference: Alea's answer, else Ganak's exact count of
the circuit (below), else ProbLog's, else tddlog's own; the difference
columns give the largest absolute and relative difference of an answer
from it.

Of the 152 instances, 21 lie past the sizes at which every system had
failed (the table leaves them out), and none every system answers in
under a second. tddlog answers 116 with `auto`, 55 with `program` and 52
with `decision` (118 by some arm); Alea 80; Dice (earlier run) 17, and
prints 0 for every answer of 8 more.

- Against Alea's warm time, over the 93 instances given to both that
  either answers, a run that does not answer counted at the 600 s limit,
  tddlog (`auto`) takes 3384 s in all and Alea 11852 s; over the 76 both
  answer, 1076 s against 3205 s. tddlog (`auto`) is faster on 44 of the
  76 both answer and Alea on 27, both under 0.2 s or level on the other
  5. tddlog (`auto`) alone answers 14, Alea alone 3. Against the Julia
  process, over the 93 instances given to both that either answers, a
  run that does not answer counted at the 600 s limit, tddlog takes 3384
  s in all and the process 21448 s; over the 76 both answer, 1076 s
  against 12337 s. For `program` against Alea, over the 63 instances
  given to both that either answers, a run that does not answer counted
  at the 600 s limit, `program` takes 7404 s in all and Alea 5933 s;
  over the 48 both answer, 1126 s against 2095 s; for `decision`, over
  the 57 instances given to both that either answers, a run that does
  not answer counted at the 600 s limit, `decision` takes 10421 s in all
  and Alea 2605 s; over the 39 both answer, 1269 s against 565 s.
- tddlog reaches further than Alea on 15 programs: addFun_max, to 29
  bits in 50.7 s, where Alea answers to 21 bits (152 s) and runs out of
  memory at 23; addFun_sum, to 29 bits in 1.04 s, where Alea answers to
  23 bits (432 s) and times out at 25; clickGraph, to 16 bits in 0.69 s,
  where Alea answers to 8 bits (6.55 s) and runs out of memory at 10;
  clinicalTrial1, to 10 bits in 51.6 s, where Alea answers to 8 bits
  (263 s) and runs out of memory at 10; clinicalTrial2, to 18 bits in
  0.021 s, where Alea answers to 12 bits (308 s) and runs out of memory
  at 14; coinBias, to 22 bits in 131 s (`decision`), where Alea answers
  to 20 bits (61 s) and runs out of memory at 22; conjugate_gaussians,
  to 29 bits in 1.52 s, where Alea answers to 23 bits (169 s) and times
  out at 25; genes, to 100 genes in 12 s, where Alea times out at 5;
  genes5, to 27 bits in 10.4 s, where Alea answers to 25 bits (81.4 s)
  and runs out of memory at 27; normal_mixture_mu1, to 9 bits in 139 s,
  where Alea answers to 7 bits (1.51 s) and runs out of memory at 9;
  normal_mixture_mu2, to 9 bits in 173 s, where Alea answers to 7 bits
  (1.74 s) and runs out of memory at 9; normal_mixture_theta, to 9 bits
  in 31.9 s, where Alea runs out of memory at 9; pi, to 20 bits in 185
  s, where Alea answers to 16 bits (334 s) and times out at 18;
  tug_of_war, to 28 bits in 0.74 s, where Alea answers to 20 bits (159
  s) and times out at 22; zeroone_w2, to 25 bits in 6.76 s, where Alea
  answers to 23 bits (326 s) and runs out of memory at 25.
- Alea reaches further on conjugate_ex3 and multimodal. Of these, tddlog
  answers no size of multimodal: circuits of 4617 to 4617 flips at their
  smallest size.
- `program` against `auto`: over the 71 instances given to both that
  either answers, a run that does not answer counted at the 600 s limit,
  `auto` takes 1790 s in all and `program` 11005 s; over the 54 both
  answer, 499 s against 1292 s; `auto` is faster on 26 of the 54 both
  answer and `program` on 14. `program` answers 1 that `auto` does not
  (spacex), `auto` 16 that `program` does not (GPA, addFun_max,
  addFun_sum, altermu2, clickGraph, coinBias, conjugate_ex3,
  conjugate_gaussians, genes, genes5, normal_mixture_mu1,
  normal_mixture_mu2, normal_mixture_theta, pi, tug_of_war and weekend).
- `decision` against `auto`: over the 67 instances given to both that
  either answers, a run that does not answer counted at the 600 s limit,
  `auto` takes 1628 s in all and `decision` 10527 s; over the 51 both
  answer, 800 s against 1396 s; `auto` is faster on 10 of the 51 both
  answer and `decision` on 40. `decision` answers 1 that `auto` does not
  (coinBias), `auto` 15 that `decision` does not (addFun_max,
  addFun_sum, clickGraph, clinicalTrial1, clinicalTrial2,
  conjugate_gaussians, normal_mixture_mu1, normal_mixture_mu2,
  normal_mixture_theta, pi, spacex, trueskill, tug_of_war, zeroone_w1
  and zeroone_w2).
- At the paper's precision (19 programs of Table 2), tddlog answers 18
  by some arm (GPA, addFun_max, addFun_sum, clickGraph, clinicalTrial1,
  clinicalTrial2, coinBias, conjugate_gaussians, normal_mixture_mu1,
  normal_mixture_mu2, normal_mixture_theta, pi, spacex, trueskill,
  tug_of_war, weekend, zeroone_w1 and zeroone_w2), Alea 11 (GPA,
  addFun_sum, clinicalTrial1, clinicalTrial2, conjugate_gaussians, pi,
  spacex, trueskill, weekend, zeroone_w1 and zeroone_w2), Dice (earlier
  run) 1 (GPA); no system answers altermu2. On the gene expression
  program at Figure 2's precision (5 to 50 genes), `auto` and `decision`
  answer every number of genes, and `program`, Alea and Dice (earlier
  run) none.

How far each system reaches: per program, the largest size (in bits;
genes for `genes`) a system answers and its time there, else its failure
at the smallest size it ran; `paper` is the paper's precision.

| program | paper | auto | program | decision | Alea | Dice (earlier) |
|---|---:|---:|---:|---:|---:|---:|
| GPA | 25 | 31 (25.6 s) | >600 at 19 | 31 (22.2 s) | 31 (10.2 s) | 31 (46.2 s) |
| addFun_max | 23 | 29 (50.7 s) | 19 (220 s) | oom at 17 | 21 (152 s) | >600 at 17 |
| addFun_sum | 23 | 29 (1.04 s) | 21 (84.3 s) | 17 (556 s) | 23 (432 s) | >600 at 17 |
| altermu2 | 17 | 15 (51 s) | 11 (92.2 s) | 15 (49.6 s) | 15 (122 s) | >600 at 11 |
| clickGraph | 10 | 16 (0.69 s) | 10 (21.4 s) | n/a at 4 | 8 (6.55 s) | 6 (47.9 s) |
| clinicalTrial1 | 8 | 10 (51.6 s) | 10 (31.6 s) | n/a at 4 | 8 (263 s) | zero at 4 |
| clinicalTrial2 | 12 | 18 (0.021 s) | 18 (0.91 s) | n/a at 6 | 12 (308 s) | 8 (137 s) |
| coinBias | 22 | 20 (36.2 s) | >600 at 16 | 22 (131 s) | 20 (61 s) | >600 at 16 |
| conjugate_ex3 |  | 20 (140 s) | >600 at 18 | 20 (66.1 s) | 24 (280 s) | >600 at 18 |
| conjugate_gaussians | 23 | 29 (1.52 s) | 23 (140 s) | >600 at 17 | 23 (169 s) | >600 at 17 |
| genes | 5-50 | 100 (12 s) | >600 at 5 | 100 (10.7 s) | >600 at 5 | >600 at 5 |
| genes5 |  | 27 (10.4 s) | >600 at 17 | 27 (9.83 s) | 25 (81.4 s) | zero at 17 |
| multimodal |  | oom at 11 | oom at 11 | oom at 11 | 11 (31.2 s) | >600 at 11 |
| normal_mixture_mu1 | 9 | 9 (139 s) | 7 (2.62 s) | 7 (0.79 s) | 7 (1.51 s) | >600 at 7 |
| normal_mixture_mu2 | 9 | 9 (173 s) | 7 (2.91 s) | oom at 7 | 7 (1.74 s) | >600 at 7 |
| normal_mixture_theta | 9 | 9 (31.9 s) | oom at 9 | oom at 9 | oom at 9 | >600 at 9 |
| pi | 14 | 20 (185 s) | 18 (174 s) | n/a at 8 | 16 (334 s) | >600 at 8 |
| spacex | 19 | 17 (55.7 s) | 19 (113 s) | oom at 13 | 19 (153 s) | >600 at 13 |
| trueskill | 10 | 10 (193 s) | 10 (53.1 s) | oom at 8 | 10 (85.7 s) | >600 at 8 |
| tug_of_war | 22 | 28 (0.74 s) | 18 (189 s) | oom at 16 | 20 (159 s) | >600 at 16 |
| weekend | 24 | 30 (24.8 s) | >600 at 18 | 30 (21.7 s) | 30 (14.2 s) | 20 (355 s) |
| zeroone_w1 | 16 | 22 (2.08 s) | 22 (5.71 s) | 20 (82.1 s) | 22 (39 s) | 14 (354 s) |
| zeroone_w2 | 19 | 25 (6.76 s) | 25 (49.8 s) | 19 (23 s) | 23 (326 s) | 13 (131 s) |

The full table:

| instance | flips | gates | tddlog | program | decision | Dice (earlier) | Alea | Alea process | ProbLog | ref | tddlog diff | program diff | decision diff | Dice (earlier) diff | Alea diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GPA_w19 | 45061 | 368680 | 17.5 | >600 | 15.6 | 29.4 | 10.2 | 564 | 0.064 | alea | 2.2e-16 / 3.6e-16 |  | 2.2e-16 / 3.6e-16 | 9.4e-14 / 1.5e-13 | ref |
| GPA_w21 | 61441 | 417832 | 17.9 | - | 16.5 | 32.3 | 9.49 | 486 | - | alea | 2.2e-16 / 3.6e-16 |  | 2.2e-16 / 3.6e-16 | 9.4e-14 / 1.5e-13 | ref |
| GPA_w23 | 77812 | 466966 | 20 | - | 17.7 | 34.8 | 9.52 | 467 | - | alea | 1.1e-16 / 1.8e-16 |  | 1.1e-16 / 1.8e-16 | 9.4e-14 / 1.5e-13 | ref |
| GPA_w25 | 94168 | 516070 | 19.7 | - | 19.1 | 38.5 | 8.68 | 438 | - | alea | 1.1e-16 / 1.8e-16 |  | 1.1e-16 / 1.8e-16 | 9.4e-14 / 1.5e-13 | ref |
| GPA_w27 | 110529 | 565184 | 21.5 | - | 18.7 | 44.1 | 9.02 | 420 | - | alea | 1.1e-16 / 1.8e-16 |  | 1.1e-16 / 1.8e-16 | 9.4e-14 / 1.5e-13 | ref |
| GPA_w29 | 126871 | 614260 | 23.5 | - | 20.3 | 44 | 9.66 | 422 | - | alea | 1.1e-16 / 1.8e-16 |  | 1.1e-16 / 1.8e-16 | 9.4e-14 / 1.5e-13 | ref |
| GPA_w31 | 143187 | 663285 | 25.6 | - | 22.2 | 46.2 | 10.2 | 423 | - | alea | 1.1e-16 / 1.8e-16 |  | 1.1e-16 / 1.8e-16 | 9.4e-14 / 1.5e-13 | ref |
| addFun_max_w17 | 566 | 18099 | 38.8 | 46.8 | oom | >600 | 6.87 | 42.3 | >600 | alea | 2.2e-16 / 4.4e-16 | 7.9e-15 / 1.6e-14 |  |  | ref |
| addFun_max_w19 | 580 | 21035 | 44.5 | 220 | - | - | 33.8 | 113 | - | alea | 3.3e-16 / 6.7e-16 | 4.5e-14 / 9.1e-14 |  |  | ref |
| addFun_max_w21 | 594 | 23603 | 37.1 | oom | - | - | 152 | 328 | - | alea | 5.6e-16 / 1.1e-15 |  |  |  | ref |
| addFun_max_w23 | 604 | 26275 | 37.2 | - | - | - | oom | oom | - | tddlog | ref |  |  |  |  |
| addFun_max_w25 | 618 | 28939 | 55.4 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| addFun_max_w27 | 630 | 31297 | 43.6 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| addFun_max_w29 | 642 | 33393 | 50.7 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| addFun_max_p1024 | 4014 | 187241 | >600 | - | - | - | - | - | - |  |  |  |  |  |  |
| addFun_max_p16 | 196 | 3801 | 0.478 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| addFun_max_p256 | 1090 | 50669 | 287 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| addFun_max_p4 | 168 | 1249 | 0.352 | oom | oom | >600 | >600 | >600 | oom | tddlog | ref |  |  |  |  |
| addFun_max_p64 | 366 | 13777 | 5.08 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| addFun_sum_w17 | 152 | 2800 | 0.433 | 4.27 | 556 | >600 | 3.63 | 72.4 | >600 | alea | 7.8e-16 / 1.6e-15 | 1.1e-13 / 2.2e-13 | 1.8e-11 / 3.6e-11 |  | ref |
| addFun_sum_w19 | 168 | 3212 | 0.5 | 18.7 | oom | - | 17.3 | 104 | - | alea | 5.6e-16 / 1.1e-15 | 5.5e-13 / 1.1e-12 |  |  | ref |
| addFun_sum_w21 | 184 | 3624 | 0.611 | 84.3 | - | - | 84.3 | 274 | - | alea | 6.7e-16 / 1.3e-15 | 1.3e-12 / 2.6e-12 |  |  | ref |
| addFun_sum_w23 | 196 | 3820 | 0.708 | oom | - | - | 432 | 493 | - | alea | 6.7e-16 / 1.3e-15 |  |  |  | ref |
| addFun_sum_w25 | 212 | 4208 | 0.728 | - | - | - | >600 | >600 | - | tddlog | ref |  |  |  |  |
| addFun_sum_w27 | 228 | 4596 | 0.851 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| addFun_sum_w29 | 244 | 4984 | 1.04 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| altermu2_w11 | 20458 | 274380 | 12.3 | 92.2 | 11.2 | >600 | 5.79 | 67.1 | >600 | alea | 4.7e-14 / 9.4e-14 | 4.8e-14 / 9.6e-14 | 4.7e-14 / 9.4e-14 |  | ref |
| altermu2_w13 | 40822 | 336280 | 19.6 | oom | 18.5 | - | 26.1 | 121 | - | alea | 3.4e-14 / 6.8e-14 |  | 3.4e-14 / 6.8e-14 |  | ref |
| altermu2_w15 | 60746 | 397700 | 51 | - | 49.6 | - | 122 | 278 | - | alea | 1.6e-14 / 3.1e-14 |  | 1.6e-14 / 3.1e-14 |  | ref |
| altermu2_w17 | 80350 | 457880 | oom | - | oom | - | oom | oom | - |  |  |  |  |  |  |
| altermu2_p1024 | 240710 | 1777240 | >600 | - | - | - | - | - | - |  |  |  |  |  |  |
| altermu2_p16 | 7670 | 35720 | 1.84 | oom | - | - | - | - | - | tddlog | ref |  |  |  |  |
| altermu2_p4 | 2230 | 13800 | 1.01 | 102 | oom | >600 | oom | oom | >600 | tddlog | ref | 8.2e-14 / 1.6e-13 |  |  |  |
| altermu2_p64 | 25430 | 121720 | 14.5 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| clickGraph_w4 | 52 | 135 | 0.006 | 0.008 | n/a | 0.171 | 0.009 | 47.9 | 0.15 | alea | 2.2e-16 / 3.7e-16 | 2.2e-16 / 3.7e-16 |  | 3.5e-13 / 5.7e-13 | ref |
| clickGraph_w6 | 104 | 345 | 0.013 | 0.012 | - | 47.9 | 0.044 | 46.7 | 0.37 | alea | 2.2e-16 / 4.1e-16 | 2.2e-16 / 4.1e-16 |  | 4.4e-13 / 8.4e-13 | ref |
| clickGraph_w8 | 156 | 555 | 0.239 | 0.143 | - | >600 | 6.55 | 64.8 | 1.52 | alea | 8.9e-16 / 1.3e-15 | 8.9e-16 / 1.3e-15 |  |  | ref |
| clickGraph_w10 | 208 | 765 | 0.32 | 21.4 | - | - | oom | oom | 5.91 | Ganak | 5.2e-15 / 1.0e-14 | 4.4e-16 / 6.6e-16 |  |  |  |
| clickGraph_w12 | 260 | 975 | 0.373 | oom | - | - | - | - | 30.9 | Ganak | 8.3e-15 / 1.4e-14 |  |  |  |  |
| clickGraph_w14 | 312 | 1185 | 0.835 | - | - | - | - | - | wrong | Ganak | 1.3e-14 / 2.0e-14 |  |  |  |  |
| clickGraph_w16 | 364 | 1395 | 0.688 | - | - | - | - | - | oom | Ganak | 4.2e-15 / 8.4e-15 |  |  |  |  |
| clinicalTrial1_w4 | 807 | 3200 | 0.041 | 0.039 | n/a | zero | 0.23 | 50.6 | 1.12 | alea | 5.8e-15 / 2.9e-14 | 5.8e-15 / 2.9e-14 |  |  | ref |
| clinicalTrial1_w6 | 1613 | 8000 | 0.245 | 0.141 | - | zero | 6.18 | 66.9 | 4.77 | alea | 1.1e-14 / 6.4e-14 | 1.1e-14 / 6.4e-14 |  |  | ref |
| clinicalTrial1_w8 | 2419 | 12800 | 1.73 | 1.34 | - | >600 | 263 | 313 | 24 | alea | 4.2e-16 / 2.4e-15 | 4.2e-16 / 2.4e-15 |  |  | ref |
| clinicalTrial1_w10 | 3225 | 17600 | 51.6 | 31.6 | - | - | oom | oom | 121 | Ganak | 5.6e-17 / 3.3e-16 | 5.6e-17 / 3.3e-16 |  |  |  |
| clinicalTrial1_w12 | 4031 | 22400 | oom | oom | - | - | - | - | oom |  |  |  |  |  |  |
| clinicalTrial2_w6 | 89 | 412 | 0.009 | 0.006 | n/a | 1.36 | 0.018 | 48.8 | 0.389 | alea | 6.7e-16 / 1.3e-15 | 6.7e-16 / 1.3e-15 |  | 3.7e-13 / 7.4e-13 | ref |
| clinicalTrial2_w8 | 133 | 658 | 0.008 | 0.008 | - | 137 | 0.315 | 49.8 | 1.07 | alea | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 |  | 4.7e-13 / 1.2e-12 | ref |
| clinicalTrial2_w10 | 177 | 904 | 0.012 | 0.011 | - | >600 | 10.3 | 79.4 | 4.73 | alea | 1.1e-15 / 2.2e-15 | 1.1e-15 / 2.2e-15 |  |  | ref |
| clinicalTrial2_w12 | 221 | 1150 | 0.022 | 0.021 | - | - | 308 | 358 | 24 | alea | 6.1e-15 / 1.2e-14 | 6.1e-15 / 1.2e-14 |  |  | ref |
| clinicalTrial2_w14 | 265 | 1396 | 0.019 | 0.055 | - | - | oom | oom | 118 | Ganak | 1.1e-16 / 2.2e-16 | 5.6e-15 / 1.1e-14 |  |  |  |
| clinicalTrial2_w16 | 309 | 1642 | 0.019 | 0.207 | - | - | - | - | oom | Ganak | 2.2e-16 / 4.4e-16 | 1.7e-15 / 3.3e-15 |  |  |  |
| clinicalTrial2_w18 | 353 | 1888 | 0.021 | 0.908 | - | - | - | - | - | Ganak | 2.2e-16 / 4.4e-16 | 3.3e-15 / 6.7e-15 |  |  |  |
| coinBias_w16 | 12422 | 152450 | 8.42 | >600 | 7.7 | >600 | 8.76 | 82.7 | oom | alea | 2.4e-15 / 5.3e-15 |  | 2.4e-15 / 5.3e-15 |  | ref |
| coinBias_w18 | 20630 | 177156 | 13.7 | - | 12.9 | - | 19.4 | 112 | - | alea | 3.8e-15 / 6.1e-15 |  | 3.8e-15 / 6.1e-15 |  | ref |
| coinBias_w20 | 28834 | 201858 | 36.2 | - | 35.1 | - | 61 | 239 | - | alea | 1.0e-14 / 2.1e-14 |  | 1.0e-14 / 2.1e-14 |  | ref |
| coinBias_w22 | 37026 | 226532 | >600 | - | 131 | - | oom | oom | - |  |  |  |  |  |  |
| coinBias_w24 | 45190 | 251150 | - | - | oom | - | - | - | - |  |  |  |  |  |  |
| conjugate_ex3_w18 | 9147 | 239177 | 41.9 | >600 | 35.9 | >600 | 6.47 | 62.4 | oom | alea | 5.6e-15 / 1.0e-14 |  | 5.6e-15 / 1.0e-14 |  | ref |
| conjugate_ex3_w20 | 9135 | 290908 | 140 | - | 66.1 | - | 17.7 | 94.3 | - | alea | 6.7e-15 / 1.3e-14 |  | 6.7e-15 / 1.3e-14 |  | ref |
| conjugate_ex3_w22 | 9135 | 342186 | >600 | - | oom | - | 63.3 | 226 | - | alea |  |  |  |  | ref |
| conjugate_ex3_w24 | 9117 | 392519 | - | - | - | - | 280 | 323 | - | alea |  |  |  |  | ref |
| conjugate_ex3_w26 | 9105 | 442375 | - | - | - | - | >600 | >600 | - |  |  |  |  |  |  |
| conjugate_gaussians_w17 | 177 | 2788 | 0.425 | 1.51 | >600 | >600 | 1.16 | 47.9 | >600 | alea | 4.0e-15 / 2.1e-14 | 6.2e-15 / 2.1e-14 |  |  | ref |
| conjugate_gaussians_w19 | 201 | 3258 | 0.527 | 6.96 | - | - | 6.67 | 62.5 | - | alea | 2.4e-15 / 5.8e-15 | 8.7e-15 / 1.7e-14 |  |  | ref |
| conjugate_gaussians_w21 | 225 | 3728 | 0.73 | 29 | - | - | 34.1 | 143 | - | alea | 4.3e-15 / 8.7e-15 | 2.5e-14 / 4.9e-14 |  |  | ref |
| conjugate_gaussians_w23 | 249 | 4198 | 0.89 | 140 | - | - | 169 | 213 | - | alea | 2.7e-15 / 6.5e-15 | 3.3e-13 / 6.6e-13 |  |  | ref |
| conjugate_gaussians_w25 | 273 | 4668 | 1.15 | oom | - | - | >600 | >600 | - | tddlog | ref |  |  |  |  |
| conjugate_gaussians_w27 | 297 | 5138 | 1.58 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| conjugate_gaussians_w29 | 321 | 5608 | 1.52 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| fair_dt14_w12 | 1795 | 9432 | 5.12 | 3.91 | 214 | >600 | 0.101 | 69.4 | oom | alea | 1.7e-17 / 5.6e-16 | 1.7e-17 / 5.6e-16 | 1.4e-17 / 2.2e-16 |  | ref |
| fair_dt14_w16 | 3034 | 13225 | 13.2 | 83.4 | oom | - | 0.732 | 72 | - | alea | 5.2e-17 / 1.7e-15 | 5.6e-17 / 8.6e-16 |  |  | ref |
| fair_dt14_w20 | 4259 | 17021 | 34.5 | oom | - | - | 15.9 | 117 | - | alea | 5.6e-17 / 1.0e-15 |  |  |  | ref |
| fair_dt14_w24 | 5272 | 20135 | 41.6 | - | - | - | >600 | >600 | - | tddlog | ref |  |  |  |  |
| fair_dt14_w28 | 6274 | 23227 | 52.5 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| fair_dt14_w32 | 7263 | 26293 | 70.8 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| fair_dt16_w12 | 420 | 2143 | 0.125 | 0.078 | 0.048 | >600 | 0.024 | 69.1 | 3.88 | alea | 2.8e-17 / 2.8e-16 | 2.8e-17 / 2.8e-16 | 2.8e-17 / 2.8e-16 |  | ref |
| fair_dt16_w16 | 638 | 2884 | 0.213 | 0.138 | 0.08 | - | 0.031 | 69.9 | 3.9 | alea | 5.6e-17 / 2.9e-16 | 5.6e-17 / 2.9e-16 | 5.6e-17 / 4.2e-16 |  | ref |
| fair_dt16_w20 | 793 | 3420 | 0.358 | 0.188 | 0.105 | - | 0.034 | 70.2 | 9.17 | alea | 8.3e-17 / 5.6e-16 | 8.3e-17 / 5.6e-16 | 8.3e-17 / 4.4e-16 |  | ref |
| fair_dt16_w24 | 780 | 3394 | 0.356 | 0.18 | 0.102 | - | 0.035 | 69.1 | 8.94 | alea | 5.6e-17 / 2.9e-16 | 5.6e-17 / 2.9e-16 | 4.2e-17 / 8.4e-16 |  | ref |
| fair_dt16_w28 | 754 | 3342 | 0.269 | 0.172 | 0.099 | - | 0.034 | 69.6 | 8.45 | alea | 2.8e-17 / 4.2e-16 | 2.8e-17 / 4.2e-16 | 5.6e-17 / 4.2e-16 |  | ref |
| fair_dt16_w32 | 741 | 3316 | 0.291 | 0.165 | 0.097 | - | 0.035 | 68.6 | 8.18 | alea | 2.8e-17 / 4.2e-16 | 2.8e-17 / 4.2e-16 | 4.2e-17 / 8.4e-16 |  | ref |
| fair_dt4_w12 | 1871 | 9632 | 4.25 | 3.02 | 160 | >600 | 0.092 | 66.4 | 263 | alea | 2.8e-17 / 1.8e-16 | 2.8e-17 / 1.8e-16 | 6.9e-18 / 2.4e-16 |  | ref |
| fair_dt4_w16 | 3064 | 13311 | 13.1 | 65.7 | oom | - | 0.608 | 68.3 | >600 | alea | 2.8e-17 / 1.9e-16 | 2.8e-17 / 1.9e-16 |  |  | ref |
| fair_dt4_w20 | 4349 | 17315 | 30.9 | oom | - | - | 13 | 106 | - | alea | 2.8e-17 / 2.5e-16 |  |  |  | ref |
| fair_dt4_w24 | 5360 | 20425 | 43.4 | - | - | - | >600 | >600 | - | tddlog | ref |  |  |  |  |
| fair_dt4_w28 | 6358 | 23509 | 54.1 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| fair_dt4_w32 | 7345 | 26571 | 71.8 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| fair_dt44_w12 | 478 | 2361 | 0.168 | 0.089 | 0.055 | >600 | 0.026 | 71.7 | 2.04 | alea | 5.6e-17 / 8.1e-16 | 8.3e-17 / 5.8e-16 | 8.3e-17 / 4.8e-16 |  | ref |
| fair_dt44_w16 | 696 | 3090 | 0.231 | 0.151 | 0.092 | - | 0.033 | 73 | 4.67 | alea | 3.5e-17 / 5.8e-16 | 3.5e-17 / 5.8e-16 | 1.4e-17 / 2.3e-16 |  | ref |
| fair_dt44_w20 | 851 | 3614 | 0.329 | 0.198 | 0.12 | - | 0.035 | 72.5 | 8.09 | alea | 1.1e-16 / 6.4e-16 | 1.1e-16 / 6.4e-16 | 5.6e-17 / 8.1e-16 |  | ref |
| fair_dt44_w24 | 838 | 3588 | 0.282 | 0.19 | 0.116 | - | 0.036 | 72 | 7.79 | alea | 8.3e-17 / 5.8e-16 | 8.3e-17 / 5.8e-16 | 8.3e-17 / 4.8e-16 |  | ref |
| fair_dt44_w28 | 812 | 3536 | 0.273 | 0.182 | 0.113 | - | 0.036 | 72.3 | 7.56 | alea | 8.3e-17 / 4.8e-16 | 8.3e-17 / 4.8e-16 | 1.1e-16 / 6.4e-16 |  | ref |
| fair_dt44_w32 | 799 | 3510 | 0.267 | 0.175 | 0.11 | - | 0.038 | 71.4 | 6.94 | alea | 5.6e-17 / 3.2e-16 | 5.6e-17 / 3.2e-16 | 8.3e-17 / 4.8e-16 |  | ref |
| genes_t5 | 67284 | 308661 | 12.4 | >600 | 11.5 | >600 | >600 | >600 | oom | tddlog | ref |  | 0.0e+00 / 0.0e+00 |  |  |
| genes_t10 | 67484 | 309241 | 11.9 | - | 10.8 | - | - | - | - | tddlog | ref |  | 0.0e+00 / 0.0e+00 |  |  |
| genes_t15 | 67684 | 309821 | 12 | - | 10.7 | - | - | - | - | tddlog | ref |  | 0.0e+00 / 0.0e+00 |  |  |
| genes_t20 | 67884 | 310401 | 11.9 | - | 10.6 | - | - | - | - | tddlog | ref |  | 0.0e+00 / 0.0e+00 |  |  |
| genes_t25 | 68084 | 310981 | 11.9 | - | 10.6 | - | - | - | - | tddlog | ref |  | 0.0e+00 / 0.0e+00 |  |  |
| genes_t30 | 68284 | 311561 | 11.9 | - | 10.6 | - | - | - | - | tddlog | ref |  | 0.0e+00 / 0.0e+00 |  |  |
| genes_t40 | 68684 | 312721 | 11.8 | - | 10.5 | - | - | - | - | tddlog | ref |  | 0.0e+00 / 0.0e+00 |  |  |
| genes_t50 | 69084 | 313881 | 11.8 | - | 10.5 | - | - | - | - | tddlog | ref |  | 0.0e+00 / 0.0e+00 |  |  |
| genes_t60 | 69484 | 315041 | 11.8 | - | 10.5 | - | - | - | - | tddlog | ref |  | 0.0e+00 / 0.0e+00 |  |  |
| genes_t80 | 70284 | 317361 | 11.9 | - | 10.5 | - | - | - | - | tddlog | ref |  | 0.0e+00 / 0.0e+00 |  |  |
| genes_t100 | 71084 | 319681 | 12 | - | 10.7 | - | - | - | - | tddlog | ref |  | 0.0e+00 / 0.0e+00 |  |  |
| genes5_w17 | 24148 | 173021 | 6.52 | >600 | 5.99 | zero | 2.22 | 59.6 | oom | alea | 5.5e-15 / 1.1e-14 |  | 5.5e-15 / 1.1e-14 |  | ref |
| genes5_w19 | 31844 | 196665 | 7.61 | - | 6.84 | >600 | 2.96 | 61.9 | - | alea | 7.5e-15 / 1.5e-14 |  | 7.5e-15 / 1.5e-14 |  | ref |
| genes5_w21 | 39336 | 219821 | 8.43 | - | 7.62 | - | 5.52 | 69.4 | - | alea | 6.9e-15 / 1.4e-14 |  | 6.9e-15 / 1.4e-14 |  | ref |
| genes5_w23 | 46572 | 242497 | 9.31 | - | 8.49 | - | 16.8 | 107 | - | alea | 3.7e-15 / 7.3e-15 |  | 3.7e-15 / 7.3e-15 |  | ref |
| genes5_w25 | 53588 | 264781 | 10.1 | - | 9.31 | - | 81.4 | 277 | - | alea | 3.9e-15 / 8.3e-15 |  | 3.9e-15 / 8.3e-15 |  | ref |
| genes5_w27 | 60520 | 286833 | 10.4 | - | 9.83 | - | oom | oom | - | tddlog | ref |  | 0.0e+00 / 0.0e+00 |  |  |
| laplace_w13 | 4125 | 110955 | 6.28 | 141 | 5.94 | >600 | 6.03 | 91.3 | >600 | alea | 4.6e-15 / 9.1e-15 | 4.6e-15 / 9.1e-15 | 4.6e-15 / 9.1e-15 |  | ref |
| laplace_w15 | 8145 | 158877 | >600 | oom | oom | - | 98.4 | 366 | - | alea |  |  |  |  | ref |
| laplace_w17 | 8133 | 205643 | - | - | - | - | oom | oom | - |  |  |  |  |  |  |
| linreg_w8 | 372 | 5404 | 0.131 | 0.332 | 0.247 | >600 | 0.174 | 86.4 | >600 | alea | 3.9e-15 / 2.8e-14 | 4.9e-15 / 2.9e-14 | 4.1e-15 / 2.9e-14 |  | ref |
| linreg_w9 | 564 | 6754 | 0.245 | 1.38 | 0.883 | - | 0.473 | 89.8 | - | alea | 4.4e-15 / 1.5e-14 | 4.0e-15 / 1.4e-14 | 4.7e-15 / 1.6e-14 |  | ref |
| linreg_w10 | 756 | 8204 | 6.82 | 5.31 | oom | - | 1.56 | 90.6 | - | alea | 5.2e-15 / 2.3e-14 | 5.2e-15 / 2.3e-14 |  |  | ref |
| linreg_w11 | 948 | 9788 | 24.5 | 25.2 | - | - | 6.25 | 105 | - | alea | 6.3e-15 / 9.1e-15 | 7.4e-15 / 8.8e-15 |  |  | ref |
| linreg_w12 | 1140 | 11532 | 40 | 101 | - | - | 31.8 | 184 | - | alea | 5.3e-15 / 2.0e-14 | 4.2e-15 / 1.9e-14 |  |  | ref |
| linreg_w13 | 1332 | 13436 | 173 | 479 | - | - | 163 | 251 | - | alea | 8.0e-15 / 2.7e-14 | 2.0e-14 / 3.5e-14 |  |  | ref |
| linreg_w14 | 1524 | 15500 | oom | oom | - | - | >600 | >600 | - |  |  |  |  |  |  |
| markov_n10 | 2050 | 7887 | 0.489 | 0.122 | 2.58 | zero | 0.099 | 87.2 | 4.99 | alea | 1.7e-15 / 9.4e-15 | 1.7e-15 / 9.4e-15 | 1.4e-15 / 1.1e-12 |  | ref |
| markov_n100 | 20500 | 78897 | 3.24 | 2.45 | - | - | 1.38 | 90.6 | 301 | alea | 3.9e-15 / 2.7e-12 | 3.9e-15 / 2.7e-12 |  |  | ref |
| markov_n1000 | 205000 | 788997 | 39.9 | 161 | - | - | 64.6 | 285 | - | alea | 4.1e-13 / 4.1e-13 | 4.1e-13 / 4.1e-13 |  |  | ref |
| markov_n20 | 4100 | 15777 | 0.456 | 0.274 | oom | >600 | 0.255 | 87.8 | 16.8 | alea | 1.2e-14 / 1.5e-14 | 1.2e-14 / 1.8e-14 |  |  | ref |
| markov_n200 | 41000 | 157797 | 10.1 | 8.54 | - | - | 4.11 | 103 | >600 | alea | 1.3e-13 / 9.0e-12 | 1.3e-13 / 9.0e-12 |  |  | ref |
| markov_n2000 | 410000 | 1577997 | oom | oom | - | - | oom | oom | - |  |  |  |  |  |  |
| markov_n50 | 10250 | 39447 | 1.33 | 0.892 | - | - | 0.593 | 89.4 | 83.9 | alea | 2.8e-14 / 2.8e-14 | 2.8e-14 / 2.8e-14 |  |  | ref |
| markov_n500 | 102500 | 394497 | 18.5 | 47.9 | - | - | 16.7 | 139 | - | alea | 1.9e-13 / 4.6e-13 | 1.9e-13 / 4.6e-13 |  |  | ref |
| multimodal_w11 | 4617 | 116721 | oom | oom | oom | >600 | 31.2 | 159 | >600 | alea |  |  |  |  | ref |
| multimodal_w13 | 13837 | 144765 | - | - | - | - | oom | oom | - |  |  |  |  |  |  |
| normal_mixture_mu1_w7 | 2583 | 25704 | 0.964 | 2.62 | 0.791 | >600 | 1.51 | 68.8 | >600 | alea | 2.9e-15 / 2.5e-14 | 2.2e-15 / 2.6e-14 | 3.0e-15 / 2.3e-14 |  | ref |
| normal_mixture_mu1_w9 | 5289 | 35812 | 139 | oom | oom | - | oom | oom | - | tddlog | ref |  |  |  |  |
| normal_mixture_mu1_w11 | 7995 | 45920 | oom | - | - | - | - | - | - |  |  |  |  |  |  |
| normal_mixture_mu2_w7 | 2583 | 45770 | 1.46 | 2.91 | oom | >600 | 1.74 | 69.6 | >600 | alea | 1.3e-14 / 1.1e-13 | 1.1e-14 / 1.1e-13 |  |  | ref |
| normal_mixture_mu2_w9 | 7913 | 63750 | 173 | oom | - | - | oom | oom | - | tddlog | ref |  |  |  |  |
| normal_mixture_mu2_w11 | 13243 | 81730 | oom | - | - | - | - | - | - |  |  |  |  |  |  |
| normal_mixture_theta_w9 | 10537 | 118560 | 31.9 | oom | oom | >600 | oom | oom | >600 | Ganak | 4.4e-16 / 5.2e-16 |  |  |  |  |
| normal_mixture_theta_w11 | 21115 | 152284 | >600 | - | - | - | - | - | - |  |  |  |  |  |  |
| pedestrian_s10 | 89 | 3176 | 1.81 | 3.18 | - | - | 12.3 | 103 | - | alea | 3.3e-16 / 4.3e-14 | 2.2e-16 / 4.3e-14 |  |  | ref |
| pedestrian_s12 | 99 | 3784 | 11.6 | 6.34 | - | - | 26.8 | 147 | - | alea | 4.4e-16 / 4.3e-14 | 5.6e-16 / 4.3e-14 |  |  | ref |
| pedestrian_s16 | 119 | 5000 | 23.4 | 17.2 | - | - | 87.5 | 329 | - | alea | 3.3e-16 / 4.2e-14 | 3.3e-16 / 4.2e-14 |  |  | ref |
| pedestrian_s20 | 139 | 6216 | 74.7 | 39.2 | - | - | 225 | 292 | - | alea | 1.0e-15 / 4.9e-14 | 6.7e-16 / 5.0e-14 |  |  | ref |
| pedestrian_s24 | 159 | 7432 | 190 | 77.7 | - | - | oom | oom | - | tddlog | ref | 3.9e-16 / 8.1e-16 |  |  |  |
| pedestrian_s32 | 199 | 9864 | 407 | 216 | - | - | - | - | - | tddlog | ref | 0.0e+00 / 0.0e+00 |  |  |  |
| pedestrian_s4 | 59 | 1393 | 0.229 | 0.113 | 13.8 | >600 | 0.176 | 65 | >600 | alea | 6.7e-16 / 2.7e-14 | 5.6e-16 / 2.7e-14 | 2.0e-13 / 4.0e-13 |  | ref |
| pedestrian_s6 | 69 | 1968 | 0.691 | 0.473 | >600 | - | 1.39 | 70.5 | - | alea | 2.8e-16 / 4.0e-14 | 2.8e-16 / 4.0e-14 |  |  | ref |
| pedestrian_s8 | 79 | 2568 | 1.79 | 1.35 | - | - | 4.5 | 78.6 | - | alea | 6.7e-16 / 4.1e-14 | 6.7e-16 / 4.1e-14 |  |  | ref |
| pi_w8 | 12 | 3759 | 0.069 | 0.044 | n/a | >600 | 0.048 | 98.7 | 2.36 | alea | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 |  |  | ref |
| pi_w10 | 16 | 6141 | 0.143 | 0.142 | - | - | 0.111 | 99.7 | 27.2 | alea | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 |  |  | ref |
| pi_w12 | 20 | 9099 | 0.415 | 0.614 | - | - | 1.21 | 104 | wrong | alea | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 |  |  | ref |
| pi_w14 | 24 | 12633 | 1.46 | 3.37 | - | - | 28.8 | 189 | >600 | alea | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 |  |  | ref |
| pi_w16 | 28 | 16743 | 6.12 | 22.8 | - | - | 334 | 436 | - | alea | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 |  |  | ref |
| pi_w18 | 32 | 21429 | 43.3 | 174 | - | - | >600 | >600 | - | tddlog | ref | 0.0e+00 / 0.0e+00 |  |  |  |
| pi_w20 | 36 | 26691 | 185 | >600 | - | - | - | - | - | tddlog | ref |  |  |  |  |
| population_s1 | 2540 | 8332 | 0.353 | oom | oom | >600 | 17.4 | 141 | >600 | alea | 1.1e-15 / 9.3e-14 |  |  |  | ref |
| population_s12 | 34957 | 115507 | 18.2 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| population_s16 | 46780 | 154582 | 33.7 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| population_s2 | 5587 | 18373 | 8.29 | - | - | - | oom | oom | - | tddlog | ref |  |  |  |  |
| population_s3 | 8637 | 28421 | 9.33 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| population_s4 | 11311 | 37357 | 10.2 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| population_s6 | 17410 | 57448 | 12.6 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| population_s8 | 23134 | 76432 | 30.7 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| spacex_w13 | 279 | 2037 | 2.27 | 1.01 | oom | >600 | 0.504 | 41.7 | >600 | alea | 4.1e-15 / 6.1e-15 | 4.1e-15 / 6.1e-15 |  |  | ref |
| spacex_w15 | 357 | 2423 | 12.8 | 4.85 | - | - | 3.14 | 48.8 | - | alea | 5.4e-15 / 8.7e-15 | 5.4e-15 / 8.7e-15 |  |  | ref |
| spacex_w17 | 435 | 2809 | 55.7 | 23.2 | - | - | 20.2 | 91.5 | - | alea | 2.8e-14 / 7.0e-14 | 2.8e-14 / 7.0e-14 |  |  | ref |
| spacex_w19 | 513 | 3195 | oom | 113 | - | - | 153 | 326 | - | alea |  | 8.5e-14 / 2.2e-13 |  |  | ref |
| spacex_w21 | 577 | 3553 | - | oom | - | - | >600 | >600 | - |  |  |  |  |  |  |
| spacex_p16 | 293 | 1752 | 104 | 41.2 | oom | - | 78.1 | 232 | - | alea | 6.2e-14 / 1.9e-13 | 6.2e-14 / 1.9e-13 |  |  | ref |
| spacex_p256 | 2970 | 23577 | - | - | - | - | oom | oom | - |  |  |  |  |  |  |
| spacex_p4 | 106 | 635 | 0.042 | 5.46 | 14.5 | >600 | 22 | 97.3 | >600 | alea | 2.2e-16 / 1.7e-14 | 1.1e-14 / 1.9e-14 | 4.4e-14 / 7.7e-14 |  | ref |
| spacex_p64 | 920 | 6092 | oom | oom | - | - | 228 | 268 | - | alea |  |  |  |  | ref |
| trueskill_w8 | 312 | 3868 | 8.95 | 2.55 | oom | >600 | 1.54 | 46.1 | >600 | alea | 2.2e-16 / 4.6e-16 | 2.2e-16 / 4.6e-16 |  |  | ref |
| trueskill_w10 | 376 | 5592 | 193 | 53.1 | - | - | 85.7 | 307 | - | alea | 3.3e-16 / 6.7e-16 | 3.3e-16 / 6.7e-16 |  |  | ref |
| trueskill_w12 | 440 | 7316 | oom | oom | - | - | >600 | >600 | - |  |  |  |  |  |  |
| tug_of_war_w16 | 392 | 2044 | 0.374 | 37 | oom | >600 | 5.54 | 79.4 | >600 | alea | 1.1e-15 / 2.1e-15 | 6.8e-15 / 1.4e-14 |  |  | ref |
| tug_of_war_w18 | 456 | 2292 | 0.448 | 189 | - | - | 30.4 | 149 | - | alea | 2.2e-16 / 4.4e-16 | 6.9e-15 / 1.4e-14 |  |  | ref |
| tug_of_war_w20 | 520 | 2540 | 0.568 | oom | - | - | 159 | 221 | - | alea | 8.3e-16 / 1.7e-15 |  |  |  | ref |
| tug_of_war_w22 | 550 | 2720 | 0.55 | - | - | - | >600 | >600 | - | Ganak | 5.0e-16 / 1.0e-15 |  |  |  |  |
| tug_of_war_w24 | 610 | 2960 | 0.692 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| tug_of_war_w26 | 670 | 3200 | 0.723 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| tug_of_war_w28 | 722 | 3416 | 0.739 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| weekend_w18 | 49151 | 368566 | 15.2 | >600 | 13.5 | 265 | 10.6 | 82.3 | oom | alea | 3.3e-16 / 8.9e-16 |  | 3.3e-16 / 8.9e-16 | 7.3e-14 / 1.9e-13 | ref |
| weekend_w20 | 65535 | 417718 | 16.7 | - | 15 | 355 | 10.8 | 83.9 | - | alea | 2.8e-16 / 7.4e-16 |  | 2.8e-16 / 7.4e-16 | 1.0e-13 / 2.7e-13 | ref |
| weekend_w22 | 81919 | 466870 | 18.6 | - | 16.6 | zero | 10.6 | 85.9 | - | alea | 5.6e-17 / 1.5e-16 |  | 5.6e-17 / 1.5e-16 |  | ref |
| weekend_w24 | 98303 | 516022 | 20.2 | - | 18 | zero | 11.3 | 86.8 | - | alea | 8.3e-16 / 2.2e-15 |  | 8.3e-16 / 2.2e-15 |  | ref |
| weekend_w26 | 114687 | 565174 | 22 | - | 19.4 | zero | 12.2 | 89.3 | - | alea | 9.4e-16 / 2.5e-15 |  | 9.4e-16 / 2.5e-15 |  | ref |
| weekend_w28 | 131071 | 614326 | 23.5 | - | 20.8 | zero | 13.6 | 92 | - | alea | 6.1e-16 / 1.6e-15 |  | 6.1e-16 / 1.6e-15 |  | ref |
| weekend_w30 | 147455 | 663478 | 24.8 | - | 21.7 | zero | 14.2 | 92.5 | - | alea | 1.6e-15 / 4.2e-15 |  | 1.6e-15 / 4.2e-15 |  | ref |
| weekend_p1024 | 28671 | 131010 | 3.72 | >600 | 3.14 | zero | 1.4 | 55.9 | oom | alea | 2.8e-16 / 7.4e-16 |  | 2.8e-16 / 7.4e-16 |  | ref |
| weekend_p16 | 639 | 2118 | 0.114 | 0.055 | 0.051 | zero | 0.017 | 51.6 | 1.04 | alea | 4.4e-16 / 1.2e-15 | 4.4e-16 / 1.2e-15 | 2.8e-16 / 7.4e-16 |  | ref |
| weekend_p256 | 8191 | 33230 | 2.73 | 22.5 | 0.77 | zero | 0.26 | 52.1 | 276 | alea | 1.4e-15 / 3.7e-15 | 1.4e-15 / 3.7e-15 | 1.3e-15 / 3.6e-15 |  | ref |
| weekend_p4 | 175 | 530 | 0.011 | 0.01 | 0.01 | zero | 0.008 | 51.4 | 0.278 | alea | 1.7e-15 / 4.4e-15 | 1.7e-15 / 4.4e-15 | 1.4e-15 / 3.7e-15 |  | ref |
| weekend_p64 | 2303 | 8410 | 1.47 | 0.877 | 0.181 | zero | 0.059 | 52.7 | 10.9 | alea | 5.6e-17 / 1.5e-16 | 5.6e-17 / 1.5e-16 | 2.2e-16 / 5.9e-16 |  | ref |
| wmi_path_n1024 | 8192 | 91060 | 5.47 | 2.69 | - | - | oom | oom | - | tddlog | ref | 1.0e-15 / 1.4e-15 |  |  |  |
| wmi_path_n128 | 1024 | 11316 | 0.626 | 0.326 | - | - | 5.12 | 68.9 | - | alea | 5.6e-16 / 7.8e-16 | 5.6e-16 / 7.8e-16 |  |  | ref |
| wmi_path_n16 | 128 | 1348 | 0.074 | 0.039 | n/a | 414 | 0.058 | 54 | >600 | alea | 2.2e-16 / 3.1e-16 | 2.2e-16 / 3.1e-16 |  | 3.5e-13 / 5.0e-13 | ref |
| wmi_path_n256 | 2048 | 22708 | 1.3 | 0.689 | - | - | 23.3 | 124 | - | alea | 4.6e-15 / 6.4e-15 | 4.6e-15 / 6.4e-15 |  |  | ref |
| wmi_path_n32 | 256 | 2772 | 0.147 | 0.077 | - | >600 | 0.273 | 54.5 | - | alea | 3.3e-16 / 4.7e-16 | 4.4e-16 / 6.3e-16 |  |  | ref |
| wmi_path_n512 | 4096 | 45492 | 2.59 | 1.34 | - | - | 99.2 | 353 | - | alea | 1.5e-14 / 2.1e-14 | 1.5e-14 / 2.1e-14 |  |  | ref |
| wmi_path_n64 | 512 | 5620 | 0.313 | 0.16 | - | - | 1.24 | 58.2 | - | alea | 7.8e-16 / 1.1e-15 | 7.8e-16 / 1.1e-15 |  |  | ref |
| wmi_star_n1024 | 8200 | 91149 | 7.5 | 4 | - | - | oom | oom | - | tddlog | ref | 0.0e+00 / 0.0e+00 |  |  |  |
| wmi_star_n128 | 1032 | 11405 | 0.742 | 0.457 | - | - | 5.55 | 70.5 | 78.3 | alea | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 |  |  | ref |
| wmi_star_n16 | 136 | 1437 | 0.1 | 0.051 | n/a | 305 | 0.065 | 54.4 | wrong | alea | 1.1e-16 / 2.1e-16 | 1.1e-16 / 2.1e-16 |  | 4.1e-14 / 7.6e-14 | ref |
| wmi_star_n256 | 2056 | 22797 | 1.6 | 0.954 | - | - | 37.8 | 172 | 162 | alea | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 |  |  | ref |
| wmi_star_n32 | 264 | 2861 | 0.177 | 0.103 | - | >600 | 0.344 | 55.1 | wrong | alea | 3.3e-16 / 6.4e-16 | 3.3e-16 / 6.4e-16 |  |  | ref |
| wmi_star_n512 | 4104 | 45581 | 3.9 | 1.95 | - | - | 185 | 348 | oom | alea | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 |  |  | ref |
| wmi_star_n64 | 520 | 5709 | 0.357 | 0.217 | - | - | 1.33 | 59.4 | 37.8 | alea | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 |  |  | ref |
| zeroone_w1_w10 | 32 | 2511 | 0.027 | 0.026 | 0.039 | 4.56 | 0.256 | 86.9 | 2.96 | alea | 7.8e-16 / 1.5e-15 | 7.8e-16 / 1.5e-15 | 4.4e-16 / 8.4e-16 | 4.7e-13 / 8.8e-13 | ref |
| zeroone_w1_w12 | 36 | 4184 | 0.088 | 0.054 | 1.62 | 39.6 | 0.389 | 87.4 | 194 | alea | 7.8e-16 / 1.5e-15 | 7.8e-16 / 1.5e-15 | 1.3e-13 / 2.6e-13 | 4.8e-13 / 9.6e-13 | ref |
| zeroone_w1_w14 | 40 | 6254 | 0.131 | 0.102 | 5.86 | 354 | 0.58 | 87.4 | >600 | alea | 1.2e-15 / 2.4e-15 | 1.2e-15 / 2.4e-15 | 2.4e-12 / 4.8e-12 | 4.2e-13 / 8.0e-13 | ref |
| zeroone_w1_w16 | 44 | 8780 | 0.321 | 0.235 | 14.9 | >600 | 1.08 | 91 | - | alea | 3.9e-15 / 7.7e-15 | 1.0e-15 / 2.0e-15 | 3.4e-12 / 6.8e-12 |  | ref |
| zeroone_w1_w18 | 48 | 11762 | 0.748 | 0.543 | 48.1 | - | 2.67 | 95.6 | - | alea | 1.6e-15 / 3.1e-15 | 1.6e-15 / 3.1e-15 | 6.8e-12 / 1.4e-11 |  | ref |
| zeroone_w1_w20 | 52 | 15200 | 1.28 | 1.54 | 82.1 | - | 9.33 | 112 | - | alea | 5.1e-15 / 1.0e-14 | 3.3e-15 / 6.7e-15 | 2.5e-11 / 5.0e-11 |  | ref |
| zeroone_w1_w22 | 56 | 19094 | 2.08 | 5.71 | oom | - | 39 | 200 | - | alea | 6.2e-15 / 1.2e-14 | 4.1e-15 / 8.2e-15 |  |  | ref |
| zeroone_w2_w13 | 38 | 5230 | 0.117 | 0.085 | 0.331 | 131 | 0.517 | 86.8 | >600 | alea | 1.7e-15 / 3.2e-15 | 1.7e-15 / 3.2e-15 | 3.4e-15 / 6.5e-15 | 4.9e-13 / 9.7e-13 | ref |
| zeroone_w2_w15 | 42 | 7560 | 0.237 | 0.171 | 1.05 | >600 | 0.991 | 88.8 | - | alea | 1.0e-14 / 1.4e-14 | 7.2e-15 / 1.0e-14 | 2.8e-15 / 4.6e-15 |  | ref |
| zeroone_w2_w17 | 46 | 10346 | 0.473 | 0.346 | 4.94 | - | 3.81 | 94.5 | - | alea | 2.9e-14 / 5.5e-14 | 2.9e-14 / 5.5e-14 | 4.8e-15 / 9.5e-15 |  | ref |
| zeroone_w2_w19 | 50 | 13588 | 1.28 | 0.909 | 23 | - | 14.2 | 121 | - | alea | 1.1e-13 / 1.6e-13 | 1.1e-13 / 1.6e-13 | 1.7e-15 / 3.3e-15 |  | ref |
| zeroone_w2_w21 | 54 | 17286 | 1.56 | 2.92 | oom | - | 71.5 | 243 | - | alea | 3.7e-15 / 7.3e-15 | 1.2e-12 / 1.9e-12 |  |  | ref |
| zeroone_w2_w23 | 58 | 21440 | 2.53 | 11.9 | - | - | 326 | 410 | - | alea | 1.6e-14 / 3.1e-14 | 5.9e-12 / 8.3e-12 |  |  | ref |
| zeroone_w2_w25 | 62 | 26050 | 6.76 | 49.8 | - | - | oom | oom | - | tddlog | ref | 2.3e-11 / 3.2e-11 |  |  |  |

Answers. The largest differences from the reference: `auto` 1.6e-13
relative (1.1e-13 absolute, zeroone_w2_w19) against Alea, none in common
against ProbLog's eight printed digits, 2.0e-14 relative (1.3e-14
absolute, clickGraph_w14) against Ganak; `program` 8.3e-12 relative
(5.9e-12 absolute, zeroone_w2_w23) and `decision` 5.0e-11 relative
(2.5e-11 absolute, zeroone_w1_w20) against Alea; Dice (earlier run)
1.2e-12 relative (4.7e-13 absolute, clinicalTrial2_w8) against Alea
where it prints answers other than 0.

ProbLog's d-DNNF is wrong on `bb_clickGraph_w14`: it gives 0.68647591
for `e1(3)` with all twelve queries and with that query alone, where
tddlog gives 0.6705662206624511. tddlog's answers converge as the width
grows (0.70508, 0.67611, 0.67178, 0.67085, 0.67062, 0.67057, 0.67055
from 4 to 16 bits), and ProbLog's agree with them to its eight digits up
to 12 bits. An independent count settles it: the circuit's Clark
completion as a weighted CNF, one count for the evidence and one for
each query with it, by Ganak in exact rational arithmetic (`--mode 1
--prob 0`), gives 0.6705662206624644, which tddlog matches to 1e-14
absolute. It is also off the reference on `bb_pi_w12` (by 1.7e-01),
where Alea, tddlog and aspmc agree.

The same counts are the reference where Alea does not answer and tddlog
alone, or tddlog and ProbLog, do (computed on the development machine,
untimed): tddlog agrees with them to 1e-14 relative on clickGraph from
10 to 16 bits, clinicalTrial1 at 10 bits, clinicalTrial2 from 14 to 18
bits, normal_mixture_theta at 9 bits and tug_of_war at 22 bits (on
tug_of_war at 22 bits through the count of the query's negation).
tddlog's answers on addFun_max from 23 to 29 bits, addFun_sum from 25 to
29 bits, coinBias at 22 bits, conjugate_gaussians from 25 to 29 bits,
genes from 5 to 100 genes, genes5 at 27 bits, normal_mixture_mu1 at 9
bits, normal_mixture_mu2 at 9 bits, pi at 18 and 20 bits, tug_of_war
from 24 to 28 bits and zeroone_w2 at 25 bits have no Ganak count and no
other system's answer: their only check is the agreement of tddlog's
arms, to 3e-11 relative, and on addFun_max from 23 to 29 bits,
addFun_sum from 25 to 29 bits, coinBias at 22 bits, conjugate_gaussians
from 25 to 29 bits, normal_mixture_mu1 at 9 bits, normal_mixture_mu2 at
9 bits, pi at 20 bits and tug_of_war from 24 to 28 bits one arm alone
answers.

### Programs of other papers

The suite's other fifteen families (99 instances; with the families above it
holds 38 families and 253 instances), from
other sources (`data/alea/README.md`): the pieces ladders of the paper's Figure 13
(`spacex`, `weekend`, `altermu2` and `addFun_max` by their number of
pieces at Table 2's bits), the laplace mixture of the paper's repository,
FairSquare's four decision trees as SPPL's Table 3 runs them, SPPL's
hierarchical Markov switching model, R2's and PSI's linear regression, the
path and star of the papers on weighted model integration, GuBPI's
pedestrian and Genfer's population model, each written as an Alea.jl
program. They ran in the same job as the paper's programs, in the same
lanes and rotation, `auto`, `program`, `decision` and Alea.jl on an
instance back to back, three repetitions. The largest size each arm
answers (median seconds; for Alea its warm time, as the papers time it),
and the failure at the next size (`oom` an allocation that failed, `>T` a
timeout, `n/a` no guard to decide between; `k/3` an instance answered
in k repetitions; a size is `wW` bits wide, `nN` a size, `sS` steps or
seasons, `pP` pieces):

| program | source | auto | program | decision | Alea |
|---|---|---:|---:|---:|---:|
| addFun_max (pieces) | Fig. 13 | p256 (287 s), >600 at p1024 | oom at p4 | oom at p4 | >600 at p4 |
| altermu2 (pieces) | Fig. 13 | p64 (14.5 s), >600 at p1024 | p4 (102 s), oom at p16 | oom at p4 | oom at p4 |
| fair_dt14 | FairSquare | w32 (70.8 s) | w16 (83.4 s), oom at w20 | w12 (214 s), oom at w16 | w20 (15.9 s), >600 at w24 |
| fair_dt16 | FairSquare | w32 (0.291 s) | w32 (0.165 s) | w32 (0.097 s) | w32 (0.035 s) |
| fair_dt4 | FairSquare | w32 (71.8 s) | w16 (65.7 s), oom at w20 | w12 (160 s), oom at w16 | w20 (13 s), >600 at w24 |
| fair_dt44 | FairSquare | w32 (0.267 s) | w32 (0.175 s) | w32 (0.11 s) | w32 (0.038 s) |
| laplace | bit-blasting repo | w13 (6.28 s), >600 at w15 | w13 (141 s), oom at w15 | w13 (5.94 s), oom at w15 | w15 (98.4 s), oom at w17 |
| linreg | R2, PSI | w13 (173 s), oom at w14 | w13 (479 s), oom at w14 | w9 (0.883 s), oom at w10 | w13 (163 s), >600 at w14 |
| markov | SPPL | n1000 (39.9 s), oom at n2000 | n1000 (161 s), oom at n2000 | n10 (2.58 s), oom at n20 | n1000 (64.6 s), oom at n2000 |
| pedestrian | GuBPI | s32 (407 s) | s32 (216 s) | s4 (13.8 s), >600 at s6 | s20 (225 s), oom at s24 |
| population | Genfer | s16 (33.7 s) | oom at s1 | oom at s1 | s1 (17.4 s), oom at s2 |
| spacex (pieces) | Fig. 13 | p16 (104 s), oom at p64 | p16 (41.2 s), oom at p64 | p4 (14.5 s), oom at p16 | p64 (228 s), oom at p256 |
| weekend (pieces) | Fig. 13 | p1024 (3.72 s) | p256 (22.5 s), >600 at p1024 | p1024 (3.14 s) | p1024 (1.4 s) |
| wmi_path | WMI | n1024 (5.47 s) | n1024 (2.69 s) | n/a at n16 | n512 (99.2 s), oom at n1024 |
| wmi_star | WMI | n1024 (7.5 s) | n1024 (4 s) | n/a at n16 | n512 (185 s), oom at n1024 |

- `auto` answers a larger size than Alea on eight families (addFun_max (pieces), altermu2 (pieces), fair_dt14, fair_dt4, pedestrian, population, wmi_path and wmi_star), the same size on five (fair_dt16, fair_dt44, linreg, markov and weekend (pieces)) and a smaller one on two (laplace and spacex (pieces)).
- `auto` and Alea both answer 59 instances: `auto` takes 668 s in all and Alea's warm time 1.16e+03 s; of the 52 where either takes 0.2 s or more, `auto` is faster on 20.
- `program` and Alea both answer 55 instances: `program` takes 1.28e+03 s in all and Alea's warm time 1.12e+03 s; of the 37 where either takes 0.2 s or more, `program` is faster on 18.
- `decision` and Alea both answer 25 instances: `decision` takes 418 s in all and Alea's warm time 31.3 s; of the 10 where either takes 0.2 s or more, `decision` is faster on 2.
- 417 answers of the arms were checked against Alea's where it answers; the largest difference is 4.1e-13 absolute.
- The population model's answers are the bits of a number; its expectation at 1, 2, 3, 4, 6, 8, 12 and 16 seasons agrees with a forward filter of the model to 7.5e-07 (24 answers).

`auto` is slower than Alea's warm time, or fails where Alea answers, on 38 instances where either takes 0.1 s or more (failures: laplace_w15 >600, spacex_p64 oom). The twelve largest ratios:

| instance | auto | Alea |
|---|---:|---:|
| laplace_w15 | >600 | 98.4 |
| fair_dt14_w12 | 5.12 | 0.101 |
| fair_dt4_w12 | 4.25 | 0.092 |
| spacex_p64 | oom | 228 |
| weekend_p64 | 1.47 | 0.059 |
| fair_dt4_w16 | 13.1 | 0.608 |
| fair_dt14_w16 | 13.2 | 0.732 |
| fair_dt16_w20 | 0.358 | 0.034 |
| weekend_p256 | 2.73 | 0.26 |
| fair_dt16_w24 | 0.356 | 0.035 |
| fair_dt44_w20 | 0.329 | 0.035 |
| fair_dt16_w32 | 0.291 | 0.035 |

### The earlier run: Dice, ProbLog and the samplers

Dice 1.0 with an unlimited stack and Alea.jl as above, in one job on a
lab machine (16 physical cores, 125 GB) on 2026-10-12, in 5 lanes of 25
GB a process, with the same rotation, timeout and skip rule, three
repetitions. ProbLog's d-DNNF ran in a pass of its own (300 s) as a
second reference. The samplers (Stan, WebPPL's rejection, MCMC and SMC)
and Psi ran in the same job, in the same lanes, between the repetitions.
Dice, Alea.jl and the circuits have not changed since; the tddlog of
that run is not reported.

- Dice answers 17 of the 152 instances. Against Alea in the same run,
  over the 44 instances given to both that either answers, a run that
  does not answer counted at the 600 s limit, Dice takes 17701 s in all
  and Alea 464 s; over the 17 both answer, 1501 s against 101 s. Dice is
  faster on 0 of the 17 both answer and Alea on 16, both under 0.2 s or
  level on the other 1. Dice alone answers 0, Alea alone 27. It prints 0
  for every answer on clinicalTrial1 at 4 and 6 bits, genes5 at 17 bits
  and weekend from 22 to 30 bits, where the evidence's probability is
  within the doubles (from 3.5e-62 on clinicalTrial1 at 6 bits to 4.2e-7
  on weekend at 22 bits, by tddlog's `--evidence-probability`): the
  zeros are Dice's, not an underflow.
- The two runs' times are not compared, but Alea's, the same system in
  both, show how far two runs differ: on the 80 instances it answers in
  both, its warm time totals 3733 s in the release run against 3297 s in
  the earlier run, and is from 0.75 to 1.60 times the earlier run's
  where both take a second or more.
- earlyoom, the machine's out-of-memory guard, ended one repetition with
  a SIGTERM when the machine's free memory ran low, after 389 s (dice on
  addFun_max_w17), on a circuit where the other repetitions ran out of
  memory or time: the lanes' limits add up to nearly all of the
  machine's memory, so lanes together can run it low before one reaches
  its own limit. It counts as a failure.

The paper's other systems, at its precision. The samplers run the
artifact's own programs of each model (baselines.md, section 10) at 10^2
(Stan 10^3) to 10^7 samples, ten times more each run, three seeds, until
a run fails or takes more than 60 s. A cell gives the median time over
the seeds at the smallest sample count whose median error is within the
target, with that count and error; else (`err`) the smallest median
error reached, its sample count and time. The target is the error of the
exact systems' answer, the circuit's answer against the continuous
model's (the paper's ground truth, from its notebook); where no exact
system answers at the paper's precision it is `none`, and the cell gives
the smallest error. The exact systems' columns are the release run's
(`auto` and Alea), at the paper's precision. Psi's column is its time
and whether it returns a closed form or leaves an integral.

| program | truth | target | tddlog | Alea | Stan | WebPPL rej. | WebPPL MCMC | WebPPL SMC | Psi |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| pi | 0.464602 | 1e-4 | 1.46 | 28.8 | 15.3 (1e6, 6e-5) | 2.2 (1e6, 9e-5) | err 2e-4 (1e7, 27.1 s) | err 2e-3 (1e5, 1.51 s) | 0.252, integral left |
| weekend | 0.374206 | 2e-8 | 20.2 | 11.3 | other model (1) | other model (2) | other model (2) | other model (2) | 0.083, closed form |
| spacex | 30.0046 | 7e-4 | memout | 153 | err 2e-3 (1e6, 14.5 s) | err 3e-3 (1e6, 7.16 s) | err 1e-2 (1e6, 9.37 s) | err 2e-2 (1e5, 3.07 s) | 0.177, integral left |
| GPA | 0.611511 | 2e-16 | 19.7 | 8.68 | n/a | err 7e-3 (1e3, 243 s) | err 2e-3 (1e4, 208 s) | err 8e-3 (1e3, 8.24 s) | 1.39, closed form |
| tug_of_war | 0.5 | 5e-7 | 0.55 | >600 | err 4e-5 (1e7, 51.9 s) | err 4e-4 (1e6, 431 s) | err 7e-4 (1e7, 97.5 s) | err 2e-4 (1e5, 114 s) | 0.537, integral left |
| altermu2 | 0.155062 | none | memout | memout | err 7e-3 (1e6, 45.4 s) | >600 | err 8e-1 (1e7, 390 s) | err 6e-1 (1e4, 18.4 s) | 24.8, closed form |
| conjugate_gaussians | 1 | 5e-6 | 0.89 | 169 | err 3e-4 (1e6, 9.01 s) | err 2e-4 (1e6, 70.7 s) | err 3e-4 (1e7, 67.5 s) | err 2e-3 (1e5, 42.3 s) | 0.019, closed form |
| normal_mixture_theta | 0.285714 | 5e-5 | 31.9 | memout | 35.9 (1e6, 4e-5) | >600 | err 4e-4 (1e7, 270 s) | err 1e-3 (1e4, 12.2 s) | >600 |
| normal_mixture_mu1 | -9.70236 | 5e-3 | 139 | memout | 0.432 (1e4, 4e-3) | >600 | 27.7 (1e6, 2e-3) | err 1e-2 (1e4, 12.4 s) | >600 |
| normal_mixture_mu2 | 9.65795 | 4e-3 | 173 | memout | 0.207 (1e3, 3e-3) | >600 | 27.5 (1e6, 2e-3) | err 3e-2 (1e4, 12.4 s) | >600 |
| zeroone_w1 | 0.0565823 | 9e-5 | 0.321 | 1.08 | err 2e-1 (1e5, 102 s) | other model (3) | other model (3) | other model (3) | >600 |
| zeroone_w2 | 3.68883 | 5e-4 | 1.28 | 14.2 | err 1e-1 (1e5, 102 s) | other model (4) | other model (4) | other model (4) | >600 |
| coinBias | 0.416667 | 2e-7 | >600 | memout | err 1e-4 (1e7, 31.5 s) | err 1e-4 (1e6, 181 s) | err 2e-4 (1e6, 5.7 s) | err 6e-4 (1e5, 164 s) | 0.221, closed form |
| addFun_sum | 0 | 4e-6 | 0.708 | 432 | err 1e-3 (1e6, 9.79 s) | err 7e-4 (1e6, 7.28 s) | err 9e-4 (1e7, 80.3 s) | err 3e-3 (1e5, 2.27 s) | n/a |
| clickGraph | 0.614154 | 2e-3 | 0.32 | memout | 0.442 (1e4, 6e-4) | 21.6 (1e4, 2e-3) | 114 (1e7, 1e-3) | err 4e-3 (1e5, 365 s) | n/a |
| trueskill | 0.5 | 3e-3 | 193 | 85.7 | 1.36 (1e5, 1e-3) | 5.91 (1e5, 4e-4) | 15.2 (1e6, 3e-3) | 17.7 (1e5, 2e-3) | 0.154, integral left |
| clinicalTrial1 | 0.170362 | 5e-16 | 1.73 | 263 | err 2e-3 (1e6, 311 s) | failed | failed | failed | >600 |
| clinicalTrial2 | 0.285714 | 7e-7 | 0.022 | 308 | err 1e-4 (1e7, 86.8 s) | other model (5) | other model (5) | other model (5) | 0.727, closed form |
| addFun_max | 0.56419 | 3e-7 | 37.2 | memout | err 6e-4 (1e6, 9.96 s) | err 7e-4 (1e6, 6.78 s) | err 4e-4 (1e7, 67.6 s) | err 1e-3 (1e5, 2.03 s) | 0.028, integral left |
| genes_5 | 0.5 | defect | 12.4 | >600 | err 2e-4 (1e6, 119 s) | >600 | err 3e-4 (1e7, 86.9 s) | err 4e-3 (1e4, 1.63 s) | 4.77, closed form |
| genes_10 | 0.5 | defect | 11.9 | skipped | err 2e-4 (1e5, 572 s) | >600 | err 2e-4 (1e7, 134 s) | err 2e-4 (1e5, 5.36 s) | 75.2, closed form |
| genes_15 | 0.5 | defect | 12 | skipped | err 1e-3 (1e3, 365 s) | >600 | err 1e-3 (1e7, 204 s) | err 4e-4 (1e4, 2.06 s) | >600 |
| genes_20 | 0.5 | defect | 11.9 | skipped | >600 | >600 | err 5e-4 (1e7, 251 s) | err 8e-4 (1e5, 8.62 s) | >600 |
| genes_25 | 0.5 | defect | 11.9 | skipped | failed | >600 | err 2e-3 (1e7, 313 s) | err 5e-4 (1e5, 10.1 s) | >600 |
| genes_30 | 0.5 | defect | 11.9 | skipped | >600 | >600 | err 2e-3 (1e7, 372 s) | err 1e-3 (1e5, 12 s) | memout |
| genes_40 | 0.5 | defect | 11.8 | skipped | failed | >600 | err 2e-3 (1e7, 492 s) | err 7e-4 (1e5, 15.9 s) | >600 |
| genes_50 | 0.5 | defect | 11.8 | skipped | failed | >600 | err 3e-3 (1e6, 65.3 s) | err 2e-4 (1e4, 3.84 s) | >600 |
| multimodal | 3.88889 | none | skipped | memout | err 9e0 (1e5, 1.5 s) | >600 | err 1e0 (1e2, 1.06 s) | err 1e-1 (1e4, 3.54 s) | n/a |
| conjugate_ex3 | 5.66667 | 7e-3 | skipped | 280 | 0.907 (1e5, 1e-3) | >600 | err 9e-1 (1e7, 47.8 s) | err 1e0 (1e5, 32.8 s) | n/a |

(1) The artifact's Stan program truncates the hours at 0 only, not to
[0, 8]; that model's answer is 0.33074 (Table 2 marks Stan unsupported
here). (2) The artifact's WebPPL program leaves out the truncation of
the hours to [0, 8]; that model's answer is 0.38995. (3) The artifact's
WebPPL program puts hard constraints of the opposite sign in place of
the 1/e penalty, and no weights meet them. (4) The artifact's WebPPL
program puts hard constraints of the opposite sign in place of the 1/e
penalty and maps a two-argument function over one list, so every point
gets the same label; that model's answer is 5.335. (5) The artifact's
WebPPL program maps a two-argument function over the pairs, so every
observation reads as a failure; that model's answer is 1/7.

Alea's discretization in the far tails. Alea.jl's `bitblast` and
`bitblast_exponential` give a piece of a distribution the mass of its
interval from Distributions.jl's `cdf` (0.25.131, StatsFuns 2.2.1), a
difference of two values near 1 in the upper tail, which loses the mass
there below double precision and comes out as 0 or as a coarse multiple
of 2^-53. A scan of every distribution the programs bit-blast, at every
precision of the ladders, with Alea's own pieces against an exact
integral of each piece (not timed), finds pieces off by more than 10% in
every program but clickGraph, clinicalTrial1, clinicalTrial2, pi,
weekend, zeroone_w1 and zeroone_w2 (the scan of GPA is incomplete).
Where an observation lies in such a tail, the answer changes:

- The gene expression program observes two blood sugar readings, 79 and
  136, each normal around 135 or 80 with standard deviation 8 by whether
  any gene occurs; the model finds them equally likely either way, so
  its answer is the prior's mean, 1/2. Where no gene occurs, 136 lies 7
  standard deviations up N(80, 8), the one reading among the defective
  pieces, and from 14 bits after the point the circuit makes the
  readings far likelier without a gene than with one, by a factor of
  1.9e3 at 14 bits, 7.8e3 at 16, 3.1e4 at 18 and 1.2e5 at 20 (the factor
  that gives the answers below), twice as much with each bit: the true
  probability of a point halves with each bit and the circuit's does
  not. The answer moves toward 1/3, the first gene's expected
  probability given that it does not occur, as far as the prior
  probability that no gene occurs, 2^-T, lets it. With 5 genes Alea
  gives 0.4991 at 12 bits, 0.3360 at 14, 0.3340 at 16, 0.3335 at 18
  (`bb_genes5_w23`, `bb_genes5_w25` and `bb_genes5_w27`). At Figure 2's
  precision, 20 bits after the point, tddlog alone finishes
  (`bb_genes_tT`, checked only by its `auto` and `decision` arms, which
  agree exactly): 0.33338 with 5 genes, 0.33470 with 10, 0.36813 with
  15, 0.48235 with 20, 0.49938 with 25, 0.49998 with 30, 0.49999950 with
  40, and from 50 genes on 0.4999995, the mean of the prior's grid,
  2^-21 below 1/2. The artifact's own runs behind Figure 2 (Alea.jl's
  `hybrid/benchmarks/or/results_T.txt`, at the same bits and pieces)
  give 0.4999995 at every T from 5 to 50. The exact systems are scored
  against the circuit's own answer, not against 1/2.
- The conjugate Gaussians of Figure 5 observe 8 and 9 under a prior N(0,
  1), each with variance 1; the posterior N(17/3, 1/3) lies 5.7 standard
  deviations up the prior, whose pieces are defective from about 6.7 at
  16 bits. Alea's answer moves away from 17/3 as the precision grows:
  5.66653 at 10 bits after the point, 5.66567 at 12, 5.66330 at 14,
  5.65919 at 16.

## Roulette's programs

`bench.py --suite roulette` holds the scaling programs of Roulette's
evaluation (Moy, Czenszak, Li, Marshall and Holtzen, PLDI 2025; the
system and how it runs, baselines.md section 11; the drivers,
data/roulette/README.md): the motivating Markov chain, diamonds and
ladders in a row, the Caesar cipher, the hidden Markov model with its
five queries, the lossy network and the perturbed-memory interpreter,
each on a ladder from the artifact's sizes up to sizes where the systems
run out of time, and the artifact's nine Bayesian networks with the
variable it queries: 88 instances in 12 families. `gen_roulette.py`
writes each program's ProbLog and Dice versions with the same
probabilities; Alea.jl runs the Dice programs (as on the Dice paper's
programs), and Roulette the artifact's own programs. The paper runs the
network and the interpreter in Roulette only; the Dice programs here
hold the message as a flag and four Booleans and unroll the
interpreter's loop. The reference is a dynamic program over each model's
states (the clique tree on the networks), `--exact`, in a pass of its
own.

### Results: tddlog 0.1.0, Dice, Alea.jl and Roulette

One job on a cloud machine (96 physical cores, 1488 GB) on 2026-10-19,
52 lanes of 28 GB, each process limited to its lane's memory. tddlog
0.1.0 in its three arms, Dice 1.0 with an unlimited stack, Alea.jl as on
the bit-blasted programs and Roulette (camoy/roulette 54d6185 on Racket
8.16, with `nocomplement` on caesar) run on an instance back to back in
one lane, in an order rotated by instance and repetition; three
repetitions, 600 s each, and the family skip rule. Roulette's time is
the one its own `time` reports for building and querying the program;
`Roulette process` adds Racket's start-up and loading.

Of the 88 instances, none lie past the sizes at which every system had
failed, and none every system answers in under a second. Answers: `auto`
88, `program` 14, `decision` 0, Dice 40, Alea 61, Roulette 40.

| family | sizes | `auto` | `program` | `decision` | Dice | Alea | Roulette |
|---|---|---:|---:|---:|---:|---:|---:|
| motivating | 1 000 to 1 000 000 | 1 000 000 (56.2 s) | memout | unsupported | 10 000 (196 s) | 100 000 (49.9 s) | 10 000 (0.75 s) |
| diamond | 1 000 to 1 000 000 | 1 000 000 (56.4 s) | memout | unsupported | 10 000 (189 s) | 100 000 (42.5 s) | 10 000 (2 s) |
| ladder | 1 000 to 1 000 000 | 1 000 000 (105 s) | memout | unsupported | 10 000 (423 s) | 100 000 (43.4 s) | 10 000 (2.55 s) |
| caesar | 25 to 20 000 | 20 000 (83.3 s) | memout | unsupported | zero, underflow | 1 000 (99.5 s) | wrong, underflow, error |
| hmm marginal likelihood | 100 to 900 | 900 (0.099 s) | memout | unsupported | 900 (0.51 s) | 900 (1.33 s) | 900 (1.49 s) |
| hmm filtering marginal | 100 to 100 000 | 100 000 (12.7 s) | memout | unsupported | zero, underflow, timeout | 3 000 (18.9 s) | 300 (0.18 s) |
| hmm smoothing marginal | 100 to 100 000 | 100 000 (13.1 s) | memout | unsupported | zero, underflow, timeout | 3 000 (19 s) | 300 (0.18 s) |
| hmm filtering joint | 100 to 100 000 | 100 000 (14.8 s) | memout | unsupported | 10 000 (87 s) | 3 000 (20 s) | 3 000 (30.9 s) |
| hmm smoothing joint | 100 to 100 000 | 100 000 (14.6 s) | memout | unsupported | 10 000 (96.9 s) | 3 000 (22 s) | 3 000 (27.9 s) |
| network | 10 to 30 000 | 30 000 (15.5 s) | memout | unsupported | 30 000 (238 s) | 1 000 (67.5 s) | 300 (19.8 s) |
| hardware | 10 to 10 000 | 10 000 (11.8 s) | 1 000 (68.2 s) | unsupported | 10 000 (9.62 s) | 1 000 (108 s) | 300 (19.8 s) |
| bn | 9 networks | 9 (0.12 s) | 9 (5.04 s) | - | 8 (20.1 s) | 9 (2.77 s) | 9 (5.17 s) |

The table above gives, a family a row, the largest size each system
answers and its time there (on the networks, how many it answers and
their total time), or its failures where it answers no size.

- Against `program`, over the 25 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, `auto`
  takes 4.81 s in all and `program` 6680 s; over the 14 both answer,
  1.44 s against 79.7 s. `auto` is faster on 5 of the 14 both answer and
  `program` on 0, both under 0.2 s or level on the other 9. `auto` alone
  answers 11, `program` alone 0.
- `decision` answers none of these programs: tddlog refuses it on 12
  (unsupported), and the larger sizes of their families are skipped.
- Against Dice, over the 75 instances given to both that either answers,
  a run that does not answer counted at the 600 s limit, `auto` takes
  224 s in all and Dice 22431 s; over the 40 both answer, 43.6 s against
  1431 s. `auto` is faster on 25 of the 40 both answer and Dice on 3,
  both under 0.2 s or level on the other 12. `auto` alone answers 35,
  Dice alone 0.
- Against Alea, over the 71 instances given to both that either answers,
  a run that does not answer counted at the 600 s limit, `auto` takes
  112 s in all and Alea 6560 s; over the 61 both answer, 36.3 s against
  560 s. `auto` is faster on 34 of the 61 both answer and Alea on 0,
  both under 0.2 s or level on the other 27. `auto` alone answers 10,
  Alea alone 0. Against Alea's process, over the 71 instances given to
  both that either answers, a run that does not answer counted at the
  600 s limit, `auto` takes 112 s in all and the process 9244 s; over
  the 61 both answer, 36.3 s against 3244 s.
- Against Roulette, over the 65 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, `auto`
  takes 88.8 s in all and Roulette 15118 s; over the 40 both answer,
  3.83 s against 118 s. `auto` is faster on 21 of the 40 both answer and
  Roulette on 0, both under 0.2 s or level on the other 19. `auto` alone
  answers 25, Roulette alone 0. Against Roulette's process, over the 65
  instances given to both that either answers, a run that does not
  answer counted at the 600 s limit, `auto` takes 88.8 s in all and the
  process 15137 s; over the 40 both answer, 3.83 s against 137 s.
- Dice prints a wrong answer (wrong) on 1 instance: bn_insurance.
- Dice prints 0 for every answer (zero) on 11 instances: caesar_25,
  caesar_50, caesar_100, hmm_filtering_joint_100,
  hmm_filtering_joint_300, hmm_filtering_marginal_100,
  hmm_filtering_marginal_300, hmm_smoothing_joint_100,
  hmm_smoothing_joint_300, hmm_smoothing_marginal_100,
  hmm_smoothing_marginal_300.
- Dice prints 0 for every answer (underflow) on 16 instances:
  caesar_200, caesar_300, caesar_500, caesar_1000, caesar_2000,
  caesar_5000, caesar_10000, caesar_20000, hmm_filtering_joint_1000,
  hmm_filtering_marginal_1000, hmm_filtering_marginal_3000,
  hmm_filtering_marginal_10000, hmm_smoothing_joint_1000,
  hmm_smoothing_marginal_1000, hmm_smoothing_marginal_3000,
  hmm_smoothing_marginal_10000.
- Roulette prints a wrong answer (wrong) on 3 instances: caesar_25,
  caesar_50, caesar_100.
- Roulette prints 0 for every answer (underflow) on 12 instances:
  caesar_200, caesar_300, caesar_500, caesar_1000, caesar_2000,
  caesar_5000, hmm_filtering_joint_1000, hmm_filtering_marginal_1000,
  hmm_filtering_marginal_3000, hmm_smoothing_joint_1000,
  hmm_smoothing_marginal_1000, hmm_smoothing_marginal_3000.

| instance | tddlog | program | decision | Dice | Alea | Roulette | Roulette #f | Alea process | Roulette process | ProbLog | ref | tddlog diff | program diff | decision diff | Dice diff | Alea diff | Roulette diff | Roulette #f diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| bn_alarm | 0.005 | 0.011 | - | 0.027 | 0.012 | 0.042 |  | 9.77 | 0.472 | 0.481 | exact | 6.7e-16 / 2.5e-14 | 5.6e-16 / 2.2e-15 |  | 2.8e-17 / 1.3e-16 | 1.1e-16 / 7.6e-16 | 1.1e-16 / 4.4e-16 |  |
| bn_cancer | 0.003 | 0.003 | n/a | 0.017 | 0.006 | 0.001 |  | 9.18 | 0.446 | 0.255 | exact | 1.1e-16 / 5.3e-16 | 1.1e-16 / 5.3e-16 |  | 2.8e-17 / 1.3e-16 | 1.1e-16 / 1.4e-16 | 0.0e+00 / 0.0e+00 |  |
| bn_hailfinder | 0.026 | 1.9 | - | 1.21 | 0.937 | 1.02 |  | 11.9 | 1.45 |  | exact | 3.9e-16 / 1.3e-15 | 3.3e-16 / 7.6e-16 |  | 4.9e-13 / 1.6e-12 | 5.6e-17 / 2.2e-16 | 1.7e-16 / 6.6e-16 |  |
| bn_hepar2 | 0.009 | 0.177 | - | 0.051 | 0.034 | 0.156 |  | 9.7 | 0.574 | - | exact | 1.7e-16 / 3.9e-16 | 5.6e-17 / 1.3e-16 |  | 4.7e-13 / 1.1e-12 | 1.1e-16 / 1.9e-16 | 2.5e-10 / 5.8e-10 |  |
| bn_insurance | 0.014 | 2.79 | - | wrong | 1.24 | 0.724 |  | 11.8 | 1.18 | >600 | exact | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 |  | 7.4e-06 / 4.4e-04 | 1.5e-11 / 2.7e-11 | 2.4e-11 / 1.4e-09 |  |
| bn_munin | 0.035 | 0.051 | - | 5.64 | 0.345 | 2.75 |  | 18.9 | 3.17 |  | exact | 4.5e-16 / 1.9e-13 | 6.4e-16 / 3.4e-13 |  | 4.3e-13 / 2.1e-12 | 1.1e-16 / 1.7e-14 | 9.6e-09 / 3.5e-06 |  |
| bn_pigs | 0.006 | 0.006 | - | 0.196 | 0.04 | 0.271 |  | 10.9 | 0.683 |  | exact | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 5.6e-17 / 2.2e-16 | 0.0e+00 / 0.0e+00 |  |
| bn_survey | 0.004 | 0.004 | - | 0.018 | 0.008 | 0.002 |  | 9.49 | 0.453 | 0.143 | exact | 2.5e-16 / 1.6e-15 | 1.4e-16 / 8.8e-16 |  | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.0e-16 | 1.1e-16 / 4.0e-16 |  |
| bn_water | 0.014 | 0.094 | - | 12.9 | 0.15 | 0.211 |  | 11.6 | 0.626 |  | exact | 2.8e-16 / 2.7e-14 | 5.0e-16 / 1.8e-14 |  | 4.6e-13 / 1.4e-12 | 1.1e-16 / 3.4e-16 | 7.2e-09 / 9.6e-08 |  |
| caesar_25 | 0.073 | oom | n/a | zero | 0.047 | wrong | 0.146 | 10.1 | wrong | 28.8 | exact | 4.9e-15 / 1.1e-13 |  |  |  | 7.8e-15 / 4.3e-13 | 5.2e-01 / 1.1e+00 | 4.6e-15 / 1.1e-13 |
| caesar_50 | 0.142 | - | - | zero | 0.137 | wrong | 0.259 | 10.3 | wrong | 61.4 | exact | 2.3e-15 / 2.2e-13 |  |  |  | 5.3e-15 / 8.5e-13 | 9.8e-01 / 4.1e+01 | 2.4e-15 / 2.2e-13 |
| caesar_100 | 0.283 | - | - | zero | 0.672 | wrong | 0.537 | 10.8 | wrong | 113 | exact | 5.2e-26 / 4.8e-13 |  |  |  | 4.4e-15 / 2.3e-12 | 1.0e+00 / 1.7e+12 | 1.1e-16 / 1.0e+00 |
| caesar_200 | 0.599 | - | - | underflow | 2.66 | underflow | 1.15 | 12.8 | underflow | 253 | exact | 1.4e-29 / 1.5e-12 |  |  |  | 1.8e-29 / 5.5e-12 |  | 9.6e-18 / 1.0e+00 |
| caesar_300 | 0.911 | - | - | underflow | 6.11 | underflow | underflow | 16.1 | underflow |  | exact | 3.6e-31 / 4.0e-12 |  |  |  | 1.2e-30 / 9.3e-12 |  |  |
| caesar_500 | 1.59 | - | - | underflow | 18.6 | underflow | underflow | 28.8 | underflow |  | exact | 2.4e-34 / 2.5e-12 |  |  |  | 1.8e-33 / 1.5e-11 |  |  |
| caesar_1000 | 3.36 | - | - | underflow | 99.5 | underflow | underflow | 110 | underflow |  | exact | 1.3e-78 / 3.2e-11 |  |  |  | 6.8e-79 / 7.4e-11 |  |  |
| caesar_2000 | 7.36 | - | - | underflow | oom | underflow | underflow | oom | underflow |  | exact | 1.0e-171 / 6.4e-11 |  |  |  |  |  |  |
| caesar_5000 | 19.2 | - | - | underflow | - | underflow | underflow | - | underflow |  | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |
| caesar_10000 | 39.4 | - | - | underflow | - | err | err | - | err |  | exact | 4.4e-16 / 4.4e-16 |  |  |  |  |  |  |
| caesar_20000 | 83.3 | - | - | underflow | - | - | - | - | - |  | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |
| diamond_1000 | 0.035 | oom | n/a | 1.61 | 0.065 | 0.171 |  | 9.52 | 0.591 | 17 | exact | 1.1e-16 / 1.2e-16 |  |  | 4.1e-14 / 4.3e-14 | 1.1e-14 / 1.1e-14 | 2.2e-16 / 2.3e-16 |  |
| diamond_3000 | 0.101 | - | - | 14.4 | 0.178 | 0.515 |  | 11.2 | 0.903 | stack | exact | 1.1e-15 / 1.3e-15 |  |  | 3.9e-13 / 4.6e-13 | 3.1e-14 / 3.6e-14 | 1.8e-15 / 2.1e-15 |  |
| diamond_10000 | 0.386 | - | - | 189 | 0.639 | 2 |  | 17.5 | 2.37 | - | exact | 7.7e-15 / 1.3e-14 |  |  | 4.5e-14 / 7.4e-14 | 1.0e-14 / 1.6e-14 | 1.8e-14 / 2.9e-14 |  |
| diamond_30000 | 1.25 | - | - | >600 | 2.39 | err |  | 45.9 | err | - | exact | 4.2e-14 / 1.9e-13 |  |  |  | 4.6e-13 / 2.1e-12 |  |  |
| diamond_100000 | 4.58 | - | - | - | 42.5 | - |  | 425 | - | - | exact | 9.1e-14 / 1.3e-11 |  |  |  | 9.5e-16 / 1.4e-13 |  |  |
| diamond_300000 | 15.3 | - | - | - | >600 | - |  | >600 | - | - | exact | 5.1e-19 / 1.7e-12 |  |  |  |  |  |  |
| diamond_1000000 | 56.4 | - | - | - | - | - |  | - | - | - | exact | 1.1e-33 / 5.5e-12 |  |  |  |  |  |  |
| hardware_10 | 0.011 | 0.01 | n/a | 0.023 | 0.014 | 0.016 |  | 10.3 | 0.425 | err | exact | 3.3e-16 / 1.2e-14 | 4.4e-16 / 1.2e-14 |  | 1.8e-14 / 2.1e-12 | 1.1e-16 / 1.4e-14 | 2.2e-16 / 1.2e-12 |  |
| hardware_30 | 0.028 | 0.063 | - | 0.037 | 0.044 | 0.137 |  | 10.7 | 0.54 | - | exact | 2.2e-16 / 1.1e-14 | 4.4e-16 / 1.1e-14 |  | 1.7e-14 / 3.3e-12 | 1.1e-16 / 7.1e-14 | 1.1e-16 / 1.1e-12 |  |
| hardware_100 | 0.085 | 0.639 | - | 0.11 | 0.722 | 1.86 |  | 11.4 | 2.31 | - | exact | 1.1e-16 / 1.1e-14 | 4.4e-16 / 1.1e-14 |  | 1.7e-14 / 3.2e-12 | 1.1e-16 / 1.9e-12 | 1.1e-16 / 3.7e-12 |  |
| hardware_300 | 0.264 | 5.83 | - | 0.287 | 8.25 | 19.8 |  | 19 | 20.6 | - | exact | 3.3e-16 / 1.1e-14 | 4.4e-16 / 1.1e-14 |  | 1.7e-14 / 3.2e-12 | 1.1e-16 / 1.9e-15 | 1.1e-16 / 4.5e-13 |  |
| hardware_1000 | 0.937 | 68.2 | - | 0.853 | 108 | oom |  | 121 | oom | - | exact | 2.2e-16 / 1.1e-14 | 4.4e-16 / 1.1e-14 |  | 1.7e-14 / 3.2e-12 | 1.1e-16 / 1.9e-15 |  |  |
| hardware_3000 | 3.07 | oom | - | 2.68 | oom | - |  | oom | - | - | exact | 2.2e-16 / 1.1e-14 |  |  | 1.7e-14 / 3.2e-12 |  |  |  |
| hardware_10000 | 11.8 | - | - | 9.62 | - | - |  | - | - | - | exact | 1.1e-16 / 1.1e-14 |  |  | 1.7e-14 / 3.2e-12 |  |  |  |
| hmm_filtering_joint_100 | 0.013 | oom | n/a | zero | 0.019 | 0.024 |  | 9.09 | 0.489 | 0.635 | exact | 5.8e-32 / 1.5e-13 |  |  |  | 2.1e-32 / 5.7e-14 | 5.9e-32 / 1.6e-13 |  |
| hmm_filtering_joint_300 | 0.032 | - | - | zero | 0.152 | 0.213 |  | 9.33 | 0.668 | 2.05 | exact | 3.9e-69 / 2.9e-14 |  |  |  | 1.7e-67 / 1.3e-12 | 5.5e-69 / 4.0e-14 |  |
| hmm_filtering_joint_1000 | 0.096 | - | - | underflow | 1.77 | underflow |  | 148 | underflow | 13 | exact | 5.6e-194 / 1.4e-11 |  |  |  | 6.1e-194 / 1.5e-11 |  |  |
| hmm_filtering_joint_3000 | 0.316 | - | - | 6.93 | 20 | 30.9 |  | 167 | 31.9 | oom | exact | 0.0e+00 / 0.0e+00 |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |
| hmm_filtering_joint_10000 | 1.22 | - | - | 87 | oom | oom |  | oom | oom | - | exact | 0.0e+00 / 0.0e+00 |  |  | 0.0e+00 / 0.0e+00 |  |  |  |
| hmm_filtering_joint_30000 | 4.05 | - | - | >600 | - | - |  | - | - | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |
| hmm_filtering_joint_100000 | 14.8 | - | - | - | - | - |  | - | - | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |
| hmm_filtering_marginal_100 | 0.025 | oom | n/a | zero | 0.019 | 0.021 |  | 8.96 | 0.485 | 0.531 | exact | 6.2e-15 / 1.0e-14 |  |  |  | 0.0e+00 / 0.0e+00 | 6.0e-15 / 9.6e-15 |  |
| hmm_filtering_marginal_300 | 0.038 | - | - | zero | 0.143 | 0.185 |  | 9.07 | 0.655 | 1.88 | exact | 6.0e-15 / 9.6e-15 |  |  |  | 0.0e+00 / 0.0e+00 | 6.2e-15 / 1.0e-14 |  |
| hmm_filtering_marginal_1000 | 0.103 | - | - | underflow | 1.7 | underflow |  | 10.9 | underflow | 12.3 | exact | 3.0e-14 / 4.9e-14 |  |  |  | 7.1e-14 / 1.1e-13 |  |  |
| hmm_filtering_marginal_3000 | 0.303 | - | - | underflow | 18.9 | underflow |  | 28.7 | underflow | stack | exact | 1.1e-13 / 1.8e-13 |  |  |  | 2.8e-13 / 4.5e-13 |  |  |
| hmm_filtering_marginal_10000 | 1.13 | - | - | underflow | oom | oom |  | oom | oom | - | exact | 4.0e-13 / 6.4e-13 |  |  |  |  |  |  |
| hmm_filtering_marginal_30000 | 3.78 | - | - | >600 | - | - |  | - | - | - | exact | 9.7e-13 / 1.6e-12 |  |  |  |  |  |  |
| hmm_filtering_marginal_100000 | 12.7 | - | - | - | - | - |  | - | - | - | exact | 3.3e-12 / 5.2e-12 |  |  |  |  |  |  |
| hmm_marginal_likelihood_100 | 0.014 | oom | n/a | 0.024 | 0.017 | 0.019 |  | 8.97 | 0.494 | oom | exact | 1.6e-48 / 5.5e-14 |  |  | 3.1e-47 / 1.1e-12 | 0.0e+00 / 0.0e+00 | 1.7e-48 / 6.0e-14 |  |
| hmm_marginal_likelihood_300 | 0.035 | - | - | 0.076 | 0.143 | 0.149 |  | 9.06 | 0.614 | - | exact | 2.6e-116 / 1.2e-12 |  |  | 2.6e-116 / 1.2e-12 | 0.0e+00 / 0.0e+00 | 2.7e-116 / 1.2e-12 |  |
| hmm_marginal_likelihood_600 | 0.067 | - | - | 0.259 | 0.616 | 0.654 |  | 146 | 1.11 | - | exact | 3.2e-219 / 6.9e-12 |  |  | 3.6e-219 / 7.8e-12 | 2.6e-221 / 5.7e-14 | 3.2e-219 / 7.0e-12 |  |
| hmm_marginal_likelihood_900 | 0.099 | - | - | 0.505 | 1.33 | 1.49 |  | 147 | 1.95 | - | exact | 1.2e-322 / 1.3e-11 |  |  | 1.2e-322 / 1.3e-11 | 0.0e+00 / 0.0e+00 | 1.2e-322 / 1.3e-11 |  |
| hmm_smoothing_joint_100 | 0.013 | oom | n/a | zero | 0.019 | 0.022 |  | 9.27 | 0.5 | 0.602 | exact | 1.3e-31 / 2.0e-13 |  |  |  | 3.6e-32 / 5.7e-14 | 1.3e-31 / 2.0e-13 |  |
| hmm_smoothing_joint_300 | 0.032 | - | - | zero | 0.143 | 0.208 |  | 9.37 | 0.674 | 2.4 | exact | 3.1e-67 / 1.3e-12 |  |  |  | 3.1e-67 / 1.4e-12 | 3.1e-67 / 1.4e-12 |  |
| hmm_smoothing_joint_1000 | 0.099 | - | - | underflow | 1.7 | underflow |  | 11.3 | underflow | 16.9 | exact | 2.0e-193 / 2.9e-11 |  |  |  | 1.0e-193 / 1.5e-11 |  |  |
| hmm_smoothing_joint_3000 | 0.325 | - | - | 7.99 | 22 | 27.9 |  | 169 | 28.8 | oom | exact | 0.0e+00 / 0.0e+00 |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |
| hmm_smoothing_joint_10000 | 1.22 | - | - | 96.9 | oom | oom |  | oom | oom | - | exact | 0.0e+00 / 0.0e+00 |  |  | 0.0e+00 / 0.0e+00 |  |  |  |
| hmm_smoothing_joint_30000 | 3.93 | - | - | >600 | - | - |  | - | - | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |
| hmm_smoothing_joint_100000 | 14.6 | - | - | - | - | - |  | - | - | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |
| hmm_smoothing_marginal_100 | 0.023 | oom | n/a | zero | 0.018 | 0.02 |  | 8.89 | 0.483 | 0.513 | exact | 5.6e-17 / 1.2e-16 |  |  |  | 0.0e+00 / 0.0e+00 | 5.6e-17 / 1.2e-16 |  |
| hmm_smoothing_marginal_300 | 0.037 | - | - | zero | 0.143 | 0.177 |  | 9.07 | 0.649 | 1.78 | exact | 6.3e-15 / 1.4e-14 |  |  |  | 0.0e+00 / 0.0e+00 | 6.2e-15 / 1.4e-14 |  |
| hmm_smoothing_marginal_1000 | 0.102 | - | - | underflow | 1.68 | underflow |  | 10.9 | underflow | 10.9 | exact | 3.2e-14 / 7.1e-14 |  |  |  | 0.0e+00 / 0.0e+00 |  |  |
| hmm_smoothing_marginal_3000 | 0.302 | - | - | underflow | 19 | underflow |  | 28.9 | underflow | oom | exact | 3.2e-14 / 7.1e-14 |  |  |  | 0.0e+00 / 0.0e+00 |  |  |
| hmm_smoothing_marginal_10000 | 1.14 | - | - | underflow | oom | oom |  | oom | oom | - | exact | 1.7e-13 / 3.8e-13 |  |  |  |  |  |  |
| hmm_smoothing_marginal_30000 | 3.91 | - | - | >600 | - | - |  | - | - | - | exact | 6.5e-13 / 1.4e-12 |  |  |  |  |  |  |
| hmm_smoothing_marginal_100000 | 13.1 | - | - | - | - | - |  | - | - | - | exact | 2.6e-12 / 5.8e-12 |  |  |  |  |  |  |
| ladder_1000 | 0.065 | oom | n/a | 3.49 | 0.08 | 0.198 |  | 10.5 | 0.598 | >600 | exact | 1.9e-15 / 6.2e-15 |  |  | 4.6e-13 / 1.5e-12 | 1.2e-15 / 3.8e-15 | 2.6e-15 / 8.4e-15 |  |
| ladder_3000 | 0.218 | - | - | 32.1 | 0.3 | 0.614 |  | 12.9 | 0.967 | - | exact | 9.2e-16 / 8.2e-15 |  |  | 3.0e-13 / 2.7e-12 | 2.4e-15 / 2.2e-14 | 6.9e-16 / 6.2e-15 |  |
| ladder_10000 | 0.782 | - | - | 423 | 1.14 | 2.55 |  | 23.6 | 2.91 | - | exact | 6.2e-15 / 1.8e-12 |  |  | 4.3e-15 / 1.3e-12 | 1.8e-15 / 5.4e-13 | 1.8e-15 / 5.3e-13 |  |
| ladder_30000 | 2.53 | - | - | >600 | 5.59 | err |  | 76.9 | err | - | exact | 1.3e-22 / 8.7e-16 |  |  |  | 5.6e-19 / 3.7e-12 |  |  |
| ladder_100000 | 9.12 | - | - | - | 43.4 | - |  | 445 | - | - | exact | 1.5e-37 / 1.6e-15 |  |  |  | 1.8e-33 / 1.8e-11 |  |  |
| ladder_300000 | 28.8 | - | - | - | >600 | - |  | >600 | - | - | exact | 3.3e-80 / 9.6e-15 |  |  |  |  |  |  |
| ladder_1000000 | 105 | - | - | - | - | - |  | - | - | - | exact | 1.3e-232 / 4.2e-15 |  |  |  |  |  |  |
| motivating_1000 | 0.035 | oom | n/a | 1.5 | 0.036 | 0.065 |  | 9.14 | 0.475 | 5.51 | exact | 5.6e-16 / 2.0e-15 |  |  | 2.7e-13 / 1.0e-12 | 5.6e-17 / 2.0e-16 | 0.0e+00 / 0.0e+00 |  |
| motivating_3000 | 0.097 | - | - | 14.4 | 0.146 | 0.211 |  | 10.4 | 0.574 | stack | exact | 5.6e-16 / 2.0e-15 |  |  | 2.7e-13 / 1.0e-12 | 5.6e-17 / 2.0e-16 | 0.0e+00 / 0.0e+00 |  |
| motivating_10000 | 0.363 | - | - | 196 | 0.484 | 0.753 |  | 15 | 1.12 | - | exact | 5.6e-16 / 2.0e-15 |  |  | 2.7e-13 / 1.0e-12 | 5.6e-17 / 2.0e-16 | 0.0e+00 / 0.0e+00 |  |
| motivating_30000 | 1.23 | - | - | >600 | 1.74 | err |  | 36.8 | err | - | exact | 5.6e-16 / 2.0e-15 |  |  |  | 5.6e-17 / 2.0e-16 |  |  |
| motivating_100000 | 4.49 | - | - | - | 49.9 | - |  | 524 | - | - | exact | 5.6e-16 / 2.0e-15 |  |  |  | 5.6e-17 / 2.0e-16 |  |  |
| motivating_300000 | 15.1 | - | - | - | >600 | - |  | >600 | - | - | exact | 5.6e-16 / 2.0e-15 |  |  |  |  |  |  |
| motivating_1000000 | 56.2 | - | - | - | - | - |  | - | - | - | exact | 5.6e-16 / 2.0e-15 |  |  |  |  |  |  |
| network_10 | 0.007 | oom | n/a | 0.031 | 0.017 | 0.029 |  | 13.1 | 0.422 | oom | exact | 3.3e-16 / 3.0e-14 |  |  | 2.3e-13 / 3.8e-12 | 3.3e-16 / 7.1e-15 | 1.1e-16 / 1.7e-15 |  |
| network_30 | 0.015 | - | - | 0.071 | 0.045 | 0.197 |  | 12.9 | 0.591 | - | exact | 2.2e-16 / 3.8e-15 |  |  | 3.0e-13 / 3.4e-12 | 1.9e-16 / 2.3e-14 | 1.8e-15 / 2.4e-15 |  |
| network_100 | 0.041 | - | - | 0.264 | 0.584 | 1.89 |  | 13.4 | 2.3 | - | exact | 4.4e-16 / 3.1e-15 |  |  | 2.7e-13 / 6.9e-13 | 1.2e-15 / 5.7e-14 | 1.7e-15 / 4.6e-15 |  |
| network_300 | 0.117 | - | - | 0.977 | 5.41 | 19.8 |  | 18.2 | 20.6 | - | exact | 1.7e-15 / 3.7e-15 |  |  | 2.5e-13 / 1.4e-12 | 6.3e-16 / 7.7e-14 | 8.5e-16 / 1.3e-14 |  |
| network_1000 | 0.399 | - | - | 4.41 | 67.5 | oom |  | 80.7 | oom | - | exact | 2.2e-16 / 2.6e-14 |  |  | 1.7e-13 / 1.0e-12 | 7.8e-16 / 4.0e-13 |  |  |
| network_3000 | 1.27 | - | - | 14 | oom | - |  | oom | - | - | exact | 1.1e-16 / 5.7e-14 |  |  | 4.4e-13 / 2.1e-12 |  |  |  |
| network_10000 | 4.68 | - | - | 64 | - | - |  | - | - | - | exact | 1.1e-14 / 2.3e-13 |  |  | 1.1e-14 / 3.9e-12 |  |  |  |
| network_30000 | 15.5 | - | - | 238 | - | - |  | - | - | - | exact | 1.1e-14 / 5.0e-13 |  |  | 1.1e-14 / 3.9e-13 |  |  |  |

Answers. The largest differences from the reference: `auto` 6.4e-11
relative (1.0e-171 absolute, caesar_2000); `program` 2.7e-11 relative
(1.5e-11 absolute, bn_insurance); Dice 1.3e-11 relative (1.2e-322
absolute, hmm_marginal_likelihood_900); Alea 7.4e-11 relative (6.8e-79
absolute, caesar_1000); Roulette 3.5e-06 relative (9.6e-09 absolute,
bn_munin); Roulette with `nocomplement` 1.0e+00 relative (1.1e-16
absolute, caesar_100).

## Geni's programs

`bench.py --suite geni` holds the programs of Geni's evaluation (Li and
Zhang, "Compiling with Generating Functions", ICFP 2025) that Dice runs,
as the paper's artifact writes them (`scripts/gen_geni.py`,
data/geni/README.md): the grid networks of its Figure 11, an N x N grid
of routers whose links each work with probability 0.5 (Fierens et al.'s
grid, on which ProbLog was evaluated), P(a packet from the top-left
router reaches the bottom-right one), and a grid whose routers each work
with a probability that depends on their left and upper neighbours,
P(the bottom-right router works); and Rubicon's translation (Holtzen et
al., CAV 2021) of the weather factory of its Figure 9, P(all n factories
are on strike within h days), by n at 10 days and by h with 10
factories. 57 instances in 4 families. Each is a Dice program, which
`dice2problog.py` translates for tddlog and ProbLog and `dice2alea.py`
for Alea.jl and `dice2roulette.py` for Roulette; Geni runs the same
programs as its own `.gir` files (`gen_geni.py --gir`, the artifact's
flags), and Storm and PRISM the weather factory as its PRISM model. The
reference is a dynamic program over each model's states (`gen_geni.py
--reference-of`, `--exact`), in a pass of its own.

### Results: tddlog 0.1.0, ProbLog, Dice, Alea.jl and Roulette

One job on a cloud machine (96 physical cores, 1488 GB) on 2026-10-19,
49 lanes of one core each, each process limited to 30 GB. tddlog 0.1.0
`auto`, ProbLog 2.3.0 (d-DNNF), Dice 1.0 with an unlimited stack and
Alea.jl as on the bit-blasted programs, and Roulette (the artifact's,
54d6185, on Racket 8.16, variant `rsdd`, the translated program compiled
by `raco make` before it runs, untimed, as the artifact's programs are;
its time is the one it reports for building and querying the program;
the process's is in a column of its own), and Geni (gennifer; Li and
Zhang, ICFP 2025), noDice (Gürtler and Kaminski, OOPSLA 2026), Psi (Gehr
et al., CAV 2016; revision ef8a9e5), Genfer (Zaiser, Murawski and Ong,
NeurIPS 2023), SPPL 2.0.4 (Saad, Rinard and Mansinghka, PLDI 2021),
Pluck (Bowers et al., PLDI 2025), PITA (cplint, SWI-Prolog 9.2.9), aspmc
1.1.1 with sharpSAT-TD, Storm 1.14 (stormpy), one Sylvan thread
(`sparse`, `exact`, `dd`), PRISM 4.8.1 (`sparse`, `hybrid`), Ace 3.0
(`auto`, `forceC2d`), Merlin 1.7.0's bucket-tree elimination (`-a bte`),
pyAgrum's junction tree, pgmpy's variable elimination, libDAI's junction
tree and TensorInference.jl's tensor network contraction (`greedy`,
`treesa`) (the process's seconds) run on an instance back to back in one
lane, in an order rotated by instance and repetition; three repetitions,
600 s each. A ladder runs smallest first in one lane, and a system that
fails on an instance skips the ladder's larger ones; the expensive tops
of the ladders (the weather factory from 15 factories and from 100 days,
the grids above 20) ran as ladders of their own, so a system that failed
below them ran them to its own failure.

Of the 57 instances, two lie past the sizes at which every system had
failed, and none every system answers in under a second. Answers: tddlog
44, ProbLog 10, Dice 43, Alea 27, Roulette 22, Geni 39, noDice 12, Psi
0, Genfer 11, SPPL 5, Pluck 9, PITA 11, aspmc 29, Storm sparse 39, Storm
exact 25, Storm dd 34, PRISM sparse 43, PRISM hybrid 22, Ace 31, Ace c2d
28, Merlin 16, pyAgrum 30, pgmpy 20, libDAI 31, TensorInference.jl
greedy 34, TensorInference.jl treesa 35.

| program | sizes | tddlog | ProbLog | Dice | Alea | Roulette | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Merlin | pyAgrum | pgmpy | libDAI | TensorInference.jl greedy | TensorInference.jl treesa |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| grid, link failures | N = 4 to 32 | 14 (22.4 s) | 6 (5.59 s) | 16 (331 s) | 14 (132 s) | 12 (41.3 s) | 16 (494 s) | 14 (418 s) | timeout at 4 | 4 (5.61 s) | 4 (12.2 s) | 6 (6.4 s) | 6 (0.61 s) | 14 (364 s) | 14 (254 s) | 12 (45.3 s) | 12 (26.6 s) | 16 (441 s) | 16 (529 s) | 12 (19.1 s) | 12 (271 s) | 12 (8.04 s) | 12 (2.04 s) | 14 (275 s) | 12 (7.27 s) | 12 (136 s) | 18 (540 s) |
| grid, router failures | N = 4 to 40 | 20 (4.45 s) | 6 (11.7 s) | 16 (77.7 s) | 14 (101 s) | 12 (13.3 s) | 16 (103 s) | 14 (149 s) | timeout at 4 | 18 (309 s) | 6 (44.6 s) | 8 (7.61 s) | 6 (1.4 s) | 14 (536 s) | 14 (90 s) | 14 (108 s) | 32 (413 s) | 40 (241 s) | 40 (240 s) | 16 (88.6 s) | 14 (223 s) | 16 (24.9 s) | 16 (6.52 s) | 20 (5.97 s) | 18 (57.6 s) | 20 (245 s) | 18 (589 s) |
| weather factory, by factories | 2 to 19 factories | 16 (17.9 s) | 2 (3.93 s) | 16 (385 s) | 11 (216 s) | 9 (66.2 s) | 11 (3.32 s) | error at 2 | error at 2 | 2 (0.14 s) | timeout at 2 | 2 (7.43 s) | 2 (0.56 s) | 13 (176 s) | 13 (138 s) | 10 (158 s) | 11 (179 s) | 10 (13.4 s) | 8 (386 s) | 13 (15.5 s) | 13 (345 s) | 9 (0.85 s) | 13 (7.72 s) | 8 (593 s) | 13 (16.5 s) | 17 (373 s) | 16 (559 s) |
| weather factory, by days | 1 to 1000 days | 1 000 (65.1 s) | 5 (51 s) | 1 000 (381 s) | 9 (56.8 s) | 8 (76.4 s) | 1 000 (162 s) | error at 1 | error at 1 | 1 (0.006 s) | 2 (134 s) | 3 (7.69 s) | 6 (165 s) | 9 (16.8 s) | 1 000 (5.95 s) | 9 (131 s) | 200 (560 s) | 1 000 (16 s) | timeout at 1 | 200 (69.6 s) | 9 (30.7 s) | zero at 1 | 100 (169 s) | 2 (1.82 s) | 100 (344 s) | 9 (69.5 s) | 9 (295 s) |

The table above gives, a program a row, the largest size each system
answers and its time there, or its failure at the smallest size it ran.

- Against ProbLog, over the 18 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 59.6 s in all and ProbLog 4892 s; over the 10 both answer, 0.32
  s against 91.6 s. tddlog is faster on 7 of the 10 both answer and
  ProbLog on 0, both under 0.2 s or level on the other 3. tddlog alone
  answers 8, ProbLog alone 0.
- Against Dice, over the 44 instances given to both that either answers,
  a run that does not answer counted at the 600 s limit, tddlog takes
  1024 s in all and Dice 3269 s; over the 42 both answer, 420 s against
  2338 s. tddlog is faster on 28 of the 42 both answer and Dice on 0,
  both under 0.2 s or level on the other 14. tddlog alone answers 1,
  Dice alone 1.
- Against Alea, over the 33 instances given to both that either answers,
  a run that does not answer counted at the 600 s limit, tddlog takes 88
  s in all and Alea 4363 s; over the 27 both answer, 27.3 s against 763
  s. tddlog is faster on 17 of the 27 both answer and Alea on 0, both
  under 0.2 s or level on the other 10. tddlog alone answers 6, Alea
  alone 0. Against the Julia process, over the 33 instances given to
  both that either answers, a run that does not answer counted at the
  600 s limit, tddlog takes 88 s in all and the process 4679 s; over the
  27 both answer, 27.3 s against 1079 s.
- Against Roulette, over the 30 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 85.7 s in all and Roulette 5119 s; over the 22 both answer, 3.31
  s against 319 s. tddlog is faster on 14 of the 22 both answer and
  Roulette on 0, both under 0.2 s or level on the other 8. tddlog alone
  answers 8, Roulette alone 0.
- Against Geni, over the 44 instances given to both that either answers,
  a run that does not answer counted at the 600 s limit, tddlog takes
  1024 s in all and Geni 4627 s; over the 38 both answer, 376 s against
  1133 s. tddlog is faster on 29 of the 38 both answer and Geni on 0,
  both under 0.2 s or level on the other 9. tddlog alone answers 5, Geni
  alone 1.
- Against noDice, over the 19 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 84.6 s in all and noDice 4853 s; over the 12 both answer, 24.6 s
  against 653 s. tddlog is faster on 8 of the 12 both answer and noDice
  on 0, both under 0.2 s or level on the other 4. tddlog alone answers
  7, noDice alone 0.
- Against Psi, over the 8 instances given to both that either answers, a
  run that does not answer counted at the 600 s limit, tddlog takes 59.1
  s in all and Psi 4800 s. tddlog alone answers 8, Psi alone 0.
- Against Genfer, over the 19 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 68.7 s in all and Genfer 5190 s; over the 11 both answer, 5.11 s
  against 390 s. tddlog is faster on 6 of the 11 both answer and Genfer
  on 0, both under 0.2 s or level on the other 5. tddlog alone answers
  8, Genfer alone 0.
- Against SPPL, over the 13 instances given to both that either answers,
  a run that does not answer counted at the 600 s limit, tddlog takes
  59.2 s in all and SPPL 4993 s; over the 5 both answer, 0.029 s against
  193 s. tddlog is faster on 5 of the 5 both answer and SPPL on 0.
  tddlog alone answers 8, SPPL alone 0.
- Against Pluck, over the 17 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 59.4 s in all and Pluck 4856 s; over the 9 both answer, 0.088 s
  against 55.8 s. tddlog is faster on 9 of the 9 both answer and Pluck
  on 0. tddlog alone answers 8, Pluck alone 0.
- Against PITA, over the 19 instances given to both that either answers,
  a run that does not answer counted at the 600 s limit, tddlog takes
  59.9 s in all and PITA 4994 s; over the 11 both answer, 0.48 s against
  194 s. tddlog is faster on 11 of the 11 both answer and PITA on 0.
  tddlog alone answers 8, PITA alone 0.
- Against aspmc, over the 34 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 91.6 s in all and aspmc 4516 s; over the 29 both answer, 31.6 s
  against 1516 s. tddlog is faster on 29 of the 29 both answer and aspmc
  on 0. tddlog alone answers 5, aspmc alone 0.
- Against Storm sparse, over the 42 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 420 s in all and Storm sparse 2439 s; over the 39 both answer,
  380 s against 639 s. tddlog is faster on 23 of the 39 both answer and
  Storm sparse on 10, both under 0.2 s or level on the other 6. tddlog
  alone answers 3, Storm sparse alone 0.
- Against Storm exact, over the 32 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 87.3 s in all and Storm exact 5105 s; over the 25 both answer,
  4.19 s against 905 s. tddlog is faster on 21 of the 25 both answer and
  Storm exact on 0, both under 0.2 s or level on the other 4. tddlog
  alone answers 7, Storm exact alone 0.
- Against Storm dd, over the 38 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 732 s in all and Storm dd 5181 s; over the 31 both answer, 34.9
  s against 1457 s. tddlog is faster on 30 of the 31 both answer and
  Storm dd on 0, both under 0.2 s or level on the other 1. tddlog alone
  answers 6, Storm dd alone 1.
- Against PRISM sparse, over the 44 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 1624 s in all and PRISM sparse 3648 s; over the 38 both answer,
  379 s against 779 s. tddlog is faster on 30 of the 38 both answer and
  PRISM sparse on 8. tddlog alone answers 4, PRISM sparse alone 2.
- Against PRISM hybrid, over the 26 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 1293 s in all and PRISM hybrid 5554 s; over the 17 both answer,
  29.5 s against 797 s. tddlog is faster on 17 of the 17 both answer and
  PRISM hybrid on 0. tddlog alone answers 7, PRISM hybrid alone 2.
- Against Ace, over the 35 instances given to both that either answers,
  a run that does not answer counted at the 600 s limit, tddlog takes
  99.8 s in all and Ace 2705 s; over the 31 both answer, 31 s against
  305 s. tddlog is faster on 31 of the 31 both answer and Ace on 0.
  tddlog alone answers 4, Ace alone 0.
- Against Ace c2d, over the 32 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 60.1 s in all and Ace c2d 3791 s; over the 28 both answer, 9.2 s
  against 1391 s. tddlog is faster on 28 of the 28 both answer and Ace
  c2d on 0. tddlog alone answers 4, Ace c2d alone 0.
- Against Merlin, over the 33 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 63.9 s in all and Merlin 10237 s; over the 16 both answer, 3.6 s
  against 37.2 s. tddlog is faster on 9 of the 16 both answer and Merlin
  on 0, both under 0.2 s or level on the other 7. tddlog alone answers
  17, Merlin alone 0.
- Against pyAgrum, over the 34 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 79.1 s in all and pyAgrum 2603 s; over the 30 both answer, 15.8
  s against 203 s. tddlog is faster on 28 of the 30 both answer and
  pyAgrum on 0, both under 0.2 s or level on the other 2. tddlog alone
  answers 4, pyAgrum alone 0.
- Against pgmpy, over the 22 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 34.3 s in all and pgmpy 2281 s; over the 20 both answer, 33.9 s
  against 1081 s. tddlog is faster on 19 of the 20 both answer and pgmpy
  on 1. tddlog alone answers 2, pgmpy alone 0.
- Against libDAI, over the 35 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 83.5 s in all and libDAI 2867 s; over the 31 both answer, 19.5 s
  against 467 s. tddlog is faster on 24 of the 31 both answer and libDAI
  on 0, both under 0.2 s or level on the other 7. tddlog alone answers
  4, libDAI alone 0.
- Against TensorInference.jl greedy, over the 36 instances given to both
  that either answers, a run that does not answer counted at the 600 s
  limit, tddlog takes 686 s in all and TensorInference.jl greedy 4185 s;
  over the 33 both answer, 58.1 s against 2612 s. tddlog is faster on 33
  of the 33 both answer and TensorInference.jl greedy on 0. tddlog alone
  answers 2, TensorInference.jl greedy alone 1.
- Against TensorInference.jl treesa, over the 36 instances given to both
  that either answers, a run that does not answer counted at the 600 s
  limit, tddlog takes 686 s in all and TensorInference.jl treesa 9527 s;
  over the 33 both answer, 76.1 s against 7979 s. tddlog is faster on 33
  of the 33 both answer and TensorInference.jl treesa on 0. tddlog alone
  answers 2, TensorInference.jl treesa alone 1.
- Roulette answers 22 of the 35 instances it ran; it runs out of memory
  on 13.
- Geni prints 0 for every answer (zero) on 5 instances: weather_n12,
  weather_n13, weather_n15, weather_n16, weather_n17.
- Merlin prints 0 for every answer (zero) on 13 instances: weather_h1,
  weather_h2, weather_h3, weather_h4, weather_h5, weather_h6,
  weather_h7, weather_h8, weather_h9, weather_n10, weather_n11,
  weather_n12, weather_n13.

| instance | tddlog | ProbLog | Dice | Alea | Roulette | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Merlin | pyAgrum | pgmpy | libDAI | TI greedy | TI treesa | Alea process | Roulette process | ref | tddlog diff | ProbLog diff | Dice diff | Alea diff | Roulette diff | Geni diff | noDice diff | Psi diff | Genfer diff | SPPL diff | Pluck diff | PITA diff | aspmc diff | Storm sparse diff | Storm exact diff | Storm dd diff | PRISM sparse diff | PRISM hybrid diff | Ace diff | Ace c2d diff | Merlin diff | pyAgrum diff | pgmpy diff | libDAI diff | TI greedy diff | TI treesa diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| link_4 | 0.004 | 0.092 | 0.017 | 0.006 | 0.001 | 0.007 | 0.017 | >600 | 5.61 | 12.2 | 5.19 | 0.435 | 2.59 | 0.155 | 0.156 | 0.408 | 1.27 | 1.26 | 1.3 | 1.33 | 0.237 | 1.45 | 19 | 0.237 | 23.1 | 28.2 | 8.32 | 0.368 | exact | 0.0e+00 / 0.0e+00 | 3.1e-09 / 5.6e-09 | 4.3e-13 / 7.5e-13 | 1.1e-16 / 2.0e-16 | 0.0e+00 / 0.0e+00 | 6.9e-09 / 1.2e-08 | 9.3e-08 / 1.6e-07 |  | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.0e-16 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 9.3e-08 / 1.6e-07 | 4.3e-13 / 7.5e-13 | 4.3e-13 / 7.5e-13 | 4.3e-13 / 7.5e-13 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| link_6 | 0.008 | 5.59 | 0.021 | 0.011 | 0.017 | 0.031 | 0.065 | - | oom | >600 | 6.4 | 0.615 | 2.63 | 0.188 | 0.202 | 1.18 | 1.28 | 1.28 | 1.45 | 1.6 | 0.066 | 0.226 | 1.32 | 0.07 | 34.6 | 51.8 | 8.72 | 0.406 | exact | 1.1e-16 / 2.2e-16 | 2.6e-09 / 5.1e-09 | 2.2e-13 / 4.3e-13 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.3e-08 / 2.5e-08 | 3.9e-07 / 7.6e-07 |  |  |  | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 3.9e-07 / 7.6e-07 | 2.2e-13 / 4.3e-13 | 2.2e-13 / 4.3e-13 | 2.2e-13 / 4.3e-13 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 |
| link_8 | 0.022 | >600 | 0.051 | 0.127 | 0.309 | 0.222 | 0.504 | - | - | - | oom | >600 | 2.97 | 0.536 | 0.733 | 2.61 | 1.28 | 1.27 | 2.2 | 3.33 | 0.093 | 0.187 | 1.46 | 0.103 | 60.7 | 98.6 | 8.89 | 0.699 | exact | 1.1e-16 / 2.3e-16 |  | 1.7e-13 / 3.6e-13 | 5.0e-16 / 1.0e-15 | 0.0e+00 / 0.0e+00 | 3.0e-08 / 6.3e-08 | 5.3e-07 / 1.1e-06 |  |  |  |  |  | 1.7e-16 / 3.5e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 0.0e+00 / 0.0e+00 | 1.7e-16 / 3.5e-16 | 4.7e-07 / 9.9e-07 | 1.7e-13 / 3.6e-13 | 1.7e-13 / 3.6e-13 | 1.7e-13 / 3.6e-13 | 5.6e-17 / 1.2e-16 | 0.0e+00 / 0.0e+00 |
| link_10 | 0.122 | - | 0.407 | 1.63 | 4.01 | 1.71 | 5.33 | - | - | - | - | - | 4.87 | 4.05 | 5.62 | 6.05 | 2.09 | 2.02 | 4.5 | 29.9 | 0.396 | 0.427 | 24 | 0.413 | 72 | 142 | 10.5 | 4.5 | exact | 8.3e-16 / 1.8e-15 |  | 2.3e-14 / 5.1e-14 | 1.6e-15 / 3.5e-15 | 8.3e-16 / 1.8e-15 | 4.9e-09 / 1.1e-08 | 3.0e-07 / 6.7e-07 |  |  |  |  |  | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 3.0e-07 / 6.7e-07 | 2.3e-14 / 5.1e-14 | 2.3e-14 / 5.1e-14 | 2.3e-14 / 5.1e-14 | 7.8e-16 / 1.7e-15 | 7.8e-16 / 1.7e-15 |
| link_12 | 1.62 | - | 3.81 | 14.1 | 41.3 | 12.1 | 55.5 | - | - | - | - | - | 33.1 | 33.7 | 45.3 | 26.6 | 9.31 | 9.31 | 19.1 | 271 | 8.04 | 2.04 | 7.91 | 7.27 | 136 | 181 | 23.1 | 42.3 | exact | 6.1e-16 / 1.4e-15 |  | 8.3e-15 / 1.9e-14 | 1.6e-15 / 3.7e-15 | 5.0e-16 / 1.1e-15 | 3.1e-08 / 7.1e-08 | 1.7e-07 / 3.8e-07 |  |  |  |  |  | 5.6e-16 / 1.3e-15 | 6.1e-16 / 1.4e-15 | 5.6e-16 / 1.3e-15 | 6.1e-16 / 1.4e-15 | 6.1e-16 / 1.4e-15 | 6.1e-16 / 1.4e-15 | 5.0e-16 / 1.1e-15 | 6.1e-16 / 1.4e-15 | 1.7e-07 / 3.8e-07 | 8.3e-15 / 1.9e-14 | 8.3e-15 / 1.9e-14 | 8.3e-15 / 1.9e-14 | 6.7e-16 / 1.5e-15 | 5.6e-16 / 1.3e-15 |
| link_14 | 22.4 | - | 25.6 | 132 (2/3) | oom | 80.1 | 418 | - | - | - | - | - | 364 | 254 | err | >600 | 63.2 | 73.8 | >600 | >600 | oom | oom | 275 | oom | oom | 262 | 141 (2/3) | oom | exact | 8.9e-16 / 2.1e-15 |  | 2.3e-13 / 5.3e-13 | 5.0e-16 / 1.2e-15 |  | 2.3e-08 / 5.5e-08 | 7.7e-08 / 1.8e-07 |  |  |  |  |  | 7.2e-16 / 1.7e-15 | 7.2e-16 / 1.7e-15 |  |  | 7.2e-16 / 1.7e-15 | 7.2e-16 / 1.7e-15 |  |  |  |  | 2.3e-13 / 5.3e-13 |  |  | 7.8e-16 / 1.8e-15 |
| link_16 | oom | - | 331 | >600 | - | 494 | >600 | - | - | - | - | - | >600 | >600 | - | - | 441 | 529 | - | - | - | - | >600 | - | - | 347 | >600 | - | exact |  |  | 4.7e-13 / 1.1e-12 |  |  | 5.1e-09 / 1.2e-08 |  |  |  |  |  |  |  |  |  |  | 6.3e-15 / 1.5e-14 | 6.3e-15 / 1.5e-14 |  |  |  |  |  |  |  | 6.2e-15 / 1.5e-14 |
| link_18 | - | - | >600 | - | - | >600 | - | - | - | - | - | - | - | - | - | - | oom | oom | - | - | - | - | - | - | - | 540 | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.6e-15 / 3.9e-15 |
| link_20 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | >600 | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| link_24 | oom | >600 | >600 | oom | oom | >600 | oom | >600 | oom | >600 | oom | >600 | >600 | >600 | >600 | err | >600 | >600 | - | - | - | - | - | - | - | - | oom | oom | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_4 | 0.005 | 0.114 | 0.018 | 0.006 | 0.002 | 0.006 | 0.018 | >600 | 0.006 | 1.68 | 5.04 | 0.397 | 2.59 | 0.158 | 0.156 | 0.456 | 1.26 | 1.27 | 1.23 | 1.32 | 0.044 | 0.132 | 17.3 | 0.045 | 23.6 | 34.6 | 8.32 | 0.37 | exact | 0.0e+00 / 0.0e+00 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.2e-16 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| router_6 | 0.008 | 11.7 | 0.02 | 0.013 | 0.011 | 0.012 | 0.058 | - | 0.015 | 44.6 | 6.52 | 1.4 | 2.7 | 0.177 | 0.179 | 1.08 | 1.27 | 1.27 | 1.61 | 1.7 | 0.071 | 0.268 | 1.39 | 0.081 | 37.7 | 71.9 | 8.75 | 0.398 | exact | 0.0e+00 / 0.0e+00 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 4.4e-16 / 8.9e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 0.0e+00 / 0.0e+00 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 |
| router_8 | 0.016 | >600 | 0.032 | 0.131 | 0.113 | 0.058 | 0.353 | - | 0.064 | >600 | 7.61 | >600 | 3 | 0.326 | 0.373 | 2.3 | 1.28 | 1.27 | 1.79 | 2.19 | 0.106 | 0.254 | 1.5 | 0.144 | 51.5 | 122 | 8.9 | 0.488 | exact | 0.0e+00 / 0.0e+00 |  | 1.4e-13 / 2.7e-13 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 8.9e-16 / 1.8e-15 |  | 1.1e-16 / 2.2e-16 |  | 0.0e+00 / 0.0e+00 | 4.4e-16 / 8.9e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| router_10 | 0.045 | - | 0.151 | 1.57 | 1.33 | 0.372 | 2.85 | - | 0.34 | - | oom | - | 5.14 | 1.59 | 1.99 | 4.31 | 1.66 | 1.69 | 2.4 | 4.26 | 0.169 | 0.376 | 1.52 | 0.276 | 47.5 | 159 | 10.6 | 1.74 | exact | 2.2e-16 / 4.4e-16 |  | 1.4e-13 / 2.7e-13 | 1.1e-16 / 2.2e-16 | 3.3e-16 / 6.7e-16 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 1.2e-15 / 2.4e-15 |  |  |  | 3.3e-16 / 6.7e-16 | 2.2e-16 / 4.4e-16 | 3.3e-16 / 6.7e-16 | 4.4e-16 / 8.9e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 2.2e-16 / 4.4e-16 | 3.3e-16 / 6.7e-16 |
| router_12 | 0.063 | - | 1.19 | 13.3 | 13.3 | 2.58 | 21 | - | 1.99 | - | - | - | 32.2 | 12 | 15 | 7.99 | 2.95 | 2.94 | 4.06 | 23.4 | 0.345 | 0.719 | 1.76 | 0.573 | 69.3 | 238 | 22.5 | 13.9 | exact | 6.7e-16 / 1.3e-15 |  | 1.3e-13 / 2.7e-13 | 5.6e-16 / 1.1e-15 | 3.3e-16 / 6.7e-16 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 2.4e-15 / 4.9e-15 |  |  |  | 4.4e-16 / 8.9e-16 | 8.9e-16 / 1.8e-15 | 3.3e-16 / 6.7e-16 | 2.2e-16 / 4.4e-16 | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 | 4.4e-16 / 8.9e-16 | 4.4e-16 / 8.9e-16 | 4.5e-07 / 9.0e-07 | 1.3e-13 / 2.7e-13 | 1.3e-13 / 2.7e-13 | 1.3e-13 / 2.7e-13 | 4.4e-16 / 8.9e-16 | 4.4e-16 / 8.9e-16 |
| router_14 | 0.311 | - | 9.02 | 101 | oom | 17.1 | 149 | - | 11.3 | - | - | - | 536 | 90 | 108 | 12.6 | 7.51 | 8.37 | 9.64 | 223 | 0.937 | 1.22 | 2.08 | 1.64 | 76 | 322 | 111 | oom | exact | 5.6e-16 / 1.1e-15 |  | 1.4e-13 / 2.7e-13 | 2.2e-16 / 4.4e-16 |  | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 2.3e-15 / 4.7e-15 |  |  |  | 3.3e-16 / 6.7e-16 | 3.3e-16 / 6.7e-16 | 4.4e-16 / 8.9e-16 | 5.6e-16 / 1.1e-15 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 4.4e-16 / 8.9e-16 | 5.6e-16 / 1.1e-15 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 4.4e-16 / 8.9e-16 | 3.3e-16 / 6.7e-16 |
| router_16 | 0.884 | - | 77.7 | oom | - | 103 | oom | - | 61.6 | - | - | - | >600 | >600 | >600 | 20.3 | 32.1 | 40.4 | 88.6 | >600 | 24.9 | 6.52 | 2.71 | 9.62 | 115 | 454 | oom | - | exact | 1.4e-14 / 2.9e-14 |  | 1.5e-13 / 3.0e-13 |  |  | 5.0e-08 / 1.0e-07 |  |  | 1.1e-14 / 2.2e-14 |  |  |  |  |  |  | 1.5e-14 / 3.0e-14 | 1.5e-14 / 2.9e-14 | 1.5e-14 / 2.9e-14 | 1.5e-14 / 3.0e-14 |  | 4.5e-07 / 9.0e-07 | 1.5e-13 / 3.0e-13 | 1.5e-13 / 3.0e-13 | 1.5e-13 / 3.0e-13 | 1.5e-14 / 2.9e-14 | 1.5e-14 / 3.0e-14 |
| router_18 | 3.76 | - | >600 | - | - | >600 | - | - | 309 | - | - | - | - | - | - | 34.9 | 183 | 242 | >600 | - | oom | oom | 3.63 | 57.6 | 160 | 589 | - | - | exact | 1.9e-14 / 3.7e-14 |  |  |  |  |  |  |  | 1.5e-14 / 2.9e-14 |  |  |  |  |  |  | 1.9e-14 / 3.8e-14 | 1.9e-14 / 3.8e-14 | 1.9e-14 / 3.8e-14 |  |  |  |  | 1.5e-13 / 3.1e-13 | 1.5e-13 / 3.1e-13 | 1.9e-14 / 3.8e-14 | 1.9e-14 / 3.8e-14 |
| router_20 | 4.45 | - | - | - | - | - | - | - | >600 | - | - | - | - | - | - | 53 | oom | >600 | - | - | - | - | 5.97 | oom | 245 | >600 | - | - | exact | 5.9e-14 / 1.2e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.8e-14 / 1.2e-13 |  |  |  |  |  |  | 7.6e-14 / 1.5e-13 |  | 5.9e-14 / 1.2e-13 |  |
| router_24 | oom | >600 | >600 | oom | oom | >600 | oom | >600 | >600 | >600 | oom | >600 | >600 | >600 | >600 | 124 | 28 | 28.6 | - | - | - | - | >600 | - | oom | - | oom | oom | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.4e-11 / 4.8e-11 | 2.4e-11 / 4.8e-11 | 2.4e-11 / 4.8e-11 |  |  |  |  |  |  |  |  |
| router_28 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 233 | 51.4 | 51.5 | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_32 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 413 | 89.6 | 89.6 | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_40 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | >600 | 241 | 240 | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| weather_h1 | 0.003 | 0.069 | 0.018 | 0.005 | 0.001 | 0.023 | err | err | 0.006 | 0.602 | 3.6 | 0.338 | 1.56 | 2.02 | 9.65 | 11.2 | 12.2 | >600 | 1.1 | 1.14 | zero | 0.315 | 1.52 | 0.229 | 31.3 | 21 | 9.08 | 0.343 | exact | 2.6e-23 / 2.8e-16 | 4.0e-16 / 4.3e-09 | 4.1e-20 / 4.4e-13 | 6.6e-23 / 7.1e-16 | 1.2e-17 / 1.3e-10 | 6.4e-09 / 6.9e-02 |  |  | 0.0e+00 / 0.0e+00 | 7.7e-21 / 8.2e-14 | 2.6e-23 / 2.8e-16 | 1.2e-17 / 1.3e-10 | 2.6e-23 / 2.8e-16 | 2.6e-23 / 2.8e-16 | 1.3e-23 / 1.4e-16 | 1.3e-23 / 1.4e-16 | 5.3e-23 / 5.7e-16 |  | 2.6e-23 / 2.8e-16 | 1.3e-23 / 1.4e-16 |  | 4.1e-20 / 4.4e-13 | 4.1e-20 / 4.4e-13 | 4.1e-20 / 4.4e-13 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| weather_h2 | 0.009 | 0.689 | 0.024 | 0.018 | 0.018 | 0.127 | - | - | oom | 134 | 6.29 | 0.413 | 2.68 | 2.09 | 15.4 | 13.4 | 12.5 | - | 1.44 | 1.56 | zero | 0.3 | 1.82 | 0.127 | 32.8 | 59.4 | 11.8 | 0.363 | exact | 2.6e-23 / 2.1e-16 | 3.5e-16 / 2.8e-09 | 4.8e-19 / 3.9e-12 | 1.1e-22 / 8.6e-16 | 8.3e-17 / 6.7e-10 | 2.4e-08 / 1.9e-01 |  |  |  | 5.2e-21 / 4.2e-14 | 2.6e-23 / 2.1e-16 | 8.3e-17 / 6.7e-10 | 2.6e-23 / 2.1e-16 | 1.3e-22 / 1.1e-15 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 4.4e-18 / 3.6e-11 |  | 0.0e+00 / 0.0e+00 | 5.3e-23 / 4.3e-16 |  | 4.8e-19 / 3.9e-12 | 4.8e-19 / 3.9e-12 | 4.8e-19 / 3.9e-12 | 2.6e-23 / 2.1e-16 | 2.6e-23 / 2.1e-16 |
| weather_h3 | 0.024 | 2.7 | 0.079 | 0.099 | 0.263 | 0.283 | - | - | - | >600 | 7.69 | 0.622 | 2.8 | 2.08 | 24.7 | 15.3 | 12.2 | - | 1.69 | 1.83 | zero | 0.214 | oom | 0.18 | 47.3 | 104 | 14.8 | 0.612 | exact | 7.9e-23 / 5.0e-16 | 2.0e-15 / 1.3e-08 | 1.6e-19 / 1.0e-12 | 1.9e-22 / 1.2e-15 | 9.9e-17 / 6.2e-10 | 4.2e-08 / 2.6e-01 |  |  |  |  | 7.9e-23 / 5.0e-16 | 9.9e-17 / 6.2e-10 | 1.1e-22 / 6.7e-16 | 3.7e-22 / 2.3e-15 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 7.4e-18 / 4.7e-11 |  | 7.9e-23 / 5.0e-16 | 2.6e-23 / 1.7e-16 |  | 1.6e-19 / 1.0e-12 |  | 1.6e-19 / 1.0e-12 | 5.3e-23 / 3.3e-16 | 2.6e-23 / 1.7e-16 |
| weather_h4 | 0.12 | 15.7 | 0.152 | 1.32 | 2.73 | 0.442 | - | - | - | - | oom | 3.21 | 2.98 | 2.09 | 35.9 | 17.8 | 12.3 | - | 2.14 | 4.46 | zero | 0.263 | - | 0.324 | 60.6 | 144 | 16.1 | 3.15 | exact | 1.1e-22 / 5.5e-16 | 4.4e-15 / 2.3e-08 | 7.6e-20 / 3.9e-13 | 1.3e-22 / 6.9e-16 | 6.2e-17 / 3.2e-10 | 7.3e-09 / 3.8e-02 |  |  |  |  |  | 6.2e-17 / 3.2e-10 | 1.3e-22 / 6.9e-16 | 4.0e-22 / 2.1e-15 | 2.6e-23 / 1.4e-16 | 2.6e-23 / 1.4e-16 | 1.0e-17 / 5.4e-11 |  | 7.9e-23 / 4.1e-16 | 5.3e-23 / 2.7e-16 |  | 7.6e-20 / 3.9e-13 |  | 7.6e-20 / 3.9e-13 | 1.3e-22 / 6.9e-16 | 7.9e-23 / 4.1e-16 |
| weather_h5 | 0.132 | 51 | 0.256 | 5.88 | 11.4 | 0.594 | - | - | - | - | - | 21.3 | 3.96 | 2.1 | 50 | 20.4 | 12.5 | - | 2.38 | 8.02 | zero | 0.352 | - | 0.54 | 50.4 | 162 | 20.6 | 12 | exact | 1.9e-22 / 8.2e-16 | 1.6e-15 / 7.1e-09 | 1.2e-19 / 5.3e-13 | 0.0e+00 / 0.0e+00 | 1.2e-16 / 5.2e-10 | 2.7e-08 / 1.2e-01 |  |  |  |  |  | 1.2e-16 / 5.2e-10 | 1.1e-22 / 4.7e-16 | 3.4e-22 / 1.5e-15 | 2.6e-23 / 1.2e-16 | 5.3e-23 / 2.3e-16 | 1.3e-17 / 5.9e-11 |  | 1.3e-22 / 5.8e-16 | 5.3e-23 / 2.3e-16 |  | 1.2e-19 / 5.3e-13 |  | 1.2e-19 / 5.3e-13 | 1.1e-22 / 4.7e-16 | 7.9e-23 / 3.5e-16 |
| weather_h6 | 0.158 | oom | 0.299 | 14.3 | 27.8 | 0.752 | - | - | - | - | - | 165 | 4.83 | 2.09 | 66.3 | 22.7 | 11.4 | - | 2.6 | 10.3 | zero | 0.468 | - | 0.803 | 56.3 | 202 | 29.1 | 28.8 | exact | 1.6e-22 / 6.1e-16 |  | 5.0e-19 / 1.9e-12 | 0.0e+00 / 0.0e+00 | 1.7e-16 / 6.6e-10 | 3.8e-08 / 1.5e-01 |  |  |  |  |  | 1.7e-16 / 6.6e-10 | 1.6e-22 / 6.1e-16 | 6.4e-22 / 2.4e-15 | 5.3e-23 / 2.0e-16 | 5.3e-23 / 2.0e-16 | 1.6e-17 / 6.3e-11 |  | 1.1e-22 / 4.0e-16 | 2.1e-22 / 8.1e-16 |  | 5.0e-19 / 1.9e-12 |  | 5.0e-19 / 1.9e-12 | 1.1e-22 / 4.0e-16 | 1.1e-22 / 4.0e-16 |
| weather_h7 | 0.224 | - | 0.354 | 26.1 | 46 | 0.908 | - | - | - | - | - | >600 | 7.06 | 2.1 | 85.2 | 27.9 | 11.3 | - | 3.26 | 19 | zero | 0.613 | - | 1.26 | 61.4 | 231 | 40.8 | 46.9 | exact | 1.6e-22 / 5.4e-16 |  | 3.8e-19 / 1.3e-12 | 0.0e+00 / 0.0e+00 | 2.7e-16 / 9.2e-10 | 4.0e-09 / 1.3e-02 |  |  |  |  |  |  | 3.2e-22 / 1.1e-15 | 3.7e-22 / 1.3e-15 | 5.3e-23 / 1.8e-16 | 1.1e-22 / 3.6e-16 | 2.0e-17 / 6.6e-11 |  | 1.6e-22 / 5.4e-16 | 3.7e-22 / 1.3e-15 |  | 3.8e-19 / 1.3e-12 |  | 3.8e-19 / 1.3e-12 | 2.1e-22 / 7.2e-16 | 2.1e-22 / 7.2e-16 |
| weather_h8 | 0.241 | - | 0.506 | 40.8 | 76.4 | 1.08 | - | - | - | - | - | - | 8.3 | 2.09 | 107 | 29.9 | 10.6 | - | 3.46 | 22.2 | zero | 0.749 | - | 1.66 | 71.7 | 269 | 55.7 | 77.6 | exact | 2.1e-22 / 6.4e-16 |  | 2.3e-19 / 7.1e-13 | 5.8e-22 / 1.8e-15 | 2.3e-16 / 6.9e-10 | 3.0e-08 / 9.2e-02 |  |  |  |  |  |  | 2.6e-22 / 8.0e-16 | 3.7e-22 / 1.1e-15 | 1.1e-22 / 3.2e-16 | 1.6e-22 / 4.8e-16 | 2.3e-17 / 6.8e-11 |  | 2.6e-22 / 8.0e-16 | 1.6e-22 / 4.8e-16 |  | 2.3e-19 / 7.1e-13 |  | 2.3e-19 / 7.1e-13 | 1.6e-22 / 4.8e-16 | 2.6e-22 / 8.0e-16 |
| weather_h9 | 0.246 | - | 0.561 | 56.8 | oom | 1.24 | - | - | - | - | - | - | 16.8 | 2.1 | 131 | 31.9 | 10 | - | 3.89 | 30.7 | zero | 0.997 | - | 2.14 | 69.5 | 295 | 71.5 | oom | exact | 1.6e-22 / 4.4e-16 |  | 1.2e-19 / 3.4e-13 | 1.1e-21 / 2.9e-15 |  | 3.5e-08 / 9.6e-02 |  |  |  |  |  |  | 1.6e-22 / 4.4e-16 | 1.3e-21 / 3.5e-15 | 5.3e-23 / 1.5e-16 | 5.3e-23 / 1.5e-16 | 2.6e-17 / 7.0e-11 |  | 2.1e-22 / 5.8e-16 | 3.7e-22 / 1.0e-15 |  | 1.2e-19 / 3.4e-13 |  | 1.2e-19 / 3.4e-13 | 5.3e-23 / 1.5e-16 | 1.1e-22 / 2.9e-16 |
| weather_h100 | 5.68 | >600 | 16.5 | oom | oom | 16.7 | >600 | err | oom | >600 | oom | >600 | >600 | 2.57 | >600 | 303 | 14.1 | >600 | 30.2 | >600 | oom | 169 | - | 344 | >600 | >600 | oom | oom | exact | 1.6e-20 / 4.5e-15 |  | 2.1e-18 / 6.0e-13 |  |  | 1.0e-09 / 2.9e-04 |  |  |  |  |  |  |  | 1.4e-19 / 3.9e-14 |  | 1.7e-20 / 4.7e-15 | 3.0e-16 / 8.6e-11 |  | 2.1e-21 / 6.1e-16 |  |  | 2.1e-18 / 6.0e-13 |  | 2.1e-18 / 6.0e-13 |  |  |
| weather_h200 | 15.2 | - | 39.9 | - | - | 32.7 | - | - | - | - | - | - | - | 2.83 | - | 560 | 14 | - | 69.6 | - | - | >600 | - | >600 | - | - | - | - | exact | 5.9e-20 / 8.5e-15 |  | 4.5e-18 / 6.4e-13 |  |  | 4.3e-08 / 6.2e-03 |  |  |  |  |  |  |  | 5.5e-19 / 8.0e-14 |  | 6.9e-20 / 9.9e-15 | 6.0e-16 / 8.7e-11 |  | 7.6e-21 / 1.1e-15 |  |  |  |  |  |  |  |
| weather_h300 | 20.7 | - | 57.8 | - | - | 48.5 | - | - | - | - | - | - | - | 3.14 | - | >600 | 13.9 | - | >600 | - | - | - | - | - | - | - | - | - | exact | 1.5e-19 / 1.5e-14 |  | 4.2e-18 / 4.1e-13 |  |  | 1.3e-08 / 1.3e-03 |  |  |  |  |  |  |  | 1.3e-18 / 1.2e-13 |  |  | 9.0e-16 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h400 | 19 | - | 96.8 | - | - | 64.8 | - | - | - | - | - | - | - | 3.28 | - | - | 13.6 | - | - | - | - | - | - | - | - | - | - | - | exact | 2.4e-19 / 1.7e-14 |  | 2.6e-17 / 1.9e-12 |  |  | 3.1e-08 / 2.2e-03 |  |  |  |  |  |  |  | 2.2e-18 / 1.6e-13 |  |  | 1.2e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h500 | 55.3 | - | 125 | - | - | 81.1 | - | - | - | - | - | - | - | 3.46 | - | - | 14.2 | - | - | - | - | - | - | - | - | - | - | - | exact | 4.1e-19 / 2.4e-14 |  | 4.0e-17 / 2.3e-12 |  |  | 2.5e-08 / 1.4e-03 |  |  |  |  |  |  |  | 3.5e-18 / 2.0e-13 |  |  | 1.5e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h600 | 13.6 | oom | 172 | oom | oom | 104 | oom | err | err | >600 | oom | >600 | >600 | 4.6 | >600 | >600 | 15.3 | >600 | - | - | - | - | - | - | - | - | oom | oom | exact | 1.6e-18 / 7.7e-14 |  | 3.6e-17 / 1.7e-12 |  |  | 1.9e-08 / 9.1e-04 |  |  |  |  |  |  |  | 5.0e-18 / 2.4e-13 |  |  | 1.8e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h700 | 40.2 | - | 206 | - | - | 116 | - | - | - | - | - | - | - | 4.27 | - | - | 15 | - | - | - | - | - | - | - | - | - | - | - | exact | 8.2e-19 / 3.4e-14 |  | 1.9e-18 / 7.8e-14 |  |  | 3.7e-08 / 1.5e-03 |  |  |  |  |  |  |  | 6.8e-18 / 2.8e-13 |  |  | 2.1e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h800 | 55.1 | - | 282 | - | - | 131 | - | - | - | - | - | - | - | 4.64 | - | - | 15.3 | - | - | - | - | - | - | - | - | - | - | - | exact | 1.0e-18 / 3.6e-14 |  | 2.1e-17 / 7.6e-13 |  |  | 6.8e-09 / 2.5e-04 |  |  |  |  |  |  |  | 9.0e-18 / 3.3e-13 |  |  | 2.4e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h900 | 58.2 | - | 338 | - | - | 147 | - | - | - | - | - | - | - | 5.08 | - | - | 15.2 | - | - | - | - | - | - | - | - | - | - | - | exact | 1.3e-18 / 4.1e-14 |  | 7.9e-18 / 2.6e-13 |  |  | 4.9e-08 / 1.6e-03 |  |  |  |  |  |  |  | 1.1e-17 / 3.7e-13 |  |  | 2.7e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h1000 | 65.1 | - | 381 | - | - | 162 | - | - | - | - | - | - | - | 5.95 | - | - | 16 | - | - | - | - | - | - | - | - | - | - | - | exact | 1.6e-18 / 4.8e-14 |  | 3.0e-17 / 8.6e-13 |  |  | 5.3e-09 / 1.5e-04 |  |  |  |  |  |  |  | 1.4e-17 / 4.1e-13 |  |  | 3.0e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_n2 | 0.011 | 3.93 | 0.018 | 0.02 | 0.03 | 0.007 | err | err | 0.135 | >600 | 7.43 | 0.56 (2/3) | 2.76 | 0.156 | 0.156 | 0.161 | 1.27 | 1.27 | 1.72 | 1.85 | 0.173 | 0.287 | 16.7 | 0.218 | 40.1 | 91.2 | 10.4 | 0.374 | exact | 2.8e-17 / 1.5e-16 | 3.2e-10 / 1.7e-09 | 3.9e-13 / 2.1e-12 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 6.0e-16 | 4.0e-08 / 2.2e-07 |  |  | 2.8e-17 / 1.5e-16 |  | 2.8e-17 / 1.5e-16 | 5.6e-16 / 3.0e-15 | 8.3e-17 / 4.5e-16 | 5.6e-17 / 3.0e-16 | 2.8e-17 / 1.5e-16 | 8.3e-17 / 4.5e-16 | 2.8e-17 / 1.5e-16 | 2.8e-17 / 1.5e-16 | 2.8e-17 / 1.5e-16 | 8.3e-17 / 4.5e-16 | 6.0e-08 / 3.3e-07 | 3.9e-13 / 2.1e-12 | 3.9e-13 / 2.1e-12 | 3.9e-13 / 2.1e-12 | 1.4e-16 / 7.5e-16 | 8.3e-17 / 4.5e-16 |
| weather_n7 | 0.038 | oom | 0.052 | 4.06 | 7.43 | 0.115 | - | - | >600 | - | oom | >600 | 4.92 | 0.182 | 1.66 | 2.15 | 1.59 | 21.5 | 2.63 | 4.28 | 0.317 | 0.65 | 102 | 1.1 | 67.5 | 251 | 17.1 | 7.94 | exact | 1.4e-20 / 2.0e-16 |  | 1.9e-17 / 2.8e-13 | 2.7e-20 / 4.0e-16 | 1.9e-16 / 2.8e-12 | 3.6e-08 / 5.4e-04 |  |  |  |  |  |  | 4.1e-20 / 6.0e-16 | 2.0e-19 / 3.0e-15 | 1.4e-20 / 2.0e-16 | 0.0e+00 / 0.0e+00 | 1.4e-20 / 2.0e-16 | 1.4e-20 / 2.0e-16 | 2.7e-20 / 4.0e-16 | 1.4e-20 / 2.0e-16 | 3.6e-07 / 5.4e-03 | 1.9e-17 / 2.8e-13 | 1.9e-17 / 2.8e-13 | 1.9e-17 / 2.8e-13 | 6.8e-20 / 1.0e-15 | 2.7e-20 / 4.0e-16 |
| weather_n8 | 0.133 | - | 0.123 | 11.7 | 20.8 | 0.26 | - | - | - | - | - | - | 5.95 | 0.27 | 7.66 | 3.49 | 12.6 | 386 | 2.81 | 6.4 | 0.39 | 0.755 | 593 | 1.51 | 70.3 | 276 | 25.3 | 21.4 | exact | 8.5e-21 / 6.5e-16 |  | 3.7e-17 / 2.9e-12 | 2.4e-20 / 1.8e-15 | 3.4e-16 / 2.6e-11 | 1.1e-08 / 8.8e-04 |  |  |  |  |  |  | 1.4e-20 / 1.0e-15 | 4.1e-20 / 3.1e-15 | 3.4e-21 / 2.6e-16 | 5.1e-21 / 3.9e-16 | 1.7e-21 / 1.3e-16 | 1.7e-21 / 1.3e-16 | 1.4e-20 / 1.0e-15 | 1.0e-20 / 7.8e-16 | 1.1e-08 / 8.8e-04 | 3.7e-17 / 2.9e-12 | 3.7e-17 / 2.9e-12 | 3.7e-17 / 2.9e-12 | 5.1e-21 / 3.9e-16 | 0.0e+00 / 0.0e+00 |
| weather_n9 | 0.305 | - | 0.315 | 33.6 | 66.2 | 0.607 | - | - | - | - | - | - | 7.55 | 0.624 | 35.6 | 8.36 | 189 | >600 | 3.49 | 11.6 | 0.853 | 1.27 | oom | 2.33 | 74.1 | 310 | 48.6 | 67.4 | exact | 2.5e-21 / 1.0e-15 |  | 4.2e-18 / 1.7e-12 | 6.8e-21 / 2.7e-15 | 2.2e-16 / 8.7e-11 | 1.9e-08 / 7.4e-03 |  |  |  |  |  |  | 3.0e-21 / 1.2e-15 | 6.4e-21 / 2.5e-15 | 8.5e-22 / 3.4e-16 | 1.3e-21 / 5.0e-16 | 2.5e-21 / 1.0e-15 |  | 3.4e-21 / 1.3e-15 | 2.1e-21 / 8.4e-16 | 4.8e-07 / 1.9e-01 | 4.2e-18 / 1.7e-12 |  | 4.2e-18 / 1.7e-12 | 4.2e-22 / 1.7e-16 | 0.0e+00 / 0.0e+00 |
| weather_n10 | 0.317 | - | 0.76 | 89.1 | oom | 1.42 | - | - | - | - | - | - | 20.3 | 2.12 | 158 | 37 | 13.4 | - | 4.27 | 20.6 | zero | 1.2 | - | 2.67 | 84.6 | 347 | 105 | oom | exact | 4.2e-22 / 1.1e-15 |  | 3.7e-19 / 9.4e-13 | 1.5e-21 / 3.8e-15 |  | 6.5e-10 / 1.6e-03 |  |  |  |  |  |  | 2.6e-22 / 6.6e-16 | 6.4e-22 / 1.6e-15 | 1.1e-22 / 2.7e-16 | 1.1e-22 / 2.7e-16 | 2.9e-17 / 7.1e-11 |  | 4.2e-22 / 1.1e-15 | 5.3e-23 / 1.3e-16 |  | 3.7e-19 / 9.4e-13 |  | 3.7e-19 / 9.4e-13 | 2.1e-22 / 5.3e-16 | 3.7e-22 / 9.3e-16 |
| weather_n11 | 0.709 | - | 1.75 | 216 | - | 3.32 | - | - | - | - | - | - | 41.1 | 8.22 | >600 | 179 | err | - | 6.16 | 98.7 | zero | 1.73 | - | 4.29 | 89.7 | 387 | 232 | - | exact | 6.6e-23 / 1.0e-15 |  | 1.4e-21 / 2.1e-14 | 9.3e-23 / 1.5e-15 |  | 3.6e-08 / 5.7e-01 |  |  |  |  |  |  | 6.6e-23 / 1.0e-15 | 4.4e-22 / 6.9e-15 |  | 1.3e-23 / 2.1e-16 |  |  | 1.3e-23 / 2.1e-16 | 5.3e-23 / 8.3e-16 |  | 1.4e-21 / 2.1e-14 |  | 1.4e-21 / 2.1e-14 | 1.3e-23 / 2.1e-16 | 1.3e-23 / 2.1e-16 |
| weather_n12 | 0.74 | - | 4.57 | oom | - | zero | - | - | - | - | - | - | 216 | 33.4 | - | err | - | - | 8.33 | 241 | zero | 2.36 | - | 8.74 | 101 | 425 | oom | - | exact | 1.2e-23 / 1.1e-15 |  | 8.7e-21 / 8.5e-13 |  |  |  |  |  |  |  |  |  | 1.2e-23 / 1.1e-15 | 6.5e-23 / 6.4e-15 |  |  |  |  | 6.6e-24 / 6.5e-16 | 2.8e-23 / 2.8e-15 |  | 8.7e-21 / 8.5e-13 |  | 8.7e-21 / 8.5e-13 | 1.7e-24 / 1.6e-16 | 5.0e-24 / 4.9e-16 |
| weather_n13 | 3.56 | - | 11.1 | - | - | zero | - | - | - | - | - | - | 176 | 138 | - | - | - | - | 15.5 | 345 | zero | 7.72 | - | 16.5 | 128 | 494 | - | - | exact | 3.1e-24 / 1.9e-15 |  | 4.7e-21 / 2.9e-12 |  |  |  |  |  |  |  |  |  | 3.1e-24 / 1.9e-15 | 1.9e-23 / 1.2e-14 |  |  |  |  | 1.7e-24 / 1.0e-15 | 1.9e-24 / 1.2e-15 |  | 4.7e-21 / 2.9e-12 |  | 4.7e-21 / 2.9e-12 | 6.2e-25 / 3.8e-16 | 2.1e-25 / 1.3e-16 |
| weather_n15 | 22 | >600 | 99.6 | >600 | oom | zero | >600 | err | oom | >600 | oom | >600 | >600 | oom | err | err | err | err | >600 | >600 | oom | oom | - | oom | 163 | 599 | >600 | oom | exact | 8.4e-26 / 2.0e-15 |  | 3.1e-23 / 7.5e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 3.2e-26 / 7.8e-16 | 3.9e-26 / 9.4e-16 |
| weather_n16 | 17.9 | >600 | 385 | >600 | oom | zero | >600 | err | oom | >600 | oom | >600 | >600 | >600 | err | err | oom | oom | - | - | - | - | - | - | 159 | 559 | >600 | oom | exact | 8.1e-27 / 1.3e-15 |  | 3.7e-25 / 5.8e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 8.1e-27 / 1.3e-15 | 4.8e-27 / 7.7e-16 |
| weather_n17 | oom | >600 | >600 | >600 | oom | zero | >600 | err | oom | >600 | oom | >600 | >600 | >600 | err | err | oom | oom | - | - | - | - | - | - | 373 | >600 | >600 | oom | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |
| weather_n18 | oom | oom | >600 | >600 | oom | >600 | >600 | err | oom | >600 | oom | >600 | >600 | >600 | err | err | err | err |  |  |  |  |  |  |  | - | >600 | oom | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| weather_n19 | oom | oom | >600 | oom | oom | >600 | >600 | err | oom | >600 | oom | >600 | >600 | >600 | err | err | oom | oom |  |  |  |  |  |  |  |  | oom | oom | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |

Answers. The largest differences from the reference: tddlog 1.2e-13
relative (5.9e-14 absolute, router_20); ProbLog 2.3e-08 relative
(4.4e-15 absolute, weather_h4); Dice 3.9e-12 relative (4.8e-19 absolute,
weather_h2); Alea 3.8e-15 relative (1.5e-21 absolute, weather_n10);
Roulette 9.2e-10 relative (2.7e-16 absolute, weather_h7); Geni 5.7e-01
relative (3.6e-08 absolute, weather_n11); noDice 1.1e-06 relative
(5.3e-07 absolute, link_8); Psi every answer the reference's; Genfer
2.9e-14 relative (1.5e-14 absolute, router_18); SPPL 8.2e-14 relative
(7.7e-21 absolute, weather_h1); Pluck 5.0e-16 relative (7.9e-23
absolute, weather_h3); PITA 6.7e-10 relative (8.3e-17 absolute,
weather_h2); aspmc 1.9e-15 relative (3.1e-24 absolute, weather_n13);
Storm sparse 4.1e-13 relative (1.4e-17 absolute, weather_h1000); Storm
exact 1.8e-15 relative (8.3e-16 absolute, link_10); Storm dd 4.8e-11
relative (2.4e-11 absolute, router_24); PRISM sparse 8.7e-11 relative
(3.0e-15 absolute, weather_h1000); PRISM hybrid 4.8e-11 relative
(2.4e-11 absolute, router_24); Ace 3.0e-14 relative (1.5e-14 absolute,
router_16); Ace c2d 2.8e-15 relative (2.8e-23 absolute, weather_n12);
Merlin 1.9e-01 relative (4.8e-07 absolute, weather_n9); pyAgrum 3.9e-12
relative (4.8e-19 absolute, weather_h2); pgmpy 3.9e-12 relative (4.8e-19
absolute, weather_h2); libDAI 3.9e-12 relative (4.8e-19 absolute,
weather_h2); TensorInference.jl greedy 1.2e-13 relative (5.9e-14
absolute, router_20); TensorInference.jl treesa 3.8e-14 relative
(1.9e-14 absolute, router_18).

## Further exact systems

The survey of exact systems (docs/baselines.md) adds systems that run
part of the program suites, each on the programs of its own paper, on
the Dice programs it accepts, on the ProbLog programs or, the network
tools, on the networks. Each ran in the same lanes as the systems above,
single-threaded, with the same limits; the time is the process's,
tddlog's too, start-up included. Every answer is compared with the
suite's reference, as Dice's and Alea.jl's are; a difference of more
than 1e-6 absolute and relative is `wrong`. Geni, noDice, Psi, Genfer,
SPPL, Pluck, PITA, aspmc, Storm sparse, Storm exact, Storm dd, PRISM
sparse, PRISM hybrid, Ace, Ace c2d, Merlin, pyAgrum, pgmpy, libDAI,
TensorInference.jl greedy, TensorInference.jl treesa and Roulette also
ran in a later run, on other machines, on the instances the release run
did not give them, each in the same lanes as tddlog; there a system's
time is compared with that run's tddlog. A time under 0.2 s counts as
0.2 s, and an instance both answer in under 0.2 s is counted in
neither's favour.

| system | answers | runs |
|---|---|---|
| Geni (gennifer; Li and Zhang, ICFP 2025) | generating functions, f32 | the programs of its own evaluation in the `geni` suite, as `.gir` files, and every other program with a Dice program (the circuits' too), translated by `dice2geni.py` |
| noDice (Gürtler and Kaminski, OOPSLA 2026) | exact | the Dice programs without `nflip`, as Dice runs them |
| Psi (Gehr et al., CAV 2016; revision ef8a9e5) | symbolic | the Dice programs, translated by `dice2psi.py`, and its own linear regression of the bit-blasted suite |
| Genfer (Zaiser, Murawski and Ong, NeurIPS 2023) | generating functions | its population model of the bit-blasted suite, the Dice programs of its own comparison, and every other program with a Dice program (the circuits' too), translated by `dice2genfer.py` |
| SPPL 2.0.4 (Saad, Rinard and Mansinghka, PLDI 2021) | symbolic sum-product | its hierarchical Markov switching model of the bit-blasted suite, and every other program with a Dice program (the circuits' too), translated by `dice2sppl.py` |
| Pluck (Bowers et al., PLDI 2025) | exact | its four sequence models of the `ppl` suite |
| PITA (cplint, SWI-Prolog 9.2.9) | exact, BDDs | the ProbLog programs tddlog reads, through `problog2pita.py` |
| aspmc 1.1.1 with sharpSAT-TD | exact, knowledge compilation | the ProbLog programs tddlog reads, the evidence asked as queries |
| Storm 1.14 (stormpy), one Sylvan thread | exact | the weather factory's PRISM model, `P=? [F<=H "allStrike"]`, and every other program with a Dice program (the circuits' too) as a Markov chain, through `dice2prism.py`: the sparse engine in floating point (`sparse`), the sparse engine on rationals (`exact`) and the symbolic BDD engine (`dd`) |
| PRISM 4.8.1 | exact | the weather factory's PRISM model and every other program with a Dice program, through `dice2prism.py`, with the sparse and the hybrid engine |
| Ace 3.0 | exact, arithmetic circuits | the Bayesian networks of the Dice paper and of Roulette's, from their BIF files, and the Dice paper's programs as the network of their ground program, written by `ground2net.py` |
| Merlin 1.7.0's bucket-tree elimination (`-a bte`) | exact | the Bayesian networks of the Dice paper and of Roulette's, from their BIF files, and the Dice paper's programs as the network of their ground program, written by `ground2net.py` |
| pyAgrum's junction tree | exact | the Bayesian networks of the Dice paper and of Roulette's, from their BIF files, and the Dice paper's programs as the network of their ground program, written by `ground2net.py` |
| pgmpy's variable elimination | exact | the Bayesian networks of the Dice paper and of Roulette's, from their BIF files, and the Dice paper's programs as the network of their ground program, written by `ground2net.py` |
| libDAI's junction tree | exact | the Bayesian networks of the Dice paper and of Roulette's, from their BIF files, and the Dice paper's programs as the network of their ground program, written by `ground2net.py` |
| TensorInference.jl's tensor network contraction | exact | the Bayesian networks of the Dice paper and of Roulette's, from their BIF files, and the Dice paper's programs as the network of their ground program, written by `ground2net.py` |
| Roulette (Moy et al., PLDI 2025) | exact | the circuits of the bit-blasted and Alea.jl programs, their Dice programs translated by `dice2roulette.py`; its own and the Dice programs are in the sections above |

### On the Dice paper's programs

| family | instances | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Merlin | pyAgrum | pgmpy | libDAI | TensorInference.jl greedy | TensorInference.jl treesa |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| bn | 17 | 17 | 14 of 17 | 11 of 17 | 4 of 17 | 13 of 17 | 7 of 17 | 10 of 17 |  |  | 14 of 17 | 14 of 17 | 14 of 17 | 14 of 17 | 14 of 17 | 17 of 17 | 15 of 16 | 17 of 17 | 17 of 17 | 15 of 16 | 17 of 17 | 15 of 16 | 17 of 17 |
| dice | 8 | 8 | 7 of 8 | 8 of 8 | 6 of 8 | 8 of 8 | 7 of 8 | 7 of 8 | 7 of 8 | 7 of 8 | 8 of 8 | 8 of 8 | 7 of 8 | 8 of 8 | 8 of 8 | 7 of 8 | 7 of 8 | 8 of 8 | 8 of 8 | 7 of 8 | 7 of 8 | 7 of 8 | 7 of 8 |

**Geni.** Geni ran 25 instances and answered 21 (1 fail, 3 time out).
Its answers differ from the reference's by at most 1.0e+00 relative
(1.3e-07 absolute, bn_water) over 21 instances. Over the 25 instances
given to both that either answers, a run that does not answer counted at
the 600 s limit, Geni takes 2627 s in all and tddlog 1.94 s; over the 21
both answer, 227 s against tddlog's 0.46. tddlog alone answers bn_andes,
bn_link, bn_munin and digitRecognition.

**noDice.** noDice ran 25 instances and answered 19 (4 run out of
memory, 2 time out). Its answers differ from the reference's by at most
1.0e+00 relative (7.5e-07 absolute, bn_water) over 19 instances. Over
the 25 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, noDice takes 3788 s in all and tddlog
1.92 s; over the 19 both answer, 188 s against tddlog's 0.33. tddlog
alone answers bn_alarm, bn_andes, bn_hailfinder, bn_link, bn_munin and 1
more.

**Psi.** Psi ran 25 instances and answered 10 (1 fail, 5 run out of
memory, 9 time out). Its answers differ from the reference's by at most
1.4e-08 relative (4.1e-11 absolute, alarm) over 10 instances. Over the
25 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Psi takes 9009 s in all and tddlog
1.92 s; over the 10 both answer, 9.1 s against tddlog's 0.026. tddlog
alone answers bn_alarm, bn_andes, bn_child, bn_hailfinder, bn_hepar2 and
10 more.

**Genfer.** Genfer ran 25 instances and answered 21 (1 run out of
memory, 3 time out). Its answers differ from the reference's by at most
1.9e+02 relative (5.8e-15 absolute, digitRecognition) over 21 instances.
Over the 25 instances given to both that either answers, a run that does
not answer counted at the 600 s limit, Genfer takes 2581 s in all and
tddlog 1.94 s; over the 21 both answer, 181 s against tddlog's 0.39.
tddlog alone answers bn_andes, bn_link, bn_munin and bn_pathfinder.

**SPPL.** SPPL ran 25 instances and answered 14 (11 time out). Its
answers differ from the reference's by at most 1.4e-08 relative (4.1e-11
absolute, alarm) over 14 instances. Over the 25 instances given to both
that either answers, a run that does not answer counted at the 600 s
limit, SPPL takes 6923 s in all and tddlog 1.94 s; over the 14 both
answer, 323 s against tddlog's 0.059. tddlog alone answers bn_andes,
bn_hailfinder, bn_hepar2, bn_insurance, bn_link and 6 more.

**Pluck.** Pluck ran 25 instances and answered 17 (1 fail, 7 run out of
memory). Its answers differ from the reference's by at most 1.4e-08
relative (4.1e-11 absolute, alarm) over 17 instances. Over the 25
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Pluck takes 4897 s in all and tddlog 1.94 s;
over the 17 both answer, 96.6 s against tddlog's 0.12. tddlog alone
answers bn_andes, bn_hailfinder, bn_insurance, bn_link, bn_munin and 3
more.

**PITA.** PITA ran 8 instances and answered 7 (1 time out). Its answers
differ from the reference's by at most 1.4e-08 relative (4.1e-11
absolute, alarm) over 7 instances. Over the 8 instances given to both
that either answers, a run that does not answer counted at the 600 s
limit, PITA takes 602 s in all and tddlog 0.18 s; over the 7 both
answer, 2.21 s against tddlog's 0.021. tddlog alone answers
digitRecognition.

**aspmc.** aspmc ran 8 instances and answered 7 (1 time out). Its
answers differ from the reference's by at most 1.4e-08 relative (4.1e-11
absolute, alarm) over 7 instances. Over the 8 instances given to both
that either answers, a run that does not answer counted at the 600 s
limit, aspmc takes 617 s in all and tddlog 0.18 s; over the 7 both
answer, 17.2 s against tddlog's 0.021. tddlog alone answers
digitRecognition.

**Storm sparse.** Storm sparse ran 25 instances and answered 22 (3 time
out). Its answers differ from the reference's by at most 1.0e+00
relative (5.8e-15 absolute, digitRecognition) over 22 instances. Over
the 25 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Storm sparse takes 2053 s in all and
tddlog 1.94 s; over the 22 both answer, 253 s against tddlog's 0.41.
tddlog alone answers bn_link, bn_munin and bn_pathfinder.

**Storm exact.** Storm exact ran 25 instances and answered 22 (3 time
out). Its answers differ from the reference's by at most 1.8e-08
relative (5.8e-15 absolute, digitRecognition) over 22 instances. Over
the 25 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Storm exact takes 2096 s in all and
tddlog 1.94 s; over the 22 both answer, 296 s against tddlog's 0.41.
tddlog alone answers bn_link, bn_munin and bn_pathfinder.

**Storm dd.** Storm dd ran 25 instances and answered 21 (4 time out).
Its answers differ from the reference's by at most 1.4e-08 relative
(4.1e-11 absolute, alarm) over 21 instances. Over the 25 instances given
to both that either answers, a run that does not answer counted at the
600 s limit, Storm dd takes 2901 s in all and tddlog 1.94 s; over the 21
both answer, 501 s against tddlog's 0.25. tddlog alone answers bn_link,
bn_munin, bn_pathfinder and digitRecognition.

**PRISM sparse.** PRISM sparse ran 25 instances and answered 22 (3 time
out). Its answers differ from the reference's by at most 1.0e+00
relative (5.8e-15 absolute, digitRecognition) over 22 instances. Over
the 25 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, PRISM sparse takes 2431 s in all and
tddlog 1.94 s; over the 22 both answer, 631 s against tddlog's 0.41.
tddlog alone answers bn_link, bn_munin and bn_pathfinder.

**PRISM hybrid.** PRISM hybrid ran 25 instances and answered 22 (3 time
out). Its answers differ from the reference's by at most 1.0e+00
relative (5.8e-15 absolute, digitRecognition) over 22 instances. Over
the 25 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, PRISM hybrid takes 2453 s in all and
tddlog 1.94 s; over the 22 both answer, 653 s against tddlog's 0.41.
tddlog alone answers bn_link, bn_munin and bn_pathfinder.

**Ace.** Ace ran 25 instances and answered 24 (1 run out of memory). Its
answers differ from the reference's by at most 1.4e-08 relative (4.1e-11
absolute, alarm) over 23 instances. Over the 25 instances given to both
that either answers, a run that does not answer counted at the 600 s
limit, Ace takes 691 s in all and tddlog 2.83 s; over the 24 both
answer, 91.4 s against tddlog's 2.56. tddlog alone answers
digitRecognition.

**Ace c2d.** Ace c2d ran 24 instances and answered 22 (1 fail, 1 time
out). Its answers differ from the reference's by at most 1.4e-08
relative (4.1e-11 absolute, alarm) over 22 instances. Over the 24
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Ace c2d takes 1250 s in all and tddlog 1.95
s; over the 22 both answer, 49.8 s against tddlog's 0.73. tddlog alone
answers bn_link and digitRecognition.

**Merlin.** Merlin ran 25 instances and answered 25. Its answers differ
from the reference's by at most 1.0e+00 relative (5.0e-07 absolute,
bn_water) over 24 instances. Over the 25 instances given to both that
either answers, Merlin takes 107 s in all and tddlog 2.83 s. Merlin is
faster on 1 (bn_pathfinder 0.071 s against 0.37).

**pyAgrum.** pyAgrum ran 25 instances and answered 25. Its answers
differ from the reference's by at most 1.0e+00 relative (1.8e-23
absolute, digitRecognition) over 24 instances. Over the 25 instances
given to both that either answers, pyAgrum takes 220 s in all and tddlog
2.83 s. pyAgrum is faster on 1 (bn_pathfinder 0.23 s against 0.37).

**pgmpy.** pgmpy ran 24 instances and answered 22 (2 time out). Its
answers differ from the reference's by at most 1.4e-08 relative (4.1e-11
absolute, alarm) over 22 instances. Over the 24 instances given to both
that either answers, a run that does not answer counted at the 600 s
limit, pgmpy takes 1274 s in all and tddlog 1.95 s; over the 22 both
answer, 74.4 s against tddlog's 0.73. tddlog alone answers bn_link and
digitRecognition.

**libDAI.** libDAI ran 25 instances and answered 24 (1 time out). Its
answers differ from the reference's by at most 1.4e-08 relative (4.1e-11
absolute, alarm) over 23 instances. Over the 25 instances given to both
that either answers, a run that does not answer counted at the 600 s
limit, libDAI takes 608 s in all and tddlog 2.83 s; over the 24 both
answer, 8.43 s against tddlog's 2.56. libDAI is faster on 1
(bn_pathfinder 0.068 s against 0.37). tddlog alone answers
digitRecognition.

**TensorInference.jl greedy.** TensorInference.jl greedy ran 24
instances and answered 22 (2 time out). Its answers differ from the
reference's by at most 1.4e-08 relative (4.1e-11 absolute, alarm) over
22 instances. Over the 24 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, TensorInference.jl
greedy takes 1804 s in all and tddlog 1.95 s; over the 22 both answer,
604 s against tddlog's 0.73. tddlog alone answers bn_link and
digitRecognition.

**TensorInference.jl treesa.** TensorInference.jl treesa ran 25
instances and answered 24 (1 time out). Its answers differ from the
reference's by at most 1.4e-08 relative (4.1e-11 absolute, alarm) over
23 instances. Over the 25 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, TensorInference.jl
treesa takes 1744 s in all and tddlog 2.83 s; over the 24 both answer,
1144 s against tddlog's 2.56. tddlog alone answers digitRecognition.

| instance | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Merlin | pyAgrum | pgmpy | libDAI | TI greedy | TI treesa | ProbLog | ref | tddlog diff | Geni diff | noDice diff | Psi diff | Genfer diff | SPPL diff | Pluck diff | PITA diff | aspmc diff | Storm sparse diff | Storm exact diff | Storm dd diff | PRISM sparse diff | PRISM hybrid diff | Ace diff | Ace c2d diff | Merlin diff | pyAgrum diff | pgmpy diff | libDAI diff | TI greedy diff | TI treesa diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| bn_alarm | 0.01 | 0.213 | >600 | >600 | 0.221 | 271 | 7.31 |  |  | 1.84 | 1.84 | 9.68 | 14 | 14 | 1.19 | 1.27 | 0.005 | 0.123 | 1.64 | 0.005 | 26.1 | 30.5 | err | exact | 6.7e-16 / 1.3e-14 | 2.0e-07 / 3.6e-06 |  |  | 2.2e-16 / 5.8e-16 | 3.6e-16 / 2.2e-15 | 2.2e-16 / 8.0e-16 |  |  | 2.2e-16 / 8.7e-16 | 1.1e-16 / 5.8e-16 | 3.3e-16 / 1.0e-15 | 6.7e-16 / 9.1e-16 | 6.7e-16 / 9.1e-16 | 3.3e-16 / 5.1e-16 | 3.3e-16 / 4.7e-16 | 4.9e-07 / 1.7e-05 | 4.4e-13 / 2.3e-12 | 4.4e-13 / 2.3e-12 | 4.4e-13 / 2.3e-12 | 1.7e-16 / 8.7e-16 | 1.7e-16 / 8.7e-16 |
| bn_andes | 0.03 | >600 | oom | oom | >600 | >600 | oom |  |  | 155 | 197 | 112 | 53.4 | 51.7 | 3.07 | 6.31 | 0.095 | 0.161 | 3.52 | 0.067 | 59.1 | 94 | >600 | exact | 2.2e-16 / 1.4e-15 |  |  |  |  |  |  |  |  | 2.0e-15 / 2.8e-15 | 1.1e-16 / 3.9e-16 | 2.0e-15 / 2.5e-15 | 2.0e-15 / 2.5e-15 | 2.0e-15 / 2.5e-15 | 5.6e-16 / 7.0e-16 | 4.4e-16 / 7.9e-16 | 4.6e-07 / 4.0e-06 | 4.7e-13 / 4.1e-12 | 4.7e-13 / 4.1e-12 | 4.7e-13 / 4.1e-12 | 1.1e-16 / 5.2e-16 | 2.2e-16 / 3.7e-16 |
| bn_asia | 0.003 | 0.01 | 0.021 | 5.26 | 0.007 | 0.619 | 4.91 |  |  | 0.304 | 0.304 | 0.363 | 2.55 | 2.54 | 1.02 | 1.08 | 0.004 | 0.127 | 1.69 | 0.005 | 21.4 | 23.3 | 0.088 | exact | 1.1e-16 / 2.5e-16 | 2.9e-08 / 2.6e-07 | 4.0e-07 / 9.2e-07 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.0e-16 | 1.1e-16 / 2.0e-16 | 1.1e-16 / 2.5e-16 |  |  | 1.1e-16 / 2.5e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.0e-16 | 1.1e-16 / 2.5e-16 | 1.1e-16 / 2.5e-16 | 1.1e-16 / 2.5e-16 | 5.6e-17 / 1.3e-16 | 4.0e-07 / 9.2e-07 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 5.6e-17 / 1.3e-16 | 5.6e-17 / 1.3e-16 |
| bn_cancer | 0.003 | 0.011 | 0.019 | 0.592 | 0.006 | 0.608 | 4.48 |  |  | 0.305 | 0.303 | 0.366 | 2.55 | 2.56 | 1.1 | 1.15 | 0.004 | 0.13 | 1.73 | 0.004 | 18.6 | 21.3 | 0.086 | exact | 1.7e-16 / 5.5e-16 | 5.6e-17 / 1.8e-16 | 5.0e-07 / 1.6e-06 | 5.6e-17 / 1.8e-16 | 1.1e-16 / 1.4e-16 | 1.1e-16 / 1.8e-16 | 1.1e-16 / 2.7e-16 |  |  | 2.2e-16 / 3.2e-16 | 5.6e-17 / 1.8e-16 | 2.2e-16 / 3.2e-16 | 2.2e-16 / 3.2e-16 | 2.2e-16 / 3.2e-16 | 1.1e-16 / 1.6e-16 | 1.1e-16 / 1.6e-16 | 5.0e-07 / 1.6e-06 | 5.6e-17 / 1.8e-16 | 5.6e-17 / 1.8e-16 | 5.6e-17 / 1.8e-16 | 1.1e-16 / 3.7e-16 | 1.1e-16 / 3.7e-16 |
| bn_child | 0.007 | 0.068 | 0.342 | >600 | 0.035 | 43.5 | 6.85 |  |  | 1.1 | 1.13 | 2.86 | 8.92 | 8.93 | 1.17 | 1.31 | 0.006 | 0.124 | 1.74 | 0.004 | 23.5 | 26.3 | 0.678 | exact | 6.1e-16 / 3.9e-15 | 7.5e-08 / 6.9e-07 | 6.2e-07 / 4.5e-06 |  | 1.1e-16 / 3.9e-16 | 2.8e-16 / 1.1e-15 | 2.2e-16 / 7.8e-16 |  |  | 2.8e-16 / 7.4e-16 | 1.1e-16 / 3.4e-16 | 2.2e-16 / 6.4e-16 | 2.2e-16 / 4.3e-16 | 2.2e-16 / 4.3e-16 | 2.2e-16 / 5.6e-16 | 3.3e-16 / 5.8e-16 | 5.0e-07 / 4.5e-06 | 5.0e-13 / 2.4e-12 | 5.0e-13 / 2.4e-12 | 5.0e-13 / 2.4e-12 | 1.1e-16 / 3.9e-16 | 1.1e-16 / 4.9e-16 |
| bn_earthquake | 0.003 | 0.01 | 0.019 | 0.697 | 0.006 | 0.6 | 4.45 |  |  | 0.294 | 0.304 | 0.344 | 2.56 | 2.55 | 1.15 | 1.18 | 0.004 | 0.135 | 1.75 | 0.007 | 18.3 | 20.7 | 0.085 | exact | 5.9e-17 / 2.8e-15 | 2.4e-08 / 3.7e-07 | 2.0e-07 / 9.6e-06 | 1.4e-17 / 4.9e-16 | 1.1e-16 / 3.3e-16 | 1.1e-16 / 1.3e-15 | 4.2e-17 / 1.5e-15 |  |  | 1.1e-16 / 3.3e-16 | 1.4e-17 / 4.9e-16 | 1.1e-16 / 4.4e-16 | 1.1e-16 / 4.9e-16 | 1.1e-16 / 4.9e-16 | 1.1e-16 / 3.3e-16 | 1.1e-16 / 3.3e-16 | 2.0e-07 / 9.6e-06 | 1.4e-17 / 4.9e-16 | 1.4e-17 / 4.9e-16 | 1.4e-17 / 4.9e-16 | 1.4e-17 / 6.6e-16 | 6.9e-18 / 3.3e-16 |
| bn_hailfinder | 0.034 | 10.5 | >600 | oom | 158 | >600 | oom |  |  | 18.2 | 18.6 | 94.5 | 37.7 | 41.4 | 1.51 | 1.7 | 0.007 | 0.14 | 1.59 | 0.007 | 27.3 | 33.1 | >600 | exact | 7.8e-16 / 9.0e-15 | 1.9e-07 / 1.2e-06 |  |  | 1.1e-16 / 4.4e-16 |  |  |  |  | 5.0e-16 / 1.1e-15 | 1.1e-16 / 4.4e-16 | 1.1e-16 / 4.4e-16 | 1.1e-16 / 4.4e-16 | 1.1e-16 / 4.4e-16 | 1.7e-16 / 4.4e-16 | 1.1e-16 / 4.0e-16 | 4.8e-07 / 3.7e-06 | 4.9e-13 / 1.6e-12 | 4.9e-13 / 1.6e-12 | 4.9e-13 / 1.6e-12 | 1.1e-16 / 8.8e-16 | 1.1e-16 / 6.6e-16 |
| bn_hepar2 | 0.024 | 0.507 | 9.24 | >600 | 0.665 | >600 | 8.73 |  |  | 7.33 | 7.46 | 37.2 | 52.3 | 52.2 | 1.5 | 1.71 | 0.008 | 0.134 | 1.9 | 0.009 | 32.4 | 41.9 | 14 | exact | 1.1e-15 / 5.1e-14 | 1.2e-07 / 1.8e-05 | 9.7e-07 / 2.4e-04 |  | 2.2e-16 / 1.0e-15 |  | 3.3e-16 / 4.0e-15 |  |  | 1.6e-15 / 2.2e-15 | 1.1e-16 / 1.0e-15 | 4.4e-16 / 1.1e-15 | 4.4e-16 / 1.0e-15 | 4.4e-16 / 1.0e-15 | 3.3e-16 / 1.0e-15 | 4.4e-16 / 7.7e-16 | 4.9e-07 / 1.2e-04 | 4.8e-13 / 4.4e-12 | 4.8e-13 / 4.4e-12 | 4.8e-13 / 4.4e-12 | 2.2e-16 / 8.1e-16 | 2.2e-16 / 8.1e-16 |
| bn_insurance | 0.017 | 8.61 | 8.38 | >600 | 10.4 | >600 | oom |  |  | 28.5 | 29.4 | 34.8 | 14.3 | 21.4 | 1.69 | 1.9 | 0.007 | 0.131 | 1.74 | 0.008 | 30.8 | 34.9 | >600 | exact | 1.5e-11 / 2.7e-11 | 1.1e-07 / 4.8e-06 | 7.1e-07 / 4.5e-05 |  | 1.5e-11 / 2.7e-11 |  |  |  |  | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 4.9e-07 / 4.5e-05 | 1.3e-11 / 2.6e-11 | 1.3e-11 / 2.6e-11 | 1.3e-11 / 2.6e-11 | 1.3e-11 / 2.6e-11 | 1.3e-11 / 2.6e-11 |
| bn_link | 0.727 | >600 | oom | oom | oom | >600 | oom |  |  | >600 | >600 | >600 | >600 | >600 | 51.4 | >600 | 33 | 85.6 | >600 | 4.08 | >600 | 175 | >600 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| bn_munin | 0.557 | >600 | oom | >600 | >600 | >600 | oom |  |  | >600 | >600 | >600 | >600 | >600 | 7.26 | - | 2.37 | 1.04 | - | 3.37 | - | 194 | >600 | exact | 8.2e-15 / 1.9e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  | 6.7e-16 / 2.4e-13 |  | 5.0e-07 / 3.0e-02 | 5.0e-13 / 4.7e-12 |  | 5.0e-13 / 4.7e-12 |  | 3.3e-16 / 2.5e-13 |
| bn_pathfinder | 0.236 | 192 | oom | >600 | >600 | >600 | err |  |  | >600 | >600 | >600 | >600 | >600 | 2.3 | 4.12 | 0.071 | 0.231 | 4.16 | 0.068 | 31.8 | 43.8 | >600 | exact | 1.1e-15 / 2.2e-12 | 9.9e-07 / 4.3e-03 |  |  |  |  |  |  |  |  |  |  |  |  | 4.4e-16 / 1.6e-14 | 5.6e-16 / 1.7e-14 | 5.0e-07 / 2.6e-03 | 4.9e-13 / 4.1e-12 | 4.9e-13 / 4.1e-12 | 4.9e-13 / 4.1e-12 | 2.8e-16 / 1.6e-14 | 2.2e-16 / 1.6e-14 |
| bn_pigs | 0.053 | 1.81 | 143 | >600 | 1.14 | >600 | oom |  |  | 22.3 | 22.2 | 63.9 | 179 | 179 | 3.49 | 13.5 | 0.145 | 0.212 | 34.9 | 0.135 | 60.7 | 113 | >600 | exact | 1.1e-16 / 2.4e-16 | 5.0e-08 / 1.0e-07 | 7.5e-07 / 3.0e-06 |  | 1.1e-16 / 2.4e-16 |  |  |  |  | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 5.0e-07 / 1.9e-06 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 4.4e-16 | 1.1e-16 / 2.4e-16 |
| bn_sachs | 0.006 | 0.032 | 0.13 | >600 | 0.018 | 1.82 | 6.79 |  |  | 0.62 | 0.614 | 1.43 | 5.08 | 5.1 | 1.17 | 1.23 | 0.004 | 0.13 | 1.7 | 0.004 | 24.5 | 25.4 | 0.186 | exact | 1.2e-10 / 4.0e-10 | 8.4e-08 / 1.1e-06 | 8.1e-07 / 1.0e-05 |  | 1.2e-10 / 4.0e-10 | 1.2e-10 / 4.0e-10 | 1.2e-10 / 4.0e-10 |  |  | 1.2e-10 / 4.0e-10 | 1.2e-10 / 4.0e-10 | 1.2e-10 / 4.0e-10 | 1.2e-10 / 4.0e-10 | 1.2e-10 / 4.0e-10 | 1.2e-10 / 4.0e-10 | 1.2e-10 / 4.0e-10 | 4.6e-07 / 3.2e-06 | 1.2e-10 / 1.9e-10 | 1.2e-10 / 1.9e-10 | 1.2e-10 / 1.9e-10 | 1.2e-10 / 1.9e-10 | 1.2e-10 / 1.9e-10 |
| bn_survey | 0.003 | 0.006 | 0.012 | 1.11 | 0.003 | 0.64 | 5.16 |  |  | 0.155 | 0.157 | 0.265 | 1.27 | 1.27 | 1.06 | 1.08 | 0.006 | 0.126 | 1.68 | 0.004 | 21 | 22.6 | 0.088 | exact | 2.5e-16 / 1.6e-15 | 3.2e-08 / 7.8e-08 | 7.5e-07 / 2.7e-06 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.0e-16 | 1.1e-16 / 3.5e-16 | 1.1e-16 / 4.0e-16 |  |  | 1.1e-16 / 3.5e-16 | 0.0e+00 / 0.0e+00 | 2.8e-17 / 1.8e-16 | 2.8e-17 / 1.8e-16 | 2.8e-17 / 1.8e-16 | 2.8e-17 / 1.8e-16 | 2.8e-17 / 1.8e-16 | 2.5e-07 / 1.4e-06 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 5.6e-17 / 2.0e-16 | 1.1e-16 / 2.0e-16 |
| bn_water | 0.022 | 12.4 | 8.35 | oom | 7.68 | >600 | 7.96 |  |  | 10.7 | 10.8 | 135 | 42.8 | 43.5 | 1.98 | 2.5 | 0.392 | 0.305 | 1.93 | 0.318 | 36.8 | 43.3 | >600 | exact | 4.4e-16 / 1.4e-10 | 1.3e-07 / 1.0e+00 | 7.5e-07 / 1.0e+00 |  | 3.3e-16 / 1.4e-10 |  | 2.2e-16 / 1.4e-10 |  |  | 4.4e-16 / 1.4e-10 | 1.1e-16 / 1.4e-10 | 3.3e-16 / 1.4e-10 | 5.6e-16 / 1.4e-10 | 5.6e-16 / 1.4e-10 | 4.4e-16 / 1.4e-10 | 3.3e-16 / 1.4e-10 | 5.0e-07 / 1.0e+00 | 4.6e-13 / 1.4e-10 | 4.6e-13 / 1.4e-10 | 4.6e-13 / 1.4e-10 | 2.2e-16 / 1.4e-10 | 2.2e-16 / 1.4e-10 |
| bn_win95pts | 0.011 | 0.771 | 1.27 | >600 | 3.1 | >600 | 7.97 |  |  | 2.67 | 2.69 | 7.17 | 20.7 | 20.6 | 1.35 | 1.41 | 0.007 | 0.13 | 1.73 | 0.008 | 36.2 | 45.2 | 152 | exact | 2.2e-16 / 3.5e-14 | 1.4e-07 / 1.3e-05 | 5.3e-07 / 4.6e-06 |  | 2.2e-16 / 7.0e-16 |  | 2.2e-16 / 7.0e-16 |  |  | 3.3e-16 / 7.0e-16 | 1.1e-16 / 7.0e-16 | 2.2e-16 / 8.1e-16 | 1.1e-16 / 7.0e-16 | 1.1e-16 / 7.0e-16 | 5.6e-16 / 8.5e-16 | 4.4e-16 / 8.3e-16 | 4.7e-07 / 4.0e-06 | 4.9e-13 / 3.3e-12 | 4.9e-13 / 3.3e-12 | 4.9e-13 / 3.3e-12 | 1.7e-16 / 5.3e-16 | 1.1e-16 / 1.0e-15 |
| dice_alarm | 0.003 | 0.005 | 0.009 | 0.069 | 0.004 | 0.573 | 4.66 | 0.313 | 2.45 | 0.146 | 0.147 | 0.19 | 1.26 | 1.26 | 1.2 | 1.25 | 0.052 | 0.167 | 1.56 | 0.053 | 20.6 | 24.3 | 0.07 | problog | 4.1e-11 / 1.4e-08 | 1.8e-08 / 6.0e-06 | 4.5e-07 / 1.5e-04 | 4.1e-11 / 1.4e-08 | 4.1e-11 / 1.4e-08 | 4.1e-11 / 1.4e-08 | 4.1e-11 / 1.4e-08 | 4.1e-11 / 1.4e-08 | 4.1e-11 / 1.4e-08 | 4.1e-11 / 1.4e-08 | 4.1e-11 / 1.4e-08 | 4.1e-11 / 1.4e-08 | 4.1e-11 / 1.4e-08 | 4.1e-11 / 1.4e-08 | 4.1e-11 / 1.4e-08 | 4.1e-11 / 1.4e-08 | 4.5e-07 / 1.5e-04 | 4.1e-11 / 1.4e-08 | 4.1e-11 / 1.4e-08 | 4.1e-11 / 1.4e-08 | 4.1e-11 / 1.4e-08 | 4.1e-11 / 1.4e-08 |
| dice_digitRecognition | 0.161 | err | 17 | oom | 0.086 | >600 | oom | >600 | >600 | 2.15 | 2.23 | >600 | 185 | 197 | oom | err | 70.7 | 130 | >600 | >600 | >600 | >600 | 23.1 | problog | 5.8e-15 / 1.8e-08 |  | 5.7e-15 / 1.0e+00 |  | 5.8e-15 / 1.9e+02 |  |  |  |  | 5.8e-15 / 1.0e+00 | 5.8e-15 / 1.8e-08 |  | 5.8e-15 / 1.0e+00 | 5.8e-15 / 1.0e+00 |  |  | 5.7e-15 / 1.0e+00 | 1.8e-23 / 1.0e+00 |  |  |  |  |
| dice_evidence1 | 0.002 | 0.005 | 0.008 | 0.021 | 0.003 | 0.559 | 4.47 | 0.313 | 2.45 | 0.146 | 0.143 | 0.142 | 1.27 | 1.27 | 1.08 | 1.18 | 0.042 | 0.149 | 1.55 | 0.043 | 18.4 | 20.2 | 0.069 | problog | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-07 / 9.9e-07 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-07 / 9.9e-07 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 |
| dice_evidence2 | 0.002 | 0.005 | 0.008 | 0.022 | 0.004 | 0.556 | 4.61 | 0.317 | 2.45 | 0.144 | 0.147 | 0.145 | 1.26 | 1.26 | 1.11 | 1.19 | 0.044 | 0.152 | 1.57 | 0.045 | 19.3 | 21.5 | 0.069 | problog | 3.3e-09 / 5.0e-09 | 3.3e-09 / 5.0e-09 | 3.3e-07 / 4.9e-07 | 3.3e-09 / 5.0e-09 | 3.3e-09 / 5.0e-09 | 3.3e-09 / 5.0e-09 | 3.3e-09 / 5.0e-09 | 3.3e-09 / 5.0e-09 | 3.3e-09 / 5.0e-09 | 3.3e-09 / 5.0e-09 | 3.3e-09 / 5.0e-09 | 3.3e-09 / 5.0e-09 | 3.3e-09 / 5.0e-09 | 3.3e-09 / 5.0e-09 | 3.3e-09 / 5.0e-09 | 3.3e-09 / 5.0e-09 | 3.3e-07 / 4.9e-07 | 3.3e-09 / 5.0e-09 | 3.3e-09 / 5.0e-09 | 3.3e-09 / 5.0e-09 | 3.3e-09 / 5.0e-09 | 3.3e-09 / 5.0e-09 |
| dice_grass | 0.002 | 0.005 | 0.009 | 0.053 | 0.006 | 0.587 | 4.66 | 0.323 | 2.45 | 0.144 | 0.144 | 0.187 | 1.27 | 1.26 | 1.18 | 1.21 | 0.045 | 0.156 | 1.58 | 0.045 | 20.5 | 23.9 | 0.071 | problog | 2.7e-09 / 3.8e-09 | 2.7e-09 / 3.8e-09 | 6.8e-07 / 9.6e-07 | 2.7e-09 / 3.8e-09 | 2.7e-09 / 3.8e-09 | 2.7e-09 / 3.8e-09 | 2.7e-09 / 3.8e-09 | 2.7e-09 / 3.8e-09 | 2.7e-09 / 3.8e-09 | 2.7e-09 / 3.8e-09 | 2.7e-09 / 3.8e-09 | 2.7e-09 / 3.8e-09 | 2.7e-09 / 3.8e-09 | 2.7e-09 / 3.8e-09 | 2.7e-09 / 3.8e-09 | 2.7e-09 / 3.8e-09 | 3.2e-07 / 4.5e-07 | 2.7e-09 / 3.8e-09 | 2.7e-09 / 3.8e-09 | 2.7e-09 / 3.8e-09 | 2.7e-09 / 3.8e-09 | 2.7e-09 / 3.8e-09 |
| dice_murderMystery | 0.003 | 0.005 | 0.009 | err | 0.003 | 0.565 | 4.56 | 0.312 | 2.45 | 0.145 | 0.148 | 0.147 | 1.26 | 1.27 | 1.12 | 1.14 | 0.043 | 0.152 | 1.57 | 0.044 | 19.5 | 21.3 | 0.068 | problog | 2.0e-10 / 1.3e-08 | 2.0e-10 / 1.3e-08 | 2.2e-07 / 1.4e-05 |  | 2.0e-10 / 1.3e-08 | 2.0e-10 / 1.3e-08 | 2.0e-10 / 1.3e-08 | 2.0e-10 / 1.3e-08 | 2.0e-10 / 1.3e-08 | 2.0e-10 / 1.3e-08 | 2.0e-10 / 1.3e-08 | 2.0e-10 / 1.3e-08 | 2.0e-10 / 1.3e-08 | 2.0e-10 / 1.3e-08 | 2.0e-10 / 1.3e-08 | 2.0e-10 / 1.3e-08 | 2.2e-07 / 1.4e-05 | 2.0e-10 / 1.3e-08 | 2.0e-10 / 1.3e-08 | 2.0e-10 / 1.3e-08 | 2.0e-10 / 1.3e-08 | 2.0e-10 / 1.3e-08 |
| dice_noisyOr | 0.003 | 0.005 | 0.01 | 1.26 | 0.015 | 0.613 | 4.54 | 0.318 | 2.46 | 0.144 | 0.143 | 0.211 | 1.26 | 1.26 | 1.15 | 1.23 | 0.045 | 0.153 | 1.57 | 0.045 | 20.9 | 26 | 0.072 | problog | 0.0e+00 / 0.0e+00 | 5.0e-08 / 6.1e-08 | 7.5e-07 / 9.2e-07 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.4e-16 | 1.1e-16 / 1.4e-16 | 0.0e+00 / 0.0e+00 | 2.2e-16 / 2.7e-16 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 2.2e-16 / 2.7e-16 | 2.2e-16 / 2.7e-16 | 2.2e-16 / 2.7e-16 | 3.3e-16 / 4.1e-16 | 2.2e-16 / 2.7e-16 | 2.5e-07 / 3.1e-07 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.4e-16 | 1.1e-16 / 1.4e-16 |
| dice_twocoins | 0.002 | 0.005 | 0.008 | 0.023 | 0.003 | 0.561 | 4.49 | 0.315 | 2.45 | 0.151 | 0.151 | 0.141 | 1.27 | 1.27 | 1.11 | 1.16 | 0.054 | 0.176 | 1.57 | 0.055 | 16.6 | 18.4 | 0.07 | problog | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-07 / 9.9e-07 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-07 / 9.9e-07 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 | 3.3e-09 / 1.0e-08 |

Seconds per system, the median over the repetitions; a failure is `>600`
(timeout), `oom`, `err` or `n/a`; each system's last columns are its
largest absolute and relative difference from the reference named in
`ref`.

### On the Dice paper's loop programs

| family | instances | tddlog | Geni | noDice | Psi | Genfer | SPPL | PITA | aspmc | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| caesar-iter | 3 | 3 | 0 of 1 | 2 of 3 | 0 of 1 | 2 of 3 | 1 of 2 | 0 of 2 | 1 of 2 | 2 of 3 | 3 of 3 | 2 of 3 | 2 of 3 | 2 of 3 |
| diamond | 4 | 4 | 1 of 4 | 3 of 4 | 0 of 1 | 3 of 4 | 3 of 4 | 2 of 3 | 4 of 4 | 4 of 4 | 2 of 3 | 4 of 4 | 4 of 4 | 4 of 4 |
| ladder | 4 | 4 | 3 of 4 | 0 of 1 | 0 of 1 | 2 of 3 | 3 of 4 | 1 of 2 | 3 of 4 | 4 of 4 | 2 of 3 | 4 of 4 | 4 of 4 | 4 of 4 |
| pmc1 | 4 | 4 | 3 of 4 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 4 of 4 | 4 of 4 | 2 of 3 | 2 of 3 | 4 of 4 | 4 of 4 | 4 of 4 |
| pmc2 | 3 | 3 | 2 of 3 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 3 of 3 | 3 of 3 | 3 of 3 | 3 of 3 | 3 of 3 | 3 of 3 | 3 of 3 |
| simpleCaesar | 3 | 3 | 0 of 1 | 1 of 3 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 1 of 2 | 1 of 2 | 3 of 3 | 1 of 2 | 1 of 2 | 1 of 2 |

**Geni.** Geni ran 17 instances and answered 9 (2 fail, 2 time out, 4
answer wrongly). Its answers differ from the reference's by at most
4.4e-05 relative (8.9e-07 absolute, ladder_k6400) over 9 instances.
Geni's answer is wrong, zero or underflowed on diamond_k1600,
diamond_k400, diamond_k6400 and ladder_k1600. Over the 17 instances
given to both that either answers, a run that does not answer counted at
the 600 s limit, Geni takes 4804 s in all and tddlog 3.3 s; over the 9
both answer, 4.48 s against tddlog's 1.47. tddlog alone answers
caesar-iter_k100, diamond_k1600, diamond_k400, diamond_k6400,
ladder_k1600 and 3 more.

**noDice.** noDice ran 13 instances and answered 6 (3 run out of memory,
3 time out, 1 answer wrongly). Its answers differ from the reference's
by at most 1.0e+00 relative (7.9e-31 absolute, caesar-iter_k100) over 6
instances. noDice's answer is wrong, zero or underflowed on
simpleCaesar_k300. Over the 13 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, noDice
takes 4227 s in all and tddlog 8.28 s; over the 6 both answer, 26.8 s
against tddlog's 0.72. tddlog alone answers caesar-iter_k3000,
diamond_k6400, ladder_k100, pmc1_k10, pmc2_k10 and 2 more.

**Psi.** Psi ran 6 instances and answered 0 (6 fail). Over the 6
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Psi takes 3600 s in all and tddlog 0.25 s.
tddlog alone answers caesar-iter_k100, diamond_k100, ladder_k100,
pmc1_k10, pmc2_k10 and 1 more.

**Genfer.** Genfer ran 14 instances and answered 8 (3 fail, 3 time out).
Its answers differ from the reference's by at most 1.0e+00 relative
(9.3e-302 absolute, caesar-iter_k1000) over 8 instances. Over the 14
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Genfer takes 3983 s in all and tddlog 3.43
s; over the 8 both answer, 383 s against tddlog's 0.92. tddlog alone
answers caesar-iter_k3000, diamond_k6400, ladder_k1600, pmc1_k10,
pmc2_k10 and 1 more.

**SPPL.** SPPL ran 13 instances and answered 7 (3 run out of memory, 3
time out). Its answers differ from the reference's by at most 6.1e-09
relative (2.2e-16 absolute, caesar-iter_k100) over 7 instances. Over the
13 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, SPPL takes 4051 s in all and tddlog
3.06 s; over the 7 both answer, 451 s against tddlog's 0.87. tddlog
alone answers caesar-iter_k1000, diamond_k6400, ladder_k6400, pmc1_k10,
pmc2_k10 and 1 more.

**PITA.** PITA ran 15 instances and answered 10 (4 time out, 1 answer
wrongly). Its answers differ from the reference's by at most 4.8e-09
relative (4.8e-09 absolute, diamond_k100) over 10 instances. PITA's
answer is wrong, zero or underflowed on caesar-iter_k100. Over the 15
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, PITA takes 3172 s in all and tddlog 1.42 s;
over the 10 both answer, 172 s against tddlog's 0.63. tddlog alone
answers caesar-iter_k100, caesar-iter_k1000, diamond_k1600, ladder_k400
and simpleCaesar_k100.

**aspmc.** aspmc ran 19 instances and answered 16 (3 time out). Its
answers differ from the reference's by at most 6.1e-09 relative (4.8e-39
absolute, caesar-iter_k100) over 16 instances. Over the 19 instances
given to both that either answers, a run that does not answer counted at
the 600 s limit, aspmc takes 2239 s in all and tddlog 4.36 s; over the
16 both answer, 439 s against tddlog's 2.08. tddlog alone answers
caesar-iter_k1000, ladder_k6400 and simpleCaesar_k300.

**Storm sparse.** Storm sparse ran 19 instances and answered 16 (2 fail,
1 time out). Its answers differ from the reference's by at most 1.0e+00
relative (9.3e-302 absolute, caesar-iter_k1000) over 16 instances. Over
the 19 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Storm sparse takes 1881 s in all and
tddlog 4.55 s; over the 16 both answer, 81.4 s against tddlog's 2.99.
Storm sparse is faster on 4 (ladder_k1600 0.32 s against 0.39,
caesar-iter_k1000 0.17 s against 0.26, diamond_k6400 0.19 s against 0.46
and ladder_k6400 0.36 s against 1.15). tddlog alone answers
caesar-iter_k3000, pmc1_k40 and simpleCaesar_k300.

**Storm exact.** Storm exact ran 18 instances and answered 15 (2 fail, 1
time out). Its answers differ from the reference's by at most 2.2e-08
relative (4.5e-96 absolute, simpleCaesar_k300) over 15 instances. Over
the 18 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Storm exact takes 1904 s in all and
tddlog 8.29 s; over the 15 both answer, 104 s against tddlog's 7.74.
Storm exact is faster on 3 (caesar-iter_k1000 0.19 s against 0.26,
caesar-iter_k3000 0.27 s against 0.81 and simpleCaesar_k1000 3.73 s
against 5.36). tddlog alone answers diamond_k1600, ladder_k1600 and
pmc1_k40.

**Storm dd.** Storm dd ran 20 instances and answered 18 (2 fail). Its
answers differ from the reference's by at most 1.0e+00 relative
(9.3e-302 absolute, caesar-iter_k1000) over 18 instances. Over the 20
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Storm dd takes 1719 s in all and tddlog 4.96
s; over the 18 both answer, 519 s against tddlog's 3.4. tddlog alone
answers caesar-iter_k3000 and simpleCaesar_k300.

**PRISM sparse.** PRISM sparse ran 20 instances and answered 18 (2
fail). Its answers differ from the reference's by at most 1.0e+00
relative (9.3e-302 absolute, caesar-iter_k1000) over 18 instances. Over
the 20 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, PRISM sparse takes 1369 s in all and
tddlog 4.96 s; over the 18 both answer, 169 s against tddlog's 3.4.
tddlog alone answers caesar-iter_k3000 and simpleCaesar_k300.

**PRISM hybrid.** PRISM hybrid ran 20 instances and answered 18 (2
fail). Its answers differ from the reference's by at most 1.0e+00
relative (9.3e-302 absolute, caesar-iter_k1000) over 18 instances. Over
the 20 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, PRISM hybrid takes 1368 s in all and
tddlog 4.96 s; over the 18 both answer, 168 s against tddlog's 3.4.
tddlog alone answers caesar-iter_k3000 and simpleCaesar_k300.

| instance | tddlog | Geni | noDice | Psi | Genfer | SPPL | PITA | aspmc | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | ProbLog | ref | tddlog diff | Geni diff | noDice diff | Psi diff | Genfer diff | SPPL diff | PITA diff | aspmc diff | Storm sparse diff | Storm exact diff | Storm dd diff | PRISM sparse diff | PRISM hybrid diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| dice_caesar-iter_k100 | 0.02 | err | 0.161 | err | 0.04 | 14.9 | wrong | 13.7 | 0.151 | 0.153 | 0.61 | 1.27 | 1.27 | 0.412 | problog | 4.8e-39 / 6.1e-09 |  | 7.9e-31 / 1.0e+00 |  | 4.8e-39 / 6.1e-09 | 2.2e-16 / 6.1e-09 | 1.0e+00 / 1.6e+60 | 4.8e-39 / 6.1e-09 | 4.8e-39 / 6.1e-09 | 4.8e-39 / 6.1e-09 | 4.8e-39 / 6.1e-09 | 4.8e-39 / 6.1e-09 | 4.8e-39 / 6.1e-09 |
| dice_caesar-iter_k1000 | 0.289 | - | 17.5 | - | 0.399 | >600 | >600 | >600 | 0.175 | 0.191 | 2.45 | 1.26 | 1.27 | 5.78 | problog | 1.5e-310 / 1.6e-09 |  | 9.3e-302 / 1.0e+00 |  | 9.3e-302 / 1.0e+00 |  |  |  | 9.3e-302 / 1.0e+00 | 1.5e-310 / 1.6e-09 | 9.3e-302 / 1.0e+00 | 9.3e-302 / 1.0e+00 | 9.3e-302 / 1.0e+00 |
| dice_caesar-iter_k3000 | 0.888 | - | oom | - | err | - | - | - | err | 0.27 | err | err | err | 37.2 | problog | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |
| dice_diamond_k100 | 0.013 | 0.019 | 0.037 | err | 0.008 | 3 | 0.977 | 2.57 | 0.144 | 0.152 | 0.472 | 1.27 | 1.27 | 0.273 | problog | 4.8e-09 / 4.8e-09 | 7.5e-07 / 7.5e-07 | 6.5e-07 / 6.5e-07 |  | 4.8e-09 / 4.8e-09 | 4.8e-09 / 4.8e-09 | 4.8e-09 / 4.8e-09 | 4.8e-09 / 4.8e-09 | 4.8e-09 / 4.8e-09 | 4.8e-09 / 4.8e-09 | 4.8e-09 / 4.8e-09 | 4.8e-09 / 4.8e-09 | 4.8e-09 / 4.8e-09 |
| dice_diamond_k400 | 0.094 | wrong | 0.217 | - | 0.025 | 13.7 | 142 | 3.2 | 0.154 | 0.164 | 1.28 | 1.27 | 1.27 | 3.27 | problog | 3.2e-09 / 3.3e-09 | 3.0e-06 / 3.0e-06 | 1.8e-07 / 1.8e-07 |  | 3.2e-09 / 3.3e-09 | 3.2e-09 / 3.3e-09 | 3.2e-09 / 3.3e-09 | 3.2e-09 / 3.3e-09 | 3.2e-09 / 3.3e-09 | 3.2e-09 / 3.3e-09 | 3.2e-09 / 3.3e-09 | 3.2e-09 / 3.3e-09 | 3.2e-09 / 3.3e-09 |
| dice_diamond_k1600 | 0.201 | wrong | 3.52 | - | 0.101 | 145 | >600 | 10.3 | 0.163 | err | 2.74 | 1.28 | 1.27 | 233 | problog | 9.4e-11 / 1.0e-10 | 1.2e-05 / 1.3e-05 | 5.0e-07 / 5.4e-07 |  | 9.4e-11 / 1.0e-10 | 9.4e-11 / 1.0e-10 |  | 9.4e-11 / 1.0e-10 | 9.4e-11 / 1.0e-10 |  | 9.4e-11 / 1.0e-10 | 9.4e-11 / 1.0e-10 | 9.4e-11 / 1.0e-10 |
| dice_diamond_k6400 | 0.508 | wrong | oom | - | err | >600 | - | 118 | 0.193 | - | 11.5 | 5.75 | 5.54 | oom | tddlog | ref | 4.0e-05 / 5.5e-05 |  |  |  |  |  | 2.2e-14 / 3.1e-14 | 5.4e-13 / 7.5e-13 |  | 2.4e-14 / 3.3e-14 | 2.4e-14 / 3.3e-14 | 2.4e-14 / 3.3e-14 |
| dice_ladder_k100 | 0.04 | 0.06 | >600 | err | 0.075 | 4.13 | 20.1 | 2.82 | 0.302 | 0.305 | 2.7 | 2.53 | 2.54 | 0.559 | problog | 1.8e-09 / 3.7e-09 | 2.6e-07 / 5.5e-07 |  |  | 1.8e-09 / 3.7e-09 | 1.8e-09 / 3.7e-09 | 1.8e-09 / 3.7e-09 | 1.8e-09 / 3.7e-09 | 1.8e-09 / 3.7e-09 | 1.8e-09 / 3.7e-09 | 1.8e-09 / 3.7e-09 | 1.8e-09 / 3.7e-09 | 1.8e-09 / 3.7e-09 |
| dice_ladder_k400 | 0.176 | 0.234 | - | - | 0.289 | 23.8 | >600 | 7.08 | 0.311 | 0.331 | 6.65 | 2.55 | 2.55 | >600 | alea | 1.8e-15 / 4.5e-15 | 9.7e-07 / 2.4e-06 |  |  | 2.5e-15 / 6.1e-15 | 1.8e-15 / 4.3e-15 |  | 2.4e-15 / 6.0e-15 | 2.7e-15 / 6.5e-15 | 1.8e-15 / 4.5e-15 | 2.8e-15 / 6.9e-15 | 2.8e-15 / 6.9e-15 | 2.8e-15 / 6.9e-15 |
| dice_ladder_k1600 | 0.414 | wrong | - | - | err | 246 | - | 114 | 0.321 | err | 40.7 | 2.53 | 2.66 | >600 | alea | 4.0e-15 / 1.8e-14 | 1.9e-06 / 8.6e-06 |  |  |  | 3.2e-15 / 1.4e-14 |  | 4.7e-15 / 2.1e-14 | 4.5e-15 / 2.0e-14 |  | 4.7e-15 / 2.1e-14 | 4.7e-15 / 2.1e-14 | 4.7e-15 / 2.1e-14 |
| dice_ladder_k6400 | 1.24 | 4.15 | - | - | - | oom | - | >600 | 0.358 | - | 421 | 11.4 | 11.2 | oom | tddlog | ref | 8.9e-07 / 4.4e-05 |  |  |  |  |  |  | 1.1e-16 / 5.6e-15 |  | 1.9e-16 / 9.2e-15 | 1.9e-16 / 9.2e-15 | 1.9e-16 / 9.2e-15 |
| dice_pmc1_k10 | 0.003 | 0.005 | >600 | err | >600 | oom | 0.314 | 1.45 | 0.492 | 0.513 | 4.23 | 2 | 1.95 | 0.06 | problog | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| dice_pmc1_k20 | 0.003 | 0.005 | - | - | - | - | 0.313 | 1.44 | 76.4 | 94.1 | 4.11 | 2.04 | 2.04 | 0.06 | problog | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| dice_pmc1_k40 | 0.003 | 0.005 | - | - | - | - | 0.314 | 1.44 | >600 | >600 | 4.23 | 2.36 | 2.5 | 0.059 | problog | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| dice_pmc1_k75 | 0.505 | >600 | - | - | - | - | 5.18 | 35.3 | - | - | 5.44 | 125 | 123 | oom | alea | 0.0e+00 / 0.0e+00 |  |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| dice_pmc2_k10 | 0.003 | 0.005 | >600 | err | >600 | oom | 0.312 | 1.45 | 0.253 | 0.262 | 2.2 | 1.27 | 1.29 | 0.059 | problog | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| dice_pmc2_k20 | 0.003 | 0.005 | - | - | - | - | 0.313 | 1.44 | 0.456 | 0.522 | 2.27 | 1.33 | 1.39 | 0.059 | problog | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| dice_pmc2_k51 | 0.073 | >600 | - | - | - | - | 2.86 | 12.4 | 0.913 | 1.13 | 2.76 | 1.98 | 2.04 | 25.1 | problog | 0.0e+00 / 0.0e+00 |  |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| dice_simpleCaesar_k100 | 0.209 | err | 5.34 | err | 382 | >600 | >600 | 112 | 0.653 | 0.842 | 3.81 | 2.18 | 2.37 | 34 | problog | 6.7e-39 / 2.1e-09 |  | 5.9e-30 / 1.0e+00 |  | 6.7e-39 / 2.1e-09 |  |  | 6.7e-39 / 2.1e-09 | 6.7e-39 / 2.1e-09 | 6.7e-39 / 2.1e-09 | 6.7e-39 / 2.1e-09 | 6.7e-39 / 2.1e-09 | 6.7e-39 / 2.1e-09 |
| dice_simpleCaesar_k300 | 0.842 | - | wrong | - | >600 | - | - | >600 | err | 1.51 | err | err | err | 109 | problog | 4.5e-96 / 2.2e-08 |  | 5.0e-01 / 2.4e+87 |  |  |  |  |  |  | 4.5e-96 / 2.2e-08 |  |  |  |
| dice_simpleCaesar_k1000 | 6.3 | - | oom | - | - | - | - | - | - | 3.73 | - | - | - | >600 | tddlog | ref |  |  |  |  |  |  |  |  | 9.5e-307 / 1.7e-14 |  |  |  |

Seconds per system, the median over the repetitions; a failure is `>600`
(timeout), `oom`, `err` or `n/a`; each system's last columns are its
largest absolute and relative difference from the reference named in
`ref`.

### On Alea.jl's and noDice's programs

| family | instances | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Roulette |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| arith | 28 | 25 | 2 of 28 | 0 of 28 | 1 of 28 | 3 of 28 | 2 of 28 |  | 18 of 28 | 20 of 28 | 13 of 28 | 11 of 28 | 16 of 28 | 10 of 28 | 10 of 28 | 22 of 28 |
| hybit | 19 | 16 | 0 of 19 | 0 of 19 | 0 of 19 | 0 of 19 | 0 of 19 |  | 6 of 19 | 4 of 19 | 0 of 19 | 0 of 19 | 1 of 19 | 2 of 19 | 2 of 19 | 5 of 19 |
| nodice | 11 | 11 | 5 of 11 | 10 of 11 | 1 of 11 | 5 of 11 | 4 of 11 |  | 9 of 11 | 6 of 11 | 6 of 11 | 7 of 11 | 10 of 11 | 10 of 11 | 10 of 11 |  |
| pluck | 4 | 4 | 0 of 4 | 0 of 4 | 0 of 4 | 0 of 4 | 0 of 4 | 3 of 4 | 3 of 4 | 3 of 4 | 0 of 4 | 0 of 4 | 1 of 4 | 1 of 4 | 1 of 4 | 3 of 4 |

**Geni.** Geni ran 62 instances and answered 7 (10 run out of memory, 45
time out). Its answers differ from the reference's by at most 3.2e-06
relative (8.0e-07 absolute, arith_linext_small) over 7 instances. Over
the 54 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Geni takes 28251 s in all and tddlog
1031 s; over the 7 both answer, 50.9 s against tddlog's 2.19. Geni is
faster on 1 (hepar2 0.15 s against 0.27). tddlog alone answers
arith_book, arith_caesar_huge, arith_caesar_large, arith_caesar_medium,
arith_caesar_small and 42 more.

**noDice.** noDice ran 62 instances and answered 10 (9 fail, 27 run out
of memory, 14 time out, 2 print zeros). Its answers differ from the
reference's by at most 7.5e-05 relative (7.4e-07 absolute, water) over
10 instances. noDice's answer is wrong, zero or underflowed on 3sat-0
and string_editing. Over the 54 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, noDice
takes 27206 s in all and tddlog 1029 s; over the 10 both answer, 806 s
against tddlog's 2.61. tddlog alone answers arith_book,
arith_caesar_huge, arith_caesar_large, arith_caesar_medium,
arith_caesar_small and 39 more.

**Psi.** Psi ran 62 instances and answered 2 (6 fail, 30 run out of
memory, 24 time out). Its answers differ from the reference's by at most
1.3e-08 relative (4.0e-09 absolute, survey) over 2 instances. Over the
54 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Psi takes 31397 s in all and tddlog
1029 s; over the 2 both answer, 197 s against tddlog's 0.034. tddlog
alone answers arith_book, arith_caesar_huge, arith_caesar_large,
arith_caesar_medium, arith_caesar_small and 47 more.

**Genfer.** Genfer ran 62 instances and answered 8 (9 fail, 39 run out
of memory, 6 time out). Its answers differ from the reference's by at
most 4.6e-08 relative (4.9e-09 absolute, insurance) over 8 instances.
Over the 54 instances given to both that either answers, a run that does
not answer counted at the 600 s limit, Genfer takes 27838 s in all and
tddlog 1031 s; over the 8 both answer, 238 s against tddlog's 2.3.
tddlog alone answers arith_book, arith_caesar_huge, arith_caesar_large,
arith_caesar_medium, arith_caesar_small and 41 more.

**SPPL.** SPPL ran 62 instances and answered 6 (7 fail, 1 run out of
memory, 48 time out). Its answers differ from the reference's by at most
1.3e-08 relative (4.0e-09 absolute, survey) over 6 instances. Over the
54 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, SPPL takes 28975 s in all and tddlog
1031 s; over the 6 both answer, 175 s against tddlog's 1.88. tddlog
alone answers arith_book, arith_caesar_huge, arith_caesar_large,
arith_caesar_medium, arith_caesar_small and 43 more.

**Pluck.** Pluck ran 4 instances and answered 3 (1 fail). Its answers
differ from the reference's by at most 1.3e+04 relative (3.0e-08
absolute, string_editing) over 3 instances. Over the 4 instances given
to both that either answers, a run that does not answer counted at the
600 s limit, Pluck takes 616 s in all and tddlog 109 s; over the 3 both
answer, 16.2 s against tddlog's 101. Pluck is faster on 1 (pcfg 5.18 s
against 101). tddlog alone answers sorted_list.

**PITA.** PITA ran 62 instances and answered 36 (25 time out, 1 answer
wrongly). Its answers differ from the reference's by at most 1.0e+00
relative (1.3e-16 absolute, arith_plankton) over 36 instances. PITA's
answer is wrong, zero or underflowed on arith_caesar_small. Over the 54
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, PITA takes 13276 s in all and tddlog 1031 s;
over the 36 both answer, 2476 s against tddlog's 100. PITA is faster on
1 (sorted_list 2.18 s against 11.8). tddlog alone answers
arith_caesar_huge, arith_caesar_large, arith_caesar_medium,
arith_caesar_small, arith_disease_small and 13 more.

**aspmc.** aspmc ran 62 instances and answered 33 (2 run out of memory,
27 time out). Its answers differ from the reference's by at most 4.6e-08
relative (4.9e-09 absolute, insurance) over 33 instances. Over the 54
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, aspmc takes 15439 s in all and tddlog 1031
s; over the 33 both answer, 2839 s against tddlog's 89.2. aspmc is
faster on 1 (sorted_list 5.07 s against 11.8). tddlog alone answers
arith_caesar_huge, arith_caesar_large, arith_fw_large, arith_gcd_medium,
arith_triangle_large and 16 more.

**Storm sparse.** Storm sparse ran 62 instances and answered 19 (1 fail,
7 run out of memory, 35 time out). Its answers differ from the
reference's by at most 4.6e-08 relative (4.9e-09 absolute, insurance)
over 19 instances. Over the 54 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, Storm
sparse takes 22425 s in all and tddlog 595 s; over the 19 both answer,
1425 s against tddlog's 14.2. tddlog alone answers arith_book,
arith_caesar_huge, arith_caesar_large, arith_disease_small,
arith_fw_large and 30 more.

**Storm exact.** Storm exact ran 62 instances and answered 18 (11 fail,
8 run out of memory, 25 time out). Its answers differ from the
reference's by at most 4.6e-08 relative (4.9e-09 absolute, insurance)
over 18 instances. Over the 54 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, Storm
exact takes 23202 s in all and tddlog 595 s; over the 18 both answer,
1602 s against tddlog's 14.6. tddlog alone answers arith_book,
arith_caesar_huge, arith_caesar_large, arith_disease_small,
arith_fw_large and 31 more.

**Storm dd.** Storm dd ran 62 instances and answered 28 (13 fail, 3 run
out of memory, 18 time out). Its answers differ from the reference's by
at most 4.6e-08 relative (4.9e-09 absolute, insurance) over 28
instances. Over the 54 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, Storm dd takes
16245 s in all and tddlog 595 s; over the 28 both answer, 645 s against
tddlog's 17.4. tddlog alone answers arith_book, arith_caesar_huge,
arith_disease_small, arith_fw_large, arith_gcd_medium and 21 more.

**PRISM sparse.** PRISM sparse ran 62 instances and answered 23 (3 fail,
12 run out of memory, 24 time out). Its answers differ from the
reference's by at most 3.4e-02 relative (6.2e-15 absolute, hmm) over 23
instances. Over the 54 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, PRISM sparse takes
20363 s in all and tddlog 595 s; over the 23 both answer, 1763 s against
tddlog's 3.9. tddlog alone answers arith_book, arith_caesar_huge,
arith_caesar_large, arith_caesar_medium, arith_disease_small and 26
more.

**PRISM hybrid.** PRISM hybrid ran 62 instances and answered 23 (3 fail,
11 run out of memory, 25 time out). Its answers differ from the
reference's by at most 3.4e-02 relative (6.2e-15 absolute, hmm) over 23
instances. Over the 54 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, PRISM hybrid takes
20345 s in all and tddlog 595 s; over the 23 both answer, 1745 s against
tddlog's 3.9. tddlog alone answers arith_book, arith_caesar_huge,
arith_caesar_large, arith_caesar_medium, arith_disease_small and 26
more.

**Roulette.** Roulette ran 51 instances and answered 30 (16 run out of
memory, 5 time out). Its answers differ from the reference's by at most
1.0e+00 relative (1.2e-57 absolute, arith_caesar_huge) over 30
instances. Over the 44 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, Roulette takes 8701
s in all and tddlog 1263 s; over the 30 both answer, 301 s against
tddlog's 325. Roulette is faster on 12 (arith_luhn_medium 0.22 s against
0.24, arith_caesar_large 1.1 s against 1.18, arith_fw_large 0.61 s
against 0.77 and arith_tugofwar 0.16 s against 0.33, the largest
differences). tddlog alone answers arith_disease_small,
arith_gcd_medium, arith_plankton, hybit_addFun_max, hybit_addFun_sum and
9 more.

| instance | flips | gates | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Roulette | Roulette process | ProbLog | ref | tddlog diff | Geni diff | noDice diff | Psi diff | Genfer diff | SPPL diff | Pluck diff | PITA diff | aspmc diff | Storm sparse diff | Storm exact diff | Storm dd diff | PRISM sparse diff | PRISM hybrid diff | Roulette diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| arith_book | 180 | 2453 | 12.8 | >600 | >600 | >600 | oom | >600 |  | 106 | 165 | >600 | err | err | oom | oom | 3.81 | 4.47 | >600 | alea | 1.2e-16 / 1.0e-15 |  |  |  |  |  |  | 8.3e-17 / 6.7e-16 | 2.8e-17 / 2.2e-16 |  |  |  |  |  | 5.6e-17 / 4.5e-16 |
| arith_caesar_huge | 6260 | 38517 | 1.66 | oom | oom | oom | err | >600 |  | >600 | >600 | >600 | >600 | >600 | >600 | >600 | 4.09 | 5.01 | >600 | alea | 3.5e-69 / 3.1e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.2e-57 / 1.0e+00 |
| arith_caesar_large | 2510 | 15417 | 0.726 | >600 | oom | oom | oom | >600 |  | >600 | >600 | >600 | >600 | 154 | >600 | >600 | 1.1 | 1.81 | 91.7 | alea | 1.3e-29 / 6.0e-13 |  |  |  |  |  |  |  |  |  |  | 1.3e-29 / 6.0e-13 |  |  | 1.3e-29 / 5.9e-13 |
| arith_caesar_medium | 1260 | 7717 | 0.759 | >600 | oom | oom | oom | >600 |  | >600 | 191 | 205 | 262 | 42.1 | >600 | >600 | 0.453 | 1.16 | 40.5 | alea | 9.1e-15 / 3.4e-13 |  |  |  |  |  |  |  | 8.9e-15 / 3.4e-13 | 9.1e-15 / 3.4e-13 | 9.1e-15 / 3.4e-13 | 9.1e-15 / 3.4e-13 |  |  | 9.0e-15 / 3.4e-13 |
| arith_caesar_small | 260 | 1557 | 0.08 | >600 | err | >600 | oom | >600 |  | wrong | 6.09 | 1.77 | 3.52 | 5.36 | 11.6 | 11 | 0.079 | 0.7 | 7.15 | alea | 1.2e-15 / 3.3e-14 |  |  |  |  |  |  | 3.1e-02 / 1.3e+03 | 1.8e-15 / 3.2e-14 | 1.2e-15 / 3.1e-14 | 1.4e-15 / 3.1e-14 | 1.4e-15 / 3.1e-14 | 1.0e-15 / 3.0e-14 | 1.0e-15 / 3.0e-14 | 1.4e-15 / 3.1e-14 |
| arith_disease_large | 3044 | 188220 | >600 | oom | oom | oom | err | err |  | >600 | oom | >600 | >600 | >600 | >600 | >600 | oom | oom | oom |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| arith_disease_medium | 1022 | 62049 | >600 | >600 | oom | oom | err | >600 |  | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | oom |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| arith_disease_small | 213 | 11614 | 11.7 | >600 | >600 | oom | err | >600 |  | >600 | 456 | >600 | >600 | err | oom | oom | >600 | >600 | >600 | alea | 1.0e-15 / 6.1e-13 |  |  |  |  |  |  |  | 8.5e-16 / 6.1e-13 |  |  |  |  |  |  |
| arith_fw_large | 44 | 7444 | 0.52 | >600 | >600 | >600 | oom | >600 |  | 489 | >600 | >600 | >600 | err | oom | oom | 0.613 | 1.24 | >600 | alea | 5.6e-17 / 8.9e-15 |  |  |  |  |  |  | 1.2e-16 / 6.1e-09 |  |  |  |  |  |  | 1.1e-16 / 7.7e-15 |
| arith_fw_medium | 17 | 2844 | 0.028 | >600 | >600 | >600 | oom | err |  | 8.5 | 9.44 | 16.4 | 55.7 | 60.8 | 294 | 314 | 0.043 | 0.67 | 15.8 | alea | 5.6e-17 / 2.6e-14 |  |  |  |  |  |  | 1.4e-16 / 5.7e-14 | 5.6e-17 / 8.3e-16 | 8.3e-17 / 4.2e-16 | 5.6e-17 / 4.2e-16 | 5.6e-17 / 4.2e-16 | 5.6e-17 / 4.2e-16 | 5.6e-17 / 4.2e-16 | 5.6e-17 / 4.2e-16 |
| arith_fw_small | 3 | 118 | 0.005 | 5.38 | err | >600 | 0.051 | 2.49 |  | 0.752 | 3.52 | 0.178 | 0.184 | 0.513 | 1.32 | 1.29 | 0.002 | 0.628 | 0.11 | alea | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.8e-16 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| arith_gcd_large | 24 | 52462 | >600 | >600 | oom | oom | err | >600 |  | >600 | >600 | >600 | >600 | >600 | oom | >600 | oom | oom | oom |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| arith_gcd_medium | 20 | 30487 | 159 | >600 | oom | oom | err | >600 |  | >600 | >600 | >600 | >600 | err | >600 | >600 | oom | oom | >600 | alea | 2.2e-16 / 3.6e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| arith_gcd_small | 16 | 15724 | 11.4 | >600 | oom | oom | oom | >600 |  | >600 | 247 | 173 | 188 | 25.9 | >600 | >600 | 9.01 | 10 | >600 | alea | 2.2e-16 / 3.6e-16 |  |  |  |  |  |  |  | 2.2e-16 / 3.6e-16 | 2.2e-16 / 3.6e-16 | 2.2e-16 / 3.6e-16 | 2.2e-16 / 3.6e-16 |  |  | 2.2e-16 / 3.6e-16 |
| arith_linext_large | 24 | 386 | 0.012 | >600 | err | >600 | oom | >600 |  | 15.2 | 223 | >600 | >600 | 44.1 | 257 | 260 | 0.241 | 3.8 | >600 | alea | 4.4e-16 / 2.5e-15 |  |  |  |  |  |  | 4.4e-16 / 2.5e-15 | 5.3e-16 / 3.0e-15 |  |  | 4.4e-16 / 2.5e-15 | 4.4e-16 / 2.5e-15 | 4.4e-16 / 2.5e-15 | 4.4e-16 / 2.5e-15 |
| arith_linext_medium | 24 | 254 | 0.008 | >600 | err | oom | 182 | >600 |  | 1.33 | 4.75 | 9.94 | 12.5 | 14 | 10.1 | 10.1 | 0.046 | 3.63 | 24.3 | alea | 6.1e-16 / 1.7e-15 |  |  |  | 6.7e-16 / 1.7e-15 |  |  | 1.4e-13 / 6.9e-13 | 5.6e-16 / 1.7e-15 | 6.7e-16 / 1.7e-15 | 6.7e-16 / 1.7e-15 | 7.8e-16 / 1.9e-15 | 7.8e-16 / 1.9e-15 | 7.8e-16 / 1.9e-15 | 7.2e-16 / 1.8e-15 |
| arith_linext_small | 8 | 56 | 0.005 | 1.58 | err | 195 | 0.036 | 11.5 |  | 1.06 | 3.26 | 0.78 | 0.828 | 1.85 | 5.29 | 5.28 | 0.008 | 2.43 | 0.151 | alea | 2.2e-16 / 4.4e-16 | 8.0e-07 / 3.2e-06 |  | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 3.9e-16 / 1.1e-15 |  | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 |
| arith_luhn_medium | 36 | 485 | 0.126 | >600 | err | >600 | oom | >600 |  | 7.04 | 4.49 | 277 | >600 | 42.2 | >600 | >600 | 0.223 | 3.1 | >600 | alea | 2.1e-15 / 1.3e-14 |  |  |  |  |  |  | 7.2e-14 / 7.9e-13 | 3.3e-16 / 2.9e-15 | 3.6e-16 / 2.7e-15 |  | 2.8e-16 / 2.6e-15 |  |  | 2.6e-14 / 2.8e-13 |
| arith_luhn_small | 24 | 328 | 0.015 | >600 | err | oom | oom | >600 |  | 1.11 | 3.65 | 2.38 | 4.97 | 9.04 | 11.2 | 10.6 | 0.051 | 2.49 | 2.77 | alea | 1.9e-16 / 1.2e-15 |  |  |  |  |  |  | 1.3e-14 / 6.9e-14 | 1.1e-16 / 1.1e-15 | 1.7e-16 / 1.2e-15 | 1.7e-16 / 1.1e-15 | 1.7e-16 / 1.1e-15 | 1.7e-16 / 1.1e-15 | 1.7e-16 / 1.1e-15 | 1.7e-16 / 1.2e-15 |
| arith_plankton | 60 | 1747 | 0.321 | >600 | >600 | >600 | oom | >600 |  | 193 | 7.33 | >600 | oom | err | oom | oom | >600 | >600 | >600 | alea | 1.1e-16 / 4.3e-14 |  |  |  |  |  |  | 1.3e-16 / 1.0e+00 | 1.1e-16 / 4.3e-14 |  |  |  |  |  |  |
| arith_radar | 66 | 1195 | 0.04 | >600 | err | >600 | oom | >600 |  | 69.4 | 36.8 | oom | err | 22.1 | >600 | >600 | 0.052 | 0.68 | 32.6 | alea | 2.8e-16 / 1.5e-15 |  |  |  |  |  |  | 6.7e-12 / 2.9e-10 | 3.1e-16 / 1.7e-15 |  |  | 2.5e-16 / 1.5e-15 |  |  | 2.5e-16 / 1.4e-15 |
| arith_ranking_large | 88 | 1212 | 0.072 | >600 | >600 | >600 | oom | >600 |  | 18.6 | 9.06 | 15.7 | 22.5 | 22.2 | 50.2 | 54.1 | 0.408 | 2.79 | 236 | alea | 2.0e-15 / 2.8e-14 |  |  |  |  |  |  | 3.8e-08 / 2.5e-04 | 1.7e-15 / 2.8e-14 | 1.8e-15 / 2.8e-14 | 1.8e-15 / 2.8e-14 | 1.8e-15 / 2.8e-14 | 1.8e-15 / 2.8e-14 | 1.8e-15 / 2.8e-14 | 1.4e-08 / 5.3e-06 |
| arith_ranking_medium | 56 | 744 | 0.021 | >600 | >600 | >600 | oom | >600 |  | 6.11 | 5.24 | 7.29 | 11.2 | 17.1 | 37.7 | 36.8 | 0.195 | 2.45 | 147 | alea | 5.6e-16 / 1.2e-14 |  |  |  |  |  |  | 8.5e-11 / 2.0e-08 | 6.1e-16 / 1.2e-14 | 6.1e-16 / 1.2e-14 | 6.1e-16 / 1.2e-14 | 5.6e-16 / 1.2e-14 | 5.6e-16 / 1.2e-14 | 5.6e-16 / 1.2e-14 | 5.3e-11 / 3.2e-09 |
| arith_ranking_small | 24 | 276 | 0.014 | >600 | err | oom | oom | >600 |  | 0.918 | 3.35 | 2.11 | 3.22 | 11.7 | 11.9 | 11.7 | 0.048 | 2.49 | 33.9 | alea | 3.3e-16 / 1.1e-15 |  |  |  |  |  |  | 6.7e-14 / 2.3e-13 | 3.3e-16 / 9.4e-16 | 3.3e-16 / 9.4e-16 | 3.9e-16 / 1.1e-15 | 4.4e-16 / 1.1e-15 | 4.4e-16 / 1.1e-15 | 4.4e-16 / 1.1e-15 | 1.6e-14 / 6.3e-14 |
| arith_triangle_large | 132 | 1916 | 25.9 | >600 | oom | >600 | oom | >600 |  | 155 | >600 | >600 | err | err | oom | oom | 20.8 | 21.8 | >600 | alea | 5.0e-15 / 7.9e-14 |  |  |  |  |  |  | 2.2e-16 / 1.7e-11 |  |  |  |  |  |  | 2.2e-16 / 7.0e-15 |
| arith_triangle_medium | 57 | 1298 | 0.575 | >600 | oom | >600 | oom | >600 |  | 5.45 | 158 | >600 | err | err | >600 | >600 | 0.767 | 1.4 | >600 | alea | 1.8e-15 / 2.9e-14 |  |  |  |  |  |  | 5.7e-17 / 1.9e-13 | 1.1e-16 / 1.4e-15 |  |  |  |  |  | 3.3e-16 / 6.1e-14 |
| arith_triangle_small | 48 | 1277 | 0.244 | >600 | oom | >600 | oom | >600 |  | 3.41 | 29.1 | 295 | err | 41 | >600 | >600 | 0.189 | 0.803 | >600 | alea | 2.8e-16 / 5.0e-15 |  |  |  |  |  |  | 3.1e-17 / 3.0e-13 | 6.9e-18 / 1.3e-15 | 4.4e-16 / 9.8e-16 |  | 5.4e-20 / 1.1e-15 |  |  | 8.0e-17 / 1.5e-14 |
| arith_tugofwar | 44 | 1372 | 0.201 | >600 | >600 | >600 | oom | >600 |  | 1.94 | 194 | >600 | err | err | >600 | >600 | 0.156 | 0.765 | >600 | alea | 3.3e-16 / 3.4e-16 |  |  |  |  |  |  | 2.2e-16 / 2.3e-16 | 4.4e-16 / 4.6e-16 |  |  |  |  |  | 3.3e-16 / 3.4e-16 |
| hybit_GPA | 94168 | 516070 | 23.6 | oom | oom | >600 | >600 | err |  | >600 | >600 | err | err | err | err | err | 33.2 | 37.8 | oom | alea | 1.1e-16 / 1.8e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.1e-16 / 1.8e-16 |
| hybit_addFun_max | 604 | 26275 | 37.5 | >600 | oom | oom | oom | >600 |  | >600 | >600 | >600 | >600 | >600 | >600 | >600 | oom | oom | >600 | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_addFun_sum | 196 | 3820 | 0.705 | >600 | >600 | >600 | oom | >600 |  | 142 | >600 | >600 | err | >600 | oom | oom | oom | oom | >600 | tddlog | ref |  |  |  |  |  |  | 4.4e-16 / 8.9e-16 |  |  |  |  |  |  |  |
| hybit_altermu2 | 80350 | 457880 | oom | oom | oom | >600 | >600 | err |  | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_clickGraph | 208 | 765 | 0.317 | >600 | >600 | >600 | oom | >600 |  | 510 | 4.13 | >600 | >600 | >600 | 282 | 297 | oom | oom | 5.53 | problog | 4.7e-09 / 9.2e-09 |  |  |  |  |  |  | 4.7e-09 / 9.2e-09 | 4.7e-09 / 9.2e-09 |  |  |  | 4.7e-09 / 9.2e-09 | 4.7e-09 / 9.2e-09 |  |
| hybit_clinicalTrial1 | 2419 | 12800 | 1.66 | >600 | >600 | oom | oom | >600 |  | >600 | 419 | >600 | >600 | >600 | err | err | oom | oom | 21.5 | problog | 6.5e-10 / 3.8e-09 |  |  |  |  |  |  |  | 6.5e-10 / 3.8e-09 |  |  |  |  |  |  |
| hybit_clinicalTrial2 | 221 | 1150 | 0.021 | >600 | oom | err | oom | >600 |  | 175 | 9.06 | >600 | err | err | err | err | oom | oom | 22.1 | problog | 4.9e-09 / 2.3e-08 |  |  |  |  |  |  | 4.9e-09 / 2.3e-08 | 4.9e-09 / 2.3e-08 |  |  |  |  |  |  |
| hybit_coinBias | 37026 | 226532 | >600 | oom | oom | oom | >600 | err |  | >600 | >600 | oom | oom | oom | oom | oom | oom | oom | oom |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_conjugate_gaussians | 249 | 4198 | 0.879 | >600 | oom | >600 | oom | >600 |  | >600 | >600 | >600 | oom | err | oom | oom | oom | oom | >600 | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_normal_mixture_mu1 | 5289 | 35812 | 135 | >600 | oom | oom | err | >600 |  | >600 | >600 | >600 | >600 | >600 | >600 | >600 | oom | oom | >600 | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_normal_mixture_mu2 | 7913 | 63750 | 162 | oom | oom | oom | err | >600 |  | >600 | oom | >600 | >600 | >600 | >600 | >600 | oom | oom | >600 | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_normal_mixture_theta | 10537 | 118560 | 30 | oom | oom | oom | >600 | >600 |  | >600 | >600 | >600 | >600 | >600 | >600 | >600 | oom | oom | >600 | exact | 4.4e-16 / 5.2e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_pi | 24 | 12633 | 1.49 | >600 | oom | oom | oom | >600 |  | 33.5 | >600 | >600 | >600 | err | >600 | >600 | 11.9 | 12.7 | >600 | alea | 5.6e-17 / 1.2e-16 |  |  |  |  |  |  | 5.6e-17 / 1.2e-16 |  |  |  |  |  |  | 5.6e-17 / 1.2e-16 |
| hybit_spacex | 513 | 3195 | oom | >600 | >600 | oom | oom | >600 |  | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | alea |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_trueskill | 376 | 5592 | 181 | >600 | >600 | >600 | oom | >600 |  | >600 | >600 | >600 | oom | >600 | >600 | >600 | oom | oom | >600 | alea | 3.3e-16 / 6.7e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_tug_of_war | 550 | 2720 | 0.54 | >600 | >600 | oom | oom | >600 |  | >600 | >600 | >600 | >600 | 23.5 | 434 | 434 | oom | oom | >600 | exact | 5.0e-16 / 1.0e-15 |  |  |  |  |  |  |  |  |  |  | 5.6e-16 / 1.1e-15 | 4.4e-16 / 8.9e-16 | 4.4e-16 / 8.9e-16 |  |
| hybit_weekend | 98303 | 516022 | 20.5 | oom | oom | >600 | >600 | err |  | >600 | >600 | oom | oom | oom | oom | oom | 30.9 | 35.4 | oom | alea | 8.3e-16 / 2.2e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.0e-10 / 2.8e-10 |
| hybit_zeroone_w1 | 44 | 8780 | 0.318 | >600 | oom | oom | oom | >600 |  | 17.1 | 346 | >600 | >600 | >600 | >600 | >600 | 11.6 | 19.5 | >600 | alea | 3.9e-15 / 7.7e-15 |  |  |  |  |  |  | 3.2e-13 / 6.5e-13 | 1.0e-15 / 2.0e-15 |  |  |  |  |  | 1.2e-13 / 2.3e-13 |
| hybit_zeroone_w2 | 50 | 13588 | 1.25 | >600 | oom | oom | err | >600 |  | 150 | >600 | >600 | >600 | >600 | >600 | >600 | 140 | 155 | >600 | alea | 1.1e-13 / 1.6e-13 |  |  |  |  |  |  | 5.8e-13 / 1.7e-12 |  |  |  |  |  |  | 1.3e-13 / 2.5e-13 |
| nodice_3sat-0 |  |  | 0.452 | >600 | zero | oom | oom | >600 |  | 16.6 | 2.74 | oom | err | 14.4 | >600 | >600 |  |  | >600 | alea | 1.3e-22 / 1.1e-15 |  |  |  |  |  |  | 1.3e-22 / 1.1e-15 | 1.3e-22 / 1.1e-15 |  |  | 1.3e-22 / 1.1e-15 |  |  |  |
| nodice_alarm |  |  | 0.049 | 0.769 | 381 | oom | 1.48 | 6.04 |  | 29.2 | 8.55 | 0.206 | 0.23 | 2.35 | 1.27 | 1.27 |  |  | 1.95 | problog | 2.5e-09 / 1.2e-08 | 4.9e-08 / 1.9e-07 | 2.4e-07 / 1.2e-06 |  | 2.5e-09 / 1.2e-08 | 2.5e-09 / 1.2e-08 |  | 2.5e-09 / 1.2e-08 | 2.5e-09 / 1.2e-08 | 2.5e-09 / 1.2e-08 | 2.5e-09 / 1.2e-08 | 2.5e-09 / 1.2e-08 | 2.5e-09 / 1.2e-08 | 2.5e-09 / 1.2e-08 |  |
| nodice_coupon2-10-obs |  |  | 0.075 | oom | 4.49 | err | oom | >600 |  | 4.55 | >600 | 84.7 | 87.5 | 7.87 | 3.37 | 2.87 |  |  | >600 | alea | 1.7e-16 / 6.3e-16 |  | 5.2e-07 / 2.0e-06 |  |  |  |  | 1.5e-15 / 5.7e-15 |  | 5.6e-16 / 2.1e-15 | 2.2e-16 / 8.5e-16 | 2.2e-16 / 8.5e-16 | 5.6e-16 / 2.1e-15 | 5.6e-16 / 2.1e-15 |  |
| nodice_coupon2-11-obs |  |  | 0.138 | >600 | 13 | err | oom | >600 |  | 13.1 | >600 | 328 | 346 | 10.5 | 5.82 | 4.33 |  |  | >600 | alea | 5.6e-15 / 2.6e-14 |  | 7.0e-07 / 3.2e-06 |  |  |  |  | 1.1e-16 / 5.1e-16 |  | 5.6e-17 / 2.5e-16 | 2.2e-16 / 1.0e-15 | 1.9e-16 / 8.8e-16 | 1.1e-16 / 5.1e-16 | 1.1e-16 / 5.1e-16 |  |
| nodice_coupon2-12-obs |  |  | 0.187 | >600 | 35.7 | err | oom | >600 |  | 63.2 | >600 | >600 | >600 | 12.6 | 11.1 | 5.9 |  |  | >600 | alea | 7.2e-16 / 3.9e-15 |  | 1.4e-07 / 7.6e-07 |  |  |  |  | 1.7e-15 / 9.1e-15 |  |  |  | 5.8e-16 / 3.2e-15 | 3.9e-16 / 2.1e-15 | 3.9e-16 / 2.1e-15 |  |
| nodice_coupon2-13-obs |  |  | 0.415 | >600 | 96.4 | err | oom | >600 |  | 211 | >600 | >600 | >600 | 16.4 | 28 | 12.2 |  |  | >600 | alea | 5.3e-16 / 3.4e-15 |  | 2.8e-07 / 1.8e-06 |  |  |  |  | 2.4e-15 / 1.6e-14 |  |  |  | 5.0e-16 / 3.3e-15 | 5.3e-16 / 3.4e-15 | 5.3e-16 / 3.4e-15 |  |
| nodice_coupon2-14-obs |  |  | 0.81 | >600 | 252 | err | oom | >600 |  | >600 | >600 | >600 | >600 | 20.3 | 77.4 | 31 |  |  | >600 | alea | 2.2e-16 / 1.7e-15 |  | 2.7e-07 / 2.1e-06 |  |  |  |  |  |  |  |  | 2.2e-16 / 1.7e-15 | 1.9e-16 / 1.5e-15 | 1.9e-16 / 1.5e-15 |  |
| nodice_hepar2 |  |  | 0.06 | 0.146 | 0.618 | oom | 0.266 | 5.28 |  | 3.54 | 181 | 0.241 | 0.248 | 2.4 | 1.26 | 1.28 |  |  | 0.751 | problog | 1.1e-09 / 6.9e-09 | 3.8e-09 / 2.3e-08 | 6.8e-07 / 4.2e-06 |  | 1.1e-09 / 6.9e-09 | 1.1e-09 / 6.9e-09 |  | 1.1e-09 / 6.9e-09 | 1.1e-09 / 6.9e-09 | 1.1e-09 / 6.9e-09 | 1.1e-09 / 6.9e-09 | 1.1e-09 / 6.9e-09 | 1.1e-09 / 6.9e-09 | 1.1e-09 / 6.9e-09 |  |
| nodice_insurance |  |  | 0.093 | 4.36 | 3.67 | oom | 4.07 | >600 |  | >600 | 30.2 | 5.29 | 5.44 | 17.7 | 4.88 | 6.67 |  |  | 48.6 | problog | 4.9e-09 / 4.6e-08 | 1.0e-07 / 1.3e-06 | 7.0e-07 / 2.9e-05 |  | 4.9e-09 / 4.6e-08 |  |  |  | 4.9e-09 / 4.6e-08 | 4.9e-09 / 4.6e-08 | 4.9e-09 / 4.6e-08 | 4.9e-09 / 4.6e-08 | 4.9e-09 / 4.6e-08 | 4.9e-09 / 4.6e-08 |  |
| nodice_survey |  |  | 0.004 | 0.014 | 0.012 | 1.64 | 0.032 | 1.4 |  | 0.495 | 2.7 | 0.153 | 0.152 | 0.254 | 1.27 | 1.27 |  |  | 0.083 | problog | 4.0e-09 / 1.3e-08 | 3.6e-08 / 9.1e-08 | 7.5e-07 / 2.7e-06 | 4.0e-09 / 1.3e-08 | 4.0e-09 / 1.3e-08 | 4.0e-09 / 1.3e-08 |  | 4.0e-09 / 1.3e-08 | 4.0e-09 / 1.3e-08 | 4.0e-09 / 1.3e-08 | 4.0e-09 / 1.3e-08 | 4.0e-09 / 1.3e-08 | 4.0e-09 / 1.3e-08 | 4.0e-09 / 1.3e-08 |  |
| nodice_water |  |  | 0.777 | 38.7 | 18.9 | oom | 50.3 | 148 |  | 16 | 62.3 | >600 | 598 | >600 | 219 | 229 |  |  | 8.16 | problog | 8.3e-10 / 5.9e-09 | 5.2e-08 / 2.7e-06 | 7.4e-07 / 7.5e-05 |  | 8.3e-10 / 5.9e-09 | 8.3e-10 / 5.9e-09 |  | 8.3e-10 / 5.9e-09 | 8.3e-10 / 5.9e-09 |  | 8.3e-10 / 5.9e-09 |  | 8.3e-10 / 5.9e-09 | 8.3e-10 / 5.9e-09 |  |
| pluck_hmm | 199 | 346 | 0.017 | >600 | oom | oom | oom | oom | 5.03 | 3.2 | 3.02 | >600 | err | 3.27 | 2.98 | 2.92 | 0.021 | 0.447 | >600 | alea | 5.6e-27 / 3.1e-14 |  |  |  |  |  | 5.7e-27 / 3.2e-14 | 1.2e-16 / 6.9e-04 | 6.3e-27 / 3.5e-14 |  |  | 5.6e-27 / 3.1e-14 | 6.2e-15 / 3.4e-02 | 6.2e-15 / 3.4e-02 | 6.7e-19 / 3.7e-06 |
| pluck_pcfg | 12031 | 532827 | 101 | oom | oom | oom | >600 | err | 5.18 | >600 | >600 | oom | oom | oom | oom | oom | 15 | 17.6 | >600 | alea | 9.3e-23 / 1.6e-15 |  |  |  |  |  | 3.0e-08 / 5.0e-01 |  |  |  |  |  |  |  | 9.3e-23 / 1.6e-15 |
| pluck_sorted_list | 57 | 4056 | 7.72 | >600 | oom | >600 | oom | >600 | err | 2.18 | 5.07 | oom | oom | >600 | >600 | >600 | 16.5 | 17.6 | >600 | alea | 1.7e-25 / 2.3e-14 |  |  |  |  |  |  | 1.7e-25 / 2.3e-14 | 1.7e-25 / 2.3e-14 |  |  |  |  |  | 1.7e-25 / 2.3e-14 |
| pluck_string_editing | 130 | 1343 | 0.058 | >600 | zero | >600 | oom | >600 | 5.95 | 1.1 | 5.2 | oom | oom | err | oom | oom | oom | oom | >600 | alea | 1.7e-26 / 7.4e-15 |  |  |  |  |  | 3.0e-08 / 1.3e+04 | 9.1e-17 / 4.0e-05 | 1.7e-26 / 7.4e-15 |  |  |  |  |  |  |

Seconds per system, the median over the repetitions; a failure is `>600`
(timeout), `oom`, `err` or `n/a`; each system's last columns are its
largest absolute and relative difference from the reference named in
`ref`.

### On the bit-blasted programs

| family | instances | tddlog | Geni | noDice | Psi | Genfer | SPPL | PITA | aspmc | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Merlin | pyAgrum | pgmpy | libDAI | TensorInference.jl greedy | TensorInference.jl treesa | Roulette |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| bb_GPA | 7 | 7 |  |  |  |  |  | 1 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 7 of 7 |
| bb_addFun_max_p | 3 | 3 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 2 of 3 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 0 of 1 |
| bb_addFun_max | 3 | 3 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 2 of 3 |
| bb_addFun_sum | 5 | 5 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 4 of 5 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 3 of 4 |
| bb_altermu2_p | 2 | 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 1 of 2 | 1 of 2 | 1 of 2 | 1 of 2 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 |
| bb_altermu2 | 2 | 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 1 of 2 |
| bb_clickGraph | 7 | 7 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 1 of 2 | 6 of 7 | 7 of 7 | 3 of 4 | 3 of 4 | 3 of 4 | 4 of 5 | 4 of 5 |  |  |  |  |  |  |  |  | 3 of 4 |
| bb_clinicalTrial1 | 4 | 4 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 2 of 4 | 0 of 1 | 0 of 1 | 2 of 3 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 2 of 3 |
| bb_clinicalTrial2 | 7 | 7 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 | 7 of 7 | 7 of 7 | 2 of 3 | 2 of 3 | 2 of 3 | 3 of 4 | 3 of 4 |  |  |  |  |  |  |  |  | 3 of 4 |
| bb_coinBias | 2 | 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 1 of 2 |
| bb_conjugate_ex3 | 2 | 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 |
| bb_conjugate_gaussians | 2 | 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 1 of 2 |
| bb_fair_dt14 | 4 | 4 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 2 of 3 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 3 of 4 |
| bb_fair_dt16 | 6 | 6 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 6 of 6 | 6 of 6 | 0 of 1 | 0 of 1 | 2 of 3 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 6 of 6 |
| bb_fair_dt44 | 6 | 6 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 6 of 6 | 6 of 6 | 0 of 1 | 0 of 1 | 2 of 3 | 1 of 2 | 1 of 2 |  |  |  |  |  |  |  |  | 6 of 6 |
| bb_fair_dt4 | 4 | 4 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 3 of 4 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 3 of 4 |
| bb_genes5 | 5 | 5 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 4 of 5 |
| bb_genes | 1 | 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |
| bb_laplace | 2 | 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 1 of 2 |
| bb_linreg | 8 | 6 | 0 of 1 | 0 of 1 | 2 of 8 | 0 of 1 | 0 of 1 | 0 of 2 | 3 of 4 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 4 of 5 |
| bb_markov | 5 | 5 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 5 | 0 of 1 | 2 of 3 | 0 of 1 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 2 of 3 |
| bb_multimodal | 1 | 0 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |
| bb_normal_mixture_mu1 | 2 | 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 1 of 2 |
| bb_normal_mixture_mu2 | 2 | 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 1 of 2 |
| bb_normal_mixture_theta | 1 | 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 0 of 1 |
| bb_pedestrian | 3 | 3 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 2 of 3 |
| bb_pi | 5 | 5 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 4 of 5 | 4 of 5 | 2 of 3 | 2 of 3 | 2 of 3 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 4 of 5 |
| bb_population | 8 | 8 | 0 of 1 | 0 of 1 | 0 of 1 | 8 of 8 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 0 of 1 |
| bb_spacex_p | 3 | 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 1 of 2 | 0 of 1 | 0 of 1 | 1 of 2 | 1 of 2 | 1 of 2 | 1 of 2 | 1 of 2 | 1 of 2 | 1 of 2 | 1 of 2 | 1 of 2 | 2 of 3 | 0 of 1 |
| bb_spacex | 4 | 3 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 3 of 4 |
| bb_trueskill | 2 | 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 1 of 2 |
| bb_tug_of_war | 7 | 7 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 1 of 2 | 0 of 1 | 0 of 1 | 7 of 7 | 7 of 7 | 7 of 7 |  |  |  |  |  |  |  |  | 2 of 3 |
| bb_weekend_p | 5 | 5 | 0 of 1 | 3 of 4 | 0 of 1 | 0 of 1 | 0 of 1 | 5 of 5 | 2 of 3 | 0 of 1 | 0 of 1 | 3 of 4 | 1 of 2 | 1 of 2 | 1 of 2 | 2 of 3 | 2 of 3 | 2 of 3 | 2 of 3 | 2 of 3 | 1 of 2 | 2 of 3 | 5 of 5 |
| bb_weekend | 7 | 7 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 7 of 7 |
| bb_wmi_path | 5 | 5 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 3 of 4 | 2 of 3 | 2 of 3 |  |  |  |  |  |  |  |  | 4 of 5 |
| bb_wmi_star | 5 | 5 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 4 of 5 | 0 of 1 | 0 of 1 | 3 of 4 | 2 of 3 | 2 of 3 |  |  |  |  |  |  |  |  | 4 of 5 |
| bb_zeroone_w1 | 7 | 7 | 0 of 1 | 2 of 3 | 0 of 1 | 0 of 1 | 0 of 1 | 7 of 7 | 4 of 5 | 1 of 2 | 1 of 2 | 2 of 3 | 1 of 2 | 1 of 2 |  |  |  |  |  |  |  |  | 6 of 7 |
| bb_zeroone_w2 | 7 | 7 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 6 of 7 | 2 of 3 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  | 4 of 5 |

**Geni.** Geni ran 37 instances and answered 0 (9 run out of memory, 28
time out). Over the 36 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, Geni takes 21600 s
in all and tddlog 201 s. tddlog alone answers addFun_max_p4,
addFun_max_w17, addFun_sum_w17, altermu2_p4, altermu2_w11 and 31 more.

**noDice.** noDice ran 44 instances and answered 7 (27 run out of
memory, 10 time out). Its answers differ from the reference's by at most
1.5e-06 relative (7.4e-07 absolute, clinicalTrial2_w6) over 7 instances.
Over the 43 instances given to both that either answers, a run that does
not answer counted at the 600 s limit, noDice takes 22142 s in all and
tddlog 207 s; over the 7 both answer, 542 s against tddlog's 2.73.
tddlog alone answers addFun_max_p4, addFun_max_w17, addFun_sum_w17,
altermu2_p4, altermu2_w11 and 31 more.

**Psi.** Psi ran 44 instances and answered 2 (7 fail, 26 run out of
memory, 9 time out). Over the 42 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, Psi
takes 24600 s in all and tddlog 1046 s. Only Psi answers linreg_w14.
tddlog alone answers addFun_max_p4, addFun_max_w17, addFun_sum_w17,
altermu2_p4, altermu2_w11 and 36 more.

**Genfer.** Genfer ran 44 instances and answered 8 (2 fail, 27 run out
of memory, 7 time out). Its answers differ from the reference's by at
most 9.7e-11 relative (8.0e-12 absolute, population_s3) over 8
instances. Over the 43 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, Genfer takes 21000
s in all and tddlog 324 s; over the 8 both answer, 0.2 s against
tddlog's 123. Genfer is faster on 8 (population_s1 0.007 s against 0.35,
population_s2 0.009 s against 8.29, population_s3 0.011 s against 9.33
and population_s4 0.013 s against 10.2, the largest differences). tddlog
alone answers addFun_max_p4, addFun_max_w17, addFun_sum_w17,
altermu2_p4, altermu2_w11 and 30 more.

**SPPL.** SPPL ran 42 instances and answered 1 (6 fail, 1 run out of
memory, 30 time out, 4 answer wrongly). Its answers differ from the
reference's by at most 5.5e-16 relative (3.3e-16 absolute,
clickGraph_w4) over 1 instance. SPPL's answer is wrong, zero or
underflowed on markov_n10, markov_n100, markov_n20 and markov_n50. Over
the 41 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, SPPL takes 24217 s in all and tddlog
217 s; over the 1 both answer, 217 s against tddlog's 0.24. tddlog alone
answers addFun_max_p4, addFun_max_w17, addFun_sum_w17, altermu2_p4,
altermu2_w11 and 35 more.

**PITA.** PITA ran 95 instances and answered 62 (2 fail, 30 time out, 1
answer wrongly). Its answers differ from the reference's by at most
1.0e+00 relative (1.4e-14 absolute, pedestrian_s4) over 61 instances.
PITA's answer is wrong, zero or underflowed on linreg_w8. Over the 94
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, PITA takes 25661 s in all and tddlog 422 s;
over the 62 both answer, 6461 s against tddlog's 132. tddlog alone
answers addFun_max_p64, addFun_max_w19, addFun_sum_w25, altermu2_p4,
altermu2_w11 and 27 more.

**aspmc.** aspmc ran 93 instances and answered 58 (1 fail, 33 time out,
1 print zeros). Its answers differ from the reference's by at most
2.9e-14 relative (5.9e-15 absolute, clinicalTrial1_w4) over 58
instances. aspmc's answer is wrong, zero or underflowed on
clinicalTrial1_w6. Over the 91 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, aspmc
takes 25995 s in all and tddlog 481 s; over the 58 both answer, 6195 s
against tddlog's 44.6. tddlog alone answers GPA_w19, addFun_max_p16,
addFun_max_w17, addFun_sum_w17, altermu2_p4 and 28 more.

**Storm sparse.** Storm sparse ran 47 instances and answered 9 (2 run
out of memory, 36 time out). Its answers differ from the reference's by
at most 2.4e-14 relative (4.1e-15 absolute, spacex_p4) over 9 instances.
Over the 46 instances given to both that either answers, a run that does
not answer counted at the 600 s limit, Storm sparse takes 22991 s in all
and tddlog 304 s; over the 9 both answer, 791 s against tddlog's 0.56.
tddlog alone answers GPA_w19, addFun_max_p4, addFun_max_w17,
addFun_sum_w17, altermu2_p4 and 32 more.

**Storm exact.** Storm exact ran 46 instances and answered 8 (7 run out
of memory, 31 time out). Its answers differ from the reference's by at
most 1.5e-15 relative (7.8e-16 absolute, zeroone_w1_w10) over 8
instances. Over the 45 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, Storm exact takes
22784 s in all and tddlog 215 s; over the 8 both answer, 584 s against
tddlog's 0.52. tddlog alone answers GPA_w19, addFun_max_p4,
addFun_max_w17, addFun_sum_w17, altermu2_p4 and 32 more.

**Storm dd.** Storm dd ran 69 instances and answered 32 (8 fail, 2 run
out of memory, 27 time out). Its answers differ from the reference's by
at most 6.4e-14 relative (1.1e-14 absolute, clinicalTrial1_w6) over 32
instances. Over the 68 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, Storm dd takes
24330 s in all and tddlog 229 s; over the 32 both answer, 2730 s against
tddlog's 8.84. tddlog alone answers GPA_w19, addFun_max_p4,
addFun_max_w17, addFun_sum_w17, altermu2_p4 and 31 more.

**PRISM sparse.** PRISM sparse ran 59 instances and answered 22 (2 fail,
4 run out of memory, 31 time out). Its answers differ from the
reference's by at most 6.3e-14 relative (3.3e-14 absolute, wmi_star_n32)
over 22 instances. Over the 58 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, PRISM
sparse takes 26464 s in all and tddlog 308 s; over the 22 both answer,
4864 s against tddlog's 5.29. tddlog alone answers GPA_w19,
addFun_max_p4, addFun_max_w17, addFun_sum_w17, altermu2_p4 and 31 more.

**PRISM hybrid.** PRISM hybrid ran 59 instances and answered 22 (2 fail,
4 run out of memory, 31 time out). Its answers differ from the
reference's by at most 6.3e-14 relative (3.3e-14 absolute, wmi_star_n32)
over 22 instances. Over the 58 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, PRISM
hybrid takes 26482 s in all and tddlog 308 s; over the 22 both answer,
4882 s against tddlog's 5.29. tddlog alone answers GPA_w19,
addFun_max_p4, addFun_max_w17, addFun_sum_w17, altermu2_p4 and 31 more.

**Ace.** Ace ran 11 instances and answered 3 (2 fail, 1 run out of
memory, 5 time out). Its answers differ from the reference's by at most
1.7e-14 relative (2.2e-16 absolute, spacex_p4) over 3 instances. Over
the 10 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Ace takes 4219 s in all and tddlog
201 s; over the 3 both answer, 19.1 s against tddlog's 1.55. tddlog
alone answers addFun_max_p4, altermu2_p16, conjugate_ex3_w18,
genes5_w17, genes_t5 and 2 more.

**Ace c2d.** Ace c2d ran 12 instances and answered 4 (2 fail, 6 time
out). Its answers differ from the reference's by at most 1.7e-14
relative (2.8e-16 absolute, spacex_p4) over 4 instances. Over the 11
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Ace c2d takes 4428 s in all and tddlog 202
s; over the 4 both answer, 228 s against tddlog's 1.67. tddlog alone
answers addFun_max_p4, altermu2_p16, conjugate_ex3_w18, genes5_w17,
genes_t5 and 2 more.

**Merlin.** Merlin ran 12 instances and answered 4 (2 fail, 5 run out of
memory, 1 time out). Its answers differ from the reference's by at most
1.0e+00 relative (4.8e-07 absolute, spacex_p4) over 4 instances. Over
the 11 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Merlin takes 4278 s in all and tddlog
202 s; over the 4 both answer, 78.3 s against tddlog's 1.67. Merlin is
faster on 1 (spacex_p4 0.29 s against 0.45). tddlog alone answers
addFun_max_p4, altermu2_p16, conjugate_ex3_w18, genes5_w17, genes_t5 and
2 more.

**pyAgrum.** pyAgrum ran 12 instances and answered 4 (2 fail, 4 run out
of memory, 2 time out). Its answers differ from the reference's by at
most 1.0e-12 relative (3.8e-13 absolute, weekend_p4) over 4 instances.
Over the 11 instances given to both that either answers, a run that does
not answer counted at the 600 s limit, pyAgrum takes 4243 s in all and
tddlog 202 s; over the 4 both answer, 43.5 s against tddlog's 1.67.
pyAgrum is faster on 1 (spacex_p4 0.41 s against 0.45). tddlog alone
answers addFun_max_p4, altermu2_p16, conjugate_ex3_w18, genes5_w17,
genes_t5 and 2 more.

**pgmpy.** pgmpy ran 12 instances and answered 4 (2 fail, 6 time out).
Its answers differ from the reference's by at most 1.0e-12 relative
(3.8e-13 absolute, weekend_p4) over 4 instances. Over the 11 instances
given to both that either answers, a run that does not answer counted at
the 600 s limit, pgmpy takes 4622 s in all and tddlog 202 s; over the 4
both answer, 422 s against tddlog's 1.67. tddlog alone answers
addFun_max_p4, altermu2_p16, conjugate_ex3_w18, genes5_w17, genes_t5 and
2 more.

**libDAI.** libDAI ran 12 instances and answered 4 (2 fail, 3 run out of
memory, 3 time out). Its answers differ from the reference's by at most
1.0e-12 relative (3.8e-13 absolute, weekend_p4) over 4 instances. Over
the 11 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, libDAI takes 4324 s in all and tddlog
202 s; over the 4 both answer, 124 s against tddlog's 1.67. libDAI is
faster on 1 (spacex_p4 0.34 s against 0.45). tddlog alone answers
addFun_max_p4, altermu2_p16, conjugate_ex3_w18, genes5_w17, genes_t5 and
2 more.

**TensorInference.jl greedy.** TensorInference.jl greedy ran 10
instances and answered 2 (2 fail, 5 run out of memory, 1 time out). Its
answers differ from the reference's by at most 1.7e-14 relative (3.3e-16
absolute, spacex_p4) over 2 instances. Over the 9 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, TensorInference.jl greedy takes 4326 s in all and tddlog 199 s;
over the 2 both answer, 126 s against tddlog's 0.47. tddlog alone
answers addFun_max_p4, altermu2_p4, conjugate_ex3_w18, genes5_w17,
genes_t5 and 2 more.

**TensorInference.jl treesa.** TensorInference.jl treesa ran 13
instances and answered 5 (2 fail, 6 time out). Its answers differ from
the reference's by at most 1.7e-14 relative (2.2e-16 absolute,
spacex_p4) over 5 instances. Over the 11 instances given to both that
either answers, a run that does not answer counted at the 600 s limit,
TensorInference.jl treesa takes 4719 s in all and tddlog 201 s; over the
5 both answer, 1119 s against tddlog's 133. tddlog alone answers
addFun_max_p16, altermu2_p4, conjugate_ex3_w18, genes5_w17, genes_t5 and
1 more.

**Roulette.** Roulette ran 130 instances and answered 97 (22 run out of
memory, 11 time out). Its answers differ from the reference's by at most
1.0e+00 relative (3.2e-15 absolute, pedestrian_s4) over 97 instances.
Over the 125 instances given to both that either answers, a run that
does not answer counted at the 600 s limit, Roulette takes 21362 s in
all and tddlog 2385 s; over the 96 both answer, 4419 s against tddlog's
1025. Roulette is faster on 30 (fair_dt16_w12 0.088 s against 0.22,
weekend_p16 0.06 s against 0.23, fair_dt44_w20 0.17 s against 0.36 and
fair_dt16_w16 0.15 s against 0.38, the largest differences). Only
Roulette answers spacex_w17. tddlog alone answers addFun_max_p4,
addFun_max_w21, addFun_sum_w23, altermu2_p4, altermu2_w13 and 23 more.

| instance | flips | gates | tddlog | Geni | noDice | Psi | Genfer | SPPL | PITA | aspmc | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Merlin | pyAgrum | pgmpy | libDAI | TI greedy | TI treesa | Roulette | Roulette process | ProbLog | ref | tddlog diff | Geni diff | noDice diff | Psi diff | Genfer diff | SPPL diff | PITA diff | aspmc diff | Storm sparse diff | Storm exact diff | Storm dd diff | PRISM sparse diff | PRISM hybrid diff | Ace diff | Ace c2d diff | Merlin diff | pyAgrum diff | pgmpy diff | libDAI diff | TI greedy diff | TI treesa diff | Roulette diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GPA_w19 | 45061 | 368680 | 17.5 |  |  |  |  |  | 0.378 | err | oom | oom | oom | oom | oom |  |  |  |  |  |  |  |  | 34.9 | 39 | 0.064 | alea | 2.2e-16 / 3.6e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 7.8e-16 / 1.3e-15 |
| GPA_w21 | 61441 | 417832 | 17.9 |  |  |  |  |  |  |  | - | - | - | - | - |  |  |  |  |  |  |  |  | 28.8 | 33 | - | alea | 2.2e-16 / 3.6e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 7.8e-16 / 1.3e-15 |
| GPA_w23 | 77812 | 466966 | 20 |  |  |  |  |  |  |  | - | - | - | - | - |  |  |  |  |  |  |  |  | 33.9 | 38.2 | - | alea | 1.1e-16 / 1.8e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.1e-16 / 1.8e-16 |
| GPA_w25 | 94168 | 516070 | 19.7 |  |  |  |  |  |  |  | - | - | - | - | - |  |  |  |  |  |  |  |  | 31.5 | 36.4 | - | alea | 1.1e-16 / 1.8e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.1e-16 / 1.8e-16 |
| GPA_w27 | 110529 | 565184 | 21.5 |  |  |  |  |  |  |  | - | - | - | - | - |  |  |  |  |  |  |  |  | 40.2 | 45.5 | - | alea | 1.1e-16 / 1.8e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 7.8e-16 / 1.3e-15 |
| GPA_w29 | 126871 | 614260 | 23.5 |  |  |  |  |  |  |  | - | - | - | - | - |  |  |  |  |  |  |  |  | 42 | 47.9 | - | alea | 1.1e-16 / 1.8e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.1e-16 / 1.8e-16 |
| GPA_w31 | 143187 | 663285 | 25.6 |  |  |  |  |  |  |  | - | - | - | - | - |  |  |  |  |  |  |  |  | 39.2 | 45.8 | - | alea | 1.1e-16 / 1.8e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.1e-16 / 1.8e-16 |
| addFun_max_w17 | 566 | 18099 | 38.8 | >600 | oom | oom | oom | >600 | 555 | >600 | >600 | >600 | >600 | >600 | >600 |  |  |  |  |  |  |  |  | 48.5 | 59.4 | >600 | alea | 2.2e-16 / 4.4e-16 |  |  |  |  |  | 1.1e-16 / 2.2e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.2e-16 / 4.4e-16 |
| addFun_max_w19 | 580 | 21035 | 44.5 | - | - | - | - | - | >600 | - | - | - | - | - | - |  |  |  |  |  |  |  |  | 220 | 238 | - | alea | 3.3e-16 / 6.7e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.2e-16 / 4.4e-16 |
| addFun_max_w21 | 594 | 23603 | 37.1 | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | oom | oom | - | alea | 5.6e-16 / 1.1e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| addFun_max_p16 | 196 | 3801 | 0.478 | - | - | - | - | - | 201 | >600 | - | - | - | - | - | - | - | - | - | - | - | - | >600 | - | - | - | tddlog | ref |  |  |  |  |  | 3.3e-16 / 6.6e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| addFun_max_p4 | 168 | 1249 | 0.352 | >600 | oom | >600 | oom | >600 | 128 | 70 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | oom | oom | >600 | oom | oom | 252 | oom | oom | oom | tddlog | ref |  |  |  |  |  | 3.3e-16 / 6.7e-16 | 3.3e-16 / 6.7e-16 |  |  |  |  |  |  |  |  |  |  |  |  | 3.3e-16 / 6.7e-16 |  |
| addFun_max_p64 | 366 | 13777 | 5.08 | - | - | - | - | - | >600 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| addFun_sum_w17 | 152 | 2800 | 0.433 | >600 | >600 | >600 | oom | >600 | 24.7 | >600 | >600 | >600 | >600 | oom | oom |  |  |  |  |  |  |  |  | 9.07 | 19.1 | >600 | alea | 7.8e-16 / 1.6e-15 |  |  |  |  |  | 2.2e-16 / 4.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 3.3e-16 / 6.7e-16 |
| addFun_sum_w19 | 168 | 3212 | 0.5 | - | - | - | - | - | 74 | - | - | - | - | - | - |  |  |  |  |  |  |  |  | 35 | 47.4 | - | alea | 5.6e-16 / 1.1e-15 |  |  |  |  |  | 3.3e-16 / 6.7e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 3.3e-16 / 6.7e-16 |
| addFun_sum_w21 | 184 | 3624 | 0.611 | - | - | - | - | - | 87.1 | - | - | - | - | - | - |  |  |  |  |  |  |  |  | 166 | 183 | - | alea | 6.7e-16 / 1.3e-15 |  |  |  |  |  | 3.3e-16 / 6.7e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.2e-16 / 4.4e-16 |
| addFun_sum_w23 | 196 | 3820 | 0.708 | - | - | - | - | - | 113 | - | - | - | - | - | - |  |  |  |  |  |  |  |  | oom | oom | - | alea | 6.7e-16 / 1.3e-15 |  |  |  |  |  | 2.2e-16 / 4.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| addFun_sum_w25 | 212 | 4208 | 0.728 | - | - | - | - | - | >600 | - | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| altermu2_w11 | 20458 | 274380 | 12.3 | oom | oom | oom | >600 | err | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  |  |  |  |  |  |  |  | 142 | 161 | >600 | alea | 4.7e-14 / 9.4e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 4.7e-14 / 9.4e-14 |
| altermu2_w13 | 40822 | 336280 | 19.6 | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | >600 | >600 | - | alea | 3.4e-14 / 6.8e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| altermu2_p16 | 7670 | 35720 | 1.84 | - | - | - | - | - | - | - | - | - | - | - | - | >600 | >600 | oom | oom | >600 | >600 | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| altermu2_p4 | 2230 | 13800 | 1.01 | >600 | oom | oom | oom | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | 15 | 219 | 32.4 | 26.2 | 183 | 76.5 | oom | >600 | >600 | >600 | >600 | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  | 3.3e-16 / 6.7e-16 | 2.2e-16 / 4.4e-16 | 4.9e-07 / 9.7e-07 | 4.8e-13 / 9.5e-13 | 4.8e-13 / 9.5e-13 | 4.8e-13 / 9.5e-13 |  |  |  |
| clickGraph_w4 | 52 | 135 | 0.006 | >600 | 161 | >600 | oom | 217 | 0.412 | 2.81 | 0.338 | 0.338 | 1.75 | 2.59 | 2.56 |  |  |  |  |  |  |  |  | 0.012 | 1.19 | 0.15 | alea | 2.2e-16 / 3.7e-16 |  | 8.2e-07 / 1.2e-06 |  |  | 3.3e-16 / 5.5e-16 | 2.2e-16 / 3.7e-16 | 2.2e-16 / 3.7e-16 | 2.2e-16 / 3.7e-16 | 2.2e-16 / 3.7e-16 | 2.2e-16 / 3.7e-16 | 2.2e-16 / 3.7e-16 | 2.2e-16 / 3.7e-16 |  |  |  |  |  |  |  |  | 2.2e-16 / 3.7e-16 |
| clickGraph_w6 | 104 | 345 | 0.013 | - | >600 | - | - | >600 | 1.66 | 2.81 | 1.36 | 1.75 | 11.6 | 5.11 | 5.12 |  |  |  |  |  |  |  |  | 0.725 | 3.14 | 0.37 | alea | 2.2e-16 / 4.1e-16 |  |  |  |  |  | 5.7e-14 / 9.7e-14 | 1.1e-16 / 2.1e-16 | 2.2e-16 / 4.2e-16 | 2.2e-16 / 4.2e-16 | 2.2e-16 / 4.2e-16 | 2.2e-16 / 4.2e-16 | 2.2e-16 / 4.2e-16 |  |  |  |  |  |  |  |  | 1.6e-14 / 2.8e-14 |
| clickGraph_w8 | 156 | 555 | 0.239 | - | - | - | - | - | 5.2 | 3.31 | 26.2 | 39.2 | 130 | 16.6 | 15.2 |  |  |  |  |  |  |  |  | 87.3 | 93.6 | 1.52 | alea | 8.9e-16 / 1.3e-15 |  |  |  |  |  | 6.5e-14 / 1.3e-13 | 4.4e-16 / 8.7e-16 | 4.4e-16 / 6.6e-16 | 4.4e-16 / 8.5e-16 | 4.4e-16 / 6.6e-16 | 4.4e-16 / 6.6e-16 | 4.4e-16 / 6.6e-16 |  |  |  |  |  |  |  |  | 1.1e-14 / 2.0e-14 |
| clickGraph_w10 | 208 | 765 | 0.32 | - | - | - | - | - | 597 (2/3) | 3.33 | >600 | >600 | >600 | 285 | 305 |  |  |  |  |  |  |  |  | oom | oom | 5.91 | Ganak | 5.2e-15 / 1.0e-14 |  |  |  |  |  | 3.5e-14 / 6.8e-14 | 3.3e-16 / 6.1e-16 |  |  |  | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 |  |  |  |  |  |  |  |  |  |
| clickGraph_w12 | 260 | 975 | 0.373 | - | - | - | - | - | 114 (2/3) | 4.56 | - | - | - | >600 | >600 |  |  |  |  |  |  |  |  | - | - | 30.9 | Ganak | 8.3e-15 / 1.4e-14 |  |  |  |  |  | 4.6e-14 / 7.8e-14 | 3.3e-16 / 6.1e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| clickGraph_w14 | 312 | 1185 | 0.835 | - | - | - | - | - | 431 (2/3) | 9.15 | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | wrong | Ganak | 1.3e-14 / 2.0e-14 |  |  |  |  |  | 3.3e-14 / 6.4e-14 | 3.3e-16 / 6.7e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| clickGraph_w16 | 364 | 1395 | 0.688 | - | - | - | - | - | >600 | 29.9 | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | oom | Ganak | 4.2e-15 / 8.4e-15 |  |  |  |  |  |  | 3.3e-16 / 6.7e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| clinicalTrial1_w4 | 807 | 3200 | 0.041 | >600 | >600 | oom | oom | >600 | err | 166 | >600 | >600 | 33 | err | err |  |  |  |  |  |  |  |  | 0.365 | 1 | 1.12 | alea | 5.8e-15 / 2.9e-14 |  |  |  |  |  |  | 5.9e-15 / 2.9e-14 |  |  | 5.9e-15 / 2.9e-14 |  |  |  |  |  |  |  |  |  |  | 5.8e-15 / 2.9e-14 |
| clinicalTrial1_w6 | 1613 | 8000 | 0.245 | - | - | - | - | - | - | zero | - | - | 330 | - | - |  |  |  |  |  |  |  |  | 11.3 | 12.5 | 4.77 | alea | 1.1e-14 / 6.4e-14 |  |  |  |  |  |  |  |  |  | 1.1e-14 / 6.4e-14 |  |  |  |  |  |  |  |  |  |  | 1.1e-14 / 6.4e-14 |
| clinicalTrial1_w8 | 2419 | 12800 | 1.73 | - | - | - | - | - | - | 333 | - | - | >600 | - | - |  |  |  |  |  |  |  |  | oom | oom | 24 | alea | 4.2e-16 / 2.4e-15 |  |  |  |  |  |  | 5.6e-16 / 3.3e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| clinicalTrial1_w10 | 3225 | 17600 | 51.6 | - | - | - | - | - | - | >600 | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | 121 | Ganak | 5.6e-17 / 3.3e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| clinicalTrial2_w6 | 89 | 412 | 0.009 | >600 | 9.31 | err | oom | >600 | 0.792 | 2.96 | 3.91 | 4.91 | 7.62 | 5.14 | 5.11 |  |  |  |  |  |  |  |  | 0.174 | 2.63 | 0.389 | alea | 6.7e-16 / 1.3e-15 |  | 7.4e-07 / 1.5e-06 |  |  |  | 6.7e-16 / 1.3e-15 | 6.7e-16 / 1.3e-15 | 6.7e-16 / 1.3e-15 | 6.7e-16 / 1.3e-15 | 6.7e-16 / 1.3e-15 | 6.7e-16 / 1.3e-15 | 6.7e-16 / 1.3e-15 |  |  |  |  |  |  |  |  | 6.7e-16 / 1.3e-15 |
| clinicalTrial2_w8 | 133 | 658 | 0.008 | - | >600 | - | - | - | 5.97 | 3.2 | 108 | 141 | 38.6 | 21.4 | 20.1 |  |  |  |  |  |  |  |  | 4.37 | 8.16 | 1.07 | alea | 5.6e-16 / 1.1e-15 |  |  |  |  |  | 1.2e-13 / 4.3e-13 | 6.7e-16 / 1.3e-15 | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 |  |  |  |  |  |  |  |  | 4.6e-14 / 3.8e-13 |
| clinicalTrial2_w10 | 177 | 904 | 0.012 | - | - | - | - | - | 29.8 | 6.54 | >600 | oom | >600 | 448 | 469 |  |  |  |  |  |  |  |  | 157 | 167 | 4.73 | alea | 1.1e-15 / 2.2e-15 |  |  |  |  |  | 1.1e-13 / 9.5e-13 | 1.0e-15 / 2.0e-15 |  |  |  | 1.0e-15 / 2.0e-15 | 1.0e-15 / 2.0e-15 |  |  |  |  |  |  |  |  | 1.1e-13 / 7.8e-13 |
| clinicalTrial2_w12 | 221 | 1150 | 0.022 | - | - | - | - | - | 135 | 6.23 | - | - | - | err | err |  |  |  |  |  |  |  |  | oom | oom | 24 | alea | 6.1e-15 / 1.2e-14 |  |  |  |  |  | 1.0e-13 / 2.1e-13 | 4.4e-16 / 9.0e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| clinicalTrial2_w14 | 265 | 1396 | 0.019 | - | - | - | - | - | 339 | 11.8 | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | 118 | Ganak | 1.1e-16 / 2.2e-16 |  |  |  |  |  | 1.7e-13 / 3.3e-13 | 2.2e-16 / 4.5e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| clinicalTrial2_w16 | 309 | 1642 | 0.019 | - | - | - | - | - | 316 | 44.3 | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | oom | Ganak | 2.2e-16 / 4.4e-16 |  |  |  |  |  | 1.9e-13 / 1.5e-12 | 2.2e-16 / 4.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| clinicalTrial2_w18 | 353 | 1888 | 0.021 | - | - | - | - | - | 219 | 142 | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | - | Ganak | 2.2e-16 / 4.4e-16 |  |  |  |  |  | 1.5e-13 / 3.0e-13 | 2.2e-16 / 4.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| coinBias_w16 | 12422 | 152450 | 8.42 | oom | oom | oom | oom | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  |  |  |  |  |  |  |  | 421 | 466 | oom | alea | 2.4e-15 / 5.3e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 8.5e-15 / 9.1e-15 |
| coinBias_w18 | 20630 | 177156 | 13.7 | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | >600 | >600 | - | alea | 3.8e-15 / 6.1e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| conjugate_ex3_w18 | 9147 | 239177 | 41.9 | oom | oom | oom | >600 | err | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | 246 | 281 | oom | alea | 5.6e-15 / 1.0e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.6e-15 / 1.3e-14 |
| conjugate_ex3_w20 | 9135 | 290908 | 140 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | >600 | >600 | - | alea | 6.7e-15 / 1.3e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| conjugate_gaussians_w17 | 177 | 2788 | 0.425 | >600 | oom | oom | oom | >600 | >600 | >600 | >600 | oom | err | >600 | >600 |  |  |  |  |  |  |  |  | 87.2 | 107 | >600 | alea | 4.0e-15 / 2.1e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.3e-10 / 7.9e-09 |
| conjugate_gaussians_w19 | 201 | 3258 | 0.527 | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | >600 | >600 | - | alea | 2.4e-15 / 5.8e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| fair_dt14_w12 | 1795 | 9432 | 5.12 | >600 | oom | oom | oom | >600 | 88.8 | 119 | >600 | >600 | err | >600 | >600 |  |  |  |  |  |  |  |  | 0.606 | 1.69 | oom | alea | 1.7e-17 / 5.6e-16 |  |  |  |  |  | 8.3e-17 / 1.3e-15 | 2.8e-17 / 4.3e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  | 8.3e-17 / 1.3e-15 |
| fair_dt14_w16 | 3034 | 13225 | 13.2 | - | - | - | - | - | 537 | >600 | - | - | - | - | - |  |  |  |  |  |  |  |  | 4.57 | 5.81 | - | alea | 5.2e-17 / 1.7e-15 |  |  |  |  |  | 1.0e-17 / 3.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.0e-17 / 3.4e-16 |
| fair_dt14_w20 | 4259 | 17021 | 34.5 | - | - | - | - | - | >600 | - | - | - | - | - | - |  |  |  |  |  |  |  |  | 105 | 109 | - | alea | 5.6e-17 / 1.0e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.6e-17 / 8.6e-16 |
| fair_dt14_w24 | 5272 | 20135 | 41.6 | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | oom | oom | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| fair_dt16_w12 | 420 | 2143 | 0.125 | >600 | oom | oom | oom | >600 | 1.88 | 8.54 | >600 | >600 | 36.1 | >600 | >600 |  |  |  |  |  |  |  |  | 0.088 | 1.09 | 3.88 | alea | 2.8e-17 / 2.8e-16 |  |  |  |  |  | 8.3e-17 / 4.3e-16 | 8.3e-17 / 4.3e-16 |  |  | 1.7e-16 / 8.7e-16 |  |  |  |  |  |  |  |  |  |  | 8.3e-17 / 4.3e-16 |
| fair_dt16_w16 | 638 | 2884 | 0.213 | - | - | - | - | - | 4.1 | 19.9 | - | - | 108 | - | - |  |  |  |  |  |  |  |  | 0.149 | 1.36 | 3.9 | alea | 5.6e-17 / 2.9e-16 |  |  |  |  |  | 5.6e-17 / 2.9e-16 | 5.6e-17 / 2.9e-16 |  |  | 5.6e-17 / 2.9e-16 |  |  |  |  |  |  |  |  |  |  | 5.6e-17 / 9.8e-16 |
| fair_dt16_w20 | 793 | 3420 | 0.358 | - | - | - | - | - | 6.41 | 43 | - | - | >600 | - | - |  |  |  |  |  |  |  |  | 0.179 | 1.49 | 9.17 | alea | 8.3e-17 / 5.6e-16 |  |  |  |  |  | 1.1e-16 / 5.8e-16 | 8.3e-17 / 5.6e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.1e-16 / 5.8e-16 |
| fair_dt16_w24 | 780 | 3394 | 0.356 | - | - | - | - | - | 7.05 | 46.1 | - | - | - | - | - |  |  |  |  |  |  |  |  | 0.133 | 1.28 | 8.94 | alea | 5.6e-17 / 2.9e-16 |  |  |  |  |  | 5.6e-17 / 9.8e-16 | 1.4e-16 / 7.3e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  | 8.3e-17 / 1.7e-15 |
| fair_dt16_w28 | 754 | 3342 | 0.269 | - | - | - | - | - | 6.76 | 38 | - | - | - | - | - |  |  |  |  |  |  |  |  | 0.175 | 1.5 | 8.45 | alea | 2.8e-17 / 4.2e-16 |  |  |  |  |  | 1.4e-17 / 2.8e-16 | 2.8e-17 / 1.5e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.4e-17 / 2.8e-16 |
| fair_dt16_w32 | 741 | 3316 | 0.291 | - | - | - | - | - | 5.2 | 57 | - | - | - | - | - |  |  |  |  |  |  |  |  | 0.168 | 1.36 | 8.18 | alea | 2.8e-17 / 4.2e-16 |  |  |  |  |  | 2.8e-17 / 5.6e-16 | 5.6e-17 / 4.2e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.8e-17 / 1.5e-16 |
| fair_dt4_w12 | 1871 | 9632 | 4.25 | >600 | oom | oom | oom | >600 | 123 | 173 | >600 | >600 | err | >600 | >600 |  |  |  |  |  |  |  |  | 0.717 | 2.04 | 263 | alea | 2.8e-17 / 1.8e-16 |  |  |  |  |  | 8.3e-17 / 5.5e-16 | 2.8e-17 / 2.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  | 8.3e-17 / 5.5e-16 |
| fair_dt4_w16 | 3064 | 13311 | 13.1 | - | - | - | - | - | 253 | >600 | - | - | - | - | - |  |  |  |  |  |  |  |  | 4.36 | 5.57 | >600 | alea | 2.8e-17 / 1.9e-16 |  |  |  |  |  | 5.6e-17 / 3.7e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.6e-17 / 3.7e-16 |
| fair_dt4_w20 | 4349 | 17315 | 30.9 | - | - | - | - | - | 414 | - | - | - | - | - | - |  |  |  |  |  |  |  |  | 87.8 | 91.2 | - | alea | 2.8e-17 / 2.5e-16 |  |  |  |  |  | 2.8e-17 / 2.5e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.8e-17 / 8.6e-16 |
| fair_dt4_w24 | 5360 | 20425 | 43.4 | - | - | - | - | - | >600 | - | - | - | - | - | - |  |  |  |  |  |  |  |  | oom | oom | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| fair_dt44_w12 | 478 | 2361 | 0.168 | >600 | >600 | oom | oom | >600 | 2.76 | 10.6 | >600 | >600 | 49.8 | 542 | 543 |  |  |  |  |  |  |  |  | 0.113 | 1.34 | 2.04 | alea | 5.6e-17 / 8.1e-16 |  |  |  |  |  | 2.8e-17 / 4.6e-16 | 2.8e-17 / 4.6e-16 |  |  | 5.6e-17 / 3.2e-16 | 5.6e-17 / 4.6e-16 | 5.6e-17 / 4.6e-16 |  |  |  |  |  |  |  |  | 7.6e-17 / 1.3e-15 |
| fair_dt44_w16 | 696 | 3090 | 0.231 | - | - | - | - | - | 4.43 | 21.5 | - | - | 107 | >600 | >600 |  |  |  |  |  |  |  |  | 0.132 | 1.26 | 4.67 | alea | 3.5e-17 / 5.8e-16 |  |  |  |  |  | 6.9e-17 / 1.2e-15 | 5.6e-17 / 3.2e-16 |  |  | 5.6e-17 / 3.5e-16 |  |  |  |  |  |  |  |  |  |  | 5.6e-17 / 9.3e-16 |
| fair_dt44_w20 | 851 | 3614 | 0.329 | - | - | - | - | - | 7.04 | 61.6 | - | - | >600 | - | - |  |  |  |  |  |  |  |  | 0.165 | 1.26 | 8.09 | alea | 1.1e-16 / 6.4e-16 |  |  |  |  |  | 8.3e-17 / 4.8e-16 | 2.2e-16 / 1.3e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.4e-16 / 8.0e-16 |
| fair_dt44_w24 | 838 | 3588 | 0.282 | - | - | - | - | - | 4.04 | 55.4 | - | - | - | - | - |  |  |  |  |  |  |  |  | 0.189 | 1.42 | 7.79 | alea | 8.3e-17 / 5.8e-16 |  |  |  |  |  | 5.6e-17 / 6.9e-16 | 1.1e-16 / 6.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.6e-17 / 8.1e-16 |
| fair_dt44_w28 | 812 | 3536 | 0.273 | - | - | - | - | - | 7.93 | 52.7 | - | - | - | - | - |  |  |  |  |  |  |  |  | 0.165 | 1.38 | 7.56 | alea | 8.3e-17 / 4.8e-16 |  |  |  |  |  | 5.6e-17 / 3.2e-16 | 1.4e-16 / 8.0e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.6e-17 / 3.2e-16 |
| fair_dt44_w32 | 799 | 3510 | 0.267 | - | - | - | - | - | 5.89 | 51.9 | - | - | - | - | - |  |  |  |  |  |  |  |  | 0.166 | 1.43 | 6.94 | alea | 5.6e-17 / 3.2e-16 |  |  |  |  |  | 8.3e-17 / 4.8e-16 | 2.8e-17 / 2.3e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  | 8.3e-17 / 4.8e-16 |
| genes_t5 | 67284 | 308661 | 12.4 | oom | oom | >600 | >600 | err | >600 | >600 | >600 | >600 | >600 | >600 | >600 | err | err | err | err | err | err | err | err | oom | oom | oom | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| genes5_w17 | 24148 | 173021 | 6.52 | oom | oom | oom | >600 | err | >600 | >600 | >600 | >600 | >600 | >600 | >600 | err | err | err | err | err | err | err | err | 53.1 | 63.4 | oom | alea | 5.5e-15 / 1.1e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.4e-15 / 1.1e-14 |
| genes5_w19 | 31844 | 196665 | 7.61 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 86 | 104 | - | alea | 7.5e-15 / 1.5e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 7.4e-15 / 1.5e-14 |
| genes5_w21 | 39336 | 219821 | 8.43 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 144 | 165 | - | alea | 6.9e-15 / 1.4e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 6.8e-15 / 1.4e-14 |
| genes5_w23 | 46572 | 242497 | 9.31 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 313 | 338 | - | alea | 3.7e-15 / 7.3e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 3.5e-15 / 7.0e-15 |
| genes5_w25 | 53588 | 264781 | 10.1 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | oom | oom | - | alea | 3.9e-15 / 8.3e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| laplace_w13 | 4125 | 110955 | 6.28 | oom | oom | oom | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  |  |  |  |  |  |  |  | 239 | 261 | >600 | alea | 4.6e-15 / 9.1e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 3.1e-07 / 1.3e-06 |
| laplace_w15 | 8145 | 158877 | >600 | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | oom | oom | - | alea |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| linreg_w8 | 372 | 5404 | 0.131 | >600 | oom | err | oom | >600 | wrong | 74.1 | >600 | >600 | >600 | >600 | >600 |  |  |  |  |  |  |  |  | 3.67 | 12.6 | >600 | alea | 3.9e-15 / 2.8e-14 |  |  |  |  |  | 9.5e-03 / 1.0e+00 | 3.9e-15 / 2.8e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  | 4.1e-15 / 2.8e-14 |
| linreg_w9 | 564 | 6754 | 0.245 | - | - | err | - | - | >600 | 143 | - | - | - | - | - |  |  |  |  |  |  |  |  | 13.5 | 23.4 | - | alea | 4.4e-15 / 1.5e-14 |  |  |  |  |  |  | 4.2e-15 / 1.5e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  | 4.2e-15 / 1.5e-14 |
| linreg_w10 | 756 | 8204 | 6.82 | - | - | err | - | - | - | 249 | - | - | - | - | - |  |  |  |  |  |  |  |  | 60 | 72.7 | - | alea | 5.2e-15 / 2.3e-14 |  |  |  |  |  |  | 6.2e-15 / 2.3e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  | 6.1e-15 / 2.3e-14 |
| linreg_w11 | 948 | 9788 | 24.5 | - | - | err | - | - | - | >600 | - | - | - | - | - |  |  |  |  |  |  |  |  | 272 | 292 | - | alea | 6.3e-15 / 9.1e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 6.8e-15 / 9.3e-15 |
| linreg_w12 | 1140 | 11532 | 40 | - | - | err | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | oom | oom | - | alea | 5.3e-15 / 2.0e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| linreg_w13 | 1332 | 13436 | 173 | - | - | err | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | - | alea | 8.0e-15 / 2.7e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| linreg_w14 | 1524 | 15500 | oom | - | - | 0.186 | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| linreg_w15 | 1716 | 17724 | - | - | - | 0.181 | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| markov_n10 | 2050 | 7887 | 0.489 | >600 | oom | oom | oom | wrong | err | 155 | >600 | >600 | 137 | >600 | >600 |  |  |  |  |  |  |  |  | 0.283 | 1.13 | 4.99 | alea | 1.7e-15 / 9.4e-15 |  |  |  |  | 6.7e-03 / 4.8e-02 |  | 1.7e-15 / 8.1e-15 |  |  | 1.7e-15 / 7.8e-15 |  |  |  |  |  |  |  |  |  |  | 1.4e-15 / 7.6e-15 |
| markov_n100 | 20500 | 78897 | 3.24 | - | - | - | - | wrong | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | 301 | alea | 3.9e-15 / 2.7e-12 |  |  |  |  | 1.8e-03 / 2.4e-01 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| markov_n20 | 4100 | 15777 | 0.456 | - | - | - | - | wrong | - | 389 | - | - | >600 | - | - |  |  |  |  |  |  |  |  | 1.82 | 2.96 | 16.8 | alea | 1.2e-14 / 1.5e-14 |  |  |  |  | 6.9e-04 / 4.0e-02 |  | 1.2e-14 / 2.1e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.2e-14 / 2.2e-14 |
| markov_n200 | 41000 | 157797 | 10.1 | - | - | - | - | oom | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | >600 | alea | 1.3e-13 / 9.0e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| markov_n50 | 10250 | 39447 | 1.33 | - | - | - | - | wrong | - | >600 | - | - | - | - | - |  |  |  |  |  |  |  |  | >600 | >600 | 83.9 | alea | 2.8e-14 / 2.8e-14 |  |  |  |  | 3.9e-04 / 2.0e-01 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| multimodal_w11 | 4617 | 116721 | oom | oom | oom | oom | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | oom | >600 | >600 | >600 | oom | >600 | oom | oom | >600 | alea |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| normal_mixture_mu1_w7 | 2583 | 25704 | 0.964 | >600 | oom | oom | err | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  |  |  |  |  |  |  |  | 23.1 | 27.9 | >600 | alea | 2.9e-15 / 2.5e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.9e-15 / 2.4e-14 |
| normal_mixture_mu1_w9 | 5289 | 35812 | 139 | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | oom | oom | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| normal_mixture_mu2_w7 | 2583 | 45770 | 1.46 | >600 | oom | oom | err | >600 | >600 | >600 | >600 | >600 | err | >600 | >600 |  |  |  |  |  |  |  |  | 24.7 | 30.1 | >600 | alea | 1.3e-14 / 1.1e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.2e-14 / 1.1e-13 |
| normal_mixture_mu2_w9 | 7913 | 63750 | 173 | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | oom | oom | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| normal_mixture_theta_w9 | 10537 | 118560 | 31.9 | oom | oom | oom | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  |  |  |  |  |  |  |  | oom | oom | >600 | Ganak | 4.4e-16 / 5.2e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| pedestrian_s4 | 59 | 1393 | 0.229 | >600 | oom | oom | oom | >600 | 51.3 | 54.5 | >600 | oom | >600 | >600 | >600 |  |  |  |  |  |  |  |  | 3.99 | 7.79 | >600 | alea | 6.7e-16 / 2.7e-14 |  |  |  |  |  | 1.4e-14 / 1.0e+00 | 6.7e-16 / 2.7e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  | 3.2e-15 / 1.0e+00 |
| pedestrian_s6 | 69 | 1968 | 0.691 | - | - | - | - | - | >600 | >600 | - | - | - | - | - |  |  |  |  |  |  |  |  | 296 | 300 | - | alea | 2.8e-16 / 4.0e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 3.3e-15 / 4.2e-14 |
| pedestrian_s8 | 79 | 2568 | 1.79 | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | >600 | >600 | - | alea | 6.7e-16 / 4.1e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| pi_w8 | 12 | 3759 | 0.069 | >600 | oom | >600 | oom | >600 | 1.48 | 14.9 | 7 | 8.22 | 8.29 | >600 | >600 |  |  |  |  |  |  |  |  | 0.039 | 0.444 | 2.36 | alea | 1.1e-16 / 2.4e-16 |  |  |  |  |  | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 |  |  |  |  |  |  |  |  |  |  | 1.1e-16 / 2.4e-16 |
| pi_w10 | 16 | 6141 | 0.143 | - | - | - | - | - | 2.19 | 49.9 | 302 | 344 | 55.4 | - | - |  |  |  |  |  |  |  |  | 0.165 | 0.601 | 27.2 | alea | 5.6e-17 / 1.2e-16 |  |  |  |  |  | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 |  |  |  |  |  |  |  |  |  |  | 5.6e-17 / 1.2e-16 |
| pi_w12 | 20 | 9099 | 0.415 | - | - | - | - | - | 4.55 | 182 | >600 | >600 | err | - | - |  |  |  |  |  |  |  |  | 1.01 | 1.47 | wrong | alea | 1.1e-16 / 2.4e-16 |  |  |  |  |  | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.1e-16 / 2.4e-16 |
| pi_w14 | 24 | 12633 | 1.46 | - | - | - | - | - | 26.5 | 498 | - | - | - | - | - |  |  |  |  |  |  |  |  | 12.3 | 13.3 | >600 | alea | 5.6e-17 / 1.2e-16 |  |  |  |  |  | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.6e-17 / 1.2e-16 |
| pi_w16 | 28 | 16743 | 6.12 | - | - | - | - | - | >600 | >600 | - | - | - | - | - |  |  |  |  |  |  |  |  | oom | oom | - | alea | 1.1e-16 / 2.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| population_s1 | 2540 | 8332 | 0.353 | >600 | >600 | oom | 0.007 | err | >600 | >600 | >600 | >600 | err | oom | oom |  |  |  |  |  |  |  |  | >600 | >600 | >600 | alea | 1.1e-15 / 9.3e-14 |  |  |  | 1.4e-14 / 3.9e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| population_s12 | 34957 | 115507 | 18.2 | - | - | - | 0.045 | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | - | tddlog | ref |  |  |  | 6.1e-12 / 9.3e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| population_s16 | 46780 | 154582 | 33.7 | - | - | - | 0.072 | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | - | tddlog | ref |  |  |  | 6.1e-12 / 9.3e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| population_s2 | 5587 | 18373 | 8.29 | - | - | - | 0.009 | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | - | tddlog | ref |  |  |  | 2.4e-14 / 1.1e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| population_s3 | 8637 | 28421 | 9.33 | - | - | - | 0.011 | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | - | tddlog | ref |  |  |  | 8.0e-12 / 9.7e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| population_s4 | 11311 | 37357 | 10.2 | - | - | - | 0.013 | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | - | tddlog | ref |  |  |  | 6.4e-12 / 9.2e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| population_s6 | 17410 | 57448 | 12.6 | - | - | - | 0.02 | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | - | tddlog | ref |  |  |  | 1.2e-13 / 2.3e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| population_s8 | 23134 | 76432 | 30.7 | - | - | - | 0.027 | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | - | tddlog | ref |  |  |  | 6.1e-12 / 9.3e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| spacex_w13 | 279 | 2037 | 2.27 | >600 | >600 | >600 | oom | >600 | >600 | 308 | >600 | >600 | >600 | >600 | >600 |  |  |  |  |  |  |  |  | 2.14 | 9 | >600 | alea | 4.1e-15 / 6.1e-15 |  |  |  |  |  |  | 2.2e-16 / 1.1e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.2e-16 / 2.0e-02 |
| spacex_w15 | 357 | 2423 | 12.8 | - | - | - | - | - | - | >600 | - | - | - | - | - |  |  |  |  |  |  |  |  | 14.7 | 22.8 | - | alea | 5.4e-15 / 8.7e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.2e-16 / 1.8e-02 |
| spacex_w17 | 435 | 2809 | 55.7 | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | 143 | 156 | - | alea | 2.8e-14 / 7.0e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.2e-16 / 6.3e-03 |
| spacex_w19 | 513 | 3195 | oom | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | >600 | >600 | - | alea |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| spacex_p16 | 293 | 1752 | 104 | - | - | - | - | - | - | >600 | >600 | - | - | >600 | >600 | >600 | >600 | oom | oom | >600 | oom | oom | 262 | - | - | - | alea | 6.2e-14 / 1.9e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.8e-16 / 8.8e-15 |  |
| spacex_p4 | 106 | 635 | 0.042 | >600 | oom | oom | oom | >600 | >600 | 3.11 | 311 | >600 | >600 | 98 | 74.8 | 2.05 | 2.44 | 0.294 | 0.411 | 18.9 | 0.336 | 56.5 | 106 | >600 | >600 | >600 | alea | 2.2e-16 / 1.7e-14 |  |  |  |  |  |  | 2.2e-16 / 1.7e-14 | 4.1e-15 / 2.4e-14 |  |  | 3.3e-16 / 1.7e-14 | 3.3e-16 / 1.7e-14 | 2.2e-16 / 1.7e-14 | 2.8e-16 / 1.7e-14 | 4.8e-07 / 1.0e+00 | 4.1e-13 / 8.2e-13 | 4.1e-13 / 8.2e-13 | 4.1e-13 / 8.2e-13 | 3.3e-16 / 1.7e-14 | 2.2e-16 / 1.7e-14 |  |
| spacex_p64 | 920 | 6092 | oom | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | >600 | - | - | - | alea |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| trueskill_w8 | 312 | 3868 | 8.95 | >600 | >600 | oom | oom | >600 | >600 | 551 | >600 | >600 | err | >600 | >600 |  |  |  |  |  |  |  |  | 3.91 | 4.5 | >600 | alea | 2.2e-16 / 4.6e-16 |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  | 7.8e-16 / 1.6e-15 |
| trueskill_w10 | 376 | 5592 | 193 | - | - | - | - | - | - | >600 | - | - | - | - | - |  |  |  |  |  |  |  |  | oom | oom | - | alea | 3.3e-16 / 6.7e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| tug_of_war_w16 | 392 | 2044 | 0.374 | >600 | >600 | oom | oom | >600 | 10.3 | 386 | >600 | >600 | 14.3 | 163 | 166 |  |  |  |  |  |  |  |  | 12.2 | 13.1 | >600 | alea | 1.1e-15 / 2.1e-15 |  |  |  |  |  | 4.3e-15 / 8.5e-15 | 1.1e-15 / 2.1e-15 |  |  | 1.1e-15 / 2.2e-15 | 2.1e-15 / 4.1e-15 | 2.1e-15 / 4.1e-15 |  |  |  |  |  |  |  |  | 1.9e-15 / 3.8e-15 |
| tug_of_war_w18 | 456 | 2292 | 0.448 | - | - | - | - | - | >600 | >600 | - | - | 18.5 | 266 | 264 |  |  |  |  |  |  |  |  | 54.4 | 56.2 | - | alea | 2.2e-16 / 4.4e-16 |  |  |  |  |  |  |  |  |  | 7.2e-16 / 1.4e-15 | 7.8e-16 / 1.6e-15 | 7.8e-16 / 1.6e-15 |  |  |  |  |  |  |  |  | 3.4e-15 / 6.9e-15 |
| tug_of_war_w20 | 520 | 2540 | 0.568 | - | - | - | - | - | - | - | - | - | 21.4 | 344 | 341 |  |  |  |  |  |  |  |  | oom | oom | - | alea | 8.3e-16 / 1.7e-15 |  |  |  |  |  |  |  |  |  | 6.7e-16 / 1.3e-15 | 6.1e-16 / 1.2e-15 | 6.1e-16 / 1.2e-15 |  |  |  |  |  |  |  |  |  |
| tug_of_war_w22 | 550 | 2720 | 0.55 | - | - | - | - | - | - | - | - | - | 24.6 | 452 | 426 |  |  |  |  |  |  |  |  | - | - | - | Ganak | 5.0e-16 / 1.0e-15 |  |  |  |  |  |  |  |  |  | 5.6e-17 / 1.1e-16 | 5.6e-17 / 1.1e-16 | 5.6e-17 / 1.1e-16 |  |  |  |  |  |  |  |  |  |
| tug_of_war_w24 | 610 | 2960 | 0.692 | - | - | - | - | - | - | - | - | - | 24.6 | 350 | 290 |  |  |  |  |  |  |  |  | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  | 1.1e-15 / 2.1e-15 | 1.1e-15 / 2.2e-15 | 1.1e-15 / 2.2e-15 |  |  |  |  |  |  |  |  |  |
| tug_of_war_w26 | 670 | 3200 | 0.723 | - | - | - | - | - | - | - | - | - | 24.7 | 351 | 343 |  |  |  |  |  |  |  |  | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  | 7.8e-16 / 1.6e-15 | 4.9e-15 / 9.9e-15 | 4.9e-15 / 9.9e-15 |  |  |  |  |  |  |  |  |  |
| tug_of_war_w28 | 722 | 3416 | 0.739 | - | - | - | - | - | - | - | - | - | 28.5 | 484 | 486 |  |  |  |  |  |  |  |  | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  | 9.4e-16 / 1.9e-15 | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 |  |  |  |  |  |  |  |  |  |
| weekend_w18 | 49151 | 368566 | 15.2 | oom | oom | oom | oom | err | >600 | >600 | oom | oom | oom | oom | oom |  |  |  |  |  |  |  |  | 35.7 | 39.9 | oom | alea | 3.3e-16 / 8.9e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 3.4e-12 / 9.0e-12 |
| weekend_w20 | 65535 | 417718 | 16.7 | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | 30.7 | 35 | - | alea | 2.8e-16 / 7.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.6e-11 / 6.9e-11 |
| weekend_w22 | 81919 | 466870 | 18.6 | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | 30.3 | 33.6 | - | alea | 5.6e-17 / 1.5e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 6.2e-11 / 1.7e-10 |
| weekend_w24 | 98303 | 516022 | 20.2 | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | 39.5 | 44.5 | - | alea | 8.3e-16 / 2.2e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.0e-10 / 2.8e-10 |
| weekend_w26 | 114687 | 565174 | 22 | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | 28.4 | 33 | - | alea | 9.4e-16 / 2.5e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.7e-10 / 1.5e-09 |
| weekend_w28 | 131071 | 614326 | 23.5 | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | 43.5 | 49.6 | - | alea | 6.1e-16 / 1.6e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 8.8e-10 / 2.3e-09 |
| weekend_w30 | 147455 | 663478 | 24.8 | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | 37.5 | 44.3 | - | alea | 1.6e-15 / 4.2e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 4.3e-09 / 1.2e-08 |
| weekend_p1024 | 28671 | 131010 | 3.72 | - | - | - | - | - | 298 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 5.64 | 7.08 | oom | alea | 2.8e-16 / 7.4e-16 |  |  |  |  |  | 7.9e-11 / 2.1e-10 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.5e-12 / 6.7e-12 |
| weekend_p16 | 639 | 2118 | 0.114 | - | 0.42 | - | - | - | 1.18 | 6.33 | - | - | 10.1 | >600 | >600 | oom | 3.68 | 45.3 | 15.3 | 201 | 46.3 | oom | 382 | 0.06 | 0.684 | 1.04 | alea | 4.4e-16 / 1.2e-15 |  | 2.8e-07 / 7.5e-07 |  |  |  | 2.7e-10 / 7.2e-10 | 6.1e-16 / 1.6e-15 |  |  | 2.2e-16 / 5.9e-16 |  |  |  | 2.8e-16 / 7.4e-16 | 2.8e-07 / 7.5e-07 | 5.2e-14 / 1.4e-13 | 5.2e-14 / 1.4e-13 | 5.2e-14 / 1.4e-13 |  | 4.4e-16 / 1.2e-15 | 6.3e-10 / 1.7e-09 |
| weekend_p256 | 8191 | 33230 | 2.73 | - | oom | - | - | - | 24.4 | - | - | - | >600 | - | - | - | - | - | - | - | - | - | - | 1.08 | 1.88 | 276 | alea | 1.4e-15 / 3.7e-15 |  |  |  |  |  | 5.5e-10 / 1.5e-09 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.2e-10 / 3.1e-10 |
| weekend_p4 | 175 | 530 | 0.011 | >600 | 0.162 | oom | oom | >600 | 0.67 | 2.81 | >600 | >600 | 2.51 | 222 | 214 | 2.06 | 2.21 | 0.347 | 1.54 | 19.1 | 0.608 | 69.3 | 116 | 0.024 | 0.615 | 0.278 | alea | 1.7e-15 / 4.4e-15 |  | 4.9e-07 / 1.3e-06 |  |  |  | 1.0e-09 / 2.7e-09 | 1.4e-15 / 3.7e-15 |  |  | 1.2e-15 / 3.1e-15 | 1.8e-15 / 4.9e-15 | 1.8e-15 / 4.9e-15 | 1.4e-15 / 3.7e-15 | 1.4e-15 / 3.7e-15 | 4.9e-07 / 1.3e-06 | 3.8e-13 / 1.0e-12 | 3.8e-13 / 1.0e-12 | 3.8e-13 / 1.0e-12 | 1.2e-15 / 3.3e-15 | 1.5e-15 / 4.0e-15 | 1.7e-10 / 4.6e-10 |
| weekend_p64 | 2303 | 8410 | 1.47 | - | 1.84 | - | - | - | 7.45 | >600 | - | - | 126 | - | - | - | >600 | oom | oom | >600 | oom | - | >600 | 0.245 | 0.921 | 10.9 | alea | 5.6e-17 / 1.5e-16 |  | 2.0e-07 / 5.3e-07 |  |  |  | 2.7e-11 / 7.2e-11 |  |  |  | 5.6e-17 / 1.5e-16 |  |  |  |  |  |  |  |  |  |  | 7.8e-11 / 2.1e-10 |
| wmi_path_n128 | 1024 | 11316 | 0.626 | - | - | - | - | - | - | - | - | - | >600 | - | - |  |  |  |  |  |  |  |  | 11.9 | 12.9 | - | alea | 5.6e-16 / 7.8e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.0e-15 / 1.4e-15 |
| wmi_path_n16 | 128 | 1348 | 0.074 | >600 | >600 | >600 | oom | >600 | >600 | 51.4 | >600 | oom | 17.6 | 42.7 | 49.1 |  |  |  |  |  |  |  |  | 0.174 | 0.576 | >600 | alea | 2.2e-16 / 3.1e-16 |  |  |  |  |  |  | 3.3e-16 / 4.7e-16 |  |  | 2.2e-16 / 3.1e-16 | 2.2e-16 / 3.1e-16 | 2.2e-16 / 3.1e-16 |  |  |  |  |  |  |  |  | 5.3e-15 / 7.5e-15 |
| wmi_path_n256 | 2048 | 22708 | 1.3 | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  | oom | oom | - | alea | 4.6e-15 / 6.4e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| wmi_path_n32 | 256 | 2772 | 0.147 | - | - | - | - | - | - | >600 | - | - | 94.8 | 184 | 194 |  |  |  |  |  |  |  |  | 0.729 | 1.19 | - | alea | 3.3e-16 / 4.7e-16 |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  | 4.7e-12 / 6.6e-12 |
| wmi_path_n64 | 512 | 5620 | 0.313 | - | - | - | - | - | - | - | - | - | 430 | >600 | >600 |  |  |  |  |  |  |  |  | 3.3 | 4.02 | - | alea | 7.8e-16 / 1.1e-15 |  |  |  |  |  |  |  |  |  | 8.9e-16 / 1.3e-15 |  |  |  |  |  |  |  |  |  |  | 8.9e-16 / 1.3e-15 |
| wmi_star_n128 | 1032 | 11405 | 0.742 | - | - | - | - | - | - | 230 | - | - | err | - | - |  |  |  |  |  |  |  |  | 10.6 | 11.5 | 78.3 | alea | 1.1e-16 / 2.2e-16 |  |  |  |  |  |  | 3.3e-16 / 6.6e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.6e-16 / 1.1e-15 |
| wmi_star_n16 | 136 | 1437 | 0.1 | >600 | oom | >600 | oom | >600 | >600 | 5.08 | >600 | oom | 10.5 | 58.8 | 55.1 |  |  |  |  |  |  |  |  | 0.186 | 0.599 | wrong | alea | 1.1e-16 / 2.1e-16 |  |  |  |  |  |  | 1.1e-16 / 2.1e-16 |  |  | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.1e-16 | 1.1e-16 / 2.1e-16 |  |  |  |  |  |  |  |  | 1.1e-16 / 2.1e-16 |
| wmi_star_n256 | 2056 | 22797 | 1.6 | - | - | - | - | - | - | >600 | - | - | - | - | - |  |  |  |  |  |  |  |  | oom | oom | 162 | alea | 2.2e-16 / 4.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| wmi_star_n32 | 264 | 2861 | 0.177 | - | - | - | - | - | - | 18.1 | - | - | 32.4 | 456 | 458 |  |  |  |  |  |  |  |  | 0.675 | 1.13 | wrong | alea | 3.3e-16 / 6.4e-16 |  |  |  |  |  |  | 2.2e-16 / 4.3e-16 |  |  | 0.0e+00 / 0.0e+00 | 3.3e-14 / 6.3e-14 | 3.3e-14 / 6.3e-14 |  |  |  |  |  |  |  |  | 5.6e-16 / 1.1e-15 |
| wmi_star_n64 | 520 | 5709 | 0.357 | - | - | - | - | - | - | 152 | - | - | 541 | >600 | >600 |  |  |  |  |  |  |  |  | 2.48 | 3.02 | 37.8 | alea | 1.1e-16 / 2.2e-16 |  |  |  |  |  |  | 2.2e-16 / 4.4e-16 |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  | 1.4e-15 / 2.8e-15 |
| zeroone_w1_w10 | 32 | 2511 | 0.027 | >600 | 22.3 | >600 | oom | >600 | 1.47 | 14.2 | 31.8 | 45.2 | 29.1 | 68.2 | 156 |  |  |  |  |  |  |  |  | 0.252 | 3.58 | 2.96 | alea | 7.8e-16 / 1.5e-15 |  | 4.0e-07 / 7.9e-07 |  |  |  | 6.2e-13 / 1.5e-12 | 7.8e-16 / 1.5e-15 | 8.9e-16 / 1.8e-15 | 7.8e-16 / 1.5e-15 | 8.9e-16 / 1.8e-15 | 7.8e-16 / 1.5e-15 | 7.8e-16 / 1.5e-15 |  |  |  |  |  |  |  |  | 1.2e-13 / 2.4e-13 |
| zeroone_w1_w12 | 36 | 4184 | 0.088 | - | 347 | - | - | - | 2.8 | 38.6 | >600 | >600 | 226 | >600 | >600 |  |  |  |  |  |  |  |  | 0.433 | 3.75 | 194 | alea | 7.8e-16 / 1.5e-15 |  | 7.8e-07 / 1.5e-06 |  |  |  | 2.9e-13 / 5.8e-13 | 4.4e-16 / 8.8e-16 |  |  | 6.7e-16 / 1.3e-15 |  |  |  |  |  |  |  |  |  |  | 1.0e-13 / 2.1e-13 |
| zeroone_w1_w14 | 40 | 6254 | 0.131 | - | oom | - | - | - | 5.59 | 88.9 | - | - | >600 | - | - |  |  |  |  |  |  |  |  | 2.12 | 7.09 | >600 | alea | 1.2e-15 / 2.4e-15 |  |  |  |  |  | 2.3e-13 / 4.6e-13 | 1.1e-15 / 2.2e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.9e-13 / 3.7e-13 |
| zeroone_w1_w16 | 44 | 8780 | 0.321 | - | - | - | - | - | 14.5 | 283 | - | - | - | - | - |  |  |  |  |  |  |  |  | 9.3 | 15.6 | - | alea | 3.9e-15 / 7.7e-15 |  |  |  |  |  | 3.2e-13 / 6.5e-13 | 1.0e-15 / 2.0e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.2e-13 / 2.3e-13 |
| zeroone_w1_w18 | 48 | 11762 | 0.748 | - | - | - | - | - | 40.2 | >600 | - | - | - | - | - |  |  |  |  |  |  |  |  | 47.4 | 56.4 | - | alea | 1.6e-15 / 3.1e-15 |  |  |  |  |  | 6.4e-13 / 1.3e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.5e-13 / 2.9e-13 |
| zeroone_w1_w20 | 52 | 15200 | 1.28 | - | - | - | - | - | 103 | - | - | - | - | - | - |  |  |  |  |  |  |  |  | 199 | 213 | - | alea | 5.1e-15 / 1.0e-14 |  |  |  |  |  | 4.3e-13 / 8.5e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.0e-13 / 2.0e-13 |
| zeroone_w1_w22 | 56 | 19094 | 2.08 | - | - | - | - | - | 214 | - | - | - | - | - | - |  |  |  |  |  |  |  |  | oom | oom | - | alea | 6.2e-15 / 1.2e-14 |  |  |  |  |  | 3.6e-13 / 7.1e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| zeroone_w2_w13 | 38 | 5230 | 0.117 | >600 | oom | oom | oom | >600 | 3.86 | 79.2 | >600 | >600 | >600 | >600 | >600 |  |  |  |  |  |  |  |  | 1.42 | 7.2 | >600 | alea | 1.7e-15 / 3.2e-15 |  |  |  |  |  | 5.0e-13 / 2.3e-12 | 7.8e-16 / 1.1e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  | 8.1e-14 / 2.7e-13 |
| zeroone_w2_w15 | 42 | 7560 | 0.237 | - | - | - | - | - | 9.26 | 569 | - | - | - | - | - |  |  |  |  |  |  |  |  | 6.02 | 13.1 | - | alea | 1.0e-14 / 1.4e-14 |  |  |  |  |  | 4.7e-13 / 1.5e-12 | 1.3e-15 / 2.7e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.7e-13 / 6.9e-13 |
| zeroone_w2_w17 | 46 | 10346 | 0.473 | - | - | - | - | - | 26.5 | >600 | - | - | - | - | - |  |  |  |  |  |  |  |  | 27.8 | 37.2 | - | alea | 2.9e-14 / 5.5e-14 |  |  |  |  |  | 4.1e-13 / 2.2e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.5e-13 / 3.0e-13 |
| zeroone_w2_w19 | 50 | 13588 | 1.28 | - | - | - | - | - | 97.5 | - | - | - | - | - | - |  |  |  |  |  |  |  |  | 116 | 130 | - | alea | 1.1e-13 / 1.6e-13 |  |  |  |  |  | 5.8e-13 / 1.7e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.3e-13 / 2.5e-13 |
| zeroone_w2_w21 | 54 | 17286 | 1.56 | - | - | - | - | - | 181 | - | - | - | - | - | - |  |  |  |  |  |  |  |  | >600 | >600 | - | alea | 3.7e-15 / 7.3e-15 |  |  |  |  |  | 2.4e-13 / 9.5e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| zeroone_w2_w23 | 58 | 21440 | 2.53 | - | - | - | - | - | 476 | - | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | - | alea | 1.6e-14 / 3.1e-14 |  |  |  |  |  | 3.2e-13 / 6.4e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| zeroone_w2_w25 | 62 | 26050 | 6.76 | - | - | - | - | - | >600 | - | - | - | - | - | - |  |  |  |  |  |  |  |  | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |

Seconds per system, the median over the repetitions; a failure is `>600`
(timeout), `oom`, `err` or `n/a`; each system's last columns are its
largest absolute and relative difference from the reference named in
`ref`.

### On Roulette's programs

| family | instances | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Merlin | pyAgrum | pgmpy | libDAI | TensorInference.jl greedy | TensorInference.jl treesa |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| bn | 9 | 9 | 6 of 6 | 5 of 5 | 3 of 4 | 6 of 6 | 3 of 4 | 5 of 6 | 3 of 4 | 6 of 6 | 9 of 9 | 9 of 9 | 9 of 9 | 9 of 9 | 9 of 9 | 9 of 9 | 9 of 9 | 9 of 9 | 9 of 9 | 8 of 9 | 9 of 9 | 8 of 9 | 9 of 9 |
| rl_caesar | 5 | 5 | 0 of 1 | 2 of 3 | 1 of 2 | 4 of 4 | 0 of 1 | 0 of 1 |  | 3 of 4 | 4 of 5 | 4 of 5 | 4 of 5 | 4 of 5 | 4 of 5 |  |  |  |  |  |  |  |  |
| rl_diamond | 5 | 5 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 3 of 4 | 4 of 5 | 0 of 1 | 2 of 3 | 2 of 3 | 2 of 3 |  |  |  |  |  |  |  |  |
| rl_hardware | 7 | 7 | 6 of 6 | 1 of 2 | 0 of 1 | 1 of 2 | 0 of 1 | 3 of 4 | 2 of 3 | 4 of 5 | 6 of 7 | 3 of 4 | 5 of 6 | 6 of 7 | 6 of 7 |  |  |  |  |  |  |  |  |
| rl_hmm_filtering_joint | 4 | 4 | 0 of 1 | 0 of 4 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  | 2 of 4 | 0 of 1 | 0 of 1 | 2 of 3 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  |
| rl_hmm_filtering_marginal | 4 | 4 | 0 of 1 | 2 of 4 | 0 of 1 | 0 of 1 | 2 of 3 | 0 of 1 |  | 2 of 4 | 2 of 3 | 3 of 4 | 2 of 3 | 2 of 3 | 2 of 3 |  |  |  |  |  |  |  |  |
| rl_hmm_marginal_likelihood | 4 | 4 | 0 of 1 | 0 of 4 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  | 4 of 4 | 0 of 1 | 0 of 1 | 2 of 3 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  |
| rl_hmm_smoothing_joint | 4 | 4 | 0 of 1 | 0 of 4 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  | 2 of 4 | 0 of 1 | 0 of 1 | 2 of 3 | 0 of 1 | 0 of 1 |  |  |  |  |  |  |  |  |
| rl_hmm_smoothing_marginal | 4 | 4 | 0 of 1 | 2 of 4 | 0 of 1 | 0 of 1 | 2 of 3 | 0 of 1 |  | 2 of 4 | 2 of 3 | 3 of 4 | 2 of 3 | 2 of 3 | 2 of 3 |  |  |  |  |  |  |  |  |
| rl_ladder | 4 | 4 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 3 of 4 | 1 of 2 | 1 of 2 | 2 of 3 | 2 of 3 |  |  |  |  |  |  |  |  |
| rl_motivating | 5 | 5 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 1 of 2 | 1 of 2 | 0 of 1 | 2 of 3 | 4 of 5 | 2 of 3 | 2 of 3 | 2 of 3 | 2 of 3 |  |  |  |  |  |  |  |  |
| rl_network | 7 | 7 | 4 of 7 | 0 of 1 | 0 of 1 | 4 of 5 | 0 of 1 | 3 of 4 | 2 of 3 | 4 of 5 | 6 of 7 | 5 of 6 | 5 of 6 | 5 of 6 | 5 of 6 |  |  |  |  |  |  |  |  |

**Geni.** Geni ran 28 instances and answered 16 (1 fail, 1 run out of
memory, 8 time out, 2 answer wrongly). Its answers differ from the
reference's by at most 2.7e+01 relative (3.7e-07 absolute, network_10)
over 16 instances. Geni's answer is wrong, zero or underflowed on
network_1000 and network_3000. Over the 27 instances given to both that
either answers, a run that does not answer counted at the 600 s limit,
Geni takes 7435 s in all and tddlog 16 s; over the 15 both answer, 235 s
against tddlog's 6.68. tddlog alone answers caesar_25, diamond_1000,
hmm_filtering_joint_100, hmm_filtering_marginal_100,
hmm_marginal_likelihood_100 and 7 more.

**noDice.** noDice ran 35 instances and answered 13 (5 run out of
memory, 5 time out, 1 underflow, 4 answer wrongly, 7 print zeros). Its
answers differ from the reference's by at most 1.0e+00 relative (6.6e-07
absolute, caesar_25) over 13 instances. noDice's answer is wrong, zero
or underflowed on hmm_filtering_joint_100, hmm_filtering_joint_1000,
hmm_filtering_joint_300, hmm_filtering_marginal_1000,
hmm_marginal_likelihood_100 and 7 more. Over the 35 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, noDice takes 13443 s in all and tddlog 5.12 s; over the 13 both
answer, 243 s against tddlog's 0.7. tddlog alone answers caesar_100,
diamond_1000, hardware_30, hmm_filtering_joint_100,
hmm_filtering_joint_1000 and 17 more.

**Psi.** Psi ran 16 instances and answered 4 (8 fail, 2 run out of
memory, 2 time out). Its answers differ from the reference's by at most
1.0e-13 relative (4.8e-15 absolute, caesar_25) over 4 instances. Over
the 16 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Psi takes 7636 s in all and tddlog
0.98 s; over the 4 both answer, 436 s against tddlog's 0.14. tddlog
alone answers bn_insurance, caesar_50, diamond_1000, hardware_10,
hmm_filtering_joint_100 and 7 more.

**Genfer.** Genfer ran 25 instances and answered 15 (2 fail, 3 run out
of memory, 5 time out). Its answers differ from the reference's by at
most 1.0e+00 relative (5.3e-26 absolute, caesar_100) over 15 instances.
Over the 24 instances given to both that either answers, a run that does
not answer counted at the 600 s limit, Genfer takes 6167 s in all and
tddlog 3.29 s; over the 14 both answer, 167 s against tddlog's 2.22.
tddlog alone answers diamond_1000, hardware_30, hmm_filtering_joint_100,
hmm_filtering_marginal_100, hmm_marginal_likelihood_100 and 5 more.

**SPPL.** SPPL ran 20 instances and answered 8 (12 time out). Its
answers differ from the reference's by at most 1.8e-14 relative (8.0e-15
absolute, hmm_smoothing_marginal_300) over 8 instances. Over the 20
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, SPPL takes 8117 s in all and tddlog 1.4 s;
over the 8 both answer, 917 s against tddlog's 0.3. tddlog alone answers
bn_insurance, caesar_25, diamond_1000, hardware_10,
hmm_filtering_joint_100 and 7 more.

**Pluck.** Pluck ran 24 instances and answered 12 (12 run out of
memory). Its answers differ from the reference's by at most 2.7e-11
relative (1.5e-11 absolute, bn_insurance) over 12 instances. Over the 23
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Pluck takes 6768 s in all and tddlog 1.77 s;
over the 12 both answer, 168 s against tddlog's 0.61. tddlog alone
answers caesar_25, diamond_1000, hardware_300, hmm_filtering_joint_100,
hmm_filtering_marginal_100 and 6 more.

**PITA.** PITA ran 13 instances and answered 7 (6 time out). Its answers
differ from the reference's by at most 1.0e+00 relative (2.2e-16
absolute, hardware_30) over 7 instances. Over the 13 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, PITA takes 3689 s in all and tddlog 0.76 s; over the 7 both
answer, 88.7 s against tddlog's 0.3. tddlog alone answers bn_insurance,
diamond_1000, hardware_100, ladder_1000, motivating_1000 and 1 more.

**aspmc.** aspmc ran 49 instances and answered 35 (10 time out, 4
underflow). Its answers differ from the reference's by at most 1.0e+00
relative (2.2e-16 absolute, caesar_100) over 35 instances. aspmc's
answer is wrong, zero or underflowed on hmm_filtering_joint_1000,
hmm_filtering_marginal_1000, hmm_smoothing_joint_1000 and
hmm_smoothing_marginal_1000. Over the 48 instances given to both that
either answers, a run that does not answer counted at the 600 s limit,
aspmc takes 10803 s in all and tddlog 12.7 s; over the 34 both answer,
2403 s against tddlog's 4.02. tddlog alone answers caesar_200,
diamond_30000, hardware_1000, hmm_filtering_joint_1000,
hmm_filtering_joint_3000 and 9 more.

**Storm sparse.** Storm sparse ran 51 instances and answered 40 (3 fail,
8 time out). Its answers differ from the reference's by at most 1.0e+00
relative (1.1e-16 absolute, caesar_100) over 40 instances. Over the 51
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Storm sparse takes 8309 s in all and tddlog
40 s; over the 40 both answer, 1709 s against tddlog's 11.7. tddlog
alone answers caesar_300, diamond_100000, hardware_10000,
hmm_filtering_joint_100, hmm_filtering_marginal_1000 and 6 more.

**Storm exact.** Storm exact ran 41 instances and answered 30 (7 fail, 4
time out). Its answers differ from the reference's by at most 2.7e-11
relative (1.5e-11 absolute, bn_insurance) over 30 instances. Over the 41
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Storm exact takes 6802 s in all and tddlog
5.97 s; over the 30 both answer, 202 s against tddlog's 2.36. tddlog
alone answers caesar_300, diamond_1000, hardware_300,
hmm_filtering_joint_100, hmm_filtering_marginal_3000 and 6 more.

**Storm dd.** Storm dd ran 49 instances and answered 38 (3 fail, 8 time
out). Its answers differ from the reference's by at most 1.0e+00
relative (1.1e-16 absolute, caesar_100) over 38 instances. Over the 49
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Storm dd takes 9049 s in all and tddlog 9.97
s; over the 38 both answer, 2449 s against tddlog's 3.59. tddlog alone
answers caesar_300, diamond_10000, hardware_3000,
hmm_filtering_joint_1000, hmm_filtering_marginal_1000 and 6 more.

**PRISM sparse.** PRISM sparse ran 45 instances and answered 34 (6 fail,
5 time out). Its answers differ from the reference's by at most 1.0e+00
relative (1.1e-16 absolute, caesar_100) over 34 instances. Over the 45
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, PRISM sparse takes 8105 s in all and tddlog
20.9 s; over the 34 both answer, 1505 s against tddlog's 6.5. tddlog
alone answers caesar_300, diamond_10000, hardware_10000,
hmm_filtering_joint_100, hmm_filtering_marginal_1000 and 6 more.

**PRISM hybrid.** PRISM hybrid ran 45 instances and answered 34 (6 fail,
5 time out). Its answers differ from the reference's by at most 1.0e+00
relative (1.1e-16 absolute, caesar_100) over 34 instances. Over the 45
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, PRISM hybrid takes 8104 s in all and tddlog
20.9 s; over the 34 both answer, 1504 s against tddlog's 6.5. tddlog
alone answers caesar_300, diamond_10000, hardware_10000,
hmm_filtering_joint_100, hmm_filtering_marginal_1000 and 6 more.

**Ace.** Ace ran 9 instances and answered 9. Its answers differ from the
reference's by at most 2.7e-11 relative (1.5e-11 absolute, bn_insurance)
over 9 instances. Over the 9 instances given to both that either
answers, Ace takes 22.6 s in all and tddlog 0.18 s.

**Ace c2d.** Ace c2d ran 9 instances and answered 9. Its answers differ
from the reference's by at most 2.7e-11 relative (1.5e-11 absolute,
bn_insurance) over 9 instances. Over the 9 instances given to both that
either answers, Ace c2d takes 118 s in all and tddlog 0.18 s.

**Merlin.** Merlin ran 9 instances and answered 9. Its answers differ
from the reference's by at most 2.0e-03 relative (4.9e-07 absolute,
bn_munin) over 9 instances. Over the 9 instances given to both that
either answers, Merlin takes 3.31 s in all and tddlog 0.18 s.

**pyAgrum.** pyAgrum ran 9 instances and answered 9. Its answers differ
from the reference's by at most 2.4e-11 relative (1.3e-11 absolute,
bn_insurance) over 9 instances. Over the 9 instances given to both that
either answers, pyAgrum takes 1.45 s in all and tddlog 0.18 s.

**pgmpy.** pgmpy ran 9 instances and answered 8 (1 time out). Its
answers differ from the reference's by at most 2.4e-11 relative (1.3e-11
absolute, bn_insurance) over 8 instances. Over the 9 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, pgmpy takes 614 s in all and tddlog 0.18 s; over the 8 both
answer, 14.4 s against tddlog's 0.12. tddlog alone answers bn_munin.

**libDAI.** libDAI ran 9 instances and answered 9. Its answers differ
from the reference's by at most 2.4e-11 relative (1.3e-11 absolute,
bn_insurance) over 9 instances. Over the 9 instances given to both that
either answers, libDAI takes 4.05 s in all and tddlog 0.18 s.

**TensorInference.jl greedy.** TensorInference.jl greedy ran 9 instances
and answered 8 (1 run out of memory). Its answers differ from the
reference's by at most 2.4e-11 relative (1.3e-11 absolute, bn_insurance)
over 8 instances. Over the 9 instances given to both that either
answers, a run that does not answer counted at the 600 s limit,
TensorInference.jl greedy takes 865 s in all and tddlog 0.18 s; over the
8 both answer, 265 s against tddlog's 0.12. tddlog alone answers
bn_munin.

**TensorInference.jl treesa.** TensorInference.jl treesa ran 9 instances
and answered 9. Its answers differ from the reference's by at most
2.4e-11 relative (1.3e-11 absolute, bn_insurance) over 9 instances. Over
the 9 instances given to both that either answers, TensorInference.jl
treesa takes 568 s in all and tddlog 0.18 s.

| instance | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Merlin | pyAgrum | pgmpy | libDAI | TI greedy | TI treesa | ProbLog | ref | tddlog diff | Geni diff | noDice diff | Psi diff | Genfer diff | SPPL diff | Pluck diff | PITA diff | aspmc diff | Storm sparse diff | Storm exact diff | Storm dd diff | PRISM sparse diff | PRISM hybrid diff | Ace diff | Ace c2d diff | Merlin diff | pyAgrum diff | pgmpy diff | libDAI diff | TI greedy diff | TI treesa diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| bn_alarm | 0.005 | 0.029 | 0.079 | 227 | 0.029 | 1.66 | 10.1 | 0.628 | 163 | 0.155 | 0.159 | 0.414 | 1.28 | 1.27 | 1.3 | 1.41 | 0.007 | 0.129 | 1.79 | 0.006 | 27 | 32.9 | 0.481 | exact | 6.7e-16 / 2.5e-14 | 1.0e-07 / 2.0e-06 | 5.5e-07 / 2.0e-05 | 2.8e-17 / 1.3e-16 | 0.0e+00 / 0.0e+00 | 3.9e-16 / 1.5e-15 | 5.6e-17 / 2.6e-16 | 1.1e-16 / 5.3e-16 | 5.6e-16 / 1.7e-15 | 1.1e-16 / 3.8e-16 | 2.8e-17 / 1.3e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.5e-16 | 2.8e-17 / 1.3e-16 | 4.5e-07 / 1.7e-05 | 2.8e-17 / 1.3e-16 | 2.8e-17 / 1.3e-16 | 2.8e-17 / 1.3e-16 | 5.6e-17 / 2.2e-16 | 1.1e-16 / 3.8e-16 |
| bn_cancer | 0.003 | 0.014 | 0.039 | 0.515 | 0.007 | 0.866 | 7.27 | 0.526 | 2.69 | 0.147 | 0.148 | 0.195 | 1.27 | 1.27 | 1.45 | 1.25 | 0.004 | 0.13 | 1.94 | 0.004 | 18.8 | 21 | 0.255 | exact | 1.1e-16 / 5.3e-16 | 2.8e-17 / 1.3e-16 | 2.8e-17 / 1.3e-16 | 2.8e-17 / 1.3e-16 | 1.1e-16 / 1.4e-16 | 2.8e-17 / 1.3e-16 | 5.6e-17 / 2.7e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.4e-16 | 2.2e-16 / 2.8e-16 | 2.8e-17 / 1.3e-16 | 1.1e-16 / 1.4e-16 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.4e-16 | 1.1e-16 / 1.4e-16 | 2.8e-17 / 1.3e-16 | 2.8e-17 / 1.3e-16 | 2.8e-17 / 1.3e-16 | 2.8e-17 / 1.3e-16 | 2.8e-17 / 1.3e-16 | 2.8e-17 / 1.3e-16 |
| bn_hailfinder | 0.026 | 15.5 |  | - | 241 | - | oom |  | 273 | 16.4 | 16.7 | 89.1 | 22.1 | 25.9 | 1.62 | 2.35 | 0.009 | 0.164 | 1.87 | 0.009 | 30 | 36.4 |  | exact | 3.9e-16 / 1.3e-15 | 7.0e-08 / 2.8e-07 |  |  | 1.1e-16 / 2.5e-16 |  |  |  | 2.8e-15 / 6.7e-15 | 5.0e-16 / 1.1e-15 | 5.6e-17 / 1.8e-16 | 5.6e-17 / 2.2e-16 | 1.1e-16 / 2.5e-16 | 1.1e-16 / 2.5e-16 | 5.6e-17 / 2.2e-16 | 5.6e-17 / 1.3e-16 | 4.8e-07 / 1.1e-06 | 4.9e-13 / 1.6e-12 | 4.9e-13 / 1.6e-12 | 4.9e-13 / 1.6e-12 | 5.6e-17 / 1.8e-16 | 5.6e-17 / 1.8e-16 |
| bn_hepar2 | 0.009 | 0.076 | 0.227 | - | 0.109 | - | 9.68 | - | 200 | 0.282 | 0.296 | 2.61 | 1.3 | 1.36 | 1.59 | 2.16 | 0.009 | 0.136 | 1.69 | 0.009 | 34.6 | 44.7 | - | exact | 1.7e-16 / 3.9e-16 | 1.2e-07 / 2.8e-07 | 1.7e-08 / 4.1e-08 |  | 0.0e+00 / 0.0e+00 |  | 1.1e-16 / 1.9e-16 |  | 3.4e-15 / 6.0e-15 | 7.8e-16 / 1.4e-15 | 0.0e+00 / 0.0e+00 | 2.2e-16 / 3.9e-16 | 1.1e-16 / 2.6e-16 | 1.1e-16 / 2.6e-16 | 1.1e-16 / 1.9e-16 | 2.2e-16 / 3.9e-16 | 1.7e-08 / 4.1e-08 | 4.7e-13 / 1.1e-12 | 4.7e-13 / 1.1e-12 | 4.7e-13 / 1.1e-12 | 1.1e-16 / 1.9e-16 | 1.1e-16 / 1.9e-16 |
| bn_insurance | 0.014 | 14.4 | 10.1 | oom | 16.4 | >600 | 36.9 | >600 | 205 | 27.2 | 28.2 | 25.8 | 7.01 | 13.7 | 1.64 | 1.97 | 0.01 | 0.134 | 1.73 | 0.008 | 30.8 | 35 | >600 | exact | 1.5e-11 / 2.7e-11 | 9.4e-08 / 4.8e-06 | 7.1e-07 / 2.9e-05 |  | 1.5e-11 / 2.7e-11 |  | 1.5e-11 / 2.7e-11 |  | 5.1e-08 / 1.3e-07 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 4.8e-07 / 2.9e-05 | 1.3e-11 / 2.4e-11 | 1.3e-11 / 2.4e-11 | 1.3e-11 / 2.4e-11 | 1.3e-11 / 2.4e-11 | 1.3e-11 / 2.4e-11 |
| bn_munin | 0.035 |  |  |  |  |  |  |  |  | 0.346 | 0.377 | 3.93 | 1.62 | 1.85 | 8.23 | 90 | 2.71 | 0.319 | >600 | 3.53 | oom | 210 |  | exact | 4.5e-16 / 1.9e-13 |  |  |  |  |  |  |  |  | 1.7e-16 / 1.8e-14 | 6.9e-18 / 1.7e-14 | 5.6e-17 / 1.7e-14 | 2.8e-17 / 1.7e-14 | 2.8e-17 / 1.7e-14 | 1.1e-16 / 1.4e-14 | 6.9e-17 / 1.3e-14 | 4.9e-07 / 2.0e-03 | 4.3e-13 / 2.1e-12 |  | 4.3e-13 / 2.1e-12 |  | 5.6e-17 / 1.6e-14 |
| bn_pigs | 0.006 |  |  |  |  |  |  |  |  | 0.148 | 0.15 | 0.17 | 1.27 | 1.28 | 3.67 | 14.6 | 0.157 | 0.171 | 1.92 | 0.15 | 64.5 | 120 |  | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| bn_survey | 0.004 | 0.016 | 0.043 | 1.72 | 0.007 | 0.928 | 7.72 | 0.492 | 2.77 | 0.152 | 0.151 | 0.261 | 1.27 | 1.27 | 1.1 | 1.11 | 0.005 | 0.128 | 1.74 | 0.004 | 21.8 | 23.8 | 0.143 | exact | 2.5e-16 / 1.6e-15 | 3.2e-08 / 7.8e-08 | 7.5e-07 / 2.7e-06 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.0e-16 | 1.1e-16 / 3.5e-16 | 1.1e-16 / 4.0e-16 | 1.4e-16 / 8.8e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 3.5e-16 | 0.0e+00 / 0.0e+00 | 2.8e-17 / 1.8e-16 | 2.8e-17 / 1.8e-16 | 2.8e-17 / 1.8e-16 | 2.8e-17 / 1.8e-16 | 2.8e-17 / 1.8e-16 | 2.5e-07 / 1.4e-06 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 5.6e-17 / 2.0e-16 | 5.6e-17 / 2.0e-16 |
| bn_water | 0.014 |  |  |  |  |  |  |  |  | 7.78 | 7.86 | 107 | 27.4 | 27.7 | 2 | 2.73 | 0.399 | 0.137 | 1.75 | 0.329 | 38 | 44.7 |  | exact | 2.8e-16 / 2.7e-14 |  |  |  |  |  |  |  |  | 3.3e-16 / 6.1e-16 | 2.8e-17 / 3.4e-16 | 3.5e-18 / 2.2e-16 | 3.3e-16 / 4.1e-16 | 3.3e-16 / 4.1e-16 | 1.1e-16 / 1.9e-16 | 2.2e-16 / 3.9e-16 | 4.5e-07 / 5.4e-05 | 4.6e-13 / 1.4e-12 | 4.6e-13 / 1.4e-12 | 4.6e-13 / 1.4e-12 | 6.9e-18 / 2.4e-16 | 2.8e-17 / 2.2e-16 |
| caesar_25 | 0.073 | err | 39.4 | 206 | 9.1 | >600 | oom |  | 16.2 | 0.327 | 0.529 | 2.73 | 1.78 | 1.83 |  |  |  |  |  |  |  |  | 28.8 | exact | 4.9e-15 / 1.1e-13 |  | 6.6e-07 / 1.0e+00 | 4.8e-15 / 1.0e-13 | 5.0e-15 / 1.1e-13 |  |  |  | 5.0e-15 / 1.0e-13 | 4.1e-15 / 1.1e-13 | 4.7e-15 / 1.0e-13 | 4.6e-15 / 1.1e-13 | 4.7e-15 / 1.1e-13 | 4.7e-15 / 1.1e-13 |  |  |  |  |  |  |  |  |
| caesar_50 | 0.142 | - | 167 | >600 | 18.4 | - | - |  | 292 | 0.626 | 1.05 | 3.81 | 2.82 | 2.73 |  |  |  |  |  |  |  |  | 61.4 | exact | 2.3e-15 / 2.2e-13 |  | 4.1e-08 / 1.0e+00 |  | 2.4e-15 / 2.2e-13 |  |  |  | 2.7e-15 / 2.2e-13 | 2.2e-15 / 2.2e-13 | 2.2e-15 / 2.2e-13 | 2.2e-15 / 2.2e-13 | 2.2e-15 / 2.2e-13 | 2.2e-15 / 2.2e-13 |  |  |  |  |  |  |  |  |
| caesar_100 | 0.283 | - | >600 | - | 31.2 | - | - |  | 388 | 1.59 | 2.51 | 6.56 | 5.43 | 5.22 |  |  |  |  |  |  |  |  | 113 | exact | 5.2e-26 / 4.8e-13 |  |  |  | 5.3e-26 / 1.0e+00 |  |  |  | 2.2e-16 / 1.0e+00 | 1.1e-16 / 1.0e+00 | 5.6e-26 / 4.7e-13 | 1.1e-16 / 1.0e+00 | 1.1e-16 / 1.0e+00 | 1.1e-16 / 1.0e+00 |  |  |  |  |  |  |  |  |
| caesar_200 | 0.599 | - | - | - | 29.9 | - | - |  | >600 | 4.85 | 7.23 | 15.4 | 12.3 | 11.7 |  |  |  |  |  |  |  |  | 253 | exact | 1.4e-29 / 1.5e-12 |  |  |  | 9.6e-18 / 1.0e+00 |  |  |  |  | 9.6e-18 / 1.0e+00 | 1.4e-29 / 1.5e-12 | 9.6e-18 / 1.0e+00 | 9.6e-18 / 1.0e+00 | 9.6e-18 / 1.0e+00 |  |  |  |  |  |  |  |  |
| caesar_300 | 0.911 |  |  | - |  |  | - |  |  | err | err | err | err | err |  |  |  |  |  |  |  |  |  | exact | 3.6e-31 / 4.0e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| diamond_1000 | 0.035 | >600 | >600 | err | oom | >600 | oom | >600 | 5.47 | 0.557 | err | 45.4 | 9.89 | 9.94 |  |  |  |  |  |  |  |  | 17 | exact | 1.1e-16 / 1.2e-16 |  |  |  |  |  |  |  | 4.0e-15 / 4.2e-15 | 1.0e-13 / 1.1e-13 |  | 5.3e-15 / 5.6e-15 | 5.3e-15 / 5.6e-15 | 5.3e-15 / 5.6e-15 |  |  |  |  |  |  |  |  |
| diamond_3000 | 0.101 | - | - | - | - | - | - | - | 23.1 | 3.05 | - | 534 | 106 | 106 |  |  |  |  |  |  |  |  | oom | exact | 1.1e-15 / 1.3e-15 |  |  |  |  |  |  |  | 1.3e-14 / 1.5e-14 | 2.9e-13 / 3.3e-13 |  | 1.3e-14 / 1.5e-14 | 1.3e-14 / 1.5e-14 | 1.3e-14 / 1.5e-14 |  |  |  |  |  |  |  |  |
| diamond_10000 | 0.386 | - | - | - | - | - | - | - | 200 | 31 | - | >600 | >600 | >600 |  |  |  |  |  |  |  |  | - | exact | 7.7e-15 / 1.3e-14 |  |  |  |  |  |  |  | 2.7e-14 / 4.4e-14 | 7.4e-13 / 1.2e-12 |  |  |  |  |  |  |  |  |  |  |  |  |
| diamond_30000 | 1.25 | - | - | - | - | - | - | - | >600 | 263 | - | - | - | - |  |  |  |  |  |  |  |  | - | exact | 4.2e-14 / 1.9e-13 |  |  |  |  |  |  |  |  | 8.7e-13 / 3.9e-12 |  |  |  |  |  |  |  |  |  |  |  |  |
| diamond_100000 | 4.58 | - | - | - | - | - | - | - | - | >600 | - | - | - | - |  |  |  |  |  |  |  |  | - | exact | 9.1e-14 / 1.3e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hardware_10 | 0.011 | 0.359 | 0.232 | oom | 6.56 | >600 | 10.2 | 0.75 | 3.48 | 0.166 | 0.172 | 1.09 | 1.27 | 1.26 |  |  |  |  |  |  |  |  | err | exact | 3.3e-16 / 1.2e-14 | 1.0e-07 / 1.0e+00 | 1.0e-07 / 1.0e+00 |  | 1.1e-16 / 6.8e-16 |  | 2.2e-16 / 1.2e-15 | 1.2e-15 / 4.8e-06 | 3.3e-16 / 3.3e-16 | 5.6e-16 / 3.4e-15 | 1.1e-16 / 1.1e-15 | 2.2e-16 / 1.2e-15 | 2.2e-16 / 1.2e-15 | 2.2e-16 / 1.2e-15 |  |  |  |  |  |  |  |  |
| hardware_30 | 0.028 | 1.32 | >600 | - | >600 | - | 11.4 | 36.4 | 4.19 | 0.209 | 0.251 | 1.94 | 1.26 | 1.27 |  |  |  |  |  |  |  |  | - | exact | 2.2e-16 / 1.1e-14 | 1.0e-07 / 1.0e+00 |  |  |  |  | 1.1e-16 / 2.1e-15 | 2.2e-16 / 1.0e+00 | 1.2e-15 / 2.4e-15 | 4.4e-16 / 1.3e-14 | 2.2e-16 / 3.6e-15 | 1.1e-16 / 1.9e-15 | 1.1e-16 / 1.9e-15 | 1.1e-16 / 1.9e-15 |  |  |  |  |  |  |  |  |
| hardware_100 | 0.085 | 4.02 | - | - | - | - | 21.2 | >600 | 24.9 | 0.521 | 0.848 | 3.61 | 1.75 | 1.58 |  |  |  |  |  |  |  |  | - | exact | 1.1e-16 / 1.1e-14 | 1.0e-07 / 1.0e+00 |  |  |  |  | 1.1e-16 / 7.5e-16 |  | 6.4e-15 / 7.5e-15 | 4.4e-16 / 4.1e-14 | 2.2e-16 / 3.5e-15 | 1.1e-16 / 7.5e-16 | 1.1e-16 / 7.5e-16 | 1.1e-16 / 7.5e-16 |  |  |  |  |  |  |  |  |
| hardware_300 | 0.264 | 12.3 | - | - | - | - | oom | - | 213 | 2.76 | err | 18.1 | 5.34 | 5.2 |  |  |  |  |  |  |  |  | - | exact | 3.3e-16 / 1.1e-14 | 1.0e-07 / 8.0e-04 |  |  |  |  |  |  | 2.2e-14 / 2.2e-14 | 4.4e-16 / 1.8e-15 |  | 1.1e-16 / 5.4e-16 | 1.1e-16 / 5.4e-16 | 1.1e-16 / 5.4e-16 |  |  |  |  |  |  |  |  |
| hardware_1000 | 0.937 | 41.5 | - | - | - | - | - | - | >600 | 27.7 | - | 209 | 46 | 45.5 |  |  |  |  |  |  |  |  | - | exact | 2.2e-16 / 1.1e-14 | 1.0e-07 / 8.0e-04 |  |  |  |  |  |  |  | 4.4e-16 / 1.8e-15 |  | 1.1e-16 / 5.4e-16 | 1.1e-16 / 5.4e-16 | 1.1e-16 / 5.4e-16 |  |  |  |  |  |  |  |  |
| hardware_3000 | 3.07 | 122 | - | - | - | - | - | - | - | 315 | - | >600 | 491 | 482 |  |  |  |  |  |  |  |  | - | exact | 2.2e-16 / 1.1e-14 | 1.0e-07 / 8.0e-04 |  |  |  |  |  |  |  | 4.4e-16 / 1.8e-15 |  |  | 1.1e-16 / 5.4e-16 | 1.1e-16 / 5.4e-16 |  |  |  |  |  |  |  |  |
| hardware_10000 | 11.8 |  |  |  |  |  |  | - | - | >600 | - | - | >600 | >600 |  |  |  |  |  |  |  |  | - | exact | 1.1e-16 / 1.1e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_filtering_joint_100 | 0.013 | >600 | zero | err | >600 | >600 | oom |  | 3.39 | >600 | >600 | 5.76 | err | err |  |  |  |  |  |  |  |  | 0.635 | exact | 5.8e-32 / 1.5e-13 |  |  |  |  |  |  |  | 5.8e-32 / 1.6e-13 |  |  | 5.9e-32 / 1.6e-13 |  |  |  |  |  |  |  |  |  |  |
| hmm_filtering_joint_300 | 0.032 | - | zero | - | - | - | - |  | 8.42 | - | - | 50 | - | - |  |  |  |  |  |  |  |  | 2.05 | exact | 3.9e-69 / 2.9e-14 |  |  |  |  |  |  |  | 5.7e-69 / 4.2e-14 |  |  | 5.3e-69 / 3.8e-14 |  |  |  |  |  |  |  |  |  |  |
| hmm_filtering_joint_1000 | 0.096 | - | wrong | - | - | - | - |  | underflow | - | - | >600 | - | - |  |  |  |  |  |  |  |  | 13 | exact | 5.6e-194 / 1.4e-11 |  | 5.0e-01 / 1.2e+182 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_filtering_joint_3000 | 0.316 | - | oom | - | - | - | - |  | >600 | - | - | - | - | - |  |  |  |  |  |  |  |  | oom | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_filtering_marginal_100 | 0.025 | >600 | 0.165 | err | >600 | 11.9 | oom |  | 3.29 | 0.2 | 0.202 | 3.94 | 1.47 | 1.45 |  |  |  |  |  |  |  |  | 0.531 | exact | 6.2e-15 / 1.0e-14 |  | 1.5e-07 / 2.4e-07 |  |  | 5.9e-15 / 9.5e-15 |  |  | 5.8e-15 / 9.3e-15 | 6.0e-15 / 9.6e-15 | 5.7e-15 / 9.1e-15 | 5.9e-15 / 9.5e-15 | 5.6e-15 / 8.9e-15 | 5.6e-15 / 8.9e-15 |  |  |  |  |  |  |  |  |
| hmm_filtering_marginal_300 | 0.038 | - | 0.659 | - | - | 148 | - |  | 7.53 | 0.339 | 0.344 | 26.2 | 5.83 | 5.76 |  |  |  |  |  |  |  |  | 1.88 | exact | 6.0e-15 / 9.6e-15 |  | 1.5e-07 / 2.4e-07 |  |  | 5.7e-15 / 9.1e-15 |  |  | 5.6e-15 / 8.9e-15 | 6.0e-15 / 9.6e-15 | 5.7e-15 / 9.1e-15 | 6.6e-15 / 1.1e-14 | 6.6e-15 / 1.1e-14 | 6.6e-15 / 1.1e-14 |  |  |  |  |  |  |  |  |
| hmm_filtering_marginal_1000 | 0.103 | - | wrong | - | - | >600 | - |  | underflow | err | 1.51 | err | err | err |  |  |  |  |  |  |  |  | 12.3 | exact | 3.0e-14 / 4.9e-14 |  | 1.2e-01 / 2.0e-01 |  |  |  |  |  |  |  | 3.0e-14 / 4.8e-14 |  |  |  |  |  |  |  |  |  |  |  |
| hmm_filtering_marginal_3000 | 0.303 | - | oom | - | - | - | - |  | >600 | - | err | - | - | - |  |  |  |  |  |  |  |  | oom | exact | 1.1e-13 / 1.8e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_marginal_likelihood_100 | 0.014 | >600 | zero | err | >600 | >600 | oom |  | 3.01 | >600 | >600 | 8.64 | err | err |  |  |  |  |  |  |  |  | oom | exact | 1.6e-48 / 5.5e-14 |  |  |  |  |  |  |  | 1.8e-48 / 6.3e-14 |  |  | 1.7e-48 / 6.1e-14 |  |  |  |  |  |  |  |  |  |  |
| hmm_marginal_likelihood_300 | 0.035 | - | zero | - | - | - | - |  | 4.56 | - | - | 134 | - | - |  |  |  |  |  |  |  |  | - | exact | 2.6e-116 / 1.2e-12 |  |  |  |  |  |  |  | 2.7e-116 / 1.2e-12 |  |  | 2.7e-116 / 1.2e-12 |  |  |  |  |  |  |  |  |  |  |
| hmm_marginal_likelihood_600 | 0.067 | - | zero | - | - | - | - |  | 8.41 | - | - | >600 | - | - |  |  |  |  |  |  |  |  | - | exact | 3.2e-219 / 6.9e-12 |  |  |  |  |  |  |  | 3.2e-219 / 7.0e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_marginal_likelihood_900 | 0.099 | - | underflow | - | - | - | - |  | 14.2 | - | - | - | - | - |  |  |  |  |  |  |  |  | - | exact | 1.2e-322 / 1.3e-11 |  |  |  |  |  |  |  | 1.2e-322 / 1.3e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_smoothing_joint_100 | 0.013 | >600 | zero | err | oom | >600 | oom |  | 3.35 | >600 | >600 | 6.18 | err | err |  |  |  |  |  |  |  |  | 0.602 | exact | 1.3e-31 / 2.0e-13 |  |  |  |  |  |  |  | 1.3e-31 / 2.0e-13 |  |  | 1.3e-31 / 2.0e-13 |  |  |  |  |  |  |  |  |  |  |
| hmm_smoothing_joint_300 | 0.032 | - | zero | - | - | - | - |  | 8.42 | - | - | 58.2 | - | - |  |  |  |  |  |  |  |  | 2.4 | exact | 3.1e-67 / 1.3e-12 |  |  |  |  |  |  |  | 3.1e-67 / 1.4e-12 |  |  | 3.1e-67 / 1.3e-12 |  |  |  |  |  |  |  |  |  |  |
| hmm_smoothing_joint_1000 | 0.099 | - | wrong | - | - | - | - |  | underflow | - | - | >600 | - | - |  |  |  |  |  |  |  |  | 16.9 | exact | 2.0e-193 / 2.9e-11 |  | 5.0e-01 / 7.4e+181 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_smoothing_joint_3000 | 0.325 | - | oom | - | - | - | - |  | >600 | - | - | - | - | - |  |  |  |  |  |  |  |  | oom | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_smoothing_marginal_100 | 0.023 | >600 | 0.165 | err | >600 | 11.8 | oom |  | 3.31 | 0.211 | 0.212 | 4.01 | 1.5 | 1.49 |  |  |  |  |  |  |  |  | 0.513 | exact | 5.6e-17 / 1.2e-16 |  | 3.5e-07 / 7.8e-07 |  |  | 3.3e-16 / 7.4e-16 |  |  | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 1.1e-16 / 2.5e-16 | 1.1e-16 / 2.5e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 |  |  |  |  |  |  |  |  |
| hmm_smoothing_marginal_300 | 0.037 | - | 0.666 | - | - | 147 | - |  | 7.6 | 0.407 | 0.42 | 26.8 | 5.79 | 5.77 |  |  |  |  |  |  |  |  | 1.78 | exact | 6.3e-15 / 1.4e-14 |  | 3.5e-07 / 7.8e-07 |  |  | 8.0e-15 / 1.8e-14 |  |  | 6.8e-15 / 1.5e-14 | 6.3e-15 / 1.4e-14 | 6.3e-15 / 1.4e-14 | 6.2e-15 / 1.4e-14 | 6.2e-15 / 1.4e-14 | 6.2e-15 / 1.4e-14 |  |  |  |  |  |  |  |  |
| hmm_smoothing_marginal_1000 | 0.102 | - | wrong | - | - | >600 | - |  | underflow | err | 2.27 | err | err | err |  |  |  |  |  |  |  |  | 10.9 | exact | 3.2e-14 / 7.1e-14 |  | 5.0e-02 / 1.1e-01 |  |  |  |  |  |  |  | 3.2e-14 / 7.1e-14 |  |  |  |  |  |  |  |  |  |  |  |
| hmm_smoothing_marginal_3000 | 0.302 | - | oom | - | - | - | - |  | >600 | - | err | - | - | - |  |  |  |  |  |  |  |  | oom | exact | 3.2e-14 / 7.1e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| ladder_1000 | 0.065 | >600 | >600 | err | err | >600 | oom | >600 | 241 | 2.54 | 2.63 | 164 | 35.1 | 35.2 |  |  |  |  |  |  |  |  | >600 | exact | 1.9e-15 / 6.2e-15 |  |  |  |  |  |  |  | 3.3e-16 / 1.1e-15 | 2.8e-16 / 9.2e-16 | 8.3e-16 / 2.7e-15 | 2.8e-16 / 9.2e-16 | 2.8e-16 / 9.2e-16 | 2.8e-16 / 9.2e-16 |  |  |  |  |  |  |  |  |
| ladder_3000 | 0.218 | - | - | - | - | - | - | - | >600 | 19.6 | err | >600 | 381 | 385 |  |  |  |  |  |  |  |  | - | exact | 9.2e-16 / 8.2e-15 |  |  |  |  |  |  |  |  | 2.9e-16 / 2.6e-15 |  |  | 2.9e-16 / 2.6e-15 | 2.9e-16 / 2.6e-15 |  |  |  |  |  |  |  |  |
| ladder_10000 | 0.782 | - | - | - | - | - | - | - | - | 187 | - | - | >600 | >600 |  |  |  |  |  |  |  |  | - | exact | 6.2e-15 / 1.8e-12 |  |  |  |  |  |  |  |  | 1.4e-17 / 4.3e-15 |  |  |  |  |  |  |  |  |  |  |  |  |
| ladder_30000 | 2.53 | - | - | - | - | - | - | - | - | >600 | - | - | - | - |  |  |  |  |  |  |  |  | - | exact | 1.3e-22 / 8.7e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| motivating_1000 | 0.035 | >600 | 24.3 | >600 | oom | 594 | 9.18 | >600 | 8.88 | 0.604 | 0.646 | 46.2 | 9.9 | 9.85 |  |  |  |  |  |  |  |  | 5.51 | exact | 5.6e-16 / 2.0e-15 |  | 2.7e-07 / 1.0e-06 |  |  | 5.6e-17 / 2.0e-16 | 2.2e-16 / 8.1e-16 |  | 3.9e-16 / 1.4e-15 | 3.9e-16 / 1.4e-15 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |
| motivating_3000 | 0.097 | - | oom | - | - | >600 | oom | - | 85.4 | 3.96 | 4.05 | 536 | 106 | 106 |  |  |  |  |  |  |  |  | oom | exact | 5.6e-16 / 2.0e-15 |  |  |  |  |  |  |  | 3.9e-16 / 1.4e-15 | 3.9e-16 / 1.4e-15 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |
| motivating_10000 | 0.363 | - | - | - | - | - | - | - | >600 | 40 | err | >600 | >600 | >600 |  |  |  |  |  |  |  |  | - | exact | 5.6e-16 / 2.0e-15 |  |  |  |  |  |  |  |  | 3.9e-16 / 1.4e-15 |  |  |  |  |  |  |  |  |  |  |  |  |
| motivating_30000 | 1.23 | - | - | - | - | - | - | - | - | 348 | - | - | - | - |  |  |  |  |  |  |  |  | - | exact | 5.6e-16 / 2.0e-15 |  |  |  |  |  |  |  |  | 3.9e-16 / 1.4e-15 |  |  |  |  |  |  |  |  |  |  |  |  |
| motivating_100000 | 4.49 | - | - | - | - | - | - | - | - | >600 | - | - | - | - |  |  |  |  |  |  |  |  | - | exact | 5.6e-16 / 2.0e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| network_10 | 0.007 | 0.774 | >600 | err | 1.9 | >600 | 10 | 1.66 | 3.15 | 0.178 | 0.213 | 1.9 | 1.27 | 1.27 |  |  |  |  |  |  |  |  | oom | exact | 3.3e-16 / 3.0e-14 | 3.7e-07 / 2.7e+01 |  |  | 1.1e-16 / 1.9e-15 |  | 3.3e-16 / 1.9e-15 | 3.3e-16 / 2.0e-08 | 3.3e-16 / 1.6e-15 | 5.6e-16 / 2.3e-15 | 2.2e-16 / 1.7e-15 | 5.6e-16 / 2.3e-15 | 5.6e-16 / 1.9e-15 | 5.6e-16 / 1.9e-15 |  |  |  |  |  |  |  |  |
| network_30 | 0.015 | 2.59 | - | - | 4.39 | - | 11.4 | 48.3 | 5.3 | 0.246 | 0.413 | 2.77 | 1.54 | 1.44 |  |  |  |  |  |  |  |  | - | exact | 2.2e-16 / 3.8e-15 | 2.4e-07 / 1.9e-01 |  |  | 4.4e-16 / 1.4e-15 |  | 2.2e-16 / 8.0e-16 | 3.6e-16 / 1.1e-10 | 1.1e-16 / 8.6e-16 | 1.1e-16 / 1.4e-15 | 3.3e-16 / 4.5e-16 | 1.1e-16 / 1.4e-15 | 1.1e-16 / 1.0e-15 | 1.1e-16 / 1.0e-15 |  |  |  |  |  |  |  |  |
| network_100 | 0.041 | 9.14 | - | - | 13 | - | 23.4 | >600 | 20.1 | 0.64 | 1.67 | 5.93 | 3.94 | 3.35 |  |  |  |  |  |  |  |  | - | exact | 4.4e-16 / 3.1e-15 | 6.0e-07 / 1.1e-03 |  |  | 1.7e-15 / 4.6e-15 |  | 1.4e-15 / 3.7e-15 |  | 6.7e-16 / 1.7e-15 | 7.2e-16 / 2.5e-15 | 8.9e-16 / 2.4e-15 | 7.2e-16 / 2.5e-15 | 5.6e-16 / 2.0e-15 | 5.6e-16 / 2.0e-15 |  |  |  |  |  |  |  |  |
| network_300 | 0.117 | 27.1 | - | - | 36.3 | - | oom | - | 220 | 3.06 | 10.5 | 27.4 | 19.5 | 17 |  |  |  |  |  |  |  |  | - | exact | 1.7e-15 / 3.7e-15 | 4.1e-07 / 6.8e-05 |  |  | 2.0e-15 / 4.1e-15 |  |  |  | 2.6e-15 / 5.1e-15 | 1.9e-15 / 4.0e-15 | 4.2e-16 / 6.4e-15 | 1.9e-15 / 4.0e-15 | 1.9e-15 / 2.9e-15 | 1.9e-15 / 2.9e-15 |  |  |  |  |  |  |  |  |
| network_1000 | 0.399 | wrong | - | - | err | - | - | - | >600 | 28.3 | 111 | 269 | 183 | 180 |  |  |  |  |  |  |  |  | - | exact | 2.2e-16 / 2.6e-14 | 1.4e-06 / 2.4e-03 |  |  |  |  |  |  |  | 3.2e-15 / 1.3e-14 | 5.3e-18 / 9.6e-15 | 3.2e-15 / 1.3e-14 | 2.4e-15 / 1.2e-14 | 2.4e-15 / 1.2e-14 |  |  |  |  |  |  |  |  |
| network_3000 | 1.27 | wrong | - | - | - | - | - | - | - | 370 | >600 | >600 | >600 | >600 |  |  |  |  |  |  |  |  | - | exact | 1.1e-16 / 5.7e-14 | 2.7e-06 / 8.1e+01 |  |  |  |  |  |  |  | 4.0e-15 / 3.5e-14 |  |  |  |  |  |  |  |  |  |  |  |  |
| network_10000 | 4.68 | oom | - | - | - | - | - | - | - | >600 | - | - | - | - |  |  |  |  |  |  |  |  | - | exact | 1.1e-14 / 2.3e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |

Seconds per system, the median over the repetitions; a failure is `>600`
(timeout), `oom`, `err` or `n/a`; each system's last columns are its
largest absolute and relative difference from the reference named in
`ref`.

### On Geni's programs

| family | instances | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Merlin | pyAgrum | pgmpy | libDAI | TensorInference.jl greedy | TensorInference.jl treesa |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| geni_link | 10 | 6 | 7 of 9 | 6 of 8 | 0 of 2 | 1 of 3 | 1 of 3 | 2 of 4 | 2 of 4 | 6 of 8 | 6 of 8 | 5 of 7 | 5 of 7 | 7 of 9 | 7 of 9 | 5 of 6 | 5 of 6 | 5 of 6 | 5 of 6 | 6 of 7 | 5 of 6 | 5 of 6 | 8 of 9 |
| geni_router | 13 | 9 | 7 of 9 | 6 of 8 | 0 of 2 | 8 of 10 | 2 of 4 | 3 of 5 | 2 of 4 | 6 of 8 | 6 of 8 | 6 of 8 | 12 of 13 | 12 of 13 | 12 of 13 | 7 of 8 | 6 of 7 | 7 of 8 | 7 of 8 | 9 of 10 | 8 of 9 | 9 of 10 | 8 of 9 |
| geni_weather_h | 19 | 19 | 19 of 19 | 0 of 3 | 0 of 3 | 1 of 4 | 2 of 5 | 3 of 6 | 6 of 9 | 9 of 11 | 19 of 19 | 9 of 11 | 11 of 13 | 19 of 19 | 0 of 3 | 11 of 12 | 9 of 10 | 0 of 10 | 10 of 11 | 2 of 3 | 10 of 11 | 9 of 10 | 9 of 10 |
| geni_weather_n | 13 | 10 | 6 of 13 | 0 of 6 | 0 of 6 | 1 of 7 | 0 of 6 | 1 of 7 | 1 of 7 | 8 of 13 | 8 of 13 | 5 of 11 | 6 of 12 | 5 of 11 | 3 of 9 | 8 of 9 | 8 of 9 | 4 of 9 | 8 of 9 | 3 of 4 | 8 of 9 | 11 of 11 | 10 of 11 |

**Geni.** Geni ran 50 instances and answered 39 (6 time out, 5 print
zeros). Its answers differ from the reference's by at most 5.7e-01
relative (3.6e-08 absolute, weather_n11) over 39 instances. Geni's
answer is wrong, zero or underflowed on weather_n12, weather_n13,
weather_n15, weather_n16 and weather_n17. Over the 44 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, Geni takes 4627 s in all and tddlog 1024 s; over the 38 both
answer, 1133 s against tddlog's 376. Only Geni answers link_16. tddlog
alone answers router_18, weather_n12, weather_n13, weather_n15 and
weather_n16.

**noDice.** noDice ran 25 instances and answered 12 (2 fail, 4 run out
of memory, 7 time out). Its answers differ from the reference's by at
most 1.1e-06 relative (5.3e-07 absolute, link_8) over 12 instances. Over
the 19 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, noDice takes 4853 s in all and tddlog
84.6 s; over the 12 both answer, 653 s against tddlog's 24.6. tddlog
alone answers router_16, weather_h1, weather_h100, weather_h600,
weather_n15 and 2 more.

**Psi.** Psi ran 13 instances and answered 0 (9 fail, 4 time out). Over
the 8 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Psi takes 4800 s in all and tddlog
59.1 s. tddlog alone answers link_4, router_4, weather_h1, weather_h100,
weather_h600 and 3 more.

**Genfer.** Genfer ran 24 instances and answered 11 (1 fail, 9 run out
of memory, 3 time out). Its answers differ from the reference's by at
most 2.9e-14 relative (1.5e-14 absolute, router_18) over 11 instances.
Over the 19 instances given to both that either answers, a run that does
not answer counted at the 600 s limit, Genfer takes 5190 s in all and
tddlog 71.5 s; over the 11 both answer, 390 s against tddlog's 5.22.
tddlog alone answers link_6, router_20, weather_h100, weather_h2,
weather_h600 and 3 more.

**SPPL.** SPPL ran 18 instances and answered 5 (13 time out). Its
answers differ from the reference's by at most 8.2e-14 relative (7.7e-21
absolute, weather_h1) over 5 instances. Over the 13 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, SPPL takes 4993 s in all and tddlog 62 s; over the 5 both
answer, 193 s against tddlog's 0.033. tddlog alone answers link_6,
router_8, weather_h100, weather_h3, weather_h600 and 3 more.

**Pluck.** Pluck ran 22 instances and answered 9 (13 run out of memory).
Its answers differ from the reference's by at most 5.0e-16 relative
(7.9e-23 absolute, weather_h3) over 9 instances. Over the 17 instances
given to both that either answers, a run that does not answer counted at
the 600 s limit, Pluck takes 4856 s in all and tddlog 62.2 s; over the 9
both answer, 55.8 s against tddlog's 0.098. tddlog alone answers link_8,
router_10, weather_h100, weather_h4, weather_h600 and 3 more.

**PITA.** PITA ran 24 instances and answered 11 (13 time out). Its
answers differ from the reference's by at most 6.7e-10 relative (8.3e-17
absolute, weather_h2) over 11 instances. Over the 19 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, PITA takes 4994 s in all and tddlog 66.1 s; over the 11 both
answer, 194 s against tddlog's 0.53. tddlog alone answers link_8,
router_8, weather_h100, weather_h600, weather_h7 and 3 more.

**aspmc.** aspmc ran 40 instances and answered 29 (11 time out). Its
answers differ from the reference's by at most 1.9e-15 relative (3.1e-24
absolute, weather_n13) over 29 instances. Over the 35 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, aspmc takes 5116 s in all and tddlog 306 s; over the 29 both
answer, 1516 s against tddlog's 37.2. tddlog alone answers link_16,
router_16, weather_h100, weather_h600, weather_n15 and 1 more.

**Storm sparse.** Storm sparse ran 48 instances and answered 39 (1 run
out of memory, 8 time out). Its answers differ from the reference's by
at most 4.1e-13 relative (1.4e-17 absolute, weather_h1000) over 39
instances. Over the 43 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, Storm sparse takes
3039 s in all and tddlog 619 s; over the 39 both answer, 639 s against
tddlog's 379. Storm sparse is faster on 10 (weather_h100 2.57 s against
5.68, weather_h600 4.6 s against 13.6, weather_h200 2.83 s against 15.2
and weather_h400 3.28 s against 19, the largest differences). tddlog
alone answers link_16, router_16, weather_n15 and weather_n16.

**Storm exact.** Storm exact ran 37 instances and answered 25 (6 fail, 6
time out). Its answers differ from the reference's by at most 1.8e-15
relative (8.3e-16 absolute, link_10) over 25 instances. Over the 32
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Storm exact takes 5105 s in all and tddlog
86.7 s; over the 25 both answer, 905 s against tddlog's 4.14. tddlog
alone answers link_14, router_16, weather_h100, weather_h600,
weather_n11 and 2 more.

**Storm dd.** Storm dd ran 45 instances and answered 34 (7 fail, 4 time
out). Its answers differ from the reference's by at most 4.8e-11
relative (2.4e-11 absolute, router_24) over 32 instances. Over the 38
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Storm dd takes 5181 s in all and tddlog 731
s; over the 31 both answer, 1457 s against tddlog's 34.7. Only Storm dd
answers router_24. tddlog alone answers link_14, weather_h300,
weather_h600, weather_n12, weather_n15 and 1 more.

**PRISM sparse.** PRISM sparse ran 52 instances and answered 43 (3 fail,
5 run out of memory, 1 time out). Its answers differ from the
reference's by at most 8.7e-11 relative (3.0e-15 absolute,
weather_h1000) over 40 instances. Over the 44 instances given to both
that either answers, a run that does not answer counted at the 600 s
limit, PRISM sparse takes 3648 s in all and tddlog 1223 s; over the 39
both answer, 1220 s against tddlog's 578. PRISM sparse is faster on 8
(weather_h200 14 s against 15.2, weather_h400 13.6 s against 19,
weather_h300 13.9 s against 20.7 and weather_h700 15 s against 40.2, the
largest differences). Only PRISM sparse answers router_24. tddlog alone
answers router_20, weather_n11, weather_n15 and weather_n16.

**PRISM hybrid.** PRISM hybrid ran 34 instances and answered 22 (2 fail,
4 run out of memory, 6 time out). Its answers differ from the
reference's by at most 4.8e-11 relative (2.4e-11 absolute, router_24)
over 19 instances. Over the 26 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, PRISM
hybrid takes 5554 s in all and tddlog 892 s; over the 18 both answer,
1326 s against tddlog's 228. Only PRISM hybrid answers router_24. tddlog
alone answers router_20, weather_h1, weather_h100, weather_h600,
weather_n15 and 2 more.

**Ace.** Ace ran 35 instances and answered 31 (4 time out). Its answers
differ from the reference's by at most 3.0e-14 relative (1.5e-14
absolute, router_16) over 31 instances. Over the 35 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, Ace takes 2705 s in all and tddlog 115 s; over the 31 both
answer, 305 s against tddlog's 41.9. tddlog alone answers link_14,
router_18, weather_h300 and weather_n15.

**Ace c2d.** Ace c2d ran 32 instances and answered 28 (4 time out). Its
answers differ from the reference's by at most 2.8e-15 relative (2.8e-23
absolute, weather_n12) over 28 instances. Over the 32 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, Ace c2d takes 3791 s in all and tddlog 65.8 s; over the 28 both
answer, 1391 s against tddlog's 11.5. tddlog alone answers link_14,
router_16, weather_h100 and weather_n15.

**Merlin.** Merlin ran 33 instances and answered 16 (4 run out of
memory, 13 print zeros). Its answers differ from the reference's by at
most 1.9e-01 relative (4.8e-07 absolute, weather_n9) over 16 instances.
Merlin's answer is wrong, zero or underflowed on weather_h1, weather_h2,
weather_h3, weather_h4, weather_h5 and 8 more. Over the 33 instances
given to both that either answers, a run that does not answer counted at
the 600 s limit, Merlin takes 10237 s in all and tddlog 69.9 s; over the
16 both answer, 37.2 s against tddlog's 5.07. Merlin is faster on 2
(router_4 0.044 s against 0.22 and link_8 0.093 s against 0.64). tddlog
alone answers link_14, router_18, weather_h1, weather_h100, weather_h2
and 12 more.

**pyAgrum.** pyAgrum ran 34 instances and answered 30 (3 run out of
memory, 1 time out). Its answers differ from the reference's by at most
3.9e-12 relative (4.8e-19 absolute, weather_h2) over 30 instances. Over
the 34 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, pyAgrum takes 2603 s in all and
tddlog 93.5 s; over the 30 both answer, 203 s against tddlog's 18.3.
pyAgrum is faster on 3 (router_4 0.13 s against 0.22, weather_h3 0.21 s
against 0.38 and link_8 0.19 s against 0.64). tddlog alone answers
link_14, router_18, weather_h200 and weather_n15.

**pgmpy.** pgmpy ran 24 instances and answered 20 (2 run out of memory,
2 time out). Its answers differ from the reference's by at most 3.9e-12
relative (4.8e-19 absolute, weather_h2) over 20 instances. Over the 24
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, pgmpy takes 3481 s in all and tddlog 442 s;
over the 20 both answer, 1081 s against tddlog's 40.2. pgmpy is faster
on 1 (router_18 3.63 s against 4.08). tddlog alone answers link_16,
router_24, weather_h3 and weather_n9.

**libDAI.** libDAI ran 35 instances and answered 31 (3 run out of
memory, 1 time out). Its answers differ from the reference's by at most
3.9e-12 relative (4.8e-19 absolute, weather_h2) over 31 instances. Over
the 35 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, libDAI takes 2867 s in all and tddlog
98.7 s; over the 31 both answer, 467 s against tddlog's 22.4. libDAI is
faster on 3 (router_4 0.045 s against 0.22, weather_h3 0.18 s against
0.38 and link_8 0.1 s against 0.64). tddlog alone answers link_14,
router_20, weather_h200 and weather_n15.

**TensorInference.jl greedy.** TensorInference.jl greedy ran 37
instances and answered 34 (2 run out of memory, 1 time out). Its answers
differ from the reference's by at most 1.2e-13 relative (5.9e-14
absolute, router_20) over 34 instances. Over the 37 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, TensorInference.jl greedy takes 4785 s in all and tddlog 318 s;
over the 34 both answer, 2985 s against tddlog's 174. tddlog alone
answers link_14, router_24 and weather_h100.

**TensorInference.jl treesa.** TensorInference.jl treesa ran 39
instances and answered 35 (4 time out). Its answers differ from the
reference's by at most 3.8e-14 relative (1.9e-14 absolute, router_18)
over 35 instances. Over the 38 instances given to both that either
answers, a run that does not answer counted at the 600 s limit,
TensorInference.jl treesa takes 10667 s in all and tddlog 1096 s; over
the 34 both answer, 8327 s against tddlog's 371. Only TensorInference.jl
treesa answers link_18. tddlog alone answers router_20, weather_h100 and
weather_n17.

| instance | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Merlin | pyAgrum | pgmpy | libDAI | TI greedy | TI treesa | ProbLog | ref | tddlog diff | Geni diff | noDice diff | Psi diff | Genfer diff | SPPL diff | Pluck diff | PITA diff | aspmc diff | Storm sparse diff | Storm exact diff | Storm dd diff | PRISM sparse diff | PRISM hybrid diff | Ace diff | Ace c2d diff | Merlin diff | pyAgrum diff | pgmpy diff | libDAI diff | TI greedy diff | TI treesa diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| link_4 | 0.004 | 0.007 | 0.017 | >600 | 5.61 | 12.2 | 5.19 | 0.435 | 2.59 | 0.155 | 0.156 | 0.408 | 1.27 | 1.26 | 1.3 | 1.33 | 0.237 | 1.45 | 19 | 0.237 | 23.1 | 28.2 | 0.092 | exact | 0.0e+00 / 0.0e+00 | 6.9e-09 / 1.2e-08 | 9.3e-08 / 1.6e-07 |  | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.0e-16 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 9.3e-08 / 1.6e-07 | 4.3e-13 / 7.5e-13 | 4.3e-13 / 7.5e-13 | 4.3e-13 / 7.5e-13 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| link_6 | 0.008 | 0.031 | 0.065 | - | oom | >600 | 6.4 | 0.615 | 2.63 | 0.188 | 0.202 | 1.18 | 1.28 | 1.28 | 1.45 | 1.6 | 0.066 | 0.226 | 1.32 | 0.07 | 34.6 | 51.8 | 5.59 | exact | 1.1e-16 / 2.2e-16 | 1.3e-08 / 2.5e-08 | 3.9e-07 / 7.6e-07 |  |  |  | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 3.9e-07 / 7.6e-07 | 2.2e-13 / 4.3e-13 | 2.2e-13 / 4.3e-13 | 2.2e-13 / 4.3e-13 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 |
| link_8 | 0.022 | 0.222 | 0.504 | - | - | - | oom | >600 | 2.97 | 0.536 | 0.733 | 2.61 | 1.28 | 1.27 | 2.2 | 3.33 | 0.093 | 0.187 | 1.46 | 0.103 | 60.7 | 98.6 | >600 | exact | 1.1e-16 / 2.3e-16 | 3.0e-08 / 6.3e-08 | 5.3e-07 / 1.1e-06 |  |  |  |  |  | 1.7e-16 / 3.5e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 0.0e+00 / 0.0e+00 | 1.7e-16 / 3.5e-16 | 4.7e-07 / 9.9e-07 | 1.7e-13 / 3.6e-13 | 1.7e-13 / 3.6e-13 | 1.7e-13 / 3.6e-13 | 5.6e-17 / 1.2e-16 | 0.0e+00 / 0.0e+00 |
| link_10 | 0.122 | 1.71 | 5.33 | - | - | - | - | - | 4.87 | 4.05 | 5.62 | 6.05 | 2.09 | 2.02 | 4.5 | 29.9 | 0.396 | 0.427 | 24 | 0.413 | 72 | 142 | - | exact | 8.3e-16 / 1.8e-15 | 4.9e-09 / 1.1e-08 | 3.0e-07 / 6.7e-07 |  |  |  |  |  | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 3.0e-07 / 6.7e-07 | 2.3e-14 / 5.1e-14 | 2.3e-14 / 5.1e-14 | 2.3e-14 / 5.1e-14 | 7.8e-16 / 1.7e-15 | 7.8e-16 / 1.7e-15 |
| link_12 | 1.62 | 12.1 | 55.5 | - | - | - | - | - | 33.1 | 33.7 | 45.3 | 26.6 | 9.31 | 9.31 | 19.1 | 271 | 8.04 | 2.04 | 7.91 | 7.27 | 136 | 181 | - | exact | 6.1e-16 / 1.4e-15 | 3.1e-08 / 7.1e-08 | 1.7e-07 / 3.8e-07 |  |  |  |  |  | 5.6e-16 / 1.3e-15 | 6.1e-16 / 1.4e-15 | 5.6e-16 / 1.3e-15 | 6.1e-16 / 1.4e-15 | 6.1e-16 / 1.4e-15 | 6.1e-16 / 1.4e-15 | 5.0e-16 / 1.1e-15 | 6.1e-16 / 1.4e-15 | 1.7e-07 / 3.8e-07 | 8.3e-15 / 1.9e-14 | 8.3e-15 / 1.9e-14 | 8.3e-15 / 1.9e-14 | 6.7e-16 / 1.5e-15 | 5.6e-16 / 1.3e-15 |
| link_14 | 22.4 | 80.1 | 418 | - | - | - | - | - | 364 | 254 | err | >600 | 63.2 | 73.8 | >600 | >600 | oom | oom | 275 | oom | oom | 262 | - | exact | 8.9e-16 / 2.1e-15 | 2.3e-08 / 5.5e-08 | 7.7e-08 / 1.8e-07 |  |  |  |  |  | 7.2e-16 / 1.7e-15 | 7.2e-16 / 1.7e-15 |  |  | 7.2e-16 / 1.7e-15 | 7.2e-16 / 1.7e-15 |  |  |  |  | 2.3e-13 / 5.3e-13 |  |  | 7.8e-16 / 1.8e-15 |
| link_16 | oom | 494 | >600 | - | - | - | - | - | >600 | >600 | - | - | 441 | 529 | - | - | - | - | >600 | - | - | 347 | - | exact |  | 5.1e-09 / 1.2e-08 |  |  |  |  |  |  |  |  |  |  | 6.3e-15 / 1.5e-14 | 6.3e-15 / 1.5e-14 |  |  |  |  |  |  |  | 6.2e-15 / 1.5e-14 |
| link_18 | - | >600 | - | - | - | - | - | - | - | - | - | - | oom | oom | - | - | - | - | - | - | - | 540 | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.6e-15 / 3.9e-15 |
| link_20 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | >600 | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| link_24 | oom | >600 | oom | >600 | oom | >600 | oom | >600 | >600 | >600 | >600 | err | >600 | >600 | - | - | - | - | - | - | - | - | >600 | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_4 | 0.005 | 0.006 | 0.018 | >600 | 0.006 | 1.68 | 5.04 | 0.397 | 2.59 | 0.158 | 0.156 | 0.456 | 1.26 | 1.27 | 1.23 | 1.32 | 0.044 | 0.132 | 17.3 | 0.045 | 23.6 | 34.6 | 0.114 | exact | 0.0e+00 / 0.0e+00 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| router_6 | 0.008 | 0.012 | 0.058 | - | 0.015 | 44.6 | 6.52 | 1.4 | 2.7 | 0.177 | 0.179 | 1.08 | 1.27 | 1.27 | 1.61 | 1.7 | 0.071 | 0.268 | 1.39 | 0.081 | 37.7 | 71.9 | 11.7 | exact | 0.0e+00 / 0.0e+00 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 4.4e-16 / 8.9e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 0.0e+00 / 0.0e+00 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 |
| router_8 | 0.016 | 0.058 | 0.353 | - | 0.064 | >600 | 7.61 | >600 | 3 | 0.326 | 0.373 | 2.3 | 1.28 | 1.27 | 1.79 | 2.19 | 0.106 | 0.254 | 1.5 | 0.144 | 51.5 | 122 | >600 | exact | 0.0e+00 / 0.0e+00 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 8.9e-16 / 1.8e-15 |  | 1.1e-16 / 2.2e-16 |  | 0.0e+00 / 0.0e+00 | 4.4e-16 / 8.9e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| router_10 | 0.045 | 0.372 | 2.85 | - | 0.34 | - | oom | - | 5.14 | 1.59 | 1.99 | 4.31 | 1.66 | 1.69 | 2.4 | 4.26 | 0.169 | 0.376 | 1.52 | 0.276 | 47.5 | 159 | - | exact | 2.2e-16 / 4.4e-16 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 1.2e-15 / 2.4e-15 |  |  |  | 3.3e-16 / 6.7e-16 | 2.2e-16 / 4.4e-16 | 3.3e-16 / 6.7e-16 | 4.4e-16 / 8.9e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 2.2e-16 / 4.4e-16 | 3.3e-16 / 6.7e-16 |
| router_12 | 0.063 | 2.58 | 21 | - | 1.99 | - | - | - | 32.2 | 12 | 15 | 7.99 | 2.95 | 2.94 | 4.06 | 23.4 | 0.345 | 0.719 | 1.76 | 0.573 | 69.3 | 238 | - | exact | 6.7e-16 / 1.3e-15 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 2.4e-15 / 4.9e-15 |  |  |  | 4.4e-16 / 8.9e-16 | 8.9e-16 / 1.8e-15 | 3.3e-16 / 6.7e-16 | 2.2e-16 / 4.4e-16 | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 | 4.4e-16 / 8.9e-16 | 4.4e-16 / 8.9e-16 | 4.5e-07 / 9.0e-07 | 1.3e-13 / 2.7e-13 | 1.3e-13 / 2.7e-13 | 1.3e-13 / 2.7e-13 | 4.4e-16 / 8.9e-16 | 4.4e-16 / 8.9e-16 |
| router_14 | 0.311 | 17.1 | 149 | - | 11.3 | - | - | - | 536 | 90 | 108 | 12.6 | 7.51 | 8.37 | 9.64 | 223 | 0.937 | 1.22 | 2.08 | 1.64 | 76 | 322 | - | exact | 5.6e-16 / 1.1e-15 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 2.3e-15 / 4.7e-15 |  |  |  | 3.3e-16 / 6.7e-16 | 3.3e-16 / 6.7e-16 | 4.4e-16 / 8.9e-16 | 5.6e-16 / 1.1e-15 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 4.4e-16 / 8.9e-16 | 5.6e-16 / 1.1e-15 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 4.4e-16 / 8.9e-16 | 3.3e-16 / 6.7e-16 |
| router_16 | 0.884 | 103 | oom | - | 61.6 | - | - | - | >600 | >600 | >600 | 20.3 | 32.1 | 40.4 | 88.6 | >600 | 24.9 | 6.52 | 2.71 | 9.62 | 115 | 454 | - | exact | 1.4e-14 / 2.9e-14 | 5.0e-08 / 1.0e-07 |  |  | 1.1e-14 / 2.2e-14 |  |  |  |  |  |  | 1.5e-14 / 3.0e-14 | 1.5e-14 / 2.9e-14 | 1.5e-14 / 2.9e-14 | 1.5e-14 / 3.0e-14 |  | 4.5e-07 / 9.0e-07 | 1.5e-13 / 3.0e-13 | 1.5e-13 / 3.0e-13 | 1.5e-13 / 3.0e-13 | 1.5e-14 / 2.9e-14 | 1.5e-14 / 3.0e-14 |
| router_18 | 3.76 | >600 | - | - | 309 | - | - | - | - | - | - | 34.9 | 183 | 242 | >600 | - | oom | oom | 3.63 | 57.6 | 160 | 589 | - | exact | 1.9e-14 / 3.7e-14 |  |  |  | 1.5e-14 / 2.9e-14 |  |  |  |  |  |  | 1.9e-14 / 3.8e-14 | 1.9e-14 / 3.8e-14 | 1.9e-14 / 3.8e-14 |  |  |  |  | 1.5e-13 / 3.1e-13 | 1.5e-13 / 3.1e-13 | 1.9e-14 / 3.8e-14 | 1.9e-14 / 3.8e-14 |
| router_20 | 4.45 | - | - | - | >600 | - | - | - | - | - | - | 53 | oom | >600 | - | - | - | - | 5.97 | oom | 245 | >600 | - | exact | 5.9e-14 / 1.2e-13 |  |  |  |  |  |  |  |  |  |  | 5.8e-14 / 1.2e-13 |  |  |  |  |  |  | 7.6e-14 / 1.5e-13 |  | 5.9e-14 / 1.2e-13 |  |
| router_24 | oom | >600 | oom | >600 | >600 | >600 | oom | >600 | >600 | >600 | >600 | 124 | 28 | 28.6 | - | - | - | - | >600 | - | oom | - | >600 | exact |  |  |  |  |  |  |  |  |  |  |  | 2.4e-11 / 4.8e-11 | 2.4e-11 / 4.8e-11 | 2.4e-11 / 4.8e-11 |  |  |  |  |  |  |  |  |
| router_28 | - | - | - | - | - | - | - | - | - | - | - | 233 | 51.4 | 51.5 | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_32 | - | - | - | - | - | - | - | - | - | - | - | 413 | 89.6 | 89.6 | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_40 | - | - | - | - | - | - | - | - | - | - | - | >600 | 241 | 240 | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| weather_h1 | 0.003 | 0.023 | err | err | 0.006 | 0.602 | 3.6 | 0.338 | 1.56 | 2.02 | 9.65 | 11.2 | 12.2 | >600 | 1.1 | 1.14 | zero | 0.315 | 1.52 | 0.229 | 31.3 | 21 | 0.069 | exact | 2.6e-23 / 2.8e-16 | 6.4e-09 / 6.9e-02 |  |  | 0.0e+00 / 0.0e+00 | 7.7e-21 / 8.2e-14 | 2.6e-23 / 2.8e-16 | 1.2e-17 / 1.3e-10 | 2.6e-23 / 2.8e-16 | 2.6e-23 / 2.8e-16 | 1.3e-23 / 1.4e-16 | 1.3e-23 / 1.4e-16 | 5.3e-23 / 5.7e-16 |  | 2.6e-23 / 2.8e-16 | 1.3e-23 / 1.4e-16 |  | 4.1e-20 / 4.4e-13 | 4.1e-20 / 4.4e-13 | 4.1e-20 / 4.4e-13 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| weather_h2 | 0.009 | 0.127 | - | - | oom | 134 | 6.29 | 0.413 | 2.68 | 2.09 | 15.4 | 13.4 | 12.5 | - | 1.44 | 1.56 | zero | 0.3 | 1.82 | 0.127 | 32.8 | 59.4 | 0.689 | exact | 2.6e-23 / 2.1e-16 | 2.4e-08 / 1.9e-01 |  |  |  | 5.2e-21 / 4.2e-14 | 2.6e-23 / 2.1e-16 | 8.3e-17 / 6.7e-10 | 2.6e-23 / 2.1e-16 | 1.3e-22 / 1.1e-15 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 4.4e-18 / 3.6e-11 |  | 0.0e+00 / 0.0e+00 | 5.3e-23 / 4.3e-16 |  | 4.8e-19 / 3.9e-12 | 4.8e-19 / 3.9e-12 | 4.8e-19 / 3.9e-12 | 2.6e-23 / 2.1e-16 | 2.6e-23 / 2.1e-16 |
| weather_h3 | 0.024 | 0.283 | - | - | - | >600 | 7.69 | 0.622 | 2.8 | 2.08 | 24.7 | 15.3 | 12.2 | - | 1.69 | 1.83 | zero | 0.214 | oom | 0.18 | 47.3 | 104 | 2.7 | exact | 7.9e-23 / 5.0e-16 | 4.2e-08 / 2.6e-01 |  |  |  |  | 7.9e-23 / 5.0e-16 | 9.9e-17 / 6.2e-10 | 1.1e-22 / 6.7e-16 | 3.7e-22 / 2.3e-15 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 7.4e-18 / 4.7e-11 |  | 7.9e-23 / 5.0e-16 | 2.6e-23 / 1.7e-16 |  | 1.6e-19 / 1.0e-12 |  | 1.6e-19 / 1.0e-12 | 5.3e-23 / 3.3e-16 | 2.6e-23 / 1.7e-16 |
| weather_h4 | 0.12 | 0.442 | - | - | - | - | oom | 3.21 | 2.98 | 2.09 | 35.9 | 17.8 | 12.3 | - | 2.14 | 4.46 | zero | 0.263 | - | 0.324 | 60.6 | 144 | 15.7 | exact | 1.1e-22 / 5.5e-16 | 7.3e-09 / 3.8e-02 |  |  |  |  |  | 6.2e-17 / 3.2e-10 | 1.3e-22 / 6.9e-16 | 4.0e-22 / 2.1e-15 | 2.6e-23 / 1.4e-16 | 2.6e-23 / 1.4e-16 | 1.0e-17 / 5.4e-11 |  | 7.9e-23 / 4.1e-16 | 5.3e-23 / 2.7e-16 |  | 7.6e-20 / 3.9e-13 |  | 7.6e-20 / 3.9e-13 | 1.3e-22 / 6.9e-16 | 7.9e-23 / 4.1e-16 |
| weather_h5 | 0.132 | 0.594 | - | - | - | - | - | 21.3 | 3.96 | 2.1 | 50 | 20.4 | 12.5 | - | 2.38 | 8.02 | zero | 0.352 | - | 0.54 | 50.4 | 162 | 51 | exact | 1.9e-22 / 8.2e-16 | 2.7e-08 / 1.2e-01 |  |  |  |  |  | 1.2e-16 / 5.2e-10 | 1.1e-22 / 4.7e-16 | 3.4e-22 / 1.5e-15 | 2.6e-23 / 1.2e-16 | 5.3e-23 / 2.3e-16 | 1.3e-17 / 5.9e-11 |  | 1.3e-22 / 5.8e-16 | 5.3e-23 / 2.3e-16 |  | 1.2e-19 / 5.3e-13 |  | 1.2e-19 / 5.3e-13 | 1.1e-22 / 4.7e-16 | 7.9e-23 / 3.5e-16 |
| weather_h6 | 0.158 | 0.752 | - | - | - | - | - | 165 | 4.83 | 2.09 | 66.3 | 22.7 | 11.4 | - | 2.6 | 10.3 | zero | 0.468 | - | 0.803 | 56.3 | 202 | oom | exact | 1.6e-22 / 6.1e-16 | 3.8e-08 / 1.5e-01 |  |  |  |  |  | 1.7e-16 / 6.6e-10 | 1.6e-22 / 6.1e-16 | 6.4e-22 / 2.4e-15 | 5.3e-23 / 2.0e-16 | 5.3e-23 / 2.0e-16 | 1.6e-17 / 6.3e-11 |  | 1.1e-22 / 4.0e-16 | 2.1e-22 / 8.1e-16 |  | 5.0e-19 / 1.9e-12 |  | 5.0e-19 / 1.9e-12 | 1.1e-22 / 4.0e-16 | 1.1e-22 / 4.0e-16 |
| weather_h7 | 0.224 | 0.908 | - | - | - | - | - | >600 | 7.06 | 2.1 | 85.2 | 27.9 | 11.3 | - | 3.26 | 19 | zero | 0.613 | - | 1.26 | 61.4 | 231 | - | exact | 1.6e-22 / 5.4e-16 | 4.0e-09 / 1.3e-02 |  |  |  |  |  |  | 3.2e-22 / 1.1e-15 | 3.7e-22 / 1.3e-15 | 5.3e-23 / 1.8e-16 | 1.1e-22 / 3.6e-16 | 2.0e-17 / 6.6e-11 |  | 1.6e-22 / 5.4e-16 | 3.7e-22 / 1.3e-15 |  | 3.8e-19 / 1.3e-12 |  | 3.8e-19 / 1.3e-12 | 2.1e-22 / 7.2e-16 | 2.1e-22 / 7.2e-16 |
| weather_h8 | 0.241 | 1.08 | - | - | - | - | - | - | 8.3 | 2.09 | 107 | 29.9 | 10.6 | - | 3.46 | 22.2 | zero | 0.749 | - | 1.66 | 71.7 | 269 | - | exact | 2.1e-22 / 6.4e-16 | 3.0e-08 / 9.2e-02 |  |  |  |  |  |  | 2.6e-22 / 8.0e-16 | 3.7e-22 / 1.1e-15 | 1.1e-22 / 3.2e-16 | 1.6e-22 / 4.8e-16 | 2.3e-17 / 6.8e-11 |  | 2.6e-22 / 8.0e-16 | 1.6e-22 / 4.8e-16 |  | 2.3e-19 / 7.1e-13 |  | 2.3e-19 / 7.1e-13 | 1.6e-22 / 4.8e-16 | 2.6e-22 / 8.0e-16 |
| weather_h9 | 0.246 | 1.24 | - | - | - | - | - | - | 16.8 | 2.1 | 131 | 31.9 | 10 | - | 3.89 | 30.7 | zero | 0.997 | - | 2.14 | 69.5 | 295 | - | exact | 1.6e-22 / 4.4e-16 | 3.5e-08 / 9.6e-02 |  |  |  |  |  |  | 1.6e-22 / 4.4e-16 | 1.3e-21 / 3.5e-15 | 5.3e-23 / 1.5e-16 | 5.3e-23 / 1.5e-16 | 2.6e-17 / 7.0e-11 |  | 2.1e-22 / 5.8e-16 | 3.7e-22 / 1.0e-15 |  | 1.2e-19 / 3.4e-13 |  | 1.2e-19 / 3.4e-13 | 5.3e-23 / 1.5e-16 | 1.1e-22 / 2.9e-16 |
| weather_h100 | 5.68 | 16.7 | >600 | err | oom | >600 | oom | >600 | >600 | 2.57 | >600 | 303 | 14.1 | >600 | 30.2 | >600 | oom | 169 | - | 344 | >600 | >600 | >600 | exact | 1.6e-20 / 4.5e-15 | 1.0e-09 / 2.9e-04 |  |  |  |  |  |  |  | 1.4e-19 / 3.9e-14 |  | 1.7e-20 / 4.7e-15 | 3.0e-16 / 8.6e-11 |  | 2.1e-21 / 6.1e-16 |  |  | 2.1e-18 / 6.0e-13 |  | 2.1e-18 / 6.0e-13 |  |  |
| weather_h200 | 15.2 | 32.7 | - | - | - | - | - | - | - | 2.83 | - | 560 | 14 | - | 69.6 | - | - | >600 | - | >600 | - | - | - | exact | 5.9e-20 / 8.5e-15 | 4.3e-08 / 6.2e-03 |  |  |  |  |  |  |  | 5.5e-19 / 8.0e-14 |  | 6.9e-20 / 9.9e-15 | 6.0e-16 / 8.7e-11 |  | 7.6e-21 / 1.1e-15 |  |  |  |  |  |  |  |
| weather_h300 | 20.7 | 48.5 | - | - | - | - | - | - | - | 3.14 | - | >600 | 13.9 | - | >600 | - | - | - | - | - | - | - | - | exact | 1.5e-19 / 1.5e-14 | 1.3e-08 / 1.3e-03 |  |  |  |  |  |  |  | 1.3e-18 / 1.2e-13 |  |  | 9.0e-16 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h400 | 19 | 64.8 | - | - | - | - | - | - | - | 3.28 | - | - | 13.6 | - | - | - | - | - | - | - | - | - | - | exact | 2.4e-19 / 1.7e-14 | 3.1e-08 / 2.2e-03 |  |  |  |  |  |  |  | 2.2e-18 / 1.6e-13 |  |  | 1.2e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h500 | 55.3 | 81.1 | - | - | - | - | - | - | - | 3.46 | - | - | 14.2 | - | - | - | - | - | - | - | - | - | - | exact | 4.1e-19 / 2.4e-14 | 2.5e-08 / 1.4e-03 |  |  |  |  |  |  |  | 3.5e-18 / 2.0e-13 |  |  | 1.5e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h600 | 13.6 | 104 | oom | err | err | >600 | oom | >600 | >600 | 4.6 | >600 | >600 | 15.3 | >600 | - | - | - | - | - | - | - | - | oom | exact | 1.6e-18 / 7.7e-14 | 1.9e-08 / 9.1e-04 |  |  |  |  |  |  |  | 5.0e-18 / 2.4e-13 |  |  | 1.8e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h700 | 40.2 | 116 | - | - | - | - | - | - | - | 4.27 | - | - | 15 | - | - | - | - | - | - | - | - | - | - | exact | 8.2e-19 / 3.4e-14 | 3.7e-08 / 1.5e-03 |  |  |  |  |  |  |  | 6.8e-18 / 2.8e-13 |  |  | 2.1e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h800 | 55.1 | 131 | - | - | - | - | - | - | - | 4.64 | - | - | 15.3 | - | - | - | - | - | - | - | - | - | - | exact | 1.0e-18 / 3.6e-14 | 6.8e-09 / 2.5e-04 |  |  |  |  |  |  |  | 9.0e-18 / 3.3e-13 |  |  | 2.4e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h900 | 58.2 | 147 | - | - | - | - | - | - | - | 5.08 | - | - | 15.2 | - | - | - | - | - | - | - | - | - | - | exact | 1.3e-18 / 4.1e-14 | 4.9e-08 / 1.6e-03 |  |  |  |  |  |  |  | 1.1e-17 / 3.7e-13 |  |  | 2.7e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h1000 | 65.1 | 162 | - | - | - | - | - | - | - | 5.95 | - | - | 16 | - | - | - | - | - | - | - | - | - | - | exact | 1.6e-18 / 4.8e-14 | 5.3e-09 / 1.5e-04 |  |  |  |  |  |  |  | 1.4e-17 / 4.1e-13 |  |  | 3.0e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_n2 | 0.011 | 0.007 | err | err | 0.135 | >600 | 7.43 | 0.56 (2/3) | 2.76 | 0.156 | 0.156 | 0.161 | 1.27 | 1.27 | 1.72 | 1.85 | 0.173 | 0.287 | 16.7 | 0.218 | 40.1 | 91.2 | 3.93 | exact | 2.8e-17 / 1.5e-16 | 4.0e-08 / 2.2e-07 |  |  | 2.8e-17 / 1.5e-16 |  | 2.8e-17 / 1.5e-16 | 5.6e-16 / 3.0e-15 | 8.3e-17 / 4.5e-16 | 5.6e-17 / 3.0e-16 | 2.8e-17 / 1.5e-16 | 8.3e-17 / 4.5e-16 | 2.8e-17 / 1.5e-16 | 2.8e-17 / 1.5e-16 | 2.8e-17 / 1.5e-16 | 8.3e-17 / 4.5e-16 | 6.0e-08 / 3.3e-07 | 3.9e-13 / 2.1e-12 | 3.9e-13 / 2.1e-12 | 3.9e-13 / 2.1e-12 | 1.4e-16 / 7.5e-16 | 8.3e-17 / 4.5e-16 |
| weather_n7 | 0.038 | 0.115 | - | - | >600 | - | oom | >600 | 4.92 | 0.182 | 1.66 | 2.15 | 1.59 | 21.5 | 2.63 | 4.28 | 0.317 | 0.65 | 102 | 1.1 | 67.5 | 251 | oom | exact | 1.4e-20 / 2.0e-16 | 3.6e-08 / 5.4e-04 |  |  |  |  |  |  | 4.1e-20 / 6.0e-16 | 2.0e-19 / 3.0e-15 | 1.4e-20 / 2.0e-16 | 0.0e+00 / 0.0e+00 | 1.4e-20 / 2.0e-16 | 1.4e-20 / 2.0e-16 | 2.7e-20 / 4.0e-16 | 1.4e-20 / 2.0e-16 | 3.6e-07 / 5.4e-03 | 1.9e-17 / 2.8e-13 | 1.9e-17 / 2.8e-13 | 1.9e-17 / 2.8e-13 | 6.8e-20 / 1.0e-15 | 2.7e-20 / 4.0e-16 |
| weather_n8 | 0.133 | 0.26 | - | - | - | - | - | - | 5.95 | 0.27 | 7.66 | 3.49 | 12.6 | 386 | 2.81 | 6.4 | 0.39 | 0.755 | 593 | 1.51 | 70.3 | 276 | - | exact | 8.5e-21 / 6.5e-16 | 1.1e-08 / 8.8e-04 |  |  |  |  |  |  | 1.4e-20 / 1.0e-15 | 4.1e-20 / 3.1e-15 | 3.4e-21 / 2.6e-16 | 5.1e-21 / 3.9e-16 | 1.7e-21 / 1.3e-16 | 1.7e-21 / 1.3e-16 | 1.4e-20 / 1.0e-15 | 1.0e-20 / 7.8e-16 | 1.1e-08 / 8.8e-04 | 3.7e-17 / 2.9e-12 | 3.7e-17 / 2.9e-12 | 3.7e-17 / 2.9e-12 | 5.1e-21 / 3.9e-16 | 0.0e+00 / 0.0e+00 |
| weather_n9 | 0.305 | 0.607 | - | - | - | - | - | - | 7.55 | 0.624 | 35.6 | 8.36 | 189 | >600 | 3.49 | 11.6 | 0.853 | 1.27 | oom | 2.33 | 74.1 | 310 | - | exact | 2.5e-21 / 1.0e-15 | 1.9e-08 / 7.4e-03 |  |  |  |  |  |  | 3.0e-21 / 1.2e-15 | 6.4e-21 / 2.5e-15 | 8.5e-22 / 3.4e-16 | 1.3e-21 / 5.0e-16 | 2.5e-21 / 1.0e-15 |  | 3.4e-21 / 1.3e-15 | 2.1e-21 / 8.4e-16 | 4.8e-07 / 1.9e-01 | 4.2e-18 / 1.7e-12 |  | 4.2e-18 / 1.7e-12 | 4.2e-22 / 1.7e-16 | 0.0e+00 / 0.0e+00 |
| weather_n10 | 0.317 | 1.42 | - | - | - | - | - | - | 20.3 | 2.12 | 158 | 37 | 13.4 | - | 4.27 | 20.6 | zero | 1.2 | - | 2.67 | 84.6 | 347 | - | exact | 4.2e-22 / 1.1e-15 | 6.5e-10 / 1.6e-03 |  |  |  |  |  |  | 2.6e-22 / 6.6e-16 | 6.4e-22 / 1.6e-15 | 1.1e-22 / 2.7e-16 | 1.1e-22 / 2.7e-16 | 2.9e-17 / 7.1e-11 |  | 4.2e-22 / 1.1e-15 | 5.3e-23 / 1.3e-16 |  | 3.7e-19 / 9.4e-13 |  | 3.7e-19 / 9.4e-13 | 2.1e-22 / 5.3e-16 | 3.7e-22 / 9.3e-16 |
| weather_n11 | 0.709 | 3.32 | - | - | - | - | - | - | 41.1 | 8.22 | >600 | 179 | err | - | 6.16 | 98.7 | zero | 1.73 | - | 4.29 | 89.7 | 387 | - | exact | 6.6e-23 / 1.0e-15 | 3.6e-08 / 5.7e-01 |  |  |  |  |  |  | 6.6e-23 / 1.0e-15 | 4.4e-22 / 6.9e-15 |  | 1.3e-23 / 2.1e-16 |  |  | 1.3e-23 / 2.1e-16 | 5.3e-23 / 8.3e-16 |  | 1.4e-21 / 2.1e-14 |  | 1.4e-21 / 2.1e-14 | 1.3e-23 / 2.1e-16 | 1.3e-23 / 2.1e-16 |
| weather_n12 | 0.74 | zero | - | - | - | - | - | - | 216 | 33.4 | - | err | - | - | 8.33 | 241 | zero | 2.36 | - | 8.74 | 101 | 425 | - | exact | 1.2e-23 / 1.1e-15 |  |  |  |  |  |  |  | 1.2e-23 / 1.1e-15 | 6.5e-23 / 6.4e-15 |  |  |  |  | 6.6e-24 / 6.5e-16 | 2.8e-23 / 2.8e-15 |  | 8.7e-21 / 8.5e-13 |  | 8.7e-21 / 8.5e-13 | 1.7e-24 / 1.6e-16 | 5.0e-24 / 4.9e-16 |
| weather_n13 | 3.56 | zero | - | - | - | - | - | - | 176 | 138 | - | - | - | - | 15.5 | 345 | zero | 7.72 | - | 16.5 | 128 | 494 | - | exact | 3.1e-24 / 1.9e-15 |  |  |  |  |  |  |  | 3.1e-24 / 1.9e-15 | 1.9e-23 / 1.2e-14 |  |  |  |  | 1.7e-24 / 1.0e-15 | 1.9e-24 / 1.2e-15 |  | 4.7e-21 / 2.9e-12 |  | 4.7e-21 / 2.9e-12 | 6.2e-25 / 3.8e-16 | 2.1e-25 / 1.3e-16 |
| weather_n15 | 22 | zero | >600 | err | oom | >600 | oom | >600 | >600 | oom | err | err | err | err | >600 | >600 | oom | oom | - | oom | 163 | 599 | >600 | exact | 8.4e-26 / 2.0e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 3.2e-26 / 7.8e-16 | 3.9e-26 / 9.4e-16 |
| weather_n16 | 17.9 | zero | >600 | err | oom | >600 | oom | >600 | >600 | >600 | err | err | oom | oom | - | - | - | - | - | - | 159 | 559 | >600 | exact | 8.1e-27 / 1.3e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 8.1e-27 / 1.3e-15 | 4.8e-27 / 7.7e-16 |
| weather_n17 | oom | zero | >600 | err | oom | >600 | oom | >600 | >600 | >600 | err | err | oom | oom | - | - | - | - | - | - | 373 | >600 | >600 | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |
| weather_n18 | oom | >600 | >600 | err | oom | >600 | oom | >600 | >600 | >600 | err | err | err | err |  |  |  |  |  |  |  | - | oom | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| weather_n19 | oom | >600 | >600 | err | oom | >600 | oom | >600 | >600 | >600 | err | err | oom | oom |  |  |  |  |  |  |  |  | oom | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |

Seconds per system, the median over the repetitions; a failure is `>600`
(timeout), `oom`, `err` or `n/a`; each system's last columns are its
largest absolute and relative difference from the reference named in
`ref`.

## Setup

The benchmark driver, `scripts/bench.py` on the
[`baselines` branch][baselines], is how every table here was made.
`bench.py gen` writes the instances from a fixed specification
(`SPEC` and the suites after it) with the generators in `scripts/`, so
an instance is identified by its name; `run` runs every (instance,
system) pair and appends a JSON line per run; `table` prints the
Markdown tables. `gen` and `run` take `--suite`, `--families` and
`--only`; after a timeout `run` skips the family's larger instances for
that system unless `--no-skip` is given. The suites are `standard` (the
default, 84 instances), `indicators` (40 larger programs: networks with
evidence, graphs to 40 nodes, grids to 40×40, UW-CSE to 12 students,
Cora to 1 000 pairs) and those of the sections above: `large`, `lit`,
`ace`, `uai`, `tpch`, `biomine`, `ppl`, `bitblast`, `roulette` and
`geni`. The standard suite's families:

- **graph** `g<n>_<m>`: a random directed graph with n nodes and m
  probabilistic edges, `path/2` transitive closure, queries `path(1,X)`.
- **grid** `grid<n>`: an n×n grid with probabilistic right and down edges
  (and some left and up), one corner-to-corner reachability query.
- **smokers** `s<n>`: the smokers network with n people, three friends
  each.
- **uwcse** `uwcse<n>`: cplint's UW-CSE CP-logic theory with one course,
  one professor and n students; annotated disjunctions and negation.
- **cora** `cora<q>`: Cora citation matching from cplint_datasets, fold m1
  (1 493 facts) under the initial theory of the learning problem (559
  rules at probability 0.3, the transitive rules dropped), q `samebib`
  queries.
- **cplint** `bloodtype`: cplint's Mendelian blood type example.
- **bn** `bn_<network>`: a Bayesian network from the bnlearn repository
  (`data/bn`) written by `scripts/gen_bn.py` as one annotated disjunction
  per CPT row, every leaf queried; **bnev** `bn_<network>_ev` observes
  half the leaves (on a network with one leaf, that leaf) and queries the
  rest.
- **dice** `dice_<program>`: the baseline programs of the Dice paper
  (`data/dice`) translated by `scripts/dice2problog.py`; observations
  inside a branch become evidence that holds when the branch is not
  taken.

Ace's suite reads `benchmarks_v1.0.zip` where it is unpacked
(`--ace-benchmarks DIR`): a
network of 2004-pgm or 2005-ijcai observes its evidence set `-0006` and
queries the unobserved leaves, one of 2006-ijar (`fs-*`) observes its
one set and queries every other variable. `ACE_NETWORKS` in `bench.py`
lists the networks, those left out and those rescaled.

Systems:

- `tddlog:<strategy>`, by default `auto`; `td`, `closed`, `families`,
  `families:0` and `dynamic` for single strategies. Time is the whole run.
- `problog:ddnnf`, `problog:sdd`, `problog:fsdd`: ProbLog 2.3.0 with
  d-DNNF (dsharp), SDD (PySDD), and forward Tp-compilation with SDDs, the
  closest relative of tddlog.
- `pita:bdd` (cplint's PITA), `aspmc:sharpsat-td` (not run on programs
  with evidence, which its ProbLog front end ignores), `scallop:topk<k>`
  (approximate).
- `dice:bdd` (`--dice CMD`) on the `dice`, `bn` and `nodice` families,
  and on the circuits of the `arith`, `hybit` and `pluck` programs (the
  Dice programs `alea_bench.jl` writes, the seconds summed). The lab
  machines run the binary copied out of its docker image (baselines.md,
  section 4).
- `alea:cudd` (`--alea CMD`) on the `dice`, `bn` and `nodice` families:
  Alea.jl on the program `dice2alea.py` writes from the Dice program, a
  network as one program that returns its queried variables; on the
  `arith`, `hybit` and `pluck` families the Alea.jl program itself,
  through `alea_bench.jl`. Times: the wall time of the Julia process,
  and `alea:cudd in-process`, what the process reports for building the
  program and running `pr` after loading Alea and a warm-up, with the
  JIT's share of it; on the Alea.jl programs also `alea:cudd warm`, the
  same done a second time in the process (baselines.md, section 5).
- `exact:cliquetree`, `exact:lit`, `exact:tpch`, `exact:reliability`
  (`--exact`): `bn_exact.py`, `lit_exact.py`, `tpch_exact.py` and
  `reliability_exact.py`, exact solvers written for this repository as
  answer references; they are not compared on time.
- `merlin:bte` (`--merlin BIN`) on the networks, and `merlin:bte-pr`
  (`-t PR`) under `--pr`.
- `agrum:jtree` (pyAgrum 3.2.1, `--agrum PY`), `pgmpy:ve` (pgmpy 1.1.2,
  `--pgmpy PY`), `libdai:jtree` (libDAI 0.3.2, `--libdai BIN`) and
  `tensorinference:treesa` and `tensorinference:greedy`
  (TensorInference.jl 0.6.3 on Julia 1.11.9, `--tensorinference`): the
  exact network tools beside Ace and Merlin, on the UAI files, with
  `-pr` variants under `--pr`. Details in [baselines.md][baselines-doc],
  section 20.
- `provsql:default` (ProvSQL 1.13.0 on PostgreSQL 16.4, `--provsql`) on the
  TPC-H programs, as SQL.
- `storm:sparse`, `storm:exact`, `storm:dd` (Storm 1.14 through stormpy,
  `--storm`) and `prism:sparse`, `prism:hybrid` (PRISM 4.8.1, `--prism`) on
  the weather factory, as a PRISM model.
- `geni:default` (Geni, gennifer, `--geni`), `nodice:default` (noDice,
  `--nodice`), `psi:default` (Psi, `--psi`), `genfer:default` (Genfer,
  `--genfer`), `sppl:default` (SPPL 2.0.4, `--sppl`) and `pluck:default`
  (Pluck, `--pluck`) on the programs of their papers; the programs and
  versions are in baselines.md, sections 12 and 22.
- `ganak:wmc` and `sharpsat-td:decot<S>` (`--pr` with `--ganak BIN`,
  `--sharpsat-td BIN`): the weighted model count of a UAI network's
  weighted CNF (`gen --cnf`) by the model counters Ganak (in exact
  rationals) and sharpSAT-TD (in 53-bit MPFR floats after S seconds of
  FlowCutter, `--sharpsat-td-decot`), a second reference for log10 Z;
  they are not compared on time.
- `ace:<variant>` (`--ace DIR`): Ace 3.0 on every instance `gen_bn.py`
  writes, reading the network file with the instance's evidence (`.inst`)
  and compiling with it (`-e`); `auto` lets Ace choose between tabular
  variable elimination and c2d, other variants name compile options. The
  time is the sum of its two Java processes. Details in
  [baselines.md][baselines-doc], section 7.

`--pr` (on `uai`) runs tddlog as
`<strategy>+evidence-probability+no-queries` and compares log10 Z.
Answers are compared with `exact:lit`, `exact:tpch` or
`exact:reliability` where it ran, else the first ProbLog compiler that
finished, else the clique tree, else `ace:auto` (under `--pr`, the first
of `--pr-ref`); `table` reports the largest difference. Approximate
systems get a column of their own.

Every run is single-threaded. Runs are made on a lab machine, never on
the development host: one after another with nothing else running, or in
lanes (`run --cpus`), one per physical core, each command pinned with
`taskset`, a family's instances in one lane in order of size and the
systems of an instance back to back, under an address-space limit
(`ulimit -v`) of at least the lane's share of memory. Lanes share memory
bandwidth and cache, so timings compare only within one run. A timed-out
run's whole process group is stopped.

[baselines]: https://github.com/Tractables/tddlog/tree/baselines
[baselines-doc]: https://github.com/Tractables/tddlog/blob/baselines/docs/baselines.md
