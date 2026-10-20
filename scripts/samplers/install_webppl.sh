#!/bin/bash
# install_webppl.sh: WebPPL 0.9.15 (npm `webppl`, MIT licence) and webppl-timeit
# 0.3.0 (MIT), the WebPPL baselines of the bit-blasting paper. The artifact's programs use `timeit` through
# `--require webppl-timeit`. Both packages and their 43 dependencies are
# installed with `npm ci` from the lock file beside this script
# (webppl-package.json, webppl-package-lock.json), which pins every package
# by version and SHA-512 integrity; npm refuses a tarball whose hash differs.
# WebPPL 0.9.15 runs on node 18 (tested with 18.19.1); it declares node >= 4.
# If the box has no node >= 12, a pinned node v18.20.4 tarball from nodejs.org is
# installed under $BASE/node and checked against nodejs.org's SHASUMS256.txt
# and the SHA-256 recorded here (the SHASUMS file is not GPG-verified).
#
# Installs to $HOME/local/webppl (WEBPPL_BASE overrides). npm's cache lives in
# $BASE/npm-cache. Idempotent. Ends with a smoke test: coinBias-like
# program, rejection sampling, 1000 samples (exact posterior mean 5/12).
set -u
set -o pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
WV=0.9.15
NV=v18.20.4
NSHA=c4b0827dc47609d0a8379e6de6c74b3934da0b1312c733b5ebdcac16e3f1e954
BASE=${WEBPPL_BASE:-$HOME/local/webppl}
mkdir -p "$BASE" || exit 1
cd "$BASE" || exit 1
echo "### $(hostname) base $BASE ($(readlink -f "$BASE"))"
df -h "$BASE" | sed 's/^/### df /'
for t in curl tar sha256sum; do
  command -v $t >/dev/null || { echo "### missing tool: $t" >&2; exit 1; }
done
NODE=$(command -v node || true)
nodemajor() { "$1" -p 'process.versions.node.split(".")[0]' 2>/dev/null || echo 0; }
if [ -z "$NODE" ] || [ "$(nodemajor "$NODE")" -lt 12 ]; then
  NODE=$BASE/node/node-$NV-linux-x64/bin/node
  if [ ! -x "$NODE" ]; then
    mkdir -p node || exit 1
    tarball=node-$NV-linux-x64.tar.gz
    (cd node &&
      curl -sSfLO "https://nodejs.org/dist/$NV/SHASUMS256.txt" &&
      curl -sSfLO "https://nodejs.org/dist/$NV/$tarball" &&
      grep " $tarball\$" SHASUMS256.txt | sha256sum -c - &&
      echo "$NSHA  $tarball" | sha256sum -c - &&
      tar xzf "$tarball" && rm -f "$tarball") || exit 1
  fi
fi
export PATH="$(dirname "$NODE"):$PATH"
NPM=$(dirname "$NODE")/npm
echo "### node $("$NODE" --version) ($NODE), npm $("$NPM" --version)"
if [ "$(grep -o '"version": "[^"]*"' node_modules/webppl/package.json 2>/dev/null | head -1)" != "\"version\": \"$WV\"" ]; then
  cp "$HERE/webppl-package.json" package.json || exit 1
  cp "$HERE/webppl-package-lock.json" package-lock.json || exit 1
  "$NPM" ci --prefix "$BASE" --cache "$BASE/npm-cache" --no-audit --no-fund 2>&1 | sed 's/^/### npm /' || exit 1
fi
W=$BASE/node_modules/webppl/webppl
[ -f "$W" ] && [ -d "$BASE/node_modules/webppl-timeit" ] || { echo "### install failed" >&2; exit 1; }
echo "### webppl $(grep -o '"version": "[^"]*"' node_modules/webppl/package.json | head -1), webppl-timeit $(grep -o '"version": "[^"]*"' node_modules/webppl-timeit/package.json | head -1)"
# smoke test
T=$(mktemp -d "${TMPDIR:-/tmp}/webppl-smoke.XXXXXX") || exit 1
cat > "$T/coin.wppl" <<'WP'
var ys = [1, 1, 0, 1, 0];
var model = function() {
  var mu = beta({a: 2, b: 5});
  map(function(y) { condition(bernoulli({p: mu}) == (y == 1)); }, ys);
  return mu;
}
timeit(function() { return expectation(Infer({method: 'rejection', samples: 1000}, model)); })
WP
out=$("$NODE" "$W" "$T/coin.wppl" --require "$BASE/node_modules/webppl-timeit" --random-seed 1 2>&1)
echo "### smoke: $out" | tr '\n' ' '; echo
rm -f "$T/coin.wppl"; rmdir "$T"
v=$(echo "$out" | tr '\n' ' ' | sed -n 's/.*value: \([^,}]*\),.*/\1/p')
[ -n "$v" ] && awk -v v="$v" 'BEGIN{exit !(v > 0.30 && v < 0.55)}' \
  || { echo "### smoke test failed: $out" >&2; exit 1; }
du -sh "$BASE" | sed 's/^/### du /'
echo "### installed: webppl $WV in $BASE (WEBPPL_HOME for webppl.sh), node $NODE"
