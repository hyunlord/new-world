#!/bin/bash
# WM-3 evaluations of one fine-tuned run (METRICS.md), each once, from the pre-registration commit (_setup/prereg_sha):
# test effects (V1, V2, V3, V5), test split timing (V4), factual T2 on v2_v19_fact test (V6); validation diagnostics:
# the lag curve on the first 40 val families and, for a hazard run, the hazard calibration.
# usage: finals.sh RUN
set -u
RUN=$1
W=/home/hyunlord/work/wm1-runs/wm3
R=/home/hyunlord/work/wm1-runs
SHA=$(cat $W/_setup/prereg_sha)
D=$(bash $W/_setup/code.sh $SHA) || exit 9
source /home/hyunlord/work/wm1-venv/bin/activate
cd $D
export WM1_CODE_COMMIT=$SHA WM3_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data
LOG=$W/_queue/finals.log
one() {   # one evaluation: OUT, MARKER (the file it writes), then the command
  local out=$1 marker=$2; shift 2
  mkdir -p $out
  if [ -e $out/$marker ]; then echo "$(date +%F_%T) $out/$marker exists" >> $LOG; return; fi
  date +%s > $out/final.start
  "$@" > $out/final.out 2>&1; echo $? > $out/final.exit; date +%s > $out/final.end
  echo "$(date +%F_%T) $out exit=$(cat $out/final.exit)" >> $LOG
}
[ "$(cat $W/$RUN/train.exit)" = "0" ] || { echo "$(date +%F_%T) $RUN train exit != 0: no finals" >> $LOG; exit 1; }
BASE=$R/wm2/effects_base/effects_base.json
one $W/${RUN}_effects effects.json python3 tools/worldmodel/wm3/effects3.py model --run $W/$RUN --base $BASE --out $W/${RUN}_effects
one $W/${RUN}_split split.json python3 tools/worldmodel/wm3/effects3.py split --run $W/$RUN --out $W/${RUN}_split
one $W/${RUN}_fact metrics.json python3 tools/worldmodel/wm3/final_fact.py --run $W/$RUN --out $W/${RUN}_fact
one $W/diag effect_lag_$RUN.json python3 tools/worldmodel/wm3/diag_lag.py --run $W/$RUN --worlds 40 --out $W/diag/effect_lag_$RUN.json
HZ=$(python3 -c "import json;print(int(float(json.load(open('$W/$RUN/config.json'))['hyper']['hazard'])>0))")
if [ "$HZ" = "1" ]; then
  one $W/${RUN}_calib calib.json python3 tools/worldmodel/wm3/effects3.py calib --run $W/$RUN --base $BASE --out $W/${RUN}_calib
fi
