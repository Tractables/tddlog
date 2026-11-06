#!/usr/bin/env python3
"""A sequence model of Pluck's (Bowers et al., PLDI 2025), ported to ProbLog.

usage: gen_pluck.py PORT [--dice OUT]

Prints PORT, data/alea/pluck/NAME.pl, the ProbLog port of the Alea.jl
program NAME.jl beside it, which ports Pluck's model (data/alea/README.md,
"Ports"). With --dice OUT, also writes the model's Dice port beside PORT,
NAME.dice, to OUT, for the systems that read Dice programs (bench.py's
DICE_PORTS: hmm alone has one).
"""
import argparse
import os
import shutil
import sys

p = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
p.add_argument('port')
p.add_argument('--dice', metavar='OUT')
a = p.parse_args()
if not a.port.endswith('.pl') or not os.path.exists(a.port):
    raise SystemExit(f'gen_pluck.py: {a.port}: no ProbLog port')
dice = a.port[:-len('.pl')] + '.dice'
if a.dice:
    if not os.path.exists(dice):
        raise SystemExit(f'gen_pluck.py: --dice: no Dice port {dice}')
    shutil.copyfile(dice, a.dice + '.part')
    os.replace(a.dice + '.part', a.dice)
with open(a.port, encoding='utf-8') as f:
    sys.stdout.write(f.read())
