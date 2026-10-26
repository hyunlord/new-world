#!/usr/bin/env bash
# M-UX2 source tests, headless oracles and serialized runtime evidence checks.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
# Direct helper calls share the full gate/capture lane before creating outputs or running work.
if [[ -z "${WORLDSIM_MUX2_LANE_FD:-}" ]]; then
    exec python3 tools/v8/mux2_gate_lock.py -- bash "$ROOT/tools/v8/mux2_gate_checks.sh" "$@"
fi
python3 tools/v8/mux2_gate_lock.py --check || exit 1
mode="${1:?expected defence, oracles or stall}"
LOG="${2:?expected external log directory}"
mkdir -p "$LOG"
source tools/v8/godot_bin.sh
case "$mode" in
  stall)
    receipt="${MUX2_STALL_RECEIPT:-}"
    if [[ -z "$receipt" ]]; then
      receipt="$LOG/capture/receipt.json"
      python3 tools/v8/mux2_gate_receipts.py capture --kind stall --out "$LOG/capture" --godot "$GODOT_BIN"
    fi
    python3 tools/v8/mux2_gate_receipts.py verify --kind stall --receipt "$receipt"
    ;;
  defence)
    : "${WORLDSIM_MUX_FIXTURES:?required external genuine format6-9 nine-file fixture root}"
    : "${WORLDSIM_B5_LEGACY9:?required external genuine H format9 fixture file}"
    (cd rust && cargo test --release -p sim-test --test steward_defence \
      --test steward_defence_fingerprint --test colony_flee_mux2 \
      --test colony_wildlife --test mux2_firewood --test mux2_fuel_sleep \
      --test colony_steward -- --test-threads=8)
    : "${MUX2_V10_FIXTURES:?required external genuine v10 fixture directory}"
    python3 tools/v8/mux2_gate_fixtures.py "$MUX2_V10_FIXTURES" "${MUX2_V10_MANIFEST:?required original fixture SHA256 manifest}"
    (cd rust && cargo test --release -p sim-test --test steward_defence_migration -- --ignored)
    (cd rust && cargo test --release -p sim-test --test legacy_migration_current -- --ignored)
    (cd rust && cargo test --release -p sim-test --test b5_flora \
      actual_h_format9_fixture_loads_as_oak_then_resaves_as_current_format -- --ignored --exact)
    ;;
  oracles)
    for test in mux2_visibility_validator_test mux2_marker_layout_test mux2_coincident_test mux2_badge_index_test mux2_badge_packing_test mux2_overlap_test mux2_badge_grid_test mux2_badge_aggregate_test mux2_bake_receipt_test mux2_badge_extent_test mux2_wall_cache_test; do
      log="$LOG/$test.log"
      bash tools/v8/run_bounded.sh 300 "$GODOT_BIN" --headless --path . \
        --script "res://tools/v8/$test.gd" > "$log" 2>&1
      python3 tools/v8/mux2_gate_oracle_log.py "$test" "$log"
    done
    log="$LOG/mux2_badge_local_test.log"
    bash tools/v8/run_bounded.sh 300 "$GODOT_BIN" --headless --path . \
      --script res://tools/v8/mux2_badge_local_test.gd -- --expect-lazy > "$log" 2>&1
    python3 tools/v8/mux2_gate_oracle_log.py mux2_badge_local_test "$log"
    log="$LOG/mux2_person_cache_test.log"
    bash tools/v8/run_bounded.sh 300 "$GODOT_BIN" --headless --path . \
      --script res://tools/v8/mux2_person_cache_test.gd -- --expect-cache > "$log" 2>&1
    python3 tools/v8/mux2_gate_oracle_log.py mux2_person_cache_test "$log"
    log="$LOG/mux2_extent_owner_test.log"
    bash tools/v8/run_bounded.sh 300 "$GODOT_BIN" --headless --path . \
      --script res://tools/v8/mux2_extent_owner_test.gd -- --expect-cache > "$log" 2>&1
    python3 tools/v8/mux2_gate_oracle_log.py mux2_extent_owner_test "$log"
    log="$LOG/stall-self-test.log"
    bash tools/v8/run_bounded.sh 120 "$GODOT_BIN" --headless --path . \
      --script res://tools/v8/mux2_stall_test.gd -- --self-test > "$log" 2>&1
    grep -Fxq 'MUX2_STALL_SELF_TEST failures=[]' "$log"
    ! grep -q 'SCRIPT ERROR' "$log"
    ;;
  *) echo "unknown M-UX2 check: $mode" >&2; exit 2 ;;
esac
