# a9-disaster-frequency-cache-and-harness

Feature slug: a9-disaster-frequency-cache-and-harness
Mode: --full
Production code changes: ~10 lines (sim-engine/src/engine.rs)

## Context

A-9 World Rules system is 95% implemented. All 8 `GlobalConstants` RON fields exist
and are parsed, but `disaster_frequency_mul` was the only one missing a cache field
in `SimResources` and a handler in `apply_world_rules()`. This breaks API consistency.

Additionally, 4 fields (season_mode, farming_enabled, temperature_bias,
disaster_frequency_mul) had zero harness coverage despite being cached.

## What Was Built

### rust/crates/sim-engine/src/engine.rs

1. New `SimResources` field:
   ```rust
   /// Disaster frequency multiplier (0.0 = no disasters, 1.0 = default, 10.0 = max).
   /// Cached only — disaster system not yet implemented (TODO: disaster-system-v1).
   pub disaster_frequency_mul: f64,
   ```

2. Default initialization: `disaster_frequency_mul: 1.0`

3. `apply_world_rules()` handler:
   ```rust
   if let Some(mul) = globals.disaster_frequency_mul {
       self.disaster_frequency_mul = mul.clamp(0.0, 10.0);
   }
   ```

4. Extended `info!` log to include `disaster_freq={:.2}`.

5. TODO comment on `season_mode` field: `// TODO(A-9 phase 2): integrate with season-system-v1`

### rust/crates/sim-test/src/main.rs

5 harness tests added:

1. `harness_a9_disaster_frequency_mul_default_value` — default is 1.0 when no RON override
2. `harness_a9_disaster_frequency_mul_applied_from_ron` — RON → SimResources cache transfer
3. `harness_a9_unused_global_constants_cache_correctly` — season_mode/farming_enabled/temperature_bias cache
4. `harness_a9_global_constants_clamping_works` — disaster_frequency_mul clamps to 10.0, temperature_bias clamps to -1.0
5. `harness_a9_all_eight_globals_cached` — compile-time regression guard for all 8 fields

## A-9 Completion Status

- ✅ 8 GlobalConstants all have SimResources cache fields (this feature)
- ✅ apply_world_rules() handles all 8 consistently
- ✅ 22 a9 harness tests pass (17 existing + 5 new)
- 🟡 Consumer systems pending as separate features:
  - disaster-system-v1, season-system-v1, farming-system-v1, climate-system-v1

## Gate Results

cargo test --workspace: passed (0 failed)
cargo clippy --workspace -- -D warnings: PASS (exit 0)
harness_a9 (22 tests): 22 passed, 0 failed
