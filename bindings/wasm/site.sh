#!/bin/bash
# site.sh OUT API PYTHON C [STAMP [BENCHMARKS]]: assemble the site served at
# https://tractables.github.io/tddlog/ in the directory OUT, which is
# replaced:
#
#   /           the playground: index.html, its scripts and styles, and
#               tddlog.js and tddlog.wasm from this crate's release build
#               (target/wasm32-unknown-emscripten/release)
#   /examples/  the example programs, copied from where examples.js says
#               the repository keeps them
#   /guide/     the guide (guide/index.html)
#   /results/   the release run's results (results/): the page, its script
#               and the data it reads, suites.json and ppl.json, which the
#               generator of README.md's results and the baselines branch's
#               docs/benchmarks.md
#               writes
#   /results/benchmarks/
#               a page per benchmark of the results' tables, with its
#               programs in every language a system read and every run:
#               BENCHMARKS, where given, the benchmarks/ directory of the
#               baselines branch (the pages show the baselines' programs,
#               which main does not hold); without it the results page
#               links to none
#   /api/       the API documentation: API is rustdoc's output directory,
#               target/doc after `cargo doc --no-deps --lib` at the root
#   /python/    the Python package's page: PYTHON is Sphinx's output for
#               bindings/python/docs
#   /c/         the C library's page: C is Sphinx's output for
#               bindings/c/docs
#
# STAMP, where given (the workflow passes the commit), is appended to the
# pages' references to their scripts and styles (app.js?v=STAMP), and the
# playground and the results page pass it on to every file they load, so
# that a browser never runs a new page with older files it holds.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
out=$1
api=$(cd "$2" && pwd)
python=$(cd "$3" && pwd)
c=$(cd "$4" && pwd)
stamp=${5:-}
benchmarks=${6:-}
module=$here/target/wasm32-unknown-emscripten/release

rm -rf "$out"
mkdir -p "$out/examples" "$out/guide" "$out/results"
out=$(cd "$out" && pwd)
cd "$here"
cp index.html styles.css theme.js app.js worker.js runtime.js examples.js "$module/tddlog.js" "$module/tddlog.wasm" "$out/"
node -e 'for (const e of require("./examples.js").tddlogExamples) console.log(e.file)' |
  while read -r file; do cp "../../$file" "$out/examples/"; done
cp guide/index.html "$out/guide/"
cp results/index.html results/results.js results/suites.json results/ppl.json "$out/results/"
if [ -n "$benchmarks" ]; then
  test -f "$benchmarks/index.json"
  cp -r "$benchmarks" "$out/results/benchmarks"
fi
test -f "$api/tddlog/index.html"
cp -r "$api" "$out/api"
printf '<!doctype html>\n<meta charset="utf-8">\n<meta http-equiv="refresh" content="0; url=tddlog/index.html">\n' > "$out/api/index.html"
test -f "$python/index.html"
cp -r "$python" "$out/python"
test -f "$c/index.html"
cp -r "$c" "$out/c"

if [ -n "$stamp" ]; then
  for page in "$out/index.html" "$out/guide/index.html" "$out/results/index.html"; do
    sed -i -E "s#(href|src)=\"((\.\./)?(styles\.css|theme\.js|runtime\.js|examples\.js|app\.js|results\.js))\"#\1=\"\2?v=$stamp\"#" "$page"
  done
  test "$(grep -c "?v=$stamp\"" "$out/index.html")" = 5
  test "$(grep -c "?v=$stamp\"" "$out/guide/index.html")" = 2
  test "$(grep -c "?v=$stamp\"" "$out/results/index.html")" = 3
  if [ -n "$benchmarks" ]; then
    find "$out/results/benchmarks" -name '*.html' -exec \
      sed -i -E "s#(href|src)=\"((\.\./)*(styles\.css|theme\.js|bench\.css|bench\.js))\"#\1=\"\2?v=$stamp\"#" {} +
  fi
fi
# GitHub Pages serves the files as they are, without Jekyll.
touch "$out/.nojekyll"
echo "site.sh: assembled $out ($(du -sh "$out" | cut -f1))"
