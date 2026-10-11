# GOAL: WorldSim — Art Pass 5: use what was approved, hair masks by position, wood and walking art, people who differ, room to stand, cold nights on screen

Repository `hyunlord/new-world`. Continue the art line: `codex/art-production-2026-10-wip` and `codex/art-production-2026-10`, both at `c6d88d625a86dfdc06cebe48923ae65ff1455cfb` (Art Pass 4; draft PR #511, stacked on the simulation PR #510).
The simulation's Pass 6 (winter that bites) is sealed: `codex/revitalize-2026-10` and `-wip` are both at `fe60de21249ad46ad2e2d9ca420d896066277c82` (code head `01fd135d`, gate green; the live game builds `game_v12`). Pass 6 adds two bridge keys: agent snapshot `colds` (PackedByteArray, 1 while the agent carries the `Cold` marker: in winter, inside the cold window 22:00–06:00, every member of a settlement whose fire is out) and settlement snapshot `wood_targets` (PackedInt32Array, the season's wood target, the same value in every row, 0 without the firewood rule). In Pass 6's 120-day probes the fire goes out on the last night of most winters (124 of 128 seed-winters). Art Pass 4 checked compatibility only up to `70d2224e`, locally.
This run is executed by **Codex**, unattended, on the Mac.

---

# ⚠️ 0. AUTONOMY — READ FIRST

1. **Never ask the owner anything.** Decide, implement, record the decision and its reason in `docs/art/production5/DECISIONS.md`.
2. **First action:** save this document unchanged as `docs/art/production5/GOAL.md` (the owner may already have placed it there; then verify it is this document), commit it in Part 0, and re-read the section of the current Part at every Part start and after any context compaction.
3. **Do not stop between Parts.** One task per Part; push to the WIP branch after every Part.
4. **Review loops: two rounds maximum.** Fix once, re-review once; then accept, record, move on.
5. **On failure: work around, note it, continue.** Partial delivery with an honest report beats a stalled run.
6. **Time rule:** record start time and each Part's start and end in `docs/art/production5/TASKS.md`. Art Pass 4 took 1 h 40 min. Past **6 hours**, finish the current step, push, go to Part 9.
7. **Three strikes** per failing target, then keep the best state and record evidence and hypothesis.
8. **Drop order when time is short:** Part 4 calls after its first call (8.6) → breath puffs (10.3 b) → round 2 of the mask review (7.6) → the cold-night capture frames (11.1). Never drop sections 5 and 6, 7.1–7.5, 7.7–7.9, Part 4's first call with its review and install, section 9, section 10 (except breath puffs), section 11 (except the cold-night frames), section 12 or section 13.
9. **Image budget: at most 8 new generations**, failures included. On a quota or rate-limit message, stop generating and continue.
10. **Honesty rules stay:** never mark a failed check as passed, never lower a threshold of section 12, keep rejected evidence, never edit or re-interpret an earlier verdict. An earlier `ok` verdict carries over only under the binding of 5.1. The scope corrections of 12.1 record the old and new definition side by side and report both results.
11. **This brief supersedes the repository's session-start instructions** (harness hook installation, the `lead/main` workflow, `harness_pipeline.sh`): do not install hooks, do not run the harness pipeline, never push to `lead/main` or `main`, never force-push.

---

# ⚠️ 1. WHERE AND HOW

