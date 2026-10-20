# V8-0 harvest — architecture claims vs. code (raw)

Status: raw harvest for the V8-0 rewrite of the instruction docs. Read-only verification against the working tree on branch `v8/wip` (HEAD `217c122fe`), 2026-10-20. No builds, tests or Godot runs were used. Evidence is file:line or a grep command with its result. Paths are repo-relative.

Sources scanned: `docs/architecture/ARCHITECTURE.md`, `docs/architecture/SYSTEMS.md`, root `CLAUDE.md`, `AGENTS.md`, `rust/crates/sim-core/CLAUDE.md` (the only `*CLAUDE.md` below the root that git tracks), `.claude/skills/worldsim-code/SKILL.md`, `.agents/skills/worldsim-code/SKILL.md` (the same content apart from "Claude Code"/"Codex" and dash/arrow punctuation), `.claude/skills/worldsim-harness/{SKILL,evaluation_criteria,test_templates}.md`. Where a claim also appears only in a code doc comment, the source column says so.

Note: `docs/architecture/ARCHITECTURE.md` and `SYSTEMS.md` describe the **pre-Rust GDScript simulation** (SimulationEngine.gd, EntityManager, SimulationBus and so on). Nearly every concrete claim in them is FALSE now. They are listed in groups below, not row by row.

Verdicts: TRUE = holds in the code now; FALSE = does not hold; PARTLY = the idea holds but the detail stated is wrong or incomplete.

---

## Facts a new ARCHITECTURE.md needs (verified)

