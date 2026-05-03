---
feature: ritual-system-v1
code_attempt: 1
---

## Files Changed
- `rust/crates/sim-core/src/config.rs`: Added 5 constants — ACTION_TIMER_PRAY, COMFORT_LOW, PRAY_COMFORT_RESTORE, PRAY_MEANING_BONUS, PRAY_TOTEM_SEARCH_RADIUS
- `rust/crates/sim-core/src/tile_grid.rs`: Added `has_furniture_within_radius()` using Chebyshev (square) distance search
- `rust/crates/sim-systems/src/runtime/cognition.rs`: Added Pray to `action_timer()`, `has_nearby_totem: bool` param to `behavior_select_action()`, Pray scoring formula, Pray to `BEHAVIOR_ACTION_ORDER`, pre-computation of `has_nearby_totem` in BehaviorRuntimeSystem caller loop
- `rust/crates/sim-systems/src/runtime/world.rs`: Added `ActionType::Pray` completion handler — re-checks totem at completion, applies PRAY_COMFORT_RESTORE + PRAY_MEANING_BONUS if totem present
- `rust/crates/sim-test/src/main.rs`: Added `harness_pray_action_restores_comfort` and `harness_pray_requires_nearby_totem` tests

## Observed Values (seed 42/43, 5 agents, 200 ticks)
- comfort_with_totem (Assertion 1): 13.62 (5-agent sum)
- comfort_without_totem (Assertion 1): 12.50 (5-agent sum)
- δ₁ (Assertion 1): 1.12
- comfort_near (Assertion 2): 0.90 (5-agent sum, seed=43)
- comfort_far (Assertion 2): 0.50 (5-agent sum, seed=43)
- δ₂ (Assertion 2): 0.40

## Threshold Compliance
- Assertion 1 (harness_pray_action_restores_comfort): plan=δ≥0.34, observed=δ=1.12, **PASS**
- Assertion 2 (harness_pray_requires_nearby_totem): plan=δ≥0.12, observed=δ=0.40, **PASS**

## Gate Result
- cargo test: **PASS** (272 passed, 0 failed, 1 ignored — 273 total in sim-test)
- clippy: **PASS** (exit code 0, no warnings)
- harness: **PASS** (2/2 passed)

## Notes
- Root bug confirmed fixed: `ActionType::Pray` was scored but absent from `BEHAVIOR_ACTION_ORDER`, meaning it could never be selected. Adding it to the order array was the critical change that allowed the pray action to fire.
- `has_nearby_totem` is pre-computed as a `bool` before `behavior_select_action()` because that function does not receive `resources` directly — this is a deliberate architectural boundary.
- `Needs::default()` starts Comfort at 1.0 (no decay system active); harness tests prime agents to 0.1 before ticking to ensure the COMFORT_LOW=0.35 threshold is crossed.
- At completion, the Pray handler re-checks totem presence (agent may have moved), which is correct defensive behavior — no concern.
- Assertion 1 δ=1.12 is well above plan threshold 0.34; Assertion 2 δ=0.40 is well above plan threshold 0.12. Both margins comfortable.
- No threshold modifications were made. Plan thresholds honored exactly.
