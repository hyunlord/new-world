# GOAL: WorldSim — ART-POLISH2: (1) walls, floors and doors as a **data-driven material system** — a wall is a construction style × a simulation material, and adding a new era's wall (mud brick, fired brick, cut stone, concrete …) is a data entry plus a templated texture set, never new code; with wall tops that carry the material, oriented along the run. (2) Faces that differ: six portrait variants per look with matching sprite tints for skin and hair, chosen per person.

Repository `hyunlord/new-world`. Base: **`v8/art-polish` = `18ac947b45963ecb54e2c4a2b1b7692d13ecfafb`**. New branch **`v8/art-polish2`** (`v8-artpolish2(pN): …`).

Executed by **Codex (GPT Astra)** in `/Users/rexxa/work/new-world-art-prod` (image tool, Python). Only `art/**`, `docs/v8/artpolish2/**`. No game code, no merge. Target 6 hours, hard stop 8. **Generation budget ≤ 320 calls** (scorecards count).

## Why
- **Owner, 2026-10-24, on the dark caps:** "나무벽 질감 말고 나머지 검정 부분은 나무벽이랑 너무 이질적이다." The dark band dominates because north–south runs show only their top (no face is visible from the south), and that large area lost its material. STRUCT's full-tile wood grain made them read as decks because the grain ran east–west on every run. The fix: the top shows the same material **seen from above, running along the wall**, slightly darker than the face, with ink edges — logs lying north–south on a north–south wall.
- **Owner, same message — the structural requirement:** this must not be a wood-only trick. Stone, mud brick, fired brick, concrete — walls of every era — must go through the same code and the same design structure. The simulation already stores a wall's material as a `MaterialId` with a `MaterialCategory` (Stone, Wood, Animal, Mineral, Plant, Mod; `TileGrid.wall_material`, set by `BlueprintKind::{LogWall, StoneWall}` in `sim-systems/src/runtime/jobs/space.rs`); the art must key on what the simulation knows.
- **Owner, on portraits:** "너무 비슷한 얼굴이 많다." There is one portrait per look (34), all generated from I-1's identity; a colony of 60 shows the same face over and over. Six variants per look, varying skin, hair colour, age and features, with the sprite tinted to match, gives 204 faces and lets elders exist.

# 0. RULES
DECISIONS in `docs/v8/artpolish2/POLISH2_DECISIONS.md`; data `/Users/rexxa/work/artpolish2-data/`; ≤ 1.5 MB per committed file; no pixel surgery; code-drawn compositing and **masked tone transforms** (Lab shifts inside a detected region) are allowed, parameterized, tested, deterministic. Style judges report only for portraits (ENV1/POLISH D1); the blind tests below gate. Drop order past the hard stop: the fourth wall style (§2.4) → cap variant (c) → children's portrait variants beyond 3. **Never drop:** the material schema with the resolver and the "add a style by data only" proof, log and drystone tops oriented along the run, the owner strip, the portrait pool for adults with sprite tints and both blind tests, the report.

# 1. PART 0 — Record
Commit this goal as `docs/v8/artpolish2/POLISH2_GOAL.md`; D1 quotes both owner messages; D2 records the cap brightness decision (×0.45 dark caps superseded by material tops; the brightness parameter survives as `top_value`).

# 2. PART 1 — The material system (`art/materials/`)

