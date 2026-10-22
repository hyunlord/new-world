# GOAL: WorldSim — M3: space — walls, doors and floors you build; hills you dig into; rooms the simulation recognizes and gives a role; and the pathfinding that walls make necessary. Plus the M2 follow-ups (long evaluation scenario, skill tie-break, quality words, economy soak in the gate) and the art record brought up to date.

Repository `hyunlord/new-world`. Base: **`v8/wip` = `f0ba4347cdd5886a58c081464dd322e10b82f538`** (M2 + the capture fix; `v8/main` = `4874c9dc`, gate green at `c0af2eb7`).

Executed by **Claude Code**, unattended, on the Mac in `/Users/rexxa/work/new-world-pass4`. No DGX. Headless except 12.3. Target 18 hours, hard stop 22. The art session may run ART-TRIAL3b on `v8/art-trial3` in parallel; do not touch `art/**`, `docs/v8/arttrial*/**`, do not merge art branches.

## Why this pass exists

ROADMAP M3: "방 인식과 역할, 언덕 파기 — 언덕을 파서 만든 방이 창고가 되고, 사람들이 그리로 나른다." It is the first of V7's three signature features (RimWorld-style space: walls, roof and furniture define a room's role, and the role changes behaviour) and the owner's "변태적 디테일" lives here. Today the colony has no wall, no door, no floor, no roof, no room, no dug tile, and **no pathfinding**: agents step by sign toward a target and ignore passability except water and mountain (`movement.rs`, `fission.rs::walkable`). The moment a wall exists, that must change, so M3 carries the pathfinder (ADR 0021) as its foundation.

`TileGrid` already has `wall_material`, `floor_material`, `terrain_type`, elevation and outdoor flags (V7 Material Phase 2 data), unused by the colony. M3 connects them to items (ADR 0019): a log wall costs logs, a stone wall costs stone, a dug hill tile yields stone.

M2's lessons are carried: the evaluation scenario runs **20 days of continuous work**, not one day; priorities break ties by skill; quality is shown as a word and a number; a 20-day economy soak becomes a gate fixture.

---

# 0. RULES
- AGENTS.md and the skills (`worldsim-architecture` first: C1–C5, the index, the seam, "add a command / job type / item kind"). Commit to `v8/wip` (`v8-m3(pN): …`), `v8/main` by fast-forward after a green gate. Never `lead/main`, `main`, `codex/*`, `v8/art-*`; no force-push; explicit paths; hook and ignore files untouched.
- **Never ask the owner** → `docs/v8/m3/M3_DECISIONS.md`.
- C3 ceilings and the regression rule are hard (gate `bench`). Pathfinding is the pass's performance risk: budget it from the start (§4).
- Parts: 0 = §5 (records + M2 follow-ups) · 1 = §6 (pathfinding) · 2 = §7 (walls, doors, floors, digging) · 3 = §8 (rooms and roles) · 4 = §9 (UI) · 5 = §10 (evaluation) · 6 = §11 (docs, review, gate, report). Push after every Part.
- **Time rule:** past 22 h, finish the step, push, go to Part 6.
- **Drop order:** 8.5 workshop-room bonus → 7.5 stone wall (keep log wall) → 8.4 room "why?" lines → 6.4 flow-field cache (keep plain A* with the memo) → 9.5 room overlay polish (keep the toggle).
- **Never drop:** Part 0, the pathfinder with its budget test, log wall + door + packed-earth floor, hill digging, room detection with bedroom and storehouse roles, beds and hauling that use rooms, the long evaluation, the soak fixture, identity, docs, report.

# 1. MACHINE
Pass4 checkout + base worktree `../new-world-m3-base` at `f0ba4347`. Data `/Users/rexxa/work/m3-data/`. `CARGO_BUILD_JOBS=8`, tests `--test-threads=8`, probe `--workers 6`.

