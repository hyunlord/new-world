# P3 8.1b — test port, batch t1 (13 files)

Files in `rust/crates/sim-test/tests/`. Triage followed `docs/v8/V8_0_INVENTORY.md` 3.0/3.1.
Unit fixtures moved to `WorldRules::baseline(SEED)` (R1); scene tests use `sim_test::baseline_engine(seed)` (R3).
No old `Option`/`bool` flag value was put back. The fixtures change only numbers: one-hot severity weights, night weights and step, `max_gatherers_per_spot`, cap `Some(0)`, aid thresholds, `wild_regions: 0`, `allowed_seasons = [false; 4]`.

Totals: **82 tests kept** (all pass in `--release`), **22 deleted**. `cargo clippy -p sim-test --tests -- -D warnings` is clean, and `rustfmt --check` passes on all 13 files.
The slowest test is `harness_hardiness_keyed_and_ranged` (pass11_exposure): about 4.9 s in release and 109 s in a debug build. Every other file runs in under 0.2 s in release.

## Coverage kept (system run with its rule active)
- FireSystem: pass5_firewood (`fire_burns_and_goes_out`, `fire_cold_raises_hunger`), pass6_winter (last and first winter nights), pass7_winter (burn by severity, last night), pass8_winter (`night_rate_floor_is_one`, `last_night_keeps_its_evening_draw`), pass11_exposure (formula, adult can die). pass6_urgent covers the urgent decision path; it does not run FireSystem.
- RelationsSystem: pass20_relations (20 tests on baseline relations = V26).
- StockpileDepositSystem: pass5_firewood `firewood_trip_and_deposit`, pass20 `trip_delivers_returns_and_is_not_redeposited` (walk() runs deposit).
- StockpileConsumeSystem: pass5_meals (5 tests).
- ResourceRegenSystem seasons/winter: pass4_seasons (`season_regen_schedule`), pass6_winter (cap, spring restore), pass7_winter (`winter_fall_by_severity`). pass12_world has no regen test anymore: `harness_area_fall_floor` was deleted per triage.

## Per file

### harness_revitalize_pass4_seasons.rs (2 kept / 2 deleted)
- Deleted: `harness_season_survives_first_winter` (v10 tuning outcome), `harness_season_v9_unchanged` (frozen pin).
- `harness_season_regen_schedule`: the fixture copies `seasons` + `clock_offset_ticks` from `baseline(seed)` into default, for seeds 0..8. The baseline regrowth is `[750,1250,1000,500]` scaled by the food-regen draw, for example seed 1 = [656,1093,875,437]. The old exact per-day values [9,15,12,6] are replaced by checks on the rule: each day gets floor or ceil of 12p/1000, each full season gets floor(84p/1000) or one more, and summer > autumn > winter.
- `harness_season_target_includes_winter`: `baseline(1)`. New literals: base 2.5 × 100.8 = 252, winter supply 12 × 12 × 0.437 = 62.928, summer/autumn add 7 × 37.872 = 265.104.

### harness_revitalize_pass5_firewood.rs (8 kept / 2 deleted)
- Deleted: `harness_firewood_ffi_rows` (v11 seed-1 literal + v10 compare), `harness_firewood_v10_unchanged` (pin).
- `harness_deadwood_placement` (R3, v10 arm dropped): camp spots are `wood_spot_area == 0`. Their count equals `variety.deadwood_spots` (14..23 on seeds 0..4) and their capacity equals `variety.deadwood_capacity`. The 5..18 ring, terrain, overlaps and spacing are checked over all spots, including the regions'.
- `harness_wood_regen_schedule`: v10 arm dropped; the 500‰ schedule is unchanged.
- `harness_firewood_stops_at_target` (R3, `baseline_engine(2)`): the "outside the window" probe moved from 09:00 to 08:59, because the baseline's urgent start is 09:00.
- `harness_firewood_claims_cap`: the store holds tonight's burn (26) to stay out of urgent mode, which would double the cap, while staying below the 39 target.
- `trip_and_deposit`, `retargets_empty_spot`, `fire_burns_and_goes_out`, `fire_cold_raises_hunger`: fixture changed to baseline only; the numbers did not change.

