#!/usr/bin/env python3
"""Benchmark tddlog against ProbLog on generated instance families.

    bench.py gen  [--dir bench] [--cnf] [--pr] [--alea CMD] [SELECTION]
    bench.py run  [--dir bench] [--out results.jsonl] [--tddlog BIN] [--problog BIN]
                  [--strategies s1,s2] [--kbs ddnnf,sdd,fsdd] [--dice CMD] [--alea CMD] [--timeout S]
                  [--pita SWIPL] [--pita-packs DIR] [--aspmc PYTHON] [--scallop SCLI] [--scallop-k 3,1000]
                  [--exact] [--ace DIR [--ace-variants auto,...] [--ace-heap 8g] [--ace-java JAVA]]
                  [--merlin BIN] [--agrum PYTHON] [--pgmpy PYTHON] [--libdai BIN] [--tensorinference JULIA]
                  [--provsql DIR] [--storm PYTHON] [--prism DIR] [--weather-models DIR]
                  [--ganak BIN] [--sharpsat-td BIN [--sharpsat-td-decot 1,10]] [--pr]
                  [--roulette CMD [--roulette-variants rsdd,nocomplement]]
                  [--no-skip] [--cpus 2,4,6,...] [--rotate R] [SELECTION]
    bench.py table results.jsonl... [--ref problog:ddnnf,...] [--approx scallop,...] [--pr-ref exact:cliquetree,...]
    bench.py compare --before a.jsonl --after b.jsonl [--systems tddlog:auto,...]
    bench.py fetch

SELECTION is `[--suite standard|indicators|large|lit|ace|uai|tpch|biomine|ppl|bitblast|roulette|geni
[--ace-benchmarks DIR | --uai-benchmarks DIR | --biomine-benchmarks DIR | --roulette-benchmarks DIR]]
[--families graph,grid,smokers,uwcse,cora,cplint,bn,bnev,dice,...] [--only name,...]`.
A strategy name may carry tddlog flags after a `+`, so `s+flag` is the run
`--vtree s --flag` and names its own column.

`gen` writes the instances from the fixed specification below with the
generators in this directory, so an instance is identified by its name
alone. The bnlearn networks of data/bn and data/bn-large are not in the
repository (no license for them is established): `fetch` downloads each
file data/fetch.txt lists, checks its SHA-256 and writes it in place, and
`gen` stops, naming `fetch`, where one is missing. `run` executes every (instance, system) pair once, single-threaded,
and appends one JSON line per run: wall seconds, exit status, the answers,
and for tddlog the largest formula size and operation counts from
`--stats`. Once a system times out or fails on an instance, the larger
instances of that family are skipped for it (`-` in the table), unless
`--no-skip` is given. Without `--cpus` the runs go one after another;
`--cpus` runs them in parallel lanes, one per CPU listed, every command of
a lane pinned to its CPU with `taskset`. A family's instances run in one
lane in order of size, so a timeout skips the larger ones as it does in a
sequential run; under `--no-skip` each instance is a job of its own. The
systems of an instance run back to back in its lane, in the order given,
or under `--rotate R` in that order rotated by a checksum of the
instance's name plus R: runs with R = 0 to n - 1 put each of n systems in
each place once, and instances that start together run different systems
first. `table`
prints a Markdown table with times and the maximum absolute answer
difference of each system against the reference system.

`--exact` adds the exact reference of the instances that have one (`EXACT`):
`exact:cliquetree`, `bn_exact.py`'s clique-tree calibration, on the networks
`gen_bn.py` writes, `exact:lit`, `lit_exact.py`, on the literature
families, `exact:tpch`, `tpch_exact.py`, on the TPC-H queries of `--suite
tpch`, `exact:reliability`, `reliability_exact.py`, on the Biomine
graphs of `--suite biomine`, and `exact:dp`, `gen_roulette.py
--reference-of` or `gen_geni.py --reference-of`, a dynamic program over
the model's states, on `--suite roulette` and `--suite geni`. They answer where ProbLog does not finish; `table` takes the
last four before ProbLog's answers and the clique tree after them. `--ace DIR` adds `ace:<variant>`, Chavira
and Darwiche's Ace 3.0, on the networks (`run_ace`), and `table` falls back
to `ace:auto` where the clique tree does not finish either; `--suite ace` is
Ace's own benchmark networks, and on `--suite uai` Ace reads a Hugin copy of
the network the program is written from. `--merlin BIN` adds `merlin:bte`, Merlin's
bucket-tree elimination, on the networks (`run_merlin`); `--suite uai` is
the UAI competition's MAR instances, Markov and Bayesian networks.

`--dice CMD` adds `dice:bdd`, Dice on the Dice program behind a `dice`,
roulette, geni or `bn` instance (`run_dice`), and `--alea CMD` adds `alea:cudd`, Alea.jl on
the same program translated by `dice2alea.py` (`run_alea`); `table` gives
Alea a second column, `alea:cudd in-process`, the time the Julia process
reports for building the program and `pr`, without Julia's start and
compilation of Alea. `--roulette CMD` adds `roulette:rsdd`, Roulette
(Moy et al., PLDI 2025) on its own programs of `--suite roulette`, and on
the other suites on the Dice program behind an instance translated by
`dice2roulette.py` (`run_roulette`); with `--roulette-variants
rsdd,nocomplement` it also queries caesar and the translated programs
without Roulette's complement (`roulette:nocomplement`).

`--suite ppl` is the benchmark programs of the Dice and Alea.jl papers and
of later papers (`PPL`): Dice programs (`nodice`), which run as a `dice`
instance's, and Alea.jl programs (`arith`, `hybit`, `pluck`, data/alea), whose
instance is the Boolean circuit Alea builds for the program, which
`alea_bench.jl` writes as a ProbLog program and as Dice programs (`gen`
runs it under `--alea CMD`). On those, `dice:bdd` runs the circuit's Dice
programs, and `alea:cudd` the Alea.jl program itself; `table` gives Alea a
third column, `alea:cudd warm`, the time of a second build and `pr` in the
same process, without Julia's compilation, as the papers time Alea.
`--suite bitblast` is the bit-blasting paper's Alea.jl programs, and
programs of other papers on exact and hybrid inference written in Alea.jl,
on ladders of size (`BITBLAST`), run as the `ppl` suite's are.

`--pr`, on `--suite uai`, compares the partition
functions as well (the UAI PR task): each tddlog strategy `s` runs as
`s+evidence-probability+no-queries`, which answers P(e) alone, as
Merlin's PR task and the counters do, on the program ground for its
evidence alone (in a Bayesian network, the ancestors of the observed
variables); the clique tree with `--evidence-probability`, and Merlin a
second time as `merlin:bte-pr` (`-t PR`). ProbLog, PITA and aspmc, which
print no P(e), are asked it as the probability of a query that holds when
the evidence does (`pe_program`), and Ace runs as for the marginals. A
record then holds `log10_z`, the base-10 logarithm of Z: for tddlog, the
clique tree, Ace and the program systems, log10 P(e) plus the constant
gen_bn.py states on the program's first line (`gen_bn.read_offset`); for
Merlin, the natural logarithm it prints divided by ln 10. `table` adds the
largest difference in log10 Z from the first of `--pr-ref` that finished.
On the uai suite `run` checks each baseline's record against the
reference when the instance's systems have run (`check_against_reference`):
marginals further than 1e-6 from the clique tree's (or Merlin's, or
tddlog's, the first that answered), or a log10 Z further than 1e-6 times
max(1, |log10 Z|), make the record a failed run. `gen --pr` writes the
uai suite's programs for the partition function alone (`gen_bn.py --query
none`): they ask no query, and a Potts network without evidence (the PR
set's queen and myciel colourings) has its colour symmetry broken
(`gen_bn.colour_symmetry`), exact for Z but not for the marginals, so
they are for `run --pr` alone; it writes no `.inst` files, so Ace, which
reads them, runs `--pr` on the programs of a plain `gen`. `--ganak BIN`,
which answers Z alone and so requires `--pr`, adds `ganak:wmc`, Ganak's
exact weighted count of a CNF (`run_ganak`; its model count on a CNF
without weights, `c t mc`):
on the uai suite the weighted CNF `gen` writes of each network and its
evidence (`gen_bn.py --cnf`, `uai_wcnf.py`). `--sharpsat-td BIN` adds
`sharpsat-td:decot<S>`, sharpSAT-TD's weighted count of the same CNFs
(`run_sharpsat_td`), one variant per FlowCutter budget S of
`--sharpsat-td-decot`.

Timed numbers are meant to come from a machine with nothing else running,
or from lanes of one `--cpus` run, all systems under the same load.
"""
import argparse
import decimal
import fractions
import gzip
import hashlib
import itertools
import json
import math
import os
import re
import shlex
import shutil
import signal
import subprocess
import tempfile
import sys
import threading
import time
import zlib
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
sys.path.insert(0, HERE)

import gen_bn  # noqa: E402
import ground2scallop  # noqa: E402
import problog2pita  # noqa: E402
import tpch2provsql  # noqa: E402  (after sys.path, so the script runs from anywhere)

# Suites that scripts/internal_suites.py, where there is one, adds: a name ->
# a Suite (its instances(base), the generator that writes each instance's
# program and weighted CNF, the default directory of its `--NAME-benchmarks`,
# and whether its programs ask no query, so that only `--pr` runs them).
try:
    from internal_suites import SUITES as EXTRA_SUITES  # noqa: E402
except ImportError:
    EXTRA_SUITES = {}
CNF_WRITERS = {suite.generator for suite in EXTRA_SUITES.values()}

# name -> (family, generator arguments)
SPEC = {}
for n, m, seed in [(10, 25, 3), (11, 29, 7), (12, 32, 7), (13, 35, 7), (14, 37, 7),
                   (15, 40, 7), (16, 44, 1), (18, 50, 1), (20, 56, 1), (20, 60, 1),
                   (25, 80, 1), (30, 100, 1)]:
    SPEC[f'g{n}_{m}'] = ('graph', ['gen_graph.py', str(n), str(m), '--seed', str(seed)])
for n in [4, 5, 6, 7, 8, 10, 12, 15, 20, 25, 30]:
    SPEC[f'grid{n}'] = ('grid', ['gen_grid.py', str(n), '--seed', '1'])
for n, k in [(6, 2), (10, 3), (15, 3), (20, 3), (50, 3), (100, 3), (200, 3)]:
    SPEC[f's{n}'] = ('smokers', ['gen_smokers.py', str(n), '--friends-per', str(k), '--seed', '1'])
for n in [2, 3, 4, 5, 6, 8, 10]:
    SPEC[f'uwcse{n}'] = ('uwcse', ['gen_uwcse.py', str(n)])
for q in [10, 50, 200, 500]:
    SPEC[f'cora{q}'] = ('cora', ['gen_cora.py', os.path.join(ROOT, 'data', 'cplint', 'cora_m1.pl'),
                                  '--queries', str(q), '--seed', '1'])
SPEC['bloodtype'] = ('cplint', ['gen_cplint.py', 'bloodtype'])
# Bayesian networks from the bnlearn repository (data/bn, Hugin format, which
# `fetch` downloads): every CPT row an annotated disjunction, the marginals
# of the leaf variables queried. Listed by node count. asia and cancer are
# examples/asia.net and examples/cancer.net, the same networks written out
# from their publications.
BN = ['cancer', 'earthquake', 'asia', 'survey', 'sachs', 'child', 'insurance', 'water', 'alarm',
      'hailfinder', 'hepar2', 'win95pts', 'pathfinder', 'andes', 'pigs', 'link', 'munin']


def bn_source(net):
    return os.path.join(ROOT, 'examples' if net in ('asia', 'cancer') else os.path.join('data', 'bn'), f'{net}.net')


for net in BN:
    SPEC[f'bn_{net}'] = ('bn', ['gen_bn.py', bn_source(net), '--query', 'leaves'])
# The same networks with half of their leaves observed, the values sampled
# forward from the network, and the other leaves queried. A network with one
# leaf (survey, mildew) has it observed and every other variable queried.
ONE_LEAF = {'survey', 'mildew'}
def ev_query(net):
    return 'all' if net in ONE_LEAF else 'leaves'
for net in BN:
    SPEC[f'bn_{net}_ev'] = ('bnev', ['gen_bn.py', bn_source(net),
                                     '--query', ev_query(net), '--evidence', 'half'])
# The baseline programs of the Dice paper (data/dice), translated.
DICE = ['twocoins', 'evidence1', 'evidence2', 'grass', 'alarm', 'noisyOr', 'murderMystery', 'digitRecognition']
for name in DICE:
    SPEC[f'dice_{name}'] = ('dice', ['dice2problog.py', os.path.join(ROOT, 'data', 'dice', f'{name}.dice')])

# Larger instances, outside the standard suite so that it stays stable.
# The suite was assembled to evaluate selective indicators, a rewrite since
# removed, and keeps its name.
INDICATORS = {name: SPEC[name] for name in [
    *(f'bn_{net}{suffix}' for net in ['insurance', 'water', 'hailfinder', 'hepar2',
                                    'win95pts', 'pathfinder', 'andes', 'pigs', 'link', 'munin']
      for suffix in ['', '_ev']),
    'g20_60', 'g30_100', 'grid15', 'grid30', 'uwcse8', 'uwcse10', 'cora500',
    'dice_digitRecognition',
]}
INDICATORS.update({
    'g40_140': ('graph', ['gen_graph.py', '40', '140', '--seed', '1']),
    'grid40': ('grid', ['gen_grid.py', '40', '--seed', '1']),
    'uwcse12': ('uwcse', ['gen_uwcse.py', '12']),
    'cora1000': ('cora', ['gen_cora.py', os.path.join(ROOT, 'data', 'cplint', 'cora_m1.pl'),
                         '--queries', '1000', '--seed', '1']),
})
for n in [15, 30]:
    INDICATORS[f'grid{n}_ev'] = ('gridev', ['gen_grid.py', str(n), '--seed', '1', '--evidence'])
INDICATORS['cora1000_ev'] = ('coraev', ['gen_cora.py', os.path.join(ROOT, 'data', 'cplint', 'cora_m1.pl'),
                                     '--queries', '1000', '--seed', '1', '--evidence', '20'])
# Dice programs with loops (data/dice/README.md): the Caesar cipher, the
# ladder and diamond networks of the Dice paper, and Dice's two model
# checking examples, translated.
for name in ['caesar-iter', 'simpleCaesar', 'ladder', 'diamond', 'pmc2', 'pmc1']:
    INDICATORS[f'dice_{name}'] = ('dice', ['dice2problog.py', os.path.join(ROOT, 'data', 'dice', f'{name}.dice')])


# The benchmark programs of the papers on Dice and Alea.jl and of a later
# paper that times Dice (docs/baselines.md, "Benchmark programs of the
# Dice and Alea.jl papers"), a suite of their own:
#   - arith: the Dice.jl programs of Cao, Garg, Holtzen, Van den Broeck and
#     Millstein, "Scaling Integer Arithmetic in Probabilistic Programs" (UAI
#     2023), ported to Alea.jl (data/alea/README.md);
#   - hybit: the Alea.jl programs of Garg, Holtzen, Van den Broeck and
#     Millstein, "Bit Blasting Probabilistic Programs" (PLDI 2024), at the
#     bits and pieces of the paper's Table 2;
#   - pluck: the Dice.jl programs of the sequence models of Bowers, Lew,
#     Tenenbaum, Solar-Lezama and Mansinghka, "Stochastic Lazy Knowledge
#     Compilation for Inference in Discrete Probabilistic Programs" (PLDI
#     2025), ported to Alea.jl;
#   - nodice: the purely probabilistic Dice programs of Gürtler and
#     Kaminski's noDice (OOPSLA 2026), translated by dice2problog.py.
# An Alea.jl program's instance is the Boolean circuit Alea builds for it,
# written by alea_bench.jl as a ProbLog program and as Dice programs
# (`gen` runs it under `--alea`).
ARITH = ['book', 'tugofwar', 'caesar_small', 'caesar_medium', 'caesar_large', 'caesar_huge',
         'ranking_small', 'ranking_medium', 'ranking_large', 'radar', 'fw_small', 'fw_medium', 'fw_large',
         'linext_small', 'linext_medium', 'linext_large', 'triangle_small', 'triangle_medium', 'triangle_large',
         'gcd_small', 'gcd_medium', 'gcd_large', 'disease_small', 'disease_medium', 'disease_large',
         'luhn_small', 'luhn_medium', 'plankton']
HYBIT = ['pi', 'weekend', 'spacex', 'GPA', 'tug_of_war', 'altermu2', 'conjugate_gaussians',
         'normal_mixture_theta', 'normal_mixture_mu1', 'normal_mixture_mu2', 'zeroone_w1', 'zeroone_w2',
         'coinBias', 'addFun_sum', 'clickGraph', 'trueskill', 'clinicalTrial1', 'clinicalTrial2', 'addFun_max']
PLUCK = ['hmm', 'pcfg', 'string_editing', 'sorted_list']
NODICE = ['coupon2-10-obs', 'coupon2-11-obs', 'coupon2-12-obs', 'coupon2-13-obs', 'coupon2-14-obs',
          'survey', 'hepar2', 'insurance', 'water', 'alarm', '3sat-0']
PPL = {}
for family, names in [('arith', ARITH), ('hybit', HYBIT), ('pluck', PLUCK)]:
    for name in names:
        PPL[f'{family}_{name}'] = (family, ['alea_bench.jl', os.path.join(ROOT, 'data', 'alea', family, f'{name}.jl')])
for name in NODICE:
    PPL[f'nodice_{name}'] = ('nodice', ['dice2problog.py', os.path.join(ROOT, 'data', 'nodice', f'{name}.dice')])
# An instance whose generator is dice2problog.py is a Dice program (its
# argument) translated; one whose generator is alea_bench.jl is the
# circuit of an Alea.jl program; the generators of DICE_WRITERS write a
# Dice program beside the ProbLog one (`--dice`, NAME.dice).
DICE_WRITERS = ('gen_roulette.py', 'gen_geni.py')
ALEA_BENCH = os.path.join(HERE, 'alea_bench.jl')

# The programs of the bit-blasting paper (Garg, Holtzen, Van den Broeck and
# Millstein, PLDI 2024) on ladders of size, a suite of its own
# (`--suite bitblast`). Each program of the paper's Table 2 (data/alea/
# hybit) is run with its precision, the bits after the binary point of
# its fixed-point numbers (the constant BB_PRECISION names), from 6 below
# the paper's to 6 above it in steps of 2 (1 at least), its pieces as in
# the paper. The programs of the paper's Section 2 (data/alea/bitblast)
# are run on ladders of their own: the gene expression program of Figure
# 1 by its number of genes T at the precision and pieces of the paper's
# Figure 2 (genes.jl), the paper's 5 to 50 genes and beyond, and with 5
# genes by its precision below that (`bb_genes5`, whose top is
# `bb_genes_t5`); the Yao-Vehtari-Gelman model of Figure 3 and the
# conjugate Gaussians of Figure 5 by their precision. A program is a
# family, `bb_NAME`, its instances `bb_NAME_wW` by W, the width of its
# numbers in bits (Table 2's column `Bit`), or `bb_genes_tT` by T,
# smallest first, so that a system that fails on one skips the larger.
# The instance's generator arguments end in the `--set NAME=VALUE` that
# alea_bench.jl runs the program with.
BB_PRECISION = {  # program: (its precision constant, the paper's value, width less precision)
    'pi': ('b', 12, 2), 'weekend': ('precision', 20, 4), 'spacex': ('bits', 11, 8), 'GPA': ('bits', 20, 5),
    'tug_of_war': ('bits', 17, 5), 'altermu2': ('bits', 11, 6), 'conjugate_gaussians': ('bits', 15, 8),
    'normal_mixture_theta': ('precision', 3, 6), 'normal_mixture_mu1': ('precision', 3, 6),
    'normal_mixture_mu2': ('precision', 3, 6), 'zeroone_w1': ('bits', 8, 8), 'zeroone_w2': ('bits', 11, 8),
    'coinBias': ('bits', 20, 2), 'addFun_sum': ('bits', 18, 5), 'clickGraph': ('bits', 8, 2),
    'trueskill': ('bits', 4, 6), 'clinicalTrial1': ('bits', 6, 2), 'clinicalTrial2': ('bits', 10, 2),
    'addFun_max': ('bits', 18, 5)}
BB_SECTION2 = {  # family: (program, the constant laddered, its values, width less precision or None)
    'genes': ('genes', 'T', [5, 10, 15, 20, 25, 30, 40, 50, 60, 80, 100], None),
    'genes5': ('genes', 'bits', [8, 10, 12, 14, 16, 18], 9),
    'multimodal': ('multimodal', 'bits', [2, 4, 6, 8, 10, 12], 7),
    'conjugate_ex3': ('conjugate_ex3', 'bits', [10, 12, 14, 16, 18, 20, 22], 8)}
BITBLAST = {}
for _name, (_const, _paper, _int) in BB_PRECISION.items():
    for _v in [_paper + d for d in range(-6, 7, 2) if _paper + d >= 1]:
        BITBLAST[f'bb_{_name}_w{_int + _v}'] = (
            f'bb_{_name}', ['alea_bench.jl', os.path.join(ROOT, 'data', 'alea', 'hybit', f'{_name}.jl'),
                            f'{_const}={_v}'])
for _name, (_prog, _const, _values, _int) in BB_SECTION2.items():
    for _v in _values:
        BITBLAST[f'bb_{_name}_' + (f't{_v}' if _int is None else f'w{_int + _v}')] = (
            f'bb_{_name}', ['alea_bench.jl', os.path.join(ROOT, 'data', 'alea', 'bitblast', f'{_prog}.jl'),
                            f'{_const}={_v}'])
# The paper's Figure 13 times four programs of its Table 2 over their
# pieces at the table's bits: families `bb_NAME_p`, instances `bb_NAME_pP`
# by the pieces P (the table's own pieces are the middle rung of `bb_NAME`
# and left out here).
BB_PIECES = {  # program: (its pieces constant, the pieces)
    'spacex': ('pieces', [4, 16, 64, 256, 1024, 4096]), 'weekend': ('num_pieces', [4, 16, 64, 256, 1024]),
    'altermu2': ('pieces', [4, 16, 64, 1024, 4096]), 'addFun_max': ('pieces', [4, 16, 64, 256, 1024, 4096])}
for _name, (_const, _values) in BB_PIECES.items():
    for _v in _values:
        BITBLAST[f'bb_{_name}_p{_v}'] = (
            f'bb_{_name}_p', ['alea_bench.jl', os.path.join(ROOT, 'data', 'alea', 'hybit', f'{_name}.jl'),
                              f'{_const}={_v}'])
# Programs of the bit-blasting paper's repository beside its tables, and of
# other papers on exact and hybrid inference, written as Alea.jl programs
# (data/alea/README.md): the laplace mixture of the paper's repository
# (bitblast/laplace.jl), FairSquare's decision trees as SPPL's Table 3 runs
# them (fairsquare/fairness.jl, a family per tree), SPPL's hierarchical
# Markov switching model (sppl/markov_switching.jl), R2's and PSI's linear
# regression (psi/linear_regression.jl), the path and star of the papers on
# weighted model integration (wmi/path.jl, wmi/star.jl), GuBPI's
# pedestrian (gubpi/pedestrian.jl) and Genfer's population model
# (genfer/population.jl). A family `bb_NAME` is laddered by one constant:
# by its bits after the binary point, instances `bb_NAME_wW` by the width W
# of its numbers, or by a size, instances `bb_NAME_nN` (steps of the Markov
# model, variables of the path and star) or `bb_NAME_sS` (the pedestrian's
# steps, the population's seasons); the trees fix `dt` for the family.
BB_PAPERS = {  # family: (directory, program, constant laddered, its values, label, width less precision, fixed)
    'laplace': ('bitblast', 'laplace', 'bits', [6, 8, 10, 12, 14, 16], 'w', 7, []),
    **{f'fair_dt{_t}': ('fairsquare', 'fairness', 'bits', [4, 8, 12, 16, 20, 24], 'w', 8, [f'dt={_t}'])
       for _t in [4, 14, 16, 44]},
    'markov': ('sppl', 'markov_switching', 'n', [10, 20, 50, 100, 200, 500, 1000, 2000], 'n', 0, []),
    'linreg': ('psi', 'linear_regression', 'bits', [1, 2, 3, 4, 5, 6, 7, 8], 'w', 7, []),
    'wmi_path': ('wmi', 'path', 'n', [16, 32, 64, 128, 256, 512, 1024], 'n', 0, []),
    'wmi_star': ('wmi', 'star', 'n', [16, 32, 64, 128, 256, 512, 1024], 'n', 0, []),
    'pedestrian': ('gubpi', 'pedestrian', 'steps', [4, 6, 8, 10, 12, 16, 20, 24, 32], 's', 0, []),
    'population': ('genfer', 'population', 'steps', [1, 2, 3, 4, 6, 8, 12, 16], 's', 0, [])}
