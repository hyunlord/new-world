---
name: worldsim-architecture
description: Use before changing any Rust in rust/crates/ or adding a bridge read — crate boundaries and dependency direction, ECS and determinism rules (integer/permille math, sorted iteration, keyed draws), the "add a system" and "add a bridge read" procedures, and module header and file size rules.
---

# WorldSim architecture

WorldSim is a deterministic colony simulation in Rust (hecs ECS) with a Godot
4.6 front end that only reads it. This skill is the contract for Rust and
bridge changes. The generated map of systems lives in `docs/ARCHITECTURE.md`
(system table from `tools/v8/system_table.py`); testing and identity live in
`worldsim-testing`; Godot/gdext specifics in `worldsim-godot`.

## Crate map (dependency direction: core ← engine ← systems ← bridge; sim-test on top)

| crate | owns | depends on |
|---|---|---|
| `rust/crates/sim-core` | ECS components (`src/components/`), `WorldRules` (`src/rules/`: `mod.rs` = `WorldRules`, one file per sub-rule struct, `baseline.rs` = `WorldRules::baseline(seed)`), keyed RNG (`src/rng.rs`), causal ring (`src/causal/`), chronicle (`src/chronicle.rs`), influence grid (`src/influence/`), tile grid (`src/tile/`), materials (`src/material/`), world variety draws (`src/variety.rs`), the M1 colony model (`src/colony/`: items and `ItemLedger`, trees, designations, zones, blueprints, reservations, `Command`, `Job`) | hecs, serde, ron, ahash, glam, … |
| `rust/crates/sim-engine` | `RuntimeSystem` trait, `SimResources` (all non-ECS world state), `SimEngine` (system list, tick loop, clock) | sim-core |
| `rust/crates/sim-systems` | every `impl RuntimeSystem` under `src/runtime/<domain>/`, `register_default_runtime_systems` (`src/lib.rs`), commands (`src/commands.rs`: `apply_command`, `submit`), the job layer (`src/runtime/jobs/`), worldgen (`src/worldgen/`, `WorldGenConfig::baseline(seed)`, `WorldGenConfig::colony(seed)`, `build_world_with`) | sim-core, sim-engine |
| `rust/crates/sim-bridge` | GDExtension cdylib: one Godot class `WorldSimNode` under `src/ffi/`, pure `collect_*` functions behind thin `#[func]` methods, `build_game_engine(seed)` | the three above + `godot` |
| `rust/crates/sim-test` | `sim_test::baseline_engine(seed)` (the frozen identity world), `sim_test::colony_engine(seed)` (the game's world), scenes, probe (`worldsim_probe`), perf (`worldsim_perf`), domain integration tests in `tests/` | all four |

The tick: `WorldSimNode::process(delta)` runs `engine.tick()` on a fixed 1/12 s
accumulator at 1× (`TICKS_PER_SECOND_AT_1X`, at most 8 ticks per frame, M1). `SimEngine::tick` sets
`resources.current_tick`, runs every system whose `tick_interval` divides the
tick in ascending `priority` (stable sort, so equal priorities keep
registration order), then advances the clock. 1,440 ticks = 1 game day
(1 tick = 1 game minute).

## Rules

### Boundaries
- **Do** keep the dependency direction core ← engine ← systems ← bridge, sim-test on top. *Reason:* a reverse edge makes the sim depend on Godot or on tests, and breaks headless `cargo test`.
- **Don't** use `godot` types outside `rust/crates/sim-bridge`. *Reason:* sim-core/engine/systems must build and test without an engine; today `use godot` appears only in sim-bridge.
- **Don't** put an `impl RuntimeSystem` in sim-bridge or simulation logic in a `#[func]` body. *Reason:* `#[func]` methods only forward to a pure Rust function and marshal; logic behind `#[func]` cannot be tested headless.
- **Do** let systems talk only through `World` components and `SimResources` fields (queues such as `building_event_queue`). *Reason:* there is no event bus; systems never call each other, and priorities serialise access on one thread.
- **Don't** let GDScript write simulation state; mutations go through the few bridge commands (`SimReads.MUTATORS`: the M1 orders `cmd_*`, `advance_ticks`, `reset_world`, `reset_world_stress`, `on_building_placed`; plus `set_sim_speed`). *Reason:* GDScript has no access to the ECS and must never become a second simulation.
- **Do** put tunables in `WorldRules` sub-rule fields (baseline value in `rules/baseline.rs`) and fixed constants as module-local `pub const`. *Reason:* there is no `config.rs`; the baseline is the one live rule set, and a field is visible in the rules dump test.
- **Don't** let a system read measurement-only state (`worldgen_report`, probe-only logs). *Reason:* measurement must not change behaviour or identity.

### Determinism
- **Do** use integers and permille (‰, 0..=1000) for new mechanics in sim-core, sim-systems and sim-engine. *Reason:* integer math is bit-reproducible across machines and builds; the rule is "no transcendentals", existing f32/f64 needs stay.
- **Don't** call float transcendentals in those three crates; gate step 7 (`forbidden_math` in `tools/v8/gate.sh`) greps `src/` for `.exp( .exp2( .ln( .log2( .log10( .powf( .powi( .sin( .cos( .tan( .tanh( .atan2( .hypot( .cbrt(` and must find 0 hits. *Reason:* their results are not guaranteed bit-identical; use an integer table (as `EXP_TABLE` in `fission_threshold.rs`) instead. The grep misses `.log(`, `ln_1p`, `exp_m1`, `asin`, `acos`, `atan`, `sinh`, `cosh` — do not use those either.
- **Do** iterate maps and sets in key order: `BTreeMap`/`BTreeSet` for every new collection, or collect and sort before an order-sensitive loop. *Reason:* std `HashMap`/`HashSet` order differs between processes and runs; some old fields (`relationships`, `settlements`, `Settlement.member_agents`) are still hash-based and must only be iterated sorted.
- **Don't** insert or remove components (structural hecs changes) or despawn while iterating a hash set or a live query. *Reason:* archetype order is insertion order, so hash order would permute every later query (c03ae2bc). Collect `(AgentId, Entity)` pairs, sort by `AgentId`, then apply.
- **Do** process agents in `AgentId` order and settlements in `SettlementId` order wherever the outcome depends on order (claims, deaths, completions); break distance ties by `(distance, id)`. *Reason:* first-come effects otherwise follow hecs or hash order.
- **Do** draw randomness with keyed draws from `rust/crates/sim-core/src/rng.rs` (`keyed_u64`, `keyed_below`, `keyed_step`, `keyed_unit_f64` on `(world_seed, stream, subject, tick, slot, replicate)`). *Reason:* a keyed draw is a pure function of its key and consumes no stream state, so skipping or adding one shifts nothing else.
- **Do** give each new purpose its own `STREAM_*` constant, add it to `STREAMS`, and keep `streams_are_distinct` green; grep every `0x…_…` 64-bit constant before picking a salt. *Reason:* reusing a stream or salt silently correlates two decisions.
- **Do** key a draw by a stable identity (lineage key), never by `AgentId`, and put an event index in `slot` when one subject can draw several times per tick. *Reason:* `AgentId` shifts with any earlier spawn, and two events with the same key draw the same value.
- **Don't** change a draw formula (for example `% n` → multiply-shift) as a refactor. *Reason:* same distribution, different values: it is a behaviour change and fails identity.
- **Don't** use wall-clock time, threads or `rayon` in simulation code. *Reason:* the sim runs serially by design; `Instant` is read only behind the optional profile.

### Code
- **Don't** add `.unwrap()` (or new `.expect(`) to production Rust you touch; return `Result`, use `unwrap_or`/`unwrap_or_default`, or handle the `None`. *Reason:* a panic in a tick kills the game; tests may unwrap.
- **Do** write a `///` doc comment on every `pub` item. *Reason:* `#![warn(missing_docs)]` + clippy `-D warnings` fail the gate otherwise.
- **Do** use enums, not strings, in hot paths; parse strings only at the FFI boundary. *Reason:* string matching is slow and unchecked.
- **Do** add `#[serde(default)]` to every new rule field. *Reason:* older serialized rules must still deserialize.
- **Do** list every exhaustive `match` and every counter/tally consumer (with file:line) before adding an enum variant, and pin its encoding with a test. *Reason:* new variants touch `as_u8`, despawn, probe census, FFI tallies and GDScript mappings.
- **Do** run `cargo clippy --workspace --all-targets -- -D warnings` (not only scoped clippy); fix lints, never suppress them. *Reason:* `--all-targets` findings slipped into commits when only scoped clippy ran.

### Colony world (M1; ADR 0012–0014)
- **Do** gate every colony behaviour on `resources.rules.colony` (`ColonyRule`, set by `WorldRules::colony(seed)`; `None` in baseline) and return at once when it is `None`. *Reason:* `baseline` is frozen and proven by identity; a colony change that runs in baseline fails the gate (ADR 0014).
- **Do** change colony state from the player only through a `Command` applied by `commands::apply_command` (via `submit`, which logs it with the next tick in `colony.command_log`). *Reason:* the log is the replay (ADR 0012); a write that bypasses it cannot be replayed and breaks the replay gate step.
- **Do** keep every created log in exactly one place (ground `ItemStack`, `Carry`, a site's delivered count, consumed) and move it with the helpers in `runtime/jobs/items.rs` and `cancel.rs` (`put_items`, `take_items`, `drop_carry`), updating `colony.item_ledger` when logs are created or consumed. *Reason:* `jobs::invariants::check` asserts created = ground + carried + delivered + consumed every tick in debug builds; a hand-rolled move is a ledger hole.
- **Do** release reservations through `jobs::cancel::{cancel, finish, release_job, on_death}`, never by removing a `Job` component directly. *Reason:* a removed job with live `colony.reservations` blocks that tree, stack, cell or site forever (a reservation leak).
- **Don't** add job logic to `decision/system.rs`. Only the seam calls go there (`jobs::holders` ~L833, `hold_for_decision` ~L1127, `settle_after_decision` ~L2661) plus compile-forced `TargetKind::Job` arms. *Reason:* ADR 0013 caps the decision change (+26 −1 lines in M1); the job layer must stay readable in `runtime/jobs/`.

### Paths and built space (M3; ADR 0021–0023)
- **Do** decide passability only in `runtime::nav::tile_cost` and call `runtime::nav::refresh(res, tile)` from the code that changes a tile (terrain, wall, door, dug flag, a space site placed, cancelled or finished). *Reason:* the cost map is incremental; a change made without `refresh` leaves a stale map, while a full rebuild hides the miss (the tests compare a rebuilt map with the incremental one).
- **Do** treat regions and the path memo as belonging to one `nav.version` and never save them; save only the paths. *Reason:* a memo hit must return exactly what a search would, which holds only while the map is the one it was searched on.
- **Do** offer a job only for a target reachable from the worker's region (`NavState::reaches`), and turn a target that becomes unreachable into `CancelReason::Unreachable` with the one-day cooldown. *Reason:* an unreachable assignment walks into a wall forever and holds its reservation.
- **Do** make a decision's goal a tile the agent can stand on (a decision consumes on its tile): out of reach, move it to the nearest reachable tile of its kind or end the walk. *Reason:* a water tile on undug hill killed its drinkers in Part 1 (M3 D10).
- **Do** check every world for closed basins when passability changes: the colony camp must lie in an open region (`worldgen/colony.rs::open_the_camp`). *Reason:* seed 403 sealed ten founders between hills and the river (M3 D17).

### Knowledge (M4; ADR 0024–0026)
- **Do** read gating only through the cached practitioner bitset (`Knowledge::practice`, a `u32`) against `knowledge::gates()` masks; never scan entries in a hot path. *Reason:* the bitset is what keeps `JobAssignSystem` within its +40 µs budget at N = 1,000; it is rewritten by every write to the component, so it cannot go stale if all writes go through `Knowledge`'s methods.
- **Do** ask the gate in every path that advances gated work, not only the job layer: the job assignment, the bill runner and the V7 decision system's construction arm (whose site snapshot leaves gated colony sites out, M4 D22). *Reason:* F10 found doors finished by passers-by after the only door maker died; the job layer's gate was correct, the second builder never asked.
- **Do** key every knowledge draw on `STREAM_KNOWLEDGE` with the lineage key as subject and the node id or a pair index in `slot`, and run the channels as daily or interval systems (oral at 20:30, observation every 30 ticks, decay every 60, discovery and the aggregate every 240). *Reason:* C4; a keyed draw per node lets one roll be skipped without moving another, and per-tick diffusion would cost every tick.
- **Don't** let a channel make anything but a fresh entry, a gain or a decay through `Knowledge`'s methods; the aggregate reads, never writes, people. *Reason:* the at-risk and lost states are computed from holders; a side write would desync the bitset and the HUD.

### Story, animals and mood (M5; ADR 0027–0030)
- **Do** plan pressure once a season with keyed draws (`STREAM_STORY`, subject = the season index) and begin/end it in the daily `StorytellerSystem`; let each effect live in the system whose behaviour it changes, asking `story::active(res, kind)`, and book its cost with `story::add_cost` / `note_cause`. *Reason:* C4 cadence (a daily system, never a per-tick poll); one place decides *when*, the behaviour systems decide *what*, so a pressure kind adds no new loop over people.
- **Do** keep mercy and gap rules (no drought or beasts before day 12, majors ≥ 6 days apart …) in the planner and test them on the calibration and held-out seeds with `story::simulate_schedule` (`colony_story.rs`). *Reason:* F1 checks them on every evaluation world; a schedule bug shows on one seed in thirty.
- **Do** keep animals in `ColonyState::wildlife` (≤ 16, not ECS entities) and route every person–animal blow through `colony.fights`, resolved by `CombatSystem`; V7's `combat_pairs` stay person–person. *Reason:* the V7 pair model and its chronicle/despawn paths stay single; adding ECS animals would put them in every `Agent`-less query and the save registry.
- **Don't** reserve anything for a person without a `Job` (a drafted person's weapon, a dressing target): the reservations invariant allows reservations only to job holders. *Reason:* the draft's arming reservation failed F10 on calibration seed 5; a drafted person takes the weapon if it is still there.
- **Do** have other systems note mood facts through `mood::note` (teaching, a gated craft, a discovery, a quarrel, a kin's death) and let only `MoodSystem` move the value. *Reason:* one writer keeps the target and the "why?" modifiers consistent with what the inspector shows.
- **Do** make an effect act on the quantity its bar measures: a worn top spares cold *hours* at its ‰ (a keyed roll an hour), so cold events, the mood's cold night and exposure damage all scale; scaling only the damage left cold events equal by construction. *Reason:* M5 D15; F5 counts events, and a hidden factor on another quantity cannot move it.

### The simulation contract (ADR 0016; M1.5)
- **Do** keep the world state a pure function of `(rules, seed, command log)` (C1): no wall clock, no hash order, no state outside `World`, `SimResources` and the systems' own fields; a saved game must restore every one of them. *Reason:* replay, colony determinism, identity and the save round trip are the gate steps that prove it.
- **Don't** let the sim crates know pixels, tile sizes, sprites, cameras or Godot (C2); every value a renderer needs is a documented snapshot read (`docs/v8/snapshot_schema.md`, checked by `check_docs_helpers.py schema`). *Reason:* the text renderer (`worldsim_probe --ascii`) is the standing second renderer and must agree with the Godot view (`renderer_agreement`).
- **Do** use the spatial index (`sim_core::spatial::ChunkIndex`, `SimResources.spatial`) for every nearest-candidate search over things on the map; never scan every candidate per agent per tick or every tile per tick (C3). *Reason:* O(agents × candidates) work is what the standing benchmark (`worldsim_perf bench`, `docs/perf/PERF_LEDGER.md`) catches; `nearest_where` reproduces the linear `(distance, y, x)` choice exactly.
- **Do** keep derived indexes current with the change counters that already exist (`ColonyState::overlay_version`, `trees_version`) rather than an update at every mutation site. *Reason:* one counter cannot miss a mutation; M1.5 rebuilds the colony indexes when the counter moves (DECISIONS D11).
- **Do** store closed-form state (`since_tick` / `due_tick` + rate, evaluated on read, as `Life::age_ticks` and `Pregnant::days_left`) instead of a value incremented every tick, and run daily work as a daily system (`tick_interval` + `tick_offset`), not as a per-tick poll (C4). *Reason:* per-tick accumulation costs every tick and drifts; a daily keyed draw is the same model at 1/1,440 of the work.
- **Do** give a new periodic system a phase (`RuntimeSystem::tick_offset`) that keeps it off the ticks where the hour-aligned systems run (interval 60, 120, 240 … at offset 0; the clock offset 480 makes those the whole hours), and split a costly periodic pass into slices over consecutive runs (the quarrel check: five slices of initiators by `AgentId % 5`, one every 6 ticks, offset 3) rather than doing it all at once. A system whose logic reads the clock must then test the hour (`minute / 60 == 8`), never an exact minute (`minute == 480`), and needs a test that its once-a-day step runs exactly once a day across a save/load. *Reason:* the tick's p99 is set by the ticks where periodic costs stack; M5's quarrel check at offset 0 landed on every hour-aligned tick and raised p99 ×1.22–1.63 with the mean within +10 % (M5 DECISIONS D18).
- **Don't** add new `f32`/`f64` state to the sim crates; new state is integer or fixed-point (permille, per-64k). Existing float needs are grandfathered until M-PERF converts them (C4).
- **Do** take every length of time from `WorldRules.calendar` (`CalendarRule::ticks_per_year`, `years_to_ticks`, `days_to_ticks`, `WorldRules::season_days`), never from a literal 1,440 / 7 / 28. *Reason:* the colony's calendar differs from the baseline's (ADR 0017); a literal silently keeps V7's.

- **Do** register every new ECS component in `sim_systems::save::world::registry()` and give a stateful system `save_state`/`load_state`; bump `sim_systems::save::FORMAT_VERSION` whenever a saved type changes; mark derived state `#[serde(skip)]` and rebuild it on load. *Reason:* an unregistered component fails every save, and a silently changed layout would misdecode old saves (C1's save round trip, gate step `save_roundtrip`).

### Headers and size
- **Do** start every `.rs` under `src/` with a `//!` header giving its purpose. Systems also give priority, interval, `Reads:`, `Writes:` and events (template in Procedure A). *Reason:* `tools/v8/system_table.py` reads `Reads:`/`Writes:` from the leading `//!` block into `docs/ARCHITECTURE.md`.
- **Do** start every kept `.gd` with a comment block giving its purpose and the bridge methods it calls. *Reason:* the bridge surface must be findable from the GDScript side.
- **Do** report every `src` file over 1,200 lines in the pass report (Procedure C). *Reason:* size is tracked, not a failure; splits are pure moves done on purpose, never by drive-by refactor.

## Procedures

Full step lists with code skeletons: `references/procedures.md`.

### A. Add a system
1. Grep first: the feature may already exist (`grep -rn "<Name>\|<Type>" rust/crates/`); check whether it is registered and whether the bridge exposes it.
2. Add or extend the component in `rust/crates/sim-core/src/components/<name>.rs` (`#[derive(Clone, Debug, Serialize, Deserialize)]`, `///` docs) and any rule fields in `rust/crates/sim-core/src/rules/<sub>.rs` with `#[serde(default)]` and the baseline value in `rules/baseline.rs`. A new rule field changes the rules dump; update the expected dump in the same commit and say why in the commit.
3. Write `rust/crates/sim-systems/src/runtime/<domain>/<name>.rs`, starting with the header:
   ```rust
   //! <Name>System: <one sentence of what it does>.
   //!
   //! Priority: <n> (`<NAME>_PRIORITY`). Interval: <k> ticks.
   //! Reads: <components and SimResources fields, one line>
   //! Writes: <components and SimResources fields, one line>
   //! Events: <chronicle kinds / causal events emitted, or none>
   ```
   `fn name(&self) -> &str { "<Name>System" }` must be a string literal (the table generator matches it). Return at once when the system's rule is `None`.
4. Apply the determinism rules: sorted iteration, structural changes collected and applied in `AgentId` order after the query, keyed draws on a new stream.
5. Register it in a `register_*_systems` helper called from `register_default_runtime_systems` (`rust/crates/sim-systems/src/lib.rs`). Pick an unused priority; if it must share one, registration order decides.
6. Test it on the rules it runs under: a domain test in `rust/crates/sim-test/tests/<domain>_*.rs` on `sim_test::baseline_engine(seed)` (V7 systems) or `sim_test::colony_engine(seed)` (colony systems) with a threshold on a measured invariant (see `worldsim-testing`), plus `#[cfg(test)]` unit tests in the module.
7. If the UI needs the data, do Procedure B.
8. Regenerate the system table: `python3 tools/v8/system_table.py --write` (it runs the `worldsim_systems` binary), then `bash tools/v8/check_docs.sh`.
9. Run the gate (`tools/v8/gate.sh <log_dir>`). A new system changes behaviour, so identity is expected to fail: record that and the new base in DECISIONS/CHANGELOG instead of claiming "no behaviour change".

### B. Add a bridge read
1. Check the getter does not exist yet (`docs/ARCHITECTURE.md` section 4, `GETTERS` in `scripts/core/sim_reads.gd`).
2. Pick the file in `rust/crates/sim-bridge/src/ffi/` for the concern (the list is in `docs/ARCHITECTURE.md` section 6: `snapshot.rs` per-frame arrays, `reads_agent.rs`, `reads_settlement.rs`, `relations.rs`, `chronicle.rs`, `family.rs`, …; `node.rs` holds only the class, the process loop, speed and every `#[func]`). A new concern gets a new `<concern>.rs` with a `//!` header, a `pub mod <concern>;` line in `ffi/mod.rs` and a `pub use` of the collector.
3. Write a pure `pub fn collect_<thing>(world: &hecs::World, resources: &SimResources, …) -> Vec<<Thing>Row>` (or a plain struct / `Option<Row>`) there. No Godot types, rows in a deterministic order (sorted by id), the id space of every argument and field documented (`AgentId`, entity bits or row index).
4. Re-export it from `rust/crates/sim-bridge/src/ffi/mod.rs` (`pub use <concern>::collect_<thing>;`). sim-bridge is built as `cdylib` (Godot loads it) and `rlib` (sim-test links it), so sim-test calls `sim_bridge::ffi::collect_<thing>` headless.
5. In the same file write `pub(super) fn <thing>_to_dict(...) -> VarDictionary` that builds parallel `Packed*Array` columns (as `agent_family_to_dict` in `family.rs`). In `node.rs` import both with `use super::<concern>::{collect_<thing>, <thing>_to_dict};` and add `#[func] fn get_<thing>(&self, …) -> VarDictionary` to the `#[godot_api] impl WorldSimNode`; the body only calls the two. At most 14 arguments (pack the rest, for example into a `Vector2i`). Read-only: a getter never mutates. Return `VarDictionary` unless a single scalar is the whole answer (existing exceptions: `get_agent_id -> i64`, `get_tile_causal_history -> VarArray`).
6. Test the collector in `rust/crates/sim-test/tests/engine_bridge_ffi.rs` or a new `engine_<concern>_ffi.rs` (`sim_bridge::ffi::collect_<thing>` on `baseline_engine`, or on `colony_engine` for colony state, which baseline does not have), asserting content and order, not just presence. Run one file with `cd rust && cargo test -p sim-test --test <file>`. A colony getter returns its empty form (version 0, empty columns) in baseline; test that too. `cargo test -p sim-bridge` unit tests are for pure marshalling helpers only.
7. GDScript: an argument-less getter goes in `SimReads.GETTERS` (cached once per frame; also `TICK_GETTERS` to cache once per simulation tick) and is read with `SimReads.read(world_sim, "get_<thing>")` as a literal name. A getter with arguments is not in `GETTERS`; GDScript calls it directly (`world_sim.get_<thing>(args)`), never per node per frame. Name it in the calling `.gd` header comment.
8. `cd rust && cargo build -p sim-bridge` before any Godot run (Godot loads `rust/target/debug/libsim_bridge.dylib`, which `cargo test` does not rebuild). Import only on a fresh checkout or worktree or after adding or moving assets, scenes, scripts or a `class_name`; a new `#[func]` alone needs none. Then `bash tools/v8/gdscript_strict_check.sh` and `bash tools/v8/smoke.sh <log_dir>`.
9. `tools/v8/identity.sh` must still pass: a pure read changes no simulation state, so a failure means the read is not pure (a bug).
10. Update the bridge API check: write the new full list with `python3 tools/v8/bridge_api.py . > docs/<line>/bridge_api_<pass>.txt` (`<line>` = the docs folder of the pass's line, `v8` for V8 passes; `<pass>` = the pass name; M1 writes `docs/v8/bridge_api_m1.txt`), change gate step 13 in `tools/v8/gate.sh` to `python3 tools/v8/bridge_api.py . --check docs/<line>/bridge_api_<pass>.txt` (no `--minus`) and the list `tools/v8/test_bridge_api.py` reads (gate step 16), update the method table and the check command in `docs/ARCHITECTURE.md` section 4, and add a CHANGELOG line naming the added or removed method — all in the same commit (`check_docs.sh` does not compare the section 4 table). `<pass>` is the pass name in lower case with `.` as `_` (M1.5 → `bridge_api_m1_5.txt`); the list goes in the line's folder (`docs/v8/`), not the pass's records sub-folder. Then run the whole gate (`tools/v8/gate.sh`). An ADR only if the read encodes a design decision. The V8-0 lists (`docs/v8/bridge_api_v7_final.txt`, `docs/v8/bridge_api_removed.txt`) stay as V8-0's record.

### C. Size and header report
1. `git ls-files 'rust/crates/*/src/*.rs' 'scripts/*.gd' | xargs wc -l | sort -rn | awk '$1 > 1200 && $2 != "total"'`.
2. Put the list in the pass report as a table (path, lines). Not a failure.
3. Check headers: `git ls-files 'rust/crates/*/src/*.rs' | while read -r f; do head -1 "$f" | grep -q '^//!' || echo "$f"; done` must print nothing (git pathspec `*` also matches `/`).

### D. Add a command (a player order)
1. Add the variant to `Command` in `rust/crates/sim-core/src/colony/command.rs` (plain integers and tiles, serde) and, if it can fail in a new way, a `RejectReason`.
2. Handle it in `apply_command` (`rust/crates/sim-systems/src/commands.rs`): check first (colony on, in map, not water, not blocked), then apply in tile order; return `CommandResult { accepted, count, reason }`; bump `trees_version` / `overlay_version` when the reads must refresh. Never touch jobs except through `jobs::cancel::cancel(…, CancelReason::Command)`.
3. Bridge: a `#[func] cmd_<name>` in `ffi/node.rs` that builds the `Command` and calls the `ffi/colony.rs` helper, which calls `commands::submit` and turns the result into `{accepted, count, reason}`. Add it to the bridge API list (Procedure B step 10).
4. Probe: if scripts should use it, add it to the probe's command file parser (`rust/crates/sim-test/src/probe/colony.rs`); the replay test then covers it.
5. Tests in `rust/crates/sim-test/tests/colony_commands.rs`: accepted case with the exact count, each reject reason, and that the log entry carries the next tick.

### E. Add a job type
1. Add a `JobKind` variant (`rust/crates/sim-core/src/colony/job.rs`) and, if needed, a `WorkType`; give it `keys()` per stage (what it reserves while walking to the source and to the destination).
2. Assign: a branch in `pick` (`runtime/jobs/assign.rs`) in the right priority order (today Build/Deliver > Chop > Haul), choosing the nearest target by `(distance, tile)` among unreserved ones; reserve every key with `cancel::reserve`.
3. Execute: arms in `target_exists`, `step` and `arrive` (`runtime/jobs/execute.rs`), and the `expected_state` it walks or works in. Items move only through `put_items` / `take_items`; finishing goes through `cancel::finish`.
4. The seam needs no change if the job is walked as `Seeking { Job }` / worked as `Consuming { Job }`.
5. Ledger: if the job produces or consumes items, update `item_ledger` in the same function and extend `jobs::invariants::check`.
6. Tests in `rust/crates/sim-test/tests/colony_jobs.rs`: the job completes, the ledger holds every tick, reservations are empty after a cancel by each `CancelReason`, a twin run stays identical.

### G. Add an item kind (M2)
1. Add the variant to `ItemKind` (`rust/crates/sim-core/src/colony/items.rs`) with its `code`, `name`, `ALL` entry, `default_stack_max` and `default_carry`; pin the codes in the unit test. The per-kind ledger (`jobs::invariants::check`) covers it automatically (it loops `ItemKind::ALL`).
2. Every item that enters the world is `item_ledger.create`d where it appears, and every item that leaves is `consume`d (or `craft_start`/`craft_end`/`burn`); move items only with `put_items`/`take_items` (a stack holds one `ItemSpec` bucket: kind, material, quality / 10).
3. Bridge: `ItemKind::code` is in `carry_kinds`, `item_kinds`, `shelf_kinds`; update `docs/v8/snapshot_schema.md`, the GDScript icon/name maps and the locale keys.
4. A new kind that is stored needs no Haul change (Haul moves any loose stack to a cell that takes its spec).

### H. Add a recipe (M2)
1. Add an entry to `rust/crates/sim-core/data/colony/recipes.ron` (names are parsed at load; an unknown name makes the whole file fail its unit test) and a `RecipeKind` variant with `code`/`name`/`ALL`.
2. Its workshop must accept it (`recipe.workshop == shop.kind`, checked by `AddBill`); its `work` (Craft or Split) decides which priority column it uses; its `skill` decides speed, quality and XP. The output's material is the first input's.
3. Tests in `colony_workshops.rs`: a `Make(n)` bill makes exactly n from known inputs (ledger transforms = n), and a cancelled craft loses its inputs (`crafts_lost`).

### I. Add a work type (M2)
1. Add the `WorkType` variant at its place in the tie-break order (`ALL`, `code`, `name`); `WorkPriorities.levels` is indexed by `code` (a fixed array: bump its length, the save format and every literal).
2. Add its arm to `pick_work` (`runtime/jobs/assign.rs`) with a memo field in `Exhausted` for each map-wide search; the priority passes call it only for people whose level for it matches the pass.
3. Bridge `get_work_priorities` lists `work_types`; the GDScript priority table reads that list.

### J. Add a blueprint kind for built space (M3)
1. Add the `BlueprintKind` variant (`rust/crates/sim-core/src/colony/orders.rs`) with its `code`, `name`, `plan` (`MaterialNeeds` lines and work) and the predicates `is_space`/`is_wall`; pin the code in the unit test.
2. Placement: `commands::check_space` decides each tile (one `RejectReason` per tile, never a silent drop); the line/rect commands take it automatically. A kind that blocks walking gets its arm in `runtime::nav::tile_cost` (site and built).
3. Completion: `jobs::space::complete_site` writes the tile grid (and `colony.beds` for a bed), consumes the materials (ledger), calls `nav::refresh` and sets `colony.rooms_dirty`; `jobs::space::deconstruct` gives back half.
4. Bridge: `BlueprintKind::name` is the order name `ffi/space.rs` parses; `get_space` columns, `docs/v8/snapshot_schema.md`, a sprite in `tools/v8/gen_tiles.py`, the build-menu key and locale keys.
5. Tests in `colony_space.rs`: built from a stockpile with the ledger exact, refused where it must be, the nav map equal to a rebuild after it.

### K. Add a room role (M3)
1. Add the `RoomRole` variant (`rust/crates/sim-core/src/rooms.rs`) with `code`/`name`/`ALL`; put it at its place in `runtime::rooms::auto_role`'s precedence (Workshop > Hall > Storehouse > Bedroom > Unassigned today).
2. Its effect goes in the system that owns the behaviour, reading `rooms::roofed_as(res, tile, role)`; never in `RoomSystem` itself.
3. Bridge: the role name in `get_rooms`/`get_room_detail`, the override dropdown and its tint in `scripts/world/colony/room_layer.gd`, locale keys; a test in `colony_rooms.rs` for detection, override and the effect.

### L. Add a knowledge node (M4)
1. Append it to `rust/crates/sim-core/data/colony/knowledge.ron` (file order is the id; never reorder — saves and chronicle values carry ids; at most 32 nodes, the bitset and the 5-bit id): lane, era, threshold, prerequisites, difficulty, discovery conditions, gates, marker weight, seed, name key.
2. A new condition or gate kind needs its parser arm in `sim_core::colony::knowledge` and its evaluation in `runtime/knowledge/discover.rs` (conditions) or `knowledge::gates()` (gates); a marker weight changes the era denominator (ADR 0026: update the ceiling in tests and docs).
3. Locale keys (name, description) in `localization/{ko,en}/ui.json` and the tech map's lane/era cell (`scripts/ui/panels/tech_panel.gd`).
4. Tests in `colony_knowledge.rs`: seeding counts, gating of what it gates, its discovery when its conditions hold.

### M. Add a gate (M4)
1. A recipe, blueprint or designation that only some may do: add `Recipe(..)`, `Blueprint(..)`, `Workshop(..)` or `Designation(..)` to its node's `gates` (or list it as universal in the pass's DECISIONS).
2. Check every path that can advance that work (`git grep` the kind): the job assignment's arm, bills, and any V7 arm that works sites or tiles in place; each must test `gates().<kind>(..) & !practice == 0`.
3. Waiting orders show their reason: `collect_knowledge_waits` (`ffi/knowledge.rs`) covers bills, sites and marks; a new kind of order needs its rows there.
4. Test: a non-practitioner never takes it (and never advances it in place); a practitioner does; the order waits with the node.

- **Do** keep run-time rates in integer tables (`XP_TABLE`, `SPEED_PERMILLE`, `QUALITY_BONUS_TABLE`, `AXE_PERMILLE` in `sim_core::colony::skills`) and convert data files' floats once at load (`MaterialStats`). *Reason:* C4; a table is reproducible and readable in a test.

### N. Add a pressure kind (M5)
1. Add the variant to `sim_core::colony::story::PressureKind` (code, name; list every `match` first) and its numbers to `StoryRule` (`#[serde(default)]`, M5 value in `StoryRule::M5`).
2. Planning: its season weights, mercy day, gap and due rule in `runtime/story/mod.rs` (`plan_season`, `due`, `force`); begin and end with a chronicle `PressureBegan` / `PressureEnded` and its intensity.
3. Effects: in the behaviour systems, each asking `story::active(res, kind)`; costs through `story::add_cost`, first effects on a person through `story::note_cause` (one `PressureCause` event a person a pressure).
4. Mood: a modifier only if the goal gives one (procedure Q); the quarrel cause map (`Modifier::quarrel_cause`).
5. Bridge and UI: `get_story` rows carry it by code; a banner text and locale keys; the year summary's `pressures`.
6. Tests: the schedule on seeds 0–7 and the held-out set (`colony_story.rs`), the effect's size against a twin without the pressure, the chronicle pair with its cost.

### O. Add an animal (M5)
1. `AnimalKind` variant (code, name), its hp and behaviour numbers in `WildlifeRule`; placement and births in `runtime/wildlife/mod.rs` (keyed on `STREAM_WILDLIFE`; deer live as a herd within `deer_herd_max_tiles` of the hearth — the job radius limits who can reach it).
2. Its steps (`AnimalState` codes are a bridge contract; add new ones at the end) and its fights through `colony.fights` (`Fighter::Animal`), resolved by `CombatSystem` (`wildlife/fight.rs`).
3. What it leaves: items through the ledger helpers (`put_items`; new kinds by procedure G), a chronicle kind if the goal names one.
4. Hunting it: `hunt::pick` and `WorkType::Hunt` (gated by `tracking`); its reservation key `ReserveKey::Animal`.
5. Bridge: `get_animals` rows (kind code), a sprite in `tools/v8/gen_tiles.py`, the schema doc.
6. Tests in `colony_wildlife.rs`: placement on reachable ground, the kill and its items in the ledger, fights resolve and end.

### P. Add a wearable or a piece of equipment (M5)
1. The item kind (procedure G) and its recipe (procedure H; a knife recipe sets `knife: true`); the effect's factor as a `const fn` on `Top` (or a new enum) — integer ‰.
2. Wearing is a job (`JobKind::Equip { from }`, the hourly dressing pass in `hunt::dress`): the neediest *free* adult in a deterministic order (hardiness, then `AgentId`); a busy person is served at a later pass.
3. The ledger counts a worn item as held (`invariants::ledger_of`); a death drops it like a held tool; the save registers the component.
4. The effect acts on what its bar measures (rule above): for a top, the keyed warm-hour roll in `FireSystem` (`STREAM_APPAREL`).
5. Bridge: the snapshot's `tops` column and `get_agent_m5`'s `top`, `top_cold_permille`; the inspector line.

### Q. Mood modifiers (M5; goal 3.4)
The modifier table lives in `sim_core::colony::people::Modifier` (`weight`, `social`, `quarrel_cause`); `mood::modifiers` reads the facts. Harm avoidance scales a negative one ×(800 + ha/5) ‰, reward dependence a social one.

| modifier | weight | fact | quarrel cause |
|---|---|---|---|
| slept in a roofed bedroom | +80 | last night's sleep (20:00–06:00) | — |
| slept outside | −120 | last night's sleep | sleep |
| fed / hungry | +40 / −150 | hunger below / at the seek threshold | — / hunger |
| thirsty | −100 | thirst at the seek threshold | hunger |
| cold last night | −120 | marked `Cold` in the night window | cold |
| quarrelled today | −100 | `Mood::quarrel_day` | — |
| taught or learned today | +60 | `mood::note` (teach, oral) | — |
| discovered something | +150 for 3 days | `mood::note` (discovery) | — |
| a kin died | −250 for 6 days | `mood::kin_died` (despawn) | grief |
| sick | −80 | `Sick` | pressure |
| worked a gated craft | +40 | `mood::note` (knowledge used) | — |
| drafted a day or more | −60 | `Drafted::since` | pressure |
| a pressure active | −40 | `story::any_active` | pressure |

A new modifier: append it to `Modifier::ALL` (codes are chronicle and bridge contracts), give it a weight, a fact read in `mood::facts_of` / `modifiers`, a quarrel cause, a locale name, and a unit test in `mood/mod.rs`.

### R. Add a neighbour event (M5.5; ADR 0031, 0032)
1. Visitors (a wave, a caravan) are records in `ColonyState::neighbours`, not ECS entities: an agent standing at the
   hearth is admitted to the settlement by V7's proximity sync, eats and appears in every census (M5.5 D16). A visitor
   becomes people only through a command (`AcceptMigrants` → `neighbours::join`, the founders' component set,
   `AgentId`s reserved when it set out so the dialog names them as the colony will).
2. Plan its date on the season's first day in `NeighbourSystem` (keyed, `STREAM_NEIGHBOURS` / `STREAM_TRADE`, subject a
   fixed tag, the season index as tick) and let `VisitorSystem` walk it (one group, one tile a step, nav paths, a
   `VisitState`); never during a major pressure (read the storyteller's planned and active majors).
3. Chronicle its arrival and departure (kinds 38–44 hold the neighbour in `cause`, numbers packed in `value`); a bridge
   row in `ffi/neighbours.rs`; locale keys.
4. Tests in `colony_neighbours.rs`: its window on the calibration seeds, a save while it walks in, its command's
   refusals.

- **Do** move every item a trade gives or takes through the ledger's trade terms: taken goods `trade_in` where they
  appear (the caravan's spot), given items `trade_out` where they leave (a `JobKind::Trade` haul's arrival). The
  identity is created + traded_in = ground + carried + held + delivered + shelved + consumed + traded_out per kind
  (`invariants::LedgerTerms::balanced`). *Reason:* a trade that creates or consumes silently is a ledger hole the
  per-tick check cannot tell from a bug.
- **Do** put content and tunables in data (RON under `sim-core/data/colony/`, embedded with `include_str!`, parsed once, a malformed file turning the feature off rather than panicking, a unit test parsing the embedded file) and match kinds by a tag or category from that data, never by a literal name; add no new closed enum for content (an unavoidable one goes into the pass's DECISIONS as M-EXT debt). *Reason:* owner directive 2026-10-24 (standing; ADR 0033 to come in M-ART2 Part 0): every structure must be extensible; M5.5 moved its neighbour, goods, story, wildlife and mood tables to data with the evaluation byte-identical (M5.5 D28).
- **Do** settle a debt of items (a caravan's `owed`) when the items move — the pickup — not when a job is assigned
  for them. *Reason:* M5.5 review F1: owed blades left the list at assignment while still on the stack, so the same 12
  blades were given twice and the ledger (which books only deliveries) could not see it.
- **Do** gate every path of a rule-bound feature on its rule, not only the headline effect. *Reason:* M5.5 review F2:
  eureka's ×3 was gated on `ColonyRule::neighbours` but its "stockpile conditions count as met" path ran for every
  rule, and the pass's own evidence (a `--neighbours off` run) measured that ungated path.
- **Do** keep the eureka rule a widening of the same keyed draw: a clue wins on `draw × 1,000 < eureka_permille`, no
  clue on `draw == 0`, so a world without clues draws exactly as before (`discover.rs`). *Reason:* a second draw for a
  clue would shift nothing else (keyed) but would make the clue path differ from the plain one by more than its odds.

## Lessons

| date | symptom | root cause | fix | source |
|---|---|---|---|---|
| 2026-10-23 | M5's benchmark kept the tick mean within +10 % of M4 but raised p99 ×1.22–1.63 | `QuarrelSystem` ran every 30 ticks at offset 0, so its whole pass (O(N²) neighbour gathering in a crowd: 279 people on one tile at N = 1,000) landed on the hour-aligned ticks where the 60/120/240-tick systems also run | cap the per-run work (the nearest 8 partners), split the pass into slices (five, by `AgentId % 5`, every 6 ticks, offset 3), move periodic systems off the hour (MoodSystem at minute 7, its morning test by hour); read the bench's `p99_tick` breakdown before guessing | `rust/crates/sim-systems/src/runtime/mood/quarrel.rs`; `docs/v8/m5/M5_DECISIONS.md` D18 |
| 2026-10-23 | M5 calibration: one world in eight never hunted although tracking was known and the herd was reachable | the deer herd's centre was drawn from all forest beyond 20 tiles of the hearth and landed 110 tiles out, beyond the job layer's 96-tile scan radius | place what a job must reach within the job radius (`deer_herd_max_tiles` 48); when a job never runs on a seed, check the target against the radius and reachability before tuning its odds | `rust/crates/sim-systems/src/runtime/wildlife/mod.rs`; `docs/v8/m5/M5_DECISIONS.md` D8 |
| 2026-10-23 | hunting barely ran on any world (rounds 0–2) although hunters were idle at times | `WorkType::Hunt` sat after Quarry and Mine in the work order, which always had work | put a food job right after building in the work order; after adding a work type, count its jobs taken per world on the first round | `rust/crates/sim-core/src/colony/job.rs`; `docs/v8/m5/M5_DECISIONS.md` D8 |
| 2026-10-23 | three of six hide wraps lay unworn on the stockpile for days | the hourly dressing pass kept each top for the neediest person and skipped the pass when that person was busy — at the one minute of the pass they nearly always were | serve the neediest *free* person; a periodic pass must not wait on one person's state at its minute | `rust/crates/sim-systems/src/runtime/jobs/hunt.rs`; `docs/v8/m5/M5_DECISIONS.md` D10 |
| 2026-10-23 | clothed and unclothed people had the same cold hours (8 vs 12 of 600) although a wrap cut exposure to 700 ‰ | the factor scaled only the damage; the `Cold` marker comes from the hearth being out, the same for the whole settlement | make the effect act on the quantity the bar measures (a keyed warm-hour roll), so events and damage scale together | `rust/crates/sim-systems/src/runtime/settlement/fire.rs`; `docs/v8/m5/M5_DECISIONS.md` D15 |
| 2026-10-23 | the F10 loss fixture's seed 5 finished two door sites after the colony lost the doorway, with no living door maker | an idle adult standing on a delivered site took the V7 decision system's construction arm and `ConstructionSystem` advanced the site; the job layer's gate was correct but this second builder never asked | put the gate in every path that can advance gated work; the decision system's site snapshot leaves out knowledge-gated colony sites (`knowledge::open_to_any_builder`); test with a person kept on the site | `rust/crates/sim-systems/src/runtime/knowledge/mod.rs`; `docs/v8/m4/M4_DECISIONS.md` D22 |
| 2026-10-23 | knowledge cost 161–212 B a person with 12-byte entries in a `Vec`, over the plan's 160 B | a `Vec`'s 24-B header and spare capacity plus a `u32` tick per entry, for a list that changes only on a rare insert | store a boxed slice of packed 6-byte entries (use day, ‰, an id+source byte, the watched part) and rebuild the slice on insert; measure bytes per person in the probe | `rust/crates/sim-core/src/colony/knowledge.rs`; `docs/v8/m4/M4_DECISIONS.md` D10 |
| 2026-10-22 | the colony's founders were stuck on hill tiles (seed 5) and a whole colony died of thirst in a hill basin (seed 403) | M3 made undug hill impassable, but V7 worldgen placed founders and camps assuming every hill tile could be walked | when a rule changes passability, re-check worldgen's assumptions: founders on open ground (D9), the camp's region open (D17); test every calibration and evaluation seed's camp region | `rust/crates/sim-systems/src/worldgen/colony.rs`; `docs/v8/m3/M3_DECISIONS.md` D9, D17 |
| 2026-10-22 | dig jobs churned (1,000+ releases a seed): half-thirsty workers took a 200-tick dig, crossed the thirst line on the way and dropped it | the job layer offered work to people about to need a drink; the decision then released the job | do not offer new work to someone at or past a hunger or thirst seek threshold (`jobs::assign`, D16); read the probe's churn report by job kind after adding a long job | `rust/crates/sim-systems/src/runtime/jobs/assign.rs`; `docs/v8/m3/M3_DECISIONS.md` D16 |
| 2026-06-22 | two engines differing only in HashMap seeds diverged (query order at tick 9, an inventory at 489, the run at 2251) | `AgentDecisionSystem` iterated a `HashSet` and did a structural `insert_one` per entity; archetype dense order is insertion order, so hash order permuted every later query | collect `(AgentId, Entity)`, sort by `AgentId`, then insert; new collections are `BTreeMap`/`BTreeSet` | `c03ae2bc`; `v7-final-art:docs/revitalize/RNG_INVENTORY.md` §2; `v7-final-art:docs/revitalize/DECISIONS.md` L365 |
| 2026-10-12 | an order-dependence survey found outcome-changing loops beyond the decision loop (bush claims, deaths feeding event ids, construction completions from a HashMap, memory insertion, routing ties) | std hash order differs between processes; hecs order changes with history (swap-remove, archetype moves) | process in `AgentId` order, sort deaths before despawn, sort completions by position, break ties by `(distance, id)`; record latent cases | `v7-final-art:docs/revitalize/RNG_INVENTORY.md` §2 O1–O9; `v7-final-art:docs/revitalize/DECISIONS.md` L535, L547, L573, L581 |
| 2026-10-16 | a test of ascending `LeftSettlement` order still passed about 43 % of the time with the sort deleted | `Settlement::member_agents` is a `HashSet` with `RandomState`; small sets iterate ascending by chance | sort before every iteration or use the set for membership only; the test builds 32 fresh engines | `v7-final-art:docs/revitalize/REVIEW_PASS19.md` Round 2 finding 1; `v7-final-art:docs/revitalize/DECISIONS.md` L593 |
| 2026-10-13 | an exponential threshold was needed while sim math had to stay reproducible across machines | float transcendentals (`exp`, `ln`, `powf`, `sin` …) are not bit-reproducible guarantees | gate greps the three sim crates for transcendental calls (0 hits); exponentials come from a committed 1,025-entry integer table with integer interpolation; widen the grep for `.log(`, `ln_1p`, `exp_m1`, `asin` … | `v7-final-art:tools/revitalize/gate_pass21.sh` L39; `v7-final-art:docs/revitalize/DECISIONS.md` L730; `v7-final-art:docs/revitalize/REVIEW_PASS16.md` Round 1 L9 |
| 2026-10-12 | a noise fork or one extra event shifted every later random value of an agent | sequential `MovementRng` streams had state-dependent draw counts (idle steps, paused dx/dy, top-ups drawn only when needed) | keyed draws `keyed_u64(seed, stream, subject, tick, slot, replicate)`: stateless, consume no stream state, so skipping one is deterministic; data-dependent draws become fixed slots | `v7-final-art:docs/revitalize/RNG_INVENTORY.md` §1; `v7-final-art:docs/revitalize/DECISIONS.md` L538, L545–546; `v7-final-art:docs/revitalize/REPORT_PASS21.md` |
| 2026-10-12 | keyed draws could give identical values to different events | a key without a distinguishing field (subject 0 for key-less movers; two disputes in one tick with the same lower-id forager) | keep the sequential stream for key-less movers instead of subject 0; put an event index in `slot` when a subject has several events per tick | `v7-final-art:docs/revitalize/DECISIONS.md` L545; `v7-final-art:docs/revitalize/REVIEW_PASS20.md` Round 1 N3 |
| 2026-10-12 | switching `% 3` to multiply-shift `(u × n) >> 64` changed trajectories | same distribution, different values | treat any draw-formula change as a behaviour change; never as a refactor | `v7-final-art:docs/revitalize/DECISIONS.md` L545 |
| 2026-10-10 | a new random stream could silently reuse a salt or advance an existing stream | salts are scattered 64-bit constants; reusing `MovementRng` would shift movement | grep every `0x…_…` constant before picking a salt; keep stream ids in `STREAMS` with a distinctness test; never advance the movement stream for a new purpose | `v7-final-art:docs/revitalize/DECISIONS.md` L317, L366, L538, L607 |
| 2026-10-10 | mutating the world during a query risked order effects and archetype churn | despawns and inserts inside the loop reorder archetypes | collect deaths and promotions in the query, apply them in `AgentId` order after it; update a present component in place | `v7-final-art:docs/revitalize/DECISIONS.md` L318, L596 |
| 2026-10-16 | the 18:00 tick cost 0.8 ms: 748 archetype moves when every activity ended at once | `remove_one` + insert per entity passes through intermediate archetypes | `World::exchange` fuses remove + insert (checked against hecs 0.10.5 that archetype and entity order are unchanged; identity proved) | `v7-final-art:docs/revitalize/DECISIONS.md` L1041–1043; `v7-final-art:docs/revitalize/REVIEW_PASS18.md` Round 1 §C.8 |
| 2026-10-15 | a temperament lean had no effect (ρ −0.010) and relation drift stopped short of its base | integer truncation: `span × d / 500` is 0 for abs(d) < 500; `(base − v) × 30 / 1000` is 0 for gaps ≤ 33 | choose spans and rates so steps are visible at realistic inputs; unit-test truncation edges; document the stall band | `v7-final-art:docs/revitalize/DECISIONS.md` L838, L1239 |
| 2026-10-10 | fractional per-pulse rates (6 units/day over 12 pulses) lost units | truncating `per_day / pulses` | permille accumulators: `acc += permille; units = acc / 1000; acc %= 1000` | `v7-final-art:docs/revitalize/DECISIONS.md` L324, L431 |
| 2026-10-10 | haul-home, the probe and the decision system disagreed on when foraging was on | movement and probe gated on `forage`, the decision system on `forage && stock_target` | one shared gate predicate wherever a feature is tested | `v7-final-art:docs/revitalize/DECISIONS.md` L341 |
| 2026-10-18 | Pass 20: 426 of 511 quarrels happened after the pair was already estranged; no pair reconciled | the avoid filter was added to the three selectors the brief named; the opportunistic Gather arm (`nearest_gather_tile`) and its post-pass re-resolution never consulted it | grep `TargetKind::GatherFood` and every caller of every function that can set the target; pass the same filter at the trigger and in the post-pass | `v7-final-art:docs/revitalize/DECISIONS.md` L1183, L1198, L1234; `v7-final-art:docs/revitalize/REVIEW_PASS20.md` Round 1 M1 |
| 2026-10-19 | two villages sharing a hearth would share an existence answer computed with different avoid lists | a memo keyed by (tile, catchment) only, not by every input | key memos by every input; test the shared-hearth case | `v7-final-art:docs/revitalize/DECISIONS.md` L1235, L1257; `v7-final-art:docs/revitalize/REVIEW_PASS21.md` Round 1 N5 |
| 2026-10-18 | Pass 20: exchange never fired in 32 worlds (0 exchanges) | tuned aid thresholds equalled the exchange's food condition and aid was tried first with the same amount, so the exchange branch was unreachable; after reordering, the condition still held on 0 of 3,550 pair-days | after any tuning, count how often each branch's condition holds and assert every intended branch is reachable; fix the condition, not only the order | `v7-final-art:docs/revitalize/DECISIONS.md` L1193, L1237; `v7-final-art:docs/revitalize/REVIEW_PASS20.md` Round 1 m5; `v7-final-art:docs/revitalize/REVIEW_PASS21.md` Round 1 m2 |
| 2026-10-12 | adding an enum variant (`DeathReason::OldAge`, `TargetKind::GatherWood`) touched many files and missed some | exhaustive matches and non-exhaustive counters (probe census, FFI tallies, `Tallies` literal) all need the case | list every consumer with file:line before adding a variant; pin its encoding with a test | `v7-final-art:docs/revitalize/DECISIONS.md` L593, L598, L637 |
| 2026-10-10 | every new public field broke whole-struct literals in tests and fixtures | literals of `Tallies`, `AgentRules`, `FirewoodRule` must list every field | pin rules leaf by leaf; put new flags in a new type instead of a field of a public struct | `v7-final-art:docs/revitalize/DECISIONS.md` L181–191, L819, L1122 |
| 2026-10-14 | the workspace gate went red without any art change (`cargo test`/clippy exit 101) | a sim change added record fields that broke struct-literal fixtures owned by another branch | update every struct-literal fixture in the same commit and run the full workspace gate right after merging | `2c52dd43`; `v7-final-art:docs/art/production7/DECISIONS.md` L7 |
| 2026-10-16 | new rule fields could break older serialized rules | — | `#[serde(default)]` on every new rule field | `v7-final-art:docs/revitalize/DECISIONS.md` L1128, L731 |
| 2026-10-11 | clippy `--all-targets` findings slipped into commits (`needless_range_loop`, `type_complexity`, missing docs) | only scoped clippy ran before some commits | run workspace clippy with `--all-targets`; fix with iterators, aliases and doc comments, never suppression | `f220395b`; `v7-final-art:docs/revitalize/DECISIONS.md` L129–130, L392 |
| 2026-10-18 | porting renderer geometry to Rust had to reproduce Godot output bit for bit | Godot 4.6 vectors are f32 with half-away-from-zero rounding while GDScript scalars are f64; operation order matters | bit-probe the runtime, replay ≥ 2,000 real calls bit-exact in `cargo test -p sim-bridge`, keep the GDScript function as oracle and off-switch, no reassociation or FMA | `37be1b85`; `v7-final-art:docs/art/production12/REPORT.md` L22; `v7-final-art:docs/art/production13b/WALKER-SCOPE.md` L31-37 |
| 2026-10-19 | the first `sim-bridge` test compile failed before any test ran | godot-core 0.4.5 implements `#[func]` parameters only up to 14 arguments; the draft had 15 | pack parameters (width/height into a `Vector2i`) to stay within 14 | `v7-final-art:docs/art/production13b/DECISIONS.md` L28 |
| 2026-10-13 | joins mixed up people and porters | three identifiers cross the bridge: `AgentId`, signed i64 entity bits, snapshot row index | join persons by `AgentId`, porters/activities by entity bits, never by row number; document the id space of every getter | `v7-final-art:docs/art/production7/DECISIONS.md` L11; `v7-final-art:docs/art/production12/REPORT.md` L26 |
| 2026-10-12 | UI work needed sim data that had no getter | the FFI exposed no intervention log or settlement lineage | add only read-only `#[func]` getters (pure collector, columns, unit test) and document the contract | `v7-final-art:docs/art/production6/REPORT.md` L142; `v7-final-art:docs/art/production7/REPORT.md` L84 |
| 2026-10-16 | a harness assumed `advance_ticks` returns a delta, and a capture at an event's tick missed the event | `advance_ticks` returns the final absolute tick; stepping to tick T runs systems 0..T-1 | compare against the expected absolute tick; read an event booked at T after tick T has run | `v7-final-art:docs/art/production11/DECISIONS.md` L13; `v7-final-art:docs/art/production13b/DECISIONS.md` L22 |
| 2026-04-26 | 10 of 11 features labelled "missing" were 85–100 % implemented and were re-implemented | memory and roadmap notes were treated as truth | grep the feature's symbols and check its wiring (registered system, bridge getter) before implementing | `v7-final-art:CLAUDE.md` "5. Memory Axiom < Code Reality" |
| 2026-10-20 | colony calibration round 0: one dehydration death per seed (8 of 8 seeds) with no hut and no thirst problem in V7 | with `shelter_planning: None` the memory "bias-only Construction" arm of the decision system still fired and set `Seeking { ConstructionSite }` with no site to reach, forever | gate that arm on `rules.colony.is_none()`; when a rule is switched off, grep every arm that assumes its data exists; count deaths per cause on the first calibration round | `docs/v8/m1/M1_DECISIONS.md` D20; `rust/crates/sim-systems/src/runtime/decision/system.rs` |
| 2026-10-20 | adding `TargetKind::Job` failed the build in the decision system, the bridge snapshot, the probe census and the test lock lists | exhaustive matches over `TargetKind` across four crates | list consumers first (`git grep -n "TargetKind::"`), then add the arm everywhere with its encoding (target code 9) and a test | `docs/v8/m1/M1_DECISIONS.md` D10, D15; `rust/crates/sim-bridge/src/ffi/snapshot.rs` |
| 2026-10-21 | an exact save/load needed the hecs world rebuilt with the same query order and the same entity ids for later spawns | hecs 0.10.5 iterates archetypes in creation order (empty ones included) and reuses freed ids from a LIFO free list that it exposes only through `World::reserve_entity` | recreate archetypes in order with zero-row column batches, `spawn_at` entities in dense order, rebuild the free list by placeholder spawn → `spawn_at(gen − 1)` → `despawn` in stack order; prove it with a save right after a death | `rust/crates/sim-systems/src/save/world.rs`; `docs/v8/m1_5/M1_5_DECISIONS.md` D17 |
| 2026-10-21 | a spatial index made `JobAssignSystem` 3× slower than the linear scan it replaced (14 % vs 4.4 % of the tick) | the ring search visited every chunk coordinate, and ~150 idle agents re-searched candidates that were all reserved | visit only non-empty chunks by exact distance; memoise within a tick the searches that found nothing anywhere (candidates only shrink inside a tick); measure an index against the code it replaces before keeping it | `rust/crates/sim-systems/src/runtime/jobs/assign.rs`; `docs/v8/m1_5/M1_5_DECISIONS.md` D15 |
| 2026-10-22 | the M1.5 gate went red on `python_unittest` after save/load added two bridge methods although step 13 (`bridge_api`) passed | `tools/v8/test_bridge_api.py` pinned M1's list file; the API change updated only gate step 13's list | when the bridge API changes, update every reader of the list (gate step 13 and `test_bridge_api.py`) and run the Python tests after merging a branch, not only before | `tools/v8/test_bridge_api.py`; `docs/v8/m1_5/M1_5_DECISIONS.md` D19 |
| 2026-10-21 | M1.5 round 0: births per (fed) woman-year read 0.24–0.84, above the rule's maximum | the fertility gate ran at 06:00, when the members' mean thirst is at its overnight peak (56–67 against the limit 60), so the settlement was "fed" on about half the mornings and the exposure count was halved | measure a gate's input at the hour the system will run before choosing the hour; `FertilitySystem` runs at 12:00 (means 25–40) | `docs/v8/m1_5/M1_5_DECISIONS.md` D7 |
| 2026-10-21 | the goal's 12 ‰ daily conception chance was meant to give 0.25 births per woman-year but gives 0.16 | 12 ‰ × 24 days counts only eligible days; gestation and the postpartum days (66) are not | compute a rate's long-run value with its dead time (1 / (gestation + postpartum + (1 − p)/p)) and test the system against an independent recursion | `docs/v8/m1_5/M1_5_DECISIONS.md` D6; `rust/crates/sim-test/tests/colony_demography.rs` |
| 2026-10-20 | a new chronicle kind (`HutBuilt` 22) failed identity on the chronicle files although no baseline event changed | each chronicle file carries the full kinds vocabulary, which grew | compare chronicles by content with the base kinds list as a prefix (`tools/v8/compare_chronicles.py`) | `docs/v8/m1/M1_DECISIONS.md` D21; `tools/v8/compare_chronicles.py` |
| 2026-10-22 | the first M2 candidate's `JobAssignSystem` cost 280 µs per tick at N = 1,000 (was 17) and the decision system's share fell 11 points only because another system grew | the gather fallback rebuilt a 1,600-entry deadwood index every tick although the colony was rarely short; memos moved into a `BTreeMap` looked up per agent | build an index lazily (`OnceCell`, at most once per tick, only when a search needs it); keep per-agent memos as fixed fields or arrays; read a share change next to the absolute µs of every system | `rust/crates/sim-systems/src/runtime/jobs/assign.rs`; `docs/v8/m2/M2_DECISIONS.md` D17 |
| 2026-10-22 | three of 32 evaluation colonies lost people to cold in their third year although firewood lay on the ground all along | the gather shortfall counted every firewood item as held, but the hearth only feeds from stockpiles and its own pile; loads dropped by released gatherers sat far away and blocked gathering forever | count only what the consumer can actually reach (`hearth::feedable_firewood`); when a stock decides whether to make more, its count and the consumer's reach must be the same set | `rust/crates/sim-systems/src/runtime/jobs/hearth.rs`; `docs/v8/m2/M2_DECISIONS.md` D19 |

