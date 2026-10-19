# GOAL: WorldSim — Art Pass 12: a frame budget the live game can keep while running, lives you can read without mistakes, crowds you can tell apart, and villages that visibly help and quarrel (porters, relations)

Repository `hyunlord/new-world`. Continue the art line: `codex/art-production-2026-10-wip` = `codex/art-production-2026-10` = `b83dc5aef50505217c9fc3cdf33dee87b4175a46` (Art Pass 11, TIMEBOXED_PARTIAL; draft PR #511).
This run is executed by **Codex**, unattended, on the Mac, in `/Users/rexxa/work/new-world-art-prod`. **The owner launches it only after the simulation's Pass 20 has finished** (section 1.2). No DGX.

**Simulation to merge:** the head of `codex/revitalize-2026-10` at launch: `6e1446b4d6af41e13e2b221b17609751510f046f`, Pass 20's green fast-forward. A dry `git merge-tree` against `b83dc5ae` was clean on 2026-10-18. It brings `game_v25` = `game_v24` + a `RelationsRule`: **porters** carry food or wood between villages (kin aid and exchange) and come home; foragers of two villages **dispute** bushes, some disputes become **quarrels**, a low relation makes villages **estranged** until they **reconcile**. New chronicle kinds 17–21 and two bridge getters. The live game becomes `game_v25`.

Five Parts of work, in this order: (0) merge, contract and a slim release baseline that **finally breaks down the running frame**; (1) **the frame budget**: a structural fix of the per-agent loop, measured in release, with exact identity; (2) **reading without mistakes**: the life text, crowds, a real-input test and the activity blind test redone with crops that contain the icon; (3) **villages that help and quarrel on screen**: porters, relation lines, chronicle sentences 17–21; (4) after-measurements and verdicts; (5) review, gate, report.

---

# ⚠️ 0. AUTONOMY — READ FIRST
1. **Never ask the owner anything.** Decisions in `docs/art/production12/DECISIONS.md`.
2. **First action:** save this document unchanged as `docs/art/production12/GOAL.md`, commit it in Part 0, re-read the current Part's section at every Part start and after any compaction.
3. **Parts:** 0 = section 4; 1 = section 5; 2 = section 6; 3 = section 7; 4 = section 8; 5 = section 9. Push after every Part.
4. **Review loops: two rounds maximum** (fresh reviewer that has not seen sections 3–7).
5. **On failure: work around, note it, continue.** Three strikes per failing target.
6. **Time rule** (record every start and end in `docs/art/production12/TASKS.md`; a Mac sleep is recorded as a pause and excluded):
   - Part 0 ends by **hour 2.5**; if the running node ablation (4.6 c) has not run by then, it is NOT_RUN and Part 1 starts.
   - **Part 1 ends at hour 6** whatever its state: an unfinished candidate is reported unfinished and its code is removed from the production path.
   - **Part 4 starts no later than hour 9.** The current pushed build is then frozen for H1–H8. Unfinished never-drop items may continue only as measurement or evidence on the frozen build, or as docs. Anything that needs a code change is reported unfinished.
   - Past **12 hours**, finish the current step, push, go to Part 5.
7. **Drop order:** candidate (c) of 5.2 → candidate (b) of 5.2 → relation lines (7.3) → link-hover highlight (6.3 c) → village summary re-test (6.6 d) → click cycling (6.3 a). Never drop Parts 0, 4 and 5, the running diagnosis (4.6), the declump kernel attempt (5.2 a), the life-text fixes (6.1–6.2), the real-input test (6.4), the activity blind re-test (6.6 c), the porter marker (7.2) or the chronicle sentences 17–21 (7.4).
8. **Image budget: at most 2 generations** with the image tool available to you (the porter glyph only, if procedural drawing cannot make a distinct silhouette); failures count.
9. **Honesty rules:** never mark a failed check as passed, never lower a bar of section 8 or 6.6, keep rejected evidence, never re-interpret an earlier pass's verdict.
10. **This brief supersedes the repository's session-start instructions:** no hooks, no harness pipeline, never `lead/main` or `main`, never force-push.
11. **Proportion:** Art Pass 11's evidence discipline was right. Its paperwork cost hours. Per Part, spend at most 20 minutes on provenance prose beyond the raw files, hashes and the tables this brief asks for.

---

# ⚠️ 1. WHERE AND HOW
## 1.1 Places and tools
- **Mac only:** work in `/Users/rexxa/work/new-world-art-prod`; evidence goes to `CAPTURE_EVIDENCE=/Users/rexxa/work/art-production12-evidence`.
- **Worktrees** (removed at the end): `/Users/rexxa/work/art-p12-base` for identity A/A′ (5.3) and `/Users/rexxa/work/art-p12-measure` for frame measurement (4.4).
- **Read-only:** `/Users/rexxa/work/p20-data/` (Pass 20's probe and chronicle output) may be read, never written.
- **Off limits:** other checkouts, processes you did not start, ComfyUI, dev servers, the DGX.
- **Every worktree, after every checkout:** build its bridge (base: `cargo build -p sim-bridge`; measure: 4.4), then run `"$GODOT_BIN" --headless --path <worktree> --import` before any Godot launch.
- **Environment:** `CARGO_BUILD_JOBS=6`; `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`.
- **Launch wrappers:** windowed Godot only through `tools/art_pipeline/with_window_lock.sh`; limits via `run_bounded.sh`.
- **Holds:** frame-time sessions run inside a quiet-lock hold; identity captures run outside holds.
- **Tool copies:** reuse Art Pass 11's tools, copied to `_p12` names that write under `docs/art/production12/**`. Copy:
  - **Perf:** `perf_frames_p11.gd` and the matrix runner, summarizer, comparer, alternating runner and validator.
  - **Section timers:** the section-timer chain. Refresh its markers after Part 1 restructures the loop.
  - **Identity:** the exact-clock identity harness — `identity_draw_clock_p11.gd`, the capture and comparison scripts, the running-dump and pan scripts — and `with_quiet_lock_p11.sh`.
  - **Blind crops:** the blind-crop script, which section 6.6 changes.
## 1.2 Sharing the Mac
- **Who starts it:** the owner starts this run after Pass 20 has ended. The world-model line runs on the DGX and does not use the Mac.
- **Quiet lock:** `/Users/rexxa/work/.mac_quiet.lock`, as in Art Pass 11, with every hold and wait logged in `docs/art/production12/perf/holds.md`. While another run holds it: no cargo, no Godot, no gate. During your own holds you may edit and commit.
- **New rule — contention is measured, not assumed:** a measured session or alternating pair is **valid** only if both are true:
  - the 1-minute load is < 6.0 at its start and at its end;
  - no process you did not start uses > 50 % CPU in a `ps -Ao pid,pcpu,comm -r | head -15` snapshot taken at its start and at its end. Store both snapshots.
- **An invalid session is not a result:** rerun it in a later hold, at most three times; never invalid three times → that item is **NOT_VERIFIED (contended)**, never REJECTED or FAIL. Art Pass 11 rejected its only candidate in a hold at load 111 → 51; Art Pass 12 does not repeat that.
- **Your own heavy processes:** during a hold, nothing you started besides the measured Godot runs. That means no parallel identity captures, no cargo, no reviewer sessions that launch Godot. Art Pass 11's load-111 hold had no recorded cause; record the cause if it happens again.
- **Editing during holds is allowed.** While Part 1 waits on holds, you may write Part 2–3 code on a local side branch `art-p12-ui`, without running Godot or cargo during a hold. Merge it into WIP only after Part 1 ends, so every Part 1 candidate is judged on Part 0 plus accepted candidates alone.

---

# 2. GIT
## 2.1 Branches and scope
- **Commits:** start from `b83dc5ae` (must equal origin); backup branch `art-p12-local-backup`. WIP gets every commit. `codex/art-production-2026-10` moves only by fast-forward to a WIP commit whose full gate is green. Run `git ls-remote` after every push. Messages `art-p12(pN): …`. Stage explicit paths only. Leave the local `.gitignore` / `.ignore` edits uncommitted.
- **May change:**
  - `scripts/**`, `scenes/**`, `shaders/**`, `assets/**` (the porter glyph only), `project.godot`;
  - `localization/**` (the keys of section 10 only);
  - `docs/art/production12/**`, `docs/art/TEST_LEDGER.md`, `tools/art_pipeline/**`;
  - Rust static-source tests under `rust/crates/sim-test/tests/` that read `.gd`/`.tscn` files (re-pinned with the same meaning; one ledger row each under `## Art Pass 12`).
- **Rust exceptions (art-owned):**
  - **in `rust/crates/sim-bridge/src/ffi/world_node.rs`:** the existing art getters (`get_intervention_state`, `get_society_snapshot`, `get_tile_causal_events`) and one new `#[func] art_declump` wrapper (5.2 a);
  - **one new file** `rust/crates/sim-bridge/src/ffi/art_kernels.rs` (pure functions, no `hecs`, no simulation state);
  - **one line** `pub mod art_kernels;` in `ffi/mod.rs`;
  - **test fixtures** under `rust/crates/sim-bridge/tests/fixtures/art_p12/`.
  - **Simulation-owned — call, never edit:** the chronicle, relation and porter getters.
  - Nothing else under `rust/`; no change to `sim_bridge.gdextension` in any committed tree.
- **Must not change:** everything else under `rust/`, `docs/revitalize/**`, `docs/worldmodel/**`, `tools/revitalize/**`, `tools/worldmodel/**`, `tools/harness/**`, `.claude/**`, `.harness/**`, and the evidence of earlier passes.
## 2.2 Part 0 merge and the relations contract
1. **Merge.**
   - `git fetch origin`. Record `git log -1 origin/codex/revitalize-2026-10` and the `git merge-tree` of the two heads.
   - Run `git merge --no-ff origin/codex/revitalize-2026-10 -m "art-p12(p0): bring in simulation pass 20"` and record the merged SHA.
   - **Conflicts:** simulation-owned paths take the simulation's side; keep the art getters; record every resolved file.
2. **Planned contract** (Pass 20 brief, sections 6.7–6.8):
   - **`get_settlement_relations()`** → packed `{a, b, value, estranged, base}` in key order (lower id first).
   - **`get_porter_snapshot()`** → packed `{ids (entity bits), from, to, kind (0 food / 1 wood), amount, stage (0 out / 1 back)}` by AgentId. It is first in Pass 20's drop order and may be absent.
   - **Chronicle kinds 17–21** (subject / other; settlement / settlement2; cause; value):

     | # | kind | subject / other | settlement / settlement2 | cause | value |
     |---|---|---|---|---|---|
     | 17 | `PorterDeparted` | porter / – | sender / receiver | 0 aid food, 1 aid wood, 2 exchange food, 3 exchange wood | amount carried |
     | 18 | `PorterDelivered` | porter / – | **receiver / sender** | as 17, or 10 + the departure cause for a failed trip | delivered amount; 0 when failed, written at the turn-back |
     | 19 | `Quarrel` | forager a / forager b | a's village / b's village | 0 | relation after |
     | 20 | `VillagesEstranged` | – | lower id / higher id | 0 | relation |
     | 21 | `VillagesReconciled` | – | lower id / higher id | 0 | relation |

   - Porters appear in the agent snapshot as Seeking with an Agent target (themselves).
   - **Delivered as planned** (Pass 20 report): pure collectors are in `rust/crates/sim-bridge/src/ffi/relations.rs`, with `#[func]`s in `world_node.rs`. **`get_porter_snapshot().amount` is the amount at departure and stays unchanged on the way back**, so the load the porter actually carries is the agent snapshot's carried food / wood.
3. **Write `docs/art/production12/RELATIONS_CONTRACT.md`:** each getter present or not, keys, types, numbering, and the delivered `docs/revitalize/RELATIONS.md` rules that the visuals depend on. Section 7 is written for the planned contract; **follow the delivered one** and record every difference.
4. **Fallbacks:**
   - **No `game_v25` on the merged head:** Part 3 is NOT_RUN except 7.4's unit tests with a fake bridge; W-multi (4.5) uses `game_v24`.
   - **Porter snapshot absent:** 7.2 runs on unit tests only; its in-game capture is NOT_RUN.

---

# 3. WHY THIS PASS (planner's review at `b83dc5ae`; verify each fact)
1. **The live game misses frames while running, and the stress worlds are far off.** Release numbers, `docs/art/production11/perf/after.md`, primary = process + render CPU:

   | world | people | frozen Z4 median | running 2× Z4 |
   |---|---|---|---|
   | W0 | 29 | 3.37 ms | 5.56 ms |
   | **W1, the live seed at day ≈ 88** | 66 | 6.78 ms | **median 12.03, p90 16.93, p99 19.34 ms** |
   | W200 | 200 | 16.32 ms | 26.52 ms, 2 ticks per frame |
   | W500 | 500 | 28.93 ms | **82.28 ms**: 5 ticks per frame (the bridge's `MAX_ITERS_PER_FRAME`, `world_node.rs` L173), population 500 → 392 per block (flagged) |

   - **W1:** 98 of 900 running frames exceed 16.67 ms.
   - **Running frames were never broken down:** sections and the node ablation exist only for frozen frames.
   - **The simulation is not the cause:** a tick costs 0.39 ms at 500 people (`docs/revitalize/SCALE.md`).
2. **Where frozen time goes** (W500, `perf/baseline-sections.md`): `total` 24.9 ms of 28.9. By section:
   - declump 6.68, pose 2.55, post-loop 2.11, frame choice 1.99, walker offsets 1.89;
   - children 1.31, meta 1.26, look 1.23, variant 1.00, easing 0.91;
   - footprints 0.68, seats 0.66, tile contexts 0.65.
3. **The per-agent loop runs in full every frame.**
   - `agent_renderer.gd` `_process` (L102–282) recomputes every agent's look, pose, frame choice, ≈ 25 `set_meta` calls (L235–263) and appearance, even when the world is frozen and nothing changed.
   - Z1 costs what Z4 costs (W500 28.76 vs 28.93 ms).
   - Art Pass 11's reader inventory (`perf/tick-cache-diagnosis.md`) lists who reads those outputs: footprints, seats, declump, `world_effects.gd`, `shelter_cold.gd`, `morning_chill.gd`, the inspector portrait.
4. **Declump is a pure function.**
   - `CampSeats.declump` (`camp_seats.gd` L161–213) and its helpers (`_walking` L128, `_covered` L215, `_declump_allowed` L226, `_spatial_cell_size` L299, `SpatialIndex` L306–365) read only their arguments.
   - It has one production call (`agent_renderer.gd` L671).
   - Art Pass 11's input memo was "rejected" at 0.983× in a contended hold: no result either way.
5. **Identity can be proven exactly now.** The exact-clock harness reproduces all six framings A/A′ with zero difference, P5 included (`perf/identity-reference/reference-validation.md`).
6. **The activity blind test measured bodies, not icons** (`production11/DECISIONS.md` L8).
   - The crop was body-centred with side = ceil(max(w, h) + 16), which leaves 8 px of top padding. The icon sits at `ICON_OFFSET_PX` (0, −30), 16 × 16 (`agent_renderer.gd` L17–18).
   - Readings 0/12 and 1/12 say poses alone do not tell activities apart. Nobody blind-read the icons.
   - Village summary: 2/6, then 5/6. QA called its symbols tiny (`village_activity_summary_p11.gd` L5–9: Galmuri9, 12 px, height 18).
7. **The life text: 44/45, and visible defects** (`production11/life/reviewer-comparison.md`, `reviews/qa-round1.md` L11–12):
   - **The miss:** 오후티 read as 오후디 (P3-S07).
   - **Korean lines split names into single syllables** (`이키`/`무`).
   - **English ellipsis:** a deceased parent's status is cut with an ellipsis (`agent_inspector_panel.gd` L394, `OVERRUN_TRIM_ELLIPSIS`).
   - **Clipping:** text is clipped at the scroll edges.
   - **Korean stockpile labels on the English screen** (`식량 174 장작 20/39`, from `building_renderer.gd` L260–265, which builds them through `Locale` — find why they stay Korean).
   - **Departures untested:** days 60–90 had no Departed entry, so the five departure causes were never checked.
   - **No real-input test:** no test used real input (clicks, links, wheel, Escape).
8. **The UI font:** `assets/ui/worldsim_theme.tres` L20–23 draws Galmuri11 (a pixel font) at 24 px.
9. **Crowds:** QA (`qa-round1.md` L13) reports that in seed 20261018 at day 60 10:00, "bodies, shelters and nearby icons overlap heavily".
   - **Clicks:** a click selects the front-most opaque sprite (`agent_renderer.gd` `drawn_entity_at`, L716–739; `world_renderer.gd` L122–164), so a person behind another cannot be selected.
   - **The selection ring:** a 1 px `Line2D` (L741–747).
10. **What Pass 20 adds and nothing draws yet:**
    - Porters walk 22–28 tiles between villages, departing at 08:00 and arriving by 15:00.
    - Relations, quarrels, estrangement.
    - Kinds 17–21 would currently fall through `chronicle_text.gd`'s sentence switch.
    - The chronicle panel's `NOTABLE` list (`chronicle_panel.gd` L4) has no new kinds.
11. **What `game_v25` actually does** (Pass 20 evaluation, seeds 19400–19431, 120 days):
    - **Porters:** 9 of the 19 multi-village worlds had deliveries (38 trips, 71 porters, all delivered, none turned back). Exchange never happens.
    - **Conflict:** 511 quarrels and 14 estrangements, no reconciliation. 14 of 32 pairs are estranged at day 120, most near −1,000.
    - **So in `game_v25` worlds most relation lines (7.3) will be red or orange.** That is the data, not a rendering bug. Pass 21 (`game_v26`) rebalances the rule after this pass and keeps the same getters.
    - **Useful worlds:** 19400 has deliveries (the lowest W-multi candidate); 19414 has two estranged pairs.

---

# 4. PART 0 — Merge, gate, slim release baseline, running diagnosis
4.1 **Merge and gate.**
- Backup branch; merge and contract (2.2).
- Full gate: `GATE_OUTPUT_DIR=docs/art/production12/gates/p0 bash tools/art_pipeline/gate.sh p0`, with Art Pass 11's `CARGO_PROFILE_TEST_OPT_LEVEL=2`.
- Fix fallout in art-owned code and tests; three strikes.
- Record that the live game builds `game_v25`.
- Commit the `_p12` tool copies as the **Part 0 tooling commit**: merge plus tools, no UI change.

4.2 **Collector.** The `_p12` collector keeps Art Pass 11's checks:
- `build_profile` matches the loaded dylib's sha;
- fresh reset per running block;
- population recorded at each block's start and end; flagged below 0.9×;
- per-frame SimReads FFI counters.

It adds the 1.2 validity record (load and `ps` snapshots) per session.

4.3 **Identity reference:**
- Point `art-p12-base` at the Part 0 tooling commit and build and import it (1.1).
- Re-derive the six framings for `game_v25` as Art Pass 11 did for `game_v24`.
- Capture A and A′ with the exact-clock harness. They must match exactly, P5 included.
- Record the running-dump (W0 and W200, 600 frames) and pan (W500, frozen and 2×, 240 frames) references.

4.4 **Release measurement worktree** `/Users/rexxa/work/art-p12-measure`, as in Art Pass 11 4.4:
- `cargo build --release -p sim-bridge`.
- Copy the release dylib over the debug path in that worktree only.
- Check the sha per session.
- Never run a debug cargo command or a gate there.

4.5 **Worlds.**
- **W0, W1, W200, W500:** as in `perf_frames_p11.gd` L9. W0 and W1 now build `game_v25`; record their populations.
- **W-multi (new):** the lowest seed in 19400–19431 whose `game_v25` chronicle in `p20-data` (or `docs/revitalize/probe/p20-2_eval/`) has a delivered `PorterDelivered` (cause < 10) before day 90.
  - Start tick = 10:00 of the day of that trip's `PorterDeparted`, so porters are on the road.
  - Minute of day = (tick + `clock_offset_ticks`) mod `ticks_per_day`, both from `get_world_info`.
  - If no seed qualifies, use the lowest seed with ≥ 2 settlements alive at day 60, at day 60 10:00, and record it.
  - Reached by `reset_world(seed)` and `advance_ticks`.
- Record every world's seed, tick, population and settlement count in `docs/art/production12/perf/worlds.md`.

4.6 **Slim baseline** (release, measure worktree, Part 0 tooling commit, valid sessions only):
- **(a) Frozen and running blocks:** for W0, W1, W200, W500 and W-multi, frozen Z4 three 300-frame blocks, and running 2× Z4 three 300-frame blocks, each after a fresh reset.
- **(b) Section timers, W500 frozen Z4.**
- **(c) Running diagnosis (never dropped, except c's ablation per 0.6):**
  - section timers on **W200 running 2×** (300 frames);
  - the SimReads FFI counters (calls and ms per getter per frame);
  - a **running node ablation** on W200 2×: each top-level node under `Main` (renderers and UI panels) disabled one at a time, 150 frames each, with the all-on and restored controls of Art Pass 11.
- **Output:** `docs/art/production12/perf/baseline.md` and `running-diagnosis.md`. The latter names, largest first, every cost ≥ 1 ms per running frame and whether it lies inside or outside `AgentRenderer`. Commit, push.

---

# 5. PART 1 — The frame budget (release measurement; identity reference = 4.3)
5.1 **Order of work.**
- Candidate (a) first: never dropped, and declump is the largest measured section.
- Then (b), then (c), each only if time remains before hour 6.
- Candidates stack: each is judged against the last accepted build.

5.2 **Candidates.**
- **a. Declump in Rust.**
  - **The kernel:** `art_kernels.rs` gets a pure `declump_offsets(...)` that reproduces `CampSeats.declump` and every helper it calls (3.4) **bit for bit**, both spatial-hash and brute-force modes.
  - **Numeric semantics:** GDScript `float` is f64. `Vector2` and `Rect2` components are f32 in the official single-precision build; check `OS.has_feature("double")` is false and record it.
    - Do every `Vector2`/`Rect2` operation in f32, in the same order as the GDScript, and every `float` operation in f64.
    - Convert at the same places: a component read into a `float` widens; a `float` stored into a `Vector2` narrows.
    - Keep iteration order and ties exactly.
  - **The wrapper:** `#[func] fn art_declump(&self, ...)` in `world_node.rs` takes packed arrays:
    - row ids, visible, walking and place-known flags in row order;
    - target ids and xy; rect ids and position/size, including rects of rows not in the order;
    - terrain width, height and bytes; blocked tiles;
    - the spatial-hash flag.
    - It returns packed ids and offsets.
  - **The GDScript side:** packs the same inputs at `agent_renderer.gd` L671 and unpacks into the dictionary the old call returned.
  - **The switch:** `P12_DECLUMP_KERNEL` (default on after acceptance) selects between kernel and original. The GDScript `declump` stays unchanged as the reference.
  - **Differential test:**
    - Record ≥ 2,000 real declump calls into a fixture: inputs and outputs, every float stored as its exact bit pattern.
    - Sources: W0, W1, W200, W500 and W-multi, frozen and running.
    - `cargo test -p sim-bridge` asserts bit-equal outputs for every call.
    - The existing `test_declump.gd` and `test_spatial_hash_p10.gd` also run through the kernel path.
- **b. Skip unchanged agents (exact-input memo).**
  - **Inventory first.** Before any code, list by grep everything the per-agent body (L155–267 and the post-loop L269–279) reads:
    - snapshot columns, the tile context, `_animation_time`, `delta`;
    - per-agent timers (`_stationary_seconds`, `_idle_seconds`, pose moments, morning chill);
    - selection, zoom and camera, `_capture_visible`.
  - **List every output** it writes, for every reader in Art Pass 11's inventory.
  - **The memo:** an agent's body is skipped in a frame only if its full input key equals the previous frame's key.
    - A time input enters the key as the value its outputs depend on: the chosen animation frame index, or whether a timer threshold is crossed. Use this only where the inventory proves the outputs depend on it through that value alone.
    - Otherwise an agent whose time input changed is never skipped.
  - **Still done for a skipped agent:**
    - timer accumulation;
    - `live` bookkeeping and retirement;
    - `_selection_index.observe`;
    - its draw row for seats, declump, easing and walker offsets.

    The skip removes the body's recomputation and property writes, not the global passes.
  - **Always fully processed:**
    - the selected agent;
    - new sprites;
    - a world reset;
    - a language change;
    - frames where `_capture_visible` or the activity oracle changes.
  - **Hit/miss counters** for diagnosis only.
- **c.** Anything else ≥ 1 ms per frame in 4.6 (c) or in the refreshed section timers. The `_apply_settlement_outline` double call (`tick-cache-diagnosis.md`) is one known item.

5.3 **Acceptance** (each candidate, against the last accepted build; the reference of 4.3 is re-captured on the last accepted build when a candidate is accepted):
- **Identity.** All of these must pass:
  - six framings: B equals A/A′ exactly, P5 included;
  - running dumps on W0 and W200, 600 frames, sprites inside the viewport and the 64 px margin band, every property in Art Pass 10's 5.2b list equal;
  - W500 pan test, frozen and 2×, 240 frames, equal.
  - Candidate (b) adds the same running dump on **W-multi** (porters moving).
- **Speed:** three alternating 150-frame pairs (`run_alternating_p12.py`) on W500 frozen Z4 **and** W200 running 2×, all sessions valid (1.2).
  - **Accepted:** the pooled median ratio is ≤ 0.97 on both, and at least 2 of the 3 pairs are faster on each.
  - **REJECTED:** valid pairs that miss this.
  - **NOT_VERIFIED (contended):** pairs never valid in three holds.
- A rejected candidate's code leaves the production path. Its evidence is kept under `docs/art/production12/perf/<candidate>-rejected/`.

---

# 6. PART 2 — Reading without mistakes
6.1 **Names that hold together and read clearly.**
- **Word joiners:** every person name and village name inserted into a sentence, a link or a label in the inspector and the chronicle panel gets U+2060 WORD JOINER between its characters (Korean and English), so a line never breaks inside a name.
  - Check that the font renders U+2060 at zero width (a capture of `카루미` with and without joiners: identical pixels on one line).
  - Copied text and tests strip U+2060.
- **Bold names:** names in Life lines and chronicle lines use `Galmuri11-Bold`, through RichTextLabel `[b]`, the theme's bold font.
- **Glyph check:** a test capture at the Life text's actual size draws `오후티 오후디 티 디 트 드 키 기` in regular and bold. Record whether 티/디 differ by at least 2 pixels in each; if not, raise the Life and chronicle font size to the next size where they do, and record it.

6.2 **Life layout.**
- **No ellipsis on people:** family rows show the full name and status, wrapping to a second line (remove `OVERRUN_TRIM_ELLIPSIS` at L394 for person rows).
- **Scroll padding:** the inspector's and the chronicle panel's scroll content end with bottom padding of one line height, so the last line is fully visible at maximum scroll. The first line is fully visible at minimum scroll.
- **Stockpile labels follow the language:** diagnose why the labels at `building_renderer.gd` L260–265 stay Korean on the English screen, and fix it so they re-render on every language change (record the cause).
- **Tests:**
  - a scan that, with locale `en`, finds no Hangul in any visible `Label`/`RichTextLabel`/drawn caption, and with `ko` no raw key or `%`;
  - a layout test that, at both scroll extremes, finds no visible text line cut by the scroll viewport.

6.3 **Crowds.**
- **a. Click cycling.**
  - `AgentRenderer.drawn_entities_at(world_position) -> PackedInt64Array`: every visible agent whose opaque rect contains the point, front to back (larger drawn y first, then larger AgentId), by `drawn_entity_at`'s rect rule.
  - `world_renderer.gd` `_try_agent_click` behaviour:
    - a click within 8 screen px and 1.5 s of the previous agent click selects the next entry of that list after the currently selected one, wrapping;
    - otherwise it selects the first entry;
    - the existing tile-centre fallback is unchanged.
- **b. The selected person stands out.**
  - The selected sprite and its activity icon draw above every other agent and icon: `z_index` + 1 within the agent layer, restored on deselect.
  - The selection ring is 2 px wide at zoom ≤ 2 and 1.5 px above.
- **c. Link hover** (droppable): hovering a person link in the inspector or the chronicle panel outlines that person's sprite with the settlement-outline shader in the ring colour, for as long as the link is hovered. Off screen → the existing off-screen marker shows them.

6.4 **Real-input integration test** (`scripts/test/art/test_real_input_p12.gd`, windowed through the lock, `Input.parse_input_event` only, no direct method calls for the actions under test). On a fixed world:
- a click selects an agent;
- two further clicks cycle through three overlapping agents and wrap;
- clicking a dead parent's link opens record mode;
- the mouse wheel scrolls the inspector to its end, and the last line is fully visible;
- Escape leaves record mode;
- C opens and closes the chronicle panel, and a name link in it selects the person;
- language switch: after switching to Korean and back, no Hangul is visible on the English screen.

Each step asserts the resulting state and saves a screenshot.

6.5 **Village summary at zoom < 2:**
- icons drawn at 2× their current screen size;
- the count in `SmallNumber` (Galmuri9, 20 px);
- layout otherwise as in Art Pass 11.

6.6 **Verification with bars** (fixed here; results recorded before any change in response):
- **(a)** 6.1–6.5's tests pass, and Art Pass 10's sidecar pose check still shows 0 mismatches.
- **(b) Captures:** the live seed (`game_v25`, `DEFAULT_GAME_SEED`) at day 30 10:00 and 15:00, and seed 20261018 at day 60 10:00, at Z4 and Z1, with legend sidecars. Selection rules for crops and supplements as in Art Pass 11 6.5 (c).
- **(c) Activity blind re-test at Z4, crops that contain the icon.** Geometry, fixed before any capture:
  - the crop is the union of the holder's body rect and its icon rect (from the sidecar; for a holder whose icon is hidden, the rect from `head_icon_position`), grown by 8 screen px on every side;
  - then squared about its centre (side = max of width and height);
  - pixels are not rescaled.
  - **Exclusion:** a holder is excluded if more than 50 % of its body rect **or** more than 50 % of its icon rect is covered by sprites or icons drawn above it. Log exclusions before any crop.
  - **Reader:** a fresh reader who has not seen Art Passes 10–11's keys, this section or the icon sheet. They get the crops and the seven activity names; answers are frozen.
  - **Bar: ≥ 10 / 14 and no kind at 0 / 2.**
  - Art Pass 11's body-only result (1/12) stays on record; no body-only re-test.
- **(d) Village summary at Z1** (droppable): six pairs as in Art Pass 11 6.5 (d), answered after (c) is frozen and before any key. **Bar: ≥ 5 / 6.**
- **(e) Life reading at the new layout.** Captures in EN and KO:
  - **The four Art Pass 11 subjects:** a living person with a dead parent, that parent in record mode, the chronicle panel, a crowded village view with a selected person.
  - **Departures:** one person per `DepartCause` present, from a W-multi day after a split; the world's chronicle names the people.
  - **Porters and quarrels:** a porter's Life section, and a quarreller's if any exists (7.4).
  - **Reading:** a fresh reader reads each capture and writes what they learn, marking guesses as "inference"; recorded before any key. The non-inference statements are compared with the chronicle data.
  - **Bar:** every statement correct; no raw key, `%` or U+2060 artifact visible; no name split across lines.
- **(f) Crowd correspondence:**
  - in each of three crowded Z4 captures (≥ 25 agents within 10 tiles of the view centre) with one selected person, the sidecar shows 0 % of the selected person's body and icon covered;
  - the reader of (e) names the selected person's activity from the picture: **≥ 3 / 3**.

---

# 7. PART 3 — Villages that help and quarrel, on screen
7.1 **Data:** `get_porter_snapshot` and `get_settlement_relations` take no arguments. Route them through `SimReads`: read once per tick, cached.

7.2 **Porters** (never dropped):
- **Body pose.** While an agent has a porter row:
  - the agent snapshot shows a carried load (food or wood > 0) → the existing carry pose of its kind ("carry" for food, "carry_wood" for wood) with `CarryLoad`. This covers the way out and a failed trip's way back;
  - no carried load → walk.
  - Never use the porter row's `amount` to decide whether a load is drawn (2.2).
  - This overrides the activity resolver's choice for that agent only. A meal or a cold pose still wins, as for any agent.
- **The porter glyph** on the activity-icon path: a pack with a strap, 12–18 source px, hue food = warm orange / wood = brown.
  - Shown at zoom ≥ 2 for both stages; return stage at 60 % alpha.
  - Procedural first; generations per 0.8. Add it to the icon sheet.
- **At zoom < 2:** the glyph at 2× above the porter, and a dotted 1 px screen line from the porter to its current destination hearth (the receiver's on the way out, the sender's on the way back), alpha 0.6, in the glyph's hue.
- **Inspector:** a porter's status line `INSPECTOR_PORTER_OUT_FMT` or `INSPECTOR_PORTER_BACK_FMT`, under the name.

7.3 **Relation lines** (droppable; zoom < 2, under the village labels, in `settlement_labels.gd` or a sibling node):
- one line between the hearths of every pair in `get_settlement_relations`, 2 px screen;
- colour by value: ≥ 150 green `#7FB069`, 0–149 light grey `#B8B8B0`, < 0 orange `#D9893A`;
- estranged: red `#C0392B`, dashed, with `UI_RELATION_ESTRANGED` at the midpoint;
- recomputed once per tick.

7.4 **Chronicle sentences 17–21** (never dropped):
- **Person roles (Life section and record mode):**
  - 17 as subject → `CHRON_PORTER_DEPARTED_FMT`;
  - 18 as subject → `CHRON_PORTER_DELIVERED_FMT`, or `CHRON_PORTER_FAILED_FMT` when cause ≥ 10. Remember 18's settlement is the **receiver**.
  - 19 as subject → `CHRON_QUARREL_FMT` with the other and settlement2; as other → with the subject and settlement.
  - Purpose noun: `CHRON_PORTER_AID` for causes 0–1 (and 10–11), `CHRON_PORTER_EXCHANGE` for 2–3 (and 12–13).
  - Load noun: `CHRON_LOAD_FOOD` for even causes, `CHRON_LOAD_WOOD` for odd causes (the cause mod 10).
- **Village lines:** 20 → `CHRON_VILLAGES_ESTRANGED_FMT`; 21 → `CHRON_VILLAGES_RECONCILED_FMT`.
- **The chronicle panel:** `NOTABLE` gains 18, 19, 20 and 21; 17 stays out, since delivery covers it.
  - Consecutive 18 entries with the same tick, settlement, settlement2 and cause become one line:
    - delivered → `CHRON_PORTER_GROUP_FMT` (count, load, summed amount, sender, receiver, purpose);
    - failed → `CHRON_PORTER_GROUP_FAILED_FMT`.
  - The subject-name prefix is omitted on these group lines and on 20/21; village names are links (center the camera).
- **Unknown kinds (≥ 22):** skipped silently, counted in a debug log.
- **GDScript tests with a fake bridge:** every kind, role and cause, in EN and KO, including the aggregation and the receiver/sender order of 18.

7.5 **Captures:**
- W-multi: a Z4 view centred on a porter on the road (stage out), and a Z1 view showing the porters' lines and the relation lines.
- A world with a `VillagesEstranged` entry (from Pass 20's evaluation chronicles), at the day of that entry, Z1. If none exists, NOT_RUN, with unit tests only.
- EN and KO for the inspector of the porter.

---

# 8. PART 4 — After: measurements and verdicts (start by hour 9)
Rerun 4.6 (a) and (b) on the frozen build (0.6), valid sessions only, plus section timers on W200 running 2×.

| # | question | rule |
|---|---|---|
| H1 | the live game, W0 frozen Z4 | median ≤ **6.0 ms** |
| H2 | **the live game running**: W1 running 2× Z4 | **p99 ≤ 16.67 ms**; median reported |
| H3 | 200 people running: W200 running 2× Z4 | median ≤ **16.67 ms**, with ticks per frame |
| H4 | 200 people frozen: W200 frozen Z4 | median ≤ **10.0 ms** |
| H5 | 500 people frozen: W500 frozen Z4 | median ≤ **16.67 ms** |
| H6 | W500 running 2×; W-multi running 2× | reported: medians, p99, ticks per frame, populations (flagged blocks shown and excluded; W500 never a pass or a fail) |
| H7 | identity of every accepted candidate (5.3) | all pass |
| H8 | activities | 6.6 (c) ≥ 10/14 with no kind at 0/2; (d) ≥ 5/6 (NOT_RUN if dropped); (a) passes |
| H9 | lives and crowds | 6.6 (e) every statement correct, no raw key, no split name; (f) 0 % covered and ≥ 3/3; 6.4 all steps pass |
| H10 | villages on screen | 7.4 tests pass; 7.5 porter capture shows the glyph and the carry pose (NOT_RUN only under 2.2's fallbacks) |

Report every number before and after with ratios, the section timers before and after (frozen W500 and running W200), and the running diagnosis next to its after-values.

---

# 9. PART 5 — Review, gate, report
1. **Independent review** (two rounds max). Scope:
   - the merge and the relations contract;
   - the release worktree (sha per session; nothing committed to `sim_bridge.gdextension`);
   - **the kernel's float semantics and the differential fixture**;
   - **the memo inventory**, if (b) was attempted;
   - session validity (1.2) for every number used in a verdict;
   - word joiners and the language scan;
   - click cycling and the real-input test;
   - the blind protocol: geometry fixed before captures; exclusions before crops; answers frozen before keys; (d) after (c);
   - kinds 17–21, including the receiver/sender order;
   - the H verdicts, the allowed paths and the quiet-lock record.
2. **Gate and publish:**
   - full gate: `GATE_OUTPUT_DIR=docs/art/production12/gates/final bash tools/art_pipeline/gate.sh final`, including `cargo test -p sim-bridge` and clippy for the new Rust;
   - fast-forward `codex/art-production-2026-10`;
   - add a PR #511 section;
   - remove both worktrees.
3. **Report:** `docs/art/production12/REPORT.md` in Korean (section 12 format), and the same text in the final message.

---

# 10. Localization keys
- **Files:** add each key in the Part that first uses it, in all five files:
  - `localization/fluent/{en,ko}/messages.ftl`;
  - `localization/compiled/{en,ko}.json`, via `python3 tools/localization_compile.py --project-root .`;
  - `localization/key_registry.json`, with the next ids.
- **Reuse:** grep each key first; reuse where the meaning is the same, and record it.
- **Korean grammar:** Korean sentences avoid particles attached to a name or village (을/를, 이/가, 은/는, 와/과, (으)로). 에서 and 까지 do not depend on the final consonant.
- **Specifiers:** every row has the same `%` specifiers in the same order in en and ko.

| key | en | ko |
|---|---|---|
| `CHRON_PORTER_DEPARTED_FMT` | Set out from %s for %s with %s ×%d (%s). | 짐을 지고 떠남 — %s → %s, %s %d (%s) |
| `CHRON_PORTER_DELIVERED_FMT` | Brought %s ×%d from %s to %s (%s). | 짐을 전함 — %s %d, %s → %s (%s) |
| `CHRON_PORTER_FAILED_FMT` | Could not reach %s and turned back (%s). | %s까지 가지 못하고 돌아옴 (%s) |
| `CHRON_PORTER_AID` | aid | 원조 |
| `CHRON_PORTER_EXCHANGE` | exchange | 교환 |
| `CHRON_LOAD_FOOD` | food | 식량 |
| `CHRON_LOAD_WOOD` | firewood | 장작 |
| `CHRON_QUARREL_FMT` | Quarrelled with %s of %s over a berry bush. | 열매 덤불을 두고 다툼 — %s (%s) |
| `CHRON_VILLAGES_ESTRANGED_FMT` | %s and %s fell out. | %s · %s 사이가 틀어짐 |
| `CHRON_VILLAGES_RECONCILED_FMT` | %s and %s made peace. | %s · %s 화해함 |
| `CHRON_PORTER_GROUP_FMT` | Porters (%d) brought %s ×%d from %s to %s (%s). | 짐꾼 %d명 — %s %d, %s → %s (%s) |
| `CHRON_PORTER_GROUP_FAILED_FMT` | Porters (%d) from %s could not reach %s (%s). | 짐꾼 %d명 — %s에서 %s까지 가지 못함 (%s) |
| `INSPECTOR_PORTER_OUT_FMT` | Carrying %s ×%d to %s | 짐 나르는 중 — %s %d → %s |
| `INSPECTOR_PORTER_BACK_FMT` | Heading home to %s | 집으로 돌아가는 중 — %s |
| `UI_RELATION_ESTRANGED` | Estranged | 사이가 틀어짐 |

**Argument order:**
- `CHRON_PORTER_DEPARTED_FMT`: sender, receiver, load, amount, purpose.
- `CHRON_PORTER_DELIVERED_FMT`: load, amount, **sender (settlement2), receiver (settlement)**, purpose.
- `CHRON_PORTER_FAILED_FMT`: receiver (settlement), purpose.
- `CHRON_QUARREL_FMT`: the other person, their village.
- The group keys: count, (load, amount sum,) sender, receiver, purpose.
- `INSPECTOR_PORTER_OUT_FMT`: load, the porter row's amount, receiver.

---

# 11. 인게임 확인사항 (오너용)
- 게임이 `game_v25`로 시작한다. 2배속으로 돌려도 마을 하나(사람 60~70명)에서 화면이 끊기지 않는지(이전에는 열 프레임 중 하나꼴로 늦었다).
- 많이 모인 곳에서 같은 자리를 여러 번 클릭하면 겹친 사람이 차례로 선택되는지. 선택한 사람과 그 사람의 아이콘이 다른 사람들 위에 그려지고 고리가 잘 보이는지.
- 생애 사건에서 이름이 줄 끝에서 끊기지 않고 굵게 보이는지(티/디가 구별되는지). 영어 화면에 한국어 "식량·장작"이 남아 있지 않은지. 스크롤을 끝까지 내렸을 때 마지막 줄이 잘리지 않는지.
- 마을이 둘 이상 생긴 뒤 오전 8시쯤 짐을 진 사람이 다른 마을로 걸어가는지(주황 짐 = 식량, 갈색 짐 = 장작). 줌아웃하면 짐꾼에서 목적지 마을까지 점선이, 마을 사이에는 관계선(초록 = 가까움, 회색 = 보통, 주황 = 나쁨, 빨간 점선 = 사이가 틀어짐)이 보이는지.
- 짐꾼을 클릭하면 "짐 나르는 중 — 식량 10 → (마을)"이 나오는지. C 키 연대기에 "짐꾼 2명 — …", 다툼, 사이가 틀어짐/화해가 문장으로 나오는지.
- 줌아웃했을 때 마을 요약 아이콘이 전보다 크게 보이는지. 영어·한국어 모두 번역 키나 %가 보이지 않는지, 콘솔 에러가 없는지.

---

# 12. 구현 완료 보고 (REPORT.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
### 구현 내용
(병합과 관계 계약, 실행 프레임 진단, Rust declump 커널과 그 밖의 프레임 예산 수정, 이름·생애 배치·언어 수정, 군중 선택, 실제 입력 테스트, 아이콘 포함 블라인드 재시험, 짐꾼·관계선·연대기 17~21)
### 구현 방법
(release 측정과 세션 유효성 규칙, 커널의 부동소수점 의미와 비트 단위 차분 고정자료, 변경 없는 에이전트 생략의 입력 목록, 동일성 검증, 워드 조이너, 클릭 순환, 크롭 기하, 연대기 역할별 문장과 묶음)
### 기능 설명
### 변경된 파일 목록
(커밋별)
### 측정 결과
H1~H10 표(전후·배수, release), 세계·조건별 중앙값/p90/p99/최대·틱/프레임·인구(표시된 블록 포함), 실행 진단(전후), 구간 타이머 전후(정지 W500·실행 W200), 후보별 판정(채택/기각/NOT_VERIFIED와 유효 세션 기록), 블라인드 결과(활동별·제외 기록)와 마을 요약 결과, 생애 판독 대조, 군중 대응 결과
### 환경
### 독립 리뷰
### 자율 결정 목록
### 실행 기록
### 확인된 제한사항
### 게이트 결과
### 정직한 판단
마을 하나를 2배속으로 돌릴 때 화면이 이제 버티는가? 몇 명까지(정지/실행, release)? 사람의 일생을 틀리지 않고 읽을 수 있는가? 붐비는 곳에서 사람을 구별·선택할 수 있는가? 짐꾼과 마을 사이 관계가 화면에서 읽히는가? 다음 아트 과제는 무엇인가?
```
The final message also includes:
- every commit hash with its Part;
- both branch heads, confirmed with `git ls-remote`;
- the PR #511 update;
- the time per Part, with any pauses.