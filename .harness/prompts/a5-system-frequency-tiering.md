# Feature: a5-system-frequency-tiering

## Summary

A-5 System Frequency Tiering — formalize the previously implicit Hot/Warm/Cold
classification of the 62 default runtime systems via a `TickTier` enum and
auto-inference from `tick_interval`. Metadata-only — execution logic unchanged.

The codebase had `tick_interval` declared on each system but no formal tier
enum or grouping API. Comments mentioned "Hot-tier and Warm-tier" without
backing types. This feature adds the missing classification layer.

## Changes Made

### rust/crates/sim-core/src/tick_tier.rs (new)
- `TickTier` enum (Hot, Warm, Cold) with serde derives
- `TickTier::from_interval(i32) -> TickTier`:
  - Hot:  tick_interval ≤ 2  (≥30 Hz at 60 FPS)
  - Warm: 3..=30
  - Cold: ≥ 31
- `TickTier::name()` → stable lowercase identifier ("hot"/"warm"/"cold")
- 2 unit tests (boundary correctness + name stability)

### rust/crates/sim-core/src/lib.rs
- `pub mod tick_tier;`
- `pub use tick_tier::TickTier;`

### rust/crates/sim-bridge/src/runtime_system.rs
- `impl DefaultRuntimeSystemSpec { fn tier() }` — auto-classify per spec
- `pub fn spec_tier(system_id: &str) -> Option<TickTier>` — lookup by name
- `pub fn tier_distribution() -> (usize, usize, usize)` — (hot, warm, cold)
- `pub fn systems_by_tier(tier: TickTier) -> Vec<&'static str>`
- `pub fn default_runtime_systems_count() -> usize` — regression guard helper

### rust/crates/sim-bridge/src/lib.rs
- Re-exports: `tier_distribution`, `spec_tier`, `systems_by_tier`,
  `default_runtime_systems_count`
- New `#[func]` debug APIs:
  - `runtime_tier_distribution() -> VarDictionary` (keys: hot/warm/cold/total)
  - `runtime_systems_by_tier(tier_str: GString) -> Array<GString>`

### rust/crates/sim-test/src/main.rs
- 5 harness tests (all pass):
  - `harness_a5_tier_enum_classifies_correctly` (A1): boundary correctness
  - `harness_a5_all_systems_have_tier` (A2): every default system tiered
  - `harness_a5_distribution_sanity` (A3): Hot=12 Warm=25 Cold=25 Total=62
  - `harness_a5_specific_system_tiers` (A4): temperament_shift_system → Hot
  - `harness_a5_default_runtime_systems_size_unchanged` (A5): 62 entries

## Scope

- DEFAULT_RUNTIME_SYSTEMS list **unchanged** (62 entries verified)
- No `tick_interval` values modified
- No system priorities changed
- No tier-based execution / parallelism (future work)
- No UI debug panel (SimBridge API only)
- No override mechanism (auto-inference only)

## Verification

- cargo check: clean
- cargo clippy: clean
- 5 A-5 harness tests: PASS
- Full sim-test suite: 1145+ passed, 0 failed (1140 prior + 5 new)
- Real distribution: Hot=12 Warm=25 Cold=25 Total=62

## Roadmap v4 Status

| Prereq | State |
|--------|-------|
| A-3 Effect Primitive | DONE |
| A-4 Causal Tracking | DONE |
| A-5 System Frequency Tiering | **DONE (this feature)** |
| A-6 Room BFS | DONE |
| A-8 Temperament | DONE |

5/13 → 6/13 prerequisite items complete.
