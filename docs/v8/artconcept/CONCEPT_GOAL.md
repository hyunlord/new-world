# GOAL: WorldSim — ART-CONCEPT: eight visual concepts for the same stone-age colony, each shown on the same subjects (people, buildings, ground, a camp scene, a portrait), so the owner can pick one look and every later asset follows it

Repository `hyunlord/new-world`. Base: **`v8/main` = `3183e77dfbe6373f1c3d3e0d08a1121fb546b694`**. Work on a new branch **`v8/art-concept`** from that commit.

Executed by **Codex (GPT Astra)** in `/Users/rexxa/work/new-world-art-prod` with the session's **image generation tool** (and its image-edit / reference-image mode if available). If no image generation tool is available, stop at Part 0 and report.

This pass touches **only** `art/concepts/**` and `docs/v8/artconcept/**`. No game code, no `assets/`, no `docs/images/`, no merge. Target 6 hours, hard stop 9.

## Why this pass exists

ART-TRIAL (2026-10-21) proved two things: the image model's output was being destroyed by our own pipeline (32-px block reduction, 48-colour quantization, forced outlines), and the target we handed it (a code-drawn mock-up) was itself crude. The owner decided on 2026-10-22 to drop pixel-art quantization and move to **illustrated 2D, top-down, 64-px tiles**, generated at full resolution. Before producing a single production asset, the owner wants to **choose the look**: many distinct concepts, each rendered on the same subjects, compared side by side, one chosen, then everything generated in that direction.

This pass produces the concept book. It generates nothing for the game yet. The pick is the owner's; the next pass (ART-TRIAL2) builds a 64-px environment set and a cut-out person rig in the chosen concept.

---

# 0. RULES
- Never ask the owner. Decisions in `docs/v8/artconcept/CONCEPT_DECISIONS.md`.
- Branch `v8/art-concept`; commits `v8-artconcept(pN): …`; push after every Part (`git push origin v8/art-concept`, verify with `git ls-remote`). Never any other branch, never force-push, explicit paths only, hook and ignore files untouched.
- **Files:** originals at generation resolution go to `/Users/rexxa/work/artconcept-data/<concept>/`. The repository gets only the boards (§5), each ≤ 1.5 MB (PNG; downscale the board, never the originals), total ≤ 20 MB under `art/concepts/`.
- **No pipeline processing of the images.** No quantization, no outline pass, no block reduction. The only allowed operations are: background removal for the composite (§4.7), downscaling for the readability test (§6.1) and for the boards.
- Parts: 0 = §2 (common brief and subjects) · 1 = §3 (concepts defined) · 2 = §4 (generation) · 3 = §5 (boards) · 4 = §6 (checks) · 5 = §7 (book and report). Past 9 hours: finish the step, go to Part 5. Drop order: concepts 8 → 7 → the portrait subject → the camp-scene subject. Never drop: at least 5 concepts × the first 5 subjects, the overview board, the readability test, the book.

# 1. MACHINE
The art-prod checkout only. Python 3 + Pillow for boards. No DGX, no ComfyUI.

