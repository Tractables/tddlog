#!/bin/bash
# install_stan.sh: CmdStan 2.32.2 (https://github.com/stan-dev/cmdstan, BSD
# 3-clause licence) as the Stan HMC baseline of the bit-blasting paper.
# 2.32.2 is the last release series that accepts both the old
# array syntax (`int y[N];`, used by 12 of the artifact's .stan files) and the
# `array[N] int` syntax (used by 4 others); 2.33 and later reject the old
# one. The artifact's own binaries were built with CmdStan 2.28.2 (stanc 2.28.1).
# The release tarball is checked against a SHA-256 taken when the tarball was
# first downloaded: the publisher gives no checksum for 2.32.2 (GitHub shows
# digests only for releases made since 2025).
#
# Installs to $HOME/local/stan (STAN_BASE overrides) on the box's local disk.
# Idempotent: a built tree is kept. Build width: JOBS (default 8). Ends with a
# smoke test: the bundled bernoulli example, posterior mean of theta (exact
# posterior mean 3/12 = 0.25 for the bundled data; the check accepts
# 0.25 +- 0.05 on 1000 draws).
set -u
set -o pipefail
SV=2.32.2
SSHA=e8174f74ec14dfc05dc2d2c1360e0098fcb93fdfe4a81b7527bf5a2fab3db40a
URL=https://github.com/stan-dev/cmdstan/releases/download/v$SV/cmdstan-$SV.tar.gz
BASE=${STAN_BASE:-$HOME/local/stan}
JOBS=${JOBS:-8}
CS=$BASE/cmdstan-$SV
mkdir -p "$BASE" || exit 1
cd "$BASE" || exit 1
echo "### $(hostname) base $BASE ($(readlink -f "$BASE"))"
df -h "$BASE" | sed 's/^/### df /'
for t in g++ make curl tar sha256sum python3; do
  command -v $t >/dev/null || { echo "### missing tool: $t" >&2; exit 1; }
done
echo "### $(g++ --version | head -1), $(make --version | head -1)"
if [ ! -x "$CS/bin/stanc" ] || [ ! -x "$CS/bin/stansummary" ]; then
  tarball=cmdstan-$SV.tar.gz
  if [ ! -f "$tarball" ] || ! echo "$SSHA  $tarball" | sha256sum -c - >/dev/null 2>&1; then
    rm -f "$tarball"
    curl -sSfLO "$URL" || exit 1
  fi
  echo "$SSHA  $tarball" | sha256sum -c - || exit 1
  # an unfinished earlier extraction or build is redone from the tarball
  [ -d "$CS" ] && [ ! -f "$CS/.tddlog-extracted" ] && { echo "### $CS exists but is not ours; stop" >&2; exit 1; }
  if [ ! -d "$CS" ]; then
    tar xzf "$tarball" || exit 1
    : > "$CS/.tddlog-extracted"
  fi
  echo "### build cmdstan $SV with make -j$JOBS"
  (cd "$CS" && make -j"$JOBS" build) 2>&1 | grep -E '^(---|make.*Error|g\+\+.*error)|CmdStan v' | sed 's/^/### make /'
  [ -x "$CS/bin/stanc" ] && [ -x "$CS/bin/stansummary" ] || { echo "### build failed" >&2; exit 1; }
  rm -f "$tarball"
fi
echo "### $("$CS/bin/stanc" --version 2>&1 | head -1)"
# smoke test: bundled bernoulli example
T=$(mktemp -d "${TMPDIR:-/tmp}/stan-smoke.XXXXXX") || exit 1
cp "$CS/examples/bernoulli/bernoulli.stan" "$CS/examples/bernoulli/bernoulli.data.json" "$T/" || exit 1
(cd "$CS" && make -j"$JOBS" "$T/bernoulli" > "$T/compile.log" 2>&1) || { tail -20 "$T/compile.log"; echo "### smoke compile failed" >&2; exit 1; }
"$T/bernoulli" sample num_samples=1000 random seed=1 data file="$T/bernoulli.data.json" output file="$T/out.csv" > "$T/run.log" 2>&1 \
  || { tail -20 "$T/run.log"; echo "### smoke run failed" >&2; exit 1; }
python3 -I - "$T/out.csv" <<'PY' || exit 1
import csv, sys
rows = []
with open(sys.argv[1]) as f:
    hdr = None
    for line in f:
        if line.startswith('#'): continue
        r = next(csv.reader([line]))
        if hdr is None: hdr = r
        else: rows.append(dict(zip(hdr, map(float, r))))
m = sum(r['theta'] for r in rows) / len(rows)
print('### smoke: bernoulli posterior mean of theta = %.4f (n=%d)' % (m, len(rows)))
sys.exit(0 if abs(m - 0.25) < 0.05 else 'theta mean is not near 0.25')
PY
rm -f "$T/bernoulli" "$T/bernoulli.hpp" "$T/bernoulli.stan" "$T/bernoulli.data.json" "$T/out.csv" "$T/compile.log" "$T/run.log"
rmdir "$T"
du -sh "$CS" | sed 's/^/### du /'
echo "### installed: cmdstan $SV in $CS (STAN_HOME for stan.sh)"
