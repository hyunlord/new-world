# GOAL: WorldSim — M-ART2: the adopted art in the real game. Import the approved assets from the art line, draw the colony at 64-px tiles in one projection, people from route I with per-person face variants, walls/floors/doors from the data-driven material system, ground/water/edges and hill faces by code, LOD markers when zoomed out, day/night/winter by tone — every rule a parameter in a file the owner can tweak. And write the extensibility contract (ADR 0033) that this pass is the first to obey.

Repository `hyunlord/new-world`. Base: **`v8/main` = `29f720aa52c9939868b65b6ea2cdfe0319e55576`** (M5.5; gate green at `ae443aa4`). `v8/wip` at the same commit. **Art source: `v8/art-polish2` = `c8ed86087c90ed5429716713e930cc5f6a9e1332`** (read-only; it contains the whole art lineage: route I, ENV1, STRUCT, POLISH, POLISH2).

Executed by **Claude Code**, unattended, in `/Users/rexxa/work/new-world-pass4`. Windowed Godot runs are needed (12.3 and the FPS runs). Target 20 hours, hard stop 24. The art session may run on `v8/art-*`; do not push there and **do not merge art branches** — this pass imports files from a pinned art commit.

## Why
The owner has approved, in order: route I whole-figure people (ART-PROD3, 34/36 looks); one projection — the ground from above, everything standing in the same front 3/4, walls/hill faces/water/edges drawn by code (ART-STRUCT, 2026-10-24 "ㅇㅇ ㅇㅋ"); wall tops that carry the material along the run, option (b), sharper than rendered (POLISH2); face variants v1, six per look with matching sprite tints, the skin mask to be fixed so it never tints clothing (POLISH2 S3 variants darkened fur cloaks); items, portraits and animals (POLISH). The rehearsal compositor showed the colony readable with calm toned-down ground, ink detail on things and people, and markers when zoomed out (ENV1). None of it is in the game: the game still draws the M1 16-px sheet (`assets/v8/tiles.png` from `tools/v8/gen_tiles.py`) at integer zooms.

**Owner directive 2026-10-24 (standing): every code and design structure must be extensible.** Stone, mud brick, fired brick, concrete — walls of every era — and by extension every content kind must go through the same code. The material system proved it on the art side (mud brick and fired brick added by data only, `DATA_ONLY_PROOF.json`). This pass writes the rule down as a contract (ADR 0033) and builds the renderer to it: the renderer resolves what to draw from manifests and schemas; no item, material, style, look or animal name appears in renderer code.

---

# 0. RULES
- AGENTS.md and the skills (`worldsim-godot` and `worldsim-architecture` first). Commit to `v8/wip` (`v8-mart2(pN): …`), `v8/main` by fast-forward after a green gate. Never `lead/main`, `main`, `codex/*`, `v8/art-*`; no force-push; explicit paths; hook/ignore files and the untracked `.ignore`, `docs/revitalize/`, `.gitignore` change untouched.
- **Never ask the owner** → `docs/v8/m_art2/MART2_DECISIONS.md`.
- **Simulation untouched** except the bridge reads of §3.2 (pure collectors) and the appearance function; baseline identity and colony replay byte-identical; C1–C5 hold.
- **Extensibility (ADR 0033, written in Part 0) is checked in review and by tests:** no content names in renderer code (a grep test over `scripts/world/**` for the item/material/style/look/animal id lists from the manifests), every drawable resolved through a manifest, a "new style by data only" test on the game side.
- Parts: 0 = §5 · 1 = §6 (import) · 2 = §7 (bridge reads, appearance) · 3 = §8 (renderer: ground, water, edges, built space) · 4 = §9 (renderer: people, animals, things, LOD, light) · 5 = §10 (UI) · 6 = §11 (verification) · 7 = §12 (docs, review, gate, report). Push after every Part.
- **Time rule:** past 24 h, finish the step, push, go to Part 7.
- **Drop order:** 9.6 winter snow overlay (keep the tone) → 10.3 item icons in panels (keep names) → ¼× zoom (keep 1× and ½×) → roof debug toggle → 8.6 drought tone.
- **Never drop:** Part 0 (ADR 0033), the import with manifest and hashes, the appearance collector, 64-px ground with blended edges, water with banks, code-drawn walls/doors/floors from the material schema with the resolver parity test, hill faces, route I people with variants and the fixed skin mask, LOD markers at ½×, day/night tone, the params file with reload, deletion of the M1 sheet path, the owner screen set, FPS budgets, identity, docs, report.

