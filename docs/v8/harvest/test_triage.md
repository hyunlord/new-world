# V8-0 test triage — `rust/crates/sim-test/tests/*.rs`

Draft for GOAL 6.5 (test triage) and the coverage table. Read-only analysis: no
`cargo` or Godot run (a measurement was running), so "passes at base" is taken from
`.harness/baseline/known_failures.txt` ("ZERO known baseline failures") and is not
re-checked here.

Scope: **151** `harness_*.rs` files (GOAL 6.5 says 150), plus the two support
modules `tests/pass17_family_support/mod.rs` and `tests/pass17_routine_support/mod.rs`
(not test files; they are adapted together with their users).

## 1. Method and facts the verdicts rest on

Per-test evidence was gathered with a brace-matching splitter over every `#[test]` fn
(transitively through local helpers), grepping for: file reads (`include_str!`,
`read_to_string`, `read_dir`) of `.gd/.tscn/.gdshader/.tres/localization/project.godot`
or `.rs` sources; `.harness`; `intervention`; `worldmodel`; scene and rules
constructors; engine construction and system registration.

Facts used throughout:

- **Default rules switch off 8 of the 28 live systems.** `WorldRules::default()`
  (`sim-core/src/rules.rs:615-650`) has every `SocietyRules` option `None`. The
  systems return at once without their rule: `ShelterPlanningSystem`
  (`settlement/shelter_planning.rs:29`), `FireSystem` (`settlement/fire.rs:92`),
  `FissionSystem` (`settlement/fission.rs:842`), `RelationsSystem`
  (`settlement/relations.rs:174`), `AgingSystem` (`agent/aging.rs:107`),
  `PairingSystem` (`agent/pairing.rs:23`), `BedAssignmentSystem` (`agent/beds.rs:23,26`),
  `MaturationSystem` (`agent/maturation.rs:21`). `StaleSeekTargetSystem` and
  `ResourceRegenSystem` do nothing until finite scarcity is seeded (`sim-systems/src/lib.rs:268-274`).
  Every V7-era test (`SimEngine::new`, `make_stage1_engine`, `register_*`) runs on
  default rules, so none of them covers the 8 gated systems.
- **Only one test file uses `game_v26`.** That file is `harness_revitalize_pass21_relations.rs`
  (`game_v26_scene` and `WorldRules::game_v26`; grep of `game_v26` over `tests/`).
  `harness_p21_game_v26_invariants_and_twin` runs all 28 systems on live rules for
  60 days, but it asserts only relations and fission invariants, plus that the tallies
  match the chronicle.
- **`build_world(cfg)` means `game_v8`.** `build_world` calls
  `build_world_with(cfg, WorldRules::game_v8())` (`worldgen/mod.rs:234-236`).
  `scenes::game_scene` = `build_world(game_default)`. `game_v8`, `game_default` and
  `game_scene` are not on the GOAL 8.1 deletion list by name. They are the root of the
  chain (`game_v9` starts from `game_v8()`, `rules.rs:1297`), though, and nothing
  live uses them. **Assumption:** they go with the chain. The tests below that use
  them are adapted or deleted on that basis.
- **The bridge helpers stay, but only tests use them.** `init_production_engine`,
  `bootstrap_spawn_agents` and `seed_finite_resource_scarcity`
  (`sim-bridge/src/ffi/world_node.rs:3018,3097,3117`) are not in the deletion list.
  In production only `scenes::legacy_scene` uses them (live init is
  `build_game_engine`, `world_node.rs:204`). About 20 KEEP files rely on them, so they
  must survive the `legacy_scene` deletion.
- **`worldmodel::digest::state_digest` is deleted, and 19 files import it.** Every
  frozen pin uses it, as do `pass7_interventions`, `pass8_winter`, `pass9_rng`,
  `pass10_exposure`, `pass10_forks`, `pass10_variety`, `pass13_world` and
  `pass16_temperament`. Kept ports must switch to `sim_test::probe::fingerprint`.
- **The bridge FFI read by the kept UI** (string grep of the 10 kept scripts) is
  `get_agent_snapshot, get_agent_detail, get_settlement_snapshot,
  get_construction_snapshot, get_building_snapshot, get_resource_snapshot,
  get_recent_deaths, get_time_snapshot, get_tallies, get_world_info,
  get_terrain_snapshot, get_porter_snapshot, get_activity_snapshot, get_person,
  get_person_chronicle, get_chronicle_since, get_agent_temperament, get_agent_family,
  get_society_snapshot, get_tile_causal_history, get_tile_causal_events,
  get_influence_overlay, set_sim_speed`. Two collectors still exist but no kept script
  reads them: `get_relationship_snapshot`, and the `seek_kind`/target fields of the
  agent snapshot (`seek_kind` and `target_x` grep empty in kept UI). Their collector
  tests are kept and flagged.
- **One test counts tests, and it will break.**
  `harness_p8_beta_a26_test_count_regression_guard` counts `#[test]` across
  `rust/crates/*` against `STATIC_TEST_FLOOR` 1600. The triage deletes about 700 tests,
  so the guard fails. It is deleted, together with its meta-harness
  `harness_count_guard_static.rs`.

### Verdict rules applied (GOAL 6.5 plus the brief)

- **DELETE** a file in any of these cases:
  - Its subject is UI or locale source text: static string assertions on `.gd`,
    `.tscn`, `localization/*`, or the pipeline-delegated stubs.
  - It targets a deleted feature: interventions, worldmodel, `.harness`, deleted UI.
  - It is a frozen pin, a `vN_unchanged` comparison, or `inert_before_vN`.
  - It only exercises behaviour that exists solely in a non-`game_v26` scene or
    version.
  - It duplicates a kept test.
- **Rust-source text greps.** I treat static assertions on `.rs` source text (for
  example "`world_node.rs` contains `self.engine.resources.current_tick`") like
  UI-text assertions: they are scaffolding that the 8.2 renames break, and compile
  and behaviour tests cover the same ground. **This extends the brief and is my
  judgment;** each such fn is named in its row.
- **Old-version scene runs.** A scene test whose subject is a version delta, a pin or
  a seed-tuned outcome is **deleted**. A scene test whose subject is a
  version-independent invariant of a live feature is **KEEP-ADAPT (R3)**, ported to
  the baseline scene. Examples of such invariants: "villagers sleep in beds at night",
  "children mature on schedule", "chronicle rebuilds rosters", "noise fork with
  replicate 0 is the identity". This reads the brief's KEEP-ADAPT case (a kept test
  calling a deleted constructor) as covering scene constructors.

### Adaptation recipes

- **R1 (rebase on the baseline).** In a `SimEngine::new` unit fixture, replace
  `WorldRules::game_vN()` and `game_vN(seed).x` with `WorldRules::baseline(seed)`, or
  with the needed sub-rules copied from it into `WorldRules::default()`. Derive
  expected numbers from the rule (`rule.burn_per_hour[…]`), not from literals.
  - **Exact** (same values): when the copied fields did not change between vN and
    v26. That holds for `night_rest`, `evening_gather`, `life_stage`,
    `shelter_planning`, `meals`, `forage` (v11+), `stock_target` (v11+), `temperament`,
    `social_seek`, `need_fallthrough`, `critical_preemption`, the v20 `firewood`, and
    the fission group fields.
  - **Re-verify** in every other case.
- **R2 (test-local literal of an old rule value).** Use only for numeric parameters.
  **Never use it for an `Option` or `bool` flag whose non-baseline side GOAL 8.3 might
  delete**, because a kept test setting the other value blocks that removal (8.3,
  condition 2). Rows that would need R2 on a flag say so and delete the fn instead.
- **R3 (port the scene to `baseline_scene(seed)`).** Drop the comparison arm against
  the old version. Re-measure any seed-specific count or threshold (founder count,
  deadwood count and capacity now come from `WorldVariety::draw_v18`). Replace
  `state_digest` with `probe::fingerprint`.

### A ruling DECISIONS needs (affects 8.3)

The 51 KEEP V7 files and most adapted unit fixtures run on `WorldRules::default()`.
That takes the non-baseline side of many flags:

- **Agent:** `strict_consume=false`, `keyed_draws=false`,
  `local_food_availability=false`, `need_fallthrough=false`, `critical_preemption=None`,
  `night_rest=None`, …
- **Society:** `BirthRule::Legacy`, `SettlementAnchor::AnyCluster`, and every option
  `None`.

