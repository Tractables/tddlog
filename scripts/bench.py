#!/usr/bin/env python3
"""Benchmark tddlog against ProbLog on generated instance families.

    bench.py gen  [--dir bench] [--cnf] [--pr] [--alea CMD] [--tddlog BIN]
                  [--circulette-netlists DIR] [SELECTION]
    bench.py run  [--dir bench] [--out results.jsonl] [--tddlog BIN] [--problog BIN]
                  [--strategies s1,s2] [--kbs ddnnf,sdd,fsdd] [--dice CMD] [--alea CMD] [--timeout S]
                  [--pita SWIPL] [--pita-packs DIR] [--aspmc PYTHON] [--scallop SCLI]
                  [--exact] [--ace DIR [--ace-variants auto,...] [--ace-heap 8g] [--ace-java JAVA]]
                  [--merlin BIN] [--agrum PYTHON] [--pgmpy PYTHON] [--libdai BIN] [--tensorinference JULIA]
                  [--provsql DIR] [--storm PYTHON] [--prism DIR] [--weather-models DIR]
                  [--ganak BIN] [--sharpsat-td BIN [--sharpsat-td-decot 1,10]] [--pr]
                  [--roulette CMD [--roulette-variants rsdd]] [--roulette-tddlog CMD]
                  [--no-skip] [--cpus 2,4,6,...] [--rotate R] [--mem-cap SIZE] [SELECTION]
    bench.py table results.jsonl... [--ref problog:ddnnf,...] [--approx SYSTEM:VARIANT,...] [--pr-ref exact:cliquetree,...]
    bench.py compare --before a.jsonl --after b.jsonl [--systems tddlog:auto,...]
    bench.py fetch

SELECTION is `[--suite standard|indicators|large|lit|ace|uai|tpch|biomine|ppl|bitblast|roulette|geni|circulette
[--ace-benchmarks DIR | --uai-benchmarks DIR | --biomine-benchmarks DIR | --roulette-benchmarks DIR
| --circulette-benchmarks DIR]]
[--families graph,grid,smokers,uwcse,cora,cplint,bn,bnev,dice,...] [--only name,...]`.
A strategy name may carry tddlog flags after a `+`, so `s+flag` is the run
`--vtree s --flag` and names its own column.

`gen` writes the instances from the fixed specification below with the
generators in this directory, so an instance is identified by its name
alone. tddlog reads a network (an instance of `gen_bn.py`), a Dice program
(of `dice2problog.py`) and a cplint LPAD (of `gen_cplint.py`) itself, the
file with a network's queries and observations (`network_args`,
`original`), so that reading it is part of tddlog's time; the program the
systems that read ProbLog programs are given is written by tddlog's
reader, `tddlog --program` (gen's `--tddlog`, `source_tree.program`), the
text the scripts gen_bn.py, dice2problog.py and lpad2problog.py wrote
before the reader replaced them (`gen_network`, `write_program`); the
generators that convert a cplint LPAD they write (gen_uwcse.py,
gen_pedigree.py) call it too. `gen`
decompresses a gzipped network for tddlog once, outside its time, as the
harness does for Ace. The bnlearn networks of data/bn and data/bn-large are not in the
repository (no license for them is established): `fetch` downloads each
file data/fetch.txt lists, checks its SHA-256 and writes it in place, and
`gen` stops, naming `fetch`, where one is missing. `run` executes every (instance, system) pair once, single-threaded,
and appends one JSON line per run: wall seconds, exit status, the answers,
and for tddlog the largest formula size and operation counts from
`--stats`. Once a system runs out of time or memory on one size of a
program (`series`), its larger sizes are skipped for it (`-` in the
table), unless `--no-skip` is given; a failure of another kind, and a
network or another program, skips nothing. Without `--cpus` the runs go
one after another; `--cpus` runs them in parallel lanes, one per CPU
listed, every command of a lane pinned to its CPU with `taskset`. A
program's sizes run in one lane in order of size, so that a skip is as in
a sequential run, and every other instance (every instance under
`--no-skip`) is a job of its own. The
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
(Moy et al., PLDI 2025; neuppl/roulette's main) on its own programs of
`--suite roulette`, on a Bayesian network through its own BN language
(data/roulette/bn.rkt on a BIF file, `roulette_bif`), and otherwise on the
Dice program behind an instance translated by `dice2roulette.py`
(`run_roulette`), timed by roulette_timer.rkt after Racket's start and the
modules' load. `--roulette-tddlog CMD` adds `roulette-tddlog:auto`,
Roulette+tddlog (`run_roulette_tddlog`): Roulette's tddlog engine (the
branch tddlog of Tractables/roulette, a fork of neuppl/roulette) writes the
circuit Roulette builds and tddlog answers it, charged Roulette's run up
to the circuit and tddlog's (docs/baselines.md, section 11). On `--suite
circulette` the template's Racket has Roulette's package roulette-lib and
Circulette installed, and `roulette:rsdd` is circulette.rkt on an instance's
netlist (`run_circulette`); tddlog reads the netlist itself with
`--evidence-probability`, P(fail) with the posteriors, and gen writes the
program its reader writes of it (`tddlog --program NAME.cql`, NAME.pl), which
the systems that read ProbLog programs are given; Roulette+tddlog runs
circulette.rkt on Roulette's tddlog engine with --circulette-tddlog, a
Circulette compiled by the fork's Racket.

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
second time as `merlin:bte-pr` (`-t PR`). ProbLog, PITA, aspmc and Scallop,
which print no P(e), are asked it as the probability of a query that holds when
the evidence does (`pe_program`), and Ace runs as for the marginals. A
record then holds `log10_z`, the base-10 logarithm of Z: for tddlog, which
reads the network, its `Z:` line for a Markov network and its P(e) for a
Bayesian one (`network_partition_function`); for the clique tree, Ace and
the program systems, log10 P(e) plus the constant the program states on
its first line (`gen_bn.read_offset`); for Merlin, the natural logarithm it prints divided by ln 10. `table` adds the
largest difference in log10 Z from the first of `--pr-ref` that finished.
On the uai suite `run` checks each baseline's record against the
reference when the instance's systems have run (`check_against_reference`):
marginals that fail the answer check (`answer_right`: off the reference
by more than 1e-6 or 1e-3 of it, beyond the digits printed) or a log10 Z
further than 1e-6 times max(1, |log10 Z|) make the record a failed run. `gen --pr` writes the
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
import contextlib
import decimal
import fcntl
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
import glob
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

import source_tree  # noqa: E402  (the source tree: its converters, examples and program)
import gen_bn  # noqa: E402
import memcap  # noqa: E402  (the resident memory cap of --mem-cap)
import ground2scallop  # noqa: E402  (Scallop's reading of a circuit)
import problog2scallop  # noqa: E402
import problog2pita  # noqa: E402
import problog2souffle  # noqa: E402
import gen_tpch  # noqa: E402  (FOLD's TPC-H program and relations)
import gen_roulette  # noqa: E402  (FOLD's roulette programs, their arguments and data)
import gen_geni  # noqa: E402  (FOLD's Geni programs, their arguments and data)
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
                   (25, 80, 1), (30, 100, 1), (32, 107, 1), (34, 113, 1)]:
    SPEC[f'g{n}_{m}'] = ('graph', ['gen_graph.py', str(n), str(m), '--seed', str(seed)])
for n in [4, 5, 6, 7, 8, 10, 12, 15, 20, 25, 30]:
    SPEC[f'grid{n}'] = ('grid', ['gen_grid.py', str(n), '--seed', '1'])
for n, k in [(6, 2), (10, 3), (15, 3), (20, 3), (50, 3), (100, 3), (200, 3),
             (1000, 3), (3000, 3), (10000, 3), (30000, 3)]:
    SPEC[f's{n}'] = ('smokers', ['gen_smokers.py', str(n), '--friends-per', str(k), '--seed', '1'])
# The same network with mutual friendships: each pair of friends written in
# both directions, a cycle on every edge. Its sizes are nested (--nested):
# each is the one before with two more people and their friendships, so
# each contains the one before. They grow by two from 4 to 16, the first
# size no system answers: tddlog answers 14 people and runs out of memory
# at 16. (The earlier ssymN drew a new graph for each size.)
for n in [4, 6, 8, 10, 12, 14, 16]:
    SPEC[f'nsym{n}'] = ('symsmokers', ['gen_smokers.py', str(n), '--friends-per', '3', '--seed', '1', '--symmetric',
                                       '--nested'])
for n in [2, 3, 4, 5, 6, 8, 10, 20, 80, 640, 2560]:
    SPEC[f'uwcse{n}'] = ('uwcse', ['gen_uwcse.py', str(n)])
for q in [10, 50, 200, 500, 1000, 2000, 4000, 8000]:
    SPEC[f'cora{q}'] = ('cora', ['gen_cora.py', os.path.join(ROOT, 'data', 'cplint', 'cora_m1.pl'),
                                  '--queries', str(q), '--seed', '1'])
SPEC['bloodtype'] = ('cplint', ['gen_cplint.py', 'bloodtype'])
# Bayesian networks from the bnlearn repository (data/bn, Hugin format, which
# `fetch` downloads): every CPT row an annotated disjunction, the marginals
# of the leaf variables queried. Listed by node count. asia and cancer are
# tests/networks/asia.net and examples/cancer.net, the same networks written out
# from their publications.
BN = ['cancer', 'earthquake', 'asia', 'survey', 'sachs', 'child', 'insurance', 'water', 'alarm',
      'hailfinder', 'hepar2', 'win95pts', 'pathfinder', 'andes', 'pigs', 'link', 'munin']


def bn_source(net):
    mine = {'asia': ('tests', 'networks'), 'cancer': ('examples',)}
    return os.path.join(source_tree.SOURCE, *mine[net], f'{net}.net') if net in mine else os.path.join(ROOT, 'data', 'bn', f'{net}.net')


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

# Larger instances, outside the standard suite so that it stays stable, and
# the Dice programs with loops. The suite was assembled to evaluate selective
# indicators, a rewrite since removed, and keeps its name; the standard
# suite's own instances are not listed again.
INDICATORS = {
    'g40_140': ('graph', ['gen_graph.py', '40', '140', '--seed', '1']),
    'grid40': ('grid', ['gen_grid.py', '40', '--seed', '1']),
    'uwcse12': ('uwcse', ['gen_uwcse.py', '12']),
    'cora1000': ('cora', ['gen_cora.py', os.path.join(ROOT, 'data', 'cplint', 'cora_m1.pl'),
                         '--queries', '1000', '--seed', '1']),
}
for n in [15, 30]:
    INDICATORS[f'grid{n}_ev'] = ('gridev', ['gen_grid.py', str(n), '--seed', '1', '--evidence'])
INDICATORS['cora1000_ev'] = ('coraev', ['gen_cora.py', os.path.join(ROOT, 'data', 'cplint', 'cora_m1.pl'),
                                     '--queries', '1000', '--seed', '1', '--evidence', '20'])
# Dice programs with loops (data/dice/README.md): the Caesar cipher and
# Dice's two model checking examples, translated. The Dice paper's diamond
# and ladder networks are one family each in the roulette suite
# (gen_roulette.py's DICE_PAPER).
for name in ['caesar-iter', 'simpleCaesar', 'pmc2', 'pmc1']:
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
#     Kaminski's noDice (OOPSLA 2026), translated by dice2problog.py:
#     its Table 1's (the coupon collector from 10 coupons, the networks,
#     3sat-0) and the artifact's others (the coupon collector from 2,
#     the bounded retransmission protocol, uniform 3-SAT of 20 and 50
#     variables), each program's sizes smallest first (data/nodice/README.md;
#     its diamonds and ladders are the Dice paper's, the roulette suite's
#     rl_diamond and rl_ladder).
# An arith or pluck program is the model: Alea.jl reads
# it (ALEA_OF, `alea_program`), and every other system a port of it
# (data/alea/README.md, "Ports"). An arith instance is its port in Dice
# (dice2problog.py), with the instance's arguments after it where the port
# has places (ARITH_PLACES: tddlog reads the port and the arguments, the
# Dice family the port with them in its places, NAME.dice); a pluck
# instance its port in ProbLog (gen_pluck.py), with hmm's Dice port beside
# it for the systems that read Dice (DICE_PORTS).
ARITH = ['book', 'tugofwar', 'caesar_small', 'caesar_medium', 'caesar_large', 'caesar_huge',
         'ranking_small', 'ranking_medium', 'ranking_large', 'radar', 'fw_small', 'fw_medium', 'fw_large',
         'linext_small', 'linext_medium', 'linext_large', 'triangle_small', 'triangle_medium', 'triangle_large',
         'gcd_small', 'gcd_medium', 'gcd_large', 'disease_small', 'disease_medium', 'disease_large',
         'luhn_small', 'luhn_medium', 'plankton']
PLUCK = ['hmm', 'pcfg', 'string_editing', 'sorted_list']
NODICE = ([f'coupon2-{n}-obs' for n in [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 16, 18, 20]] +
          ['survey', 'hepar2', 'insurance', 'water', 'alarm', '3sat-0'] +
          [f'uf20-0{n}' for n in range(990, 1000)] + ['uf50-0999'] +
          [f'brp{n}-2' for n in [4, 6, 8, 10, 16, 64, 256, 1024, 4096, 16384]])
ARITH_DIR, PLUCK_DIR = (os.path.join(ROOT, 'data', 'alea', d) for d in ('arith', 'pluck'))
# The ports that are one program for every size: the sizes' values of its places
ARITH_PLACES = {'disease': {'small': '50 101 100', 'medium': '100 501 500', 'large': '500 1501 1500'},
                'triangle': {'small': '15 50 101 16384', 'medium': '15 100 201 16384',
                             'large': '20 500 1001 524288'},
                'gcd': {'small': '9 256 13', 'medium': '11 1024 16', 'large': '13 4096 19'}}
# caesar-small to -huge are one Alea.jl program, the first LENGTH letters of
# its text (each size's text in the paper's files is a prefix of the next's)
CAESAR_LENGTH = {'small': 10, 'medium': 50, 'large': 100, 'huge': 250}
PPL = {}
ALEA_OF = {}   # an instance's generator arguments -> its Alea.jl program and setting (alea_program)
for name in ARITH:
    _prog, _, _size = name.rpartition('_')
    _port = (['dice2problog.py', os.path.join(ARITH_DIR, f'{_prog}.dice'), *ARITH_PLACES[_prog][_size].split()]
             if _prog in ARITH_PLACES else ['dice2problog.py', os.path.join(ARITH_DIR, f'{name}.dice')])
    PPL[f'arith_{name}'] = ('arith', _port)
    ALEA_OF[tuple(_port)] = ([os.path.join(ARITH_DIR, 'caesar.jl'), f'LENGTH={CAESAR_LENGTH[_size]}']
                             if _prog == 'caesar' else [os.path.join(ARITH_DIR, f'{name}.jl')])
for name in PLUCK:
    PPL[f'pluck_{name}'] = ('pluck', ['gen_pluck.py', os.path.join(PLUCK_DIR, f'{name}.pl')])
    ALEA_OF[tuple(PPL[f'pluck_{name}'][1])] = [os.path.join(PLUCK_DIR, f'{name}.jl')]
for name in NODICE:
    PPL[f'nodice_{name}'] = ('nodice', ['dice2problog.py', os.path.join(ROOT, 'data', 'nodice', f'{name}.dice')])
# An instance whose generator is dice2problog.py is a Dice program (its
# argument) translated by tddlog's reader (`write_program`: the program
# the script wrote); one whose generator is alea_bench.jl is the circuit
# of an Alea.jl program; the generators of DICE_WRITERS write a
# Dice program beside the ProbLog one (`--dice`, NAME.dice), as gen_pluck.py
# does for the Pluck programs of DICE_PORTS (`writes_dice`).
DICE_WRITERS = ('gen_roulette.py', 'gen_geni.py')
DICE_PORTS = {('gen_pluck.py', 'hmm.pl')}   # (generator, its argument's file name)
# The generators whose every instance a system reads as a program of its own,
# not one written from the instance's (no ground route): Alea.jl and Pluck
# the Pluck programs' (alea_program, run_pluck).
OWN_PROGRAMS = {'gen_pluck.py': ('alea', 'pluck')}


def writes_dice(gargs):
    """Whether the generator of an instance, its arguments `gargs`, writes a
    Dice program beside its ProbLog one (`--dice`, NAME.dice). The port is
    named by its file, so that the capture's rooted arguments name it too."""
    return gargs[0] in DICE_WRITERS or (len(gargs) > 1 and (gargs[0], os.path.basename(gargs[1])) in DICE_PORTS)


# The systems that read the Dice program its own authors wrote where the
# Dice paper has one (the roulette suite's diamond and ladder: gen_roulette's
# DICE_PAPER); the others' translators read its loop, NAME.port.dice
# (`--dice-port`), so that each target writes the loop as a loop.
DICE_OWN = ('dice', 'nodice')


def dice_ported(gargs):
    """Whether an instance's generator writes the port NAME.port.dice beside
    its Dice program (gen_roulette.py's DICE_PORT)."""
    return gargs[0] == 'gen_roulette.py' and len(gargs) > 1 and gargs[1] in gen_roulette.DICE_PORT


# The generators whose programs tddlog's reader writes (`tddlog --program`):
# of a network, of a Dice program, of a cplint LPAD, whole or with its
# students or families made anew, and of a Circulette netlist.
PROGRAM_OF_TDDLOG = ('gen_bn.py', 'dice2problog.py', 'gen_cplint.py', 'gen_uwcse.py', 'gen_pedigree.py',
                     'gen_circulette.py')
def fold_key(family, a):
    """The name of the data file of an instance of `family` from its
    generator's arguments `a`: the family and the arguments, as one word."""
    return family + ''.join('_' + w.lstrip('-') for w in a)


# The families that are one program, its data and its arguments, the values
# of its places $1, $2, ... (tddlog's program arguments): generator ->
# (the program's name, the generator's arguments that print it with
# --template, the keys of its data files), from an instance's generator
# arguments. An instance's data is one file per key, DIR/data/KEY.pl, that
# the program consults (`:- consult($1).`; a file shared by instances, as
# TPC-H's relations of a scale factor, has a key shared by them) and that
# the generator prints with `--data KEY`; the places of an instance are its
# data files, quoted and relative to the program, then the generator's
# --arguments. gen writes each program once, as DIR/programs/PROGRAM.pl,
# each data file once, and of each instance its arguments (a line each,
# NAME.args) and NAME.pl, the program with them in its places and its
# consults inlined (`tddlog --program`), which the systems that take no
# arguments read. tddlog reads the program, the data and the arguments
# itself (`folded`).
FOLD = {
    'gen_threedie.py': lambda a: ('threedie', [], []),
    'gen_pgrid.py': lambda a: (f'pgrid{a[0]}', [a[0]], []),
    'gen_pcfg.py': lambda a: ('pcfg', [a[0]], []),
    'gen_dnahmm.py': lambda a: ('dnahmm', [], []),
    'gen_plcg.py': lambda a: ('plcg', [], []),
    'gen_hmmpos.py': lambda a: ('hmmpos', [], []),
    'gen_graph.py': lambda a: ('graph', a, [fold_key('graph', a)]),
    'gen_grid.py': lambda a: ('gridev' if '--evidence' in a else 'grid', a,
                              [fold_key('grid', [w for w in a if w != '--evidence'])]),
    'gen_smokers.py': lambda a: ('smokers', a, [fold_key('smokers', a)]),
    'gen_uwcse.py': lambda a: ('uwcse', a, [fold_key('uwcse', a)]),
    'gen_pedigree.py': lambda a: ('pedigree', a, [fold_key('pedigree', a)]),
    'gen_tpch.py': lambda a: (lambda f: (f[0], a, f[1]))(gen_tpch.family(a[:])),
    'gen_taint.py': lambda a: ('taint', a, ['taint_' + os.path.basename(a[0]).split('.')[0].replace('-', '_')]),
    # the roulette suite's programs, hmm's five questions one and the two hardware ladders one
    'gen_roulette.py': lambda a: gen_roulette.fold(a[0], int(a[1])),
    'gen_geni.py': lambda a: gen_geni.fold(a[0], [int(s) for s in a[1:]]),
}
FOLD_DIR = 'programs'


def fold_of(gargs):
    """FOLD's entry of an instance, (program, template arguments, data
    keys), None where its generator is not one of FOLD's or does not fold
    this program."""
    entry = FOLD.get(gargs[0])
    return entry(gargs[1:]) if entry else None


DATA_DIR = 'data'
# A scaled Dice loop's name (dice_scale.py: the loop of data/dice/LOOP.dice run
# K times), which `folded` reads as the loop's program and its count where NAME.args is beside it.
SCALED_LOOP = re.compile(r'dice_(.+)_k\d+')
# The generators of SOUFFLE_WRITERS write the benchmark's original Souffle
# program and its input relations beside the ProbLog one (`--souffle`,
# NAME.souffle/), which PSouffle reads instead of a translation.
SOUFFLE_WRITERS = ('gen_sidechannel.py',)
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
# smallest first, so that a system that runs out of time or memory on one
# skips the larger.
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
# Precisions past the paper's 6 above, for the programs that a system still
# answered there.
BB_PAST = {'pi': [20], 'zeroone_w1': [18, 22, 26, 30], 'zeroone_w2': [19, 21, 23, 25]}
BB_SECTION2 = {  # family: (program, the constant laddered, its values, width less precision or None)
    'genes': ('genes', 'T', [5, 10, 15, 20, 25, 30, 40, 50, 60, 80, 100], None),
    'genes5': ('genes', 'bits', [8, 10, 12, 14, 16, 18], 9),
    'multimodal': ('multimodal', 'bits', [2, 4, 6, 8, 10, 12], 7),
    'conjugate_ex3': ('conjugate_ex3', 'bits', [10, 12, 14, 16, 18, 20, 22], 8)}
BITBLAST = {}
for _name, (_const, _paper, _int) in BB_PRECISION.items():
    for _v in [_paper + d for d in range(-6, 7, 2) if _paper + d >= 1] + BB_PAST.get(_name, []):
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
    'spacex': ('pieces', [4, 16, 64, 256, 1024, 4096]),
    'weekend': ('num_pieces', [4, 16, 64, 256, 1024, 4096, 16384, 65536]),
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
    **{f'fair_dt{_t}': ('fairsquare', 'fairness', 'bits', [4, 8, 12, 16, 20, 24] + ([28, 32] if _t in (4, 14) else []),
                        'w', 8, [f'dt={_t}'])
       for _t in [4, 14, 16, 44]},
    'markov': ('sppl', 'markov_switching', 'n', [10, 20, 50, 100, 200, 500, 1000, 2000, 4000], 'n', 0, []),
    'linreg': ('psi', 'linear_regression', 'bits', [1, 2, 3, 4, 5, 6, 7, 8], 'w', 7, []),
    'wmi_path': ('wmi', 'path', 'n', [16, 32, 64, 128, 256, 512, 1024, 4096, 16384, 65536], 'n', 0, []),
    'wmi_star': ('wmi', 'star', 'n', [16, 32, 64, 128, 256, 512, 1024, 4096, 16384, 65536], 'n', 0, []),
    'pedestrian': ('gubpi', 'pedestrian', 'steps', [4, 6, 8, 10, 12, 16, 20, 24, 32, 40], 's', 0, []),
    'population': ('genfer', 'population', 'steps', [1, 2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64], 's', 0, [])}
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
# artifact's benchmark/ directory). gen_roulette.py writes a family's ProbLog
# program (FOLD) and an instance's Dice program; `--roulette CMD` runs Roulette itself on the
# artifact's program, ported to neuppl/roulette's example language
# (data/roulette, `run_roulette`). The hmm marginal likelihood stops
# at 900 steps: past about 915 it is below the smallest normal double.
RL_LADDER = [1000, 3000, 10000, 30000, 100000, 300000, 1000000]
RL_SIZES = {'motivating': RL_LADDER + [3000000, 9000000], 'diamond': [100, 300] + RL_LADDER + [3000000, 6000000],
            'ladder': [100, 300] + RL_LADDER + [3000000],
            'caesar': [25, 50, 100, 200, 300, 500, 1000, 2000, 5000, 10000, 20000, 50000, 100000],
            'hmm-marginal-likelihood': [100, 300, 600, 900],
            **{f'hmm-{q}': [100, 300, 1000, 3000, 10000, 30000, 100000, 300000, 1000000, 3000000]
               for q in ['filtering-marginal', 'smoothing-marginal', 'filtering-joint', 'smoothing-joint']},
            'network': [10, 30, 100, 300, 1000, 3000, 10000, 30000, 100000, 300000, 1000000],
            'hardware': [10, 30, 100, 300, 1000, 3000, 10000, 30000, 100000, 300000, 1000000],
            'hardware-bitwidth': [5, 10, 20, 50, 100, 300, 1000, 3000, 10000, 30000, 100000, 300000, 1000000,
                                  3000000, 10000000, 100000000]}
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
# which is geni_weather_n's). gen_geni.py writes each family's ProbLog
# program (FOLD) and each instance's Dice program, and answers it
# (`exact:dp`).
GENI_SIZES = {'link': [4, 6, 8, 10, 12, 14, 16, 18, 20, 24, 28, 32],
              'router': [4, 6, 8, 10, 12, 14, 16, 18, 20, 24, 28, 32, 40, 48, 56, 64, 72]}
GENI_WEATHER_N = [2, 7, 8, 9, 10, 11, 12, 13, 15, 16, 17, 18, 19]
GENI_WEATHER_H = list(range(1, 10)) + list(range(100, 1001, 100)) + [3000, 10000, 30000]
GENI = {}
for _family, _sizes in GENI_SIZES.items():
    for _n in _sizes:
        GENI[f'geni_{_family}_{_n}'] = (f'geni_{_family}', ['gen_geni.py', _family, str(_n)])
for _n in GENI_WEATHER_N:
    GENI[f'geni_weather_n{_n}'] = ('geni_weather_n', ['gen_geni.py', 'weather', str(_n), '10'])
for _h in GENI_WEATHER_H:
    GENI[f'geni_weather_h{_h}'] = ('geni_weather_h', ['gen_geni.py', 'weather', '10', str(_h)])

# The benchmarks of Circulette (Sandia; a language on Roulette's package
# roulette-lib for the fault analysis of digital circuits, gen_circulette.py),
# a suite of its own (`--suite circulette`, `--circulette-benchmarks` its
# checkout): its four netlists (ripple_adder.cql is exp.cql), the designs
# of examples/ that rkt_of_verilog.py converts (all but
# sram_2_16_2_scn4m_subm and openfpga_syn_spram_4x1_tmrg), each as its
# authors wrote it, and two of them grown into families of sizes:
# fullAdder8.v's adder by its bits (ct_fulladder, 8 the file as written)
# and openfpga_mac_2.v's multiply-accumulate by DATA_WIDTH (ct_mac, 2 and
# 4 openfpga_mac_2.v and openfpga_mac_4.v). gen_circulette.py wrote each
# netlist NAME.cql once, with the arguments below; rkt_of_verilog.py then
# numbered a design's ports in an order that changed between runs, so the
# netlists every system reads are those files, kept apart from the
# repository as Circulette's (`--circulette-netlists DIR`), and gen copies
# each and checks its SHA-256 (CT_SHA256). tddlog reads the netlist itself
# (`original`), and gen writes the program of it (`tddlog --program`, NAME.pl)
# for the systems that read ProbLog programs; Roulette runs circulette.rkt
# itself on the netlist (`run_circulette`). The other netlists are no sizes
# of one another (COLLECTIONS).
CT_ROOT = ['1-bit-mux', '4-bit-comparator', 'adder8', 'exp']
CT_DESIGNS = {'halfAdder': 'adder/halfAdder.v', 'fullAdder': 'adder/halfAdder.v,adder/fullAdder.v',
              'ALU': 'alu/ALU.v', 'ALU_Controller': 'alu/ALU.v,alu/ALU_Controller.v', 'FIFO': 'alu/FIFO.v',
              'ALU_Top': 'alu/ALU.v,alu/FIFO.v,alu/ALU_Controller.v,alu/ALU_Top.v',
              **{d: d + '.v' for d in ['2bit-adder', '4bit-multiplier', '8bit-adder', 'jsrflipflop', 'mux2',
                                       'serial-crc', 'openfpga_and4', 'openfpga_and4_tmrg',
                                       'openfpga_asyn_spram_4x1', 'openfpga_clock_divider',
                                       'openfpga_dual_port_ram_1k', 'openfpga_pulse_generator',
                                       'openfpga_syn_spram_4x1', 'openfpga_syn_spram_4x1_tmr']}}
CT_SIZES = {'fulladder': [2, 4, 6, 8, 10, 12, 16, 18, 20, 24, 32, 48], 'mac': [1, 2, 3, 4, 5, 6]}
CIRCULETTE = {}
for _n in CT_ROOT:
    CIRCULETTE[f'ct_{_n}'] = ('circulette', ['gen_circulette.py', 'root', _n])
for _n, _files in CT_DESIGNS.items():
    CIRCULETTE[f'ct_{_n}'] = ('circulette', ['gen_circulette.py', 'verilog', _files])
for _family, _sizes in CT_SIZES.items():
    for _n in _sizes:
        CIRCULETTE[f'ct_{_family}{_n}'] = (f'ct_{_family}', ['gen_circulette.py', _family, str(_n)])
# The SHA-256 of each netlist of CIRCULETTE, which gen checks (gen_circulette).
CT_SHA256 = {
    'ct_1-bit-mux': 'fb57af80b9032bb45fc368c62b95f6754113d7bd93b1e00c179f28bdb4b83ea9',
    'ct_2bit-adder': '0874bf0def916032a2efbf2981e8d1460eacb00952adc4f9616c86b4f20d1ddc',
    'ct_4-bit-comparator': '6b73588b6c0b98173be13cc25367fd6a5ea11635a5be6aa67246fd21a06327a2',
    'ct_4bit-multiplier': 'e307b556cb2be02aeb317a3f4e25cd45c25d5661b82c817552febd20e5df799a',
    'ct_8bit-adder': '224e24765c08ddf83158ad3a50e4a10060ba3601ebf576b3bf16276e3941f6f4',
    'ct_ALU': '1eac4e5ef4a436caabba3c462b50e509a8d4f67140cd162043ec40659dec2c87',
    'ct_ALU_Controller': '21b6652bebb3612edcb35317b5370018f6e5e853fde4a21ca59b0d385cab60ef',
    'ct_ALU_Top': '1f237cf36d81b163be798715b64dee8e18d9c93357d1408436cac21b10bd9383',
    'ct_FIFO': '2d0cbe0a07c00e282643d1921a99a349ccfe7da708a6bce4b6daf0b507987971',
    'ct_adder8': '2578694384a99da86db6f9a2ed6b47240c75e76a84f9c2a4bf05b4d7335b62f5',
    'ct_exp': 'a4807aba10fb084d4c40b76d562d44447c0c8b44623425ddab5a77e37e5ba6f1',
    'ct_fullAdder': '49aae82314f8b12f89f7f459c2ca906dfece5a11e6725f46a48ba8cf0e563bc4',
    'ct_fulladder10': '5e34a82b70461493f010e26d29b688a7e075b14d43e2954c1e1ca62f29689300',
    'ct_fulladder12': 'adbe428660a3795711b6ef9392a5916d63d4c325ec1d4ba1a262851f790cfd22',
    'ct_fulladder16': '66f450bf6e3a51053c7f4ba6f0f62d6ff3ba5aaedbb94d4b79efe1fb6a65bd04',
    'ct_fulladder18': 'f413fc645e2ba97091a5d55f0d06a1b5f8b3e070371d60a37fe6a99c985356c7',
    'ct_fulladder2': 'd9eb29d68c659bcf44f88a21d812e1fc2daeb018d895cdea00ccb387486d2fa8',
    'ct_fulladder20': '7585d494a2cfa3d1d9c50524fc674d03a6ea568fa7e31c6b41adcbc1efd5e268',
    'ct_fulladder24': '7ce25d5b026ea8ca4991b0027d2cc86b43f1f05252954002459a7752a3710ab3',
    'ct_fulladder32': 'c805ddb84b8680658d292f35a3c5b03bb90d8ca9ca8e7425bde97fbd464ec26c',
    'ct_fulladder4': 'f3aafa9246ba4c6a107acebda563437f5589f0eb9902e3bb36df532f95fb662b',
    'ct_fulladder48': '27c2a3f07273657b1d3a7f4e3158ad3ac92ad836fd5b2ce98240cd2caae0abdb',
    'ct_fulladder6': '8ffe96dfe818caada9b4e5f1fc939e096b26a75df612368ef90e17c8dea8ca2d',
    'ct_fulladder8': '6b067bf7d6baaf9ff1018d9e4634e31281e75796372af7387f462cd6629cc04c',
    'ct_halfAdder': '595396658680161ced8df8fda20ade3d67ab3931643e6d983b0481a143a0290a',
    'ct_jsrflipflop': '1fe8eddec8150f45032c5bf1deba786477000628f012063e4dce1263d626ed7a',
    'ct_mac1': '96fa245184a30a2bbdc21e464d005769ed5c2c3a702428d446553745a555898b',
    'ct_mac2': '3555f8c298c4f654aa21cd46132639fde639d4e112878593644216a010ba294e',
    'ct_mac3': '1f91281cf34cc4ab256656bdb3d813b21d12335c12dba51f2511f09c501a2ca0',
    'ct_mac4': '0cdaaa061320dc11ced20a1d7a5e09df0832159a91aaddb2e781e20c65dfea57',
    'ct_mac5': '1dc97466bd2d49954ae622c229cf017ef82ffdd810d6ffca2dd2b75307d907f2',
    'ct_mac6': '686538b8cdab75ddf33f64d85c92c7b9c5a38f3be831480955b983e40d90246d',
    'ct_mux2': '9ebd0d5ff8c6c101add5de0325c178de908c1801be85d42d8ce3f79cba4a303d',
    'ct_openfpga_and4': '5ec141d7948c8c94a6091de0c438a9dc16cb2534300abf14a50676f1d21b8c2c',
    'ct_openfpga_and4_tmrg': '63d6dc561e340dfe2326530e74738f2d2e055ac8cf7e5553b56bd2233c103eb2',
    'ct_openfpga_asyn_spram_4x1': 'a2833c128ffa661b1b857893dc928bcb617dabd1d7952f4a79270aaf15a2e273',
    'ct_openfpga_clock_divider': '1d16ee4d736c0533c8420c60619e52f38e7927283b9fe2ecc55e18610930d827',
    'ct_openfpga_dual_port_ram_1k': '736deda101a949c9e23a33afa412efa558a8a2ccc4d705470ed227126429da2a',
    'ct_openfpga_pulse_generator': '6945b17670d41883f34e6a094a6793add552b24454451e62d653defdec62975f',
    'ct_openfpga_syn_spram_4x1': '50d047bbfa9d97dbba7be0fde1a1119594375357fdcd9cafa4bc6c39f6422706',
    'ct_openfpga_syn_spram_4x1_tmr': '3525c1023a37cdfc4f99a043e4f1ea4300da0c653923965df6ed3cb0d16452c6',
    'ct_serial-crc': '01a17fb1695a3762aaea32a742489b2d58598674c2edd0f6c55808f415dbc6e0',
}


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
for n in [4, 8, 16, 32, 64, 96, 192, 384, 768]:
    LIT[f'mnist_sum{n}'] = ('mnist_sum', ['gen_mnist_sum.py', str(n), '--seed', '1'])
for g, w in [(4, 4), (4, 6), (6, 6), (10, 6), (5, 10), (6, 8), (16, 6), (32, 6), (6, 10), (7, 10),
             (7, 12), (14, 10), (7, 13), (7, 14)]:
    LIT[f'pedigree{g}_{w}'] = ('pedigree', ['gen_pedigree.py', str(g), str(w), '--seed', '1'])
for n, t in [(6, 10), (10, 10), (14, 10), (18, 20), (22, 40), (45, 40), (180, 40), (240, 40)]:
    LIT[f'sickness{n}_{t}'] = ('sickness', ['gen_sickness.py', str(n), str(t), '--seed', '1'])
for d, f in [(6, 9), (8, 12), (10, 15), (15, 22), (20, 30), (30, 45), (60, 90), (120, 180), (240, 360),
             (300, 450), (360, 540)]:
    LIT[f'noisyor{d}_{f}'] = ('noisyor', ['gen_noisyor.py', str(d), str(f), '--seed', '1'])
