#!/bin/bash
# WM-3 Part 1 audit (GOAL.md 6) on the three WM-2 M1b-act checkpoints; train and val families only.
# usage: audit.sh SHA
set -u
SHA=$1
W=/home/hyunlord/work/wm1-runs
D=$(bash $W/wm3/_setup/code.sh $SHA) || exit 9
source /home/hyunlord/work/wm1-venv/bin/activate
cd $D
export WM1_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data
python3 tools/worldmodel/wm3/audit.py --run $W/wm2/wm2_m1bact_v19_s0 $W/wm2/wm2_m1bact_v19_s1 $W/wm2/wm2_m1bact_v19_s2 \
  --out $W/wm3/audit