### harness_revitalize_pass5_meals.rs (5 kept / 1 deleted)
- Deleted: `harness_meal_v10_unchanged` (pin). All 5 meal fns use `baseline(1)` unchanged (MealRule is exact).

### harness_revitalize_pass5_preempt.rs (2 kept / 1 deleted)
- Deleted: `harness_preempt_v10_unchanged`.
- Both fns use `baseline(1)`. The bush moved from (38,22) to (36,22): baseline `local_food_availability` limits foraging to the catchment (Chebyshev ≤ 16), and v11's bush was 18 away.

### harness_revitalize_survival_fix.rs (10 kept / 1 deleted)
- Deleted: `harness_economy_clock_offset_is_display_only` (false under the live rules, per triage).
- The chase, fallthrough and preemption fns (9) moved from `game_v8()` to `baseline(1)` with no changes. `harness_economy_clock_initial_values` now uses `baseline_engine(1)` (08:00).

### harness_revitalize_pass6_winter.rs (7 kept / 3 deleted)
- Deleted: `harness_firewood_v11_unchanged` (pin), and `harness_winter_spread_regrowth` and `harness_winter_regrowth_branches_without_one_field` (severity-`None` branch, per triage).
- SEED 3 (normal first winter, day-27 night draw +2, day-21 fall 8).
- `formula_table`: explicit literals 39 / 19.5 / 195 / 156 / 39; the no-seasons arm was dropped (it would set an Option to None).
- `by_season`: the winter expected value uses the severity's rate.
- `autumn_only_prep`: v11 arm dropped; an autumn ≥ summer + 150 bar was added.
- `winter_cap_buries_deadwood`: pulses at the first winter pulse (00:00 day 21) because of `windfall_over_snow`. With cap 0, the units lost over one day equal the day's fall (8).
- `spring_restores_deadwood`: the winter starts at 00:00.
- `last_winter_night_is_winter`: the night burns 5 an hour (normal 3 + 2, not spring's 2); v11 arm dropped. `first_winter_night_is_autumn`: unchanged.

### harness_revitalize_pass6_urgent.rs (4 kept / 1 deleted)
- Deleted: `harness_urgent_v11_unchanged` (pin).
- SEED 3; "tonight" now uses `tonight_burn` (severity) = 39. All 4 fns otherwise unchanged.

### harness_revitalize_pass7_winter.rs (7 kept / 1 deleted)
- Deleted: `harness_firewood_v12_unchanged`.
- SEED 1 (climate weights 350/400/250).
- `severity_is_a_pure_draw`: the v12 arm was replaced by "severity only in winter hours" (−1 up to 07:00 day 21, ≥0 until 07:00 day 28).
- `burn_by_severity` and `last_night_keeps_its_winter_severity` add the night's draw (Pass 8; seed 1 day 22 = 0).
- `winter_fall_by_severity`: 12 pulses of day 22 from 00:00; the fall is the severity's value + the day's draw (floor 0), with a ≥ 2 bar.
- `severity_table`: R2 numeric weights [300,400,300] on the baseline rule.

### harness_revitalize_pass8_winter.rs (6 kept / 1 deleted)
- Deleted: `harness_firewood_v13_unchanged`. The windfall tests named in the triage were already absent.
- SEED 1. `night_burn_is_a_pure_draw` and `fall_spread_is_a_pure_draw` drop the v13 arms and assert the baseline weights [250,500,250], step 2 and spread 2. `draw_table` asserts that the baseline's parameters equal the table's.
- `villagers_do_not_see_tonights_draw`: rewritten without the "blind" copy, which would set an Option to None. Target, tonight's burn and urgency are compared with values built from the severity rate alone. The fire burns off-rate in 117 / 65 / 91 / 91 hours on seeds 0..4; the bar is 26.

### harness_revitalize_pass11_exposure.rs (3 kept / 1 deleted)
- Deleted: `harness_v16_exposure_unchanged`. The rain test was already absent.
- `exposure_v2_formula`: baseline [7.5,5,5] instead of the START constant [8,6,8]. Spot values: 3.5, 6.25, 9.685, child 9.075, elder 4.5.
- `exposure_adult_can_die`: the frail adult loses 77.48 on night 1 and dies on night 2. The hardy adult ends at 44 hp (100 − 2 × 28; measured, no healing seen). The "tuned v17" arm was dropped as redundant.
- `hardiness_keyed_and_ranged` (R3, `baseline_engine`, seeds 3/41/900017 × 6 days, plain + fork; v16 arm dropped). Measured births are 1 / 11 / 5. The bar is ≥ 1 per seed and run and ≥ 10 in total (measured 17).

### harness_revitalize_pass12_fission.rs (3 kept / 3 deleted)
- Deleted: `harness_hazard_inert_in_v17`, `harness_hazard_keyed_with_replicate` and `harness_hazard_order_independent` (v18 scenes, hazard-draw-only).
- The baseline hazard is V19 (24..44, crowd weight 0, scarcity 1000 / 4 days, ceiling 300) with pressure.
- `hazard_formula`: new hand-derived terms (300,750,750,67), (300,1000,750,67), (300,750,1000,90), (0,1000,1000,0). The constructed checks assert draw = 0, P = h on day 1, θ = `fission_threshold(law, seed, 1, 0)` > 0 and split = P ≥ θ. The smallest-village case is now 28 members (`min_members`).
- `rises_with_shortfall`: baseline hazard. `allowed_seasons`: baseline [t,t,t,f]; scene part dropped.

### harness_revitalize_pass12_world.rs (5 kept / 1 deleted)
- Deleted: `harness_area_fall_floor` (v18_nofission scene, per triage). The rain test was already absent.
- `region_fill_delivers` (renamed, v17 arm dropped; 128 baseline worlds): bushes 2519/2519 with 940 from the fill, deadwood 3692/3716 with 129 from the fill. The bars are ≥ 0.95 of requested, fill ≥ 470 bushes and fill ≥ 64 deadwood.
- `region_fill_never_touches_terrain_or_water`: compares against the same world with `wild_regions: 0` (a count) instead of `region_fill: false` (a flag). Terrain, water and buildings must be equal, and the bush and wood counts must differ by exactly `region_*_placed`.
- `region_water_rule`: baseline worlds 0..64.
- `local_food_availability` (renamed, v17 arms dropped, `baseline(0)`). `local_gather_stays_in_the_catchment`: kept (pure fn).

### harness_revitalize_pass20_relations.rs (20 kept / 4 deleted)
- Deleted: `harness_relations_rule_only_in_game_v25`, `harness_relations_rule_values_pinned`, `harness_relations_game_v25_is_deterministic` (per triage), and `harness_relations_tuned_aid_takes_the_exchange_food_condition` (asserts V25 aid-before-exchange; the V26 side is covered by `harness_p21_exchange_first_takes_the_exchange_over_the_aid`).
- Fixture `baseline(20)`; `RULE` is a const read from `WorldRules::baseline(SEED).society.relations` (V26). Re-derived values:
  - keep line: 20 (was 29). Cases are 23 → 0, 28 → 8, 31 → 11 (10 + 1).
  - drift 30‰: −315 → −302 estranged, −160 → −151 still estranged, −152 → −143 reconciled.
  - quarrel chronicle value: 300 − 5 − 40 = 255.
  - integration: −290 → −335 after the quarrel, then −316 estranged after the drift.
  - exchange food leg: 32 − 20 = 12, camp left with 20. The brief aid thresholds (numeric) stay, so "none" is visible under exchange-first's fallback.
- The other 14 fns pass unchanged on V26.

## Coverage that could not be kept
None. FireSystem, RelationsSystem, StockpileDeposit/Consume and ResourceRegen (seasons and winter) are each still exercised with the rule active. The regional area-floor regrowth no longer has a test in these files: `harness_area_fall_floor` was deleted per triage.
