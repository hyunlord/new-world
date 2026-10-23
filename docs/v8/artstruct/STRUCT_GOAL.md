# GOAL: WorldSim — ART-STRUCT: one projection. The ground is seen from above; everything with height — walls, doors, trees, rocks, workshops, beds, people, animals — is seen from the same front 3/4 as the adopted people. Walls, hill faces, water and terrain edges are drawn by code from tileable 05 textures, not generated case by case. Re-composite the ENV1 scenes and put the result before the owner.

Repository `hyunlord/new-world`. Base: **`v8/art-env1` = `429a18000266483ee40d756840d68b08840313c5`**. New branch **`v8/art-struct`** (`v8-artstruct(pN): …`).

Executed by **Codex (GPT Astra)** in `/Users/rexxa/work/new-world-art-prod` (image tool, Python, Godot 4.6 for FPS). Only `art/**`, `docs/v8/artstruct/**`. No game code. No merge. Target 6 hours, hard stop 8. **Generation budget ≤ 120 calls, ≤ 3 tries per asset.**

## Why
The owner on ENV1's renders (2026-10-23): "다른 건 그럭저럭인데 구조물들이 너무 부자연스럽다." The cause is a mixed projection: people are front 3/4 (~20° down) while the lean-to/hut is a roof plan on four posts (reads as a table, lower than the people beside it), the log wall is a flat line with rounded bends (reads as a hose), bedrolls are rugs, dug rock and river are square blocks, and the wood bench alone is a side view — three projections in one frame. RimWorld and Dwarf Fortress look natural because they keep one rule: the floor from above, everything that stands up from the same front angle, walls as blocks with a cap and a face, no roofs drawn, and walls/edges assembled by code from linked textures. ENV1's 246/256 wall seam failures show the same lesson from the other side: 16 generated neighbour cases cannot be made to join; one tileable texture cut by code always joins.

# 0. RULES
DECISIONS in `docs/v8/artstruct/STRUCT_DECISIONS.md`; data `/Users/rexxa/work/artstruct-data/`; ≤ 1.5 MB per committed file; no pixel surgery on generated art, no quantization; code-drawn elements (cuts, masks, blends, tints) are allowed and are the point of this pass — every one is a function of declared parameters, tested, deterministic. Reuse the ENV1 compositor (`art/compositor/`) and its real scene sources; do not rebuild them. Drop order past the hard stop: wolf retry → drought render → items re-angle → ¼×. **Never drop:** the projection rule, the wall system, hill faces, water/terrain edges, the re-angled workshops/beds/hearth/rocks, the hut as walls, the re-composited scenes and the owner boards, the report. The owner's eye is the verdict this time; reviewers are not used (ENV1's per-cell reviewer bar measured noise).

