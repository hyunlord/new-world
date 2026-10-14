#!/usr/bin/env bash
# Pass 14 Part 0 items 2-3 on the DGX: identity probe (seeds 0..32), replicate-0 unforked recordings (12000..12002),
# timeline probe (12000..12032, --sample-every 60). One job at a time, nice 10, 4 workers, each gated by wait_load.sh.
set -uo pipefail
B=/home/hyunlord/work/worldsim-wm-data/p14/bin; D=/home/hyunlord/work/worldsim-wm-data/p14
export WORLDSIM_GIT_SHA=2ba871c09819c240904510bbf26b0fb2a59a79b0
CLK=02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30
$B/wait_load.sh identity; date "+start identity %F %T"
nice -n 10 $B/probe_frozen --scene game_v19 --seeds 0..32 --days 120 --clock-samples $CLK --sample-every 120 --workers 4 --check-determinism --out $D/identity_v19_120day; echo "identity exit $?"; date "+end identity %T"
$B/wait_load.sh unforked; date "+start unforked %T"
nice -n 10 $B/wm_frozen --scene game_v19 --seeds 12000..12002 --days 120 --every 60 --workers 2 --out $D/unforked; echo "unforked exit $?"; date "+end unforked %T"
$B/wait_load.sh timelineA; date "+start timelineA %T"
nice -n 10 $B/probe_frozen --scene game_v19 --seeds 12000..12032 --days 120 --clock-samples $CLK --sample-every 60 --workers 4 --check-determinism --out $D/timeline_A; echo "timelineA exit $?"; date "+end timelineA %T"
