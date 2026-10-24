"""The tddlog source tree the benchmark scripts run on.

The scripts sit in the source tree's `scripts/`, or in a checkout that holds
the source tree in `tddlog/`. Importing this module finds it and puts its
`scripts/` and `tests/` on the import path: the converters and the reference
answers the scripts share with it.
"""
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


for _dir in (SOURCE / 'scripts', SOURCE / 'tests'):
    if str(_dir) not in sys.path:
        sys.path.append(str(_dir))

if __name__ == '__main__':
    print(SOURCE)   # for the shell scripts
