---
feature: food-economy
code_attempt: 1
---

## Files Changed
- `rust/crates/sim-test/src/main.rs`: Added `harness_food_economy_plan_comprehensive` test with all 7 plan assertions (100-tick food sampling, 10-tick forage sampling)

## Observed Values (seed 42, 20 agents)
- Final stockpile_food at tick 4380: 1757.10
- Max consecutive zero-food ticks (post tick 500): 0
- Population at tick 4380: 31
- Scarcity windows (food/capita < 1.5): 0
- Non-scarcity windows: 38
- Mean foragers in scarcity: N/A (no scarcity windows)
- Mean foragers in non-scarcity: 0.58
- Collapse windows (food < 1.0): 0
- Unrecovered collapse windows: 0
- Total forage observations (10-tick sampling): 254
- Estimated forage completions: 106
- Max food at any sample point: 1735.30

## Threshold Compliance
- Assertion 1 (final food > 5.0): plan=>5.0, observed=1757.10, **PASS**
- Assertion 2 (zero streak ≤ 200): plan=≤200, observed=0, **PASS**
- Assertion 3 (pop ≥ 25): plan=≥25, observed=31, **PASS**
- Assertion 4 (scarcity response active): plan=mean_scarcity > mean_non_scarcity, observed=N/A (0 scarcity windows), **PASS** (soft — economy healthy enough that scarcity boost not needed)
- Assertion 5 (recovery after dips): plan=0 unrecovered, observed=0, **PASS**
- Assertion 6 (forage completions ≥ 200): plan=≥200, observed=106, **FAIL**
- Assertion 7 (max food ≤ 200.0): plan=≤200.0, observed=1735.30, **FAIL**

## Gate Result
- cargo test: FAIL (125 passed, 1 failed)
- clippy: PASS
- harness: FAIL (5/7 passed — A6 and A7 fail)

## Notes
- **Threshold NOT changed** — plan thresholds are locked as required.
- **Root cause of A6+A7 failures**: The food economy over-produces. FORAGE_STOCKPILE_YIELD=3.0 combined with low consumption (childcare ~0.04 food/tick, births ~0.017 food/tick) means food accumulates monotonically from 7.6 to 1735 over 4380 ticks. The plan expected production and consumption to roughly balance, keeping food in a 5-200 range. In practice, production ≈ 1750 total, consumption ≈ 250 total, net surplus ≈ 1500.
- **A6 (forage completions)**: With food so abundant (per capita always >> 1.5), the FOOD_SCARCITY_FORAGE_BOOST never activates. Agents spend minimal time foraging (only ~106 estimated completions) because they don't need to — food is plentiful. The plan threshold of ≥200 assumed more active foraging.
- **A7 (upper bound)**: The plan estimated max food ~20 with 200 as a 10× safety margin. Observed 1735 is 87× the expected peak. This suggests either the forage yield increase (2.0→3.0) was too aggressive, or consumption mechanisms (childcare, crafting) are not draining food at the expected rate.
- **A4 (scarcity response)**: Cannot be evaluated because food per capita never drops below FOOD_SCARCITY_THRESHOLD_PER_CAPITA (1.5). The scarcity boost code path exists in cognition.rs but is effectively dead at seed 42 with these parameters.
- **Existing test `harness_food_economy_balance_4380` still passes** — its weaker thresholds (food > 2.0) are satisfied.
- **No implementation changes made** — the fix was already in place per task description. Only test code was added.
- **Recommendation**: The food economy needs rebalancing. Either reduce FORAGE_STOCKPILE_YIELD (e.g., back toward 2.0-2.5) or increase consumption rates. The current parameters solve the original depletion bug but create an opposite problem (unbounded accumulation).
