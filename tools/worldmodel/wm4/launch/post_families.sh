#!/bin/bash
# WM-4 Part 1 (GOAL.md 6.2.6): the obs manifest (every episode verified) and DATASET_OBS's numbers on train + val only
# (`effects4.py describe`: policy summary, scales, pooled counts and ESS, the val families' true effect distribution).
# usage: post_families.sh SHA
set -u
SHA=$1
W=/home/hyunlord/work/wm1-runs/wm4
D=$(bash $W/_setup/code.sh $SHA) || exit 9
source /home/hyunlord/work/wm1-venv/bin/activate
cd $D
export WM1_CODE_COMMIT=$SHA WM4_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data
O=/home/hyunlord/work/worldsim-wm-data/obs
python3 tools/worldmodel/wm4/obs_split.py manifest --root $O --policy $O/policy/policy.json --out $W/obs_manifest/obs_manifest.json \
  > $W/_logs/manifest.out 2>&1; echo "manifest exit $?"
python3 tools/worldmodel/wm4/effects4.py describe --base /home/hyunlord/work/wm1-runs/wm2/effects_base/effects_base.json \
  --policy $O/policy/policy.json --out $W/describe > $W/_logs/describe.out 2>&1; echo "describe exit $?"
