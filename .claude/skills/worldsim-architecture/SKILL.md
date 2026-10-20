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
| `rust/crates/sim-core` | ECS components (`src/components/`), `WorldRules` (`src/rules/`: `mod.rs` = `WorldRules`, one file per sub-rule struct, `baseline.rs` = `WorldRules::baseline(seed)`), keyed RNG (`src/rng.rs`), causal ring (`src/causal/`), chronicle (`src/chronicle.rs`), influence grid (`src/influence/`), tile grid (`src/tile/`), materials (`src/material/`), world variety draws (`src/variety.rs`) | hecs, serde, ron, ahash, glam, … |
| `rust/crates/sim-engine` | `RuntimeSystem` trait, `SimResources` (all non-ECS world state), `SimEngine` (system list, tick loop, clock) | sim-core |
| `rust/crates/sim-systems` | every `impl RuntimeSystem` under `src/runtime/<domain>/`, `register_default_runtime_systems` (`src/lib.rs`), worldgen (`src/worldgen/`, `WorldGenConfig::baseline(seed)`, `build_world_with`) | sim-core, sim-engine |
| `rust/crates/sim-bridge` | GDExtension cdylib: one Godot class `WorldSimNode` under `src/ffi/`, pure `collect_*` functions behind thin `#[func]` methods, `build_game_engine(seed)` | the three above + `godot` |
| `rust/crates/sim-test` | `sim_test::baseline_engine(seed)`, scenes, probe (`worldsim_probe`), perf (`worldsim_perf`), domain integration tests in `tests/` | all four |

The tick: `WorldSimNode::process(delta)` runs `engine.tick()` on a fixed 1/30 s
accumulator (at most 5 ticks per frame). `SimEngine::tick` sets
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
- **Don't** let GDScript write simulation state; mutations go through the few bridge commands (`SimReads.MUTATORS` such as `advance_ticks`, `reset_world`, `on_building_placed`, plus `set_sim_speed`). *Reason:* GDScript has no access to the ECS and must never become a second simulation.
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
6. Test it on the live rules: a domain test in `rust/crates/sim-test/tests/<domain>_*.rs` on `sim_test::baseline_engine(seed)` with a threshold on a measured invariant (see `worldsim-testing`), plus `#[cfg(test)]` unit tests in the module.
7. If the UI needs the data, do Procedure B.
8. Regenerate the system table: `python3 tools/v8/system_table.py --write` (it runs the `worldsim_systems` binary), then `bash tools/v8/check_docs.sh`.
9. Run the gate (`tools/v8/gate.sh <log_dir>`). A new system changes behaviour, so identity is expected to fail: record that and the new base in DECISIONS/CHANGELOG instead of claiming "no behaviour change".

### B. Add a bridge read
1. Write a pure `pub fn collect_<thing>(world: &hecs::World, resources: &SimResources, …) -> Vec<<Thing>Row>` (or a plain struct) in the matching `rust/crates/sim-bridge/src/ffi/<concern>.rs`. No Godot types, rows in a deterministic order (sorted by id), the id space of every argument and field documented (`AgentId`, entity bits or row index).
2. Re-export it from `rust/crates/sim-bridge/src/ffi/mod.rs` so sim-test can call it headless.
3. Add the `#[func] fn get_<thing>(&self, …) -> VarDictionary` to the `#[godot_api] impl WorldSimNode`: call the collector, then a private `<thing>_to_dict` that builds parallel `Packed*Array` columns. At most 14 arguments (pack the rest, for example into a `Vector2i`). Read-only: a getter never mutates.
4. Add the getter name to `SimReads.GETTERS` in `scripts/core/sim_reads.gd` (also `TICK_GETTERS` when it may be cached per tick) and read it with `SimReads.read(world, "get_<thing>")` as a literal name.
5. Test the collector in Rust (`sim_bridge::ffi::collect_<thing>` on `baseline_engine`), asserting content, not just presence.
6. Update the bridge API check: `python3 tools/v8/bridge_api.py .` prints the exported list; update the expected list under `docs/v8/bridge_api_*` that gate step 13 compares against (see the step in `tools/v8/gate.sh`) in the same commit, and list the new name in CHANGELOG.
7. `cargo build -p sim-bridge` before any Godot run (Godot loads the cdylib, which `cargo test` does not rebuild), then the strict check and smoke.

### C. Size and header report
1. `git ls-files 'rust/crates/*/src/*.rs' 'scripts/*.gd' | xargs wc -l | sort -rn | awk '$1 > 1200 && $2 != "total"'`.
2. Put the list in the pass report as a table (path, lines). Not a failure.
3. Check headers: `git ls-files 'rust/crates/*/src/*.rs' | while read -r f; do head -1 "$f" | grep -q '^//!' || echo "$f"; done` must print nothing (git pathspec `*` also matches `/`).

## Lessons

| date | symptom | root cause | fix | source |
|---|---|---|---|---|
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
