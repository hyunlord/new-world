#!/usr/bin/env bash
# usage: perf_p16.sh <out_dir> <final_bin_dir> <base_bin_dir>
# Pass 16 performance rule (PASS16_GOAL.md 8.5; PERF_BUDGET.md section 2), meant to
# run inside quiet_lock_p16.sh (one hold):
#  (a)  code cost   — `worldsim_perf stress --rules game_v20_inert` vs `--rules game_v19`,
#       seed 15100, n = 500, 64 x 64, 1 day, fed;
#  (a') behaviour   — `--rules game_v20` vs `game_v19`, same runs;
#       three interleaved repeats (v19, inert, v20) x 3 into <out>/a/r{1,2,3}/;
#  (a'') `worldsim_perf quick` with the base build and with the final build;
#  (b)  the three-size stress run under game_v20 (and, for context, game_v19):
#       --seeds 15100..15101 --agents 100,500,1000 --side 64 --days 2 --feed fed.
set -uo pipefail
OUT="${1:?out}"; FINAL="${2:?final bin dir}"; BASE="${3:?base bin dir}"
mkdir -p "$OUT"
status=0
run() { echo "$(date '+%T') $*" >> "$OUT/runs.log"; "$@" >> "$OUT/runs.log" 2>&1 || status=1; }
for r in 1 2 3; do
    for rules in game_v19 game_v20_inert game_v20; do
        run "$FINAL/worldsim_perf" stress --seeds 15100..15101 --agents 500 --side 64 --days 1 --feed fed --rules "$rules" --out "$OUT/a/r$r"
    done
done
run "$BASE/worldsim_perf" quick --out "$OUT/quick_base"
run "$FINAL/worldsim_perf" quick --out "$OUT/quick_final"
for rules in game_v20 game_v19; do
    run "$FINAL/worldsim_perf" stress --seeds 15100..15101 --agents 100,500,1000 --side 64 --days 2 --feed fed --rules "$rules" --out "$OUT/b_$rules"
done
echo "status $status" >> "$OUT/runs.log"
exit "$status"
