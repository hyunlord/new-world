# GOAL: WorldSim — ART-TREES: trees as data — summer-green species in the single projection, bare winter forms for deciduous trees, seasons by code tint on the canopy only, a canopy-mass texture for the zoomed-out map, and a species schema keyed by the simulation's wood material names so that a new species is a data entry, never code.

Repository `hyunlord/new-world`. Base: **`v8/art-polish2` = `c8ed86087c90ed5429716713e930cc5f6a9e1332`** (the art line). New branch **`v8/art-trees`** (`v8-arttrees(pN): …`).

Executed by **Codex (GPT Astra)** in `/Users/rexxa/work/new-world-art-prod` (image tool, Python). Only `art/**`, `docs/v8/arttrees/**`. No game code, no merge. Target 4 hours, hard stop 6. **Generation budget ≤ 120 calls.**

## Why
In the game (M-ART2, `v8/main` `d796fcac`; D19 on `v8/wip` `a1c20220`) the forests read as ochre, dead-looking masses; at ¼× the grass is green and the forest yellow, so forest and meadow read inverted. The renderer does not tint them — the TRIAL2b 05 tree sources (`art/style05/env/plants/{oak,birch,pine}_{large,small}.png`) are autumnal yellow-green. Seasons must be a code transform on a summer-green canonical, not baked into the source. And under ADR 0033 (extensibility contract, owner directive 2026-10-24) trees must be a data family: the simulation has no species yet (every log is `oak`; the renderer picks a species per tile by a hash in `assets/v8art/bindings.json`), and M-EXT will give trees species whose names are the simulation's wood materials (`oak`, `pine`, `birch` in `rust/crates/sim-core/data/material/**`), which then tint the logs and log walls through the material system. The art must be keyed by those names now.

# 0. RULES
DECISIONS in `docs/v8/arttrees/TREES_DECISIONS.md`; data `/Users/rexxa/work/arttrees-data/`; ≤ 1.5 MB per committed file; no pixel surgery; masked tone transforms allowed, parameterized, tested. `art/style05/PROJECTION.md` applies (front 3/4, base anchor on the tile's lower third, light upper left). Style judges: judge v2 tree family **reports**; it gates only if it rejects all three negative controls in a calibration run first (record the run; ENV1/POLISH D1 precedent). Drop order past the hard stop: bush variants → second shape variant per size → spring tint. **Never drop:** the species schema and resolver with the data-only proof, summer canopies for oak/pine/birch in two sizes, bare winter forms for oak and birch, canopy masks, the season tints, the canopy-mass texture, the owner boards, the report.

# 1. PART 0 — Schema (`art/flora/`)
1. Commit this goal as `docs/v8/arttrees/TREES_GOAL.md`.
2. `species.json` (+ JSON schema): per species `id` (= the sim wood material name), `habit` (`deciduous` | `evergreen` | `shrub`), `sizes` {`small`, `large`} each with `variants` (paths, anchor, canvas), `bare` variants (deciduous only), `stump`, `canopy_mask` paths, `seasons` (tint records, below), `map_canopy` (the canopy-mass texture id for ¼×), `log_material` (the material id whose tint logs and walls use — equals `id` for trees).
3. `habit_defaults.json`: per habit the default seasons and a fallback species, so an unknown species id renders as its habit's default.
4. `resolve_flora.py`: `(species_id, size, season, variant_hash) → frame path + mask + tint`, pure, no species names in code (grep test, as the material system).
5. `ADDING_A_SPECIES.md`: template prompt → generate summer + bare (if deciduous) → masks → checks → JSON → render the test strip.

# 2. PART 1 — Generation
- **Summer canonicals:** oak, pine, birch × large (2–3 tiles tall) and small (1–1.5 tiles) × 2 shape variants = 12 sprites, front 3/4, summer greens from `palette_05.json` (add `foliage_summer_*` entries if missing, documented), trunk and roots visible at the base, a soft canopy shadow drawn by code (not in the sprite). Attach a route I sheet for scale and the hut/structure anchors for angle. ≤ 3 tries each.
- **Bare winter forms:** oak and birch × large/small × 1 variant = 4 sprites (branches only, same trunk silhouette as the summer form's variant 1 — attach it as reference). Pine is evergreen (no bare form).
- **Bushes (droppable):** berry bush full/bare re-checked against the projection rule; regenerate only if plan view.
- **Stumps:** per species, small, base-anchored.
- **Canopy-mass texture** for ¼×: a seamless 256×256 top-down forest canopy (summer green) in the 05 wash; the renderer will bake it into chunks where trees stand dense and tint it per season.
- Checks per sprite: `projection_check.py`, structure (single component, no clipping), and a **summer-green check**: mean canopy hue within the palette's foliage range and canopy saturation ≥ 0.30.

# 3. PART 2 — Masks and seasons
- **Canopy masks** per sprite: foliage pixels by Lab clustering against the foliage palette, excluding trunk/branch/ink clusters; check: trunk box (lowest 30 % of height, central 40 % of width) holds ≤ 2 % mask pixels; ≥ 85 % of foliage-cluster pixels covered.
- **Season tints** (Lab shifts on the masked canopy only; trunk, ink and alpha untouched), per habit with per-species overrides:
  - spring: lighter, yellower green (ΔL +6, Δb +6);
  - summer: identity;
  - autumn: deciduous → ochre/rust (oak toward ochre-brown, birch toward yellow), evergreen unchanged;
  - winter: deciduous → swap to the bare sprite; evergreen → darker (ΔL −8) plus a **snow cap** drawn by code on the upper canopy edge (mask erosion from the top, white at ≤ 70 % alpha).
  Season records live in `species.json`; the compositor applies them; the same records go to M-ART2.1 for the game shader.
- **Data-only proof:** a test-only species `test_alder` reusing oak's textures with its own season overrides renders in all four seasons with no `.py` change (assert the commit's file list).

# 4. PART 3 — Boards and report
- `art/boards/trees_owner.png`: each species × size × variant in four seasons at 1× and 2× on calm meadow, beside a route I person for scale.
- `art/boards/trees_scene.png`: the ENV1 day-6 scene re-composited with the new trees (summer) at 1× and ¼× (canopy mass at ¼×), next to the M-ART2 in-game screenshot of the same scene (`docs/images/mart2_compare_sheet.png` crops on `v8/main`) — so the owner sees ochre vs green, forest vs meadow at ¼×.
- `art/boards/trees_seasons_scene.png`: the same scene at 1× in spring, summer, autumn, winter.
- `docs/v8/arttrees/TREES_REPORT.md` (Korean): 판정(오너 판단 대기) · 스키마와 해석기(종 = 시뮬레이션 나무 재료 이름) · 생성(종·크기·변형별 시도와 검사) · 앙상한 겨울 형태 · 수관 마스크 품질 · 계절 틴트 표 · ¼× 수관 덩어리 · 데이터만으로 종 추가 증명 · 보드 · 변경된 파일 · 제한사항 · **M-ART2.1에 넘길 것**(가져올 경로 목록, species.json 읽는 법, 계절 셰이더 입력) · **M-EXT에 넘길 것**(시뮬레이션에 나무 종을 넣으면 통나무 재료와 벽 색이 이어지는 경로) · 오너 결정 필요. Push; `git ls-remote origin v8/art-trees`.
