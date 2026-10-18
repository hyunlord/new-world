#!/bin/bash
# WM-5 Part 1 (GOAL.md 6.2): the obs2 split of game_v19 47000-48639 with the frozen WM-2 recorder.
# Stages (one per call):
#   r0        6.2.1  round 0, factual (no schedule) into obs2/rounds/r0/, verified
#   round K   6.2.3  round K >= 1: wm5/obs2.py round-schedules writes obs2/schedules/r<K>/ (the worlds with a new cast),
#                    the recorder records them (--schedule-dir, --include-factual is not needed: every seed of the call
#                    has a file; one call per round over the listed seeds), then obs2.py round-check (prefix checks)
#   families  6.2.5  val and test worlds: one recorder call per (world, ref) into obs2/families/<role>/c<ref>/flat/
# nice 10; WORKERS (default 4 beside a training, 8 when none runs). WORLDSIM_GIT_SHA = the recorder's source (WM-2).
# Each step logs start / end / exit in $L; a step whose .exit is 0 is skipped; a failed step stops the job.
# usage: record_obs2.sh STAGE CODE_SHA [K]
set -u
STAGE=$1; SHA=$2; K=${3:-}
B=/home/hyunlord/work/wm1-runs/_build/src_wm2/rust/target/release
D=/home/hyunlord/work/worldsim-wm-data/obs2
L=/home/hyunlord/work/wm1-runs/wm5/_record
PY=/home/hyunlord/work/wm1-venv/bin/python
C=$(bash /home/hyunlord/work/wm1-runs/wm5/_setup/code.sh $SHA) || exit 9
export WORLDSIM_GIT_SHA=f5373ab2b7679c4a0d3a5c86b256971c86f93d41
W=${WORKERS:-4}
mkdir -p "$L" "$D"
step() {   # step NAME CMD...: skipped when NAME.exit is 0 already
  local n=$1; shift
  if [ "$(cat $L/$n.exit 2>/dev/null)" = "0" ]; then echo "$(date +%F_%T) $n done before: skipped"; return 0; fi
  date +%s > $L/$n.start; echo "$(date +%F_%T) start $n (workers $W, load $(cut -d' ' -f1-3 /proc/loadavg))"
  "$@" > $L/$n.log 2>&1; local rc=$?
  echo $rc > $L/$n.exit; date +%s > $L/$n.end; echo "$(date +%F_%T) end $n exit $rc"
  [ $rc -eq 0 ] || exit $rc
}
case $STAGE in
r0)
  step r0 nice -n 10 $B/worldsim_wm --scene game_v19 --seeds 47000..48640 --days 120 --every 60 --workers $W --out $D/rounds/r0
  step r0_verify $PY $C/tools/worldmodel/wm5/obs2.py verify --dir $D/rounds/r0 --expect 1640 --out $L/r0_verify.json ;;
*) echo "unknown stage $STAGE"; exit 2 ;;
esac
echo "$(date +%F_%T) record_obs2 $STAGE done"
