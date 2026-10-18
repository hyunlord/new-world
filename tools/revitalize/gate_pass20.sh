#!/usr/bin/env bash
# usage: gate_pass20.sh <log_dir>
# Pass 19's 15 gate steps unchanged (gate_pass19.sh); Pass 20 runs every step
# in its own process group under run_watched_p20.sh (SIGSTOP while a foreign
# quiet lock exists, SIGCONT after; stopped intervals logged), re-runs
# perf_quick when it was stopped, and compares allowed paths to 5cb16639.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
LOG="${1:?usage: gate_pass20.sh <log_dir>}"
mkdir -p "$LOG"
export CARGO_BUILD_JOBS=6
GODOT_BIN="${GODOT_BIN:-/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot}"
SUMMARY="$LOG/gate.txt"
bash tools/revitalize/wait_mac_p20.sh "$LOG/RUNLOG.txt" 'Pass 20 gate' || exit 1
{ echo "head $(git rev-parse HEAD)"; echo "started $(date '+%Y-%m-%d %H:%M:%S %Z')"; } > "$SUMMARY"
status=0
step() {
    local name="$1"; shift
    WATCH_STATUS="$LOG/$name.stopped" bash tools/revitalize/run_watched_p20.sh "$LOG/RUNLOG.txt" "gate $name" \
        bash -c 'out="$1"; shift; exec "$@" > "$out" 2>&1' _ "$LOG/$name.log" "$@"
    local code=$?
    echo "$name exit $code stopped_s $(cat "$LOG/$name.stopped" 2>/dev/null || echo ?)" >> "$SUMMARY"
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
PERF_QUICK='cd rust && rm -rf target/p20_quick && cargo build --release -p sim-test && ./target/release/worldsim_perf quick --out target/p20_quick && python3 -c "import json; j=json.load(open(\"target/p20_quick/quick_stress_v19_s15100_n500_side64_fed.json\")); t=j[\"tick_summary\"]; assert t[\"ticks\"] > 0 and j[\"founders\"] == 500, t; print(\"quick mean_ms\", t[\"mean_ms\"], \"p999_ms\", t[\"p999_ms\"], \"max_ms\", t[\"max_ms\"])"'
step perf_quick bash -c "$PERF_QUICK"
# Pass 19/20 (1.2): a timed step that was stopped is re-run (at most twice).
for attempt in 1 2; do
    stopped=$(cat "$LOG/perf_quick.stopped" 2>/dev/null || echo 0)
    [[ "$stopped" == 0.0 || "$stopped" == 0 ]] && break
    mv "$LOG/perf_quick.log" "$LOG/perf_quick.stopped_attempt$attempt.log"
    echo "perf_quick was stopped ${stopped}s; re-run $attempt" >> "$SUMMARY"
    step perf_quick bash -c "$PERF_QUICK"
done
grep -E '^quick mean_ms' "$LOG/perf_quick.log" >> "$SUMMARY"
echo 'perf_quick is untimed validation; use the quiet-lock stage measurements for T6' >> "$SUMMARY"
step allowed_paths bash -c 'base=5cb16639b2334800aee9b907a7f3b26f0c657aaf; changed=$(git diff --name-only "$base"..HEAD); bad=$(printf "%s\n" "$changed" | grep -v "^$" | { grep -vE "^(rust/.*|tools/revitalize/.*|docs/revitalize/.*)$"; printf "%s\n" "$changed" | grep -E "^rust/crates/sim-test/src/worldmodel/"; } ); printf "%s\n" "$changed" | sed "s/^/changed: /"; echo "outside section 2: ${bad:-none}"; [ -z "$bad" ]'
echo "finished $(date '+%Y-%m-%d %H:%M:%S %Z') status $status" >> "$SUMMARY"
exit "$status"
