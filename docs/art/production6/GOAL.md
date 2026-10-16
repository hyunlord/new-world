# GOAL: WorldSim — Art Pass 6: cold you can see where people are, the god's hand on screen, loads you can tell from meals, a village that does not pile up

Repository `hyunlord/new-world`. Continue the art line: `codex/art-production-2026-10-wip` and `codex/art-production-2026-10`, both at `ca47e703f7136a00cac83eb242fbec8dccc77d52` (Art Pass 5; draft PR #511, stacked on the simulation PR #510).
The simulation's Pass 8 is sealed: `codex/revitalize-2026-10` and `-wip` are both at `4c35e29573bc28878de0cfed9ac67e0df0aa4c48` (gate green). Since Art Pass 5's base (Pass 6, `fe60de21`) it adds: `game_v13` (Pass 7: a severity drawn per winter, claims bounded by what a spot holds, seven god interventions behind one FFI command) and `game_v14` (Pass 8: the burn changes night by night and the deadwood fall day by day; windfall lands on the snow). The live game builds `game_v14` after the merge. No simulation run is active during this pass; a learned-model run (WM-1) works in another clone and on the DGX and touches nothing of yours.
This run is executed by **Codex**, unattended, on the Mac.

---

# ⚠️ 0. AUTONOMY — READ FIRST

1. **Never ask the owner anything.** Decide, implement, record the decision and its reason in `docs/art/production6/DECISIONS.md`.
2. **First action:** save this document unchanged as `docs/art/production6/GOAL.md` (the owner may already have placed it there; then verify it is this document), commit it in Part 0, and re-read the section of the current Part at every Part start and after any context compaction.
3. **Do not stop between Parts.** One task per Part; push to the WIP branch after every Part.
4. **Review loops: two rounds maximum.** Fix once, re-review once; then accept, record, move on.
5. **On failure: work around, note it, continue.** Partial delivery with an honest report beats a stalled run.
6. **Time rule:** record start time and each Part's start and end in `docs/art/production6/TASKS.md`. Art Pass 5 took 1 h 39 min. Past **8 hours**, finish the current step, push, go to Part 9.
7. **Three strikes** per failing target, then keep the best state and record evidence and hypothesis.
8. **Drop order when time is short:** the cold-snap and windfall effects of Part 5 (keep their buttons and replies) → the morning chill of Part 2 → the generated god-hand icons (use text buttons) → the second blind-test round. Never drop Parts 0, 1, 3, 4, 7, 8, 9, the god's hand buttons with their replies, or the rain effect.
9. **Image budget: at most 6 new generations**, failures included. On a quota or rate-limit message, stop generating and continue.
10. **Honesty rules stay:** never mark a failed check as passed, never lower a threshold of section 12, keep rejected evidence, never edit or re-interpret an earlier verdict. Captures that contain a god intervention carry `INTERVENTION SCENARIO` in their metadata and the `god_` prefix in their file names, and never count as natural evidence.
11. **This brief supersedes the repository's session-start instructions** (harness hook installation, the `lead/main` workflow, `harness_pipeline.sh`): do not install hooks, do not run the harness pipeline, never push to `lead/main` or `main`, never force-push.

---

# ⚠️ 1. WHERE AND HOW

- Mac only, the existing clone `/Users/rexxa/work/new-world-art-prod`. Run every pipeline script from the repository root. New raw generations go to `/Users/rexxa/work/art-production6-raw/<sheet_id>/attempt_<n>.png`; capture evidence to `CAPTURE_EVIDENCE=/Users/rexxa/work/art-production6-evidence`. Earlier raw folders are read-only. Raws never enter git.
- Never touch `/Users/rexxa/work/new-world-pass4` (simulation), `/Users/rexxa/work/new-world-wm1` (WM-1), `~/github/new-world-wt/lead`, `/Users/rexxa/work/new-world-capture` or any other checkout; never kill a process you did not start; no ComfyUI, no dev servers, no DGX.
- `export CARGO_BUILD_JOBS=6`; cargo tests `-- --test-threads=6`. `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`. Pipeline Python uses Pillow and numpy only.
- Windowed Godot only through `tools/art_pipeline/with_window_lock.sh <seconds> <command...>` in one command (from a background session inside `launchctl asuser $(id -u) …`, still inside the lock; each locked command under 25 minutes). Headless Godot needs no lock; other limits use `tools/art_pipeline/run_bounded.sh`.
- Image generation: the built-in `$imagegen` skill, magenta background. Prompts never name existing games, studios, artists or characters. Villager-prop prompts use the Art Pass 4 form (`docs/art/production4/wood/prompts/V_a4.txt`); icon prompts ask for a 4 × 4 grid of flat 16 × 16 pixel icons on magenta with a 1-px dark outline, in the D3 style reference's palette.
- Scripts of earlier passes that write under `docs/art/production5/**` are copied to `_p6` versions writing under `docs/art/production6/**`, never called as they are.
- **Independent review** means a fresh reviewer session or subagent that has not seen the pipeline code or sections 3–10 of this brief. The blind activity test keeps Art Pass 3's procedure (answer file frozen first; observers get only the images and the allowed labels).

---

# 2. GIT

- `git fetch origin && git checkout codex/art-production-2026-10-wip`; it must equal `origin/codex/art-production-2026-10-wip` (`ca47e703`). Local-only commits (for example Art Pass 5's unpushed compatibility merge `631434e3`) go to a backup branch `art-p6-local-backup` first.
- **Part 0 merges the simulation green head** `4c35e29573bc28878de0cfed9ac67e0df0aa4c48` exactly: `git merge --no-ff 4c35e295 -m "art-p6(p0): bring in simulation passes 7 and 8"`. The planner's trial merge has **one conflict**, whitespace only, in `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs` (the same blank-line conflict Art Pass 5 resolved locally). Keep the art side's file (content assertions, no hash pin) and delete the import `use sim_test::worldmodel::sha256::sha256_hex;` that the merge brings in with no remaining caller (the gate's `clippy --all-targets -D warnings` would fail on it); ledger row. Elsewhere: under `rust/crates/sim-*/src/**`, `docs/revitalize/**`, `docs/worldmodel/**`, `tools/revitalize/**` and `tools/worldmodel/**` take the simulation's side; record every resolved file. Merge no later simulation head in this pass.
- WIP gets every commit; `codex/art-production-2026-10` moves only by fast-forward to a WIP commit whose full gate (Part 9) is green. Never force-push. After every push `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`.
- Commit messages `art-p6(pN): <what changed>`; stage explicit paths only.
- **May change:** `scripts/**`, `scenes/**`, `shaders/**`, `assets/**`, `localization/**`, `project.godot`, `docs/art/**` (new files only under `docs/art/production6/**`, plus `docs/art/TEST_LEDGER.md`), `tools/art_pipeline/**`, Rust static-source tests under `rust/crates/sim-test/tests/` that read `.gd`/`.tscn`/`.gdshader` files (ledger row each under `## Art Pass 6`).
- **One narrow exception in simulation-owned code (Part 5.1):** one additive, read-only FFI getter in `rust/crates/sim-bridge/src/ffi/world_node.rs` with its pure collector and unit test. Nothing else under `rust/` outside the tests above.
- **Must not change:** everything else under `rust/`, `docs/revitalize/**`, `docs/worldmodel/**`, `tools/revitalize/**`, `tools/worldmodel/**`, `tools/harness/**`, `.claude/**`, `.harness/audit|baseline|reviews/**`, and evidence under `docs/art/production{,2,3,4,5}/**`.

---

# 3. WHY THIS PASS (planner's review of ca47e703, with evidence)

1. **The cold is real but invisible, by design error.** Art Pass 5 reported "natural cold not captured". The planner opened its own capture `docs/art/production5/shots/l_d29_0200_F2N.entities.json.gz`: tick 39,960 (02:00 of display day 29, the last night of the first winter), settlement fire out, wood stock 0, `colds` = 1 for **42 of 42** villagers, the HUD reading "추위 42" — and all 42 in state Consuming/Sleep inside shelters, so `agent_renderer.gd` hides them (`sheltered_sleep`). The cold window (22:00–06:00) is sleeping time: body-level cold cues (crossed arms, shiver, breath) can never be seen in natural play. The planner's Art Pass 5 brief put the cue on the wrong object. This pass puts it where the people are: on shelters and the hearth at night, on bodies the morning after.
2. **The night is labelled with the wrong season.** In the same frame the HUD reads "29일차 02:00 · 밤 · 봄 1일째": the simulation books that night to the winter evening it began (`winter_severity` in the time snapshot is ≥ 0 until the burn ends at 07:00), but the label shows the calendar's spring. Everyone freezes under a "spring" label.
3. **At 2× a load looks like a meal.** Blind test at 2×: all three carry crops (`V_b2_04`, `V_a2_04` ×2) were labelled "eat", and two of three eat crops "carry". In both, a small red-brown object sits at chest height inside the body outline (`docs/art/production5/blind/blind_2x.png`, numbers 11, 13, 20 vs 8, 14, 15). 2× accuracy 0.583, 4× 0.875.
4. **The installed wood-carry art was never seen.** A17: one wood carrier visible in all captures, using a fallback; real `carry_wood` frames observed: 0. The captures never framed autumn afternoons, when the live seed sends 14 gatherers and 9 haulers out at 15:00 (days 15–16).
5. **The winter village still piles up.** A18 at d24: 3 of 37 villagers over-occluded (limit 1).
6. **The god has no hands on screen.** Pass 7 added `apply_intervention(kind, settlement_id, amount) -> {ok, applied, tick}` for seven kinds; the game shows nothing of it.

## 3.1 Facts of the live seed under `game_v14` (planner's probe, seed 20,261,010, the `DEFAULT_GAME_SEED` the live game and the captures use)
- First winter (display days 22–28, severity normal): the fire is out with every member cold at 02:00 of **days 28 and 29**; lit at 02:00 of days 23–27.
- Second winter (harsh): cold at 02:00 of days 54–57. Third winter: days 84–85.
- Autumn wood gathering at 15:00: days 15 and 16 (14 + 9 and 8 + 7 gatherers + haulers), none from day 17.
- Re-measure these with the merged build in Part 0 (headless `advance_ticks` or the probe) and use your measured days if they differ; record both.

---

# 4. PART 0 — Merge and baseline
1. Backup branch, merge (section 2), record conflicts and resolutions.
2. Full gate (`GATE_OUTPUT_DIR=docs/art/production6/gates/p0 bash tools/art_pipeline/gate.sh p0`); fix merge fallout in art-owned paths only.
3. Record that the live game builds `game_v14`, that `get_time_snapshot` carries `winter_severity`, and that `apply_intervention("gift_food", -1, 10)` returns `ok` true once a settlement exists (a fresh world has none: advance with `advance_ticks` until `get_settlement_snapshot().settlement_ids` is non-empty; before that the reply is `ok` false). Re-measure 3.1.
4. Baseline captures of the frames named in Part 7 that exist at the merged head before any art change (for before/after). Commit, push.

---

# 5. PART 1 — Cold where the people are (night)

## 5.1 Which shelter is cold
A shelter is cold when at least one villager with `colds == 1` is in Consuming/Sleep on that shelter's tile (agent `xs, ys` = the building's tile; building snapshot `kinds == 1`). Compute this in GDScript from the agent and building snapshots each frame (read-only; no simulation state).

## 5.2 Cues (procedural from existing art; no generation needed)
- **Frosted shelter:** a frost variant of each shelter sprite, made by a deterministic pixel pass over its current frame — every opaque pixel in the top 40 % of the sprite whose up-neighbour is transparent becomes the frost colour `#D8E8F0`, and every second such roof pixel one row lower becomes `#A8C0D0`; no other pixel changes. Shown while the shelter is cold; fades in and out over 0.4 s real time.
- **Cold breath from the doorway:** a 3 × 2 pale puff (the existing `ColdBreath` texture) rising from the shelter's doorway pixel every 2.4 s, offset per shelter id.
- **Cold hearth:** when the settlement's fire is out on a winter night (`fire_lit` 0 and `winter_severity` ≥ 0 between 18:00 and 07:00), the hearth shows the existing `S1_16` "cold_ring" frame (already chosen by `wood_visuals.gd` when wood is 0) also when wood is above 0 but the fire is out (rain), with a 3-px pale-grey ash patch `#8A8A8A` at its centre, and **no smoke** (`world_effects.gd` currently draws smoke for every hearth).
- **Night chill tint:** while any shelter is cold, a screen-space tint toward `Color(0.80, 0.88, 1.0)` at strength 0.12, eased over 1 s. Never stronger; the daylight controller keeps working.
- The existing body cues (crossed arms, shiver, breath) stay for any awake visible villager with `colds == 1`.
- Metadata per captured frame: per shelter `cold_sleepers`, `frost_alpha`; per hearth `cold_ash`; the tint strength.

## 5.3 Tests
`test_shelter_cold.gd` (a shelter with one cold sleeper frosts, one with only warm sleepers does not, a cold villager awake elsewhere does not frost a shelter; the frost pass changes only edge roof pixels; fade timing), `test_cold_hearth.gd` (cold ash only on winter nights with the fire out, including the night booked to winter after the calendar turned to spring). Commit, push.

---

# 6. PART 2 — The morning after
- UI-side memory (presentation only): for each villager, the last tick its `colds` flag was 1. Chill cues write their own metadata keys (`chill_alpha`, `chill_since_tick`), never the cold keys (`cold`, `shiver_offset`, `puff_alpha`). A villager awake and visible within **90 game minutes** after its flag cleared shows the chill cues (crossed arms when idle/sitting/standing, shiver, breath), fading out over the last 30 of those minutes; meal and work poses win as they do for cold.
- The memory is cleared on `reset_world` and never persisted; it never reaches the simulation. It must be fed by every agent snapshot the UI reads: the capture tool advances the world synchronously between shots, so `capture_framings.gd:_advance_to` must pass each advance chunk's agent snapshot (`colds`) to the memory (at most every 60 ticks), or the morning frames would see only the previous shot's flags.
- `test_morning_chill.gd` (cleared flag at 06:00 → cues until 07:30, gone at 07:31; a villager never cold → no cues; reset clears). Commit, push.

---

# 7. PART 3 — The HUD tells the night's truth
1. **Night season:** when `winter_severity` ≥ 0 and the calendar `season` is not winter (the booked last winter night, 00:00–07:00 of the first spring day), the season part of the clock reads the winter's last day with a night marker: new key `HUD_SEASON_LAST_NIGHT_FMT` = en `"%s, night of day %d"` / ko `"%s %d일째 밤"`, filled with the winter's name and `days_per_season`. Update `test_hud_season_label.gd`.
2. **This winter:** while `winter_severity` ≥ 0 (every winter hour except 00:00–07:00 of the first winter day, which is booked to the autumn evening, plus the booked last night) the top bar shows the severity name with keys `HUD_WINTER_MILD` "Mild winter"/"온화한 겨울", `HUD_WINTER_NORMAL` "Winter"/"보통 겨울", `HUD_WINTER_HARSH` "Harsh winter"/"혹독한 겨울". Hidden when `winter_severity` is −1. (The god may know; the villagers do not — nothing else reads it.)
3. **Cold count:** the existing "추위 N" gains the snowflake icon of 9.4 (or a text fallback) and the cold colour `#A8C0D0` while N > 0.
4. Tests for each. Commit, push.

---

# 8. PART 4 — A load is not a meal at 2×

## 8.1 The rule (measured on the native sprite)
Rows are measured from the top of the frame's opaque bounding box (0 % = top of the head). A carry pose's load must (a) break the body's outline: at least 2 opaque load pixels outside the torso columns (the x-range of the clothing region at the frame's middle row, from its region map) on one side, and (b) lie between 50 % and 80 % of the height (waist to thigh), with (c) a 1-px dark outline where it meets transparency. An eat pose's food stays within the top 45 % (at the mouth). Write `tools/art_pipeline/load_silhouette_p6.py` that measures (a)–(c) on every installed carry and eat frame and reports per frame.

## 8.2 The fix, cheapest first
1. **Composited load layer:** for carry frames that fail (a)–(c), the renderer draws the load as a separate sprite: a basket (food) or a bundle (wood) built from the frame's own load pixels (cut from the frame by its existing region map or by colour within the load area), enlarged to at least 7 × 5 native pixels with a dark outline, anchored to the carrying hand at hip height on the facing side, flipped with the body, drawn in the body's z-order. The body frame keeps its hands; the original small load pixels are covered by the composited one. This is an attached prop, not a floating icon.
2. **Only if review rejects (1):** generate carry props or carry frames (at most 4 of the 6 images), through the Art Pass 4/5 install path with its guards, reviews and binding.
3. Review: an independent reviewer judges every carry group at 1×, 2× and 4× (`ok` / `floating` / `wrong_object` / `reads_as_eat`) on boards in `docs/art/production6/carry/boards/`. Install only `ok` groups.

## 8.3 Tests
`test_carry_load_layer.gd` (anchor, flip, z-order, food vs wood, hidden when not carrying). Commit, push.

---

# 9. PART 5 — The god's hand

## 9.1 One read-only FFI getter (the section 2 exception)
`#[func] fn get_intervention_state(&self) -> VarDictionary` built on a pure `collect_intervention_state(resources: &SimResources, now: u64) -> InterventionState` with a unit test, additive only:
- `rain_settlement_ids: PackedInt32Array`, `rain_ticks_left: PackedInt32Array` (from `fire_blocked_until`, only entries still in the future, by settlement id);
- `cold_snap_nights_left: int`, `cold_snap_tonight: bool`;
- the last 32 records of `intervention_log`, oldest first: `log_ticks: PackedInt64Array`, `log_kinds: PackedStringArray` (the names of `sim_systems::runtime::intervention::KINDS`, found by matching on the `Intervention` variant), `log_amounts: PackedInt64Array` (the variant's amount, hours, nights or units — `InterventionRecord` has no amount field), `log_applied: PackedInt64Array`.
Record it in DECISIONS.md and in the report's compatibility section (the next simulation pass inherits it). Run `cargo test`, Clippy and the bridge build after adding it.

## 9.2 Panel (`scripts/ui/panels/god_hand_panel.gd` + scene)
- A compact panel anchored bottom-right in the main UI layer (new `god_hand_panel.gd` + `.tscn`; the current HUD has no bottom bar), seven buttons, each with an icon (generated sheet of 9.4, else text) and a tooltip; one click casts on the home settlement (`settlement_id` −1) with these presets: gift food 60, spoil food 60, gift wood 40, rot wood 40, rain 13 hours, cold snap 1 night, windfall 20.
- After the call, a toast shows the localized kind, the amount and `applied` ("비 13시간 — 적용 13"); `ok` false shows `GOD_HAND_FAILED`. Buttons are disabled while no settlement exists.
- A small log list (last 8 records from 9.1) under the panel, collapsed by default.
- The panel never writes simulation state except through `apply_intervention`.

## 9.3 Effects (read from 9.1 and the snapshots; presentation only)
- **Rain:** while a settlement's `rain_ticks_left` > 0, rain streaks (procedural 1 × 3 px lines, `#9FB4C8` at alpha 0.5, 1 per 24 × 24 screen px, falling at 120 px/s) over a circle of 12 tiles around its formation tile, and the hearth shows the cold hearth of 5.2. Required (never dropped).
- **Cold snap:** while `cold_snap_tonight`, the night chill tint of 5.2 at strength 0.18 and the HUD line `GOD_HAND_COLD_SNAP_ACTIVE` with nights left.
- **Windfall:** after a cast, deadwood spots whose amount rose since the previous resource snapshot play a 0.6 s fall (the pile sprite drops 6 px with a 1-px bounce).
- **Gift / spoil / rot:** the settlement's stockpile label flashes for 1.2 s, green `#7CC46A` for gifts and brown `#8A6A3A` for spoil/rot, with a 4-particle burst (procedural).
- Tests: `test_god_hand.gd` against a stub node with the same methods (seven buttons with localized labels and presets; each click calls `apply_intervention` with the preset; replies and failures shown; effects start and stop from a scripted `get_intervention_state`), and one headless script on the real node that first advances until a settlement exists, then casts each kind once and asserts `ok` and a non-negative `applied`.

## 9.4 Icons (optional, at most 2 of the 6 images)
One 4 × 4 icon sheet (7 icons + snowflake + spares), normalized to 16 × 16 on the production palette, reviewed (`ok` / `unclear` / `off_style`); unclear icons fall back to text.

## 9.5 Localization (en / ko, every key in both)
| key | en | ko |
|---|---|---|
| GOD_HAND_TITLE | God's hand | 신의 손 |
| GOD_HAND_GIFT_FOOD | Gift food | 식량 내리기 |
| GOD_HAND_SPOIL_FOOD | Spoil food | 식량 썩히기 |
| GOD_HAND_GIFT_WOOD | Gift wood | 장작 내리기 |
| GOD_HAND_ROT_WOOD | Rot wood | 장작 썩히기 |
| GOD_HAND_RAIN | Rain | 비 |
| GOD_HAND_COLD_SNAP | Cold snap | 한파 |
| GOD_HAND_WINDFALL | Windfall | 바람 |
| GOD_HAND_APPLIED_FMT | %s %s — applied %d | %s %s — 적용 %d |
| GOD_HAND_FAILED | Nothing happened | 아무 일도 일어나지 않았다 |
| GOD_HAND_COLD_SNAP_ACTIVE | Cold snap: %d nights | 한파: %d밤 |
| GOD_HAND_LOG | Recent acts | 최근의 손길 |
| HUD_SEASON_LAST_NIGHT_FMT | %s, night of day %d | %s %d일째 밤 |
| HUD_WINTER_MILD | Mild winter | 온화한 겨울 |
| HUD_WINTER_NORMAL | Winter | 보통 겨울 |
| HUD_WINTER_HARSH | Harsh winter | 혹독한 겨울 |
Units in the toast ("60", "13시간", "1밤") use existing number formatting plus `GOD_HAND_UNIT_HOURS` "h"/"시간" and `GOD_HAND_UNIT_NIGHTS` "nights"/"밤" (add both). Check for duplicates before adding. Commit, push.

---

# 10. PART 6 — A winter village that does not pile up
1. Find the cause of A18's three over-occluded villagers at d24 (metadata of `l_d24_1200_F2`): which activities, where, which declump rule skipped them. Record it.
2. Extend Art Pass 5's seats and declump (front-to-back, manifest canvas rects, shifts 1–8 px, never onto water, walkers untouched) to the cases found (expected: idle winter crowds near the hearth by day). The target is A18 ≤ 1 in every captured frame, d24 included.
3. Extend `test_declump.gd` with the d24 arrangement (positions from the metadata). Commit, push.

---

# 11. PART 7 — Captures
## 11.1 Plans (`scripts/test/art/capture_plans.gd`)
- `ART` unchanged. `ART_LONG` = Art Pass 5's list, plus: autumn carriers `[15, "15:00", "F2"]`, `[15, "15:30", "FC"]`, `[16, "15:00", "FC"]` with a new framing `FC` in `scripts/test/capture_framings.gd` (zoom 4, centred on the lowest-id walking villager with `carried_woods` > 0; fails with a clear message when there is none, like F3 and F5); the cold nights `[28, "02:00", "F2N"]`, `[28, "02:00", "F2Z"]` (zoom 6 on the shelters), `[29, "02:00", "F2N"]`; the morning after `[29, "06:30", "F2"]`, `[29, "07:15", "F2"]`; the control morning `[27, "07:15", "F2"]`. Use the days measured in Part 0 if they differ. Chronological order.
- New `ART_GOD` (a separate world, every file name prefixed `god_`, metadata `INTERVENTION SCENARIO`), in chronological order in one forward-only world (effects accumulate; record the full cast list in every frame's metadata): spoil food 60 at d10 12:00 → frame at +0.5 s real time; cold snap 1 at d16 17:00 → d17 02:00 F2N; gift wood 40 at d23 12:00 → frame at +1 s; windfall 20 at d23 12:05 → frames at +0.2 s and +1 s; rain 13 h at d24 17:00 → frames d24 18:30 F2 and d25 02:00 F2N. Plus one panel screenshot with the log open. Add `art_god` to the capture tool's plan whitelist.
- Metadata gains: shelter cold fields, hearth cold ash, tint, morning-chill fields per villager, load-layer fields per carrier, intervention state.
## 11.2 Run
`art`, `art_long`, `art_god` into `docs/art/production6/shots`; `calibration` into `docs/art/production6/calibration`.

---

# 12. PART 8 — Measure (thresholds unchanged unless stated)
- A1–A10, A12–A16, A19, A20 as in Art Pass 5 (A19's body clause stays; report its N/A honestly), except that A19's "no cue on a villager without the marker" clause exempts villagers inside the morning-chill window, identified by the separate chill metadata keys of Part 2 (`chill_alpha`, `chill_since_tick`); chill cues never write the cold keys. The `_p6` copies of `measure_p5.py` and `ingame_qa*.py` update their pinned capture-set lists to the new plans, and A17's carrier scan reads every natural frame, not only `l_d10_1500_F2`.
- **A11 (raised: 4× from 0.80 to 0.85, 2× added):** blind test rebuilt with 24 crops (at least 3 carry and 3 eat, from this pass's frames); 4× accuracy ≥ 0.85, **2× accuracy ≥ 0.70 and carry identified at 2× in ≥ 2 of 3**. Two observers; report both and the confusion matrices.
- **A17:** as Art Pass 5, plus real `carry_wood` frames observed on at least 3 visible wood carriers from at least 2 bodies across the natural captures; carriers on a fallback are listed with the reason.
- **A18:** ≤ 1 over-occluded villager in **every** captured natural frame.
- **A21 cold on shelters (new):** in the natural cold-night frames (d28, d29 02:00), every visible shelter with ≥ 1 cold sleeper is frosted and its doorway puffs within 2.4 s; no shelter is frosted in the lit nights (d23, d25, d27 02:00); the hearth shows cold ash exactly when the fire is out on a winter night.
- **A22 morning chill (new):** at d29 06:30 and 07:15 every visible awake villager whose flag cleared ≤ 90 game minutes earlier shows the chill cues; at d27 07:15 none does.
- **A23 HUD truth (new):** d29 02:00 shows the winter's last night (not spring), the winter line shows the severity in frames with `winter_severity` ≥ 0 and is absent otherwise; "추위 N" matches the snapshot's count in every frame.
- **A24 god's hand (new):** the 9.2–9.3 tests pass; in `ART_GOD`, every cast returns `ok`, its effect is present in the next frame's metadata (rain streaks over the settlement and cold ash; label flash; windfall fall; cold-snap tint), and no natural frame shows any intervention effect.
- **A25 the cold is visible (new):** in 100 % of natural frames of a dark winter night (fire out, `colds` > 0), at least two on-screen cold cues among frost, doorway breath, cold ash, tint and the HUD count are visible (Art Pass 5: the HUD count only).
- Before/after against Art Pass 5 at the same framings (d29 02:00 F2N, d2 12:00 F2, a carry crop at 2×), `cold_night.png` from a **natural** capture this time (F2Z at d28 02:00), `god_hand.png` (the panel and four effects, labelled INTERVENTION SCENARIO), `carry_2x.png` (every carry group at 2× before and after, beside the eat poses), the season strip and the two GIFs. Commit, push.

---

# 13. PART 9 — Final gate, compatibility, report, PR
1. Full gate: `GATE_OUTPUT_DIR=docs/art/production6/gates/final bash tools/art_pipeline/gate.sh final`. Fast-forward `codex/art-production-2026-10` if green; otherwise say exactly what is red.
2. Budgets: `assets/art/**` ≤ 25 MB, `docs/art/production6/**` ≤ 30 MB (JPEG unless a check needs lossless).
3. An independent review of the final screens (natural cold night, morning after, carry at 2×, god's hand) with verdicts `ACCEPT` / `NEEDS WORK` and reasons, recorded as given.
4. `docs/art/production6/REPORT.md` in Korean in the section 15 format, answering every item of section 14 with its evidence.
5. Update PR #511's description with an Art Pass 6 section (`gh pr edit 511 --body-file …`; say so if `gh` is not authenticated). Keep it a draft.

---

# 14. 인게임 확인사항 (게임을 직접 켰을 때 확인할 것)

1. 겨울 마지막 밤(새 게임 기준 28·29일차 02:00 무렵) 불이 꺼지면 오두막 지붕에 서리가 끼고, 문간에서 입김이 오르고, 화덕은 불씨 없는 재로 바뀌며 화면이 살짝 푸르게 식는다. 불이 켜진 밤에는 그런 모습이 없다.
2. 그 밤의 시계가 "봄 1일째"가 아니라 "겨울 7일째 밤"으로 나오고, 겨울 동안 상단에 "온화한 겨울 / 보통 겨울 / 혹독한 겨울"이 보인다. 추위 수 옆에 눈송이가 붙는다.
3. 추운 밤을 보낸 아침(29일차 06:30~07:30), 일어난 주민들이 잠시 팔짱을 끼고 떨며 입김을 낸다. 따뜻한 밤 다음 아침에는 그렇지 않다.
4. 2배 줌에서 바구니나 장작을 든 주민과 먹는 주민이 구분된다. 짐은 허리춤에서 몸 밖으로 삐져나오고, 먹을 것은 입가에 있다.
5. 가을 오후(15~16일차) 장작을 메고 돌아오는 주민이 실제 장작 운반 그림으로 걷는다.
6. 하단의 "신의 손"에서 일곱 가지를 누를 수 있고, 누르면 결과 알림이 뜬다. 비를 내리면 마을 위로 빗줄기가 보이고 화덕이 꺼진다. 바람은 가지 더미를 떨어뜨리고, 선물·부패는 저장소 라벨이 번쩍인다.
7. 겨울 낮 화덕 주변 주민들이 심하게 겹치지 않는다.
8. 콘솔에 SCRIPT ERROR가 없고 프레임 시간이 Art Pass 5와 비슷하다.

---

# 15. 구현 완료 보고 (REPORT.md와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
오두막·화덕 단위의 추위, 다음 날 아침의 한기, 밤의 계절 표기와 겨울 혹독함 줄, 짐 실루엣 규칙과 합성 짐 레이어, 신의 손(FFI 조회 함수·패널·효과), 겨울 떼어 놓기

### 실제 게임 화면 검증
A1~A25 표(수치), 블라인드 식별 혼동 행렬(관찰자 2명), 자연 장면과 개입 장면의 구분, before/after·추운 밤(자연)·신의 손·2배 운반·계절 스트립·GIF 경로

### 시뮬레이션 작업과의 호환
병합한 시뮬레이션 커밋과 충돌 해결, 추가한 FFI 조회 함수(다음 시뮬레이션 작업이 이어받을 것), v14에서 측정한 추운 밤·가을 운반 날짜

### 변경된 파일 목록
(커밋별)

### 자율 결정 목록
DECISIONS.md 요약

### 확인된 제한사항

### Harness 결과
(cargo test 수치, clippy, strict, bridge, smoke, Python, Godot 아트 장면 테스트)

### 정직한 판단
처음 보는 사람이 추운 밤과 신의 손을 화면만 보고 알아챌까? 아직 아닌 부분은 무엇인가?
```

Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the simulation commit merged, whether PR #511's description was updated, image calls used, and the paths of the before/after, `cold_night.png`, `god_hand.png`, `carry_2x.png`, the season strip, the GIFs and the review boards.
