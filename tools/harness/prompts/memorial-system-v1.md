# memorial-system-v1 — Implement ActionType::Mourn

## Section 1: Implementation Intent

Complete the `ActionType::Mourn` behavior so agents with high Sadness near a cairn will
autonomously perform a mourning ritual, reducing Sadness and restoring Meaning. This is
the third "asset→gameplay" connection feature (after ritual-system-v1 and shelter-quality-modifier),
activating the already-merged Round 1 cairn sprites. Mourn is driven by `EmotionType::Sadness`
(the first emotion-layer integration into gameplay), not a need.

## Section 2: What to Build

### sim-core/src/enums.rs
Add `Mourn` variant at the END of `ActionType` (after `PlaceFurniture`, discriminant 30).

### sim-core/src/config.rs
Add 5 constants after `PRAY_TOTEM_SEARCH_RADIUS`:
- `ACTION_TIMER_MOURN: i32 = 8`
- `MOURN_SADNESS_THRESHOLD: f64 = 0.35`
- `MOURN_SADNESS_RELIEF: f64 = 0.15`
- `MOURN_MEANING_BONUS: f64 = 0.04`
- `MOURN_CAIRN_SEARCH_RADIUS: i32 = 3`

### sim-systems/src/runtime/cognition.rs
1. Add `ActionType::Mourn => config::ACTION_TIMER_MOURN` to `action_timer()` match
2. Add `ActionType::Mourn => (0.0, 0.4, 0.6, 0.1)` to `temperament_action_bias()` (HA/RD dominant)
3. Add `ActionType::Mourn` to `BEHAVIOR_ACTION_ORDER` array (size 21)
4. Add `has_nearby_cairn: bool` parameter to `behavior_select_action()`
5. Score Mourn when `has_nearby_cairn`: `behavior_urgency(sadness) * 0.5 + 0.1`
6. Pre-compute `has_nearby_cairn` in the BehaviorRuntimeSystem caller loop using
   `resources.buildings.values()` (not tile_grid) — cairns are ECS buildings, not furniture

### sim-systems/src/runtime/world.rs
Add `ActionType::Mourn =>` handler in the movement_system completion match:
- Re-check cairn presence at completion (Chebyshev radius)
- Apply `Sadness += -MOURN_SADNESS_RELIEF` (emotion.add with negative delta)
- Apply `Meaning += MOURN_MEANING_BONUS`

### sim-engine/src/frame_snapshot.rs
Add `ActionType::Mourn => 30` to `action_state_code` exhaustive match.

### sim-test/src/main.rs (inside mod tests)
Add 4 harness tests:
- `harness_memorial_mourn_action_registered` (A1): discriminant checks (Mourn=30, Pray=18, Sadness=4)
- `harness_memorial_mourn_reduces_sadness` (A2): near-cairn vs no-cairn, delta >= 0.08 on Sadness sum
- `harness_memorial_mourn_no_effect_without_cairn` (A3): near-cairn vs far-cairn (10,10),
  far-near sadness delta >= 0.05
- `harness_memorial_natural_mourn_over_simulation` (A4): Meaning delta > 0.01 over 2000 ticks

### localization/en/actions.json and localization/ko/actions.json
Add `"ACTION_MOURN": "Mourn"` (en) and `"ACTION_MOURN": "애도"` (ko).

## Section 3: How to Implement

The feature is already implemented. The harness evaluates the existing implementation.

Key decisions:
- `Mourn` is appended as discriminant 30 (after PlaceFurniture=29) to avoid breaking existing
  GDScript icon mapping (discriminants 0-29 are hardcoded in entity_renderer.gd)
- `has_nearby_cairn` uses `resources.buildings` (HashMap<BuildingId, Building>) not tile_grid,
  because cairns are spawned as ECS buildings, not furniture tiles
- `emotion.add(EmotionType::Sadness, -RELIEF)` auto-clamps to [0, 1] — no explicit `.set()` needed
- TCI affinity `(0.0, 0.4, 0.6, 0.1)`: HA and RD dominant personalities mourn more
  (harm avoidance + reward dependence = empathic/anxious = grief-prone)
- Natural emotion decay is ~85% over 200 ticks — A3 uses near-cairn vs far-cairn comparison
  rather than absolute threshold to isolate the radius gate
- A4 uses 2000 ticks to capture Meaning accumulation across sporadic Mourn completions

## Section 4: Dispatch Plan

All work direct (already implemented).

## Section 5: Localization Checklist

| Key | File | en value | ko value |
|-----|------|----------|----------|
| ACTION_MOURN | actions.json | Mourn | 애도 |

## Section 6: Verification

```bash
cargo test -p sim-test harness_memorial -- --nocapture
# Expected: 4 passed

cargo test --workspace
# Expected: 293+ passed, 0 failed

cargo clippy --workspace -- -D warnings
# Expected: no errors
```

Test results:
- `harness_memorial_mourn_action_registered`: ActionType::Mourn=30 Pray=18 EmotionType::Sadness=4 ✓
- `harness_memorial_mourn_reduces_sadness`: sadness_delta=0.1346 >= 0.08 ✓
- `harness_memorial_mourn_no_effect_without_cairn`: delta(far-near)=0.1346 >= 0.05 ✓
- `harness_memorial_natural_mourn_over_simulation`: meaning_delta=0.6889 > 0.01 ✓
