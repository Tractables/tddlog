#!/bin/bash
# The network baselines' unit tests (tests/test_bn_baselines.py) on a box,
# after the installer: bash scripts/bn_baselines_dry.sh INSTALLER_DIR
# (the directory of install_bn_baselines.sh).
set -u
D=${1:?usage: bn_baselines_dry.sh INSTALLER_DIR, the directory of install_bn_baselines.sh}
export LIBDAI_SRC=$PWD/scripts/libdai_uai.cpp
bash "$D/install_bn_baselines.sh" || exit 1
P=$HOME/local/code/bn-baselines
export BN_PYTHON=$P/venv/bin/python BN_LIBDAI=$P/bin/libdai_uai
export JULIA_DEPOT_PATH=$P/julia-depot
export BN_TENSORINFERENCE="$HOME/local/alea/julia-1.11.9/bin/julia --startup-file=no --threads=1 --project=$P/TensorInference"
export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1
"$P/venv/bin/python" -m unittest -v tests.test_bn_baselines 2>&1 | grep -v Warning > "$P/dry.log"; grep -E "^(FAIL|ERROR|OK|Ran|[A-Za-z.]*Error|AssertionError)|^    [a-z]" "$P/dry.log" | cut -c1-260 | head -${DRY_LINES:-40}
