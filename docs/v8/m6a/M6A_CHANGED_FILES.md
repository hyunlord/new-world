# M6a 커밋별 변경 파일 목록

비교 BASE: `5b0c88812374b87e7854cc9203f144ad0ccd9009`. 이 목록의 마지막 구현·도구 커밋: `bd799460c25a7019dd5d3f30296587c96674615b`.

목표 §4의 커밋별 파일 목록이다. 각 블록은 해당 커밋의 `git show --format= --name-status --no-renames` 출력을 그대로 기록한다. A는 추가, M은 수정, D는 삭제이며, 이름 변경도 원래 경로 삭제와 새 경로 추가로 명시한다. 같은 파일이 여러 커밋에서 바뀌면 각 커밋에 반복한다. 이 문서 자체와 이후 문서 마감 커밋은 아래 고정된 범위에 포함되지 않는다. 파일 변경 목록은 검증·수용·게시 성공을 의미하지 않는다.

## v8-m6a(p0): Preserve the approved food-chain contract before implementation

커밋: `a3106c9d727ef10dd6f29cb2a5fe5d493e582565` · 변경 경로 1개

```text
A	docs/v8/m6a/M6A_GOAL.md
```

## v8-m6a(p1): Preserve the approved depth sequence and pass boundaries

커밋: `cb3f79cc69df10ee05b10d5a5330e0226f67fc68` · 변경 경로 6개

```text
M	.claude/skills/worldsim-docs-upkeep/SKILL.md
M	AGENTS.md
M	docs/HISTORY.md
M	docs/ROADMAP.md
M	docs/VISION.md
M	docs/v8/DEPTH_ROADMAP.md
```

## v8-m6a(p2): Bound the food economy before changing colony behavior

커밋: `31221e26e5a41df4a1c1427f607d535d8fd16169` · 변경 경로 3개

```text
A	docs/decisions/0052-item-food-chain.md
A	docs/v8/m6a/M6A_DECISIONS.md
A	docs/v8/m6a/M6A_DESIGN.md
```

## v8-m6a(p3): make colony food physical before survival calibration

커밋: `dba478e54ea2b6b21aace3c32ae2ab64de10235b` · 변경 경로 293개

