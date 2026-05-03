---
feature: a4-causal-tracking-exposure
code_attempt: 1
---

## Files Changed
- `rust/crates/sim-bridge/src/lib.rs`: Added `use sim_core::CausalEvent` import, `causal_events_to_vardict()` private helper, and `recent_causes` field (max 8, newest-first) to `runtime_get_entity_detail()` — present from prior harness pass
- `rust/crates/sim-test/src/main.rs`: Added 9 new plan-aligned harness tests (assertions 1–9) alongside the existing 3; total a4 harness coverage is now 12 tests
- `scripts/ui/panels/entity_detail_panel_v5.gd`: Modified `_refresh_events()` to read `recent_causes` from bridge dict; added "Recent Causes" section (top 5, localized label + signed magnitude, fallback to raw kind) — present from prior harness pass
- `localization/en/ui.json`: Added `UI_CAUSAL_RECENT`, `CAUSE_PRAY`, `CAUSE_MOURN`, `CAUSE_RITUAL_COMFORT`, `CAUSE_HEARTH_WARMTH`, `CAUSE_SHELTER_SAFETY`, `CAUSE_FOOD_GRADIENT`, `CAUSE_BAND_LEADER_ELECTED`, `CAUSE_CONFLICT`, `CAUSE_TOOL_BROKEN` — present from prior harness pass
- `localization/ko/ui.json`: Same 10 keys as EN with Korean translations — present from prior harness pass

## Observed Values (seed 42, 20 initial agents)

### Existing 3 assertions (unchanged)
- causal_log delta over 1000 ticks: 1184 (pre=0, post=1184)
- entity with recent causes after 2000 ticks: entity=0, 8 recent events
- pray causal entries after 200 ticks + totem + low Comfort: pray_count=4

### New 9 assertions (plan thresholds locked)
- accumulates_substantially delta (1000 ticks): 1184
- newest_first_ordering violations: 0
- ring_buffer_cap_enforced max_per_entity: 32 (cap=32)
- total_entries_bounded total (2000 ticks): 1376 [EXCEEDS plan upper_bound=640]
  — population grew 20→43 agents during 2000 ticks; 43×32=1376
- majority_of_agents_have_events: 43/43 agents have events
- agents_saturate_ring_buffer: 43/43 agents fully saturated (cap=32)
- all_events_have_valid_fields violations: 0
- pray_recorded_in_log pray_count: 4
- diverse_cause_kinds: 8 distinct kinds (band_formed, band_dissolved,
  band_split_overpopulation, band_leader_elected, band_promoted,
  food_gradient, danger_gradient, shelter_safety)

## Threshold Compliance

### Existing 3
- Assertion (causal_log_populated): plan=≥10 delta, observed=1184, PASS
- Assertion (entity_has_recent_causes): plan=≥1 entity with events, observed=entity 0 (8 events), PASS
- Assertion (pray_recorded): plan=≥1 kind='pray' entry, observed=4, PASS

### New 9
- Assertion 1 (accumulates_substantially): plan=delta≥100, observed=1184, PASS
- Assertion 2 (newest_first_ordering): plan=violations=0, observed=0, PASS
- Assertion 3 (ring_buffer_cap_enforced): plan=max≤32, observed=32, PASS
- Assertion 4 (total_entries_bounded): plan=total≤640 (20×32), observed=1376, **FAIL**
- Assertion 5 (majority_of_agents_have_events): plan=≥15/20 agents, observed=43/43, PASS
- Assertion 6 (agents_saturate_ring_buffer): plan=≥8/20 saturated, observed=43/43, PASS
- Assertion 7 (all_events_have_valid_fields): plan=violations=0, observed=0, PASS
- Assertion 8 (pray_recorded_in_log): plan=≥1 kind='pray', observed=4, PASS
- Assertion 9 (diverse_cause_kinds): plan=≥3 distinct kinds, observed=8, PASS

## Gate Result
- cargo test: FAIL (309 passed, 1 failed — harness_a4_total_entries_bounded, finished in 294.76s)
- clippy: PASS (0 warnings, finished in 0.89s incremental)
- harness (a4 filter): FAIL (11/12 passed — 3 old + 8/9 new pass; assertion 4 fails)

## Notes

### Threshold discrepancy — assertion 4 (total_entries_bounded) [DO NOT CHANGE]
The plan formula `20 × 32 = 640` hardcodes the initial agent count (20). After 2000 ticks,
births grow the population to 43 agents, making the actual maximum `43 × 32 = 1376`.

The ring buffer eviction is **correctly implemented** — assertion 3 passes (max_per_entity=32).
The total exceeds 640 solely because there are more agents than the plan assumed.
This is a planning assumption mismatch, not an implementation bug.

**Threshold NOT changed per HDD rules.**
Recommendation for RE-PLAN: replace `20 × 32 = 640` with
`current_agent_count × CAUSAL_LOG_MAX_PER_ENTITY` (dynamically evaluated at assertion time).

### All other observations
- 8 distinct cause kinds observed (threshold=3): band_formed, band_dissolved,
  band_split_overpopulation, band_leader_elected, band_promoted, food_gradient,
  danger_gradient, shelter_safety
- All 43 agents (initial 20 + 23 born) have full ring buffers after 2000 ticks
- Zero field integrity violations: every push site populates kind + summary_key correctly
- newest-first ordering invariant holds: 0 violations across all agents

### Prior attempt context
sim-bridge, entity_detail_panel_v5.gd, and localization keys were all already implemented
in the prior pass (the 3 original tests passed with PASS). This pass added the 9 stronger
plan assertions. All 9 new tests compiled and ran cleanly; only assertion 4's hardcoded
population formula produced a failure.
