#!/bin/bash
# tddlog-onehot.sh: tddlog on the one-hot translation of the program
# bench.py names, as `bench.py run --strategies auto,auto+onehot
# --tddlog-arm auto+onehot=scripts/tddlog-onehot.sh` runs it. It drops
# bench.py's `--onehot` and reads $ONEHOT/NAME.pl, which
# dice2problog_onehot.py wrote, in place of NAME.pl. TDDLOG is the binary.
a=()
for x in "$@"; do
  case $x in
    --onehot) ;;
    *.pl) p="$ONEHOT/$(basename "$x")"
      [ -f "$p" ] || { echo "error: no one-hot program $p" >&2; exit 2; }
      a+=("$p") ;;
    *) a+=("$x") ;;
  esac
done
exec "$TDDLOG" "${a[@]}"
