#!/bin/bash
# WM-5 test evaluations (GOAL.md 9.3–9.5), each once, from the pre-registration commit (_setup/prereg_sha, METRICS.md
# part b) or the commit in FINALS_SHA (a later commit that changes no rule; recorded in each output's code_commit):
# every script refuses to overwrite its output file. Kinds:
#   rescore RUN     a WM-4 CRN / CT run (crn_obs_s0 …): its WM-4 test split re-scored with rounding (D2)
#   describe        DATASET_OBS2's numbers (train + val only)
#   base            obs2 cases, baselines, val selection, test predictions, floor (effects5.py base)
#   ridge [priv]    ridge_obs / ridge_priv (v2 train fit, v2 val λ, v2 and obs2 test predictions)
#   causal RUN      a WM-5 CRN / CT run: v2 test (v2-trained) or obs2 test (obs2-trained), last and best checkpoints
#   causalref RUN   a WM-4 CRN / CT obs run on the obs2 test (as kept; rounded and unrounded)
#   m1obs2 RUN      an M1b-obs2 run: obs2 test effects, factual T2 on the obs2 test
#   ref RUN_PATH    a WM-2 / WM-3 / WM-4 M1 checkpoint on the obs2 test (effects only)
# usage: finals5.sh KIND ARG
set -u
KIND=$1; ARG=${2:-}
W=/home/hyunlord/work/wm1-runs/wm5
R=/home/hyunlord/work/wm1-runs
SHA=${FINALS_SHA:-$(cat $W/_setup/prereg_sha)}
D=$(bash $W/_setup/code.sh $SHA) || exit 9
source /home/hyunlord/work/wm1-venv/bin/activate
cd $D
export WM1_CODE_COMMIT=$SHA WM3_CODE_COMMIT=$SHA WM4_CODE_COMMIT=$SHA WM5_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data
LOG=$W/_queue/finals.log
mkdir -p $W/_queue
BASE=$R/wm2/effects_base/effects_base.json
OBSB=$R/wm4/obs_base/obs_base.json
O2B=$W/obs2_base/obs2_base.json
C5=tools/worldmodel/wm5/causal5.py
E5=tools/worldmodel/wm5/effects5.py
one() {   # one evaluation: OUT, MARKER (the file it writes), then the command; up to 3 attempts 5 min apart on a CUDA
  local out=$1 marker=$2; shift 2   # context failure (GOAL.md 1)
  mkdir -p $out
  if [ -e $out/$marker ]; then echo "$(date +%F_%T) $out/$marker exists" >> $LOG; return; fi
  local tag=${marker%.json}
  for attempt in 1 2 3; do
    date +%s > $out/$tag.start
    "$@" > $out/$tag.out 2>&1; echo $? > $out/$tag.exit; date +%s > $out/$tag.end
    echo "$(date +%F_%T) $out/$marker exit=$(cat $out/$tag.exit) attempt=$attempt load=$(cut -d' ' -f1-3 /proc/loadavg)" >> $LOG
    [ "$(cat $out/$tag.exit)" = "0" ] && return
    grep -q "CUDA_ERROR_OUT_OF_MEMORY\|cuDevicePrimaryCtxRetain\|CUDA error: out of memory" $out/$tag.out || return
    cp $out/$tag.out $out/$tag.out.attempt$attempt; sleep 300
  done
}
case $KIND in
rescore)
  if [[ $ARG == *_obs_* ]]; then SPLIT=obs; CASES=$OBSB; else SPLIT=v2; CASES=$BASE; fi
  one $W/rescored_$ARG rescored_rounded.json python3 $C5 effects --run $R/wm4/$ARG --split $SPLIT --cases $CASES \
    --name rescored_rounded --out $W/rescored_$ARG ;;
describe)
  one $W/describe describe.json python3 $E5 describe --base $BASE --out $W/describe ;;
base)
  one $W/obs2_base obs2_base.json python3 $E5 base --base $BASE --out $W/obs2_base ;;
ridge)
  N=ridge_obs; FLAG=""; [ "$ARG" = "priv" ] && { N=ridge_priv; FLAG=--priv; }
  one $W/$N $N.json python3 $E5 ridge --base $BASE --obs2-base $O2B --out $W/$N $FLAG ;;
causal)
  [ "$(cat $W/$ARG/train.exit)" = "0" ] || { echo "$(date +%F_%T) $ARG train exit != 0: no finals" >> $LOG; exit 1; }
  if [[ $ARG == *_obs2_* ]]; then SPLIT=obs2; CASES=$O2B; else SPLIT=v2; CASES=$BASE; fi
  for WH in last best; do
    one $W/${ARG}_effects effects_$WH.json python3 $C5 effects --run $W/$ARG --which $WH --split $SPLIT --cases $CASES \
      --out $W/${ARG}_effects
  done ;;
causalref)
  one $W/ref_${ARG}_obs2 effects_last.json python3 $C5 effects --run $R/wm4/$ARG --which last --split obs2 --cases $O2B \
    --out $W/ref_${ARG}_obs2 ;;
m1obs2)
  [ "$(cat $W/$ARG/train.exit)" = "0" ] || { echo "$(date +%F_%T) $ARG train exit != 0: no finals" >> $LOG; exit 1; }
  one $W/${ARG}_effects effects.json python3 $E5 m1 --run $W/$ARG --obs2-base $O2B --out $W/${ARG}_effects
  one $W/${ARG}_fact metrics.json python3 tools/worldmodel/wm5/m1_obs2.py final --run $W/$ARG --out $W/${ARG}_fact ;;
ref)
  N=$(basename $ARG)
  one $W/ref_${N}_obs2 effects.json python3 $E5 m1 --run $ARG --obs2-base $O2B --out $W/ref_${N}_obs2 ;;
*) echo "unknown kind $KIND"; exit 2 ;;
esac
