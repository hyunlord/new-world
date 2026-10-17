#!/bin/bash
# WM-4 9.4: Platt recalibration of wm3_full_s{0,1,2}'s hazard heads (train-family rollouts, val Brier; no test data).
# usage: recal.sh SHA
set -u
SHA=$1
W=/home/hyunlord/work/wm1-runs
D=$(bash $W/wm4/_setup/code.sh $SHA) || exit 9
source /home/hyunlord/work/wm1-venv/bin/activate
cd $D
export WM1_CODE_COMMIT=$SHA WM4_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data
for S in 0 1 2; do
  python3 tools/worldmodel/wm4/recal_hazard.py fit --run $W/wm3/wm3_full_s$S --base $W/wm2/effects_base/effects_base.json \
    --out $W/wm4/wm4_haz_s$S > $W/wm4/_logs/recal_s$S.out 2>&1
  echo "s$S exit $?"
done
