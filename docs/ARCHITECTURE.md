# WorldSim architecture

How the code is laid out and how data moves through it. Direction and goals:
[VISION.md](VISION.md). Rules for working in each area: the skills in
`.claude/skills/` (`worldsim-architecture`, `worldsim-godot`,
`worldsim-testing`). Every path named here exists (`tools/v8/check_docs.sh`
checks); the system table is generated (`python3 tools/v8/system_table.py --write`).

## 1. Crates and folders

The simulation is Rust; Godot 4.6 draws it and takes input through one
GDExtension class. Dependency direction (no cycles):

```
sim-core  ←  sim-engine  ←  sim-systems  ←  sim-bridge  (GDExtension cdylib, Godot loads it)
    ↑             ↑               ↑               ↑
    └─────────────┴───────────────┴───────────────┴──  sim-test (probe, perf, tests; never shipped)
```

| crate | path | role |
|---|---|---|
| sim-core | `rust/crates/sim-core/src/` | data only: ECS components (`components/`), rules (`rules/`: `WorldRules`, one file per sub-rule group, `baseline.rs`), keyed RNG (`rng.rs`), per-world hidden parameters (`variety.rs`), causal log (`causal/`), chronicle (`chronicle.rs`), influence grid (`influence/`), tile grid (`tile/`), material model (`material/`, data for M2), the M1 colony model (`colony/`: items and the log ledger, trees, designations, zones, blueprints, reservations, commands, jobs), the spatial index (`spatial.rs`), the M3 cost map, regions and A* (`nav.rs`) and room detection (`rooms.rs`), the M4 knowledge model (`colony/knowledge.rs`; data `data/colony/knowledge.ron`, `KnowledgeRule` in `rules/knowledge.rs`) |
| sim-engine | `rust/crates/sim-engine/src/lib.rs` | `SimEngine` (hecs `World` + `SimResources` + the priority-sorted system list), the `RuntimeSystem` trait, `Tallies`, the tick loop and its profiler |
| sim-systems | `rust/crates/sim-systems/src/` | every system (`runtime/<domain>/`), their registration (`lib.rs` `register_default_runtime_systems`), the command entry point (`commands.rs`: `apply_command`, `submit`), the job layer (`runtime/jobs/`; apprenticeship in `jobs/teach.rs`), the M4 knowledge systems (`runtime/knowledge/`) and world generation (`worldgen/`: `WorldGenConfig::baseline`, `WorldGenConfig::colony` with `worldgen/colony.rs`, `build_world_with`) |
| sim-bridge | `rust/crates/sim-bridge/src/ffi/` | the Godot class `WorldSimNode` (`#[func]` methods), pure `collect_*` readers behind them, the live engine builder (`build_game_engine`, the colony world), M1 orders and colony reads (`colony.rs`), economy (`economy.rs`), space (`space.rs`), knowledge (`knowledge.rs`) |
| sim-test | `rust/crates/sim-test/` | `worldsim_probe` (headless measurement, identity; `--scene colony` with `--commands` for scripted orders and replay), `worldsim_perf` (tick timing), `worldsim_systems` (the live system list), integration tests in `tests/` |

Other folders: `scenes/main.tscn` (the only scene), `scripts/` (GDScript,
section 6), `assets/` (fonts, theme, the HUD icon atlas, `v8art/`: the adopted art with `manifest.json`, `bindings.json`, `render_params.json` — section 14), `shaders/`,
`localization/` (section 7), `tools/v8/` (gate, identity, perf, smoke,
capture, contact sheet, inventory, docs checks), `tools/play.sh` and `tools/play/` (play any version
in its own worktree; the owner's double-click launchers), `docs/` (this file and the owner docs).

## 2. Data flow

```
Godot frame ──► WorldSimNode::process(delta)            (rust/crates/sim-bridge/src/ffi/node.rs)
                 accumulator += delta × sim_speed
                 while accumulator ≥ 1/12 s (max 8 per frame): engine.tick()
                      │
                      ▼
               SimEngine::tick()                          (rust/crates/sim-engine/src/lib.rs)
                 for each system, lowest priority first,
                 when current_tick % interval == 0:
                   system.tick(&mut world, &mut resources)
                 then advance tick and day clock
                      │
                      ▼
GDScript layers / panels  ──► SimReads.read(world_sim, "get_…")   (scripts/core/sim_reads.gd)
                 ──► #[func] get_…  ──► pure collect_…(&World, &SimResources)
                 ──► Dictionary of Packed*Array columns (one row per agent / building / …)
```

- The simulation runs at a fixed 12 ticks per second of real time at 1×
  (M1, ADR 0004: `TICKS_PER_SECOND_AT_1X`); rendering runs at the display
  rate. Each frame `WorldSimNode::process`
  (`rust/crates/sim-bridge/src/ffi/node.rs`) adds `delta × sim_speed` to an
  accumulator and runs `engine.tick()` while it holds ≥ 1/12 s, at most 8
  ticks per frame (`MAX_TICKS_PER_FRAME`). If more than 8 ticks' worth is
  left after a frame, the surplus is dropped (the accumulator is reset to
  exactly one step, 1/12 s, so the next frame runs at least one tick): the
  game then runs slower than the chosen speed and never catches up later. Speed s needs at least 1.5 × s frames per second
  (4×: 6 fps).
- Speed: in `scripts/world/camera_controller.gd` the keys `1`/`2`/`3`/`4`
  set 1×/2×/3×/4× = 12/24/36/48 ticks per real second, so a simulated day
  lasts 120/60/40/30 s (`seconds_per_day`); `P` pauses by disabling the
  WorldSim node's processing; the top bar shows the speed and the pause.
  `Home`/`H` centres the camera on the camp's hearth.
  `set_sim_speed` scales only the time fed to the accumulator and clamps to
  0–4× (0 also stops the clock); it never changes the tick length.
- GDScript never writes simulation state. The only writes from Godot are
  bridge commands: the M1 orders (`cmd_*`, below), `on_building_placed`,
  `reset_world`, `reset_world_stress`, `save_world`, `load_world` (M1.5), `set_sim_speed` and the tooling
  `advance_ticks` (`SimReads.MUTATORS` holds all but `set_sim_speed`, which
  `scripts/world/camera_controller.gd` calls directly on the number keys).
- Calendar (M1.5, ADR 0017): one tick is one simulated minute; a day is
  1,440 ticks (120 s of real time at 1×). `WorldRules.calendar`
  (`CalendarRule`, `rust/crates/sim-core/src/rules/calendar.rs`) holds
  `ticks_per_day`, `days_per_season` and `seasons_per_year`; every derived
  length is a method (`days_per_year`, `ticks_per_year`, `years_to_ticks`,
  `days_to_ticks`, `whole_years`, `date_of_day`). The colony has 6-day
  seasons and a 24-day year (34,560 ticks = 48 minutes at 1×; a season 12
  minutes); the baseline keeps V7's 7-day seasons and 28-day year (40,320
  ticks). Season readers take the day count from the calendar
  (`WorldRules::season_days`); `SeasonRule::days_per_season` must equal it
  (`WorldRules::calendar_consistent`).
- The displayed date and time count from the display clock
  `current_tick + clock_offset_ticks` (480: tick 0 is 08:00 of day 1).
  `get_time_snapshot` gives `day_index = (tick + 480) / 1440`, the zero-based
  `year_index` and `season` (= `season_index`, 0–3 = spring … winter, the
  `HUD_SEASON_*` keys), the one-based `day_of_season` and `day_of_year`,
  `days_per_season`, `days_per_year`, `time_of_day` and `winter_severity`.
  The top bar reads `3년 차 · 봄 2일째` (`UI_TIME_YEAR_FMT` + `HUD_SEASON_DAY_FMT`).
- Systems run on the raw tick: a system is due when
  `current_tick % interval == tick_offset % interval`
  (`RuntimeSystem::tick_offset`, default 0, so a daily system without an
  offset runs at 08:00 display time). `FertilitySystem` uses the offset to
  run at 12:00.
- Start and pause: the game starts at 1×, not paused. `P` toggles the pause
  and keeps the chosen speed; a speed key pressed while paused changes the
  speed and stays paused.
- Walking: a person steps one tile every `AgentRules.move_interval_ticks`
  ticks: 3 in the colony (4 tiles per real second at 1×), 8 in baseline.
  The baseline steps by the sign of the difference toward its target (V7).
  The colony (M3, ADR 0021) follows a stored A* path (`SimResources.nav.paths`)
  over the cost map: river, mountain, undug hill and walls are impassable, a
  door costs 12 against 10, a diagonal 14/10, no corner cutting; idle and play
  steps are refused into impassable tiles. A decision target out of reach
  moves to the nearest reachable tile of its kind or the walk ends; a job
  target out of reach cancels the job `Unreachable` (section 10).
- Ages (colony): years of the calendar, computed from `Life::born_tick`
  (nothing increments): elders at 50, lifespans 62 ± 8 years (clamped 40–85),
  founders 16–40, children until 15 (`LifeStageRule.child_ticks`), infants
  until 3 (`Child::is_infant`, derived). The agent snapshot's `ages_years`
  column feeds the inspector ("어른 · 24세"); `ages_days` stays for tooling.
  Births: `BirthRule::Fertility` (`FertilitySystem`, ADR 0017) — a daily keyed
  conception chance for fertile, paired, fed women, an 18-day `Pregnant`
  component (`pregnant_days` column), the birth through `SettlementSystem`'s
  birth path via `SimResources.due_births`. The baseline keeps V7's ages in
  days and `BirthRule::Provisioned`.

### Orders, jobs and items (M1, colony world only)

```
order tool (scripts/world/colony/order_tool.gd) ──► WorldSimNode.cmd_*(…)       (ffi/node.rs → ffi/colony.rs)
   ──► commands::submit(engine, Command)        (sim-systems/src/commands.rs)
         apply_command checks and applies it at once: designations, zones, blueprint sites,
         cancels; returns {accepted, count, reason}; appends CommandRecord{tick = next tick}
         to SimResources.colony.command_log (the replay log, ADR 0012)
   ──► JobAssignSystem 124                       (runtime/jobs/assign.rs)
         idle or leisured adults, AgentId order: Build (deliver, then build) > Chop > Haul;
         each job reserves its tree / stack / cell / site (SimResources.colony.reservations)
   ──► AgentDecisionSystem 125 + jobs::seam      (runtime/decision/system.rs, runtime/jobs/seam.rs)
         M1.5: a job holder runs the decision only with an urgent hunger, thirst or fatigue
         (threshold + ColonyRule.yield_margin_permille) or a critical one, and not while carrying
         a load within finish_leg_tiles of its destination; then any non-idle choice cancels
         the job (CancelReason::Needs) and releases its reservations
   ──► JobExecuteSystem 127                      (runtime/jobs/execute.rs)
         walk, chop (logs drop as an ItemStack), pick up into Carry, put down (merge, spill),
         deliver into the site's MaterialNeeds; ConstructionSystem 133 finishes the hut
   ──► reads: get_trees, get_colony_overlay, get_colony_versions, the agent snapshot's
         jobs / carry_kinds / carry_counts columns ──► scripts/world/colony/*.gd
```

