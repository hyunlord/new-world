# P3 8.3 — dead-flag candidates in `WorldRules`

Read-only analysis of `rust/crates/**` at the working tree of 2026-10-20 (no build, no edits).
Paths below are relative to `rust/crates/`. "Test" = `#[cfg(test)]` modules (including the
`#[path]` test files `sim-test/src/probe/{routine,routine_batch,routine_death,family_trace}_tests.rs`),
`*/tests/**`, `*/benches/**`. Production rule sources checked: `WorldRules::baseline` is the only
non-test constructor (`sim-bridge/src/ffi/world_node.rs:195,204`, `sim-test/src/scenes.rs:16`,
`sim-test/src/perf.rs:117` `stress_rules` = baseline); `WorldRules::default()` is what
`SimResources::new` installs (`sim-engine/src/lib.rs:652`), so every test on a bare
`SimEngine::new` / `SimResources::new` runs on default(). No rules are deserialized anywhere
(no `serde_json::from_*`/`ron::from_*` of rule types), so `#[serde(default)]` never injects values.

**Applies to every removal below:** `sim-core/tests/baseline_matches_v7_final.rs` compares
`format!("{:#?}", WorldRules::baseline(seed))` with `sim-core/tests/data/v7_final_rules_seed{0,1,7,42}.txt`;
each removed field (or an `Option` made non-optional) changes those 4 dumps, which the test's
header says must be updated in the same commit with a decisions entry (ADR 0011).

## 1. Candidates

### 1a. Strict candidates — (a), (b), (c) hold; no test anywhere sets the other value

| field path | baseline | default | branch sites removed | dead branch | test sites (none set the other value) | printed in probe? | notes |
|---|---|---|---|---|---|---|---|
| `society.birth::Provisioned.sustain.capacity_weighted` | `true` | n/a (birth = `Legacy`) | `sim-systems/src/runtime/settlement/food_economy.rs:189-200` (`weighted` match) | unweighted source count *under a sustain rule*; the else arm stays reachable via `Legacy`/`sustain: None` → rewrite the pattern to `sustain: Some(_)` | none (only readers: none in tests) | no | only reader is that `matches!` |
| `society.stock_target.winter_prep` | `true` | n/a (`stock_target` None) | `sim-systems/src/runtime/settlement/food_economy.rs:107-111` (`if let (true, Some(seasons), Some(..))`) | no winter-shortfall term in summer/autumn; keep the seasons check | none (no `StockTargetRule` literal or assignment outside baseline) | no | not the same field as `FirewoodRule.winter_prep` |
| `society.firewood.winter_prep` | `true` | n/a | `sim-systems/src/runtime/settlement/firewood.rs:312-316` (`wood_target`) | no winter prep added; keep the seasons check | literals set `true`: `sim-systems/src/runtime/settlement/firewood.rs:374`, `sim-bridge/src/ffi/world_node.rs:4309`; reader `sim-test/tests/harness_revitalize_pass6_winter.rs:122` (`expected_target`) | no | both literals need the line deleted; pass6 helper drops `rule.winter_prep &&` |
| `society.exposure.use_hardiness` | `true` | n/a (`exposure` None) | `sim-systems/src/runtime/settlement/fire.rs:259`; `sim-systems/src/runtime/settlement/settlement_system.rs:1083`; `sim-systems/src/worldgen/camp.rs:162` | damage without the hardiness factor; no `Hardiness` on founders/newborns under exposure → the two `is_some_and(use_hardiness)` become `exposure.is_some()` | asserts only: `sim-test/tests/harness_revitalize_pass10_exposure.rs:189`, `harness_revitalize_pass11_exposure.rs:179` | **yes**: `sim-test/src/probe/exposure_health.rs:79,83` (`exposure_v2` block gate + value), `:110` (gate of `sustain_utilization`) | removing it changes the per-seed `exposure_v2` key (and its gate must become `hunger_weight != 0.0` or unconditional) |
| `society.fission.stock_share` | `true` | n/a (`fission` None) | `sim-systems/src/runtime/settlement/fission.rs:963` (else `(0, 0)` :988) | group leaves without its share of food/wood | none | **yes**: `sim-test/src/probe/exposure_health.rs:102` (per-seed `fission` block) | |
| `society.fission.hazard` (`Option<FissionHazard>`) | `Some(..)` | n/a | `sim-systems/src/runtime/settlement/fission.rs:690-695` (`None =>` crowding gate) | v17 crowding gate; make the field `FissionHazard` (pressure "requires hazard" then always holds) | `sim-test/tests/harness_revitalize_pass13_fission.rs:49` sets `Some(START_HAZARD)` (same side); readers `harness_revitalize_pass12_fission.rs:28,116` (`.and_then(|f| f.hazard)`) | not directly (fission_checks output is unchanged) | makes `crowding` unread, see 1b |
| `society.fission.pressure.reset_on_split` | `true` | n/a | `sim-systems/src/runtime/settlement/fission.rs:1070` | parent keeps its pressure after a split | `sim-test/tests/harness_revitalize_pass13_fission.rs:41` literal `true` (same side) | no | |
| `society.relations.avoid_gather` | `true` | n/a (`relations` None) | `sim-systems/src/runtime/decision/agent_decision.rs:1239` (→ `relations_rule.is_some()`), used at :1242 | no avoid list at the Gather-arm selectors | assert only: `sim-test/tests/harness_revitalize_pass21_relations.rs:215` | no | pass21 header :8-11 already says the `false` arms were removed in V8-0 |
| `society.relations.exchange_first` | `true` | n/a | `sim-systems/src/runtime/settlement/relations.rs:577` (`if rule.exchange_first &&`), `:645` (`if !rule.exchange_first && row.exchange &&` — whole arm dead) | aid-before-exchange order | assert only: `sim-test/tests/harness_revitalize_pass21_relations.rs:218` | no | |

