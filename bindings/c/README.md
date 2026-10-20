# tddlog from C and C++

<p align="center">
  <a href="https://github.com/Tractables/tddlog/releases/latest"><img
     src="https://img.shields.io/github/v/release/Tractables/tddlog"
     alt="GitHub release"></a>
  <a href="https://github.com/Tractables/tddlog/actions/workflows/c-bindings.yml"><img
     src="https://github.com/Tractables/tddlog/actions/workflows/c-bindings.yml/badge.svg"
     alt="C bindings"></a>
  <a href="https://github.com/Tractables/tddlog/blob/main/bindings/c/include/tddlog.h"><img
     src="https://img.shields.io/badge/API-C%20%2F%20C%2B%2B-blue"
     alt="C and C++ API"></a>
  <a href="https://github.com/Tractables/tddlog/blob/main/LICENSE"><img
     src="https://img.shields.io/badge/license-Apache--2.0-blue.svg"
     alt="License: Apache-2.0"></a>
</p>

A C ABI over [tddlog](https://github.com/Tractables/tddlog): a ProbLog program
goes in as text, with the command line's options, and the probabilities of
its queries come out through an opaque solution handle.

**The ABI is unstable before 1.0.** Struct layouts, status codes and
signatures can change in any release. Build against the version you ship with
and check `tddlog_version()` at run time.

This is a separate crate with its own `Cargo.toml`. It builds the tddlog
sources in the repository and carries the same version, which
`tddlog_version()` reports.

## Use the API

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

`examples/alarm.c` is this program on the alarm example, checked against the
probabilities worked out by hand; `tests/answers.c` checks a dozen of
ProbLog's system tests against ProbLog's answers under several vtree
strategies.

- `TddlogOptions` holds the command line's options: `vtree` (`--vtree`, null
  for `"auto"`), `evidence_probability` (`--evidence-probability`) and
  `queries` (false for `--no-queries`, which needs `evidence_probability`).
  `tddlog_solve` takes null options as the defaults.
- `tddlog_solve` returns `TDDLOG_OK` or an error status:
  `TDDLOG_ERROR_INVALID_INPUT` for a null pointer, text that is not UTF-8, an
  unknown strategy or options that ask for nothing; `TDDLOG_ERROR_PARSE`,
  `TDDLOG_ERROR_GROUND` and `TDDLOG_ERROR_COMPILE` for a program tddlog cannot
  answer; `TDDLOG_ERROR_PANIC` for a bug. `tddlog_last_error_message()` gives
  the message the command line prints.
- The answers are in the order of the program's queries, each query written
  as ProbLog prints it (`\+p` for `query(\+p)`). Where the evidence is
  inconsistent, `tddlog_solution_consistent` is false and there are none.
- `tddlog_solution_evidence_probability` gives `P(e)` and its base-10
  logarithm, which stays exact where `P(e)` is under the least double, where
  `evidence_probability` asked for it, and returns false otherwise.

The caller owns the solution and releases it with `tddlog_solution_free`. The
query strings belong to the solution and live until it is freed;
`tddlog_version()` is static; the string from `tddlog_last_error_message()`
belongs to tddlog and lives until the next `tddlog_solve` on that thread.

tddlog is single-threaded. Each thread may call `tddlog_solve` on its own
program; error messages are recorded per thread, and threads may read one
solution at once.

## Build and install with CMake

From a source checkout, with Rust (1.88 or newer), a C++20 compiler for the
tree-decomposition solvers goatd vendors, a C compiler and CMake:

```sh
cmake -S bindings/c -B build/c -DCMAKE_BUILD_TYPE=Release
cmake --build build/c --config Release
ctest --test-dir build/c -C Release --output-on-failure
cmake --install build/c --config Release --prefix /your/install/prefix
```

The installation holds the header, the shared library, a CMake package and a
pkg-config file, and LICENSE, NOTICE and THIRD-PARTY.md under
`share/licenses/tddlog`. An application then uses
`find_package(tddlog CONFIG REQUIRED)` and links `tddlog::tddlog`, with
`CMAKE_PREFIX_PATH` set to the prefix, or reads its flags from pkg-config:

```sh
export PKG_CONFIG_PATH=/your/install/prefix/lib/pkgconfig
cc prog.c $(pkg-config --cflags --libs tddlog) -o prog
```

The library needs no Rust toolchain at run time; the dynamic loader has to
find it (an rpath, `LD_LIBRARY_PATH` on Linux, `DYLD_LIBRARY_PATH` on
macOS).

`python bindings/c/check.py` does all of the above and then builds and runs
the example against the installed package, moved to another prefix, through
both CMake and pkg-config, which it needs on PATH.

## Build with cargo alone

```sh
cargo build --release --manifest-path bindings/c/Cargo.toml
```

builds `libtddlog_c.so` (`libtddlog_c.dylib` on macOS) and `libtddlog_c.a` in
`bindings/c/target/release`; the header is `bindings/c/include/tddlog.h`.
Against the shared library nothing else is needed. A static link also needs
the C++ runtime the tree-decomposition solvers are compiled against and the
system libraries Rust's standard library uses; rustc prints the list:

```sh
cargo rustc --release --manifest-path bindings/c/Cargo.toml -- \
    --print native-static-libs
```

which on Linux prints something like `-lstdc++ -lgcc_s -lutil -lrt -lpthread
-lm -ldl -lc`. Pass that after the archive:

```sh
cc -Ibindings/c/include prog.c bindings/c/target/release/libtddlog_c.a \
   -lstdc++ -lgcc_s -lutil -lrt -lpthread -lm -ldl -lc -o prog
```

## Install from a release

Each GitHub release attaches a `tddlog-c-<tag>-<target>.tar.gz` archive for
`x86_64-unknown-linux-gnu` and `aarch64-apple-darwin`, and a
`tddlog-c-<tag>-x86_64-pc-windows-msvc.zip`. It holds the header,
the shared and the static library, LICENSE, NOTICE, THIRD-PARTY.md, and
`NATIVE-STATIC-LIBS.txt`, the system libraries a static link needs on that
platform. On Windows the libraries are `tddlog_c.dll`, its import library
`tddlog_c.dll.lib` and the static `tddlog_c.lib`; the CMake and pkg-config
packages are not built there.

## Regenerate the header

`include/tddlog.h` is generated from `src/lib.rs` by
[cbindgen](https://github.com/mozilla/cbindgen), which the crate's build
script runs, and committed. `check.py` fails where the committed copy has
drifted; after changing `src/lib.rs`, run

```sh
python bindings/c/check.py --write-header
```
