#!/bin/bash
# webppl.sh PROGRAM METHOD SAMPLES SEED: run one WebPPL program of the
# bit-blasting artifact and print `estimate NAME VALUE` and the program's own
# timeit wall time as `sampler_seconds S`.
#   PROGRAM  a .wppl file of the artifact (baselines/webppl/<bench>/<bench>.wppl,
#            or_N/or_N.wppl); NAME = its basename without .wppl
#   METHOD   rejection | MCMC | SMC   (the artifact's argv.m; MCMC is WebPPL's
#            default Metropolis-Hastings kernel, no kernel option is given)
#   SAMPLES  the artifact's argv.s, used as both `samples` and `particles`
# Environment: WEBPPL_HOME (default $HOME/local/webppl), NODE (default node),
# WEBPPL_TIMEOUT seconds (default 120; 0 = none).
set -u -o pipefail
[ $# -eq 4 ] || { echo "usage: $0 PROGRAM METHOD SAMPLES SEED" >&2; exit 2; }
PROG=$1; METHOD=$2; SAMPLES=$3; SEED=$4
WEBPPL_HOME=${WEBPPL_HOME:-$HOME/local/webppl}
NODE=${NODE:-node}
[ -f "$PROG" ] || { echo "### no program $PROG" >&2; exit 2; }
case "$METHOD" in rejection|MCMC|SMC) ;; *) echo "### METHOD must be rejection, MCMC or SMC" >&2; exit 2;; esac
NAME=$(basename "$PROG" .wppl)
T=${WEBPPL_TIMEOUT:-120}
[ -f "$WEBPPL_HOME/node_modules/webppl/webppl" ] || { echo "### WebPPL not installed in $WEBPPL_HOME" >&2; exit 2; }
if [ "$T" -gt 0 ]; then TO=(timeout "$T"); else TO=(); fi
OUT=$("${TO[@]}" "$NODE" "$WEBPPL_HOME/node_modules/webppl/webppl" "$PROG" \
        --require "$WEBPPL_HOME/node_modules/webppl-timeit" --random-seed "$SEED" \
        -- --s "$SAMPLES" --m "$METHOD" 2>&1)
RC=$?
if [ $RC -ne 0 ]; then
  echo "$OUT" | tail -15 >&2
  echo "### WEBPPL FAILED (rc=$RC; 124 = timeout) $NAME $METHOD $SAMPLES" >&2
  exit 1
fi
# timeit prints { value: V, runtimeInMilliseconds: MS }
VAL=$(echo "$OUT" | tr '\n' ' ' | sed -n 's/.*value: \([^,}]*\),.*/\1/p')
MS=$(echo "$OUT" | tr '\n' ' ' | sed -n 's/.*runtimeInMilliseconds: \([0-9.e+-]*\).*/\1/p')
if [ -z "$VAL" ]; then
  echo "$OUT" | tail -15 >&2
  echo "### no timeit result in the output of $NAME" >&2; exit 1
fi
echo "estimate $NAME.$METHOD $VAL"
echo "sampler_seconds $(awk -v ms="$MS" 'BEGIN{printf "%.3f", ms/1000}')"
