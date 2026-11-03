#!/usr/bin/env bash
# usage: gate.sh /absolute/whole-gate.json
#
# Generic V8 gate: source-bound BASE reference producers followed by all 51
# candidate steps. One inherited lane owns every build, capture and observation.
# Per-pass modes are not accepted. Native returns and postchecks remain separate;
# only the final complete collector can qualify the recorded stall exception.
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
#   12 perf              fresh same-scene observations on both products (report only;
#                        paired performance acceptance is steps 25 and 45)
#   13 bridge_api        tools/v8/bridge_api.py --check docs/v8/bridge_api_core.txt
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
#   22 schema_docs       tools/v8/check_docs_helpers.py schema: docs/v8/snapshot_schema.md names every bridge
#                        getter and key (M1.5, ADR 0016 C2)
#   23 renderer_agreement tools/v8/renderer_agreement.sh: the text renderer's people = the Godot agent layer's,
#                        seed 100, U-script, tick 900 (C2)
#   24 save_roundtrip    tools/v8/save_roundtrip.sh: save -> load -> tick equals tick, colony seeds 0-7 (C1)
#   25 bench             tools/v8/bench.sh: the standing colony benchmark against docs/perf/PERF_LEDGER.md
#                        (> +10 % on tick mean, p99 or RSS/agent fails; since M2 a C3 ceiling miss fails; since M5 a
#                        justification quoted from CHANGELOG.md comes through BENCH_JUSTIFIED and is limited to
#                        the scenes in BENCH_JUSTIFIED_SCENES — the M5 gate: BENCH_PASS=M5,
#                        BENCH_JUSTIFIED_SCENES=n200_side64)
#   26 soak              tools/v8/soak.sh: the economy soak fixture (M3 goal 3.4), seeds 0..4, 20 days,
#                        tools/v8/soak_script.json: ledger and reservations exact, 0 deaths, determinism
#   27 nav_bench         worldsim_perf nav-bench: N = 1,000 with the soak script, movement + nav µs per tick
#                        against the 600 µs budget, path requests, memo hits, nodes expanded (report only; M3 §4)
#   28 knowledge         colony_knowledge tests + the loss fixture tools/v8/m4_f10_script.json on seeds 0..8,
#                        scored by docs/v8/m4/m4_score.py --f10 (8/8: KnowledgeLost, Lost, door orders waiting)
#   29 story             the storyteller fixture (M5 goal §12): colony seeds 0..2 × 48 days with
#                        tools/v8/m5_u_script.json (no panic: the probe exits 0), scored by
#                        docs/v8/m5/m5_score.py --story (F1's schedule, ledger and reservations exact every tick)
#   30 neighbours        the neighbours fixture (M5.5 goal §11): colony seeds 0..2 × 48 days with
#                        tools/v8/m55_u_script.json, scored by docs/v8/m5_5/m55_score.py --neighbours (two
#                        neighbours, waves 1 and 2 in their windows and accepted, a trade, ledger and reservations exact)
#   31 art_manifest      tools/v8/art_import.py --verify: assets/v8art/ = its manifest (hashes, sizes, budget,
#                        no unlisted file, every binding and material texture resolves; M-ART2 §3.3)
#   32 resolver_parity   material_resolver.gd = the art line's resolve.py on tools/v8/resolver_parity_fixture.json;
#                        since M-ART2.1 flora_resolver.gd = resolve_flora.py on tools/v8/flora_parity_fixture.json
#   33 extensibility_grep no content id literal and no 16-px tile size in scripts/world/** (ADR 0033), the mask and
#                        import tests (tools/v8/test_mart2_extensibility.py, test_art_masks.py, test_art_import.py)
#   34 mart2_render      tools/v8/mart2_ground_test.gd and mart2_people_test.gd headless (chunk bake hash, edges,
#                        banks, tone, one-chunk dig rebake, data-only wall; appearance, anchors, LOD, params reload);
#                        since M-ART2.1 also rounded shores, water lift and texture, hill edges (ground test) and
#                        tools/v8/mart21_trees_test.gd (four seasons per species, bare winter, data-only species, ¼× canopy)
#   35 steward           two seeds × 24 days ON: deterministic, ledger/reservations exact, no panics
#   36 data_only_ext     seven-family selected-pack simulation and headless real-scene UI (M-UX B3)
#   37 mux2_defence      defence/replay/flight/fuel plus genuine format10 migration
#   38 mux2_oracles      visibility negative controls, marker/overlap/wall-cache and stall scorer
#   39 mux2_visibility   fresh current-policy saves and actual eight-scene GPU acceptance
#   40 mux2_why          actual windowed 53-check input and screenshot acceptance
#   41 mux2_stall_receipt fresh two-arm windowed timing after builds, or strict explicit receipt verification
#   42 baseline_isolation source boundary/dead seam checks and real registration tests
#   43 mind_data_only     shipped policy warm-at-hearth real target/record regression
#   44 core_shadow        fresh frozen seed0 x20d fixtures, steward OFF and ON
#   45 core_evaluation    source/binary-bound held-out outcome/why and raw performance checks
#   46 mux21_regressions  strict tools, headless HUD cursor tests and real-renderer marker cache tests
#   47 m6a_food_fixture   two ordinary worlds x72 days, food ledger and nonvacuous chain
#   48 m6a_food_data_only fresh selected plants/items pack end-to-end
#   Since M-ART2 step 23 (renderer_agreement) also checks appearance: every person's look and face variant.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
# Hold the lane before any build, import, test, profiler or Godot process starts.
# The launcher passes its locked descriptor through bash; nested captures reuse it.
if [[ -z "${WORLDSIM_MUX2_LANE_FD:-}" ]]; then
    exec python3 tools/v8/mux2_gate_lock.py -- bash "$ROOT/tools/v8/gate.sh" "$@"
