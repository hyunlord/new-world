# M-CORE commit file inventory

Exact committed path lists through p7. Later delivery commits are appended after they exist; raw evidence remains external.

## v8-core(p0): preserve the owner contract before measuring the colony

Commit: `001f236c308a88615a650fca9af83d80041aa7a8`

```text
A	docs/v8/core/CORE_GOAL.md
```

## v8-core(p1): preserve the approved depth contract before replacing decisions

Commit: `06f431b33d32962169eaa7abf8e4be14ce7978c3`

```text
M	CHANGELOG.md
M	docs/HISTORY.md
M	docs/ROADMAP.md
M	docs/VISION.md
A	docs/v8/DEPTH_ROADMAP.md
A	docs/v8/core/CORE_DECISIONS.md
A	docs/v8/core/CORE_INVENTORY.md
```

## v8-core(p2): make the old brain observable before replacing its decisions

Commit: `e5835e51760fc08808b324e7cb62c718ee79fa22`

```text
M	CHANGELOG.md
M	docs/v8/core/CORE_DECISIONS.md
M	docs/v8/core/CORE_INVENTORY.md
M	rust/crates/sim-systems/Cargo.toml
M	rust/crates/sim-systems/src/lib.rs
A	rust/crates/sim-systems/src/perf_arms.rs
M	rust/crates/sim-systems/src/runtime/decision/system.rs
M	rust/crates/sim-systems/src/runtime/jobs/seam.rs
M	rust/crates/sim-test/Cargo.toml
A	rust/crates/sim-test/src/arm_profile.rs
M	rust/crates/sim-test/src/bench.rs
M	rust/crates/sim-test/src/bin/worldsim_probe.rs
M	rust/crates/sim-test/src/lib.rs
M	rust/crates/sim-test/src/probe/colony.rs
A	rust/crates/sim-test/src/story_yield/cli.rs
A	rust/crates/sim-test/src/story_yield/metric.rs
A	rust/crates/sim-test/src/story_yield/mod.rs
A	rust/crates/sim-test/src/story_yield/tests.rs
A	tools/v8/core_arm_report.py
A	tools/v8/core_score.py
A	tools/v8/test_core_arm_report.py
A	tools/v8/test_core_score.py
```

## v8-core(p3): restore planner-approved ledger admission for timing

Commit: `23378db34f4a55b698ccbf5177d097658d454e3e`

```text
M	CHANGELOG.md
M	docs/v8/core/CORE_DECISIONS.md
M	docs/v8/core/CORE_GOAL.md
M	docs/v8/core/CORE_INVENTORY.md
A	docs/v8/core/CORE_PLANNER_CORRECTION_2026-10-26.md
```

## v8-core(p4): ground the replacement budget in admitted measurements

Commit: `891c6cae7a2427b35b8f4524788c9861660cac48`

```text
M	docs/v8/core/CORE_DECISIONS.md
M	docs/v8/core/CORE_INVENTORY.md
A	docs/v8/core/CORE_PERF.md
```

## v8-core(p5): prove the colony mind before changing its production brain

Commit: `0ef68a1d2b6eb946e150963071b2e623fa58d4b0`

