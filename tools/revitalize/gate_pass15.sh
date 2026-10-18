#!/usr/bin/env bash
# usage: gate_pass15.sh <log_dir>
# The Pass 15 gate (PASS15_GOAL.md 4.4), run from the repository root that contains
# this script: gate_pass13.sh's steps unchanged (prepare, debug and release
# `cargo test --workspace`, both clippy runs, the forbidden-math grep, bridge build,
# Godot import, strict check, smoke, both Python discoveries — here with
# WM_BIN=/nonexistent PROBE_BIN=/nonexistent so no recorder or probe runs inside
# the gate, and their run/skip counts recorded), plus `worldsim_perf quick`
# (release build, output to rust/target/p15_quick/, never committed; it must exit 0
# and write valid JSON) and the allowed-paths check of section 2
# (`git diff --name-only f5373ab2..HEAD`; PASS15_EXTRA_ALLOWED may name the one
# module of Part 5's optional fix). Every step runs even if an earlier one fails;
# each step's log and exit code land in <log_dir>, and <log_dir>/gate.txt
# summarises them. Exit 0 only when every step passed.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
LOG="${1:?usage: gate_pass15.sh <log_dir>}"
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
counts() {
    local ran skipped
    ran=$(grep -E '^Ran [0-9]+ test' "$LOG/$1.log" | awk '{print $2}')
    skipped=$(grep -oE 'skipped=[0-9]+' "$LOG/$1.log" | cut -d= -f2)
    echo "$1 ran ${ran:-?} skipped ${skipped:-0} result $(grep -E "^(OK|FAILED)" "$LOG/$1.log" | tail -1)" >> "$SUMMARY"
}
step python_unittest env WM_BIN=/nonexistent PROBE_BIN=/nonexistent python3 -m unittest discover -s tools/revitalize -p 'test_*.py'
counts python_unittest
step python_worldmodel env WM_BIN=/nonexistent PROBE_BIN=/nonexistent python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
counts python_worldmodel
step perf_quick bash -c 'cd rust && rm -rf target/p15_quick && cargo build --release -p sim-test && ./target/release/worldsim_perf quick --out target/p15_quick && python3 -c "import json,sys; j=json.load(open(\"target/p15_quick/quick_stress_v19_s15100_n500_side64_fed.json\")); t=j[\"tick_summary\"]; assert t[\"ticks\"] > 0 and j[\"founders\"] == 500, t; print(\"quick mean_ms\", t[\"mean_ms\"], \"p999_ms\", t[\"p999_ms\"], \"max_ms\", t[\"max_ms\"])"'
grep -E '^quick mean_ms' "$LOG/perf_quick.log" >> "$SUMMARY"
step allowed_paths bash -c 'extra="${PASS15_EXTRA_ALLOWED:-}"; bad=$(git diff --name-only f5373ab2b7679c4a0d3a5c86b256971c86f93d41..HEAD | grep -vE "^(rust/crates/sim-engine/src/lib\.rs|rust/crates/sim-systems/src/worldgen/mod\.rs|rust/crates/sim-test/.*|rust/crates/sim-bridge/src/ffi/world_node\.rs|tools/revitalize/.*|docs/revitalize/.*)$" | { if [ -n "$extra" ]; then grep -vxF "$extra"; else cat; fi; }); git diff --name-only f5373ab2b7679c4a0d3a5c86b256971c86f93d41..HEAD | sed "s/^/changed: /"; echo "outside section 2: ${bad:-none}"; [ -z "$bad" ]'
echo "finished $(date '+%Y-%m-%d %H:%M:%S %Z') status $status" >> "$SUMMARY"
exit "$status"
