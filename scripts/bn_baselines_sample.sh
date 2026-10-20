#!/bin/bash
# A sample of every network suite through every UAI baseline, on a box, after
# the installer (bash scripts/bn_baselines_dry.sh builds the drivers):
#   bash scripts/bn_baselines_sample.sh [N [TIMEOUT [CPUS]]]
# N instances of each suite (every k-th by name; default 8), TIMEOUT seconds
# (60), CPUS the pinned lanes (default the last four of the box). Merlin,
# libDAI, pyAgrum, pgmpy and TensorInference.jl run in the same lanes, tddlog
# not; the tables compare each against the reference (`--exact`, Merlin).
# Reads the packages in ~/ace-pkg and ~/uai-pkg and fetches the large
# networks (data/fetch.txt). PR=1 runs the PR task (log10 Z) on the tuning
# PR set instead. Ends with sample_check.py and sample_times.py. Results in the checkout's tmp/bn-sample.
set -u
N=${1:-8}; TO=${2:-60}
NC=$(nproc); CPUS=${3:-$((NC - 4)),$((NC - 3)),$((NC - 2)),$((NC - 1))}
P=$HOME/local/code/bn-baselines
OUT=$PWD/tmp/bn-sample; mkdir -p "$OUT" && cd "$OUT/../.." || exit 1
rm -rf "$OUT"/*; mkdir -p "$OUT/pkg"
export TMPDIR=$OUT
(cd "$OUT/pkg" && unzip -q "$HOME/ace-pkg/benchmarks_v1.0.zip" && mkdir mar final tune &&
  (cd mar && unzip -q "$HOME/uai-pkg/MAR_tune.zip") && (cd tune && unzip -q "$HOME/uai-pkg/PR_tune.zip")) || exit 1
ACEB=$(dirname "$(find "$OUT/pkg" -maxdepth 3 -type d -name '2004-pgm' | head -1)")
MERLIN=$HOME/uai-pkg/merlin
SYS=(--kbs '' --strategies '' --merlin "$MERLIN" --libdai "$P/bin/libdai_uai" --agrum "$P/venv/bin/python"
     --pgmpy "$P/venv/bin/python"
     --tensorinference "$HOME/local/alea/julia-1.11.9/bin/julia --startup-file=no --threads=1 --project=$P/TensorInference")
export JULIA_DEPOT_PATH=$P/julia-depot
B="python3 scripts/bench.py"
"$P/venv/bin/pip" list 2>/dev/null | grep -iE '^(pyagrum|pgmpy|numpy) ' | sed 's/^/### pip: /'
grep -A2 'name = "TensorInference"' "$P/TensorInference/Manifest.toml" | grep version | sed 's/^/### TensorInference.jl: /'
sample() { # dir: N names of the instances in dir, evenly spread, comma separated
  ls "$1"/*.pl | xargs -n1 basename | sed 's/\.pl$//' | sort | awk -v n="$N" '{a[NR]=$0} END {k=int(NR/n); if (k<1) k=1; for (i=1; i<=NR; i+=k) printf "%s%s", (c++ ? "," : ""), a[i]}'
}
run() { # tag, dir, bench.py gen args...
  local tag=$1 dir=$2; shift 2
  $B gen --dir "$dir" "$@" >/dev/null 2>"$OUT/gen.$tag.err" || { echo "### gen $tag failed"; tail -3 "$OUT/gen.$tag.err"; return; }
  local names; names=$(sample "$dir")
  echo "### $tag: $names"
  $B run --dir "$dir" --only "$names" --cpus "$CPUS" --timeout "$TO" --no-skip --exact "${SYS[@]}" "$@" --out "$OUT/$tag.jsonl" >/dev/null 2>"$OUT/run.$tag.err" \
    || { echo "### run $tag failed"; tail -3 "$OUT/run.$tag.err"; }
  $B table "$OUT/$tag.jsonl" 2>&1 | cut -c1-220
}
if [ "${PR:-0}" = 1 ]; then  # log10 Z on the tuning PR set (and the MAR set's networks)
  run uaipr "$OUT/bench-uaipr" --suite uai --pr --uai-benchmarks "$OUT/pkg/tune/PR"
  run uaimarpr "$OUT/bench-uaimarpr" --suite uai --pr --uai-benchmarks "$OUT/pkg/mar/MAR"
else
  run std "$OUT/bench" --families bn,bnev
  run large "$OUT/bench-large" --suite large
  run ace "$OUT/bench-ace" --suite ace --ace-benchmarks "$ACEB"
  run uai "$OUT/bench-uai" --suite uai --uai-benchmarks "$OUT/pkg/mar/MAR"
fi
python3 scripts/sample_check.py "$OUT"
python3 scripts/sample_times.py "$OUT"