- A felled tree creates logs; from then on every log is in exactly one
  place: a ground `ItemStack`, a `Carry`, a site's delivered count, or
  consumed by a finished hut. `ItemLedger` (`SimResources.colony.item_ledger`)
  counts created and consumed logs, and `jobs::invariants::check` asserts
  created = ground + carried + delivered + consumed (with the reservation and
  stack invariants) every tick in debug builds; the colony tests check it too.
- All of it is gated on `rules.colony` (`ColonyRule`, `WorldRules::colony`):
  in the baseline world every job system returns at once, so the frozen
  baseline stays identical (ADR 0014).
- The decision system gains only the seam: +26 −1 lines in M1 and +4 −1 in M1.5 (`git diff --numstat`) that call
  `jobs::seam::{holders, hold_for_decision, skips_decision, settle_after_decision}` and the
  compile-forced `TargetKind::Job` arms; the memory "bias-only Construction"
  arm is off in the colony (M1 D20).

## 3. Live systems

The live engine is `build_game_engine(seed)` = `build_world_with(&WorldGenConfig::colony(seed), WorldRules::colony(seed))` since M1 (ADR 0014; the frozen identity world is the same call with `baseline`),
which registers the colony systems below (`register_world_runtime_systems`). Lower
priority runs first; a system runs on ticks where `tick % interval == 0`.
Reads/writes come from each file's `//!` header.

<!-- system-table:begin (generated by tools/v8/system_table.py; do not edit) -->

