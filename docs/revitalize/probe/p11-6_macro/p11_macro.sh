#!/usr/bin/env bash
# Pass 11 Part 6 macro sensitivity (PASS11_GOAL.md 12.2): 120 days, no --agents, the
# rot_wood and gift_wood schedules plus the factual, forked at 28,800 with R = 2.
# usage: p11_macro.sh FROZEN_SHA
set -euo pipefail
W=/Users/rexxa/work/p11bin/wm_frozen
D=/Users/rexxa/work/worldsim-wm-data/p11
export WORLDSIM_GIT_SHA=${1:?frozen commit sha}
mkdir -p $D/schedules/macro
cp $D/schedules/main/s*_rot_wood.json $D/schedules/main/s*_gift_wood.json $D/schedules/macro/
date '+start %H:%M:%S'
$W --scene game_v17 --seeds 12000..12032 --days 120 --every 60 --schedule-dir $D/schedules/macro --include-factual --fork-tick 28800 --replicates 2 --workers 4 --out $D/macro
date '+end %H:%M:%S'
echo MACRO_DONE
