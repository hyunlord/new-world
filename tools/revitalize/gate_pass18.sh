#!/usr/bin/env bash
# usage: gate_pass18.sh <log_dir>
# Pass 17 gate steps unchanged (15 steps); Pass 18 checks the foreign lock before each step,
# compares allowed paths to 8cbb697c, and treats perf_quick as untimed validation.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
LOG="${1:?usage: gate_pass18.sh <log_dir>}"
mkdir -p "$LOG"
export CARGO_BUILD_JOBS=6
GODOT_BIN="${GODOT_BIN:-/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot}"
SUMMARY="$LOG/gate.txt"
bash tools/revitalize/wait_mac_p18.sh "$LOG/RUNLOG.txt" 'Pass 18 gate' || exit 1
{ echo "head $(git rev-parse HEAD)"; echo "started $(date '+%Y-%m-%d %H:%M:%S %Z')"; } > "$SUMMARY"
status=0
step() {
    local name="$1"; shift
    if ! bash tools/revitalize/wait_mac_p18.sh "$LOG/RUNLOG.txt" "gate $name"; then
        echo "$name wait_failed exit 1" >> "$SUMMARY"
        status=1
        return 1
    fi
    "$@" > "$LOG/$name.log" 2>&1
    local code=$?
    echo "$name exit $code" >> "$SUMMARY"
    (( code == 0 )) || status=1
}
totals() {
    grep -E '^test result' "$LOG/$1.log" | awk -v n="$1" '{p+=$4; f+=$6; i+=$8} END {print n, "passed", p, "failed", f, "ignored", i}' >> "$SUMMARY"
}
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
step perf_quick bash -c 'cd rust && rm -rf target/p18_quick && cargo build --release -p sim-test && bash ../tools/revitalize/wait_mac_p18.sh "$1/RUNLOG.txt" "gate perf_quick executable" && ./target/release/worldsim_perf quick --out target/p18_quick && python3 -c "import json; j=json.load(open(\"target/p18_quick/quick_stress_v19_s15100_n500_side64_fed.json\")); t=j[\"tick_summary\"]; assert t[\"ticks\"] > 0 and j[\"founders\"] == 500, t; print(\"quick mean_ms\", t[\"mean_ms\"], \"p999_ms\", t[\"p999_ms\"], \"max_ms\", t[\"max_ms\"])"' _ "$(cd "$LOG" && pwd)"
grep -E '^quick mean_ms' "$LOG/perf_quick.log" >> "$SUMMARY"
echo 'perf_quick is untimed validation; use the quiet-lock stage measurements for R5/F8' >> "$SUMMARY"
step allowed_paths bash -c 'base=8cbb697c97aac800935813d2c79c34195ebef6a3; changed=$(git diff --name-only "$base"..HEAD); bad=$(printf "%s\n" "$changed" | grep -v "^$" | { grep -vE "^(rust/.*|tools/revitalize/.*|docs/revitalize/.*)$"; printf "%s\n" "$changed" | grep -E "^rust/crates/sim-test/src/worldmodel/"; } ); printf "%s\n" "$changed" | sed "s/^/changed: /"; echo "outside section 2: ${bad:-none}"; [ -z "$bad" ]'
echo "finished $(date '+%Y-%m-%d %H:%M:%S %Z') status $status" >> "$SUMMARY"
exit "$status"