```text
M	.claude/skills/worldsim-pitfalls/SKILL.md
M	CHANGELOG.md
M	README.md
M	assets/ui/alerts.json
M	assets/ui/chronicle_feed.json
A	assets/ui/food_chronicle.json
M	assets/v8art/bindings.json
M	assets/v8art/render_params.json
M	docs/ARCHITECTURE.md
M	docs/ISSUES.md
M	docs/decisions/0052-item-food-chain.md
A	docs/v8/bridge_api_m6a.txt
M	docs/v8/core/MIND_AUTHORING.md
M	docs/v8/core/SYSTEM_WORLDS.md
M	docs/v8/m6a/M6A_DECISIONS.md
A	docs/v8/m6a/M6A_REPORT.md
M	docs/v8/snapshot_schema.md
M	localization/compiled/en.json
M	localization/compiled/ko.json
M	localization/en/ui.json
M	localization/key_registry.json
M	localization/ko/ui.json
M	rust/crates/sim-bridge/src/ffi/content.rs
A	rust/crates/sim-bridge/src/ffi/food/chronicle.rs
A	rust/crates/sim-bridge/src/ffi/food/marshal.rs
A	rust/crates/sim-bridge/src/ffi/food/mod.rs
A	rust/crates/sim-bridge/src/ffi/food/person.rs
A	rust/crates/sim-bridge/src/ffi/food/plants.rs
A	rust/crates/sim-bridge/src/ffi/food/stacks.rs
A	rust/crates/sim-bridge/src/ffi/food/summary.rs
M	rust/crates/sim-bridge/src/ffi/knowledge.rs
M	rust/crates/sim-bridge/src/ffi/mod.rs
M	rust/crates/sim-bridge/src/ffi/node.rs
M	rust/crates/sim-bridge/src/ffi/reads_settlement.rs
M	rust/crates/sim-bridge/src/ffi/snapshot.rs
A	rust/crates/sim-bridge/src/ffi/snapshot/food_tests.rs
M	rust/crates/sim-bridge/src/ffi/ui_facts/marshal.rs
M	rust/crates/sim-bridge/src/ffi/ui_facts/mod.rs
M	rust/crates/sim-core/data/colony/blueprints.ron
M	rust/crates/sim-core/data/colony/items.ron
M	rust/crates/sim-core/data/colony/knowledge.ron
M	rust/crates/sim-core/data/colony/mind.ron
A	rust/crates/sim-core/data/colony/plants.ron
M	rust/crates/sim-core/data/colony/recipes.ron
M	rust/crates/sim-core/data/colony/steward.ron
M	rust/crates/sim-core/src/chronicle.rs
M	rust/crates/sim-core/src/colony/command.rs
A	rust/crates/sim-core/src/colony/content/defs/food.rs
M	rust/crates/sim-core/src/colony/content/defs/item.rs
M	rust/crates/sim-core/src/colony/content/defs/mod.rs
A	rust/crates/sim-core/src/colony/content/defs/plant.rs
M	rust/crates/sim-core/src/colony/content/defs/recipe.rs
M	rust/crates/sim-core/src/colony/content/host.rs
M	rust/crates/sim-core/src/colony/content/ids.rs
M	rust/crates/sim-core/src/colony/content/load.rs
A	rust/crates/sim-core/src/colony/content/load_plants.rs
M	rust/crates/sim-core/src/colony/content/mod.rs
M	rust/crates/sim-core/src/colony/content/raw_item.rs
M	rust/crates/sim-core/src/colony/content/raw_recipe.rs
M	rust/crates/sim-core/src/colony/content/resolve_blueprints.rs
M	rust/crates/sim-core/src/colony/content/resolve_items.rs
M	rust/crates/sim-core/src/colony/content/resolve_recipes.rs
M	rust/crates/sim-core/src/colony/content/sources.rs
M	rust/crates/sim-core/src/colony/content/tests.rs
M	rust/crates/sim-core/src/colony/content/validate.rs
A	rust/crates/sim-core/src/colony/food.rs
M	rust/crates/sim-core/src/colony/items.rs
M	rust/crates/sim-core/src/colony/job.rs
M	rust/crates/sim-core/src/colony/knowledge/parse.rs
M	rust/crates/sim-core/src/colony/knowledge/registry_tests.rs
M	rust/crates/sim-core/src/colony/knowledge/tests.rs
M	rust/crates/sim-core/src/colony/mod.rs
M	rust/crates/sim-core/src/colony/orders.rs
M	rust/crates/sim-core/src/colony/people.rs
A	rust/crates/sim-core/src/colony/steward/food_plan.rs
M	rust/crates/sim-core/src/colony/steward/mod.rs
M	rust/crates/sim-core/src/colony/steward/rules.rs
M	rust/crates/sim-core/src/colony/workshop.rs
M	rust/crates/sim-core/src/rng.rs
M	rust/crates/sim-core/src/spatial.rs
A	rust/crates/sim-core/tests/m6a_content.rs
M	rust/crates/sim-core/tests/recipe_registry.rs
A	rust/crates/sim-engine/src/food_supply_cache.rs
M	rust/crates/sim-engine/src/legacy_v7.rs
M	rust/crates/sim-engine/src/lib.rs
M	rust/crates/sim-engine/src/registration.rs
M	rust/crates/sim-systems/src/commands.rs
M	rust/crates/sim-systems/src/lib.rs
M	rust/crates/sim-systems/src/runtime/agent/movement.rs
M	rust/crates/sim-systems/src/runtime/agent_actions/consume.rs
M	rust/crates/sim-systems/src/runtime/agent_actions/needs.rs
A	rust/crates/sim-systems/src/runtime/food/choice.rs
A	rust/crates/sim-systems/src/runtime/food/drying.rs
A	rust/crates/sim-systems/src/runtime/food/events.rs
A	rust/crates/sim-systems/src/runtime/food/ground.rs
A	rust/crates/sim-systems/src/runtime/food/items.rs
A	rust/crates/sim-systems/src/runtime/food/loss.rs
A	rust/crates/sim-systems/src/runtime/food/meals.rs
A	rust/crates/sim-systems/src/runtime/food/migration.rs
A	rust/crates/sim-systems/src/runtime/food/migration/physical.rs
A	rust/crates/sim-systems/src/runtime/food/migration/porters.rs
A	rust/crates/sim-systems/src/runtime/food/migration/reserves.rs
A	rust/crates/sim-systems/src/runtime/food/mod.rs
A	rust/crates/sim-systems/src/runtime/food/owners.rs
A	rust/crates/sim-systems/src/runtime/food/placement.rs
A	rust/crates/sim-systems/src/runtime/food/plants.rs
A	rust/crates/sim-systems/src/runtime/food/pots.rs
A	rust/crates/sim-systems/src/runtime/food/reserves.rs
A	rust/crates/sim-systems/src/runtime/food/storage.rs
A	rust/crates/sim-systems/src/runtime/food/validation.rs
M	rust/crates/sim-systems/src/runtime/jobs/assign.rs
M	rust/crates/sim-systems/src/runtime/jobs/bills.rs
M	rust/crates/sim-systems/src/runtime/jobs/cancel.rs
M	rust/crates/sim-systems/src/runtime/jobs/craft.rs
A	rust/crates/sim-systems/src/runtime/jobs/craft_food.rs
M	rust/crates/sim-systems/src/runtime/jobs/execute.rs
A	rust/crates/sim-systems/src/runtime/jobs/food_admission_tests.rs
A	rust/crates/sim-systems/src/runtime/jobs/food_job_tests.rs
A	rust/crates/sim-systems/src/runtime/jobs/food_transfers.rs
A	rust/crates/sim-systems/src/runtime/jobs/harvest.rs
A	rust/crates/sim-systems/src/runtime/jobs/harvest_encoding_tests.rs
M	rust/crates/sim-systems/src/runtime/jobs/hunt.rs
A	rust/crates/sim-systems/src/runtime/jobs/hunt_food_loss_tests.rs
M	rust/crates/sim-systems/src/runtime/jobs/invariants.rs
M	rust/crates/sim-systems/src/runtime/jobs/mod.rs
A	rust/crates/sim-systems/src/runtime/jobs/pot_ingredient_tests.rs
A	rust/crates/sim-systems/src/runtime/jobs/pot_supply.rs
A	rust/crates/sim-systems/src/runtime/jobs/pot_supply_tests.rs
M	rust/crates/sim-systems/src/runtime/jobs/teach.rs
M	rust/crates/sim-systems/src/runtime/mind/behaviours/context.rs
A	rust/crates/sim-systems/src/runtime/mind/behaviours/food.rs
M	rust/crates/sim-systems/src/runtime/mind/behaviours/gather.rs
A	rust/crates/sim-systems/src/runtime/mind/behaviours/harvest_food.rs
M	rust/crates/sim-systems/src/runtime/mind/behaviours/mod.rs
M	rust/crates/sim-systems/src/runtime/mind/behaviours/needs.rs
M	rust/crates/sim-systems/src/runtime/mind/behaviours/owned.rs
M	rust/crates/sim-systems/src/runtime/mind/behaviours/postpass.rs
M	rust/crates/sim-systems/src/runtime/mind/behaviours/prepare.rs
M	rust/crates/sim-systems/src/runtime/mind/behaviours/routine.rs
M	rust/crates/sim-systems/src/runtime/mind/behaviours/social.rs
M	rust/crates/sim-systems/src/runtime/mind/behaviours/tests.rs
M	rust/crates/sim-systems/src/runtime/mind/context.rs
M	rust/crates/sim-systems/src/runtime/mind/context/dependency_sum_tests.rs
M	rust/crates/sim-systems/src/runtime/mind/context/overlay_slot_tests.rs
M	rust/crates/sim-systems/src/runtime/mind/continuation_runtime_tests.rs
M	rust/crates/sim-systems/src/runtime/mind/facts.rs
A	rust/crates/sim-systems/src/runtime/mind/food_targets.rs
A	rust/crates/sim-systems/src/runtime/mind/harvest_food.rs
A	rust/crates/sim-systems/src/runtime/mind/harvest_food_cache_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/harvest_food_tests.rs
M	rust/crates/sim-systems/src/runtime/mind/job_origins.rs
M	rust/crates/sim-systems/src/runtime/mind/mod.rs
A	rust/crates/sim-systems/src/runtime/mind/physical_food_fact_tests.rs
M	rust/crates/sim-systems/src/runtime/mind/primary_refresh_system_tests.rs
M	rust/crates/sim-systems/src/runtime/mind/system.rs
M	rust/crates/sim-systems/src/runtime/mind/system_owner_tests.rs
M	rust/crates/sim-systems/src/runtime/mod.rs
M	rust/crates/sim-systems/src/runtime/mood/mod.rs
M	rust/crates/sim-systems/src/runtime/neighbours/mod.rs
M	rust/crates/sim-systems/src/runtime/neighbours/trade.rs
M	rust/crates/sim-systems/src/runtime/resource_regen/mod.rs
M	rust/crates/sim-systems/src/runtime/settlement/fission.rs
A	rust/crates/sim-systems/src/runtime/settlement/fission/migration.rs
M	rust/crates/sim-systems/src/runtime/settlement/food_economy.rs
A	rust/crates/sim-systems/src/runtime/settlement/food_supply.rs
M	rust/crates/sim-systems/src/runtime/settlement/relations.rs
M	rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs
M	rust/crates/sim-systems/src/runtime/settlement/shelter_planning.rs
M	rust/crates/sim-systems/src/runtime/settlement/stockpile_consume.rs
M	rust/crates/sim-systems/src/runtime/settlement/stockpile_deposit.rs
A	rust/crates/sim-systems/src/runtime/steward/food.rs
A	rust/crates/sim-systems/src/runtime/steward/food/cooking.rs
A	rust/crates/sim-systems/src/runtime/steward/food/cooking_bill_tests.rs
A	rust/crates/sim-systems/src/runtime/steward/food/dependencies.rs
A	rust/crates/sim-systems/src/runtime/steward/food/harvest.rs
A	rust/crates/sim-systems/src/runtime/steward/food/observe.rs
A	rust/crates/sim-systems/src/runtime/steward/food/storage.rs
A	rust/crates/sim-systems/src/runtime/steward/food/storage_tests.rs
A	rust/crates/sim-systems/src/runtime/steward/food/summary.rs
M	rust/crates/sim-systems/src/runtime/steward/lifecycle.rs
M	rust/crates/sim-systems/src/runtime/steward/mod.rs
M	rust/crates/sim-systems/src/runtime/steward/planning.rs
M	rust/crates/sim-systems/src/runtime/steward/production.rs
M	rust/crates/sim-systems/src/runtime/steward/production_optimization_tests.rs
M	rust/crates/sim-systems/src/runtime/survival/mod.rs
M	rust/crates/sim-systems/src/runtime/wildlife/mod.rs
A	rust/crates/sim-systems/src/save/food_catalog.rs
A	rust/crates/sim-systems/src/save/food_catalog_ids.rs
A	rust/crates/sim-systems/src/save/food_catalog_v12.json
A	rust/crates/sim-systems/src/save/food_tests.rs
M	rust/crates/sim-systems/src/save/mind_tests.rs
M	rust/crates/sim-systems/src/save/mod.rs
M	rust/crates/sim-systems/src/worldgen/mod.rs
A	rust/crates/sim-systems/src/worldgen/plants.rs
A	rust/crates/sim-test/data/m6a_apple/items.ron
A	rust/crates/sim-test/data/m6a_apple/pack.ron
A	rust/crates/sim-test/data/m6a_apple/plants.ron
M	rust/crates/sim-test/src/bench.rs
M	rust/crates/sim-test/src/bin/worldsim_probe.rs
M	rust/crates/sim-test/src/lib.rs
A	rust/crates/sim-test/src/m6a/cli.rs
A	rust/crates/sim-test/src/m6a/data_only.rs
A	rust/crates/sim-test/src/m6a/metrics.rs
A	rust/crates/sim-test/src/m6a/mod.rs
A	rust/crates/sim-test/src/m6a/observer.rs
M	rust/crates/sim-test/src/nav_bench.rs
M	rust/crates/sim-test/src/perf.rs
A	rust/crates/sim-test/src/perf_food_tests.rs
M	rust/crates/sim-test/src/probe/colony.rs
M	rust/crates/sim-test/src/probe/fingerprint.rs
A	rust/crates/sim-test/src/probe/food_fingerprint.rs
A	rust/crates/sim-test/src/probe/food_fingerprint_tests.rs
M	rust/crates/sim-test/src/probe/mod.rs
M	rust/crates/sim-test/tests/b4_haul.rs
M	rust/crates/sim-test/tests/b4_haul/boundaries.rs
M	rust/crates/sim-test/tests/b4_haul/continuity.rs
M	rust/crates/sim-test/tests/colony_hides.rs
M	rust/crates/sim-test/tests/colony_jobs.rs
M	rust/crates/sim-test/tests/colony_knowledge.rs
M	rust/crates/sim-test/tests/colony_neighbours.rs
M	rust/crates/sim-test/tests/colony_steward.rs
M	rust/crates/sim-test/tests/colony_steward_housing/construction.rs
M	rust/crates/sim-test/tests/colony_story.rs
M	rust/crates/sim-test/tests/colony_wildlife.rs
M	rust/crates/sim-test/tests/colony_world.rs
M	rust/crates/sim-test/tests/engine_economy_ffi.rs
M	rust/crates/sim-test/tests/engine_ui_facts.rs
A	rust/crates/sim-test/tests/m6a_food_data_only.rs
A	rust/crates/sim-test/tests/m6a_food_economy.rs
A	rust/crates/sim-test/tests/m6a_food_events.rs
A	rust/crates/sim-test/tests/m6a_food_migration.rs
A	rust/crates/sim-test/tests/m6a_food_mind.rs
A	rust/crates/sim-test/tests/m6a_food_save.rs
A	rust/crates/sim-test/tests/m6a_food_steward.rs
A	rust/crates/sim-test/tests/m6a_food_storage.rs
A	rust/crates/sim-test/tests/m6a_legacy_food_retirement.rs
A	rust/crates/sim-test/tests/m6a_legacy_supply.rs
A	rust/crates/sim-test/tests/m6a_needs_preemption.rs
A	rust/crates/sim-test/tests/m6a_pending_food.rs
A	rust/crates/sim-test/tests/m6a_plants.rs
A	rust/crates/sim-test/tests/m6a_stale_food.rs
A	rust/crates/sim-test/tests/m6a_winter_plan_evidence.rs
M	rust/crates/sim-test/tests/save_load.rs
M	scenes/main.tscn
M	scripts/core/sim_reads.gd
M	scripts/ui/chronicle_text.gd
M	scripts/ui/content_context.gd
A	scripts/ui/food_text.gd
A	scripts/ui/food_text.gd.uid
M	scripts/ui/hud/order_bar.gd
M	scripts/ui/mux/alert_evaluator.gd
M	scripts/ui/mux/alerts_panel.gd
M	scripts/ui/mux/ui_facts_adapter.gd
M	scripts/ui/panels/agent_inspector_panel.gd
M	scripts/ui/panels/causal_panel.gd
M	scripts/ui/panels/chronicle_panel.gd
M	scripts/ui/panels/item_panel.gd
M	scripts/ui/panels/room_panel.gd
M	scripts/ui/panels/workshop_panel.gd
M	scripts/ui/story_text.gd
M	scripts/world/art/art_data.gd
M	scripts/world/colony/order_tool.gd
A	scripts/world/colony/plant_layer.gd
A	scripts/world/colony/plant_layer.gd.uid
M	tools/v8/core_gate_checks.sh
A	tools/v8/fixtures/m6a_plant_art.json
M	tools/v8/gate.sh
A	tools/v8/m6a_causal.py
A	tools/v8/m6a_data_only.py
A	tools/v8/m6a_evidence.py
A	tools/v8/m6a_gate.py
A	tools/v8/m6a_gate_art.py
A	tools/v8/m6a_gate_checks.sh
A	tools/v8/m6a_gate_evidence.py
A	tools/v8/m6a_gate_history.py
A	tools/v8/m6a_gate_source.py
A	tools/v8/m6a_gate_story.py
A	tools/v8/m6a_perf.py
A	tools/v8/m6a_perf_config.py
A	tools/v8/m6a_perf_validate.py
A	tools/v8/m6a_plant_art_test.gd
A	tools/v8/m6a_plant_art_test.gd.uid
A	tools/v8/m6a_run.py
A	tools/v8/m6a_score.py
A	tools/v8/m6a_ui_capture.gd
M	tools/v8/test_bridge_api.py
M	tools/v8/test_core_mind_ui.gd
M	tools/v8/test_locale_trf.gd
A	tools/v8/test_m6a_causal.py
A	tools/v8/test_m6a_gate.py
A	tools/v8/test_m6a_perf.py
A	tools/v8/test_m6a_run.py
A	tools/v8/test_m6a_score.py
```

