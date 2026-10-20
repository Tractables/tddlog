#!/usr/bin/env bash
# check-versions.sh [TAG]: the crate and the two bindings carry one version,
# and a release tag, where given, names that version. A tag whose version had
# not been bumped would build a wheel labelled with the previous release and
# publish it to PyPI, where a version can be yanked but never replaced.
set -euo pipefail
cd "$(dirname "$0")/.."

version() { sed -n 's/^version = "\(.*\)"$/\1/p' "$1" | head -n 1; }

core=$(version Cargo.toml)
status=0
for manifest in bindings/python/Cargo.toml bindings/c/Cargo.toml; do
  found=$(version "$manifest")
  if [ "$found" != "$core" ]; then
    echo "$manifest is version $found, but the crate is $core"
    status=1
  fi
done
tag=${1:-}
if [ -n "$tag" ] && [ "$tag" != "v$core" ]; then
  echo "the tag $tag does not name the crate's version $core"
  status=1
fi
exit $status