fi
python3 tools/v8/mux2_gate_lock.py --check || exit 1
# The normal gate receives a committed whole-gate selector. The internal session
# path is hash checked again by every stage; it is not an acceptance bypass.
if [[ "${1:-}" != --whole-session ]]; then
    exec python3 tools/v8/generic_whole_gate.py prepare "${1:?usage: gate.sh WHOLE_GATE.json}"
fi
SESSION="${2:?session}"; SESSION_SHA="${3:?session hash}"; ARM="${4:-candidate}"
LOG=$(python3 tools/v8/generic_whole_gate.py info "$SESSION" "$SESSION_SHA" "$ARM" output) || exit 2
BASE_DATA=$(python3 tools/v8/generic_whole_gate.py info "$SESSION" "$SESSION_SHA" base identity) || exit 2
mkdir -p "$LOG"
LOG="$(cd "$LOG" && pwd -P)"
mkdir -p "$LOG/settings" || exit 1
export WORLDSIM_TEST_SETTINGS_DIR="$LOG/settings"
source tools/v8/godot_bin.sh
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-8}"
SUMMARY="$LOG/gate.txt"
{ echo "head $(git rev-parse HEAD)"; echo "dirty $(git status --porcelain --untracked-files=no | grep -v '^ M .gitignore$' | wc -l | tr -d ' ')"; echo "started $(date '+%Y-%m-%d %H:%M:%S %Z')"; } > "$SUMMARY"
status=0
stage_status=0

