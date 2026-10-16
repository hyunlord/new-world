# GOAL: WorldSim — Art Pass 7: many villages on one map — who belongs where, a group walking out to found a new home, and deaths told truthfully

Repository `hyunlord/new-world`. Continue the art line: `codex/art-production-2026-10-wip` and `codex/art-production-2026-10`, both at `cd998d3807358521ead77b0a7314e93fd8283878` (Art Pass 6; draft PR #511, stacked on the simulation PR #510).
The simulation line is sealed at Pass 11: `codex/revitalize-2026-10` and `-wip` are both at `31559232e0a24af12d29f490d440d1c5ee5f3661` (gate green). Since the art line's last merged simulation head (`4c35e295`, Pass 8) it adds: `game_v15` (Pass 9: keyed draws, noise forks), `game_v16` (Pass 10: hidden world parameters per seed, **death by exposure**), `game_v17` (Pass 11: hardiness, adults can freeze, rain soaks the woodpile, wild regions, and **village fission**: a crowded village sends a group of 6–14 to a wild region, where it builds a hearth and two shelters and becomes a second or third village; small remnants merge away; abandoned villages leave their buildings). The live game builds `game_v17` after the merge.
At the same time, the simulation's Pass 12 (Claude Code) works in `/Users/rexxa/work/new-world-pass4` on the same Mac and measures timings there, and a learned-model run works in `/Users/rexxa/work/new-world-wm1` (computing on the DGX). Neither touches your clone.
This run is executed by **Codex**, unattended, on the Mac.

---

# ⚠️ 0. AUTONOMY — READ FIRST

1. **Never ask the owner anything.** Decide, implement, record the decision and its reason in `docs/art/production7/DECISIONS.md`.
2. **First action:** save this document unchanged as `docs/art/production7/GOAL.md` (the owner may already have placed it there; then verify it is this document), commit it in Part 0, and re-read the section of the current Part at every Part start and after any context compaction.
3. **Do not stop between Parts.** One task per Part; push to the WIP branch after every Part.
4. **Review loops: two rounds maximum.** Fix once, re-review once; then accept, record, move on.
5. **On failure: work around, note it, continue.** Partial delivery with an honest report beats a stalled run.
6. **Time rule:** record start time and each Part's start and end in `docs/art/production7/TASKS.md`. Past **10 hours**, finish the current step, push, go to Part 9.
7. **Three strikes** per failing target, then keep the best state and record evidence and hypothesis.
8. **Drop order when time is short:** Part 6 (Art Pass 6 leftovers) → A27's blind test with a single observer instead of two → the abandoned-village look of 8.4 → the founders' bundle (keep the trail and the site marker). Never drop Parts 0, 1, 2, 3, 7, 8, 9, the settlement labels, the founding feed lines or the exposure death fixes.
9. **Image budget: at most 4 new generations**, failures included. Prefer procedural drawing and existing art. On a quota or rate-limit message, stop generating and continue.
10. **Honesty rules stay:** never mark a failed check as passed, never lower a threshold of section 12, keep rejected evidence, never edit or re-interpret an earlier verdict. Captures that contain a god intervention carry `INTERVENTION SCENARIO` in their metadata and the `god_` prefix and never count as natural evidence. Captures of the society scenario (11.1) carry `SOCIETY SCENARIO seed <n>` and the `soc_` prefix: they are natural (no intervention) but from another seed than the live game, and are reported separately.
11. **This brief supersedes the repository's session-start instructions** (harness hook installation, the `lead/main` workflow, `harness_pipeline.sh`): do not install hooks, do not run the harness pipeline, never push to `lead/main` or `main`, never force-push.

---

# ⚠️ 1. WHERE AND HOW

- Mac only, the existing clone `/Users/rexxa/work/new-world-art-prod`. Run every pipeline script from the repository root. New raw generations go to `/Users/rexxa/work/art-production7-raw/<sheet_id>/attempt_<n>.png`; capture evidence to `CAPTURE_EVIDENCE=/Users/rexxa/work/art-production7-evidence`. Earlier raw and evidence folders are read-only. Raws never enter git.
- Never touch `/Users/rexxa/work/new-world-pass4` (simulation), `/Users/rexxa/work/new-world-wm1` (learned models), `~/github/new-world-wt/lead`, `/Users/rexxa/work/new-world-capture` or any other checkout; never kill a process you did not start; no ComfyUI, no dev servers, no DGX.
- `export CARGO_BUILD_JOBS=4` (the simulation run builds at the same time); cargo tests `-- --test-threads=4`. `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`. Pipeline Python uses Pillow and numpy only.
- Windowed Godot only through `tools/art_pipeline/with_window_lock.sh <seconds> <command...>` in one command (from a background session inside `launchctl asuser $(id -u) …`, still inside the lock; each locked command under 25 minutes). Headless Godot needs no lock; other limits use `tools/art_pipeline/run_bounded.sh`.
- **Performance measurements (A10, A30)**: before each, record `uptime` and `pgrep -fl "cargo|worldsim_|rustc"` (the simulation run may be busy). Compare frame times only between runs made back to back under the same load; say so beside the numbers.
- Image generation: the built-in `$imagegen` skill, magenta background, prompts never name existing games, studios, artists or characters; prop prompts use the Art Pass 4 form (`docs/art/production4/wood/prompts/V_a4.txt`).
- Scripts of earlier passes that write under `docs/art/production6/**` are copied to `_p7` versions writing under `docs/art/production7/**`, never called as they are.
- **Independent review** means a fresh reviewer session or subagent that has not seen the pipeline code or sections 3–10 of this brief. Blind tests keep Art Pass 3's procedure (answer file frozen first; observers get only the images and the allowed answers).

---

# 2. GIT

- `git fetch origin && git checkout codex/art-production-2026-10-wip`; it must equal `origin/codex/art-production-2026-10-wip` (`cd998d38`). Local-only commits go to a backup branch `art-p7-local-backup` first.
- **Part 0 merges the simulation green head** `31559232e0a24af12d29f490d440d1c5ee5f3661` exactly: `git merge --no-ff 31559232 -m "art-p7(p0): bring in simulation passes 9 to 11"`. The planner's trial merge (`git merge-tree`) has **no conflicts**; Art Pass 6's `get_intervention_state` in `world_node.rs` and the simulation's `game_v17` constructors merge side by side. If your merge shows a conflict anyway: under `rust/crates/sim-*/src/**`, `docs/revitalize/**`, `docs/worldmodel/**`, `tools/revitalize/**` and `tools/worldmodel/**` take the simulation's side, keep the art side's `get_intervention_state`, record every resolved file. Merge no later simulation head in this pass.
- WIP gets every commit; `codex/art-production-2026-10` moves only by fast-forward to a WIP commit whose full gate (Part 9) is green. Never force-push. After every push `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`.
- Commit messages `art-p7(pN): <what changed>`; stage explicit paths only; never stage capture evidence of earlier passes.
- **May change:** `scripts/**`, `scenes/**`, `shaders/**`, `assets/**`, `localization/**`, `project.godot`, `docs/art/**` (new files only under `docs/art/production7/**`, plus `docs/art/TEST_LEDGER.md`), `tools/art_pipeline/**`, Rust static-source tests under `rust/crates/sim-test/tests/` that read `.gd`/`.tscn`/`.gdshader` files (ledger row each under `## Art Pass 7`).
- **One narrow exception in simulation-owned code (Part 3):** one additive, read-only FFI getter `get_society_snapshot` in `rust/crates/sim-bridge/src/ffi/world_node.rs` with its pure collector and unit test. Nothing else under `rust/` outside the tests above. (The simulation's Pass 12 is told not to add a function of that name and not to rename the fields it reads.)
- **Must not change:** everything else under `rust/`, `docs/revitalize/**`, `docs/worldmodel/**`, `tools/revitalize/**`, `tools/worldmodel/**`, `tools/harness/**`, `.claude/**`, `.harness/audit|baseline|reviews/**`, and evidence under `docs/art/production{,2,3,4,5,6}/**`.

---

# 3. WHY THIS PASS (planner's review, with evidence)

1. **The map can now hold two or three villages, and the UI assumes one.** Hearth smoke and wood looks are set per hearth sprite (`building_renderer.gd` L314–323, `world_effects.gd` L114) from `wood_visuals.gd` L6–28, which picks the **nearest** formation tile — so an abandoned or retired hearth inherits a neighbouring village's lit fire and smokes (the formation tile is the hearth tile, `settlement_system.rs` ≈ L439, L468). `camera_controller.gd` L184 centres on the largest village; `god_hand_panel.gd` L92 always casts on −1 (the lowest id, the original village); `settlement_overview_renderer.gd` draws identical discs with no names; `hud_status_panel.gd` L106–111 announces any new or vanished settlement by its raw id ("Settlement %d founded"); nothing says which villager belongs where, and a group walking 22–28 tiles out of the village looks like villagers wandering off.
2. **A new village appears without a word.** Departure, founding, a group turning back and an abandoned village have tallies (`fissions`, `fissions_cancelled`, `remnant_merges` in `get_tallies`) but no feed line, no marker, no trail.
3. **Deaths are told wrong since Pass 10.** `hud_topbar.gd` L194 sums starvation, dehydration, combat and old age — not `deaths_exposure`; `hud_status_panel.gd` L40 has four death news keys, so reason 4 (exposure) is dropped (L250); `death_viz_renderer.gd` L101–112 draws any unknown reason, exposure included, in the **combat red**. With Pass 11, exposure is the most common death in a harsh winter (adults 41 %, elders 36 %, children 23 % of exposure deaths on held-out seeds).
4. **Art Pass 6 left four visual failures** (A18 winter crowd, A21 doorway breath, A22 morning chill, A24 windfall). They stay measured; Part 6 tries the two cheapest if time allows.

## 3.1 Facts (planner's probe of `31559232`, `game_v17`)
- **The live seed (`DEFAULT_GAME_SEED` 20,261,010) never splits**: 120 days, 1 settlement, 29 villagers settled, 0 fissions. Its cold nights and autumn carry days differ from Art Pass 6's v14 measurements: re-measure everything (Part 0) before planning captures.
- **A seed that splits (the society scenario):** seed **20,261,013** — 2 settlements alive at the probe's day-29 checkpoint (a probe checkpoint at day N is tick N × 1440 = display day N + 1 at 08:00, so: display day 30, 08:00; the fission check runs at 06:00 of display day 29), a second fission later, 3 settlements alive and 66 villagers at day 120. The probe already writes `fission_log` (depart, founded, cancelled ticks, members) and per-settlement `founded_tick` / `parent` (`sim-test/src/probe/pass11.rs` L31–55): `rust/target/release/worldsim_probe --scene game_v17 --seeds 20261013..20261014 --days 120 --sample-every 60 --workers 1 --out <dir>`. Fallbacks: 20,261,022 (3 settlements, 100 villagers at day 120), 20,261,017 (2 settlements). `reset_world(seed)` builds `game_v17(seed)` (`world_node.rs` L225–229).
- What the simulation exposes today: `get_settlement_snapshot()` → `ids`, `settlement_ids`, `centroid_xs/ys`, `member_counts`, `formation_xs/ys`, `food_stocks`, `wood_stocks`, `fire_lit`, `wood_targets`; `get_agent_snapshot()` → `ids`, `agent_ids`, `xs`, `ys`, `states`, … (no home settlement, no founding flag); `get_tallies()` → the five death counts, `fissions`, `fissions_cancelled`, `remnant_merges`, …; `get_recent_deaths()` → `xs`, `ys`, `reasons` (0 starvation, 1 dehydration, 2 combat, 3 old age, 4 exposure), `ticks`, `current_tick`.

---

# 4. PART 0 — Merge and baseline
1. Backup branch, merge (section 2), record the result.
2. Full gate (`GATE_OUTPUT_DIR=docs/art/production7/gates/p0 bash tools/art_pipeline/gate.sh p0`); fix merge fallout in art-owned paths only (static pins that break because simulation code moved: re-pin with the same meaning, ledger rows, never weaken).
3. Record that the live game builds `game_v17`, and the `get_tallies` keys.
4. **Re-measure with the merged build** (headless `advance_ticks`, or the probe in `rust/target/release/worldsim_probe --scene game_v17`): for the live seed, the 02:00 cold nights of the first three winters, the autumn wood-gathering days at 15:00, and every exposure death (tick, stage); for the society seed (and the fallbacks if needed), the departure tick (06:00 of the day the group gets `Founding`), the founding tick (the daughter appears in `settlement_ids`), the group size, the site tile, every later fission, cancellation or dissolution with ticks, and the first exposure death in a daughter. Write `docs/art/production7/MEASURED_DAYS.md`; every capture plan uses these values. If the society seed does not split in the live build, take the first fallback that does and record why.
5. **Capture tooling for a second seed** (no rendering change; record that it is tooling only): today the capture tool builds one world on the default seed with no reset (`capture_framings.gd` L56–98), plan rows are `[day, clock, framing]` (`ART_GOD` rows have 7 fields, `capture_plans.gd` L61–72), `_village_tile` picks the largest village (L251–261) and casts record `settlement_id: -1` (L161). Add: a row `["RESET", <seed>]` that calls `reset_world(seed)`, re-reads `clock_offset_ticks` and awaits two process frames before advancing (`world_reset` is emitted deferred, `world_node.rs` ≈ L231); a fourth row field `target` ∈ `orig | daughter | leader | all` used by the framings; selection through one named method (e.g. `SettlementSelection.select(id)`); `art_society` in **both** plan whitelists (`capture_framings.gd` L49–50 and `tools/art_pipeline/capture_art.sh` L8), and excluded from rewriting the combined `manifest.json` as `art_god` is (`capture_art.sh` ≈ L95–106); the scenario string `SOCIETY SCENARIO seed <n>` in the three places that write the scenario (`capture_framings.gd` ≈ L209, L243, L369). In `_advance_to` (L313–331, synchronous 8-tick steps), after each `advance_ticks` chunk also call the status panel's `_ingest_snapshots()` (precedent: `ingest_chill_snapshot`, L325), so feed lines and deaths are not lost between shots (deaths older than `DEATH_RETAIN_TICKS` 120 are skipped, `hud_status_panel.gd` L39, L250).
6. Baseline captures (before any art change) of the live seed's d2 12:00 F2 and a frame of the society seed one game hour after founding (wide enough to show both villages). Commit, push.

---

# 5. PART 1 — The UI's one-village assumptions (audit)
`docs/art/production7/MULTI_SETTLEMENT_UI_AUDIT.md`: list with file:line every place in `scripts/**` and `scenes/**` that (a) takes the first, lowest or only settlement, (b) uses one hearth or one formation tile as "the" centre (camera, cold hearth, wood visuals, world effects, nature clearing, overview discs), (c) re-renders a whole layer when a settlement appears (terrain and nature fingerprints include formation tiles: `terrain_renderer.gd` L138, `nature_renderer.gd` L95), (d) counts or labels "the" village (HUD, god's hand, inspector, capture metadata, `ingame_qa*.py`, `measure_p6.py`). For each: what it does with two or three settlements and what this pass does (fix in Parts 2–5, or harmless, with reason). Fix here (never dropped): `wood_visuals.settlement()` matches a hearth to the settlement whose formation tile **equals** the hearth tile; a hearth with no match returns `{}`, gets `fire_lit` false, no smoke and cold ash. Commit, push.

---

# 6. PART 2 — Deaths told truthfully
1. **HUD sum:** `hud_topbar.gd` L194 adds `deaths_exposure` (and any `deaths_*` key the tallies carry, by an explicit list of five, never by a prefix scan). Update `test_hud_old_age.gd`'s expectations; add a five-reason case.
2. **Feed:** the panel shows at most `MAX_VISIBLE_NOTIFICATIONS` = 4 lines (`hud_status_panel.gd` L12, L166; pinned in `harness_p14_delta_status_panel.rs` L311–323) — keep it. Add a `_feed_log` array that records every push (key, arguments, tick) and is never trimmed except on reset, written into capture metadata; A26 and A28 count from it, not from the visible lines. `DEATH_NEWS_KEYS` gains `HUD_FEED_DEATH_EXPOSURE` at index 4, in the form of the other four (fluent `messages.ftl` L5201–5204 en / L5203–5206 ko, compiled and registered the same way): en "A villager froze to death", ko "주민이 얼어 죽었다". The death icon index stays 8.
3. **Colour:** `death_viz_renderer.gd` gains `COLOR_EXPOSURE = Color(0.70, 0.84, 0.97)` (pale ice) for reason 4, and an **unknown** reason (≥ 5 or < 0) is drawn `Color(0.55, 0.55, 0.55)` and logged once, never in the combat red. Update the header comment's legend.
4. Tests `test_death_exposure.gd`: a recent-deaths stub with all five reasons → five `_feed_log` entries with the right keys (four visible), five colours, the HUD sum equals the five tallies. Commit, push.

---

# 7. PART 3 — One read-only getter: who belongs where (the section 2 exception)
`#[func] fn get_society_snapshot(&self) -> VarDictionary`, built on a pure `collect_society_snapshot(world: &hecs::World, resources: &SimResources) -> SocietySnapshot` with a unit test, additive only, reading `&self.engine.world` / `&self.engine.resources` as the other getters do:
- per settlement, in SettlementId order, **with the same filter as `collect_settlement_snapshot`** (it skips settlements without a positioned member): `settlement_ids: PackedInt32Array`, `parent_ids: PackedInt32Array` (`Settlement::parent`, −1 for none), `founded_ticks: PackedInt64Array` (`Settlement::founded_at`), `member_counts: PackedInt32Array`;
- per member, in AgentId order: `home_agent_ids: PackedInt64Array` (AgentId is u64; `get_agent_snapshot`'s `agent_ids` is a PackedInt64Array of the same values), `home_settlement_ids: PackedInt32Array` (from each settlement's `member_agents`; an agent in no roster is absent). Walking founders stay in the parent's roster until founding, so a departure's parent = the founders' home settlement, and the parent's roster count includes the group while it walks;
- per agent carrying `Founding` (`founding.rs`: `site: (u32, u32)`, `fission_id: u32`, `leader: AgentId`, `since_tick: u64`), in AgentId order: `founder_agent_ids: PackedInt64Array`, `founder_fission_ids`, `founder_site_xs`, `founder_site_ys` (PackedInt32Array), `founder_leader: PackedByteArray` (1 if the agent is its group's leader), `founder_since_ticks: PackedInt64Array`;
- the fission log (`SimResources::fission_log`, `FissionRecord`, `sim-engine/src/lib.rs` ≈ L467–492), in fission-id order: `fission_ids`, `fission_parents`, `fission_site_xs`, `fission_site_ys`, `fission_sizes` (PackedInt32Array), `fission_depart_ticks`, `fission_founded_ticks`, `fission_cancelled_ticks` (PackedInt64Array, −1 when not yet), `fission_daughters` (PackedInt32Array, −1 when none) — feed lines then come from log ticks and cannot be lost between refreshes;
- `retired_hearth_xs`, `retired_hearth_ys` (PackedInt32Array, from `SimResources::retired_hearths` ≈ L454), for 8.4;
- `current_tick: int`.
Test the agent join on one built world. Record the getter in DECISIONS.md and in the report's compatibility section (the simulation's Pass 12 is told to keep every field it reads). Run `cargo test -p sim-bridge`, `cargo clippy --workspace --all-targets -- -D warnings` (the gate's) and `cargo clippy --workspace -- -D warnings`, and the bridge build after adding it. Commit, push.

---

# 8. PART 4 — Many villages on one map

## 8.1 Names and labels
- Every alive settlement has a name: its rank in SettlementId order among all settlements ever seen in this world (1 = the original), kept for its lifetime (a UI-side map from id to number, cleared on `reset_world`). Label at the formation tile, above the hearth: `SETTLEMENT_LABEL_FMT` ("Village %d" / "%d번 마을") with the member count beside it (roster members minus that settlement's walking founders) in the existing small HUD font; a daughter's label adds its parent: `SETTLEMENT_LABEL_DAUGHTER_FMT` ("Village %d · from %d" / "%d번 마을 · %d번에서"). Visible at every zoom step, scaled so it stays 9–12 screen px tall; never covers the hearth sprite.
- Each settlement gets a colour from a fixed four-colour palette by its number (`#E0B060`, `#7FB0D8`, `#B88AD0`, `#8CC07A`): the label's underline, its overview disc (`settlement_overview_renderer.gd`, no longer identical discs) and the selection highlight.
- Top bar: `HUD_SETTLEMENTS_FMT` ("Villages %d" / "마을 %d") when more than one settlement is alive; hidden with one.
- Create the label, trail and marker layers **in code** under an existing node: `main.tscn`'s `load_steps=21` is pinned in seven static tests (e.g. `harness_p14_zeta_zoom_adaptive.rs` L597, `harness_p14_epsilon_activity_trails.rs` L495); if you must add scene nodes, re-pin every one with ledger rows. The overview renderer keeps `OVERVIEW_ALPHA` 0.28 and reads only `get_settlement_snapshot` (pinned a12/a13); pass colours and names to it from the label layer.

## 8.2 Belonging
Clicking a label (or a disc at overview zoom) selects that settlement: its members get a 1-px outline in its colour (from `home_settlement_ids`), the others none; Escape or clicking empty ground clears it. No outline without a selection (the default view stays clean). Inspector: the selected villager's panel shows its village name.

## 8.3 The god's hand chooses a village
`god_hand_panel.gd` casts on the selected settlement's id (else −1 = home, as today) and shows the target's name in the panel title (`GOD_HAND_TARGET_FMT` "God's hand — %s" / "신의 손 — %s"). The toast names the village with `GOD_HAND_APPLIED_TARGET_FMT` ("%s: %s %s — applied %d" / "%s: %s %s — 적용 %d", village name first, same argument order in both languages). `get_intervention_state`'s rain ids must then contain the selected id (the gate test uses a stub; the real node is checked in `ART_GOD`'s society world, A29).

## 8.4 Abandoned villages (drop third)
A Hearth in the building snapshot whose tile is in `retired_hearth_xs/ys` (Part 3) is drawn abandoned: no smoke, cold ash, buildings desaturated 60 % and at alpha 0.85, and the label `SETTLEMENT_ABANDONED_FMT` ("Abandoned village %d" / "버려진 %d번 마을") in grey.

## 8.5 Tests
`test_settlement_labels.gd` (numbering stable across a dissolution and a reset, daughter label names its parent, colours by number, top-bar count shown only with ≥ 2), `test_settlement_select.gd` (outline only on the selected village's members; the god's hand target follows the selection), `test_every_hearth.gd` (two hearths with different fire and wood states each drawn by their own settlement), `test_abandoned_village.gd`. Commit, push.

---

# 9. PART 5 — A group walking out to found a home
1. **Feed lines** (from `get_society_snapshot`'s fission log and settlement rows each refresh; one line per event, never repeated; cleared on reset; every `%d` is a UI village number of 8.1, never a SettlementId). They **replace** the existing `HUD_FEED_SETTLEMENT_FORMED` / `HUD_FEED_SETTLEMENT_DISSOLVED` pushes (`hud_status_panel.gd` L106–111): the first village keeps `FORMED` (with its number), a settlement with a parent pushes only `HUD_FEED_SETTLEMENT_FOUNDED`, a disappearance pushes only `HUD_FEED_SETTLEMENT_ABANDONED`.
   - departure — a new fission log entry: `HUD_FEED_FISSION_DEPARTED` ("%d people left Village %d" / "%d명이 %d번 마을을 떠났다");
   - founding — a new settlement id with a parent: `HUD_FEED_SETTLEMENT_FOUNDED` ("Village %d was founded" / "%d번 마을이 세워졌다");
   - cancellation — a log entry got a cancelled tick: `HUD_FEED_FISSION_CANCELLED` ("A group turned back" / "무리가 돌아왔다");
   - abandonment — an alive settlement id disappeared: `HUD_FEED_SETTLEMENT_ABANDONED` ("Village %d was abandoned" / "%d번 마을이 버려졌다").
2. **The trail:** while a group walks, a dashed line (2 px dashes, 3 px gaps, `#E8D8A8`, alpha 0.6) from the leader's tile to the site, drawn under the agents; gone when the group's `Founding` markers are gone.
3. **The site marker:** a small stake with a pennant in the parent's colour on the site tile (procedural: a 1 × 6 px dark pole, a 4 × 3 px pennant, a 1-px dark outline; or one generated prop if the review rejects it), from departure until founding or cancellation.
4. **The bundle (drop fourth):** every founder walks with a bundle on its back — Art Pass 6's composited load layer (`carry_load_layer.gd`) anchored at the upper back instead of the hip, using the wood-bundle look, hidden when the founder eats, sleeps or carries a real load.
5. Tests `test_founding_group.gd` (feed lines once per event with the right numbers on a scripted sequence of snapshots: depart → walk → found; depart → cancel; a remnant abandonment; the trail and marker appear and go; bundle anchor and flip). Commit, push.

---

# 10. PART 6 — Art Pass 6 leftovers (drop first)
At most one attempt each, thresholds unchanged: **A21** — the doorway breath was not identified: make the puff 5 × 3 px, alpha 0.85 at birth, rising 6 px over 1.2 s, every 1.6 s (still offset per shelter id), and re-measure; **A24** — the windfall was not identified: add a 2-px pale dust ring at each spot that rose, 0.6 s, and hold the pile drop for 0.8 s. A18 and A22 are re-measured as they are (no change). Commit, push.

---

# 11. PART 7 — Captures

## 11.1 Plans (`scripts/test/art/capture_plans.gd`)
- `ART` unchanged. `ART_LONG` = Art Pass 6's list with every day re-mapped to Part 0's `game_v17` measurements of the live seed (cold nights, the morning after, the warm-control morning, autumn carriers); record the mapping in `MEASURED_DAYS.md` and in each frame's metadata.
- New `ART_SOCIETY` (natural, society seed via the `RESET` row of 4.5, `soc_` prefix, metadata `SOCIETY SCENARIO seed <n>`, rows `[day, clock, framing, target]`), from Part 0's ticks: the crowded village the evening before departure (F2); departure + 30 min with a new framing `FD` (zoom 4 on the group leader; fails with a clear message when no founder exists); the walk at +2 h and +4 h (`FD`); the founding tick + 10 min at the site (`FS`, zoom 4 on the daughter's formation tile); founding day 12:00 with a new framing `FV` (the smallest zoom step that contains every alive formation tile with a 6-tile margin); the same `FV` with the original village selected (outline) and with the daughter selected; the daughter's first winter night at 02:00 (`F2N` on the daughter); the frame after the first exposure death in a daughter (zoom 4, death mark visible); the second fission's departure, if any; an abandonment, if any, at +1 h. Chronological, one forward-only world; the run may be split into several bounded invocations (each under `run_bounded` and the 25-minute lock), each replaying from `RESET`.
- `ART_GOD`: Art Pass 6's casts on the live seed, plus, after a `RESET` row, a second world on the society seed after the founding: select the daughter, cast rain 13 h → frames +0.5 s and at the next 02:00 (`F2N` on the daughter), and one frame of the original village at the same time (its fire unaffected).
- Metadata gains: society snapshot per frame (settlement ids, names, parents, member counts, founders), labels drawn (text, tile, colour), selection, trail and marker state, feed lines shown, the five death tallies and the HUD death number.

## 11.2 Run
`art`, `art_long`, `art_society`, `art_god` into `docs/art/production7/shots`; `calibration` into `docs/art/production7/calibration`.

---

# 12. PART 8 — Measure (thresholds unchanged unless stated)
- A1–A25 as Art Pass 6 on the re-mapped `ART_LONG` — scanning **only the live-seed natural frames**; `soc_` frames feed A26–A30 and one separate A18 line (`measure_p7.py` and `ingame_qa*_p7.py` copied from the `_p6` versions with the new plans; A18, A21, A22, A24 re-measured honestly whatever Part 6 did).
- **A26 deaths told truthfully (new):** in every frame the HUD death number equals the sum of the five death tallies; every death in the capture windows yields exactly one feed line with its reason's key and its reason's colour; no non-combat reason is ever drawn in the combat colour (scan of every frame's death metadata).
- **A27 villages legible (new):** in every frame with ≥ 2 alive settlements, each alive settlement whose formation tile is in view has exactly one label at it, numbers unique and stable across the run, a daughter's label names its parent, the top-bar count equals the alive count, discs differ in colour. **Blind:** two observers each get 4 `FV`-type frames (no selection) and answer "how many villages?" and "which one was founded from which?" — ≥ 3 of 4 correct each.
- **A28 the walk is visible (new):** in the departure and walk frames every visible founder has the bundle (or, if dropped, say so), the trail runs from the leader to the site, the site marker is present; after founding both are gone; feed lines appear once per event and match the tallies and the snapshot (departure count = group size). **Blind:** two observers see the departure and walk frames and answer "what are these people doing?" from {working, wandering, moving away to settle, fleeing, unsure} — "moving away to settle" by both.
- **A29 the hand on one village (new):** the society-scenario rain lands on the selected daughter only (`get_intervention_state` rain ids, the daughter's cold ash and dark night, the original village's fire unaffected in the same frame), and the toast names it.
- **A30 no one-village leftovers (new):** every audit item fixed or justified; frame time: at the end of `art_society`, (a) 300 frames at day 28 22:00 (1 settlement) at the zoom step `FV` will use, then (b) 300 frames at the first frame with 3 settlements alive, at `FV`, back to back in the same run (`frame_times_soc_1.json`, `frame_times_soc_3.json`; `_measure_frame_times` today runs only in the `art` plan at zoom 1, `capture_framings.gd` L447–472 — extend it); (b)'s median within +15 % of (a)'s. If 3 settlements are not reached in a bounded run, use fallback seed 20,261,022 for (b) or report N/A with the measured day.
- Boards: `villages.png` (FV frames with and without selection, labels readable at 1×), `founding.png` (departure, walk, site, founding), `deaths.png` (the five death colours and feed lines), before/after of d2 12:00 F2, the society baseline vs the final frame. Commit, push.

---

# 13. PART 9 — Final gate, compatibility, report, PR
1. Full gate: `GATE_OUTPUT_DIR=docs/art/production7/gates/final bash tools/art_pipeline/gate.sh final`. Fast-forward `codex/art-production-2026-10` if green; otherwise say exactly what is red.
2. Budgets: `assets/art/**` ≤ 25 MB, `docs/art/production7/**` ≤ 30 MB (JPEG unless a check needs lossless).
3. An independent review of the final screens (two villages with labels, the walk, a daughter's cold night, the five death colours) with verdicts `ACCEPT` / `NEEDS WORK` and reasons, recorded as given.
4. `docs/art/production7/REPORT.md` in Korean in the section 15 format, answering every item of section 14 with its evidence.
5. Update PR #511's description with an Art Pass 7 section (`gh pr edit 511 --body-file …`; say so if `gh` is not authenticated). Keep it a draft.

## 13.1 Localization (en / ko, every key in both; check for duplicates first)
| key | en | ko |
|---|---|---|
| HUD_FEED_DEATH_EXPOSURE | A villager froze to death | 주민이 얼어 죽었다 |
| SETTLEMENT_LABEL_FMT | Village %d | %d번 마을 |
| SETTLEMENT_LABEL_DAUGHTER_FMT | Village %d · from %d | %d번 마을 · %d번에서 |
| SETTLEMENT_ABANDONED_FMT | Abandoned village %d | 버려진 %d번 마을 |
| HUD_SETTLEMENTS_FMT | Villages %d | 마을 %d |
| GOD_HAND_TARGET_FMT | God's hand — %s | 신의 손 — %s |
| GOD_HAND_APPLIED_TARGET_FMT | %s: %s %s — applied %d | %s: %s %s — 적용 %d |
| HUD_FEED_FISSION_DEPARTED | %d people left Village %d | %d명이 %d번 마을을 떠났다 |
| HUD_FEED_SETTLEMENT_FOUNDED | Village %d was founded | %d번 마을이 세워졌다 |
| HUD_FEED_FISSION_CANCELLED | A group turned back | 무리가 돌아왔다 |
| HUD_FEED_SETTLEMENT_ABANDONED | Village %d was abandoned | %d번 마을이 버려졌다 |

---

# 14. 인게임 확인사항 (게임을 직접 켰을 때 확인할 것)

1. 게임이 game_v17로 시작하고 콘솔에 SCRIPT ERROR가 없다.
2. 추위로 죽은 사람이 상단 사망 수에 포함되고, 알림 줄에 "주민이 얼어 죽었다"가 뜨며, 사망 표시가 전투의 빨강이 아니라 옅은 얼음색이다.
3. 분열하는 세계(시드 20261013 등)에서 Part 0이 잰 출발일 아침(29일차 무렵) 한 무리가 짐을 메고(짐 표시를 생략했다면 보고서에 명시) 점선을 따라 깃발 쪽으로 걸어가고, "N명이 1번 마을을 떠났다" 알림이 뜬다. 도착하면 "2번 마을이 세워졌다"가 뜨고 깃발과 점선이 사라진다.
4. 마을마다 이름표("1번 마을", "2번 마을 · 1번에서")와 색이 있고, 상단에 마을 수가 보인다. 이름표를 누르면 그 마을 사람들만 테두리가 생긴다.
5. 딸 마을을 고른 뒤 신의 손으로 비를 내리면 그 마을에만 비가 오고, 원래 마을의 불은 그대로다.
6. 사람이 모두 떠난 마을은 회색으로 남고 "버려진 N번 마을"로 표시되며, 버려진 화덕에서는 연기가 나지 않는다(회색 표시를 생략했다면 보고서에 명시; 연기 없음은 생략 불가).
7. 기본 시드(20261010)의 겨울 밤·아침·가을 운반 장면이 v17에서 새로 잰 날짜로 다시 찍혀 있다.
8. 콘솔에 SCRIPT ERROR가 없고, 마을이 셋일 때도 프레임 시간이 크게 늘지 않는다.

---

# 15. 구현 완료 보고 (REPORT.md와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
UI의 한-마을 가정 감사와 수정, 사망 표시 바로잡기(합계·알림·색), 사회 스냅샷 FFI 조회 함수, 마을 이름·색·선택·소속 표시, 신의 손 대상 선택, 버려진 마을, 떠나는 무리(알림·점선·깃발·짐), 측정한 v17 날짜

### 실제 게임 화면 검증
A1~A30 표(수치), 블라인드 결과(관찰자 2명, A27·A28), 자연·사회 시나리오·개입 장면의 구분, 보드 경로

### 시뮬레이션 작업과의 호환
병합한 시뮬레이션 커밋과 결과, 추가한 FFI 조회 함수(다음 시뮬레이션 작업이 이어받을 것), v17에서 잰 날짜와 사회 시나리오 시드

### 변경된 파일 목록
(커밋별)

### 자율 결정 목록
DECISIONS.md 요약

### 확인된 제한사항

### Harness 결과
(cargo test 수치, clippy, strict, bridge, smoke, Python, Godot 아트 장면 테스트)

### 정직한 판단
처음 보는 사람이 화면만 보고 마을이 몇 개인지, 어느 마을이 어디서 갈라졌는지, 저 무리가 새 마을을 세우러 가는 중인지 알아챌까? 아직 아닌 부분은 무엇인가?
```

Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the simulation commit merged, whether PR #511's description was updated, image calls used, and the paths of `MEASURED_DAYS.md`, the UI audit, `villages.png`, `founding.png`, `deaths.png`, the before/after and the review boards.