```text
M	.claude/skills/worldsim-pitfalls/SKILL.md
M	AGENTS.md
M	CHANGELOG.md
M	docs/ARCHITECTURE.md
M	docs/HISTORY.md
M	docs/ISSUES.md
M	docs/ROADMAP.md
M	docs/VISION.md
A	docs/decisions/0046-colony-mind.md
A	docs/decisions/0047-baseline-isolation.md
A	docs/v8/bridge_api_core.txt
M	docs/v8/core/CORE_DECISIONS.md
M	docs/v8/core/CORE_INVENTORY.md
A	docs/v8/core/CORE_SHADOW.md
A	docs/v8/core/MIND_AUTHORING.md
A	docs/v8/core/SYSTEM_WORLDS.md
M	docs/v8/snapshot_schema.md
M	localization/compiled/en.json
M	localization/compiled/ko.json
M	localization/en/ui.json
M	localization/key_registry.json
M	localization/ko/ui.json
M	rust/crates/sim-bridge/src/ffi/causal.rs
A	rust/crates/sim-bridge/src/ffi/mind.rs
M	rust/crates/sim-bridge/src/ffi/mod.rs
M	rust/crates/sim-bridge/src/ffi/node.rs
A	rust/crates/sim-core/data/colony/mind.ron
M	rust/crates/sim-core/src/causal/event.rs
M	rust/crates/sim-core/src/chronicle.rs
A	rust/crates/sim-core/src/colony/mind/compile.rs
A	rust/crates/sim-core/src/colony/mind/error.rs
A	rust/crates/sim-core/src/colony/mind/facts.rs
A	rust/crates/sim-core/src/colony/mind/mod.rs
A	rust/crates/sim-core/src/colony/mind/model.rs
A	rust/crates/sim-core/src/colony/mind/raw.rs
A	rust/crates/sim-core/src/colony/mind/select.rs
A	rust/crates/sim-core/src/colony/mind/selection.rs
A	rust/crates/sim-core/src/colony/mind/selection_tests.rs
A	rust/crates/sim-core/src/colony/mind/tests.rs
A	rust/crates/sim-core/src/colony/mind_job_origins.rs
A	rust/crates/sim-core/src/colony/mind_records.rs
A	rust/crates/sim-core/src/colony/mind_records/codec.rs
A	rust/crates/sim-core/src/colony/mind_records/dictionary.rs
A	rust/crates/sim-core/src/colony/mind_records/display.rs
A	rust/crates/sim-core/src/colony/mind_records/history.rs
A	rust/crates/sim-core/src/colony/mind_records/reader.rs
A	rust/crates/sim-core/src/colony/mind_records/source.rs
A	rust/crates/sim-core/src/colony/mind_records/tests.rs
A	rust/crates/sim-core/src/colony/mind_state.rs
M	rust/crates/sim-core/src/colony/mod.rs
M	rust/crates/sim-engine/src/legacy_v7.rs
M	rust/crates/sim-engine/src/lib.rs
A	rust/crates/sim-engine/src/registration.rs
M	rust/crates/sim-systems/Cargo.toml
M	rust/crates/sim-systems/src/commands.rs
M	rust/crates/sim-systems/src/lib.rs
A	rust/crates/sim-systems/src/mind_observer.rs
M	rust/crates/sim-systems/src/runtime/agent/movement.rs
R097	rust/crates/sim-systems/src/runtime/decision/consume.rs	rust/crates/sim-systems/src/runtime/agent_actions/consume.rs
R094	rust/crates/sim-systems/src/runtime/decision/forage.rs	rust/crates/sim-systems/src/runtime/agent_actions/forage.rs
R097	rust/crates/sim-systems/src/runtime/decision/memory_bias.rs	rust/crates/sim-systems/src/runtime/agent_actions/memory_bias.rs
A	rust/crates/sim-systems/src/runtime/agent_actions/mod.rs
R100	rust/crates/sim-systems/src/runtime/decision/needs.rs	rust/crates/sim-systems/src/runtime/agent_actions/needs.rs
R093	rust/crates/sim-systems/src/runtime/decision/routine/lifecycle.rs	rust/crates/sim-systems/src/runtime/agent_actions/routine/lifecycle.rs
R094	rust/crates/sim-systems/src/runtime/decision/routine/mod.rs	rust/crates/sim-systems/src/runtime/agent_actions/routine/mod.rs
R099	rust/crates/sim-systems/src/runtime/decision/routine/places.rs	rust/crates/sim-systems/src/runtime/agent_actions/routine/places.rs
A	rust/crates/sim-systems/src/runtime/agent_actions/routine/preview.rs
A	rust/crates/sim-systems/src/runtime/agent_actions/routine/selected.rs
R098	rust/crates/sim-systems/src/runtime/decision/targets.rs	rust/crates/sim-systems/src/runtime/agent_actions/targets.rs
R086	rust/crates/sim-systems/src/runtime/decision/wood.rs	rust/crates/sim-systems/src/runtime/agent_actions/wood.rs
M	rust/crates/sim-systems/src/runtime/decision/mod.rs
M	rust/crates/sim-systems/src/runtime/decision/system.rs
M	rust/crates/sim-systems/src/runtime/jobs/assign.rs
M	rust/crates/sim-systems/src/runtime/jobs/seam.rs
M	rust/crates/sim-systems/src/runtime/memory/memory_system.rs
A	rust/crates/sim-systems/src/runtime/mind/behaviours/context.rs
A	rust/crates/sim-systems/src/runtime/mind/behaviours/gather.rs
A	rust/crates/sim-systems/src/runtime/mind/behaviours/gather_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/behaviours/mod.rs
A	rust/crates/sim-systems/src/runtime/mind/behaviours/needs.rs
A	rust/crates/sim-systems/src/runtime/mind/behaviours/owned.rs
A	rust/crates/sim-systems/src/runtime/mind/behaviours/postpass.rs
A	rust/crates/sim-systems/src/runtime/mind/behaviours/postpass_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/behaviours/prepare.rs
A	rust/crates/sim-systems/src/runtime/mind/behaviours/routine.rs
A	rust/crates/sim-systems/src/runtime/mind/behaviours/settle.rs
A	rust/crates/sim-systems/src/runtime/mind/behaviours/social.rs
A	rust/crates/sim-systems/src/runtime/mind/behaviours/tests.rs
A	rust/crates/sim-systems/src/runtime/mind/context.rs
A	rust/crates/sim-systems/src/runtime/mind/facts.rs
A	rust/crates/sim-systems/src/runtime/mind/job_origins.rs
A	rust/crates/sim-systems/src/runtime/mind/mod.rs
A	rust/crates/sim-systems/src/runtime/mind/program.rs
A	rust/crates/sim-systems/src/runtime/mind/record.rs
A	rust/crates/sim-systems/src/runtime/mind/record/draft.rs
A	rust/crates/sim-systems/src/runtime/mind/record/memory.rs
A	rust/crates/sim-systems/src/runtime/mind/record/provenance.rs
A	rust/crates/sim-systems/src/runtime/mind/record/tests.rs
A	rust/crates/sim-systems/src/runtime/mind/record/types.rs
A	rust/crates/sim-systems/src/runtime/mind/shadow/compare.rs
A	rust/crates/sim-systems/src/runtime/mind/shadow/mod.rs
A	rust/crates/sim-systems/src/runtime/mind/system.rs
A	rust/crates/sim-systems/src/runtime/mind/system_owner_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/system_rest_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/system_tests.rs
M	rust/crates/sim-systems/src/runtime/mod.rs
M	rust/crates/sim-systems/src/runtime/mood/mod.rs
M	rust/crates/sim-systems/src/runtime/resource_regen/mod.rs
M	rust/crates/sim-systems/src/runtime/settlement/fire.rs
M	rust/crates/sim-systems/src/runtime/settlement/firewood.rs
M	rust/crates/sim-systems/src/runtime/settlement/food_economy.rs
M	rust/crates/sim-systems/src/runtime/settlement/stockpile_consume.rs
M	rust/crates/sim-systems/src/runtime/social/social_interaction_system.rs
M	rust/crates/sim-systems/src/runtime/steward/mod.rs
M	rust/crates/sim-systems/src/runtime/wildlife/draft.rs
M	rust/crates/sim-systems/src/runtime/wildlife/flee.rs
A	rust/crates/sim-systems/src/save/mind_tests.rs
A	rust/crates/sim-systems/src/save/mind_validation.rs
M	rust/crates/sim-systems/src/save/mod.rs
M	rust/crates/sim-systems/src/worldgen/mod.rs
M	rust/crates/sim-test/Cargo.toml
M	rust/crates/sim-test/src/bench.rs
M	rust/crates/sim-test/src/bin/worldsim_probe.rs
M	rust/crates/sim-test/src/lib.rs
A	rust/crates/sim-test/src/mind_coverage.rs
A	rust/crates/sim-test/src/mind_coverage_tests.rs
A	rust/crates/sim-test/src/mind_memory.rs
A	rust/crates/sim-test/src/mind_memory_tests.rs
A	rust/crates/sim-test/src/mind_shadow.rs
A	rust/crates/sim-test/src/mind_shadow/factory.rs
M	rust/crates/sim-test/src/probe/adult_work.rs
M	rust/crates/sim-test/src/probe/colony.rs
M	rust/crates/sim-test/src/probe/fingerprint.rs
M	rust/crates/sim-test/src/probe/meals_firewood.rs
M	rust/crates/sim-test/src/probe/metrics.rs
A	rust/crates/sim-test/src/probe/mind_fingerprint.rs
A	rust/crates/sim-test/src/probe/mind_fingerprint_tests.rs
M	rust/crates/sim-test/src/probe/mod.rs
M	rust/crates/sim-test/src/probe/seasons_forage.rs
M	rust/crates/sim-test/tests/engine_causal_events.rs
A	rust/crates/sim-test/tests/engine_mind_ffi.rs
M	rust/crates/sim-test/tests/settlement_construction.rs
M	rust/crates/sim-test/tests/settlement_construction_components.rs
M	scripts/ui/panels/agent_inspector_panel.gd
M	scripts/ui/panels/causal_panel.gd
A	tools/v8/core_gate_checks.sh
A	tools/v8/core_perf.sh
A	tools/v8/core_perf_check.py
A	tools/v8/core_shadow_freeze.py
A	tools/v8/core_story_check.py
A	tools/v8/core_why_check.py
M	tools/v8/gate.sh
M	tools/v8/test_bridge_api.py
A	tools/v8/test_core_mind_ui.gd
A	tools/v8/test_core_mind_ui.gd.uid
A	tools/v8/test_core_perf_check.py
A	tools/v8/test_core_shadow_freeze.py
A	tools/v8/test_core_story_check.py
A	tools/v8/test_core_why_check.py
```

