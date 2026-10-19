# GOAL: WorldSim — Art Pass 2: salvage the art we rejected, fix the scale, clear the forest

Repository `hyunlord/new-world`. Continue the Art Production 1 line: `codex/art-production-2026-10-wip` and `codex/art-production-2026-10`, both at `f3f2cafc950811ba7e798eb9cbf6e007e5d35dc7` (draft PR #511, stacked on the simulation PR #510).
This run is executed by **Codex**, unattended. A separate **Claude Code** run (simulation Pass 4) works on the same Mac in `/Users/rexxa/work/new-world-pass4`; section 1 says how to share the machine.

---

# ⚠️ 0. AUTONOMY — READ FIRST

1. **Never ask the owner anything.** Decide, implement, record the decision and its reason in `docs/art/production2/DECISIONS.md`.
2. **First action:** save this document unchanged as `docs/art/production2/GOAL.md` and commit it in Part 0. At the start of every Part and after any context compaction, re-read the section for that Part from that file; never work from a remembered number.
3. **Do not stop between Parts.** Keep a task list (one task per Part). Push to the WIP branch after every Part.
4. **Review loops: two rounds maximum.** Fix once, re-review once; on a second objection accept, record, move on.
5. **On failure: work around, note it, continue.** A partial delivery with an honest report beats a stalled run.
6. **Time rule.** Record the start time and every Part's start and end time. Past **8 hours**, finish the current step, push, go to Part 8.
7. **Three strikes.** If one target still fails after three fix rounds, stop on it, keep the best state, record evidence and your best hypothesis, move on.
8. **Drop order when time is short:** fill generation (8.1) → season tint (10.4) → death news feed (section 11, item 2). Never drop sections 5, 6, 7, 9, 10.1–10.3, 12 or 13.
9. **Image budget: at most 18 new image generations** (60 total minus the 42 already used), counting failures. If the tool reports a quota or rate limit, stop generating and continue with what you have.
10. **Honesty rules from Production 1 stay.** Never mark a failed check as passed, never lower a QA threshold of section 12, keep rejected evidence.

---

# ⚠️ 1. WHERE AND HOW

- **Mac only** (user `rexxa`), the existing clone `/Users/rexxa/work/new-world-art-prod`. Raw generations from Production 1 are in `/Users/rexxa/work/art-production-raw/<sheet>/attempt_<n>.png` with JSON sidecars; new raws go to `/Users/rexxa/work/art-production2-raw/<sheet_or_fill_id>/attempt_<n>.png`. Raws never enter git.
- **Never touch** `/Users/rexxa/work/new-world-pass4` (the running simulation Pass 4), `~/github/new-world-wt/lead`, `/Users/rexxa/work/new-world-capture` or any other checkout, and never kill a process you did not start.
- Share the CPU with Pass 4: `export CARGO_BUILD_JOBS=6`; cargo tests `-- --test-threads=6`. `gate.sh` and `capture_art.sh` hard-code `CARGO_BUILD_JOBS=8`; change both to `${CARGO_BUILD_JOBS:-8}` in Part 0.
- `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot` (else the first Godot 4.6 from `mdfind "kMDItemCFBundleIdentifier == 'org.godotengine.godot'"`).
- Every windowed Godot run goes through `tools/art_pipeline/with_window_lock.sh <seconds> <command...>` in one command (the lock is shared with Pass 4's captures). Headless Godot needs no lock. Use `tools/art_pipeline/run_bounded.sh` for every other time limit (macOS has no `timeout`).
- Python 3 with Pillow and NumPy only.
- Image generation: the built-in `$imagegen` skill (gpt-image-2). It returns 1254 × 1254 even when 1024 × 1024 is requested; that is fine, v4 measures the real grid. Prompts never name existing games, studios, artists or characters.

---

# 2. GIT

- Start: `git fetch origin && git checkout codex/art-production-2026-10-wip`. If the local branch has commits that origin lacks, keep them on a local branch `art-p2-local-backup` and record them in DECISIONS.md; then make the WIP equal to `origin/codex/art-production-2026-10-wip` (`f3f2cafc`).
- **Part 0 merges the simulation Pass 3 head** `a8611e07aabbb4043f5fac2e8dd14b375a0ada5d` (pinned; it is the head of `codex/revitalize-2026-10`, the base of PR #511) into the art WIP with `git merge --no-ff a8611e07 -m "art-p2(p0): bring in simulation pass 3"`. Production 1 already proved this merge clean (`ce6d9494`). Do not merge anything newer from the simulation branches except in the local, unpushed compatibility check of Part 7.
- Every commit goes to `codex/art-production-2026-10-wip` immediately; `codex/art-production-2026-10` moves only by fast-forward to a WIP commit whose full gate (Part 8) is green. Never force-push or rewrite history. After every push, `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`.
- Commit messages: `art-p2(pN): <what changed>`. Stage explicit paths only.
- **Paths you may change:** `scripts/**`, `scenes/**`, `shaders/**`, `assets/**`, `localization/**`, `project.godot`, `docs/art/**`, `tools/art_pipeline/**`, and the Rust static-source tests under `rust/crates/sim-test/tests/` that read `.gd`/`.tscn`/`.gdshader` files (each change gets a ledger row, section 4.4).
- **Paths you must not change:** everything else under `rust/` (simulation, bridge, data), `docs/revitalize/**`, `tools/revitalize/**`, `tools/harness/**`, `.claude/**`, `.harness/audit|baseline|reviews/**`.

---

# 3. WHY THIS PASS (evidence from Production 1, verified by the planner)

Production 1 installed a sound renderer (native pixels, integer zoom, y-sorted world objects, palette-ramp villagers, Korean HUD, night lighting, 5.9 ms median frame) but almost no art:

- **Good generated art was rejected.** 42 calls produced 659 candidate cells; 13 were accepted (2 %). In `docs/art/production/sheets/V_a1_attempt_1.jpg` the first adult sheet shows one consistent character in S, N and E idle and three walk cycles; `N1_attempt_1.jpg` shows good broadleaf trees, conifers, a dead tree, stumps, shrubs, a fallen log and a pale-bark tree. All were rejected.
- **The planner's acceptance rules caused it, not the generator:**
  - size: 588 of 659 cells (89 %) exceeded the class maximum. The adult maximum 18 × 28 was narrower than D3's own builder (20 × 28); the generator draws adults at a median native size of 19 × 33 (height 23–37), children at a median height of 29, trees up to 64 tall;
  - colour: 507 cells (77 %) exceeded mean ΔE76 6 against the fixed 116-colour palette (candidate median 6.81);
  - a frame was only judged once the same view's idle was accepted, so 373 cells (57 %) carry `missing_accepted_view_idle`;
  - prompting for smaller sprites at checkpoint 20 did not work: the generator does not follow texel-exact instructions.
  - Set aside those three rules and keep every other check (berry stages, fire ramp, child height, basket size, feet baseline, walk legs, exact icon canvas): **234 of 256 slots already have a passing candidate** among the 42 raws (216 if mean ΔE ≤ 10 is still required).
- **The fallback made it worse.** A missing view fell back to the D3 front gatherer (`direction_09_villager_gatherer`), which holds a basket of berries, so walk, gather, build, talk, eat, sit and every N/E view of adult A show the same front basket sprite. Many decals, props and effects reuse one brown blob. Blind activity identification (A11) scored 4/24 at both zooms.
- **The forest hides the village.** `scripts/ui/nature_renderer.gd:114` puts at least one tree on 88 % of forest tiles and two on 15 %, with crowns wider than a tile, so the forest renders as one canopy carpet: 8 villagers were unmeasurable behind trees and A5's minimum contrast was 0.055 (required 0.10). The trees themselves are procedural lollipops.

This pass recovers the rejected art with a corrected scale rule, puts a human-quality review in front of acceptance, replaces the one-sprite fallback, and clears the forest.

---

# 4. NON-NEGOTIABLE RULES

## 4.1 One world scale (decision)
Keep D3's scale: **adult standing height 28 texels, child 21**. Generated sprites larger than their target are re-gridded down (section 6); nothing is ever upscaled. Escape hatch, decided once in Part 2 and recorded: if the review (7.3) rejects more than 25 % of fitted villager cells as "detail destroyed by fitting" while the same cells unfitted are acceptable, switch villagers to the generator's own scale (adult 33, child 25) and multiply every non-villager target in 6.2 by 33/28 (rounded); never mix the two scales.

## 4.2 What acceptance means now
Automatic checks only guard against broken output (format, geometry, gross colour error, semantics). Whether a cell is good is decided by the independent visual review (7.3). Numbers never accept a cell the review rejected, and the review never accepts a cell that fails a guard.

## 4.3 Simulation contract
GDScript reads snapshots only. New behaviour keyed to Pass 4 data (`life_stages == 2`, `deaths_old_age`, reason 3 in `get_recent_deaths`, `season`/`day_of_season`/`days_per_season` in `get_time_snapshot`) must be feature-detected: with the Pass 3 bridge those keys are absent and the screen must look exactly as without this code.

## 4.4 Tests
Python tests next to the tools (`tools/art_pipeline/test_*.py`), Godot scene tests in `scripts/test/art/test_*.gd` (run by `gate.sh`). Rust static-source assertions you must re-pin get one row each in `docs/art/TEST_LEDGER.md` under `## Art Pass 2` (old assertion, new assertion, reason) and are run through `tools/art_pipeline/mutation_audit.py` for the changed assertions only. Never weaken an assertion that protects simulation-read-only, click radius or event meaning.

---

# 5. PART 0 — Baseline

1. `git status --short`, `git log --oneline -3`; record start time, `ART2_BASE_SHA`, `uname -a` in DECISIONS.md. Commit `GOAL.md`.
2. Merge `a8611e07` (section 2). Run the full gate: `GATE_OUTPUT_DIR=docs/art/production2/gates/p0 bash tools/art_pipeline/gate.sh p0`. If something fails only because of the merge, fix it in art-owned paths and record it.
3. Inventory the raws: for each of the 42 files record path, size, SHA-256 and the sheet/attempt it belongs to in `docs/art/production2/raw_inventory.json`; a missing or unreadable raw is recorded and its slots count as having no candidate from it.
4. Commit, push WIP.

---

# 6. PART 1 — Converter v4: fit to the target instead of rejecting

## 6.1 Algorithm (`tools/art_pipeline/pixel_normalize_v4.py`, reusing v3 pieces)
1. Key the cell and crop to the foreground box exactly as `recover_cell_v3` does; detect the grid with `v3_grid.detect_grid` → period `p` (shared axes unless v3 separates them), phase `φ`, native size `(w0, h0)` at `p`. If `p` is within 0.05 of a search bound (4.0 or 8.0), add the warning `grid_suspect`.
2. Scale factor `f ≤ 1` (6.2). New period `p' = p / f`, same phase origin.
3. Quantise every source pixel of the crop to the production palette (nearest in CIELAB, `quantize_native` logic). Build blocks at `p'` with `native_grid._blocks`. A block is opaque when at least half of its pixels are foreground; its colour is the **most frequent palette colour** among its foreground pixels (ties: lower L*).
4. Crop to the opaque box, run `remove_spill`, then the existing `pixel_normalize.outline` with the palette's darkest colour, as Production 1 does before packing.
5. Metrics: native sizes before and after, `p`, `p'`, `f`; mean ΔE76 between the per-block median colour (v3 style sample at `p'`) and the final colour over common opaque texels; palette membership; binary alpha; berry and fire texel counts (`v3_semantics.semantic_counts`).

## 6.2 Targets and factors
All sizes refer to the native sprite **before** the outline, as in Production 1.
- **Characters.** For each sheet attempt, group frames by (view, activity). Standing groups: idle, walk, carry, gather, build, talk, crossed, hop, run. Group factor `f_g = min(1, H / median(h0 of the group))` with `H` = 28 (adult) or 21 (child). Non-standing groups (sit, crouch, eat, drink, asleep, hug) use the attempt's median standing factor; an attempt without standing frames uses the median factor of the same body's other attempts. After fitting, a standing group's median height must be within `H ± 1`, no standing frame may differ from `H` by more than 2 (walk bob), and widths stay ≤ 22 (adult) or ≤ 17 (child); lying frames ≤ 30 × 16 (adult) or ≤ 24 × 12 (child).
- **Objects:** per cell `f = min(1, W/w0, Hmax/h0)` with the maxima below; U1 icons are fitted into 16 × 16 and then centred on an exact 16 × 16 canvas.

| sheet / items | max (w × h texels) |
|---|---|
| S1 hearths (unlit, lit ×4, embers ×2, cold ring) | 24 × 26 |
| S1 storage pits ×3, woodpile, sitting log | 24 × 22 |
| S1 shelters A, B, C; S2 construction stages ×4, drying rack, hide frame | 34 × 30 |
| S2 other camp items (basket must stay smaller than every bush stage) | 24 × 22 |
| N1 broadleaf large ×3 | 40 × 48 |
| N1 broadleaf small ×2, conifers ×2, dead tree, pale-bark tree | 30 × 40 |
| N1 sapling | 16 × 24 |
| N1 stumps ×2, shrubs ×3, fallen log | 24 × 18 |
| N2 berry bush stages ×4 | 26 × 22 |
| N2 boulders, rock clusters, flint, reeds, cattails | 24 × 22 |
| N2 outcrops ×2 | 32 × 30 |
| N3 decals | 12 × 12 |
| F1 effects | 16 × 16 |
| F2 ambient life | 16 × 12 |
| U1 icons | exactly 16 × 16 canvas |

## 6.3 Guards (automatic rejection)
format (palette membership 100 %, binary alpha); geometry of 6.2; mean ΔE76 > 12 (gross colour failure; > 6 is only the warning `colour_drift`); `berry_stage_order` (full > two > one ≥ 1, bare = 0) checked on the chosen four stages; lit hearth fire ≥ 12 texels after fitting; child standing height < every chosen adult standing height; basket smaller than every bush stage; feet on the last opaque row; within one walk/carry/run group head-top varies ≤ 1 texel and frames 1 and 3 differ from 2 and 4 in ≥ 4 texels of the bottom 40 % of rows. **Removed:** the class-maximum rejection, the hard ΔE ≤ 6, and `missing_accepted_view_idle`.

## 6.4 Tests (`test_normalize_v4.py`)
Factor never exceeds 1; a random 19 × 33 palette sprite upscaled by 5.9 with phase 1.7 fits to height 28 ± 1 and agrees with a nearest-neighbour downscale of the original 19 × 33 sprite to the same size in ≥ 80 % of texels (same palette colour); mode pooling picks the majority colour and the darker one on ties; a 1-texel dark outline survives fitting; U1 output is exactly 16 × 16; output alpha is binary; the Production 1 v3 tests still pass. Commit, push WIP.

---

# 7. PART 2 — Re-judge all 42 raws and review them like an art lead

## 7.1 Candidates
`tools/art_pipeline/salvage_v4.py` runs v4 on every cell of every raw (`production_sheets.SHEETS` defines each slot) and writes `docs/art/production2/candidates.json`: per slot, every candidate with attempt, factor, metrics, guards and warnings. Production 1's records under `docs/art/production/` stay untouched.

## 7.2 Choosing within a body
- **Animation groups come from one attempt.** For each body (adult_a, adult_b, child) and each (view, activity) group, choose the attempt with the most guard-passing frames in that group (ties: lower mean ΔE). Frames missing inside a chosen group are derived from that group's own frames with the Production 1 rules of `procedural_animation.py` (e.g. walk 4 = walk 2) and marked `derived`.
- **Reference idle per body:** the S idle 1 of the attempt that supplies the S walk group (else the best guard-passing S idle). It is the identity reference for review and for fill generation.
- **Objects:** best guard-passing candidate per slot (fewest warnings, then lowest ΔE, then earliest attempt).

## 7.3 Independent visual review (the acceptance step)
- Build one review sheet per production sheet in `docs/art/production2/review/<sheet>.png`: per slot the raw cell, the fitted sprite at 6×, the fitted sprite at 2× on the muted grass colour, and for characters the body's reference idle at 6×; for villagers also the region map (skin, hair, clothing, outline, other) from `regions.classify`.
- Hand the sheets and only this rubric to a **separate reviewer session with fresh context** (as Production 1 did for its visual reviews): per slot `ok` or `reject` with one reason from: `wrong_pose`, `wrong_view`, `broken_anatomy`, `different_person` (hair, beard or build differs from the reference idle), `wrong_object`, `artefact` (magenta remnant, text, stray texels), `detail_destroyed_by_fit`, `region_map_wrong` (e.g. face classified as clothing). Style preferences are not reasons. The reviewer writes `docs/art/production2/review/verdicts.json` before seeing any acceptance statistics.
- A rejected slot gets its next candidate (another attempt) and one more review round; then it is left for Part 3.
- Apply the escape hatch of 4.1 here if its condition holds, re-run 7.1–7.3 once at the other scale, and record both results.

## 7.4 Output
`docs/art/production2/salvage.md`: per sheet the number of slots accepted from raws, derived, still empty, with reasons; contact sheets of every accepted slot. Commit, push WIP.

---

# 8. PART 3 — Fill the gaps, then fall back honestly

## 8.1 Fill generation (≤ 18 calls)
List the empty slots. Priority: villager activities of adult_a, adult_b, child (walk, carry, gather, build, eat, drink, sit, talk, asleep, every view) → structures and construction stages → trees → everything else. Group them into 4 × 4 fill sheets of one class; villager fill sheets contain one body only.
- References: `docs/art/D3/corrected_sheet_v2.png` (style) and, for villagers, the body's reference idle upscaled 8× nearest (identity).
- Prompt: the Production 1 frame text (magenta `#FF00FF` background, equal cells, one item centred per cell, wide margins, no text, no background shadows, same style and lighting as the reference) plus per-cell descriptions from `prompt_for`. **Remove every texel-count and palette-hex instruction** (the CHECKPOINT20 contract); v4 handles scale and palette. For villagers add: "the same person as the reference image, same hair, face and clothing".
- Every fill raw goes through 7.1–7.3. **Checkpoint after 6 calls:** if fewer than half of the attempted fill cells are accepted, stop generating and record why.

## 8.2 Fallbacks v2 (only for slots still empty)
- **Villagers:** (1) the same body and activity mirrored E ↔ W; (2) procedural frames from the same body's accepted frames of the same view (`procedural_animation.py` rules); (3) the same body's frame of that activity in the S view; (4) the same body's idle in that view plus the activity's U1 icon above the head (drawn at zoom ≥ 2 only). Never use `direction_09_villager_gatherer` or any basket-holding frame for anything but carry, and never borrow another body's frames.
- **Objects:** simulation-visible things (hearths, storage, shelters, construction stages, berry bushes) keep their current installed art when nothing better exists. Purely decorative items (decals, ambient life, extra props, effects) without accepted art are **not drawn at all**; the shared brown blob is removed from every slot.

## 8.3 Install
Write the manifest entries (`source` = `generated:<sheet>/<cell>@<raw-attempt>:v4:f=<factor>` or `derived:<rule>` or `procedural:<rule>`; `qa_status` = `accepted` only for review-approved cells), repack with `pack_atlas.py`, rebuild region atlases with `regions.py`, keep the D3 direction sprites as reference entries. Update `assets/art/manifest.json` audits (no unregistered or missing PNG, binary alpha, recorded sizes). Commit, push WIP.

---

# 9. PART 4 — Villagers on screen

1. `agent_renderer.gd` uses the new frames through the existing lookup (`_animation_frame`, about line 329). Remove any path that can show a basket outside `carry`.
2. Activity icons of 8.2 (4) are drawn by the renderer only when the frame lookup fell through to that fallback.
3. **Elders (forward-compatible):** when `life_stages[i] == 2`, use the adult body and set the hair variant to grey (3) or white (4) of the five hair ramps (black, dark brown, auburn, grey, white), chosen by `ArtHash.mix(agent_id) % 2`; skin and clothing unchanged. While the snapshot reports any elder, non-elder adults draw their hair variant from 0–2 only, so grey and white hair mean "elder". Without value 2 nothing changes.
4. Tests: update `test_agent_activity.gd` (basket only in carry, icon fallback), add `test_elder_variant.gd`.
Commit, push WIP.

---

# 10. PART 5 — Nature: groves, clearings, see-through canopies

## 10.1 Density (`nature_renderer.gd`)
Replace the per-tile rule at line 114. Grove value `g(x, y)` = bilinear value noise on a 6-tile lattice from `ArtHash.mix` (deterministic). Forest tiles: at most one tree, chance `0.05 + 0.55 × smoothstep(0.35, 0.75, g)`, tune the two constants so that **25–35 % of forest tiles carry a tree** on the capture seed; species by `_tree_id` with the salvaged N1 set (large broadleaf, small broadleaf, conifer at altitude > 150, rare dead and pale-bark trees). Forest tiles without a tree: 8 % understory (shrub, stump, fallen log, sapling). Grass tiles: 1.5 % lone tree, 3 % shrub.

## 10.2 Clearings
No tree, shrub or log within Chebyshev 4 of any building, construction site, hearth, settlement formation tile or berry bush.

## 10.3 See-through canopies
Every frame, a tree whose drawn rect covers a villager's head point (feet − 20 px) while the tree's anchor y is greater than the villager's feet y gets `modulate.a = 0.45`; otherwise 1.0. Use a tile-bucketed lookup so the cost stays far below 1 ms with 200 villagers.

## 10.4 Season tint (forward-compatible)
When `get_time_snapshot()` has `season` and `days_per_season > 0`, the ground and the nature sprites (never villagers, structures or UI) get a tint: spring `(0.97, 1.0, 0.95)`, summer `(1, 1, 1)`, autumn `(1.0, 0.93, 0.80)`, winter `(0.86, 0.90, 1.0)`, cross-faded over the first 6 in-game hours of a season. Terrain layers take it as `modulate`; nature sprites share the y-sorted layer with villagers, so set it per sprite as `modulate.rgb` and keep `modulate.a` for 10.3. Without the keys: `(1, 1, 1)`.

## 10.5 Tests
`test_nature_density.gd` (coverage 25–35 %, zero objects inside the clearings, identical placement after a reload), `test_canopy_fade.gd`, `test_season_tint.gd` (with and without keys); keep `test_nature_clearance.gd` and `test_ysort_props.gd` green. Commit, push WIP.

---

# 11. PART 6 — HUD and news

1. `hud_topbar.gd:143`: the death count adds `tallies.get("deaths_old_age", 0)`.
2. Death news: `hud_status_panel.gd` reads `get_recent_deaths()` (deduplicate by tick, x, y, reason) and pushes one entry per death with its cause: reason 0 starvation, 1 dehydration, 2 combat, 3 old age. `death_viz_renderer.gd` gains reason 3 = grey `(0.70, 0.70, 0.70)`.
3. Season label (forward-compatible): with the season keys the top bar shows the season and the day of the season next to the clock; without them nothing changes.
4. Localization keys (add to `localization/fluent/en/messages.ftl` and `ko/messages.ftl`, recompile with the same compiler and registry step Production 1 used in `198e7176`; check for duplicates first):

| key | en | ko |
|---|---|---|
| `HUD_FEED_DEATH_STARVATION` | A villager starved | 주민이 굶주려 죽었다 |
| `HUD_FEED_DEATH_DEHYDRATION` | A villager died of thirst | 주민이 목말라 죽었다 |
| `HUD_FEED_DEATH_COMBAT` | A villager died in a fight | 주민이 싸움으로 죽었다 |
| `HUD_FEED_DEATH_OLD_AGE` | A villager died of old age | 주민이 노환으로 세상을 떠났다 |
| `HUD_SEASON_SPRING` | Spring | 봄 |
| `HUD_SEASON_SUMMER` | Summer | 여름 |
| `HUD_SEASON_AUTUMN` | Autumn | 가을 |
| `HUD_SEASON_WINTER` | Winter | 겨울 |
| `HUD_SEASON_DAY_FMT` | %s, day %d | %s %d일째 |

5. Tests: `test_hud_old_age.gd` (count with and without the key, news entry per reason, no duplicate entries), season label with and without keys. Commit, push WIP.

---

# 12. PART 7 — Look at it, measure it, compare it

1. **Captures:** `bash tools/art_pipeline/with_window_lock.sh 1800 bash tools/art_pipeline/capture_art.sh art docs/art/production2/shots`, plus the calibration plan into `docs/art/production2/calibration`.
2. **QA** with `ingame_qa.py` (thresholds unchanged): A1–A12, where
   - A5: median ≥ 0.15, minimum ≥ 0.10, **no unmeasurable villager**. Record each sprite's `modulate.a` in the capture metadata; a canopy faded by 10.3 (alpha ≤ 0.5) does not make a villager unmeasurable and its blended pixels count as what the player sees;
   - A11: blind identification on 24 crops from a calibration line-up covering walk, carry, gather, build, eat, drink, sit, talk and child (different bodies and ramps, shuffled; asleep is hidden in shelters and excluded), graded like Production 1 (labels saved and hashed before the key is opened): 4× ≥ 0.8 and 2× ≥ 0.6. Extend `calibration_scene.gd` if needed so the line-up uses the renderer's own path (`ArtSpriteLibrary.apply`, `apply_variant`, the renderer's frame rate).
   New checks:
   - **A13 distinct activities** (atlas level): for adult_a, adult_b and child, each of walk, carry, gather, build, eat, drink, sit, talk has at least one frame whose opaque-texel symmetric difference with the body's S idle is ≥ 15 %; no idle, walk, build, talk, sit, drink or asleep frame has ≥ 6 berry-ramp texels (the basket rule);
   - **A14 forest:** on the capture seed, 25–35 % of forest tiles carry a tree and zero objects lie inside the clearings of 10.2.
3. **Before/after** against Production 1 at the same framings (`docs/art/production/shots/a_d2_1200_F2.png`, `a_d2_2300_F2N.png`, `a_d2_1200_F1.png`): `docs/art/production2/before_after.png`; noon and evening GIFs (40 frames, ≤ 8 MB each).
4. **Compatibility with the simulation's latest green:** in a temporary local branch merge `origin/codex/revitalize-2026-10` as it is at that moment (Pass 4 may have advanced it), build the bridge, run the art gate's headless checks and the noon and night captures, record conflicts, `life_stages`/`season`/`deaths_old_age` presence and screenshots in `docs/art/production2/compatibility/`, delete the branch, never push it.
5. Commit, push WIP.

---

# 13. PART 8 — Final gate, report, PR

1. Full gate on the final WIP head: `GATE_OUTPUT_DIR=docs/art/production2/gates/final bash tools/art_pipeline/gate.sh final` (cargo test, clippy all targets, strict GDScript, bridge build, headless smoke and error scan, Python tests, every Godot art scene test). Fast-forward `codex/art-production-2026-10` if green, else say exactly what is red.
2. Budgets: `assets/art/**` ≤ 25 MB, `docs/art/production2/**` ≤ 40 MB (JPEG 85 for screenshots and sheets).
3. `docs/art/production2/REPORT.md` in Korean in the section 16 format, answering every item of section 15 with its evidence (test, capture, QA number, or not verified).
4. Update the PR #511 description with a short Art Pass 2 section (`gh pr edit 511 --body-file …`; skip and say so if `gh` is not authenticated). Keep it a draft.

---

# 14. OUT OF SCOPE

Simulation code and FFI; new sprite sheets beyond the 18 calls; new UI panels; music and sound; weather effects; merging into `lead/main`; Pass 4's paths.

---

# 15. 인게임 확인사항 (게임을 직접 켰을 때 확인할 것)

1. 주민이 걷는 방향에 따라 정면, 뒷모습, 옆모습이 다르게 보이고 걸음 동작이 보인다.
2. 바구니는 식량을 들고 이동할 때만 보인다. 걷기, 채집, 건설, 대화 중에는 바구니가 없다.
3. 채집, 건설, 먹기, 마시기, 앉기, 대화가 서로 다른 자세로 보이고, 2배 줌에서도 대략 구분된다.
4. 숲은 무리 지은 숲과 빈터로 나뉘고, 화덕·쉼터·덤불 주변은 비어 있다. 주민이 나무 뒤로 가면 나무가 반투명해져 주민이 보인다.
5. 동그라미에 막대를 꽂은 나무와 갈색 덩어리 소품이 더 이상 없다. 활엽수, 침엽수, 고사목, 그루터기, 관목, 쓰러진 통나무가 섞여 보인다.
6. 아이는 어른보다 확실히 작고, 두 성인 체형이 서로 다르게 보인다.
7. 소식창에 주민의 죽음이 원인과 함께 나온다.
8. 시뮬레이션 Pass 4가 병합된 뒤에는 노인이 흰머리나 회색 머리로 보이고, 사망 수에 노환이 포함되며, 상단에 계절과 날짜가 나오고 땅과 나무 색이 계절에 따라 바뀐다. 병합 전에는 이 항목이 화면에 영향을 주지 않는다.
9. 콘솔에 SCRIPT ERROR가 없고 프레임 시간이 Production 1과 비슷하다.

---

# 16. 구현 완료 보고 (REPORT.md와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
v4 맞춤 축소, 몸통별 배율, 검토 기준, 대체 규칙, 숲 배치와 반투명, 계절·노인·노환 기능 감지 방식

### 되살린 결과
시트별 표: 원본에서 되살림 / 파생 / 새 생성으로 채움 / 대체 / 그리지 않음
검토 거절 사유 분포, 사용한 생성 횟수(최대 18), 축척 결정(28 유지 또는 탈출구 사용)

### 실제 게임 화면 검증
A1~A14 표(수치), 블라인드 식별 혼동 행렬, before/after 경로, GIF 경로

### 시뮬레이션 작업과의 호환
Pass 3 병합 결과, 최신 green과의 임시 병합 결과(충돌, 노인·계절·노환 키 여부, 캡처)

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

Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, whether PR #511's description was updated, image calls used, and the paths of `before_after.png`, the two GIFs and the review sheets.