### 1b. Candidates once inert test literals are edited — (a), (c) hold; the only "other value" in any test is a whole-struct legacy literal in a test that never reaches the branch

Strictly these fail (b) (a test literal names the other value). In every case the test only calls
functions that do not read the field, so deleting the field (and the literal line) cannot change any
assertion. Lead's call.

The two legacy `FirewoodRule` literals: **L1** = `sim-systems/src/runtime/settlement/firewood.rs:366-401` (`fn rule()`;
its tests call only `burn_hours`, `burn_per_day`, `wood_target`, `night_season`, `urgent_with`/`is_urgent`
→ `tonight_burn`/`severity_level_at`, `winter_severity`, `wood_in_hand`); **L2** = `sim-bridge/src/ffi/world_node.rs:4301-4336`
(`v11_firewood`; test calls only `collect_settlement_wood_targets` → `wood_target`).

| field path | baseline | branch sites removed | dead branch | only other-value test site (inert) | printed? | notes |
|---|---|---|---|---|---|---|
| `society.family.elder_partner_follows` | `true` | `sim-systems/src/runtime/settlement/fission.rs:891-895,904` (`follows.then_some(couples)` → `Some(couples)`) | elder partner stays behind | `sim-core/src/rules/routine.rs:235` literal `false` in `family_familiarity_threshold_preserves_integer_extremes` (calls only `familiarity_threshold`) | no | households tests pass `None` to `select_departure*` directly (a parameter, unaffected); reader `sim-core/src/components/family.rs:156` asserts true |
| `society.aging.elder_evening_tether` | `true` | `sim-systems/src/runtime/agent/movement.rs:212-215` (`match (flag, evening_radius)` → `match evening_radius`) | elder ignores the evening tether | `sim-systems/src/runtime/agent/aging.rs:196` `const RULE` `false` (tests: lifespan fns + `AgingSystem.tick`, no movement) | no | reader `sim-test/tests/harness_revitalize_pass6_elders.rs:65` (assert) |
| `society.firewood.claims_bounded_by_amount` | `true` | `sim-systems/src/runtime/decision/agent_decision.rs:242-246` (else `max_gatherers_per_spot`), `:1126`, `:3472` (`is_some_and(flag)` → `is_some()`) | unbounded claims / arrived gatherer drops claim | L1 `:391`, L2 `:4326` | no | reader `harness_revitalize_pass7_winter.rs:357` (assert) |
| `society.firewood.winter_regen_units_per_day` (`Option<u32>`) | `Some(6)` | `sim-systems/src/runtime/resource_regen/mod.rs:437` (the `None` arm stays reachable outside winter via `.filter(|_| winter)`), `sim-systems/src/runtime/settlement/firewood.rs:242` (`?`) | per-spot all-year regrowth in winter | L1 `:384`, L2 `:4319` | no | field can become `u32`; the `match` keeps its winter filter |
| `society.firewood.winter_deadwood_cap` (`Option<u8>`) | `Some(1)` | `sim-systems/src/runtime/resource_regen/mod.rs:433` (`if let Some(cap)`), `:451,:456` (passed as `Option`), `:464` (`.filter(|_| winter)` keeps a reachable `None` outside winter) | no snow burial / no winter cap | L1 `:385`, L2 `:4320`; `harness_revitalize_pass6_winter.rs:227` sets `Some(0)` (same side) | no | field → `u8`; helper params stay `Option<u8>` |
| `society.firewood.winter_night_burn_permille` (`Option<[u16;3]>`) | `Some([250,500,250])` | `sim-systems/src/runtime/settlement/firewood.rs:160` (let-else `return 0`), `:222` (`is_some() &&`) | every night at its severity's rate | L1 `:392`, L2 `:4327`; `harness_revitalize_pass8_winter.rs:58` sets `Some(..)` (same side) | **yes**: `sim-test/src/probe/night_burn.rs:49` (`rules` block of `summary.json`; test `:118-119`) | |
| `society.firewood.windfall_over_snow` | `true` | `sim-systems/src/runtime/resource_regen/mod.rs:428` (`!rule.windfall_over_snow \|\|`) | burial at every winter pulse | L1 `:395`, L2 `:4330` | **yes**: `sim-test/src/probe/night_burn.rs:52` (`rules` block) | reader `harness_revitalize_pass6_winter.rs:193` (assert) |
| `society.firewood.regional_wood` | `true` | `sim-systems/src/runtime/resource_regen/mod.rs:447` (keep `!resources.wood_spot_area.is_empty()`; `regen_wood_winter` stays reachable through it) | world-total round robin | L1 `:397`, L2 `:4332` | **yes**: `sim-test/src/probe/exposure_health.rs:90,95` (`firewood_v17` gate + value) | |
| `society.firewood.floor_separate` | `true` | `sim-systems/src/runtime/resource_regen/mod.rs:323` (via param `:271`, sole call `:453`) | shared accumulator for the floor extra | L1 `:400`, L2 `:4335` | no | `regen_wood_winter_by_area` has one caller; param can go |
| `society.fission.crowding` (f64, dependent) | `1.0` | only reader `sim-systems/src/runtime/settlement/fission.rs:692`, inside the `hazard: None` arm of 1a | — (unread once `hazard` is non-optional) | `sim-test/tests/harness_revitalize_pass17_family_fission.rs:23` sets `0.0` with hazard on (inert) | **yes**: `sim-test/src/probe/exposure_health.rs:99` (per-seed `fission` block) | remove only together with `hazard` |

