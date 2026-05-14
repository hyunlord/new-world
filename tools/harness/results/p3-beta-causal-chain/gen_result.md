---
feature: p3-beta-causal-chain
code_attempt: 2
---

## Files Changed
- rust/crates/sim-core/src/causal/event.rs: Clarified `parent()` doc-comment per evaluator issue #5 — explains that the stored field is immutable and only `trace_parents` observes eviction.
- rust/crates/sim-test/tests/harness_p3_beta_causal_chain.rs: Strengthened four harness tests per evaluator issues #1–#4 (BuildingPlaced uniqueness; explicit ids in chain_depth_three; per-tile stamp count + parent isolation in multi_tick_chains_independent; trace_parents[0].id + channel check in no_parent_after_eviction).

## Observed Values (seed 42, 20 agents — N/A, this feature uses 64×64 sim with one or two BuildingPlaced events, no agents)
- Assertion 1 (event_id_monotonic): retained_event_count = 8, strictly_increasing = true
- Assertion 2 (building_placed_has_no_parent): BuildingPlaced count = 1, parent() = None
- Assertion 3 (stamp_dirty_parent_is_building): StampDirty count = 6, mismatch_count = 0
- Assertion 4 (influence_changed_parent_is_stamp_dirty): noise_influence.parent == Some(noise_stamp.id) = true
- Assertion 5 (chain_depth_three): chain.len() = 3, ids = [2, 1, 0]
- Assertion 6 (cross_channel_isolation): noise_parent_channel = Noise, danger_parent_channel = Danger, parents differ
- Assertion 7 (multi_tick_chains_independent): id_a ≠ id_b, 6 stamps on each tile each parented to that tile's BuildingPlaced
- Assertion 8 (no_parent_after_eviction): warmth_inf.parent.is_some() = true, no Warmth StampDirty in ring, trace_parents.len() = 1, chain[0].id() = warmth_inf_id, chain[0].channel() = Some(Warmth)

## Threshold Compliance
- Assertion 1 (event_id_monotonic): plan = strictly_increasing && retained_event_count==8, observed = both true, PASS
- Assertion 2 (building_placed_has_no_parent): plan = count==1 && parent==None, observed = 1, None, PASS
- Assertion 3 (stamp_dirty_parent_is_building): plan = count==6 && every parent==BuildingPlaced.id && mismatch==0, observed = 6, all match, 0, PASS
- Assertion 4 (influence_changed_parent_is_stamp_dirty): plan = Noise InfluenceChanged.parent == Some(Noise StampDirty.id), observed = match, PASS
- Assertion 5 (chain_depth_three): plan = len==3 && exact ids in [child, parent, root] order && root.parent==None, observed = [2,1,0] with root.parent=None, PASS
- Assertion 6 (cross_channel_isolation): plan = each InfluenceChanged.parent resolves to same-channel StampDirty && parents differ, observed = both resolve correctly and differ, PASS
- Assertion 7 (multi_tick_chains_independent): plan = distinct BuildingPlaced ids && 6 stamps on each tile, all parented within-tile, no cross-tile leakage, observed = all conditions met, PASS
- Assertion 8 (no_parent_after_eviction): plan = parent.is_some() && no Warmth StampDirty retained && trace_parents.len()==1 containing only warmth_influence, observed = all conditions met, PASS

## Gate Result
- cargo test --workspace: PASS (397 passed, 0 failed)
- cargo test -p sim-test --test harness_p3_beta_causal_chain: PASS (8/8)
- clippy --workspace --all-targets -- -D warnings: PASS (clean)
- harness: PASS (8/8)

## Notes
- All 5 issues from the previous evaluator review were addressed exactly as specified:
  1. building_placed_has_no_parent now collects all BuildingPlaced events and asserts count == 1 before checking parent().
  2. chain_depth_three now asserts exact ids: chain[0].id() == 2, chain[1].id() == 1, chain[2].id() == 0.
  3. multi_tick_chains_independent now asserts 6 StampDirty per tile, every stamp parented to its own tile's BuildingPlaced, and no cross-tile leakage in either direction.
  4. no_parent_after_eviction now asserts chain[0].id() == warmth_inf_id and chain[0].channel() == Some(Warmth).
  5. parent() doc-comment rewritten to state that the stored field is immutable; eviction only affects trace_parents lookup, not the field value.
- No threshold discrepancies. All observed values fall exactly inside the locked plan thresholds.
- No production-code (sim-core / sim-engine / sim-systems) wiring changes in this attempt — implementation from attempt 1 was already correct; only test strengthening and a documentation clarification were required.
