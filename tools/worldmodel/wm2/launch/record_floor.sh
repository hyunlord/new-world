#!/bin/bash
# WM-2 Part 1 (GOAL.md 6.4): the noise-floor families, recorded beside the first training at 4 workers, nice 10:
# for each test world and start s in {20040, 56040, 99240}, a factual-only family forked at s with R = 4 (empty
# schedule dir + --include-factual) into v2/<v19|v16>/floor_s<s>/flat/; then two unforked game_v19 test worlds
# (v2/v19/_unforked/) for the replicate-0 identity check. RESUME=1 adds --resume.
set -u
B=/home/hyunlord/work/wm1-runs/_build/src_wm2/rust/target/release
D=/home/hyunlord/work/worldsim-wm-data/v2
L=/home/hyunlord/work/wm1-runs/wm2/_record
export WORLDSIM_GIT_SHA=26b9af1f4b1552359ad812efcd38b10449ca7175
W=${WORKERS:-4}
R=""; [ "${RESUME:-0}" = 1 ] && R="--resume"
mkdir -p "$L" "$D/_empty_schedules"
step() {
  local n=$1; shift
  if [ "$(cat $L/$n.exit 2>/dev/null)" = "0" ]; then echo "$(date +%F_%T) $n done before: skipped"; return 0; fi
  date +%s > $L/$n.start; echo "$(date +%F_%T) start $n"
  "$@" > $L/$n.log 2>&1; local rc=$?
  echo $rc > $L/$n.exit; date +%s > $L/$n.end; echo "$(date +%F_%T) end $n exit $rc"
  [ $rc -eq 0 ] || exit $rc
}
for spec in "game_v19 v19 40480..40560" "game_v16 v16 42480..42560"; do
  set -- $spec
  for s in 20040 56040 99240; do
    step floor_$2_s$s nice -n 10 $B/worldsim_wm --scene $1 --seeds $3 --days 120 --every 60 --schedule-dir $D/_empty_schedules \
      --include-factual --fork-tick $s --replicates 4 --workers $W --out $D/$2/floor_s$s/flat $R
  done
done
step unforked_v19 nice -n 10 $B/worldsim_wm --scene game_v19 --seeds 40480..40482 --days 120 --every 60 --workers 2 --out $D/v19/_unforked $R
echo "$(date +%F_%T) record_floor done"
