# P3 8.1b — `sim-test/src/**` unit tests ported to the baseline

Scope: only the `#[cfg(test)]` modules of `rust/crates/sim-test/src/{perf,lib}.rs` and
`src/probe/*.rs`. No non-test code changed.

Result: `cargo test -p sim-test --lib --release` → `test result: ok. 88 passed; 0 failed`.
Debug `cargo test -p sim-test --lib --no-run` compiles. `cargo clippy -p sim-test --lib --tests -- -D warnings`:
0 diagnostics in `src/` (remaining errors are all in `tests/`, owned by other agents).

Counts: 0 tests deleted outright, 33 tests ported/rewritten, 2 shared test fixtures ported,
1 test added (pass4, keeps the "neutral without the rule" half of a deleted version-delta assertion).

## perf.rs
- `profiled_and_plain_game_v19_reach_the_same_fingerprint` → `profiled_and_plain_baseline_reach_the_same_fingerprint`: `baseline_scene(7)`.
- `a_short_fed_stress_run_reports_consistent_numbers`: spec label `stress_v19` → `stress_baseline` (build ignores the label; assertions unchanged, all hold).
- `files_and_index_round_trip`: label and expected file names `map_stress_baseline_s3_n24_side64_fed.*`.
- `natural_spec_names_the_scene_founders`: scene `baseline`, stem `natural_baseline_…`; added: its rules equal `WorldRules::baseline(15000)`.
- `rules_flag_picks_the_stress_rules_and_label`: version-delta parts deleted (labels `stress_v19…v25`, `game_v24 != game_v23` etc.). Now: `""`, `--rules baseline`, `--rules game_v26` → `stress_baseline`; `--scene` ignored by stress; `game_v18/v19/v20_inert/v25/v27` rejected; `--rules` rejected for natural; `stress_rules` = `WorldRules::baseline(seed)` and ≠ `WorldRules::default()`; a built stress engine carries `baseline(9)`; stem `stress_stress_baseline_s9_n30…`.
- `cli_parses_each_subcommand`, `nearest_rank_percentiles`, `slowest_ticks_keep_the_twenty_largest`, `display_clock_uses_the_offset`: unchanged.

## lib.rs
- `shared_scene_starts_at_tick_zero` → `baseline_engine_is_the_baseline_scene_at_tick_zero`: `baseline_engine(3)` at tick 0, rules = `baseline(3)`, fingerprint = `scenes::baseline_scene(3)`'s.

## probe/chronicle_out.rs
- `twins_compare_new_entries_and_people`: `game_v23_scene(1)` → `baseline_scene(1)`; founder threshold 20 → `WorldVariety::draw_v18(1).starting_agents`; document scene name `baseline`.

## probe/deaths_observer.rs
- `observation_does_not_change_fingerprint`: `legacy_scene()` → `baseline_scene(0)` (2 frames per tick still hold).

## probe/fingerprint.rs
- `independent_legacy_engines_match` → `independent_baseline_engines_match`: `baseline_scene(0)` twice; added: seed 0 ≠ seed 1.
- `tally_changes_are_observable`: `baseline_scene(0)`.

## probe/long_run.rs
- `owned_beds_children_food_and_idle_hearth_have_exact_values`: `WorldRules::game_v8()` → `WorldRules::default()` plus the baseline's `shelter_planning` (8 per shelter). Explicitly no sustain rule (reporting falls back to 0.85 / radius 16), no seasons, no life stage (the fixture matures a child at tick 0, which the baseline's 5,760-tick life stage would reject). All exact values unchanged.
- `reporting_is_additive_finite_and_partitions_population`: `game_scene(0)` → `baseline_scene(0)`; `adults == 24` → `== WorldVariety::draw_v18(0).starting_agents`.

