# tddlog for Python

<p align="center">
  <img src="https://raw.githubusercontent.com/Tractables/tddlog/main/docs/logo.svg"
       alt="tddlog" width="340">
  <br>
  <a href="https://github.com/Tractables/tddlog/actions/workflows/ci.yml"><img
     src="https://github.com/Tractables/tddlog/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://tractables.github.io/tddlog/python/"><img
     src="https://img.shields.io/badge/docs-Python-blue" alt="Python API"></a>
  <a href="https://tractables.github.io/tddlog/guide/"><img
     src="https://img.shields.io/badge/docs-guide-blue" alt="Guide"></a>
</p>

**Exact probabilistic inference from Python.**

tddlog computes exact probabilities for ProbLog programs, LPADs, Dice
programs and Bayesian and Markov networks, by compiling them into tree
decision diagrams. This package calls the same Rust implementation as the
[command line and Rust crate](https://github.com/Tractables/tddlog) and
the [C library](https://tractables.github.io/tddlog/c/).

## Install

tddlog is not on PyPI yet. Build it from a checkout, with Python 3.10 or
later, Rust 1.88 or newer and a C++20 compiler. `pyproject.toml` cannot
reference files above its own directory, so the license and the notices
are copied in first:

```sh
git clone https://github.com/Tractables/tddlog
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
`:- begin_lpad.`), `"dice"`, or a network's, `"bif"`, `"hugin"`
(`.net`), `"xdsl"` or `"uai"`, with every variable queried. A network's
`evidence` is the text of its evidence file, an Ace `.inst` file or, for
a UAI network, a UAI `.evid` file:

```python
network = open("examples/asia.net").read()
observed = open("tests/networks/asia_leaves.inst").read()
solution = tddlog.solve(network, format="hugin", evidence=observed)
```

For a UAI `MARKOV` network, `evidence_probability` is the partition
function given the evidence. A
program tddlog cannot answer raises `tddlog.ParseError`,
`GroundingError` or `CompilationError`, with the message the command line
prints. `solve` releases the interpreter lock, so threads can solve
several programs at once.

The [API reference](https://tractables.github.io/tddlog/python/reference.html)
documents each function and exception, `examples/` holds this program
and one built from Python data, and the
[guide](https://tractables.github.io/tddlog/guide/) describes the
language, with worked examples that open in the
[playground](https://tractables.github.io/tddlog/).

## Develop

From the checkout, after the copy above, build the extension in place
and run its tests:

```sh
python -m venv .venv
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