for _name, (_dir, _prog, _const, _values, _label, _int, _fixed) in BB_PAPERS.items():
    for _v in _values:
        BITBLAST[f'bb_{_name}_{_label}{_int + _v}'] = (
            f'bb_{_name}', ['alea_bench.jl', os.path.join(ROOT, 'data', 'alea', _dir, f'{_prog}.jl'),
                            *_fixed, f'{_const}={_v}'])

# The scaling programs of Roulette's evaluation (Moy, Czenszak, Li,
# Marshall and Holtzen, PLDI 2025; gen_roulette.py, data/roulette/
# README.md), a suite of its own (`--suite roulette`): each program a
# family `rl_NAME` on a ladder of its size, from the artifact's sizes to
# sizes where the systems run out of time, and the nine networks of its
# Table 2, each with the marginal of the one variable the artifact queries,
# read from the artifact's .bif files (`--roulette-benchmarks`, the
# artifact's benchmark/ directory). gen_roulette.py writes a program's
# ProbLog and Dice programs; `--roulette CMD` runs Roulette itself on the
# artifact's program (`run_roulette`). The hmm marginal likelihood stops
# at 900 steps: past about 915 it is below the smallest normal double.
RL_LADDER = [1000, 3000, 10000, 30000, 100000, 300000, 1000000]
RL_SIZES = {'motivating': RL_LADDER, 'diamond': RL_LADDER, 'ladder': RL_LADDER,
            'caesar': [25, 50, 100, 200, 300, 500, 1000, 2000, 5000, 10000, 20000],
            'hmm-marginal-likelihood': [100, 300, 600, 900],
            **{f'hmm-{q}': [100, 300, 1000, 3000, 10000, 30000, 100000]
               for q in ['filtering-marginal', 'smoothing-marginal', 'filtering-joint', 'smoothing-joint']},
            'network': [10, 30, 100, 300, 1000, 3000, 10000, 30000],
            'hardware': [10, 30, 100, 300, 1000, 3000, 10000]}
# The artifact's networks, smallest first, and the variable it queries.
RL_NETWORKS = [('cancer', 'Xray'), ('survey', 'T'), ('alarm', 'PRESS'), ('insurance', 'PropCost'),
               ('hepar2', 'itching'), ('hailfinder', 'R5Fcst'), ('water', 'CBODD_12_45'),
               ('pigs', 'p392115290'), ('munin', 'L_SUR_CV_CA')]


def rl_family(program):
    return 'rl_' + program.replace('-', '_')


def roulette_suite(base):
    spec = {}
    for program, sizes in RL_SIZES.items():
        for n in sizes:
            spec[f'{rl_family(program)}_{n}'] = (rl_family(program), ['gen_roulette.py', program, str(n)])
    for net, var in RL_NETWORKS:
        spec[f'rl_bn_{net}'] = ('bn', ['gen_bn.py', os.path.join(base, 'bayesian-networks', f'{net}.bif'),
                                       '--query', var])
    return spec


# The programs of Geni's evaluation that Dice runs (Li and Zhang, ICFP
# 2025; gen_geni.py, data/geni/README.md), a suite of its own (`--suite
# geni`): the grid networks of its Figure 11, `geni_link` (link failures)
# and `geni_router` (router failures), by the side N of the N x N grid,
# from below the artifact's sizes to past where the systems run out of
# time, and Rubicon's weather factory of its Figure 9 by its number of
# factories N at 10 days (`geni_weather_n`, the artifact's N) and by its
# days H with 10 factories (`geni_weather_h`, the artifact's H but 10,
# which is geni_weather_n's). gen_geni.py writes each program's ProbLog
# and Dice programs and answers it (`exact:dp`).
GENI_SIZES = {'link': [4, 6, 8, 10, 12, 14, 16, 18, 20, 24, 28, 32],
              'router': [4, 6, 8, 10, 12, 14, 16, 18, 20, 24, 28, 32, 40]}
GENI_WEATHER_N = [2, 7, 8, 9, 10, 11, 12, 13, 15, 16, 17, 18, 19]
GENI_WEATHER_H = list(range(1, 10)) + list(range(100, 1001, 100))
GENI = {}
for _family, _sizes in GENI_SIZES.items():
    for _n in _sizes:
        GENI[f'geni_{_family}_{_n}'] = (f'geni_{_family}', ['gen_geni.py', _family, str(_n)])
for _n in GENI_WEATHER_N:
    GENI[f'geni_weather_n{_n}'] = ('geni_weather_n', ['gen_geni.py', 'weather', str(_n), '10'])
for _h in GENI_WEATHER_H:
    GENI[f'geni_weather_h{_h}'] = ('geni_weather_h', ['gen_geni.py', 'weather', '10', str(_h)])


# The networks the flip-hoisting evaluation adds (data/bn-large, gzipped,
# barley, diabetes and mildew from the same ProbLog copies as data/bn,
# munin1 to munin4 from bnlearn), with and without evidence. Not in the
# standard suite.
LARGE = {}
for net in ['barley', 'diabetes', 'mildew', 'munin1', 'munin2', 'munin3', 'munin4']:
    source = os.path.join(ROOT, 'data', 'bn-large', f'{net}.net.gz')
    LARGE[f'bn_{net}'] = ('bn', ['gen_bn.py', source, '--query', 'leaves'])
    LARGE[f'bn_{net}_ev'] = ('bnev', ['gen_bn.py', source, '--query', ev_query(net), '--evidence', 'half'])

# Five observations on internal nodes, sampled forward from the network.
for net in ['insurance', 'hailfinder', 'andes', 'link', 'munin']:
    INDICATORS[f'bn_{net}_internal5'] = ('bnint', ['gen_bn.py', os.path.join(ROOT, 'data', 'bn', f'{net}.net'),
        '--query', 'leaves', '--evidence', '5', '--evidence-from', 'internal', '--seed', '1'])

# Ace's benchmarks (Chavira and Darwiche, benchmarks_v1.0.zip: networks with
# .inst evidence from three papers), licensed for non-commercial research
# use and so unpacked outside the tree (`--ace-benchmarks`). One instance
# per network gen_bn.py reads, listed by family and size. A network of
# 2004-pgm and 2005-ijcai comes with 16 evidence sets: -0000 is empty,
# -0001 to -0005 observe roots only (none for most blockmaps), -0006 to
# -0010 a random set of variables (up to about half of them), -0011 to
# -0015 every leaf. The instance takes -0006 and queries the leaves left
# unobserved. A network of 2006-ijar has one set, which observes most of
# its variables, and the instance queries every other one. The 2005-ijcai
# copies of diabetes and munin1 to munin4 write some tables in
# percent, and those of water and mildew a root as weights, so they are
# read with gen_bn.py's --rescale (Ace's runs rescale every row too:
# `run_ace`); munin1, munin3 and munin4 have three variables more than
# bnlearn's copies in data/bn-large. Left out: 2005-ijcai's copies of
# blockmap_05_03 and mastermind_03_08_03, which are 2004-pgm's networks,
# and tcc4f.obfuscated, whose noisy-max nodes are not tables.
ACE_NETWORKS = (
    [('acenet', '2005-ijcai', n) for n in ['water.hugin', 'mildew.hugin', 'alarm.xdsl', 'hailfinder.xdsl',
                                           'pathfinder.xdsl', 'students_03_02.net', 'diabetes.hugin', 'pigs.hugin']]
    + [('acemunin', '2005-ijcai', f'munin{k}.hugin') for k in [1, 2, 3, 4]]
    + [('acemastermind', '2004-pgm', f'mastermind_{n}.net')
       for n in ['03_08_03', '04_08_03', '05_08_03', '06_08_03', '03_08_04', '04_08_04', '10_08_03', '03_08_05']]
    + [('aceblockmap', '2004-pgm', f'blockmap_{n:02}_{k:02}.net') for n in [5, 10, 15, 20, 22] for k in [1, 2, 3]]
    + [('acefs', '2006-ijar', f'fs-{n:02}.net') for n in [1, 4, 7, 10, 13, 16, 19, 22, 25, 28, 29]])
ACE_RESCALE = {'water', 'mildew', 'diabetes', 'munin1', 'munin2', 'munin3', 'munin4'}

# The UAI 2022 inference competition's tuning instances for MAR (MAR.zip of
# https://www.ics.uci.edu/~dechter/uaicompetition/2022/TuningBenchmarks/),
# Markov and Bayesian networks in the UAI format, each NAME.uai with one
# evidence file NAME.uai.evid, unpacked outside the tree (`--uai-benchmarks`).
# One instance per network, `uai_<name>`, in the family its name begins with
# (uaigrids, uaipromedus, ...), listed by family and the size of its file;
# it queries every variable left unobserved. The 2022 final sets name their
# networks after their origin: protein side chains by a PDB code (1aie),
# weighted CSPs with `.wcsp` and factor graphs with `.fg` before `.uai`, and
# numbered instances with `-` or `.` as well as `_` (mastermind_10_08_03-0009).


def uai_name(stem):
    return 'uai_' + re.sub(r'\W', '_', stem).lower()


def uai_family(stem):
    if re.fullmatch(r'\d[a-z0-9]{3}', stem.lower()):
        return 'uaiprotein'
    if stem.endswith('.wcsp'):
        return 'uaiwcsp'
    return 'uai' + re.sub(r'([-_.]?\d+)+$', '', stem.removesuffix('.fg')).lower()


def uai_suite(base):
    if not os.path.isdir(base):
        raise SystemExit(f'--suite uai reads UAI MAR instances, unpacked at {base} (--uai-benchmarks)')
    stems = [f.removesuffix('.uai') for f in os.listdir(base)
             if f.endswith('.uai') and os.path.exists(os.path.join(base, f + '.evid'))]
    if not stems:
        raise SystemExit(f'--uai-benchmarks {base}: no NAME.uai with a NAME.uai.evid there')
    stems.sort(key=lambda stem: (uai_family(stem), os.path.getsize(os.path.join(base, stem + '.uai')), stem))
    return {uai_name(stem): (uai_family(stem), ['gen_bn.py', os.path.join(base, stem + '.uai'),
                                            '--evid', os.path.join(base, stem + '.uai.evid')])
            for stem in stems}


def ace_name(network):
    return 'ace_' + re.sub(r'\W', '_', network.split('.')[0])


def ace_suite(base):
    if not os.path.isdir(base):
        raise SystemExit(f"--suite ace reads Ace's benchmarks, unpacked at {base} (--ace-benchmarks)")
    spec = {}
    for family, paper, network in ACE_NETWORKS:
        stem = network.split('.')[0]
        if family == 'acefs':
            evidence, query = f'{stem}.inst', 'all'
        else:
            evidence, query = f'{stem}-0006.inst', 'leaves'
        spec[ace_name(network)] = (family, ['gen_bn.py', os.path.join(base, paper, network), '--query', query,
                                            '--evidence-file', os.path.join(base, paper, evidence)]
                                   + (['--rescale'] if stem in ACE_RESCALE else []))
    return spec


# Families from published evaluations of probabilistic logic programming,
# probabilistic databases and neurosymbolic learning, each grown on a random
# structure from a seed (docs/benchmarks.md). A suite of their own, so that
# the standard suite's totals stay comparable across commits.
LIT = {}
for n in [4, 8, 16, 32, 64, 96]:
    LIT[f'mnist_sum{n}'] = ('mnist_sum', ['gen_mnist_sum.py', str(n), '--seed', '1'])
for g, w in [(4, 4), (4, 6), (6, 6), (10, 6), (5, 10), (6, 8), (16, 6), (32, 6), (6, 10), (7, 10)]:
    LIT[f'pedigree{g}_{w}'] = ('pedigree', ['gen_pedigree.py', str(g), str(w), '--seed', '1'])
for n, t in [(6, 10), (10, 10), (14, 10), (18, 20), (22, 40)]:
    LIT[f'sickness{n}_{t}'] = ('sickness', ['gen_sickness.py', str(n), str(t), '--seed', '1'])
for d, f in [(6, 9), (8, 12), (10, 15), (15, 22), (20, 30), (30, 45)]:
    LIT[f'noisyor{d}_{f}'] = ('noisyor', ['gen_noisyor.py', str(d), str(f), '--seed', '1'])
for c, pegs, r in [(4, 4, 8), (6, 4, 8), (8, 4, 8), (10, 4, 8), (4, 5, 8), (6, 5, 8), (8, 5, 8)]:
    LIT[f'mastermind{c}_{pegs}_{r}'] = ('mastermind', ['gen_mastermind.py', str(c), str(pegs), str(r), '--seed', '1'])
for n in [10, 15, 20, 30, 40]:
    LIT[f'pdb{n}'] = ('pdb', ['gen_pdb.py', str(n), '--seed', '1'])
LIT_FAMILIES = list(dict.fromkeys(family for family, _ in LIT.values()))

# The exact reference of the instances a generator writes, for the instances
# ProbLog does not finish: generator -> (variant, command for a program).
EXACT = {'gen_bn.py': ('cliquetree', lambda path: ['bn_exact.py', '--precision', '17', path])}
for family in LIT_FAMILIES:
    EXACT[f'gen_{family}.py'] = ('lit', lambda path, family=family: ['lit_exact.py', family, path,
                                                                    '--precision', '17'])


# TPC-H as a tuple-independent probabilistic database (gen_tpch.py), the
# workload of SPROUT and of dissociation: each Boolean query alone, a
# family of its own, at scale factors from 0.001 to 0.1, instance
# tpch_<query>_<1000 SF>, from the smallest at which its selections leave a
# tuple. q19's leave at most three below SF 0.1; it is not in the suite.
# Each query is also written through views (`--views`), as a person would
# write it over a database, instance tpch_<query>v_<1000 SF>: the same
# database and lineage, so the same answer.
TPCH = {}
for q, first in [('q1', 1), ('q2', 2), ('q3', 1), ('q4', 1), ('q6', 1), ('q7', 2), ('q10', 1), ('q11', 5),
                 ('q12', 1), ('q14', 1), ('q15', 1), ('q16', 1), ('q17', 5), ('q18', 1), ('q20', 2), ('q21', 2)]:
    for views in ['', 'v']:
        for m in [1, 2, 5, 10, 20, 50, 100]:
            if m >= first:
                TPCH[f'tpch_{q}{views}_{m}'] = (f'tpch_{q}{views}', ['gen_tpch.py', str(m / 1000), '--queries', q, '--seed', '1']
                                                + (['--views'] if views else []))
EXACT['gen_tpch.py'] = ('tpch', lambda path: ['tpch_exact.py', path, '--precision', '17'])

# Biomine (gen_biomine.py): the union of the K most probable paths between
# Alzheimer disease and four genes in the human Biomine graph of 2018,
# licensed for academic and non-commercial use and so read from outside
# the tree (`--biomine-benchmarks`, the directory that holds the zip). K
# stops at 64, the largest the reference answers.
BIOMINE_GRAPH = 'human_3_oct_2018.zip'
BIOMINE_K = [1, 2, 4, 8, 12, 16, 24, 32, 48, 64]
EXACT['gen_biomine.py'] = ('reliability', lambda path: ['reliability_exact.py', path, '--precision', '17'])
EXACT['gen_roulette.py'] = ('dp', lambda path: ['gen_roulette.py', '--reference-of', path])
EXACT['gen_geni.py'] = ('dp', lambda path: ['gen_geni.py', '--reference-of', path])


def biomine_suite(base):
    graph = os.path.join(base, BIOMINE_GRAPH)
    if not os.path.exists(graph):
        raise SystemExit(f'--suite biomine reads the Biomine graph {BIOMINE_GRAPH} in {base} (--biomine-benchmarks)')
    return {f'biomine{k}': ('biomine', ['gen_biomine.py', graph, str(k)]) for k in BIOMINE_K}


def instances(args):
    spec = ace_suite(args.ace_benchmarks) if args.suite == 'ace' else \
        uai_suite(args.uai_benchmarks) if args.suite == 'uai' else \
        biomine_suite(args.biomine_benchmarks) if args.suite == 'biomine' else \
        EXTRA_SUITES[args.suite].instances(getattr(args, args.suite + '_benchmarks')) if args.suite in EXTRA_SUITES else \
        roulette_suite(args.roulette_benchmarks) if args.suite == 'roulette' else \
        {'standard': SPEC, 'indicators': INDICATORS, 'large': LARGE, 'lit': LIT, 'tpch': TPCH,
         'ppl': PPL, 'bitblast': BITBLAST, 'geni': GENI}[args.suite]
    unknown = set(args.only) - spec.keys()
    if unknown:
        raise SystemExit('unknown instances: ' + ', '.join(sorted(unknown)))
    return {n: entry for n, entry in spec.items()
            if (not args.families or entry[0] in args.families) and (not args.only or n in args.only)}


def has_cnf(entry):
    """Whether `gen` writes a weighted CNF beside the instance's program,
    for the weighted model counters (Ganak, sharpSAT-TD): the CNF
    of an extra suite's instance, uncompressed, and a UAI network's weighted CNF with its
    evidence (`gen_bn.py --cnf`)."""
    gargs = entry[1]
    return gargs[0] in CNF_WRITERS or (gargs[0] == 'gen_bn.py' and gargs[1].removesuffix('.gz').endswith('.uai'))


FETCH = os.path.join(ROOT, 'data', 'fetch.txt')


def fetched():
    """The files `fetch` downloads (data/fetch.txt): {absolute path:
    (SHA-256, URL)}."""
    out = {}
    with open(FETCH) as f:
        for line in f:
            words = line.split('#', 1)[0].split()
            if words:
                sha, path, url = words
                out[os.path.join(ROOT, path)] = (sha, url)
    return out


def fetch(args):
    """Download each file of data/fetch.txt that is not in place and
    check every file's SHA-256, a downloaded one before it is written. The
    SHA-256 is that of the file at the URL: where the path ends in .gz and
    the URL does not, the file is written gzipped and checked
    decompressed."""
    import urllib.request
    bad = []
    for path, (sha, url) in fetched().items():
        zipped = path.endswith('.gz') and not url.endswith('.gz')
        rel = os.path.relpath(path, ROOT)
        if os.path.exists(path):
            with (gzip.open if zipped else open)(path, 'rb') as f:
                data = f.read()
            if hashlib.sha256(data).hexdigest() != sha:
                bad.append(f'{rel}: its SHA-256 is not that of {url}')
            continue
        with urllib.request.urlopen(url, timeout=args.timeout) as r:
            data = r.read()
        got = hashlib.sha256(data).hexdigest()
        if got != sha:
            bad.append(f'{url}: SHA-256 {got}, not {sha}; not written')
            continue
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path + '.part', 'wb') as f:
            f.write(gzip.compress(data, mtime=0) if zipped else data)
        os.replace(path + '.part', path)
        print(f'{rel}: {len(data)} bytes from {url}')
    if bad:
        raise SystemExit('\n'.join(bad))


def gen(args):
    os.makedirs(args.dir, exist_ok=True)
    spec = instances(args)
    need = fetched()
    missing = sorted({os.path.relpath(g, ROOT) for _, gargs in spec.values() for g in gargs
                      if isinstance(g, str) and g in need and not os.path.exists(g)})
    if missing:
        raise SystemExit(f'{len(missing)} networks are not in place ({", ".join(missing[:3])}'
                         f'{", ..." if len(missing) > 3 else ""}): python3 scripts/bench.py fetch downloads them')
    # A UAI network's CNF is written on request (`--cnf`): the tuning set's
    # come to 221 MB, which only the counters read.
    if args.cnf and not any(has_cnf(e) and e[1][0] == 'gen_bn.py' for e in spec.values()):
        raise SystemExit('--cnf writes the weighted CNFs of UAI networks, and the suite has none')
    alea = [n for n, e in spec.items() if e[1][0] == 'alea_bench.jl']
    if alea and not getattr(args, 'alea', ''):
        raise SystemExit(f'{alea[0]} is the circuit of an Alea.jl program, which alea_bench.jl writes under '
                         f'--alea ({len(alea)} instances)')
    if getattr(args, 'alea', '') and not alea:
        raise SystemExit('--alea writes the circuits of Alea.jl programs, and none is selected')
    if getattr(args, 'pr', False) and args.suite != 'uai':
        raise SystemExit('--pr writes the uai suite\'s programs for the partition function alone, and the suite is '
                         + args.suite)
    for name, entry in spec.items():
        gargs = entry[1]
        if gargs[0] == 'alea_bench.jl':
            # NAME.pl, NAME__dJ.dice and NAME.dicemap.
            subprocess.run(alea_command(args, 'export', gargs[1:], os.path.abspath(os.path.join(args.dir, name))),
                           check=True, stdout=subprocess.DEVNULL)
            continue
        cmd = [sys.executable, os.path.join(HERE, gargs[0])] + gargs[1:]
        if gargs[0] == 'gen_bn.py':
            # The observations, for Ace, which reads the network file, and
            # the network as a UAI file with its observations, for Merlin
            # (which reads a UAI network's own file). Under --pr the
            # programs for the partition function alone, without the
            # observations: run --pr gives Ace the .inst files of a plain gen.
            if getattr(args, 'pr', False):
                cmd += ['--query', 'none']
            else:
                cmd += ['--inst', os.path.join(args.dir, name + '.inst')]
            if not gargs[1].endswith('.uai'):
                cmd += ['--uai', os.path.join(args.dir, name + '.uai')]
        if gargs[0] in DICE_WRITERS:
            cmd += ['--dice', os.path.join(args.dir, name + '.dice')]
        if has_cnf(entry) and (gargs[0] in CNF_WRITERS or args.cnf):
            cmd += ['--cnf', os.path.join(args.dir, name + '.cnf')]
        out = subprocess.run(cmd, capture_output=True, text=True, check=True).stdout
        with open(os.path.join(args.dir, name + '.pl'), 'w') as f:
            f.write(out)
    print(f'{len(spec)} instances in {args.dir}')


def parse_answers(out):
    res = {}
    for line in out.splitlines():
        if line.startswith('evidence is inconsistent'):
            return 'inconsistent'
        if line.startswith(EVIDENCE_LINE):
            continue
        if ':' in line:
            q, v = line.rsplit(':', 1)
            try:
                res[re.sub(r'\s+', '', q)] = float(v)
            except ValueError:
                pass
    return res


EVIDENCE_LINE = 'P(e):'


def parse_evidence_probability(out):
    """log10 P(e) from the line `--evidence-probability` prints after the
    answers, `P(e):<tab>p<tab>log10 l` (tddlog and bn_exact.py alike), `-inf`
    for inconsistent evidence; None without the line."""
    for line in out.splitlines():
        if line.startswith(EVIDENCE_LINE):
            fields = line.split('\t')
            if len(fields) != 3 or not fields[2].startswith('log10 '):
                raise ValueError(f'not an evidence line: {line!r}')
            return float(fields[2].removeprefix('log10 '))
    return None


def partition_function(out, path):
    """log10 P(e) and log10 Z of a run of the program at `path` that printed
    the `--evidence-probability` line: Z is P(e) times the constant gen_bn.py
    states on the program's first line, which it writes for a Markov
    network. A ValueError where either is missing."""
    log10_pe = parse_evidence_probability(out)
    if log10_pe is None:
        raise ValueError('no P(e) line')
    return log10_pe, log10_pe + program_offset(path)


def program_offset(path):
    """The constant `log10 Z - log10 P(e)` gen_bn.py states on the first
    line of the program at `path`; a ValueError where it states none."""
    with open(path) as f:
        offset = gen_bn.read_offset(f.readline())
    if offset is None:
        raise ValueError(f'{path} states no log10 Z - log10 P(e) on its first line '
                         '(gen_bn.py writes it for a MARKOV network; rerun bench.py gen)')
    return offset


# Under --pr the systems that print no P(e) are asked it as a query of a
# program of their own (`pe_program`).
PE_SYSTEMS = ('problog', 'pita', 'aspmc')
PE_QUERY = 'bench_pe'


