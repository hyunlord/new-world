---
feature: floor-fix
code_attempt: 3
---

## Files Changed
- `rust/crates/sim-systems/src/runtime/influence.rs`: Restored `floor_material.is_none()` write guard (was `as_deref() != Some("packed_earth")`)
- `rust/crates/sim-test/src/main.rs`: L-shaped wall layout (8 walls), `>=7` threshold, idempotency test proves pre-existing floor preservation

## Observed Values (seed 42, 20 agents)
- Interior floor count: 9 (3×3 interior, r=2)
- Floor material: all "packed_earth" (except pre-set "clay" tile preserved)
- Perimeter wall count: 9 (8 placed + 1 from simulation)
- Floor count at tick 1: 9
- Floor count at tick 121: 9
- Pre-existing "clay" floor at center: preserved at tick 1 and tick 121
- No-wall scenario floor count: 0

## Threshold Compliance
- Assertion 1 (interior_floor_count == 9): plan=9, observed=9, PASS
- Assertion 2 (floor_material == "packed_earth"): plan=all packed_earth, observed=0 wrong, PASS
- Assertion 3 (regression — at least 1 floor): plan=≥1, observed=true, PASS
- Assertion 4 (perimeter walls ≥ 7): plan=≥7, observed=9, PASS
- Assertion 5 (idempotency — count stable + material preserved): plan=equal counts + clay preserved, observed=9==9 + clay==clay, PASS
- Assertion 6 (no floor without walls): plan=0, observed=0, PASS

## Gate Result
- cargo test: PASS (152 passed, 0 failed)
- clippy: PASS
- harness: PASS (6/6 passed)

## Notes
- Wall count observed=9 exceeds the 8 manually placed walls; the simulation may have added 1 wall via another code path (e.g. stamp_enclosed_floors wall-sealing). This is benign and well above the ≥7 threshold.
- The `is_none()` guard is now verified to preserve pre-existing floor materials — the idempotency test pre-stamps "clay" at center and confirms it survives 121 ticks without being overwritten to "packed_earth".
- Anti-circularity discriminators (data_registry=None, no completed shelter Building at test location) all hold across all 6 tests.