### 1c. Unread fields (no production reader at all; removal is trivially behaviour-preserving)

| field path | baseline | readers | test sites | printed? |
|---|---|---|---|---|
| `society.firewood.rain_soak_permille` | `300` | none in sim-core/systems/engine/bridge (`Intervention::Rain` was removed; see `sim-systems/src/runtime/settlement/fire.rs:40`) | L1 `:396`, L2 `:4331` (`0`, inert) | **yes**: `sim-test/src/probe/exposure_health.rs:90,94` (`firewood_v17` gate + value) |
| `society.firewood.rain_wet_burn` | `true` | none (`fire.rs:40` comment: acted only on intervention rain) | L1 `:399`, L2 `:4334` (`false`, inert) | no |

## 2. Near-misses ((a) holds, but a test sets/exercises the other value)

Default() route (`SimResources::new` → `WorldRules::default()`, `sim-engine/src/lib.rs:652`; explicit uses e.g.
`harness_revitalize_pass3_food.rs:19`, `harness_revitalize_pass3_village.rs:17`, `harness_revitalize_pass9_rng.rs:212`,
`harness_revitalize_survival_fix.rs:169,254`, `harness_revitalize_loop.rs:190`, `sim-systems/src/worldgen/mod.rs:150`):
default() differs from baseline for every direct field of `WorldRules`, `AgentRules`, `SocietyRules`, so each stays:

- `seasons` (Some → None)
- `agent.idle_tether_radius`, `agent.social_seek`, `agent.critical_preemption`, `agent.night_rest`, `agent.evening_gather`, `agent.forage`, `agent.temperament`, `agent.routine` (Some → None)
- `agent.need_fallthrough`, `agent.strict_consume`, `agent.preempt_gathering`, `agent.keyed_draws`, `agent.local_food_availability` (true → false)
- `society.family`, `shelter_planning`, `relief_radius`, `life_stage`, `stock_target`, `aging`, `meals`, `firewood`, `exposure`, `fission`, `relations` (Some → None)
- `society.birth` (`Provisioned` → `Legacy`; also asserted `harness_revitalize_loop.rs:194`)
- `society.settlement_anchor` (`Hearth` → `AnyCluster`; arms `settlement_system.rs:428,443`)

Inside sub-rules (tests that really run the other side):