def pe_program(path, out):
    """Writes to `out` the program at `path` asking P(e) as a query: its
    queries and observations dropped, and `bench_pe :- e1, ..., en.` and
    `query(bench_pe).` added, the conjunction of what it observed (gen_bn.py
    observes atoms, `evidence(a).`), so that a system that prints only the
    probabilities of queries answers the PR task (`--pr`). Its first line,
    the offset gen_bn.py states, is kept. Returns `out`."""
    kept, observed = [], []
    for line in open(path).read().splitlines(keepends=True):
        m = re.match(r'^evidence\((.*?)(?:,\s*(true|false))?\)\.\s*(?:%.*)?$', line)
        if m:
            if m.group(2) == 'false':
                raise ValueError(f'{path}: a negative observation, which gen_bn.py does not write: {line.strip()}')
            observed.append(m.group(1).strip())
        elif not re.match(r'^query\(', line):
            kept.append(line)
    body = ', '.join(observed) or 'true'
    with open(out, 'w') as f:
        f.write(''.join(kept) + f'{PE_QUERY} :- {body}.\nquery({PE_QUERY}).\n')
    return out


# On the networks of --suite uai, observed factors take P(e) as low as
# 1e-618, and a baseline that computes in doubles can answer wrongly (Ace
# prints P(e) = 0 and posteriors that are not numbers where it underflows).
# There every baseline's answer is checked against the instance's reference
# once all its systems have run (`check_against_reference`): the marginals
# against the first of ANSWER_REFS that answered, to CHECK_TOL, log10 Z
# against the first of PR_REFS, to CHECK_TOL times max(1, |log10 Z|).
CHECKED_SUITES = ('uai',)
CHECK_TOL = 1e-6
ANSWER_REFS = [('exact', 'cliquetree'), ('merlin', 'bte'), ('tddlog', None)]
PR_REFS = [('exact', 'cliquetree'), ('merlin', 'bte-pr'), ('ganak', 'wmc'), ('tddlog', None)]


def answers_diff(a, b):
    """The largest difference of two answers over the queries both answer,
    infinite where one finds the evidence inconsistent and the other does
    not, None where they share no query."""
    if isinstance(a, dict) and isinstance(b, dict):
        return max((answer_diff(a[q], b[q]) for q in a.keys() & b.keys()), default=None)
    return 0.0 if a == b else math.inf


def check_against_reference(rec, done):
    """Checks a baseline's record `rec` of an instance against the
    reference among `done`, the instance's records by (system, variant): an
    answer further from the reference's than the tolerance makes the record
    an error, a failed run, as a crash would. The record keeps its answers,
    the reference it was checked against and the difference (`ref`,
    `ref_diff`; `pr_ref`, `pr_ref_diff` for log10 Z). A record with nothing
    to check against stays as it is, and so does the reference itself."""
    for key, refs, prefix in (('answers', ANSWER_REFS, 'ref'), ('log10_z', PR_REFS, 'pr_ref')):
        if rec['status'] != 'ok' or key not in rec:
            continue
        for system, variant in refs:
            ref = next((r for (s, v), r in done.items() if s == system and variant in (None, v)
                        and r['status'] == 'ok' and key in r), None)
            if ref is None:
                continue
            if ref is rec:
                break
            if key == 'answers':
                d, tol = answers_diff(rec['answers'], ref['answers']), CHECK_TOL
                if d is None:
                    continue
            else:
                z = ref['log10_z']
                d, tol = pr_diff(rec['log10_z'], z), CHECK_TOL * max(1.0, abs(z)) if math.isfinite(z) else 0.0
            name = f"{ref['system']}:{ref['variant']}"
            rec[prefix], rec[prefix + '_diff'] = name, d
            if d > tol:
                what = 'the marginals differ' if key == 'answers' else 'log10 Z differs'
                rec['status'] = 'error'
                rec['error'] = f"{what} from {name}'s by {d:.2e}, more than {tol:.1e}"
            break


def pr_variant(strategy):
    """The tddlog variant `--pr` runs for `strategy`: with
    `--evidence-probability` and `--no-queries`, P(e) alone."""
    name, *opts = strategy.split('+')
    return '+'.join([name] + opts + [o for o in ('evidence-probability', 'no-queries') if o not in opts])


def parse_stats(err):
    st = {}
    m = re.search(r'largest formula (\d+) pairs(?: \(final (\d+)\))?, ([0-9.]+)s', err)
    if m:
        st['compile_pairs'] = st['pairs'] = int(m.group(1))
        if m.group(2):
            st['final_pairs'] = int(m.group(2))
        st['compile_s'] = float(m.group(3))
    # The query phase conjoins the evidence, and with indicators the links,
    # after compilation: `families` spends its pairs there.
    m = re.search(r'evidence and queries: ([0-9.]+)s, evidence (\d+) pairs, largest formula (\d+) pairs', err)
    if m:
        st['query_s'] = float(m.group(1))
        st['evidence_pairs'] = int(m.group(2))
        st['query_pairs'] = int(m.group(3))
        st['pairs'] = max(st.get('pairs', 0), st['evidence_pairs'], st['query_pairs'])
    m = re.search(r'vtree (.*?) over (\d+) vars in ([0-9.]+)s', err)
    if m:
        st['vtree'] = m.group(1)
        st['vars'] = int(m.group(2))
        st['vtree_s'] = float(m.group(3))
    m = re.search(r'ground: (\d+) atoms .*?, (\d+) rules, ([0-9.]+)s', err)
    if m:
        st['atoms'] = int(m.group(1))
        st['rules'] = int(m.group(2))
        st['ground_s'] = float(m.group(3))
    m = re.search(r'indicators: (\w+), (\d+) copies of (\d+) atoms', err)
    if m:
        st['indicator_policy'] = m.group(1)
        st['indicator_copies'] = int(m.group(2))
        st['indicator_atoms'] = int(m.group(3))
    kinds = ['and', 'or', 'negate', 'minimize', 'equivalent']
    m = re.search(r'compile operations: and (\d+), or (\d+), negate (\d+), minimize (\d+), equivalent (\d+) ', err)
    if m:
        for kind, n in zip(kinds, m.groups()):
            st[kind + '_n'] = int(n)
    # A trial stopped while compiling also says how far its compile got,
    # and one in its query phase how many of its steps it has finished.
    # An attempt that may hold operations past its watch says to what cap.
    trials = re.findall(r'^trial: (\S+) indicators=(\w+), watch=(\d+)(?:, cap=(\d+))?, work=(\d+), '
                        r'phase=(\w+)(?:, (?:components|steps) \d+/\d+(?: at round \d+)?)?, (\w+), ([0-9.]+)s$',
                        err, re.M)
    if trials:
        st['trials'] = [dict(vtree=v, indicators=p, watch=int(w), **({'cap': int(c)} if c else {}), work=int(k),
                             phase=phase, outcome=outcome, seconds=float(t))
                        for v, p, w, c, k, phase, outcome, t in trials]
    # Under auto, the seconds each trial's layout took to build; within a
    # families layout, the min-fill eliminations (where it says), the
    # decomposer's searches and the laying out and weighing of the
    # candidates; within `td`'s, its eliminations (where it says) and its
    # search; `dynamic`'s min-fill start (where it says).
    layouts = re.findall(r'^layout: (\S+) indicators=(\w+), ([0-9.]+)s$', err, re.M)
    if layouts:
        st['layouts'] = [dict(vtree=v, indicators=p, seconds=float(t)) for v, p, t in layouts]
    families = re.findall(r'^families: .*?(?:; min-fill (\d+) units charged, ([0-9.]+)s)?; searched (\d+) units charged, '
                          r'([0-9.]+)s, laid out and weighed ([0-9.]+)s$', err, re.M)
    if families:
        st['families_layouts'] = [dict(**({'min_fill_charged': int(mu), 'min_fill_s': float(ms)} if mu else {}),
                                       charged=int(u), search_s=float(a), weigh_s=float(b))
                                  for mu, ms, u, a, b in families]
    td = re.findall(r'^(td: .*), (\d+) charged, ([0-9.]+)s$', err, re.M)
    if td:
        st['td_searches'] = []
        for line, u, t in td:
            e = re.search(r', eliminations min-degree (\d+)(?:, min-fill (\d+))? units,', line)
            st['td_searches'].append(dict(**({'min_degree_charged': int(e.group(1))} if e else {}),
                                          **({'min_fill_charged': int(e.group(2))} if e and e.group(2) else {}),
                                          charged=int(u), seconds=float(t)))
    dynamic = re.findall(r'^dynamic: min-fill (\d+) units charged$', err, re.M)
    if dynamic:
        st['dynamic_layouts'] = [dict(charged=int(u)) for u in dynamic]
    # Projection reports what it did unless it is off, and only one of the
    # two lines at a time: none of the components moved, or this many did.
    m = re.search(r'projection: (\d+) of (\d+) components on (\d+) projected vtrees, '
                  r'largest kept cone (\d+) of (\d+) facts', err)
    if m:
        st['projected'] = int(m.group(1))
        st['scopes'] = int(m.group(3))
        st['largest_cone'] = int(m.group(4))
        st['facts'] = int(m.group(5))
    elif re.search(r'projection: every one of the \d+ components', err):
        st['projected'] = 0
    return st


# The lane of a `run` worker thread: the CPU its commands are pinned to,
# or None without `--cpus`.
LANE = threading.local()
# The commands under way, which a stopped driver kills (`stop_all`), and
# whether it has been stopped.
LIVE = set()
LIVE_LOCK = threading.Lock()
STOPPED = threading.Event()


def kill(p):
    if os.name == 'posix':
        try:
            os.killpg(p.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass  # The group exited at the timeout boundary.
    else:
        p.kill()


def stop_all():
    """Stops the driver's commands: the driver's SIGTERM handler and Ctrl-C
    reach the main thread, and the commands run in the workers' threads. The
    new session of each command protects descendants from accidental timeout
    leaks, but also keeps a process-group kill of the driver from reaching
    them unless the driver explicitly forwards cancellation."""
    STOPPED.set()
    with LIVE_LOCK:
        for p in LIVE:
            kill(p)


def timed(cmd, timeout, cwd=None, env=None):
    if STOPPED.is_set():
        raise SystemExit('stopped')
    cpu = getattr(LANE, 'cpu', None)
    if cpu is not None:
        cmd = ['taskset', '-c', str(cpu)] + list(cmd)
    t0 = time.monotonic()
    # ProbLog and aspmc launch compilers of their own. Killing only the
    # frontend leaves those compilers consuming CPU during later timings.
    with subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                          text=True, cwd=cwd, env=env, start_new_session=(os.name == 'posix')) as p:
        with LIVE_LOCK:
            LIVE.add(p)
        try:
            stdout, stderr = p.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            kill(p)
            p.communicate()
            return {'status': 'timeout', 'seconds': timeout}
        except BaseException:
            kill(p)
            p.communicate()
            raise
        finally:
            with LIVE_LOCK:
                LIVE.discard(p)
    if STOPPED.is_set():
        raise SystemExit('stopped')
    dt = time.monotonic() - t0
    if p.returncode != 0:
        err = stderr.strip() or stdout.strip() or f'exit {p.returncode}'
        # Rust's last line can be just the backtrace hint. Retain the cause,
        # including allocation failures, without unbounded trace records.
        if len(err) > 8192:
            err = err[:4096] + '\n... output truncated ...\n' + err[-4096:]
        return {'status': 'error', 'seconds': dt, 'error': err}
    return {'status': 'ok', 'seconds': dt, 'stdout': stdout, 'stderr': stderr}


def queries(path):
    """The query atoms of a program, in order."""
    return [m.group(1) for m in re.finditer(r'^query\((.*)\)\.\s*(?:%.*)?$', open(path).read(), re.M)]


def parse_dice(out):
    """Dice's table: one row per value of the returned expression, a tuple's
    value written `(true, false)`."""
    rows = {}
    for line in out.splitlines():
        f = line.rsplit(None, 1)
        if len(f) == 2 and f[0] != 'Value':
            try:
                rows[re.sub(r'\s+', '', f[0])] = float(f[1])
            except ValueError:
                pass
    return rows


def dice_marginals(rows, n):
    """The probability that each of a returned tuple's n Booleans is true,
    from Dice's joint table; None where the rows are not such tuples."""
    split = {}
    for value, p in rows.items():
        parts = value.removeprefix('(').removesuffix(')').split(',')
        if not value.startswith('(') or len(parts) != n or not set(parts) <= {'true', 'false'}:
            return None
        split[tuple(parts)] = p
    return [sum(p for parts, p in split.items() if parts[i] == 'true') for i in range(n)]


def run_dice(args, name, path, tool='dice'):
    """Run Dice (or, with `tool`, another system that reads Dice programs and prints Dice's table: noDice, `--nodice`) on the program(s) behind a translated instance: the source
    program of a `dice` or `nodice` instance, the program a generator of
    DICE_WRITERS writes beside its instance, the programs of an Alea.jl
    program's circuit (`run_dice_circuit`), and for a network one generated
    program per queried variable, with the observations (Dice returns one
    value; a tuple of them would print the joint; `gen_bn.dice_programs`). The answers are keyed like the ProbLog program's
    queries: the row of value k is the k-th query of that variable."""
    family, gargs = instances(args)[name]
    if gargs[0] == 'dice2problog.py':
        programs = [(gargs[1], queries(path))]
    elif gargs[0] in DICE_WRITERS:
        programs = [(os.path.join(args.dir, name + '.dice'), queries(path))]
    elif gargs[0] == 'alea_bench.jl':
        return run_dice_circuit(args, name, tool)
    elif gargs[0] == 'gen_bn.py':
        try:
            per_variable = gen_bn.dice_programs(gargs[1:])
        except SystemExit as x:
            return {'status': 'error', 'seconds': 0, 'error': f'gen_bn.py --format dice: {x}'}
        programs = []
        for v, text in per_variable:
            dice_path = os.path.join(args.dir, f'{name}__{v}.dice')
            with open(dice_path, 'w') as f:
                f.write(text)
            programs.append((dice_path, [q for q in queries(path) if q.split('(')[0] == v]))
    else:
        return {'status': 'error', 'seconds': 0, 'error': f'no Dice program for family {family}'}
    total, answers = 0.0, {}
    for program, qs in programs:
        program = os.path.abspath(program)
        cmd = getattr(args, tool).format(file=program, dir=os.path.dirname(program),
                                         base=os.path.basename(program)).split()
        r = timed(cmd, max(0.0, args.timeout - total))
        total += r['seconds']
        if r['status'] != 'ok':
            r['seconds'] = total
            return r
        rows = parse_dice(r['stdout'])
        marginals = dice_marginals(rows, len(qs)) if len(qs) > 1 else None
        if 'true' in rows and len(qs) == 1:
            answers[re.sub(r'\s+', '', qs[0])] = rows['true']
        elif marginals is not None:
            # A returned tuple of Booleans, its k-th the k-th query.
            for q, p in zip(qs, marginals):
                answers[re.sub(r'\s+', '', q)] = p
        else:
            for k, q in enumerate(qs):
                if str(k) in rows:
                    answers[re.sub(r'\s+', '', q)] = rows[str(k)]
    return {'status': 'ok', 'seconds': total, 'answers': answers}


def run_dice_circuit(args, name, tool='dice'):
    """Dice (or noDice, `tool`) on the programs alea_bench.jl writes for the circuit of an
    Alea.jl program, NAME__dJ.dice, listed in NAME.dicemap: a line `FILE
    bool ATOM` answers ATOM with the probability of true, a line `FILE
    joint PRED N` answers PRED(U) for U in 0 .. 2^N - 1 with the row of the
    returned N Booleans (nested pairs, the most significant first) whose
    value is U. The seconds are the sum of the programs'."""
    total, answers = 0.0, {}
    for line in open(os.path.join(args.dir, name + '.dicemap')):
        file, kind, atom, *rest = line.split()
        program = os.path.abspath(os.path.join(args.dir, file))
        cmd = getattr(args, tool).format(file=program, dir=os.path.dirname(program),
                                         base=os.path.basename(program)).split()
        r = timed(cmd, max(0.0, args.timeout - total))
        total += r['seconds']
        if r['status'] != 'ok':
            r['seconds'] = total
            return r
        rows = parse_dice(r['stdout'])
        if kind == 'bool':
            answers[atom] = rows.get('true', 0.0)
            continue
        n = int(rest[0])
        dist = [0.0] * 2 ** n
        for value, p in rows.items():
            bits = re.findall(r'true|false', value)
            if len(bits) != n:
                return {'status': 'error', 'seconds': total,
                        'error': f'{tool}: row {value} of {file} is not {n} Booleans'}
            dist[int(''.join('1' if b == 'true' else '0' for b in bits), 2)] += p
        for u, p in enumerate(dist):
            answers[f'{atom}({u})'] = p
    return {'status': 'ok', 'seconds': total, 'answers': answers}


def dice_source(args, name):
    """The Dice program behind a `dice` or `nodice` instance (its source), an
    instance of a generator of DICE_WRITERS (beside it) or a network (the
    network and its observations, a Markov network's factors among them,
    with its queried variables returned as a tuple, gen_bn.py --format
    dice; not a program that asks nothing), as a path; None for other
    families."""
    _, gargs = instances(args)[name]
    if gargs[0] == 'dice2problog.py':
        return gargs[1]
    if gargs[0] in DICE_WRITERS:
        return os.path.join(args.dir, name + '.dice')
    if gargs[0] == 'gen_bn.py' and '--query none' not in ' '.join(gargs):
        out = os.path.join(args.dir, f'{name}.dice')
        if out not in DICE_WRITTEN:   # once a run, though every system asks (an instance's runs share a lane)
            with open(out, 'w') as f:
                f.write(subprocess.run([sys.executable, os.path.join(HERE, 'gen_bn.py'), *gargs[1:], '--format', 'dice'],
                                       capture_output=True, text=True, check=True).stdout)
            DICE_WRITTEN.add(out)
        return out
    return None


DICE_WRITTEN = set()


def parse_alea(out, tag='alea'):
    """The distributions a dice2alea.py script prints (a dice2roulette.py
    program, with tag `roulette`), one per component of the program's value
    ({value: probability}), and its `alea_seconds` and
    `alea_compile_seconds`."""
    comps, times = {}, {}
    for line in out.splitlines():
        f = line.split()
        if len(f) == 4 and f[0] == tag:
            comps.setdefault(int(f[1]), {})[f[2]] = float(f[3])
        elif len(f) == 2 and f[0] in ('alea_seconds', 'alea_compile_seconds'):
            times[f[0]] = float(f[1])
    return [comps[k] for k in sorted(comps)], times


def alea_answers(qs, comps):
    """The answers to the program's queries `qs` from Alea's distributions,
    in order: a Boolean component answers one query, its probability of
    true; an integer component answers the queries that follow with one
    predicate, the k-th its probability of value k (the values of a `dice`
    instance's integer, of a network's variable in declaration order).
    ValueError where the components and the queries do not line up."""
    answers, i = {}, 0
    for comp in comps:
        if i >= len(qs):
            raise ValueError(f'{len(comps)} components and {len(qs)} queries')
        if 'true' in comp:
            answers[re.sub(r'\s+', '', qs[i])] = comp['true']
            i += 1
            continue
        pred, k = qs[i].split('(')[0], 0
        while i < len(qs) and '(' in qs[i] and qs[i].split('(')[0] == pred and \
                re.sub(r'\s+', '', qs[i]) not in answers:
            if str(k) not in comp:
                raise ValueError(f'query {qs[i]} asks value {k} of a component with values 0 to {len(comp) - 1}')
            answers[re.sub(r'\s+', '', qs[i])] = comp[str(k)]
            i, k = i + 1, k + 1
        if k == 0:
            raise ValueError(f'query {qs[i]} for an integer component')
    if i != len(qs):
        raise ValueError(f'{len(comps)} components answer {i} of {len(qs)} queries')
    return answers


def run_alea(args, name, path):
    """Alea.jl on the Dice program behind a `dice`, `nodice`, roulette, geni or `bn` instance
    (`dice_source`), translated by dice2alea.py, or on the Alea.jl program
    of an instance written from one (`run_alea_program`): a network as one
    program whose value is the tuple of its queried variables, all answered
    by one `pr`. The translation is not timed, but bounded by the timeout (an
    error past it, so that a lane cannot stall); the seconds are the Julia process's,
    and `inproc_seconds` what it reports for building the program and `pr`
    (`jit_seconds` of it Julia's compilation)."""
    family, gargs = instances(args)[name]
    if gargs[0] == 'alea_bench.jl':
        return run_alea_program(args, gargs[1:])
    source = dice_source(args, name)
    if source is None:
        return {'status': 'error', 'seconds': 0, 'error': f'no Dice program for family {family}'}
    jl = os.path.abspath(os.path.join(args.dir, f'{name}.alea.jl'))
    try:
        t = subprocess.run([sys.executable, os.path.join(HERE, 'dice2alea.py'), source], capture_output=True,
                           text=True, timeout=args.timeout)
    except subprocess.TimeoutExpired:
        return {'status': 'error', 'seconds': 0, 'error': f'dice2alea.py: no translation within {args.timeout:g} s'}
    if t.returncode != 0:
        return {'status': 'error', 'seconds': 0, 'error': 'dice2alea.py: ' + t.stderr.strip()[-2000:]}
    with open(jl, 'w') as f:
        f.write(t.stdout)
    r = timed(args.alea.format(file=jl, dir=os.path.dirname(jl), base=os.path.basename(jl)).split(), args.timeout)
    if r['status'] != 'ok':
        return r
    comps, times = parse_alea(r['stdout'])
    try:
        answers = alea_answers(queries(path), comps)
    except ValueError as x:
        return {'status': 'error', 'seconds': r['seconds'], 'error': f'alea: {x}'}
    stats = {}
    if 'alea_seconds' in times:
        stats['inproc_seconds'] = round(times['alea_seconds'], 3)
    if 'alea_compile_seconds' in times:
        stats['jit_seconds'] = round(times['alea_compile_seconds'], 3)
    return {'status': 'ok', 'seconds': r['seconds'], 'answers': answers, 'stats': stats}


def alea_command(args, mode, program, prefix=None, warm_below=None):
    """The command that runs alea_bench.jl under the `--alea` template on
    `program`, an instance's generator arguments after `alea_bench.jl`:
    the program's path, then the NAME=VALUE it is run with (`--set`)."""
    cmd = args.alea.format(file=ALEA_BENCH, dir=HERE, base=os.path.basename(ALEA_BENCH)).split() + [mode]
    if warm_below is not None:
        cmd += ['--warm-below', f'{warm_below:g}']
    for setting in program[1:]:
        cmd += ['--set', setting]
    return cmd + [program[0]] + ([prefix] if prefix else [])


def circuit_size(path):
    """The flips and gates of the circuit alea_bench.jl wrote as the
    ProbLog program `path`, from its header (`% N flips, M gates, ...`)."""
    with open(path) as f:
        for line in itertools.islice(f, 3):
            m = re.match(r'% (\d+) flips, (\d+) gates,', line)
            if m:
                return {'flips': int(m.group(1)), 'gates': int(m.group(2))}
    return {}


def run_alea_program(args, program):
    """Alea.jl on an Alea.jl program (an instance's generator arguments
    after `alea_bench.jl`, as `alea_command` takes them), by
    alea_bench.jl: the answers it prints (`alea_q ATOM P`), named as the
    queries of the circuit's ProbLog program are. Besides the seconds of
    the Julia process, `inproc_seconds` is what it reports for building
    the program and `pr` the first time, `jit_seconds` Julia's compilation
    within it, and `warm_seconds` the second time, without compilation,
    run when the first took less than a third of the timeout; then, where
    the second took less than a sixth, the sizes of the BDDs `pr` compiles
    (`bdd_evidence`, `bdd_query_max`, `bdd_joint_max`, `bdd_peak`, in
    nodes). `printed`
    keeps the first 100 lines it prints besides the answers."""
    r = timed(alea_command(args, 'run', program, warm_below=args.timeout / 3), args.timeout)
    if r['status'] != 'ok':
        return r
    answers, stats, printed = {}, {}, []
    keys = {'alea_seconds': 'inproc_seconds', 'alea_compile_seconds': 'jit_seconds',
            'alea_warm_seconds': 'warm_seconds'}
    sizes = {'alea_bdd_evidence': 'bdd_evidence', 'alea_bdd_query_max': 'bdd_query_max',
             'alea_bdd_joint_max': 'bdd_joint_max', 'alea_bdd_peak': 'bdd_peak'}
    for line in r['stdout'].splitlines():
        f = line.split()
        if len(f) == 3 and f[0] == 'alea_q':
            answers[f[1]] = float(f[2])
            continue
        if len(f) == 2 and f[0] in keys:
            stats[keys[f[0]]] = round(float(f[1]), 3)
        elif len(f) == 2 and f[0] in sizes:
            stats[sizes[f[0]]] = int(f[1])
        if len(printed) < 100:
            printed.append(line)
    # What Alea prints besides the answers (expectations, values of worlds,
    # times, sizes), the first 100 lines.
    stats['printed'] = printed
    return {'status': 'ok', 'seconds': r['seconds'], 'answers': answers, 'stats': stats}


