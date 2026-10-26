#!/usr/bin/env bash
# usage: gate.sh <log_dir> [--base-data DIR]
#
# V8 gate (V8-0 goal 11.1; M1 11.3 added steps 17-20; V8-PLAY 2.3 added step 21; M1.5 10.4 added steps
# 22-25; M3 added steps 26-27; M4 added step 28; M5 added step 29; M5.5 added step 30; M-ART2 added steps 31-34; M-UX added step 35). Forty-one steps, each logged to <log_dir>/<step>.log
# with its start time, duration and exit code in <log_dir>/gate.txt. The
# gate passes only when every step except `perf` and `nav_bench` (report only) exits 0.
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
#   13 bridge_api        tools/v8/bridge_api.py --check docs/v8/bridge_api_mux2.txt
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
#   40 mux2_why          actual windowed 30-check input and screenshot acceptance
#   41 mux2_stall_receipt fresh two-arm windowed timing after builds, or strict explicit receipt verification
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
LOG="${1:?usage: gate.sh <log_dir> [--base-data DIR]}"
BASE_DATA=/Users/rexxa/work/v8-0-data/base
[[ "${2:-}" == --base-data ]] && BASE_DATA="$3"
mkdir -p "$LOG"
LOG="$(cd "$LOG" && pwd -P)"
mkdir -p "$LOG/settings" || exit 1
export WORLDSIM_TEST_SETTINGS_DIR="$LOG/settings"
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
    if [[ "$name" != perf && "$name" != nav_bench ]] && (( code != 0 )); then status=1; fi
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
step 13 bridge_api python3 tools/v8/bridge_api.py . --check docs/v8/bridge_api_mux2.txt
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
# M1.5 (goal 10.4, ADR 0016): the contract's new checks.
step 22 schema_docs python3 tools/v8/check_docs_helpers.py schema .
step 23 renderer_agreement bash tools/v8/renderer_agreement.sh "$LOG/renderer_agreement"
grep -q '^RENDERER_AGREEMENT PASS' "$LOG/renderer_agreement.log" || { echo "   renderer_agreement: no PASS line" >> "$SUMMARY"; status=1; }
step 24 save_roundtrip bash tools/v8/save_roundtrip.sh
grep -q '^SAVE_ROUNDTRIP PASS' "$LOG/save_roundtrip.log" || { echo "   save_roundtrip: no PASS line" >> "$SUMMARY"; status=1; }
step 25 bench bash tools/v8/bench.sh "$LOG/bench"
grep -E 'admission|ceiling|rss outer|bench: |x[0-9]' "$LOG/bench.log" | sed 's/^/   /' >> "$SUMMARY"
# M3 (goal 3.4): the economy soak fixture, every pass from now on.
step 26 soak bash tools/v8/soak.sh "$LOG/soak" rust/target/release/worldsim_probe
grep -E '^SOAK' "$LOG/soak.log" | sed 's/^/   /' >> "$SUMMARY"
step 27 nav_bench rust/target/release/worldsim_perf nav-bench --out "$LOG/nav_bench/nav_bench.json"
grep -E '^nav_bench' "$LOG/nav_bench.log" | sed 's/^/   /' >> "$SUMMARY"
# M4 (goal §11): the knowledge tests and the loss fixture (F10: the doorway's only holder killed at tick 1800 on
# seeds 0..8 -> KnowledgeLost within 240 ticks, the doorway Lost, door orders waiting on it).
step 28 knowledge bash -c 'cd rust && cargo test --release -p sim-test --test colony_knowledge -- --test-threads=8 && cd .. && rust/target/release/worldsim_probe --scene colony --seeds 0..8 --days 4 --workers 4 --commands tools/v8/m4_f10_script.json --out "$0/knowledge_f10" > /dev/null && python3 docs/v8/m4/m4_score.py --f10 "$0/knowledge_f10/summary.json" | tee /dev/stderr | grep -q "^F10: 8/8 PASS"' "$LOG"
grep -E '^F10|test result' "$LOG/knowledge.log" | sed 's/^/   /' >> "$SUMMARY"
# M5 (goal §12): the storyteller fixture — two worlds through two years of pressure.
step 29 story bash -c 'rust/target/release/worldsim_probe --scene colony --seeds 0..2 --days 48 --workers 2 --commands tools/v8/m5_u_script.json --out "$0/story" > /dev/null && python3 docs/v8/m5/m5_score.py --story "$0/story/summary.json"' "$LOG"
grep -E '^STORY' "$LOG/story.log" | sed 's/^/   /' >> "$SUMMARY"
# M5.5 (goal §11): the neighbours fixture — two worlds × 48 days with neighbours on, every wave accepted, the blade
# trade at every caravan: waves 1 and 2 in their windows and accepted, at least one trade, ledger exact (with traded
# in and out) and reservations exact every tick.
step 30 neighbours bash -c 'rust/target/release/worldsim_probe --scene colony --seeds 0..2 --days 48 --workers 2 --commands tools/v8/m55_u_script.json --out "$0/neighbours" > /dev/null && python3 docs/v8/m5_5/m55_score.py --neighbours "$0/neighbours/summary.json"' "$LOG"
grep -E '^NEIGHBOURS' "$LOG/neighbours.log" | sed 's/^/   /' >> "$SUMMARY"
# M-ART2 (goal §11): the imported art, the material resolver, the extensibility grep and the renderer tests.
step 31 art_manifest python3 tools/v8/art_import.py --verify
step 32 resolver_parity bash -c 'bash tools/v8/run_bounded.sh 300 "$GODOT_BIN" --headless --path . --script res://tools/v8/resolver_parity.gd -- "$0/resolver_godot.json" && python3 tools/v8/resolver_parity.py --check "$0/resolver_godot.json"' "$LOG"
step 33 extensibility_grep python3 -m unittest tools/v8/test_mart2_extensibility.py tools/v8/test_art_masks.py tools/v8/test_art_import.py
step 34 mart2_render bash -c 'for t in mart2_ground_test mart2_people_test mart21_trees_test; do bash tools/v8/run_bounded.sh 600 "$GODOT_BIN" --headless --path . --script res://tools/v8/$t.gd > "$0/$t.out" 2>&1; grep -E "^MART21?_(GROUND|PEOPLE|TREES) (PASS|FAIL)|_DONE" "$0/$t.out"; grep -q "_DONE failures=0" "$0/$t.out" || exit 1; ! grep -q "SCRIPT ERROR" "$0/$t.out" || { grep "SCRIPT ERROR" "$0/$t.out" | head -5; exit 1; }; done' "$LOG"
grep -E 'PASS|FAIL' "$LOG/resolver_parity.log" | tail -1 | sed 's/^/   /' >> "$SUMMARY"
grep -cE '^MART21?_(GROUND|PEOPLE|TREES) PASS' "$LOG/mart2_render.log" | awk '{print "   mart2_render checks passed", $1}' >> "$SUMMARY"
step 35 steward bash tools/v8/mux_steward.sh "$LOG/steward"
step 36 data_only_ext bash tools/v8/data_only_ext.sh "$LOG/data_only_ext"
# All builds finish before the serialized M-UX2 GPU lane. Missing/stale timing evidence fails.
step 37 mux2_defence bash tools/v8/mux2_gate_checks.sh defence "$LOG/mux2-defence"
step 38 mux2_oracles bash tools/v8/mux2_gate_checks.sh oracles "$LOG/mux2-oracles"
step 39 mux2_visibility python3 tools/v8/mux2_gate_receipts.py capture --kind visibility --out "$LOG/mux2-visibility" --godot "$GODOT_BIN"
step 40 mux2_why python3 tools/v8/mux2_gate_receipts.py capture --kind why --out "$LOG/mux2-why" --godot "$GODOT_BIN"
step 41 mux2_stall_receipt bash tools/v8/mux2_gate_checks.sh stall "$LOG/mux2-stall"
echo "finished $(date '+%Y-%m-%d %H:%M:%S %Z') status $status" >> "$SUMMARY"
cat "$SUMMARY"
exit "$status"
