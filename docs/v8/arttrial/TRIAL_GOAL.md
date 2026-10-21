# GOAL: WorldSim — ART-TRIAL: can generated images carry the colony's look? A 32-px view-B asset set from GPT image generation, a paper-doll person system, a pipeline that makes every image obey one concept, and an honest A/B against the code-drawn sheet

Repository `hyunlord/new-world`. Base: **`v8/main` = `3183e77dfbe6373f1c3d3e0d08a1121fb546b694`** (M1-ART). Work on a new branch **`v8/art-trial`** from that commit.

Executed by **Codex (GPT Astra)** in `/Users/rexxa/work/new-world-art-prod`, with the session's **image generation tool**. If no image generation tool is available in this session, stop at Part 0 and report that; do not substitute code-drawn or stock art.

**In parallel**, Claude Code runs M1.5 on `v8/wip` (simulation and tools). This trial touches **only** `art/**`, `docs/v8/arttrial/**` and `docs/images/arttrial_*.png`. No `rust/`, `scripts/`, `scenes/`, `assets/`, `shaders/`, `localization/`, `tools/v8/`. No merge into `v8/main` or `v8/wip`; the owner decides after the report.

## Why this pass exists

M1-ART drew the colony in code (`tools/v8/gen_tiles.py`, ADR 0015). It made the game readable, but it hit the ceiling of code-drawn pixels: a fresh viewer named 7 of 10 crops (child, sleeping person and lean-to unreadable), and the look is "clear" rather than "good". The owner (2026-10-21) wants:
1. image generation for assets, **as long as one concept is enforced across every image**;
2. **people whose look varies with their attributes and equipment** (sex, age, household, hair, clothing, tool, carried item, state), which means layered sprites, not one drawing per person;
3. a decision on whether the map keeps today's 16-px base or moves up.

This trial answers all three with evidence, not opinion. It produces a complete small asset set at a **32-px tile base** (the resolution where generated detail survives and paper-doll layers fit), a pipeline that forces palette, outline and grid, a person-variation board, and side-by-side comparisons the owner can judge. It changes no game code.

---

# 0. RULES
- Never ask the owner. Decisions in `docs/v8/arttrial/TRIAL_DECISIONS.md` (numbered: context, choice, reason, evidence).
- Branch `v8/art-trial`; commits `v8-arttrial(pN): …`; push after every Part with `git push origin v8/art-trial`; verify with `git ls-remote origin v8/art-trial`. Never push any other branch; never force-push; stage explicit paths; never touch `.claude/settings.json`, `.gitignore`, `.ignore`.
- **Repository size:** commit only the final sheet(s), the comparison images and ≤ 1.5 MB per file; raw generations, rejects and intermediates live in `/Users/rexxa/work/arttrial-data/` (paths in the report). Keep the committed total under 12 MB.
- Parts: 0 = §3 (style bible, palette, pipeline) · 1 = §4 (environment set) · 2 = §5 (people) · 3 = §6 (composites and boards) · 4 = §7 (checks) · 5 = §8 (report). Target 10 hours, hard stop 14; past 14, finish the step and go to Part 5.
- Drop order: 5.4 (portraits) → 4.3 (animals) → 5.3's fourth and fifth layer families → 6.3 (night composite). Never drop: the style bible, the pipeline with its tests, the hut / trees / log piles / lean-to / hearth, the paper-doll base with ≥ 3 layer families, the person-variation board, the A/B composite, the blind read test, the report.

# 1. MACHINE
The art-prod checkout only. No DGX, no ComfyUI. Data in `/Users/rexxa/work/arttrial-data/`. Python 3 with Pillow and numpy (install into a venv under the data dir if missing).

# 2. REFERENCES (read before generating)
- `docs/images/v8_target_view_b.png` — the chosen mock-up (view B: straight top-down, dark earthy, one person per tile).
- `docs/images/m1art_contact_sheet.png` and `docs/images/m1_*.png` — the current code-drawn look (the thing to beat).
- `docs/VISION.md` §6 — hierarchy: people and events > items and work spots > walls and floors > ground. Mood words: 촘촘함, 흙과 돌, 화덕 불빛.
- `tools/v8/gen_tiles.py` (read only) — the current sheet's names, sizes and palettes; `assets/v8/tiles.json`.
- `docs/decisions/0015-colony-art-one-code-drawn-sheet.md` — the craft rules M1-ART wrote (outline, 3–4 values, top-left light, calm ground, people brightest, backed text). The generated set must obey the same rules after the pipeline.

---

# 3. PART 0 — Style bible, palette, pipeline

