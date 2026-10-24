# GOAL: WorldSim — M-ART2.1: finish the in-game look before M-EXT — textured water with rounded shorelines instead of the flat blue staircase, summer-green trees with seasons from ART-TREES, the ¼× map that reads like a map, the owner's playtest notes, and the D19 commit brought through a gate.

Repository `hyunlord/new-world`. Base: **`v8/wip` = `a1c20220b5dc1d4c86fc983950152b7691ae6a41`** (D19 on top of `v8/main` `d796fcac`, ungated). **Art source:** the head of `v8/art-trees` when Part 0 starts — verify it contains `docs/v8/arttrees/TREES_REPORT.md`, record its SHA in DECISIONS; if the report is absent, stop and report (do not import an unfinished art branch).

Executed by **Claude Code**, unattended, in `/Users/rexxa/work/new-world-pass4`. **One session per checkout: before starting, confirm no other Claude Code session is using this working tree (`git status` clean, no foreign uncommitted files); if there are foreign changes, stop and report.** Windowed Godot runs for 12.3 and FPS. Target 8 hours, hard stop 10.

## Why
M-ART2 (`d796fcac`, gate 34/34) put the adopted art in the game. Two things still read wrong, and one process slip needs closing:
- **Water.** D19 made rivers a flat saturated light blue to guarantee contrast; it lost the 05 water texture, and the 64-px tile staircase of the river outline is now the most visible shape on screen. Contrast must come from value and hue while the texture stays, and the outline must be rounded — the simulation's river is tile-shaped, the drawing need not be.
- **Trees.** The ochre forests are the TRIAL2b sources, not a tint bug (D19 evidence: ΔS = 0). ART-TREES makes summer-green species, bare winter forms, canopy masks, season tints and a canopy-mass texture for ¼×, as data (`art/flora/species.json`, ADR 0033).
- **Process.** D19 was made by a second session in this checkout while the first ran its gate; the gated commit was clean, but `a1c20220` has never been gated. This pass gates it.

