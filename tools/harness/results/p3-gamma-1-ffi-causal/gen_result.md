---
feature: p3-gamma-1-ffi-causal
code_attempt: 2
---

## Files Changed
- rust/crates/sim-bridge/src/ffi/world_node.rs: Added `try_collect_tile_causal_history` and `try_collect_event_chain` pure-Rust FFI-mirror helpers (bounds + negative-id rejection); made `tile_idx_from_coords` `pub`; refactored both `#[func]` bodies to delegate to the helpers (cleaner Bridge Identity Contract).
- rust/crates/sim-bridge/src/ffi/mod.rs: Re-exported `try_collect_tile_causal_history`, `try_collect_event_chain`, and `tile_idx_from_coords` alongside existing γ-1 symbols.
- rust/crates/sim-test/tests/harness_p3_gamma_1_ffi.rs: (A5) strengthened to locate the same-channel `StampDirty` and assert `influence.parent == Some(stamp.id)` via a robust `find_map` over filtered InfluenceChanged candidates (handles ring eviction); (A8) extended to probe both fabricated id `999` and absent id `0`; (new A8b) added `harness_p3_gamma_1_ffi_oob_and_negative_id_return_empty` covering negative-x, negative-y, x ≥ width, y ≥ height, far-OOB, negative `event_id`, and `i64::MIN` event_id for both helpers.

## Observed Values (seed 42, 20 agents — γ-1 is FFI-surface only; setup is 64×64 engine + Phase 2 systems)
- A1 empty-tile history length: 0
- A2 ring length after 13 pushes: 8 (strictly monotonic ids, 0 inversions across 7 adjacent pairs)
- A3 BuildingPlaced view (BSS-only): kind="building_placed", parent=None, position=Some((32,32)), radius=Some(12), channel=None, region=None, old=None, new=None — 9/9 conditions hold
- A4 StampDirty view (BSS-only): kind="stamp_dirty", parent=Some(building_id), channel=Some(idx), region=Some((20,20,44,44)) (Chebyshev box matches), position=None, radius=None, old=None, new=None — 8/8 conditions hold
- A5 InfluenceChanged view (full Phase 2): kind="influence_changed", parent==Some(matched_stamp.id), channel=Some(idx), position=Some((32,32)), radius=None, region=None, old=Some(f32), new=Some(f32) — 8/8 conditions hold
- A6 chain length from leaf id=2: 3, variants=[InfluenceChanged, StampDirty, BuildingPlaced], root.parent=None
- A7 chain length from root id=42: 1, kind="building_placed", parent=None
- A8 missing-id chain length (999): 0; absent-id chain length (0): 0
- A8b OOB+negative probes: all 8 probes return length 0 (no panic)
- A9 cross-channel: Noise chain[1].channel == Noise idx; Danger chain[1].channel == Danger idx; noise_stamp.id != danger_stamp.id
- A10 sentinel: None.map(...).unwrap_or(-1) == -1; Some(7u64).map(...) == 7i64
- A11 re-exports compile: `use sim_bridge::ffi::{collect_tile_causal_history, collect_event_chain, CausalEventView}` succeeds; new helpers also re-exported.

## Threshold Compliance
- Assertion 1 (enumerator_empty_on_unstamped_tile): plan=length==0, observed=0, PASS
- Assertion 2 (enumerator_returns_all_events_after_building_with_ring_cap): plan=length==8 ∧ strict-monotonic, observed=8 ∧ 7/7 monotonic pairs, PASS
- Assertion 3 (dict_schema_building_placed): plan=9 conditions, observed=9/9, PASS
- Assertion 4 (dict_schema_stamp_dirty): plan=8 conditions (exact Chebyshev bounds), observed=8/8, PASS
- Assertion 5 (dict_schema_influence_changed): plan=8 conditions including parent==Some(stamp_dirty_event_id), observed=8/8, PASS
- Assertion 6 (chain_returns_three_events_with_root_parent_none): plan=length==3 ∧ variant order ∧ root.parent==None, observed=3 ∧ [InfluenceChanged,StampDirty,BuildingPlaced] ∧ None, PASS
- Assertion 7 (chain_root_event_returns_single_element): plan=length==1 ∧ BuildingPlaced ∧ parent==None, observed=1 ∧ "building_placed" ∧ None, PASS
- Assertion 8 (chain_invalid_event_id_returns_empty): plan=length==0 for both fabricated large id and absent id 0, observed=0 (id=999) ∧ 0 (id=0), PASS
- Assertion 9 (cross_channel_chain_isolation): plan=3 conjuncts, observed=3/3, PASS
- Assertion 10 (parent_none_serialized_as_minus_one_sentinel): plan=None→-1 ∧ Some(7)→7, observed both, PASS
- Assertion 11 (collectors_publicly_reexported): plan=compile success, observed compile success, PASS
- New Edge-case Assertion (oob_and_negative_id_return_empty): plan=empty Vec across negative-x, negative-y, x≥width, y≥height, negative event_id, i64::MIN, observed all empty, PASS

## Gate Result
- cargo test (workspace): PASS — full workspace runs zero failures; sim-test gamma-1 file alone: 12/12 passed (11 plan assertions + 1 added OOB/negative-id test).
- clippy (`cargo clippy --workspace --all-targets -- -D warnings`): PASS — clean exit 0.
- harness (gamma-1 file): PASS — 12/12.

## Notes
- Code attempt 2 review issues are all addressed:
  1. A5 now asserts `influence.parent == Some(stamp.id)` for the same-channel `StampDirty`. The lookup uses `find_map` to skip InfluenceChanged events whose parent stamp was evicted (first attempt naively picked the oldest InfluenceChanged whose parent Warmth-StampDirty was indeed evicted out of the 8-slot ring after 13 pushes — that bug was caught by the test itself and fixed before reporting).
  2. A8 now probes both `id=999` (fabricated large missing id) and `id=0` (absent id), both yielding empty chains.
  3. New `harness_p3_gamma_1_ffi_oob_and_negative_id_return_empty` test exercises out-of-bounds coords (negative x, negative y, x≥width, y≥height, far-OOB) and negative `event_id` (`-1` and `i64::MIN`) through the newly introduced pure-Rust FFI-mirror helpers `try_collect_tile_causal_history` / `try_collect_event_chain`. These helpers also tighten the Bridge Identity Contract — the `#[func]` bodies now consist of a single forwarding call plus marshalling, matching the `enqueue_building_placed` pattern.
- No threshold discrepancies. No production `.unwrap()` introduced. The exposed surface remains read-only (γ-1 scope unchanged); the two new helpers borrow `&SimResources` only.
- VLM `no-godot-scope` auto-credit still applies — no `.tscn`, `.gd`, or `.gdshader` files were edited.