## v8-m6a(p4): let food work finish and preserve reserves through winter

커밋: `de25dd74ed894365b4a59318fa4318056cb71b6d` · 변경 경로 30개

```text
M	.claude/skills/worldsim-pitfalls/SKILL.md
M	CHANGELOG.md
A	docs/decisions/0053-food-action-and-winter-reserves.md
M	docs/v8/m6a/M6A_DECISIONS.md
M	docs/v8/m6a/M6A_REPORT.md
M	rust/crates/sim-core/data/colony/steward.ron
M	rust/crates/sim-core/src/colony/food.rs
M	rust/crates/sim-systems/src/runtime/food/loss.rs
M	rust/crates/sim-systems/src/runtime/food/validation.rs
A	rust/crates/sim-systems/src/runtime/jobs/cooking_heat_tests.rs
M	rust/crates/sim-systems/src/runtime/jobs/craft_food.rs
M	rust/crates/sim-systems/src/runtime/jobs/mod.rs
M	rust/crates/sim-systems/src/runtime/mind/behaviours/food.rs
A	rust/crates/sim-systems/src/runtime/mind/behaviours/food_invalidation_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/behaviours/food_retention_tests.rs
M	rust/crates/sim-systems/src/runtime/mind/behaviours/mod.rs
M	rust/crates/sim-systems/src/runtime/mind/behaviours/tests.rs
M	rust/crates/sim-systems/src/runtime/mind/food_targets.rs
M	rust/crates/sim-systems/src/runtime/settlement/fire.rs
M	rust/crates/sim-systems/src/runtime/steward/food.rs
M	rust/crates/sim-systems/src/runtime/steward/food/cooking.rs
M	rust/crates/sim-systems/src/runtime/steward/food/dependencies.rs
M	rust/crates/sim-systems/src/runtime/steward/food/harvest.rs
A	rust/crates/sim-systems/src/runtime/steward/food/horizon_tests.rs
A	rust/crates/sim-systems/src/runtime/steward/food/preservation.rs
A	rust/crates/sim-systems/src/runtime/steward/food/preservation_tests.rs
M	rust/crates/sim-systems/src/runtime/steward/food/summary.rs
M	rust/crates/sim-test/src/probe/food_fingerprint_tests.rs
M	rust/crates/sim-test/tests/m6a_food_save.rs
M	tools/v8/m6a_ui_capture.gd
```

