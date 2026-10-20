# V8-0 Part 3, 8.1b — test port, team 2 (24 files + 2 support modules)

Scope: the 24 `rust/crates/sim-test/tests/harness_revitalize_*` files listed in the brief, plus
`pass17_family_support/mod.rs` and `pass17_routine_support/mod.rs`. Triage source:
`docs/v8/V8_0_INVENTORY.md` sections 3.0, 3.1 and 3.2. No git operations were run.

## Summary

| | count |
|---|---|
| test fns kept and ported (or kept as is) | 117 |
| test fns deleted | 23 |
| new test fns | 1 (`harness_revitalize_pass17_routine_edges::planned_shelter_is_built_by_a_worker_recruited_from_an_activity`) |
| tests run in the 24 files | 118, all pass in `--release` |

Verification (run on 2026-10-20):
- `cargo test -p sim-test --release --test <each of the 24>`: all pass. File times in release:
  pass21_relations 22.8 s, pass19_chronicle 7.0 s, pass16_temperament 6.4 s, pass11_fission 6.0 s,
  pass9_rng 6.0 s, pass13_fission 5.4 s, pass4_aging 3.1 s, pass3_village 2.0 s, loop 1.1 s,
  pass3_rhythm 0.8 s, pass10_variety 0.5 s, all others < 0.2 s.
- Slowest single test in release: `harness_revitalize_pass21_relations::harness_p21_baseline_invariants_and_twin`, 22.5 s (world 32, 36 days, every-tick invariants plus a daily twin). In a debug build it takes 480 s (8 min, measured with other builds running on the machine); this is the closest to the ~10 min limit. The next slowest in debug is pass16 `every_resident_has_temperament_and_rd_rate` at 148 s.
- `cargo clippy -p sim-test --tests -- -D warnings`: nothing reported for these files. `rustfmt --check --edition 2021`: clean for all 26 files.
- No reference to `game_vN`, `game_scene`, `legacy_scene`, `game_default`, `V18_…V26_` constants, `worldmodel`, `state_digest` or `build_world(` is left in these files.

Coverage (section 3.2): every rules-gated system that only these files cover is still exercised,
with its live (baseline) rule on and a measured threshold:
- ShelterPlanningSystem: pass3_village `plans_with_headroom` (6 residents -> 1 site, 5 -> 0), loop `shelters_planned_and_completed` (seed 1: 1 planned, 1 completed, no site lost), pass17_routine_edges new `planned_shelter_is_built_by_a_worker_recruited_from_an_activity` (ticks the system directly).
- MaturationSystem: pass3_rhythm `children_grow_up` (baseline seed 1: first newborn at tick 720 matures at tick 6,481 = born + 5,760 + 1).
- PairingSystem: pass17_family (4 fns tick it with the V23 baseline family rule, plus a new at-threshold control).
- BedAssignmentSystem: pass17_family_beds (2 fns), pass3_rhythm `villagers_sleep_in_beds_at_night` (20/24 and 21/26 asleep in their own bed at 02:00, bar 70 %), pass4_aging bed cleanup.
- AgingSystem: pass4_aging `old_age_death_and_replacement` (baseline seed 1: first old-age death at tick 23,401, a birth follows).
- FissionSystem: pass11_fission (unit fixtures + whole world 8), pass13_fission, pass17_family_fission, pass19_chronicle_events, pass19_chronicle (world 8, 32 days), pass21_relations (world 32, 36 days).
- RelationsSystem: pass21_relations (13 unit fixtures on the baseline relations rule + the 36-day scene).
No system lost coverage.

Rules applied: no fixture sets a non-baseline `Option`/`bool` flag value (no `hazard: None`,
`fission: None`, `use_hardiness: false`, `keyed_draws: false`, `strict_consume: false`,
`HousingCount::NearFormation`, `avoid_gather: false`, ...). The contrast arms that needed one were
dropped. Numeric parameters kept as test-local literals (R2) are named in each file's section.

## Per file


## Group A (fork A): fission, relations, chronicle