# 2. CONTEXT (planner's survey at `4874c9dc`; paths under `rust/crates/`; verify)
- **Tiles:** `sim-core/src/tile/grid.rs` `TileGrid { width, height, wall_material, floor_material, ground_resource, terrain_type, elevation?, outdoor? … }` (SoA). `TerrainType { Plain, Forest, Mountain, Hill, River, Coast, Desert, … }`. Walkability today: `!(River | Mountain)` in two local helpers; `AgentMovementSystem` (120) steps by signum every `move_interval_ticks` (colony 3) with no passability test.
- **Buildings:** `BuildingKind { Hearth, Shelter, Stockpile, Hut }`; huts give 4 beds at the centre tile; `BedAssignmentSystem` (145) assigns households to beds via `SimResources.bed_of`; workshops (M2) are not V7 buildings (D22).
- **Colony data (M1–M2):** items with material/quality, `MaterialNeeds`, `BlueprintKind { Hut, KnappingSpot, WoodBench, DryingRack }` with `plan()`, zones, designations (`Chop`, `Quarry`), `ReserveKey`, commands in the log, `ChunkIndex` spatial index, jobs (`Build, Deliver, Craft, Quarry, Chop, Gather, Haul`), `WorkPriorities`, `Skills`.
- **Influence:** `sim-core/src/influence/{blocking,channel,grid,propagate}.rs` and `runtime/influence/*` (warmth/danger fields; `blocking.rs` reads wall materials) — live in both worlds; verify what a wall tile does to them today.
- **Firewood/cold:** `FireSystem` (147) applies cold at night to people not in a shelter/hut; `ExposureRule`; M1.5 calibrated winter prep.
- **Evaluation tooling:** `worldsim_probe --scene colony --commands … --days N`; M2-U script `tools/v8/m2_u_script.json`; `m2_score.py`; `bench`; the ledger.
- **UI:** F3 build menu (1–4), F7 quarry, workshop panel, P priorities, Space pause, F5/F9 save/load; inspector; overlay layer; `gen_tiles.py` temporary sprites.
- **Art state to record (§5.1):** ART-TRIAL2 FAIL (034c38ab) → TRIAL2b withdrew the 02 fallback: environment 35/38 at 2.0 tries (2433a612); TRIAL3 (352f4101): template + layer contract + structural checker (87/87, 603 combos), route A generated people structurally sound but style 2.5/5, route B blocked by tool login; recommendation: a 2D artist against the template, or route B if the tool works; TRIAL3b (route B retry) pending.

# 3. DESIGN

## 3.1 Pathfinding (ADR 0021)
- **Passability** per tile, computed from `TileGrid` + colony data, cached as a `Vec<u8>` cost map in `SimResources.nav` with a `nav_version`: impassable = River, Mountain, **undug Hill**, wall tile without a door, a construction site of a wall; cost 10 for plain/forest/floor, 14 diagonal, 12 for a door tile, 30 for shallow water bank (if present). Updated **incrementally** by the code that changes a tile (wall built/removed, door, dig complete, site placed/cancelled): set the cell, bump the version.
- **A\*** over 8-neighbours with Chebyshev heuristic, integer costs, deterministic tie-break (f, then h, then `(y, x)`); corner cutting forbidden past impassable cells. Implementation in `sim-core/src/nav.rs` (pure, tested), used through `SimResources.nav.path(from, to) -> Option<Vec<(u32,u32)>>`.
- **Memo:** `BTreeMap<(from_chunk, to), Rc<Path>>`-style cache keyed by `(nav_version, from, to)` with a bounded size (4,096 entries, LRU by insertion order), cleared when `nav_version` changes. Agents store their path in `SeekTarget { tile, path: Vec<(u32,u32)>, path_version }` and re-plan when the version changes or the next step is blocked.
- **Movement:** `AgentMovementSystem` follows the path one step per move interval; baseline keeps the signum step (gate by `rules.colony`; identity). Unreachable target → the job is cancelled with `CancelReason::Unreachable` (counted) and the target gets a 1-day cooldown for that agent.
- **Budget (pre-registered):** at N = 1,000 with the M3 soak script (walls and rooms present), `AgentMovementSystem` + pathfinding ≤ 0.6 ms per tick mean; the whole tick stays under C3. A `nav_bench` in `worldsim_perf` reports path count, mean nodes expanded, cache hit rate.
- **Flow-field cache (droppable, 6.4):** for the 8 most-used destinations per day (stockpile zones, hearth, workshops) keep a flow field recomputed on `nav_version` change, used when the destination matches.