# 2. PART 0 — What every concept must share (`art/concepts/COMMON_BRIEF.md`)
The concepts differ in **style**; they must not differ in **content or camera**, or the comparison is unfair.
- **Camera:** straight top-down for every map subject (ground, buildings, camp scene), with a slight "front face" cheat on tall objects so volumes read. Characters for the map are also seen from above (head, shoulders, feet), not from the side. The portrait is the only front-facing image.
- **Content:** stone age. Hide, bark, logs, reeds, river stones, thatch, bone, ochre, charcoal. No metal, no dyed cloth, no modern anything. A small founding camp by a river at a forest edge.
- **Mood baseline:** the game's words are 촘촘함 (dense), 흙과 돌 (earth and stone), 화덕 불빛 (hearth light). A concept may interpret them its own way (warm or cold, soft or hard), but every scene has a hearth as its warmest point and people must read against the ground.
- **No text, no watermark, no UI** in any image. Magenta background only for the two cut-out subjects (§4.7); all others are full scenes or objects on their natural ground.
- **Fixed subject list** (identical prompts' content across concepts; only the style block changes):

| # | subject | what the image shows | size |
|---|---|---|---|
| S1 | gatherer | a stone-age woman with a reed basket of berries, seen from above, standing on packed earth | 1024² |
| S2 | woodcutter | a stone-age man with a stone axe, three logs on his shoulder, from above | 1024² |
| S3 | elder and child | an elder leaning on a stick and a child beside, from above, on the same ground | 1024² |
| S4 | hut | a 3×3-tile log-walled hut from above, no roof (interior visible: packed earth floor, four bedrolls, a small stone hearth), south doorway | 1024² |
| S5 | camp props | lean-to shelter, a stone hearth with fire, a stacked log pile, a berry bush, on packed earth, from above | 1024² |
| S6 | ground swatch | one image split in four quadrants, each a seamless-looking top-down ground: river bank with shallow water; forest floor under canopy; grassy meadow; bare hill rock | 1024² |
| S7 | camp scene | the whole founding camp from above at the scale where a person is about 64 px tall: hut, three lean-tos, hearth, stockpile, river bank on one side, forest edge on another, eight people at work | 1536×1024 (or the model's widest) |
| S8 | portrait | a 3/4 front portrait of the woodcutter's face and shoulders, for an inspector panel | 512² |

# 3. PART 1 — The concepts (`art/concepts/<nn>_<slug>/CONCEPT.md`, one each)
Eight concepts, deliberately far apart. For each write: a one-line pitch (Korean), the style block that is appended to every prompt (English), the palette intent (5 colour words), the line/shading rule, the two reference painters or traditions it leans on (described, not copied), and the **risk for the game** (readability at 64 px, consistency across hundreds of assets, mood fit). You may replace at most two of these with concepts of your own if you record why.

| # | slug | pitch (Korean) | style block (summary) |
|---|---|---|---|
| 01 | `earth_and_ash` | 흙과 재. 어둡고 차분한 유화풍, 화덕만 뜨겁다 | painterly, muted earth tones, soft directional light, visible brush texture, low saturation except fire |
| 02 | `folk_woodcut` | 민화·목판. 굵은 선, 평면 색, 따뜻함 | bold dark contour lines, flat colour fills, limited warm palette, woodblock texture |
| 03 | `watercolor_field` | 수채 들판. 맑고 부드럽고 밝다 | watercolour washes, paper grain, soft edges, bright but natural greens and ochres |
| 04 | `diorama` | 디오라마. 작은 모형 세상 | miniature model look, soft global illumination, slight tilt-shift, clean materials, gentle shadows |
| 05 | `ink_and_tone` | 잉크 선화. 옛 도감 같은 정밀함 | fine ink line work, two-tone shading, restrained colour wash, scientific-plate feel |
| 06 | `cave_ochre` | 벽화. 황토와 숯의 원시성 | ochre / charcoal / bone palette, stylized silhouettes, rough mineral texture, ritual feel |
| 07 | `storybook_gouache` | 동화 과슈. 두껍고 둥근 형태 | opaque gouache, rounded simplified shapes, warm shadows, friendly proportions |
| 08 | `cold_north` | 북방의 추위. 푸른 그림자, 거친 질감 | cool desaturated palette, long blue shadows, rough stone and fur textures, stark light |

# 4. PART 2 — Generation (`/Users/rexxa/work/artconcept-data/<nn>_<slug>/S<k>_<try>.png`)
1. For each concept, generate **S1 first** and iterate (≤ 4 tries) until it matches the CONCEPT.md style block by your reading. Freeze it as the concept's **reference image**.
2. Generate S2–S8 **with the reference image attached** ("match this exact style, same lighting and material treatment") — ≤ 3 tries each; keep the best; log every try (prompt, try number, kept / rejected and why) in `art/concepts/<nn>_<slug>/LOG.md`.
3. **Consistency rule:** if S2 or S4 visibly departs from S1's style after three tries, record "consistency risk: high" for the concept; do not spend more tries.
4. **Camera rule:** a map subject that comes out isometric or side-on is a rejected try even if beautiful.
5. Keep every try. Nothing is deleted or retouched.
6. Budget: about 8 concepts × 8 subjects × ~2 tries ≈ 130 generations. Record the real count in `COST.md` (per concept: tries, kept, wall time).
7. **Cut-outs for the readability test:** S1 and S2 are additionally generated once on a flat magenta background (same prompt + "isolated on solid #FF00FF background") so they can be placed on the S6 ground in §6.1. Background removal by chroma key only.

# 5. PART 3 — Boards (`art/concepts/boards/`, PIL, each ≤ 1.5 MB)
- `concept_<nn>_<slug>.png`: the concept's eight subjects on one 2×4 board, each subject downscaled to 480 px on its short side, labelled with the subject id and the concept pitch in Korean (Galmuri or any system font; label boxes dark).
- `overview_people.png`: S2 (woodcutter) of every concept in a row, same size — the fastest way to feel the differences.
- `overview_scene.png`: S7 of every concept, two per row.
- `overview_hut.png`: S4 of every concept in a row.
- `readability.png`: the §6.1 test images, one row per concept.

# 6. PART 4 — Checks (pre-registered; informative, not pass/fail — the owner picks)
## 6.1 Readability at game scale
For each concept: place the S1 and S2 cut-outs onto a 4×4 tiling of its S6 forest-floor and meadow quadrants, downscaled so one tile = 64 px and a person ≈ 56 px tall (the real game scale at 1×); crop 640×360. A fresh reviewer (a second Codex agent or the model in a new context, images only, no concept names) answers for each crop: "How many people are there, what is each doing, and what is on the ground?" Score 0–3 per crop (people counted / activities named / ground named). Record verbatim answers.
## 6.2 Mood fit
The same reviewer gets the eight S7 scenes unlabelled and ranks them for "촘촘함, 흙과 돌, 화덕 불빛" fit, and separately for "I would want to watch this for an hour". Record the two rankings verbatim.
## 6.3 Consistency
For each concept, your own reading of S1 vs S2 vs S4: same medium, same light, same line weight? One line each; "consistency risk: low / medium / high".
## 6.4 Production fit
For each concept, a short note on what the next pass would have to solve: how a cut-out rig's parts (head, torso, arms, legs, held items) would be generated in this style; how seamless ground textures behave in this style (does the medium hide seams?); how 200 assets would stay consistent.

# 7. PART 5 — The book and the report
- `docs/v8/artconcept/CONCEPT_BOOK.md` (Korean): one section per concept — pitch, the board link, the four check results, your honest strengths/weaknesses; then a comparison table (readability score, mood rank, watchability rank, consistency risk, production notes); then **your recommendation of two finalists and why**, without choosing for the owner.
- `docs/v8/artconcept/COST.md`.
- `docs/v8/artconcept/CONCEPT_REPORT.md` (Korean, §8 format). Push; `git ls-remote origin v8/art-concept`. No merge.

# 8. 구현 완료 보고 (한국어)
```
## 구현 완료 보고 — ART-CONCEPT
### 구현 의도
### 공통 조건 (카메라, 내용, 분위기 기준, 주제 8개)
### 컨셉 8개 (슬러그, 한 줄 피치, 참고 이미지 확정까지 시도 수)
### 보드 (경로 목록)
### 검사 결과 (6.1 가독성 점수표, 6.2 분위기·시청 순위 원문, 6.3 일관성 위험, 6.4 제작 적합 메모)
### 비용 (총 생성 수, 컨셉별 시도·채택, 소요 시간)
### 추천 결선 2개와 이유 (선택은 오너)
### 변경된 파일 목록 (커밋별)
### 제한사항 (카메라 규칙 위반으로 버린 수, 모델이 못 한 것, 드롭한 컨셉·주제)
git ls-remote 결과
```
