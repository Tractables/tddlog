# tddlog for Python

<p align="center">
  <img src="https://raw.githubusercontent.com/Tractables/tddlog/main/docs/logo.svg"
       alt="tddlog" width="340">
  <br>
  <a href="https://pypi.org/project/tddlog/"><img
     src="https://img.shields.io/pypi/v/tddlog.svg" alt="PyPI"></a>
  <a href="https://github.com/Tractables/tddlog/actions/workflows/ci.yml"><img
     src="https://github.com/Tractables/tddlog/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://tractables.github.io/tddlog/python/"><img
     src="https://img.shields.io/badge/docs-Python-blue" alt="Python API"></a>
  <a href="https://tractables.github.io/tddlog/guide/"><img
     src="https://img.shields.io/badge/docs-guide-blue" alt="Guide"></a>
</p>

**Exact probabilistic inference from Python.**

tddlog computes exact probabilities for ProbLog programs, LPADs, Dice
programs, Circulette netlists and Bayesian and Markov networks, by
compiling them into tree decision diagrams. This package calls the same
Rust implementation as the [command line and Rust crate](https://github.com/Tractables/tddlog) and
the [C library](https://tractables.github.io/tddlog/c/).

## Install

```sh
pip install tddlog
```

Wheels cover CPython 3.10 and later on Linux and macOS (x86-64 and
ARM64) and Windows x86-64, and need nothing else installed. Elsewhere
pip builds from the source distribution, as below.

pip also builds the package from the repository, for a commit after the
release, with Python 3.10 or later, Rust 1.88 or newer
([rustup](https://rustup.rs) installs it), a C++20 compiler (GCC 11 or
newer or Clang 14 or newer) and `make`; the command line's
[install notes](https://github.com/Tractables/tddlog#install) say what
to do when the default `cargo` or `g++` is older. The build takes a few
minutes on eight cores. In a virtual environment:

```sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install "git+https://github.com/Tractables/tddlog#subdirectory=bindings/python"
```

From a checkout, the license and the notices are copied in first, since
`pyproject.toml` cannot reference files above its own directory:

```sh
git clone --depth 1 https://github.com/Tractables/tddlog
cd tddlog
mkdir -p bindings/python/notices
cp LICENSE NOTICE THIRD-PARTY.md bindings/python/notices/
python -m pip install ./bindings/python
```

## Getting started

```python
import tddlog

solution = tddlog.solve("""
    0.1::burglary.
    0.2::earthquake.
    0.7::hears_alarm(john).
    0.4::hears_alarm(mary).
    alarm :- burglary.
    alarm :- earthquake.
    calls(X) :- alarm, hears_alarm(X).
    evidence(calls(john), true).
    query(burglary).
    query(earthquake).
    query(calls(mary)).
""", evidence_probability=True)

for query, probability in solution.answers.items():
    print(f"{query}: {probability:.6f}")
print(f"P(e) = {solution.evidence_probability:.6f}")
```

prints

```text
burglary: 0.357143
earthquake: 0.714286
calls(mary): 0.400000
P(e) = 0.196000
```

`solution.answers` maps each query, as ProbLog prints it, to its
probability given the evidence, in the program's order; it is `None`
where the evidence is inconsistent. `solve` takes the command line's
options as keywords: `vtree` (`--vtree`), `evidence_probability`
(`--evidence-probability`) and `queries` (`False` for `--no-queries`).
`format` says what the text is, as the command line tells it by the
file's extension: `"problog"`, the default (an LPAD where it holds
`:- begin_lpad.`), `"dice"`, `"circulette"` (a `.cql` netlist), or a
network's, `"bif"`, `"hugin"`
(`.net`), `"xdsl"` or `"uai"`, with every variable queried. A network's
`evidence` is the text of its evidence file, an Ace `.inst` file or, for
a UAI network, a UAI `.evid` file:

```python
network = open("examples/car.net").read()
observed = open("examples/car.inst").read()
solution = tddlog.solve(network, format="hugin", evidence=observed)
```

`arguments` are a ProbLog program's, an LPAD's or a Dice program's
arguments, the values the command line takes after the program's file: a
list of strings, each the text of a term that takes the places `$1`, `$2`,
... of the program, as in `tddlog.solve(open("examples/chain.pl").read(),
arguments=["10", "0.9", "4"])`; in a Dice program a place stands for an
integer, and its argument is a natural number.

For a UAI network, `evidence_probability` is the partition
function given the evidence. A
program tddlog cannot answer raises `tddlog.ParseError`,
`GroundingError` or `CompilationError`, with the message the command line
prints. `solve` releases the interpreter lock, so threads can solve
several programs at once.

The [API reference](https://tractables.github.io/tddlog/python/reference.html)
documents each function and exception, `bindings/python/examples/` holds
this program and one built from Python data
(`python bindings/python/examples/alarm.py` in a checkout), and the
[guide](https://tractables.github.io/tddlog/guide/) describes the
language, with worked examples that open in the
[playground](https://tractables.github.io/tddlog/playground/).

## Develop

From the checkout, after the copy above, build the extension in place
and run its tests:

```sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install 'maturin>=1.9,<2' pytest
maturin develop --release --manifest-path bindings/python/Cargo.toml
python -m pytest bindings/python/tests
```

`maturin build --release --manifest-path bindings/python/Cargo.toml`
builds a wheel instead.

## Acknowledgements

This material is based upon work supported by the Defense Advanced Research
Projects Agency (DARPA) under Agreement No. HR0011269E224.
