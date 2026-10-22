# GOAL: WorldSim — ART-TRIAL3: the character pipeline decided by structure — one template, one layer contract, automated structural checks, and two production routes (A: 2D layers drawn on the template with the image model's edit mode; B: 3D rigged model pre-rendered through an ink shader) each taken end to end for one body, judged on moving GIFs

Repository `hyunlord/new-world`. Base: **`v8/art-trial2` = `2433a6128e89fde30a4dd9e6995fbfa75049ba30`** (TRIAL2b: environment accepted under judge v2; people failed). New branch **`v8/art-trial3`**.

Executed by **Codex (GPT Astra)** in `/Users/rexxa/work/new-world-art-prod` with: the image generation tool (reference/edit mode), the **Higgsfield CLI** (`higgsfield auth login` done by the owner beforehand; if not logged in, route B uses Meshy if a key exists, else B stops and says so), **Blender 4.x** (if absent, install the official build with `brew install --cask blender` and record the version; if that fails, B stops), and Godot 4.6 for `art/proto/`. Only `art/**` and `docs/v8/arttrial3/**`. No game code, no merge. Target 10 hours, hard stop 14.

## Why

Two attempts at people failed for the same reason: **parts were assembled at runtime from images that were never designed to fit together** (32-px layer subtraction in ART-TRIAL; generated parts on a skeleton in TRIAL2/2b: hollow torsos, floating arms, an axe through the head). Environment is solved (35/38 accepted, 2.0 tries each). The owner's instruction on 2026-10-22: decide **whole vs parts** properly, learn from how other games do it, and build the structure that makes such failures impossible before generating anything else.

The structure below is the one every shipped 2D colony game uses in some form: **a fixed character template, layers authored on that template (never extracted by subtraction), a fixed upper body across frames so overlays are drawn once per direction, and automated structural checks before any taste judgement.** The two routes differ only in *who draws on the template*: the image model's edit mode (A) or a 3D model rendered through an ink shader (B). Both must pass the same checks and the same moving-GIF readability test. The owner picks the route from the evidence.

---

# 0. RULES
- Never ask the owner; decisions in `docs/v8/arttrial3/TRIAL3_DECISIONS.md`. Branch `v8/art-trial3`; commits `v8-arttrial3(pN): …`; push after every Part; `git ls-remote` check. Never other branches, never force-push, explicit paths.
- Files: originals and renders to `/Users/rexxa/work/arttrial3-data/`; the repository gets the template spec, the checker, the accepted layers at working resolution (≤ 1.5 MB each), GIFs (≤ 2 MB each), boards, and docs; `art/` growth ≤ 40 MB.
- Processing: no quantization, no outline forcing. Allowed: chroma-key, Lanczos downscale, **hole filling inside a declared layer box** (§2.4), alpha clean-up, composition. Every operation logged in the layer's sidecar.
- **No taste judgement before the structural checks pass.** A layer that fails §2.5 is not shown on any board except the failure board.
- Try limits: A: 3 per layer; B: 3 per render setup (not per frame). A route whose structural pass rate is below 50 % after its tries is stopped and reported.
- Parts: 0 = §2 (template, contract, checker) · 1 = §3 (route A) · 2 = §4 (route B) · 3 = §5 (lab, GIFs, tests) · 4 = §6 (numbers, verdict) · 5 = §7 (report). Past 14 h: finish the step, go to Part 5. Drop order: B's north direction → A's north direction → the second item per route → B entirely (if Blender or the 3D tool is unavailable by hour 3, report and continue with A only). Never drop: §2 in full, one route end to end, the GIF test, the numbers.

# 1. REFERENCES
`art/style05/STYLE.md` (v2, front 3/4 character rule), `anchors/person_v2.png` and the hut/props anchors, `art/style05/boards/people_v2_2x.png` (what failed and how), `docs/v8/arttrial2/TRIAL2B_REPORT.md`, `art/proto/` (Godot project; reuse the camp scene and calm ground), `docs/VISION.md` §6.

