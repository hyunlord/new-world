#!/bin/bash
# WM-1.3 test evaluations of one trained run (METRICS_1_3.md 4), each once, from the pre-registration commit.
# usage: finals_w13.sh RUN DATASET
set -u
RUN=$1; DS=$2
W=/home/hyunlord/work/wm1-runs
SHA=$(cat $W/_setup/prereg_sha)
D=$W/_code/${SHA:0:8}
[ -d "$D" ] || { git -C /home/hyunlord/work/new-world-wm1 fetch -q origin; git -C /home/hyunlord/work/new-world-wm1 worktree add -f --detach "$D" "$SHA" >/dev/null 2>&1; }
source /home/hyunlord/work/wm1-venv/bin/activate
cd $D
export WM1_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data
LOG=$W/_queue/finals.log
one() {   # one evaluation: OUT then the final arguments
  local out=$1; shift
  mkdir -p $out
  if [ -e $out/metrics.json ] || [ -e $out/probe.json ]; then echo "$(date +%F_%T) $out exists" >> $LOG; return; fi
  date +%s > $out/final.start
  "$@" > $out/final.out 2>&1; echo $? > $out/final.exit; date +%s > $out/final.end
  echo "$(date +%F_%T) $out exit=$(cat $out/final.exit)" >> $LOG
}
[ "$(cat $W/$RUN/train.exit)" = "0" ] || { echo "$(date +%F_%T) $RUN train exit != 0: no finals" >> $LOG; exit 1; }
one $W/$RUN python3 tools/worldmodel/wm1/m1_gru.py final --run $W/$RUN
case $DS in
  w13_v16_*) one $W/${RUN}_short python3 tools/worldmodel/wm1/m1_gru.py final --run $W/$RUN --ckpt best_wm12rule.pt --out $W/${RUN}_short ;;
esac
case $DS in
  w13_v16_n400) one $W/${RUN}_legacy python3 tools/worldmodel/wm1/m1_gru.py final --run $W/$RUN --eval-dataset p10_v16 --out $W/${RUN}_legacy ;;
esac
if [ "$RUN" = "m1tf_w13v17_n400_s0" ]; then
  one $W/probe_$RUN python3 tools/worldmodel/wm1/probe_world.py --run $W/$RUN --out $W/probe_$RUN
fi