ROULETTE_DATA = os.path.join(ROOT, 'data', 'roulette')
ROULETTE_TIME = re.compile(r'cpu time: (\d+) real time: (\d+) gc time: (\d+)')


def roulette_program(args, name, variant):
    """The Racket program and arguments Roulette runs an instance with. On
    --suite roulette, the artifact's program as data/roulette's driver
    prints its answers in full (`roulette_q ATOM P`; a network's
    `roulette_v VALUE P`), or the artifact's own hmm.rkt, which prints its
    one answer on its last line; on the other suites, the instance's Dice
    program translated by dice2roulette.py (`roulette_translate`), which
    prints each component's distribution (`roulette K VALUE P`). Under the
    variant `nocomplement` caesar and the translated programs query with
    `(query e #f)` (data/roulette/caesar.rkt, dice2roulette.py); None for
    the artifact's other programs, which it does not apply to."""
    if args.suite != 'roulette':
        return os.path.abspath(os.path.join(args.dir, f'{name}.roulette.rkt')), \
            ['--no-complement'] if variant == 'nocomplement' else []
    family, gargs = instances(args)[name]
    if variant == 'nocomplement' and gargs[:2] != ['gen_roulette.py', 'caesar']:
        return None
    if family == 'bn':
        return os.path.join(ROULETTE_DATA, 'bayes.rkt'), [name.removeprefix('rl_bn_')]
    program, size = gargs[1], gargs[2]
    if program.startswith('hmm-'):
        return os.path.join(args.roulette_benchmarks, 'hmm.rkt'), ['roulette', program[4:], size]
    return os.path.join(ROULETTE_DATA, program + '.rkt'), [size] + (['--no-complement'] if variant == 'nocomplement' else [])


def roulette_command(args, prog, *words):
    """The `--roulette` template on program `prog`; with words, Racket
    runs them on it instead (`-l- raco make`)."""
    file = ' '.join(words + (prog,))
    return args.roulette.format(file=file, dir=os.path.dirname(prog), base=os.path.basename(prog)).split()


def roulette_translate(args, name, rkt):
    """The instance's Dice program (`dice_source`) translated by
    dice2roulette.py into rkt and compiled by `raco make`, as the artifact's
    programs are before they run: neither is timed, both are bounded by the
    timeout. An error record where either fails, else None."""
    source = dice_source(args, name)
    if source is None:
        return {'status': 'error', 'seconds': 0, 'error': f'no Dice program for {name}'}
    try:
        t = subprocess.run([sys.executable, os.path.join(HERE, 'dice2roulette.py'), source], capture_output=True,
                           text=True, timeout=args.timeout)
    except subprocess.TimeoutExpired:
        return {'status': 'error', 'seconds': 0, 'error': f'dice2roulette.py: no translation within {args.timeout:g} s'}
    if t.returncode != 0:
        return {'status': 'error', 'seconds': 0, 'error': 'dice2roulette.py: ' + t.stderr.strip()[-2000:]}
    with open(rkt, 'w') as f:
        f.write(t.stdout)
    c = timed(roulette_command(args, rkt, '-l-', 'raco', 'make'), args.timeout)
    if c['status'] != 'ok':
        return {'status': 'error', 'seconds': 0, 'error': f"raco make: {c['status']} {c.get('error', '')[-2000:]}"}
    return None


def run_roulette(args, name, path, variant):
    """Roulette (`--roulette`, a command template for Racket with Roulette
    installed, {file} the program) on an instance (`roulette_program`). The
    answers are keyed as the ProbLog program's queries; Roulette leaves out
    the values of probability 0, which answer 0, and prints the time of
    building and querying the program (`(time ...)`), kept as
    `inproc_seconds`, without Racket's start and loading. Values that
    answer no query (network.rkt's #t, at a probability of 0 up to
    rounding) are kept under `other`."""
    family, gargs = instances(args)[name]
    prog, rest = roulette_program(args, name, variant)
    if args.suite != 'roulette':
        err = roulette_translate(args, name, prog)
        if err:
            return err
    r = timed(roulette_command(args, prog) + rest, args.timeout)
    if r['status'] != 'ok':
        return r
    printed, values, stats, last = {}, {}, {}, None
    for line in r['stdout'].splitlines():
        f = line.split()
        m = ROULETTE_TIME.match(line)
        if m:
            stats['inproc_seconds'] = int(m.group(2)) / 1000
        elif len(f) == 3 and f[0] == 'roulette_q':
            printed[f[1]] = float(f[2])
        elif len(f) == 3 and f[0] == 'roulette_v':
            values[f[1]] = float(f[2])
        elif len(f) == 1:
            try:
                last = float(f[0])
            except ValueError:
                pass
    if args.suite != 'roulette':
        try:
            printed = alea_answers(queries(path), parse_alea(r['stdout'], 'roulette')[0])
        except ValueError as x:
            return {'status': 'error', 'seconds': r['seconds'], 'error': f'roulette: {x}'}
    elif family == 'bn':
        import gen_bn
        var = gen_bn.ident(gargs[3])
        printed = {f'{var}({gen_bn.ident(v)})': p for v, p in values.items()}
    elif gargs[1].startswith('hmm-') and last is not None:
        printed = {'q': last}
    qs = [re.sub(r'\s+', '', q) for q in queries(path)]
    if not printed:
        tail = (r.get('stderr', '').strip() or r['stdout'].strip()).splitlines()
        return {'status': 'error', 'seconds': r['seconds'], 'error': 'no answers: ' + (tail[-1] if tail else 'empty output')}
    other = {a: p for a, p in printed.items() if a not in qs}
    if other:
        stats['other'] = other
    return {'status': 'ok', 'seconds': r['seconds'], 'answers': {q: printed.get(q, 0.0) for q in qs}, 'stats': stats}


# --- Systems for probabilistic programs (rule 15: every exact method with an
# implementation is a baseline). Each is a command template given by
# `--NAME CMD` ({file}, {dir}, {base}), runs the instances it accepts, and
# answers the instance's queries; PROGRAM_SYSTEMS maps NAME to its
# `accepts(args, name)` and `run(args, name, path)`. The translation of the
# instance's program into the system's language is untimed, as dice2alea.py's.

PROGRAM_SYSTEMS = {}


def program_system(name, help, accepts, run):
    PROGRAM_SYSTEMS[name] = {'help': help, 'accepts': accepts, 'run': run}


def program_command(args, name, file):
    return getattr(args, name).format(file=file, dir=os.path.dirname(file), base=os.path.basename(file)).split()


# The exit status of a translator (dice2genfer.py, dice2sppl.py) whose
# target language cannot express the program: the instance is recorded as
# `unsupported`, with the translator's reason, and counts as unanswered.
REFUSED = 3


def translation(args, script, argv):
    """(the output of `script ARGV`, None) bounded by the timeout, or (None,
    the record of its failure)."""
    try:
        t = subprocess.run([sys.executable, os.path.join(HERE, script), *argv], capture_output=True, text=True,
                           timeout=args.timeout)
    except subprocess.TimeoutExpired:
        return None, {'status': 'error', 'seconds': 0, 'error': f'{script}: no translation within {args.timeout:g} s'}
    if t.returncode != 0:
        status = 'unsupported' if t.returncode == REFUSED else 'error'
        return None, {'status': status, 'seconds': 0, 'error': f'{script}: ' + t.stderr.strip()[-2000:]}
    return t.stdout, None


def translated(args, script, argv, out, what):
    """Writes the translation of an instance (`script ARGV`, its output or the
    file OUT it writes) bounded by the timeout; the error record on failure, else None."""
    stdout, err = translation(args, script, argv)
    if err:
        return err
    if out is not None and stdout:
        with open(out, 'w') as f:
            f.write(stdout)
    return None


# --- Dice programs through a translation --------------------------------------
# Genfer, SPPL, Geni, Psi and Pluck run, besides their own programs, on every
# instance that has Dice programs: the program behind the instance
# (`dice_source`) or the Dice programs of an Alea.jl program's circuit
# (NAME.dicemap), each translated into the system's language (dice2genfer.py,
# dice2sppl.py, dice2geni.py, dice2psi.py, dice2pluck.py); noDice reads them
# as Dice does.
# A program the language cannot express is refused by the translator
# (REFUSED) and recorded as `unsupported`.

def accepts_translated(args, name):
    return instances(args)[name][1][0] == 'alea_bench.jl' or dice_source(args, name) is not None


def run_translated(args, name, path, system, masses):
    """SYSTEM on the Dice programs of an instance, through masses(args,
    label, source, joint, budget), which runs the translation of the Dice
    program `source` (its files named after `label`) within `budget`
    seconds and returns (the distributions of the components of its value,
    each {value: probability} with a Boolean's values `true` and `false`,
    {'seconds': S, ...}) or (None, the record of its failure). The program
    behind the instance answers the ProbLog program's queries
    (alea_answers). Of a circuit's, as `run_dice_circuit` reads them, a
    `bool` program answers its atom with the probability of true and a
    `joint` one, its value N Booleans, is asked as the int whose bits they
    are (the translators' --joint), PRED(U) the mass of U. The seconds add
    up; `inproc_seconds` too, where the system gives them."""
    total, inproc, answers = 0.0, None, {}

    def ask(label, source, joint):
        nonlocal total, inproc
        comps, r = masses(args, label, source, joint, max(0.0, args.timeout - total))
        total += r['seconds']
        if 'inproc_seconds' in r.get('stats', {}):
            inproc = (inproc or 0.0) + r['stats']['inproc_seconds']
        return comps, r
    if instances(args)[name][1][0] == 'alea_bench.jl':
        for line in open(os.path.join(args.dir, name + '.dicemap')):
            file, kind, atom, *rest = line.split()
            comps, r = ask(file, os.path.abspath(os.path.join(args.dir, file)), kind == 'joint')
            if comps is None:
                return {**r, 'seconds': total}
            want = ['true'] if kind == 'bool' else [str(u) for u in range(2 ** int(rest[0]))]
            if len(comps) != 1 or any(v not in comps[0] for v in want):
                return {'status': 'error', 'seconds': total, 'error': f'{system}: {file} gives '
                        f'{[sorted(c) for c in comps]}, not the {kind} value of its dicemap line'}
            if kind == 'bool':
                answers[atom] = comps[0]['true']
            else:
                for u, v in enumerate(want):
                    answers[f'{atom}({u})'] = comps[0][v]
    else:
        comps, r = ask(name, dice_source(args, name), False)
        if comps is None:
            return {**r, 'seconds': total}
        try:
            answers = alea_answers(queries(path), comps)
        except ValueError as x:
            return {'status': 'error', 'seconds': total, 'error': f'{system}: {x}'}
    rec = {'status': 'ok', 'seconds': total, 'answers': answers}
    if inproc is not None:
        rec['stats'] = {'inproc_seconds': round(inproc, 3)}
    return rec


def by_component(system, script, ext, flags, masses_of):
    """The `masses` of run_translated for a system that answers one variable
    a run (Genfer, Geni, Psi): SCRIPT lists the components of the program's
    value (`--components`: `bool` or `int W`) and writes a program for each,
    run with the arguments flags(n) for its n values (2 for a Boolean);
    masses_of(stdout, n) reads a run's {value: probability}, which must
    cover the n values."""
    def masses(args, label, source, joint, budget):
        j = ['--joint'] if joint else []
        listing, err = translation(args, script, [source, '--components'] + j)
        if err:
            return None, err
        comps, total = [], 0.0
        for k, line in enumerate(listing.splitlines()):
            ty = line.split()
            n = 2 if ty[0] == 'bool' else 2 ** int(ty[1])
            file = os.path.abspath(os.path.join(args.dir, f'{label}.{system}{k}.{ext}'))
            err = translated(args, script, [source, str(k)] + j, file, system)
            if err:
                return None, {**err, 'seconds': total}
            r = timed(program_command(args, system, file) + flags(n), max(0.0, budget - total))
            total += r['seconds']
            if r['status'] != 'ok':
                return None, {**r, 'seconds': total}
            got = masses_of(r['stdout'], n)
            why = (f'masses of {sorted(got)} for {n} values' if sorted(got) != list(range(n)) else
                   'every value has mass 0 as printed: the observations\' probability is below its precision'
                   if sum(got.values()) == 0 else None)
            if why:
                return None, {'status': 'error', 'seconds': total,
                              'error': f'{system}: {why}: ' + r['stdout'].strip()[-300:]}
            comps.append({'true': got[1], 'false': got[0]} if ty[0] == 'bool' else
                         {str(v): got[v] for v in range(n)})
        return comps, {'seconds': total}
    return masses


GENI_ROW = re.compile(r'Pr\((.*)\) = ([-+0-9.eE]+)\s*$')


def parse_geni(out):
    """Geni's distribution of the program's value, {value: probability}."""
    dist = {}
    for line in out.splitlines():
        m = GENI_ROW.match(line.strip())
        if m:
            dist[m.group(1)] = float(m.group(2))
    return dist


def accepts_geni(args, name):
    return instances(args)[name][1][0] == 'gen_geni.py' or accepts_translated(args, name)


def geni_masses(out):
    """The masses Geni prints for a program of dice2geni.py, {value:
    probability}: a value is a Boolean, 0 or 1, or a tuple of bits, the
    most significant first. Geni does not normalize by the observations'
    probability, so the masses are divided by their sum (left as printed
    where they all print as 0)."""
    masses = {int(''.join(re.findall(r'[01]', v)), 2): p for v, p in parse_geni(out).items()}
    z = sum(masses.values())
    return {v: p / z for v, p in masses.items()} if z > 0 else masses


# a translated Dice program's masses: a run for each component of its value
GENI_MASSES = by_component('geni', 'dice2geni.py', 'gir', lambda n: [], lambda out, n: geni_masses(out))


def run_geni(args, name, path):
    """Geni on the instance's program in the artifact's language. It prints
    the distribution of the program's value, `Pr(V) = P` to seven decimals
    (its probabilities are f32). The grids return a Boolean, which it prints as
    1 or 0, answering their one query with Pr(1); the weather factory returns 1 when the strikes
    have not all happened by the horizon and 0 when they have, so the first
    query (all strike within H days) is Pr(0) and the second, the constant
    false of the Dice program's value, 0. The other instances' Dice programs
    are translated (`run_translated`)."""
    family, gargs = instances(args)[name]
    if gargs[0] != 'gen_geni.py':
        return run_translated(args, name, path, 'geni', GENI_MASSES)
    gir = os.path.abspath(os.path.join(args.dir, f'{name}.geni'))
    err = translated(args, 'gen_geni.py', [*gargs[1:], '--gir', gir, '--geni-benchmarks', args.geni_benchmarks], None,
                     'geni')
    if err:
        return err
    r = timed(program_command(args, 'geni', gir), args.timeout)
    if r['status'] != 'ok':
        return r
    dist = parse_geni(r['stdout'])
    qs = [re.sub(r'\s+', '', q) for q in queries(path)]
    if gargs[1] == 'weather':
        printed = [dist.get('0'), 0.0]
    else:
        printed = [dist.get('1')]
    if len(qs) != len(printed) or None in printed:
        return {'status': 'error', 'seconds': r['seconds'], 'error': f'geni: no answer for {qs} in {sorted(dist)}'}
    return {'status': 'ok', 'seconds': r['seconds'], 'answers': dict(zip(qs, printed))}


program_system('geni', "Geni (gennifer; Li and Zhang, ICFP 2025) on the instances of --suite geni, in its own "
               "language (gen_geni.py --gir), and on the other Dice programs, translated by dice2geni.py; "
               "e.g. 'GENI/bin/gennifer -l -i {file}', the artifact's flags",
               accepts_geni, run_geni)


PSI_TERM = re.compile(r'(?:([^+δ]*)·)?δ\((-?\d+)\)\[')


def parse_psi(out):
    """The distribution Psi prints for a program's value, `p(x) = c·δ(v)[x]+...`,
    as {value: probability}, or None where there is no such line. A coefficient
    is a rational such as `3/10` or a product of them."""
    for line in out.splitlines():
        if not line.startswith('p('):
            continue
        dist = {}
        for coef, value in PSI_TERM.findall(line.split(' = ', 1)[1]):
            expr = coef.replace('·', '*').strip() or '1'
            if not re.fullmatch(r'[0-9/*().\s-]+', expr):
                raise ValueError(f'psi: coefficient {coef!r}')
            dist[int(value)] = dist.get(int(value), 0.0) + eval(expr, {'__builtins__': {}})
        return dist
    return None


def psi_masses(out, n):
    """The masses Psi prints for a program of dice2psi.py, {value:
    probability} over the n values (a value it does not print has mass
    0), or {} where it prints no distribution."""
    try:
        dist = parse_psi(out)
    except ValueError:
        return {}
    return {} if dist is None else {v: dist.get(v, 0.0) for v in range(n)}


# a translated Dice program's masses: a run for each component of its value
PSI_MASSES = by_component('psi', 'dice2psi.py', 'psi', lambda n: [], psi_masses)


def own_psi(args, name):
    return name.startswith('bb_linreg_w') and name in instances(args)


def accepts_psi(args, name):
    return own_psi(args, name) or accepts_translated(args, name)


def run_psi(args, name, path):
    """Psi (Gehr et al., CAV 2016) on its own linear regression (`bb_linreg`,
    run_psi_linreg) and on the other instances' Dice programs, translated by
    dice2psi.py (`run_translated`), one run for each component of a
    program's value (PSI_MASSES): Psi prints a component's distribution in
    exact rationals."""
    if own_psi(args, name):
        return run_psi_linreg(args, name, path)
    return run_translated(args, name, path, 'psi', PSI_MASSES)


def run_psi_linreg(args, name, path):
    """Psi on its own linear regression program (`bb_linreg`: R2's and PSI's
    program, gen_psi_linreg.py), the posterior means of the slope and the
    intercept by `psi --expectation`, one run each, the seconds added. Psi
    answers the continuous model exactly, in rationals; the circuit's bits
    are those of the discretized one, so the answers are keyed `E(a)` and
    `E(b)` and are checked against the conjugate posterior of
    data/alea/psi/README.md, not against the circuit's queries."""
    total, answers = 0.0, {}
    for v in 'ab':
        file = os.path.abspath(os.path.join(args.dir, f'{name}.{v}.psi'))
        err = translated(args, 'gen_psi_linreg.py', [v], file, 'psi')
        if err:
            return err
        cmd = args.psi.replace('{file}', '--expectation {file}')
        r = timed(cmd.format(file=file, dir=os.path.dirname(file), base=os.path.basename(file)).split(),
                  max(0.0, args.timeout - total))
        total += r['seconds']
        if r['status'] != 'ok':
            r['seconds'] = total
            return r
        m = re.search(r'=\s*(\S+)\s*$', r['stdout'].strip())
        try:
            answers[f'E({v})'] = float(fractions.Fraction(m.group(1)))
        except (AttributeError, ValueError):
            return {'status': 'error', 'seconds': total, 'error': 'psi: no expectation: ' + r['stdout'].strip()[-300:]}
    return {'status': 'ok', 'seconds': total, 'answers': answers}


program_system('psi', "Psi (Gehr et al., CAV 2016) on the Dice programs, the circuits' included, translated by "
               "dice2psi.py, and on its own linear regression of the bitblast suite (gen_psi_linreg.py); e.g. "
               "'prlimit --stack=unlimited PSI {file}'", accepts_psi, run_psi)


def accepts_pluck(args, name):
    return instances(args)[name][0] == 'pluck' or accepts_translated(args, name)


PLUCK_BIT = re.compile(r'true|false|(?<![\w.])[01](?![\w.])')


def parse_pluck(out):
    """The distributions Pluck's load_pluck_file prints for a program of
    dice2pluck.py: {K: {value: probability}} for its query cK, a value a
    Boolean (1 or 0) or the list of an int's bits (the most significant
    first) as the int; {} for a query whose observations are impossible.
    A value printed twice (two paths to it) adds up."""
    dists, cur = {}, None
    for line in out.splitlines():
        m = re.fullmatch(r'c(\d+):', line.strip())
        if m and not line[:1].isspace():
            cur = dists.setdefault(int(m.group(1)), {})
            continue
        if not line[:1].isspace():
            cur = None
        if cur is None or not line.strip() or 'impossible constraint' in line:
            continue
        value, p = line.strip().rsplit(None, 1)
        bits = PLUCK_BIT.findall(value)
        if bits:
            v = int(''.join('1' if b in ('true', '1') else '0' for b in bits), 2)
            cur[v] = cur.get(v, 0.0) + float(p)
    return dists


def pluck_masses(args, label, source, joint, budget):
    """The `masses` of run_translated for Pluck: dice2pluck.py writes one
    program with a query per component of the value, and pluck_dice.jl runs
    it once (one Julia start for all the components)."""
    j = ['--joint'] if joint else []
    listing, err = translation(args, 'dice2pluck.py', [source, '--components'] + j)
    if err:
        return None, err
    file = os.path.abspath(os.path.join(args.dir, f'{label}.pluck'))
    err = translated(args, 'dice2pluck.py', [source] + j, file, 'pluck')
    if err:
        return None, err
    r = timed(program_command(args, 'pluck', os.path.join(HERE, 'pluck_dice.jl')) + [file], budget)
    if r['status'] != 'ok':
        return None, r
    dists = parse_pluck(r['stdout'])
    comps = []
    for k, line in enumerate(listing.splitlines()):
        ty = line.split()
        n = 2 if ty[0] == 'bool' else 2 ** int(ty[1])
        got = dists.get(k)
        why = ('no distribution printed' if got is None else 'the observations are impossible as printed'
               if not got else f'values {sorted(got)} for {n}' if max(got) >= n else None)
        if why:
            return None, {'status': 'error', 'seconds': r['seconds'],
                          'error': f'pluck: component {k}: {why}: ' + r['stdout'].strip()[-300:]}
        comps.append({'true': got.get(1, 0.0), 'false': got.get(0, 0.0)} if ty[0] == 'bool' else
                     {str(v): got.get(v, 0.0) for v in range(n)})
    stats = {}
    for line in r['stdout'].splitlines():
        f = line.split()
        if len(f) == 2 and f[0] == 'pluck_seconds':
            stats['inproc_seconds'] = round(float(f[1]), 3)
    return comps, {'seconds': r['seconds'], 'stats': stats}


def run_pluck(args, name, path):
    """Pluck (Bowers et al., PLDI 2025) on its own program of the instance's
    sequence model (pluck_bench.jl, which runs the artifact's program at the
    paper's size), and on the Dice programs of the other instances,
    translated by dice2pluck.py and run by pluck_dice.jl (`run_translated`).
    The `--pluck` template runs Julia on the artifact's Pluck.jl with {file}
    the script and the artifact's directory and the model or the program
    after it: e.g. 'JULIA --project=ART/PluckArtifact.jl/Pluck.jl {file}
    ART'. The sequence model is a Boolean, the instance's one query."""
    family, gargs = instances(args)[name]
    if family != 'pluck':
        return run_translated(args, name, path, 'pluck', pluck_masses)
    model = os.path.basename(gargs[1])[:-3]
    r = timed(program_command(args, 'pluck', os.path.join(HERE, 'pluck_bench.jl')) + [model], args.timeout)
    if r['status'] != 'ok':
        return r
    values, stats = {}, {}
    for line in r['stdout'].splitlines():
        f = line.split()
        if len(f) == 3 and f[0] == 'pluck_v':
            values[f[1]] = values.get(f[1], 0.0) + float(f[2])
        elif len(f) == 2 and f[0] == 'pluck_seconds':
            stats['inproc_seconds'] = round(float(f[1]), 3)
        elif len(f) == 2 and f[0] == 'pluck_warm_seconds':
            stats['warm_seconds'] = round(float(f[1]), 3)
    qs = [re.sub(r'\s+', '', q) for q in queries(path)]
    if len(qs) != 1 or not values:
        return {'status': 'error', 'seconds': r['seconds'], 'error': f'pluck: values {sorted(values)} for queries {qs}'}
    return {'status': 'ok', 'seconds': r['seconds'], 'answers': {qs[0]: values.get('True', values.get('true', 0.0))},
            'stats': stats}


program_system('pluck', "Pluck (Bowers et al., PLDI 2025) on the sequence models of the pluck family, run by "
               "pluck_bench.jl on the artifact's Pluck.jl, and on the other Dice programs, translated by dice2pluck.py "
               "and run by pluck_dice.jl", accepts_pluck, run_pluck)