---

# 2. PART 0 — Template, layer contract, structural checker (`art/character/`)

## 2.1 The template (`art/character/TEMPLATE.md` + `template_guides/*.png`)
Game units: tile = 64 px at 1×. **Sprite canvas 64 × 96 px** (one tile wide, 1.5 tiles tall; the head may overhang the tile above). Authoring resolution is **4× (256 × 384)**; every coordinate below is in game px and is multiplied by 4 for authoring.

| element | adult (56 px tall) | child (42 px) | elder (adult, stooped) |
|---|---|---|---|
| foot line (bottom of feet) | y = 88 | y = 88 | y = 88 |
| contact shadow | ellipse centre (32, 90), 28×8, alpha 110 | 22×6 | as adult |
| head box | x 20–44, y 32–50 | x 23–41, y 46–60 | y 36–54 (tilted forward) |
| torso box | x 18–46, y 50–70 | x 21–43, y 60–74 | x 18–46, y 54–72 |
| hip line | y = 70 | y = 74 | y = 72 |
| legs | y 70–88 | y 74–88 | y 72–88 |
| hand anchors, south | L (14, 66), R (50, 66) | L (17, 70), R (47, 70) | L (14, 68), R (50, 68) |
| hand anchors, east | front (44, 64), back (26, 62) | (41, 68), (27, 66) | (44, 66), (26, 64) |
| hand anchors, north | L (50, 66), R (14, 66) | mirrored | mirrored |
| shoulder/carry anchor | (32, 52) | (32, 60) | (32, 55) |
| sleep canvas | 96 × 64, body axis horizontal, head left, centre (48, 36) | | |

Directions: **S (front 3/4, camera ~20° down), E (side 3/4), N (back)**; W = E flipped. Frames: `idle`, `walk_a`, `walk_b`, `work_a`, `work_b`, `sleep`. **Fixed upper body rule:** in `idle`, `walk_*`, `work_*` of one direction, the pixels inside the head box and torso box are identical (ΔE ≤ 3, alpha equal); only legs, arms and the shoulder line's 2-px margin may change. Arms belong to the body layer; sleeves are not separate art.

`template_guides/<body>_<dir>.png`: the boxes, lines and anchors drawn as thin cyan guides on a transparent 256×384 canvas. These are **inputs to generation** (A attaches them; B overlays them for alignment) and inputs to the checker.

## 2.2 Layers and draw order
`shadow → body[frame] → top[dir] → hair[dir] → beard[dir] → headgear[dir] → item[frame, hand anchor] → carry[dir, shoulder anchor] → (optional) outline pass`. Every layer is a full 64×96 (4×: 256×384) RGBA PNG on the template, plus a **mask PNG** of the same size with region ids: 0 none, 1 skin, 2 hair, 3 cloth primary, 4 cloth secondary, 5 item. Tinting at runtime by region (household → cloth primary; agent hash → hair; skin tone → skin).

## 2.3 Minimum set per route (one body: adult male)
- body: S/E/N × idle, walk_a, walk_b, work_a, work_b + sleep (16 renders; N droppable → 11);
- top: `hide_wrap`, `fur_cloak` × S/E/N (6);
- hair: `short`, `long_tied` × S/E/N (6);
- items: `stone_axe` (grip pixel declared), `log_bundle` (carry) (2);
- child and elder are **derived** (scale/stoop as in TEMPLATE; no generation) and checked.

## 2.4 Layer contract (`art/character/LAYER_CONTRACT.md`)
1. Canvas exact; origin and anchors from the template; nothing outside the canvas.
2. **No subtraction.** An overlay is authored as the garment/hair itself; where it covers the body it is opaque, interior included. Hole filling inside the layer's declared box is the only allowed repair, and it is logged.
3. Overlays never redraw the body: a top's alpha outside the torso box ± 4 px must be 0 except declared extras (cloak hem ≤ 12 px below the hip line, collar ≤ 4 px into the head box).
4. Items declare a grip pixel; carry sprites declare a shoulder pixel; the checker places them at the anchors.
5. Masks: every opaque pixel has a region id; skin only where the body shows; cloth ids only on tops.
6. Fixed upper body (§2.1) across frames.
7. Style is judged **after** structure, by judge v2 (person family), scores reported, not gating in this pass.

