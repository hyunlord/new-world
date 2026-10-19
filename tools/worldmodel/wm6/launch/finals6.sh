#!/bin/bash
# WM-6 test evaluations (GOAL.md 9.2, 9.3), each once, after the pre-registration (METRICS.md; _setup/prereg_sha must
# exist, else refused) from SHA (the pre-registration commit or a later one that changes no rule; recorded in each
# output's code_commit). The v2 test features (_prep/test.npz) are prepared here, on the first call, and nowhere else.
#   finals6.sh SHA RUN...          v2 test → RUN/effects.json         (runs whose train.exit is 0)
#   OBS2=1 finals6.sh SHA RUN...   obs2 test r0 → RUN/effects_obs2.json (WM-5's obs2_base.json)
# Every evaluation logs RUN/eval{,_obs2}.{start,end,exit,log}; an existing output is never overwritten
# (effects6.py evaluate refuses it too). GBM runs use the side venv (DECISIONS L2.7).
set -u
export OPENBLAS_NUM_THREADS=1 OMP_NUM_THREADS=1      # multi-threaded OpenBLAS stalls on the shared DGX (DECISIONS L2.9)
SHA=$1; shift
W6=/home/hyunlord/work/wm1-runs/wm6
R=/home/hyunlord/work/wm1-runs
PY=/home/hyunlord/work/wm1-venv/bin/python
GPY=/home/hyunlord/work/wm6-gbm-venv/bin/python
BASE=$R/wm2/effects_base/effects_base.json
O2BASE=$R/wm5/obs2_base/obs2_base.json
[ -e $W6/_setup/prereg_sha ] || { echo "$(date +%F_%T) no _setup/prereg_sha: test evaluations refused"; exit 3; }
D=$(bash $W6/_setup/code.sh $SHA) || exit 9
cd $D
export WM1_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data
if [ ! -e $W6/_prep/test.npz ]; then
  date +%s > $W6/_logs/prep_test.start
  nice -n 10 $PY tools/worldmodel/wm6/effects6.py prepare --part test --base $BASE --out $W6/_prep > $W6/_logs/prep_test.log 2>&1
  echo $? > $W6/_logs/prep_test.exit; date +%s > $W6/_logs/prep_test.end
  [ "$(cat $W6/_logs/prep_test.exit)" = 0 ] || { echo "prepare test failed"; exit 4; }
fi
TAG=""; EXTRA=""; OUTF=effects.json
if [ "${OBS2:-0}" = 1 ]; then TAG="_obs2"; EXTRA="--obs2-base $O2BASE --root $WM_DATA_ROOT"; OUTF=effects_obs2.json; fi
for RUN in "$@"; do
  RD=$W6/$RUN
  [ -e $RD/$OUTF ] && { echo "$RUN: $OUTF exists, skipped"; continue; }
  [ "$(cat $RD/train.exit 2>/dev/null)" = 0 ] || { echo "$RUN: not finished (train.exit $(cat $RD/train.exit 2>/dev/null)), skipped"; continue; }
  P=$PY; grep -q '"model": "gbm"' $RD/config.json && P=$GPY
  date +%s > $RD/eval$TAG.start
  nice -n 10 $P tools/worldmodel/wm6/effects6.py evaluate --run $RD --prep $W6/_prep --base $BASE $EXTRA > $RD/eval$TAG.log 2>&1
  echo $? > $RD/eval$TAG.exit; date +%s > $RD/eval$TAG.end
  echo "$(date +%F_%T) $RUN eval$TAG exit $(cat $RD/eval$TAG.exit)"
done
