#!/usr/bin/env python3
"""The benchmarks of Circulette as netlists, for bench.py's --suite
circulette.

    gen_circulette.py KIND ARG --circulette DIR --cql OUT.cql [--python PY]

Circulette (Sandia National Laboratories, github.com/sandialabs/circulette,
LGPL-3.0; SAND2025-13969R) is a language on Roulette's package roulette-lib
for the fault analysis of digital circuits. A netlist (.cql) lists a
circuit's word gates; circulette.rkt evaluates it for one cycle with every
input false and each gate's output flipped by a fault of probability 0.01,
conditions on the outputs differing from the expected ones (all false), and
asks rsdd for P(fail) and for the posterior of every single, pair and triple
of faults: 1 + n + C(n,2) + C(n,3) calls of `infer` for n faults.

KIND and ARG name the netlist, written to OUT.cql:
  root NAME        Circulette's own NAME.cql (1-bit-mux, 4-bit-comparator,
                   adder8, exp), copied.
  verilog FILES    examples/FILES (comma separated; the modules a design
                   instantiates before it) concatenated and converted by
                   rkt_of_verilog.py, as examples/Makefile converts them.
  fulladder N      examples/adder/fullAdder8.v's ripple-carry adder at N
                   bits, with halfAdder.v and fullAdder.v; N = 8 is the
                   file as written.
  mac N            examples/openfpga_mac_2.v's multiply-accumulate at
                   DATA_WIDTH N; N = 2 is the file as written (and
                   openfpga_mac_4.v is N = 4, up to comments).

tddlog reads the netlist itself (its Circulette reader), and `tddlog
--program OUT.cql` writes the ProbLog program the other systems read
(bench.py's gen). The netlists of bench.py's instances were written once by
this script and are kept as files (bench.py gen --circulette-netlists,
checked against CT_SHA256), since rkt_of_verilog.py at Circulette's commit
numbers a design's ports in an order that changes between runs.

PY (default python3) runs rkt_of_verilog.py: a Python with pyrtl 0.12 and
Circulette's patch, with yosys on PATH.
"""
import argparse
import os
import shutil
import subprocess
import tempfile


def verilog(circulette, kind, arg):
    """The Verilog text of a netlist of kind verilog, fulladder or mac."""
    ex = os.path.join(circulette, 'examples')

    def read(*parts):
        with open(os.path.join(ex, *parts)) as f:
            return f.read()

    if kind == 'verilog':
        return ''.join(read(*f.split('/')) for f in arg.split(','))
    n = int(arg)
    if n < 1:
        raise SystemExit(f'gen_circulette.py: {kind} {n}: the size is at least 1')
    if kind == 'fulladder':
        adder = read('adder', 'fullAdder8.v')
        if n != 8:
            for old, new in [('[7:0]', f'[{n - 1}:0]'), ('i < 8', f'i < {n}'), ('carry[7]', f'carry[{n - 1}]'),
                             ('module fullAdder8', f'module fullAdder{n}')]:
                if old not in adder:
                    raise SystemExit(f'gen_circulette.py: fullAdder8.v has no {old}')
                adder = adder.replace(old, new)
        return read('adder', 'halfAdder.v') + read('adder', 'fullAdder.v') + adder
    if kind == 'mac':
        mac = read('openfpga_mac_2.v')
        if n != 2:
            for old, new in [('parameter DATA_WIDTH = 2', f'parameter DATA_WIDTH = {n}'),
                             ('module mac_2', f'module mac_{n}')]:
                if old not in mac:
                    raise SystemExit(f'gen_circulette.py: openfpga_mac_2.v has no {old}')
                mac = mac.replace(old, new)
        return mac
    raise SystemExit(f'gen_circulette.py: unknown kind {kind}')


def netlist(args):
    """Writes the netlist to args.cql."""
    if args.kind == 'root':
        shutil.copyfile(os.path.join(args.circulette, args.arg + '.cql'), args.cql)
        return
    text = verilog(args.circulette, args.kind, args.arg)
    stem = os.path.basename(args.cql)[:-len('.cql')]
    # every path by its resolved name, and pyrtl's temporary files beside
    # the netlist: YoWASP's sandbox, which runs yosys, follows no link. Two
    # runs numbered a design's inputs and outputs differently (the same model,
    # its gates alike); the string hash is fixed, against an order that
    # follows it.
    with tempfile.TemporaryDirectory(dir=os.path.dirname(os.path.realpath(args.cql))) as d:
        with open(os.path.join(d, stem + '.v'), 'w') as f:
            f.write(text)
        p = subprocess.run([args.python, os.path.join(os.path.realpath(args.circulette), 'rkt_of_verilog.py'),
                            stem + '.v'], cwd=d, capture_output=True, text=True,
                           env={**os.environ, 'TMPDIR': d, 'PYTHONHASHSEED': '0'})
        out = os.path.join(d, stem + '.cql')
        if p.returncode != 0 or not os.path.exists(out):
            tail = (p.stderr.strip() or p.stdout.strip()).splitlines()[-5:]
            raise SystemExit(f'gen_circulette.py: rkt_of_verilog.py on {args.kind} {args.arg}: exit {p.returncode}: '
                             + ' | '.join(tail))
        shutil.copyfile(out, args.cql)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('kind', choices=['root', 'verilog', 'fulladder', 'mac'])
    ap.add_argument('arg')
    ap.add_argument('--circulette', required=True, help="Circulette's checkout")
    ap.add_argument('--cql', required=True, help='the netlist written (NAME.cql)')
    ap.add_argument('--python', default='python3', help='the Python that runs rkt_of_verilog.py')
    args = ap.parse_args()
    if not args.cql.endswith('.cql'):
        raise SystemExit('gen_circulette.py: --cql names a .cql file')
    netlist(args)


if __name__ == '__main__':
    main()
