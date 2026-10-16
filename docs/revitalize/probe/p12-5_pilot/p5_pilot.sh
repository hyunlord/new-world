#!/usr/bin/env bash
# Pass 12 Part 5 pilot recording (PASS12_GOAL.md 11.1) on the frozen commit: seeds 24000..24111, 120 days.
# usage: p5_pilot.sh FROZEN_SHA
set -euo pipefail
W=/Users/rexxa/work/p12bin/wm_frozen
D=/Users/rexxa/work/worldsim-wm-data/p12/pilot
export WORLDSIM_GIT_SHA=${1:?frozen commit sha}
for scene in game_v16 game_v17 game_v18_nofission game_v18; do
  date "+start $scene %H:%M:%S"
  $W --scene $scene --seeds 24000..24112 --days 120 --every 60 --workers 4 --out $D/$scene
  date "+end $scene %H:%M:%S"
done
echo PILOT_DONE