GENFER_STAT = re.compile(r'^(Expected value:\s+E|Total measure:\s+Z) = (\S+)\s*$')
# p(n)/Z after an observation, p(n) for a program without one
GENFER_MASS = re.compile(r'^(?:Normalized:\s+)?p\((\d+)\)(?: / Z)? = (\S+)\s*$')
# The Dice programs of the standard suite that Genfer's repository has as
# programs of its own (benchmarks/neurips2023/exact/NAME/NAME.sgcl, the
# paper's comparison with Dice, Prodigy and PSI), each returning the one
# Boolean the instance queries.
GENFER_DICE = {'dice_twocoins': 'twoCoins', 'dice_grass': 'grass', 'dice_evidence1': 'evidence1',
               'dice_evidence2': 'evidence2', 'dice_noisyOr': 'noisyOr', 'dice_alarm': 'alarm',
               'dice_murderMystery': 'murderMystery'}


def own_genfer(args, name):
    return name in GENFER_DICE or (name.startswith('bb_population_s') and name in instances(args))


def accepts_genfer(args, name):
    return own_genfer(args, name) or accepts_translated(args, name)


# a translated Dice program's masses: a run for each component of its value,
# with `--limit 2^w` (2 for a Boolean) for the masses of all its values
GENFER_MASSES = by_component('genfer', 'dice2genfer.py', 'sgcl', lambda n: ['--limit', str(n)],
                             lambda out, n: parse_genfer(out)[1])


def parse_genfer(out):
    """Genfer's expectation and its normalized masses p(n)/Z, as floats
    (a rational such as 509/719 is read exactly)."""
    stats, masses = {}, {}
    for line in out.splitlines():
        if m := GENFER_STAT.match(line):
            stats[m.group(1)[-1]] = float(fractions.Fraction(m.group(2)))
        elif m := GENFER_MASS.match(line):
            masses[int(m.group(1))] = float(fractions.Fraction(m.group(2)))
    return stats, masses


def run_genfer(args, name, path):
    """Genfer (Zaiser, Murawski and Ong, NeurIPS 2023) on the programs it has
    as its own. The Dice programs of its comparison with Dice, Prodigy and
    PSI (`GENFER_DICE`) are its shipped .sgcl files in exact rational
    arithmetic (`--rational`), the expectation of the returned Boolean the
    instance's probability. Its population model (`bb_population`), whose
    Poisson rates are not rational, runs in floating point with the program
    gen_genfer.py writes, and its masses p(n)/Z for n up to 1023, the
    program's range, answer the instance's bit queries `eK(J)` (bit J,
    counted from the most significant of the program's 10, is set), as
    Alea.jl's `pr` of the bits does. The other instances' Dice programs are
    translated (`run_translated`)."""
    if not own_genfer(args, name):
        return run_translated(args, name, path, 'genfer', GENFER_MASSES)
    qs = [re.sub(r'\s+', '', q) for q in queries(path)]
    if name in GENFER_DICE:
        program = os.path.join(args.genfer_benchmarks, 'exact', GENFER_DICE[name], GENFER_DICE[name] + '.sgcl')
        flags = ['--rational']
    else:
        program = os.path.abspath(os.path.join(args.dir, f'{name}.sgcl'))
        steps = instances(args)[name][1][-1].split('=')[1]
        err = translated(args, 'gen_genfer.py', ['population', steps], program, 'genfer')
        if err:
            return err
        flags = ['--limit', '1024']
    r = timed(program_command(args, 'genfer', program) + flags, args.timeout)
    if r['status'] != 'ok':
        return r
    stats, masses = parse_genfer(r['stdout'])
    if 'E' not in stats:
        return {'status': 'error', 'seconds': r['seconds'], 'error': 'genfer: no expectation: ' + r['stdout'].strip()[-300:]}
    if name in GENFER_DICE:
        if len(qs) != 1:
            return {'status': 'error', 'seconds': r['seconds'], 'error': f'genfer: {len(qs)} queries'}
        return {'status': 'ok', 'seconds': r['seconds'], 'answers': {qs[0]: stats['E']}}
    total = sum(masses.values())
    if len(masses) != 1024 or abs(total - 1) > 1e-9:
        return {'status': 'error', 'seconds': r['seconds'], 'error': f'genfer: {len(masses)} masses, sum {total}'}
    answers = {}
    for q in qs:
        m = re.fullmatch(r'e\d+\((\d+)\)', q)
        if not m:
            return {'status': 'error', 'seconds': r['seconds'], 'error': f'genfer: query {q} is not a bit'}
        bit = 1 << (10 - int(m.group(1)))
        answers[q] = sum(p for n, p in masses.items() if n & bit)
    return {'status': 'ok', 'seconds': r['seconds'], 'answers': answers, 'stats': {'expectation': stats['E']}}


program_system('genfer', "Genfer (Zaiser, Murawski and Ong, NeurIPS 2023) on the programs of its own that the "
               "suites contain (--genfer-benchmarks) and on the other Dice programs, translated by "
               "dice2genfer.py; e.g. 'GENFER/bin/genfer {file}'", accepts_genfer, run_genfer)


def own_sppl(args, name):
    return name.startswith('bb_markov_n') and name in instances(args)


def accepts_sppl(args, name):
    return own_sppl(args, name) or accepts_translated(args, name)


def sppl_masses(args, label, source, joint, budget):
    """A translated Dice program's masses (`run_translated`): dice2sppl.py's
    program, run by sppl_dice.py, which compiles it once and asks `prob` for
    each value of each component of the program's value, or with `joint`
    each assignment of its Booleans (`sppl_c K VALUE P`). The seconds are
    the process's, its `sppl_seconds` (the compile and the queries)
    `inproc_seconds`."""
    file = os.path.abspath(os.path.join(args.dir, f'{label}.sppl'))
    err = translated(args, 'dice2sppl.py', [source] + (['--joint'] if joint else []), file, 'sppl')
    if err:
        return None, err
    r = timed(program_command(args, 'sppl', os.path.join(HERE, 'sppl_dice.py')) + [file], budget)
    if r['status'] != 'ok':
        return None, r
    comps, stats = {}, {}
    for line in r['stdout'].splitlines():
        f = line.split()
        if len(f) == 4 and f[0] == 'sppl_c':
            comps.setdefault(int(f[1]), {})[f[2]] = float(f[3])
        elif len(f) == 2 and f[0] == 'sppl_seconds':
            stats['inproc_seconds'] = float(f[1])
    return [comps[k] for k in sorted(comps)], {'seconds': r['seconds'], 'stats': stats}


def run_sppl(args, name, path):
    """SPPL (Saad, Rinard and Mansinghka, PLDI 2021) on its own hierarchical
    Markov switching model (`bb_markov`), run by sppl_markov.py as the model's
    notebook runs it: exact on the continuous model, so its answers differ
    from the circuit's, which rounds the normal emissions to a grid, by that
    discretization. The seconds are the process's; it prints the answers as
    `sppl_q QUERY P`. The other instances' Dice programs are translated
    (`run_translated`)."""
    if not own_sppl(args, name):
        return run_translated(args, name, path, 'sppl', sppl_masses)
    steps = instances(args)[name][1][-1].split('=')[1]
    r = timed(program_command(args, 'sppl', os.path.join(HERE, 'sppl_markov.py')) + [steps], args.timeout)
    if r['status'] != 'ok':
        return r
    printed, stats = {}, {}
    for line in r['stdout'].splitlines():
        f = line.split()
        if len(f) == 3 and f[0] == 'sppl_q':
            printed[f[1]] = float(f[2])
        elif len(f) == 2 and f[0] == 'sppl_seconds':
            stats['inproc_seconds'] = round(float(f[1]), 3)
    qs = [re.sub(r'\s+', '', q) for q in queries(path)]
    if sorted(qs) != sorted(printed):
        return {'status': 'error', 'seconds': r['seconds'], 'error': f'sppl: answers {sorted(printed)} for queries {qs}'}
    return {'status': 'ok', 'seconds': r['seconds'], 'answers': printed, 'stats': stats}


program_system('sppl', "SPPL (Saad, Rinard and Mansinghka, PLDI 2021) on its own Markov switching model of the "
               "bitblast suite, run by sppl_markov.py, and on the Dice programs, translated by dice2sppl.py "
               "and run by sppl_dice.py; e.g. 'SPPL/venv/bin/python -I {file}'", accepts_sppl, run_sppl)


program_system('nodice', "noDice (Gürtler and Kaminski, OOPSLA 2026) on the Dice programs, the circuits' included, as "
               "Dice runs them; e.g. 'NODICE/_build/default/bin/nDice.exe {file}'", accepts_translated,
               lambda args, name, path: run_dice(args, name, path, 'nodice'))


def run_pita(args, name, path):
    """PITA (cplint) on the LPAD translation of the program. One swipl
    process consults the program and asks `prob/2` for every query, so the
    seconds cover parsing, transformation and every query, as the other
    systems' do. PITA answers one query at a time and shares nothing between
    them, which is the shape of the system, not of the harness."""
    lpad = os.path.join(args.dir, name + '.lpad.pl')
    with open(lpad, 'w') as f:
        f.write(problog2pita.program(open(path).read(), driver=True))
    packs = ['-g', f"attach_packs('{args.pita_packs}',[duplicate(replace)])"] if args.pita_packs else []
    return timed([args.pita, '-q'] + packs + ['-g', f"consult('{os.path.abspath(lpad)}'), main", '-t', 'halt'],
                 args.timeout)


# aspmc's ProbLog front end recognizes query/1 but reads evidence/1,2 as
# ordinary facts, and exits successfully with unconditional probabilities.
# A program with evidence is given to it with the evidence asked as queries
# (`aspmc_conditioned`), and P(q | e) is P(q and e) / P(e), as Scallop's
# answers are (`parse_scallop`).
ASPMC_E = 'bench_e'
ASPMC_QE = 'bench_qe_'


def aspmc_conditioned(text):
    """The ProbLog program `text` with its evidence asked as queries, or
    None where it observes nothing: its queries and observations dropped,
    `bench_e :- E.` asked, E the conjunction of what it observed, and every
    `query(Q) :- B.` (B optional) replaced by `bench_qe_Q :- B, Q, E.`, asked
    for the same solutions of B, so that P(Q | e) is P(bench_qe_Q) /
    P(bench_e) (`parse_aspmc`)."""
    from lpad2problog import split_clauses, split_top, strip_comments
    kept, queries, observed = [], [], []
    for clause in split_clauses(strip_comments(text)):
        head, body = problog2pita.split_neck(clause)
        m = re.fullmatch(r'\s*(query|evidence)\s*\((.*)\)\s*', head, re.S)
        if not m:
            kept.append(clause + '.')
        elif m.group(1) == 'query':
            if not re.match(r'[a-z]\w*\s*(\(|$)', m.group(2).strip()):
                raise ValueError(f'a query that is not an atom, which aspmc cannot be asked conditioned: {clause}')
            queries.append((' '.join(m.group(2).split()), body and ' '.join(body.split())))
        elif body is not None:
            raise ValueError(f'an observation with a body, which aspmc cannot be asked: {clause}')
        else:
            parts = split_top(' '.join(m.group(2).split()), ',')
            truth = parts[-1].strip() if len(parts) > 1 else 'true'
            atom = ','.join(parts[:-1] if len(parts) > 1 else parts).strip()
            if truth not in ('true', 'false'):
                raise ValueError(f'an observation that is neither true nor false: {clause}')
            observed.append(atom if truth == 'true' else f'\\+{atom}')
    if not observed:
        return None
    e = ', '.join(observed)
    lines = kept + [f'{ASPMC_E} :- {e}.', f'query({ASPMC_E}).']
    for q, body in queries:
        guard = f'{body}, ' if body else ''
        lines += [f'{ASPMC_QE}{q} :- {guard}{q}, {e}.',
                  f'query({ASPMC_QE}{q})' + (f' :- {body}.' if body else '.')]
    return '\n'.join(lines) + '\n'


def parse_aspmc(out):
    """aspmc prints `[RESULT] aspmc: <query>: <probability>`, on stderr.
    For a program asked through `aspmc_conditioned`, the answer to Q is
    P(bench_qe_Q) / P(bench_e), and 'inconsistent' where P(bench_e) is 0."""
    res = {}
    for line in out.splitlines():
        m = re.match(r'\[RESULT\] aspmc:\s+(.*):\s+([0-9.eE+-]+)\s*$', line)
        if m:
            res[re.sub(r'\s+', '', m.group(1))] = float(m.group(2))
    if ASPMC_E not in res:
        return res
    if res[ASPMC_E] == 0:
        return 'inconsistent'
    return {q.removeprefix(ASPMC_QE): p / res[ASPMC_E] for q, p in res.items() if q.startswith(ASPMC_QE)}


# aspmc's grammar reads a probability as 1, 1.0... or 0.digits, and not in
# exponent notation, in which a small one is written (gen_bn.py's 1e-05).
ASPMC_EXPONENT = re.compile(r'(?<![\w.])(\d+(?:\.\d*)?[eE][-+]?\d+)(?=\s*::)')


def aspmc_numbers(text):
    """The ProbLog program `text` with each probability in exponent notation
    written out in positional notation, exactly: the same double."""
    return ASPMC_EXPONENT.sub(lambda m: format(decimal.Decimal(m.group(1)), 'f'), text)


def run_aspmc(args, name, path):
    """aspmc on the ProbLog program, with its evidence asked as queries
    (`aspmc_conditioned`) where it has any and its probabilities in the
    notation it reads (`aspmc_numbers`). It writes `out.cnf` and the
    decomposer's scratch into the working directory, so each instance gets
    one of its own."""
    d = os.path.join(args.dir, 'aspmc-' + name)
    os.makedirs(d, exist_ok=True)
    with open(path) as f:
        text = f.read()
    try:
        conditioned = aspmc_conditioned(text)
    except ValueError as e:
        return {'status': 'error', 'seconds': 0.0, 'error': str(e)}
    program = aspmc_numbers(text if conditioned is None else conditioned)
    if program != text:
        path = os.path.join(d, name + '.pl')
        with open(path, 'w') as f:
            f.write(program)
    python = os.path.abspath(args.aspmc) if os.sep in args.aspmc else args.aspmc
    r = timed([python, '-u', '-m', 'aspmc.main', '-m', 'problog', '-c', '-v', 'result',
               os.path.abspath(path)], args.timeout, cwd=d)
    if r['status'] == 'ok':
        r['answers'] = parse_aspmc(r['stderr'])
    return r


def parse_scallop(out, scl):
    """scli prints `<relation>: {<p>::()}` for a nullary relation, `{}` when
    it has no proof. The query names map back through the comment lines
    `ground2scallop.py` writes, and with evidence the answer is
    `q<i>e / e`."""
    names = dict(re.findall(r'^// query (q\d+) (.*)$', scl, re.M))
    probs = {}
    for line in out.splitlines():
        m = re.match(r'^(\w+): \{(?:([0-9.eE+-]+)::\(\))?\}\s*$', line)
        if m:
            probs[m.group(1)] = float(m.group(2)) if m.group(2) else 0.0
    res = {}
    for q, atom in names.items():
        key = re.sub(r'\s+', '', atom)
        if 'e' in probs:
            res[key] = probs[q + 'e'] / probs['e'] if probs['e'] else float('nan')
        elif q in probs:
            res[key] = probs[q]
    return res


def run_scallop(args, name, path, variant):
    """Scallop on the ground program tddlog prints (`ground2scallop.py`),
    under the top-k-proofs provenance with k from the variant `topk<k>`.
    The seconds are the grounding's and Scallop's together. The answer is
    exact only when k is at least the number of proofs of every tuple on
    the way (`docs/baselines.md`); below that it is a lower bound."""
    k = re.fullmatch(r'topk(\d+)', variant)
    if not k:
        raise SystemExit(f'unknown Scallop variant {variant}')
    g = timed([args.tddlog, '--ground', path], args.timeout)
    if g['status'] != 'ok':
        return g
    scl = ground2scallop.translate(g['stdout'])
    scl_path = os.path.join(args.dir, name + '.scl')
    with open(scl_path, 'w') as f:
        f.write(scl)
    r = timed([args.scallop, '-p', 'topkproofs', '-k', k.group(1), scl_path],
              max(1.0, args.timeout - g['seconds']))
    r['seconds'] += g['seconds']
    if r['status'] == 'ok':
        r['answers'] = parse_scallop(r['stdout'], scl)
    return r


# AceCompile's options as its own usage prints them, with whether each takes
# an argument. The readme spells two of them otherwise; Ace refuses those
# spellings.
ACE_OPTIONS = {'forceC2d': False, 'forceTabular': False, 'd02': False, 'sbk05': False, 'cd05': False,
               'cd06': False, 'dtBnOrder': False, 'dtClauseMinfill': False, 'dtHypergraph': True,
               'noEclause': False, 'Xseed': True, 'Xlog': False}
ACE_README_NAMES = {'noTabular': 'forceC2d', 'dtBnMinfill': 'dtBnOrder'}


def ace_flags(variant):
    """The Ace compile options a variant stands for. `auto` is none, so that
    Ace chooses tabular compilation or c2d by the number of entries its
    elimination order would create; otherwise the variant lists options without their dash, joined
    by `+`, with `=` before an argument: `forceC2d+cd05+dtClauseMinfill`,
    `forceC2d+dtHypergraph=3`, `Xseed=1`, `Xlog` (a circuit evaluated in
    log space, against underflow). The evidence and the normalization are
    the harness's (`run_ace`)."""
    if variant == 'auto':
        return []
    flags = []
    for o in variant.split('+'):
        name, eq, value = o.partition('=')
        if name in ACE_README_NAMES:
            raise SystemExit(f'Ace variant {variant}: -{name} is the readme\'s name; the code\'s is '
                             f'-{ACE_README_NAMES[name]}')
        if name not in ACE_OPTIONS:
            raise SystemExit(f'Ace variant {variant}: -{name} is not a compile method of AceCompile '
                             f'({", ".join(ACE_OPTIONS)})')
        if ACE_OPTIONS[name] != bool(eq and value):
            raise SystemExit(f'Ace variant {variant}: -{name} takes ' + ('an argument' if ACE_OPTIONS[name] else 'none'))
        flags += ['-' + name] + ([value] if value else [])
    return flags


def parse_ace_marginals(text, stdout=''):
    """Ace's `.marginals` file for one evidence set: Pr(e), and per variable
    the line `Pr(X,e) = [p1, ..., pk]`, which despite its name holds the
    posterior of each value of X given e, in the network's order of the
    values. The header has the circuit's size and Ace's own times. A circuit
    compiled with `-Xlog` is evaluated in log space, against underflow: its
    lines are `log_e(Pr(e)) = ...` and `log_e(Pr(X,e)) = [...]`, the
    natural logarithms of the same numbers (`-∞` or `-Infinity` for zero).
    The file gives P(e) (or its logarithm) to six digits, and AceEvaluate's
    standard output, `stdout`, in full, which is taken where it has it.
    Returns log10 P(e), -inf where Ace's P(e) is zero (inconsistent
    evidence, or a P(e) that underflows a double), the posteriors by
    variable, and the header."""
    if len(re.findall(r'^e = ', text, re.M)) != 1:
        raise ValueError('not the marginals of one evidence set')

    def number(x):
        return float(x.strip().replace('∞', 'inf'))
    log = bool(re.search(r'^log_e\(Pr\(e\)\) = ', text, re.M))
    pe = r'^log_e\(Pr\(e\)\) = (\S+)\s*$' if log else r'^Pr\(e\) = (\S+)\s*$'
    full = re.findall(pe, stdout, re.M)
    pe = number(full[0] if len(full) == 1 else re.search(pe, text, re.M).group(1))
    if log:
        rows = {m.group(1): [math.exp(number(x)) for x in m.group(2).split(',')]
                for m in re.finditer(r'^log_e\(Pr\((.*),e\)\) = \[(.*)\]\s*$', text, re.M)}
        log10_pe = pe / math.log(10)
    else:
        rows = {m.group(1): [float(x) for x in m.group(2).split(',')]
                for m in re.finditer(r'^Pr\((.*),e\) = \[(.*)\]\s*$', text, re.M)}
        log10_pe = math.log10(pe) if pe > 0 else -math.inf
    head = {}
    for key, pattern in [('ac_nodes', r'^  Nodes : (\d+)'), ('ac_edges', r'^  Edges : (\d+)'),
                         ('ace_eval_init_ms', r'^  Total Initialization Time \(ms\) : ([0-9.Ee+-]+)'),
                         ('ace_inference_ms', r'^  Total Inference Time \(ms\) : ([0-9.Ee+-]+)')]:
        m = re.search(pattern, text, re.M)
        if m:
            head[key] = (int if key.startswith('ac_') else float)(m.group(1))
    return log10_pe, rows, head


def hugin_for_ace(data):
    """The bytes of a Hugin file as Ace's reader takes them, with the same
    numbers. Its lexer refuses a `+` in a number, in an exponent
    (`0.000e+00`, in pathfinder, diabetes and munin1 to munin4) or as a
    sign, so every `+` before a digit or a point is dropped, outside quoted
    names and `%` comments as gen_bn.py reads them. A number with no digit
    before its point (`.25`) it reads as it is."""
    return re.sub(rb'("[^"]*")|(%[^\n]*)|\+(?=[0-9.])', lambda m: m.group(1) or m.group(2) or b'', data)


def network_answers(rows, network, qs):
    """The marginals of the query atoms `x(v)` of a gen_bn.py program, whose
    names are the network's through `gen_bn.ident`, from a baseline's
    marginals by variable."""
    order, variables, _ = network
    by_ident = {gen_bn.ident(v): v for v in order}
    answers = {}
    for q in qs:
        m = re.fullmatch(r'(\w+)\((\w+)\)', q.replace(' ', ''))
        v = by_ident[m.group(1)]
        answers[q.replace(' ', '')] = rows[v][[gen_bn.ident(u) for u in variables[v]].index(m.group(2))]
    return answers