- `agent.temperament.effects` — `sim-systems/src/runtime/decision/agent_decision.rs:4329` (`inert_temperament_keeps_the_global_window` ticks `AgentDecisionSystem` with `effects = false`).
- `society.birth.sustain` (Some → None) — `sim-test/tests/harness_revitalize_pass17_family_births.rs:34`.
- `society.shelter_planning.housing` (`Owned` → `NearFormation`) — `sim-systems/src/runtime/settlement/housing.rs:71` (asserts capacity 32).
- `firewood.winter_prep_days` (None) — L1 `:381` used by `target_adds_a_winter_of_burn_in_summer_and_autumn_only` (asserts `7.0 × 39`), L2 `:4316` (asserts 292).
- `firewood.winter_prep_in_summer` (true) — same two tests assert the summer prep (L1 `:382`, L2 `:4317`).
- `firewood.winter_base_days` (None) — L1 `:383`, winter day 21 asserts `1.5 × 39`.
- `firewood.night_season_from_evening` (false) — `firewood.rs:449,457` (`night_season_belongs_to_the_evening_only_when_enabled`).
- `firewood.urgent_start_minute` (None) — L1 `:387` in `urgent_needs_the_rule_and_less_than_tonights_burn` ("urgent mode is off without a start minute").
- `firewood.winter_severity_permille` (None) — `firewood.rs:494,521`; `sim-bridge/src/ffi/world_node.rs:3571` (runs 21 days).
- `fission.pressure` (None) — `sim-bridge/src/ffi/world_node.rs:3910` (runs 1,440 ticks on the hazard-draw path).
- `fission.pressure.threshold_table` (`Exponential` → `Uniform`) — `sim-core/src/fission_threshold.rs:184,194` (unit tests of `fission_threshold`).
- `fission.allowed_seasons` (winter `false` → all false) — `sim-test/tests/harness_revitalize_pass12_fission.rs:248`.
- `agent.routine.end_stagger` (None) — `sim-core/src/rules/routine.rs:333` (`end_minute` unit test asserts 1080).

Zero-sentinel arms (the field's value is used, so the field stays; only the `== 0` arm is dead in baseline):

- `relations.dispute_cap_per_pair_day` = 3: arms `agent_decision.rs:3050,3061`; a test sets 0 — `sim-test/src/probe/relation_caps.rs:76` (probe key gate only, `relation_caps.rs:26`).
- `relations.quarrel_cap_per_pair_day` = 1: arms `agent_decision.rs:3051,3063`; **no test sets 0** (arm removable; `capped` stays true through the dispute cap).
- `firewood.winter_fall_spread` = 2: `firewood.rs:183` `spread == 0 ||`; 0 only in L1/L2 (inert). Printed `night_burn.rs:51`.
- `firewood.area_fall_floor_permille` = 1000: `resource_regen/mod.rs:288` `floor_permille > 0` else-empty set; 0 only in L1/L2 (inert).
- `exposure.hunger_weight` = 0.5: `fire.rs:264` `!= 0.0` guard; no test sets 0.0. (The guard is arithmetically a no-op skip; printed `exposure_health.rs:79,83`.)
- `firewood.winter_night_burn_step` = 2 (0 only in L1/L2) has no branch; plain value. `fission.hazard.weight_crowd_permille` = 0 makes the crowd term `q` contribute nothing to `h` in baseline, but `q` is logged in `FissionCheck` and tests set 350/500 (`pass13_fission.rs:31`, `fission.rs:1150`).

## 3. Count

- Fields examined: **208** (all fields of `WorldRules` and the 27 reachable types, enum-variant fields included:
  WorldRules 4, AgentRules 15, TemperamentRule 7, SocialSeekRule 2, SocietyRules 13, BirthRule::Provisioned 5,
  ShelterPlanningRule 9, SustainRule 4, NightRestRule 5, EveningGatherRule 3, LifeStageRule 5, ForageRule 5,
  StockTargetRule 3, AgingRule 8, MealRule 6, FirewoodRule 34, ExposureRule 4, FissionRule 16, ThresholdLaw 3,
  FissionPressure 3, FissionHazard 8, SeasonRule 2, RoutineRule 11, EndStagger 3, FamilyRule 3, RelationsRule 27).
- Flag-like (bool, Option, enum, bool array): 59 — strict candidates 9 (1a), literal-edit candidates 9 + 1 dependent (1b),
  unread 2 (1c, one is a bool), near-misses 27 via default() + 13 inside sub-rules.
- The other ~149 are plain numerics/arrays: no value of theirs makes a branch unreachable, except the 5 zero-sentinel arms listed above.
