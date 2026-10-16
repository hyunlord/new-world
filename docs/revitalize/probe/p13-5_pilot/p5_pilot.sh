#!/usr/bin/env bash
# Pass 13 Part 5 replicate pilot (PASS13_GOAL.md 11.1): fresh worlds 26000..26063, 120 days, --every 60, no agents,
# noise fork at tick 1 with R = 4 (empty schedule dir, factual only), frozen recorder (schema 6).
# usage: p5_pilot.sh FROZEN_SHA [scenes...]
set -euo pipefail
W=/Users/rexxa/work/p13bin/wm_frozen
D=/Users/rexxa/work/worldsim-wm-data/p13
export WORLDSIM_GIT_SHA=${1:?frozen commit sha}; shift
for scene in ${@:-game_v19 game_v18 game_v16 game_v17}; do
  date "+start $scene %H:%M:%S"
  $W --scene $scene --seeds 26000..26064 --days 120 --every 60 --schedule-dir $D/empty_schedules --include-factual --fork-tick 1 --replicates 4 --workers 4 --out $D/pilot/$scene
  date "+end $scene %H:%M:%S"
done
echo PILOT_DONE