## 3.2 Walls, doors, floors, digging (ADR 0022)
```rust
pub enum BlueprintKind { Hut, KnappingSpot, WoodBench, DryingRack, LogWall, StoneWall, Door, PackedFloor }
```
| kind | footprint | needs | progress | result |
|---|---|---|---|---|
| LogWall | 1×1 | Log 2 | 40 | `wall_material = oak` (the log's material) |
| StoneWall (7.5, droppable) | 1×1 | Stone 3 | 60 | `wall_material = granite/limestone` |
| Door | 1×1 | Log 1 | 30 | wall tile with `door: true` (new `TileGrid.door: Vec<bool>`), passable cost 12, blocks room flood like a wall |
| PackedFloor | 1×1 | — (work only) | 15 | `floor_material = packed_earth` |

- Drag placement: `PlaceLine { kind, x0, y0, x1, y1 }` (Bresenham, walls/floors) and `PlaceRect { kind, … , hollow: bool }`. Each tile becomes a 1×1 site with its own `MaterialNeeds`; a wall site on a tile with a tree, node, item stack or person is refused (items must be hauled off first: a site placed on items creates a Haul-away job automatically).
- **Deconstruct** designation on walls/doors/floors: a `Deconstruct` job returns half the materials (rounded down) as items on the tile.
- **Digging:** designation `Dig` on Hill tiles adjacent (8-neighbourhood) to a passable tile; `JobKind::Dig { tile }` (`WorkType::Mine`, `dig_ticks` 150, skill `Mining`); completion: terrain stays Hill but `TileGrid.dug: Vec<bool>` is set, the tile becomes passable with `floor_material = rock`, and yields `Stone 3` of the hill's rock material (keyed). Undug Hill tiles around a dug pocket act as walls for rooms and for passability. A hill tile is never dug if it is on the map edge.
- Walls, doors, dug tiles and floors update the nav map incrementally and the influence blocking cache.

## 3.3 Rooms and roles (ADR 0023)
- **Detection** (`sim-core/src/rooms.rs`, `RoomSystem` priority 152, interval 60, and on demand after any wall/door/dig change via a dirty flag): flood fill over passable-or-floor tiles bounded by walls, doors and undug hill; a region is a **room** when it is enclosed (no path to the map edge or to a region larger than 400 tiles without crossing a door) and 2 ≤ area ≤ 120. Doors belong to no room. Rooms get stable ids (keyed by their lowest `(y, x)` tile; re-detected rooms keep the id when ≥ 50 % of tiles overlap). `SimResources.rooms: BTreeMap<RoomId, Room { tiles, doors, roofed: bool, role: RoomRole, dug: bool }>`; `room_of_tile`.
- **Roofed** = enclosed and area ≤ 120 (M3 has no roof item; the enclosure is the roof). Dug pockets are roofed by the hill.
- **Role** (auto, recomputed with detection; the player may **override** by command `SetRoomRole { room, role }`, recorded):
  - `Bedroom` if the room contains ≥ 1 bed item/tile and no workshop/stockpile; capacity = beds inside (a **Bedroll** 1×1 blueprint: Hide 1 or Log 1, progress 10, gives 1 bed — add it; hut beds remain);
  - `Storehouse` if ≥ 50 % of its tiles are stockpile-zone cells;
  - `Workshop` if it contains a workshop building;
  - `Hall` if it contains the hearth;
  - `Unassigned` otherwise.
- **Effects (the "role changes behaviour" part):**
  - Bedroom: `BedAssignmentSystem` prefers beds in a roofed bedroom; sleepers in a roofed room do not take night cold (`FireSystem` treats a roofed room like a hut); sleep recovers 20 % faster (integer: sleep decay ×800 ‰ while asleep in a roofed bedroom).
  - Storehouse: hauling prefers stockpile cells inside a roofed storehouse (score bonus equivalent to 16 tiles of distance), and `Maintain` bills count only covered stock when a storehouse exists (so the colony "wants" things indoors); items in a storehouse are flagged `covered` in the overlay.
  - Workshop room (8.5, droppable): craft work ticks ×900 ‰ when the workshop stands in a roofed workshop room.
  - Hall: the evening gathering uses the hearth's room tiles when it is a room (the existing hearth-circle logic picks tiles inside).
- **"Why?" lines (8.4, droppable):** the causal panel on a room tile shows "침실 · 지붕 있음 · 침대 4 · 추위 보호"; the chronicle logs `RoomFormed { role }` once per room.

## 3.4 M2 follow-ups (Part 0)
- **Skill tie-break:** within one priority level, `JobAssignSystem` orders agents by the work type's skill level descending before `AgentId` (D-record; M2 fixtures change → re-capture the M2 colony fixtures and say so).
- **Quality words:** inspector/overlay show `부싯돌 날 · 좋음 (62)`; five bands 0–19 조악 / 20–39 보통 / 40–59 좋음 / 60–79 훌륭 / 80–100 걸작 (keys in §9.6).
- **Economy soak fixture:** `tools/v8/m3_soak_script.json` — 20 days: chop marks renewed every 2 days (15 trees nearest the camp not yet marked), quarry 3 nodes, workshops, bills `Make 20` blades, `Make 4` axes, `Maintain 40` firewood, `Maintain 10` blades; plus M3's build orders (§10). Gate step `soak` runs it on 4 seeds and asserts: ledger exact, reservations exact, 0 deaths, determinism — every pass from now on.

---

# 4. PERFORMANCE PLAN (written before coding; recorded in DECISIONS)
Expected costs at N = 1,000: ≤ 200 path requests per 100 ticks; A* on 192² with walls ≤ 2,000 nodes typical; memo hit ≥ 60 % on hauling routes. The `nav_bench` is run after Part 1 and after Part 3; if the movement + nav share exceeds 0.6 ms, apply in order: (1) path reuse for same `(from_chunk, to)`; (2) the flow-field cache; (3) cap re-plans per tick (round-robin). Record each step's effect.

---

# 5. PART 0 — Records and follow-ups
1. Commit this goal as `docs/v8/m3/M3_GOAL.md`.
2. **Art record update** on the main line: ADR 0018 status note replaced by the §2 "art state" (TRIAL2b withdrawal, TRIAL3 template/contract/checker, A's style result, B pending, artist recommendation); ROADMAP rows for ART-TRIAL2b, ART-TRIAL3, ART-TRIAL3b (running); HISTORY lines; archive index.
3. ADR 0021 (pathfinding), 0022 (built space and digging), 0023 (rooms and roles) drafted with Context/Decision/Alternatives/Consequences; finalized in Part 6.
4. §3.4 follow-ups implemented, tested, M2 fixtures re-captured where the tie-break changes them (identity for baseline unaffected).
5. Base data: identity base, M2 colony fixtures, ledger base line (`bench`), the soak fixture's first run → `$DATA/base/`.
Push.

# 6. PART 1 — Pathfinding
§3.1 and §4. Tests: `nav_astar_basic` (known grid, known path), `nav_corner_cutting`, `nav_unreachable`, `nav_incremental_update` (build a wall, version bumps, path changes), `nav_memo_hit`, `movement_follows_path` (colony), baseline identity (movement unchanged there), `nav_bench` numbers in DECISIONS.

# 7. PART 2 — Walls, doors, floors, digging
§3.2. Tests: a 5×5 log-wall rectangle with a door is built from a stockpile of logs (ledger exact); a wall site on items creates a haul-away; deconstruct returns half; dig a 3×3 pocket into a hill (stone yield, passable after, ledger); nav updates after each; identity.

# 8. PART 3 — Rooms and roles
§3.3. Tests: the 5×5 enclosure with a door is a room; without the door it is still a room (enclosed) but unreachable → its beds are not assigned; a dug pocket is a room with `dug = true`; role auto-detection for each role; override command; sleepers in a bedroom take no night cold at −10 °C severity (use the exposure path); hauling prefers the storehouse (a log ends inside when both a covered and an open cell exist at equal distance); identity.

# 9. PART 4 — UI
1. F3 build menu grows: 5 통나무 벽, 6 문, 7 바닥, 8 침상 (and 9 돌담 if 7.5 kept); walls/floors by **drag line** (shift for rectangle, hollow by default); the ghost turns red on refused tiles with the reason toast.
2. **F8 파기** (drag over hill tiles), **F9 is already load → move 철거 to F10** (`Deconstruct` drag). Keep F5/F9 save/load. Update README's key table.
3. **Room overlay toggle `R`:** rooms tinted by role (침실 파랑, 창고 노랑, 작업장 청록, 홀 주황, 미정 회색) with a label at the room's centre (`{role} · {tiles}칸 · 침대 n`); click a room → a small panel: role (dropdown to override), tiles, beds, covered items; (9.5 droppable: door icons, roof hatching).
4. Overlay: wall/door/floor/dug tiles drawn (temporary sprites in `gen_tiles.py`: log wall 4 autotile variants by neighbours, door, packed floor, dug rock floor, bedroll); undug hill unchanged.
5. Inspector: "잠자리: 침실 3 (지붕 있음)" line; "방: 창고 2" on items.
6. Localization keys in `docs/v8/m3/M3_LOCALE.md` (≥ 40 expected: blueprint names, roles, room panel, dig/deconstruct orders, reasons, quality words, chronicle `RoomFormed`). `Locale.trf` for formats.

# 10. PART 5 — Evaluation (fresh seeds 400–431; calibration 0–7 ≤ 6 rounds)
**M3-U script** (resolved per seed at tick 360, then renewals): the soak economy of §3.4 **plus**: day 1 — a 7×5 log-wall house with a door south of the hearth, 4 bedrolls inside, packed floor; day 2 — a 6×6 stockpile zone enclosed by log walls with a door (storehouse); day 3 — a dig order for a 4×4 pocket into the nearest hill within 40 tiles (if no hill, a second walled storehouse). Run **20 days**.

| # | bar | pass |
|---|---|---|
| F1 | the house is finished and recognized as a Bedroom with 4 beds by day 8 | ≥ 28/32 |
| F2 | the storehouse is a Storehouse by day 10 and holds ≥ 60 % of all stockpiled items by day 20 | ≥ 28/32 |
| F3 | the hill pocket is dug by day 14 and is a room with `dug = true` (seeds with a hill) | ≥ 90 % of those seeds |
| F4 | ≥ 3 people sleep in the bedroom on every night from day 9 | ≥ 28/32 |
| F5 | pathing: 0 `Unreachable` cancels for targets inside rooms with doors; ≥ 1 route in each world passes through a door | 32/32 |
| F6 | nav budget (§3.1) at N = 1,000 soak | pass |
| F7 | skill growth (M2's E7 redone with the long script): Knap level of the top knapper at day 20 ≥ 5, and blade quality mean day 15–20 − day 1–5 ≥ 8 | ≥ 24/32 |
| F8 | axe use (M2's E2 redone): ≥ 10 Chop jobs with an axe after day 3 | ≥ 28/32 |
| F9 | ledger per kind exact; reservations; 0 deaths in 20 days; 3-year no-orders run still 0 cold deaths (≥ 30/32) | all |
| F10 | bench ceilings + regression; baseline identity; replay byte-identical | all |

# 11. PART 6 — Docs, review, gate, report
- Docs: CHANGELOG; ROADMAP (M3 done; 다음 → M-ART2 if TRIAL3b reported viable, else M4; recommend); HISTORY; ADR 0021–0023 final; ARCHITECTURE (system table, §2 movement, §9 items/jobs extended with space, nav and rooms data flow; schema v3); `snapshot_schema.md`; `PERF_LEDGER.md`; skills (`worldsim-architecture`: nav rules, incremental updates, "add a blueprint kind", "add a room role"; `worldsim-testing`: soak fixture; pitfalls).
- Review: two fresh subagents — code (nav determinism and memo invalidation; incremental passability correctness; room id stability; ledger across deconstruct/dig; seam line count ≤ +20 on `decision/system.rs`); numbers (F1, F2, F7, F8 recomputed from raw).
- 12.3 Screen checks (windowed): the room overlay with three roles labelled; a dug pocket; the build menu with a wall drag ghost; inspector lines. Shots to `$DATA/shots/`.
- Gate (now 27 steps with `soak` and `nav_bench`-report) green on a clean commit → fast-forward → report commit → docs step → fast-forward.

# 12. Not in M3
Roofs as items; windows; room temperature model; furniture beyond bedroll; deeper mining (L1+); multi-storey; pathfinding for animals; neighbours; the 64-px view.

# 13. 인게임 확인사항 (오너용)
1. F3-5 벽을 드래그해 네모를 그리고 F3-6 문을 하나 놓으면, 사람들이 통나무를 날라 벽을 세우고 문을 단다. 벽이 선 뒤에는 **사람들이 벽을 돌아서 문으로 다닌다.**
2. R 키를 누르면 방이 색으로 칠해지고 "침실 · 35칸 · 침대 4"처럼 이름이 붙는다. 침상을 넣으면 침실, 비축 구역을 넣으면 창고로 바뀐다.
3. 밤에 침실에서 자는 사람은 추위를 안 탄다. 바깥 움막보다 침실을 먼저 고른다.
4. 창고가 생기면 통나무·돌·장작이 **창고 안으로** 옮겨지고, 사람 상세에 "방: 창고"가 보인다.
5. F8로 언덕을 드래그하면 곡괭이질로 파 들어가고, 돌이 나오며, 판 자리가 방이 된다. 비축 구역을 그 안에 그리면 창고가 된다.
6. F10 철거로 벽을 헐면 통나무 절반이 돌아온다.
7. 품질이 "좋음 (62)"처럼 보인다. 같은 사람이 날을 20개쯤 만들면 품질이 눈에 띄게 오른다.
8. 20일을 돌려도 사람들이 벽에 끼거나 못 가는 곳을 가려다 멈추지 않는다.

# 14. 구현 완료 보고 (한국어; `docs/v8/m3/M3_REPORT.md`와 최종 메시지)
```
## 구현 완료 보고 — M3
### 구현 의도
### Part 0 기록 (아트 상태 갱신, ADR 초안, M2 후속 3건, 고정 테스트)
### 구현 내용 (경로 찾기, 벽·문·바닥·파기, 방·역할, UI — 파일별)
### 구현 방법 (A*와 메모·증분 갱신, 방 탐지와 id 안정성, 역할 판정과 효과 수치, 재료 흐름, 결정 틱 접점 줄 수)
### 성능 계획과 실제 (nav_bench 전후, 적용한 조치와 효과, 벤치 장부)
### 평가 결과 (F1–F10 표 + 시드별 부록)
### 보정 기록
### 변경된 파일 목록 (커밋별)
### 리뷰 결과
### 확인된 제한사항
### 오너 결정 필요 (다음: M-ART2 vs M4; 방 역할 자동/수동 기본; 지붕 모델)
### Gate 결과
git ls-remote 결과
```
