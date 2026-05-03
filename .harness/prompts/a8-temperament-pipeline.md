# Feature: `a8-temperament-pipeline` — Production Registration

## Section 1: Implementation Intent

A-8 Temperament Pipeline was ~85% already implemented:
- ✅ `Temperament` ECS component (NS/HA/RD/P latent+expressed axes, genes, awakened, shift_count)
- ✅ `TemperamentRuleSet` with PRS weights, bias matrix, shift rules
- ✅ RON data at `sim-data/data/temperament/default_weights.ron`
- ✅ `TemperamentShiftRuntimeSystem` (starvation recovery Pass-2, event-store Pass-3)
- ✅ `temperament_action_bias()` in cognition — already called during action selection
- ✅ `TemperamentDetail` SimBridge exposure, UI panels

**The only gap**: `TemperamentShiftRuntimeSystem` was NOT in `DEFAULT_RUNTIME_SYSTEMS` (sim-bridge). It was only registered in sim-test (line 483). So in-game agents never underwent Cloninger shift events. `expressed` permanently equalled `latent`, `awakened` stayed `false` forever.

## Section 2: What to Build

### `rust/crates/sim-bridge/src/runtime_system.rs`

1. Add `TemperamentShift = 65` to `RuntimeSystemId` enum (after `Territory = 64`)
2. Add `Self::TemperamentShift => "temperament_shift_system"` to `registry_name()` match
3. Add `Self::TemperamentShift` to `all()` array
4. Change `DEFAULT_RUNTIME_SYSTEMS: [DefaultRuntimeSystemSpec; 61]` → `[62]`
5. Add spec after `Trait` (priority 100): `{ system_id: TemperamentShift, priority: 101, tick_interval: 1 }`
6. Add dispatch arm: `RuntimeSystemId::TemperamentShift => engine.register(TemperamentShiftRuntimeSystem::new(...))`
7. Add import: `TemperamentShiftRuntimeSystem` to use block
8. Add `pub fn default_runtime_system_registry_names() -> Vec<&'static str>` for sim-test access

### `rust/crates/sim-bridge/src/lib.rs`

Re-export `default_runtime_system_registry_names` as `pub use`.

### `rust/crates/sim-test/src/main.rs`

Three harness tests:
- `harness_a8_temperament_shift_registered_in_production` — checks "temperament_shift_system" in DEFAULT
- `harness_a8_temperament_starvation_recovery_shift` — injects starvation_recovery event → shift_count +1
- `harness_a8_temperament_awakened_after_shift` — same event → awakened=true

## Section 3: How to Implement

All changes are in sim-bridge (registration wiring) and sim-test (harness verification).
No changes to TemperamentShiftRuntimeSystem itself (already correct).

Priority 101 (same as Chronicle): runs after Trait (100), before any downstream consumers.
tick_interval=1: needed for starvation pass-2 hunger threshold tracking.

Harness tests use event_store injection (Pass-3 path) for reliability:
- Set hunger=1.0 to suppress Pass-2 interference
- Inject `SimEvent { tags: ["starvation_recovery"] }` → `sim_event_to_shift_key` returns "starvation_recovery"
- RON rule: starvation_recovery → HA +0.05, P -0.05, no conditions → always fires

## Section 4: Dispatch Plan

| # | Ticket | File | Mode |
|---|--------|------|:----:|
| T1 | Add TemperamentShift to enum, registry_name, all(), DEFAULT array, dispatch | sim-bridge/src/runtime_system.rs | 🔴 DIRECT |
| T2 | Add public accessor function + lib.rs re-export | sim-bridge/src/lib.rs | 🔴 DIRECT |
| T3 | 3 harness tests | sim-test/src/main.rs | 🔴 DIRECT |

## Section 5: Localization Checklist

No new localization keys. Temperament label keys already exist.

## Section 6: Verification

```bash
cd rust
cargo test -p sim-test harness_a8 -- --nocapture
# Expected: 3 passed

cargo test --workspace
# Expected: 280+ passed, 0 failed

cargo clippy --workspace -- -D warnings
# Expected: clean
```

Checks:
- `grep "DefaultRuntimeSystemSpec; 62"` → 1 match
- `grep "temperament_shift_system"` → in registry_names
- `grep "TemperamentShiftRuntimeSystem::new"` in sim-bridge → ≥1 match
