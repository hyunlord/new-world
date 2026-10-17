#!/bin/bash
# WM-2 Part 1 (GOAL.md 6.1-6.3): factual timelines of game_v19 40000-40559 -> T_low / T_high / T_eve, the four-arm
# schedules, the game_v19 families (fork 14,280, R = 2, factual included), then the game_v16 control 42000-42559.
# 8 workers (no training runs during this job), nice 10. WORLDSIM_GIT_SHA = the merge commit. Each step logs its
# start / end / exit in $L; a failed step stops the job. RESUME=1 adds --resume to the recorder steps.
set -u
B=/home/hyunlord/work/wm1-runs/_build/src_wm2/rust/target/release
D=/home/hyunlord/work/worldsim-wm-data/v2
C=/home/hyunlord/work/new-world-wm1
L=/home/hyunlord/work/wm1-runs/wm2/_record
PY=/home/hyunlord/work/wm1-venv/bin/python
export WORLDSIM_GIT_SHA=26b9af1f4b1552359ad812efcd38b10449ca7175
CLK=02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30
W=${WORKERS:-8}
R=""; [ "${RESUME:-0}" = 1 ] && R="--resume"
mkdir -p "$L" "$D/v19" "$D/v16"
step() {   # step NAME CMD...: skipped when NAME.exit is 0 already
  local n=$1; shift
  if [ "$(cat $L/$n.exit 2>/dev/null)" = "0" ]; then echo "$(date +%F_%T) $n done before: skipped"; return 0; fi
  date +%s > $L/$n.start; echo "$(date +%F_%T) start $n"
  "$@" > $L/$n.log 2>&1; local rc=$?
  echo $rc > $L/$n.exit; date +%s > $L/$n.end; echo "$(date +%F_%T) end $n exit $rc"
  [ $rc -eq 0 ] || exit $rc
}
step timeline nice -n 10 $B/worldsim_probe --scene game_v19 --seeds 40000..40560 --days 120 --clock-samples $CLK \
  --sample-every 60 --workers $W --check-determinism --out $D/v19/timeline
step timeline_out $PY $C/tools/revitalize/timeline_p14.py --summary $D/v19/timeline/summary.json --out $D/v19/timeline_out
step schedules $PY $C/tools/revitalize/schedules_p14.py --timeline $D/v19/timeline_out/timeline.json \
  --arms sp240L,sp120L,gf240E,rw120E --out $D/v19/schedules
step families nice -n 10 $B/worldsim_wm --scene game_v19 --seeds 40000..40560 --days 120 --every 60 \
  --schedule-dir $D/v19/schedules --include-factual --fork-tick 14280 --replicates 2 --workers $W --out $D/v19/flat $R
step v16 nice -n 10 $B/worldsim_wm --scene game_v16 --seeds 42000..42560 --days 120 --every 60 --workers $W --out $D/v16 $R
echo "$(date +%F_%T) record_v2 done"
