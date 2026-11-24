#!/bin/bash
# campaign.sh <bin-tag>: PERF2-0 DGX sim campaign (nohup). Output ~/worldsim-3d/perf2/c-<tag>/.
# 1. render scenes (plain bin, core 18): stress_1000 / stress_village_300 at day, night, winter.
# 2. timing (plain bin, core 19, cpu-timing.lock, load < 10): n100/n300/n1000 x 30 days, 3 reps, order reversed on rep 2;
#    rep 1 also writes the day-30 stress_1000 save. Reps 1-2 give the determinism pair (day fingerprints + end save fnv).
# 3. subprofile (sub bin, core 19): n300 and n1000 x 30 days (ColonyMind scopes, A* time).
set -uo pipefail
W=~/worldsim-3d/perf2; TAG=${1:?tag}; B=$W/bins/$TAG; O=$W/c-$TAG; mkdir -p $O/scenes $O/timing $O/sub
exec >> $O/run.log 2>&1
export WORLDSIM_GIT_SHA=$(cat $W/repo.sha 2>/dev/null || echo unknown)
cd $W/repo
waitload() { while [ $(cut -d. -f1 /proc/loadavg) -ge 10 ]; do sleep 20; done; }
echo "$(date -Is) start $(cat /proc/loadavg)"
for s in stress_1000 stress_village_300; do for at in day night winter; do
  waitload
  /usr/bin/time -f 'wall_s=%e maxrss_kib=%M' taskset -c 18 $B/plain/worldsim_perf perf2-scene --scene $s --at $at --out $O/scenes/$s-$at.wsave
done; done
echo "$(date -Is) SCENES_DONE"
exec 9>$HOME/worldsim-3d/cpu-timing.lock
sample() { local name=$1; shift; flock -x 9; waitload
  echo "$(date -Is) start $name $(cat /proc/loadavg)"
  /usr/bin/time -o $O/$name.time -f 'wall_s=%e user_s=%U sys_s=%S maxrss_kib=%M' taskset -c 19 "$@"
  echo "$(date -Is) finish $name $(cat /proc/loadavg)"; flock -u 9; }
for rep in 1 2 3; do
  if [ $rep = 2 ]; then order="1000 300 100"; else order="100 300 1000"; fi
  for n in $order; do
    extra=(); [ $rep = 1 ] && [ $n = 1000 ] && extra=(--save-out $O/scenes/stress_1000-d30.wsave)
    sample timing/n$n-r$rep $B/plain/worldsim_perf perf2-sim --scene n$n --days 30 --out $O/timing/n$n-r$rep.json "${extra[@]}"
  done
done
echo "$(date -Is) TIMING_DONE"
for n in 300 1000; do sample sub/n$n $B/sub/worldsim_perf perf2-sim --scene n$n --days 30 --out $O/sub/n$n.json; done
echo "$(date -Is) CAMPAIGN_DONE"
