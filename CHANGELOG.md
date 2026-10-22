# Changelog

All notable changes per pass. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Older history (pre-V7, V7, revitalize passes 1–21, art passes 1–13b, world-model
WM-1–6) is summarised in [docs/HISTORY.md](docs/HISTORY.md); the full record is at
tags `v7-final-sim` and `v7-final-art`.

## M2 — 2026-10-22

Things and making (`docs/v8/m2/M2_GOAL.md`): stone and flint as items, workshops, bills, work priorities, skills and quality, stone axes as tools, and firewood through the job layer — the first piece taken out of the decision tick. **The colony's behaviour changes**; the `baseline` world is bit-identical (identity PASS at every Part). Evaluation `docs/v8/m2/M2_EVALUATION.md` (seeds 300–331: E1, E3, E5, E6, E9, E10 PASS; E2, E4, E7, E8 FAIL), report `docs/v8/m2/M2_REPORT.md`. Benchmark ledger `docs/perf/PERF_LEDGER.md` blocks "base M1.5" and "M2" (N = 1,000: 1.71 ms; ceilings 4/4 PASS; the decision system 82.7 % → 80.0 % of the tick).

### Added
- ADR 0018 (art direction: illustrated 2D top-down at 64-px tiles, concept 05 ink-and-tone, cut-out people, generated environment); VISION §6 rewritten for it; ROADMAP rows ART-TRIAL, ART-CONCEPT, ART-TRIAL2, M-ART2, M-PERF; HISTORY and `docs/archive/INDEX.md` list the art branches.
- ADR 0019 (items, bills and the economic loop), ADR 0020 (skills and quality tables).
- Items with material and quality: `ItemKind` Log, Firewood, Stone, Flint, FlintBlade, StoneAxe, Hide; `ItemSpec { kind, material, quality }`; stacks merge per (kind, material, quality / 10); per-kind stack maxima and carry loads.
- The colony's material table (`sim_core::colony::materials`): granite, limestone, flint, chert and oak embedded from `data/material/**` and integerized once into `MaterialStats` (‰); `material::parse_ron_str`.
- Resource nodes (`ResourceNode`, `NodeKind`): boulders on 25 % of hills and 2 % of plains, flint outcrops 1 per 40 river-bank tiles (`STREAM_BOULDER`, `STREAM_OUTCROP`); `Designation::Quarry`, `Command::DesignateQuarry`, `JobKind::Quarry` (batches of 4).
- Per-kind item ledger (`jobs::invariants::ledger_of`: created = ground + carried + held + delivered + shelved + consumed) with craft transforms (`craft_start`/`craft_end`, `crafted`, `craft_inputs`) and `burned`.
- Workshops (`BlueprintKind` KnappingSpot 1×1 Stone 4, WoodBench 2×1 Log 6 + Stone 2, DryingRack 2×1 Log 4; `MaterialNeeds` with two lines; `Workshop` with work tile, shelf and bills), recipes (`data/colony/recipes.ron`: knap blade, haft axe, split firewood), bills (`Make(n)`/`Maintain(n)`, suspend, order), `BillSystem` (123, hourly), `JobKind::Supply` and `JobKind::Craft`; commands `AddBill`, `RemoveBill`, `SetBillMode`, `MoveBill`, `SetBillSuspended` (logged and replayed).
- Work priorities (`WorkPriorities`, `Command::SetWorkPriority`; 0 never … 4; default 3, elders' quarrying 0); `WorkType` Craft > Build > Quarry > Chop > Split > Haul; assignment in passes by priority level.
- Skills (`Skills`, `SkillKind` knap, woodwork, chop, quarry, build), `XP_TABLE`, `SPEED_PERMILLE`, `QUALITY_BONUS_TABLE` (`STREAM_QUALITY`), material and temperament terms; stone axes as `Tool` (picked up within 24 tiles, `AXE_PERMILLE`, worn out after 200 chops into a stone).
- Firewood through jobs: `JobKind::Gather` (deadwood → firewood items, `WorkType::Split`), `HearthFeedSystem` (146, hourly: tonight's burn from firewood items into the settlement's wood), the wood bench's auto firewood bill.
- Bridge (`ffi/economy.rs`): `cmd_designate_quarry`, `cmd_add_bill`, `cmd_remove_bill`, `cmd_set_bill_mode`, `cmd_move_bill`, `cmd_set_bill_suspended`, `cmd_set_work_priority`, `get_workshops`, `get_work_priorities`, `get_recipes`, `get_agent_economy`; `get_colony_overlay` gains `designation_kinds`, `node_xs/ys/kinds/left`, `item_qualities` (`docs/v8/bridge_api_m2.txt`, 58 items; snapshot schema version 2).
- Tests: `colony_items.rs`, `colony_workshops.rs`, `colony_skills.rs`, `colony_firewood.rs`, `engine_economy_ffi.rs`; the M2 command log replays byte-identically (`probe::colony` test).
- Probe: the `m2` block (timeline, firewood by day, priority agents, skills), `nearest_nodes`, `blueprint_spot`, script entries that wait for a workshop (`"after"`); `tools/v8/m2_u_script.json`; `docs/v8/m2/m2_score.py`.
- ARCHITECTURE §9 "Items and jobs"; skills: architecture procedures G–I (item kind, recipe, work type), the per-kind ledger pattern, lessons.

### Changed
- The decision system's firewood arms are off in the M2 colony (`jobs::hearth::decision_firewood`; `decision/system.rs` +2 −1); the baseline keeps them.
- A job holder at work on a craft yields only to a critical need (calibration, DECISIONS D13).
- Save `FORMAT_VERSION` 2.
- Gate step `bench`: a C3 ceiling miss now fails (`tools/v8/bench_ledger.py check`), no justification excuses it.
- Gate step 13 and `tools/v8/test_bridge_api.py` read `docs/v8/bridge_api_m2.txt`.
- The colony rule is `ColonyRule::M2` (`ColonyRule::economy`).
- Pause moved from P to Space (P opens the priority table); F3 opens a build menu; README updated.
- Code-review fixes (DECISIONS D21): a player's firewood bill takes over the auto-bill's craft; cancelled sites return their materials with their material (`NeedLine::material`); the decision's food "loaded" check reads the firewood rule itself again; workshops and priorities are refused without the economy (`NotYet`); unfinished workshops' work tiles are protected (`ColonyState::site_work_tiles`); resource nodes block blueprints.
- A finished workshop is not a V7 building: no `building_registry` entry and no `BuildingPlaced` (it read as a hearth; D22).
- The firewood shortfall counts only firewood a hearth can use (D19: stray loads had stopped gathering and three evaluation colonies froze in year 3).

## M1.5 — 2026-10-22

The colony's own calendar and demographics, and the simulation contract (ADR 0016) that every later pass is held to (`docs/v8/m1_5/M1_5_GOAL.md`). **The colony's behaviour changes** (calendar, ages, births); the `baseline` world is bit-identical (identity PASS). The colony fixtures are re-captured at each behaviour-changing commit (`/Users/rexxa/work/m1_5-data/{p1,p4,p6}/fixtures/`). Evaluation `docs/v8/m1_5/M1_5_EVALUATION.md`, report `docs/v8/m1_5/M1_5_REPORT.md`; benchmark ledger `docs/perf/PERF_LEDGER.md` (N = 1,000: 1.79 ms; ceilings 4/4 PASS). Gate 25/25 green at `cdd4a775` (`v8/main`).

### Added
- ADR 0016 (the simulation contract C1–C5 and its gate steps), ADR 0017 (calendar and demographics); `AGENTS.md` points at ADR 0016.
- `CalendarRule` (`WorldRules.calendar`): baseline 7-day seasons (unchanged), colony 6-day seasons and a 24-day year; every derived length is a method.
- `BirthRule::Fertility` and `FertilitySystem` (priority 150, daily at 12:00): conception 33 ‰ a day for fertile (16–44), paired, fed women; `Pregnant { conceived_tick, due_tick, father }`, 18 days of gestation, 48 days after a birth; births through the settlement's birth path (`SimResources.due_births`). `RuntimeSystem::tick_offset` (default 0).
- Colony ages in years: elders at 50, lifespans 62 ± 8 years clamped to 40–85 (`AgingRule.lifespan_bounds_ticks`), founders 16–40, childhood until 15, infants until 3 (`LifeStageRule.infant_ticks`, `Child::is_infant`).
- Bridge reads: `get_time_snapshot` `year_index`, `season_index`, `day_of_year`, `days_per_year`; `get_world_info` `ticks_per_year`; `get_agent_snapshot` columns `ages_years`, `infants`, `pregnant_days`.
- UI: the top bar reads `1년 차 · 봄 1일째`; the inspector shows ages in years (`INSPECTOR_AGE_FMT` "%d세"), the infant stage (`INSPECTOR_STAGE_INFANT`) and a pregnancy (`UI_INSPECT_PREGNANT_FMT`); deaths read "{age}세에 세상을 떠남"; `UI_TIME_YEAR_FMT`.
- Measurement: `worldsim_perf bench` / `bench-one` (`sim_test::bench`, `WorldGenConfig::colony_stress`, `tools/v8/bench_script.json`), `tools/v8/bench_ledger.py` (+ tests); the colony probe's `--chronicle-out`, `churn.json` (the decision seam counts job releases by need, kind, stage and need value in `ColonyState::release_counts`, measurement only) and a `demography` block (births per woman-year, population per year, old-age deaths, coming of age); colony runs up to 480 days.
- Spatial index `sim_core::spatial` (`ChunkIndex`, 16 × 16 chunks, ring queries, `nearest_where` with the job layer's `(distance, y, x)` order); `SimResources.spatial`; `JobAssignSystem` finds every candidate through it (colony fixtures byte-identical before and after).
- `docs/perf/PERF_LEDGER.md`, `tools/v8/bench.sh` (the gate step `bench`), `worldsim_perf bench-summarise`.
- Text renderer `worldsim_probe --ascii` (`sim_test::probe::ascii`, from the bridge's collectors), `docs/v8/snapshot_schema.md` (31 getters, schema version 1) with `check_docs_helpers.py schema` (check 10), `tools/v8/renderer_agreement.sh` + `.gd` (text renderer = Godot agent layer, seed 100, tick 900), `engine_ascii.rs`, `test_schema_docs.py`.
- Job churn remedies (`ColonyRule::M1_5`: `yield_margin_permille` 150, `finish_leg_tiles` 12; `jobs::seam::keeps_job`): a job yields only to an urgent hunger, thirst or fatigue and never to foraging, firewood or social; a near load is delivered first. M1 fixtures: finished / taken 0.21 → 0.98.
- Review fixes: a mother does not conceive on her due day, and the postpartum wait counts calendar days (48); the probe reports conceptions per at-risk woman-day; `nearest_where` visits only non-empty chunks by distance, `JobAssignSystem` skips searches found empty earlier in the tick (`JobAssignSystem` 18 → 6 µs per tick in the scripted N = 200 benchmark scene); the agents' index is built on demand (`SpatialIndex::agents_at`; `SpatialIndexSystem` removed).
- Save and load (C1): `sim_systems::save` (bincode, `FORMAT_VERSION` 1, sorted hash collections, exact hecs rebuild with the allocator's free list, system state via `RuntimeSystem::save_state`/`load_state`); bridge `save_world`, `load_world` (`docs/v8/bridge_api_m1_5.txt`, 47 items); F5 save / F9 load with toasts (`UI_SAVE_DONE`, `UI_LOAD_DONE`, `UI_SAVE_FAILED`, `UI_LOAD_FAILED`); `tools/v8/save_roundtrip.sh`, `tools/v8/save_load_check.gd`, `save_load.rs`.
- Code-review fixes (DECISIONS D18): a job holder that may not yield skips the decision for the tick (`jobs::skips_decision`, `JobHolders`; no more undone choices with per-tick side effects); a critical need releases even a near load; a due birth falls back to the mother's current settlement or waits; save test through a pregnancy and a queued birth; the engine clock is asserted equal to the calendar.
- Gate steps 22 `schema_docs`, 23 `renderer_agreement`, 24 `save_roundtrip`, 25 `bench` (`tools/v8/gate.sh`); AGENTS.md and the skills (`worldsim-architecture`: the contract rules, save registry; `worldsim-testing`: benchmark, ledger, renderer agreement, the 25-step table; lessons in all three); ARCHITECTURE §8 "Contract and benchmark"; `tools/v8/save_load_check.gd --shots`.
- Tests: `colony_demography.rs` (`calendar_derivations`, `ages_in_years`, `fertility_rate_matches_rule`, `no_births_without_partner_or_food`, `pregnancy_to_birth_household`).

### Changed
- The cancel order moved from F5 to F6 (F5 is save).
- Season readers take the day count from the calendar (`WorldRules::season_days`); the year of a tick is the calendar's year.
- Colony: `FirewoodRule.winter_prep_days` = one colony winter (calibration, DECISIONS D9); births by fertility, not by the provisioned cooldown (10 days no longer double the colony: 0 births in 10 days, 11–15 people after 3 years on the calibration seeds).
- The baseline rules dumps gain the off-valued fields `infant_ticks: None`, `lifespan_bounds_ticks: None` and the baseline calendar.

## M1-ART — 2026-10-21

The colony view drawn like the mock-up: real ground, a canopy that reads as forest, buildings and log piles you can read, people you can find (`docs/v8/m1art/ART_GOAL.md`, report `docs/v8/m1art/ART_REPORT.md`). No simulation change (`rust/` untouched; identity, colony determinism and replay pass).

### Added
- `tools/v8/gen_tiles.py` → `assets/v8/tiles.png` + `tiles.json`: one code-painted sheet (ground families, forest floor, worn earth with dither edges, hill, rock tops and cliffs, deep/shallow water with 15 bank masks and 4 inner corners, stockpile floor parts, 3 species × 3 sizes of 24×24 crowns and their shadow, stump, log piles, hut with walls/bedrolls/hearth, blueprint ghosts, lean-to, two hearth frames, layered people with stand/stride/sleep frames and halo, carried logs, tools, axe chop glyph, hover frame, order-bar icons, people palettes); `tools/v8/test_gen_tiles.py` (23 tests).
- Terrain: value-noise grass clusters on the world seed, forest floor around trees, an eroded camp clearing, the water autotile (`terrain_layer.gd`).
- Canopy chunks with y-ordered overlapping crowns, shadows (not at 1×) and stumps for a game day (`tree_layer.gd`); ground-level overlay pass for stockpile floors, log piles and blueprint ghosts with delivered piles; backed blueprint label and a 3-px bar (`overlay_layer.gd`); two-frame hearth fire, lean-to shelter (`building_layer.gd`).
- People: outline and halo layers, walking frames, west flip, logs across the shoulders, sleepers on the hut's bedrolls, warm light near a fire at night (`agent_layer.gd`).
- Order bar icons, gold active mode, macOS hint `UI_ORDER_HINT_IDLE_MAC` (en/ko); `tools/v8/capture.gd --m1-set` dumps on-screen positions per shot and adds `m1_10_morning_2x.png`; `tools/v8/m1_gui_test.gd --no-vsync`; `tools/v8/contact_sheet.py --columns` and blank cells.
- `docs/images/m1art_contact_sheet.png`; ADR 0015; `docs/v8/m1art/` (goal, checklist, decisions, report).

### Changed
- `chronicle_panel.gd` shows HutBuilt (kind 22); `UI_CAUSAL_PANEL_PLACEHOLDER` lost its dev marker "(γ-2-β)".
- `daylight_controller.gd`: the hearth glow is centred on the hearth (it sat half a tile off); the night curve is shared (`tint_at`, `night_factor_at`).
- `worldsim-godot` (craft rules, procedures 11–12, lessons), `worldsim-pitfalls` (lessons), `rendering-m1.md`, ARCHITECTURE §7, inventory rules.

### Removed
- `tools/v8/gen_temp_tiles.py` (540 lines) and `test_gen_temp_tiles.py` (renamed and rewritten as `gen_tiles.py` / `test_gen_tiles.py`); `assets/v8/temp/` (the placeholder sheet); `docs/images/m1_contact_sheet.png` (replaced).

## V8-PLAY — 2026-10-21

Screenshots of the current game for the owner and a one-command way to play any version (`docs/v8/play/PLAY_GOAL.md`, report `docs/v8/play/PLAY_REPORT.md`). No simulation or game code changed.

### Added
- `tools/play.sh [version] [--seed N] [--speed 1-4] [--no-build] [--headless-check]`: resolves Godot, plays `main`/`wip`/`m1`/`v8-0`/`v7`/any ref from its own worktree under `/Users/rexxa/work/new-world-play/`, builds and imports only when stale, logs to `/Users/rexxa/work/play-data/logs/`.
- `tools/play/WorldSim.command`, `tools/play/WorldSim-v7.command`: double-click launchers (Desktop symlinks).
- `tools/v8/test_play.sh` and gate step 21 `play_check` (exit codes 3/2/4; headless check of main, m1, v7).
- `tools/v8/capture.gd --m1-set` (the owner shot set P1–P9) and `tools/v8/contact_sheet.py` (+ `test_contact_sheet.py`).
- `docs/images/m1_contact_sheet.png` and five M1 shots; `docs/v8/play/` (goal, decisions, screen report, contact-sheet manifest, report).
- Skill lessons (`worldsim-godot`: chronicle `NOTABLE` gap, the editor binary loads the debug library, scan-then-reset captures; `worldsim-pitfalls`: bash 3.2 and Hangul after `$var`).

### Changed
- README "빌드와 실행": the desktop launcher first, then `tools/play.sh`, the manual commands last; a "지금 화면" line.
- `tools/v8/check_docs.sh`: requires a `docs/images/*_contact_sheet.png` linked from README.
- End-of-pass checklist (AGENTS.md, `worldsim-docs-upkeep`): item 7, the play check and the contact sheet.
- `worldsim-godot` procedure 4 (capture and contact sheet) and new procedure 10 (play a version); the gate has 21 steps.

## M1 — 2026-10-20

The first colony the player commands (`docs/v8/m1/M1_GOAL.md`, report `docs/v8/m1/M1_REPORT.md`). The game now runs the **colony** world (192 × 192, 10 founders); the frozen **baseline** world stays the V7 game and still passes identity (ADR 0014). Orders are recorded commands (ADR 0012); people carry them out as jobs that give way to every survival need (ADR 0013).

### Added
- Simulation (all gated by `rules.colony`, `None` in baseline):
  - `sim-core/src/colony/`: items (`ItemKind::Log`, `ItemStack`, `Carry`, the `ItemLedger`), trees, chop designations, stockpile zones, hut blueprints (`ColonySite`, `MaterialNeeds`), reservations, `Command` / `CommandResult` / `CommandRecord`, `Job` / `JobKind` / `JobStage`; `ColonyRule` and `WorldRules::colony`; `SimResources::colony`.
  - `sim-systems/src/commands.rs` (`apply_command`, `submit`) and `runtime/jobs/`: `JobAssignSystem` (124), `JobExecuteSystem` (127), the decision seam, cancel and release, merge and spill, invariants.
  - `BuildingKind::Hut`, `TargetKind::Job`, chronicle kind `HutBuilt` (22).
  - Colony worldgen (`WorldGenConfig::colony`, `worldgen/colony.rs`): area-scaled bushes and deadwood, 18 wild regions asked, about 10,000 trees, the founder adult check. New keyed streams `STREAM_TREE_LOGS`, `STREAM_COLONY_BUSH`, `STREAM_COLONY_WOOD`, `STREAM_COLONY_FOUNDER_FIX`.
- Bridge methods: `cmd_designate_chop`, `cmd_designate_chop_tiles`, `cmd_cancel_area`, `cmd_add_stockpile`, `cmd_remove_zone_area`, `cmd_place_blueprint`, `can_place_blueprint`, `get_trees`, `get_colony_overlay`, `get_colony_versions`. The agent snapshot gains the `jobs`, `carry_kinds` and `carry_counts` columns. API list `docs/v8/bridge_api_m1.txt`.
- GDScript:
  - the colony view `scripts/world/colony/`: terrain, trees, buildings, order overlay, people, order tool;
  - `scripts/ui/hud/order_bar.gd`;
  - `Locale.trf`;
  - 24 localization keys plus `BUILDING_HUT`.
- Tooling:
  - `worldsim_probe --scene colony [--commands FILE] [--commands-out DIR]`, `worldsim_perf natural --scene colony [--commands FILE]`, `tools/v8/u_script.json`;
  - `tools/v8/verify_v8_0.gd`, `gen_temp_tiles.py` → `assets/v8/temp/tiles.{png,json}`, `m1_gui_test.gd`, `test_locale_trf.gd`, `compare_chronicles.py`;
  - gate steps 17–20 (`m1_tests`, `colony_determinism`, `replay`, `uid_check`).
- Tests: `colony_commands.rs`, `colony_jobs.rs`, `colony_world.rs`, `engine_colony_ffi.rs`, probe and unit tests; `test_gen_temp_tiles.py`, `test_compare_chronicles.py`.
- Docs: ADRs 0012–0014; `docs/v8/m1/` (DECISIONS, base numbers, V8-0 screen check, evaluation, report); the target image `docs/images/v8_target_view_b.png`.

### Changed
- The live game: `build_game_engine` builds the colony world. The bridge feeds 12 ticks per second at 1× (a day in 120 s), at most 8 per frame. Keys 1–4 set 1×–4× (12–48 ticks/s), F1–F5 choose the order mode (ADR 0004 gains the tick definition).
- `decision/system.rs`: +26 −1 lines (the job seam; the memory bias-only Construction arm is off in the colony, D20).
- `ConstructionSystem`: colony sites wait for their logs; finished huts are registered through `register_dwelling`, the path planned shelters use. `BedAssignmentSystem` uses per-dwelling capacity (huts sleep 4). `despawn_agent` drops a worker's load.
- `camp::regions` checks "within `region_water_max` of water" with a precomputed grid (same predicate).
- Identity compares chronicle files with `compare_chronicles.py`: the kinds vocabulary may grow at its end.
- `.uid` files are committed (check 8 of `check_docs.sh`). `.gitignore` ignores evidence images under `docs/` except `docs/images/`.
- Baseline dumps gained `colony: None` (rules and worldgen).
- Part 6 review fixes:
  - in the colony, `ConstructionSystem` frees a `Seeking { ConstructionSite }` agent whose site is gone (cancelled or completed); before, it froze until it died;
  - a finished builder whose state already moved on ends its job with `jobs::cancel::finish_in_place`;
  - `put_items` debug-asserts that no item is left over.
- Part 6 view fixes:
  - the top bar counts huts as shelters and blueprints as sites, adds the log count, and renames 식량 → 비축 식량 and 열매 → 야생 열매;
  - the carried log is a light bundle above the head, tunics are brighter, the child's eyes sit above a skin row, elder hair is white;
  - blueprint progress bars sit under a "통나무 d/n · 작업 p%" label, and order toasts name what was ordered;
  - the clock shows 저녁 from 17:00 to 21:00;
  - `Home`/`H` centres the camera on the camp;
  - deadwood is quieter.

### Removed
- V7 renderers and helpers (21 scripts: terrain, nature, building, agent and world renderers and their helpers), `scripts/world/cold_audit.json`, `shaders/settlement_outline.gdshader`.
- Bridge methods `art_declump`, `art_walker_offsets` (`ffi/art_kernels.rs`).
- The influence overlay cycled with Space (part of the deleted `world_renderer.gd`).

## V8-0 — 2026-10-20

The clean slate for the V8 settlement sim. The simulation is unchanged
(32 worlds × 120 days: probe summaries, raw samples and chronicles identical
to `v7-final-art`); the only intended behaviour changes are that the god-hand
panel and the V7 overlays are gone from the game and the intervention methods
are gone from the bridge.

### Added
- `WorldRules::baseline(seed)` and `WorldGenConfig::baseline(seed)` (`rust/crates/sim-core/src/rules/baseline.rs`, `rust/crates/sim-systems/src/worldgen/mod.rs`): the live game's rules as one flat literal equal to V7's `game_v26`; `sim_test::baseline_engine(seed)`; probe scene `baseline` (alias `game_v26`).
- Tests: `sim-core/tests/baseline_matches_v7_final.rs` and `sim-systems/tests/baseline_worldgen_matches_v7_final.rs` (the baseline against the V7 dumps, seeds 0, 1, 7, 42); `sim-test/tests/determinism.rs` baseline tick-for-tick test; `tools/v8/test_bridge_api.py`, `tools/v8/test_gd_reach.py`.
- Tooling `tools/v8/`: `gate.sh` (16 steps), `identity.sh` + `compare_summaries.py` (32 worlds × 120 days), `smoke.sh`/`smoke.gd`, `capture.gd`, `perf.sh` + `perf_summary.py`, `bridge_api.py`, `gd_reach.py`, `inventory.py`, `locale_keys.py`, `locale_prune.py`, `system_table.py`, `check_docs.sh`, `worldsim_systems` binary (the live system list).
- Docs: `README.md`, `docs/VISION.md`, `docs/ROADMAP.md`, `docs/HISTORY.md`, `docs/ARCHITECTURE.md` (generated system table), ADRs `docs/decisions/0001`–`0011`, `docs/archive/INDEX.md`, this file; `AGENTS.md` rewritten (78 lines), `CLAUDE.md` = `@AGENTS.md` + 8 lines.
- Skills `.claude/skills/`: `worldsim-architecture`, `worldsim-godot`, `worldsim-testing`, `worldsim-pitfalls`, `worldsim-docs-upkeep` (with sourced lesson tables), plus `systematic-debugging` and `verification-before-completion` moved from `.agents/skills/`; `.agents/skills/<name>` are symlinks to them.

### Changed
- The `game_v8 … game_v26` rule chain and its scenes became the one baseline (`rules.rs` → `rules/`; `rules/mod.rs` 1,821 → 819 lines); the perf `quick` world runs under the baseline rules (`quick_stress_baseline_s15100_n500_side64_fed.json`, same fingerprint as V7 `stress --rules game_v26`).
- Seven dead rule fields removed (`capacity_weighted`, two `winter_prep`, `rain_wet_burn`, `reset_on_split`, `avoid_gather`, `exchange_first`; DECISIONS D25).
- Names: pass and version suffixes removed from Rust and GDScript identifiers and files (probe `pass*` modules renamed by content; `draw_v18` → `draw`; `*_p10…_p13b.gd` renamed; D24).
- Structure: `sim-bridge/src/ffi/world_node.rs` (4,364 lines) split into `node.rs`, `engine_build.rs`, `snapshot.rs`, `reads_agent.rs`, `reads_settlement.rs`, `causal.rs`; `sim-systems/.../decision/agent_decision.rs` (4,337) into `system.rs`, `targets.rs`, `needs.rs`, `forage.rs`, `memory_bias.rs`, `wood.rs`; GDScript laid out as `scripts/core`, `scripts/world`, `scripts/ui/hud`, `scripts/ui/panels`.
- Tests: 108 `sim-test/tests/harness_*.rs` files became 68 domain files (`agents_*`, `settlement_*`, `family_*`, `relations_*`, `chronicle_*`, `engine_*`, `determinism.rs`); tests of version deltas and frozen worlds deleted, the rest ported to the baseline.
- Localization source switched from Fluent to the category JSONs (`localization/<lang>/<category>.json`), pruned to the 156 keys the kept UI uses.
- Every `src/**/*.rs` has a `//!` header (systems: priority, interval, reads, writes, events); every `.gd` a purpose + bridge-methods comment block.
- `.claude/settings.json` committed as the neutral `{}` (the V7 harness hooks pointed at deleted scripts).

### Removed
- The god hand: GDScript panel and scene, bridge methods `apply_intervention` and `get_intervention_state`, `sim-engine/src/intervention.rs`, `sim-systems/src/runtime/intervention.rs` and intervention-only fields (ADR 0010).
- V7 overlays and panels outside the kept UI (activity badges and tooltips, village activity summary, settlement relation lines, seek/death/need-bar/activity-trail overlays, restless meter, founding-group renderer, feed ledger, status panel, zoom LOD controller): `scripts/ui` 53 → 8 files (plus 31 moved to `scripts/core`, `scripts/world`, `scripts/ui/hud`); `scripts/test` 222 files (33,947 lines) deleted.
- `docs/`: 17,280 files (2.36 GiB of pass records, evidence and boards) → 27 files (the new docs and the V8-0 records); the index of what was deleted and where it lives is `docs/archive/INDEX.md`.
- `tools/`: 895 → 25 files (V7 harness, revitalize, art pipeline, world-model tooling).
- `sim-test`: 202 → 110 files (world-model recorder, ablation scenes, pass scaffolding, version-delta tests); `sim-bridge` test fixtures of 135 MB (art kernels).
- Assets: 888 → 198 files (15.0 → 12.9 MiB); shaders 11 → 3; scenes 3 → 1; localization 47 → 8 files and 5,287 → 156 keys (the Fluent sources and the key-owner policy).
- Whole tree: 20,299 files / 2,533 MiB → 714 files / 18.5 MiB (at the gate commit `672fffef`). Everything removed is at tag `v7-final-art`.