## 3.1 Style bible (`art/STYLE.md`, English; a Korean summary at the top)
Write the one concept every prompt and every check refers to:
- **View:** straight top-down (camera looking down, no perspective), with a 10–15 % "front face" cheat on tall objects (hut walls, trees show a sliver of their south side) so they read as volumes. No isometric, no 3/4.
- **Grid:** 32 px per tile. Objects snap to the grid: 1×1 (32×32), 1×1 tall (32×48, 16 px overhang upward), 2×2, 3×3 (96×96). People are 32×32 (adult fills ~28 px tall), children 24 px tall inside a 32 box.
- **Line and light:** 1-px dark outline `#141210` on every object silhouette; none on ground tiles. Light from the top-left; cast shadows 1–2 px down-right in a darker ground tone. 3–4 values per material, no gradients, no anti-aliasing, no dithering except on ground edges.
- **Palette:** the 48-colour master palette of 3.2; nothing outside it after the pipeline.
- **Mood:** dark earthy ground, desaturated greens, warm wood and hide, the hearth as the only hot colour. People carry the brightest skin/tunic values on screen.
- **Materials:** hide, bark, round logs with end-grain, packed earth, river stones, thatch/branches, woven reeds. Stone age: no metal, no cloth dyes beyond ochre/charcoal/bone.
- **Prompt template** (fill-in form used for every asset): subject, size and grid rule, view, outline and values, palette words, background `#FF00FF` magenta solid, "no text, no watermark, no scene, single object centred", negative list (isometric, perspective, gradient, blur, photo, 3D render).
- **Reference discipline:** the first accepted asset of each family (one tree, one building, one person) becomes the **reference image** attached to every later prompt of that family ("match this exact style").

## 3.2 Master palette (`art/palette/worldsim_b.json` + `worldsim_b.png` swatch)
Build it from the mock-up and the current sheet: extract the 40 most used opaque colours of `assets/v8/tiles.png` and `docs/images/v8_target_view_b.png` (k-means in Lab, then snap to the nearest existing pixel colour), add the 4 skin tones, 4 hair tones and 6 household tunic hues used by `tools/v8/gen_tiles.py`/`view.gd` (read them from the files), dedupe within ΔE < 4, cap at **48**. Record the method; commit the JSON (hex list with names) and the swatch.

## 3.3 Pipeline (`art/pipeline/`, Python, unit-tested with `art/pipeline/test_pipeline.py`)
Every generated image goes through, in order:
1. `ingest.py`: magenta (and near-magenta) background → alpha; trim; report the bounding box.
2. `pixelize.py`: generate at a multiple of the target (ask the model for 1024×1024 of a 32-px-cell subject, i.e. 32× scale, or 512 for 16×); reduce with **block mode** (majority colour per block) to the target size, never bilinear.
3. `palette.py`: map every opaque pixel to the nearest master-palette colour (Lab distance); report how many pixels moved more than ΔE 8 (a quality signal).
4. `outline.py`: enforce a 1-px `#141210` outline on the silhouette (add where missing, never thicken).
5. `snap.py`: centre on the grid cell, pad to the declared box (32×32, 32×48, 96×96 …), reject if the subject exceeds the box.
6. `layer_extract.py` (people only, §5): subtract the mannequin to isolate a clothing/hair/tool layer.
7. `sheet.py`: pack accepted assets into `art/trial/sheet.png` + `sheet.json` (name → rect, box, anchor), same schema as `assets/v8/tiles.json` so a later integration pass can swap sheets.
8. `check.py`: per asset — size exact, palette compliance 100 %, outline present, alpha clean (no stray pixels outside the silhouette), and a **ΔE-to-reference** score against the family reference. Writes `art/trial/CHECKS.md`.

Tests: each step on 3 synthetic images (a magenta-background circle, a square with a gradient, a mannequin + mannequin-with-hat pair).

---

# 4. PART 1 — Environment set (32-px base)
Generate, accept and pipeline these. For each: at least 2 candidates, pick by `check.py` score and your own reading; keep the rejects in the data dir with one line why.

