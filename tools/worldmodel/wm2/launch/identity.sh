#!/bin/bash
# WM-2 Part 0.3 identity: Stage A group F (p14-1_stageA/commands.md) for seeds 12000..12002 (half-open: 12000, 12001)
# with the new worldsim_wm, into v2/_identity/.
set -u
B=/home/hyunlord/work/wm1-runs/_build/src_wm2/rust/target/release
P=/home/hyunlord/work/worldsim-wm-data/p14
export WORLDSIM_GIT_SHA=26b9af1f4b1552359ad812efcd38b10449ca7175
nice -n 10 $B/worldsim_wm --scene game_v19 --seeds 12000..12002 --days 120 --every 60 --schedule-dir $P/schedules_A/F \
  --include-factual --fork-tick 14280 --replicates 2 --workers 8 --out /home/hyunlord/work/worldsim-wm-data/v2/_identity
