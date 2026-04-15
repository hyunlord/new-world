---
feature: a8-temperament
code_attempt: 1
---

## Files Changed
- `rust/crates/sim-core/src/temperament.rs`: Fixed `snap_if_sub_minimum()` floating-point comparison — added epsilon tolerance (1e-9) so deltas exactly at TEMPERAMENT_MIN_SHIFT_DELTA (0.05) are not erroneously snapped back.
- `rust/crates/sim-test/src/main.rs`: Replaced old A-8 harness test section (lines 11310-12034) with 15 new plan v3 assertions.

## Observed Values (seed 42, 20 agents)
- Non-zero bias action count: 26/30
- NS→Explore bias (NS=0.9): +0.054
- HA→Flee bias (HA=0.9): +0.054, HA→SeekShelter: +0.042
- RD→Socialize bias (RD=0.9): +0.048, RD→Teach: +0.036
- P→Build bias (P=0.9): +0.042, P→GatherStone: +0.030
- HA→Hunt bias (HA=0.9): -0.024 (suppressed)
- Eat/Drink bias: 0.000000 across all axis configs
- Event keys accepted: 9/9 recognized, 1/1 unrecognized rejected
- Directional sign checks: 23/23 passed
- Shift magnitude range: [0.05, 0.15] — all 23 nonzero deltas within bounds
- Lifetime cap: shift_count=3 after 3 shifts, 4th rejected
- Axis clamping: all axes in [0.0, 1.0] at boundaries
- Agents with Temperament: 25/25 (≥20 threshold)
- Axis std devs: NS=0.158, HA=0.220, RD=0.094, P=0.264 — all 4 axes > 0.05
- Agents with shift_count > 0 after 4380 ticks: 56/56

## Threshold Compliance
- Assertion 1 (bias_coverage_breadth): plan=≥20, observed=26, PASS
- Assertion 2 (ns_explore_sign): plan=bias>0, observed=+0.054, PASS
- Assertion 3 (ha_flee_seekshelter_sign): plan=bias>0, observed=Flee+0.054/Shelter+0.042, PASS
- Assertion 4 (rd_socialize_teach_sign): plan=bias>0, observed=Social+0.048/Teach+0.036, PASS
- Assertion 5 (p_build_gatherstone_sign): plan=bias>0, observed=Build+0.042/Stone+0.030, PASS
- Assertion 6 (ha_hunt_suppression): plan=bias<0, observed=-0.024, PASS
- Assertion 7 (metabolic_neutrality): plan=exactly 0.0, observed=0.0, PASS
- Assertion 8 (shift_event_key_acceptance): plan=9 accepted + 1 rejected, observed=9+1, PASS
- Assertion 9 (shift_directional_signs): plan=22+ sign checks, observed=23/23, PASS
- Assertion 10 (shift_magnitude_bounds): plan=[0.05,0.15], observed=all within, PASS
- Assertion 11 (shift_lifetime_cap): plan=4th rejected + count=3, observed=correct, PASS
- Assertion 12 (shift_axis_clamping): plan=[0.0,1.0], observed=all within, PASS
- Assertion 13 (all_agents_present): plan=≥20, observed=25, PASS
- Assertion 14 (axis_diversity): plan=≥2 axes std>0.05, observed=4/4, PASS
- Assertion 15 (shift_fires_integration): plan=≥1 shifted, observed=56, PASS

## Gate Result
- cargo test: PASS (991 passed, 0 failed)
- clippy: PASS
- harness: PASS (15/15 plan v3 assertions passed)

## Notes
- The RED phase caught a real floating-point bug in `snap_if_sub_minimum()`: deltas of exactly 0.05 (the minimum) were being snapped back to latent because `(0.5 - 0.05) - 0.5` can produce a value slightly less than 0.05 in IEEE 754 arithmetic. Fixed by adding epsilon tolerance.
- Plan says "22 directional sign checks" but actual nonzero delta count across 9 events × 4 axes is 23. The test checks all 23 and requires ≥22, so no threshold conflict.
- Assertion 13 observed 25 agents (not 20) because births occur during the 100-tick warm-up. The test uses ≥20 threshold per plan.
- Assertion 15 observed 56/56 agents shifted after 4380 ticks — far exceeding the ≥1 threshold. This is because starvation recovery triggers frequently in the test environment.
- The old plan v2 harness tests (17 functions at lines 11310-12034) were replaced with the 15 plan v3 assertions. The 2 earlier plan v1 tests (lines 2832-3440) remain as supplementary coverage.
