# tddlog for C and C++

<p align="center">
  <img src="https://raw.githubusercontent.com/Tractables/tddlog/main/docs/logo.svg"
       alt="tddlog" width="340">
  <br>
  <a href="https://github.com/Tractables/tddlog/actions/workflows/c-bindings.yml"><img
     src="https://github.com/Tractables/tddlog/actions/workflows/c-bindings.yml/badge.svg"
     alt="C bindings"></a>
  <a href="https://tractables.github.io/tddlog/c/"><img
     src="https://img.shields.io/badge/docs-C-blue" alt="C API"></a>
  <a href="https://tractables.github.io/tddlog/guide/"><img
     src="https://img.shields.io/badge/docs-guide-blue" alt="Guide"></a>
</p>

**Exact probabilistic inference from C and C++.**

tddlog computes exact probabilities for ProbLog programs, LPADs, Dice
programs and Bayesian and Markov networks, by compiling them into tree
decision diagrams. This library exposes the
same Rust implementation as the
[command line and Rust crate](https://github.com/Tractables/tddlog) and
the [Python package](https://tractables.github.io/tddlog/python/)
through a C interface: a program goes in as text, in any of these
formats, with the command line's options, and the probabilities of its queries come out through an
opaque solution handle.

**The ABI is unstable before 1.0.** Struct layouts, status codes and
signatures can change in any release. Build against the version you ship
with and check `tddlog_version()` at run time.

## Install

There are no prebuilt archives yet. Build and install the package from a
checkout, with Rust 1.88 or newer, a C++20 compiler for the
tree-decomposition solvers goatd vendors, a C compiler and CMake:

```sh
git clone https://github.com/Tractables/tddlog
cd tddlog
cmake -S bindings/c -B build/c -DCMAKE_BUILD_TYPE=Release
cmake --build build/c --config Release
ctest --test-dir build/c -C Release --output-on-failure
cmake --install build/c --config Release --prefix /your/install/prefix
```

The installation holds the header, the shared library, a CMake package,
a pkg-config file, and the license and notices under
`share/licenses/tddlog`. An application then uses
`find_package(tddlog CONFIG REQUIRED)` and links `tddlog::tddlog`, with
`CMAKE_PREFIX_PATH` set to the prefix, or reads its flags from
pkg-config:

```sh
export PKG_CONFIG_PATH=/your/install/prefix/lib/pkgconfig
cc prog.c $(pkg-config --cflags --libs tddlog) -o prog
```

The dynamic loader has to find the library at run time (an rpath,
`LD_LIBRARY_PATH` on Linux, `DYLD_LIBRARY_PATH` on macOS). The CMake
package is for Linux and macOS.

Without CMake, as on Windows,
`cargo build --release --manifest-path bindings/c/Cargo.toml` builds the
libraries in `bindings/c/target/release`: `libtddlog_c.so`
(`libtddlog_c.dylib` on macOS) and `libtddlog_c.a`, or on Windows
`tddlog_c.dll`, `tddlog_c.dll.lib` and `tddlog_c.lib`. The header is
`bindings/c/include/tddlog.h`. A static link also needs the C++ runtime
and the system libraries that
`cargo rustc --release --manifest-path bindings/c/Cargo.toml -- --print native-static-libs`
prints, after the archive. On Linux:

```sh
cc -Ibindings/c/include prog.c bindings/c/target/release/libtddlog_c.a \
   -lstdc++ -lgcc_s -lutil -lrt -lpthread -lm -ldl -lc -o prog
```

## Getting started

This program answers the queries of `program`, a ProbLog program as
text, and the probability of its evidence:

```c
TddlogOptions options = tddlog_options_default();
options.evidence_probability = true;

TddlogSolution *solution;
if (tddlog_solve(program, &options, &solution) != TDDLOG_OK) {
    fprintf(stderr, "%s\n", tddlog_last_error_message());
    return 1;
}
for (size_t i = 0; i < tddlog_solution_num_answers(solution); i++)
    printf("%s:\t%f\n", tddlog_solution_query(solution, i),
           tddlog_solution_probability(solution, i));
double evidence, log10_evidence;
if (tddlog_solution_evidence_probability(solution, &evidence, &log10_evidence))
    printf("P(e):\t%g\tlog10 %f\n", evidence, log10_evidence);
tddlog_solution_free(solution);
```

`examples/alarm.c` runs it on the alarm example and checks the answers
against the probabilities worked out by hand.

- `TddlogOptions` holds the command line's options: `vtree` (`--vtree`,
  null for `"auto"`), `evidence_probability` (`--evidence-probability`)
  and `queries` (false for `--no-queries`, which needs
  `evidence_probability`). Null options are the defaults.
- `format` in the options says what the text is, as the command line
  tells it by the file's extension: `"problog"` or null (an LPAD where it
  holds `:- begin_lpad.`), `"dice"`, or a network's, `"bif"`, `"hugin"`
  (`.net`), `"xdsl"` or `"uai"`, with every variable queried. A network's
  `evidence` is the text of its evidence file, an Ace `.inst` file or,
  for a UAI network, a UAI `.evid` file. For a UAI `MARKOV` network,
  `P(e)` is the partition function given the evidence.
- `tddlog_solve` returns `TDDLOG_OK` or an error status, and
  `tddlog_last_error_message()` the message the command line prints:
  `TDDLOG_ERROR_INVALID_INPUT` for a null pointer, an unknown format or
  strategy, evidence for a program that is not a network, or options
  that ask for nothing;
  `TDDLOG_ERROR_PARSE`, `TDDLOG_ERROR_GROUND` and `TDDLOG_ERROR_COMPILE`
  for a program tddlog cannot answer; `TDDLOG_ERROR_PANIC` for a bug.
- The answers are in the order of the program's queries, each as ProbLog
  prints it. Where the evidence is inconsistent,
  `tddlog_solution_consistent` is false and there are none.
- `tddlog_solution_evidence_probability` gives `P(e)` and its base-10
  logarithm, which stays exact where `P(e)` is under the least double,
  when `evidence_probability` asked for them, and returns false
  otherwise.
- The caller frees the solution with `tddlog_solution_free`; its query
  strings live until then. The string from `tddlog_last_error_message()`
  lives until the next `tddlog_solve` on that thread.
- tddlog is single-threaded: each thread may solve its own program, and
  error messages are kept per thread.

The [header](https://github.com/Tractables/tddlog/blob/main/bindings/c/include/tddlog.h)
documents every type and function, and the
[guide](https://tractables.github.io/tddlog/guide/) describes the
language, with worked examples that open in the
[playground](https://tractables.github.io/tddlog/).

## Develop

`python bindings/c/check.py` builds, tests and installs the package with
CMake, then builds and runs the example against the installed package,
moved to another prefix, through both CMake and pkg-config (which it
needs on PATH). `tests/answers.c` checks a dozen of ProbLog's system
tests against ProbLog's answers under several vtree strategies, and a
Dice program and networks read in their formats.

`include/tddlog.h` is generated from `src/lib.rs` by
[cbindgen](https://github.com/mozilla/cbindgen), which the crate's build
script runs, and committed. `check.py` fails where the committed copy has
drifted; after changing `src/lib.rs`, run
`python bindings/c/check.py --write-header`.

## Acknowledgements

This material is based upon work supported by the Defense Advanced Research
Projects Agency (DARPA) under Agreement No. HR0011269E224.