def run_ace(args, name, path, variant):
    """Ace 3.0 on the network file the program was generated from, with the
    program's observations from the `.inst` file `gen` writes beside it. The
    file is copied (or unzipped) into a scratch directory of the instance,
    since Ace writes the circuit and the marginals next to it, and the copy
    of a Hugin file is written as Ace's reader takes it (`hugin_for_ace`).
    Ace reads a UAI file only as a Bayesian network, so a UAI network is
    given to it as the network gen_bn.py makes of it, written as a Hugin
    file (`gen_bn.write_hugin`): a Markov network's factors are variables
    observed in the `.inst`, as they are in the program.

    Ace multiplies a table's rows as written, and gen_bn.py rescales a row
    that the file's rounding leaves off one (pathfinder's `0.3333333` three
    times), so Ace compiles with `-Xnormalize`, an option of its code the
    readme leaves out, which rescales every row on reading: without it its
    marginals differ from the program's by up to 7e-8 on pathfinder, with
    it they agree to 1e-14. gen_bn.py leaves a row short of one by less
    than 1e-9 as written (two in sachs, three in insurance), and tddlog
    divides such a row by its sum (within 1e-8 of one) as Ace does; no
    network of data/bn, data/bn-large or Ace's benchmarks has a row
    gen_bn.py leaves further from one, so the rescaling matches tddlog's
    reading of the program on all of them.

    An instance with evidence is compiled with it (`-e`), and evaluated with
    the same file. tddlog compiles one program for one evidence set, so its
    time includes what an evidence-specific compile costs, and Ace's should
    too: with `-e` Ace restricts its tables to the evidence before it orders
    the variables and chooses its route, as tddlog conditions its program.
    Ace's evidence-free compile, whose circuit answers any evidence set and
    whose cost its papers spread over many, is not run.

    The two Java processes are timed apart, `compile_s` and `evaluate_s`,
    and `seconds` is their sum. The record also keeps the route Ace took
    (`ace_method`: tabularCompilation or c2dCompilation), the circuit's
    size, P(e), and Ace's own times, which leave out the JVM's start."""
    source = instances(args)[name][1][1]
    inst = os.path.abspath(os.path.join(args.dir, name + '.inst'))
    if not os.path.exists(inst):
        return {'status': 'error', 'seconds': 0.0, 'error': f'no {inst}: rerun bench.py gen'}
    network = gen_bn.parse(source)
    if source.endswith('.uai'):
        network = network[0]   # a Markov network's factors are observed in the .inst
    evidence = gen_bn.read_inst(inst, network)
    with open(path) as program:
        observed = len(re.findall(r'^evidence\(', program.read(), re.M))
    if observed != len(evidence):
        return {'status': 'error', 'seconds': 0.0,
                'error': f'{inst} has {len(evidence)} observations and the program {observed}: rerun bench.py gen'}
    if evidence and 'sbk05' in variant.split('+'):
        # Checked on bn_alarm_ev: compiled with -e, the sbk05 circuit gives
        # P(e) = 0.01503 against 0.02740, and posteriors off by 0.02; without
        # -e, and with the other encodings or the tabular route with it, Ace
        # answers as the clique tree does.
        return {'status': 'error', 'seconds': 0.0,
                'error': "Ace's sbk05 encoding answers wrongly when compiled with evidence (-e)"}
    d = os.path.abspath(os.path.join(args.dir, f'ace-{name}-{variant}'))
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d)
    net = os.path.basename(source).removesuffix('.gz')
    with (gzip.open if source.endswith('.gz') else open)(source, 'rb') as f:
        data = f.read()
    if net.endswith('.uai'):
        net, data = net.removesuffix('.uai') + '.net', gen_bn.write_hugin(network).encode()
    if net.endswith(('.net', '.hugin')):
        data = hugin_for_ace(data)
    with open(os.path.join(d, net), 'wb') as f:
        f.write(data)
    ace = os.path.abspath(args.ace)
    # Ace's unit resolution sorts with a comparator that is not a total
    # order, which the sort of Java 7 and later refuses (munin with its
    # evidence); the property restores the merge sort of the Java it was
    # written for. The JVM's scratch goes to the instance's directory.
    java = [args.ace_java, f'-Xmx{args.ace_heap}', '-Djava.util.Arrays.useLegacyMergeSort=true',
            f'-Djava.io.tmpdir={d}', f'-DACEC2D={ace}/c2d_linux',
            '-classpath', ':'.join(f'{ace}/{jar}' for jar in ['ace.jar', 'inflib.jar', 'jdom.jar'])]
    env = dict(os.environ, LD_LIBRARY_PATH=ace)
    try:
        c = timed(java + ['edu.ucla.belief.ace.AceCompile', '-Xnormalize'] + ace_flags(variant)
                  + (['-e', inst] if evidence else []) + [net], args.timeout, cwd=d, env=env)
        stats = {'compile_s': round(c['seconds'], 3), 'ace_heap': args.ace_heap}
        if c['status'] != 'ok':
            return {**c, 'stats': stats}
        for key, pattern in [('ace_method', r'^Method : (\S+)'), ('ace_compile_total_s', r'^Total Time \(s\) : (\S+)')]:
            m = re.search(pattern, c['stdout'], re.M)
            if m:
                stats[key] = m.group(1) if key == 'ace_method' else float(m.group(1))
        rest = args.timeout - c['seconds']
        e = timed(java + ['edu.ucla.belief.ace.AceEvaluate', net, inst], rest, cwd=d, env=env) if rest > 0 else \
            {'status': 'timeout', 'seconds': 0.0}
        total = c['seconds'] + e['seconds']
        stats['evaluate_s'] = round(e['seconds'], 3)
        if e['status'] != 'ok':
            return {**e, 'seconds': args.timeout if e['status'] == 'timeout' else total, 'stats': stats}
        try:
            with open(os.path.join(d, net + '.marginals')) as f:
                log10_pe, rows, head = parse_ace_marginals(f.read(), e.get('stdout', ''))
            stats.update(head, ace_log10_pe=log10_pe)
            answers = 'inconsistent' if log10_pe == -math.inf else network_answers(rows, network, queries(path))
            # under --pr, log10 Z from the P(e) it prints, as tddlog's from its own
            pr = {'log10_z': log10_pe + program_offset(path)} if getattr(args, 'pr', False) else {}
        except (OSError, ValueError, KeyError, AttributeError) as x:
            return {'status': 'error', 'seconds': total, 'error': f'Ace marginals: {type(x).__name__} {x}',
                    'stats': stats}
        return {'status': 'ok', 'seconds': total, 'answers': answers, 'stats': stats, **pr}
    finally:
        shutil.rmtree(d, ignore_errors=True)   # the circuit of a large network is large


def parse_merlin_marginals(text):
    """Merlin's MAR file: `PR`, `STATUS` with `true` or `false` (evidence
    found inconsistent), then `MAR`, the number of variables and for each
    its domain size and marginals. Returns the marginals by variable, or
    None for inconsistent evidence. Merlin computes in floating point, not
    in logarithms: where the partition function overflows it reports `PR
    inf` with a true status and writes `-nan` for every marginal, which is
    an error here."""
    status = re.search(r'^STATUS\s*\n\s*(\w+)', text, re.M)
    if status and status.group(1) == 'false':
        return None
    tok = text.split('MAR', 1)[1].split()
    rows, pos = [], 1
    for _ in range(int(tok[0])):
        k = int(tok[pos])
        rows.append([float(x) for x in tok[pos + 1:pos + 1 + k]])
        pos += 1 + k
    if not all(math.isfinite(p) for row in rows for p in row):
        raise ValueError('marginals that are not numbers (the partition function overflows a double)')
    return rows


def parse_merlin_pr(text):
    """log10 Z from Merlin's PR file (`-t PR -O uai`): `PR`, the natural
    logarithm of the partition function given the evidence to six decimals
    and the function itself in parentheses, `5.783825 (3.250000e+02)`, then
    `STATUS` and a MAR section that means nothing under `-t PR`. Merlin
    multiplies and sums its messages in doubles and takes the logarithm at
    the roots of the bucket tree: the status is false where the exponential
    of the logarithm underflows, which leaves the logarithm right, and the
    logarithm is `-inf` for inconsistent evidence or a message that
    underflowed, read here as zero, and `inf` where one overflowed, an
    error."""
    m = re.match(r'\s*PR\s*\n\s*(\S+)', text)
    if not m:
        raise ValueError('no PR line')
    ln_z = float(m.group(1))
    if math.isnan(ln_z) or ln_z == math.inf:
        raise ValueError(f'a partition function of {m.group(1)} (a message overflows a double)')
    return ln_z / math.log(10)


def uai_inputs(args, name):
    """The network of a gen_bn.py instance, the UAI file the baselines that
    read UAI files run on and its evidence file: the file the program was
    generated from, or for another format the UAI copy and observations
    `gen` writes beside the program. The network is None where the
    observations are missing."""
    gargs = instances(args)[name][1]
    network = gen_bn.parse(gargs[1])
    if gargs[1].endswith('.uai'):
        return network[0], os.path.abspath(gargs[1]), os.path.abspath(gargs[gargs.index('--evid') + 1])
    model = os.path.abspath(os.path.join(args.dir, name + '.uai'))
    evid = model + '.evid'
    return (network if os.path.exists(evid) else None), model, evid


def run_merlin(args, name, path, variant='bte'):
    """Merlin (Marinescu's libmerlin, BSD) with bucket-tree elimination (`-a
    bte`) along a min-fill order, the exact MAR route of its command line,
    on the UAI file the program was generated from and its evidence file,
    or for a network in another format on the UAI copy and observations
    `gen` writes beside the program. It writes the marginals to six
    decimals, so its answers differ from the others' by up to 5e-7.
    `bte-pr` runs the PR task (`-t PR`) instead and gives `log10_z`
    (`parse_merlin_pr`), its natural logarithm to six decimals, so up to
    2.2e-7 off in log10."""
    task = {'bte': 'MAR', 'bte-pr': 'PR'}[variant]
    network, model, evid = uai_inputs(args, name)
    if network is None:
        return {'status': 'error', 'seconds': 0.0, 'error': f'no {evid}: rerun bench.py gen'}
    d = os.path.abspath(os.path.join(args.dir, f'merlin-{name}'))
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d)
    try:
        r = timed([os.path.abspath(args.merlin), '-f', model, '-e', evid, '-a', 'bte', '-t', task,
                   '-O', 'uai', '-o', os.path.join(d, 'out')], args.timeout, cwd=d)
        if r['status'] != 'ok':
            return r
        width = re.search(r'induced width\s*:\s*(\d+)', r['stdout'])
        stats = {'merlin_width': int(width.group(1))} if width else {}
        try:
            with open(os.path.join(d, 'out.' + task)) as f:
                text = f.read()
            if task == 'PR':
                return {'status': 'ok', 'seconds': r['seconds'], 'log10_z': parse_merlin_pr(text), 'stats': stats}
            rows = parse_merlin_marginals(text)
            answers = 'inconsistent' if rows is None else \
                network_answers(dict(zip(network[0], rows)), network, queries(path))
        except (OSError, ValueError, IndexError, KeyError, AttributeError) as x:
            what = 'partition function' if task == 'PR' else 'marginals'
            return {'status': 'error', 'seconds': r['seconds'], 'error': f'Merlin {what}: {type(x).__name__} {x}',
                    'stats': stats}
        return {'status': 'ok', 'seconds': r['seconds'], 'answers': answers, 'stats': stats}
    finally:
        shutil.rmtree(d, ignore_errors=True)


# The exact network systems that read the UAI file, besides Merlin: each is
# a driver in Merlin's output format (scripts/bn_uai_run.py, libdai_uai.cpp,
# tensorinference_uai.jl, docs/baselines.md sections 12 to 15). The
# variants are the task: the marginals, and `-pr` the partition function.
UAI_SYSTEMS = {
    'agrum': (('jtree',), lambda a: [a.agrum, os.path.join(HERE, 'bn_uai_run.py'), 'agrum']),
    'pgmpy': (('ve',), lambda a: [a.pgmpy, os.path.join(HERE, 'bn_uai_run.py'), 'pgmpy']),
    'libdai': (('jtree',), lambda a: [a.libdai]),
    # the contraction order by simulated annealing (TreeSA, the package's default) or greedily
    'tensorinference': (('treesa', 'greedy'),
                        lambda a: shlex.split(a.tensorinference) + [os.path.join(HERE, 'tensorinference_uai.jl')]),
}


def run_uai_system(args, name, path, system, variant):
    """pyAgrum (junction tree), pgmpy (variable elimination), libDAI (junction
    tree) or TensorInference.jl (tensor network contraction) on the UAI file
    of a gen_bn.py instance (`uai_inputs`), the observations applied by
    slicing the tables. The marginals asked are the program's query atoms
    (rule 19: what the others are asked), and the `-pr` variant gives
    `log10_z`, the natural logarithm the driver writes over ln 10. The
    record keeps the seconds of the inference proper, without the start of
    the interpreter and the libraries."""
    network, model, evid = uai_inputs(args, name)
    if network is None:
        return {'status': 'error', 'seconds': 0.0, 'error': f'no {evid}: rerun bench.py gen'}
    task = 'PR' if variant.endswith('-pr') else 'MAR'
    variant = variant.removesuffix('-pr')
    order = network[0]
    index = {gen_bn.ident(v): i for i, v in enumerate(order)}
    qs = queries(path) if task == 'MAR' else []
    qvars = sorted({index[re.fullmatch(r'(\w+)\((\w+)\)', q.replace(' ', '')).group(1)] for q in qs})
    d = os.path.abspath(os.path.join(args.dir, f'{system}-{name}'))
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d)
    try:
        out = os.path.join(d, 'out')
        cmd = UAI_SYSTEMS[system][1](args) + [model, evid, out, task] + ([','.join(map(str, qvars))] if qvars else [])
        env = dict(os.environ, OMP_NUM_THREADS='1', OPENBLAS_NUM_THREADS='1', MKL_NUM_THREADS='1', BN_VARIANT=variant)
        r = timed(cmd, args.timeout, cwd=d, env=env)
        if r['status'] != 'ok':
            return r
        try:
            with open(out) as f:
                text = f.read()
            secs = re.search(r'^inference_seconds ([0-9.eE+-]+)$', text, re.M)
            stats = {f'{system}_inference_seconds': float(secs.group(1))} if secs else {}
            if task == 'PR':
                return {'status': 'ok', 'seconds': r['seconds'], 'log10_z': parse_merlin_pr(text), 'stats': stats}
            rows = parse_merlin_marginals(text)
            answers = 'inconsistent' if rows is None else \
                network_answers({order[i]: row for i, row in zip(qvars, rows)}, network, qs)
        except (OSError, ValueError, IndexError, KeyError, AttributeError) as x:
            return {'status': 'error', 'seconds': r['seconds'], 'error': f'{system}: {type(x).__name__} {x}'}
        return {'status': 'ok', 'seconds': r['seconds'], 'answers': answers, 'stats': stats}
    finally:
        shutil.rmtree(d, ignore_errors=True)


class ProvSQLServer:
    """A PostgreSQL cluster of its own with ProvSQL loaded, in `d`, on a
    unix socket (no port), its processes pinned to the lane's core as the
    timed commands are (they inherit the affinity)."""

    def __init__(self, prefix, d):
        self.bin = os.path.join(os.path.abspath(prefix), 'pg', 'bin')
        self.d = d
        # a unix socket's path is at most 107 bytes: the socket lives in a short directory of its own
        self.sock = tempfile.mkdtemp(prefix='ps', dir='/tmp')
        cpu = getattr(LANE, 'cpu', None)
        self.pin = ['taskset', '-c', str(cpu)] if cpu is not None else []

    def run(self, cmd, **kw):
        r = subprocess.run(self.pin + cmd, capture_output=True, text=True, **kw)
        if r.returncode:
            raise RuntimeError((r.stderr or r.stdout).strip()[-400:])
        return r

    def start(self):
        data = os.path.join(self.d, 'data')
        self.run([os.path.join(self.bin, 'initdb'), '-D', data, '-A', 'trust'])
        with open(os.path.join(data, 'postgresql.conf'), 'a') as f:
            f.write(f"shared_preload_libraries = 'provsql'\nlisten_addresses = ''\nunix_socket_directories = '{self.sock}'\n"
                    'fsync = off\nsynchronous_commit = off\nfull_page_writes = off\nautovacuum = off\n'
                    'max_wal_size = 4GB\nshared_buffers = 512MB\n')
        self.run([os.path.join(self.bin, 'pg_ctl'), '-D', data, '-l', os.path.join(self.d, 'server.log'), '-w', 'start'])
        self.psql_cmd = [os.path.join(self.bin, 'psql'), '-h', self.sock, '-d', 'postgres', '-X']
        self.run(self.psql_cmd + ['-q', '-v', 'ON_ERROR_STOP=1', '-c', 'CREATE EXTENSION provsql CASCADE'])
        self.run(self.psql_cmd + ['-q', '-v', 'ON_ERROR_STOP=1', '-c', 'SELECT provsql.setup_search_path()'])   # for new sessions

    def stop(self):
        subprocess.run([os.path.join(self.bin, 'pg_ctl'), '-D', os.path.join(self.d, 'data'), '-m', 'immediate', 'stop'],
                       capture_output=True)
        shutil.rmtree(self.sock, ignore_errors=True)


# The programs ProvSQL is given: tuple-independent tables (every fact
# independent) and non-recursive conjunctive rules, which tpch2provsql.py
# writes as tables and views: TPC-H, the probabilistic databases of the lit
# suite (pdb) and its noisy-or networks (noisyor, whose findings are
# evidence). The lit suite's other families are recursive (mnist_sum,
# sickness, mastermind) or read annotated disjunctions (mnist_sum,
# pedigree, mastermind), which a tuple-independent table cannot hold.
PROVSQL_WRITERS = ('gen_tpch.py', 'gen_pdb.py', 'gen_noisyor.py')


def run_provsql(args, name, path):
    """ProvSQL (Senellart, MIT; v1.13.0 on PostgreSQL 16) on a program of
    tuple-independent tables (PROVSQL_WRITERS): `tpch2provsql.py` writes it
    as tables with a probability per tuple and one view per rule predicate,
    loaded into a cluster of its own before the clock (a database holds its
    data), and each query is the SQL `SELECT probability_evaluate(provenance())
    FROM q` with ProvSQL's own choice of evaluation method, timed through psql
    as the other systems' commands are. Under evidence e the answer of q is
    P(q and e) / P(e), two such queries. The record's seconds are the
    queries', within the timeout together."""
    d = os.path.abspath(os.path.join(args.dir, f'provsql-{name}'))
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d)
    server = ProvSQLServer(args.provsql, d)
    try:
        sql, qs, ev = tpch2provsql.translate(open(path).read())
        with open(os.path.join(d, 'load.sql'), 'w') as f:
            f.write(sql)
        try:
            server.start()
            server.run(server.psql_cmd + ['-q', '-v', 'ON_ERROR_STOP=1', '-f', os.path.join(d, 'load.sql')], timeout=args.timeout)
        except (RuntimeError, subprocess.TimeoutExpired) as x:
            return {'status': 'error', 'seconds': 0.0, 'error': f'ProvSQL load: {x}'}
        answers, seconds = {}, 0.0
        for q, query in qs + ([('', ev)] if ev else []):
            r = timed(server.psql_cmd + ['-At', '-c', query], max(0.0, args.timeout - seconds), cwd=d)
            seconds += r['seconds']
            if r['status'] != 'ok':
                return {**r, 'seconds': seconds}
            try:
                # a row of the probability and the provenance token ProvSQL adds; no row: the lineage is empty
                answers[q] = float(r['stdout'].split('|')[0].strip() or 0.0)
            except ValueError as x:
                return {'status': 'error', 'seconds': seconds, 'error': f'ProvSQL answer: {x}'}
        if ev:
            pe = answers.pop('')
            if pe <= 0:
                return {'status': 'error', 'seconds': seconds, 'error': f'ProvSQL: the evidence has probability {pe}'}
            answers = {q: p / pe for q, p in answers.items()}
        return {'status': 'ok', 'seconds': seconds, 'answers': answers}
    finally:
        server.stop()
        shutil.rmtree(d, ignore_errors=True)


# Probabilistic model checkers: Storm through stormpy (scripts/storm_check.py)
# with the explicit model in doubles (`sparse`) or in rationals (`exact`) and
# the model as decision diagrams (`dd`), and PRISM 4.8.1 with its explicit
# (`sparse`) and its default hybrid engines (`-exact` refuses bounded until).
# On Rubicon's weather factory they read its PRISM models
# `weatherfactoryN.prism` (install_modelcheckers.sh fetches them; GPL-3, so
# never shipped) with the property `P=? [F<=H "allStrike"]`: the probability
# that all N factories are on strike within H days, the query of the Dice
# programs `gen_geni.py weather N H` writes. On every other instance with
# Dice programs they read dice2prism.py's Markov chain of each program, one
# a component of its value (`run_translated`).
MODEL_CHECKERS = {'storm': ('sparse', 'exact', 'dd'), 'prism': ('sparse', 'hybrid')}


def weather_size(entry):
    """(factories, days) of a weather factory instance of `GENI`, else None."""
    gargs = entry[1]
    return (int(gargs[2]), int(gargs[3])) if gargs[0] == 'gen_geni.py' and gargs[1] == 'weather' else None


def accepts_model_checker(args, name):
    return weather_size(instances(args)[name]) is not None or accepts_translated(args, name)


def checker_command(args, system, variant, model, props, dice=True):
    """The command of Storm or PRISM on a model and its properties (a file, or
    for PRISM with dice=False one property, -pf). PRISM's iterations on
    dice2prism.py's chains (unbounded reachability; the weather factory's is
    bounded) run to a relative change of 1e-15 and without its default cap of
    10 000 iterations: on an acyclic chain they stop once nothing changes,
    after as many as the chain is long."""
    if system == 'storm':
        return [args.storm, os.path.join(HERE, 'storm_check.py'), model, variant, props]
    cmd = [os.path.join(os.path.abspath(args.prism), 'bin', 'prism'), model] + ([props] if dice else ['-pf', props]) + \
        ['-' + variant, '-javamaxmem', args.prism_heap]
    return cmd + (['-cuddmaxmem', args.prism_heap, '-epsilon', '1e-15', '-maxiters', '1000000000'] if dice else [])


CHECKER_RESULT = {'storm': re.compile(r'^result ([0-9.eE+-]+)(?: (\d+(?:/\d+)?))?\s*$', re.M),
                  'prism': re.compile(r'^Result: ([0-9.eE+-]+)', re.M)}


def checker_results(system, out):
    """The values Storm or PRISM prints for its properties, in order: a
    Fraction where Storm's exact engine gives the rational, else a float."""
    return [fractions.Fraction(m.group(2)) if m.lastindex == 2 and m.group(2) else float(m.group(1))
            for m in CHECKER_RESULT[system].finditer(out)]


def checker_seconds(system, out):
    """The seconds of the inference proper: Storm's building and checking,
    PRISM's model construction and model checking."""
    if system == 'storm':
        m = re.search(r'^inference_seconds ([0-9.eE+-]+)$', out, re.M)
        return float(m.group(1)) if m else None
    times = [float(x) for x in re.findall(r'^Time for model (?:construction|checking): ([0-9.eE+-]+) sec', out, re.M)]
    return sum(times) if times else None


def checker_distribution(system, out, props, ty):
    """The distribution of a component of type ty (`bool`, or `int W`) from
    the output of Storm or PRISM on dice2prism.py's model and properties (the
    file props): value v has P(done and v) / P(done), P(done) 1 where
    nothing is observed. ValueError where the output does not give them."""
    with open(props) as f:
        names = re.findall(r'^"(\w+)":', f.read(), re.M)
    values = checker_results(system, out)
    if len(values) != len(names):
        tail = out.strip().splitlines()[-3:]
        raise ValueError(f'{len(values)} results for {len(names)} properties: {" / ".join(tail)}')
    got = dict(zip(names, values))
    done = got.pop('obs', 1)
    if done == 0:
        raise ValueError('the observations have probability 0')
    if ty[0] == 'bool':
        t = got.get('v1', 0) / done
        return {'true': float(t), 'false': float(1 - t)}
    # the values up to the largest one asked of a number wider than a circuit's joint (16 bits)
    n = 2 ** int(ty[1]) if int(ty[1]) <= 16 else 1 + max((int(v[1:]) for v in got), default=0)
    return {str(v): float(got.get(f'v{v}', 0) / done) for v in range(n)}


# dice2prism.py's translations in this process, by (label, source, joint):
# the components' listing and each component's files, made once for every
# checker and engine, and the seconds they took.
CHAINS = {}


def checker_masses(system, variant):
    """The `masses` of run_translated for Storm or PRISM: dice2prism.py lists
    the components (`--components`) and writes a model and its properties
    for each, which the checker builds once and answers. The translations
    are not timed, as no system's are, but their seconds over all the
    engines are bounded by the time limit: past it the instance is a
    timeout, a chain too large to write."""
    def masses(args, label, source, joint, budget):
        j = ['--joint'] if joint else []
        made = CHAINS.setdefault((label, source, joint), {'seconds': 0.0})

        def once(key, make):
            if key not in made:
                t0 = time.time()
                made[key] = make()
                made['seconds'] += time.time() - t0
            return made[key]

        def slow():
            if made['seconds'] <= args.timeout:
                return None
            return {'status': 'timeout', 'seconds': args.timeout,
                    'error': f'dice2prism.py: the translations took {made["seconds"]:.0f} s, beyond the time limit'}
        listing, err = once('listing', lambda: translation(args, 'dice2prism.py', [source, '--components'] + j))
        if err:
            return None, err
        comps, total, inproc = [], 0.0, 0.0
        for k, line in enumerate(listing.splitlines()):
            base = os.path.abspath(os.path.join(args.dir, f'{label}.dtmc{k}'))  # one for every checker and engine
            err = slow() or once(k, lambda: translated(args, 'dice2prism.py', [source, str(k), '--props', base + '.props']
                                                       + j, base + '.prism', system))
            if err:
                return None, {**err, 'seconds': max(total, err['seconds'])}
            r = timed(checker_command(args, system, variant, base + '.prism', base + '.props'), max(0.0, budget - total),
                      env=dict(os.environ, OMP_NUM_THREADS='1'))
            total += r['seconds']
            if r['status'] != 'ok':
                return None, {**r, 'seconds': total}
            try:
                comps.append(checker_distribution(system, r['stdout'], base + '.props', line.split()))
            except ValueError as x:
                return None, {'status': 'error', 'seconds': total, 'error': f'{system}: {x}'}
            inproc += checker_seconds(system, r['stdout']) or 0.0
        return comps, {'seconds': total, 'stats': {'inproc_seconds': inproc}}
    return masses


