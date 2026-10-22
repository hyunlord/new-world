# GOAL: WorldSim — ART-TRIAL2b: fix the two planner mistakes of TRIAL2 (people drawn from straight above; a style judge that compared objects against whole scenes), re-score the existing 38 environment assets with a judge that works, make people the RimWorld way (front 3/4, stylized, readable at 56 px) in the 05 ink look, calm the ground, and give the real sustainability numbers

Repository `hyunlord/new-world`. Base: **`v8/art-trial2` = `034c38abc57924c4d5de54ec4194f160d257f76c`**. Continue on the same branch (new commits `v8-arttrial2b(pN): …`).

Executed by **Codex (GPT Astra)** in `/Users/rexxa/work/new-world-art-prod` with the image tool (reference/edit mode) and Godot 4.6 for `art/proto/`. Target 5 hours, hard stop 7. Only `art/**` and `docs/v8/arttrial2/**`. No game code, no merge.

## Why

TRIAL2's verdict "FAIL → map 02, portraits 05" is **not acted on**, because its two decisive measurements were broken by the brief, not by the style:
1. **The numeric judge** compared every single object's colour histogram against six anchors that are mostly whole scenes. A lone stone axe on magenta cannot match a camp scene's palette; 37 of 38 first-try assets "failed numerically" while the model judge passed 33 of 38, and the planner's own look at `art/style05/boards/environment_first_attempt.png` finds the trees, hut, bench, hearth, piles and tools consistent and usable. The report's last paragraph names exactly this flaw. C1/C2 were therefore measured with a wrong instrument.
2. **People seen from straight above** (COMMON_BRIEF's camera rule for characters) read as insects at 56 px (`people_meadow_2x.png`, the 200-crowd capture). Every top-down colony game draws the map from above and the people from the front; the owner saw the crowd on 2026-10-22 and called it unreadable. That rule is withdrawn.

Two real findings stand and are carried: the palisade autotile's joints and the mirror-blended floors show seams (`AUTOTILE_REVIEW.md`), and the generated meadow is too busy to put people on. The rig itself works (6 states, 200 people at 81 FPS).

---

# 0. RULES
As TRIAL2 §0 (no owner questions → `TRIAL2_DECISIONS.md` continued; originals to `/Users/rexxa/work/arttrial2-data/`; ≤ 1.5 MB per committed file; no quantization/outline forcing; try limits: 3 per asset). Drop order past 7 h: §5 autotile fix → §4.4 crowd. Never drop: the judge fix and re-score (§2), people (§3) with the readability test, the ground fix (§4.1), the numbers (§6).

# 1. PART 0 — Withdraw the two rules in writing
- `art/style05/STYLE.md`: replace the character camera rule with **§3.1**; add the ground rule **§4.1**; record both as D-numbered decisions with the TRIAL2 evidence paths.
- `art/concepts/COMMON_BRIEF.md`: the same character rule change, marked "amended 2026-10-22 (TRIAL2b)".

# 2. PART 1 — A judge that measures style, not content (`art/pipeline2/judge.py` v2)
- **Per-family anchors:** the numeric checks compare an asset only with anchors of its own kind: a ground tile with the ground anchor quadrant; a tree with the accepted tree; a building with the hut; an item with the props crop; a person with the person anchor. Six anchors become ~10 crops (`anchors/manifest.json` v2), the original six files unchanged.
- **Content-insensitive measures:** drop the global Lab histogram; keep line density (edge ratio) and local saturation ranges, add **stroke-width distribution** (distance-transform of the edge map) and **wash flatness** (variance of luminance inside edge-bounded regions). Calibrate leave-one-out on the anchor crops (all pass) and on the three wrong-style images (all fail); record in `JUDGE_CALIBRATION.md` v2.
- **Model judge** unchanged (two anchors of the same family + candidate, four 1–5 scores, pass = all ≥ 4).
- **Re-score the 38 first-attempt environment assets with v2, no new generation.** Report first-try pass by family under v2. This is the C2 number TRIAL2 should have produced. Assets that pass v2 are promoted to `art/style05/env/` as accepted (sidecars updated: "accepted under judge v2, TRIAL2b").
- Generate only the assets that fail v2 or the model judge, ≤ 3 tries each; report C1 (tries per accepted) honestly, counting the first attempt as try 1.

# 3. PART 2 — People the RimWorld way, in the 05 look
## 3.1 The character rule (replaces the top-down rule)
- **View:** front 3/4 from slightly above (about 20° down), the map stays top-down. The feet touch the tile's lower third; a soft contact shadow under them.
- **Proportions:** stylized for 56 px: head ≈ 30 % of height, shoulders wide, limbs simplified, hands as mitts, feet as wedges. No realistic anatomy.
- **Silhouette:** every figure must read as a person at 48 px with a 2-px dark ink contour; tunic/hide as one clear shape; hair as one shape.
- **Facing:** south (front) and east (side 3/4); west = flipped; north (back) optional.
- **Items** are large relative to the body (an axe head the size of the head); the carried log bundle sits across the shoulders and is wider than the body.
## 3.2 Person anchor
Generate the new person anchor (gatherer or woodcutter) under 3.1 with the 05 ink rules and the hut anchor attached for medium; ≤ 4 tries; freeze it as `anchors/person_v2.png` (the old person anchor stays in the manifest for history, marked withdrawn).
## 3.3 Parts and rig
- Generate a **front 3/4 parts sheet** for one adult body (head, torso, upper/lower arms, upper/lower legs, feet; mitts included in lower arms) on magenta, ≤ 3 tries; overlays: hair ×3, top ×3 (hide wrap, fur cloak, woven tunic), items ×3 (axe, log bundle, basket), hat ×1; child/elder by scale and posture as before.
- Rebuild the rig in `art/proto/` for the front view (the existing Skeleton2D structure and the six animations are re-targeted; IK for the axe kept). `PersonComposer.gd` unchanged in interface.
- Render at game scale (64-px tiles, 56-px person) on the calmed ground of §4.1.
## 3.4 Readability test (same protocol and bars as TRIAL2 §6.3)
12 composed people at 1× and 2× on meadow and forest floor; two fresh reviewers: activity + carrying ≥ 10/12 at 2×, ≥ 8/12 at 1×; sex/age/item ≥ 20/24. Record verbatim. If a bar is missed, one revision round on the specific parts, rerun with new reviewers; report both.

# 4. PART 3 — Ground and autotile
## 4.1 Calm ground
- Regenerate the six ground textures with the prompt additions "calm, low contrast, soft wash, few small details, no flowers, no scattered stones; detail only at large scale" and the ground anchor attached; ≤ 3 tries each; seamless by tiling-aware prompt first, mirror-blend only as a last resort (and then say so).
- Add a **contrast cap** post-step for ground only (allowed because it is a global tone adjustment, not quantization): luminance standard deviation ≤ 60 % of the person anchor's, saturation ≤ 0.35 mean. Record values per texture.
- Show the 12-person board on old vs new ground side by side (`boards/ground_before_after.png`).
## 4.2 Autotile fixes (droppable)
- Generate the straight palisade segment as a **tileable strip** (prompt "seamless along the horizontal axis") and the corner as a cut of a generated 2×2 corner piece; re-derive the 16; replace the mirror-blended floor with a tiling-prompted floor. List remaining seams in `AUTOTILE_REVIEW.md` v2 with the piece pairs. Verdict line as before.
## 4.3 Code companion contrast
Raise the ink line to 2 px and the wash to 25 % on the new calm ground; recapture; one-line verdict.
## 4.4 Crowd (droppable)
200 front-view people on calm meadow at 1×: FPS over 60 s and a fresh reviewer's count of a 640×360 crop (±10 %).

# 5. PART 4 — The numbers (pre-registered; same bars as TRIAL2 §8)
| # | measure | bar |
|---|---|---|
| C1 | tries per accepted environment asset (first attempt counts as 1; v2 judge) | ≤ 2.5 |
| C2 | v2 first-try pass rate / after retries | ≥ 60 % / ≥ 90 % |
| C3 | autotile verdict | works or ≤ 3 listed fixes |
| C5 | rig ≥ 4 animations + 3.4 bars | both |
| C6 | 200 people ≥ 60 FPS | pass |
| C7 | tries per accepted person part | ≤ 3.0 |
| G1 (new) | ground: 12-person board readability at 1× improves by ≥ 3 correct answers vs TRIAL2's | pass |

Verdict rule as TRIAL2 §8. `PROJECTION.md` v2 with real denominators.

# 6. PART 5 — Report
`docs/v8/arttrial2/TRIAL2B_REPORT.md` (Korean, TRIAL2's §10 format plus a first section "TRIAL2 판정의 정정": what was wrong with the instrument, with the v2 re-score table). Boards: `env_accepted_v2.png`, `people_v2_1x/2x.png`, `ground_before_after.png`, `autotile_v2.png`. Push; `git ls-remote origin v8/art-trial2`.
