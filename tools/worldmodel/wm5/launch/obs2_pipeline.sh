#!/bin/bash
# WM-5 Part 1 driver (GOAL.md 6.2): after round 0 (record_obs2.sh r0), every round until one adds no cast (at most 5
# recording rounds), finalize (observed episodes, policy.json = replay), the families, the unforked check of 3 sampled
# test worlds, then the manifest and DATASET_OBS2's numbers (train + val only). WORKERS 4 (beside the trainings).
# usage: obs2_pipeline.sh CODE_SHA
set -u
SHA=$1
W=/home/hyunlord/work/wm1-runs/wm5
L=$W/_record
D=/home/hyunlord/work/worldsim-wm-data/obs2
PY=/home/hyunlord/work/wm1-venv/bin/python
C=$(bash $W/_setup/code.sh $SHA) || exit 9
R=$C/tools/worldmodel/wm5/launch/record_obs2.sh
export WORKERS=${WORKERS:-4}
while [ "$(cat $L/r0_verify.exit 2>/dev/null)" != "0" ]; do
  [ -e $L/r0.exit ] && [ "$(cat $L/r0.exit)" != "0" ] && { echo "round 0 failed"; exit 3; }
  [ -e $L/r0_verify.exit ] && { echo "round 0 verify failed"; exit 3; }
  sleep 60
done
echo "$(date +%F_%T) round 0 verified"
for K in 1 2 3 4 5 6; do
  bash $R round $SHA $K || exit $?
  N=$($PY -c "import json;print(json.load(open('$D/policy/round_$K.json'))['new_casts'])")
  echo "$(date +%F_%T) round $K: $N new casts"
  [ "$N" = "0" ] && break
done
bash $R finalize $SHA || exit $?
bash $R families $SHA || exit $?
UNFORKED_SEEDS="$($PY $C/tools/worldmodel/wm5/obs2.py unforked-seeds --root $D)" bash $R unforked $SHA || exit $?
cd $C
export WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data WM1_CODE_COMMIT=$SHA
$PY tools/worldmodel/wm5/obs2.py manifest --root $D --out $W/obs2_manifest/obs2_manifest.json > $L/manifest.log 2>&1; echo $? > $L/manifest.exit
echo "$(date +%F_%T) manifest exit $(cat $L/manifest.exit)"
echo "$(date +%F_%T) obs2 pipeline done"
