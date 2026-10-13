#!/usr/bin/env bash
# Pass 10 Part 6 fork experiment (PASS10_GOAL.md section 12): Pass 9's p6_forks.sh with
# --scene game_v16 --replicates 4, output under p10/, recorder = the frozen commit.
set -euo pipefail
W=/Users/rexxa/work/p10bin/wm_frozen
D=/Users/rexxa/work/worldsim-wm-data/p10
export WORLDSIM_GIT_SHA=581195820e8e6e79924f0583339da077bfc8d9f7
date '+start %H:%M:%S'
# Branch families: 5 C_cf variants + the one-unit gift + the factual, 32 seeds, R = 4, fork at 28,800.
$W --scene game_v16 --seeds 12000..12032 --days 35 --every 60 --agents --schedule-dir $D/schedules/main --include-factual --fork-tick 28800 --replicates 4 --workers 4 --out $D/forks
date '+main %H:%M:%S'
# Placebos (amount 0) on 8 seeds into the same directory (their factual members are re-recorded identically).
$W --scene game_v16 --seeds 12000..12008 --days 35 --every 60 --agents --schedule-dir $D/schedules/placebo --include-factual --fork-tick 28800 --replicates 4 --workers 4 --out $D/forks
date '+placebo %H:%M:%S'
# Unforked runs of every family (replicate-0 identity).
$W --scene game_v16 --seeds 12000..12032 --days 35 --every 60 --agents --schedule-dir $D/schedules/main --include-factual --workers 4 --out $D/unforked
$W --scene game_v16 --seeds 12000..12008 --days 35 --every 60 --agents --schedule-dir $D/schedules/placebo --include-factual --workers 4 --out $D/unforked
date '+unforked %H:%M:%S'
echo FORKS_DONE