# 1. MACHINE
Pass4 checkout + base worktree `../new-world-mart2-base` at `29f720aa`; an **art worktree** `../new-world-art-src` checked out at `c8ed8608` (read-only; delete it at the end). Data `/Users/rexxa/work/mart2-data/`.

# 2. CONTEXT (planner's survey; verify)
- **Renderer today** (`scripts/world/`): `colony/view.gd` (`const TILE := 16`, the M1 sheet `assets/v8/tiles.png` + `tiles.json` from `tools/v8/gen_tiles.py`, palettes, value noise), `colony/terrain_layer.gd` (TileMapLayer, water 4-bit bank masks, worn earth, hills/rock), `colony/agent_layer.gd` (442 lines: pooled Node2D layers modulated by palettes, frames stand/stride/sleep, carry/tool layers), `colony/building_layer.gd` (hearth, shelters, huts, workshops, M3 walls by 4-bit mask, doors, floors, fords, dug hill), `animal_layer.gd`, `tree_layer.gd`, `overlay_layer.gd`, `room_layer.gd`, `visitor_layer.gd`, `order_tool.gd`, `camera_controller.gd` (integer zoom 1–4×, default 2×; WASD/arrows/drag; Home/H), `daylight_controller.gd` (`TILE_SIZE 16`), `art_sprite_library.gd` + `assets/art/atlas/*` (an older production atlas), `wood_visuals.gd`, `porter_presentation.gd`. Window 1920×1080, stretch `canvas_items`.
- **Bridge reads that matter:** `get_terrain_snapshot`, `get_agent_snapshot` (`life_stages` 0 adult / 1 child / 2 elder, `tops` 0 loincloth / 1 hide wrap / 2 fur cloak, `mood_bands`, `drafted`; **no sex column** — sex is in the per-agent detail `female`), `get_space` (`wall_xs/ys`, `wall_kinds` 0 log / 1 stone / 2 door — **no material id**; floors, fords, dug), items/overlay, buildings, workshops, trees, animals, visitors. Text renderer `worldsim_probe --ascii … --people-json` (C2 agreement).
- **Art source (at `c8ed8608`):** `art/character/route_i/` + `MANIFEST.json` (34 looks × 10 frames, TEMPLATE_V3: 80×112 upright, foot anchor (40,104); 112×80 sleep, anchor (56,72)); `docs/v8/artprod3/HANDOVER.md` (tile placement `(tile_left+32, tile_top+43)`, frame plan, fallbacks for the 2 missing looks); POLISH2 face variants (portraits 128 px, per-variant tint records, masks, `docs/v8/artpolish2/` incl. the person→variant hash rule); `art/materials/` (schemas, `construction_styles.json`, `material_looks.json`, `category_defaults.json`, `resolve.py`, textures, `ADDING_A_STYLE.md`); `art/style05/PROJECTION.md`; compositor reference implementations `art/compositor/{walls,edges,ground,tone,placements}.py` and `wall_params.json`; ENV1/STRUCT/POLISH adopted environment, structures, items (`art/style05/env1/items`, POLISH items), animals (wolf + deer sheets, 35° S facing rule), `HANDOFF_PATHS.md` (76 candidate paths).
- **Owner choices fixed for this pass:** wall top option **(b)** (full width, along the run) with `blur_px 0–1` and `top_value ≈ 0.80` as starting values (tunable); face variants v1 adopted; portraits adopted; items adopted; deer and wolf adopted; roof not drawn; N facing = S (no back art yet); elders = adult look + an older face variant + grey hair tint.

# 3. DESIGN