step() {
    local n="$1" name="$2"; shift 2
    local start end code
    start=$(date +%s)
    echo "[$n $name] $(date '+%H:%M:%S') $*" > "$LOG/$name.log"
    python3 tools/v8/generic_whole_gate.py step "$SESSION" "$SESSION_SHA" "$ARM" "$n" "$name" -- "$@" >> "$LOG/$name.log" 2>&1
    code=$?
    if (( code == 125 )) || { [[ "$n" == 41 || "$n" == 25 || "$n" == 44 || "$n" == 45 || "$n" == 47 || "$n" == 48 || "$n" == 50 ]] && (( code == 2 )); }; then echo "UNKNOWN ownership; remaining steps NOT_RUN" >> "$SUMMARY"; exit 125; fi
    end=$(date +%s)
    printf '%2s %-18s exit %3s  %5ss  started %s\n' "$n" "$name" "$code" "$((end - start))" "$(date -r "$start" '+%H:%M:%S')" >> "$SUMMARY"
    if [[ "$name" != perf && "$name" != nav_bench ]] && (( code != 0 )); then stage_status=1; fi
    return "$code"
}
summarize_tests() {
    local name="$1" producer_exit="$2"
    if ! python3 tools/v8/gate_test_summary.py "$LOG/$name.log" "$producer_exit" > "$LOG/$name.summary.json"; then
        echo "   $name accounting FAIL (see $name.summary.json)" >> "$SUMMARY"
        status=1
        return
    fi
    python3 - "$LOG/$name.summary.json" "$name" >> "$SUMMARY" <<'PY_SUMMARY' || status=1
import json
from pathlib import Path
import sys
result = json.loads(Path(sys.argv[1]).read_text())
print(f"   {sys.argv[2]} passed {result['passed']} failed {result['failed']} ignored {result['ignored']}")
PY_SUMMARY
}
selected() {
    [[ "$ARM" == candidate ]] && return 0
    case "$1" in
        2|3|4|5|9|10|11|12|17|18|19|21|23|24|26|27|28|29|30|34|35|36|37|38|39|40|41|42|43|46|49) return 0 ;;
        *) return 1 ;;
    esac
}
if selected 1; then
step 1 fmt bash -c 'cd rust && cargo fmt --all -- --check'
fi
if selected 2; then
step 2 bridge_build bash -c 'cd rust && cargo build -p sim-bridge'
fi
if selected 3; then
step 3 godot_import bash -c 'bash tools/v8/run_bounded.sh 600 "$GODOT_BIN" --headless --path . --import --editor --quit; bash tools/v8/run_bounded.sh 600 "$GODOT_BIN" --headless --path . --import --editor --quit'
fi
if selected 4; then
step 4 test_debug bash -c 'cd rust && cargo test --workspace --no-fail-fast -- --test-threads=8'
summarize_tests test_debug "$?"
fi
if selected 5; then
step 5 test_release bash -c 'cd rust && cargo test --workspace --release --no-fail-fast -- --test-threads=8'
summarize_tests test_release "$?"
fi
if selected 6; then
step 6 clippy bash -c 'cd rust && cargo clippy --workspace --all-targets -- -D warnings'
fi
if selected 7; then
step 7 forbidden_math bash -c "cd rust && ! grep -rnE '\\.(exp|exp2|ln|log2|log10|powf|powi|sin|cos|tan|tanh|atan2|hypot|cbrt)\\(' crates/sim-core/src crates/sim-systems/src crates/sim-engine/src"
fi
if selected 8; then
step 8 gdscript_strict bash tools/v8/gdscript_strict_check.sh
fi
if selected 9; then
step 9 smoke bash tools/v8/smoke.sh "$LOG/smoke"
if [[ "$ARM" == candidate ]]; then
    python3 tools/v8/generic_whole_gate.py companions "$SESSION" "$SESSION_SHA" || status=1
