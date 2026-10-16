#!/bin/bash
# WM-4 test evaluations (GOAL.md 9.2), each once, from the pre-registration commit (_setup/prereg_sha, METRICS.md part
# b): every script refuses to overwrite its output file. Kinds:
#   base            obs baselines, val selection, obs test cases (effects4.py base -> obs_base/obs_base.json)
#   ablation RUN    a wm3_ctrl / wm3_os seed: WM-3's test effects, split timing, factual T2 (WM-3's finals)
#   haz K           wm4_haz_sK: test effects (V3 shift), split timing (V4); test calibration of it and of wm3_full_sK
#   m1obs RUN       an M1b-obs run: obs test effects, factual T2 168 h on obs test
#   ref RUN_PATH    a WM-2 / WM-3 M1 checkpoint on the obs test (effects only)
#   causal RUN      a CRN / CT run: obs test effects (obs-trained) or v2 test effects (v2-trained)
# usage: finals4.sh KIND ARG
set -u
KIND=$1; ARG=${2:-}
W=/home/hyunlord/work/wm1-runs/wm4
R=/home/hyunlord/work/wm1-runs
SHA=$(cat $W/_setup/prereg_sha)
D=$(bash $W/_setup/code.sh $SHA) || exit 9
source /home/hyunlord/work/wm1-venv/bin/activate
cd $D
export WM1_CODE_COMMIT=$SHA WM3_CODE_COMMIT=$SHA WM4_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data
LOG=$W/_queue/finals.log
mkdir -p $W/_queue
BASE=$R/wm2/effects_base/effects_base.json
OBSB=$W/obs_base/obs_base.json
POL=/home/hyunlord/work/worldsim-wm-data/obs/policy/policy.json
one() {   # one evaluation: OUT, MARKER (the file it writes), then the command
  local out=$1 marker=$2; shift 2
  mkdir -p $out
  if [ -e $out/$marker ]; then echo "$(date +%F_%T) $out/$marker exists" >> $LOG; return; fi
  local tag=${marker%.json}
  date +%s > $out/$tag.start
  "$@" > $out/$tag.out 2>&1; echo $? > $out/$tag.exit; date +%s > $out/$tag.end
  echo "$(date +%F_%T) $out/$marker exit=$(cat $out/$tag.exit)" >> $LOG
}
case $KIND in
base)
  one $W/obs_base obs_base.json python3 tools/worldmodel/wm4/effects4.py base --base $BASE --policy $POL --out $W/obs_base ;;
ablation)
  [ "$(cat $W/$ARG/train.exit)" = "0" ] || { echo "$(date +%F_%T) $ARG train exit != 0: no finals" >> $LOG; exit 1; }
  one $W/${ARG}_effects effects.json python3 tools/worldmodel/wm3/effects3.py model --run $W/$ARG --base $BASE --out $W/${ARG}_effects
  one $W/${ARG}_split split.json python3 tools/worldmodel/wm3/effects3.py split --run $W/$ARG --out $W/${ARG}_split
  one $W/${ARG}_fact metrics.json python3 tools/worldmodel/wm3/final_fact.py --run $W/$ARG --out $W/${ARG}_fact ;;
haz)
  H=$W/wm4_haz_s$ARG
  one ${H}_effects effects.json python3 tools/worldmodel/wm3/effects3.py model --run $H --base $BASE --out ${H}_effects
  one ${H}_split split.json python3 tools/worldmodel/wm3/effects3.py split --run $H --out ${H}_split
  one ${H}_calib calib_test.json python3 tools/worldmodel/wm4/recal_hazard.py calib --run $H --base $BASE --role test --out ${H}_calib
  one $W/wm3_full_s${ARG}_calib calib_test.json python3 tools/worldmodel/wm4/recal_hazard.py calib --run $R/wm3/wm3_full_s$ARG \
    --base $BASE --role test --out $W/wm3_full_s${ARG}_calib ;;
m1obs)
  [ "$(cat $W/$ARG/train.exit)" = "0" ] || { echo "$(date +%F_%T) $ARG train exit != 0: no finals" >> $LOG; exit 1; }
  one $W/${ARG}_effects effects.json python3 tools/worldmodel/wm4/effects4.py m1 --run $W/$ARG --obs-base $OBSB --out $W/${ARG}_effects
  one $W/${ARG}_fact metrics.json python3 tools/worldmodel/wm4/m1_obs.py final --run $W/$ARG --out $W/${ARG}_fact ;;
ref)
  N=$(basename $ARG)
  one $W/ref_${N}_obs effects.json python3 tools/worldmodel/wm4/effects4.py m1 --run $ARG --obs-base $OBSB --out $W/ref_${N}_obs ;;
causal)
  [ "$(cat $W/$ARG/train.exit)" = "0" ] || { echo "$(date +%F_%T) $ARG train exit != 0: no finals" >> $LOG; exit 1; }
  if [[ $ARG == *_obs_* ]]; then
    one $W/${ARG}_effects effects.json python3 tools/worldmodel/wm4/causal.py effects --run $W/$ARG --split obs --cases $OBSB --out $W/${ARG}_effects
  else
    one $W/${ARG}_effects effects.json python3 tools/worldmodel/wm4/causal.py effects --run $W/$ARG --split v2 --cases $BASE --out $W/${ARG}_effects
  fi ;;
*) echo "unknown kind $KIND"; exit 2 ;;
esac