| # | system | priority | interval (ticks) | file | reads | writes |
|---|---|---|---|---|---|---|
| 1 | `BuildingStampSystem` | 90 | 1 | `rust/crates/sim-systems/src/runtime/influence/building_stamp.rs` | `SimResources.building_event_queue`, `influence_grid` (dimensions), `current_tick` | `SimResources.building_event_queue` (drained), `influence_grid.dirty_regions`, `building_registry`, `causal_log` |
| 2 | `InfluenceUpdateSystem` | 100 | 1 | `rust/crates/sim-systems/src/runtime/influence/update.rs` | `SimResources.influence_grid` (`current`, `dirty_regions`), `tile_grid`, `material_blocking_cache`, `causal_log` (latest `StampDirty`) | `SimResources.influence_grid` (`pending`, drained `dirty_regions`, buffer swap), `causal_log` |
| 3 | `AgentInfluenceSampleSystem` | 110 | 1 | `rust/crates/sim-systems/src/runtime/influence/agent_sample.rs` | `Position`; `SimResources.influence_grid` (current buffer) | `InfluenceSample` |
| 4 | `AgentMovementSystem` | 120 | 1 | `rust/crates/sim-systems/src/runtime/agent/movement.rs` | `AgentState`, `SeekTarget`, `Agent`, `Child`, `Elder`, `Inventory`, `Lineage`, `Founding`, `Temperament`, `PorterTrip`, `Activity`; `SimResources.settlements`, `tile_grid`, `rules`, `nav` (colony), `food_tiles`, `water_tiles`, `sleep_tiles`, `bed_of` | `Position`, `MovementRng`, `SeekTarget` (colony: an unreachable need target moves to the nearest reachable one); `SimResources.haulers`, `nav` (paths, blocked goals, the map when missing) |
| 5 | `DraftSystem` | 121 | 1 | `rust/crates/sim-systems/src/runtime/wildlife/draft.rs` | `Agent`, `Position`, `Drafted`, `Tool`, `Hunger`, `Thirst`; `SimResources.colony` (wildlife, fights, ground items), `nav`, `rules` | `Drafted` (order, arming; removed at a critical need), `AgentState`, `SeekTarget`, `Tool` (picked up); `SimResources.colony` (fights, reservations, ground items, wildlife tallies, story cost), `chronicle` |
| 6 | `StewardDefenceSystem` | 122 | 1 | `rust/crates/sim-systems/src/runtime/steward/defence.rs` | people, health, weapons, wildlife and defence policy. | ordinary draft commands, owned defender ids and quiet-window state. |
| 7 | `BillSystem` | 123 | 60 | `rust/crates/sim-systems/src/runtime/jobs/bills.rs` | `Job`, `Carry`; `SimResources.colony` (workshops and bills, ground items, zones, reservations), `rules.colony` | `SimResources.colony.workshops` (active bill, shelf), ground items (spilled shelf items), `knife_wear` (M5: reset without a shelved blade) |
| 8 | `FleeSystem` | 123 | 3 | `rust/crates/sim-systems/src/runtime/wildlife/flee.rs` | `Agent`, `Position`, `Drafted`, `Fleeing`, `Hunger`, `Thirst`, `Job`; `SimResources.colony` (wildlife, rooms), `nav`, `building_kinds` (the hearth), `rules` | `Fleeing` (inserted, renewed, removed), `AgentState`, `SeekTarget`, `Job` and `Activity` (a flight cancels them); `SimResources.colony` (reservations and ground items through the cancel, wildlife tallies `flights`), `spatial.agents` |
| 9 | `JobAssignSystem` | 124 | 1 | `rust/crates/sim-systems/src/runtime/jobs/assign.rs` | `Agent`, `Position`, `AgentState`, `Job`, `Child`, `PorterTrip`, `Hunger`, `Thirst`, `Sleep`, `Temperament`; `SimResources.colony` (trees, designations, ground items, zones, reservations, footprints), `combat_pairs`, `rules.colony`; colony sites (`ColonySite`, `MaterialNeeds`, `ConstructionSite`) | `Job`, `SeekTarget`, `AgentState` (`Seeking { Job }`); `SimResources.colony.reservations`, `colony.tallies`, `spatial.colony` (rebuilt when the overlay changed) |
| 10 | `ColonyMindSystem` | 125 | 1 | `rust/crates/sim-systems/src/runtime/mind/system.rs` | ? | ? |
| 11 | `StewardSystem` | 125 | 360 | `rust/crates/sim-systems/src/runtime/steward/mod.rs` | colony, people, registered rules. | ordinary command effects and steward provenance. |
| 12 | `StaleSeekTargetSystem` | 126 | 1 | `rust/crates/sim-systems/src/runtime/resource_regen/mod.rs` | `AgentState`, `Position`, `SeekTarget`, food/water/sleep tile maps | `SeekTarget` (insert/remove), `AgentState` |
| 13 | `JobExecuteSystem` | 127 | 1 | `rust/crates/sim-systems/src/runtime/jobs/execute.rs` | `Agent`, `Job`, `AgentState`, `Position`, `Carry`; `SimResources.colony` (trees, designations, ground items, zones), `rules.colony`, `current_tick`; colony sites (`ColonySite`, `MaterialNeeds`, `ConstructionSite`) | `Job`, `Carry`, `AgentState`, `SeekTarget`, `MaterialNeeds` (delivered); `SimResources.colony` (trees, trees_version, designations, ground items, reservations, item_ledger, overlay_version, tallies) |
| 14 | `HungerDecaySystem` | 130 | 1 | `rust/crates/sim-systems/src/runtime/needs/hunger_decay.rs` | `Hunger`, `Cold`; `SimResources.rules.society.firewood` | `Hunger` |
| 15 | `ThirstDecaySystem` | 131 | 1 | `rust/crates/sim-systems/src/runtime/needs/thirst_decay.rs` | `Thirst`; `SimResources.colony.story` (M5 drought) | `Thirst` |
| 16 | `SleepDecaySystem` | 132 | 1 | `rust/crates/sim-systems/src/runtime/needs/sleep_decay.rs` | `Sleep`, `AgentState`, `Position`, `Sick` (M5); `SimResources.colony.rooms` (M3), `rules.colony.story` (M5) | `Sleep` |
| 17 | `ConstructionSystem` | 133 | 1 | `rust/crates/sim-systems/src/runtime/construction/construction_system.rs` | `Position`, `AgentState`, `ConstructionSite`, `MaterialNeeds`, `ColonySite`; `SimResources.tile_grid`, `causal_log` (`ConstructionStarted`), `site_owner`, `settlements`, `rules` | `ConstructionSite` (progress, despawn), `AgentState`; `SimResources.causal_log`, `building_registry`, `building_kinds`, `building_owner`, `site_owner`, sleep tiles, `colony` (footprints, item ledger, tallies) |
| 18 | `SocialInteractionSystem` | 134 | 1 | `rust/crates/sim-systems/src/runtime/social/social_interaction_system.rs` | `Agent`, `Position`, `AgentState`; `SimResources.causal_log` (`SocialInteractionStarted`), `interaction_progress` | `AgentState`, `Social`; `SimResources.interaction_progress`, `relationships` (familiarity), `causal_log`, `tallies` |
| 19 | `SocialDecaySystem` | 135 | 1 | `rust/crates/sim-systems/src/runtime/needs/social_decay.rs` | `Social` | `Social` |
| 20 | `MemorySystem` | 136 | 1 | `rust/crates/sim-systems/src/runtime/memory/memory_system.rs` | `Agent`; `SimResources.causal_log` (this tick's events, `MemoryRecalled`) | `Memory` |
| 21 | `CombatSystem` | 137 | 1 | `rust/crates/sim-systems/src/runtime/combat/combat_system.rs` | `Agent`, `Position`, `BodyHealth`; `SimResources.combat_pairs`, `causal_log` (`CombatStarted`), `settlements` | `BodyHealth`, `AgentState`, `Memory`, `Tool` (M5 spear wear); `SimResources.combat_pairs`, `combat_progress`, `relationships` (hostility), `causal_log`, `colony` (fights, wildlife, ground items, ledger; M5); deaths via `despawn_agent` |
| 22 | `SettlementSystem` | 138 | 1 | `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs` | `Agent`, `Position`, `Hunger`, `Thirst`, `Kin`, `Sex`, `Lineage`, `Temperament`, `Founding`; `SimResources.building_registry`, `building_kinds`, `causal_log`, `fission_log`, `rules`, `due_births` (M1.5, under `BirthRule::Fertility`) | `SimResources.settlements`, `building_owner`, `dissolutions`, `chronicle`, `causal_log`, `tallies`, `due_births` (drained); spawns newborns (needs, `Memory`, `Lineage`, `Kin`, `Sex`, `Child`, `Life`); mother's `Kin.last_birth_tick` |
| 23 | `StarvationSystem` | 139 | 1 | `rust/crates/sim-systems/src/runtime/survival/mod.rs` | `Agent`, `Position`, `Hunger`, `Thirst`, `Cold`, `ExposureClock`; `SimResources.rules.society.exposure` | `BodyHealth`; deaths via `despawn_agent` (despawn; `SimResources.settlements` rosters, `relationships`, social and combat maps, `bed_of`, `haulers`, `recent_deaths`, `chronicle`; M1: a worker's load and reservations in `colony`) |
| 24 | `ResourceRegenSystem` | 140 | 120 | `rust/crates/sim-systems/src/runtime/resource_regen/mod.rs` | `SimResources.*_source_max`, `wood_spot_area`, `wild_regions`, `settlements`, `rules`, `colony.story` (M5 drought) | `SimResources` food/water/sleep/wood tile maps, regen accumulators and cursors, `tallies`, `colony.story` (drought cost) |
| 25 | `StockpileDepositSystem` | 141 | 10 | `rust/crates/sim-systems/src/runtime/settlement/stockpile_deposit.rs` | `Agent`, `Position`, `PorterTrip`; `SimResources.settlements` (rosters, formation tile), `rules` | `Inventory`; `SimResources.settlements` (stockpile), `haulers`, `tallies` |
| 26 | `StockpileConsumeSystem` | 142 | 5 | `rust/crates/sim-systems/src/runtime/settlement/stockpile_consume.rs` | `Agent`, `Position`, `AgentState`, `Founding`, `Hunger`; `SimResources.settlements`, `rules.society` (`meals`, `relief_radius`) | `Hunger`; `SimResources.settlements` (stockpile food), `tallies` |
| 27 | `ShelterPlanningSystem` | 143 | 1 | `rust/crates/sim-systems/src/runtime/settlement/shelter_planning.rs` | `ConstructionSite`; `SimResources.settlements`, `building_registry`, `building_kinds`, `building_owner`, `tile_grid`, resource tile maps | spawns `ConstructionSite`; `SimResources.site_owner`, `shelter_plan_skips` |
| 28 | `MaturationSystem` | 144 | 60 | `rust/crates/sim-systems/src/runtime/agent/maturation.rs` | `Agent`, `Child`; `SimResources.current_tick` | `Child` (removed), `Hunger`, `Thirst`, `Sleep` (adult growth rates); `SimResources.chronicle` |
| 29 | `PairingSystem` | 144 | 1 | `rust/crates/sim-systems/src/runtime/agent/pairing.rs` | `Agent`, `Kin`, `Sex`, `Child`, `Elder`, `BodyHealth`; `SimResources.relationships` (familiarity), `settlements` | `Kin` (partner), `FamilyObservations`; `SimResources.chronicle` |
| 30 | `BedAssignmentSystem` | 145 | 1 | `rust/crates/sim-systems/src/runtime/agent/beds.rs` | `Agent`, `Kin`, `Child`, `BodyHealth` (households); `SimResources.building_owner`, `building_kinds`, `settlements`, `rules.society.family`, `rules.colony` | `SimResources.bed_of` |
| 31 | `AgingSystem` | 146 | 60 | `rust/crates/sim-systems/src/runtime/agent/aging.rs` | `Agent`, `Life`, `Position`, `Elder` | `Elder` (inserted); `SimResources.chronicle`; deaths via `despawn_agent` |
| 32 | `HearthFeedSystem` | 146 | 60 | `rust/crates/sim-systems/src/runtime/jobs/hearth.rs` | `SimResources.colony` (ground items, zones, reservations), `settlements` (formation tile, stockpile), `rules.society.firewood`, the clock | `SimResources.settlements` (stockpile wood), `colony` (ground items, item ledger `burned`) |
| 33 | `FireSystem` | 147 | 60 | `rust/crates/sim-systems/src/runtime/settlement/fire.rs` | `Agent`, `Founding`, `PorterTrip`, `Child`, `Elder`, `Hardiness`, `Hunger`, `Position`, `AgentState`, `Apparel`, `Lineage` (M5); `SimResources.settlements`, `rules`, `world_seed`, `colony.rooms` (M3), `colony.story` (M5) | `Cold`, `ExposureClock`, `BodyHealth`; `SimResources.fires`, `settlements` (stockpile wood), `tallies`, `colony.story` (snap cost, touched), `causal_log` (M5); deaths via `despawn_agent` |
| 34 | `FissionSystem` | 148 | 1 | `rust/crates/sim-systems/src/runtime/settlement/fission.rs` | `Agent`, `Position`, `Child`, `Elder`, `Lineage`, `Kin`, `Temperament`, `PorterTrip`, `AgentState`; `SimResources.settlements`, `wild_regions`, `tile_grid`, `rules` | `Founding`, `AgentState`, `SeekTarget`; `SimResources.settlements`, `building_kinds`, `building_event_queue`, `retired_hearths`, `fission_log`, `fission_checks`, `dissolutions`, `chronicle` |
| 35 | `RelationsSystem` | 149 | 1 | `rust/crates/sim-systems/src/runtime/settlement/relations.rs` | `Agent`, `Position`, `AgentState`, `Hunger`, `Temperament`, `Child`, `Elder`, `Founding`; `SimResources.settlements`, `rules` | `PorterTrip`, `Inventory`, `AgentState`, `SeekTarget`; `SimResources.settlement_relations`, `settlements` (stockpiles), `chronicle`, `tallies` |
| 36 | `FertilitySystem` | 150 | 1440 | `rust/crates/sim-systems/src/runtime/agent/fertility.rs` | `Agent`, `Sex`, `Kin`, `Life`, `Lineage`, `Child`, `Pregnant`, `Hunger`, `Thirst`, `BodyHealth`; `SimResources.settlements`, `rules.society.birth`, `rules.calendar`, `rules.clock_offset_ticks`, `world_seed` | `Pregnant` (inserted at a conception, removed when due); `SimResources.due_births` |
| 37 | `RoomSystem` | 152 | 1 | `rust/crates/sim-systems/src/runtime/rooms.rs` | `Agent`, `Position`; `SimResources.tile_grid`, `building_kinds`, `settlements`, `colony` (beds, zones, occupied, room overrides) | `SimResources.colony` (rooms, room_of_tile, next_room_id, rooms_announced, rooms_dirty, rooms_version); chronicle |
| 38 | `OralTransmissionSystem` | 160 | 1440 | `rust/crates/sim-systems/src/runtime/knowledge/oral.rs` | `Agent`, `Position`, `Child`, `Lineage`, `Life`, `Knowledge`; `SimResources.colony` (rooms), `settlements` (hearths), `rules`, `world_seed` | `Knowledge` (listeners' new entries; fire keeping and storytelling stamped used); `SimResources.colony.tallies` |
| 39 | `ObservationSystem` | 161 | 30 | `rust/crates/sim-systems/src/runtime/knowledge/observe.rs` | `Agent`, `Position`, `AgentState`, `Job`, `Knowledge`; `SimResources.colony` (workshops and bills, sites), `rules` | `Knowledge` (`observed`, `Source::Observed` entries); `SimResources.colony.tallies` (observed) |
| 40 | `KnowledgeDecaySystem` | 162 | 60 | `rust/crates/sim-systems/src/runtime/knowledge/decay.rs` | `Agent`, `AgentState`, `Knowledge`; `rules` (knowledge, calendar), `current_tick` | `Knowledge` (proficiency, entries at 0 removed, lore stamped used); `SimResources.colony.tallies` (forgotten) |
| 41 | `DiscoverySystem` | 163 | 240 | `rust/crates/sim-systems/src/runtime/knowledge/discover.rs` | `Agent`, `Child`, `Lineage`, `Temperament`, `Knowledge`, `Skills`; `SimResources.colony` (ground items, zones, resource nodes, item ledger), `settlements` (hearths), `rules` (neighbours: eureka), `world_seed` | `Knowledge` (a discovered node at `discovery_gain` ‰, `Source::Discovered`); `SimResources.colony.tallies` (discoveries), `colony.neighbours.tallies` (eurekas), `chronicle` |
| 42 | `TechAggregateSystem` | 164 | 240 | `rust/crates/sim-systems/src/runtime/knowledge/aggregate.rs` | `Agent`, `Child`, `Knowledge`; `rules` (knowledge, calendar), `current_tick` | `SimResources.colony.tech` (states, era progress, version), `chronicle` |
| 43 | `StorytellerSystem` | 170 | 1440 | `rust/crates/sim-systems/src/runtime/story/mod.rs` | `Agent`, `Child`, `Sick`, `Lineage`; `SimResources.colony` (story, ground items, zones, rooms, beds), `bed_of`, `tallies` (deaths), `rules` (story, calendar, seasons), `world_seed` | `Sick` (inserted, removed); `SimResources.colony.story` (planned, active, history, version), `chronicle`, `causal_log` |
| 44 | `AnimalSystem` | 171 | 3 | `rust/crates/sim-systems/src/runtime/wildlife/mod.rs` | `Agent`, `Position`, `Drafted`, `Tool`; `SimResources.colony` (wildlife, story, rooms, zones, ground items), `nav`, `tile_grid`, `settlements`, `building_kinds`, `rules`, `world_seed` | `SimResources.colony` (wildlife: animals, meat lots, tallies; fights; ground items, item ledger; story via `story::end_pressure`), `settlements` (food), `spatial.agents` |
| 45 | `MoodSystem` | 172 | 60 | `rust/crates/sim-systems/src/runtime/mood/mod.rs` | `Agent`, `Child`, `Life`, `AgentState`, `Position`, `Cold`, `Hunger`, `Thirst`, `Sick`, `Drafted`, `Temperament`; `SimResources.colony` (rooms, story), `rules` (mood, calendar, clock offset) | `Mood` (inserted for adults and children of 10 years or more; value, nights, breakdown), `Job` (cancelled at a breakdown); `SimResources.colony` (reservations), `chronicle` |
| 46 | `QuarrelSystem` | 173 | 6 | `rust/crates/sim-systems/src/runtime/mood/quarrel.rs` | `Agent`, `Position`, `AgentState`, `Mood`, `Kin`, `Lineage`, `Temperament`, `Drafted`; `SimResources.colony` (bonds, story), `relationships` (hostility), `rules`, `world_seed` | `Mood` (value, quarrel day); `SimResources.colony` (bonds, fights), `relationships` (hostility), `chronicle` |
| 47 | `NeighbourSystem` | 174 | 1440 | `rust/crates/sim-systems/src/runtime/neighbours/mod.rs` | `Agent`, `Knowledge`; `SimResources.colony` (neighbours, story, ground items, zones), `settlements` (food), `nav`, `tile_grid`, `building_kinds` (the hearth), `rules`, `world_seed` | `SimResources.colony.neighbours` (list, waves, caravans, plans, tallies, version) |
| 48 | `VisitorSystem` | 175 | 3 | `rust/crates/sim-systems/src/runtime/neighbours/visitors.rs` | `SimResources.colony.neighbours`, `nav`, `building_kinds` (the hearth), `rules` | ECS people (a wave accepted on its way joins on arrival, `join::accept`); `SimResources.colony.neighbours` (waves, caravans: position, path, state, answer; neighbours' attitude and last contact; tallies, version), `chronicle` |
| 49 | `InfluenceVisualizationSystem` | 1000 | 6 | `rust/crates/sim-systems/src/runtime/influence/visualization.rs` | `SimResources.influence_grid` (current Warmth and Danger buffers), `current_tick` | its own `VisualizationDigest` (no world or resource state) |

<!-- system-table:end -->

## 4. Bridge API (`WorldSimNode`, grouped by concern)

All methods live on the one Godot class `WorldSimNode` (signal `world_reset`
after a reset or a load). The exact exported list is checked by gate step 13, currently
`python3 tools/v8/bridge_api.py . --check docs/v8/bridge_api_mux2.txt` (M-UX2; the existing M5.5-named list has100 exports after Stage A; M5 checked `docs/v8/bridge_api_m5.txt` (91), M4 `docs/v8/bridge_api_m4.txt`, M3 `docs/v8/bridge_api_m3.txt`, M2 `docs/v8/bridge_api_m2.txt`, M1.5 `docs/v8/bridge_api_m1_5.txt`, M1 `docs/v8/bridge_api_m1.txt`, V8-0 `docs/v8/bridge_api_v7_final.txt` minus `docs/v8/bridge_api_removed.txt`)
(a pass that changes the API updates this command; section 6).

| concern | methods |
|---|---|
| immutable content boot | static `bootstrap_catalog()` (preflight receipt), static `create()` (validated node or null), `get_content_catalog()` (selected registry metadata) |
| steward and UI facts | `get_ui_facts`, `get_steward` (independent defence toggle, last batch and active owned counts), `cmd_set_steward`, `cmd_set_steward_defence` (ordinary logged commands; ADR 0045) |
| optional native profiling | `mux2_native_timings` (read-only, `WORLDSIM_MUX2_PROFILE=1`; cumulative counters and epoch, not simulation policy) |
| lifecycle and speed | `reset_world`, `reset_world_stress` (benchmark world), `set_sim_speed`, `advance_ticks` (tooling) |
| save files (M1.5) | `save_world(path) -> bool`, `load_world(path) -> bool` (absolute path; GDScript globalizes `user://saves/<seed>-<tick>.wsave`; a load replaces the engine and emits `world_reset`; bytes from `sim_systems::save`) |
| per-frame snapshots | `get_agent_snapshot`, `get_building_snapshot`, `get_construction_snapshot`, `get_resource_snapshot`, `get_terrain_snapshot`, `get_time_snapshot`, `get_tallies`, `get_world_info`, `get_recent_deaths`, `get_activity_snapshot`, `get_porter_snapshot` |
| one agent | `get_agent_detail`, `get_agent_id`, `get_agent_temperament`, `get_agent_family`, `get_relationship_snapshot`, `get_person` |
| settlements | `get_settlement_snapshot`, `get_society_snapshot`, `get_settlement_relations` |
| chronicle | `get_chronicle_since`, `get_person_chronicle`, `get_settlement_chronicle` |
| causal "why?" | `get_tile_causal_history`, `get_tile_causal_events`, `get_event_chain`, `get_tile_detail`, `get_influence_overlay` |
| commands | `on_building_placed`; M1 orders `cmd_designate_chop`, `cmd_designate_chop_tiles`, `cmd_cancel_area`, `cmd_add_stockpile`, `cmd_remove_zone_area`, `cmd_place_blueprint` (each returns `{accepted, count, reason}` and is logged with the next tick, ADR 0012) |
| colony reads (M1) | `can_place_blueprint`, `get_colony_versions` (poll), `get_trees`, `get_colony_overlay`; the agent snapshot's `jobs`, `carry_kinds`, `carry_counts` columns |
| economy (M2) | orders `cmd_designate_quarry`, `cmd_add_bill`, `cmd_remove_bill`, `cmd_set_bill_mode`, `cmd_move_bill`, `cmd_set_bill_suspended`, `cmd_set_work_priority` (names parsed in `ffi/economy.rs`; logged like every order); reads `get_workshops`, `get_work_priorities`, `get_recipes`, `get_agent_economy(agent_id)`; `get_colony_overlay`'s `designation_kinds`, `node_*`, `item_qualities`; `cmd_place_blueprint` kinds `knapping_spot`, `wood_bench`, `drying_rack` |
| built space and rooms (M3) | orders `cmd_place_line`, `cmd_place_rect`, `cmd_designate_dig`, `cmd_designate_deconstruct`, `cmd_set_room_role` (`ffi/space.rs`; `cmd_place_blueprint` kinds `log_wall`, `stone_wall`, `door`, `packed_floor`, `bedroll`); reads `get_place_preview` (drag ghost with per-tile reasons), `get_space` (walls, doors, floors, dug tiles), `get_rooms`, `get_room_detail(room_id)`; `get_colony_versions`' `rooms_version`; `get_colony_overlay`'s `item_covered`; `get_agent_economy`'s `bed`, `bed_room` |
| knowledge (M4) | orders `cmd_order_teach(teacher, learner, node)`, `cmd_cancel_teach(order)`, `cmd_designate_clay` (`ffi/knowledge.rs`; `cmd_place_blueprint` kinds `grinding_stone`, `pottery_bench`; recipes `knap_fine_blade`, `grind_axe`, `form_pot`, `fire_pot`); reads `get_tech` (per node state, holders, practitioners, names; era progress ‰ and label; at-risk count), `get_tech_node(node)` (holders with proficiency, learners, discovery conditions, gates), `get_agent_knowledge(agent_id)` (entries with proficiency, source, last use, practitioner flag), `get_teach_orders`, `get_knowledge_waits` (bills, sites and marks waiting for a practitioner, with the missing node); `get_space`'s `ford_xs`, `ford_ys` |
| story, animals and mood (M5) | orders `cmd_draft(agent, on)`, `cmd_undraft_all`, `cmd_draft_move`, `cmd_draft_attack` (an animal), `cmd_draft_attack_person`, `cmd_designate_hunt(animal)`, `cmd_add_hunt_zone`, `cmd_cancel_hunt`, `cmd_set_dress_order` (`ffi/story.rs`); reads `get_story` (pressures in force, the firewood line, wolves), `get_animals` (deer and wolves, hunt zones), `get_agent_m5(agent_id)` (mood, modifiers, apparel, draft, illness), `get_drafted`, `get_chronicle_why(index)` (the "→ 왜?" chain), `get_year_summary(year)`, `get_teach_suggestion(teacher, node)`; the agent snapshot's `tops`, `mood_bands`, `drafted`, `sick` columns; the chronicle rows' `index`, `link`, `link_entry`, `link_code` |
| neighbours and trade (M5.5) | orders `cmd_accept_migrants(wave)`, `cmd_decline_migrants(wave)`, `cmd_trade(caravan, give, take)` (`ffi/neighbours.rs`); reads `get_neighbours` (the two neighbours, waves, caravans, clues), `get_wave(wave)` (its people with reserved ids and top knowledge), `get_caravan(caravan)` (goods with values and unknown marks, the colony's stockpiled stacks); the agent snapshot's `fleeing` column; `get_agent_m5`'s `fleeing`; `get_year_summary`'s `migrants_joined`, `trades`, `eurekas` |
| appearance and materials (M-ART2) | no new method: the agent snapshot's `sexes`, `look_ids`, `variants` columns (`ffi/appearance.rs`, `collect_appearance`, rule file `sim-bridge/data/appearance.ron`); `get_space`'s `wall_materials`, `wall_styles`, `wall_categories`, `floor_materials`, `floor_styles`, `floor_categories` (`ffi/space.rs`; categories from `ffi/material_category.rs`, which embeds `sim-core/data/material/**` through `sim-bridge/build.rs`); the text renderer's `--people-json` `appearance` |
| calendar and ages (M1.5) | `get_time_snapshot`'s `year_index`, `season_index`, `day_of_year`, `days_per_year`; `get_world_info`'s `ticks_per_year`; the agent snapshot's `ages_years`, `infants`, `pregnant_days` columns |

Every getter, key, type, id space and row order is documented in
`docs/v8/snapshot_schema.md` (schema version 11, M-UX2); the gate step
`schema_docs` (`python3 tools/v8/check_docs_helpers.py schema .`) fails on any
getter or key missing there (ADR 0016 C2).

The god-hand methods `apply_intervention` and `get_intervention_state` were
removed in V8-0 (ADR 0010); the render kernels `art_declump` and
`art_walker_offsets` were removed in M1 with the V7 agent renderer.

## 5. Determinism and integer math

Short form; the full rules and their history are in
`.claude/skills/worldsim-architecture/SKILL.md`.

- Integers and permille in `sim-core`, `sim-systems`, `sim-engine`: no
  `exp/ln/powf/sin/…` (gate step 7 greps for them).
- Iterate maps and sets in key order (`BTreeMap`/`BTreeSet`, or collect and sort).
  Never insert or remove hecs components while iterating a hash collection.
- Randomness comes from keyed draws (`rust/crates/sim-core/src/rng.rs`): a
  draw is addressed by its purpose and keys, consumes no shared stream state,
  so skipping one changes nothing else.
- Proof of "no behaviour change": `tools/v8/identity.sh` (32 worlds × 120
  days: probe summaries, raw samples and chronicles equal) and
  `worldsim_probe --check-determinism`.

## 6. How to …

### Add a system

1. Put the component(s) it needs in `rust/crates/sim-core/src/components/` (plain structs, `Debug, Clone`, serde).
2. Write the system in `rust/crates/sim-systems/src/runtime/<domain>/<name>.rs`: a struct implementing `RuntimeSystem` (`name`, `priority`, `tick_interval`, `tick`). Start the file with a `//!` header giving purpose, `Priority:`, `Interval:`, `Reads:`, `Writes:`, `Events:`.
3. Gate it on a rule in `WorldRules` (`rust/crates/sim-core/src/rules/`) when it changes behaviour, and set the rule in `rules/baseline.rs`; update the baseline dumps in `rust/crates/sim-core/tests/data/` in the same commit and record why.
4. Register it in the matching `register_<domain>_systems` helper that `register_default_runtime_systems` calls (`rust/crates/sim-systems/src/lib.rs`); pick an unused priority (equal priorities run in registration order).
5. Test it on `sim_test::baseline_engine(seed)` (V7 systems) or `sim_test::colony_engine(seed)` (colony systems) in the matching domain test file (`rust/crates/sim-test/tests/<domain>_*.rs`) with a threshold on a measured invariant.
6. Regenerate the table above: `python3 tools/v8/system_table.py --write`.
7. If behaviour changes on purpose, re-capture the identity base data and say so in the pass's decisions.

### Add a bridge read

The full procedure with code skeletons is `.claude/skills/worldsim-architecture/SKILL.md` ("B. Add a bridge read") and its `references/procedures.md`; the steps:

1. Check first that the getter does not exist (section 4, `GETTERS` in `scripts/core/sim_reads.gd`).
2. Pick the file of `rust/crates/sim-bridge/src/ffi/` for the concern: `node.rs` (the class, process loop, speed and every `#[func]`), `snapshot.rs` (per-frame facade: agents, construction, recent deaths, clock, tallies, world info, terrain, buildings; resources delegate to `snapshot/resources.rs`), `reads_agent.rs` (agent detail, temperament, relationships), `reads_settlement.rs` (settlement and society), `causal.rs`, `chronicle.rs`, `family.rs`, `relations.rs` (village relations, porters), `activity.rs`, `engine_build.rs` (engine construction), `colony.rs` (M1 orders and colony reads), `appearance.rs` (M-ART2 looks and face variants), `material_category.rs` (M-ART2 material category names), `mod.rs`. A new concern gets a new file `<concern>.rs` with a `//!` header, a `pub mod <concern>;` line in `ffi/mod.rs` and a `pub use` of the collector.
3. In that file write a pure `pub fn collect_<thing>(&World, &SimResources, …) -> Row/Vec<Row>` (no Godot types; rows sorted by id; say which id space each field uses) and re-export it from `rust/crates/sim-bridge/src/ffi/mod.rs` (`pub use <concern>::collect_<thing>;`). sim-bridge is built as both `cdylib` (what Godot loads) and `rlib` (what sim-test links; `crate-type` in `rust/crates/sim-bridge/Cargo.toml`), so sim-test can call the collector without Godot.
4. In the same file write the marshaller `pub(super) fn <thing>_to_dict(...) -> VarDictionary` (columns as `Packed*Array`, ids as signed i64), as `agent_family_to_dict` in `family.rs`, `rows_to_dict` in `chronicle.rs`, `porter_rows_to_dict` in `relations.rs`.
5. Add a `#[func] fn get_<thing>(&self, …) -> VarDictionary` to the one `#[godot_api] impl WorldSimNode` block in `rust/crates/sim-bridge/src/ffi/node.rs`, importing both with `use super::<concern>::{collect_<thing>, <thing>_to_dict};`. Its body only calls the collector and the marshaller; read-only, at most 14 parameters. `VarDictionary` is the rule; a few getters return a scalar or an array (`get_agent_id -> i64`, `get_tile_causal_history -> VarArray`). A new read returns `VarDictionary` unless a single scalar is the whole answer.
6. Test the collector in `rust/crates/sim-test/tests/engine_bridge_ffi.rs` (the bridge-surface domain file) or a new `engine_<concern>_ffi.rs`: call `sim_bridge::ffi::collect_<thing>` on `sim_test::baseline_engine(seed)` (or `sim_test::colony_engine(seed)` for colony state; a colony getter returns its empty form in baseline) and assert content and order. Unit tests inside sim-bridge (`cargo test -p sim-bridge`) are for pure marshalling helpers only.
7. GDScript side. `SimReads` (`scripts/core/sim_reads.gd`, `class_name SimReads`, node `Main/SimReads`) serves only argument-less getters listed in `GETTERS`: add the name there and read it with `SimReads.read(world_sim, "get_<thing>")` (a literal name). `GETTERS` are cached once per frame; those also in `TICK_GETTERS` (today `get_porter_snapshot`, `get_settlement_relations`) once per simulation tick; terrain until a reset. A getter that takes arguments (for example `get_agent_detail(entity_bits)`) is not in `GETTERS`: call it directly (`world_sim.get_<thing>(args)` or `_world_sim.call("get_<thing>", args)`), never per node per frame. `get_tile_causal_events` has its own helper `SimReads.causal_events(...)`. Name the method in the calling `.gd` file's header comment ("Bridge methods: …").
8. Run, in order: `cd rust && cargo build -p sim-bridge` (Godot loads the debug library this builds; `cargo test` does not rebuild it); the headless import (`bash tools/v8/run_bounded.sh 600 "$GODOT_BIN" --headless --path . --import --editor --quit`) only on a fresh checkout or worktree (twice, judge the second) or after adding or moving assets, scenes, scripts or a `class_name` (adding a `#[func]` to an already imported checkout needs no import); `bash tools/v8/gdscript_strict_check.sh`; `bash tools/v8/smoke.sh <log_dir>`. The strict check's exit 3 compares GDScript calls `world_sim.X(` / `_world_sim.X(` and the names passed to `SimReads.read/mutate` with the `#[func]` names in the Rust source (`tools/v8/bridge_api.py` scans `rust/crates/sim-bridge/src`), so it does not need the build; dynamic `call("name")` strings are not checked by it, only exercised by the smoke.
9. A pure read changes no simulation state, so `bash tools/v8/identity.sh` must still pass. If it fails, the read is not pure: that is a bug.
10. Record the API change. Gate step 13 checks that the set of exported names (`class`, `func`, `signal` lines; order and signatures are not compared) equals an expected list — at V8-0, the V7 base list (`docs/v8/bridge_api_v7_final.txt`) minus `docs/v8/bridge_api_removed.txt`. A pass that adds or removes a `#[func]` writes one list for the pass, `python3 tools/v8/bridge_api.py . > docs/<line>/bridge_api_<pass>.txt`, where `<line>` is the docs folder of the pass's line (`v8` for V8 passes) and `<pass>` the pass name in lower case with `.` as `_` (`m1`; M1.5 → `m1_5`), so M1's list is docs/v8/bridge_api_m1.txt. Every later commit of the same pass that changes the API rewrites it. In the same commit, change gate step 13 in `tools/v8/gate.sh` to `python3 tools/v8/bridge_api.py . --check docs/<line>/bridge_api_<pass>.txt` (no `--minus`), update the check command and the method table in section 4 of this file, and add a CHANGELOG line naming the added or removed method. An ADR is needed only if the read encodes a design decision. The V8-0 lists stay as V8-0's record. Run one test file with `cd rust && cargo test -p sim-test --test <file>`; a test file that covers M1 colony code also goes into gate step 17's name list. Finish with the whole gate (`bash tools/v8/gate.sh <log_dir>`).

## 7. GDScript layout and localization

| folder | contents |
|---|---|
| `scripts/core/` | `locale.gd` (autoload `Locale`), `sim_reads.gd` (`SimReads`: every bridge read), `settlement_selection.gd` (selected village), `main_bootstrap.gd` and `presentation_bootstrap.gd` (catalog/presentation validation before main children) |
| `scripts/world/colony/` | the colony view (node `ColonyView` in `scenes/main.tscn`; M-ART2, section 14): `view.gd` (TILE 64, the base point (tile_left + 32, tile_top + 43), tile/keyed hashes, value noise), `ground_layer.gd` (ground chunks, wall/door/hill-face strips, ford stones, water shader; the LRU chunk cache and incremental space rebake), `tree_layer.gd` (trees through the flora catalogue with the season shader, stumps, bushes, deadwood by 16×16-tile chunk near the view; hidden where the ¼× chunk carries the canopy), `building_layer.gd` (structures, huts by the bindings ring, workshops, hearth states, bedrolls), `overlay_layer.gd` (zones, item stacks by count bucket, carcasses, designation marks, site ghosts, labels), `room_layer.gd` (R: room tints and labels), `animal_layer.gd`, `visitor_layer.gd`, `agent_layer.gd` (route I people as feet-anchored nodes with the variant shader, selection/drafted outline and night halo, hit test) + `lod_markers.gd` (MultiMesh discs and activity glyphs at ¼×; route I bodies remain at ½×), `order_tool.gd` (F1–F10 modes, drag lines and rectangles with the `get_place_preview` ghost at 64 px, the `cmd_*` calls). Everything that stands sorts by base y in the y-sorted child `ColonyView/World`; the ground chunks sit below it; marks and labels above |
| `scripts/world/` | `art/` (M-ART2: `art_data.gd` manifest/bindings/params lookups and F12 reload, `material_resolver.gd`, `flora_resolver.gd` (M-ART2.1), `ground_baker.gd`, `wall_baker.gd`, `tone.gd`, `site_ghost.gd`), `camera_controller.gd` (zoom ladder ¼×–2× from the params, pan, speed keys 1–4, pause, F12), `daylight_controller.gd` (night CanvasModulate, hearth lights, `dark_at`; + `wood_visuals.gd`), shared helpers (`art_hash.gd`, `porter_presentation.gd`); `wood_visuals.gd` retains only the settlement snapshot lookup used by daylight. The unused frame helper cluster and its `art_sprite_library.gd` dependency were removed in MUX Stage C. HUD icons use `ArtUI` → `ArtData` → `bindings.ui_icons`, whose existing atlas texture remains live |
| `scripts/ui/hud/` | `hud_topbar.gd`, `order_bar.gd` (F1–F5 buttons and the order toast) |
| `scripts/ui/panels/` | `chronicle_panel.gd`, `agent_inspector_panel.gd`, `causal_panel.gd` (explicit why actions only; compact anchored scroll panel), `item_panel.gd` (item inspector with why action), `priority_panel.gd` (M2 work priorities, 7 work types since M3), `workshop_panel.gd` (M2 bills of the selected workshop), `room_panel.gd` (M3: a room's role dropdown → `cmd_set_room_role`, tiles, beds, doors, covered items; made by `order_bar.gd`), `art_ui.gd` (shared panel style) |
| `scripts/ui/mux/` | HUD, settings, alerts, goals, help and learning panels; `controls_catalog.gd` shares the README registry; `korean_text.gd` adds display-only word joiners without changing raw Locale strings |
| `scripts/ui/` | text helpers shared by panels (`chronicle_text.gd`, `person_names.gd`, `person_read_cache.gd`) |

Every `.gd` starts with a comment block: purpose and the bridge methods it
calls. UI text goes through `Locale.ltr("KEY")`; keys live in
`localization/<lang>/<category>.json` (the source) and are compiled into
`localization/compiled/<lang>.json` (what the game loads) by
`python3 tools/localization_compile.py --project-root .`.

## 8. Contract and benchmark (M1.5, ADR 0016)

The simulation contract binds every pass; each clause has its gate step.

| clause | what it says | how it is checked |
|---|---|---|
| C1 reproducible | the state at a tick is a pure function of `(rules, seed, command log)`; a save is that state with a format version, and `save → load → tick` = `tick` | `identity`, `determinism`, `colony_determinism`, `replay`, `save_roundtrip` (`tools/v8/save_roundtrip.sh`; `sim_systems::save`) |
| C2 view-independent | the sim crates know no pixels or Godot; renderers read documented snapshots only; a second renderer agrees with the Godot view | `schema_docs`, `renderer_agreement` (`tools/v8/renderer_agreement.sh`: `worldsim_probe --ascii`, `sim_test::probe::ascii`, against `scripts/world/colony/agent_layer.gd`), `bridge_api` |
| C3 measured budget | colony 192², N = 200/1,000/2,000: tick mean, p99, per-system share, bytes per agent and per tile; ceilings 5 ms (N = 1,000), 10 ms (N = 2,000), 4 KiB per agent, 96 B per tile; > +10 % against the previous pass fails; no O(agents × candidates) search without the spatial index | `bench` (`tools/v8/bench.sh` → `worldsim_perf bench`, `sim_test::bench`; ledger `docs/perf/PERF_LEDGER.md`) |
| C4 method | closed-form state, event-driven work, integer tables, no new float state, keyed draws, key-ordered maps, `(y, x)` then id ties | `forbidden_math`, identity, review |
| C5 structure | logic in the owning system, generated system table, files over 1,200 lines reported | `docs`, the pass report |

- **Spatial index** (`rust/crates/sim-core/src/spatial.rs`): `ChunkIndex<T>` over 16 × 16-tile chunks
  (`BTreeMap<(cy, cx), Vec<(tile, T)>>`, sorted `(y, x, T)`); `nearest_where` returns the smallest
  `(Chebyshev distance, y, x, T)` that passes a predicate, visiting non-empty chunks by exact distance. Kept in
  `SimResources.spatial`: the colony job searches' indexes (ground stacks, zone cells, chop marks), rebuilt when
  `ColonyState::overlay_version` changes, and the agents' index, built on demand once per tick
  (`SpatialIndex::agents_at`). Derived state: never saved.
- **Benchmark** (`worldsim_perf bench`): six scenes on `WorldGenConfig::colony_stress(15100, N, side)` under
  `WorldRules::colony`, each in its own process for a clean resident size, 1,440 warm-up and 2,880 profiled
  ticks, 3 repeats, medians; fed like `quick`. `bench-summarise` merges interleaved sessions of two binaries.
- **Save files** (`rust/crates/sim-systems/src/save/`): magic `WORLDSIM`, `FORMAT_VERSION`, then a catalog-revision envelope before typed body decoding (format11, ADR0038/0044/0045). The body includes the tick, all persisted `SimResources` (hash collections in key order), the hecs world (archetypes in creation order, entity ids and generations, allocator free list), each system's `save_state`, Steward state and sparse content identity. Explicit format6/7 readers retain historical field widths and indexes; format8 and format9 each accept only their frozen bundled checkpoint revision. Format11 persists defence state and uses the frozen formats7–10 steward wire layout in `save/legacy_steward.rs`; legacy defence defaults are inserted on migration. Format10 appended a persisted tree-material default and sparse standing-tree overrides; legacy6–9 trees migrate to historical oak without species rerolls, and only missing validated embedded wood stats are added. Novel illness provenance is sparse agent-keyed state, validated against live Sick components before load returns and removed on recovery or death.

## 9. Items and jobs (M2, ADR 0019, ADR 0020)

The colony's economic loop, all under `ColonyRule::economy` (`None` in the baseline and in M1's rules):

```
worldgen (colony.rs: nodes, material table)
   │  boulders / flint outcrops (ColonyState.nodes)        trees (M1)        deadwood (SimResources.wood_tiles)
   ▼                                                        ▼                  ▼
orders (commands.rs): DesignateQuarry · DesignateChop · AddStockpile · PlaceBlueprint(workshop) · AddBill … · SetWorkPriority
   ▼
JobAssignSystem 124 — passes by priority level (1..4), within a pass fuel-only Haul first at the worker's own Haul priority, then WorkType order Craft > Build > Quarry > Chop > Split > Haul (higher priorities and Haul=0 still win; M1 unchanged)
   │  Craft: Supply (inputs → shelf), Craft (at the work tile)        ◄── BillSystem 123 (hourly): active bill per workshop
   │  Build: Deliver (per material line), Build                       Quarry: Quarry (batch of 4)
   │  Chop: Chop (+ axe stop: Tool)                                   Split: the firewood bill's Craft, else Gather (deadwood)
   │  Haul: loose stacks → stockpile cells (per item spec)
   ▼
JobExecuteSystem 127 — walk, work (ticks × SPEED_PERMILLE[level] × AXE_PERMILLE), put/take items, ledger
   │  craft::start (inputs consumed) … craft::finish_craft (quality draw, output at the work tile, XP)
   ▼
items: ground stacks (ColonyState.ground_items, one ItemSpec bucket per tile) · Carry · Tool · site MaterialNeeds · workshop shelves
   ▼
HearthFeedSystem 146 (hourly) — firewood items (stockpiles, hearth pile) → settlement stockpile[Wood] → FireSystem 147
```

- **Item spec**: `ItemSpec { kind, material, quality }`; a stack merges only within one (kind, material, quality / 10)
  bucket. Per-kind stack maxima and carry loads: `ItemKind::default_stack_max` / `default_carry` (logs from the rule).
- **Ledger**: `jobs::invariants::ledger_of(world, res, kind)` — `created = ground + carried + held + delivered +
  shelved + consumed`, checked every tick in tests, in the probe (`ledger_violation_ticks`) and in debug builds.
- **Workshops** live in `ColonyState::workshops` (id order): footprint (`Occupant::Workshop(id)`), work tile (south of
  the centre column), shelf, ordered bills (`auto` = the hearth's standing firewood bill).
- **Skills and tools**: `Skills` (per `SkillKind`), `WorkPriorities`, `Tool` are colony-only ECS components, saved
  (`save::world::registry`).
- **Decision seam**: `jobs::hearth::decision_firewood` turns the decision tick's firewood rule off in the colony
  (`decision/system.rs` +2 −1); the seam lets a craft at work yield only to a critical need (`jobs::seam`).

## 10. Space, paths and rooms (M3, ADR 0021–0023)

Built space, digging and rooms, under `ColonyRule::space` (`SpaceRule`; `None` in the baseline and in the M1/M2 rules):

```
orders (commands.rs): PlaceBlueprint / PlaceLine / PlaceRect (log_wall, stone_wall, door, packed_floor, bedroll)
                      · DesignateDig · DesignateDeconstruct · SetRoomRole        (check_space: one reason per tile)
   ▼
ColonyState.space_sites (1×1 sites, MaterialNeeds)      dig / deconstruct marks (ColonyState.designations)
   ▼
JobAssignSystem 124 — WorkType Build: Clear (items off a site), Deliver, Build (from beside a wall site),
   │                  Deconstruct; WorkType Mine: Dig (frontier tiles only); every target reachable from
   │                  the worker's region, an Unreachable target skipped for a day
   ▼
JobExecuteSystem 127 — jobs::space::{complete_site, deconstruct, dig}: the tile grid's wall / door / floor / dug
   │  columns, materials consumed or refunded (half), 3 stone per dug tile, ledger
   │  ──► runtime::nav::refresh (cost map, nav.version; regions and memo rebuilt lazily) ──► movement (paths)
   │  ──► colony.rooms_dirty
   ▼
RoomSystem 152 — sim_core::rooms::detect (4-connected flood fill, bounds = walls, doors, undug hill, water,
   │  mountain; 2–120 tiles, off the edge, colony-made), assign_ids (≥ 50 % overlap keeps the id), roles
   │  (Workshop > Hall > Storehouse > Bedroom > Unassigned, or the override), RoomFormed
   ▼
effects: beds (agent/beds.rs: roofed bedroom first, reachable only) · no night cold under a roof (settlement/fire.rs)
   · rest ×800 ‰ in a bedroom (needs/sleep_decay.rs) · hauls to the storehouse and restock (jobs/assign.rs)
   · Maintain counts covered stock (jobs/bills.rs) · workshop room crafts at 900 ‰ (jobs/craft.rs) · evening hall
```

- **Cost map and regions** (`rust/crates/sim-core/src/nav.rs`, `rust/crates/sim-systems/src/runtime/nav.rs`): one
  byte per tile from `runtime::nav::tile_cost`, built at worldgen and refreshed by the code that changes a tile;
  `nav.version` bumps when a cost moved. Regions (connected components under the move rule) and the path memo
  (4,096 `(from, to)` entries) belong to one version and are rebuilt lazily; neither is saved. Paths are saved
  (save `FORMAT_VERSION` 3).
- **Tile columns**: `TileGrid.door` and `TileGrid.dug` (`#[serde(default)]`) beside V7's `wall_material` and
  `floor_material`; a dug tile keeps terrain `Hill` (passable because dug).
- **Rooms** live in `ColonyState.rooms` (id order) with `room_of_tile`, `room_overrides`, `rooms_version`; derived
  from the tile grid and the sites, so only ids and overrides need the save (they are part of `ColonyState`).
- **Decision seam**: `decision/system.rs` is unchanged in M3; decision targets are made reachable in movement
  (`runtime/agent/movement.rs`), and a job holder at its hunger or thirst seek threshold gets no new job
  (`jobs::assign`, M3 DECISIONS D16).
- **Measurement**: `worldsim_perf nav-bench` (gate step 27, report only; budget 600 µs of movement per tick at
  N = 1,000), the soak (`tools/v8/soak.sh`, gate step 26: 4 seeds × 20 days, ledger, reservations, deaths,
  determinism), the probe's `m3` block (`rust/crates/sim-test/src/probe/space.rs`).

## 11. Knowledge and technology (M4, ADR 0024–0026)

What people know, how it moves, and what the settlement is observed to know, under `ColonyRule::knowledge`
(`KnowledgeRule::M4`; `None` in the baseline and in the M1–M3 rules, where every system below returns at once):

```
data: sim-core/data/colony/knowledge.ron (24 nodes) ──► knowledge::defs() ──► knowledge::gates()
   │  (recipe / blueprint / designation / channel / lore masks, built once)
   ▼
worldgen/colony.rs seed_founders (STREAM_KNOWLEDGE; prereq-aware) ──► Knowledge component per person
   │  (6-byte entries + cached practitioner bitset `practice`; saved, format v4)
   ▼
gate: JobAssignSystem 124 (ctx.practice covers the job's mask, else an unmemoised gated search; bills, sites and
   │  marks wait) · BillSystem (runnable only with a practitioner) · AgentDecisionSystem's site snapshot leaves out
   │  gated colony sites (D22) · craft quality capped by QUAL_CAP[proficiency / 100]; a gated job done = use (+2 ‰)
   ▼
channels: OralTransmissionSystem 160 (20:30 daily: gathering within 16 tiles; rehearsal; one keyed lesson a listener)
   │  · ObservationSystem 161 (every 30: +5 ‰ near gated work, below threshold) · apprenticeship (OrderTeach →
   │  jobs/teach.rs Teach/Learn pair, 60-tick sessions) · KnowledgeDecaySystem 162 (every 60: after 6 unused days,
   │  ‰ per hour by source to a floor) · DiscoverySystem 163 (every 240: curious holders of the prerequisites, the
   │  node's settlement conditions, keyed 1 in 40 / 90 / 200)
   ▼
TechAggregateSystem 164 (every 240) ──► ColonyState.tech (per node: holders, practitioners, state, names; era ‰)
   │  ──► chronicle KnowledgeAtRisk / KnowledgeLost / KnowledgeLearned / KnowledgeDiscovered
   ▼
bridge ffi/knowledge.rs (get_tech, get_tech_node, get_agent_knowledge, get_teach_orders, get_knowledge_waits)
   ──► HUD era line and ⚠ badge, T tech map, K teach dialog, the inspector's 지식 tab, waiting lines
```

- **The practitioner bitset** (`Knowledge::practice`, `u32`, bit i = proficiency ≥ node i's threshold) is rewritten by
  every write to the component (`set`, `gain`, `use_mask`, decay); gating, the aggregate and the waits read only it.
  A new node must keep the map at ≤ 32 nodes (ids in 5 bits).
- **Chains**: clay banks (`NodeKind::ClayBank`, worldgen near rivers, `STREAM_CLAY`) → `Designation::ClayPit` →
  `JobKind::DigClay` (Clay) → `pottery_bench` `form_pot` (RawPot) → the hearth's `HearthFire` workshop `fire_pot`
  (ClayPot); ground stone: `grinding_stone` `grind_axe` (GroundAxe, 120 chops, faster); stone axes break after 40
  chops (`EconomyRule::tools_break`, chronicle `ToolBroke`).
- **Fords**: the tile grid's `ford` column (cost 30), placed at worldgen on the hearth's shortest way to the largest
  forest cut off by a river within 40 tiles (M4 DECISIONS D5).
- **Decision seam**: `decision/system.rs` +4 −1 in M4 (the site snapshot's gate; the budget was +20).
- **Measurement**: the probe's `m4` block (`rust/crates/sim-test/src/probe/knowledge.rs`), `docs/v8/m4/m4_score.py`
  (F1–F9, F11; `--f10` for the loss fixture `tools/v8/m4_f10_script.json`), the bench's `knowledge_us_per_tick` and
  `job_assign_us_per_tick`, `tools/v8/m4_u_script.json`; tests `rust/crates/sim-test/tests/colony_knowledge.rs`.

## 12. Story, animals and mood (M5, ADR 0027–0030)

Pressure from outside, the animals it brings, what people wear against the cold and how they feel, under
`ColonyRule::story`, `::wildlife` and `::mood` (`StoryRule::M5`, `WildlifeRule::M5`, `MoodRule::M5`; `None` in the
baseline and the M1–M4 rules, where every system below returns at once):

```
StorytellerSystem 170 (daily): plans each season with keyed draws (STREAM_STORY), mercy and gap rules, due kinds
   │  ──► ColonyState.story (planned / active / history; cost per pressure) ──► chronicle PressureBegan / Ended
   ▼  the effects live where the behaviour lives (they ask story::active):
   │  ColdSnap → FireSystem 147 (burn ×2, roofed sleepers cold without the lit hearth, exposure ×2 outside a room)
   │  Drought → ResourceRegenSystem (forage ×400 ‰), ThirstDecaySystem (×1,200 ‰)
   │  Illness → Sick (keyed spread in a bedroom), SleepDecaySystem ×1,500 ‰, the job layer's work ticks ×1,500 ‰
   │  Beasts → AnimalSystem's pack
   ▼
AnimalSystem 171 (every 3): ColonyState.wildlife (≤ 12 deer as a herd near the hearth, ≤ 4 wolves; not ECS) —
   │  deer graze, flee, rest; wolves walk a nav path to the nearest uncovered person and bite through
   │  ColonyState.fights, which CombatSystem 137 resolves (V7's combat_pairs stay person–person); meat on a
   │  stockpile or hearth feeds the larder hourly and spoils by lot after 6 days
   ▼
DraftSystem 121 (every tick): Drafted people take only MoveTo / Attack orders (auto-arm with a spear or axe within
   │  12 tiles; undraft themselves at a critical need); the decision seam (jobs/seam.rs) keeps them inert
   ▼
job layer: WorkType::Hunt (tracking practitioners; JobKind::Hunt: fetch a spear, stalk, attack, kill → meat, hide)
   │  · DryingRack scrape_hide (a knife = a flint blade on the shelf, 8 uses) → CuredHide → make_hide_wrap /
   │  make_fur_cloak · the hourly dressing pass (JobKind::Equip, the least cold-tolerant free adult first) →
   │  Apparel (a worn top spares cold hours at its ‰, STREAM_APPAREL)
   ▼
MoodSystem 172 (every 60, minute 7): Mood toward 500 + Σ modifiers (the other systems note facts through mood::note) —
   │  < 250 tantrum risk (work ×1,200 ‰), < 100 a breakdown (chronicle), ≥ 750 content (work ×900 ‰)
   ▼
QuarrelSystem 173 (every 30 ticks in five slices, one every 6): pairs within 2 tiles, the nearest 8, not kin-close; keyed (STREAM_QUARREL) quarrel → relation
   │  (ColonyState.bonds), hostility, mood; a fight of one hit at hostility 0.6 → chronicle Quarrel / Fight
   ▼
chronicle cause side table (Chronicle::causes: CauseRef entry / pressure / modifier; saved, format v5)
   ──► bridge ffi/story.rs (get_story, get_animals, get_agent_m5, get_drafted, get_chronicle_why, get_year_summary;
       the snapshot's tops / mood_bands / drafted / sick) ──► banners, D draft, M mood overlay, the inspector's
       기분 / 옷 lines, the chronicle's "→ 왜?" chains and the 이야기 tab
```

- **Keyed draws**: `STREAM_STORY` (season plans, intensity, illness), `STREAM_WILDLIFE` (herd, steps, hits),
  `STREAM_QUARREL` (chemistry, quarrel rolls), `STREAM_APPAREL` (a top's warm hour); 36 streams.
- **Items** (M5): meat 11, cured hide 12, hide wrap 13, fur cloak 14, spear 15, wolf hide 16; spoilage, knife wear
  and spear wear are `consumed` in the per-kind ledger; a worn top counts as held.
- **Decision seam**: `decision/system.rs` unchanged in M5 (+0; budget +20); drafted people are put in the seam's
  "may not yield" set in `jobs/seam.rs` (+10 −2).
- **Save**: `FORMAT_VERSION` 5 (Apparel, Sick, Drafted, Mood components; the storyteller, wildlife, fights, bonds,
  the chronicle's cause table).
- **Measurement**: the probe's `m5` block (`rust/crates/sim-test/src/probe/m5.rs`), `--story on|off|no-beasts`,
  `docs/v8/m5/m5_score.py` (F1–F10; `--story` for the gate's fixture), `tools/v8/m5_u_script.json` and
  `m5_u_nodraft_script.json`; tests `colony_story.rs`, `colony_wildlife.rs`, `colony_hides.rs`, `colony_mood.rs`,
  `engine_story_ffi.rs`.

## 13. Neighbours, migrants and trade (M5.5, ADR 0031–0032; corrections to ADR 0027–0030)

Two neighbours off the map send migrant waves and caravans, under `ColonyRule::neighbours` (`NeighbourRule::M5_5`;
`None` before M5.5 and in the baseline, where every system below returns at once). Visitors are not ECS entities
until a wave is accepted (M5.5 DECISIONS D16: V7's proximity sync would admit anyone standing at the hearth):

```
NeighbourSystem 174 (daily): ColonyState.neighbours — two neighbours drawn from the seed (STREAM_NEIGHBOURS: edge,
   │  distance, population, attitude, the Neolithic nodes they know), populations drift ±1 %; waves 1 (day 8–12) and 2
   │  (day 20–26), then a roll each later season (chance from food surplus, stock value, attitude; cap 40 people);
   │  a caravan date each season (STREAM_TRADE goods: the artifacts the neighbour can make and the colony cannot,
   │  staples), never in a major pressure or before day 6
   ▼
VisitorSystem 175 (every 3, offset 1): a wave / a caravan walks as one group from its edge to the meeting tile by the
   │  hearth (chronicle MigrantsArrived / TraderArrived); a wave waits a day for AcceptMigrants / DeclineMigrants
   │  (unanswered: it leaves, attitude −5; accepted on its way: it joins on arrival), a caravan trades a day, then
   │  they walk out
   ▼
commands: AcceptMigrants → neighbours::join (founder-like colonists: reserved AgentIds, Knowledge source Migrant,
   │  wraps, settlement membership; beds at 20:00) · Trade → neighbours::trade (80 % / 120 % balance; taken goods on the
   │  spot = ledger traded_in; given stacks owed until picked up) → job layer JobKind::Trade (Haul work, before
   │  carcasses and loose stacks) carries them to the spot = ledger traded_out
   ▼
DiscoverySystem 163 (under the neighbour rule): clues (an artifact held that no colonist can make → its producing
   │  nodes) count their stockpile conditions as met and win the keyed draw ×3 → chronicle Eureka when the clue
   │  decided the discovery
   ▼
bridge ffi/neighbours.rs (get_neighbours, get_wave, get_caravan, cmd_accept_migrants, cmd_decline_migrants,
    cmd_trade; the snapshot's fleeing) ──► the migrant dialog, the trade dialog, the N panel, the tech map's rumours
```

- **Corrections** (ADR 0027–0030 amendments): the storyteller holds back an illness during or 2 days after a major
  and a major while 2 or more are sick (`story::held_back`); `FleeSystem` 123 (every 3; it shares 123 with
  `BillSystem` and runs after it, by registration order) sends undrafted people near a
  hunting wolf to a reachable roof it can reach before the wolf by actual path/speed comparison, else a guarded reachable hearth, else away (30-tick reevaluation; pending bites also respect roofs; ADR0045). `Fleeing` skips the decision seam;
  the hunt turns into a haul of the meat at the kill (`hunt::haul_home`; the rest of a carcass is hauled first,
  `Wildlife::carcasses`); a hide wrap takes 1 cured hide, a knife lasts 4 uses; the mood's pressure weights and the
  quarrel constants are rule fields; a quarrel's cause is a concrete record (`chronicle::NeedSample` behind
  `CauseRef::Sample`, a death, a pressure, an earlier quarrel) checked by `mood::check_cause`.
- **Ledger**: per kind, created + traded_in = ground + carried + held + delivered + shelved + consumed + traded_out.
- **Items and values**: `sim-core/data/colony/values.ron` (`colony::values::value`: base × (500 + 5 q) / 1,000; per
  item `made_by`, `clue_nodes`, `tags`).
- **Data, not code** (owner directive 2026-10-24, DECISIONS D28): the M5.5 rules and content are RON files under
  `sim-core/data/colony/` — `neighbours.ron` (rule, name keys, goods table by tag, guaranteed node, migrants' top),
  `story.ron`, `wildlife.ron`, `mood.ron`, `values.ron` — embedded and parsed once (`ColonyRule::m5_5()`); code
  matches items by tag (`lacked`, `staple`, `carcass`), never by name.
- **Keyed draws**: `STREAM_NEIGHBOURS`, `STREAM_TRADE`; 38 streams.
- **Save**: `FORMAT_VERSION` 6 (`Fleeing`, neighbours, waves, caravans, the ledger's traded terms, need samples, the
  knowledge source `Migrant`, chronicle kinds 38–44).
- **Measurement**: the probe's `m55` block (`rust/crates/sim-test/src/probe/m55.rs`), `--neighbours on|off`,
  `docs/v8/m5_5/m55_score.py` (F1–F10; `--neighbours` for the gate's step 30), `tools/v8/m55_u_script.json` and
  `m55_u_nodraft_script.json`; tests `colony_neighbours.rs`, `engine_neighbours_ffi.rs` and the M5.5 cases in
  `colony_story.rs`, `colony_wildlife.rs`, `colony_hides.rs`, `colony_mood.rs`.

## 14. The renderer (M-ART2, ADR 0033–0034)

The colony is drawn at 64-px tiles in one projection: a tile's base point is (tile_left + 32, tile_top + 43); a wall is
a 16-px top over a 44-px face, so a wall row's strip covers [row_top − 16, row_top + 48). Data flow (ADR 0034):

```
bridge snapshots (get_terrain, get_space + materials/styles/categories, get_agent_snapshot + sexes/look_ids/variants,
   get_trees, get_building_snapshot, get_colony_overlay, get_animals, get_neighbours, get_time_snapshot …)
   ▼  resolution — data only, no content names in code (ADR 0033)
assets/v8art/bindings.json   sim codes → art ids (terrain, space, buildings, workshops, items, animals, trees, nodes,
                             blueprints, order icons, visitors, people, tones)
assets/v8art/manifest.json   art id → file, role, SHA-256, size, anchor, footprint; the 36 logical looks
scripts/world/art/material_resolver.gd   (style, material, category, kind) → top/face textures and tint (the art
                             line's resolve.py order; schema $defs supply defaults; parity test vs resolve.py)
assets/v8art/flora/ + flora_resolver.gd  (species, size, season, variant) → frame, canopy mask, season tint, anchor
                             (resolve_flora.py port, parity test; species from authoritative saved get_trees column)
   ▼  drawing
GroundBaker   16×16-tile chunks (1,024² RGBA8) on the CPU with native Image ops; dithered blends; rounded shores and
              hill edges from a closed-form field per tile quadrant (cached masks), banks and ink on the contour; water
              transparent → ground_chunk.gdshader draws the 05 water below with a per-chunk lift; at ¼× the trees'
              canopy mass baked in; LRU (ground.chunk_cache_max), ½/¼ textures below 1×
WallBaker     per-row strips (tops along the run, faces where the south is open, hill faces) in ColonyView/World
layers        trees (tree_season.gdshader: season Lab shift, bare winter, snow; hidden at ¼×), structures, items,
              animals, visitors, people: nodes at their base point in the y-sorted World
people        route I frame by activity, person_variant.gdshader (Lab shift through the fixed skin/hair mask),
              outline for selection/drafted/night halo; LOD below 1× = lod_markers.gd (MultiMesh; ring = household)
   ▼  tone
tone.gd (ENV1 ground cap at load, winter desaturation + snow, drought) · daylight_controller.gd (CanvasModulate
value × 0.55 with the blue shift, additive hearth PointLight2D within 5 tiles)
```

- **Params**: every tunable is in `assets/v8art/render_params.json` (camera, ground, water, walls, people, lod, light,
  season, zones, sites, labels, debug); `ArtData.reload_params()` on F12 in debug builds re-reads it and bumps
  `params_version()`, which every layer watches.
- **D19 terrain separation (ADR 0035)**: the ground-only ENV1 cap is mean luminance 0.38/saturation 0.30;
  water has a separate blue contrast floor. `ground.map_mode` supplies terrain colours and weights after ½/¼
  texture reduction. Standing trees retain their source textures. `mart2_ground_test.gd -- --rendered` verifies
  actual GPU pixels; its headless mode explicitly skips these claims.
- **Adding content** (`assets/v8art/ADDING_ART.md`): a new wall material or style is a material JSON + texture; a new
  item, animal or structure is a manifest row (via `tools/v8/art_import.json`) plus a bindings row. The renderer code
  does not change; `tools/v8/test_mart2_extensibility.py` (gate step 33) fails on a content literal in the renderer.
- **Import**: `tools/v8/art_import.py` copies from the art line's commit by git object reads, bytes unchanged;
  `--verify` (gate step 31) checks every file's hash and size against the manifest.
- **Snapshot schema v7** (`docs/v8/snapshot_schema.md`): `get_agent_snapshot` gains `sexes`, `look_ids`, `variants`;
  `get_space` gains `wall_/floor_materials`, `_styles`, `_categories`.
- **Appearance**: `look = ((age × 2 + sex) × 3 + top) × 3 + hair`, `variant` 0–5 (elders 3 or 4) from
  `sim-bridge/data/appearance.ron` and the AgentId; the probe's `--people-json` carries the same (renderer agreement,
  gate step 23).
- **Tests**: `mart2_ground_test.gd` and `mart2_people_test.gd` (headless, gate step 34), `resolver_parity`
  (gate step 32), `mart2_ui_test.gd` (windowed, run by hand: 42 checks), `capture.gd --mart2-set` (the owner set),
  `mart2_fps.sh` (FPS budgets).


## 15. Immutable content catalog (M-EXT core, ADR 0038)

`rust/crates/sim-core/src/colony/content/` owns pure catalog construction, versioned RON parsing, stable u32 item/blueprint/recipe IDs, ordered indexes, cross-reference validation and one sticky process installation. `items.ron`, `blueprints.ron`, `recipes.ron` and selected knowledge sources are resolved together. Categories/capabilities select mechanics; display names and construction styles remain data. Pressure and animal migration plus the complete seven-family extension proof are still pending in this core checkpoint.

The CLI's `sim-test/src/content_boot.rs` and bridge's `ffi/content_boot.rs` acquire selected source bytes before typed rules or engine construction. `scripts/core/main_bootstrap.gd` invokes the controlled factory and presentation preflight before main children attach. `scripts/ui/content_context.gd` exposes registry metadata to UI consumers. Full-width content columns use PackedInt64Array; mechanical state codes remain separate. Door classification is an explicit `get_space` flag, independent of its style name.

`colony/content_state.rs` retains sparse novel completed blueprint/style identities per physical layer and selected non-default cutters. Historical worlds retain their prior completed physical/material projections and empty completed/cutter maps; migrated standing trees now retain historical oak through an explicit persisted tree-material default. `save/content_validation.rs` rejects mismatched layers, styles, bounds and capabilities rather than repairing them silently. Flora membership/distributions in `rust/crates/sim-core/data/colony/flora.ron` resolve existing physical Wood identities; worldgen uses an independent keyed draw while preserving placement and log yields. Felling and renderer reads consume the saved default/override, never a current-data reroll. Frozen resource and Chronicle readers live in `sim-engine/src/legacy_v7.rs` and `sim-core/src/chronicle/legacy_v7.rs`; they preserve historical field order and indexes directly.

Selected source revision includes exact logical source names and bytes, including supplied presentation files. It is an identity checksum, not authentication. Replacing a selected pack requires a fresh process; unknown or mismatched saves fail before typed decoding. Bundled render-parameter F12 reload remains independent of the frozen selected extension sources. See `docs/decisions/0038-immutable-content-catalog.md` for compatibility limits and the required per-commit safety net.

## Colony decision architecture (M-CORE, ADR 0046–0047)

**Current staging:** p6 `fbc9a9babc9bb5df1d06ca1e655901e480521f66` commits colony `ColonyMindSystem` and baseline `AgentDecisionSystem` registration at priority 125, with decoded-rule save migration. P7 moves the legacy selector to `runtime/baseline/decision` and separately removes dead colony gates/seam and the live old-shadow factory. This isolation is uncommitted; final checks and post-isolation identity remain pending. Historical M1–M5.5 descriptions above retain the pre-switch design, not current colony policy. The generated live system table above describes the baseline constructor.

```text
mind.ron + fact/behaviour registry
  -> MindProgram::load (strict compiler; dictionary-local IDs)
  -> TickContext / ProviderPlan (AgentId-sorted observations; versioned dependencies)
  -> pure band selector (first eligible band; integer score; authored tie order; hysteresis)
  -> registered start / continue / finish effects
  -> routine invitation replay + deferred jobs/targets/claims postpass
  -> MindChoice history -> read-only get_agent_mind -> inspector / Why
```

`sim-core/src/colony/mind/` owns schema, compilation, columnar facts and pure selection. `sim-systems/src/runtime/mind/` owns factual readers, resolved target capabilities, coordination, lifecycle effects and records. A provider may observe eligibility and availability but cannot invoke the old chooser or reserve a target. Effects invalidate shared domains; the coordinator refreshes facts before later same-tick choices. Lifecycle callbacks preserve consumption/arrival boundaries, social snapshots, claim order and existing job/flight/draft ownership. No node-name switch belongs in the coordinator.

Neutral mechanics now live under `runtime/agent_actions/` (`consume`, `needs`, `forage`, `wood`, `targets`, `memory_bias`, `routine`), with direct legacy API reexports. ADR 0039's original consumption extraction is preserved and extended; the legacy cascade still owns its original mutation order. The new routine API accepts an already selected kind/place and uses a node-keyed new-path duration without changing the legacy draw.

Choice records retain dictionary identity, exact witnesses and up to three raw candidates from the winning band. Hysteresis may retain a choice outside the raw top three. Witnesses can include rejected alternatives; UI must distinguish evaluated inputs from the chosen action's causal explanation. History is bounded per person, while critical band changes can enter the chronicle. `ffi/mind.rs` collects records without re-evaluating policy; the inspector caches its argument getter by visible selection and simulation tick.

The current module boundary is colony -> `runtime/mind`, baseline -> `runtime/baseline/decision`, both -> neutral `agent_actions`. The pure move has separate hash proof from dead-gate/seam removal. Final import checks and identity are still pending. No `sim-baseline` crate is introduced: the legacy selector shares crate-private mechanics and settlement/chronicle helpers with the crate that owns registration; a standalone split would need additional helper/registrar extraction to avoid a circular dependency. Supported baseline constructors/save fixtures define the preservation scope, not arbitrary injected colony components. See [ADR 0047](decisions/0047-baseline-isolation.md) and [D20](v8/core/CORE_DECISIONS.md).

## 16. Opt-in policy evaluation observations (B4, ADR 0040)

`sim-test/src/probe/b4.rs` and its child modules retain immutable pre-tick craft inputs and reconcile persistent completion tallies, including deceased agents. Colony probe command/before/after seams record daily, migration, caravan and explicitly unresolved haul observations. This is probe-owned state, not a runtime system or save field. The three B4 CLI options are an explicit complete opt-in; ordinary output remains unchanged when absent. `sim-test/tests/b4_controlled.rs` exercises real cold filtering and paid craft finish boundaries. `tools/v8/b4_score.py` independently validates exact input hashes, full coverage and integer/keyed arithmetic. Natural exposure denominators, controlled mechanisms and outcome acceptance remain separate; raw artifacts live outside the repository.


## 17. M-UX2 defence and observation boundaries (in progress)

`runtime/steward/defence.rs` supplies the priority122/every-tick response before
Flee123; `arming.rs` matches physical weapons and ranks the complete proposed
team. Slow Steward125/360 remains after assignment. Both use ordinary commands;
fast source replay is `before_flight`, ordinary source replay `after_assignment`.
Saved ownership/manual expiry/response/quiet state lives in
`sim-core/src/colony/steward/state.rs`. [ADR0045](decisions/0045-defend-through-player-respecting-steward-orders.md)
records the intentional behavior change and player precedence.

`SimResources.fatal_animal_attacks` is an exact resolver-produced transient
observation queue, cleared at tick start and excluded from serde. The opt-in
probe observer (`sim-test/src/probe/mux2.rs`) consumes it without changing policy.
Native timing counters likewise stay outside simulation state.

Renderer candidates use `scripts/world/colony/marker_layout.gd` for screen-space
marker placement; source implementation does not establish visibility or FPS
acceptance. Final frozen-source captures, frame-time checks, paired held-out
evaluation and the full gate remain pending. Raw M-UX2 evidence stays outside
the repository in `/Users/rexxa/work/mux2-data`.