## 2.5 Structural checker (`art/pipeline3/check_layer.py`, `check_set.py`; unit tests on synthetic layers: a good layer, a hollow torso, a floating arm, a shifted foot line, a mask gap)
Per layer: canvas; alignment (feet bottom within ±2 px of the foot line; head alpha bbox within the head box ± 3; hands within ± 4 of anchors on body layers, found as the mask-5-free skin extremities or declared); **connectivity** (body silhouette one 8-connected component; items one component); **no holes** (any transparent region fully enclosed by the silhouette larger than 12 px² at 1× fails; the gap between legs is exempt when it opens to the foot line); **coverage** (a top covers ≥ 95 % of the torso box); **mask completeness**; **fixed upper body** across frames.
Per set: render every combination (body frames × tops × hair × items × 3 dirs) and check: union silhouette hole-free; no body pixel visible through a top in the torso box; item grip at the anchor; nothing clipped. Output `CHECKS.md` + JSON per route, and a **failure board** of every rejected layer with the failing rule drawn on it.

---

# 3. PART 1 — Route A: 2D layers drawn on the template (image model, edit mode)
1. **Base body, S idle:** prompt = STYLE.md character block + "fit the figure to these guide boxes exactly; feet on the lower line; neutral grey skin `#9A9A9A`; plain loincloth; no hair; magenta background", with the template guide and the person anchor attached. ≤ 3 tries until the checker passes.
2. **E and N idle:** edit mode on the S idle ("the same figure turned to its right side / seen from behind, same proportions, same guides").
3. **Frames:** edit mode per direction: "same figure, only the legs stride / only the arms swing an axe; head and torso unchanged". The fixed-upper-body check decides; if the model moves the torso, copy the idle head+torso pixels over the frame (allowed: it is the rule) and re-check.
4. **Overlays:** for each top/hair: edit mode "the same figure wearing a fur cloak; output **only the cloak**, opaque, exactly where it sits on this figure, magenta elsewhere". If the model cannot output overlay-only, generate the dressed figure and build the overlay by **mask-box extraction with hole fill** (keep changed pixels within torso box ± margins, fill interior) — logged, then checked by the contract (no holes, coverage).
5. Items: axe and log bundle on magenta with declared grip/shoulder pixels.
6. Masks: generated by the pipeline from flat-colour regions (skin grey, cloth primary as flat `#C0C0C0` requested in prompts), verified by the checker; hand-fix of masks by code is allowed and logged.
7. Record tries, time and checker pass/fail per layer.

# 4. PART 2 — Route B: 3D rigged model pre-rendered through an ink shader
1. **Model:** Higgsfield CLI game-generation skill (or Meshy): a stylized stone-age adult male, proportions of §2.1 (head ≈ 30 % of height), neutral grey skin material, loincloth; export rigged GLB. Two outfit variants (hide wrap, fur cloak) as separate meshes or material sets; two hair meshes; an axe and a log bundle as props. ≤ 3 generation setups.
2. **Animation:** preset clips idle, walk, axe swing, carry walk, lie/sleep from the tool's library; retarget in Blender if needed.
3. **Blender headless render** (`art/pipeline3/render_b.py`): orthographic camera at the template's angle (S: 20° down from the front; E and N by rotating the model), framing so the figure's feet land on the foot line and height = 56 px at 1× (render at 4×); **ink look**: Freestyle lines (2 px at 4×, dark `#141210`), 3-band toon shading, flat wash colours, paper-grain post in compositor. Render **separate passes** by object id: body, top, hair, item → the layer files of §2.2, with masks from material ids. Frames sampled from the clips to `idle`, `walk_a/b`, `work_a/b`, `sleep`.
4. Run the same checker. Fix camera/framing, not the images.
5. Record tool credits used, minutes per render setup, and the number of manual Blender fixes.

