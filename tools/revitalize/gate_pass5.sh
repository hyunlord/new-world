#!/usr/bin/env bash
# usage: gate_pass5.sh <log_dir>
# Runs the Pass 5 gate (PASS5_GOAL.md section 4.4, same steps as Pass 4) from the repository root
# that contains this script. Every step runs even if an earlier one fails;
# each step's log and exit code land in <log_dir>, and <log_dir>/gate.txt
# summarises them. Exit 0 only when every step passed.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
LOG="${1:?usage: gate_pass5.sh <log_dir>}"
mkdir -p "$LOG"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-6}"
GODOT_BIN="${GODOT_BIN:-/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot}"
SUMMARY="$LOG/gate.txt"
{ echo "head $(git rev-parse HEAD)"; echo "started $(date '+%Y-%m-%d %H:%M:%S %Z')"; } > "$SUMMARY"
status=0
step() {
    local name="$1"; shift
    "$@" > "$LOG/$name.log" 2>&1
    local code=$?
    echo "$name exit $code" >> "$SUMMARY"
    (( code == 0 )) || status=1
}
totals() {
    grep -E '^test result' "$LOG/$1.log" | awk -v n="$1" '{p+=$4; f+=$6; i+=$8} END {print n, "passed", p, "failed", f, "ignored", i}' >> "$SUMMARY"
}
# A fresh checkout needs the GDExtension library and Godot's import cache
# before the headless GDScript parse tests inside `cargo test` can resolve
# WorldSimNode (environment preparation, repeated below as gate steps).
step prepare_bridge bash -c 'cd rust && cargo build -p sim-bridge'
step prepare_import bash tools/revitalize/run_bounded.sh 600 "$GODOT_BIN" --headless --path . --import --editor --quit
step test_debug bash -c 'cd rust && cargo test --workspace --no-fail-fast -- --test-threads=6'
totals test_debug
step test_release bash -c 'cd rust && cargo test --workspace --release --no-fail-fast -- --test-threads=6'
totals test_release
step clippy bash -c 'cd rust && cargo clippy --workspace -- -D warnings'
step clippy_all_targets bash -c 'cd rust && cargo clippy --workspace --all-targets -- -D warnings'
step forbidden_math bash -c "cd rust && ! grep -rnE '\\.(exp|exp2|ln|log2|log10|powf|powi|sin|cos|tan|tanh|atan2|hypot|cbrt)\\(' crates/sim-core/src crates/sim-systems/src crates/sim-engine/src"
step bridge_build bash -c 'cd rust && cargo build -p sim-bridge'
step godot_import bash tools/revitalize/run_bounded.sh 600 "$GODOT_BIN" --headless --path . --import --editor --quit
step strict_check env GODOT_BIN="$GODOT_BIN" bash tools/harness/gdscript_strict_check.sh
step smoke bash tools/revitalize/run_bounded.sh 600 "$GODOT_BIN" --headless --path . --quit-after 900
if grep -E 'SCRIPT ERROR|ERROR:' "$LOG/smoke.log" | grep -viE 'alsa|audio' > "$LOG/smoke_errors.log"; then
    echo "smoke_errors found $(wc -l < "$LOG/smoke_errors.log")" >> "$SUMMARY"; status=1
else
    echo "smoke_errors none" >> "$SUMMARY"
fi
step python_unittest python3 -m unittest discover -s tools/revitalize -p 'test_*.py'
echo "finished $(date '+%Y-%m-%d %H:%M:%S %Z') status $status" >> "$SUMMARY"
exit "$status"
