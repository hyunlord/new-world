# GOAL: WorldSim — M1-ART: the colony view that looks like the mock-up (real tiles instead of placeholders, canopy instead of dots, buildings and log piles you can read, people you can find; no simulation change)

Repository `hyunlord/new-world`. Base: **`v8/main` = `v8/wip` = `bba3110e4b350d30da8ec556fb0fb5188f2e714c`** (V8-PLAY).

Executed by **Claude Code** on the Mac in `/Users/rexxa/work/new-world-pass4`. Windowed Godot runs are authorized and required. Target 8 hours, hard stop 12. No DGX, no ComfyUI.

## Why this pass exists

The owner played M1 on 2026-10-21 and said the visuals are too crude to judge the game. The planner compared the committed shots (`docs/images/m1_*.png`) with the chosen mock-up (`docs/images/v8_target_view_b.png`):
- **The palette is right.** On-screen grass is exactly the mock-up's `(58, 74, 46)`; the trees use the mock-up's greens. Do not redo the palette.
- **Everything else is placeholder.** The hut is a grey square outline; the shelter is a box icon; 34 logs in the stockpile show as four small squares; trees are identical flat circles on every forest tile, so a forest reads as green dots and hides the people; the ground is a flat field with no texture clusters; people are 16-px stubs with no outline weight.
- **The mock-up's content is missing on purpose** (huts with beds, a farm, a cave): those are M2–M3. But the mock-up's *rendering* of what M1 already has (ground, trees, hut, logs, stockpile, hearth, people) is reachable now, and that is this pass.

M1-ART changes **only** `tools/v8/gen_tiles.py` (renamed from `gen_temp_tiles.py`), `assets/v8/**`, `scripts/world/**`, `scripts/ui/**`, `shaders/**`, `localization/**` and docs. **No `rust/**` change** except the two reads in 3.3 if needed. Baseline identity, colony determinism and the M1 evaluation stay valid because the simulation does not change.

---

# 0. Rules
- AGENTS.md and the skills apply (`worldsim-godot` first, then `worldsim-pitfalls`).
- Commit to `v8/wip`, messages `v8-m1art(pN): …`; `v8/main` by fast-forward after a green gate. Never `lead/main`, `main`, `codex/*`; no force-push; explicit paths; hook file and local ignore lines untouched.
- Never ask the owner; decisions in `docs/v8/m1art/ART_DECISIONS.md`.
- Parts: 0 = §4; 1 = §5 (tiles); 2 = §6 (layers); 3 = §7 (UI and bugs); 4 = §8 (checks); 5 = §9 (docs, gate, report). Push after every Part.
- **Time rule:** past 12 hours, finish the step, push, go to Part 5. Drop order: 6.6 (hearth flicker) → 6.5 (sleep pose) → 5.4 tree species 3 → 7.3 (order-bar icons). Never drop: the ground, canopy, hut, log piles, people, the two bug fixes, the blind test, the contact sheet, the gate.

---

# 1. MACHINE
Pass4 checkout only; data in `/Users/rexxa/work/m1art-data/`; `GODOT_BIN` as `tools/v8/godot_bin.sh`. Close every window you open.

# 2. REFERENCES (read before drawing)
- `docs/images/v8_target_view_b.png` — the target. `docs/v8/m1/mockup_gen_reference.py` — how it was drawn (palettes, tile painters, person, items). Reuse its painters where they are better than `gen_temp_tiles.py`'s.
- `docs/VISION.md` §6 — hierarchy: **people and events > items and work spots > walls and floors > ground**. Mood words: 촘촘함, 흙과 돌, 화덕 불빛.
- `docs/images/m1_03_work_2x.png`, `m1_04_hut_done_2x.png`, `m1_06_far_1x.png` — what is wrong now.
- Pixel-art craft rules for this pass (write them into `worldsim-godot` as a section):
  1. Every sprite has a 1-px dark outline `(20, 18, 16)` on its silhouette; ground tiles have none.
  2. Three to four values per material (shadow, base, light, highlight); no gradients, no anti-aliasing.
  3. Light from the top-left; cast shadows down-right, 1 px, darker ground colour, not black.
  4. Ground is the calmest layer: variation by **clusters** (noise), never by per-tile random colour.
  5. Nothing on the ground competes with a person in value contrast: ground and ground-level items stay within the mid range; people get the brightest skin/tunic values and the heaviest outline.
  6. Text on the map has a dark backing box (`art_ui.gd` style).