fi
fi
if selected 10; then
step 10 identity bash -c 'if [[ "$1" == base ]]; then (cd rust && cargo build --release -p sim-test --bin worldsim_probe) && bash tools/v8/identity.sh --capture rust/target/release "$0/identity"; else bash tools/v8/identity.sh "$2" "$0/identity"; fi' "$LOG" "$ARM" "$BASE_DATA"
fi
if selected 11; then
step 11 determinism bash -c 'cd rust && cargo build --release -p sim-test --bin worldsim_probe && ./target/release/worldsim_probe --scene baseline --seeds 0..4 --days 120 --workers 4 --check-determinism --out "$0/determinism_probe" && echo "determinism checked on seeds 0..4"' "$LOG"
grep -q 'DETERMINISM PASS' "$LOG/determinism.log" || { echo "   determinism: no DETERMINISM PASS line" >> "$SUMMARY"; status=1; }
fi
if selected 12; then
step 12 perf bash -c '(cd rust && cargo build --release -p sim-test) && bash tools/v8/perf.sh rust/target/release "$0/perf" base > /dev/null && echo "Fresh arm observations; paired performance acceptance is step25/45"' "$LOG"
grep -E 'WARN|OK \(' "$LOG/perf.log" | sed 's/^/   /' >> "$SUMMARY"
fi
if selected 13; then
step 13 bridge_api python3 tools/v8/bridge_api.py . --check docs/v8/bridge_api_read.txt
fi
if selected 14; then
step 14 docs bash tools/v8/check_docs.sh
fi
if selected 15; then
step 15 inventory_closure python3 tools/v8/inventory.py check docs/v8/V8_0_INVENTORY.md
fi
if selected 16; then
step 16 python_unittest bash -c 'cd "$1" && python3 -m unittest discover -s tools/v8 -p '\''test_*.py'\''' whole-harness "$(python3 tools/v8/generic_whole_gate.py info "$SESSION" "$SESSION_SHA" candidate harness)"
# M1 (goal 11.3): the Part 1-3 tests by name, the Locale.trf test, colony
# determinism with the U-script, replay of a recorded command log, the UID check.
fi
if selected 17; then
step 17 m1_tests bash -c 'cd rust && cargo test --release -p sim-test --test colony_commands --test colony_jobs --test colony_world --test engine_colony_ffi -- --test-threads=8 && cargo test --release -p sim-test --lib probe::colony && cargo test --release -p sim-systems --lib jobs:: && cargo test --release -p sim-core --lib colony:: && cd .. || exit $?; bash tools/v8/run_bounded.sh 120 "$GODOT_BIN" --headless --path . --script res://tools/v8/test_locale_trf.gd > "$0/m1-trf.log" 2>&1; godot_code=$?; cat "$0/m1-trf.log" || exit 1; (( godot_code == 0 )) && grep -q "^TRF_TEST_PASS" "$0/m1-trf.log"' "$LOG"
summarize_tests m1_tests "$?"
fi
if selected 18; then
step 18 colony_determinism bash -c 'cd rust && cargo build --release -p sim-test --bin worldsim_probe && ./target/release/worldsim_probe --scene colony --seeds 0..4 --days 10 --workers 4 --commands ../tools/v8/u_script.json --check-determinism --out "$0/colony_determinism"' "$LOG"
grep -q 'DETERMINISM PASS' "$LOG/colony_determinism.log" || { echo "   colony_determinism: no DETERMINISM PASS line" >> "$SUMMARY"; status=1; }
fi
if selected 19; then
step 19 replay bash -c 'p=rust/target/release/worldsim_probe; $p --scene colony --seeds 5..7 --days 3 --workers 2 --commands tools/v8/u_script.json --out "$0/replay/first" --commands-out "$0/replay/log" && $p --scene colony --seeds 5..6 --days 3 --workers 1 --commands "$0/replay/log/colony_s5_commands.json" --out "$0/replay/s5" && $p --scene colony --seeds 5..6 --days 3 --workers 1 --commands tools/v8/u_script.json --out "$0/replay/s5_script" && cmp "$0/replay/s5/summary.json" "$0/replay/s5_script/summary.json" && echo "replay: the recorded log reproduces the scripted run byte for byte"' "$LOG"
fi
if selected 20; then
step 20 uid_check bash -c 'out=$(python3 tools/v8/check_docs_helpers.py uid); [[ -z "$out" ]] || { echo "$out"; exit 1; }; echo "every Godot source has its committed .uid"'
fi
if selected 21; then
step 21 play_check bash tools/v8/test_play.sh
grep -c '^PLAY_TEST_PASS' "$LOG/play_check.log" | awk '{print "   play_check passed", $1, "of 6"}' >> "$SUMMARY"
# M1.5 (goal 10.4, ADR 0016): the contract's new checks.
fi
if selected 22; then
step 22 schema_docs python3 tools/v8/check_docs_helpers.py schema .
fi
if selected 23; then
step 23 renderer_agreement bash tools/v8/renderer_agreement.sh "$LOG/renderer_agreement"
grep -q '^RENDERER_AGREEMENT PASS' "$LOG/renderer_agreement.log" || { echo "   renderer_agreement: no PASS line" >> "$SUMMARY"; status=1; }
fi
if selected 24; then
step 24 save_roundtrip bash tools/v8/save_roundtrip.sh
grep -q '^SAVE_ROUNDTRIP PASS' "$LOG/save_roundtrip.log" || { echo "   save_roundtrip: no PASS line" >> "$SUMMARY"; status=1; }
fi
if selected 25; then
step 25 bench python3 tools/v8/generic_whole_gate_acceptance.py run "$SESSION" "$SESSION_SHA" 25 "${LOG%/*}/steps/candidate/25-bench/acceptance"
grep -E 'admission|ceiling|rss outer|bench: |x[0-9]' "$LOG/bench.log" | sed 's/^/   /' >> "$SUMMARY"
# M3 (goal 3.4): the economy soak fixture, every pass from now on.
fi
if selected 26; then
step 26 soak bash tools/v8/soak.sh "$LOG/soak" rust/target/release/worldsim_probe
grep -E '^SOAK' "$LOG/soak.log" | sed 's/^/   /' >> "$SUMMARY"
fi
if selected 27; then
step 27 nav_bench rust/target/release/worldsim_perf nav-bench --out "$LOG/nav_bench/nav_bench.json"
grep -E '^nav_bench' "$LOG/nav_bench.log" | sed 's/^/   /' >> "$SUMMARY"
# M4 (goal §11): the knowledge tests and the loss fixture (F10: the doorway's only holder killed at tick 1800 on
# seeds 0..8 -> KnowledgeLost within 240 ticks, the doorway Lost, door orders waiting on it).
fi
if selected 28; then
step 28 knowledge bash -c 'set -euo pipefail; cd rust && cargo test --release -p sim-test --test colony_knowledge -- --test-threads=8 && cd .. && rust/target/release/worldsim_probe --scene colony --seeds 0..8 --days 4 --workers 4 --commands tools/v8/m4_f10_script.json --out "$0/knowledge_f10" --commands-out "$0/knowledge_f10/commands" > /dev/null && python3 docs/v8/m4/m4_score.py --f10 "$0/knowledge_f10/summary.json" | tee /dev/stderr | grep -q "^F10: 8/8 PASS" && rust/target/release/worldsim_probe --scene colony --seeds 0..8 --days 4 --workers 4 --commands tools/v8/gate_f10_no_kill.json --out "$0/knowledge_f10_control" > /dev/null' "$LOG"
grep -E '^F10|test result' "$LOG/knowledge.log" | sed 's/^/   /' >> "$SUMMARY"
# M5 (goal §12): the storyteller fixture — two worlds through two years of pressure.
fi
if selected 29; then
step 29 story bash -c 'rust/target/release/worldsim_probe --scene colony --seeds 0..2 --days 48 --workers 2 --commands tools/v8/m5_u_script.json --out "$0/story" > /dev/null && python3 docs/v8/m5/m5_score.py --story "$0/story/summary.json"' "$LOG"
grep -E '^STORY' "$LOG/story.log" | sed 's/^/   /' >> "$SUMMARY"
# M5.5 (goal §11): the neighbours fixture — two worlds × 48 days with neighbours on, every wave accepted, the blade
# trade at every caravan: waves 1 and 2 in their windows and accepted, at least one trade, ledger exact (with traded
# in and out) and reservations exact every tick.
fi
if selected 30; then
step 30 neighbours bash -c 'rust/target/release/worldsim_probe --scene colony --seeds 0..2 --days 48 --workers 2 --commands tools/v8/m55_u_script.json --out "$0/neighbours" > /dev/null && python3 docs/v8/m5_5/m55_score.py --neighbours "$0/neighbours/summary.json"' "$LOG"
grep -E '^NEIGHBOURS' "$LOG/neighbours.log" | sed 's/^/   /' >> "$SUMMARY"
# M-ART2 (goal §11): the imported art, the material resolver, the extensibility grep and the renderer tests.
fi
if selected 31; then
step 31 art_manifest python3 tools/v8/art_import.py --verify
fi
if selected 32; then
step 32 resolver_parity bash -c 'bash tools/v8/run_bounded.sh 300 "$GODOT_BIN" --headless --path . --script res://tools/v8/resolver_parity.gd -- "$0/resolver_godot.json" && python3 tools/v8/resolver_parity.py --check "$0/resolver_godot.json"' "$LOG"
fi
if selected 33; then
step 33 extensibility_grep python3 -m unittest tools/v8/test_mart2_extensibility.py tools/v8/test_art_masks.py tools/v8/test_art_import.py
fi
if selected 34; then
step 34 mart2_render bash -c 'set -euo pipefail; for t in mart2_ground_test mart2_people_test mart21_trees_test; do bash tools/v8/run_bounded.sh 600 "$GODOT_BIN" --headless --path . --script res://tools/v8/$t.gd > "$0/$t.out" 2>&1; grep -E "^MART21?_(GROUND|PEOPLE|TREES) (PASS|FAIL)|_DONE" "$0/$t.out"; grep -q "_DONE failures=0" "$0/$t.out" || exit 1; ! grep -q "SCRIPT ERROR" "$0/$t.out" || { grep "SCRIPT ERROR" "$0/$t.out" | head -5; exit 1; }; done' "$LOG"
grep -E 'PASS|FAIL' "$LOG/resolver_parity.log" | tail -1 | sed 's/^/   /' >> "$SUMMARY"
grep -cE '^MART21?_(GROUND|PEOPLE|TREES) PASS' "$LOG/mart2_render.log" | awk '{print "   mart2_render checks passed", $1}' >> "$SUMMARY"
fi
if selected 35; then
step 35 steward bash tools/v8/mux_steward.sh "$LOG/steward"
fi
if selected 36; then
step 36 data_only_ext bash tools/v8/data_only_ext.sh "$LOG/data_only_ext"
# M-CORE compile checks precede the serialized GPU lane.
fi
if selected 42; then
step 42 baseline_isolation bash tools/v8/core_gate_checks.sh baseline_isolation "$LOG/core-registration-tests.log"
fi
if selected 43; then
step 43 mind_data_only bash tools/v8/core_gate_checks.sh data_only "$LOG/core-data-only-tests.log"
# All builds finish before the serialized M-UX2 GPU lane. Missing/stale timing evidence fails.
fi
if selected 37; then
step 37 mux2_defence bash tools/v8/mux2_gate_checks.sh defence "$LOG/mux2-defence"
fi
if selected 38; then
step 38 mux2_oracles bash tools/v8/mux2_gate_checks.sh oracles "$LOG/mux2-oracles"
fi
if selected 39; then
step 39 mux2_visibility python3 tools/v8/mux2_gate_receipts.py capture --kind visibility --out "$LOG/mux2-visibility" --godot "$GODOT_BIN"
fi
if selected 40; then
step 40 mux2_why python3 tools/v8/mux2_gate_receipts.py capture --kind why --out "$LOG/mux2-why" --godot "$GODOT_BIN"
fi
if selected 41; then
step 41 mux2_stall_receipt python3 tools/v8/generic_whole_gate_stall_capture.py "$SESSION" "$SESSION_SHA" "$ARM" "$LOG/mux2-stall"
fi
if selected 44; then
step 44 core_shadow python3 tools/v8/generic_whole_gate_acceptance.py run "$SESSION" "$SESSION_SHA" 44 "${LOG%/*}/steps/candidate/44-core_shadow/acceptance"
fi
if selected 45; then
step 45 core_evaluation python3 tools/v8/generic_whole_gate_acceptance.py run "$SESSION" "$SESSION_SHA" 45 "${LOG%/*}/steps/candidate/45-core_evaluation/acceptance"
grep -E '^CORE_PERF ' "$LOG/core_evaluation.log" 2>/dev/null | sed 's/^/   /' >> "$SUMMARY"
fi
if selected 46; then
step 46 mux21_regressions bash -c '
    set -euo pipefail
    bash tools/v8/gdscript_strict_check.sh tools/v8/mux21_layout_test.gd tools/v8/mux21_toast_test.gd tools/v8/mux21_play_capture.gd tools/v8/mux21_why_locale_test.gd
    mkdir -p "$WORLDSIM_GATE_UI_OUT/toast" "$WORLDSIM_GATE_UI_OUT/layout"
    WORLDSIM_GATE_UI_OUT="$WORLDSIM_GATE_UI_OUT/toast" bash tools/v8/run_bounded.sh 180 "$GODOT_BIN" --headless --path . --script res://tools/v8/mux21_toast_test.gd > "$0/mux21-toast.log" 2>&1
    grep -q "^MUX21_TOAST_DONE " "$0/mux21-toast.log"
    ! grep -q "SCRIPT ERROR" "$0/mux21-toast.log"
    WORLDSIM_GATE_UI_OUT="$WORLDSIM_GATE_UI_OUT/layout" bash tools/v8/run_bounded.sh 300 "$GODOT_BIN" --path . --rendering-method gl_compatibility --script res://tools/v8/mux21_layout_test.gd -- --cache > "$0/mux21-layout.log" 2>&1
    grep -q "^MUX21_LAYOUT_DONE " "$0/mux21-layout.log"
    ! grep -q "SCRIPT ERROR" "$0/mux21-layout.log"
    for language in en ko; do
        mkdir -p "$WORLDSIM_GATE_UI_OUT/why-$language"
        WORLDSIM_GATE_UI_OUT="$WORLDSIM_GATE_UI_OUT/why-$language" bash tools/v8/run_bounded.sh 180 "$GODOT_BIN" --path . --resolution 1920x1080 --script res://tools/v8/mux21_why_locale_test.gd -- --locale="$language" > "$0/mux21-why-$language.log" 2>&1
        ! grep -E "SCRIPT ERROR|Parse Error|ERROR:" "$0/mux21-why-$language.log"
    done
    python3 -c '\''import json,sys; from pathlib import Path