def run_model_checker(args, name, path, system, variant):
    """Storm or PRISM (`MODEL_CHECKERS`) on the PRISM model of a weather
    factory instance, the answer of the program's one query; on the other
    instances, on the Markov chains of their Dice programs (`run_translated`)."""
    size = weather_size(instances(args)[name])
    if size is None:
        return run_translated(args, name, path, system, checker_masses(system, variant))
    n, h = size
    model = os.path.join(os.path.abspath(args.weather_models), f'weatherfactory{n}.prism')
    if not os.path.exists(model):
        return {'status': 'error', 'seconds': 0.0, 'error': f'no {model}: run install_modelcheckers.sh'}
    r = timed(checker_command(args, system, variant, model, f'P=? [F<={h} "allStrike"]', dice=False), args.timeout,
              env=dict(os.environ, OMP_NUM_THREADS='1'))
    if r['status'] != 'ok':
        return r
    got = checker_results(system, r['stdout'])
    if not got:
        return {'status': 'error', 'seconds': r['seconds'], 'error': f'{system}: no result in its output'}
    qs = queries(path)
    secs = checker_seconds(system, r['stdout']) if system == 'storm' else None
    # The program's queries are the goal and the constant false of the Dice tuple `(hit(res), false)`: the model
    # checker answers the first, the rest have probability 0 (gen_geni.py weather_reference).
    answers = {q: 0.0 for q in qs}
    answers[qs[0]] = float(got[0])
    return {'status': 'ok', 'seconds': r['seconds'], 'answers': answers,
            'stats': {f'{system}_inference_seconds': secs} if secs is not None else {}}


def decimal_log10(digits):
    """log10 of a positive natural number written in decimal, of any
    length, as an integer shift plus the logarithm of its first 17 digits:
    Python refuses to convert a string of more than 4 300 digits to an int,
    Ganak's fractions run to thousands, and a quotient of two such numbers
    keeps a double's precision only if the shifts cancel before the sum."""
    digits = digits.lstrip('0')
    head = digits[:17]
    return len(digits) - len(head), math.log10(int(head))


def parse_ganak(out):
    """log10 Z from Ganak's `c s exact arb frac N/D` line (its weighted
    count in exact fractions) or `c s exact arb int N` line (its model
    count), `-inf` where it prints `s UNSATISFIABLE`."""
    if re.search(r'^s UNSATISFIABLE\s*$', out, re.M):
        return -math.inf
    m = re.search(r'^c s exact arb (?:frac|int) (\d+)(?:/(\d+))?\s*$', out, re.M)
    if not m:
        raise ValueError('no `c s exact arb frac` or `c s exact arb int` line')
    if not m.group(1).strip('0'):
        return -math.inf
    (shift_n, log_n), (shift_d, log_d) = decimal_log10(m.group(1)), decimal_log10(m.group(2) or '1')
    return (shift_n - shift_d) + (log_n - log_d)


def model_counting(cnf):
    """Whether a DIMACS CNF is an unweighted model counting instance, its
    type line `c t mc`, read up to its
    `p cnf` line; the counters then count models rather than weights."""
    with open(cnf) as f:
        for line in f:
            s = line.split()
            if s[:2] == ['c', 't']:
                return s[2:3] == ['mc']
            if s[:1] == ['p']:
                return False
    return False


def run_ganak(args, name, path):
    """Ganak (Sharma, Roy, Soos and Meel, IJCAI 2019; MIT license), a
    model counter with component caching that shares nothing with tddlog,
    on the weighted CNF `gen` writes beside the program (`has_cnf`):
    weighted counting in exact rationals (`--mode 1`), or on a model
    counting instance (`model_counting`) counting in integers (`--mode 0`),
    without its probabilistic hashing (`--prob 0`), so that the count is
    exact. Gives `log10_z`."""
    cnf = os.path.join(args.dir, name + '.cnf')
    if not os.path.exists(cnf):
        return {'status': 'error', 'seconds': 0.0, 'error': f'no {cnf}: rerun bench.py gen'}
    mode = '0' if model_counting(cnf) else '1'
    r = timed([os.path.abspath(args.ganak), '--mode', mode, '--prob', '0', os.path.abspath(cnf)], args.timeout)
    if r['status'] != 'ok':
        return r
    try:
        return {'status': 'ok', 'seconds': r['seconds'], 'log10_z': parse_ganak(r['stdout'])}
    except ValueError as x:
        return {'status': 'error', 'seconds': r['seconds'], 'error': f'Ganak: {x}'}


def parse_sharpsat_td(out):
    """log10 Z from sharpSAT-TD's `c s log10-estimate` line, the base-10
    logarithm of its weighted count to 16 digits (`-inf` for a count of
    zero), or from its `c s exact arb int N` line, its model count, and
    `-inf` where it prints `s UNSATISFIABLE`."""
    if re.search(r'^s UNSATISFIABLE\s*$', out, re.M):
        return -math.inf
    m = re.search(r'^c s exact arb int (\d+)\s*$', out, re.M)
    if m:
        if not m.group(1).strip('0'):
            return -math.inf
        shift, log = decimal_log10(m.group(1))
        return shift + log
    m = re.search(r'^c s log10-estimate (\S+)\s*$', out, re.M)
    if not m or not re.search(r'^c s exact arb float ', out, re.M):
        raise ValueError('no `c s log10-estimate` and `c s exact arb float` lines')
    z = float(m.group(1))
    if math.isnan(z) or z == math.inf:
        raise ValueError(f'a log10 Z of {m.group(1)}')
    return z


def sharpsat_td_cache_mb():
    """sharpSAT-TD's cache bound in MB (`-cs`, in units of 10^6 bytes),
    as its readme sets it for a memory bound X: X/2 - 500, here with X the
    address-space limit of the run (`ulimit -v`), which its commands
    inherit. Without `-cs` it takes 95% of the machine's free memory,
    whatever the limit, so a run without a limit is refused."""
    import resource
    soft = resource.getrlimit(resource.RLIMIT_AS)[0]
    if soft == resource.RLIM_INFINITY:
        raise SystemExit('--sharpsat-td bounds its cache by half the address-space limit (ulimit -v), and there is '
                         "none; without a bound it takes 95% of the machine's free memory")
    cache = soft // 2_000_000 - 500
    if cache < 100:
        raise SystemExit(f'--sharpsat-td: an address-space limit of {soft} bytes leaves it no cache')
    return cache


def run_sharpsat_td(args, name, path, variant):
    """sharpSAT-TD (Korhonen and Jarvisalo, CP 2021; MIT license), a
    model counter with component caching whose decisions follow a tree
    decomposition, on the weighted CNF `gen` writes beside the program
    (`has_cnf`): `-WE`, weighted counting in MPFR floats of a
    double's 53-bit significand and an unbounded exponent (on a model
    counting instance, `model_counting`, counting in integers, without
    `-WE`), with the tree
    decomposition FlowCutter finds in S seconds (`-decot S`, the variant
    `decot<S>`), which it always spends, weighted 100 in the decision
    heuristic (`-decow 100`, as its readme runs it). It runs FlowCutter
    as `./flow_cutter_pace17`, so it runs in the directory of its binary,
    and writes the primal graph to a scratch directory of the instance.
    It stops on an empty clause, which no network of the UAI sets
    produces (`uai_wcnf.py`). Gives `log10_z`."""
    budget = variant.removeprefix('decot')
    cnf = os.path.abspath(os.path.join(args.dir, name + '.cnf'))
    if not os.path.exists(cnf):
        return {'status': 'error', 'seconds': 0.0, 'error': f'no {cnf}: rerun bench.py gen'}
    binary = os.path.abspath(args.sharpsat_td)
    d = os.path.abspath(os.path.join(args.dir, f'sharpsat-td-{name}-{variant}'))
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d)
    try:
        weighted = [] if model_counting(cnf) else ['-WE']
        r = timed([binary] + weighted + ['-decot', budget, '-decow', '100', '-tmpdir', d,
                   '-cs', str(sharpsat_td_cache_mb()), cnf], args.timeout, cwd=os.path.dirname(binary))
    finally:
        shutil.rmtree(d, ignore_errors=True)
    if r['status'] != 'ok':
        return r
    try:
        return {'status': 'ok', 'seconds': r['seconds'], 'log10_z': parse_sharpsat_td(r['stdout'])}
    except ValueError as x:
        return {'status': 'error', 'seconds': r['seconds'], 'error': f'sharpSAT-TD: {x}'}


def tddlog_flags(variant):
    """The tddlog options a strategy name stands for. A variant is a vtree
    strategy, optionally followed by `+flag` or `+flag=value` pieces that
    become options of their own, so `s+flag=value` runs `--vtree s --flag
    value`."""
    strategy, *opts = variant.split('+')
    flags = ['--vtree', strategy]
    for o in opts:
        name, _, value = o.partition('=')
        flags += ['--' + name] + ([value] if value else [])
    return flags


def tddlog_arms(arms, strategies):
    """{strategy: binary} from `--tddlog-arm STRATEGY=PATH` options: a
    strategy of `strategies` run with a binary of its own (an arm built
    from another commit). A strategy not run, given twice, or a binary that
    is not executable fails here."""
    out = {}
    for a in arms:
        strategy, sep, path = a.partition('=')
        if not sep or not strategy or not path:
            raise SystemExit(f'--tddlog-arm {a}: the form is STRATEGY=PATH')
        if strategy not in strategies:
            raise SystemExit(f'--tddlog-arm {a}: {strategy} is not one of the strategies run '
                             f'({",".join(strategies) or "none"})')
        if strategy in out:
            raise SystemExit(f'--tddlog-arm {a}: {strategy} has a binary already')
        if not os.access(path, os.X_OK):
            raise SystemExit(f'--tddlog-arm {a}: {path} is not an executable')
        out[strategy] = path
    return out


def run(args):
    spec = instances(args)
    names = list(spec)
    if args.pr:
        if args.suite not in ('uai', *EXTRA_SUITES):
            raise SystemExit(f'--pr compares the partition functions of the {" and ".join(["uai", *EXTRA_SUITES])} '
                             f'suite{"s" if EXTRA_SUITES else ""}, and the suite is ' + args.suite)
        if not (args.strategies or args.exact or args.merlin or args.ganak or getattr(args, 'sharpsat_td', '')
                or args.ace or args.kbs or args.pita or args.aspmc or any(getattr(args, s, '') for s in UAI_SYSTEMS)):
            raise SystemExit('--pr compares the partition functions of tddlog, --exact, --ace, --merlin, --agrum, '
                             '--pgmpy, --libdai, --tensorinference, --kbs, --pita, --aspmc, --ganak and '
                             '--sharpsat-td, and none is run')
        if args.scallop:
            raise SystemExit("--scallop's top-k proofs approximate the queries' probabilities, and --pr compares "
                             'exact partition functions')
    if args.suite in EXTRA_SUITES:
        if not args.pr:
            raise SystemExit(f'the {args.suite} programs ask no query: their task is the weighted model count (--pr)')
        given = [o for o, on in [('--kbs', args.kbs), ('--pita', args.pita), ('--aspmc', args.aspmc),
                                 ('--scallop', args.scallop), ('--dice', args.dice), ('--alea', getattr(args, 'alea', '')),
                                 *[('--' + o, getattr(args, o, '')) for o in PROGRAM_SYSTEMS]] if on]
        if given:
            raise SystemExit(f'{", ".join(given)} answer queries, and the {args.suite} programs ask none '
                             "(pass --kbs '' for no ProbLog run)")
    for flag, binary in [('--ganak', args.ganak), ('--sharpsat-td', getattr(args, 'sharpsat_td', ''))]:
        if not binary:
            continue
        if not any(has_cnf(spec[n]) for n in names):
            raise SystemExit(f'{flag} counts the weighted CNFs of the {" and ".join(["uai", *EXTRA_SUITES])} '
                             f'suite{"s" if EXTRA_SUITES else ""}, and none is selected')
        if not args.pr:
            raise SystemExit(f'{flag} answers the partition function alone, which --pr compares')
        if not os.access(binary, os.X_OK):
            raise SystemExit(f'{flag} {binary}: not an executable')
        missing = [n for n in names if has_cnf(spec[n]) and not os.path.exists(os.path.join(args.dir, n + '.cnf'))]
        if missing:
            raise SystemExit(f'{flag}: no CNF for {missing[0]} in {args.dir} ({len(missing)} instances; '
                             f'run bench.py gen{" --cnf" if spec[missing[0]][1][0] == "gen_bn.py" else ""})')
    if getattr(args, 'sharpsat_td', ''):
        if not os.access(os.path.join(os.path.dirname(os.path.abspath(args.sharpsat_td)), 'flow_cutter_pace17'),
                         os.X_OK):
            raise SystemExit(f'--sharpsat-td {args.sharpsat_td} runs ./flow_cutter_pace17 from its own directory, '
                             'and there is none there')
        sharpsat_td_cache_mb()   # no address-space limit fails before the first run
        for budget in args.sharpsat_td_decot:
            # sharpSAT-TD asserts 0.0001 < S < 10000. FlowCutter runs under
            # `timeout`, in a process group of its own that a kill at the
            # timeout does not reach, and a budget that is the whole timeout
            # leaves nothing to count with.
            if not re.fullmatch(r'\d+(\.\d+)?', budget) or not 0.0001 < float(budget) < 10000:
                raise SystemExit(f'--sharpsat-td-decot {budget}: FlowCutter\'s budget is seconds, '
                                 'between 0.0001 and 10000')
            if float(budget) >= args.timeout:
                raise SystemExit(f'--sharpsat-td-decot {budget}: FlowCutter\'s budget is spent in full, and the '
                                 f'timeout is {args.timeout:g} s')
    strategies = [pr_variant(s) for s in args.strategies] if args.pr else args.strategies
    args.tddlog_binary = {pr_variant(s) if args.pr else s: b
                          for s, b in tddlog_arms(getattr(args, 'tddlog_arm', []), args.strategies).items()}
    systems = [('tddlog', s) for s in strategies] + [('problog', k) for k in args.kbs]
    if args.pita:
        systems.append(('pita', 'bdd'))
    if args.aspmc:
        systems.append(('aspmc', 'sharpsat-td'))
    if args.scallop:
        systems += [('scallop', f'topk{k}') for k in args.scallop_k]
    if args.dice:
        systems.append(('dice', 'bdd'))
    if getattr(args, 'alea', ''):
        systems.append(('alea', 'cudd'))
    if getattr(args, 'roulette', ''):
        unknown = [v for v in args.roulette_variants if v not in ('rsdd', 'nocomplement')]
        if unknown:
            raise SystemExit(f'--roulette-variants {unknown[0]}: the variants are rsdd and nocomplement')
        if 'nocomplement' in args.roulette_variants and \
                not any(roulette_program(args, n, 'nocomplement') for n in names):
            raise SystemExit('--roulette-variants nocomplement changes the query of caesar and of the '
                             'translated Dice programs, and none is selected')
        systems += [('roulette', v) for v in args.roulette_variants]
    for opt, ps in PROGRAM_SYSTEMS.items():
        if getattr(args, opt, ''):
            if not any(ps['accepts'](args, n) for n in names):
                raise SystemExit(f'--{opt} runs {ps["help"].split(" on ")[1].split(",")[0]}, and none is selected')
            systems.append((opt, 'default'))
    if args.exact:
        variants = list(dict.fromkeys(EXACT[spec[n][1][0]][0] for n in names if spec[n][1][0] in EXACT))
        if not variants:
            raise SystemExit('--exact answers the networks, the literature families, tpch, biomine, roulette and geni, '
                             'and none is selected')
        systems += [('exact', v) for v in variants]
    if args.ace:
        if not any(spec[n][1][0] == 'gen_bn.py' for n in names):
            raise SystemExit('--ace answers the networks gen_bn.py writes, and none is selected')
        networks = [n for n in names if spec[n][1][0] == 'gen_bn.py']
        if not any(os.path.exists(os.path.join(args.dir, n + '.inst')) for n in networks):
            raise SystemExit(f'--ace reads the observations bench.py gen writes beside each network (NAME.inst), and '
                             f'{args.dir} has none' + (' (gen --pr writes none)' if args.pr else ''))
        missing = [f for f in ['ace.jar', 'inflib.jar', 'jdom.jar', 'c2d_linux'] if not os.path.exists(os.path.join(args.ace, f))]
        if missing:
            raise SystemExit(f'--ace {args.ace}: no {", ".join(missing)} there')
        for v in args.ace_variants:
            ace_flags(v)   # an unknown form fails before the first run
        systems += [('ace', v) for v in args.ace_variants]
    if args.merlin:
        if not any(spec[n][1][0] == 'gen_bn.py' for n in names):
            raise SystemExit('--merlin answers the networks gen_bn.py writes, and none is selected')
        if not os.access(args.merlin, os.X_OK):
            raise SystemExit(f'--merlin {args.merlin}: not an executable')
        systems.append(('merlin', 'bte'))
        if args.pr:
            systems.append(('merlin', 'bte-pr'))
    for system, (variants, _) in UAI_SYSTEMS.items():
        if getattr(args, system, ''):
            if not any(spec[n][1][0] == 'gen_bn.py' for n in names):
                raise SystemExit(f'--{system} answers the networks gen_bn.py writes, and none is selected')
            for variant in variants:
                systems.append((system, variant))
                if args.pr:
                    systems.append((system, variant + '-pr'))
    if getattr(args, 'provsql', ''):
        if not any(spec[n][1][0] in PROVSQL_WRITERS for n in names):
            raise SystemExit('--provsql answers the programs of tuple-independent tables (PROVSQL_WRITERS: TPC-H, pdb, noisyor), and none is selected')
        systems.append(('provsql', 'default'))
    for system, variants in MODEL_CHECKERS.items():
        if getattr(args, system, ''):
            if not any(accepts_model_checker(args, n) for n in names):
                raise SystemExit(f'--{system} answers the weather factory and the instances with Dice programs, '
                                 'and none is selected')
            if any(weather_size(spec[n]) for n in names) and not getattr(args, 'weather_models', ''):
                raise SystemExit(f'--{system} reads the weather factory\'s PRISM models of --weather-models')
            systems += [(system, v) for v in getattr(args, f'{system}_variants') or variants]
    if args.ganak:
        systems.append(('ganak', 'wmc'))
    if getattr(args, 'sharpsat_td', ''):
        systems += [('sharpsat-td', f'decot{s}') for s in args.sharpsat_td_decot]
    rotate = getattr(args, 'rotate', None)
    if rotate is not None and len(systems) < 2:
        raise SystemExit('--rotate orders the systems of an instance, and there is one')
    cpus = getattr(args, 'cpus', [])
    if len(set(cpus)) != len(cpus):
        raise SystemExit(f'--cpus {",".join(map(str, cpus))}: a CPU listed twice')
    if cpus and not shutil.which('taskset'):
        raise SystemExit('--cpus pins the lanes with taskset, and there is none on the PATH')
    # Instances of a family are listed by size. Once a system times out or
    # fails on one, the larger ones of that family are skipped rather than
    # waited out; the family is then a job of one lane.
    families = {}
    for name in names:
        families.setdefault(spec[name][0], []).append(name)
    jobs = [[name] for name in names] if args.no_skip else list(families.values())
    lanes = cpus or [None]
    free = list(lanes)
    lock = threading.Lock()

    def job(out, names):
        with lock:
            LANE.cpu = free.pop()
        try:
            run_job(args, spec, systems, out, lock, names)
        finally:
            with lock:
                free.append(LANE.cpu)

    with open(args.out, 'a') as out, ThreadPoolExecutor(len(lanes)) as pool:
        futures = [pool.submit(job, out, j) for j in jobs]
        try:
            for f in futures:
                f.result()
        except BaseException:
            for f in futures:
                f.cancel()
            stop_all()
            raise


def system_order(systems, name, rotate):
    """The order in which `systems` run on instance `name`: as given, or
    under `--rotate R` rotated by a checksum of the name plus R."""
    if rotate is None:
        return systems
    k = (zlib.crc32(name.encode()) + rotate) % len(systems)
    return systems[k:] + systems[:k]


def run_job(args, spec, systems, out, lock, names):
    """Runs `systems` on `names`, instances of one family in order of size,
    and appends the records to `out` under `lock`. On the suites of
    CHECKED_SUITES a baseline's record is written once every system has run
    on the instance, checked against the reference
    (`check_against_reference`)."""
    def emit(rec):
        extra = f" pairs={rec['pairs']}" if 'pairs' in rec else ''
        if rec.get('query_pairs'):
            extra += f" query={rec['query_pairs']}"
        if 'log10_z' in rec:
            extra += f" log10Z={rec['log10_z']:.6f}"
        with lock:
            out.write(json.dumps(rec) + '\n')
            out.flush()
            print(f"{rec['instance']:10} {rec['system']}:{rec['variant']:10} {rec['status']:8} "
                  f"{rec['seconds']:8.2f}s{extra}", flush=True)

    dead = set()
    checked = getattr(args, 'suite', None) in CHECKED_SUITES
    for name in names:
        path = os.path.join(args.dir, name + '.pl')
        pe = pe_program(path, os.path.join(args.dir, name + '.pe.pl')) \
            if args.pr and any(s in PE_SYSTEMS for s, _ in systems) else None
        done, held = {}, []
        for system, variant in system_order(systems, name, getattr(args, 'rotate', None)):
            if system == 'exact' and EXACT.get(spec[name][1][0], ('',))[0] != variant:
                continue
            if system in ('ace', 'merlin', *UAI_SYSTEMS) and spec[name][1][0] != 'gen_bn.py':
                continue
            if system == 'provsql' and spec[name][1][0] not in PROVSQL_WRITERS:
                continue
            if system in MODEL_CHECKERS and not accepts_model_checker(args, name):
                continue
            if system in ('ganak', 'sharpsat-td') and not has_cnf(spec[name]):
                continue
            if system == 'roulette' and roulette_program(args, name, variant) is None:
                continue
            if system in PROGRAM_SYSTEMS and not PROGRAM_SYSTEMS[system]['accepts'](args, name):
                continue
            if (system, variant, spec[name][0]) in dead:
                rec = {'instance': name, 'family': spec[name][0], 'system': system,
                       'variant': variant, 'status': 'skipped', 'seconds': 0}
                with lock:
                    out.write(json.dumps(rec) + '\n')
                    print(f"{name:10} {system}:{variant:10} skipped (timed out on a smaller instance)")
                continue
            src = pe if pe and system in PE_SYSTEMS else path
            if system == 'dice':
                r = run_dice(args, name, path)
            elif system == 'alea':
                r = run_alea(args, name, path)
            elif system == 'pita':
                r = run_pita(args, name, src)
            elif system == 'aspmc':
                r = run_aspmc(args, name, src)
            elif system == 'scallop':
                r = run_scallop(args, name, path, variant)
            elif system == 'ace':
                r = run_ace(args, name, path, variant)
            elif system == 'merlin':
                r = run_merlin(args, name, path, variant)
            elif system in UAI_SYSTEMS:
                r = run_uai_system(args, name, path, system, variant)
            elif system == 'provsql':
                r = run_provsql(args, name, path)
            elif system in MODEL_CHECKERS:
                r = run_model_checker(args, name, path, system, variant)
            elif system == 'roulette':
                r = run_roulette(args, name, path, variant)
            elif system in PROGRAM_SYSTEMS:
                r = PROGRAM_SYSTEMS[system]['run'](args, name, path)
            elif system == 'ganak':
                r = run_ganak(args, name, path)
            elif system == 'sharpsat-td':
                r = run_sharpsat_td(args, name, path, variant)
            elif system == 'exact':
                script, *rest = EXACT[spec[name][1][0]][1](path)
                if args.pr:
                    rest.append('--evidence-probability')   # the uai suite's, bn_exact.py
                r = timed([sys.executable, os.path.join(HERE, script)] + rest, args.timeout)
            else:
                if system == 'tddlog':
                    binary = args.tddlog_binary.get(variant, args.tddlog)
                    cmd = [binary, '--stats', '--precision', '17'] + tddlog_flags(variant) + [path]
                else:
                    cmd = [args.problog, '-k', variant, src]
                r = timed(cmd, args.timeout)
            rec = {'instance': name, 'family': spec[name][0], 'system': system,
                   'variant': variant, 'status': r['status'], 'seconds': round(r['seconds'], 3)}
            if spec[name][1][0] == 'alea_bench.jl':
                rec.update(circuit_size(path))
            rec.update(r.get('stats', {}))
            if r['status'] == 'ok' and 'log10_z' in r:
                rec['log10_z'] = r['log10_z']   # Merlin's PR task and the counters, and Ace's beside its answers
            if r['status'] == 'ok' and ('answers' in r or 'log10_z' not in r):
                # tddlog under `--no-queries` answers P(e) alone, as Merlin's
                # PR task does.
                asked = not (system == 'tddlog' and 'no-queries' in variant.split('+')[1:])
                if asked:
                    rec['answers'] = r['answers'] if system in ('dice', 'alea', 'aspmc', 'scallop', 'ace', 'merlin',
                                                                'roulette', 'provsql', *MODEL_CHECKERS, *UAI_SYSTEMS,
                                                                *PROGRAM_SYSTEMS) \
                        else parse_answers(r['stdout'])
                if system == 'tddlog':
                    rec.update(parse_stats(r['stderr']))
                if (system == 'tddlog' and 'evidence-probability' in variant.split('+')[1:]) or \
                        (system == 'exact' and args.pr):
                    try:
                        rec['log10_pe'], rec['log10_z'] = partition_function(r['stdout'], path)
                    except (OSError, ValueError) as x:
                        r = {'status': 'error', 'error': f'partition function: {x}'}
                        rec['status'] = 'error'
                        rec.pop('answers', None)
                # A system that exits 0 without printing an answer failed
                # (aspmc does so on a parse error), and a run without
                # answers can be checked against nothing.
                if asked and rec['status'] == 'ok' and not rec['answers'] and queries(path):
                    tail = (r.get('stderr', '').strip() or r.get('stdout', '').strip()).splitlines()
                    r = {'status': 'error', 'error': 'no answers: ' + (tail[-1] if tail else 'empty output')}
                    rec['status'] = 'error'
                    del rec['answers']
                # P(e), asked as a query (pe_program), and log10 Z from it
                if pe and src == pe and rec['status'] == 'ok':
                    p = rec.pop('answers').get(PE_QUERY)
                    try:
                        if p is None:
                            raise ValueError(f'no answer for {PE_QUERY}')
                        rec['log10_pe'] = math.log10(p) if p > 0 else -math.inf
                        rec['log10_z'] = rec['log10_pe'] + program_offset(path)
                    except (OSError, ValueError) as x:
                        r = {'status': 'error', 'error': f'partition function: {x}'}
                        rec['status'] = 'error'
            if r['status'] in ('error', 'unsupported'):
                rec['error'] = r['error']
                # A failed run prints its stats before the error, which
                # ends what it prints.
                if system == 'tddlog':
                    rec.update(parse_stats(r['error']))
                    m = re.search(r'^(?:parse|grounding|vtree|indicator|compilation) error: .*', r['error'], re.M | re.S)
                    if m:
                        rec['error'] = m.group(0)
            # a program the system's language cannot express says nothing of
            # the family's larger ones
            if r['status'] not in ('ok', 'unsupported') and not args.no_skip:
                dead.add((system, variant, spec[name][0]))
            done[(system, variant)] = rec
            if checked and system not in ('tddlog', 'exact') and rec['status'] == 'ok':
                held.append(rec)
            else:
                emit(rec)
        for rec in held:
            check_against_reference(rec, done)
            emit(rec)