for c, pegs, r in [(4, 4, 8), (6, 4, 8), (8, 4, 8), (10, 4, 8), (4, 5, 8), (6, 5, 8), (8, 5, 8),
                   (8, 8, 8), (16, 5, 8), (32, 5, 8), (64, 5, 8)]:
    LIT[f'mastermind{c}_{pegs}_{r}'] = ('mastermind', ['gen_mastermind.py', str(c), str(pegs), str(r), '--seed', '1'])
for n in [10, 15, 20, 30, 40, 80, 160, 320, 640]:
    LIT[f'pdb{n}'] = ('pdb', ['gen_pdb.py', str(n), '--seed', '1'])
LIT_FAMILIES = list(dict.fromkeys(family for family, _ in LIT.values()))

# The exact reference of the instances a generator writes, for the instances
# ProbLog does not finish: generator -> (variant, command for a program).
EXACT = {'gen_bn.py': ('cliquetree', lambda path: ['bn_exact.py', '--precision', '17', path])}
for family in LIT_FAMILIES:
    EXACT[f'gen_{family}.py'] = ('lit', lambda path, family=family: ['lit_exact.py', family, path,
                                                                    '--precision', '17'])

# The families of the second literature scan:
# Prob-EC's event calculus and PSouffle's taint analyses of Android apps
# (logic programs), and two #P-hard queries on probabilistic databases,
# Beame et al.'s safe queries whose lineage has no small decision-DNNF and
# Fink et al.'s path of two edges on a random graph. Each has its
# reference in lit_exact.py but taint, which has none but ProbLog's and
# PSouffle's, the paper's own system. The taint programs are PSouffle's
# (MIT licence), which data/fetch.txt names.
TAINT_APPS = ['and-roc', 'andors-trail', 'angulo', 'app-018', 'app-ca7', 'app-kQm', 'barcode', 'ginger-master',
              'noisy-sounds', 'savage-knife', 'tilt-mazes', 'tom-droid', 'tuio-droid', 'video-game', 'yaaic']
for t in [250, 500, 1000, 2000, 4000, 8000]:
    LIT[f'eventcalc4_{t}'] = ('eventcalc', ['gen_eventcalc.py', '4', str(t), '--seed', '1'])
for app in TAINT_APPS:
    LIT[f'taint_{app.replace("-", "_")}'] = ('taint', ['gen_taint.py',
                                                        os.path.join(ROOT, 'data', 'psouffle', 'taint', f'{app}.pl.gz')])
# PSouffle's side-channel analyses (the same paper), whose 19 cases are
# Souffle programs that gen_sidechannel.py translates and PSouffle reads
# as they are (SOUFFLE_WRITERS).
for case in ['P1'] + [f'P{k}' for k in range(3, 21)]:
    LIT[f'sidechannel_{case.lower()}'] = ('sidechannel', ['gen_sidechannel.py'] + [
        os.path.join(ROOT, 'data', 'psouffle', 'side_channel', f'{case}.{kind}.dl.gz') for kind in ('souffle', 'problog')])
for n in [3, 4, 6, 8, 11, 16, 23, 32]:
    LIT[f'liftsep{n}'] = ('liftsep', ['gen_liftsep.py', str(n), '--seed', '1'])
for n in [6, 8, 10, 12, 14, 17, 20, 24]:
    LIT[f'rgraph{n}'] = ('rgraph', ['gen_rgraph.py', str(n), '--seed', '1'])
for family in ['eventcalc', 'liftsep', 'rgraph']:
    EXACT[f'gen_{family}.py'] = ('lit', lambda path, family=family: ['lit_exact.py', family, path,
                                                                    '--precision', '17'])

# Grammars and an HMM over lists, which tddlog's grounder reads since it
# grounds function symbols and lists: the ProbLog tutorial's PCFG (the
# derivation's stack), asked how the derivation of a balanced word of length
# N and three or more top-level blocks brackets them, given the word (the
# first two rules: 1/2 for three blocks), and cplint's HMM of DNA, whose
# choices are indexed by
# the states before them, asked the probability of a sequence of length N.
for n in [8, 12, 16, 20, 24, 28, 32]:
    LIT[f'pcfg{n}'] = ('pcfg', ['gen_pcfg.py', 'stack', str(n), '--seed', '1'])
for n in [4, 8, 12, 16, 20, 24, 28]:
    LIT[f'dnahmm{n}'] = ('dnahmm', ['gen_dnahmm.py', str(n), '--seed', '1'])
for script, family in [('gen_pcfg.py', 'pcfg'), ('gen_dnahmm.py', 'dnahmm')]:
    EXACT[script] = ('lit', lambda path, family=family: ['lit_exact.py', family, path, '--precision', '17'])

# cplint's examples whose choices are indexed by the list of choices
# before them: the probabilistic left-corner grammar, whose clauses negate
# atoms with variables of their own, asked the probability of a word of N
# letters, and the part-of-speech HMM, asked the probability of its
# example's sentence said K times.
for n in [4, 6, 8, 10, 12, 14, 16]:
    LIT[f'plcg{n}'] = ('plcg', ['gen_plcg.py', str(n), '--seed', '1'])
for k in [1, 2, 3, 4, 5]:
    LIT[f'hmmpos{k}'] = ('hmmpos', ['gen_hmmpos.py', str(k)])
for family in ['plcg', 'hmmpos']:
    EXACT[f'gen_{family}.py'] = ('lit', lambda path, family=family: ['lit_exact.py', family, path, '--precision', '17'])

# DeepProbLog's handwritten formulas: the probability of the label of a
# formula of N symbols, each image's classifier an annotated disjunction,
# the formula's symbols a list that parse/2 evaluates.
for n in [3, 5, 7, 9, 11]:
    LIT[f'hwf{n}'] = ('hwf', ['gen_hwf.py', str(n), '--seed', '1'])
EXACT['gen_hwf.py'] = ('lit', lambda path: ['lit_exact.py', 'hwf', path, '--precision', '17'])


# The third literature scan's programs: the
# systems' own, each a program of its own (COLLECTIONS) but threedie and
# pgrid. problogtut: the ProbLog tutorial's programs that ask marginals, as
# the pages write them (gen_problogtut.py; the pages are fetched, no
# licence), all but the three of the aProbLog page, whose semirings are not
# probabilities, basic/03_dice's third, which ProbLog refuses (a query left
# non-ground), and six copies of problogex's; their reference is ProbLog's,
# one query at a time (problog_each.py). problogex: the example programs of
# ProbLog's repository (data/problog, Apache 2.0), each its own reference
# in the comment that states its expected answers. cplintpita: cplint's
# examples written for PITA (data/cplint, Artistic 2.0), read as LPADs,
# ProbLog's answers one query at a time their reference (markov_chaindb,
# pcfglrdb and var_objdb bounded at PITA's depth, as `tddlog --program`
# writes them); left out are meta and metacomb, which no system answers as
# the harness gives them (prob/2 inside the program), and the copies
# coinmsw_memo (coinmsw with memo mode set) and eruption_tab (eruption
# tabled); with them cplint on SWISH's examples that are not copies of the
# pack's (data/swish, Simplified BSD), cplint_swish_<name>. threedie: PITA's three-sided die thrown until a 3 (Riguzzi and
# Swift, ICLP 2010, Fig. 3; cplint's threesideddice.pl), asked on(N,1),
# (2/3)^N/3, N up to where a double still holds it (PITA and ProbLog stop
# between 100 and 1000).
PROBLOGTUT = {
    'advanced/10_urnballmodel': [1],
    'basic/01_coins': [1, 2, 3],
    'basic/02_bayes': [4],
    'basic/03_dice': [1, 2, 4, 5, 6, 7, 8, 9],
    'basic/05_smokers': [1],
    'basic/08_rule_probs': [1, 2, 3, 4, 5, 6, 7, 8, 9],
    'basic/09_higherorderfunctions': [1, 2, 3, 4, 5],
    'basic/10_inhibitioneffects': [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
    'tutslides/01_basic': [1, 2, 3, 4],
    'tutslides/02_more': [1, 2, 3, 4],
    'tutslides/03_stochmem': [1, 2],
    'tutslides/04_bayesian_learning': [1],
    'tutslides/05_complexity': [1],
    'tutslides/06_smart_students': [1, 2, 3],
    'various/01_montyhall': [1, 2],
    'various/02_bousi': [1, 2],
    'various/03_crowds': [1],
    'various/04_nampally': [1, 2, 3],
    'various/05_numberguessing': [1],
    'various/06_hmm': [1, 2, 3],
    'various/06_pcfg': [1, 2, 3, 4, 5],
    'various/06_slp': [1],
    'various/07_ppdatalog': [1, 2, 3, 4, 5, 6, 7],
    'various/08_bayesian_dataflow': [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
    'various/09_airflap': [1],
    'various/10_causalpower': [1, 2],
    'various/10_prism': [3, 5, 6, 7, 8, 9],
    'various/11_semweb': [1],
    'various/12_pingpong': [1],
    'various/13_incomplete_info': [1, 2, 3, 4],
    'various/14_robot_key': [1, 2],
    'various/15_spaghetti': [1],
    'various/16_cplogic': [1, 2],
    'various/17_taxonomy': [1],
}
for page, ks in PROBLOGTUT.items():
    for k in ks:
        LIT[f'problogtut_{page.replace("/", "_")}_{k}'] = ('problogtut', ['gen_problogtut.py', page, str(k)])
PROBLOGEX = ['3_tossing_coin', '4_bayesian_net', '4_1_bayesian_net', '5_bayesian_net', '6_hmm_weather',
             '7_probabilistic_graph', '8_smokers_network', '9_packing_problem', '10_cards', '11_ads_numerical',
             '12_holidays', 'coin', 'smokers_or', 'advars_smokers', 'advars_smokers_alt', 'tc_1', 'tc_2', 'tc_3',
             'negation', 'swap', 'add', 'generated']
for name in PROBLOGEX:
    LIT[f'problogex_{name}'] = ('problogex', ['gen_problogex.py', name])
CPLINTPITA = ['alarm', 'bag_game', 'coin', 'coin2', 'coinmsw', 'dice', 'earthquake', 'epidemic', 'eruption',
              'event_calculus', 'flexprob', 'jail', 'light', 'markov_chaindb', 'markov_factor', 'markov_network',
              'mendel', 'mendelc', 'mln', 'monty', 'multiple_paths', 'multiple_paths_simple', 'path', 'pathdb',
              'pcfg', 'pcfglrdb', 'persons', 'simpson', 'sneezing', 'testdb', 'threesideddice', 'trigger',
              'var_objdb', 'viral']
for name in CPLINTPITA:
    LIT[f'cplint_{name}'] = ('cplintpita', ['gen_cplint.py', name])
for name in ['admission', 'smoking', 'sneezing_simple', 'viraltab']:
    LIT[f'cplint_swish_{name}'] = ('cplintpita', ['gen_cplint.py', name, 'swish'])
for n in [10, 100, 300, 1000, 1500]:
    LIT[f'threedie{n}'] = ('threedie', ['gen_threedie.py', str(n)])
for script in ['gen_problogtut.py', 'gen_cplint.py']:
    EXACT[script] = ('problog1', lambda path: ['problog_each.py', path])
EXACT['gen_problogex.py'] = ('lit', lambda path: ['lit_exact.py', 'expected', path, '--precision', '17'])
EXACT['gen_threedie.py'] = ('lit', lambda path: ['lit_exact.py', 'threedie', path, '--precision', '17'])
# ProbLog2's probabilistic grid (Fierens et al., TPLP 2015, Section 9.1 and
# 9.2.1, Fig. 5): its 16 x 16 grid asked a path over distance D, ProbLog2
# at about 110 s at D = 10 in the paper and beyond its 300 s after (at 120
# s on a lab box ProbLog stops between D = 4 and 6, PITA between 6 and 8),
# then larger grids corner to corner, 18 x 18 and 20 x 20.
for n, d in [(16, 4), (16, 5), (16, 6), (16, 7), (16, 8), (16, 10), (16, 12), (16, 15), (18, 17), (20, 19)]:
    LIT[f'pgrid{n}_{d}'] = ('pgrid', ['gen_pgrid.py', str(n), str(d)])
EXACT['gen_pgrid.py'] = ('lit', lambda path: ['lit_exact.py', 'pgrid', path, '--precision', '17'])


# TPC-H as a tuple-independent probabilistic database (gen_tpch.py), the
# workload of SPROUT and of dissociation: each Boolean query alone, a
# family of its own, at scale factors from 0.001 to 1, instance
# tpch_<query>_<1000 SF>, from the smallest at which its selections leave a
# tuple. q19's leave at most three below SF 0.1; it is not in the suite.
# Each query is also written through views (`--views`), as a person would
# write it over a database, instance tpch_<query>v_<1000 SF>: the same
# database and lineage, so the same answer. A family stops at TPCH_LAST,
# past the first scale factor at which no system answered within the
# suite's limit.
TPCH = {}
TPCH_LAST = {'q7': 500, 'q7v': 200, 'q21': 100, 'q21v': 100}
for q, first in [('q1', 1), ('q2', 2), ('q3', 1), ('q4', 1), ('q6', 1), ('q7', 2), ('q10', 1), ('q11', 5),
                 ('q12', 1), ('q14', 1), ('q15', 1), ('q16', 1), ('q17', 5), ('q18', 1), ('q20', 2), ('q21', 2)]:
    for views in ['', 'v']:
        for m in [1, 2, 5, 10, 20, 50, 100, 200, 500, 1000]:
            if first <= m <= TPCH_LAST.get(q + views, 1000):
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
# UW-CSE's students are alike given the course and the professor, so a lifted
# computation answers every size, past where Ace and Merlin stop (uwcse2560).
EXACT['gen_uwcse.py'] = ('lifted', lambda path: ['uwcse_exact.py', path, '--precision', '17'])


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
         'ppl': PPL, 'bitblast': BITBLAST, 'geni': GENI, 'circulette': CIRCULETTE}[args.suite]
    unknown = set(args.only) - spec.keys()
    if unknown:
        raise SystemExit('unknown instances: ' + ', '.join(sorted(unknown)))
    return {n: entry for n, entry in spec.items()
            if (not args.families or entry[0] in args.families) and (not args.only or n in args.only)}


# The families of different programs, each with the pattern that matches the
# sizes of a program among them (its group names the program; the sizes are
# listed smallest first): Alea.jl's arithmetic programs, noDice's coupon
# collector and bounded retransmission protocol (its networks and 3-SAT
# formulas are none a size of another), and
# bench_one.py's scaled Dice loops. The taint family's apps
# are one analysis of fifteen programs, none a size of another, and so are
# the side-channel family's cases, whose rules differ from case to case.
COLLECTIONS = {'dice': None, 'cplint': None, 'pluck': None, 'taint': None, 'sidechannel': None,
               'circulette': None,
               'problogtut': None, 'problogex': None, 'cplintpita': None,
               'arith': r'(arith_[a-z]+)_(?:small|medium|large|huge)', 'nodice': r'(nodice_(?:coupon2|brp))-?\d+-(?:obs|2)',
               'dice_scaled': r'(dice_.+)_k\d+'}


def series(family, name):
    """The program of which instance `name` of `family` is one size, or None:
    a network (the families of bnlearn's, Ace's and the UAI sets' networks
    hold different networks) or a program of one size in a family of
    different programs (COLLECTIONS). Every other family is one program at
    sizes listed smallest first."""
    if family in ('bn', 'bnev', 'bnint') or family.startswith(('ace', 'uai')):
        return None
    if family in COLLECTIONS:
        m = COLLECTIONS[family] and re.fullmatch(COLLECTIONS[family], name)
        return m.group(1) if m else None
    return family


# The instances of the suites this script lists itself, each by its place in
# its suite's listing, so a program's sizes smallest first (series). A table's
# file need not hold them in that order: a run in lanes writes each record as
# its job ends (Geni's weather H at 600 to 1000 days before 4).
LISTED = {}
for _spec in (SPEC, INDICATORS, LARGE, LIT, TPCH, PPL, BITBLAST, GENI, CIRCULETTE):
    for _k, _name in enumerate(_spec):
        LISTED.setdefault(_name, _k)


# How the systems say that an allocation failed, and the run's own kill at
# the memory cap (`timed`): C++'s bad_alloc, the C libraries' (CUDD's,
# SDD's, GMP's, glibc's) and Rust's messages, Python's MemoryError and
# numpy's, Java's and Julia's OutOfMemoryError, OCaml's, D's and LLVM's
# messages, PRISM's `Error: Out of memory.`, tddlog's own refusals, and
# V8's stack at its fatal heap limit where the record keeps only the end
# of node's output (WebPPL's).
MEMOUT = re.compile(r'killed by the memory cap|bad_alloc|(?:unable to|cannot|could not) (?:re)?allocate|'
                    r'memory allocation (?:of \d+ bytes failed|failed|refused)|out of memory|OutOfMemoryError|'
                    r'MemoryError|Java heap space|(?:malloc|calloc|realloc) failed|ran out of memory|'
                    r'for lack of memory|v8::internal::Runtime_Allocate|'
                    r'Not enough resources: (?:private_|shared_)?table_space', re.I)
# Texts that name memory and say nothing of an allocation that failed:
# PRISM's header of its limits, printed with every error; a C++ allocator
# in a type's name (PSouffle's assertions); Julia's memory type in a stack
# trace and the summary it prints after a fatal signal; the limit of the
# handles of Julia's old parser on a long expression; and the address-space
# limit that geni_wrap.sh could not raise.
NOT_MEMOUT = re.compile(r'Memory limits: cudd=\S+ java\(heap\)=\S+|std::allocator<|GenericMemory|'
                        r'Allocations: \d+ \(Pool: \d+; Big: \d+\); GC: \d+|memory-error out of gc handles|'
                        r'ulimit: virtual memory: cannot modify limit', re.I)


class Unclassed(ValueError):
    """An error that names memory, whose text neither MEMOUT nor NOT_MEMOUT
    classes."""


def memout(r):
    """Whether record r failed for want of memory: an error whose text says
    that an allocation failed, or that the memory cap killed the run
    (MEMOUT). An error that names memory otherwise is one of the known
    texts of NOT_MEMOUT, and raises Unclassed where it is not (a
    new text is read and listed, not guessed)."""
    if r['status'] != 'error':
        return False
    e = str(r.get('error'))
    if MEMOUT.search(e):
        return True
    if re.search(r'memory|alloc', NOT_MEMOUT.sub('', e), re.I):
        raise Unclassed(f'{r.get("system", "?")} on {r.get("instance", "?")}: an error that names memory and is '
                        f'neither an allocation failure nor one of the known texts (bench.py NOT_MEMOUT): {e[-400:]!r}')
    return False


# A run that a signal from outside ended before its time limit (`timed`):
# neither the harness's own kill at the limit (a timeout) nor the memory
# cap's (a memout), but the box's out-of-memory guard (earlyoom's SIGTERM
# to the largest process when the box's free memory runs low, whatever that
# process's own cap), the kernel's out-of-memory killer (SIGKILL) or another
# process. Such a run is not the system's failure: its record has status
# `stopped`, its returncode and what the run printed, and the run is done
# again.
STOP_SIGNALS = (-signal.SIGTERM, -signal.SIGKILL)
# The same stop in a record written before the status `stopped`, an error:
# a process that printed nothing (`exit -15`, `exit -9`), or Julia's report
# of the SIGTERM it was sent.
STOPPED_TEXT = re.compile(r'exit -(?:15|9)$|\[\d+\] signal 15: Terminated\n')


def stopped(r):
    """Whether record r is a run stopped from outside before its time
    limit (STOP_SIGNALS): status `stopped`, or an earlier error whose text
    is such a stop's (STOPPED_TEXT) and no allocation failure's (MEMOUT)."""
    if r['status'] == 'stopped':
        return True
    e = str(r.get('error') or '')
    return r['status'] == 'error' and bool(STOPPED_TEXT.match(e)) and not MEMOUT.search(e)


# A translator for one system that did not finish within the time limit
# (`translated`, dice2alea.py's and dice2roulette.py's calls): an error, its
# text kept, that the skip rule takes for a timeout (ran_out). A system that
# cannot be handed a program's smaller size within the limit is not handed
# its larger ones either.
TRANSLATION_LIMIT = re.compile(r'\S+: no translation within ')


def translation_limit(r):
    """Whether record r is a translator's time limit (TRANSLATION_LIMIT)."""
    return r['status'] == 'error' and bool(TRANSLATION_LIMIT.match(str(r.get('error') or '')))


# PSouffle's compilation stopped at the run's limit, or at PSOUFFLE_COMPILE_S
# where that is lower (psouffle_binary): an error, its text kept, that the
# processing charges as a timeout past every limit (rel/charge.py) and the
# skip rule takes for one (ran_out).
COMPILE_TIMEOUT = "PSouffle's compilation: timeout"


def compile_timeout(r):
    """Whether record r is PSouffle's compilation stopped at its bound (COMPILE_TIMEOUT)."""
    return r['status'] == 'error' and str(r.get('error') or '').startswith(COMPILE_TIMEOUT)


def ran_out(r):
    """Whether a run ran out of time or memory (`memout`), for the skip rule,
    a translator's time limit (translation_limit) and PSouffle's compilation
    stopped at its bound (compile_timeout) counted as timeouts. An
    unclassed error does not skip the larger sizes: running them costs time
    and nothing else, and the processing stops on the text."""
    try:
        return r['status'] == 'timeout' or translation_limit(r) or compile_timeout(r) or memout(r)
    except Unclassed:
        return False


def stale_refusal(r):
    """Whether record r is the refusal of aspmc on a program with evidence by
    the harness before `aspmc_conditioned`, an error of this text. The
    harness runs aspmc there since, and a later run's record of the
    instance replaces each such one in the results (docs/baselines.md,
    aspmc): it says nothing of aspmc, and neither a table nor a page shows
    it."""
    return r['status'] == 'error' and 'aspmc ProbLog frontend does not condition on evidence' in str(r.get('error'))


def unexpressed(r):
    """Whether record r is a program its system was not given since it cannot
    be: a translator's refusal of what the system's language lacks, or the
    language's own refusal (status `unsupported`, the reason in `error`).
    It is a program the system does not support, on
    every suite, never one it failed on. The tables and the
    PPL suites' data (ppl_data.py) both read a record's failure so."""
    return r.get('status') == 'unsupported'


# The systems the tables time by their own time in the process,
# without their language's start, their packages' load and their
# compilation: Alea.jl (and Alea.jl+tddlog), Pluck and TensorInference.jl in Julia, Roulette in
# Racket, WebPPL in node (its compile and run of the program after a
# warm-up). Their records' `seconds` stay the process's wall time, and a run
# whose wall time passed the limit stays a timeout.
OWN_TIME = ('alea', 'alea-tddlog', 'roulette', 'roulette-tddlog', 'pluck', 'tensorinference', 'webppl')


def own_seconds(r):
    """The own time of an answering record r of a system in OWN_TIME, its
    stats among its keys: warm where it ran so (`warm_seconds`: Alea.jl's,
    Pluck's and TensorInference.jl's second run in the process), else in
    the process less the compilation within it (`inproc_seconds` less
    `jit_seconds` where it reports one) and less the load of the system's
    library within it (`header_seconds`, WebPPL's compile of its header, as
    Julia's packages and Racket's modules load outside it), else the
    process's wall time (TensorInference.jl's runs before its warm-up)."""
    if r.get('warm_seconds') is not None:
        return r['warm_seconds']
    if r.get('inproc_seconds') is not None:
        return max(r['inproc_seconds'] - r.get('jit_seconds', 0.0) - r.get('header_seconds', 0.0), 0.0)
    return r['seconds']


# The times rel/charge.py adds a conversion's seconds to, which it keeps as they were before
# that charge under RAW + the key.
CHARGED_TIMES = ('seconds', 'inproc_seconds', 'warm_seconds')
RAW = 'raw_'


def uncharged(r):
    """Record r without the conversion it pays (rel/charge.py's
    `translation_seconds`): its status and times as they were before that
    charge (`raw_status`, `raw_seconds` and the other CHARGED_TIMES), its
    answers back where the charge put them past the limit, and no
    `translation`. A PSouffle compilation or a ground route charged with it
    stays in them. A record that pays no conversion is r itself. The
    circuits Alea.jl exports are read so (no one pays the
    export), and Alea.jl+tddlog's record with it."""
    if RAW + 'status' not in r:
        return r
    out = {k: v for k, v in r.items() if not k.startswith(RAW) and k not in ('translation', 'translation_seconds')}
    out['status'] = r[RAW + 'status']
    out.update({k: r[RAW + k] for k in CHARGED_TIMES if RAW + k in r})
    if 'answers_past_limit' in out and out['status'] != 'timeout':
        out['answers'] = out.pop('answers_past_limit')
    return out


def has_cnf(entry):
    """Whether `gen` writes a weighted CNF beside the instance's program,
    for the weighted model counters (Ganak, sharpSAT-TD): the CNF
    of an extra suite's instance, uncompressed, and a UAI network's weighted CNF with its
    evidence (`gen_bn.py --cnf`)."""
    gargs = entry[1]
    return gargs[0] in CNF_WRITERS or (gargs[0] == 'gen_bn.py' and gargs[1].removesuffix('.gz').endswith('.uai'))


def network_file(args, gargs):
    """The network file of a gen_bn.py instance as tddlog reads it: the file
    itself, or a gzipped one's copy that `gen` decompresses once into
    networks/ of the instance's directory (data/bn-large's), outside its
    time."""
    source = gargs[1]
    if not source.endswith('.gz'):
        return source
    return os.path.join(os.path.abspath(args.dir), 'networks', os.path.basename(source).removesuffix('.gz'))


def network_args(args, name, gargs, no_queries):
    """tddlog's arguments that read the network of a gen_bn.py instance
    itself, as the program gen_bn.py writes of it (its first argument the
    file, `network_file`), and the files they name: the queries (`--query`,
    or `--no-queries` for gen_bn.py's `--query none`; neither with
    `no_queries`, where the caller asks for the program of P(e) alone), the
    observations (`--evidence`: Ace's .inst file, a UAI .evid file, or for
    sampled ones the .inst file `gen` writes of them) and `--rescale`."""
    a = gen_bn.arguments().parse_args(gargs[1:])
    if a.format != 'problog':
        raise SystemExit(f'{name}: tddlog reads a network as gen_bn.py writes its ProbLog program, '
                         f'and the instance asks for --format {a.format}')
    out = [network_file(args, gargs)]
    if not no_queries:
        out += ['--no-queries'] if a.query == 'none' else ['--query', a.query]
    if a.evidence:
        if getattr(args, 'pr', False):
            raise SystemExit(f'{name}: its observations are sampled into the .inst file a plain gen writes, '
                             'and --pr writes none')
        out += ['--evidence', os.path.abspath(os.path.join(args.dir, name + '.inst'))]
    for f in (a.evidence_file, a.evid):
        if f:
            out += ['--evidence', f]
    if a.rescale:
        out.append('--rescale')
    return out, [out[0]] + [out[i + 1] for i, x in enumerate(out) if x == '--evidence']


def original(gargs, variant, stem=None):
    """The file tddlog reads itself where NAME.pl is its reader's
    conversion of a whole file (`write_program`): a Dice program
    (dice2problog.py; one with places is `folded`'s), a cplint LPAD
    (gen_cplint.py) or a Circulette netlist (gen_circulette.py, NAME.cql
    beside NAME.pl: `stem` is DIR/NAME, the instance's), so that reading it
    is part of tddlog's time, as a network's is (`network_args`). None for
    another instance."""
    if gargs[0] == 'dice2problog.py':
        return gargs[1]
    if gargs[0] == 'gen_cplint.py':
        return os.path.join(ROOT, 'data', (gargs[2:] or ['cplint'])[0], gargs[1] + '.pl')   # gen_cplint.py NAME [swish]
    if gargs[0] == 'gen_circulette.py':
        if stem is None:
            raise ValueError("a Circulette instance's netlist is gen's, in its directory: original needs its stem")
        return os.path.abspath(stem + '.cql')
    return None


def folded(args, name, gargs, variant):
    """What tddlog reads of an instance of FOLD: its family's program and
    its arguments' file, (DIR/programs/PROGRAM.pl, DIR/NAME.args), so that
    reading the program, its data and binding its places is part of
    tddlog's time. Of a scaled Dice loop, D/dice_LOOP_kK.dice (dice_scale.py)
    beside its count's file D/NAME.args, likewise the loop's
    program, whose iterate count is the place $1, (D/programs/LOOP.dice,
    D/NAME.args). Of a Dice program with places, its arguments after it
    (ARITH_PLACES), the program and DIR/NAME.args, which gen writes. None
    for another instance."""
    if gargs[0] == 'dice2problog.py' and len(gargs) > 2:
        return gargs[1], os.path.join(args.dir, name + '.args')
    if gargs[0] == 'dice2problog.py':
        d, m = os.path.dirname(gargs[1]), SCALED_LOOP.fullmatch(name)
        given = os.path.join(d, name + '.args')
        return (os.path.join(d, FOLD_DIR, m.group(1) + '.dice'), given) if m and os.path.exists(given) else None
    fold = fold_of(gargs)
    if fold is None:
        return None
    return os.path.join(args.dir, FOLD_DIR, fold[0] + '.pl'), os.path.join(args.dir, name + '.args')


def folded_data(args, gargs):
    """The data files of an instance of FOLD, DIR/data/KEY.pl for each key
    of its family's entry, in the order of its places (none for a scaled
    Dice loop)."""
    fold = fold_of(gargs)
    return [os.path.join(args.dir, DATA_DIR, key + '.pl') for key in fold[2]] if fold else []


def arguments_of(path):
    """The arguments of a FOLD instance, a line each of its NAME.args."""
    with open(path) as f:
        return [line for line in f.read().splitlines() if line]


def fold_run(gargs, *words):
    """The text a FOLD generator prints for `words` (--template, --arguments
    or --data KEY) after an instance's own arguments."""
    return subprocess.run([sys.executable, str(source_tree.script(gargs[0]))] + words[0] + list(words[1:]),
                          capture_output=True, text=True, check=True).stdout


def fold_template(gargs):
    """The program of a FOLD instance, (its file's name, its text): its
    generator's --template."""
    program, targs, _ = fold_of(gargs)
    return program + '.pl', fold_run(gargs, targs, '--template')


def fold_data(gargs, key):
    """The text of the data file KEY of a FOLD instance: its generator's
    --data KEY."""
    return fold_run(gargs, gargs[1:], '--data', key)


def fold_arguments(gargs):
    """The arguments of a FOLD instance, a line each: its data files,
    quoted atoms relative to the program's directory (consult adds the
    `.pl`), then its generator's --arguments."""
    data = [f"'../{DATA_DIR}/{key}'" for key in fold_of(gargs)[2]]
    return data + fold_run(gargs, gargs[1:], '--arguments').splitlines()


def folded_program(gargs, binary=None):
    """The text of a FOLD instance's NAME.pl, without a directory of
    instances: its program and data, written to a temporary directory, with
    its arguments in its places and its consults inlined (`tddlog
    --program`, `binary` as source_tree.program takes it)."""
    import tempfile
    name, text = fold_template(gargs)
    with tempfile.TemporaryDirectory() as d:
        os.makedirs(os.path.join(d, FOLD_DIR))
        os.makedirs(os.path.join(d, DATA_DIR))
        for key in fold_of(gargs)[2]:
            with open(os.path.join(d, DATA_DIR, key + '.pl'), 'w') as f:
                f.write(fold_data(gargs, key))
        path = os.path.join(d, FOLD_DIR, name)
        with open(path, 'w') as f:
            f.write(text)
        return source_tree.program(path, *fold_arguments(gargs), binary=binary)


def write_once(path, text, written):
    """Writes `text` to `path` unless this gen did (`written`, the paths so far).
    Gens running in parallel may write the same data file, with the same
    text: each writes a part of its own and replaces `path` with it."""
    if path not in written:
        os.makedirs(os.path.dirname(path), exist_ok=True)
        part = f'{path}.{os.getpid()}.part'
        with open(part, 'w') as f:
            f.write(text)
        os.replace(part, path)
        written.add(path)


def gen_folded(args, name, gargs, written):
    """`gen` of an instance of FOLD: its family's program and its data
    files (each once a gen, `written` the paths so far), its arguments
    (NAME.args) and the program with them in its places and its data
    consulted (NAME.pl, `write_program`)."""
    program, _, keys = fold_of(gargs)
    path = os.path.join(args.dir, FOLD_DIR, program + '.pl')
    if path not in written:
        write_once(path, fold_template(gargs)[1], written)
    for key, file in zip(keys, folded_data(args, gargs)):
        if file not in written:
            write_once(file, fold_data(gargs, key), written)
    words = fold_arguments(gargs)
    with open(os.path.join(args.dir, name + '.args'), 'w') as f:
        f.write(''.join(w + '\n' for w in words))
    write_program(args, name, [path] + words)


def program_args(args, name, gargs):
    """The arguments of `tddlog --program` that write the program `gen`
    writes of a gen_bn.py instance (`gen_network`): the network with its
    queries and observations, or under --pr its observations alone (gen_bn.py's
    `--query none`)."""
    pr = getattr(args, 'pr', False)
    argv, _ = network_args(args, name, gargs, pr)
    return argv + ['--no-queries'] if pr else argv


def evidence_file(gargs):
    """The .inst file a gen_bn.py instance takes its observations from
    (`--evidence-file`: an evidence set of Ace's archive), else None."""
    return gen_bn.arguments().parse_args(gargs[1:]).evidence_file


def ace_observations(args, name, gargs):
    """The .inst file Ace reads for a gen_bn.py instance: the one the
    instance names (`evidence_file`), which tddlog reads too, so that a
    network's page has one file of its observations; else the one `gen`
    writes beside the program (NAME.inst) of the observations it samples or
    converts."""
    return os.path.abspath(evidence_file(gargs) or os.path.join(args.dir, name + '.inst'))


def gen_network(args, name, gargs):
    """`gen` of a gen_bn.py instance: the gzipped network decompressed once
    (`network_file`); the files only the baselines read, written by
    gen_bn.py (the observations as Ace's .inst file, which tddlog reads
    where they are sampled, unless the instance names an .inst file of its
    own, which Ace and tddlog both read as it is (`ace_observations`); the
    network as a UAI file for Merlin; the
    weighted CNF under --cnf), its program discarded; and NAME.pl, the
    program the systems that read ProbLog programs are given, as tddlog's
    reader writes it (`tddlog --program`, the same text gen_bn.py writes),
    so that there is one conversion of a network into a program."""
    net = network_file(args, gargs)
    if net != gargs[1] and not os.path.exists(net):
        os.makedirs(os.path.dirname(net), exist_ok=True)
        with gzip.open(gargs[1], 'rb') as f, open(net + '.part', 'wb') as g:
            shutil.copyfileobj(f, g)
        os.replace(net + '.part', net)
    side = []
    if not getattr(args, 'pr', False):
        if not evidence_file(gargs):
            side += ['--inst', os.path.join(args.dir, name + '.inst')]
        if not gargs[1].endswith('.uai'):
            side += ['--uai', os.path.join(args.dir, name + '.uai')]
    if has_cnf((None, gargs)) and getattr(args, 'cnf', False):
        side += ['--cnf', os.path.join(args.dir, name + '.cnf')]
    if side:
        # Under --pr the CNF alone, of the program for the partition function.
        pr = ['--query', 'none'] if getattr(args, 'pr', False) else []
        subprocess.run([sys.executable, str(source_tree.script('gen_bn.py')), *gargs[1:], *pr, *side],
                       stdout=subprocess.DEVNULL, check=True)
    write_program(args, name, program_args(args, name, gargs))
    if uai_kind(net) == 'BAYES':
        # A Bayesian network's Z is P(e): its program states log10 Z -
        # log10 P(e) = 0 on its first line, as a Markov network's states its
        # offset, so that `program_offset` reads log10 Z from any system's P(e).
        pl = os.path.join(args.dir, name + '.pl')
        with open(pl) as f:
            text = f.read()
        if gen_bn.read_offset(text) is None:
            with open(pl + '.part', 'w') as f:
                f.write(gen_bn.offset_line(0.0) + text)
            os.replace(pl + '.part', pl)


def uai_kind(path):
    """BAYES or MARKOV, the first word of a UAI model file; None for a file
    in another format."""
    if not path.endswith('.uai'):
        return None
    with open(path) as f:
        words = f.read(64).split()
    return words[0] if words else None


PLACE = re.compile(r'\$(\d+)')


def fill_places(text, words):
    """A Dice program's text with its places ($1, $2, ...) given `words`,
    outside its `//` comments, as tddlog binds them: the program the
    systems that take no arguments read, the same program. ValueError on a
    place without a word or a word without a place."""
    used, out = set(), []
    for line in text.splitlines(keepends=True):
        code, sep, comment = line.partition('//')

        def word(m):
            k = int(m.group(1))
            if not 1 <= k <= len(words):
                raise ValueError(f'place ${k} of a program given {len(words)} arguments')
            used.add(k)
            return words[k - 1]
        out.append(PLACE.sub(word, code) + sep + comment)
    if used != set(range(1, len(words) + 1)):
        raise ValueError(f'{len(words)} arguments for the places {sorted(used)}')
    return ''.join(out)


def write_program(args, name, argv):
    """NAME.pl of instance `name`, the program tddlog reads a file as
    (`tddlog --program ARGV`, gen's --tddlog): a network's (`gen_network`),
    a Dice program's, a Circulette netlist's (`gen_circulette`), or a FOLD
    program's with its arguments in its places (`gen_folded`), which the
    systems that read ProbLog programs are given."""
    part = os.path.join(args.dir, name + '.pl.part')
    with open(part, 'w') as f:
        source_tree.program(*argv, stdout=f, binary=getattr(args, 'tddlog', None))
    os.replace(part, os.path.join(args.dir, name + '.pl'))


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
        # a few tries: one of the hosts of Ihler's collection presents another
        # host's certificate, and the next connection reaches another
        for attempt in range(6):
            try:
                with urllib.request.urlopen(url, timeout=args.timeout) as r:
                    data = r.read()
                break
            except OSError:
                if attempt == 5:
                    raise
                time.sleep(2 + 2 * attempt)
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
    netlists = [n for n, e in spec.items() if e[1][0] == 'gen_circulette.py']
    if getattr(args, 'circulette_netlists', '') and not netlists:
        raise SystemExit("--circulette-netlists gives the Circulette instances' netlists, and none is selected")
    if netlists and not getattr(args, 'circulette_netlists', ''):
        raise SystemExit(f"{netlists[0]} is a Circulette netlist, which gen copies from --circulette-netlists DIR "
                         f'({len(netlists)} instances)')
    tddlog = getattr(args, 'tddlog', None) or str(source_tree.BINARY)
    os.environ['TDDLOG'] = tddlog   # the generators that convert an LPAD (--template of gen_uwcse.py, gen_pedigree.py)
    if any(e[1][0] in PROGRAM_OF_TDDLOG or fold_of(e[1]) for e in spec.values()) and not os.access(tddlog, os.X_OK):
        raise SystemExit(f'--tddlog {tddlog}: not an executable, and gen writes the program of a network, a Dice '
                         'program, an LPAD or a program given arguments with `tddlog --program`')
    written = set()   # the FOLD programs this gen wrote
    for name, entry in spec.items():
        gargs = entry[1]
        if gargs[0] == 'alea_bench.jl':
            # NAME.pl, NAME__dJ.dice and NAME.dicemap.
            subprocess.run(alea_command(args, 'export', gargs[1:], os.path.abspath(os.path.join(args.dir, name))),
                           check=True, stdout=subprocess.DEVNULL)
            continue
        if gargs[0] == 'gen_bn.py':
            # Under --pr the programs for the partition function alone,
            # without the observations: run --pr gives Ace the .inst files
            # of a plain gen.
            gen_network(args, name, gargs)
            continue
        if gargs[0] == 'dice2problog.py':
            if len(gargs) > 2:
                # a program with places: the instance's arguments, a line each (NAME.args, which
                # tddlog reads beside the program), and the program with them in its places
                # (NAME.dice, which the systems that take no arguments read)
                write_new(os.path.join(args.dir, name + '.args'), ''.join(w + '\n' for w in gargs[2:]))
                with open(gargs[1]) as f:
                    write_new(os.path.join(args.dir, name + '.dice'), fill_places(f.read(), gargs[2:]))
            write_program(args, name, gargs[1:])
            continue
        if gargs[0] == 'gen_circulette.py':
            gen_circulette(args, name, gargs)
            continue
        if fold_of(gargs):
            gen_folded(args, name, gargs, written)
            if writes_dice(gargs):   # the roulette suite's: the instance's Dice program
                subprocess.run([sys.executable, str(source_tree.script(gargs[0]))] + gargs[1:] +
                               ['--dice', os.path.join(args.dir, name + '.dice')], check=True)
            if dice_ported(gargs):   # and the loop the other translators read (dice_source)
                subprocess.run([sys.executable, str(source_tree.script(gargs[0]))] + gargs[1:] +
                               ['--dice-port', os.path.join(args.dir, name + '.port.dice')], check=True)
            continue
        cmd = [sys.executable, str(source_tree.script(gargs[0]))] + gargs[1:]
        if writes_dice(gargs):
            cmd += ['--dice', os.path.join(args.dir, name + '.dice')]
        if gargs[0] in SOUFFLE_WRITERS:
            cmd += ['--souffle', os.path.join(args.dir, name + '.souffle')]
        if has_cnf(entry) and (gargs[0] in CNF_WRITERS or args.cnf):
            cmd += ['--cnf', os.path.join(args.dir, name + '.cnf')]
        # a generator that converts an LPAD or a Dice program calls `tddlog
        # --program` (source_tree.program) with gen's --tddlog
        out = subprocess.run(cmd, capture_output=True, text=True, check=True,
                             env=dict(os.environ, TDDLOG=tddlog)).stdout
        write_new(os.path.join(args.dir, name + '.pl'), out)
    print(f'{len(spec)} instances in {args.dir}')


def gen_circulette(args, name, gargs):
    """Writes a Circulette instance's netlist NAME.cql, a copy of
    --circulette-netlists' NAME.cql whose SHA-256 is CT_SHA256's, and the
    program tddlog's reader writes of it, NAME.pl (`write_program`); a
    netlist that is missing or another file, or that the reader refuses,
    stops gen."""
    src = os.path.join(args.circulette_netlists, name + '.cql')
    if name not in CT_SHA256:
        raise SystemExit(f'{name}: no SHA-256 in CT_SHA256 for its netlist')
    try:
        with open(src, 'rb') as f:
            data = f.read()
    except OSError as e:
        raise SystemExit(f'{name}: {e}')
    got = hashlib.sha256(data).hexdigest()
    if got != CT_SHA256[name]:
        raise SystemExit(f'{src}: SHA-256 {got}, not {CT_SHA256[name]}, the netlist the records read')
    cql = original(gargs, '', os.path.join(args.dir, name))
    with open(cql + '.part', 'wb') as f:
        f.write(data)
    os.replace(cql + '.part', cql)
    write_program(args, name, [cql])


def parse_answers(out):
    res = {}
    for line in out.splitlines():
        if line.startswith('evidence is inconsistent'):
            return 'inconsistent'
        if line.startswith((EVIDENCE_LINE, Z_LINE)):
            continue
        if ':' in line:
            q, v = line.rsplit(':', 1)
            try:
                res[re.sub(r'\s+', '', q)] = float(v)
            except ValueError:
                pass
    return res


# A system that stops on evidence of probability 0 says so, and that is its
# answer, as tddlog's `evidence is inconsistent` is: ProbLog's
# InconsistentEvidenceError (problog_each.py reads it so too), PSouffle's
# `Inconsistent evidence: A is deterministically false`.
INCONSISTENT = re.compile(r'InconsistentEvidenceError|Inconsistent evidence')
EVIDENCE_LINE = 'P(e):'
# The line tddlog prints in its place for a Markov network it reads itself:
# the partition function given the evidence, `Z:<tab>z<tab>log10 l`.
Z_LINE = 'Z:'


def parse_evidence_probability(out, label=EVIDENCE_LINE):
    """log10 P(e) from the line `--evidence-probability` prints after the
    answers, `P(e):<tab>p<tab>log10 l` (tddlog and bn_exact.py alike), `-inf`
    for inconsistent evidence; None without the line. With `label` Z_LINE,
    log10 Z from tddlog's line of the same form."""
    for line in out.splitlines():
        if line.startswith(label):
            fields = line.split('\t')
            if len(fields) != 3 or not fields[2].startswith('log10 '):
                raise ValueError(f'not an evidence line: {line!r}')
            return float(fields[2].removeprefix('log10 '))
    return None


def network_partition_function(out):
    """log10 Z of a run of tddlog on a network file (`network_args`) that
    printed its `--evidence-probability` line: the `Z:` line's, which it
    prints for a Markov network (the factors' normalization applied), or
    for a Bayesian network, whose partition function given the evidence is
    P(e), the `P(e):` line's. A ValueError where it printed neither."""
    for label in (Z_LINE, EVIDENCE_LINE):
        log10 = parse_evidence_probability(out, label)
        if log10 is not None:
            return log10
    raise ValueError('no Z or P(e) line')


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


# The systems that read Dice programs.
DICE_SYSTEMS = ('dice', 'alea', 'roulette', 'nodice', 'psi', 'genfer', 'sppl', 'geni', 'pluck', 'webppl')
# Under --pr the systems that print no P(e) are asked it as a query of a
# program of their own (`pe_program`), and the systems that read Dice
# programs (Storm and PRISM their Markov chains, `run_translated`) the
# probability that a network's program returns true where it
# returns the conjunction of the observations (`dice_source`), Roulette and
# Roulette+tddlog that of the conjunction in Roulette's BN language
# (`roulette_bif`), with the program's query `bench_pe` naming it.
PE_SYSTEMS = ('problog', 'pita', 'aspmc', 'scallop', *DICE_SYSTEMS, 'roulette-tddlog', 'storm', 'prism')
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
    write_new(out, ''.join(kept) + f'{PE_QUERY} :- {body}.\nquery({PE_QUERY}).\n')
    return out


# The answer check, the one the run and the processing make: an answer a is
# right against its reference's r when |a - r| <= 1e-9 (floating-point
# noise), or when |a - r| <= 1e-6 + e(a) + e(r) and |a - r| <= 1e-3 |r| +
# e(a) + e(r). e(x) is half a unit in the last place a system prints
# (PRINTED; noDice's last digit is not rounded, a whole unit), so float32
# and a printed last digit pass, while an answer off by more than 1e-9 and
# by orders of magnitude (3e-8 for 2e-12) fails.
# log10 Z keeps its own check: CHECK_TOL times max(1, |log10 Z|).
CHECK_TOL = 1e-6
CHECK_REL = 1e-3
CHECK_NOISE = 1e-9
CHECK_TEXT = ('off the reference by more than 1e-9 and either by more than 1e-6 or by more than 1e-3 of it, beyond '
              'the last digit either of the two prints')
# What a system prints of a probability: ('dec', d) d decimals, ('sig', k) k
# significant digits, ('unit', u) answers good to u; any other in full. Geni
# printed seven decimals of its masses before its build printed them in full
# (run_geni; kept while records of the earlier build stand), noDice six decimals
# whose last is not rounded, Merlin six (its MAR file), ProbLog and PSouffle
# eight significant digits, the network tools' drivers twelve, the exact
# references written for the evaluation 17 decimals.
PRINTED = {'merlin': ('dec', 6), 'nodice': ('unit', 1e-6), 'geni': ('dec', 7), 'problog': ('sig', 8),
           'psouffle': ('sig', 8), 'agrum': ('sig', 12), 'libdai': ('sig', 12), 'pgmpy': ('sig', 12),
           ('exact', 'cliquetree'): ('dec', 17), ('exact', 'lit'): ('dec', 17), ('exact', 'tpch'): ('dec', 17),
           ('exact', 'reliability'): ('dec', 17), ('exact', 'lifted'): ('dec', 17)}
# An answer's reference: an exact record (the solvers written for the
# evaluation, bn_exact.py and the like, or the generator's own), then
# the systems that compute in exact arithmetic (Psi's rationals, Storm's
# exact engine, Genfer's rational runs on its own Dice programs:
# reference_variant), then Alea.jl's, ProbLog's, Merlin's (six decimals)
# and Ace's records, which compute in doubles; the first of them that
# answered a query of the record checked and comes before it in this order
# (the instance's reference itself is checked against none); tddlog's only
# where none did (the taint apps), and never for tddlog's own. Ace
# computes in doubles and underflows (below): an Ace answer of 0 or NaN is
# never a reference (reference_answers).
REFERENCE_ORDER = [('exact', None), ('psi', None), ('storm', 'exact'), ('genfer', 'rational'), ('alea', None),
                   ('problog', 'ddnnf'), ('problog', 'sdd'), ('problog', 'fsdd'), ('merlin', 'bte'), ('ace', None)]
UNDERFLOWS = {'ace'}
# On the networks of --suite uai, observed factors take P(e) as low as
# 1e-618, and a baseline that computes in doubles can answer wrongly (Ace
# prints P(e) = 0 and posteriors that are not numbers where it underflows).
# There every baseline's answer is checked against the instance's reference
# once all its systems have run (`check_against_reference`): the marginals
# by the answer check, log10 Z against the first of PR_REFS.
CHECKED_SUITES = ('uai',)
PR_REFS = [('exact', 'cliquetree'), ('merlin', 'bte-pr'), ('ganak', 'wmc'), ('tddlog', None)]

def printed_error(system, variant, x):
    """e(x): half a unit in the last place system:variant prints of x (PRINTED), 0 in full."""
    p = PRINTED.get((system, variant)) or PRINTED.get(system)
    if p is None:
        return 0.0
    kind, n = p
    if kind == 'unit':
        return n
    if kind == 'dec':
        return 0.5 * 10.0 ** -n
    return 0.0 if x == 0 or not math.isfinite(x) else 0.5 * 10.0 ** (math.floor(math.log10(abs(x))) - n + 1)


def answer_right(a, r, ea=0.0, er=0.0):
    """The answer check of a against the reference's r, with their printed errors e(a) and e(r)."""
    if not isinstance(a, (int, float)) or not isinstance(r, (int, float)):
        return a == r
    d = answer_diff(a, r)
    return d <= CHECK_NOISE or (d <= CHECK_TOL + ea + er and d <= CHECK_REL * abs(r) + ea + er)


def atom_args(q):
    """(name, top-level arguments) of the atom q as ProbLog prints it; (q, []) for a constant."""
    m = re.fullmatch(r'([^(]+)\((.*)\)', q)
    if not m:
        return q, []
    args, depth, cur = [], 0, ''
    for c in m.group(2):
        if c == ',' and depth == 0:
            args.append(cur.strip()); cur = ''
            continue
        depth += (c in '([') - (c in ')]')
        cur += c
    return m.group(1), args + [cur.strip()]


def instances_of(q, keys):
    """The keys of `keys` that are ground instances of the atom q, whose
    arguments include variables (ProbLog prints a query none of whose
    instances it derives with its variables: different_color(X3,X4)); None
    where q has no variable as an argument."""
    name, args = atom_args(q)
    var = [bool(re.fullmatch(r'[A-Z_]\w*', x)) for x in args]
    if not any(var):
        return None
    out = []
    for k in keys:
        n, xs = atom_args(k)
        if n != name or len(xs) != len(args):
            continue
        bound = {}
        if all(bound.setdefault(p, x) == x if v and p != '_' else v or p == x for p, x, v in zip(args, xs, var)):
            out.append(k)
    return out


def left_out(rec, ref):
    """The queries of reference ref's answers that record rec leaves out and
    does not answer by leaving them out. A query rec does not print is
    answered 0: the systems that print only what they derive mean it so
    (aspmc, whose clingo grounding removes an atom that no rule derives;
    PITA, whose prob/2 answers a query with variables for the instances it
    derives; Scallop and PSouffle, which print the tuples they derive), and
    that answer is right where the reference's is 0 to the check's noise and
    its printed digits. A query left out where the reference's answer is not
    0 is an answer dropped (a translator's empty relation, a truncated
    output). Roulette on a Circulette instance answers a posterior at or
    below Circulette's printing threshold by leaving it out (printed_answers).
    Answers of rec's that are not a dict answer no query; none is left out
    of a reference whose answers are not a dict."""
    a, b = printed_answers(rec, ref), ref.get('answers')
    if not isinstance(b, dict):
        return []
    a = a if isinstance(a, dict) else {}
    return [q for q in b if q not in a
            and not answer_right(0.0, b[q], 0.0, printed_error(ref['system'], ref.get('variant'), b[q]))]


def answers_check(rec, ref):
    """(right, the largest difference, the queries compared) of record rec's
    answers against reference ref's, over the queries both answer; None where
    they share none. A reference's query with variables at 0 (ProbLog derives
    none of its instances) is compared with each of rec's instances of it,
    which must be 0 too (tddlog grounds them over the domain). A query of
    ref's that rec leaves out where ref's answer is not 0 (left_out) makes
    rec wrong, at an infinite difference: a record that drops answers (a
    translator's empty relation, a truncated output) answers less than was
    asked, whatever the queries both answer show. Answers that are not a dict
    (the evidence inconsistent to the system) are right where both are
    equal."""
    a, b = rec.get('answers'), ref.get('answers')
    if not isinstance(a, dict) or not isinstance(b, dict):
        return (a == b, 0.0 if a == b else math.inf, 1) if a is not None and b is not None else None
    qs = [q for q in b if q in a]
    zero = {q: ks for q in b if q not in a and b[q] == 0 for ks in [instances_of(q, a)] if ks}
    pairs = [(q, q) for q in qs] + [(k, q) for q, ks in zero.items() for k in ks]
    if not pairs:
        return None
    gone = left_out(rec, ref)
    right = not gone and all(answer_right(a[k], b[q], printed_error(rec['system'], rec.get('variant'), a[k]),
                                          printed_error(ref['system'], ref.get('variant'), b[q])) for k, q in pairs)
    worst = max([answer_diff(a[k], b[q]) if all(isinstance(x, (int, float)) for x in (a[k], b[q]))
                 else (0.0 if a[k] == b[q] else math.inf) for k, q in pairs] + [math.inf] * bool(gone))
    return right, worst, len(qs) + len(zero)


def reference_answers(rec):
    """The answers of rec that may stand as a reference: all of them, but
    for a system that computes in doubles and underflows (UNDERFLOWS: Ace)
    those that are not 0 or NaN, so that an underflowed 0 never makes
    another system's answer wrong."""
    a = rec.get('answers')
    if rec['system'] not in UNDERFLOWS or not isinstance(a, dict):
        return a
    return {q: p for q, p in a.items() if not (isinstance(p, (int, float)) and (p == 0 or math.isnan(p)))}


def reference_variant(r):
    """Record r's variant as REFERENCE_ORDER names it: `rational` for
    Genfer's run on one of its own Dice programs (GENFER_DICE, run_genfer's
    --rational), which computes in exact rationals; its other runs compute
    in floating point and are no reference."""
    if r['system'] == 'genfer':
        return 'rational' if r.get('instance') in GENFER_DICE else None
    return r.get('variant')


def own_question(r):
    """Whether record r answers a question of its own, not its instance's
    queries: Psi's run on its own linear regression (run_psi_linreg), whose
    E(a) and E(b) are the continuous model's posterior means, where the
    circuit's queries are the bits of the discretized one. Such a record is
    no reference (reference_records), and no reference holds it to the
    instance's queries (full_reference): it shares none of them."""
    return r['system'] == 'psi' and psi_linreg(str(r.get('instance', '')))


def reference_records(records):
    """The records an answer is checked against, of `records` (one
    instance's): those that answered (ok, a non-empty dict of finite
    answers, reference_answers's), in REFERENCE_ORDER, then tddlog's
    default arm (`auto`; another arm may answer
    other queries). An Ace record stands with its reference_answers alone,
    a copy. A record that answers a question of its own (own_question) is
    none. Where none answered so, those of the evidence found inconsistent,
    in the same order. The first is the instance's reference."""
    def rank(r):
        if own_question(r):
            return None
        for k, (s, v) in enumerate(REFERENCE_ORDER):
            if r['system'] == s and v in (None, reference_variant(r)):
                return k
        return len(REFERENCE_ORDER) if r['system'] == 'tddlog' and r.get('variant') in (None, 'auto') else None
    answered = []
    for r in records:
        if r.get('status') != 'ok' or rank(r) is None:
            continue
        a = reference_answers(r)
        if isinstance(a, dict) and a and all(isinstance(p, (int, float)) and math.isfinite(p) for p in a.values()):
            answered.append(r if a is r.get('answers') else dict(r, answers=a))
    if not answered:
        # where none answers a query, those that find the evidence inconsistent: a
        # program whose evidence has probability 0 and that asks nothing else (Circulette's
        # ct_ALU_Controller, which no fault fails), where a record that does not say so
        # answers wrongly
        answered = [r for r in records if r.get('status') == 'ok' and rank(r) is not None
                    and r.get('answers') == 'inconsistent']
    return sorted(answered, key=rank)


def same_record(a, b):
    """Whether records a and b are of one system and variant (a reference
    may be a copy: reference_records)."""
    return (a['system'], a.get('variant')) == (b['system'], b.get('variant'))


def references_above(rec, refs):
    """The records of `refs` (reference_records's) that rec is checked
    against: those before rec's own system and variant, all of them where
    rec is not among them; tddlog's never for tddlog's own. The instance's
    reference itself has none."""
    k = next((k for k, x in enumerate(refs) if (x['system'], x.get('variant')) == (rec['system'], rec.get('variant'))),
             len(refs))
    return [x for x in refs[:k] if not (rec['system'] == x['system'] == 'tddlog')]


def answer_reference(rec, refs):
    """The first of references_above(rec, refs) that shares a query with rec,
    None where none does."""
    return next((x for x in references_above(rec, refs) if answers_check(rec, x) is not None), None)


def full_reference(rec, refs):
    """The reference whose every query rec must answer (a record
    that leaves a query out answers less than was asked):
    answer_reference's, else the first above rec, where rec shares a query
    with none. Not for another arm of tddlog's than its default, which may
    answer other queries (it may name its own: reference_records),
    and is checked where it shares one; none for a record that answers a
    question of its own (own_question)."""
    if own_question(rec):
        return None
    x = answer_reference(rec, refs)
    if x is not None or (rec['system'] == 'tddlog' and rec.get('variant') not in (None, 'auto')):
        return x
    return next(iter(references_above(rec, refs)), None)


def check_against_reference(rec, done):
    """Checks a baseline's record `rec` of an instance against the
    reference among `done`, the instance's records by (system, variant): an
    answer that fails the answer check (answers_check, against
    answer_reference) or a log10 Z further from the reference's than
    CHECK_TOL times its size makes the record an error, a failed run, as a
    crash would. The record keeps its answers, the reference it was checked
    against and the difference (`ref`, `ref_diff`; `pr_ref`, `pr_ref_diff`
    for log10 Z). A record with nothing to check against stays as it is, and
    so does the reference itself."""
    if rec['status'] == 'ok' and 'answers' in rec:
        ref = answer_reference(rec, reference_records(done.values()))
        if ref is not None:
            right, d, _ = answers_check(rec, ref)
            rec['ref'], rec['ref_diff'] = f"{ref['system']}:{ref['variant']}", d
            if not right:
                rec['status'] = 'error'
                rec['error'] = f"the marginals differ from {rec['ref']}'s by {d:.2e}, {CHECK_TEXT}"
    if rec['status'] != 'ok' or 'log10_z' not in rec:
        return
    for system, variant in PR_REFS:
        ref = next((r for (s, v), r in done.items() if s == system and variant in (None, v)
                    and r['status'] == 'ok' and 'log10_z' in r), None)
        if ref is None:
            continue
        if ref is rec:
            break
        z = ref['log10_z']
        d, tol = pr_diff(rec['log10_z'], z), CHECK_TOL * max(1.0, abs(z)) if math.isfinite(z) else 0.0
        rec['pr_ref'], rec['pr_ref_diff'] = f"{ref['system']}:{ref['variant']}", d
        if d > tol:
            rec['status'] = 'error'
            rec['error'] = f"log10 Z differs from {rec['pr_ref']}'s by {d:.2e}, more than {tol:.1e}"
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
# or None without `--cpus`; and the memory the record under way has used
# so far (`usage`, which `run_job` resets for each record), or None.
LANE = threading.local()
# The commands under way, which a stopped driver kills (`stop_all`), and
# whether it has been stopped.
LIVE = set()
LIVE_LOCK = threading.Lock()
STOPPED = threading.Event()
# The run's resident memory cap (`cap_memory`): (memcap.Root, bytes), or None.
MEMCAP = None
# The error of a record whose commands the cap killed (`memout` reads it).
CAP_KILLED = 'killed by the memory cap'


def cap_error(cap_kb):
    return f'{CAP_KILLED} of {cap_kb >> 10} MiB of resident memory'


def cap_memory(limit):
    """Caps every command `timed` runs at `limit` bytes of resident memory,
    each in a cgroup of its own (memcap.py), and sets the systems' own
    memory options from it (`memory_options`). An address-space limit
    beside it would still refuse reservations below the cap, so a run
    that has one is refused."""
    global MEMCAP
    import resource
    soft = resource.getrlimit(resource.RLIMIT_AS)[0]
    if soft != resource.RLIM_INFINITY:
        raise SystemExit(f'--mem-cap caps resident memory, and the run has an address-space limit too '
                         f'(ulimit -v {soft >> 10}): start it without one')
    try:
        MEMCAP = (memcap.Root(), limit)
    except memcap.MemcapError as e:
        raise SystemExit(f'--mem-cap: {e}')


def memory_bound():
    """The memory a command may take, in bytes: the cap, else the
    address-space limit, else None."""
    if MEMCAP:
        return MEMCAP[1]
    import resource
    soft = resource.getrlimit(resource.RLIMIT_AS)[0]
    return None if soft == resource.RLIM_INFINITY else soft


def julia_heap_hint(cmd, limit):
    """cmd with `--heap-size-hint` at 0.8 of `limit` after its Julia
    executable, where it runs Julia without one: Julia's collector
    otherwise aims at a share of the machine's memory and lets garbage
    grow past the cap before it collects."""
    i = next((k for k, a in enumerate(cmd) if os.path.basename(a) == 'julia'), None)
    if i is None or any(a.startswith('--heap-size-hint') for a in cmd):
        return cmd
    return cmd[:i + 1] + [f'--heap-size-hint={limit * 8 // 10 >> 20}M'] + cmd[i + 1:]


class _Popen(subprocess.Popen):
    """A Popen that keeps its process's resource usage (`os.wait4`): its
    `ru_maxrss` is the largest peak resident set of the process and of the
    descendants it waited for, each on its own."""
    rusage = None

    def _try_wait(self, wait_flags):
        try:
            pid, sts, ru = os.wait4(self.pid, wait_flags)
        except ChildProcessError:
            return self.pid, 0
        if pid == self.pid:
            self.rusage = ru
        return pid, sts


def take_usage(rec, r):
    """Puts the memory of the record's commands (`LANE.usage`, which the
    caller set to {} before the first) into record rec, whose run's result
    is r, and returns r: where the cap killed one of the commands and rec
    failed, rec and the returned result are that memout, whatever the
    system made of the kill."""
    usage, LANE.usage = getattr(LANE, 'usage', None) or {}, None
    if usage.pop('cap_killed', False) and rec['status'] != 'ok':
        r = {'status': 'error', 'error': cap_error(usage['mem_cap_kb'])}
        rec['status'], rec['error'] = 'error', r['error']
    rec.update(usage)
    return r


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
    """Runs cmd within `timeout` seconds: its status, seconds and output.
    Under the memory cap (`cap_memory`) it runs in a cgroup of its own,
    and a command the cap killed is an error `CAP_KILLED`. Its memory goes
    to the record under way (`LANE.usage`): `maxrss_kb`, the largest peak
    resident set of one of its processes (`os.wait4`), and under the cap
    `mem_peak_kb`, the peak of the whole tree as the cap counts it, the
    largest of the record's commands."""
    if STOPPED.is_set():
        raise SystemExit('stopped')
    cap = memcap.Cap(*MEMCAP) if MEMCAP else None
    if cap:
        cmd = cap.command(julia_heap_hint(list(cmd), cap.limit))
    cpu = getattr(LANE, 'cpu', None)
    if cpu is not None:
        cmd = ['taskset', '-c', str(cpu)] + list(cmd)
    t0 = time.monotonic()
    # ProbLog and aspmc launch compilers of their own. Killing only the
    # frontend leaves those compilers consuming CPU during later timings.
    try:
        with _Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                    text=True, cwd=cwd, env=env, start_new_session=(os.name == 'posix')) as p:
            with LIVE_LOCK:
                LIVE.add(p)
            if cap:
                cap.start()
            try:
                stdout, stderr = p.communicate(timeout=timeout)
            except subprocess.TimeoutExpired:
                kill(p)
                p.communicate()
                stdout = None
            except BaseException:
                kill(p)
                p.communicate()
                raise
            finally:
                with LIVE_LOCK:
                    LIVE.discard(p)
    finally:
        peak, killed = cap.finish() if cap else (None, False)
    dt = time.monotonic() - t0
    usage = getattr(LANE, 'usage', None)
    if usage is not None:
        if p.rusage is not None:
            usage['maxrss_kb'] = max(usage.get('maxrss_kb', 0), p.rusage.ru_maxrss)
        if cap:
            usage['mem_peak_kb'] = max(usage.get('mem_peak_kb', 0), peak >> 10)
            usage['mem_cap_kb'] = cap.limit >> 10
            usage['cap_killed'] = usage.get('cap_killed', False) or killed
    if STOPPED.is_set():
        raise SystemExit('stopped')
    if killed:
        return {'status': 'error', 'seconds': dt, 'returncode': p.returncode, 'error': cap_error(cap.limit >> 10)}
    if stdout is None:
        return {'status': 'timeout', 'seconds': timeout}
    if p.returncode != 0:
        err = stderr.strip() or stdout.strip() or f'exit {p.returncode}'
        # Rust's last line can be just the backtrace hint. Retain the cause,
        # including allocation failures, without unbounded trace records.
        if len(err) > 8192:
            err = err[:4096] + '\n... output truncated ...\n' + err[-4096:]
        r = {'status': 'error', 'seconds': dt, 'error': err, 'returncode': p.returncode}
        if p.returncode in STOP_SIGNALS and dt < timeout and not MEMOUT.search(err):
            r['status'] = 'stopped'     # a signal from outside, before the limit (STOP_SIGNALS)
        return r
    return {'status': 'ok', 'seconds': dt, 'stdout': stdout, 'stderr': stderr}


