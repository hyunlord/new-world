#!/bin/bash
# WM-3 gate, DGX half (GOAL.md 4.5): unittest discovery in the venv at SHA, zero skips expected.
# usage: gate.sh SHA
set -u
SHA=$1
W=/home/hyunlord/work/wm1-runs
D=$(bash $W/wm3/_setup/code.sh $SHA) || exit 9
OUT=$W/wm3/gate_${SHA:0:8}
mkdir -p "$OUT"
source /home/hyunlord/work/wm1-venv/bin/activate
cd $D
export WM_BIN=$W/_build/src_wm2/rust/target/release/worldsim_wm
export PROBE_BIN=$W/_build/src_wm2/rust/target/release/worldsim_probe
export WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data
export WM1_CODE_COMMIT=$SHA WM2_CODE_COMMIT=$SHA
date +%s > "$OUT/gate.start"
python3 -m unittest discover -s tools/worldmodel -p 'test_*.py' -v > "$OUT/gate.txt" 2>&1
echo $? > "$OUT/gate.exit"
date +%s > "$OUT/gate.end"
