#!/usr/bin/env python3
"""bench_one.py: one system's run on one Dice program, recorded as `bench.py
run` records it (one JSON line, appended to OUT under a lock). The table of
docs/benchmarks.md of Dice's loops scaled to more steps ran it on the
programs `dice_scale.py` wrote.

    bench_one.py SCRIPTS SYSTEM PROGRAM REP CPU TIMEOUT OUT

SCRIPTS is a tddlog scripts directory with dice2alea.py; SYSTEM is
tddlog, onehot, dice, alea or problog (ProbLog's d-DNNF on the .pl, the
reference); PROGRAM a path without extension whose .dice and
.pl (`tddlog --program`'s translation) exist. tddlog reads the .dice
itself, so that reading it is part of its time (bench.py's `original`);
`onehot` is tddlog on the program's one-hot translation, $ONEHOT/NAME.pl
(dice2problog_onehot.py), recorded as tddlog's variant auto+onehot. The
runs are bench.py's own: `timed` for the wall time, pinned to CPU,
`run_dice` and `run_alea` for Dice and Alea.jl with the program as a
`dice` instance (the Alea script written by dice2alea.py, untimed, to a
directory of the run's own under the program's, work/). The commands come from the environment: TDDLOG,
DICE (a binary), ALEA (a template with {file}) and PROBLOG (a binary).
"""
import fcntl
import json
import os
import sys
from types import SimpleNamespace

scripts, system, program, rep, cpu, timeout, out = sys.argv[1:]
sys.path.insert(0, scripts)
import bench  # noqa: E402

name = os.path.basename(program)
pl, dice = program + '.pl', program + '.dice'
bench.instances = lambda args: {name: ('dice', ['dice2problog.py', dice])}
bench.LANE.cpu = int(cpu)
# Alea's script goes to a directory of the run's own: the repetitions of a
# program may run at once in two lanes.
work = os.path.join(os.path.dirname(program), 'work', f'{name}.{system}.{rep}')
os.makedirs(work, exist_ok=True)
args = SimpleNamespace(dir=work, timeout=float(timeout),
                       dice=os.environ['DICE'] + ' {file}', alea=os.environ['ALEA'])
if system in ('tddlog', 'onehot'):
    src = dice if system == 'tddlog' else os.path.join(os.environ['ONEHOT'], name + '.pl')
    r = bench.timed([os.environ['TDDLOG'], '--stats', '--precision', '17', src], args.timeout)
    if r['status'] == 'ok':
        r['answers'] = bench.parse_answers(r['stdout'])
    system, variant = 'tddlog', 'auto' if system == 'tddlog' else 'auto+onehot'
elif system == 'dice':
    r, variant = bench.run_dice(args, name, pl), 'bdd'
elif system == 'alea':
    r, variant = bench.run_alea(args, name, pl), 'cudd'
elif system == 'problog':
    r, variant = bench.timed([os.environ['PROBLOG'], '-k', 'ddnnf', pl], args.timeout), 'ddnnf'
    if r['status'] == 'ok':
        r['answers'] = bench.parse_answers(r['stdout'])
else:
    raise SystemExit(f'unknown system {system}')
rec = {'instance': name, 'family': 'dice_scaled', 'system': system, 'variant': variant, 'rep': int(rep),
       'status': r['status'], 'seconds': round(r['seconds'], 3)}
if variant == 'auto':
    rec['original'] = os.path.basename(dice)       # no converted program (rel/charge.py)
rec.update(r.get('stats', {}))
if r['status'] == 'ok':
    rec['answers'] = r['answers']
    if system == 'tddlog':
        rec.update(bench.parse_stats(r['stderr']))
elif r['status'] == 'error':
    rec['error'] = r['error'][-2000:]
with open(out, 'a') as f:
    fcntl.flock(f, fcntl.LOCK_EX)
    f.write(json.dumps(rec) + '\n')
inproc = f" in-process {rec['inproc_seconds']:.2f} s" if 'inproc_seconds' in rec else ''
print(f"### {name} {system}:{variant} rep {rep} cpu {cpu}: {rec['status']} {rec['seconds']:.2f} s{inproc}"
      + (f" ({rec['error'][-300:]})" if 'error' in rec else ''))