## probe/metrics.rs
- `idle_actionable_uses_night_threshold_and_preserves_v8` → `idle_actionable_uses_night_threshold_only_with_night_rest`: `WorldRules::baseline(0)` with `night_rest` set explicitly (None / the baseline's 21:00–06:00 rule); expectations (0, 1, 0) unchanged.
- `game_boot_shelters_are_initial_not_new_completions`: `baseline_scene(1)`; 3 shelters / 5 buildings are the worldgen camp (hearth, 3 shelters, stockpile).
- `states_partition_population_and_snapshot_does_not_mutate`: `baseline_scene(0)`.

## probe/mod.rs
- `display_social_counter_does_not_change_legacy_fingerprint` → `display_social_counter_does_not_change_the_fingerprint`: `baseline_scene(0)`.
- `schema_thirteen_is_exactly_additive`: `baseline_scene(0)` (key count unchanged).

## probe/pass4.rs
- `v9_world_reports_neutral_pass4_values` (version delta) → `baseline_world_reports_its_seasons_at_tick_zero`: season 0, day 1 (one-based), `days_per_season` from the rule, tallies/forage/haul 0, supply = 120 × the world's spring / winter `regen_permille`.
- added `without_seasons_the_supply_is_unscaled`: baseline with `seasons = None` → 0 days, supply 120 unscaled (the neutral half of the old test).

## probe/pass5.rs
- `v10_world_reports_neutral_pass5_values` → `baseline_world_reports_zero_pass5_tallies_at_tick_zero`: tick-0 tallies 0, work share 0, idle share 1 (rule-dependent keys such as wood/fire/cold dropped).
- `v11_world_reports_its_deadwood` → `baseline_world_reports_its_deadwood`: `20 × 6` → `(deadwood_spots_placed + region_deadwood_placed) × draw_v18(3).deadwood_capacity`, with a placement sanity bound (≥ half the requested camp spots).

## probe/pass6.rs
- `v11_world_reports_neutral_pass6_values` → `baseline_world_reports_zero_pass6_values_at_tick_zero`: `baseline_scene(3)`.
- `gatherers_elders_and_urgent_have_exact_values`: `game_v11()` → `baseline(0)` (urgent start still overridden to 420; spring burn 2 × 13 h = 26 unchanged).

## probe/pass7.rs
- `v12_world_reports_neutral_pass7_values` → `baseline_world_reports_no_winter_severity_in_spring`: `baseline_scene(3)`.
- `wood_retargets_tally_has_its_exact_value`: `baseline_scene(3)`.

## probe/pass8.rs
- `v13_world_reports_neutral_pass8_values_outside_winter` → `baseline_world_reports_pass8_values_outside_winter`: `baseline_scene(3)`; 20:00 burn = `night_rate` = rule `burn_per_hour[0]` (2); the legacy scene comparison → a bare `SimEngine::new` (no firewood rule).
- `winter_fall_and_rules_block_read_the_rule`: `baseline_scene(0)`; first winter day from the rule (3 × `days_per_season`); severity weights and night-burn permille read from `WorldRules::baseline(0)` (they are drawn per world); burn hours via `burn_hours`; keyed draws 1; legacy block → bare engine; noise-fork check on the baseline engine.

## probe/pass10.rs
- `v15_world_reports_full_health_and_no_variety` → `baseline_world_reports_full_health_and_its_variety`: tick-0 health unchanged; `world_variety` = `variety_json(draw_v18(3))`, `exposure` from the rule, `starting_agents` / `deadwood_spots_requested` from the variety, severity weights from the rule.
- `seed_winter_windows_count_deaths_and_dark_nights`: unchanged.

## probe/pass12.rs
- `work_classes_add_up_to_the_work_share_numerator`: `baseline_scene(3)`, label `baseline`. Under the baseline's routine, adults on `TargetKind::Activity` are in `adult_act_total` but in no class (seed 3, 15:00 day 1: 4 classed of 26), so the partition is now `classes + on_activity == total` (counted in the test); `total > 0` → `total × 2 ≥ founders`. Work-share equality unchanged.

## probe/pass17_death_tests.rs
- fixture `Harness::new` and `observer_is_absent_from_old_worlds_and_checks_tick_and_single_drain`: `WorldRules::game_v21(7)` → `WorldRules::baseline(7)` (all assertions hold unchanged).

## probe/family_trace_tests.rs
- fixture `engine()`: `WorldRules::game_v22(42)` → `WorldRules::baseline(42)` (all assertions hold unchanged).

## probe/pass20.rs
- `only_game_v25_has_the_block` → `the_block_is_present_with_the_relations_rule_only`: block present for `baseline_scene(3)` with tick-0 zeros; absent when `relations = None`.

## probe/pass21.rs
- `only_capped_rules_have_the_keys`: keys present for `baseline_scene(3)`; absent with `dispute_cap_per_pair_day = 0` (the old `game_v25` shape) and with `relations = None`.

## probe/args.rs
- `clock_minutes_validate_sort_deduplicate_and_convert_with_offset`: old-scene acceptance list deleted; now `baseline` / `game_v26` accepted, default `baseline`, `game`, `legacy`, `game_v9`, `game_v19`, `game_v20_inert`, `game_v25`, `game_v27` rejected; usage names `--scene baseline|game_v26`, the unknown-scene error names `baseline`.
- `validates_half_open_range_and_hard_worker_limit`: `--scene game` → `--scene baseline`.

## probe/runner.rs
- `both_v16_scenes_take_both_twin_checks` → `the_baseline_scene_takes_both_twin_checks`: source checks unchanged; every scene `ProbeArgs` accepts (`baseline`, `game_v26`) is twin-checked and builds with its seed; `game_v19` is not.

## Non-test findings (not changed, for the lead)
1. `probe/pass12.rs` `ACTIVITIES` has no class for `TargetKind::Activity` (the routine target maps to `"activity"`, which `position` never finds), so under the baseline the ten `adult_act_*` classes no longer partition `adult_act_total` as the module doc says. Either add an `activity` class (schema change) or fix the doc.
2. `perf.rs` `RunSpec::scene` doc comment still describes the `stress_v19` / `stress_v20` / `stress_v20_inert` labels.
3. `cargo fmt --check` diffs in non-test code: `perf.rs:871` (the `--rules` error `format!`) and `probe/runner.rs:188` (`debug_assert!` in `scene_engine`).