## v8-core(p6): give colony worlds their verified mind while preserving baseline

Commit: `fbc9a9babc9bb5df1d06ca1e655901e480521f66`

```text
M	AGENTS.md
M	CHANGELOG.md
M	docs/ARCHITECTURE.md
M	docs/HISTORY.md
M	docs/ISSUES.md
M	docs/decisions/0046-colony-mind.md
M	docs/decisions/0047-baseline-isolation.md
M	docs/v8/core/CORE_DECISIONS.md
M	docs/v8/core/MIND_AUTHORING.md
M	docs/v8/core/SYSTEM_WORLDS.md
M	rust/crates/sim-engine/src/registration.rs
M	rust/crates/sim-systems/src/lib.rs
M	rust/crates/sim-systems/src/runtime/mind/system_rest_tests.rs
M	rust/crates/sim-systems/src/runtime/mind/system_tests.rs
M	rust/crates/sim-systems/src/save/mind_tests.rs
M	rust/crates/sim-systems/src/save/mod.rs
M	rust/crates/sim-systems/src/worldgen/mod.rs
M	rust/crates/sim-test/src/mind_shadow.rs
M	rust/crates/sim-test/tests/colony_commands.rs
M	tools/v8/core_gate_checks.sh
A	tools/v8/core_shadow_test_migration.py
A	tools/v8/test_core_shadow_test_migration.py
```

