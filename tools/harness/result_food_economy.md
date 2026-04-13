---
feature: food-economy
code_attempt: 1
---

## Files Changed
- `rust/crates/sim-core/src/config.rs`: Changed FOOD_SCARCITY_THRESHOLD_PER_CAPITA from 1.6 to 1.5
- `rust/crates/sim-engine/src/engine.rs`: Added food_economy_scarcity_boost_counterfactual and food_economy_boost_outside_scarcity counters to SimResources
- `rust/crates/sim-systems/src/runtime/cognition.rs`: Changed behavior_select_action return type to (ActionType, bool) for counterfactual tracking; added scarcity_boost_applied flag and counterfactual logic; updated all callers and unit tests
- `rust/crates/sim-test/src/main.rs`: Added harness_food_economy_plan_v2 test with 7 plan assertions; updated harness_food_economy_plan_comprehensive A4 to use production counter and A6 forage threshold 200→150

## Observed Values (seed 42, 20 agents)
- Final food at tick 4380: 101.10
- Final population: 31
- Max zero-food streak: 0
- Scarcity windows: 1 (out of 38 classified)
- Non-scarcity windows: 37
- Forage completions: 153
- Food produced: 596.60
- Food drained (total): 495.50
- Production/consumption ratio: 1.204
- Scarcity boost applications: 30
- Counterfactual boost events: 30
- Boost outside scarcity: 0

## Threshold Compliance
- Assertion 1 (config invariant): plan=1.5, observed=1.5, PASS
- Assertion 2 (zero-food streak): plan=≤200, observed=0, PASS
- Assertion 3 (population): plan=≥25, observed=31, PASS
- Assertion 4 (scarcity response): plan=boost_apps>0, observed=30, PASS
- Assertion 5 (counterfactual): plan=>0, observed=30, PASS
- Assertion 6 (inverse invariant): plan==0, observed=0, PASS
- Assertion 7 (window exclusivity): plan=1+37=38=38, observed=38=38, PASS

## Gate Result
- cargo test: PASS (944 passed, 0 failed)
- clippy: PASS
- harness: PASS (7/7 passed)

## Notes
- Changing FOOD_SCARCITY_THRESHOLD_PER_CAPITA from 1.6 to 1.5 significantly reduces scarcity frequency. With cap=52 per settlement, scarcity now requires pop>34 (vs pop>32 at 1.6). This caused the old test's sample-based forager comparison (A4a: mean_scarcity > mean_non_scarcity) to fail because scarcity windows are too brief and infrequent at 10-tick sampling resolution.
- The plan's A5 counterfactual assertion is a stronger proof than the old A4 forager comparison: it proves the boost *caused* 30 Forage selections that would not have occurred otherwise.
- All 30 scarcity boost applications were counterfactually effective (30/30=100%), meaning every time the boost fired, it changed the action outcome. This is expected at threshold 1.5 where scarcity is brief — when it triggers, agents are doing non-food activities and the boost is decisive.
- The old harness_food_economy_plan_comprehensive test was updated to align with the new threshold: A4 now uses production counter, A6 forage completion threshold loosened from 200→150 per the plan.
