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

tddlog 0.1.0 `auto` as of 2026-10-22 (tididi c7c0c5d6, goatd e592d1e),
linked statically, and every baseline, each suite's baselines in one job
(seven jobs on five lab machines (48 physical cores and 743 GB; 36
physical cores and 503 GB; 20 physical cores and 503 GB; 16 physical
cores and 251 GB; 16 physical cores and 125 GB), 2026-10-19 to
2026-10-21), in parallel lanes, and tddlog alone in a later job for
each, on the same machine, in the same lanes, with the same limits and
repetitions (2026-10-22 to 2026-10-29) but the UAI networks's in 32
lanes rather than 8, with the same 31 GB a process, the UAI networks
asked P(e)'s in 28 lanes rather than 14, with the same 35 GB a process
and the final PR set's in 17 lanes rather than 34, with the same 29 GB a
process (Setup). The runs on Bayesian and Markov networks ran a build of
2026-10-28 (tididi ca759585, goatd e592d1e). The runs beside Roulette on
its upstream main and Roulette+tddlog's ran a build of 2026-10-31
(tididi c5642b97, goatd e592d1e). The third literature scan ran a build
of 2026-10-30 (tididi df7b84a3, goatd e592d1e). The reruns of tddlog
where an older build's record stood, and every system's runs on eleven
of the growth sizes ran a build of 2026-11-01 (tididi c5642b97, goatd
e592d1e). The runs beside Scallop ran a build of 2026-10-29 (tididi
df7b84a3, goatd e592d1e). The runs on the literature's logic programs
ran a build of 2026-10-31 (tididi c5642b97, goatd e592d1e). The runs on
the standard suite and the runs beside TensorInference.jl timed warm ran
a build of 2026-10-29 (tididi b5d9b21d, goatd e592d1e). The
event-calculus programs, the reruns of the literature scan's taint apps
and the growth sizes ran a build of 2026-10-25 with twin contraction
(tididi b9e5064e, goatd e592d1e) and a recursion over a successor
relation grounded level by level, which lacks only the release's sparse
route (tididi 9fc0a599). The runs on four of cplint on SWISH's examples,
and tddlog's on the families read as one program and its arguments ran a
build of 2026-10-31 (tididi c5642b97, goatd e592d1e). The reruns of
tddlog where an older build's record stood ran a build of 2026-10-31
(tididi c5642b97, goatd e592d1e). The runs on the symmetric smokers ran
a build of 2026-10-30 (tididi df7b84a3, goatd e592d1e). The runs on five
more of cplint's examples and one more of ProbLog's tutorial ran a build
of 2026-10-31 (tididi c5642b97, goatd e592d1e). The release's main
contains every solver change those builds have. The baselines: ProbLog
2.3.0 with d-DNNF, SDD and forward SDD compilation, PITA's BDDs and
aspmc with sharpSAT-TD on the program suites and Ace's networks, the
large networks and the UAI networks; Ace 3.0 (default route and c2d
forced, and in log space, `-Xlog`, on the UAI networks), Merlin 1.7.0's
bucket-tree elimination (`-a bte`), pyAgrum's junction tree, pgmpy's
variable elimination, libDAI's junction tree and TensorInference.jl's
tensor network contraction on the networks, and ProvSQL on TPC-H; Dice,
Alea.jl, Roulette, noDice, Psi, Genfer, SPPL, Geni and Pluck, the exact
systems that read Dice programs, on the standard suite's networks, Ace's
networks, the large networks and the UAI networks and with Ace, Merlin
and the network tools on the ground programs of the standard suite, the
literature suite, TPC-H and Biomine (the ground route). Seconds are wall
time, parsing and grounding included; the limit is 300 s a run on the
standard, literature and UAI suites and 600 s on Ace's and the large
networks. A program written in another language is converted once into
the file tddlog reads (`tddlog --program`'s of a network, which the
systems that read ProbLog programs are given while tddlog reads the
network itself), and that conversion, timed on its own, is part of the
time of tddlog and of every system whose input is made from its file, a
run that passes the limit with it counting as a timeout; a translator
written only to hand a program to one baseline (PITA's and PSouffle's,
the Hugin and UAI files of Ace, Merlin and the network tools) is not
timed.

| suite | instances | tddlog finishes (total s) | faster than ProbLog, PITA, aspmc and Ace | Merlin faster | faster than pyAgrum, pgmpy, libDAI and TensorInference.jl (ProvSQL on TPC-H) | finished where no exact system did |
|---|---:|---:|---:|---:|---:|---:|
| standard | 105 | 100 (181.1) | 70 of 70 (24 under 0.2 s) | 2 of 8 (26 under 0.2 s) | 4 of 6 (28 under 0.2 s) | 4 |
| literature | 328 | 303 (1659.6) | 91 of 94 (160 under 0.2 s) |  |  | 35 |
| Ace's networks | 46 | 46 (15.6) | 46 of 46 | 3 of 17 (13 under 0.2 s) | 13 of 16 (12 under 0.2 s) | 0 |
| large networks | 14 | 14 (15.2) | 14 of 14 | 10 of 14 | 4 of 14 | 0 |
| TPC-H | 295 | 289 (2235.2) | 191 of 191 |  | 0 of 259 | 28 |

An instance that tddlog and the fastest baseline both finish in under
0.2 s counts neither as faster nor as slower: differences of a few
milliseconds of start-up are not compared.

**Standard suite** (random graphs, grids, smokers, symmetric smokers,
UW-CSE, Cora, blood types, Bayesian networks with and without evidence,
the Dice programs). No exact system finishes g32_107, g34_113, grid30
and nsym16, which tddlog does not finish either, nor g25_80, g30_100,
nsym14 and uwcse2560, which it does (g30_100 in 100.5 s, most of the
suite's total). Against the fastest of ProbLog, PITA, aspmc and Ace on
each instance, in total time: Of the 105 instances both are given,
tddlog answers 100 and ProbLog, PITA, aspmc or Ace 94. Over the 100 that
either answers, a run that does not answer counted at the 300 s limit,
tddlog takes 181.1 s in all and the fastest of them 2 532 s; over the 94
both answer, 30.4 s against 731.5. tddlog alone answers g20_60, g25_80,
g30_100, uwcse2560, grid20 and 1 more. Neither answers g32_107, g34_113,
grid25, grid30 and nsym16. Merlin answers all 34 networks; the two both
take under 0.2 s on 26, and of the other 8 Merlin is faster on 2: 5.1 to
5.7 times on pathfinder (bn_pathfinder 0.081 s against 0.410). tddlog is
faster on 6, among them bn_link_ev (0.31 s against 29.1) and bn_munin_ev
(0.79 s against 2.20), and takes 5.0 s over the 34 against Merlin's
64.5.

**Literature suite.** tddlog finishes 303 of 328, 35 of them where no
exact baseline finishes (dnahmm16, eventcalc4_2000, eventcalc4_4000,
hmmpos3, hwf7, mnist_sum96 to mnist_sum768, noisyor240_360,
noisyor300_450, pdb320, pedigree5_10, pedigree6_8 to pedigree7_13,
pedigree10_6 to pedigree32_6, pgrid16_8 to pgrid18_17, rgraph20,
sickness45_40 to sickness240_40, taint_app_ca7, taint_savage_knife,
taint_yaaic and threedie1500). Against the fastest of ProbLog, PITA and
aspmc on each instance, in total time: Of the 328 instances both are
given, tddlog answers 303 and ProbLog, PITA or aspmc 254. Over the 304
that either answers, a run that does not answer counted at the 300 s
limit, tddlog takes 1 960 s in all and the fastest of them 17 073 s;
over the 253 both answer, 200.1 s against 2 072. tddlog is slower on
mastermind32_5_8 (9.92 s against ProbLog's 5.36) and mastermind16_5_8
(4.15 s against ProbLog's 3.00). tddlog alone answers mnist_sum16,
pedigree4_6, mnist_sum32, mnist_sum64, mnist_sum96 and 45 more. Only
ProbLog, PITA or aspmc answers problogtut_various_05_numberguessing_1
(ProbLog, 0.78 s). Neither answers taint_angulo, eventcalc4_8000,
rgraph24, taint_tuio_droid, taint_video_game and 19 more.

**Ace's networks and the large networks.** Against Ace in total time, on
Ace's networks: Both are given 46 instances and answer all of them. Over
the 46 both answer, tddlog takes 15.6 s in all and Ace 495.8 s. On the
large networks: Both are given 14 instances and answer all of them. Over
the 14 both answer, tddlog takes 15.2 s in all and Ace 134.7 s. Merlin
runs out of memory or time on 16 of Ace's networks (ace_blockmap_10_01,
ace_blockmap_10_03, ace_blockmap_15_03 to ace_blockmap_20_02,
ace_blockmap_22_01, ace_blockmap_22_02 and ace_fs_07 to ace_fs_29); of
the other 30 it is faster on 3 (ace_pathfinder 0.06 s against 0.25,
ace_diabetes 0.17 against 0.36, the rest by at most 0.19 s); the two
both take under 0.2 s on 13 more. On the large networks Merlin is faster
on barley, diabetes, mildew, munin2 and munin3 with and without evidence
(bn_barley 1.57 s against 2.19); tddlog on munin1 and munin4 with and
without evidence (bn_munin1 1.02 s against 26.4).

**The other network tools.** pyAgrum, pgmpy, libDAI and
TensorInference.jl read the same UAI files as Merlin and run in the same
lanes, single-threaded; TensorInference.jl with two contraction orders,
`treesa` and `greedy`, of which the faster counts; its time is its own
in the process, warm as Alea.jl's: a small model goes through the same
calls first, and the instance is solved a second time where the first
took under a third of the limit (else the first counts, less Julia's
compilation within it), so that Julia's start, the package's load and
their compilation are left out, and the search for a contraction order
stays in. Their answers go through the same checks as Merlin's (below),
and each has columns in the tables. On the standard suite's networks (34
instances) pyAgrum answers 34, pgmpy answers 30, libDAI answers 34,
TensorInference.jl answers 34, Merlin 34, tddlog 34. Of the 6 that
tddlog and a tool both answer in 0.2 s or more, tddlog is faster than
the fastest tool on 4, and takes 3.5 s over them against the fastest
tool's 9.6; a tool is faster on 2 (bn_pathfinder libDAI 0.08 s against
0.41 and bn_pathfinder_ev libDAI 0.07 s against 0.39). Both take under
0.2 s on 28 more. On Ace's networks (46 instances) pyAgrum answers 28,
pgmpy answers 17, libDAI answers 28, TensorInference.jl answers 22,
Merlin 30, tddlog 46. Of the 16 that tddlog and a tool both answer in
0.2 s or more, tddlog is faster than the fastest tool on 13, and takes
2.9 s over them against the fastest tool's 164.3; a tool is faster on 3
(ace_diabetes libDAI 0.16 s against 0.36, ace_pathfinder libDAI 0.06 s
against 0.25 and ace_munin3 pyAgrum 0.38 s against 0.38). Both take
under 0.2 s on 12 more. tddlog alone answers 18. On the large networks
(14 instances) pyAgrum answers 14, pgmpy answers 8, libDAI answers 14,
TensorInference.jl answers 14, Merlin 14, tddlog 14. Of the 14 that
tddlog and a tool both answer in 0.2 s or more, tddlog is faster than
the fastest tool on 4, and takes 15.2 s over them against the fastest
tool's 28.1; a tool is faster on 10 (bn_barley TensorInference.jl 0.26 s
against 2.19, bn_mildew TensorInference.jl 0.13 s against 1.76 and
bn_barley_ev TensorInference.jl 0.26 s against 1.84). On the UAI MAR
tuning set (120 instances) pyAgrum answers 107, pgmpy answers 53, libDAI
answers 117, TensorInference.jl answers 113, Merlin 113, tddlog 120. Of
the 98 that tddlog and a tool both answer in 0.2 s or more, tddlog is
faster than the fastest tool on 93, and takes 338.0 s over them against
the fastest tool's 871.3; a tool is faster on 5 (uai_objectdetection_56
pyAgrum 2.61 s against 3.34, uai_objectdetection_44 pyAgrum 2.66 s
against 3.28 and uai_objectdetection_64 pyAgrum 2.78 s against 3.29).
Both take under 0.2 s on 21 more. tddlog alone answers 1. On the
partition function of the MAR tuning set's networks (120 instances)
pyAgrum answers 108, pgmpy answers 104, libDAI answers 120,
TensorInference.jl answers 115, Merlin 114, tddlog 120. Of the 98 that
tddlog and a tool both answer in 0.2 s or more, tddlog is faster than
the fastest tool on 42, and takes 591.0 s over them against the fastest
tool's 517.2; a tool is faster on 56 (uai_objectdetection_29 pgmpy
3.99 s against 14.7, uai_objectdetection_21 pgmpy 2.07 s against 12.1
and uai_objectdetection_57 pgmpy 1.93 s against 11.7). Both take under
0.2 s on 22 more. On the final PR set (197 instances) pyAgrum answers
19, pgmpy answers 6, libDAI answers 21, TensorInference.jl answers 52,
Merlin 28, tddlog 158. Of the 61 that tddlog and a tool both answer in
0.2 s or more, tddlog is faster than the fastest tool on 54, and takes
700.9 s over them against the fastest tool's 2 649; a tool is faster on
7 (uai_1vfy TensorInference.jl 20.0 s against 194.0, uai_1aie
TensorInference.jl 10.8 s against 104.1 and uai_1f94 TensorInference.jl
15.6 s against 93.1). tddlog alone answers 97, a tool alone 6. On the PR
tuning set (121 instances) pyAgrum answers 108, pgmpy answers 104,
libDAI answers 121, TensorInference.jl answers 115, Merlin 115, tddlog
121. Of the 98 that tddlog and a tool both answer in 0.2 s or more,
tddlog is faster than the fastest tool on 58, and takes 542.7 s over
them against the fastest tool's 594.0; a tool is faster on 40
(uai_objectdetection_21 pgmpy 3.22 s against 11.6,
uai_objectdetection_15 pgmpy 3.17 s against 11.5 and
uai_objectdetection_25 pgmpy 3.21 s against 11.4). Both take under 0.2 s
on 23 more. On Ihler's networks (370 instances) pyAgrum answers 102,
pgmpy answers 111, libDAI answers 117, TensorInference.jl answers 150,
Merlin 111, tddlog 336. Of the 103 that tddlog and a tool both answer in
0.2 s or more, tddlog is faster than the fastest tool on 79, and takes
898.5 s over them against the fastest tool's 5 403; a tool is faster on
24 (uai_bn_29 pyAgrum 0.31 s against 204.4, uai_bn_22 pgmpy 3.27 s
against 178.8 and uai_bn_20 TensorInference.jl 6.49 s against 124.0).
Both take under 0.2 s on 45 more. tddlog alone answers 188, a tool alone
8.

**The Dice family and the ground route.** Dice, Alea.jl, Roulette,
noDice, Psi, Genfer, SPPL, Geni and Pluck, the exact systems that read
Dice programs, ran on the standard suite's networks, Ace's networks, the
large networks and the UAI networks as the Dice programs of
`scripts/gen_bn.py` and with Ace, Merlin and the other network tools on
the standard suite, the literature suite, TPC-H and Biomine, on the
network and the Dice program that `scripts/ground2net.py` writes of each
program's ground program. Alea.jl's, Pluck's and Roulette's times leave
out the start-up and compilation of Julia and Racket, as on the page of
the Dice family's suites. A baseline's answer off the reference by more
than 1e-9 and either by more than 1e-6 or by more than 1e-3 of it,
beyond the last digit either of the two prints, counts as a failure
(`wrong`): PSouffle on biomine1, biomine2, biomine4, biomine8, biomine12
and 15 more; Dice on dice_digitRecognition; Geni on ace_alarm,
ace_water, bn_alarm_ev, bn_child_ev, bn_hailfinder_ev and 31 more; PITA
on ace_fs_07, ace_mastermind_03_08_03, pcfgcount32, plcg4,
problogtut_basic_03_dice_8 and 49 more; ProbLog on uai_promedus_24;
aspmc on uai_bn_30, uai_bn_31, uai_bn_32, uai_bn_34, uai_bn_36 and 34
more; noDice on uai_blockmap_10_03_0010, uai_grid10x10_f10,
uai_grids_12, uai_mastermind_03_08_04_0001, uai_mastermind_03_08_04_0002
and 32 more; PRISM on uai_grids_17 and uai_grids_18; Ace on uai_bn_26,
uai_diagnose_a, uai_mastermind_03_08_05_0012 and
uai_mastermind_04_08_04_0012; pgmpy on uai_bn_20, uai_bn_21, uai_bn_26,
uai_bn_27, uai_diagnose_a and 1 more; libDAI on uai_diagnose_a and
uai_diagnose_b; TensorInference.jl on uai_diagnose_a and uai_diagnose_b;
pyAgrum on uai_diagnose_a and uai_diagnose_b.

**Validation.** Every answer is compared with a reference (Setup). The
largest absolute differences: 3.6e-7 over 64 843 answers on 97 standard
instances (ProbLog prints eight digits), 5.0e-9 on the 293 literature
instances (taint_app_ca7, taint_savage_knife, pedigree7_12,
pedigree14_10, pedigree7_13, sickness45_40, sickness180_40,
sickness240_40, noisyor120_180 and noisyor240_360 have no reference this
run), 1.9e-14 over 25 967 answers on 46 of Ace's 46 networks, 4.9e-11 on
the 14 large networks. g25_80, g30_100 and grid20 have no exact
reference; `scripts/sample_paths.py` estimates their answers from a
million sampled graphs (200 000 for the grids), and each of the 76
answers lies within 2.7 standard errors of its estimate (checked with an
earlier version's answers, from which this run's differ by at most
2.7e-14).

The other network tools' answers against the same references: pyAgrum
248 marginal runs to 3.6e-7 and 337 partition functions to 6.0e-5 in
log10; pgmpy 160 marginal runs to 3.6e-7 and 325 partition functions to
6.0e-5 in log10; libDAI 258 marginal runs to 5.0e-7 and 379 partition
functions to 4.4e-4 in log10; TensorInference.jl 392 marginal runs to
5.0e-7 and 650 partition functions to 4.4e-4 in log10 (the tools print
six digits, as Merlin does).

In the tables, ratio is the fastest exact baseline's time over tddlog's,
Merlin, pyAgrum, pgmpy, libDAI and TensorInference.jl among them on the
networks; `>300` is a timeout; `-` means no exact baseline finished;
`skip` is a run that the baseline job left out after the baseline ran
out of time or memory on a smaller size of the same program.

### Standard suite (105 instances, 300 s)

| instance | tddlog 0.1.0 | fastest exact baseline | ratio | merlin:bte | agrum:jtree | pgmpy:ve | libdai:jtree | tensorinference:treesa | tensorinference:greedy |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| g10_25 | 0.01 | 0.11 (merlin:bte) | 0.5 | 0.11 | 0.26 | 2.49 | 0.11 | 20.18 | 0.13 |
| g11_29 | 0.01 | 0.22 (problog:sdd) | 1.1 | err | err | err | err | err | err |
| g12_32 | 0.01 | 0.26 (problog:sdd) | 1.3 | skip | skip | skip | skip | skip | skip |
| g13_35 | 0.01 | 0.31 (problog:sdd) | 1.5 | skip | skip | skip | skip | skip | skip |
| g14_37 | 0.01 | 0.32 (problog:sdd) | 1.6 | skip | skip | skip | skip | skip | skip |
| g15_40 | 0.03 | 1.07 (problog:sdd) | 5.4 | skip | skip | skip | skip | skip | skip |
| g16_44 | 0.01 | 0.36 (problog:sdd) | 1.8 | skip | skip | skip | skip | skip | skip |
| g18_50 | 0.04 | 1.07 (problog:sdd) | 5.3 | skip | skip | skip | skip | skip | skip |
| g20_56 | 0.22 | 134.98 (problog:sdd) | 602.6 | skip | skip | skip | skip | skip | skip |
| g20_60 | 0.84 | - |  | skip | skip | skip | skip | skip | skip |
| g25_80 | 3.23 | - |  | skip | skip | skip | skip | skip | skip |
| g30_100 | 100.54 | - |  | skip | skip | skip | skip | skip | skip |
| g32_107 | >300 | - |  | skip | skip | skip | skip | skip | skip |
| g34_113 | skip | - |  | skip | skip | skip | skip | skip | skip |
| uwcse2 | 0.04 | 0.10 (merlin:bte) | 0.5 | 0.10 | 0.27 | 2.26 | 0.12 | 37.76 | 0.15 |
| uwcse3 | 0.02 | 0.12 (merlin:bte) | 0.6 | 0.12 | 0.27 | 2.14 | 0.15 | 58.17 | 0.70 |
| uwcse4 | 0.01 | 0.13 (merlin:bte) | 0.7 | 0.13 | 0.30 | 2.16 | 0.20 | 71.69 | 0.52 |
| uwcse5 | 0.01 | 0.16 (merlin:bte) | 0.8 | 0.16 | 0.37 | 2.20 | 0.28 | 91.23 | 0.67 |
| uwcse6 | 0.01 | 0.18 (merlin:bte) | 0.9 | 0.18 | 0.38 | 2.25 | 0.35 | 110.72 | 2.23 |
| uwcse8 | 0.02 | 0.25 (merlin:bte) | 1.3 | 0.25 | 0.45 | 2.45 | 0.60 | 162.93 | 28.19 |
| uwcse10 | 0.02 | 0.35 (merlin:bte) | 1.8 | 0.35 | 0.64 | 2.56 | 0.87 | 198.12 | err |
| uwcse20 | 0.04 | 1.10 (merlin:bte) | 5.5 | 1.10 | 1.37 | 3.42 | 4.11 | >300 | skip |
| uwcse80 | 0.12 | 11.54 (ace:auto) | 57.7 | 17.47 | 18.30 | 18.17 | 173.72 | skip | skip |
| uwcse640 | 1.43 | 92.10 (ace:auto) | 64.5 | >300 | >300 | >300 | >300 | skip | skip |
| uwcse2560 | 6.43 | - |  | skip | skip | skip | skip | skip | skip |
| s6 | 0.01 | 0.06 (libdai:jtree) | 0.3 | 0.08 | 0.23 | 2.05 | 0.06 | 2.43 | 0.08 |
| s10 | 0.01 | 0.11 (libdai:jtree) | 0.6 | 0.12 | 0.25 | 2.43 | 0.11 | 28.01 | 0.15 |
| s15 | 0.01 | 0.06 (libdai:jtree) | 0.3 | 0.07 | 0.21 | 2.04 | 0.06 | 2.70 | 0.07 |
| s20 | 0.01 | 0.07 (libdai:jtree) | 0.3 | 0.07 | 0.21 | 2.08 | 0.07 | 6.31 | 0.08 |
| s50 | 0.01 | 0.07 (merlin:bte) | 0.4 | 0.07 | 0.22 | 2.32 | 0.07 | 9.45 | 0.08 |
| s100 | 0.01 | 0.07 (libdai:jtree) | 0.4 | 0.08 | 0.22 | 2.53 | 0.07 | 8.34 | 0.08 |
| s200 | 0.01 | 0.09 (merlin:bte) | 0.4 | 0.09 | 0.25 | 4.07 | 0.10 | 16.75 | 0.11 |
| s1000 | 0.03 | 0.15 (libdai:jtree) | 0.8 | 0.17 | 0.47 | 60.43 | 0.15 | 70.79 | 0.70 |
| s3000 | 0.06 | 0.53 (libdai:jtree) | 2.6 | 0.69 | 2.10 | >300 | 0.53 | >300 | 0.92 |
| s10000 | 0.23 | 5.05 (merlin:bte) | 22.1 | 5.05 | 29.65 | skip | 12.46 | skip | >300 |
| s30000 | 0.67 | 17.50 (pita:bdd) | 26.0 | err | >300 | skip | 36.69 | skip | skip |
| cora10 | 0.05 | 0.12 (tensorinference:greedy) | 0.6 | 0.13 | 0.24 | 2.50 | 0.14 | 25.27 | 0.12 |
| cora50 | 0.03 | 0.17 (merlin:bte) | 0.9 | 0.17 | 0.41 | 6.96 | 0.22 | 111.20 | 0.40 |
| cora200 | 0.03 | 0.65 (merlin:bte) | 3.3 | 0.65 | 1.57 | 140.22 | 1.43 | >300 | 0.93 |
| cora500 | 0.05 | 1.97 (merlin:bte) | 9.8 | 1.97 | 6.16 | >300 | 8.49 | skip | err |
| cora1000 | 0.26 | 6.09 (merlin:bte) | 23.7 | 6.09 | 26.11 | skip | 43.14 | skip | skip |
| cora2000 | 0.21 | 13.19 (pita:bdd) | 63.7 | 23.04 | 128.06 | skip | 246.90 | skip | skip |
| cora4000 | 0.64 | 26.25 (pita:bdd) | 40.8 | err | >300 | skip | >300 | skip | skip |
| cora8000 | 1.09 | 53.90 (pita:bdd) | 49.6 | skip | skip | skip | skip | skip | skip |
| grid4 | 0.01 | 0.07 (libdai:jtree) | 0.4 | 6.22 | 0.23 | 2.05 | 0.07 | 6.98 | 0.10 |
| grid5 | 0.01 | 0.08 (libdai:jtree) | 0.4 | 0.09 | 0.22 | 2.02 | 0.08 | 12.60 | 0.09 |
| grid6 | 0.01 | 0.08 (merlin:bte) | 0.4 | 0.08 | 0.23 | 2.04 | 0.09 | 20.65 | 0.11 |
| grid7 | 0.01 | 0.09 (merlin:bte) | 0.5 | 0.09 | 0.24 | 2.08 | 0.09 | 29.48 | 0.12 |
| grid8 | 0.01 | 0.10 (merlin:bte) | 0.5 | 0.10 | 0.26 | 2.07 | 0.10 | 41.10 | 0.18 |
| grid10 | 0.02 | 0.15 (merlin:bte) | 0.7 | 0.15 | 0.30 | 2.16 | 0.15 | 67.19 | 0.28 |
| grid12 | 0.03 | 0.28 (libdai:jtree) | 1.4 | 0.30 | 0.41 | 2.23 | 0.28 | 108.45 | 0.37 |
| grid15 | 0.13 | 0.66 (tensorinference:greedy) | 3.3 | 12.46 | 1.87 | 2.46 | 7.72 | 196.22 | 0.66 |
| grid20 | 2.99 | 4.72 (pgmpy:ve) | 1.6 | err | err | 4.72 | err | >300 | 96.59 |
| grid25 | >300 | 156.37 (pgmpy:ve) |  | skip | skip | 156.37 | skip | skip | >300 |
| grid30 | skip | - |  | skip | skip | >300 | skip | skip | skip |
| bloodtype | 0.01 | 0.30 (libdai:jtree) | 1.5 | 0.33 | 0.32 | 2.60 | 0.30 | 78.62 | 0.34 |
| dice_twocoins | 0.01 | 0.07 (merlin:bte) | 0.3 | 0.07 | 0.20 | 1.96 | 0.07 | 0.61 | 0.07 |
| dice_evidence1 | 0.01 | 0.06 (libdai:jtree) | 0.3 | 0.07 | 0.20 | 1.98 | 0.06 | 0.39 | 0.07 |
| dice_evidence2 | 0.01 | 0.07 (tensorinference:greedy) | 0.3 | 0.07 | 0.21 | 2.02 | 0.07 | 1.35 | 0.07 |
| dice_grass | 0.01 | 0.07 (merlin:bte) | 0.3 | 0.07 | 0.21 | 1.98 | 0.07 | 2.55 | 0.07 |
| dice_alarm | 0.01 | 0.07 (merlin:bte) | 0.3 | 0.07 | 0.21 | 1.98 | 0.07 | 2.77 | 0.08 |
| dice_noisyOr | 0.01 | 0.07 (merlin:bte) | 0.4 | 0.07 | 0.21 | 2.01 | 0.08 | 4.60 | 0.07 |
| dice_murderMystery | 0.01 | 0.06 (tensorinference:greedy) | 0.3 | 0.07 | 0.20 | 1.99 | 0.07 | 1.51 | 0.06 |
| dice_digitRecognition | 0.37 | 9.62 (problog:sdd) | 26.0 | 78.43 | 224.77 | >300 | >300 | >300 | >300 |
| nsym4 | 0.01 | 0.10 (libdai:jtree) | 0.5 | 0.12 | 0.24 | 2.23 | 0.10 | 16.36 | 0.18 |
| nsym6 | 0.02 | 0.34 (problog:sdd) | 1.7 | err | err | err | err | >300 | err |
| nsym8 | 0.12 | 3.46 (aspmc:sharpsat-td) | 17.3 | skip | skip | skip | skip | skip | skip |
| nsym10 | 2.71 | 10.08 (aspmc:sharpsat-td) | 3.7 | skip | skip | skip | skip | skip | skip |
| nsym12 | 16.39 | 169.44 (aspmc:sharpsat-td) | 10.3 | skip | skip | skip | skip | skip | skip |
| nsym14 | 36.65 | - |  | skip | skip | skip | skip | skip | skip |
| nsym16 | err | - |  | skip | skip | skip | skip | skip | skip |
| bn_cancer_ev | 0.03 | 0.00 (libdai:jtree) | 0.0 | 0.00 | 0.10 | 1.34 | 0.00 | - | - |
| bn_cancer | 0.04 | 0.00 (libdai:jtree) | 0.0 | 0.00 | 0.10 | 1.30 | 0.00 | - | - |
| bn_earthquake_ev | 0.02 | 0.00 (merlin:bte) | 0.0 | 0.00 | 0.10 | 1.29 | 0.00 | - | - |
| bn_earthquake | 0.06 | 0.00 (merlin:bte) | 0.0 | 0.00 | 0.11 | 1.31 | 0.00 | - | - |
| bn_asia_ev | 0.02 | 0.00 (merlin:bte) | 0.0 | 0.00 | 0.10 | 1.31 | 0.00 | - | - |
| bn_asia | 0.06 | 0.00 (merlin:bte) | 0.0 | 0.00 | 0.10 | 1.31 | 0.00 | - | - |
| bn_survey_ev | 0.01 | 0.00 (merlin:bte) | 0.0 | 0.00 | 0.10 | 1.32 | 0.00 | - | - |
| bn_survey | 0.04 | 0.00 (merlin:bte) | 0.0 | 0.00 | 0.10 | 1.30 | 0.00 | - | - |
| bn_sachs_ev | 0.03 | 0.00 (libdai:jtree) | 0.0 | 0.00 | 0.11 | 1.32 | 0.00 | - | - |
| bn_sachs | 0.05 | 0.00 (merlin:bte) | 0.0 | 0.00 | 0.11 | 1.27 | 0.00 | - | - |
| bn_child | 0.05 | 0.01 (libdai:jtree) | 0.0 | 0.01 | 0.10 | 1.28 | 0.01 | - | - |
| bn_child_ev | 0.03 | 0.00 (libdai:jtree) | 0.0 | 0.01 | 0.11 | 1.29 | 0.00 | - | - |
| bn_insurance_ev | 0.04 | 0.01 (libdai:jtree) | 0.0 | 0.01 | 0.12 | 1.28 | 0.01 | - | - |
| bn_insurance | 0.07 | 0.01 (merlin:bte) | 0.0 | 0.01 | 0.11 | 1.29 | 0.01 | - | - |
| bn_water_ev | 0.06 | 0.24 (agrum:jtree) | 1.2 | 0.36 | 0.24 | 1.38 | 0.32 | - | - |
| bn_alarm_ev | 0.03 | 0.01 (libdai:jtree) | 0.0 | 0.01 | 0.12 | 1.36 | 0.01 | - | - |
| bn_hailfinder_ev | 0.07 | 0.01 (libdai:jtree) | 0.0 | 0.01 | 0.12 | 1.37 | 0.01 | - | - |
| bn_hepar2_ev | 0.06 | 0.01 (libdai:jtree) | 0.0 | 0.02 | 0.13 | 1.47 | 0.01 | - | - |
| bn_win95pts_ev | 0.04 | 0.01 (libdai:jtree) | 0.0 | 0.01 | 0.12 | 1.38 | 0.01 | - | - |
| bn_pathfinder_ev | 0.39 | 0.07 (merlin:bte) | 0.2 | 0.07 | 0.17 | 1.99 | 0.07 | - | - |
| bn_andes_ev | 0.06 | 0.07 (libdai:jtree) | 0.3 | 0.09 | 0.14 | 1.88 | 0.07 | - | - |
| bn_pigs_ev | 0.08 | 0.14 (merlin:bte) | 0.7 | 0.14 | 0.19 | 37.13 | 0.14 | - | - |
| bn_water | 0.08 | 0.31 (agrum:jtree) | 1.5 | 0.34 | 0.31 | 1.48 | 0.32 | - | - |
| bn_alarm | 0.06 | 0.01 (merlin:bte) | 0.0 | 0.01 | 0.11 | 1.33 | 0.01 | - | - |
| bn_hailfinder | 0.08 | 0.01 (libdai:jtree) | 0.1 | 0.01 | 0.12 | 1.32 | 0.01 | - | - |
| bn_hepar2 | 0.07 | 0.01 (libdai:jtree) | 0.1 | 0.02 | 0.12 | 1.56 | 0.01 | - | - |
| bn_link_ev | 0.31 | 3.62 (libdai:jtree) | 11.6 | 29.06 | 45.61 | >300 | 3.62 | - | - |
| bn_win95pts | 0.06 | 0.01 (libdai:jtree) | 0.0 | 0.01 | 0.12 | 1.41 | 0.01 | - | - |
| bn_pathfinder | 0.41 | 0.08 (libdai:jtree) | 0.2 | 0.08 | 0.20 | 3.45 | 0.08 | - | - |
| bn_andes | 0.08 | 0.08 (libdai:jtree) | 0.4 | 0.08 | 0.14 | 2.86 | 0.08 | - | - |
| bn_pigs | 0.10 | 0.14 (merlin:bte) | 0.7 | 0.14 | 0.19 | 28.61 | 0.15 | - | - |
| bn_link | 0.82 | 4.07 (libdai:jtree) | 4.9 | 29.55 | 47.89 | >300 | 4.07 | - | - |
| bn_munin_ev | 0.79 | 0.89 (agrum:jtree) | 1.1 | 2.20 | 0.89 | >300 | 3.33 | - | - |
| bn_munin | 0.79 | 0.90 (agrum:jtree) | 1.1 | 2.19 | 0.90 | err | 3.35 | - | - |

### Literature suite (328 instances, 300 s)

| instance | tddlog 0.1.0 | fastest exact baseline | ratio |
|---|---:|---|---:|
| noisyor6_9 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| pdb10 | 0.02 | 0.10 (problog:sdd) | 0.5 |
| mnist_sum4 | 0.03 | 1.85 (pita:bdd) | 9.2 |
| mastermind4_4_8 | 0.04 | 0.43 (problog:sdd) | 2.1 |
| pdb15 | 0.01 | 0.16 (problog:sdd) | 0.8 |
| mastermind6_4_8 | 0.04 | 0.61 (problog:sdd) | 3.0 |
| pedigree4_4 | 0.05 | - |  |
| sickness6_10 | 0.03 | 0.59 (pita:bdd) | 3.0 |
| noisyor8_12 | 0.01 | 0.15 (problog:sdd) | 0.7 |
| noisyor10_15 | 0.01 | 0.21 (problog:ddnnf) | 1.1 |
| pdb20 | 0.01 | 0.13 (problog:sdd) | 0.7 |
| mastermind8_4_8 | 0.05 | 0.75 (problog:sdd) | 3.7 |
| noisyor15_22 | 0.02 | 0.88 (problog:ddnnf) | 4.4 |
| pdb30 | 0.01 | 0.35 (problog:sdd) | 1.7 |
| pdb40 | 0.01 | 2.99 (aspmc:sharpsat-td) | 15.0 |
| mnist_sum8 | 0.03 | 33.26 (pita:bdd) | 166.3 |
| mastermind10_4_8 | 0.06 | 0.92 (problog:sdd) | 4.6 |
| noisyor20_30 | 0.01 | 7.88 (problog:ddnnf) | 39.4 |
| sickness10_10 | 0.02 | 2.30 (pita:bdd) | 11.5 |
| noisyor30_45 | 0.01 | 55.65 (pita:bdd) | 278.2 |
| mnist_sum16 | 0.07 | - |  |
| pedigree4_6 | 0.04 | - |  |
| sickness14_10 | 0.02 | 5.87 (pita:bdd) | 29.4 |
| sickness18_20 | 0.03 | 204.43 (pita:bdd) | 1022.1 |
| mastermind4_5_8 | 0.04 | 0.60 (problog:sdd) | 3.0 |
| mnist_sum32 | 0.24 | - |  |
| mnist_sum64 | 0.81 | - |  |
| mnist_sum96 | 1.75 | - |  |
| pedigree6_6 | 0.07 | - |  |
| pedigree10_6 | 0.19 | - |  |
| pedigree5_10 | 0.65 | - |  |
| pedigree6_8 | 0.09 | - |  |
| pedigree16_6 | 0.29 | - |  |
| pedigree32_6 | 1.13 | - |  |
| pedigree6_10 | 3.86 | - |  |
| sickness22_40 | 0.04 | - |  |
| pedigree7_10 | 3.00 | - |  |
| mastermind6_5_8 | 0.05 | 1.11 (problog:sdd) | 5.6 |
| mastermind8_5_8 | 0.07 | 1.84 (problog:sdd) | 9.2 |
| taint_app_ca7 | 3.08 | - |  |
| taint_andors_trail | 1.59 | 236.85 (pita:bdd) | 149.3 |
| taint_barcode | 3.13 | - |  |
| taint_tom_droid | 1.54 | - |  |
| taint_angulo | >300 | - |  |
| taint_tilt_mazes | 1.52 | - |  |
| eventcalc4_250 | 0.17 | 7.80 (problog:sdd) | 39.0 |
| eventcalc4_500 | 0.72 | 29.05 (problog:sdd) | 40.1 |
| eventcalc4_1000 | 6.03 | 136.79 (problog:sdd) | 22.7 |
| eventcalc4_2000 | 50.88 | - |  |
| eventcalc4_4000 | 215.27 | - |  |
| eventcalc4_8000 | >300 | - |  |
| taint_and_roc | 1.39 | - |  |
| taint_noisy_sounds | 2.95 | - |  |
| taint_savage_knife | 6.37 | - |  |
| taint_app_kQm | 1.03 | 238.44 (pita:bdd) | 230.6 |
| rgraph6 | 0.07 | 0.14 (problog:sdd) | 0.7 |
| rgraph8 | 0.12 | 0.23 (problog:sdd) | 1.1 |
| rgraph10 | 0.12 | 0.32 (problog:sdd) | 1.6 |
| rgraph12 | 1.97 | 3.50 (problog:sdd) | 1.8 |
| rgraph14 | 4.61 | 4.81 (problog:sdd) | 1.0 |
| rgraph17 | 4.60 | 38.62 (problog:sdd) | 8.4 |
| rgraph20 | 205.23 | - |  |
| rgraph24 | >300 | - |  |
| taint_app_018 | 2.62 | 21.95 (pita:bdd) | 8.4 |
| taint_tuio_droid | >300 | - |  |
| taint_video_game | >300 | - |  |
| taint_ginger_master | 1.69 | - |  |
| taint_yaaic | 5.38 | - |  |
| liftsep3 | 0.01 | 0.32 (problog:sdd) | 1.6 |
| liftsep4 | 0.04 | 0.89 (pita:bdd) | 4.5 |
| liftsep6 | 0.04 | 6.84 (aspmc:sharpsat-td) | 34.2 |
| liftsep8 | 1.24 | 19.70 (aspmc:sharpsat-td) | 15.9 |
| liftsep11 | >300 | - |  |
| liftsep16 | skip | - |  |
| liftsep23 | skip | - |  |
| liftsep32 | skip | - |  |
| mnist_sum192 | 5.90 | - |  |
| mnist_sum384 | 24.52 | - |  |
| mnist_sum768 | 143.32 | - |  |
| pedigree7_12 | 8.19 | - |  |
| pedigree14_10 | 10.14 | - |  |
| pedigree7_13 | 82.45 | - |  |
| pedigree7_14 | err | - |  |
| sickness45_40 | 0.09 | - |  |
| sickness180_40 | 87.36 | - |  |
| sickness240_40 | 274.76 | - |  |
| noisyor60_90 | 0.03 | - |  |
| noisyor120_180 | 0.13 | - |  |
| noisyor240_360 | 11.21 | - |  |
| noisyor360_540 | err | - |  |
| mastermind8_8_8 | 0.18 | - |  |
| mastermind16_5_8 | 4.15 | 3.00 (problog:sdd) | 0.7 |
| mastermind32_5_8 | 9.92 | 5.36 (problog:sdd) | 0.5 |
| mastermind64_5_8 | 8.89 | 20.04 (problog:sdd) | 2.3 |
| pdb80 | 0.01 | 4.18 (aspmc:sharpsat-td) | 20.9 |
| pdb160 | 0.42 | 16.53 (aspmc:sharpsat-td) | 38.9 |
| pdb320 | 2.39 | - |  |
| pdb640 | >300 | - |  |
| plcg4 | 0.01 | 0.57 (problog:sdd) | 2.8 |
| plcg6 | 0.08 | 4.40 (aspmc:sharpsat-td) | 22.0 |
| plcg8 | 3.77 | 97.30 (problog:sdd) | 25.8 |
| plcg10 | >300 | - |  |
| plcg12 | skip | - |  |
| plcg14 | skip | - |  |
| plcg16 | skip | - |  |
| problogtut_various_16_cplogic_1 | 0.01 | 0.09 (problog:sdd) | 0.5 |
| problogex_4_1_bayesian_net | 0.01 | 0.09 (problog:sdd) | 0.5 |
| cplint_pcfg | 0.01 | 0.12 (problog:sdd) | 0.6 |
| problogtut_various_13_incomplete_info_4 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_tutslides_02_more_3 | 0.03 | 0.15 (problog:ddnnf) | 0.8 |
| problogtut_basic_10_inhibitioneffects_3 | 0.01 | 0.09 (problog:fsdd) | 0.5 |
| problogex_tc_1 | 0.01 | 0.09 (problog:fsdd) | 0.5 |
| problogtut_basic_10_inhibitioneffects_7 | 0.01 | 0.12 (problog:sdd) | 0.6 |
| cplint_viral | 0.01 | 0.10 (problog:fsdd) | 0.5 |
| problogtut_basic_03_dice_4 | 0.01 | 0.11 (problog:fsdd) | 0.6 |
| problogtut_basic_09_higherorderfunctions_4 | 0.01 | 0.28 (problog:sdd) | 1.4 |
| problogtut_basic_08_rule_probs_7 | 0.01 | 0.12 (problog:sdd) | 0.6 |
| pcfgcount8 | 0.01 | 0.31 (problog:fsdd) | 1.6 |
| pcfgcount16 | 0.02 | 0.57 (pita:bdd) | 2.8 |
| pcfgcount24 | 0.04 | 0.71 (pita:bdd) | 3.6 |
| pcfgcount32 | 0.10 | 3.03 (aspmc:sharpsat-td) | 15.1 |
| pcfgcount48 | 0.41 | 3.27 (aspmc:sharpsat-td) | 8.0 |
| pcfgcount64 | 1.02 | 3.84 (aspmc:sharpsat-td) | 3.8 |
| pcfgcount96 | 6.04 | 9.16 (aspmc:sharpsat-td) | 1.5 |
| pcfgcount128 | 17.14 | 21.88 (aspmc:sharpsat-td) | 1.3 |
| problogtut_various_08_bayesian_dataflow_2 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogex_9_packing_problem | 0.01 | 0.12 (problog:fsdd) | 0.6 |
| problogtut_various_06_pcfg_4 | 0.02 | 0.27 (problog:sdd) | 1.4 |
| cplint_trigger | 0.00 | 0.09 (problog:sdd) | 0.4 |
| problogtut_various_16_cplogic_2 | 0.01 | 0.09 (problog:fsdd) | 0.5 |
| problogtut_tutslides_01_basic_3 | 0.01 | 0.09 (problog:sdd) | 0.5 |
| problogtut_tutslides_03_stochmem_1 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| cplint_mln | 0.01 | 0.10 (problog:fsdd) | 0.5 |
| problogtut_basic_03_dice_9 | 0.02 | 0.33 (problog:sdd) | 1.6 |
| cplint_monty | 0.01 | 0.14 (problog:sdd) | 0.7 |
| problogtut_various_02_bousi_2 | 0.01 | 0.11 (problog:sdd) | 0.5 |
| cplint_mendel | 0.01 | 0.11 (problog:sdd) | 0.5 |
| problogex_4_bayesian_net | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_tutslides_06_smart_students_2 | 0.01 | 0.12 (problog:sdd) | 0.6 |
| problogtut_basic_01_coins_3 | 0.01 | 0.09 (problog:sdd) | 0.5 |
| problogtut_various_08_bayesian_dataflow_10 | 0.01 | 0.11 (problog:fsdd) | 0.5 |
| problogtut_various_08_bayesian_dataflow_8 | 0.02 | 0.13 (problog:sdd) | 0.7 |
| problogtut_various_10_prism_6 | 0.02 | 0.19 (problog:fsdd) | 0.9 |
| problogtut_various_07_ppdatalog_7 | 0.03 | 0.14 (problog:sdd) | 0.7 |
| problogtut_various_17_taxonomy_1 | 0.01 | 0.20 (problog:sdd) | 1.0 |
| noisyor300_450 | 0.40 | - |  |
| problogtut_tutslides_01_basic_4 | 0.01 | 0.09 (problog:sdd) | 0.5 |
| cplint_pathdb | 0.01 | 0.11 (problog:sdd) | 0.5 |
| problogtut_basic_10_inhibitioneffects_9 | 0.01 | 0.13 (problog:fsdd) | 0.6 |
| cplint_coin2 | 0.00 | 0.10 (problog:sdd) | 0.5 |
| pgrid16_4 | 0.01 | 0.59 (problog:sdd) | 3.0 |
| pgrid16_5 | 0.01 | 0.71 (pita:bdd) | 3.6 |
| pgrid16_6 | 0.01 | 4.38 (pita:bdd) | 21.9 |
| pgrid16_7 | 0.02 | 6.27 (pita:bdd) | 31.3 |
| pgrid16_8 | 0.03 | - |  |
| pgrid16_10 | 0.05 | - |  |
| pgrid16_12 | 0.18 | - |  |
| pgrid16_15 | 1.85 | - |  |
| pgrid18_17 | 7.75 | - |  |
| pgrid20_19 | >300 | - |  |
| problogtut_various_14_robot_key_1 | 0.01 | 0.12 (problog:sdd) | 0.6 |
| cplint_eruption | 0.01 | 0.09 (problog:fsdd) | 0.5 |
| problogtut_basic_02_bayes_4 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_basic_09_higherorderfunctions_3 | 0.08 | 2.69 (problog:sdd) | 13.4 |
| problogtut_basic_10_inhibitioneffects_4 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogex_tc_2 | 0.01 | 0.09 (problog:sdd) | 0.5 |
| cplint_multiple_paths_simple | 0.01 | 0.09 (problog:sdd) | 0.5 |
| threedie10 | 0.02 | 0.19 (problog:fsdd) | 0.9 |
| threedie100 | 0.07 | 1.36 (problog:sdd) | 6.8 |
| threedie300 | 0.26 | 5.15 (problog:sdd) | 19.5 |
| threedie1000 | 1.92 | 274.00 (problog:sdd) | 143.0 |
| threedie1500 | 3.83 | - |  |
| problogtut_tutslides_02_more_4 | 0.01 | 0.12 (problog:sdd) | 0.6 |
| problogtut_various_13_incomplete_info_3 | 0.01 | 0.10 (problog:fsdd) | 0.5 |
| cplint_flexprob | 0.01 | 0.09 (problog:sdd) | 0.5 |
| problogtut_various_14_robot_key_2 | 0.01 | 0.12 (problog:sdd) | 0.6 |
| problogtut_various_10_prism_8 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_various_06_hmm_1 | 0.01 | 0.15 (problog:sdd) | 0.7 |
| problogtut_various_06_pcfg_3 | 0.01 | 0.14 (problog:sdd) | 0.7 |
| problogtut_basic_10_inhibitioneffects_12 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| cplint_dice | 0.01 | 0.14 (problog:ddnnf) | 0.7 |
| problogtut_various_08_bayesian_dataflow_5 | 0.01 | 0.13 (problog:sdd) | 0.6 |
| problogtut_tutslides_01_basic_2 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| cplint_alarm | 0.01 | 0.10 (problog:fsdd) | 0.5 |
| hwf3 | 0.01 | 0.49 (problog:sdd) | 2.5 |
| hwf5 | 0.16 | 1.20 (pita:bdd) | 6.0 |
| hwf7 | 11.46 | - |  |
| hwf9 | err | - |  |
| hwf11 | skip | - |  |
| problogtut_various_08_bayesian_dataflow_6 | 0.04 | 0.28 (problog:fsdd) | 1.4 |
| problogtut_various_07_ppdatalog_1 | 0.01 | 0.13 (problog:fsdd) | 0.6 |
| problogtut_various_10_causalpower_1 | 0.01 | 0.14 (problog:sdd) | 0.7 |
| problogtut_basic_10_inhibitioneffects_11 | 0.01 | 0.10 (problog:fsdd) | 0.5 |
| problogex_smokers_or | 0.01 | 0.11 (problog:sdd) | 0.5 |
| cplint_mendelc | 0.01 | 0.13 (problog:fsdd) | 0.7 |
| problogtut_basic_03_dice_8 | 0.01 | 0.16 (problog:ddnnf) | 0.8 |
| problogtut_various_06_hmm_2 | 0.01 | 0.17 (problog:sdd) | 0.8 |
| problogex_advars_smokers_alt | 0.01 | 0.11 (problog:sdd) | 0.5 |
| problogtut_basic_10_inhibitioneffects_2 | 0.01 | 0.10 (problog:fsdd) | 0.5 |
| problogtut_tutslides_02_more_2 | 0.01 | 0.09 (problog:sdd) | 0.5 |
| problogtut_basic_09_higherorderfunctions_5 | 0.01 | 0.22 (problog:sdd) | 1.1 |
| problogtut_basic_08_rule_probs_6 | 0.06 | 0.10 (problog:sdd) | 0.5 |
| problogtut_basic_03_dice_5 | 0.02 | 0.25 (problog:sdd) | 1.2 |
| problogtut_various_06_pcfg_5 | 0.01 | 0.31 (problog:sdd) | 1.6 |
| problogtut_various_08_bayesian_dataflow_3 | 0.01 | 0.13 (problog:sdd) | 0.7 |
| problogex_12_holidays | 0.01 | 0.12 (problog:sdd) | 0.6 |
| problogtut_various_11_semweb_1 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_basic_03_dice_2 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_various_03_crowds_1 | 0.01 | 0.24 (problog:sdd) | 1.2 |
| problogtut_basic_08_rule_probs_1 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_basic_09_higherorderfunctions_2 | 0.01 | 0.10 (problog:fsdd) | 0.5 |
| problogtut_various_08_bayesian_dataflow_13 | 0.01 | 0.11 (problog:sdd) | 0.5 |
| problogtut_various_01_montyhall_2 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_various_13_incomplete_info_2 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogex_tc_3 | 0.01 | 0.10 (problog:fsdd) | 0.5 |
| problogtut_basic_10_inhibitioneffects_5 | 0.01 | 0.11 (problog:sdd) | 0.5 |
| cplint_event_calculus | 0.01 | 0.10 (problog:fsdd) | 0.5 |
| problogex_7_probabilistic_graph | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_various_10_prism_5 | 0.01 | 0.11 (problog:sdd) | 0.5 |
| problogtut_various_07_ppdatalog_4 | 0.01 | 0.11 (problog:fsdd) | 0.5 |
| problogtut_various_08_bayesian_dataflow_4 | 0.01 | 0.11 (problog:sdd) | 0.6 |
| problogtut_various_12_pingpong_1 | 0.18 | 0.92 (pita:bdd) | 4.6 |
| problogtut_various_06_pcfg_2 | 0.03 | 0.54 (pita:bdd) | 2.7 |
| problogtut_various_08_bayesian_dataflow_11 | 0.01 | 1.85 (problog:sdd) | 9.2 |
| problogtut_various_08_bayesian_dataflow_9 | 0.01 | 0.16 (problog:ddnnf) | 0.8 |
| cplint_coin | 0.01 | 0.10 (problog:fsdd) | 0.5 |
| problogtut_various_07_ppdatalog_6 | 0.01 | 0.12 (problog:sdd) | 0.6 |
| problogtut_various_10_prism_7 | 0.01 | 0.53 (pita:bdd) | 2.6 |
| problogtut_tutslides_06_smart_students_3 | 0.01 | 0.12 (problog:sdd) | 0.6 |
| problogtut_basic_01_coins_2 | 0.01 | 0.09 (problog:fsdd) | 0.4 |
| cplint_testdb | 0.01 | 0.09 (problog:fsdd) | 0.5 |
| problogtut_various_04_nampally_1 | 0.01 | 0.19 (problog:sdd) | 0.9 |
| problogtut_tutslides_06_smart_students_1 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_basic_10_inhibitioneffects_8 | 0.01 | 0.12 (problog:sdd) | 0.6 |
| problogex_5_bayesian_net | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_basic_01_coins_1 | 0.00 | 0.09 (problog:sdd) | 0.4 |
| cplint_simpson | 0.01 | 0.10 (problog:fsdd) | 0.5 |
| problogtut_various_08_bayesian_dataflow_12 | 0.01 | 0.12 (problog:sdd) | 0.6 |
| problogtut_various_07_ppdatalog_5 | 0.01 | 0.11 (problog:sdd) | 0.5 |
| problogtut_basic_05_smokers_1 | 0.01 | 0.11 (problog:sdd) | 0.5 |
| problogtut_various_04_nampally_2 | 0.01 | 0.11 (problog:sdd) | 0.6 |
| problogex_swap | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_various_05_numberguessing_1 | err | 0.78 (problog:sdd) |  |
| problogtut_basic_09_higherorderfunctions_1 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_basic_08_rule_probs_2 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_basic_03_dice_1 | 0.01 | 0.09 (problog:sdd) | 0.5 |
| cplint_sneezing | 0.01 | 0.09 (problog:sdd) | 0.5 |
| problogtut_basic_10_inhibitioneffects_6 | 0.01 | 0.12 (problog:sdd) | 0.6 |
| problogtut_various_01_montyhall_1 | 0.01 | 0.11 (problog:sdd) | 0.5 |
| problogtut_advanced_10_urnballmodel_1 | 0.02 | 0.23 (problog:sdd) | 1.2 |
| problogtut_various_13_incomplete_info_1 | 0.01 | 0.16 (problog:sdd) | 0.8 |
| problogex_add | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_various_06_slp_1 | 0.04 | 0.57 (pita:bdd) | 2.9 |
| problogex_11_ads_numerical | 0.01 | 0.12 (problog:fsdd) | 0.6 |
| problogtut_various_06_pcfg_1 | 0.01 | 0.14 (problog:fsdd) | 0.7 |
| problogtut_various_06_hmm_3 | 0.03 | 0.62 (problog:ddnnf) | 3.1 |
| problogtut_various_10_prism_9 | 0.04 | 0.26 (problog:sdd) | 1.3 |
| cplint_light | 0.00 | 0.09 (problog:sdd) | 0.5 |
| problogtut_various_08_bayesian_dataflow_7 | 0.01 | 0.24 (problog:sdd) | 1.2 |
| problogtut_basic_10_inhibitioneffects_10 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_tutslides_02_more_1 | 0.01 | 0.12 (problog:sdd) | 0.6 |
| problogtut_basic_10_inhibitioneffects_1 | 0.00 | 0.09 (problog:sdd) | 0.5 |
| problogtut_basic_03_dice_6 | 0.01 | 0.12 (problog:sdd) | 0.6 |
| problogtut_basic_08_rule_probs_5 | 0.08 | 0.10 (problog:sdd) | 0.5 |
| hmmpos1 | 0.02 | 0.54 (pita:bdd) | 2.7 |
| hmmpos2 | 1.40 | 4.07 (pita:bdd) | 2.9 |
| hmmpos3 | 35.96 | - |  |
| hmmpos4 | >300 | - |  |
| hmmpos5 | skip | - |  |
| problogtut_various_15_spaghetti_1 | 0.01 | 0.16 (problog:sdd) | 0.8 |
| problogtut_various_04_nampally_3 | 0.06 | 10.91 (aspmc:sharpsat-td) | 54.6 |
| problogex_10_cards | 0.01 | 0.10 (problog:fsdd) | 0.5 |
| problogex_advars_smokers | 0.01 | 0.11 (problog:fsdd) | 0.6 |
| problogtut_tutslides_01_basic_1 | 0.01 | 0.09 (problog:fsdd) | 0.5 |
| problogtut_various_10_causalpower_2 | 0.01 | 0.17 (problog:sdd) | 0.8 |
| problogtut_various_07_ppdatalog_2 | 0.01 | 0.13 (problog:sdd) | 0.7 |
| problogtut_various_10_prism_3 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| cplint_markov_network | 0.04 | 0.12 (problog:sdd) | 0.6 |
| dnahmm4 | 0.01 | 0.16 (problog:sdd) | 0.8 |
| dnahmm8 | 0.15 | 1.11 (pita:bdd) | 5.5 |
| dnahmm12 | 6.87 | 233.69 (pita:bdd) | 34.0 |
| dnahmm16 | 236.68 | - |  |
| dnahmm20 | >300 | - |  |
| dnahmm24 | skip | - |  |
| dnahmm28 | skip | - |  |
| cplint_threesideddice | 0.01 | 0.12 (problog:fsdd) | 0.6 |
| problogex_negation | 0.01 | 0.09 (problog:fsdd) | 0.5 |
| problogtut_basic_08_rule_probs_8 | 0.07 | 0.10 (problog:fsdd) | 0.5 |
| problogtut_tutslides_04_bayesian_learning_1 | 0.07 | 0.34 (problog:fsdd) | 1.7 |
| cplint_markov_factor | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogex_8_smokers_network | 0.01 | 0.11 (problog:sdd) | 0.6 |
| problogex_generated | 0.01 | 0.15 (problog:sdd) | 0.8 |
| cplint_epidemic | 0.01 | 0.10 (problog:sdd) | 0.5 |
| cplint_jail | 0.01 | 0.13 (problog:sdd) | 0.7 |
| cplint_path | 0.01 | 0.10 (problog:fsdd) | 0.5 |
| problogtut_basic_08_rule_probs_3 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| pcfg8 | 0.01 | 0.17 (problog:sdd) | 0.9 |
| pcfg12 | 0.01 | 0.55 (pita:bdd) | 2.7 |
| pcfg16 | 0.03 | 0.64 (pita:bdd) | 3.2 |
| pcfg20 | 0.18 | 1.47 (pita:bdd) | 7.3 |
| pcfg24 | 1.54 | 4.41 (aspmc:sharpsat-td) | 2.9 |
| pcfg28 | 14.91 | 19.03 (aspmc:sharpsat-td) | 1.3 |
| pcfg32 | 92.69 | 101.82 (aspmc:sharpsat-td) | 1.1 |
| problogtut_various_09_airflap_1 | 0.02 | 0.77 (pita:bdd) | 3.8 |
| problogtut_various_07_ppdatalog_3 | 0.01 | 0.19 (problog:fsdd) | 0.9 |
| problogtut_tutslides_03_stochmem_2 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_various_08_bayesian_dataflow_14 | 0.01 | 0.14 (problog:sdd) | 0.7 |
| cplint_earthquake | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_basic_08_rule_probs_9 | 0.01 | 0.11 (problog:sdd) | 0.6 |
| problogex_6_hmm_weather | 0.01 | 0.15 (problog:ddnnf) | 0.7 |
| cplint_multiple_paths | 0.00 | 0.10 (problog:sdd) | 0.5 |
| problogtut_various_02_bousi_1 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogtut_basic_08_rule_probs_4 | 0.01 | 0.10 (problog:fsdd) | 0.5 |
| cplint_bag_game | 0.01 | 0.09 (problog:sdd) | 0.5 |
| problogtut_tutslides_05_complexity_1 | 0.01 | 0.10 (problog:sdd) | 0.5 |
| problogex_coin | 0.01 | 0.09 (problog:sdd) | 0.5 |
| problogtut_basic_03_dice_7 | 0.01 | 0.12 (problog:sdd) | 0.6 |
| problogtut_various_08_bayesian_dataflow_1 | 0.01 | 0.09 (problog:sdd) | 0.5 |
| problogex_3_tossing_coin | 0.01 | 0.09 (problog:fsdd) | 0.5 |
| cplint_persons | 0.01 | 0.15 (problog:sdd) | 0.8 |
| cplint_markov_chaindb | 0.01 | 0.23 (problog:sdd) | 1.1 |
| cplint_var_objdb | 0.01 | 0.10 (problog:sdd) | 0.5 |
| cplint_pcfglrdb | 0.01 | 0.23 (problog:fsdd) | 1.1 |
| cplint_coinmsw | 0.01 | 0.10 (problog:fsdd) | 0.5 |
| cplint_swish_sneezing_simple | 0.01 | 0.08 (problog:fsdd) | 0.4 |
| cplint_swish_admission | 0.00 | 0.08 (problog:fsdd) | 0.4 |
| cplint_swish_smoking | 0.01 | 0.09 (problog:fsdd) | 0.4 |
| cplint_swish_viraltab | 0.01 | 0.09 (problog:sdd) | 0.5 |

### Ace's suite (46 instances, 600 s)

| instance | tddlog 0.1.0 | fastest exact baseline | ratio | merlin:bte | agrum:jtree | pgmpy:ve | libdai:jtree | tensorinference:treesa | tensorinference:greedy |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| ace_water | 0.06 | 0.02 (libdai:jtree) | 0.1 | 0.17 | 0.13 | 1.48 | 0.02 | - | - |
| ace_fs_01 | 0.03 | 0.00 (libdai:jtree) | 0.0 | 0.09 | 0.14 | 1.46 | 0.00 | - | - |
| ace_mastermind_03_08_03 | 0.05 | 0.34 (merlin:bte) | 1.7 | 0.34 | 0.50 | >600 | 0.97 | - | - |
| ace_blockmap_05_01 | 0.04 | 0.80 (merlin:bte) | 4.0 | 0.80 | 1.95 | >600 | 1.20 | - | - |
| ace_fs_04 | 0.03 | 0.01 (libdai:jtree) | 0.1 | 0.02 | 0.17 | 1.85 | 0.01 | - | - |
| ace_mildew | 0.19 | 0.22 (agrum:jtree) | 1.1 | 1.58 | 0.22 | 1.54 | 0.44 | - | - |
| ace_munin1 | 0.13 | 0.16 (merlin:bte) | 0.8 | 0.16 | 0.18 | 12.62 | 0.17 | - | - |
| ace_fs_07 | 0.04 | 2.40 (ace:auto) | 12.0 | err | err | >600 | err | err | >600 |
| ace_alarm | 0.05 | 0.01 (libdai:jtree) | 0.0 | 0.01 | 0.12 | 1.40 | 0.01 | - | - |
| ace_hailfinder | 0.05 | 0.01 (libdai:jtree) | 0.0 | 0.01 | 0.12 | 1.43 | 0.01 | - | - |
| ace_munin2 | 0.17 | 0.08 (merlin:bte) | 0.4 | 0.08 | 0.29 | 7.41 | 0.09 | - | - |
| ace_pathfinder | 0.25 | 0.06 (libdai:jtree) | 0.2 | 0.06 | 0.23 | 1.81 | 0.06 | - | - |
| ace_students_03_02 | 0.04 | 0.04 (merlin:bte) | 0.2 | 0.04 | 0.17 | 1.55 | 0.06 | - | - |
| ace_munin3 | 0.38 | 0.20 (merlin:bte) | 0.5 | 0.20 | 0.38 | err | 0.38 | - | - |
| ace_diabetes | 0.36 | 0.16 (libdai:jtree) | 0.4 | 0.17 | 0.24 | 1.52 | 0.16 | - | - |
| ace_pigs | 0.07 | 0.04 (merlin:bte) | 0.2 | 0.04 | 0.18 | 3.94 | 0.04 | - | - |
| ace_blockmap_05_02 | 0.02 | 0.05 (merlin:bte) | 0.2 | 0.05 | 0.24 | 8.52 | 0.06 | - | - |
| ace_munin4 | 0.76 | 0.94 (agrum:jtree) | 1.2 | 2.33 | 0.94 | >600 | 3.49 | - | - |
| ace_blockmap_05_03 | 0.05 | 0.07 (merlin:bte) | 0.4 | 0.07 | 0.29 | 4.63 | 0.15 | - | - |
| ace_blockmap_10_01 | 0.07 | 4.04 (ace:auto) | 20.2 | err | err | >600 | err | >600 | err |
| ace_mastermind_04_08_03 | 0.06 | 1.27 (agrum:jtree) | 6.3 | 1.33 | 1.27 | >600 | 2.31 | - | - |
| ace_fs_10 | 0.10 | 4.06 (ace:forceC2d) | 20.3 | err | err | err | err | >600 | >600 |
| ace_fs_13 | 0.19 | 7.70 (ace:forceC2d) | 38.5 | err | err | err | err | >600 | >600 |
| ace_mastermind_05_08_03 | 0.06 | 0.10 (merlin:bte) | 0.5 | 0.10 | 0.49 | 16.98 | 0.25 | - | - |
| ace_fs_16 | 0.36 | 12.86 (ace:forceC2d) | 35.9 | err | err | err | err | >600 | >600 |
| ace_fs_19 | 0.58 | 19.70 (ace:auto) | 33.7 | err | err | err | err | >600 | err |
| ace_fs_22 | 0.89 | 35.40 (ace:auto) | 39.8 | err | err | err | err | >600 | err |
| ace_fs_25 | 1.32 | 58.63 (ace:auto) | 44.4 | err | >600 | >600 | >600 | >600 | err |
| ace_fs_28 | 1.82 | 105.74 (ace:auto) | 58.0 | err | >600 | >600 | >600 | >600 | err |
| ace_mastermind_06_08_03 | 0.06 | 1.53 (agrum:jtree) | 7.7 | 2.45 | 1.53 | >600 | 3.15 | - | - |
| ace_blockmap_10_02 | 0.09 | 0.72 (merlin:bte) | 3.6 | 0.72 | 5.63 | 79.67 | 3.10 | >600 | >600 |
| ace_blockmap_10_03 | 0.26 | 7.87 (ace:forceC2d) | 30.6 | err | err | >600 | err | >600 | err |
| ace_fs_29 | 2.02 | 125.26 (ace:auto) | 62.1 | err | >600 | >600 | >600 | >600 | >600 |
| ace_blockmap_15_01 | 0.21 | 11.48 (ace:forceC2d) | 54.9 | 57.81 | 71.77 | >600 | 331.76 | >600 | >600 |
| ace_blockmap_15_02 | 0.23 | 6.99 (merlin:bte) | 30.9 | 6.99 | 78.39 | >600 | 83.87 | >600 | >600 |
| ace_blockmap_15_03 | 0.60 | 19.42 (ace:forceC2d) | 32.4 | err | err | >600 | >600 | >600 | err |
| ace_blockmap_20_01 | 0.47 | 28.32 (ace:forceC2d) | 60.0 | >600 | >600 | >600 | >600 | >600 | err |
| ace_blockmap_20_02 | 0.50 | 31.18 (ace:forceC2d) | 62.7 | err | >600 | >600 | >600 | >600 | err |
| ace_blockmap_20_03 | 0.55 | 32.61 (ace:forceC2d) | 59.2 | 104.49 | >600 | >600 | >600 | >600 | >600 |
| ace_blockmap_22_01 | 0.67 | 44.21 (ace:forceC2d) | 66.5 | err | >600 | >600 | >600 | >600 | >600 |
| ace_blockmap_22_02 | 0.68 | 47.82 (ace:forceC2d) | 70.6 | >600 | >600 | >600 | >600 | >600 | >600 |
| ace_blockmap_22_03 | 0.71 | 49.45 (ace:forceC2d) | 70.0 | 253.54 | >600 | >600 | >600 | >600 | >600 |
| ace_mastermind_03_08_04 | 0.04 | 0.14 (merlin:bte) | 0.7 | 0.14 | 0.76 | 9.74 | 0.35 | - | - |
| ace_mastermind_04_08_04 | 0.07 | 0.39 (merlin:bte) | 2.0 | 0.39 | 1.18 | >600 | 1.72 | >600 | >600 |
| ace_mastermind_10_08_03 | 0.09 | 0.55 (merlin:bte) | 2.7 | 0.55 | 1.14 | >600 | 1.01 | >600 | err |
| ace_mastermind_03_08_05 | 0.10 | 0.62 (merlin:bte) | 3.1 | 0.62 | 2.18 | err | 3.00 | >600 | >600 |

### Large networks (14 instances, 600 s)

| instance | tddlog 0.1.0 | fastest exact baseline | ratio | merlin:bte | agrum:jtree | pgmpy:ve | libdai:jtree | tensorinference:treesa | tensorinference:greedy |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| bn_barley_ev | 1.84 | 0.52 (agrum:jtree) | 0.3 | 1.41 | 0.52 | 1.80 | 1.50 | - | - |
| bn_barley | 2.19 | 0.77 (agrum:jtree) | 0.4 | 1.57 | 0.77 | 6.63 | 1.85 | - | - |
| bn_diabetes_ev | 1.00 | 0.26 (agrum:jtree) | 0.3 | 0.86 | 0.26 | 1.49 | 0.94 | - | - |
| bn_diabetes | 1.00 | 0.28 (agrum:jtree) | 0.3 | 0.89 | 0.28 | 1.54 | 0.96 | - | - |
| bn_mildew_ev | 1.71 | 0.45 (libdai:jtree) | 0.3 | 1.47 | 0.45 | 1.59 | 0.45 | - | - |
| bn_mildew | 1.76 | 0.31 (agrum:jtree) | 0.2 | 1.46 | 0.31 | 1.43 | 0.44 | - | - |
| bn_munin1 | 1.02 | 11.01 (agrum:jtree) | 10.8 | 26.36 | 11.01 | err | 50.61 | - | - |
| bn_munin1_ev | 0.92 | 12.28 (agrum:jtree) | 13.3 | 25.01 | 12.28 | >600 | 44.01 | - | - |
| bn_munin2 | 0.52 | 0.36 (merlin:bte) | 0.7 | 0.36 | 0.44 | 44.88 | 0.63 | - | - |
| bn_munin3 | 0.56 | 0.43 (merlin:bte) | 0.8 | 0.43 | 0.45 | >600 | 0.76 | - | - |
| bn_munin4 | 0.75 | 0.98 (agrum:jtree) | 1.3 | 2.29 | 0.98 | >600 | 3.16 | - | - |
| bn_munin2_ev | 0.62 | 0.35 (merlin:bte) | 0.6 | 0.35 | 0.41 | 20.39 | 0.58 | - | - |
| bn_munin3_ev | 0.58 | 0.41 (agrum:jtree) | 0.7 | 0.41 | 0.41 | err | 0.71 | - | - |
| bn_munin4_ev | 0.77 | 0.83 (agrum:jtree) | 1.1 | 2.21 | 0.83 | >600 | 3.20 | - | - |

### Against each baseline

For each baseline, in total time as the README compares two systems: the
instances given to both, how many each answers, each one's total time
over those either answers (a run that does not answer counted at the
suite's time limit), the two totals over those both answer, and the
instances on which tddlog is slower (both under 0.2 s counted for
neither). TensorInference.jl's row is its faster contraction order on
each instance; its two orders follow.

**Standard suite: networks**

| baseline | given | baseline answers | tddlog answers | baseline s | tddlog s | both answer | baseline s, both | tddlog s, both | tddlog slower |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Ace | 34 | 34 | 34 | 107.9 | 5.0 | 34 | 107.9 | 5.0 | 0 |
| Merlin | 34 | 34 | 34 | 64.5 | 5.0 | 34 | 64.5 | 5.0 | 2 |
| pyAgrum | 34 | 34 | 34 | 99.3 | 5.0 | 34 | 99.3 | 5.0 | 2 |
| pgmpy | 34 | 30 | 34 | 1 308 | 5.0 | 30 | 108.2 | 2.3 | 0 |
| libDAI | 34 | 34 | 34 | 15.7 | 5.0 | 34 | 15.7 | 5.0 | 2 |
| TensorInference.jl | 34 | 34 | 34 | 347.8 | 5.0 | 34 | 347.8 | 5.0 | 2 |
| TensorInference.jl treesa | 34 | 34 | 34 | 546.4 | 5.0 | 34 | 546.4 | 5.0 | 0 |
| TensorInference.jl greedy | 34 | 30 | 34 | 1 204 | 5.0 | 30 | 3.6 | 2.3 | 2 |
| ProbLog | 34 | 17 | 34 | 5 712 | 5.0 | 17 | 611.6 | 0.7 | 0 |
| PITA | 34 | 18 | 34 | 5 087 | 5.0 | 18 | 287.4 | 1.6 | 0 |
| aspmc | 34 | 23 | 34 | 5 344 | 5.0 | 23 | 2 044 | 1.1 | 0 |
| Dice | 34 | 31 | 34 | 1 677 | 5.0 | 31 | 777.2 | 3.8 | 0 |
| Alea.jl | 34 | 31 | 34 | 1 147 | 5.0 | 31 | 246.8 | 3.8 | 0 |
| Roulette | 34 | 28 | 34 | 2 144 | 6.5 | 28 | 344.2 | 2.9 | 0 |
| noDice | 34 | 27 | 34 | 3 095 | 5.0 | 27 | 995.2 | 2.8 | 0 |
| Psi | 34 | 8 | 34 | 7 820 | 5.0 | 8 | 20.4 | 0.3 | 0 |
| Genfer | 34 | 25 | 34 | 3 200 | 5.0 | 25 | 500.2 | 1.3 | 0 |
| SPPL | 34 | 12 | 34 | 6 627 | 5.0 | 12 | 27.4 | 0.5 | 0 |
| Geni | 34 | 20 | 34 | 4 331 | 5.0 | 20 | 131.0 | 1.0 | 0 |
| Pluck | 34 | 24 | 34 | 3 685 | 5.0 | 24 | 684.6 | 1.2 | 0 |
| Storm | 34 | 28 | 34 | 2 631 | 5.0 | 28 | 830.8 | 2.2 | 0 |
| PRISM | 34 | 27 | 34 | 2 917 | 5.0 | 27 | 816.9 | 1.4 | 0 |

**Ace's networks**

| baseline | given | baseline answers | tddlog answers | baseline s | tddlog s | both answer | baseline s, both | tddlog s, both | tddlog slower |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Ace | 46 | 46 | 46 | 495.8 | 15.6 | 46 | 495.8 | 15.6 | 0 |
| Merlin | 46 | 30 | 46 | 10 035 | 15.6 | 30 | 435.4 | 5.0 | 3 |
| pyAgrum | 46 | 28 | 46 | 10 971 | 15.6 | 28 | 170.8 | 3.7 | 3 |
| pgmpy | 46 | 17 | 46 | 17 558 | 15.6 | 17 | 157.5 | 1.7 | 0 |
| libDAI | 46 | 28 | 46 | 11 238 | 15.6 | 28 | 437.8 | 3.7 | 3 |
| TensorInference.jl | 46 | 22 | 46 | 16 366 | 15.6 | 22 | 1 966 | 3.0 | 1 |
| TensorInference.jl treesa | 46 | 22 | 46 | 16 619 | 15.6 | 22 | 2 219 | 3.0 | 0 |
| TensorInference.jl greedy | 46 | 11 | 46 | 21 002 | 15.6 | 11 | 1.7 | 1.3 | 1 |
| ProbLog | 46 | 13 | 46 | 20 125 | 15.6 | 13 | 325.5 | 1.8 | 0 |
| PITA | 46 | 14 | 46 | 20 227 | 15.6 | 14 | 1 027 | 1.8 | 0 |
| aspmc | 46 | 10 | 46 | 22 373 | 15.6 | 10 | 772.5 | 0.4 | 0 |
| Dice | 43 | 14 | 43 | 18 544 | 10.4 | 14 | 1 144 | 1.4 | 0 |
| Alea.jl | 43 | 34 | 43 | 6 375 | 10.4 | 34 | 975.3 | 6.6 | 0 |
| Roulette | 46 | 29 | 46 | 10 935 | 20.4 | 29 | 734.8 | 5.5 | 0 |
| noDice | 43 | 11 | 43 | 20 956 | 10.4 | 11 | 1 756 | 0.7 | 0 |
| Psi | 43 | 1 | 43 | 25 241 | 10.4 | 1 | 40.9 | 0.0 | 0 |
| Genfer | 43 | 4 | 43 | 23 601 | 10.4 | 4 | 201.1 | 0.2 | 0 |
| SPPL | 43 | 21 | 43 | 15 133 | 10.4 | 21 | 1 933 | 1.3 | 0 |
| Geni | 43 | 3 | 43 | 24 024 | 10.4 | 3 | 24.0 | 0.1 | 0 |
| Pluck | 46 | 25 | 46 | 14 204 | 15.2 | 25 | 1 604 | 2.6 | 0 |
| Storm | 46 | 7 | 46 | 23 890 | 15.6 | 7 | 489.9 | 0.3 | 0 |
| PRISM | 46 | 9 | 46 | 23 246 | 15.6 | 9 | 1 046 | 0.5 | 0 |

**Large networks**

| baseline | given | baseline answers | tddlog answers | baseline s | tddlog s | both answer | baseline s, both | tddlog s, both | tddlog slower |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Ace | 14 | 14 | 14 | 134.7 | 15.2 | 14 | 134.7 | 15.2 | 0 |
| Merlin | 14 | 14 | 14 | 65.1 | 15.2 | 14 | 65.1 | 15.2 | 10 |
| pyAgrum | 14 | 14 | 14 | 29.4 | 15.2 | 14 | 29.4 | 15.2 | 10 |
| pgmpy | 14 | 8 | 14 | 3 680 | 15.2 | 8 | 79.7 | 10.6 | 3 |
| libDAI | 14 | 14 | 14 | 109.8 | 15.2 | 14 | 109.8 | 15.2 | 7 |
| TensorInference.jl | 14 | 14 | 14 | 650.6 | 15.2 | 14 | 650.6 | 15.2 | 4 |
| TensorInference.jl treesa | 14 | 14 | 14 | 665.6 | 15.2 | 14 | 665.6 | 15.2 | 0 |
| TensorInference.jl greedy | 14 | 6 | 14 | 5 381 | 15.2 | 6 | 580.6 | 9.4 | 4 |
| ProbLog | 14 | 0 | 14 | 8 400 | 15.2 | 0 | 0.0 | 0.0 | 0 |
| PITA | 14 | 0 | 14 | 8 400 | 15.2 | 0 | 0.0 | 0.0 | 0 |
| aspmc | 14 | 0 | 14 | 8 400 | 15.2 | 0 | 0.0 | 0.0 | 0 |
| Dice | 14 | 3 | 14 | 7 624 | 15.2 | 3 | 1 024 | 1.8 | 0 |
| Alea.jl | 14 | 3 | 14 | 7 112 | 15.2 | 3 | 512.1 | 1.8 | 0 |
| Roulette | 14 | 0 | 14 | 8 400 | 19.8 | 0 | 0.0 | 0.0 | 0 |
| noDice | 14 | 0 | 14 | 8 400 | 15.2 | 0 | 0.0 | 0.0 | 0 |
| Psi | 14 | 0 | 14 | 8 400 | 15.2 | 0 | 0.0 | 0.0 | 0 |
| Genfer | 14 | 0 | 14 | 8 400 | 15.2 | 0 | 0.0 | 0.0 | 0 |
| SPPL | 14 | 0 | 14 | 8 400 | 15.2 | 0 | 0.0 | 0.0 | 0 |
| Geni | 14 | 0 | 14 | 8 400 | 15.2 | 0 | 0.0 | 0.0 | 0 |
| Pluck | 14 | 0 | 14 | 8 400 | 16.9 | 0 | 0.0 | 0.0 | 0 |
| Storm | 14 | 0 | 14 | 8 400 | 15.2 | 0 | 0.0 | 0.0 | 0 |
| PRISM | 14 | 0 | 14 | 8 400 | 15.2 | 0 | 0.0 | 0.0 | 0 |

**Standard suite: programs**

| baseline | given | baseline answers | tddlog answers | baseline s | tddlog s | both answer | baseline s, both | tddlog s, both | tddlog slower |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| ProbLog | 71 | 50 | 66 | 5 453 | 176.1 | 50 | 652.8 | 2.8 | 0 |
| PITA | 71 | 52 | 66 | 4 674 | 176.1 | 52 | 474.2 | 4.2 | 0 |
| aspmc | 71 | 35 | 66 | 10 057 | 176.1 | 35 | 757.1 | 19.7 | 0 |
| Ace | 71 | 49 | 66 | 5 543 | 176.1 | 49 | 443.1 | 4.2 | 0 |
| Merlin | 71 | 44 | 66 | 6 757 | 176.1 | 44 | 157.4 | 2.0 | 0 |
| pyAgrum | 71 | 44 | 66 | 7 050 | 176.1 | 44 | 449.5 | 2.0 | 0 |
| pgmpy | 71 | 40 | 66 | 8 565 | 476.1 | 39 | 308.3 | 3.8 | 0 |
| libDAI | 71 | 44 | 66 | 7 141 | 176.1 | 44 | 540.7 | 2.3 | 0 |
| TensorInference.jl | 71 | 38 | 66 | 8 734 | 176.1 | 38 | 334.0 | 3.7 | 0 |
| TensorInference.jl treesa | 71 | 35 | 66 | 10 923 | 176.1 | 35 | 1 623 | 0.6 | 0 |
| TensorInference.jl greedy | 71 | 37 | 66 | 8 836 | 176.1 | 37 | 135.8 | 3.7 | 0 |
| Dice | 71 | 43 | 66 | 8 154 | 176.1 | 43 | 1 254 | 3.0 | 0 |
| Alea.jl | 71 | 49 | 66 | 5 205 | 176.1 | 49 | 105.4 | 3.3 | 0 |
| Roulette | 71 | 49 | 66 | 5 339 | 186.2 | 49 | 238.6 | 2.8 | 0 |
| noDice | 71 | 38 | 66 | 9 093 | 176.1 | 38 | 692.8 | 3.3 | 0 |
| Psi | 71 | 9 | 66 | 17 384 | 176.1 | 9 | 284.5 | 0.1 | 0 |
| Genfer | 71 | 29 | 66 | 11 465 | 176.1 | 29 | 365.0 | 3.2 | 1 |
| SPPL | 71 | 12 | 66 | 16 381 | 176.1 | 12 | 181.1 | 0.1 | 0 |
| Geni | 71 | 24 | 66 | 12 885 | 176.1 | 24 | 285.4 | 1.7 | 0 |
| Pluck | 71 | 44 | 66 | 7 359 | 176.1 | 44 | 759.2 | 3.2 | 0 |
| Storm | 71 | 38 | 66 | 9 741 | 176.1 | 38 | 1 341 | 1.3 | 0 |
| PRISM | 71 | 30 | 66 | 11 727 | 176.1 | 30 | 927.0 | 0.5 | 0 |

**Literature suite**

| baseline | given | baseline answers | tddlog answers | baseline s | tddlog s | both answer | baseline s, both | tddlog s, both | tddlog slower |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| ProbLog | 328 | 232 | 303 | 22 943 | 1 960 | 231 | 1 342 | 54.2 | 2 |
| PITA | 328 | 191 | 303 | 35 387 | 1 660 | 191 | 1 787 | 29.3 | 0 |
| aspmc | 328 | 206 | 303 | 30 600 | 1 660 | 206 | 1 500 | 143.9 | 0 |
| Ace | 98 | 38 | 86 | 15 369 | 1 152 | 38 | 969.2 | 18.9 | 0 |
| Merlin | 98 | 23 | 86 | 19 528 | 1 452 | 22 | 264.6 | 1.9 | 0 |
| pyAgrum | 98 | 22 | 86 | 19 911 | 1 452 | 21 | 142.2 | 2.0 | 0 |
| pgmpy | 98 | 15 | 86 | 21 761 | 1 452 | 14 | 151.1 | 1.6 | 0 |
| libDAI | 98 | 24 | 86 | 19 261 | 1 452 | 23 | 353.9 | 2.0 | 0 |
| TensorInference.jl | 98 | 21 | 86 | 19 807 | 1 150 | 21 | 306.9 | 1.4 | 0 |
| TensorInference.jl treesa | 98 | 20 | 86 | 20 659 | 1 150 | 20 | 858.6 | 1.0 | 0 |
| TensorInference.jl greedy | 98 | 18 | 86 | 20 473 | 1 150 | 18 | 72.9 | 1.3 | 0 |
| Dice | 98 | 17 | 86 | 21 374 | 1 124 | 17 | 673.6 | 7.5 | 0 |
| Alea.jl | 98 | 32 | 86 | 16 292 | 1 124 | 32 | 92.3 | 38.4 | 4 |
| Roulette | 98 | 33 | 86 | 16 414 | 1 141 | 33 | 514.3 | 69.2 | 2 |
| noDice | 98 | 11 | 86 | 23 027 | 1 124 | 11 | 527.3 | 0.6 | 0 |
| Psi | 98 | 0 | 86 | 25 800 | 1 124 | 0 | 0.0 | 0.0 | 0 |
| Genfer | 98 | 1 | 86 | 25 500 | 1 124 | 1 | 0.3 | 0.1 | 0 |
| SPPL | 98 | 5 | 86 | 24 740 | 1 124 | 5 | 440.5 | 0.1 | 0 |
| Geni | 98 | 0 | 86 | 25 800 | 1 124 | 0 | 0.0 | 0.0 | 0 |
| Pluck | 98 | 23 | 86 | 19 332 | 1 175 | 23 | 432.1 | 7.8 | 0 |
| Storm | 98 | 16 | 86 | 21 927 | 1 183 | 16 | 927.2 | 0.6 | 0 |
| PRISM | 98 | 8 | 86 | 23 653 | 1 183 | 8 | 253.2 | 0.2 | 0 |

## The literature suite

Families from published evaluations and the systems' own example
programs (`--suite lit`). The ten families of the first scans below are
grown on a random structure from a seed, all but taint. The generators are
written for this repository; the copied material is cplint's programs
(`data/cplint`, Artistic License 2.0), cplint on SWISH's (`data/swish`,
Simplified BSD License), the taint analyses of PSouffle's
evaluation (`data/psouffle/taint`, MIT), ProbLog's example programs
(`data/problog`, Apache License 2.0) and DeepProbLog's handwritten-formula
model (`data/deepproblog`, Apache License 2.0). The ProbLog tutorial's
pages state no licence: `data/fetch.txt` fetches them, and their programs
are not shown.

The later scans
added the systems' own programs and the programs with lists and function
symbols that tddlog's grounder reads:

- **problogtut** `problogtut_<page>_<k>` (`gen_problogtut.py`): the
  programs of the ProbLog tutorial's pages that ask marginals, as the
  pages write them, each its own queries.
- **problogex** `problogex_<name>` (`gen_problogex.py`): the example
  programs of ProbLog's repository (`test/`), each with the answers its
  comment states.
- **cplintpita** `cplint_<name>` (`gen_cplint.py`): cplint's examples
  written for PITA, LPADs that tddlog reads itself and `tddlog --program`
  converts to ProbLog for the other systems; `cplint_swish_<name>` are
  cplint on SWISH's examples that are not copies of these.
- **threedie** `threedie<n>` (`gen_threedie.py`): PITA's three-sided die
  thrown until it shows 3 (Riguzzi and Swift, ICLP 2010, Fig. 3), the
  probability that throw n shows 1.
- **pcfg**, **pcfgcount** `pcfg<n>`, `pcfgcount<n>` (`gen_pcfg.py`): the
  ProbLog tutorial's probabilistic context-free grammar, each choice
  identified by the derivation's stack (pcfg) or by its nonterminal's
  count of expansions (pcfgcount), asked how the derivation of a balanced
  word of length n starts, given the word.
- **dnahmm** `dnahmm<n>` (`gen_dnahmm.py`): cplint's HMM of DNA sequences
  (Christiansen and Gallagher, ICLP 2009; PITA's evaluation, Riguzzi and
  Swift, TPLP 2011), each choice indexed by the states before it, the
  probability of a sequence of length n.
- **plcg** `plcg<n>` (`gen_plcg.py`): cplint's probabilistic left-corner
  grammar (Sato, Kameya and Kurihara, AMAI 2008), the probability of a
  word of n letters.
- **hmmpos** `hmmpos<k>` (`gen_hmmpos.py`): cplint's part-of-speech HMM,
  the probability of its example's sentence said k times.
- **hwf** `hwf<n>` (`gen_hwf.py`): DeepProbLog's handwritten formulas
  (Manhaeve et al., Artificial Intelligence 2021), the probability of the
  label of a formula of n symbols.
- **pgrid** `pgrid<n>_<d>` (`gen_pgrid.py`): ProbLog2's probabilistic grid
  (Fierens et al., TPLP 2015, Section 9), the probability of a path to
  the far corner of an n x n grid from the node d steps before it on the
  diagonal.

The reference of problogtut and cplintpita is ProbLog, one query at a
time (`scripts/problog_each.py`, `exact:problog1`); problogex's is each
program's stated answers; the others' is `scripts/lit_exact.py`.

The first scans' families:

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
- **eventcalc** `eventcalc4_<t>` (`gen_eventcalc.py`): Prob-EC's event
  calculus (Skarlatidis, Artikis, Filippou, Paliouras, TPLP 2015) on a
  simulated scene of four people over t frames: inertia through time,
  with negated events and terminations; both fluents, meeting and moving,
  queried for every ordered pair every 10 frames.
- **taint** `taint_<app>` (`gen_taint.py`): the points-to and taint
  analyses of 15 Android apps of PSouffle's evaluation (Li, Xia, Adnan,
  Wang, FMCAD 2026): 104 probabilistic rules, mutually recursive, over
  8,000 to 57,000 facts, every derived tuple of 30 query patterns queried.
- **liftsep** `liftsep<n>` (`gen_liftsep.py`): the safe queries QW and Q9
  of Beame, Li, Roy, Suciu (TODS 2017, Example 3.6) on a complete domain
  of n constants, which lifted inference answers in polynomial time and
  whose lineage has no decision-DNNF smaller than 2^Omega(sqrt n).
- **rgraph** `rgraph<n>` (`gen_rgraph.py`): the path of two edges of Fink,
  Huang, Olteanu (VLDB J. 2013, Section 5), a #P-hard self-join, on a
  random subgraph of the n-clique.

The reference, `scripts/lit_exact.py` (`exact:lit`), shares nothing with
tddlog: convolution (mnist_sum), clique-tree calibration (pedigree,
noisyor), forward filtering over the 2^n states of a step (sickness) and
over the 16 joint states of a pair (eventcalc), enumeration of the codes
the feedback admits (mastermind), variable elimination of a pairwise model
(pdb), inclusion-exclusion over the lineage's clauses (liftsep) and a sum
of the matchings' weights (rgraph). It agrees with ProbLog on a small
instance of each of the first six families (`tests/lit/`,
`tests/test_lit.py`), on which `tests/examples.rs` checks tddlog, and of
eventcalc, liftsep and rgraph (`tests/test_litscan.py`), and bounds the
instance sizes (the pedigree clique tree does not fit in 30 GB from
pedigree8_8 on). Taint has no reference: there tddlog's answers are
checked against PSouffle's, the evaluation's own system.
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
marginals given the evidence. The results by type of program count a
network as Bayesian when its factors are conditional probability tables
over an acyclic graph: the three Pedigree networks are, and Promedus's
files, in which two factors end in the same variable, count as Markov
networks with the others. The baselines are Merlin (`--merlin`,
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

Ihler's UAI model collection (<https://ics.uci.edu/~ihler/uai-data/>,
`data/uai-ihler/README.md`; `python3 scripts/bench.py fetch` downloads
it) gives a suite of its own, uaiihler: its groups BN (the UAI 2006
evaluation's Bayesian networks), classics, Grids (the 20 x 20 Ising
grids with a wrapped border), pedigree and promedas (noisy-or chains),
each read as `--suite uai --uai-benchmarks data/uai-ihler/GROUP`, less
the networks whose file is one of the UAI 2022 sets' by SHA-256. They
are asked the partition function as the PR sets are, by the same
systems: the network tools on the UAI file, Ace on the Hugin file, the
ProbLog family and aspmc on the program, and the clique tree, Ganak and
sharpSAT-TD as references.

### Results: the marginals

The MAR task, in one job on one lab machine (16 physical cores and 125
GB), 2026-10-20 to 2026-10-21 (300 s a run). tddlog faster than Merlin
counts the instances where tddlog is faster than Merlin or answers
alone, not those both finish in under 0.2 s; seconds are each one's
total over the instances either answers, a run that does not answer
counted at the limit.

| family | instances | tddlog | Merlin | tddlog faster than Merlin | tddlog s | Merlin s |
|---|---:|---:|---:|---:|---:|---:|
| Alchemy | 1 | 1 | 0 | 1 | 0.3 | 300.0 |
| CSP | 3 | 3 | 3 | 2 | 8.7 | 63.8 |
| DBN | 6 | 6 | 6 | 6 | 13.2 | 136.9 |
| Grids | 8 | 8 | 2 | 7 | 12.7 | 1 804 |
| ObjectDetection | 65 | 65 | 65 | 65 | 303.4 | 3 157 |
| Pedigree | 3 | 3 | 3 | 3 | 0.2 | 5.8 |
| Promedus | 28 | 28 | 28 | 15 | 2.6 | 504.9 |
| Segmentation | 6 | 6 | 6 | 6 | 1.3 | 4.0 |
| all | 120 | 120 | 113 | 105 | 342.4 | 5 977 |

Merlin prints `-nan` where the partition function overflows a double
(Alchemy_11, Grids_13, 14, 17) and fails on Grids_15, 16 and 18. The two
both take under 0.2 s on 14 of Merlin's 113 answers; tddlog is faster on
98 of the other 99 (329.6 s against 3 877 over all 113): every DBN,
ObjectDetection, Pedigree and Segmentation network, CSP_11 and 13,
Grids_11 and 15 of the 15 Promedus; Merlin is faster on CSP_12 (109 ms
against 327). tddlog's marginals agree with the reference's (Setup) to
9.1e-14 on 111 networks and with Merlin's six printed digits on the
other 5. ProbLog's forward SDD compile (`fsdd`) stops with
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
| Ace | 120 | 66 | 120 | 18 800 | 342.4 | 66 | 2 600 | 62.6 | 0 |
| Merlin | 120 | 113 | 120 | 5 977 | 342.4 | 113 | 3 877 | 329.6 | 1 |
| pyAgrum | 120 | 107 | 120 | 6 218 | 342.4 | 107 | 2 318 | 328.8 | 5 |
| pgmpy | 120 | 53 | 120 | 23 848 | 342.4 | 53 | 3 748 | 134.6 | 0 |
| libDAI | 120 | 117 | 120 | 6 645 | 342.4 | 117 | 5 745 | 329.8 | 0 |
| TensorInference.jl | 120 | 113 | 120 | 3 092 | 342.4 | 113 | 992.5 | 326.2 | 0 |
| TensorInference.jl treesa | 120 | 113 | 120 | 5 408 | 342.4 | 113 | 3 308 | 326.2 | 0 |
| TensorInference.jl greedy | 120 | 59 | 120 | 18 892 | 342.4 | 59 | 591.8 | 74.0 | 0 |
| ProbLog | 120 | 0 | 120 | 36 000 | 342.4 | 0 | 0.0 | 0.0 | 0 |
| PITA | 120 | 1 | 120 | 35 894 | 342.4 | 1 | 193.8 | 0.0 | 0 |
| aspmc | 120 | 0 | 120 | 36 000 | 342.4 | 0 | 0.0 | 0.0 | 0 |
| Dice | 120 | 10 | 120 | 33 351 | 342.4 | 10 | 351.4 | 0.7 | 0 |
| Alea.jl | 120 | 14 | 120 | 32 248 | 342.4 | 14 | 448.0 | 0.9 | 0 |
| Roulette | 120 | 4 | 120 | 35 366 | 335.0 | 4 | 566.3 | 0.2 | 0 |
| noDice | 120 | 0 | 120 | 36 000 | 342.4 | 0 | 0.0 | 0.0 | 0 |
| Psi | 120 | 0 | 120 | 36 000 | 342.4 | 0 | 0.0 | 0.0 | 0 |
| Genfer | 120 | 4 | 120 | 35 306 | 342.4 | 4 | 505.7 | 0.2 | 0 |
| SPPL | 120 | 2 | 120 | 35 565 | 342.4 | 2 | 165.4 | 0.1 | 0 |
| Geni | 120 | 0 | 120 | 36 000 | 342.4 | 0 | 0.0 | 0.0 | 0 |
| Pluck | 120 | 1 | 120 | 35 786 | 371.6 | 1 | 86.2 | 0.0 | 0 |
| Storm | 120 | 1 | 120 | 35 862 | 342.4 | 1 | 161.9 | 0.0 | 0 |
| PRISM | 120 | 0 | 120 | 36 000 | 342.4 | 0 | 0.0 | 0.0 | 0 |

The PR task on the same 120 networks (`--pr`, one job on one lab machine
(16 physical cores and 251 GB), 2026-10-20 to 2026-10-21): tddlog's
log10 Z agrees with the reference's to 5.7e-14 on 107 instances and with
Merlin's to 2.2e-7 on 114. tddlog answers all 120 in 591.8 s, Merlin's
`-t PR` 114 in 1 247. Summed by family where both answer, Merlin is
faster on Segmentation (0.9 s against 2.3) and slower on the other
families (ObjectDetection 984.5 against 526.8, Promedus 130.9 against
1.0).

Against each baseline (columns as in "Against each baseline"):

| baseline | given | baseline answers | tddlog answers | baseline s | tddlog s | both answer | baseline s, both | tddlog s, both | tddlog slower |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Ace | 120 | 70 | 120 | 17 880 | 591.8 | 70 | 2 880 | 109.9 | 0 |
| Merlin | 120 | 114 | 120 | 3 047 | 591.8 | 114 | 1 247 | 571.1 | 38 |
| pyAgrum | 120 | 108 | 120 | 5 812 | 591.8 | 108 | 2 212 | 570.8 | 18 |
| pgmpy | 120 | 104 | 120 | 6 377 | 591.8 | 104 | 1 577 | 501.9 | 39 |
| libDAI | 120 | 120 | 120 | 4 564 | 591.8 | 120 | 4 564 | 591.8 | 5 |
| TensorInference.jl | 120 | 115 | 120 | 2 350 | 591.8 | 115 | 849.7 | 571.2 | 41 |
| TensorInference.jl treesa | 120 | 115 | 120 | 4 721 | 591.8 | 115 | 3 221 | 571.2 | 29 |
| TensorInference.jl greedy | 120 | 60 | 120 | 18 174 | 591.8 | 60 | 173.9 | 128.4 | 16 |
| ProbLog | 120 | 0 | 120 | 36 000 | 591.8 | 0 | 0.0 | 0.0 | 0 |
| PITA | 120 | 25 | 120 | 29 132 | 591.8 | 25 | 631.8 | 0.9 | 0 |
| aspmc | 120 | 18 | 120 | 31 189 | 591.8 | 18 | 588.8 | 1.3 | 0 |
| Dice | 120 | 17 | 120 | 31 325 | 591.8 | 17 | 424.7 | 0.7 | 0 |
| Alea.jl | 120 | 18 | 120 | 30 887 | 591.8 | 18 | 287.5 | 0.7 | 0 |
| Roulette | 120 | 17 | 120 | 31 184 | 570.1 | 17 | 283.9 | 0.5 | 0 |
| noDice | 120 | 0 | 120 | 36 000 | 591.8 | 0 | 0.0 | 0.0 | 0 |
| Psi | 120 | 0 | 120 | 36 000 | 591.8 | 0 | 0.0 | 0.0 | 0 |
| Genfer | 120 | 4 | 120 | 34 843 | 591.8 | 4 | 43.1 | 0.1 | 0 |
| SPPL | 120 | 4 | 120 | 35 200 | 591.8 | 4 | 399.5 | 0.1 | 0 |
| Geni | 120 | 0 | 120 | 36 000 | 591.8 | 0 | 0.0 | 0.0 | 0 |
| Pluck | 120 | 13 | 120 | 32 245 | 591.8 | 13 | 144.6 | 0.4 | 0 |
| Storm | 120 | 5 | 120 | 34 553 | 591.8 | 5 | 52.8 | 0.2 | 0 |
| PRISM | 120 | 9 | 120 | 33 738 | 591.8 | 9 | 438.4 | 0.4 | 0 |

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
| blockmap | 14 | 14 | 4 | 14 | 10 | 0 | 7.1 | 3 096 |
| BN | 5 | 5 | 1 | 5 | 4 | 0 | 0.9 | 1 251 |
| fs | 1 | 1 | 0 | 1 | 1 | 0 | 0.1 | 300.0 |
| mastermind | 67 | 67 | 19 | 67 | 48 | 0 | 34.3 | 14 738 |
| or_chain | 25 | 25 | 2 | 25 | 23 | 0 | 1.3 | 7 085 |
| pedigree | 1 | 1 | 0 | 1 | 1 | 0 | 176.5 | 300.0 |
| protein | 19 | 14 | 0 | 14 | 14 | 5 | 682.1 | 4 200 |
| rus | 60 | 30 | 0 | 30 | 30 | 30 | 3 075 | 9 000 |
| wcsp | 5 | 1 | 2 | 1 | 0 | 3 | 314.1 | 163.6 |
| all | 197 | 158 | 28 | 158 | 131 | 38 | 4 292 | 40 133 |

Against each baseline (the partition function; columns as in "Against
each baseline"):

| baseline | given | baseline answers | tddlog answers | baseline s | tddlog s | both answer | baseline s, both | tddlog s, both | tddlog slower |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Ace | 197 | 95 | 158 | 21 182 | 3 992 | 95 | 2 282 | 40.5 | 1 |
| Merlin | 197 | 28 | 158 | 40 133 | 4 292 | 27 | 765.9 | 18.4 | 0 |
| pyAgrum | 197 | 19 | 158 | 43 154 | 3 992 | 19 | 1 454 | 16.9 | 0 |
| pgmpy | 197 | 6 | 158 | 46 377 | 3 992 | 6 | 777.4 | 2.1 | 0 |
| libDAI | 197 | 21 | 158 | 42 225 | 3 992 | 21 | 1 125 | 3.4 | 0 |
| TensorInference.jl | 197 | 52 | 158 | 36 572 | 5 792 | 46 | 2 829 | 699.7 | 7 |
| TensorInference.jl treesa | 197 | 52 | 158 | 37 085 | 5 792 | 46 | 3 342 | 699.7 | 7 |
| TensorInference.jl greedy | 197 | 15 | 158 | 43 885 | 3 992 | 15 | 985.0 | 43.3 | 0 |
| ProbLog | 197 | 53 | 158 | 32 997 | 3 992 | 53 | 1 497 | 12.9 | 0 |
| PITA | 197 | 62 | 158 | 31 183 | 3 992 | 62 | 2 383 | 15.7 | 0 |
| aspmc | 197 | 77 | 158 | 28 537 | 3 992 | 77 | 4 237 | 35.2 | 0 |
| Dice | 197 | 33 | 158 | 37 781 | 3 992 | 33 | 280.8 | 1.8 | 0 |
| Alea.jl | 197 | 83 | 158 | 22 535 | 3 992 | 83 | 34.9 | 41.5 | 30 |
| Roulette | 197 | 79 | 162 | 25 193 | 1 653 | 79 | 292.6 | 41.1 | 11 |
| noDice | 197 | 7 | 158 | 45 300 | 3 992 | 7 | 0.1 | 0.3 | 0 |
| Psi | 197 | 29 | 158 | 38 845 | 3 992 | 29 | 144.9 | 1.3 | 0 |
| Genfer | 197 | 16 | 158 | 43 306 | 3 992 | 16 | 705.7 | 0.8 | 0 |
| SPPL | 197 | 7 | 158 | 45 305 | 3 992 | 7 | 5.2 | 0.3 | 0 |
| Geni | 197 | 7 | 158 | 45 300 | 3 992 | 7 | 0.0 | 0.3 | 0 |
| Pluck | 197 | 76 | 158 | 25 834 | 3 992 | 76 | 1 234 | 36.3 | 0 |
| Storm | 197 | 29 | 158 | 38 706 | 3 992 | 29 | 6.0 | 1.3 | 0 |
| PRISM | 197 | 29 | 158 | 38 739 | 3 992 | 29 | 39.0 | 1.3 | 0 |

Of tddlog's 39 misses, 34 are timeouts (29 of rus, 5 of protein) and 5
memory refusals (4 of wcsp, 1 of rus); Merlin answers bwt3ac_wcsp of
them. Merlin answers 28, and tddlog is faster on each it also answers.
tddlog's log10 Z agrees within 2.2e-7 with another exact answer wherever
one exists: Merlin's, the reference's (Setup) or a weighted model count
of the network's CNF by Ganak or sharpSAT-TD (111 of its 158 answers);
the other 47 (30 of rus, 13 of protein, 3 of BN, 1 of pedigree) tddlog
alone answers.

The tuning set:

| family | instances | tddlog | Merlin | tddlog fastest | tddlog alone | none | tddlog s | Merlin s |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Alchemy | 1 | 1 | 0 | 1 | 1 | 0 | 0.2 | 300.0 |
| CSP | 3 | 3 | 3 | 2 | 0 | 0 | 6.5 | 17.7 |
| DBN | 6 | 6 | 6 | 4 | 0 | 0 | 24.0 | 43.9 |
| grid10x10.f | 1 | 1 | 1 | 0 | 0 | 0 | 0.1 | 0.0 |
| Grids | 8 | 8 | 3 | 7 | 5 | 0 | 24.5 | 1 574 |
| ObjectDetection | 65 | 65 | 65 | 56 | 0 | 0 | 485.1 | 1 204 |
| Pedigree | 3 | 3 | 3 | 2 | 0 | 0 | 0.1 | 1.5 |
| Promedus | 28 | 28 | 28 | 14 | 0 | 0 | 0.7 | 147.9 |
| Segmentation | 6 | 6 | 6 | 1 | 0 | 0 | 2.1 | 1.3 |
| all | 121 | 121 | 115 | 87 | 6 | 0 | 543.4 | 3 291 |

| baseline | given | baseline answers | tddlog answers | baseline s | tddlog s | both answer | baseline s, both | tddlog s, both | tddlog slower |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Ace | 121 | 67 | 121 | 19 535 | 543.4 | 67 | 3 335 | 85.2 | 0 |
| Merlin | 121 | 115 | 121 | 3 291 | 543.4 | 115 | 1 491 | 524.8 | 16 |
| pyAgrum | 121 | 108 | 121 | 6 543 | 543.4 | 108 | 2 643 | 518.6 | 6 |
| pgmpy | 121 | 104 | 121 | 6 371 | 543.4 | 104 | 1 271 | 462.7 | 23 |
| libDAI | 121 | 121 | 121 | 6 279 | 543.4 | 121 | 6 279 | 543.4 | 0 |
| TensorInference.jl | 121 | 115 | 121 | 2 587 | 543.4 | 115 | 787.0 | 519.4 | 31 |
| TensorInference.jl treesa | 121 | 115 | 121 | 4 984 | 543.4 | 115 | 3 184 | 519.4 | 19 |
| TensorInference.jl greedy | 121 | 61 | 121 | 18 175 | 543.4 | 61 | 174.6 | 118.4 | 15 |
| ProbLog | 121 | 0 | 121 | 36 300 | 543.4 | 0 | 0.0 | 0.0 | 0 |
| PITA | 121 | 25 | 121 | 29 475 | 543.4 | 25 | 675.3 | 0.6 | 0 |
| aspmc | 121 | 24 | 121 | 30 781 | 543.4 | 24 | 1 681 | 1.9 | 0 |
| Dice | 121 | 18 | 121 | 31 331 | 543.4 | 18 | 431.4 | 0.6 | 0 |
| Alea.jl | 121 | 19 | 121 | 30 897 | 543.4 | 19 | 296.7 | 0.6 | 0 |
| Roulette | 121 | 18 | 121 | 31 206 | 561.0 | 18 | 305.6 | 0.6 | 0 |
| noDice | 121 | 0 | 121 | 36 300 | 543.4 | 0 | 0.0 | 0.0 | 0 |
| Psi | 121 | 0 | 121 | 36 300 | 543.4 | 0 | 0.0 | 0.0 | 0 |
| Genfer | 121 | 4 | 121 | 35 142 | 543.4 | 4 | 42.4 | 0.1 | 0 |
| SPPL | 121 | 4 | 121 | 35 485 | 543.4 | 4 | 385.1 | 0.1 | 0 |
| Geni | 121 | 0 | 121 | 36 300 | 543.4 | 0 | 0.0 | 0.0 | 0 |
| Pluck | 121 | 13 | 121 | 32 545 | 543.4 | 13 | 145.1 | 0.2 | 0 |
| Storm | 121 | 6 | 121 | 34 587 | 543.4 | 6 | 87.0 | 0.3 | 0 |
| PRISM | 121 | 10 | 121 | 33 751 | 543.4 | 10 | 451.4 | 0.4 | 0 |

tddlog answers all 121 in 543.4 s, Merlin 115 in 1 491 s. Merlin is
faster on 16 of 97 instances that both answer in 0.2 s or more (9
ObjectDetection, 5 Segmentation, 2 DBN); the two both take under 0.2 s
on 18. log10 Z agrees with the reference's to 5.7e-14 on 113 instances
and with Merlin's to 2.2e-7 on 115, and with a weighted model count by
Ganak or sharpSAT-TD to 7.1e-15 on 40. uai_grids_18, uai_grids_17 and
uai_grids_16 only tddlog answers.

### Results: Ihler's networks

The 370 networks of Ihler's UAI model collection whose files are not the
UAI 2022 sets' (`data/uai-ihler/README.md`), asked the partition
function given the evidence as the PR sets are, 300 s a run (two jobs on
two lab machines (48 physical cores and 743 GB; 36 physical cores and
503 GB), an unknown date); columns as in the PR sets' table.

| family | instances | tddlog | Merlin | tddlog fastest | tddlog alone | none | tddlog s | Merlin s |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| BN | 115 | 89 | 56 | 51 | 40 | 19 | 2 975 | 12 088 |
| classics | 18 | 18 | 16 | 7 | 2 | 0 | 0.8 | 621.9 |
| Grids | 3 | 0 | 0 | 0 | 0 | 3 | 0.0 | 0.0 |
| pedigree | 21 | 19 | 4 | 19 | 15 | 2 | 120.2 | 4 651 |
| promedas | 213 | 210 | 35 | 191 | 175 | 3 | 68.7 | 52 803 |
| all | 370 | 336 | 111 | 268 | 232 | 27 | 3 165 | 70 163 |

tddlog answers 336 of the 370. Its log10 Z agrees with the reference's
to 2.8e-11 on 99 instances and with Merlin's to 6.0e-5 on 104, and with
a weighted model count by Ganak or sharpSAT-TD to 2.9e-6 on 175. No
reference answers Pedigree25, Pedigree38, Pedigree31, Pedigree19,
Pedigree7, Pedigree41, Pedigree44, Pedigree34, Pedigree13, Pedigree40,
Pedigree9, BN_19, BN_17, or_chain_207_fg, or_chain_192_fg,
or_chain_124_fg, or_chain_160_fg, or_chain_118_fg, or_chain_125_fg,
or_chain_23_fg, or_chain_108_fg, or_chain_170_fg, or_chain_202_fg,
or_chain_135_fg, or_chain_66_fg, or_chain_179_fg, or_chain_101_fg,
or_chain_52_fg, or_chain_144_fg, or_chain_196_fg, or_chain_115_fg,
or_chain_80_fg, or_chain_222_fg, or_chain_216_fg, or_chain_110_fg,
or_chain_164_fg, or_chain_89_fg, or_chain_43_fg, or_chain_121_fg,
or_chain_27_fg, or_chain_9_fg, or_chain_131_fg, or_chain_99_fg,
or_chain_243_fg, or_chain_13_fg, or_chain_181_fg, or_chain_51_fg,
or_chain_97_fg, or_chain_7_fg, or_chain_14_fg, or_chain_20_fg,
or_chain_58_fg, or_chain_173_fg, or_chain_244_fg, or_chain_30_fg,
or_chain_126_fg, or_chain_75_fg, or_chain_112_fg, or_chain_39_fg,
or_chain_41_fg, or_chain_157_fg, or_chain_87_fg, or_chain_185_fg,
or_chain_249_fg, or_chain_231_fg, or_chain_143_fg, or_chain_3_fg,
or_chain_177_fg, or_chain_24_fg, or_chain_240_fg, or_chain_19_fg,
or_chain_167_fg, or_chain_100_fg, or_chain_215_fg, or_chain_228_fg,
or_chain_195_fg, or_chain_83_fg, or_chain_200_fg, or_chain_21_fg,
or_chain_59_fg, or_chain_245_fg, or_chain_183_fg, or_chain_16_fg,
or_chain_234_fg, or_chain_6_fg, or_chain_146_fg, or_chain_145_fg,
or_chain_219_fg, or_chain_32_fg, or_chain_224_fg, or_chain_31_fg,
or_chain_162_fg, or_chain_210_fg, or_chain_127_fg, or_chain_25_fg,
or_chain_95_fg, or_chain_239_fg, or_chain_60_fg, or_chain_54_fg,
or_chain_230_fg, or_chain_11_fg, or_chain_2_fg, or_chain_44_fg,
or_chain_117_fg, or_chain_79_fg, or_chain_119_fg, or_chain_166_fg,
or_chain_22_fg, or_chain_70_fg, or_chain_123_fg, or_chain_134_fg,
or_chain_217_fg, or_chain_73_fg, or_chain_120_fg, or_chain_47_fg,
or_chain_81_fg, or_chain_151_fg, or_chain_57_fg, or_chain_104_fg,
or_chain_213_fg, or_chain_187_fg, or_chain_233_fg and or_chain_26_fg to
check it.

Against each baseline (columns as in "Against each baseline"):

| baseline | given | baseline answers | tddlog answers | baseline s | tddlog s | both answer | baseline s, both | tddlog s, both | tddlog slower |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Ace | 370 | 151 | 336 | 59 476 | 2 780 | 145 | 2 074 | 685.4 | 10 |
| Merlin | 370 | 111 | 336 | 70 163 | 3 165 | 104 | 548.0 | 731.8 | 23 |
| pyAgrum | 370 | 102 | 336 | 73 122 | 3 165 | 95 | 809.8 | 731.5 | 18 |
| pgmpy | 370 | 111 | 336 | 71 746 | 3 165 | 104 | 2 128 | 744.3 | 17 |
| libDAI | 370 | 117 | 336 | 68 655 | 3 165 | 110 | 837.5 | 879.8 | 21 |
| TensorInference.jl | 370 | 150 | 336 | 64 706 | 3 465 | 142 | 6 346 | 895.2 | 20 |
| TensorInference.jl treesa | 370 | 133 | 336 | 74 257 | 3 465 | 125 | 10 677 | 357.0 | 1 |
| TensorInference.jl greedy | 370 | 105 | 336 | 71 884 | 3 165 | 98 | 456.3 | 881.3 | 20 |
| ProbLog | 370 | 50 | 336 | 86 138 | 979.9 | 50 | 338.2 | 1.7 | 0 |
| PITA | 370 | 101 | 336 | 74 515 | 979.9 | 101 | 4 015 | 3.6 | 0 |
| aspmc | 370 | 76 | 336 | 80 744 | 1 280 | 75 | 2 159 | 9.1 | 0 |

## The TPC-H suite

The probabilistic-database workload of SPROUT (Olteanu, Huang, Koch, ICDE
2009) and dissociation (Gatterbauer and Suciu, VLDB J. 2017): TPC-H with
every tuple an independent probabilistic fact and the TPC-H queries as
Boolean queries that keep their joins and selections (`--suite tpch`,
295 instances). `scripts/gen_tpch.py` writes the data after the schema
and value distributions of the TPC-H specification (clause 4.2.3), since
TPC's dbgen may not be redistributed. Each query is a family,
`tpch_<query>_<1000 SF>`, at scale factors 0.001 to 1 from the
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
probability by a method of its own choice) runs the same instances as
SQL (`scripts/tpch2provsql.py`): a table per fact predicate, a view per
rule predicate, the seconds those of loading the data and of the queries
through `psql` (the server's start is not timed).

### Results

tddlog 0.1.0 `auto`, ProbLog's three compilations, PITA, aspmc and
ProvSQL 1.13.0 on PostgreSQL 16.4 (its own choice of method; the seconds
of the database's load and its queries), in a second job, on a lab
machine with 20 physical cores and 503 GB of memory, on 2026-10-19, in
18 lanes of 55 GB of address space, 600 s a run.

tddlog answers 289 of the 295 (2 235 s in all), each within 4.0e-14 of
`exact:tpch`; it misses q21v and q21 at SF 0.1; q18v at SF 1; q7 and q7v
at SF 0.2; q7 at SF 0.5. Against the fastest of ProbLog, PITA and aspmc
on each instance, in total time: Of the 295 instances both are given,
tddlog answers 289 and ProbLog, PITA or aspmc 191. Over the 289 that
either answers, a run that does not answer counted at the 600 s limit,
tddlog takes 2 235 s in all and the fastest of them 76 490 s; over the
191 both answer, 230.3 s against 17 690. tddlog alone answers
tpch_q10v_5, tpch_q7v_50, tpch_q7_50, tpch_q10_5, tpch_q11_50 and 93
more. Neither answers tpch_q21v_100, tpch_q21_100, tpch_q18v_1000,
tpch_q7_200, tpch_q7_500 and 1 more. By query, flat and views together
(seconds in total time, as in "Against each baseline": over the
instances either answers, a run that does not answer counted at 600 s;
largest scale factor answered, times 1 000):

| query | instances | tddlog | ProbLog, PITA or aspmc | tddlog s | ProbLog, PITA or aspmc s | largest SF, tddlog / others | ProvSQL | tddlog s | ProvSQL s |
|---|---:|---:|---:|---:|---:|---|---:|---:|---:|
| q1 | 20 | 20 | 12 | 253.3 | 5 905 | 1000 / 50 | 0 | 0.0 | 0.0 |
| q2 | 18 | 18 | 16 | 14.3 | 1 480 | 1000 / 500 | 0 | 0.0 | 0.0 |
| q3 | 20 | 20 | 16 | 136.5 | 4 408 | 1000 / 200 | 0 | 0.0 | 0.0 |
| q4 | 20 | 20 | 16 | 139.3 | 4 638 | 1000 / 200 | 0 | 0.0 | 0.0 |
| q6 | 20 | 20 | 13 | 107.0 | 5 739 | 1000 / 100 | 0 | 0.0 | 0.0 |
| q7 | 15 | 12 | 8 | 318.6 | 2 609 | 100 / 20 | 0 | 0.0 | 0.0 |
| q10 | 20 | 20 | 4 | 147.0 | 9 643 | 1000 / 2 | 0 | 0.0 | 0.0 |
| q11 | 16 | 16 | 6 | 18.1 | 6 051 | 1000 / 20 | 0 | 0.0 | 0.0 |
| q12 | 20 | 20 | 10 | 187.9 | 6 723 | 1000 / 50 | 0 | 0.0 | 0.0 |
| q14 | 20 | 20 | 11 | 143.9 | 7 042 | 1000 / 200 | 0 | 0.0 | 0.0 |
| q15 | 20 | 20 | 16 | 166.9 | 5 399 | 1000 / 200 | 0 | 0.0 | 0.0 |
| q16 | 20 | 20 | 16 | 16.1 | 3 111 | 1000 / 200 | 0 | 0.0 | 0.0 |
| q17 | 16 | 16 | 9 | 140.9 | 5 447 | 1000 / 100 | 0 | 0.0 | 0.0 |
| q18 | 20 | 19 | 13 | 416.2 | 5 278 | 1000 / 100 | 0 | 0.0 | 0.0 |
| q20 | 18 | 18 | 18 | 10.2 | 560.8 | 1000 / 1000 | 0 | 0.0 | 0.0 |
| q21 | 12 | 10 | 7 | 18.9 | 2 457 | 50 / 20 | 0 | 0.0 | 0.0 |

ProvSQL runs 279 of the 295 instances and answers 259 (20 timeout
otherwise), each within 2.1e-12 of `exact:tpch` (259 compared). Against
ProvSQL in total time: ProvSQL is given none of these instances.

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

The same job as the TPC-H results above, 600 s a run (Scallop in a later
job). Seconds, the fastest of ProbLog's three compilations:

| K | 1 | 2 | 4 | 8 | 12 | 16 | 24 | 32 | 48 | 64 |
|---|---|---|---|---|---|---|---|---|---|---|
| tddlog | 0.01 | 0.01 | 0.01 | 0.02 | 0.07 | 0.12 | 0.16 | 3.57 | 2.34 | 172.7 |
| ProbLog | 0.13 | 0.20 | 0.42 | 73.0 | >600 |  |  |  |  |  |
| PITA | 0.52 | 0.50 | 0.50 | 4.73 | >600 |  |  |  |  |  |
| aspmc | 2.78 | 7.88 | >600 |  |  |  |  |  |  |  |
| Scallop | 0.01 | err |  |  |  |  |  |  |  |  |

tddlog answers every K, within 3.4e-12 of the reference; ProbLog answers
to K = 8 (within 4.6e-9, its eight printed digits), PITA to K = 8
(within 3.3e-16) and aspmc to K = 2; Scallop to K = 1 (within 5.6e-17).

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

tddlog 0.1.0 `auto` as of 2026-10-22 (tididi c7c0c5d6, goatd e592d1e),
linked statically, Dice 1.0 with an unlimited stack and Alea.jl at
revision cf5e5b8 (Julia 1.11.9, CUDD 3.0.0) in one job on a lab machine
(16 physical cores, 125 GB) on 2026-10-22, in 8 lanes, each process
limited to 15 GB of address space. The runs on Dice programs and LPADs
ran a build of 2026-10-27 that reads Dice programs and LPADs itself
(tididi 7eb08230, goatd e592d1e). The runs beside Roulette on its
upstream main and Roulette+tddlog's ran a build of 2026-11-01 (tididi
c5642b97, goatd e592d1e). The release's main contains every solver
change those builds have. The three systems run on a program back to
back in one lane, in an order rotated by program and repetition
(`bench.py run --rotate`); three repetitions, 300 s each on the networks
and the paper's programs and 600 s on the loops. Seconds are the median
of the repetitions, a failed repetition counted as longer than any time;
`(k/3)` marks a program answered in k repetitions, `>T` a timeout and
`oom` an allocation that failed inside the process. Alea has three
columns:

- **Alea**: the wall time of the Julia process, as every other column is
  the wall time of its system's process. It includes Julia's start,
  `using Alea`, a warm-up program and parsing the script: 8.39 s in the
  median over the runs, from 8.11 to 20.8 s.
- **in-process**: what the script reports for building the program and
  running `pr`.
- **JIT**: Julia's compilation within the in-process time, mostly of
  Alea's methods for the program's types; the translated program's own
  functions are interpreted (baselines.md, section 5).

Answers. Answers were checked in a pass of their own, with ProbLog's
d-DNNF and, on the networks, the clique tree as references, then Alea's
where neither answers. tddlog agrees with the reference on every program
it answers; its largest relative differences are 4.0e-10 from the clique
tree (bn_sachs) and 1.9e-10 from Alea (dice_simpleCaesar). Alea agrees
with it on every program Alea finishes; its largest are 4.0e-10 from the
clique tree (bn_sachs). Alea computes probabilities in log space and
gives tiny posteriors: down to 2.7e-152 on digitRecognition and 5.5e-293
on simpleCaesar; tddlog counts an answer that small on the complement's
formula (design.md, Queries) and prints it in scientific notation, and
agrees with Alea's to a relative 1.9e-10. Dice prints 0 for every answer
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

| instance | tddlog | Dice | Alea | in-process | JIT |
|---|---:|---:|---:|---:|---:|
| bn_andes | 0.04 | 235.37 | oom | oom | oom |
| bn_child | 0.01 | 0.15 | 9.37 | 1.01 | 0.99 |
| bn_hailfinder | 0.04 | 1.30 | 11.32 | 2.75 | 1.99 |
| bn_hepar2 | 0.03 | 0.91 | 9.99 | 1.28 | 1.02 |
| bn_insurance | 0.02 | 1.04 (wrong) | 11.98 | 3.08 | 1.92 |
| bn_link | 0.59 | oom | 44.25 | 33.18 | 2.79 |
| bn_munin | 0.49 | >300 | oom | oom | oom |
| bn_pathfinder | 0.27 | 5.38 (wrong) | 30.77 | 10.03 | 3.55 |
| bn_pigs | 0.06 | 3.19 | 11.68 | 2.18 | 1.88 |
| bn_sachs | 0.01 | 0.08 | 9.81 | 1.36 | 1.35 |
| bn_water | 0.03 | 126.41 | 11.75 | 1.70 | 1.51 |
| bn_win95pts | 0.01 | 0.42 | 9.73 | 1.14 | 1.12 |

On the five networks that every system answers in under a second (alarm,
asia, cancer, earthquake and survey) and the paper's seven other small
programs, tddlog takes 0.004 to 0.01 s, Dice 0.02 to 0.26 s and Alea
8.26 to 9.22 s, of which 0.04 to 0.97 s in the process, nearly all of it
JIT. tddlog is the fastest on 12 of the 17 networks (against Alea
without its compilation). Alea's time in the process is below Dice's on
link, pigs and water, and its wall time on link and water.

On digitRecognition tddlog takes 0.35 s, Dice 0.20 s (all zero) and Alea
11.05 s, 1.94 s in the process (1.52 s JIT).

The loop programs at their own sizes (bench.py's `indicators` suite) and
scaled by `dice_scale.py` (`zero`: Dice printed 0 for every answer, the
evidence below its cutoff of 1e-6):

| program | steps | tddlog | Dice | Alea | in-process | JIT |
|---|---:|---:|---:|---:|---:|---:|
| diamond | 100 | 0.01 | 0.02 | 8.23 | 0.07 | 0.07 |
| diamond | 400 | 0.03 | 0.03 | 8.49 | 0.16 | 0.07 |
| diamond | 1 600 | 0.14 | 0.07 | 10.26 | 2.10 | 0.07 |
| diamond | 6 400 | 0.35 | 0.19 | 49.04 | 40.54 | 0.07 |
| ladder | 100 | 0.05 | 0.02 | 9.41 | 1.16 | 1.13 |
| ladder | 400 | 0.20 | 0.03 | 9.63 | 1.45 | 1.12 |
| ladder | 1 600 | 0.46 | 0.10 | 14.96 | 6.81 | 1.12 |
| ladder | 6 400 | 1.45 | 0.35 | oom | oom | oom |
| caesar-iter | 100 | 0.03 | 0.03 (zero) | 9.65 | 1.04 | 1.02 |
| caesar-iter | 1 000 | 0.41 | 0.09 (zero) | 10.12 | 1.96 | 0.95 |
| caesar-iter | 3 000 | 1.60 | 0.25 (zero) | 21.98 | 13.63 | 0.98 |
| simpleCaesar | 100 | 0.35 | 0.11 (zero) | 9.57 | 1.43 | 1.19 |
| simpleCaesar | 300 | 0.87 | 0.30 (zero) | 12.25 | 3.82 | 1.24 |
| simpleCaesar | 1 000 | 3.84 | 0.86 (zero) | 40.32 | 31.80 | 1.23 |
| pmc1 | 10 | 0.14 | >600 | 14.90 | 6.48 | 6.46 |
| pmc1 | 20 | 0.26 | >600 | 15.27 | 6.57 | 6.54 |
| pmc1 | 40 | 0.17 | >600 | 14.97 | 6.61 | 6.56 |
| pmc1 | 75 | 0.69 | >600 | 31.24 | 22.78 | 17.12 |
| pmc2 | 10 | 0.06 | 87.85 | 11.95 | 3.38 | 3.37 |
| pmc2 | 20 | 0.10 | >600 | 12.15 | 3.50 | 3.49 |
| pmc2 | 51 | 0.23 | >600 | 18.26 | 10.00 | 8.65 |

On diamond Dice is faster than tddlog from 1 600 steps on (0.19 s
against 0.35 at 6 400 steps). Alea's time in the process on diamond
grows from 0.072 s at 100 steps to 40.5 s at 6 400. On ladder Dice is
faster than tddlog from 100 steps on (0.35 s against 1.45 at 6 400
steps). Alea answers ladder up to 1 600 steps (oom at 6 400). On pmc1
and pmc2 Dice fails on 6 of the 7 sizes; Alea answers 7, in 3.38 to
22.78 s in the process, and tddlog 7, in 0.06 to 0.69 s.

tddlog holds a Dice program's integers in bits, as Dice and Alea do (its
Dice reader); an integer of n values takes about log2 n atoms. The same
binary ran on the translation that held them one-hot, an atom per value,
in the same lanes: on the 48 programs of noDice and data/dice that both
answer, one-hot takes 139 s in all against 91.8 s in bits, and their
answers agree to a relative 1.7e-13. Where either takes 0.1 s or more
and the two differ by a quarter or more:

| program | bits | one-hot | bits pairs | one-hot pairs |
|---|---:|---:|---:|---:|
| dice_caesar-iter | 1.60 | 0.49 | 77 124 | 84 024 |
| dice_caesar-iter_k1000 | 0.41 | 0.19 | 14 006 | 28 024 |
| dice_caesar-iter_k3000 | 0.93 | 0.49 | 77 124 | 84 024 |
| dice_diamond_k1600 | 0.14 | 0.25 | 17 595 | 19 182 |
| dice_diamond_k400 | 0.03 | 0.11 | 4 395 | 4 800 |
| dice_diamond_k6400 | 0.35 | 0.90 | 70 395 | 76 800 |
| dice_ladder | 0.05 | 0.11 | 2 861 | 2 590 |
| dice_ladder_k100 | 0.06 | 0.11 | 2 861 | 2 590 |
| dice_ladder_k1600 | 0.46 | 0.77 | 50 405 | 49 562 |
| dice_ladder_k400 | 0.20 | 0.31 | 12 077 | 12 356 |
| dice_ladder_k6400 | 1.45 | 1.97 | 204 005 | 198 356 |
| dice_pmc1 | 0.69 | 0.98 | 184 | 147 |
| dice_pmc1_k20 | 0.26 | 0.14 | 0 | 0 |
| dice_pmc1_k75 | 0.69 | 0.96 | 184 | 147 |
| dice_pmc2 | 0.23 | 0.30 | 41 | 30 |
| dice_simpleCaesar | 3.84 | 0.74 | 76 062 | 161 074 |
| dice_simpleCaesar_k100 | 0.35 | 0.12 | 6 476 | 17 074 |
| dice_simpleCaesar_k1000 | 3.90 | 0.74 | 76 062 | 161 074 |
| dice_simpleCaesar_k300 | 0.87 | 0.25 | 17 733 | 49 074 |
| nodice_alarm | 0.05 | 0.14 | 13 698 | 15 969 |
| nodice_coupon2-10-obs | 0.10 | 0.18 | 502 608 | 1 004 262 |
| nodice_coupon2-11-obs | 0.20 | 0.27 | 1 740 428 | 2 236 628 |
| nodice_coupon2-12-obs | 0.17 | 0.71 | 2 857 960 | 10 237 648 |
| nodice_coupon2-13-obs | 0.58 | 2.26 | 8 252 950 | 25 871 969 |
| nodice_coupon2-14-obs | 0.74 | 3.04 | 19 626 442 | 64 259 106 |
| nodice_coupon2-16-obs | 5.84 | 22.16 | 30 473 630 | 217 660 970 |
| nodice_coupon2-18-obs | 65.45 | 96.46 | 517 450 552 | 402 338 412 |
| nodice_hepar2 | 0.08 | 0.14 | 8 622 | 119 550 |
| nodice_insurance | 0.10 | 0.19 | 1 158 747 | 141 555 |
| nodice_water | 0.36 | 0.73 | 471 618 | 1 105 004 |

The repetitions differ by more than half in two places where one takes a
second or more, most on dice_pmc1_k75 (tddlog 0.69 to 1.12 s) and
dice_ladder_k6400 (tddlog 1.45 to 2.3 s).

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
programs (`nodice`, 69). Alea runs an Alea.jl program itself; tddlog,
ProbLog and Dice run the Boolean circuit Alea builds for it, which
`alea_bench.jl` writes (a ProbLog program, and a Dice program per
Boolean answer or per bit of an expectation, the seconds summed). The
`nodice` programs are Dice programs, translated by `dice2problog.py` for
tddlog and ProbLog and by `dice2alea.py` for Alea.

### Results: tddlog 0.1.0, Dice and Alea.jl

tddlog 0.1.0 `auto` as of 2026-10-28 (tididi ca759585, goatd e592d1e),
linked statically, Dice 1.0 with an unlimited stack and Alea.jl at
revision cf5e5b8 (Julia 1.11.9, CUDD 3.0.0) in one job on a lab machine
(16 physical cores, 251 GB) on 2026-10-29, in 10 lanes, each process
limited to 25 GB of address space. The runs beside Roulette on its
upstream main and Roulette+tddlog's ran a build of 2026-11-01 (tididi
c5642b97, goatd e592d1e). The release's main contains every solver
change those builds have. The three systems run on an instance back to
back in one lane, in an order rotated by instance and repetition; three
repetitions, 600 s each. ProbLog's d-DNNF ran in a pass of its own (300
s) as a second reference. Seconds are the median of the repetitions, a
failed repetition counted as longer than any time; `(k/3)` marks an
instance answered in k repetitions, `>T` a timeout, `oom` an allocation
that failed inside the process, `killed` a SIGTERM from outside the
driver and `stack` a stack overflow. The columns:

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
| book | 5.3 | 9.40 | >600 | 66.11 | 52.95 | 51.72 | 1.18 | >300 | 7e-17 / 6e-16 |  |
| tugofwar | 0.106 | 0.11 | 296.12 | 79.75 | 69.21 | 69.14 | 0.04 | >300 | 8e-16 / 8e-16 | 4e-13 / 5e-13 |
| caesar_small | 0.041 | 0.11 | 0.55 | 81.89 | 71.37 | 71.31 | 0.03 | 11.74 | 1e-15 / 3e-14 | zero |
| caesar_medium | 0.239 | 1.60 | 15.43 | 82.32 | 71.38 | 71.16 | 0.16 | 60.67 | 9e-15 / 3e-13 | zero |
| caesar_large | 0.556 | 1.09 | 62.05 | 82.63 | 71.50 | 71.06 | 0.40 | 128.09 | 1e-29 / 6e-13 | zero |
| caesar_huge |  | 2.29 | 411.33 | 89.98 | 75.56 | 73.49 | 1.95 | >300 | 3e-69 / 3e-12 | underflow |
| ranking_small | 0.007 | 0.01 | 0.71 | 41.80 | 31.36 | 31.35 | 0.008 | 48.50 | 3e-16 / 1e-15 | 5e-13 / 2e-12 |
| ranking_medium | 0.022 | 0.03 | 209.36 | 42.14 | 31.62 | 31.60 | 0.01 | 202.13 | 6e-16 / 1e-14 | 4e-13 / 2e-12 |
| ranking_large | 0.048 | 0.04 | >600 | 41.87 | 31.29 | 31.25 | 0.03 | >300 | 2e-15 / 3e-14 |  |
| radar | 0.034 | 0.11 | 0.21 | 56.43 | 45.51 | 45.46 | 0.03 | 41.37 | 3e-16 / 2e-15 | 5e-13 / 3e-12 |
| fw_small | 0.009 | 0.01 | 0.05 | 87.69 | 77.52 | 77.49 | 0.01 | 0.13 | 0 | 0 |
| fw_medium | 0.515 | 0.10 | >600 | 96.13 | 84.86 | 84.33 | 0.49 | 17.83 | 8e-17 / 3e-14 |  |
| fw_large | 3.41 | 0.51 | >600 | 102.19 | 85.87 | 83.06 | 2.71 | >300 | 4e-17 / 9e-15 |  |
| linext_small | 0.003 | 0.006 | 0.10 | 39.43 | 29.02 | 29.01 | 0.005 | 0.19 | 2e-16 / 4e-16 | 2e-16 / 4e-16 |
| linext_medium | 0.007 | 0.01 | 0.24 | 40.80 | 30.55 | 30.54 | 0.007 | 41.82 | 6e-16 / 2e-15 | 3e-13 / 3e-12 |
| linext_large | 0.072 | 0.02 | 0.85 | 41.54 | 30.98 | 30.96 | 0.02 | >300 | 4e-16 / 2e-15 | 4e-13 / 2e-12 |
| triangle_small | 0.086 | 0.24 | 91.32 | 52.50 | 41.99 | 41.93 | 0.05 | >300 | 3e-16 / 5e-15 | 2e-13 / 7e-13 |
| triangle_medium | 0.455 | 0.47 | 465.86 | 53.14 | 42.37 | 42.18 | 0.15 | >300 | 2e-15 / 3e-14 | 4e-13 / 2e-12 |
| triangle_large | 17.4 | 18.93 | >600 | 88.31 | 54.27 | 41.93 | 12.60 | >300 | 5e-15 / 8e-14 |  |
| gcd_small | 2.88 | 8.60 | >600 | 21.38 | 6.03 | 3.61 | 2.35 | >300 | 2e-16 / 4e-16 |  |
| gcd_medium | 104 | 107.82 | >600 | 277.76 | 91.48 | 3.69 | 84.50 | >300 | 2e-16 / 4e-16 |  |
| gcd_large | >3600 | >600 | >600 | >600 | >600 | >600 | >600 | oom |  |  |
| disease_small | 7.91 | 20.38 | oom | 196.46 | 91.03 | 36.49 | 53.95 | >300 | 8e-16 / 6e-13 |  |
| disease_medium | 764 | 306.43 | oom | oom | oom | oom | oom | oom | unchecked |  |
| disease_large |  | >600 | oom | >600 | >600 | >600 | >600 | stack |  |  |
| luhn_small | 0.039 | 0.02 | 0.27 | 66.55 | 55.94 | 55.89 | 0.01 | 4.73 | 2e-16 / 1e-15 | 2e-13 / 2e-12 |
| luhn_medium | 4.58 | 0.15 | 4.20 | 66.59 | 56.08 | 55.98 | 0.04 | >300 | 8e-16 / 5e-15 | 4e-13 / 2e-12 |
| plankton |  | 0.33 | oom | 47.92 | 36.79 | 36.66 | 0.14 | >300 | 1e-16 / 4e-14 |  |

The programs of the bit-blasting paper's Table 2, at its precision
(conjugate_gaussians on the artifact file's observations 8 and 9;
`data/alea/hybit/conjugate_gaussians.jl` now observes Table 2's 1 and
2):

| instance | tddlog | Dice | Alea | in-proc | JIT | warm | ProbLog | tddlog Δ | Dice Δ |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| pi | 0.94 | >600 | 167.89 | 112.17 | 90.36 | 22.17 | >300 | 6e-17 / 1e-16 |  |
| weekend | 20.96 | 480.42 | 93.14 | 56.16 | 43.96 | 12.42 | stack | 8e-16 / 2e-15 | zero |
| spacex | oom | >600 | 299.81 | 152.22 | 31.37 | 136.72 | >300 |  |  |
| GPA | 29.07 | 43.56 | 514.77 | 503.97 | 491.13 |  | stack | 1e-16 / 2e-16 | 9e-14 / 2e-13 |
| tug_of_war | 0.42 | >600 | >600 | >600 | >600 | >600 | >300 | unchecked |  |
| altermu2 | oom | >600 | oom | oom | oom | oom | >300 |  |  |
| conjugate_gaussians | 0.92 | >600 | 219.37 | 208.27 | 34.85 |  | >300 | 3e-15 / 6e-15 |  |
| normal_mixture_theta | 19.80 | >600 | oom | oom | oom | oom | >300 | unchecked |  |
| normal_mixture_mu1 | 179.41 | >600 | 410.77 | 398.69 | 55.55 |  | >300 | 1e-14 / 1e-13 |  |
| normal_mixture_mu2 | 114.41 | >600 | oom | oom | oom | oom | >300 | 2e-14 / 2e-14 |  |
| zeroone_w1 | 0.96 | >600 | 91.91 | 79.17 | 78.09 | 0.98 | >300 | 4e-15 / 8e-15 |  |
| zeroone_w2 | 2.24 | >600 | 117.52 | 87.77 | 75.61 | 12.62 | >300 | 1e-13 / 2e-13 |  |
| coinBias | >600 | >600 | 311.51 | 299.54 | 47.47 |  | stack |  |  |
| addFun_sum | 0.64 | >600 | 520.92 | 508.59 | 53.60 |  | >300 | 7e-16 / 1e-15 |  |
| clickGraph | 0.29 | >600 | oom | oom | oom | oom | 8.30 | 5e-9 / 9e-9 (P) |  |
| trueskill | 34.33 | >600 | 275.79 | 108.74 | 32.57 | 78.11 | >300 | 2e-16 / 3e-16 |  |
| clinicalTrial1 | 1.80 | >600 | 254.34 | 243.24 | 39.69 |  | 23.71 | 4e-16 / 3e-15 |  |
| clinicalTrial2 | 0.03 | >600 | 302.99 | 291.39 | 39.95 |  | 30.94 | 4e-15 / 7e-15 |  |
| addFun_max | 16.30 | >600 | oom | oom | oom | oom | >300 | unchecked |  |

Pluck's sequence models:

| instance | paper | tddlog | Dice | Alea | in-proc | JIT | warm | ProbLog | tddlog Δ | Dice Δ |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| hmm | 0.00276 | 0.03 | >600 | 11.36 | 0.62 | 0.61 | 0.008 | >300 | 6e-27 / 3e-14 |  |
| pcfg | 8.83 | 105.35 | >600 | 41.59 | 11.69 | 2.66 | 9.31 | >300 | 9e-23 / 2e-15 |  |
| string_editing | 61.3 | 0.06 | 1.52 | 167.25 | 53.52 | 2.03 | 51.31 | >300 | 2e-26 / 7e-15 | 2e-24 / 1e-12 |
| sorted_list | 21.9 | 5.83 | 43.19 | 85.18 | 26.05 | 1.33 | 23.87 | >300 | 2e-25 / 2e-14 | 4e-24 / 5e-13 |

noDice's programs:

| instance | paper | tddlog | Dice | Alea | in-proc | JIT | ProbLog | tddlog Δ | Dice Δ |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| coupon2-10-obs | 0.721 | 0.10 | 0.35 | 9.90 | 1.34 | 1.14 | >300 | 2e-16 / 8e-16 | 3e-13 / 1e-12 |
| coupon2-11-obs | 1.96 | 0.20 | 0.96 | 10.55 | 1.84 | 1.17 | >300 | 3e-16 / 1e-15 | 2e-13 / 7e-13 |
| coupon2-12-obs | 5.8 | 0.17 | 2.76 | 11.16 | 2.72 | 1.10 | >300 | 7e-16 / 4e-15 | 2e-13 / 1e-12 |
| coupon2-13-obs | 20.7 | 0.58 | 9.55 | 15.32 | 6.24 | 1.18 | >300 | 5e-16 / 3e-15 | 1e-13 / 7e-13 |
| coupon2-14-obs | 63.4 | 0.74 | 35.07 | 33.08 | 24.40 | 1.21 | >300 | 2e-16 / 2e-15 | 2e-14 / 2e-13 |
| coupon2-16-obs |  | 5.84 | >600 | >600 | >600 | >600 | >300 | unchecked |  |
| coupon2-18-obs |  | 65.45 | >600 | >600 | >600 | >600 | >300 | unchecked |  |
| coupon2-20-obs |  | oom | >600 | >600 | >600 | >600 | >300 |  |  |
| survey | 0.001 | 0.005 | 0.07 | 8.92 | 0.48 | 0.47 | 0.11 | 1e-16 / 4e-16 | 1e-16 / 2e-16 |
| hepar2 | 0.099 | 0.08 | 0.08 | 9.59 | 0.93 | 0.78 | 0.92 | 1e-16 / 1e-16 | 3e-14 / 2e-13 |
| insurance | 0.58 | 0.10 | 0.24 | 11.13 | 2.24 | 1.47 | 50.27 | 2e-16 / 2e-15 | 7e-6 / 4e-4 |
| water | 9.24 | 0.36 | 214.82 | 11.62 | 1.57 | 1.42 | 8.79 | 1e-16 / 1e-15 | 5e-13 / 1e-12 |
| alarm | 160 | 0.05 | 0.18 | 10.04 | 1.12 | 0.76 | 2.18 | 8e-17 / 4e-16 | 3e-13 / 8e-13 |
| 3sat-0 | 19.5 | 0.40 | 9.49 | 9.63 | 1.39 | 0.34 | >300 | 1e-22 / 1e-15 | 1e-19 / 1e-12 |

Answers. Of the 65 instances tddlog answers 59, Dice 31, Alea 53 and
ProbLog's d-DNNF 19. tddlog answers eight where Alea fails (arith
disease_medium, hybit tug_of_war, hybit normal_mixture_theta, hybit
normal_mixture_mu2, hybit clickGraph, hybit addFun_max, nodice
coupon2-16-obs and nodice coupon2-18-obs). Alea answers two that tddlog
does not (hybit spacex and hybit coinBias). tddlog answers every
instance Dice answers.

tddlog agrees with the reference on every instance both answer, to a
relative 3.1e-12 at most (on arith caesar_huge, absolute 3.5e-69). Where
Alea fails, ProbLog agrees with tddlog to its eight printed digits on
hybit clickGraph (5e-9). tddlog alone answers arith disease_medium,
hybit tug_of_war, hybit normal_mixture_theta, hybit addFun_max, nodice
coupon2-16-obs and nodice coupon2-18-obs (unchecked). Dice agrees to a
relative 2.7e-12 or less wherever it prints nonzero answers, but for
nodice insurance. Dice prints 0 for every answer on arith caesar_small,
arith caesar_medium, arith caesar_large, arith caesar_huge and hybit
weekend. Their evidence has probability 4.6e-15 (small), 2.3e-69
(medium), 4.5e-132 (large), 1.1e-323 (huge) and 1.1e-7 (weekend), from
tddlog's `--evidence-probability`; Dice prints zeros wherever the
evidence has probability below 1e-6 (baselines.md, section 4), so they
are marked `zero` where a double holds the evidence and `underflow`
where it is below the smallest normal double. On insurance Dice is off
by 7.4e-6 (relative 4.4e-4): its `discrete` drops the network's table
entries below 1e-4.

Failures. Of the 585 timed runs 36 ended on a failed allocation: 9 of
tddlog's, 10 of Dice's and 17 of Alea's.

Times. Against Dice (its zero answers counted as answers), over the 59
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, tddlog takes 1087 s in all and Dice 19201 s;
over the 31 both answer, 65 s against 2401 s. tddlog is faster on 26 of
the 31 both answer and Dice on 0, both under 0.2 s or level on the other
5. tddlog alone answers 28, Dice alone 0. Against Alea's wall time, over
the 61 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, tddlog takes 2287 s in all and Alea
10770 s; over the 51 both answer, 558 s against 5359 s; against its
first time in the process, over the 61 instances given to both that
either answers, a run that does not answer counted at the 600 s limit,
tddlog takes 2287 s in all and Alea 9353 s; over the 51 both answer, 558
s against 4101 s. Without Julia's compilation (the warm time, or the
first time in the process less its JIT where there was no warm run),
over the 61 instances given to both that either answers, a run that does
not answer counted at the 600 s limit, tddlog takes 2287 s in all and
Alea 7034 s; over the 51 both answer, 558 s against 1845 s. tddlog is
faster on 21 of the 51 both answer and Alea on 14, both under 0.2 s or
level on the other 16. tddlog alone answers 8, Alea alone 2. Alea's
largest leads are on pluck pcfg (9.31 s against 105), arith gcd_medium
(84.5 s against 108), hybit GPA (12.5 s against 29.1), hybit weekend
(12.4 s against 21) and arith book (1.18 s against 9.4).

Of the 22 instances where tddlog takes 1 s or more, `auto`'s attempts
before the one that answers (vtrees it tries and drops, and attempts
stopped at a smaller work limit) take half the time or more on 5: on
Alea's largest leads they take 16.6 of 105 s on pluck pcfg, 16.5 of 108
s on arith gcd_medium, 0 of 29.1 s on hybit GPA, 0 of 21 s on hybit
weekend and 1.04 of 9.4 s on arith book, and without them tddlog would
take 88.7 s on pluck pcfg, 91.3 s on arith gcd_medium, 29.1 s on hybit
GPA, 21 s on hybit weekend and 8.37 s on arith book against Alea's 9.31,
84.5, 12.5, 12.4 and 1.18.

Alea's first run is mostly Julia compiling Alea's methods for the
program's types: 491 of 504 s on hybit GPA, 29 to 84.3 s on the
arithmetic programs but the gcd ones. Its times without compilation on
the arithmetic programs run from 0.0079 of Cao et al.'s Dice.jl times
(arith luhn_medium 0.036 s against 4.58) to 6.8 times them (arith
disease_small 54 s against 7.91). Dice's times here and in noDice's
Table 1 differ by large factors both ways on the same programs (nodice
survey 0.067 s against 0.001, nodice alarm 0.18 s against 160).

## The bit-blasting paper's programs

`bench.py --suite bitblast` holds the programs of the bit-blasting paper
(Garg, Holtzen, Van den Broeck and Millstein, PLDI 2024) on ladders of
size, as circuits that `alea_bench.jl` writes from Alea.jl programs:
each program of its Table 2 (`data/alea/hybit`) at the paper's precision
and at 2, 4 and 6 bits after the point below and above it, and the three
that a system still answered 6 bits above further, until none answers:
`pi` at 20 bits after the point, `zeroone_w1` at 18, 22, 26 and 30,
`zeroone_w2` at 19, 21, 23 and 25; the gene expression program of its
Section 2.1 (`data/alea/bitblast/ genes.jl`) at Figure 2's precision
from 5 to 100 genes (the paper's 5 to 50), and with 5 genes by precision
(`bb_genes5`); and the multimodal model of Figure 3 and the conjugate
Gaussians of Figure 5 by precision. An instance is `bb_NAME_wW`, W the
width of the program's numbers in bits (Table 2's column `Bit`), or
`bb_genes_tT` with T genes: 163 instances in 23 families. Alea's own
bit-count assertions reject two of them (`bb_normal_mixture_theta_w7`,
`bb_multimodal_w9`), which leaves 161. The paper's other systems run on
the artifact's own programs of the same models (baselines.md, section
10).

### Results: tddlog 0.1.0 and Alea.jl, with Dice and ProbLog from an earlier run

tddlog 0.1.0 as of 2026-10-28 (tididi ca759585, goatd e592d1e), linked
statically, in three arms: `auto`; `--vtree program` (`program`), a
right-linear vtree over the facts in the reverse of the order the
program states them, so that the diagram reads the circuit's flips in
the order Alea made them; and `--vtree decision` (`decision`), which
grounds the program again with each derived atom conditioned on the
selector flips that guard it and places each selector above the flips it
decides between (`n/a` where the grounder reads no flip as a guard:
clickGraph, clinicalTrial1, clinicalTrial2 and pi). The runs beside
Roulette on its upstream main and Roulette+tddlog's ran a build of
2026-10-31 (tididi c5642b97, goatd e592d1e). The reruns of tddlog where
an older build's record stood, and every system's runs on eleven of the
growth sizes ran a build of 2026-11-01 (tididi c5642b97, goatd e592d1e).
The growth sizes ran a build of 2026-10-25 with twin contraction (tididi
b9e5064e, goatd e592d1e) and a recursion over a successor relation
grounded level by level, which lacks only the release's sparse route
(tididi 9fc0a599). The runs beside Scallop ran a build of 2026-10-29
(tididi df7b84a3, goatd e592d1e). Beside a cross job's systems, one run
of tddlog on an instance where the rerun's was skipped ran that job's
build, as of 2026-10-21 (tididi c7c0c5d6, goatd e592d1e). The release's
main contains every solver change those builds have. Alea.jl at revision
cf5e5b8 on Julia 1.11.9 with CUDD 3.0.0, Distributions.jl 0.25.131 and
StatsFuns 2.2.1. One job on a lab machine (20 physical cores, 503 GB) on
2026-10-29 and 30, in 15 lanes of 33 GB a process, with the earlier
run's rotation, 600 s and the skip rule; the four run on an instance
back to back in one lane, three repetitions. Dice and ProbLog were not
rerun: their columns are the earlier run's (below; three repetitions for
Dice, ProbLog's d-DNNF in a pass of its own), a run of their own, and
their times are not compared with the others' here. The page's plots
leave them out.

Seconds are the median of the repetitions, a failed repetition counted
as longer than any time; `(k/n)` marks an instance answered in k of n
repetitions. Alea's time is its warm time (the program built and
answered a second time in the process, without Julia's compilation, as
the papers time it; the first run's time less its compilation where the
first took more than 200 s), and `Alea process` the wall time of the
Julia process. `oom` is an allocation that failed inside the process,
`killed` a SIGTERM from earlyoom, the machine's out-of-memory guard,
when its free memory ran low, `zero` every answer printed 0 though a
double holds the answers, `-` not run after it ran out of time or memory
on a smaller size. `ref` names the reference: Alea's answer, else
Ganak's exact count of the circuit (below), else ProbLog's, else
tddlog's own; the difference columns give the largest absolute and
relative difference of an answer from it.

Of the 161 instances, 22 lie past the sizes at which every system had
failed (the table leaves them out), and none every system answers in
under a second. tddlog answers 125 with `auto`, 59 with `program` and 53
with `decision` (125 by some arm); Alea 74; Dice (earlier run) 17, and
prints 0 for every answer of 8 more.

- Against Alea's warm time, over the 87 instances given to both that
  either answers, a run that does not answer counted at the 600 s limit,
  tddlog (`auto`) takes 3004 s in all and Alea 13214 s; over the 72 both
  answer, 2012 s against 4721 s. tddlog (`auto`) is faster on 44 of the
  72 both answer and Alea on 24, both under 0.2 s or level on the other
  4. tddlog (`auto`) alone answers 14, Alea alone 1. Against the Julia
  process, over the 87 instances given to both that either answers, a
  run that does not answer counted at the 600 s limit, tddlog takes 3004
  s in all and the process 22935 s; over the 72 both answer, 2012 s
  against 14188 s. For `program` against Alea, over the 63 instances
  given to both that either answers, a run that does not answer counted
  at the 600 s limit, `program` takes 7456 s in all and Alea 6987 s;
  over the 49 both answer, 1848 s against 2740 s; for `decision`, over
  the 64 instances given to both that either answers, a run that does
  not answer counted at the 600 s limit, `decision` takes 18137 s in all
  and Alea 4758 s; over the 35 both answer, 1888 s against 1625 s.
- tddlog reaches further than Alea on 14 programs: GPA, to 31 bits in
  58.3 s, where Alea times out at 19; addFun_max, to 29 bits in 37.7 s,
  where Alea answers to 21 bits (215 s) and times out at 23; addFun_sum,
  to 29 bits in 1.79 s, where Alea answers to 21 bits (122 s) and times
  out at 23; clickGraph, to 16 bits in 0.97 s, where Alea answers to 8
  bits (9.92 s) and times out at 10; clinicalTrial1, to 14 bits in 25.6
  s, where Alea answers to 8 bits (347 s) and times out at 10;
  clinicalTrial2, to 18 bits in 0.06 s, where Alea answers to 12 bits
  (374 s) and times out at 14; conjugate_gaussians, to 29 bits in 3.23
  s, where Alea answers to 23 bits (230 s) and times out at 25; genes,
  to 100 genes in 26.2 s, where Alea times out at 5; normal_mixture_mu2,
  to 9 bits in 199 s, where Alea answers to 7 bits (2.51 s) and times
  out at 9; normal_mixture_theta, to 9 bits in 40.8 s, where Alea times
  out at 9; pi, to 20 bits in 184 s, where Alea answers to 16 bits (305
  s) and times out at 18; tug_of_war, to 28 bits in 1.02 s, where Alea
  answers to 20 bits (197 s) and times out at 22; zeroone_w1, to 30 bits
  in 12.7 s, where Alea answers to 22 bits (56.1 s) and times out at 26;
  zeroone_w2, to 29 bits in 18.6 s, where Alea answers to 21 bits (98.4
  s) and times out at 23.
- Alea reaches further on conjugate_ex3. Of these, tddlog answers no
  size of none.
- `program` against `auto`: over the 77 instances given to both that
  either answers, a run that does not answer counted at the 600 s limit,
  `auto` takes 1742 s in all and `program` 13797 s; over the 59 both
  answer, 850 s against 2997 s; `auto` is faster on 34 of the 59 both
  answer and `program` on 18. `program` answers 0 that `auto` does not,
  `auto` 18 that `program` does not (GPA, addFun_max, addFun_sum,
  altermu2, clickGraph, clinicalTrial1, coinBias, conjugate_ex3,
  conjugate_gaussians, genes, genes5, multimodal, normal_mixture_theta,
  pi, tug_of_war, weekend, zeroone_w1 and zeroone_w2).
- `decision` against `auto`: over the 92 instances given to both that
  either answers, a run that does not answer counted at the 600 s limit,
  `auto` takes 2609 s in all and `decision` 25817 s; over the 53 both
  answer, 1542 s against 2417 s; `auto` is faster on 11 of the 53 both
  answer and `decision` on 42. `decision` answers 0 that `auto` does
  not, `auto` 39 that `decision` does not (addFun_max, addFun_sum,
  clickGraph, clinicalTrial1, clinicalTrial2, conjugate_gaussians,
  multimodal, normal_mixture_mu1, normal_mixture_mu2,
  normal_mixture_theta, pi, spacex, trueskill, tug_of_war, zeroone_w1
  and zeroone_w2).
- At the paper's precision (19 programs of Table 2), tddlog answers 18
  by some arm (GPA, addFun_max, addFun_sum, clickGraph, clinicalTrial1,
  clinicalTrial2, coinBias, conjugate_gaussians, normal_mixture_mu1,
  normal_mixture_mu2, normal_mixture_theta, pi, spacex, trueskill,
  tug_of_war, weekend, zeroone_w1 and zeroone_w2), Alea 11
  (clinicalTrial1, clinicalTrial2, coinBias, conjugate_gaussians,
  normal_mixture_mu1, pi, spacex, trueskill, weekend, zeroone_w1 and
  zeroone_w2), Dice (earlier run) 1 (GPA); no system answers altermu2.
  On the gene expression program at Figure 2's precision (5 to 50
  genes), `auto` and `decision` answer every number of genes, and
  `program`, Alea and Dice (earlier run) none.

How far each system reaches: per program, the largest size (in bits;
genes for `genes`) a system answers and its time there, else its failure
at the smallest size it ran; `paper` is the paper's precision.

| program | paper | auto | program | decision | Alea | Dice (earlier) |
|---|---:|---:|---:|---:|---:|---:|
| GPA | 25 | 31 (58.3 s) | >600 at 19 | 31 (51.8 s) | >600 at 19 | 31 (69.8 s) |
| addFun_max | 23 | 29 (37.7 s) | 19 (176 s) | oom at 17 | 21 (215 s) | >600 at 17 |
| addFun_sum | 23 | 29 (1.79 s) | 21 (61.3 s) | 17 (441 s) | 21 (122 s) | >600 at 17 |
| altermu2 | 17 | 15 (73.1 s) | 13 (392 s) | 15 (69.9 s) | 15 (143 s) | >600 at 11 |
| clickGraph | 10 | 16 (0.97 s) | 10 (20.7 s) | n/a at 4 | 8 (9.92 s) | 6 (47.9 s) |
| clinicalTrial1 | 8 | 14 (25.6 s) | 10 (40.7 s) | n/a at 4 | 8 (347 s) | zero at 4 |
| clinicalTrial2 | 12 | 18 (0.06 s) | 18 (1.3 s) | n/a at 6 | 12 (374 s) | 8 (137 s) |
| coinBias | 22 | 22 (201 s) | >600 at 16 | 22 (162 s) | 22 (416 s) | >600 at 16 |
| conjugate_ex3 |  | 20 (84.3 s) | oom at 18 | 20 (72.5 s) | 24 (383 s) | >600 at 18 |
| conjugate_gaussians | 23 | 29 (3.23 s) | 23 (151 s) | 17 (417 s) | 23 (230 s) | >600 at 17 |
| genes | 5-50 | 100 (26.2 s) | oom at 5 | 100 (23.8 s) | >600 at 5 | >600 at 5 |
| genes5 |  | 27 (24.2 s) | >600 at 17 | 27 (20.7 s) | 27 (503 s) | zero at 17 |
| multimodal |  | 11 (501 s) | oom at 11 | oom at 11 | 11 (50.3 s) | >600 at 11 |
| normal_mixture_mu1 | 9 | 9 (202 s) | 9 (370 s) | 7 (1.67 s) | 9 (480 s) | >600 at 7 |
| normal_mixture_mu2 | 9 | 9 (199 s) | 9 (591 s) | oom at 7 | 7 (2.51 s) | >600 at 7 |
| normal_mixture_theta | 9 | 9 (40.8 s) | oom at 9 | oom at 9 | >600 at 9 | >600 at 9 |
| pi | 14 | 20 (184 s) | 16 (92.3 s) | n/a at 8 | 16 (305 s) | >600 at 8 |
| spacex | 19 | 19 (130 s) | 19 (74 s) | oom at 13 | 19 (204 s) | >600 at 13 |
| trueskill | 10 | 10 (61.2 s) | 10 (35.1 s) | oom at 8 | 10 (123 s) | >600 at 8 |
| tug_of_war | 22 | 28 (1.02 s) | 18 (151 s) | oom at 16 | 20 (197 s) | >600 at 16 |
| weekend | 24 | 30 (49.2 s) | >600 at 18 | 30 (44.9 s) | 30 (22.1 s) | 20 (359 s) |
| zeroone_w1 | 16 | 30 (12.7 s) | 26 (137 s) | 20 (95.3 s) | 22 (56.1 s) | 14 (354 s) |
| zeroone_w2 | 19 | 29 (18.6 s) | 27 (274 s) | 19 (21.3 s) | 21 (98.4 s) | 13 (131 s) |

The full table:

| instance | flips | gates | tddlog | program | decision | Dice (earlier) | Alea | Alea process | ProbLog | ref | tddlog diff | program diff | decision diff | Dice (earlier) diff | Alea diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GPA_w19 | 45061 | 368680 | 31.2 | >600 | 26.9 | 43.2 | >600 | >600 | stack | tddlog | ref |  |  | 9.4e-14 / 1.5e-13 |  |
| GPA_w21 | 61441 | 417832 | 34.8 | - | 31.2 | 47.7 | - | - | stack | tddlog | ref |  |  | 9.4e-14 / 1.5e-13 |  |
| GPA_w23 | 77812 | 466966 | 38.9 | - | 34.9 | 52.4 | - | - | stack | tddlog | ref |  |  | 9.4e-14 / 1.5e-13 |  |
| GPA_w25 | 94168 | 516070 | 44.5 | - | 40.2 | 56.1 | - | - | stack | tddlog | ref |  |  | 9.4e-14 / 1.5e-13 |  |
| GPA_w27 | 110529 | 565184 | 49.6 | - | 44.6 | 61 | - | - | stack | tddlog | ref |  |  | 9.4e-14 / 1.5e-13 |  |
| GPA_w29 | 126871 | 614260 | 53.7 | - | 48.5 | 66.8 | - | - | stack | tddlog | ref |  |  | 9.4e-14 / 1.5e-13 |  |
| GPA_w31 | 143187 | 663285 | 58.3 | - | 51.8 | 69.8 | - | - | stack | tddlog | ref |  |  | 9.4e-14 / 1.5e-13 |  |
| addFun_max_w17 | 566 | 18099 | 18.4 | 41.8 | oom | >600 | 10.3 | 70.7 | >600 | alea | 2.2e-16 / 4.4e-16 | 7.9e-15 / 1.6e-14 |  |  | ref |
| addFun_max_w19 | 580 | 21035 | 40.9 | 176 | - | - | 47.7 | 176 | - | alea | 3.3e-16 / 6.7e-16 | 4.5e-14 / 9.1e-14 |  |  | ref |
| addFun_max_w21 | 594 | 23603 | 30.9 | >600 | - | - | 215 | 259 | - | alea | 5.6e-16 / 1.1e-15 |  |  |  | ref |
| addFun_max_w23 | 604 | 26275 | 33.8 | - | - | - | >600 | >600 | - | tddlog | ref |  |  |  |  |
| addFun_max_w25 | 618 | 28939 | 35 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| addFun_max_w27 | 630 | 31297 | 36.1 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| addFun_max_w29 | 642 | 33393 | 37.7 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| addFun_max_p1024 | 4014 | 187241 | >600 | - | - | - | - | - | - |  |  |  |  |  |  |
| addFun_max_p16 | 196 | 3801 | 0.918 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| addFun_max_p256 | 1090 | 50669 | 175 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| addFun_max_p4 | 168 | 1249 | 0.505 | oom | oom | >600 | >600 | >600 | oom | tddlog | ref |  |  |  |  |
| addFun_max_p64 | 366 | 13777 | 5.29 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| addFun_sum_w17 | 152 | 2800 | 0.813 | 3.66 | 441 | >600 | 5.38 | 123 | >600 | alea | 7.8e-16 / 1.6e-15 | 1.1e-13 / 2.2e-13 | 1.8e-11 / 3.6e-11 |  | ref |
| addFun_sum_w19 | 168 | 3212 | 0.926 | 14.6 | oom | - | 28.5 | 181 | - | alea | 5.6e-16 / 1.1e-15 | 5.5e-13 / 1.1e-12 |  |  | ref |
| addFun_sum_w21 | 184 | 3624 | 1.08 | 61.3 | - | - | 122 | 238 | - | alea | 6.7e-16 / 1.3e-15 | 1.3e-12 / 2.6e-12 |  |  | ref |
| addFun_sum_w23 | 196 | 3820 | 1.21 | oom | - | - | >600 | >600 | - | tddlog | ref |  |  |  |  |
| addFun_sum_w25 | 212 | 4208 | 1.36 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| addFun_sum_w27 | 228 | 4596 | 1.54 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| addFun_sum_w29 | 244 | 4984 | 1.79 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| altermu2_w11 | 20458 | 274380 | 23.1 | 125 | 21.5 | >600 | 8.16 | 112 | >600 | alea | 4.7e-14 / 9.4e-14 | 4.8e-14 / 9.6e-14 | 4.7e-14 / 9.4e-14 |  | ref |
| altermu2_w13 | 40822 | 336280 | 36.1 | 392 | 34.1 | - | 35.4 | 193 | - | alea | 3.4e-14 / 6.8e-14 | 2.9e-14 / 5.7e-14 | 3.4e-14 / 6.8e-14 |  | ref |
| altermu2_w15 | 60746 | 397700 | 73.1 | oom | 69.9 | - | 143 | 231 | - | alea | 1.6e-14 / 3.1e-14 |  | 1.6e-14 / 3.1e-14 |  | ref |
| altermu2_w17 | 80350 | 457880 | oom | - | oom | - | >600 | >600 | - |  |  |  |  |  |  |
| altermu2_p1024 | 240710 | 1777240 | >600 | - | - | - | - | - | - |  |  |  |  |  |  |
| altermu2_p16 | 7670 | 35720 | 3.86 | oom | - | - | - | - | - | tddlog | ref |  |  |  |  |
| altermu2_p4 | 2230 | 13800 | 2.49 | 138 | oom | >600 | >600 | >600 | >600 | merlin | 4.9e-07 / 9.7e-07 | 4.9e-07 / 9.7e-07 |  |  |  |
| altermu2_p64 | 25430 | 121720 | 26.3 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| clickGraph_w4 | 52 | 135 | 0.015 | 0.014 | n/a | 0.176 | 0.016 | 85.8 | 0.155 | alea | 2.2e-16 / 3.7e-16 | 2.2e-16 / 3.7e-16 |  | 3.5e-13 / 5.7e-13 | ref |
| clickGraph_w6 | 104 | 345 | 0.03 | 0.028 | n/a | 47.9 | 0.083 | 82.7 | 0.377 | alea | 2.2e-16 / 4.1e-16 | 2.2e-16 / 4.1e-16 |  | 4.4e-13 / 8.4e-13 | ref |
| clickGraph_w8 | 156 | 555 | 0.389 | 0.265 | n/a | >600 | 9.92 | 113 | 1.53 | alea | 6.7e-16 / 9.9e-16 | 6.7e-16 / 9.9e-16 |  |  | ref |
| clickGraph_w10 | 208 | 765 | 0.58 | 20.7 | n/a | - | >600 | >600 | 5.92 | Ganak | 5.2e-15 / 1.0e-14 | 1.0e-15 / 1.5e-15 |  |  |  |
| clickGraph_w12 | 260 | 975 | 0.762 | oom | n/a | - | - | - | 30.9 | Ganak | 8.3e-15 / 1.4e-14 |  |  |  |  |
| clickGraph_w14 | 312 | 1185 | 1.16 | - | n/a | - | - | - | wrong | Ganak | 1.2e-14 / 2.1e-14 |  |  |  |  |
| clickGraph_w16 | 364 | 1395 | 0.971 | - | n/a | - | - | - | oom | Ganak | 4.2e-15 / 8.4e-15 |  |  |  |  |
| clinicalTrial1_w4 | 807 | 3200 | 0.494 | 0.486 | n/a | zero | 0.439 | 94.2 | 1.51 | alea | 5.8e-15 / 2.9e-14 | 5.8e-15 / 2.9e-14 |  |  | ref |
| clinicalTrial1_w6 | 1613 | 8000 | 1 | 0.715 | n/a | zero | 10.9 | 125 | 5.19 | alea | 1.1e-14 / 6.4e-14 | 1.1e-14 / 6.4e-14 |  |  | ref |
| clinicalTrial1_w8 | 2419 | 12800 | 3.39 | 2.47 | n/a | >600 | 347 | 440 | 24.5 | alea | 4.4e-16 / 2.6e-15 | 4.4e-16 / 2.6e-15 |  |  | ref |
| clinicalTrial1_w10 | 3225 | 17600 | 8.53 | 40.7 | n/a | - | >600 | >600 | 121 | Ganak | 1.1e-16 / 6.5e-16 | 5.6e-17 / 3.3e-16 |  |  |  |
| clinicalTrial1_w12 | 4031 | 22400 | 12.2 | oom | n/a | - | - | - | oom | merlin | 4.5e-07 / 2.6e-06 |  |  |  |  |
| clinicalTrial1_w14 | 4837 | 27200 | 25.6 | - | n/a | - | - | - | - | ace-Xlog | 4.4e-15 / 2.6e-14 |  |  |  |  |
| clinicalTrial2_w6 | 89 | 412 | 0.019 | 0.018 | n/a | 1.37 | 0.033 | 86.8 | 0.398 | alea | 6.7e-16 / 1.3e-15 | 6.7e-16 / 1.3e-15 |  | 3.7e-13 / 7.4e-13 | ref |
| clinicalTrial2_w8 | 133 | 658 | 0.026 | 0.024 | n/a | 137 | 0.442 | 89 | 1.08 | alea | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 |  | 4.7e-13 / 1.2e-12 | ref |
| clinicalTrial2_w10 | 177 | 904 | 0.032 | 0.03 | n/a | >600 | 13.8 | 128 | 4.74 | alea | 1.1e-15 / 2.2e-15 | 1.1e-15 / 2.2e-15 |  |  | ref |
| clinicalTrial2_w12 | 221 | 1150 | 0.049 | 0.046 | n/a | - | 374 | 460 | 24 | alea | 3.6e-15 / 7.1e-15 | 3.6e-15 / 7.1e-15 |  |  | ref |
| clinicalTrial2_w14 | 265 | 1396 | 0.052 | 0.104 | n/a | - | >600 | >600 | 118 | Ganak | 1.1e-16 / 2.2e-16 | 4.2e-15 / 8.4e-15 |  |  |  |
| clinicalTrial2_w16 | 309 | 1642 | 0.056 | 0.319 | n/a | - | - | - | oom | Ganak | 2.2e-16 / 4.4e-16 | 2.4e-15 / 4.9e-15 |  |  |  |
| clinicalTrial2_w18 | 353 | 1888 | 0.06 | 1.3 | n/a | - | - | - | - | Ganak | 2.2e-16 / 4.4e-16 | 2.8e-15 / 5.6e-15 |  |  |  |
| coinBias_w16 | 12422 | 152450 | 14 | >600 | 12.5 | >600 | 13.9 | 145 | oom | alea | 2.4e-15 / 5.3e-15 |  | 2.4e-15 / 5.3e-15 |  | ref |
| coinBias_w18 | 20630 | 177156 | 23 | - | 21.4 | - | 29.6 | 189 | - | alea | 3.8e-15 / 6.1e-15 |  | 3.8e-15 / 6.1e-15 |  | ref |
| coinBias_w20 | 28834 | 201858 | 51 | - | 48.5 | - | 104 | 298 | - | alea | 1.0e-14 / 2.1e-14 |  | 1.0e-14 / 2.1e-14 |  | ref |
| coinBias_w22 | 37026 | 226532 | 201 | - | 162 | - | 416 | 524 | - | alea | 1.4e-14 / 2.9e-14 |  | 1.4e-14 / 2.9e-14 |  | ref |
| coinBias_w24 | 45190 | 251150 | >600 | - | oom | - | >600 | >600 | - |  |  |  |  |  |  |
| conjugate_ex3_w18 | 9147 | 239177 | 51.8 | oom | 45.9 | >600 | 10.4 | 110 | oom | alea | 5.6e-15 / 1.0e-14 |  | 5.6e-15 / 1.0e-14 |  | ref |
| conjugate_ex3_w20 | 9135 | 290908 | 84.3 | - | 72.5 | - | 25.2 | 152 | - | alea | 6.4e-15 / 1.3e-14 |  | 6.4e-15 / 1.3e-14 |  | ref |
| conjugate_ex3_w22 | 9135 | 342186 | >600 | - | oom | - | 93.2 | 347 | - | alea |  |  |  |  | ref |
| conjugate_ex3_w24 | 9117 | 392519 | - | - | - | - | 383 | 461 | - | alea |  |  |  |  | ref |
| conjugate_ex3_w26 | 9105 | 442375 | - | - | - | - | >600 | >600 | - |  |  |  |  |  |  |
| conjugate_gaussians_w17 | 177 | 2788 | 0.624 | 2.21 | 417 | >600 | 1.6 | 83.3 | >600 | alea | 4.0e-15 / 2.1e-14 | 6.2e-15 / 2.1e-14 | 4.1e-15 / 2.2e-14 |  | ref |
| conjugate_gaussians_w19 | 201 | 3258 | 0.849 | 8.38 | oom | - | 9.78 | 109 | - | alea | 2.4e-15 / 5.8e-15 | 8.7e-15 / 1.7e-14 |  |  | ref |
| conjugate_gaussians_w21 | 225 | 3728 | 1.1 | 34.1 | - | - | 47.3 | 214 | - | alea | 4.3e-15 / 8.7e-15 | 2.5e-14 / 4.9e-14 |  |  | ref |
| conjugate_gaussians_w23 | 249 | 4198 | 1.86 | 151 | - | - | 230 | 309 | - | alea | 2.7e-15 / 6.5e-15 | 3.3e-13 / 6.6e-13 |  |  | ref |
| conjugate_gaussians_w25 | 273 | 4668 | 1.66 | oom | - | - | >600 | >600 | - | tddlog | ref |  |  |  |  |
| conjugate_gaussians_w27 | 297 | 5138 | 2.42 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| conjugate_gaussians_w29 | 321 | 5608 | 3.23 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| fair_dt14_w12 | 1795 | 9432 | 7.97 | 4.04 | 274 | >600 | 0.191 | 128 | oom | alea | 2.8e-17 / 5.6e-16 | 1.7e-17 / 5.6e-16 | 1.4e-17 / 2.2e-16 |  | ref |
| fair_dt14_w16 | 3034 | 13225 | 14.6 | 62.9 | oom | - | 1.08 | 135 | - | alea | 5.2e-17 / 1.7e-15 | 5.6e-17 / 8.6e-16 |  |  | ref |
| fair_dt14_w20 | 4259 | 17021 | 30.4 | oom | - | - | 20.6 | 192 | - | alea | 5.6e-17 / 1.0e-15 |  |  |  | ref |
| fair_dt14_w24 | 5272 | 20135 | 35.8 | - | - | - | >600 | >600 | - | tddlog | ref |  |  |  |  |
| fair_dt14_w28 | 6274 | 23227 | 51.2 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| fair_dt14_w32 | 7263 | 26293 | 84.4 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| fair_dt14_w36 | 8145 | 29141 | 86.4 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| fair_dt14_w40 | 8974 | 31891 | 97.2 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| fair_dt16_w12 | 420 | 2143 | 0.417 | 0.211 | 0.121 | >600 | 0.048 | 129 | 3.9 | alea | 2.8e-17 / 2.8e-16 | 2.8e-17 / 2.8e-16 | 2.8e-17 / 2.8e-16 |  | ref |
| fair_dt16_w16 | 638 | 2884 | 0.634 | 0.367 | 0.194 | - | 0.067 | 129 | 3.92 | alea | 5.6e-17 / 2.9e-16 | 5.6e-17 / 2.9e-16 | 5.6e-17 / 4.2e-16 |  | ref |
| fair_dt16_w20 | 793 | 3420 | 0.804 | 0.476 | 0.236 | - | 0.069 | 131 | 9.19 | alea | 8.3e-17 / 5.6e-16 | 8.3e-17 / 5.6e-16 | 8.3e-17 / 4.4e-16 |  | ref |
| fair_dt16_w24 | 780 | 3394 | 0.832 | 0.476 | 0.24 | - | 0.067 | 128 | 8.96 | alea | 5.6e-17 / 2.9e-16 | 5.6e-17 / 2.9e-16 | 4.2e-17 / 8.4e-16 |  | ref |
| fair_dt16_w28 | 754 | 3342 | 0.699 | 0.457 | 0.233 | - | 0.068 | 126 | 8.47 | alea | 2.8e-17 / 4.2e-16 | 2.8e-17 / 4.2e-16 | 5.6e-17 / 4.2e-16 |  | ref |
| fair_dt16_w32 | 741 | 3316 | 0.694 | 0.44 | 0.228 | - | 0.069 | 126 | 8.2 | alea | 2.8e-17 / 4.2e-16 | 2.8e-17 / 4.2e-16 | 4.2e-17 / 8.4e-16 |  | ref |
| fair_dt4_w12 | 1871 | 9632 | 4.54 | 3.47 | 210 | >600 | 0.201 | 128 | 263 | alea | 2.8e-17 / 1.8e-16 | 2.8e-17 / 1.8e-16 | 6.9e-18 / 2.4e-16 |  | ref |
| fair_dt4_w16 | 3064 | 13311 | 15.3 | 49.9 | oom | - | 0.933 | 126 | >600 | alea | 2.8e-17 / 1.9e-16 | 2.8e-17 / 1.9e-16 |  |  | ref |
| fair_dt4_w20 | 4349 | 17315 | 31.2 | oom | - | - | 16.9 | 172 | - | alea | 2.8e-17 / 2.5e-16 |  |  |  | ref |
| fair_dt4_w24 | 5360 | 20425 | 38.3 | - | - | - | >600 | >600 | - | tddlog | ref |  |  |  |  |
| fair_dt4_w28 | 6358 | 23509 | 54.9 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| fair_dt4_w32 | 7345 | 26571 | 103 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| fair_dt4_w36 | 8223 | 29411 | 131 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| fair_dt4_w40 | 9050 | 32157 | 90.9 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| fair_dt44_w12 | 478 | 2361 | 0.371 | 0.243 | 0.137 | >600 | 0.051 | 137 | 2.06 | alea | 8.3e-17 / 5.8e-16 | 8.3e-17 / 5.8e-16 | 8.3e-17 / 4.8e-16 |  | ref |
| fair_dt44_w16 | 696 | 3090 | 0.541 | 0.38 | 0.214 | - | 0.062 | 132 | 4.69 | alea | 2.8e-17 / 1.6e-16 | 3.5e-17 / 5.8e-16 | 1.4e-17 / 2.3e-16 |  | ref |
| fair_dt44_w20 | 851 | 3614 | 0.723 | 0.473 | 0.261 | - | 0.068 | 131 | 8.11 | alea | 1.1e-16 / 6.4e-16 | 1.1e-16 / 6.4e-16 | 5.6e-17 / 8.1e-16 |  | ref |
| fair_dt44_w24 | 838 | 3588 | 0.679 | 0.46 | 0.26 | - | 0.071 | 130 | 7.82 | alea | 8.3e-17 / 5.8e-16 | 8.3e-17 / 5.8e-16 | 8.3e-17 / 4.8e-16 |  | ref |
| fair_dt44_w28 | 812 | 3536 | 0.662 | 0.451 | 0.256 | - | 0.065 | 132 | 7.58 | alea | 8.3e-17 / 4.8e-16 | 8.3e-17 / 4.8e-16 | 1.1e-16 / 6.4e-16 |  | ref |
| fair_dt44_w32 | 799 | 3510 | 0.602 | 0.434 | 0.249 | - | 0.072 | 132 | 6.97 | alea | 2.8e-17 / 2.3e-16 | 5.6e-17 / 3.2e-16 | 8.3e-17 / 4.8e-16 |  | ref |
| genes_t5 | 67284 | 308661 | 25.7 | oom | 22.4 | >600 | >600 | >600 | oom | tddlog | ref |  |  |  |  |
| genes_t10 | 67484 | 309241 | 25.2 | - | 22.3 | - | - | - | - | tddlog | ref |  |  |  |  |
| genes_t15 | 67684 | 309821 | 25.2 | - | 23.1 | - | - | - | - | tddlog | ref |  |  |  |  |
| genes_t20 | 67884 | 310401 | 25.4 | - | 22.7 | - | - | - | - | tddlog | ref |  |  |  |  |
| genes_t25 | 68084 | 310981 | 25.2 | - | 22.3 | - | - | - | - | tddlog | ref |  |  |  |  |
| genes_t30 | 68284 | 311561 | 25 | - | 22.4 | - | - | - | - | tddlog | ref |  |  |  |  |
| genes_t40 | 68684 | 312721 | 25.3 | - | 22.3 | - | - | - | - | tddlog | ref |  |  |  |  |
| genes_t50 | 69084 | 313881 | 26 | - | 22.6 | - | - | - | - | tddlog | ref |  |  |  |  |
| genes_t60 | 69484 | 315041 | 25.8 | - | 23.1 | - | - | - | - | tddlog | ref |  |  |  |  |
| genes_t80 | 70284 | 317361 | 27 | - | 23.8 | - | - | - | - | tddlog | ref |  |  |  |  |
| genes_t100 | 71084 | 319681 | 26.2 | - | 23.8 | - | - | - | - | tddlog | ref |  |  |  |  |
| genes5_w17 | 24148 | 173021 | 12.8 | >600 | 11.3 | zero | 3.23 | 103 | oom | alea | 5.4e-15 / 1.1e-14 |  | 5.4e-15 / 1.1e-14 |  | ref |
| genes5_w19 | 31844 | 196665 | 15.3 | - | 13.8 | >600 | 4.67 | 116 | - | alea | 7.5e-15 / 1.5e-14 |  | 7.5e-15 / 1.5e-14 |  | ref |
| genes5_w21 | 39336 | 219821 | 17.1 | - | 15.6 | - | 8.64 | 123 | - | alea | 6.9e-15 / 1.4e-14 |  | 6.9e-15 / 1.4e-14 |  | ref |
| genes5_w23 | 46572 | 242497 | 19.4 | - | 17.4 | - | 24.1 | 173 | - | alea | 3.7e-15 / 7.3e-15 |  | 3.7e-15 / 7.3e-15 |  | ref |
| genes5_w25 | 53588 | 264781 | 21.1 | - | 19 | - | 107 | 305 | - | alea | 3.9e-15 / 8.4e-15 |  | 3.9e-15 / 8.4e-15 |  | ref |
| genes5_w27 | 60520 | 286833 | 24.2 | - | 20.7 | - | 503 (2/3) | 600 (2/3) | - | alea | 2.9e-15 / 7.3e-15 |  | 2.9e-15 / 7.3e-15 |  | ref |
| laplace_w13 | 4125 | 110955 | 10.2 | 141 | 9.52 | >600 | 8.57 | 170 | >600 | alea | 4.6e-15 / 9.1e-15 | 4.6e-15 / 9.1e-15 | 4.6e-15 / 9.1e-15 |  | ref |
| laplace_w15 | 8145 | 158877 | >600 | 416 | oom | - | 146 | 289 | - | alea |  | 5.1e-15 / 1.0e-14 |  |  | ref |
| laplace_w17 | 8133 | 205643 | - | oom | - | - | >600 | >600 | - |  |  |  |  |  |  |
| linreg_w8 | 372 | 5404 | 0.397 | 0.9 | 0.518 | >600 | 0.375 | 158 | >600 | alea | 3.9e-15 / 2.8e-14 | 4.7e-15 / 3.0e-14 | 4.1e-15 / 2.9e-14 |  | ref |
| linreg_w9 | 564 | 6754 | 0.702 | 2.79 | 1.4 | - | 0.732 | 160 | - | alea | 4.6e-15 / 1.6e-14 | 4.0e-15 / 1.4e-14 | 4.2e-15 / 1.5e-14 |  | ref |
| linreg_w10 | 756 | 8204 | 7.28 | 7.77 | 4.46 | - | 1.99 | 154 | - | alea | 6.3e-15 / 2.4e-14 | 5.1e-15 / 2.4e-14 | 6.2e-15 / 2.4e-14 |  | ref |
| linreg_w11 | 948 | 9788 | 12.5 | 26 | 29.7 | - | 8.38 | 171 | - | alea | 6.3e-15 / 9.1e-15 | 6.6e-15 / 1.0e-14 | 7.3e-15 / 1.0e-14 |  | ref |
| linreg_w12 | 1140 | 11532 | 36.1 | 106 | oom | - | 40 | 255 | - | alea | 4.9e-15 / 1.9e-14 | 4.3e-15 / 1.8e-14 |  |  | ref |
| linreg_w13 | 1332 | 13436 | 208 | 502 | - | - | 192 | 330 | - | alea | 8.0e-15 / 2.7e-14 | 1.9e-14 / 3.8e-14 |  |  | ref |
| linreg_w14 | 1524 | 15500 | oom | oom | - | - | >600 | >600 | - |  |  |  |  |  |  |
| markov_n10 | 2050 | 7887 | 1.28 | 0.468 | 3.39 | zero | 0.195 | 160 | 5.06 | alea | 1.7e-15 / 9.7e-15 | 1.7e-15 / 9.7e-15 | 1.4e-15 / 1.1e-12 |  | ref |
| markov_n100 | 20500 | 78897 | 10.7 | 8.88 | - | - | 2.47 | 164 | 302 | alea | 4.0e-15 / 2.7e-12 | 4.0e-15 / 2.7e-12 |  |  | ref |
| markov_n1000 | 205000 | 788997 | 99.5 | 356 | - | - | 91.2 | 237 | - | alea | 4.1e-13 / 4.1e-13 | 4.1e-13 / 4.1e-13 |  |  | ref |
| markov_n20 | 4100 | 15777 | 1.12 | 1.06 | oom | >600 | 0.48 | 164 | 17 | alea | 1.2e-14 / 1.5e-14 | 1.2e-14 / 1.7e-14 |  |  | ref |
| markov_n200 | 41000 | 157797 | 22.8 | 25.2 | - | - | 6.78 | 178 | >600 | alea | 1.3e-13 / 9.0e-12 | 1.3e-13 / 9.0e-12 |  |  | ref |
| markov_n2000 | 410000 | 1577997 | 212 | oom | - | - | 381 | 523 | - | alea | 4.8e-13 / 1.0e-12 |  |  |  | ref |
| markov_n50 | 10250 | 39447 | 4.4 | 3.38 | - | - | 1.18 | 161 | 84.3 | alea | 2.8e-14 / 2.8e-14 | 2.8e-14 / 2.8e-14 |  |  | ref |
| markov_n500 | 102500 | 394497 | 44.9 | 110 | - | - | 26.5 | 228 | - | alea | 1.9e-13 / 4.6e-13 | 1.9e-13 / 4.6e-13 |  |  | ref |
| multimodal_w11 | 4617 | 116721 | 501 | oom | oom | >600 | 50.3 | 259 | >600 | alea | 9.1e-15 / 1.8e-14 |  |  |  | ref |
| multimodal_w13 | 13837 | 144765 | oom | - | - | - | >600 | >600 | - |  |  |  |  |  |  |
| normal_mixture_mu1_w7 | 2583 | 25704 | 2.13 | 3.85 | 1.67 | >600 | 2.08 | 121 | >600 | alea | 2.9e-15 / 2.5e-14 | 2.2e-15 / 2.6e-14 | 2.9e-15 / 2.2e-14 |  | ref |
| normal_mixture_mu1_w9 | 5289 | 35812 | 202 | 370 | oom | - | 480 (2/3) | 594 (2/3) | - | alea | 1.3e-14 / 1.2e-13 | 1.4e-14 / 1.2e-13 |  |  | ref |
| normal_mixture_mu1_w11 | 7995 | 45920 | >600 | oom | - | - | >600 | >600 | - |  |  |  |  |  |  |
| normal_mixture_mu2_w7 | 2583 | 45770 | 3.14 | 5.15 | oom | >600 | 2.51 | 125 | >600 | alea | 1.3e-14 / 1.1e-13 | 1.1e-14 / 1.1e-13 |  |  | ref |
| normal_mixture_mu2_w9 | 7913 | 63750 | 199 | 591 | - | - | >600 | >600 | - | tddlog | ref |  |  |  |  |
| normal_mixture_mu2_w11 | 13243 | 81730 | >600 | oom | - | - | - | - | - |  |  |  |  |  |  |
| normal_mixture_theta_w9 | 10537 | 118560 | 40.8 | oom | oom | >600 | >600 | >600 | >600 | Ganak | 2.2e-16 / 5.2e-16 |  |  |  |  |
| normal_mixture_theta_w11 | 21115 | 152284 | oom | - | - | - | - | - | - |  |  |  |  |  |  |
| pedestrian_s10 | 89 | 3176 | 2.19 | 3.42 | - | - | 15 | 147 | - | alea | 3.3e-16 / 4.3e-14 | 3.9e-16 / 4.2e-14 |  |  | ref |
| pedestrian_s12 | 99 | 3784 | 6.06 | 6.44 | - | - | 32.8 | 202 | - | alea | 4.4e-16 / 4.3e-14 | 4.4e-16 / 4.3e-14 |  |  | ref |
| pedestrian_s16 | 119 | 5000 | 12 | 17.5 | - | - | 110 | 314 | - | alea | 3.3e-16 / 4.2e-14 | 4.4e-16 / 4.1e-14 |  |  | ref |
| pedestrian_s20 | 139 | 6216 | 79 | 40.1 | - | - | 265 | 363 | - | alea | 8.9e-16 / 4.9e-14 | 7.8e-16 / 5.0e-14 |  |  | ref |
| pedestrian_s24 | 159 | 7432 | 145 | 78.9 | - | - | >600 | >600 | - | tddlog | ref |  |  |  |  |
| pedestrian_s32 | 199 | 9864 | 383 | 204 | - | - | - | - | - | tddlog | ref |  |  |  |  |
| pedestrian_s4 | 59 | 1393 | 0.352 | 0.205 | 7.76 | >600 | 0.236 | 114 | >600 | alea | 6.7e-16 / 2.7e-14 | 5.6e-16 / 2.7e-14 | 1.7e-13 / 3.4e-13 |  | ref |
| pedestrian_s40 | 239 | 12296 | 339 | 477 | - | - | - | - | - | tddlog | ref |  |  |  |  |
| pedestrian_s6 | 69 | 1968 | 0.516 | 0.626 | >600 | - | 1.72 | 108 | - | alea | 7.8e-16 / 4.0e-14 | 2.8e-16 / 4.0e-14 |  |  | ref |
| pedestrian_s8 | 79 | 2568 | 2.23 | 1.53 | - | - | 5.62 | 121 | - | alea | 6.7e-16 / 4.1e-14 | 6.7e-16 / 4.1e-14 |  |  | ref |
| pi_w8 | 12 | 3759 | 0.168 | 0.213 | n/a | >600 | 0.08 | 178 | 2.39 | alea | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 |  |  | ref |
| pi_w10 | 16 | 6141 | 0.293 | 0.663 | n/a | - | 0.187 | 186 | 27.2 | alea | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 |  |  | ref |
| pi_w12 | 20 | 9099 | 0.688 | 2.58 | n/a | - | 1.45 | 182 | wrong | alea | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 |  |  | ref |
| pi_w14 | 24 | 12633 | 1.63 | 13.7 | n/a | - | 30.4 | 267 | >600 | alea | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 |  |  | ref |
| pi_w16 | 28 | 16743 | 7.33 | 92.3 | n/a | - | 305 | 479 | - | alea | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 |  |  | ref |
| pi_w18 | 32 | 21429 | 37.4 | >600 | n/a | - | >600 | >600 | - | tddlog | ref |  |  |  |  |
| pi_w20 | 36 | 26691 | 184 | - | n/a | - | - | - | - | tddlog | ref |  |  |  |  |
| pi_w22 | 40 | 32529 | >600 | - | n/a | - | - | - | - |  |  |  |  |  |  |
| population_s1 | 2540 | 8332 | 0.841 | >600 | oom | >600 | 23.9 | 234 | >600 | alea | 1.1e-15 / 9.2e-14 |  |  |  | ref |
| population_s12 | 34957 | 115507 | 26.1 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| population_s16 | 46780 | 154582 | 47.2 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| population_s2 | 5587 | 18373 | 11.6 | - | - | - | 326 | 479 | - | alea | 3.7e-15 / 6.6e-15 |  |  |  | ref |
| population_s24 | 70426 | 232732 | 36 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| population_s3 | 8637 | 28421 | 11.8 | - | - | - | >600 | >600 | - | ace-Xlog | 5.6e-14 / 7.5e-13 |  |  |  |  |
| population_s32 | 94072 | 310882 | 48.7 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| population_s4 | 11311 | 37357 | 13.4 | - | - | - | - | - | - | ace-Xlog | 1.0e-13 / 3.5e+05 |  |  |  |  |
| population_s48 | 141364 | 467182 | 74.4 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| population_s6 | 17410 | 57448 | 16.8 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| population_s64 | 188656 | 623482 | 109 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| population_s8 | 23134 | 76432 | 38.8 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| spacex_w13 | 279 | 2037 | 0.716 | 0.877 | oom | >600 | 0.755 | 73.8 | >600 | alea | 2.2e-16 / 9.4e-16 | 4.1e-15 / 6.1e-15 |  |  | ref |
| spacex_w15 | 357 | 2423 | 5.41 | 3.56 | - | - | 4.61 | 82.7 | - | alea | 5.4e-15 / 8.7e-15 | 5.4e-15 / 8.7e-15 |  |  | ref |
| spacex_w17 | 435 | 2809 | 33.4 | 15.7 | - | - | 28.1 | 142 | - | alea | 2.8e-14 / 7.0e-14 | 2.8e-14 / 7.0e-14 |  |  | ref |
| spacex_w19 | 513 | 3195 | 130 | 74 | - | - | 204 | 277 | - | alea | 8.5e-14 / 2.2e-13 | 8.5e-14 / 2.2e-13 |  |  | ref |
| spacex_w21 | 577 | 3553 | - | oom | - | - | >600 | >600 | - |  |  |  |  |  |  |
| spacex_p16 | 293 | 1752 | 57.7 | 29.2 | oom | - | 102 | 284 | - | alea | 6.2e-14 / 1.9e-13 | 6.2e-14 / 1.9e-13 |  |  | ref |
| spacex_p256 | 2970 | 23577 | - | - | - | - | >600 | >600 | - |  |  |  |  |  |  |
| spacex_p4 | 106 | 635 | 0.093 | 4.75 | 13.2 | >600 | 33.6 | 158 | >600 | alea | 6.7e-16 / 1.6e-14 | 1.1e-14 / 1.9e-14 | 4.4e-14 / 7.7e-14 |  | ref |
| spacex_p64 | 920 | 6092 | oom | oom | - | - | 359 | 432 | - | alea |  |  |  |  | ref |
| trueskill_w8 | 312 | 3868 | 3.24 | 2.02 | oom | >600 | 2.51 | 84.1 | >600 | alea | 1.7e-16 / 3.4e-16 | 1.7e-16 / 3.4e-16 |  |  | ref |
| trueskill_w10 | 376 | 5592 | 61.2 | 35.1 | - | - | 123 | 329 | - | alea | 1.7e-16 / 3.4e-16 | 1.7e-16 / 3.4e-16 |  |  | ref |
| trueskill_w12 | 440 | 7316 | oom | oom | - | - | >600 | >600 | - |  |  |  |  |  |  |
| tug_of_war_w16 | 392 | 2044 | 0.542 | 32.7 | oom | >600 | 7.56 | 131 | >600 | alea | 1.1e-15 / 2.1e-15 | 7.2e-15 / 1.4e-14 |  |  | ref |
| tug_of_war_w18 | 456 | 2292 | 0.617 | 151 | - | - | 37.7 | 216 | - | alea | 2.2e-16 / 4.4e-16 | 6.8e-15 / 1.4e-14 |  |  | ref |
| tug_of_war_w20 | 520 | 2540 | 0.735 | oom | - | - | 197 | 310 | - | alea | 8.3e-16 / 1.7e-15 |  |  |  | ref |
| tug_of_war_w22 | 550 | 2720 | 0.771 | - | - | - | >600 | >600 | - | Ganak | 5.0e-16 / 1.0e-15 |  |  |  |  |
| tug_of_war_w24 | 610 | 2960 | 0.472 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| tug_of_war_w26 | 670 | 3200 | 0.945 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| tug_of_war_w28 | 722 | 3416 | 1.02 | - | - | - | - | - | - | tddlog | ref |  |  |  |  |
| weekend_w18 | 49151 | 368566 | 25.2 | >600 | 22.7 | 268 | 14.6 | 138 | oom | alea | 3.3e-16 / 8.9e-16 |  | 3.3e-16 / 8.9e-16 | 7.3e-14 / 1.9e-13 | ref |
| weekend_w20 | 65535 | 417718 | 30 | - | 27.1 | 359 | 16.3 | 147 | - | alea | 2.8e-16 / 7.4e-16 |  | 2.8e-16 / 7.4e-16 | 1.0e-13 / 2.7e-13 | ref |
| weekend_w22 | 81919 | 466870 | 33.7 | - | 31 | zero | 15.5 | 142 | - | alea | 5.6e-17 / 1.5e-16 |  | 5.6e-17 / 1.5e-16 |  | ref |
| weekend_w24 | 98303 | 516022 | 37.5 | - | 34.1 | zero | 17.2 | 151 | - | alea | 8.3e-16 / 2.2e-15 |  | 8.3e-16 / 2.2e-15 |  | ref |
| weekend_w26 | 114687 | 565174 | 42.6 | - | 38 | zero | 19.5 | 157 | - | alea | 9.4e-16 / 2.5e-15 |  | 9.4e-16 / 2.5e-15 |  | ref |
| weekend_w28 | 131071 | 614326 | 44.7 | - | 41.6 | zero | 20.7 | 154 | - | alea | 6.1e-16 / 1.6e-15 |  | 6.1e-16 / 1.6e-15 |  | ref |
| weekend_w30 | 147455 | 663478 | 49.2 | - | 44.9 | zero | 22.1 | 161 | - | alea | 1.6e-15 / 4.2e-15 |  | 1.6e-15 / 4.2e-15 |  | ref |
| weekend_p1024 | 28671 | 131010 | 8.29 | >600 | 7.62 | zero | 2.33 | 102 | oom | alea | 2.8e-16 / 7.4e-16 |  | 2.8e-16 / 7.4e-16 |  | ref |
| weekend_p16 | 639 | 2118 | 0.134 | 0.113 | 0.124 | zero | 0.032 | 94.1 | 1.05 | alea | 5.0e-16 / 1.3e-15 | 4.4e-16 / 1.2e-15 | 2.8e-16 / 7.4e-16 |  | ref |
| weekend_p16384 | 327679 | 2031530 | 196 | - | 176 | >600 | 225 | 320 | - | alea | 6.1e-16 / 1.6e-15 |  | 6.1e-16 / 1.6e-15 |  | ref |
| weekend_p256 | 8191 | 33230 | 4.67 | 22.4 | 1.81 | zero | 0.496 | 96.2 | 276 | alea | 1.4e-15 / 3.7e-15 | 1.4e-15 / 3.7e-15 | 1.3e-15 / 3.6e-15 |  | ref |
| weekend_p4 | 175 | 530 | 0.028 | 0.025 | 0.027 | zero | 0.015 | 96.7 | 0.283 | alea | 1.7e-15 / 4.4e-15 | 1.7e-15 / 4.4e-15 | 1.4e-15 / 3.7e-15 |  | ref |
| weekend_p4096 | 98303 | 516022 | 39.1 | - | 34.9 | zero | 17.7 | 146 | - | alea | 8.3e-16 / 2.2e-15 |  | 8.3e-16 / 2.2e-15 |  | ref |
| weekend_p64 | 2303 | 8410 | 1.98 | 1.16 | 0.444 | zero | 0.106 | 93.6 | 10.9 | alea | 5.6e-17 / 1.5e-16 | 5.6e-17 / 1.5e-16 | 2.2e-16 / 5.9e-16 |  | ref |
| weekend_p65536 | 1048575 | 7995294 | oom | - | oom | - | >600 | >600 | - |  |  |  |  |  |  |
| wmi_path_n1024 | 8192 | 91060 | 9.33 | 4.76 | n/a | - | oom | oom | - | ace-auto | 1.6e-15 / 2.2e-15 | 1.6e-15 / 2.2e-15 |  |  |  |
| wmi_path_n128 | 1024 | 11316 | 1.16 | 0.673 | n/a | - | 6.96 | 112 | - | alea | 1.0e-15 / 1.4e-15 | 1.0e-15 / 1.4e-15 |  |  | ref |
| wmi_path_n16 | 128 | 1348 | 0.137 | 0.073 | n/a | 414 | 0.121 | 101 | >600 | alea | 1.1e-16 / 1.6e-16 | 1.1e-16 / 1.6e-16 |  | 3.5e-13 / 5.0e-13 | ref |
| wmi_path_n16384 | 131072 | 1458100 | 172 | 84.2 | n/a | - | - | - | - | tddlog | ref |  |  |  |  |
| wmi_path_n256 | 2048 | 22708 | 2.15 | 1.17 | n/a | - | 28.6 | 181 | - | alea | 5.0e-15 / 7.1e-15 | 5.0e-15 / 7.1e-15 |  |  | ref |
| wmi_path_n32 | 256 | 2772 | 0.276 | 0.145 | n/a | >600 | 0.41 | 92.3 | - | alea | 3.3e-16 / 4.7e-16 | 3.3e-16 / 4.7e-16 |  |  | ref |
| wmi_path_n4096 | 32768 | 364468 | 48.8 | 19.9 | n/a | - | - | - | - | tddlog | ref |  |  |  |  |
| wmi_path_n512 | 4096 | 45492 | 4.46 | 2.43 | n/a | - | 122 | 337 | - | alea | 1.6e-14 / 2.3e-14 | 1.6e-14 / 2.3e-14 |  |  | ref |
| wmi_path_n64 | 512 | 5620 | 0.57 | 0.303 | n/a | - | 1.66 | 96.1 | - | alea | 5.6e-16 / 7.8e-16 | 5.6e-16 / 7.8e-16 |  |  | ref |
| wmi_path_n65536 | 524288 | 5832628 | >600 | 365 | n/a | - | - | - | - |  |  |  |  |  |  |
| wmi_star_n1024 | 8200 | 91149 | 10.7 | 6.11 | n/a | - | >600 | >600 | - | ace-Xlog | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 |  |  |  |
| wmi_star_n128 | 1032 | 11405 | 1.19 | 0.725 | n/a | - | 8.08 | 113 | 78.4 | alea | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 |  |  | ref |
| wmi_star_n16 | 136 | 1437 | 0.157 | 0.088 | n/a | 305 | 0.123 | 97.3 | wrong | alea | 1.1e-16 / 2.1e-16 | 1.1e-16 / 2.1e-16 |  | 4.1e-14 / 7.6e-14 | ref |
| wmi_star_n16384 | 131080 | 1458189 | 187 | 105 | n/a | - | - | - | - | tddlog | ref |  |  |  |  |
| wmi_star_n256 | 2056 | 22797 | 2.36 | 1.48 | n/a | - | 36.7 | 197 | 162 | alea | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 |  |  | ref |
| wmi_star_n32 | 264 | 2861 | 0.298 | 0.177 | n/a | >600 | 0.413 | 88.2 | wrong | alea | 3.3e-16 / 6.4e-16 | 3.3e-16 / 6.4e-16 |  |  | ref |
| wmi_star_n4096 | 32776 | 364557 | 55.1 | 25.5 | n/a | - | - | - | - | tddlog | ref |  |  |  |  |
| wmi_star_n512 | 4104 | 45581 | 5 | 3 | n/a | - | 165 | 251 | oom | alea | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 |  |  | ref |
| wmi_star_n64 | 520 | 5709 | 0.606 | 0.347 | n/a | - | 1.88 | 94.7 | 37.8 | alea | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  | ref |
| wmi_star_n65536 | 524296 | 5832717 | >600 | 459 | n/a | - | - | - | - |  |  |  |  |  |  |
| zeroone_w1_w10 | 32 | 2511 | 0.338 | 0.308 | 0.31 | 4.79 | 0.433 | 156 | 3.2 | alea | 7.8e-16 / 1.5e-15 | 7.8e-16 / 1.5e-15 | 3.3e-16 / 8.1e-16 | 4.7e-13 / 8.8e-13 | ref |
| zeroone_w1_w12 | 36 | 4184 | 0.602 | 0.5 | 2.37 | 40 | 0.688 | 155 | 195 | alea | 7.8e-16 / 1.5e-15 | 7.8e-16 / 1.5e-15 | 5.9e-14 / 1.1e-13 | 4.8e-13 / 9.6e-13 | ref |
| zeroone_w1_w14 | 40 | 6254 | 0.826 | 0.796 | 7.02 | 354 | 1.09 | 157 | >600 | alea | 1.9e-15 / 3.8e-15 | 1.2e-15 / 2.4e-15 | 1.6e-12 / 3.2e-12 | 4.2e-13 / 8.0e-13 | ref |
| zeroone_w1_w16 | 44 | 8780 | 1.3 | 1.21 | 16.6 | >600 | 1.89 | 160 | - | alea | 3.9e-15 / 7.7e-15 | 1.0e-15 / 2.0e-15 | 2.8e-12 / 5.7e-12 |  | ref |
| zeroone_w1_w18 | 48 | 11762 | 2.52 | 2.04 | 51.5 | - | 4.26 | 170 | - | alea | 1.6e-15 / 3.1e-15 | 1.6e-15 / 3.1e-15 | 7.8e-12 / 1.6e-11 |  | ref |
| zeroone_w1_w20 | 52 | 15200 | 3.16 | 4.01 | 95.3 | - | 13.9 | 198 | - | alea | 4.2e-15 / 8.4e-15 | 3.3e-15 / 6.7e-15 | 1.7e-11 / 3.5e-11 |  | ref |
| zeroone_w1_w22 | 56 | 19094 | 4.39 | 10.6 | oom | - | 56.1 | 319 | - | alea | 6.2e-15 / 1.2e-14 | 4.1e-15 / 8.2e-15 |  |  | ref |
| zeroone_w1_w26 | 64 | 28250 | 6.52 | 137 | - | - | >600 | >600 | - | tddlog | ref |  |  |  |  |
| zeroone_w1_w30 | 72 | 39230 | 12.7 | >600 | - | - | - | - | - | tddlog | ref |  |  |  |  |
| zeroone_w1_w34 | 928 | 54788 | oom | - | - | - | - | - | - |  |  |  |  |  |  |
| zeroone_w2_w13 | 38 | 5230 | 0.66 | 0.637 | 0.897 | 131 | 0.926 | 155 | >600 | alea | 2.0e-15 / 3.8e-15 | 8.9e-16 / 1.5e-15 | 3.2e-15 / 6.1e-15 | 4.9e-13 / 9.7e-13 | ref |
| zeroone_w2_w15 | 42 | 7560 | 1.05 | 0.961 | 1.88 | >600 | 1.7 | 157 | - | alea | 1.0e-14 / 1.4e-14 | 7.2e-15 / 1.0e-14 | 4.2e-15 / 6.9e-15 |  | ref |
| zeroone_w2_w17 | 46 | 10346 | 1.91 | 1.52 | 5.75 | - | 5.81 | 168 | - | alea | 3.2e-14 / 5.8e-14 | 2.9e-14 / 5.5e-14 | 4.7e-15 / 9.3e-15 |  | ref |
| zeroone_w2_w19 | 50 | 13588 | 3.6 | 2.81 | 21.3 | - | 23 | 209 | - | alea | 1.1e-13 / 1.6e-13 | 1.1e-13 / 1.6e-13 | 1.8e-15 / 3.5e-15 |  | ref |
| zeroone_w2_w21 | 54 | 17286 | 4.43 | 6.35 | oom | - | 98.4 | 257 | - | alea | 4.0e-15 / 7.9e-15 | 1.2e-12 / 1.9e-12 |  |  | ref |
| zeroone_w2_w23 | 58 | 21440 | 4.86 | 18.6 | - | - | >600 | >600 | - | tddlog | ref |  |  |  |  |
| zeroone_w2_w25 | 62 | 26050 | 11 | 66 | - | - | - | - | - | tddlog | ref |  |  |  |  |
| zeroone_w2_w27 | 66 | 31116 | 10.2 | 274 | - | - | - | - | - | tddlog | ref |  |  |  |  |
| zeroone_w2_w29 | 70 | 36638 | 18.6 | >600 | - | - | - | - | - | tddlog | ref |  |  |  |  |
| zeroone_w2_w31 | 772 | 44920 | >600 | - | - | - | - | - | - |  |  |  |  |  |  |

Answers. The largest differences from the reference: `auto` 2.2e-13
relative (8.5e-14 absolute, spacex_w19) against Alea, none in common
against ProbLog's eight printed digits, 2.1e-14 relative (1.2e-14
absolute, clickGraph_w14) against Ganak; `program` 2.6e-12 relative
(1.3e-12 absolute, addFun_sum_w21) and `decision` 3.6e-11 relative
(1.8e-11 absolute, addFun_sum_w17) against Alea; Dice (earlier run)
1.2e-12 relative (4.7e-13 absolute, clinicalTrial2_w8) against Alea
where it prints answers other than 0.

ProbLog's d-DNNF is wrong on `bb_clickGraph_w14`: it gives 0.68647591
for `e1(3)` with all twelve queries and with that query alone, where
tddlog gives 0.6705662206624646. tddlog's answers converge as the width
grows (0.70508, 0.67611, 0.67178, 0.67085, 0.67062, 0.67057, 0.67055
from 4 to 16 bits), and ProbLog's agree with them to its eight digits up
to 12 bits. An independent count settles it: the circuit's Clark
completion as a weighted CNF, one count for the evidence and one for
each query with it, by Ganak in exact rational arithmetic (`--mode 1
--prob 0`), gives 0.6705662206624644, which tddlog matches to 2e-16
absolute. It is also off the reference on `bb_pi_w12` (by 1.7e-01),
where Alea, tddlog and aspmc agree.

The same counts are the reference where Alea does not answer and tddlog
alone, or tddlog and ProbLog, do (computed on the development machine,
untimed): tddlog agrees with them to 1e-14 relative on clickGraph from
10 to 16 bits, clinicalTrial1 at 10 bits, clinicalTrial2 from 14 to 18
bits, normal_mixture_theta at 9 bits and tug_of_war at 22 bits (on
tug_of_war at 22 bits through the count of the query's negation).
tddlog's answers on addFun_max from 23 to 29 bits, addFun_sum from 23 to
29 bits, clinicalTrial1 at 12 and 14 bits, conjugate_gaussians from 25
to 29 bits, genes from 5 to 100 genes, normal_mixture_mu2 at 9 bits, pi
at 18 and 20 bits, tug_of_war from 24 to 28 bits, zeroone_w1 at 26 and
30 bits and zeroone_w2 from 23 to 29 bits have no Ganak count and no
other system's answer: their only check is the agreement of tddlog's
arms, to 3e-06 relative, and on addFun_max from 23 to 29 bits,
addFun_sum from 23 to 29 bits, clinicalTrial1 at 12 and 14 bits,
conjugate_gaussians from 25 to 29 bits, pi at 18 and 20 bits, tug_of_war
from 24 to 28 bits, zeroone_w1 at 30 bits and zeroone_w2 at 29 bits one
arm alone answers.

### Programs of other papers

The suite's other fifteen families (117 instances; with the families above it
holds 38 families and 281 instances), from
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
| addFun_max (pieces) | Fig. 13 | p256 (175 s), >600 at p1024 | oom at p4 | oom at p4 | >600 at p4 |
| altermu2 (pieces) | Fig. 13 | p64 (26.3 s), >600 at p1024 | p4 (138 s), oom at p16 | oom at p4 | >600 at p4 |
| fair_dt14 | FairSquare | w40 (97.2 s) | w16 (62.9 s), oom at w20 | w12 (274 s), oom at w16 | w20 (20.6 s), >600 at w24 |
| fair_dt16 | FairSquare | w32 (0.694 s) | w32 (0.44 s) | w32 (0.228 s) | w32 (0.069 s) |
| fair_dt4 | FairSquare | w40 (90.9 s) | w16 (49.9 s), oom at w20 | w12 (210 s), oom at w16 | w20 (16.9 s), >600 at w24 |
| fair_dt44 | FairSquare | w32 (0.602 s) | w32 (0.434 s) | w32 (0.249 s) | w32 (0.072 s) |
| laplace | bit-blasting repo | w13 (10.2 s), >600 at w15 | w15 (416 s), oom at w17 | w13 (9.52 s), oom at w15 | w15 (146 s), >600 at w17 |
| linreg | R2, PSI | w13 (208 s), oom at w14 | w13 (502 s), oom at w14 | w11 (29.7 s), oom at w12 | w13 (192 s), >600 at w14 |
| markov | SPPL | n2000 (212 s) | n1000 (356 s), oom at n2000 | n10 (3.39 s), oom at n20 | n2000 (381 s) |
| pedestrian | GuBPI | s40 (339 s) | s40 (477 s) | s4 (7.76 s), >600 at s6 | s20 (265 s), >600 at s24 |
| population | Genfer | s64 (109 s) | >600 at s1 | oom at s1 | s2 (326 s), >600 at s3 |
| spacex (pieces) | Fig. 13 | p16 (57.7 s), oom at p64 | p16 (29.2 s), oom at p64 | p4 (13.2 s), oom at p16 | p64 (359 s), >600 at p256 |
| weekend (pieces) | Fig. 13 | p16384 (196 s), oom at p65536 | p256 (22.4 s), >600 at p1024 | p16384 (176 s), oom at p65536 | p16384 (225 s), >600 at p65536 |
| wmi_path | WMI | n16384 (172 s), >600 at n65536 | n65536 (365 s) | n/a at n16 | n512 (122 s), oom at n1024 |
| wmi_star | WMI | n16384 (187 s), >600 at n65536 | n65536 (459 s) | n/a at n16 | n512 (165 s), >600 at n1024 |

- `auto` answers a larger size than Alea on eight families (addFun_max (pieces), altermu2 (pieces), fair_dt14, fair_dt4, pedestrian, population, wmi_path and wmi_star), the same size on five (fair_dt16, fair_dt44, linreg, markov and weekend (pieces)) and a smaller one on two (laplace and spacex (pieces)).
- A fixed vtree reaches further than `auto` on laplace, wmi_path and wmi_star.
- `auto` and Alea both answer 63 instances: `auto` takes 1.22e+03 s in all and Alea's warm time 2.34e+03 s; of the 59 where either takes 0.2 s or more, `auto` is faster on 24.
- `program` and Alea both answer 56 instances: `program` takes 1.97e+03 s in all and Alea's warm time 1.47e+03 s; of the 52 where either takes 0.2 s or more, `program` is faster on 19.
- `decision` and Alea both answer 29 instances: `decision` takes 778 s in all and Alea's warm time 301 s; of the 24 where either takes 0.2 s or more, `decision` is faster on 2.
- 444 answers of the arms were checked against Alea's where it answers; the largest difference is 4.8e-13 absolute.
- The population model's answers are the bits of a number; its expectation at 1, 2, 3, 4, 6, 8, 12 and 16 seasons agrees with a forward filter of the model to 7.5e-07 (24 answers).

`auto` is slower than Alea's warm time, or fails where Alea answers, on 40 instances where either takes 0.1 s or more (failures: laplace_w15 >600, spacex_p64 oom). The twelve largest ratios:

| instance | auto | Alea |
|---|---:|---:|
| laplace_w15 | >600 | 146 |
| fair_dt14_w12 | 7.97 | 0.191 |
| spacex_p64 | oom | 359 |
| fair_dt4_w12 | 4.54 | 0.201 |
| weekend_p64 | 1.98 | 0.106 |
| fair_dt4_w16 | 15.3 | 0.933 |
| fair_dt14_w16 | 14.6 | 1.08 |
| fair_dt16_w24 | 0.832 | 0.067 |
| fair_dt16_w20 | 0.804 | 0.069 |
| fair_dt44_w20 | 0.723 | 0.068 |
| fair_dt16_w28 | 0.699 | 0.068 |
| fair_dt44_w28 | 0.662 | 0.065 |

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
  both, show how far two runs differ: on the 73 instances it answers in
  both, its warm time totals 4694 s in the release run against 3048 s in
  the earlier run, and is from 1.19 to 2.00 times the earlier run's
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
| pi | 0.464602 | 1e-4 | 1.63 | 30.4 | 15.3 (1e6, 6e-5) | 2.2 (1e6, 9e-5) | err 2e-4 (1e7, 27.1 s) | err 2e-3 (1e5, 1.51 s) | 0.252, integral left |
| weekend | 0.374206 | 2e-8 | 37.5 | 17.2 | other model (1) | other model (2) | other model (2) | other model (2) | 0.083, closed form |
| spacex | 30.0046 | 7e-4 | 130 | 204 | err 2e-3 (1e6, 14.5 s) | err 3e-3 (1e6, 7.16 s) | err 1e-2 (1e6, 9.37 s) | err 2e-2 (1e5, 3.07 s) | 0.177, integral left |
| GPA | 0.611511 | 1e-16 | 44.5 | skipped | n/a | err 7e-3 (1e3, 243 s) | err 2e-3 (1e4, 208 s) | err 8e-3 (1e3, 8.24 s) | 1.39, closed form |
| tug_of_war | 0.5 | 5e-7 | 0.771 | >600 | err 4e-5 (1e7, 51.9 s) | err 4e-4 (1e6, 431 s) | err 7e-4 (1e7, 97.5 s) | err 2e-4 (1e5, 114 s) | 0.537, integral left |
| altermu2 | 0.155062 | none | memout | >600 | err 7e-3 (1e6, 45.4 s) | >600 | err 8e-1 (1e7, 390 s) | err 6e-1 (1e4, 18.4 s) | 24.8, closed form |
| conjugate_gaussians | 1 | 5e-6 | 1.86 | 230 | err 3e-4 (1e6, 9.01 s) | err 2e-4 (1e6, 70.7 s) | err 3e-4 (1e7, 67.5 s) | err 2e-3 (1e5, 42.3 s) | 0.019, closed form |
| normal_mixture_theta | 0.285714 | 5e-5 | 40.8 | >600 | 35.9 (1e6, 4e-5) | >600 | err 4e-4 (1e7, 270 s) | err 1e-3 (1e4, 12.2 s) | >600 |
| normal_mixture_mu1 | -9.70236 | 5e-3 | 202 | 480 | 0.432 (1e4, 4e-3) | >600 | 27.7 (1e6, 2e-3) | err 1e-2 (1e4, 12.4 s) | >600 |
| normal_mixture_mu2 | 9.65795 | 4e-3 | 199 | >600 | 0.207 (1e3, 3e-3) | >600 | 27.5 (1e6, 2e-3) | err 3e-2 (1e4, 12.4 s) | >600 |
| zeroone_w1 | 0.0565823 | 9e-5 | 1.3 | 1.89 | err 2e-1 (1e5, 102 s) | other model (3) | other model (3) | other model (3) | >600 |
| zeroone_w2 | 3.68883 | 5e-4 | 3.6 | 23 | err 1e-1 (1e5, 102 s) | other model (4) | other model (4) | other model (4) | >600 |
| coinBias | 0.416667 | 2e-7 | 201 | 416 | err 1e-4 (1e7, 31.5 s) | err 1e-4 (1e6, 181 s) | err 2e-4 (1e6, 5.7 s) | err 6e-4 (1e5, 164 s) | 0.221, closed form |
| addFun_sum | 0 | 4e-6 | 1.21 | >600 | err 1e-3 (1e6, 9.79 s) | err 7e-4 (1e6, 7.28 s) | err 9e-4 (1e7, 80.3 s) | err 3e-3 (1e5, 2.27 s) | n/a |
| clickGraph | 0.614154 | 2e-3 | 0.58 | >600 | 0.442 (1e4, 6e-4) | 21.6 (1e4, 2e-3) | 114 (1e7, 1e-3) | err 4e-3 (1e5, 365 s) | n/a |
| trueskill | 0.5 | 3e-3 | 61.2 | 123 | 1.36 (1e5, 1e-3) | 5.91 (1e5, 4e-4) | 15.2 (1e6, 3e-3) | 17.7 (1e5, 2e-3) | 0.154, integral left |
| clinicalTrial1 | 0.170362 | 5e-16 | 3.39 | 347 | err 2e-3 (1e6, 311 s) | failed | failed | failed | >600 |
| clinicalTrial2 | 0.285714 | 7e-7 | 0.049 | 374 | err 1e-4 (1e7, 86.8 s) | other model (5) | other model (5) | other model (5) | 0.727, closed form |
| addFun_max | 0.56419 | 3e-7 | 33.8 | >600 | err 6e-4 (1e6, 9.96 s) | err 7e-4 (1e6, 6.78 s) | err 4e-4 (1e7, 67.6 s) | err 1e-3 (1e5, 2.03 s) | 0.028, integral left |
| genes_5 | 0.5 | defect | 25.7 | >600 | err 2e-4 (1e6, 119 s) | >600 | err 3e-4 (1e7, 86.9 s) | err 4e-3 (1e4, 1.63 s) | 4.77, closed form |
| genes_10 | 0.5 | defect | 25.2 | skipped | err 2e-4 (1e5, 572 s) | >600 | err 2e-4 (1e7, 134 s) | err 2e-4 (1e5, 5.36 s) | 75.2, closed form |
| genes_15 | 0.5 | defect | 25.2 | skipped | err 1e-3 (1e3, 365 s) | >600 | err 1e-3 (1e7, 204 s) | err 4e-4 (1e4, 2.06 s) | >600 |
| genes_20 | 0.5 | defect | 25.4 | skipped | >600 | >600 | err 5e-4 (1e7, 251 s) | err 8e-4 (1e5, 8.62 s) | >600 |
| genes_25 | 0.5 | defect | 25.2 | skipped | failed | >600 | err 2e-3 (1e7, 313 s) | err 5e-4 (1e5, 10.1 s) | >600 |
| genes_30 | 0.5 | defect | 25 | skipped | >600 | >600 | err 2e-3 (1e7, 372 s) | err 1e-3 (1e5, 12 s) | memout |
| genes_40 | 0.5 | defect | 25.3 | skipped | failed | >600 | err 2e-3 (1e7, 492 s) | err 7e-4 (1e5, 15.9 s) | >600 |
| genes_50 | 0.5 | defect | 26 | skipped | failed | >600 | err 3e-3 (1e6, 65.3 s) | err 2e-4 (1e4, 3.84 s) | >600 |
| multimodal | 3.88889 | none | memout | >600 | err 9e0 (1e5, 1.5 s) | >600 | err 1e0 (1e2, 1.06 s) | err 1e-1 (1e4, 3.54 s) | n/a |
| conjugate_ex3 | 5.66667 | 7e-3 | skipped | 383 | 0.907 (1e5, 1e-3) | >600 | err 9e-1 (1e7, 47.8 s) | err 1e0 (1e5, 32.8 s) | n/a |

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
variable it queries: 130 instances in 13 families. `gen_roulette.py`
writes each program's ProbLog and Dice versions with the same
probabilities; Alea.jl runs the Dice programs (as on the Dice paper's
programs), and Roulette the artifact's programs in its upstream
language, only the `#lang` line and the `query` forms changed. The paper
runs the network and the interpreter in Roulette only; the Dice programs
here hold the message as a flag and four Booleans and unroll the
interpreter's loop. The reference is a dynamic program over each model's
states (the clique tree on the networks), `--exact`, in a pass of its
own.

### Results: tddlog 0.1.0, Dice, Alea.jl, Roulette and Roulette+tddlog

One job on a lab machine (16 physical cores, 125 GB) on 2026-10-22, 4
lanes of 31 GB, each process limited to its lane's memory. tddlog 0.1.0
as of 2026-10-22 (tididi c7c0c5d6, goatd e592d1e) in its three arms,
Dice 1.0 with an unlimited stack and Alea.jl as on the bit-blasted
programs run on an instance back to back in one lane, in an order
rotated by instance and repetition; three repetitions, 600 s each; a
system that runs out of time or memory on a size is not run on the
program's larger sizes. Roulette (neuppl/roulette 60f77cb on Racket 9.3)
and Roulette+tddlog, tddlog on the circuit Roulette builds (baselines.md
section 11), ran again in a job of their own on a lab machine (36
physical cores, 503 GB) on 2026-11-01 and 02, 34 lanes of 29 GB, each
instance beside tddlog `auto` (built from 2ceeb514) in the same lane and
in the same way, and are compared with that `auto`, the column `auto`
(Roulette's job). Roulette's time is its run of the program after
Racket's start and the modules' load; `Roulette process` adds them.
Roulette+tddlog's is the same run up to the circuit, the writing of the
circuit and tddlog's run on it. The runs beside Roulette on its upstream
main and Roulette+tddlog's ran a build of 2026-11-01 (tididi c5642b97,
goatd e592d1e). The runs on Bayesian and Markov networks ran a build of
2026-10-28 (tididi ca759585, goatd e592d1e). The growth sizes ran a
build of 2026-10-25 with twin contraction (tididi b9e5064e, goatd
e592d1e) and a recursion over a successor relation grounded level by
level, which lacks only the release's sparse route (tididi 9fc0a599).
The runs beside Scallop ran a build of 2026-10-29 (tididi df7b84a3,
goatd e592d1e). The reruns of runs skipped without a failure on a
smaller size ran a build of 2026-10-24 that adds a vtree search run to
convergence (tididi ef11abfd, goatd e592d1e). The release's main
contains every solver change those builds have.

Of the 130 instances, four lie past the sizes at which every system had
failed, and two every system answers in under a second. Answers: `auto`
111, `program` 30, `decision` 16, Dice 45, Alea 80, Roulette 68,
Roulette+tddlog 91.

| family | sizes | `auto` | `program` | `decision` | Dice | Alea | Roulette | Roulette+tddlog |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| motivating | 1 000 to 9 000 000 | 1 000 000 (60.5 s) | memout | unsupported, memout | 10 000 (203 s) | 100 000 (55.4 s) | 1 000 000 (287 s) | 1 000 000 (409 s) |
| diamond | 1 000 to 6 000 000 | 1 000 000 (62 s) | memout | unsupported, memout | 10 000 (147 s) | 100 000 (46.6 s) | 300 000 (178 s) | 300 000 (263 s) |
| ladder | 1 000 to 3 000 000 | 1 000 000 (119 s) | memout | unsupported, memout | 10 000 (435 s) | 100 000 (45.9 s) | 300 000 (233 s) | 300 000 (370 s) |
| caesar | 25 to 100 000 | 20 000 (86.8 s) | memout | unsupported | zero, underflow, timeout | 2 000 (348 s) | 200 (2.44 s) | 10 000 (317 s) |
| hmm marginal likelihood | 100 to 900 | 900 (0.13 s) | memout | unsupported | 900 (0.44 s) | 900 (1.22 s) | 900 (2.27 s) | 900 (1.02 s) |
| hmm filtering marginal | 100 to 3 000 000 | 300 000 (50.8 s) | memout | unsupported, memout | zero, underflow, timeout | 10 000 (238 s) | 300 (0.3 s) | 30 000 (42 s) |
| hmm smoothing marginal | 100 to 3 000 000 | 300 000 (54.8 s) | memout | unsupported, memout | zero, underflow, timeout | 10 000 (232 s) | 300 (0.3 s) | 30 000 (42.6 s) |
| hmm filtering joint | 100 to 3 000 000 | 300 000 (59.3 s) | memout | unsupported, memout | 10 000 (76.5 s) | 3 000 (18.9 s) | 3 000 (40 s) | 10 000 (35.9 s) |
| hmm smoothing joint | 100 to 3 000 000 | 300 000 (47 s) | memout | unsupported, memout | 10 000 (81 s) | 10 000 (241 s) | 3 000 (36.4 s) | 10 000 (39.9 s) |
| network | 10 to 1 000 000 | 100 000 (129 s) | memout | unsupported, memout | 30 000 (207 s) | 1 000 (67.4 s) | 300 (22.8 s) | 30 000 (419 s) |
| hardware | 10 to 1 000 000 | 100 000 (180 s) | 1 000 (80.1 s) | unsupported, memout | 100 000 (295 s) | 1 000 (103 s) | 300 (22 s) | 1 000 (423 s) |
| hardware bitwidth | 5 to 100 000 000 | 100 000 000 (0.009 s) | 100 000 000 (0.009 s) | 100 000 000 (0.008 s) | 10 (0.62 s) | 10 000 000 (44.9 s) | 3 000 000 (263 s) | 3 000 000 (270 s) |
| bn | 9 networks | 9 (0.4 s) | 9 (7.1 s) | - | 9 (2.67 s) | 9 (2.55 s) | 9 (13.6 s) | 9 (13.8 s) |

The table above gives, a family a row, the largest size each system
answers and its time there (on the networks, how many it answers and
their total time), or its failures where it answers no size.

- Against `program`, over the 41 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, `auto`
  takes 5.17 s in all and `program` 6694 s; over the 30 both answer,
  1.88 s against 94.3 s. `auto` is faster on 6 of the 30 both answer and
  `program` on 0, both under 0.2 s or level on the other 24. `auto`
  alone answers 11, `program` alone 0.
- Against `decision`, over the 110 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, `auto`
  takes 1061 s in all and `decision` 56400 s; over the 16 both answer,
  0.15 s against 0.13 s. `auto` is faster on 0 of the 16 both answer and
  `decision` on 0, both under 0.2 s or level on the other 16. `auto`
  alone answers 94, `decision` alone 0.
- Against Dice, over the 83 instances given to both that either answers,
  a run that does not answer counted at the 600 s limit, `auto` takes
  618 s in all and Dice 24470 s; over the 45 both answer, 277 s against
  1670 s. `auto` is faster on 30 of the 45 both answer and Dice on 3,
  both under 0.2 s or level on the other 12. `auto` alone answers 38,
  Dice alone 0.
- Against Alea, over the 94 instances given to both that either answers,
  a run that does not answer counted at the 600 s limit, `auto` takes
  274 s in all and Alea 10051 s; over the 80 both answer, 53.2 s against
  1651 s. `auto` is faster on 42 of the 80 both answer and Alea on 0,
  both under 0.2 s or level on the other 38. `auto` alone answers 14,
  Alea alone 0. Against Alea's process, over the 94 instances given to
  both that either answers, a run that does not answer counted at the
  600 s limit, `auto` takes 274 s in all and the process 13113 s; over
  the 80 both answer, 53.2 s against 4713 s.
- Against Roulette, over the 90 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, `auto`
  takes 778 s in all and Roulette 14687 s; over the 68 both answer, 245
  s against 1487 s. `auto` is faster on 49 of the 68 both answer and
  Roulette on 0, both under 0.2 s or level on the other 19. `auto` alone
  answers 22, Roulette alone 0. Against Roulette's process, over the 90
  instances given to both that either answers, a run that does not
  answer counted at the 600 s limit, `auto` takes 778 s in all and the
  process 14738 s; over the 68 both answer, 245 s against 1538 s.
- Against Roulette+tddlog, over the 101 instances given to both that
  either answers, a run that does not answer counted at the 600 s limit,
  `auto` takes 1103 s in all and Roulette+tddlog 9640 s; over the 91
  both answer, 460 s against 3640 s. `auto` is faster on 69 of the 91
  both answer and Roulette+tddlog on 0, both under 0.2 s or level on the
  other 22. `auto` alone answers 10, Roulette+tddlog alone 0.
- Dice prints 0 for every answer (zero) on 11 instances: caesar_25,
  caesar_50, caesar_100, hmm_filtering_joint_100,
  hmm_filtering_joint_300, hmm_filtering_marginal_100,
  hmm_filtering_marginal_300, hmm_smoothing_joint_100,
  hmm_smoothing_joint_300, hmm_smoothing_marginal_100,
  hmm_smoothing_marginal_300.
- Dice prints 0 for every answer (underflow) on 18 instances:
  caesar_200, caesar_300, caesar_500, caesar_1000, caesar_2000,
  caesar_5000, caesar_10000, caesar_20000, hmm_filtering_joint_1000,
  hmm_filtering_marginal_1000, hmm_filtering_marginal_3000,
  hmm_filtering_marginal_10000, hmm_filtering_marginal_30000,
  hmm_smoothing_joint_1000, hmm_smoothing_marginal_1000,
  hmm_smoothing_marginal_3000, hmm_smoothing_marginal_10000,
  hmm_smoothing_marginal_30000.
- Roulette prints 0 for every answer (underflow) on 13 instances:
  caesar_300, caesar_500, caesar_1000, caesar_2000, caesar_5000,
  caesar_10000, caesar_20000, hmm_filtering_joint_1000,
  hmm_filtering_marginal_1000, hmm_filtering_marginal_3000,
  hmm_smoothing_joint_1000, hmm_smoothing_marginal_1000,
  hmm_smoothing_marginal_3000.

| instance | tddlog | program | decision | Dice | Alea | Roulette | Roulette+tddlog | `auto` (Roulette's job) | Alea process | Roulette process | ProbLog | ref | tddlog diff | program diff | decision diff | Dice diff | Alea diff | Roulette diff | Roulette+tddlog diff | `auto` (Roulette's job) diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| bn_alarm | 0.026 | 0.031 | n/a | 0.066 | 0.01 | 0.117 | 0.15 | 0.01 | 11.6 | 0.867 | 0.501 | exact | 6.7e-16 / 2.5e-14 | 5.6e-16 / 2.2e-15 |  | 1.1e-16 / 5.3e-16 | 5.6e-17 / 7.6e-16 | 1.1e-16 / 5.3e-16 | 1.1e-16 / 5.3e-16 | 6.7e-16 / 2.5e-14 |
| bn_cancer | 0.026 | 0.03 | n/a | 0.06 | 0.006 | 0.009 | 0.016 | 0.005 | 11.7 | 0.754 | 0.282 | exact | 1.1e-16 / 5.3e-16 | 1.1e-16 / 5.3e-16 |  | 2.8e-17 / 1.3e-16 | 1.1e-16 / 1.4e-16 | 0.0e+00 / 0.0e+00 | 2.8e-17 / 1.3e-16 | 1.1e-16 / 5.3e-16 |
| bn_hailfinder | 0.063 | 2.09 | n/a | 0.434 | 1.24 | 1.81 | 1.45 | 0.051 | 14.9 | 2.56 | >600 | exact | 5.0e-16 / 1.6e-15 | 5.0e-16 / 1.1e-15 |  | 4.9e-13 / 1.6e-12 | 1.7e-16 / 5.4e-16 | 2.2e-16 / 5.4e-16 | 2.2e-16 / 7.2e-16 | 5.0e-16 / 1.6e-15 |
| bn_hepar2 | 0.034 | 0.229 | n/a | 0.074 | 0.022 | 0.405 | 0.528 | 0.016 | 12.3 | 1.15 | 3.35 | exact | 2.8e-16 / 6.5e-16 | 5.6e-17 / 1.3e-16 |  | 4.7e-13 / 1.1e-12 | 5.6e-17 / 1.3e-16 | 2.5e-10 / 5.8e-10 | 2.5e-10 / 5.8e-10 | 2.8e-16 / 6.5e-16 |
| bn_insurance | 0.043 | 4.45 | n/a | 0.757 | 1.03 | 1.18 | 0.345 | 0.031 | 15.1 | 1.95 | >600 | exact | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 |  | 1.5e-11 / 2.6e-11 | 1.5e-11 / 2.7e-11 | 2.4e-11 / 1.4e-09 | 2.4e-11 / 1.4e-09 | 1.5e-11 / 2.7e-11 |
| bn_munin | 0.104 | 0.117 | n/a | 0.241 | 0.075 | 8.66 | 9.47 | 0.054 | 13.8 | 9.4 | 64.3 | exact | 4.5e-16 / 1.9e-13 | 6.5e-16 / 3.5e-13 |  | 3.0e-13 / 2.8e-12 | 5.6e-17 / 1.7e-14 | 9.6e-09 / 3.5e-06 | 9.6e-09 / 3.5e-06 | 4.5e-16 / 1.9e-13 |
| bn_pigs | 0.031 | 0.021 | n/a | 0.06 | 0.007 | 0.822 | 0.871 | 0.009 | 12.3 | 1.56 | 1.2 | exact | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 5.6e-17 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| bn_survey | 0.027 | 0.014 | n/a | 0.063 | 0.006 | 0.011 | 0.019 | 0.005 | 11.8 | 0.753 | 0.153 | exact | 4.7e-16 / 3.0e-15 | 1.4e-16 / 8.8e-16 |  | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.0e-16 | 1.1e-16 / 2.0e-16 | 1.1e-16 / 2.0e-16 | 4.7e-16 / 3.0e-15 |
| bn_water | 0.048 | 0.125 | n/a | 0.916 | 0.159 | 0.559 | 0.946 | 0.039 | 14.5 | 1.3 | 18.9 | exact | 2.8e-16 / 2.7e-14 | 5.0e-16 / 1.8e-14 |  | 9.9e-14 / 7.0e-13 | 1.1e-16 / 4.9e-16 | 7.2e-09 / 9.6e-08 | 7.2e-09 / 9.6e-08 | 2.8e-16 / 2.7e-14 |
| caesar_25 | 0.076 | oom | n/a | zero | 0.049 | 0.291 | 0.503 | 0.153 | 9.62 | 0.999 | 28.8 | exact | 5.1e-15 / 1.0e-13 |  |  |  | 7.8e-15 / 4.3e-13 | 4.6e-15 / 1.1e-13 | 5.1e-15 / 1.1e-13 | 5.1e-15 / 1.0e-13 |
| caesar_50 | 0.139 | - | n/a | zero | 0.102 | 0.56 | 1.01 | 0.296 | 9.72 | 1.27 | 61.4 | exact | 2.3e-15 / 2.2e-13 |  |  |  | 5.3e-15 / 8.5e-13 | 2.3e-15 / 2.2e-13 | 2.3e-15 / 2.3e-13 | 2.3e-15 / 2.2e-13 |
| caesar_100 | 0.294 | - | n/a | zero | 0.456 | 1.19 | 2.11 | 0.588 | 10.1 | 1.91 | 113 | exact | 4.4e-16 / 4.8e-13 |  |  |  | 4.4e-15 / 2.3e-12 | 1.1e-16 / 1.0e+00 | 5.1e-26 / 4.9e-13 | 4.4e-16 / 4.8e-13 |
| caesar_200 | 0.571 | - | n/a | underflow | 2.3 | 2.44 | 4.22 | 1.2 | 12 | 3.15 | 253 | exact | 1.4e-29 / 1.5e-12 |  |  |  | 1.8e-29 / 5.5e-12 | 9.6e-18 / 1.0e+00 | 1.4e-29 / 1.5e-12 | 1.4e-29 / 1.5e-12 |
| caesar_300 | 0.89 | - | n/a | underflow | 5.51 | underflow | 6.86 | 1.82 | 15.2 | underflow | 207 | exact | 3.6e-31 / 4.0e-12 |  |  |  | 1.2e-30 / 9.3e-12 |  | 3.6e-31 / 4.0e-12 | 3.6e-31 / 4.0e-12 |
| caesar_500 | 1.53 | - | n/a | underflow | 16.9 | underflow | 10.9 | 3.17 | 26.6 | underflow | 448 | exact | 2.2e-16 / 2.5e-12 |  |  |  | 1.8e-33 / 1.5e-11 |  | 2.3e-34 / 2.5e-12 | 2.4e-34 / 2.5e-12 |
| caesar_1000 | 3.22 | - | n/a | underflow | 74.6 | underflow | 22.8 | 6.62 | 84.9 | underflow | >600 | exact | 1.1e-16 / 3.2e-11 |  |  |  | 6.8e-79 / 7.4e-11 |  | 1.3e-78 / 3.2e-11 | 1.1e-16 / 3.2e-11 |
| caesar_2000 | 7.01 | - | n/a | underflow | 348 | underflow | 46.8 | 13.9 | 360 | underflow | - | exact | 3.3e-16 / 6.4e-11 |  |  |  | 2.1e-171 / 1.3e-10 |  | 1.0e-171 / 6.4e-11 | 1.1e-16 / 6.4e-11 |
| caesar_5000 | 17.5 | - | n/a | underflow | oom | underflow | 145 | 35.9 | oom | underflow | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  | 0.0e+00 / 0.0e+00 | 2.2e-16 / 2.2e-16 |
| caesar_10000 | 38.2 | - | n/a | underflow | - | underflow | 317 | 76.1 | - | underflow | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.1e-16 |
| caesar_20000 | 86.8 | - | n/a | underflow | - | underflow | >600 | 150 | - | underflow | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  | 3.3e-16 / 3.3e-16 |
| caesar_50000 | oom | - | n/a | >600 | - | >600 | - | oom | - | >600 | - | exact |  |  |  |  |  |  |  |  |
| caesar_100000 | oom | - | n/a | >600 | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |
| diamond_1000 | 0.038 | oom | n/a | 1.14 | 0.049 | 0.352 | 0.556 | 0.076 | 9.13 | 1.19 | 17 | exact | 1.1e-16 / 1.2e-16 |  |  | 4.1e-14 / 4.3e-14 | 1.1e-14 / 1.1e-14 | 3.3e-16 / 3.5e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.2e-16 |
| diamond_3000 | 0.11 | - | n/a | 12.3 | 0.171 | 1.02 | 1.78 | 0.236 | 10.9 | 1.84 | stack | exact | 1.1e-15 / 1.3e-15 |  |  | 3.9e-13 / 4.6e-13 | 3.1e-14 / 3.6e-14 | 3.0e-15 / 3.5e-15 | 8.9e-16 / 1.0e-15 | 1.1e-15 / 1.3e-15 |
| diamond_10000 | 0.399 | - | n/a | 147 | 0.629 | 4.14 | 6.62 | 0.76 | 18.1 | 4.87 | stack | exact | 7.7e-15 / 1.3e-14 |  |  | 4.5e-14 / 7.4e-14 | 1.0e-14 / 1.6e-14 | 1.8e-14 / 3.0e-14 | 7.3e-15 / 1.2e-14 | 7.7e-15 / 1.3e-14 |
| diamond_30000 | 1.21 | - | n/a | >600 | 2.1 | 15 | 23.8 | 2.26 | 44.9 | 15.7 | stack | exact | 4.2e-14 / 1.9e-13 |  |  |  | 4.6e-13 / 2.1e-12 | 3.5e-14 / 1.6e-13 | 1.7e-14 / 7.4e-14 | 4.2e-14 / 1.9e-13 |
| diamond_100000 | 4.46 | - | n/a | - | 46.6 | 49.7 | 78.6 | 8 | 380 | 50.5 | stack | exact | 9.1e-14 / 1.3e-11 |  |  |  | 9.5e-16 / 1.4e-13 | 6.0e-15 / 9.0e-13 | 2.9e-15 / 4.3e-13 | 9.1e-14 / 1.3e-11 |
| diamond_300000 | 17.9 | - | n/a | - | >600 | 178 | 263 | 25.2 | >600 | 179 | stack | exact | 5.1e-19 / 1.7e-12 |  |  |  |  | 9.5e-19 / 3.1e-12 | 4.6e-19 / 1.5e-12 | 5.1e-19 / 1.7e-12 |
| diamond_1000000 | 62 | - | oom | - | - | >600 | >600 | 87.3 | - | >600 | >600 | exact | 1.1e-33 / 5.5e-12 |  |  |  |  |  |  | 1.1e-33 / 5.5e-12 |
| diamond_3000000 | oom | - | oom | - | - | - | - | >600 | - | - | >600 | exact |  |  |  |  |  |  |  |  |
| hardware_10 | 0.011 | 0.01 | n/a | 0.024 | 0.008 | 0.041 | 0.07 | 0.02 | 9.6 | 0.77 | err | exact | 1.2e-18 / 1.2e-14 | 4.4e-16 / 1.2e-14 |  | 1.8e-14 / 2.1e-12 | 1.1e-16 / 1.4e-14 | 1.1e-16 / 3.3e-13 | 1.1e-17 / 3.3e-13 | 1.2e-18 / 1.2e-14 |
| hardware_30 | 0.032 | 0.067 | n/a | 0.035 | 0.035 | 0.217 | 0.309 | 0.059 | 9.68 | 0.948 | >600 | exact | 2.2e-16 / 1.1e-14 | 4.4e-16 / 1.1e-14 |  | 1.7e-14 / 3.3e-12 | 1.1e-16 / 7.1e-14 | 1.1e-16 / 1.1e-12 | 1.1e-16 / 1.1e-12 | 2.2e-16 / 1.1e-14 |
| hardware_100 | 0.091 | 0.684 | n/a | 0.112 | 0.58 | 2.3 | 2.18 | 0.186 | 10.3 | 3.09 | - | exact | 1.1e-16 / 1.1e-14 | 4.4e-16 / 1.1e-14 |  | 1.7e-14 / 3.2e-12 | 1.1e-16 / 1.9e-12 | 1.1e-16 / 3.7e-12 | 2.2e-16 / 3.7e-12 | 2.2e-16 / 1.1e-14 |
| hardware_300 | 0.277 | 6.19 | n/a | 0.314 | 7.26 | 22 | 43 | 0.602 | 17.5 | 23.1 | - | exact | 3.3e-16 / 1.1e-14 | 4.4e-16 / 1.1e-14 |  | 1.7e-14 / 3.2e-12 | 1.1e-16 / 1.9e-15 | 1.1e-16 / 1.1e-13 | 2.2e-16 / 1.1e-13 | 4.4e-16 / 1.1e-14 |
| hardware_1000 | 0.925 | 80.1 | n/a | 0.871 | 103 | oom | 423 | 2 | 115 | oom | - | exact | 2.2e-16 / 1.1e-14 | 4.4e-16 / 1.1e-14 |  | 1.7e-14 / 3.2e-12 | 1.1e-16 / 1.9e-15 |  | 1.1e-16 / 1.1e-13 | 3.3e-16 / 1.1e-14 |
| hardware_3000 | 2.95 | oom | n/a | 2.53 | oom | - | >600 | 6.66 | oom | - | - | exact | 2.2e-16 / 1.1e-14 |  |  | 1.7e-14 / 3.2e-12 |  |  |  | 1.1e-16 / 1.1e-14 |
| hardware_10000 | 11 | - | n/a | 9.43 | - | - | - | 21.9 | - | - | oom | exact | 1.1e-16 / 1.1e-14 |  |  | 1.7e-14 / 3.2e-12 |  |  |  | 1.1e-16 / 1.1e-14 |
| hardware_30000 | 48 | - | n/a | 50.9 | - | - | - | 76.8 | - | - | - | exact | 3.3e-16 / 1.1e-14 |  |  | 1.7e-14 / 3.2e-12 |  |  |  | 3.3e-16 / 1.1e-14 |
| hardware_100000 | 180 | - | n/a | 295 | - | - | - | 248 | - | - | - | exact | 1.1e-16 / 1.4e-13 |  |  | 1.7e-14 / 3.2e-12 |  |  |  | 1.1e-16 / 1.4e-13 |
| hardware_300000 | oom | - | n/a | >600 | - | - | - | err | - | - | - | exact |  |  |  |  |  |  |  |  |
| hardware_1000000 | oom | - | oom | >600 | - | - | - | oom | - | - | - | exact |  |  |  |  |  |  |  |  |
| hardware_bitwidth_5 | 0.008 | 0.007 | 0.011 | 0.282 | 0.022 | 0.007 | 0.018 | 0.008 | 80.6 | 0.533 | 0.187 | exact | 3.3e-16 / 6.8e-16 | 3.3e-16 / 6.8e-16 | 2.2e-16 / 6.8e-16 | 5.0e-16 / 5.0e-12 | 3.3e-16 / 3.6e-15 | 2.2e-16 / 2.2e-13 | 2.2e-16 / 2.2e-13 | 3.3e-16 / 6.8e-16 |
| hardware_bitwidth_10 | 0.009 | 0.008 | 0.007 | 0.62 | 0.01 | 0.007 | 0.018 | 0.006 | 14.3 | 0.537 | 0.171 | exact | 3.3e-16 / 6.8e-16 | 3.3e-16 / 6.8e-16 | 2.2e-16 / 6.8e-16 | 5.0e-16 / 5.0e-12 | 3.3e-16 / 3.6e-15 | 2.2e-16 / 2.2e-13 | 2.2e-16 / 2.2e-13 | 3.3e-16 / 6.8e-16 |
| hardware_bitwidth_20 | 0.01 | 0.009 | 0.007 | >600 | 0.011 | 0.008 | 0.019 | 0.006 | 14.2 | 0.541 | 0.16 | exact | 3.3e-16 / 6.8e-16 | 3.3e-16 / 6.8e-16 | 2.2e-16 / 6.8e-16 |  | 3.3e-16 / 3.6e-15 | 2.2e-16 / 2.2e-13 | 2.2e-16 / 2.2e-13 | 3.3e-16 / 6.8e-16 |
| hardware_bitwidth_50 | 0.01 | 0.01 | 0.009 | - | 0.011 | 0.009 | 0.021 | 0.007 | 14.2 | 0.536 | 0.162 | exact | 3.3e-16 / 6.8e-16 | 3.3e-16 / 6.8e-16 | 2.2e-16 / 6.8e-16 |  | 3.3e-16 / 3.6e-15 | 2.2e-16 / 2.2e-13 | 2.2e-16 / 2.2e-13 | 3.3e-16 / 6.8e-16 |
| hardware_bitwidth_100 | 0.01 | 0.008 | 0.007 | - | 0.011 | 0.02 | 0.023 | 0.006 | 14.3 | 0.547 | 0.162 | exact | 3.3e-16 / 6.8e-16 | 3.3e-16 / 6.8e-16 | 2.2e-16 / 6.8e-16 |  | 3.3e-16 / 3.6e-15 | 2.2e-16 / 2.2e-13 | 2.2e-16 / 2.2e-13 | 3.3e-16 / 6.8e-16 |
| hardware_bitwidth_300 | 0.01 | 0.008 | 0.008 | - | 0.012 | 0.031 | 0.044 | 0.006 | 14.2 | 0.559 | 0.161 | exact | 3.3e-16 / 6.8e-16 | 3.3e-16 / 6.8e-16 | 2.2e-16 / 6.8e-16 |  | 3.3e-16 / 3.6e-15 | 2.2e-16 / 2.2e-13 | 2.2e-16 / 2.2e-13 | 3.3e-16 / 6.8e-16 |
| hardware_bitwidth_1000 | 0.009 | 0.008 | 0.007 | - | 0.014 | 0.07 | 0.082 | 0.006 | 14.2 | 0.599 | 0.168 | exact | 3.3e-16 / 6.8e-16 | 3.3e-16 / 6.8e-16 | 2.2e-16 / 6.8e-16 |  | 3.3e-16 / 3.6e-15 | 2.2e-16 / 2.2e-13 | 2.2e-16 / 2.2e-13 | 3.3e-16 / 6.8e-16 |
| hardware_bitwidth_3000 | 0.01 | 0.008 | 0.008 | - | 0.02 | 0.197 | 0.201 | 0.006 | 14.2 | 0.726 | 0.167 | exact | 3.3e-16 / 6.8e-16 | 3.3e-16 / 6.8e-16 | 2.2e-16 / 6.8e-16 |  | 3.3e-16 / 3.6e-15 | 2.2e-16 / 2.2e-13 | 2.2e-16 / 2.2e-13 | 3.3e-16 / 6.8e-16 |
| hardware_bitwidth_10000 | 0.009 | 0.009 | 0.009 | - | 0.045 | 0.574 | 0.623 | 0.006 | 14.2 | 1.11 | 0.166 | exact | 3.3e-16 / 6.8e-16 | 3.3e-16 / 6.8e-16 | 2.2e-16 / 6.8e-16 |  | 3.3e-16 / 3.6e-15 | 2.2e-16 / 2.2e-13 | 2.2e-16 / 2.2e-13 | 3.3e-16 / 6.8e-16 |
| hardware_bitwidth_30000 | 0.009 | 0.008 | 0.008 | - | 0.128 | 1.74 | 1.78 | 0.007 | 14.3 | 2.27 | 0.172 | exact | 3.3e-16 / 6.8e-16 | 3.3e-16 / 6.8e-16 | 2.2e-16 / 6.8e-16 |  | 3.3e-16 / 3.6e-15 | 2.2e-16 / 2.2e-13 | 2.2e-16 / 2.2e-13 | 3.3e-16 / 6.8e-16 |
| hardware_bitwidth_100000 | 0.01 | 0.008 | 0.007 | - | 0.589 | 5.86 | 5.85 | 0.006 | 14.8 | 6.4 | 0.168 | exact | 3.3e-16 / 6.8e-16 | 3.3e-16 / 6.8e-16 | 2.2e-16 / 6.8e-16 |  | 3.3e-16 / 3.6e-15 | 2.2e-16 / 2.2e-13 | 2.2e-16 / 2.2e-13 | 3.3e-16 / 6.8e-16 |
| hardware_bitwidth_300000 | 0.009 | 0.008 | 0.007 | - | 1.56 | 18.5 | 18.3 | 0.006 | 15.8 | 19 | 0.17 | exact | 3.3e-16 / 6.8e-16 | 3.3e-16 / 6.8e-16 | 2.2e-16 / 6.8e-16 |  | 3.3e-16 / 3.6e-15 | 2.2e-16 / 2.2e-13 | 2.2e-16 / 2.2e-13 | 3.3e-16 / 6.8e-16 |
| hardware_bitwidth_1000000 | 0.008 | 0.007 | 0.007 | - | 4.68 | 68.2 | 69.5 | 0.006 | 19.1 | 68.7 | 0.164 | exact | 3.3e-16 / 6.8e-16 | 3.3e-16 / 6.8e-16 | 2.2e-16 / 6.8e-16 |  | 3.3e-16 / 3.6e-15 | 2.2e-16 / 2.2e-13 | 2.2e-16 / 2.2e-13 | 3.3e-16 / 6.8e-16 |
| hardware_bitwidth_3000000 | 0.009 | 0.008 | 0.007 | - | 13.8 | 263 | 270 | 0.007 | 28.1 | 263 | 0.166 | exact | 3.3e-16 / 6.8e-16 | 3.3e-16 / 6.8e-16 | 2.2e-16 / 6.8e-16 |  | 3.3e-16 / 3.6e-15 | 2.2e-16 / 2.2e-13 | 2.2e-16 / 2.2e-13 | 3.3e-16 / 6.8e-16 |
| hardware_bitwidth_10000000 | 0.008 | 0.009 | 0.008 | - | 44.9 | >600 | >600 | 0.007 | 59.4 | >600 | 0.165 | exact | 3.3e-16 / 6.8e-16 | 3.3e-16 / 6.8e-16 | 2.2e-16 / 6.8e-16 |  | 3.3e-16 / 3.6e-15 |  |  | 3.3e-16 / 6.8e-16 |
| hardware_bitwidth_100000000 | 0.009 | 0.009 | 0.008 | - | oom | - | - | 0.007 | oom | - | 0.164 | exact | 3.3e-16 / 6.8e-16 | 3.3e-16 / 6.8e-16 | 2.2e-16 / 6.8e-16 |  |  |  |  | 3.3e-16 / 6.8e-16 |
| hmm_filtering_joint_100 | 0.014 | oom | n/a | zero | 0.015 | 0.057 | 0.063 | 0.024 | 8.76 | 0.773 | 0.635 | exact | 5.8e-32 / 1.5e-13 |  |  |  | 2.1e-32 / 5.7e-14 | 5.9e-32 / 1.6e-13 | 5.9e-32 / 1.6e-13 | 5.8e-32 / 1.5e-13 |
| hmm_filtering_joint_300 | 0.046 | - | n/a | zero | 0.13 | 0.348 | 0.193 | 0.065 | 8.97 | 1.06 | 2.05 | exact | 3.8e-69 / 2.8e-14 |  |  |  | 1.7e-67 / 1.3e-12 | 5.5e-69 / 4.0e-14 | 5.7e-69 / 4.1e-14 | 3.8e-69 / 2.8e-14 |
| hmm_filtering_joint_1000 | 0.106 | - | n/a | underflow | 1.66 | underflow | 0.886 | 0.201 | 147 | underflow | 13 | exact | 5.5e-194 / 1.4e-11 |  |  |  | 6.1e-194 / 1.5e-11 |  | 5.6e-194 / 1.4e-11 | 5.5e-194 / 1.4e-11 |
| hmm_filtering_joint_3000 | 0.351 | - | n/a | 6.2 | 18.9 | 40 | 4.94 | 0.648 | 166 | 41.5 | oom | exact | 0.0e+00 / 0.0e+00 |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| hmm_filtering_joint_10000 | 1.21 | - | n/a | 76.5 | oom | oom | 35.9 | 2.26 | oom | oom | - | exact | 0.0e+00 / 0.0e+00 |  |  | 0.0e+00 / 0.0e+00 |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| hmm_filtering_joint_30000 | 3.64 | - | n/a | >600 | - | - | >600 | 7.13 | - | - | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |
| hmm_filtering_joint_100000 | 13.2 | - | n/a | - | - | - | - | 25.8 | - | - | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |
| hmm_filtering_joint_300000 | 59.3 | - | oom | - | stack | - | - | 83.6 | stack | - | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |
| hmm_filtering_joint_1000000 | oom | - | - | - | stack | - | - | oom | stack | - | - | exact |  |  |  |  |  |  |  |  |
| hmm_filtering_joint_3000000 | - | - | - | - | stack | - | - | - | stack | - | - | exact |  |  |  |  |  |  |  |  |
| hmm_filtering_marginal_100 | 0.027 | oom | n/a | zero | 0.014 | 0.056 | 0.058 | 0.04 | 8.61 | 0.843 | 0.531 | exact | 6.2e-15 / 1.0e-14 |  |  |  | 0.0e+00 / 0.0e+00 | 6.0e-15 / 9.6e-15 | 5.8e-15 / 9.3e-15 | 6.2e-15 / 1.0e-14 |
| hmm_filtering_marginal_300 | 0.042 | - | n/a | zero | 0.12 | 0.303 | 0.172 | 0.072 | 8.63 | 1.02 | 1.88 | exact | 6.0e-15 / 9.6e-15 |  |  |  | 0.0e+00 / 0.0e+00 | 6.2e-15 / 1.0e-14 | 5.8e-15 / 9.3e-15 | 6.0e-15 / 9.6e-15 |
| hmm_filtering_marginal_1000 | 0.11 | - | n/a | underflow | 1.51 | underflow | 0.631 | 0.18 | 10.4 | underflow | 12.3 | exact | 3.0e-14 / 4.9e-14 |  |  |  | 7.1e-14 / 1.1e-13 |  | 3.0e-14 / 4.8e-14 | 3.0e-14 / 4.9e-14 |
| hmm_filtering_marginal_3000 | 2.48 | - | n/a | underflow | 18.2 | underflow | 2.21 | 0.608 | 27.8 | underflow | stack | exact | 1.1e-13 / 1.8e-13 |  |  |  | 2.8e-13 / 4.5e-13 |  | 1.1e-13 / 1.8e-13 | 1.1e-13 / 1.8e-13 |
| hmm_filtering_marginal_10000 | 3.02 | - | n/a | underflow | 238 | oom | 8.84 | 2.07 | 252 | oom | stack | exact | 4.0e-13 / 6.4e-13 |  |  |  | 5.7e-13 / 9.1e-13 |  | 3.9e-13 / 6.3e-13 | 4.0e-13 / 6.4e-13 |
| hmm_filtering_marginal_30000 | 3.66 | - | n/a | underflow | oom | - | 42 | 6.37 | oom | - | stack | exact | 9.7e-13 / 1.6e-12 |  |  |  |  |  | 9.6e-13 / 1.5e-12 | 9.7e-13 / 1.6e-12 |
| hmm_filtering_marginal_100000 | 11.8 | - | n/a | >600 | - | - | oom | 22.6 | - | - | stack | exact | 3.3e-12 / 5.2e-12 |  |  |  |  |  |  | 3.3e-12 / 5.2e-12 |
| hmm_filtering_marginal_300000 | 50.8 | - | oom | - | - | - | - | 71.1 | - | - | stack | exact | 1.5e-11 / 2.4e-11 |  |  |  |  |  |  | 1.5e-11 / 2.4e-11 |
| hmm_filtering_marginal_1000000 | oom | - | - | - | - | - | - | oom | - | - | >600 | exact |  |  |  |  |  |  |  |  |
| hmm_marginal_likelihood_100 | 0.016 | oom | n/a | 0.033 | 0.013 | 0.049 | 0.068 | 0.027 | 8.5 | 0.767 | oom | exact | 1.7e-48 / 6.1e-14 |  |  | 3.1e-47 / 1.1e-12 | 0.0e+00 / 0.0e+00 | 1.7e-48 / 6.0e-14 | 1.7e-48 / 6.0e-14 | 1.7e-48 / 6.1e-14 |
| hmm_marginal_likelihood_300 | 0.038 | - | n/a | 0.074 | 0.124 | 0.257 | 0.22 | 0.071 | 8.69 | 0.996 | - | exact | 2.7e-116 / 1.2e-12 |  |  | 2.6e-116 / 1.2e-12 | 0.0e+00 / 0.0e+00 | 2.7e-116 / 1.2e-12 | 2.6e-116 / 1.2e-12 | 2.7e-116 / 1.2e-12 |
| hmm_marginal_likelihood_600 | 0.098 | - | n/a | 0.231 | 0.519 | 0.995 | 0.64 | 0.136 | 146 | 1.77 | - | exact | 3.2e-219 / 7.0e-12 |  |  | 3.6e-219 / 7.8e-12 | 2.6e-221 / 5.7e-14 | 3.2e-219 / 7.0e-12 | 3.2e-219 / 7.0e-12 | 3.2e-219 / 7.0e-12 |
| hmm_marginal_likelihood_900 | 0.13 | - | n/a | 0.44 | 1.22 | 2.27 | 1.02 | 0.199 | 146 | 3.07 | - | exact | 1.2e-322 / 1.3e-11 |  |  | 1.2e-322 / 1.3e-11 | 0.0e+00 / 0.0e+00 | 1.2e-322 / 1.3e-11 | 1.2e-322 / 1.3e-11 | 1.2e-322 / 1.3e-11 |
| hmm_smoothing_joint_100 | 0.014 | oom | n/a | zero | 0.015 | 0.055 | 0.064 | 0.023 | 8.74 | 0.769 | 0.602 | exact | 1.3e-31 / 2.0e-13 |  |  |  | 3.6e-32 / 5.7e-14 | 1.3e-31 / 2.0e-13 | 1.3e-31 / 2.0e-13 | 1.3e-31 / 2.0e-13 |
| hmm_smoothing_joint_300 | 0.036 | - | n/a | zero | 0.128 | 0.326 | 0.198 | 0.063 | 8.99 | 1.03 | 2.4 | exact | 3.1e-67 / 1.3e-12 |  |  |  | 3.1e-67 / 1.4e-12 | 3.1e-67 / 1.4e-12 | 3.1e-67 / 1.4e-12 | 3.1e-67 / 1.3e-12 |
| hmm_smoothing_joint_1000 | 0.123 | - | n/a | underflow | 1.56 | underflow | 0.884 | 0.199 | 10.8 | underflow | 16.9 | exact | 2.0e-193 / 2.9e-11 |  |  |  | 1.0e-193 / 1.5e-11 |  | 2.0e-193 / 2.9e-11 | 2.0e-193 / 2.9e-11 |
| hmm_smoothing_joint_3000 | 0.313 | - | n/a | 6.93 | 17.5 | 36.4 | 5.06 | 0.63 | 164 | 37.9 | oom | exact | 0.0e+00 / 0.0e+00 |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| hmm_smoothing_joint_10000 | 1.12 | - | n/a | 81 | 241 | oom | 39.9 | 2.23 | 393 | oom | - | exact | 0.0e+00 / 0.0e+00 |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| hmm_smoothing_joint_30000 | 3.54 | - | n/a | >600 | stack | - | >600 | 6.8 | stack | - | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |
| hmm_smoothing_joint_100000 | 12.8 | - | n/a | - | stack | - | - | 24.7 | stack | - | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |
| hmm_smoothing_joint_300000 | 47 | - | oom | - | stack | - | - | 81.7 | stack | - | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |
| hmm_smoothing_joint_1000000 | oom | - | oom | - | stack | - | - | oom | stack | - | - | exact |  |  |  |  |  |  |  |  |
| hmm_smoothing_joint_3000000 | oom | - | oom | - | stack | - | - | - | stack | - | - | exact |  |  |  |  |  |  |  |  |
| hmm_smoothing_marginal_100 | 0.027 | oom | n/a | zero | 0.014 | 0.054 | 0.056 | 0.04 | 8.48 | 0.794 | 0.513 | exact | 5.6e-17 / 1.2e-16 |  |  |  | 0.0e+00 / 0.0e+00 | 5.6e-17 / 1.2e-16 | 1.7e-16 / 3.7e-16 | 5.6e-17 / 1.2e-16 |
| hmm_smoothing_marginal_300 | 0.066 | - | n/a | zero | 0.118 | 0.298 | 0.17 | 0.074 | 8.63 | 1.02 | 1.78 | exact | 6.3e-15 / 1.4e-14 |  |  |  | 0.0e+00 / 0.0e+00 | 6.2e-15 / 1.4e-14 | 6.3e-15 / 1.4e-14 | 6.3e-15 / 1.4e-14 |
| hmm_smoothing_marginal_1000 | 0.11 | - | n/a | underflow | 1.48 | underflow | 0.623 | 0.181 | 10.3 | underflow | 10.9 | exact | 3.2e-14 / 7.1e-14 |  |  |  | 0.0e+00 / 0.0e+00 |  | 3.2e-14 / 7.0e-14 | 3.2e-14 / 7.1e-14 |
| hmm_smoothing_marginal_3000 | 0.315 | - | n/a | underflow | 17 | underflow | 2.22 | 0.604 | 26.5 | underflow | err | exact | 3.2e-14 / 7.1e-14 |  |  |  | 0.0e+00 / 0.0e+00 |  | 3.2e-14 / 7.1e-14 | 3.2e-14 / 7.1e-14 |
| hmm_smoothing_marginal_10000 | 3.52 | - | n/a | underflow | 232 | oom | 8.78 | 2.11 | 246 | oom | - | exact | 1.7e-13 / 3.8e-13 |  |  |  | 0.0e+00 / 0.0e+00 |  | 1.7e-13 / 3.8e-13 | 1.7e-13 / 3.8e-13 |
| hmm_smoothing_marginal_30000 | 6.18 | - | n/a | underflow | oom | - | 42.6 | 6.53 | oom | - | - | exact | 6.5e-13 / 1.4e-12 |  |  |  |  |  | 6.5e-13 / 1.4e-12 | 6.5e-13 / 1.4e-12 |
| hmm_smoothing_marginal_100000 | 12 | - | n/a | >600 | - | - | oom | 22.8 | - | - | - | exact | 2.6e-12 / 5.8e-12 |  |  |  |  |  |  | 2.6e-12 / 5.8e-12 |
| hmm_smoothing_marginal_300000 | 54.8 | - | n/a | - | - | - | - | 72.5 | - | - | - | exact | 1.6e-11 / 3.5e-11 |  |  |  |  |  |  | 1.6e-11 / 3.5e-11 |
| hmm_smoothing_marginal_1000000 | oom | - | oom | - | - | - | - | oom | - | - | - | exact |  |  |  |  |  |  |  |  |
| ladder_1000 | 0.073 | oom | n/a | 3.53 | 0.085 | 0.393 | 0.816 | 0.153 | 10.1 | 1.11 | >600 | exact | 4.4e-16 / 1.5e-15 |  |  | 4.6e-13 / 1.5e-12 | 1.2e-15 / 3.8e-15 | 7.8e-16 / 2.6e-15 | 6.1e-16 / 2.0e-15 | 4.4e-16 / 1.5e-15 |
| ladder_3000 | 0.235 | - | n/a | 37 | 0.292 | 1.67 | 3.08 | 0.477 | 13.3 | 2.39 | - | exact | 1.4e-16 / 1.2e-15 |  |  | 3.0e-13 / 2.7e-12 | 2.4e-15 / 2.2e-14 | 6.2e-16 / 5.6e-15 | 1.1e-16 / 1.0e-15 | 1.4e-16 / 1.2e-15 |
| ladder_10000 | 0.795 | - | n/a | 435 | 1.18 | 5.42 | 10.6 | 1.73 | 25.2 | 6.17 | - | exact | 1.3e-18 / 3.9e-16 |  |  | 4.3e-15 / 1.3e-12 | 1.8e-15 / 5.4e-13 | 3.9e-17 / 1.2e-14 | 1.9e-17 / 5.5e-15 | 1.3e-18 / 3.9e-16 |
| ladder_30000 | 2.54 | - | n/a | >600 | 5.5 | 19.8 | 32.7 | 5.03 | 80.8 | 20.6 | - | exact | 1.4e-21 / 9.0e-15 |  |  |  | 5.6e-19 / 3.7e-12 | 3.5e-21 / 2.3e-14 | 1.4e-21 / 9.0e-15 | 1.4e-21 / 9.0e-15 |
| ladder_100000 | 8.91 | - | n/a | - | 45.9 | 81.4 | 128 | 17.8 | 476 | 82.2 | - | exact | 8.4e-36 / 8.8e-14 |  |  |  | 1.8e-33 / 1.8e-11 | 8.1e-36 / 8.6e-14 | 8.4e-36 / 8.8e-14 | 8.4e-36 / 8.8e-14 |
| ladder_300000 | 28.4 | - | oom | - | >600 | 233 | 370 | 55 | >600 | 234 | - | exact | 3.3e-79 / 9.6e-14 |  |  |  |  | 1.6e-79 / 4.7e-14 | 3.3e-79 / 9.6e-14 | 3.3e-79 / 9.6e-14 |
| ladder_1000000 | 119 | - | - | - | - | >600 | >600 | 144 | - | >600 | - | exact | 2.8e-231 / 9.0e-14 |  |  |  |  |  |  | 2.8e-231 / 9.0e-14 |
| ladder_3000000 | oom | - | oom | - | - | - | - | err | - | - | - | exact |  |  |  |  |  |  |  |  |
| motivating_1000 | 0.037 | oom | n/a | 1.59 | 0.032 | 0.136 | 0.24 | 0.069 | 8.76 | 0.922 | 5.51 | exact | 5.6e-16 / 2.0e-15 |  |  | 2.7e-13 / 1.0e-12 | 5.6e-17 / 2.0e-16 | 0.0e+00 / 0.0e+00 | 5.6e-17 / 2.0e-16 | 5.6e-16 / 2.0e-15 |
| motivating_3000 | 0.103 | - | n/a | 16.4 | 0.112 | 0.486 | 0.782 | 0.213 | 9.97 | 1.32 | stack | exact | 5.6e-16 / 2.0e-15 |  |  | 2.7e-13 / 1.0e-12 | 5.6e-17 / 2.0e-16 | 0.0e+00 / 0.0e+00 | 5.6e-17 / 2.0e-16 | 5.6e-16 / 2.0e-15 |
| motivating_10000 | 0.401 | - | n/a | 203 | 0.435 | 1.74 | 2.62 | 0.727 | 15.1 | 2.49 | stack | exact | 5.6e-16 / 2.0e-15 |  |  | 2.7e-13 / 1.0e-12 | 5.6e-17 / 2.0e-16 | 0.0e+00 / 0.0e+00 | 5.6e-17 / 2.0e-16 | 5.6e-16 / 2.0e-15 |
| motivating_30000 | 1.18 | - | n/a | >600 | 1.47 | 5.49 | 8.89 | 2.18 | 35.5 | 6.24 | stack | exact | 5.6e-16 / 2.0e-15 |  |  |  | 5.6e-17 / 2.0e-16 | 0.0e+00 / 0.0e+00 | 5.6e-17 / 2.0e-16 | 5.6e-16 / 2.0e-15 |
| motivating_100000 | 4.22 | - | n/a | - | 55.4 | 22.4 | 33.3 | 7.88 | 453 | 23.2 | stack | exact | 5.6e-16 / 2.0e-15 |  |  |  | 5.6e-17 / 2.0e-16 | 0.0e+00 / 0.0e+00 | 5.6e-17 / 2.0e-16 | 5.6e-16 / 2.0e-15 |
| motivating_300000 | 18.3 | - | n/a | - | >600 | 73.1 | 103 | 25.3 | >600 | 74.2 | stack | exact | 5.6e-16 / 2.0e-15 |  |  |  |  | 0.0e+00 / 0.0e+00 | 5.6e-17 / 2.0e-16 | 5.6e-16 / 2.0e-15 |
| motivating_1000000 | 60.5 | - | n/a | - | - | 287 | 409 | 85.8 | - | 288 | >600 | exact | 5.6e-16 / 2.0e-15 |  |  |  |  | 0.0e+00 / 0.0e+00 | 5.6e-17 / 2.0e-16 | 5.6e-16 / 2.0e-15 |
| motivating_3000000 | >600 | - | oom | - | - | >600 | >600 | oom | - | >600 | >600 | exact |  |  |  |  |  |  |  |  |
| network_10 | 0.008 | oom | n/a | 0.038 | 0.013 | 0.059 | 0.072 | 0.015 | 12.2 | 0.766 | err | exact | 1.2e-15 / 3.0e-14 |  |  | 2.3e-13 / 3.8e-12 | 3.3e-16 / 7.1e-15 | 4.4e-16 / 1.4e-15 | 3.3e-16 / 3.0e-14 | 1.2e-15 / 3.0e-14 |
| network_30 | 0.019 | - | n/a | 0.074 | 0.039 | 0.279 | 0.19 | 0.033 | 12.2 | 0.989 | - | exact | 3.2e-15 / 5.9e-15 |  |  | 3.0e-13 / 3.4e-12 | 1.9e-16 / 2.3e-14 | 2.2e-16 / 1.6e-15 | 2.2e-16 / 3.6e-15 | 3.2e-15 / 5.9e-15 |
| network_100 | 0.059 | - | n/a | 0.255 | 0.462 | 2.47 | 0.729 | 0.106 | 12.7 | 3.24 | - | exact | 1.7e-15 / 4.4e-15 |  |  | 2.7e-13 / 6.9e-13 | 1.2e-15 / 5.7e-14 | 7.2e-16 / 2.9e-15 | 1.1e-15 / 3.1e-15 | 1.7e-15 / 4.4e-15 |
| network_300 | 0.159 | - | n/a | 0.84 | 5.01 | 22.8 | 2.17 | 0.307 | 17.2 | 24 | - | exact | 6.4e-15 / 1.2e-14 |  |  | 2.5e-13 / 1.4e-12 | 6.3e-16 / 7.7e-14 | 2.4e-15 / 1.1e-14 | 6.9e-16 / 1.1e-14 | 6.4e-15 / 1.2e-14 |
| network_1000 | 0.559 | - | n/a | 3.48 | 67.4 | oom | 8.79 | 1.04 | 80.3 | oom | - | exact | 3.0e-15 / 1.6e-14 |  |  | 1.7e-13 / 1.0e-12 | 7.8e-16 / 4.0e-13 |  | 2.2e-16 / 4.2e-14 | 3.0e-15 / 1.6e-14 |
| network_3000 | 1.71 | - | n/a | 11.6 | oom | - | 26.8 | 3.25 | oom | - | - | exact | 1.0e-14 / 1.0e-14 |  |  | 4.4e-13 / 2.1e-12 |  |  | 5.6e-16 / 9.8e-14 | 1.0e-14 / 1.0e-14 |
| network_10000 | 5.89 | - | n/a | 56.3 | - | - | 102 | 11.6 | - | - | - | exact | 2.2e-16 / 3.0e-14 |  |  | 1.1e-14 / 3.9e-12 |  |  | 1.1e-14 / 6.3e-13 | 2.2e-16 / 3.0e-14 |
| network_30000 | 18.9 | - | n/a | 207 | - | - | 419 | 36.4 | - | - | - | exact | 4.6e-15 / 1.1e-13 |  |  | 1.1e-14 / 3.9e-13 |  |  | 1.1e-14 / 2.0e-12 | 4.6e-15 / 1.1e-13 |
| network_100000 | 129 | - | n/a | >600 | - | - | >600 | 195 | - | - | - | exact | 4.6e-15 / 5.9e-13 |  |  |  |  |  |  | 4.6e-15 / 5.9e-13 |
| network_300000 | oom | - | n/a | - | - | - | - | oom | - | - | - | exact |  |  |  |  |  |  |  |  |
| network_1000000 | - | - | oom | - | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |

Answers. The largest differences from the reference: `auto` 6.4e-11
relative (3.3e-16 absolute, caesar_2000); `program` 2.7e-11 relative
(1.5e-11 absolute, bn_insurance); `decision` 6.8e-16 relative (2.2e-16
absolute, hardware_bitwidth_10); Dice 2.6e-11 relative (1.5e-11
absolute, bn_insurance); Alea 1.3e-10 relative (2.1e-171 absolute,
caesar_2000); Roulette 1.0e+00 relative (1.1e-16 absolute, caesar_100);
Roulette+tddlog 3.5e-06 relative (9.6e-09 absolute, bn_munin).

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
factories. 64 instances in 4 families. Each is a Dice program, which
tddlog's Dice reader translates for tddlog and ProbLog (`tddlog
--program`) and `dice2alea.py` for Alea.jl and `dice2roulette.py` for
Roulette; Geni runs the same programs as its own `.gir` files
(`gen_geni.py --gir`, the artifact's flags), and Storm and PRISM the
weather factory as its PRISM model. The reference is a dynamic program
over each model's states (`gen_geni.py --reference-of`, `--exact`), in a
pass of its own.

### Results: tddlog 0.1.0, ProbLog, Dice, Alea.jl and Roulette

One job on a lab machine (36 physical cores, 503 GB) on 2026-10-29, 16
lanes of one core each, each process limited to 31 GB. tddlog 0.1.0
`auto` as of 2026-10-28 (tididi ca759585, goatd e592d1e), ProbLog 2.3.0
(d-DNNF), Dice 1.0 with an unlimited stack and Alea.jl as on the
bit-blasted programs, and Roulette (the artifact's, 54d6185, on Racket
8.16, variant `rsdd`, the translated program compiled by `raco make`
before it runs, untimed, as the artifact's programs are; its time is the
one it reports for building and querying the program; the process's is
in a column of its own), and Geni (gennifer; Li and Zhang, ICFP 2025),
noDice (Gürtler and Kaminski, OOPSLA 2026), Psi (Gehr et al., CAV 2016;
revision ef8a9e5), Genfer (Zaiser, Murawski and Ong, NeurIPS 2023), SPPL
2.0.4 (Saad, Rinard and Mansinghka, PLDI 2021), Pluck (Bowers et al.,
PLDI 2025), PITA (cplint, SWI-Prolog 9.2.9), aspmc 1.1.1 with
sharpSAT-TD, Scallop 0.2.5 (668bfb6; Li, Huang and Naik, PLDI 2023),
Storm 1.14 (stormpy), one Sylvan thread (`sparse`, `exact`, `dd`), PRISM
4.8.1 (`sparse`, `hybrid`), Ace 3.0 (`auto`, `forceC2d`), Merlin 1.7.0's
bucket-tree elimination (`-a bte`), pyAgrum's junction tree, pgmpy's
variable elimination, libDAI's junction tree and TensorInference.jl's
tensor network contraction (`greedy`, `treesa`) (the process's seconds)
run on an instance back to back in one lane, in an order rotated by
instance and repetition; three repetitions, 600 s each. A ladder runs
smallest first in one lane, and a system that runs out of time or memory
on an instance skips the ladder's larger ones; the expensive tops of the
ladders (the weather factory from 15 factories and from 100 days, the
grids above 20) ran as ladders of their own, so a system that failed
below them ran them to its own failure. The runs beside Roulette on its
upstream main and Roulette+tddlog's ran a build of 2026-11-01 (tididi
c5642b97, goatd e592d1e). The reruns of tddlog where an older build's
record stood ran a build of 2026-10-31 (tididi c5642b97, goatd e592d1e).
The growth sizes ran a build of 2026-10-25 with twin contraction (tididi
b9e5064e, goatd e592d1e) and a recursion over a successor relation
grounded level by level, which lacks only the release's sparse route
(tididi 9fc0a599). The release's main contains every solver change those
builds have.

Of the 64 instances, two lie past the sizes at which every system had
failed, and none every system answers in under a second. Answers: tddlog
50, ProbLog 10, Dice 43, Alea 27, Roulette 22, Geni 38, noDice 12, Psi
0, Genfer 11, SPPL 5, Pluck 12, PITA 11, aspmc 29, Scallop 1, Storm
sparse 43, Storm exact 27, Storm dd 35, PRISM sparse 47, PRISM hybrid
22, Ace 31, Ace c2d 28, Merlin 16, pyAgrum 30, pgmpy 20, libDAI 31,
TensorInference.jl greedy 34, TensorInference.jl treesa 32.

| program | sizes | tddlog | ProbLog | Dice | Alea | Roulette | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Scallop | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Merlin | pyAgrum | pgmpy | libDAI | TensorInference.jl greedy | TensorInference.jl treesa |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| grid, link failures | N = 4 to 32 | 18 (148 s) | 6 (8.67 s) | 16 (430 s) | 14 (401 s) | 12 (48.7 s) | 14 (122 s) | 14 (468 s) | timeout at 4 | 4 (5.61 s) | 4 (12.2 s) | 8 (13 s) | 6 (0.61 s) | 14 (364 s) | memout at 4 | 14 (253 s) | 14 (324 s) | 14 (149 s) | 16 (439 s) | 16 (530 s) | 12 (19.1 s) | 12 (271 s) | 12 (8.04 s) | 12 (2.04 s) | 14 (275 s) | 12 (7.27 s) | 12 (36.6 s) | 18 (448 s) |
| grid, router failures | N = 4 to 72 | 24 (89.7 s) | 6 (16.5 s) | 16 (89.7 s) | 14 (138 s) | 12 (17 s) | 16 (142 s) | 14 (180 s) | timeout at 4 | 18 (309 s) | 6 (44.6 s) | 10 (17.6 s) | 6 (1.4 s) | 14 (536 s) | memout at 4 | 16 (333 s) | 16 (427 s) | 40 (266 s) | 48 (223 s) | 48 (150 s) | 16 (88.6 s) | 14 (223 s) | 16 (24.9 s) | 16 (6.52 s) | 20 (5.97 s) | 18 (57.6 s) | 20 (11.2 s) | 16 (416 s) |
| weather factory, by factories | 2 to 19 factories | 18 (84.1 s) | 2 (5.69 s) | 16 (432 s) | 11 (266 s) | 9 (77.4 s) | 11 (4.77 s) | error at 2 | error at 2 | 2 (0.14 s) | timeout at 2 | 2 (6.15 s) | 2 (0.56 s) | 13 (176 s) | memout at 2 | 13 (176 s) | 10 (191 s) | 11 (207 s) | 10 (12.7 s) | 7 (20.6 s) | 13 (15.5 s) | 13 (345 s) | 9 (0.85 s) | 13 (7.72 s) | 8 (593 s) | 13 (16.5 s) | 17 (168 s) | 13 (443 s) |
| weather factory, by days | 1 to 30000 days | 3 000 (116 s) | 5 (67.4 s) | 1 000 (523 s) | 9 (72 s) | 8 (102 s) | 1 000 (271 s) | error at 1 | error at 1 | 1 (0.006 s) | 2 (134 s) | 4 (29.9 s) | 6 (165 s) | 9 (16.8 s) | 1 (0.019 s) | 30 000 (189 s) | 9 (147 s) | 100 (413 s) | 30 000 (183 s) | timeout at 1 | 200 (69.6 s) | 9 (30.7 s) | zero at 1 | 100 (169 s) | 2 (1.82 s) | 100 (344 s) | 9 (0.63 s) | 9 (286 s) |

The table above gives, a program a row, the largest size each system
answers and its time there, or its failure at the smallest size it ran.

- Against ProbLog, over the 21 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 323 s in all and ProbLog 6725 s; over the 10 both answer, 0.29 s
  against 125 s. tddlog is faster on 7 of the 10 both answer and ProbLog
  on 0, both under 0.2 s or level on the other 3. tddlog alone answers
  11, ProbLog alone 0.
- Against Dice, over the 49 instances given to both that either answers,
  a run that does not answer counted at the 600 s limit, tddlog takes
  839 s in all and Dice 7023 s; over the 43 both answer, 312 s against
  3423 s. tddlog is faster on 32 of the 43 both answer and Dice on 0,
  both under 0.2 s or level on the other 11. tddlog alone answers 6,
  Dice alone 0.
- Against Alea, over the 37 instances given to both that either answers,
  a run that does not answer counted at the 600 s limit, tddlog takes
  384 s in all and Alea 7191 s; over the 27 both answer, 7.39 s against
  1191 s. tddlog is faster on 17 of the 27 both answer and Alea on 0,
  both under 0.2 s or level on the other 10. tddlog alone answers 10,
  Alea alone 0. Against the Julia process, over the 37 instances given
  to both that either answers, a run that does not answer counted at the
  600 s limit, tddlog takes 384 s in all and the process 7579 s; over
  the 27 both answer, 7.39 s against 1579 s.
- Against Roulette, over the 26 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 7.09 s in all and Roulette 2805 s; over the 22 both answer, 3.05
  s against 405 s. tddlog is faster on 14 of the 22 both answer and
  Roulette on 0, both under 0.2 s or level on the other 8. tddlog alone
  answers 4, Roulette alone 0.
- Against Geni, over the 48 instances given to both that either answers,
  a run that does not answer counted at the 600 s limit, tddlog takes
  691 s in all and Geni 7803 s; over the 38 both answer, 216 s against
  1803 s. tddlog is faster on 29 of the 38 both answer and Geni on 0,
  both under 0.2 s or level on the other 9. tddlog alone answers 10,
  Geni alone 0.
- Against noDice, over the 38 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 388 s in all and noDice 16352 s; over the 12 both answer, 5.55 s
  against 752 s. tddlog is faster on 8 of the 12 both answer and noDice
  on 0, both under 0.2 s or level on the other 4. tddlog alone answers
  26, noDice alone 0.
- Against Psi, over the 35 instances given to both that either answers,
  a run that does not answer counted at the 600 s limit, tddlog takes
  627 s in all and Psi 21000 s. tddlog alone answers 35, Psi alone 0.
- Against Genfer, over the 26 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 465 s in all and Genfer 9390 s; over the 11 both answer, 6.03 s
  against 390 s. tddlog is faster on 6 of the 11 both answer and Genfer
  on 0, both under 0.2 s or level on the other 5. tddlog alone answers
  15, Genfer alone 0.
- Against SPPL, over the 16 instances given to both that either answers,
  a run that does not answer counted at the 600 s limit, tddlog takes
  322 s in all and SPPL 6793 s; over the 5 both answer, 0.047 s against
  193 s. tddlog is faster on 5 of the 5 both answer and SPPL on 0.
  tddlog alone answers 11, SPPL alone 0.
- Against Pluck, over the 17 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 23.6 s in all and Pluck 3105 s; over the 12 both answer, 0.29 s
  against 105 s. tddlog is faster on 12 of the 12 both answer and Pluck
  on 0. tddlog alone answers 5, Pluck alone 0.
- Against PITA, over the 22 instances given to both that either answers,
  a run that does not answer counted at the 600 s limit, tddlog takes
  323 s in all and PITA 6794 s; over the 11 both answer, 0.43 s against
  194 s. tddlog is faster on 11 of the 11 both answer and PITA on 0.
  tddlog alone answers 11, PITA alone 0.
- Against aspmc, over the 38 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 388 s in all and aspmc 6916 s; over the 29 both answer, 11.6 s
  against 1516 s. tddlog is faster on 29 of the 29 both answer and aspmc
  on 0. tddlog alone answers 9, aspmc alone 0.
- Against Scallop, over the 12 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 322 s in all and Scallop 6600 s; over the 1 both answer, 0.004 s
  against 0.019 s. tddlog is faster on 0 of the 1 both answer and
  Scallop on 0, both under 0.2 s or level on the other 1. tddlog alone
  answers 11, Scallop alone 0.
- Against Storm sparse, over the 49 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 1291 s in all and Storm sparse 5286 s; over the 41 both answer,
  337 s against 1024 s. tddlog is faster on 27 of the 41 both answer and
  Storm sparse on 10, both under 0.2 s or level on the other 4. tddlog
  alone answers 7, Storm sparse alone 1.
- Against Storm exact, over the 37 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 388 s in all and Storm exact 7725 s; over the 27 both answer,
  8.23 s against 1725 s. tddlog is faster on 24 of the 27 both answer
  and Storm exact on 0, both under 0.2 s or level on the other 3. tddlog
  alone answers 10, Storm exact alone 0.
- Against Storm dd, over the 41 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 1001 s in all and Storm dd 6139 s; over the 32 both answer, 111
  s against 1275 s. tddlog is faster on 31 of the 32 both answer and
  Storm dd on 1. tddlog alone answers 8, Storm dd alone 1.
- Against PRISM sparse, over the 51 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 2041 s in all and PRISM sparse 6125 s; over the 41 both answer,
  480 s against 1241 s. tddlog is faster on 33 of the 41 both answer and
  PRISM sparse on 8. tddlog alone answers 8, PRISM sparse alone 2.
- Against PRISM hybrid, over the 29 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 1140 s in all and PRISM hybrid 6915 s; over the 18 both answer,
  154 s against 901 s. tddlog is faster on 17 of the 18 both answer and
  PRISM hybrid on 1. tddlog alone answers 10, PRISM hybrid alone 1.
- Against Ace, over the 36 instances given to both that either answers,
  a run that does not answer counted at the 600 s limit, tddlog takes
  139 s in all and Ace 3305 s; over the 31 both answer, 19.3 s against
  305 s. tddlog is faster on 31 of the 31 both answer and Ace on 0.
  tddlog alone answers 5, Ace alone 0.
- Against Ace c2d, over the 33 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 116 s in all and Ace c2d 4391 s; over the 28 both answer, 8.33 s
  against 1391 s. tddlog is faster on 28 of the 28 both answer and Ace
  c2d on 0. tddlog alone answers 5, Ace c2d alone 0.
- Against Merlin, over the 34 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 121 s in all and Merlin 10837 s; over the 16 both answer, 3.85 s
  against 37.2 s. tddlog is faster on 9 of the 16 both answer and Merlin
  on 0, both under 0.2 s or level on the other 7. tddlog alone answers
  18, Merlin alone 0.
- Against pyAgrum, over the 35 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 128 s in all and pyAgrum 3203 s; over the 30 both answer, 12.2 s
  against 203 s. tddlog is faster on 28 of the 30 both answer and
  pyAgrum on 0, both under 0.2 s or level on the other 2. tddlog alone
  answers 5, pyAgrum alone 0.
- Against pgmpy, over the 25 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 244 s in all and pgmpy 4081 s; over the 20 both answer, 16.9 s
  against 1081 s. tddlog is faster on 19 of the 20 both answer and pgmpy
  on 1. tddlog alone answers 5, pgmpy alone 0.
- Against libDAI, over the 36 instances given to both that either
  answers, a run that does not answer counted at the 600 s limit, tddlog
  takes 133 s in all and libDAI 3467 s; over the 31 both answer, 16.5 s
  against 467 s. tddlog is faster on 24 of the 31 both answer and libDAI
  on 0, both under 0.2 s or level on the other 7. tddlog alone answers
  5, libDAI alone 0.
- Against TensorInference.jl greedy, over the 40 instances given to both
  that either answers, a run that does not answer counted at the 600 s
  limit, tddlog takes 523 s in all and TensorInference.jl greedy 3856 s;
  over the 34 both answer, 142 s against 256 s. tddlog is faster on 22
  of the 34 both answer and TensorInference.jl greedy on 5, both under
  0.2 s or level on the other 7. tddlog alone answers 6,
  TensorInference.jl greedy alone 0.
- Against TensorInference.jl treesa, over the 38 instances given to both
  that either answers, a run that does not answer counted at the 600 s
  limit, tddlog takes 349 s in all and TensorInference.jl treesa 9641 s;
  over the 32 both answer, 213 s against 6041 s. tddlog is faster on 32
  of the 32 both answer and TensorInference.jl treesa on 0. tddlog alone
  answers 6, TensorInference.jl treesa alone 0.
- Roulette answers 22 of the 26 instances it ran; it runs out of memory
  on 4.
- Geni prints 0 for every answer (zero) on 4 instances: weather_n12,
  weather_n13, weather_n15, weather_n16.
- Merlin prints 0 for every answer (zero) on 13 instances: weather_h1,
  weather_h2, weather_h3, weather_h4, weather_h5, weather_h6,
  weather_h7, weather_h8, weather_h9, weather_n10, weather_n11,
  weather_n12, weather_n13.

| instance | tddlog | ProbLog | Dice | Alea | Roulette | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Scallop | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Merlin | pyAgrum | pgmpy | libDAI | TI greedy | TI treesa | Alea process | Roulette process | ref | tddlog diff | ProbLog diff | Dice diff | Alea diff | Roulette diff | Geni diff | noDice diff | Psi diff | Genfer diff | SPPL diff | Pluck diff | PITA diff | aspmc diff | Scallop diff | Storm sparse diff | Storm exact diff | Storm dd diff | PRISM sparse diff | PRISM hybrid diff | Ace diff | Ace c2d diff | Merlin diff | pyAgrum diff | pgmpy diff | libDAI diff | TI greedy diff | TI treesa diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| link_4 | 0.008 | 0.186 | 0.056 | 0.005 | 0.005 | 0.024 | 0.048 | >600 | 5.61 | 12.2 | 3.96 | 0.435 | 2.59 | oom | 0.155 | 0.155 | 0.462 | 1.27 | 1.27 | 1.3 | 1.33 | 0.237 | 1.45 | 19 | 0.237 | 0.0559 | 6.69 | 11.3 | 0.716 | exact | 0.0e+00 / 0.0e+00 | 3.1e-09 / 5.6e-09 | 4.3e-13 / 7.5e-13 | 1.1e-16 / 2.0e-16 | 0.0e+00 / 0.0e+00 | 6.9e-09 / 1.2e-08 | 9.3e-08 / 1.6e-07 |  | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.0e-16 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 9.3e-08 / 1.6e-07 | 4.3e-13 / 7.5e-13 | 4.3e-13 / 7.5e-13 | 4.3e-13 / 7.5e-13 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| link_6 | 0.012 | 8.67 | 0.055 | 0.012 | 0.032 | 0.062 | 0.105 | - | oom | >600 | 5.16 | 0.615 | 2.63 | - | 0.187 | 0.205 | 1.8 | 1.28 | 1.27 | 1.45 | 1.6 | 0.066 | 0.226 | 1.32 | 0.07 | 0.109 | 22 | 11.4 | 0.747 | exact | 1.1e-16 / 2.2e-16 | 2.6e-09 / 5.1e-09 | 2.2e-13 / 4.3e-13 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.3e-08 / 2.5e-08 | 3.9e-07 / 7.6e-07 |  |  |  | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 |  | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 3.9e-07 / 7.6e-07 | 2.2e-13 / 4.3e-13 | 2.2e-13 / 4.3e-13 | 2.2e-13 / 4.3e-13 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 |
| link_8 | 0.025 | >600 | 0.094 | 0.164 | 0.494 | 0.358 | 0.779 | - | - | - | 13 | >600 | 2.97 | - | 0.537 | 0.729 | 6.13 | 1.26 | 1.26 | 2.2 | 3.33 | 0.093 | 0.187 | 1.46 | 0.103 | 0.183 | 46.5 | 11.5 | 1.2 | exact | 0.0e+00 / 0.0e+00 |  | 1.7e-13 / 3.6e-13 | 5.0e-16 / 1.0e-15 | 5.6e-17 / 1.2e-16 | 3.0e-08 / 6.3e-08 | 5.3e-07 / 1.1e-06 |  |  |  | 5.6e-17 / 1.2e-16 |  | 1.7e-16 / 3.5e-16 |  | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 0.0e+00 / 0.0e+00 | 1.7e-16 / 3.5e-16 | 4.7e-07 / 9.9e-07 | 1.7e-13 / 3.6e-13 | 1.7e-13 / 3.6e-13 | 1.7e-13 / 3.6e-13 | 5.6e-17 / 1.2e-16 | 1.1e-16 / 2.3e-16 |
| link_10 | 0.183 | - | 0.624 | 1.9 | 5.67 | 2.69 | 7.31 | - | - | - | err | - | 4.87 | - | 4.03 | 5.69 | 12.3 | 2.15 | 2.12 | 4.5 | 29.9 | 0.396 | 0.427 | 24 | 0.413 | 0.766 | 86.2 | 13.4 | 6.49 | exact | 8.3e-16 / 1.8e-15 |  | 2.3e-14 / 5.1e-14 | 1.6e-15 / 3.5e-15 | 8.3e-16 / 1.8e-15 | 4.9e-09 / 1.1e-08 | 3.0e-07 / 6.7e-07 |  |  |  |  |  | 8.3e-16 / 1.8e-15 |  | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 3.0e-07 / 6.7e-07 | 2.3e-14 / 5.1e-14 | 2.3e-14 / 5.1e-14 | 2.3e-14 / 5.1e-14 | 8.9e-16 / 1.9e-15 | 8.3e-16 / 1.8e-15 |
| link_12 | 1.47 | - | 4.9 | 17.8 | 48.7 | 18.7 | 65 | - | - | - | - | - | 33.1 | - | 33.6 | 45.5 | 35.1 | 9.35 | 8.93 | 19.1 | 271 | 8.04 | 2.04 | 7.91 | 7.27 | 36.6 | 148 | 29.4 | 50.2 | exact | 7.2e-16 / 1.6e-15 |  | 8.3e-15 / 1.9e-14 | 1.6e-15 / 3.7e-15 | 6.1e-16 / 1.4e-15 | 3.1e-08 / 7.1e-08 | 1.7e-07 / 3.8e-07 |  |  |  |  |  | 5.6e-16 / 1.3e-15 |  | 6.1e-16 / 1.4e-15 | 5.6e-16 / 1.3e-15 | 6.1e-16 / 1.4e-15 | 6.1e-16 / 1.4e-15 | 6.1e-16 / 1.4e-15 | 5.0e-16 / 1.1e-15 | 6.1e-16 / 1.4e-15 | 1.7e-07 / 3.8e-07 | 8.3e-15 / 1.9e-14 | 8.3e-15 / 1.9e-14 | 8.3e-15 / 1.9e-14 | 5.6e-16 / 1.3e-15 | 7.2e-16 / 1.6e-15 |
| link_14 | 3.27 | - | 32.7 | 401 (2/3) | oom | 122 | 468 | - | - | - | - | - | 364 | - | 253 | 324 | 149 | 63.1 | 73.7 | >600 | >600 | oom | oom | 275 | oom | err | 206 | 414 (2/3) | oom | exact | 8.9e-16 / 2.1e-15 |  | 2.3e-13 / 5.3e-13 | 5.0e-16 / 1.2e-15 |  | 2.3e-08 / 5.5e-08 | 7.7e-08 / 1.8e-07 |  |  |  |  |  | 7.2e-16 / 1.7e-15 |  | 7.2e-16 / 1.7e-15 | 7.8e-16 / 1.8e-15 | 7.2e-16 / 1.7e-15 | 7.2e-16 / 1.7e-15 | 7.2e-16 / 1.7e-15 |  |  |  |  | 2.3e-13 / 5.3e-13 |  |  | 7.8e-16 / 1.8e-15 |
| link_16 | 53 | - | 430 | >600 | - | >600 | >600 | - | - | - | - | - | >600 | - | >600 | >600 | >600 | 439 | 530 | - | - | - | - | >600 | - | err | 299 | >600 | - | exact | 6.2e-15 / 1.5e-14 |  | 4.7e-13 / 1.1e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 6.3e-15 / 1.5e-14 | 6.3e-15 / 1.5e-14 |  |  |  |  |  |  |  | 6.3e-15 / 1.5e-14 |
| link_18 | 148 | - | >600 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | >600 | >600 | - | - | - | - | - | - | oom | 448 | - | - | exact | 1.7e-15 / 4.0e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.8e-15 / 4.3e-15 |
| link_20 | oom | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | >600 | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| link_24 | oom | >600 | >600 | >600 | - | >600 | oom | >600 | oom | >600 | - | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | - | - | - | - | - | - | - | - | >600 | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_4 | 0.007 | 0.163 | 0.026 | 0.005 | 0.006 | 0.017 | 0.042 | >600 | 0.006 | 1.68 | 3.79 | 0.397 | 2.59 | oom | 0.157 | 0.156 | 0.355 | 1.26 | 1.28 | 1.23 | 1.32 | 0.044 | 0.132 | 17.3 | 0.045 | 0.0687 | 12.8 | 11.3 | 0.71 | exact | 1.1e-16 / 2.2e-16 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.2e-16 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 |  | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.1e-16 / 2.2e-16 | 0.0e+00 / 0.0e+00 |
| router_6 | 0.012 | 16.5 | 0.047 | 0.014 | 0.023 | 0.034 | 0.074 | - | 0.015 | 44.6 | 5.26 | 1.4 | 2.7 | - | 0.169 | 0.169 | 0.893 | 1.27 | 1.28 | 1.61 | 1.7 | 0.071 | 0.268 | 1.39 | 0.081 | 0.148 | 35 | 11.1 | 0.725 | exact | 0.0e+00 / 0.0e+00 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 4.4e-16 / 8.9e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.2e-16 |  | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 0.0e+00 / 0.0e+00 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 |
| router_8 | 0.027 | >600 | 0.064 | 0.171 | 0.197 | 0.098 | 0.486 | - | 0.064 | >600 | 6.35 | >600 | 3 | - | 0.254 | 0.302 | 1.73 | 1.27 | 1.27 | 1.79 | 2.19 | 0.106 | 0.254 | 1.5 | 0.144 | 0.244 | 74.3 | 11.1 | 0.901 | exact | 0.0e+00 / 0.0e+00 |  | 1.4e-13 / 2.7e-13 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 8.9e-16 / 1.8e-15 |  | 1.1e-16 / 2.2e-16 |  | 0.0e+00 / 0.0e+00 |  | 4.4e-16 / 8.9e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.2e-16 |
| router_10 | 0.064 | - | 0.231 | 1.76 | 2.09 | 0.546 | 3.71 | - | 0.34 | - | 17.6 | - | 5.14 | - | 0.955 | 1.35 | 3.33 | 1.31 | 1.29 | 2.4 | 4.26 | 0.169 | 0.376 | 1.52 | 0.276 | 0.364 | 130 | 12.6 | 2.85 | exact | 2.2e-16 / 4.4e-16 |  | 1.4e-13 / 2.7e-13 | 1.1e-16 / 2.2e-16 | 3.3e-16 / 6.7e-16 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 1.2e-15 / 2.4e-15 |  | 3.3e-16 / 6.7e-16 |  | 3.3e-16 / 6.7e-16 |  | 2.2e-16 / 4.4e-16 | 3.3e-16 / 6.7e-16 | 4.4e-16 / 8.9e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 2.2e-16 / 4.4e-16 | 3.3e-16 / 6.7e-16 |
| router_12 | 0.101 | - | 1.48 | 16.8 | 17 | 3.65 | 26.1 | - | 1.99 | - | err | - | 32.2 | - | 6.63 | 9.31 | 5.57 | 2.04 | 1.94 | 4.06 | 23.4 | 0.345 | 0.719 | 1.76 | 0.573 | 0.617 | 203 | 28.2 | 17.9 | exact | 6.7e-16 / 1.3e-15 |  | 1.3e-13 / 2.7e-13 | 5.6e-16 / 1.1e-15 | 3.3e-16 / 6.7e-16 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 2.4e-15 / 4.9e-15 |  |  |  | 4.4e-16 / 8.9e-16 |  | 8.9e-16 / 1.8e-15 | 3.3e-16 / 6.7e-16 | 2.2e-16 / 4.4e-16 | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 | 4.4e-16 / 8.9e-16 | 4.4e-16 / 8.9e-16 | 4.5e-07 / 9.0e-07 | 1.3e-13 / 2.7e-13 | 1.3e-13 / 2.7e-13 | 1.3e-13 / 2.7e-13 | 3.3e-16 / 6.7e-16 | 4.4e-16 / 8.9e-16 |
| router_14 | 0.383 | - | 12.1 | 138 | oom | 23.6 | 180 | - | 11.3 | - | - | - | 536 | - | 48.4 | 65 | 8 | 5.62 | 5.55 | 9.64 | 223 | 0.937 | 1.22 | 2.08 | 1.64 | 0.57 | 296 | 150 | oom | exact | 5.6e-16 / 1.1e-15 |  | 1.4e-13 / 2.7e-13 | 2.2e-16 / 4.4e-16 |  | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 2.3e-15 / 4.7e-15 |  |  |  | 3.3e-16 / 6.7e-16 |  | 3.3e-16 / 6.7e-16 | 4.4e-16 / 8.9e-16 | 5.6e-16 / 1.1e-15 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 4.4e-16 / 8.9e-16 | 5.6e-16 / 1.1e-15 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 3.3e-16 / 6.7e-16 | 4.4e-16 / 8.9e-16 |
| router_16 | 1.14 | - | 89.7 | >600 | - | 142 | >600 | - | 61.6 | - | - | - | >600 | - | 333 | 427 | 10.6 | 28.1 | 32.6 | 88.6 | >600 | 24.9 | 6.52 | 2.71 | 9.62 | 0.916 | 416 | >600 | - | exact | 1.4e-14 / 2.9e-14 |  | 1.5e-13 / 3.0e-13 |  |  | 5.0e-08 / 1.0e-07 |  |  | 1.1e-14 / 2.2e-14 |  |  |  |  |  | 1.4e-14 / 2.8e-14 | 1.5e-14 / 3.0e-14 | 1.5e-14 / 3.0e-14 | 1.5e-14 / 2.9e-14 | 1.5e-14 / 2.9e-14 | 1.5e-14 / 3.0e-14 |  | 4.5e-07 / 9.0e-07 | 1.5e-13 / 3.0e-13 | 1.5e-13 / 3.0e-13 | 1.5e-13 / 3.0e-13 | 1.5e-14 / 3.0e-14 | 1.5e-14 / 2.9e-14 |
| router_18 | 4.26 | - | >600 | - | - | >600 | - | - | 309 | - | - | - | - | - | >600 | >600 | 15.7 | 169 | 207 | >600 | - | oom | oom | 3.63 | 57.6 | 2.31 | >600 | - | - | exact | 1.9e-14 / 3.7e-14 |  |  |  |  |  |  |  | 1.5e-14 / 2.9e-14 |  |  |  |  |  |  |  | 1.9e-14 / 3.8e-14 | 1.9e-14 / 3.8e-14 | 1.9e-14 / 3.8e-14 |  |  |  |  | 1.5e-13 / 3.1e-13 | 1.5e-13 / 3.1e-13 | 1.9e-14 / 3.8e-14 |  |
| router_20 | 5.66 | - | - | - | - | - | - | - | >600 | - | - | - | - | - | - | - | 20.6 | >600 | >600 | - | - | - | - | 5.97 | oom | 11.2 | >600 | - | - | exact | 5.9e-14 / 1.2e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.8e-14 / 1.2e-13 |  |  |  |  |  |  | 7.6e-14 / 1.5e-13 |  | 5.9e-14 / 1.2e-13 |  |
| router_24 | 89.7 | >600 | >600 | oom | - | >600 | oom | >600 | >600 | >600 | - | >600 | >600 | oom | >600 | >600 | 38.5 | 7.85 | 8.47 | - | - | - | - | >600 | - | oom | - | oom | - | exact | 2.4e-11 / 4.8e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.4e-11 / 4.8e-11 | 2.4e-11 / 4.8e-11 | 2.4e-11 / 4.8e-11 |  |  |  |  |  |  |  |  |
| router_28 | oom | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 64.3 | 13.7 | 13.8 | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_32 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 104 | 22.8 | 23.1 | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_40 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 266 | 61.3 | 61.3 | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_48 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | >600 | 223 | 150 | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_56 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | err | err | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_64 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | err | err | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_72 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | >600 | >600 | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| weather_h1 | 0.004 | 0.162 | 0.053 | 0.005 | 0.004 | 0.049 | err | err | 0.006 | 0.602 | 2.49 | 0.338 | 1.56 | 0.019 | 2.64 | 12.7 | 20.4 | 10.1 | >600 | 1.1 | 1.14 | zero | 0.315 | 1.52 | 0.229 | 0.046 | 1.26 | 10.5 | 0.713 | exact | 2.6e-23 / 2.8e-16 | 4.0e-16 / 4.3e-09 | 4.1e-20 / 4.4e-13 | 6.6e-23 / 7.1e-16 | 0.0e+00 / 0.0e+00 | 6.4e-09 / 6.9e-02 |  |  | 0.0e+00 / 0.0e+00 | 7.7e-21 / 8.2e-14 | 2.6e-23 / 2.8e-16 | 1.2e-17 / 1.3e-10 | 2.6e-23 / 2.8e-16 | 5.3e-23 / 5.7e-16 | 2.6e-23 / 2.8e-16 | 1.3e-23 / 1.4e-16 | 1.3e-23 / 1.4e-16 | 5.3e-23 / 5.7e-16 |  | 2.6e-23 / 2.8e-16 | 1.3e-23 / 1.4e-16 |  | 4.1e-20 / 4.4e-13 | 4.1e-20 / 4.4e-13 | 4.1e-20 / 4.4e-13 | 5.3e-23 / 5.7e-16 | 9.3e-23 / 9.9e-16 |
| weather_h2 | 0.016 | 1.06 | 0.063 | 0.018 | 0.044 | 0.19 | err | err | oom | 134 | 5.21 | 0.413 | 2.68 | oom | 2.62 | 19 | 23.3 | 10.5 | - | 1.44 | 1.56 | zero | 0.3 | 1.82 | 0.127 | 0.124 | 28.5 | 13.5 | 0.748 | exact | 2.6e-23 / 2.1e-16 | 3.5e-16 / 2.8e-09 | 4.8e-19 / 3.9e-12 | 1.1e-22 / 8.6e-16 | 2.6e-23 / 2.1e-16 | 2.4e-08 / 1.9e-01 |  |  |  | 5.2e-21 / 4.2e-14 | 2.6e-23 / 2.1e-16 | 8.3e-17 / 6.7e-10 | 2.6e-23 / 2.1e-16 |  | 1.3e-22 / 1.1e-15 | 2.6e-23 / 2.1e-16 | 0.0e+00 / 0.0e+00 | 4.4e-18 / 3.6e-11 |  | 0.0e+00 / 0.0e+00 | 5.3e-23 / 4.3e-16 |  | 4.8e-19 / 3.9e-12 | 4.8e-19 / 3.9e-12 | 4.8e-19 / 3.9e-12 | 7.9e-23 / 6.4e-16 | 7.9e-23 / 6.4e-16 |
| weather_h3 | 0.036 | 3.74 | 0.124 | 0.12 | 0.572 | 0.417 | err | err | - | >600 | 6.54 | 0.622 | 2.8 | - | 2.64 | 29.1 | 25.4 | 10.6 | - | 1.69 | 1.83 | zero | 0.214 | oom | 0.18 | 0.211 | 59.2 | 17.3 | 1.29 | exact | 5.3e-23 / 3.3e-16 | 2.0e-15 / 1.3e-08 | 1.6e-19 / 1.0e-12 | 1.9e-22 / 1.2e-15 | 2.6e-23 / 1.7e-16 | 4.2e-08 / 2.6e-01 |  |  |  |  | 7.9e-23 / 5.0e-16 | 9.9e-17 / 6.2e-10 | 1.1e-22 / 6.7e-16 |  | 3.7e-22 / 2.3e-15 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 7.4e-18 / 4.7e-11 |  | 7.9e-23 / 5.0e-16 | 2.6e-23 / 1.7e-16 |  | 1.6e-19 / 1.0e-12 |  | 1.6e-19 / 1.0e-12 | 7.9e-23 / 5.0e-16 | 1.1e-22 / 6.7e-16 |
| weather_h4 | 0.049 | 21.6 | 0.22 | 1.44 | 4.25 | 0.637 | err | err | - | - | 29.9 | 3.21 | 2.98 | - | 2.65 | 41.5 | 26.6 | 10.1 | - | 2.14 | 4.46 | zero | 0.263 | - | 0.324 | 0.312 | 95.8 | 18.5 | 5.04 | exact | 5.3e-23 / 2.7e-16 | 4.4e-15 / 2.3e-08 | 7.6e-20 / 3.9e-13 | 1.3e-22 / 6.9e-16 | 1.1e-22 / 5.5e-16 | 7.3e-09 / 3.8e-02 |  |  |  |  | 1.3e-22 / 6.9e-16 | 6.2e-17 / 3.2e-10 | 1.3e-22 / 6.9e-16 |  | 4.0e-22 / 2.1e-15 | 0.0e+00 / 0.0e+00 | 2.6e-23 / 1.4e-16 | 1.0e-17 / 5.4e-11 |  | 7.9e-23 / 4.1e-16 | 5.3e-23 / 2.7e-16 |  | 7.6e-20 / 3.9e-13 |  | 7.6e-20 / 3.9e-13 | 1.1e-22 / 5.5e-16 | 1.1e-22 / 5.5e-16 |
| weather_h5 | 0.118 | 67.4 | 0.341 | 6.63 | 15.8 | 0.846 | err | err | - | - | err | 21.3 | 3.96 | - | 2.63 | 57.3 | 28.4 | 9.87 | - | 2.38 | 8.02 | zero | 0.352 | - | 0.54 | 0.375 | 138 | 23.6 | 16.8 | exact | 1.9e-22 / 8.2e-16 | 1.6e-15 / 7.1e-09 | 1.2e-19 / 5.3e-13 | 0.0e+00 / 0.0e+00 | 7.9e-23 / 3.5e-16 | 2.7e-08 / 1.2e-01 |  |  |  |  |  | 1.2e-16 / 5.2e-10 | 1.1e-22 / 4.7e-16 |  | 3.4e-22 / 1.5e-15 | 0.0e+00 / 0.0e+00 | 5.3e-23 / 2.3e-16 | 1.3e-17 / 5.9e-11 |  | 1.3e-22 / 5.8e-16 | 5.3e-23 / 2.3e-16 |  | 1.2e-19 / 5.3e-13 |  | 1.2e-19 / 5.3e-13 | 1.1e-22 / 4.7e-16 | 1.3e-22 / 5.8e-16 |
| weather_h6 | 0.144 | oom | 0.403 | 16.4 | 36.4 | 1.08 | err | err | - | - | - | 165 | 4.83 | - | 2.64 | 75.8 | 30.4 | 9.93 | - | 2.6 | 10.3 | zero | 0.468 | - | 0.803 | 0.507 | 170 | 33.5 | 37.7 | exact | 2.1e-22 / 8.1e-16 |  | 5.0e-19 / 1.9e-12 | 0.0e+00 / 0.0e+00 | 1.1e-22 / 4.0e-16 | 3.8e-08 / 1.5e-01 |  |  |  |  |  | 1.7e-16 / 6.6e-10 | 1.6e-22 / 6.1e-16 |  | 6.4e-22 / 2.4e-15 | 5.3e-23 / 2.0e-16 | 5.3e-23 / 2.0e-16 | 1.6e-17 / 6.3e-11 |  | 1.1e-22 / 4.0e-16 | 2.1e-22 / 8.1e-16 |  | 5.0e-19 / 1.9e-12 |  | 5.0e-19 / 1.9e-12 | 0.0e+00 / 0.0e+00 | 2.1e-22 / 8.1e-16 |
| weather_h7 | 0.161 | - | 0.472 | 31.3 | 58.8 | 1.28 | err | err | - | - | - | >600 | 7.06 | - | 2.65 | 97.1 | 32.5 | 9.94 | - | 3.26 | 19 | zero | 0.613 | - | 1.26 | 0.638 | 206 | 48.2 | 60.3 | exact | 3.7e-22 / 1.3e-15 |  | 3.8e-19 / 1.3e-12 | 0.0e+00 / 0.0e+00 | 2.1e-22 / 7.2e-16 | 4.0e-09 / 1.3e-02 |  |  |  |  |  |  | 3.2e-22 / 1.1e-15 |  | 3.7e-22 / 1.3e-15 | 5.3e-23 / 1.8e-16 | 1.1e-22 / 3.6e-16 | 2.0e-17 / 6.6e-11 |  | 1.6e-22 / 5.4e-16 | 3.7e-22 / 1.3e-15 |  | 3.8e-19 / 1.3e-12 |  | 3.8e-19 / 1.3e-12 | 1.1e-22 / 3.6e-16 | 3.2e-22 / 1.1e-15 |
| weather_h8 | 0.196 | - | 0.744 | 50.5 | 102 | 1.53 | err | err | - | - | - | - | 8.3 | - | 2.65 | 121 | 34.7 | 9.88 | - | 3.46 | 22.2 | zero | 0.749 | - | 1.66 | 1.11 | 251 | 68.2 | 104 | exact | 3.7e-22 / 1.1e-15 |  | 2.3e-19 / 7.1e-13 | 5.8e-22 / 1.8e-15 | 1.1e-22 / 3.2e-16 | 3.0e-08 / 9.2e-02 |  |  |  |  |  |  | 2.6e-22 / 8.0e-16 |  | 3.7e-22 / 1.1e-15 | 5.3e-23 / 1.6e-16 | 1.6e-22 / 4.8e-16 | 2.3e-17 / 6.8e-11 |  | 2.6e-22 / 8.0e-16 | 1.6e-22 / 4.8e-16 |  | 2.3e-19 / 7.1e-13 |  | 2.3e-19 / 7.1e-13 | 5.3e-23 / 1.6e-16 | 3.7e-22 / 1.1e-15 |
| weather_h9 | 0.183 | - | 0.828 | 72 | oom | 1.74 | err | err | - | - | - | - | 16.8 | - | 2.66 | 147 | 37.4 | 10.9 | - | 3.89 | 30.7 | zero | 0.997 | - | 2.14 | 0.634 | 286 | 89.5 | oom | exact | 2.6e-22 / 7.3e-16 |  | 1.2e-19 / 3.4e-13 | 1.1e-21 / 2.9e-15 |  | 3.5e-08 / 9.6e-02 |  |  |  |  |  |  | 1.6e-22 / 4.4e-16 |  | 1.3e-21 / 3.5e-15 | 5.3e-23 / 1.5e-16 | 5.3e-23 / 1.5e-16 | 2.6e-17 / 7.0e-11 |  | 2.1e-22 / 5.8e-16 | 3.7e-22 / 1.0e-15 |  | 1.2e-19 / 3.4e-13 |  | 1.2e-19 / 3.4e-13 | 2.1e-22 / 5.8e-16 | 4.8e-22 / 1.3e-15 |
| weather_h100 | 2.77 | >600 | 21.1 | oom | - | 25.9 | >600 | err | oom | >600 | - | >600 | >600 | oom | 3.53 | >600 | 413 | 13.1 | >600 | 30.2 | >600 | oom | 169 | - | 344 | >600 | >600 | oom | - | exact | 4.1e-20 / 1.2e-14 |  | 2.1e-18 / 6.0e-13 |  |  | 1.0e-09 / 2.9e-04 |  |  |  |  |  |  |  |  | 1.4e-19 / 3.9e-14 |  | 1.7e-20 / 4.7e-15 | 3.0e-16 / 8.6e-11 |  | 2.1e-21 / 6.1e-16 |  |  | 2.1e-18 / 6.0e-13 |  | 2.1e-18 / 6.0e-13 |  |  |
| weather_h200 | 7.05 | - | 51.9 | - | - | 53.4 | - | err | - | - | - | - | - | - | 4.16 | - | >600 | 13.9 | - | 69.6 | - | - | >600 | - | >600 | - | - | - | - | exact | 1.2e-19 / 1.8e-14 |  | 4.5e-18 / 6.4e-13 |  |  | 4.3e-08 / 6.2e-03 |  |  |  |  |  |  |  |  | 5.5e-19 / 8.0e-14 |  |  | 6.0e-16 / 8.7e-11 |  | 7.6e-21 / 1.1e-15 |  |  |  |  |  |  |  |
| weather_h300 | 11 | - | 76.5 | - | - | 79.6 | - | err | - | - | - | - | - | - | 4.7 | - | - | 14.3 | - | >600 | - | - | - | - | - | - | - | - | - | exact | 2.7e-19 / 2.6e-14 |  | 4.2e-18 / 4.1e-13 |  |  | 1.3e-08 / 1.3e-03 |  |  |  |  |  |  |  |  | 1.3e-18 / 1.2e-13 |  |  | 9.0e-16 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h400 | 14.3 | - | 124 | - | - | 107 | - | err | - | - | - | - | - | - | 5.26 | - | - | 14.5 | - | - | - | - | - | - | - | - | - | - | - | exact | 4.7e-19 / 3.4e-14 |  | 2.6e-17 / 1.9e-12 |  |  | 3.1e-08 / 2.2e-03 |  |  |  |  |  |  |  |  | 2.2e-18 / 1.6e-13 |  |  | 1.2e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h500 | 18.2 | - | 170 | - | - | 133 | - | err | - | - | - | - | - | - | 5.78 | - | - | 15.1 | - | - | - | - | - | - | - | - | - | - | - | exact | 7.2e-19 / 4.2e-14 |  | 4.0e-17 / 2.3e-12 |  |  | 2.5e-08 / 1.4e-03 |  |  |  |  |  |  |  |  | 3.5e-18 / 2.0e-13 |  |  | 1.5e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h600 | 22.9 | oom | 220 | oom | - | 163 | >600 | err | err | >600 | err | >600 | >600 | >600 | 6.59 | >600 | >600 | 16.5 | >600 | - | - | - | - | - | - | - | - | oom | - | exact | 1.1e-18 / 5.2e-14 |  | 3.6e-17 / 1.7e-12 |  |  | 1.9e-08 / 9.1e-04 |  |  |  |  |  |  |  |  | 5.0e-18 / 2.4e-13 |  |  | 1.8e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h700 | 26.6 | - | 268 | - | - | 186 | - | err | err | - | - | - | - | - | 7.05 | - | - | 16.7 | - | - | - | - | - | - | - | - | - | - | - | exact | 1.4e-18 / 5.6e-14 |  | 1.9e-18 / 7.8e-14 |  |  | 3.7e-08 / 1.5e-03 |  |  |  |  |  |  |  |  | 6.8e-18 / 2.8e-13 |  |  | 2.1e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h800 | 30.6 | - | 376 | - | - | 216 | - | err | err | - | - | - | - | - | 7.52 | - | - | 17.1 | - | - | - | - | - | - | - | - | - | - | - | exact | 1.8e-18 / 6.7e-14 |  | 2.1e-17 / 7.6e-13 |  |  | 6.8e-09 / 2.5e-04 |  |  |  |  |  |  |  |  | 9.0e-18 / 3.3e-13 |  |  | 2.4e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h900 | 34.7 | - | 447 | - | - | 240 | - | err | err | - | - | - | - | - | 8.36 | - | - | 18.5 | - | - | - | - | - | - | - | - | - | - | - | exact | 2.3e-18 / 7.4e-14 |  | 7.9e-18 / 2.6e-13 |  |  | 4.9e-08 / 1.6e-03 |  |  |  |  |  |  |  |  | 1.1e-17 / 3.7e-13 |  |  | 2.7e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h1000 | 39.8 | - | 523 | - | - | 271 | - | err | err | - | - | - | - | - | 8.82 | - | - | 18.9 | - | - | - | - | - | - | - | - | - | - | - | exact | 2.7e-18 / 8.0e-14 |  | 3.0e-17 / 8.6e-13 |  |  | 5.3e-09 / 1.5e-04 |  |  |  |  |  |  |  |  | 1.4e-17 / 4.1e-13 |  |  | 3.0e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |
| weather_h3000 | 116 | - | >600 | - | - | >600 | - | err | - | - | - | - | - | - | 21 | - | - | 29.5 | - | - | - | - | - | - | - | - | - | - | - | exact | 2.1e-17 / 2.1e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.3e-16 / 1.2e-12 |  |  | 9.1e-15 / 8.8e-11 |  |  |  |  |  |  |  |  |  |
| weather_h10000 | oom | - | - | - | - | - | - | err | - | - | - | - | - | - | 61.5 | - | - | 69.7 | - | - | - | - | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.4e-15 / 4.1e-12 |  |  | 3.0e-14 / 8.8e-11 |  |  |  |  |  |  |  |  |  |
| weather_h30000 | - | - | - | - | - | - | - | err | - | - | - | - | - | - | 189 | - | - | 183 | - | - | - | - | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.3e-14 / 1.2e-11 |  |  | 9.3e-14 / 9.0e-11 |  |  |  |  |  |  |  |  |  |
| weather_n2 | 0.026 | 5.69 | 0.05 | 0.023 | 0.059 | 0.022 | err | err | 0.135 | >600 | 6.15 | 0.56 | 2.76 | oom | 0.21 | 0.209 | 0.211 | 1.59 | 1.57 | 1.72 | 1.85 | 0.173 | 0.287 | 16.7 | 0.218 | 0.445 | 57.2 | 14 | 0.764 | exact | 2.8e-17 / 1.5e-16 | 3.2e-10 / 1.7e-09 | 3.9e-13 / 2.1e-12 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 4.0e-08 / 2.2e-07 |  |  | 2.8e-17 / 1.5e-16 |  | 2.8e-17 / 1.5e-16 | 5.6e-16 / 3.0e-15 | 8.3e-17 / 4.5e-16 |  | 5.6e-17 / 3.0e-16 | 2.8e-17 / 1.5e-16 | 8.3e-17 / 4.5e-16 | 2.8e-17 / 1.5e-16 | 2.8e-17 / 1.5e-16 | 2.8e-17 / 1.5e-16 | 8.3e-17 / 4.5e-16 | 6.0e-08 / 3.3e-07 | 3.9e-13 / 2.1e-12 | 3.9e-13 / 2.1e-12 | 3.9e-13 / 2.1e-12 | 1.7e-16 / 9.0e-16 | 2.5e-16 / 1.4e-15 |
| weather_n7 | 0.064 | oom | 0.099 | 4.8 | 9.64 | 0.188 | err | err | >600 | - | err | >600 | 4.92 | - | 0.246 | 2 | 5.47 | 2.39 | 20.6 | 2.63 | 4.28 | 0.317 | 0.65 | 102 | 1.1 | 0.664 | 219 | 21.4 | 10.5 | exact | 1.4e-20 / 2.0e-16 |  | 1.9e-17 / 2.8e-13 | 2.7e-20 / 4.0e-16 | 2.7e-20 / 4.0e-16 | 3.6e-08 / 5.4e-04 |  |  |  |  |  |  | 4.1e-20 / 6.0e-16 |  | 2.0e-19 / 3.0e-15 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 1.4e-20 / 2.0e-16 | 1.4e-20 / 2.0e-16 | 2.7e-20 / 4.0e-16 | 1.4e-20 / 2.0e-16 | 3.6e-07 / 5.4e-03 | 1.9e-17 / 2.8e-13 | 1.9e-17 / 2.8e-13 | 1.9e-17 / 2.8e-13 | 9.5e-20 / 1.4e-15 | 1.6e-19 / 2.4e-15 |
| weather_n8 | 0.147 | - | 0.203 | 14 | 25.6 | 0.403 | err | err | - | - | - | - | 5.95 | - | 0.347 | 9.11 | 9.72 | 13.9 | >600 | 2.81 | 6.4 | 0.39 | 0.755 | 593 | 1.51 | 1.11 | 255 | 31.4 | 26.6 | exact | 1.9e-20 / 1.4e-15 |  | 3.7e-17 / 2.9e-12 | 2.4e-20 / 1.8e-15 | 5.1e-21 / 3.9e-16 | 1.1e-08 / 8.8e-04 |  |  |  |  |  |  | 1.4e-20 / 1.0e-15 |  | 4.1e-20 / 3.1e-15 | 1.7e-21 / 1.3e-16 | 5.1e-21 / 3.9e-16 | 1.7e-21 / 1.3e-16 |  | 1.4e-20 / 1.0e-15 | 1.0e-20 / 7.8e-16 | 1.1e-08 / 8.8e-04 | 3.7e-17 / 2.9e-12 | 3.7e-17 / 2.9e-12 | 3.7e-17 / 2.9e-12 | 8.5e-21 / 6.5e-16 | 2.0e-20 / 1.6e-15 |
| weather_n9 | 0.186 | - | 0.473 | 39.5 | 77.4 | 0.94 | err | err | - | - | - | - | 7.55 | - | 0.823 | 42.3 | 18.1 | 197 | - | 3.49 | 11.6 | 0.853 | 1.27 | oom | 2.33 | 0.599 | 291 | 57.7 | 79.4 | exact | 3.8e-21 / 1.5e-15 |  | 4.2e-18 / 1.7e-12 | 6.8e-21 / 2.7e-15 | 1.7e-21 / 6.7e-16 | 1.9e-08 / 7.4e-03 |  |  |  |  |  |  | 3.0e-21 / 1.2e-15 |  | 6.4e-21 / 2.5e-15 | 8.5e-22 / 3.4e-16 | 1.3e-21 / 5.0e-16 | 2.5e-21 / 1.0e-15 |  | 3.4e-21 / 1.3e-15 | 2.1e-21 / 8.4e-16 | 4.8e-07 / 1.9e-01 | 4.2e-18 / 1.7e-12 |  | 4.2e-18 / 1.7e-12 | 8.5e-22 / 3.4e-16 | 2.1e-21 / 8.4e-16 |
| weather_n10 | 0.203 | - | 1.18 | 110 | oom | 2.16 | err | err | - | - | - | - | 20.3 | - | 2.82 | 191 | 52.6 | 12.7 | - | 4.27 | 20.6 | zero | 1.2 | - | 2.67 | 0.764 | 331 | 129 | oom | exact | 4.2e-22 / 1.1e-15 |  | 3.7e-19 / 9.4e-13 | 1.5e-21 / 3.8e-15 |  | 6.5e-10 / 1.6e-03 |  |  |  |  |  |  | 2.6e-22 / 6.6e-16 |  | 6.4e-22 / 1.6e-15 | 1.1e-22 / 2.7e-16 | 1.1e-22 / 2.7e-16 | 2.9e-17 / 7.1e-11 |  | 4.2e-22 / 1.1e-15 | 5.3e-23 / 1.3e-16 |  | 3.7e-19 / 9.4e-13 |  | 3.7e-19 / 9.4e-13 | 2.1e-22 / 5.3e-16 | 3.2e-22 / 8.0e-16 |
| weather_n11 | 0.307 | - | 2.38 | 266 | - | 4.77 | err | err | - | - | - | - | 41.1 | - | 10.9 | >600 | 207 | err | - | 6.16 | 98.7 | zero | 1.73 | - | 4.29 | 0.808 | 377 | 287 | - | exact | 7.9e-23 / 1.2e-15 |  | 1.4e-21 / 2.1e-14 | 9.3e-23 / 1.5e-15 |  | 3.6e-08 / 5.7e-01 |  |  |  |  |  |  | 6.6e-23 / 1.0e-15 |  | 4.4e-22 / 6.9e-15 |  | 1.3e-23 / 2.1e-16 |  |  | 1.3e-23 / 2.1e-16 | 5.3e-23 / 8.3e-16 |  | 1.4e-21 / 2.1e-14 |  | 1.4e-21 / 2.1e-14 | 4.0e-23 / 6.2e-16 | 5.3e-23 / 8.3e-16 |
| weather_n12 | 0.802 | - | 5.92 | oom | - | zero | err | err | - | - | - | - | 216 | - | 45.3 | - | >600 | oom | - | 8.33 | 241 | zero | 2.36 | - | 8.74 | 1.35 | 405 | oom | - | exact | 1.3e-23 / 1.3e-15 |  | 8.7e-21 / 8.5e-13 |  |  |  |  |  |  |  |  |  | 1.2e-23 / 1.1e-15 |  | 6.5e-23 / 6.4e-15 |  |  |  |  | 6.6e-24 / 6.5e-16 | 2.8e-23 / 2.8e-15 |  | 8.7e-21 / 8.5e-13 |  | 8.7e-21 / 8.5e-13 | 0.0e+00 / 0.0e+00 | 9.9e-24 / 9.8e-16 |
| weather_n13 | 3.4 | - | 13.2 | - | - | zero | >600 | err | - | - | - | - | 176 | - | 176 | - | - | - | - | 15.5 | 345 | zero | 7.72 | - | 16.5 | 1.86 | 443 | - | - | exact | 4.1e-24 / 2.6e-15 |  | 4.7e-21 / 2.9e-12 |  |  |  |  |  |  |  |  |  | 3.1e-24 / 1.9e-15 |  | 1.9e-23 / 1.2e-14 |  |  |  |  | 1.7e-24 / 1.0e-15 | 1.9e-24 / 1.2e-15 |  | 4.7e-21 / 2.9e-12 |  | 4.7e-21 / 2.9e-12 | 2.1e-25 / 1.3e-16 | 1.4e-24 / 9.0e-16 |
| weather_n15 | 16.9 | >600 | 115 | >600 | - | zero | >600 | err | oom | >600 | - | >600 | >600 | oom | >600 | err | err | err | err | >600 | >600 | oom | oom | - | oom | 7.04 | >600 | >600 | - | exact | 1.0e-25 / 2.5e-15 |  | 3.1e-23 / 7.5e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 3.2e-26 / 7.8e-16 |  |
| weather_n16 | 21.8 | >600 | 432 | >600 | - | zero | >600 | err | oom | >600 | - | >600 | >600 | oom | >600 | err | err | oom | oom | - | - | - | - | - | - | 15.4 | >600 | >600 | - | exact | 0.0e+00 / 0.0e+00 |  | 3.7e-25 / 5.8e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.6e-27 / 2.6e-16 |  |
| weather_n17 | 83.9 | >600 | >600 | >600 | - | >600 | >600 | err | oom | >600 | - | >600 | >600 | oom | >600 | err | err | oom | oom | >600 | >600 | oom | oom | >600 | oom | 168 | >600 | >600 | - | exact | 4.0e-27 / 3.7e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.0e-28 / 1.9e-16 |  |
| weather_n18 | 84.1 | >600 | >600 | >600 | - | >600 | >600 | err | oom | >600 | - | >600 | >600 | oom | >600 | err | err | err | err | - | - | - | - | - | - | >600 | - | >600 | - | exact | 2.3e-28 / 1.4e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| weather_n19 | oom | >600 | >600 | >600 | - | >600 | >600 | err | oom | >600 | - | >600 | >600 | oom | >600 | err | err | oom | oom | - | - | - | - | - | - | - | - | >600 | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |

Answers. The largest differences from the reference: tddlog 4.8e-11
relative (2.4e-11 absolute, router_24); ProbLog 2.3e-08 relative
(4.4e-15 absolute, weather_h4); Dice 3.9e-12 relative (4.8e-19 absolute,
weather_h2); Alea 3.8e-15 relative (1.5e-21 absolute, weather_n10);
Roulette 1.8e-15 relative (8.3e-16 absolute, link_10); Geni 5.7e-01
relative (3.6e-08 absolute, weather_n11); noDice 1.1e-06 relative
(5.3e-07 absolute, link_8); Psi every answer the reference's; Genfer
2.9e-14 relative (1.5e-14 absolute, router_18); SPPL 8.2e-14 relative
(7.7e-21 absolute, weather_h1); Pluck 6.9e-16 relative (1.3e-22
absolute, weather_h4); PITA 6.7e-10 relative (8.3e-17 absolute,
weather_h2); aspmc 1.9e-15 relative (3.1e-24 absolute, weather_n13);
Scallop 5.7e-16 relative (5.3e-23 absolute, weather_h1); Storm sparse
1.2e-11 relative (1.3e-14 absolute, weather_h30000); Storm exact 3.0e-14
relative (1.5e-14 absolute, router_16); Storm dd 4.8e-11 relative
(2.4e-11 absolute, router_24); PRISM sparse 9.0e-11 relative (9.3e-14
absolute, weather_h30000); PRISM hybrid 4.8e-11 relative (2.4e-11
absolute, router_24); Ace 3.0e-14 relative (1.5e-14 absolute,
router_16); Ace c2d 2.8e-15 relative (2.8e-23 absolute, weather_n12);
Merlin 1.9e-01 relative (4.8e-07 absolute, weather_n9); pyAgrum 3.9e-12
relative (4.8e-19 absolute, weather_h2); pgmpy 3.9e-12 relative (4.8e-19
absolute, weather_h2); libDAI 3.9e-12 relative (4.8e-19 absolute,
weather_h2); TensorInference.jl greedy 1.2e-13 relative (5.9e-14
absolute, router_20); TensorInference.jl treesa 2.9e-14 relative
(1.5e-14 absolute, router_16).

## Further exact systems

The survey of exact systems (docs/baselines.md) adds systems that run
part of the program suites, each on the programs of its own paper, on
the Dice programs it accepts, on the ProbLog programs or, the network
tools, on the networks. Each ran in the same lanes as the systems above,
single-threaded, with the same limits; the time is the process's,
tddlog's too, start-up included. Every answer is compared with the
suite's reference, as Dice's and Alea.jl's are; an answer off the
reference by more than 1e-9 and either by more than 1e-6 or by more than
1e-3 of it, beyond the last digit either of the two prints is `wrong`.
Each comparison pairs a system with tddlog run on the same machine in
the same lanes. A time under 0.2 s counts as 0.2 s, and an instance both
answer in under 0.2 s is counted in neither's favour.

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
| Scallop 0.2.5 (668bfb6; Li, Huang and Naik, PLDI 2023) | exact, every proof kept (`allproofs`) | the ground program tddlog prints for each ProbLog program, through `ground2scallop.py` |
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

| family | instances | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Scallop | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Ace log space | Merlin | pyAgrum | pgmpy | libDAI | TensorInference.jl greedy | TensorInference.jl treesa | Roulette |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| bn | 17 | 17 | 13 of 17 | 14 of 17 | 4 of 17 | 14 of 17 | 6 of 17 | 13 of 17 | 11 of 17 | 12 of 17 | 2 of 17 | 14 of 17 | 13 of 17 | 15 of 17 | 14 of 17 | 14 of 17 | 17 of 17 | 16 of 17 |  | 17 of 17 | 17 of 17 | 15 of 17 | 17 of 17 | 15 of 17 | 17 of 17 | 16 of 17 |
| dice | 8 | 8 | 7 of 8 | 8 of 8 | 6 of 8 | 8 of 8 | 7 of 8 | 7 of 8 | 7 of 8 | 7 of 8 | 7 of 8 | 8 of 8 | 8 of 8 | 7 of 8 | 8 of 8 | 8 of 8 | 7 of 8 | 7 of 8 | 1 of 1 | 8 of 8 | 8 of 8 | 7 of 8 | 7 of 8 | 7 of 8 | 7 of 8 | 7 of 7 |

**Geni.** Geni ran 25 instances and answered 20 (1 fail, 3 time out, 1
answer wrongly). Its answers differ from the reference's by at most
1.8e-05 relative (1.2e-07 absolute, bn_hepar2) over 20 instances. Geni's
answer is wrong, zero or underflowed on bn_water. Over the 25 instances
given to both that either answers, a run that does not answer counted at
the 600 s limit, Geni takes 3139 s in all and tddlog 2.06 s; over the 20
both answer, 139 s against tddlog's 0.32. tddlog alone answers bn_link,
bn_munin, bn_pathfinder, bn_water and digitRecognition.

**noDice.** noDice ran 25 instances and answered 22 (3 time out). Its
answers differ from the reference's by at most 1.0e+00 relative (7.5e-07
absolute, bn_water) over 22 instances. Over the 25 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, noDice takes 2207 s in all and tddlog 2.06 s; over the 22 both
answer, 407 s against tddlog's 1.52. tddlog alone answers bn_alarm,
bn_hailfinder and bn_munin.

**Psi.** Psi ran 25 instances and answered 10 (1 fail, 14 time out). Its
answers differ from the reference's by at most 4.9e-16 relative (1.4e-17
absolute, bn_earthquake) over 10 instances. Over the 25 instances given
to both that either answers, a run that does not answer counted at the
600 s limit, Psi takes 9011 s in all and tddlog 2.06 s; over the 10 both
answer, 11 s against tddlog's 0.074. tddlog alone answers bn_alarm,
bn_andes, bn_child, bn_hailfinder, bn_hepar2 and 10 more.

**Genfer.** Genfer ran 25 instances and answered 22 (1 fail, 2 time
out). Its answers differ from the reference's by at most 1.9e+02
relative (5.8e-15 absolute, digitRecognition) over 22 instances. Over
the 25 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Genfer takes 2055 s in all and tddlog
2.06 s; over the 22 both answer, 255 s against tddlog's 0.7. Genfer is
faster on 1 (digitRecognition 0.1 s against 0.35). tddlog alone answers
bn_link, bn_munin and bn_pathfinder.

**SPPL.** SPPL ran 25 instances and answered 13 (12 time out). Its
answers differ from the reference's by at most 4.0e-10 relative (1.2e-10
absolute, bn_sachs) over 13 instances. Over the 25 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, SPPL takes 7221 s in all and tddlog 2.06 s; over the 13 both
answer, 20.8 s against tddlog's 0.1. tddlog alone answers bn_alarm,
bn_andes, bn_hailfinder, bn_hepar2, bn_insurance and 7 more.

**Pluck.** Pluck ran 25 instances and answered 20 (3 fail, 1 run out of
memory, 1 time out). Its answers differ from the reference's by at most
4.0e-10 relative (1.2e-10 absolute, bn_sachs) over 20 instances. Over
the 25 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Pluck takes 3202 s in all and tddlog
2.06 s; over the 20 both answer, 202 s against tddlog's 0.31. tddlog
alone answers bn_andes, bn_link, bn_munin, bn_pathfinder and
digitRecognition.

**PITA.** PITA ran 25 instances and answered 18 (7 time out). Its
answers differ from the reference's by at most 5.5e-09 relative (2.2e-16
absolute, bn_water) over 18 instances. Over the 25 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, PITA takes 4451 s in all and tddlog 2.26 s; over the 18 both
answer, 251 s against tddlog's 0.84. tddlog alone answers bn_alarm,
bn_andes, bn_hailfinder, bn_insurance, bn_munin and 2 more.

**aspmc.** aspmc ran 25 instances and answered 19 (6 time out). Its
answers differ from the reference's by at most 1.4e-07 relative (5.1e-08
absolute, bn_insurance) over 19 instances. Over the 25 instances given
to both that either answers, a run that does not answer counted at the
600 s limit, aspmc takes 4455 s in all and tddlog 2.26 s; over the 19
both answer, 855 s against tddlog's 0.29. tddlog alone answers bn_andes,
bn_link, bn_munin, bn_pathfinder, bn_pigs and 1 more.

**Scallop.** Scallop ran 25 instances and answered 9 (5 run out of
memory, 11 time out). Its answers differ from the reference's by at most
4.4e-16 relative (2.8e-17 absolute, bn_earthquake) over 9 instances.
Over the 25 instances given to both that either answers, a run that does
not answer counted at the 600 s limit, Scallop takes 9604 s in all and
tddlog 2.28 s; over the 9 both answer, 3.9 s against tddlog's 0.061.
tddlog alone answers bn_alarm, bn_andes, bn_asia, bn_child,
bn_hailfinder and 11 more.

**Storm sparse.** Storm sparse ran 25 instances and answered 22 (3 time
out). Its answers differ from the reference's by at most 1.0e+00
relative (5.8e-15 absolute, digitRecognition) over 22 instances. Over
the 25 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Storm sparse takes 2140 s in all and
tddlog 2.08 s; over the 22 both answer, 340 s against tddlog's 0.72.
tddlog alone answers bn_link, bn_munin and bn_pathfinder.

**Storm exact.** Storm exact ran 25 instances and answered 21 (4 time
out). Its answers differ from the reference's by at most 4.0e-10
relative (1.2e-10 absolute, bn_sachs) over 21 instances. Over the 25
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Storm exact takes 2468 s in all and tddlog
2.08 s; over the 21 both answer, 67.6 s against tddlog's 0.68. tddlog
alone answers bn_andes, bn_link, bn_munin and bn_pathfinder.

**Storm dd.** Storm dd ran 25 instances and answered 22 (3 time out).
Its answers differ from the reference's by at most 4.0e-10 relative
(1.2e-10 absolute, bn_sachs) over 22 instances. Over the 25 instances
given to both that either answers, a run that does not answer counted at
the 600 s limit, Storm dd takes 2340 s in all and tddlog 2.08 s; over
the 22 both answer, 540 s against tddlog's 0.95. tddlog alone answers
bn_munin, bn_pathfinder and digitRecognition.

**PRISM sparse.** PRISM sparse ran 25 instances and answered 22 (3 time
out). Its answers differ from the reference's by at most 1.0e+00
relative (5.8e-15 absolute, digitRecognition) over 22 instances. Over
the 25 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, PRISM sparse takes 2502 s in all and
tddlog 2.08 s; over the 22 both answer, 702 s against tddlog's 0.72.
tddlog alone answers bn_link, bn_munin and bn_pathfinder.

**PRISM hybrid.** PRISM hybrid ran 25 instances and answered 22 (3 time
out). Its answers differ from the reference's by at most 1.0e+00
relative (5.8e-15 absolute, digitRecognition) over 22 instances. Over
the 25 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, PRISM hybrid takes 2519 s in all and
tddlog 2.08 s; over the 22 both answer, 719 s against tddlog's 0.72.
tddlog alone answers bn_link, bn_munin and bn_pathfinder.

**Ace.** Ace ran 25 instances and answered 24 (1 fail). Its answers
differ from the reference's by at most 4.0e-10 relative (1.2e-10
absolute, bn_sachs) over 24 instances. Over the 25 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, Ace takes 691 s in all and tddlog 3.34 s; over the 24 both
answer, 91.4 s against tddlog's 2.97. tddlog alone answers
digitRecognition.

**Ace c2d.** Ace c2d ran 25 instances and answered 23 (1 fail, 1 time
out). Its answers differ from the reference's by at most 4.0e-10
relative (1.2e-10 absolute, bn_sachs) over 23 instances. Over the 25
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Ace c2d takes 1357 s in all and tddlog 3.34
s; over the 23 both answer, 157 s against tddlog's 2.15. tddlog alone
answers bn_link and digitRecognition.

**Ace log space.** Ace log space ran 1 instance and answered 1. Its
answers differ from the reference's by at most 3.8e-12 relative (1.3e-27
absolute, digitRecognition) over 1 instance. Over the 1 instance given
to both that either answers, Ace log space takes 11.7 s in all and
tddlog 0.37 s.

**Merlin.** Merlin ran 25 instances and answered 25. Its answers differ
from the reference's by at most 1.0e+00 relative (5.0e-07 absolute,
bn_water) over 25 instances. Over the 25 instances given to both that
either answers, Merlin takes 107 s in all and tddlog 3.34 s. Merlin is
faster on 1 (bn_pathfinder 0.071 s against 0.41).

**pyAgrum.** pyAgrum ran 25 instances and answered 25. Its answers
differ from the reference's by at most 1.0e+00 relative (1.7e-27
absolute, digitRecognition) over 25 instances. Over the 25 instances
given to both that either answers, pyAgrum takes 220 s in all and tddlog
3.34 s. pyAgrum is faster on 1 (bn_pathfinder 0.23 s against 0.41).

**pgmpy.** pgmpy ran 25 instances and answered 22 (3 time out). Its
answers differ from the reference's by at most 1.9e-10 relative (1.2e-10
absolute, bn_sachs) over 22 instances. Over the 25 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, pgmpy takes 1874 s in all and tddlog 3.34 s; over the 22 both
answer, 74.4 s against tddlog's 1.36. tddlog alone answers bn_link,
bn_munin and digitRecognition.

**libDAI.** libDAI ran 25 instances and answered 24 (1 time out). Its
answers differ from the reference's by at most 1.9e-10 relative (1.2e-10
absolute, bn_sachs) over 24 instances. Over the 25 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, libDAI takes 608 s in all and tddlog 3.34 s; over the 24 both
answer, 8.45 s against tddlog's 2.97. libDAI is faster on 1
(bn_pathfinder 0.068 s against 0.41). tddlog alone answers
digitRecognition.

**TensorInference.jl greedy.** TensorInference.jl greedy ran 25
instances and answered 22 (1 run out of memory, 2 time out). Its answers
differ from the reference's by at most 1.9e-10 relative (1.2e-10
absolute, bn_sachs) over 22 instances. Over the 25 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, TensorInference.jl greedy takes 2404 s in all and tddlog 3.34
s; over the 22 both answer, 604 s against tddlog's 1.36. tddlog alone
answers bn_link, bn_munin and digitRecognition.

**TensorInference.jl treesa.** TensorInference.jl treesa ran 25
instances and answered 24 (1 time out). Its answers differ from the
reference's by at most 1.9e-10 relative (1.2e-10 absolute, bn_sachs)
over 24 instances. Over the 25 instances given to both that either
answers, a run that does not answer counted at the 600 s limit,
TensorInference.jl treesa takes 1744 s in all and tddlog 3.34 s; over
the 24 both answer, 1144 s against tddlog's 2.97. tddlog alone answers
digitRecognition.

**Roulette.** Roulette ran 24 instances and answered 23 (1 time out).
Its answers differ from the reference's by at most 4.0e-10 relative
(1.2e-10 absolute, bn_sachs) over 23 instances. Over the 24 instances
given to both that either answers, a run that does not answer counted at
the 600 s limit, Roulette takes 691 s in all and tddlog 2.7 s; over the
23 both answer, 91.2 s against tddlog's 1.84. tddlog alone answers
bn_munin.

| instance | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Scallop | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Ace log space | Merlin | pyAgrum | pgmpy | libDAI | TI greedy | TI treesa | Roulette | Roulette process | ProbLog | ref | tddlog diff | Geni diff | noDice diff | Psi diff | Genfer diff | SPPL diff | Pluck diff | PITA diff | aspmc diff | Scallop diff | Storm sparse diff | Storm exact diff | Storm dd diff | PRISM sparse diff | PRISM hybrid diff | Ace diff | Ace c2d diff | Ace log space diff | Merlin diff | pyAgrum diff | pgmpy diff | libDAI diff | TI greedy diff | TI treesa diff | Roulette diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| bn_alarm | 0.013 | 0.357 | >600 | >600 | 0.325 | >600 | 8.63 | >600 | 148 | >600 | 2.32 | 2.52 | 6.99 | 18.6 | 18.9 | 1.19 | 1.27 |  | 0.005 | 0.123 | 1.64 | 0.005 | 26.1 | 30.5 | 0.58 | 1.25 | err | exact | 6.7e-16 / 1.2e-14 | 2.5e-07 / 3.5e-06 |  |  | 6.7e-16 / 9.2e-16 |  | 3.3e-16 / 7.6e-16 |  | 1.7e-15 / 2.9e-15 |  | 2.2e-16 / 8.7e-16 | 1.1e-16 / 5.8e-16 | 4.4e-16 / 1.1e-15 | 6.7e-16 / 9.6e-16 | 6.7e-16 / 9.6e-16 | 3.3e-16 / 5.1e-16 | 3.3e-16 / 4.7e-16 |  | 4.9e-07 / 1.7e-05 | 4.4e-13 / 2.3e-12 | 4.4e-13 / 2.3e-12 | 4.4e-13 / 2.3e-12 | 1.7e-16 / 8.7e-16 | 1.7e-16 / 8.7e-16 | 2.2e-16 / 1.2e-15 |
| bn_andes | 0.044 | 99.5 | 128 | >600 | 79 | >600 | err | >600 | >600 | >600 | 276 | >600 | 74.7 | 73.4 | 77 | 3.07 | 6.31 |  | 0.095 | 0.161 | 3.52 | 0.067 | 59.1 | 94 | 29 | 30.1 | >600 | exact | 2.2e-16 / 1.4e-15 | 1.4e-07 / 1.4e-06 | 8.8e-07 / 5.4e-06 |  | 2.4e-15 / 3.1e-15 |  |  |  |  |  | 2.3e-15 / 2.6e-15 |  | 2.2e-15 / 2.8e-15 | 2.4e-15 / 3.1e-15 | 2.4e-15 / 3.1e-15 | 5.6e-16 / 7.0e-16 | 4.4e-16 / 7.9e-16 |  | 4.6e-07 / 4.0e-06 | 4.7e-13 / 4.1e-12 | 4.7e-13 / 4.1e-12 | 4.7e-13 / 4.1e-12 | 1.1e-16 / 5.2e-16 | 2.2e-16 / 3.4e-16 | 1.1e-16 / 8.7e-16 |
| bn_asia | 0.009 | 0.021 | 0.064 | 6.13 | 0.015 | 0.784 | 5.09 | 0.877 | 2.7 | oom | 0.389 | 0.386 | 0.476 | 3.16 | 3.19 | 1.02 | 1.08 |  | 0.004 | 0.127 | 1.69 | 0.005 | 21.4 | 23.3 | 0.004 | 0.681 | 0.177 | exact | 1.1e-16 / 2.5e-16 | 2.9e-08 / 2.6e-07 | 4.0e-07 / 9.2e-07 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.0e-16 | 1.4e-17 / 1.3e-16 | 1.1e-16 / 2.5e-16 | 1.1e-16 / 2.0e-16 | 2.2e-16 / 2.5e-16 |  | 1.1e-16 / 2.5e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.0e-16 | 1.1e-16 / 2.5e-16 | 1.1e-16 / 2.5e-16 | 1.1e-16 / 2.5e-16 | 5.6e-17 / 1.3e-16 |  | 4.0e-07 / 9.2e-07 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 5.6e-17 / 1.3e-16 | 5.6e-17 / 1.3e-16 | 1.1e-16 / 2.5e-16 |
| bn_cancer | 0.007 | 0.021 | 0.065 | 0.847 | 0.015 | 0.753 | 4.59 | 1.41 | 2.69 | 0.314 | 0.414 | 0.386 | 0.41 | 3.16 | 3.17 | 1.1 | 1.15 |  | 0.004 | 0.13 | 1.73 | 0.004 | 18.6 | 21.3 | 0.004 | 0.688 | 0.168 | exact | 1.7e-16 / 5.5e-16 | 5.6e-17 / 1.8e-16 | 5.0e-07 / 1.6e-06 | 5.6e-17 / 1.8e-16 | 1.1e-16 / 1.4e-16 | 1.1e-16 / 1.8e-16 | 1.1e-16 / 2.7e-16 | 5.6e-17 / 1.8e-16 | 2.2e-16 / 3.2e-16 | 1.1e-16 / 1.6e-16 | 2.2e-16 / 3.2e-16 | 5.6e-17 / 1.8e-16 | 2.2e-16 / 3.2e-16 | 2.2e-16 / 3.2e-16 | 2.2e-16 / 3.2e-16 | 1.1e-16 / 1.6e-16 | 1.1e-16 / 1.6e-16 |  | 5.0e-07 / 1.6e-06 | 5.6e-17 / 1.8e-16 | 5.6e-17 / 1.8e-16 | 5.6e-17 / 1.8e-16 | 1.1e-16 / 3.7e-16 | 1.1e-16 / 3.7e-16 | 1.1e-16 / 3.7e-16 |
| bn_child | 0.015 | 0.175 | 0.28 | >600 | 0.116 | 10.6 | 8.04 | 1.34 | 62.7 | oom | 1.39 | 1.54 | 2.82 | 11.7 | 11.8 | 1.17 | 1.31 |  | 0.006 | 0.124 | 1.74 | 0.004 | 23.5 | 26.3 | 0.063 | 0.726 | 1.04 | exact | 5.0e-16 / 3.2e-15 | 7.5e-08 / 6.9e-07 | 6.2e-07 / 4.5e-06 |  | 2.2e-16 / 3.9e-16 | 2.2e-16 / 9.0e-16 | 2.2e-16 / 6.4e-16 | 3.1e-16 / 3.5e-15 | 6.7e-16 / 1.4e-15 |  | 3.9e-16 / 8.0e-16 | 1.1e-16 / 3.1e-16 | 2.2e-16 / 5.6e-16 | 2.2e-16 / 4.5e-16 | 2.2e-16 / 4.5e-16 | 2.2e-16 / 5.1e-16 | 4.4e-16 / 6.8e-16 |  | 5.0e-07 / 4.5e-06 | 5.0e-13 / 2.4e-12 | 5.0e-13 / 2.4e-12 | 5.0e-13 / 2.4e-12 | 2.2e-16 / 3.9e-16 | 2.2e-16 / 4.9e-16 | 1.1e-16 / 5.0e-16 |
| bn_earthquake | 0.011 | 0.021 | 0.066 | 0.997 | 0.016 | 0.782 | 4.53 | 0.901 | 2.54 | 0.156 | 0.382 | 0.379 | 0.436 | 3.34 | 3.41 | 1.15 | 1.18 |  | 0.004 | 0.135 | 1.75 | 0.007 | 18.3 | 20.7 | 0.004 | 0.68 | 0.168 | exact | 5.9e-17 / 2.8e-15 | 2.4e-08 / 3.7e-07 | 2.0e-07 / 9.6e-06 | 1.4e-17 / 4.9e-16 | 1.1e-16 / 3.3e-16 | 1.1e-16 / 1.3e-15 | 4.2e-17 / 1.5e-15 | 1.4e-17 / 4.9e-16 | 1.1e-16 / 4.9e-16 | 2.8e-17 / 4.4e-16 | 1.1e-16 / 3.3e-16 | 1.4e-17 / 4.9e-16 | 1.1e-16 / 4.4e-16 | 1.1e-16 / 4.9e-16 | 1.1e-16 / 4.9e-16 | 1.1e-16 / 3.3e-16 | 1.1e-16 / 3.3e-16 |  | 2.0e-07 / 9.6e-06 | 1.4e-17 / 4.9e-16 | 1.4e-17 / 4.9e-16 | 1.4e-17 / 4.9e-16 | 1.4e-17 / 6.6e-16 | 6.9e-18 / 3.3e-16 | 4.2e-17 / 1.6e-15 |
| bn_hailfinder | 0.037 | 19.4 | >600 | >600 | 134 | >600 | 58.4 | >600 | 273 | >600 | 5.79 | 6.45 | 18 | 26.3 | 23.8 | 1.51 | 1.7 |  | 0.007 | 0.14 | 1.59 | 0.007 | 27.3 | 33.1 | 2.84 | 3.55 | >600 | exact | 7.2e-16 / 9.0e-15 | 1.9e-07 / 1.2e-06 |  |  | 2.2e-16 / 7.2e-16 |  | 1.1e-16 / 5.9e-16 |  | 3.2e-15 / 6.8e-15 |  | 4.4e-16 / 1.0e-15 | 2.2e-16 / 5.4e-16 | 1.7e-16 / 6.6e-16 | 2.8e-16 / 6.6e-16 | 2.8e-16 / 6.6e-16 | 2.2e-16 / 6.0e-16 | 2.2e-16 / 5.0e-16 |  | 4.8e-07 / 3.7e-06 | 4.9e-13 / 1.6e-12 | 4.9e-13 / 1.6e-12 | 4.9e-13 / 1.6e-12 | 1.7e-16 / 5.4e-16 | 2.8e-16 / 6.3e-16 | 2.2e-16 / 5.4e-16 |
| bn_hepar2 | 0.031 | 1 | 1.98 | >600 | 1.07 | >600 | 8.95 | 86.2 | 177 | >600 | 8.36 | 9.04 | 23.4 | 68.8 | 68.6 | 1.5 | 1.71 |  | 0.008 | 0.134 | 1.9 | 0.009 | 32.4 | 41.9 | 0.749 | 1.43 | 23.2 | exact | 1.1e-15 / 5.1e-14 | 1.2e-07 / 1.8e-05 | 9.7e-07 / 2.4e-04 |  | 2.2e-16 / 1.1e-15 |  | 3.3e-16 / 4.0e-15 | 2.2e-16 / 7.2e-15 | 5.9e-15 / 6.6e-15 |  | 1.8e-15 / 1.9e-15 | 1.1e-16 / 1.0e-15 | 4.4e-16 / 1.1e-15 | 3.3e-16 / 1.0e-15 | 3.3e-16 / 1.0e-15 | 3.3e-16 / 1.0e-15 | 4.4e-16 / 7.7e-16 |  | 4.9e-07 / 1.2e-04 | 4.8e-13 / 4.4e-12 | 4.8e-13 / 4.4e-12 | 4.8e-13 / 4.4e-12 | 2.2e-16 / 8.1e-16 | 2.2e-16 / 8.1e-16 | 1.1e-16 / 3.8e-15 |
| bn_insurance | 0.022 | 13.8 | 7.97 | >600 | 30.8 | >600 | 21.7 | >600 | 40.2 | >600 | 3.65 | 4.82 | 15.2 | 11.8 | 12.3 | 1.69 | 1.9 |  | 0.007 | 0.131 | 1.74 | 0.008 | 30.8 | 34.9 | 2.38 | 3.11 | >600 | exact | 1.5e-11 / 2.7e-11 | 1.2e-07 / 4.9e-06 | 7.1e-07 / 4.5e-05 |  | 1.5e-11 / 2.7e-11 |  | 1.5e-11 / 2.7e-11 |  | 5.1e-08 / 1.4e-07 |  | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 |  | 4.9e-07 / 4.5e-05 | 1.3e-11 / 2.6e-11 | 1.3e-11 / 2.6e-11 | 1.3e-11 / 2.6e-11 | 1.3e-11 / 2.6e-11 | 1.3e-11 / 2.6e-11 | 1.5e-11 / 2.7e-11 |
| bn_link | 0.592 | >600 | 139 | >600 | err | >600 | err | 101 | >600 | >600 | >600 | >600 | 282 | >600 | >600 | 51.4 | >600 |  | 33 | 85.6 | >600 | 4.08 | >600 | 175 | 9.26 | 10.1 | >600 | alea | 6.0e-14 / 4.8e-13 |  | 5.3e-07 / 2.9e-03 |  |  |  |  | 9.4e-16 / 7.1e-12 |  |  |  |  | 6.7e-16 / 2.8e-15 |  |  | 1.6e-13 / 1.8e-13 |  |  | 4.7e-07 / 2.8e-03 | 5.0e-13 / 5.0e-13 |  | 5.0e-13 / 5.0e-13 |  | 3.3e-16 / 5.4e-15 | 4.4e-16 / 3.0e-15 |
| bn_munin | 0.492 | >600 | >600 | >600 | >600 | >600 | err | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | 7.26 | 107 |  | 2.37 | 1.04 | >600 | 3.37 | oom | 194 | >600 | >600 | >600 | exact | 3.6e-15 / 1.6e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 6.7e-16 / 2.4e-13 | 5.4e-15 / 2.4e-13 |  | 5.0e-07 / 3.0e-02 | 5.0e-13 / 4.7e-12 |  | 5.0e-13 / 4.7e-12 |  | 4.4e-16 / 2.5e-13 |  |
| bn_pathfinder | 0.271 | >600 | 17.1 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | 2.3 | 4.12 |  | 0.071 | 0.231 | 4.16 | 0.068 | 31.8 | 43.8 | 20.3 | 21.1 | >600 | exact | 1.1e-15 / 2.2e-12 |  | 9.4e-07 / 5.4e-03 |  |  |  |  |  |  |  |  |  |  |  |  | 4.4e-16 / 1.6e-14 | 5.6e-16 / 1.7e-14 |  | 5.0e-07 / 2.6e-03 | 4.9e-13 / 4.1e-12 | 4.9e-13 / 4.1e-12 | 4.9e-13 / 4.1e-12 | 4.4e-16 / 1.6e-14 | 2.8e-16 / 1.6e-14 | 3.3e-16 / 1.7e-14 |
| bn_pigs | 0.057 | 4.4 | 91.9 | >600 | 3.02 | >600 | 23.6 | 7.95 | >600 | >600 | 29.4 | 30 | 91.8 | 237 | 238 | 3.49 | 13.5 |  | 0.145 | 0.212 | 34.9 | 0.135 | 60.7 | 113 | 25.4 | 26.1 | >600 | exact | 1.1e-16 / 2.4e-16 | 5.0e-08 / 1.0e-07 | 7.5e-07 / 3.0e-06 |  | 1.1e-16 / 2.4e-16 |  | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 |  |  | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 |  | 5.0e-07 / 1.9e-06 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 4.4e-16 | 1.1e-16 / 4.3e-16 | 1.1e-16 / 2.4e-16 |
| bn_sachs | 0.011 | 0.086 | 0.15 | >600 | 0.054 | 2.96 | 7.84 | 1.13 | 3.97 | oom | 0.796 | 0.806 | 1.48 | 6.65 | 6.42 | 1.17 | 1.23 |  | 0.004 | 0.13 | 1.7 | 0.004 | 24.5 | 25.4 | 0.037 | 0.723 | 0.324 | exact | 1.2e-10 / 4.0e-10 | 8.4e-08 / 1.1e-06 | 8.1e-07 / 1.0e-05 |  | 1.2e-10 / 4.0e-10 | 1.2e-10 / 4.0e-10 | 1.2e-10 / 4.0e-10 | 1.9e-10 / 2.4e-09 | 5.6e-17 / 3.6e-16 |  | 1.2e-10 / 4.0e-10 | 1.2e-10 / 4.0e-10 | 1.2e-10 / 4.0e-10 | 1.2e-10 / 4.0e-10 | 1.2e-10 / 4.0e-10 | 1.2e-10 / 4.0e-10 | 1.2e-10 / 4.0e-10 |  | 4.6e-07 / 3.2e-06 | 1.2e-10 / 1.9e-10 | 1.2e-10 / 1.9e-10 | 1.2e-10 / 1.9e-10 | 1.2e-10 / 1.9e-10 | 1.2e-10 / 1.9e-10 | 1.2e-10 / 4.0e-10 |
| bn_survey | 0.008 | 0.013 | 0.05 | 1.56 | 0.009 | 0.872 | 6.12 | 0.908 | 2.82 | oom | 0.202 | 0.205 | 0.302 | 1.62 | 1.66 | 1.06 | 1.08 |  | 0.006 | 0.126 | 1.68 | 0.004 | 21 | 22.6 | 0.005 | 0.68 | 0.182 | exact | 4.7e-16 / 3.0e-15 | 3.2e-08 / 7.8e-08 | 7.5e-07 / 2.7e-06 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.0e-16 | 1.1e-16 / 3.5e-16 | 1.1e-16 / 4.0e-16 | 1.4e-16 / 8.8e-16 | 1.1e-16 / 7.1e-16 |  | 1.1e-16 / 3.5e-16 | 0.0e+00 / 0.0e+00 | 2.8e-17 / 1.8e-16 | 2.8e-17 / 1.8e-16 | 2.8e-17 / 1.8e-16 | 2.8e-17 / 1.8e-16 | 2.8e-17 / 1.8e-16 |  | 2.5e-07 / 1.4e-06 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 5.6e-17 / 2.0e-16 | 1.1e-16 / 2.0e-16 | 5.6e-17 / 3.5e-16 |
| bn_water | 0.029 | wrong | 2.4 | >600 | 6.72 | >600 | 9.2 | 43.1 | 18.5 | >600 | 4.34 | 4.44 | 14.8 | 16.5 | 16.2 | 1.98 | 2.5 |  | 0.392 | 0.305 | 1.93 | 0.318 | 36.8 | 43.3 | 0.358 | 1.03 | >600 | exact | 4.4e-16 / 1.4e-10 | 1.2e-07 / 1.0e+00 | 7.5e-07 / 1.0e+00 |  | 1.1e-16 / 1.4e-10 |  | 1.1e-16 / 1.4e-10 | 2.2e-16 / 5.5e-09 | 3.3e-16 / 1.4e-10 |  | 4.4e-16 / 1.4e-10 | 1.1e-16 / 1.4e-10 | 1.1e-16 / 1.4e-10 | 4.4e-16 / 1.4e-10 | 4.4e-16 / 1.4e-10 | 4.4e-16 / 1.4e-10 | 3.3e-16 / 1.4e-10 |  | 5.0e-07 / 1.0e+00 | 4.6e-13 / 1.4e-10 | 4.6e-13 / 1.4e-10 | 4.6e-13 / 1.4e-10 | 2.2e-16 / 1.4e-10 | 2.2e-16 / 1.4e-10 | 1.1e-16 / 1.4e-10 |
| bn_win95pts | 0.014 | 0.254 | 0.376 | >600 | 0.269 | >600 | 10.9 | 3.52 | 104 | >600 | 3.58 | 3.32 | 5.84 | 27 | 28 | 1.35 | 1.41 |  | 0.007 | 0.13 | 1.73 | 0.008 | 36.2 | 45.2 | 0.089 | 0.753 | 197 | exact | 2.2e-16 / 3.5e-14 | 1.3e-07 / 1.3e-05 | 5.3e-07 / 4.6e-06 |  | 2.2e-16 / 3.8e-16 |  | 2.2e-16 / 3.9e-16 | 2.2e-16 / 1.7e-14 | 4.4e-16 / 8.1e-15 |  | 5.6e-16 / 9.7e-16 | 1.1e-16 / 2.0e-16 | 3.3e-16 / 4.1e-16 | 2.2e-16 / 3.8e-16 | 2.2e-16 / 3.8e-16 | 5.6e-16 / 9.7e-16 | 4.4e-16 / 5.0e-16 |  | 4.7e-07 / 4.0e-06 | 4.9e-13 / 3.3e-12 | 4.9e-13 / 3.3e-12 | 4.9e-13 / 3.3e-12 | 1.1e-16 / 5.3e-16 | 1.1e-16 / 4.0e-16 | 2.2e-16 / 4.1e-16 |
| dice_alarm | 0.007 | 0.005 | 0.01 | 0.073 | 0.003 | 0.573 | 3.56 | 0.315 | 2.45 | 0.056 | 0.156 | 0.153 | 0.192 | 1.27 | 1.27 | 1.2 | 1.25 |  | 0.054 | 0.169 | 1.56 | 0.055 | 20.6 | 24.3 | 0.003 | 0.678 | 0.072 | alea | 5.2e-18 / 1.7e-15 | 1.8e-08 / 6.0e-06 | 4.5e-07 / 1.5e-04 | 8.7e-19 / 2.9e-16 | 8.7e-19 / 2.9e-16 | 3.5e-17 / 1.2e-14 | 1.3e-18 / 4.3e-16 | 2.2e-18 / 7.2e-16 | 8.7e-19 / 2.9e-16 | 1.3e-18 / 4.3e-16 | 1.7e-18 / 5.8e-16 | 8.7e-19 / 2.9e-16 | 1.7e-18 / 5.8e-16 | 1.3e-18 / 4.3e-16 | 1.3e-18 / 4.3e-16 | 1.7e-18 / 5.8e-16 | 1.3e-18 / 4.3e-16 |  | 4.5e-07 / 1.5e-04 | 4.5e-15 / 1.5e-12 | 4.5e-15 / 1.5e-12 | 4.5e-15 / 1.5e-12 | 1.7e-18 / 5.8e-16 | 1.7e-18 / 5.8e-16 | 1.3e-18 / 4.3e-16 |
| dice_digitRecognition | 0.352 | err | 17.7 | >600 | 0.102 | >600 | oom | >600 | >600 | oom | 2.16 | 2.25 | >600 | 183 | 197 | err | err | 11.7 | 70.8 | 130 | >600 | >600 | >600 | >600 |  |  | 17.9 | alea | 5.8e-15 / 7.5e-12 |  | 5.7e-15 / 1.0e+00 |  | 5.8e-15 / 1.9e+02 |  |  |  |  |  | 5.8e-15 / 1.0e+00 | 5.8e-15 / 7.5e-12 |  | 5.8e-15 / 1.0e+00 | 5.8e-15 / 1.0e+00 |  |  | 1.3e-27 / 3.8e-12 | 5.7e-15 / 1.0e+00 | 1.7e-27 / 1.0e+00 |  |  |  |  |  |
| dice_evidence1 | 0.007 | 0.005 | 0.01 | 0.021 | 0.002 | 0.559 | 3.37 | 0.315 | 2.45 | 0.042 | 0.157 | 0.15 | 0.161 | 1.28 | 1.27 | 1.08 | 1.18 |  | 0.044 | 0.151 | 1.55 | 0.045 | 18.4 | 20.2 | 0.002 | 0.676 | 0.07 | alea | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 3.3e-07 / 1.0e-06 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 0.0e+00 / 0.0e+00 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 |  | 3.3e-07 / 1.0e-06 | 3.3e-13 / 1.0e-12 | 3.3e-13 / 1.0e-12 | 3.3e-13 / 1.0e-12 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 |
| dice_evidence2 | 0.006 | 0.005 | 0.009 | 0.021 | 0.003 | 0.556 | 3.49 | 0.319 | 2.46 | 0.038 | 0.149 | 0.16 | 0.154 | 1.26 | 1.27 | 1.11 | 1.19 |  | 0.046 | 0.154 | 1.57 | 0.047 | 19.3 | 21.5 | 0.003 | 0.681 | 0.069 | alea | 1.1e-16 / 1.7e-16 | 0.0e+00 / 0.0e+00 | 3.3e-07 / 5.0e-07 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.7e-16 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 3.3e-07 / 5.0e-07 | 3.3e-13 / 5.0e-13 | 3.3e-13 / 5.0e-13 | 3.3e-13 / 5.0e-13 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| dice_grass | 0.006 | 0.005 | 0.011 | 0.056 | 0.006 | 0.587 | 3.56 | 0.325 | 2.45 | 0.045 | 0.148 | 0.148 | 0.197 | 1.27 | 1.28 | 1.18 | 1.21 |  | 0.047 | 0.158 | 1.58 | 0.047 | 20.5 | 23.9 | 0.003 | 0.679 | 0.073 | alea | 1.1e-16 / 1.6e-16 | 0.0e+00 / 0.0e+00 | 6.8e-07 / 9.6e-07 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.6e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.6e-16 | 2.2e-16 / 3.1e-16 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.6e-16 | 1.1e-16 / 1.6e-16 | 1.1e-16 / 1.6e-16 | 1.1e-16 / 1.6e-16 | 0.0e+00 / 0.0e+00 |  | 3.2e-07 / 4.6e-07 | 3.8e-13 / 5.3e-13 | 3.8e-13 / 5.3e-13 | 3.8e-13 / 5.3e-13 | 1.1e-16 / 1.6e-16 | 1.1e-16 / 1.6e-16 | 0.0e+00 / 0.0e+00 |
| dice_murderMystery | 0.004 | 0.005 | 0.011 | err | 0.002 | 0.565 | 3.44 | 0.314 | 2.45 | 0.045 | 0.152 | 0.152 | 0.159 | 1.27 | 1.27 | 1.12 | 1.14 |  | 0.045 | 0.154 | 1.57 | 0.046 | 19.5 | 21.3 | 0.002 | 0.681 | 0.069 | alea | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 2.2e-07 / 1.4e-05 |  | 3.5e-18 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 3.5e-18 / 2.2e-16 | 3.5e-18 / 2.2e-16 | 3.5e-18 / 2.2e-16 | 3.5e-18 / 2.2e-16 | 3.5e-18 / 2.2e-16 | 3.5e-18 / 2.2e-16 | 3.5e-18 / 2.2e-16 | 3.5e-18 / 2.2e-16 | 3.5e-18 / 2.2e-16 | 3.5e-18 / 2.2e-16 | 3.5e-18 / 2.2e-16 |  | 2.2e-07 / 1.4e-05 | 6.0e-15 / 3.8e-13 | 6.0e-15 / 3.8e-13 | 6.0e-15 / 3.8e-13 | 3.5e-18 / 2.2e-16 | 3.5e-18 / 2.2e-16 | 3.5e-18 / 2.2e-16 |
| dice_noisyOr | 0.008 | 0.005 | 0.012 | 1.28 | 0.016 | 0.613 | 3.44 | 0.321 | 2.46 | 3.17 | 0.146 | 0.145 | 0.218 | 1.27 | 1.27 | 1.15 | 1.24 |  | 0.048 | 0.156 | 1.58 | 0.048 | 20.9 | 26 | 0.003 | 0.674 | 0.077 | alea | 0.0e+00 / 0.0e+00 | 5.0e-08 / 6.1e-08 | 7.5e-07 / 9.2e-07 | 1.1e-16 / 1.4e-16 | 1.1e-16 / 1.4e-16 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.4e-16 | 1.1e-16 / 1.4e-16 | 2.2e-16 / 2.7e-16 | 1.1e-16 / 1.4e-16 | 1.1e-16 / 1.4e-16 | 1.1e-16 / 1.4e-16 | 1.1e-16 / 1.4e-16 | 1.1e-16 / 1.4e-16 | 2.2e-16 / 2.7e-16 | 1.1e-16 / 1.4e-16 |  | 2.5e-07 / 3.1e-07 | 1.1e-16 / 1.4e-16 | 1.1e-16 / 1.4e-16 | 1.1e-16 / 1.4e-16 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.4e-16 |
| dice_twocoins | 0.005 | 0.005 | 0.008 | 0.022 | 0.002 | 0.561 | 3.38 | 0.318 | 2.46 | 0.043 | 0.157 | 0.155 | 0.159 | 1.26 | 1.27 | 1.11 | 1.16 |  | 0.057 | 0.179 | 1.58 | 0.058 | 16.6 | 18.4 | 0.002 | 0.664 | 0.119 | alea | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 3.3e-07 / 1.0e-06 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 0.0e+00 / 0.0e+00 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 |  | 3.3e-07 / 1.0e-06 | 3.3e-13 / 1.0e-12 | 3.3e-13 / 1.0e-12 | 3.3e-13 / 1.0e-12 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 | 5.6e-17 / 1.7e-16 |

Seconds per system, the median over the repetitions; a failure is `>600`
(timeout), `oom`, `err` or `n/a`; each system's last columns are its
largest absolute and relative difference from the reference named in
`ref`.

### On the Dice paper's loop programs

| family | instances | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Ace log space | Merlin | pyAgrum | pgmpy | libDAI | TensorInference.jl greedy | TensorInference.jl treesa | Roulette |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| caesar-iter | 7 | 7 | 0 of 7 | 2 of 3 | 0 of 7 | 2 of 7 | 1 of 2 | 2 of 4 | 0 of 2 | 1 of 3 | 2 of 7 | 6 of 7 | 2 of 6 | 2 of 6 | 2 of 6 | 1 of 4 | 1 of 4 | 3 of 4 | 2 of 3 | 2 of 3 | 2 of 3 | 1 of 2 | 1 of 2 | 1 of 2 | 2 of 4 |
| diamond | 8 | 8 | 1 of 8 | 3 of 4 | 0 of 8 | 3 of 8 | 3 of 4 | 3 of 5 | 2 of 3 | 4 of 5 | 8 of 8 | 5 of 6 | 5 of 6 | 5 of 6 | 5 of 6 | 5 of 7 | 2 of 7 |  | 3 of 4 | 3 of 4 | 4 of 5 | 4 of 5 | 2 of 3 | 2 of 3 | 3 of 4 |
| ladder | 8 | 7 | 3 of 7 | 0 of 4 | 0 of 8 | 2 of 8 | 3 of 4 | 2 of 4 | 1 of 2 | 3 of 5 | 8 of 8 | 5 of 6 | 3 of 4 | 4 of 5 | 4 of 5 | 2 of 8 | 2 of 8 |  | 2 of 3 | 3 of 4 | 3 of 4 | 3 of 4 | 2 of 3 | 1 of 2 | 3 of 4 |
| pmc1 | 7 | 7 | 3 of 4 | 0 of 4 | 0 of 7 | 0 of 1 | 0 of 1 | 7 of 7 | 7 of 7 | 4 of 5 | 2 of 3 | 2 of 3 | 7 of 7 | 6 of 7 | 6 of 7 | 4 of 5 | 4 of 5 |  | 3 of 4 | 3 of 4 | 3 of 4 | 3 of 4 | 3 of 7 | 3 of 5 | 7 of 7 |
| pmc2 | 7 | 7 | 2 of 3 | 0 of 3 | 0 of 7 | 0 of 1 | 0 of 1 | 7 of 7 | 7 of 7 | 3 of 5 | 7 of 7 | 7 of 7 | 7 of 7 | 7 of 7 | 7 of 7 | 7 of 7 | 7 of 7 |  | 7 of 7 | 7 of 7 | 7 of 7 | 7 of 7 | 7 of 7 | 7 of 7 | 7 of 7 |
| simpleCaesar | 7 | 6 | 0 of 4 | 1 of 3 | 0 of 7 | 1 of 2 | 0 of 1 | 1 of 3 | 0 of 1 | 1 of 3 | 1 of 7 | 5 of 6 | 1 of 7 | 1 of 7 | 1 of 7 | 1 of 5 | 1 of 5 | 4 of 5 | 1 of 2 | 1 of 4 | 1 of 2 | 1 of 2 | 1 of 4 | 0 of 1 | 1 of 4 |

**Geni.** Geni ran 33 instances and answered 9 (9 fail, 3 run out of
memory, 3 time out, 9 answer wrongly). Its answers differ from the
reference's by at most 4.4e-05 relative (8.9e-07 absolute, ladder_k6400)
over 9 instances. Geni's answer is wrong, zero or underflowed on
diamond_k102400, diamond_k1600, diamond_k25600, diamond_k400,
diamond_k409600 and 4 more. Over the 33 instances given to both that
either answers, a run that does not answer counted at the 600 s limit,
Geni takes 14404 s in all and tddlog 494 s; over the 9 both answer, 4.48
s against tddlog's 2.46. Geni is faster on 1 (pmc1_k20 0.005 s against
0.26). tddlog alone answers caesar-iter_k100, caesar-iter_k1000,
caesar-iter_k10000, caesar-iter_k100000, caesar-iter_k3000 and 19 more.

**noDice.** noDice ran 21 instances and answered 6 (4 run out of memory,
10 time out, 1 answer wrongly). Its answers differ from the reference's
by at most 1.0e+00 relative (7.9e-31 absolute, caesar-iter_k100) over 6
instances. noDice's answer is wrong, zero or underflowed on
simpleCaesar_k300. Over the 21 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, noDice
takes 9027 s in all and tddlog 10.9 s; over the 6 both answer, 27.4 s
against tddlog's 0.96. tddlog alone answers caesar-iter_k3000,
diamond_k6400, ladder_k100, ladder_k1600, ladder_k400 and 10 more.

**Psi.** Psi ran 44 instances and answered 0 (44 fail). Over the 42
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Psi takes 25200 s in all and tddlog 1007 s.
tddlog alone answers caesar-iter_k100, caesar-iter_k1000,
caesar-iter_k10000, caesar-iter_k100000, caesar-iter_k3000 and 37 more.

**Genfer.** Genfer ran 27 instances and answered 8 (16 fail, 3 time
out). Its answers differ from the reference's by at most 1.0e+00
relative (9.3e-302 absolute, caesar-iter_k1000) over 8 instances. Over
the 26 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Genfer takes 11183 s in all and
tddlog 465 s; over the 8 both answer, 383 s against tddlog's 1.21.
Genfer is faster on 1 (caesar-iter_k1000 0.4 s against 0.41). tddlog
alone answers caesar-iter_k10000, caesar-iter_k100000,
caesar-iter_k3000, caesar-iter_k30000, caesar-iter_k300000 and 13 more.

**SPPL.** SPPL ran 13 instances and answered 7 (6 time out). Its answers
differ from the reference's by at most 2.7e-13 relative (2.2e-16
absolute, caesar-iter_k100) over 7 instances. Over the 13 instances
given to both that either answers, a run that does not answer counted at
the 600 s limit, SPPL takes 4051 s in all and tddlog 3.69 s; over the 7
both answer, 451 s against tddlog's 0.92. tddlog alone answers
caesar-iter_k1000, diamond_k6400, ladder_k6400, pmc1_k10, pmc2_k10 and 1
more.

**Pluck.** Pluck ran 30 instances and answered 22 (8 fail). Its answers
differ from the reference's by at most 1.0e+00 relative (9.3e-302
absolute, caesar-iter_k1000) over 22 instances. Over the 30 instances
given to both that either answers, a run that does not answer counted at
the 600 s limit, Pluck takes 5341 s in all and tddlog 364 s; over the 22
both answer, 541 s against tddlog's 326. Pluck is faster on 2 (pmc1_k300
26 s against 94.2 and pmc1_k600 54.5 s against 184). tddlog alone
answers caesar-iter_k10000, caesar-iter_k3000, diamond_k25600,
diamond_k6400, ladder_k1600 and 3 more.

**PITA.** PITA ran 22 instances and answered 17 (4 time out, 1 answer
wrongly). Its answers differ from the reference's by at most 1.9e-14
relative (1.9e-14 absolute, diamond_k400) over 17 instances. PITA's
answer is wrong, zero or underflowed on caesar-iter_k100. Over the 22
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, PITA takes 3473 s in all and tddlog 326 s;
over the 17 both answer, 473 s against tddlog's 325. PITA is faster on 3
(pmc1_k150 4.03 s against 18.3, pmc1_k300 11.2 s against 94.2 and
pmc1_k600 36.3 s against 184). tddlog alone answers caesar-iter_k100,
caesar-iter_k1000, diamond_k1600, ladder_k400 and simpleCaesar_k100.

**aspmc.** aspmc ran 26 instances and answered 16 (1 run out of memory,
9 time out). Its answers differ from the reference's by at most 5.2e-12
relative (2.7e-42 absolute, simpleCaesar_k100) over 16 instances. Over
the 26 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, aspmc takes 6415 s in all and tddlog
64.3 s; over the 16 both answer, 415 s against tddlog's 3.29. tddlog
alone answers caesar-iter_k1000, caesar-iter_k10000, diamond_k25600,
ladder_k25600, ladder_k6400 and 5 more.

**Storm sparse.** Storm sparse ran 40 instances and answered 28 (11
fail, 1 time out). Its answers differ from the reference's by at most
1.0e+00 relative (9.3e-302 absolute, caesar-iter_k1000) over 27
instances. Over the 39 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, Storm sparse takes
6831 s in all and tddlog 1312 s; over the 27 both answer, 193 s against
tddlog's 362. Storm sparse is faster on 12 (diamond_k6400 0.21 s against
0.35, simpleCaesar_k100 0.2 s against 0.35, caesar-iter_k1000 0.17 s
against 0.41 and ladder_k1600 0.33 s against 0.69, the largest
differences). Only Storm sparse answers ladder_k1638400. tddlog alone
answers caesar-iter_k10000, caesar-iter_k100000, caesar-iter_k3000,
caesar-iter_k30000, caesar-iter_k300000 and 6 more.

**Storm exact.** Storm exact ran 35 instances and answered 30 (2 fail, 3
time out). Its answers differ from the reference's by at most 1.9e-10
relative (1.0e-302 absolute, simpleCaesar_k1000) over 30 instances. Over
the 35 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Storm exact takes 3866 s in all and
tddlog 425 s; over the 30 both answer, 866 s against tddlog's 158. Storm
exact is faster on 9 (simpleCaesar_k100 0.32 s against 0.35,
caesar-iter_k1000 0.21 s against 0.41, simpleCaesar_k300 0.66 s against
0.86 and caesar-iter_k3000 0.37 s against 0.91, the largest
differences). tddlog alone answers caesar-iter_k300000, diamond_k102400,
ladder_k102400, pmc1_k40 and simpleCaesar_k30000.

**Storm dd.** Storm dd ran 37 instances and answered 25 (8 fail, 4 time
out). Its answers differ from the reference's by at most 1.0e+00
relative (9.3e-302 absolute, caesar-iter_k1000) over 25 instances. Over
the 36 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Storm dd takes 7226 s in all and
tddlog 598 s; over the 25 both answer, 626 s against tddlog's 329. Storm
dd is faster on 3 (pmc1_k150 17.3 s against 18.3, pmc1_k300 22.1 s
against 94.2 and pmc1_k600 29.9 s against 184). tddlog alone answers
caesar-iter_k10000, caesar-iter_k100000, caesar-iter_k3000,
caesar-iter_k30000, diamond_k102400 and 6 more.

**PRISM sparse.** PRISM sparse ran 38 instances and answered 25 (8 fail,
1 run out of memory, 4 time out). Its answers differ from the
reference's by at most 1.0e+00 relative (9.3e-302 absolute,
caesar-iter_k1000) over 25 instances. Over the 37 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, PRISM sparse takes 8324 s in all and tddlog 605 s; over the 25
both answer, 1124 s against tddlog's 147. tddlog alone answers
caesar-iter_k10000, caesar-iter_k100000, caesar-iter_k3000,
caesar-iter_k30000, diamond_k102400 and 7 more.

**PRISM hybrid.** PRISM hybrid ran 38 instances and answered 25 (8 fail,
1 run out of memory, 4 time out). Its answers differ from the
reference's by at most 1.0e+00 relative (9.3e-302 absolute,
caesar-iter_k1000) over 25 instances. Over the 37 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, PRISM hybrid takes 8345 s in all and tddlog 605 s; over the 25
both answer, 1145 s against tddlog's 147. tddlog alone answers
caesar-iter_k10000, caesar-iter_k100000, caesar-iter_k3000,
caesar-iter_k30000, diamond_k102400 and 7 more.

**Ace.** Ace ran 36 instances and answered 20 (12 fail, 4 time out). Its
answers differ from the reference's by at most 5.2e-12 relative (2.7e-42
absolute, simpleCaesar_k100) over 19 instances. Over the 35 instances
given to both that either answers, a run that does not answer counted at
the 600 s limit, Ace takes 9140 s in all and tddlog 320 s; over the 20
both answer, 140 s against tddlog's 32. Ace is faster on 2 (pmc2_k800
6.44 s against 7.01 and pmc2_k1600 11.9 s against 15.2). tddlog alone
answers caesar-iter_k1000, caesar-iter_k10000, caesar-iter_k3000,
diamond_k102400, diamond_k409600 and 10 more.

**Ace c2d.** Ace c2d ran 36 instances and answered 17 (12 fail, 4 time
out, 3 answer wrongly). Its answers differ from the reference's by at
most 5.2e-12 relative (2.7e-42 absolute, simpleCaesar_k100) over 17
instances. Ace c2d's answer is wrong, zero or underflowed on
diamond_k1600, diamond_k25600 and diamond_k6400. Over the 35 instances
given to both that either answers, a run that does not answer counted at
the 600 s limit, Ace c2d takes 10866 s in all and tddlog 320 s; over the
17 both answer, 66.1 s against tddlog's 29.3. Ace c2d is faster on 2
(pmc2_k800 6.61 s against 7.01 and pmc2_k1600 11.6 s against 15.2).
tddlog alone answers caesar-iter_k1000, caesar-iter_k10000,
caesar-iter_k3000, diamond_k102400, diamond_k1600 and 13 more.

**Ace log space.** Ace log space ran 9 instances and answered 7 (2 time
out). Its answers differ from the reference's by at most 2.8e-10
relative (1.5e-302 absolute, simpleCaesar_k1000) over 6 instances. Over
the 9 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Ace log space takes 1506 s in all and
tddlog 81.2 s; over the 7 both answer, 306 s against tddlog's 31.4.
tddlog alone answers caesar-iter_k10000 and simpleCaesar_k10000.

**Merlin.** Merlin ran 23 instances and answered 18 (4 run out of
memory, 1 time out). Its answers differ from the reference's by at most
1.0e+00 relative (7.9e-31 absolute, caesar-iter_k100) over 18 instances.
Over the 23 instances given to both that either answers, a run that does
not answer counted at the 600 s limit, Merlin takes 3392 s in all and
tddlog 32.6 s; over the 18 both answer, 392 s against tddlog's 29.1.
Merlin is faster on 5 (pmc2_k51 0.24 s against 0.25, pmc2_k200 1.26 s
against 1.28, pmc2_k400 2.71 s against 3.33 and pmc2_k800 5.48 s against
7.01, the largest differences). tddlog alone answers caesar-iter_k3000,
diamond_k6400, ladder_k1600, pmc1_k75 and simpleCaesar_k300.

**pyAgrum.** pyAgrum ran 26 instances and answered 19 (1 fail, 1 run out
of memory, 5 time out). Its answers differ from the reference's by at
most 1.0e+00 relative (9.3e-302 absolute, caesar-iter_k1000) over 19
instances. Over the 26 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, pyAgrum takes 4502
s in all and tddlog 62.9 s; over the 19 both answer, 302 s against
tddlog's 29.8. pyAgrum is faster on 3 (pmc2_k400 2.93 s against 3.33,
pmc2_k800 5.41 s against 7.01 and pmc2_k1600 10.9 s against 15.2).
tddlog alone answers caesar-iter_k3000, diamond_k6400, ladder_k6400,
pmc1_k75, simpleCaesar_k1000 and 2 more.

**pgmpy.** pgmpy ran 25 instances and answered 20 (1 run out of memory,
4 time out). Its answers differ from the reference's by at most 1.0e+00
relative (9.3e-302 absolute, caesar-iter_k1000) over 20 instances. Over
the 25 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, pgmpy takes 4013 s in all and tddlog
36.3 s; over the 20 both answer, 1013 s against tddlog's 30.2. pgmpy is
faster on 1 (pmc2_k1600 12.5 s against 15.2). tddlog alone answers
caesar-iter_k3000, diamond_k25600, ladder_k6400, pmc1_k75 and
simpleCaesar_k300.

**libDAI.** libDAI ran 24 instances and answered 19 (1 run out of
memory, 4 time out). Its answers differ from the reference's by at most
4.9e-12 relative (4.8e-42 absolute, simpleCaesar_k100) over 19
instances. Over the 24 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, libDAI takes 3457 s
in all and tddlog 35.4 s; over the 19 both answer, 457 s against
tddlog's 29.8. libDAI is faster on 5 (pmc2_k200 1.28 s against 1.28,
pmc2_k51 0.24 s against 0.25, pmc2_k400 2.68 s against 3.33 and
pmc2_k800 5.43 s against 7.01, the largest differences). tddlog alone
answers caesar-iter_k1000, diamond_k25600, ladder_k6400, pmc1_k75 and
simpleCaesar_k300.

**TensorInference.jl greedy.** TensorInference.jl greedy ran 26
instances and answered 16 (5 fail, 5 time out). Its answers differ from
the reference's by at most 5.2e-12 relative (2.7e-42 absolute,
simpleCaesar_k100) over 16 instances. Over the 26 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, TensorInference.jl greedy takes 7033 s in all and tddlog 356 s;
over the 16 both answer, 1033 s against tddlog's 28.6.
TensorInference.jl greedy is faster on 3 (pmc2_k400 2.67 s against 3.33,
pmc2_k800 5.44 s against 7.01 and pmc2_k1600 10.9 s against 15.2).
tddlog alone answers caesar-iter_k1000, diamond_k1600, ladder_k1600,
pmc1_k150, pmc1_k300 and 5 more.

**TensorInference.jl treesa.** TensorInference.jl treesa ran 20
instances and answered 14 (1 fail, 5 time out). Its answers differ from
the reference's by at most 2.6e-13 relative (6.9e-44 absolute,
caesar-iter_k100) over 14 instances. Over the 20 instances given to both
that either answers, a run that does not answer counted at the 600 s
limit, TensorInference.jl treesa takes 4990 s in all and tddlog 48.1 s;
over the 14 both answer, 1390 s against tddlog's 28. TensorInference.jl
treesa is faster on 1 (pmc2_k1600 14.1 s against 15.2). tddlog alone
answers caesar-iter_k1000, diamond_k1600, ladder_k400, pmc1_k150,
pmc1_k75 and 1 more.

**Roulette.** Roulette ran 30 instances and answered 23 (4 run out of
memory, 3 underflow). Its answers differ from the reference's by at most
1.0e+00 relative (9.3e-302 absolute, caesar-iter_k1000) over 23
instances. Roulette's answer is wrong, zero or underflowed on
caesar-iter_k3000, simpleCaesar_k1000 and simpleCaesar_k300. Over the 30
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Roulette takes 5404 s in all and tddlog 643
s; over the 23 both answer, 1204 s against tddlog's 588. Roulette is
faster on 5 (pmc1_k10 0.029 s against 0.27, pmc1_k20 0.042 s against
0.29, pmc1_k40 0.069 s against 0.34 and pmc1_k300 151 s against 171, the
largest differences). tddlog alone answers caesar-iter_k10000,
caesar-iter_k3000, diamond_k6400, ladder_k6400, simpleCaesar_k1000 and 2
more.

| instance | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Ace log space | Merlin | pyAgrum | pgmpy | libDAI | TI greedy | TI treesa | Roulette | Roulette process | ProbLog | ref | tddlog diff | Geni diff | noDice diff | Psi diff | Genfer diff | SPPL diff | Pluck diff | PITA diff | aspmc diff | Storm sparse diff | Storm exact diff | Storm dd diff | PRISM sparse diff | PRISM hybrid diff | Ace diff | Ace c2d diff | Ace log space diff | Merlin diff | pyAgrum diff | pgmpy diff | libDAI diff | TI greedy diff | TI treesa diff | Roulette diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| dice_caesar-iter_k100 | 0.028 | err | 0.172 | err | 0.04 | 14.9 | 6.04 | wrong | 14.3 | 0.147 | 0.151 | 1.14 | 1.27 | 1.28 | 3.38 | 3.5 | 3.14 | 0.833 | 0.869 | 5.75 | 3.31 | 65.3 | 342 | 0.078 | 0.784 | 0.688 | alea | 5.6e-44 / 2.5e-13 |  | 7.9e-31 / 1.0e+00 |  | 6.9e-44 / 2.6e-13 | 2.2e-16 / 2.7e-13 | 6.9e-44 / 2.6e-13 | 1.0e+00 / 1.6e+60 | 6.3e-44 / 2.5e-13 | 6.9e-44 / 2.6e-13 | 6.9e-44 / 2.6e-13 | 6.9e-44 / 2.6e-13 | 6.9e-44 / 2.6e-13 | 6.9e-44 / 2.6e-13 | 6.8e-44 / 2.6e-13 | 5.6e-44 / 2.5e-13 | 1.1e-43 / 1.4e-13 | 7.9e-31 / 1.0e+00 | 5.7e-44 / 7.6e-14 | 5.7e-44 / 7.6e-14 | 5.7e-44 / 7.6e-14 | 6.9e-44 / 2.6e-13 | 6.9e-44 / 2.6e-13 | 6.9e-44 / 2.6e-13 |
| dice_caesar-iter_k1000 | 0.41 | err | 18.4 | err | 0.399 | >600 | 43.9 | >600 | >600 | 0.172 | 0.214 | 6.04 | 1.68 | 1.65 | err | err | 15.6 | 355 | 98.9 | 339 | >600 | >600 | >600 | 3.95 | 4.76 | 7.3 | alea | 4.3e-312 / 4.6e-11 |  | 9.3e-302 / 1.0e+00 |  | 9.3e-302 / 1.0e+00 |  | 9.3e-302 / 1.0e+00 |  |  | 9.3e-302 / 1.0e+00 | 4.3e-312 / 4.6e-11 | 9.3e-302 / 1.0e+00 | 9.3e-302 / 1.0e+00 | 9.3e-302 / 1.0e+00 |  |  | 5.5e-312 / 5.9e-11 | 9.3e-302 / 1.0e+00 | 9.3e-302 / 1.0e+00 | 9.3e-302 / 1.0e+00 |  |  |  | 9.3e-302 / 1.0e+00 |
| dice_caesar-iter_k3000 | 0.928 | err | oom | err | err | - | err | - | - | err | 0.366 | err | err | err | err | err | 51.1 | >600 | >600 | >600 | - | - | - | underflow | underflow | 30.2 | alea | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |
| dice_caesar-iter_k10000 | 2.6 | err | - | err | err | - | err | - | >600 | err | 1.47 | err | err | err | >600 | >600 | >600 | - | - | - | - | - | - | oom | oom | >600 | alea | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |
| dice_caesar-iter_k30000 | 8.88 | err | - | err | err | - | - | - | - | err | 7 | err | err | err | - | - | - | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |
| dice_caesar-iter_k100000 | 27.7 | err | - | err | err | - | - | - | - | err | 56.9 | >600 | >600 | >600 | - | - | - | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |
| dice_caesar-iter_k300000 | 90.6 | oom | - | err | err | - | - | - | - | err | err | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| dice_diamond_k100 | 0.01 | 0.019 | 0.043 | err | 0.008 | 3 | 5.05 | 0.809 | 2.69 | 0.148 | 0.152 | 0.721 | 1.28 | 1.28 | 2.14 | 2.31 |  | 0.127 | 0.337 | 2.1 | 0.153 | 45 | 132 | 0.046 | 0.743 | 0.419 | alea | 4.2e-15 / 4.2e-15 | 7.5e-07 / 7.6e-07 | 6.5e-07 / 6.5e-07 |  | 4.0e-15 / 4.0e-15 | 4.2e-15 / 4.2e-15 | 4.4e-15 / 4.5e-15 | 4.3e-15 / 4.4e-15 | 4.2e-15 / 4.2e-15 | 1.5e-14 / 1.5e-14 | 4.8e-15 / 4.8e-15 | 4.4e-15 / 4.5e-15 | 4.4e-15 / 4.5e-15 | 4.4e-15 / 4.5e-15 | 4.7e-15 / 4.7e-15 | 4.7e-15 / 4.7e-15 |  | 3.5e-07 / 3.6e-07 | 1.1e-14 / 1.1e-14 | 1.1e-14 / 1.1e-14 | 1.1e-14 / 1.1e-14 | 4.7e-15 / 4.7e-15 | 4.7e-15 / 4.7e-15 | 4.7e-15 / 4.7e-15 |
| dice_diamond_k400 | 0.027 | wrong | 0.234 | err | 0.025 | 13.7 | 6.13 | 120 | 3.48 | 0.154 | 0.166 | 2.62 | 1.28 | 1.27 | 3.51 | 4.16 |  | 0.555 | 1.45 | 3.27 | 1.1 | 81.1 | 520 | 0.463 | 1.17 | 4.49 | alea | 1.8e-14 / 1.8e-14 | 3.0e-06 / 3.0e-06 | 1.8e-07 / 1.9e-07 |  | 1.3e-14 / 1.3e-14 | 1.7e-14 / 1.7e-14 | 1.6e-14 / 1.7e-14 | 1.9e-14 / 1.9e-14 | 1.8e-14 / 1.8e-14 | 6.0e-14 / 6.1e-14 | 1.9e-14 / 1.9e-14 | 1.6e-14 / 1.7e-14 | 1.6e-14 / 1.7e-14 | 1.6e-14 / 1.7e-14 | 1.8e-14 / 1.8e-14 | 1.9e-14 / 2.0e-14 |  | 1.8e-07 / 1.9e-07 | 2.2e-13 / 2.3e-13 | 2.2e-13 / 2.3e-13 | 2.2e-13 / 2.3e-13 | 1.8e-14 / 1.9e-14 | 1.8e-14 / 1.9e-14 | 1.9e-14 / 1.9e-14 |
| dice_diamond_k1600 | 0.135 | wrong | 3.6 | err | 0.101 | 145 | 31.8 | >600 | 10.2 | 0.166 | 0.346 | 8.24 | 1.52 | 1.53 | 8.27 | wrong |  | 4.83 | 25 | 16.7 | 15.6 | >600 | >600 | 8.27 | 9.17 | 154 | alea | 6.4e-14 / 7.0e-14 | 1.2e-05 / 1.3e-05 | 5.0e-07 / 5.4e-07 |  | 5.2e-14 / 5.6e-14 | 6.7e-14 / 7.3e-14 | 6.3e-14 / 6.8e-14 |  | 6.4e-14 / 7.0e-14 | 2.3e-13 / 2.5e-13 | 7.1e-14 / 7.7e-14 | 6.3e-14 / 6.8e-14 | 6.3e-14 / 6.8e-14 | 6.3e-14 / 6.8e-14 | 6.9e-14 / 7.5e-14 | inf / inf |  | 5.0e-07 / 5.4e-07 | 3.2e-13 / 3.4e-13 | 3.2e-13 / 3.4e-13 | 3.2e-13 / 3.4e-13 |  |  | 7.2e-14 / 7.8e-14 |
| dice_diamond_k6400 | 0.353 | wrong | oom | err | err | >600 | err | - | 92.1 | 0.209 | 4.86 | 21.2 | 7.75 | 8.55 | 22.2 | wrong |  | oom | >600 | 277 | 258 | - | - | oom | oom | oom | alea | 1.8e-13 / 2.5e-13 | 4.0e-05 / 5.5e-05 |  |  |  |  |  |  | 1.8e-13 / 2.5e-13 | 7.4e-13 / 1.0e-12 | 2.0e-13 / 2.8e-13 | 1.8e-13 / 2.4e-13 | 1.8e-13 / 2.4e-13 | 1.8e-13 / 2.4e-13 | 2.0e-13 / 2.8e-13 | inf / inf |  |  |  | 1.9e-13 / 2.6e-13 | 1.9e-13 / 2.6e-13 |  |  |  |
| dice_diamond_k25600 | 2.26 | wrong | - | err | err | - | err | - | >600 | 0.459 | 147 | 274 | 171 | 160 | 47.5 | wrong |  | - | - | >600 | >600 | - | - | - | - | >600 | ace-auto | 4.6e-14 / 1.7e-13 | 6.3e-05 / 2.3e-04 |  |  |  |  |  |  |  | 9.1e-13 / 3.3e-12 | 8.0e-15 / 2.9e-14 | 4.4e-14 / 1.6e-13 | 4.4e-14 / 1.6e-13 | 4.4e-14 / 1.6e-13 |  | inf / inf |  |  |  |  |  |  |  |  |
| dice_diamond_k102400 | 7.5 | wrong | - | err | err | - | - | - | - | 1.16 | >600 | >600 | >600 | >600 | err | err |  | - | - | - | - | - | - | - | - | - | tddlog | ref | 9.7e-05 / 1.6e-02 |  |  |  |  |  |  |  | 8.3e-14 / 1.4e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| dice_diamond_k409600 | 31.1 | wrong | - | err | err | - | - | - | - | 4.23 | - | - | - | - | >600 | >600 |  | - | - | - | - | - | - | - | - | - | tddlog | ref | 2.1e-03 / 1.6e+06 |  |  |  |  |  |  |  | 7.1e-20 / 5.6e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| dice_diamond_k1638400 | 142 | oom | - | err | err | - | - | - | - | 16.9 | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  | 5.9e-46 / 2.2e-10 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| dice_ladder_k100 | 0.057 | 0.06 | >600 | err | 0.075 | 4.13 | 5.63 | 9.66 | 3.04 | 0.302 | 0.305 | 6.89 | 2.55 | 2.54 | 2.67 | 3.07 |  | 0.268 | 0.601 | 2.76 | 0.396 | 47.1 | 228 | 0.083 | 0.786 | 0.766 | alea | 5.6e-16 / 1.2e-15 | 2.6e-07 / 5.4e-07 |  |  | 5.6e-16 / 1.2e-15 | 6.1e-16 / 1.3e-15 | 5.6e-16 / 1.2e-15 | 5.6e-16 / 1.2e-15 | 5.6e-16 / 1.2e-15 | 7.2e-16 / 1.5e-15 | 5.6e-16 / 1.2e-15 | 7.2e-16 / 1.5e-15 | 7.2e-16 / 1.5e-15 | 7.2e-16 / 1.5e-15 | 3.9e-16 / 8.2e-16 | 5.6e-16 / 1.2e-15 |  | 1.6e-07 / 3.3e-07 | 6.8e-14 / 1.4e-13 | 6.8e-14 / 1.4e-13 | 6.8e-14 / 1.4e-13 | 6.7e-16 / 1.4e-15 | 6.7e-16 / 1.4e-15 | 7.2e-16 / 1.5e-15 |
| dice_ladder_k400 | 0.196 | 0.234 | >600 | err | 0.289 | 23.8 | 16 | >600 | 7.61 | 0.306 | 0.336 | 21.3 | 2.52 | 2.55 | 5.63 | 8.08 |  | 1.65 | 5.06 | 8.88 | 4.44 | 233 | >600 | 0.869 | 1.59 | oom | alea | 1.9e-15 / 4.6e-15 | 9.7e-07 / 2.4e-06 |  |  | 2.5e-15 / 6.1e-15 | 1.8e-15 / 4.3e-15 | 2.1e-15 / 5.2e-15 |  | 2.4e-15 / 6.0e-15 | 2.7e-15 / 6.5e-15 | 1.8e-15 / 4.5e-15 | 2.8e-15 / 6.9e-15 | 2.8e-15 / 6.9e-15 | 2.8e-15 / 6.9e-15 | 2.4e-15 / 5.8e-15 | 2.5e-15 / 6.1e-15 |  | 1.3e-07 / 3.1e-07 | 4.9e-13 / 1.2e-12 | 4.9e-13 / 1.2e-12 | 4.9e-13 / 1.2e-12 | 2.7e-15 / 6.5e-15 |  | 2.4e-15 / 5.8e-15 |
| dice_ladder_k1600 | 0.462 | wrong | >600 | err | err | 246 | err | - | 109 | 0.327 | 0.807 | 84 | 3.29 | 3.23 | err | err |  | oom | 139 | 106 | 74.5 | >600 | - | 14.5 | 15.5 | - | alea | 3.9e-15 / 1.7e-14 | 1.9e-06 / 8.6e-06 |  |  |  | 3.2e-15 / 1.4e-14 |  |  | 4.7e-15 / 2.1e-14 | 4.5e-15 / 2.0e-14 | 4.1e-15 / 1.8e-14 | 4.7e-15 / 2.1e-14 | 4.7e-15 / 2.1e-14 | 4.7e-15 / 2.1e-14 |  |  |  |  | 3.7e-13 / 1.7e-12 | 3.7e-13 / 1.7e-12 | 3.7e-13 / 1.7e-12 |  |  | 4.4e-15 / 2.0e-14 |
| dice_ladder_k6400 | 1.45 | 4.15 | oom | err | err | >600 | - | - | oom | 0.397 | 11.7 | >600 | 22.8 | 20.8 | err | err |  | - | >600 | >600 | >600 | - | - | oom | oom | - | tddlog | ref | 8.9e-07 / 4.4e-05 |  |  |  |  |  |  |  | 2.0e-16 / 1.0e-14 | 1.0e-16 / 5.1e-15 |  | 2.8e-16 / 1.4e-14 | 2.8e-16 / 1.4e-14 |  |  |  |  |  |  |  |  |  |  |
| dice_ladder_k25600 | 7.16 | wrong | - | err | err | - | err | - | >600 | 0.922 | 392 | - | >600 | >600 | err | err |  | - | - | - | - | - | - | - | - | >600 | tddlog | ref | 2.8e-05 / 2.1e+01 |  |  |  |  |  |  |  | 3.7e-20 / 2.7e-14 | 2.3e-20 / 1.6e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |
| dice_ladder_k102400 | 26.5 | wrong | - | err | err | - | - | - | - | 2.75 | >600 | - | - | - | err | err |  | - | - | - | - | - | - | - | - | - | tddlog | ref | 3.0e-05 / 1.0e+18 |  |  |  |  |  |  |  | 3.5e-36 / 1.2e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| dice_ladder_k409600 | 114 | oom | - | err | err | - | - | - | - | 9.48 | - | - | - | - | err | err |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  | 5.8e-103 / 1.1e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| dice_ladder_k1638400 | oom | - | - | err | err | - | - | - | - | 37.9 | - | - | - | - | err | err |  | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| dice_pmc1_k10 | 0.145 | 0.005 | >600 | err | >600 | >600 | 9.24 | 1.24 | 1.66 | 0.476 | 0.499 | 8.22 | 1.96 | 2 | 1.22 | 1.39 |  | 0.296 | 0.438 | 2.27 | 0.309 | 15.5 | 16.4 | 0.029 | 0.741 | 0.176 | alea | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| dice_pmc1_k20 | 0.257 | 0.005 | >600 | err | - | - | 9.91 | 1.32 | 1.6 | 107 | 134 | 7.7 | 2 | 1.97 | 1.24 | 1.42 |  | 0.316 | 0.455 | 2.32 | 0.32 | 15.7 | 16.4 | 0.042 | 0.761 | 0.197 | alea | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| dice_pmc1_k40 | 0.174 | 0.005 | >600 | err | - | - | 10.3 | 1.5 | 1.65 | >600 | >600 | 7.71 | 2.49 | 2.57 | 1.37 | 1.39 |  | 0.271 | 0.39 | 2.27 | 0.255 | 15.3 | 16.1 | 0.069 | 0.775 | 0.233 | alea | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| dice_pmc1_k75 | 0.694 | >600 | >600 | err | - | - | 13.9 | 2.28 | 37.3 | - | - | 10.2 | 145 | 156 | 4.1 | 3.7 |  | oom | oom | oom | oom | err | err | 11.8 | 12.6 | oom | alea | 0.0e+00 / 0.0e+00 |  |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |
| dice_pmc1_k150 | 18.3 | - | - | err | - | - | 19.6 | 4.03 | >600 | - | - | 17.3 | 285 | 286 | >600 | >600 |  | - | - | - | - | err | >600 | 62.7 | 63.5 | oom | alea | 0.0e+00 / 0.0e+00 |  |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |
| dice_pmc1_k300 | 94.2 | - | - | err | - | - | 26 | 11.2 | - | - | - | 22.1 | 395 | 413 | - | - |  | - | - | - | - | err | - | 151 | 152 | - | alea | 0.0e+00 / 0.0e+00 |  |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |
| dice_pmc1_k600 | 184 | - | - | err | - | - | 54.5 | 36.3 | - | - | - | 29.9 | oom | oom | - | - |  | - | - | - | - | err | - | 329 | 330 | - | alea | 0.0e+00 / 0.0e+00 |  |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |
| dice_pmc2_k10 | 0.061 | 0.005 | >600 | err | >600 | >600 | 8 | 0.743 | 1.57 | 0.253 | 0.264 | 3.54 | 1.34 | 1.36 | 1.26 | 1.27 |  | 0.115 | 0.271 | 2.1 | 0.114 | 15.4 | 16.5 | 0.01 | 0.772 | 0.143 | alea | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| dice_pmc2_k20 | 0.104 | 0.005 | >600 | err | - | - | 8.29 | 0.796 | 1.56 | 0.426 | 0.499 | 3.71 | 1.38 | 1.33 | 1.23 | 1.25 |  | 0.126 | 0.282 | 2.14 | 0.135 | 15.5 | 16.4 | 0.026 | 0.755 | 0.157 | alea | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| dice_pmc2_k51 | 0.24 | >600 | >600 | err | - | - | 9.73 | 1.02 | 14 | 0.82 | 1.06 | 5.62 | 2.17 | 2.06 | 1.75 | 1.54 |  | 0.244 | 0.389 | 2.26 | 0.239 | 25.6 | 31.2 | 2.96 | 3.68 | 23.4 | alea | 0.0e+00 / 0.0e+00 |  |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| dice_pmc2_k200 | 1.28 | - | - | err | - | - | 14.3 | 2.99 | >600 | 2.08 | 2.91 | 10.8 | 5.63 | 5.6 | 1.98 | 2.03 |  | 1.26 | 1.37 | 2.76 | 1.28 | 20.6 | 24.2 | 37.8 | 38.5 | oom | alea | 0.0e+00 / 0.0e+00 |  |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| dice_pmc2_k400 | 3.33 | - | - | err | - | - | 30.2 | 17.7 | >600 | 7.53 | 10.9 | 15.9 | 11.8 | 13.2 | 3.92 | 3.99 |  | 2.71 | 2.93 | 4.65 | 2.68 | 2.67 | 6.62 | 73 | 73.8 | >600 | alea | 0.0e+00 / 0.0e+00 |  |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| dice_pmc2_k800 | 7.01 | - | - | err | - | - | 52.4 | 57.1 | - | 12.7 | 17.9 | 20.5 | 20.1 | 20 | 6.44 | 6.61 |  | 5.48 | 5.41 | 7.16 | 5.43 | 5.44 | 9.1 | 156 | 157 | - | alea | 0.0e+00 / 0.0e+00 |  |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| dice_pmc2_k1600 | 15.2 | - | - | err | - | - | 103 | 204 | - | 23.5 | 33.3 | 32.8 | 32.1 | 33 | 11.9 | 11.6 |  | 10.4 | 10.9 | 12.5 | 10.4 | 10.9 | 14.1 | 350 | 351 | - | alea | 0.0e+00 / 0.0e+00 |  |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| dice_simpleCaesar_k100 | 0.352 | err | 5.01 | err | 382 | >600 | 57.2 | >600 | 114 | 0.198 | 0.315 | 4.27 | 1.56 | 1.51 | 8.7 | 8.83 | 8.38 | 7.28 | 8.71 | 211 | 77.7 | 419 | >600 | 0.865 | 1.62 | 61.1 | alea | 2.7e-42 / 5.2e-12 |  | 5.9e-30 / 1.0e+00 |  | 2.7e-42 / 5.2e-12 |  | 2.7e-42 / 5.2e-12 |  | 2.7e-42 / 5.2e-12 | 2.7e-42 / 5.3e-12 | 2.7e-42 / 5.2e-12 | 2.7e-42 / 5.3e-12 | 2.7e-42 / 5.2e-12 | 2.7e-42 / 5.2e-12 | 2.7e-42 / 5.2e-12 | 2.7e-42 / 5.2e-12 | 6.8e-43 / 5.5e-12 | 5.9e-30 / 1.0e+00 | 4.8e-42 / 4.9e-12 | 4.8e-42 / 4.9e-12 | 4.8e-42 / 4.9e-12 | 2.7e-42 / 5.2e-12 |  | 2.7e-42 / 5.2e-12 |
| dice_simpleCaesar_k300 | 0.867 | err | wrong | err | >600 | - | err | - | >600 | err | 0.66 | err | err | err | err | err | 17.2 | oom | err | >600 | >600 | err | - | underflow | underflow | 186 | alea | 2.7e-99 / 1.3e-11 |  | 5.0e-01 / 2.4e+87 |  |  |  |  |  |  |  | 2.7e-99 / 1.3e-11 |  |  |  |  |  | 1.7e-99 / 8.1e-12 |  |  |  |  |  |  |  |
| dice_simpleCaesar_k1000 | 3.9 | err | oom | err | - | - | - | - | - | err | 1.91 | err | err | err | err | err | 52.1 | - | >600 | - | - | >600 | - | underflow | underflow | >600 | alea | 1.0e-302 / 1.9e-10 |  |  |  |  |  |  |  |  |  | 1.0e-302 / 1.9e-10 |  |  |  |  |  | 1.5e-302 / 2.8e-10 |  |  |  |  |  |  |  |
| dice_simpleCaesar_k3000 | 23.3 | >600 | - | err | - | - | err | - | >600 | err | 7.74 | err | err | err | err | err | 159 | - | >600 | - | - | >600 | - | oom | oom | >600 | ace-Xlog | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |
| dice_simpleCaesar_k10000 | 47.2 | - | - | err | - | - | - | - | - | err | 30.1 | err | err | err | >600 | >600 | >600 | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |
| dice_simpleCaesar_k30000 | 142 | - | - | err | - | - | - | - | - | err | err | err | err | err | - | - | - | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| dice_simpleCaesar_k100000 | oom | - | - | err | - | - | - | - | - | err | - | >600 | >600 | >600 | - | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |

Seconds per system, the median over the repetitions; a failure is `>600`
(timeout), `oom`, `err` or `n/a`; each system's last columns are its
largest absolute and relative difference from the reference named in
`ref`.

### On Alea.jl's and noDice's programs

| family | instances | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Scallop | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Merlin | pyAgrum | pgmpy | libDAI | TensorInference.jl greedy | TensorInference.jl treesa | Roulette |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| arith | 28 | 26 | 2 of 28 | 0 of 28 | 1 of 28 | 3 of 28 | 2 of 28 | 19 of 28 | 18 of 28 | 20 of 28 | 2 of 28 | 13 of 28 | 12 of 28 | 19 of 28 | 9 of 28 | 9 of 28 | 20 of 25 | 19 of 24 | 14 of 22 | 14 of 22 | 13 of 21 | 14 of 22 | 12 of 20 | 15 of 21 | 22 of 25 |
| hybit | 19 | 16 | 0 of 19 | 0 of 19 | 0 of 19 | 0 of 19 | 0 of 19 | 2 of 19 | 6 of 19 | 4 of 19 | 0 of 19 | 0 of 19 | 0 of 19 | 1 of 19 | 2 of 19 | 2 of 19 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 5 of 19 |
| nodice | 69 | 68 | 32 of 66 | 51 of 58 | 1 of 69 | 49 of 63 | 35 of 56 | 59 of 64 | 50 of 55 | 57 of 64 | 3 of 32 | 63 of 67 | 61 of 66 | 63 of 66 | 61 of 66 | 62 of 67 | 61 of 64 | 53 of 63 | 53 of 60 | 54 of 60 | 52 of 66 | 54 of 60 | 39 of 59 | 50 of 55 | 61 of 65 |
| pluck | 4 | 4 | 0 of 4 | 0 of 4 | 0 of 4 | 0 of 4 | 0 of 4 | 1 of 4 | 3 of 4 | 3 of 4 | 0 of 4 | 0 of 4 | 0 of 4 | 1 of 4 | 1 of 4 | 1 of 4 | 1 of 4 | 1 of 4 | 0 of 4 | 1 of 4 | 1 of 4 | 1 of 4 | 1 of 4 | 2 of 4 | 3 of 4 |

**Geni.** Geni ran 117 instances and answered 34 (1 fail, 10 run out of
memory, 57 time out, 15 answer wrongly). Its answers differ from the
reference's by at most 3.2e-06 relative (8.0e-07 absolute,
arith_linext_small) over 34 instances. Geni's answer is wrong, zero or
underflowed on brp4096-2, brp64-2, diamond-1000-p, diamond-10000-p,
diamond-200-p and 10 more. Over the 112 instances given to both that
either answers, a run that does not answer counted at the 600 s limit,
Geni takes 46929 s in all and tddlog 1479 s; over the 34 both answer,
129 s against tddlog's 9.78. Geni is faster on 3 (hepar2 0.15 s against
0.76, brp256-2 0.31 s against 1.29 and brp1024-2 1.24 s against 6.39).
tddlog alone answers arith_book, arith_caesar_huge, arith_caesar_large,
arith_caesar_medium, arith_caesar_small and 73 more.

**noDice.** noDice ran 109 instances and answered 51 (9 fail, 29 run out
of memory, 18 time out, 2 print zeros). Its answers differ from the
reference's by at most 4.9e-02 relative (4.6e-07 absolute, uf20-0990)
over 51 instances. noDice's answer is wrong, zero or underflowed on
3sat-0 and string_editing. Over the 103 instances given to both that
either answers, a run that does not answer counted at the 600 s limit,
noDice takes 33085 s in all and tddlog 1124 s; over the 51 both answer,
1885 s against tddlog's 8.68. tddlog alone answers arith_book,
arith_caesar_huge, arith_caesar_large, arith_caesar_medium,
arith_caesar_small and 47 more.

**Psi.** Psi ran 120 instances and answered 2 (53 fail, 28 run out of
memory, 37 time out). Its answers differ from the reference's by at most
4.4e-16 relative (2.2e-16 absolute, arith_linext_small) over 2
instances. Over the 114 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, Psi takes 67397 s
in all and tddlog 1547 s; over the 2 both answer, 197 s against tddlog's
0.01. tddlog alone answers arith_book, arith_caesar_huge,
arith_caesar_large, arith_caesar_medium, arith_caesar_small and 107
more.

**Genfer.** Genfer ran 114 instances and answered 52 (17 fail, 39 run
out of memory, 6 time out). Its answers differ from the reference's by
at most 7.0e-14 relative (6.3e-14 absolute, diamond-2000-p) over 52
instances. Over the 109 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, Genfer takes 34755
s in all and tddlog 1479 s; over the 52 both answer, 555 s against
tddlog's 11.4. Genfer is faster on 5 (diamond-2000-p 0.24 s against
0.35, brp64-2 0.081 s against 0.28, hepar2 0.27 s against 0.76 and
brp256-2 0.31 s against 1.29, the largest differences). tddlog alone
answers arith_book, arith_caesar_huge, arith_caesar_large,
arith_caesar_medium, arith_caesar_small and 52 more.

**SPPL.** SPPL ran 107 instances and answered 37 (7 fail, 1 run out of
memory, 62 time out). Its answers differ from the reference's by at most
8.4e-14 relative (7.6e-14 absolute, diamond-2000-p) over 37 instances.
Over the 102 instances given to both that either answers, a run that
does not answer counted at the 600 s limit, SPPL takes 40569 s in all
and tddlog 1060 s; over the 37 both answer, 1569 s against tddlog's
4.58. tddlog alone answers arith_book, arith_caesar_huge,
arith_caesar_large, arith_caesar_medium, arith_caesar_small and 60 more.

**Pluck.** Pluck ran 115 instances and answered 81 (6 fail, 25 run out
of memory, 1 time out, 2 answer wrongly). Its answers differ from the
reference's by at most 3.4e-13 relative (9.1e-15 absolute,
arith_caesar_medium) over 81 instances. Pluck's answer is wrong, zero or
underflowed on pcfg and string_editing. Over the 110 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, Pluck takes 19489 s in all and tddlog 1080 s; over the 81 both
answer, 2089 s against tddlog's 40.1. tddlog alone answers arith_book,
arith_caesar_huge, arith_caesar_large, arith_disease_medium,
arith_disease_small and 24 more.

**PITA.** PITA ran 106 instances and answered 77 (28 time out, 1 answer
wrongly). Its answers differ from the reference's by at most 1.0e+00
relative (1.3e-16 absolute, arith_plankton) over 77 instances. PITA's
answer is wrong, zero or underflowed on arith_caesar_small. Over the 101
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, PITA takes 17507 s in all and tddlog 1058 s;
over the 77 both answer, 3107 s against tddlog's 59.3. PITA is faster on
1 (sorted_list 2.2 s against 5.82). tddlog alone answers
arith_caesar_huge, arith_caesar_large, arith_caesar_medium,
arith_caesar_small, arith_disease_medium and 19 more.

**aspmc.** aspmc ran 115 instances and answered 84 (2 run out of memory,
29 time out). Its answers differ from the reference's by at most 9.2e-09
relative (4.7e-09 absolute, hybit_clickGraph) over 84 instances. Over
the 110 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, aspmc takes 19681 s in all and tddlog
1063 s; over the 84 both answer, 4081 s against tddlog's 75.5. aspmc is
faster on 2 (sorted_list 5.09 s against 5.82 and uf50-0999 3.09 s
against 13.1). tddlog alone answers arith_caesar_huge,
arith_caesar_large, arith_disease_medium, arith_fw_large,
arith_gcd_medium and 21 more.

**Scallop.** Scallop ran 83 instances and answered 5 (47 run out of
memory, 31 time out). Its answers differ from the reference's by at most
4.4e-16 relative (2.2e-16 absolute, arith_linext_small) over 5
instances. Over the 77 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, Scallop takes 43200
s in all and tddlog 1117 s; over the 5 both answer, 0.48 s against
tddlog's 0.036. tddlog alone answers arith_book, arith_caesar_huge,
arith_caesar_large, arith_caesar_medium, arith_caesar_small and 67 more.

**Storm sparse.** Storm sparse ran 118 instances and answered 76 (1
fail, 2 run out of memory, 39 time out). Its answers differ from the
reference's by at most 1.3e-12 relative (7.7e-13 absolute,
diamond-10000-p) over 76 instances. Over the 113 instances given to both
that either answers, a run that does not answer counted at the 600 s
limit, Storm sparse takes 24385 s in all and tddlog 1617 s; over the 76
both answer, 2185 s against tddlog's 77.2. Storm sparse is faster on 4
(hepar2 0.18 s against 0.76, brp256-2 0.27 s against 1.28, brp1024-2
0.29 s against 6.09 and brp4096-2 0.4 s against 45.1). tddlog alone
answers arith_book, arith_caesar_huge, arith_caesar_large,
arith_disease_medium, arith_disease_small and 32 more.

**Storm exact.** Storm exact ran 117 instances and answered 73 (6 fail,
5 run out of memory, 33 time out). Its answers differ from the
reference's by at most 3.4e-13 relative (9.1e-15 absolute,
arith_caesar_medium) over 73 instances. Over the 112 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, Storm exact takes 25220 s in all and tddlog 1609 s; over the 73
both answer, 1820 s against tddlog's 75.7. Storm exact is faster on 5
(brp64-2 0.27 s against 0.27, brp4096-2 45 s against 45.1, hepar2 0.19 s
against 0.76 and brp256-2 0.36 s against 1.28, the largest differences).
tddlog alone answers arith_book, arith_caesar_huge, arith_caesar_large,
arith_disease_medium, arith_disease_small and 34 more.

**Storm dd.** Storm dd ran 117 instances and answered 84 (7 fail, 1 run
out of memory, 25 time out). Its answers differ from the reference's by
at most 6.0e-13 relative (1.3e-29 absolute, arith_caesar_large) over 83
instances. Over the 112 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, Storm dd takes
19832 s in all and tddlog 2267 s; over the 83 both answer, 3006 s
against tddlog's 247. Storm dd is faster on 1 (coupon2-18-obs 18 s
against 54.4). Only Storm dd answers coupon2-20-obs. tddlog alone
answers arith_book, arith_caesar_huge, arith_disease_medium,
arith_disease_small, arith_fw_large and 23 more.

**PRISM sparse.** PRISM sparse ran 117 instances and answered 73 (2
fail, 6 run out of memory, 36 time out). Its answers differ from the
reference's by at most 3.4e-02 relative (6.2e-15 absolute, hmm) over 73
instances. Over the 112 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, PRISM sparse takes
25683 s in all and tddlog 1614 s; over the 73 both answer, 2283 s
against tddlog's 62.7. PRISM sparse is faster on 2 (brp1024-2 4.16 s
against 6.09 and brp4096-2 25.1 s against 45.1). tddlog alone answers
arith_book, arith_caesar_huge, arith_caesar_large, arith_caesar_medium,
arith_disease_medium and 34 more.

**PRISM hybrid.** PRISM hybrid ran 118 instances and answered 74 (2
fail, 6 run out of memory, 36 time out). Its answers differ from the
reference's by at most 3.4e-02 relative (6.2e-15 absolute, hmm) over 74
instances. Over the 113 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, PRISM hybrid takes
26015 s in all and tddlog 1669 s; over the 74 both answer, 2615 s
against tddlog's 70.3. PRISM hybrid is faster on 2 (brp1024-2 3.58 s
against 6.09 and brp4096-2 29.1 s against 45.1). tddlog alone answers
arith_book, arith_caesar_huge, arith_caesar_large, arith_caesar_medium,
arith_disease_medium and 34 more.

**Ace.** Ace ran 94 instances and answered 82 (1 fail, 11 time out). Its
answers differ from the reference's by at most 6.0e-13 relative (3.3e-16
absolute, arith_caesar_large) over 82 instances. Over the 94 instances
given to both that either answers, a run that does not answer counted at
the 600 s limit, Ace takes 8996 s in all and tddlog 822 s; over the 82
both answer, 1796 s against tddlog's 525. Ace is faster on 1 (brp16384-2
181 s against 446). tddlog alone answers arith_book, arith_caesar_huge,
arith_disease_small, arith_gcd_medium, arith_triangle_large and 7 more.

**Ace c2d.** Ace c2d ran 92 instances and answered 73 (2 fail, 12 time
out, 5 answer wrongly). Its answers differ from the reference's by at
most 6.0e-13 relative (1.1e-16 absolute, arith_caesar_large) over 73
instances. Ace c2d's answer is wrong, zero or underflowed on
diamond-10000-p, diamond-2000-p, diamond-3000-p, diamond-4000-p and
diamond-5000-p. Over the 92 instances given to both that either answers,
a run that does not answer counted at the 600 s limit, Ace c2d takes
12779 s in all and tddlog 713 s; over the 73 both answer, 1379 s against
tddlog's 64.9. tddlog alone answers arith_book, arith_caesar_huge,
arith_disease_small, arith_gcd_small, arith_triangle_large and 14 more.

**Merlin.** Merlin ran 87 instances and answered 67 (5 fail, 12 run out
of memory, 2 time out, 1 print zeros). Its answers differ from the
reference's by at most 1.0e+00 relative (4.7e-17 absolute,
arith_caesar_large) over 67 instances. Merlin's answer is wrong, zero or
underflowed on hmm. Over the 87 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, Merlin
takes 12378 s in all and tddlog 237 s; over the 67 both answer, 378 s
against tddlog's 17. Merlin is faster on 1 (hepar2 0.23 s against 0.76).
tddlog alone answers arith_book, arith_caesar_huge, arith_disease_small,
arith_fw_large, arith_gcd_small and 15 more.

**pyAgrum.** pyAgrum ran 87 instances and answered 69 (2 fail, 11 run
out of memory, 5 time out). Its answers differ from the reference's by
at most 5.1e-12 relative (4.2e-29 absolute, arith_caesar_large) over 69
instances. Over the 87 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, pyAgrum takes 12132
s in all and tddlog 237 s; over the 69 both answer, 1332 s against
tddlog's 17.3. pyAgrum is faster on 1 (hepar2 0.4 s against 0.76).
tddlog alone answers arith_book, arith_caesar_huge, arith_disease_small,
arith_fw_large, arith_gcd_small and 13 more.

**pgmpy.** pgmpy ran 92 instances and answered 66 (1 fail, 4 run out of
memory, 21 time out). Its answers differ from the reference's by at most
4.2e-12 relative (4.5e-13 absolute, insurance) over 66 instances. Over
the 92 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, pgmpy takes 17808 s in all and tddlog
246 s; over the 66 both answer, 2208 s against tddlog's 21.4. tddlog
alone answers arith_book, arith_caesar_large, arith_disease_small,
arith_fw_large, arith_gcd_small and 21 more.

**libDAI.** libDAI ran 87 instances and answered 69 (4 fail, 10 run out
of memory, 4 time out). Its answers differ from the reference's by at
most 5.1e-12 relative (4.2e-29 absolute, arith_caesar_large) over 69
instances. Over the 87 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, libDAI takes 12486
s in all and tddlog 237 s; over the 69 both answer, 1686 s against
tddlog's 18.8. libDAI is faster on 1 (hepar2 0.35 s against 0.76).
tddlog alone answers arith_book, arith_caesar_huge, arith_disease_small,
arith_fw_large, arith_gcd_small and 13 more.

**TensorInference.jl greedy.** TensorInference.jl greedy ran 84
instances and answered 52 (17 fail, 9 run out of memory, 6 time out).
Its answers differ from the reference's by at most 4.5e-14 relative
(4.3e-14 absolute, diamond-1000-p) over 52 instances. Over the 84
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, TensorInference.jl greedy takes 19670 s in
all and tddlog 188 s; over the 52 both answer, 470 s against tddlog's
4.74. TensorInference.jl greedy is faster on 1 (hepar2 0.28 s against
0.76). tddlog alone answers arith_book, arith_caesar_medium,
arith_disease_small, arith_fw_large, arith_gcd_small and 27 more.

**TensorInference.jl treesa.** TensorInference.jl treesa ran 81
instances and answered 67 (5 run out of memory, 9 time out). Its answers
differ from the reference's by at most 4.3e-14 relative (1.2e-16
absolute, arith_plankton) over 67 instances. Over the 81 instances given
to both that either answers, a run that does not answer counted at the
600 s limit, TensorInference.jl treesa takes 16893 s in all and tddlog
190 s; over the 67 both answer, 8493 s against tddlog's 19. tddlog alone
answers arith_caesar_medium, arith_disease_small, arith_fw_large,
arith_gcd_small, arith_triangle_medium and 9 more.

**Roulette.** Roulette ran 113 instances and answered 91 (16 run out of
memory, 6 time out). Its answers differ from the reference's by at most
1.0e+00 relative (1.2e-57 absolute, arith_caesar_huge) over 91
instances. Over the 110 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, Roulette takes
13186 s in all and tddlog 1747 s; over the 91 both answer, 1786 s
against tddlog's 417. Roulette is faster on 10 (arith_caesar_large 1.33
s against 1.49, water 0.41 s against 0.61, arith_triangle_small 0.21 s
against 0.52 and arith_caesar_medium 0.58 s against 1.92, the largest
differences). tddlog alone answers arith_disease_small,
arith_gcd_medium, arith_plankton, hybit_addFun_max, hybit_addFun_sum and
14 more.

| instance | flips | gates | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Scallop | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Merlin | pyAgrum | pgmpy | libDAI | TI greedy | TI treesa | Roulette | Roulette process | ProbLog | ref | tddlog diff | Geni diff | noDice diff | Psi diff | Genfer diff | SPPL diff | Pluck diff | PITA diff | aspmc diff | Scallop diff | Storm sparse diff | Storm exact diff | Storm dd diff | PRISM sparse diff | PRISM hybrid diff | Ace diff | Ace c2d diff | Merlin diff | pyAgrum diff | pgmpy diff | libDAI diff | TI greedy diff | TI treesa diff | Roulette diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| arith_book | 180 | 2453 | 9.4 | >600 | >600 | >600 | oom | >600 | oom | 106 | 165 | oom | >600 | >600 | >600 | >600 | >600 | >600 | >600 | err | oom | >600 | err | err | 347 | 3.58 | 4.37 | >600 | alea | 6.9e-17 / 5.6e-16 |  |  |  |  |  |  | 8.3e-17 / 6.7e-16 | 2.8e-17 / 2.2e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.1e-16 / 8.9e-16 | 5.6e-17 / 4.5e-16 |
| arith_caesar_huge | 6260 | 38517 | 2.29 | oom | oom | oom | err | >600 | oom | >600 | >600 | oom | >600 | >600 | >600 | >600 | >600 | err | err | oom | >600 | - | >600 | - | - | 5.33 | 6.24 | >600 | alea | 3.5e-69 / 3.1e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.2e-57 / 1.0e+00 |
| arith_caesar_large | 2510 | 15417 | 1.09 | >600 | oom | oom | oom | >600 | oom | >600 | >600 | oom | >600 | >600 | 168 | >600 | >600 | 14.6 | 33.7 | 22.4 | 89.1 | >600 | 425 | - | - | 1.33 | 2.12 | 128 | alea | 1.3e-29 / 6.0e-13 |  |  |  |  |  |  |  |  |  |  |  | 1.3e-29 / 6.0e-13 |  |  | 3.3e-16 / 6.0e-13 | 1.1e-16 / 6.0e-13 | 4.7e-17 / 1.0e+00 | 4.2e-29 / 5.1e-12 |  | 4.2e-29 / 5.1e-12 |  |  | 1.3e-29 / 5.9e-13 |
| arith_caesar_medium | 1260 | 7717 | 1.6 | >600 | oom | oom | oom | >600 | 24.1 | >600 | 191 | oom | 188 | 242 | 44.2 | >600 | >600 | 9.2 | 13.9 | 5.8 | 16.5 | 268 | 69.6 | >600 | >600 | 0.576 | 1.31 | 60.7 | alea | 9.1e-15 / 3.4e-13 |  |  |  |  |  | 9.1e-15 / 3.4e-13 |  | 8.9e-15 / 3.4e-13 |  | 9.1e-15 / 3.4e-13 | 9.1e-15 / 3.4e-13 | 9.1e-15 / 3.4e-13 |  |  | 9.1e-15 / 3.4e-13 | 9.1e-15 / 3.4e-13 | 9.8e-08 / 1.0e+00 | 2.3e-13 / 3.2e-12 | 2.3e-13 / 3.2e-12 | 2.3e-13 / 3.2e-12 |  |  | 9.1e-15 / 3.4e-13 |
| arith_caesar_small | 260 | 1557 | 0.108 | >600 | err | >600 | oom | >600 | 6.42 | wrong | 6.11 | >600 | 1.65 | 3.1 | 10.3 | 10.2 | 9.71 | 3.52 | 4.02 | 1.01 | 1.05 | 22.3 | 2.29 | 0.709 | 200 | 0.105 | 0.853 | 11.7 | alea | 1.3e-15 / 3.3e-14 |  |  |  |  |  | 1.3e-15 / 3.2e-14 | 3.1e-02 / 1.3e+03 | 1.8e-15 / 3.2e-14 |  | 1.2e-15 / 3.1e-14 | 1.4e-15 / 3.1e-14 | 1.4e-15 / 3.1e-14 | 1.0e-15 / 3.0e-14 | 1.0e-15 / 3.0e-14 | 1.3e-15 / 3.2e-14 | 1.3e-15 / 3.1e-14 | 4.9e-07 / 1.0e+00 | 5.0e-13 / 2.9e-12 | 5.0e-13 / 2.9e-12 | 5.0e-13 / 2.9e-12 | 1.1e-15 / 3.2e-14 | 1.6e-15 / 3.2e-14 | 1.4e-15 / 3.1e-14 |
| arith_disease_large | 3044 | 188220 | >600 | oom | oom | oom | err | err | oom | >600 | oom | >600 | >600 | >600 | >600 | >600 | >600 | - | - | - | - | - | - | - | - | - | - | oom |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| arith_disease_medium | 1022 | 62049 | 306 | >600 | oom | oom | err | >600 | oom | >600 | >600 | oom | >600 | >600 | >600 | >600 | >600 | - | - | - | - | - | - | - | - | - | - | oom | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| arith_disease_small | 213 | 11614 | 20.4 | >600 | >600 | oom | err | >600 | oom | >600 | 456 | oom | >600 | >600 | err | oom | oom | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | alea | 8.2e-16 / 6.1e-13 |  |  |  |  |  |  |  | 8.5e-16 / 6.1e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| arith_fw_large | 44 | 7444 | 0.507 | >600 | >600 | >600 | oom | >600 | 20.8 | 489 | >600 | oom | >600 | >600 | >600 | oom | oom | 403 | 413 | oom | oom | >600 | oom | oom | oom | 0.768 | 1.49 | >600 | alea | 4.2e-17 / 8.9e-15 |  |  |  |  |  | 4.2e-17 / 7.5e-15 | 1.2e-16 / 6.1e-09 |  |  |  |  |  |  |  | 1.9e-16 / 7.5e-15 | 1.4e-16 / 7.5e-15 |  |  |  |  |  |  | 5.6e-17 / 7.4e-15 |
| arith_fw_medium | 17 | 2844 | 0.1 | >600 | >600 | >600 | oom | err | 6.32 | 8.54 | 9.48 | oom | 14.3 | 51.2 | 60.3 | 243 | 255 | 2.81 | 2.31 | 0.506 | 0.485 | 2.86 | 0.465 | 0.197 | 40.2 | 0.086 | 0.782 | 17.8 | alea | 8.3e-17 / 2.6e-14 |  |  |  |  |  | 8.3e-17 / 6.2e-16 | 1.4e-16 / 5.7e-14 | 5.6e-17 / 8.3e-16 |  | 8.3e-17 / 4.2e-16 | 5.6e-17 / 4.2e-16 | 5.6e-17 / 4.2e-16 | 5.6e-17 / 4.2e-16 | 5.6e-17 / 4.2e-16 | 5.6e-17 / 6.2e-16 | 5.6e-17 / 6.8e-16 | 4.2e-07 / 1.6e-03 | 3.3e-13 / 2.6e-12 | 3.3e-13 / 2.6e-12 | 3.3e-13 / 2.6e-12 | 2.8e-17 / 2.8e-16 | 5.6e-17 / 4.2e-16 | 8.3e-17 / 6.2e-16 |
| arith_fw_small | 3 | 118 | 0.012 | 5.38 | err | >600 | 0.057 | 2.5 | 5.73 | 0.758 | 3.53 | 0.121 | 0.176 | 0.18 | 0.549 | 1.29 | 1.29 | 1.13 | 1.17 | 0.074 | 0.206 | 1.85 | 0.068 | 0.0534 | 1.79 | 0.01 | 0.744 | 0.127 | alea | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  |  | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.8e-16 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| arith_gcd_large | 24 | 52462 | >600 | >600 | oom | oom | err | >600 | oom | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | - | - | - | - | - | - | - | - | - | - | oom |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| arith_gcd_medium | 20 | 30487 | 108 | >600 | oom | oom | err | >600 | oom | >600 | >600 | oom | >600 | >600 | 278 | >600 | >600 | >600 | - | - | - | - | - | - | - | oom | oom | >600 | alea | 2.2e-16 / 3.6e-16 |  |  |  |  |  |  |  |  |  |  |  | 2.2e-16 / 3.6e-16 |  |  |  |  |  |  |  |  |  |  |  |
| arith_gcd_small | 16 | 15724 | 8.6 | >600 | oom | oom | oom | >600 | oom | >600 | 247 | oom | 157 | 174 | 30.6 | >600 | >600 | 284 | >600 | oom | oom | >600 | oom | oom | >600 | 8.15 | 9.03 | >600 | alea | 2.2e-16 / 3.6e-16 |  |  |  |  |  |  |  | 2.2e-16 / 3.6e-16 |  | 2.2e-16 / 3.6e-16 | 2.2e-16 / 3.6e-16 | 2.2e-16 / 3.6e-16 |  |  | 2.2e-16 / 3.6e-16 |  |  |  |  |  |  |  | 2.2e-16 / 3.6e-16 |
| arith_linext_large | 24 | 386 | 0.016 | >600 | err | >600 | oom | >600 | 49.6 | 15.2 | 223 | oom | >600 | >600 | 405 | >600 | >600 | 5.68 | 2.84 | 13.3 | 1.61 | 63.3 | 18.8 | 3.71 | 51.8 | 0.37 | 6.05 | >600 | alea | 4.4e-16 / 2.5e-15 |  |  |  |  |  | 4.4e-16 / 2.5e-15 | 4.4e-16 / 2.5e-15 | 5.3e-16 / 3.0e-15 |  |  |  | 4.4e-16 / 2.5e-15 |  |  | 4.4e-16 / 2.5e-15 | 4.4e-16 / 2.5e-15 | 4.3e-07 / 8.0e-06 | 4.3e-13 / 2.4e-12 | 4.3e-13 / 2.4e-12 | 4.3e-13 / 2.4e-12 | 4.2e-16 / 2.3e-15 | 4.7e-16 / 2.6e-15 | 4.4e-16 / 2.5e-15 |
| arith_linext_medium | 24 | 254 | 0.012 | >600 | err | oom | 182 | >600 | 34.9 | 1.34 | 4.75 | oom | 4.11 | 7.17 | 42.9 | 14.8 | 14.8 | 2.05 | 2.06 | 0.378 | 0.288 | 5.39 | 0.376 | 0.201 | 32.8 | 0.058 | 4.23 | 41.8 | alea | 6.1e-16 / 1.7e-15 |  |  |  | 6.7e-16 / 1.7e-15 |  | 7.2e-16 / 1.8e-15 | 1.4e-13 / 6.9e-13 | 5.6e-16 / 1.7e-15 |  | 6.7e-16 / 1.7e-15 | 6.7e-16 / 1.7e-15 | 7.8e-16 / 1.9e-15 | 7.8e-16 / 1.9e-15 | 7.8e-16 / 1.9e-15 | 6.7e-16 / 1.7e-15 | 7.2e-16 / 1.8e-15 | 3.3e-07 / 2.5e-06 | 3.3e-13 / 2.5e-12 | 3.3e-13 / 2.5e-12 | 3.3e-13 / 2.5e-12 | 6.7e-16 / 1.7e-15 | 6.7e-16 / 1.7e-15 | 7.2e-16 / 1.8e-15 |
| arith_linext_small | 8 | 56 | 0.006 | 1.58 | err | 195 | 0.037 | 11.5 | 17.3 | 1.06 | 3.26 | 0.295 | 0.685 | 0.688 | 2.07 | 5.16 | 5.18 | 1.41 | 1.46 | 0.078 | 0.195 | 1.97 | 0.078 | 0.0592 | 6.36 | 0.017 | 2.83 | 0.189 | alea | 2.2e-16 / 4.4e-16 | 8.0e-07 / 3.2e-06 |  | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 3.9e-16 / 1.1e-15 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 3.0e-16 | 2.2e-16 / 3.0e-16 | 2.2e-16 / 4.4e-16 |
| arith_luhn_medium | 36 | 485 | 0.15 | >600 | err | >600 | oom | >600 | 44.8 | 7.05 | 4.5 | oom | 328 | >600 | 79.4 | >600 | >600 | 2.36 | 2.87 | 0.181 | 0.438 | 14.5 | 0.325 | 0.466 | 59.6 | 0.368 | 4.65 | >600 | alea | 7.8e-16 / 4.6e-15 |  |  |  |  |  | 3.1e-16 / 2.1e-15 | 7.2e-14 / 7.9e-13 | 3.3e-16 / 2.9e-15 |  | 3.6e-16 / 2.7e-15 |  | 2.8e-16 / 2.6e-15 |  |  | 3.3e-16 / 2.4e-15 | 2.8e-16 / 2.1e-15 | 4.1e-07 / 4.1e-06 | 3.5e-13 / 1.8e-12 | 3.5e-13 / 1.8e-12 | 3.5e-13 / 1.8e-12 | 3.1e-16 / 2.4e-15 | 3.3e-16 / 2.4e-15 | 3.3e-16 / 2.4e-15 |
| arith_luhn_small | 24 | 328 | 0.024 | >600 | err | oom | oom | >600 | 24.1 | 1.12 | 3.66 | oom | 2.47 | 4.99 | 14.4 | 10.2 | 9.93 | 2.12 | 2.43 | 0.166 | 0.322 | 6.81 | 0.19 | 0.168 | 36.8 | 0.092 | 2.9 | 4.73 | alea | 1.7e-16 / 1.2e-15 |  |  |  |  |  | 1.9e-16 / 1.2e-15 | 1.3e-14 / 6.9e-14 | 1.1e-16 / 1.1e-15 |  | 1.7e-16 / 1.2e-15 | 1.7e-16 / 1.1e-15 | 1.7e-16 / 1.1e-15 | 1.7e-16 / 1.1e-15 | 1.7e-16 / 1.1e-15 | 1.7e-16 / 1.2e-15 | 2.2e-16 / 1.4e-15 | 4.1e-07 / 4.4e-06 | 1.9e-13 / 1.9e-12 | 1.9e-13 / 1.9e-12 | 1.9e-13 / 1.9e-12 | 1.9e-16 / 1.4e-15 | 1.7e-16 / 1.4e-15 | 1.7e-16 / 1.2e-15 |
| arith_plankton | 60 | 1747 | 0.33 | >600 | >600 | >600 | oom | >600 | 7.22 | 193 | 7.36 | oom | >600 | oom | err | oom | oom | 7.04 | 6.15 | oom | oom | >600 | oom | err | 192 | >600 | >600 | >600 | alea | 1.1e-16 / 4.3e-14 |  |  |  |  |  | 1.0e-16 / 4.4e-14 | 1.3e-16 / 1.0e+00 | 1.1e-16 / 4.3e-14 |  |  |  |  |  |  | 1.2e-16 / 4.4e-14 | 1.2e-16 / 4.4e-14 |  |  |  |  |  | 1.2e-16 / 4.3e-14 |  |
| arith_radar | 66 | 1195 | 0.107 | >600 | err | >600 | oom | >600 | 7.68 | 69.4 | 36.9 | oom | oom | err | 28 | >600 | >600 | 2.09 | 2.12 | 0.186 | 0.363 | 4.19 | 0.237 | 0.201 | 30 | 0.073 | 0.803 | 41.4 | alea | 2.8e-16 / 1.5e-15 |  |  |  |  |  | 2.5e-16 / 1.4e-15 | 6.7e-12 / 2.9e-10 | 3.1e-16 / 1.7e-15 |  |  |  | 2.5e-16 / 1.5e-15 |  |  | 2.8e-16 / 1.5e-15 | 2.8e-16 / 1.5e-15 | 4.8e-07 / 3.2e-05 | 4.8e-13 / 2.7e-12 | 4.8e-13 / 2.7e-12 | 4.8e-13 / 2.7e-12 | 2.5e-16 / 1.4e-15 | 2.8e-16 / 1.5e-15 | 2.5e-16 / 1.4e-15 |
| arith_ranking_large | 88 | 1212 | 0.044 | >600 | >600 | >600 | oom | >600 | 236 | 18.6 | 9.06 | oom | 13.9 | 20.1 | 44.9 | 42.6 | 44.4 | 2.87 | 3.21 | 0.346 | 0.601 | 6.92 | 1.01 | 0.662 | 139 | 0.52 | 3.37 | >600 | alea | 2.0e-15 / 2.8e-14 |  |  |  |  |  | 1.9e-15 / 2.8e-14 | 3.8e-08 / 2.5e-04 | 1.7e-15 / 2.8e-14 |  | 1.8e-15 / 2.8e-14 | 1.8e-15 / 2.8e-14 | 1.8e-15 / 2.8e-14 | 1.8e-15 / 2.8e-14 | 1.8e-15 / 2.8e-14 | 1.8e-15 / 2.8e-14 | 1.7e-15 / 2.8e-14 | 4.7e-07 / 1.3e-02 | 4.6e-13 / 2.2e-12 | 4.6e-13 / 2.2e-12 | 4.6e-13 / 2.2e-12 | 1.8e-15 / 2.8e-14 | 1.8e-15 / 2.8e-14 | 1.9e-15 / 2.8e-14 |
| arith_ranking_medium | 56 | 744 | 0.029 | >600 | >600 | >600 | oom | >600 | 34.9 | 6.12 | 5.24 | oom | 6.75 | 10.3 | 38.5 | 31.7 | 31.7 | 2.35 | 2.76 | 0.215 | 0.401 | 4.37 | 0.425 | 0.273 | 81 | 0.247 | 3.05 | 202 | alea | 5.6e-16 / 1.2e-14 |  |  |  |  |  | 5.6e-16 / 1.2e-14 | 8.5e-11 / 2.0e-08 | 6.1e-16 / 1.2e-14 |  | 6.1e-16 / 1.2e-14 | 6.1e-16 / 1.2e-14 | 5.6e-16 / 1.2e-14 | 5.6e-16 / 1.2e-14 | 5.6e-16 / 1.2e-14 | 6.1e-16 / 1.3e-14 | 6.1e-16 / 1.3e-14 | 5.0e-07 / 2.1e-04 | 4.4e-13 / 2.0e-12 | 4.4e-13 / 2.0e-12 | 4.4e-13 / 2.0e-12 | 5.6e-16 / 1.2e-14 | 6.1e-16 / 1.2e-14 | 6.1e-16 / 1.2e-14 |
| arith_ranking_small | 24 | 276 | 0.014 | >600 | err | oom | oom | >600 | 23.8 | 0.92 | 3.36 | oom | 1.9 | 2.86 | 24.3 | 9.92 | 10.2 | 1.99 | 2.04 | 0.115 | 0.261 | 2.66 | 0.138 | 0.112 | 28.5 | 0.048 | 2.9 | 48.5 | alea | 3.3e-16 / 1.1e-15 |  |  |  |  |  | 4.4e-16 / 1.3e-15 | 6.7e-14 / 2.3e-13 | 3.3e-16 / 9.4e-16 |  | 3.3e-16 / 9.4e-16 | 3.9e-16 / 1.1e-15 | 4.4e-16 / 1.1e-15 | 4.4e-16 / 1.1e-15 | 4.4e-16 / 1.1e-15 | 3.3e-16 / 9.5e-16 | 3.3e-16 / 9.5e-16 | 4.2e-07 / 2.8e-06 | 5.0e-13 / 1.7e-12 | 5.0e-13 / 1.7e-12 | 5.0e-13 / 1.7e-12 | 3.3e-16 / 1.3e-15 | 3.3e-16 / 9.4e-16 | 3.3e-16 / 1.1e-15 |
| arith_triangle_large | 132 | 1916 | 18.9 | >600 | oom | >600 | oom | >600 | 54 | 155 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | - | - | - | - | - | - | 22.9 | 23.9 | >600 | alea | 5.0e-15 / 7.9e-14 |  |  |  |  |  | 1.1e-16 / 1.2e-15 | 2.2e-16 / 1.7e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.2e-16 / 9.7e-16 |
| arith_triangle_medium | 57 | 1298 | 0.472 | >600 | oom | >600 | oom | >600 | 7.58 | 5.46 | 158 | >600 | >600 | err | 184 | >600 | >600 | 112 | 79.9 | - | - | - | - | - | oom | 0.81 | 1.56 | >600 | alea | 1.8e-15 / 2.9e-14 |  |  |  |  |  | 1.1e-16 / 1.4e-15 | 5.7e-17 / 1.9e-13 | 1.1e-16 / 1.4e-15 |  |  |  | 2.2e-16 / 2.2e-15 |  |  | 4.1e-13 / 4.4e-13 | 3.1e-14 / 3.3e-14 |  |  |  |  |  |  | 1.1e-16 / 1.3e-15 |
| arith_triangle_small | 48 | 1277 | 0.243 | >600 | oom | >600 | oom | >600 | 6.39 | 3.42 | 29.1 | >600 | 272 | 381 | 49.4 | >600 | >600 | 76.4 | 46.1 | oom | oom | oom | oom | err | 152 | 0.211 | 0.91 | >600 | alea | 2.8e-16 / 5.0e-15 |  |  |  |  |  | 1.4e-17 / 1.1e-15 | 3.1e-17 / 3.0e-13 | 6.9e-18 / 1.3e-15 |  | 4.4e-16 / 9.8e-16 | 6.9e-18 / 1.1e-15 | 5.4e-20 / 1.1e-15 |  |  | 1.4e-14 / 2.5e-14 | 4.0e-14 / 4.3e-14 |  |  |  |  |  | 8.7e-19 / 9.8e-16 | 1.4e-17 / 1.1e-15 |
| arith_tugofwar | 44 | 1372 | 0.107 | >600 | >600 | >600 | oom | >600 | 7.99 | 1.96 | 194 | oom | >600 | >600 | 285 | >600 | >600 | 66.9 | 82.9 | oom | oom | oom | oom | oom | oom | 0.166 | 0.869 | >600 | alea | 7.8e-16 / 8.0e-16 |  |  |  |  |  | 2.2e-16 / 2.3e-16 | 2.2e-16 / 2.3e-16 | 4.4e-16 / 4.6e-16 |  |  |  | 3.3e-16 / 3.4e-16 |  |  | 3.3e-16 / 3.4e-16 | 4.4e-16 / 4.6e-16 |  |  |  |  |  |  | 3.3e-16 / 3.4e-16 |
| hybit_GPA | 94168 | 516070 | 29.1 | oom | oom | >600 | >600 | err | oom | >600 | >600 | >600 | >600 | >600 | err | >600 | >600 | - | - | - | - | - | - | - | - | 39.1 | 42.7 | oom | alea | 1.1e-16 / 1.8e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.2e-16 / 3.6e-16 |
| hybit_addFun_max | 604 | 26275 | 16.3 | >600 | oom | oom | oom | >600 | oom | >600 | >600 | oom | >600 | >600 | >600 | >600 | >600 | - | - | - | - | - | - | - | - | oom | oom | >600 | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_addFun_sum | 196 | 3820 | 0.637 | >600 | >600 | >600 | oom | >600 | oom | 142 | >600 | oom | >600 | >600 | >600 | >600 | >600 | - | - | - | - | - | - | - | - | oom | oom | >600 | alea | 6.7e-16 / 1.3e-15 |  |  |  |  |  |  | 2.2e-16 / 4.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_altermu2 | 80350 | 457880 | oom | oom | oom | >600 | >600 | err | oom | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | - | - | - | - | - | - | - | - | >600 | >600 | >600 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_clickGraph | 208 | 765 | 0.294 | >600 | >600 | >600 | oom | >600 | oom | 510 | 4.14 | oom | >600 | >600 | >600 | 279 | 298 | - | - | - | - | - | - | - | - | oom | oom | 8.3 | problog | 4.7e-09 / 9.2e-09 |  |  |  |  |  |  | 4.7e-09 / 9.2e-09 | 4.7e-09 / 9.2e-09 |  |  |  |  | 4.7e-09 / 9.2e-09 | 4.7e-09 / 9.2e-09 |  |  |  |  |  |  |  |  |  |
| hybit_clinicalTrial1 | 2419 | 12800 | 1.8 | >600 | >600 | oom | oom | >600 | oom | >600 | 419 | oom | >600 | >600 | >600 | err | err | - | - | - | - | - | - | - | - | oom | oom | 23.7 | alea | 4.4e-16 / 2.6e-15 |  |  |  |  |  |  |  | 5.6e-16 / 3.3e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_clinicalTrial2 | 221 | 1150 | 0.034 | >600 | oom | err | oom | >600 | oom | 175 | 9.07 | oom | >600 | >600 | >600 | >600 | >600 | - | - | - | - | - | - | - | - | oom | oom | 30.9 | alea | 3.6e-15 / 7.1e-15 |  |  |  |  |  |  | 1.0e-13 / 2.1e-13 | 4.4e-16 / 9.0e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_coinBias | 37026 | 226532 | >600 | oom | oom | oom | >600 | err | oom | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | - | - | - | - | - | - | - | - | oom | oom | oom | alea |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_conjugate_gaussians | 249 | 4198 | 0.924 | >600 | oom | >600 | oom | >600 | oom | >600 | >600 | oom | >600 | >600 | err | >600 | >600 | - | - | - | - | - | - | - | - | oom | oom | >600 | alea | 2.7e-15 / 6.5e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_normal_mixture_mu1 | 5289 | 35812 | 179 | >600 | oom | oom | err | >600 | oom | >600 | >600 | oom | >600 | >600 | >600 | >600 | >600 | - | - | - | - | - | - | - | - | oom | oom | >600 | alea | 1.3e-14 / 1.2e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_normal_mixture_mu2 | 7913 | 63750 | 114 | oom | oom | oom | err | >600 | oom | >600 | oom | oom | >600 | >600 | >600 | >600 | >600 | - | - | - | - | - | - | - | - | oom | oom | >600 | alea | 1.9e-14 / 2.3e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_normal_mixture_theta | 10537 | 118560 | 19.8 | oom | oom | oom | >600 | >600 | oom | >600 | >600 | oom | >600 | >600 | >600 | >600 | >600 | - | - | - | - | - | - | - | - | oom | oom | >600 | exact | 2.2e-16 / 5.2e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_pi | 24 | 12633 | 0.939 | >600 | oom | oom | oom | >600 | oom | 33.5 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | err | oom | oom | >600 | err | oom | >600 | 12.4 | 13.4 | >600 | alea | 5.6e-17 / 1.2e-16 |  |  |  |  |  |  | 5.6e-17 / 1.2e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.6e-17 / 1.2e-16 |
| hybit_spacex | 513 | 3195 | oom | >600 | >600 | oom | oom | >600 | oom | >600 | >600 | oom | >600 | >600 | >600 | >600 | >600 | - | - | - | - | - | - | - | - | >600 | >600 | >600 | alea |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_trueskill | 376 | 5592 | 34.3 | >600 | >600 | >600 | oom | >600 | oom | >600 | >600 | oom | >600 | >600 | >600 | >600 | >600 | - | - | - | - | - | - | - | - | oom | oom | >600 | alea | 1.7e-16 / 3.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hybit_tug_of_war | 550 | 2720 | 0.424 | >600 | >600 | oom | oom | >600 | 10 | >600 | >600 | oom | >600 | >600 | 31.4 | 416 | 424 | - | - | - | - | - | - | - | - | oom | oom | >600 | tddlog | ref |  |  |  |  |  | 5.0e-16 / 1.0e-15 |  |  |  |  |  | 5.6e-16 / 1.1e-15 | 4.4e-16 / 8.9e-16 | 4.4e-16 / 8.9e-16 |  |  |  |  |  |  |  |  |  |
| hybit_weekend | 98303 | 516022 | 21 | oom | oom | >600 | >600 | err | >600 | >600 | >600 | >600 | >600 | err | >600 | >600 | >600 | - | - | - | - | - | - | - | - | 39.2 | 42.8 | oom | alea | 8.3e-16 / 2.2e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 7.8e-16 / 2.1e-15 |
| hybit_zeroone_w1 | 44 | 8780 | 0.956 | >600 | oom | oom | oom | >600 | 223 | 17.7 | 347 | >600 | >600 | >600 | >600 | >600 | >600 | - | - | - | - | - | - | - | - | 15.2 | 24.8 | >600 | alea | 3.9e-15 / 7.7e-15 |  |  |  |  |  | 8.9e-16 / 1.8e-15 | 3.2e-13 / 6.5e-13 | 1.0e-15 / 2.0e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 8.9e-16 / 1.8e-15 |
| hybit_zeroone_w2 | 50 | 13588 | 2.24 | >600 | oom | oom | err | >600 | oom | 151 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | - | - | - | - | - | - | - | - | 172 | 186 | >600 | alea | 1.1e-13 / 1.6e-13 |  |  |  |  |  |  | 5.8e-13 / 1.7e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 6.7e-16 / 1.3e-15 |
| nodice_3sat-0 |  |  | 0.396 | >600 | zero | oom | oom | >600 | 22.6 | 16.6 | 2.74 | >600 | >600 | err | 21 | >600 | >600 | >600 | >600 | oom | oom | oom | oom | oom | oom | 3.92 | 4.75 | >600 | alea | 1.3e-22 / 1.1e-15 |  |  |  |  |  | 1.3e-22 / 1.1e-15 | 1.3e-22 / 1.1e-15 | 1.3e-22 / 1.1e-15 |  |  |  | 1.3e-22 / 1.1e-15 |  |  |  |  |  |  |  |  |  |  | 1.3e-22 / 1.1e-15 |
| nodice_alarm |  |  | 0.047 | 0.769 | 465 | >600 | 1.48 | 6.04 | 9.41 | 29.2 | 8.56 | oom | 0.209 | 0.236 | 3.12 | 1.26 | 1.29 | 2.83 | 3.07 | 0.29 | 0.602 | 3.34 | 0.681 | 0.72 | 139 | 458 | 459 | 2.18 | alea | 8.3e-17 / 4.1e-16 | 5.1e-08 / 2.1e-07 | 2.4e-07 / 1.2e-06 |  | 2.2e-16 / 5.5e-16 | 2.2e-16 / 5.5e-16 | 5.6e-17 / 1.4e-16 | 3.9e-16 / 1.1e-15 | 2.2e-16 / 8.1e-16 |  | 1.7e-16 / 5.4e-16 | 1.1e-16 / 2.7e-16 | 2.2e-16 / 8.1e-16 | 5.6e-17 / 1.4e-16 | 5.6e-17 / 1.4e-16 | 5.6e-17 / 1.4e-16 | 1.7e-16 / 4.3e-16 | 2.4e-07 / 1.2e-06 | 3.1e-13 / 7.9e-13 | 3.1e-13 / 7.9e-13 | 3.1e-13 / 7.9e-13 | 2.2e-16 / 5.7e-16 | 2.2e-16 / 5.7e-16 | 1.1e-16 / 2.8e-16 |
| nodice_brp10-2 |  |  | 0.077 | 0.017 | 102 | err | 0.017 | 2.44 | 8.28 | 0.641 | 3.62 | - | 0.264 | 0.266 | 0.736 | 2.2 | 2.17 | 2.12 | 2.22 | 0.133 | 0.361 | 7.77 | 0.138 | 0.921 | 44.4 | 0.047 | 0.747 | 1.11 | alea | 1.1e-16 / 1.1e-16 | 4.4e-09 / 4.4e-09 | 4.0e-07 / 4.0e-07 |  | 2.2e-16 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 2.2e-16 / 2.2e-16 | 2.2e-16 / 2.2e-16 | 0.0e+00 / 0.0e+00 |  | 7.8e-16 / 7.8e-16 | 0.0e+00 / 0.0e+00 | 4.4e-16 / 4.4e-16 | 3.3e-16 / 3.3e-16 | 3.3e-16 / 3.3e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.1e-16 | 4.0e-07 / 4.0e-07 | 2.5e-13 / 2.5e-13 | 2.5e-13 / 2.5e-13 | 2.5e-13 / 2.5e-13 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 0.0e+00 / 0.0e+00 |
| nodice_brp1024-2 |  |  | 6.39 | 1.24 | - | err | 2.3 | >600 | 80 | >600 | >600 | - | 0.288 | 1.66 | 15.1 | 4.16 | 3.58 | 18.9 | 30.5 | 36.5 | 267 | 134 | 141 | >600 | - | 23.5 | 24.2 | - | alea | 6.7e-16 / 6.8e-16 | 4.8e-07 / 4.9e-07 |  |  | 2.0e-14 / 2.1e-14 |  | 3.9e-15 / 4.0e-15 |  |  |  | 8.7e-14 / 9.0e-14 | 1.1e-15 / 1.1e-15 | 3.0e-14 / 3.1e-14 | 2.5e-14 / 2.5e-14 | 2.5e-14 / 2.5e-14 | 1.7e-15 / 1.7e-15 | 1.3e-15 / 1.4e-15 | 1.9e-07 / 2.0e-07 | 1.1e-13 / 1.2e-13 | 1.1e-13 / 1.2e-13 | 1.1e-13 / 1.2e-13 |  |  | 1.1e-15 / 1.1e-15 |
| nodice_brp16-2 |  |  | 0.058 | 0.023 | >600 | err | 0.024 | 3.25 | 8.26 | 0.646 | 4.34 | - | 0.258 | 0.289 | 0.976 | 2.21 | 2.18 | 2.16 | 2.3 | 0.127 | 0.321 | 2.43 | 0.141 | 0.258 | 72.5 | 0.065 | 0.761 | 80.4 | alea | 0.0e+00 / 0.0e+00 | 6.7e-08 / 6.7e-08 |  |  | 5.6e-16 / 5.6e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 |  | 1.2e-15 / 1.2e-15 | 0.0e+00 / 0.0e+00 | 6.7e-16 / 6.7e-16 | 6.7e-16 / 6.7e-16 | 6.7e-16 / 6.7e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.1e-16 | 3.3e-07 / 3.3e-07 | 2.3e-13 / 2.3e-13 | 2.3e-13 / 2.3e-13 | 2.3e-13 / 2.3e-13 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| nodice_brp16384-2 |  |  | 379 | err | - | err | err | - | - | - | - | - | err | err | err | err | err | 181 | >600 | - | - | - | - | - | - | >600 | >600 | - | alea | 2.3e-13 / 3.5e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.2e-13 / 1.8e-13 |  |  |  |  |  |  |  |  |
| nodice_brp256-2 |  |  | 1.29 | 0.306 | - | err | 0.311 | 489 | 11.7 | 21.7 | 370 | - | 0.269 | 0.357 | 4.62 | 2.57 | 2.49 | 7.22 | 8.74 | 2.9 | 11.1 | 10.2 | 8.64 | 4.27 | >600 | 3.02 | 3.72 | - | alea | 1.1e-16 / 1.1e-16 | 7.2e-08 / 7.2e-08 |  |  | 5.9e-15 / 5.9e-15 | 3.3e-16 / 3.4e-16 | 7.8e-16 / 7.8e-16 | 6.7e-16 / 6.7e-16 | 2.8e-15 / 2.8e-15 |  | 2.2e-14 / 2.3e-14 | 3.3e-16 / 3.4e-16 | 8.7e-15 / 8.7e-15 | 5.7e-15 / 5.7e-15 | 5.7e-15 / 5.7e-15 | 4.4e-16 / 4.5e-16 | 4.4e-16 / 4.5e-16 | 1.3e-07 / 1.3e-07 | 4.4e-13 / 4.4e-13 | 4.4e-13 / 4.4e-13 | 4.4e-13 / 4.4e-13 | 3.3e-16 / 3.4e-16 |  | 3.3e-16 / 3.4e-16 |
| nodice_brp4-2 |  |  | 0.021 | 0.01 | 0.21 | err | 0.009 | 1.47 | 8.15 | 0.766 | 3.12 | >600 | 0.293 | 0.271 | 0.44 | 2.16 | 2.22 | 1.62 | 1.74 | 0.08 | 0.23 | 2.15 | 0.08 | 0.1 | 15.5 | 0.01 | 0.724 | 0.223 | alea | 1.1e-16 / 1.1e-16 | 5.0e-08 / 5.0e-08 | 8.5e-07 / 8.5e-07 |  | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 |  | 3.3e-16 / 3.3e-16 | 0.0e+00 / 0.0e+00 | 2.2e-16 / 2.2e-16 | 2.2e-16 / 2.2e-16 | 2.2e-16 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 2.2e-16 / 2.2e-16 | 1.5e-07 / 1.5e-07 | 1.4e-13 / 1.4e-13 | 1.4e-13 / 1.4e-13 | 1.4e-13 / 1.4e-13 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 0.0e+00 / 0.0e+00 |
| nodice_brp4096-2 |  |  | 35.6 | wrong | - | err | err | - | err | - | - | - | 0.395 | 45 | 124 | 25.1 | 29.1 | 59.5 | 205 | err | >600 | >600 | >600 | - | - | 160 | 162 | - | alea | 1.1e-14 / 1.2e-14 | 1.6e-06 / 1.7e-06 |  |  |  |  |  |  |  |  | 3.2e-13 / 3.5e-13 | 3.1e-15 / 3.5e-15 | 1.2e-13 / 1.4e-13 | 1.0e-13 / 1.2e-13 | 1.0e-13 / 1.2e-13 | 7.8e-16 / 8.7e-16 | 1.4e-14 / 1.6e-14 |  |  |  |  |  |  | 2.9e-15 / 3.2e-15 |
| nodice_brp6-2 |  |  | 0.033 | 0.013 | 0.343 | err | 0.011 | 1.76 | 7.98 | 0.645 | 3.9 | - | 0.291 | 0.279 | 0.544 | 2.28 | 2.33 | 1.95 | 2 | 0.102 | 0.275 | 2.23 | 0.102 | 0.167 | 24.7 | 0.028 | 0.735 | 0.289 | alea | 4.4e-16 / 4.4e-16 | 2.9e-08 / 2.9e-08 | 2.3e-07 / 2.3e-07 |  | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 |  | 4.4e-16 / 4.4e-16 | 0.0e+00 / 0.0e+00 | 2.2e-16 / 2.2e-16 | 2.2e-16 / 2.2e-16 | 2.2e-16 / 2.2e-16 | 1.1e-16 / 1.1e-16 | 0.0e+00 / 0.0e+00 | 2.3e-07 / 2.3e-07 | 4.5e-13 / 4.5e-13 | 4.5e-13 / 4.5e-13 | 4.5e-13 / 4.5e-13 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 0.0e+00 / 0.0e+00 |
| nodice_brp64-2 |  |  | 0.281 | wrong | - | err | 0.081 | 20.3 | 8.43 | 1.01 | 24 | - | 0.289 | 0.272 | 2.47 | 2.22 | 2.26 | 3.25 | 3.65 | 0.375 | 0.977 | 2.88 | 0.694 | 1.04 | 333 | 0.492 | 1.18 | >600 | alea | 0.0e+00 / 0.0e+00 | 1.2e-06 / 1.2e-06 |  |  | 1.7e-15 / 1.7e-15 | 1.1e-16 / 1.1e-16 | 3.3e-16 / 3.3e-16 | 3.3e-16 / 3.3e-16 | 4.4e-16 / 4.4e-16 |  | 5.0e-15 / 5.0e-15 | 1.1e-16 / 1.1e-16 | 1.8e-15 / 1.8e-15 | 1.6e-15 / 1.6e-15 | 1.6e-15 / 1.6e-15 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 2.6e-07 / 2.6e-07 | 3.0e-13 / 3.0e-13 | 3.0e-13 / 3.0e-13 | 3.0e-13 / 3.0e-13 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 |
| nodice_brp8-2 |  |  | 0.054 | 0.015 | 2.46 | err | 0.014 | 1.97 | 8.54 | 0.641 | 3.42 | - | 0.31 | 0.28 | 0.662 | 2.31 | 2.34 | 2 | 2.18 | 0.117 | 0.29 | 2.38 | 0.119 | 0.247 | 34.4 | 0.034 | 0.741 | 0.462 | alea | 0.0e+00 / 0.0e+00 | 1.1e-08 / 1.1e-08 | 6.9e-07 / 6.9e-07 |  | 2.2e-16 / 2.2e-16 | 1.1e-16 / 1.1e-16 | 2.2e-16 / 2.2e-16 | 2.2e-16 / 2.2e-16 | 3.3e-16 / 3.3e-16 |  | 5.6e-16 / 5.6e-16 | 0.0e+00 / 0.0e+00 | 4.4e-16 / 4.4e-16 | 3.3e-16 / 3.3e-16 | 3.3e-16 / 3.3e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.1e-16 | 3.1e-07 / 3.1e-07 | 3.0e-14 / 3.0e-14 | 3.0e-14 / 3.0e-14 | 3.0e-14 / 3.0e-14 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| nodice_coupon2-10-obs |  |  | 0.1 | oom | 4.27 | err | oom | >600 | 14.8 | 4.55 | >600 | oom | 6.95 | 18.1 | 3.5 | 3.49 | 1.88 | 17.8 | 26.4 | oom | oom | 14.7 | oom | err | 146 | 1.55 | 2.28 | >600 | alea | 2.2e-16 / 8.5e-16 |  | 5.2e-07 / 2.0e-06 |  |  |  | 2.2e-16 / 8.5e-16 | 1.5e-15 / 5.7e-15 |  |  | 1.7e-16 / 6.3e-16 | 2.2e-16 / 8.5e-16 | 1.7e-16 / 6.3e-16 | 1.7e-16 / 6.3e-16 | 1.7e-16 / 6.3e-16 | 5.6e-17 / 2.1e-16 | 5.6e-17 / 2.1e-16 |  |  | 3.2e-13 / 1.2e-12 |  |  | 3.3e-16 / 1.3e-15 | 2.2e-16 / 8.5e-16 |
| nodice_coupon2-11-obs |  |  | 0.198 | >600 | 12.2 | err | oom | >600 | 25.1 | 13.1 | >600 | oom | 20.9 | 58.8 | 5.17 | 8.21 | 3.45 | 51.4 | 66.8 | - | - | 9.17 | - | err | 203 | 6.78 | 7.53 | >600 | alea | 2.8e-16 / 1.3e-15 |  | 7.0e-07 / 3.2e-06 |  |  |  | 2.5e-16 / 1.1e-15 | 1.1e-16 / 5.1e-16 |  |  | 1.1e-16 / 5.1e-16 | 2.2e-16 / 1.0e-15 | 3.3e-16 / 1.5e-15 | 1.1e-16 / 5.1e-16 | 1.1e-16 / 5.1e-16 | 8.3e-17 / 3.8e-16 | 8.0e-16 / 3.7e-15 |  |  | 1.6e-13 / 7.1e-13 |  |  | 1.9e-16 / 8.8e-16 | 1.4e-16 / 6.3e-16 |
| nodice_coupon2-12-obs |  |  | 0.174 | >600 | 34.8 | err | oom | >600 | 60 | 63.2 | >600 | oom | 62.3 | 172 | 5.88 | 21.3 | 7.87 | >600 | >600 | - | - | 11.2 | - | err | 218 | 51.2 | 51.9 | >600 | alea | 6.7e-16 / 3.6e-15 |  | 1.4e-07 / 7.6e-07 |  |  |  | 6.7e-16 / 3.6e-15 | 1.7e-15 / 9.1e-15 |  |  | 3.9e-16 / 2.1e-15 | 6.7e-16 / 3.6e-15 | 7.2e-16 / 3.9e-15 | 3.9e-16 / 2.1e-15 | 3.9e-16 / 2.1e-15 |  |  |  |  | 1.8e-13 / 1.0e-12 |  |  | 7.2e-16 / 3.9e-15 | 6.7e-16 / 3.6e-15 |
| nodice_coupon2-13-obs |  |  | 0.581 | >600 | 97 | err | oom | >600 | err | 211 | >600 | oom | 182 | err | 7.42 | 60.3 | 20.8 | - | - | - | - | 146 | - | err | 295 | 593 | 594 | >600 | alea | 5.0e-16 / 3.3e-15 |  | 2.8e-07 / 1.8e-06 |  |  |  |  | 2.4e-15 / 1.6e-14 |  |  | 5.8e-16 / 3.8e-15 |  | 6.4e-16 / 4.2e-15 | 5.8e-16 / 3.8e-15 | 5.8e-16 / 3.8e-15 |  |  |  |  | 1.0e-13 / 6.8e-13 |  |  | 5.3e-16 / 3.4e-15 | 5.3e-16 / 3.4e-15 |
| nodice_coupon2-14-obs |  |  | 0.744 | >600 | 259 | err | oom | >600 | err | >600 | >600 | oom | 502 | oom | 8.32 | 155 | 56.4 | - | - | - | - | 121 | - | err | 333 | >600 | >600 | >600 | alea | 2.2e-16 / 1.7e-15 |  | 2.7e-07 / 2.1e-06 |  |  |  |  |  |  |  | 2.8e-17 / 2.2e-16 |  | 5.6e-17 / 4.3e-16 | 2.8e-17 / 2.2e-16 | 2.8e-17 / 2.2e-16 |  |  |  |  | 2.0e-14 / 1.6e-13 |  |  | 2.2e-16 / 1.7e-15 |  |
| nodice_coupon2-16-obs |  |  | 5.84 | - | >600 | err | - | - | - | - | - | >600 | >600 | - | 13.5 | oom | 465 | - | - | - | - | oom | - | - | >600 | - | - | >600 | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  | 1.6e-15 / 1.8e-14 |  | 1.7e-15 / 1.9e-14 |  |  |  |  |  |  |  |  |  |
| nodice_coupon2-18-obs |  |  | 65.5 | - | oom | err | - | - | - | - | - | oom | - | - | 18 | - | oom | - | - | - | - | - | - | - | - | - | - | >600 | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  | 9.2e-16 / 1.5e-14 |  |  |  |  |  |  |  |  |  |  |  |
| nodice_coupon2-2-obs |  |  | 0.006 | 0.013 | 0.037 | err | 0.005 | 1.06 | 6.91 | 0.566 | 2.9 | 0.028 | 0.277 | 0.27 | 0.304 | 2.14 | 2.16 | 1.41 | 1.41 | 0.068 | 0.214 | 2.1 | 0.068 | 0.0658 | 1.56 | 0.003 | 0.718 | 0.128 | alea | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| nodice_coupon2-20-obs |  |  | oom | - | oom | err | - | - | - | - | - | oom | - | - | 26.2 | - | - | - | - | - | - | - | - | - | - | - | - | >600 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| nodice_coupon2-3-obs |  |  | 0.009 | 0.028 | 0.04 | err | 0.007 | 2.27 | 8.82 | 0.599 | 2.91 | >600 | 0.286 | 0.281 | 0.428 | 2.29 | 2.31 | 1.71 | 1.73 | 0.073 | 0.218 | 2.12 | 0.074 | 0.089 | 11.8 | 0.005 | 0.716 | 0.189 | alea | 2.2e-16 / 2.5e-16 | 2.2e-16 / 2.5e-16 | 1.1e-07 / 1.3e-07 |  | 3.3e-16 / 3.7e-16 | 2.2e-16 / 2.5e-16 | 2.2e-16 / 2.5e-16 | 0.0e+00 / 0.0e+00 | 2.2e-16 / 2.5e-16 |  | 0.0e+00 / 0.0e+00 | 2.2e-16 / 2.5e-16 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 2.2e-16 / 2.5e-16 | 2.2e-16 / 2.5e-16 | 1.1e-07 / 1.3e-07 | 1.1e-13 / 1.3e-13 | 1.1e-13 / 1.3e-13 | 1.1e-13 / 1.3e-13 | 2.2e-16 / 2.5e-16 | 2.2e-16 / 2.5e-16 | 2.2e-16 / 2.5e-16 |
| nodice_coupon2-4-obs |  |  | 0.01 | 0.087 | 0.052 | err | 0.156 | >600 | 10.8 | 0.614 | 2.98 | - | 0.277 | 0.317 | 0.591 | 2.42 | 2.42 | 1.92 | 2.04 | 0.102 | 0.23 | 2.09 | 0.092 | 0.184 | 25.4 | 0.005 | 0.705 | 1.42 | alea | 2.2e-16 / 2.9e-16 | 2.0e-08 / 2.7e-08 | 3.7e-07 / 4.9e-07 |  | 2.2e-16 / 2.9e-16 |  | 2.2e-16 / 2.9e-16 | 2.2e-16 / 2.9e-16 | 2.2e-16 / 2.9e-16 |  | 2.2e-16 / 2.9e-16 | 2.2e-16 / 2.9e-16 | 2.2e-16 / 2.9e-16 | 2.2e-16 / 2.9e-16 | 2.2e-16 / 2.9e-16 | 2.2e-16 / 2.9e-16 | 2.2e-16 / 2.9e-16 | 3.7e-07 / 4.9e-07 | 3.7e-13 / 4.9e-13 | 3.7e-13 / 4.9e-13 | 3.7e-13 / 4.9e-13 | 2.2e-16 / 2.9e-16 | 2.2e-16 / 2.9e-16 | 2.2e-16 / 2.9e-16 |
| nodice_coupon2-5-obs |  |  | 0.015 | 1.72 | 0.101 | err | 34.7 | - | 10.3 | 0.653 | 3.27 | - | 0.289 | 0.372 | 0.865 | 2.42 | 2.3 | 2.35 | 2.44 | 0.237 | 0.253 | 2.08 | 0.13 | 0.199 | 52.2 | 0.011 | 0.702 | 189 | alea | 1.1e-16 / 1.7e-16 | 8.5e-08 / 1.3e-07 | 1.1e-16 / 1.7e-16 |  | 1.1e-16 / 1.7e-16 |  | 1.1e-16 / 1.7e-16 | 4.4e-16 / 7.0e-16 | 1.1e-16 / 1.7e-16 |  | 1.1e-16 / 1.7e-16 | 1.1e-16 / 1.7e-16 | 1.1e-16 / 1.7e-16 | 1.1e-16 / 1.7e-16 | 1.1e-16 / 1.7e-16 | 1.1e-16 / 1.7e-16 | 1.0e-15 / 1.6e-15 | 1.1e-16 / 1.7e-16 | 1.1e-16 / 1.7e-16 | 1.1e-16 / 1.7e-16 | 1.1e-16 / 1.7e-16 | 1.1e-16 / 1.7e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.7e-16 |
| nodice_coupon2-6-obs |  |  | 0.018 | 5.53 | 0.191 | err | err | - | 10.6 | 0.692 | 3.54 | - | 0.351 | 0.52 | 1.08 | 2.49 | 2.35 | 3.01 | 3.26 | 0.693 | 1.34 | 2.23 | 0.204 | 0.429 | 67.5 | 0.018 | 0.719 | >600 | alea | 2.2e-16 / 4.2e-16 | 8.3e-08 / 1.6e-07 | 3.5e-07 / 6.5e-07 |  |  |  | 1.1e-16 / 2.1e-16 | 3.3e-16 / 6.2e-16 | 1.1e-16 / 2.1e-16 |  | 1.1e-16 / 2.1e-16 | 1.1e-16 / 2.1e-16 | 2.2e-16 / 4.2e-16 | 1.1e-16 / 2.1e-16 | 1.1e-16 / 2.1e-16 | 3.3e-16 / 6.2e-16 | 1.1e-16 / 2.1e-16 | 3.5e-07 / 6.5e-07 | 1.2e-14 / 2.3e-14 | 1.2e-14 / 2.3e-14 | 1.2e-14 / 2.3e-14 | 1.1e-16 / 2.1e-16 | 2.2e-16 / 4.2e-16 | 2.2e-16 / 4.2e-16 |
| nodice_coupon2-7-obs |  |  | 0.03 | 17.5 | 0.439 | err | - | - | 10.8 | 0.917 | 9.51 | - | 0.578 | 1.24 | 1.49 | 2.65 | 2.51 | 3.98 | 4.32 | 3.17 | 0.614 | 2.38 | 0.354 | 5.73 | 116 | 0.06 | 0.757 | - | alea | 4.4e-16 / 9.9e-16 | 2.9e-07 / 6.4e-07 | 1.9e-07 / 4.2e-07 |  |  |  | 4.4e-16 / 9.9e-16 | 2.2e-16 / 5.0e-16 | 6.1e-16 / 1.4e-15 |  | 2.2e-16 / 5.0e-16 | 4.4e-16 / 9.9e-16 | 4.4e-16 / 9.9e-16 | 2.2e-16 / 5.0e-16 | 2.2e-16 / 5.0e-16 | 4.4e-16 / 9.9e-16 | 8.3e-16 / 1.9e-15 | 1.9e-07 / 4.2e-07 | 4.0e-13 / 8.9e-13 | 4.0e-13 / 8.9e-13 | 4.0e-13 / 8.9e-13 | 3.9e-16 / 8.7e-16 | 2.8e-16 / 6.2e-16 | 5.0e-16 / 1.1e-15 |
| nodice_coupon2-8-obs |  |  | 0.03 | 51 | 1.12 | err | - | - | 11.3 | 1.04 | 51.5 | - | 1.4 | 3.44 | 1.88 | 2.98 | 2.74 | 126 | >600 | err | err | 69.9 | err | oom | 159 | 0.132 | 0.834 | - | alea | 4.4e-16 / 1.2e-15 | 2.7e-07 / 7.1e-07 | 5.8e-07 / 1.5e-06 |  |  |  | 4.4e-16 / 1.2e-15 | 4.4e-16 / 1.2e-15 | 3.9e-16 / 1.0e-15 |  | 4.4e-16 / 1.2e-15 | 4.4e-16 / 1.2e-15 | 4.4e-16 / 1.2e-15 | 4.4e-16 / 1.2e-15 | 4.4e-16 / 1.2e-15 | 4.4e-16 / 1.2e-15 |  |  |  | 3.3e-13 / 8.9e-13 |  |  | 5.6e-16 / 1.5e-15 | 4.4e-16 / 1.2e-15 |
| nodice_coupon2-9-obs |  |  | 0.058 | >600 | 3.52 | err | - | - | 13.7 | 1.71 | 304 | - | 4.21 | 12.2 | 2.62 | 4.08 | 3.31 | 14.1 | - | - | - | 3.04 | - | - | 206 | 0.413 | 1.11 | - | alea | 5.0e-16 / 1.6e-15 |  | 1.1e-07 / 3.4e-07 |  |  |  | 6.7e-16 / 2.1e-15 | 2.1e-15 / 6.7e-15 | 3.9e-16 / 1.2e-15 |  | 6.1e-16 / 1.9e-15 | 5.6e-16 / 1.8e-15 | 5.0e-16 / 1.6e-15 | 6.1e-16 / 1.9e-15 | 6.1e-16 / 1.9e-15 | 5.6e-16 / 1.8e-15 |  |  |  | 3.2e-13 / 1.0e-12 |  |  | 5.0e-16 / 1.6e-15 | 4.4e-16 / 1.4e-15 |
| nodice_diamond-1-p |  |  | 0.006 | 0.026 | 0.035 | err | 0.006 | 1.06 | 6.69 | 0.567 | 2.87 | 0.016 | 0.268 | 0.266 | 0.264 | 2.16 | 2.2 | 1.3 | 1.31 | 0.071 | 0.217 | 2.16 | 0.072 | 0.0687 | 2.68 | 0.003 | 0.721 | 0.121 | alea | 1.1e-16 / 1.1e-16 | 2.5e-09 / 2.5e-09 | 2.5e-09 / 2.5e-09 |  | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 3.3e-16 / 3.3e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 2.5e-09 / 2.5e-09 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 | 1.1e-16 / 1.1e-16 |
| nodice_diamond-100-p |  |  | 0.023 | 0.036 | 0.086 | err | 0.015 | 6.44 | 10.3 | 1.96 | 3.12 | >600 | 0.299 | 0.303 | 2.06 | 2.6 | 2.69 | 2.36 | 2.62 | 0.13 | 0.355 | 2.33 | 0.154 | 0.364 | 108 | 0.05 | 0.786 | 0.685 | alea | 4.8e-15 / 4.8e-15 | 7.0e-07 / 7.1e-07 | 6.0e-07 / 6.1e-07 |  | 4.0e-15 / 4.0e-15 | 4.2e-15 / 4.2e-15 | 4.4e-15 / 4.5e-15 | 4.7e-15 / 4.7e-15 | 4.2e-15 / 4.2e-15 |  | 1.5e-14 / 1.5e-14 | 4.8e-15 / 4.8e-15 | 4.4e-15 / 4.5e-15 | 4.4e-15 / 4.5e-15 | 4.4e-15 / 4.5e-15 | 5.0e-15 / 5.0e-15 | 4.9e-15 / 4.9e-15 | 4.0e-07 / 4.0e-07 | 2.5e-13 / 2.5e-13 | 2.5e-13 / 2.5e-13 | 2.5e-13 / 2.5e-13 | 4.7e-15 / 4.7e-15 | 4.7e-15 / 4.7e-15 | 4.7e-15 / 4.7e-15 |
| nodice_diamond-1000-p |  |  | 0.149 | wrong | 2.05 | err | 0.118 | 106 | 22.4 | - | 9.08 | - | 1.17 | 1.24 | 112 | 22.4 | 21 | 6.17 | 8.52 | 2.25 | 9.09 | 8.35 | 6.08 | 3.16 | - | 3.36 | 4.16 | 89.8 | alea | 4.6e-14 / 4.8e-14 | 7.8e-06 / 8.2e-06 | 3.3e-07 / 3.4e-07 |  | 3.4e-14 / 3.5e-14 | 4.0e-14 / 4.2e-14 | 4.0e-14 / 4.2e-14 |  | 4.2e-14 / 4.4e-14 |  | 1.5e-13 / 1.6e-13 | 4.6e-14 / 4.8e-14 | 4.0e-14 / 4.2e-14 | 4.0e-14 / 4.2e-14 | 4.0e-14 / 4.2e-14 | 4.5e-14 / 4.7e-14 | 4.3e-14 / 4.5e-14 | 3.3e-07 / 3.4e-07 | 3.1e-13 / 3.3e-13 | 3.1e-13 / 3.3e-13 | 3.1e-13 / 3.3e-13 | 4.3e-14 / 4.5e-14 |  | 4.6e-14 / 4.8e-14 |
| nodice_diamond-10000-p |  |  | 2.23 | wrong | 465 | err | err | - | - | - | >600 | - | 63.2 | 84.8 | - | - | - | 24.1 | wrong | err | >600 | 542 | 537 | - | - | - | - | oom | alea | 3.0e-14 / 5.0e-14 | 5.3e-05 / 8.8e-05 | 2.5e-07 / 4.1e-07 |  |  |  |  |  |  |  | 7.7e-13 / 1.3e-12 | 3.1e-14 / 5.1e-14 |  |  |  | 2.9e-14 / 4.9e-14 | inf / inf |  |  | 1.8e-13 / 3.0e-13 | 1.8e-13 / 3.0e-13 |  |  |  |
| nodice_diamond-200-p |  |  | 0.037 | wrong | 0.165 | err | 0.028 | 13.3 | 10.6 | 23.6 | 3.5 | - | 0.371 | 0.367 | 5.58 | 3.24 | 3.18 | 2.83 | 3.18 | 0.223 | 0.627 | 2.55 | 0.366 | 0.71 | 247 | 0.126 | 0.83 | 2.03 | alea | 9.5e-15 / 9.6e-15 | 1.4e-06 / 1.4e-06 | 8.4e-08 / 8.5e-08 |  | 6.8e-15 / 6.8e-15 | 8.3e-15 / 8.4e-15 | 8.1e-15 / 8.2e-15 | 9.2e-15 / 9.3e-15 | 9.2e-15 / 9.3e-15 |  | 3.0e-14 / 3.0e-14 | 9.5e-15 / 9.6e-15 | 8.1e-15 / 8.2e-15 | 8.1e-15 / 8.2e-15 | 8.1e-15 / 8.2e-15 | 8.8e-15 / 8.9e-15 | 9.0e-15 / 9.1e-15 | 8.4e-08 / 8.5e-08 | 1.9e-13 / 1.9e-13 | 1.9e-13 / 1.9e-13 | 1.9e-13 / 1.9e-13 | 9.3e-15 / 9.4e-15 | 9.3e-15 / 9.4e-15 | 9.4e-15 / 9.5e-15 |
| nodice_diamond-2000-p |  |  | 0.347 | wrong | 9.7 | err | 0.236 | 361 | 62.1 | - | 22.6 | - | 2.8 | 3.17 | 436 | 88.6 | 82.8 | 9.54 | wrong | 7.97 | 47.1 | 24.1 | 24.2 | >600 | - | 14.4 | 15.4 | oom | alea | 8.7e-14 / 9.6e-14 | 1.5e-05 / 1.7e-05 | 9.1e-07 / 1.0e-06 |  | 6.3e-14 / 7.0e-14 | 7.6e-14 / 8.4e-14 | 7.7e-14 / 8.5e-14 |  | 7.8e-14 / 8.6e-14 |  | 2.8e-13 / 3.1e-13 | 8.7e-14 / 9.6e-14 | 7.7e-14 / 8.5e-14 | 7.7e-14 / 8.5e-14 | 7.7e-14 / 8.5e-14 | 8.2e-14 / 9.0e-14 | inf / inf | 8.6e-08 / 9.5e-08 | 1.3e-13 / 1.4e-13 | 1.3e-13 / 1.4e-13 | 1.3e-13 / 1.4e-13 |  |  | 8.8e-14 / 9.7e-14 |
| nodice_diamond-300-p |  |  | 0.066 | wrong | 0.281 | err | 0.04 | 20.7 | 11.3 | 106 | 3.95 | - | 0.418 | 0.468 | 11.9 | 4.19 | 4.07 | 3.25 | 3.67 | 0.329 | 1.02 | 2.89 | 0.665 | 1.49 | 357 | 0.268 | 1 | 4.65 | alea | 1.4e-14 / 1.4e-14 | 2.1e-06 / 2.1e-06 | 3.1e-07 / 3.2e-07 |  | 1.0e-14 / 1.0e-14 | 1.3e-14 / 1.3e-14 | 1.2e-14 / 1.2e-14 | 1.3e-14 / 1.3e-14 | 1.3e-14 / 1.3e-14 |  | 4.6e-14 / 4.6e-14 | 1.4e-14 / 1.4e-14 | 1.2e-14 / 1.2e-14 | 1.2e-14 / 1.2e-14 | 1.2e-14 / 1.2e-14 | 1.4e-14 / 1.4e-14 | 1.4e-14 / 1.4e-14 | 3.1e-07 / 3.2e-07 | 3.5e-13 / 3.5e-13 | 3.5e-13 / 3.5e-13 | 3.5e-13 / 3.5e-13 | 1.4e-14 / 1.4e-14 | 1.4e-14 / 1.4e-14 | 1.4e-14 / 1.4e-14 |
| nodice_diamond-3000-p |  |  | 0.539 | wrong | 25 | err | err | >600 | 142 | - | 50 | - | 5.17 | 6.21 | >600 | 212 | 211 | 14.3 | wrong | 17.4 | 240 | 88.7 | 74.6 | - | - | 32.1 | 33.6 | oom | alea | 1.3e-13 / 1.5e-13 | 2.2e-05 / 2.6e-05 | 2.9e-07 / 3.3e-07 |  |  |  | 1.1e-13 / 1.3e-13 |  | 1.1e-13 / 1.3e-13 |  | 4.1e-13 / 4.8e-13 | 1.2e-13 / 1.4e-13 |  | 1.1e-13 / 1.3e-13 | 1.1e-13 / 1.3e-13 | 1.2e-13 / 1.4e-13 | inf / inf | 2.9e-07 / 3.3e-07 | 3.0e-13 / 3.4e-13 | 3.0e-13 / 3.4e-13 | 3.0e-13 / 3.4e-13 |  |  | 1.2e-13 / 1.4e-13 |
| nodice_diamond-400-p |  |  | 0.07 | wrong | 0.415 | err | 0.05 | 30.5 | 12.6 | 340 | 4.49 | - | 0.506 | 0.523 | 18 | 5.11 | 5.3 | 3.74 | 4.63 | 0.51 | 1.61 | 3.26 | 1.1 | 0.995 | 475 | 0.465 | 1.2 | 9.08 | alea | 1.9e-14 / 1.9e-14 | 3.1e-06 / 3.1e-06 | 1.7e-07 / 1.8e-07 |  | 1.3e-14 / 1.3e-14 | 1.7e-14 / 1.7e-14 | 1.6e-14 / 1.7e-14 | 1.3e-14 / 1.3e-14 | 1.8e-14 / 1.8e-14 |  | 6.0e-14 / 6.1e-14 | 1.9e-14 / 1.9e-14 | 1.6e-14 / 1.7e-14 | 1.6e-14 / 1.7e-14 | 1.6e-14 / 1.7e-14 | 1.8e-14 / 1.8e-14 | 1.9e-14 / 1.9e-14 | 1.7e-07 / 1.8e-07 | 6.3e-14 / 6.4e-14 | 6.3e-14 / 6.4e-14 | 6.3e-14 / 6.4e-14 | 1.8e-14 / 1.9e-14 | 1.8e-14 / 1.9e-14 | 1.9e-14 / 1.9e-14 |
| nodice_diamond-4000-p |  |  | 0.827 | wrong | 50.3 | err | err | - | 244 | - | 81.2 | - | 8.34 | 10.4 | - | 396 | 389 | 19 | wrong | 31.4 | 306 | 103 | 91.2 | - | - | 61.9 | 64.1 | oom | alea | 1.6e-13 / 1.9e-13 | 2.8e-05 / 3.4e-05 | 2.8e-07 / 3.4e-07 |  |  |  | 1.4e-13 / 1.7e-13 |  | 1.4e-13 / 1.7e-13 |  | 5.3e-13 / 6.4e-13 | 1.6e-13 / 1.9e-13 |  | 1.4e-13 / 1.7e-13 | 1.4e-13 / 1.7e-13 | 1.5e-13 / 1.9e-13 | inf / inf | 2.8e-07 / 3.4e-07 | 1.8e-13 / 2.2e-13 | 1.8e-13 / 2.2e-13 | 1.8e-13 / 2.2e-13 |  |  | 1.5e-13 / 1.9e-13 |
| nodice_diamond-500-p |  |  | 0.084 | wrong | 0.573 | err | 0.061 | 40.9 | 13.4 | >600 | 5.01 | - | 0.601 | 0.584 | 28.1 | 7.05 | 7.09 | 4.05 | 5.19 | 0.697 | 2.39 | 3.85 | 1.63 | 1.29 | >600 | 0.766 | 1.5 | 15.3 | alea | 2.4e-14 / 2.4e-14 | 3.8e-06 / 3.9e-06 | 5.4e-07 / 5.5e-07 |  | 1.6e-14 / 1.7e-14 | 2.1e-14 / 2.1e-14 | 2.1e-14 / 2.1e-14 |  | 2.2e-14 / 2.3e-14 |  | 7.4e-14 / 7.6e-14 | 2.4e-14 / 2.4e-14 | 2.1e-14 / 2.1e-14 | 2.1e-14 / 2.1e-14 | 2.1e-14 / 2.1e-14 | 2.3e-14 / 2.4e-14 | 2.3e-14 / 2.3e-14 | 4.6e-07 / 4.7e-07 | 4.1e-13 / 4.2e-13 | 4.1e-13 / 4.2e-13 | 4.1e-13 / 4.2e-13 | 2.3e-14 / 2.3e-14 |  | 2.3e-14 / 2.4e-14 |
| nodice_diamond-5000-p |  |  | 0.844 | wrong | 86.5 | err | err | - | err | - | 173 | - | 13.5 | 17.6 | - | >600 | >600 | 16.7 | wrong | 41.5 | 262 | 125 | 143 | - | - | oom | oom | oom | alea | 1.9e-13 / 2.4e-13 | 3.3e-05 / 4.3e-05 | 2.4e-08 / 3.1e-08 |  |  |  |  |  | 1.7e-13 / 2.2e-13 |  | 6.3e-13 / 8.1e-13 | 1.9e-13 / 2.4e-13 |  |  |  | 1.8e-13 / 2.4e-13 | inf / inf | 2.4e-08 / 3.1e-08 | 1.2e-13 / 1.5e-13 | 1.2e-13 / 1.5e-13 | 1.2e-13 / 1.5e-13 |  |  |  |
| nodice_diamond-700-p |  |  | 0.111 | wrong | 1.06 | err | 0.085 | 68.7 | 16.5 | - | 6.38 | - | 0.817 | 0.832 | 55.7 | 11.2 | 11.7 | 5.02 | 7.12 | 1.21 | 4.54 | 5.1 | 3.07 | 1.94 | - | 1.61 | 2.35 | 36 | alea | 3.2e-14 / 3.4e-14 | 5.3e-06 / 5.5e-06 | 7.1e-07 / 7.3e-07 |  | 2.3e-14 / 2.4e-14 | 2.8e-14 / 2.9e-14 | 2.9e-14 / 3.0e-14 |  | 3.0e-14 / 3.1e-14 |  | 1.0e-13 / 1.1e-13 | 3.2e-14 / 3.4e-14 | 2.9e-14 / 3.0e-14 | 2.9e-14 / 3.0e-14 | 2.9e-14 / 3.0e-14 | 3.2e-14 / 3.3e-14 | 3.1e-14 / 3.2e-14 | 2.9e-07 / 3.0e-07 | 5.0e-13 / 5.2e-13 | 5.0e-13 / 5.2e-13 | 5.0e-13 / 5.2e-13 | 3.1e-14 / 3.2e-14 |  | 3.2e-14 / 3.3e-14 |
| nodice_diamond-800-p |  |  | 0.131 | wrong | 1.41 | err | 0.116 | 80.3 | 19.7 | - | 7.22 | - | 0.878 | 0.923 | 70.6 | 14.3 | 14 | 5.37 | 6.76 | 1.47 | 5.74 | 5.9 | 3.93 | 2.32 | - | 1.99 | 2.74 | 51 | alea | 3.7e-14 / 3.8e-14 | 6.0e-06 / 6.3e-06 | 5.6e-07 / 5.8e-07 |  | 2.6e-14 / 2.7e-14 | 3.2e-14 / 3.4e-14 | 3.2e-14 / 3.4e-14 |  | 3.4e-14 / 3.5e-14 |  | 1.2e-13 / 1.2e-13 | 3.7e-14 / 3.8e-14 | 3.2e-14 / 3.4e-14 | 3.2e-14 / 3.4e-14 | 3.2e-14 / 3.4e-14 | 3.6e-14 / 3.8e-14 | 3.7e-14 / 3.8e-14 | 4.4e-07 / 4.6e-07 | 4.6e-13 / 4.8e-13 | 4.6e-13 / 4.8e-13 | 4.6e-13 / 4.8e-13 | 3.5e-14 / 3.7e-14 |  | 3.7e-14 / 3.8e-14 |
| nodice_diamond-900-p |  |  | 0.131 | wrong | 1.69 | err | 0.108 | 90 | 21.6 | - | 8.04 | - | 1.04 | 1.1 | 90.5 | 17.8 | 17.9 | 5.73 | 8.03 | 1.94 | 7.97 | 7.43 | 5.01 | 2.6 | - | 2.46 | 3.19 | 69.4 | alea | 4.1e-14 / 4.3e-14 | 6.8e-06 / 7.1e-06 | 6.1e-07 / 6.3e-07 |  | 3.0e-14 / 3.1e-14 | 3.6e-14 / 3.8e-14 | 3.6e-14 / 3.8e-14 |  | 3.8e-14 / 3.9e-14 |  | 1.3e-13 / 1.4e-13 | 4.1e-14 / 4.3e-14 | 3.6e-14 / 3.8e-14 | 3.6e-14 / 3.8e-14 | 3.6e-14 / 3.8e-14 | 4.0e-14 / 4.2e-14 | 4.1e-14 / 4.3e-14 | 3.9e-07 / 4.1e-07 | 4.1e-13 / 4.3e-13 | 4.1e-13 / 4.3e-13 | 4.1e-13 / 4.3e-13 | 3.9e-14 / 4.1e-14 |  | 4.1e-14 / 4.3e-14 |
| nodice_hepar2 |  |  | 0.077 | 0.146 | 0.654 | >600 | 0.266 | 5.28 | 8.8 | 3.55 | 181 | oom | 0.18 | 0.194 | 1.94 | 1.28 | 1.27 | 2.4 | 2.55 | 0.232 | 0.395 | 1.74 | 0.348 | 0.282 | 80.4 | 0.5 | 1.22 | 0.923 | alea | 1.1e-16 / 1.3e-16 | 2.7e-09 / 1.7e-08 | 6.8e-07 / 4.2e-06 |  | 1.1e-16 / 5.1e-16 | 5.6e-17 / 3.4e-16 | 1.1e-16 / 3.4e-16 | 1.1e-16 / 3.4e-16 | 2.2e-15 / 3.1e-15 |  | 7.8e-16 / 2.1e-15 | 1.1e-16 / 3.4e-16 | 3.3e-16 / 5.1e-16 | 1.1e-16 / 6.9e-16 | 1.1e-16 / 6.9e-16 | 5.6e-17 / 3.4e-16 | 3.3e-16 / 4.0e-16 | 3.2e-07 / 2.0e-06 | 3.1e-14 / 1.9e-13 | 3.1e-14 / 1.9e-13 | 3.1e-14 / 1.9e-13 | 1.1e-16 / 3.4e-16 | 1.1e-16 / 3.4e-16 | 1.1e-16 / 1.3e-16 |
| nodice_insurance |  |  | 0.103 | 4.36 | 3.61 | oom | 4.07 | >600 | 42.8 | >600 | 30.3 | oom | 0.501 | 0.665 | 9.49 | 1.29 | 1.35 | 6.68 | 7.52 | 6.39 | 2.73 | 10.6 | 4.68 | 10.6 | 287 | 1.78 | 2.54 | 50.3 | alea | 1.7e-16 / 1.9e-15 | 9.8e-08 / 1.2e-06 | 7.0e-07 / 2.9e-05 |  | 2.2e-16 / 1.0e-15 |  | 1.1e-16 / 1.2e-15 |  | 6.7e-16 / 2.5e-15 |  | 2.2e-16 / 1.9e-15 | 1.1e-16 / 1.4e-15 | 2.2e-16 / 1.0e-15 | 1.1e-16 / 1.0e-15 | 1.1e-16 / 1.0e-15 | 2.2e-16 / 1.2e-15 | 5.6e-17 / 8.3e-16 | 4.8e-07 / 2.9e-05 | 4.5e-13 / 4.2e-12 | 4.5e-13 / 4.2e-12 | 4.5e-13 / 4.2e-12 | 1.1e-16 / 1.7e-15 | 5.6e-17 / 1.2e-15 | 1.1e-16 / 1.0e-15 |
| nodice_ladder-1-p |  |  | 0.006 | 0.015 | 0.04 | err | 0.009 | 1.08 | 7.17 | 0.57 | 2.86 | 0.017 | 0.556 | 0.595 | 0.549 | 4.32 | 4.43 | 1.33 | 1.4 | 0.07 | 0.304 | 2.2 | 0.07 | 0.0684 | 2.24 | 0.003 | 0.718 | 0.144 | alea | 1.1e-16 / 2.2e-16 | 5.0e-08 / 1.0e-07 | 2.5e-07 / 5.0e-07 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 2.5e-07 / 5.0e-07 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 5.6e-17 / 1.1e-16 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| nodice_ladder-10-p |  |  | 0.011 | 0.033 | 0.042 | err | 0.019 | 1.69 | 10.5 | 0.679 | 2.86 | >600 | 0.564 | 0.586 | 0.911 | 4.71 | 4.81 | 1.75 | 1.79 | 0.082 | 0.243 | 2.28 | 0.083 | 0.105 | 18.6 | 0.007 | 0.713 | 0.171 | alea | 5.6e-17 / 1.1e-16 | 1.2e-08 / 2.4e-08 | 1.1e-07 / 2.3e-07 |  | 1.7e-16 / 3.4e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.2e-16 | 2.8e-16 / 5.6e-16 | 1.7e-16 / 3.4e-16 |  | 0.0e+00 / 0.0e+00 | 5.6e-17 / 1.1e-16 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 1.7e-16 / 3.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-07 / 2.3e-07 | 5.2e-14 / 1.0e-13 | 5.2e-14 / 1.0e-13 | 5.2e-14 / 1.0e-13 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 |
| nodice_ladder-100-p |  |  | 0.079 | 0.116 | - | err | 0.112 | 8.59 | 11.2 | 42.9 | 3.76 | - | 0.616 | 0.597 | 8.29 | 5.76 | 5.82 | 2.81 | 3.28 | 0.271 | 0.667 | 2.98 | 0.413 | 0.725 | 218 | 0.084 | 0.768 | 1.29 | alea | 5.0e-16 / 1.1e-15 | 2.4e-07 / 5.1e-07 |  |  | 5.6e-16 / 1.2e-15 | 4.4e-16 / 9.3e-16 | 6.1e-16 / 1.3e-15 | 8.3e-16 / 1.8e-15 | 5.6e-16 / 1.2e-15 |  | 6.7e-16 / 1.4e-15 | 5.6e-16 / 1.2e-15 | 7.8e-16 / 1.6e-15 | 7.8e-16 / 1.6e-15 | 7.8e-16 / 1.6e-15 | 3.3e-16 / 7.0e-16 | 4.4e-16 / 9.3e-16 | 1.6e-07 / 3.3e-07 | 4.4e-14 / 9.2e-14 | 4.4e-14 / 9.2e-14 | 4.4e-14 / 9.2e-14 | 6.1e-16 / 1.3e-15 | 6.1e-16 / 1.3e-15 | 7.8e-16 / 1.6e-15 |
| nodice_ladder-20-p |  |  | 0.015 | 0.034 | 0.278 | err | 0.028 | 2.55 | 10.9 | 0.662 | 2.9 | - | 0.636 | 0.533 | 1.39 | 4.62 | 4.69 | 2.13 | 2.21 | 0.093 | 0.268 | 2.32 | 0.094 | 0.232 | 39.6 | 0.01 | 0.713 | 0.241 | alea | 1.7e-16 / 3.4e-16 | 1.4e-07 / 2.9e-07 | 6.6e-07 / 1.3e-06 |  | 2.8e-16 / 5.6e-16 | 5.6e-17 / 1.1e-16 | 1.7e-16 / 3.4e-16 | 2.8e-16 / 5.6e-16 | 2.2e-16 / 4.5e-16 |  | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 2.8e-16 / 5.6e-16 | 2.2e-16 / 4.5e-16 | 3.4e-07 / 7.0e-07 | 2.6e-13 / 5.3e-13 | 2.6e-13 / 5.3e-13 | 2.6e-13 / 5.3e-13 | 1.1e-16 / 2.2e-16 | 1.7e-16 / 3.4e-16 | 1.1e-16 / 2.2e-16 |
| nodice_ladder-30-p |  |  | 0.021 | 0.046 | 235 | err | 0.041 | 3.28 | 11.1 | 0.968 | 2.97 | - | 0.533 | 0.529 | 1.93 | 4.74 | 4.84 | 2.22 | 2.38 | 0.11 | 0.304 | 2.44 | 0.117 | 0.217 | 63 | 0.015 | 0.707 | 0.337 | alea | 2.2e-16 / 4.5e-16 | 1.3e-07 / 2.7e-07 | 5.7e-07 / 1.2e-06 |  | 2.2e-16 / 4.5e-16 | 1.1e-16 / 2.3e-16 | 2.2e-16 / 4.5e-16 | 1.1e-16 / 2.3e-16 | 3.3e-16 / 6.8e-16 |  | 1.7e-16 / 3.4e-16 | 1.7e-16 / 3.4e-16 | 1.7e-16 / 3.4e-16 | 1.7e-16 / 3.4e-16 | 1.7e-16 / 3.4e-16 | 1.7e-16 / 3.4e-16 | 5.6e-17 / 1.1e-16 | 4.3e-07 / 8.8e-07 | 8.0e-14 / 1.6e-13 | 8.0e-14 / 1.6e-13 | 8.0e-14 / 1.6e-13 | 1.7e-16 / 3.4e-16 | 1.7e-16 / 3.4e-16 | 1.7e-16 / 3.4e-16 |
| nodice_ladder-40-p |  |  | 0.026 | 0.053 | >600 | err | 0.05 | 3.94 | 10.7 | 1.24 | 3.03 | - | 0.577 | 0.546 | 2.56 | 4.72 | 4.59 | 2.27 | 2.42 | 0.121 | 0.323 | 2.86 | 0.137 | 0.302 | 82.4 | 0.033 | 0.734 | 0.411 | alea | 2.2e-16 / 4.5e-16 | 1.1e-07 / 2.3e-07 |  |  | 2.2e-16 / 4.5e-16 | 1.7e-16 / 3.4e-16 | 2.2e-16 / 4.5e-16 | 6.7e-16 / 1.4e-15 | 4.4e-16 / 9.1e-16 |  | 2.2e-16 / 4.5e-16 | 2.2e-16 / 4.5e-16 | 2.2e-16 / 4.5e-16 | 2.2e-16 / 4.5e-16 | 2.2e-16 / 4.5e-16 | 5.6e-17 / 1.1e-16 | 1.1e-16 / 2.3e-16 | 2.1e-07 / 4.3e-07 | 3.4e-13 / 7.0e-13 | 3.4e-13 / 7.0e-13 | 3.4e-13 / 7.0e-13 | 2.2e-16 / 4.5e-16 | 1.7e-16 / 3.4e-16 | 2.2e-16 / 4.5e-16 |
| nodice_ladder-50-p |  |  | 0.033 | 0.063 | - | err | 0.059 | 4.63 | 10.8 | 2.46 | 3.12 | - | 0.554 | 0.556 | 3.81 | 4.94 | 4.93 | 2.32 | 2.55 | 0.137 | 0.347 | 2.37 | 0.165 | 0.351 | 105 | 0.039 | 0.734 | 0.514 | alea | 1.7e-16 / 3.4e-16 | 1.4e-07 / 3.0e-07 |  |  | 2.8e-16 / 5.7e-16 | 1.7e-16 / 3.4e-16 | 2.8e-16 / 5.7e-16 | 5.6e-17 / 1.1e-16 | 4.4e-16 / 9.1e-16 |  | 2.8e-16 / 5.7e-16 | 2.8e-16 / 5.7e-16 | 2.8e-16 / 5.7e-16 | 2.8e-16 / 5.7e-16 | 2.8e-16 / 5.7e-16 | 1.7e-16 / 3.4e-16 | 1.7e-16 / 3.4e-16 | 2.6e-07 / 5.2e-07 | 2.7e-13 / 5.6e-13 | 2.7e-13 / 5.6e-13 | 2.7e-13 / 5.6e-13 | 3.3e-16 / 6.8e-16 | 3.3e-16 / 6.8e-16 | 3.9e-16 / 8.0e-16 |
| nodice_ladder-60-p |  |  | 0.04 | 0.076 | - | err | 0.069 | 5.38 | 11.1 | 5.99 | 3.18 | - | 0.555 | 0.562 | 4.38 | 5.04 | 4.99 | 2.48 | 2.69 | 0.158 | 0.392 | 2.53 | 0.201 | 0.714 | 125 | 0.046 | 0.744 | 0.639 | alea | 2.8e-16 / 5.7e-16 | 1.9e-07 / 4.0e-07 |  |  | 3.3e-16 / 6.9e-16 | 2.2e-16 / 4.6e-16 | 3.9e-16 / 8.0e-16 | 1.7e-16 / 3.4e-16 | 3.9e-16 / 8.0e-16 |  | 4.4e-16 / 9.2e-16 | 3.3e-16 / 6.9e-16 | 4.4e-16 / 9.2e-16 | 4.4e-16 / 9.2e-16 | 4.4e-16 / 9.2e-16 | 3.3e-16 / 6.9e-16 | 2.2e-16 / 4.6e-16 | 9.3e-08 / 1.9e-07 | 2.4e-13 / 5.0e-13 | 2.4e-13 / 5.0e-13 | 2.4e-13 / 5.0e-13 | 3.3e-16 / 6.9e-16 | 3.3e-16 / 6.9e-16 | 5.6e-16 / 1.1e-15 |
| nodice_ladder-70-p |  |  | 0.047 | 0.086 | - | err | 0.079 | 5.93 | 11.1 | 8.52 | 3.3 | - | 0.566 | 0.579 | 5.19 | 6.69 | 5.95 | 2.52 | 2.78 | 0.177 | 0.436 | 2.55 | 0.245 | 0.604 | 147 | 0.053 | 0.745 | 0.781 | alea | 3.3e-16 / 6.9e-16 | 2.2e-07 / 4.5e-07 |  |  | 4.4e-16 / 9.2e-16 | 2.8e-16 / 5.8e-16 | 4.4e-16 / 9.2e-16 | 3.3e-16 / 6.9e-16 | 3.9e-16 / 8.1e-16 |  | 5.6e-16 / 1.2e-15 | 3.3e-16 / 6.9e-16 | 5.6e-16 / 1.2e-15 | 5.6e-16 / 1.2e-15 | 5.6e-16 / 1.2e-15 | 4.4e-16 / 9.2e-16 | 4.4e-16 / 9.2e-16 | 3.2e-07 / 6.6e-07 | 3.4e-13 / 7.0e-13 | 3.4e-13 / 7.0e-13 | 3.4e-13 / 7.0e-13 | 3.9e-16 / 8.1e-16 | 3.9e-16 / 8.1e-16 | 5.6e-16 / 1.2e-15 |
| nodice_ladder-80-p |  |  | 0.058 | 0.095 | - | err | 0.091 | 6.67 | 11 | 17.7 | 3.4 | - | 0.597 | 0.62 | 5.92 | 5.14 | 5.15 | 2.64 | 2.97 | 0.208 | 0.498 | 2.66 | 0.289 | 0.915 | 173 | 0.059 | 0.759 | 0.926 | alea | 5.0e-16 / 1.0e-15 | 2.8e-07 / 5.7e-07 |  |  | 4.4e-16 / 9.3e-16 | 2.8e-16 / 5.8e-16 | 5.6e-16 / 1.2e-15 | 1.7e-16 / 3.5e-16 | 5.0e-16 / 1.0e-15 |  | 6.7e-16 / 1.4e-15 | 4.4e-16 / 9.3e-16 | 6.7e-16 / 1.4e-15 | 6.7e-16 / 1.4e-15 | 6.7e-16 / 1.4e-15 | 3.3e-16 / 6.9e-16 | 3.3e-16 / 6.9e-16 | 4.8e-07 / 9.9e-07 | 3.4e-14 / 7.2e-14 | 3.4e-14 / 7.2e-14 | 3.4e-14 / 7.2e-14 | 4.4e-16 / 9.3e-16 | 5.6e-16 / 1.2e-15 | 5.6e-16 / 1.2e-15 |
| nodice_ladder-90-p |  |  | 0.065 | 0.106 | - | err | 0.106 | 7.61 | 11 | 33.9 | 3.56 | - | 0.573 | 0.559 | 7.25 | 5.48 | 5.6 | 2.77 | 3.01 | 0.242 | 0.589 | 2.81 | 0.355 | 0.636 | 200 | 0.069 | 0.759 | 1.1 | alea | 3.9e-16 / 8.1e-16 | 2.3e-07 / 4.9e-07 |  |  | 4.4e-16 / 9.3e-16 | 3.9e-16 / 8.1e-16 | 5.0e-16 / 1.0e-15 | 2.8e-16 / 5.8e-16 | 5.0e-16 / 1.0e-15 |  | 6.1e-16 / 1.3e-15 | 5.0e-16 / 1.0e-15 | 6.7e-16 / 1.4e-15 | 6.7e-16 / 1.4e-15 | 6.7e-16 / 1.4e-15 | 4.4e-16 / 9.3e-16 | 3.3e-16 / 7.0e-16 | 3.7e-07 / 7.7e-07 | 1.5e-13 / 3.1e-13 | 1.5e-13 / 3.1e-13 | 1.5e-13 / 3.1e-13 | 5.6e-16 / 1.2e-15 | 5.6e-16 / 1.2e-15 | 6.1e-16 / 1.3e-15 |
| nodice_survey |  |  | 0.005 | 0.014 | 0.031 | 1.59 | 0.032 | 1.4 | 6.11 | 0.498 | 2.7 | oom | 0.164 | 0.162 | 0.247 | 1.28 | 1.28 | 1.37 | 1.45 | 0.08 | 0.212 | 1.47 | 0.079 | 0.242 | 6.09 | 0.005 | 0.715 | 0.107 | alea | 1.1e-16 / 3.5e-16 | 3.2e-08 / 7.8e-08 | 7.5e-07 / 2.7e-06 | 1.1e-16 / 2.0e-16 | 2.8e-17 / 1.8e-16 | 8.3e-17 / 5.3e-16 | 1.1e-16 / 4.0e-16 | 1.1e-16 / 4.0e-16 | 2.8e-17 / 1.8e-16 |  | 1.1e-16 / 5.3e-16 | 1.1e-16 / 2.0e-16 | 1.1e-16 / 5.3e-16 | 1.1e-16 / 5.3e-16 | 1.1e-16 / 5.3e-16 | 2.8e-17 / 1.8e-16 | 1.1e-16 / 2.0e-16 | 2.5e-07 / 1.4e-06 | 1.1e-16 / 2.0e-16 | 1.1e-16 / 2.0e-16 | 1.1e-16 / 2.0e-16 | 1.1e-16 / 2.0e-16 | 5.6e-17 / 3.5e-16 | 8.3e-17 / 5.3e-16 |
| nodice_uf20-0990 |  |  | 0.025 | >600 | 0.083 | >600 | 32.3 | >600 | 10.1 | 0.744 | 3.02 | >600 | 28.8 | 45.3 | 2.64 | 4.96 | 4.6 | 8.78 | 13.8 | 9.59 | 1.26 | >600 | 5.43 | >600 | 122 | 0.013 | 0.725 | 25.4 | alea | 6.8e-21 / 7.1e-16 |  | 4.6e-07 / 4.9e-02 |  | 6.8e-21 / 7.1e-16 |  | 6.8e-21 / 7.1e-16 | 6.8e-21 / 7.1e-16 | 6.8e-21 / 7.1e-16 |  | 6.8e-21 / 7.1e-16 | 6.8e-21 / 7.1e-16 | 6.8e-21 / 7.1e-16 | 6.8e-21 / 7.1e-16 | 6.8e-21 / 7.1e-16 | 6.8e-21 / 7.1e-16 | 6.8e-21 / 7.1e-16 | 4.6e-07 / 4.9e-02 | 2.5e-18 / 2.6e-13 |  | 2.5e-18 / 2.6e-13 |  | 6.8e-21 / 7.1e-16 | 6.8e-21 / 7.1e-16 |
| nodice_uf20-0991 |  |  | 0.027 | >600 | 0.087 | >600 | 23.3 | >600 | 10.1 | 0.745 | 3.03 | >600 | 27.8 | 43.5 | 2.81 | 4.91 | 4.57 | 15.9 | 11.8 | 24.7 | 2.81 | >600 | 16.2 | err | 112 | 0.014 | 0.721 | 25.6 | alea | 3.4e-21 / 1.6e-16 |  | 1.9e-08 / 9.1e-04 |  | 3.4e-21 / 1.6e-16 |  | 3.4e-21 / 1.6e-16 | 3.4e-21 / 1.6e-16 | 3.4e-21 / 1.6e-16 |  | 3.4e-21 / 1.6e-16 | 3.4e-21 / 1.6e-16 | 3.4e-21 / 1.6e-16 | 3.4e-21 / 1.6e-16 | 3.4e-21 / 1.6e-16 | 3.4e-21 / 1.6e-16 | 3.4e-21 / 1.6e-16 | 1.9e-08 / 9.1e-04 | 3.8e-17 / 1.8e-12 |  | 3.8e-17 / 1.8e-12 |  | 0.0e+00 / 0.0e+00 | 3.4e-21 / 1.6e-16 |
| nodice_uf20-0992 |  |  | 0.026 | >600 | 0.089 | >600 | 24.9 | >600 | 10.3 | 0.823 | 2.15 | >600 | 25.2 | 38 | 2.72 | 4.95 | 4.59 | 10.3 | 18.7 | 13.4 | 1.91 | >600 | 8.15 | err | 108 | 0.014 | 0.713 | 21.1 | alea | 8.5e-22 / 2.2e-16 |  | 1.9e-07 / 4.9e-02 |  | 8.5e-22 / 2.2e-16 |  | 8.5e-22 / 2.2e-16 | 8.5e-22 / 2.2e-16 | 8.5e-22 / 2.2e-16 |  | 8.5e-22 / 2.2e-16 | 8.5e-22 / 2.2e-16 | 8.5e-22 / 2.2e-16 | 8.5e-22 / 2.2e-16 | 8.5e-22 / 2.2e-16 | 8.5e-22 / 2.2e-16 | 8.5e-22 / 2.2e-16 | 1.9e-07 / 4.9e-02 | 5.0e-18 / 1.3e-12 |  | 5.0e-18 / 1.3e-12 |  | 8.5e-22 / 2.2e-16 | 8.5e-22 / 2.2e-16 |
| nodice_uf20-0993 |  |  | 0.025 | >600 | 0.076 | >600 | 35.7 | >600 | 10.5 | 0.928 | 2.13 | >600 | 22.3 | 36.1 | 2.5 | 4.44 | 4.12 | 11.6 | 11.4 | 23.9 | 2.33 | >600 | 15.2 | err | 110 | 0.011 | 0.715 | 21.4 | alea | 0.0e+00 / 0.0e+00 |  | 9.3e-08 / 4.9e-02 |  | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 9.3e-08 / 4.9e-02 | 2.5e-18 / 1.3e-12 |  | 2.5e-18 / 1.3e-12 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| nodice_uf20-0994 |  |  | 0.027 | >600 | 0.083 | >600 | 25.2 | >600 | 10.2 | 0.77 | 3.12 | >600 | 23.5 | 38 | 2.59 | 4.83 | 4.66 | 9.47 | 15.3 | 13.4 | 1.62 | 278 | 6.89 | err | 119 | 0.011 | 0.728 | 15.5 | alea | 3.4e-21 / 2.5e-16 |  | 3.5e-07 / 2.6e-02 |  | 3.4e-21 / 2.5e-16 |  | 3.4e-21 / 2.5e-16 | 3.4e-21 / 2.5e-16 | 3.4e-21 / 2.5e-16 |  | 3.4e-21 / 2.5e-16 | 3.4e-21 / 2.5e-16 | 3.4e-21 / 2.5e-16 | 3.4e-21 / 2.5e-16 | 3.4e-21 / 2.5e-16 | 3.4e-21 / 2.5e-16 | 3.4e-21 / 2.5e-16 | 3.5e-07 / 2.6e-02 | 1.2e-17 / 9.4e-13 | 1.2e-17 / 9.4e-13 | 1.2e-17 / 9.4e-13 |  | 3.4e-21 / 2.5e-16 | 3.4e-21 / 2.5e-16 |
| nodice_uf20-0995 |  |  | 0.028 | >600 | 0.092 | >600 | 30.2 | >600 | 10.8 | 0.788 | 3.15 | >600 | 26 | 41.3 | 2.77 | 4.91 | 4.47 | 9.91 | 41.6 | 14.1 | 1.49 | >600 | 11.4 | err | 120 | 0.014 | 0.741 | 25.7 | alea | 1.7e-21 / 5.9e-16 |  | 1.4e-07 / 4.9e-02 |  | 1.7e-21 / 5.9e-16 |  | 1.7e-21 / 5.9e-16 | 1.7e-21 / 5.9e-16 | 1.7e-21 / 5.9e-16 |  | 1.7e-21 / 5.9e-16 | 1.7e-21 / 5.9e-16 | 1.7e-21 / 5.9e-16 | 1.7e-21 / 5.9e-16 | 1.7e-21 / 5.9e-16 | 1.7e-21 / 5.9e-16 | 1.7e-21 / 5.9e-16 | 1.4e-07 / 4.9e-02 | 1.3e-18 / 4.4e-13 |  | 1.3e-18 / 4.4e-13 |  | 1.7e-21 / 5.9e-16 | 1.7e-21 / 5.9e-16 |
| nodice_uf20-0996 |  |  | 0.027 | >600 | 0.101 | >600 | 30.1 | >600 | 10.1 | 0.747 | 2.96 | >600 | 22.4 | 34.9 | 2.74 | 4.71 | 4.45 | 7.86 | 21.9 | 6.27 | 0.869 | >600 | 5.08 | 417 | 116 | 0.013 | 0.733 | 35.7 | alea | 3.4e-21 / 7.1e-16 |  | 2.3e-07 / 4.9e-02 |  | 3.4e-21 / 7.1e-16 |  | 3.4e-21 / 7.1e-16 | 3.4e-21 / 7.1e-16 | 3.4e-21 / 7.1e-16 |  | 3.4e-21 / 7.1e-16 | 3.4e-21 / 7.1e-16 | 3.4e-21 / 7.1e-16 | 3.4e-21 / 7.1e-16 | 3.4e-21 / 7.1e-16 | 3.4e-21 / 7.1e-16 | 3.4e-21 / 7.1e-16 | 2.3e-07 / 4.9e-02 | 1.2e-18 / 2.6e-13 |  | 1.2e-18 / 2.6e-13 | 3.4e-21 / 7.1e-16 | 3.4e-21 / 7.1e-16 | 3.4e-21 / 7.1e-16 |
| nodice_uf20-0997 |  |  | 0.027 | >600 | 0.083 | >600 | 24.9 | >600 | 9.91 | 0.755 | 3.09 | >600 | 28.6 | 43.5 | 2.71 | 5.2 | 5.09 | 27.9 | 24 | 48.5 | 6.07 | >600 | 33.3 | err | 118 | 0.014 | 0.718 | 20.3 | alea | 5.1e-21 / 4.4e-16 |  | 4.4e-07 / 3.9e-02 |  | 5.1e-21 / 4.4e-16 |  | 5.1e-21 / 4.4e-16 | 5.1e-21 / 4.4e-16 | 5.1e-21 / 4.4e-16 |  | 5.1e-21 / 4.4e-16 | 5.1e-21 / 4.4e-16 | 5.1e-21 / 4.4e-16 | 5.1e-21 / 4.4e-16 | 5.1e-21 / 4.4e-16 | 5.1e-21 / 4.4e-16 | 5.1e-21 / 4.4e-16 | 4.4e-07 / 3.9e-02 | 2.5e-17 / 2.2e-12 |  | 2.5e-17 / 2.2e-12 |  | 5.1e-21 / 4.4e-16 | 5.1e-21 / 4.4e-16 |
| nodice_uf20-0998 |  |  | 0.031 | >600 | 0.103 | >600 | 21.7 | >600 | 10.1 | 0.835 | 3.29 | >600 | 28.3 | 61.6 | 2.91 | 4.96 | 4.56 | 8.91 | 19.4 | 11 | 1.53 | >600 | 7.9 | >600 | 132 | 0.02 | 0.742 | 32.8 | alea | 8.5e-22 / 1.3e-16 |  | 3.2e-07 / 4.9e-02 |  | 8.5e-22 / 1.3e-16 |  | 8.5e-22 / 1.3e-16 | 8.5e-22 / 1.3e-16 | 8.5e-22 / 1.3e-16 |  | 8.5e-22 / 1.3e-16 | 8.5e-22 / 1.3e-16 | 8.5e-22 / 1.3e-16 | 8.5e-22 / 1.3e-16 | 8.5e-22 / 1.3e-16 | 8.5e-22 / 1.3e-16 | 8.5e-22 / 1.3e-16 | 3.2e-07 / 4.9e-02 | 3.8e-18 / 5.6e-13 |  | 3.8e-18 / 5.6e-13 |  | 1.7e-21 / 2.5e-16 | 8.5e-22 / 1.3e-16 |
| nodice_uf20-0999 |  |  | 0.033 | >600 | 0.106 | >600 | 29.9 | >600 | 10.5 | 0.768 | 2.98 | >600 | 30.7 | 49.4 | 2.88 | 5.06 | 5 | 9.57 | 28.6 | 7.85 | 1.22 | >600 | 5.67 | err | 115 | 0.014 | 0.719 | 26.1 | alea | 3.7e-20 / 1.7e-15 |  | 6.5e-08 / 3.0e-03 |  | 3.7e-20 / 1.7e-15 |  | 3.7e-20 / 1.7e-15 | 3.7e-20 / 1.7e-15 | 3.7e-20 / 1.7e-15 |  | 3.7e-20 / 1.7e-15 | 3.7e-20 / 1.7e-15 | 3.7e-20 / 1.7e-15 | 3.7e-20 / 1.7e-15 | 3.7e-20 / 1.7e-15 | 3.7e-20 / 1.7e-15 | 3.7e-20 / 1.7e-15 | 6.5e-08 / 3.0e-03 | 4.4e-17 / 2.0e-12 |  | 4.4e-17 / 2.0e-12 |  | 3.7e-20 / 1.7e-15 | 3.7e-20 / 1.7e-15 |
| nodice_uf50-0999 |  |  | 13.8 | >600 | >600 | >600 | err | >600 | err | >600 | 3.09 | >600 | >600 | >600 | err | >600 | >600 | >600 | >600 | err | err | err | err | oom | oom | oom | oom | >600 | alea | 1.9e-27 / 4.0e-14 |  |  |  |  |  |  |  | 1.9e-27 / 4.0e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| nodice_water |  |  | 0.364 | 38.7 | 17.3 | oom | 50.3 | 148 | 9.62 | 16 | 62.3 | oom | 38.4 | 38.3 | 20.1 | 4.41 | 4.55 | 5.76 | 6.32 | oom | 18.1 | >600 | oom | err | 370 | 0.414 | 1.11 | 8.79 | alea | 1.1e-16 / 1.1e-15 | 5.2e-08 / 2.7e-06 | 7.4e-07 / 7.5e-05 |  | 1.1e-16 / 4.9e-16 | 2.8e-17 / 8.9e-16 | 3.5e-18 / 4.5e-16 | 1.7e-16 / 6.1e-15 | 2.8e-17 / 4.5e-16 |  | 2.2e-16 / 7.3e-16 | 6.9e-18 / 5.6e-16 | 2.2e-16 / 4.5e-16 | 2.8e-17 / 5.6e-16 | 2.8e-17 / 5.6e-16 | 3.3e-16 / 4.5e-16 | 2.2e-16 / 4.5e-16 |  | 4.6e-13 / 1.4e-12 |  |  |  | 2.8e-17 / 3.4e-16 | 2.8e-17 / 4.5e-16 |
| pluck_hmm | 199 | 346 | 0.025 | >600 | oom | oom | oom | oom | 0.031 | 3.2 | 3.02 | oom | >600 | >600 | 8.59 | 3.06 | 3.19 | 1.78 | 2.01 | zero | 0.391 | 1.54 | 0.286 | 0.197 | 52.8 | 0.034 | 0.778 | >600 | alea | 5.6e-27 / 3.1e-14 |  |  |  |  |  | 5.7e-27 / 3.2e-14 | 1.2e-16 / 6.9e-04 | 6.3e-27 / 3.5e-14 |  |  |  | 5.6e-27 / 3.1e-14 | 6.2e-15 / 3.4e-02 | 6.2e-15 / 3.4e-02 | 5.7e-27 / 3.1e-14 | 5.6e-27 / 3.1e-14 |  | 6.2e-26 / 3.4e-13 | 6.2e-26 / 3.4e-13 | 6.2e-26 / 3.4e-13 | 5.5e-27 / 3.0e-14 | 5.4e-27 / 3.0e-14 | 5.7e-27 / 3.2e-14 |
| pluck_pcfg | 12031 | 532827 | 105 | oom | oom | oom | >600 | err | wrong | >600 | >600 | >600 | oom | oom | oom | oom | oom | >600 | >600 | >600 | >600 | >600 | >600 | oom | >600 | 23 | 25.6 | >600 | alea | 9.3e-23 / 1.6e-15 |  |  |  |  |  | 3.0e-08 / 5.0e-01 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 9.3e-23 / 1.6e-15 |
| pluck_sorted_list | 57 | 4056 | 5.83 | >600 | oom | >600 | oom | >600 | err | 2.2 | 5.09 | oom | >600 | oom | >600 | >600 | >600 | >600 | >600 | oom | oom | >600 | oom | oom | >600 | 14.3 | 15.3 | >600 | alea | 1.7e-25 / 2.3e-14 |  |  |  |  |  |  | 1.7e-25 / 2.3e-14 | 1.7e-25 / 2.3e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.7e-25 / 2.3e-14 |
| pluck_string_editing | 130 | 1343 | 0.06 | >600 | zero | >600 | oom | >600 | wrong | 1.11 | 5.21 | oom | >600 | oom | err | oom | oom | >600 | >600 | oom | oom | >600 | oom | err | 265 | oom | oom | >600 | alea | 1.7e-26 / 7.4e-15 |  |  |  |  |  | 3.0e-08 / 1.3e+04 | 9.1e-17 / 4.0e-05 | 1.7e-26 / 7.4e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.1e-26 / 9.0e-15 |  |

Seconds per system, the median over the repetitions; a failure is `>600`
(timeout), `oom`, `err` or `n/a`; each system's last columns are its
largest absolute and relative difference from the reference named in
`ref`.

### On the bit-blasted programs

| family | instances | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Scallop | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Ace log space | Merlin | pyAgrum | pgmpy | libDAI | TensorInference.jl greedy | TensorInference.jl treesa | Roulette |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| bb_GPA | 7 | 7 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 7 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 6 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 3 | 0 of 4 |  | 0 of 2 | 0 of 2 | 0 of 2 | 0 of 1 | 0 of 2 | 0 of 2 | 4 of 7 |
| bb_addFun_max_p | 3 | 3 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 2 of 3 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 0 of 1 |
| bb_addFun_max | 3 | 3 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 2 of 3 |
| bb_addFun_sum | 5 | 5 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 4 of 5 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 3 of 4 |
| bb_altermu2_p | 2 | 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 1 of 2 |  | 1 of 2 | 1 of 2 | 1 of 2 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 |
| bb_altermu2 | 7 | 3 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 7 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 |
| bb_clickGraph | 7 | 7 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 1 of 2 | 1 of 2 | 6 of 7 | 7 of 7 | 0 of 1 | 3 of 4 | 3 of 4 | 3 of 4 | 4 of 5 | 4 of 5 | 7 of 7 | 7 of 7 |  | 7 of 7 | 7 of 7 | 7 of 7 | 7 of 7 | 7 of 7 | 7 of 7 | 3 of 4 |
| bb_clinicalTrial1 | 6 | 6 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 2 | 2 of 4 | 0 of 1 | 0 of 1 | 0 of 1 | 2 of 3 | 0 of 6 | 0 of 6 | 5 of 6 | 1 of 6 | 1 of 1 | 5 of 6 | 6 of 6 | 6 of 6 | 3 of 4 | 0 of 1 | 0 of 1 | 2 of 3 |
| bb_clinicalTrial2 | 7 | 7 | 0 of 1 | 1 of 2 | 0 of 7 | 0 of 1 | 0 of 1 | 2 of 3 | 7 of 7 | 7 of 7 | 0 of 1 | 2 of 3 | 2 of 3 | 2 of 3 | 3 of 4 | 3 of 4 | 7 of 7 | 7 of 7 |  | 7 of 7 | 7 of 7 | 7 of 7 | 7 of 7 | 7 of 7 | 7 of 7 | 3 of 4 |
| bb_coinBias | 2 | 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 |
| bb_conjugate_ex3 | 7 | 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 7 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 |
| bb_conjugate_gaussians | 3 | 3 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 2 of 3 |
| bb_fair_dt14 | 7 | 7 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 6 of 7 | 2 of 3 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 3 of 4 |
| bb_fair_dt16 | 6 | 6 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 6 of 6 | 6 of 6 | 6 of 6 | 0 of 1 | 0 of 1 | 0 of 1 | 2 of 3 | 0 of 1 | 0 of 1 | 1 of 2 | 2 of 3 |  | 1 of 2 | 0 of 1 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 6 of 6 |
| bb_fair_dt44 | 6 | 6 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 6 of 6 | 6 of 6 | 6 of 6 | 0 of 1 | 0 of 1 | 0 of 1 | 2 of 3 | 0 of 1 | 0 of 1 | 6 of 6 | 2 of 3 |  | 1 of 2 | 0 of 1 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 6 of 6 |
| bb_fair_dt4 | 7 | 7 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 6 of 7 | 3 of 4 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 3 of 4 |
| bb_genes5 | 6 | 6 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 6 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 4 of 5 |
| bb_genes | 11 | 11 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 11 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 11 | 0 of 11 |  | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |
| bb_laplace | 2 | 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 1 of 2 |  | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 |
| bb_linreg | 8 | 6 | 0 of 1 | 0 of 1 | 2 of 8 | 0 of 1 | 0 of 1 | 1 of 2 | 0 of 2 | 3 of 4 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 5 of 6 | 5 of 6 |  | 2 of 3 | 2 of 3 | 1 of 2 | 2 of 3 | 1 of 2 | 1 of 1 | 4 of 5 |
| bb_markov | 8 | 8 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 8 | 0 of 1 | 0 of 4 | 2 of 3 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 3 of 6 | 3 of 6 | 2 of 3 | 2 of 3 | 3 of 4 | 3 of 4 | 2 of 3 | 1 of 2 | 0 of 1 | 2 of 3 |
| bb_multimodal | 1 | 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |
| bb_normal_mixture_mu1 | 5 | 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 5 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 2 | 0 of 2 | 1 of 3 | 1 of 2 | 1 of 2 | 1 of 2 | 1 of 2 | 0 of 1 | 0 of 1 | 1 of 2 |
| bb_normal_mixture_mu2 | 3 | 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 3 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 2 | 0 of 2 | 1 of 3 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 |
| bb_normal_mixture_theta | 1 | 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |
| bb_pedestrian | 8 | 8 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 6 of 7 | 7 of 8 |  | 2 of 3 | 2 of 3 | 2 of 3 | 2 of 3 | 2 of 4 | 4 of 5 | 2 of 3 |
| bb_pi | 7 | 6 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 3 of 4 | 4 of 5 | 4 of 5 | 0 of 1 | 2 of 3 | 2 of 3 | 2 of 5 | 0 of 1 | 0 of 1 | 3 of 4 | 3 of 4 |  | 2 of 3 | 2 of 3 | 1 of 2 | 2 of 3 | 0 of 2 | 2 of 3 | 5 of 6 |
| bb_population | 12 | 12 | 0 of 1 | 0 of 1 | 0 of 1 | 12 of 12 | 0 of 3 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 6 | 1 of 4 | 4 of 4 | 1 of 2 | 1 of 2 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 |
| bb_spacex_p | 3 | 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 0 of 1 | 1 of 2 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 1 of 2 | 1 of 2 | 1 of 2 | 1 of 2 |  | 1 of 2 | 1 of 2 | 1 of 2 | 1 of 2 | 1 of 2 | 2 of 3 | 0 of 1 |
| bb_spacex | 4 | 4 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 1 of 2 |  | 2 of 3 | 1 of 2 | 0 of 1 | 2 of 3 | 1 of 2 | 2 of 3 | 3 of 4 |
| bb_trueskill | 2 | 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 1 of 2 |  | 1 of 2 | 0 of 1 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 1 of 2 |
| bb_tug_of_war | 7 | 7 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 7 of 7 | 1 of 2 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 | 7 of 7 | 7 of 7 | 7 of 7 | 0 of 1 | 0 of 1 |  | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 2 of 3 |
| bb_weekend_p | 8 | 7 | 0 of 1 | 3 of 4 | 0 of 1 | 0 of 1 | 0 of 1 | 5 of 6 | 5 of 8 | 2 of 3 | 0 of 1 | 0 of 1 | 0 of 1 | 3 of 4 | 1 of 2 | 1 of 2 | 2 of 5 | 2 of 3 |  | 2 of 3 | 2 of 3 | 2 of 3 | 2 of 3 | 1 of 2 | 2 of 3 | 6 of 8 |
| bb_weekend | 7 | 7 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 7 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 |  | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 5 of 7 |
| bb_wmi_path | 9 | 9 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 2 of 4 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 0 of 1 | 3 of 4 | 2 of 3 | 2 of 3 | 7 of 9 | 3 of 8 |  | 5 of 6 | 5 of 6 | 6 of 7 | 5 of 6 | 3 of 4 | 2 of 3 | 5 of 6 |
| bb_wmi_star | 8 | 8 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 2 of 4 | 0 of 1 | 4 of 5 | 0 of 1 | 0 of 1 | 0 of 1 | 3 of 4 | 2 of 3 | 2 of 3 | 4 of 8 | 3 of 7 | 3 of 4 | 5 of 6 | 5 of 6 | 5 of 6 | 3 of 4 | 1 of 2 | 1 of 2 | 5 of 6 |
| bb_zeroone_w1 | 11 | 9 | 0 of 1 | 2 of 3 | 0 of 1 | 0 of 1 | 0 of 1 | 4 of 5 | 7 of 11 | 4 of 5 | 0 of 1 | 1 of 2 | 1 of 2 | 2 of 3 | 1 of 2 | 1 of 2 | 2 of 3 | 2 of 3 |  | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 6 of 7 |
| bb_zeroone_w2 | 7 | 7 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 2 of 3 | 6 of 7 | 2 of 3 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 1 of 2 |  | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 4 of 5 |

**Geni.** Geni ran 38 instances and answered 0 (1 fail, 9 run out of
memory, 28 time out). Over the 38 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, Geni
takes 22800 s in all and tddlog 785 s. tddlog alone answers GPA_w19,
addFun_max_p4, addFun_max_w17, addFun_sum_w17, altermu2_p4 and 33 more.

**noDice.** noDice ran 45 instances and answered 7 (1 fail, 27 run out
of memory, 10 time out). Its answers differ from the reference's by at
most 1.5e-06 relative (7.4e-07 absolute, clinicalTrial2_w6) over 7
instances. Over the 45 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, noDice takes 23343
s in all and tddlog 793 s; over the 7 both answer, 543 s against
tddlog's 3.12. noDice is faster on 1 (weekend_p64 1.88 s against 1.98).
tddlog alone answers GPA_w19, addFun_max_p4, addFun_max_w17,
addFun_sum_w17, altermu2_p4 and 33 more.

**Psi.** Psi ran 51 instances and answered 2 (13 fail, 2 run out of
memory, 34 time out). Over the 50 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, Psi
takes 29400 s in all and tddlog 1651 s. Only Psi answers linreg_w14.
tddlog alone answers GPA_w19, addFun_max_p4, addFun_max_w17,
addFun_sum_w17, altermu2_p4 and 44 more.

**Genfer.** Genfer ran 55 instances and answered 12 (7 fail, 24 run out
of memory, 12 time out). Its answers differ from the reference's by at
most 3.5e+05 relative (6.4e-12 absolute, population_s4) over 12
instances. Over the 51 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, Genfer takes 23403
s in all and tddlog 1622 s; over the 12 both answer, 3.41 s against
tddlog's 435. Genfer is faster on 12 (population_s1 0.009 s against
0.84, population_s2 0.012 s against 11.6, population_s3 0.017 s against
11.8 and population_s4 0.018 s against 13.4, the largest differences).
tddlog alone answers GPA_w19, addFun_max_p4, addFun_max_w17,
addFun_sum_w17, altermu2_p4 and 34 more.

**SPPL.** SPPL ran 87 instances and answered 1 (47 fail, 4 run out of
memory, 31 time out, 4 answer wrongly). Its answers differ from the
reference's by at most 5.5e-16 relative (3.3e-16 absolute,
clickGraph_w4) over 1 instance. SPPL's answer is wrong, zero or
underflowed on markov_n10, markov_n100, markov_n20 and markov_n50. Over
the 78 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, SPPL takes 46688 s in all and tddlog
2270 s; over the 1 both answer, 488 s against tddlog's 0.015. tddlog
alone answers GPA_w19, GPA_w21, GPA_w23, GPA_w25, GPA_w27 and 72 more.

**Pluck.** Pluck ran 91 instances and answered 54 (27 fail, 10 time
out). Its answers differ from the reference's by at most 2.8e-14
relative (4.1e-15 absolute, linreg_w8) over 54 instances. Over the 91
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Pluck takes 26117 s in all and tddlog 1723
s; over the 54 both answer, 3917 s against tddlog's 504. tddlog alone
answers GPA_w19, addFun_max_p4, addFun_max_w17, addFun_sum_w17,
altermu2_p4 and 32 more.

**PITA.** PITA ran 106 instances and answered 61 (4 fail, 40 time out, 1
answer wrongly). Its answers differ from the reference's by at most
1.0e+00 relative (1.4e-14 absolute, pedestrian_s4) over 61 instances.
PITA's answer is wrong, zero or underflowed on linreg_w8. Over the 102
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, PITA takes 31074 s in all and tddlog 1109 s;
over the 61 both answer, 6474 s against tddlog's 157. tddlog alone
answers GPA_w19, addFun_max_p64, addFun_max_w19, addFun_sum_w25,
altermu2_p4 and 36 more.

**aspmc.** aspmc ran 93 instances and answered 58 (34 time out, 1 print
zeros). Its answers differ from the reference's by at most 2.9e-14
relative (5.9e-15 absolute, clinicalTrial1_w4) over 58 instances.
aspmc's answer is wrong, zero or underflowed on clinicalTrial1_w6. Over
the 93 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, aspmc takes 27200 s in all and tddlog
1016 s; over the 58 both answer, 6200 s against tddlog's 54.6. tddlog
alone answers GPA_w19, addFun_max_p16, addFun_max_w17, addFun_sum_w17,
altermu2_p4 and 30 more.

**Scallop.** Scallop ran 38 instances and answered 0 (24 run out of
memory, 14 time out). Over the 38 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, Scallop
takes 22800 s in all and tddlog 787 s. tddlog alone answers GPA_w19,
addFun_max_p4, addFun_max_w17, addFun_sum_w17, altermu2_p4 and 33 more.

**Storm sparse.** Storm sparse ran 52 instances and answered 9 (5 fail,
1 run out of memory, 37 time out). Its answers differ from the
reference's by at most 2.4e-14 relative (4.1e-15 absolute, spacex_p4)
over 9 instances. Over the 52 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, Storm
sparse takes 26541 s in all and tddlog 1067 s; over the 9 both answer,
741 s against tddlog's 1.37. tddlog alone answers GPA_w19, GPA_w21,
GPA_w23, GPA_w25, GPA_w27 and 38 more.

**Storm exact.** Storm exact ran 46 instances and answered 8 (38 time
out). Its answers differ from the reference's by at most 1.5e-15
relative (7.8e-16 absolute, zeroone_w1_w10) over 8 instances. Over the
46 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Storm exact takes 23346 s in all and
tddlog 790 s; over the 8 both answer, 546 s against tddlog's 1.28.
tddlog alone answers GPA_w19, addFun_max_p4, addFun_max_w17,
addFun_sum_w17, altermu2_p4 and 33 more.

**Storm dd.** Storm dd ran 71 instances and answered 32 (1 fail, 38 time
out). Its answers differ from the reference's by at most 6.4e-14
relative (1.1e-14 absolute, clinicalTrial1_w6) over 32 instances. Over
the 70 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Storm dd takes 25200 s in all and
tddlog 816 s; over the 32 both answer, 2400 s against tddlog's 16.
tddlog alone answers GPA_w19, addFun_max_p4, addFun_max_w17,
addFun_sum_w17, altermu2_p4 and 33 more.

**PRISM sparse.** PRISM sparse ran 63 instances and answered 21 (6 fail,
36 time out). Its answers differ from the reference's by at most 6.3e-14
relative (3.3e-14 absolute, wmi_star_n32) over 21 instances. Over the 63
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, PRISM sparse takes 29602 s in all and tddlog
905 s; over the 21 both answer, 4402 s against tddlog's 7.58. tddlog
alone answers GPA_w19, addFun_max_p4, addFun_max_w17, addFun_sum_w17,
altermu2_p4 and 37 more.

**PRISM hybrid.** PRISM hybrid ran 63 instances and answered 21 (6 fail,
36 time out). Its answers differ from the reference's by at most 6.3e-14
relative (3.3e-14 absolute, wmi_star_n32) over 21 instances. Over the 63
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, PRISM hybrid takes 29593 s in all and tddlog
905 s; over the 21 both answer, 4393 s against tddlog's 7.58. tddlog
alone answers GPA_w19, addFun_max_p4, addFun_max_w17, addFun_sum_w17,
altermu2_p4 and 37 more.

**Ace.** Ace ran 126 instances and answered 65 (22 fail, 4 run out of
memory, 35 time out). Its answers differ from the reference's by at most
1.0e+00 relative (1.1e-15 absolute, population_s1) over 64 instances.
Over the 124 instances given to both that either answers, a run that
does not answer counted at the 600 s limit, Ace takes 39691 s in all and
tddlog 2541 s; over the 65 both answer, 4291 s against tddlog's 160.
tddlog alone answers GPA_w19, GPA_w21, GPA_w23, addFun_max_p4,
addFun_max_w17 and 54 more.

**Ace c2d.** Ace c2d ran 120 instances and answered 54 (18 fail, 4 run
out of memory, 36 time out, 8 answer wrongly). Its answers differ from
the reference's by at most 1.0e+00 relative (1.2e-15 absolute,
population_s1) over 54 instances. Ace c2d's answer is wrong, zero or
underflowed on clinicalTrial1_w10, clinicalTrial1_w12,
clinicalTrial1_w14, clinicalTrial1_w6, clinicalTrial1_w8 and 3 more.
Over the 119 instances given to both that either answers, a run that
does not answer counted at the 600 s limit, Ace c2d takes 43062 s in all
and tddlog 2614 s; over the 54 both answer, 4062 s against tddlog's 194.
tddlog alone answers GPA_w19, GPA_w21, GPA_w23, GPA_w25, addFun_max_p4
and 60 more.

**Ace log space.** Ace log space ran 18 instances and answered 12 (4
fail, 2 time out). Its answers differ from the reference's by at most
1.0e+00 relative (2.2e-14 absolute, population_s1) over 8 instances.
Over the 15 instances given to both that either answers, a run that does
not answer counted at the 600 s limit, Ace log space takes 2735 s in all
and tddlog 621 s; over the 11 both answer, 335 s against tddlog's 118.
Ace log space is faster on 1 (clinicalTrial1_w14 21.5 s against 25.7).
tddlog alone answers markov_n500, normal_mixture_mu1_w9,
normal_mixture_mu2_w9 and wmi_star_n4096.

**Merlin.** Merlin ran 86 instances and answered 49 (3 fail, 23 run out
of memory, 11 time out). Its answers differ from the reference's by at
most 1.0e+00 relative (4.2e-07 absolute, linreg_w8) over 47 instances.
Over the 86 instances given to both that either answers, a run that does
not answer counted at the 600 s limit, Merlin takes 23337 s in all and
tddlog 1489 s; over the 49 both answer, 1137 s against tddlog's 62.9.
Merlin is faster on 5 (clickGraph_w8 0.17 s against 0.39, clickGraph_w10
0.23 s against 0.59, clickGraph_w12 0.3 s against 0.77 and
clickGraph_w16 0.47 s against 0.97, the largest differences). tddlog
alone answers GPA_w19, GPA_w21, addFun_max_p4, addFun_max_w17,
addFun_sum_w17 and 32 more.

**pyAgrum.** pyAgrum ran 82 instances and answered 46 (3 fail, 17 run
out of memory, 16 time out). Its answers differ from the reference's by
at most 1.0e+00 relative (4.3e-13 absolute, population_s1) over 46
instances. Over the 82 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, pyAgrum takes 24818
s in all and tddlog 1206 s; over the 46 both answer, 3218 s against
tddlog's 80.1. pyAgrum is faster on 3 (clickGraph_w10 0.55 s against
0.59, clickGraph_w12 0.71 s against 0.77 and clickGraph_w14 0.9 s
against 1.16). tddlog alone answers GPA_w19, GPA_w21, addFun_max_p4,
addFun_max_w17, addFun_sum_w17 and 31 more.

**pgmpy.** pgmpy ran 79 instances and answered 43 (3 fail, 2 run out of
memory, 31 time out). Its answers differ from the reference's by at most
2.6e-06 relative (4.5e-07 absolute, clinicalTrial1_w12) over 43
instances. Over the 79 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, pgmpy takes 24601 s
in all and tddlog 1190 s; over the 43 both answer, 3001 s against
tddlog's 82. tddlog alone answers GPA_w19, GPA_w21, addFun_max_p4,
addFun_max_w17, addFun_sum_w17 and 31 more.

**libDAI.** libDAI ran 80 instances and answered 44 (4 fail, 13 run out
of memory, 19 time out). Its answers differ from the reference's by at
most 1.0e+00 relative (4.3e-13 absolute, population_s1) over 44
instances. Over the 80 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, libDAI takes 24101
s in all and tddlog 1212 s; over the 44 both answer, 2501 s against
tddlog's 35.4. libDAI is faster on 4 (clickGraph_w8 0.24 s against 0.39,
clickGraph_w10 0.38 s against 0.59, clickGraph_w12 0.55 s against 0.77
and clickGraph_w14 0.76 s against 1.16). tddlog alone answers GPA_w19,
addFun_max_p4, addFun_max_w17, addFun_sum_w17, altermu2_p16 and 31 more.

**TensorInference.jl greedy.** TensorInference.jl greedy ran 64
instances and answered 25 (12 fail, 12 run out of memory, 15 time out).
Its answers differ from the reference's by at most 9.3e-07 relative
(1.7e-16 absolute, spacex_w13) over 25 instances. Over the 64 instances
given to both that either answers, a run that does not answer counted at
the 600 s limit, TensorInference.jl greedy takes 23488 s in all and
tddlog 899 s; over the 25 both answer, 87.9 s against tddlog's 8.82.
TensorInference.jl greedy is faster on 4 (clickGraph_w10 0.31 s against
0.59, clickGraph_w16 0.63 s against 0.97, clickGraph_w12 0.41 s against
0.77 and clickGraph_w14 0.53 s against 1.16). tddlog alone answers
GPA_w19, GPA_w21, addFun_max_p4, addFun_max_w17, addFun_sum_w17 and 34
more.

**TensorInference.jl treesa.** TensorInference.jl treesa ran 68
instances and answered 32 (36 time out). Its answers differ from the
reference's by at most 9.3e-07 relative (2.2e-16 absolute, spacex_w13)
over 32 instances. Over the 67 instances given to both that either
answers, a run that does not answer counted at the 600 s limit,
TensorInference.jl treesa takes 25751 s in all and tddlog 940 s; over
the 32 both answer, 4751 s against tddlog's 76.2. tddlog alone answers
GPA_w19, GPA_w21, addFun_max_p16, addFun_max_w17, addFun_sum_w17 and 30
more.

**Roulette.** Roulette ran 137 instances and answered 97 (7 fail, 18 run
out of memory, 15 time out). Its answers differ from the reference's by
at most 1.1e-13 relative (1.2e-14 absolute, normal_mixture_mu2_w7) over
97 instances. Over the 134 instances given to both that either answers,
a run that does not answer counted at the 600 s limit, Roulette takes
28448 s in all and tddlog 2467 s; over the 97 both answer, 6248 s
against tddlog's 802. Roulette is faster on 22 (fair_dt44_w12 0.16 s
against 0.36, fair_dt16_w12 0.14 s against 0.4, fair_dt44_w32 0.24 s
against 0.55 and fair_dt44_w28 0.25 s against 0.6, the largest
differences). tddlog alone answers GPA_w25, GPA_w29, GPA_w31,
addFun_max_p4, addFun_max_w21 and 32 more.

| instance | flips | gates | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Scallop | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Ace log space | Merlin | pyAgrum | pgmpy | libDAI | TI greedy | TI treesa | Roulette | Roulette process | ProbLog | ref | tddlog diff | Geni diff | noDice diff | Psi diff | Genfer diff | SPPL diff | Pluck diff | PITA diff | aspmc diff | Scallop diff | Storm sparse diff | Storm exact diff | Storm dd diff | PRISM sparse diff | PRISM hybrid diff | Ace diff | Ace c2d diff | Ace log space diff | Merlin diff | pyAgrum diff | pgmpy diff | libDAI diff | TI greedy diff | TI treesa diff | Roulette diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GPA_w19 | 45061 | 368680 | 31.2 | err | err | >600 | >600 | err | >600 | >600 | >600 | >600 | err | >600 | >600 | >600 | >600 | oom | oom |  | >600 | >600 | >600 | >600 | >600 | >600 | 33.9 | 36.8 | oom | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |
| GPA_w21 | 61441 | 417832 | 34.8 | - | - | - | - | err | - | - | - | - | err | - | - | - | - | oom | oom |  | >600 | >600 | >600 | - | >600 | >600 | 34.3 | 37.4 | oom | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |
| GPA_w23 | 77812 | 466966 | 38.9 | - | - | - | - | err | - | - | - | - | err | - | - | - | - | >600 | oom |  | - | - | - | - | - | - | 41.9 | 45.9 | oom | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.1e-16 / 1.8e-16 |
| GPA_w25 | 94168 | 516070 | 44.5 | - | - | - | - | err | - | - | - | - | err | - | - | - | - | - | >600 |  | - | - | - | - | - | - | err | err | oom | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| GPA_w27 | 110529 | 565184 | 49.6 | - | - | - | - | err | - | - | - | - | err | - | - | - | - | - | - |  | - | - | - | - | - | - | 42.7 | 46.7 | oom | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.1e-16 / 1.8e-16 |
| GPA_w29 | 126871 | 614260 | 53.7 | - | - | - | - | err | - | - | - | - | oom | - | - | - | - | - | - |  | - | - | - | - | - | - | err | err | oom | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| GPA_w31 | 143187 | 663285 | 58.3 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | err | err | oom | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| addFun_max_w17 | 566 | 18099 | 18.4 | >600 | oom | >600 | oom | >600 | err | 555 | >600 | oom | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  | oom | oom | >600 | >600 | oom | >600 | 63.5 | 77.7 | >600 | alea | 2.2e-16 / 4.4e-16 |  |  |  |  |  |  | 1.1e-16 / 2.2e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.1e-16 / 2.2e-16 |
| addFun_max_w19 | 580 | 21035 | 40.9 | - | - | - | - | - | - | >600 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 297 | 318 | - | alea | 3.3e-16 / 6.7e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.2e-16 / 4.4e-16 |
| addFun_max_w21 | 594 | 23603 | 30.9 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | >600 | >600 | - | alea | 5.6e-16 / 1.1e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| addFun_max_p16 | 196 | 3801 | 0.918 | - | - | - | - | - | - | 201 | >600 | - | - | - | - | - | - | - | - |  | - | - | - | - | - | >600 | - | - | - | tddlog | ref |  |  |  |  |  |  | 3.3e-16 / 6.6e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| addFun_max_p4 | 168 | 1249 | 0.505 | >600 | oom | >600 | oom | >600 | err | 128 | 70 | oom | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  | oom | oom | >600 | err | err | 191 | oom | oom | oom | tddlog | ref |  |  |  |  |  |  | 3.3e-16 / 6.7e-16 | 3.3e-16 / 6.7e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 4.4e-16 / 8.9e-16 |  |
| addFun_max_p64 | 366 | 13777 | 5.29 | - | - | - | - | - | - | >600 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| addFun_sum_w17 | 152 | 2800 | 0.813 | >600 | >600 | >600 | oom | >600 | err | 24.7 | >600 | oom | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  | oom | err | err | oom | oom | >600 | 16.5 | 29 | >600 | alea | 7.8e-16 / 1.6e-15 |  |  |  |  |  |  | 2.2e-16 / 4.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.2e-16 / 4.4e-16 |
| addFun_sum_w19 | 168 | 3212 | 0.926 | - | - | - | - | - | - | 74 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 72.8 | 88.1 | - | alea | 5.6e-16 / 1.1e-15 |  |  |  |  |  |  | 3.3e-16 / 6.7e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.2e-16 / 4.4e-16 |
| addFun_sum_w21 | 184 | 3624 | 1.08 | - | - | - | - | - | - | 87.1 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 326 | 345 | - | alea | 6.7e-16 / 1.3e-15 |  |  |  |  |  |  | 3.3e-16 / 6.7e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.1e-16 / 2.2e-16 |
| addFun_sum_w23 | 196 | 3820 | 1.21 | - | - | - | - | - | - | 113 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | oom | oom | - | tddlog | ref |  |  |  |  |  |  | 4.4e-16 / 8.9e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| addFun_sum_w25 | 212 | 4208 | 1.36 | - | - | - | - | - | - | >600 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| altermu2_w11 | 20458 | 274380 | 23.1 | oom | oom | oom | >600 | err | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  | >600 | >600 | >600 | >600 | >600 | >600 | 175 | 193 | >600 | alea | 4.7e-14 / 9.4e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 4.7e-14 / 9.4e-14 |
| altermu2_w13 | 40822 | 336280 | 36.1 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | >600 | >600 | - | alea | 3.4e-14 / 6.8e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| altermu2_w15 | 60746 | 397700 | 73.1 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | alea | 1.6e-14 / 3.1e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| altermu2_w17 | 80350 | 457880 | oom | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| altermu2_w19 | 99554 | 517740 | - | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| altermu2_w21 | 117438 | 574640 | - | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| altermu2_w23 | 135402 | 631860 | - | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| altermu2_p16 | 7670 | 35720 | 3.86 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | >600 | >600 |  | oom | oom | >600 | >600 | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| altermu2_p4 | 2230 | 13800 | 2.49 | >600 | oom | >600 | oom | >600 | err | >600 | >600 | oom | >600 | >600 | >600 | >600 | >600 | 15.1 | 219 |  | 32.4 | 26.3 | 183 | 76.6 | oom | >600 | >600 | >600 | >600 | merlin | 4.9e-07 / 9.7e-07 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 4.9e-07 / 9.7e-07 | 4.9e-07 / 9.7e-07 |  | ref | 4.9e-07 / 9.7e-07 | 4.9e-07 / 9.7e-07 | 4.9e-07 / 9.7e-07 |  |  |  |
| clickGraph_w4 | 52 | 135 | 0.015 | >600 | 161 | >600 | oom | 488 | 76.4 | 0.417 | 2.81 | oom | 0.348 | 0.348 | 2.26 | 2.65 | 2.57 | 2.1 | 2.13 |  | 0.094 | 0.264 | 2.24 | 0.095 | 0.0772 | 15.9 | 0.021 | 1.45 | 0.155 | alea | 2.2e-16 / 3.7e-16 |  | 8.2e-07 / 1.2e-06 |  |  | 3.3e-16 / 5.5e-16 | 2.2e-16 / 3.7e-16 | 2.2e-16 / 3.7e-16 | 2.2e-16 / 3.7e-16 |  | 2.2e-16 / 3.7e-16 | 2.2e-16 / 3.7e-16 | 2.2e-16 / 3.7e-16 | 2.2e-16 / 3.7e-16 | 2.2e-16 / 3.7e-16 | 2.2e-16 / 3.7e-16 | 2.2e-16 / 3.7e-16 |  | 1.9e-07 / 3.1e-07 | 3.5e-13 / 5.7e-13 | 3.5e-13 / 5.7e-13 | 3.5e-13 / 5.7e-13 | 2.2e-16 / 3.7e-16 | 2.2e-16 / 3.7e-16 | 2.2e-16 / 3.7e-16 |
| clickGraph_w6 | 104 | 345 | 0.03 | - | >600 | - | - | >600 | err | 1.66 | 2.82 | - | 1.31 | 1.69 | 28.1 | 5.13 | 5.14 | 2.41 | 2.62 |  | 0.129 | 0.323 | 2.46 | 0.152 | 0.171 | 41.1 | 1.17 | 4.06 | 0.377 | alea | 2.2e-16 / 4.1e-16 |  |  |  |  |  |  | 5.7e-14 / 9.7e-14 | 1.1e-16 / 2.1e-16 |  | 2.2e-16 / 4.2e-16 | 2.2e-16 / 4.2e-16 | 2.2e-16 / 4.2e-16 | 2.2e-16 / 4.2e-16 | 2.2e-16 / 4.2e-16 | 2.2e-16 / 4.2e-16 | 2.2e-16 / 4.2e-16 |  | 4.8e-07 / 8.1e-07 | 4.4e-13 / 8.4e-13 | 4.4e-13 / 8.4e-13 | 4.4e-13 / 8.4e-13 | 2.2e-16 / 4.1e-16 | 2.2e-16 / 4.1e-16 | 2.2e-16 / 4.2e-16 |
| clickGraph_w8 | 156 | 555 | 0.389 | - | - | - | - | - | - | 5.21 | 3.32 | - | 23.7 | 36.3 | 178 | 16.5 | 14.8 | 2.64 | 2.89 |  | 0.174 | 0.417 | 3.03 | 0.241 | 0.43 | 68 | 172 | 179 | 1.53 | alea | 6.7e-16 / 9.9e-16 |  |  |  |  |  |  | 6.5e-14 / 1.3e-13 | 4.4e-16 / 8.7e-16 |  | 4.4e-16 / 6.6e-16 | 4.4e-16 / 8.5e-16 | 4.4e-16 / 6.6e-16 | 4.4e-16 / 6.6e-16 | 4.4e-16 / 6.6e-16 | 4.4e-16 / 8.5e-16 | 3.3e-16 / 6.5e-16 |  | 4.2e-07 / 8.0e-07 | 4.9e-13 / 9.7e-13 | 4.9e-13 / 9.7e-13 | 4.9e-13 / 9.7e-13 | 4.4e-16 / 8.5e-16 | 4.4e-16 / 6.6e-16 | 3.3e-16 / 6.5e-16 |
| clickGraph_w10 | 208 | 765 | 0.58 | - | - | - | - | - | - | 597 (2/3) | 3.34 | - | >600 | >600 | >600 | 279 | 295 | 2.88 | 3.39 |  | 0.23 | 0.549 | 3.98 | 0.376 | 0.315 | 99.2 | oom | oom | 5.92 | Ganak | 5.2e-15 / 1.0e-14 |  |  |  |  |  |  | 3.5e-14 / 6.8e-14 | 3.3e-16 / 6.1e-16 |  |  |  |  | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 3.3e-16 / 6.6e-16 |  | 4.0e-07 / 8.0e-07 | 4.8e-13 / 9.2e-13 | 4.8e-13 / 9.2e-13 | 4.8e-13 / 9.2e-13 | 1.1e-16 / 1.7e-16 | 1.1e-16 / 2.2e-16 |  |
| clickGraph_w12 | 260 | 975 | 0.762 | - | - | - | - | - | - | 114 (2/3) | 4.57 | - | - | - | - | >600 | >600 | 3.45 | 3.95 |  | 0.3 | 0.711 | 5.38 | 0.548 | 0.406 | 133 | - | - | 30.9 | Ganak | 8.3e-15 / 1.4e-14 |  |  |  |  |  |  | 4.6e-14 / 7.8e-14 | 3.3e-16 / 6.1e-16 |  |  |  |  |  |  | 4.4e-16 / 8.9e-16 | 2.2e-16 / 4.3e-16 |  | 4.6e-07 / 9.2e-07 | 4.8e-13 / 9.6e-13 | 4.8e-13 / 9.6e-13 | 4.8e-13 / 9.6e-13 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 1.7e-16 |  |
| clickGraph_w14 | 312 | 1185 | 1.16 | - | - | - | - | - | - | 431 (2/3) | 9.17 | - | - | - | - | - | - | 3.49 | 4.18 |  | 0.366 | 0.898 | 7.67 | 0.756 | 0.527 | 171 | - | - | wrong | Ganak | 1.2e-14 / 2.1e-14 |  |  |  |  |  |  | 3.3e-14 / 6.4e-14 | 3.3e-16 / 6.7e-16 |  |  |  |  |  |  | 3.3e-16 / 5.0e-16 | 4.4e-16 / 8.2e-16 |  | 4.6e-07 / 9.1e-07 | 4.9e-13 / 9.8e-13 | 4.9e-13 / 9.8e-13 | 4.9e-13 / 9.8e-13 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 |  |
| clickGraph_w16 | 364 | 1395 | 0.971 | - | - | - | - | - | - | >600 | 29.9 | - | - | - | - | - | - | 3.82 | 4.57 |  | 0.472 | 1.13 | 11 | 1.08 | 0.627 | 196 | - | - | oom | Ganak | 4.2e-15 / 8.4e-15 |  |  |  |  |  |  |  | 3.3e-16 / 6.7e-16 |  |  |  |  |  |  | 3.3e-16 / 6.7e-16 | 3.3e-16 / 6.7e-16 |  | 4.7e-07 / 9.5e-07 | 4.6e-13 / 7.8e-13 | 4.6e-13 / 7.8e-13 | 4.6e-13 / 7.8e-13 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 |  |
| clinicalTrial1_w4 | 807 | 3200 | 0.494 | >600 | >600 | >600 | oom | >600 | err | err | 166 | oom | >600 | >600 | 35.4 | err | err | 10.8 | 7.95 |  | 6.86 | 4.76 | 8.52 | 44.7 | oom | >600 | 0.719 | 1.42 | 1.51 | alea | 5.8e-15 / 2.9e-14 |  |  |  |  |  |  |  | 5.9e-15 / 2.9e-14 |  |  |  | 5.9e-15 / 2.9e-14 |  |  | 5.8e-15 / 2.9e-14 | 5.8e-15 / 2.9e-14 |  | 2.5e-07 / 1.2e-06 | 4.5e-13 / 2.2e-12 | 4.5e-13 / 2.2e-12 | 4.5e-13 / 2.2e-12 |  |  | 5.8e-15 / 2.9e-14 |
| clinicalTrial1_w6 | 1613 | 8000 | 1 | - | - | - | - | - | - | >600 | zero | - | - | - | 178 | err | err | 13.6 | wrong |  | 20.5 | 29.4 | 28.5 | 242 | - | - | 14.3 | 15.3 | 5.19 | alea | 1.1e-14 / 6.4e-14 |  |  |  |  |  |  |  |  |  |  |  | 1.1e-14 / 6.4e-14 |  |  | 1.1e-14 / 6.4e-14 | inf / inf |  | 4.5e-07 / 2.6e-06 | 7.7e-14 / 4.5e-13 | 7.7e-14 / 4.5e-13 | 7.7e-14 / 4.5e-13 |  |  | 1.1e-14 / 6.4e-14 |
| clinicalTrial1_w8 | 2419 | 12800 | 3.39 | - | - | - | - | - | - | - | 333 | - | - | - | >600 | err | err | 16.2 | wrong |  | 37.7 | 79.8 | 57.4 | 572 | - | - | oom | oom | 24.5 | alea | 4.4e-16 / 2.6e-15 |  |  |  |  |  |  |  | 5.6e-16 / 3.3e-15 |  |  |  |  |  |  | 4.7e-16 / 2.8e-15 | inf / inf |  | 4.5e-07 / 2.6e-06 | 2.5e-13 / 1.4e-12 | 2.5e-13 / 1.4e-12 | 2.5e-13 / 1.4e-12 |  |  |  |
| clinicalTrial1_w10 | 3225 | 17600 | 8.53 | - | - | - | - | - | - | - | >600 | - | - | - | - | err | err | 20.7 | wrong |  | 55.8 | 167 | 118 | >600 | - | - | - | - | 121 | Ganak | 1.1e-16 / 6.5e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.8e-17 / 1.6e-16 | inf / inf |  | 4.5e-07 / 2.6e-06 | 2.5e-13 / 1.4e-12 | 2.5e-13 / 1.4e-12 |  |  |  |  |
| clinicalTrial1_w12 | 4031 | 22400 | 12.2 | - | - | - | - | - | - | - | - | - | - | - | - | err | err | 28.8 | wrong |  | 77.3 | 393 | 212 | - | - | - | - | - | oom | merlin | 4.5e-07 / 2.6e-06 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 4.5e-07 / 2.6e-06 | inf / inf |  | ref | 4.5e-07 / 2.6e-06 | 4.5e-07 / 2.6e-06 |  |  |  |  |
| clinicalTrial1_w14 | 4837 | 27200 | 25.6 | - | - | - | - | - | - | - | - | - | - | - | - | err | err | err | wrong | 21.5 | oom | 495 | 268 | - | - | - | - | - | - | ace-Xlog | 4.4e-15 / 2.6e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | inf / inf |  |  | 2.5e-13 / 1.5e-12 | 2.5e-13 / 1.5e-12 |  |  |  |  |
| clinicalTrial2_w6 | 89 | 412 | 0.019 | >600 | 9.32 | err | oom | >600 | 38.5 | 0.801 | 2.97 | oom | 3.01 | 4.12 | 12.9 | 5.17 | 5.13 | 1.86 | 1.96 |  | 0.088 | 1.38 | 21.1 | 0.09 | 0.249 | 10.1 | 0.223 | 3.01 | 0.398 | alea | 6.7e-16 / 1.3e-15 |  | 7.4e-07 / 1.5e-06 |  |  |  | 6.7e-16 / 1.3e-15 | 6.7e-16 / 1.3e-15 | 6.7e-16 / 1.3e-15 |  | 6.7e-16 / 1.3e-15 | 6.7e-16 / 1.3e-15 | 6.7e-16 / 1.3e-15 | 6.7e-16 / 1.3e-15 | 6.7e-16 / 1.3e-15 | 6.7e-16 / 1.3e-15 | 6.7e-16 / 1.3e-15 |  | 3.9e-07 / 1.5e-06 | 3.7e-13 / 7.4e-13 | 3.7e-13 / 7.4e-13 | 3.7e-13 / 7.4e-13 | 6.7e-16 / 1.3e-15 | 5.6e-16 / 1.1e-15 | 6.7e-16 / 1.3e-15 |
| clinicalTrial2_w8 | 133 | 658 | 0.026 | - | >600 | err | - | - | 472 | 5.98 | 3.21 | - | 83.3 | 119 | 75.2 | 20.7 | 19.5 | 2.19 | 2.38 |  | 0.097 | 0.273 | 2.39 | 0.1 | 0.118 | 17.6 | 7.83 | 12.2 | 1.08 | alea | 5.6e-16 / 1.1e-15 |  |  |  |  |  | 5.6e-16 / 1.1e-15 | 1.2e-13 / 4.3e-13 | 6.7e-16 / 1.3e-15 |  | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 |  | 4.7e-07 / 9.3e-07 | 4.7e-13 / 1.2e-12 | 4.7e-13 / 1.2e-12 | 4.7e-13 / 1.2e-12 | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.3e-15 | 5.6e-16 / 1.1e-15 |
| clinicalTrial2_w10 | 177 | 904 | 0.032 | - | - | err | - | - | err | 29.8 | 6.56 | - | >600 | >600 | >600 | 452 | 466 | 2.28 | 2.44 |  | 0.111 | 0.28 | 2.68 | 0.117 | 0.128 | 25.1 | 288 | 298 | 4.74 | alea | 1.1e-15 / 2.2e-15 |  |  |  |  |  |  | 1.1e-13 / 9.5e-13 | 1.0e-15 / 2.0e-15 |  |  |  |  | 1.0e-15 / 2.0e-15 | 1.0e-15 / 2.0e-15 | 1.1e-15 / 2.2e-15 | 1.1e-15 / 2.2e-15 |  | 4.7e-07 / 4.0e-06 | 4.5e-13 / 1.4e-12 | 4.5e-13 / 1.4e-12 | 4.5e-13 / 1.4e-12 | 1.1e-15 / 2.2e-15 | 1.1e-15 / 2.2e-15 | 1.0e-15 / 2.0e-15 |
| clinicalTrial2_w12 | 221 | 1150 | 0.049 | - | - | err | - | - | - | 135 | 6.24 | - | - | - | - | >600 | >600 | 2.34 | 2.62 |  | 0.122 | 0.299 | 2.81 | 0.132 | 0.158 | 31.4 | oom | oom | 24 | alea | 3.6e-15 / 7.1e-15 |  |  |  |  |  |  | 1.0e-13 / 2.1e-13 | 4.4e-16 / 9.0e-16 |  |  |  |  |  |  | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 |  | 4.7e-07 / 4.3e-06 | 4.5e-13 / 9.0e-13 | 4.5e-13 / 9.0e-13 | 4.5e-13 / 9.0e-13 | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 |  |
| clinicalTrial2_w14 | 265 | 1396 | 0.052 | - | - | err | - | - | - | 339 | 11.8 | - | - | - | - | - | - | 2.45 | 2.77 |  | 0.142 | 0.334 | 3.13 | 0.16 | 0.167 | 39.7 | - | - | 118 | Ganak | 1.1e-16 / 2.2e-16 |  |  |  |  |  |  | 1.7e-13 / 3.3e-13 | 2.2e-16 / 4.5e-16 |  |  |  |  |  |  | 6.7e-16 / 1.3e-15 | 2.2e-16 / 4.4e-16 |  | 4.9e-07 / 4.4e-06 | 4.8e-13 / 1.4e-12 | 4.8e-13 / 1.4e-12 | 4.8e-13 / 1.4e-12 | 1.1e-16 / 2.2e-16 | 5.6e-17 / 1.3e-16 |  |
| clinicalTrial2_w16 | 309 | 1642 | 0.056 | - | - | err | - | - | - | 316 | 44.4 | - | - | - | - | - | - | 2.5 | 3.07 |  | 0.159 | 0.359 | 3.65 | 0.188 | 0.188 | 46.8 | - | - | oom | Ganak | 2.2e-16 / 4.4e-16 |  |  |  |  |  |  | 1.9e-13 / 1.5e-12 | 2.2e-16 / 4.4e-16 |  |  |  |  |  |  | 5.6e-16 / 1.1e-15 | 2.2e-16 / 4.4e-16 |  | 4.9e-07 / 3.5e-06 | 4.2e-13 / 3.4e-12 | 4.2e-13 / 3.4e-12 | 4.2e-13 / 3.4e-12 | 1.1e-16 / 2.5e-16 | 5.6e-17 / 1.3e-16 |  |
| clinicalTrial2_w18 | 353 | 1888 | 0.06 | - | - | err | - | - | - | 219 | 142 | - | - | - | - | - | - | 2.67 | 3.22 |  | 0.173 | 0.406 | 4.76 | 0.211 | 0.398 | 54 | - | - | - | Ganak | 2.2e-16 / 4.4e-16 |  |  |  |  |  |  | 1.5e-13 / 3.0e-13 | 2.2e-16 / 4.4e-16 |  |  |  |  |  |  | 1.7e-16 / 3.9e-16 | 5.0e-16 / 1.0e-15 |  | 4.2e-07 / 1.4e-06 | 4.4e-13 / 1.7e-12 | 4.4e-13 / 1.7e-12 | 4.4e-13 / 1.7e-12 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.3e-16 |  |
| coinBias_w16 | 12422 | 152450 | 14 | oom | oom | >600 | >600 | >600 | err | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  | >600 | >600 | >600 | >600 | >600 | >600 | 326 | 351 | oom | alea | 2.4e-15 / 5.3e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 6.1e-16 / 1.2e-15 |
| coinBias_w18 | 20630 | 177156 | 23 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | >600 | >600 | - | alea | 3.8e-15 / 6.1e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| conjugate_ex3_w18 | 9147 | 239177 | 51.8 | oom | oom | >600 | >600 | err | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  | >600 | >600 | >600 | >600 | >600 | >600 | 317 | 349 | oom | alea | 5.6e-15 / 1.0e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.6e-15 / 1.3e-14 |
| conjugate_ex3_w20 | 9135 | 290908 | 84.3 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | >600 | >600 | - | alea | 6.4e-15 / 1.3e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| conjugate_ex3_w22 | 9135 | 342186 | >600 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | alea |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| conjugate_ex3_w24 | 9117 | 392519 | - | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | alea |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| conjugate_ex3_w26 | 9105 | 442375 | - | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| conjugate_ex3_w28 | 9087 | 490533 | - | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| conjugate_ex3_w30 | 9066 | 538535 | - | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| conjugate_gaussians_w17 | 177 | 2788 | 0.624 | >600 | oom | >600 | oom | >600 | err | >600 | >600 | oom | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  | oom | err | >600 | oom | err | >600 | 63.9 | 76.6 | >600 | alea | 4.0e-15 / 2.1e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 3.9e-15 / 2.1e-14 |
| conjugate_gaussians_w19 | 201 | 3258 | 0.849 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 414 | 434 | - | alea | 2.4e-15 / 5.8e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.1e-15 / 6.4e-15 |
| conjugate_gaussians_w21 | 225 | 3728 | 1.1 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | >600 | >600 | - | alea | 4.3e-15 / 8.7e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| fair_dt14_w12 | 1795 | 9432 | 7.97 | >600 | oom | >600 | oom | >600 | 24.1 | 88.9 | 119 | oom | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  | err | oom | >600 | oom | oom | >600 | 0.844 | 2.21 | oom | alea | 2.8e-17 / 5.6e-16 |  |  |  |  |  | 6.9e-18 / 2.2e-16 | 8.3e-17 / 1.3e-15 | 2.8e-17 / 4.3e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 3.5e-18 / 1.1e-16 |
| fair_dt14_w16 | 3034 | 13225 | 14.6 | - | - | - | - | - | 37.2 | 538 | >600 | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 5.05 | 6.59 | - | alea | 5.2e-17 / 1.7e-15 |  |  |  |  |  | 4.2e-17 / 6.5e-16 | 1.0e-17 / 3.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 4.2e-17 / 6.5e-16 |
| fair_dt14_w20 | 4259 | 17021 | 30.4 | - | - | - | - | - | 58.7 | >600 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 84.9 | 87.9 | - | alea | 5.6e-17 / 1.0e-15 |  |  |  |  |  | 1.4e-17 / 4.5e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.4e-17 / 4.5e-16 |
| fair_dt14_w24 | 5272 | 20135 | 35.8 | - | - | - | - | - | 87.6 | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | oom | oom | - | tddlog | ref |  |  |  |  |  | 2.8e-17 / 6.7e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| fair_dt14_w28 | 6274 | 23227 | 51.2 | - | - | - | - | - | 128 | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  | 1.4e-17 / 4.5e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| fair_dt14_w32 | 7263 | 26293 | 84.4 | - | - | - | - | - | 171 | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  | 2.8e-17 / 4.3e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| fair_dt14_w36 | 8145 | 29141 | 86.4 | - | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| fair_dt16_w12 | 420 | 2143 | 0.417 | >600 | oom | >600 | oom | >600 | 17.3 | 1.89 | 8.56 | oom | >600 | >600 | 48.3 | >600 | >600 | 16 | 8.29 |  | 39 | oom | oom | 15.2 | oom | >600 | 0.141 | 1.53 | 3.9 | alea | 2.8e-17 / 2.8e-16 |  |  |  |  |  | 2.8e-17 / 2.8e-16 | 8.3e-17 / 4.3e-16 | 8.3e-17 / 4.3e-16 |  |  |  | 1.7e-16 / 8.7e-16 |  |  | 2.8e-17 / 2.8e-16 | 5.6e-17 / 4.2e-16 |  | 7.0e-08 / 1.1e-06 |  |  | 4.0e-13 / 2.1e-12 |  |  | 3.5e-17 / 6.9e-16 |
| fair_dt16_w16 | 638 | 2884 | 0.634 | - | - | - | - | - | 17.3 | 4.12 | 19.9 | - | - | - | 98.9 | - | - | >600 | 64.8 |  | oom | - | - | err | - | - | 0.185 | 1.63 | 3.92 | alea | 5.6e-17 / 2.9e-16 |  |  |  |  |  | 5.6e-17 / 2.9e-16 | 5.6e-17 / 2.9e-16 | 5.6e-17 / 2.9e-16 |  |  |  | 5.6e-17 / 2.9e-16 |  |  |  | 1.9e-16 / 1.4e-15 |  |  |  |  |  |  |  | 2.8e-17 / 2.8e-16 |
| fair_dt16_w20 | 793 | 3420 | 0.804 | - | - | - | - | - | 17.3 | 6.43 | 43.1 | - | - | - | >600 | - | - | - | >600 |  | - | - | - | - | - | - | 0.21 | 1.68 | 9.19 | alea | 8.3e-17 / 5.6e-16 |  |  |  |  |  | 5.6e-17 / 7.0e-16 | 1.1e-16 / 5.8e-16 | 8.3e-17 / 5.6e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 8.3e-17 / 7.0e-16 |
| fair_dt16_w24 | 780 | 3394 | 0.832 | - | - | - | - | - | 17.4 | 7.08 | 46.1 | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 0.211 | 1.65 | 8.96 | alea | 5.6e-17 / 2.9e-16 |  |  |  |  |  | 4.9e-17 / 9.8e-16 | 5.6e-17 / 9.8e-16 | 1.4e-16 / 7.3e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.6e-17 / 9.8e-16 |
| fair_dt16_w28 | 754 | 3342 | 0.699 | - | - | - | - | - | 17.4 | 6.79 | 38 | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 0.208 | 1.64 | 8.47 | alea | 2.8e-17 / 4.2e-16 |  |  |  |  |  | 3.5e-17 / 7.0e-16 | 1.4e-17 / 2.8e-16 | 2.8e-17 / 1.5e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.4e-17 / 2.8e-16 |
| fair_dt16_w32 | 741 | 3316 | 0.694 | - | - | - | - | - | 17.2 | 5.22 | 57 | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 0.201 | 1.59 | 8.2 | alea | 2.8e-17 / 4.2e-16 |  |  |  |  |  | 3.5e-17 / 7.0e-16 | 2.8e-17 / 5.6e-16 | 5.6e-17 / 4.2e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 3.5e-17 / 7.0e-16 |
| fair_dt4_w12 | 1871 | 9632 | 4.54 | >600 | oom | >600 | oom | >600 | 22.6 | 123 | 173 | oom | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  | oom | oom | err | oom | oom | >600 | 0.789 | 2.2 | 263 | alea | 2.8e-17 / 1.8e-16 |  |  |  |  |  | 0.0e+00 / 0.0e+00 | 8.3e-17 / 5.5e-16 | 2.8e-17 / 2.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.8e-17 / 2.4e-16 |
| fair_dt4_w16 | 3064 | 13311 | 15.3 | - | - | - | - | - | 32.2 | 253 | >600 | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 4.29 | 5.79 | >600 | alea | 2.8e-17 / 1.9e-16 |  |  |  |  |  | 2.8e-17 / 1.9e-16 | 5.6e-17 / 3.7e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.8e-17 / 1.9e-16 |
| fair_dt4_w20 | 4349 | 17315 | 31.2 | - | - | - | - | - | 50.6 | 414 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 72.2 | 74.7 | - | alea | 2.8e-17 / 2.5e-16 |  |  |  |  |  | 5.6e-17 / 4.9e-16 | 2.8e-17 / 2.5e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.6e-17 / 3.7e-16 |
| fair_dt4_w24 | 5360 | 20425 | 38.3 | - | - | - | - | - | 70.7 | >600 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | oom | oom | - | tddlog | ref |  |  |  |  |  | 8.3e-17 / 6.1e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| fair_dt4_w28 | 6358 | 23509 | 54.9 | - | - | - | - | - | 106 | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  | 2.8e-17 / 2.5e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| fair_dt4_w32 | 7345 | 26571 | 103 | - | - | - | - | - | 141 | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  | 5.6e-17 / 3.7e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| fair_dt4_w36 | 8223 | 29411 | 131 | - | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| fair_dt44_w12 | 478 | 2361 | 0.371 | >600 | >600 | >600 | oom | >600 | 17.3 | 2.78 | 10.6 | oom | >600 | >600 | 72.2 | >600 | >600 | 21.4 | 6.12 |  | 21.7 | oom | >600 | 12 | >600 | >600 | 0.16 | 1.62 | 2.06 | alea | 8.3e-17 / 5.8e-16 |  |  |  |  |  | 5.6e-17 / 4.6e-16 | 2.8e-17 / 4.6e-16 | 2.8e-17 / 4.6e-16 |  |  |  | 5.6e-17 / 3.2e-16 |  |  | 5.6e-17 / 3.2e-16 | 4.2e-17 / 6.9e-16 |  | 1.1e-07 / 7.5e-07 |  |  | 3.0e-13 / 1.7e-12 |  |  | 2.8e-17 / 4.6e-16 |
| fair_dt44_w16 | 696 | 3090 | 0.541 | - | - | - | - | - | 17.2 | 4.45 | 21.6 | - | - | - | 126 | - | - | 63.9 | 81 |  | oom | - | - | oom | - | - | 0.201 | 1.7 | 4.69 | alea | 2.8e-17 / 1.6e-16 |  |  |  |  |  | 8.3e-17 / 4.8e-16 | 6.9e-17 / 1.2e-15 | 5.6e-17 / 3.2e-16 |  |  |  | 5.6e-17 / 3.5e-16 |  |  | 9.2e-16 / 1.2e-14 | 5.3e-16 / 4.2e-15 |  |  |  |  |  |  |  | 5.6e-17 / 4.6e-16 |
| fair_dt44_w20 | 851 | 3614 | 0.723 | - | - | - | - | - | 17.5 | 7.06 | 61.6 | - | - | - | >600 | - | - | 342 | >600 |  | - | - | - | - | - | - | 0.255 | 1.72 | 8.11 | alea | 1.1e-16 / 6.4e-16 |  |  |  |  |  | 5.6e-17 / 8.1e-16 | 8.3e-17 / 4.8e-16 | 2.2e-16 / 1.3e-15 |  |  |  |  |  |  | 1.4e-15 / 2.3e-14 |  |  |  |  |  |  |  |  | 5.6e-17 / 4.6e-16 |
| fair_dt44_w24 | 838 | 3588 | 0.679 | - | - | - | - | - | 17.6 | 4.06 | 55.4 | - | - | - | - | - | - | 396 | - |  | - | - | - | - | - | - | 0.257 | 1.75 | 7.82 | alea | 8.3e-17 / 5.8e-16 |  |  |  |  |  | 5.6e-17 / 4.6e-16 | 5.6e-17 / 6.9e-16 | 1.1e-16 / 6.4e-16 |  |  |  |  |  |  | 3.2e-15 / 5.4e-14 |  |  |  |  |  |  |  |  | 5.6e-17 / 4.6e-16 |
| fair_dt44_w28 | 812 | 3536 | 0.662 | - | - | - | - | - | 17.4 | 7.96 | 52.7 | - | - | - | - | - | - | 426 | - |  | - | - | - | - | - | - | 0.247 | 1.72 | 7.58 | alea | 8.3e-17 / 4.8e-16 |  |  |  |  |  | 2.8e-17 / 2.3e-16 | 5.6e-17 / 3.2e-16 | 1.4e-16 / 8.0e-16 |  |  |  |  |  |  | 3.6e-16 / 2.1e-15 |  |  |  |  |  |  |  |  | 6.9e-18 / 1.2e-16 |
| fair_dt44_w32 | 799 | 3510 | 0.602 | - | - | - | - | - | 17.4 | 5.91 | 51.9 | - | - | - | - | - | - | 278 | - |  | - | - | - | - | - | - | 0.245 | 1.71 | 6.97 | alea | 2.8e-17 / 2.3e-16 |  |  |  |  |  | 2.8e-17 / 3.5e-16 | 8.3e-17 / 4.8e-16 | 2.8e-17 / 2.3e-16 |  |  |  |  |  |  | 2.8e-16 / 3.1e-15 |  |  |  |  |  |  |  |  | 2.8e-17 / 1.6e-16 |
| genes_t5 | 67284 | 308661 | 25.7 | oom | oom | >600 | >600 | err | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | err | err |  | >600 | >600 | >600 | >600 | >600 | >600 | oom | oom | oom | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| genes_t10 | 67484 | 309241 | 25.2 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | err | err |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| genes_t15 | 67684 | 309821 | 25.2 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | err | err |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| genes_t20 | 67884 | 310401 | 25.4 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | err | err |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| genes_t25 | 68084 | 310981 | 25.2 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | err | err |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| genes_t30 | 68284 | 311561 | 25 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | err | err |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| genes_t40 | 68684 | 312721 | 25.3 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | err | err |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| genes_t50 | 69084 | 313881 | 26 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | err | err |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| genes_t60 | 69484 | 315041 | 25.8 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | err | err |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| genes_t80 | 70284 | 317361 | 27 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | err | err |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| genes_t100 | 71084 | 319681 | 26.2 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | err | err |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| genes5_w17 | 24148 | 173021 | 12.8 | oom | oom | oom | >600 | err | err | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  | >600 | >600 | >600 | >600 | >600 | >600 | 71.1 | 83.1 | oom | alea | 5.4e-15 / 1.1e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.4e-15 / 1.1e-14 |
| genes5_w19 | 31844 | 196665 | 15.3 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 107 | 122 | - | alea | 7.5e-15 / 1.5e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 7.4e-15 / 1.5e-14 |
| genes5_w21 | 39336 | 219821 | 17.1 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 197 | 218 | - | alea | 6.9e-15 / 1.4e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 6.8e-15 / 1.4e-14 |
| genes5_w23 | 46572 | 242497 | 19.4 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 394 | 420 | - | alea | 3.7e-15 / 7.3e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 3.5e-15 / 7.0e-15 |
| genes5_w25 | 53588 | 264781 | 21.1 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | oom | oom | - | alea | 3.9e-15 / 8.4e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| genes5_w27 | 60520 | 286833 | 24.2 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | alea | 2.9e-15 / 7.3e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| laplace_w13 | 4125 | 110955 | 10.2 | oom | oom | >600 | >600 | >600 | err | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | 486 | 465 |  | >600 | >600 | >600 | >600 | >600 | >600 | 246 | 264 | >600 | alea | 4.6e-15 / 9.1e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 4.5e-15 / 9.0e-15 | 4.6e-15 / 9.1e-15 |  |  |  |  |  |  |  | 4.6e-15 / 9.2e-15 |
| laplace_w15 | 8145 | 158877 | >600 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | >600 | >600 |  | - | - | - | - | - | - | oom | oom | - | alea |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| linreg_w8 | 372 | 5404 | 0.397 | >600 | oom | err | oom | >600 | 274 | wrong | 74.2 | >600 | >600 | >600 | >600 | >600 | >600 | 6.25 | 8.32 |  | 1.97 | 3.79 | 77 | 6.17 | 2.84 | 362 | 4.87 | 16.8 | >600 | alea | 3.9e-15 / 2.8e-14 |  |  |  |  |  | 4.1e-15 / 2.8e-14 | 9.5e-03 / 1.0e+00 | 3.9e-15 / 2.8e-14 |  |  |  |  |  |  | 4.1e-15 / 2.9e-14 | 4.1e-15 / 2.9e-14 |  | 4.2e-07 / 1.0e+00 | 3.7e-13 / 1.8e-12 | 3.7e-13 / 1.8e-12 | 3.7e-13 / 1.8e-12 | 4.2e-15 / 1.4e-12 | 4.2e-15 / 1.4e-12 | 4.1e-15 / 2.8e-14 |
| linreg_w9 | 564 | 6754 | 0.702 | - | - | err | - | - | >600 | >600 | 143 | - | - | - | - | - | - | 12.8 | 13.8 |  | 27.8 | 80.9 | >600 | 21.9 | >600 | - | 18.5 | 32 | - | alea | 4.6e-15 / 1.6e-14 |  |  |  |  |  |  |  | 4.2e-15 / 1.5e-14 |  |  |  |  |  |  | 4.4e-15 / 1.4e-14 | 3.9e-15 / 1.5e-14 |  | 4.9e-07 / 1.0e+00 | 4.5e-13 / 2.1e-12 |  | 4.5e-13 / 2.1e-12 |  |  | 4.3e-15 / 1.5e-14 |
| linreg_w10 | 756 | 8204 | 7.28 | - | - | err | - | - | - | - | 249 | - | - | - | - | - | - | 41.3 | 48.3 |  | oom | oom | - | oom | - | - | 85.8 | 102 | - | alea | 6.3e-15 / 2.4e-14 |  |  |  |  |  |  |  | 6.2e-15 / 2.3e-14 |  |  |  |  |  |  | 6.1e-15 / 2.3e-14 | 6.0e-15 / 2.3e-14 |  |  |  |  |  |  |  | 6.1e-15 / 2.3e-14 |
| linreg_w11 | 948 | 9788 | 12.5 | - | - | err | - | - | - | - | >600 | - | - | - | - | - | - | 136 | 121 |  | - | - | - | - | - | - | 407 | 431 | - | alea | 6.3e-15 / 9.1e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.4e-15 / 1.0e-14 | 6.9e-15 / 1.0e-14 |  |  |  |  |  |  |  | 6.8e-15 / 9.3e-15 |
| linreg_w12 | 1140 | 11532 | 36.1 | - | - | err | - | - | - | - | - | - | - | - | - | - | - | 570 | 545 |  | - | - | - | - | - | - | >600 | >600 | - | alea | 4.9e-15 / 1.9e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.8e-15 / 1.8e-14 | 3.6e-15 / 1.8e-14 |  |  |  |  |  |  |  |  |
| linreg_w13 | 1332 | 13436 | 208 | - | - | err | - | - | - | - | - | - | - | - | - | - | - | >600 | >600 |  | - | - | - | - | - | - | - | - | - | alea | 8.0e-15 / 2.7e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| linreg_w14 | 1524 | 15500 | oom | - | - | 0.275 | - | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| linreg_w15 | 1716 | 17724 | - | - | - | 0.275 | - | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| markov_n10 | 2050 | 7887 | 1.28 | >600 | oom | >600 | oom | wrong | err | err | 155 | oom | >600 | >600 | 182 | >600 | >600 | 7.7 | 8.66 |  | 3.18 | 9.73 | 13.5 | 21 | 2.57 | >600 | 0.61 | 2.04 | 5.06 | alea | 1.7e-15 / 9.7e-15 |  |  |  |  | 6.7e-03 / 4.8e-02 |  |  | 1.7e-15 / 8.1e-15 |  |  |  | 1.7e-15 / 7.8e-15 |  |  | 1.7e-15 / 6.7e-15 | 1.7e-15 / 6.7e-15 |  | 2.8e-07 / 1.0e+00 | 2.3e-13 / 9.1e-13 | 2.3e-13 / 9.1e-13 | 2.3e-13 / 9.1e-13 | 1.5e-15 / 8.2e-15 |  | 1.4e-15 / 7.6e-15 |
| markov_n100 | 20500 | 78897 | 10.7 | - | - | - | - | wrong | - | >600 | - | - | - | - | - | - | - | err | err | 24.9 | - | >600 | >600 | - | - | - | - | - | 302 | alea | 4.0e-15 / 2.7e-12 |  |  |  |  | 1.8e-03 / 2.4e-01 |  |  |  |  |  |  |  |  |  |  |  | 7.2e-15 / 3.1e-12 |  |  |  |  |  |  |  |
| markov_n1000 | 205000 | 788997 | 99.5 | - | - | - | - | oom | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | alea | 4.1e-13 / 4.1e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| markov_n20 | 4100 | 15777 | 1.12 | - | - | - | - | wrong | - | err | 390 | - | - | - | >600 | - | - | 12.7 | 19 |  | 10.7 | 47.9 | 43.3 | 100 | >600 | - | 2.1 | 3.56 | 17 | alea | 1.2e-14 / 1.5e-14 |  |  |  |  | 6.9e-04 / 4.0e-02 |  |  | 1.2e-14 / 2.1e-14 |  |  |  |  |  |  | 1.2e-14 / 1.3e-14 | 1.2e-14 / 1.3e-14 |  | 4.4e-07 / 1.0e+00 | 5.2e-14 / 1.6e-12 | 5.2e-14 / 1.6e-12 | 5.2e-14 / 1.6e-12 |  |  | 1.2e-14 / 2.2e-14 |
| markov_n200 | 41000 | 157797 | 22.8 | - | - | - | - | oom | - | - | - | - | - | - | - | - | - | err | err | 50.1 | - | - | - | - | - | - | - | - | >600 | alea | 1.3e-13 / 9.0e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.2e-13 / 7.5e-12 |  |  |  |  |  |  |  |
| markov_n2000 | 410000 | 1577997 | 212 | - | - | - | - | oom | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | alea | 4.8e-13 / 1.0e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| markov_n50 | 10250 | 39447 | 4.4 | - | - | - | - | wrong | - | err | >600 | - | - | - | - | - | - | 19.8 | 46.3 |  | oom | 461 | 354 | >600 | - | - | >600 | >600 | 84.3 | alea | 2.8e-14 / 2.8e-14 |  |  |  |  | 3.9e-04 / 2.0e-01 |  |  |  |  |  |  |  |  |  | 2.7e-14 / 4.7e-13 | 2.8e-14 / 4.8e-13 |  |  | 2.1e-13 / 2.1e-13 | 2.1e-13 / 2.1e-13 |  |  |  |  |
| markov_n500 | 102500 | 394497 | 44.9 | - | - | - | - | oom | - | - | - | - | - | - | - | - | - | >600 | >600 | >600 | - | - | - | - | - | - | - | - | - | alea | 1.9e-13 / 4.6e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| multimodal_w11 | 4617 | 116721 | 501 | oom | oom | >600 | >600 | >600 | err | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  | oom | >600 | >600 | >600 | oom | >600 | >600 | >600 | >600 | alea | 9.1e-15 / 1.8e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| normal_mixture_mu1_w7 | 2583 | 25704 | 2.13 | >600 | oom | >600 | err | >600 | err | >600 | >600 | oom | >600 | >600 | >600 | >600 | >600 | err | err | 11.9 | 31.2 | 119 | 301 | 353 | >600 | >600 | 31.5 | 37.6 | >600 | alea | 2.9e-15 / 2.5e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.4e-14 / 1.4e-14 | 2.0e-07 / 1.0e+00 | 4.8e-13 / 8.1e-13 | 4.8e-13 / 8.1e-13 | 4.8e-13 / 8.1e-13 |  |  | 2.9e-15 / 2.4e-14 |
| normal_mixture_mu1_w9 | 5289 | 35812 | 202 | - | - | - | err | - | - | - | - | - | - | - | - | - | - | >600 | >600 | err | oom | >600 | >600 | >600 | - | - | oom | oom | - | alea | 1.3e-14 / 1.2e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| normal_mixture_mu1_w11 | 7995 | 45920 | >600 | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - | - | err | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| normal_mixture_mu1_w13 | 10701 | 56028 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| normal_mixture_mu1_w15 | 13407 | 66136 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| normal_mixture_mu2_w7 | 2583 | 45770 | 3.14 | >600 | oom | >600 | err | >600 | err | >600 | >600 | oom | >600 | >600 | >600 | >600 | >600 | err | err | 17 | 49.2 | >600 | >600 | >600 | oom | >600 | 30.5 | 36.3 | >600 | alea | 1.3e-14 / 1.1e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 4.3e-14 / 8.2e-13 | 9.7e-08 / 1.0e+00 |  |  |  |  |  | 1.2e-14 / 1.1e-13 |
| normal_mixture_mu2_w9 | 7913 | 63750 | 199 | - | - | - | err | - | - | - | - | - | - | - | - | - | - | >600 | >600 | err | oom | - | - | - | - | - | oom | oom | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| normal_mixture_mu2_w11 | 13243 | 81730 | >600 | - | - | - | >600 | - | - | - | - | - | - | - | - | - | - | - | - | err | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| normal_mixture_theta_w9 | 10537 | 118560 | 40.8 | oom | oom | >600 | >600 | >600 | err | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  | oom | >600 | >600 | >600 | err | >600 | oom | oom | >600 | Ganak | 2.2e-16 / 5.2e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| pedestrian_s10 | 89 | 3176 | 2.19 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 71.2 | 74 |  | - | - | - | - | err | 330 | - | - | - | alea | 3.3e-16 / 4.3e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.7e-15 / 4.3e-14 | 6.1e-16 / 4.3e-14 |  |  |  |  |  |  | 2.2e-16 / 3.4e-10 |  |
| pedestrian_s12 | 99 | 3784 | 6.06 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 53.9 | 56.1 |  | - | - | - | - | - | >600 | - | - | - | alea | 4.4e-16 / 4.3e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.3e-15 / 4.3e-14 | 1.9e-15 / 4.3e-14 |  |  |  |  |  |  |  |  |
| pedestrian_s16 | 119 | 5000 | 12 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 166 | 317 |  | - | - | - | - | - | - | - | - | - | alea | 3.3e-16 / 4.2e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.3e-15 / 4.2e-14 | 6.7e-16 / 4.1e-14 |  |  |  |  |  |  |  |  |
| pedestrian_s20 | 139 | 6216 | 79 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | >600 | 467 |  | - | - | - | - | - | - | - | - | - | alea | 8.9e-16 / 4.9e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.0e-15 / 5.1e-14 |  |  |  |  |  |  |  |  |
| pedestrian_s24 | 159 | 7432 | 145 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | >600 |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| pedestrian_s4 | 59 | 1393 | 0.352 | >600 | oom | >600 | oom | >600 | >600 | 51.3 | 54.5 | oom | >600 | >600 | >600 | >600 | >600 | 15.1 | 11.5 |  | 25.9 | 4.88 | 22.2 | 11.7 | 14.7 | 122 | 3.95 | 8.4 | >600 | alea | 6.7e-16 / 2.7e-14 |  |  |  |  |  |  | 1.4e-14 / 1.0e+00 | 6.7e-16 / 2.7e-14 |  |  |  |  |  |  | 6.7e-16 / 2.7e-14 | 1.3e-15 / 2.8e-14 |  | 4.5e-07 / 1.0e+00 | 2.5e-13 / 1.2e-12 | 2.5e-13 / 1.2e-12 | 2.5e-13 / 1.2e-12 | 5.0e-16 / 3.4e-10 | 5.0e-16 / 3.4e-10 | 5.0e-16 / 2.7e-14 |
| pedestrian_s6 | 69 | 1968 | 0.516 | - | - | - | - | - | - | >600 | >600 | - | - | - | - | - | - | 41.6 | 18.1 |  | 94.7 | 149 | 96.1 | 41.1 | 48.5 | 182 | 260 | 265 | - | alea | 7.8e-16 / 4.0e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 6.1e-16 / 4.0e-14 | 6.7e-16 / 4.0e-14 |  | 2.6e-07 / 1.0e+00 | 5.0e-13 / 3.1e-12 | 5.0e-13 / 3.1e-12 | 5.0e-13 / 3.1e-12 | 2.8e-16 / 3.4e-10 | 3.3e-16 / 3.4e-10 | 3.1e-16 / 4.0e-14 |
| pedestrian_s8 | 79 | 2568 | 2.23 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 25.2 | 24.6 |  | oom | oom | >600 | oom | err | 254 | >600 | >600 | - | alea | 6.7e-16 / 4.1e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.9e-15 / 4.1e-14 | 7.8e-16 / 4.1e-14 |  |  |  |  |  |  | 5.6e-16 / 3.4e-10 |  |
| pi_w8 | 12 | 3759 | 0.168 | >600 | oom | >600 | >600 | >600 | 8.63 | 1.51 | 14.9 | oom | 6.85 | 8.06 | 15.2 | >600 | >600 | 4.23 | 3.73 |  | 0.91 | 0.561 | 79.1 | 0.962 | err | 67.7 | 0.101 | 0.78 | 2.39 | alea | 1.1e-16 / 2.4e-16 |  |  |  |  |  | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 |  | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 |  |  | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 |  | 4.7e-08 / 1.0e-07 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 |  | 1.7e-16 / 3.6e-16 | 1.1e-16 / 2.4e-16 |
| pi_w10 | 16 | 6141 | 0.293 | - | - | - | - | - | 13.5 | 2.23 | 49.9 | - | 294 | 334 | 66.5 | - | - | 18.4 | 18.5 |  | 30.9 | 3.88 | >600 | 24.7 | err | 133 | 0.287 | 0.985 | 27.2 | alea | 5.6e-17 / 1.2e-16 |  |  |  |  |  | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 |  | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 |  |  | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 |  | 3.7e-07 / 7.9e-07 | 6.2e-14 / 1.3e-13 |  | 6.2e-14 / 1.3e-13 |  | 1.1e-16 / 2.4e-16 | 5.6e-17 / 1.2e-16 |
| pi_w12 | 20 | 9099 | 0.688 | - | - | - | - | - | 58.6 | 4.61 | 182 | - | >600 | >600 | err | - | - | 241 | 242 |  | oom | oom | - | err | - | >600 | 1.5 | 2.22 | wrong | alea | 1.1e-16 / 2.4e-16 |  |  |  |  |  | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 |  |  |  |  |  |  | 1.1e-16 / 2.4e-16 | 1.1e-16 / 2.4e-16 |  |  |  |  |  |  |  | 1.1e-16 / 2.4e-16 |
| pi_w14 | 24 | 12633 | 1.63 | - | - | - | - | - | err | 26.6 | 498 | - | - | - | >600 | - | - | >600 | >600 |  | - | - | - | - | - | - | 11.5 | 12.4 | >600 | alea | 5.6e-17 / 1.2e-16 |  |  |  |  |  |  | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.6e-17 / 1.2e-16 |
| pi_w16 | 28 | 16743 | 7.33 | - | - | - | - | - | - | >600 | >600 | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 84.6 | 86.5 | - | alea | 1.1e-16 / 2.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.1e-16 / 2.4e-16 |
| pi_w18 | 32 | 21429 | 37.4 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | oom | oom | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| pi_w22 | 40 | 32529 | >600 | - | - | - | - | - | - | - | - | - | - | - | >600 | - | - | - | - |  | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| population_s1 | 2540 | 8332 | 0.841 | >600 | >600 | >600 | 0.009 | err | err | >600 | >600 | oom | >600 | >600 | >600 | >600 | >600 | 14.2 | 58.2 | 12.4 | 15.8 | 170 | >600 | 187 | oom | >600 | >600 | >600 | >600 | alea | 1.1e-15 / 9.2e-14 |  |  |  | 1.4e-14 / 3.9e-11 |  |  |  |  |  |  |  |  |  |  | 1.1e-15 / 1.0e+00 | 1.2e-15 / 1.0e+00 | 2.2e-14 / 1.0e+00 | 3.6e-07 / 1.0e+00 | 4.3e-13 / 1.0e+00 |  | 4.3e-13 / 1.0e+00 |  |  |  |
| population_s12 | 34957 | 115507 | 26.1 | - | - | - | 0.06 | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  | 6.1e-12 / 9.3e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| population_s16 | 46780 | 154582 | 47.2 | - | - | - | 0.093 | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  | 6.1e-12 / 9.3e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| population_s2 | 5587 | 18373 | 11.6 | - | - | - | 0.012 | err | - | - | - | - | - | - | - | - | - | err | err | 14.8 | oom | oom | - | >600 | - | - | - | - | - | alea | 3.7e-15 / 6.6e-15 |  |  |  | 2.3e-14 / 1.1e-13 |  |  |  |  |  |  |  |  |  |  |  |  | 6.7e-14 / 1.3e-13 |  |  |  |  |  |  |  |
| population_s24 | 70426 | 232732 | 36 | - | - | - | 0.196 | - | - | - | - | - | - | - | - | - | - | >600 | >600 |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  | 6.2e-12 / 9.3e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| population_s3 | 8637 | 28421 | 11.8 | - | - | - | 0.017 | >600 | - | - | - | - | - | - | - | - | - | err | >600 | 23.7 | - | - | - | - | - | - | - | - | - | ace-Xlog | 5.6e-14 / 7.5e-13 |  |  |  | 8.0e-12 / 9.7e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| population_s32 | 94072 | 310882 | 48.7 | - | - | - | 0.349 | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  | 6.2e-12 / 9.3e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| population_s4 | 11311 | 37357 | 13.4 | - | - | - | 0.018 | - | - | - | - | - | - | - | - | - | - | err | - | 30.7 | - | - | - | - | - | - | - | - | - | ace-Xlog | 1.0e-13 / 3.5e+05 |  |  |  | 6.4e-12 / 3.5e+05 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| population_s48 | 141364 | 467182 | 74.4 | - | - | - | 0.867 | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  | 6.2e-12 / 9.3e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| population_s6 | 17410 | 57448 | 16.8 | - | - | - | 0.026 | - | - | - | - | - | - | - | - | - | - | >600 | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  | 1.2e-13 / 2.3e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| population_s64 | 188656 | 623482 | 109 | - | - | - | 1.73 | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  | 6.2e-12 / 9.3e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| population_s8 | 23134 | 76432 | 38.8 | - | - | - | 0.034 | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  | 6.1e-12 / 9.3e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| spacex_w13 | 279 | 2037 | 0.716 | >600 | >600 | >600 | oom | >600 | err | >600 | 308 | oom | >600 | >600 | >600 | >600 | >600 | 13.1 | 314 |  | 15 | 171 | >600 | 4.24 | 3.87 | 195 | 3.37 | 12.3 | >600 | alea | 2.2e-16 / 9.4e-16 |  |  |  |  |  |  |  | 2.2e-16 / 1.1e-15 |  |  |  |  |  |  | 3.3e-16 / 7.5e-16 | 1.8e-15 / 2.6e-15 |  | 3.4e-07 / 1.0e+00 | 4.7e-13 / 9.7e-13 |  | 4.7e-13 / 9.7e-13 | 1.7e-16 / 9.3e-07 | 2.2e-16 / 9.3e-07 | 2.8e-16 / 9.4e-16 |
| spacex_w15 | 357 | 2423 | 5.41 | - | - | - | - | - | - | - | >600 | - | - | - | - | - | - | >600 | >600 |  | 163 | oom | - | 281 | err | 255 | 18.4 | 28.8 | - | alea | 5.4e-15 / 8.7e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 3.7e-07 / 1.0e+00 |  |  | 5.0e-13 / 8.1e-13 |  | 6.1e-16 / 9.0e-07 | 2.2e-16 / 4.1e-15 |
| spacex_w17 | 435 | 2809 | 33.4 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |  | err | - | - | oom | - | >600 | 161 | 175 | - | alea | 2.8e-14 / 7.0e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.7e-16 / 8.5e-15 |
| spacex_w19 | 513 | 3195 | 130 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | >600 | >600 | - | alea | 8.5e-14 / 2.2e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| spacex_p16 | 293 | 1752 | 57.7 | - | - | - | - | - | err | - | >600 | - | >600 | - | - | >600 | >600 | >600 | >600 |  | oom | oom | >600 | oom | err | 222 | - | - | - | alea | 6.2e-14 / 1.9e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.8e-16 / 4.4e-07 |  |
| spacex_p4 | 106 | 635 | 0.093 | >600 | oom | >600 | oom | >600 | 465 | >600 | 3.11 | oom | 300 | >600 | >600 | 94.7 | 74.5 | 2.05 | 2.44 |  | 0.298 | 0.415 | 18.9 | 0.34 | 0.522 | 61 | >600 | >600 | >600 | alea | 6.7e-16 / 1.6e-14 |  |  |  |  |  | 4.4e-16 / 1.7e-14 |  | 2.2e-16 / 1.7e-14 |  | 4.1e-15 / 2.4e-14 |  |  | 3.3e-16 / 1.7e-14 | 3.3e-16 / 1.7e-14 | 2.2e-16 / 1.7e-14 | 2.8e-16 / 1.7e-14 |  | 4.8e-07 / 1.0e+00 | 4.1e-13 / 8.2e-13 | 4.1e-13 / 8.2e-13 | 4.1e-13 / 8.2e-13 | 3.3e-16 / 2.1e-08 | 2.2e-16 / 2.1e-08 |  |
| spacex_p64 | 920 | 6092 | oom | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | >600 | - | - | - | alea |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| trueskill_w8 | 312 | 3868 | 3.24 | >600 | >600 | >600 | oom | >600 | err | >600 | 551 | oom | >600 | >600 | >600 | >600 | >600 | 22.5 | 256 |  | 12.1 | err | oom | 21.6 | err | >600 | 6.46 | 7.34 | >600 | alea | 1.7e-16 / 3.4e-16 |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  | 2.2e-16 / 4.6e-16 | 7.2e-16 / 1.5e-15 |  | 5.6e-08 / 1.2e-07 |  |  | 1.4e-13 / 3.0e-13 |  |  | 2.2e-16 / 4.6e-16 |
| trueskill_w10 | 376 | 5592 | 61.2 | - | - | - | - | - | - | - | >600 | - | - | - | - | - | - | >600 | >600 |  | oom | - | - | err | - | - | oom | oom | - | alea | 1.7e-16 / 3.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| tug_of_war_w16 | 392 | 2044 | 0.542 | >600 | >600 | >600 | oom | >600 | 10 | 10.3 | 386 | oom | >600 | >600 | 23.8 | 166 | 168 | >600 | >600 |  | >600 | oom | >600 | oom | >600 | >600 | 15.7 | 16.7 | >600 | alea | 1.1e-15 / 2.1e-15 |  |  |  |  |  | 1.0e-15 / 2.0e-15 | 4.3e-15 / 8.5e-15 | 1.1e-15 / 2.1e-15 |  |  |  | 1.1e-15 / 2.2e-15 | 2.1e-15 / 4.1e-15 | 2.1e-15 / 4.1e-15 |  |  |  |  |  |  |  |  |  | 8.9e-16 / 1.8e-15 |
| tug_of_war_w18 | 456 | 2292 | 0.617 | - | - | - | - | - | 11.2 | >600 | >600 | - | - | - | 26.1 | 258 | 252 | - | - |  | - | - | - | - | - | - | 68.1 | 69.9 | - | alea | 2.2e-16 / 4.4e-16 |  |  |  |  |  | 6.7e-16 / 1.3e-15 |  |  |  |  |  | 7.2e-16 / 1.4e-15 | 7.8e-16 / 1.6e-15 | 7.8e-16 / 1.6e-15 |  |  |  |  |  |  |  |  |  | 6.7e-16 / 1.3e-15 |
| tug_of_war_w20 | 520 | 2540 | 0.735 | - | - | - | - | - | 12.4 | - | - | - | - | - | 30.5 | 334 | 329 | - | - |  | - | - | - | - | - | - | oom | oom | - | alea | 8.3e-16 / 1.7e-15 |  |  |  |  |  | 7.8e-16 / 1.6e-15 |  |  |  |  |  | 6.7e-16 / 1.3e-15 | 6.1e-16 / 1.2e-15 | 6.1e-16 / 1.2e-15 |  |  |  |  |  |  |  |  |  |  |
| tug_of_war_w22 | 550 | 2720 | 0.771 | - | - | - | - | - | 13.8 | - | - | - | - | - | 32.6 | 434 | 428 | - | - |  | - | - | - | - | - | - | - | - | - | Ganak | 5.0e-16 / 1.0e-15 |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  | 5.6e-17 / 1.1e-16 | 5.6e-17 / 1.1e-16 | 5.6e-17 / 1.1e-16 |  |  |  |  |  |  |  |  |  |  |
| tug_of_war_w24 | 610 | 2960 | 0.472 | - | - | - | - | - | 14.8 | - | - | - | - | - | 33.6 | 391 | 304 | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  | 5.6e-17 / 1.1e-16 |  |  |  |  |  | 0.0e+00 / 0.0e+00 | 5.6e-17 / 1.1e-16 | 5.6e-17 / 1.1e-16 |  |  |  |  |  |  |  |  |  |  |
| tug_of_war_w26 | 670 | 3200 | 0.945 | - | - | - | - | - | 16.3 | - | - | - | - | - | 30.7 | 349 | 346 | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  | 7.8e-16 / 1.6e-15 |  |  |  |  |  | 7.8e-16 / 1.6e-15 | 4.9e-15 / 9.9e-15 | 4.9e-15 / 9.9e-15 |  |  |  |  |  |  |  |  |  |  |
| tug_of_war_w28 | 722 | 3416 | 1.02 | - | - | - | - | - | 17.5 | - | - | - | - | - | 34.7 | 479 | 482 | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  | 8.3e-16 / 1.7e-15 |  |  |  |  |  | 9.4e-16 / 1.9e-15 | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 |  |  |  |  |  |  |  |  |  |  |
| weekend_w18 | 49151 | 368566 | 25.2 | oom | oom | >600 | >600 | err | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 |  | >600 | >600 | >600 | >600 | >600 | >600 | 34.9 | 37.9 | oom | alea | 3.3e-16 / 8.9e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 3.9e-16 / 1.0e-15 |
| weekend_w20 | 65535 | 417718 | 30 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 35.2 | 38.4 | - | alea | 2.8e-16 / 7.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.8e-16 / 7.4e-16 |
| weekend_w22 | 81919 | 466870 | 33.7 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 42.6 | 46.6 | - | alea | 5.6e-17 / 1.5e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.7e-16 / 4.5e-16 |
| weekend_w24 | 98303 | 516022 | 37.5 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 44.8 | 48.8 | - | alea | 8.3e-16 / 2.2e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 7.8e-16 / 2.1e-15 |
| weekend_w26 | 114687 | 565174 | 42.6 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 44.4 | 48.5 | - | alea | 9.4e-16 / 2.5e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 8.3e-16 / 2.2e-15 |
| weekend_w28 | 131071 | 614326 | 44.7 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | err | err | - | alea | 6.1e-16 / 1.6e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| weekend_w30 | 147455 | 663478 | 49.2 | - | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | err | err | - | alea | 1.6e-15 / 4.2e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| weekend_p1024 | 28671 | 131010 | 8.29 | - | - | - | - | - | 98.2 | 299 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 6.89 | 8.22 | oom | alea | 2.8e-16 / 7.4e-16 |  |  |  |  |  | 1.1e-16 / 3.0e-16 | 7.9e-11 / 2.1e-10 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.1e-16 / 3.0e-16 |
| weekend_p16 | 639 | 2118 | 0.134 | - | 0.429 | - | - | - | 8.38 | 1.19 | 6.34 | - | - | - | 13.8 | >600 | >600 | 35.8 | 3.69 |  | 45.3 | 15.3 | 201 | 46.3 | err | 358 | 0.075 | 0.798 | 1.05 | alea | 5.0e-16 / 1.3e-15 |  | 2.8e-07 / 7.5e-07 |  |  |  | 5.0e-16 / 1.3e-15 | 2.7e-10 / 7.2e-10 | 6.1e-16 / 1.6e-15 |  |  |  | 2.2e-16 / 5.9e-16 |  |  | 4.4e-16 / 1.2e-15 | 2.8e-16 / 7.4e-16 |  | 2.8e-07 / 7.5e-07 | 5.2e-14 / 1.4e-13 | 5.2e-14 / 1.4e-13 | 5.2e-14 / 1.4e-13 |  | 4.4e-16 / 1.2e-15 | 5.0e-16 / 1.3e-15 |
| weekend_p16384 | 327679 | 2031530 | 196 | - | - | - | - | - | - | >600 | - | - | - | - | - | - | - | >600 | - |  | - | - | - | - | - | - | err | err | - | alea | 6.1e-16 / 1.6e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| weekend_p256 | 8191 | 33230 | 4.67 | - | oom | - | - | - | 15.6 | 24.6 | - | - | - | - | >600 | - | - | - | - |  | - | - | - | - | - | - | 1.42 | 2.24 | 276 | alea | 1.4e-15 / 3.7e-15 |  |  |  |  |  | 1.3e-15 / 3.4e-15 | 5.5e-10 / 1.5e-09 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.3e-15 / 3.4e-15 |
| weekend_p4 | 175 | 530 | 0.028 | >600 | 0.167 | >600 | oom | >600 | 6.98 | 0.675 | 2.81 | >600 | >600 | >600 | 4.41 | 220 | 210 | 2.06 | 2.21 |  | 0.352 | 1.55 | 19.1 | 0.613 | 1.16 | 70.1 | 0.031 | 0.743 | 0.283 | alea | 1.7e-15 / 4.4e-15 |  | 4.9e-07 / 1.3e-06 |  |  |  | 1.4e-15 / 3.7e-15 | 1.0e-09 / 2.7e-09 | 1.4e-15 / 3.7e-15 |  |  |  | 1.2e-15 / 3.1e-15 | 1.8e-15 / 4.9e-15 | 1.8e-15 / 4.9e-15 | 1.4e-15 / 3.7e-15 | 1.4e-15 / 3.7e-15 |  | 4.9e-07 / 1.3e-06 | 3.8e-13 / 1.0e-12 | 3.8e-13 / 1.0e-12 | 3.8e-13 / 1.0e-12 | 1.6e-15 / 4.2e-15 | 1.4e-15 / 3.7e-15 | 1.4e-15 / 3.7e-15 |
| weekend_p4096 | 98303 | 516022 | 39.1 | - | - | - | - | - | >600 | >600 | - | - | - | - | - | - | - | oom | - |  | - | - | - | - | - | - | 43.2 | 47.1 | - | alea | 8.3e-16 / 2.2e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 7.8e-16 / 2.1e-15 |
| weekend_p64 | 2303 | 8410 | 1.98 | - | 1.88 | - | - | - | 8.99 | 7.49 | >600 | - | - | - | 114 | - | - | >600 | >600 |  | oom | oom | >600 | oom | - | >600 | 0.321 | 1.05 | 10.9 | alea | 5.6e-17 / 1.5e-16 |  | 2.0e-07 / 5.3e-07 |  |  |  | 1.1e-16 / 3.0e-16 | 2.7e-11 / 7.2e-11 |  |  |  |  | 5.6e-17 / 1.5e-16 |  |  |  |  |  |  |  |  |  |  |  | 1.1e-16 / 3.0e-16 |
| weekend_p65536 | 1048575 | 7995294 | oom | - | - | - | - | - | - | >600 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | err | err | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| wmi_path_n1024 | 8192 | 91060 | 9.33 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 93.9 | - |  | - | - | >600 | - | - | - | - | - | - | ace-auto | 1.6e-15 / 2.2e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| wmi_path_n128 | 1024 | 11316 | 1.16 | - | - | - | - | - | - | - | - | - | - | - | >600 | - | - | 19 | err |  | 10.8 | 107 | 29.6 | 57 | >600 | - | 16.5 | 17.6 | - | alea | 1.0e-15 / 1.4e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 7.8e-16 / 1.1e-15 |  |  | 2.5e-08 / 3.5e-08 | 3.6e-13 / 5.1e-13 | 3.6e-13 / 5.1e-13 | 3.6e-13 / 5.1e-13 |  |  | 1.0e-15 / 1.4e-15 |
| wmi_path_n16 | 128 | 1348 | 0.137 | >600 | >600 | >600 | oom | >600 | 39.4 | >600 | 51.4 | oom | >600 | >600 | 27.7 | 48.1 | 51.5 | 5.06 | 11.6 |  | 0.534 | 1.35 | 3.11 | 1.32 | 0.629 | 212 | 0.312 | 1.04 | >600 | alea | 1.1e-16 / 1.6e-16 |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  | 3.3e-16 / 4.7e-16 |  |  |  | 2.2e-16 / 3.1e-16 | 2.2e-16 / 3.1e-16 | 2.2e-16 / 3.1e-16 | 3.3e-16 / 4.7e-16 | 1.1e-16 / 1.6e-16 |  | 1.4e-08 / 2.0e-08 | 3.5e-13 / 5.0e-13 | 3.5e-13 / 5.0e-13 | 3.5e-13 / 5.0e-13 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.6e-16 | 0.0e+00 / 0.0e+00 |
| wmi_path_n16384 | 131072 | 1458100 | 172 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | >600 | >600 |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| wmi_path_n256 | 2048 | 22708 | 2.15 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 31.5 | err |  | 36.7 | 325 | 114 | 223 | - | - | 67.7 | 70.1 | - | alea | 5.0e-15 / 7.1e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.8e-15 / 8.2e-15 |  |  | 2.5e-08 / 3.5e-08 | 3.6e-13 / 5.0e-13 | 3.6e-13 / 5.0e-13 | 3.6e-13 / 5.0e-13 |  |  | 5.9e-15 / 8.3e-15 |
| wmi_path_n32 | 256 | 2772 | 0.276 | - | - | - | - | - | 145 | - | >600 | - | - | - | 69.1 | 210 | 211 | 7.67 | 33.5 |  | 1.26 | 4.53 | 4.51 | 4.37 | 1.09 | 467 | 1.09 | 1.83 | - | alea | 3.3e-16 / 4.7e-16 |  |  |  |  |  | 1.1e-16 / 1.6e-16 |  |  |  |  |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.6e-16 | 5.6e-16 / 7.8e-16 |  | 2.5e-08 / 3.5e-08 | 3.6e-13 / 5.1e-13 | 3.6e-13 / 5.1e-13 | 3.6e-13 / 5.1e-13 | 1.1e-16 / 1.6e-16 | 1.1e-16 / 1.6e-16 | 1.1e-16 / 1.6e-16 |
| wmi_path_n4096 | 32768 | 364468 | 48.8 | - | - | - | - | - | err | - | - | - | - | - | - | - | - | oom | oom |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| wmi_path_n512 | 4096 | 45492 | 4.46 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 62.9 | >600 |  | oom | >600 | 447 | >600 | - | - | oom | oom | - | alea | 1.6e-14 / 2.3e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.5e-14 / 2.1e-14 |  |  |  |  | 3.8e-13 / 5.3e-13 |  |  |  |  |
| wmi_path_n64 | 512 | 5620 | 0.57 | - | - | - | - | - | err | - | - | - | - | - | 230 | >600 | >600 | 11.7 | 111 |  | 3.4 | 43 | 9.76 | 15.6 | 2.92 | >600 | 4.25 | 5.1 | - | alea | 5.6e-16 / 7.8e-16 |  |  |  |  |  |  |  |  |  |  |  | 8.9e-16 / 1.3e-15 |  |  | 1.0e-15 / 1.4e-15 | 8.9e-16 / 1.3e-15 |  | 2.5e-08 / 3.5e-08 | 3.6e-13 / 5.1e-13 | 3.6e-13 / 5.1e-13 | 3.6e-13 / 5.1e-13 | 1.0e-15 / 1.4e-15 |  | 8.9e-16 / 1.3e-15 |
| wmi_star_n1024 | 8200 | 91149 | 10.7 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | err | >600 | 91.3 | - | - | - | - | - | - | - | - | - | ace-Xlog | 1.1e-16 / 2.2e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| wmi_star_n128 | 1032 | 11405 | 1.19 | - | - | - | - | - | - | - | 230 | - | - | - | >600 | - | - | 16.6 | wrong |  | 36 | 50.3 | 42 | >600 | - | - | 15.3 | 16.5 | 78.4 | alea | 1.1e-16 / 2.2e-16 |  |  |  |  |  |  |  | 3.3e-16 / 6.6e-16 |  |  |  |  |  |  | 3.3e-16 / 6.6e-16 | inf / inf |  | 2.1e-08 / 4.1e-08 | 4.2e-13 / 8.3e-13 | 4.2e-13 / 8.3e-13 |  |  |  | 3.3e-16 / 6.6e-16 |
| wmi_star_n16 | 136 | 1437 | 0.157 | >600 | oom | >600 | oom | >600 | 26.3 | >600 | 5.08 | oom | >600 | >600 | 19 | 69 | 68.6 | 4.39 | 6.27 |  | 0.607 | 1.14 | 3.42 | 2.52 | 5.14 | 220 | 0.334 | 1.07 | wrong | alea | 1.1e-16 / 2.1e-16 |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  | 1.1e-16 / 2.1e-16 |  |  |  | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.1e-16 | 1.1e-16 / 2.1e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.1e-16 |  | 5.7e-08 / 1.1e-07 | 4.1e-14 / 7.6e-14 | 4.1e-14 / 7.6e-14 | 4.1e-14 / 7.6e-14 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| wmi_star_n256 | 2056 | 22797 | 2.36 | - | - | - | - | - | - | - | >600 | - | - | - | - | - | - | err | wrong | 15.7 | 216 | 230 | 152 | - | - | - | 64.6 | 66.4 | 162 | alea | 2.2e-16 / 4.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | inf / inf | 8.9e-16 / 1.8e-15 | 1.8e-07 / 3.6e-07 | 3.8e-13 / 7.5e-13 | 3.8e-13 / 7.5e-13 |  |  |  | 1.1e-16 / 2.2e-16 |
| wmi_star_n32 | 264 | 2861 | 0.298 | - | - | - | - | - | 76.7 | - | 18.1 | - | - | - | 45.9 | 501 | 513 | 5.85 | 8.93 |  | 1.8 | 3.17 | 5.23 | 12.8 | oom | >600 | 1.06 | 1.8 | wrong | alea | 3.3e-16 / 6.4e-16 |  |  |  |  |  | 1.1e-16 / 2.1e-16 |  | 2.2e-16 / 4.3e-16 |  |  |  | 0.0e+00 / 0.0e+00 | 3.3e-14 / 6.3e-14 | 3.3e-14 / 6.3e-14 | 1.1e-16 / 2.1e-16 | 0.0e+00 / 0.0e+00 |  | 3.2e-07 / 6.1e-07 | 3.2e-13 / 6.1e-13 | 3.2e-13 / 6.1e-13 | 3.2e-13 / 6.1e-13 |  |  | 1.1e-16 / 2.1e-16 |
| wmi_star_n4096 | 32776 | 364557 | 55.1 | - | - | - | - | - | err | - | - | - | - | - | - | - | - | >600 | - | >600 | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| wmi_star_n512 | 4104 | 45581 | 5 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | err | wrong | 33.6 | >600 | >600 | >600 | - | - | - | oom | oom | oom | alea | 2.2e-16 / 4.4e-16 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | inf / inf | 4.4e-16 / 8.8e-16 |  |  |  |  |  |  |  |
| wmi_star_n64 | 520 | 5709 | 0.606 | - | - | - | - | - | err | - | 152 | - | - | - | 179 | >600 | >600 | 9.69 | 17 |  | 7.35 | 12.3 | 13.7 | 94.3 | - | - | 3.84 | 4.67 | 37.8 | alea | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  | 2.2e-16 / 4.4e-16 |  |  |  | 0.0e+00 / 0.0e+00 |  |  | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 |  | 4.6e-07 / 9.1e-07 | 9.1e-14 / 1.8e-13 | 9.1e-14 / 1.8e-13 | 9.1e-14 / 1.8e-13 |  |  | 1.1e-16 / 2.2e-16 |
| zeroone_w1_w10 | 32 | 2511 | 0.338 | >600 | 22.6 | >600 | oom | >600 | 50.2 | 1.71 | 14.4 | >600 | 29 | 42.7 | 66.1 | 69.2 | 148 | 6.8 | 4.65 |  | oom | oom | err | oom | err | 101 | 0.547 | 5.06 | 3.2 | alea | 7.8e-16 / 1.5e-15 |  | 4.0e-07 / 7.9e-07 |  |  |  | 7.8e-16 / 1.5e-15 | 6.2e-13 / 1.5e-12 | 7.8e-16 / 1.5e-15 |  | 8.9e-16 / 1.8e-15 | 7.8e-16 / 1.5e-15 | 8.9e-16 / 1.8e-15 | 7.8e-16 / 1.5e-15 | 7.8e-16 / 1.5e-15 | 1.2e-14 / 2.4e-14 | 5.6e-16 / 1.1e-15 |  |  |  |  |  |  | 7.8e-16 / 1.5e-15 | 7.8e-16 / 1.5e-15 |
| zeroone_w1_w12 | 36 | 4184 | 0.602 | - | 348 | - | - | - | 76.5 | 3.16 | 39 | - | >600 | >600 | 300 | >600 | >600 | 28.6 | 37.3 |  | - | - | - | - | - | >600 | 1.28 | 7.42 | 195 | alea | 7.8e-16 / 1.5e-15 |  | 7.8e-07 / 1.5e-06 |  |  |  | 7.8e-16 / 1.5e-15 | 2.9e-13 / 5.8e-13 | 4.4e-16 / 8.8e-16 |  |  |  | 6.7e-16 / 1.3e-15 |  |  | 3.8e-15 / 7.5e-15 | 1.4e-14 / 2.9e-14 |  |  |  |  |  |  |  | 7.8e-16 / 1.5e-15 |
| zeroone_w1_w14 | 40 | 6254 | 0.826 | - | oom | - | - | - | 127 | 6.13 | 89.4 | - | - | - | >600 | - | - | >600 | >600 |  | - | - | - | - | - | - | 3.96 | 12 | >600 | alea | 1.9e-15 / 3.8e-15 |  |  |  |  |  | 1.0e-15 / 2.0e-15 | 2.3e-13 / 4.6e-13 | 1.1e-15 / 2.2e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.0e-15 / 2.0e-15 |
| zeroone_w1_w16 | 44 | 8780 | 1.3 | - | - | - | - | - | 313 | 15.3 | 284 | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 16.4 | 27.5 | - | alea | 3.9e-15 / 7.7e-15 |  |  |  |  |  | 8.9e-16 / 1.8e-15 | 3.2e-13 / 6.5e-13 | 1.0e-15 / 2.0e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 8.9e-16 / 1.8e-15 |
| zeroone_w1_w18 | 48 | 11762 | 2.52 | - | - | - | - | - | >600 | 41.2 | >600 | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 79.7 | 93.3 | - | alea | 1.6e-15 / 3.1e-15 |  |  |  |  |  |  | 6.4e-13 / 1.3e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.6e-15 / 3.1e-15 |
| zeroone_w1_w20 | 52 | 15200 | 3.16 | - | - | - | - | - | - | 104 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 368 | 387 | - | alea | 4.2e-15 / 8.4e-15 |  |  |  |  |  |  | 4.3e-13 / 8.5e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 7.8e-16 / 1.6e-15 |
| zeroone_w1_w22 | 56 | 19094 | 4.39 | - | - | - | - | - | - | 215 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | >600 | >600 | - | alea | 6.2e-15 / 1.2e-14 |  |  |  |  |  |  | 3.6e-13 / 7.1e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| zeroone_w1_w26 | 64 | 28250 | 6.52 | - | - | - | - | - | - | >600 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| zeroone_w1_w30 | 72 | 39230 | 12.7 | - | - | - | - | - | - | >600 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| zeroone_w1_w34 | 928 | 54788 | oom | - | - | - | - | - | - | >600 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| zeroone_w1_w38 | 1136 | 70016 | - | - | - | - | - | - | - | >600 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| zeroone_w2_w13 | 38 | 5230 | 0.66 | >600 | oom | >600 | oom | >600 | 94.2 | 4.31 | 79.6 | >600 | >600 | >600 | >600 | >600 | >600 | 284 | 258 |  | err | oom | >600 | oom | oom | >600 | 2.29 | 9.19 | >600 | alea | 2.0e-15 / 3.8e-15 |  |  |  |  |  | 8.9e-16 / 1.2e-15 | 5.0e-13 / 2.3e-12 | 7.8e-16 / 1.1e-15 |  |  |  |  |  |  | 3.2e-14 / 6.1e-14 | 9.1e-15 / 1.5e-14 |  |  |  |  |  |  |  | 8.9e-16 / 1.2e-15 |
| zeroone_w2_w15 | 42 | 7560 | 1.05 | - | - | - | - | - | 191 | 9.87 | 570 | - | - | - | - | - | - | >600 | >600 |  | - | - | - | - | - | - | 10 | 23.1 | - | alea | 1.0e-14 / 1.4e-14 |  |  |  |  |  | 1.3e-15 / 2.7e-15 | 4.7e-13 / 1.5e-12 | 1.3e-15 / 2.7e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.3e-15 / 2.7e-15 |
| zeroone_w2_w17 | 46 | 10346 | 1.91 | - | - | - | - | - | >600 | 27.3 | >600 | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 39.4 | 50.4 | - | alea | 3.2e-14 / 5.8e-14 |  |  |  |  |  |  | 4.1e-13 / 2.2e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.4e-15 / 2.9e-15 |
| zeroone_w2_w19 | 50 | 13588 | 3.6 | - | - | - | - | - | - | 98.6 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | 189 | 204 | - | alea | 1.1e-13 / 1.6e-13 |  |  |  |  |  |  | 5.8e-13 / 1.7e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 6.7e-16 / 1.3e-15 |
| zeroone_w2_w21 | 54 | 17286 | 4.43 | - | - | - | - | - | - | 182 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | >600 | >600 | - | alea | 4.0e-15 / 7.9e-15 |  |  |  |  |  |  | 2.4e-13 / 9.5e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| zeroone_w2_w23 | 58 | 21440 | 4.86 | - | - | - | - | - | - | 477 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  | 3.1e-13 / 6.3e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| zeroone_w2_w25 | 62 | 26050 | 11 | - | - | - | - | - | - | >600 | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | tddlog | ref |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |

Seconds per system, the median over the repetitions; a failure is `>600`
(timeout), `oom`, `err` or `n/a`; each system's last columns are its
largest absolute and relative difference from the reference named in
`ref`.

### On Roulette's programs

| family | instances | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Scallop | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Ace log space | Merlin | pyAgrum | pgmpy | libDAI | TensorInference.jl greedy | TensorInference.jl treesa | Roulette |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| bn | 9 | 9 | 8 of 9 | 8 of 9 | 4 of 9 | 9 of 9 | 8 of 9 | 9 of 9 | 7 of 9 | 7 of 9 | 2 of 9 | 9 of 9 | 9 of 9 | 9 of 9 | 9 of 9 | 9 of 9 | 9 of 9 | 9 of 9 |  | 9 of 9 | 9 of 9 | 8 of 9 | 9 of 9 | 8 of 9 | 9 of 9 | 9 of 9 |
| rl_caesar | 13 | 11 | 0 of 9 | 2 of 4 | 1 of 3 | 4 of 13 | 0 of 2 | 0 of 2 | 0 of 3 | 3 of 5 | 0 of 1 | 4 of 10 | 7 of 8 | 4 of 9 | 4 of 10 | 4 of 10 | 2 of 11 | 2 of 9 | 6 of 7 | 3 of 4 | 4 of 5 | 2 of 3 | 3 of 4 | 1 of 3 | 0 of 2 | 4 of 12 |
| rl_diamond | 9 | 7 | 0 of 1 | 0 of 1 | 0 of 6 | 0 of 1 | 0 of 1 | 0 of 1 | 0 of 1 | 3 of 4 | 0 of 1 | 4 of 5 | 4 of 5 | 1 of 2 | 2 of 3 | 2 of 3 | 6 of 9 | 1 of 8 |  | 2 of 3 | 2 of 3 | 3 of 4 | 3 of 4 | 1 of 2 | 0 of 1 | 6 of 7 |
| rl_hardware | 11 | 9 | 6 of 7 | 1 of 3 | 0 of 2 | 1 of 7 | 0 of 6 | 3 of 5 | 2 of 4 | 4 of 6 | 0 of 1 | 6 of 7 | 5 of 6 | 5 of 6 | 6 of 7 | 6 of 7 | 3 of 11 | 3 of 9 | 3 of 8 | 4 of 5 | 3 of 4 | 5 of 6 | 5 of 6 | 3 of 5 | 3 of 4 | 4 of 5 |
| rl_hardware_bitwidth | 16 | 16 | 0 of 16 | 2 of 3 | 0 of 1 | 16 of 16 | 16 of 16 | 16 of 16 | 16 of 16 | 16 of 16 | 0 of 1 | 3 of 16 | 3 of 16 | 2 of 3 | 2 of 3 | 2 of 3 | 16 of 16 | 16 of 16 |  | 16 of 16 | 16 of 16 | 16 of 16 | 16 of 16 | 16 of 16 | 16 of 16 | 14 of 15 |
| rl_hmm_filtering_joint | 10 | 8 | 0 of 1 | 0 of 4 | 0 of 10 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 2 of 4 | 0 of 1 | 0 of 1 | 0 of 1 | 2 of 3 | 0 of 7 | 0 of 7 | 2 of 8 | 2 of 8 | 5 of 6 | 0 of 4 | 2 of 5 | 2 of 6 | 4 of 5 | 2 of 3 | 2 of 3 | 3 of 5 |
| rl_hmm_filtering_marginal | 10 | 8 | 0 of 1 | 2 of 4 | 0 of 10 | 0 of 1 | 2 of 3 | 0 of 1 | 0 of 2 | 2 of 4 | 0 of 1 | 2 of 9 | 5 of 6 | 2 of 7 | 2 of 7 | 2 of 7 | 2 of 8 | 2 of 8 | 5 of 6 | 2 of 4 | 2 of 6 | 2 of 6 | 4 of 5 | 2 of 3 | 2 of 3 | 2 of 5 |
| rl_hmm_marginal_likelihood | 4 | 4 | 0 of 1 | 0 of 4 | 0 of 4 | 0 of 1 | 0 of 1 | 0 of 1 | 3 of 4 | 4 of 4 | 0 of 1 | 0 of 1 | 0 of 1 | 3 of 4 | 0 of 4 | 0 of 4 | 4 of 4 | 4 of 4 |  | 0 of 4 | 4 of 4 | 4 of 4 | 4 of 4 | 3 of 4 | 2 of 3 | 4 of 4 |
| rl_hmm_smoothing_joint | 10 | 8 | 0 of 1 | 0 of 4 | 0 of 10 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 2 of 4 | 0 of 1 | 0 of 1 | 0 of 1 | 2 of 3 | 0 of 7 | 0 of 7 | 2 of 9 | 2 of 9 | 5 of 7 | 0 of 4 | 2 of 7 | 2 of 8 | 4 of 5 | 2 of 3 | 2 of 3 | 3 of 5 |
| rl_hmm_smoothing_marginal | 10 | 8 | 0 of 1 | 2 of 4 | 0 of 10 | 0 of 1 | 2 of 3 | 0 of 1 | 2 of 3 | 2 of 4 | 0 of 1 | 2 of 9 | 5 of 6 | 2 of 7 | 2 of 7 | 2 of 7 | 2 of 8 | 2 of 8 | 5 of 6 | 2 of 4 | 2 of 6 | 2 of 6 | 4 of 5 | 2 of 3 | 2 of 3 | 2 of 5 |
| rl_ladder | 8 | 7 | 0 of 1 | 0 of 1 | 0 of 5 | 0 of 8 | 0 of 1 | 0 of 1 | 0 of 1 | 1 of 2 | 0 of 1 | 3 of 4 | 3 of 4 | 1 of 2 | 2 of 3 | 2 of 3 | 5 of 6 | 1 of 7 |  | 1 of 2 | 2 of 3 | 2 of 3 | 2 of 3 | 0 of 1 | 0 of 1 | 6 of 7 |
| rl_motivating | 9 | 7 | 0 of 1 | 1 of 2 | 0 of 1 | 0 of 1 | 1 of 2 | 2 of 5 | 0 of 1 | 2 of 3 | 0 of 1 | 4 of 5 | 4 of 5 | 1 of 2 | 2 of 3 | 2 of 3 | 6 of 9 | 3 of 9 |  | 2 of 3 | 2 of 3 | 3 of 4 | 3 of 4 | 1 of 2 | 0 of 1 | 7 of 8 |
| rl_network | 11 | 9 | 2 of 7 | 0 of 1 | 0 of 11 | 4 of 11 | 0 of 1 | 3 of 4 | 2 of 3 | 4 of 5 | 0 of 1 | 6 of 7 | 5 of 6 | 5 of 6 | 5 of 6 | 5 of 6 | 5 of 6 | 4 of 5 |  | 4 of 5 | 5 of 6 | 1 of 2 | 5 of 6 | 3 of 4 | 3 of 4 | 4 of 5 |

**Geni.** Geni ran 56 instances and answered 16 (8 fail, 2 run out of
memory, 9 time out, 21 answer wrongly). Its answers differ from the
reference's by at most 1.0e+00 relative (1.0e-07 absolute, hardware_10)
over 16 instances. Geni's answer is wrong, zero or underflowed on
bn_munin, hardware_bitwidth_10, hardware_bitwidth_100,
hardware_bitwidth_1000, hardware_bitwidth_10000 and 16 more. Over the 54
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Geni takes 23057 s in all and tddlog 33.1 s;
over the 16 both answer, 257 s against tddlog's 4.82. tddlog alone
answers bn_munin, caesar_100, caesar_1000, caesar_200, caesar_25 and 33
more.

**noDice.** noDice ran 44 instances and answered 18 (8 run out of
memory, 6 time out, 1 underflow, 4 answer wrongly, 7 print zeros). Its
answers differ from the reference's by at most 1.0e+00 relative (6.6e-07
absolute, caesar_25) over 18 instances. noDice's answer is wrong, zero
or underflowed on hmm_filtering_joint_100, hmm_filtering_joint_1000,
hmm_filtering_joint_300, hmm_filtering_marginal_1000,
hmm_marginal_likelihood_100 and 7 more. Over the 44 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, noDice takes 15853 s in all and tddlog 21.7 s; over the 18 both
answer, 253 s against tddlog's 0.89. tddlog alone answers bn_hailfinder,
caesar_100, caesar_300, diamond_1000, hardware_10000 and 21 more.

**Psi.** Psi ran 82 instances and answered 5 (60 fail, 7 run out of
memory, 10 time out). Its answers differ from the reference's by at most
1.0e-13 relative (4.8e-15 absolute, caesar_25) over 5 instances. Over
the 69 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Psi takes 38693 s in all and tddlog
474 s; over the 5 both answer, 293 s against tddlog's 0.19. tddlog alone
answers bn_hailfinder, bn_hepar2, bn_insurance, bn_munin, bn_water and
59 more.

**Genfer.** Genfer ran 71 instances and answered 34 (27 fail, 4 run out
of memory, 6 time out). Its answers differ from the reference's by at
most 1.0e+00 relative (5.3e-26 absolute, caesar_100) over 34 instances.
Over the 64 instances given to both that either answers, a run that does
not answer counted at the 600 s limit, Genfer takes 18322 s in all and
tddlog 732 s; over the 34 both answer, 322 s against tddlog's 2.52.
tddlog alone answers caesar_1000, caesar_10000, caesar_2000,
caesar_20000, caesar_300 and 25 more.

**SPPL.** SPPL ran 47 instances and answered 29 (3 fail, 2 run out of
memory, 13 time out). Its answers differ from the reference's by at most
1.9e-14 relative (1.7e-16 absolute, bn_munin) over 29 instances. Over
the 45 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, SPPL takes 11038 s in all and tddlog
242 s; over the 29 both answer, 1438 s against tddlog's 0.74. tddlog
alone answers bn_insurance, caesar_25, caesar_300, diamond_1000,
hardware_10 and 11 more.

**Pluck.** Pluck ran 48 instances and answered 33 (14 fail, 1 run out of
memory). Its answers differ from the reference's by at most 2.7e-11
relative (1.5e-11 absolute, bn_insurance) over 33 instances. Over the 46
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Pluck takes 8171 s in all and tddlog 14.3 s;
over the 33 both answer, 371 s against tddlog's 0.93. tddlog alone
answers caesar_25, caesar_300, diamond_1000, hardware_10000,
hardware_300 and 8 more.

**PITA.** PITA ran 51 instances and answered 34 (1 fail, 15 time out, 1
answer wrongly). Its answers differ from the reference's by at most
1.0e+00 relative (2.2e-16 absolute, hardware_30) over 34 instances.
PITA's answer is wrong, zero or underflowed on
hmm_filtering_marginal_100. Over the 51 instances given to both that
either answers, a run that does not answer counted at the 600 s limit,
PITA takes 11483 s in all and tddlog 14.1 s; over the 34 both answer,
1283 s against tddlog's 0.74. tddlog alone answers bn_hailfinder,
bn_insurance, caesar_25, caesar_300, caesar_50 and 12 more.

**aspmc.** aspmc ran 70 instances and answered 52 (14 time out, 4
underflow). Its answers differ from the reference's by at most 1.0e+00
relative (2.2e-16 absolute, caesar_100) over 52 instances. aspmc's
answer is wrong, zero or underflowed on hmm_filtering_joint_1000,
hmm_filtering_marginal_1000, hmm_smoothing_joint_1000 and
hmm_smoothing_marginal_1000. Over the 70 instances given to both that
either answers, a run that does not answer counted at the 600 s limit,
aspmc takes 13691 s in all and tddlog 27.8 s; over the 52 both answer,
2891 s against tddlog's 3.04. tddlog alone answers bn_munin, bn_pigs,
caesar_200, caesar_300, diamond_30000 and 13 more.

**Scallop.** Scallop ran 21 instances and answered 2 (13 run out of
memory, 6 time out). Its answers differ from the reference's by at most
2.7e-16 relative (1.1e-16 absolute, bn_cancer) over 2 instances. Over
the 21 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Scallop takes 11402 s in all and
tddlog 0.76 s; over the 2 both answer, 2.21 s against tddlog's 0.051.
tddlog alone answers bn_alarm, bn_hailfinder, bn_hepar2, bn_insurance,
bn_munin and 14 more.

**Storm sparse.** Storm sparse ran 84 instances and answered 43 (28
fail, 1 run out of memory, 12 time out). Its answers differ from the
reference's by at most 1.0e+00 relative (1.1e-16 absolute, caesar_100)
over 43 instances. Over the 78 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, Storm
sparse takes 22807 s in all and tddlog 188 s; over the 43 both answer,
1807 s against tddlog's 14.9. tddlog alone answers caesar_1000,
caesar_2000, caesar_300, caesar_500, diamond_100000 and 30 more.

**Storm exact.** Storm exact ran 74 instances and answered 50 (13 fail,
11 time out). Its answers differ from the reference's by at most 3.2e-11
relative (1.3e-78 absolute, caesar_1000) over 50 instances. Over the 74
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Storm exact takes 16507 s in all and tddlog
65.5 s; over the 50 both answer, 2107 s against tddlog's 27.7. tddlog
alone answers caesar_2000, diamond_100000, hardware_3000,
hardware_bitwidth_100, hardware_bitwidth_1000 and 19 more.

**Storm dd.** Storm dd ran 63 instances and answered 39 (10 fail, 2 run
out of memory, 12 time out). Its answers differ from the reference's by
at most 1.0e+00 relative (1.1e-16 absolute, caesar_100) over 39
instances. Over the 57 instances given to both that either answers, a
run that does not answer counted at the 600 s limit, Storm dd takes
13103 s in all and tddlog 126 s; over the 39 both answer, 2303 s against
tddlog's 4.86. tddlog alone answers caesar_1000, caesar_300, caesar_500,
diamond_3000, hardware_3000 and 13 more.

**PRISM sparse.** PRISM sparse ran 76 instances and answered 36 (24
fail, 4 run out of memory, 12 time out). Its answers differ from the
reference's by at most 1.0e+00 relative (1.1e-16 absolute, caesar_100)
over 36 instances. Over the 66 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, PRISM
sparse takes 19672 s in all and tddlog 258 s; over the 36 both answer,
1672 s against tddlog's 8.05. tddlog alone answers caesar_1000,
caesar_2000, caesar_300, caesar_500, diamond_10000 and 25 more.

**PRISM hybrid.** PRISM hybrid ran 76 instances and answered 36 (24
fail, 4 run out of memory, 12 time out). Its answers differ from the
reference's by at most 1.0e+00 relative (1.1e-16 absolute, caesar_100)
over 36 instances. Over the 66 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, PRISM
hybrid takes 19708 s in all and tddlog 258 s; over the 36 both answer,
1708 s against tddlog's 8.05. tddlog alone answers caesar_1000,
caesar_2000, caesar_300, caesar_500, diamond_10000 and 25 more.

**Ace.** Ace ran 114 instances and answered 64 (25 fail, 18 run out of
memory, 7 time out). Its answers differ from the reference's by at most
2.7e-11 relative (1.5e-11 absolute, bn_insurance) over 64 instances.
Over the 105 instances given to both that either answers, a run that
does not answer counted at the 600 s limit, Ace takes 26579 s in all and
tddlog 805 s; over the 64 both answer, 1979 s against tddlog's 68.
tddlog alone answers caesar_100, caesar_1000, caesar_200, caesar_2000,
caesar_300 and 36 more.

**Ace c2d.** Ace c2d ran 109 instances and answered 51 (30 fail, 17 run
out of memory, 9 time out, 2 answer wrongly). Its answers differ from
the reference's by at most 9.8e-09 relative (9.8e-13 absolute,
hardware_100) over 51 instances. Ace c2d's answer is wrong, zero or
underflowed on diamond_3000 and ladder_3000. Over the 99 instances given
to both that either answers, a run that does not answer counted at the
600 s limit, Ace c2d takes 29235 s in all and tddlog 699 s; over the 51
both answer, 435 s against tddlog's 2.47. tddlog alone answers
caesar_100, caesar_1000, caesar_200, caesar_300, caesar_500 and 43 more.

**Ace log space.** Ace log space ran 40 instances and answered 29 (5
fail, 1 run out of memory, 5 time out). Its answers differ from the
reference's by at most 8.6e-08 relative (5.4e-08 absolute,
hmm_filtering_marginal_100000) over 29 instances. Over the 37 instances
given to both that either answers, a run that does not answer counted at
the 600 s limit, Ace log space takes 6827 s in all and tddlog 579 s;
over the 29 both answer, 2027 s against tddlog's 107. tddlog alone
answers caesar_5000, hardware_10000, hardware_100000, hardware_30000,
hmm_filtering_joint_300000 and 3 more.

**Merlin.** Merlin ran 67 instances and answered 45 (2 fail, 8 run out
of memory, 5 underflow, 7 print zeros). Its answers differ from the
reference's by at most 1.0e+00 relative (5.7e-13 absolute, caesar_100)
over 45 instances. Merlin's answer is wrong, zero or underflowed on
hmm_filtering_joint_100, hmm_filtering_joint_1000,
hmm_filtering_joint_300, hmm_filtering_marginal_1000,
hmm_marginal_likelihood_100 and 7 more. Over the 67 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, Merlin takes 13296 s in all and tddlog 14.1 s; over the 45 both
answer, 96 s against tddlog's 2.34. tddlog alone answers caesar_200,
diamond_10000, hardware_1000, hmm_filtering_joint_100,
hmm_filtering_joint_1000 and 17 more.

**pyAgrum.** pyAgrum ran 77 instances and answered 55 (8 fail, 2 run out
of memory, 12 time out). Its answers differ from the reference's by at
most 1.0e+00 relative (4.3e-13 absolute, caesar_100) over 55 instances.
Over the 75 instances given to both that either answers, a run that does
not answer counted at the 600 s limit, pyAgrum takes 13366 s in all and
tddlog 237 s; over the 55 both answer, 1366 s against tddlog's 4.36.
tddlog alone answers caesar_300, diamond_10000, hardware_300,
hmm_filtering_joint_1000, hmm_filtering_joint_3000 and 15 more.

**pgmpy.** pgmpy ran 77 instances and answered 52 (10 fail, 1 run out of
memory, 14 time out). Its answers differ from the reference's by at most
2.4e-11 relative (1.3e-11 absolute, bn_insurance) over 52 instances.
Over the 75 instances given to both that either answers, a run that does
not answer counted at the 600 s limit, pgmpy takes 15678 s in all and
tddlog 241 s; over the 52 both answer, 1878 s against tddlog's 4.17.
tddlog alone answers bn_munin, caesar_100, diamond_30000, hardware_3000,
hmm_filtering_joint_1000 and 18 more.

**libDAI.** libDAI ran 76 instances and answered 66 (10 time out). Its
answers differ from the reference's by at most 1.0e+00 relative (4.3e-13
absolute, caesar_100) over 66 instances. Over the 76 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, libDAI takes 8062 s in all and tddlog 32.9 s; over the 66 both
answer, 2062 s against tddlog's 12.9. tddlog alone answers caesar_200,
diamond_30000, hardware_3000, hmm_filtering_joint_10000,
hmm_filtering_marginal_10000 and 5 more.

**TensorInference.jl greedy.** TensorInference.jl greedy ran 58
instances and answered 44 (1 run out of memory, 13 time out). Its
answers differ from the reference's by at most 2.4e-11 relative (1.3e-11
absolute, bn_insurance) over 44 instances. Over the 58 instances given
to both that either answers, a run that does not answer counted at the
600 s limit, TensorInference.jl greedy takes 8418 s in all and tddlog
4.11 s; over the 44 both answer, 18.3 s against tddlog's 1.24. tddlog
alone answers bn_munin, caesar_100, caesar_50, diamond_3000,
hardware_1000 and 9 more.

**TensorInference.jl treesa.** TensorInference.jl treesa ran 53
instances and answered 41 (12 time out). Its answers differ from the
reference's by at most 2.4e-11 relative (1.3e-11 absolute, bn_insurance)
over 41 instances. Over the 53 instances given to both that either
answers, a run that does not answer counted at the 600 s limit,
TensorInference.jl treesa takes 10720 s in all and tddlog 2.5 s; over
the 41 both answer, 3520 s against tddlog's 1.1. tddlog alone answers
caesar_25, caesar_50, diamond_1000, hardware_300,
hmm_filtering_joint_1000 and 7 more.

**Roulette.** Roulette ran 92 instances and answered 68 (6 run out of
memory, 5 time out, 13 underflow). Its answers differ from the
reference's by at most 1.0e+00 relative (1.1e-16 absolute, caesar_100)
over 68 instances. Roulette's answer is wrong, zero or underflowed on
caesar_1000, caesar_10000, caesar_2000, caesar_20000, caesar_300 and 8
more. Over the 90 instances given to both that either answers, a run
that does not answer counted at the 600 s limit, Roulette takes 14687 s
in all and tddlog 778 s; over the 68 both answer, 1487 s against
tddlog's 245. tddlog alone answers caesar_1000, caesar_10000,
caesar_2000, caesar_20000, caesar_300 and 17 more.

| instance | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Scallop | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Ace log space | Merlin | pyAgrum | pgmpy | libDAI | TI greedy | TI treesa | Roulette | Roulette process | ProbLog | ref | tddlog diff | Geni diff | noDice diff | Psi diff | Genfer diff | SPPL diff | Pluck diff | PITA diff | aspmc diff | Scallop diff | Storm sparse diff | Storm exact diff | Storm dd diff | PRISM sparse diff | PRISM hybrid diff | Ace diff | Ace c2d diff | Ace log space diff | Merlin diff | pyAgrum diff | pgmpy diff | libDAI diff | TI greedy diff | TI treesa diff | Roulette diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| bn_alarm | 0.026 | 0.027 | 0.061 | 84.3 | 0.021 | 1.63 | 7.67 | 0.648 | 163 | oom | 0.228 | 0.222 | 0.486 | 1.73 | 1.75 | 1.3 | 1.41 |  | 0.007 | 0.129 | 1.79 | 0.006 | 0.0052 | 2.42 | 0.117 | 0.867 | 0.501 | exact | 6.7e-16 / 2.5e-14 | 4.6e-08 / 1.7e-06 | 5.5e-07 / 2.0e-05 | 1.1e-16 / 5.3e-16 | 1.1e-16 / 2.6e-16 | 3.9e-16 / 1.5e-15 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 4.4e-16 | 5.6e-16 / 1.7e-15 |  | 1.1e-16 / 3.9e-16 | 1.1e-16 / 5.3e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 3.9e-16 | 1.1e-16 / 3.9e-16 | 1.1e-16 / 3.9e-16 | 5.6e-17 / 2.6e-16 |  | 4.5e-07 / 1.7e-05 | 1.1e-16 / 5.3e-16 | 1.1e-16 / 5.3e-16 | 1.1e-16 / 5.3e-16 | 8.3e-17 / 3.9e-16 | 8.3e-17 / 3.9e-16 | 1.1e-16 / 5.3e-16 |
| bn_cancer | 0.026 | 0.012 | 0.044 | 0.544 | 0.008 | 0.762 | 5.13 | 0.553 | 2.72 | 1.32 | 0.208 | 0.21 | 0.252 | 1.69 | 1.71 | 1.45 | 1.25 |  | 0.004 | 0.13 | 1.94 | 0.004 | 0.001 | 0.187 | 0.009 | 0.754 | 0.282 | exact | 1.1e-16 / 5.3e-16 | 2.8e-17 / 1.3e-16 | 2.8e-17 / 1.3e-16 | 2.8e-17 / 1.3e-16 | 1.1e-16 / 1.4e-16 | 2.8e-17 / 1.3e-16 | 5.6e-17 / 2.7e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.4e-16 | 1.1e-16 / 2.7e-16 | 2.2e-16 / 2.8e-16 | 2.8e-17 / 1.3e-16 | 1.1e-16 / 1.4e-16 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 1.4e-16 | 1.1e-16 / 1.4e-16 |  | 2.8e-17 / 1.3e-16 | 2.8e-17 / 1.3e-16 | 2.8e-17 / 1.3e-16 | 2.8e-17 / 1.3e-16 | 2.8e-17 / 1.3e-16 | 2.8e-17 / 1.3e-16 | 0.0e+00 / 0.0e+00 |
| bn_hailfinder | 0.063 | 19.4 | >600 | >600 | 132 | 408 | 58.5 | >600 | 268 | >600 | 3.55 | 3.96 | 15 | 4.02 | 4.06 | 1.62 | 2.35 |  | 0.009 | 0.164 | 1.87 | 0.009 | 0.0073 | 3.7 | 1.81 | 2.56 | >600 | exact | 5.0e-16 / 1.6e-15 | 1.6e-07 / 5.3e-07 |  |  | 2.2e-16 / 7.2e-16 | 1.7e-16 / 5.4e-16 | 1.1e-16 / 3.6e-16 |  | 2.1e-15 / 4.7e-15 |  | 4.4e-16 / 1.0e-15 | 2.2e-16 / 5.4e-16 | 1.7e-16 / 5.4e-16 | 2.8e-16 / 6.6e-16 | 2.8e-16 / 6.6e-16 | 2.2e-16 / 5.0e-16 | 2.2e-16 / 5.0e-16 |  | 4.8e-07 / 1.1e-06 | 4.9e-13 / 1.6e-12 | 4.9e-13 / 1.6e-12 | 4.9e-13 / 1.6e-12 | 1.7e-16 / 5.4e-16 | 1.7e-16 / 4.4e-16 | 2.2e-16 / 5.4e-16 |
| bn_hepar2 | 0.034 | 0.056 | 0.129 | >600 | 0.092 | 6.53 | 7.77 | 24.5 | 200 | >600 | 0.256 | 0.284 | 1.48 | 1.96 | 1.92 | 1.59 | 2.16 |  | 0.009 | 0.136 | 1.69 | 0.009 | 0.0106 | 5.99 | 0.405 | 1.15 | 3.35 | exact | 2.8e-16 / 6.5e-16 | 1.2e-07 / 2.8e-07 | 1.7e-08 / 4.1e-08 |  | 0.0e+00 / 0.0e+00 | 2.2e-16 / 3.9e-16 | 1.1e-16 / 1.9e-16 | 1.1e-16 / 1.9e-16 | 3.4e-15 / 6.0e-15 |  | 1.1e-15 / 1.9e-15 | 0.0e+00 / 0.0e+00 | 3.3e-16 / 5.8e-16 | 3.3e-16 / 5.8e-16 | 3.3e-16 / 5.8e-16 | 1.1e-16 / 1.9e-16 | 2.2e-16 / 3.9e-16 |  | 1.7e-08 / 4.1e-08 | 4.7e-13 / 1.1e-12 | 4.7e-13 / 1.1e-12 | 4.7e-13 / 1.1e-12 | 1.1e-16 / 1.9e-16 | 5.6e-17 / 1.3e-16 | 2.5e-10 / 5.8e-10 |
| bn_insurance | 0.043 | 14 | 7.19 | >600 | 30.8 | >600 | 20.3 | >600 | 205 | >600 | 3.12 | 3.66 | 10.9 | 2.92 | 3.33 | 1.64 | 1.97 |  | 0.01 | 0.134 | 1.73 | 0.008 | 0.005 | 3.07 | 1.18 | 1.95 | >600 | exact | 1.5e-11 / 2.7e-11 | 2.0e-08 / 1.2e-06 | 7.1e-07 / 2.9e-05 |  | 1.5e-11 / 2.7e-11 |  | 1.5e-11 / 2.7e-11 |  | 5.1e-08 / 1.3e-07 |  | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 | 1.5e-11 / 2.7e-11 |  | 4.8e-07 / 2.9e-05 | 1.3e-11 / 2.4e-11 | 1.3e-11 / 2.4e-11 | 1.3e-11 / 2.4e-11 | 1.3e-11 / 2.4e-11 | 1.3e-11 / 2.4e-11 | 2.4e-11 / 1.4e-09 |
| bn_munin | 0.104 | wrong | 4.92 | >600 | 1.35 | 3.38 | 9.13 | 500 | >600 | >600 | 0.511 | 0.583 | 4.52 | 2.54 | 2.77 | 8.23 | 90 |  | 2.71 | 0.319 | >600 | 3.53 | oom | 96.6 | 8.66 | 9.4 | 64.3 | exact | 4.5e-16 / 1.9e-13 | 3.0e-04 / 1.1e-02 | 8.2e-07 / 2.0e-03 |  | 5.6e-17 / 1.7e-14 | 1.7e-16 / 1.9e-14 | 1.1e-16 / 1.7e-14 | 2.5e-16 / 2.7e-13 |  |  | 1.7e-16 / 1.8e-14 | 5.6e-17 / 1.7e-14 | 1.1e-16 / 1.7e-14 | 5.6e-17 / 1.7e-14 | 5.6e-17 / 1.7e-14 | 1.1e-16 / 1.4e-14 | 5.6e-17 / 1.3e-14 |  | 4.9e-07 / 2.0e-03 | 4.3e-13 / 2.1e-12 |  | 4.3e-13 / 2.1e-12 |  | 5.6e-17 / 1.6e-14 | 9.6e-09 / 3.5e-06 |
| bn_pigs | 0.031 | 0.015 | 0.037 | 0.553 | 0.009 | 0.739 | 5.58 | 1.45 | >600 | 0.898 | 0.214 | 0.211 | 0.251 | 1.71 | 1.69 | 3.67 | 14.6 |  | 0.157 | 0.171 | 1.92 | 0.15 | 1.24 | 35.3 | 0.822 | 1.56 | 1.2 | exact | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 5.6e-17 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| bn_survey | 0.027 | 0.014 | 0.05 | 1.55 | 0.009 | 0.868 | 5.42 | 0.502 | 2.78 | oom | 0.223 | 0.216 | 0.311 | 1.65 | 1.66 | 1.1 | 1.11 |  | 0.005 | 0.128 | 1.74 | 0.004 | 0.0012 | 0.645 | 0.011 | 0.753 | 0.153 | exact | 4.7e-16 / 3.0e-15 | 3.2e-08 / 7.8e-08 | 7.5e-07 / 2.7e-06 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.0e-16 | 1.1e-16 / 3.5e-16 | 1.1e-16 / 4.0e-16 | 1.4e-16 / 8.8e-16 | 0.0e+00 / 0.0e+00 |  | 1.1e-16 / 3.5e-16 | 0.0e+00 / 0.0e+00 | 2.8e-17 / 1.8e-16 | 2.8e-17 / 1.8e-16 | 2.8e-17 / 1.8e-16 | 2.8e-17 / 1.8e-16 | 2.8e-17 / 1.8e-16 |  | 2.5e-07 / 1.4e-06 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 5.6e-17 / 2.0e-16 | 1.1e-16 / 2.0e-16 | 1.1e-16 / 2.0e-16 |
| bn_water | 0.048 | 6.4 | 0.977 | >600 | 5.83 | 81.6 | 8.84 | 39.2 | 177 | >600 | 2.2 | 2.33 | 5.04 | 3.16 | 3.17 | 2 | 2.73 |  | 0.399 | 0.137 | 1.75 | 0.329 | 0.0452 | 5.61 | 0.559 | 1.3 | 18.9 | exact | 2.8e-16 / 2.7e-14 | 5.9e-08 / 2.6e-06 | 7.4e-07 / 7.5e-05 |  | 1.1e-16 / 1.4e-16 | 2.2e-16 / 5.6e-16 | 1.1e-16 / 3.7e-16 | 3.1e-16 / 1.3e-14 | 1.1e-16 / 4.5e-16 |  | 1.1e-16 / 3.9e-16 | 1.1e-16 / 3.4e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.9e-16 | 2.2e-16 / 4.9e-16 | 2.8e-17 / 1.9e-16 | 3.3e-16 / 4.1e-16 |  | 4.5e-07 / 5.4e-05 | 4.6e-13 / 1.4e-12 | 4.6e-13 / 1.4e-12 | 4.6e-13 / 1.4e-12 | 1.1e-16 / 3.4e-16 | 1.1e-16 / 2.2e-16 | 7.2e-09 / 9.6e-08 |
| caesar_25 | 0.076 | err | 39.4 | 206 | 9.1 | >600 | err | err | 16.2 | oom | 0.418 | 0.754 | 6.3 | 2.16 | 2.18 | 4.28 | 5 |  | 1.73 | 3.29 | 60.4 | 7.12 | 1.44 | >600 | 0.291 | 0.999 | 28.8 | exact | 5.1e-15 / 1.0e-13 |  | 6.6e-07 / 1.0e+00 | 4.8e-15 / 1.0e-13 | 5.0e-15 / 1.1e-13 |  |  |  | 5.0e-15 / 1.0e-13 |  | 4.8e-15 / 1.1e-13 | 4.7e-15 / 1.0e-13 | 4.6e-15 / 1.1e-13 | 4.7e-15 / 1.1e-13 | 4.7e-15 / 1.1e-13 | 5.0e-15 / 1.0e-13 | 5.0e-15 / 1.1e-13 |  | 3.4e-07 / 1.0e+00 | 4.6e-13 / 4.4e-12 | 4.6e-13 / 4.4e-12 | 4.6e-13 / 4.4e-12 | 4.3e-15 / 1.1e-13 |  | 4.6e-15 / 1.1e-13 |
| caesar_50 | 0.139 | err | 167 | >600 | 18.4 | - | - | >600 | 292 | - | 0.793 | 1.51 | 9.41 | 3.68 | 3.64 | 7.28 | 10.5 |  | 5.23 | 13.8 | 155 | 35.4 | >600 | >600 | 0.56 | 1.27 | 61.4 | exact | 2.3e-15 / 2.2e-13 |  | 4.1e-08 / 1.0e+00 |  | 2.3e-15 / 2.2e-13 |  |  |  | 2.6e-15 / 2.2e-13 |  | 2.2e-15 / 2.2e-13 | 2.2e-15 / 2.2e-13 | 2.2e-15 / 2.2e-13 | 2.2e-15 / 2.1e-13 | 2.2e-15 / 2.1e-13 | 2.2e-15 / 2.2e-13 | 2.3e-15 / 2.2e-13 |  | 4.1e-08 / 1.0e+00 | 4.8e-13 / 3.6e-12 | 4.8e-13 / 3.6e-12 | 4.8e-13 / 3.6e-12 |  |  | 2.3e-15 / 2.2e-13 |
| caesar_100 | 0.294 | err | >600 | - | 31.2 | - | - | - | 388 | - | 1.93 | 3.57 | 17.7 | 7.81 | 7.53 | err | err | 9.81 | 28.6 | 67.6 | >600 | 258 | >600 | - | 1.19 | 1.91 | 113 | exact | 4.4e-16 / 4.8e-13 |  |  |  | 5.3e-26 / 1.0e+00 |  |  |  | 2.2e-16 / 1.0e+00 |  | 1.1e-16 / 1.0e+00 | 5.6e-26 / 4.7e-13 | 1.1e-16 / 1.0e+00 | 1.1e-16 / 1.0e+00 | 1.1e-16 / 1.0e+00 |  |  | 4.4e-15 / 6.8e-13 | 5.7e-13 / 1.0e+00 | 4.3e-13 / 1.0e+00 |  | 4.3e-13 / 1.0e+00 |  |  | 1.1e-16 / 1.0e+00 |
| caesar_200 | 0.571 | err | - | - | 29.9 | - | - | - | >600 | - | 6.35 | 10.5 | 42 | 18.9 | 18.4 | err | err | 16.5 | oom | 368 | - | >600 | - | - | 2.44 | 3.15 | 253 | exact | 1.4e-29 / 1.5e-12 |  |  |  | 9.6e-18 / 1.0e+00 |  |  |  |  |  | 9.6e-18 / 1.0e+00 | 1.4e-29 / 1.5e-12 | 9.6e-18 / 1.0e+00 | 9.6e-18 / 1.0e+00 | 9.6e-18 / 1.0e+00 |  |  | 1.1e-30 / 2.7e-12 |  | 9.6e-18 / 1.0e+00 |  |  |  |  | 9.6e-18 / 1.0e+00 |
| caesar_300 | 0.89 | err | oom | oom | err | >600 | err | >600 | >600 | - | err | 21.1 | err | err | err | err | err | 22.3 | - | >600 | - | - | - | - | underflow | underflow | 207 | exact | 3.6e-31 / 4.0e-12 |  |  |  |  |  |  |  |  |  |  | 3.6e-31 / 4.0e-12 |  |  |  |  |  | 2.3e-13 / 2.0e-12 |  |  |  |  |  |  |  |
| caesar_500 | 1.53 | err | - | - | err | - | - | - | - | - | err | 51.4 | err | err | err | err | err | 39 | - | - | - | - | - | - | underflow | underflow | 448 | exact | 2.2e-16 / 2.5e-12 |  |  |  |  |  |  |  |  |  |  | 2.4e-34 / 2.5e-12 |  |  |  |  |  | 2.3e-13 / 9.1e-13 |  |  |  |  |  |  |  |
| caesar_1000 | 3.22 | >600 | - | - | err | - | - | - | - | - | err | 296 | >600 | err | err | err | >600 | 91.3 | - | - | - | - | - | - | underflow | underflow | >600 | exact | 1.1e-16 / 3.2e-11 |  |  |  |  |  |  |  |  |  |  | 1.3e-78 / 3.2e-11 |  |  |  |  |  | 1.8e-79 / 4.5e-12 |  |  |  |  |  |  |  |
| caesar_2000 | 7.01 | - | - | - | err | - | - | - | - | - | >600 | >600 | - | >600 | >600 | err | - | 246 | - | - | - | - | - | - | underflow | underflow | - | exact | 3.3e-16 / 6.4e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |
| caesar_5000 | 17.5 | - | - | - | err | - | - | - | - | - | - | - | - | - | - | >600 | - | >600 | - | - | - | - | - | - | underflow | underflow | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| caesar_10000 | 38.2 | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | underflow | underflow | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| caesar_20000 | 86.8 | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | underflow | underflow | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| caesar_50000 | oom | oom | - | - | err | - | - | - | - | - | >600 | - | oom | oom | oom | err | err | - | - | - | - | - | - | - | >600 | >600 | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| caesar_100000 | oom | oom | - | - | err | - | - | - | - | - | oom | - | oom | oom | oom | err | err | - | - | - | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| diamond_1000 | 0.038 | >600 | >600 | err | oom | >600 | err | >600 | 5.47 | >600 | 0.564 | 0.639 | 53.6 | 9.98 | 10.1 | 4.25 | 9.06 |  | 2.19 | 5.47 | 5.13 | 3.46 | 1.74 | >600 | 0.352 | 1.19 | 17 | exact | 1.1e-16 / 1.2e-16 |  |  |  |  |  |  |  | 4.0e-15 / 4.2e-15 |  | 1.0e-13 / 1.1e-13 | 0.0e+00 / 0.0e+00 | 5.3e-15 / 5.6e-15 | 5.3e-15 / 5.6e-15 | 5.3e-15 / 5.6e-15 | 5.3e-15 / 5.6e-15 | 5.3e-15 / 5.6e-15 |  | 2.4e-07 / 2.5e-07 | 4.1e-14 / 4.3e-14 | 4.1e-14 / 4.3e-14 | 4.1e-14 / 4.3e-14 | 2.1e-15 / 2.2e-15 |  | 3.3e-16 / 3.5e-16 |
| diamond_3000 | 0.11 | - | - | err | - | - | - | - | 23.1 | - | 3.32 | 3.87 | >600 | 118 | 117 | 7.8 | wrong |  | 14.5 | 61 | 28.7 | 28.8 | >600 | - | 1.02 | 1.84 | oom | exact | 1.1e-15 / 1.3e-15 |  |  |  |  |  |  |  | 1.3e-14 / 1.5e-14 |  | 2.9e-13 / 3.3e-13 | 0.0e+00 / 0.0e+00 |  | 1.3e-14 / 1.5e-14 | 1.3e-14 / 1.5e-14 | 1.3e-14 / 1.5e-14 | inf / inf |  | 2.5e-07 / 2.9e-07 | 3.9e-13 / 4.6e-13 | 3.9e-13 / 4.6e-13 | 3.9e-13 / 4.6e-13 |  |  | 3.0e-15 / 3.5e-15 |
| diamond_10000 | 0.399 | - | - | err | - | - | - | - | 200 | - | 31.9 | 46.1 | - | >600 | >600 | 18.5 | err |  | oom | >600 | 354 | 344 | - | - | 4.14 | 4.87 | oom | exact | 7.7e-15 / 1.3e-14 |  |  |  |  |  |  |  | 2.7e-14 / 4.4e-14 |  | 7.4e-13 / 1.2e-12 | 0.0e+00 / 0.0e+00 |  |  |  | 3.3e-14 / 5.5e-14 |  |  |  |  | 4.5e-14 / 7.4e-14 | 4.5e-14 / 7.4e-14 |  |  | 1.8e-14 / 3.0e-14 |
| diamond_30000 | 1.21 | - | - | >600 | - | - | - | - | >600 | - | 262 | 454 | - | - | - | 47.5 | err |  | - | - | >600 | >600 | - | - | 15 | 15.7 | oom | exact | 4.2e-14 / 1.9e-13 |  |  |  |  |  |  |  |  |  | 8.7e-13 / 3.9e-12 | 0.0e+00 / 0.0e+00 |  |  |  | 3.4e-14 / 1.5e-13 |  |  |  |  |  |  |  |  | 3.5e-14 / 1.6e-13 |
| diamond_100000 | 4.46 | - | - | - | - | - | - | - | - | - | >600 | >600 | - | - | - | 154 | err |  | - | - | - | - | - | - | 49.7 | 50.5 | oom | exact | 9.1e-14 / 1.3e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 3.5e-15 / 5.3e-13 |  |  |  |  |  |  |  |  | 6.0e-15 / 9.0e-13 |
| diamond_300000 | 17.9 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 520 | >600 |  | - | - | - | - | - | - | 178 | 179 | oom | exact | 5.1e-19 / 1.7e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 4.8e-19 / 1.6e-12 |  |  |  |  |  |  |  |  | 9.5e-19 / 3.1e-12 |
| diamond_1000000 | 62 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | err | - |  | - | - | - | - | - | - | >600 | >600 | >600 | exact | 1.1e-33 / 5.5e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| diamond_3000000 | oom | - | - | err | - | - | - | - | - | - | - | - | - | - | - | err | err |  | - | - | - | - | - | - | - | - | >600 | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| diamond_6000000 | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - | err | err |  | - | - | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hardware_10 | 0.011 | 0.359 | 0.232 | oom | 6.56 | >600 | 8.62 | 0.75 | 3.48 | oom | 0.166 | 0.165 | 1.57 | 1.28 | 1.28 | 2.02 | 1.76 |  | 0.14 | 0.276 | 1.63 | 0.145 | 0.131 | 28.3 | 0.041 | 0.77 | err | exact | 1.2e-18 / 1.2e-14 | 1.0e-07 / 1.0e+00 | 1.0e-07 / 1.0e+00 |  | 1.1e-16 / 6.8e-16 |  | 2.2e-16 / 1.2e-15 | 1.2e-15 / 4.8e-06 | 3.3e-16 / 3.3e-16 |  | 5.6e-16 / 3.4e-15 | 1.1e-16 / 1.1e-15 | 2.2e-16 / 1.2e-15 | 2.2e-16 / 1.2e-15 | 2.2e-16 / 1.2e-15 | 4.4e-16 / 9.5e-16 | 1.1e-16 / 1.9e-15 |  | 1.0e-07 / 1.0e+00 | 1.8e-14 / 2.1e-12 | 1.8e-14 / 2.1e-12 | 1.8e-14 / 2.1e-12 | 1.1e-16 / 5.4e-16 | 1.1e-16 / 8.1e-16 | 1.1e-16 / 3.3e-13 |
| hardware_30 | 0.032 | 1.32 | >600 | - | >600 | - | 9.75 | 36.4 | 4.19 | - | 0.206 | 0.249 | 3.42 | 1.28 | 1.28 | 2.26 | 2.63 |  | 0.226 | 32.3 | 2.04 | 0.31 | 0.623 | 115 | 0.217 | 0.948 | >600 | exact | 2.2e-16 / 1.1e-14 | 1.0e-07 / 1.0e+00 |  |  |  |  | 1.1e-16 / 2.1e-15 | 2.2e-16 / 1.0e+00 | 1.2e-15 / 2.4e-15 |  | 4.4e-16 / 1.3e-14 | 2.2e-16 / 3.6e-15 | 1.1e-16 / 1.9e-15 | 1.1e-16 / 1.9e-15 | 1.1e-16 / 1.9e-15 | 3.3e-16 / 3.6e-15 | 1.1e-16 / 2.6e-15 |  | 1.0e-07 / 1.0e+00 | 1.7e-14 / 3.3e-12 | 1.7e-14 / 3.3e-12 | 1.7e-14 / 3.3e-12 | 1.1e-16 / 2.3e-15 | 1.1e-16 / 1.9e-15 | 1.1e-16 / 1.1e-12 |
| hardware_100 | 0.091 | 4.02 | - | - | - | - | 19.5 | >600 | 24.9 | - | 0.499 | 0.831 | 5.94 | 1.65 | 1.64 | 4.31 | 6.22 |  | 0.83 | 185 | 6.14 | 2.28 | 1.38 | 465 | 2.3 | 3.09 | - | exact | 1.1e-16 / 1.1e-14 | 1.0e-07 / 1.0e+00 |  |  |  |  | 1.1e-16 / 7.5e-16 |  | 6.4e-15 / 7.5e-15 |  | 4.4e-16 / 4.1e-14 | 2.2e-16 / 3.5e-15 | 1.1e-16 / 7.5e-16 | 1.1e-16 / 7.5e-16 | 1.1e-16 / 7.5e-16 | 1.0e-15 / 2.0e-15 | 9.8e-13 / 9.8e-09 |  | 1.0e-07 / 1.0e+00 | 1.7e-14 / 3.2e-12 | 1.7e-14 / 3.2e-12 | 1.7e-14 / 3.2e-12 | 1.1e-16 / 1.0e-15 | 1.1e-16 / 3.0e-15 | 1.1e-16 / 3.7e-12 |
| hardware_300 | 0.277 | 12.3 | - | - | - | - | err | - | 213 | - | 2.68 | 5.57 | 24.4 | 5.17 | 5 | err | err | 8.6 | 4.99 | err | 43.5 | 22.2 | >600 | >600 | 22 | 23.1 | - | exact | 3.3e-16 / 1.1e-14 | 1.0e-07 / 8.0e-04 |  |  |  |  |  |  | 2.2e-14 / 2.2e-14 |  | 4.4e-16 / 1.8e-15 | 2.2e-16 / 5.4e-16 | 1.1e-16 / 5.4e-16 | 1.1e-16 / 5.4e-16 | 1.1e-16 / 5.4e-16 |  |  | 2.9e-15 / 1.1e-14 | 1.0e-07 / 8.0e-04 |  | 1.7e-14 / 3.2e-12 | 1.7e-14 / 3.2e-12 |  |  | 1.1e-16 / 1.1e-13 |
| hardware_1000 | 0.925 | 41.5 | - | - | - | - | - | - | >600 | - | 27.1 | 71.3 | 232 | 45.5 | 45.4 | err | >600 | 18.7 | oom | - | 424 | 217 | >600 | - | oom | oom | - | exact | 2.2e-16 / 1.1e-14 | 1.0e-07 / 8.0e-04 |  |  |  |  |  |  |  |  | 4.4e-16 / 1.8e-15 | 2.2e-16 / 5.4e-16 | 1.1e-16 / 5.4e-16 | 1.1e-16 / 5.4e-16 | 1.1e-16 / 5.4e-16 |  |  | 6.6e-15 / 7.1e-15 |  |  | 1.7e-14 / 3.2e-12 | 1.7e-14 / 3.2e-12 |  |  |  |
| hardware_3000 | 2.95 | 122 | - | - | - | - | - | - | - | - | 317 | >600 | >600 | 524 | 595 | err | - | 52.5 | - | - | >600 | >600 | - | - | - | - | - | exact | 2.2e-16 / 1.1e-14 | 1.0e-07 / 8.0e-04 |  |  |  |  |  |  |  |  | 4.4e-16 / 1.8e-15 |  |  | 1.1e-16 / 5.4e-16 | 1.1e-16 / 5.4e-16 |  |  | 8.3e-15 / 2.5e-14 |  |  |  |  |  |  |  |
| hardware_10000 | 11 | err | oom | oom | err | err | err | >600 | >600 | - | >600 | - | - | >600 | >600 | oom | - | err | - | - | - | - | - | - | - | - | oom | exact | 1.1e-16 / 1.1e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hardware_30000 | 48 | - | - | - | err | err | - | - | - | - | - | - | - | - | - | oom | oom | oom | - | - | - | - | - | - | - | - | - | exact | 3.3e-16 / 1.1e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hardware_100000 | 180 | - | - | - | >600 | oom | - | - | - | - | - | - | - | - | - | err | err | err | - | - | - | - | - | - | - | - | - | exact | 1.1e-16 / 1.4e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hardware_300000 | oom | - | - | - | err | oom | - | - | - | - | - | - | - | - | - | err | err | err | - | - | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hardware_1000000 | oom | - | - | - | err | err | - | - | - | - | - | - | - | - | - | err | err | err | - | - | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hardware_bitwidth_5 | 0.008 | wrong | 3.55 | >600 | 0.03 | 1.5 | 7.68 | 0.502 | 2.7 | oom | 0.198 | 0.202 | 1.24 | 1.68 | 1.49 | 1.15 | 1.13 |  | 0.065 | 0.207 | 1.7 | 0.083 | 0.132 | 8.06 | 0.007 | 0.533 | 0.187 | exact | 3.3e-16 / 6.8e-16 | 1.3e-07 / 4.0e+00 | 6.0e-08 / 1.0e+00 |  | 1.1e-16 / 5.0e-16 | 3.3e-16 / 3.7e-15 | 2.2e-16 / 5.4e-16 | 3.3e-16 / 1.0e-08 | 1.1e-16 / 6.6e-16 |  | 7.8e-16 / 1.2e-15 | 3.3e-16 / 6.6e-16 | 1.1e-16 / 5.4e-16 | 1.1e-16 / 5.4e-16 | 1.1e-16 / 5.4e-16 | 2.2e-16 / 5.0e-16 | 2.2e-16 / 6.6e-16 |  | 6.0e-08 / 1.0e+00 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 3.3e-16 / 1.3e-15 | 3.3e-16 / 8.3e-16 | 2.2e-16 / 2.2e-13 |
| hardware_bitwidth_10 | 0.009 | wrong | 2.88 | - | 0.028 | 1.36 | 7.33 | 0.53 | 2.76 | - | 0.209 | 0.21 | 516 | 120 | 115 | 1.13 | 1.17 |  | 0.062 | 0.18 | 1.52 | 0.074 | 0.0704 | 8.07 | 0.007 | 0.537 | 0.171 | exact | 3.3e-16 / 6.8e-16 | 1.3e-07 / 4.0e+00 | 6.0e-08 / 1.0e+00 |  | 1.1e-16 / 5.0e-16 | 3.3e-16 / 3.7e-15 | 2.2e-16 / 5.4e-16 | 3.3e-16 / 1.0e-08 | 1.1e-16 / 6.6e-16 |  | 7.8e-16 / 1.2e-15 | 3.3e-16 / 6.6e-16 | 1.1e-16 / 5.4e-16 | 1.1e-16 / 5.4e-16 | 1.1e-16 / 5.4e-16 | 2.2e-16 / 5.0e-16 | 2.2e-16 / 6.6e-16 |  | 6.0e-08 / 1.0e+00 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 3.3e-16 / 1.3e-15 | 3.3e-16 / 1.3e-15 | 2.2e-16 / 2.2e-13 |
| hardware_bitwidth_20 | 0.01 | wrong | oom | - | 0.028 | 1.37 | 7.26 | 0.513 | 2.75 | - | 0.194 | 0.198 | >600 | >600 | >600 | 1.15 | 1.23 |  | 0.054 | 0.148 | 1.5 | 0.058 | 0.12 | 8.21 | 0.008 | 0.541 | 0.16 | exact | 3.3e-16 / 6.8e-16 | 1.3e-07 / 4.0e+00 |  |  | 1.1e-16 / 5.0e-16 | 3.3e-16 / 3.7e-15 | 2.2e-16 / 5.4e-16 | 3.3e-16 / 1.0e-08 | 1.1e-16 / 6.6e-16 |  | 7.8e-16 / 1.2e-15 | 3.3e-16 / 6.6e-16 |  |  |  | 2.2e-16 / 5.0e-16 | 2.2e-16 / 6.6e-16 |  | 6.0e-08 / 1.0e+00 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 3.3e-16 / 1.3e-15 | 3.3e-16 / 9.9e-16 | 2.2e-16 / 2.2e-13 |
| hardware_bitwidth_50 | 0.01 | wrong | - | - | 0.029 | 1.36 | 7.05 | 0.508 | 2.76 | - | err | err | - | - | - | 1.2 | 1.25 |  | 0.058 | 0.148 | 1.53 | 0.057 | 0.0693 | 7.48 | 0.009 | 0.536 | 0.162 | exact | 3.3e-16 / 6.8e-16 | 1.3e-07 / 4.0e+00 |  |  | 1.1e-16 / 5.0e-16 | 3.3e-16 / 3.7e-15 | 2.2e-16 / 5.4e-16 | 3.3e-16 / 1.0e-08 | 1.1e-16 / 6.6e-16 |  |  |  |  |  |  | 2.2e-16 / 5.0e-16 | 2.2e-16 / 6.6e-16 |  | 6.0e-08 / 1.0e+00 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 3.3e-16 / 1.3e-15 | 3.3e-16 / 1.2e-15 | 2.2e-16 / 2.2e-13 |
| hardware_bitwidth_100 | 0.01 | wrong | - | - | 0.028 | 1.36 | 7.04 | 0.507 | 2.74 | - | err | err | - | - | - | 1.25 | 1.22 |  | 0.057 | 0.148 | 1.52 | 0.088 | 0.0675 | 7.9 | 0.02 | 0.547 | 0.162 | exact | 3.3e-16 / 6.8e-16 | 1.3e-07 / 4.0e+00 |  |  | 1.1e-16 / 5.0e-16 | 3.3e-16 / 3.7e-15 | 2.2e-16 / 5.4e-16 | 3.3e-16 / 1.0e-08 | 1.1e-16 / 6.6e-16 |  |  |  |  |  |  | 2.2e-16 / 5.0e-16 | 2.2e-16 / 6.6e-16 |  | 6.0e-08 / 1.0e+00 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 3.3e-16 / 1.3e-15 | 3.3e-16 / 1.3e-15 | 2.2e-16 / 2.2e-13 |
| hardware_bitwidth_300 | 0.01 | wrong | - | - | 0.028 | 1.38 | 7.02 | 0.52 | 2.75 | - | err | err | - | - | - | 1.24 | 1.29 |  | 0.084 | 0.153 | 1.51 | 0.058 | 0.0698 | 7.95 | 0.031 | 0.559 | 0.161 | exact | 3.3e-16 / 6.8e-16 | 1.3e-07 / 4.0e+00 |  |  | 1.1e-16 / 5.0e-16 | 3.3e-16 / 3.7e-15 | 2.2e-16 / 5.4e-16 | 3.3e-16 / 1.0e-08 | 1.1e-16 / 6.6e-16 |  |  |  |  |  |  | 2.2e-16 / 5.0e-16 | 2.2e-16 / 6.6e-16 |  | 6.0e-08 / 1.0e+00 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 3.3e-16 / 1.3e-15 | 3.3e-16 / 1.3e-15 | 2.2e-16 / 2.2e-13 |
| hardware_bitwidth_1000 | 0.009 | wrong | - | - | 0.029 | 1.38 | 7.05 | 0.519 | 2.72 | - | err | err | - | - | - | 1.22 | 1.25 |  | 0.061 | 0.15 | 1.51 | 0.055 | 0.0663 | 8.46 | 0.07 | 0.599 | 0.168 | exact | 3.3e-16 / 6.8e-16 | 1.3e-07 / 4.0e+00 |  |  | 1.1e-16 / 5.0e-16 | 3.3e-16 / 3.7e-15 | 2.2e-16 / 5.4e-16 | 3.3e-16 / 1.0e-08 | 1.1e-16 / 6.6e-16 |  |  |  |  |  |  | 2.2e-16 / 5.0e-16 | 2.2e-16 / 6.6e-16 |  | 6.0e-08 / 1.0e+00 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 3.3e-16 / 1.3e-15 | 3.3e-16 / 9.9e-16 | 2.2e-16 / 2.2e-13 |
| hardware_bitwidth_3000 | 0.01 | wrong | - | - | 0.027 | 1.36 | 7.01 | 0.52 | 2.73 | - | err | err | - | - | - | 1.24 | 1.25 |  | 0.056 | 0.148 | 1.52 | 0.059 | 0.105 | 6.34 | 0.197 | 0.726 | 0.167 | exact | 3.3e-16 / 6.8e-16 | 1.3e-07 / 4.0e+00 |  |  | 1.1e-16 / 5.0e-16 | 3.3e-16 / 3.7e-15 | 2.2e-16 / 5.4e-16 | 3.3e-16 / 1.0e-08 | 1.1e-16 / 6.6e-16 |  |  |  |  |  |  | 2.2e-16 / 5.0e-16 | 2.2e-16 / 6.6e-16 |  | 6.0e-08 / 1.0e+00 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 3.3e-16 / 1.3e-15 | 3.3e-16 / 1.3e-15 | 2.2e-16 / 2.2e-13 |
| hardware_bitwidth_10000 | 0.009 | wrong | - | - | 0.03 | 1.37 | 6.58 | 0.517 | 2.74 | - | err | err | - | - | - | 1.23 | 1.27 |  | 0.059 | 0.148 | 1.51 | 0.057 | 0.0633 | 7.26 | 0.574 | 1.11 | 0.166 | exact | 3.3e-16 / 6.8e-16 | 1.3e-07 / 4.0e+00 |  |  | 1.1e-16 / 5.0e-16 | 3.3e-16 / 3.7e-15 | 2.2e-16 / 5.4e-16 | 3.3e-16 / 1.0e-08 | 1.1e-16 / 6.6e-16 |  |  |  |  |  |  | 2.2e-16 / 5.0e-16 | 2.2e-16 / 6.6e-16 |  | 6.0e-08 / 1.0e+00 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 3.3e-16 / 1.3e-15 | 3.3e-16 / 1.3e-15 | 2.2e-16 / 2.2e-13 |
| hardware_bitwidth_30000 | 0.009 | wrong | - | - | 0.03 | 1.36 | 6.37 | 0.509 | 2.72 | - | err | err | - | - | - | 1.26 | 1.28 |  | 0.056 | 0.147 | 1.54 | 0.055 | 0.061 | 6.74 | 1.74 | 2.27 | 0.172 | exact | 3.3e-16 / 6.8e-16 | 1.3e-07 / 4.0e+00 |  |  | 1.1e-16 / 5.0e-16 | 3.3e-16 / 3.7e-15 | 2.2e-16 / 5.4e-16 | 3.3e-16 / 1.0e-08 | 1.1e-16 / 6.6e-16 |  |  |  |  |  |  | 2.2e-16 / 5.0e-16 | 2.2e-16 / 6.6e-16 |  | 6.0e-08 / 1.0e+00 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 3.3e-16 / 1.3e-15 | 3.3e-16 / 1.2e-15 | 2.2e-16 / 2.2e-13 |
| hardware_bitwidth_100000 | 0.01 | wrong | - | - | 0.03 | 1.34 | 6.37 | 0.506 | 2.72 | - | err | err | - | - | - | 1.27 | 1.28 |  | 0.052 | 0.147 | 1.53 | 0.057 | 0.063 | 6.84 | 5.86 | 6.4 | 0.168 | exact | 3.3e-16 / 6.8e-16 | 1.3e-07 / 4.0e+00 |  |  | 1.1e-16 / 5.0e-16 | 3.3e-16 / 3.7e-15 | 2.2e-16 / 5.4e-16 | 3.3e-16 / 1.0e-08 | 1.1e-16 / 6.6e-16 |  |  |  |  |  |  | 2.2e-16 / 5.0e-16 | 2.2e-16 / 6.6e-16 |  | 6.0e-08 / 1.0e+00 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 3.3e-16 / 1.3e-15 | 3.3e-16 / 9.9e-16 | 2.2e-16 / 2.2e-13 |
| hardware_bitwidth_300000 | 0.009 | wrong | - | - | 0.03 | 1.38 | 6.4 | 0.512 | 2.72 | - | err | err | - | - | - | 1.22 | 1.29 |  | 0.055 | 0.149 | 1.54 | 0.056 | 0.062 | 6.24 | 18.5 | 19 | 0.17 | exact | 3.3e-16 / 6.8e-16 | 1.3e-07 / 4.0e+00 |  |  | 1.1e-16 / 5.0e-16 | 3.3e-16 / 3.7e-15 | 2.2e-16 / 5.4e-16 | 3.3e-16 / 1.0e-08 | 1.1e-16 / 6.6e-16 |  |  |  |  |  |  | 2.2e-16 / 5.0e-16 | 2.2e-16 / 6.6e-16 |  | 6.0e-08 / 1.0e+00 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 3.3e-16 / 1.3e-15 | 3.3e-16 / 9.9e-16 | 2.2e-16 / 2.2e-13 |
| hardware_bitwidth_1000000 | 0.008 | wrong | - | - | 0.03 | 1.36 | 6.39 | 0.512 | 2.73 | - | err | err | - | - | - | 1.27 | 1.3 |  | 0.053 | 0.151 | 1.53 | 0.055 | 0.0622 | 6.74 | 68.2 | 68.7 | 0.164 | exact | 3.3e-16 / 6.8e-16 | 1.3e-07 / 4.0e+00 |  |  | 1.1e-16 / 5.0e-16 | 3.3e-16 / 3.7e-15 | 2.2e-16 / 5.4e-16 | 3.3e-16 / 1.0e-08 | 1.1e-16 / 6.6e-16 |  |  |  |  |  |  | 2.2e-16 / 5.0e-16 | 2.2e-16 / 6.6e-16 |  | 6.0e-08 / 1.0e+00 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 3.3e-16 / 1.3e-15 | 3.3e-16 / 9.9e-16 | 2.2e-16 / 2.2e-13 |
| hardware_bitwidth_3000000 | 0.009 | wrong | - | - | 0.03 | 1.36 | 6.61 | 0.518 | 2.72 | - | err | err | - | - | - | 1.22 | 1.31 |  | 0.053 | 0.15 | 1.52 | 0.053 | 0.0612 | 6.68 | 263 | 263 | 0.166 | exact | 3.3e-16 / 6.8e-16 | 1.3e-07 / 4.0e+00 |  |  | 1.1e-16 / 5.0e-16 | 3.3e-16 / 3.7e-15 | 2.2e-16 / 5.4e-16 | 3.3e-16 / 1.0e-08 | 1.1e-16 / 6.6e-16 |  |  |  |  |  |  | 2.2e-16 / 5.0e-16 | 2.2e-16 / 6.6e-16 |  | 6.0e-08 / 1.0e+00 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 3.3e-16 / 1.3e-15 | 3.3e-16 / 8.3e-16 | 2.2e-16 / 2.2e-13 |
| hardware_bitwidth_10000000 | 0.008 | wrong | - | - | 0.03 | 1.35 | 6.39 | 0.52 | 2.73 | - | err | err | - | - | - | 1.26 | 1.32 |  | 0.057 | 0.148 | 1.53 | 0.055 | 0.0611 | 7.06 | >600 | >600 | 0.165 | exact | 3.3e-16 / 6.8e-16 | 1.3e-07 / 4.0e+00 |  |  | 1.1e-16 / 5.0e-16 | 3.3e-16 / 3.7e-15 | 2.2e-16 / 5.4e-16 | 3.3e-16 / 1.0e-08 | 1.1e-16 / 6.6e-16 |  |  |  |  |  |  | 2.2e-16 / 5.0e-16 | 2.2e-16 / 6.6e-16 |  | 6.0e-08 / 1.0e+00 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 3.3e-16 / 1.3e-15 | 3.3e-16 / 1.2e-15 |  |
| hardware_bitwidth_100000000 | 0.009 | wrong | - | - | 0.029 | 1.37 | 6.32 | 0.521 | 2.74 | - | err | err | - | - | - | 1.24 | 1.27 |  | 0.056 | 0.147 | 1.54 | 0.058 | 0.0609 | 6.94 | - | - | 0.164 | exact | 3.3e-16 / 6.8e-16 | 1.3e-07 / 4.0e+00 |  |  | 1.1e-16 / 5.0e-16 | 3.3e-16 / 3.7e-15 | 2.2e-16 / 5.4e-16 | 3.3e-16 / 1.0e-08 | 1.1e-16 / 6.6e-16 |  |  |  |  |  |  | 2.2e-16 / 5.0e-16 | 2.2e-16 / 6.6e-16 |  | 6.0e-08 / 1.0e+00 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 5.0e-16 / 5.0e-12 | 3.3e-16 / 1.3e-15 | 3.3e-16 / 1.3e-15 |  |
| hmm_filtering_joint_100 | 0.014 | >600 | zero | err | >600 | >600 | err | 55.3 | 3.39 | oom | >600 | >600 | 8.66 | err | err | 2.08 | 2.3 |  | zero | 2.79 | 27.5 | 0.355 | 0.277 | 87.1 | 0.057 | 0.773 | 0.635 | exact | 5.8e-32 / 1.5e-13 |  |  |  |  |  |  | 1.9e-19 / 5.2e-01 | 5.8e-32 / 1.6e-13 |  |  |  | 5.9e-32 / 1.6e-13 |  |  | 5.9e-32 / 1.6e-13 | 5.9e-32 / 1.6e-13 |  |  | 5.4e-31 / 1.4e-12 | 5.4e-31 / 1.4e-12 | 5.4e-31 / 1.4e-12 | 5.9e-32 / 1.6e-13 | 5.9e-32 / 1.6e-13 | 5.9e-32 / 1.6e-13 |
| hmm_filtering_joint_300 | 0.046 | - | zero | err | - | - | - | >600 | 8.42 | - | - | - | 38.5 | err | err | 2.81 | 3.41 |  | zero | 1.45 | 2.17 | 0.916 | 0.703 | 324 | 0.348 | 1.06 | 2.05 | exact | 3.8e-69 / 2.8e-14 |  |  |  |  |  |  |  | 5.7e-69 / 4.2e-14 |  |  |  | 5.3e-69 / 3.8e-14 |  |  | 5.6e-69 / 4.1e-14 | 5.6e-69 / 4.1e-14 |  |  | 1.2e-67 / 8.9e-13 | 1.2e-67 / 8.9e-13 | 1.2e-67 / 8.9e-13 | 5.4e-69 / 3.9e-14 | 5.5e-69 / 4.0e-14 | 5.5e-69 / 4.0e-14 |
| hmm_filtering_joint_1000 | 0.106 | - | wrong | err | - | - | - | - | underflow | - | - | - | >600 | err | err | err | err | 5.93 | underflow | err | err | 12 | >600 | >600 | underflow | underflow | 13 | exact | 5.5e-194 / 1.4e-11 |  | 5.0e-01 / 1.2e+182 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 6.4e-194 / 1.6e-11 |  |  |  | 5.9e-194 / 1.5e-11 |  |  |  |
| hmm_filtering_joint_3000 | 0.351 | - | oom | err | - | - | - | - | >600 | - | - | - | - | >600 | >600 | oom | oom | 12.2 | oom | >600 | err | 97.7 | - | - | 40 | 41.5 | oom | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  | 0.0e+00 / 0.0e+00 |  |  | 0.0e+00 / 0.0e+00 |
| hmm_filtering_joint_10000 | 1.21 | - | - | err | - | - | - | - | - | - | - | - | - | - | - | oom | oom | 31.3 | - | - | >600 | >600 | - | - | oom | oom | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |
| hmm_filtering_joint_30000 | 3.64 | - | - | err | - | - | - | - | - | - | - | - | - | - | - | oom | oom | 77.4 | - | - | - | - | - | - | - | - | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |
| hmm_filtering_joint_100000 | 13.2 | - | - | err | - | - | - | - | - | - | - | - | - | - | - | oom | oom | 264 | - | - | - | - | - | - | - | - | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |
| hmm_filtering_joint_300000 | 59.3 | - | - | err | - | - | - | - | - | - | - | - | - | err | err | >600 | >600 | >600 | - | >600 | >600 | - | - | - | - | - | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_filtering_joint_1000000 | oom | - | - | oom | - | - | - | - | - | - | - | - | - | err | err | - | - | - | - | - | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_filtering_joint_3000000 | - | - | - | oom | - | - | - | - | - | - | - | - | - | oom | oom | - | - | - | - | - | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_filtering_marginal_100 | 0.027 | >600 | 0.165 | err | >600 | 11.9 | err | wrong | 3.29 | oom | 0.181 | 0.189 | 3.9 | 1.3 | 1.31 | 2.13 | 2.26 |  | 0.297 | 0.472 | 25.9 | 0.34 | 0.498 | 76.1 | 0.056 | 0.843 | 0.531 | exact | 6.2e-15 / 1.0e-14 |  | 1.5e-07 / 2.4e-07 |  |  | 5.9e-15 / 9.5e-15 |  | 8.1e-01 / 1.3e+00 | 5.8e-15 / 9.3e-15 |  | 6.0e-15 / 9.6e-15 | 5.7e-15 / 9.1e-15 | 5.9e-15 / 9.5e-15 | 5.6e-15 / 8.9e-15 | 5.6e-15 / 8.9e-15 | 4.9e-15 / 7.8e-15 | 5.6e-15 / 8.9e-15 |  | 1.5e-07 / 2.4e-07 | 2.0e-13 / 3.2e-13 | 2.0e-13 / 3.2e-13 | 2.0e-13 / 3.2e-13 | 5.8e-15 / 9.3e-15 | 5.8e-15 / 9.3e-15 | 6.0e-15 / 9.6e-15 |
| hmm_filtering_marginal_300 | 0.042 | - | 0.659 | err | - | 148 | - | >600 | 7.53 | - | 0.285 | 0.291 | 18 | 3.31 | 3.34 | 2.76 | 3.3 |  | 0.598 | 1.36 | 2.17 | 0.808 | 0.797 | 292 | 0.303 | 1.02 | 1.88 | exact | 6.0e-15 / 9.6e-15 |  | 1.5e-07 / 2.4e-07 |  |  | 5.7e-15 / 9.1e-15 |  |  | 5.6e-15 / 8.9e-15 |  | 6.0e-15 / 9.6e-15 | 5.7e-15 / 9.1e-15 | 6.6e-15 / 1.1e-14 | 6.6e-15 / 1.1e-14 | 6.6e-15 / 1.1e-14 | 6.3e-15 / 1.0e-14 | 5.7e-15 / 9.1e-15 |  | 1.5e-07 / 2.4e-07 | 2.0e-13 / 3.2e-13 | 2.0e-13 / 3.2e-13 | 2.0e-13 / 3.2e-13 | 5.8e-15 / 9.3e-15 | 5.8e-15 / 9.3e-15 | 6.2e-15 / 1.0e-14 |
| hmm_filtering_marginal_1000 | 0.11 | - | wrong | err | - | >600 | - | - | underflow | - | err | 0.976 | err | err | err | err | err | 5.67 | underflow | err | err | 7.99 | >600 | >600 | underflow | underflow | 12.3 | exact | 3.0e-14 / 4.9e-14 |  | 1.2e-01 / 2.0e-01 |  |  |  |  |  |  |  |  | 3.0e-14 / 4.8e-14 |  |  |  |  |  | 2.1e-13 / 3.4e-13 |  |  |  | 1.7e-13 / 2.7e-13 |  |  |  |
| hmm_filtering_marginal_3000 | 2.48 | - | oom | err | - | - | - | - | >600 | - | err | 7.98 | >600 | >600 | >600 | oom | oom | 11 | oom | err | err | 76.9 | - | - | underflow | underflow | oom | exact | 1.1e-13 / 1.8e-13 |  |  |  |  |  |  |  |  |  |  | 1.1e-13 / 1.8e-13 |  |  |  |  |  | 1.0e-10 / 1.6e-10 |  |  |  | 3.1e-13 / 4.9e-13 |  |  |  |
| hmm_filtering_marginal_10000 | 3.02 | - | - | err | - | - | - | - | - | - | err | 82.9 | - | - | - | oom | oom | 26.4 | - | >600 | >600 | >600 | - | - | oom | oom | oom | exact | 4.0e-13 / 6.4e-13 |  |  |  |  |  |  |  |  |  |  | 3.9e-13 / 6.3e-13 |  |  |  |  |  | 7.7e-11 / 1.2e-10 |  |  |  |  |  |  |  |
| hmm_filtering_marginal_30000 | 3.66 | - | - | err | - | - | - | - | - | - | >600 | >600 | - | - | - | oom | oom | 80.5 | - | - | - | - | - | - | - | - | oom | exact | 9.7e-13 / 1.6e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 6.8e-12 / 1.1e-11 |  |  |  |  |  |  |  |
| hmm_filtering_marginal_100000 | 11.8 | - | - | err | - | - | - | - | - | - | - | - | - | - | - | oom | oom | 258 | - | - | - | - | - | - | - | - | oom | exact | 3.3e-12 / 5.2e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.4e-08 / 8.6e-08 |  |  |  |  |  |  |  |
| hmm_filtering_marginal_300000 | 50.8 | - | - | err | - | - | - | - | - | - | err | - | err | err | err | >600 | >600 | >600 | - | >600 | >600 | - | - | - | - | - | oom | exact | 1.5e-11 / 2.4e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_filtering_marginal_1000000 | oom | - | - | err | - | - | - | - | - | - | err | - | err | err | err | - | - | - | - | - | - | - | - | - | - | - | >600 | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_filtering_marginal_3000000 | - | - | - | err | - | - | - | - | - | - | err | - | err | err | err | - | - | - | - | - | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_marginal_likelihood_100 | 0.016 | >600 | zero | err | >600 | >600 | err | 2.94 | 3.01 | oom | >600 | >600 | 11.6 | err | err | 2.1 | 2.39 |  | zero | 0.579 | 27.5 | 0.398 | 0.3 | 100 | 0.049 | 0.767 | oom | exact | 1.7e-48 / 6.1e-14 |  |  |  |  |  |  | 1.7e-48 / 6.1e-14 | 1.8e-48 / 6.3e-14 |  |  |  | 1.7e-48 / 6.1e-14 |  |  | 1.7e-48 / 6.1e-14 | 1.7e-48 / 6.1e-14 |  |  | 3.1e-47 / 1.1e-12 | 3.1e-47 / 1.1e-12 | 3.1e-47 / 1.1e-12 | 1.8e-48 / 6.4e-14 | 1.8e-48 / 6.5e-14 | 1.7e-48 / 6.0e-14 |
| hmm_marginal_likelihood_300 | 0.038 | - | zero | err | - | - | - | 60.5 | 4.56 | - | - | - | 73.7 | err | err | 2.98 | 3.73 |  | zero | 1.65 | 2.38 | 1.15 | 0.785 | 369 | 0.257 | 0.996 | - | exact | 2.7e-116 / 1.2e-12 |  |  |  |  |  |  | 2.6e-116 / 1.2e-12 | 2.7e-116 / 1.2e-12 |  |  |  | 2.7e-116 / 1.2e-12 |  |  | 2.7e-116 / 1.2e-12 | 2.7e-116 / 1.2e-12 |  |  | 2.6e-116 / 1.2e-12 | 2.6e-116 / 1.2e-12 | 2.6e-116 / 1.2e-12 | 2.7e-116 / 1.2e-12 | 2.7e-116 / 1.2e-12 | 2.7e-116 / 1.2e-12 |
| hmm_marginal_likelihood_600 | 0.098 | - | zero | err | - | - | - | 419 | 8.41 | - | - | - | 529 | err | err | 4.4 | 6.16 |  | zero | 6.25 | 5.42 | 4.34 | 2.05 | >600 | 0.995 | 1.77 | - | exact | 3.2e-219 / 7.0e-12 |  |  |  |  |  |  | 3.2e-219 / 7.0e-12 | 3.2e-219 / 7.0e-12 |  |  |  | 3.2e-219 / 7.0e-12 |  |  | 3.2e-219 / 7.0e-12 | 3.2e-219 / 7.0e-12 |  |  | 3.6e-219 / 7.8e-12 | 3.6e-219 / 7.8e-12 | 3.6e-219 / 7.8e-12 | 3.2e-219 / 7.0e-12 |  | 3.2e-219 / 7.0e-12 |
| hmm_marginal_likelihood_900 | 0.13 | - | underflow | err | - | - | - | >600 | 14.2 | - | - | - | >600 | >600 | >600 | 12.3 | 15 |  | underflow | 22 | 15.9 | 15.6 | >600 | - | 2.27 | 3.07 | - | exact | 1.2e-322 / 1.3e-11 |  |  |  |  |  |  |  | 1.2e-322 / 1.3e-11 |  |  |  |  |  |  | 1.2e-322 / 1.2e-11 | 1.2e-322 / 1.2e-11 |  |  | 1.2e-322 / 1.3e-11 | 1.2e-322 / 1.3e-11 | 1.2e-322 / 1.3e-11 |  |  | 1.2e-322 / 1.3e-11 |
| hmm_smoothing_joint_100 | 0.014 | >600 | zero | err | oom | >600 | err | 15.4 | 3.35 | oom | >600 | >600 | 8.88 | err | err | 2.05 | 2.26 |  | zero | 0.342 | 1.65 | 0.18 | 0.271 | 87 | 0.055 | 0.769 | 0.602 | exact | 1.3e-31 / 2.0e-13 |  |  |  |  |  |  | 6.3e-19 / 1.0e+00 | 1.3e-31 / 2.0e-13 |  |  |  | 1.3e-31 / 2.0e-13 |  |  | 1.3e-31 / 2.0e-13 | 1.3e-31 / 2.0e-13 |  |  | 1.0e-31 / 1.6e-13 | 1.0e-31 / 1.6e-13 | 1.0e-31 / 1.6e-13 | 1.3e-31 / 2.0e-13 | 1.3e-31 / 2.0e-13 | 1.3e-31 / 2.0e-13 |
| hmm_smoothing_joint_300 | 0.036 | - | zero | err | - | - | - | >600 | 8.42 | - | - | - | 42.3 | err | err | 3.06 | 3.87 |  | zero | 1.49 | 2.28 | 0.922 | 0.661 | 324 | 0.326 | 1.03 | 2.4 | exact | 3.1e-67 / 1.3e-12 |  |  |  |  |  |  |  | 3.1e-67 / 1.4e-12 |  |  |  | 3.1e-67 / 1.3e-12 |  |  | 3.1e-67 / 1.4e-12 | 3.1e-67 / 1.4e-12 |  |  | 9.1e-68 / 4.0e-13 | 9.1e-68 / 4.0e-13 | 9.1e-68 / 4.0e-13 | 3.1e-67 / 1.4e-12 | 3.1e-67 / 1.3e-12 | 3.1e-67 / 1.4e-12 |
| hmm_smoothing_joint_1000 | 0.123 | - | wrong | err | - | - | - | - | underflow | - | - | - | >600 | err | err | err | err | 6.05 | underflow | err | err | 8.88 | >600 | >600 | underflow | underflow | 16.9 | exact | 2.0e-193 / 2.9e-11 |  | 5.0e-01 / 7.4e+181 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 9.6e-194 / 1.4e-11 |  |  |  | 1.9e-193 / 2.9e-11 |  |  |  |
| hmm_smoothing_joint_3000 | 0.313 | - | oom | err | - | - | - | - | >600 | - | - | - | - | >600 | >600 | oom | oom | 11.7 | err | >600 | err | 86.4 | - | - | 36.4 | 37.9 | oom | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  | 0.0e+00 / 0.0e+00 |  |  | 0.0e+00 / 0.0e+00 |
| hmm_smoothing_joint_10000 | 1.12 | - | - | err | - | - | - | - | - | - | - | - | - | - | - | oom | oom | 30.4 | - | - | >600 | >600 | - | - | oom | oom | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |
| hmm_smoothing_joint_30000 | 3.54 | - | - | err | - | - | - | - | - | - | - | - | - | - | - | oom | oom | 78.9 | - | - | - | - | - | - | - | - | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |
| hmm_smoothing_joint_100000 | 12.8 | - | - | err | - | - | - | - | - | - | - | - | - | - | - | oom | oom | 258 | - | - | - | - | - | - | - | - | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |
| hmm_smoothing_joint_300000 | 47 | - | - | err | - | - | - | - | - | - | - | - | - | err | err | >600 | >600 | >600 | - | >600 | >600 | - | - | - | - | - | - | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_smoothing_joint_1000000 | oom | - | - | oom | - | - | - | - | - | - | - | - | - | err | err | err | err | err | - | err | err | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_smoothing_joint_3000000 | oom | - | - | oom | - | - | - | - | - | - | - | - | - | oom | oom | - | - | - | - | oom | oom | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_smoothing_marginal_100 | 0.027 | >600 | 0.165 | err | >600 | 11.8 | err | 2.96 | 3.31 | oom | 0.195 | 0.198 | 3.92 | 1.33 | 1.3 | 2.07 | 2.31 |  | 0.298 | 0.664 | 27.5 | 0.338 | 0.497 | 74.7 | 0.054 | 0.794 | 0.513 | exact | 5.6e-17 / 1.2e-16 |  | 3.5e-07 / 7.8e-07 |  |  | 3.3e-16 / 7.4e-16 |  | 2.2e-16 / 4.9e-16 | 5.6e-17 / 1.2e-16 |  | 5.6e-17 / 1.2e-16 | 1.1e-16 / 2.5e-16 | 1.1e-16 / 2.5e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 2.8e-16 / 6.2e-16 | 2.2e-16 / 4.9e-16 |  | 3.5e-07 / 7.8e-07 | 3.9e-13 / 8.7e-13 | 3.9e-13 / 8.7e-13 | 3.9e-13 / 8.7e-13 | 1.7e-16 / 3.7e-16 | 1.7e-16 / 3.7e-16 | 5.6e-17 / 1.2e-16 |
| hmm_smoothing_marginal_300 | 0.066 | - | 0.666 | err | - | 147 | - | 64.3 | 7.6 | - | 0.323 | 0.322 | 18.3 | 3.34 | 3.39 | 2.75 | 3.32 |  | 0.45 | 1.37 | 2.03 | 0.768 | 0.766 | 290 | 0.298 | 1.02 | 1.78 | exact | 6.3e-15 / 1.4e-14 |  | 3.5e-07 / 7.8e-07 |  |  | 8.0e-15 / 1.8e-14 |  | 6.2e-15 / 1.4e-14 | 6.8e-15 / 1.5e-14 |  | 6.3e-15 / 1.4e-14 | 6.3e-15 / 1.4e-14 | 6.2e-15 / 1.4e-14 | 6.2e-15 / 1.4e-14 | 6.2e-15 / 1.4e-14 | 6.8e-15 / 1.5e-14 | 6.0e-15 / 1.3e-14 |  | 3.5e-07 / 7.8e-07 | 3.9e-13 / 8.6e-13 | 3.9e-13 / 8.6e-13 | 3.9e-13 / 8.6e-13 | 6.3e-15 / 1.4e-14 | 6.2e-15 / 1.4e-14 | 6.2e-15 / 1.4e-14 |
| hmm_smoothing_marginal_1000 | 0.11 | - | wrong | err | - | >600 | - | >600 | underflow | - | err | 1.4 | err | err | err | err | err | 5.7 | underflow | err | err | 11.1 | >600 | >600 | underflow | underflow | 10.9 | exact | 3.2e-14 / 7.1e-14 |  | 5.0e-02 / 1.1e-01 |  |  |  |  |  |  |  |  | 3.2e-14 / 7.1e-14 |  |  |  |  |  | 2.0e-13 / 4.5e-13 |  |  |  | 3.6e-13 / 8.0e-13 |  |  |  |
| hmm_smoothing_marginal_3000 | 0.315 | - | oom | err | - | - | - | - | >600 | - | err | 12 | >600 | >600 | >600 | oom | oom | 11.4 | oom | err | err | 72.9 | - | - | underflow | underflow | err | exact | 3.2e-14 / 7.1e-14 |  |  |  |  |  |  |  |  |  |  | 3.2e-14 / 7.1e-14 |  |  |  |  |  | 6.8e-12 / 1.5e-11 |  |  |  | 3.6e-13 / 8.0e-13 |  |  |  |
| hmm_smoothing_marginal_10000 | 3.52 | - | - | err | - | - | - | - | - | - | err | 126 | - | - | - | oom | oom | 28.5 | - | >600 | >600 | >600 | - | - | oom | oom | - | exact | 1.7e-13 / 3.8e-13 |  |  |  |  |  |  |  |  |  |  | 1.7e-13 / 3.8e-13 |  |  |  |  |  | 1.9e-10 / 4.1e-10 |  |  |  |  |  |  |  |
| hmm_smoothing_marginal_30000 | 6.18 | - | - | err | - | - | - | - | - | - | >600 | >600 | - | - | - | oom | oom | 74.6 | - | - | - | - | - | - | - | - | - | exact | 6.5e-13 / 1.4e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 7.4e-11 / 1.6e-10 |  |  |  |  |  |  |  |
| hmm_smoothing_marginal_100000 | 12 | - | - | err | - | - | - | - | - | - | - | - | - | - | - | oom | oom | 245 | - | - | - | - | - | - | - | - | - | exact | 2.6e-12 / 5.8e-12 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.0e-10 / 2.3e-10 |  |  |  |  |  |  |  |
| hmm_smoothing_marginal_300000 | 54.8 | - | - | err | - | - | - | - | - | - | err | - | err | err | err | >600 | >600 | >600 | - | >600 | >600 | - | - | - | - | - | - | exact | 1.6e-11 / 3.5e-11 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_smoothing_marginal_1000000 | oom | - | - | err | - | - | - | - | - | - | err | - | err | err | err | - | - | - | - | - | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| hmm_smoothing_marginal_3000000 | - | - | - | err | - | - | - | - | - | - | err | - | err | err | err | - | - | - | - | - | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| ladder_1000 | 0.073 | >600 | >600 | err | err | >600 | err | >600 | 241 | oom | 3.72 | 3.79 | 186 | 34.7 | 34.9 | 6.67 | 10.6 |  | 6.8 | 22.9 | 53.1 | 21.1 | >600 | >600 | 0.393 | 1.11 | >600 | exact | 4.4e-16 / 1.5e-15 |  |  |  |  |  |  |  | 3.3e-16 / 1.1e-15 |  | 2.8e-16 / 9.2e-16 | 8.3e-16 / 2.7e-15 | 2.8e-16 / 9.2e-16 | 2.8e-16 / 9.2e-16 | 2.8e-16 / 9.2e-16 | 6.1e-16 / 2.0e-15 | 6.1e-16 / 2.0e-15 |  | 2.8e-07 / 9.2e-07 | 4.6e-13 / 1.5e-12 | 4.6e-13 / 1.5e-12 | 4.6e-13 / 1.5e-12 |  |  | 7.8e-16 / 2.6e-15 |
| ladder_3000 | 0.235 | - | - | err | err | - | - | - | >600 | - | 30 | 32.3 | >600 | 408 | 384 | 13.9 | wrong |  | err | 292 | 270 | 219 | - | - | 1.67 | 2.39 | - | exact | 1.4e-16 / 1.2e-15 |  |  |  |  |  |  |  |  |  | 2.9e-16 / 2.6e-15 | 2.4e-16 / 2.1e-15 |  | 2.9e-16 / 2.6e-15 | 2.9e-16 / 2.6e-15 | 1.2e-16 / 1.1e-15 | inf / inf |  |  | 3.0e-13 / 2.7e-12 | 3.0e-13 / 2.7e-12 | 3.0e-13 / 2.7e-12 |  |  | 6.2e-16 / 5.6e-15 |
| ladder_10000 | 0.795 | - | - | err | err | - | - | - | - | - | 307 | 330 | - | >600 | >600 | 34.3 | err |  | - | >600 | >600 | >600 | - | - | 5.42 | 6.17 | - | exact | 1.3e-18 / 3.9e-16 |  |  |  |  |  |  |  |  |  | 1.4e-17 / 4.3e-15 | 1.1e-17 / 3.2e-15 |  |  |  | 2.2e-17 / 6.4e-15 |  |  |  |  |  |  |  |  | 3.9e-17 / 1.2e-14 |
| ladder_30000 | 2.54 | - | - | >600 | err | - | - | - | - | - | >600 | >600 | - | - | - | 103 | err |  | - | - | - | - | - | - | 19.8 | 20.6 | - | exact | 1.4e-21 / 9.0e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.1e-21 / 7.1e-15 |  |  |  |  |  |  |  |  | 3.5e-21 / 2.3e-14 |
| ladder_100000 | 8.91 | - | - | - | err | - | - | - | - | - | - | - | - | - | - | 319 | err |  | - | - | - | - | - | - | 81.4 | 82.2 | - | exact | 8.4e-36 / 8.8e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 8.4e-36 / 8.8e-14 |  |  |  |  |  |  |  |  | 8.1e-36 / 8.6e-14 |
| ladder_300000 | 28.4 | - | - | - | err | - | - | - | - | - | - | - | - | - | - | >600 | >600 |  | - | - | - | - | - | - | 233 | 234 | - | exact | 3.3e-79 / 9.6e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.6e-79 / 4.7e-14 |
| ladder_1000000 | 119 | - | - | - | err | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | >600 | >600 | - | exact | 2.8e-231 / 9.0e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| ladder_3000000 | oom | - | - | err | err | - | - | - | - | - | - | - | - | - | - | - | err |  | - | - | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| motivating_1000 | 0.037 | >600 | 24.3 | >600 | oom | 594 | 7.45 | >600 | 8.88 | oom | 0.608 | 0.647 | 53.9 | 9.96 | 9.96 | 3.45 | 4.33 |  | 1.35 | 3.65 | 20.5 | 2.45 | 1.27 | >600 | 0.136 | 0.922 | 5.51 | exact | 5.6e-16 / 2.0e-15 |  | 2.7e-07 / 1.0e-06 |  |  | 5.6e-17 / 2.0e-16 | 2.2e-16 / 8.1e-16 |  | 3.9e-16 / 1.4e-15 |  | 3.9e-16 / 1.4e-15 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 5.6e-17 / 2.0e-16 | 5.6e-17 / 2.0e-16 |  | 2.7e-07 / 1.0e-06 | 2.7e-13 / 1.0e-12 | 2.7e-13 / 1.0e-12 | 2.7e-13 / 1.0e-12 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 |
| motivating_3000 | 0.103 | - | oom | - | - | >600 | 48.5 | - | 85.4 | - | 3.85 | 4.15 | >600 | 118 | 117 | 7.14 | 16.6 |  | 17 | 40.4 | 24.6 | 25 | >600 | - | 0.486 | 1.32 | oom | exact | 5.6e-16 / 2.0e-15 |  |  |  |  |  | 2.2e-16 / 8.1e-16 |  | 3.9e-16 / 1.4e-15 |  | 3.9e-16 / 1.4e-15 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 5.6e-17 / 2.0e-16 | 5.6e-17 / 2.0e-16 |  | 2.7e-07 / 1.0e-06 | 2.7e-13 / 1.0e-12 | 2.7e-13 / 1.0e-12 | 2.7e-13 / 1.0e-12 |  |  | 0.0e+00 / 0.0e+00 |
| motivating_10000 | 0.401 | - | - | - | - | - | err | - | >600 | - | 39.6 | 41.5 | - | >600 | >600 | 17 | 48.2 |  | oom | oom | 246 | 268 | - | - | 1.74 | 2.49 | oom | exact | 5.6e-16 / 2.0e-15 |  |  |  |  |  |  |  |  |  | 3.9e-16 / 1.4e-15 | 0.0e+00 / 0.0e+00 |  |  |  | 5.6e-17 / 2.0e-16 | 5.6e-17 / 2.0e-16 |  |  |  | 2.7e-13 / 1.0e-12 | 2.7e-13 / 1.0e-12 |  |  | 0.0e+00 / 0.0e+00 |
| motivating_30000 | 1.18 | - | - | - | - | - | - | - | - | - | 338 | 356 | - | - | - | 36.7 | err |  | - | - | >600 | >600 | - | - | 5.49 | 6.24 | oom | exact | 5.6e-16 / 2.0e-15 |  |  |  |  |  |  |  |  |  | 3.9e-16 / 1.4e-15 | 0.0e+00 / 0.0e+00 |  |  |  | 5.6e-17 / 2.0e-16 |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |
| motivating_100000 | 4.22 | - | - | - | - | - | - | - | - | - | >600 | >600 | - | - | - | 127 | err |  | - | - | - | - | - | - | 22.4 | 23.2 | oom | exact | 5.6e-16 / 2.0e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.6e-17 / 2.0e-16 |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |
| motivating_300000 | 18.3 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 411 | err |  | - | - | - | - | - | - | 73.1 | 74.2 | oom | exact | 5.6e-16 / 2.0e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 5.6e-17 / 2.0e-16 |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |
| motivating_1000000 | 60.5 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | err | err |  | - | - | - | - | - | - | 287 | 288 | >600 | exact | 5.6e-16 / 2.0e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.0e+00 / 0.0e+00 |
| motivating_3000000 | >600 | - | - | - | - | - | oom | - | - | - | - | - | - | - | - | err | err |  | - | - | - | - | - | - | >600 | >600 | >600 | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| motivating_9000000 | - | - | - | - | - | - | err | - | - | - | - | - | - | - | - | err | err |  | - | - | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| network_10 | 0.008 | wrong | >600 | err | 1.9 | >600 | 8.5 | 1.66 | 3.15 | oom | 0.184 | 0.22 | 3.46 | 1.28 | 1.29 | 1.81 | 2.03 |  | 0.114 | 0.182 | 2.47 | 0.205 | 0.399 | 38.3 | 0.059 | 0.766 | err | exact | 1.2e-15 / 3.0e-14 | 3.7e-07 / 2.7e+01 |  |  | 1.1e-16 / 1.9e-15 |  | 3.3e-16 / 1.9e-15 | 3.3e-16 / 2.0e-08 | 3.3e-16 / 1.6e-15 |  | 5.6e-16 / 2.3e-15 | 2.2e-16 / 1.7e-15 | 5.6e-16 / 2.3e-15 | 5.6e-16 / 1.9e-15 | 5.6e-16 / 1.9e-15 | 6.9e-18 / 1.3e-15 | 3.3e-16 / 1.1e-15 |  | 3.8e-07 / 1.0e+00 | 2.3e-13 / 3.8e-12 | 2.3e-13 / 3.8e-12 | 2.3e-13 / 3.8e-12 | 3.3e-16 / 1.9e-15 | 3.3e-16 / 1.7e-15 | 4.4e-16 / 1.4e-15 |
| network_30 | 0.019 | wrong | - | err | 4.39 | - | 9.78 | 48.3 | 5.3 | - | 0.254 | 0.423 | 6.43 | 1.57 | 1.44 | 2.55 | 3.28 |  | 0.25 | 0.396 | err | 0.604 | 0.32 | 122 | 0.279 | 0.989 | - | exact | 3.2e-15 / 5.9e-15 | 2.4e-07 / 1.9e-01 |  |  | 4.4e-16 / 1.4e-15 |  | 2.2e-16 / 8.0e-16 | 3.6e-16 / 1.1e-10 | 1.1e-16 / 8.6e-16 |  | 1.1e-16 / 1.4e-15 | 3.3e-16 / 4.5e-16 | 1.1e-16 / 1.4e-15 | 1.1e-16 / 1.0e-15 | 1.1e-16 / 1.0e-15 | 3.3e-16 / 6.8e-16 | 1.3e-15 / 1.8e-15 |  | 4.5e-07 / 1.9e-01 | 3.0e-13 / 3.4e-12 |  | 3.0e-13 / 3.4e-12 | 1.4e-16 / 1.1e-15 | 2.2e-16 / 9.9e-16 | 2.2e-16 / 1.6e-15 |
| network_100 | 0.059 | 9.14 | - | err | 13 | - | 21.3 | >600 | 20.1 | - | 0.652 | 1.7 | 12.4 | 3.98 | 3.43 | 5.03 | 15.7 |  | 1.27 | 2.16 | - | 2.97 | 0.893 | 458 | 2.47 | 3.24 | - | exact | 1.7e-15 / 4.4e-15 | 6.0e-07 / 1.1e-03 |  |  | 1.7e-15 / 4.6e-15 |  | 1.4e-15 / 3.7e-15 |  | 6.7e-16 / 1.7e-15 |  | 7.2e-16 / 2.5e-15 | 8.9e-16 / 2.4e-15 | 7.2e-16 / 2.5e-15 | 5.6e-16 / 2.0e-15 | 5.6e-16 / 2.0e-15 | 1.8e-15 / 4.7e-15 | 1.4e-15 / 3.8e-15 |  | 4.9e-07 / 4.5e-03 | 2.7e-13 / 6.9e-13 |  | 2.7e-13 / 6.9e-13 | 8.9e-16 / 2.4e-15 | 8.9e-16 / 2.4e-15 | 7.2e-16 / 2.9e-15 |
| network_300 | 0.159 | 27.1 | - | err | 36.3 | - | err | - | 220 | - | 3.06 | 10.6 | 36.4 | 20 | 17 | 9.69 | 111 |  | 4.94 | 18.3 | - | 18.8 | >600 | >600 | 22.8 | 24 | - | exact | 6.4e-15 / 1.2e-14 | 4.1e-07 / 6.8e-05 |  |  | 2.0e-15 / 4.1e-15 |  |  |  | 2.6e-15 / 5.1e-15 |  | 1.9e-15 / 4.0e-15 | 4.2e-16 / 6.4e-15 | 1.9e-15 / 4.0e-15 | 1.9e-15 / 2.9e-15 | 1.9e-15 / 2.9e-15 | 2.3e-15 / 6.6e-15 | 2.7e-15 / 9.0e-15 |  | 3.8e-07 / 2.1e-04 | 2.5e-13 / 1.4e-12 |  | 2.5e-13 / 1.4e-12 |  |  | 2.4e-15 / 1.1e-14 |
| network_1000 | 0.559 | wrong | - | err | err | - | - | - | >600 | - | 30.1 | 112 | 296 | 182 | 182 | 24.6 | >600 |  | oom | 204 | - | 179 | - | - | oom | oom | - | exact | 3.0e-15 / 1.6e-14 | 1.4e-06 / 2.4e-03 |  |  |  |  |  |  |  |  | 3.2e-15 / 1.3e-14 | 5.3e-18 / 9.6e-15 | 3.2e-15 / 1.3e-14 | 2.4e-15 / 1.2e-14 | 2.4e-15 / 1.2e-14 | 1.3e-15 / 5.3e-15 |  |  |  | 1.7e-13 / 1.0e-12 |  | 1.7e-13 / 1.0e-12 |  |  |  |
| network_3000 | 1.71 | wrong | - | err | err | - | - | - | - | - | 383 | >600 | >600 | >600 | >600 | >600 | - |  | - | >600 | - | >600 | - | - | - | - | - | exact | 1.0e-14 / 1.0e-14 | 2.7e-06 / 8.1e+01 |  |  |  |  |  |  |  |  | 4.0e-15 / 3.5e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| network_10000 | 5.89 | err | - | err | err | - | - | - | - | - | >600 | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | exact | 2.2e-16 / 3.0e-14 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| network_30000 | 18.9 | - | - | err | err | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | exact | 4.6e-15 / 1.1e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| network_100000 | 129 | - | - | err | err | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | exact | 4.6e-15 / 5.9e-13 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| network_300000 | oom | - | - | err | err | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| network_1000000 | - | - | - | err | oom | - | - | - | - | - | - | - | - | - | - | - | - |  | - | - | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |

Seconds per system, the median over the repetitions; a failure is `>600`
(timeout), `oom`, `err` or `n/a`; each system's last columns are its
largest absolute and relative difference from the reference named in
`ref`.

### On Geni's programs

| family | instances | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Scallop | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Merlin | pyAgrum | pgmpy | libDAI | TensorInference.jl greedy | TensorInference.jl treesa | Roulette |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| geni_link | 10 | 8 | 6 of 8 | 6 of 8 | 0 of 2 | 1 of 3 | 1 of 3 | 3 of 4 | 2 of 4 | 6 of 8 | 0 of 2 | 6 of 8 | 6 of 8 | 6 of 8 | 7 of 9 | 7 of 9 | 5 of 6 | 5 of 6 | 5 of 6 | 5 of 6 | 6 of 7 | 5 of 6 | 5 of 8 | 8 of 9 | 5 of 6 |
| geni_router | 17 | 10 | 7 of 9 | 6 of 8 | 0 of 2 | 8 of 10 | 2 of 4 | 4 of 5 | 2 of 4 | 6 of 8 | 0 of 2 | 7 of 9 | 7 of 9 | 13 of 14 | 13 of 17 | 13 of 17 | 7 of 8 | 6 of 7 | 7 of 8 | 7 of 8 | 9 of 10 | 8 of 9 | 9 of 10 | 7 of 9 | 5 of 6 |
| geni_weather_h | 22 | 20 | 19 of 20 | 0 of 11 | 0 of 22 | 1 of 8 | 2 of 5 | 4 of 6 | 6 of 9 | 9 of 11 | 1 of 4 | 22 of 22 | 9 of 11 | 10 of 12 | 22 of 22 | 0 of 3 | 11 of 12 | 9 of 10 | 0 of 10 | 10 of 11 | 2 of 3 | 10 of 11 | 9 of 10 | 9 of 10 | 8 of 9 |
| geni_weather_n | 13 | 12 | 6 of 13 | 0 of 13 | 0 of 13 | 1 of 7 | 0 of 6 | 1 of 2 | 1 of 7 | 8 of 13 | 0 of 6 | 8 of 13 | 5 of 11 | 6 of 12 | 5 of 12 | 2 of 8 | 8 of 10 | 8 of 10 | 4 of 10 | 8 of 10 | 3 of 5 | 8 of 10 | 11 of 12 | 8 of 11 | 4 of 5 |

**Geni.** Geni ran 50 instances and answered 38 (8 time out, 4 print
zeros). Its answers differ from the reference's by at most 5.7e-01
relative (3.6e-08 absolute, weather_n11) over 38 instances. Geni's
answer is wrong, zero or underflowed on weather_n12, weather_n13,
weather_n15 and weather_n16. Over the 48 instances given to both that
either answers, a run that does not answer counted at the 600 s limit,
Geni takes 7803 s in all and tddlog 691 s; over the 38 both answer, 1803
s against tddlog's 216. tddlog alone answers link_16, router_18,
router_24, weather_h3000, weather_n12 and 5 more.

**noDice.** noDice ran 40 instances and answered 12 (16 fail, 2 run out
of memory, 10 time out). Its answers differ from the reference's by at
most 1.1e-06 relative (5.3e-07 absolute, link_8) over 12 instances. Over
the 38 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, noDice takes 16352 s in all and
tddlog 388 s; over the 12 both answer, 752 s against tddlog's 5.55.
tddlog alone answers link_16, router_16, router_24, weather_h1,
weather_h100 and 21 more.

**Psi.** Psi ran 39 instances and answered 0 (35 fail, 4 time out). Over
the 35 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Psi takes 21000 s in all and tddlog
627 s. tddlog alone answers link_4, router_24, router_4, weather_h1,
weather_h100 and 30 more.

**Genfer.** Genfer ran 28 instances and answered 11 (5 fail, 9 run out
of memory, 3 time out). Its answers differ from the reference's by at
most 2.9e-14 relative (1.5e-14 absolute, router_18) over 11 instances.
Over the 26 instances given to both that either answers, a run that does
not answer counted at the 600 s limit, Genfer takes 9390 s in all and
tddlog 465 s; over the 11 both answer, 390 s against tddlog's 6.03.
tddlog alone answers link_6, router_20, router_24, weather_h100,
weather_h1000 and 10 more.

**SPPL.** SPPL ran 18 instances and answered 5 (13 time out). Its
answers differ from the reference's by at most 8.2e-14 relative (7.7e-21
absolute, weather_h1) over 5 instances. Over the 16 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, SPPL takes 6793 s in all and tddlog 322 s; over the 5 both
answer, 193 s against tddlog's 0.047. tddlog alone answers link_6,
router_24, router_8, weather_h100, weather_h3 and 6 more.

**Pluck.** Pluck ran 17 instances and answered 12 (5 fail). Its answers
differ from the reference's by at most 6.9e-16 relative (1.3e-22
absolute, weather_h4) over 12 instances. Over the 17 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, Pluck takes 3105 s in all and tddlog 23.6 s; over the 12 both
answer, 105 s against tddlog's 0.29. tddlog alone answers link_10,
router_12, weather_h5, weather_h600 and weather_n7.

**PITA.** PITA ran 24 instances and answered 11 (13 time out). Its
answers differ from the reference's by at most 6.7e-10 relative (8.3e-17
absolute, weather_h2) over 11 instances. Over the 22 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, PITA takes 6794 s in all and tddlog 323 s; over the 11 both
answer, 194 s against tddlog's 0.43. tddlog alone answers link_8,
router_24, router_8, weather_h100, weather_h600 and 6 more.

**aspmc.** aspmc ran 40 instances and answered 29 (11 time out). Its
answers differ from the reference's by at most 1.9e-15 relative (3.1e-24
absolute, weather_n13) over 29 instances. Over the 38 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, aspmc takes 6916 s in all and tddlog 388 s; over the 29 both
answer, 1516 s against tddlog's 11.6. tddlog alone answers link_16,
router_16, router_24, weather_h100, weather_h600 and 4 more.

**Scallop.** Scallop ran 14 instances and answered 1 (11 run out of
memory, 2 time out). Its answers differ from the reference's by at most
5.7e-16 relative (5.3e-23 absolute, weather_h1) over 1 instance. Over
the 12 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Scallop takes 6600 s in all and
tddlog 322 s; over the 1 both answer, 0.019 s against tddlog's 0.004.
tddlog alone answers link_4, router_24, router_4, weather_h100,
weather_h2 and 6 more.

**Storm sparse.** Storm sparse ran 52 instances and answered 43 (9 time
out). Its answers differ from the reference's by at most 1.2e-11
relative (1.3e-14 absolute, weather_h30000) over 43 instances. Over the
49 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, Storm sparse takes 5286 s in all and
tddlog 1298 s; over the 41 both answer, 1024 s against tddlog's 337.
Storm sparse is faster on 10 (weather_h200 4.16 s against 7.05,
weather_h300 4.7 s against 11, weather_h400 5.26 s against 14.3 and
weather_h500 5.78 s against 18.2, the largest differences). Only Storm
sparse answers weather_h10000. tddlog alone answers link_16, router_18,
router_24, weather_n15, weather_n16 and 2 more.

**Storm exact.** Storm exact ran 39 instances and answered 27 (5 fail, 7
time out). Its answers differ from the reference's by at most 3.0e-14
relative (1.5e-14 absolute, router_16) over 27 instances. Over the 37
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Storm exact takes 7725 s in all and tddlog
395 s; over the 27 both answer, 1725 s against tddlog's 8.23. tddlog
alone answers link_16, router_18, router_24, weather_h100, weather_h600
and 5 more.

**Storm dd.** Storm dd ran 46 instances and answered 35 (5 fail, 6 time
out). Its answers differ from the reference's by at most 4.8e-11
relative (2.4e-11 absolute, router_24) over 32 instances. Over the 41
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Storm dd takes 6139 s in all and tddlog 1009
s; over the 32 both answer, 1275 s against tddlog's 112. Storm dd is
faster on 1 (router_24 38.5 s against 90.9). Only Storm dd answers
router_28. tddlog alone answers link_16, weather_h200, weather_h600,
weather_n12, weather_n15 and 3 more.

**PRISM sparse.** PRISM sparse ran 60 instances and answered 47 (5 fail,
4 run out of memory, 4 time out). Its answers differ from the
reference's by at most 9.0e-11 relative (9.3e-14 absolute,
weather_h30000) over 43 instances. Over the 51 instances given to both
that either answers, a run that does not answer counted at the 600 s
limit, PRISM sparse takes 6125 s in all and tddlog 2048 s; over the 41
both answer, 1241 s against tddlog's 487. PRISM sparse is faster on 8
(weather_h500 15.1 s against 18.2, weather_h600 16.5 s against 22.9,
weather_h700 16.7 s against 26.6 and weather_h800 17.1 s against 30.6,
the largest differences). Only PRISM sparse answers router_28 and
weather_h10000. tddlog alone answers link_18, router_20, weather_n11,
weather_n12, weather_n15 and 3 more.

**PRISM hybrid.** PRISM hybrid ran 37 instances and answered 22 (4 fail,
3 run out of memory, 8 time out). Its answers differ from the
reference's by at most 4.8e-11 relative (2.4e-11 absolute, router_24)
over 18 instances. Over the 29 instances given to both that either
answers, a run that does not answer counted at the 600 s limit, PRISM
hybrid takes 6915 s in all and tddlog 1147 s; over the 18 both answer,
901 s against tddlog's 161. PRISM hybrid is faster on 1 (router_24 8.47
s against 90.9). Only PRISM hybrid answers router_28. tddlog alone
answers link_18, router_20, weather_h1, weather_h100, weather_h600 and 5
more.

**Ace.** Ace ran 36 instances and answered 31 (5 time out). Its answers
differ from the reference's by at most 3.0e-14 relative (1.5e-14
absolute, router_16) over 31 instances. Over the 36 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, Ace takes 3305 s in all and tddlog 138 s; over the 31 both
answer, 305 s against tddlog's 19.3. tddlog alone answers link_14,
router_18, weather_h300, weather_n15 and weather_n17.

**Ace c2d.** Ace c2d ran 33 instances and answered 28 (5 time out). Its
answers differ from the reference's by at most 2.8e-15 relative (2.8e-23
absolute, weather_n12) over 28 instances. Over the 33 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, Ace c2d takes 4391 s in all and tddlog 115 s; over the 28 both
answer, 1391 s against tddlog's 8.39. tddlog alone answers link_14,
router_16, weather_h100, weather_n15 and weather_n17.

**Merlin.** Merlin ran 34 instances and answered 16 (5 run out of
memory, 13 print zeros). Its answers differ from the reference's by at
most 1.9e-01 relative (4.8e-07 absolute, weather_n9) over 16 instances.
Merlin's answer is wrong, zero or underflowed on weather_h1, weather_h2,
weather_h3, weather_h4, weather_h5 and 8 more. Over the 34 instances
given to both that either answers, a run that does not answer counted at
the 600 s limit, Merlin takes 10837 s in all and tddlog 120 s; over the
16 both answer, 37.2 s against tddlog's 3.85. tddlog alone answers
link_14, router_18, weather_h1, weather_h100, weather_h2 and 13 more.

**pyAgrum.** pyAgrum ran 35 instances and answered 30 (4 run out of
memory, 1 time out). Its answers differ from the reference's by at most
3.9e-12 relative (4.8e-19 absolute, weather_h2) over 30 instances. Over
the 35 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, pyAgrum takes 3203 s in all and
tddlog 127 s; over the 30 both answer, 203 s against tddlog's 12.3.
tddlog alone answers link_14, router_18, weather_h200, weather_n15 and
weather_n17.

**pgmpy.** pgmpy ran 25 instances and answered 20 (2 run out of memory,
3 time out). Its answers differ from the reference's by at most 3.9e-12
relative (4.8e-19 absolute, weather_h2) over 20 instances. Over the 25
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, pgmpy takes 4081 s in all and tddlog 250 s;
over the 20 both answer, 1081 s against tddlog's 16.9. pgmpy is faster
on 1 (router_18 3.63 s against 4.26). tddlog alone answers link_16,
router_24, weather_h3, weather_n17 and weather_n9.

**libDAI.** libDAI ran 36 instances and answered 31 (4 run out of
memory, 1 time out). Its answers differ from the reference's by at most
3.9e-12 relative (4.8e-19 absolute, weather_h2) over 31 instances. Over
the 36 instances given to both that either answers, a run that does not
answer counted at the 600 s limit, libDAI takes 3467 s in all and tddlog
132 s; over the 31 both answer, 467 s against tddlog's 16.5. tddlog
alone answers link_14, router_20, weather_h200, weather_n15 and
weather_n17.

**TensorInference.jl greedy.** TensorInference.jl greedy ran 40
instances and answered 34 (2 fail, 2 run out of memory, 2 time out). Its
answers differ from the reference's by at most 1.2e-13 relative (5.9e-14
absolute, router_20) over 34 instances. Over the 40 instances given to
both that either answers, a run that does not answer counted at the 600
s limit, TensorInference.jl greedy takes 3856 s in all and tddlog 528 s;
over the 34 both answer, 256 s against tddlog's 141. TensorInference.jl
greedy is faster on 5 (router_16 0.92 s against 1.13, weather_n13 1.86 s
against 3.4, router_18 2.31 s against 4.26 and weather_n16 15.4 s
against 21.8, the largest differences). tddlog alone answers link_14,
link_16, link_18, router_24, weather_h100 and 1 more.

**TensorInference.jl treesa.** TensorInference.jl treesa ran 39
instances and answered 32 (7 time out). Its answers differ from the
reference's by at most 2.9e-14 relative (1.5e-14 absolute, router_16)
over 32 instances. Over the 38 instances given to both that either
answers, a run that does not answer counted at the 600 s limit,
TensorInference.jl treesa takes 9641 s in all and tddlog 354 s; over the
32 both answer, 6041 s against tddlog's 220. tddlog alone answers
router_18, router_20, weather_h100, weather_n15, weather_n16 and 1 more.

**Roulette.** Roulette ran 26 instances and answered 22 (4 run out of
memory). Its answers differ from the reference's by at most 1.8e-15
relative (8.3e-16 absolute, link_10) over 22 instances. Over the 26
instances given to both that either answers, a run that does not answer
counted at the 600 s limit, Roulette takes 2805 s in all and tddlog 13.5
s; over the 22 both answer, 405 s against tddlog's 3.84. tddlog alone
answers link_14, router_14, weather_h9 and weather_n10.

| instance | tddlog | Geni | noDice | Psi | Genfer | SPPL | Pluck | PITA | aspmc | Scallop | Storm sparse | Storm exact | Storm dd | PRISM sparse | PRISM hybrid | Ace | Ace c2d | Merlin | pyAgrum | pgmpy | libDAI | TI greedy | TI treesa | Roulette | Roulette process | ProbLog | ref | tddlog diff | Geni diff | noDice diff | Psi diff | Genfer diff | SPPL diff | Pluck diff | PITA diff | aspmc diff | Scallop diff | Storm sparse diff | Storm exact diff | Storm dd diff | PRISM sparse diff | PRISM hybrid diff | Ace diff | Ace c2d diff | Merlin diff | pyAgrum diff | pgmpy diff | libDAI diff | TI greedy diff | TI treesa diff | Roulette diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| link_4 | 0.008 | 0.024 | 0.048 | >600 | 5.61 | 12.2 | 3.96 | 0.435 | 2.59 | oom | 0.155 | 0.155 | 0.462 | 1.27 | 1.27 | 1.3 | 1.33 | 0.237 | 1.45 | 19 | 0.237 | 0.0559 | 6.69 | 0.005 | 0.716 | 0.186 | exact | 0.0e+00 / 0.0e+00 | 6.9e-09 / 1.2e-08 | 9.3e-08 / 1.6e-07 |  | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.0e-16 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |  | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 9.3e-08 / 1.6e-07 | 4.3e-13 / 7.5e-13 | 4.3e-13 / 7.5e-13 | 4.3e-13 / 7.5e-13 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 |
| link_6 | 0.012 | 0.062 | 0.105 | - | oom | >600 | 5.16 | 0.615 | 2.63 | - | 0.187 | 0.205 | 1.8 | 1.28 | 1.27 | 1.45 | 1.6 | 0.066 | 0.226 | 1.32 | 0.07 | 0.109 | 22 | 0.032 | 0.747 | 8.67 | exact | 1.1e-16 / 2.2e-16 | 1.3e-08 / 2.5e-08 | 3.9e-07 / 7.6e-07 |  |  |  | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 |  | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 3.9e-07 / 7.6e-07 | 2.2e-13 / 4.3e-13 | 2.2e-13 / 4.3e-13 | 2.2e-13 / 4.3e-13 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 |
| link_8 | 0.025 | 0.358 | 0.779 | - | - | - | 13 | >600 | 2.97 | - | 0.537 | 0.729 | 6.13 | 1.26 | 1.26 | 2.2 | 3.33 | 0.093 | 0.187 | 1.46 | 0.103 | 0.183 | 46.5 | 0.494 | 1.2 | >600 | exact | 0.0e+00 / 0.0e+00 | 3.0e-08 / 6.3e-08 | 5.3e-07 / 1.1e-06 |  |  |  | 5.6e-17 / 1.2e-16 |  | 1.7e-16 / 3.5e-16 |  | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 5.6e-17 / 1.2e-16 | 0.0e+00 / 0.0e+00 | 1.7e-16 / 3.5e-16 | 4.7e-07 / 9.9e-07 | 1.7e-13 / 3.6e-13 | 1.7e-13 / 3.6e-13 | 1.7e-13 / 3.6e-13 | 5.6e-17 / 1.2e-16 | 1.1e-16 / 2.3e-16 | 5.6e-17 / 1.2e-16 |
| link_10 | 0.183 | 2.69 | 7.31 | - | - | - | err | - | 4.87 | - | 4.03 | 5.69 | 12.3 | 2.15 | 2.12 | 4.5 | 29.9 | 0.396 | 0.427 | 24 | 0.413 | 0.766 | 86.2 | 5.67 | 6.49 | - | exact | 8.3e-16 / 1.8e-15 | 4.9e-09 / 1.1e-08 | 3.0e-07 / 6.7e-07 |  |  |  |  |  | 8.3e-16 / 1.8e-15 |  | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 | 3.0e-07 / 6.7e-07 | 2.3e-14 / 5.1e-14 | 2.3e-14 / 5.1e-14 | 2.3e-14 / 5.1e-14 | 8.9e-16 / 1.9e-15 | 8.3e-16 / 1.8e-15 | 8.3e-16 / 1.8e-15 |
| link_12 | 1.47 | 18.7 | 65 | - | - | - | - | - | 33.1 | - | 33.6 | 45.5 | 35.1 | 9.35 | 8.93 | 19.1 | 271 | 8.04 | 2.04 | 7.91 | 7.27 | 36.6 | 148 | 48.7 | 50.2 | - | exact | 7.2e-16 / 1.6e-15 | 3.1e-08 / 7.1e-08 | 1.7e-07 / 3.8e-07 |  |  |  |  |  | 5.6e-16 / 1.3e-15 |  | 6.1e-16 / 1.4e-15 | 5.6e-16 / 1.3e-15 | 6.1e-16 / 1.4e-15 | 6.1e-16 / 1.4e-15 | 6.1e-16 / 1.4e-15 | 5.0e-16 / 1.1e-15 | 6.1e-16 / 1.4e-15 | 1.7e-07 / 3.8e-07 | 8.3e-15 / 1.9e-14 | 8.3e-15 / 1.9e-14 | 8.3e-15 / 1.9e-14 | 5.6e-16 / 1.3e-15 | 7.2e-16 / 1.6e-15 | 6.1e-16 / 1.4e-15 |
| link_14 | 3.27 | 122 | 468 | - | - | - | - | - | 364 | - | 253 | 324 | 149 | 63.1 | 73.7 | >600 | >600 | oom | oom | 275 | oom | err | 206 | oom | oom | - | exact | 8.9e-16 / 2.1e-15 | 2.3e-08 / 5.5e-08 | 7.7e-08 / 1.8e-07 |  |  |  |  |  | 7.2e-16 / 1.7e-15 |  | 7.2e-16 / 1.7e-15 | 7.8e-16 / 1.8e-15 | 7.2e-16 / 1.7e-15 | 7.2e-16 / 1.7e-15 | 7.2e-16 / 1.7e-15 |  |  |  |  | 2.3e-13 / 5.3e-13 |  |  | 7.8e-16 / 1.8e-15 |  |
| link_16 | 53 | >600 | >600 | - | - | - | - | - | >600 | - | >600 | >600 | >600 | 439 | 530 | - | - | - | - | >600 | - | err | 299 | - | - | - | exact | 6.2e-15 / 1.5e-14 |  |  |  |  |  |  |  |  |  |  |  |  | 6.3e-15 / 1.5e-14 | 6.3e-15 / 1.5e-14 |  |  |  |  |  |  |  | 6.3e-15 / 1.5e-14 |  |
| link_18 | 148 | - | - | - | - | - | - | - | - | - | - | - | - | >600 | >600 | - | - | - | - | - | - | oom | 448 | - | - | - | exact | 1.7e-15 / 4.0e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.8e-15 / 4.3e-15 |  |
| link_20 | oom | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | >600 | - | - | - | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| link_24 | oom | >600 | oom | >600 | oom | >600 | - | >600 | >600 | >600 | >600 | >600 | >600 | >600 | >600 | - | - | - | - | - | - | - | - | - | - | >600 | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_4 | 0.007 | 0.017 | 0.042 | >600 | 0.006 | 1.68 | 3.79 | 0.397 | 2.59 | oom | 0.157 | 0.156 | 0.355 | 1.26 | 1.28 | 1.23 | 1.32 | 0.044 | 0.132 | 17.3 | 0.045 | 0.0687 | 12.8 | 0.006 | 0.71 | 0.163 | exact | 1.1e-16 / 2.2e-16 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 |  | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.1e-16 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.2e-16 |
| router_6 | 0.012 | 0.034 | 0.074 | - | 0.015 | 44.6 | 5.26 | 1.4 | 2.7 | - | 0.169 | 0.169 | 0.893 | 1.27 | 1.28 | 1.61 | 1.7 | 0.071 | 0.268 | 1.39 | 0.081 | 0.148 | 35 | 0.023 | 0.725 | 16.5 | exact | 0.0e+00 / 0.0e+00 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 4.4e-16 / 8.9e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.2e-16 |  | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 0.0e+00 / 0.0e+00 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 |
| router_8 | 0.027 | 0.098 | 0.486 | - | 0.064 | >600 | 6.35 | >600 | 3 | - | 0.254 | 0.302 | 1.73 | 1.27 | 1.27 | 1.79 | 2.19 | 0.106 | 0.254 | 1.5 | 0.144 | 0.244 | 74.3 | 0.197 | 0.901 | >600 | exact | 0.0e+00 / 0.0e+00 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 8.9e-16 / 1.8e-15 |  | 1.1e-16 / 2.2e-16 |  | 0.0e+00 / 0.0e+00 |  | 4.4e-16 / 8.9e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 0.0e+00 / 0.0e+00 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 0.0e+00 / 0.0e+00 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 |
| router_10 | 0.064 | 0.546 | 3.71 | - | 0.34 | - | 17.6 | - | 5.14 | - | 0.955 | 1.35 | 3.33 | 1.31 | 1.29 | 2.4 | 4.26 | 0.169 | 0.376 | 1.52 | 0.276 | 0.364 | 130 | 2.09 | 2.85 | - | exact | 2.2e-16 / 4.4e-16 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 1.2e-15 / 2.4e-15 |  | 3.3e-16 / 6.7e-16 |  | 3.3e-16 / 6.7e-16 |  | 2.2e-16 / 4.4e-16 | 3.3e-16 / 6.7e-16 | 4.4e-16 / 8.9e-16 | 1.1e-16 / 2.2e-16 | 1.1e-16 / 2.2e-16 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 2.2e-16 / 4.4e-16 | 3.3e-16 / 6.7e-16 | 3.3e-16 / 6.7e-16 |
| router_12 | 0.101 | 3.65 | 26.1 | - | 1.99 | - | err | - | 32.2 | - | 6.63 | 9.31 | 5.57 | 2.04 | 1.94 | 4.06 | 23.4 | 0.345 | 0.719 | 1.76 | 0.573 | 0.617 | 203 | 17 | 17.9 | - | exact | 6.7e-16 / 1.3e-15 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 2.4e-15 / 4.9e-15 |  |  |  | 4.4e-16 / 8.9e-16 |  | 8.9e-16 / 1.8e-15 | 3.3e-16 / 6.7e-16 | 2.2e-16 / 4.4e-16 | 5.6e-16 / 1.1e-15 | 5.6e-16 / 1.1e-15 | 4.4e-16 / 8.9e-16 | 4.4e-16 / 8.9e-16 | 4.5e-07 / 9.0e-07 | 1.3e-13 / 2.7e-13 | 1.3e-13 / 2.7e-13 | 1.3e-13 / 2.7e-13 | 3.3e-16 / 6.7e-16 | 4.4e-16 / 8.9e-16 | 3.3e-16 / 6.7e-16 |
| router_14 | 0.383 | 23.6 | 180 | - | 11.3 | - | - | - | 536 | - | 48.4 | 65 | 8 | 5.62 | 5.55 | 9.64 | 223 | 0.937 | 1.22 | 2.08 | 1.64 | 0.57 | 296 | oom | oom | - | exact | 5.6e-16 / 1.1e-15 | 5.0e-08 / 1.0e-07 | 4.5e-07 / 9.0e-07 |  | 2.3e-15 / 4.7e-15 |  |  |  | 3.3e-16 / 6.7e-16 |  | 3.3e-16 / 6.7e-16 | 4.4e-16 / 8.9e-16 | 5.6e-16 / 1.1e-15 | 2.2e-16 / 4.4e-16 | 2.2e-16 / 4.4e-16 | 4.4e-16 / 8.9e-16 | 5.6e-16 / 1.1e-15 | 4.5e-07 / 9.0e-07 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 1.4e-13 / 2.7e-13 | 3.3e-16 / 6.7e-16 | 4.4e-16 / 8.9e-16 |  |
| router_16 | 1.14 | 142 | >600 | - | 61.6 | - | - | - | >600 | - | 333 | 427 | 10.6 | 28.1 | 32.6 | 88.6 | >600 | 24.9 | 6.52 | 2.71 | 9.62 | 0.916 | 416 | - | - | - | exact | 1.4e-14 / 2.9e-14 | 5.0e-08 / 1.0e-07 |  |  | 1.1e-14 / 2.2e-14 |  |  |  |  |  | 1.4e-14 / 2.8e-14 | 1.5e-14 / 3.0e-14 | 1.5e-14 / 3.0e-14 | 1.5e-14 / 2.9e-14 | 1.5e-14 / 2.9e-14 | 1.5e-14 / 3.0e-14 |  | 4.5e-07 / 9.0e-07 | 1.5e-13 / 3.0e-13 | 1.5e-13 / 3.0e-13 | 1.5e-13 / 3.0e-13 | 1.5e-14 / 3.0e-14 | 1.5e-14 / 2.9e-14 |  |
| router_18 | 4.26 | >600 | - | - | 309 | - | - | - | - | - | >600 | >600 | 15.7 | 169 | 207 | >600 | - | oom | oom | 3.63 | 57.6 | 2.31 | >600 | - | - | - | exact | 1.9e-14 / 3.7e-14 |  |  |  | 1.5e-14 / 2.9e-14 |  |  |  |  |  |  |  | 1.9e-14 / 3.8e-14 | 1.9e-14 / 3.8e-14 | 1.9e-14 / 3.8e-14 |  |  |  |  | 1.5e-13 / 3.1e-13 | 1.5e-13 / 3.1e-13 | 1.9e-14 / 3.8e-14 |  |  |
| router_20 | 5.66 | - | - | - | >600 | - | - | - | - | - | - | - | 20.6 | >600 | >600 | - | - | - | - | 5.97 | oom | 11.2 | >600 | - | - | - | exact | 5.9e-14 / 1.2e-13 |  |  |  |  |  |  |  |  |  |  |  | 5.8e-14 / 1.2e-13 |  |  |  |  |  |  | 7.6e-14 / 1.5e-13 |  | 5.9e-14 / 1.2e-13 |  |  |
| router_24 | 89.7 | >600 | oom | >600 | >600 | >600 | - | >600 | >600 | oom | >600 | >600 | 38.5 | 7.85 | 8.47 | - | - | - | - | >600 | - | oom | - | - | - | >600 | exact | 2.4e-11 / 4.8e-11 |  |  |  |  |  |  |  |  |  |  |  | 2.4e-11 / 4.8e-11 | 2.4e-11 / 4.8e-11 | 2.4e-11 / 4.8e-11 |  |  |  |  |  |  |  |  |  |
| router_28 | oom | - | - | - | - | - | - | - | - | - | - | - | 64.3 | 13.7 | 13.8 | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_32 | - | - | - | - | - | - | - | - | - | - | - | - | 104 | 22.8 | 23.1 | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_40 | - | - | - | - | - | - | - | - | - | - | - | - | 266 | 61.3 | 61.3 | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_48 | - | - | - | - | - | - | - | - | - | - | - | - | >600 | 223 | 150 | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_56 | - | - | - | - | - | - | - | - | - | - | - | - | - | err | err | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_64 | - | - | - | - | - | - | - | - | - | - | - | - | - | err | err | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| router_72 | - | - | - | - | - | - | - | - | - | - | - | - | - | >600 | >600 | - | - | - | - | - | - | - | - | - | - | - |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| weather_h1 | 0.004 | 0.049 | err | err | 0.006 | 0.602 | 2.49 | 0.338 | 1.56 | 0.019 | 2.64 | 12.7 | 20.4 | 10.1 | >600 | 1.1 | 1.14 | zero | 0.315 | 1.52 | 0.229 | 0.046 | 1.26 | 0.004 | 0.713 | 0.162 | exact | 2.6e-23 / 2.8e-16 | 6.4e-09 / 6.9e-02 |  |  | 0.0e+00 / 0.0e+00 | 7.7e-21 / 8.2e-14 | 2.6e-23 / 2.8e-16 | 1.2e-17 / 1.3e-10 | 2.6e-23 / 2.8e-16 | 5.3e-23 / 5.7e-16 | 2.6e-23 / 2.8e-16 | 1.3e-23 / 1.4e-16 | 1.3e-23 / 1.4e-16 | 5.3e-23 / 5.7e-16 |  | 2.6e-23 / 2.8e-16 | 1.3e-23 / 1.4e-16 |  | 4.1e-20 / 4.4e-13 | 4.1e-20 / 4.4e-13 | 4.1e-20 / 4.4e-13 | 5.3e-23 / 5.7e-16 | 9.3e-23 / 9.9e-16 | 0.0e+00 / 0.0e+00 |
| weather_h2 | 0.016 | 0.19 | err | err | oom | 134 | 5.21 | 0.413 | 2.68 | oom | 2.62 | 19 | 23.3 | 10.5 | - | 1.44 | 1.56 | zero | 0.3 | 1.82 | 0.127 | 0.124 | 28.5 | 0.044 | 0.748 | 1.06 | exact | 2.6e-23 / 2.1e-16 | 2.4e-08 / 1.9e-01 |  |  |  | 5.2e-21 / 4.2e-14 | 2.6e-23 / 2.1e-16 | 8.3e-17 / 6.7e-10 | 2.6e-23 / 2.1e-16 |  | 1.3e-22 / 1.1e-15 | 2.6e-23 / 2.1e-16 | 0.0e+00 / 0.0e+00 | 4.4e-18 / 3.6e-11 |  | 0.0e+00 / 0.0e+00 | 5.3e-23 / 4.3e-16 |  | 4.8e-19 / 3.9e-12 | 4.8e-19 / 3.9e-12 | 4.8e-19 / 3.9e-12 | 7.9e-23 / 6.4e-16 | 7.9e-23 / 6.4e-16 | 2.6e-23 / 2.1e-16 |
| weather_h3 | 0.036 | 0.417 | err | err | - | >600 | 6.54 | 0.622 | 2.8 | - | 2.64 | 29.1 | 25.4 | 10.6 | - | 1.69 | 1.83 | zero | 0.214 | oom | 0.18 | 0.211 | 59.2 | 0.572 | 1.29 | 3.74 | exact | 5.3e-23 / 3.3e-16 | 4.2e-08 / 2.6e-01 |  |  |  |  | 7.9e-23 / 5.0e-16 | 9.9e-17 / 6.2e-10 | 1.1e-22 / 6.7e-16 |  | 3.7e-22 / 2.3e-15 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 7.4e-18 / 4.7e-11 |  | 7.9e-23 / 5.0e-16 | 2.6e-23 / 1.7e-16 |  | 1.6e-19 / 1.0e-12 |  | 1.6e-19 / 1.0e-12 | 7.9e-23 / 5.0e-16 | 1.1e-22 / 6.7e-16 | 2.6e-23 / 1.7e-16 |
| weather_h4 | 0.049 | 0.637 | err | err | - | - | 29.9 | 3.21 | 2.98 | - | 2.65 | 41.5 | 26.6 | 10.1 | - | 2.14 | 4.46 | zero | 0.263 | - | 0.324 | 0.312 | 95.8 | 4.25 | 5.04 | 21.6 | exact | 5.3e-23 / 2.7e-16 | 7.3e-09 / 3.8e-02 |  |  |  |  | 1.3e-22 / 6.9e-16 | 6.2e-17 / 3.2e-10 | 1.3e-22 / 6.9e-16 |  | 4.0e-22 / 2.1e-15 | 0.0e+00 / 0.0e+00 | 2.6e-23 / 1.4e-16 | 1.0e-17 / 5.4e-11 |  | 7.9e-23 / 4.1e-16 | 5.3e-23 / 2.7e-16 |  | 7.6e-20 / 3.9e-13 |  | 7.6e-20 / 3.9e-13 | 1.1e-22 / 5.5e-16 | 1.1e-22 / 5.5e-16 | 1.1e-22 / 5.5e-16 |
| weather_h5 | 0.118 | 0.846 | err | err | - | - | err | 21.3 | 3.96 | - | 2.63 | 57.3 | 28.4 | 9.87 | - | 2.38 | 8.02 | zero | 0.352 | - | 0.54 | 0.375 | 138 | 15.8 | 16.8 | 67.4 | exact | 1.9e-22 / 8.2e-16 | 2.7e-08 / 1.2e-01 |  |  |  |  |  | 1.2e-16 / 5.2e-10 | 1.1e-22 / 4.7e-16 |  | 3.4e-22 / 1.5e-15 | 0.0e+00 / 0.0e+00 | 5.3e-23 / 2.3e-16 | 1.3e-17 / 5.9e-11 |  | 1.3e-22 / 5.8e-16 | 5.3e-23 / 2.3e-16 |  | 1.2e-19 / 5.3e-13 |  | 1.2e-19 / 5.3e-13 | 1.1e-22 / 4.7e-16 | 1.3e-22 / 5.8e-16 | 7.9e-23 / 3.5e-16 |
| weather_h6 | 0.144 | 1.08 | err | err | - | - | - | 165 | 4.83 | - | 2.64 | 75.8 | 30.4 | 9.93 | - | 2.6 | 10.3 | zero | 0.468 | - | 0.803 | 0.507 | 170 | 36.4 | 37.7 | oom | exact | 2.1e-22 / 8.1e-16 | 3.8e-08 / 1.5e-01 |  |  |  |  |  | 1.7e-16 / 6.6e-10 | 1.6e-22 / 6.1e-16 |  | 6.4e-22 / 2.4e-15 | 5.3e-23 / 2.0e-16 | 5.3e-23 / 2.0e-16 | 1.6e-17 / 6.3e-11 |  | 1.1e-22 / 4.0e-16 | 2.1e-22 / 8.1e-16 |  | 5.0e-19 / 1.9e-12 |  | 5.0e-19 / 1.9e-12 | 0.0e+00 / 0.0e+00 | 2.1e-22 / 8.1e-16 | 1.1e-22 / 4.0e-16 |
| weather_h7 | 0.161 | 1.28 | err | err | - | - | - | >600 | 7.06 | - | 2.65 | 97.1 | 32.5 | 9.94 | - | 3.26 | 19 | zero | 0.613 | - | 1.26 | 0.638 | 206 | 58.8 | 60.3 | - | exact | 3.7e-22 / 1.3e-15 | 4.0e-09 / 1.3e-02 |  |  |  |  |  |  | 3.2e-22 / 1.1e-15 |  | 3.7e-22 / 1.3e-15 | 5.3e-23 / 1.8e-16 | 1.1e-22 / 3.6e-16 | 2.0e-17 / 6.6e-11 |  | 1.6e-22 / 5.4e-16 | 3.7e-22 / 1.3e-15 |  | 3.8e-19 / 1.3e-12 |  | 3.8e-19 / 1.3e-12 | 1.1e-22 / 3.6e-16 | 3.2e-22 / 1.1e-15 | 2.1e-22 / 7.2e-16 |
| weather_h8 | 0.196 | 1.53 | err | err | - | - | - | - | 8.3 | - | 2.65 | 121 | 34.7 | 9.88 | - | 3.46 | 22.2 | zero | 0.749 | - | 1.66 | 1.11 | 251 | 102 | 104 | - | exact | 3.7e-22 / 1.1e-15 | 3.0e-08 / 9.2e-02 |  |  |  |  |  |  | 2.6e-22 / 8.0e-16 |  | 3.7e-22 / 1.1e-15 | 5.3e-23 / 1.6e-16 | 1.6e-22 / 4.8e-16 | 2.3e-17 / 6.8e-11 |  | 2.6e-22 / 8.0e-16 | 1.6e-22 / 4.8e-16 |  | 2.3e-19 / 7.1e-13 |  | 2.3e-19 / 7.1e-13 | 5.3e-23 / 1.6e-16 | 3.7e-22 / 1.1e-15 | 1.1e-22 / 3.2e-16 |
| weather_h9 | 0.183 | 1.74 | err | err | - | - | - | - | 16.8 | - | 2.66 | 147 | 37.4 | 10.9 | - | 3.89 | 30.7 | zero | 0.997 | - | 2.14 | 0.634 | 286 | oom | oom | - | exact | 2.6e-22 / 7.3e-16 | 3.5e-08 / 9.6e-02 |  |  |  |  |  |  | 1.6e-22 / 4.4e-16 |  | 1.3e-21 / 3.5e-15 | 5.3e-23 / 1.5e-16 | 5.3e-23 / 1.5e-16 | 2.6e-17 / 7.0e-11 |  | 2.1e-22 / 5.8e-16 | 3.7e-22 / 1.0e-15 |  | 1.2e-19 / 3.4e-13 |  | 1.2e-19 / 3.4e-13 | 2.1e-22 / 5.8e-16 | 4.8e-22 / 1.3e-15 |  |
| weather_h100 | 2.77 | 25.9 | >600 | err | oom | >600 | - | >600 | >600 | oom | 3.53 | >600 | 413 | 13.1 | >600 | 30.2 | >600 | oom | 169 | - | 344 | >600 | >600 | - | - | >600 | exact | 4.1e-20 / 1.2e-14 | 1.0e-09 / 2.9e-04 |  |  |  |  |  |  |  |  | 1.4e-19 / 3.9e-14 |  | 1.7e-20 / 4.7e-15 | 3.0e-16 / 8.6e-11 |  | 2.1e-21 / 6.1e-16 |  |  | 2.1e-18 / 6.0e-13 |  | 2.1e-18 / 6.0e-13 |  |  |  |
| weather_h200 | 7.05 | 53.4 | - | err | - | - | - | - | - | - | 4.16 | - | >600 | 13.9 | - | 69.6 | - | - | >600 | - | >600 | - | - | - | - | - | exact | 1.2e-19 / 1.8e-14 | 4.3e-08 / 6.2e-03 |  |  |  |  |  |  |  |  | 5.5e-19 / 8.0e-14 |  |  | 6.0e-16 / 8.7e-11 |  | 7.6e-21 / 1.1e-15 |  |  |  |  |  |  |  |  |
| weather_h300 | 11 | 79.6 | - | err | - | - | - | - | - | - | 4.7 | - | - | 14.3 | - | >600 | - | - | - | - | - | - | - | - | - | - | exact | 2.7e-19 / 2.6e-14 | 1.3e-08 / 1.3e-03 |  |  |  |  |  |  |  |  | 1.3e-18 / 1.2e-13 |  |  | 9.0e-16 / 8.7e-11 |  |  |  |  |  |  |  |  |  |  |
| weather_h400 | 14.3 | 107 | - | err | - | - | - | - | - | - | 5.26 | - | - | 14.5 | - | - | - | - | - | - | - | - | - | - | - | - | exact | 4.7e-19 / 3.4e-14 | 3.1e-08 / 2.2e-03 |  |  |  |  |  |  |  |  | 2.2e-18 / 1.6e-13 |  |  | 1.2e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |  |
| weather_h500 | 18.2 | 133 | - | err | - | - | - | - | - | - | 5.78 | - | - | 15.1 | - | - | - | - | - | - | - | - | - | - | - | - | exact | 7.2e-19 / 4.2e-14 | 2.5e-08 / 1.4e-03 |  |  |  |  |  |  |  |  | 3.5e-18 / 2.0e-13 |  |  | 1.5e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |  |
| weather_h600 | 22.9 | 163 | >600 | err | err | >600 | err | >600 | >600 | >600 | 6.59 | >600 | >600 | 16.5 | >600 | - | - | - | - | - | - | - | - | - | - | oom | exact | 1.1e-18 / 5.2e-14 | 1.9e-08 / 9.1e-04 |  |  |  |  |  |  |  |  | 5.0e-18 / 2.4e-13 |  |  | 1.8e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |  |
| weather_h700 | 26.6 | 186 | - | err | err | - | - | - | - | - | 7.05 | - | - | 16.7 | - | - | - | - | - | - | - | - | - | - | - | - | exact | 1.4e-18 / 5.6e-14 | 3.7e-08 / 1.5e-03 |  |  |  |  |  |  |  |  | 6.8e-18 / 2.8e-13 |  |  | 2.1e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |  |
| weather_h800 | 30.6 | 216 | - | err | err | - | - | - | - | - | 7.52 | - | - | 17.1 | - | - | - | - | - | - | - | - | - | - | - | - | exact | 1.8e-18 / 6.7e-14 | 6.8e-09 / 2.5e-04 |  |  |  |  |  |  |  |  | 9.0e-18 / 3.3e-13 |  |  | 2.4e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |  |
| weather_h900 | 34.7 | 240 | - | err | err | - | - | - | - | - | 8.36 | - | - | 18.5 | - | - | - | - | - | - | - | - | - | - | - | - | exact | 2.3e-18 / 7.4e-14 | 4.9e-08 / 1.6e-03 |  |  |  |  |  |  |  |  | 1.1e-17 / 3.7e-13 |  |  | 2.7e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |  |
| weather_h1000 | 39.8 | 271 | - | err | err | - | - | - | - | - | 8.82 | - | - | 18.9 | - | - | - | - | - | - | - | - | - | - | - | - | exact | 2.7e-18 / 8.0e-14 | 5.3e-09 / 1.5e-04 |  |  |  |  |  |  |  |  | 1.4e-17 / 4.1e-13 |  |  | 3.0e-15 / 8.7e-11 |  |  |  |  |  |  |  |  |  |  |
| weather_h3000 | 116 | >600 | - | err | - | - | - | - | - | - | 21 | - | - | 29.5 | - | - | - | - | - | - | - | - | - | - | - | - | exact | 2.1e-17 / 2.1e-13 |  |  |  |  |  |  |  |  |  | 1.3e-16 / 1.2e-12 |  |  | 9.1e-15 / 8.8e-11 |  |  |  |  |  |  |  |  |  |  |
| weather_h10000 | oom | - | - | err | - | - | - | - | - | - | 61.5 | - | - | 69.7 | - | - | - | - | - | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  | 1.4e-15 / 4.1e-12 |  |  | 3.0e-14 / 8.8e-11 |  |  |  |  |  |  |  |  |  |  |
| weather_h30000 | - | - | - | err | - | - | - | - | - | - | 189 | - | - | 183 | - | - | - | - | - | - | - | - | - | - | - | - | exact |  |  |  |  |  |  |  |  |  |  | 1.3e-14 / 1.2e-11 |  |  | 9.3e-14 / 9.0e-11 |  |  |  |  |  |  |  |  |  |  |
| weather_n2 | 0.026 | 0.022 | err | err | 0.135 | >600 | 6.15 | 0.56 | 2.76 | oom | 0.21 | 0.209 | 0.211 | 1.59 | 1.57 | 1.72 | 1.85 | 0.173 | 0.287 | 16.7 | 0.218 | 0.445 | 57.2 | 0.059 | 0.764 | 5.69 | exact | 2.8e-17 / 1.5e-16 | 4.0e-08 / 2.2e-07 |  |  | 2.8e-17 / 1.5e-16 |  | 2.8e-17 / 1.5e-16 | 5.6e-16 / 3.0e-15 | 8.3e-17 / 4.5e-16 |  | 5.6e-17 / 3.0e-16 | 2.8e-17 / 1.5e-16 | 8.3e-17 / 4.5e-16 | 2.8e-17 / 1.5e-16 | 2.8e-17 / 1.5e-16 | 2.8e-17 / 1.5e-16 | 8.3e-17 / 4.5e-16 | 6.0e-08 / 3.3e-07 | 3.9e-13 / 2.1e-12 | 3.9e-13 / 2.1e-12 | 3.9e-13 / 2.1e-12 | 1.7e-16 / 9.0e-16 | 2.5e-16 / 1.4e-15 | 0.0e+00 / 0.0e+00 |
| weather_n7 | 0.064 | 0.188 | err | err | >600 | - | err | >600 | 4.92 | - | 0.246 | 2 | 5.47 | 2.39 | 20.6 | 2.63 | 4.28 | 0.317 | 0.65 | 102 | 1.1 | 0.664 | 219 | 9.64 | 10.5 | oom | exact | 1.4e-20 / 2.0e-16 | 3.6e-08 / 5.4e-04 |  |  |  |  |  |  | 4.1e-20 / 6.0e-16 |  | 2.0e-19 / 3.0e-15 | 0.0e+00 / 0.0e+00 | 0.0e+00 / 0.0e+00 | 1.4e-20 / 2.0e-16 | 1.4e-20 / 2.0e-16 | 2.7e-20 / 4.0e-16 | 1.4e-20 / 2.0e-16 | 3.6e-07 / 5.4e-03 | 1.9e-17 / 2.8e-13 | 1.9e-17 / 2.8e-13 | 1.9e-17 / 2.8e-13 | 9.5e-20 / 1.4e-15 | 1.6e-19 / 2.4e-15 | 2.7e-20 / 4.0e-16 |
| weather_n8 | 0.147 | 0.403 | err | err | - | - | - | - | 5.95 | - | 0.347 | 9.11 | 9.72 | 13.9 | >600 | 2.81 | 6.4 | 0.39 | 0.755 | 593 | 1.51 | 1.11 | 255 | 25.6 | 26.6 | - | exact | 1.9e-20 / 1.4e-15 | 1.1e-08 / 8.8e-04 |  |  |  |  |  |  | 1.4e-20 / 1.0e-15 |  | 4.1e-20 / 3.1e-15 | 1.7e-21 / 1.3e-16 | 5.1e-21 / 3.9e-16 | 1.7e-21 / 1.3e-16 |  | 1.4e-20 / 1.0e-15 | 1.0e-20 / 7.8e-16 | 1.1e-08 / 8.8e-04 | 3.7e-17 / 2.9e-12 | 3.7e-17 / 2.9e-12 | 3.7e-17 / 2.9e-12 | 8.5e-21 / 6.5e-16 | 2.0e-20 / 1.6e-15 | 5.1e-21 / 3.9e-16 |
| weather_n9 | 0.186 | 0.94 | err | err | - | - | - | - | 7.55 | - | 0.823 | 42.3 | 18.1 | 197 | - | 3.49 | 11.6 | 0.853 | 1.27 | oom | 2.33 | 0.599 | 291 | 77.4 | 79.4 | - | exact | 3.8e-21 / 1.5e-15 | 1.9e-08 / 7.4e-03 |  |  |  |  |  |  | 3.0e-21 / 1.2e-15 |  | 6.4e-21 / 2.5e-15 | 8.5e-22 / 3.4e-16 | 1.3e-21 / 5.0e-16 | 2.5e-21 / 1.0e-15 |  | 3.4e-21 / 1.3e-15 | 2.1e-21 / 8.4e-16 | 4.8e-07 / 1.9e-01 | 4.2e-18 / 1.7e-12 |  | 4.2e-18 / 1.7e-12 | 8.5e-22 / 3.4e-16 | 2.1e-21 / 8.4e-16 | 1.7e-21 / 6.7e-16 |
| weather_n10 | 0.203 | 2.16 | err | err | - | - | - | - | 20.3 | - | 2.82 | 191 | 52.6 | 12.7 | - | 4.27 | 20.6 | zero | 1.2 | - | 2.67 | 0.764 | 331 | oom | oom | - | exact | 4.2e-22 / 1.1e-15 | 6.5e-10 / 1.6e-03 |  |  |  |  |  |  | 2.6e-22 / 6.6e-16 |  | 6.4e-22 / 1.6e-15 | 1.1e-22 / 2.7e-16 | 1.1e-22 / 2.7e-16 | 2.9e-17 / 7.1e-11 |  | 4.2e-22 / 1.1e-15 | 5.3e-23 / 1.3e-16 |  | 3.7e-19 / 9.4e-13 |  | 3.7e-19 / 9.4e-13 | 2.1e-22 / 5.3e-16 | 3.2e-22 / 8.0e-16 |  |
| weather_n11 | 0.307 | 4.77 | err | err | - | - | - | - | 41.1 | - | 10.9 | >600 | 207 | err | - | 6.16 | 98.7 | zero | 1.73 | - | 4.29 | 0.808 | 377 | - | - | - | exact | 7.9e-23 / 1.2e-15 | 3.6e-08 / 5.7e-01 |  |  |  |  |  |  | 6.6e-23 / 1.0e-15 |  | 4.4e-22 / 6.9e-15 |  | 1.3e-23 / 2.1e-16 |  |  | 1.3e-23 / 2.1e-16 | 5.3e-23 / 8.3e-16 |  | 1.4e-21 / 2.1e-14 |  | 1.4e-21 / 2.1e-14 | 4.0e-23 / 6.2e-16 | 5.3e-23 / 8.3e-16 |  |
| weather_n12 | 0.802 | zero | err | err | - | - | - | - | 216 | - | 45.3 | - | >600 | oom | - | 8.33 | 241 | zero | 2.36 | - | 8.74 | 1.35 | 405 | - | - | - | exact | 1.3e-23 / 1.3e-15 |  |  |  |  |  |  |  | 1.2e-23 / 1.1e-15 |  | 6.5e-23 / 6.4e-15 |  |  |  |  | 6.6e-24 / 6.5e-16 | 2.8e-23 / 2.8e-15 |  | 8.7e-21 / 8.5e-13 |  | 8.7e-21 / 8.5e-13 | 0.0e+00 / 0.0e+00 | 9.9e-24 / 9.8e-16 |  |
| weather_n13 | 3.4 | zero | >600 | err | - | - | - | - | 176 | - | 176 | - | - | - | - | 15.5 | 345 | zero | 7.72 | - | 16.5 | 1.86 | 443 | - | - | - | exact | 4.1e-24 / 2.6e-15 |  |  |  |  |  |  |  | 3.1e-24 / 1.9e-15 |  | 1.9e-23 / 1.2e-14 |  |  |  |  | 1.7e-24 / 1.0e-15 | 1.9e-24 / 1.2e-15 |  | 4.7e-21 / 2.9e-12 |  | 4.7e-21 / 2.9e-12 | 2.1e-25 / 1.3e-16 | 1.4e-24 / 9.0e-16 |  |
| weather_n15 | 16.9 | zero | >600 | err | oom | >600 | - | >600 | >600 | oom | >600 | err | err | err | err | >600 | >600 | oom | oom | - | oom | 7.04 | >600 | - | - | >600 | exact | 1.0e-25 / 2.5e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 3.2e-26 / 7.8e-16 |  |  |
| weather_n16 | 21.8 | zero | >600 | err | oom | >600 | - | >600 | >600 | oom | >600 | err | err | oom | oom | - | - | - | - | - | - | 15.4 | >600 | - | - | >600 | exact | 0.0e+00 / 0.0e+00 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1.6e-27 / 2.6e-16 |  |  |
| weather_n17 | 83.9 | >600 | >600 | err | oom | >600 | - | >600 | >600 | oom | >600 | err | err | oom | oom | >600 | >600 | oom | oom | >600 | oom | 168 | >600 | - | - | >600 | exact | 4.0e-27 / 3.7e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 2.0e-28 / 1.9e-16 |  |  |
| weather_n18 | 84.1 | >600 | >600 | err | oom | >600 | - | >600 | >600 | oom | >600 | err | err | err | err | - | - | - | - | - | - | >600 | - | - | - | >600 | exact | 2.3e-28 / 1.4e-15 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| weather_n19 | oom | >600 | >600 | err | oom | >600 | - | >600 | >600 | oom | >600 | err | err | oom | oom | - | - | - | - | - | - | - | - | - | - | >600 | exact |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |

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
`--only`; after a timeout or an allocation failure `run` skips the
larger sizes of the same program for that system unless `--no-skip` is
given (a network is no size of anything, and a family of different
programs is a ladder only where a program's sizes are listed in it,
`series`). The suites are `standard` (the
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
