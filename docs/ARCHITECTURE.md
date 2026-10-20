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
| sim-core | `rust/crates/sim-core/src/` | data only: ECS components (`components/`), rules (`rules/`: `WorldRules`, one file per sub-rule group, `baseline.rs`), keyed RNG (`rng.rs`), per-world hidden parameters (`variety.rs`), causal log (`causal/`), chronicle (`chronicle.rs`), influence grid (`influence/`), tile grid (`tile/`), material model (`material/`, data for M2), the M1 colony model (`colony/`: items and the log ledger, trees, designations, zones, blueprints, reservations, commands, jobs) |
| sim-engine | `rust/crates/sim-engine/src/lib.rs` | `SimEngine` (hecs `World` + `SimResources` + the priority-sorted system list), the `RuntimeSystem` trait, `Tallies`, the tick loop and its profiler |
| sim-systems | `rust/crates/sim-systems/src/` | every system (`runtime/<domain>/`), their registration (`lib.rs` `register_default_runtime_systems`), the command entry point (`commands.rs`: `apply_command`, `submit`), the job layer (`runtime/jobs/`) and world generation (`worldgen/`: `WorldGenConfig::baseline`, `WorldGenConfig::colony` with `worldgen/colony.rs`, `build_world_with`) |
| sim-bridge | `rust/crates/sim-bridge/src/ffi/` | the Godot class `WorldSimNode` (`#[func]` methods), pure `collect_*` readers behind them, the live engine builder (`build_game_engine`, the colony world), M1 orders and colony reads (`colony.rs`) |
| sim-test | `rust/crates/sim-test/` | `worldsim_probe` (headless measurement, identity; `--scene colony` with `--commands` for scripted orders and replay), `worldsim_perf` (tick timing), `worldsim_systems` (the live system list), integration tests in `tests/` |

Other folders: `scenes/main.tscn` (the only scene), `scripts/` (GDScript,
section 6), `assets/` (fonts, theme, atlas, terrain sheets), `shaders/`,
`localization/` (section 7), `tools/v8/` (gate, identity, perf, smoke,
capture, inventory, docs checks), `docs/` (this file and the owner docs).

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
  one step): the game then runs slower than the chosen speed and never
  catches up later. Speed s needs at least 1.5 × s frames per second
  (4×: 6 fps).
- Speed: in `scripts/world/camera_controller.gd` the keys `1`/`2`/`3`/`4`
  set 1×/2×/3×/4× = 12/24/36/48 ticks per real second, so a simulated day
  lasts 120/60/40/30 s (`seconds_per_day`); `P` pauses by disabling the
  WorldSim node's processing; the top bar shows the speed and the pause.
  `set_sim_speed` scales only the time fed to the accumulator and clamps to
  0–4× (0 also stops the clock); it never changes the tick length.
- GDScript never writes simulation state. The only writes from Godot are
  bridge commands: the M1 orders (`cmd_*`, below), `on_building_placed`,
  `reset_world`, `reset_world_stress`, `set_sim_speed` and the tooling
  `advance_ticks` (`SimReads.MUTATORS` holds all but `set_sim_speed`, which
  `scripts/world/camera_controller.gd` calls directly on the number keys).
- Calendar now (V7 calendar, unchanged by M1): one tick is one simulated
  minute; a day is 1,440 ticks (120 s of real time at 1×); a season is 7
  days (10,080 ticks); a year is 4 seasons = 28 days (40,320 ticks ≈ 56
  minutes at 1×).
- The displayed day, season and time of day count from the display clock
  `current_tick + clock_offset_ticks` (480: tick 0 is 08:00 of day 1).
  `get_time_snapshot` gives `day_index = (tick + 480) / 1440`,
  `season = (day_index / days_per_season) % 4` and
  `day_of_season = day_index % days_per_season + 1`. Systems run on the raw
  tick (`current_tick % interval == 0`, no offset), so a daily (1,440-tick)
  system runs at 08:00 display time.
- Ages are in simulated days today (`AgingRule`, `LifeStageRule` in
  `rust/crates/sim-core/src/rules/baseline.rs`): a child is adult after
  5,760 ticks (4 days), lifespans average about 58 days, founders start
  6–40 days old.
