#!/bin/bash
# stan.sh MODEL_DIR SAMPLES SEED: compile (once) and run one Stan program
# of the bit-blasting artifact with CmdStan, print the posterior mean of the
# queried quantity as `estimate NAME VALUE` and the sampler's own wall time
# as `sampler_seconds warmup W sampling S`.
#   MODEL_DIR  directory holding NAME.stan (NAME = its basename unless
#              STAN_NAME is set, e.g. STAN_NAME=or_5 in the `or` directory)
#              and, if the model has data, NAME.data.R (or DIRNAME.data.R,
#              or STAN_DATA=file).
# Environment: STAN_HOME  CmdStan tree (default $HOME/local/stan/cmdstan-2.32.2)
#              STAN_WORK  where models are built and outputs go
#                         (default ./stan-work); the artifact's tree is never written
#              WARMUP (default 1000), CHAINS (default 1), STAN_EXTRA (more
#              sampler arguments, e.g. "algorithm=hmc engine=nuts")
# One compile at a time: do not run two compiles of the same model at once.
set -u -o pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
[ $# -eq 3 ] || { echo "usage: $0 MODEL_DIR SAMPLES SEED" >&2; exit 2; }
DIR="$(cd "$1" && pwd)" || exit 2
SAMPLES=$2; SEED=$3
STAN_HOME=${STAN_HOME:-$HOME/local/stan/cmdstan-2.32.2}
WORK=${STAN_WORK:-$PWD/stan-work}
WARMUP=${WARMUP:-1000}; CHAINS=${CHAINS:-1}
NAME=${STAN_NAME:-$(basename "$DIR")}
[ -f "$DIR/$NAME.stan" ] || { echo "### no $DIR/$NAME.stan" >&2; exit 2; }
[ -x "$STAN_HOME/bin/stanc" ] || { echo "### CmdStan not built in $STAN_HOME" >&2; exit 2; }
DATA=${STAN_DATA:-}
if [ -z "$DATA" ]; then
  for c in "$DIR/$NAME.data.R" "$DIR/$(basename "$DIR").data.R"; do
    [ -f "$c" ] && { DATA=$c; break; }
  done
fi
mkdir -p "$WORK/$NAME" || exit 1
# copied only when it differs, so that the model is built once
cmp -s "$DIR/$NAME.stan" "$WORK/$NAME/$NAME.stan" || cp "$DIR/$NAME.stan" "$WORK/$NAME/$NAME.stan" || exit 1
BIN=$WORK/$NAME/$NAME
if [ ! -x "$BIN" ] || [ "$WORK/$NAME/$NAME.stan" -nt "$BIN" ]; then
  echo "### compile $NAME"
  (cd "$STAN_HOME" && nice make -j"${JOBS:-1}" "$BIN" > "$WORK/$NAME/compile.log" 2>&1) \
    || { echo "### COMPILE FAILED, see $WORK/$NAME/compile.log" >&2; tail -20 "$WORK/$NAME/compile.log" >&2; exit 1; }
fi
OUT=$WORK/$NAME/out_${SEED}.csv
echo "### run $NAME samples=$SAMPLES warmup=$WARMUP chains=$CHAINS seed=$SEED data=${DATA:-none}"
ARGS=(sample num_samples="$SAMPLES" num_warmup="$WARMUP" ${STAN_EXTRA:-} random seed="$SEED")
[ -n "$DATA" ] && ARGS+=(data file="$DATA")
if [ "$CHAINS" -eq 1 ]; then
  "$BIN" "${ARGS[@]}" output file="$OUT" > "$WORK/$NAME/run_${SEED}.log" 2>&1 \
    || { echo "### RUN FAILED, see $WORK/$NAME/run_${SEED}.log" >&2; tail -20 "$WORK/$NAME/run_${SEED}.log" >&2; exit 1; }
  CSV=$OUT
else
  echo "### CHAINS>1 not supported by this sketch" >&2; exit 2
fi
python3 -I "$HERE/stan_estimates.py" "$CSV" "$NAME"
