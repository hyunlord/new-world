# Part 1 implementation handoff

Authorized planning artifact only. No product edits or Godot/cargo workload in preparing this file. Read GOAL section 5 and `READ_AUDIT.md` before implementing. Inspected HEAD: `49bc0b7c87159340b8024d6b9327bc394dc6daa0`; Part 0 measurement/capture scripts are concurrent work and must be reread before editing. This file uses the specifically requested production9 destination, not the planning skill's default `.omo` scaffold. No extra approval or child-agent workflow is required by this authorized handoff.

## Outcome and boundaries

Every existing simulation snapshot consumer under `scripts/` uses one main-scene reader when its **actual world instance** matches that reader. Real main-scene data is read once per engine frame, terrain once per world, with mutation invalidation and immutable Dictionary/Array containers. Fake and standalone worlds retain direct pass-through behavior. Source tests verify migration without confusing fake definitions with calls. Shared snapshots must preserve every existing visual and same-frame test meaning.

Do not start these edits until the leader finishes the Part 0 baseline that requires unchanged production consumers. Do not change Locale, content, assets, shaders, simulation behavior, process priorities, or prior evidence. Counter instrumentation must not alter the measured baseline scripts between the before and after runs except the announced shared support-file addition, with hashes recorded.

## 1. Freeze the shared reader API first

Own `scripts/ui/sim_reads.gd`, class `SimReads`, extending Node, and the single scene addition immediately after WorldSim in `scenes/main.tscn`. No `_process` is needed: use `Engine.get_process_frames()` lazily when reading or querying counters. This also avoids accidentally making SimReads a node-ablation target.

Public static routing API:

```gdscript
static func resolve(world: Node) -> Node
static func read(world: Node, raw_getter: String) -> Dictionary
static func mutate(world: Node, method: String, arguments: Array = []) -> Variant
```

Public instance routing API (for dynamic callers that cannot parse a `SimReads` symbol on the base checkout):

```gdscript
func owns_world(world: Node) -> bool
func read_raw(raw_getter: String) -> Dictionary
func mutate_raw(method: String, arguments: Array = []) -> Variant
func invalidate() -> void
func peek_uncounted(raw_getter: String) -> Dictionary
func frame_call_counts() -> Dictionary
func frame_call_usec() -> Dictionary
func frame_id() -> int
```

`read` and `mutate` first resolve the passed world. A matching reader routes to `read_raw`/`mutate_raw`. With no matching reader, use `world.call(raw_getter)` or `world.callv(method, arguments)` inside **sim_reads.gd only**. Preserve return values; null world reads return `{}`, null mutations return `null`. Production callers already have guards; do not add repeated error logs for expected fake/missing-world tests. Getter results that are not Dictionaries normalize to `{}` as the HUD helpers already do; retain prior guards in consumers where they distinguish malformed snapshots.

Instance named getters map exactly as follows:

| Instance method | Raw WorldSim method | Cache lifetime |
|---|---|---|
| `agent_snapshot()` | `get_agent_snapshot` | engine frame |
| `settlement_snapshot()` | `get_settlement_snapshot` | engine frame |
| `construction_snapshot()` | `get_construction_snapshot` | engine frame |
| `building_snapshot()` | `get_building_snapshot` | engine frame |
| `resource_snapshot()` | `get_resource_snapshot` | engine frame |
| `time_snapshot()` | `get_time_snapshot` | engine frame |
| `tallies()` | `get_tallies` | engine frame |
| `recent_deaths()` | `get_recent_deaths` | engine frame |
| `society_snapshot()` | `get_society_snapshot` | engine frame |
| `intervention_state()` | `get_intervention_state` | engine frame |
| `world_info()` | `get_world_info` | engine frame |
| `terrain_snapshot()` | `get_terrain_snapshot` | world identity/reset |