## 3.1 ADR 0033 — extensibility contract (written in Part 0)
1. **Kinds are data, behaviour is code.** Content kinds (items, materials, construction styles, recipes, knowledge, apparel, looks, animals, pressures, eras, UI labels) live in versioned data files (RON in the simulation, JSON in art/assets); code branches on tags, categories and capabilities, never on names.
2. **Registry + resolver + fallback** for every content family: an unknown entry renders/behaves by its category default.
3. **A "data only" test per family** in the gate (this pass: the wall-style test on the game side; M-EXT adds items, recipes, pressures, animals).
4. **UI generated from data** (build menu, designations, tech map lanes, item names/icons).
5. **Every art family has template + schema + checks + `ADDING_*.md`.**
6. **Systems meet through components and events;** adding one changes others only by registration.
7. **Saves are versioned with migrations.**
8. **Every pass's code review checks for new hard-coded content names and new content enums**; existing ones are listed as debt (M5.5 D29) for M-EXT.

## 3.2 Bridge reads and appearance (Part 2)
- `get_agent_snapshot` adds `sexes` (PackedByteArray: 0 male, 1 female) and `appearance` columns from a new pure collector `collect_appearance` in `sim-bridge`: `look_ids` (index into the imported look list, by age band × sex × top × hair) and `variants` (0–5). **Hair style and face variant are pure functions of AgentId** (`hair = h1(agent_id) mod 3`, `variant = (agent_id × 2654435761 mod 2^32) mod 6`; elders pick among their look's older variants with the same hash; children among child variants) with the constants in `sim-bridge/data/appearance.ron` — not simulation state, so no save change, and the text renderer's `--people-json` prints the same `look`/`variant` (C2 agreement extended to appearance).
- `get_space` adds `wall_materials` (PackedInt32Array, `MaterialId` index; `-1` none) and `wall_styles` (the blueprint kind that built it) so the renderer resolves `(style, material)`; floors get `floor_materials` likewise.
- Bridge API list and snapshot schema updated (v7).

## 3.3 Import (Part 1)
`tools/v8/art_import.py --from <art commit> --out assets/v8art/` copies only **adopted** assets, by explicit lists in `tools/v8/art_import.json` (route I frames + manifest; face variant tint records and portraits; materials schemas, JSON and textures; adopted ground, water, edge, rock-face, structure, workshop, item, animal, tree assets; contact shadow), converts nothing but formats (PNG kept), writes `assets/v8art/manifest.json` (every file: role, source path, source commit, SHA-256, size, anchor/canvas metadata from the source manifests) and `assets/v8art/LICENSE.md` (all generated in-project; no third-party assets). Budget ≤ 40 MB for `assets/v8art/`. Re-running the import on the same commit is byte-identical (test). The archive, diagnostics and pipelines stay on the art line.

## 3.4 Ground, water, edges, built space (Part 3)
- **World scale:** `View.TILE := 64`; every layer and the daylight controller use it (no other `16` left; grep test).
- **Camera:** zoom steps **¼×, ½×, 1×, 2×** (camera zoom 0.25, 0.5, 1, 2), default 1×; wheel and `-`/`=`; pan speed scaled by zoom; bounds as now.
- **Ground** drawn per 16×16-tile chunk into baked `ImageTexture`s at world reset and when a tile in the chunk changes (dig, floor built): per tile the ground texture by terrain/floor (sampled with world offsets so textures continue), then the **noise-dithered 12-px blend** between different grounds (port `edges.py`: fixed-seed value noise, max of edge masks at corners); floors keep crisp edges against walls. **Ground tone cap** (`tone.py`) applied at bake: mean luminance ≤ 0.32, std ≤ 0.25 × the person reference, saturation ≤ 0.22 (the ENV1 locked targets).
- **Water:** a shader on a water TileMapLayer sampling the seamless water texture with world offsets and a gentle 0.05 Hz UV drift; banks by a code mask (10-px noise falloff + 2-px ink line on the land side); fords = water + the stepping-stone sprites placed by code.
- **Walls, doors, floors (the material system):** a GDScript port of `art/materials/resolve.py` (`scripts/world/art/material_resolver.gd`) reading the imported JSON; **resolver parity test** — a fixture of (style, material) pairs resolved by Python (art worktree) and GDScript must agree exactly (gate step). Assembly ports `walls.py` with option (b): top along the run from the E–W/N–S strips (world-offset continuous), material tint, `top.value/saturation/blur`, ink edges on borders to non-wall; face only where the south neighbour is not a wall; end pieces only at face ends; junction tops at corners/T/cross; doors as standing sprites in wall height. Baked per chunk with the ground (walls rarely change); a change rebakes one chunk. **Hill faces** by the same rule (undug hill with passable south neighbour draws the rock face). No style/material literal in the code (grep test).
- **Zones** as an 8 % tint with corner ticks; designations as ink marks; blueprint sites as the finished sprite with a pale outline + 25 % wash (code).
- **Data-only test on the game side:** a test-only `material_looks` entry (a new material id mapped to `mud_brick`) and a test blueprint mapping render a wall in a headless scene without any `.gd` change (assert the resolved texture set and the baked chunk's alpha coverage).

## 3.5 People, animals, things, LOD, light (Part 4)
- **People:** a pooled `Sprite2D` per person with the route I frame by look/state/facing (S, E, W = flip E, N = S), anchored at `(tile_left+32, tile_top+43)` minus the frame anchor; contact shadow 28×8 as its own sprite; frame plan per HANDOVER (walk = idle/walk_a alternating at 4 Hz game-time plus a 1-px bob; work = work_a/idle; carry; sleep). **Variant tint** by a shader with the imported skin/hair masks (one mask texture per frame) and the variant's Lab-derived target colours precomputed to RGB multipliers: **fix the skin mask** so it never covers clothing — rebuild skin masks from the route I frames with the clothing clusters excluded (fur, hide, loincloth palettes from the look) and accept a mask only if its pixels lie inside the face/forearm/leg boxes of TEMPLATE_V3 (test on all 340 frames; report counts); the two missing looks use HANDOVER's fallbacks. Selection outline and drafted red outline as shader params. Depth sort by base y (YSort), walls' faces included.
- **Animals:** wolf and deer sheets with the 35° S rule; carcass = dead frame.
- **Things:** structures, workshops, hearth states, beds, rocks, trees, items with count buckets (1 / 2–8 / 9+) from the manifest; item stacks drawn at the tile's lower third.
- **LOD:** at ½× and ¼×, people become a filled circle (10 px / 6 px screen) in the top's colour with a 1-px ink ring, plus at ½× a small activity glyph (axe, log, zz, !); animals a coloured triangle (wolf red, deer tan); items and structures stay sprites (scaled by the camera). Thresholds in params.
- **Light and season by tone:** night = value × 0.55 with a blue shift and hearth light radius 5 tiles restoring value (port the compositor's night transform as a CanvasModulate + light textures); winter = ground desaturated + snow overlay texture at ≤ 60 % alpha on ground chunks; drought (droppable) = grass hue toward ochre. People/things keep their colours except the global value change.
- **Params file:** `assets/v8art/render_params.json` (wall top value/saturation/blur, ground tone caps, LOD thresholds and marker sizes, night value and hearth radius, snow alpha, shadow alpha, bob height) — read at start, **reloaded with F12** in debug builds (toast "렌더 설정 다시 읽음"); the owner tunes by editing the file.

## 3.6 UI (Part 5)
- Inspector: the person's **portrait** (variant) at 128 px; the look name in words (e.g. "가죽 랩 · 묶은 머리"); top and variant.
- Panels that list items (storehouse, trade, workshop bills): item icons from the manifest at 32 px beside names (10.3, droppable).
- The order/zone tools draw their previews in the new scale (ghosts as the finished sprite with the pale outline).
- HUD unchanged except a zoom indicator.

---

# 4. PERFORMANCE PLAN (in DECISIONS before coding)
Budgets on the Mac (Godot 4.6, windowed 1920×1080, vsync off for measurement): seed 0 colony at day 20 (≈ 30 people) at 1×, ½×, ¼×: **≥ 120 FPS**; the 1,000-person bench scene (`worldsim_perf`'s N = 1,000 world loaded via a debug scene) at ½× and ¼× with LOD: **≥ 60 FPS**; chunk rebake ≤ 4 ms per chunk; texture memory ≤ 512 MB (report Godot's video memory monitor). Simulation tick cost unchanged (bench gate). If FPS misses: (1) atlas the frames per look, (2) MultiMesh for LOD markers, (3) bake trees into chunks at ¼×.

# 5. PART 0 — Records
1. Commit this goal as `docs/v8/m_art2/MART2_GOAL.md`.
2. **ADR 0033** (§3.1) accepted; AGENTS.md gets the rule in its first screen; `worldsim-architecture` skill gets "the extensibility checklist" and `worldsim-godot` "renderer resolves through manifests".
3. ADR 0018 status: route I, the single projection, the material system, face variants, portraits, items, animals adopted (cite the art commits and owner messages); ROADMAP rows ART-ENV1, ART-STRUCT, ART-POLISH, ART-POLISH2 and M-ART2; HISTORY.
4. M5.5 owner decisions recorded (to be implemented in M-EXT Part 0): population cap also at acceptance; trader goods include items of techniques with ≤ 2 practitioners and rare next-era artifacts; meat goes beside the hearth when no stockpile has room; F4/F9 bars rewritten with their arithmetic in M-EXT's evaluation; migrant decline cost stays −5.
5. ADR 0034 draft (the renderer's data flow: snapshot → appearance/material resolution → chunks/sprites → tone) — final in Part 7.
Push.

# 6. PART 1 — Import
§3.3. Tests: manifest complete (every referenced path exists, hash matches), size budget, idempotent import, the two fallback looks resolved. Commit the assets in reviewable chunks (≤ 200 files per commit).

# 7. PART 2 — Bridge reads and appearance
§3.2. Tests: appearance is a pure function (same agent id → same look/variant across save/load and replay); the text renderer's people JSON carries the same look/variant as the bridge (renderer agreement); wall/floor material columns match `TileGrid`; bridge API list v7; baseline identity.

# 8. PART 3 — Ground, water, edges, built space
§3.4. Tests: chunk bake deterministic (hash of a baked chunk for seed 0 at tick 0 stable across runs); no square edges (the luminance step test from STRUCT on a baked chunk); resolver parity; the data-only wall test; dig rebakes exactly one chunk; no `16` tile literal left.

# 9. PART 4 — People, animals, things, LOD, light
§3.5. Tests: anchor placement (a person on tile (x, y) has its frame's foot pixel at the computed screen point, all four facings); skin masks: 0 clothing-cluster pixels in any skin mask (all 340 frames) and the box rule; variant tint deterministic; LOD switch at the thresholds; night tone keeps people's silhouettes separable (luminance contrast person vs ground ≥ 0.15 on the night screen); params reload works.

# 10. PART 5 — UI
§3.6. `mart2_ui_test.gd` (≥ 25 checks: portrait shown, item icons, previews scaled, zoom indicator, F12 reload).

# 11. PART 6 — Verification
- **Owner screen set** (windowed, `capture.gd --mart2-set`): the same seed 0 scenes as ENV1 (day 6 noon, day 6 02:00, first winter noon) at 1× and ½×, a 2× close-up of the house with sleepers, the storehouse, the dug pocket, the ford, a workshop with three workers, the wolf attack with a drafted party, the migrant arrival; plus each scene rendered by the **old renderer at its commit** for comparison → `docs/images/mart2_contact_sheet.png` and `docs/v8/m_art2/screens/`.
- FPS runs per §4 with the numbers in the report; texture memory.
- Gate: all 30 steps + `art_manifest` (manifest/hashes/sizes), `resolver_parity`, `appearance_agreement` (in renderer_agreement), and the grep tests (no content literals in `scripts/world/**`, no 16-px literal).
- Delete the M1 sheet path once the new renderer passes: `tools/v8/gen_tiles.py`, `assets/v8/tiles.png/json`, the old `assets/art/atlas/*` and `art_sprite_library.gd` if unused, palette code in `view.gd`; record the deletions in CHANGELOG.

# 12. PART 7 — Docs, review, gate, report
- Docs: CHANGELOG; ROADMAP (M-ART2 done; 다음 → M-EXT, then M6); HISTORY; ADR 0033 final, 0034 final; ARCHITECTURE (renderer section rewritten: layers, chunks, resolver, appearance, tone, params; snapshot schema v7); `assets/v8art/README.md` (how to re-import from a newer art commit); skills (`worldsim-godot`: chunk baking, YSort with wall faces, shader tints, LOD, params reload; pitfalls).
- Review: two fresh subagents — code (extensibility grep and review checklist from ADR 0033; resolver parity; mask fix; chunk invalidation; YSort/occlusion; no simulation change beyond the bridge reads); visual (the owner screen set against the ENV1/STRUCT/POLISH2 boards: projection consistent, walls option (b), no tinted clothing, LOD readable — a written comparison, not a score).
- Gate green on a clean commit → fast-forward → report → docs → fast-forward.

# 13. Not in M-ART2
N facing art, more tops, elder-specific sprites, roof art, weather particles, animated water beyond UV drift, sound, new UI panels.

# 14. 인게임 확인사항 (오너용)
1. 게임을 켜면 64픽셀 칸의 05 화면이 나온다. 바닥은 차분하고, 사람·물건·건물만 잉크 그림으로 또렷하다. 지형 경계가 네모가 아니라 자연스럽게 섞인다.
2. 사람이 route I 그림으로 걷고, 도끼질하고, 통나무를 나르고, 잔다. 얼굴·피부·머리색이 사람마다 다르고, 노인은 흰머리다. 짙은 피부 사람의 털망토가 어둡게 물들지 않는다.
3. 사람을 클릭하면 인스펙터에 그 사람의 초상이 나온다.
4. 통나무 벽은 남쪽 면에 통나무 앞면, 위에서는 벽 방향으로 누운 통나무 윗면이 보인다. 돌벽도 같은 방식이다. 지붕 없이 방 안이 보인다.
5. 언덕을 파면 굴 입구 위에 바위 면이 생겨 동굴처럼 보인다. 강은 물결 텍스처, 여울에는 징검돌.
6. 휠로 ½×, ¼×로 멀어지면 사람이 상의 색 동그라미와 활동 아이콘으로 바뀐다. 1×로 오면 다시 그림.
7. 밤에는 어두워지고 모닥불 주변만 밝다. 겨울에는 바닥이 바래고 눈이 덮인다.
8. `assets/v8art/render_params.json`에서 벽 윗면 밝기 등을 바꾸고 F12를 누르면 바로 반영된다.
9. FPS가 1×에서도 ½×에서도 떨어지지 않는다.

# 15. 구현 완료 보고 (한국어; `docs/v8/m_art2/MART2_REPORT.md`와 최종 메시지)
```
## 구현 완료 보고 — M-ART2
### 구현 의도
### Part 0 기록 (ADR 0033 확장성 계약, 아트 채택 기록, M5.5 결정 기록)
### 가져오기 (자산 수·용량·출처 커밋·매니페스트)
### 브리지와 외모 (성별·재료 열, 외모 함수, 텍스트 렌더러 일치)
### 렌더러 (바닥 청크·경계·물·벽 재료 해석·언덕 면·사람·변형 틴트와 피부 마스크 수정·동물·물건·LOD·빛과 계절·설정 파일)
### 확장성 검사 (리터럴 grep, 데이터만으로 벽 추가 시험, 해석기 일치)
### 성능 (장면·줌별 FPS, 청크 재굽기 시간, 텍스처 메모리, 틱 비용 불변)
### 화면 비교 (옛 렌더러 vs 새 렌더러, 장면별)
### 지운 것 (M1 시트 경로 등)
### 변경된 파일 목록 (커밋별)
### 리뷰 결과 (코드·화면)
### 확인된 제한사항
### 오너 결정 필요 (설정값 조정, N 방향, 상의 추가, 지붕 토글)
### Gate 결과
git ls-remote 결과
```