## v8-core(p7): preserve the research baseline behind a colony-free decision boundary

Commit: `be12050c951dfb025f3440d34b657eec0663b817`

```text
M	AGENTS.md
M	CHANGELOG.md
M	docs/ARCHITECTURE.md
M	docs/HISTORY.md
M	docs/ISSUES.md
M	docs/decisions/0047-baseline-isolation.md
M	docs/v8/core/CORE_DECISIONS.md
M	docs/v8/core/MIND_AUTHORING.md
M	rust/crates/sim-systems/src/lib.rs
R100	rust/crates/sim-systems/src/runtime/decision/mod.rs	rust/crates/sim-systems/src/runtime/baseline/decision/mod.rs
R098	rust/crates/sim-systems/src/runtime/decision/system.rs	rust/crates/sim-systems/src/runtime/baseline/decision/system.rs
A	rust/crates/sim-systems/src/runtime/baseline/mod.rs
M	rust/crates/sim-systems/src/runtime/jobs/mod.rs
D	rust/crates/sim-systems/src/runtime/jobs/seam.rs
M	rust/crates/sim-systems/src/runtime/mod.rs
M	rust/crates/sim-systems/src/worldgen/mod.rs
M	rust/crates/sim-test/src/bin/worldsim_probe.rs
M	rust/crates/sim-test/src/bin/worldsim_systems.rs
M	rust/crates/sim-test/src/lib.rs
M	rust/crates/sim-test/src/mind_shadow.rs
D	rust/crates/sim-test/src/mind_shadow/factory.rs
M	rust/crates/sim-test/src/probe/deaths.rs
M	rust/crates/sim-test/tests/agents_combat.rs
M	rust/crates/sim-test/tests/agents_decision.rs
M	rust/crates/sim-test/tests/agents_memory_system.rs
M	rust/crates/sim-test/tests/agents_preemption.rs
M	rust/crates/sim-test/tests/agents_rhythm.rs
M	rust/crates/sim-test/tests/agents_seek.rs
M	rust/crates/sim-test/tests/agents_sleep.rs
M	rust/crates/sim-test/tests/agents_viz.rs
M	rust/crates/sim-test/tests/b4_controlled/cold.rs
M	rust/crates/sim-test/tests/chronicle_combat.rs
M	rust/crates/sim-test/tests/determinism.rs
M	rust/crates/sim-test/tests/engine_worldgen.rs
M	rust/crates/sim-test/tests/family_households.rs
A	rust/crates/sim-test/tests/mind_shadow_retired.rs
M	rust/crates/sim-test/tests/mux2_fuel_sleep.rs
M	rust/crates/sim-test/tests/relations_caps.rs
M	rust/crates/sim-test/tests/relations_rule.rs
M	rust/crates/sim-test/tests/relations_social_components.rs
M	rust/crates/sim-test/tests/relations_social_freeze_fix.rs
M	rust/crates/sim-test/tests/relations_social_system.rs
M	rust/crates/sim-test/tests/relations_social_ui.rs
M	rust/crates/sim-test/tests/routine_support/mod.rs
M	rust/crates/sim-test/tests/settlement_construction.rs
M	rust/crates/sim-test/tests/settlement_firewood.rs
M	rust/crates/sim-test/tests/settlement_food.rs
M	rust/crates/sim-test/tests/settlement_gather.rs
M	rust/crates/sim-test/tests/settlement_migration.rs
M	rust/crates/sim-test/tests/settlement_resource_scarcity.rs
M	rust/crates/sim-test/tests/settlement_resource_substrate.rs
M	rust/crates/sim-test/tests/settlement_seasons.rs
M	rust/crates/sim-test/tests/settlement_village.rs
M	tools/v8/core_gate_checks.sh
M	tools/v8/core_perf_check.py
M	tools/v8/system_table.py
M	tools/v8/test_core_perf_check.py
```

