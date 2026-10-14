#!/bin/bash
# WM-2 baselines (GOAL.md 10.1), CPU, nice 10, from a git worktree at CODE_SHA.
#   baselines.sh val  CODE_SHA   calendar family + persistence + ridge on validation (v2_v19_fact, v2_v16): selection
#   baselines.sh test CODE_SHA   after the pre-registration: the calendar family on test, the effect baselines
#                                (truths, val selection, test predictions), the split-timing baselines
set -u
STAGE=$1; SHA=$2
W=/home/hyunlord/work/wm1-runs/wm2
D=$W/_code/${SHA:0:8}
[ -d "$D" ] || { git -C /home/hyunlord/work/new-world-wm1 fetch -q origin; git -C /home/hyunlord/work/new-world-wm1 worktree add -f --detach "$D" "$SHA" >/dev/null 2>&1; }
source /home/hyunlord/work/wm1-venv/bin/activate
cd $D
export WM1_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data
LOG=$W/_queue/baselines.log
mkdir -p $W/_queue
one() {
  local out=$1 marker=$2; shift 2
  mkdir -p $out
  if [ -e $out/$marker ]; then echo "$(date +%F_%T) $out/$marker exists" >> $LOG; return; fi
  date +%s > $out/job.start
  nice -n 10 "$@" > $out/job.out 2>&1; echo $? > $out/job.exit; date +%s > $out/job.end
  echo "$(date +%F_%T) $out exit=$(cat $out/job.exit) $(( $(cat $out/job.end) - $(cat $out/job.start) ))s" >> $LOG
}
if [ "$STAGE" = val ]; then
  one $W/calendar_val_v19 calendar_val.json python3 tools/worldmodel/wm1/climatology.py val --dataset v2_v19_fact --out $W/calendar_val_v19 &
  one $W/calendar_val_v16 calendar_val.json python3 tools/worldmodel/wm1/climatology.py val --dataset v2_v16 --out $W/calendar_val_v16 &
  wait
else
  one $W/calendar_v19 calendar.json python3 tools/worldmodel/wm1/climatology.py final --dataset v2_v19_fact --selection $W/calendar_val_v19/calendar_val.json --out $W/calendar_v19 &
  one $W/calendar_v16 calendar.json python3 tools/worldmodel/wm1/climatology.py final --dataset v2_v16 --selection $W/calendar_val_v16/calendar_val.json --out $W/calendar_v16 &
  one $W/split_base split_base.json python3 tools/worldmodel/wm2/split_task.py base --out $W/split_base &
  one $W/effects_base effects_base.json python3 tools/worldmodel/wm2/effects.py base --out $W/effects_base
  wait
fi
echo "$(date +%F_%T) baselines $STAGE done" >> $LOG
