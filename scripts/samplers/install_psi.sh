#!/bin/bash
# install_psi.sh: Psi (https://github.com/eth-sri/psi, Boost Software License
# 1.0), the exact symbolic inference system of Gehr, Misailovic and Vechev
# (CAV 2016), as a baseline for the bit-blasting paper's Psi programs.
# Psi is written in D. It is built with LDC 1.32.0 (Apache-style LDC license,
# BSD 3-clause for the LLVM-derived parts), the version Psi's own
# build-release.sh pins; the tarball is checked against the SHA-256 computed
# when it was first downloaded on 2026-10-11 (GitHub gives no digest for it)
# and Psi's published MD5 (395fb064a9f1b2383ee39d4f988612aa). Psi is pinned at
# commit ef8a9e52 (main, 2025-03-13) with its three submodules (tgehr/ast
# e343546, tgehr/sym a0d42b7, tgehr/util 0d44034). Psi's dependencies-release.sh
# and build-release.sh are not run; their steps are done here with pinned
# downloads. The build is one single-threaded ldmd2 call (about 90 s).
#
# Installs under $HOME/local/psi (PSI_BASE overrides). Idempotent: a piece
# already there is kept. Ends with a smoke test.
set -u
set -o pipefail
LDC_VERSION=1.32.0
LDC_SHA256=5a7b22cfa31568504f3f36ed0f474415c2760bf71d478aca0896a6857c00ba5a
LDC_MD5=395fb064a9f1b2383ee39d4f988612aa
PSI_COMMIT=ef8a9e52c94089a0b4e547ac3ff99db10f3517dd
AST_COMMIT=e3435465f80a3a9b732a4c322f422f74ccad59f9
SYM_COMMIT=a0d42b71eff80cecaac1339d68fd9303ce6f55cb
UTIL_COMMIT=0d4403434fc885feba9db158e1de301b0d8e6d0c
BASE=${PSI_BASE:-$HOME/local/psi}
LDC=$BASE/ldc2-$LDC_VERSION-linux-x86_64
mkdir -p "$BASE" || exit 1
cd "$BASE" || exit 1
echo "### $(hostname) base $BASE ($(readlink -f "$BASE"))"
df -h "$BASE" | sed 's/^/### df /'
if [ ! -x "$LDC/bin/ldmd2" ]; then
  tarball=ldc2-$LDC_VERSION-linux-x86_64.tar.xz
  rm -f "$tarball.part"
  curl -sSfL -o "$tarball.part" \
    "https://github.com/ldc-developers/ldc/releases/download/v$LDC_VERSION/$tarball" || exit 1
  echo "$LDC_SHA256  $tarball.part" | sha256sum -c - || exit 1
  echo "$LDC_MD5  $tarball.part" | md5sum -c - || exit 1
  tar xJf "$tarball.part" || exit 1
  rm -f "$tarball.part"
fi
"$LDC/bin/ldc2" --version | sed -n '1s/^/### /p' || exit 1   # sed reads it all: no SIGPIPE under pipefail
if [ ! -d psi-src/.git ]; then
  git clone -q https://github.com/eth-sri/psi.git psi-src || exit 1
fi
cd psi-src || exit 1
git cat-file -e "$PSI_COMMIT^{commit}" 2>/dev/null || git fetch -q origin || exit 1
git checkout -q --detach "$PSI_COMMIT" || exit 1
git submodule -q update --init || exit 1
for pair in ast:$AST_COMMIT sym:$SYM_COMMIT util:$UTIL_COMMIT; do
  d=${pair%%:*}; c=${pair#*:}
  [ "$(git -C "$d" rev-parse HEAD)" = "$c" ] || git -C "$d" checkout -q --detach "$c" || exit 1
  [ "$(git -C "$d" rev-parse HEAD)" = "$c" ] || { echo "### submodule $d not at $c"; exit 1; }
done
echo "### psi $(git log --oneline -1) ($(git log -1 --format=%ad))"
if [ ! -x psi ] || [ "$(cat .built-commit 2>/dev/null)" != "$PSI_COMMIT" ]; then
  # as build-release.sh does, with a single compiler process
  "$LDC/bin/ldmd2" -O -release -inline -boundscheck=off -J. -Jlibrary \
    *.d ast/*.d sym/*.d util/*.d -ofpsi || exit 1
  echo "$PSI_COMMIT" > .built-commit
fi
PSI=$BASE/psi-src/psi
# smoke test: a coin flip, then a continuous example with a known answer
cat > "$BASE/smoke1.psi" <<'EOF'
def main(){ x := flip(0.3); return x; }
EOF
cat > "$BASE/smoke2.psi" <<'EOF'
def main(){ a := gauss(0,1); cobserve(gauss(a,1),1); cobserve(gauss(a,1),2); return a; }
EOF
out=$(timeout 60 "$PSI" "$BASE/smoke1.psi") || exit 1
echo "### smoke1: $out"
case "$out" in *"3/10"*"7/10"*) ;; *) echo "### smoke1 failed"; exit 1;; esac
out=$(timeout 60 "$PSI" --expectation "$BASE/smoke2.psi") || exit 1
echo "### smoke2: $out"
case "$out" in *"= 1"*) ;; *) echo "### smoke2 failed (posterior mean of a should be 1)"; exit 1;; esac
echo "### installed: psi $PSI (ldc $LDC)"