## 2.1 Schema (JSON, versioned, validated by a schema test)
- `construction_styles.json` — one entry per **way of building** (independent of material): `id` (`log`, `drystone`, later `mud_brick`, `fired_brick`, `cut_stone`, `timber_frame`, `concrete` …), `kind` (`wall` | `floor` | `door`), `era` (informational), `textures` (paths of the texture set, below), `top` (`{ value: 0.75, saturation: 0.85, blur_px: 2, ink_edge_px: 2, width_px: 64, side_shadow: 0.0, orientation: "along_run" }`), `face` (`{ height_px: 44 }`), `ends` and `corner` pieces, `prompt_template` (id), `sim_blueprints` (the `BlueprintKind` names that produce it: `LogWall` → `log`, `StoneWall` → `drystone`).
- `material_looks.json` — one entry per **simulation material** the art knows by name (`oak`, `pine`, `birch`, `granite`, `limestone`, `sandstone`, `basalt`, …; read the names from `rust/crates/sim-core/data/material/**` at `v8/main`): `category` (the sim's `MaterialCategory`), `tint` (`{ dL, da, db }` in Lab), optional `texture_override` per style.
- `category_defaults.json` — per `MaterialCategory`: the default style for an unknown blueprint and a default tint, so an unlisted material still renders.
- **Resolver** (`art/materials/resolve.py`): `(style_id, material_id) → texture set + tint + parameters`, falling back material → category default → style default; pure function, unit-tested, no style or material names inside the code.

## 2.2 Texture set per style (the template)
For a wall style: (1) **face strip** 256×44, seamless horizontally; (2) **top strip, along east–west** 256×64, the material seen from above as built (logs lying east–west; stones laid in a course), seamless horizontally; (3) **top strip, along north–south** 64×256, seamless vertically (generated, not rotated — the light stays upper left); (4) **end piece** for face ends; (5) **corner/junction top** 64×64 (log ends crossing; a corner stone); (6) **door** closed/open in wall height. Floors: one seamless 256×256 top texture. A prompt template per kind in `art/materials/prompts/` with slots `{style_description}`, `{material}`, `{era}`; every texture passes the seam instrument (STRUCT's numeric join test) and `projection_check.py` where applicable; ≤ 3 tries each.

## 2.3 Assembly (`art/compositor/walls.py`, rewritten to read only the schema)
- Every wall tile draws its **top** from the run direction: a tile whose wall neighbours are east/west uses the E–W strip (offset by world x); north/south uses the N–S strip (offset by world y); a corner, T or cross uses the junction top; an isolated tile uses the junction top. Tops are tinted by the material and transformed by `top.value/saturation/blur`, with `ink_edge_px` on edges that border non-wall tiles.
- Faces exactly as STRUCT (south side only where the south neighbour is not a wall), end pieces only at face ends.
- No style or material is mentioned in the assembly code; a test greps for literals.

## 2.4 Proof that new eras need no code
- Implement `log` (oak, pine, birch by tint) and `drystone` (granite, limestone, sandstone by tint) fully.
- Add **`mud_brick`** (Neolithic) **through data and generation only**: fill the template, generate its set, add the JSON entries; the commit adding it must not touch any `.py` file (a test asserts the diff's file list). Render it.
- (Droppable) a fourth style `fired_brick` the same way.
- `art/materials/ADDING_A_STYLE.md`: the procedure in six steps (template → generate → seam/projection checks → JSON → render the test layouts → owner strip), and the same for a new material (a tint entry, or a texture override).

## 2.5 Owner strip
`art/boards/walls_material_strip.png` at 2× and 1×: the house and storehouse scene with tops (a) dark ×0.45 (POLISH), (b) material tops along the run, value 0.75, (c) material tops at width 44 px centred with a soft side shadow; plus a row showing the same small room in log-oak, log-pine, drystone-granite, drystone-sandstone, mud brick (and fired brick if done).

# 3. PART 2 — Faces that differ
## 3.1 Variation plan (fixed before generation; `art/character/FACE_VARIANTS.md`)
Axes: skin tone S1 light / S2 mid / S3 deep; hair colour H1 black / H2 dark brown / H3 auburn / H4 grey (older adults only); age band young / adult / older (lines, greying); features (one or two of: freckles, a scar, heavy brows, broad nose, narrow face, round face); men's beard full / short / clean. Six variants per adult look, assigned by a fixed table that covers all three skins, at least three hair colours, two older variants (one H4), and three beard types for men; six per child look covering three skins and three hair colours (no H4, no beard). Each variant gets an id `v1..v6`.

## 3.2 Sprite tints
- **Masks per route I frame:** skin and hair regions by Lab clustering seeded at the look's known skin/hair colours, constrained to the body/head box of TEMPLATE_V3; stored as PNG per frame; checks: skin mask covers ≥ 60 % of the face box and both forearm boxes in idle S; hair mask lies inside the head box ± 4 px and covers ≥ 50 % of the hair silhouette; no mask pixel on the prop or clothing clusters (sample audit of 40 frames, recorded).
- **Tint:** a Lab shift of the masked pixels to the variant's target skin / hair colour (targets from `palette_05.json`, added if missing); alpha and outline pixels untouched; all 10 frames of a look use the same transform. Output: either baked frames per variant (`route_i/<look>/<variant>/…`) or a per-variant transform record for M-ART2's shader — produce both: baked for the boards, the record for the game.

## 3.3 Portraits
Generate the variant portraits: 17 adult looks × 6 (the 2 MISSING adult looks use their fallbacks' variants) + 18 child looks × 6, ≤ 2 tries each, attaching the look's `S/idle.png` **tinted to the variant**, the variant's description, and `anchors/portrait.png` for style. The ENV1/POLISH portrait of each look may serve as one variant if it matches the plan. Structure check as POLISH.

## 3.4 Blind tests (gating)
- **Distinctness by description:** for 8 looks (fixed rule: 4 adult, 4 child, both sexes), reviewer A writes one line per variant (6 portraits shuffled); reviewer B matches A's six lines to the six portraits. Bar: ≥ 5/6 matched for every look (then swap roles once).
- **Identity with tints:** for 36 variants (fixed sample covering every axis), show the portrait beside four tinted idle sprites at 2× (the right one; same look other variants ×2; another look ×1). Bar ≥ 32/36 per reviewer.
- Miss → one regeneration round of the failing variants (within tries), two new reviewers.
- **Person → variant rule** for M-ART2 (documented, implemented in the compositor): `variant = v[(agent_id × 2654435761) mod 2^32 mod 6]`, elders (age ≥ 50) pick among their look's older variants by the same hash, children among the child variants.

# 4. PART 3 — Boards and report
- Boards: `walls_material_strip.png`; `faces_owner.png` (8 looks × 6 variants: portrait at 128 + tinted sprite at 2×); `crowd_faces.png` (the ENV1 day scene at 1× with tints applied per agent id — the colony no longer looks cloned).
- `docs/v8/artpolish2/POLISH2_REPORT.md` (Korean): 판정(오너 판단 대기) · 재료 체계(스키마, 해석기, 폴백, 진흙벽돌로 증명한 "데이터만으로 추가") · 벽 윗면 방향 규칙과 세 변형 · 얼굴 변형 계획 · 마스크 품질 · 초상 생성과 두 맹검 시험 · 보드 · 변경된 파일 · 제한사항 · **M-ART2에 넘길 것**(스키마 파일과 해석 규칙, Godot에서 읽을 형식, 사람→변형 규칙, 틴트 기록) · 오너 결정 필요(윗면 a/b/c, 얼굴 변형 채택). Push; `git ls-remote origin v8/art-polish2`.