# 3. CONTEXT (verify)
- 3.1 `tools/v8/gen_temp_tiles.py` (540 lines) draws `assets/v8/temp/tiles.png` + `tiles.json` (name → rect). Painters: `grass`, `dirt`, `water`, `hill`, `mountain`, `tree(variant)`, `hearth`, `shelter`, `hut`, `hut_ghost`, `person_layers(child)`, `shadow`, `carry_log`, log stacks.
- 3.2 Layers in `scripts/world/colony/`: `terrain_layer.gd` (TileMapLayer, 4 grass variants, dark patches), `tree_layer.gd` (one tile per sim tree), `overlay_layer.gd` (zones, chop marks, item stacks, blueprint ghosts), `building_layer.gd`, `agent_layer.gd` (pooled Node2D: shadow + skin/hair/tunic modulate, carry bundle, tool, tween), `view.gd` (palettes `TUNICS`, `HAIRS`, `SKIN`, `ELDER_HAIR`). Camera: integer zoom 1–4×, default 2×. `daylight_controller.gd`: CanvasModulate day/night tints and hearth glows.
- 3.3 The agent snapshot has `states` and `sleeps` columns (`sim-bridge/src/ffi/snapshot.rs` ≈ L390–397). If "asleep" and "facing" cannot be derived from existing columns, you may add **read-only** columns (a `collect_*` change plus the API list update; no system change).
- 3.4 Trees are simulation objects (`SimResources.trees`, one per forest tile outside the clearing, ~10,000). **Do not change their placement**; change how they are drawn.
- 3.5 The two bugs from V8-PLAY: `scripts/ui/panels/chronicle_panel.gd` `NOTABLE` lacks kind 22 (`HutBuilt`); `UI_CAUSAL_PANEL_PLACEHOLDER` still contains the dev marker "(γ-2-β)".

---

# 4. PART 0 — Baseline shots and the checklist
1. Commit this goal as `docs/v8/m1art/ART_GOAL.md`.
2. Capture the M1 set once more at the current head with `tools/v8/capture.gd --m1-set` → `$DATA/before/` (not committed). These are the "before" images.
3. Write `docs/v8/m1art/CHECKLIST.md`: the mock-up elements (ground texture clusters, worn earth, river banks, hill rock, canopy, tree shadow, hut, shelter, hearth, stockpile floor, log piles ×3, blueprint ghost + label, person silhouette, person shadow, tunic colours, carry bundle, tool, child size, elder hair, sleep pose, night light) each with a row: before (있음/임시/없음) → target → after. Fill "after" in Part 4.

---

# 5. PART 1 — The tiles (`tools/v8/gen_tiles.py` → `assets/v8/tiles.png` + `tiles.json`)
Keep the generator deterministic (`SEED`), unit-tested (sheet size, every name present, no sprite exceeds its rect, outline colour present on every object sprite). Delete `assets/v8/temp/` when the new sheet is wired.

## 5.1 Ground (16×16, no outline)
- grass: 6 variants in two families, `g` (base) and `G` (dark cluster), each with 1–3 blade marks and ≤ 2 speck pixels; the families differ by ~10 in value only.
- forest floor: 4 variants, darker and browner than grass (`FOREST_FLOOR`), with leaf-litter specks; used under canopy.
- worn earth: 4 variants (`d`), and 2 "edge" variants with grass dithering on one side for the clearing boundary (used by the autotile in 6.1).
- hill: 3 variants with rock speckle; mountain: 3 variants of cracked rock with a lighter top edge.
- water: 4 deep variants with 2-px ripples, 4 shallow variants, and **8 bank tiles** (4 sides + 4 inner corners) with a light sand lip, for the 4-bit autotile.
- stockpile floor: tamped earth variant with a faint 1-px stake at each corner cell (drawn by the overlay from a corner/edge/interior set of 3 tiles).

## 5.2 Canopy (trees)
- 3 species × 3 size variants, drawn **24×24** (canopy larger than the tile, so neighbours overlap into a mass): round oak, lumpy birch (lighter, with trunk-white specks), dark pine (pointed, with a lighter crown). Each with highlight at the top-left, a darker underside, and a 1-px outline.
- A separate 20×8 canopy shadow sprite (translucent dark, alpha 90) placed 2 px down-right.
- A stump (16×16) for a felled tree's tile: used by the overlay for 1 game day after felling (read from the chop mark removal; if no event exists, draw the stump while the tile is in the "recently felled" set the tree layer keeps locally from version diffs).