## p8 delivery commit containing this inventory

The commit containing this section preserves the p7 Rust tree; its exact staged paths are below. Exact commit: `44e0a0266c085d409c16c77f4a2a0181b8a2a750`. The full gate at this commit failed; this is not a publication claim.

```text
M	CHANGELOG.md
M	docs/HISTORY.md
M	docs/ISSUES.md
M	docs/ROADMAP.md
M	docs/decisions/0046-colony-mind.md
M	docs/decisions/0047-baseline-isolation.md
A	docs/images/core_mind_hunger_en.png
A	docs/images/core_mind_threat_ko.png
A	docs/v8/core/CORE_CHANGED_FILES.md
M	docs/v8/core/CORE_DECISIONS.md
A	docs/v8/core/CORE_EVAL.md
M	docs/v8/core/CORE_PERF.md
A	docs/v8/core/CORE_REPORT.md
A	docs/v8/core/STORY_YIELD.md
M	localization/compiled/en.json
M	localization/compiled/ko.json
M	localization/en/ui.json
M	localization/key_registry.json
M	localization/ko/ui.json
M	scripts/ui/chronicle_text.gd
M	scripts/ui/panels/agent_inspector_panel.gd
M	scripts/ui/panels/chronicle_panel.gd
A	tools/v8/test_core_chronicle.gd
M	tools/v8/test_core_mind_ui.gd
```

