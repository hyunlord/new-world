# GOAL: WorldSim — Art Pass 8: the newest world on screen — a restless village you can see coming, news worth reading, a destination you can find, and the four visual failures that keep coming back

Repository `hyunlord/new-world`. Continue the art line: `codex/art-production-2026-10-wip` = `codex/art-production-2026-10` = `a54486e8075c502964fffb48120fd2d61a825d50` (Art Pass 7 plus its gate hotfix; draft PR #511, stacked on the simulation PR #510).
The simulation line has moved on twice since the art line's last merge (`31559232`, Pass 11, `game_v17`): Pass 12 (`game_v18`: fission as a daily hazard, regions that deliver their bushes and water, rain that burns wood) and Pass 13 (`game_v19`: fission when a village's accumulated pressure reaches a hidden threshold θ of its own; the same seed now splits on nearly the same day, and the god's food acts nudge that day by a few days). The live game builds `game_v19` after this pass's merge.
This run is executed by **Codex**, unattended, on the Mac. No simulation run is active during this pass; a learned-model run may compute on the DGX and touches nothing of yours.

**Simulation commit to merge:** `2ba871c09819c240904510bbf26b0fb2a59a79b0` (Pass 13's green head, gate green; the WIP head `a12d7a35` after it changes docs and probe outputs only).

---

# ⚠️ 0. AUTONOMY — READ FIRST

1. **Never ask the owner anything.** Decide, implement, record the decision and its reason in `docs/art/production8/DECISIONS.md`.
2. **First action:** save this document unchanged as `docs/art/production8/GOAL.md` (or verify the owner's copy is this document), commit it in Part 0, and re-read the section of the current Part at every Part start and after any context compaction.
3. **Do not stop between Parts.** One task per Part; push to the WIP branch after every Part.
4. **Review loops: two rounds maximum.**
5. **On failure: work around, note it, continue.**
6. **Time rule:** record start time and each Part's start and end in `docs/art/production8/TASKS.md`. Past **10 hours**, finish the current step, push, go to Part 9.
7. **Three strikes** per failing target, then keep the best state and record evidence and hypothesis.
8. **Drop order when time is short:** Part 4's A22 morning-chill redesign (8.2) → Part 4's A24 world effects (8.3) → the restlessness meter of Part 3 (keep the getter fields) → A18's human-judged companion (A18b). Never drop Parts 0, 1, 2, 5, 6, 7, 8, 9, Part 4's A18 fix and A30 redesign, the feed rework or the off-screen destination marker.
9. **Image budget: at most 4 new generations**, failures included; prefer procedural drawing.
10. **Honesty rules stay:** never mark a failed check as passed, never lower a threshold of section 10, keep rejected evidence, never edit or re-interpret an earlier verdict. Captures with a god intervention carry `INTERVENTION SCENARIO` and the `god_` prefix; society-scenario captures carry `SOCIETY SCENARIO seed <n>` and the `soc_` prefix, reported separately.
11. **This brief supersedes the repository's session-start instructions**: do not install hooks, do not run the harness pipeline, never push to `lead/main` or `main`, never force-push.

---

# ⚠️ 1. WHERE AND HOW
- Mac only, the clone `/Users/rexxa/work/new-world-art-prod`. Raw generations to `/Users/rexxa/work/art-production8-raw/…`; capture evidence to `CAPTURE_EVIDENCE=/Users/rexxa/work/art-production8-evidence`. Earlier raw and evidence folders are read-only. Raws never enter git.
- Never touch `/Users/rexxa/work/new-world-pass4`, `/Users/rexxa/work/new-world-wm1`, `~/github/new-world-wt/lead`, `/Users/rexxa/work/new-world-capture` or any other checkout; never kill a process you did not start; no ComfyUI, no dev servers, no DGX.
- `export CARGO_BUILD_JOBS=6`; cargo tests `-- --test-threads=6`. Before every full gate and every performance measurement record `uptime` and `pgrep -fl "cargo|rustc|worldsim_|Godot"`; if a foreign heavy process runs, wait (poll every 5 minutes, at most 60 minutes), then run and record it. (Art Pass 7's hotfix gate timed out at 1,800 s while the simulation pass was running and passed in 1,267 s on an idle Mac.)
- `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`. Windowed Godot only through `tools/art_pipeline/with_window_lock.sh` (as Art Pass 7); headless needs no lock; other limits `tools/art_pipeline/run_bounded.sh`.
- Scripts of earlier passes that write under `docs/art/production7/**` are copied to `_p8` versions writing under `docs/art/production8/**`.
- Independent review and blind tests: as Art Pass 7 (fresh reviewer that has not seen sections 3–8; answer files frozen first).

---

# 2. GIT

## 2.1 Branches and scope
- Start from `codex/art-production-2026-10-wip` = `a54486e8` (must equal origin); local-only commits go to `art-p8-local-backup` first.
- WIP gets every commit; `codex/art-production-2026-10` moves only by fast-forward to a WIP commit whose full gate (Part 9) is green. Never force-push. After every push `git ls-remote` must equal `git rev-parse HEAD`.
- Commit messages `art-p8(pN): <what changed>`; stage explicit paths only.
- **May change:** `scripts/**`, `scenes/**`, `shaders/**`, `assets/**`, `localization/**`, `project.godot`, `docs/art/**` (new files only under `docs/art/production8/**`, plus `docs/art/TEST_LEDGER.md`), `tools/art_pipeline/**`, Rust static-source tests under `rust/crates/sim-test/tests/` that read `.gd`/`.tscn`/`.gdshader` files (ledger row each under `## Art Pass 8`).
- **Rust exceptions (narrow):** in `rust/crates/sim-bridge/src/ffi/world_node.rs` only — (a) the art-owned getters `get_intervention_state` and `get_society_snapshot`, their collectors and **their unit tests**, which you may extend additively (Part 3) and must keep compiling and passing after the merge (Art Pass 7's gate went red because a merged simulation field broke an art-owned test fixture and the brief did not allow fixing it — this brief does); (b) nothing else under `rust/` except the static-source tests above.
- **Must not change:** everything else under `rust/`, `docs/revitalize/**`, `docs/worldmodel/**`, `tools/revitalize/**`, `tools/worldmodel/**`, `tools/harness/**`, `.claude/**`, `.harness/**`, evidence under `docs/art/production{,2,3,4,5,6,7}/**`.

## 2.2 Part 0 merge
`git merge --no-ff 2ba871c09819c240904510bbf26b0fb2a59a79b0 -m "art-p8(p0): bring in simulation passes 12 and 13"`. Check first that it contains `game_v19` (`rust/crates/sim-core/src/rules.rs`); the Pass 13 report itself is on the WIP head `a12d7a35` (read it there with `git show a12d7a35:docs/revitalize/REPORT_PASS13.md`). Conflicts: under `rust/crates/sim-*/src/**`, `docs/revitalize/**`, `docs/worldmodel/**`, `tools/revitalize/**`, `tools/worldmodel/**` take the simulation's side, keep the art getters; record every resolved file. **Planner's trial merge** (`git merge-tree` of `a54486e8` and `2ba871c0`, then a build): no textual conflicts; one compile fallout in an art-owned test — the two `FissionRecord` literals in `society_snapshot_tests` (`world_node.rs` ≈ L3407–3416) lack Pass 13's `pressure_at_split` and `threshold_at_split`; add both as `0`. With that, `cargo check --workspace --tests` is clean, sim-bridge's tests pass 23/23, and a scan of the static source pins finds nothing broken. **Build and run the full gate right after the merge** (Part 0.3), fixing fallout in art-owned code and art-owned tests (2.1 a) and in static pins (re-pinned with the same meaning, ledger rows).

---

# 3. WHY THIS PASS (planner's review of Art Pass 7 `4c120522`/`a54486e8`, with evidence)

1. **The news feed is noise.** In the society capture at day 29 12:00 the feed log holds 762 lines, 728 of them `HUD_FEED_STOCKPILE_DEPOSIT` / `_CONSUME` (`hud_status_panel.gd` L96–112: a line every time a village's food moves by ≥ 3, `FEED_FOOD_DELTA_THRESHOLD` L19). The four visible lines (`MAX_VISIBLE_NOTIFICATIONS` L13) are almost always stockpile lines, so "13 people left Village 1" is pushed out within seconds. The wording is wrong too: an ordinary meal from the store reads "기근 식량 소비" / "used Food (famine)" (`messages.ftl` L165–166), and the lines say "정착지 1" while the labels say "1번 마을".
2. **A28: the destination is off screen.** At the leader framing (zoom 4) a site 22–28 tiles away cannot be in view; 3 of 5 walk frames failed. The trail runs off the edge with nothing telling where it goes.
3. **A30 did not isolate the UI.** `_measure_frame_times` (`capture_framings.gd` L541–566) times the wall clock between `frame_post_draw` signals (it includes vsync and display pacing), and it compared the 1-settlement run at day 28 (35 villagers, median 9.4–9.8 ms) with the 3-settlement run at day 85 (50 villagers, samples 15.3–16.5 ms, clustered just under the 16.7 ms period of 60 Hz) under a load average that changed from 176.7 to 36.2. Population, tick, load and pacing are confounded; the +62 % cannot be attributed to the village layers.
4. **Four failures keep coming back** (A18 crowds since Art Pass 5; A22 morning chill and A24 windfall/food effects since Art Pass 6). A18 (`docs/art/production7/measurements.json`, code `A18`, `detail.frames`; "over-occluded" = ≥ 0.5 of a villager covered by villagers drawn in front, at most 1 per frame, `tools/art_pipeline/ingame_qa_pixels_p7.py` L64–99): 23 of 117 natural frames, worst 21 of 29 — and the planner's reviewer found that in the worst frames most over-occluded villagers are **walkers** (`l_d29_0630` 19 of 21, `l_d29_0715` 11 of 14, `l_d27_0715` 6 of 7: the morning stream out of the shelters), which the declump contract leaves untouched; 19 of the 23 failures are `a_strip_*` / `a_evening_*` frames whose per-frame entity files are only in Art Pass 7's raw evidence. A22: no observer identified a shiver; the warm-control morning was not warm. A24: spoil food and the wood acts show only a label flash.
5. **The simulation now has a quantity worth showing.** In `game_v19` (`rules.rs` `V19_HAZARD` / `V19_PRESSURE`) a village's fission pressure P (`Settlement::fission_pressure`, thousandths) grows at 06:00 on days every fission gate is open (spring to autumn, never in winter; members ≥ 28 to be eligible) by h = 300 × g × s / 10⁶, where g rises from 0 at 24 members to 1 at 44 and s is the food shortfall below a line of members × adult need × 4 days (about 300 food for 32 members) — crowding has weight 0 — and decays by 10 a day when h = 0; it splits when P reaches θ (`Settlement::fission_threshold`, exponential, mean 1,200). The god's food acts move P only while the store is below that line; Pass 13 measured spoiling 60 food moving the split earlier in about half the cases and gifting 60 moving it later in about half, by a few days. A god who can see a village getting restless can choose to feed it or let it split — the first time the god's hand and the shape of the world meet on screen.

---

# 4. PART 0 — Merge, gate, measure
1. Backup branch; merge (2.2); record conflicts and resolutions.
2. Record that the live game builds `game_v19` and the `get_tallies` keys (new ones from Passes 12–13 included).
3. Full gate (`GATE_OUTPUT_DIR=docs/art/production8/gates/p0 bash tools/art_pipeline/gate.sh p0`) on an idle Mac (section 1); fix fallout per 2.2; re-run until green or three strikes.
4. **Re-measure with the merged build** (`rust/target/release/worldsim_probe --scene game_v19 --seeds a..b --days 120 --sample-every 60 --workers 4 --out <dir>`; a probe checkpoint at day N is display day N + 1, 08:00): for the live seed (`DEFAULT_GAME_SEED` 20,261,010) the 02:00 cold nights of the first three winters, the autumn wood days at 15:00, every exposure death, and every fission (departure, founding, ticks, sizes; v19 may now split this seed). For the society scenario, scan seeds 20,261,011–20,261,042 (`--seeds 20261011..20261043`, half-open) and pick the first that has a departure between display days 20 and 60, a second fission later, **3 settlements alive at the same time** on some day (A30 needs it) and ≥ 2 alive at day 120; record the next two as fallbacks. Write `docs/art/production8/MEASURED_DAYS.md`. Every capture plan uses these values.
5. Baseline captures (before art changes) of the live seed d2 12:00 F2 and the society seed one hour after its first founding (FV). Commit, push.

---

# 5. PART 1 — News worth reading
(Every new localization key is added **in the Part that first uses it**, as Art Pass 7 did: fluent `messages.ftl` en and ko, then the compiled JSON, then `key_registry.json`; a missing key makes `_loc` return the key and `key % args` fail at runtime, and `harness_stockpile_feed_2_5c.rs` L151–200 requires the registry and both compiled locales to match. Section 11 is the complete list.)
1. **Classes.** Write the full key → class table in DESIGN.md and code. *Protected events*: the five deaths, `HUD_FEED_FISSION_DEPARTED`, `HUD_FEED_SETTLEMENT_FOUNDED`, `HUD_FEED_FISSION_CANCELLED`, `HUD_FEED_SETTLEMENT_ABANDONED`, `HUD_FEED_SETTLEMENT_FORMED`, and new god's-hand cast lines (`HUD_FEED_GOD_CAST_FMT`, section 11 — there is no cast feed line today). *Ordinary events*: births, construction sites started/completed. *Routine*: stockpile movements.
2. **Routine never reaches the visible feed.** The existing ≥ 3 aggregation (L107–112) stays; each routine movement is still written to `_feed_log` under the old keys (`HUD_FEED_STOCKPILE_DEPOSIT` / `_CONSUME`) with `class: "routine"` and `visible: false` (so `test_founding_group.gd` L66–68 keeps its meaning; update it only if its assertion is about visibility). Each village gets **one daily summary line** at 20:00 (`HUD_FEED_DAY_SUMMARY_FMT`): stored = sum of positive deltas, eaten = sum of negative deltas between 20:00 and 20:00 (day boundaries from `get_time_snapshot`, never a `TICKS_PER_DAY` constant — forbidden by `harness_revitalize_screen_contract.rs` L45–47); god's-hand food casts and fission share transfers are logged separately and excluded from the sums; shown when |net| ≥ 10 or eaten ≥ 10.
3. **Protected events stay readable:** retention is counted in **game ticks**, frozen while paused: a protected line stays visible for at least 240 ticks (8 s at 1×) before a newer line can push it out; when more than 4 protected lines compete, the oldest goes first and the panel shows a "+N" counter (`HUD_FEED_MORE_FMT`). Re-pin `NOTIF_FADE_FRAMES` / `MAX_VISIBLE_NOTIFICATIONS` pins (`harness_p14_delta_status_panel.rs` L316–330) with the same meaning if you change them (ledger rows). Tests inject the clock.
4. **Wording:** keys with a settlement argument use the village numbers of Art Pass 7 (never raw ids); no line says "famine" / "기근" for ordinary eating.
5. Tests `test_feed_priority.gd` (a scripted day with 50 stockpile changes, one departure and one cast: the departure and the cast stay visible ≥ 240 ticks; no routine line is visible; one summary line at 20:00 with the right sums; the log keeps every record with its class), `test_feed_wording.gd` (every key with a settlement argument uses village numbers; no famine wording is pushed). Commit, push.

---

# 6. PART 2 — A destination you can find
1. **Off-screen marker:** when a founding group's site is outside the viewport, draw at the screen edge, on the line from the screen centre to the site, a small arrow (procedural, 9 × 9 px, the parent's colour of Art Pass 7, 1-px dark outline) with the distance in tiles beside it (`FOUNDING_SITE_DISTANCE_FMT` "%d tiles" / "%d칸"); clicking it pans the camera to the site. When the site is on screen, the Art Pass 7 stake shows instead; never both.
2. The same arrow (village colour, no distance) for a **selected** village whose hearth is off screen.
3. Tests `test_offscreen_marker.gd` (edge placement for 8 directions, hidden when on screen, click pans, follows the parent colour). Commit, push.

---

# 7. PART 3 — A restless village (v19)
1. **Getter extension (2.1 a):** `get_society_snapshot` gains, per settlement row, `fission_pressures` and `fission_thresholds` (PackedInt64Array, thousandths, from `Settlement::fission_pressure` / `fission_threshold` that Pass 13 adds; 0 under older rules). The dictionary is built in loops (`world_node.rs` ≈ L2464–2494) and no sim-test pins its key list; extend the unit test (≈ L3360–3439) field by field, and update its `FissionRecord` literal (≈ L3394–3403) if Pass 13 added fields. Read the Pass 13 report for the exact names; if they differ, use theirs and record it.
2. **The meter (drop third):** hidden when θ = 0; otherwise beside each village label a small 3-segment "restlessness" bar showing P/θ (thirds: calm / uneasy / ready to split), in the village's colour, visible at zoom ≥ 2 and in the inspector of a selected village with the words `VILLAGE_RESTLESS_CALM` "Calm" / "평온", `VILLAGE_RESTLESS_UNEASY` "Uneasy" / "술렁임", `VILLAGE_RESTLESS_READY` "Ready to split" / "갈라질 기세". The god may know what the villagers do not (as the winter's severity line).
3. Tests `test_restless_meter.gd` (thirds, hidden without the field, colour, inspector words). Commit, push.

---

# 8. PART 4 — The four that keep coming back
1. **A18 crowds.** Diagnose the 23 failing natural frames of Art Pass 7 (time of day, activity, walking or not, where), reading the strip and evening entity files from Art Pass 7's raw evidence (`/Users/rexxa/work/art-production7-evidence`, read-only): record a table. Then fix the dominant case **from the table**. Expected from the planner's sample: walkers streaming out of the shelters in the morning. Allowed fixes: (a) a render-only lateral offset for walkers that overlap another villager — up to 3 screen px perpendicular to the walking direction, deterministic by AgentId, never onto water or a building, the simulation position untouched; (b) extend `CampSeats._rebuild` (`camp_seats.gd` L52–97: base radius 2 tiles, + 1 per 12 members) for seated and idle crowds. Do not add a parallel seating system. Thresholds unchanged. Add **A18b** (reported beside A18, never replacing it): two blind observers judge each of 12 crowded frames "can you count the people here?" (yes / roughly / no).
2. **A22 morning chill (drop first).** The chill cue was too small to see. Replace it, for villagers within 90 game minutes after their cold flag cleared and awake: a 1-px pale outline `#CFE3F5` around the sprite at alpha 0.8 fading to 0 over the last 30 minutes, plus the existing breath puff at 1.5× size. Pick the warm-control morning from Part 0's measurement (a morning after a lit night with **no** cold flag in the preceding 12 hours); if none exists in the first 60 days, report the control as unavailable rather than using a cold one.
3. **A24 world effects (drop second).** (When villages share a food pile, the effect plays on the pile nearest the target village's formation tile. "Held 1.0 s" means the ring stays at full alpha for 1.0 s, then fades over 0.3 s. Pixel sizes are native texels.) `spoil_food`: a 1.2 s brown puff (6 procedural particles `#6B4E2A`) rising from the village's food pile and the pile sprite tinted toward brown for 1.2 s; `rot_wood`: the wood pile darkens (multiply `#5A4A3A`, 1.2 s) with 4 grey dust particles; `gift_food` / `gift_wood`: 6 light particles falling onto the pile; windfall: Art Pass 7's dust ring held 1.0 s. Effects play on the target village only.
4. **A30, measured properly.** Rewrite the measurement: vsync disabled for the measurement window (`DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)`, restored after), simulation frozen, the same tick and camera. Per frame: the CPU process time = `Time.get_ticks_usec()` taken in a node with `process_priority` −1000 and in one with +1000; plus `RenderingServer.viewport_get_measured_render_time_cpu(get_viewport().get_viewport_rid())` (after `viewport_set_measure_render_time(rid, true)`) and `RenderingServer.get_frame_setup_time_cpu()`; the GPU time (`viewport_get_measured_render_time_gpu`) recorded separately, or "unavailable" if it reads 0. **Do not use `Performance.TIME_PROCESS`** (it publishes the maximum over a ~1 s window, not a per-frame value). A30 becomes: at the society seed's first frame with ≥ 3 settlements alive, at `FV`, 300 frames with the Art Pass 7–8 village layers **on** and 300 with them **off** (labels, meters, trails, markers, outlines hidden through one debug flag), back to back, alternating in blocks of 50; the median of (process + render CPU) with layers on ≤ 1.15 × off, or ≤ off + 1.0 ms. Report the old wall-clock numbers beside it, labelled as pacing. Keep the old A30 row from Art Pass 7 as it was.
5. Tests for each changed behaviour (`test_meal_ring.gd`, `test_morning_chill_v2.gd`, `test_intervention_world_effects.gd`, `test_frame_measure.gd` on a stub). Commit, push.

---

# 9. PART 5 — Captures
- `ART` unchanged. `ART_LONG` re-mapped to Part 0's `game_v19` days for the live seed (cold nights, the morning after and the warm control, autumn carriers), plus the live seed's own fission frames if it splits.
- `ART_SOCIETY` on the new society seed (RESET row; feed frame = departure + 3 game hours): the crowded village the evening before departure with the meter visible (zoom 2); departure + 30 min `FD` (the off-screen arrow visible); the walk at +2 h `FD`; founding + 10 min `FS`; founding day 12:00 `FV` with and without a selection; the daughter's first winter night `F2N`; the second departure; and a **feed frame** at departure + 3 game hours showing the departure line still visible.
- `ART_GOD`: Art Pass 7's casts, with its society rows (`capture_plans.gd` L94–98, hard-coded to seed 20,261,013 and a daughter at d50–51) re-mapped to the new seed and Part 0's days; plus the **paired worlds** of A33: on the society seed, on three days before the first departure, outside winter, where Part 0 shows the original village eligible and its food below the scarcity line (food < members × adult need × 4 days; the meter moves only there; record the food and the line on each cast day), `spoil_food` 60 in world P (`p_spoil_*` file stems) and, after a RESET, `gift_food` 60 on the same days in world G (`p_gift_*`); frames of the meter the morning after each cast; both worlds run to a final day-120 frame from which the departure days are read (`fission_depart_ticks` of the getter), since they are not known in advance. A missing target is recorded, not a plan abort.
- Metadata: feed classes and visible lines, marker state (on/off screen, distance), meter values, per-frame P and θ, A30 measurement blocks. Commit, push.

---

# 10. PART 6 — Measure (thresholds unchanged unless stated)
- A1–A29 as Art Pass 7 on the new frames (A1–A25 live-seed natural frames only; `soc_` frames feed A26–A31 and a separate A18 line). A30 as redefined in 8.4 (the Art Pass 7 row stays recorded as it was).
- **A31 news worth reading (new):** in every frame no routine line is visible; every event in the capture windows appears exactly once in the log; every protected event since the previous frame is visible or counted in "+N"; the departure line is visible in the feed frame (departure + 3 game hours); daily summaries match the logged movements of the day.
- **A32 the way is shown (new):** in every frame with a walking group, either the stake is on screen or exactly one edge arrow points to it (angle within 10°), with the right distance ± 1 tile; blind: two observers shown the `FD` frames answer "where is the group going?" by pointing a direction among 8 — correct by both.
- **A33 the god moves the day (new):** in `ART_GOD`'s paired worlds, the meter of world P reads higher than world G's the morning after the last cast, and P's departure is strictly earlier than G's (a tie, or no departure in either, fails; a departure in only one world counts as a difference in the right direction if it is P's); report both days.
- Boards `news.png` (feed before/after), `way.png` (arrow and stake), `restless.png` (meter through a build-up and split), `crowds.png` (A18 before/after), `effects.png` (A24), plus `before-after-live.png` and `before-after-society.png`. Commit, push.

---

# 11. PART 7 — Localization (en / ko, every key in both; check for duplicates)
| key | en | ko |
|---|---|---|
| HUD_FEED_DAY_SUMMARY_FMT | Village %d today: food %+d (stored %d, eaten %d) | %d번 마을 오늘: 식량 %+d (저장 %d, 소비 %d) |
| HUD_FEED_MORE_FMT | +%d more | +%d건 더 |
| FOUNDING_SITE_DISTANCE_FMT | %d tiles | %d칸 |
| VILLAGE_RESTLESS_CALM | Calm | 평온 |
| VILLAGE_RESTLESS_UNEASY | Uneasy | 술렁임 |
| VILLAGE_RESTLESS_READY | Ready to split | 갈라질 기세 |
| VILLAGE_RESTLESS_TITLE | Restlessness | 술렁임 정도 |
| HUD_FEED_GOD_CAST_FMT | God's hand on Village %d: %s | %d번 마을에 신의 손: %s |

`%+d` works in GDScript formatting; format through the panel's `_loc` helper as the other feed lines do. The keys are added in the Parts that use them (Part 1 note); this Part checks completeness, duplicates and both compiled locales. Commit, push.

---

# 12. PART 8 — Final review
An independent review of the final screens (news feed during a departure, the off-screen arrow, the meter before a split, a morning crowd, the god's paired worlds) with `ACCEPT` / `NEEDS WORK` and reasons, recorded as given.

# 13. PART 9 — Final gate, report, PR
1. Full gate on an idle Mac: `GATE_OUTPUT_DIR=docs/art/production8/gates/final bash tools/art_pipeline/gate.sh final`. Fast-forward `codex/art-production-2026-10` if green; otherwise say exactly what is red. **The Rust part of the gate must actually run and pass** (Art Pass 7 published a report while the Rust tests had never compiled).
2. Budgets: `assets/art/**` ≤ 25 MB, `docs/art/production8/**` ≤ 30 MB.
3. `docs/art/production8/REPORT.md` in Korean in the section 15 format.
4. Update PR #511's description with an Art Pass 8 section; keep it a draft.

---

# 14. 인게임 확인사항 (게임을 직접 켰을 때 확인할 것)
1. 게임이 game_v19로 시작하고 콘솔에 SCRIPT ERROR가 없다.
2. 소식창에 "정착지 1 식량 4 적재" 같은 줄이 더 이상 쏟아지지 않고, 마을마다 저녁 8시에 하루 요약 한 줄만 나온다. "13명이 1번 마을을 떠났다" 같은 사건 줄은 몇 초 이상 남아 있다.
3. 무리가 떠나면 화면 가장자리에 목적지 방향 화살표와 거리("24칸")가 보이고, 누르면 그쪽으로 화면이 간다. 화면 밖에 있는 마을을 선택해도 그 방향 화살표가 나온다. 사건이 많을 때는 소식창에 "+N건 더"가 붙는다.
4. 마을 이름표 옆 술렁임 막대가 마을이 크고(24명 넘게) 저장 식량이 4일치 아래로 내려간 날마다 차오르고(겨울에는 멈춤), 다 차면 무리가 떠난다. 그런 때 식량을 썩히면 빨리 차고, 식량을 내려 주면 늦게 찬다. 식량이 넉넉할 때는 거의 변하지 않는다.
5. 아침 화덕 주변 주민들이 덜 겹쳐 보인다.
6. 추운 밤 다음 아침, 일어난 주민들 둘레에 옅은 하늘색 테두리와 큰 입김이 잠시 보인다.
7. 식량 썩히기·장작 썩히기·선물을 쓰면 저장고 위에 갈색 연기나 떨어지는 알갱이가 보인다.
8. 마을이 셋일 때 이름표·막대·점선을 켜고 꺼도 프레임 시간이 거의 같다.

---

# 15. 구현 완료 보고 (REPORT.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
병합과 병합 직후 게이트, v19 날짜·사회 시나리오 시드 측정, 소식 분류·하루 요약·사건 유지, 화면 밖 목적지 화살표, 술렁임 막대와 조회 함수 확장, 군중 식사 고리, 아침 한기 v2, 개입 세계 효과, 프레임 측정 재설계(vsync 끔, CPU 시간, 레이어 켬/끔)

### 실제 게임 화면 검증
A1~A33 표(수치), 블라인드 결과, 자연·사회·개입 장면 구분, 보드 경로

### 시뮬레이션 작업과의 호환
병합한 시뮬레이션 커밋과 결과, 조회 함수 확장 내용(다음 시뮬레이션 작업이 이어받을 것), v19에서 잰 날짜와 사회 시나리오 시드

### 변경된 파일 목록
(커밋별)

### 자율 결정 목록
DECISIONS.md 요약

### 확인된 제한사항

### Harness 결과
(cargo test 수치와 소요 시간, clippy, strict, bridge, smoke, Python, Godot 아트 장면 테스트, 게이트 실행 당시 맥 부하)

### 정직한 판단
처음 보는 사람이 소식창만 보고 마을에 무슨 일이 일어났는지 알 수 있는가? 떠나는 무리가 어디로 가는지, 어느 마을이 곧 갈라질지 화면에서 보이는가? 신의 손이 세계의 모양을 바꾸는 것이 보이는가? 아직 아닌 것은?
```

Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the simulation commit merged, whether PR #511's description was updated, image calls used, and the paths of `MEASURED_DAYS.md` and every board.