## 5.3 Buildings and objects
- **Hut 48×48 (3×3):** log walls seen from above (two courses of horizontal logs with end-grain dots on the outer ring), a 1-tile doorway on the south side, a packed-earth floor with 4 bedrolls (2×2 arrangement, hides + a lighter pillow) and a small hearth stone. No roof (the mock-up shows interiors).
- **Hut ghost:** the same silhouette in the blueprint blue at alpha 80 with a dashed outline; the delivered logs are drawn as real log piles on the footprint's south tiles as they arrive (overlay, from `delivered`).
- **Shelter (V7 1-tile):** a lean-to, 16×20 (overhangs 4 px up): two poles, a hide roof, a dark opening.
- **Hearth:** a 16×16 stone ring with 2 fire frames (6.6).
- **Log piles:** 3 sizes (1–8, 9–16, 17–25) of stacked round logs with end-grain, 16×16, consistent with the mock-up's `item_sprite('log')`.
- **Berry bush** full/bare and deadwood: keep, restyle to the outline rule.

## 5.4 People (16×16, one tile)
- Adult: 3 greyscale layers (skin, hair, tunic) + outline layer (opaque, drawn last, not modulated). Proportions: head 6×5 with 2 eye pixels, tunic 8×6 with a belt line, legs 2 px. **Two walk frames** (legs alternate) and one **sleep frame** (lying, 16×8, head to the left).
- Child: 12×12 with the same layers and 2 walk frames.
- Carry bundle: 3 logs across the shoulders (not above the head), outlined; tool: axe/hammer 5×7 held at the right hand.
- Shadow: 10×4 ellipse, alpha 110, under the feet.
- Facing: a `flip_h` for west; no separate sprites.

## 5.5 Marks
- chop mark: a small axe glyph (not an X) at the tile's top-right, 7×7, outlined, yellow.
- selection: 1-px yellow rectangle; hover: 1-px white at alpha 120.

---