## v8-m6a(p5): avoid repeated food-query work and duplicate policy storage

커밋: `9936a8fd26be15d171afccf4000c3ebf113326dc` · 변경 경로 17개

```text
M	.claude/skills/worldsim-pitfalls/SKILL.md
M	CHANGELOG.md
M	README.md
M	docs/HISTORY.md
M	docs/archive/INDEX.md
A	docs/images/m6a_contact_sheet.png
D	docs/images/mux_contact_sheet.png
M	docs/v8/m6a/M6A_DECISIONS.md
M	docs/v8/m6a/M6A_REPORT.md
M	docs/v8/mux/B5_VERIFICATION.md
M	rust/crates/sim-systems/src/lib.rs
M	rust/crates/sim-systems/src/runtime/food/choice.rs
A	rust/crates/sim-systems/src/runtime/food/choice_availability_tests.rs
M	rust/crates/sim-systems/src/runtime/jobs/assign.rs
M	rust/crates/sim-systems/src/runtime/mind/program.rs
A	rust/crates/sim-systems/src/runtime/mind/program_source_tests.rs
M	tools/v8/m6a_ui_capture.gd
```

## v8-m6a(p6): avoid repeated haul searches after destination exhaustion

커밋: `e156ae29b804038224ff8928895025f64742f478` · 변경 경로 6개

