#!/usr/bin/env bash
# Pass 13 Part 6.2 macro sensitivity (PASS13_GOAL.md 12.2, J11): 120 days, no --agents, seeds 12000..12031,
# fork 28,800, R = 2, factual + rot_wood, gift_wood, spoil_food 60, gift_food 60 at 29,400 on home
# (p13/schedules/macro, written explicitly). Both game_v19 and, fresh, game_v18 with the frozen recorder.
# usage: p13_macro.sh FROZEN_SHA [scenes...]
set -euo pipefail
W=/Users/rexxa/work/p13bin/wm_frozen
D=/Users/rexxa/work/worldsim-wm-data/p13
export WORLDSIM_GIT_SHA=${1:?frozen commit sha}; shift
for scene in ${@:-game_v19 game_v18}; do
  out=$D/macro; [[ $scene == game_v18 ]] && out=$D/macro_v18
  date "+start $scene %H:%M:%S"
  $W --scene $scene --seeds 12000..12032 --days 120 --every 60 --schedule-dir $D/schedules/macro --include-factual --fork-tick 28800 --replicates 2 --workers 4 --out $out
  date "+end $scene %H:%M:%S"
done
echo MACRO_DONE