Read literally, 8.3's "no kept test sets it to the other value" keeps all those
branches alive. Decide whether `default()`-based tests count as "setting" a field. If
they do, 8.3 can remove almost nothing in `AgentRules` or `SocietyRules`. If they do
not, record that rule-independent tests run on `default()` by design.

## 2. Per-file triage

Abbreviations:

| Abbreviation | Meaning |
|---|---|
| `new` | `SimEngine::new` with `WorldRules::default()` |
| `+reg` | `register_default_runtime_systems` |
| `+ph2` | `register_phase2_systems` (or the agent, needs and decision registration helpers) |
| `stage1` | the file-local `make_stage1_engine` |
| `boot` | the bridge's `bootstrap_spawn_agents` |
| `scar` | `seed_finite_resource_scarcity` |
| `prod` | `init_production_engine` |
| T | static text assertion (UI, locale or `.rs` source) |

| file | verdict | subject | scenes/constructors used | reason | fns to delete or adapt |
|---|---|---|---|---|---|
| harness_a15_controlled_determinism.rs | KEEP-TRIM | AgentDecisionSystem migrant-tag determinism, lockstep twins | `new`+reg, boot, scar | engine determinism on default rules; one fn reads `.harness/` | delete `harness_a15_baseline_registry_consistency` (reads `.harness/baseline/known_failures.txt`, L512-513) |
| harness_art_p9_sim_reads.rs | DELETE | GDScript `SimReads` routing scanner | — (T: scripts/**, `reset_world_stress`, `apply_intervention`) | UI source text; references deleted bridge funcs | all |
| harness_b1_zoom_default_raise.rs | DELETE | camera ZOOM constants | — (T: `camera_controller.gd`, docs/art) | UI source text | all |
| harness_carry_viz_2_5b.rs | KEEP | `collect_agent_snapshot.carried_food` | `new` | kept collector (`carried_food` read by `agent_renderer.gd`) | — |
| harness_count_guard_static.rs | DELETE | meta-harness of the p8β A26 test-count guard | — (T: reads test sources) | pass scaffolding; the guarded test is deleted | all |
| harness_d_phase_a_runtime_warnings.rs | DELETE | GDScript warnings plus FFI presence by grep | — (T: `.gd` and `world_node.rs`) | UI and Rust source text | all |
| harness_fix_d_memory_arm.rs | KEEP | MemorySystem arm classification, scarcity-scene lockstep | `new`+reg, boot, scar | engine behaviour, default rules | — |
| harness_fix_inspector_overlap.rs | DELETE | inspector/topbar geometry constants | — (T) | UI source text | all |
| harness_g_phase_a_agent_frame_zoom.rs | DELETE | agent sprite frame, zoom constants | — (T, docs/art) | UI source text | all |
| harness_h_phase_a_input_camera_color.rs | DELETE | trackpad, pause, camera tracking, shader | — (T) | UI source text | all |
| harness_inventory_2_1_data.rs | KEEP | `Inventory` / `ResourceKind` / settlement stockpile data, serde | pure sim-core | rule-independent data behaviour | — |
| harness_membership_belonging.rs | KEEP | SettlementSystem membership model | `new`+reg, boot, `enqueue_building_placed` | engine behaviour | — |
| harness_newborn_mobile.rs | KEEP | newborns get MovementRng and growth rates, actually move | `new`+reg, boot | engine behaviour | — |
| harness_next_a_postverify.rs | KEEP | building queue → BuildingStamp → InfluenceUpdate via FFI enqueue | `new`+ph2 | engine and bridge behaviour | — |
| harness_p10_alpha_settlement.rs | KEEP | Settlement component, constants, `AgentBorn` event | pure, stage1 | rule-independent | — |
| harness_p10_beta_settlement_system.rs | KEEP | SettlementSystem formation, dissolution, births, migration pull, history | `new`+reg | engine behaviour | — |
| harness_p10_gamma_settlement_chronicle.rs | KEEP | settlement causal chronicle end to end, determinism | stage1, `new`+reg | engine behaviour (`.harness` only in a doc comment, L31) | — |
| harness_p10_gamma_settlement_migration.rs | KEEP | migration FSM pathing, auto-join, aborts | `new`+reg, boot | engine behaviour | — |
| harness_p11_alpha_agent_renderer.rs | KEEP-TRIM | `collect_agent_snapshot` state_tag, positions, ids | `new` | collector tests kept; renderer and shader text deleted | delete (T): `harness_p11_alpha_agent_renderer_enables_use_colors`, `…_reads_states_array`, `…_interpolates_positions`, `…_applies_state_tints`, `harness_p11_alpha_shader_captures_instance_color`, `…_shader_modulates_palette_output`, `…_checksum_covers_all_rows`, `…_checksum_is_identity_aware`, `…_d1_idle_tint_is_cool_blue`, `…_d1_no_idle_white_identity_tint`, `…_d1_state_tints_exact_ordered_palette`, `harness_revitalize_agent_shadows_follow_interpolated_feet`, `harness_revitalize_agent_reset_discards_old_world_interpolation` |
| harness_p12_alpha_camera_zoom.rs | DELETE | camera zoom, `main.tscn` | — (T) | UI source text | all |
| harness_p12_beta_terrain.rs | DELETE | tileset, terrain renderer, runtime-harness script | — (T, `.harness`) | UI and asset text | all |
| harness_p12_beta2_construction_sites.rs | KEEP-TRIM | `collect_construction_snapshot` | pure collector | collector kept (`get_construction_snapshot` read by kept UI) | keep a6, a7; delete (T): a1, a2, a3, a4, a5 (bridge source), a8, a9, a10, a11, a12, a13, a14 (GDScript) |
| harness_p12_gamma_settlement_furniture.rs | KEEP-TRIM | `collect_settlement_snapshot` centroid and formation | pure collector | collector kept | keep a6, a7, a8; delete (T): a1–a5, a9–a15 |
| harness_p13_alpha_camera_and_buildings.rs | DELETE | camera, building sprite paths | — (T) | UI source text | all |
| harness_p13_beta_resource_placeholders.rs | DELETE | resource placeholder sprites | — (T) | UI source text | all |
| harness_p13_delta_basic_hud.rs | DELETE | `hud_topbar.gd` structure | — (T) | UI source text (kept UI file, V7 text) | all |
| harness_p13_epsilon_bootstrap_seed.rs | DELETE | bootstrap building sprites | — (T) | UI source text | all |
| harness_p13_gamma_interaction_cues.rs | DELETE | `STATE_SCALE_BOOST` | — (T) | UI source text | all |
| harness_p14_alpha_agent_sprite_overhaul.rs | DELETE | role hue, head icon | — (T) | UI source text | all |
| harness_p14_beta_resource_building_variety.rs | DELETE | resource and fixture sprite paths | — (T) | UI source text | all |
| harness_p14_delta_status_panel.rs | DELETE | `hud_status_panel.gd` | — (T) | deleted UI | all |
| harness_p14_epsilon_activity_trails.rs | DELETE | `activity_trail_renderer.gd` | — (T) | deleted UI | all |
| harness_p14_gamma_click_inspector.rs | KEEP-TRIM | `collect_agent_detail` contract | stage1, `new`+reg | collector kept (`get_agent_detail`) | delete (T): a13, a14, a15 (`world_node.rs` text), a16–a24, a26, a27, a28, `harness_revitalize_inspector_gather_food_is_not_social_target` (GDScript); delete a29 `workspace_gate_clean` and a30 `gdscript_parse_clean` (eprintln-only stubs) |
| harness_p14_zeta_zoom_adaptive.rs | DELETE | `zoom_lod_controller.gd`, settlement overview | — (T) | deleted UI | all |
| harness_p3_alpha_event_recording.rs | KEEP | tile causal log (8-event FIFO) | `new`+ph2 | engine behaviour | — |
| harness_p3_beta_causal_chain.rs | KEEP | event-id parent chains | `new`+ph2 | engine behaviour | — |
| harness_p3_gamma_1_ffi.rs | KEEP | `collect_tile_causal_history`, `collect_event_chain` | `new`+ph2 | kept collectors | — |
| harness_p3_gamma_2_alpha_locale_panel_scaffold.rs | DELETE | `locale.gd`, `causal_panel.gd`, locale keys | — (T, localization) | UI and locale text | all |
| harness_p3_gamma_2_beta_tile_click_chain.rs | DELETE | world_renderer click, causal panel text, compiled keys | — (T) | UI and locale text | all |
| harness_p4_alpha_agent_core.rs | KEEP | Position/Agent components, `spawn_agent`, AgentInfluenceSample | `new` | engine behaviour | — |
| harness_p4_beta_movement.rs | KEEP | AgentMovementSystem Brownian step, MovementRng | `new`+ph2 | engine behaviour | — |
| harness_p4_gamma_rendering.rs | KEEP | `collect_agent_snapshot` positions and order | `new`+ph2 | kept collector | — |
| harness_p5_alpha_agent_id_hunger.rs | KEEP | AgentId minting, Hunger, HungerDecaySystem | `new`+ph2 | engine behaviour | — |
| harness_p5_beta_decision.rs | KEEP | Thirst, AgentState FSM, AgentDecisionSystem need cascade | stage1, `new`+ph2 | engine behaviour | — |
| harness_p5_gamma_sleep_daynight_chronicle.rs | KEEP | Sleep, SleepDecaySystem, time of day, sleep FSM | `new` | engine behaviour | — |
| harness_p6_alpha_construction_components.rs | KEEP | blueprint / ConstructionSite components | pure | rule-independent | — |
| harness_p6_beta_construction_system.rs | KEEP | ConstructionSystem and the 4-link causal chain | `new`+ph2 | engine behaviour | — |
| harness_p6_gamma_construction_chronicle.rs | KEEP | construction chronicle | `new`+ph2 | engine behaviour | — |
| harness_p7_alpha_social_components.rs | KEEP-TRIM | Social, RelationshipKey/State, TargetKind::Agent | stage1, `new` | components kept; source audits deleted | delete (T): `harness_p7_alpha_a10_social_no_default_impl`, `…_a34_sim_resources_schema_unchanged`, `…_a35_no_phase7_beta_constants`, `…_a36_agent_decision_named_arms_audit`; adapt `…_a11_make_stage1_engine_helper_attaches_social` by removing sub-assert (b) (the `sim-engine/src/lib.rs` window scan) |
| harness_p7_beta_social_system.rs | KEEP | SocialInteractionSystem, SocialDecaySystem, social cascade | stage1, `new`+reg | engine behaviour | — |
| harness_p7_delta_social_ui.rs | KEEP-TRIM | agent-snapshot state_tag, `collect_relationship_snapshot` | stage1, `new`+reg | collectors kept (relationship snapshot has no kept-UI reader: flag) | delete (T): `harness_locale_compiled_contains_all_seven_keys_en`, `…_ko`, `harness_locale_seven_keys_pairwise_distinct_en`; delete (duplicates of p11-α a1–a5): `harness_state_tag_idle_default`, `harness_state_tag_seeking_agent`, `harness_state_tag_consuming_agent`, `harness_state_tag_consuming_other` |
| harness_p7_gamma_social_chronicle.rs | KEEP | two-agent social chronicle | `new`+reg | engine behaviour | — |
| harness_p8_alpha_memory_components.rs | KEEP-TRIM | Memory / MemoryEntry component | pure | component kept | delete (T): `harness_p8_alpha_a19_import_path_audit` (`include_str!` of its own file) |
| harness_p8_beta_memory_system.rs | KEEP-TRIM | MemorySystem encoding, decay, cascade bias | `new`+reg | engine behaviour | delete `harness_p8_beta_a26_test_count_regression_guard` (floor 1600 breaks after the triage); adapt `…_a22_panic_free_full_year_and_no_unwrap_in_cascade_helpers` by removing sub-b (`include_str!` of `agent_decision.rs` / `memory_system.rs`) |
| harness_p8_delta_memory_ui.rs | KEEP-TRIM | MemoryRecalled FFI dict, snapshot `agent_id` | stage1, `new`+reg | FFI dict and collector kept | delete (T): `…_locale_compiled_en_seven_keys_present`, `…_ko_seven_keys_present_with_hangul`, `…_seven_keys_pairwise_distinct_en`, `…_ko`, `…_ffi_dict_schema_single_source_of_truth` (`world_node.rs` text), `…_causal_panel_no_hardcoded_english_for_recalled_event`, `…_locale_compiled_json_structural_integrity`, `…_recalling_cue_has_real_caller`, `…_renderer_recall_lookup_uses_agent_id_domain`; delete `…_imports_compile` (trivial) and `…_phase7_delta_state_tag_regression_guard` (duplicates p11-α) |
| harness_p8_gamma_memory_chronicle.rs | KEEP | MemorySystem lifecycle chronicle | `new`+reg | engine behaviour | — |
| harness_p9_alpha_body_health.rs | KEEP | BodyHealth, hostility | pure | rule-independent | — |
| harness_p9_beta_combat_system.rs | KEEP | CombatSystem, causal chain, purge | `new`+reg | engine behaviour | — |
| harness_p9_delta_combat_ui.rs | KEEP-TRIM | combat FFI dict kinds, `defender_id` | pure FFI-dict | FFI dict kept | delete (T): `…_locale_en_five_keys`, `…_locale_ko_five_keys`, `…_locale_pairwise_distinct_en`, `…_panel_no_hardcoded_english`, `…_phase8_recall_regression` |
| harness_p9_gamma_combat_chronicle.rs | KEEP | combat chronicle | `new`+reg | engine behaviour | — |
| harness_phase2_determinism.rs | KEEP | influence propagation determinism | `new` | engine behaviour | — |
| harness_phase2_ffi.rs | KEEP | `enqueue_building_placed`, stamp dirty regions | `new` | engine and bridge behaviour | — |
| harness_phase2_substantial.rs | KEEP | full Phase 2 pipeline, 1k agents | `new`+ph2 | engine behaviour | — |
| harness_phase2.rs | KEEP | influence system metadata, AgentInfluenceSample, viz | `new`+ph2 | engine behaviour | — |
| harness_resource_deposit_to_stockpile.rs | KEEP | StockpileDepositSystem | `new`+reg, boot | engine behaviour | — |
| harness_resource_gather_pickup.rs | KEEP | gather pickup to Inventory, FSM | stage1, `new`+reg, boot | engine behaviour | — |
| harness_resource_scarcity.rs | KEEP | finite resources, ResourceRegen, StaleSeekTarget, mortality | prod, `new`+reg, boot, scar | engine behaviour; needs `prod` and `scar` to survive | — |
| harness_revitalize_legacy_pin.rs | DELETE | `legacy_scene` trajectory pin | `legacy_scene` | frozen pin of a deleted scene | all |
| harness_revitalize_loop.rs | KEEP-ADAPT | v8 game loop: survival, shelters, births, builders, movement cadence | `game_scene` (= v8), `WorldRules::game_v8`, `default` | mixed | R3: `harness_population_game_world_survives_3_days`, `harness_building_shelters_planned_and_completed`; R1 (re-verify housing, since baseline counts owned housing with headroom 2): `harness_population_births_gated_by_food_and_housing`, `harness_building_assigned_builder_walks_and_clears_missing_site`; delete `harness_social_idle_agents_stay_near_home` (v8 idle tether on a v8 world; v26 routines move adults away); keep as is: `harness_population_rules_default_is_legacy`, `harness_population_movement_only_steps_on_configured_tick_multiples` |
| harness_revitalize_pass10_exposure.rs | KEEP-ADAPT | exposure v1 (`game_v16_neutral`) | `game_v15_scene`, `game_v16_neutral(_scene)`, `state_digest` | live exposure is v2 (`hunger_weight` 0.5, `use_hardiness` true) | delete `harness_exposure_inert_before_v16` (v15 pins), `harness_exposure_adult_survives_one_night` (v1 adult damage; pass11 `adult_can_die` covers v2), `harness_exposure_only_from_cold` (v16_neutral scene); R1 (baseline `exposure`, fixture needs hardiness; re-verify): `harness_exposure_kills_a_cold_elder`, `harness_exposure_blocks_healing`, `harness_exposure_order_independent` (replace `state_digest`) |
| harness_revitalize_pass10_forks.rs | DELETE | noise-fork identities on v16/v18 | `game_v16_scene`, `game_v18(_nofission)_scene`, `state_digest` | old scenes; the same properties are ported once from pass9_rng (R3) | all |
| harness_revitalize_pass10_frozen.rs | DELETE | v15 rules and fingerprint pins | `game_v15_scene` | frozen pin | all |
| harness_revitalize_pass10_variety.rs | KEEP-ADAPT | WorldVariety draws | `WorldVariety::draw` (v16 RANGES), `game_v16*` | live draw is `draw_v18` / `RANGES_V18` (`variety.rs:118-126`) | delete `harness_variety_neutral_equals_v16_neutral`; R1 (to `draw_v18`/`RANGES_V18`): `harness_variety_ranges`, `harness_variety_deterministic`; R3: `harness_variety_independent_of_replicate`, `harness_variety_worldgen_accepts_every_drawn_world` (needs a baseline worldgen `*_with(variety)` or equivalent); replace `state_digest` |
| harness_revitalize_pass11_exposure.rs | KEEP-ADAPT | exposure v2, hardiness, rain soak | `game_v16/v17_nofission`, `apply_intervention` | v2 is live | delete `harness_rain_soaks_only_in_v17` (`Intervention::Rain`, L253-267), `harness_v16_exposure_unchanged`; R1 (re-verify: baseline damage [7.5, 5, 5] vs v17 [9, 5, 6]): `harness_exposure_v2_formula`, `harness_exposure_adult_can_die`; R3: `harness_hardiness_keyed_and_ranged` |
| harness_revitalize_pass11_fission.rs | KEEP-ADAPT | fission group, founding, timeout, remnants, regions | `WorldRules::game_v17`, `game_v16/v17(_nofission)_scene` | the fixture triggers via the crowding gate (hazard `None`), which live rules never reach (`fission.rs:551-562`) | delete `harness_fission_inert_before_v17`, `harness_fission_only_when_crowded` (dead branch), `harness_fission_order_independent` (v17 scene; pass13 `pressure_order_independent` is the live-trigger twin); R1 (fixture rules from `baseline`, split driven through pressure; do **not** keep `hazard: None` (R2 on a flag)): `harness_fission_group_rules`, `harness_fission_group_resampled_by_replicate`, `harness_fission_founds_daughter`, `harness_fission_timeout_and_cancel`, `harness_remnant_merges`, `harness_shared_bushes_count_once`; R3: `harness_v17_regions_placed` (drop the v16 arm), `harness_fission_founds_daughter_in_a_whole_world` |
| harness_revitalize_pass11_frozen.rs | DELETE | v16 pins, `draw` literals | `game_v16_scene` | frozen pin | all |
| harness_revitalize_pass12_fission.rs | KEEP-ADAPT | fission hazard formula, seasons | `WorldRules::game_v17/v18`, `game_v18_scene` | the hazard is still live (`V19_HAZARD`, feeding pressure) | delete `harness_hazard_inert_in_v17`, `harness_hazard_keyed_with_replicate` (v18 scene, hazard-only branch), `harness_hazard_order_independent` (v18 scene); R1 (V19_HAZARD from baseline): `harness_hazard_formula`, `harness_hazard_rises_with_shortfall`, `harness_allowed_seasons` (drop the scene part) |
| harness_revitalize_pass12_frozen.rs | DELETE | v17 pins, RANGES literal | `game_v17(_nofission)_scene` | frozen pin | all |
| harness_revitalize_pass12_world.rs | KEEP-ADAPT | region fill, region water, local catchment, wet-wood burn | `WorldRules::game_v17/v18(_nofission)`, worldgen v17/v18, `apply_intervention` | mixed | delete `harness_rain_wet_burn_only_in_v18` (`Intervention::Rain`, L176), `harness_area_fall_floor` (v18_nofission scene); R3: `harness_region_fill_delivers_in_v18_only` (drop the v17 arm), `harness_region_fill_never_touches_terrain_or_water`, `harness_region_water_rule`; R1 (drop the v17 arms): `harness_local_food_availability_only_in_v18`; keep `harness_local_gather_stays_in_the_catchment` (pure fn) |
| harness_revitalize_pass13_fission.rs | KEEP-ADAPT | fission pressure and threshold (the live trigger) | `WorldRules::game_v17/v18/v19`, `game_v19_scene` | v19 fission = v26 fission (v20–v26 do not touch it) | delete `harness_pressure_inert_before_v19`, `harness_v18_hazard_unchanged`; keep `harness_threshold_table` (pure); R1 (re-verify: v23+ family `elder_partner_follows` alters split groups): `harness_pressure_accumulates_and_splits`, `harness_pressure_moves_with_food`, `harness_threshold_is_world_draw`; R3: `harness_pressure_order_independent` |
| harness_revitalize_pass13_frozen.rs | DELETE | v18 pins | `game_v18(_nofission)_scene` | frozen pin | all |
| harness_revitalize_pass13_world.rs | DELETE | settled-area floor v18 vs v19, v19 fork | `game_v18/v19_nofission_scene`, `state_digest` | version comparison; fork duplicate | all |
| harness_revitalize_pass16_frozen.rs | DELETE | v19 pins | `game_v19(_nofission)_scene` | frozen pin | all |
| harness_revitalize_pass16_temperament.rs | KEEP-ADAPT | temperament on every resident | `game_v19/v20/v20_inert_scene`, `state_digest` | temperament is live | delete `harness_v20_inert_equals_v19_at_40320_ticks`, `harness_v20_diverges_and_adults_wander_by_ns` (v19 vs v20 contrast); R3: `harness_v20_every_resident_has_temperament_and_rd_rate` (drop the inert arm) |
| harness_revitalize_pass17_family_beds.rs | KEEP-ADAPT | household bed packing (`housing_capacity`, beds) | `new` + `game_v20(7).{night_rest, shelter_planning}` | fields unchanged in v26 | R1 exact: both fns (`household_packing_…`, `oversized_household_…`); plus the support-module change |
| harness_revitalize_pass17_family_births.rs | KEEP-ADAPT | family births (mother choice, lineage, gates) | `new` + `game_v20(7).{shelter_planning, life_stage}` + support | fields unchanged in v26 | R1 exact for the three fns; support module (below) |
| harness_revitalize_pass17_family_fission.rs | KEEP-ADAPT | household-aware fission seeding | `WorldRules::game_v17(7)` + `V22_FAMILY` | the fission group fields are unchanged into v26 | R1 (baseline fission and family, re-verify `elder_partner_follows=true`) for all three fns; do not keep `game_v17` (hazard `None`) |
| harness_revitalize_pass17_family.rs | KEEP-ADAPT | PairingSystem, household partition, registration order | support (`V22_FAMILY`, `game_v20` temperament), `game_v21/v22_scene` | Pairing is live | keep `family_system_registration_follows_maturation_before_beds_once`; R1 via support for `pairing_uses_clock_offset_familiarity_ties_and_female_order`, `family_none_is_inert_and_dead_partner_is_cleared_only_at_pair_minute`, `pairing_excludes_close_kin_children_elders_and_below_threshold`, `household_partition_prefers_mother_and_preserves_widow_children_once` (re-verify: baseline family is V23, `min_familiarity` 200 and `elder_partner_follows` true); R3: `founders_receive_keyed_sex_and_empty_kin_only_with_family_enabled` (drop the v21 arm) |
| harness_revitalize_pass17_frozen.rs | DELETE | v20 pins | `game_v20(_inert)_scene` | frozen pin | all |
| harness_revitalize_pass17_routine_edges.rs | KEEP-ADAPT | routine edge cases | support (`game_v21` routine), `game_v20(7).{forage, stock_target, shelter_planning}` | routine is live | R1 (support routine = baseline V24 with `end_stagger`; re-verify `membership_change_aborts_and_day_end_completes_only_arrived_activities`; keeping `end_stagger: None` would block 8.3) for all 5 fns |
| harness_revitalize_pass17_routine_invitations.rs | KEEP-ADAPT | routine invitations and peers | support module only | routine is live | no edits in the file; re-verify the 4 fns after the support change |
| harness_revitalize_pass17_routine.rs | KEEP-ADAPT | routine arrival, interrupts, occupancy, haul block | support + `game_v20(7).society.firewood` | live | R1 (firewood is exact; the routine change is via support) for all 7 fns |
| harness_revitalize_pass18_frozen.rs | DELETE | v21/v22 pins | `game_v21/v22_scene` | frozen pin | all |
| harness_revitalize_pass19_chronicle_events.rs | KEEP-ADAPT | chronicle of a cancelled fission, remnant merge, dissolutions | `WorldRules::game_v17`, `new` | chronicle is kept UI data | keep `harness_chronicle_records_a_settlement_system_dissolution` (default rules); R1 (baseline fission, no `hazard: None`): `harness_chronicle_records_a_cancelled_fission_and_skips_the_dead`, `harness_chronicle_records_remnant_merges_and_empty_dissolutions` |
| harness_revitalize_pass19_chronicle.rs | KEEP-ADAPT | chronicle rebuilds rosters | `game_v23_scene`, `game_v16_scene`, `legacy_scene` | chronicle completeness is a live invariant | R3: `harness_chronicle_rebuilds_rosters_through_a_fission_game_v23`; delete `harness_chronicle_rebuilds_rosters_without_fission_and_stays_bounded` (needs a no-fission scene, none kept) and `harness_chronicle_rebuilds_rosters_in_the_legacy_scene` |
| harness_revitalize_pass19_frozen.rs | DELETE | v23 pins | `game_v23_scene` | frozen pin | all |
| harness_revitalize_pass20_frozen.rs | DELETE | v24 pins | `game_v24_scene` | frozen pin | all |
| harness_revitalize_pass20_relations.rs | KEEP-ADAPT | RelationsSystem core (measures, trips, porters, drift, disputes, quarrels) | `new` + `WorldRules::game_v25(SEED)`, `V25_RELATIONS`, `game_v25_scene` | RelationsSystem is live; pass21 tests only the v26 deltas | delete `harness_relations_rule_only_in_game_v25`, `harness_relations_rule_values_pinned` (V25 const pin), `harness_relations_game_v25_is_deterministic` (duplicates the p21 twin); R1 for the other 21: fixture `baseline(SEED)`, with `RULE = baseline relations` (V26), and re-derive thresholds. Keeping a V25 literal is exact (baseline with V25 relations is exactly `game_v25`) but sets `avoid_gather=false`, `exchange_first=false` and caps 0, which blocks 8.3 for those flags. Delete any fn whose assertion only holds with those V25 flags if pass21 covers the V26 side |
| harness_revitalize_pass21_frozen.rs | DELETE | v25 pins | `game_v25_scene` | frozen pin (pins v25, not v26) | all |
| harness_revitalize_pass21_relations.rs | KEEP-ADAPT | v26 relations deltas, 60-day live invariants | `game_v26(_scene)`, `game_v25`, `game_v24` | the only live-rules file | delete `harness_p21_game_v26_is_game_v25_with_the_v26_relations` (superseded by `sim-core/tests/baseline_matches_v7_final.rs`, GOAL 8.1) and `harness_p21_relations_only_in_v25_and_v26`; adapt `v26()` (L89) to `baseline(SEED)` and `game_v26_scene` to `baseline_scene`; drop the `v25()` (L85) contrast arms in `…gather_arm_and_post_pass_skip_the_estranged_side`, `…post_pass_without_a_target…`, `…exchange_first_takes_the_exchange_over_the_aid`, `…exchange_first_when_the_higher_id_sends_food`, `…keep_line_below_the_giver_threshold…`, `…integration_contact_falls_under_avoid…` (the v25 arms set non-baseline flags, which blocks 8.3); `…v26_relations_frozen_at_grid_g2` reads the baseline's relations if `V26_RELATIONS` is inlined |
| harness_revitalize_pass3_food.rs | KEEP-ADAPT | birth gate supply/need, strict consume, relief radius | `new` + `WorldRules::game_v9`, `game_v8` | mechanics are live | R1 (re-verify: baseline birth uses utilization 0.78 and `capacity_weighted`): all 6; drop the v8 arms of `harness_food_no_free_meals` and `harness_food_relief_reaches_tether` (they set `strict_consume=false` and relief `None`) |
| harness_revitalize_pass3_rhythm.rs | KEEP-ADAPT | night rest, beds, evening gather, maturation | `game_v9_scene`, `new` + `game_v9` | mixed | keep `harness_rhythm_minute_window_wraps_midnight`; R1 (exact for night_rest and evening_gather): `harness_rhythm_wake_for_critical_thirst`, `harness_rhythm_bed_seek_ignores_intermediate_shelters`, `harness_rhythm_reassigned_bed_replaces_existing_sleep_seek`; R3: `harness_rhythm_villagers_sleep_in_beds_at_night`, `harness_rhythm_evening_near_hearth`, `harness_rhythm_children_grow_up` (the only MaturationSystem behaviour test); delete `harness_rhythm_sleep_seek_does_not_dehydrate_seed33` (v9 seed regression) |
| harness_revitalize_pass3_village.rs | KEEP-ADAPT | hearth settlements, owned housing, planning headroom | `game_v9_scene`, `game_scene`, `new` + `game_v9/v8` | ShelterPlanning is live | R3: `harness_village_one_settlement_per_hearth`; R1 (exact for `shelter_planning`): `harness_village_owned_capacity_counts_each_shelter_once`, `harness_village_plans_with_headroom` (drop the v8 arm), `harness_village_completed_site_is_owned`, `harness_village_dissolution_releases_only_its_owned_buildings`; delete `harness_village_v8_unchanged` |
| harness_revitalize_pass4_aging.rs | KEEP-ADAPT | lifespans, founders, old-age death and replacement | `game_v10_scene`, `game_v9_scene`, `WorldRules::game_v10` | AgingSystem is live | R1 (exact: lifespan fields unchanged since v10): `harness_aging_lifespans_bounded_and_deterministic`; R3 (founder count is no longer 24): `harness_aging_founders`, `harness_aging_old_age_death_and_replacement` (the only AgingSystem-tick test), `harness_aging_elders_stay_home` (re-verify with routine elder weights); delete `harness_aging_v9_unchanged` |
| harness_revitalize_pass4_forage.rs | KEEP-ADAPT | purposeful forage, haul home, claims | `game_v10_scene`, `build_world_with` v9/v10, `new` + `game_v10` | forage is live (`max_foragers_per_bush` is 2 in v26 vs 1 in v10) | delete `harness_forage_seed8_no_longer_stagnates` (v9/v10 seed regression), `harness_forage_v9_unchanged`; R1 (re-verify): `harness_forage_claims_cap`, `harness_forage_retargets_when_bush_empties`, `harness_forage_gatherer_whose_tile_vanished_idles_instead_of_freezing`; R3: `harness_forage_stops_at_target`, `harness_haul_home_deposits`, `harness_haul_home_after_the_forage_window` |
| harness_revitalize_pass4_seasons.rs | KEEP-ADAPT | seasonal regrowth, winter store target | `new` + `game_v10`, `game_v9/v10_scene` | seasons are live (regen scaled by variety) | R1 (expected values from the rule): `harness_season_regen_schedule`, `harness_season_target_includes_winter`; delete `harness_season_survives_first_winter` (v10 tuning outcome), `harness_season_v9_unchanged` |
| harness_revitalize_pass5_firewood.rs | KEEP-ADAPT | deadwood, firewood trips, hearth fire, cold | `game_v10/v11_scene`, `new` + `game_v10/v11` | FireSystem is live | delete `harness_firewood_ffi_rows` (v11 seed-1 deadwood literal 20 × 6 plus a v10 comparison; the collector is covered by show_resource_depletion), `harness_firewood_v10_unchanged`; R3: `harness_deadwood_placement` (drop the v10 arm), `harness_firewood_stops_at_target`; R1 (re-verify: baseline has `claims_bounded_by_amount` and night-burn draws): `harness_wood_regen_schedule` (drop the v10 arm), `harness_firewood_trip_and_deposit`, `harness_firewood_claims_cap`, `harness_firewood_retargets_empty_spot`, `harness_fire_burns_and_goes_out`, `harness_fire_cold_raises_hunger` |
| harness_revitalize_pass5_meals.rs | KEEP-ADAPT | hearth meals from the store | `new` + `game_v11`, `game_v10_scene` | MealRule unchanged v11→v26 | R1 for the 5 meal fns; delete `harness_meal_v10_unchanged` |
| harness_revitalize_pass5_preempt.rs | KEEP-ADAPT | critical needs pre-empt gathering | `new` + `game_v11`, `game_v10_scene` | live | R1: `harness_gather_preempted_by_thirst`, `harness_preempted_gatherer_is_not_sent_out_again`; delete `harness_preempt_v10_unchanged` |
| harness_revitalize_pass6_elders.rs | KEEP-ADAPT | elder evening tether | `new` + `game_v12/v11/v10` | aging and evening_gather unchanged v12→v26 | R1 exact: `harness_elder_evening_tether_v12`, `harness_elder_tether_returns_after_the_evening`; delete `harness_elder_tether_v11_unchanged` (sets `elder_evening_tether=false`) |
| harness_revitalize_pass6_frozen.rs | DELETE | v11 pins | `game_v11_scene` | frozen pin | all |
| harness_revitalize_pass6_urgent.rs | KEEP-ADAPT | urgent firewood mornings | `new` + `game_v12`, `game_v11_scene` | `urgent_start_minute` 540 is live | R1 (re-verify "tonight's burn" with severity and night draws): `harness_urgent_window_opens_at_dawn`, `harness_urgent_two_per_spot`, `harness_urgent_ends_when_wood_is_in_hand`, `harness_urgent_is_per_settlement`; delete `harness_urgent_v11_unchanged` |
| harness_revitalize_pass6_winter.rs | KEEP-ADAPT | wood target formula, autumn prep, winter deadwood cap/regrowth | `WorldRules::game_v11/v12` | the firewood functions are live; v12 had no winter severity, baseline has severity `Some` | delete `harness_firewood_v11_unchanged`; R1: `harness_wood_target_formula_table` (keep its explicit literals, base it on the baseline rule), `harness_wood_target_by_season`, `harness_autumn_only_prep`, `harness_winter_cap_buries_deadwood`, `harness_spring_restores_deadwood`, `harness_last_winter_night_is_winter`, `harness_first_winter_night_is_autumn`; delete (severity-`None` / `winter_regen_units_per_day` branch, unreachable on baseline; R2 would block 8.3): `harness_winter_spread_regrowth`, `harness_winter_regrowth_branches_without_one_field` |
| harness_revitalize_pass7_frozen.rs | DELETE | v12 pins | `game_v12_scene` | frozen pin | all |
| harness_revitalize_pass7_interventions.rs | DELETE | god-hand interventions | `game_v13(_scene)`, `apply_intervention`, `worldmodel::schedule` | deleted feature | all |
| harness_revitalize_pass7_winter.rs | KEEP-ADAPT | winter severity draw, burn and fall by severity, bounded claims | `new` + `game_v13`, `game_v12` | severity is live | delete `harness_firewood_v12_unchanged`; R1: `harness_winter_severity_is_a_pure_draw` (drop the v12 arm), `harness_winter_burn_by_severity`, `harness_winter_fall_by_severity`, `harness_last_night_keeps_its_winter_severity`, `harness_autumn_prep_ignores_severity`, `harness_claims_bounded_by_amount`; R2 (numeric weights only): `harness_winter_severity_table` (pins [520, 280, 200]; baseline weights come from variety) |
| harness_revitalize_pass8_frozen.rs | DELETE | v13 pins | `game_v13_scene` | frozen pin | all |
| harness_revitalize_pass8_winter.rs | KEEP-ADAPT | night-burn and fall-spread draws | `new` + `game_v14/v13`, `build_world_with(game_v13)`, `apply_intervention`, `state_digest` | the draws are live | delete `harness_windfall_lands_on_snow` (`Intervention::Windfall`, L260), `harness_snow_burial_once_is_inert_without_windfall` (v13 worlds + `state_digest`, windfall-only property), `harness_firewood_v13_unchanged`; R1: `harness_night_burn_is_a_pure_draw`, `harness_fall_spread_is_a_pure_draw` (drop the v13 arms), `harness_villagers_do_not_see_tonights_draw`, `harness_last_night_keeps_its_evening_draw`, `harness_night_rate_floor_is_one` (drop the `cold_snap` arm, L205: `cold_snap_nights` is intervention state); R2 (numeric): `harness_night_and_fall_draw_table` |
| harness_revitalize_pass9_frozen.rs | DELETE | v14 pins | `game_v14_scene` | frozen pin | all |
| harness_revitalize_pass9_rng.rs | KEEP-ADAPT | keyed draws, lineage keys, noise forks | `game_v14/v15_scene`, `state_digest` | keyed draws are live | keep `harness_lineage_keys_stable_under_birth_shift`, `harness_lineage_keys_do_not_collide`, `harness_keyed_draw_is_pure`, `harness_keyed_step_balanced`; R3 (replace `state_digest` with `fingerprint`): `harness_lineage_founder_keys_follow_seed_and_spawn_index`, `harness_v15_newborns_carry_their_first_parents_key`, `harness_v15_movement_independent_of_history`, `harness_noise_fork_replicate0_is_identity`, `harness_noise_fork_prefix_identity`; delete `harness_v14_draws_unchanged` |
| harness_revitalize_screen_contract.rs | DELETE | renderer/HUD read only the snapshot (GDScript) | — (T) | UI source text | all |
| harness_revitalize_survival_fix.rs | KEEP-ADAPT | social chase limits, need fallthrough, critical preemption, clock | `new` + `game_v8`, `default`, `game_scene` | the mechanics are live | R1 (exact for `social_seek`, `need_fallthrough`, `critical_preemption`; re-verify the other v8 fields the fixture inherits, L17): all chase, fallthrough and preemption fns (9); R3: `harness_economy_clock_initial_values`; delete `harness_economy_clock_offset_is_display_only` (false under live rules: routine, pairing and relations add `clock_offset_ticks`) |
| harness_revitalize_tallies.rs | KEEP-ADAPT | cumulative Tallies for births and deaths | `new`+reg | engine behaviour | `harness_economy_tallies_count_births_and_deaths`: remove `interventions_applied: 0` from the Tallies literal (L76) when the field leaves `sim-engine/src/lib.rs:112`; keep `harness_economy_combat_death_uses_cumulative_tally` |
| harness_revitalize_worldgen.rs | KEEP-ADAPT | seeded worldgen determinism and invariants, building snapshot | `build_world(WorldGenConfig::game_default)` (= v8 rules) | worldgen is live | R3 for all 3 (to `WorldGenConfig::baseline` / `build_game_engine`); re-check the hard-coded seed list (L56) |
| harness_s16_alpha_seektarget.rs | KEEP | SeekTarget nearest-resource targeting | `new`, boot | engine behaviour | — |
| harness_s16_alpha0_resource_substrate.rs | KEEP-TRIM | bootstrap resource substrate, resource snapshot | `new`, boot | engine and collector | delete (T): `harness_resource_bridge_identity_contract_static`, `harness_resource_decorative_layer_preserved_static` |
| harness_s16_beta_movement.rs | KEEP | directed movement and the gathering loop | `new`+reg | engine behaviour | — |
| harness_s16_delta_need_stagger.rs | KEEP | bootstrap need stagger, growth rates | `new`+reg, boot | engine behaviour | — |
| harness_s16_epsilon_state_viz.rs | KEEP-TRIM | agent-snapshot `seek_kind` / target coords | `new`+reg, boot | collector exists (no kept-UI reader: flag) | delete (T): `harness_s16_epsilon_a9_renderer_and_scene_static_structure` |
| harness_s16_gamma_visual.rs | KEEP-TRIM | `clamp_sim_speed`, substrate, gathering FSM | `new`, boot | `set_sim_speed` is kept (`camera_controller.gd:80`) | keep a1, a10, a11; delete (T): a2, a3 (`world_node.rs` text), a4, a5, a6, a7, a8, a9 (GDScript) |
| harness_s16_zeta_social_freeze_fix.rs | KEEP | social `Seeking{Agent}` targets the partner tile | `new`+reg, boot | engine behaviour | — |
| harness_settlement_food_stock.rs | KEEP | settlement-snapshot `food_stock` | `new`+reg, boot | kept collector | — |
| harness_settlement_marker_fixed.rs | KEEP | `formation_tile` fixed marker | stage1, `new`+reg, boot | kept collector and SettlementSystem | — |
| harness_settlement_migration_unfreeze.rs | KEEP | migration mass-freeze regression | `new`+reg, boot | engine behaviour | — |
| harness_settlements_zero_regression.rs | KEEP | settlement-snapshot count vs live settlements | `new`+reg, boot | kept collector | — |
| harness_show_agent_needs_bars.rs | KEEP-TRIM | agent-snapshot hunger/thirst/sleep | stage1, `new`+reg | the fields are read by `agent_inspector_panel.gd` | delete (T): `harness_needs_a5_renderer_and_bridge_contract`, `harness_needs_a6_scene_wiring_additive` (`need_bar_renderer.gd` is deleted) |
| harness_show_death_visual.rs | KEEP-TRIM | `recent_deaths` buffer, `collect_recent_deaths` | `new` | `get_recent_deaths` is kept (`building_renderer.gd:428`) | delete (T): `harness_death_a5_renderer_structural_wiring`, `harness_death_a6_scene_wiring_additive`, `harness_death_a7_ffi_func_presence_and_forwarding`, `harness_death_a15_cross_language_retain_const_sync`; adapt: `harness_death_a10_current_tick_scalar_is_live` (remove part (b), the `world_node.rs` grep), `harness_death_a12_empty_then_single_fixture_collector` (remove the `death_viz_renderer.gd` guard check) |
| harness_show_resource_depletion.rs | KEEP | resource-snapshot `amount` / `max` | prod, `new` | kept collector (`amounts`/`maxes` read by `building_renderer.gd`) | — |
| harness_starvation_death.rs | KEEP | StarvationSystem, `despawn_agent`, roster cleanup | `new`+reg, boot | engine behaviour | — |
| harness_stockpile_consume.rs | KEEP | StockpileConsumeSystem famine relief | `new`+reg, boot | engine behaviour | — |
| harness_stockpile_feed_2_5c.rs | DELETE | HUD feed locale keys, `food_stocks` parity | module-level `include_str!` of `localization/compiled/*.json` and `key_registry.json` (L42-44) | locale text; a11 duplicates `harness_settlement_food_stock` a4 | all |
| harness_t7_10_a_warmth_wiring.rs | KEEP | Warmth BFS channel | `new`+ph2 | engine behaviour | — |
| harness_t7_10_b_light_shadowcast_wiring.rs | KEEP | Light shadowcast channel | `new`+ph2 | engine behaviour | — |
| harness_t7_10_b1_light_viz_toggle.rs | KEEP-TRIM | concurrent Warmth/Light channel validity | `new`+ph2 | engine behaviour | delete (T): `harness_t7_10_b1_gdscript_toggle_handler_present` |
| harness_t7_10_b1_space_toggle.rs | DELETE | SPACE overlay toggle in `world_renderer.gd` | — (T) | UI source text | all |
| harness_t7_10_c_noise_linear_wiring.rs | KEEP | Noise channel | `new`+ph2 | engine behaviour | — |
| harness_t7_10_d_danger_linear_wiring.rs | KEEP | Danger channel | `new`+ph2 | engine behaviour | — |
| harness_t7_10_e_spiritual_bfs_wiring.rs | KEEP-TRIM | Spiritual channel | `new`+ph2 | engine behaviour | delete (T): `harness_t7_10_e_spiritual_decay_constant_matches_spec` (`update.rs` text), `harness_t7_10_e_stale_constant_scan_regression_guard` |
| harness_t7_10_f_beauty_bfs_wiring.rs | KEEP | Beauty channel | `new`+ph2 | engine behaviour | — |
| harness_t7_9_a_scaffold.rs | KEEP-TRIM | engine tick count | `new`+ph2 | engine behaviour | keep a1–a3; delete (T): `harness_scaffold_world_node_source_process_identity`, `harness_scaffold_main_tscn_source_identity`, `harness_scaffold_gdextension_source_identity`, `harness_scaffold_world_renderer_gd_source_identity` |
| harness_t7_9_b_render_mechanism.rs | DELETE | world_node accumulator and renderer path | — (T: `world_node.rs`, `world_renderer.gd`) | source text only | all |

**Support modules (adapt together with their users):**

- `pass17_family_support/mod.rs`:
  - L13: take temperament from `WorldRules::baseline(7)` instead of `game_v20(7)`
    (exact).
  - L11: replace `Some(WorldRules::V22_FAMILY)` with the baseline family (V23). This
    one needs re-verification.
- `pass17_routine_support/mod.rs` L14: take the routine from `WorldRules::baseline(7)`
  (V24, with `end_stagger`) instead of `game_v21(7)`. This needs re-verification.

### Counts

| verdict | files |
|---|---|
| KEEP | 51 |
| KEEP-TRIM | 19 |
| KEEP-ADAPT | 38 |
| DELETE | 43 |
| **total** | **151** |

- **Revitalize files** (57): 38 KEEP-ADAPT and 19 DELETE (the 14 frozen files, plus
  `legacy_pin`, `pass7_interventions`, `pass10_forks`, `pass13_world` and
  `screen_contract`).
- **Other files** (94): 51 KEEP, 19 KEEP-TRIM and 24 DELETE.

## 3. Coverage table (live systems → tests that run them with their logic active)

"Run" means the system ticks with its rule present, either directly
(`System::new().tick(…)`) or through a registry plus `engine.tick()`, and the test
asserts on its output. Under default rules, the 8 rules-gated systems tick but return
at once, so V7 default-rule files do not count for them.

Every file with `+reg` also ticks all 28 systems. `harness_revitalize_pass21_relations`
(`harness_p21_game_v26_invariants_and_twin`) ticks all 28 under live rules for 60 days,
and asserts only relations, fission ≥ 1 and tally-vs-chronicle.

Suffixes: (T) = KEEP-TRIM, (A) = KEEP-ADAPT, R3 = coverage exists only after the scene
port.

| # | system | KEEP / KEEP-TRIM / KEEP-ADAPT files exercising it |
|---|---|---|
| 1 | BuildingStampSystem | phase2, phase2_ffi, phase2_substantial, phase2_determinism, next_a_postverify, p3_alpha_event_recording, p3_beta_causal_chain, p3_gamma_1_ffi, t7_10_a/b/c/d/f, t7_10_e (T), t7_10_b1_light_viz_toggle (T), p10_beta (a25/a26), settlement_marker_fixed, settlements_zero_regression |
| 2 | InfluenceUpdateSystem | phase2, phase2_ffi, phase2_substantial, phase2_determinism, next_a_postverify, t7_10_a/b/c/d/f, t7_10_e (T), t7_10_b1_light_viz_toggle (T), p3_beta_causal_chain, p3_gamma_1_ffi |
| 3 | AgentInfluenceSampleSystem | phase2 (`harness_agent_sample_*`), phase2_substantial (AIS warmth, 1k agents), p4_alpha_agent_core (`…agent_sample_runtime_integration`), p4_beta_movement (`…influence_sampler_still_reads_canonical_position`) |
| 4 | AgentMovementSystem | p4_beta_movement, p4_gamma_rendering (d8), s16_beta_movement, newborn_mobile, p10_gamma_settlement_migration, s16_zeta_social_freeze_fix, revitalize_loop (A, movement cadence), revitalize_pass17_routine (A), revitalize_pass6_elders (A), revitalize_pass20_relations (A) |
| 5 | AgentDecisionSystem | p5_beta_decision, p5_gamma_sleep_daynight_chronicle, p6_beta_construction_system, p7_beta_social_system, p8_beta_memory_system (T), p9_beta_combat_system, s16_alpha_seektarget, s16_beta_movement, s16_zeta_social_freeze_fix, resource_gather_pickup, resource_scarcity, fix_d_memory_arm, a15_controlled_determinism (T), p10_gamma_settlement_migration, settlement_migration_unfreeze; (A): revitalize_survival_fix, pass3_rhythm, pass4_forage, pass5_meals, pass5_preempt, pass6_urgent, pass17_routine*, pass21_relations |
| 6 | StaleSeekTargetSystem | resource_scarcity (a22 `seeking_reroutes…`, a25 `seeking_exits…`), the only one; fix_d_memory_arm and a15 (T) run it in scarcity scenes without targeting it |
| 7 | HungerDecaySystem | p5_alpha_agent_id_hunger, p5_beta_decision, s16_delta_need_stagger, starvation_death, stockpile_consume |
| 8 | ThirstDecaySystem | p5_beta_decision, s16_delta_need_stagger, starvation_death (a4 dehydration) |
| 9 | SleepDecaySystem | p5_gamma_sleep_daynight_chronicle, s16_delta_need_stagger, p6_beta_construction_system |
| 10 | ConstructionSystem | p6_beta_construction_system, p6_gamma_construction_chronicle, p7_beta_social_system (a7), p8_beta_memory_system (T; a5, a7, a8), p10_beta (a26); (A): revitalize_pass3_village, revitalize_loop |
| 11 | SocialInteractionSystem | p7_beta_social_system, p7_gamma_social_chronicle, p7_delta_social_ui (T, end-to-end fns), s16_zeta_social_freeze_fix, p8_beta (T; a9, a10), p9_gamma_combat_chronicle; (A): pass17_routine_invitations |
| 12 | SocialDecaySystem | p7_beta_social_system (a13b loneliness), p7_gamma_social_chronicle |
| 13 | MemorySystem | p8_beta_memory_system (T), p8_gamma_memory_chronicle, fix_d_memory_arm, p9_beta_combat_system (a20, a21), p9_gamma_combat_chronicle, p8_delta_memory_ui (T), p10_beta (a20) |
| 14 | CombatSystem | p9_beta_combat_system, p9_gamma_combat_chronicle, starvation_death (a10), show_death_visual (T; a9), p10_gamma_settlement_chronicle |
| 15 | SettlementSystem | p10_beta, p10_gamma_settlement_chronicle, p10_gamma_settlement_migration, membership_belonging, newborn_mobile, settlement_marker_fixed, settlements_zero_regression, settlement_migration_unfreeze, resource_deposit_to_stockpile, settlement_food_stock, stockpile_consume, s16_zeta (a9 births); (A): pass3_food, pass3_village, pass17_family_births, pass19_chronicle_events, revitalize_loop |
| 16 | StarvationSystem | starvation_death, show_death_visual (T), resource_scarcity (a6, a12, a24), stockpile_consume; (A): revitalize_tallies |
| 17 | ResourceRegenSystem | resource_scarcity (a4, a5, a14, a18, a19, a21, a26, a27, bounded trickle), show_resource_depletion (a10); (A): pass4_seasons, pass6_winter, pass7_winter, pass8_winter, pass12_world |
| 18 | StockpileDepositSystem | resource_deposit_to_stockpile; (A): pass5_firewood, pass20_relations |
| 19 | StockpileConsumeSystem | stockpile_consume; (A): pass3_food, pass5_meals |
| 20 | ShelterPlanningSystem | **no KEEP or KEEP-TRIM test**; (A): pass3_village (`plans_with_headroom`, `completed_site_is_owned`), revitalize_loop (`assigned_builder…`; R3 `shelters_planned_and_completed`), pass17_routine_edges (`builders_are_recruited…`) |
| 21 | MaturationSystem | **no KEEP or KEEP-TRIM test**; (A): only pass3_rhythm `children_grow_up` (R3), and pass17_family `family_system_registration_follows_maturation_before_beds_once` (registry order only, no tick). **Effectively uncovered until the R3 port lands.** |
| 22 | PairingSystem | **no KEEP or KEEP-TRIM test**; (A): pass17_family (4 pairing/household fns via the support fixture) |
| 23 | BedAssignmentSystem | **no KEEP or KEEP-TRIM test**; (A): pass17_family_beds, pass3_rhythm (`bed_seek_ignores…`, `reassigned_bed…`; R3 `sleep_in_beds_at_night`), pass4_aging (R3 bed cleanup in `old_age_death…`) |
| 24 | AgingSystem | **no KEEP or KEEP-TRIM test**; (A): only pass4_aging R3 ports (`founders`, `old_age_death_and_replacement`, `elders_stay_home`); `lifespans_bounded…` tests the pure `newborn_life` fn, not the system tick. **Effectively uncovered until the R3 port lands.** |
| 25 | FireSystem | **no KEEP or KEEP-TRIM test**; (A): pass5_firewood, pass6_winter, pass7_winter, pass8_winter, pass10_exposure, pass11_exposure, pass6_urgent |
| 26 | FissionSystem | **no KEEP or KEEP-TRIM test**; (A): pass11_fission, pass12_fission, pass13_fission, pass17_family_fission, pass19_chronicle_events, pass21_relations (fissions ≥ 1 on the 60-day run) |
| 27 | RelationsSystem | **no KEEP or KEEP-TRIM test**; (A): pass21_relations, pass20_relations |
| 28 | InfluenceVisualizationSystem | phase2 (`harness_viz_*`), phase2_determinism (a5 viz digest), p6_beta_construction_system, p7_beta_social_system |

Systems with no test once the adapted files are counted: **none**. The coverage is
still uneven:

- **No KEEP or KEEP-TRIM test** (only adapted files): ShelterPlanningSystem,
  MaturationSystem, PairingSystem, BedAssignmentSystem, AgingSystem, FireSystem,
  FissionSystem, RelationsSystem. These are exactly the 8 rules-gated systems, which
  default rules switch off.
- **Covered only by a scene port (R3) that is not yet written**: MaturationSystem
  (`pass3_rhythm::harness_rhythm_children_grow_up`) and the AgingSystem tick
  (`pass4_aging::harness_aging_old_age_death_and_replacement`). If those ports fail
  re-verification and are dropped, GOAL 8.4 requires a minimal new test for each.
- **Single-file coverage**: StaleSeekTargetSystem (resource_scarcity only),
  StockpileDepositSystem and StockpileConsumeSystem (one KEEP file each).

## 4. Unit tests inside `rust/crates/sim-test/src/**` in deleted modules

| module | tests |
|---|---|
| `worldmodel/digest.rs` (2) | `fnv_matches_known_vector`, `digest_sees_what_the_fingerprint_misses` |
| `worldmodel/export.rs` (7) | `first_row_is_the_fresh_engine_and_shapes_match_the_fields`, `schema_two_hides_the_weather_and_lists_observable_fields`, `schema_five_records_hardiness_regions_and_parent_slots`, `schema_four_records_the_hidden_world_variety_of_game_v16`, `schedules_are_recorded_and_change_the_rows_after_their_tick`, `resume_keeps_only_the_requested_episode`, `arguments_validate` |
| `worldmodel/npy.rs` (2) | `header_bytes_are_exact_and_aligned`, `data_follows_the_header_little_endian` |
| `worldmodel/schedule.rs` (2) | `parses_rows_and_rejects_bad_schedules`, `due_rows_apply_in_file_order_at_their_tick` |
| `worldmodel/sha256.rs` (1) | `matches_the_standard_test_vectors` |
| `bin/worldsim_wm.rs` (1) | `cli_rejects_unknown_flags` |
| `probe/ablation.rs` (3) | `each_scene_differs_from_its_base_in_one_place`, `each_v18_scene_differs_from_v18_nofission_in_one_place`, `reverting_exposure_gives_v16_exposure` |
| `probe/pass4.rs` (1) | `v9_world_reports_neutral_pass4_values` |
| `probe/pass5.rs` (2) | `v10_world_reports_neutral_pass5_values`, `v11_world_reports_its_deadwood` |
| `probe/pass6.rs` (2) | `v11_world_reports_neutral_pass6_values`, `gatherers_elders_and_urgent_have_exact_values` |
| `probe/pass7.rs` (2) | `v12_world_reports_neutral_pass7_values`, `rain_and_cold_snap_and_tallies_have_exact_values` |
| `probe/pass8.rs` (2) | `v13_world_reports_neutral_pass8_values_outside_winter`, `winter_fall_and_rules_block_read_the_rule` |
| `probe/pass10.rs` (2) | `v15_world_reports_full_health_and_no_variety`, `seed_winter_windows_count_deaths_and_dark_nights` |
| `probe/pass12.rs` (1) | `work_classes_add_up_to_the_work_share_numerator` |
| `probe/pass17_tests.rs` (9) | `completed_minutes_equal_consuming_intervals_when_walking_precedes_arrival` … `twin_comparison_detects_routine_timers_without_changing_fingerprint` |
| `probe/pass17_batch_tests.rs` (4) | `transient_start_and_end_preserve_redraw_without_fabricating_consumption` … `twin_comparison_detects_batch_difference_with_same_latest_event` |
| `probe/pass17_death_tests.rs` (7) | `start_and_redraw_survive_same_tick_old_age_despawn` … `observer_is_absent_from_old_worlds_and_checks_tick_and_single_drain` |
| `probe/pass20.rs` (1) | `only_game_v25_has_the_block` |
| `probe/pass21.rs` (1) | `only_capped_rules_have_the_keys` |

`probe/pass11.rs`, `pass13.rs`, `pass17.rs` and `pass17_activity.rs` have no tests.
Total: **52** unit tests in deleted modules.

**Kept modules whose tests depend on deleted fixtures (adapt or delete with the fixture):**

| file | test | deleted dependency |
|---|---|---|
| `lib.rs` | `shared_scene_starts_at_tick_zero` | `legacy_scene` |
| `scenes.rs` | `game_scene_preserves_seed_and_starts_with_twenty_four_agents` | `game_scene` |
| `scenes.rs` | `legacy_boot_buildings_are_queued_without_advancing_time` | `legacy_scene` |
| `perf.rs` | `profiled_and_plain_game_v19_reach_the_same_fingerprint` | game_v19 stress (re-pointed in 8.1) |
| `perf.rs` | `rules_flag_picks_the_stress_rules_and_label` | game_v19 stress (re-pointed in 8.1) |
| `probe/runner.rs` | `both_v16_scenes_take_both_twin_checks` | v16 scenes |
| `probe/fingerprint.rs` | `independent_legacy_engines_match` | the legacy scene (verify) |
| `probe/metrics.rs` | `idle_actionable_uses_night_threshold_and_preserves_v8` | `game_v8` (verify) |
