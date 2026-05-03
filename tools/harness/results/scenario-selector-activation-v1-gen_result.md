---
feature: scenario-selector-activation-v1
code_attempt: 2
---

## Files Changed
- `rust/crates/sim-test/src/main.rs`: Added `make_resources_with_two_scenarios()` helper and `harness_a9b_cross_scenario_switch_eternal_winter_to_perpetual_summer` test (Assertion 13) to the `harness_a9b_scenario_activation` module. No production code changes.

## Observed Values (seed 42, 0 agents — 0-tick SimResources assertions)
- Assertion 1 — season_mode after EternalWinter activation: "eternal_winter", get_active_scenario_name(): Some("EternalWinter")
- Assertion 2 — get_active_scenario_name() before and after apply_world_rules(): None, None
- Assertion 3 — activate_scenario_by_name("NonExistentScenario"): Err(...)
- Assertion 4 — season_for_tick at 0/5000/999999 under EternalWinter: "winter", "winter", "winter"
- Assertion 5 — season_for_tick at 0/5000/999999 under PerpetualSummer: "summer", "summer", "summer"
- Assertion 6 — season_mode/name after two EternalWinter activations: "eternal_winter", Some("EternalWinter")
- Assertion 7 — EternalWinter GlobalConstants: hunger_decay_rate=HUNGER_DECAY_RATE*1.3, warmth_decay_rate=WARMTH_DECAY_RATE*2.0, food_regen_mul=0.2, wood_regen_mul=0.5, farming_enabled=false, temperature_bias=-0.7
- Assertion 8 — EternalWinter AgentConstants: mortality_mul=1.3, skill_xp_mul=1.5, fertility_mul=0.7, lifespan_mul=0.8, body_potential_mul=1.0 (None-preserved), move_speed_mul=1.0 (None-preserved)
- Assertion 9 — hunger_decay_rate/warmth_decay_rate after 2× EternalWinter: HUNGER_DECAY_RATE*1.3 / WARMTH_DECAY_RATE*2.0 (no accumulation)
- Assertion 10 — BarrenWorld disaster_frequency_mul: 1.5
- Assertion 11 — PerpetualSummer hunger_decay_rate/warmth_decay_rate: HUNGER_DECAY_RATE*0.9 / WARMTH_DECAY_RATE*0.3
- Assertion 12 — PerpetualSummer food_regen_mul=1.5, body_potential_mul=1.0, move_speed_mul=1.0, disaster_frequency_mul=1.0
- Assertion 13 — After EternalWinter→PerpetualSummer switch: season_mode="eternal_summer", name=Some("PerpetualSummer"), hunger_decay_rate=HUNGER_DECAY_RATE*0.9, farming_enabled=true, mortality_mul=0.85

## Threshold Compliance
- Assertion 1 (activate_eternal_winter_sets_season_mode_and_name): plan=season_mode=="eternal_winter" AND name==Some("EternalWinter"), observed=PASS, PASS
- Assertion 2 (default_has_no_active_scenario_before_and_after_apply_world_rules): plan==None (both), observed=None / None, PASS
- Assertion 3 (unknown_scenario_returns_err): plan=is_err()==true, observed=Err(...), PASS
- Assertion 4 (eternal_winter_season_for_tick_always_winter): plan=="winter" at 0/5000/999999, observed=PASS
- Assertion 5 (perpetual_summer_season_for_tick_always_summer): plan=="summer" at 0/5000/999999, observed=PASS
- Assertion 6 (idempotent_double_activation_preserves_mode_and_name): plan=unchanged strings, observed=PASS
- Assertion 7 (eternal_winter_all_global_constants_applied): plan=6 field thresholds (abs<1e-12), observed=PASS
- Assertion 8 (eternal_winter_all_agent_constants_applied): plan=6 field thresholds (abs<1e-12), observed=PASS
- Assertion 9 (idempotent_double_activation_no_multiplier_accumulation): plan=HUNGER_DECAY_RATE*1.3 (not *1.69), observed=PASS
- Assertion 10 (barren_world_disaster_frequency_discriminative_value): plan==1.5 (abs<1e-12), observed=1.5, PASS
- Assertion 11 (perpetual_summer_global_constants_numeric_applied): plan=HUNGER_DECAY_RATE*0.9/WARMTH_DECAY_RATE*0.3 (abs<1e-12), observed=PASS
- Assertion 12 (perpetual_summer_food_regen_set_and_none_fields_preserved): plan=1.5/1.0/1.0/1.0 (abs<1e-12), observed=PASS
- Assertion 13 (cross_scenario_switch_eternal_winter_to_perpetual_summer): plan=5 sub-checks (season_mode/name/hunger_decay/farming_enabled/mortality_mul), observed=PASS

## Gate Result
- cargo test (harness_a9b filter): PASS (13 passed, 0 failed)
- cargo test --workspace: PASS for a9b (27 pre-existing failures in unrelated features: food_economy, shelter, wall_click, sprite_regression — none in harness_a9b_* namespace)
- clippy: PASS (exit code 0)
- harness: PASS (13/13 passed)

## Notes
- This was a test-only change (code_attempt 2). Production code was already correct per Evaluator verdict on attempt 1.
- The sole gap was Assertion 13 missing from the module. Added `make_resources_with_two_scenarios()` helper (loads BaseRules + two named scenarios into one registry) so both `activate_scenario_by_name()` calls resolve without Err.
- The 27 pre-existing workspace failures are unrelated to this feature and were present before this attempt. They are not regressions introduced by this change.
- Assertion 10 fragility (BarrenWorld.ron disaster_frequency_mul=1.5) documented per plan: if the RON value is changed to 1.0 the assertion loses discriminative power.