# 6. PART 2 — The layers
## 6.1 Terrain (`terrain_layer.gd`)
- Variant choice by **value noise** (the mock-up's `value_noise`, ported to GDScript with the world seed): threshold → dark-grass clusters; forest floor under every sim tree tile and its 8 neighbours; worn earth = the existing clearing plus a noise-eroded edge using the dither tiles.
- **Water autotile:** 4-bit neighbour mask → bank tiles; shallow ring one tile wide outside the bank.
- Rebuilt only on `world_reset`.

## 6.2 Canopy (`tree_layer.gd`)
- Per sim tree: species by keyed hash of `(x, y)` (3 species, weighted 50/30/20), size by hash, a ±2 px jitter offset, shadow sprite under it. Draw order by `y` so southern canopies overlap northern ones.
- **Never on top of a person:** canopy sprites render below the agent layer, and a person standing on a forest tile gets a 1-px light outline halo (agent layer, 6.4) so it stays visible.
- At 1× zoom: draw the 16×16 variant without shadow (impostor) to keep 60 FPS over the full map.

## 6.3 Overlay (`overlay_layer.gd`)
- Stockpile zones: the 3-tile floor set (corner/edge/interior) instead of a flat tint; a 1-px darker border.
- Log piles by count thresholds; a tiny count label only on hover.
- Blueprint: the ghost + delivered piles + the label "통나무 d/12 · 작업 p%" in a backing box; a 3-px progress bar under the footprint.
- Stumps (5.2).

## 6.4 People (`agent_layer.gd`)
- Walk frame alternates every move step; idle = frame 0; sleeping (from the snapshot's sleep column, or 3.3's read) = sleep frame on the bed tile; `flip_h` when the last step went west.
- Outline layer always; halo when on a forest tile or at night.
- Tunic palette: keep the 6 household colours but raise their value so the darkest is ≥ 25 % lighter than forest floor (measure with PIL in a unit test on the sheet: luminance).
- Carry bundle on the shoulders; tool at the hand; elder hair grey; child 12×12.

## 6.5 Sleep pose (droppable)
Use the sleep frame when the agent is asleep; place it centred on its tile.

## 6.6 Hearth and night (droppable)
- Hearth fire alternates 2 frames every 0.4 s; the existing glow stays.
- At night, people inside the hut and shelters get a warm modulate from the glow radius (reuse `daylight_controller.gd`'s night factor).

---

# 7. PART 3 — UI and the two bugs
## 7.1 Bugs
- Add kind 22 to `chronicle_panel.gd` `NOTABLE`; verify with the capture that "오두막이 완성됨" appears.
- Remove "(γ-2-β)" from `UI_CAUSAL_PANEL_PLACEHOLDER` (en and ko); recompile localization.
## 7.2 Map text
- Blueprint label and toasts get the backing box; font stays Galmuri11 at the design-pixel rule.
## 7.3 Order bar (droppable)
- Each F-key button gets a 12×12 icon from the sheet (axe, crate, hut, eraser, X); active mode highlighted; the hint text "fn + F1–F5 (Mac)" when the OS is macOS.

No new localization keys are expected. If you add one, list it en/ko in the report.

---

# 8. PART 4 — Checks (pre-registered)
## 8.1 Contact sheet
`tools/v8/contact_sheet.py` → `docs/images/m1art_contact_sheet.png`: for each of P1, P3, P4, P5, P6, P7, P9 the **before** (Part 0) and **after** side by side, and the mock-up crop beside the after of P3. Replace `m1_contact_sheet.png` (≤ 12 images in `docs/images/`; keep the 5 M1 shots only if the total stays ≤ 12, else keep P3 and P6).

## 8.2 Blind read test (fresh `general-purpose` subagent, images only, no code)
Give it 10 crops (96×96 at 2×) from the after shots, shuffled, and ask "what is this?": hut, shelter, hearth, stockpile with logs, log pile on bare ground, blueprint ghost, person carrying logs, person chopping, child, sleeping person at night. **Bar: ≥ 8/10 named correctly** (synonyms count). Record answers verbatim.

## 8.3 Find-the-people test (same kind of subagent, second instance)
Give it the after P3 and P6 and ask it to mark every person (x, y). **Bar: ≥ 90 % of the people in the snapshot found at 2×, ≥ 70 % at 1×.** Compare with the agent snapshot positions through the capture tool's dump.

## 8.4 Mock-up comparison
The CHECKLIST "after" column: **every row 있음**, except rows whose content M1 does not have (farm, cave, side panel content) which are marked "M2+".

## 8.5 Performance
`tools/v8/perf.sh`-style FPS at 1× and 2× over 120 s at 4× speed, before and after: **after ≥ 60 FPS at both**, and ≥ 0.9 × before.

## 8.6 Unchanged simulation
`tools/v8/identity.sh` passes; `colony_determinism` and `replay` gate steps pass; `git diff --stat bba3110e..HEAD -- rust/` shows only 3.3's read columns, if any.

If 8.2 or 8.3 misses the bar: one revision round on the failing sprites, then rerun with a new subagent. Report both rounds.

---

# 9. PART 5 — Docs, gate, report
- CHANGELOG, ROADMAP (an "M1-ART" row, done), HISTORY line, ARCHITECTURE §7 (new layer files, the sheet, the autotile), `worldsim-godot` (craft rules, autotile recipe, how to add a sprite, canopy/agent draw order lesson), `worldsim-pitfalls` lessons.
- `README.md` "지금 화면" → the new contact sheet.
- Independent review (one fresh subagent): the diff for any `rust/` change beyond reads, and the GDScript for per-frame allocations in the layers.
- Gate green on a clean commit → fast-forward `v8/main` → report commit → docs step → fast-forward.

---

# 10. 인게임 확인사항 (오너용)
1. 2배 확대에서 **사람이 먼저 보인다.** 숲 안에 서 있어도 밝은 테두리 덕분에 찾을 수 있다.
2. 숲이 점이 아니라 **덩어리**로 보이고, 나무 종류와 크기가 섞여 있다. 벤 자리에는 그루터기가 남는다.
3. 땅에 **얼룩과 길**이 있다. 캠프 주변은 밟힌 흙, 강가는 모래 둔덕, 물은 깊은 곳과 얕은 곳이 다르다.
4. 오두막이 **통나무 벽과 잠자리**로 보이고, 청사진은 통나무가 오는 만큼 바닥에 쌓인다. 거처는 가죽 지붕의 움막이다.
5. 비축 구역은 다진 흙 바닥에 말뚝이 있고, 통나무가 쌓일수록 더미가 커진다.
6. 사람은 걸을 때 다리가 움직이고, 통나무는 어깨에 메고, 밤에는 잠자리에 누워 있다.
7. 밤에 화덕 불이 흔들리고 주변 사람이 따뜻한 색으로 보인다.
8. 연대기에 "오두막이 완성됨"이 뜨고, '왜?' 패널에 개발 표식이 없다.
9. 1배 전체 지도에서도 끊기지 않는다.
10. 바탕화면 `WorldSim.command`로 그대로 실행된다.

# 11. 구현 완료 보고 (한국어; `docs/v8/m1art/ART_REPORT.md`와 최종 메시지)
```
## 구현 완료 보고 — M1-ART
### 구현 의도
### 구현 내용 (시트·레이어·UI·버그, 파일별)
### 구현 방법 (노이즈 군집, 오토타일, 캐노피 겹침, 사람 레이어, 측정 방법)
### 검사 결과 (8.1 시트 경로, 8.2 블라인드 10문항 답과 점수, 8.3 사람 찾기 비율, 8.4 체크리스트 표, 8.5 FPS 전후, 8.6 identity)
### 변경된 파일 목록 (커밋별)
### 제한사항 (드롭한 것, 아직 임시인 것, M2에서 그릴 것)
### 오너 결정 필요
### Gate 결과
git ls-remote 결과
```