## p9 failed-gate report commit

Documentation only. The p8 full gate remains FAIL (42/45); no runtime, policy, threshold, fixture or evidence file is changed. Exact p9 SHA is available from Git history after commit creation.

```text
M	CHANGELOG.md
M	docs/HISTORY.md
M	docs/ISSUES.md
M	docs/ROADMAP.md
M	docs/v8/core/CORE_CHANGED_FILES.md
M	docs/v8/core/CORE_DECISIONS.md
M	docs/v8/core/CORE_PERF.md
M	docs/v8/core/CORE_REPORT.md
```

## P-3d: preserve real effects while reducing repeated decision preparation

Commit: `826a7ecd9e38ca3fc1fff253baa0933aee593022` (local, unpublished). Default-feature build and committed shadow/fresh evaluation pass; final paired side64 performance FAIL remains blocking.

```text
M	CHANGELOG.md
M	docs/ARCHITECTURE.md
M	docs/HISTORY.md
M	docs/ROADMAP.md
M	docs/decisions/0046-colony-mind.md
M	docs/v8/core/CORE_DECISIONS.md
M	docs/v8/core/CORE_PERF.md
M	docs/v8/core/CORE_REPORT.md
M	docs/v8/core/MIND_AUTHORING.md
M	rust/Cargo.toml
M	rust/crates/sim-core/src/colony/mind/compile.rs
A	rust/crates/sim-core/src/colony/mind/continuation.rs
A	rust/crates/sim-core/src/colony/mind/continuation_cache.rs
A	rust/crates/sim-core/src/colony/mind/continuation_cache_lifecycle_tests.rs
A	rust/crates/sim-core/src/colony/mind/continuation_cache_scratch_tests.rs
A	rust/crates/sim-core/src/colony/mind/continuation_cache_tests.rs
A	rust/crates/sim-core/src/colony/mind/continuation_cadence_tests.rs
A	rust/crates/sim-core/src/colony/mind/continuation_carry_tests.rs
A	rust/crates/sim-core/src/colony/mind/continuation_jump_tests.rs
A	rust/crates/sim-core/src/colony/mind/continuation_prefix_tests.rs
A	rust/crates/sim-core/src/colony/mind/continuation_reference.rs
A	rust/crates/sim-core/src/colony/mind/continuation_tests.rs
M	rust/crates/sim-core/src/colony/mind/mod.rs
M	rust/crates/sim-core/src/colony/mind/model.rs
M	rust/crates/sim-core/src/colony/mind/select.rs
A	rust/crates/sim-core/src/colony/mind/selector_jump_reference.rs
A	rust/crates/sim-core/src/colony/mind/selector_jump_tests.rs
M	rust/crates/sim-core/src/colony/mind_records.rs
M	rust/crates/sim-core/src/colony/mind_records/codec.rs
M	rust/crates/sim-core/src/colony/mind_records/tests.rs
M	rust/crates/sim-systems/src/runtime/mind/behaviours/mod.rs
M	rust/crates/sim-systems/src/runtime/mind/behaviours/tests.rs
M	rust/crates/sim-systems/src/runtime/mind/context.rs
M	rust/crates/sim-systems/src/runtime/mind/context/food_index_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/context_full_refresh_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/context_index_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/context_lazy_food_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/context_optimization_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/context_primary_snapshot_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/context_refresh_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/context_rich_upgrade_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/context_row_order_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/context_running_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/context_scope_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/continuation_cache_runtime_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/continuation_runtime.rs
A	rust/crates/sim-systems/src/runtime/mind/continuation_runtime_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/continuation_save_tests.rs
M	rust/crates/sim-systems/src/runtime/mind/facts.rs
M	rust/crates/sim-systems/src/runtime/mind/mod.rs
A	rust/crates/sim-systems/src/runtime/mind/primary_refresh_system_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/primary_snapshot_system_tests.rs
M	rust/crates/sim-systems/src/runtime/mind/program.rs
A	rust/crates/sim-systems/src/runtime/mind/scheduling.rs
A	rust/crates/sim-systems/src/runtime/mind/scheduling_counts.rs
A	rust/crates/sim-systems/src/runtime/mind/scheduling_tests.rs
A	rust/crates/sim-systems/src/runtime/mind/social_bucket_reference.rs
A	rust/crates/sim-systems/src/runtime/mind/social_bucket_tests.rs
M	rust/crates/sim-systems/src/runtime/mind/system.rs
M	tools/v8/core_gate_checks.sh
A	tools/v8/core_p3d_fixture_schema.json
A	tools/v8/core_p3d_fixtures.py
A	tools/v8/core_p3d_host_check.py
A	tools/v8/core_p3d_successor.py
M	tools/v8/core_score.py
M	tools/v8/core_story_check.py
M	tools/v8/core_why_check.py
A	tools/v8/test_core_p3d_fixtures.py
A	tools/v8/test_core_p3d_host_check.py
A	tools/v8/test_core_p3d_successor.py
M	tools/v8/test_core_score.py
A	tools/v8/test_core_seed_protocol.py
M	tools/v8/test_core_story_check.py
M	tools/v8/test_core_why_check.py
```

