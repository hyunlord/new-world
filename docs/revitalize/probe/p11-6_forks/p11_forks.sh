#!/usr/bin/env bash
# Pass 11 Part 6 fork experiment (PASS11_GOAL.md 12.1): Pass 10's p6_forks.sh with
# --scene game_v17 --replicates 4, output under p11/, recorder = the frozen commit.
# usage: p11_forks.sh FROZEN_SHA
set -euo pipefail
W=/Users/rexxa/work/p11bin/wm_frozen
D=/Users/rexxa/work/worldsim-wm-data/p11
export WORLDSIM_GIT_SHA=${1:?frozen commit sha}
date '+start %H:%M:%S'
# Branch families: 5 C_cf variants + the one-unit gift + the factual, 32 seeds, R = 4, fork at 28,800.
$W --scene game_v17 --seeds 12000..12032 --days 35 --every 60 --agents --schedule-dir $D/schedules/main --include-factual --fork-tick 28800 --replicates 4 --workers 4 --out $D/forks
date '+main %H:%M:%S'
# Placebos (amount 0) on 8 seeds into the same directory.
$W --scene game_v17 --seeds 12000..12008 --days 35 --every 60 --agents --schedule-dir $D/schedules/placebo --include-factual --fork-tick 28800 --replicates 4 --workers 4 --out $D/forks
date '+placebo %H:%M:%S'
# Unforked runs of every family (replicate-0 identity).
$W --scene game_v17 --seeds 12000..12032 --days 35 --every 60 --agents --schedule-dir $D/schedules/main --include-factual --workers 4 --out $D/unforked
$W --scene game_v17 --seeds 12000..12008 --days 35 --every 60 --agents --schedule-dir $D/schedules/placebo --include-factual --workers 4 --out $D/unforked
date '+unforked %H:%M:%S'
echo FORKS_DONE
