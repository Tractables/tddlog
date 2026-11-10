# tddlog for C and C++

<p align="center">
  <img src="https://raw.githubusercontent.com/Tractables/tddlog/main/docs/logo.svg"
       alt="tddlog" width="340">
  <br>
  <a href="https://github.com/Tractables/tddlog/actions/workflows/c-bindings.yml"><img
     src="https://github.com/Tractables/tddlog/actions/workflows/c-bindings.yml/badge.svg"
     alt="C bindings"></a>
  <a href="https://tractables.github.io/tddlog/"><img
     src="https://img.shields.io/badge/website-results%2C%20playground%2C%20guide-e3a72f" alt="Website: results, playground, guide"></a>
  <a href="https://tractables.github.io/tddlog/c/"><img
     src="https://img.shields.io/badge/docs-C-blue" alt="C API"></a>
</p>

**Exact probabilistic inference from C and C++.**

tddlog computes exact probabilities for ProbLog programs, LPADs, Dice
programs, Circulette netlists and Bayesian and Markov networks. This
library exposes the same Rust implementation as the
[command line](https://github.com/Tractables/tddlog) through a C
interface. A program goes in as text, and the probabilities of its
queries come out through an opaque solution handle.

**The ABI is unstable before 1.0.** Struct layouts, status codes and
signatures can change in any release. Build against the version you ship
with and check `tddlog_version()` at run time.

## Install

The [release](https://github.com/Tractables/tddlog/releases/tag/v0.1.1)
has a prebuilt archive for Linux x86-64 and ARM64, macOS ARM64 and
x86-64, and Windows x86-64. It needs no Rust toolchain. On Linux x86-64:

```sh
curl -L https://github.com/Tractables/tddlog/releases/download/v0.1.1/tddlog-c-v0.1.1-x86_64-unknown-linux-gnu.tar.gz | tar xz
export PKG_CONFIG_PATH=$PWD/tddlog-c-v0.1.1-x86_64-unknown-linux-gnu/lib/pkgconfig
cc prog.c $(pkg-config --cflags --libs tddlog) -o prog
```

- A `.tar.gz` holds the header in `include/`, the shared and static
  libraries, the CMake package and `lib/pkgconfig` in `lib/`, and the
  license and notices in `share/licenses/tddlog`.
- With CMake, set `CMAKE_PREFIX_PATH` to the unpacked directory, call
  `find_package(tddlog CONFIG REQUIRED)` and link `tddlog::tddlog`. The
  CMake package is for Linux and macOS.
- The Windows zip holds the header, `tddlog_c.dll`, its import library
  `tddlog_c.dll.lib` and the static `tddlog_c.lib`.
- `NATIVE-STATIC-LIBS.txt` in each archive names the system libraries a
  static link needs.
- The dynamic loader has to find the library at run time: an rpath,
  `LD_LIBRARY_PATH` on Linux or `DYLD_LIBRARY_PATH` on macOS.

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

`examples/alarm.c` runs it on the alarm example and checks the answers.
It calls `log10`, so it links the maths library too:

```sh
cc bindings/c/examples/alarm.c $(pkg-config --cflags --libs tddlog) -lm -o alarm
```

- `TddlogOptions` holds the command line's options: `vtree`,
  `evidence_probability` and `queries`. Null options are the defaults.
- `format` says what the text is: `"problog"` or null (LPADs included),
  `"dice"`, `"circulette"`, or a network's `"bif"`, `"hugin"`, `"xdsl"`
  or `"uai"`.
- `evidence` is the text of a network's evidence file (Ace's `.inst` or
  UAI's `.evid`).
- `arguments` are strings that take the program's places `$1`, `$2`, ...
- `tddlog_solve` returns `TDDLOG_OK` or an error status, and
  `tddlog_last_error_message()` gives the command line's message.
- The answers are in the order of the program's queries. Where the
  evidence is inconsistent, `tddlog_solution_consistent` is false and
  there are none.
- The caller frees the solution with `tddlog_solution_free`. Its query
  strings live until then.
- tddlog is single-threaded. Each thread may solve its own program, and
  error messages are kept per thread.

The [header](https://github.com/Tractables/tddlog/blob/main/bindings/c/include/tddlog.h)
documents every type, function and status code. The
[guide](https://tractables.github.io/tddlog/guide/) describes the
language, and its worked examples open in the
[playground](https://tractables.github.io/tddlog/playground/). The
[website](https://tractables.github.io/tddlog/) has the benchmark results.

## Build from source

This needs Rust 1.88 or newer ([rustup](https://rustup.rs) installs it),
a C++20 compiler, a C compiler, `make` and CMake 3.16 or newer. The
guide's
[getting started](https://tractables.github.io/tddlog/guide/#getting-started)
says what to do if the build fails.

```sh
git clone --depth 1 https://github.com/Tractables/tddlog
cd tddlog
cmake -S bindings/c -B build/c -DCMAKE_BUILD_TYPE=Release
cmake --build build/c --config Release
ctest --test-dir build/c -C Release --output-on-failure
cmake --install build/c --config Release --prefix /your/install/prefix
```

The installation is laid out as the archive above. Without CMake, as on
Windows, cargo builds the libraries in `bindings/c/target/release`, and
the header is `bindings/c/include/tddlog.h`:

```sh
cargo build --release --manifest-path bindings/c/Cargo.toml
```

A static link also needs the C++ runtime and the system libraries that
`cargo rustc --release --manifest-path bindings/c/Cargo.toml -- --print native-static-libs`
prints. On Linux:

```sh
cc -Ibindings/c/include prog.c bindings/c/target/release/libtddlog_c.a \
   -lstdc++ -lgcc_s -lutil -lrt -lpthread -lm -ldl -lc -o prog
```

`python bindings/c/check.py` builds, tests and installs the package, then
builds the example against it through CMake and pkg-config.
`include/tddlog.h` is generated from `src/lib.rs` by
[cbindgen](https://github.com/mozilla/cbindgen) and committed. After
changing `src/lib.rs`, run `python bindings/c/check.py --write-header`.

## Acknowledgements

This material is based upon work supported by the Defense Advanced Research
Projects Agency (DARPA) under Agreement No. HR0011269E224.
