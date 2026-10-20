# tddlog in the browser

The playground at <https://tractables.github.io/tddlog/>: a ProbLog program
goes in, and tddlog answers its queries in the tab. The solver is the crate's
[`solve`](https://tractables.github.io/tddlog/api/tddlog/fn.solve.html),
built for WebAssembly with Emscripten; it runs in a worker, so the page stays
live during a run and a run can be cancelled.

The examples are programs the repository tests against ProbLog: burglary
and alarm with evidence, coins, a die thrown until it shows six, cards drawn
by annotated disjunctions, the asia Bayesian network, smokers in a social
network, reachability in a graph with cycles, two integers added in
binary, a game whose winning positions are defined by negation through
recursion, gear wheels that turn each other, and the sum of two digits.
Beside each answer the page shows ProbLog's. A program can be edited or
written from scratch; the vtree strategy (`--vtree` on the command line),
P(e) (`--evidence-probability`) and the digits shown (`--precision`) are
settings. Under the answers the page says what tddlog did: the layout that
answered (for `auto`, the trial that won and how many compilations it took),
the size of the ground program, the largest formulas in pairs, and the time
of each stage. The address carries the example and the settings, and an
edited program after `#program=`, so a result can be linked to.

The page is `index.html`, `styles.css`, `theme.js`, `app.js`, `worker.js`,
`runtime.js` and `examples.js` beside the Emscripten build of this crate and
the example programs: no framework, no bundler, nothing fetched from
anywhere else. It follows `main`. The guide (`guide/`: the language,
worked examples, the command line and the bindings) and the results
(`results/`) are pages of their own with the same styles; the guide's
examples open in the playground through `?example=<key>`, and the browser
test checks that each key it links to is one of `examples.js`.

## Results

`results/` is the page at <https://tractables.github.io/tddlog/results/>:
the release run against every exact system that accepts a suite's
programs, a family of suites at a time. Each suite has a plot of the
instances each system answers within a time limit (from 0.2 s, on a log
scale) that draws the systems given all of its instances, a small plot
for each part of the instances that some systems were given alone, the
findings in prose beside it, and a table of every instance's times.
`results.js` draws it from two files: `suites.json`, the networks
and the logic programs, and `ppl.json`, the programs of Dice, Alea.jl,
Roulette and Geni. Both are generated from the release run's records, as
the README's results and the baselines branch's docs/benchmarks.md are, so the three change
together; they are data, not to be edited by hand.

## Memory

WebAssembly's 32-bit address space caps the heap at 4 GB, and a browser may
grant less. Where the heap cannot grow, an allocation fails as on the
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

goatd's vendored FlowCutter is C++ and throws, so it is compiled with em++
for the exception model rustc links with. Opening `index.html` from the file
system does not work: the module is fetched, so the files have to come from
a server. `site.sh` puts the guide's page under `guide/`, the results under
`results/`, the API
documentation under `api/`, and the Python and C bindings' pages, built
with Sphinx from their READMEs and their references, under `python/` and
`c/`.

## Tests

```sh
cargo test --release --target x86_64-unknown-linux-gnu
node --test tests/runtime.cjs
node tests/examples.cjs site/tddlog.js
npm install --no-save playwright@1.49.1
npx playwright install chromium
node tests/browser.cjs site screenshots
```

The first tests the wrapper natively, the second the page's runtime with a
stand-in for the module, the third runs every example through the built
module against ProbLog's answers and P(e), as `examples.js` records them,
and the last drives the assembled site in a headless Chromium, the
results page among it.

## Publishing

`.github/workflows/pages.yml` builds the module, the API documentation and
the site, runs the tests, and on `main` publishes the site to GitHub Pages.
It stamps the references in the pages with the commit (`app.js?v=<sha>`),
and the playground passes the stamp on to the worker, the module and the
example programs, so a new page never runs with an older file a browser
still holds.
