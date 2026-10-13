#!/usr/bin/env bash
# Pass 9 Part 3 recordings (PASS9_GOAL.md section 8), wm_p2 = Rust of 874dbf54.
set -euo pipefail
W=/Users/rexxa/work/p9bin/wm_p2
D=/Users/rexxa/work/worldsim-wm-data/p9
export WORLDSIM_GIT_SHA=874dbf542886963a67b56c2d57e4a8c5d5e6ea42
date '+start %H:%M:%S'
# Branch families: 5 C_cf variants + the one-unit gift + the factual, 32 seeds, R = 8, fork at 28,800.
$W --scene game_v15 --seeds 12000..12032 --days 35 --every 60 --agents --schedule-dir $D/schedules/main --include-factual --fork-tick 28800 --replicates 8 --workers 4 --resume --out $D/forks
date '+main %H:%M:%S'
# Placebos (amount 0) on 8 seeds into the same directory; their factual members are already there (resume keeps them).
$W --scene game_v15 --seeds 12000..12008 --days 35 --every 60 --agents --schedule-dir $D/schedules/placebo --include-factual --fork-tick 28800 --replicates 8 --workers 4 --resume --out $D/forks
date '+placebo %H:%M:%S'
# Unforked runs of every family (replicate-0 identity, 8.1.2).
$W --scene game_v15 --seeds 12000..12032 --days 35 --every 60 --agents --schedule-dir $D/schedules/main --include-factual --workers 4 --resume --out $D/unforked
$W --scene game_v15 --seeds 12000..12008 --days 35 --every 60 --agents --schedule-dir $D/schedules/placebo --include-factual --workers 4 --resume --out $D/unforked
date '+unforked %H:%M:%S'
echo P3_DONE
