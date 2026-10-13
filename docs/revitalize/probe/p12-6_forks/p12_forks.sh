#!/usr/bin/env bash
# Pass 12 Part 6 fork experiment (PASS12_GOAL.md 12.1): Pass 11's p11_forks.sh with
# --scene game_v18, output under p12/, recorder = the frozen commit (p12bin/wm_frozen).
# usage: p12_forks.sh FROZEN_SHA
set -euo pipefail
W=/Users/rexxa/work/p12bin/wm_frozen
D=/Users/rexxa/work/worldsim-wm-data/p12
export WORLDSIM_GIT_SHA=${1:?frozen commit sha}
date '+start %H:%M:%S'
$W --scene game_v18 --seeds 12000..12032 --days 35 --every 60 --agents --schedule-dir $D/schedules/main --include-factual --fork-tick 28800 --replicates 4 --workers 4 --out $D/forks
date '+main %H:%M:%S'
$W --scene game_v18 --seeds 12000..12008 --days 35 --every 60 --agents --schedule-dir $D/schedules/placebo --include-factual --fork-tick 28800 --replicates 4 --workers 4 --out $D/forks
date '+placebo %H:%M:%S'
$W --scene game_v18 --seeds 12000..12032 --days 35 --every 60 --agents --schedule-dir $D/schedules/main --include-factual --workers 4 --out $D/unforked
$W --scene game_v18 --seeds 12000..12008 --days 35 --every 60 --agents --schedule-dir $D/schedules/placebo --include-factual --workers 4 --out $D/unforked
date '+unforked %H:%M:%S'
echo FORKS_DONE
