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
| sim-core | `rust/crates/sim-core/src/` | data only: ECS components (`components/`), rules (`rules/`: `WorldRules`, one file per sub-rule group, `baseline.rs`), keyed RNG (`rng.rs`), per-world hidden parameters (`variety.rs`), causal log (`causal/`), chronicle (`chronicle.rs`), influence grid (`influence/`), tile grid (`tile/`), material model (`material/`, data for M2), the M1 colony model (`colony/`: items and the log ledger, trees, designations, zones, blueprints, reservations, commands, jobs), the spatial index (`spatial.rs`), the M3 cost map, regions and A* (`nav.rs`) and room detection (`rooms.rs`) |
| sim-engine | `rust/crates/sim-engine/src/lib.rs` | `SimEngine` (hecs `World` + `SimResources` + the priority-sorted system list), the `RuntimeSystem` trait, `Tallies`, the tick loop and its profiler |
| sim-systems | `rust/crates/sim-systems/src/` | every system (`runtime/<domain>/`), their registration (`lib.rs` `register_default_runtime_systems`), the command entry point (`commands.rs`: `apply_command`, `submit`), the job layer (`runtime/jobs/`) and world generation (`worldgen/`: `WorldGenConfig::baseline`, `WorldGenConfig::colony` with `worldgen/colony.rs`, `build_world_with`) |
| sim-bridge | `rust/crates/sim-bridge/src/ffi/` | the Godot class `WorldSimNode` (`#[func]` methods), pure `collect_*` readers behind them, the live engine builder (`build_game_engine`, the colony world), M1 orders and colony reads (`colony.rs`) |
| sim-test | `rust/crates/sim-test/` | `worldsim_probe` (headless measurement, identity; `--scene colony` with `--commands` for scripted orders and replay), `worldsim_perf` (tick timing), `worldsim_systems` (the live system list), integration tests in `tests/` |

Other folders: `scenes/main.tscn` (the only scene), `scripts/` (GDScript,
section 6), `assets/` (fonts, theme, atlas, terrain sheets), `shaders/`,
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
which registers every system below (`register_default_runtime_systems`). Lower
priority runs first; a system runs on ticks where `tick % interval == 0`.
Reads/writes come from each file's `//!` header.

<!-- system-table:begin (generated by tools/v8/system_table.py; do not edit) -->

