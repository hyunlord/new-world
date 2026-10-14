#!/usr/bin/env bash
# Pass 14 Part 3: the factual timeline of Stage B's fresh seeds 14000..14063 (the Part 0 item 3 probe), run after PREREG.
set -uo pipefail
B=/home/hyunlord/work/worldsim-wm-data/p14/bin; D=/home/hyunlord/work/worldsim-wm-data/p14
export WORLDSIM_GIT_SHA=2ba871c09819c240904510bbf26b0fb2a59a79b0
CLK=02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30
$B/wait_load.sh timelineB; date "+start timelineB %F %T"
nice -n 10 $B/probe_frozen --scene game_v19 --seeds 14000..14064 --days 120 --clock-samples $CLK --sample-every 60 --workers 4 --check-determinism --out $D/timeline_B
echo "timelineB exit $?"; date "+end timelineB %F %T"
