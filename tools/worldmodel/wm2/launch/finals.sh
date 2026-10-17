#!/bin/bash
# WM-2 test evaluations of one trained run (METRICS.md), each once, from the pre-registration commit (_setup/prereg_sha):
# v2_v19 runs: factual T2 on v2_v19_fact (W1), the effect predictions (W3, W4), the split-timing task (W5);
# the v2_v16 run: factual T2 on v2_v16 (W6). The floor-matched scores (W2) run once at the end (floor.sh).
# usage: finals.sh RUN
set -u
RUN=$1
W=/home/hyunlord/work/wm1-runs/wm2
SHA=$(cat $W/_setup/prereg_sha)
D=$W/_code/${SHA:0:8}
[ -d "$D" ] || { git -C /home/hyunlord/work/new-world-wm1 fetch -q origin; git -C /home/hyunlord/work/new-world-wm1 worktree add -f --detach "$D" "$SHA" >/dev/null 2>&1; }
source /home/hyunlord/work/wm1-venv/bin/activate
cd $D
export WM1_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data
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
DS=$(python3 -c "import json;print(json.load(open('$W/$RUN/config.json'))['dataset'])")
if [ "$DS" = "v2_v19" ]; then
  one $W/${RUN}_fact metrics.json python3 tools/worldmodel/wm1/m1_gru.py final --run $W/$RUN --eval-dataset v2_v19_fact --out $W/${RUN}_fact
  one $W/${RUN}_effects effects.json python3 tools/worldmodel/wm2/effects.py model --run $W/$RUN --base $W/effects_base/effects_base.json --out $W/${RUN}_effects
  one $W/${RUN}_split split.json python3 tools/worldmodel/wm2/split_task.py model --run $W/$RUN --out $W/${RUN}_split
else
  one $W/$RUN metrics.json python3 tools/worldmodel/wm1/m1_gru.py final --run $W/$RUN
fi