# 1. PART 0 — The projection rule (`art/style05/PROJECTION.md`)
1. Commit this goal as `docs/v8/artstruct/STRUCT_GOAL.md`.
2. Write the rule with numbers (64-px tile at 1×):
   - **Ground plane** (grass, forest floor, dirt, floors, water, zones, marks): straight top-down, tiles exactly 64×64.
   - **Standing things:** front 3/4 matching route I (~20° down, light from upper left); the base sits on the tile's bottom third, foot/base anchor at `(tile_left + 32, tile_top + 43)` as HANDOVER fixes for people; sprites may extend upward beyond the tile; depth sort by base y, then x.
   - **Heights** (1×): wall block = cap 16 px + face 44 px (a wall reads about as tall as a person's 56 px); door = wall height; bedroll 14 px high, 1×2 tiles; workbenches 30–40 px; drying rack 48 px; hearth 20 px with flame above; boulder 24–36 px; flint outcrop 20 px; clay bank 16 px; trees 2–3 tiles tall (existing 05 trees: check against the rule, re-angle only if they read as plan view); people 56 / children 40; deer 40 shoulder, wolf 28.
   - **Roofs are not drawn.** Buildings show their walls, door, floor and contents. A roof overlay (toggle) may be drawn at 35 % alpha for debugging only.
   - **Occlusion:** a wall's face covers at most the 44 px above its own tile row — people standing directly north of a south wall are partly hidden, like RimWorld; never more.
3. A `projection_check.py`: for a standing sprite, the base row (lowest alpha ≥ 128) is within ±2 px of its anchor; height within the table's band; light direction estimate (mean luminance gradient of the opaque region) points from upper left (report the angle).

# 2. PART 1 — Walls by code (`art/compositor/walls.py`)
- **Textures to generate per material (log, stone; mud brick droppable):** (a) **face strip** 256×44, seamless horizontally (prompt says so; verify), front 3/4 of the wall's south face; (b) **cap strip** 256×16, seamless horizontally, the wall's top seen from above-front; (c) **end piece** 24×60 (the cut end of a run: log ends / stone edge); (d) **inner corner cap** 16×16. ≤ 3 tries each.
- **Tileability instrument (no reviewers):** tile the strip 4× and measure, at each join column, the mean absolute RGB difference between the two columns adjacent to the join, against the 95th percentile of the same statistic between adjacent interior columns; pass when join ≤ 1.15 × interior p95 at every join. Record numbers.
- **Assembly rule:** every wall tile draws the cap (cut from the cap strip at offset = world x × 64 mod 256, so neighbours continue the texture); a wall tile draws the face **only if its south neighbour is not a wall**; a run's ends get end pieces where the east/west neighbour is not a wall; T and cross junctions draw caps only; corners join because caps are continuous. **Doors**: a closed slab and an open frame generated as standing sprites in wall height (≤ 3 tries each, per material), placed in the wall run. Unit tests on synthetic layouts (straight, L, U, T, room with door) render without gaps, by alpha coverage of the expected cells.
- **Hill faces with the same rule:** an undug hill tile whose south neighbour is passable (dug floor, plain, floor) draws a **rock face** (generated rock face strip 256×44, rock cap strip from the hill texture) — so a dug pocket reads as a cave cut into the hill. Undug hill interior stays top-down rock.
- **The hut (3×3 building)** renders as a log-wall ring with a door and a packed-earth floor through this system; the lean-to (shelter) as a 3/4 standing sprite (regenerate, ≤ 3 tries) that reads as a lean-to at person scale.

# 3. PART 2 — Water and terrain edges by code (`art/compositor/edges.py`)
- **Water:** a seamless 256×256 calm water texture in 05 (≤ 3 tries); river tiles sample it with world offsets; a **bank mask** drawn by code: for each water tile edge touching land, a noise-dithered alpha falloff of 10 px with a 2-px ink edge line on the land side; fords: the water texture with stepping-stone sprites (generate 1 sheet of 4 stones, ≤ 3 tries) placed by code.
- **Ground transitions:** between different ground textures (meadow/forest floor/dirt/packed earth/rock floor), a noise-dithered blend of 12 px from the edge using a fixed-seed value noise; corner cases by the max of the two edge masks. Floors (packed earth, dug rock) inside rooms keep crisp edges against walls (no blend under a wall).
- **Zones:** the stockpile zone becomes a faint tint (8 % of the zone colour) with small corner ticks; no grid lines (ENV1's grid read as a floor).
- Tests: deterministic output for a fixed scene; no visible square edge (the rendered luminance along any terrain boundary has no straight run of identical-step change longer than 24 px — measure).

# 4. PART 3 — Re-angled standing objects
Regenerate in front 3/4 with **an accepted route I sheet attached as angle and scale reference** and the 05 anchors: knapping spot, wood bench, drying rack (empty, with a hide), pottery bench, grinding stone, hearth (unlit, lit, firing with pots), bedroll (empty, occupied is a person sprite on it), boulder (large, small), flint outcrop, clay bank (full, depleted), stump, carcass (if not from the animal sheet). ≤ 3 tries each; accept = judge v2 family pass + `projection_check.py` pass + your one-line reading. Existing items (`env/items`, ENV1 items) are re-checked with the projection rule and re-angled only where they read as plan view (droppable). **Wolf retry (droppable):** one sheet, ≤ 3 tries, fixing the pose ENV1 failed.

# 5. PART 4 — Re-composite and the owner boards
- Re-render ENV1's three real scenes (day 6 noon, day 6 02:00, winter noon) at 1× and ½× (LOD as ENV1) with the new walls, edges, water, zones and objects; same declared overlay (house, storehouse, dug pocket, workshops, animals) so the comparison is like for like.
- **Owner board 1** `art/boards/struct_owner_board.png`: day 6 noon, 1× — current renderer · ENV1 · STRUCT, side by side, same crop.
- **Owner board 2** `struct_closeups_2x.png`: at 2× — the walled house with door and bedrolls and people inside; the stone storehouse; the dug pocket in the hill; the river with the ford; a workshop corner with three workers.
- **Owner board 3** `struct_conditions.png`: STRUCT at night and winter, 1× and ½× LOD.
- **Crowd FPS** in Godot: 200 people + 600 environment sprites + code-drawn walls for 3 rooms at 1×, 60 s ≥ 60 FPS (the wall system ported as a Godot draw routine or baked per chunk — document which).
- No reviewer test; the report presents and does not claim approval.

# 6. PART 5 — Report
`docs/v8/artstruct/STRUCT_REPORT.md` (Korean): 판정(오너 판단 대기) · 투영 규칙(수치 표) · 벽 시스템(텍스처별 이음새 수치, 조립 규칙, 테스트) · 언덕 면 · 물과 지형 경계 · 다시 그린 구조물(가족별 시도·수락·투영 검사 수치) · 늑대 재시도 · 합성 결과와 세 보드 · FPS · 변경된 파일 · 제한사항 · **M-ART2에 넘길 것**(게임 렌더러가 구현할 함수 목록: 벽 조립, 언덕 면, 물/경계 블렌드, 구역 틴트, 깊이 정렬, LOD; 자산 경로 표) · 오너 결정 필요(벽 높이 체감, 지붕 토글 기본값, 밤 밝기). Push; `git ls-remote origin v8/art-struct`.