Each named method is a thin call to `read_raw`, which uses a fixed allowlist. Reject unsupported names with a clear error rather than accidentally caching arbitrary WorldSim operations. Wrappers named exactly like WorldSim are `apply_intervention(kind: String, settlement_id: int, amount: int) -> Dictionary`, `advance_ticks(n: int) -> int`, `reset_world(seed: int) -> bool`, `reset_world_stress(seed: int, agents: int) -> bool`, `on_building_placed(x: int, y: int, radius: int) -> bool`; these delegate to `mutate_raw`. Do not discard the return of advance_ticks: `capture_framings.gd:390` uses it as the loop's progress value.

### World identity and fake seams

- Resolve the real source lazily through the reader's sibling `../WorldSim`. Connect to `world_reset` in `_ready` before later reader nodes' `_ready` callbacks. Revalidate identity on each read/owns-world check, clearing all cached data if the source object changed.
- `resolve(world)` must search `world.get_parent().get_node_or_null("SimReads")` first. This supports a detached main scene before it enters the tree (`test_every_hearth.gd:65–69`) and multiple main instances (`test_nature_density.gd:91–104`). Never select a reader merely because it is named `/root/Main/SimReads`.
- A candidate is accepted only when `candidate.owns_world(world)` is true by Object identity. A root-scene fallback, if needed for injected structures, must apply the same check. No parent, freed parent, fake world, or unmatched world means pass-through.
- Memoize and freeze only when the reader's resolved world `is WorldSimNode`. Fake Nodes always call their getter again, even twice in one engine frame, and retain mutable returned objects. `test_hud_old_age`, `test_feed_priority`, `test_structure_states`, `test_nature_clearance`, and the stockpile harness depend on this behavior.
- Existing renderers' `_world_sim` declarations and injected mock identities must remain intact. Several tests rewrite exactly `var _world_sim: WorldSimNode` to `Node`; adding another declaration with that substring or binding permanently to the scene's real world breaks their seam.
- Do not attach a new caching node to standalone WorldSimNode probes. Static fallback keeps their existing behavior and avoids changing their lifecycle.

### Invalidation and immutable data

Use separate cache-data and counter-frame state. `invalidate()` clears every frame entry but **does not zero counters**, otherwise two mutations/reads in one engine frame undercount FFI. Normal frame rollover clears frame entries and counters. Keep terrain in a separate dictionary plus a validity flag; do not use empty Dictionary as the validity sentinel.

All five mutators call the world, then `invalidate()`, even if the returned operation reports false. Normal/stress reset additionally clear terrain immediately after the call. Their Rust signal is `call_deferred` (`world_node.rs:241,254`), so waiting for it leaves a same-frame stale-map hole. The world-reset handler clears terrain and frame entries again; do not suppress that event or reorder renderer listeners. Source identity change performs the same complete cache clear.

On a real-world cache miss, recursively visit Dictionary values and Array elements, then `make_read_only()` each visited Dictionary/Array. Do not mutate containers while traversing; FFI snapshots are acyclic trees of built-in values. Packed arrays cannot be frozen: preserve the audited no-mutation consumer contract. `probe_packed_p9.gd` and `perf/packed_sharing.log` prove extraction/index assignment shares storage on Godot 4.6.

Required caller repairs: duplicate HUD `settlements` before adding `settlement_ids` (`hud_status_panel.gd:115–117`); replace `god_hand_effects._state` with a new `{}` at reset instead of clearing the cached dictionary (`:65,:79`). Keep the existing duplicated HUD `_clock` and the deep `mock.sites` duplicate in `test_nature_clearance.gd:110`.

### Counter contract already consumed by perf

`perf_frames_p9.gd:10–21` already expects `peek_uncounted(raw_getter)`; `_sample` expects `frame_call_counts()` and `frame_call_usec()`. Keep these exact names and raw-getter arguments.

