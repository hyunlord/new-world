# GOAL: WorldSim — Art Pass 9: frames before features — measure what the screen costs, read the simulation once per frame, and keep every pixel the same

Repository `hyunlord/new-world`. Continue the art line: `codex/art-production-2026-10-wip` = `codex/art-production-2026-10` = `56059e0f9a5cad2edea76bb2a3b12b8559539eb1` (Art Pass 8; draft PR #511, stacked on the simulation PR #510).
This run is executed by **Codex**, unattended, on the Mac. **At the same time a simulation run (Revitalize Pass 16, Claude Code) works on the same Mac** in `/Users/rexxa/work/new-world-pass4` and runs cargo builds, tests and short timed benchmarks; a learned-model run computes on the DGX and touches nothing of yours. Section 1.2 is how the two runs share the Mac.

**Simulation commit to merge:** `da8e45c7858b7973330ebecef8ff35ae5ee936ee` (Pass 15's green head; it contains Pass 14's tools and Pass 15's benchmark, and a debug getter `reset_world_stress(seed, agents)` that builds a crowded `game_v19` world).

**This pass changes nothing the player sees.** It changes how the UI reads the simulation and how often it recomputes, proves with pixel-identical captures that the picture did not change, and measures the frame time before and after on the normal game and on crowded worlds. No new art, no new localization keys, **no image generations**.

---

# ⚠️ 0. AUTONOMY — READ FIRST
1. **Never ask the owner anything.** Decide, implement, record the decision and its reason in `docs/art/production9/DECISIONS.md`.
2. **First action:** save this document unchanged as `docs/art/production9/GOAL.md` (or verify the owner's copy is this document), commit it in Part 0, and re-read the section of the current Part at every Part start and after any context compaction.
3. **Do not stop between Parts.** One task per Part; push to the WIP branch after every Part.
4. **Review loops: two rounds maximum.**
5. **On failure: work around, note it, continue.** A target missed with honest numbers is a valid result; a missing number is not.
6. **Time rule:** record start time and each Part's start and end in `docs/art/production9/TASKS.md`. Past **10 hours**, finish the current step, push, go to Part 5.
7. **Three strikes** per failing target, then keep the best state that still passes the pixel identity (Part 2) and record evidence and hypothesis.
8. **Drop order when time is short:** the n = 1,000 and n = 2,000 sizes (4.2) → the per-node ablation of the after state (Part 4) → Part 3's third node → Part 3's second node. Never drop Parts 0, 1, 2, 4, 5, the baseline ablation, the n = 500 measurement or the pixel identity.
9. **Image budget: 0.** No generation of any kind.
10. **Honesty rules stay:** never mark a failed check as passed, never lower a threshold of section 8, keep rejected evidence, never edit or re-interpret an earlier pass's verdict.
11. **This brief supersedes the repository's session-start instructions**: do not install hooks, do not run the harness pipeline, never push to `lead/main` or `main`, never force-push.

---

# ⚠️ 1. WHERE AND HOW

## 1.1 Places and tools
- Mac only, the clone `/Users/rexxa/work/new-world-art-prod`. Capture evidence to `CAPTURE_EVIDENCE=/Users/rexxa/work/art-production9-evidence`. Earlier evidence folders are read-only.
- **One extra checkout is yours:** a `git worktree` of this clone at `/Users/rexxa/work/art-p9-base`, detached at the Part 0 merge commit, with its own debug `sim-bridge` build — the "before" build for frame times (4.2) and pixel identity (Part 2). A fresh worktree has no imported assets (`.godot/` and `*.import` are ignored): after building its library run `"$GODOT_BIN" --headless --path /Users/rexxa/work/art-p9-base --import` before any capture or measurement there (its cargo build falls under 1.2; check free disk first). Untracked copies of this pass's measurement and capture scripts may be placed in it (they must run there unchanged; 5.1's fallback makes that possible). Remove it at the end (Part 5).
- Never touch `/Users/rexxa/work/new-world-pass4`, `/Users/rexxa/work/new-world-wm1`, `~/github/new-world-wt/lead`, `/Users/rexxa/work/new-world-capture` or any other checkout; never kill a process you did not start; no ComfyUI, no dev servers, no DGX.
- `export CARGO_BUILD_JOBS=6`; cargo tests `-- --test-threads=6`. `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`. Windowed Godot only through `tools/art_pipeline/with_window_lock.sh`; other limits `tools/art_pipeline/run_bounded.sh`.
- Scripts of earlier passes that write under `docs/art/production8/**` are copied to `_p9` versions writing under `docs/art/production9/**`.

## 1.2 Sharing the Mac with the simulation run (both briefs carry this protocol)
- **Quiet lock:** the directory `/Users/rexxa/work/.mac_quiet.lock` (`mkdir` is the atomic acquire). Write `owner`, `pid`, `purpose`, `started` (ISO time) into `info.txt` inside it; remove it when done. Hold it **only while measuring time**: one hold per **measurement session** (one Godot launch measuring one world, 4.2), at most **60 minutes** per hold; quit Godot at the end of the session and release. Leave at least **10 minutes** between two of your holds so the other run can build and test. Never leave a Godot window running while you wait (an idle window still renders and loads the Mac). `with_window_lock.sh` (it waits without a timeout) is acquired **inside** the quiet-lock hold, and time spent waiting on it counts toward the 60 minutes.
- **While the other run holds it:** do not start a cargo build or test, a Godot run, a probe or a gate; wait, polling every 5 minutes. A lock older than 2 hours is stale: remove it, record it in DECISIONS, continue.
- **Before taking it:** wait until no `cargo`, `rustc`, `worldsim_*` or `Godot` process of the other run is running (`pgrep -x` on the names; record the list) and the 1-minute load average (`sysctl -n vm.loadavg`) is < 4; after 60 minutes of waiting, take it anyway and mark the block `contended`.
- Record every hold (start, end, load before and after) in `docs/art/production9/perf/holds.md`.

---

# 2. GIT

## 2.1 Branches and scope
- Start from `codex/art-production-2026-10-wip` = `56059e0f` (must equal origin); local-only commits go to `art-p9-local-backup` first.
- WIP gets every commit; `codex/art-production-2026-10` moves only by fast-forward to a WIP commit whose full gate (Part 5) is green. Never force-push. After every push `git ls-remote` must equal `git rev-parse HEAD`.
- Commit messages `art-p9(pN): <what changed>`; stage explicit paths only.
- **May change:** `scripts/**`, `scenes/**`, `project.godot`, `docs/art/**` (new files only under `docs/art/production9/**`, plus `docs/art/TEST_LEDGER.md`), `tools/art_pipeline/**`, Rust static-source tests under `rust/crates/sim-test/tests/` that read `.gd`/`.tscn` files (re-pinned with the same meaning when a call site moves; one ledger row each under `## Art Pass 9`).
- **Rust exceptions (narrow):** in `rust/crates/sim-bridge/src/ffi/world_node.rs` only — (a) the art-owned getters `get_intervention_state`, `get_society_snapshot`, their collectors and unit tests (keep them compiling and passing); (b) **one new additive art-owned getter** for batched causal events (5.3), its collector and unit tests. Nothing else under `rust/`.
- **Must not change:** everything else under `rust/`, `shaders/**`, `assets/**`, `localization/**`, `docs/revitalize/**`, `docs/worldmodel/**`, `tools/revitalize/**`, `tools/worldmodel/**`, `tools/harness/**`, `.claude/**`, `.harness/**`, evidence under `docs/art/production{,2,…,8}/**`.

## 2.2 Part 0 merge
`git merge --no-ff da8e45c7858b7973330ebecef8ff35ae5ee936ee -m "art-p9(p0): bring in simulation passes 14 and 15"`. Conflicts: under `rust/crates/sim-*/src/**`, `docs/revitalize/**`, `docs/worldmodel/**`, `tools/revitalize/**`, `tools/worldmodel/**` take the simulation's side, keep the art getters; record every resolved file.
**Planner's trial merge** (on a Linux copy): `git merge-tree` of `56059e0f` and `da8e45c7` exits 0 (no textual conflicts; 1,119 paths change, all from the simulation line: Pass 14's tools and probe outputs, Pass 15's Rust and benchmark). The Rust delta is Pass 15's only: `sim-engine/src/lib.rs` (a profiled tick, no new field in an existing struct), `sim-systems/src/worldgen/mod.rs` (`WorldGenConfig::stress_v19`), `sim-bridge/src/ffi/world_node.rs` (`build_stress_engine`, `#[func] reset_world_stress`), `sim-test` (benchmark binary). After the merge `cargo test -p sim-bridge` passes 26/26, `cargo test -p sim-test --no-run` builds, and the 14 static-source test binaries that read any changed Rust file pass. Godot was not run by the planner: **build and run the full gate right after the merge** (4.1 step 2).

---

# 3. WHY THIS PASS (planner's review, with evidence)
1. **The screen already costs more than a frame at today's population.** Art Pass 8's A30 (frozen world, 3 settlements, ~50 villagers, `docs/art/production8/REPORT.md` L65; method in `FRAME_MEASURE_README.md`) measured the median of `process_ms + render_cpu_ms` at **17.10 ms with the village layers off and 18.98 ms with them on** — above the 16.67 ms of 60 FPS before the simulation runs a single tick.
2. **The simulation is not the cost.** Pass 15 (`docs/revitalize/PERF_BUDGET.md` after the merge) measured a tick at 0.08 ms in the natural game and 0.30 ms with 500 people; the Rust side of the per-frame getters it could time costs 0.026 ms per call set (single calls, n = 500; two art-only getters untimed; ×callers 0.285 ms). Those are **release**-build numbers; the editor loads the **debug** library (`sim_bridge.gdextension`: `macos.debug = res://rust/target/debug/libsim_bridge.dylib`), where the Rust side is several times slower — this pass measures what the owner actually runs and records the build profile.
3. **The UI reads the same data again and again.** Pass 15's inventory of this branch (`docs/revitalize/probe/p15-4_scaling/inventory.md`, section B, after the merge) counts, per frame: `get_agent_snapshot` from 9 call sites (+4 more with the inspector open), `get_settlement_snapshot` from about 12 (two of them **once per hearth** in `building_renderer.gd` L136 and L357), `get_time_snapshot` from 15 call sites (13 + 5 per hearth calls per frame; three sites **once per hearth** in `building_renderer.gd` L285/L319/L320), `get_terrain_snapshot` — **the whole map** — every frame in `agent_renderer.gd` L477 (`_activity_context`, L452–), and `get_tile_causal_history` **once per distinct agent tile, twice** (`agent_renderer.gd` L334 `_ingest_memory_recalls`, L386 `_ingest_combat_events`). Every call builds a new Godot `Dictionary` with packed arrays; the conversion, not the Rust collector, is the expensive part, and it grows with the population and the number of hearths.
4. **The next game passes add people and per-person detail** (temperament, family). Pass 15 gave the simulation a budget; the screen needs one too, measured the same honest way, before more is drawn.

---

# 4. PART 0 — Merge, gate, baseline frame times

## 4.1 Merge and gate
1. Backup branch; merge (2.2); record conflicts and resolutions. Record that `reset_world_stress` exists (`world_node.rs`) and what it builds.
2. Full gate (`GATE_OUTPUT_DIR=docs/art/production9/gates/p0 bash tools/art_pipeline/gate.sh p0`) under 1.2's rules (no quiet lock needed for the gate; it is not a timing measurement, but do not run it while the other run holds the lock). Fix fallout in art-owned code and tests (2.1); re-run until green or three strikes.

## 4.2 The measurement (`scripts/test/art/perf_frames_p9.gd`)
- **Reuse only the probe and the per-frame readout:** `frame_cpu_probe.gd` (the −1000 / +1000 process bracket) and `frame_measure_session.gd`'s `Runtime.begin / next_frame / restore`. Its `collect()` is hard-wired to A30 (12 × 50 layer-toggle blocks, three settlements, a frozen world detected through `process_mode`) and must not be used for this matrix; write a new collector.
- **Build:** the editor loads the debug library; measure with it (what the owner plays), the same profile before and after, recorded in every output. No release-library run: the editor always selects `macos.debug`; never copy a release library over `rust/target/debug/`. Measurement sessions run **without** `--fixed-fps` (real frame times); only Part 2's identity captures use it.
- **Worlds:** (W0) the live game: `reset_world(20261010)`, `advance_ticks` to display day 29, 12:00 (tick 40,560); (W1) the society seed of Art Pass 8: seed 20,261,013 advanced to tick 126,994 (first three living settlements; A30's state); (Wn) `reset_world_stress(15100, n)` advanced 1,440 ticks, **n ∈ {200, 500, 1,000, 2,000}**. `reset_world_stress` builds an **unfed** world: record the alive count at the measured tick (Pass 15 saw unfed n = 500 average 475 alive).
- **Framings:** (Z4) the default zoom 4 centred on the first settlement's hearth; (Z1) zoom 1 centred on the same point (MEDIUM/FAR tier, overview layers drawn). Window 1,920 × 1,080, the camera's own processing frozen as A30 did.
- **Session = one Godot launch for one world** (holding the quiet lock, 1.2): reset and advance; then **all frozen blocks first** (`set_sim_speed(0)`; vsync disabled, `Engine.max_fps = 0`: Z4 × 3 blocks, Z1 × 3 blocks), then the **running blocks** (`Engine.max_fps = 60`, vsync disabled: speed 2× — the player's maximum, key 4 → 2.0 in `camera_controller.gd` L78 — × 3 blocks, then speed 4× — the API's clamp — × 3 blocks, Z4 only). Running blocks change the world, so they come last; nothing frozen is measured after a running block.
- **Per block:** discard 30 frames after any state change, then count **300 frames**; per frame `process_ms`, `render_cpu_ms`, `setup_cpu_ms`, GPU ms if available, wall pacing (diagnostic), **ticks run in that frame**. **Primary = `process_ms + render_cpu_ms`** (as A30). Report median, p90, p99, max per block and pooled, frames > 16.67 ms; for running blocks also the same statistics over **tick-carrying frames only**. Never use `Performance.TIME_PROCESS`.
- **Baseline node ablation** (sessions W0 and W500, frozen, Z4, after the frozen blocks and before the running ones): for every node under `Main` with a `_process` (the renderers, controllers and UI panels of `scenes/main.tscn` and the nodes they create at runtime; **not** `WorldSim` or `Camera2D`), save its `visible` and `process_mode`, disable it (`PROCESS_MODE_DISABLED`, `visible = false`) for one 150-frame block, restore both saved values, and run an all-enabled 150-frame block before the next node; **cost** = median(all on, the neighbouring blocks) − median(node off). A node whose disabling takes children with it (`AgentRenderer` → `FoundingGroups`, `SettlementOverviewRenderer` → `Labels`) is reported as that group. Renderers draw into shared layers that have no script (`Main/WorldObjects`, `Main/Shadows`, created by `art_sprite_library.gd` L74–83) and `BuildingRenderer`'s `NightChillLayer` is a `CanvasLayer` its parent's `visible` does not hide: ablate those three layers as rows of their own (`visible = false` only). Also one block with every ablatable node and layer off (the floor). After the ablation, check the zoom tier's visibility state equals the saved one (`ZoomLodController` only re-hides on a tier change).
- **F7's A30 rerun:** A30 itself, before (from the base worktree) and after: `capture_framings.gd --plan art_society` with `CAPTURE_SOCIETY_PERF_ONLY=1` and `--out "$CAPTURE_EVIDENCE/a30-before"` / `a30-after` (the script writes only under `--out`; no `_p9` copy needed; its PERF_ONLY branch needs no measured dates).
- **The harness's own reads** (`frame_measure_session.gd` L35–39 calls three getters every frame, outside the bracket) are excluded from F4's counts: they go through `SimReads`' non-counting path (`SimReads.peek_uncounted(...)`) after Part 1, or reset the counters at the begin stamp; state which. Write `perf_frames_p9.gd` so the same file runs on the base build (no `SimReads`) and the current one (5.1's fallback).
- Outputs: `docs/art/production9/perf/baseline.json` (every sample), `baseline.md` (tables), machine facts (`sysctl -n machdep.cpu.brand_string hw.ncpu hw.memsize`, macOS and Godot versions, build profile), load before and after each session.
- Commit, push.

---

# 5. PART 1 — Read once (`scripts/ui/sim_reads.gd`)

## 5.1 The reader
- **Mock worlds and the fallback:** 19 test scripts give readers a fake world (e.g. `test_hud_old_age.gd` L26–28; `harness_2_5c.gd` L77 sets `panel._world_sim = sim`), change its data and call `_process` / `_ingest_snapshots` several times in one engine frame. So: every reader resolves `../SimReads` (or the main scene's `SimReads`) and, when none exists, reads its world directly as today (a pass-through); `SimReads` calls its world through `call()` (no `WorldSimNode` typing) and **memoizes only when the world `is WorldSimNode`** — a fake world is always passed through. The mock-based tests keep their meaning unchanged.
- A new node script `class_name SimReads` instanced in `scenes/main.tscn` as `Main/SimReads`, **directly after `WorldSim`** (so it is ready and connected before any reader's `_ready` and before readers connect to `world_reset`), resolving `WorldSim` lazily. No `process_priority` change is needed: `WorldSim` is already the first child at priority 0 and no other non-test node has a negative priority, so no reader sees data from before the frame's tick today; state this, and keep it so.
- **Memoized per frame:** `agent_snapshot()`, `settlement_snapshot()`, `construction_snapshot()`, `building_snapshot()`, `resource_snapshot()`, `time_snapshot()`, `tallies()`, `recent_deaths()`, `society_snapshot()`, `intervention_state()`, `world_info()`: the first call in a frame (`Engine.get_process_frames()`) calls the `WorldSim` getter, later calls in that frame return the same object.
- **Memoized per world:** `terrain_snapshot()` — read on first use and again after `world_reset` only. Prove first that terrain and elevation cannot change after worldgen (the planner and the reviewer found writes to `terrain_type` and `elevation` only in `rust/crates/sim-systems/src/worldgen/terrain.rs`); confirm by grepping every crate for both fields and `TileGrid` mutators and record the result. If any runtime write exists, memoize per tick instead.
- **Invalidation:** `invalidate()` clears every per-frame entry. Every mutating `WorldSimNode` call — `apply_intervention`, `advance_ticks`, `reset_world`, `reset_world_stress`, `on_building_placed` — goes through `SimReads` wrappers that call the node and then `invalidate()`, **in game scripts and in test, capture and measurement scripts alike** (in-game today only `god_hand_panel.gd` L110 mutates; but `capture_framings.gd` L390 loops `advance_ticks(1)` and reads snapshots in the same frame — with memoization it would read stale data unless it goes through the wrapper). `SimReads` also invalidates on `world_reset`.
- **Shared objects must not be mutated:** the cache calls `make_read_only()` on every returned `Dictionary` and on every `Array` inside it. That is shallow: **packed arrays are shared by reference** in Godot 4 and cannot be made read-only, so `var xs: PackedInt32Array = snap["xs"]; xs.append(…)` (or `xs[i] = …`, `xs.sort()`) silently changes the cached copy for later readers. First confirm the sharing with a five-line headless script in this Godot version and record the result. Then audit every reader of a memoized getter for in-place mutation of the returned dictionary, its arrays **and its packed arrays**; where a reader mutates, it takes a `duplicate()` first (known case: `hud_status_panel.gd` L116–117 writes a `settlement_ids` key). A test runs every renderer and panel for 600 frames on W0 and W500 and asserts no script error and no `read-only state` error in the captured log (`OS.add_logger` if available in this Godot version, else run headless and grep stderr).
- **Counting:** `SimReads` counts FFI calls per frame per getter and their wall time (`Time.get_ticks_usec`), readable by the measurement script.

## 5.2 Migration
Replace **every** call of a memoized getter under `scripts/` — per-frame, per-tick-change, conditional and per-hearth loops alike, and in tests, captures and measurement scripts — with `SimReads`. The per-hearth loops read the snapshot once before the loop. `get_agent_detail` (inspector, one agent) and `get_influence_overlay` (overlay on) stay direct. A static test (`rust/crates/sim-test/tests/harness_art_p9_sim_reads.rs`, reading `.gd` files) covers every game script under `scripts/` (not `scripts/test/**`) and every test, capture or measurement script that loads `main.tscn`; it ignores `func get_…(` definitions (fakes) and mock-based unit tests, and fails if a covered script other than `sim_reads.gd` calls a memoized getter or a mutating method directly, in any of the three forms used today: typed (`_world_sim.get_time_snapshot()`), `.call("get_settlement_snapshot")`, and string dispatch through a helper (`_snapshot("get_tallies")`, `hud_status_panel.gd` L306–309, `hud_topbar.gd` L219–222); `has_method("get_…")` guards are not calls. The only allowed direct calls are inside old-path equivalence tests listed by name in the static test. Re-pin with the same meaning the existing static asserts that pin moved lines (about 13 asserts in 8 files pin the hearth line with `_world_sim.get_time_snapshot()`, e.g. `harness_p12_beta_terrain.rs` L192), one ledger row each.

## 5.3 Causal events in one call
`agent_renderer.gd` scans every distinct agent tile twice per frame with `get_tile_causal_history(x, y)` (each tile's history is a ring buffer of 8 events, oldest first) to find new `memory_recalled` and combat events. Replace both scans with **one call per tick change**: a new art-owned getter in `world_node.rs`, `#[func] fn get_tile_causal_events(xs: PackedInt32Array, ys: PackedInt32Array, kinds: PackedStringArray) -> Array` returning, for the given tiles, every event whose `kind` is in `kinds`, each once (deduplicated by event id), in the order the per-tile path would see them (tiles in input order, events oldest first), with Rust unit tests (empty input; two tiles sharing an event; kinds filter; order). The renderer calls it only when the simulation tick changed since its last call, **and also on the frame after it clears an overflowing seen-set** (the old code clears it above 512 entries, L350–351 and L402–403, and the next scan re-arms recalls still in the rings — keep that), and it resets its last-tick marker on `world_reset`. Cue timers keep counting down per frame (`_clear_expired_*`, L353–364, L405–416). The renderer passes the deduplicated agent tiles in snapshot row order and splits the result by kind (recalls first, then combat — the old order: all recalls in tile order, then all combats). **Equivalence test** (`scripts/test/art/test_causal_batch_p9.gd`, headless, driven by `advance_ticks(1)` per frame through `SimReads`, not by wall-clock speed): on W500 for 600 ticks, **per kind**, the sequence of (tick, event id) the renderer ingests through the new path equals the old per-tile path's (run the old function inside the test).

Commit, push.

---

# 6. PART 2 — Pixel identity (the proof that nothing visible changed)
- **Determinism of a capture:** several visuals advance with frame `delta` (`building_renderer.gd` L60 `_elapsed += delta` → the hearth flame frame in `wood_visuals.gd` L36; `agent_renderer.gd` L75; `world_effects.gd` L81 with its spawn schedule L141–153; `daylight_controller.gd` L36; `god_hand_effects.gd` L78; the god's-hand toast `god_hand_panel.gd` L86). Therefore launch every identity capture with **`--fixed-fps 60`** (Godot then passes a constant delta), drive everything by frame counts (no wall-clock waits), and capture exactly 120 frames after the last state change.
- **Capture script:** a new `scripts/test/art/capture_identity_p9.gd` (runs unchanged in the base worktree as an untracked copy, using 5.1's fallback) produces P1, P2, P4, P5, P6; P3 comes from `capture_framings.gd` with its FV row (`capture_plans.gd` L16, `[33, "09:49", "FV", "all"]`) fed through `CAPTURE_PLAN_JSON`. Every capture of a framing happens at the **same `Engine.get_process_frames()`** value in A, A′ and B (`building_renderer.gd`'s `_elapsed` accumulates from launch and is never reset), with the same sequence of resets and advances before it.
- **Framings** (all frozen, `set_sim_speed(0)`): (P1) W0 Z4 at day 29 12:00 (tick 40,560); (P2) W0 Z4 at day 23, 02:00 (tick 31,320; the verified lit night, `docs/art/production8/MEASURED_DAYS.md` L53); (P3) W1 in Art Pass 8's FV framing (`capture_framings.gd`'s all-villages framing) at tick 46,189 (`MEASURED_DAYS.md` L246); (P4) W1 Z1 at tick 126,994 (overview layers on); (P5) W500 Z4 after 1,440 ticks; (P6) W0 Z4 at tick 40,560 with the inspector open on the lowest-id agent and the god's-hand panel open.
- **Control first:** capture all six with the **base build** (`/Users/rexxa/work/art-p9-base`) **twice** (A and A′, separate launches). Then with the current build (B). **Pass rule per framing:** if A = A′ exactly (RGBA), B must equal A exactly; if A ≠ A′, B passes when its difference from A is no larger than A′'s (count of differing pixels and maximum channel difference), and the framing is reported as "not bit-reproducible" with the cause. `tools/art_pipeline/pixel_identity_p9.py` computes both and writes diff images.
- Rerun B after every later change that touches a renderer or panel (Part 3). Commit the comparison table and downscaled previews under `docs/art/production9/identity/` (full-size in the evidence folder). A failing framing is a bug in Parts 1/3: find and fix it; three strikes, then revert the offending change.

---

# 7. PART 3 — The three biggest remaining costs
After Part 1, rerun the node ablation (W500 frozen Z4) and take the **three ablation rows with the largest remaining cost**. For each, find per-frame work whose inputs did not change and skip it: e.g. rebuild a `MultiMesh` or `queue_redraw` only when the tick, the camera transform or the zoom tier changed; recompute derived dictionaries once per tick, not once per frame; do not process a node whose layer is hidden. Each change: the pixel identity of Part 2 still passes, and its own before/after cost from a 150-frame alternating pair is recorded. No change to what is drawn, where, or in what colour.

Commit, push after each row.

---

# 8. PART 4 — After: the same measurement, and the verdicts
Rerun 4.2 exactly (all worlds, framings, sessions, blocks, the ablation, the A30 rerun) on the final build. Then:

| # | question | rule |
|---|---|---|
| F1 | the live game, frozen, Z4 (W0) | PASS if the median primary ≤ **8.0 ms** (half a frame) |
| F2 | 500 people, frozen, Z4 (W500) | PASS if the median primary ≤ **12.0 ms** and the p99 ≤ 16.67 ms |
| F3 | 500 people, running at 2× (the player's maximum), Z4, `max_fps` 60 | PASS if the median primary over **tick-carrying frames** ≤ **16.67 ms**; 4× reported likewise (no verdict) |
| F4 | FFI calls per frame (W500 frozen Z4) | PASS if the calls `SimReads` counts plus the remaining direct per-frame call sites of the static inventory (`get_agent_detail`, `get_influence_overlay` when their panel/overlay is open — closed in this framing, so 0 — plus at most one `get_tile_causal_events` per frame) total ≤ **20**, and none scales with agents, tiles or hearths |
| F5 | pixel identity (Part 2, final build) | PASS if every framing passes Part 2's rule |
| F6 | headroom | the largest n in {200, 500, 1,000, 2,000} meeting F2's rule (no verdict) |
| F7 | A30 | A30's own measurement (4.2) before → after, layers on and off (no verdict) |

Report every number before and after side by side with the speed-up factor, the ablation table before and after, and the build profile. Commit, push.

---

# 9. PART 5 — Review, gate, report
1. Independent review (fresh reviewer that has not seen sections 3–7): the merge; that no memoized getter or mutating method is called directly outside `sim_reads.gd` (run the static test); the invalidation rule in game and test scripts; read-only data and the caller audit including packed arrays; the terrain-is-static proof; the batched causal getter and its equivalence test; the pixel identity (open two pairs of captures and the comparison output); one block's median recomputed by hand from `baseline.json` and from the after file; that the F verdicts follow section 8; the allowed paths; the quiet-lock holds. Findings and dispositions in `docs/art/production9/REVIEW.md`.
2. Full gate (`GATE_OUTPUT_DIR=docs/art/production9/gates/final bash tools/art_pipeline/gate.sh final`); fast-forward `codex/art-production-2026-10` if green; add an Art Pass 9 section to PR #511's description.
3. Remove `/Users/rexxa/work/art-p9-base` (`git worktree remove`).
4. `docs/art/production9/REPORT.md` in Korean (section 11 format) and the same text in the final message.

---

# 10. 인게임 확인사항 (오너용)
- 게임 화면은 이전과 **똑같아야** 한다. 같은 시드, 같은 시간에 마을·불·사람·밤 표현·패널이 그대로인지 확인한다(이 패스의 목적은 같은 그림을 더 싸게 그리는 것).
- 최고 속도(숫자 키 4 = 2배속)에서 화면이 이전보다 부드러운지, 특히 마을이 셋으로 갈라진 뒤에도 끊김이 줄었는지.
- 콘솔에 새 에러(특히 "read-only" 관련)가 없는지.
- 마우스로 주민을 클릭해 인스펙터를 열고, 신의 손 패널로 개입을 한 번 써 본 뒤에도 화면이 즉시 갱신되는지(무효화 규칙 확인).

---

# 11. 구현 완료 보고 (REPORT.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
무엇을 만들었는가? (측정 스크립트, SimReads, 호출 이전, 일괄 인과 이벤트 게터, 비용 상위 노드 정리)

### 구현 방법
측정 방법(프레임 괄호, 렌더 CPU, 동결/실행, 블록·버림), 프레임 메모이제이션과 무효화, 읽기 전용 공유와 호출자 감사, 지형 불변 증명, 픽셀 동일성 방법

### 기능 설명
플레이어가 보는 것은 그대로이고 무엇이 얼마나 싸졌는가?

### 변경된 파일 목록
(커밋별)

### 측정 결과
F1~F7 표(전후·배수), 세계·화면·조건별 중앙값/p90/p99/최대, 노드별 비용 표(전후), 프레임당 FFI 호출 수(게터별, 전후), 16.67 ms 초과 프레임 수, A30 조건 전후

### 환경
맥 사양, Godot 버전, 블록별 부하와 contended 여부, 조용함 잠금 기록(다른 실행과의 대기 포함)

### 독립 리뷰
지적 사항과 처리 결과

### 자율 결정 목록
DECISIONS.md 요약

### 실행 기록
Part별 시작·종료 시각과 소요 시간, 대기 구간

### 확인된 제한사항

### 게이트 결과
(단계별 결과, 테스트 수치, 허용 경로 밖 변경 없음)

### 정직한 판단
지금 화면은 몇 명까지 60 FPS로 그릴 수 있는가? 다음으로 비싼 것은 무엇인가?
```
Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the PR #511 update, and the time spent per Part.