def queries(path):
    """The query atoms of a program, in order."""
    return [m.group(1) for m in re.finditer(r'^query\((.*)\)\.\s*(?:%.*)?$', open(path).read(), re.M)]


DICE_VALUE = re.compile(r'true|false|-?\d+|\(.*\)|\[.*\]')


def parse_dice(out):
    """Dice's table: one row per value of the returned expression, a tuple's
    value written `(true, false)`. Only rows whose first column is a value
    count: noDice prints its runtimes after the table (`Compilation
    0.569023`, `noDice (total) 0.569052`), which made a returned tuple's
    rows read as no tuple table (dice_marginals)."""
    rows = {}
    for line in out.splitlines():
        f = line.rsplit(None, 1)
        if len(f) == 2 and DICE_VALUE.fullmatch(v := re.sub(r'\s+', '', f[0])):
            try:
                rows[v] = float(f[1])
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
    program of a dice2problog.py instance and the program a generator writes
    beside its instance (`dice_source`), the programs of an Alea.jl
    program's circuit (`run_dice_circuit`), and for a network one generated
    program per queried variable (Dice returns one value; a tuple of them
    would print the joint; `gen_bn.dice_programs`), which observes the
    observations (noDice) or, for Dice, returns the variable beside their
    conjunction, the rows where it holds divided here by their sum (Dice 1.0
    prints zeros where the observations have probability under 1e-6; noDice
    prints six decimals, too few for such a joint), and for another program one program per query of its ground program
    (`ground_route`, run as a circuit's). The answers are keyed like the ProbLog program's
    queries: the row of value k is the k-th query of that variable."""
    family, gargs = instances(args)[name]
    joint = gargs[0] == 'gen_bn.py' and tool == 'dice' and not getattr(args, 'pr', False)
    if getattr(args, 'pr', False):   # P(e), of a network alone, asked as `path`'s one query
        source = dice_source(args, name, tool)
        if source is None:
            return {'status': 'error', 'seconds': 0, 'error': f"--pr: no program of P(e) for family {family}"}
        programs = [(source, queries(path), None)]
    elif gargs[0] == 'dice2problog.py' or writes_dice(gargs):
        programs = [(dice_source(args, name, tool), queries(path), None)]
    elif gargs[0] == 'alea_bench.jl':
        return run_dice_circuit(args, name, os.path.join(args.dir, name + '.dicemap'), tool)
    elif gargs[0] == 'gen_bn.py':
        try:
            # for Dice, each returns its variable and whether the observations hold
            per_variable = gen_bn.dice_programs(gargs[1:], joint=joint)
        except SystemExit as x:
            return {'status': 'error', 'seconds': 0, 'error': f'gen_bn.py --format dice: {x}'}
        # each written just before its run and removed after it (scratch)
        programs = [(os.path.join(args.dir, f'{name}__{v}.dice'), [q for q in queries(path) if q.split('(')[0] == v],
                     text) for v, text in per_variable]
    elif ground_dice(args, name):
        return run_dice_circuit(args, name, GROUNDED[(name, 'dicemap')]['stem'] + '.dicemap', tool)
    else:
        return {'status': 'error', 'seconds': 0, 'error': f'no Dice program for family {family}'}
    total, answers, error = 0.0, {}, None
    for program, qs, text in programs:
        program = os.path.abspath(program)
        cmd = getattr(args, tool).format(file=program, dir=os.path.dirname(program),
                                         base=os.path.basename(program)).split()
        with scratch(*([program] if text is not None else [])):
            if text is not None:
                write_new(program, text)
            r = timed(cmd, max(0.0, args.timeout - total))
        total += r['seconds']
        if r['status'] != 'ok':
            r['seconds'] = total
            return r
        rows = parse_dice(r['stdout'])
        if joint:
            # (value k, observations held): P(k | e) is its row with true over the sum of those
            held = {v[1:-len(',true)')]: p for v, p in rows.items() if v.startswith('(') and v.endswith(',true)')}
            pe = math.fsum(held.values())
            if not pe > 0:
                # the instance's error, after its other programs (each is
                # captured where every run's table is empty)
                error = error or f'{tool}: the observations have probability {pe} in {os.path.basename(program)}'
                continue
            for k, q in enumerate(qs):
                answers[re.sub(r'\s+', '', q)] = held.get(str(k), 0.0) / pe
            continue
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
    if error:
        return {'status': 'error', 'seconds': total, 'error': error}
    return {'status': 'ok', 'seconds': total, 'answers': answers}


def run_dice_circuit(args, name, dicemap, tool='dice'):
    """Dice (or noDice, `tool`) on the programs alea_bench.jl writes for the
    circuit of an Alea.jl program, NAME__dJ.dice, or ground2net.py for the
    queries of a ground program, listed in `dicemap` beside them: a line `FILE
    bool ATOM` answers ATOM with the probability of true, a line `FILE
    joint PRED N` answers PRED(U) for U in 0 .. 2^N - 1 with the row of the
    returned N Booleans (nested pairs, the most significant first) whose
    value is U. The seconds are the sum of the programs'."""
    total, answers = 0.0, {}
    for line in open(dicemap):
        file, kind, atom, *rest = line.split()
        program = os.path.abspath(os.path.join(os.path.dirname(dicemap), file))
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


def dice_source(args, name, system=None):
    """The Dice program behind a dice2problog.py instance (its source, or
    with its arguments in its places NAME.dice), an instance whose generator
    writes one (writes_dice, beside it) or a network (the
    network and its observations, a Markov network's factors among them,
    with its queried variables returned as a tuple, gen_bn.py --format
    dice; not a program that asks nothing), as a path; for another program
    but a circuit the one ground2net.py writes of its ground program, which
    returns its queries as a tuple (`ground_route`, run before the system);
    None for a circuit and a network that asks nothing. Under --pr, the
    program of a network's P(e), which returns the conjunction of its
    observations (`gen_bn.dice_programs`), and None for another instance.
    Of an instance with a port (dice_ported), the port for a `system` that
    is not one of DICE_OWN."""
    _, gargs = instances(args)[name]
    if getattr(args, 'pr', False):
        if gargs[0] != 'gen_bn.py':
            return None
        out = os.path.join(args.dir, f'{name}.pe.dice')
        if out not in DICE_WRITTEN:
            write_new(out, gen_bn.dice_programs(gargs[1:], pe=True)[0][1])
            DICE_WRITTEN.add(out)
        return out
    if gargs[0] == 'dice2problog.py' and len(gargs) == 2:
        return gargs[1]
    if dice_ported(gargs) and system not in DICE_OWN:
        return os.path.join(args.dir, name + '.port.dice')
    if gargs[0] == 'dice2problog.py' or writes_dice(gargs):   # with its arguments in its places, or beside it
        return os.path.join(args.dir, name + '.dice')
    if gargs[0] == 'gen_bn.py' and '--query none' not in ' '.join(gargs):
        out = os.path.join(args.dir, f'{name}.dice')
        if out not in DICE_WRITTEN:   # once a run, though every system asks (an instance's runs share a lane)
            write_new(out, subprocess.run([sys.executable, str(source_tree.script('gen_bn.py')), *gargs[1:], '--format', 'dice'],
                                          capture_output=True, text=True, check=True).stdout)
            DICE_WRITTEN.add(out)
        return out
    if ground_dice(args, name):
        # problog2ppl.py's program, or ground2net.py's where it refused (dice_route); ground2net.py's name
        # before the route has run
        stem = GROUNDED.get((name, 'dice'), {}).get('stem') or os.path.join(ground_dir(args, name), f'{name}.dice')
        return stem + '.dice'
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


# The roulette suite's families that Alea.jl reads in its own language:
# Alea.jl's own diamond, and the ladder's port, Alea.jl having none (data/alea/README.md,
# verification), each run by alea_bench.jl with its size as N.
ALEA_PROGRAMS = {f: os.path.join(ROOT, 'data', 'alea', 'verification', f'{f}.jl') for f in ('diamond', 'ladder')}


def alea_program(args, gargs):
    """The Alea.jl program of an instance of ALEA_PROGRAMS's families, or of
    the ppl suite's arith and pluck programs (ALEA_OF), and its setting, as
    alea_command takes them; None for another instance."""
    if args.suite == 'roulette' and gargs[0] == 'gen_roulette.py' and gargs[1] in ALEA_PROGRAMS:
        return [ALEA_PROGRAMS[gargs[1]], f'N={gargs[2]}']
    if args.suite == 'ppl':
        return ALEA_OF.get(tuple(gargs))
    return None


ALEA_ATOM = re.compile(r'q(\d+)(?:\((\d+)\))?')


def alea_program_answers(path, answers, values=None):
    """The answers to the queries of the instance's program at `path` from
    those of its Alea.jl program (alea_program), named as Alea.jl names the
    components of its value and as the circuit Alea.jl exports asks them,
    in order: a Boolean's `qK` answers one query; a number's worlds of its
    bits that are not constant, `qK(U)`, answer the queries that follow
    with one predicate, PRED(0) ... PRED(N - 1) for the number's N values,
    PRED(I) the mass of the worlds whose value (`values`, alea_bench.jl's
    `alea_value qK(U) V`) is I modulo N, so that a signed value is its two's
    complement, as the ports' integers number their values. ValueError
    where they do not line up (alea_answers)."""
    values = values or {}
    qs = [re.sub(r'\s+', '', q) for q in queries(path)]
    bools, worlds = {}, {}
    for atom, p in answers.items():
        m = ALEA_ATOM.fullmatch(atom)
        if m is None:
            raise ValueError(f'{atom} names no component')
        if m.group(2) is None:
            bools[int(m.group(1))] = p
        elif atom in values:
            worlds.setdefault(int(m.group(1)), []).append((values[atom], p))
        elif p == 0:   # a world `run` found no mass on, whose value it does not print
            worlds.setdefault(int(m.group(1)), [])
        else:
            raise ValueError(f'no value of {atom}')
    comps, i = [], 0
    for k in range(1, len(set(bools) | set(worlds)) + 1):
        if k in bools:
            comps.append({'true': bools[k]})
            i += 1
            continue
        if k not in worlds:
            raise ValueError(f'no component q{k}')
        pred, n = qs[i].split('(')[0] if i < len(qs) else None, 0
        while i < len(qs) and '(' in qs[i] and qs[i].split('(')[0] == pred:
            i, n = i + 1, n + 1
        if n == 0:
            raise ValueError(f'component q{k} is a number, and the queries that follow ask none')
        dist = {str(v): 0.0 for v in range(n)}
        for v, p in worlds[k]:
            dist[str(v % n)] += p
        comps.append(dist)
    return alea_answers(qs, comps)


def run_alea(args, name, path):
    """Alea.jl on the Dice program behind a `dice`, `nodice`, roulette, geni or `bn` instance
    (`dice_source`), translated by dice2alea.py, or on the Alea.jl program
    of an instance written from one (`run_alea_program`): a network as one
    program whose value is the tuple of its queried variables, all answered
    by one `pr`. On the families of ALEA_PROGRAMS and ALEA_OF, the Alea.jl
    program there, its components' answers named as the instance's queries,
    in order (alea_program_answers). The translation is not timed, but bounded by
    translation_bound (a harness time-out past it, so that a lane cannot
    stall); the seconds are the Julia process's, and `inproc_seconds` what
    it reports for building the program and `pr`
    (`jit_seconds` of it Julia's compilation)."""
    family, gargs = instances(args)[name]
    if gargs[0] == 'alea_bench.jl':
        return run_alea_program(args, gargs[1:])
    own = alea_program(args, gargs)
    if own is not None:
        r = run_alea_program(args, own)
        if r['status'] != 'ok':
            return r
        try:
            return dict(r, answers=alea_program_answers(path, r['answers'], r['values']))
        except ValueError as x:
            return {'status': 'error', 'seconds': r['seconds'], 'error': f'alea: {x}'}
    source = dice_source(args, name, 'alea')
    if source is None:
        return {'status': 'error', 'seconds': 0, 'error': f'no Dice program for family {family}'}
    jl = os.path.abspath(os.path.join(args.dir, f'{name}.alea.jl'))
    out, err = translation(args, 'dice2alea.py', [source])
    if err:
        return err
    write_new(jl, out)
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


def alea_command(args, mode, program, prefix=None, warm_below=None, template=None):
    """The command that runs alea_bench.jl under the `--alea` template (or
    `template`, --alea-tddlog's) on `program`, an instance's generator
    arguments after `alea_bench.jl`: the program's path, then the
    NAME=VALUE it is run with (`--set`) and the (NAME, PATH) of the files
    it reads (`--file`, a logic program's facts)."""
    cmd = (template or args.alea).format(file=ALEA_BENCH, dir=HERE, base=os.path.basename(ALEA_BENCH)).split() + [mode]
    if warm_below is not None:
        cmd += ['--warm-below', f'{warm_below:g}']
    for setting in program[1:]:
        cmd += ['--file', *setting] if isinstance(setting, tuple) else ['--set', setting]
    return cmd + [program[0]] + ([prefix] if prefix else [])


def circuit_size(path):
    """The flips and gates of the circuit alea_bench.jl wrote as the
    ProbLog program `path`, from its header (`% N flips, M gates, ...`);
    nothing where it wrote none (the run's record says so)."""
    if not os.path.exists(path):
        return {}
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
    answers, values, stats, printed = {}, {}, {}, []
    keys = {'alea_seconds': 'inproc_seconds', 'alea_compile_seconds': 'jit_seconds',
            'alea_warm_seconds': 'warm_seconds'}
    sizes = {'alea_bdd_evidence': 'bdd_evidence', 'alea_bdd_query_max': 'bdd_query_max',
             'alea_bdd_joint_max': 'bdd_joint_max', 'alea_bdd_peak': 'bdd_peak'}
    for line in r['stdout'].splitlines():
        f = line.split()
        if len(f) == 3 and f[0] == 'alea_q':
            answers[f[1]] = float(f[2])
            continue
        if len(f) == 3 and f[0] == 'alea_value':
            values[f[1]] = int(f[2])
        if len(f) == 2 and f[0] in keys:
            stats[keys[f[0]]] = round(float(f[1]), 3)
        elif len(f) == 2 and f[0] in sizes:
            stats[sizes[f[0]]] = int(f[1])
        if len(printed) < 100:
            printed.append(line)
    # What Alea prints besides the answers (expectations, values of worlds,
    # times, sizes), the first 100 lines.
    stats['printed'] = printed
    return {'status': 'ok', 'seconds': r['seconds'], 'answers': answers, 'values': values, 'stats': stats}


ROULETTE_DATA = os.path.join(ROOT, 'data', 'roulette')
# roulette_timer.rkt, which runs a Roulette program and reports its run's
# seconds once Racket and the modules it imports are loaded
# (`roulette_timed`), and its line.
ROULETTE_TIMER = os.path.join(HERE, 'roulette_timer.rkt')
ROULETTE_TIMED = re.compile(r'^roulette_timer: program ([0-9.eE+-]+) s$', re.M)


BIF_WRITTEN = {}


def rktd_lists(text):
    """The data of a file of Racket lists of symbols (networks.rktd), its
    `;` comments dropped: a list as a Python list, a symbol as a string."""
    out = [[]]
    for tok in re.findall(r'[()]|[^\s()]+', re.sub(r';[^\n]*', '', text)):
        if tok == '(':
            out.append([])
        elif tok == ')':
            if len(out) == 1:
                raise ValueError('a ) without its (')
            done = out.pop()
            out[-1].append(done)
        else:
            out[-1].append(tok)
    if len(out) != 1:
        raise ValueError('a ( without its )')
    return out[0]


def roulette_bif(args, name):
    """A Bayesian network as Roulette's own BN language reads it
    (roulette/example/bn, data/roulette/bn.rkt): (the BIF file, the file of
    what bn.rkt asks of it, {its word for a queried variable: (the
    variable's name in the ProbLog program, its values in the BIF file,
    their names in the ProbLog program)}), the asks written in the
    instance's directory once. A network of --suite roulette is the
    artifact's own BIF file, asked its queried variable in the artifact's
    variable order (benchmark/main.rkt, networks.rktd); another network of
    gen_bn.py is the BIF file gen_bn.py writes of it, written there too, with
    its observations, queries and order (`gen_bn.bif_programs`; under --pr,
    P(e)). None for another instance and a Markov network, whose factors
    BIF does not hold (it runs the Dice program dice2roulette.py
    translates)."""
    asks = os.path.abspath(os.path.join(args.dir, f'{name}.roulette.asks'))
    if asks not in BIF_WRITTEN:
        family, gargs = instances(args)[name]
        made = None
        if args.suite == 'roulette' and family == 'bn':
            net, query = gargs[1], gargs[gargs.index('--query') + 1]
            orders = {e[0]: e[2] for e in rktd_lists(open(os.path.join(ROULETTE_DATA, 'networks.rktd')).read())[0]}
            values = gen_bn.parse(net)[1][query]
            write_new(asks, f"order {' '.join(orders[name.removeprefix('rl_bn_')])}\nquery {query}\n")
            made = (os.path.abspath(net), asks,
                    {query: (gen_bn.ident(query), values, [gen_bn.ident(u) for u in values])})
        elif args.suite not in ('roulette', 'circulette') and gargs[0] == 'gen_bn.py':
            written = gen_bn.bif_programs(gargs[1:], pe=getattr(args, 'pr', False))
            if written is not None:
                text, lines, names = written
                bif = os.path.abspath(os.path.join(args.dir, f'{name}.roulette.bif'))
                write_new(bif, text)
                write_new(asks, lines)
                made = (bif, asks, names)
        BIF_WRITTEN[asks] = made
    return BIF_WRITTEN[asks]


def roulette_network_answers(names, values):
    """The answers keyed as the ProbLog program's queries, from the
    probability of each value of each queried variable that Roulette's BN
    language gives ({word: {VALUE: probability}}, VALUE as Racket's `write`
    writes it; `roulette_bif`'s names)."""
    printed = {}
    for word, dist in values.items():
        var, bif_values, atoms = names[word]
        for v, p in dist.items():
            u = v.strip('|')   # a symbol that reads as a number, between bars
            if u not in bif_values:
                raise ValueError(f'{word} takes {v}, which the BIF file does not declare')
            printed[f'{var}({atoms[bif_values.index(u)]})'] = p
    return printed


def roulette_program(args, name, checkout=None):
    """The Racket program and arguments Roulette (neuppl/roulette's main,
    docs/baselines.md section 11) runs an instance with, under the variant
    rsdd. A Bayesian network, data/roulette/bn.rkt on its BIF file and what
    it asks (`roulette_bif`), which prints its distributions (`roulette_b
    VARIABLE VALUE P`, under --pr `roulette_e P`). On --suite roulette, the
    artifact's program as data/roulette's driver runs it and prints its
    answers in full (`roulette_q ATOM P VALUE`); on the other suites, the
    instance's Dice program translated by dice2roulette.py
    (`roulette_translate`), which prints each component's distribution
    (`roulette K VALUE P`). On --suite circulette, circulette.rkt on the
    instance's netlist (`run_circulette`), of the Circulette `checkout`
    (by default --circulette-benchmarks, which Roulette's Racket compiles;
    Roulette+tddlog's is --circulette-tddlog)."""
    if args.suite == 'circulette':
        return (os.path.join(checkout or args.circulette_benchmarks, 'circulette.rkt'),
                [original(instances(args)[name][1], '', os.path.join(args.dir, name))])
    bif = roulette_bif(args, name)
    if bif is not None:
        return os.path.join(ROULETTE_DATA, 'bn.rkt'), [bif[0], bif[1]]
    if args.suite != 'roulette':
        return os.path.abspath(os.path.join(args.dir, f'{name}.roulette.rkt')), []
    family, gargs = instances(args)[name]
    program, size = gargs[1], gargs[2]
    if program.startswith('hmm-'):
        return os.path.join(ROULETTE_DATA, 'hmm.rkt'), [program[4:], size]
    return os.path.join(ROULETTE_DATA, program + '.rkt'), [size]


def roulette_command(template, prog, *words):
    """A Racket command template (`--roulette`'s, `--roulette-tddlog`'s) on
    program `prog`; with words, Racket runs them on it instead (`-l- raco
    make`, roulette_timer.rkt, `-l- roulette/engine/tddlog`)."""
    file = ' '.join(words + (prog,))
    return template.format(file=file, dir=os.path.dirname(prog), base=os.path.basename(prog)).split()


# A Roulette module that `raco make` did not compile within the time limit;
# records made before 2026-11-05 have `raco make: timeout ` as an error.
RACO_MAKE_LIMIT = 'raco make: the module did not compile within the time limit of {limit:g} s'


def roulette_make(args, template, prog, *more):
    """`raco make` of program `prog` and the files `more` under a Racket
    command template (`roulette_command`), untimed: an error record where
    it fails, else None. Bounded by the time limit, past which the record
    is `unsupported` (RACO_MAKE_LIMIT), at 0 s as every compile is: a
    program whose module does not compile within the limit is one Roulette
    cannot express. One compile at a time for each template, under a
    lock in the temporary directory, which a run's lanes share (the
    runner's TMPDIR). Without the lock, lanes that start together race:
    `raco make` without `-j` takes no lock of its own, and they compile the
    same modules into the same compiled root at once (the program's, and
    under Roulette+tddlog's PLTCOMPILEDROOTS the installation's modules).
    One of them then deletes a compiled file that another has just found,
    and the other fails at 0 s ("cannot open module file", six records of
    the ace table)."""
    key = hashlib.sha256(template.encode()).hexdigest()[:16]
    with open(os.path.join(tempfile.gettempdir(), f'raco-make-{key}.lock'), 'w') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        c = timed(roulette_command(template, prog, '-l-', 'raco', 'make') + list(more), args.timeout)
    if c['status'] == 'timeout':
        return {'status': 'unsupported', 'seconds': 0, 'error': RACO_MAKE_LIMIT.format(limit=args.timeout)}
    if c['status'] != 'ok':
        return {'status': 'error', 'seconds': 0, 'error': f"raco make: {c['status']} {c.get('error', '')[-2000:]}"}
    return None


def roulette_timed(args, prog, rest, budget, cwd=None):
    """Roulette's run of program `prog` on arguments `rest` within `budget`
    seconds, under roulette_timer.rkt, the program and the timer compiled
    by `raco make` before, untimed. Its record holds the timer's seconds,
    the program's run once Racket and the modules it imports are loaded,
    as `inproc_seconds` (own_seconds): the span Roulette's tddlog engine's
    `program S s` reports, which Roulette+tddlog pays.
    An error record where the compile fails (`roulette_make`) or the timer
    prints no line."""
    err = roulette_make(args, args.roulette, prog, ROULETTE_TIMER)
    if err:
        return err
    r = timed(roulette_command(args.roulette, ROULETTE_TIMER) + [prog] + rest, budget, cwd=cwd)
    if r['status'] != 'ok':
        return r
    t = ROULETTE_TIMED.search(r.get('stderr', ''))
    if t is None:
        tail = (r.get('stderr', '').strip() or r['stdout'].strip()).splitlines()
        return {'status': 'error', 'seconds': r['seconds'],
                'error': 'no roulette_timer line: ' + (tail[-1] if tail else 'empty output')}
    return {**r, 'inproc_seconds': round(float(t.group(1)), 3)}


def roulette_translate(args, source, rkt, joint=False):
    """The Dice program `source` translated by dice2roulette.py (with
    `--joint` where `joint`) into rkt, untimed and bounded by
    translation_bound: an error record where it fails, else None."""
    out, err = translation(args, 'dice2roulette.py', [source] + (['--joint'] if joint else []))
    if err:
        return err
    write_new(rkt, out)
    return None


def roulette_masses(args, label, source, joint, budget):
    """Roulette on one of a circuit's Dice programs, as `run_translated`
    asks it: the distributions of its value's components, and its seconds
    and `inproc_seconds`."""
    rkt = os.path.abspath(os.path.join(args.dir, label.removesuffix('.dice') + '.roulette.rkt'))
    with scratch(rkt):
        err = roulette_translate(args, source, rkt, joint)
        if err:
            return None, err
        r = roulette_timed(args, rkt, [], budget)
    if r['status'] != 'ok':
        return None, r
    return parse_alea(r['stdout'], 'roulette')[0], \
        {'seconds': r['seconds'], 'stats': {'inproc_seconds': r['inproc_seconds']}}


def roulette_source(args, name, prog):
    """The program Roulette runs on an instance written where it is a
    translation: the instance's Dice program by dice2roulette.py into
    `prog`, outside --suite roulette, --suite circulette (circulette.rkt on
    the netlist) and the Bayesian networks (`roulette_bif`). An error record
    where there is none or it fails, else None."""
    if args.suite in ('roulette', 'circulette') or roulette_bif(args, name) is not None:
        return None
    source = dice_source(args, name, 'roulette')
    if source is None:
        return {'status': 'error', 'seconds': 0, 'error': f'no Dice program for {name}'}
    return roulette_translate(args, source, prog)


def run_roulette(args, name, path, variant):
    """Roulette (`--roulette`, a command template for Racket with
    neuppl/roulette's packages roulette-lib and roulette installed, {file}
    the program) on an instance (`roulette_program`), timed by
    roulette_timer.rkt (`roulette_timed`). The answers are keyed as the
    ProbLog program's queries (under --pr, P(e) as PE_QUERY's); Roulette
    leaves out the values of probability 0, which answer 0. Values that
    answer no query (network.rkt's #t, at a probability of 0 up to
    rounding) are kept under `other`. An Alea.jl program's circuit is run as
    its Dice programs, one at a time (`run_translated`)."""
    _, gargs = instances(args)[name]
    if args.suite == 'circulette':
        return run_circulette(args, name, path, variant)
    if args.suite != 'roulette' and gargs[0] == 'alea_bench.jl':
        return run_translated(args, name, path, 'roulette', roulette_masses)
    prog, rest = roulette_program(args, name)
    err = roulette_source(args, name, prog)
    if err:
        return err
    r = roulette_timed(args, prog, rest, args.timeout)
    if r['status'] != 'ok':
        return r
    bif = roulette_bif(args, name)
    printed, values, stats = {}, {}, {'inproc_seconds': r['inproc_seconds']}
    for line in r['stdout'].splitlines():
        f = line.split()
        if len(f) == 4 and f[0] == 'roulette_b':
            values.setdefault(f[1], {})[f[2]] = float(f[3])
        elif len(f) == 2 and f[0] == 'roulette_e':
            printed[PE_QUERY] = float(f[1])
        elif len(f) >= 3 and f[0] == 'roulette_q':
            printed[f[1]] = float(f[2])
    try:
        if bif is not None:
            printed.update(roulette_network_answers(bif[2], values))
        elif args.suite != 'roulette':
            printed = alea_answers(queries(path), parse_alea(r['stdout'], 'roulette')[0])
    except ValueError as x:
        return {'status': 'error', 'seconds': r['seconds'], 'error': f'roulette: {x}'}
    return roulette_record(r, path, printed, stats)


def roulette_record(r, path, printed, stats):
    """The record of a run `r` that printed the answers `printed`, keyed as
    the instance's queries, those it does not print 0, those it prints that
    no query asks kept under `other`; an error where it printed none."""
    qs = [re.sub(r'\s+', '', q) for q in queries(path)]
    if not printed:
        tail = (r.get('stderr', '').strip() or r['stdout'].strip()).splitlines()
        return {'status': 'error', 'seconds': r['seconds'], 'error': 'no answers: ' + (tail[-1] if tail else 'empty output')}
    other = {a: p for a, p in printed.items() if a not in qs}
    if other:
        stats['other'] = other
    return {'status': 'ok', 'seconds': r['seconds'], 'answers': {q: printed.get(q, 0.0) for q in qs}, 'stats': stats}


# Roulette+tddlog: Roulette's own run of a program up to the circuit it
# builds, which Roulette's tddlog engine writes as a ProbLog program, and
# tddlog on it. The engine's command line (`racket -l- roulette/engine/tddlog
# PROGRAM.rkt ARG ...`, in the branch tddlog of Tractables/roulette), its
# summary line, and its notes of the calls of `infer` and of each value asked.
RT_ENGINE = ('-l-', 'roulette/engine/tddlog')
RT_SUMMARY = re.compile(r'^tddlog-engine: \d+ queries, .*; program ([0-9.eE+-]+) s, circuit ([0-9.eE+-]+) s$', re.M)
RT_VALUE = re.compile(r'^% q(\d+)\((\d+)\): (.*)$', re.M)
# The value of Roulette's query where its observations fail (a gensym).
RT_FAILED = re.compile(r'^⊥\d*$')


def roulette_tddlog_accepts(args, name):
    """Roulette+tddlog's instances: those of --suite roulette and --suite
    circulette (circulette.rkt on the netlist), the Bayesian networks
    Roulette reads in its BN language (`roulette_bif`), and the Dice
    programs Roulette runs through dice2roulette.py: the Dice paper's
    (data/dice, --suite standard and indicators), noDice's (data/nodice)
    and the arith programs' Dice ports (data/alea/arith; --suite ppl), and
    Geni's (gen_geni.py's, --suite geni); and the logic programs Roulette
    reads as problog2ppl.py writes them (lp_route). Not a circuit's Dice
    programs nor a Markov network's, whose Roulette+tddlog would be
    Alea.jl's or the network's own reader with more steps between."""
    if args.suite in ('roulette', 'circulette'):
        return True
    gargs = instances(args)[name][1]
    if gargs[0] in ('dice2problog.py', 'gen_geni.py') or lp_route(gargs, 'roulette-tddlog', getattr(args, 'pr', False)):
        return True
    if gargs[0] != 'gen_bn.py':
        return False
    key = tuple(gargs)
    if key not in BAYESIAN:
        BAYESIAN[key] = gen_bn.is_bayesian(gargs[1:])
    return BAYESIAN[key]


BAYESIAN = {}   # gen_bn.py's arguments: whether they read a Bayesian network (roulette_tddlog_accepts)


def roulette_tddlog_calls(text, answers):
    """The distribution of each call of `infer` that a Roulette+tddlog
    circuit `text` asks values of ({K: {VALUE: probability}}, VALUE as the
    circuit's note writes it), from tddlog's answers to its queries qK(I),
    divided as Roulette's `query` divides them: by the sum of the values
    other than the failed observations' (⊥), which are left out; all 0
    where that sum is 0."""
    calls = {}
    for k, i, value in RT_VALUE.findall(text):
        q = f'q{k}({i})'
        if q not in answers:
            raise ValueError(f'tddlog answers no {q}')
        calls.setdefault(int(k), {})[value] = answers[q]
    for k, dist in calls.items():
        total = sum(p for v, p in dist.items() if not RT_FAILED.match(v))
        calls[k] = {v: (p / total if total > 0 else 0.0) for v, p in dist.items() if not RT_FAILED.match(v)}
    return calls


def roulette_tddlog_components(calls):
    """The distributions of a translated Dice program's components, as
    parse_alea reads them from its run, from those of the calls of `infer`
    (roulette_tddlog_calls): the K-th component is the (2K-1)-th call (its
    query's value; the 2K-th is its observations'), a Boolean's values true
    and false, an integer's (a list of Booleans, least significant first)
    every value 0 to 2^W - 1."""
    comps = []
    for k in range(1, max(calls, default=0) + 1, 2):
        dist = calls.get(k, {})
        if set(dist) <= {'#t', '#f'}:
            comps.append({'true': dist.get('#t', 0.0), 'false': dist.get('#f', 0.0)})
            continue
        bits = {v: v[1:-1].split() for v in dist}
        widths = {len(b) for b in bits.values()}
        if len(widths) != 1 or any(x not in ('#t', '#f') for b in bits.values() for x in b):
            raise ValueError(f'call {k} takes {sorted(dist)}, neither Booleans nor lists of one width')
        w = widths.pop()
        comp = {str(u): 0.0 for u in range(2 ** w)}
        for v, b in bits.items():
            comp[str(sum(2 ** j for j, x in enumerate(b) if x == '#t'))] = dist[v]
        comps.append(comp)
    return comps


def roulette_tddlog_atom(family, atom, value, dist):
    """The atom a driver prints a value under, as it would print it given
    the distribution `dist` that tddlog answers rather than the exporter's,
    which holds every value the circuit can take, those of probability 0
    too. hardware-bitwidth.rkt labels #f (out of fuel) one past the largest
    total it is given, and Roulette leaves out the totals of probability 0:
    the label is one past the largest total of positive probability."""
    if family == 'rl_hardware_bitwidth' and value == '#f':
        return f'res({max((int(v) for v, p in dist.items() if v.isdigit() and p > 0), default=-1) + 1})'
    return atom


def tddlog_on_circuit(args, pl, limit, pe=False):
    """tddlog (--tddlog's binary, strategy auto) on the ProbLog program
    `pl` that an integration's exporter wrote (Roulette's tddlog engine,
    Alea.jl's tddlog backend), within `limit` seconds, at least one; with
    `pe`, the probability of its evidence (a Circulette netlist's)."""
    return timed([args.tddlog, '--stats', '--precision', '17'] + tddlog_flags('auto')
                 + (['--evidence-probability'] if pe else []) + [pl], max(1.0, limit))


def run_roulette_tddlog(args, name, path, lp=None):
    """Roulette+tddlog on an instance: Roulette's program for it
    (`roulette_program`, variant rsdd; under `lp`, from run_lp, a logic
    program's module as problog2ppl.py writes it, its arguments and the
    queries it answers, in order) run on Roulette's tddlog engine under
    --roulette-tddlog's Racket (the fork's), which writes the circuit
    Roulette builds as a ProbLog program, and tddlog (--tddlog's binary,
    strategy auto) on that program. The program is compiled before (raco
    make), untimed. The time charged, kept as `inproc_seconds`
    (own_seconds), is the engine's `program S s` (Roulette's run of the
    program, as roulette_timer.rkt times Roulette's own), its `circuit S s`
    (writing the program) and tddlog's
    run; the record's seconds are the two processes' wall times. tddlog's
    answers are read as Roulette's query reads its engine's
    (roulette_tddlog_calls): a driver's single query's values as the atoms
    it prints them under (`roulette_q ATOM P VALUE`), a network's queries
    as run_roulette keys them (bn.rkt's K-th query, or P(e), is the
    (2K-1)-th call, the 2K-th its observations', and a circuit of another
    number of calls is an error), a translated Dice
    program's components as run_roulette reads them
    (roulette_tddlog_components). On --suite circulette, circulette.rkt of
    --circulette-tddlog on the netlist, run where the netlist is as
    run_circulette runs it, and tddlog with --evidence-probability: the
    posteriors keyed as the queries of the program tddlog's reader writes of
    the netlist (`circulette_circuit_answers`), P(fail) kept as `p_fail`."""
    family, _ = instances(args)[name]
    ct = args.suite == 'circulette'
    if lp is None:
        prog, rest = roulette_program(args, name, args.circulette_tddlog if ct else None)
        err = roulette_source(args, name, prog)
        if err:
            return err
    else:
        prog, rest = lp[0], lp[1]
    racket = args.roulette_tddlog
    err = roulette_make(args, racket, prog)
    if err:
        return err
    x = timed(roulette_command(racket, prog, *RT_ENGINE) + rest, args.timeout,
              **({'cwd': os.path.dirname(rest[0])} if ct else {}))
    if x['status'] != 'ok':
        return x
    m = RT_SUMMARY.search(x.get('stderr', ''))
    if m is None or not x['stdout']:
        tail = x.get('stderr', '').strip().splitlines()
        return {'status': 'error', 'seconds': x['seconds'],
                'error': "Roulette's tddlog engine wrote no program: " + (tail[-1] if tail else 'empty output')}
    program_s, circuit_s = float(m.group(1)), float(m.group(2))
    pl = os.path.abspath(os.path.join(args.dir, f'{name}.roulette.pl'))
    write_new(pl, x['stdout'])
    stats = {'program_s': round(program_s, 3), 'circuit_s': round(circuit_s, 3),
             'circuit_bytes': len(x['stdout'].encode())}
    t = tddlog_on_circuit(args, pl, args.timeout - program_s - circuit_s, pe=ct)
    os.remove(pl)   # the circuit is the program's, which the run keeps
    seconds = x['seconds'] + t['seconds']
    if t['status'] != 'ok':
        return {**t, 'seconds': seconds}
    charge = program_s + circuit_s + t['seconds']
    stats.update(tddlog_s=round(t['seconds'], 3), inproc_seconds=round(charge, 3))
    if charge > args.timeout:
        return {'status': 'timeout', 'seconds': seconds, 'stats': stats}
    answers = parse_answers(t['stdout'])
    if ct:
        try:
            printed, stats['p_fail'] = circulette_circuit_answers(
                x['stdout'], answers, parse_evidence_probability(t['stdout']), circulette_gates(path))
        except (OSError, ValueError) as e:
            return {'status': 'error', 'seconds': seconds, 'error': f'roulette-tddlog: {e}'}
        return {'status': 'ok', 'seconds': seconds, 'answers': printed, 'stats': stats}
    try:
        if answers == 'inconsistent':
            raise ValueError("the circuit's evidence is inconsistent")
        calls = roulette_tddlog_calls(x['stdout'], answers)
        bif = roulette_bif(args, name)
        if bif is not None:
            asked = 1 if getattr(args, 'pr', False) else len(bif[2])
            n = CT_CALLS.search(x['stdout'])
            if n is None or int(n.group(1)) != 2 * asked:
                raise ValueError(f"bn.rkt called infer {n.group(1) if n else 'an unnoted number of'} times, "
                                 f"not twice for each of its {asked} queries")
        if lp is not None:
            printed = lp_named(lp[2], roulette_tddlog_components(calls))
        elif bif is not None and getattr(args, 'pr', False):
            printed = {PE_QUERY: calls.get(1, {}).get('#t', 0.0)}
        elif bif is not None:
            printed = roulette_network_answers(bif[2], {w: calls.get(2 * k + 1, {}) for k, w in enumerate(bif[2])})
        elif args.suite != 'roulette':
            printed = alea_answers(queries(path), roulette_tddlog_components(calls))
        else:
            # the atoms the driver prints, each with its value (print.rkt's emit)
            dist, printed = calls.get(1, {}), {}
            for line in x['stderr'].splitlines():
                f = line.split(maxsplit=3)
                if len(f) == 4 and f[0] == 'roulette_q':
                    a, p = roulette_tddlog_atom(family, f[1], f[3], dist), dist.get(f[3], 0.0)
                    if p > 0 or a not in printed:    # a value of probability 0, which Roulette leaves out
                        printed[a] = p
    except ValueError as e:
        return {'status': 'error', 'seconds': seconds, 'error': f'roulette-tddlog: {e}'}
    if lp is not None:   # keyed as problog2ppl.py lists the queries, every one answered (lp_named)
        return {'status': 'ok', 'seconds': seconds, 'answers': printed, 'stats': stats}
    return roulette_record({**x, 'seconds': seconds}, path, printed, stats)


# Circulette's report (circulette.rkt's diagnostic-query): P(fail), then the
# posterior given the failure of each single, pair and triple of faults
# whose joint weight is above 0, the pairs' above 2e-4 and the triples'
# above 1e-8 (CT_PRINTED_ABOVE, by the number of faults; printed_answers).
# Circulette numbers the faults 0 to n - 1 in the order of the netlist's
# live logic gates, the order of the gate_* facts of the program tddlog's
# reader writes of it (NAME.pl, `tddlog --program NAME.cql`;
# circulette_gates). Every system's answers are keyed as that program's
# queries, fault(G), faults(A,B) and faults(A,B,C) for gate ids A < B < C
# (circulette_atom): tddlog's and the ProbLog family's as they print them,
# Roulette's and Roulette+tddlog's through the gates' order.
CT_PRINTED_ABOVE = {1: 0.0, 2: 2e-4, 3: 1e-8}
CT_FAIL = re.compile(r'^Diagnosis: \(Circuit-Fault: #t\) \| Prob: (\S+)$', re.M)
CT_FAULTS = re.compile(r'^Gates? (\d+)(?: & (\d+))?(?: & (\d+))? Fault \| Posterior P\([^)]*\): (\S+)$', re.M)
CT_GATE = re.compile(r'^gate_(?:and|or|xor|not)\((\d+),', re.M)
CT_QUERY = re.compile(r'fault\(\d+\)|faults\(\d+,\d+(?:,\d+)?\)')
# The note of Roulette's tddlog engine's circuit that counts the calls of `infer`.
CT_CALLS = re.compile(r'^% infer was called (\d+) times', re.M)


def circulette_gates(path):
    """The gate ids of the program tddlog's reader writes of a netlist
    (NAME.pl), in the order of its gate_* facts: Circulette's faults 0 to
    n - 1."""
    with open(path) as f:
        return [int(g) for g in CT_GATE.findall(f.read())]


def circulette_atom(gates, faults):
    """The query of the reader's program that asks the posterior of
    Circulette's faults `faults` (one to three of its indices, of `gates`)."""
    ids = sorted(gates[i] for i in faults)
    return f'fault({ids[0]})' if len(ids) == 1 else f"faults({','.join(map(str, ids))})"


def circulette_size(atom):
    """How many faults a query of the reader's program asks the posterior
    of: 1 for fault(G), 2 or 3 for faults(...); None for another atom."""
    a = re.sub(r'\s+', '', atom)
    return a.count(',') + 1 if CT_QUERY.fullmatch(a) else None


def circulette_asked(n):
    """The posteriors Circulette asks of n faults: every single, pair and triple."""
    return n + math.comb(n, 2) + math.comb(n, 3)


def circulette_faults(calls):
    """The n faults of a run of circulette.rkt that called `infer` `calls`
    times: P(fail)'s, and one for each single, pair and triple of faults."""
    n = 0
    while 1 + circulette_asked(n) < calls:
        n += 1
    if 1 + circulette_asked(n) != calls:
        raise ValueError(f'{calls} calls of infer are not 1 + n + C(n,2) + C(n,3) for any n')
    return n


def circulette_order(n):
    """The faults that each call of `infer` after P(fail)'s asks of n faults,
    in the order of circulette.rkt's loops: the singles, the pairs and the
    triples, each ascending, the second call's first."""
    yield from ((i,) for i in range(n))
    yield from itertools.combinations(range(n), 2)
    yield from itertools.combinations(range(n), 3)


def circulette_rekey(answers, gates):
    """Answers keyed as the queries of a circuit that Roulette's tddlog
    engine writes of circulette.rkt's run on a netlist of the gates
    `gates`, qK(I) for the K-th call of `infer` (I the index of its value
    #t), keyed as the queries of the program tddlog's reader writes of the
    netlist (circulette_atom), the failure's call, q1, left out: those of
    Roulette+tddlog (circulette_circuit_answers) and of the systems that
    read that circuit before tddlog read the netlists. A key
    of another form, a call past the last or a call answered twice is an
    error."""
    by_call = {}
    for q, p in answers.items():
        m = re.fullmatch(r'q(\d+)\(\d+\)', q)
        if m is None:
            raise ValueError(f'{q} is no query of the circuit')
        k = int(m.group(1))
        if k in by_call:
            raise ValueError(f'call {k} of infer answered twice')
        by_call[k] = p
    by_call.pop(1, None)
    last = 1 + circulette_asked(len(gates))
    if by_call and not 2 <= min(by_call) <= max(by_call) <= last:
        raise ValueError(f'calls {min(by_call)} to {max(by_call)} of infer, of {last} for {len(gates)} faults')
    out = {}
    for k, faults in zip(itertools.count(2), circulette_order(len(gates))):
        if not by_call:
            break
        if k in by_call:
            out[circulette_atom(gates, faults)] = by_call.pop(k)
    return out


def circulette_circuit_answers(text, answers, log10_pe, gates):
    """(the posteriors, P(fail)) of tddlog's answers `answers`, with log10
    P(e) (--evidence-probability), to the circuit `text` that Roulette's
    tddlog engine writes of circulette.rkt's run on a netlist of the gates
    `gates`: its queries are the calls of `infer` (CT_CALLS, 1 + n + C(n,2)
    + C(n,3) for the gates' n faults), the first, q1, the failure, whose
    guard is the evidence (a fact; `q1(I) :- fail.` where no fault fails the
    outputs), so that P(fail) is P(e) times its answer, and each other
    answer is a posterior (circulette_rekey). Where P(fail) is 0 there is no
    posterior, and the answers are 'inconsistent', as tddlog's on the
    netlist."""
    m = CT_CALLS.search(text)
    if m is None:
        raise ValueError('the circuit does not count the calls of infer')
    calls = int(m.group(1))
    if circulette_faults(calls) != len(gates):
        raise ValueError(f'the circuit asks of {circulette_faults(calls)} faults, and the netlist has '
                         f'{len(gates)} gates')
    asked = {}
    for k, i, value in RT_VALUE.findall(text):
        if value == '#t':
            asked.setdefault(int(k), []).append(f'q{k}({i})')
    if sorted(asked) != list(range(1, calls + 1)) or any(len(qs) != 1 for qs in asked.values()):
        raise ValueError(f'the circuit does not ask the value #t of each of its {calls} calls once')
    if log10_pe is None:
        raise ValueError('tddlog prints no P(e)')
    if answers == 'inconsistent' and log10_pe == -math.inf:
        return 'inconsistent', 0.0
    if not isinstance(answers, dict):
        raise ValueError("tddlog answers none of the circuit's queries")
    missing = [qs[0] for qs in asked.values() if qs[0] not in answers]
    if missing:
        raise ValueError(f'tddlog answers no {missing[0]}')
    failure = answers[asked[1][0]]
    p_fail = 10 ** log10_pe * failure
    if p_fail == 0:
        return 'inconsistent', 0.0
    if abs(failure - 1) > CHECK_NOISE:
        raise ValueError("the circuit's evidence is not the failure, so its answers are no posteriors")
    return circulette_rekey({qs[0]: answers[qs[0]] for qs in asked.values()}, gates), p_fail


def printed_answers(rec, ref):
    """The answers of `rec`, a record of an instance, against the reference
    record `ref`. Roulette's on a Circulette instance computes every
    posterior and prints those above CT_PRINTED_ABOVE (by the number of
    faults the query asks of, circulette_size), so a query it leaves out
    where the reference's answer is at or below its threshold (within
    CHECK_NOISE) is answered: the reference's answer stands for it. A query
    left out above its threshold stays left out, and the record answers less
    than was asked."""
    a = rec.get('answers')
    if rec.get('system') != 'roulette' or rec.get('instance') not in CIRCULETTE or not isinstance(a, dict) \
            or not isinstance(ref.get('answers'), dict):
        return a
    full = dict(a)
    for q, p in ref['answers'].items():
        k = None if q in a else circulette_size(q)
        if k and p <= CT_PRINTED_ABOVE[k] + CHECK_NOISE:
            full[q] = p
    return full


def run_circulette(args, name, path, variant):
    """Roulette on a Circulette instance: circulette.rkt on its netlist, as
    Circulette's examples/Makefile runs it (`racket circulette.rkt
    NAME.cql`), Racket the --roulette template's (with roulette-lib and
    Circulette), rsdd its engine, compiled by `raco make` before the clock.
    It runs under roulette_timer.rkt (`roulette_timed`).
    The posteriors it prints (CT_FAULTS, Circulette's faults by index) are
    keyed as the queries of the program tddlog's reader writes of the
    netlist (NAME.pl, `path`; circulette_atom), and P(fail) is kept as
    `p_fail`, which tddlog's --evidence-probability gives as P(e)."""
    prog, rest = roulette_program(args, name)
    r = roulette_timed(args, prog, rest, args.timeout, cwd=os.path.dirname(rest[0]))
    if r['status'] != 'ok':
        return r
    m = CT_FAIL.search(r['stdout'])
    if m is None:
        tail = (r.get('stderr', '').strip() or r['stdout'].strip()).splitlines()
        return {'status': 'error', 'seconds': r['seconds'], 'error': 'no answers: ' + (tail[-1] if tail else 'empty output')}
    stats = {'p_fail': float(m.group(1)), 'inproc_seconds': r['inproc_seconds']}
    try:
        gates = circulette_gates(path)
        answers = {}
        for *faults, p in CT_FAULTS.findall(r['stdout']):
            fs = [int(f) for f in faults if f]
            if max(fs) >= len(gates) or len(set(fs)) != len(fs):
                raise ValueError(f'faults {fs}, and the netlist has {len(gates)} gates')
            q = circulette_atom(gates, fs)
            if q in answers:
                raise ValueError(f'the posterior of faults {fs} printed twice')
            answers[q] = float(p)
        if stats['p_fail'] == 0:
            # no posterior given a failure of probability zero: tddlog's
            # 'evidence is inconsistent' (parse_answers)
            if answers:
                raise ValueError(f'{len(answers)} posteriors given a failure of probability 0')
            answers = 'inconsistent'
    except (OSError, ValueError) as x:
        return {'status': 'error', 'seconds': r['seconds'], 'error': f'circulette: {x}'}
    return {'status': 'ok', 'seconds': r['seconds'], 'answers': answers, 'stats': stats}


# --- Systems for probabilistic programs (every exact method with an
# implementation is a baseline). Each is a command template given by
# `--NAME CMD` ({file}, {dir}, {base}), runs the instances it accepts, and
# answers the instance's queries; PROGRAM_SYSTEMS maps NAME to its
# `accepts(args, name)` and `run(args, name, path)`. The translation of the
# instance's program into the system's language is untimed, as dice2alea.py's;
# a system answered one component a run has the translations bounded by a
# time limit of their own (by_component).

PROGRAM_SYSTEMS = {}


def program_system(name, help, accepts, run):
    PROGRAM_SYSTEMS[name] = {'help': help, 'accepts': accepts, 'run': run}


def program_command(args, name, file):
    return getattr(args, name).format(file=file, dir=os.path.dirname(file), base=os.path.basename(file)).split()


# The exit status of a translator (dice2genfer.py, dice2sppl.py) whose
# target language cannot express the program: the instance is recorded as
# `unsupported`, with the translator's reason, and counts as unanswered.
REFUSED = 3


# A harness translator's time-out: a translation into a system's language
# (`translation`, by_component's and checker_masses' bounds on them) that
# did not finish. The system never got the program, and the translation's
# time is not charged to it, so the record is the harness's error and
# not the system's failure. Such a record
# carries `harness: 'timeout'`; older ones are known by their text.
HARNESS_TIMEOUT = re.compile(r'\S+\.py: (no translation within |the translation of \d+ components passed the time '
                             r'limit|the translations took \d+ s, beyond the time limit)')


def harness_timeout(rec):
    """Whether rec is a harness translator's time-out (HARNESS_TIMEOUT)."""
    return rec.get('status') == 'error' and (rec.get('harness') == 'timeout'
                                             or bool(HARNESS_TIMEOUT.match(rec.get('error') or '')))


def translation_bound(args):
    """The seconds the harness's translations of an instance into a
    system's language may take: --translation-timeout, by default four
    times --timeout. A translation written only to hand a program to one
    system is not charged to it, so the time limit does not bound it."""
    bound = getattr(args, 'translation_timeout', None)
    return 4 * args.timeout if bound is None else bound


def harness_timeout_record(script, what):
    """The record of a harness translation past its bound (translation_bound).
    The system never got the program, so the record is the harness's
    error, `harness: 'timeout'`, not the system's failure."""
    return {'status': 'error', 'harness': 'timeout', 'seconds': 0, 'error': f'{script}: {what}'}


def translation(args, script, argv):
    """(the output of `script ARGV`, None) bounded by translation_bound, or
    (the lines it printed, the record of its failure, whose error says how
    it ended: `ended`). Every translator the harness runs runs through it.
    A translator that writes a file a component (`--each`) prints a line
    once each is written, so that the lines of one that failed say which
    files are complete."""
    bound = translation_bound(args)
    try:
        t = subprocess.run([sys.executable, str(source_tree.script(script)), *argv], capture_output=True, text=True,
                           timeout=bound)
    except subprocess.TimeoutExpired as x:
        out = x.stdout.decode(errors='replace') if isinstance(x.stdout, bytes) else x.stdout or ''
        return out[:out.rfind('\n') + 1], harness_timeout_record(script, f'no translation within {bound:g} s')
    if t.returncode != 0:
        status = 'unsupported' if t.returncode == REFUSED else 'error'
        return t.stdout, {'status': status, 'seconds': 0, 'error': f'{script}: ' + ended(t.returncode, t.stderr)}
    return t.stdout, None


def ended(returncode, stderr):
    """How a process that failed ended: the tail of its stderr, which for a
    Python exception is its traceback, ending in the exception's type and
    message; led by the signal that killed it, if one did, which leaves
    nothing on stderr (a stack overflow's SIGSEGV; the SIGTERM or SIGKILL
    of a box out of memory, as dice2roulette.py on taint at 80 GB before
    `leaves`); its exit status where it printed nothing."""
    tail = stderr.strip()[-2000:]
    if returncode < 0:
        try:
            how = f'killed by {signal.Signals(-returncode).name}'
        except ValueError:
            how = f'killed by signal {-returncode}'
        return f'{how}: {tail}' if tail else how
    return tail or f'exit {returncode} with nothing on stderr'


def translated(args, script, argv, out, what):
    """Writes the translation of an instance (`script ARGV`, its output or the
    file OUT it writes) bounded by the timeout; the error record on failure, else None."""
    stdout, err = translation(args, script, argv)
    if err:
        return err
    if out is not None and stdout:
        write_new(out, stdout)
    return None


# --- Dice programs through a translation --------------------------------------
# Genfer, SPPL, Geni, Psi and Pluck run, besides their own programs, and
# WebPPL run on every instance that has Dice programs: the program behind the
# instance (`dice_source`) or the Dice programs of an Alea.jl program's
# circuit (NAME.dicemap), each translated into the system's language
# (dice2genfer.py, dice2sppl.py, dice2geni.py, dice2psi.py, dice2pluck.py,
# dice2webppl.py); noDice reads them as Dice does.
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
    up; `inproc_seconds` and `header_seconds` (WebPPL's) too, where the
    system gives them."""
    total, inproc, answers = 0.0, {}, {}

    def ask(label, source, joint):
        nonlocal total
        comps, r = masses(args, label, source, joint, max(0.0, args.timeout - total))
        total += r['seconds']
        for key in ('inproc_seconds', 'header_seconds'):
            if key in r.get('stats', {}):
                inproc[key] = inproc.get(key, 0.0) + r['stats'][key]
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
        comps, r = ask(name, dice_source(args, name, system), False)
        if comps is None:
            return {**r, 'seconds': total}
        try:
            answers = alea_answers(queries(path), comps)
        except ValueError as x:
            return {'status': 'error', 'seconds': total, 'error': f'{system}: {x}'}
    rec = {'status': 'ok', 'seconds': total, 'answers': answers}
    if inproc:
        rec['stats'] = {key: round(s, 3) for key, s in inproc.items()}
    return rec


def by_component(system, script, ext, flags, masses_of):
    """The `masses` of run_translated for a system that answers one variable
    a run (Genfer, Geni, Psi): SCRIPT writes a program for each component
    of the program's value from one translation (`--each PREFIX`, K's file
    PREFIXK.EXT, its type printed once it is written: `bool` or `int W`),
    each run with the arguments flags(n) for its n values (2 for a Boolean);
    masses_of(stdout, n) reads a run's {value: probability}, which must
    cover the n values. The record's seconds are the runs', as for every
    translated system; the translation is bounded by translation_bound, not
    by the time limit, since a program of thousands of components
    translates for hours: past it the record is the harness's time-out
    (harness_timeout_record), not the system's failure. The components a
    translation that failed (a later component refused, its bound) wrote
    run first, in order, and the record is then its failure."""
    def masses(args, label, source, joint, budget):
        j = ['--joint'] if joint else []
        prefix = os.path.abspath(os.path.join(args.dir, f'{label}.{system}'))
        listing, failed = translation(args, script, [source, '--each', prefix] + j)
        # the components' files, the one a translation that failed left unfinished among them
        with scratch(*(f'{prefix}{k}.{ext}' for k in range(len(listing.splitlines()) + 1))):
            comps, total = [], 0.0
            for k, line in enumerate(listing.splitlines()):
                ty = line.split()
                n = 2 if ty[0] == 'bool' else 2 ** int(ty[1])
                file = f'{prefix}{k}.{ext}'
                r = timed(program_command(args, system, file) + flags(n), max(0.0, budget - total))
                total += r['seconds']
                if r['status'] != 'ok':
                    return None, {**r, 'seconds': total}
                got = masses_of(r['stdout'], n)
                why = (f'masses of {sorted(got)} for {n} values' if sorted(got) != list(range(n)) else
                       'a mass is NaN as printed' if any(math.isnan(p) for p in got.values()) else
                       'every value has mass 0 as printed: the observations\' probability is below its precision'
                       if sum(got.values()) == 0 else None)
                if why:
                    return None, {'status': 'error', 'seconds': total,
                                  'error': f'{system}: {why}: ' + r['stdout'].strip()[-300:]}
                comps.append({'true': got[1], 'false': got[0]} if ty[0] == 'bool' else
                             {str(v): got[v] for v in range(n)})
            if failed:
                return None, {**failed, 'seconds': total}
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
    the distribution of the program's value, `Pr(V) = P` (the artifact to seven decimals of
    f32 numbers, the build run here in full, of f64 ones). The grids return a Boolean, which it prints as
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


# a term `c·δ(v)[`: its coefficient c runs to the `+` before it, but for the
# `+` of an exponent (`1.5e+00`, as Psi's --dp prints its numbers)
PSI_TERM = re.compile(r'(?:((?:[^+δ]|(?<=\d[eE])\+)*)·)?δ\((-?\d+)\)\[')


def parse_psi(out):
    """The distribution Psi prints for a program's value, `p(x) = c·δ(v)[x]+...`,
    as {value: probability}, or None where there is no such line. A coefficient
    is a rational such as `3/10`, a number such as `1.25e-03` (--dp) or a
    product of them."""
    for line in out.splitlines():
        if not line.startswith('p('):
            continue
        dist = {}
        for coef, value in PSI_TERM.findall(line.split(' = ', 1)[1]):
            expr = coef.replace('·', '*').strip() or '1'
            if not re.fullmatch(r'[0-9/*().\s-]+', re.sub(r'(?<=\d)[eE][+-]?(?=\d)', '', expr)):
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


def psi_linreg(name):
    """Whether instance `name` is a circuit of Psi's own linear regression
    (`bb_linreg_w*`), on which Psi runs its own program (run_psi_linreg)."""
    return name.startswith('bb_linreg_w')


# The Psi programs of the artifact of Cao et al. (UAI 2023) whose model is
# Alea.jl's (data/alea/arith/psi/ARITH.psi, as they are but a header): Psi
# reads its own program of those arith instances, and dice2psi.py's
# translation of the port of the others (luhn, radar and tugofwar, whose Psi
# programs are other models: docs/baselines.md).
PSI_ARITH = os.path.join(ARITH_DIR, 'psi')
# The last `return X;` of a program, main's
PSI_RETURN = re.compile(r'^([ \t]*)return[ \t]+(\w+)[ \t]*;?[ \t]*$', re.M)


def psi_program(name):
    """The artifact's Psi program of arith instance `name`, or None."""
    path = os.path.join(PSI_ARITH, name.removeprefix('arith_') + '.psi')
    return path if name.startswith('arith_') and os.path.exists(path) else None


def own_psi(args, name):
    return name in instances(args) and (psi_linreg(name) or psi_program(name) is not None)


def accepts_psi(args, name):
    return own_psi(args, name) or accepts_translated(args, name)


def run_psi(args, name, path):
    """Psi (Gehr et al., CAV 2016) on its own linear regression (`bb_linreg`,
    run_psi_linreg) and arith programs (run_psi_own), and on the other
    instances' Dice programs, translated by dice2psi.py (`run_translated`),
    one run for each component of a program's value (PSI_MASSES): Psi
    prints a component's distribution in exact rationals, or in numbers
    under its --dp backend (parse_psi reads both)."""
    if psi_linreg(name) and own_psi(args, name):
        return run_psi_linreg(args, name, path)
    if own_psi(args, name):
        return run_psi_own(args, name, path)
    return run_translated(args, name, path, 'psi', PSI_MASSES)


def query_components(path):
    """The components of the value behind the queries of the program at
    `path`, in order, as the ports' queries ask them (alea_answers): (None,
    QUERY) for a Boolean, a query of its own without arguments, and (PRED,
    N) for a number, the N queries PRED(0) ... PRED(N - 1) that follow."""
    comps = []
    for q in (re.sub(r'\s+', '', q) for q in queries(path)):
        pred = q.split('(')[0] if '(' in q else None
        if pred is not None and comps and comps[-1][0] == pred:
            comps[-1][1] += 1
        else:
            comps.append([pred, 1 if pred is not None else q])   # a Boolean: its query
    return [tuple(c) for c in comps]


def run_psi_own(args, name, path):
    """Psi on the artifact's program of arith instance `name` (psi_program),
    one run for each component of its value, as Psi is asked on every
    program (`run_translated`): a value that is an array (linext's and
    ranking's `return ps;`) is asked one entry a run (`return ps[K];`). A
    Boolean component answers its query with the mass of 1 (Psi's Booleans
    are 0 and 1); a number answers its N queries, the I-th the mass of I,
    every value it has lying in 0 ... N - 1 (a value outside them is an
    error: the ports' numbers are unsigned where Psi reads its own
    program). The seconds add up."""
    comps = query_components(path)
    with open(psi_program(name)) as f:
        text = f.read()
    ret = list(PSI_RETURN.finditer(text))
    if len(comps) > 1 and not ret:
        return {'status': 'error', 'seconds': 0, 'error': f'psi: {len(comps)} components and no `return X;` to index'}
    total, answers, qs = 0.0, {}, [re.sub(r'\s+', '', q) for q in queries(path)]
    for k, (pred, n) in enumerate(comps):
        file = os.path.abspath(os.path.join(args.dir, f'{name}.psi{k}.psi'))
        if len(comps) > 1:
            m = ret[-1]
            source = text[:m.start()] + f'{m.group(1)}return {m.group(2)}[{k}];' + text[m.end():]
        else:
            source = text
        with scratch(file):
            write_new(file, source)
            r = timed(program_command(args, 'psi', file), max(0.0, args.timeout - total))
        total += r['seconds']
        if r['status'] != 'ok':
            return {**r, 'seconds': total}
        try:
            dist = parse_psi(r['stdout'])
        except ValueError as x:
            return {'status': 'error', 'seconds': total, 'error': str(x)}
        if not dist:
            return {'status': 'error', 'seconds': total, 'error': 'psi: no distribution: ' + r['stdout'].strip()[-300:]}
        if pred is None:
            if not set(dist) <= {0, 1}:
                return {'status': 'error', 'seconds': total, 'error': f'psi: values {sorted(dist)} of a Boolean'}
            answers[n] = dist.get(1, 0.0)
            continue
        if not all(0 <= v < n for v in dist):
            return {'status': 'error', 'seconds': total,
                    'error': f'psi: values {sorted(dist)[:3]}... of a number with {n} values'}
        for q in qs:
            if q.split('(')[0] == pred:
                answers[q] = dist.get(int(q.split('(')[1].rstrip(')')), 0.0)
    return {'status': 'ok', 'seconds': total, 'answers': answers}


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
        with scratch(file):
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
    Boolean (1 or 0), an int's constructor `(Vk)` as k, or the list of an
    int's bits (the most significant first) as the int; {} for a query
    whose observations are impossible.
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
        m = re.fullmatch(r'\(V(\d+)\)', value)
        bits = None if m else PLUCK_BIT.findall(value)
        if m or bits:
            v = int(m.group(1)) if m else int(''.join('1' if b in ('true', '1') else '0' for b in bits), 2)
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
    with scratch(file):
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
    return comps, {'seconds': r['seconds'], 'stats': pluck_stats(r['stdout'])}


def pluck_stats(out):
    """pluck_dice.jl's time, load_pluck_file's, as `inproc_seconds`."""
    stats = {}
    for line in out.splitlines():
        f = line.split()
        if len(f) == 2 and f[0] == 'pluck_seconds':
            stats['inproc_seconds'] = round(float(f[1]), 3)
    return stats


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
    model = os.path.basename(gargs[1]).removesuffix('.pl')   # gen_pluck.py data/alea/pluck/NAME.pl
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
    (a rational such as 509/719 is read exactly, NaN as NaN)."""
    def number(s):
        return math.nan if s == 'NaN' else float(fractions.Fraction(s))
    stats, masses = {}, {}
    for line in out.splitlines():
        if m := GENFER_STAT.match(line):
            stats[m.group(1)[-1]] = number(m.group(2))
        elif m := GENFER_MASS.match(line):
            masses[int(m.group(1))] = number(m.group(2))
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
    each assignment of its Booleans (sppl_run). The seconds are the
    process's."""
    file = os.path.abspath(os.path.join(args.dir, f'{label}.sppl'))
    with scratch(file):
        err = translated(args, 'dice2sppl.py', [source] + (['--joint'] if joint else []), file, 'sppl')
        if err:
            return None, err
        r, comps, stats = sppl_run(args, file, budget)
    if r['status'] != 'ok':
        return None, r
    return comps, {'seconds': r['seconds'], 'stats': stats}


def sppl_run(args, file, budget):
    """sppl_dice.py on the SPPL program `file`: its run, the distributions
    of the components in order, each {VALUE: P} from its lines `sppl_c K
    VALUE P`, and its stats, the `sppl_seconds` it prints (the compile and
    the queries) `inproc_seconds`."""
    r = timed(program_command(args, 'sppl', os.path.join(HERE, 'sppl_dice.py')) + [file], budget)
    comps, stats = {}, {}
    for line in r['stdout'].splitlines() if r['status'] == 'ok' else []:
        f = line.split()
        if len(f) == 4 and f[0] == 'sppl_c':
            comps.setdefault(int(f[1]), {})[f[2]] = float(f[3])
        elif len(f) == 2 and f[0] == 'sppl_seconds':
            stats['inproc_seconds'] = float(f[1])
    return r, [comps[k] for k in sorted(comps)], stats


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


def webppl_masses(args, label, source, joint, budget):
    """A translated Dice program's masses (`run_translated`): dice2webppl.py's
    program, whose model WebPPL enumerates exactly (`Infer` with `method:
    'enumerate'`), run by webppl_dice.js through WebPPL's API, which reads
    each component's marginal from the one distribution of the value
    (`webppl_c K VALUE P`). The seconds are the process's; its
    `webppl_seconds`, WebPPL's compile and run of the program after a
    warm-up, without node's start and the modules' load, `inproc_seconds`,
    and `webppl_header_seconds`, the compile of WebPPL's header (its
    library, compiled with every program) within them, `header_seconds`,
    which own_seconds leaves out as the load of WebPPL's library."""
    file = os.path.abspath(os.path.join(args.dir, f'{label}.wppl'))
    with scratch(file):
        err = translated(args, 'dice2webppl.py', [source] + (['--joint'] if joint else []), file, 'webppl')
        if err:
            return None, err
        r = timed(program_command(args, 'webppl', os.path.join(HERE, 'webppl_dice.js')) + [file], budget)
    if r['status'] != 'ok':
        return None, r
    comps, stats = webppl_output(r['stdout'])
    return comps, {'seconds': r['seconds'], 'stats': stats}


def webppl_output(stdout):
    """(the distributions of the components, in order, the stats) that
    webppl_dice.js prints (`webppl_masses`)."""
    comps, stats = {}, {}
    keys = {'webppl_seconds': 'inproc_seconds', 'webppl_header_seconds': 'header_seconds'}
    for line in stdout.splitlines():
        f = line.split()
        if len(f) == 4 and f[0] == 'webppl_c':
            comps.setdefault(int(f[1]), {})[f[2]] = float(f[3])
        elif len(f) == 2 and f[0] in keys:
            stats[keys[f[0]]] = float(f[1])
    return [comps[k] for k in sorted(comps)], stats


def run_webppl(args, name, path):
    """WebPPL (Goodman and Stuhlmüller, dippl.org) on the instances' Dice
    programs, translated by dice2webppl.py and enumerated exactly
    (`webppl_masses`)."""
    return run_translated(args, name, path, 'webppl', webppl_masses)


program_system('webppl', "WebPPL (Goodman and Stuhlmüller, dippl.org) by exact enumeration on the Dice programs, the "
               "circuits' included, translated by dice2webppl.py, and on the logic programs, written by "
               "problog2ppl.py, run by webppl_dice.js; e.g. 'env "
               "NODE_PATH=WEBPPL/node_modules prlimit --stack=unlimited NODE --stack-size=65500 {file}'",
               accepts_translated, run_webppl)


def accepts_alea_tddlog(args, name):
    gargs = instances(args)[name][1]
    return alea_program(args, gargs) is not None or lp_route(gargs, 'alea-tddlog', getattr(args, 'pr', False))


ALEA_EXPORT_TIMES = ('alea_seconds', 'alea_compile_seconds', 'alea_warm_seconds')


def run_alea_tddlog(args, name, path, lp=None):
    """Alea.jl+tddlog on an instance of ALEA_PROGRAMS'
    families or of the ppl suite's arith and pluck programs (ALEA_OF),
    or on a logic program as problog2ppl.py writes it for Alea.jl (`lp`,
    from run_lp: the program as alea_command takes it and the queries it
    answers, in order), beside plain tddlog on the instance's program:
    Alea.jl runs the instance's Alea.jl program (alea_program) up to the
    circuit it builds and writes it as a ProbLog program, by the export
    the bitblast suite's gen makes (alea_command's `export` under
    --alea-tddlog's template, timed warm as Alea.jl's own record is), and
    tddlog answers from that program (tddlog_on_circuit). The time
    charged, kept as `inproc_seconds` (own_seconds), is the export's own,
    its warm run where it made one (else its first less Julia's
    compilation), and tddlog's run; the record's seconds are the two
    processes' wall times. The record holds the export: charge.py charges
    an export to the readers of the bitblast suite's circuits alone
    (TRANSLATE), so it adds nothing here. The circuit's answers are named
    as Alea.jl's are (alea_program_answers, a number's worlds by the values
    the export prints after its timed part); its files go once tddlog has
    read it, as Roulette+tddlog's do."""
    family, gargs = instances(args)[name]
    own = alea_program(args, gargs) if lp is None else lp[0]
    if own is None:
        return {'status': 'error', 'seconds': 0, 'error': f'alea-tddlog: no Alea.jl program for family {family}'}
    prefix = os.path.abspath(os.path.join(args.dir, f'{name}.alea'))
    pl = prefix + '.pl'
    try:
        x = timed(alea_command(args, 'export', own, prefix, warm_below=args.timeout / 3,
                               template=getattr(args, 'alea-tddlog')), args.timeout)
        if x['status'] != 'ok':
            return x
        times, values = {}, {}
        for line in x['stdout'].splitlines():
            f = line.split()
            if len(f) == 2 and f[0] in ALEA_EXPORT_TIMES:
                times[f[0]] = float(f[1])
            elif len(f) == 3 and f[0] == 'alea_value':   # the value of a world of a number's bits
                values[f[1]] = int(f[2])
        if 'alea_seconds' not in times or not os.path.exists(pl):
            return {'status': 'error', 'seconds': x['seconds'], 'error': 'alea-tddlog: alea_bench.jl export wrote no circuit'}
        jit = times.get('alea_compile_seconds', 0.0)
        export_s = times.get('alea_warm_seconds', times['alea_seconds'] - (0.0 if math.isnan(jit) else jit))
        stats = {'export_s': round(export_s, 3), **circuit_size(pl)}
        t = tddlog_on_circuit(args, pl, args.timeout - export_s)
    finally:
        for f in [pl, prefix + '.dicemap'] + glob.glob(glob.escape(prefix) + '__d*.dice'):
            if os.path.exists(f):
                os.remove(f)
    seconds = x['seconds'] + t['seconds']
    if t['status'] != 'ok':
        return {**t, 'seconds': seconds, 'stats': stats}
    charge = export_s + t['seconds']
    stats.update(tddlog_s=round(t['seconds'], 3), inproc_seconds=round(charge, 3))
    if charge > args.timeout:
        return {'status': 'timeout', 'seconds': seconds, 'stats': stats}
    answers = parse_answers(t['stdout'])
    try:
        if answers == 'inconsistent':
            raise ValueError("the circuit's evidence is inconsistent")
        answers = alea_program_answers(path, answers, values) if lp is None else lp_named(lp[1], alea_bools(answers))
    except ValueError as e:
        return {'status': 'error', 'seconds': seconds, 'error': f'alea-tddlog: {e}', 'stats': stats}
    return {'status': 'ok', 'seconds': seconds, 'answers': answers, 'stats': stats}


program_system('alea-tddlog', "Alea.jl+tddlog on the roulette suite's diamond and ladder, Alea.jl's own diamond and "
               "the ladder's port (ALEA_PROGRAMS), on the ppl suite's arith and pluck programs (ALEA_OF) and on the "
               "logic programs as problog2ppl.py writes them, which "
               "Alea.jl runs up to the circuit it exports (alea_bench.jl export) for tddlog to answer; the command "
               "template of --alea's Julia, e.g. 'env JULIA_DEPOT_PATH=... julia --project=ALEA {file}'",
               accepts_alea_tddlog, run_alea_tddlog)


program_system('nodice', "noDice (Gürtler and Kaminski, OOPSLA 2026) on the Dice programs, the circuits' included, as "
               "Dice runs them; e.g. 'NODICE/_build/default/bin/nDice.exe {file}'", accepts_translated,
               lambda args, name, path: run_dice(args, name, path, 'nodice'))


def write_new(path, text):
    """Write text as a new file at path, renamed into place, never into the
    file already there. A job's directory may hold hard links to its
    table's files (a runner may copy a table's instances with cp -al for
    each system's job), and a file written over in
    place changes every copy and the original; parts of one run that share
    --dir read each other's files. PITA's LPAD written over in place
    truncated the file another part was consulting (Geni's programs:
    "Syntax error: Unexpected end of file", then `Unknown procedure:
    main/1`). Every file a run writes under a name its directory may hold
    already goes through here; a system's staging directory that the run
    removes and makes afresh (Ace's, Merlin's, the network tools') needs
    not."""
    part = f'{path}.{os.getpid()}.part'
    with open(part, 'w') as f:
        f.write(text)
    os.replace(part, path)


@contextlib.contextmanager
def scratch(*paths):
    """Removes the files PATHS when the block ends, whether the call in it
    succeeded, failed or raised: a translation written for one call of a
    system (a query's, a component's or a variable's program, or the
    program of one of a circuit's Dice programs), which nothing reads after
    that call. A run that writes one for each query or component of an
    instance and keeps it leaves a copy of the program for each beside the
    instance, so that a run over many instances fills the disk. A file
    already gone (one the translation did not write) is passed over."""
    try:
        yield
    finally:
        for path in paths:
            with contextlib.suppress(FileNotFoundError):
                os.remove(path)


def run_pita(args, name, path, own=None):
    """PITA (cplint) on the LPAD translation of the program, or where the
    instance is an LPAD written for PITA (`own`, cplint's examples) on that
    program as it is, with a driver that asks the converted program's
    queries (`problog2pita.own`; its own system's program is the
    model). One swipl process consults the program and asks `prob/2` for
    every query, so the seconds cover parsing, transformation and every
    query, as the other systems' do. PITA answers one query at a time and
    shares nothing between them, which is the shape of the system, not of
    the harness. Its tables may fill the run's address-space limit (`ulimit
    -v`), not only SWI's default table space of 1 GB, so that the memory
    limit binds, as it does for the other systems."""
    lpad = os.path.join(args.dir, name + '.lpad.pl')
    write_new(lpad, problog2pita.own(open(own).read(), open(path).read()) if own
              else problog2pita.program(open(path).read(), driver=True))
    packs = ['-g', f"attach_packs('{args.pita_packs}',[duplicate(replace)])"] if args.pita_packs else []
    limit = address_space_limit()
    space = ['-g', f'set_prolog_flag(table_space, {limit})'] if limit else []
    return timed([args.pita, '-q'] + packs + space +
                 ['-g', f"consult('{os.path.abspath(lpad)}'), main", '-t', 'halt'], args.timeout)


# aspmc's ProbLog front end recognizes query/1 but reads evidence/1,2 as
# ordinary facts, and exits successfully with unconditional probabilities.
# A program with evidence is given to it with the evidence asked as queries
# (`aspmc_conditioned`), and P(q | e) is P(q and e) / P(e), as Scallop's
# answers are (`problog2scallop.answers`).
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
                raise ValueError(f'a query that is not an atom, which the evidence as queries cannot ask: {clause}')
            queries.append((' '.join(m.group(2).split()), body and ' '.join(body.split())))
        elif body is not None:
            raise ValueError(f'an observation with a body, which the evidence as queries cannot ask: {clause}')
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
    # each `_` of a query a name of its own, the same in the head, the body and the query it is
    # asked by, which aspmc_variables would give three names, the head's unbound
    stem, count = aspmc_fresh(text, '_Q'), itertools.count(1)
    for q, body in queries:
        q = ASPMC_VARIABLE.sub(lambda m: f'{stem}{next(count)}' if m.group('v') == '_' else m.group(0), q)
        guard = f'{body}, ' if body else ''
        lines += [f'{ASPMC_QE}{q} :- {guard}{q}, {e}.',
                  f'query({ASPMC_QE}{q})' + (f' :- {body}.' if body else '.')]
    return '\n'.join(lines) + '\n'


def parse_aspmc(out, conditioned=False):
    """aspmc prints `[RESULT] aspmc: <query>: <probability>`, on stderr.
    For a program asked through `aspmc_conditioned` (`conditioned`), the
    answer to Q is P(bench_qe_Q) / P(bench_e), and 'inconsistent' where
    P(bench_e) is 0, or where aspmc prints nothing, bench_e an atom its
    grounder removed as no rule derives it."""
    res = {}
    for line in out.splitlines():
        m = re.match(r'\[RESULT\] aspmc:\s+(.*):\s+([0-9.eE+-]+)\s*$', line)
        if m:
            res[re.sub(r'\s+', '', m.group(1))] = float(m.group(2))
    if ASPMC_E not in res:
        return 'inconsistent' if conditioned and not res else res
    if res[ASPMC_E] == 0:
        return 'inconsistent'
    return {q.removeprefix(ASPMC_QE): p / res[ASPMC_E] for q, p in res.items() if q.startswith(ASPMC_QE)}


# aspmc's grammar reads a probability as 1, 1.0... or 0.digits, and not in
# exponent notation, in which a small one is written (gen_bn.py's 1e-05),
# nor as a fraction (the ProbLog tutorial's `1/3::`).
ASPMC_EXPONENT = re.compile(r'(?<![\w.])(\d+(?:\.\d*)?[eE][-+]?\d+)(?=\s*::)')
ASPMC_FRACTION = re.compile(r'(?<![\w./])(\d+(?:\.\d+)?)\s*/\s*(\d+(?:\.\d+)?)(?=\s*::)')


def aspmc_numbers(text):
    """The ProbLog program `text` with each probability in exponent notation
    written out in positional notation, exactly: the same double; and each
    fraction A/B as the double ProbLog divides it to, A / B, written so."""
    text = ASPMC_FRACTION.sub(lambda m: format(decimal.Decimal(repr(float(m.group(1)) / float(m.group(2)))), 'f'),
                              text)
    return ASPMC_EXPONENT.sub(lambda m: format(decimal.Decimal(m.group(1)), 'f'), text)


# aspmc's grammar has no `is`: its grounder, clingo, binds a variable to an
# integer expression with `=` (the Dice loops' step rules, `T1 is T + 1`).
ASPMC_IS = re.compile(r'(?<![\w])([A-Z_]\w*)\s+is\s+')


def aspmc_is(text):
    """The ProbLog program `text` with each `V is E` written `V = E`."""
    return ASPMC_IS.sub(r'\1 = ', text)


# The comments, quoted atoms, strings and character codes of a ProbLog
# program, which the rewrites for aspmc below leave as they are.
ASPMC_SKIP = r"""'(?:[^'\\]|\\.|'')*'|"(?:[^"\\]|\\.)*"|/\*.*?\*/|0'\\?.|%[^\n]*"""
# aspmc's grammar reads a `%` comment only with a character after the `%`: a
# line that is a `%` alone, as the ProbLog tutorial and our ports write between
# a comment's paragraphs, stops its lexer ("No terminal matches '%'"). It reads
# no block comment (`/* */`) either.
ASPMC_COMMENT = re.compile(ASPMC_SKIP, re.S)


def aspmc_comments(text):
    """The ProbLog program `text` without its block comments and its empty
    `%` comments, which aspmc's grammar cannot read, a block comment leaving
    its line breaks; quoted atoms, strings and the other comments are left
    as they are."""
    def sub(m):
        s = m.group(0)
        if s.startswith('/*'):
            return '\n' * s.count('\n') or ' '
        return '' if s.startswith('%') and not s[1:].strip() else s
    return ASPMC_COMMENT.sub(sub, text)


def aspmc_uncommented(text):
    """`text` with each comment blanked, its line breaks kept: its clauses
    at the same places."""
    return ASPMC_COMMENT.sub(lambda m: re.sub(r'[^\n]', ' ', m.group(0)) if m.group(0)[0] in '%/' else m.group(0),
                             text)


def aspmc_fresh(text, stem, tail=r'\d'):
    """`stem`, its last character repeated until no name of `text` starts
    with it followed by `tail`: the start of the names a rewrite adds."""
    while re.search(rf'(?<!\w){re.escape(stem)}{tail}', text):
        stem += stem[-1]
    return stem


def aspmc_spans(text, positions):
    """The (start, end) of each clause of `text` (aspmc_uncommented) that one
    of the ascending `positions` falls in: from the full stop of the clause
    before it, a `.` before white space, to its own."""
    spans = []
    for p in positions:
        if spans and p < spans[-1][1]:
            continue
        s = p
        while (s := text.rfind('.', 0, s)) >= 0 and not text[s + 1:s + 2].isspace():
            pass
        e = p
        while (e := text.find('.', e)) >= 0 and e + 1 < len(text) and not text[e + 1].isspace():
            e += 1
        spans.append((s + 1, len(text) if e < 0 else e))
    return spans


def aspmc_rewritten(text, positions, rewrite):
    """The ProbLog program `text` with each clause that one of the ascending
    `positions` falls in written as rewrite(clause) writes it, `clause` its
    text without comments and full stop: the clauses that take its place, or
    None where it stays as it is. Only those clauses are read, so that a
    rewrite of a few clauses of a large program costs little."""
    bare = aspmc_uncommented(text)
    out, at = [], 0
    for s, e in aspmc_spans(bare, positions):
        clause = bare[s:e]
        new = rewrite(clause.strip())
        if new is not None:
            out += [text[at:s + len(clause) - len(clause.lstrip())], '.\n'.join(new)]
            at = e
    return ''.join(out) + text[at:] if out else text


def aspmc_names(text):
    """The variables of `text` in order, `_` each time it occurs."""
    return [m.group('v') for m in ASPMC_VARIABLE.finditer(text) if m.group('v')]


def aspmc_enclosed(literal):
    """Whether `literal` is a goal in parentheses, `(G)`."""
    from lpad2problog import split_top
    return literal.startswith('(') and split_top(literal, ')')[1:] == ['']


def aspmc_disjuncts(body):
    """The disjuncts of the ProbLog body `body`, each the list of its
    literals: its disjunctions (`;`) and its goals in parentheses written
    out, ProbLog's `not` as `\\+`, and `true` left out; a negated goal in
    parentheses, `\\+ (G)`, is one literal. ValueError on an if-then-else,
    which aspmc's grammar has no form of."""
    from lpad2problog import split_top
    if re.search(r'->', ASPMC_COMMENT.sub(' ', body)):
        raise ValueError(f"an if-then-else, which aspmc's grammar has no form of: {' '.join(body.split())}")
    out = []
    for alt in split_top(body, ';'):
        conjs = [[]]
        for lit in split_top(alt, ','):
            lit = lit.strip()
            m = re.fullmatch(r'not(?:\s+|(?=\())(.+)', lit, re.S)
            if m:
                lit = '\\+' + m.group(1).strip()
            if lit == 'true':
                continue
            if aspmc_enclosed(lit):
                inner = aspmc_disjuncts(lit[1:-1])
                conjs = [c + d for c in conjs for d in inner]
            else:
                conjs = [c + [lit] for c in conjs]
        out += conjs
    return out


def aspmc_weighted(head):
    """Whether the head `head` has a probability (`p::a`, an annotated disjunction)."""
    from lpad2problog import split_top
    return any(problog2pita.head_atom(d)[0] for d in split_top(head, ';'))


# aspmc's grammar has no quoted atom (cplint's `'$choice_3'`, PRISM's `'A'`):
# each is given a plain name, which means the same, as clingo compares names:
# a program that compares atoms by the standard order of terms (`@<`) keeps
# that order only where the names are its own.
ASPMC_PLAIN = re.compile(r'[a-z][a-zA-Z0-9_]*')


def aspmc_quoted(text):
    """(program, names): the ProbLog program `text` with each quoted atom
    written as a plain name, `'abc'` as abc, the same atom, any other as
    bench_q1, bench_q2..., one for each atom; and the quoted atom of each
    such name. ValueError where it compares terms by the standard order
    (`@<`...) and has an atom so renamed."""
    stem, names, by = aspmc_fresh(text, 'bench_q'), {}, {}

    def sub(m):
        s = m.group(0)
        if not s.startswith("'"):
            return s
        atom = re.sub(r"''|\\(.)", lambda e: "'" if e.group(0) == "''" else {'n': '\n', 't': '\t'}.get(
            e.group(1), e.group(1)), s[1:-1])
        if ASPMC_PLAIN.fullmatch(atom) and atom != 'not':
            return atom
        if atom not in by:
            by[atom] = f'{stem}{len(by) + 1}'
            names[by[atom]] = s
        return by[atom]
    out = ASPMC_COMMENT.sub(sub, text)
    if names and any(m.group('op') in ASPMC_ORDER for m in ASPMC_OPERATOR.finditer(out)):
        raise ValueError('a comparison by the standard order of terms in a program with quoted atoms, '
                         "which the names aspmc's grammar reads order otherwise")
    return out, names


def aspmc_unquoted(query, names):
    """The query `query` aspmc answered with each name of aspmc_quoted's
    `names` written as the quoted atom it stands for."""
    return re.sub(r'(?<!\w)\w+', lambda m: names.get(m.group(0), m.group(0)), query) if names else query


# Prolog's comparisons in the notation of aspmc's grammar, which is its
# grounder clingo's, and ProbLog's second neck `<-`. On the ground terms
# ProbLog compares, `\=` and `\==` are clingo's `!=`, `==` its `=`, and the
# arithmetic comparisons of integers (`=:=`, `=\=`, `=<`) its `=`, `!=` and
# `<=`; the standard order of terms (`@<`...) agrees with clingo's order of
# integers and of names (aspmc's grammar has no floats).
ASPMC_OPERATORS = {'<-': ':-', '\\=': '!=', '\\==': '!=', '=\\=': '!=', '==': '=', '=:=': '=', '=<': '<=',
                   '@<': '<', '@=<': '<=', '@>': '>', '@>=': '>='}
ASPMC_ORDER = {'@<', '@=<', '@>', '@>='}
# the comparisons whose meaning depends on what is bound when ProbLog calls
# them: an unbound variable unifies with any term, is identical to none, and
# comes first in the standard order, where clingo binds it before it compares
ASPMC_BINDING = {'\\=', '\\==', '=='} | ASPMC_ORDER
ASPMC_SYMBOL = r'[-+*/\\^<>=~:.?@#&$]'   # Prolog's symbol characters, which make one token together
ASPMC_OPERATOR = re.compile(ASPMC_SKIP + rf'|(?<!{ASPMC_SYMBOL})(?P<op>' + '|'.join(
    map(re.escape, sorted(ASPMC_OPERATORS, key=len, reverse=True))) + rf')(?!{ASPMC_SYMBOL})', re.S)


def aspmc_operators(text):
    """The ProbLog program `text` with its neck `<-` and its comparisons in
    the notation aspmc's grammar reads (ASPMC_OPERATORS). ValueError where a
    comparison of ASPMC_BINDING reads a variable, `_` included, that neither
    the head of its clause nor a positive literal before it binds (`p(X) :-
    X \\= a, q(X).`)."""
    found = set()

    def sub(m):
        op = m.group('op')
        if op is None:
            return m.group(0)
        found.add(op)
        return ASPMC_OPERATORS[op]
    out = ASPMC_OPERATOR.sub(sub, text)
    if found & ASPMC_BINDING:
        aspmc_bound_first(ASPMC_OPERATOR.sub(lambda m: ':-' if m.group('op') == '<-' else m.group(0), text))
    return out


def aspmc_bound_first(text):
    """ValueError where a comparison of ASPMC_BINDING in the ProbLog program
    `text` reads a variable that neither the head of its clause (which the
    call binds) nor a positive literal before it binds."""
    def check(lits, bound, clause):
        for lit in lits:
            neg = lit.startswith('\\+')
            inner = lit[2:].strip() if neg else lit
            if neg and aspmc_enclosed(inner):
                for conj in aspmc_disjuncts(inner[1:-1]):
                    check(conj, set(bound), clause)
            elif any(m.group('op') in ASPMC_BINDING for m in ASPMC_OPERATOR.finditer(lit)):
                free = [v for v in aspmc_names(lit) if v not in bound]
                if free:
                    raise ValueError(f'a comparison of {free[0]}, which ProbLog compares before a literal binds '
                                     f'it and clingo binds first: {clause}')
            elif not neg:
                bound.update(v for v in aspmc_names(lit) if v != '_')

    def visit(clause):
        head, body = problog2pita.split_neck(clause)
        for conj in aspmc_disjuncts(body) if body is not None else ():
            check(conj, {v for v in aspmc_names(head) if v != '_'}, ' '.join(clause.split()))
    aspmc_rewritten(text, [m.start() for m in ASPMC_OPERATOR.finditer(text) if m.group('op') in ASPMC_BINDING],
                    visit)


# The bodies aspmc's grammar does not read, which aspmc_bodies writes as
# conjunctions, the negated heads, and the calls of ProbLog's built-ins,
# which it has none of: its grounder clingo reads one as an atom no rule
# derives.
ASPMC_BUILTIN = (problog2pita.SWI_BUILTIN | {'subquery/2', 'subquery/3'}) - {'true/0', 'fail/0', 'false/0', 'not/1'}
_CALLED = '|'.join(sorted({f.rsplit('/', 1)[0] for f in ASPMC_BUILTIN if not f.endswith('/0')}))
_ATOMS = '|'.join(sorted(f.rsplit('/', 1)[0] for f in ASPMC_BUILTIN if f.endswith('/0')))
ASPMC_BODY = re.compile(
    ASPMC_SKIP + r'|(?P<form>:-(?:[^.;%]|\.(?!\s))*;|\\\+\s*\(|(?<![\w$])not(?=\s*\(|\s+[a-z\\(])'
    r'|(?::-|,|;|\()\s*(?:true(?=\s*[,;)]|\s*\.(?:\s|$))|\()|->)|(?P<neg>::\s*\\\+)'
    rf'|(?P<call>(?<![\w$])(?:(?:{_CALLED})\s*\(|(?:{_ATOMS})(?![\w$(])))', re.S)


def aspmc_bodies(text):
    """The ProbLog program `text` with each body in the form aspmc's grammar
    reads, a conjunction of literals, which means the same: a body with
    disjunctions as a clause for each of its disjuncts (aspmc_disjuncts); a
    negated conjunction, disjunction or comparison `\\+ (G)` as the negation of an atom
    bench_n1(V..) over the variables G shares with the rest of its clause,
    defined by `bench_n1(V..) :- C, (G).`, C the positive literals before it,
    which bind V.. as ProbLog's call does and as clingo's safety needs, and
    hold wherever the clause reads bench_n1 (C is left out where V.. is
    empty); a negated literal in parentheses as `\\+a`; `true` left out. A
    predicate p with a negated head (`0.2::\\+p :- b.`, ProbLog's
    inhibition) is written as ProbLog reads it, and as problog2pita.py
    writes it for PITA: its positive clauses define bench_pos_p, its negated
    ones bench_neg_p, and p(X0..) :- bench_pos_p(X0..), \\+bench_neg_p(X0..).
    ValueError where it
    cannot: a probabilistic clause with a disjunctive body (a clause for each
    disjunct would make a choice for each), an if-then-else, a call of a
    built-in of ProbLog's (ASPMC_BUILTIN) that the program does not define."""
    from lpad2problog import split_top
    forms = [m for m in ASPMC_BODY.finditer(text) if m.lastgroup in ('form', 'neg', 'call')]
    if not forms:
        return text
    defined, negated = (problog2pita.defined(aspmc_uncommented(text)) if any(m.lastgroup != 'form' for m in forms)
                        else (set(), set()))
    aux, pos, neg = (aspmc_fresh(text, s, t) for s, t in (('bench_n', r'\d'), ('bench_pos_', ''), ('bench_neg_', '')))
    count = itertools.count(1)
    positions = {m.start() for m in forms}
    if negated:
        name = '|'.join(re.escape(f.rsplit('/', 1)[0]) for f in negated)
        positions |= {m.start() for m in re.finditer(ASPMC_SKIP + rf'|(?P<p>(?<![\w$])(?:{name})(?!\w))', text, re.S)
                      if m.group('p')}

    def polar(head):
        if not negated or re.fullmatch(r'\s*(query|evidence)\s*\(.*\)\s*', head, re.S):
            return head
        out = []
        for d in split_top(head, ';'):
            label, atom, negative = problog2pita.head_atom(d)
            out.append(label + (neg if negative else pos) + atom if problog2pita.functor(atom) in negated
                       else d.strip())
        return head if out == [d.strip() for d in split_top(head, ';')] else '; '.join(out)

    def rewrite(clause):
        head, body = problog2pita.split_neck(clause)
        if not head.strip():
            return None   # a directive
        h = polar(head)
        changed = h is not head
        if body is None:
            return [h.strip()] if changed else None
        done, todo = [], [(h, body)]
        while todo:
            h, b = todo.pop(0)
            conjs = aspmc_disjuncts(b)
            if conjs != [[lit.strip() for lit in split_top(b, ',')]]:
                changed = True
                if len(conjs) > 1 and aspmc_weighted(h):
                    raise ValueError("a probabilistic clause with a disjunctive body, which aspmc's grammar writes "
                                     'only as a clause for each disjunct, a choice each: ' + ' '.join(clause.split()))
            for conj in conjs:
                lits = []
                for i, lit in enumerate(conj):
                    inner = lit[2:].strip() if lit.startswith('\\+') else None
                    enclosed = inner is not None and aspmc_enclosed(inner)
                    goal = inner[1:-1].strip() if enclosed else inner
                    # a negated group: a conjunction, a disjunction, or a goal in parentheses that is not
                    # an atom (`\+ (X = a)`), which aspmc's grammar does not negate
                    if not enclosed or (len(split_top(goal, ',')) == 1 and len(split_top(goal, ';')) == 1
                                        and problog2pita.functor(goal)):
                        f = problog2pita.functor(lit if goal is None else goal)
                        if f in ASPMC_BUILTIN and f not in defined:
                            raise ValueError(f"a call of {f}, a built-in of ProbLog's that aspmc's language does "
                                             f"not have: {' '.join(clause.split())}")
                        if enclosed:
                            lit, changed = '\\+' + goal, True
                        lits.append(lit)
                        continue
                    rest = set(aspmc_names(h + ' ' + ' '.join(conj[:i] + conj[i + 1:])))
                    shared = [v for v in dict.fromkeys(aspmc_names(goal)) if v != '_' and v in rest]
                    atom = f'{aux}{next(count)}' + (f"({','.join(shared)})" if shared else '')
                    context = [c for c in conj[:i] if not c.startswith('\\+')] if shared else []
                    todo.append((atom, ', '.join(context + [f'({goal})'])))
                    lits.append('\\+' + atom)
                    changed = True
                done.append(f"{h.strip()} :- {', '.join(lits)}" if lits else h.strip())
        return done if changed else None
    out = aspmc_rewritten(text, sorted(positions), rewrite)
    for f in sorted(negated):
        name, n = f.rsplit('/', 1)
        args = f"({','.join(f'X{i}' for i in range(int(n)))})" if int(n) else ''
        out += f'\n{name}{args} :- {pos}{name}{args}, \\+{neg}{name}{args}.\n'
    return out


# aspmc 1.1.1's ProbLog front end stops with a TypeError on a probabilistic
# clause whose body compares (`0.999::p(V) :- m(M, I, V), I != 0.`, the taint
# apps): _prepare_grounding reads the variables of the body's literals, and
# ArtithmeticAtom.get_variables returns None. Such a body is folded into an
# atom over its variables, which tells the same groundings apart.
ASPMC_COMPARISON = rf'(?<!{ASPMC_SYMBOL})(?P<op>!=|<=|>=|<|>|=)(?!{ASPMC_SYMBOL})'
ASPMC_COMPARES = re.compile(ASPMC_SKIP + '|' + ASPMC_COMPARISON, re.S)


def aspmc_folded(text):
    """The ProbLog program `text`, its variables named as aspmc's grammar
    reads them (aspmc_variables), with the body B of each probabilistic
    clause that has a comparison among its literals folded into an atom
    over B's variables, bench_b1(V..) :- B.: the clause has the same
    groundings, each a choice of its own, and no comparison in its body."""
    from lpad2problog import split_top
    stem, count = aspmc_fresh(text, 'bench_b'), itertools.count(1)

    def compares(lit):
        s = ASPMC_COMMENT.sub(' ', lit)
        while (t := re.sub(r'\([^()]*\)', ' ', s)) != s:
            s = t
        return re.search(ASPMC_COMPARISON, s) is not None

    def rewrite(clause):
        head, body = problog2pita.split_neck(clause)
        if body is None or not aspmc_weighted(head) or not any(compares(lit) for lit in split_top(body, ',')):
            return None
        names = [v for v in dict.fromkeys(aspmc_names(body)) if v != '_']
        atom = f'{stem}{next(count)}' + (f"({','.join(names)})" if names else '')
        return [f'{head.strip()} :- {atom}', f'{atom} :- {body.strip()}']
    return aspmc_rewritten(text, [m.start() for m in ASPMC_COMPARES.finditer(text) if m.group('op')], rewrite)


def aspmc_constants(text):
    """The ProbLog program `text`, its comparisons in clingo's notation
    (aspmc_operators), with each operand of a comparison that is a ground
    term, which aspmc's grammar does not read there (it reads variables and
    integer expressions), compared through a variable bound by a fact that
    holds the term: `B != brand45` as `bench_k1(K1), B != K1` and
    `bench_k1(brand45).`, which means the same."""
    from lpad2problog import split_top
    stem, var, terms = aspmc_fresh(text, 'bench_k'), aspmc_fresh(text, 'K'), {}

    def flat(s):   # s with its quoted parts and what parentheses enclose blanked, at the same places
        s = ASPMC_COMMENT.sub(lambda m: ' ' * len(m.group(0)), s)
        while (t := re.sub(r'\([^()]*\)', lambda m: ' ' * len(m.group(0)), s)) != s:
            s = t
        return s

    def operand(side, lits):
        t = side.strip()
        if aspmc_names(t) or not re.search(r'(?<!\w)[a-z]', ASPMC_COMMENT.sub(' x ', t)):
            return side   # a variable's expression, or an integer one (a string is a term)
        k = terms.setdefault(t, len(terms) + 1)
        lits.append(f'{stem}{k}({var}{k})')
        return f' {var}{k} '

    def rewrite(clause):
        head, body = problog2pita.split_neck(clause)
        if body is None or not head.strip():
            return None
        out, changed = [], False
        for lit in (lit.strip() for lit in split_top(body, ',')):
            m = None if lit.startswith('\\+') else re.search(ASPMC_COMPARISON, flat(lit))
            if m:
                bound = []
                new = (operand(lit[:m.start()], bound) + m.group('op') + operand(lit[m.end():], bound)).strip()
                if bound:
                    lit, changed = re.sub(r'\s+', ' ', new), True
                    out += bound
            out.append(lit)
        return [f"{head.strip()} :- {', '.join(out)}"] if changed else None
    out = aspmc_rewritten(text, [m.start() for m in ASPMC_COMPARES.finditer(text) if m.group('op')], rewrite)
    return out + ''.join(f'\n{stem}{k}({t}).' for t, k in terms.items()) + ('\n' if terms else '')


# clingo projects an anonymous variable out of a body of two or more
# literals into an auxiliary atom, which has no symbol, and aspmc's ProbLog
# front end looks each ground rule's head up by its symbol: `c :- o(W),
# b(W, _).` stops it with a KeyError in _process_grounding. A named variable
# is not projected, so each `_` is given a name of its own, which means the
# same (docs/baselines.md section 6). aspmc's grammar reads only
# `[A-Z][a-zA-Z0-9]*` as a variable, and any other variable (`_V1`, `_X`,
# `A_b`) as a constant: left out of the variables that tell a probabilistic
# rule's groundings apart, `_V1` gave cplint's eruption 0.42 for ProbLog's
# 0.588. So every variable it cannot read is named in that form too.
ASPMC_VARIABLE = re.compile(ASPMC_SKIP + r'|(?P<v>(?<!\w)[A-Z_]\w*)', re.S)
ASPMC_READS = re.compile(r'[A-Z][a-zA-Z0-9]*')


def aspmc_variables(text):
    """The ProbLog program `text` with each variable aspmc's grammar cannot
    read named `V1`, `V2`...: each `_` a name of its own, each other name
    (`_X`, `A_b`) one name wherever it occurs, and a longer prefix where the
    program has a name of that form already; quoted atoms, strings, comments
    and the variables it reads are left as they are."""
    prefix = 'V'
    while re.search(rf'(?<!\w){prefix}\d', text):
        prefix += 'V'
    count = itertools.count(1)
    names = {}

    def name(m):
        v = m.group('v')
        if v is None or ASPMC_READS.fullmatch(v):
            return m.group(0)
        if v == '_':
            return f'{prefix}{next(count)}'
        if v not in names:
            names[v] = f'{prefix}{next(count)}'
        return names[v]
    return ASPMC_VARIABLE.sub(name, text)


# aspmc's ProbLog front end refuses much of what ProbLog reads: lists,
# floats, facts with variables (its grounder clingo's "unsafe variables");
# it refused 109 of the ProbLog tutorial's 122 programs. A
# program it refuses so is one it does not support, `unsupported` with its
# reason (aspmc reads the program first order, and is given no ground
# program in its place). Where it stops with a KeyError reading clingo's
# grounding (problogprogram.py's _process_grounding) on the program
# aspmc_variables writes, it fails on a program its language expresses,
# ProbLog reading the same file: an error, not a refusal. A circuit
# Alea.jl exports (an evaluation of its own, read as the circuit or a
# translation of it) has no first-order source: one aspmc
# refuses is given to it again as the ground program tddlog prints
# (`tddlog --ground`), every predicate and constant renamed to a plain
# name (`aspmc_ground`), the record marked `aspmc_route: ground`; the
# grounding is part of that run's time, and the refused run's is not.
# Records so marked on another instance (before 2026-11-04) read a
# program's ground program.
ASPMC_REFUSED = re.compile(r'lark\.exceptions\.|RuntimeError: (?:parsing failed|grounding stopped because of errors)'
                           r'|error: (?:unsafe variables|syntax error)')


def aspmc_ground(text):
    """(program, names) of a ground ProbLog program as `tddlog --ground`
    prints it: the program with predicate i named `p<i>` and constant j
    `c<j>` (an integer kept), in the syntax aspmc reads, and the original of
    each renamed atom, without blanks, by its new name."""
    from lpad2problog import split_clauses, split_top, strip_comments
    preds, consts, names = {}, {}, {}

    def term(t):
        t = re.sub(r'\s+', '', t)
        return t if re.fullmatch(r'-?\d+', t) else consts.setdefault(t, f'c{len(consts)}')

    def atom(t):
        t = t.strip()
        m = re.fullmatch(r"('(?:[^'\\]|\\.)*'|[^(]+?)\s*(?:\((.*)\))?", t, re.S)
        if not m:
            raise ValueError(f'not a ground atom: {t}')
        args = split_top(m.group(2), ',') if m.group(2) is not None else []
        new = preds.setdefault((m.group(1).strip(), len(args)), f'p{len(preds)}')
        new += f"({','.join(term(a) for a in args)})" if args else ''
        names[new] = re.sub(r'\s+', '', t)
        return new

    def literal(t):
        t = t.strip()
        return '\\+' + atom(t[2:]) if t.startswith('\\+') else atom(t)

    lines = []
    for clause in split_clauses(strip_comments(text)):
        clause = ' '.join(clause.split())
        m = re.fullmatch(r'(query|evidence)\((.*)\)', clause)
        if m and m.group(1) == 'query':
            lines.append(f'query({atom(m.group(2))}).')
            continue
        if m:
            parts = split_top(m.group(2), ',')
            truth = parts[-1].strip() if len(parts) > 1 else 'true'
            lit = (','.join(parts[:-1]) if len(parts) > 1 else parts[0]).strip()
            if lit.startswith('\\+'):
                lit, truth = lit[2:], {'true': 'false', 'false': 'true'}[truth]
            lines.append(f'evidence({atom(lit)},{truth}).')
            continue
        head, body = problog2pita.split_neck(clause)
        parts = split_top(head, ':')
        head = f'{parts[0].strip()}::{atom(parts[2])}' if len(parts) == 3 and parts[1] == '' else atom(head)
        lines.append(f'{head}.' if body is None else f"{head} :- {', '.join(literal(b) for b in split_top(body, ','))}.")
    return '\n'.join(lines) + '\n', names


def run_aspmc(args, name, path, circuit=False):
    """aspmc on the ProbLog program (`aspmc_run`). A program it refuses
    (ASPMC_REFUSED), or that the harness cannot write in its language (its
    evidence as queries, its syntax), is `unsupported`; a circuit is then
    given to it as its ground program
    (`aspmc_ground`), the record marked `aspmc_route: ground`."""
    r = aspmc_run(args, name, path)
    if r['status'] != 'unsupported' and not (r['status'] == 'error' and
                                             ASPMC_REFUSED.search(str(r.get('error', '')))):
        return r
    if not circuit:
        return {**r, 'status': 'unsupported'}
    g = timed([args.tddlog, '--ground', path], args.timeout)
    if g['status'] != 'ok':
        g.pop('stdout', None)
        return {**g, 'stats': {'aspmc_route': 'ground'}}
    try:
        program, names = aspmc_ground(g.pop('stdout'))
    except ValueError as e:
        return {'status': 'error', 'seconds': g['seconds'], 'error': str(e), 'stats': {'aspmc_route': 'ground'}}
    ground = os.path.join(args.dir, 'aspmc-' + name, name + '.ground.pl')
    os.makedirs(os.path.dirname(ground), exist_ok=True)
    write_new(ground, program)
    rargs = argparse.Namespace(**{**vars(args), 'timeout': max(1.0, args.timeout - g['seconds'])})
    # aspmc's form of the ground program, where it needs one, takes the ground program's place, apart
    # from the form of the program it refused (the capture records each)
    r = aspmc_run(rargs, name, ground, form=ground)
    r['seconds'] += g['seconds']
    r.setdefault('stats', {})['aspmc_route'] = 'ground'
    if isinstance(r.get('answers'), dict):
        r['answers'] = {names.get(q, q): p for q, p in r['answers'].items()}
    return r


def aspmc_run(args, name, path, form=None):
    """aspmc on the ProbLog program, with its evidence asked as queries
    (`aspmc_conditioned`) where it has any, and written in the syntax its
    grammar reads where ProbLog's means the same: its probabilities, `is`,
    comments, quoted atoms, comparisons, bodies, the terms it compares and
    its variables (`aspmc_numbers`, `aspmc_is`, `aspmc_comments`,
    `aspmc_quoted`, `aspmc_operators`, `aspmc_bodies`, `aspmc_constants`,
    `aspmc_variables`), and the bodies of
    its probabilistic clauses that compare folded (`aspmc_folded`): that
    form of the program at `form` (default aspmc-NAME/NAME.pl). A program
    those rewrites cannot write is `unsupported`, with the reason. It writes
    `out.cnf` and the decomposer's scratch into the working directory, so
    each instance gets one of its own. A ground query it does not print, an
    atom its grounder removed, answers 1 where the program states it as a
    fact (`bench_pe :- true.`, P(e) where nothing is observed) and 0
    otherwise, as no rule derives it."""
    from lpad2problog import split_clauses, strip_comments
    d = os.path.join(args.dir, 'aspmc-' + name)
    os.makedirs(d, exist_ok=True)
    with open(path) as f:
        text = f.read()
    clauses = split_clauses(strip_comments(text))

    def bare(q):   # as aspmc prints it, a quoted atom that need not be quoted without its quotes
        return re.sub(r"'([a-z][a-zA-Z0-9_]*)'", lambda m: m.group(1) if m.group(1) != 'not' else m.group(0),
                      re.sub(r'\s+', '', q))
    asked = [bare(m.group(1)) for clause in clauses if (m := re.fullmatch(r'\s*query\s*\((.*)\)\s*', clause, re.S))]
    facts = {bare(m.group(1)) for clause in clauses
             if (m := re.fullmatch(r'\s*([a-z]\w*(?:\(.*\))?)\s*(?::-\s*true\s*)?', clause, re.S))}
    try:
        conditioned = aspmc_conditioned(text)
        program, quoted = aspmc_quoted(aspmc_is(aspmc_numbers(aspmc_comments(text) if conditioned is None
                                                              else conditioned)))
        program = aspmc_folded(aspmc_variables(aspmc_constants(aspmc_bodies(aspmc_operators(program)))))
    except ValueError as e:
        # the harness's rewrites cannot write it in aspmc's language: a program it does not support
        return {'status': 'unsupported', 'seconds': 0.0, 'error': f'translator: {e}'}
    if program != text:
        path = form or os.path.join(d, name + '.pl')
        write_new(path, program)
    python = os.path.abspath(args.aspmc) if os.sep in args.aspmc else args.aspmc
    r = timed([python, '-u', '-m', 'aspmc.main', '-m', 'problog', '-c', '-v', 'result',
               os.path.abspath(path)], args.timeout, cwd=d)
    if r['status'] == 'ok':
        r['answers'] = parse_aspmc(r['stderr'], conditioned is not None)
        if isinstance(r['answers'], dict):
            r['answers'] = {aspmc_unquoted(q, quoted): p for q, p in r['answers'].items()}
            for q in asked:
                if not re.search(r'[(,][A-Z_]', q):
                    r['answers'].setdefault(q, 1.0 if q in facts else 0.0)
    return r


# Scallop's top-k proofs (`-p topkproofs -k K`) keep the K most probable
# proofs of each tuple and count the weighted models of that DNF exactly: an
# SDD built from it (docs/baselines.md section 3). At K = usize::MAX, the
# largest its command line takes, no proof is ever dropped: the truncation
# after a disjunction or a conjunction keeps every clause, and a negation
# expands the whole product of its CNF. The answer is then exact, and the
# harness runs Scallop so alone, as the variant `allproofs` (k unbounded).
SCALLOP_K = 2 ** 64 - 1
SCALLOP_VARIANT = 'allproofs'


def scallop_variant(system, variant):
    """A record's or a call's variant, Scallop's top-k at SCALLOP_K named
    SCALLOP_VARIANT: a run before the variant had its name wrote the same
    command as topk followed by K."""
    return SCALLOP_VARIANT if system == 'scallop' and variant == f'topk{SCALLOP_K}' else variant


def holds(specs, what):
    """The holds TAG=SYSTEM[@INSTANCES][!] of the processing (rerun.py
    --system-hold, capmerge.py --hold): TAG's records or calls of SYSTEM on the
    instances the regular expression INSTANCES matches whole (every instance
    without it) are left out, so that an earlier run's stay in their place.
    A hold marked `!` may leave nothing in place, a withdrawal: an instance no
    earlier run gave SYSTEM (its record withdrawn until a later run gives it
    one, its call with it). `what` names the option in the error a spec of
    another form stops with. A hold is (TAG, SYSTEM, INSTANCES, marked, spec)."""
    out = []
    for spec in specs:
        tag, eq, rest = spec.partition('=')
        marked = rest.endswith('!')
        system, at, pattern = rest.removesuffix('!').partition('@')
        if not eq or not tag or not system or (at and not pattern):
            raise SystemExit(f'{what} {spec}: not TAG=SYSTEM[@INSTANCES][!]')
        out.append((tag, system, re.compile(pattern if at else '.*'), marked, spec))
    return out


def held(hold_list, tag, system, instance):
    """The spec of the first hold of `hold_list` (holds) on TAG's record or call of
    `system` on `instance`, or None."""
    return next((spec for t, s, pattern, _, spec in hold_list
                 if t == tag and s == system and pattern.fullmatch(instance)), None)


# A Scallop record's route: Scallop reads a program first order, as
# problog2scallop.py writes it from the program the ProbLog family reads
# (`first-order`). A circuit Alea.jl exports has no first-order source (an
# evaluation of its own, read as the circuit or a translation of it):
# Scallop reads the ground program tddlog prints of it, a relation per gate
# (ground2scallop.py; `ground`), and its seconds
# hold the grounding's. Scallop's records without the field (before
# 2026-11-04) read the ground program on every instance.
SCALLOP_ROUTE = 'first-order'
SCALLOP_CIRCUIT_ROUTE = 'ground'


def run_scallop(args, name, path, variant, circuit=False):
    """Scallop on the program, translated first order into Scallop
    (`problog2scallop.py`, the harness's translator, whose time is not
    charged), or on a circuit's ground program (`run_scallop_ground`),
    under the top-k-proofs provenance at k = SCALLOP_K, which is exact. A
    program the translation refuses is `unsupported`, with its reason:
    Scallop is given no ground program in its place."""
    if variant != SCALLOP_VARIANT:
        raise SystemExit(f'unknown Scallop variant {variant}')
    if circuit:
        return run_scallop_ground(args, name, path)
    stats = {'scallop_route': SCALLOP_ROUTE}
    scl_path = os.path.join(args.dir, name + '.scl')
    err = translated(args, 'problog2scallop.py', [path], scl_path, 'scallop')
    if err:
        return err | {'stats': stats}
    r = timed([args.scallop, '-p', 'topkproofs', '-k', str(SCALLOP_K), scl_path], args.timeout)
    if r['status'] == 'ok':
        with open(scl_path) as f:
            head = ''.join(itertools.takewhile(lambda line: line.startswith('//'), f))
        try:
            r['answers'] = problog2scallop.answers(r['stdout'], head)
        except ValueError as x:
            r = {'status': 'error', 'seconds': r['seconds'], 'error': str(x)}
    return r | {'stats': stats}


def run_scallop_ground(args, name, path):
    """Scallop on a circuit's ground program, as tddlog prints it, in
    Scallop's relations (`ground2scallop.py`). The seconds are the
    grounding's and Scallop's together."""
    stats = {'scallop_route': SCALLOP_CIRCUIT_ROUTE}
    g = timed([args.tddlog, '--ground', path], args.timeout)
    if g['status'] != 'ok':
        g.pop('stdout', None)
        return g | {'stats': stats}
    scl = ground2scallop.translate(g['stdout'])
    scl_path = os.path.join(args.dir, name + '.scl')
    write_new(scl_path, scl)
    r = timed([args.scallop, '-p', 'topkproofs', '-k', str(SCALLOP_K), scl_path],
              max(1.0, args.timeout - g['seconds']))
    r['seconds'] += g['seconds']
    if r['status'] == 'ok':
        r['answers'] = ground2scallop.answers(r['stdout'], scl)
    return r | {'stats': stats}


# AceCompile's options as its own usage prints them, with whether each takes
# an argument. The readme spells two of them otherwise; Ace refuses those
# spellings.
ACE_OPTIONS = {'forceC2d': False, 'forceTabular': False, 'd02': False, 'sbk05': False, 'cd05': False,
               'cd06': False, 'dtBnOrder': False, 'dtClauseMinfill': False, 'dtHypergraph': True,
               'noEclause': False, 'Xseed': True, 'Xlog': False}
ACE_README_NAMES = {'noTabular': 'forceC2d', 'dtBnMinfill': 'dtBnOrder'}


# Checked on the UAI MAR and PR sets (docs/baselines.md, section 7): a c2d
# circuit evaluated in log space gives log_e(Pr(e)) = -inf where P(e) is
# 1.6e-2, or fails in the evaluator, so such a run is a failed run.
ACE_XLOG_C2D = "Ace chose c2d, whose circuit it evaluates wrongly in log space (-Xlog)"


def ace_xlog_c2d(variant, method):
    """Whether an Ace run of `variant` whose compile chose `method` (its
    record's `ace_method`) is one whose answer is not to be read: log space
    on the c2d route (ACE_XLOG_C2D)."""
    return 'Xlog' in variant.split('+') and method == 'c2dCompilation'


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


def query_cells(network, qs):
    """The variable and the value index whose marginal answers each query
    atom `x(v)` of a gen_bn.py program, whose names are the network's
    through `gen_bn.ident`."""
    order, variables, _ = network
    by_ident = {gen_bn.ident(v): v for v in order}
    cells = {}
    for q in qs:
        m = re.fullmatch(r'(\w+)\((\w+)\)', q.replace(' ', ''))
        v = by_ident[m.group(1)]
        cells[q.replace(' ', '')] = (v, [gen_bn.ident(u) for u in variables[v]].index(m.group(2)))
    return cells


def network_answers(rows, cells):
    """The answers to the queries from a baseline's marginals by variable,
    through the queries' cells (`query_cells`, `ground_cells`)."""
    return {q: rows[v][i] for q, (v, i) in cells.items()}


def run_ace(args, name, path, variant):
    """Ace 3.0 on the network file the program was generated from, with the
    program's observations from the `.inst` file `gen` writes beside it, or
    from the instance's own (an evidence set of Ace's archive, which tddlog
    reads too: `ace_observations`). The network file is copied (or unzipped) into a scratch directory of the instance,
    since Ace writes the circuit and the marginals next to it, and the copy
    of a Hugin file is written as Ace's reader takes it (`hugin_for_ace`).
    Ace reads a UAI file only as a Bayesian network, and a BIF file (Roulette's
    networks) not at all (its reader's classes are not in the jar), so a UAI
    or BIF network is given to it as the network gen_bn.py makes of it,
    written as a Hugin file (`gen_bn.write_hugin`): a Markov network's
    factors are variables observed in the `.inst`, as they are in the
    program.

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
    if instances(args)[name][1][0] != 'gen_bn.py':
        g = GROUNDED[(name, 'hugin')]
        inst, cells = g['stem'] + '.inst', g['cells']
        net, data = name + '.net', gen_bn.read_bytes(g['stem'] + '.net')
        with open(inst) as f:
            evidence = re.findall(r'^<inst ', f.read(), re.M)
    else:
        source = instances(args)[name][1][1]
        inst = ace_observations(args, name, instances(args)[name][1])
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
        cells = query_cells(network, queries(path))
        net = os.path.basename(source).removesuffix('.gz')
        data = gen_bn.read_bytes(source)
        if net.endswith(('.uai', '.bif')):
            net, data = os.path.splitext(net)[0] + '.net', gen_bn.write_hugin(network).encode()
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
    if net.endswith(('.net', '.hugin')):
        data = hugin_for_ace(data)
    with open(os.path.join(d, net), 'wb') as f:
        f.write(data)
    ace = os.path.abspath(args.ace)
    # Ace's unit resolution sorts with a comparator that is not a total
    # order, which the sort of Java 7 and later refuses (munin with its
    # evidence); the property restores the merge sort of the Java it was
    # written for. The JVM's scratch goes to the instance's directory.
    stack = getattr(args, 'ace_stack', None)
    java = [args.ace_java, f'-Xmx{args.ace_heap}', *([f'-Xss{stack}'] if stack else []),
            '-Djava.util.Arrays.useLegacyMergeSort=true',
            f'-Djava.io.tmpdir={d}', f'-DACEC2D={ace}/c2d_linux',
            '-classpath', ':'.join(f'{ace}/{jar}' for jar in ['ace.jar', 'inflib.jar', 'jdom.jar'])]
    env = dict(os.environ, LD_LIBRARY_PATH=ace)
    try:
        c = timed(java + ['edu.ucla.belief.ace.AceCompile', '-Xnormalize'] + ace_flags(variant)
                  + (['-e', inst] if evidence else []) + [net], args.timeout, cwd=d, env=env)
        stats = {'compile_s': round(c['seconds'], 3), 'ace_heap': args.ace_heap,
                 **({'ace_stack': stack} if stack else {})}
        if c['status'] != 'ok':
            return {**c, 'stats': stats}
        for key, pattern in [('ace_method', r'^Method : (\S+)'), ('ace_compile_total_s', r'^Total Time \(s\) : (\S+)')]:
            m = re.search(pattern, c['stdout'], re.M)
            if m:
                stats[key] = m.group(1) if key == 'ace_method' else float(m.group(1))
        if ace_xlog_c2d(variant, stats.get('ace_method')):
            # the circuit is not evaluated and the run fails
            return {'status': 'error', 'seconds': c['seconds'], 'stats': stats, 'error': ACE_XLOG_C2D}
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
            answers = 'inconsistent' if log10_pe == -math.inf else network_answers(rows, cells)
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


def uai_inputs(args, name, path):
    """The variables of an instance's network in order, the UAI file the
    baselines that read UAI files run on, its evidence file, and the
    queries' cells: for a gen_bn.py instance the file the program was
    generated from, or for another format the UAI copy and observations
    `gen` writes beside the program; for another program the network of its
    ground program (`ground_route`). The variables are None where the
    observations are missing."""
    gargs = instances(args)[name][1]
    if gargs[0] != 'gen_bn.py':
        g = GROUNDED[(name, 'uai')]
        return g['order'], g['stem'] + '.uai', g['stem'] + '.uai.evid', g['cells']
    network = gen_bn.parse(gargs[1])
    if gargs[1].endswith('.uai'):
        network = network[0]
        return network[0], os.path.abspath(gargs[1]), os.path.abspath(gargs[gargs.index('--evid') + 1]), \
            query_cells(network, queries(path))
    model = os.path.abspath(os.path.join(args.dir, name + '.uai'))
    evid = model + '.evid'
    return (network[0] if os.path.exists(evid) else None), model, evid, query_cells(network, queries(path))


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
    order, model, evid, cells = uai_inputs(args, name, path)
    if order is None:
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
                network_answers(dict(zip(order, rows)), cells)
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
# A driver timed warm (TensorInference.jl's): its record's stats and the driver's lines they come from
UAI_WARM = (('inproc_seconds', 'inference_seconds'), ('jit_seconds', 'compile_seconds'),
            ('warm_seconds', 'warm_seconds'), ('warmup_seconds', 'warmup_seconds'))


def run_uai_system(args, name, path, system, variant):
    """pyAgrum (junction tree), pgmpy (variable elimination), libDAI (junction
    tree) or TensorInference.jl (tensor network contraction) on the UAI file
    of a gen_bn.py instance (`uai_inputs`), the observations applied by
    slicing the tables. The marginals asked are the program's query atoms
    (what the others are asked), and the `-pr` variant gives
    `log10_z`, the natural logarithm the driver writes over ln 10. The
    record keeps the seconds of the inference proper, without the start of
    the interpreter and the libraries. TensorInference.jl's driver times it
    as alea_bench.jl times Alea.jl (own_seconds): after a warm-up on a small
    model (`warmup_seconds`), the first solve (`inproc_seconds`, Julia's
    compilation within it `jit_seconds`) and, where that took under a third
    of the limit, a second (`warm_seconds`)."""
    order, model, evid, cells = uai_inputs(args, name, path)
    if order is None:
        return {'status': 'error', 'seconds': 0.0, 'error': f'no {evid}: rerun bench.py gen'}
    task = 'PR' if variant.endswith('-pr') else 'MAR'
    variant = variant.removesuffix('-pr')
    index = {v: i for i, v in enumerate(order)}
    if task != 'MAR':
        cells = {}
    qvars = sorted({index[v] for v, _ in cells.values()})
    d = os.path.abspath(os.path.join(args.dir, f'{system}-{name}'))
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d)
    try:
        out = os.path.join(d, 'out')
        # the query variables in a file (`@FILE`): on the largest networks
        # the list is longer than Linux takes for one argument (E2BIG)
        cmd = UAI_SYSTEMS[system][1](args) + [model, evid, out, task]
        if qvars:
            with open(os.path.join(d, 'vars'), 'w') as f:
                f.write(','.join(map(str, qvars)))
            cmd.append('@' + os.path.join(d, 'vars'))
        env = dict(os.environ, OMP_NUM_THREADS='1', OPENBLAS_NUM_THREADS='1', MKL_NUM_THREADS='1', BN_VARIANT=variant)
        if system in OWN_TIME:   # its second solve, where the first took under a third of the limit (as Alea.jl's)
            env['BN_WARM_BELOW'] = f'{args.timeout / 3:g}'
        r = timed(cmd, args.timeout, cwd=d, env=env)
        if r['status'] != 'ok':
            return r
        try:
            with open(out) as f:
                text = f.read()
            line = lambda key: re.search(rf'^{key} ([0-9.eE+-]+)$', text, re.M)
            if system in OWN_TIME:
                # timed warm as Alea.jl is (own_seconds): the first solve after the warm-up on a
                # small model, Julia's compilation within it, the second solve where there was one
                stats = {stat: float(m.group(1)) for stat, key in UAI_WARM if (m := line(key))}
            else:
                stats = {f'{system}_inference_seconds': float(m.group(1))} if (m := line('inference_seconds')) else {}
            if task == 'PR':
                return {'status': 'ok', 'seconds': r['seconds'], 'log10_z': parse_merlin_pr(text), 'stats': stats}
            rows = parse_merlin_marginals(text)
            answers = 'inconsistent' if rows is None else \
                network_answers({order[i]: row for i, row in zip(qvars, rows)}, cells)
        except (OSError, ValueError, IndexError, KeyError, AttributeError) as x:
            return {'status': 'error', 'seconds': r['seconds'], 'error': f'{system}: {type(x).__name__} {x}'}
        return {'status': 'ok', 'seconds': r['seconds'], 'answers': answers, 'stats': stats}
    finally:
        shutil.rmtree(d, ignore_errors=True)


# The systems that read networks or Dice programs are given a program that
# is neither through its ground program (every system tries every
# problem): `tddlog --ground` prints it, and ground2net.py writes it as a
# Bayesian network of Boolean variables (UAI for Merlin and the UAI
# systems, Hugin for Ace). Both steps are timed and their seconds added to
# the system's, as Scallop's grounding is. The systems that read Dice
# programs are given the logic program as problog2ppl.py writes it in Dice
# instead (`dice_route`): over the tuples of its relations, a recursion over
# a step a function `iterate`d, as a Dice user writes it. That translation
# is the harness's and is not charged, as problog2ppl.py's others are not
# (`lp_route`). Where problog2ppl.py refuses the program, they are given the
# Dice programs ground2net.py writes of its ground program, charged as above.
NETWORK_SYSTEMS = ('ace', 'merlin', *UAI_SYSTEMS)
DICE_FORMS = ('dice', 'dicemap')   # the forms of ground_format that dice_route writes
GROUNDED = {}   # (instance, format) -> the ground route's record, once a run


def ground_dice(args, name):
    """Whether the systems that read Dice programs are given instance `name`
    through its ground program: it has no Dice program of its own
    (`dice_source`) and is not a circuit, and the run does not ask log10 Z
    (--pr), which a ground program does not state."""
    return not getattr(args, 'pr', False) and grounds_dice(instances(args)[name][1])


def grounds_dice(gargs):
    """`ground_dice` on an instance whose generator arguments are `gargs`,
    outside --pr: not where a Dice program of its own is beside it
    (writes_dice), nor on Circulette's, which Roulette reads as its netlist
    and the Dice family is not given (docs/baselines.md, "Circulette")."""
    return not writes_dice(gargs) and gargs[0] not in ('dice2problog.py', 'gen_bn.py', 'alea_bench.jl', 'gen_circulette.py')


def ground_format(args, name, system, variant):
    """The format of the ground route `system` takes on instance `name`:
    'hugin' (Ace) or 'uai' (Merlin, the UAI systems) on an instance that is
    not a gen_bn.py network, 'dicemap' (Dice and noDice, a program a query)
    or 'dice' (the other systems that read Dice programs, one program, and
    Storm and PRISM, its Markov chain) where `ground_dice`; None otherwise,
    and under --pr, where the systems are asked log10 Z, which a ground
    program does not state. Not for a system that reads a program of its own
    (OWN_PROGRAMS), nor for one that reads the logic program written in its
    language (`lp_route`)."""
    return ground_form(instances(args)[name][1], system, variant, getattr(args, 'pr', False))


def ground_form(gargs, system, variant, pr=False):
    """`ground_format` on an instance whose generator arguments are `gargs`
    (the capture's, for the processing), in a run that asks log10 Z if
    `pr`."""
    gen = gargs[0]
    if pr or variant.endswith('-pr') or system in OWN_PROGRAMS.get(gen, ()) or lp_route(gargs, system):
        return None
    if system in NETWORK_SYSTEMS and gen != 'gen_bn.py':
        return 'hugin' if system == 'ace' else 'uai'
    if (system in DICE_SYSTEMS or system in MODEL_CHECKERS) and grounds_dice(gargs):
        return 'dicemap' if system in ('dice', 'nodice') else 'dice'
    return None


def takes_networks(args, spec, names, opt):
    """Fails fast where network system `opt` would run on none of `names`:
    it answers every instance (a gen_bn.py network as it is, another
    through its ground program), but under --pr only the networks."""
    if not names or (args.pr and not any(spec[n][1][0] == 'gen_bn.py' for n in names)):
        raise SystemExit(f'--{opt} answers log10 Z of the networks gen_bn.py writes under --pr, and none is selected'
                         if names else f'--{opt}: no instance is selected')


def ground_dir(args, name):
    return os.path.abspath(os.path.join(args.dir, f'ground-{name}'))


def ground_route(args, name, path, form):
    """The ground route of instance `name` in format `form` (`ground_format`),
    as a record: its status, its seconds (`tddlog --ground` and
    ground2net.py together), and where it went well the ground program
    (`ground`, whose queries the Dice systems' answers follow), the stem of
    the files ground2net.py wrote (`stem`), the network's variables in
    order (`order`) and the queries' cells (`cells`). A translation
    ground2net.py refuses (a component with negation through recursion, a
    network too large) is `unsupported`. A Dice format is problog2ppl.py's
    first (`dice_route`), and ground2net.py's only where that refuses the
    program. Run once a run per instance and format; the instance's
    directory is removed once its systems have run (`drop_ground`)."""
    key = (name, form)
    if key in GROUNDED:
        return GROUNDED[key]
    d = ground_dir(args, name)
    os.makedirs(d, exist_ok=True)
    if form in DICE_FORMS:
        rec = dice_route(args, name, path, form)
        if rec is not None:
            GROUNDED[key] = rec
            return rec
    ground = os.path.join(d, name + '.ground.pl')
    g = GROUNDED.get((name, 'ground'))
    if g is None:
        g = GROUNDED[(name, 'ground')] = timed([args.tddlog, '--ground', path], args.timeout)
        if g['status'] == 'ok':
            write_new(ground, g.pop('stdout'))
    if g['status'] != 'ok':
        rec = {k: g[k] for k in ('status', 'seconds', 'error') if k in g}
    else:
        stem = os.path.join(d, f'{name}.{form}')
        t = timed([sys.executable, os.path.join(HERE, 'ground2net.py'), ground, stem, '--format', form],
                  max(1.0, args.timeout - g['seconds']))
        rec = {'status': t['status'], 'seconds': g['seconds'] + t['seconds']}
        if t['status'] == 'error':
            rec.update(status='unsupported' if t.get('returncode') == REFUSED else 'error', error=t['error'])
        elif t['status'] == 'ok':
            with open(stem + '.json') as f:
                net = json.load(f)
            rec.update(ground=ground, stem=stem, order=net['order'],
                       cells={q: (v, ['false', 'true'].index(u)) for q, (v, u) in net['queries'].items()})
    GROUNDED[key] = rec
    return rec


def dice_route(args, name, path, form):
    """The Dice programs problog2ppl.py writes of logic program `path` in
    format `form` (DICE_FORMS: one program, NAME.lp.dice, or a program a
    query listed in NAME.lp.dicemap), as ground_route's record: the
    harness's translation, so its seconds are 0; `ground` a file of the
    queries in the order the programs return them, which the systems'
    answers follow; `stem`. None where problog2ppl.py refuses the program,
    which then takes the ground program's route. A translation past its
    bound is the harness's error (translation)."""
    os.makedirs(ground_dir(args, name), exist_ok=True)
    stem = os.path.join(ground_dir(args, name), f'{name}.lp')
    out, err = translation(args, 'problog2ppl.py', [form, path, f'{stem}.{form}'])
    if err is not None:
        return None if err['status'] == 'unsupported' else err
    asked = os.path.join(ground_dir(args, name), f'{name}.lp.queries.pl')
    write_new(asked, ''.join(f'query({q}).\n' for q in out.splitlines()))
    return {'status': 'ok', 'seconds': 0.0, 'ground': asked, 'stem': stem}


def drop_ground(args, name):
    """Forgets instance `name`'s ground routes and removes their files."""
    for key in [k for k in list(GROUNDED) if k[0] == name]:
        del GROUNDED[key]
    shutil.rmtree(ground_dir(args, name), ignore_errors=True)


# --- Logic programs written over their facts ------------------------------------
# Alea.jl, Roulette, WebPPL, Psi, Pluck and SPPL are given a logic program (an instance the
# other systems that read Dice programs are given through its ground
# program, `grounds_dice`) as problog2ppl.py writes it in their language,
# first order: a relation a table over its facts, a rule a loop over the
# relations its body joins, a recursive component a loop of stages, the
# facts a file the program reads (in Psi's, Pluck's and SPPL's program, run_lp_psi,
# run_lp_pluck and run_lp_sppl).
# Alea.jl+tddlog and Roulette+tddlog read
# the same program as Alea.jl and Roulette, so that each differs from its
# system only in the inference. The translation is the harness's,
# untimed and bounded by translation_bound; a program it cannot write is
# `unsupported`.
LP_SYSTEMS = {'alea': 'alea', 'alea-tddlog': 'alea', 'roulette': 'roulette', 'roulette-tddlog': 'roulette',
              'webppl': 'webppl', 'psi': 'psi', 'pluck': 'pluck', 'sppl': 'sppl'}
# the languages whose program holds its facts (problog2ppl.py's INLINE): no facts file
LP_INLINE = ('psi', 'pluck', 'sppl')
LP_EXT = {'alea': 'jl', 'roulette': 'rkt', 'webppl': 'wppl', 'psi': 'psi', 'pluck': 'pluck', 'sppl': 'sppl'}


def lp_route(gargs, system, pr=False):
    """Whether `system` is given the instance whose generator arguments are
    `gargs` as problog2ppl.py writes it, outside a run that asks log10 Z:
    not where it reads a program of its own (OWN_PROGRAMS, and Alea.jl's
    of ALEA_OF for Alea.jl and Alea.jl+tddlog)."""
    return (system in LP_SYSTEMS and not pr and grounds_dice(gargs) and system not in OWN_PROGRAMS.get(gargs[0], ())
            and not (LP_SYSTEMS[system] == 'alea' and tuple(gargs) in ALEA_OF))


def lp_named(qs, comps):
    """The answers to problog2ppl.py's queries `qs` from the distributions of
    the program's components, the K-th the K-th query's ({'true': P, ...});
    ValueError where they do not line up."""
    if len(comps) != len(qs) or any('true' not in c for c in comps):
        raise ValueError(f'{len(comps)} Boolean distributions for {len(qs)} queries')
    return {re.sub(r'\s+', '', q): c['true'] for q, c in zip(qs, comps)}


def alea_bools(answers):
    """The components of an Alea.jl program whose value is a tuple of
    Booleans, from alea_bench.jl's answers `qK`, in order."""
    if any(not re.fullmatch(r'q\d+', a) for a in answers):
        raise ValueError(f'components that are not Booleans: {sorted(answers)[:4]}')
    return [{'true': answers[f'q{k}']} if f'q{k}' in answers else {} for k in range(1, len(answers) + 1)]


def run_lp(args, name, path, system):
    """SYSTEM (of LP_SYSTEMS) on logic program `path`, written by
    problog2ppl.py in its language with its facts file, which the program
    reads: Alea.jl's program by alea_bench.jl (run_alea_program, the facts
    its `--file`), Alea.jl+tddlog's export of the same program
    (run_alea_tddlog), Roulette's module timed by roulette_timer.rkt
    (`roulette_timed`) on the facts, Roulette+tddlog's engine on the same
    (run_roulette_tddlog), WebPPL's program run by webppl_dice.js, Psi's
    (run_lp_psi), which holds its facts, by `psi --dp`, Pluck's by pluck_dice.jl
    (run_lp_pluck), SPPL's by sppl_dice.py (run_lp_sppl). The
    answers are keyed as the queries problog2ppl.py lists; the stats as
    those of the system's own programs. The program and its facts file are
    removed after the run (scratch)."""
    lang = LP_SYSTEMS[system]
    prog = os.path.abspath(os.path.join(args.dir, f'{name}.lp.{lang}.{LP_EXT[lang]}'))
    facts = os.path.abspath(os.path.join(args.dir, f'{name}.lp.{lang}.facts'))
    with scratch(prog, facts):
        out, err = translation(args, 'problog2ppl.py', [lang, path, prog] + ([] if lang in LP_INLINE else [facts]))
        if err:
            return err
        qs = out.splitlines()
        if system == 'psi':
            return run_lp_psi(args, name, prog, qs)
        if system == 'pluck':
            return run_lp_pluck(args, prog, qs)
        if system == 'sppl':
            return run_lp_sppl(args, prog, qs)
        if system == 'alea-tddlog':
            return run_alea_tddlog(args, name, path, ([prog, ('FACTS', facts)], qs))
        if system == 'roulette-tddlog':
            return run_roulette_tddlog(args, name, path, (prog, [facts], qs))
        if system == 'alea':
            r = run_alea_program(args, [prog, ('FACTS', facts)])
            if r['status'] != 'ok':
                return r
            try:
                comps = alea_bools(r['answers'])
            except ValueError as e:
                return {'status': 'error', 'seconds': r['seconds'], 'error': f'alea: {e}'}
            stats = r['stats']
        elif system == 'roulette':
            r = roulette_timed(args, prog, [facts], args.timeout)
            if r['status'] != 'ok':
                return r
            comps, stats = parse_alea(r['stdout'], 'roulette')[0], {'inproc_seconds': r['inproc_seconds']}
        else:
            r = timed(program_command(args, 'webppl', os.path.join(HERE, 'webppl_dice.js')) + [prog, facts],
                      args.timeout)
            if r['status'] != 'ok':
                return r
            comps, stats = webppl_output(r['stdout'])
    try:
        answers = lp_named(qs, comps)
    except ValueError as e:
        # (run_alea_program keeps what Alea.jl printed in its stats, not as stdout)
        shown = r.get('stdout') or '\n'.join(stats.get('printed', []))
        return {'status': 'error', 'seconds': r['seconds'], 'error': f'{system}: {e}: ' + shown.strip()[-300:]}
    return {'status': 'ok', 'seconds': r['seconds'], 'stats': stats, 'answers': answers}


def run_lp_psi(args, name, prog, qs):
    """Psi on the program problog2ppl.py wrote at `prog`, which holds its
    facts and returns the array of the queries' holdings: one run a query,
    its `return queries;` made `return queries[K];` (NAME.lp.psiK.psi), as
    Psi is asked its own programs' components (run_psi_own). Psi runs its
    exact back end for discrete programs, `--dp`, which the command gains
    where it lacks it: its symbolic back end does not get through the
    stages' `while` loop. A query answers with the mass of 1. The seconds
    add up."""
    with open(prog) as f:
        text = f.read()
    if text.count('return queries;') != 1:
        return {'status': 'error', 'seconds': 0, 'error': 'psi: no `return queries;` to index'}
    total, comps = 0.0, []
    for k in range(len(qs)):
        file = os.path.abspath(os.path.join(args.dir, f'{name}.lp.psi{k}.psi'))
        cmd = program_command(args, 'psi', file)
        if '--dp' not in cmd:
            cmd.insert(cmd.index(file), '--dp')
        with scratch(file):
            write_new(file, text.replace('return queries;', f'return queries[{k}];'))
            r = timed(cmd, max(0.0, args.timeout - total))
        total += r['seconds']
        if r['status'] != 'ok':
            return {**r, 'seconds': total}
        try:
            dist = parse_psi(r['stdout'])
        except ValueError as x:
            return {'status': 'error', 'seconds': total, 'error': str(x)}
        if not dist or not set(dist) <= {0, 1}:
            return {'status': 'error', 'seconds': total,
                    'error': 'psi: no Boolean distribution: ' + r['stdout'].strip()[-300:]}
        comps.append({'true': dist.get(1, 0.0), 'false': dist.get(0, 0.0)})
    return {'status': 'ok', 'seconds': total, 'stats': {}, 'answers': lp_named(qs, comps)}


def run_lp_pluck(args, prog, qs):
    """Pluck on the program problog2ppl.py wrote at `prog`, which holds its
    facts and asks the K-th query as its query cK: pluck_dice.jl runs it
    once (one Julia start for all the queries), as on the Dice programs
    (pluck_masses), a query's distribution read by parse_pluck."""
    r = timed(program_command(args, 'pluck', os.path.join(HERE, 'pluck_dice.jl')) + [prog], args.timeout)
    if r['status'] != 'ok':
        return r
    dists, comps = parse_pluck(r['stdout']), []
    for k in range(len(qs)):
        got = dists.get(k)
        why = ('no distribution printed' if got is None else 'the evidence is impossible as printed' if not got
               else f'values {sorted(got)} of a Boolean' if not set(got) <= {0, 1} else None)
        if why:
            return {'status': 'error', 'seconds': r['seconds'],
                    'error': f'pluck: query {k}: {why}: ' + r['stdout'].strip()[-300:]}
        comps.append({'true': got.get(1, 0.0), 'false': got.get(0, 0.0)})
    return {'status': 'ok', 'seconds': r['seconds'], 'stats': pluck_stats(r['stdout']), 'answers': lp_named(qs, comps)}


def run_lp_sppl(args, prog, qs):
    """SPPL on the program problog2ppl.py wrote at `prog`, which holds its
    facts: sppl_dice.py compiles it once and asks each query, the K-th of
    the events its module's `queries_()` returns (its `# queries` line), of
    the model given the evidence (sppl_run)."""
    r, comps, stats = sppl_run(args, prog, args.timeout)
    if r['status'] != 'ok':
        return r
    try:
        answers = lp_named(qs, comps)
    except ValueError as e:
        return {'status': 'error', 'seconds': r['seconds'], 'error': f'sppl: {e}: ' + r['stdout'].strip()[-300:]}
    return {'status': 'ok', 'seconds': r['seconds'], 'stats': stats, 'answers': answers}


class ProvSQLServer:
    """A PostgreSQL cluster of its own with ProvSQL loaded, in `d`, on a
    unix socket (no port), its processes pinned to the lane's core as the
    timed commands are (they inherit the affinity). The postmaster runs in
    the foreground, in a session of its own and, under the memory cap, in a
    cgroup of its own, because ProvSQL's backend starts compilers (d4) that
    outlive the client a timeout killed: `stop` kills the session and the
    cgroup and fails if any process is left. The server's temporary files
    (`TMPDIR`, which ProvSQL's compiler directories follow) are in `d`; the
    run stops when the root filesystem has less than ROOT_FREE_GB free."""
    ROOT_FREE_GB = 2

    def __init__(self, prefix, d):
        self.bin = os.path.join(os.path.abspath(prefix), 'pg', 'bin')
        self.d = d
        self.tmp = os.path.join(d, 'tmp')
        os.makedirs(self.tmp)
        # a unix socket's path is at most 107 bytes: the socket lives in a short directory of its own
        self.sock = tempfile.mkdtemp(prefix='ps', dir='/tmp')
        cpu = getattr(LANE, 'cpu', None)
        self.pin = ['taskset', '-c', str(cpu)] if cpu is not None else []
        self.proc = self.cap = None
        self.t0 = time.time()

    def run(self, cmd, **kw):
        r = subprocess.run(self.pin + cmd, capture_output=True, text=True, **kw)
        if r.returncode:
            raise RuntimeError((r.stderr or r.stdout).strip()[-400:])
        return r

    def start(self):
        free = shutil.disk_usage('/').free
        if free < self.ROOT_FREE_GB << 30:
            raise SystemExit(f'ProvSQL: the root filesystem has {free >> 20} MB free, less than {self.ROOT_FREE_GB} GB')
        data = os.path.join(self.d, 'data')
        self.run([os.path.join(self.bin, 'initdb'), '-D', data, '-A', 'trust'])
        with open(os.path.join(data, 'postgresql.conf'), 'a') as f:
            f.write(f"shared_preload_libraries = 'provsql'\nlisten_addresses = ''\nunix_socket_directories = '{self.sock}'\n"
                    'fsync = off\nsynchronous_commit = off\nfull_page_writes = off\nautovacuum = off\n'
                    'max_wal_size = 4GB\nshared_buffers = 512MB\n')
        cmd = [os.path.join(self.bin, 'postgres'), '-D', data]
        if MEMCAP:
            self.cap = memcap.Cap(*MEMCAP)
            cmd = self.cap.command(cmd)
        self.proc = subprocess.Popen(self.pin + cmd, stdout=open(os.path.join(self.d, 'server.log'), 'w'),
                                     stderr=subprocess.STDOUT, start_new_session=True,
                                     env={**os.environ, 'TMPDIR': self.tmp, 'PROVSQL_SERVER': self.d})
        self.psql_cmd = [os.path.join(self.bin, 'psql'), '-h', self.sock, '-d', 'postgres', '-X']
        ready = [os.path.join(self.bin, 'pg_isready'), '-h', self.sock, '-q']
        for _ in range(600):
            if self.proc.poll() is not None:
                raise RuntimeError('postgres exited: ' + open(os.path.join(self.d, 'server.log')).read()[-300:])
            if subprocess.run(ready).returncode == 0:
                break
            time.sleep(0.1)
        else:
            raise RuntimeError('postgres did not accept connections within 60 s')
        self.run(self.psql_cmd + ['-q', '-v', 'ON_ERROR_STOP=1', '-c', 'CREATE EXTENSION provsql CASCADE'])
        self.run(self.psql_cmd + ['-q', '-v', 'ON_ERROR_STOP=1', '-c', 'SELECT provsql.setup_search_path()'])   # for new sessions

    @staticmethod
    def marked(pid, mark):
        try:
            with open(f'/proc/{pid}/environ', 'rb') as f:
                return mark in f.read().split(b'\0')
        except OSError:
            return False

    def stop(self):
        """Kills the server and what it started, and checks that nothing of
        the run is left (a process or a /tmp/provsql* directory)."""
        left = []
        if self.proc is not None:
            try:
                os.killpg(self.proc.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            self.proc.wait()
            for _ in range(36):   # freeing a server of many GB takes longer than memcap's 5 s
                if not self.cap:
                    break
                try:
                    self.cap.finish()   # cgroup.kill
                    break
                except memcap.MemcapError:
                    pass
            else:
                left.append('processes of the server cgroup')
            # what a backend started and a session or reparenting lost: every process
            # of ours that carries the server's marker in its environment
            mark = f'PROVSQL_SERVER={self.d}'.encode()
            for _ in range(50):
                alive = [pid for pid in os.listdir('/proc') if pid.isdigit() and pid != str(os.getpid())
                         and self.marked(pid, mark)]
                if not alive:
                    break
                for pid in alive:
                    try:
                        os.kill(int(pid), signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                time.sleep(0.1)
            else:
                left.append('processes ' + ' '.join(alive))
        left += [f for f in glob.glob('/tmp/provsql*') if os.path.getmtime(f) >= self.t0]
        shutil.rmtree(self.sock, ignore_errors=True)
        if left:
            raise RuntimeError('ProvSQL left ' + ', '.join(left))


# The programs ProvSQL is given: tuple-independent tables (every fact
# independent) and non-recursive conjunctive rules, which tpch2provsql.py
# writes as tables and views: TPC-H, the probabilistic databases of the lit
# suite (pdb), its noisy-or networks (noisyor, whose findings are evidence)
# and its two families of database queries (liftsep, rgraph). The lit
# suite's other families are recursive (mnist_sum, sickness, mastermind,
# eventcalc, taint) or read annotated disjunctions (mnist_sum, pedigree,
# mastermind), which a tuple-independent table cannot hold.
PROVSQL_WRITERS = ('gen_tpch.py', 'gen_pdb.py', 'gen_noisyor.py', 'gen_liftsep.py', 'gen_rgraph.py')


def run_provsql(args, name, path):
    """ProvSQL (Senellart, MIT; v1.13.0 on PostgreSQL 16) on a program of
    tuple-independent tables (PROVSQL_WRITERS): `tpch2provsql.py` writes it
    as tables with a probability per tuple and one view per rule predicate,
    loaded into a cluster of its own (the load, `psql -f load.sql`, is timed:
    building the tables and their provenance is parsing the input, as
    tddlog's reading of its fact files is; the start of the server is not),
    and each query is the SQL `SELECT probability_evaluate(provenance())
    FROM q` with ProvSQL's own choice of evaluation method, timed through psql
    as the other systems' commands are. Under evidence e the answer of q is
    P(q and e) / P(e), two such queries. The record's seconds are the load's
    and the queries', within the timeout together; the load's are the
    record's stats `load_seconds`, and a load past the timeout is a timeout."""
    d = os.path.abspath(os.path.join(args.dir, f'provsql-{name}'))
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d)
    server = ProvSQLServer(args.provsql, d)
    try:
        r = provsql_run(args, path, server, d)
    finally:
        try:
            server.stop()
        except RuntimeError as x:
            r = {'status': 'error', 'seconds': 0.0, 'error': f'ProvSQL: {x}'}
        shutil.rmtree(d, ignore_errors=True)
    return r


def provsql_run(args, path, server, d):
    """`run_provsql`'s load and queries on a server not yet started."""
    sql, qs, ev = tpch2provsql.translate(open(path).read())
    with open(os.path.join(d, 'load.sql'), 'w') as f:
        f.write(sql)
    try:
        server.start()
    except RuntimeError as x:
        return {'status': 'error', 'seconds': 0.0, 'error': f'ProvSQL load: {x}'}
    r = timed(server.psql_cmd + ['-q', '-v', 'ON_ERROR_STOP=1', '-f', os.path.join(d, 'load.sql')], args.timeout, cwd=d)
    load = r['seconds']
    if r['status'] != 'ok':
        return {**r, 'error': f"ProvSQL load: {r['error']}"} if r['status'] == 'error' else r
    answers, seconds = {}, load
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
    return {'status': 'ok', 'seconds': seconds, 'stats': {'load_seconds': round(load, 3)}, 'answers': answers}


# PSouffle (Li, Xia, Adnan and Wang, FMCAD 2026; install_psouffle.sh), exact
# probabilistic Datalog on Souffle: the derivation graph of the queried
# tuples compiled into BDDs with CUDD. It reads the programs of the logic
# program and database suites as problog2souffle.py writes them (a program
# it cannot express is refused, `unsupported`); the networks' programs,
# a relation per CPT row in that translation, are not given. Souffle
# compiles the rules into a C++ program that reads the facts when it runs:
# that compilation depends on the rules alone, is shared by the instances
# whose translations are the same text (the taint apps), and is timed apart
# from the run, as the paper times it: every record after it carries the
# seconds its text's compilation took as `compile_s` (where an earlier
# instance's compilation is reused, that compilation's), and the processing
# charges them to the record (rel/charge.py, final_all.sh's step T). The
# run is its default full inference, on one thread: its
# graph rewrites (`--rewrite`) lose derivations at 3fb0fd06 (docs/baselines.md).
PSOUFFLE_SUITES = ('standard', 'indicators', 'lit', 'tpch', 'biomine', 'circulette')
PSOUFFLE_COMPILE_S = 3600


def accepts_psouffle(args, name):
    return args.suite in PSOUFFLE_SUITES and instances(args)[name][1][0] != 'gen_bn.py'


def psouffle_binary(args, program):
    """(the compiled program of the Souffle text PROGRAM, the seconds its
    compilation took, or None and the record of its failure). Each text is
    compiled once, by the first lane that needs it, the others waiting and
    given the seconds it took."""
    d = os.path.abspath(os.path.join(args.dir, 'psouffle-bin', hashlib.sha256(program.encode()).hexdigest()[:20]))
    # the compilation is charged to the run (rel/charge.py): one past the
    # run's limit is a timeout already, and is not let run longer
    bound = min(PSOUFFLE_COMPILE_S, float(args.timeout))
    os.makedirs(d, exist_ok=True)
    with open(d + '.lock', 'w') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        if os.path.exists(os.path.join(d, 'failed')):
            with open(os.path.join(d, 'failed')) as f:
                rec = json.load(f)
            # a compilation stopped at a lower bound is tried again
            if not compile_timeout(rec) or rec['stats']['compile_s'] >= bound:
                return None, rec['stats']['compile_s'], rec
        if os.path.exists(os.path.join(d, 'compiled')):
            with open(os.path.join(d, 'compiled')) as f:
                return os.path.join(d, 'compute'), float(f.read()), None
        write_new(os.path.join(d, 'compute.dl'), program)
        r = timed([args.psouffle, '--full-only', 'compute.dl', '-o', 'compute'], bound, cwd=d)
        if r['status'] != 'ok':
            rec = {'status': 'error', 'seconds': 0.0,
                   'error': f"PSouffle's compilation: {r['status']} {str(r.get('error', ''))[-2000:]}",
                   'stats': {'compile_s': round(r['seconds'], 3)}}
            write_new(os.path.join(d, 'failed'), json.dumps(rec))
            return None, r['seconds'], rec
        write_new(os.path.join(d, 'compiled'), repr(r['seconds']))
        return os.path.join(d, 'compute'), r['seconds'], None


def run_psouffle(args, name, path):
    """PSouffle on the translation of the program (problog2souffle.py: the
    Souffle program, its input relations' facts and probabilities, and the
    names of its relations), compiled (psouffle_binary) and run on the facts
    within the timeout. A ground query PSouffle does not list has
    probability 0; a query with variables is answered by every tuple of its
    relation that PSouffle lists and that the query matches. A benchmark
    whose original is a Souffle program (SOUFFLE_WRITERS) is run as it is,
    from the copy `gen` wrote beside the program."""
    d = os.path.abspath(os.path.join(args.dir, f'psouffle-{name}'))
    shutil.rmtree(d, ignore_errors=True)
    original = os.path.splitext(path)[0] + '.souffle'
    try:
        if os.path.isdir(original):
            shutil.copytree(original, d)
        else:
            err = translated(args, 'problog2souffle.py', [path, d], None, 'psouffle')
            if err:
                return err
        with open(os.path.join(d, 'compute.dl')) as f:
            binary, compile_s, err = psouffle_binary(args, f.read())
        if err:
            return err
        stats = {'compile_s': round(compile_s, 3)}
        os.makedirs(os.path.join(d, 'out'))
        try:
            r = timed([binary, '--jobs=1', '-F', os.path.join(d, 'input'), '-D', os.path.join(d, 'out')],
                      args.timeout, cwd=d)
        except OSError as x:
            return {'status': 'error', 'seconds': 0.0, 'error': f'PSouffle: {x}', 'stats': stats}
        if r['status'] == 'error' and INCONSISTENT.search(r.get('error', '')):
            return {'status': 'ok', 'seconds': r['seconds'], 'answers': 'inconsistent', 'stats': stats}
        if r['status'] != 'ok':
            return r | {'stats': stats}
        try:
            with open(os.path.join(d, 'out', 'facts.prob')) as f, open(os.path.join(d, 'names.tsv')) as g:
                listed = problog2souffle.answers(f.read(), g.read())
            with open(os.path.join(d, 'queries.txt')) as f:
                asked = f.read().splitlines()   # the translation's: a query with a body is its ground queries
        except OSError as x:
            return {'status': 'error', 'seconds': r['seconds'], 'error': f'PSouffle: {x}', 'stats': stats}
        answers = {}
        for q in asked:
            pattern = query_pattern(q)
            if pattern is None:
                answers[q] = listed.get(q, 0.0)
            else:
                answers.update((a, p) for a, p in listed.items() if pattern.fullmatch(a))
        return {'status': 'ok', 'seconds': r['seconds'], 'answers': answers, 'stats': stats}
    finally:
        shutil.rmtree(d, ignore_errors=True)


def query_pattern(q):
    """The atoms a query with variables matches, as a pattern over atoms as
    tddlog prints them; None for a ground query."""
    m = re.fullmatch(r'(\w+)\((.*)\)', q)
    if not m:
        return None
    parts = tpch2provsql.split_top(m.group(2))
    if not any(re.fullmatch(r'[A-Z_]\w*', a) for a in parts):
        return None
    return re.compile(re.escape(m.group(1)) + r'\(' + ','.join(
        r'[^,()]+' if re.fullmatch(r'[A-Z_]\w*', a) else re.escape(a) for a in parts) + r'\)')


program_system('psouffle', "PSouffle (Li, Xia, Adnan and Wang, FMCAD 2026) on the programs of the logic program and "
               "database suites, translated by problog2souffle.py; the souffle binary install_psouffle.sh builds",
               accepts_psouffle, run_psouffle)


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
    for PRISM with dice=False one property, -pf). PRISM's Java heap and its
    decision diagrams' memory are both --prism-heap, on every model (CUDD's
    own default of 1 GB filled on the weather factory). PRISM's iterations on
    dice2prism.py's chains (unbounded reachability; the weather factory's is
    bounded) run to a relative change of 1e-15 and without its default cap of
    10 000 iterations: on an acyclic chain they stop once nothing changes,
    after as many as the chain is long."""
    if system == 'storm':
        return [args.storm, os.path.join(HERE, 'storm_check.py'), model, variant, props]
    cmd = [os.path.join(os.path.abspath(args.prism), 'bin', 'prism'), model] + ([props] if dice else ['-pf', props]) + \
        ['-' + variant, '-javamaxmem', args.prism_heap, '-cuddmaxmem', args.prism_heap]
    return cmd + (['-epsilon', '1e-15', '-maxiters', '1000000000'] if dice else [])


CHECKER_RESULT = {'storm': re.compile(r'^result ([0-9.eE+-]+)(?: (\d+(?:/\d+)?))?\s*$', re.M),
                  'prism': re.compile(r'^Result: ([0-9.eE+-]+)', re.M)}


def checker_results(system, out):
    """The values Storm or PRISM prints for its properties, in order: a
    Fraction where Storm's exact engine gives the rational, else a float."""
    if hasattr(sys, 'set_int_max_str_digits'):
        sys.set_int_max_str_digits(0)   # an exact answer's numerator can have 10 000 digits
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
# what each printed and its failure (translation), made once for every
# checker and engine.
CHAINS = {}


def checker_masses(system, variant):
    """The `masses` of run_translated for Storm or PRISM: dice2prism.py
    writes a model and its properties for each component from one
    translation (`--each`), which the checker builds once and answers. The
    translation is made once for every checker and engine (CHAINS); it is
    not timed, as no system's is, and is bounded by translation_bound: past
    it the record is the harness's time-out (translation), a chain too
    large to write, no program handed to the checker. The components a
    translation that failed wrote run first, in order, and the record is
    then its failure."""
    def masses(args, label, source, joint, budget):
        j = ['--joint'] if joint else []
        prefix = os.path.abspath(os.path.join(args.dir, f'{label}.dtmc'))   # one for every checker and engine
        key = (label, source, joint)
        if key not in CHAINS:
            CHAINS[key] = translation(args, 'dice2prism.py', [source, '--each', prefix] + j)
        listing, failed = CHAINS[key]
        comps, total, inproc = [], 0.0, 0.0
        for k, line in enumerate(listing.splitlines()):
            base = f'{prefix}{k}'
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
        if failed:
            return None, {**failed, 'seconds': total}
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


def address_space_limit():
    """The run's address-space limit in bytes (`ulimit -v`), which its
    commands inherit; None without one."""
    import resource
    soft = resource.getrlimit(resource.RLIMIT_AS)[0]
    return None if soft == resource.RLIM_INFINITY else soft


def sharpsat_td_cache_mb():
    """sharpSAT-TD's cache bound in MB (`-cs`, in units of 10^6 bytes),
    as its readme sets it for a memory bound X: X/2 - 500, here with X the
    run's memory cap (`--mem-cap`) or else its address-space limit (`ulimit
    -v`), which its commands inherit. Without `-cs` it takes 95% of the
    machine's free memory, whatever the limit, so a run without either is
    refused."""
    bound = memory_bound()
    if bound is None:
        raise SystemExit('--sharpsat-td bounds its cache by half the memory cap (--mem-cap) or the address-space '
                         "limit (ulimit -v), and there is neither; without a bound it takes 95% of the machine's "
                         'free memory')
    cache = bound // 2_000_000 - 500
    if cache < 100:
        raise SystemExit(f'--sharpsat-td: a memory bound of {bound} bytes leaves it no cache')
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
    if getattr(args, 'tddlog', None):
        source_tree.refuse_quick(args.tddlog)
    if getattr(args, 'mem_cap', None) is not None:
        cap_memory(args.mem_cap)
    spec = instances(args)
    names = list(spec)
    if args.pr:
        if args.suite not in ('uai', *EXTRA_SUITES):
            raise SystemExit(f'--pr compares the partition functions of the {" and ".join(["uai", *EXTRA_SUITES])} '
                             f'suite{"s" if EXTRA_SUITES else ""}, and the suite is ' + args.suite)
        if not (args.strategies or args.exact or args.merlin or args.ganak or getattr(args, 'sharpsat_td', '')
                or args.ace or args.kbs or args.pita or args.aspmc or args.scallop
                or any(getattr(args, s, '') for s in UAI_SYSTEMS)
                or any(getattr(args, s.replace('-', '_'), '') for s in PE_SYSTEMS)):   # P(e) as a query
            raise SystemExit('--pr compares the partition functions of tddlog, --exact, --ace, --merlin, --agrum, '
                             '--pgmpy, --libdai, --tensorinference, --kbs, --ganak, --sharpsat-td and the systems '
                             'that answer P(e) as a query ('
                             + ', '.join('--' + s for s in PE_SYSTEMS if s != 'problog') + '), and none is run')
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
    for binary in args.tddlog_binary.values():
        source_tree.refuse_quick(binary)
    systems = [('tddlog', s) for s in strategies] + [('problog', k) for k in args.kbs]
    if args.pita:
        systems.append(('pita', 'bdd'))
    if args.aspmc:
        systems.append(('aspmc', 'sharpsat-td'))
    if args.scallop:
        systems.append(('scallop', SCALLOP_VARIANT))
    if args.dice:
        systems.append(('dice', 'bdd'))
    if getattr(args, 'alea', ''):
        systems.append(('alea', 'cudd'))
    if getattr(args, 'roulette', ''):
        gone = [v for v in args.roulette_variants if v in ('nocomplement', 'extexp')]
        if gone:
            # docs/baselines.md, section 11
            raise SystemExit(f'--roulette-variants {gone[0]} was a variant of the Roulette artifact, which '
                             "neuppl/roulette's query has no form of: it takes no complement and counts in doubles")
        if 'bayes' in args.roulette_variants:
            raise SystemExit('--roulette-variants bayes was the artifact\'s driver on a network that observes nothing: '
                             'every Bayesian network now runs in Roulette\'s BN language under rsdd (roulette_bif)')
        unknown = [v for v in args.roulette_variants if v != 'rsdd']
        if unknown:
            raise SystemExit(f'--roulette-variants {unknown[0]}: the one variant is rsdd')
        systems += [('roulette', v) for v in args.roulette_variants]
    if getattr(args, 'roulette_tddlog', ''):
        if not any(roulette_tddlog_accepts(args, n) for n in names):
            raise SystemExit('--roulette-tddlog runs --suite roulette, the Bayesian networks, the Dice programs '
                             "of data/dice, data/nodice and gen_geni.py and Circulette's netlists, and none is selected")
        if args.suite == 'circulette' and not getattr(args, 'circulette_tddlog', ''):
            raise SystemExit("--roulette-tddlog on --suite circulette runs circulette.rkt of --circulette-tddlog, "
                             "a Circulette compiled by its Racket, and there is none")
        systems.append(('roulette-tddlog', 'auto'))
    if getattr(args, 'circulette_tddlog', '') and not (args.suite == 'circulette' and getattr(args, 'roulette_tddlog', '')):
        raise SystemExit('--circulette-tddlog is the Circulette of --roulette-tddlog on --suite circulette, '
                         'and neither runs')
    for opt, ps in PROGRAM_SYSTEMS.items():
        if getattr(args, opt, ''):
            if not any(ps['accepts'](args, n) for n in names):
                raise SystemExit(f'--{opt} runs {ps["help"].split(" on ")[1].split(",")[0]}, and none is selected')
            systems.append((opt, 'default'))
    if args.exact:
        variants = list(dict.fromkeys(EXACT[spec[n][1][0]][0] for n in names if spec[n][1][0] in EXACT))
        if not variants:
            raise SystemExit('--exact answers the networks, the literature families, tpch, biomine, roulette, geni and uwcse, '
                             'and none is selected')
        systems += [('exact', v) for v in variants]
        if 'problog1' in variants:
            # problog_each.py's ProbLog, one query at a time
            os.environ['BENCH_PROBLOG'] = os.path.abspath(args.problog)
    if args.ace:
        takes_networks(args, spec, names, 'ace')
        # the networks whose observations Ace reads from the .inst file gen writes (ace_observations)
        networks = [n for n in names if spec[n][1][0] == 'gen_bn.py' and not evidence_file(spec[n][1])]
        if networks and not any(os.path.exists(os.path.join(args.dir, n + '.inst')) for n in networks):
            raise SystemExit(f'--ace reads the observations bench.py gen writes beside each network (NAME.inst), and '
                             f'{args.dir} has none' + (' (gen --pr writes none)' if args.pr else ''))
        missing = [f for f in ['ace.jar', 'inflib.jar', 'jdom.jar', 'c2d_linux'] if not os.path.exists(os.path.join(args.ace, f))]
        if missing:
            raise SystemExit(f'--ace {args.ace}: no {", ".join(missing)} there')
        for v in args.ace_variants:
            ace_flags(v)   # an unknown form fails before the first run
        systems += [('ace', v) for v in args.ace_variants]
    if args.merlin:
        takes_networks(args, spec, names, 'merlin')
        if not os.access(args.merlin, os.X_OK):
            raise SystemExit(f'--merlin {args.merlin}: not an executable')
        systems.append(('merlin', 'bte'))
        if args.pr:
            systems.append(('merlin', 'bte-pr'))
    for system, (variants, _) in UAI_SYSTEMS.items():
        if getattr(args, system, ''):
            takes_networks(args, spec, names, system)
            for variant in variants:
                systems.append((system, variant))
                if args.pr:
                    systems.append((system, variant + '-pr'))
    if getattr(args, 'provsql', ''):
        if not any(spec[n][1][0] in PROVSQL_WRITERS for n in names):
            raise SystemExit('--provsql answers the programs of tuple-independent tables (PROVSQL_WRITERS: TPC-H, pdb, noisyor, liftsep, rgraph), and none is selected')
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
    # --systems: a timed run's job of one system on a program's sizes (the
    # skip rule is a system's own), the other options as the whole run's
    chosen = getattr(args, 'systems', [])
    unknown = [s for s in chosen if s not in {s for s, _ in systems}]
    if unknown:
        raise SystemExit(f'--systems {unknown[0]}: not one of the systems the other options run '
                         f'({", ".join(dict.fromkeys(s for s, _ in systems))})')
    if chosen:
        systems = [(s, v) for s, v in systems if s in chosen]
    rotate = getattr(args, 'rotate', None)
    if rotate is not None and len(systems) < 2:
        raise SystemExit('--rotate orders the systems of an instance, and there is one')
    cpus = getattr(args, 'cpus', [])
    if len(set(cpus)) != len(cpus):
        raise SystemExit(f'--cpus {",".join(map(str, cpus))}: a CPU listed twice')
    if cpus and not shutil.which('taskset'):
        raise SystemExit('--cpus pins the lanes with taskset, and there is none on the PATH')
    # A program's sizes are listed smallest first. Once a system runs out of
    # time or memory on one, the larger ones are skipped rather than waited
    # out; the sizes are then a job of one lane, every other instance a job
    # of its own.
    jobs = {}
    for name in names:
        jobs.setdefault((not args.no_skip and series(spec[name][0], name)) or (name,), []).append(name)
    jobs = list(jobs.values())
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
    """Runs `systems` on `names`, one instance or a program's sizes in order
    of size, and appends the records to `out` under `lock`. On the suites of
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
        program = series(spec[name][0], name)
        stem = os.path.join(args.dir, name)
        done, held = {}, []
        for system, variant in system_order(systems, name, getattr(args, 'rotate', None)):
            if system == 'exact' and EXACT.get(spec[name][1][0], ('',))[0] != variant:
                continue
            form = ground_format(args, name, system, variant)
            if system in NETWORK_SYSTEMS and spec[name][1][0] != 'gen_bn.py' and form is None:
                continue
            if system == 'provsql' and spec[name][1][0] not in PROVSQL_WRITERS:
                continue
            if system in MODEL_CHECKERS and not accepts_model_checker(args, name):
                continue
            if system in ('ganak', 'sharpsat-td') and not has_cnf(spec[name]):
                continue
            if system == 'roulette-tddlog' and not roulette_tddlog_accepts(args, name):
                continue
            if system in PROGRAM_SYSTEMS and not PROGRAM_SYSTEMS[system]['accepts'](args, name):
                continue
            if (system, variant, program) in dead:
                rec = {'instance': name, 'family': spec[name][0], 'system': system,
                       'variant': variant, 'status': 'skipped', 'seconds': 0}
                with lock:
                    out.write(json.dumps(rec) + '\n')
                    print(f"{name:10} {system}:{variant:10} skipped (ran out of time or memory on a smaller size)")
                continue
            src = pe if pe and system in PE_SYSTEMS else path
            LANE.usage = {}   # the memory of the record's commands (timed)
            # the systems given the instance through its ground program
            ground = ground_route(args, name, path, form) if form else None
            rargs, rpath = args, src
            if ground and ground['status'] == 'ok':
                rargs = argparse.Namespace(**{**vars(args), 'timeout': max(1.0, args.timeout - ground['seconds'])})
                rpath = ground['ground']
            if ground and ground['status'] != 'ok':
                r = dict(ground)
            elif lp_route(spec[name][1], system, args.pr):
                r = run_lp(args, name, src, system)
            elif system == 'dice':
                r = run_dice(rargs, name, rpath)
            elif system == 'alea':
                r = run_alea(rargs, name, rpath)
            elif system == 'pita':
                own = original(spec[name][1], '') if spec[name][1][0] == 'gen_cplint.py' else None
                r = run_pita(args, name, src, own)
            elif system == 'aspmc':
                r = run_aspmc(args, name, src, circuit=spec[name][1][0] == 'alea_bench.jl')
            elif system == 'scallop':
                r = run_scallop(args, name, src, variant, circuit=spec[name][1][0] == 'alea_bench.jl')
            elif system == 'ace':
                r = run_ace(rargs, name, rpath, variant)
            elif system == 'merlin':
                r = run_merlin(rargs, name, rpath, variant)
            elif system in UAI_SYSTEMS:
                r = run_uai_system(rargs, name, rpath, system, variant)
            elif system == 'provsql':
                r = run_provsql(args, name, path)
            elif system in MODEL_CHECKERS:
                r = run_model_checker(rargs, name, rpath, system, variant)
            elif system == 'roulette':
                r = run_roulette(rargs, name, rpath, variant)
            elif system == 'roulette-tddlog':
                r = run_roulette_tddlog(rargs, name, rpath)
            elif system in PROGRAM_SYSTEMS:
                r = PROGRAM_SYSTEMS[system]['run'](rargs, name, rpath)
            elif system == 'ganak':
                r = run_ganak(args, name, path)
            elif system == 'sharpsat-td':
                r = run_sharpsat_td(args, name, path, variant)
            elif system == 'exact':
                script, *rest = EXACT[spec[name][1][0]][1](path)
                if args.pr:
                    rest.append('--evidence-probability')   # the uai suite's, bn_exact.py
                r = timed([sys.executable, str(source_tree.script(script))] + rest, args.timeout)
            else:
                if system == 'tddlog':
                    binary = args.tddlog_binary.get(variant, args.tddlog)
                    target, inputs = [path], []
                    if spec[name][1][0] == 'gen_bn.py':
                        # the network itself, read as part of tddlog's run
                        target, inputs = network_args(args, name, spec[name][1],
                                                      'no-queries' in variant.split('+')[1:])
                    elif folded(args, name, spec[name][1], variant):
                        # the family's program and the instance's arguments (a scaled Dice loop's
                        # program and its count), likewise
                        inputs = list(folded(args, name, spec[name][1], variant)) + folded_data(args, spec[name][1])
                        target = inputs[:1] + (arguments_of(inputs[1]) if os.path.exists(inputs[1]) else [])
                    elif original(spec[name][1], variant, stem):
                        # the Dice program, the LPAD or the netlist itself, likewise
                        target = inputs = [original(spec[name][1], variant, stem)]
                    cmd = [binary, '--stats', '--precision', '17'] + tddlog_flags(variant) + target
                    if args.suite == 'circulette' and '--evidence-probability' not in cmd:
                        # Circulette's question includes P(fail), the evidence's probability
                        cmd[-len(target):-len(target)] = ['--evidence-probability']
                else:
                    cmd = [args.problog, '-k', variant, src]
                missing = [f for f in inputs if not os.path.exists(f)] if system == 'tddlog' else []
                r = {'status': 'error', 'seconds': 0.0, 'error': f'no {missing[0]}: rerun bench.py gen'} if missing \
                    else timed(cmd, args.timeout)
                if system == 'problog' and r['status'] == 'error' and INCONSISTENT.search(r.get('error', '')):
                    r = {**r, 'status': 'ok', 'stdout': 'evidence is inconsistent\n', 'stderr': ''}
            if ground and ground['status'] == 'ok':   # the grounding and the translation are timed
                r['seconds'] += ground['seconds']
                r.setdefault('stats', {})['ground_s'] = round(ground['seconds'], 3)
            rec = {'instance': name, 'family': spec[name][0], 'system': system,
                   'variant': variant, 'status': r['status'], 'seconds': round(r['seconds'], 3)}
            if spec[name][1][0] == 'alea_bench.jl':
                rec.update(circuit_size(path))
            if system == 'tddlog' and spec[name][1][0] == 'gen_bn.py':
                # what tddlog read: the network, no converted program (rel/charge.py)
                rec['network'] = os.path.basename(network_file(args, spec[name][1]))
            elif system == 'tddlog' and folded(args, name, spec[name][1], variant):
                # the family's program and the instance's arguments, no NAME.pl (rel/charge.py)
                # (not `program`, the series whose sizes the skip rule follows)
                template, given = folded(args, name, spec[name][1], variant)
                rec['original'] = os.path.basename(template)
                if os.path.exists(given):
                    rec['arguments'] = arguments_of(given)
                if folded_data(args, spec[name][1]):
                    rec['data'] = [os.path.basename(f) for f in folded_data(args, spec[name][1])]
            elif system == 'tddlog' and original(spec[name][1], variant, stem):
                # the Dice program, the LPAD or the netlist, no converted program (rel/charge.py)
                rec['original'] = os.path.basename(original(spec[name][1], variant, stem))
            rec.update(r.get('stats', {}))
            if r['status'] == 'ok' and 'log10_z' in r:
                rec['log10_z'] = r['log10_z']   # Merlin's PR task and the counters, and Ace's beside its answers
            if r['status'] == 'ok' and ('answers' in r or 'log10_z' not in r):
                # tddlog under `--no-queries` answers P(e) alone, as Merlin's
                # PR task does.
                asked = not (system == 'tddlog' and 'no-queries' in variant.split('+')[1:])
                if asked:
                    rec['answers'] = r['answers'] if system in ('dice', 'alea', 'aspmc', 'scallop', 'ace', 'merlin',
                                                                'roulette', 'roulette-tddlog', 'provsql', *MODEL_CHECKERS,
                                                                *UAI_SYSTEMS,
                                                                *PROGRAM_SYSTEMS) \
                        else parse_answers(r['stdout'])
                if system == 'tddlog':
                    rec.update(parse_stats(r['stderr']))
                if (system == 'tddlog' and ('evidence-probability' in variant.split('+')[1:]
                                            or args.suite == 'circulette')) or \
                        (system == 'exact' and args.pr):
                    try:
                        if 'network' in rec:
                            rec['log10_z'] = network_partition_function(r['stdout'])
                        elif args.suite == 'circulette':
                            # P(fail), the evidence's probability, which Circulette prints (run_circulette)
                            rec['log10_pe'] = parse_evidence_probability(r['stdout'])
                            if rec['log10_pe'] is None:
                                raise ValueError('no P(e) line')
                            rec['p_fail'] = 10 ** rec['log10_pe']
                        else:
                            rec['log10_pe'], rec['log10_z'] = partition_function(r['stdout'], path)
                    except (OSError, ValueError) as x:
                        r = {'status': 'error', 'error': f'partition function: {x}'}
                        rec['status'] = 'error'
                        rec.pop('answers', None)
                # A system that exits 0 without printing an answer failed
                # (aspmc does so on a parse error), and a run without
                # answers can be checked against nothing.
                if asked and rec['status'] == 'ok' and not rec['answers'] and \
                        (circulette_asked(len(circulette_gates(path))) if args.suite == 'circulette' else queries(path)):
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
            if r['status'] == 'stopped':    # stopped from outside before the limit (STOP_SIGNALS)
                rec['error'], rec['returncode'] = r['error'], r.get('returncode')
            r = take_usage(rec, r)
            # out of time or memory on one size of a program: its larger sizes
            # are skipped; another failure says nothing of them
            if program and ran_out(r) and not args.no_skip:
                dead.add((system, variant, program))
            done[(system, variant)] = rec
            if checked and system not in ('tddlog', 'exact') and rec['status'] == 'ok':
                held.append(rec)
            else:
                emit(rec)
        for rec in held:
            check_against_reference(rec, done)
            emit(rec)
        drop_ground(args, name)


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
    if rec['status'] == 'stopped':
        return 'stop'
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
    given = [f'--{k.replace("_", "-")}' for k in ['ace_variants', 'ace_heap', 'ace_java', 'ace_stack']
             if getattr(args, k, None) is not None]
    if given and not args.ace:
        raise SystemExit(f'{", ".join(given)} without --ace: Ace is not run')
    if args.sharpsat_td_decot is not None and not args.sharpsat_td:
        raise SystemExit('--sharpsat-td-decot without --sharpsat-td: sharpSAT-TD is not run')
    if getattr(args, 'roulette_variants', None) is not None and not getattr(args, 'roulette', ''):
        raise SystemExit('--roulette-variants without --roulette: Roulette is not run')
    # Under the memory cap the Java systems' heaps are set from it: Ace's at
    # 0.8 of it (the JVM's own memory and c2d's beside the heap), PRISM's
    # heap and its decision diagrams' memory 0.8 each (the cap kills a run that
    # fills both).
    cap = getattr(args, 'mem_cap', None)
    if cap is not None:
        for k, share in [('ace_heap', 0.8), ('prism_heap', 0.8)]:
            heap = getattr(args, k, None)
            if heap is None:
                setattr(args, k, f'{int(cap * share) >> 20}m')
            elif memcap.size(heap) > cap * share:
                raise SystemExit(f'--{k.replace("_", "-")} {heap} is more than {share:.2g} of --mem-cap '
                                 f'({cap >> 20} MiB): the cap would kill the run before the heap is full')
    if getattr(args, 'prism_heap', '') is None:
        args.prism_heap = '4g'
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
    g.add_argument('--circulette-netlists', default='',
                   help="the directory of the Circulette instances' netlists, NAME.cql, which gen copies and checks "
                   'against their SHA-256 (CT_SHA256)')
    g.add_argument('--tddlog', default=str(source_tree.BINARY),
                   help="the tddlog that writes the program of a network, a Dice program, an LPAD or a Circulette "
                   "netlist (`tddlog --program`), which the systems that read ProbLog programs are given")
    g.set_defaults(fn=gen)
    r = sub.add_parser('run')
    r.add_argument('--dir', default=os.path.join(ROOT, 'bench'))
    r.add_argument('--out', default='results.jsonl')
    r.add_argument('--tddlog', default=str(source_tree.BINARY))
    r.add_argument('--problog', default=os.path.join(ROOT, 'tmp', 'venv', 'bin', 'problog'))
    r.add_argument('--strategies', default='auto', type=lambda s: s.split(',') if s else [])
    r.add_argument('--tddlog-arm', action='append', default=[], metavar='STRATEGY=PATH',
                   help='run the tddlog strategy STRATEGY (one of --strategies) with the binary PATH '
                   'instead of --tddlog, as an arm built from another commit; repeatable')
    r.add_argument('--kbs', default='ddnnf,sdd,fsdd', type=lambda s: s.split(',') if s else [])
    r.add_argument('--dice', default='', help='command template for Dice with {file}, {dir}, {base}; '
                   'e.g. "docker run --rm -v {dir}:/b sholtzen/dice dice /b/{base}"; empty = not run')
    r.add_argument('--alea', default='', help='command template for Alea.jl with {file}, {dir}, {base}, run on '
                   'the script dice2alea.py writes, on the one problog2ppl.py writes of a logic program, or on '
                   'alea_bench.jl for an Alea.jl program; e.g. '
                   '"julia --project=ALEA_DIR {file}"; empty = not run')
    r.add_argument('--pita', default='', help='swipl binary with the cplint pack; empty = not run')
    r.add_argument('--pita-packs', default='', help='pack directory to attach before consulting')
    r.add_argument('--aspmc', default='', help='python with aspmc importable; empty = not run')
    r.add_argument('--scallop', default='', help='the scli binary, run with its top-k proofs at the largest k '
                   '(SCALLOP_K), which are exact; empty = not run')
    r.add_argument('--ace', default='', help="Ace 3.0's directory (ace.jar, inflib.jar, jdom.jar, c2d_linux); "
                   'empty = not run')
    r.add_argument('--ace-variants', type=lambda s: s.split(','),
                   help='Ace compile methods, comma separated: auto (the default), or options joined by + '
                   '(forceC2d+cd05)')
    r.add_argument('--ace-heap', help="the Java heap of Ace's processes (-Xmx), by default 8g")
    r.add_argument('--ace-stack', help="the thread stack of Ace's processes (-Xss, 512m say), against a "
                   "StackOverflowError on deep networks; by default the JVM's")
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
    r.add_argument('--prism-heap', help="the Java heap of PRISM's process (-javamaxmem) and its decision "
                   "diagrams' memory (-cuddmaxmem), by default 4g, or 0.8 of --mem-cap")
    r.add_argument('--weather-models', default='', help="the directory with Rubicon's weatherfactoryN.prism, for --storm and --prism")
    r.add_argument('--provsql', default='', help="the directory install_provsql.sh installs ProvSQL in (pg/bin/initdb, ...), "
                   'for the programs of tuple-independent tables (TPC-H; pdb, noisyor, liftsep and rgraph of --suite lit); empty = not run')
    r.add_argument('--roulette', default='', help="command template for Racket with neuppl/roulette's packages "
                   "roulette-lib and roulette installed, {file} the program: data/roulette's driver of the "
                   "artifact's program on --suite roulette, a Bayesian network's BIF file in Roulette's BN "
                   "language (data/roulette/bn.rkt), the program problog2ppl.py writes of a logic program, "
                   "the Dice program dice2roulette.py translates on the others; "
                   "e.g. \"env PLTUSERHOME=... racket {file}\"; empty = not run")
    r.add_argument('--roulette-variants', type=lambda s: s.split(','),
                   help="Roulette's runs: rsdd, the default and the one variant")
    r.add_argument('--roulette-tddlog', default='',
                   help="command template, as --roulette's, for the Racket that runs Roulette+tddlog "
                   "(Roulette's tddlog engine, then tddlog on the circuit it writes) on --suite roulette, the "
                   "Bayesian networks, the Dice programs of data/dice, data/nodice and gen_geni.py, and "
                   "Circulette's netlists; empty = not run")
    r.add_argument('--circulette-tddlog', default='',
                   help="--suite circulette under --roulette-tddlog: a Circulette checkout compiled by that "
                   "Racket (the fork's roulette-lib), whose circulette.rkt Roulette+tddlog runs, apart from "
                   "--circulette-benchmarks, which Roulette's Racket compiles")
    for opt, ps in PROGRAM_SYSTEMS.items():
        r.add_argument('--' + opt, dest=opt, default='', help=ps['help'] + '; empty = not run')
    r.add_argument('--geni-benchmarks', default=os.path.join(os.path.expanduser('~'), 'local/geni/src/bench/ICFP'),
                   help="the Geni artifact's bench/ICFP directory, where the weather factory's programs are")
    r.add_argument('--genfer-benchmarks', default=os.path.join(os.path.expanduser('~'), 'local/genfer/src/benchmarks/neurips2023'),
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
                   '(merlin:bte-pr), Ace from the P(e) it prints, ProbLog, PITA, aspmc and Scallop asked P(e) as a query, and '
                   '--ganak and --sharpsat-td on the weighted CNF of the network')
    r.add_argument('--timeout', type=float, default=300)
    r.add_argument('--translation-timeout', type=float,
                   help='seconds the translations of an instance into a system\'s language may take '
                   '(dice2genfer.py, dice2prism.py and the like, which are not timed): past them the '
                   'record is the harness\'s time-out, not the system\'s failure; by default 4 x --timeout')
    for p in [g, r]:
        p.add_argument('--suite', choices=['standard', 'indicators', 'large', 'lit', 'ace', 'uai', 'tpch', 'biomine',
                                           'ppl', 'bitblast', 'roulette', 'geni', 'circulette', *EXTRA_SUITES],
                       default='standard')
        p.add_argument('--ace-benchmarks', help="where Ace's benchmarks_v1.0.zip is unpacked, for --suite ace; "
                       'by default tmp/ace-benchmarks/benchmarks_v1.0')
        p.add_argument('--uai-benchmarks', help="the directory of the UAI MAR instances (NAME.uai, NAME.uai.evid), "
                       'for --suite uai; by default tmp/uai-benchmarks/MAR')
        p.add_argument('--biomine-benchmarks', help=f'the directory of the Biomine graph {BIOMINE_GRAPH}, for '
                       '--suite biomine; by default tmp/biomine-benchmarks')
        for name, suite in EXTRA_SUITES.items():
            p.add_argument(f'--{name}-benchmarks', help=f'{suite.help}, for --suite {name}; by default '
                           f'tmp/{suite.default_directory}')
        p.add_argument('--roulette-benchmarks', help="the Roulette artifact's benchmark/ directory, whose "
                       'networks (bayesian-networks/NAME.bif) gen writes the ProbLog programs of, for --suite '
                       'roulette; by default tmp/roulette-benchmarks')
        p.add_argument('--circulette-benchmarks', default='tmp/circulette',
                       help="Circulette's checkout (circulette.rkt, its netlists, examples/ and rkt_of_verilog.py), "
                       'for --suite circulette; by default tmp/circulette')
        p.add_argument('--families', default='', type=lambda s: s.split(',') if s else [])
        p.add_argument('--only', default='', type=lambda s: s.split(',') if s else [])
    r.add_argument('--no-skip', action='store_true',
                   help='run every size of a program even after a smaller one ran out of time or memory')
    r.add_argument('--systems', default=[], type=lambda s: s.split(',') if s else [],
                   help='run only these of the systems the other options give (tddlog, problog, exact, ...)')
    r.add_argument('--rotate', type=int, default=None, help="rotate the systems' order on each instance by a "
                   "checksum of its name plus R (repetition R of a timed run)")
    r.add_argument('--mem-cap', type=memcap.size,
                   help='every command capped at this much resident memory (31g, 512m, or bytes), each in a cgroup '
                   'of its own (memcap.py; the run starts under `memcap.py delegate --`, without `ulimit -v`); the '
                   "records keep each command's peak; Ace's and PRISM's heaps and Julia's heap size hint are set "
                   'from it'
                   )
    r.add_argument('--cpus', default='', type=lambda s: [int(c) for c in s.split(',')] if s else [],
                   help="run in parallel lanes, one per CPU listed, each command pinned to its lane's CPU with "
                   "taskset; a program's sizes are a job of one lane, every other instance (every instance "
                   "under --no-skip) a job of its own")
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
    t.add_argument('--approx', default='', type=lambda s: s.split(',') if s else [],
                   help='systems or system:variant names that approximate (the records of Scallop\'s top-k '
                   'proofs at a k below SCALLOP_K, scallop:topk10 say); their difference from the reference '
                   'gets a column of its own instead of entering the exact one')
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
