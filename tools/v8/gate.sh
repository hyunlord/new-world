#!/usr/bin/env bash
# usage: gate.sh <log_dir> [--base-data DIR]
#
# V8 gate (goal 11.1). Sixteen steps, each logged to <log_dir>/<step>.log
# with its start time, duration and exit code in <log_dir>/gate.txt. The
# gate passes only when every step except `perf` (report only) exits 0.
#    1 fmt               cargo fmt --all -- --check
#    2 bridge_build      cargo build -p sim-bridge (the library Godot loads)
#    3 godot_import      bounded headless --import --editor --quit (twice: a fresh
#                        import logs font errors before fonts exist)
#    4 test_debug        cargo test --workspace --no-fail-fast -- --test-threads=8
#    5 test_release      the same in --release
#    6 clippy            cargo clippy --workspace --all-targets -- -D warnings
#    7 forbidden_math    float transcendental calls in sim-core/sim-systems/sim-engine src
#    8 gdscript_strict   tools/v8/gdscript_strict_check.sh
#    9 smoke             tools/v8/smoke.sh (counts must not exceed the base counts)
#   10 identity          tools/v8/identity.sh <base data> (32 worlds × 120 days)
#   11 determinism       probe --check-determinism, seeds 0..4
#   12 perf              tools/v8/perf.sh + perf_summary.py --compare (report only;
#                        > +10 % mean is a WARN)
#   13 bridge_api        tools/v8/bridge_api.py --check docs/v8/bridge_api_v7_final.txt
#                        --minus docs/v8/bridge_api_removed.txt
#   14 docs              tools/v8/check_docs.sh
#   15 inventory_closure tools/v8/inventory.py check docs/v8/V8_0_INVENTORY.md
#   16 python_unittest   python3 -m unittest discover -s tools/v8 -p 'test_*.py'
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
LOG="${1:?usage: gate.sh <log_dir> [--base-data DIR]}"
BASE_DATA=/Users/rexxa/work/v8-0-data/base
[[ "${2:-}" == --base-data ]] && BASE_DATA="$3"
mkdir -p "$LOG"
LOG="$(cd "$LOG" && pwd -P)"
source tools/v8/godot_bin.sh
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-8}"
SUMMARY="$LOG/gate.txt"
{ echo "head $(git rev-parse HEAD)"; echo "dirty $(git status --porcelain --untracked-files=no | grep -v '^ M .gitignore$' | wc -l | tr -d ' ')"; echo "started $(date '+%Y-%m-%d %H:%M:%S %Z')"; } > "$SUMMARY"
status=0
step() {
    local n="$1" name="$2"; shift 2
    local start end code
    start=$(date +%s)
    echo "[$n $name] $(date '+%H:%M:%S') $*" > "$LOG/$name.log"
    "$@" >> "$LOG/$name.log" 2>&1
    code=$?
    end=$(date +%s)
    printf '%2s %-18s exit %3s  %5ss  started %s\n' "$n" "$name" "$code" "$((end - start))" "$(date -r "$start" '+%H:%M:%S')" >> "$SUMMARY"
    if [[ "$name" != perf ]] && (( code != 0 )); then status=1; fi
    return "$code"
}
step 1 fmt bash -c 'cd rust && cargo fmt --all -- --check'
step 2 bridge_build bash -c 'cd rust && cargo build -p sim-bridge'
step 3 godot_import bash -c 'bash tools/v8/run_bounded.sh 600 "$GODOT_BIN" --headless --path . --import --editor --quit; bash tools/v8/run_bounded.sh 600 "$GODOT_BIN" --headless --path . --import --editor --quit'
step 4 test_debug bash -c 'cd rust && cargo test --workspace --no-fail-fast -- --test-threads=8'
grep -E '^test result' "$LOG/test_debug.log" | awk '{p+=$4; f+=$6; i+=$8} END {print "   test_debug passed", p, "failed", f, "ignored", i}' >> "$SUMMARY"
step 5 test_release bash -c 'cd rust && cargo test --workspace --release --no-fail-fast -- --test-threads=8'
grep -E '^test result' "$LOG/test_release.log" | awk '{p+=$4; f+=$6; i+=$8} END {print "   test_release passed", p, "failed", f, "ignored", i}' >> "$SUMMARY"
step 6 clippy bash -c 'cd rust && cargo clippy --workspace --all-targets -- -D warnings'
step 7 forbidden_math bash -c "cd rust && ! grep -rnE '\\.(exp|exp2|ln|log2|log10|powf|powi|sin|cos|tan|tanh|atan2|hypot|cbrt)\\(' crates/sim-core/src crates/sim-systems/src crates/sim-engine/src"
step 8 gdscript_strict bash tools/v8/gdscript_strict_check.sh
step 9 smoke bash tools/v8/smoke.sh "$LOG/smoke"
step 10 identity bash tools/v8/identity.sh "$BASE_DATA" "$LOG/identity"
step 11 determinism bash -c 'cd rust && cargo build --release -p sim-test --bin worldsim_probe && ./target/release/worldsim_probe --scene baseline --seeds 0..4 --days 120 --workers 4 --check-determinism --out "$0/determinism_probe" && echo "determinism checked on seeds 0..4"' "$LOG"
grep -q 'DETERMINISM PASS' "$LOG/determinism.log" || { echo "   determinism: no DETERMINISM PASS line" >> "$SUMMARY"; status=1; }
step 12 perf bash -c '(cd rust && cargo build --release -p sim-test) && bash tools/v8/perf.sh rust/target/release "$0/perf" candidate > /dev/null && python3 tools/v8/perf_summary.py --compare "$1/perf/perf_summary.json" "$0/perf/perf_summary.json"' "$LOG" "$BASE_DATA"
grep -E 'WARN|OK \(' "$LOG/perf.log" | sed 's/^/   /' >> "$SUMMARY"
step 13 bridge_api python3 tools/v8/bridge_api.py . --check docs/v8/bridge_api_v7_final.txt --minus docs/v8/bridge_api_removed.txt
step 14 docs bash tools/v8/check_docs.sh
step 15 inventory_closure python3 tools/v8/inventory.py check docs/v8/V8_0_INVENTORY.md
step 16 python_unittest python3 -m unittest discover -s tools/v8 -p 'test_*.py'
echo "finished $(date '+%Y-%m-%d %H:%M:%S %Z') status $status" >> "$SUMMARY"
cat "$SUMMARY"
exit "$status"
