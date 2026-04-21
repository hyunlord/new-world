# ritual-system-v1 — Implement ActionType::Pray

## Section 1: Implementation Intent

Complete the `ActionType::Pray` behavior so agents with low Comfort near a totem will
autonomously perform a prayer ritual, restoring Comfort and Meaning. This grounds the
faith/ritual system in agent cognition without requiring a room — any totem placed on
the tile grid within Chebyshev radius 3 is sufficient.

## Section 2: What to Build

### sim-core/src/config.rs
Add 5 constants after `ACTION_TIMER_SEEK_SHELTER`:
- `ACTION_TIMER_PRAY: i32 = 6`
- `COMFORT_LOW: f64 = 0.35`
- `PRAY_COMFORT_RESTORE: f64 = 0.08`
- `PRAY_MEANING_BONUS: f64 = 0.02`
- `PRAY_TOTEM_SEARCH_RADIUS: i32 = 3`

### sim-core/src/tile_grid.rs
Add `has_furniture_within_radius(ax: i32, ay: i32, radius: i32, furniture_id: &str) -> bool`
using Chebyshev distance (square search). Already done in latest commit.

### sim-systems/src/runtime/cognition.rs
1. Add `ActionType::Pray => config::ACTION_TIMER_PRAY` to `action_timer()` match
2. Add `has_nearby_totem: bool` parameter to `behavior_select_action()`
3. Score Pray when `has_nearby_totem`: `behavior_urgency(1.0 - comfort) * 0.4 + 0.2`
4. Add `ActionType::Pray` to `BEHAVIOR_ACTION_ORDER` array (was missing — Pray scored
   but could never be selected without this entry)
5. Pre-compute `has_nearby_totem = comfort < COMFORT_LOW && tile_grid.has_furniture_within_radius(...)`
   in the BehaviorRuntimeSystem caller loop

### sim-systems/src/runtime/world.rs
Add `ActionType::Pray =>` handler in the movement_system completion match:
- Re-check totem presence at completion (agent may have moved)
- Apply `Comfort += PRAY_COMFORT_RESTORE` and `Meaning += PRAY_MEANING_BONUS` if totem found

### sim-test/src/main.rs (inside mod tests)
Add 2 harness tests:
- `harness_pray_action_restores_comfort`: 5 agents primed to Comfort=0.1, near totem
  vs same without totem; assert total comfort is higher with totem after 200 ticks
- `harness_pray_requires_nearby_totem`: near totem (129,128) vs far totem (10,10);
  assert near > far total comfort after 200 ticks

## Section 3: How to Implement

The feature is already implemented. The harness evaluates the existing implementation.

Key decisions:
- `has_nearby_totem` is pre-computed as a bool before `behavior_select_action()` because
  that function does not take `resources` directly
- `Needs::default()` gives `[1.0; 14]` — Comfort starts at 1.0 (no decay system), so
  harness tests prime agents to 0.1 before ticking to ensure the threshold is crossed
- `ActionType::Pray` MUST be in `BEHAVIOR_ACTION_ORDER` or it can never be selected
  even when scored (this was the root bug found and fixed)

## Section 4: Dispatch Plan

All work direct (already implemented).

## Section 5: Localization Checklist

No new localization keys. `STATUS_PRAY` already exists.

## Section 6: Verification

```bash
cargo test -p sim-test harness_pray -- --nocapture
# Expected: 2 passed

cargo clippy -p sim-core -p sim-systems -- -D warnings
# Expected: no errors
```

Test results:
- `harness_pray_action_restores_comfort`: comfort_with_totem=13.62 > comfort_without_totem=12.50 ✓
- `harness_pray_requires_nearby_totem`: comfort_near=0.90 > comfort_far=0.50 ✓
