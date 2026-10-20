#!/bin/bash
# WM-6 Part 1 (GOAL.md 6): the v2x counterfactual worlds game_v19 50000-51599 with the frozen WM-2 recorder
# (_build/src_wm2, WORLDSIM_GIT_SHA f5373ab2…), WM-2's flags (record_v2.sh) except R = 1:
#   1. timeline: worldsim_probe with WM-2's clock list and --check-determinism, in seed chunks of CHUNK (each chunk's
#      worker count chosen at its start), merged (wm6/data6.py merge-timeline), then timeline_p14.py and
#      schedules_p14.py --arms sp240L,sp120L,gf240E,rw120E into v2x/;
#   2. families: one recorder call per world (--seeds s..s+1 --workers 1 --schedule-dir v2x/schedules
#      --include-factual --fork-tick 14280 --replicates 1) into v2x/flat/; such a call reproduces WM-2's r0 members bit
#      for bit (DECISIONS L1.1);
#   3. link (families_v2.link) and the per-family prefix check (families_v2.check: every member equals none_r0 up to
#      the fork row 238, every arm equals none_r0 up to its cast row).
# Workers (GOAL.md 0.3.7): 8 when no training runs, 4 while one does. The training queue (queue.sh) touches
# _setup/training.want before a run and waits until _record/rec.load <= 4; probe chunks read the marker at their start,
# the per-world loop before every launch. nice 10. Each step logs start / end / exit in $L; a step whose .exit is 0 is
# skipped on a re-run; a failed step stops the job.
# usage: record_v2x.sh CODE_SHA
set -u
SHA=$1
B=/home/hyunlord/work/wm1-runs/_build/src_wm2/rust/target/release
D=/home/hyunlord/work/worldsim-wm-data/v2x
W6=/home/hyunlord/work/wm1-runs/wm6
L=$W6/_record
PY=/home/hyunlord/work/wm1-venv/bin/python
C=$(bash $W6/_setup/code.sh $SHA) || exit 9
export WORLDSIM_GIT_SHA=f5373ab2b7679c4a0d3a5c86b256971c86f93d41
CLK=02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30
LO=${LO:-50000}; HI=${HI:-51600}; CHUNK=${CHUNK:-100}
mkdir -p "$L" "$D" "$L/fam"
cap() { [ -e $W6/_setup/training.want ] && echo 4 || echo 8; }
setload() { echo "$1" > $L/rec.load; }
step() {   # step NAME CMD...: skipped when NAME.exit is 0 already
  local n=$1; shift
  if [ "$(cat $L/$n.exit 2>/dev/null)" = "0" ]; then echo "$(date +%F_%T) $n done before: skipped"; return 0; fi
  date +%s > $L/$n.start; echo "$(date +%F_%T) start $n (load $(cut -d' ' -f1-3 /proc/loadavg))"
  "$@" > $L/$n.log 2>&1; local rc=$?
  echo $rc > $L/$n.exit; date +%s > $L/$n.end; echo "$(date +%F_%T) end $n exit $rc"
  [ $rc -eq 0 ] || { setload 0; exit $rc; }
}
rm -f $L/rec.done
# 1. timeline, in chunks
for ((a=LO; a<HI; a+=CHUNK)); do
  b=$(( a + CHUNK > HI ? HI : a + CHUNK )); n=probe_$a
  [ "$(cat $L/$n.exit 2>/dev/null)" = "0" ] && continue
  K=$(cap); echo $K > $L/$n.workers; date +%s > $L/$n.start
  echo "$(date +%F_%T) start $n seeds $a..$b workers $K (load $(cut -d' ' -f1-3 /proc/loadavg))"
  rm -rf $D/timeline/c$a
  nice -n 10 $B/worldsim_probe --scene game_v19 --seeds $a..$b --days 120 --clock-samples $CLK --sample-every 60 \
    --workers $K --check-determinism --out $D/timeline/c$a > $L/$n.log 2>&1 &
  P=$!; setload $K
  wait $P; rc=$?
  echo $rc > $L/$n.exit; date +%s > $L/$n.end; setload 0
  echo "$(date +%F_%T) end $n exit $rc $(tail -1 $L/$n.log)"
  [ $rc -eq 0 ] || exit $rc
done
step merge $PY $C/tools/worldmodel/wm6/data6.py merge-timeline --root $D/timeline --lo $LO --hi $HI --chunk $CHUNK \
  --out $D/timeline/summary.json
step timeline_out $PY $C/tools/revitalize/timeline_p14.py --summary $D/timeline/summary.json --out $D/timeline_out
step schedules $PY $C/tools/revitalize/schedules_p14.py --timeline $D/timeline_out/timeline.json \
  --arms sp240L,sp120L,gf240E,rw120E --out $D/schedules
# 2. families, one call per world, dynamic concurrency
if [ "$(cat $L/families.exit 2>/dev/null)" != "0" ]; then
  [ -e $L/families.start ] || date +%s > $L/families.start
  echo "$(date +%F_%T) start families (load $(cut -d' ' -f1-3 /proc/loadavg))"
  PIDS=()
  prune() { local q=() p; for p in "${PIDS[@]}"; do kill -0 $p 2>/dev/null && q+=($p); done; PIDS=("${q[@]}"); }
  for ((s=LO; s<HI; s++)); do
    [ "$(cat $L/fam/s$s.exit 2>/dev/null)" = "0" ] && continue
    while true; do prune; setload ${#PIDS[@]}; [ ${#PIDS[@]} -lt $(cap) ] && break; sleep 2; done
    if [ -e $L/fam/s$s.exit ]; then rm -rf $D/flat/game_v19_s${s}_*; fi      # a failed earlier attempt: record again
    ( nice -n 10 $B/worldsim_wm --scene game_v19 --seeds $s..$((s + 1)) --days 120 --every 60 --schedule-dir $D/schedules \
        --include-factual --fork-tick 14280 --replicates 1 --workers 1 --out $D/flat > $L/fam/s$s.log 2>&1
      echo $? > $L/fam/s$s.exit ) &
    PIDS+=($!)
    [ $(( (s - LO) % 100 )) = 0 ] && echo "$(date +%F_%T) families: launched s$s (cap $(cap), load $(cut -d' ' -f1-3 /proc/loadavg))"
  done
  while true; do prune; setload ${#PIDS[@]}; [ ${#PIDS[@]} = 0 ] && break; sleep 2; done
  BAD=$(for ((s=LO; s<HI; s++)); do [ "$(cat $L/fam/s$s.exit 2>/dev/null)" = "0" ] || echo $s; done | wc -l)
  echo $BAD > $L/families.bad; date +%s > $L/families.end
  [ "$BAD" = "0" ] && echo 0 > $L/families.exit || { echo 3 > $L/families.exit; echo "$(date +%F_%T) families: $BAD worlds failed"; exit 3; }
  echo "$(date +%F_%T) end families: $(ls $D/flat | wc -l) episodes"
fi
setload 0
# 3. link and prefix check
step link $PY $C/tools/worldmodel/wm2/families_v2.py link --flat $D/flat --fam $D/fam
step check $PY $C/tools/worldmodel/wm2/families_v2.py check --fam $D/fam --out $L/check.json
touch $L/rec.done
echo "$(date +%F_%T) record_v2x done"