Common facts (re-measured on `sim_test::baseline_engine`, release):
- A 30-day scan of baseline worlds 0..48: no world splits before display day 28 (the first winter, days 21–27, is a closed season; checks start on day 11). Earliest splits: worlds 8, 28, 32, 34, 38 split on day 28 and found their daughter on day 29. 30 days of one world take about 6–8 s in release.
- Bare fixtures that used `game_v17` (fission triggered by the crowding gate, `hazard: None`) now use `WorldRules::baseline(seed)` and set the settlement's `fission_threshold = 1`: the live pressure rule then splits at the first eligible 06:00 check (32 members > `size_lo` 24 and 100 food < 4 days of need give h > 0). No `hazard: None` or other non-baseline flag is set anywhere.

### harness_revitalize_pass11_fission.rs
Deleted:
- `harness_fission_inert_before_v17` — version delta (v16 / v17_nofission scenes).
- `harness_fission_only_when_crowded` — tests the crowding gate, a branch the live rules (hazard `Some`) never reach.
- `harness_fission_order_independent` — v17 scene; `pass13::harness_pressure_order_independent` is the live-trigger twin.
Ported (R1, fixture `baseline(seed)` + threshold 1):
- `harness_fission_group_rules`, `harness_fission_group_resampled_by_replicate`, `harness_fission_founds_daughter`, `harness_fission_timeout_and_cancel`, `harness_remnant_merges` — unchanged assertions pass. In the bare world nobody has `Kin`, so every household is a singleton and the live family + temperament selection (neutral NS 500 for all) picks the same set as the old keyed ranking (group 11 of 32, first four ranked adults).
- `harness_shared_bushes_count_once` — dropped the "without the fission rule (game_v16)" arm (it set `fission: None`).
Ported (R3):
- `harness_v17_regions_placed` — `baseline_engine(seed)`, v16 arm dropped. The deadwood ring is now 2–9 (the baseline's `region_fill` adds a 7–9 fill ring); added: every centre has water within 12 (`region_water_max`).
- `harness_fission_founds_daughter_in_a_whole_world` — world 8 (split day 28, daughter day 29), at most 34 days. The old "`exclusivity_moves == 0`" was seed-specific: on world 8 one camp adult standing at the new hearth is joined by the formation scan. Now: the tally equals the daughter's members outside the group, and is ≤ 1.
Slowest release test: `harness_fission_founds_daughter_in_a_whole_world`, file 6.1 s.

### harness_revitalize_pass13_fission.rs
Deleted: `harness_pressure_inert_before_v19` (version delta, v17/v18 scenes, 90+30 days), `harness_v18_hazard_unchanged` (frozen pin of V18_HAZARD).
Kept as is: `harness_threshold_table` (pure).
Ported (R1): `start_rules` = `baseline(seed)` with the 7.1 start hazard/pressure and `min_members` 32 (numeric parameters only, R2) — `harness_pressure_accumulates_and_splits`, `harness_pressure_moves_with_food` pass with unchanged numbers (no later in ≥ 6 of 8 seeds, earlier in ≥ 3).
- `harness_threshold_is_world_draw` — law from `baseline(7)`; the replicate-0 / replicate-3 twin now runs 14 days instead of 120 on `baseline_engine(7)` (checks from day 11; new guard: ≥ 3 checks per replicate); camp θ check on `baseline_engine(3)`.
Ported (R3): `harness_pressure_order_independent` — the whole-world twin runs `baseline_engine(9)` for 14 days instead of `game_v19_scene(9)` for 120 (new guard: ≥ 3 checks logged).
Slowest release test: `harness_threshold_is_world_draw` / `harness_pressure_order_independent`, file 5.3 s.

### harness_revitalize_pass19_chronicle_events.rs
Kept as is: `harness_chronicle_records_a_settlement_system_dissolution` (default rules).
Ported (R1, `baseline(seed)` + threshold 1): `harness_chronicle_records_remnant_merges_and_empty_dissolutions` (unchanged); `harness_chronicle_records_a_cancelled_fission_and_skips_the_dead` — with the live family rule the departures are no longer labelled `Ranked`: measured on seed 11, 4 `Seed` (= `min_adults`) and 7 `Household`; the test asserts no `Ranked`, exactly 4 `Seed`, and Seed + Household = the group.
Slowest release test: under 0.1 s.

### harness_revitalize_pass19_chronicle.rs
Deleted: `harness_chronicle_rebuilds_rosters_without_fission_and_stays_bounded` (needs the no-fission `game_v16` scene), `harness_chronicle_rebuilds_rosters_in_the_legacy_scene` (legacy scene).
Ported (R3): `harness_chronicle_rebuilds_rosters_through_a_fission_game_v23` → `harness_chronicle_rebuilds_rosters_through_a_fission`: world 8 for 32 days (was `game_v23` world 0 for 60), roster replay checked every day. Re-measured at day 32: 20 `Paired`, 14 `Departed`, 23 `FoundingMember`, 39 births, 7 deaths. Asserts: 1 fission and 1 `DaughterFounded`; `Born` = births; `Paired` ≥ 10; `Departed` = the group size (and ≥ `group_min` 6); `FoundingMember` = the founder count; `Died` = the death tallies and ≥ 3.
Slowest release test: 7.0 s.

### harness_revitalize_pass21_relations.rs
Deleted: `harness_p21_game_v26_is_game_v25_with_the_v26_relations` (superseded by `sim-core/tests/baseline_matches_v7_final.rs`), `harness_p21_relations_only_in_v25_and_v26` (version delta).
Changed in the file: `V25`/`V26` constants and `v25()`/`v26()` replaced by `RULE` (the baseline's relations, a const `match` on `WorldRules::baseline(SEED)`) and `rules()` = `baseline(SEED)`.
Ported (R1), v25 contrast arms dropped (they set `avoid_gather`/`exchange_first` false and caps 0):
- `harness_p21_gather_arm_and_post_pass_skip_the_estranged_side`, `harness_p21_post_pass_without_a_target_skips_the_estranged_side` — the contrast is now inside the baseline: estranged (avoid) vs not estranged (relation −100).
- `harness_p21_dispute_cap_stops_the_loss_not_the_count`, `harness_p21_quarrel_cap_skips_the_draw_and_the_pickup_proceeds` — the uncapped v25 arm dropped.
- `harness_p21_exchange_first_takes_the_exchange_over_the_aid`, `harness_p21_exchange_first_when_the_higher_id_sends_food` — v25 aid arms dropped; the exchange assertions unchanged.
- `harness_p21_keep_line_below_the_giver_threshold_leaves_a_surplus` — v25 arm replaced by a baseline arm below the 2,500 ‰ giver line (camp 23 food: no aid).
- `harness_p21_integration_contact_falls_under_avoid_and_the_pair_reconciles` — the v25 "never reconciles" arm dropped; asserts no contact while estranged and reconciliation within 40 days.
- Unchanged apart from the rename: `…v26_relations_frozen_at_grid_g2` (now reads the baseline's rule), `…gather_memo_keys_the_avoid_list_owner`, `…quarrels_while_estranged_are_counted`, `…exchange_first_falls_back_to_aid_when_a_leg_cannot_depart`, `…drift_is_thirty_permille_with_an_integer_stall`.
Ported (R3): `harness_p21_game_v26_invariants_and_twin` → `harness_p21_baseline_invariants_and_twin`: world 32 for 36 days (was `game_v26` world 0 for 60). World 32 founds its daughter on day 29, so RelationsSystem works on a live pair for a week. Every-tick invariants (caps, living pairs, range), the daily twin (fingerprint, relations, chronicle), trip balance and tally/chronicle checks are as before. New thresholds from the re-measure (14 decision records, 2 porter trips delivering 10 each, 27 disputes with 13 uncosted, 3 quarrels with 13 capped, relation 159 from base 300): ≥ 7 decisions, ≥ 1 porter and ≥ 10 delivered, ≥ 10 disputes, ≥ 1 uncosted and ≥ 1 capped quarrel, relation below its base, ≥ 1 fission.
Slowest release test: `harness_p21_baseline_invariants_and_twin`, 22.5 s.

Coverage: FissionSystem (pass11 unit + whole world, pass13, pass19_chronicle_events, pass19_chronicle, pass21) and RelationsSystem (pass21 unit fixtures + the 36-day scene) stay exercised under the baseline rule with measured thresholds.


## Group B: pass17 family/routine, support modules, pass16_temperament

All run with `cargo test -p sim-test --release --test <stem>`, all pass. `cargo clippy -p sim-test --test <each> -- -D warnings` is clean, and `rustfmt --edition 2021` has been applied.

### pass17_family_support/mod.rs
- `fixture()` now takes `family` and `temperament` from `WorldRules::baseline(7)`. Before, it used `V22_FAMILY` and `game_v20(7)`. The baseline family is V23: pairing at familiarity 0.200, and elder partners follow. Every user was re-run against it (below).

### pass17_routine_support/mod.rs
- `fixture()` now takes the routine from `WorldRules::baseline(7)`. Before, it used `game_v21(7)`. The baseline routine is V24, which has `end_stagger`. Every user was re-run against it (below).

### harness_revitalize_pass17_family.rs (6 kept, 0 deleted)
- `pairing_uses_clock_offset_familiarity_ties_and_female_order`: no edit. It passes with the V23 threshold, because the test uses familiarity 0.3, which is at or above 0.200.
- `family_none_is_inert_and_dead_partner_is_cleared_only_at_pair_minute`: no edit. Passes.
- `pairing_excludes_close_kin_children_elders_and_below_threshold`: re-verified, and it failed under V23. The literal 0.299 was below V22's 0.300 but is above V23's 0.200. The below-threshold candidate now uses `rule.familiarity_threshold() - 0.001`, read from the rule. I also added a control: on the next day, the same candidate at exactly the threshold pairs (`>=` in `pairing.rs`). This shows the exclusions are not passing because the matcher does nothing.
- `household_partition_prefers_mother_and_preserves_widow_children_once`: no edit. Passes.
- `family_system_registration_follows_maturation_before_beds_once`: kept as is.
- `founders_receive_keyed_sex_and_empty_kin_only_with_family_enabled`: ported with R3 to `sim_test::baseline_engine(7)`, and the `game_v21` arm is dropped. Assertions: the founder count equals `WorldVariety::draw_v18(7).starting_agents`; every founder has a `Sex` equal to `Sex::from_lineage` and an empty `Kin`; both sexes are present (0 < women < people); there are no FamilyObservations at tick 0.
- PairingSystem coverage: ticked directly by 4 functions, with the live (V23) rule.

### harness_revitalize_pass17_family_beds.rs (2 kept)
- `shelters()` now reads `night_rest` and `shelter_planning` from `baseline(7)`. Before, it used `game_v20(7)`. This is R1 exact: the fields are unchanged. Both functions pass unchanged and tick BedAssignmentSystem with the live rules.

### harness_revitalize_pass17_family_births.rs (3 kept)
- `birth_world()` now takes `shelter_planning` and `life_stage` from `baseline(7)`. Before, it used `game_v20(7)`. This is R1 exact. The 3 functions pass with the V23 family from the support module.

### harness_revitalize_pass17_family_fission.rs (3 kept)
- `departure_world()`: the rules are now `WorldRules::baseline(7)` instead of `game_v17(7)` plus `V22_FAMILY`. That means the live pressure trigger, with the hazard on and `elder_partner_follows=true`. `hazard: None` and the crowding branch are no longer used.
- The numeric gates stay as test values (R2): `min_age_days`, `cooldown_days`, `min_members`, `group_*`, `min_adults`.
- A world this small has hazard h = 0 (members ≤ `size_lo` 24). The fixture therefore presets `fission_pressure = 1_000_000`, so the daily check decays it by 10 and it still meets the drawn threshold. That is the live condition: P ≥ θ, P > 0.
- New assertions in `household_seed_can_exceed_target_k…`: `threshold_at_split > 0`, `pressure_at_split == 999_990`, and pressure ≥ threshold.
- All 3 functions pass with their old member, household, couple and stock-share (food 50) expectations. The constant temperament (500) shift in the rank key leaves the order unchanged.

### harness_revitalize_pass17_routine.rs (7 kept)
- `firewood` now comes from `baseline(7)` instead of `game_v20(7)`. This is R1 exact for the gather window and haul threshold. The routine change arrives through the support module. All 7 pass.

### harness_revitalize_pass17_routine_edges.rs (5 kept, 1 new)
- `unavailable_foraging…`: `forage` and `stock_target` now come from `baseline(7)`. The claim cap is read from the rule (2). Passes.
- `builders_are_recruited…`: `shelter_planning` now comes from `baseline(7)`. Passes.
- `membership_change_aborts_and_day_end_completes_only_arrived_activities`: re-verified under `end_stagger`. Keys 4–6 with P 500 end at 1065 + (key/60) % 30 + 0 = 17:45, which is ≤ 18:00. I added asserts that the rule has `end_stagger` and that each `end_minute(key) <= 1080`. Passes with the old outcomes.
- `no_place_rest…` and `missing_stockpile…`: unchanged. Pass.
- New `planned_shelter_is_built_by_a_worker_recruited_from_an_activity`. It ticks ShelterPlanningSystem directly with the baseline rule, twice, one plan interval apart. Then exactly 1 site exists (1 member + headroom 2 ≥ capacity 0 plans one, and a pending site blocks a second). The site is owned by settlement 1, on ring `ring_min` (2), and `skipped_no_tile` is 0. The decision cascade then sends the active worker to that tile. I added it because the existing `builders_are_recruited…` only exercises the `shelter_planning` rule inside AgentDecisionSystem and never ticks ShelterPlanningSystem.

### harness_revitalize_pass17_routine_invitations.rs (4 kept)
- No edits. All 4 re-verified under the V24 routine from the support module and pass.

### harness_revitalize_pass16_temperament.rs (1 kept, 2 deleted)
- Deleted `harness_v20_inert_equals_v19_at_40320_ticks`: an identity check between the placebo and v19 (a version comparison; both scenes are gone).
- Deleted `harness_v20_diverges_and_adults_wander_by_ns`: a v19 vs v20 contrast.
- Ported `harness_v20_every_resident_has_temperament_and_rd_rate` with R3 to `baseline_engine(3)`, and the inert arm is dropped. Assertions:
  - The rule has `effects=true`.
  - Every founder has `Temperament::founder` and growth rate `rule.social_rate(0.04, t)`.
  - The founder rates are not all equal.
  - After 30 days, every agent has Temperament and Social, with the RD rate.
  - Births ≥ 4. Re-measured on seed 3: 26 founders, 8 births, 27 alive. The bar is half the measured births.

### Counts and timing
- Functions kept and ported: 31. Deleted: 2. New: 1.
- Slowest test: `harness_v20_every_resident_has_temperament_and_rd_rate`, 6.4 s in release and 148 s in a debug build. Every other test finishes in under 0.01 s.


## Group C: village, rhythm, aging, forage, elders, loop

All 7 files pass `cargo test -p sim-test --release --test <stem>`. Explicit `cargo clippy -p sim-test --test <each of the 7> -- -D warnings` is clean. All 7 were run through rustfmt.
Slowest release test: `harness_revitalize_pass4_aging::harness_aging_old_age_death_and_replacement`, 3.05 s. It now stops once a birth follows the first old-age death.

### harness_revitalize_pass3_food.rs (6 kept, 0 deleted)
- New helper `food_rules()` (R1). It copies these from `baseline(7)` into `default()`: birth (Provisioned, utilization 0.78, capacity_weighted), shelter_planning, settlement_anchor, relief_radius, strict_consume and idle_tether_radius.
- `harness_food_supply_and_need_formula`: still checks supply 120 and need 57.6. `sustainable_adults` is now called with the baseline utilization 0.78, giving 39 (v9 had 0.85 and 42).
- `harness_food_no_free_meals`: the v8 arm (strict_consume=false) is dropped. A second eater on an emptied tile stays at hunger 80, and meals_without_food is 0.
- `harness_food_birth_blocked_at_supply`: 42 members became 39, the first refused size under the baseline gate (40 × 2.4 = 96 > 93.6).
- `harness_food_birth_allowed_when_sustainable`: still 24 members → 1 birth. Added the boundary: 38 members → 1 birth (39 × 2.4 = 93.6 ≤ 93.6).
- `harness_food_birth_blocked_without_reserve`: only the rules changed (stock 38 < 10 + 28.8).
- `harness_food_relief_reaches_tether`: the v8 arm (relief None) is dropped. It asserts baseline relief_radius == idle_tether_radius (8), and a member 7 tiles out eats from the store (80 → 50).

### harness_revitalize_pass3_village.rs (5 kept, 1 deleted)
- Deleted `harness_village_v8_unchanged`: a v8 fingerprint pin.
- New helper `village_rules()` (R1, exact). It copies baseline shelter_planning and the hearth anchor into `default()`.
- `harness_village_one_settlement_per_hearth` (R3): `baseline_engine(seed)` for seeds 0..8 × 2 days. One village, owning every building, with at least 5 owned buildings. This is before any fission can happen (min_age_days 10).
- `harness_village_owned_capacity_counts_each_shelter_once`: uses the baseline rule and asserts Owned. Capacities: 32 for the owner, 0 for the other village. The NearFormation arm is dropped, since it re-created an old enum value.
- `harness_village_plans_with_headroom`: the v8 arm is dropped. It now tests both sides of headroom 2 with one owned shelter. 6 residents → 1 planned site; 5 residents → 0 sites. This is the ShelterPlanningSystem test.
- `harness_village_completed_site_is_owned` and `harness_village_dissolution_releases_only_its_owned_buildings`: only the rules changed (`village_rules()`).

### harness_revitalize_pass3_rhythm.rs (7 kept, 1 deleted)
- Deleted `harness_rhythm_sleep_seek_does_not_dehydrate_seed33`: a v9 regression on one seed.
- `harness_rhythm_minute_window_wraps_midnight`: kept as is.
- `harness_rhythm_wake_for_critical_thirst`, `harness_rhythm_bed_seek_ignores_intermediate_shelters`, `harness_rhythm_reassigned_bed_replaces_existing_sleep_seek` (R1): `game_v9()` became the full `WorldRules::baseline(1)`. They pass unchanged, fatigue 9.88 included.
- `harness_rhythm_villagers_sleep_in_beds_at_night` (R3, baseline seed 1): residents asleep on their assigned `bed_of` bed at 02:00 measured 20/24 (day 2) and 21/26 (day 3). The bar stays at ≥ 70 %. This is the BedAssignmentSystem test.
- `harness_rhythm_evening_near_hearth` (R3, seed 1): idle adults within the evening tether at 19:30 measured 21/21 and 16/18. The bar was raised from ½ to ¾.
- `harness_rhythm_children_grow_up` (R3, seed 1; the MaturationSystem test): first newborn at tick 720, matured at tick 6,481 (born + 5,760 + 1). Need rates and the no-work checks are unchanged.

### harness_revitalize_pass4_aging.rs (4 kept, 1 deleted)
- Deleted `harness_aging_v9_unchanged`: a v9 fingerprint pin.
- `rule()` now reads the baseline aging rule (R1, exact).
- `harness_aging_lifespans_bounded_and_deterministic`: keeps the `newborn_life` checks. It adds the live keyed path, `keyed_newborn_life`: same band, mean 57–59 days, and the lifespan does not change with the birth tick.
- `harness_aging_founders` (R3, seeds 0..8): the count was fixed at 24; now it equals `WorldGenConfig::baseline(seed).starting_agents`, within `RANGES_V18` 18..=30. Founders' Life now equals `keyed_founder_life(seed, lineage.key, 0, rule)`. The unkeyed `founder_life` no longer matches, because baseline uses keyed draws. Ages are still 6–40 days, at least 3 days of life remain, and seed 0 has no old-age death in 3 days.
- `harness_aging_old_age_death_and_replacement` (R3, baseline seed 1; the AgingSystem-tick test):
  - Measured: first old-age death at tick 23,401 (day 16), and a birth on the next tick.
  - Over the full 60 days: 28 old-age deaths, 37 births, 0 starvation and 0 dehydration deaths.
  - The test now breaks once a birth follows the first old-age death. Bed and roster cleanup of the dead is still asserted.
- `harness_aging_elders_stay_home` (R3, seed 5, routine on): 6,509 idle elder observations in 3 days (bar > 1000). No building or gathering, and no drift past the tether.

### harness_revitalize_pass4_forage.rs (6 kept, 2 deleted)
- Deleted `harness_forage_seed8_no_longer_stagnates` (a v9/v10 regression on one seed) and `harness_forage_v9_unchanged` (a pin). The `v9_with_forage` helper is removed.
- `forage_fixture` now starts from `WorldRules::baseline(7)` (R1). `max_foragers_per_bush` = per_bush is test-local (R2, numeric; baseline is 2, v10 was 1).
- `harness_forage_claims_cap`, `harness_forage_retargets_when_bush_empties`, `harness_forage_gatherer_whose_tile_vanished_idles_instead_of_freezing`: pass unchanged on the baseline fixture.
- `harness_forage_stops_at_target` (R3, seed 2): passes unchanged.
- `harness_haul_home_deposits` (R3, seed 4) and `harness_haul_home_after_the_forage_window` (R3, seed 4):
  - Other villagers deposit in the same window (measured 5 vs v10's 1, and 36 vs 35), so the tally assert `== deposits + 1` became `> deposits`.
  - The hauler's own deposit is now asserted directly: its load empties inside the deposit radius, and the stock gains the load.

### harness_revitalize_pass6_elders.rs (2 kept, 1 deleted)
- Deleted `harness_elder_tether_v11_unchanged`: it sets elder_evening_tether=false.
- `harness_elder_evening_tether_v12` was renamed `harness_elder_evening_tether`. It and `harness_elder_tether_returns_after_the_evening` use `WorldRules::baseline(7)` (R1, exact). The thresholds come from the rule.

### harness_revitalize_loop.rs (6 kept, 1 deleted)
- Deleted `harness_social_idle_agents_stay_near_home`: a v8 idle tether on a v8 world. The `home_distance` helper is removed.
- `harness_population_game_world_survives_3_days` (R3, baseline seeds 1..=3):
  - Measured residents after 3 days: 23→28, 21→26, 26→27. The floor was a fixed 20; it is now "≥ founders".
  - Idle need breaches measured 322/109,996, 334/101,520 and 293/115,920 (about 0.3 %). The bar was lowered from 10 % to 2 %.
  - Dehydration deaths are still 0.
- `harness_building_shelters_planned_and_completed` (R3, seed 1): measured 1 site planned and 1 completed in 3 days. It now also asserts that no planned site is lost: each one became a Shelter or is still a site. This is the ShelterPlanningSystem scene test.
- `harness_population_births_gated_by_food_and_housing` (R1):
  - Uses baseline shelter_planning (owned housing), the hearth anchor (a Hearth was added to the fixture) and the baseline `sustain` gate (4 bushes were added).
  - The old `sustain: None` is gone. Cooldown 10, food cost 6 and 4 residents per shelter are R2 numeric values.
  - Result: 1 birth, then housing full (4/4).
- `harness_building_assigned_builder_walks_and_clears_missing_site` (R1): `game_v8()` became the full `baseline(7)`, keeping the numeric overrides move_interval 1 and idle_pause 1.0. It passes unchanged.
- `harness_population_rules_default_is_legacy` and `harness_population_movement_only_steps_on_configured_tick_multiples`: kept as is.

### Coverage
All four systems this group must keep are still covered, with their rule active:
- ShelterPlanningSystem: pass3_village `plans_with_headroom`, loop `shelters_planned_and_completed`.
- MaturationSystem: pass3_rhythm `children_grow_up`.
- BedAssignmentSystem: pass3_rhythm `sleep_in_beds_at_night`, pass4_aging bed cleanup.
- AgingSystem: pass4_aging `old_age_death_and_replacement`.

No new test files.


### harness_revitalize_worldgen.rs (3 kept, 0 deleted)

- Ported (R3) `harness_territory_worldgen_deterministic`, `harness_territory_worldgen_invariants`: `build_world(game_default)` -> `build_world_with(WorldGenConfig::baseline(seed), WorldRules::baseline(seed))`. The hard-coded seed list (1..=8 plus 247, 338, 345, 383, 426, 728, 759, 773) passes unchanged. The bush invariant is split: camp bushes (Chebyshev distance <= 16 from the hearth) number exactly `berry_bushes` and keep the old 6..=16 / Forest / Plain-neighbour checks; wild-region bushes (Pass 11) must lie beyond the camp reach (max(forage 16, wood 18) = 18, read from the rules) and number at most `wild_regions * region_bushes`. Building counts (1 hearth, 3 shelters, 1 stockpile) and the full camp settlement at tick 2 hold on all 16 seeds.
- Ported (R3) `harness_building_snapshot_matches_registry`: `sim_test::baseline_engine(1)`; still 5 rows.
- Release time: 0.03 s for the file.

### harness_revitalize_pass10_variety.rs (4 kept, 1 deleted)

- Deleted `harness_variety_neutral_equals_v16_neutral`: version delta (`game_v16_with(NEUTRAL)` vs `game_v16_neutral`, both gone).
- Ported (R1) `harness_variety_ranges`: `draw` -> `draw_v18`, `RANGES` -> `RANGES_V18`; the range/coverage checks and the slot-formula check now also cover the three region slots 8-10 (wild_regions, region_bushes, region_deadwood); the "neutral climate is v15's severity" line was dropped (`game_v15` is gone).
- Ported (R3) `harness_variety_independent_of_replicate`: `baseline_engine(seed)`; the recorded variety is the full `draw_v18(seed)` (no `without_regions`); rebuilt from `WorldGenConfig::baseline` / `WorldRules::baseline`; `fingerprint` equality kept.
- Ported (R1/R3) `harness_variety_deterministic`: `baseline_engine`; bush count = `berry_bushes + region_bushes_placed`, wood spots = camp + region deadwood placed; `rules == WorldRules::baseline(seed)`; regen = base [750, 1250, 1000, 500] scaled by the draw; 1,440-tick fingerprint twin.
- Ported (R3) `harness_variety_worldgen_accepts_every_drawn_world` (seeds 0..400): camp bushes, camp deadwood and founders exact as before; new re-measured region invariants: every placed region gets all its bushes (8,009 of 8,009) and at most its drawn deadwood; 7 seeds (75, 94, 107, 241, 282, 324, 393) place one region fewer than drawn (no candidate centre), none two fewer, none zero - bound <= 10 short seeds; region deadwood placed 11,697 of 11,779 (993 permille, 19 seeds short) - bound >= 990 permille.
- Release time: 0.51 s for the file.

### harness_revitalize_pass10_exposure.rs (3 kept, 3 deleted)

- Deleted `harness_exposure_inert_before_v16` (v15 pins), `harness_exposure_adult_survives_one_night` (v1 adult damage; pass11 `adult_can_die` covers v2), `harness_exposure_only_from_cold` (`game_v16_neutral_scene` seed 6), and the now-unused `PreFire` observer / `exposure_deaths_at` helpers.
- Fixture (R1): `WorldRules::baseline(7)` (exposure v2 `[7.5, 5, 5]`, `use_hardiness` true, `hunger_weight` 0.5) with `world_seed` 7; every resident gets `Hardiness { permille: 1000 }` and hunger 0, so a cold hour costs exactly the rule's damage.
- Ported `harness_exposure_kills_a_cold_elder`: the old start-values arm (`use_hardiness: false`, `hunger_weight: 0`) was dropped (R2 on a flag); now: an elder at 3 x 5.0 = 15 hp dies in hour 3, at 15.5 hp in hour 4; new hardiness arms: 15.5 hp at 1300 permille dies in hour 3, at 700 permille in hour 5.
- Ported `harness_exposure_blocks_healing`: block/heal checks unchanged (heal_block_hours 2, +0.05 hp per tick after the block); the v15 "no rule heals at once" arm (`game_v15`, exposure `None`) is replaced by a resident whose last cold hour is a day back healing on the first run.
- Ported `harness_exposure_order_independent`: unchanged assertions (>= 4 deaths, two in one hour, AgentId order within an hour) hold under the baseline damage; it already used `fingerprint`.
- Release time: 0.00 s for the file.

### harness_revitalize_pass9_rng.rs (9 kept, 1 deleted)

- Deleted `harness_v14_draws_unchanged` (v14 fingerprint pins).
- Kept as is: `harness_lineage_keys_stable_under_birth_shift`, `harness_lineage_keys_do_not_collide`, `harness_keyed_draw_is_pure`, `harness_keyed_step_balanced`.
- Ported (R3) `harness_lineage_founder_keys_follow_seed_and_spawn_index`: `baseline_engine`; founder count = `draw_v18(seed).starting_agents` instead of 24.
- Ported (R3) `harness_v15_newborns_carry_their_first_parents_key`, renamed `harness_newborns_carry_their_first_parents_key`: `baseline_engine(1)`, 28 days kept (40,320 ticks); founders from the draw (23); re-measured 15 births, 32 alive; threshold `births >= 8` (was `> 0`).
- Ported `harness_v15_movement_independent_of_history`, renamed `harness_movement_independent_of_history`: the sequential-stream arm (`keyed_draws = false`) was dropped (it would pin a non-baseline flag value); replaced by a non-triviality bound on the keyed run: 147 of 300 ticks move (expected about 0.6 x 8/9), bound 120..=200; the scene arm runs `baseline_engine(2)` (MovementRng never advanced under keyed draws).
- Ported (R3) `harness_noise_fork_replicate0_is_identity`, `harness_noise_fork_prefix_identity`: `baseline_engine(12_000 / 12_001)`, 4 days, `fingerprint` rows.
- Slowest release test: `harness_newborns_carry_their_first_parents_key`, 5.9 s.