# 0. RULES
- AGENTS.md (ADR 0033 extensibility contract first) and the skills. Commit to `v8/wip` (`v8-mart21(pN): …`), `v8/main` by fast-forward after a green gate. Never `lead/main`, `main`, `codex/*`, `v8/art-*`; no merge of art branches (import by `tools/v8/art_import.py` from the pinned commit); no force-push; explicit paths; leave the untracked `.ignore`, `docs/revitalize/`, `.gitignore` change alone.
- **Simulation untouched** (no Rust change outside `sim-bridge` reads if any are needed; identity byte-identical).
- **Never ask the owner** → `docs/v8/m_art2_1/MART21_DECISIONS.md`. If `docs/v8/m_art2_1/OWNER_FEEDBACK.md` exists at start (the owner's playtest notes), every item in it is in scope unless it needs simulation changes (those go to M-EXT, recorded).
- Parts: 0 = §4 · 1 = §5 (water and shores) · 2 = §6 (trees and seasons) · 3 = §7 (¼× map, LOD markers, owner notes) · 4 = §8 (verification, docs, gate, report).
- Drop order: portrait grey hair for elders → LOD household rings → spring tint. **Never drop:** rounded shores with textured water, the tree import with seasons, ¼× canopy baking, the gate on the final commit, the compare sheet, the report.

# 1. CONTEXT (verify)
- Renderer: `scripts/world/art/{art_data,material_resolver,ground_baker,wall_baker,tone}.gd`, `ground_layer.gd` (16×16 chunk bake, LRU 48, reduced textures below 1×), `shaders/ground_chunk.gdshader` (water under transparent chunk pixels, UV drift), `tree_layer.gd`, `agent_layer.gd`, `lod_markers.gd` (MultiMesh), `daylight_controller.gd`; params `assets/v8art/render_params.json` (F12 reload); bindings `assets/v8art/bindings.json` (`trees.species` picked by a keyed hash because the simulation has no species).
- D19 params: `water.colour [0.42, 0.62, 0.76]`, `texture_std 0.025`, `minimum_land_luminance_difference 0.1`, `minimum_land_hue_distance 0.12`; `ground.tone {mean_max 0.38, saturation_max 0.30}`; `ground.map_mode.tint_strength {0.5: 0.5, 0.25: 0.8}` with `tint_by_ground`.
- FPS at M-ART2: ¼× 120.65 against a 120 floor (2,252 standing sprites drawn); the plan's step 3 was "bake trees into chunks at ¼×".

# 2. DESIGN
## 2.1 Water and shores
- **Rounded outline:** at bake, compute per chunk (with a 2-tile apron) a signed distance field from the water/land tile boundary (exact Euclidean distance transform on a 4×-upsampled tile grid, then bilinear); water coverage = smoothstep over ±6 px around a contour offset so that a straight shore stays on the tile edge while a 90° step becomes a quarter circle of radius ≈ 22 px; the bank (10-px dithered falloff, 2-px ink on the land side) follows that contour, not the tile edge. Fords keep their stones on the rounded water.
- **Texture back:** water samples the 05 water texture again (D19's flat colour removed); contrast is guaranteed by a **value/hue lift applied to the texture** (shift its mean to meet `minimum_land_luminance_difference` and `minimum_land_hue_distance` against the adjacent land's mean in that chunk) with `texture_std` restored to the texture's own (measure; expect ≈ 0.06–0.09). Same rule at night and in winter (ice tint droppable).
- **Hill/rock edges:** the same SDF rounding for undug hill tops against open ground (faces stay straight where they hang).
- Params: `water.shore_radius_px`, `water.contour_soft_px`, `water.texture_lift` (on/off), `hill.edge_radius_px`.

## 2.2 Trees and seasons
- Import ART-TREES' adopted files (extend `tools/v8/art_import.json`; manifest + hashes), `art/flora/species.json` and `habit_defaults.json` as `assets/v8art/flora/`.
- `tree_layer.gd` resolves `(species, size, season, variant)` through a GDScript port of `resolve_flora.py` (**parity test** like the material resolver, gate step `resolver_parity` extended); species still by the keyed hash until M-EXT gives trees species (record in DECISIONS that the hash must be replaced by the snapshot's species column then).
- **Season shader:** canopy mask + the species' season Lab shift (precomputed to RGB multipliers as for faces); winter swaps deciduous trees to bare frames and draws the snow cap by mask; the calendar's season from `get_world_info`/time snapshot drives it; transitions blend over the first day of a season.
- **¼× canopy baking:** at ¼× (and only there), trees are baked into the ground chunk as the canopy-mass texture under each tree's footprint (tinted by season, darkened at the canopy edge), and the standing tree sprites are not drawn; target ¼× FPS ≥ 180.

## 2.3 The ¼× map and markers
- Keep D19's `map_mode` tints; tune after trees so that forest (green canopy mass), meadow, bare earth, rock and water are five distinguishable classes at ¼× (measure: the mean colours of the five classes on the seed-0 ¼× screen have pairwise ΔE ≥ 10; record).
- **LOD markers** (droppable part): marker fill = top colour, ring = household colour from a 6-hue palette in params, so markers stop reading as one tan crowd.
- **Elder portraits** (droppable): a portrait hair mask (Lab clustering, as the sprite masks) and the grey-hair shift for elders' portraits.

# 3. PERFORMANCE
Same scenes and method as M-ART2 (`tools/v8/mart2_fps.sh`): seed 0 day 20 at 1×/½×/¼× ≥ 120 FPS (¼× target ≥ 180 with baked canopy); 1,000 people at ½×/¼× ≥ 60; chunk rebake ≤ 4 ms median (the SDF adds cost — report it; if over, compute the SDF on the 2×-upsampled grid instead); texture memory ≤ 512 MB.

# 4. PART 0 — Intake
1. Commit this goal as `docs/v8/m_art2_1/MART21_GOAL.md`; record the art SHA and the clean-checkout check.
2. Read `OWNER_FEEDBACK.md` if present; list each item with in-scope / M-EXT in DECISIONS.
3. Run the M-ART2 gate on `a1c20220` once to know where D19 stands (record failures; do not fix in Part 0).

# 5. PART 1 — Water and shores
§2.1. Tests: a synthetic L-shaped river bakes a quarter-circle shore (measure the contour's radius ±3 px); a straight shore stays on the tile edge (±1 px); the contrast rule holds at day/night/winter on seed 0 chunks (min ΔY, hue distance recorded); texture std restored; fords on rounded water; no square step longer than 24 px along any water edge (the STRUCT luminance-step instrument).

# 6. PART 2 — Trees and seasons
§2.2. Tests: flora resolver parity; four seasons render per species (summer canopy saturation ≥ 0.30 after the game's tone path — trees are never tone-capped); winter deciduous = bare frames; the data-only `test_alder` from ART-TREES renders without code change; ¼× canopy bake on, sprites off; FPS.

# 7. PART 3 — ¼× map, markers, owner notes
§2.3 and the owner's notes. Tests: the five-class ΔE measure; marker rings by household; elder portrait grey if kept.

# 8. PART 4 — Verification, docs, gate, report
- Recapture `docs/images/mart2_compare_sheet.png` (old · M-ART2 · M-ART2.1 for the same six scenes) and a seasons strip `docs/images/mart21_seasons.png` (one scene at 1× and ¼× in four seasons).
- Docs: CHANGELOG, ROADMAP (M-ART2.1 done; next M-EXT), HISTORY, ADR 0034 addendum (SDF shores, flora resolver, ¼× canopy bake), `assets/v8art/README.md`, skills (`worldsim-godot`: SDF shores, season shader, ¼× baking; pitfalls: **one session per checkout**).
- Review: one fresh subagent — code (no content literals; parity; chunk invalidation with SDF aprons; season transitions) and the compare sheet read against this goal's "Why".
- Gate (all M-ART2 steps; resolver_parity now covers flora) green on a clean commit → fast-forward `v8/main` → report → docs → fast-forward.

# 9. 인게임 확인사항 (오너용)
1. 강이 하늘색 띠가 아니라 05 물결 그림이고, 물가가 칸 모양 계단이 아니라 둥글게 휘어 있다. 땅과는 확실히 구분된다.
2. 나무가 여름에는 초록, 가을에는 참나무·자작나무만 누렇게, 겨울에는 앙상한 가지와 소나무 위 눈. 계절이 바뀌는 첫날에 서서히 바뀐다.
3. ¼×로 멀어지면 숲은 초록 덩어리, 풀밭·맨땅·바위·물이 각각 다른 색으로 지도처럼 보이고, FPS가 넉넉하다.
4. 멀리서 사람 동그라미 테두리가 가구(집안)마다 색이 달라서 누가 어느 집 사람인지 대략 보인다.
5. 플레이 노트(OWNER_FEEDBACK.md)에 적은 것이 반영돼 있다.

# 10. 구현 완료 보고 (한국어; `docs/v8/m_art2_1/MART21_REPORT.md`와 최종 메시지)
```
## 구현 완료 보고 — M-ART2.1
### 구현 의도
### Part 0 (아트 SHA, 작업 트리 확인, 오너 노트 처리 목록, a1c20220 게이트 결과)
### 물과 물가 (거리장 둥근 윤곽, 텍스처 복원과 대비 규칙 — 수치)
### 나무와 계절 (가져온 파일, 종 해석기 일치, 계절 셰이더, 겨울 앙상함과 눈, ¼× 수관 굽기)
### ¼× 지도와 표지 (다섯 지형 ΔE, 가구 테두리, 노인 초상)
### 성능 (줌별 FPS, 재굽기 시간, 텍스처 메모리)
### 화면 비교 (옛 · M-ART2 · M-ART2.1, 계절 띠)
### 변경된 파일 목록 (커밋별)
### 리뷰 결과
### 확인된 제한사항
### 오너 결정 필요
### Gate 결과
git ls-remote 결과
```
