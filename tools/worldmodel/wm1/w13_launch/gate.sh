#!/bin/bash
# usage: gate.sh <sha>   (runs the unittest discovery in the DGX code clone at <sha>)
set -u
SHA=$1
cd /home/hyunlord/work/new-world-wm1 && git fetch -q origin && git checkout -q --detach "$SHA" || exit 9
OUT=/home/hyunlord/work/wm1-runs/gate_${SHA:0:8}
mkdir -p "$OUT"
source /home/hyunlord/work/wm1-venv/bin/activate
export WM_BIN=/home/hyunlord/work/wm1-runs/_build/src_wm13/rust/target/release/worldsim_wm
export PROBE_BIN=/home/hyunlord/work/wm1-runs/_build/src_wm13/rust/target/release/worldsim_probe
export WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data
export WM1_CODE_COMMIT=$(git rev-parse HEAD)
date +%s > "$OUT/gate.start"
python3 -m unittest discover -s tools/worldmodel -p 'test_*.py' -v > "$OUT/gate.txt" 2>&1
echo $? > "$OUT/gate.exit"
date +%s > "$OUT/gate.end"
