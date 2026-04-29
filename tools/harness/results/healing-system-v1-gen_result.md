---
feature: healing-system-v1
code_attempt: 2
---

## Files Changed
- `rust/crates/sim-test/src/main.rs`: Fixed `harness_b1_natural_clot_reduces_bleed_rate` assertion from weakened direction-only `bleed_rate < 5` to plan-required exact-value `assert_eq!(bleed_rate, 2)`

## Observed Values (seed 42, 20 agents)
- bleed_rate after 3 ticks (initial=5): 2
- BLEEDING flag cleared after 2 ticks (initial bleed_rate=2): true, bleed_rate==0
- hp after 10 ticks (initial=70, no wound flags): 80 (>= 71)
- Skills XP after add_xp calls: 1.5 (> 0)
- has_knowledge before learn(): false; after learn(): true
- target hp after Treat action (initial=50): 81 (> 50)
- treater SKILL_HEALING XP after Treat: 0.5 (> 0)
- base treater (level=0) hp healed: 51; skilled treater (level=10) hp healed: 81
- alive agents after 10 ticks (BLEEDING brain hp=3, bleed_rate=1): 20/20 (fraction=1.0)

## Threshold Compliance
- Assertion 1 (natural_clot_reduces_bleed_rate): plan==2, observed=2, PASS
- Assertion 2 (natural_clot_clears_bleeding_flag): plan=BLEEDING cleared + bleed_rate==0, observed=true+0, PASS
- Assertion 3 (natural_heal_restores_hp): plan>=71, observed=80, PASS
- Assertion 4 (skills_xp_accumulates): plan>0, observed=1.5, PASS
- Assertion 5 (knowledge_has_knowledge_invariant): plan=false→true, observed=false→true, PASS
- Assertion 6 (treat_action_heals_injured_target): plan>50, observed=81, PASS
- Assertion 7 (treat_awards_xp_to_treater): plan>0, observed=0.5, PASS
- Assertion 8 (treat_skilled_heals_more_than_unskilled): plan=skilled>base, observed=81>51, PASS
- Assertion 9 (population_survives_clottable_wounds): plan>=0.50, observed=1.0 (20/20), PASS

## Gate Result
- cargo test: PASS (423 passed, 26 failed — all 26 are pre-existing failures, no regressions)
- clippy: PASS
- harness: PASS (9/9 passed)

## Notes
- The only change in attempt 2 vs attempt 1 is fixing the weakened assertion in `harness_b1_natural_clot_reduces_bleed_rate`: `assert!(bleed_rate < 5)` → `assert_eq!(bleed_rate, 2)`. The implementation was already correct and the observed value is exactly 2 as the plan requires.
- 26 pre-existing failures are unchanged from attempt 1 (harness_work_injury_*, harness_action_non_idle_ratio_*, harness_a9_behavioral_*, harness_blueprint_*, harness_building_visuals_*, harness_locale_*, harness_food_economy_*, harness_ns_biases_*, harness_memorial_*, harness_p2b3_*, harness_pray_*, harness_shelter_*, harness_sprite_*, harness_temperament_*, harness_population_growth_*, harness_wall_click_info_*).
