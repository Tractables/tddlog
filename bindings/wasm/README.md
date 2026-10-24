# tddlog in the browser

<p align="center">
  <img src="https://raw.githubusercontent.com/Tractables/tddlog/main/docs/logo.svg"
       alt="tddlog" width="340">
  <br>
  <a href="https://github.com/Tractables/tddlog/actions/workflows/pages.yml"><img
     src="https://github.com/Tractables/tddlog/actions/workflows/pages.yml/badge.svg"
     alt="Site"></a>
  <a href="https://tractables.github.io/tddlog/"><img
     src="https://img.shields.io/badge/run-in%20the%20browser-blue" alt="Run in the browser"></a>
</p>

The [playground](https://tractables.github.io/tddlog/) runs tddlog in the
tab: a ProbLog program goes in, and the page answers its queries. The
solver is the crate's
[`solve`](https://tractables.github.io/tddlog/api/tddlog/fn.solve.html),
built for WebAssembly with Emscripten. It runs in a worker, so the page
stays live during a run and a run can be cancelled.

This directory holds the module's wrapper and the site: the playground,
the [guide](https://tractables.github.io/tddlog/guide/) and the
[results](https://tractables.github.io/tddlog/results/).

## The playground

The examples are programs the repository tests against ProbLog, from
burglary and alarm to a game defined by negation through recursion, and
the page shows ProbLog's answer beside each of tddlog's. A program can be
edited or written from scratch. The vtree strategy (`--vtree` on the
command line), P(e) (`--evidence-probability`) and the digits shown
(`--precision`) are settings. Under the answers the page says what tddlog
did: the layout that answered (for `auto`, the trial that won and how
many compilations it took), the size of the ground program, the largest
formulas in pairs, and the time of each stage. The editor colours the
program as the guide does; a file can be opened in it or dropped on it,
and Download saves the program. The address carries the example, the
settings and an edited program (after `#program=`, where its code is at
most 64 KB), so a result can be linked to. `#bench=SUITE/FAMILY/INSTANCE`
(the results page's "Open in the playground") opens the program tddlog
was given for an instance of the release run, with that run's answers
beside tddlog's, and runs it where it is at most 1 MB and tddlog answered
it natively within 2 s.

## The site

The playground is `index.html`, `styles.css`, `theme.js`, `code.js`,
`app.js`, `worker.js`, `runtime.js` and `examples.js` beside the
Emscripten build and the example programs: no framework, no bundler,
nothing fetched from anywhere else. It follows `main`. The guide
(`guide/`) and the results (`results/`) are pages of their own with the
same styles; the guide's examples open in the playground through
`?example=<key>`, and the browser test checks that each key it links to
is one of `examples.js`.

The results page shows the release run against every exact system that
accepts a suite's programs, one view at a time, named by the address's
hash and browsed from a rail beside it: an overview of the kinds of
problems; for each kind, the instances each system answers within a time
limit and its total time, as the README's figures; for each suite a plot,
tddlog against each system, the findings, its families and a table of
every instance's times; and for each family of instances their times,
each instance's programs in every language a system was given, its
answers, and every run of a system in a drawer beside the view.
`results/results.js` draws it from `suites.json` (the networks and the
logic programs) and `ppl.json` (the programs of Dice, Alea.jl, Roulette
and Geni), and from `benchmarks/`, a JSON file per family, where the site
has it. All are generated from the release run's records, as the README's
results and the baselines branch's docs/benchmarks.md are, so they change
together; they are not edited by hand. `benchmarks/` holds the
baselines' programs, so it is on the baselines branch, which the Pages
workflow fetches it from. `404.html` sends a link to a benchmark's former
page (`results/benchmarks/SUITE/NAME.html`) to its view.

## Memory

WebAssembly's 32-bit address space caps the heap at 4 GB, and a browser
may grant less. Where the heap cannot grow, an allocation fails as on the
command line: an operation the solver can refuse retires its trial and
`auto` goes on with the others; any other aborts the module, and the page
says that the run ran out of memory and loads the solver again.

## Build locally

You need the [Emscripten SDK](https://emscripten.org/docs/getting_started/downloads.html)
4.0.7 on PATH, the Rust target, Node.js, and Python 3 for the bindings'
pages:

```sh
rustup target add wasm32-unknown-emscripten
cd bindings/wasm
CXX_wasm32_unknown_emscripten=em++ \
AR_wasm32_unknown_emscripten=emar \
CXXFLAGS_wasm32_unknown_emscripten=-fwasm-exceptions \
  cargo build --release
(cd ../.. && DOCS_RS=1 cargo doc --no-deps --lib)
# The Python and C pages, in a virtualenv: the Python page reads the
# docstrings of the package, installed from this checkout.
(cd ../.. && mkdir -p bindings/python/notices &&
  cp LICENSE NOTICE THIRD-PARTY.md bindings/python/notices/ &&
  pip install -r bindings/python/docs/requirements.txt ./bindings/python &&
  python -m sphinx -W -b html -d target/docs-cache/python bindings/python/docs target/python-docs &&
  python -m sphinx -W -b html -d target/docs-cache/c bindings/c/docs target/c-docs)
./site.sh site ../../target/doc ../../target/python-docs ../../target/c-docs
python3 -m http.server -d site
```

goatd's vendored FlowCutter is C++ and throws, so it is compiled with
em++ for the exception model rustc links with. The module is fetched, so
the files have to come from a server: opening `index.html` from the file
system does not work. `site.sh` puts the guide under `guide/`, the
results under `results/`, the API documentation under `api/`, and the
Python and C pages, built with Sphinx from their READMEs and their
references, under `python/` and `c/`.

## Tests

```sh
cargo test --release --target x86_64-unknown-linux-gnu
node --test tests/runtime.cjs
node tests/examples.cjs site/tddlog.js
npm install --no-save playwright@1.49.1
npx playwright install chromium
node tests/browser.cjs site screenshots
```

The first tests the wrapper natively, the second the page's runtime with
a stand-in for the module, the third runs every example through the built
module against ProbLog's answers and P(e), as `examples.js` records them,
and the last drives the assembled site, the results page among it, in a
headless Chromium.

## Publishing

`.github/workflows/pages.yml` builds the module, the API documentation
and the site, runs the tests, and on `main` publishes the site to GitHub
Pages. It stamps the references in the pages with the commit
(`app.js?v=<sha>`), and the playground passes the stamp on to the worker,
the module and the example programs, so a new page never runs with an
older file a browser still holds.

## Logo

`docs/logo.svg` is the project logo. Site assembly copies it to the site root
and adds it to the playground, guide and results pages with the page's
cache stamp. The binding documentation and Rust API use the same SVG.