```text
M	.claude/skills/worldsim-pitfalls/SKILL.md
M	CHANGELOG.md
M	docs/v8/m6a/M6A_DECISIONS.md
M	docs/v8/m6a/M6A_REPORT.md
M	rust/crates/sim-systems/src/runtime/jobs/assign.rs
A	rust/crates/sim-systems/src/runtime/jobs/loose_haul_tests.rs
```

## v8-m6a(p7): reuse guards while authored predicate truth remains unchanged

커밋: `5857cdc4f89c24d4a4d5f2ad2f55703c937572c3` · 변경 경로 10개

```text
M	.claude/skills/worldsim-pitfalls/SKILL.md
M	CHANGELOG.md
M	docs/v8/m6a/M6A_DECISIONS.md
M	docs/v8/m6a/M6A_REPORT.md
M	rust/crates/sim-core/src/colony/mind/continuation.rs
M	rust/crates/sim-core/src/colony/mind/continuation_cache.rs
A	rust/crates/sim-core/src/colony/mind/continuation_cache_equivalence_tests.rs
M	rust/crates/sim-core/src/colony/mind/continuation_cache_scratch_tests.rs
A	rust/crates/sim-core/src/colony/mind/continuation_partitions.rs
M	rust/crates/sim-systems/src/runtime/mind/continuation_cache_runtime_tests.rs
```

