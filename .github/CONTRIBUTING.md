# Contributing

## Checks

```sh
cargo fmt --check
cargo test --release
cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
python3 -m unittest discover -s tests -p 'test_*.py'
python3 .github/scripts/check_release.py
```

`cargo test` runs `tests/it/`, the differential tests against ProbLog's
answers and exact sums, and the crate's doctests; CI runs it under
`--profile quick`, which builds without link-time optimization and gives
the same answers, and also checks that Rust 1.88 builds the crate. The
Python tests check the scripts and need numpy; `check_release.py` checks
that the crate and the bindings declare one version.

## Every answer is checkable

Every probability tddlog prints must be checkable against ProbLog. A change
that adds a construct, a rewrite or a vtree strategy lands with a test in
`tests/it/` against ProbLog's answer (ProbLog 2.3.0, `-k sdd`, one query at
a time) or against a sum worked out by hand. `tests/compare.py` runs a
program through tddlog and ProbLog and diffs the probabilities, and
`tests/fuzz_problog.py` does it for random programs in the supported
fragment; both need ProbLog (`pip install problog`).

An option that does not apply to a program, or a combination that asks for
nothing, is an error: tddlog never ignores an option silently. The library
is single-threaded; the caller decides on parallelism.

## Examples and the guide

`examples/*.pl` are small programs with known answers, and
`tests/it/examples.rs` checks each under several strategies. The programs
the playground offers are listed in `bindings/wasm/examples.js` with
ProbLog's answers to 12 significant digits and P(e), which
`bindings/wasm/tests/examples.cjs` checks the built module against. The
guide (`bindings/wasm/guide/index.html`) quotes the programs and outputs of
the examples it walks through; a change to one of them changes the guide
too.

## Bindings and the site

- Python: `bindings/python/README.md` builds the extension with maturin;
  `pytest bindings/python/tests` tests it.
- C: `python bindings/c/check.py` checks the committed header against
  cbindgen's, builds and tests the library with CMake, and builds the
  example against the installed package; after a change to
  `bindings/c/src/lib.rs`, `python bindings/c/check.py --write-header`
  regenerates the header.
- WebAssembly and the site: `bindings/wasm/README.md` builds the module and
  the site and runs its tests, the last in a headless Chromium.

Each binding is a crate of its own, which CI formats with `cargo fmt
--check` and lints with `cargo clippy -- -D warnings`.

## Documentation

`docs/design.md` describes the code as it is, and changes with it. The
README's results, `docs/benchmarks.md` (on the baselines branch) and the results page's data
(`bindings/wasm/results/*.json`) are generated from the records of a
release run, and are not edited by hand. Documentation and commit messages
are plain prose: technical facts, without flourish.

## Releasing

A release sets one version in `Cargo.toml`, the three bindings'
`Cargo.toml` and `CITATION.cff`, and
`python3 .github/scripts/check_release.py --release-tag v<version>`
checks them before the tag is made. Publishing a GitHub release of that
tag builds and tests the wheels and uploads them to PyPI through trusted
publishing (`wheels.yml`), and attaches the C archives and their
checksums to the release (`c-bindings.yml`). Neither replaces what it
published: a PyPI version can be yanked but not uploaded again, and an
attached archive stays as it is.