### Hard-stop documentation follow-up

Documentation only: CHANGELOG.md; docs/HISTORY.md; docs/ROADMAP.md; docs/v8/core/CORE_CHANGED_FILES.md, CORE_DECISIONS.md, CORE_EVAL.md, CORE_PERF.md, CORE_REPORT.md, CORE_SHADOW.md. Runtime/tools remain byte-identical to826a7ecd. Full gate incomplete, required paired performance FAIL; no publication.

### P7 conditional acceptance harness

No Rust or policy changes. New tools: core_p7_admission.py, core_p7_perf.py, core_p7_bench.py, core_p7_acceptance.py, core_p7_publication.py and five focused test modules. Updated gate.sh, perf.sh, core_gate_checks.sh, mux2_gate_receipts.py, mux2_stall_test.gd and its command tests enforce scoped admission/justification. CORE_P7_TOOL_DELTA.json records exact before/after hashes for every tools/v8 delta. CHANGELOG, ISSUES, CORE_DECISIONS, CORE_PERF and CORE_REPORT record the owner decision, open debt and pending full gate. This checkpoint is not a gate pass.

P7 denominator correction adds core_p7_ledger.py and focused precision/validation tests; core_p7_bench.py and core_p7_publication.py bind the unchanged bench rules to the exact fresh BASE ledger. Historical ledger and runtime remain unchanged. D38 preserves the pre-timing interrupted run; the next run starts from step1.
