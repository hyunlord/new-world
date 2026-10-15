# GOAL: WorldSim — Art Pass 10: a crowd the screen can carry, and a day you can read — the agent renderer at scale, the new daily activities on screen, and family and temperament in the inspector

Repository `hyunlord/new-world`. Continue the art line: `codex/art-production-2026-10-wip` = `codex/art-production-2026-10` = `21d9ddd089a77bfc3daa9b9765888eb76a890a85` (Art Pass 9; draft PR #511).
This run is executed by **Codex**, unattended, on the Mac, in `/Users/rexxa/work/new-world-art-prod`. **At the same time a simulation run (Pass 18, Codex) works in `/Users/rexxa/work/new-world-pass4`**; section 1.2 is how the two runs share the Mac. No DGX.

**Simulation commit to merge:** `8cbb697c97aac800935813d2c79c34195ebef6a3` (Pass 17's green head; it brings Passes 16–17: temperament `game_v20`, daily routine `game_v21`, family `game_v22`; the live game builds `game_v22`; new bridge getters `get_agent_temperament`, `get_activity_snapshot`, `get_agent_family`; `state_tag` 4 = activity, `target_kind` 8 = activity).

Three Parts of work, in this order: (1) **the agent renderer at scale**, with every pixel unchanged; (2) **draw the activities**; (3) **family and temperament in the inspector**.

---

# ⚠️ 0. AUTONOMY — READ FIRST
1. **Never ask the owner anything.** Decisions in `docs/art/production10/DECISIONS.md`.
2. **First action:** save this document unchanged as `docs/art/production10/GOAL.md`, commit it in Part 0, re-read the current Part's section at every Part start and after any compaction.
3. **Parts:** 0 = section 4; 1 = section 5; 2 = section 6; 3 = section 7; 4 = section 8; 5 = section 9. Push after every Part.
4. **Review loops: two rounds maximum** (fresh reviewer that has not seen sections 3–7).
5. **On failure: work around, note it, continue.** Three strikes per failing target, then keep the best state that still passes Part 1's pixel identity.
6. **Time rule:** record times in `docs/art/production10/TASKS.md`. Past **12 hours**, finish the current step, push, go to Part 5.
7. **Drop order:** the household highlight (7.3) → n = 1,000 measurements → Part 2's icon polish (6.4). Never drop Parts 0, 1, 4, 5, the activity poses (6.2–6.3) or the inspector sections (7.1–7.2).
8. **Image budget: at most 4 generations** (only for activity icons if no existing frame or procedural drawing works); failures count.
9. **Honesty rules:** never mark a failed check as passed, never lower a threshold of section 8, keep rejected evidence, never re-interpret an earlier pass's verdict.
10. **This brief supersedes the repository's session-start instructions:** no hooks, no harness pipeline, never `lead/main` or `main`, never force-push.

---

# ⚠️ 1. WHERE AND HOW
## 1.1 Places and tools
- Mac only, `/Users/rexxa/work/new-world-art-prod`; evidence to `CAPTURE_EVIDENCE=/Users/rexxa/work/art-production10-evidence`; one base worktree `/Users/rexxa/work/art-p10-base` (detached at the Part 0 merge commit, its own debug `sim-bridge` build, then `"$GODOT_BIN" --headless --path … --import`), removed at the end. Never touch other checkouts or processes you did not start; no ComfyUI, no dev servers, no DGX.
- `CARGO_BUILD_JOBS=6`; `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`; windowed Godot only through `tools/art_pipeline/with_window_lock.sh`; limits via `run_bounded.sh`. **Frame-time sessions run inside a quiet-lock hold; identity captures run outside holds** (Art Pass 9's `run_identity_p9.sh` L35–38, L52 waits until the lock is absent — inside your own hold it would deadlock).
- Reuse Art Pass 9's tools, copied to `_p10` names writing under `docs/art/production10/**`: `perf_frames_p9.gd`, `frame_measure_session.gd`, `frame_cpu_probe.gd`, `with_quiet_lock_p9.sh`, `run_perf_matrix_p9.sh`, `summarize_perf_p9.py`, `compare_perf_p9.py`, `summarize_ablation_p9.py`, `pixel_identity_p9.py`, `capture_identity_p9.gd`, `capture_framings_p9.gd`, `run_identity_p9.sh`.
## 1.2 Sharing the Mac with the simulation run (both briefs carry this protocol)
Quiet lock `/Users/rexxa/work/.mac_quiet.lock` (`mkdir`; `info.txt` with owner, pid, purpose, started; removed when done). One hold per measurement session (one Godot launch, one world), ≤ 60 minutes, ≥ 10 minutes between your holds; quit Godot at the end of the session; never leave a Godot window running while waiting. While the other run holds it: no cargo build or test, no Godot, no gate; poll every 5 minutes; a lock older than 2 hours is stale (remove, record). Before taking it: no `cargo`/`rustc`/`worldsim_*`/`Godot` process of the other run (`pgrep -x`) and 1-minute load < 4; after 60 minutes take it anyway and mark the session `contended`. Record every hold and wait in `docs/art/production10/perf/holds.md`.

---

# 2. GIT
## 2.1 Branches and scope
- Start from `21d9ddd0` (must equal origin); backup branch `art-p10-local-backup`. WIP gets every commit; `codex/art-production-2026-10` moves only by fast-forward to a WIP commit whose full gate is green; `git ls-remote` after every push. Messages `art-p10(pN): …`; stage explicit paths only.
- **May change:** `scripts/**`, `scenes/**`, `shaders/**`, `assets/**` (new icons only), `localization/**` (the keys of section 10 only), `project.godot`, `docs/art/production10/**`, `docs/art/TEST_LEDGER.md`, `tools/art_pipeline/**`, Rust static-source tests under `rust/crates/sim-test/tests/` that read `.gd`/`.tscn` files (re-pinned with the same meaning; one ledger row each under `## Art Pass 10`).
- **Rust exceptions:** in `rust/crates/sim-bridge/src/ffi/world_node.rs` only the art-owned getters (`get_intervention_state`, `get_society_snapshot`, `get_tile_causal_events`), their collectors and tests. Nothing else under `rust/`.
- **Must not change:** everything else under `rust/`, `docs/revitalize/**`, `docs/worldmodel/**`, `tools/revitalize/**`, `tools/worldmodel/**`, `tools/harness/**`, `.claude/**`, `.harness/**`, evidence of earlier passes.
## 2.2 Part 0 merge
`git merge --no-ff 8cbb697c97aac800935813d2c79c34195ebef6a3 -m "art-p10(p0): bring in simulation passes 16 and 17"`. Conflicts: simulation-owned paths take the simulation's side; keep the art getters; record every resolved file.
**Planner's trial merge** (Linux): `git merge-tree` of `21d9ddd0` and `8cbb697c` exits 0; `cargo test -p sim-bridge` passes 41/41 after the merge; `cargo test -p sim-test --no-run` builds; all 44 `sim-test` test binaries that read source files pass. Godot was not run: **build and run the full gate right after the merge** (4.1).

---

# 3. WHY THIS PASS (planner's review with a code survey at `21d9ddd0`; verify each fact)
1. **The agent renderer is the frame budget.** Art Pass 9 left W500 frozen at Z4 at 51.5 ms per frame, of which `AgentRenderer` (with its `FoundingGroups` child) is 46.4 ms — ≈ 90 µs of GDScript per agent per frame; W200 frozen 29.7 ms. Natural worlds reach 100–150 people late in a run; the game's target is hundreds.
2. **There is no MultiMesh:** every agent is a pooled `Sprite2D` in the shared, y-sorted `Main/WorldObjects` layer (with buildings and trees), plus a shadow `Sprite2D` in `Main/Shadows` and up to ~9 child nodes (`CarryLoad`, `FounderBundle`, `ColdBreath`, `ChillBreath`, `ChillOutline`, `SettlementOutline`, an activity icon) (`agent_renderer.gd` L66, L209–230, L454). Drawing is cheap (the layer ablates to ≈ 0); **the cost is `_process`**. Converting to a MultiMesh would break the y-sort interleave with buildings and trees: out of scope.
3. **What `_process` does per frame** (`agent_renderer.gd` L75–207; survey of the planner's reviewer):
   - tick-only work redone every frame: the snapshot checksum (L98, L428–435), talk-cache compare (L97, L519–523), `ingest_chill_snapshot` (L120, L281–295) and `MorningChill.ingest`, `_activity_context()` rebuilt from building/construction/resource snapshots (L116, L462–489), `_tile_context` per unique tile with a **nested loop over every hearth and shelter** (L141–144, L491–510);
   - per agent: ~8 dictionary gets/sets; `elder_look` `duplicate()`s the look dictionary (L132; `agent_activity.gd` L76–82); `_pose_moments.update` allocates two dictionaries (L150); `founders.has()` is a linear scan (L190); `Activity.choose_frame` builds 3–11 key strings and `duplicate()`s a dictionary (L164, L437–440; `agent_activity.gd` L128–173); **`gather_wood` has no frames, so every wood-gathering agent calls `wood_icon(ArtSpriteLibrary.entries())`, and `entries()` is `_entries.duplicate(true)` — a deep copy of the 306-sprite atlas per agent per frame** (`agent_activity.gd` L166 inside `choose_frame`, and L108–109; `art_sprite_library.gd` L39–41); `SpriteLibrary.apply_variant` does a string concat and **`ResourceLoader.exists()` per agent per frame** (L171; `art_sprite_library.gd` L87–105); ~40 `set_meta` and ~9 `get_node_or_null` per agent; a 12-key `draw_agents` dictionary per agent (L192);
   - `_apply_drawn_positions` (L193, L545–618): marks an 11 × 15 block around each hearth every frame; `CampSeats.update` compares dictionaries and assigns seats with a candidates × assignments search (`camp_seats.gd` L16–50); `declump` is **O(N²)** `Rect2.intersection` with up to 16 retries (L128–177); `walker_offsets` rebuilds a list of all N − 1 rects for every "walking" agent **with a non-zero walking direction** (L199–242, L213–215), with up to 6 retries; "walking" includes `Seeking` agents even when not moving (`agent_activity.gd` L38; `_walking` L95–96) — whether a frozen world pays this is for the section timers (5.1) to show;
   - post-loop per agent: `Cold.apply`, `_morning_chill.apply` (several `get_node_or_null`), `_apply_settlement_outline` (ShaderMaterial parameter sets), shadow, cue; `_retire_missing` allocates `keys()` arrays; `_update_selection_ring` scans all sprites.
   - 22 `sim-test` files read `agent_renderer.gd` — 19 through `include_str!` (e.g. `harness_p11_alpha_agent_renderer.rs` L389–401 requires the literal `SpriteLibrary.apply(sprite, sprite_id)`), 3 at runtime (e.g. `harness_d_phase_a_runtime_warnings.rs` L76).
   - `reset_world_stress` builds `game_v19` (`world_node.rs` ≈ L184–186, L279–281): crowded worlds have no temperament, no activities and no families. Frame-time measurements on W200/W500 therefore measure the renderer on v19 behaviour; activity rendering cost is measured on live worlds only (a limitation to report).
4. **The simulation now has a day the screen does not show.** In `game_v21`/`v22` the daytime idle share of adults fell from 0.60 to 0.12; children play 70–93 % of the day and elders sit by the fire 60–75 % (Pass 17). But the art branch maps `state_tag` 4 to `idle` (or `walk` while moving) and an unknown target with state 3 to `eat` (`agent_activity.gd` L14–42); the inspector clamps unknown states to its last label (`agent_inspector_panel.gd` L34–48, L140–152). Kinds come from `get_activity_snapshot()` → `{ids (entity bits), kinds 0–6, place_x, place_y, until_tick}` (kinds: 0 TendFire, 1 Craft, 2 Repair, 3 HearthCircle, 4 Explore, 5 Play, 6 SitByFire).
5. **People now have temperament and family, invisible on screen:** `get_agent_temperament(entity_bits)` → `{found, ns, ha, rd, p}` (permille); `get_agent_family(entity_bits)` → `{found, female, mother, father, partner, children}` (agent ids, −1 when none; `children` a sorted `PackedInt64Array` of living children's agent ids).
6. **Available frames** (`assets/art/atlas/world_atlas.json`; groups built at `agent_activity.gd` L84–105): walk, idle (all bodies, E/N/S); sit (a: E/S; b: E/N/S; child: E/S); talk (E/S); eat; gather; carry; build (adults E/N/S; child E/S); asleep; drink; carry_wood (adults N); crossed, crouch (adults S); hop, hug (child S); run (child E). No `gather_wood` frames.

---

# 4. PART 0 — Merge, gate, the new baseline
4.1 Backup; merge (2.2); full gate (`GATE_OUTPUT_DIR=docs/art/production10/gates/p0 bash tools/art_pipeline/gate.sh p0`) under 1.2; fix fallout in art-owned code and tests; three strikes.
4.2 Record that the live game builds `game_v22` and the new getters' outputs on the live seed at day 30, 10:00 (counts of `state_tag` 4 per kind, five families).
4.3 **New baseline** (the merge changes the live world, so Art Pass 9's captures are no longer the reference): with the merge-commit build, Art Pass 9's measurement matrix (`perf_frames_p10.gd`: W0, W1, W200, W500; frozen Z4/Z1, running 2× and 4×; three 300-frame blocks; ablation on W0 and W500) and Art Pass 9's six identity framings A and A′ (independent launches, `--fixed-fps 60`, captured at equal process-frame counts). **Re-derive every framing for `game_v22` first:** the framings are hard-coded to seed histories the merge changes (`capture_identity_p9.gd` CASES L12–16; P3's plan in `run_identity_p9.sh` L29 and the `target != 46189` check in `capture_framings_p9.gd` L286; W1 in `perf_frames_p9.gd` L9; P1/P6 abort without a settlement at tick 40,560, L58–60). For each framing record settlements, population and fire state at its tick under `game_v22`; where a framing no longer shows what it was meant to (P2 a lit night, P3/P4 several villages), pick the first tick (or another seed among 20,261,011–20,261,042) that does, and edit all three places in the `_p10` copies. P5 (`reset_world_stress`, still `game_v19`) should reproduce Art Pass 9's final B exactly — check it as a free consistency test. Commit, push.

---

# 5. PART 1 — The agent renderer at scale (every pixel unchanged)
5.1 **Instrument first:** add per-section timers to `AgentRenderer._process` behind a debug flag (off by default, zero cost when off): reads, checksum/talk/chill ingest, activity context, tile contexts, the per-agent loop split into look/pose/frame choice/variant/meta/children, `_apply_drawn_positions` split into seats/declump/walker offsets, the post-loop pass, retire, selection. Run W500 frozen Z4 once; write `docs/art/production10/perf/agent_sections.md`.
5.2 **Fix, largest first** (each change keeps Part 1's identity — 5.3 — and is measured with a 150-frame alternating pair). **Part 1 changes speed only, never meaning**: what is drawn, where and when stays exactly as today.
a. **Cache tick-only work.** Cache key = (simulation tick, a `SimReads` mutation counter incremented by every mutating wrapper — `apply_intervention`, `advance_ticks`, `reset_world*`, `on_building_placed` — and a reset counter incremented on `world_reset`). Cacheable: activity context, tile contexts (replace the hearth/shelter nested loop by a per-key lookup), chill ingest, talk delta, `resolve` minus the `moving` bit, `camp_pose` inputs, cold, founder membership (a set, not `has()` on an array), look/elder look (no per-frame `duplicate()`). **Not cached** (read every frame as today): home settlement and outline (`SettlementSelection.observe()` is called by other nodes, `settlement_labels.gd` L55, `hud_status_panel.gd` L118), and everything on the per-frame list below.
   **Must stay per frame:** pose-moment timers (`agent_pose_moments.gd` L16, L19, L27), `_stationary_seconds` and `_idle_seconds` (idle agents turn S after 1 s, `agent_activity.gd` L62), animation frame index, `Cold.apply` shiver/breath and the chill breath, position lerp and easing (`agent_renderer.gd` L588), walker offsets, selection-ring pulse, cue counters, icon zoom visibility (L446–448), `selected_id`, camera/canvas transform.
b. **Remove per-frame allocations and lookups:** no `entries()` deep copy (cache the wood icon); cache `apply_variant`'s `ResourceLoader.exists` result per page/variant; cache child-node references per sprite (no per-frame `get_node_or_null`); reuse temporary dictionaries; `set_meta` only when the value changed; selection ring via the `agent_id → sprite` map.
c. **Make spatial work cheaper without changing its result:** keep `_walking` exactly as today (`camp_seats.gd` L95–96). Give `declump` and `walker_offsets` a spatial hash (cell ≥ the largest merged footprint including `CarryLoad`, plus the 8 px shift) that returns candidates **in the original iteration order** of the fronts (`_covered` sums non-integer areas against a 0.30 threshold; a different summation order can flip it). Skip `declump` / `walker_offsets` only when a hash of every agent's (target position, rect — which depends on the current frame's size, `flip_h` and `CarryLoad` visibility, `agent_renderer.gd` L569–582 — walking flag, visibility) **and** the canvas transform equals last frame's. Seats: reassign only when the blocked set, the claimants or any claimant's `stationary_seconds ≥ 0.3` status changed.
d. Anything else the section timers show above 1 ms at W500.
5.2b **Running check** (frozen framings never cross a tick, so a cache that fails to invalidate would pass 5.3): a deterministic test with `--fixed-fps 60` on W0 (start tick 40,560) and W200 (start tick 1,440) that calls `advance_ticks(1)` **through `SimReads`' mutating wrapper** (so the caches and the mutation counter see it) every 2 frames for 600 frames and dumps, every frame, each agent sprite's (position, texture region, `flip_h`, visibility, modulate/self_modulate, z) and every child node's visibility — run it on the base worktree (as an untracked copy, as Art Pass 9 did with its capture script) and the current build; the dumps must be identical. (Agent renderers read no wall clock or RNG; `world_effects.gd`'s unseeded RNG affects only non-agent effects, which the dump excludes.)
5.3 **Identity for this Part:** after each change, capture the six framings (B) and compare with A/A′ by Art Pass 9's rule (if A = A′ exactly, B must equal A exactly; otherwise B's difference from A ≤ A′'s). A failure is a bug: fix or revert (three strikes).
5.4 Re-pin moved static source pins with the same meaning (ledger rows); keep `SpriteLibrary.apply(sprite, sprite_id)` semantics where pinned.
Commit and push after each accepted change.

---

# 6. PART 2 — Draw the activities
6.1 **Read** `get_activity_snapshot()` through `SimReads` as a memoized per-frame getter (add it to `GETTERS`, to the equivalence test `scripts/test/art/test_sim_reads_equivalence_p9.gd` L30–35, and to `PROTECTED` in `rust/crates/sim-test/tests/harness_art_p9_sim_reads.rs` L4–10); map `ids` (entity bits) to sprites.
6.2 **Poses** (in `agent_activity.gd`: a `state == 4` branch before the others; `target_kind` 8 added to the target match). `choose_frame` falls back pose/view → pose/S → idle/view (L130), and several poses exist only facing S (crouch, crossed, hop) or only for some bodies; so each kind has an explicit chain per body, tried in order, with the view each pose has:
| kind | at the place (Consuming), chain per body | walking to the place | facing at the place |
|---|---|---|---|
| 0 TendFire | adults: `crouch` (S only) → `sit` | walk | S when the pose is S-only; otherwise toward the hearth |
| 1 Craft | adults: `sit` → `idle` | walk | toward the stockpile where `sit` has that view (adult_a/child: E/S only), else S |
| 2 Repair | adults: `build` (E/N/S) | walk | toward the nearest shelter |
| 3 HearthCircle | adults and elders: `sit` → `idle` | walk | toward the hearth where `sit` has the view, else S |
| 4 Explore | adults: `idle` (looking outward) | walk | away from the hearth |
| 5 Play | children: `hop` (S) alternating with `run` (E, mirrored for W) every 0.5 s; when no frame for the view, `hop` S | `run` E/W, else `walk` | free |
| 6 SitByFire | elders: `sit` → `idle` | walk | toward the hearth where `sit` has the view, else S |
Add a facing override input to `render_facing` (L57–62) for "toward/away from a tile". Every `(kind, body, view)` has a defined result (unit-tested). **Existing overrides keep precedence** and are the expected result in 6.5 (d): cold or morning chill turns idle/sit/hop into `crossed` (`cold_effects.gd` L11; `agent_renderer.gd` L162), a meal into `eat` (L158–159), a pickup into `gather_wood` (L152–153); agents asleep in shelters stay hidden.
Activity holders **that have arrived** (`until_tick ≥ 0`) carry a row flag `activity_place_known`: excluded from `CampSeats` seating (`_eligible`) and from receiving declump shifts, but **kept in the tile spread** (`camp_seats.gd` L116–123: the simulation allows two holders per place tile, and the ±5 px pair spread keeps them from drawing on top of each other) and **kept in declump's `fronts`** so others move away from them. A holder still walking to its place is a normal walker. A playing child drawn with `run` is excluded from walker offsets while `activity_place_known` is set (it is moving within one tile of its place). Elders have no body of their own: they use the adult_a/adult_b frames (`agent_renderer.gd` L163) with elder hair. "Toward/away from the hearth" means the settlement's hearth nearest the place tile (places are measured from the formation tile, `places.rs` L41).
Explore gets its own activity string `explore` that uses the `idle` frames but is **not** rewritten by `camp_pose` (`agent_activity.gd` L64–70) and whose facing override takes precedence over the idle turn-to-S after 1 s (L62).
6.3 **Inspector state label:** `get_agent_detail` keeps its 9 keys, so the kind comes from `get_activity_snapshot` (by entity bits): `state_tag` 4 shows the kind's label (section 10) instead of a clamped label; `target_kind` 8 shows `INSPECTOR_TARGET_ACTIVITY_PLACE_FMT` with the place; extend `STATE_LABELS` / `TARGET_LABELS` without clamping.
6.4 **Icons** for Craft (a tool) and Explore (a direction marker), procedural or from existing sprites; at most 4 generations if neither works. Today icons show only for `fallback_kind` `idle_icon` / `wood_icon` at zoom ≥ 2 (`agent_renderer.gd` L175–176): add an `activity_icon` fallback kind on the same path (same zoom rule).
6.5 **Verification:** (a) unit tests of `resolve` for every (state 4, kind) pair; (b) captures of the live seed (`game_v22`) at day 30 10:00 and 15:00, and of a populous `game_v22` world at day 60 10:00 — choose it with the release `worldsim_probe --scene game_v22 --seeds 20261011..20261027 --days 60 --workers 4` (headless, ≤ 30 minutes; fewer seeds if the cap is hit) by the largest population at day 60 (crowded worlds from `reset_world_stress` are `game_v19` and have no activities) — at Z4 and Z1, each with a legend sidecar listing every agent in an activity and the pose drawn; (c) a **blind identification**: a fresh reviewer that has not read this section is shown 14 crops (two per kind, from the captures, agent centred, 4× nearest-neighbour upscale) and the 7 kind names; record the answers before revealing the key; report the share correct (no pass bar — a measurement for the next pass); (d) every activity holder in the captures is drawn with its kind's expected pose after the 6.2 chains and the existing overrides (sidecar check: 0 mismatches; each override case listed).

---

# 7. PART 3 — Family and temperament in the inspector
7.1 **Temperament section** (after the cold row): a heading and four horizontal bars (NS, HA, RD, P; 0–1,000) with labels (section 10 keys); a new bar helper (the existing `_make_bar` indexes 3-element arrays — do not extend it beyond its range). Values from `get_agent_temperament` (direct call, only while the panel is open). Missing component → the section hidden.
7.2 **Family section:** sex; partner, mother, father (each as a clickable name-or-id row that selects that agent, or `PANEL_FAMILY_NONE` — "None" / "없음" — when −1 or dead/missing); "children: n" with the first five ids clickable. Values from `get_agent_family`. Check the panel still fits (620 px height): if not, make the panel scroll.
7.3 **Household highlight:** when an agent is selected, draw a thin ring (pooled `Line2D`, the selection ring's style at lower alpha) on its partner and living children (`agent_id → sprite` lookup), only while the inspector is open.
7.4 **Verification:** GDScript tests with a fake world for both sections (all fields present, missing component, dead partner, clicking selects the agent); a capture of the inspector on a partnered parent in the live seed with the household rings visible.

---

# 8. PART 4 — After: measurements and verdicts
Rerun 4.3's frame-time matrix on the final build; the pixel identity (H5) is judged on **Part 1's final build** only (Parts 2–3 change pixels by design; their captures are new evidence). Add W1000 frozen Z4 if time allows. Note in the report that frozen worlds never invalidate the tick caches (flattering H1–H3), while running at 2× invalidates about every frame.
| # | question | rule |
|---|---|---|
| H1 | the live game, frozen Z4 (W0) | median primary ≤ **6.0 ms** |
| H2 | 200 people, frozen Z4 (W200) | median ≤ **12.0 ms** and p99 ≤ 16.67 ms |
| H3 | 500 people, frozen Z4 (W500) | median ≤ **16.67 ms** |
| H4 | 500 people, running 2×, tick-carrying frames | median ≤ 16.67 ms (reported with ticks per frame and the population at the start and end — Art Pass 9's W500 fell from 500 to ≈ 100 while running) |
| H5 | Part 1's pixel identity (six framings, final Part 1 build vs A/A′) and the running check 5.2b | all pass |
| H6 | AgentRenderer ablation cost at W500 | reported before → after (no bar) |
| H7 | activities on screen | 6.5 (a) all pass and (d) 0 mismatches; blind identification share reported |
| H8 | inspector | 7.4 tests pass |
Report every number before and after with the speed-up, and the section timers before and after.

---

# 9. PART 5 — Review, gate, report
1. Independent review (two rounds max): the merge; identity of Part 1; each 5.2 change; that tick-only caches invalidate on tick change and reset; the activity mapping and the sidecar check; the blind protocol (answers frozen before the key); the inspector tests; localization keys in all five files; the F/H verdicts; allowed paths; quiet-lock record.
2. Full gate (`GATE_OUTPUT_DIR=docs/art/production10/gates/final bash tools/art_pipeline/gate.sh final`); fast-forward `codex/art-production-2026-10`; PR #511 section; remove `/Users/rexxa/work/art-p10-base`.
3. `docs/art/production10/REPORT.md` in Korean (section 12 format) and the same text in the final message.

---

# 10. Localization keys
Add in the Part that first uses them, in all five files (`localization/fluent/{en,ko}/messages.ftl`, `localization/compiled/{en,ko}.json` with counts, `localization/key_registry.json` with the next ids and counts). **Reuse these existing keys** (same meaning): `UI_FAMILY` (Family / 가족), `PANEL_FAMILY_SPOUSE` (배우자), `PANEL_FAMILY_NONE`, `UI_MOTHER` (모), `UI_FATHER` (부), `UI_FEMALE` (여), `UI_MALE` (남). The existing `TEMPERAMENT_*` keys are the four classical temperaments — a different meaning; do not reuse them. New keys:
| key | en | ko |
|---|---|---|
| `UI_ACTIVITY_TEND_FIRE` | Tending the fire | 불을 지키는 중 |
| `UI_ACTIVITY_CRAFT` | Making things | 무언가 만드는 중 |
| `UI_ACTIVITY_REPAIR` | Mending a shelter | 쉼터를 손보는 중 |
| `UI_ACTIVITY_HEARTH_CIRCLE` | Sitting at the hearth | 화덕가에 둘러앉음 |
| `UI_ACTIVITY_EXPLORE` | Looking around afar | 멀리 둘러보는 중 |
| `UI_ACTIVITY_PLAY` | Playing | 노는 중 |
| `UI_ACTIVITY_SIT_BY_FIRE` | Resting by the fire | 불가에서 쉬는 중 |
| `INSPECTOR_TARGET_ACTIVITY_PLACE_FMT` | Place (%d, %d) | 장소 (%d, %d) |
| `UI_INSPECTOR_TEMPERAMENT` | Temperament | 기질 |
| `UI_TEMPERAMENT_NS` | Curiosity | 호기심 |
| `UI_TEMPERAMENT_HA` | Caution | 신중함 |
| `UI_TEMPERAMENT_RD` | Warmth | 정 |
| `UI_TEMPERAMENT_P` | Persistence | 끈기 |
| `UI_FAMILY_CHILDREN_FMT` | Children: %d | 자녀: %d명 |
Grep for each new key first; if one exists, reuse it and record.

---

# 11. 인게임 확인사항 (오너용)
- 사람이 많아져도(마을이 셋으로 갈라진 뒤, 또는 붐비는 세계) 화면이 끊기지 않는지. 이전과 같은 장면은 그림이 똑같은지.
- 낮에 사람들이 각자 무언가 하는 모습이 보이는지: 불 앞에 쪼그려 앉은 사람, 앉아서 만드는 사람(도구 아이콘), 쉼터 앞에서 망치질하는 사람, 화덕 둘레에 둘러앉은 사람들, 멀리 나가 서 있는 사람(표시 아이콘), 뛰어노는 아이들, 불가에 앉은 노인.
- 주민을 클릭하면 인스펙터에 지금 하는 활동 이름이 나오고, 기질 막대 4개(호기심·신중함·정·끈기)와 가족(성별, 배우자, 모, 부, 자녀 수)이 보이는지. 배우자나 자녀 이름을 누르면 그 사람이 선택되는지.
- 한 사람을 선택하면 그 배우자와 아이들에게 옅은 고리가 생기는지.
- 콘솔 에러, 번역 키가 그대로 노출되는 곳이 없는지(영어·한국어 모두).

---

# 12. 구현 완료 보고 (REPORT.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
### 구현 내용
(렌더러 수정, 활동 포즈와 아이콘, 인스펙터 기질·가족, 가구 강조)
### 구현 방법
(구간 타이머, 틱 캐시와 무효화, 공간 해시, 픽셀 동일성, 활동 매핑, 블라인드 식별 절차)
### 기능 설명
### 변경된 파일 목록
(커밋별)
### 측정 결과
H1~H8 표(전후·배수), 세계·화면·조건별 중앙값/p90/p99/최대, AgentRenderer 구간 타이머 전후, 노드별 비용 전후, 블라인드 식별 정답률(활동별)
### 환경
### 독립 리뷰
### 자율 결정 목록
### 실행 기록
### 확인된 제한사항
### 게이트 결과
### 정직한 판단
화면은 몇 명까지 버티는가? 낮의 활동이 눈으로 읽히는가? 다음 아트 과제는 무엇인가?
```
Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the PR #511 update, and the time per Part.