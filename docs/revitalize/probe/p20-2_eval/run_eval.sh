#!/usr/bin/env bash
# Pass 20 7.4: game_v25 and game_v24 on 19400..19431, 120 days, Pass 19's
# evaluation flags + --chronicle-out; then a second game_v25 run (C4 twin).
set -euo pipefail
D=/Users/rexxa/work/p20-data/p2
BIN=/Users/rexxa/work/p20-data/bin_p2
HOURS=00:00,01:00,02:00,03:00,04:00,05:00,06:00,07:00,08:00,09:00,10:00,11:00,12:00,13:00,14:00,15:00,16:00,17:00,18:00,19:00,20:00,21:00,22:00,23:00
export WORLDSIM_GIT_SHA=3562a9d4
(cd $BIN && shasum -a 256 worldsim_probe worldsim_perf) > $D/eval_binary.sha256
for job in game_v25:A game_v24:A game_v25:B; do
  scene=${job%%:*}; run=${job#*:}
  echo "eval $scene $run start $(date '+%F %T')" >> $D/times.txt
  bash /Users/rexxa/work/new-world-pass4/tools/revitalize/run_watched_p20.sh $D/RUNLOG.txt "7.4 $scene run $run" \
    bash -c "$BIN/worldsim_probe --scene $scene --seeds 19400..19432 --days 120 --sample-every 120 --clock-samples $HOURS --workers 4 --check-determinism --agent-trace --out $D/$run/$scene --chronicle-out $D/$run/chronicle_$scene > $D/${scene}_$run.log 2>&1"
  echo "eval $scene $run end $(date '+%F %T')" >> $D/times.txt
done
