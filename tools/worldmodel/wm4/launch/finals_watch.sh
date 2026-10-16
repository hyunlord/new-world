#!/bin/bash
# WM-4: launch each run's test evaluations (finals4.sh, from the pre-registration commit) once its training has ended, in
# queue order, one evaluation job at a time.
W=/home/hyunlord/work/wm1-runs/wm4
for spec in "m1obs m1b_obs_s0" "causal crn_obs_s0" "causal ct_obs_s0" "m1obs m1b_obs_s1" "causal crn_obs_s1" "causal ct_obs_s1" "causal crn_v2_s0" "causal ct_v2_s0" "causal crn_v2_s1" "causal ct_v2_s1"; do
  set -- $spec
  while [ ! -e $W/$2/train.exit ]; do
    grep -q "queue stops\|queue done" $W/_queue/queue.log 2>/dev/null && [ ! -e $W/$2/train.start ] && { echo "$(date +%F_%T) $2 never trained"; continue 2; }
    sleep 60
  done
  echo "$(date +%F_%T) finals $1 $2"
  bash $W/_setup/finals4.sh $1 $2
done
echo "$(date +%F_%T) finals_watch done"
