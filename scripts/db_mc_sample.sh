#!/bin/bash
# A sample of the TPC-H suite through ProvSQL and of the weather factory
# through Storm and PRISM, on a box, after install_provsql.sh and
# install_modelcheckers.sh:
#   bash scripts/db_mc_sample.sh [N [TIMEOUT [CPUS]]]
# N instances of each (every k-th by name; default 8), TIMEOUT seconds (120),
# CPUS the pinned lanes (default the last four of the box). The tables compare
# each system's answers with the reference (`--exact`). Results in tmp/db-mc-sample.
set -u
N=${1:-8}; TO=${2:-120}
NC=$(nproc); CPUS=${3:-$((NC - 4)),$((NC - 3)),$((NC - 2)),$((NC - 1))}
OUT=$PWD/tmp/db-mc-sample; rm -rf "$OUT"; mkdir -p "$OUT"
export TMPDIR=$OUT
PS=$HOME/local/code/provsql; MC=$HOME/local/code/modelcheckers
B="python3 scripts/bench.py"
sample() { # dir [regex]: N names evenly spread
  ls "$1"/*.pl | xargs -n1 basename | sed 's/\.pl$//' | grep -E "${2:-.}" | sort | awk -v n="$N" '{a[NR]=$0} END {k=int(NR/n); if (k<1) k=1; for (i=1; i<=NR; i+=k) printf "%s%s", (c++ ? "," : ""), a[i]}'
}
run() { # tag dir regex bench.py args...
  local tag=$1 dir=$2 re=$3; shift 3
  $B gen --dir "$dir" "$@" >/dev/null 2>"$OUT/gen.$tag.err" || { echo "### gen $tag failed"; tail -3 "$OUT/gen.$tag.err"; return; }
  local names; names=$(sample "$dir" "$re"); echo "### $tag: $names"
  $B run --dir "$dir" --only "$names" --cpus "$CPUS" --timeout "$TO" --no-skip --exact --kbs '' --strategies '' "$@" \
    "${EXTRA[@]}" --out "$OUT/$tag.jsonl" >/dev/null 2>"$OUT/run.$tag.err" || { echo "### run $tag failed"; tail -3 "$OUT/run.$tag.err"; }
  $B table "$OUT/$tag.jsonl" 2>&1 | cut -c1-230
}
EXTRA=(--provsql "$PS")
run tpch "$OUT/bench-tpch" . --suite tpch
EXTRA=(--storm "$MC/venv/bin/python" --prism "$MC/prism-4.8.1-linux64-x86" --weather-models "$MC/weatherfactory")
run weather "$OUT/bench-geni" 'weather' --suite geni