| # | system | priority | interval (ticks) | file | reads | writes |
|---|---|---|---|---|---|---|
| 1 | `BuildingStampSystem` | 90 | 1 | `rust/crates/sim-systems/src/runtime/influence/building_stamp.rs` | `SimResources.building_event_queue`, `influence_grid` (dimensions), `current_tick` | `SimResources.building_event_queue` (drained), `influence_grid.dirty_regions`, `building_registry`, `causal_log` |
| 2 | `InfluenceUpdateSystem` | 100 | 1 | `rust/crates/sim-systems/src/runtime/influence/update.rs` | `SimResources.influence_grid` (`current`, `dirty_regions`), `tile_grid`, `material_blocking_cache`, `causal_log` (latest `StampDirty`) | `SimResources.influence_grid` (`pending`, drained `dirty_regions`, buffer swap), `causal_log` |
| 3 | `AgentInfluenceSampleSystem` | 110 | 1 | `rust/crates/sim-systems/src/runtime/influence/agent_sample.rs` | `Position`; `SimResources.influence_grid` (current buffer) | `InfluenceSample` |
| 4 | `AgentMovementSystem` | 120 | 1 | `rust/crates/sim-systems/src/runtime/agent/movement.rs` | `AgentState`, `SeekTarget`, `Agent`, `Child`, `Elder`, `Inventory`, `Lineage`, `Founding`, `Temperament`, `PorterTrip`, `Activity`; `SimResources.settlements`, `tile_grid`, `rules`, `nav` (colony), `food_tiles`, `water_tiles`, `sleep_tiles`, `bed_of` | `Position`, `MovementRng`, `SeekTarget` (colony: an unreachable need target moves to the nearest reachable one); `SimResources.haulers`, `nav` (paths, blocked goals, the map when missing) |
| 5 | `BillSystem` | 123 | 60 | `rust/crates/sim-systems/src/runtime/jobs/bills.rs` | `Job`, `Carry`; `SimResources.colony` (workshops and bills, ground items, zones, reservations), `rules.colony` | `SimResources.colony.workshops` (active bill, shelf), ground items (spilled shelf items) |
| 6 | `JobAssignSystem` | 124 | 1 | `rust/crates/sim-systems/src/runtime/jobs/assign.rs` | `Agent`, `Position`, `AgentState`, `Job`, `Child`, `PorterTrip`; `SimResources.colony` (trees, designations, ground items, zones, reservations, footprints), `combat_pairs`, `rules.colony`; colony sites (`ColonySite`, `MaterialNeeds`, `ConstructionSite`) | `Job`, `SeekTarget`, `AgentState` (`Seeking { Job }`); `SimResources.colony.reservations`, `colony.tallies`, `spatial.colony` (rebuilt when the overlay changed) |
| 7 | `AgentDecisionSystem` | 125 | 1 | `rust/crates/sim-systems/src/runtime/decision/system.rs` | `Agent`, `Position`, `AgentState`, `SeekTarget`, `Hunger`, `Thirst`, `Sleep`, `Social`, `Inventory`, `Memory`, `Temperament`, `Lineage`, `Founding`, `Child`, `Elder`, `SettlementMigrant`, `PorterTrip`, `ConstructionSite`; `SimResources.rules`, `tile_grid`, `food_tiles`, `water_tiles`, `sleep_tiles`, `wood_tiles`, `settlements`, `settlement_relations`, `site_owner`, `social_retry_after`, `causal_log` | `AgentState`, `Hunger`, `Thirst`, `Sleep`, `Inventory`, `SeekTarget`, `SettlementMigrant`, routine `Activity`/`ActivityRest`/`RoutineObservationBatch`; `SimResources.food_tiles`, `water_tiles`, `sleep_tiles`, `wood_tiles`, `tallies`, `causal_log`, `chronicle`, `combat_pairs`, `social_chase_since`, `social_retry_after`, `settlement_relations` |
| 8 | `StaleSeekTargetSystem` | 126 | 1 | `rust/crates/sim-systems/src/runtime/resource_regen/mod.rs` | `AgentState`, `Position`, `SeekTarget`, food/water/sleep tile maps | `SeekTarget` (insert/remove), `AgentState` |
| 9 | `JobExecuteSystem` | 127 | 1 | `rust/crates/sim-systems/src/runtime/jobs/execute.rs` | `Agent`, `Job`, `AgentState`, `Position`, `Carry`; `SimResources.colony` (trees, designations, ground items, zones), `rules.colony`, `current_tick`; colony sites (`ColonySite`, `MaterialNeeds`, `ConstructionSite`) | `Job`, `Carry`, `AgentState`, `SeekTarget`, `MaterialNeeds` (delivered); `SimResources.colony` (trees, trees_version, designations, ground items, reservations, item_ledger, overlay_version, tallies) |
| 10 | `HungerDecaySystem` | 130 | 1 | `rust/crates/sim-systems/src/runtime/needs/hunger_decay.rs` | `Hunger`, `Cold`; `SimResources.rules.society.firewood` | `Hunger` |
| 11 | `ThirstDecaySystem` | 131 | 1 | `rust/crates/sim-systems/src/runtime/needs/thirst_decay.rs` | `Thirst` | `Thirst` |
| 12 | `SleepDecaySystem` | 132 | 1 | `rust/crates/sim-systems/src/runtime/needs/sleep_decay.rs` | `Sleep`, `AgentState`, `Position`; `SimResources.colony.rooms` (M3) | `Sleep` |
| 13 | `ConstructionSystem` | 133 | 1 | `rust/crates/sim-systems/src/runtime/construction/construction_system.rs` | `Position`, `AgentState`, `ConstructionSite`, `MaterialNeeds`, `ColonySite`; `SimResources.tile_grid`, `causal_log` (`ConstructionStarted`), `site_owner`, `settlements`, `rules` | `ConstructionSite` (progress, despawn), `AgentState`; `SimResources.causal_log`, `building_registry`, `building_kinds`, `building_owner`, `site_owner`, sleep tiles, `colony` (footprints, item ledger, tallies) |
| 14 | `SocialInteractionSystem` | 134 | 1 | `rust/crates/sim-systems/src/runtime/social/social_interaction_system.rs` | `Agent`, `Position`, `AgentState`; `SimResources.causal_log` (`SocialInteractionStarted`), `interaction_progress` | `AgentState`, `Social`; `SimResources.interaction_progress`, `relationships` (familiarity), `causal_log`, `tallies` |
| 15 | `SocialDecaySystem` | 135 | 1 | `rust/crates/sim-systems/src/runtime/needs/social_decay.rs` | `Social` | `Social` |
| 16 | `MemorySystem` | 136 | 1 | `rust/crates/sim-systems/src/runtime/memory/memory_system.rs` | `Agent`; `SimResources.causal_log` (this tick's events, `MemoryRecalled`) | `Memory` |
| 17 | `CombatSystem` | 137 | 1 | `rust/crates/sim-systems/src/runtime/combat/combat_system.rs` | `Agent`, `Position`, `BodyHealth`; `SimResources.combat_pairs`, `causal_log` (`CombatStarted`), `settlements` | `BodyHealth`, `AgentState`, `Memory`; `SimResources.combat_pairs`, `combat_progress`, `relationships` (hostility), `causal_log`; deaths via `despawn_agent` |
| 18 | `SettlementSystem` | 138 | 1 | `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs` | `Agent`, `Position`, `Hunger`, `Thirst`, `Kin`, `Sex`, `Lineage`, `Temperament`, `Founding`; `SimResources.building_registry`, `building_kinds`, `causal_log`, `fission_log`, `rules`, `due_births` (M1.5, under `BirthRule::Fertility`) | `SimResources.settlements`, `building_owner`, `dissolutions`, `chronicle`, `causal_log`, `tallies`, `due_births` (drained); spawns newborns (needs, `Memory`, `Lineage`, `Kin`, `Sex`, `Child`, `Life`); mother's `Kin.last_birth_tick` |
| 19 | `StarvationSystem` | 139 | 1 | `rust/crates/sim-systems/src/runtime/survival/mod.rs` | `Agent`, `Position`, `Hunger`, `Thirst`, `Cold`, `ExposureClock`; `SimResources.rules.society.exposure` | `BodyHealth`; deaths via `despawn_agent` (despawn; `SimResources.settlements` rosters, `relationships`, social and combat maps, `bed_of`, `haulers`, `recent_deaths`, `chronicle`; M1: a worker's load and reservations in `colony`) |
| 20 | `ResourceRegenSystem` | 140 | 120 | `rust/crates/sim-systems/src/runtime/resource_regen/mod.rs` | `SimResources.*_source_max`, `wood_spot_area`, `wild_regions`, `settlements`, `rules` | `SimResources` food/water/sleep/wood tile maps, regen accumulators and cursors, `tallies` |
| 21 | `StockpileDepositSystem` | 141 | 10 | `rust/crates/sim-systems/src/runtime/settlement/stockpile_deposit.rs` | `Agent`, `Position`, `PorterTrip`; `SimResources.settlements` (rosters, formation tile), `rules` | `Inventory`; `SimResources.settlements` (stockpile), `haulers`, `tallies` |
| 22 | `StockpileConsumeSystem` | 142 | 5 | `rust/crates/sim-systems/src/runtime/settlement/stockpile_consume.rs` | `Agent`, `Position`, `AgentState`, `Founding`, `Hunger`; `SimResources.settlements`, `rules.society` (`meals`, `relief_radius`) | `Hunger`; `SimResources.settlements` (stockpile food), `tallies` |
| 23 | `ShelterPlanningSystem` | 143 | 1 | `rust/crates/sim-systems/src/runtime/settlement/shelter_planning.rs` | `ConstructionSite`; `SimResources.settlements`, `building_registry`, `building_kinds`, `building_owner`, `tile_grid`, resource tile maps | spawns `ConstructionSite`; `SimResources.site_owner`, `shelter_plan_skips` |
| 24 | `MaturationSystem` | 144 | 60 | `rust/crates/sim-systems/src/runtime/agent/maturation.rs` | `Agent`, `Child`; `SimResources.current_tick` | `Child` (removed), `Hunger`, `Thirst`, `Sleep` (adult growth rates); `SimResources.chronicle` |
| 25 | `PairingSystem` | 144 | 1 | `rust/crates/sim-systems/src/runtime/agent/pairing.rs` | `Agent`, `Kin`, `Sex`, `Child`, `Elder`, `BodyHealth`; `SimResources.relationships` (familiarity), `settlements` | `Kin` (partner), `FamilyObservations`; `SimResources.chronicle` |
| 26 | `BedAssignmentSystem` | 145 | 1 | `rust/crates/sim-systems/src/runtime/agent/beds.rs` | `Agent`, `Kin`, `Child`, `BodyHealth` (households); `SimResources.building_owner`, `building_kinds`, `settlements`, `rules.society.family`, `rules.colony` | `SimResources.bed_of` |
| 27 | `AgingSystem` | 146 | 60 | `rust/crates/sim-systems/src/runtime/agent/aging.rs` | `Agent`, `Life`, `Position`, `Elder` | `Elder` (inserted); `SimResources.chronicle`; deaths via `despawn_agent` |
| 28 | `HearthFeedSystem` | 146 | 60 | `rust/crates/sim-systems/src/runtime/jobs/hearth.rs` | `SimResources.colony` (ground items, zones, reservations), `settlements` (formation tile, stockpile), `rules.society.firewood`, the clock | `SimResources.settlements` (stockpile wood), `colony` (ground items, item ledger `burned`) |
| 29 | `FireSystem` | 147 | 60 | `rust/crates/sim-systems/src/runtime/settlement/fire.rs` | `Agent`, `Founding`, `PorterTrip`, `Child`, `Elder`, `Hardiness`, `Hunger`, `Position`, `AgentState`; `SimResources.settlements`, `rules`, `colony.rooms` (M3) | `Cold`, `ExposureClock`, `BodyHealth`; `SimResources.fires`, `settlements` (stockpile wood), `tallies`; deaths via `despawn_agent` |
| 30 | `FissionSystem` | 148 | 1 | `rust/crates/sim-systems/src/runtime/settlement/fission.rs` | `Agent`, `Position`, `Child`, `Elder`, `Lineage`, `Kin`, `Temperament`, `PorterTrip`, `AgentState`; `SimResources.settlements`, `wild_regions`, `tile_grid`, `rules` | `Founding`, `AgentState`, `SeekTarget`; `SimResources.settlements`, `building_kinds`, `building_event_queue`, `retired_hearths`, `fission_log`, `fission_checks`, `dissolutions`, `chronicle` |
| 31 | `RelationsSystem` | 149 | 1 | `rust/crates/sim-systems/src/runtime/settlement/relations.rs` | `Agent`, `Position`, `AgentState`, `Hunger`, `Temperament`, `Child`, `Elder`, `Founding`; `SimResources.settlements`, `rules` | `PorterTrip`, `Inventory`, `AgentState`, `SeekTarget`; `SimResources.settlement_relations`, `settlements` (stockpiles), `chronicle`, `tallies` |
| 32 | `FertilitySystem` | 150 | 1440 | `rust/crates/sim-systems/src/runtime/agent/fertility.rs` | `Agent`, `Sex`, `Kin`, `Life`, `Lineage`, `Child`, `Pregnant`, `Hunger`, `Thirst`, `BodyHealth`; `SimResources.settlements`, `rules.society.birth`, `rules.calendar`, `rules.clock_offset_ticks`, `world_seed` | `Pregnant` (inserted at a conception, removed when due); `SimResources.due_births` |
| 33 | `RoomSystem` | 152 | 1 | `rust/crates/sim-systems/src/runtime/rooms.rs` | `Agent`, `Position`; `SimResources.tile_grid`, `building_kinds`, `settlements`, `colony` (beds, zones, occupied, room overrides) | `SimResources.colony` (rooms, room_of_tile, next_room_id, rooms_announced, rooms_dirty, rooms_version); chronicle |
| 34 | `InfluenceVisualizationSystem` | 1000 | 6 | `rust/crates/sim-systems/src/runtime/influence/visualization.rs` | `SimResources.influence_grid` (current Warmth and Danger buffers), `current_tick` | its own `VisualizationDigest` (no world or resource state) |

<!-- system-table:end -->

## 4. Bridge API (`WorldSimNode`, grouped by concern)

All methods live on the one Godot class `WorldSimNode` (signal `world_reset`
after a reset or a load). The exact exported list is checked by gate step 13, currently
`python3 tools/v8/bridge_api.py . --check docs/v8/bridge_api_m3.txt` (M3; M2 checked `docs/v8/bridge_api_m2.txt`, M1.5 `docs/v8/bridge_api_m1_5.txt`, M1 `docs/v8/bridge_api_m1.txt`, V8-0 `docs/v8/bridge_api_v7_final.txt` minus `docs/v8/bridge_api_removed.txt`)
(a pass that changes the API updates this command; section 6).

| concern | methods |
|---|---|
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
| calendar and ages (M1.5) | `get_time_snapshot`'s `year_index`, `season_index`, `day_of_year`, `days_per_year`; `get_world_info`'s `ticks_per_year`; the agent snapshot's `ages_years`, `infants`, `pregnant_days` columns |

Every getter, key, type, id space and row order is documented in
`docs/v8/snapshot_schema.md` (schema version 3, M3); the gate step
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
2. Pick the file of `rust/crates/sim-bridge/src/ffi/` for the concern: `node.rs` (the class, process loop, speed and every `#[func]`), `snapshot.rs` (per-frame arrays: agents, construction, resources, recent deaths, clock, tallies, world info, terrain, buildings), `reads_agent.rs` (agent detail, temperament, relationships), `reads_settlement.rs` (settlement and society), `causal.rs`, `chronicle.rs`, `family.rs`, `relations.rs` (village relations, porters), `activity.rs`, `engine_build.rs` (engine construction), `colony.rs` (M1 orders and colony reads), `mod.rs`. A new concern gets a new file `<concern>.rs` with a `//!` header, a `pub mod <concern>;` line in `ffi/mod.rs` and a `pub use` of the collector.
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
| `scripts/core/` | `locale.gd` (autoload `Locale`), `sim_reads.gd` (`SimReads`: every bridge read), `settlement_selection.gd` (selected village) |
| `scripts/world/colony/` | the M1 view (node `ColonyView` in `scenes/main.tscn`), drawn from one sheet (`assets/v8/tiles.png` + `tiles.json`, painted in code by `tools/v8/gen_tiles.py`, unit-tested by `tools/v8/test_gen_tiles.py`; M1-ART): `view.gd` (sheet rects and palettes, TileSet builder, tile/keyed hashes, the ground's value noise), `terrain_layer.gd` (noise-clustered grass, forest floor, eroded clearing with turned dither edges, water autotile: 4-bit land mask → `bank_<mask>`, diagonal → `bank_in_*`, then shallow, then deep; rock tops and cliffs), `tree_layer.gd` (24×24 crowns drawn by y-ordered 16×16-tile chunks, shadows except at 1×, stumps for a day, bushes), `building_layer.gd` (hut, lean-to, two-frame hearth; M3: walls by neighbour mask and doors at z 3, a z 1 child "Floors" for packed floors, dug rock and bedrolls, rebuilt when `get_space.version` moves), `overlay_layer.gd` (z 1 child: stockpile floors, log piles, blueprint ghosts; z 4: chop, quarry, dig and deconstruct marks, blueprint and space-site label and bar, hover count with "방: 창고 n" and "지붕 아래"), `room_layer.gd` (M3, child "Rooms" of the overlay: R toggles role tints at z 2 and room labels at z 4; keeps the tile → room index for clicks and the inspector), `agent_layer.gd` (people as layered sprites: halo, logs behind, skin/hair/tunic, outline, log in front, tool; stand/stride/sleep frames; sleepers on bedrolls; hit test), `order_tool.gd` (F1–F10 modes: chop, stockpile, build menu 1–9 — space kinds by drag line or Shift-rectangle with the `get_place_preview` ghost —, cancel, quarry, dig F8, deconstruct F10; drag rectangles, the `cmd_*` calls; F5/F9 stay save/load). Draw order by z: terrain 0, ground overlay, floors and stumps 1, canopy and room tints 2, buildings and walls 3, marks and room labels 4, people 5 |
| `scripts/world/` | `camera_controller.gd` (zoom 1–4, pan, speed keys 1–4, pause), `daylight_controller.gd` (+ `wood_visuals.gd`), helpers the panels still use (`art_hash.gd`, `art_sprite_library.gd`, `porter_presentation.gd`) |
| `scripts/ui/hud/` | `hud_topbar.gd`, `order_bar.gd` (F1–F5 buttons and the order toast) |
| `scripts/ui/panels/` | `chronicle_panel.gd`, `agent_inspector_panel.gd`, `causal_panel.gd`, `priority_panel.gd` (M2 work priorities, 7 work types since M3), `workshop_panel.gd` (M2 bills of the selected workshop), `room_panel.gd` (M3: a room's role dropdown → `cmd_set_room_role`, tiles, beds, doors, covered items; made by `order_bar.gd`), `art_ui.gd` (shared panel style) |
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
- **Benchmark** (`worldsim_perf bench`): five scenes on `WorldGenConfig::colony_stress(15100, N, side)` under
  `WorldRules::colony`, each in its own process for a clean resident size, 1,440 warm-up and 2,880 profiled
  ticks, 3 repeats, medians; fed like `quick`. `bench-summarise` merges interleaved sessions of two binaries.
- **Save files** (`rust/crates/sim-systems/src/save/`): magic `WORLDSIM`, `FORMAT_VERSION`, bincode of the tick,
  all of `SimResources` (hash collections in key order), the hecs world (archetypes in creation order, entity ids
  and generations, the allocator's free list) and each system's `save_state`.

## 9. Items and jobs (M2, ADR 0019, ADR 0020)

The colony's economic loop, all under `ColonyRule::economy` (`None` in the baseline and in M1's rules):

```
worldgen (colony.rs: nodes, material table)
   │  boulders / flint outcrops (ColonyState.nodes)        trees (M1)        deadwood (SimResources.wood_tiles)
   ▼                                                        ▼                  ▼
orders (commands.rs): DesignateQuarry · DesignateChop · AddStockpile · PlaceBlueprint(workshop) · AddBill … · SetWorkPriority
   ▼
JobAssignSystem 124 — passes by priority level (1..4), within a pass WorkType order Craft > Build > Quarry > Chop > Split > Haul
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