- Count actual counted FFI misses only, one integer entry per **raw getter name**, e.g. `get_agent_snapshot`. Cache hits count zero. Terrain counts only its actual cold read. Return dictionary copies so the harness cannot corrupt counters.
- Time the actual world `.call` with `Time.get_ticks_usec()` and aggregate integer microseconds per raw getter. Exclude cache lookup and recursive freezing from FFI wall time; they still remain naturally in the frame process measurement. These are conversion-call times, not Rust-only collector times.
- Counter access advances the counter frame when necessary and returns current-frame data even if no counted read occurred (empty dictionary means zero). `frame_id()` gives the counter engine frame; add a perf assertion that it equals the bracket's `sample.process_frame`. The bracket publishes deferred within the frame (`frame_cpu_probe.gd:34–44`); do not silently attach counters to another frame if this assertion fails.
- `peek_uncounted` performs a fresh raw getter call, with no counter write and **no cache fill**. Otherwise harness reads would warm the cache and hide actual renderer miss cost/counts. It can recursively freeze that fresh result for consistency, but must not freeze fake-world results. It does not clear counters.
- Diagnostic getter reads in both `MeasurementRuntime.snapshot` and the inherited A30 `frame_measure_session.Runtime.snapshot` use this path. The latter currently has three direct reads at `:35–39` and must migrate too.
- Do not count mutation wrappers in the getter budget. Mutations happen outside timed frozen blocks. The batched causal call is recorded separately for F4 (zero on steady frozen frames, at most one when called), or instrumented under its own raw-method key exactly once; never add it twice. Leave `get_agent_detail` and `get_influence_overlay` direct and inventory whether their panels are open.

## 2. Resolve the unchanged-base-script/static-rule tension explicitly

Current new tools contain direct fallbacks: `perf_frames_p9.gd`'s `MeasurementRuntime.read` and `_mutate`, `capture_identity_p9.gd:103–108`, `capture_framings_p9.gd:1053–1057`. They need those fallbacks because the base worktree has no SimReads class/node. Simply replacing them with a static `SimReads` reference fails parsing on base; exempting every capture script silently weakens the static rule.

Chosen resolution: centralize the **old-path equivalence implementation** in one actual test file, `scripts/test/art/test_sim_reads_equivalence_p9.gd`, explicitly named in the static test's permitted old-path tests. It extends SceneTree for its own executable equivalence tests, and exposes static helper functions callable when the script is merely preloaded. Its own startup runs only when launched as the test, not when preloaded by captures.

The helper exposes `read(world, raw_getter, uncounted=false)` and `mutate(world, method, arguments=[])`. It obtains a sibling reader dynamically, checks `owns_world(world)`, then calls `read_raw`/`peek_uncounted` or `mutate_raw`. Only when there is no matching reader does it call its separately named `old_read(world, raw_getter)` / `old_mutate(world, method, arguments)` functions. These two functions contain the actual world `call/callv` and are also the old side of this file's equivalence assertions. This is the brief's explicit old-path-equivalence exemption, not a blanket exemption for helper or capture files.

All baseline-compatible measurement/capture scripts preload this test helper and use its routing functions; they must not reference the `SimReads` global class, preload `sim_reads.gd`, or retain their own direct fallback. Copy this helper as an **untracked measurement/capture support script** into the base worktree along with the unchanged driver files. Do not copy the product SimReads implementation or change the base main scene. The driver/helper bytes and hashes must match on base and current. Baseline executes old path; current main executes reader path. Standalone/fake routes still use old pass-through where no reader exists.

Strengthen the static exception: allow direct world calls only inside the two named old-path functions of the one named executable equivalence test; scan its routing functions and all other scripts normally. Add a static assertion that neither capture nor perf script defines a local raw fallback and that the helper's reader branch requires identity equality. Test the helper itself in three states: matching real main reader, unmatched/fake world, and a main scene with the reader removed before adding it to tree. The third reproduces base routing without changing baseline production files.

Record this concrete resolution in DECISIONS.md at implementation. If the leader chooses a different mechanism, retain the literal same-file base operation and the narrow old-path-only allowance; do not hide direct calls with string concatenation or a renamed helper.

