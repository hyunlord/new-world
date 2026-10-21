#!/usr/bin/env bash
# usage: gate.sh <log_dir> [--base-data DIR]
#
# V8 gate (V8-0 goal 11.1; M1 11.3 added steps 17-20; V8-PLAY 2.3 added step 21). Twenty-one steps, each logged to <log_dir>/<step>.log
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
#   13 bridge_api        tools/v8/bridge_api.py --check docs/v8/bridge_api_m1_5.txt
#   14 docs              tools/v8/check_docs.sh
#   15 inventory_closure tools/v8/inventory.py check docs/v8/V8_0_INVENTORY.md
#   16 python_unittest   python3 -m unittest discover -s tools/v8 -p 'test_*.py'
#   17 m1_tests          the M1 Part 1-3 tests by name (colony_commands, colony_jobs, colony_world,
#                        engine_colony_ffi, probe::colony, jobs::, colony::) + tools/v8/test_locale_trf.gd
#   18 colony_determinism  colony probe, seeds 0..4, 10 days, U-script, --check-determinism
#   19 replay            a scripted colony run's recorded log replays to a byte-identical summary
#   20 uid_check         tools/v8/check_docs_helpers.py uid (committed Godot UIDs, M1 5.3)
#   21 play_check        tools/v8/test_play.sh: tools/play.sh exit codes (unknown version 3, no Godot 2,
#                        bad argument 4) and --headless-check main, m1, v7 (worktrees under PLAY_ROOT)
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
step 13 bridge_api python3 tools/v8/bridge_api.py . --check docs/v8/bridge_api_m1_5.txt
step 14 docs bash tools/v8/check_docs.sh
step 15 inventory_closure python3 tools/v8/inventory.py check docs/v8/V8_0_INVENTORY.md
step 16 python_unittest python3 -m unittest discover -s tools/v8 -p 'test_*.py'
# M1 (goal 11.3): the Part 1-3 tests by name, the Locale.trf test, colony
# determinism with the U-script, replay of a recorded command log, the UID check.
step 17 m1_tests bash -c 'cd rust && cargo test --release -p sim-test --test colony_commands --test colony_jobs --test colony_world --test engine_colony_ffi -- --test-threads=8 && cargo test --release -p sim-test --lib probe::colony && cargo test --release -p sim-systems --lib jobs:: && cargo test --release -p sim-core --lib colony:: && cd .. && bash tools/v8/run_bounded.sh 120 "$GODOT_BIN" --headless --path . --script res://tools/v8/test_locale_trf.gd 2>&1 | tee /dev/stderr | grep -q "^TRF_TEST_PASS"'
grep -E '^test result' "$LOG/m1_tests.log" | awk '{p+=$4; f+=$6} END {print "   m1_tests passed", p, "failed", f}' >> "$SUMMARY"
step 18 colony_determinism bash -c 'cd rust && cargo build --release -p sim-test --bin worldsim_probe && ./target/release/worldsim_probe --scene colony --seeds 0..4 --days 10 --workers 4 --commands ../tools/v8/u_script.json --check-determinism --out "$0/colony_determinism"' "$LOG"
grep -q 'DETERMINISM PASS' "$LOG/colony_determinism.log" || { echo "   colony_determinism: no DETERMINISM PASS line" >> "$SUMMARY"; status=1; }
step 19 replay bash -c 'p=rust/target/release/worldsim_probe; $p --scene colony --seeds 5..7 --days 3 --workers 2 --commands tools/v8/u_script.json --out "$0/replay/first" --commands-out "$0/replay/log" && $p --scene colony --seeds 5..6 --days 3 --workers 1 --commands "$0/replay/log/colony_s5_commands.json" --out "$0/replay/s5" && $p --scene colony --seeds 5..6 --days 3 --workers 1 --commands tools/v8/u_script.json --out "$0/replay/s5_script" && cmp "$0/replay/s5/summary.json" "$0/replay/s5_script/summary.json" && echo "replay: the recorded log reproduces the scripted run byte for byte"' "$LOG"
step 20 uid_check bash -c 'out=$(python3 tools/v8/check_docs_helpers.py uid); [[ -z "$out" ]] || { echo "$out"; exit 1; }; echo "every Godot source has its committed .uid"'
step 21 play_check bash tools/v8/test_play.sh
grep -c '^PLAY_TEST_PASS' "$LOG/play_check.log" | awk '{print "   play_check passed", $1, "of 6"}' >> "$SUMMARY"
echo "finished $(date '+%Y-%m-%d %H:%M:%S %Z') status $status" >> "$SUMMARY"
cat "$SUMMARY"
exit "$status"
