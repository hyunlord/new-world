#!/usr/bin/env bash
# usage: perf_sweeps_p15.sh <step>...   (steps: overhead natural stress_fed stress_raw repeat map)
# Pass 15 timed sweeps (PASS15_GOAL.md 7.0–7.3), run from the repository root with the
# release `worldsim_perf` already built at the pushed commit. Each step is one timed batch
# through perf_batch_p15.sh (load / process checks, 10-minute waits, `contended` after 2 h),
# logged to $P15_DATA/batches.log. Outputs: docs/revitalize/probe/p15-*/ (JSON, runs.json);
# the .tick_us.bin sidecars stay beside them and are not committed (except the one the
# review recomputes).
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
P15_DATA="${P15_DATA:-/Users/rexxa/work/p15-data}"
BIN="$ROOT/rust/target/release/worldsim_perf"
BATCH="$ROOT/tools/revitalize/perf_batch_p15.sh"
LOG="$P15_DATA/batches.log"
export WORLDSIM_GIT_SHA="${WORLDSIM_GIT_SHA:-$(git rev-parse HEAD)}"
PROBE="docs/revitalize/probe"
status=0
for step in "$@"; do
    echo "== $step $(date '+%F %T %Z') sha $WORLDSIM_GIT_SHA" >> "$LOG"
    case "$step" in
        overhead) "$BATCH" "$LOG" "$BIN" overhead --scene game_v19 --seed 15000 --days 30 --repeats 3 --out "$PROBE/p15-0_final" ;;
        natural) "$BATCH" "$LOG" "$BIN" natural --scene game_v19 --seeds 15000..15032 --days 120 --out "$PROBE/p15-1_natural" ;;
        stress_fed) "$BATCH" "$LOG" "$BIN" stress --seeds 15100..15108 --agents 25,50,100,200,500,1000,2000 --side 64 --days 2 --feed fed --out "$PROBE/p15-2_stress" ;;
        stress_raw) "$BATCH" "$LOG" "$BIN" stress --seeds 15100..15108 --agents 25,50,100,200,500,1000,2000 --side 64 --days 2 --feed raw --out "$PROBE/p15-2_stress" ;;
        repeat) for r in 1 2 3; do "$BATCH" "$LOG" "$BIN" stress --seeds 15100..15101 --agents 500 --side 64 --days 2 --feed fed --out "$P15_DATA/repeat/r$r" || status=1; done ;;
        map) "$BATCH" "$LOG" "$BIN" map --seeds 15100..15108 --sides 64,128,256 --days 2 --out "$PROBE/p15-3_map" ;;
        *) echo "unknown step $step" >&2; status=2; continue ;;
    esac
    code=$?
    (( code == 0 )) || status=1
    echo "== $step done $(date '+%F %T %Z') exit $code" >> "$LOG"
done
exit "$status"