## v8-m6a(p8): preserve observed food causes instead of inferring missing history

커밋: `e2ba280f345202b7f334b213ea2c13e2de828c46` · 변경 경로 50개

```text
M	.claude/skills/worldsim-godot/references/common-pitfalls.md
M	.claude/skills/worldsim-testing/SKILL.md
M	CHANGELOG.md
M	assets/ui/chronicle_feed.json
M	assets/ui/food_chronicle.json
M	docs/ARCHITECTURE.md
A	docs/decisions/0054-food-operation-evidence.md
M	docs/v8/m6a/M6A_DECISIONS.md
M	docs/v8/m6a/M6A_REPORT.md
M	docs/v8/snapshot_schema.md
M	localization/compiled/en.json
M	localization/compiled/ko.json
M	localization/en/ui.json
M	localization/key_registry.json
M	localization/ko/ui.json
M	rust/crates/sim-bridge/src/ffi/food/chronicle.rs
M	rust/crates/sim-core/src/chronicle.rs
M	rust/crates/sim-core/src/colony/food.rs
A	rust/crates/sim-core/src/colony/food_evidence.rs
M	rust/crates/sim-core/src/colony/mod.rs
M	rust/crates/sim-systems/src/runtime/food/events.rs
A	rust/crates/sim-systems/src/runtime/food/evidence.rs
A	rust/crates/sim-systems/src/runtime/food/evidence_tests.rs
A	rust/crates/sim-systems/src/runtime/food/evidence_validation.rs
A	rust/crates/sim-systems/src/runtime/food/evidence_validation_tests.rs
M	rust/crates/sim-systems/src/runtime/food/meals.rs
M	rust/crates/sim-systems/src/runtime/food/mod.rs
A	rust/crates/sim-systems/src/runtime/food/operation_tests.rs
M	rust/crates/sim-systems/src/runtime/food/validation.rs
M	rust/crates/sim-systems/src/runtime/jobs/cancel.rs
M	rust/crates/sim-systems/src/runtime/jobs/execute.rs
M	rust/crates/sim-systems/src/runtime/jobs/harvest.rs
M	rust/crates/sim-systems/src/runtime/mind/behaviours/harvest_food.rs
M	rust/crates/sim-systems/src/runtime/mind/context.rs
M	rust/crates/sim-systems/src/runtime/mind/facts.rs
M	rust/crates/sim-systems/src/runtime/mind/harvest_food.rs
M	rust/crates/sim-systems/src/runtime/mind/harvest_food_tests.rs
M	rust/crates/sim-systems/src/runtime/mind/system.rs
A	rust/crates/sim-systems/src/save/evidence_tests.rs
M	rust/crates/sim-systems/src/save/food_tests.rs
A	rust/crates/sim-systems/src/save/legacy_food_v13.rs
M	rust/crates/sim-systems/src/save/mod.rs
M	rust/crates/sim-test/tests/m6a_food_mind.rs
A	rust/crates/sim-test/tests/m6a_hunger_admission.rs
M	rust/crates/sim-test/tests/m6a_pending_food.rs
M	rust/crates/sim-test/tests/m6a_winter_plan_evidence.rs
M	scripts/ui/chronicle_text.gd
M	tools/v8/m6a_ui_capture.gd
M	tools/v8/test_core_mind_ui.gd
M	tools/v8/test_locale_trf.gd
```

