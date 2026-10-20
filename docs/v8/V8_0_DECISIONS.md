# V8-0 decisions

One numbered entry per decision taken without the owner (goal 0.3). Format:
context, choice, reason, evidence. Raw data paths are under
`/Users/rexxa/work/v8-0-data/` (`$DATA`, outside the repository).

## D1 — Push target of `v8/wip`
- Context: the owner's launch command created `v8/wip` from `origin/codex/art-production-2026-10`, so its upstream was that `codex/*` branch; a bare `git push` would have pushed to a forbidden branch.
- Choice: the first push was `git push -u origin v8/wip`, re-pointing the upstream to `origin/v8/wip`; every later push names the branch explicitly.
- Reason: goal section 2 (never push `codex/*`).
- Evidence: `git rev-parse --abbrev-ref @{u}` before/after; `$DATA/PROGRESS.md`.

## D2 — Tags
- Context: no `v7-final-*` tag existed on the remote (`git ls-remote origin 'refs/tags/v7-final-*'` was empty).
- Choice: annotated tags `v7-final-sim` → `354e4a13` and `v7-final-art` → `b1ca032f` created and pushed; no `-v8pin` variant was needed.
- Evidence: `git ls-remote` in the Part 0 log of this file's commit.

## D3 — Identity capture flags
- Context: goal 4.1 asks for the probe on `game_v26`, seeds 0–31, 120 days, with chronicle, and `--check-determinism` on seeds 0–3.
- Choice: `tools/v8/identity.sh` runs the probe with Pass 21's identity flags (`--clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 --sample-every 120`, so the clock-sample blocks of the summary are compared too) and `--workers 6`; the determinism check is a second probe run on seeds 0..4 (half-open) whose `DETERMINISM PASS` line is required.
- Reason: the same flags as Pass 21 cover the most summary content; worker count does not change results (Pass 21 identity procedure).
- Evidence: `tools/v8/identity.sh`, `$DATA/base/probe.log`, `$DATA/base/determinism.txt`.