- **Crates** (`rust/Cargo.toml` members; dependencies from each `Cargo.toml`):
  - `sim-core`: components (`src/components/*.rs`, 36 files), `rules.rs` + `rules/routine.rs` (`WorldRules` presets), `rng.rs` (keyed draws and stream constants), `causal/` (per-tile causal ring), `chronicle.rs`, `influence/` (8-channel grid), `tile/grid.rs`, `material/` (RON material loader and registry), `variety.rs`, `fission_threshold.rs`. Depends on hecs, serde, glam, ron, ahash, thiserror, arrayvec. Material RON data lives under `rust/crates/sim-core/data/material/<category>/*.ron` (105 files).
  - `sim-engine`: `RuntimeSystem` trait, `SimResources` (all non-ECS world state), `SimEngine` (system list + tick loop), `intervention.rs` (types for god's-hand acts). Depends on sim-core only.
  - `sim-systems`: every `impl RuntimeSystem` (28 of them) under `src/runtime/<domain>/`; `worldgen/` (`build_world_with`, `WorldGenConfig::game_vN`); `subprofile.rs` (feature-gated timing). Depends on sim-core and sim-engine.
  - `sim-bridge`: GDExtension (`crate-type = ["cdylib","rlib"]`), a single Godot class `WorldSimNode` in `src/ffi/world_node.rs`, plus `ffi/{activity,chronicle,family,relations,art_kernels}.rs`. Depends on sim-core, sim-engine, sim-systems, godot (Cargo.toml `godot = "0.4"`, Cargo.lock 0.4.5; code comments wrongly say "godot 0.5").
  - `sim-test`: a library (`perf`, `probe`, `scenes`, `worldmodel`), 3 binaries (`worldsim_probe`, `worldsim_wm`, `worldsim_perf`), and 153 integration test files in `tests/`. Depends on all four crates and serde_json.
  - Dependency direction: `sim-core ← sim-engine ← sim-systems ← sim-bridge`, with `sim-test` on top of all of them. `rayon` is declared in `[workspace.dependencies]` but no crate uses it.
- **Engine construction**: `pub fn build_game_engine(seed) -> SimEngine` at `rust/crates/sim-bridge/src/ffi/world_node.rs:204` is `build_world_with(&WorldGenConfig::game_v26(seed), WorldRules::game_v26(seed))`. `WorldSimNode::init` (`:233-242`, using `DEFAULT_GAME_SEED = 20_261_010`) and `reset_world` (`:382-388`) call it. `reset_world_stress` uses `game_v19`/`stress_v19`. `build_world_with` (`sim-systems/src/worldgen/mod.rs`, around line 257) calls `SimEngine::new(w, h, MaterialRegistry::new())` and then `register_default_runtime_systems`. The registry is therefore empty in the live game. The world is 64×64 (`WorldGenConfig::game_default`, worldgen/mod.rs:207-211).
- **System registration**: `register_default_runtime_systems` at `rust/crates/sim-systems/src/lib.rs:317` calls 16 `register_*_systems` helpers. `SimEngine::register_system` re-sorts by `priority()` after each push (stable `sort_by_key`, `sim-engine/src/lib.rs:926-929`).
- **System priority table** (lower runs first; interval = `current_tick % n == 0`):

  | prio | system | interval | file (sim-systems/src/runtime/…) |
  |---|---|---|---|
  | 90 | BuildingStampSystem | 1 | influence/building_stamp.rs |
  | 100 | InfluenceUpdateSystem | 1 | influence/update.rs |
  | 110 | AgentInfluenceSampleSystem | 1 | influence/agent_sample.rs |
  | 120 | AgentMovementSystem | 1 | agent/movement.rs |
  | 125 | AgentDecisionSystem | 1 | decision/agent_decision.rs |
  | 126 | StaleSeekTargetSystem | 1 | resource_regen/mod.rs (`STALE_SEEK_PRIORITY`) |
  | 130 | HungerDecaySystem | 1 | needs/hunger_decay.rs |
  | 131 | ThirstDecaySystem | 1 | needs/thirst_decay.rs |
  | 132 | SleepDecaySystem | 1 | needs/sleep_decay.rs |
  | 133 | ConstructionSystem | 1 | construction/construction_system.rs |
  | 134 | SocialInteractionSystem | 1 | social/social_interaction_system.rs |
  | 135 | SocialDecaySystem | 1 | needs/social_decay.rs |
  | 136 | MemorySystem | 1 | memory/memory_system.rs |
  | 137 | CombatSystem | 1 | combat/combat_system.rs |
  | 138 | SettlementSystem | 1 | settlement/settlement_system.rs |
  | 139 | StarvationSystem | 1 | survival/mod.rs |
  | 140 | ResourceRegenSystem | 120 (`REGEN_INTERVAL`) | resource_regen/mod.rs |
  | 141 | StockpileDepositSystem | 10 | settlement/stockpile_deposit.rs |
  | 142 | StockpileConsumeSystem | 5 | settlement/stockpile_consume.rs |
  | 143 | ShelterPlanningSystem | 1 | settlement/shelter_planning.rs |
  | 144 | MaturationSystem | 60 | agent/maturation.rs |
  | 144 | PairingSystem | 1 | agent/pairing.rs |
  | 145 | BedAssignmentSystem | 1 | agent/beds.rs |
  | 146 | AgingSystem | 60 (`AGING_INTERVAL`) | agent/aging.rs (`AGING_PRIORITY`) |
  | 147 | FireSystem | 60 | settlement/fire.rs |
  | 148 | FissionSystem | 1 | settlement/fission.rs (`FISSION_PRIORITY`) |
  | 149 | RelationsSystem | 1 | settlement/relations.rs |
  | 1000 | InfluenceVisualizationSystem | 6 | influence/visualization.rs |

  Two systems share priority 144. Their order comes from registration order: `register_rhythm_systems` registers Maturation before Pairing, and the sort is stable.
- **Tick loop**: Godot calls `WorldSimNode::process(delta)` (`world_node.rs:245-262`). It adds `delta * sim_speed` to an accumulator and runs `engine.tick()` while `accumulator >= FIXED_DT` (`FIXED_DT = 1.0/30.0`, `:176`), at most `MAX_ITERS_PER_FRAME = 5` times per frame (`:178`), then clamps the accumulator. `sim_speed` is clamped to [0, 4] (`clamp_sim_speed`, `:184`). In `SimEngine::tick_inner` (`sim-engine/src/lib.rs:980-1031`), the order is: set `resources.current_tick`, prune `recent_deaths`, run each due system in priority order, increment `current_tick`, recompute `time_of_day` from `ticks_per_day` (1440 by default, `lib.rs:710`, so 1 tick is 1 game minute) plus `rules.clock_offset_ticks`.
- **How a `#[func]` reaches a collector**: `#[godot_api] impl WorldSimNode` methods take `&self` or `&mut self`. They call a `pub fn collect_*` (pure Rust on `&hecs::World` or `&SimResources`), then a private `*_to_dict` marshaller that builds a `VarDictionary` of parallel `Packed*Array` columns. Example: `get_agent_family` → `collect_agent_family(&self.engine.world, bits)` → `agent_family_to_dict` (`world_node.rs:653-658`). Mutators follow the same shape: `on_building_placed` → `enqueue_building_placed(&mut resources, …)` (`:533-535`) and `apply_intervention` → `apply_intervention_command(&mut engine, …)` (`:445-447`, `:3291`). The collectors are re-exported from `sim_bridge::ffi` (`ffi/mod.rs:17-42`), and sim-test calls them headless.
- **How GDScript reads the bridge**: `scripts/ui/sim_reads.gd` (`class_name SimReads`, node `SimReads` beside node `WorldSim` of type `WorldSimNode` in `scenes/main.tscn:34-36`). Whitelists: `GETTERS` (15 snapshot getters), `TICK_GETTERS` (cached per sim tick), `MUTATORS` (`apply_intervention`, `advance_ticks`, `reset_world`, `reset_world_stress`, `on_building_placed`). Results are cached once per process frame (`_sync_frame`), and terrain is cached until the `world_reset` signal. Static entry points: `SimReads.read(world, getter)`, `SimReads.causal_events(...)`, `SimReads.mutate(world, method, args)`. Some per-entity reads bypass it (rows G03 and G04).
- **GDExtension**: `sim_bridge.gdextension` at the repo root, `entry_symbol = "gdext_rust_init"`, loads `res://rust/target/{debug,release}/libsim_bridge.{dylib,so}` / `sim_bridge.dll`. The only autoload is `Locale` (`project.godot` `[autoload]`). Godot feature tag is "4.6".

---

## Crates & boundaries

| id | claim | verdict | evidence | source |
|---|---|---|---|---|
| C01 | The workspace has six crates: sim-core, sim-data, sim-systems, sim-engine, sim-bridge, sim-test | PARTLY | `rust/Cargo.toml` members: sim-core, sim-engine, sim-systems, sim-test, sim-bridge (5). No sim-data | CLAUDE.md "Runtime Topology", "Directory Structure"; AGENTS.md "Crate Structure"; skill Part 2 "Crate Responsibilities" |
| C02 | A `sim-data` crate loads RON into typed structs (MaterialDef, RecipeDef, StructureDef, WorldRuleset, TemperamentRules) | FALSE | `ls rust/crates` has no sim-data. The RON loader is `sim-core/src/material/loader.rs:89` (materials only). RecipeDef/StructureDef/WorldRuleset/TemperamentRules: `grep -rn 'WorldRuleset' rust/crates/*/src` returns 0 hits | CLAUDE.md "Runtime Topology"; AGENTS.md; skill "sim-data/" |
| C03 | sim-core owns the ECS components | TRUE | `sim-core/src/components/` holds 36 files (agent, position, hunger, thirst, sleep, social, life, lineage, temperament, porter, relations, …) | CLAUDE.md "ECS Components"; skill "sim-core/" |
| C04 | Config constants live in `sim-core/src/config.rs` and the GDScript `GameConfig` mirrors them | FALSE | `ls rust/crates/sim-core/src/config.rs` says no such file. Constants are module-local `pub const` (e.g. `AGING_PRIORITY` aging.rs:20, `REGEN_INTERVAL` resource_regen/mod.rs:62). Tunables are fields of `WorldRules` (sim-core/src/rules.rs:9). No GameConfig script exists | CLAUDE.md "Config Constants", "Runtime Topology"; skill Rule 4 |
| C05 | sim-engine owns the tick loop and system scheduling | TRUE | `SimEngine::register_system` lib.rs:926, `tick_inner` lib.rs:980-1031 | CLAUDE.md "Runtime Topology"; AGENTS.md; skill "sim-engine/" |
| C06 | The Rust EventBus lives in `sim-engine/src/event_bus.rs` / `events.rs` | FALSE | Neither file exists. `grep -rn 'EventBus' rust/crates/*/src` returns 0 hits. sim-engine/src has only lib.rs and intervention.rs | CLAUDE.md "Runtime Topology", "EventBus (Rust side)" |
| C07 | sim-engine manages the World Rules lifecycle (Settings → Compile → Runtime, on_action, RuleHistory) | FALSE | `WorldRules` is a `Copy` struct in sim-core/src/rules.rs:9-18, picked once by `build_world_with(cfg, rules)` and stored in `SimResources.rules`. There is no compile stage, on_action hook or history | CLAUDE.md "World Rules (5 Slots)"; AGENTS.md; skill "sim-engine/" |
| C08 | Every simulation system is in sim-systems | TRUE | `grep -rc 'impl RuntimeSystem for' rust/crates/sim-systems/src` finds 28 impls, all under `src/runtime/` | CLAUDE.md "Runtime Topology" |
| C09 | sim-bridge contains no `impl RuntimeSystem` | TRUE | `grep -rn 'impl RuntimeSystem' rust/crates/sim-bridge/src` returns 0 hits | sim-bridge/src/lib.rs module doc ("Signal D") |
| C10 | Dependency direction: core ← engine ← systems ← bridge; sim-test on top | TRUE | sim-engine depends on sim-core; sim-systems on sim-core and sim-engine; sim-bridge on all three plus godot; sim-test on all four (`Cargo.toml` `[dependencies]`) | CLAUDE.md "Subdirectory CLAUDE.md" (crate graph, implied); skill crate list |
| C11 | Godot types appear only in sim-bridge | TRUE | `grep -rln 'use godot' rust/crates/{sim-core,sim-engine,sim-systems,sim-test}` returns 0 hits. `godot` appears only in sim-bridge/Cargo.toml | skill Rule 7; CLAUDE.md Common Mistakes #14 |
| C12 | sim-bridge is a GDExtension cdylib | TRUE | sim-bridge/Cargo.toml:8 `crate-type = ["cdylib", "rlib"]`; lib.rs:46 `#[gdextension] unsafe impl ExtensionLibrary` | CLAUDE.md "Tech Stack" |
| C13 | The extension descriptor is `rust/worldsim.gdextension` | FALSE | The file is `/sim_bridge.gdextension` (repo root), entry `gdext_rust_init` | CLAUDE.md "Autoloads" table row 5 |
| C14 | ECS is hecs | TRUE | workspace `hecs = "0.10"`, Cargo.lock hecs 0.10.5 | CLAUDE.md "Tech Stack"; AGENTS.md |
| C15 | rayon parallelises systems | FALSE | `rayon` is only in `[workspace.dependencies]`. No crate's `[dependencies]` lists it. `grep -rn 'par_iter\|rayon' rust/crates/*/src` returns 0 hits | CLAUDE.md "Agent Identity", "Tech Stack"; AGENTS.md "Project Overview" |
| C16 | sim-test is a headless test binary | PARTLY | It is a lib (`src/lib.rs`: perf, probe, scenes, worldmodel), 3 bins (`src/bin/worldsim_{probe,wm,perf}.rs`) and 153 test files in `tests/`. There is no `src/main.rs` | CLAUDE.md "Directory Structure"; skill "sim-test/" |
| C17 | Only sim-bridge needs Godot to test; the other crates `cargo test` without it | PARTLY | Core, engine and systems need no Godot. sim-bridge also builds and tests without Godot: sim-test calls its pure collectors (`grep -c 'sim_bridge' rust/crates/sim-test/tests/*.rs` totals 124). Only `WorldSimNode` and `#[func]` need the engine | skill Part 4 Lesson 9 |
| C18 | sim-core holds Influence Grid, Causal Log, Tile Grid, Temperament | TRUE | `sim-core/src/influence/`, `causal/`, `tile/grid.rs`, `components/temperament.rs` | AGENTS.md "Crate Structure"; skill "sim-core/" |
| C19 | sim-core holds Effect Primitives (`effect.rs`) and a Room system (`room.rs`) | FALSE | Neither file exists. `grep -rlw 'Room' rust/crates/*/src` returns 0 hits. `grep -rn 'AddStat\|MulStat' rust/crates/*/src` returns 0 hits | AGENTS.md; skill "sim-core/"; CLAUDE.md "Effect Primitive Standard" |
| C20 | sim-bridge owns the "Oracle text pipeline boundary (player input → LLM → response)" | FALSE | `grep -rni 'llm\|oracle' rust/crates/sim-bridge/src` returns 0 hits. No LLM dependency exists | AGENTS.md "Crate Structure"; skill "sim-bridge/"; CLAUDE.md "LLM Integration" |

## Engine & scheduling

| id | claim | verdict | evidence | source |
|---|---|---|---|---|
| E01 | A uniform `RuntimeSystem` trait: name, priority, tick_interval, tick | TRUE | sim-engine/src/lib.rs:229-248: `fn name(&self)->&str; fn priority(&self)->u32; fn tick_interval(&self)->u64; fn tick(&mut self, world:&mut World, resources:&mut SimResources)`. Stored as `Box<dyn RuntimeSystem + Send>` | skill "sim-systems/ RuntimeSystems"; CLAUDE.md rule 5 |
| E02 | Systems are `fn system_name(world: &mut hecs::World, resources: &Resources, events: &mut EventBus)` | FALSE | Systems are trait objects with `&mut self`. No `events` argument and no `Resources` type. `grep -rn 'events: &mut EventBus' rust/crates` returns 0 hits | CLAUDE.md "Coding Conventions → Rust" |
| E03 | Systems run in ascending priority order | TRUE | `self.systems.sort_by_key(\|s\| s.priority())` lib.rs:928. Test `test_priority_sort_lower_first` lib.rs:1175 | ARCHITECTURE.md principle 6; SYSTEMS.md intro |
| E04 | A system runs on ticks where `current_tick % tick_interval == 0` | TRUE | `self.current_tick.is_multiple_of(system.tick_interval())` lib.rs:999. Test `test_tick_interval_skip` lib.rs:1255 | SYSTEMS.md intro; trait doc lib.rs:242-244 |
| E05 | Production systems are registered in `register_default_runtime_systems` | TRUE | sim-systems/src/lib.rs:317-334. Called from `worldgen::build_world_with` and from a legacy bootstrap at sim-bridge world_node.rs:3119 | (task brief); CLAUDE.md rule 5 (wrong location, see E06) |
| E06 | The production system list is `DEFAULT_RUNTIME_SYSTEMS` in sim-bridge | FALSE | `grep -rn 'DEFAULT_RUNTIME_SYSTEMS' rust/crates` returns 0 hits | CLAUDE.md rule 5 "Production wiring check" |
| E07 | Behaviour order is `BEHAVIOR_ACTION_ORDER` in `cognition.rs` | FALSE | `grep -rn 'BEHAVIOR_ACTION_ORDER' rust/crates` returns 0 hits. No cognition.rs. Decision logic is `runtime/decision/agent_decision.rs` (3827 lines, AgentDecisionSystem prio 125) | CLAUDE.md rule 5 |
| E08 | 53+ RuntimeSystems are registered | FALSE | 28 `impl RuntimeSystem`, and all 28 are registered by `register_default_runtime_systems` | skill "sim-systems/" |
| E09 | Systems are tiered Hot/Warm/Cold by frequency | PARTLY | `UpdateTier {Hot, Warm, Cold}` exists at sim-core/src/influence/channel.rs:138 (per influence channel). The scheduler uses only `tick_interval`. `grep -rn 'UpdateTier' rust/crates/sim-systems/src` returns 0 hits. Intervals in use: 1, 5, 6, 10, 60, 120 | CLAUDE.md decision 2; AGENTS.md; skill; sim-engine lib.rs:22-25 doc |
| E10 | Phase-2 priority reservations: 90 BuildingStamp, 100 InfluenceUpdate, 110 AgentInfluenceSample, 1000 InfluenceVisualization | TRUE | building_stamp.rs:63, update.rs:224, agent_sample.rs:53, visualization.rs:60 | sim-engine trait doc lib.rs:235-239 |
| E11 | System list/priorities: ResourceRegen 5, JobAssignment 8, Needs 10 … StatsRecorder 90 | FALSE | That was the GDScript engine. Now ResourceRegenSystem is prio 140 with interval 120. No JobAssignment, Needs, Behavior, Gathering, Migration or StatsRecorder systems exist (see priority table) | ARCHITECTURE.md "시스템 등록 순서"; SYSTEMS.md table |
| E12 | Simulation runs at a fixed rate (20–30 TPS) with a Gaffer accumulator; render at native FPS | TRUE | `FIXED_DT = 1.0/30.0` world_node.rs:176. `process()` accumulator loop :245-262. Spiral guard `MAX_ITERS_PER_FRAME = 5` :178 | CLAUDE.md decision 9; AGENTS.md decision 9 |
| E13 | `main.gd _process(delta) → sim_engine.update(delta)` drives the tick | FALSE | No main.gd. `scenes/main.tscn` instances `WorldSimNode`, whose Rust `process()` drives ticks | ARCHITECTURE.md diagram, "데이터 흐름" |
| E14 | Speed control does not alter per-tick determinism | TRUE | `self.accumulator += delta * self.sim_speed` :255 (only the input is scaled). `clamp_sim_speed` → [0,4] :184. Called via `set_sim_speed` `#[func]` | CLAUDE.md FFI rule "Godot → Rust: … speed change" |
| E15 | 1 tick = 2 game hours, 12 ticks/day, 4,380 ticks/year | FALSE | `ticks_per_day: 1440` sim-engine/src/lib.rs:710, so 1 tick = 1 game minute. `time_of_day` is recomputed at end of tick lib.rs:1013-1023 | SYSTEMS.md "시간 체계" |
| E16 | Tick reference: 100 ticks ≈ 2.5 min, 4380 = 1 year, 8760 = 2 years | FALSE | Same evidence as E15: at 1440 ticks/day, 4380 ticks is about 3 game days | test_templates.md "Tick Count Reference" |
| E17 | `current_tick` is copied into `SimResources::current_tick` before any system runs | TRUE | lib.rs:984 `self.resources.current_tick = self.current_tick;` before the system loop | sim-engine lib.rs:18-20 doc |
| E18 | Double-buffer + damping + sigmoid saturation | PARTLY | `InfluenceGrid` is "8-channel double-buffered SoA" (influence/grid.rs:1,36) and InfluenceUpdateSystem swaps it (runtime/influence/mod.rs:5). `grep -rli 'sigmoid' rust/crates/*/src` returns 0 hits | CLAUDE.md decision 8; AGENTS.md; skill "sim-engine/" |
| E19 | Reactive ECS (ChangeTracker) | FALSE | `grep -rn 'ChangeTracker' rust/crates` returns 0 hits | CLAUDE.md decision 10; AGENTS.md |
| E20 | Systems never call each other; shared state is the only channel | TRUE | Systems communicate only through `&mut World` / `&mut SimResources` fields (queues such as `building_event_queue`, `combat_pairs`). The engine loop calls each `system.tick` in turn | ARCHITECTURE.md principle 3 (concept); CLAUDE.md EventBus contract (mechanism wrong) |
| E21 | Systems run serially on one thread with single-writer access | TRUE | `SimResources` doc lib.rs:262-266 ("trusting priorities to serialise access"); sequential `for` loop lib.rs:998 | CLAUDE.md "Agent Identity" (determinism) |
| E22 | Godot → Rust commands are spawn, speed change and similar | PARTLY | The mutators are `apply_intervention`, `advance_ticks`, `reset_world`, `reset_world_stress`, `on_building_placed`, `set_sim_speed` (sim_reads.gd MUTATORS plus camera_controller.gd:80). There is no spawn command | CLAUDE.md "The FFI Boundary Rule" |
| E23 | Player intervention is a faith-costed oracle with LLM interpretation | FALSE | God's hand = `enum Intervention` (GiftFood, SpoilFood, Rain, ColdSnap, …) in sim-engine/src/intervention.rs, applied by sim-systems/src/runtime/intervention.rs, with no RNG (intervention.rs:1-6). `grep -rliw 'faith\|oracle' rust/crates/*/src` returns 0 hits | CLAUDE.md "Oracle System"; AGENTS.md |
| E24 | Per-tick profiling never changes the simulation | TRUE | `tick_profiled` docs lib.rs:947-953. `Instant` is read only when a profile is given (lib.rs:977-979) | sim-engine doc (Pass 15) |

## ECS & data

| id | claim | verdict | evidence | source |
|---|---|---|---|---|
| D01 | All components derive serde `Serialize` + `Deserialize` | TRUE | In `sim-core/src/components/*.rs`, 50 `derive` lines sit above pub structs/enums. Every one includes Serialize (the 2 non-matching hits are comments) | CLAUDE.md decision 5; AGENTS.md |
| D02 | 18+ component types "Identity through Faith" | FALSE | `grep -rn 'struct Identity\|struct Faith' rust/crates` returns 0 hits. Real components include Agent, Position, AgentState, SeekTarget, Hunger, Thirst, Sleep, Social, Inventory, Memory, BodyHealth, Child, Life, Elder, Cold, Lineage, Hardiness, Temperament, ConstructionSite, PorterTrip, … | skill "sim-core/" |
| D03 | Behavior/Identity/Stress components (job string, name) | FALSE | No `struct Behavior` / `Identity` / `Stress` in rust/crates | test_templates.md "ECS Component Query", "Job Distribution" |
| D04 | Sparse social relations are a BTreeMap capped at 100 | PARTLY | `relationships: HashMap<RelationshipKey, RelationshipState>` sim-engine/src/lib.rs:401. No cap constant. Iteration is made deterministic by collect-and-sort (social_interaction_system.rs:221; pairing.rs:63) | CLAUDE.md decision 4; AGENTS.md |
| D05 | Influence Grid has 8–12 typed channels | TRUE | 8 channels: Warmth, Light, Noise, FoodAroma, Danger, Social, Spiritual, Beauty (influence/channel.rs:33-…). `COUNT = 8` :80 | CLAUDE.md decision 3 |
| D06 | LOD 4-tier `LodTier` component | PARTLY | `enum LodTier {Full, Medium, Simplified, Dormant}` influence/propagate.rs:32. It is not a component, and sim-systems never uses it (only sim-core influence code and 4 tests) | CLAUDE.md decision 1; AGENTS.md |
| D07 | Causal tracking: per-entity 32-event ring buffer | FALSE | Per-tile ring of 8: `TILE_CAUSAL_RING_SIZE = 8` causal/ring_buffer.rs:24 (ArrayVec); `CausalLogStorage::push(tile_idx, event)` storage.rs:45; `trace_parents` :142; held at `SimResources.causal_log` lib.rs:344. causal/mod.rs:3-5 documents the change from 32/entity | CLAUDE.md decision 7; AGENTS.md; skill "Causal Log (causal_log.rs)" |
| D08 | CausalEvent variants cover the recorded event kinds | TRUE | 15 variants: BuildingPlaced, StampDirty, InfluenceChanged, AgentDecision, ConstructionStarted/Completed, SocialInteractionStarted/Completed, MemoryRecalled, CombatStarted/Completed, AgentBorn, SettlementFormed, SettlementDissolved, AgentDied (causal/event.rs) | CLAUDE.md "Tech Stack: Events" (concept) |
| D09 | "All state changes recorded as events" (event sourcing through EventBus) | PARTLY | There is no bus. Systems mutate state directly. Records are kept in the causal log (per tile), the chronicle (`SimResources.chronicle` lib.rs:536) and per-feature logs (e.g. FissionRecord, TripRecord) | CLAUDE.md "Tech Stack"; ARCHITECTURE.md principle 2 |
| D10 | The chronicle is append-only and write-only for the sim; probes and the bridge read it | TRUE | sim-core/src/chronicle.rs:1-10 doc. Bridge readers `get_chronicle_since`, `get_person_chronicle`, `get_settlement_chronicle` (ffi/chronicle.rs) | sim-core/src/chronicle.rs (code doc) |
| D11 | Event names are past tense | PARTLY | CausalEvent mostly yes (BuildingPlaced, AgentDied). Exceptions: `StampDirty`, `AgentDecision`, `*Started` | CLAUDE.md "EventBus (Rust side)" |
| D12 | Building 2-layer: structural tile grid (walls/floors/roof/room_id) plus furniture ECS | PARTLY | `TileGrid {wall_material, floor_material, ground_resource …}` tile/grid.rs:15-25 has no roof or room_id. Buildings are SimResources BTreeMaps keyed by tile (`building_kinds`, `site_owner`, `building_owner` lib.rs:271-277). Construction sites are ECS entities (`world.spawn((ConstructionSite::new(…` shelter_planning.rs:103) | CLAUDE.md "Building System (2-layer)"; AGENTS.md |
| D13 | Room detection (dirty BFS flood fill, room roles) | FALSE | `grep -rlw 'Room\|room_id' rust/crates/*/src` returns 0 hits | CLAUDE.md "Building System"; skill |
| D14 | Tag+threshold recipes, never material IDs | FALSE | No recipe system. `grep -rli recipe rust/crates/*/src` hits only worldgen comments | CLAUDE.md decision 12; AGENTS.md; skill |
| D15 | MaterialDef has name, category, tags, properties | PARTLY | `MaterialDef {id, name, category, properties, tier, natural_in, mod_source}` material/definition.rs:16-23. No `tags` | skill "RON File Standards → Material" |
| D16 | Material RON files live in `sim-data/materials/*.ron` | FALSE | `git ls-files '*.ron'` lists 105 files, all under `rust/crates/sim-core/data/material/{animal,mineral,plant,stone,wood}/` | skill "RON File Standards"; CLAUDE.md "Material System" |
| D17 | Material formulas: damage = hardness×1.2, speed = 5/density, durability = hardness×density×10 | FALSE | derivation.rs:217-233 e.g. `axe_damage_cut = (shear_yield/10_000)*(1+hardness/20)`, `axe_speed = 5/(1+density/2_500)` | CLAUDE.md "Material System"; AGENTS.md "Key Formulas" |
| D18 | Materials drive the live game | FALSE | `build_world_with` uses `MaterialRegistry::new()` (empty, registry.rs:32) at worldgen/mod.rs:257. `grep -rn 'material::loader' rust/crates/{sim-systems,sim-bridge,sim-engine}/src` returns 0 hits | CLAUDE.md "Material System" (implied) |
| D19 | All content is RON ("Build like a mod"); new content = new .ron, zero .rs changes | FALSE | Material RON is the only data. All gameplay tuning is Rust `const fn` presets `WorldRules::game_v8 … game_v26` (33 `pub const fn game_v*` in sim-core/src/rules.rs; `game_v26` :1021) and `WorldGenConfig::game_v*` | CLAUDE.md decision 6; AGENTS.md "Critical Rules"; skill "Rules" |
| D20 | World Rules have 5 slots (resource/space/agent/society/global) | PARTLY | `WorldRules {agent: AgentRules, society: SocietyRules, clock_offset_ticks, seasons: Option<SeasonRule>}` rules.rs:9-18. Optional sub-rules default to "absent = legacy behaviour" | CLAUDE.md decision 14; AGENTS.md |
| D21 | Temperament is Cloninger TCI 4-axis NS/HA/RD/P | TRUE | `Temperament {ns, ha, rd, p: u16}` components/temperament.rs:30-38 (permille 0–1000) | CLAUDE.md decision 13; sim-core/CLAUDE.md Pass 16 |
| D22 | Temperament = σ(W_PRS·G38 + ε), internal f64, latent vs expressed, → HEXACO, archetype_label() | FALSE | Drawn from the lineage key on `STREAM_TEMPERAMENT` (rng.rs:88). Values are u16 with no gene vector. `grep -rn 'archetype_label\|latent' rust/crates/*/src` returns 0 hits. "hexaco" appears in one file (a doc reference) | CLAUDE.md "Temperament Pipeline"; AGENTS.md "Key Formulas"; skill |
| D23 | Temperament exists only under `AgentRules::temperament` (game_v20+) and is inserted where Hardiness is | TRUE | `Temperament::founder/child` and `TemperamentRule` in rules.rs; bridge `collect_agent_temperament` returns None when absent (world_node.rs:208-216) | sim-core/CLAUDE.md "Pass 16" |
| D24 | `Child {born_tick: u64}` is a marker that does no updates | TRUE | components/child.rs:6-8 | sim-core/CLAUDE.md "Pass 3" |
| D25 | `Life {born_tick: i64, lifespan_ticks: u64}` + zero-sized `Elder`; AgingSystem prio 146 does the work | TRUE | life.rs:6-10, :22; aging.rs:20 `AGING_PRIORITY = 146`, :23 interval 60 | sim-core/CLAUDE.md "Pass 4" |
| D26 | Zero-sized `Cold`, managed by FireSystem (prio 147, interval 60); hunger grows by `growth_rate × cold_hunger_permille / 1000` | TRUE | cold.rs:9; fire.rs:84,88; hunger_decay.rs:56-59 | sim-core/CLAUDE.md "Pass 5" |
| D27 | `Lineage {key: u64, births: u32}`; founders `Lineage::founder(seed, index)`, newborns `parent.child(seed)`; never derived from AgentId | TRUE | lineage.rs:15-19, `founder` :24, `child` :29; test `founders_depend_on_seed_and_index_only` :41 | sim-core/CLAUDE.md "Pass 9" |
| D28 | `PorterTrip {from, to, kind, amount, stage, depart_tick}`; `SettlementRelations` in SimResources; RelationsSystem prio 149 | TRUE | porter.rs:59-67; `SimResources.settlement_relations` lib.rs:541; relations.rs:166 | sim-core/CLAUDE.md "Pass 20" |
| D29 | Settlement stockpiles are `stockpile_stone` / `stockpile_wood` f64 fields; `resources.buildings`, `band_store`, `territory_grid`, `calendar`, `map` | FALSE | `Settlement.stockpile: BTreeMap<ResourceKind, u32>` settlement.rs:134. `grep -rn 'stockpile_stone\|band_store\|territory_grid\|pub calendar' rust/crates/*/src` returns 0 hits | test_templates.md (all templates) |
| D30 | IDs: AgentId/BuildingId/EventId are `u64`, SettlementId `u32`, issued by monotonic counters | TRUE | agent.rs:24, settlement.rs:27,35, causal/event.rs:210; `issue_agent_id` lib.rs:759, `issue_settlement_id` :769 | (fact for new doc) |
| D31 | Agent AI is Utility AI → GOAP (blueprints, construction AI) | FALSE | `grep -rli goap rust/crates/*/src` returns 0 hits. Decisions are AgentDecisionSystem (agent_decision.rs) plus routine modules (`decision/routine/{lifecycle,places}.rs`) | CLAUDE.md "Tech Stack: AI"; skill "sim-systems/" |
| D32 | Old GDScript data classes (EntityData, BuildingData, SettlementData, WorldData 256×256, ResourceMap) | FALSE | None exist. The world is 64×64 (`WorldGenConfig::game_default` worldgen/mod.rs:207-211; bridge `DEFAULT_W/H = 64` world_node.rs:66-68) | ARCHITECTURE.md file map; SYSTEMS.md "데이터 클래스" |

## Determinism & math

| id | claim | verdict | evidence | source |
|---|---|---|---|---|
| M01 | All simulation math is f64 | PARTLY | Mixed. `Hunger {value: f32, growth_rate: f32}` hunger.rs:22-24; `AgentRules.hunger_rate: f32` rules.rs:315; influence math f32 (propagate.rs:228-249). Token counts in sim-core/systems/engine: 54 `f32` vs 288 `f64`. Newer passes are integer (see M03) | CLAUDE.md "Coding Conventions → Rust"; AGENTS.md; skill Rule 1 |
| M02 | Forbidden float-math grep: no `.exp/.exp2/.ln/.log2/.log10/.powf/.powi/.sin/.cos/.tan/.tanh/.atan2/.hypot/.cbrt(` in sim-core, sim-systems, sim-engine | TRUE | The gate runs `! grep -rnE '\.(exp\|exp2\|ln\|…\|cbrt)\(' crates/sim-core/src crates/sim-systems/src crates/sim-engine/src`. Running it now returns 0 hits | tools/revitalize/gate_pass21.sh:39 (not stated in any V7 doc) |
| M03 | New mechanics use integer/permille math | TRUE | `grep -rn permille rust/crates/{sim-core,sim-systems}/src` gives 218 hits; `Temperament` u16 permille; `RelationState.value` −1000..=1000; stockpiles `u32`; resource tiles `u8` (lib.rs:370-396) | sim-core/CLAUDE.md Passes 5, 16, 20 |
| M04 | `.sqrt()` is outside the forbidden list and still used | TRUE (note) | Production `.sqrt()`: influence/propagate.rs:248 (f32). The other hit, temperament.rs:125, is test code | (gap in M02 list) |
| M05 | Keyed draws: `keyed_u64(seed, stream, subject, tick, slot, replicate)` = chained splitmix64, a pure function of its key; plus `keyed_unit_f64`, `below`/`keyed_below`, `keyed_step` | TRUE | rng.rs:143, :148, :154, :159, :164; module doc rng.rs:1-21 | sim-core/CLAUDE.md "Pass 9" |
| M06 | Stream constants are listed in `STREAMS` and must stay distinct | TRUE | `pub const STREAMS: [u64; 22]` rng.rs:106; test `streams_are_distinct` :194 | sim-core/CLAUDE.md "Pass 9" |
| M07 | Keyed draws apply only under `AgentRules::keyed_draws` (game_v15+); older scenes use sequential per-agent RNG | TRUE | `MovementRng` (splitmix64 component) agent/movement.rs:68; branch `let keyed = rules.keyed_draws` :223; `draw_lifespan(rng: &mut MovementRng, …)` aging.rs:34 | sim-core/CLAUDE.md "Pass 9" |
| M08 | Noise fork `NoiseFork {tick, replicate}` | TRUE | rng.rs:171-175; `SimEngine::apply_noise_fork` lib.rs:1041, free fn :1065; `noise_replicate_for` :740 | sim-core/CLAUDE.md "Pass 9" |
| M09 | Sorted collections (BTreeMap) or sorted iteration keep runs deterministic | PARTLY | 148 `BTreeMap<`/`BTreeSet<` uses, but 83 `HashMap`/`HashSet`/`AHashMap` lines remain: `food_tiles`, `relationships`, `settlements`, `combat_pairs` (lib.rs:370-442), `Settlement.member_agents: HashSet` settlement.rs:106. Order-sensitive paths sort first (79 `sort*` calls in sim-systems, e.g. social_interaction_system.rs:221) | CLAUDE.md decision 4; AGENTS.md "serde/BTreeMap" |
| M10 | No wall-clock or thread nondeterminism in sim logic | TRUE | `Instant` appears only in sim-systems/src/subprofile.rs (feature `subprofile`) and engine profiling. No rayon (C15) | CLAUDE.md "Agent Identity" (deterministic simulation) |
| M11 | Determinism is tested | TRUE | tests/harness_phase2_determinism.rs, tests/harness_a15_controlled_determinism.rs; `state_digest` sim-test/src/worldmodel/digest.rs:27; `fingerprint` probe/fingerprint.rs:33; `tools/v8/identity.sh` | CLAUDE.md "Agent Identity" |
| M12 | Interventions draw no random numbers | TRUE | sim-engine/src/intervention.rs:1-6 doc; applied between ticks | sim-engine intervention.rs (code doc) |
| M13 | Bootstrap need rates are f64 | FALSE (detail) | `BOOTSTRAP_HUNGER_RATE: f32` world_node.rs:91 next to f64 thirst/sleep rates | CLAUDE.md "All f64" |

## Bridge / FFI

| id | claim | verdict | evidence | source |
|---|---|---|---|---|
| B01 | The bridge is one Godot class `WorldSimNode` (a Node) | TRUE | `ffi/world_node.rs`; `[node name="WorldSim" type="WorldSimNode"]` scenes/main.tscn:34; exports listed in docs/v8/bridge_api_v7_final.txt | sim-bridge lib.rs doc |
| B02 | GDScript calls `SimBridge.get_*()` on a `SimBridge` autoload | FALSE | The only autoload is `Locale` (project.godot `[autoload]`). Scripts hold a node reference (`world_sim`, `_world_sim`, `_world`). `grep -rn 'SimBridge\.' scripts` hits only a comment (causal_panel.gd:5) | CLAUDE.md "FFI Boundary Rule", "Autoloads", "GDScript conventions" |
| B03 | Rust → Godot data is frame snapshots in PackedArrays | TRUE | Getters return `VarDictionary` of parallel packed columns. Agent snapshot: `ids` (hecs entity bits, PackedInt64), `xs`/`ys` PackedInt32, `states` PackedByte, `agent_ids`, `ages_days`, `colds` (world_node.rs:599-613, 1925-1950). Row order = hecs query order (:1799-1830) | CLAUDE.md "FFI Boundary Rule" |
| B04 | Rust events are relayed to GDScript and re-emitted as SimulationBus signals | FALSE | The bridge has one signal, `world_reset` (world_node.rs:273-274). GDScript pulls events: `get_chronicle_since(next, 512)`, `get_tile_causal_events`, `get_recent_deaths`. No SimulationBus | CLAUDE.md "EventBus", "SimulationBus" |
| B05 | Bridge Identity Contract: logic lives in pure-Rust `pub fn`s; `#[func]` bodies only forward or marshal | TRUE | `on_building_placed` → `enqueue_building_placed` (:533-535); `get_tile_causal_history`/`get_event_chain` → `collect_*`; `get_agent_family` → `collect_agent_family` (:653-658); `get_agent_temperament` → `collect_agent_temperament` (:208). `get_agent_snapshot` combines three collectors (:599-613) but adds no logic | sim-bridge lib.rs:23-35, ffi/mod.rs:3-14, world_node.rs:13-34 |
| B06 | Collectors are exported for headless tests | TRUE | `pub use` of `collect_agent_snapshot`, `collect_settlement_snapshot`, `collect_resource_snapshot`, … in ffi/mod.rs:23-42; used by sim-test (e.g. `sim_bridge::ffi::collect_agent_snapshot` in 6 test refs) | sim-bridge lib.rs doc |
| B07 | `build_game_engine(seed)` is the pure constructor of the live world | TRUE | world_node.rs:204-206 (game_v26), used by `init` :236 and `reset_world` :384 | (task brief); world_node.rs:196-203 doc |
| B08 | `sim-bridge/src/lib.rs` (~8200 lines) contains all FFI | FALSE | lib.rs is 47 lines. FFI is ffi/world_node.rs (4115), art_kernels.rs (1312), chronicle.rs (298), family.rs (221), activity.rs (169), relations.rs (152) | CLAUDE.md "Key Files" |
| B09 | WorldSimNode exposes 5 FFI methods | FALSE | 40-line export list in docs/v8/bridge_api_v7_final.txt (about 38 funcs). `grep -c '#\[func\]' ffi/world_node.rs` gives 55 (including tests and helpers) | sim-bridge lib.rs:8-17 doc (stale) |
| B10 | Agents are drawn with a MultiMesh fed by `Vec<f32> → PackedFloat32Array` | FALSE | agent_renderer.gd uses `Sprite2D` pools (agent_renderer.gd:39-40). The bridge exposes no MultiMesh buffer. MultiMesh appears only in zoom_lod_controller.gd / death_viz_renderer.gd | CLAUDE.md "Current State" (Phase 4-γ MultiMeshInstance2D); AGENTS.md; skill "sim-bridge/" |
| B11 | A `FrameSnapshot` type | FALSE | `grep -rn FrameSnapshot rust/crates` returns 0 hits | skill "sim-bridge/" |
| B12 | Influence grid is uploaded to Godot as data | PARTLY | `get_influence_overlay(channel) -> PackedByteArray`, read by world_renderer.gd:88 (no texture upload in Rust) | skill "sim-bridge/" |
| B13 | Some rendering math is in Rust behind the bridge | TRUE | `art_declump`, `art_walker_offsets` `#[func]` → `declump_offsets` / `walker_offsets` in ffi/art_kernels.rs (rendering only, f32 bits preserved) | (fact for new doc; contradicts "GDScript renders" only in emphasis) |
| B14 | Godot loads `rust/target/debug/libsim_bridge.dylib`, which `cargo test` may not rebuild (stale-dylib guard) | TRUE | sim_bridge.gdextension `macos.debug = "res://rust/target/debug/libsim_bridge.dylib"` | CLAUDE.md "Pipeline Stale-Dylib Guard" |
| B15 | Read FFI values with a `_safe_float()` helper | FALSE | `grep -rn '_safe_float' scripts` returns 0 hits | CLAUDE.md "Current State: FFI Safety" |
| B16 | SimBridge getter, system and UI are separate concerns (getter needed for UI to see data) | TRUE | UI can see only what a `#[func]` exposes. Example: Temperament reaches UI only through `get_agent_temperament` | skill Part 4 Lesson 8 |

## GDScript & UI

| id | claim | verdict | evidence | source |
|---|---|---|---|---|
| G01 | All simulation logic is Rust; GDScript is UI, rendering and input only | TRUE | No `scripts/systems/` or `scripts/ai/`. `scripts/` = core (locale.gd, settlement_selection.gd), rendering/, test/, ui/ | CLAUDE.md "Agent Identity", "Runtime Topology"; AGENTS.md |
| G02 | GDScript never reads entity state directly; it calls bridge getters | TRUE | All sim data reaches GDScript through `WorldSimNode` method calls. No other path exists (the ECS lives in Rust) | CLAUDE.md "The FFI Boundary Rule" |
| G03 | Snapshot reads funnel through `scripts/ui/sim_reads.gd` (SimReads) | PARTLY | SimReads whitelists 15 GETTERS, caches per frame and per tick, and invalidates on `world_reset` (sim_reads.gd:4-15, 103-140). Bypasses: world_renderer.gd:88,140,174,188 (`get_influence_overlay`, `get_agent_detail`, `get_tile_causal_history`); person_read_cache.gd:41-44; agent_inspector_panel.gd:392,597; chronicle_panel.gd:56 | (task brief) |
| G04 | GDScript never modifies simulation state except through bridge commands | TRUE | Mutations go through SimReads.MUTATORS (`apply_intervention`, `advance_ticks`, `reset_world`, `reset_world_stress`, `on_building_placed`) plus camera_controller.gd:80 `call("set_sim_speed", spd)` (direct, not via SimReads) | CLAUDE.md "FFI Boundary Rule", Common Mistakes #4 |
| G05 | Autoloads in order: GameConfig, SimulationBus, SimulationBusV2, Locale, SimBridge | FALSE | Only `Locale="*res://scripts/core/locale.gd"` | CLAUDE.md "Autoloads" |
| G06 | `scripts/core/simulation/` holds SimulationBus (UI relay) and GameConfig | FALSE | Directory missing. `grep -rn 'SimulationBus\|GameConfig' scripts` returns 0 hits | CLAUDE.md "Directory Structure", "SimulationBus (GDScript side)" |
| G07 | `class_name` at the top of every .gd file | FALSE | 7 of 52 tracked `scripts/ui/**/*.gd` start a line with `class_name` | CLAUDE.md "Coding Conventions → GDScript" |
| G08 | Typed GDScript (`var x: T`) | TRUE | scripts/ui: 1891 `var name:` declarations vs 6 untyped `var name =` | CLAUDE.md "Coding Conventions → GDScript" |
| G09 | Key UI files: hud.gd (~4200), entity_detail_panel_v3/v4.gd, entity_renderer.gd, minimap_panel.gd, scripts/ui/renderers/ | FALSE | All missing. Real UI: 52 scripts in scripts/ui/ plus panels/ (agent_inspector_panel, causal_panel, chronicle_panel, hud_topbar, hud_status_panel, god_hand_panel, art_ui, feed_food_ledger) | CLAUDE.md "Current State", "Key Files"; ARCHITECTURE.md; SYSTEMS.md "렌더러" |
| G10 | world_renderer.gd, camera_controller.gd, building_renderer.gd exist | TRUE | `scripts/ui/{world_renderer,camera_controller,building_renderer}.gd` | ARCHITECTURE.md file map; CLAUDE.md |
| G11 | Main scene is driven by `scenes/main/main.gd` | FALSE | `run/main_scene="res://scenes/main.tscn"` (project.godot:19). No main.gd; root `Main` Node2D holds WorldSim, SimReads and renderers (main.tscn:32-49) | ARCHITECTURE.md "scenes/" |
| G12 | Godot 4.6 | TRUE | project.godot:20 `config/features=PackedStringArray("4.6", "Forward Plus")` | CLAUDE.md "Tech Stack" |
| G13 | GDScript managers (EntityManager, BuildingManager, SettlementManager, SaveManager, Pathfinder, NameGenerator, EventLogger, GameCalendar, DeceasedRegistry) | FALSE | None exist in scripts/. No Rust pathfinding either (`grep -rli 'pathfind\|astar' rust/crates/*/src` returns 0 hits). No save/load | ARCHITECTURE.md; SYSTEMS.md "코어 매니저", "오토로드" |
| G14 | Headless GDScript harnesses exist for FFI/UI checks | TRUE | scripts/test/harness_visual_verify.gd, scripts/test/p3_gamma_2_alpha/harness_locale_panel.gd, … | CLAUDE.md "When NOT to use sim-test" |
| G15 | The strict check exists (parse errors, warnings as errors, FFI binding diff) | TRUE | tools/harness/gdscript_strict_check.sh present; run as step `strict_check` in gate_pass21.sh:42 | CLAUDE.md "Pipeline GDScript Strict Check" |

## Localization

| id | claim | verdict | evidence | source |
|---|---|---|---|---|
| L01 | All player-facing text goes through the custom `Locale` autoload, never `tr()` / TranslationServer | TRUE | `grep -rnE '(^\|[^a-zA-Z_.])tr\(' scripts` returns 0 hits; Locale is the only autoload | CLAUDE.md "Localization Rules"; skill Part 1 |
| L02 | Call it as `Locale.ltr("KEY")` | PARTLY | The API is `ltr`, but UI code resolves `get_node_or_null("/root/Locale")` and calls `.ltr()` / `.call("ltr", …)` (11 ui files, e.g. building_renderer.gd:52,266; chronicle_text.gd:12). A bare `Locale.` global fails the `--check-only` gate (hud_status_panel.gd:44-47) | CLAUDE.md; skill Part 1 |
| L03 | Locale API: ltr, trf1, trf2, trf3, tr_id, get_month_name, set_locale | FALSE | locale.gd public funcs: `ltr` :80, `set_language` :83, `key_count` :88, signal `language_changed` | skill "Locale API Reference" |
| L04 | Every key exists in both en and ko | TRUE | compiled en.json and ko.json each hold 5287 keys; the symmetric difference is 0 | CLAUDE.md; skill Rule 4 |
| L05 | Locale loads `localization/compiled/{lang}.json` (from tools/localization_compile.py), falls back to per-category dirs, and takes the default language from manifest | TRUE | locale.gd:5-15, 20-33; manifest `"default_locale": "ko"`; tools/localization_compile.py exists | locale.gd header (code doc); CLAUDE.md harness notes |
| L06 | Key prefix → file convention (UI_*→ui.json, EVENT_*→events.json, …) | PARTLY | Files exist (localization/en/{ui,events,deaths,emotions,buildings,…}.json); per-key placement not audited | skill Rule 5 |
| L07 | Rust produces no user-visible text; GDScript resolves keys | TRUE | Bridge returns ids and kind codes (e.g. activity kinds 0..6, porter kind 0/1, world_node.rs:706-716); GDScript maps them to keys (chronicle_text.gd:12, settlement_labels.gd:124) | skill Part 1 "Core Premise" |
| L08 | Hardcoded-string scans come back clean | TRUE | skill scan 1 (`\.text\s*=\s*"[A-Z]`) returns 0 hits. The `.contains(` scan returns 1 hit, settlement_labels.gd:125 `format.contains("%d")`, which checks a format string, not English text | skill "Localization Verification" |
| L09 | Localization sources include `localization/fluent/`, `key_registry.json`, `compiled/` | TRUE | `ls localization`: compiled, en, fluent, key_owners.json, key_registry.json, ko, manifest.json | CLAUDE.md "Requires pipeline" list |

## Testing conventions

| id | claim | verdict | evidence | source |
|---|---|---|---|---|
| T01 | Harness tests go in `rust/crates/sim-test/src/main.rs` inside `#[cfg(test)] mod tests` | FALSE | No main.rs. Tests are integration files `rust/crates/sim-test/tests/*.rs` (151 `harness_*.rs` + 2 `pass17_*`) | CLAUDE.md "HDD RED"; test_templates.md header |
| T02 | Test names are `harness_<category>_<assertion>` with categories job/resource/building/band/territory/population/economy | PARTLY | 1628 of 1686 `#[test]` fns start with `harness_`. Files and fns are named by phase or pass (`harness_p7_beta_social_system.rs`, `harness_revitalize_pass20_relations.rs`), not by those categories | CLAUDE.md "Naming convention" |
| T03 | Standard setup `make_stage1_engine(42, 20)` + `engine.run_ticks(N)`; helpers `collect_positions` | FALSE | `grep -rn 'make_stage1_engine\|fn run_ticks' rust/crates` returns 0 hits. Fixtures are in sim-test/src/scenes.rs (`legacy_scene()`, `game_scene(seed)`, `game_vN_scene(seed)`, `…_with(seed, &variety)`); ticks run as an `engine.tick()` loop or `sim_bridge::ffi::world_node::advance_engine_ticks` (:3129) | CLAUDE.md "Harness test patterns"; test_templates.md |
| T04 | Access via `engine.world()` / `engine.resources()` / `resources_mut()` | FALSE | No such methods (`grep -n 'pub fn world(\|pub fn resources(' sim-engine/src/lib.rs` returns 0 hits). `SimEngine.world` and `.resources` are pub fields (lib.rs:901-908) | CLAUDE.md "Harness test patterns"; test_templates.md |
| T05 | Seed 42, 20 agents, 256×256 map is the standard baseline | FALSE | Live default `DEFAULT_GAME_SEED = 20_261_010` (worldgen/mod.rs:8); 64×64 maps | test_templates.md "Standard Setup"; evaluation_criteria.md "Type C" |
| T06 | `#[cfg(test)]` unit tests in every module | PARTLY | 98 of 122 `.rs` files in sim-core/engine/systems/bridge src contain `cfg(test)` | CLAUDE.md Rust conventions; skill Rule 6 |
| T07 | Gate: `cargo test --workspace && cargo clippy --workspace -- -D warnings` | TRUE | Same commands in tools/revitalize/gate_pass21.sh:36-38, which also runs `--all-targets` clippy, forbidden_math, sim-bridge build, Godot import, strict check, smoke | CLAUDE.md "Post-work"; AGENTS.md "Gate Command" |
| T08 | `cargo test` runs without Godot | TRUE | No Godot dependency outside sim-bridge. sim-bridge's godot crate builds without an engine, and the collectors are tested from sim-test | skill Lesson 9 |
| T09 | Diagnostic `println!` only inside tests, never in game code | TRUE (spot) | Not exhaustively audited. No `println!` hit outside cfg(test) blocks was found while reading engine/bridge | CLAUDE.md "RETRY on failure"; test_templates.md |
| T10 | Pre-commit harness hook and installer exist | TRUE | hooks/pre-commit-harness, tools/harness/install_hooks.sh, tools/harness/harness_pipeline.sh, tools/harness/authorize_env_bypass.sh, .harness/baseline/known_failures.txt | CLAUDE.md "Session Startup", rule 7.1; harness SKILL.md |

## Layout & naming

| id | claim | verdict | evidence | source |
|---|---|---|---|---|
| N01 | Tree: `rust/crates/{sim-core,sim-data,sim-systems,sim-engine,sim-bridge,sim-test}`, `scripts/{core,systems,ai,ui}`, `data/`, `localization/`, `tools/gate.sh` | PARTLY | rust/crates: no sim-data. scripts = {core, rendering, test, ui}. `data/` and `tools/gate.sh` are missing. `localization/` and `tools/` exist | CLAUDE.md "Directory Structure" |
| N02 | Subdirectory CLAUDE.md files for rust/, sim-core, sim-data, sim-systems, sim-engine, sim-bridge, scripts/core, scripts/ui, data | FALSE | `git ls-files '*CLAUDE.md'` lists only `CLAUDE.md` and `rust/crates/sim-core/CLAUDE.md` | CLAUDE.md "Subdirectory CLAUDE.md Files"; header note |
| N03 | sim-core files: influence_grid.rs, effect.rs, causal_log.rs, tile_grid.rs, room.rs, temperament.rs, config.rs | FALSE | Actual: influence/{grid,channel,blocking,propagate}.rs, causal/{event,ring_buffer,storage}.rs, tile/grid.rs, components/temperament.rs; no effect/room/config | skill "sim-core/" |
| N04 | sim-systems modules: `runtime/<domain>/…` | TRUE | runtime/mod.rs:13-25: agent, combat, construction, decision, influence, intervention, memory, needs, resource_regen, settlement, social, survival; plus `chronicle_log.rs`, `worldgen/`, `subprofile.rs` | skill Lesson 7 (example `runtime/psychology.rs` does not exist) |
| N05 | Rust naming: snake_case modules, PascalCase types, `*System` suffix for systems | TRUE | All 28 systems are `XxxSystem` structs; modules snake_case | CLAUDE.md "Coding Conventions → Rust" |
| N06 | No `unwrap()` in production Rust | PARTLY | 0 `.unwrap()` before the first `#[cfg(test)]` in core/engine/systems/bridge, but 6 production `.expect(` calls: components/memory.rs:232, worldgen/terrain.rs:119,123, worldgen/camp.rs:80,96,143 | CLAUDE.md conventions, Common Mistakes #7; AGENTS.md "Unwrap Audit"; skill Rule 2 |
| N07 | `///` doc comments on all pub items | TRUE | `#![warn(missing_docs)]` in sim-core/engine/systems/bridge lib.rs (+ clippy `-D warnings` gate) | CLAUDE.md conventions; skill Rule 5 |
| N08 | Enums over strings in hot paths | TRUE | `AgentState`, `TargetKind`, `ResourceKind`, `BuildingKind`, `ChronicleKind` enums. Strings are parsed only at the FFI boundary (`apply_intervention(kind: GString…)` → `parse_intervention`, world_node.rs:3291+) | CLAUDE.md Common Mistakes #15; skill Rule 3 |
| N09 | `unsafe` is forbidden in sim crates | TRUE | `#![forbid(unsafe_code)]` in sim-core, sim-engine and sim-systems lib.rs. sim-bridge uses `deny(unsafe_op_in_unsafe_fn)` with only the `unsafe impl ExtensionLibrary` | sim-bridge lib.rs:1-4 (code doc) |
| N10 | Work happens on `lead/main` (AGENTS: `git reset --hard origin/lead/main`) | FALSE (now) | Local branches: main, v8/wip, codex/revitalize-2026-10-wip. `origin/HEAD -> origin/main`; `remotes/origin/lead/main` still exists | CLAUDE.md "Repository", "Pre-flight"; AGENTS.md "Pre-flight" |
| N11 | GDScript classes PascalCase, variables/functions snake_case | TRUE (spot) | e.g. `class_name SimReads`, `func read_raw`, `var _tick_cache` (sim_reads.gd) | CLAUDE.md "Coding Conventions → GDScript" |
| N12 | Content scale and performance budget tables (Phase 1: ~20 materials, ~1K agents …) | FALSE (stale) | 105 materials loaded only by tests. The live world is 64×64 (B-facts). No furniture, tech or recipes. These are V3.1 targets, not the current state | CLAUDE.md "Content Scale", "Performance Budget" |
| N13 | The V7 phase plan (A-1…A-10 prerequisites) is current | FALSE | Root CLAUDE.md says V7 closed 2026-05-17; work since then is "Revitalize Pass N" (sim-core/CLAUDE.md) and art passes (git log `art-p13b`) | AGENTS.md "Current Phase" |

---

## Bulk-FALSE: `docs/architecture/ARCHITECTURE.md` and `SYSTEMS.md` (GDScript-era)

These are not separate rows. All of the following are absent from the tree (checked with `ls`/grep): `scenes/main/main.gd`; `scripts/core/{simulation_engine,simulation_system,simulation_bus,game_config,event_logger,world_data,world_generator,resource_map,entity_data,entity_manager,building_data,building_manager,settlement_data,settlement_manager,pathfinder,name_generator,save_manager,chunk_index,relationship_manager,game_calendar,deceased_registry}.gd`; `scripts/ai/behavior_system.gd`; every `scripts/systems/*.gd` and `scripts/systems/phase4/*.gd`; `scripts/ui/{entity_renderer,hud,minimap_panel,stats_panel,stats_detail_panel,entity_detail_panel,building_detail_panel}.gd`; `data/naming_cultures/*.json`; the SimulationBus signal list; the `simulation_event` event catalogue; QUIET_EVENTS. Only the generic principles carry over: E03, E04, E20 and the separation of simulation from rendering. The "RefCounted data classes" and "GameConfig centralisation" principles are FALSE.

## Counts

TRUE 74 (including 1 "note" and 2 "spot") · FALSE 55 (including 1 "now", 1 "stale" and 1 "detail") · PARTLY 23. Total 152 rows.