## v8-m6a(p9): keep historical food explanations readable and verifiable

커밋: `5c7e05e677cebda16fea5825f88d83561acd9d2d` · 변경 경로 11개

```text
M	.claude/skills/worldsim-godot/references/common-pitfalls.md
M	CHANGELOG.md
M	docs/images/m6a_contact_sheet.png
M	docs/v8/m6a/M6A_DECISIONS.md
M	docs/v8/m6a/M6A_REPORT.md
M	localization/compiled/en.json
M	localization/compiled/ko.json
M	localization/en/ui.json
M	localization/key_registry.json
M	localization/ko/ui.json
M	tools/v8/m6a_ui_capture.gd
```

## v8-m6a(p10): preserve frozen evidence while observing BASE for the required 72 days

커밋: `bd799460c25a7019dd5d3f30296587c96674615b` · 변경 경로 11개

```text
M	.claude/skills/worldsim-testing/SKILL.md
M	CHANGELOG.md
M	docs/v8/m6a/M6A_DECISIONS.md
M	docs/v8/m6a/M6A_REPORT.md
A	tools/v8/m6a_base72.py
A	tools/v8/m6a_base72_entry.rs
M	tools/v8/m6a_gate_evidence.py
M	tools/v8/m6a_gate_source.py
A	tools/v8/m6a_observer_successor.py
A	tools/v8/test_m6a_base72.py
A	tools/v8/test_m6a_observer_successor.py
```