- Mac only, the existing clone `/Users/rexxa/work/new-world-art-prod`. Run every pipeline script from the repository root (recovery ledgers use relative image paths). New raw generations go to `/Users/rexxa/work/art-production5-raw/<sheet_id>/attempt_<n>.png`; capture evidence goes to `CAPTURE_EVIDENCE=/Users/rexxa/work/art-production5-evidence`. The earlier raw folders (`art-production-raw`, `art-production2-raw`, `art-production3-raw`, `art-production4-raw`) are read-only (the wood fingerprints of 5.1 read the raw crops in `art-production4-raw`). Raws never enter git.
- Never touch `/Users/rexxa/work/new-world-pass4` (the simulation clone), `~/github/new-world-wt/lead`, `/Users/rexxa/work/new-world-capture` or any other checkout; never kill a process you did not start; no ComfyUI, no dev servers.
- `export CARGO_BUILD_JOBS=6`; cargo tests `-- --test-threads=6`. `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`. Pipeline Python uses Pillow and numpy only (no scipy).
- Windowed Godot only through `tools/art_pipeline/with_window_lock.sh <seconds> <command...>` in one command; from a background session run that command in the GUI session with `launchctl asuser $(id -u) …`, still inside the lock. Keep every locked command under 25 minutes (the simulation's lock script treats a lock older than 30 minutes as stale). Headless Godot needs no lock; other limits use `tools/art_pipeline/run_bounded.sh`.
- Image generation: the built-in `$imagegen` skill (gpt-image-2), magenta background, returns 1254 × 1254. Prompts never name existing games, studios, artists or characters. Use the Art Pass 4 prompt form (`docs/art/production4/wood/prompts/V_a4.txt`): 4 × 4 grid, cell list, Image 1 the D3 style reference, Image 2 the body's identity reference at 8× nearest, "each figure holds its own physical prop; no floating icons".
- Scripts of Art Pass 4 that write under `docs/art/production4/**` (for example `wood_candidates_p4.configuration()`, `apply_natural_audit_p4.py`) are copied to `_p5` versions writing under `docs/art/production5/**`, never called as they are.
- **Independent review** means a fresh reviewer session or subagent that has not seen the pipeline code, sections 3 and 5–8 of this brief, or earlier verdicts for the same candidates. The blind activity test (A11) keeps Art Pass 3's procedure: freeze the answer file first, give each observer only the images and the allowed labels.

---

# 2. GIT

- `git fetch origin && git checkout codex/art-production-2026-10-wip`; it must equal `origin/codex/art-production-2026-10-wip` (`c6d88d62`). Local-only commits (for example Art Pass 4's unpushed compatibility merge `f8aa2ade`) go to a backup branch `art-p5-local-backup` first.
- **Part 0 merges the simulation green head:** `git merge --no-ff <head> -m "art-p5(p0): bring in simulation pass 6"`, where `<head>` is exactly `fe60de21249ad46ad2e2d9ca420d896066277c82` (Pass 6), even if `origin/codex/revitalize-2026-10` has moved; record the hash. Pass 6 changed no art-owned path, so the merge should be clean. Conflict rules as in Art Pass 4: under `rust/crates/sim-*/src/**`, `docs/revitalize/**` and `tools/revitalize/**` take the simulation's side; in Rust static-source tests that pin GDScript keep the art assertions and the simulation's new assertions both; record every resolved file.
- **Merge no later simulation head in this pass.** The simulation's Pass 7 may run at the same time and moves green to `game_v13` (winters of different severity, a new FFI command); this pass captures and measures against Pass 6's `game_v12`, and the next art pass merges v13. If `origin/codex/revitalize-2026-10` has moved, record its hash in DECISIONS.md and leave it.
- WIP gets every commit; `codex/art-production-2026-10` moves only by fast-forward to a WIP commit whose full gate (Part 9) is green. Never force-push. After every push `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`; serialize git mutations and that check.
- Commit messages `art-p5(pN): <what changed>`; stage explicit paths only.
- **May change:** `scripts/**`, `scenes/**`, `shaders/**`, `assets/**`, `localization/**`, `project.godot`, `docs/art/**` (new files only under `docs/art/production5/**`, plus `docs/art/TEST_LEDGER.md`), `tools/art_pipeline/**`, Rust static-source tests under `rust/crates/sim-test/tests/` that read `.gd`/`.tscn`/`.gdshader` files (ledger row each in `docs/art/TEST_LEDGER.md` under `## Art Pass 5`, mutation audit for the changed assertions only).
- **Must not change:** anything else under `rust/`, `docs/revitalize/**`, `tools/revitalize/**`, `tools/harness/**`, `.claude/**`, `.harness/audit|baseline|reviews/**`, and any evidence file under `docs/art/production{,2,3,4}/**`.

---

# 3. WHY THIS PASS (planner's review of c6d88d62)

The planner measured each point on the committed evidence of `c6d88d62`; an independent check reproduced every number below.

1. **Approved art was thrown away by guards, not by the review.** Art Pass 4 installed 6 of 130 approved recovery candidates. Of the 206 candidates the review rejected 76; technical guards then excluded 93 approvals — frame guards 77 (`despeckle_over_3_percent` 48, `geometry` 24, `gross_colour_error` 13, `standing_height` 3, some together) and group guards 16 (`head_top` 14, `group_median_height` 2); 37 reached combination trials and 6 were installed. All 120 frame-guard failures among the 206 were already stored in the Art Pass 3 ledgers before the review began; the planner counted only review reasons (planner's error).
2. **The despeckle limit is redundant and scale-biased.** Reviewers saw the despeckled sprite (`reprocess_p3.py` builds `native` and `sprite` from `cleaned.image`). Median share of changed texels on the 206: child 2.72 % (n = 113), adult_a 1.71 % (20), adult_b 1.89 % (73); 49 of the 113 child candidates exceed 3 %; the 48 excluded approvals changed 4–13 texels each.
3. **The search spends its strikes on combinations that cannot pass.** `choose_lowest` (`merge_p4_selection.py`) ranks each target's candidates by colour error, walks their product and counts every failed combination as a strike against each target in it, while `validate_physical_group` rejects any group that mixes (sheet, attempt). For child E run all four cells of `V_c2` attempt 3 are approved and pass every guard, but cell 06's attempt-2 frame (ΔE 8.37 against 11.79) ranked first, three mixed combinations used up the strikes and the valid set was never tried. Run is not shown in game; the defect is general.
4. **A cycle with one bad or missing frame is dropped.** Carry groups failed `pending_cycle_completion`; recovery never used the same-phase copy of Art Pass 2/3 (`salvage_v4._derive`, rule `same_cycle_phase_copy`).
5. **What is recoverable today** (planner's re-run of `guard_group` and `_leg_difference` on the stored images, with despeckle as a warning, one physical set per animation, at most one same-phase copy per four-frame cycle, two-pose for carry):
   - child walk S, E, N — `P2_fill_03` attempt 1, cells 01–12: all approved, equal heights per view, leg difference ≥ 21, ΔE 6.6–8.8; the nine frames that fail do so only on despeckle (3.5–4.6 %). Today all 12 child walk frames are procedural.
   - carry, 20 frames, 13 of them real: adult_a S (`P3_fill_01` cells 01, 03, 04, frame 1 a copy of cell 04), adult_a E (cells 06, 07, two-pose), adult_b S (`P2_fill_02` cells 05, 07, 08, frame 1 a copy of cell 08), adult_b E (cells 10, 11, 12, frame 0 a copy of cell 11), child E (`P2_fill_06` cells 02 and 03, two-pose; cells 01 and 04 are 23 px tall and fail the group median). Today no body has a real carry frame.
   - child build S (`P3_fill_03` cells 13, 14); child run E (`V_c2` attempt 3, not shown in game).
   - W1: 9 of the 13 approved objects were excluded — despeckle 5 (`W1_03`, `05`, `07`, `08`, `16`), 1 px over a size cap 4 (`W1_01`, `02`, `04` are 17 px tall against 16; `W1_15` is 25 px wide against 24). With them every deadwood stage and the wood icon have real art.
6. **Hair masks by colour cannot work.** 45 of the 76 recovery rejections are `hair_mask_wrong` (43 of the 73 adult_b candidates: long hair below the 0.62 H reach, dark inner hair left out, arm texels taken in). 31 of the 32 adult wood cells (`V_a4`, `V_b4`) were rejected for the mask although the reviewers wrote that the sticks read at 2×. In the 116-colour palette 10 of adult_a's 11 hair colours and 5 of adult_b's 8 also occur in the W1 wood sprites, and the seed rows (top 34 %) contain the shoulder load. The natural-colour audit rejected 29 installed adult_b masks; all 29 frames are still drawn (27 with the rejected masks, `V_b3_03` and `V_b3_12` with newer unreviewed masks), so white-haired adult_b elders show white crowns over dark long hair.
   The planner prototyped masks moved by position from approved masks of the same body and view (7.3) on these 105 frames. On the 74 recovery and audit frames most masks covered the whole hair including the long hair; the failures were build poses with a raised arm inside the hair template and head bands aligning below IoU 0.60. On the 31 wood frames it failed: the load sits where long hair would be, and in front views the load inside the head band breaks the alignment. Wood therefore needs new art drawn with a hair key (section 8).
7. **Wood frames break the width caps.** Adults 22 px against 16–29 px with the load, children 17 px against 16–26 px. Only adults gather wood (Pass 5), so child wood frames stay out of scope.
8. **A12 was set above its own expectation (planner's error).** The d2 world has 24 adults and 2 children. With 2 bodies × 3 adult hair colours × 3 tones = 18 tuples, 17 random adults give 11.2 distinct tuples on average and reach 12 with probability 0.41. With 2 × 5 × 4 = 40 the expectation is 14.0 and the probability 0.97. This pass changes the design (more variety), not the check.
9. **Crowding.** Of the 9 half-hidden villagers in d18 and d24, 5 stood outside seat radius 4, 2 had no legal seat within 48 px (the remaining candidates were water), 1 was walking and 1 was seated but still covered: seat rows are 14 px apart for sprites 30 px tall. A5's measurable share at d24 (25/42 = 59.5 %) fails for the same reason; its contrast passed (0.345, 0.354).
10. **Weak or missing evidence.** A11: child 1/3 at both scales (the child walk is procedural); carry read as drink (carry is idle + icon). A7 passed on 8 moving samples. A17 passed with 0 real wood frames.
11. **Pass 6 is invisible.** Cold nights exist in the simulation; nothing on screen shows them. The wood target exists; the label shows the stock only.

**Decision.** Use what was already approved before drawing anything new: guards that only repeat the review become warnings, the search follows the one-set-per-animation rule instead of fighting it, and a missing or bad cycle frame may be replaced by its same-phase twin. Hair masks are moved by position from approved masks, not chosen by colour. New adult art is drawn with hair in a reserved key colour, so its mask is exact by construction. Adults get five hair colours and four tones. Stationary villagers get room. Cold villagers fold their arms, shiver and breathe out. Accepted costs: copied and two-pose cycle frames; child wood frames unused.

---

# 4. PART 0 — Baseline

1. Record start time, `ART5_BASE_SHA`, `uname -a`. Commit `GOAL.md`.
2. Merge the simulation green head (section 2). Full gate: `GATE_OUTPUT_DIR=docs/art/production5/gates/p0 bash tools/art_pipeline/gate.sh p0`. Fix merge fallout in art-owned paths only and record it.
3. Record that the live game builds `game_v12` after the merge and that `colds` and `wood_targets` appear in its snapshots.
4. Commit, push WIP.

---

# 5. PART 1 — Harvest the approved frames (no image calls, no new frame verdicts)

## 5.1 Pools and binding
- Recovery: ledger `docs/art/production4/recovery/candidates.json` (206 records), verdicts `docs/art/production4/recovery/reviews/final_reviews.json`, fingerprints `docs/art/production4/recovery/review_index.json`.
- Wood: ledger `docs/art/production4/wood/processed/candidates.json` (64), verdicts `docs/art/production4/wood/reviews/final_reviews.json`, fingerprints `docs/art/production4/wood/processed/review_index.json`.
- An `ok` verdict carries over only if `salvage_verdicts.candidate_fingerprint(record)` of the unchanged ledger record still equals the stored fingerprint (206/206 for recovery at `c6d88d62`). Never edit those ledgers.
- Child `carry_wood` and `gather_wood` cells (`V_c3`) are not installed: children do not gather wood.

## 5.2 Guard changes (contract v5)
The rules below replace the old ones wherever conversion, selection, installer and QA apply them: `v4_contracts.py`, the `rejected` flag of `despeckle.py`, `reprocess_p3.py`, `wood_candidates_p4.py` and `merge_p4_selection.py` (the 3 % limit lives in all four). Stored results in old ledgers stay as history. Update the tests that pin the old values and record each.
- **Despeckle:** changed share > 3 % → warning `despeckle_over_3_percent`; > 10 % → error `despeckle_over_10_percent`. Use each record's stored despeckle counts or, for wood records, `metrics.despeckle_change_fraction`; never re-convert.
- **Wood width caps:** `carry_wood` and `gather_wood` frames may be 30 px wide (adult) or 26 px (child); height caps unchanged.
- **The bend:** a `gather_wood` frame whose record `frame` is 0 is exempt from `standing_height`, from the wider-than-tall rule of the hair-mask contract (`hair_registry_p4.py`, `hair_freeze_p4.py`) and from the adult reference heights of `guard_chosen`; that group's `group_median_height` uses its other frames. Pass the record's `frame` into every guard call that needs it (`FrameSpec` has no frame index).
- **Figure top:** adult `carry_wood` and `gather_wood` frames that have a hair mask measure `standing_height`, `group_median_height` and `head_top` from the first row of the hair mask instead of the first opaque row (a load can rise above the head).
- **Wood anchors:** `carry_wood` and `gather_wood` frames take anchor x from the centre of their opaque texels in the bottom 3 rows instead of `width // 2`, so an asymmetric load does not move the body between frames.
- **W1 caps:** deadwood cells 1–8 24 × 18 (was 24 × 16); woodpiles 9–11 and 13–15 26 × 20 (was 24 × 20); icon cells unchanged.
- Everything else stays: palette membership, binary alpha, colour error > 12, feet baseline, `head_top` ≤ 1 px, leg difference ≥ 4, group median height ±1, cross guards.

## 5.3 Completions of a physical set
A physical set is (source sheet, attempt). Its usable frames are its approved frames that pass the v5 frame guards. For a four-frame cycle (walk, run, carry, carry_wood) the completions are:
- **C0:** all four usable frames.
- **C1:** for one frame f, a copy of the usable frame of the same parity (f ± 2) takes f's place — whether f is missing or usable but breaks a group guard. At most one copy. Use `salvage_v4._derive` with rule `same_cycle_phase_copy` only (if no same-parity frame exists there is no C1; its procedural fallback is never used); when the target cell has no ledger record, build a minimal record from the set's sheet, attempt, cell, body, view, activity and frame. Copies are written under `docs/art/production5/harvest/derived/`; status `derived`.
- **C2 (carry and carry_wood only):** two usable frames p (even) and q (odd); frames 0 and 2 show p, frames 1 and 3 show q; warning `two_pose_cycle`.
Two-frame groups (for example carry N, which the guard already marks `two_frame_cycle`) and every non-cycle activity take all their frames from one set, all usable, with no copies. A completion must pass `guard_group` and the cross guards. Copies are never made from unapproved frames or across sets.

## 5.4 Search
For each target group (body, view, activity) rank every completion of every set: (number of copies, max colour error, mean colour error, attempt, C0 before C1 before C2). Walk the ranking; guard failures cost nothing (they are computed); the first completion that passes every guard goes to the animation review (5.6). A review rejection is one strike for the group; three strikes end the group; after a rejection continue down the ranking. Never form a combination that mixes sets. Objects (W1): per slot, fewest warnings, then lowest colour error. Record every completion tried and every set left untried with its reason in `docs/art/production5/harvest/search.json`.

## 5.5 What may be replaced
A group is replaced only by a complete set, and only when every installed frame of the group is `procedural`, `derived` or `fallback`, or the new completion has more real (uncopied) frames than the group has `accepted` frames. Groups are never mixed. Once a body has real carry frames its idle + floating icon disappears (Art Pass 3's 7.5 rule) and borrowed carry views give way to real ones.

## 5.6 Animation review
Before any villager group is installed or changed (here and in sections 7 and 8), a fresh independent reviewer judges it as an animation: the frames in order at 4× on grey and at 2× on grass, a looping GIF at 2× (8 fps for cycles, 4 fps otherwise), copied frames marked, the body's reference idle beside them. Verdict `ok` or `reject` with one reason: `cycle_pop` (a frame jumps in height, position, proportions or colour), `wrong_pose`, `wrong_view`, `different_person`. Boards and verdicts in `docs/art/production5/harvest/animation/`.

## 5.7 Selection and install (`tools/art_pipeline/merge_p5_selection.py`)
The Art Pass 4 path cannot install these frames as it stands: `install_v4.py` needs a verdict row whose `candidate_id` equals the record id (copies have new ids), rejects any stored `guards`/`group_guards` error (the recovery records carry the old despeckle and geometry errors), recomputes the fingerprint (any added field breaks it) and re-runs `guard_frame`/`guard_group` in `validate_assembled` without despeckle, mask-top or frame data; `merge_p4_selection.py` needs a review for every candidate. Write `merge_p5_selection.py`, which:
- checks each original record's fingerprint against its review index before use;
- writes `docs/art/production5/harvest/installer_candidates.json` from the unchanged records with `guards`, `group_guards` and `cross_guards` set to the v5 results and the originals kept as `guards_p4`;
- gives each copy an envelope verdict row copied from its source's `ok` row, marked `inherited_from: <source candidate id>`;
- passes the despeckle share, the hair-mask top and the record `frame` into `validate_assembled`;
- freezes the reviewed masks of installed adult frames into `tools/art_pipeline/hair_masks_p5/` (source `p4_stored`, 7.7);
- leaves `install_v4.build_plan` and `apply_plan` as the only writers of manifest, atlas and region pages.
Record drawables by status and fallbacks by kind, before and after (Art Pass 4: 273 = accepted 152 + derived 3 + procedural 51 + fallback 67; fallback = retained_installed 41 + idle_icon 18 + borrowed_carry_view 8). Compare with section 3 point 5 and explain every difference.

## 5.8 Renderer
`AgentActivity._real_carry` also accepts entries with `qa_status` `derived` whose `source` is `derived:same_cycle_phase_copy`; a body counts as having real carry only when at least one of its carry entries is `accepted`. Without this, a C1 carry group would loop only its three real frames.

## 5.9 Tests
Python: `test_v4_contracts.py` updated (despeckle warning and error bands, wood width caps, the bend, the hair-mask figure top, wood anchors, W1 caps); `test_harvest_p5.py` (the `V_c2` case: a complete set ranked below mixed ones is found on the first try; no mixed combination is ever formed; C1 replaces a usable frame that breaks `head_top`; C2 only for carry; no procedural derivation; a changed record fingerprint drops its verdict; a copy's envelope row names its source). Godot: `test_agent_activity.gd` (real carry frames for every body that has them; a C1 carry group plays all four frames in order; a two-pose cycle loops). Commit, push WIP.

---

# 6. PART 2 — Five hair colours and four tones

## 6.1 Ramps
`assets/art/villager_ramps.png` keeps its 15 × 3 layout: rows 0–3 skin (unchanged), rows 4–10 hair, rows 11–14 the first four of the old clothing ramps (skin and clothing rows are unread since Art Pass 4). Hair rows, columns light, mid, dark — every colour is already in the production palette:

| hair | row | name | light | mid | dark |
|---|---|---|---|---|---|
| 0 | 4 | black | (63,60,54) | (41,38,33) | (16,11,3) |
| 1 | 5 | dark brown | (99,89,69) | (75,66,47) | (47,40,22) |
| 2 | 6 | brown | (149,120,89) | (122,95,65) | (91,67,38) |
| 3 | 7 | auburn | (168,99,41) | (136,65,25) | (87,36,17) |
| 4 | 8 | flaxen | (202,196,141) | (174,169,115) | (142,137,85) |
| 5 | 9 | grey | (187,185,183) | (160,158,156) | (129,127,125) |
| 6 | 10 | white | (242,241,240) | (217,216,217) | (187,185,183) |

Update `tools/art_pipeline/production_palette.json` `rgb_ramps.villager` (hair 7 ramps, clothing 4) and `variant_names` to match; `test_production_palette.py` then passes unchanged (shape, membership, descending lightness).

## 6.2 Shader
`shaders/villager_ramp.gdshader`: `row = 4.0 + clamp(variant_codes.g, 0.0, 6.0)`; tone = 0.88 when r < 0.5, 0.96 when r < 1.5, 1.04 when r < 2.5, else 1.12; comment `r = tone 0..3, g = hair 0..6`. Everything else as Art Pass 4 (texels with luma < 0.10 stay as drawn).

## 6.3 Variants
- `ArtHash.look`: `"tone": posmod(h >> 9, 4)`; hair stays `posmod(h >> 11, 5)`; keep the `skin` and `clothing` lines (Rust static tests assert them).
- `AgentActivity.elder_look`: elders `5 + posmod(ArtHash.mix(agent_id), 2)`; everyone else `posmod(hair, 5)`.
- `apply_variant` unchanged: `Color(tone / 15.0, hair / 15.0, 0.0, 1.0)`.
- Calibration scene `ramp_variant` actors: `{"tone": index % 4, "hair": index % 7}`.

## 6.4 QA emulation and boards
`ingame_qa_types.sprite_pixels(…, variants=True)` uses the new rows and tones; `RAMP`, `A4` and A16's colour checks follow; tolerance ±1 per channel. Every board or QA script that reads a ramp row looks it up as 4 + hair code (for example `natural_boards_p4.py` used row 8 for white; white is now row 10); `ingame_qa_age_p3.py` expects non-elder hair 0–4 and elder hair 5 or 6.

## 6.5 Tests
`test_art_hash.gd` (tone 0–3, each 22–28 % of 3,000 ids; hair 0–4, each 17–23 %), `test_elder_variant.gd` (elders 5 or 6 only, everyone else 0–4), `test_variant_codes.gd`, the Python emulation and palette tests. Commit, push WIP.

---

# 7. PART 3 — Hair masks by position

## 7.1 Scope
Re-mask (a) the 45 recovery candidates rejected `hair_mask_wrong`; (b) the 31 `V_a4`/`V_b4` cells rejected `hair_mask_wrong`; (c) the 29 installed adult_b frames the natural-colour audit (`docs/art/production4/natural_audit/final_reviews.json`) rejected `hair_mask_wrong`. 105 in all; recompute and record. Frames with identical RGBA share one mask: re-mask such a set once, in the view of the slot the art was drawn for (for example `V_b2_05`–`08` are pixel copies of N-view art `V_b3_01`/`02`), and record every copy it applies to.

## 7.2 References
Per body and view (S, E, N): installed frames with `ok` in the natural-colour audit and their mask read through `reviewed_hair_maps_p4.json` (`mask_path`; the flat files `hair_masks_p4/{V_a2_15,V_a2_16,V_b3_13,V_c2_14,V_c2_16}.png` no longer match their sprites); recovery candidates with `ok` in the recovery `final_reviews.json` and their `sprite_hair_mask`; frames installed in Part 1. Work on the outlined sprite canvas (the stored masks are on it) and write the native mask from it. Exclude asleep, lying, `carry_wood`, `gather_wood` and masks under 6 texels. Planner's count: adult_a S 23, E 16, N 7; adult_b S 3, E 17, N 7.

## 7.3 Algorithm (`tools/art_pipeline/hair_masks_v5.py`, exactly)
1. Head band of an image: its opaque texels in rows [t0, t0 + round(0.34 H)), t0 = first opaque row, H = opaque height.
2. Place each reference bottom-centred on the frame's canvas, then search integer shifts (dx, dy) ∈ [−3, 3]² for the highest IoU of the two head bands (ties: smaller |dx| + |dy|, then smaller dy, then smaller dx).
3. Take the best same-activity reference if its IoU ≥ 0.75, else the best reference overall (ties: reference id). Best IoU < 0.60 → no new mask; record.
4. T = the reference mask moved by that shift; T⁺ = T dilated by one texel (8-neighbourhood).
5. Body hair colours = every colour seen at least 3 times inside the body's reference masks, except the outline colour (3,0,0), with no share threshold (the planner's prototype: share thresholds of 15 % and 40 % leave the dark long hair out again).
6. M = opaque texels of T⁺ that have a body hair colour and are opaque in the native (never the outline ring), reduced to the largest 8-connected component that meets T.
7. Fill, twice: a texel inside T, opaque in the native, with at least 5 of its 8 neighbours in M joins M.
8. Shade by the body's hair-colour luma terciles (Art Pass 4 rule).

## 7.4 Boards
Per candidate: the sprite at 6× in its own colours; at 2× on grass beside the body's reference idle; the mask overlay at 6×; previews at 4× with hair 6 (white) and hair 0 (black). `carry_wood` and `gather_wood` cells (adults only; elders never gather wood) show hair 4 (flaxen) instead of white.

## 7.5 Review (round 1)
Fresh independent reviewer, Art Pass 4's `REVIEW_RUBRIC.md` unchanged (its hair rule applies to the previews shown); wood cells also with Art Pass 4's wood lines (the bundle reads as sticks at 2× in all three views and is visible from behind).

## 7.6 Round 2
Only candidates rejected `hair_mask_wrong` in round 1, after one change to `hair_masks_v5.py` applied to every candidate of 7.1 (for example: drop colours whose share inside the reference masks is below 5 %). Record the change and the before/after counts.

## 7.7 Install and the mask registry
- Approved (a) and (b) candidates enter the Part 1 search (5.3–5.7).
- Approved (c) frames get their new mask. A frame whose current RGBA equals the audited image returns to its pre-audit status (`changes[].from` in `docs/art/production4/natural_audit/apply_report.json`: 21 were `accepted`, 2 `procedural`); any other keeps its status; record each.
- New and changed masks go to `tools/art_pipeline/hair_masks_p5/<frame_id>.png` with the registry `tools/art_pipeline/reviewed_hair_maps_p5.json` (frame id → mask sha256, `mask_path`, source `p4_stored` | `template_v5` | `hair_key`, verdict). `HairRegistry.load` and `regions.py` read both registries with p5 taking precedence for the same (body, activity, RGBA) key instead of raising "Conflicting reviewed masks"; every atlas repack rebuilds the region pages from these stored masks.

## 7.8 Tests
`test_hair_masks_v5.py`: hair-coloured wood outside T⁺ is excluded; long hair inside the template is covered; a dark inner patch inside T is filled; the outline ring is never in M; IoU < 0.60 gives no mask; same input, same mask. Registry tests for p5 precedence. Commit, push WIP.

## 7.9 Record for Part 4
Approved / re-masked for (a), (b) and (c) separately, and the list of 8.1.

---

# 8. PART 4 — New art with key-coloured hair (≤ 8 calls)

## 8.1 What to draw
After Part 3, list the target groups that have no installed frame, or have any installed frame that is `procedural`, `derived` or `fallback`, or that the natural-colour audit rejected `wrong_pose` or `wrong_view` while the slot's RGBA is still the audited one. Generate in this order, one call per sheet, skipping a sheet when none of its groups is on that list:
1. `V_a6`, adult_a, key hair: row 1 walking facing the viewer (left foot forward, feet passing, right foot forward, feet passing); row 2 walking facing right (the same four); row 3 carrying a basket of berries walking away (two steps), building facing right with a stone hammer (raised, striking); row 4 building facing away (raised, striking), gathering berries facing right (reaching into a bush, picking).
2. `V_a5`, adult_a wood, key hair: Art Pass 4's `V_a4` cells (rows 1–3 carrying a bundle of sticks facing the viewer, facing right, facing away, four steps each; row 4 bending to pick up a stick and standing up holding it, facing the viewer twice, facing right twice).
3. `V_b5`, adult_b wood, key hair: the same cells for adult_b.
4. `V_b6`, adult_b, key hair: row 1 building facing right (raised, striking), building facing away (raised, striking); row 2 gathering berries facing right (two), facing away (two); row 3 drinking facing away (two), carrying a basket walking away (two steps); row 4 arms crossed facing the viewer, standing idle facing the viewer (weight on the left foot, weight on the right foot), standing idle facing right.
5. `V_c4`, child, no key (children have no hair mask): row 1 carrying a small basket facing the viewer (four steps); row 2 gathering facing the viewer (two), facing right (two); row 3 drinking facing right (two), building facing right (two); row 4 hopping facing the viewer (two), talking facing the viewer (two).
Register the sheets in `docs/art/production5/fill_catalog.json` with `target_slot` set to the existing slot of the same body, view, activity and frame (for example `V_a5_01` → `V_a4_01`), processed as the Art Pass 3 fills were (`fill_catalog.load_catalog`, `fill_v4.process`). Never add them to `production_sheets.SHEETS` (every SHEETS cell becomes a target of its own, which would duplicate groups). Calls left over go to retries of sheets with fewer than half of their cells approved, best first.

## 8.2 Key colours
Three reserved colours, never in the production palette: light `#00DCD2`, mid `#00AAA2`, dark `#007D76`, in `tools/art_pipeline/hair_key_palette.json`.

## 8.3 Key reference and prompt
- Image 2 of a key sheet is `docs/art/production5/references/<body>_idle_key_8x.png`: the body's installed reference idle with its hair recoloured by its approved mask (shade 0 → light, 1 → mid, 2 → dark), enlarged 8× nearest. adult_a: the installed frame of slot `V_a1_01` (`P2_fill_01_01@1`) with its p4 mask (audit `ok`; the old reference `V_a1_01@1` is on installation hold). adult_b: the installed frame of slot `V_b1_01` only if Part 3 approved its new mask; otherwise draw adult_b sheets in Art Pass 4's form without the key and record it. Never use a rejected mask. A fresh reviewer checks each key reference once (teal covers exactly the hair; face, skin and clothing unchanged); one fix round; if still rejected, no key for that body.
- Prompt additions for key sheets: "The hair is painted only in flat teal #00DCD2, with #00AAA2 and #007D76 for darker hair shading, exactly as in Image 2. Teal appears nowhere else: not on skin, eyebrows, clothing, props, outline or background." For walking rows: "In the four walking cells of a row the head stays at the same height and the load stays in the same place."

## 8.4 Conversion
`tools/art_pipeline/key_candidates_p5.py`, modelled on `wood_candidates_p4.py` (v4 recovery and fit, adult height 28, child 21), quantising to the production palette plus the three key colours. For adult cells:
- The mask is the texels quantised to a key colour (light → shade 0, mid → 1, dark → 2), native and outlined; despeckle never changes a key texel.
- `mean_delta_e`, `colour_drift` and the 5.4 ranking errors are computed on the key-quantised native, before substitution; record both values.
- Then the key texels become the body's hair browns: the body's three most frequent colours inside its approved reference masks (7.2), ordered by lightness, mapped to light, mid and dark. The final native is 100 % production palette and reads correctly without the shader.
- New frame errors on key sheets: `key_hair_missing` (fewer than 6 key texels in an adult cell that is not asleep); `key_hair_outside_head` (more than 2 key texels outside the largest 8-connected key component, or any key texel below row t0 + round(0.80 H)); `key_fringe` (more than 2 texels 8-adjacent to the key component whose colour has hue 140°–200° and saturation ≥ 0.20 but is not a key colour — anti-aliased teal that quantised to a green).
- Key masks are exempt from the wider-than-tall rule of the hair-mask contract.

## 8.5 Review
Boards and rubric as 7.4–7.5. The raw cell is shown with the note that teal is a mask key by design and is replaced; `different_person` is judged against the natural-colour reference idle, never the key reference.

## 8.6 Checkpoint
After the first key sheet: at least 12 of its 16 cells free of `key_hair_missing`, `key_hair_outside_head` and `key_fringe`, and at least 8 of 16 approved. Otherwise stop key generation and record why; `V_c4` may still run.

## 8.7 Install
By the Part 1 rules (5.3–5.8). Record every call in `docs/art/production5/image_calls.json`. Commit, push WIP.

---

# 9. PART 5 — Room to stand

## 9.1 Seats (`scripts/ui/camp_seats.gd`)
- Lattice offsets (16·i + 8·(j mod 2), 18·j) for i ∈ [−7, 7], j ∈ [−4, 4], with (dx / 104)² + (dy / 72)² ≤ 1 and length ≥ 16 px (was 14·j inside 88 × 60, length ≥ 14).
- Free-seat separation 16 px (was 14); eligibility Chebyshev ≤ 6 (was 4); reach 64 px (was 48).
- The blocked-tile scan in `agent_renderer._apply_drawn_positions` covers hearth.y − 5 … + 5 and hearth.x − 7 … + 7.
- Everything else of Art Pass 4's 7.2 stays (eligible poses, persistence, ascending id order, tie rule).

## 9.2 Declump stationary villagers
After seats and the same-tile spread, `CampSeats.declump(rows, seated, targets, terrain, blocked, rects)` returns an extra horizontal offset for each stationary, unseated agent:
- rect(a) = the canvas rect of a's current frame (size and anchor from the manifest) at its target position; never read textures back from the GPU.
- Process agents front to back: descending (target feet y, id). covered(a) = the sum, over agents in front of a (larger feet y, or equal y and larger id), of area(rect(a) ∩ rect(b)) at their final targets, divided by area(rect(a)), capped at 1.
- If covered(a) ≥ 0.30: try shifts of 1 to 8 px, alternating sides, starting on the side away from the nearest front agent (equal x: right first for even ids, left first for odd ids); take the first shift with covered(a) < 0.30 whose feet tile is not water (from the terrain snapshot, anywhere on the map) nor a building, hearth, stockpile, construction site or deadwood tile; if none, the allowed shift that minimises covered(a); if every shift is blocked, 0.
- Moving and seated agents are never shifted (seated agents still count as front sprites). Targets go through the existing smoothing (τ 0.08 s).

## 9.3 Tests
`test_camp_seats.gd` (41 stationary idle villagers within 6 tiles of a hearth get distinct seats ≥ 16 px apart; nobody moves when another leaves; same input, same seats); `test_declump.gd` (two stationary villagers on vertically adjacent tiles with the same x end below 0.30; water on one side forces the other side; moving and seated villagers never shift; same input, same offsets). Commit, push WIP.

---

# 10. PART 6 — Winter on screen (Pass 6 data)

## 10.1 Feature detection
Read `colds` and `wood_targets` only when present; without them Art Pass 4's behaviour is unchanged. Tests cover both.

## 10.2 Cold pose
An agent with `colds[i] == 1` that is stationary with pose `idle`, `sit` or `hop` (not the meal pose, not asleep, not talking) shows `crossed`, frames in this order: `body/crossed/view`, `body/crossed/S`, `body/sit/view`, `body/idle/view`, skipping frames the natural-colour audit rejected `wrong_pose` or `wrong_view` while their RGBA is unchanged (today adult_b's only crossed frame `V_b3_16`). A seated cold villager keeps its seat (eligibility accepts `crossed`).

## 10.3 Shiver and breath (render only)
- (a) Shiver: a stationary cold agent's drawn x alternates +1 px and 0 every 1/6 s, phase (agent_id mod 6) / 36 s.
- (b) Breath: every visible cold agent, moving or not, puffs a 3 × 2 px white puff (`Color(1, 1, 1, 0.55)`) 2 px above the top of its frame every 2.4 s, phase (agent_id mod 24) × 0.1 s; the puff rises 4 px and fades to alpha 0 over 1.0 s. The texture is created at runtime (no new art file), drawn by `scripts/ui/cold_effects.gd` or `world_effects.gd` (record which). Capture metadata records each puff's alpha.

## 10.4 Labels
- Stockpile label (`building_renderer.gd`): split it into a food Label and a wood Label side by side. The wood Label reads `<HUD_STOCKPILE_WOOD> <wood>/<target>` when `wood_targets` exists and the value for the anchor's settlement is > 0 (all rows carry the same value; never sum targets), plain text, tinted with a new constant `WOOD_SHORT_TINT = Color(0.96, 0.62, 0.32)` when wood < target. Update the label parsing of `ingame_qa_wood_p4.py` (record).
- HUD top bar: `<HUD_COLD> <n>` after the wood label, n = agents with `colds == 1`, shown only when n > 0.
- Inspector: the line `INSPECTOR_COLD` when the agent's `colds == 1`.

## 10.5 Keys
Check duplicates first; add to `localization/fluent/{en,ko}/messages.ftl` and recompile with the compiler and registry step Art Pass 4 used.

| key | en | ko |
|---|---|---|
| `HUD_COLD` | Cold | 추위 |
| `INSPECTOR_COLD` | Cold — the fire is out | 불이 꺼져 추위에 떨고 있음 |

## 10.6 Tests
`test_cold_presentation.gd` (cold stationary idle → crossed; cold sit → crossed; the meal pose wins; moving cold → walk with puffs; not cold → unchanged; no `colds` key → unchanged; a body without usable crossed frames → sit); `test_hud_wood.gd` (target text and tint; no target → stock only); an inspector test for the cold line. Commit, push WIP.

---

# 11. PART 7 — Captures

## 11.0 Simulation check
Confirm the branch still contains `fe60de21` and no later simulation commit (section 2), then run `cargo test`, Clippy, strict, bridge and smoke before capturing.

## 11.1 Plans and metadata
- `ART` unchanged.
- `ART_LONG` (`scripts/test/art/capture_plans.gd`) = Art Pass 4's list plus the cold-night frames [23, "02:00", "F2N"], [25, "02:00", "F2N"], [27, "02:00", "F2N"] and [29, "02:00", "F2N"] (the night of day 28 is the last winter night, the one Pass 6 darkens most often), kept in chronological order.
- Metadata gains, per frame: agent `colds` and settlement `wood_targets`; per villager its cold presentation (pose, shiver offset, puff alpha), declump offset and seat; per installed group its source set and copied frames.

## 11.2 Run
As in Art Pass 4: `art` and `art_long` into `docs/art/production5/shots`, then `calibration` into `docs/art/production5/calibration`.

---

# 12. PART 8 — Measure

## 12.1 Scope corrections (record old and new side by side; report both results)
- **A7:** at least 30 moving-villager samples; if the stills give fewer, add the noon GIF's frames; report the count (Art Pass 4: 8).
- **A12:** the tuple keeps its form (body, hair for adults or −1 for children, tone) with hair now 0–4 and tone 0–3 (section 6); thresholds unchanged. Also report A12 counted with Art Pass 4's three hair colours and three tones.
- **A16:** elder hair 5 or 6, every other villager 0–4 (was 3 or 4 and 0–2).

## 12.2 Checks (thresholds unchanged unless stated)
- A1–A6, A8–A11, A13–A16 and A18 as in Art Pass 4.
- **A17** as in Art Pass 4, plus (f) every visible deadwood sprite uses W1 art for its stage when an approved W1 cell exists for it; (g) report how many visible wood carriers use real `carry_wood` frames.
- **A19 cold (new):** (a) the 10.6 tests pass; (b) in every captured frame with a visible villager whose `colds == 1`, each such villager follows 10.2 when eligible and shows a puff within its 2.4 s cycle, and no villager without the marker does; if no captured frame has a visible cold villager, (b) is N/A with the counts of cold villagers per frame; (c) every visible stockpile wood label shows wood/target where `wood_targets` > 0.
- **A20 harvest integrity (new):** every villager frame installed or changed in this pass traces to an `ok` frame verdict whose candidate fingerprint matches its review index, to an envelope row inherited from such a verdict, or to an `ok` verdict of this pass; every `derived` frame installed here is a pixel copy of an approved frame of its own physical set; every group installed or changed here (borrowed views included) has an `ok` animation review; no installed group mixes physical sets.

## 12.3 Before/after and sheets
`before_after.png` against Art Pass 4 at the same framings; season strip d10, d18, d24, d32; noon and dinner GIFs (40 frames, ≤ 8 MB each); `villagers_natural.png` (tones 0–3; adults hair 0–6); `harvest_strip.png` (every group installed in this pass at 2× in frame order, copies marked); `hair_masks_before_after.png` (the 29 audited frames: old and new mask, white preview); `cold_night.png` (a cold villager from a capture, else from the scene test). Commit, push WIP.

---

# 13. PART 9 — Final gate, compatibility, report, PR

1. Full gate on the final WIP head: `GATE_OUTPUT_DIR=docs/art/production5/gates/final bash tools/art_pipeline/gate.sh final`. Fast-forward `codex/art-production-2026-10` if green; otherwise say exactly what is red.
2. If `origin/codex/revitalize-2026-10` moved after your last merge, repeat Art Pass 4's local, unpushed compatibility merge and record it.
3. Budgets: `assets/art/**` ≤ 25 MB, `docs/art/production5/**` ≤ 40 MB; commit JPEG unless a check needs a lossless PNG.
4. `docs/art/production5/REPORT.md` in Korean in the section 15 format, answering every item of section 14 with its evidence.
5. Update PR #511's description with an Art Pass 5 section (`gh pr edit 511 --body-file …`; say so if `gh` is not authenticated). Keep it a draft.

---

# 14. 인게임 확인사항 (게임을 직접 켰을 때 확인할 것)

1. 아이들이 실제 걷기 그림으로 걷는다. 제자리에서 발만 바뀌는 걸음이 없다.
2. 바구니를 든 어른과 아이가 실제 운반 그림으로 걷고, 머리 위에 뜬 바구니 아이콘이 없다. 두 동작이 번갈아 나오는 운반도 끊기거나 튀지 않는다.
3. 마른 가지 더미가 주울 때마다 단계별로 다른 그림으로 줄어들고, 다 주운 자리에도 흔적이 남는다. 장작을 메고 가는 주민은 실제 장작 운반 그림으로 걷거나, 그 그림이 아직 없는 몸이면 장작 아이콘이 붙는다.
4. 어른이 장작을 메고 걸을 때 장작에 머리색이 번지지 않고, 짐 때문에 몸이 좌우로 튀지 않는다.
5. 흰머리 노인은 머리 전체가 희다. 긴 머리 아래쪽이 검게 남거나 팔·도구·짐이 하얗게 칠해지지 않고, 외곽선에 흰 테두리가 생기지 않는다.
6. 어른 머리색이 검정·짙은 갈색·갈색·적갈색·아마색 다섯 가지이고, 피부 밝기가 네 단계로 다르다. 회색과 흰 머리는 노인뿐이다.
7. 화덕 둘레에 앉은 주민이 서로 절반 넘게 가리지 않고, 화덕에서 떨어져 서 있는 주민들도 겹치지 않게 조금씩 벌어진다. 걷는 주민은 밀리지 않고, 물 위로 밀려나는 주민이 없다. 보이는 주민을 클릭하면 그 주민이 선택된다.
8. 겨울밤 화덕 불이 꺼지면 깨어 있는 주민이 팔짱을 끼고 떨며 입김을 내뿜는다. 불이 켜져 있거나 겨울이 아니면 그런 모습이 없다.
9. 저장소 라벨에 장작이 "205/273"처럼 목표와 함께 나오고, 목표보다 적으면 장작 부분만 주황빛이 된다. 추운 주민이 있으면 상단에 "추위 N"이 나오고, 그 주민의 정보창에 추위 줄이 나온다.
10. 콘솔에 SCRIPT ERROR가 없고 프레임 시간이 Art Pass 4와 비슷하다.

---

# 15. 구현 완료 보고 (REPORT.md와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
가드 조정, 세트 단위 탐색과 같은 위상 복사, 설치 경로(merge_p5_selection), 애니메이션 리뷰, 위치 기반 머리 마스크, 키 색 머리 생성, 머리색·톤, 자리와 떼어 놓기, 추위 표시와 라벨

### 회수 결과
Part 1: 설치된 그룹·프레임 수(체형·동작·방향별, 실제/복사), 상태별·대체 종류별 전후(Art Pass 4 대비), 계획자 예상과의 차이와 이유
Part 3: (a)(b)(c)별 재처리 수와 승인 수, 2차 변경 내용
Part 4: 생성 횟수(최대 8)와 시트별 승인 수, 체크포인트 결과

### 실제 게임 화면 검증
A1~A20 표(수치), 범위 수정 전후(A7·A12·A16, 이전 값과 새 값 모두), 블라인드 식별 혼동 행렬, 계절 스트립·before/after·회수 스트립·마스크 전후·추운 밤·GIF·주민 시트 경로

### 시뮬레이션 작업과의 호환
병합한 시뮬레이션 커밋(충돌과 해결), Pass 6 데이터(추위·장작 목표)가 실제 화면에 보이는지

### 변경된 파일 목록
(커밋별)

### 자율 결정 목록
DECISIONS.md 요약

### 확인된 제한사항

### Harness 결과
(cargo test 수치, clippy, strict, bridge, smoke, Python, Godot 아트 장면 테스트)

### 정직한 판단
처음 보는 사람이 이 화면을 완성도 있는 픽셀 아트 게임으로 볼까? 아직 아닌 부분은 무엇인가?
```

Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the simulation commit(s) merged, whether PR #511's description was updated, image calls used, and the paths of `before_after.png`, `harvest_strip.png`, `hair_masks_before_after.png`, `cold_night.png`, the season strip, the two GIFs, `villagers_natural.png` and the review boards.