sys.path.insert(0,"tools/v8")
from mux2_gate_visual import validate_why_checks
for language in ("en","ko"):
 validate_why_checks((Path(sys.argv[1])/f"mux21-why-{language}.log").read_text(),64)
for filename,prefix in [("mux21-toast.log","MUX21_TOAST_DONE "),("mux21-layout.log","MUX21_LAYOUT_DONE ")]:
 rows=[json.loads(line[len(prefix):]) for line in (Path(sys.argv[1])/filename).read_text().splitlines() if line.startswith(prefix)]
 assert len(rows)==1 and rows[0]["failures"]==0 and rows[0]["checks"]>0,filename
 if filename=="mux21-layout.log": assert rows[0]["display"]!="headless",filename
'\'' "$0"
' "$LOG"
fi
if selected 47; then
step 47 m6a_food_fixture python3 tools/v8/generic_whole_gate_acceptance.py run "$SESSION" "$SESSION_SHA" 47 "${LOG%/*}/steps/candidate/47-m6a_food_fixture/acceptance"
fi
if selected 48; then
step 48 m6a_food_data_only python3 tools/v8/generic_whole_gate_acceptance.py run "$SESSION" "$SESSION_SHA" 48 "${LOG%/*}/steps/candidate/48-m6a_food_data_only/acceptance"
# M-READ gate steps
fi
if selected 49; then
step 49 read_data_only bash -c 'cd rust && cargo test --release -p sim-core --lib "colony::diagnosis::tests::data_only_check_addition" -- --exact'
fi
if selected 50; then
step 50 read_preservation python3 tools/v8/generic_whole_gate_acceptance.py run "$SESSION" "$SESSION_SHA" 50 "${LOG%/*}/steps/candidate/50-read_preservation/acceptance"
fi
if selected 51; then
step 51 read_diagnosis_test bash -c '
    set -euo pipefail
    RESULT="docs/v8/read/read_30s_result.json"
    [[ -f "$RESULT" ]] || { echo "result file $RESULT does not exist"; exit 1; }
    python3 -c "
import json, sys
r = json.load(open(sys.argv[1]))
passed = sum(1 for q in r[\"questions\"] if q[\"pass\"])
total = len(r[\"questions\"])
q1 = next(q for q in r[\"questions\"] if q[\"id\"] == \"Q1\")
q5 = next(q for q in r[\"questions\"] if q[\"id\"] == \"Q5\")
p1 = q1[\"pass\"]
p5 = q5[\"pass\"]
print(f\"30s test: {passed}/{total}, Q1 pass={p1}, Q5 pass={p5}\")
assert passed >= 5, f\"need >=5/6, got {passed}/{total}\"
assert q1[\"pass\"], \"Q1 must pass\"
assert q5[\"pass\"], \"Q5 must pass\"
print(\"READ_DIAGNOSIS_TEST PASS\")
" "$RESULT"
'
fi
python3 tools/v8/generic_whole_gate.py finish "$SESSION" "$SESSION_SHA" "$ARM" "$status" "$stage_status" || exit 1
status=$((status | stage_status))
echo "finished $(date '+%Y-%m-%d %H:%M:%S %Z') status $status" >> "$SUMMARY"

cat "$SUMMARY"
exit "$status"
