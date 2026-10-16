#!/usr/bin/env bash
# usage: perf_part3_p15.sh
# Pass 15 Part 3 as ONE timed batch (DECISIONS.md Pass 15): run it through
# perf_batch_p15.sh so the load / process gate is checked once; inside, the sweeps
# run strictly one after another (one timed process at a time) in the drop order's
# reverse priority: natural (7.1), stress fed (7.2), repeatability, stress raw, map (7.3).
# Before each sweep the date, load averages and running processes are appended to
# $P15_DATA/batches.log. If the 10-hour working-time rule is reached, set
# P15_SKIP="map raw" (the drop order) before starting, or touch $P15_DATA/stop_after_current
# to stop after the sweep in progress.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
P15_DATA="${P15_DATA:-/Users/rexxa/work/p15-data}"
BIN="$ROOT/rust/target/release/worldsim_perf"
LOG="$P15_DATA/batches.log"
SKIP=" ${P15_SKIP:-} "
export WORLDSIM_GIT_SHA="${WORLDSIM_GIT_SHA:-$(git rev-parse HEAD)}"
PROBE="docs/revitalize/probe"
status=0
note() {
    { echo "-- $1 $(date '+%F %T %Z') loadavg $(sysctl -n vm.loadavg) contended ${PERF_CONTENDED:-0}";
      echo "   processes: $(for n in Godot cargo rustc clippy-driver worldsim_probe worldsim_wm; do pgrep -x "$n" | sed "s/^/$n:/"; done | tr '\n' ' ')"; } >> "$LOG"
}
run() {
    local name="$1"; shift
    if [[ "$SKIP" == *" $name "* ]]; then echo "-- $name skipped (P15_SKIP)" >> "$LOG"; return; fi
    if [ -e "$P15_DATA/stop_after_current" ]; then echo "-- $name skipped (stop_after_current)" >> "$LOG"; return; fi
    note "$name start"
    "$@" 2> "$P15_DATA/sweep_$name.log"
    local code=$?
    (( code == 0 )) || status=1
    note "$name end exit $code"
}
run natural "$BIN" natural --scene game_v19 --seeds 15000..15032 --days 120 --out "$PROBE/p15-1_natural"
run fed "$BIN" stress --seeds 15100..15108 --agents 25,50,100,200,500,1000,2000 --side 64 --days 2 --feed fed --out "$PROBE/p15-2_stress"
for r in 1 2 3; do
    run "repeat$r" "$BIN" stress --seeds 15100..15101 --agents 500 --side 64 --days 2 --feed fed --out "$P15_DATA/repeat/r$r"
done
run raw "$BIN" stress --seeds 15100..15108 --agents 25,50,100,200,500,1000,2000 --side 64 --days 2 --feed raw --out "$PROBE/p15-2_stress"
run map "$BIN" map --seeds 15100..15108 --sides 64,128,256 --days 2 --out "$PROBE/p15-3_map"
exit "$status"