- ADR 0004 (`docs/decisions/0004-compressed-year.md`): the tick half is
  decided and live since M1 (one tick = one minute, 12 ticks per second at
  1×, a 2-minute day). The calendar half (6-day seasons, a 24-day year,
  ages in years, aging and fertility recalibrated) is milestone M1.5 in
  `docs/ROADMAP.md`; until then the V7 calendar above holds.

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
         a job holder keeps its job unless the decision picks a survival or social choice;
         then the job is cancelled (CancelReason::Needs) and its reservations released
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
- The decision system gains only the seam: about 25 lines that call
  `jobs::seam::{holders, hold_for_decision, settle_after_decision}` and the
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
| 4 | `AgentMovementSystem` | 120 | 1 | `rust/crates/sim-systems/src/runtime/agent/movement.rs` | `AgentState`, `SeekTarget`, `Agent`, `Child`, `Elder`, `Inventory`, `Lineage`, `Founding`, `Temperament`, `PorterTrip`, `Activity`; `SimResources.settlements`, `tile_grid`, `rules` | `Position`, `MovementRng`; `SimResources.haulers` |
| 5 | `JobAssignSystem` | 124 | 1 | `rust/crates/sim-systems/src/runtime/jobs/assign.rs` | `Agent`, `Position`, `AgentState`, `Job`, `Child`, `PorterTrip`; `SimResources.colony` (trees, designations, ground items, zones, reservations, footprints), `combat_pairs`, `rules.colony`; colony sites (`ColonySite`, `MaterialNeeds`, `ConstructionSite`) | `Job`, `SeekTarget`, `AgentState` (`Seeking { Job }`); `SimResources.colony.reservations`, `colony.tallies` |
| 6 | `AgentDecisionSystem` | 125 | 1 | `rust/crates/sim-systems/src/runtime/decision/system.rs` | `Agent`, `Position`, `AgentState`, `SeekTarget`, `Hunger`, `Thirst`, `Sleep`, `Social`, `Inventory`, `Memory`, `Temperament`, `Lineage`, `Founding`, `Child`, `Elder`, `SettlementMigrant`, `PorterTrip`, `ConstructionSite`; `SimResources.rules`, `tile_grid`, `food_tiles`, `water_tiles`, `sleep_tiles`, `wood_tiles`, `settlements`, `settlement_relations`, `site_owner`, `social_retry_after`, `causal_log` | `AgentState`, `Hunger`, `Thirst`, `Sleep`, `Inventory`, `SeekTarget`, `SettlementMigrant`, routine `Activity`/`ActivityRest`/`RoutineObservationBatch`; `SimResources.food_tiles`, `water_tiles`, `sleep_tiles`, `wood_tiles`, `tallies`, `causal_log`, `chronicle`, `combat_pairs`, `social_chase_since`, `social_retry_after`, `settlement_relations` |
| 7 | `StaleSeekTargetSystem` | 126 | 1 | `rust/crates/sim-systems/src/runtime/resource_regen/mod.rs` | `AgentState`, `Position`, `SeekTarget`, food/water/sleep tile maps | `SeekTarget` (insert/remove), `AgentState` |
| 8 | `JobExecuteSystem` | 127 | 1 | `rust/crates/sim-systems/src/runtime/jobs/execute.rs` | `Agent`, `Job`, `AgentState`, `Position`, `Carry`; `SimResources.colony` (trees, designations, ground items, zones), `rules.colony`, `current_tick`; colony sites (`ColonySite`, `MaterialNeeds`, `ConstructionSite`) | `Job`, `Carry`, `AgentState`, `SeekTarget`, `MaterialNeeds` (delivered); `SimResources.colony` (trees, trees_version, designations, ground items, reservations, item_ledger, overlay_version, tallies) |
| 9 | `HungerDecaySystem` | 130 | 1 | `rust/crates/sim-systems/src/runtime/needs/hunger_decay.rs` | `Hunger`, `Cold`; `SimResources.rules.society.firewood` | `Hunger` |
| 10 | `ThirstDecaySystem` | 131 | 1 | `rust/crates/sim-systems/src/runtime/needs/thirst_decay.rs` | `Thirst` | `Thirst` |
| 11 | `SleepDecaySystem` | 132 | 1 | `rust/crates/sim-systems/src/runtime/needs/sleep_decay.rs` | `Sleep` | `Sleep` |
| 12 | `ConstructionSystem` | 133 | 1 | `rust/crates/sim-systems/src/runtime/construction/construction_system.rs` | `Position`, `AgentState`, `ConstructionSite`, `MaterialNeeds`, `ColonySite`; `SimResources.tile_grid`, `causal_log` (`ConstructionStarted`), `site_owner`, `settlements`, `rules` | `ConstructionSite` (progress, despawn), `AgentState`; `SimResources.causal_log`, `building_registry`, `building_kinds`, `building_owner`, `site_owner`, sleep tiles, `colony` (footprints, item ledger, tallies) |
| 13 | `SocialInteractionSystem` | 134 | 1 | `rust/crates/sim-systems/src/runtime/social/social_interaction_system.rs` | `Agent`, `Position`, `AgentState`; `SimResources.causal_log` (`SocialInteractionStarted`), `interaction_progress` | `AgentState`, `Social`; `SimResources.interaction_progress`, `relationships` (familiarity), `causal_log`, `tallies` |
| 14 | `SocialDecaySystem` | 135 | 1 | `rust/crates/sim-systems/src/runtime/needs/social_decay.rs` | `Social` | `Social` |
| 15 | `MemorySystem` | 136 | 1 | `rust/crates/sim-systems/src/runtime/memory/memory_system.rs` | `Agent`; `SimResources.causal_log` (this tick's events, `MemoryRecalled`) | `Memory` |
| 16 | `CombatSystem` | 137 | 1 | `rust/crates/sim-systems/src/runtime/combat/combat_system.rs` | `Agent`, `Position`, `BodyHealth`; `SimResources.combat_pairs`, `causal_log` (`CombatStarted`), `settlements` | `BodyHealth`, `AgentState`, `Memory`; `SimResources.combat_pairs`, `combat_progress`, `relationships` (hostility), `causal_log`; deaths via `despawn_agent` |
| 17 | `SettlementSystem` | 138 | 1 | `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs` | `Agent`, `Position`, `Hunger`, `Thirst`, `Kin`, `Sex`, `Lineage`, `Temperament`, `Founding`; `SimResources.building_registry`, `building_kinds`, `causal_log`, `fission_log`, `rules` | `SimResources.settlements`, `building_owner`, `dissolutions`, `chronicle`, `causal_log`, `tallies`; spawns newborns (needs, `Memory`, `Lineage`, `Kin`, `Sex`, `Child`, `Life`); mother's `Kin.last_birth_tick` |
| 18 | `StarvationSystem` | 139 | 1 | `rust/crates/sim-systems/src/runtime/survival/mod.rs` | `Agent`, `Position`, `Hunger`, `Thirst`, `Cold`, `ExposureClock`; `SimResources.rules.society.exposure` | `BodyHealth`; deaths via `despawn_agent` (despawn; `SimResources.settlements` rosters, `relationships`, social and combat maps, `bed_of`, `haulers`, `recent_deaths`, `chronicle`; M1: a worker's load and reservations in `colony`) |
| 19 | `ResourceRegenSystem` | 140 | 120 | `rust/crates/sim-systems/src/runtime/resource_regen/mod.rs` | `SimResources.*_source_max`, `wood_spot_area`, `wild_regions`, `settlements`, `rules` | `SimResources` food/water/sleep/wood tile maps, regen accumulators and cursors, `tallies` |
| 20 | `StockpileDepositSystem` | 141 | 10 | `rust/crates/sim-systems/src/runtime/settlement/stockpile_deposit.rs` | `Agent`, `Position`, `PorterTrip`; `SimResources.settlements` (rosters, formation tile), `rules` | `Inventory`; `SimResources.settlements` (stockpile), `haulers`, `tallies` |
| 21 | `StockpileConsumeSystem` | 142 | 5 | `rust/crates/sim-systems/src/runtime/settlement/stockpile_consume.rs` | `Agent`, `Position`, `AgentState`, `Founding`, `Hunger`; `SimResources.settlements`, `rules.society` (`meals`, `relief_radius`) | `Hunger`; `SimResources.settlements` (stockpile food), `tallies` |
| 22 | `ShelterPlanningSystem` | 143 | 1 | `rust/crates/sim-systems/src/runtime/settlement/shelter_planning.rs` | `ConstructionSite`; `SimResources.settlements`, `building_registry`, `building_kinds`, `building_owner`, `tile_grid`, resource tile maps | spawns `ConstructionSite`; `SimResources.site_owner`, `shelter_plan_skips` |
| 23 | `MaturationSystem` | 144 | 60 | `rust/crates/sim-systems/src/runtime/agent/maturation.rs` | `Agent`, `Child`; `SimResources.current_tick` | `Child` (removed), `Hunger`, `Thirst`, `Sleep` (adult growth rates); `SimResources.chronicle` |
| 24 | `PairingSystem` | 144 | 1 | `rust/crates/sim-systems/src/runtime/agent/pairing.rs` | `Agent`, `Kin`, `Sex`, `Child`, `Elder`, `BodyHealth`; `SimResources.relationships` (familiarity), `settlements` | `Kin` (partner), `FamilyObservations`; `SimResources.chronicle` |
| 25 | `BedAssignmentSystem` | 145 | 1 | `rust/crates/sim-systems/src/runtime/agent/beds.rs` | `Agent`, `Kin`, `Child`, `BodyHealth` (households); `SimResources.building_owner`, `building_kinds`, `settlements`, `rules.society.family`, `rules.colony` | `SimResources.bed_of` |
| 26 | `AgingSystem` | 146 | 60 | `rust/crates/sim-systems/src/runtime/agent/aging.rs` | `Agent`, `Life`, `Position`, `Elder` | `Elder` (inserted); `SimResources.chronicle`; deaths via `despawn_agent` |
| 27 | `FireSystem` | 147 | 60 | `rust/crates/sim-systems/src/runtime/settlement/fire.rs` | `Agent`, `Founding`, `PorterTrip`, `Child`, `Elder`, `Hardiness`, `Hunger`, `Position`; `SimResources.settlements`, `rules` | `Cold`, `ExposureClock`, `BodyHealth`; `SimResources.fires`, `settlements` (stockpile wood), `tallies`; deaths via `despawn_agent` |
| 28 | `FissionSystem` | 148 | 1 | `rust/crates/sim-systems/src/runtime/settlement/fission.rs` | `Agent`, `Position`, `Child`, `Elder`, `Lineage`, `Kin`, `Temperament`, `PorterTrip`, `AgentState`; `SimResources.settlements`, `wild_regions`, `tile_grid`, `rules` | `Founding`, `AgentState`, `SeekTarget`; `SimResources.settlements`, `building_kinds`, `building_event_queue`, `retired_hearths`, `fission_log`, `fission_checks`, `dissolutions`, `chronicle` |
| 29 | `RelationsSystem` | 149 | 1 | `rust/crates/sim-systems/src/runtime/settlement/relations.rs` | `Agent`, `Position`, `AgentState`, `Hunger`, `Temperament`, `Child`, `Elder`, `Founding`; `SimResources.settlements`, `rules` | `PorterTrip`, `Inventory`, `AgentState`, `SeekTarget`; `SimResources.settlement_relations`, `settlements` (stockpiles), `chronicle`, `tallies` |
| 30 | `InfluenceVisualizationSystem` | 1000 | 6 | `rust/crates/sim-systems/src/runtime/influence/visualization.rs` | `SimResources.influence_grid` (current Warmth and Danger buffers), `current_tick` | its own `VisualizationDigest` (no world or resource state) |

<!-- system-table:end -->

## 4. Bridge API (`WorldSimNode`, grouped by concern)

All methods live on the one Godot class `WorldSimNode` (signal `world_reset`
after a reset). The exact exported list is checked by gate step 13, currently
`python3 tools/v8/bridge_api.py . --check docs/v8/bridge_api_m1.txt` (M1; V8-0 checked against `docs/v8/bridge_api_v7_final.txt` minus `docs/v8/bridge_api_removed.txt`)
(a pass that changes the API updates this command; section 6).

| concern | methods |
|---|---|
| lifecycle and speed | `reset_world`, `reset_world_stress` (benchmark world), `set_sim_speed`, `advance_ticks` (tooling) |
| per-frame snapshots | `get_agent_snapshot`, `get_building_snapshot`, `get_construction_snapshot`, `get_resource_snapshot`, `get_terrain_snapshot`, `get_time_snapshot`, `get_tallies`, `get_world_info`, `get_recent_deaths`, `get_activity_snapshot`, `get_porter_snapshot` |
| one agent | `get_agent_detail`, `get_agent_id`, `get_agent_temperament`, `get_agent_family`, `get_relationship_snapshot`, `get_person` |
| settlements | `get_settlement_snapshot`, `get_society_snapshot`, `get_settlement_relations` |
| chronicle | `get_chronicle_since`, `get_person_chronicle`, `get_settlement_chronicle` |
| causal "why?" | `get_tile_causal_history`, `get_tile_causal_events`, `get_event_chain`, `get_tile_detail`, `get_influence_overlay` |
| commands | `on_building_placed`; M1 orders `cmd_designate_chop`, `cmd_designate_chop_tiles`, `cmd_cancel_area`, `cmd_add_stockpile`, `cmd_remove_zone_area`, `cmd_place_blueprint` (each returns `{accepted, count, reason}` and is logged with the next tick, ADR 0012) |
| colony reads (M1) | `can_place_blueprint`, `get_colony_versions` (poll), `get_trees`, `get_colony_overlay`; the agent snapshot's `jobs`, `carry_kinds`, `carry_counts` columns |

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
5. Test it on `sim_test::baseline_engine(seed)` in the matching domain test file (`rust/crates/sim-test/tests/<domain>_*.rs`) with a threshold on a measured invariant.
6. Regenerate the table above: `python3 tools/v8/system_table.py --write`.
7. If behaviour changes on purpose, re-capture the identity base data and say so in the pass's decisions.

### Add a bridge read

The full procedure with code skeletons is `.claude/skills/worldsim-architecture/SKILL.md` ("B. Add a bridge read") and its `references/procedures.md`; the steps:

1. Check first that the getter does not exist (section 4, `GETTERS` in `scripts/core/sim_reads.gd`).
2. Pick the file of `rust/crates/sim-bridge/src/ffi/` for the concern: `node.rs` (the class, process loop, speed and every `#[func]`), `snapshot.rs` (per-frame arrays: agents, construction, resources, recent deaths, clock, tallies, world info, terrain, buildings), `reads_agent.rs` (agent detail, temperament, relationships), `reads_settlement.rs` (settlement and society), `causal.rs`, `chronicle.rs`, `family.rs`, `relations.rs` (village relations, porters), `activity.rs`, `engine_build.rs` (engine construction), `colony.rs` (M1 orders and colony reads), `mod.rs`. A new concern gets a new file `<concern>.rs` with a `//!` header, a `pub mod <concern>;` line in `ffi/mod.rs` and a `pub use` of the collector.
3. In that file write a pure `pub fn collect_<thing>(&World, &SimResources, …) -> Row/Vec<Row>` (no Godot types; rows sorted by id; say which id space each field uses) and re-export it from `rust/crates/sim-bridge/src/ffi/mod.rs` (`pub use <concern>::collect_<thing>;`). sim-bridge is built as both `cdylib` (what Godot loads) and `rlib` (what sim-test links; `crate-type` in `rust/crates/sim-bridge/Cargo.toml`), so sim-test can call the collector without Godot.
4. In the same file write the marshaller `pub(super) fn <thing>_to_dict(...) -> VarDictionary` (columns as `Packed*Array`, ids as signed i64), as `agent_family_to_dict` in `family.rs`, `rows_to_dict` in `chronicle.rs`, `porter_rows_to_dict` in `relations.rs`.
5. Add a `#[func] fn get_<thing>(&self, …) -> VarDictionary` to the one `#[godot_api] impl WorldSimNode` block in `rust/crates/sim-bridge/src/ffi/node.rs`, importing both with `use super::<concern>::{collect_<thing>, <thing>_to_dict};`. Its body only calls the collector and the marshaller; read-only, at most 14 parameters. `VarDictionary` is the rule; a few getters return a scalar or an array (`get_agent_id -> i64`, `get_tile_causal_history -> VarArray`). A new read returns `VarDictionary` unless a single scalar is the whole answer.
6. Test the collector in `rust/crates/sim-test/tests/engine_bridge_ffi.rs` (the bridge-surface domain file) or a new `engine_<concern>_ffi.rs`: call `sim_bridge::ffi::collect_<thing>` on `sim_test::baseline_engine(seed)` and assert content and order. Unit tests inside sim-bridge (`cargo test -p sim-bridge`) are for pure marshalling helpers only.
7. GDScript side. `SimReads` (`scripts/core/sim_reads.gd`, `class_name SimReads`, node `Main/SimReads`) serves only argument-less getters listed in `GETTERS`: add the name there and read it with `SimReads.read(world_sim, "get_<thing>")` (a literal name). `GETTERS` are cached once per frame; those also in `TICK_GETTERS` (today `get_porter_snapshot`, `get_settlement_relations`) once per simulation tick; terrain until a reset. A getter that takes arguments (for example `get_agent_detail(entity_bits)`) is not in `GETTERS`: call it directly (`world_sim.get_<thing>(args)` or `_world_sim.call("get_<thing>", args)`), never per node per frame. `get_tile_causal_events` has its own helper `SimReads.causal_events(...)`. Name the method in the calling `.gd` file's header comment ("Bridge methods: …").
8. Run, in order: `cd rust && cargo build -p sim-bridge` (Godot loads the debug library this builds; `cargo test` does not rebuild it); the headless import (`bash tools/v8/run_bounded.sh 600 "$GODOT_BIN" --headless --path . --import --editor --quit`) only on a fresh checkout or worktree (twice, judge the second) or after adding or moving assets, scenes, scripts or a `class_name` (adding a `#[func]` to an already imported checkout needs no import); `bash tools/v8/gdscript_strict_check.sh`; `bash tools/v8/smoke.sh <log_dir>`. The strict check's exit 3 compares GDScript calls `world_sim.X(` / `_world_sim.X(` and the names passed to `SimReads.read/mutate` with the `#[func]` names in the Rust source (`tools/v8/bridge_api.py` scans `rust/crates/sim-bridge/src`), so it does not need the build; dynamic `call("name")` strings are not checked by it, only exercised by the smoke.
9. A pure read changes no simulation state, so `bash tools/v8/identity.sh` must still pass. If it fails, the read is not pure: that is a bug.
10. Record the API change. Gate step 13 checks that the set of exported names (`class`, `func`, `signal` lines; order and signatures are not compared) equals an expected list — at V8-0, the V7 base list (`docs/v8/bridge_api_v7_final.txt`) minus `docs/v8/bridge_api_removed.txt`. A pass that adds or removes a `#[func]` writes one list for the pass, `python3 tools/v8/bridge_api.py . > docs/<line>/bridge_api_<pass>.txt`, where `<line>` is the docs folder of the pass's line (`v8` for V8 passes) and `<pass>` the pass name (`m1`), so M1's list is docs/v8/bridge_api_m1.txt. Every later commit of the same pass that changes the API rewrites it. In the same commit, change gate step 13 in `tools/v8/gate.sh` to `python3 tools/v8/bridge_api.py . --check docs/<line>/bridge_api_<pass>.txt` (no `--minus`), update the check command and the method table in section 4 of this file, and add a CHANGELOG line naming the added or removed method. An ADR is needed only if the read encodes a design decision. The V8-0 lists stay as V8-0's record.

## 7. GDScript layout and localization

| folder | contents |
|---|---|
| `scripts/core/` | `locale.gd` (autoload `Locale`), `sim_reads.gd` (`SimReads`: every bridge read), `settlement_selection.gd` (selected village) |
| `scripts/world/colony/` | the M1 view (node `ColonyView` in `scenes/main.tscn`): `view.gd` (shared tile-set builder from `assets/v8/temp/tiles.png` + `tiles.json`, tile hash, click dispatch), `terrain_layer.gd`, `tree_layer.gd`, `building_layer.gd`, `overlay_layer.gd` (designations, zones, blueprints, stacks), `agent_layer.gd` (people, carried logs, hit test), `order_tool.gd` (F1–F5 modes, drag rectangles, the `cmd_*` calls) |
| `scripts/world/` | `camera_controller.gd` (zoom 1–4, pan, speed keys 1–4, pause), `daylight_controller.gd` (+ `wood_visuals.gd`), helpers the panels still use (`art_hash.gd`, `art_sprite_library.gd`, `porter_presentation.gd`) |
| `scripts/ui/hud/` | `hud_topbar.gd`, `order_bar.gd` (F1–F5 buttons and the order toast) |
| `scripts/ui/panels/` | `chronicle_panel.gd`, `agent_inspector_panel.gd`, `causal_panel.gd`, `art_ui.gd` (shared panel style) |
| `scripts/ui/` | text helpers shared by panels (`chronicle_text.gd`, `person_names.gd`, `person_read_cache.gd`) |

Every `.gd` starts with a comment block: purpose and the bridge methods it
calls. UI text goes through `Locale.ltr("KEY")`; keys live in
`localization/<lang>/<category>.json` (the source) and are compiled into
`localization/compiled/<lang>.json` (what the game loads) by
`python3 tools/localization_compile.py --project-root .`.
