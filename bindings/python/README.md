# tddlog for Python

<p align="center">
  <a href="https://pypi.org/project/tddlog/"><img
     src="https://img.shields.io/pypi/v/tddlog.svg" alt="PyPI"></a>
  <a href="https://pypi.org/project/tddlog/"><img
     src="https://img.shields.io/pypi/pyversions/tddlog.svg" alt="Python versions"></a>
  <a href="https://github.com/Tractables/tddlog/actions/workflows/wheels.yml"><img
     src="https://github.com/Tractables/tddlog/actions/workflows/wheels.yml/badge.svg"
     alt="Wheels"></a>
  <a href="https://github.com/Tractables/tddlog/blob/main/LICENSE"><img
     src="https://img.shields.io/badge/license-Apache--2.0-blue.svg"
     alt="License: Apache-2.0"></a>
</p>

Python bindings for [tddlog](https://github.com/Tractables/tddlog), built with
[PyO3](https://pyo3.rs) and [maturin](https://www.maturin.rs). tddlog computes
exact marginal probabilities for programs in the Datalog fragment of ProbLog,
by compiling them bottom up into tree decision diagrams.

## Install

```sh
pip install tddlog
```

Wheels cover CPython 3.10 and later on Linux x86-64, macOS arm64 and Windows x86-64.
Anywhere else, pip builds from the source distribution, which needs a Rust
toolchain (1.88 or newer) and a C++20 compiler.

## Use

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

`solution.answers` maps each query, as ProbLog prints it (`\+p` for
`query(\+p)`), to its probability, in the order of the program's queries. It
is `None` where the evidence is inconsistent. `examples/` holds this program
and one built from Python data.

## Options

`solve` takes the command line's options under its names:

- `vtree`: the vtree strategy, `"auto"` by default, or one of the layouts
  `tddlog --help` lists (`"td"`, `"closed"`, `"families"`, `"families:0"`,
  `"dynamic"`, `"program"`, `"program:fanin"`, `"decision"`). Every
  strategy that applies to a program gives the same answers; they differ
  in how fast they get there.
- `evidence_probability=True`: also compute `P(e)`, the probability of the
  evidence, as `solution.evidence_probability` and, exact below the least
  float, `solution.log10_evidence_probability`. Both are `None` otherwise.
  The evidence no query depends on is then compiled rather than dropped.
- `queries=False`: leave out the program's queries and compute `P(e)` alone,
  as for a partition function. It needs `evidence_probability=True`.

An unknown strategy, or options that ask for nothing, raise `ValueError`.

## Errors

A program tddlog cannot answer raises a subclass of `tddlog.Error` whose
message is the one the command line prints:

- `tddlog.ParseError`: the program is not in the language tddlog reads;
- `tddlog.GroundingError`: it uses a construct outside the supported
  fragment or has a rule that is not range restricted;
- `tddlog.CompilationError`: the strategy asked for does not apply to the
  program, or the well-founded model of some world leaves an atom undefined
  (negation through a cycle with no two-valued reading).

The supported language is described in the
[guide](https://tractables.github.io/tddlog/guide/#language).

## Threads

The interpreter lock is released for the whole of a `solve`, so a caller can
solve several programs at once from Python threads. Each `solve` runs on the
thread that calls it.

## Building from source

The extension builds the tddlog sources in the repository, so it needs a Rust
toolchain, a C++20 compiler for the tree-decomposition solvers goatd vendors,
and maturin. The three notice files are copied in from the repository root
first, since `pyproject.toml` cannot reference files above its own directory:

```sh
mkdir -p bindings/python/notices
cp LICENSE NOTICE THIRD-PARTY.md bindings/python/notices/
pip install maturin
maturin build --release --manifest-path bindings/python/Cargo.toml
```

`maturin develop --release` installs the extension into the active
virtualenv, and `pytest bindings/python/tests` tests it.