## D4 — `git_sha` excluded from the summary comparison
- Context: the probe writes the run label `WORLDSIM_GIT_SHA` into `summary.json` (`probe/metrics.rs`) and into every chronicle file. Base and candidate are different commits.
- Choice: `tools/v8/compare_summaries.py` (a copy of Pass 21's) excludes `git_sha` as a label key in addition to the timing keys, and now also fails on one-sided paths; the chronicle comparison excludes only `git_sha` (Pass 21 also tolerated an extended `kinds` list; V8-0 does not).
- Reason: `git_sha` is provenance, not simulation output; everything else must be identical.
- Evidence: `tools/v8/compare_summaries.py`, `tools/v8/identity.sh`.

## D5 — Worldgen dumps live beside the rules dumps
- Context: goal 5.3 puts both dumps under `rust/crates/sim-core/tests/data/`, but `WorldGenConfig` is defined in `sim-systems` (`worldgen/mod.rs`), which `sim-core` cannot depend on.
- Choice: both dump families are committed under `sim-core/tests/data/` as the goal says; the rules comparison test lives in `sim-core/tests/`, the worldgen comparison test in `sim-systems/tests/` and reads the same files by relative path.
- Evidence: `rust/crates/sim-core/tests/data/v7_final_*`.

## D6 — Screenshots
- Context: `tools/revitalize/capture.sh` exits 77 ("Tier A requires an Aqua session") because this run's shell is in a `Background` launchd session; its GDScript (`scripts/test/capture_framings.gd`) is deleted by this pass anyway.
- Choice: a fresh, small `tools/v8/capture.gd` (main scene, `advance_ticks` to tick 1440, a zoomed-in shot at the first village tile at zoom 4 and a zoomed-out shot at the map centre at zoom 1, the framings of Pass 21's F2/F1) runs windowed with `--rendering-driver opengl3 --resolution 1920x1080`, which works from this session. The same script captures base (copied untracked into the base worktree) and candidate.
- Evidence: `$DATA/base/shots/{near,far}_t1440.png`, `$DATA/base/shots.log`.

## D7 — Smoke counts at base
- Context: goal 5.3 asks for the counts of `SCRIPT ERROR`, `ERROR:` and missing-locale warnings in a 900-frame headless smoke at base.
- Choice: counted after two headless imports (the first import of a fresh worktree logs font-import errors before the fonts are imported; the gate also imports twice). `Locale.ltr` returns the key itself for a missing key and logs nothing; the only locale warnings `locale.gd` can print start with `Locale:`, so that prefix is what is counted.
- Evidence: `$DATA/base/smoke.log` (5 lines: 0 / 0 / 0), `$DATA/base/import.log` (13 font-import errors, first import), `$DATA/base/import2.log` (0).

## D8 — The V8 smoke also scans labels for raw keys
- Context: `Locale.ltr` returns the key itself for a missing key and logs nothing (`scripts/core/locale.gd`), so a `Locale:` warning count cannot catch a key pruned by mistake (7.4).
- Choice: `tools/v8/smoke.sh` runs the plain 900-frame boot and `tools/v8/smoke.gd`, which boots the main scene for 900 frames and counts Label/RichTextLabel/Button texts that are key-shaped (`^[A-Z][A-Z0-9]*(_[A-Z0-9]+)+$`). Base counts were recorded with the same scripts in the base worktree: 0 script errors, 0 errors, 0 locale warnings, 0 raw keys (75 labels).
- Evidence: `tools/v8/smoke_base_counts.txt`, `$DATA/base/smoke_v8/`.

## D9 — `probe/pass*.rs` are kept (and renamed), not deleted
- Context: goal 6.3 lists `src/probe/pass*.rs` for deletion, but those modules compute blocks of the probe's `summary.json` and raw rows (`probe/runner.rs` calls `pass10::seed_info`, `pass11::seed_block`, `pass12::seed_block`, `pass13::seed_block`, `pass20::seed_block`, …; `summary.rs` calls `pass4_sections`, `pass5_sections`, `pass8::rules_block`). Deleting them would change the summary that identity (4.1) requires to be identical.
- Choice: keep them as "probe modules the kept binaries need" (7.7) and rename them by content in 8.2.
- Evidence: `rust/crates/sim-test/src/probe/{runner,summary}.rs`.

## D10 — `reset_world_stress` stays in the bridge API, re-pointed to the baseline
- Context: 8.1 deletes `reset_world_stress` constructors that nothing kept uses; 4.2 requires the final bridge API to equal the base list minus the intervention methods only. The bridge method `reset_world_stress` builds `WorldGenConfig::stress_v19` under `WorldRules::game_v19`.
- Choice: keep the `#[func]` name; in 8.1 it builds the same stress world under the baseline rules (the world generator inputs of `stress_v19` equal the baseline's apart from founders and size). Benchmark-only, never called by the kept UI.
- Evidence: `docs/v8/bridge_api_v7_final.txt`; `sim-bridge/src/ffi/world_node.rs` `build_stress_engine`.

## D11 — `nature_renderer.gd` is kept (ASK), not deleted
- Context: 7.3 keeps an explicit list of renderers and deletes "everything else in `scripts/ui/**`", with examples that are all overlays, badges and HUD panels. `nature_renderer.gd` (+ `nature_canopy.gd`) draws the world's trees, berry bushes and deadwood — the resources people walk to — not an overlay. Without it the map shows bare terrain and people walk to invisible food.
- Choice: when in doubt, keep (4.4): the node and its two scripts stay, the item goes on the ASK list with the recommendation to replace it with the M1 tile layer.
- Evidence: `scripts/ui/nature_renderer.gd` header; base shots `$DATA/base/shots/*.png`.

## D12 — Art-kernel fixture tests and their 135 MB fixtures are deleted
- Context: `sim-bridge/src/ffi/art_kernels.rs` has three unit tests that read `tests/fixtures/art_p12/declump_v1.bin` (57 MB) and `art_p13b/walker_v1.bin` (83 MB), recorded by the deleted `scripts/test/art` capture tools. The docs gate (11.1 step 14) forbids tracked files over 5 MB, and VISION §5 reduces the art measurement rigs to one perf regression check.
- Choice: delete the fixtures and the fixture-reading tests; the other `art_kernels` unit tests stay.
- Evidence: `art_kernels.rs` lines ~372, ~1101, ~1245 at `v7-final-art`.

## D13 — 7.7 (sim-test research code) runs before 7.2 (god hand)
- Context: the world-model recorder (`sim-test/src/worldmodel/**`), `perf.rs`'s intervention mode and several harness tests call the intervention API; deleting the god hand first would leave them uncompilable.
- Choice: the sim-test batch (worldmodel, ablation, triaged tests) is committed before the god-hand batch. Each batch is still one area per commit, with build + tests + clippy + identity after it.

## D14 — Localization source becomes the category JSONs
- Context: `localization/compiled/*.json` (what `locale.gd` loads) is generated by `tools/localization_compile.py` from `localization/fluent/*/messages.ftl` (manifest `source_format: fluent`); the category JSONs disagree with it on 1,457 keys (Part 1 scratch run). `tools/localization_compile.py` reproduces the committed compiled files byte-identically.
- Choice: keep `tools/localization_compile.py` as the generator; `tools/v8/locale_prune.py` writes the kept keys into the category JSONs with the values of the current compiled tables, switches the manifest to `source_format: json`, `preserve_key_ids: false`; Fluent and `key_owners.json` are deleted; `key_registry.json` stays because the generator rewrites it on every run. `locale_prune.py --verify` checks compiled = category JSONs = kept keys, each with the `v7-final-art` value.
- Evidence: scratch dry run (157 keys, categories ui + buildings), `tools/v8/locale_prune.py`.

## D15 — Tests on `WorldRules::default()` count as "setting" a flag (8.3)
- Context: the test triage found that 51 KEEP files and most adapted fixtures run on `WorldRules::default()`, which takes the non-baseline side of many flags (`strict_consume=false`, `keyed_draws=false`, `night_rest=None`, `BirthRule::Legacy`, every `SocietyRules` option `None`, …). 8.3 removes a field only when "no kept test sets it to the other value".
- Choice: the literal reading — a kept test that runs on `default()` does reach the other side, so those branches stay. 8.3 then applies only to fields whose other side no kept test reaches (checked per field when 8.3 runs).
- Reason: deleting a branch that kept tests execute would turn rule-independent tests into tests of nothing; 8.3 is the most droppable restructure step.
- Evidence: `docs/v8/harvest/test_triage.md` §1 "A ruling DECISIONS needs".

## D16 — `game_v8`, `game_default`, `game_scene` and `build_world` go with the chain
- Context: the chain's root (`WorldRules::game_v8`, `WorldGenConfig::game_default`, `scenes::game_scene`, `worldgen::build_world`) is not named in 8.1 but nothing live uses it.
- Choice: delete them in 8.1 with the chain; tests that used them are adapted to the baseline (triage recipe R1/R3) or deleted. The bridge's pure helpers used by kept tests (`init_production_engine`, `bootstrap_spawn_agents`, `seed_finite_resource_scarcity`) stay.
- Evidence: `docs/v8/harvest/test_triage.md` §1.

## D17 — 7.6: `--cacheinfo` cleared skip-worktree; restored at once
- Context: the goal's 7.6 commands (`git hash-object -w` + `git update-index --cacheinfo 100644,<sha>,.claude/settings.json`) re-create the index entry, which drops the skip-worktree bit: after the commit `git ls-files -v .claude/settings.json` showed `H`.
- Choice: re-set it immediately with `git update-index --skip-worktree .claude/settings.json` (the working file was never edited); verified `S`, and `git show HEAD:.claude/settings.json` byte-equal to the local `{}` file. Recorded as a pitfall lesson.
- Evidence: commit `19baad71`; this log.
- Ordering note (extends D13): 7.6 and a separate tools batch (`53da844f`) were committed before 7.2–7.5 because they touch no code the later batches depend on.

## D18 — 7.2 scope: what "intervention-only" removed and what stayed
- Removed (runtime state only interventions set; 0/false/empty in every natural run): `SimResources::{fire_blocked_until, cold_snap_nights, cold_snap_tonight, intervention_log}`, `Tallies::{interventions_applied, wet_burn_wood}` (`wet_burn_wood` was incremented only by the intervention-rain branch of `FireSystem`), the types `Intervention` / `InterventionRecord` (`sim-engine/src/intervention.rs`), `sim-systems/src/runtime/intervention.rs`, the rain and cold-snap branches of `FireSystem` and the cold-snap `+1` of `firewood::night_rate`, the bridge's `apply_intervention` and `get_intervention_state` `#[func]`s with `parse_intervention`, `collect_intervention_state`, `apply_intervention_command`, `intervention_reply_to_dict` and their four tests, the GDScript panel/scene/effects and `SimReads` entries.
- Both bridge methods are intervention methods (`get_intervention_state` only reported rain, cold snaps and the intervention log), so `docs/v8/bridge_api_removed.txt` lists both; `tools/v8/bridge_api.py --check` passes (37 = 39 − 2).
- Kept: the rule fields `FirewoodRule::{rain_soak_permille, rain_wet_burn}`. They acted only through interventions, but they are rules data with non-zero values (300, true) printed in the probe summary's rules block and in the v7-final rules dumps; removing them would make identity fail on non-zero values. `windfall_over_snow` is read by the live `ResourceRegenSystem` and is not intervention-only. All three are listed as candidates for removal when the probe schema is next revised. (Update, 8.3: `rain_wet_burn` turned out not to be printed by the probe and has no reader, so 8.3 removed it — D25.)
- Probe output: five zero-valued keys disappear (`interventions_applied_total`, `fire_blocked_settlements`, `cold_snap_nights_left`, `wet_burn_wood_total` in raw rows, `wet_burn_wood` in the summary). `tools/v8/identity_removed_keys.txt` lists them; identity drops a listed key from the base data only where its base value is zero/false/null/empty.

## D19 — The fed stress world keeps its daily food gift inside the perf harness
- Context: `worldsim_perf`'s fed stress world (`quick`, `stress --feed fed`) fed the camp each day through the GiftFood intervention.
- Choice: `sim-test/src/perf.rs` `gift_food` does the same stockpile deposit (store `amount` food in the lowest-id settlement, report what was added). Benchmark-only; the game has no such call.
- Evidence: the `quick` fingerprint after 8.1 must equal base `stress --rules game_v26` (`d4d9b81677e1ebcc`).

## D20 — 7.3: overlays drawn from inside a kept file were unhooked there
- Context: `agent_renderer.gd` (keep set) itself created three of the overlays 7.3 deletes: the activity badges (`_update_activity_icon`, one call site), the porter route lines (a `PorterRoutes` child) and the founding-group layer (a `FoundingGroups` child, with its offscreen markers).
- Choice: remove those three hooks from `agent_renderer.gd` (and the now-unused `porter_settlements` read; the write-skip cache key keeps the zoom value it used) so `founding_group_renderer.gd` becomes unreachable and is deleted. Everything else the renderer needs stays, unchanged, as "M1 replaces" dependencies: `agent_activity.gd` + `activity_icons_p11.gd` (frame/pose choice), `porter_presentation_p12.gd` (porter poses and rows), `offscreen_marker.gd` (the chronicle-hover ring), `art_*`, `camp_seats.gd`, `carry_load_layer.gd`, `cold_effects.gd`, `morning_chill.gd`, `agent_frame_choices_p10.gd`, `agent_pose_moments.gd`, `agent_selection_index_p10.gd`, `settlement_selection.gd`, `chronicle_text.gd`, `person_*`, `shelter_cold.gd`, `wood_visuals.gd`, `nature_layout.gd`, `panels/art_ui.gd`, `sim_reads.gd`.
- Removed from `main.tscn`: WorldEffects, ActivityTrailRenderer, SeekVizRenderer, NeedBarRenderer, DeathVizRenderer, SettlementOverviewRenderer, ZoomLodController, HudStatusPanel (the village news/status panel), ActivityTooltipP13 (the god-hand nodes went in 7.2). Kept: NatureRenderer (D11).
- Deleted scripts (unreachable after the edit, `tools/v8/gd_reach.py`): activity_trail_renderer, death_viz_renderer, founding_group_renderer, need_bar_renderer, panels/feed_food_ledger, panels/hud_status_panel, person_activity_tooltip_p13, restless_meter, seek_viz_renderer, settlement_labels, settlement_overview_renderer, settlement_relations_p12, village_activity_summary_p11, world_effects, zoom_lod_controller.
- Evidence: import 0 errors; smoke 0/0/0/0; strict check 38 files + 27 FFI calls clean; candidate shots `$DATA/checks/p2_73/shots/{near,far,inspect,causal,chronicle}_t1440.png` show the same world (population 26, 4 shelters), the top bar, chronicle, inspector and causal panels.

## D21 — 8.1: how the baseline literal was produced and checked
- `rust/crates/sim-core/src/rules/baseline.rs` was generated from the Part 0 dump `v7_final_rules_seed0.txt` (Debug syntax is Rust struct-literal syntax once enum variants get their paths); the two fields that differ between the four dumped seeds — `firewood.winter_severity_permille` and `seasons.regen_permille` — are written as the calls the chain used (`variety.winter_severity_permille()`, `variety.scale_regen([750, 1250, 1000, 500])`, `WorldVariety::draw_v18(seed)`). `WorldGenConfig::baseline` likewise (founders, bushes, deadwood, forest and regions from the draw).
- `tests/baseline_matches_v7_final.rs` (sim-core) and `tests/baseline_worldgen_matches_v7_final.rs` (sim-systems) compare `format!("{:#?}")` with the four dumps: both pass.
- 8.1 is done in two commits: (a) add the baseline next to the chain and switch every live caller (bridge `build_game_engine`, `build_stress_engine` (D10), probe scene `baseline` with `game_v26` as an alias, perf default rules `baseline`, `sim_test::baseline_engine`), verified by identity; (b) delete the chain and port the tests.
- The perf `quick` world becomes `stress(15100, 500, 64)` under the baseline rules; its output file is `quick_stress_baseline_s15100_n500_side64_fed.json` (was `quick_stress_v19_…`), and its fingerprint must equal the base `stress --rules game_v26` run (`d4d9b81677e1ebcc`).

## D22 — Codex skills are relative symlinks
- `.agents/skills/<name>` → `../../.claude/skills/<name>` for all seven skills (git mode 120000). Codex reads skill files through the filesystem; no evidence was found that it cannot follow a symlinked directory (the OS resolves it before any reader sees it), so the goal's default applies and no sync script is needed. `check_docs.sh` verifies each link resolves and is stored as a symlink.

## D23 — 8.1b: the chain is deleted; how the tests were ported
- Deleted: every `WorldRules::game_v8 … game_v26` constructor with its `_nofission` / `_neutral` / `_inert` / `_with` variants and the `V18_HAZARD … V26_RELATIONS` constants (`rules/mod.rs` 1,821 → 819 lines, its version-delta tests with it), `WorldGenConfig::game_default … game_v26` and `stress_v19`, `worldgen::build_world`, and every scene but `baseline_scene` (`scenes.rs` 329 → 34 lines). The probe accepts `baseline` (alias `game_v26`, default `baseline`); perf labels are `stress_baseline` only.
- Tests that pinned a version delta, a frozen world or an old-scene outcome were deleted; tests of a version-independent property were ported to the baseline (unit fixtures: `WorldRules::baseline(seed)` or a sub-rule copied from it; scenes: `sim_test::baseline_engine(seed)`; seed-specific numbers re-measured and written in comments; numeric parameters of an old rule kept as test-local literals where the test is about that arithmetic, never an old flag value). Where a test's premise was "rule X off", the baseline is used with X switched off explicitly.
- Counts: sim-core/sim-systems/sim-bridge unit tests ported by the lead (routine, family, firewood, fire, housing, terrain, settlement, decision, beds, movement, households, bridge snapshots); `sim-test/src` unit tests: 33 ported, 0 deleted, 1 added; `sim-test/tests` batch 1 (13 files): 82 kept, 22 deleted; batch 2 (24 files + 2 support modules): 117 kept, 23 deleted, 1 new. Per-test records: `docs/v8/p3/8_1b_*.md`.
- Coverage: every live system keeps at least one test with its rule active (batch reports, "Coverage kept"). `harness_area_fall_floor` (regional winter-fall floor) was deleted per the triage, so that sub-feature of `ResourceRegenSystem` has no dedicated test; the system itself is covered.

## D24 — 8.2: renames (pass and version suffixes removed)
Every rename keeps the meaning in the name. Probe JSON keys and values (`"pass11"`, `"exposure_v2"`, `"firewood_v17"`, …) are output data the identity check compares, so they keep their spelling; only the Rust identifiers that hold them changed. No bridge `#[func]` name changed.
- **Variety:** `WorldVariety::draw_v18` → `draw`; `RANGES_V18` merged into `RANGES` (region bushes 8–12, region deadwood 12–18). Deleted, unused after 8.1b: the pre-v18 `draw` (old region ranges), `NEUTRAL`, `without_regions`. Variety tests rewritten for the one range set.
- **Probe modules** (`rust/crates/sim-test/src/probe/`): pass4 → `seasons_forage`, pass5 → `meals_firewood`, pass6 → `wood_work`, pass7 → `winter_severity`, pass8 → `night_burn`, pass10 → `exposure_health`, pass11 → `fission_settlements`, pass12 → `adult_work`, pass13 → `area_floor`, pass17 → `routine_trace`, pass17_activity → `routine_accounting`, pass17_tests / _batch_tests / _death_tests → `routine_tests` / `routine_batch_tests` / `routine_death_tests`, pass20 → `relations_block`, pass21 → `relation_caps`. Functions `pass4_sections` / `pass5_sections` → `seasons_forage_sections` / `meals_firewood_sections`; runner/summary fields `pass11/12/13/20` → `fission_block` / `adult_work_block` / `area_floor_block` / `relations_block`.
- **Rust test names:** bridge `pass3_agent_snapshot_tests` → `agent_snapshot_tests`; households `select_departure_p18` → `select_departure_reference` (+ its two tests); routine places `choose_p17` → `choose_reference` (+ its test); firewood `…matches_pass6_without_weights` → `…matches_the_unweighted_burn`; three probe tick-zero tests named by module instead of pass.
- **GDScript files:** `activity_icons_p11.gd` → `activity_icons.gd`, `agent_frame_choices_p10.gd` → `agent_frame_choices.gd`, `agent_selection_index_p10.gd` → `agent_selection_index.gd`, `art_declump_p12.gd` → `declump_offsets.gd` (class `ArtDeclumpP12` → `DeclumpOffsets`), `art_walker_p13b.gd` → `walker_offsets.gd` (`ArtWalkerP13B` → `WalkerOffsets`), `porter_presentation_p12.gd` → `porter_presentation.gd`.
- **GDScript identifiers:** `forced_p13` → `forced_write`, `inspector_p13` → `inspector_node`, `set/clear_hovered_person_p12` → `set/clear_hovered_person`, `set/get_perf_candidate_p10/p12/p13/p13b` → `set/get_allocation_cache` / `declump_kernel` / `write_skip` / `walker_kernel`; tooling env vars → `WORLDSIM_DECLUMP_KERNEL`, `WORLDSIM_WALKER_KERNEL`, `WORLDSIM_ACTIVITY_OUT`, `WORLDSIM_WRITE_SKIP`; log lines de-numbered.
- **Assets:** `assets/art/atlas/carry_p6.json` / `carry_p6_body.png` / `carry_p6_props.png` → `carry_atlas.json` / `carry_body.png` / `carry_props.png` (re-imported; the `.import` files are gitignored).
- Comments naming the deleted generator `tools/revitalize/exp_table_p13.py` now say it is at tag `v7-final-sim`. Test files under `rust/crates/sim-test/tests/` are renamed by domain in 8.4.

## D25 — 8.3: dead rule fields removed
Analysis of all 208 rule fields (27 types): `docs/v8/p3/8_3_candidates.md`. Removed are the fields that meet all three conditions of 8.3 and are not printed by the probe (removing a printed field changes the probe summary, so identity would not hold). Each removal also deletes the field's line from the four `sim-core/tests/data/v7_final_rules_seed*.txt` dumps.
- `BirthRule::Provisioned.sustain.capacity_weighted` = true — `food_economy::supply_per_day` matched `sustain: Some(SustainRule { capacity_weighted: true, .. })`; now `sustain: Some(_)` (the unweighted count stays for `Legacy` and `sustain: None`).
- `StockTargetRule.winter_prep` = true — `food_economy::stock_target`: the `(false, …)` arm (no winter shortfall) removed; the seasons check stays.
- `FirewoodRule.winter_prep` = true — `firewood::wood_target`: the no-winter-prep arm removed; two test literals (which set `true`) and a test helper's `rule.winter_prep &&` dropped.
- `FirewoodRule.rain_wet_burn` = true — no reader since 7.2 (it acted only on intervention rain); two inert test literals dropped.
- `FissionPressure.reset_on_split` = true — `fission.rs`: the parent's P is always reset after a split; one test literal (`true`) dropped.
- `RelationsRule.avoid_gather` = true — `agent_decision.rs`: the avoid list applies whenever a relations rule is present (`is_some_and(avoid_gather)` → `is_some()`); a test assert dropped.
- `RelationsRule.exchange_first` = true — `relations.rs`: the aid-before-exchange arm ("b. Exchange" after aid) deleted and the exchange-first block made unconditional; a test assert dropped.
- Kept although dead in the baseline, because the probe prints them (removal would change the summary): `ExposureRule.use_hardiness`, `FissionRule.stock_share`, `FirewoodRule.{winter_night_burn_permille, windfall_over_snow, regional_wood, rain_soak_permille}`, `FissionRule.crowding` (with `hazard`). Kept because a test literal sets the other value (strictly condition 2 fails, although those tests never reach the branch): `FamilyRule.elder_partner_follows`, `AgingRule.elder_evening_tether`, `FirewoodRule.{claims_bounded_by_amount, winter_regen_units_per_day, winter_deadwood_cap, floor_separate}`. Not touched: zero-sentinel arms of numeric fields (`quarrel_cap_per_pair_day == 0`, …) — the fields are live values. All are listed in the candidates file for the next probe-schema revision.

## D26 — 8.4: tests by domain
- The 108 kept `rust/crates/sim-test/tests/harness_*.rs` files became 68 domain files (`agents_*`, `settlement_*`, `family_*`, `relations_*`, `chronicle_*`, `engine_*`, `determinism.rs`). A file with one source is a `git mv`; a merged file holds each source verbatim as `mod <name> { … }`, so helpers stay private to their source and no test body changed (test paths gain the module prefix). Support modules: `pass17_family_support/` → `family_support/`, `pass17_routine_support/` → `routine_support/`, declared once at the top of the merged file.
- Domains: social interaction and the relations rule are `relations_*`; memory, combat, needs, death and routine are agent behaviour (`agents_*`); construction, resources, stockpiles, food, seasons, firewood and fission are `settlement_*`; influence, the causal log, the bridge surface, keyed draws, worldgen and tallies are `engine_*`; the three determinism suites are `determinism.rs`.
- `sim_test::baseline_engine(seed)` already existed (8.1a). New test: `determinism.rs` `baseline::baseline_runs_are_identical_tick_for_tick_and_seeds_differ` (two baseline engines per seed 0 and 1, fingerprints equal every hour for a day, the fingerprint changes in ≥ 18 of the 24 hours, seeds 0 and 1 differ).
- Coverage (6.5 table, inventory §3.2): every live system still has a kept test after 8.1b — the two R3 ports the table flagged (`children_grow_up` for MaturationSystem, `old_age_death_and_replacement` for AgingSystem) were ported, so no other minimal test was needed.
- Mapping (new ← old, `harness_` dropped):
  - `engine_influence` ← phase2
  - `engine_influence_pipeline` ← phase2_substantial
  - `engine_influence_channels` ← t7_10_a_warmth_wiring, t7_10_b_light_shadowcast_wiring, t7_10_b1_light_viz_toggle, t7_10_c_noise_linear_wiring, t7_10_d_danger_linear_wiring, t7_10_e_spiritual_bfs_wiring, t7_10_f_beauty_bfs_wiring
  - `engine_bridge_ffi` ← phase2_ffi
  - `engine_bridge_postverify` ← next_a_postverify
  - `engine_tick_scaffold` ← t7_9_a_scaffold
  - `engine_causal_events` ← p3_alpha_event_recording
  - `engine_causal_chain` ← p3_beta_causal_chain
  - `engine_causal_ffi` ← p3_gamma_1_ffi
  - `engine_rng` ← revitalize_pass9_rng
  - `engine_worldgen` ← revitalize_worldgen, revitalize_pass10_variety, revitalize_pass12_world
  - `engine_tallies` ← revitalize_tallies
  - `determinism` ← phase2_determinism, a15_controlled_determinism, fix_d_memory_arm
  - `agents_core` ← p4_alpha_agent_core, revitalize_pass16_temperament
  - `agents_movement` ← p4_beta_movement, s16_beta_movement
  - `agents_snapshot` ← p4_gamma_rendering, p11_alpha_agent_renderer, carry_viz_2_5b
  - `agents_viz` ← s16_epsilon_state_viz, show_agent_needs_bars, s16_gamma_visual
  - `agents_inspector` ← p14_gamma_click_inspector
  - `agents_needs` ← p5_alpha_agent_id_hunger, s16_delta_need_stagger
  - `agents_decision` ← p5_beta_decision
  - `agents_seek` ← s16_alpha_seektarget
  - `agents_sleep` ← p5_gamma_sleep_daynight_chronicle
  - `agents_preemption` ← revitalize_survival_fix, revitalize_pass5_preempt
  - `agents_rhythm` ← revitalize_pass3_rhythm
  - `agents_aging` ← revitalize_pass4_aging, revitalize_pass6_elders
  - `agents_exposure` ← revitalize_pass10_exposure, revitalize_pass11_exposure
  - `agents_newborn` ← newborn_mobile
  - `agents_starvation` ← starvation_death
  - `agents_death_viz` ← show_death_visual
  - `agents_memory_components` ← p8_alpha_memory_components
  - `agents_memory_system` ← p8_beta_memory_system
  - `agents_memory_ui` ← p8_delta_memory_ui
  - `agents_body_health` ← p9_alpha_body_health
  - `agents_combat` ← p9_beta_combat_system
  - `agents_combat_ui` ← p9_delta_combat_ui
  - `agents_routine` ← revitalize_pass17_routine, revitalize_pass17_routine_edges, revitalize_pass17_routine_invitations
  - `relations_social_components` ← p7_alpha_social_components
  - `relations_social_system` ← p7_beta_social_system
  - `relations_social_ui` ← p7_delta_social_ui
  - `relations_social_freeze_fix` ← s16_zeta_social_freeze_fix
  - `relations_rule` ← revitalize_pass20_relations
  - `relations_caps` ← revitalize_pass21_relations
  - `family_households` ← revitalize_pass17_family, revitalize_pass17_family_beds, revitalize_pass17_family_births, revitalize_pass17_family_fission
  - `settlement_data` ← p10_alpha_settlement
  - `settlement_system` ← p10_beta_settlement_system
  - `settlement_migration` ← p10_gamma_settlement_migration, settlement_migration_unfreeze
  - `settlement_membership` ← membership_belonging
  - `settlement_hud_regressions` ← settlement_marker_fixed, settlements_zero_regression
  - `settlement_construction_components` ← p6_alpha_construction_components
  - `settlement_construction` ← p6_beta_construction_system
  - `settlement_construction_render` ← p12_beta2_construction_sites, p12_gamma_settlement_furniture
  - `settlement_village` ← revitalize_pass3_village, revitalize_loop
  - `settlement_resource_substrate` ← s16_alpha0_resource_substrate
  - `settlement_resource_scarcity` ← resource_scarcity
  - `settlement_resource_depletion` ← show_resource_depletion
  - `settlement_gather` ← resource_gather_pickup
  - `settlement_inventory` ← inventory_2_1_data
  - `settlement_stockpile` ← resource_deposit_to_stockpile, stockpile_consume, settlement_food_stock
  - `settlement_food` ← revitalize_pass3_food, revitalize_pass4_forage, revitalize_pass5_meals
  - `settlement_seasons` ← revitalize_pass4_seasons, revitalize_pass6_winter, revitalize_pass7_winter, revitalize_pass8_winter
  - `settlement_firewood` ← revitalize_pass5_firewood, revitalize_pass6_urgent
  - `settlement_fission` ← revitalize_pass11_fission, revitalize_pass12_fission, revitalize_pass13_fission
  - `chronicle_construction` ← p6_gamma_construction_chronicle
  - `chronicle_social` ← p7_gamma_social_chronicle
  - `chronicle_memory` ← p8_gamma_memory_chronicle
  - `chronicle_combat` ← p9_gamma_combat_chronicle
  - `chronicle_settlement` ← p10_gamma_settlement_chronicle
  - `chronicle_rebuild` ← revitalize_pass19_chronicle, revitalize_pass19_chronicle_events
