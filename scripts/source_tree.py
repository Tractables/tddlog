"""The tddlog source tree the benchmark scripts run on.

The scripts sit in the source tree's `scripts/`, or in a checkout that holds
the source tree in `tddlog/`. Importing this module finds it and puts its
`scripts/` and `tests/` on the import path: the converters and the reference
answers the scripts share with it.

`program(path)` is the program tddlog reads a file as (`tddlog --program`):
the conversion of a network, a Dice program or an LPAD that the harness
gives the systems that read ProbLog programs.
"""
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'tddlog' if (ROOT / 'tddlog' / 'Cargo.toml').is_file() else ROOT
# The command-line program, as `cargo build --release` writes it.
BINARY = SOURCE / 'target' / 'release' / 'tddlog'


def script(name):
    """The path of the script NAME: the benchmark scripts' own, or the source
    tree's."""
    own = ROOT / 'scripts' / name
    return own if own.exists() else SOURCE / 'scripts' / name


def program(*args, stdout=None, binary=None):
    """The program tddlog reads a file as, `tddlog --program ARGS` (ARGS
    the file and, for a network, its queries and observations): a
    network's, a Dice program's or an LPAD's, by `binary`, else $TDDLOG,
    else BINARY. Returns its text, or with `stdout` (a file) writes it
    there. Its notes and errors go to standard error, and a failure stops
    the caller."""
    cmd = [binary or os.environ.get('TDDLOG') or str(BINARY), '--program', *map(str, args)]
    if stdout is not None:
        stdout.flush()
    p = subprocess.run(cmd, stdout=stdout if stdout is not None else subprocess.PIPE, text=True)
    if p.returncode != 0:
        raise SystemExit(f'{" ".join(cmd)}: exit {p.returncode}')
    return p.stdout


for _dir in (SOURCE / 'scripts', SOURCE / 'tests'):
    if str(_dir) not in sys.path:
        sys.path.append(str(_dir))

if __name__ == '__main__':
    print(SOURCE)   # for the shell scripts