# 5. PART 3 — Character lab, GIFs, tests
1. **Character lab** (`art/proto/scenes/character_lab.tscn`): a grid of every combination for each route (body × tops × hair × items × dirs), the six frames looping, on the calm meadow and forest floor of TRIAL2b, at 1× and 2×; a toggle between routes; a capture script that writes `boards/lab_<route>_<zoom>.png` and **GIFs** `gifs/<route>_<combo>_<zoom>.gif` (8 fps loops of walk, work, carry, sleep; `idle` as a 2-frame bob).
2. **Readability test, on GIFs** (fresh reviewers, images/GIFs only, no route names): 12 looping combos per route at 2× and at 1×; questions per GIF: what is the person doing (idle / walking / chopping / carrying / sleeping), what is carried, which top, which hair. **Bars:** activity ≥ 10/12 at 2× and ≥ 8/12 at 1×; attributes (carry, top, hair) ≥ 30/36. Two reviewers per route; one revision round allowed per route, then rerun with new reviewers.
3. **Style scores:** judge v2 (person family) on 6 stills per route, reported.
4. **Crowd:** 200 of each route walking in the camp scene at 1×, FPS over 60 s, one capture each.
5. **Owner boards:** `boards/compare_AB_2x.png` (same 12 combos side by side), `boards/compare_AB_1x.png`, `boards/failures_<route>.png`.

# 6. PART 4 — Numbers and verdict (pre-registered)
| # | measure | bar |
|---|---|---|
| S1 | structural pass rate of accepted layers | 100 % (anything else is not "accepted") |
| S2 | tries (A) / render setups (B) per accepted layer | A ≤ 2.0; B ≤ 1.5 setups per set |
| R1 | GIF readability (activity, attributes) | bars of §5.2 |
| T1 | style score vs 05 anchors (mean of four 1–5) | reported; ≥ 4.0 preferred |
| P1 | 200 walking at 1× | ≥ 60 FPS |
| K1 | cost: minutes and credits per accepted layer; projection to 4 bodies × (3 tops, 4 hair, 6 items, 2 headgear) × 3 dirs × 6 frames | reported as a range |

**Verdict rule:** a route is *viable* when S1 = 100 %, R1 met, P1 met. If both viable → recommend the one with the higher T1 unless its K1 is more than 2× the other's. If only one viable → that one. If neither → recommend commissioning the base figures from a 2D artist against this template and contract, with generation kept for environment. Write the verdict with the numbers beside it; the owner decides.

# 7. PART 5 — Report
`docs/v8/arttrial3/TRIAL3_REPORT.md` (Korean, §8 format), `TEMPLATE.md`, `LAYER_CONTRACT.md`, `CHECKS_A.md`, `CHECKS_B.md`, `COST.md`, boards and GIFs. `art/character/PRODUCTION.md` (English): how to add a body, an overlay or an item under the contract, for either route. Push; `git ls-remote origin v8/art-trial3`.

# 8. 구현 완료 보고 (한국어)
```
## 구현 완료 보고 — ART-TRIAL3
### 구현 의도
### 템플릿·레이어 계약·구조 검사기 (좌표 요약, 검사 규칙, 단위 테스트 5종 결과)
### 경로 A (레이어별 시도·통과, 편집 모드가 못 한 것, 마스크 처리)
### 경로 B (모델·리깅·클립 출처, 렌더 설정, 패스 분리, Blender 수정 수, 크레딧)
### 캐릭터 랩과 GIF (보드·GIF 경로)
### 검사 결과 (S1·S2·R1·T1·P1·K1 표, 리뷰어 원문 요약, 실패 보드)
### 판정 (규칙 적용 결과와 권고; 선택은 오너)
### 변경된 파일 목록 (커밋별)
### 제한사항 (드롭한 것, 도구가 없어서 못 한 것, 다음 패스가 풀 것)
git ls-remote 결과
```