## 3. Disjoint implementation ownership

All executors must preserve concurrent edits, reread their scope first, and report cross-scope changes upward. No worker commits independently; leader integrates Part 1, runs gates, stages explicit paths, and pushes the WIP branch.

| Lane | Exclusive write ownership | Specific work / dependency |
|---|---|---|
| Reader API owner | `scripts/ui/sim_reads.gd`, `scenes/main.tscn`, `scripts/test/art/test_sim_reads_p9.gd`, `scripts/test/art/test_sim_reads_equivalence_p9.gd` | Implement API, real/fake identity, resetterrain, readonly recursion, counters and compatibility helper. Publish exact API before other lanes edit. No Rust/other tests. |
| Renderer consumer owner | Existing `scripts/ui/*.gd` except `sim_reads.gd` and `agent_renderer.gd`; existing `scripts/rendering/art/*.gd` | Migrate every inventoried read via static `SimReads.read(actual_world, raw_name)`; named instance methods remain available but avoid per-caller mutable resolver state. Hoist building per-hearth snapshots; repair effects reset alias. Do not touch panel subdirectory. |
| UI panel owner | `scripts/ui/panels/*.gd` | Migrate HUD string helpers centrally; migrate inspector/hand panel; mutation through `SimReads.mutate`; duplicate compatibility settlement dictionary. Preserve fake test injection and Locale behavior. |
| Causal owner | `scripts/ui/agent_renderer.gd`, allowed additive getter/collector/tests in `rust/crates/sim-bridge/src/ffi/world_node.rs`, `scripts/test/art/test_causal_batch_p9.gd` | Migrate its ordinary reads, batch tile histories once per changed tick plus seen-set-overflow rearm. Use the shared router for explicit advances; keep cue timers/frame order. No other Rust. |
| Test/capture owner | All existing `scripts/test/**` except reader/causal new tests; new `scripts/test/art/test_sim_reads_consumers_p9.gd` | Migrate every real call and mutation, including mixed tests and standalone probes; adapt baseline-compatible scripts through equivalence helper; keep fake definitions and fake updates unchanged. Do not run old measurement scripts that write prior docs; use copies where required. |
| Static/ledger owner | `rust/crates/sim-test/tests/harness_art_p9_sim_reads.rs`, existing static-source `.rs` tests that pin migrated call sites, `docs/art/TEST_LEDGER.md` | Exhaustive migration test plus intent-preserving repins and one ledger row each. Coordinate reads of in-progress files; no production code or other Rust tests. |

These are ownership partitions, not an instruction to spawn six agents immediately. The leader chooses bounded delegation after Part 0. Reader API is the dependency barrier; renderer, panel, causal, and capture lanes may then overlap. Static/ledger finalizes after consumer changes land. `READ_AUDIT.md` remains source evidence rather than an editable scratchpad for other lanes.

## 4. Migration details that must survive integration

- Inventory source lines and exact fake-world files are in READ_AUDIT. Include `scripts/rendering/art/god_hand_effects.gd`, `scripts/ui/world_renderer.gd`'s action-triggered agent snapshot, inspector's four conditional snapshot call sites, and tests not loaded by the normal gate.
- Keep `_snapshot(method)` helpers in HUD if useful, but change their body to `SimReads.read(_world_sim, method)`. Static test must trace helper arguments; helper spellings are not exceptions.
- Replace `test_nature_clearance.gd:49–50`'s raw `world.call(method)` with the shared read route. This file loads main and also defines fakes; a fake definition only exempts that definition, never the whole file.
- Hoist settlement/time before loops at building renderer's `:136,319–320,357`. Preserve `_hour()` semantics for tests that directly call `_animate_hearths`. Do not use a once-per-frame computed hour to replace a same-frame fake world's changed hour; the shared fake pass-through must still be evaluated for each explicit refresh.
- Standalone real scripts include `measure_days_p7/p8`, `probe_p6`, `test_feed_casts`, `test_interventions_real_p6`, `test_society_snapshot_p7`. Route them even though the static-test coverage floor is main-loading tests. Exclude `set_sim_speed` from mutation allowlist as specified: it does not modify current snapshots immediately, and ordinary frame rollover observes subsequent ticks.
- For the causal branch, preserve row-order tile deduplication, oldest-first event order, recalls-before-combat ingestion, world-reset tick marker, both seen-set overflow rearm conditions, and existing clear-expired timers every frame. The new Rust getter must be additive only. Verify per-kind ingested `(tick,id)` sequences against old path for W500 600 explicit ticks, not just raw return equality.

