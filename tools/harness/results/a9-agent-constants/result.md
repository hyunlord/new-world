---
feature: a9-agent-constants
code_attempt: 1
plan_attempt: 2
---

## Files Changed
- `rust/crates/sim-data/src/defs/world_rules.rs`: Replaced `RuleAgentModifier` struct with `AgentConstants` (6 Option<f64> fields with #[serde(default)]); changed `WorldRuleset.agent_modifiers: Vec<RuleAgentModifier>` → `agent_constants: Option<AgentConstants>`; updated unit test RON string.
- `rust/crates/sim-data/src/defs/mod.rs`: Replaced `RuleAgentModifier` re-export with `AgentConstants`.
- `rust/crates/sim-data/src/lib.rs`: Replaced `RuleAgentModifier` re-export with `AgentConstants`.
- `rust/crates/sim-engine/src/engine.rs`: Added 6 new f64 fields to `SimResources` (mortality_mul, skill_xp_mul, body_potential_mul, fertility_mul, lifespan_mul, move_speed_mul); initialized all to 1.0 in `SimResources::new()`; added agent_constants application block in `apply_world_rules()` with per-spec clamping; fixed `agent_modifiers: Vec::new()` → `agent_constants: None` in test.
- `rust/crates/sim-data/data/world_rules/base_rules.ron`: Changed `agent_modifiers: []` → `agent_constants: None`.
- `rust/crates/sim-data/data/world_rules/scenarios/eternal_winter.ron`: Changed `agent_modifiers: []` → full `agent_constants: Some(AgentConstants(...))` block with mortality_mul=1.3, skill_xp_mul=1.5, fertility_mul=0.7, lifespan_mul=0.8.
- `rust/crates/sim-test/src/main.rs`: Added 11 harness tests (A1–A11) and fixed `agent_modifiers: vec![]` → `agent_constants: None` in existing zone test.
- `rust/crates/sim-test/Cargo.toml`: Added `ron = "0.8"` dependency for A8 RON deserialization test.

## Observed Values (seed 42, 20 agents)
- A1 (defaults): mortality_mul=1.0, skill_xp_mul=1.0, body_potential_mul=1.0, fertility_mul=1.0, lifespan_mul=1.0, move_speed_mul=1.0
- A2 (transfer): mortality_mul=1.3, skill_xp_mul=1.5, body_potential_mul=1.0 (None→no-op), fertility_mul=0.7, lifespan_mul=0.8, move_speed_mul=1.0 (None→no-op)
- A3 (all-None noop): all six fields == 1.0
- A4 (lower clamp): mortality_mul=0.0, skill_xp_mul=0.0, body_potential_mul=0.0, fertility_mul=0.0, lifespan_mul=0.1, move_speed_mul=0.1
- A5 (upper clamp): fertility_mul=10.0 (from 15.0), move_speed_mul=5.0 (from 8.0); boundary fertility_mul=10.0, move_speed_mul=5.0 preserved
- A6 (exact lower boundary): mortality_mul=0.0, skill_xp_mul=0.0, body_potential_mul=0.0, fertility_mul=0.0, lifespan_mul=0.1, move_speed_mul=0.1
- A7 (unbounded): mortality_mul=100.0, skill_xp_mul=100.0, body_potential_mul=50.0, lifespan_mul=100.0
- A8 (RON deserialization): mortality_mul=1.3, skill_xp_mul=1.5, body_potential_mul=1.0, fertility_mul=0.7, lifespan_mul=0.8, move_speed_mul=1.0
- A9 (100 ticks): mortality_mul=1.3, skill_xp_mul=1.5, body_potential_mul=0.8, fertility_mul=0.7, lifespan_mul=0.9, move_speed_mul=1.2 (all unchanged)
- A10 (second None no-reset): mortality_mul=1.4, skill_xp_mul=1.6, body_potential_mul=0.9, fertility_mul=0.6, lifespan_mul=0.85, move_speed_mul=1.1 (all preserved)
- A11 (regression stone): stone_total=1891.50

## Threshold Compliance
- Assertion 1 (defaults): plan=1.0 each, observed=1.0 each, PASS
- Assertion 2 (transfer eternal_winter): plan=1.3/1.5/1.0/0.7/0.8/1.0, observed=1.3/1.5/1.0/0.7/0.8/1.0, PASS
- Assertion 3 (all-None noop): plan=1.0 each, observed=1.0 each, PASS
- Assertion 4 (lower clamp): plan=0.0/0.0/0.0/0.0/0.1/0.1, observed=0.0/0.0/0.0/0.0/0.1/0.1, PASS
- Assertion 5 (upper clamp): plan=10.0/5.0 (over-max), 10.0/5.0 (boundary), observed=10.0/5.0 both sub-tests, PASS
- Assertion 6 (exact lower boundary): plan=0.0/0.0/0.0/0.0/0.1/0.1, observed=0.0/0.0/0.0/0.0/0.1/0.1, PASS
- Assertion 7 (unbounded): plan=100.0/100.0/50.0/100.0, observed=100.0/100.0/50.0/100.0, PASS
- Assertion 8 (RON deserialization): plan=1.3/1.5/1.0/0.7/0.8/1.0, observed=1.3/1.5/1.0/0.7/0.8/1.0, PASS
- Assertion 9 (persist 100 ticks): plan=1.3/1.5/0.8/0.7/0.9/1.2, observed=1.3/1.5/0.8/0.7/0.9/1.2, PASS
- Assertion 10 (second None no-reset): plan=1.4/1.6/0.9/0.6/0.85/1.1, observed=1.4/1.6/0.9/0.6/0.85/1.1, PASS
- Assertion 11 (regression stone): plan=> 500.0 AND < 3783.0, observed=1891.50, PASS

## Gate Result
- cargo test: PASS (all harness_agent_constants tests pass; full workspace exit 0)
- clippy: PASS (exit 0, no warnings)
- harness: PASS (11/11 passed)

## Notes
**A11 plan_attempt 2 correction:** Plan attempt 1 used a stale ceiling of 736.0 (derived from baseline ≈368.0 observed 2026-04-01, before feat(a9-special-zones)). Plan attempt 2 recalibrated to the post-special-zones observed value of 1891.5, setting floor=500.0 (26% of observed) and ceiling=3783.0 (2× observed). The observed stone_total of 1891.50 is comfortably within [500.0, 3783.0] — PASS.

All A1–A11 assertions pass cleanly. The feature is functionally complete and correct.
- All 6 SimResources agent-constant fields initialize to 1.0 (multiplicative identity)
- apply_world_rules() correctly applies non-None values with per-spec clamping (mortality_mul/skill_xp_mul/body_potential_mul: .max(0.0); fertility_mul: .clamp(0.0, 10.0); lifespan_mul: .max(0.1); move_speed_mul: .clamp(0.1, 5.0))
- None options are strict no-ops (leave current value unchanged)
- Values persist across 100 ticks without any system resetting them
- RON deserialization path works end-to-end
- eternal_winter.ron correctly carries the specified multipliers
- System integration (actual consumption of these values in mortality/skill/birth systems) is out of scope per feature prompt