def fmt(rec):
    if rec is None:
        return ''
    if rec['status'] == 'ok':
        return f"{rec['seconds']:.2f}"
    if rec['status'] == 'timeout':
        return f">{rec['seconds']:.0f}"
    if rec['status'] == 'skipped':
        return '-'
    if rec['status'] == 'unsupported':
        return 'n/a'
    return 'err'


def fmt_stat(rec, key):
    if rec is None or rec['status'] != 'ok' or key not in rec:
        return ''
    return f"{rec[key]:.2f}"


def load(paths):
    """The records of `paths`, keyed by (instance, system, variant) with a
    later file overriding an earlier one, and the systems in the order they
    first appear."""
    recs = [json.loads(l) for path in paths for l in open(path) if l.strip()]
    by = {}
    for r in recs:
        by[(r['instance'], r['system'], r['variant'])] = r
    systems = []
    for r in recs:
        key = (r['system'], r['variant'])
        if key not in systems:
            systems.append(key)
    return by, systems


def result_names(*records):
    """Known suite order, followed by any names present only in saved results."""
    present = dict.fromkeys(key[0] for by in records for key in by)
    known = SPEC | INDICATORS | LARGE | LIT | TPCH | PPL | dict.fromkeys(ace_name(n) for _, _, n in ACE_NETWORKS) | \
        dict.fromkeys(f'biomine{k}' for k in BIOMINE_K)
    return [n for n in known if n in present] + [n for n in present if n not in known]


def answer_diff(p, q):
    """|p - q|, or infinity where either answer is not a number, which
    would otherwise pass every comparison."""
    d = abs(p - q)
    return d if d == d else math.inf


def pr_diff(a, b):
    """|a - b| of two log10 Z, zero where they are equal (both `-inf` for
    evidence of probability zero), infinity where either is not a number."""
    if a == b:
        return 0.0
    d = abs(a - b)
    return d if d == d else math.inf


def table(args):
    by, systems = load(args.results)
    refs = [tuple(r.split(':')) for r in args.ref.split(',')]
    pr_refs = [tuple(r.split(':')) for r in args.pr_ref.split(',')]
    approx = [s for s in systems if s[0] in args.approx or f'{s[0]}:{s[1]}' in args.approx]
    exact = [s for s in systems if s not in approx]
    names = result_names(by)
    pr = any('log10_z' in r for r in by.values())
    # A system that reports its own time in the process (Alea, without
    # Julia's start) gets a second column for it, and a third for the time
    # of a second run in the process, without compilation.
    extra = {'inproc_seconds': ' in-process', 'warm_seconds': ' warm'}
    shown = {(r['system'], r['variant'], k) for r in by.values() for k in extra if k in r}
    cols = [c for s, v in systems for c in [(s, v, 'seconds')] + [(s, v, k) for k in extra if (s, v, k) in shown]]
    head = ['instance', 'atoms', 'rules'] + [f'{s}:{v}' + extra.get(c, '') for s, v, c in cols] + ['max diff (vs)']
    if approx:
        head.append('approximate: max diff')
    if pr:
        head.append('log10 Z: max diff (vs)')
    print('| ' + ' | '.join(head) + ' |')
    print('|' + '---|' * len(head))

    def max_diff(n, ref, among):
        """The largest difference from the reference's answers, infinite
        where one system finds the evidence inconsistent and the other
        answers."""
        d = None
        for s, v in among:
            r = by.get((n, s, v))
            if not r or r['status'] != 'ok' or 'answers' not in r:
                continue
            if isinstance(r.get('answers'), dict) and isinstance(ref['answers'], dict):
                for q, p in ref['answers'].items():
                    if q in r['answers']:
                        d = max(d or 0.0, answer_diff(p, r['answers'][q]))
            elif r.get('answers') != ref['answers']:
                d = math.inf
        return d

    for n in names:
        row = [n]
        any_td = next((by[k] for k in by if k[0] == n and k[1] == 'tddlog' and 'atoms' in by[k]), None)
        row += [str(any_td['atoms']) if any_td else '', str(any_td['rules']) if any_td else '']
        row += [fmt(by.get((n, s, v))) if c == 'seconds' else fmt_stat(by.get((n, s, v)), c) for s, v, c in cols]
        ref = next((by[(n, rs, rv)] for rs, rv in refs
                    if (n, rs, rv) in by and by[(n, rs, rv)]['status'] == 'ok' and 'answers' in by[(n, rs, rv)]), None)
        has_ref = ref is not None and ref.get('answers') is not None
        d = max_diff(n, ref, exact) if has_ref else None
        row.append(f"{d or 0.0:.1e} ({ref['system']}:{ref['variant']})" if has_ref else '')
        if approx:
            d = max_diff(n, ref, approx) if has_ref else None
            row.append('' if d is None else f'{d:.1e}')
        if pr:
            z = next((by[(n, rs, rv)] for rs, rv in pr_refs
                      if (n, rs, rv) in by and by[(n, rs, rv)]['status'] == 'ok' and 'log10_z' in by[(n, rs, rv)]), None)
            others = [by[(n, s, v)]['log10_z'] for s, v in exact
                      if (n, s, v) in by and by[(n, s, v)]['status'] == 'ok' and 'log10_z' in by[(n, s, v)]]
            d = max((pr_diff(z['log10_z'], x) for x in others), default=0.0) if z else None
            row.append('' if z is None else f"{d:.1e} ({z['system']}:{z['variant']})")
        print('| ' + ' | '.join(row) + ' |')


def compare(args):
    """Two runs of the same instances, one system at a time.

    The seconds are what each run took and the ratio is after over before,
    so under one is faster. `pairs` is the largest formula reported across
    compilation and evidence/query evaluation; `compile_pairs` preserves
    the compilation-only size. Older records without `compile_pairs` may
    omit the evidence/query phase from `pairs`.
    `answers` says whether every query the two runs share came out the same
    to 1e-9, which no change to the portfolio may move.
    """
    before, systems = load(args.before)
    after, _ = load(args.after)
    wanted = [tuple(x.split(':', 1)) if ':' in x else (x, '') for x in args.systems] or systems
    for system, variant in wanted:
        rows, faster, slower, grew, shrank, moved = [], 0, 0, 0, 0, []
        tb = ta = 0.0
        for n in result_names(before, after):
            b, a = before.get((n, system, variant)), after.get((n, system, variant))
            if not b or not a or b['status'] != 'ok' or a['status'] != 'ok':
                if b and a and b['status'] != a['status']:
                    rows.append((n, b['status'], a['status'], '', '', ''))
                continue
            tb, ta = tb + b['seconds'], ta + a['seconds']
            ratio = a['seconds'] / b['seconds'] if b['seconds'] else float('nan')
            faster += ratio < 0.97
            slower += ratio > 1.03
            pb, pa = b.get('pairs'), a.get('pairs')
            grew += pb is not None and pa is not None and pa > pb
            shrank += pb is not None and pa is not None and pa < pb
            same = 'yes'
            if isinstance(b.get('answers'), dict) and isinstance(a.get('answers'), dict):
                d = max((answer_diff(p, a['answers'][q]) for q, p in b['answers'].items()
                         if q in a['answers']), default=0.0)
                if d > 1e-9:
                    same, _ = f'{d:.1e}', moved.append(n)
            rows.append((n, f"{b['seconds']:.2f}", f"{a['seconds']:.2f}", f'{ratio:.2f}',
                         '' if pb == pa else f'{pb} > {pa}', same))
        if not rows:
            continue
        print(f'### {system}:{variant}' if variant else f'### {system}')
        head = ['instance', 'before', 'after', 'after/before', 'pairs', 'answers']
        print('| ' + ' | '.join(head) + ' |')
        print('|' + '---|' * len(head))
        for r in rows:
            print('| ' + ' | '.join(r) + ' |')
        print(f'\n{len(rows)} instances, {faster} faster and {slower} slower by more than 3%, '
              f'{tb:.1f} s against {ta:.1f} s in total; '
              f'{grew} formulas larger and {shrank} smaller; '
              + ('every shared answer agrees to 1e-9.' if not moved
                 else f'answers moved on {", ".join(moved)}.') + '\n')


def ace_defaults(args):
    """Ace's options mean something only with `--ace` (and `--ace-benchmarks`
    only with `--suite ace`): given without it, one is an error rather than
    ignored. Otherwise the defaults are filled in."""
    if args.cmd not in ('gen', 'run'):
        return
    if args.suite != 'ace' and args.ace_benchmarks is not None:
        raise SystemExit('--ace-benchmarks is where --suite ace reads its networks, and the suite is ' + args.suite)
    if args.suite != 'uai' and args.uai_benchmarks is not None:
        raise SystemExit('--uai-benchmarks is where --suite uai reads its networks, and the suite is ' + args.suite)
    if args.suite != 'biomine' and args.biomine_benchmarks is not None:
        raise SystemExit('--biomine-benchmarks is where --suite biomine reads its graph, and the suite is '
                         + args.suite)
    for name, suite in EXTRA_SUITES.items():
        if args.suite != name and getattr(args, name + '_benchmarks', None) is not None:
            raise SystemExit(f'--{name}-benchmarks is where --suite {name} reads its input, and the suite is '
                             + args.suite)
        if getattr(args, name + '_benchmarks', None) is None:
            setattr(args, name + '_benchmarks', os.path.join(ROOT, 'tmp', suite.default_directory))
    if args.suite != 'roulette' and getattr(args, 'roulette_benchmarks', None) is not None:
        raise SystemExit('--roulette-benchmarks is where --suite roulette reads its networks, and the suite is '
                         + args.suite)
    if getattr(args, 'roulette_benchmarks', None) is None:
        args.roulette_benchmarks = os.path.join(ROOT, 'tmp', 'roulette-benchmarks')
    if args.biomine_benchmarks is None:
        args.biomine_benchmarks = os.path.join(ROOT, 'tmp', 'biomine-benchmarks')
    if args.ace_benchmarks is None:
        args.ace_benchmarks = os.path.join(ROOT, 'tmp', 'ace-benchmarks', 'benchmarks_v1.0')
    if args.uai_benchmarks is None:
        args.uai_benchmarks = os.path.join(ROOT, 'tmp', 'uai-benchmarks', 'MAR')
    if args.cmd != 'run':
        return
    given = [f'--{k.replace("_", "-")}' for k in ['ace_variants', 'ace_heap', 'ace_java'] if getattr(args, k) is not None]
    if given and not args.ace:
        raise SystemExit(f'{", ".join(given)} without --ace: Ace is not run')
    if args.sharpsat_td_decot is not None and not args.sharpsat_td:
        raise SystemExit('--sharpsat-td-decot without --sharpsat-td: sharpSAT-TD is not run')
    if getattr(args, 'roulette_variants', None) is not None and not getattr(args, 'roulette', ''):
        raise SystemExit('--roulette-variants without --roulette: Roulette is not run')
    for k, default in [('ace_variants', ['auto']), ('ace_heap', '8g'), ('ace_java', 'java'),
                       ('sharpsat_td_decot', ['1']), ('roulette_variants', ['rsdd'])]:
        if getattr(args, k, None) is None:
            setattr(args, k, default)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest='cmd', required=True)
    g = sub.add_parser('gen')
    g.add_argument('--dir', default=os.path.join(ROOT, 'bench'))
    g.add_argument('--cnf', action='store_true',
                   help="also write each UAI network's weighted CNF with its evidence, for --ganak and --sharpsat-td")
    g.add_argument('--alea', default='', help='command template for Julia with Alea.jl, {file} the script, as '
                   "run's; writes the circuits of the ppl suite's Alea.jl programs with alea_bench.jl")
    g.add_argument('--pr', action='store_true',
                   help="the uai suite's programs for the partition function alone, for run --pr: no queries, and "
                   "a Potts network's colour symmetry broken (gen_bn.py --query none)")
    g.set_defaults(fn=gen)
    r = sub.add_parser('run')
    r.add_argument('--dir', default=os.path.join(ROOT, 'bench'))
    r.add_argument('--out', default='results.jsonl')
    r.add_argument('--tddlog', default=os.path.join(ROOT, 'target', 'release', 'tddlog'))
    r.add_argument('--problog', default=os.path.join(ROOT, 'tmp', 'venv', 'bin', 'problog'))
    r.add_argument('--strategies', default='auto', type=lambda s: s.split(',') if s else [])
    r.add_argument('--tddlog-arm', action='append', default=[], metavar='STRATEGY=PATH',
                   help='run the tddlog strategy STRATEGY (one of --strategies) with the binary PATH '
                   'instead of --tddlog, as an arm built from another commit; repeatable')
    r.add_argument('--kbs', default='ddnnf,sdd,fsdd', type=lambda s: s.split(',') if s else [])
    r.add_argument('--dice', default='', help='command template for Dice with {file}, {dir}, {base}; '
                   'e.g. "docker run --rm -v {dir}:/b sholtzen/dice dice /b/{base}"; empty = not run')
    r.add_argument('--alea', default='', help='command template for Alea.jl with {file}, {dir}, {base}, run on '
                   'the script dice2alea.py writes, or on alea_bench.jl for an Alea.jl program; e.g. '
                   '"julia --project=ALEA_DIR {file}"; empty = not run')
    r.add_argument('--pita', default='', help='swipl binary with the cplint pack; empty = not run')
    r.add_argument('--pita-packs', default='', help='pack directory to attach before consulting')
    r.add_argument('--aspmc', default='', help='python with aspmc importable; empty = not run')
    r.add_argument('--scallop', default='', help='the scli binary; empty = not run')
    r.add_argument('--scallop-k', default='1000', type=lambda s: s.split(','),
                   help='the k of each top-k-proofs run, comma separated')
    r.add_argument('--ace', default='', help="Ace 3.0's directory (ace.jar, inflib.jar, jdom.jar, c2d_linux); "
                   'empty = not run')
    r.add_argument('--ace-variants', type=lambda s: s.split(','),
                   help='Ace compile methods, comma separated: auto (the default), or options joined by + '
                   '(forceC2d+cd05)')
    r.add_argument('--ace-heap', help="the Java heap of Ace's processes (-Xmx), by default 8g")
    r.add_argument('--ace-java', help='the java that runs Ace (Java 8 or later), by default java')
    r.add_argument('--merlin', default='', help="Merlin's command-line binary, for the networks; empty = not run")
    r.add_argument('--agrum', default='', help="a Python with pyAgrum, for the networks (scripts/bn_uai_run.py); empty = not run")
    r.add_argument('--pgmpy', default='', help="a Python with pgmpy, for the networks (scripts/bn_uai_run.py); empty = not run")
    r.add_argument('--libdai', default='', help="the libdai_uai binary (scripts/libdai_uai.cpp, libDAI's junction tree), for the "
                   'networks; empty = not run')
    r.add_argument('--tensorinference', default='', help="the Julia command that runs TensorInference.jl, for the networks, up "
                   "to the script: `julia --startup-file=no --threads=1 --project=DIR`; empty = not run")
    r.add_argument('--storm', default='', help="a Python with stormpy (Storm), for the weather factory of --suite geni "
                   'and the Markov chains dice2prism.py writes of the Dice programs (scripts/storm_check.py); '
                   'empty = not run')
    r.add_argument('--storm-variants', type=lambda s: s.split(','), help='sparse, exact, dd (all by default)')
    r.add_argument('--prism', default='', help="PRISM's directory (bin/prism), for the weather factory and the "
                   'Markov chains of the Dice programs; empty = not run')
    r.add_argument('--prism-variants', type=lambda s: s.split(','), help='the engines sparse, hybrid (both by default)')
    r.add_argument('--prism-heap', default='4g', help="the Java heap of PRISM's process (-javamaxmem), and on the Dice "
                   "programs' chains its decision diagrams' memory (-cuddmaxmem), by default 4g")
    r.add_argument('--weather-models', default='', help="the directory with Rubicon's weatherfactoryN.prism, for --storm and --prism")
    r.add_argument('--provsql', default='', help="the directory install_provsql.sh installs ProvSQL in (pg/bin/initdb, ...), "
                   'for the programs of tuple-independent tables (TPC-H, pdb and noisyor of --suite lit); empty = not run')
    r.add_argument('--roulette', default='', help='command template for Racket with Roulette installed, {file} the '
                   "program: the artifact's on --suite roulette, the Dice program dice2roulette.py translates on "
                   "the others; e.g. \"env PLTUSERHOME=... racket {file}\"; empty = not run")
    r.add_argument('--roulette-variants', type=lambda s: s.split(','),
                   help="Roulette's runs, comma separated: rsdd (the artifact's, the default) and nocomplement "
                   "(caesar's and the translated programs' query without the complement, `(query e #f)`)")
    for opt, ps in PROGRAM_SYSTEMS.items():
        r.add_argument('--' + opt, default='', help=ps['help'] + '; empty = not run')
    r.add_argument('--geni-benchmarks', default=os.path.expanduser('~/local/geni/src/bench/ICFP'),
                   help="the Geni artifact's bench/ICFP directory, where the weather factory's programs are")
    r.add_argument('--genfer-benchmarks', default=os.path.expanduser('~/local/genfer/src/benchmarks/neurips2023'),
                   help="Genfer's benchmarks/neurips2023 directory, where the Dice programs' .sgcl files are")
    r.add_argument('--ganak', default='', help='Ganak, the weighted model counter, for --pr on the CNFs of the '
                   'uai suite; empty = not run')
    r.add_argument('--sharpsat-td', default='', help="sharpSAT-TD's binary, with flow_cutter_pace17 beside it, "
                   'for --pr on the CNFs of the uai suite; empty = not run')
    r.add_argument('--sharpsat-td-decot', type=lambda s: s.split(','),
                   help="sharpSAT-TD's FlowCutter budgets in seconds (-decot), one variant each, comma separated; "
                   'by default 1')
    r.add_argument('--pr', action='store_true',
                   help='on --suite uai, the partition function as well: tddlog with --evidence-probability '
                   '--no-queries (P(e) alone), the clique tree with --evidence-probability, Merlin under -t PR '
                   '(merlin:bte-pr), Ace from the P(e) it prints, ProbLog, PITA and aspmc asked P(e) as a query, and '
                   '--ganak and --sharpsat-td on the weighted CNF of the network')
    r.add_argument('--timeout', type=float, default=300)
    for p in [g, r]:
        p.add_argument('--suite', choices=['standard', 'indicators', 'large', 'lit', 'ace', 'uai', 'tpch', 'biomine',
                                           'ppl', 'bitblast', 'roulette', 'geni', *EXTRA_SUITES], default='standard')
        p.add_argument('--ace-benchmarks', help="where Ace's benchmarks_v1.0.zip is unpacked, for --suite ace; "
                       'by default tmp/ace-benchmarks/benchmarks_v1.0')
        p.add_argument('--uai-benchmarks', help="the directory of the UAI MAR instances (NAME.uai, NAME.uai.evid), "
                       'for --suite uai; by default tmp/uai-benchmarks/MAR')
        p.add_argument('--biomine-benchmarks', help=f'the directory of the Biomine graph {BIOMINE_GRAPH}, for '
                       '--suite biomine; by default tmp/biomine-benchmarks')
        for name, suite in EXTRA_SUITES.items():
            p.add_argument(f'--{name}-benchmarks', help=f'{suite.help}, for --suite {name}; by default '
                           f'tmp/{suite.default_directory}')
        p.add_argument('--roulette-benchmarks', help="the Roulette artifact's benchmark/ directory (its networks, "
                       'bayesian-networks/NAME.bif, and hmm.rkt), for --suite roulette; by default '
                       'tmp/roulette-benchmarks')
        p.add_argument('--families', default='', type=lambda s: s.split(',') if s else [])
        p.add_argument('--only', default='', type=lambda s: s.split(',') if s else [])
    r.add_argument('--no-skip', action='store_true', help='run every instance even after a timeout in the family')
    r.add_argument('--rotate', type=int, default=None, help="rotate the systems' order on each instance by a "
                   "checksum of its name plus R (repetition R of a timed run)")
    r.add_argument('--cpus', default='', type=lambda s: [int(c) for c in s.split(',')] if s else [],
                   help='run in parallel lanes, one per CPU listed, each command pinned to its lane\'s CPU with '
                   'taskset; a family is a job of one lane, or under --no-skip each instance')
    r.add_argument('--exact', action='store_true',
                   help="the exact reference where an instance has one: bn_exact.py's clique-tree answers on the "
                   "networks, lit_exact.py's on the literature families, tpch_exact.py's on the tpch suite, "
                   "reliability_exact.py's on the biomine suite and gen_roulette.py's on the roulette suite")
    r.set_defaults(fn=run)
    t = sub.add_parser('table')
    t.add_argument('results', nargs='+', help='result files; a later file overrides earlier rows for the same (instance, system, variant)')
    t.add_argument('--ref', default='exact:lit,exact:tpch,exact:reliability,exact:dp,problog:ddnnf,problog:fsdd,problog:sdd,'
                   'exact:cliquetree,ace:auto,merlin:bte',
                   help='reference systems in order of preference; the first one that finished is used')
    t.add_argument('--pr-ref', default='exact:cliquetree,merlin:bte-pr,ganak:wmc',
                   help='references for log10 Z, in order of preference; the column is printed where a record '
                   'holds one (run --pr)')
    t.add_argument('--approx', default='scallop', type=lambda s: s.split(',') if s else [],
                   help='systems or system:variant names that approximate; their difference from the '
                   'reference gets a column of its own instead of entering the exact one (default: scallop)')
    t.set_defaults(fn=table)
    c = sub.add_parser('compare')
    c.add_argument('--before', nargs='+', required=True)
    c.add_argument('--after', nargs='+', required=True)
    c.add_argument('--systems', default='', type=lambda s: s.split(',') if s else [],
                   help='system:variant names; the default is every system of the earlier run')
    c.set_defaults(fn=compare)
    f = sub.add_parser('fetch', help='download the networks data/fetch.txt lists and check their SHA-256')
    f.add_argument('--timeout', type=float, default=300, help='seconds a download may take')
    f.set_defaults(fn=fetch)
    args = ap.parse_args()
    ace_defaults(args)
    def terminate(signum, _frame):
        raise SystemExit(128 + signum)

    previous = signal.signal(signal.SIGTERM, terminate)
    try:
        args.fn(args)
    finally:
        signal.signal(signal.SIGTERM, previous)


if __name__ == '__main__':
    main()