| name | box | notes |
|---|---|---|
| `hut` | 96×96 | log walls (two courses, end-grain on the ring), south doorway, packed-earth floor, 4 bedrolls, small hearth stone; **no roof** (interiors visible, as the mock-up) |
| `hut_ghost` | 96×96 | derived in the pipeline: silhouette of `hut` at blueprint blue, alpha 80, dashed outline — not generated |
| `lean_to` | 32×48 | two poles, hide roof, dark opening facing south; must read as a shelter at 32 px (M1-ART's failed crop) |
| `hearth` ×2 frames | 32×32 | stone ring, fire frames differ in flame shape; the only saturated orange on the sheet |
| `tree_oak` S/L | 48×48 / 64×64 | round canopy, lighter top-left, dark underside |
| `tree_birch` S/L | 48×48 / 64×64 | lighter, lumpy, bark specks |
| `tree_pine` S/L | 48×48 / 64×64 | pointed, dark, lighter crown |
| `canopy_shadow` | derived | 60 % of the canopy ellipse, alpha 90 — not generated |
| `stump` | 32×32 | fresh cut, rings visible |
| `log_pile_1/2/3` | 32×32 | 1–8 / 9–16 / 17–25 round logs, stacked, end-grain |
| `berry_bush_full/bare` | 32×32 | |
| `deadwood` | 32×32 | |
| `boulder` | 32×32 | |
| `stockpile_floor` corner/edge/interior | 32×32 ×3 | tamped earth with a stake — ground-like, no outline |
| `ground_grass` ×4, `ground_dark_grass` ×2, `ground_dirt` ×3, `ground_forest_floor` ×3, `water_deep` ×2, `water_shallow` ×2, `hill` ×2, `mountain` ×2 | 32×32 | **ground is the one place generation may lose:** generate a 256×256 seamless texture per material (prompt "seamless tileable top-down texture"), cut 32-px tiles from it, test tiling by placing a 4×4 repeat and checking seams visually and with an edge-difference metric; if seams fail twice, keep the code-drawn ground from `gen_tiles.py` for the composite and say so |
| `bank` tiles | 32×32 ×8 | 4 sides + 4 inner corners of the river edge, derived from `water_shallow` + a sand lip in the pipeline if generation cannot keep the edge geometry |

## 4.3 Animals (droppable)
`deer`, `boar`, `wolf` at 32×32, side view with a top-down tilt, for later.

---

# 5. PART 2 — People: the paper doll
Goal: **one mannequin, many layers, deterministic composition.** A person's look = body + skin tone + hair style/colour + top + headwear + tool + carried item + state. Layers are generated **on the mannequin** and extracted by subtraction, so they align pixel-for-pixel.

## 5.1 Mannequin (`body_*`)
- Bodies: `adult_m`, `adult_f`, `child` (24 px tall), `elder` (slightly stooped). 32×32 each.
- Directions: **south (front), east (side)**; north (back) optional; west is the east frame flipped.
- Frames per direction: `idle`, `walk_a`, `walk_b`, `work` (arm raised), and `sleep` (one frame, lying, 32×16, south only).
- Generate each body as a **neutral grey mannequin** with the skin area in a flat mid-grey `#9A9A9A`, hair region empty (bald), wearing nothing but a plain loincloth in `#6E6E6E`. The flat greys become `modulate` channels in the renderer: skin tone and loincloth are tinted at runtime.
- Generate frame sets as **one sheet per body** (prompt for a 5-frame strip on one image, same character, "sprite sheet, frames left to right: idle, walk A, walk B, work, sleep") so the character stays consistent; cut frames in the pipeline. If the model cannot keep frames consistent, generate `idle` first and the other frames as **edits of that image** with the reference attached.

## 5.2 Layer families (each generated as "the mannequin wearing X", same frame set, then extracted)
Minimum three families in the trial; five if time allows:
1. `hair_*`: 5 styles (short, long tied, braided, bun, shaved sides) in flat `#B0B0B0` (tinted at runtime); `beard_*` 2.
2. `top_*`: `hide_wrap`, `fur_cloak`, `woven_tunic`, `leather_vest` — colour areas in flat `#C0C0C0` (household/clan tint) with the material's texture in two darker values that are **not** tinted.
3. `tool_*`: `axe`, `hammer`, `hoe`, `pick`, `spear`, `basket` — drawn in the right hand; `work` frame shows the swing.
4. `carry_*` (droppable): `logs`, `berries`, `stone` on the shoulders/back.
5. `head_*` (droppable): `headband`, `fur_hat`, `hood`.

Extraction (`layer_extract.py`): pixels that differ from the mannequin frame (ΔE > 6) become the layer; a 1-px morphological close removes holes; the layer keeps its own outline where it meets air. Test on the synthetic pair.

## 5.3 Composition rule (`art/trial/PERSON_SPEC.md`)
Document the draw order (shadow → body → top → hair → beard → head → tool → carry → outline pass), the tint channels (`#9A9A9A` skin, `#6E6E6E` loincloth, `#C0C0C0` top, `#B0B0B0` hair), the 2 directions + flip, the frame names, and **how attributes map to layers** (sex → body; age < 15 → child body; age ≥ 50 → elder body; household id → top tint from the 6-hue set; keyed hash of agent id → hair style and colour; job → tool; carry → carry layer; asleep → sleep frame). This is the spec the integration pass implements in `agent_layer.gd`; the trial implements it in `art/pipeline/compose.py` for the boards.

## 5.4 Portraits (droppable)
Two 64×64 face portraits for the inspector, generated **from a composed sprite's attributes** (same hair style/colour, skin, top material, sex, age) so the portrait matches the figure on the map. Record the prompt.

---

# 6. PART 3 — Composites and boards (`art/pipeline/compose.py`, PIL)
1. **A/B composite** `docs/images/arttrial_ab.png`: the same scene — a camp with the hut, three lean-tos, hearth, a 4×4 stockpile with piles, 15 trees of mixed species, a river with banks, dirt clearing, 10 people with varied layers (two carrying logs, two chopping, one child, one elder) — rendered **(a)** with the trial sheet at 32-px base shown at 1× on a 1920×1080 canvas, and **(b)** the M1-ART screenshot `docs/images/m1_03_work_2x.png` cropped to the same scene size. Put the mock-up crop as (c).
2. **Person-variation board** `docs/images/arttrial_people.png`: 24 composed people (every body × 2 directions, varied hair/top/tool/carry), each labelled with its attribute string, at 3× so the owner can read them.
3. **Night composite** (droppable): the same scene at night with the hearth glow and sleep frames inside the hut.
4. **Reading sheet** `docs/images/arttrial_crops.png`: the 10 blind-test crops (7.1) with their answers, assembled **after** the test.

---

# 7. PART 4 — Checks (pre-registered)
## 7.1 Blind read test (a fresh reviewer that has not seen the prompts: a second Codex agent or the image model asked in a new context with images only)
10 crops at 2× from the A/B composite (a): hut, lean-to, hearth, stockpile with logs, log pile on bare ground, person carrying logs, person chopping, child, elder, sleeping person. **Bar: ≥ 8 / 10** (synonyms count). Record answers verbatim. One revision round allowed on failed assets, then a rerun with a new reviewer; report both.
## 7.2 Consistency
`check.py` on every accepted asset: size exact, palette 100 %, outline present, alpha clean — **bar: 100 % of assets pass**. ΔE-to-reference per family: report the distribution; flag any asset above the family's 90th percentile for the owner.
## 7.3 Variation
On the person board: all 24 composites pairwise distinct (no two identical pixel sets), and a fresh reviewer, given 6 random composites, states for each the sex / age group / whether carrying / which tool — **bar: ≥ 20 of 24 attribute answers correct**.
## 7.4 Cost ledger
`docs/v8/arttrial/COST.md`: per asset family — generations attempted, accepted, retries, wall time; total images and total time. This is what decides whether the method scales to M6.

---

# 8. PART 5 — Report
- `docs/v8/arttrial/TRIAL_REPORT.md` (Korean, §9 format) and `docs/v8/arttrial/DECISION_BRIEF.md` (Korean): the owner's one-page decision page — **16-px code-drawn vs 32-px generated**, with the A/B image, the blind-test scores of both (M1-ART's 7/10 is the code-drawn score), the cost ledger, what a 32-px base changes in the game (camera: today's 2× becomes 1×; add 0.5× for the far view; tile maps 4× the pixels; `gen_tiles.py` retired or kept for ground only), and your recommendation.
- Push `v8/art-trial`; `git ls-remote origin v8/art-trial`. **Do not** fast-forward `v8/main`.

# 9. 구현 완료 보고 (한국어)
```
## 구현 완료 보고 — ART-TRIAL
### 구현 의도
### 스타일 바이블과 팔레트 (요약, 파일 경로)
### 파이프라인 (단계, 테스트 수, 거부 기준)
### 자산 목록 (환경 / 사람 레이어 / 초상화: 만든 것, 드롭한 것, 생성 시도·채택 수)
### 사람 변형 체계 (레이어 가족, 합성 순서, 속성 → 레이어 대응)
### 검사 결과 (7.1 블라인드 10문항 답과 점수 — 코드 그림 7/10과 비교, 7.2 일관성, 7.3 변형 읽기, 7.4 비용)
### A/B 판단 자료 (이미지 경로, 32px 전환이 게임에 요구하는 변경)
### 권고 (생성 채택 / 혼합 / 코드 유지 — 근거)
### 변경된 파일 목록 (커밋별)
### 제한사항 (지형 타일 결과, 프레임 일관성, 모델이 못 한 것)
git ls-remote 결과
```