## 5. Static coverage and evidence acceptance

Static test walks all game `.gd` files recursively (not just `scripts/ui`) and all test/capture/measurement files that load main plus their known routing/support helpers. Also inspect standalone-real inventory for completed migration. Remove comments before matching, ignore fake `func get_...` definitions and `has_method` guards, and detect typed direct calls, literal `.call/.callv`, and `_snapshot("raw_name")` reaching direct dispatch. Treat raw `world.call(method)` in a helper as a violation unless it is one of the two exact old-path functions above or inside sim_reads.gd. No constructed-string evasion is allowed.

Existing source tests need more than the brief's approximately 13 hearth pins: the current repository has `get_*()` assumptions in p12/p13/p14, s16, show-needs/death and revitalize screen-contract harnesses. Re-pin to the actual read route **and keep the consumer assertion**, not merely assert sim_reads contains the getter. For hoisted hearth reads assert the source still supplies time/winter/settlement data to the same art functions. One TEST_LEDGER row per changed assertion with old/new meaning.

Required acceptance evidence, produced later under the quiet-lock rules:

1. Reader test: real same-frame reads share Dictionary identity, different frames refresh; one cached getter miss counted once; terrain stays cached after advance; normal/stress reset terrain refreshes before deferred signal; invalidate does not reset counters; fake mutation twice in one frame is visible; no reader from another world is selected; nested Array/Dictionary `is_read_only` checks; packed copy mutation test demonstrates duplicated consumer independence. Logs under `docs/art/production9/gates/p1/`.
2. Executable old-path equivalence test: compare all 12 getter payloads with old direct path on frozen W0 and W500; compare wrapper return/state behavior for advance/reset/building/intervention and same-frame invalidation. This makes the named old-path exemption a real test, not a disguised bypass. Record mutation fixture setup, including any test-only world changes; never use those fixtures for identity captures.
3. Existing fake-world tests: run the audit's full definition inventory plus mixed main/fake seams. No stale same-frame data, new parse errors, read-only errors, or hidden fixture changes.
4. W0/W500 consumer runner: instantiate actual main, reach target ticks, enable panels/renderers sufficiently to exercise conditional readers, run 600 frames each, reset after caches have been used to hit the effects-reset alias, and capture stderr. Fail on any script/read-only error. Keep the frame count and worlds explicit; this is not substituted by unit tests.
5. Static source test and all changed-pin Rust binaries pass with original semantic assertions preserved. Full Part 1 gate uses the leader's existing gate runner and concurrency constraints.
6. Causal batch Rust tests cover empty, shared-event dedup, kind filter and ordering. Headless equivalence log proves 600 ticks per kind with no sequence difference, including reset and overflow rearm cases where exercised; if natural 600 ticks do not hit overflow, add targeted fixture coverage rather than claiming it was observed.
7. On current W500 frozen blocks, perf counter frame equals process bracket frame; diagnostic snapshots do not increment or populate counted caches; counted total plus inventoried residual calls supports F4. No perf target claim before the after measurements.
8. Part 2 pixel identity remains a separate required gate after this migration. No source/unit/runtime smoke test here substitutes for RGBA comparisons or 600-frame consumer logs.

Implementation is complete only after integration verifies these outcomes, not merely after the migration regex reports no matches. This handoff itself does not claim any implementation or acceptance test has run.
