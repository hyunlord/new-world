# GOAL: WorldSim — M2: things and making (stone and flint as items, workshops, craft orders, work priorities, skills and quality, firewood through the job layer), with the art direction change recorded on the main line first

Repository `hyunlord/new-world`. Base: **`v8/main` = `v8/wip` = `e5594e3f661cdddffb28c70bc2e25562e5cd3479`** (M1.5, gate green at `cdd4a775`).

Executed by **Claude Code**, unattended, on the Mac in `/Users/rexxa/work/new-world-pass4`. No DGX. Headless except the screen checks in 11.3. Target 18 hours, hard stop 22.

**In parallel**, the art session runs ART-TRIAL2 on `v8/art-trial2` (only `art/**`, `docs/v8/arttrial2/**`). Do not touch those paths; do not merge that branch.

## Why this pass exists

ROADMAP M2: "재료 연결, 작업장, 제작 주문, 작업 우선순위 표 — 부싯돌을 주문하면 돌깎이가 만들고, 숙련도에 따라 품질이 다르다." After M1 and M1.5 the colony has one item (`Log`), three orders, one blueprint, jobs that replace leisure, a 24-day year and fertility. It has no second material, no workshop, no way to ask for a thing to be made, no way to say who does what, and no skill. The material data of V7 (`sim-core/src/material/`, 100+ RON definitions with 15 properties, `flint.ron`, `chert.ron`, `granite.ron` among them) is still unconnected to any runtime.

M2 adds the economic loop every later milestone builds on: **resource nodes → gathering jobs → items in stockpiles → craft orders at workshops → tools that change work → skills that change speed and quality.** It also takes the first real step of dismantling the 3,490-line decision tick (ADR 0016 C5): **firewood moves from the autonomous decision path into the job layer**, measured by tick share and line count.

Before any of that, Part 0 records on the main line what the art track decided this week, because the owner's rule is that direction, roadmap and work live in the repository's markdown, and the art decisions so far live only on side branches.

---

# 0. RULES
- AGENTS.md and the skills apply; read `worldsim-architecture` (C1–C5, the index, the seam), `worldsim-testing`, `worldsim-pitfalls`.
- Commit to `v8/wip`, messages `v8-m2(pN): …`; `v8/main` by fast-forward after a green gate. Never `lead/main`, `main`, `codex/*`, `v8/art-*`; no force-push; explicit paths; hook and ignore files untouched.
- **Never ask the owner.** Decisions in `docs/v8/m2/M2_DECISIONS.md`.
- **C3 is now hard:** gate step `bench` fails on a ceiling miss (N=1,000 tick mean ≤ 5 ms; N=2,000 ≤ 10 ms; ≤ 4 KiB/agent; ≤ 96 B/tile) and on > +10 % regression vs `docs/perf/PERF_LEDGER.md`'s last line.
- Parts: 0 = §4 (records) · 1 = §5 (items, nodes, data) · 2 = §6 (workshops, bills, recipes) · 3 = §7 (priorities, skills, quality, tools) · 4 = §8 (firewood through jobs; decision-tick reduction) · 5 = §9 (UI) · 6 = §10 (evaluation) · 7 = §11 (docs, review, gate, report). Push after every Part.
- **Time rule:** past 22 hours, finish the step, push, go to Part 7.
- **Drop order:** 7.4 tool durability → 9.4 bill-queue panel polish (keep a minimal list) → 8.3 the second decision-tick extraction (foraging stays) → 6.4 the drying rack and hide (keep knapping + woodworking only).
- **Never drop:** Part 0, stone/flint nodes and items, the knapping spot with its bill, the priority table and its UI, skills with the quality draw, firewood through jobs (8.1–8.2), baseline identity, the evaluation, docs and the report.

# 1. MACHINE
Pass4 checkout plus a base worktree `../new-world-m2-base` at `e5594e3f`. Data in `/Users/rexxa/work/m2-data/`. `CARGO_BUILD_JOBS=8`, tests `--test-threads=8`, probe `--workers 6`.

# 2. CONTEXT (planner's survey at `e5594e3f`; paths under `rust/crates/`; verify)
- **Colony data:** `sim-core/src/colony/{items,orders,job,command}.rs`: `ItemKind { Log }`, `ItemStack`, `Carry`, `ItemLedger`, zones, designations, `BlueprintKind { Hut }`, `MaterialNeeds`, `ReserveKey`, `Command`, `WorkType { Build, Chop, Haul }`, `JobKind { Chop, Haul, Deliver, Build }`. `ColonyRule` (`rules/colony.rs`): tree logs, chop ticks 90, carry 5, stack 25, hut 12 logs / 240 progress / 4 beds, scan radius 96, stuck ticks, `yield_margin_permille`.
- **Jobs:** `sim-systems/src/runtime/jobs/{assign,execute,cancel,invariants,items,seam}.rs`; `JobAssignSystem` 124, `JobExecuteSystem` 127; the spatial index `sim-core/src/spatial.rs` (`ChunkIndex`, 16×16) is used by assign. Seam: `runtime/jobs/seam.rs` + 4 lines in `decision/system.rs`.
- **Firewood today:** `runtime/settlement/firewood.rs` (pure helpers), `fire.rs` (`FireSystem` 147: burns settlement `stockpile[Wood]`, cold nights), the decision tick's `GatherWood` arm (36 mentions in `decision/system.rs`) that sends agents to deadwood tiles (`SimResources.wood_tiles`) and deposits via `StockpileDepositSystem` 141. `FirewoodRule` (winter prep now one season, M1.5 calibration).
- **Materials:** `sim-core/src/material/` (`MaterialId`, `MaterialProperties` 15 fields, `registry`, `loader` for `data/material/**/*.ron`, `derivation`, `explanation`). Not loaded by the live engine today; verify where the registry is constructed in tests.
- **Skills:** no component. `Temperament { ns, ha, rd, p }` exists (TCI).
- **Buildings:** `BuildingKind { Hearth, Shelter, Stockpile, Hut }`, `ConstructionSystem` 133, `ConstructionSite`, `BlueprintKind::Hut` path in `commands.rs` + `jobs/execute.rs` (Deliver/Build).
- **Calendar (M1.5):** `CalendarRule`, 24-day year, `FertilitySystem` 150; save/load (`sim_systems::save`), text renderer (`worldsim_probe --ascii`), `docs/v8/snapshot_schema.md`, benchmark `worldsim_perf bench` + ledger; U-script for M1 fixtures in `docs/v8/m1/`.
- **UI:** `scripts/world/colony/{view,order_tool,overlay_layer,agent_layer,…}.gd`, `scripts/ui/hud/order_bar.gd` (F1 베기, F2 비축, F3 오두막, F4 구역 지우기, F6 취소; F5/F9 save/load), inspector `scripts/ui/panels/agent_inspector_panel.gd`, `Locale.ltr/trf`.
- **Art track (side branches, not on main):** `v8/art-trial` (32-px generated trial, failed), `v8/art-concept` (8 concepts; owner chose 05 ink-and-tone on 2026-10-22), `v8/art-trial2` (running). The decisions: drop pixel-art quantization; illustrated 2D top-down at 64-px tiles; one person = one tile stays; people as cut-out rigs built from generated parts with attributes and equipment as part swaps; portraits generated; procedural overlays follow an ink "code companion" rule.

---

# 3. DESIGN (what M2 builds; values are the colony's, baseline untouched)

## 3.1 Items
```rust
pub enum ItemKind { Log, Firewood, Stone, Flint, FlintBlade, StoneAxe, Hide /* 6.4 */ }
```
- `ItemStack` gains `quality: u8` (0–100; 50 for raw materials; crafted items carry the craft's quality) and `material: Option<MaterialId>` (raw: `flint`, `granite`, `oak`…; `Log` = the tree's species material, default `oak`). Stacks merge only when kind, material and quality bucket (quality / 10) match.
- Stack max per kind: Log 25, Firewood 50, Stone 30, Flint 30, FlintBlade 20, StoneAxe 5, Hide 10.
- Carry: logs 5, firewood 10, stone 6, flint 8, blades 10, axes 2.
- `ItemLedger` becomes per kind; conservation per kind holds every tick (M1 invariant generalized), with crafting as a **transform** entry: inputs consumed, outputs created, both logged.

## 3.2 Resource nodes
- **Boulders** (`Boulder { stone_left: u8 }`, 8–20 stone): placed at colony worldgen on hill tiles and 2 % of grass tiles with a keyed draw; quarrying yields `Stone` of the tile's rock material (hill → `granite`, else `limestone`).
- **Flint outcrops** (`FlintOutcrop { flint_left: u8 }`, 6–14): on river-bank tiles (within 2 of water), 1 per 40 bank tiles, keyed.
- Designation `Quarry` (rect over boulders/outcrops) → `JobKind::Quarry { node }`, `WorkType::Quarry`, `quarry_ticks` 120 per unit batch of 4 (so a boulder of 12 takes 3 batches), items spilled by the 7.4 merge/spill rule of M1.
- Nodes are exhausted and removed at 0; no regrowth. Both indexed in the spatial index.

## 3.3 Workshops (blueprints, built like the hut with `MaterialNeeds`)
| kind | footprint | needs | progress | bills it accepts |
|---|---|---|---|---|
| `KnappingSpot` | 1×1 | Stone 4 | 60 | `FlintBlade` |
| `WoodBench` | 2×1 | Log 6, Stone 2 | 120 | `StoneAxe`, `Firewood` (split logs) |
| `DryingRack` (6.4, droppable) | 2×1 | Log 4 | 80 | `Hide` (from a future animal item; M2 only places it and accepts no bill — record) |

A workshop has a **work tile** (its south tile) and an **input shelf**: items delivered for a bill sit on the workshop's footprint as reserved stacks until consumed.

## 3.4 Recipes (`sim-core/src/colony/recipes.rs`, data in `data/colony/recipes.ron`, loaded through `sim-data`'s pattern)
| recipe | workshop | inputs | output | work ticks | skill |
|---|---|---|---|---|---|
| knap blade | KnappingSpot | Flint 1 | FlintBlade 1 | 45 | Knap |
| haft axe | WoodBench | FlintBlade 1, Log 1 | StoneAxe 1 | 90 | Woodwork |
| split firewood | WoodBench | Log 1 | Firewood 4 | 30 | Woodwork |

Material carries through: a blade made from `flint` is `FlintBlade{material: flint}`; the axe inherits the blade's material and the log's species for display.

## 3.5 Bills (`Bill { id, workshop, recipe, mode: Make(n) | Maintain(n), remaining, suspended }`)
- Commands: `AddBill`, `RemoveBill`, `SetBillMode`, `MoveBill` (order within a workshop). Bills are in the command log (replay).
- `Maintain(n)`: active while the stockpiled count of the output kind (across stockpile zones, unreserved) < n.
- `BillSystem` (priority 123, interval 60): for each active bill with inputs available (in stockpiles or on the ground, unreserved) emits **craft work** as a `JobKind::Craft { workshop, bill }` after the inputs are delivered (`JobKind::Deliver` generalized to any `MaterialNeeds`-like target: the bill's input shelf). Output goes to the shelf, then a Haul job takes it to a stockpile that accepts the kind.

## 3.6 Work priorities
- `WorkPriorities` component: `BTreeMap<WorkType, u8>` with 0 = never, 1 = highest … 4 = lowest; default 3 for every type, `Child` all 0, elders `Quarry` 0.
- `WorkType { Craft, Build, Quarry, Chop, Split (firewood), Haul }` — the **order among equal priority** is this list. `JobAssignSystem` picks the lowest priority number first, then this order, then nearest.
- Command `SetWorkPriority { agent, work, level }` (command log).

## 3.7 Skills, speed, quality
- `Skills` component: `BTreeMap<WorkType, Skill { level: u8 (0–20), xp: u32 }>`; XP += work ticks completed; level thresholds from a table `XP_TABLE[21]` (quadratic, integer). Baseline never has the component.
- Speed: work ticks of a job are multiplied by `SPEED_PERMILLE[level]` (table: 1400 at 0 → 1000 at 5 → 700 at 20). Integer, no float.
- Quality of a crafted item: `q = clamp(base 30 + 3·level + bonus, 0, 100)` with `bonus` a keyed draw from `QUALITY_BONUS_TABLE` (−10 … +15, triangular, keyed by `(seed, agent, bill, tick)`); the **material** adds `(hardness − 50)/5` for blades (from `MaterialProperties`, integerized once at registry load). Record where each number comes from.
- Temperament hook (small, documented): `p` (persistence) ≥ 60 ‰ adds +2 to quality, `ns` ≥ 60 ‰ adds −2 ticks… keep it to two documented effects or drop (DECISIONS).
- **Tools:** an agent assigned a Chop job picks up the best unreserved `StoneAxe` in a stockpile within 24 tiles on the way (a Haul-like leg), holds it in `Carry`-adjacent `Tool` slot; Chop work ticks ×`AXE_PERMILLE[quality bucket]` (600 … 450). Durability (7.4, droppable): 200 uses, then the axe becomes `Stone 1`.

## 3.8 Firewood through jobs (the first decision-tick extraction)
- The hearth's need is an **auto-bill**: `Maintain(firewood_days × burn_per_day)` on the colony's `WoodBench` once it exists; before a bench exists, a `GatherDeadwood` job (`WorkType::Split`'s fallback): deadwood tiles yield `Firewood` items directly (2 per tile batch) instead of the abstract settlement wood.
- A `HearthFeedSystem` (priority 146, interval 60) moves `Firewood` items from stockpiles (nearest first) into the settlement's abstract `stockpile[Wood]` by the existing unit conversion (1 Firewood = 1 wood unit), so `FireSystem` stays as it is. Record the conversion.
- **Remove** the decision tick's `GatherWood` arm for the colony: gate it off under `rules.colony` (baseline keeps it; identity). Measure: lines in `decision/system.rs` no longer reachable in the colony (report), and the decision system's tick share at N=1,000 before/after (benchmark per-system table).
- 8.3 (droppable): the same for berry foraging as a `Gather` job with `Berries` items — **not** in M2 unless 8.1–8.2 and all bars are green by hour 14; if attempted, food stays an abstract settlement stock fed by a `FoodFeedSystem` like firewood.

---

# 4. PART 0 — Records on the main line (owner's rule: direction, roadmap and work live in the repo's markdown)
1. Commit this goal as `docs/v8/m2/M2_GOAL.md`.
2. **ADR 0018 — Art direction: illustrated 2D top-down at 64-px tiles, concept 05, cut-out people, generated environment.** Context (M1-ART ceiling, ART-TRIAL failure with numbers, ART-CONCEPT boards, the owner's choice and the sustainability condition), decision, alternatives (02 woodcut, 01 painterly, staying pixel), consequences (camera 64 px at 1×, 0.5× far zoom; `gen_tiles.py` retired for everything but overlays; M-ART2 integration after ART-TRIAL2's verdict), sources (the three branches and their reports by commit).
3. **VISION.md §6 rewritten**: the look (concept 05), hierarchy unchanged, the person as a rig with attributes and equipment, portraits, the code-companion rule; the old §6.1 table and mock-up references moved to a "history" note.
4. **ROADMAP.md**: rows for ART-TRIAL (done, failed, commit), ART-CONCEPT (done, commit), ART-TRIAL2 (running, branch), **M-ART2 통합** (pending: 64-px camera, sheet/rig loader, generated environment, code companion overlays, portraits), M2 (this pass, running), M-PERF (pending, after M2). "다음" updated.
5. **HISTORY.md**: lines for the three art passes and the direction change (2026-10-21/22).
6. **docs/archive/INDEX.md**: the art branches listed as living branches with what they hold.
7. `CHANGELOG.md`: an "Unreleased — M2" header that Part 7 fills.
Push Part 0 before any code.

# 5. PART 1 — Items, nodes, data
3.1, 3.2, the ledger generalization, the recipe/material loading (materials registry constructed once at `build_world_with` for the colony, with the 15-property struct integerized into `MaterialStats { hardness, density, workability, flammability }` ‰ for runtime use; baseline does not load it). Tests: ledger per kind over 8 seeds × 10 days with quarry and chop orders; node exhaustion; identity.

# 6. PART 2 — Workshops, bills, recipes
3.3–3.5; `Deliver` generalized; `Craft` job (stage `Working { ticks_left }` at the work tile; inputs consumed at start, output created at the end, ledger transform); bill commands in the log and replay. Tests: knap bill of 5 completes from 5 flint; `Maintain(10)` firewood keeps 10 ± carry in stock over 10 days; cancel mid-craft returns inputs (or not — decide and record: M2 rule is inputs are consumed at start, so a cancelled craft loses them, logged as `consumed`); identity.

# 7. PART 3 — Priorities, skills, quality, tools
3.6–3.7. Tests: a person with Chop = 0 never chops in 10 days with 30 trees marked; XP grows only on completed work; speed table monotone; quality distribution for level 0 vs level 10 (mean difference ≥ 25 over 200 draws); an axe halves chop time (ticks ratio within 0.55–0.65); identity.

# 8. PART 4 — Firewood through jobs
3.8. Pre-registered bars in §10. Tests: no abstract `GatherWood` targets ever appear in a colony run (assert on the snapshot's target kinds); `HearthFeedSystem` conservation (firewood items − fed = wood units); baseline identity (the decision arm still runs for baseline).

# 9. PART 5 — UI (GDScript; the current 16-px view stays — M-ART2 replaces it; keep changes to panels and the order bar)
1. Order bar: F3 becomes a **build menu** (오두막 / 돌깎이 자리 / 목공대 / 건조대) with `1–4` inside the menu; add `F7 채석` (quarry designation). Keep F6 취소.
2. **Workshop panel** (click a workshop): its bills as a list — recipe name, mode (만들기 n / 유지 n), remaining, suspend toggle, ▲▼ order, 삭제; an "추가" button listing the workshop's recipes. Commands through the bridge.
3. **Priorities table** (`P` key): rows = adults, columns = the six work types, cells cycle 0–4 on click with colour; a 모두 기본 reset. Reads `WorkPriorities`, writes `SetWorkPriority`.
4. Inspector: 기술 lines (`돌 다듬기 7 · 숙련`, levels named 0–4 초보 / 5–9 견습 / 10–14 숙련 / 15–20 달인), 도구 line, and the carried item with material and quality (`부싯돌 날 ×3 · 품질 62`).
5. Overlay: nodes (boulder, outcrop) and workshops drawn from the existing sheet with two new temporary tiles (`gen_tiles.py`, minimal — M-ART2 replaces them); quarry marks; item stacks by kind (icon + count).
6. Localization keys: list every key en/ko in `docs/v8/m2/M2_LOCALE.md` (≥ 40 expected: item names with materials, recipes, work types, skill names and tiers, bill modes, panel labels, quarry order, reject reasons). Use `Locale.trf` for formats.

# 10. PART 6 — Evaluation (fresh seeds 300–331; calibration on 0–7 only, ≤ 6 rounds, recorded)
Script M2-U (resolved per seed at tick 360): chop 20 trees; stockpile 6×6 accepting everything; quarry the 2 nearest boulders and 1 outcrop; place KnappingSpot and WoodBench; bills: knap `Make 6`, haft axe `Make 2`, firewood `Maintain 40`; set one adult's Chop to 0 and another's Craft to 1. Run 20 days.

| # | bar | pass |
|---|---|---|
| E1 | 6 blades made within 10 days | ≥ 28/32 |
| E2 | 2 axes made within 15 days; at least one Chop job afterwards used an axe | ≥ 28/32 |
| E3 | firewood items in stock ≥ 20 on every day from day 8 | ≥ 28/32 |
| E4 | the Chop = 0 adult has 0 Chop jobs; the Craft = 1 adult holds ≥ 50 % of craft jobs | 32/32 |
| E5 | per-kind ledger exact every tick; reservations invariant | 32/32 |
| E6 | no hunger/thirst/cold death in 20 days; and in a 3-year no-orders run (`Maintain 40` firewood auto-bill only) ≥ 30/32 with 0 cold deaths | both |
| E7 | quality: mean quality of blades by the Craft=1 adult at day 20 > mean at day 3 by ≥ 8 | ≥ 24/32 |
| E8 | decision system share of tick at N=1,000 (bench) drops by ≥ 10 points vs the M1.5 ledger; `GatherWood` arm unreachable in colony | both |
| E9 | bench ceilings (hard) and ≤ +10 % regression | pass |
| E10 | baseline identity; replay of M2-U byte-identical | pass |

# 11. PART 7 — Docs, review, gate, report
## 11.1 Docs
CHANGELOG (fill "Unreleased — M2"); ROADMAP (M2 done + commit; 다음 → M-ART2 if ART-TRIAL2 reported, else M3; recommend); HISTORY; **ADR 0019 items, bills and the economic loop** (why inputs are consumed at start, why food stays abstract, the material integerization), **ADR 0020 skills and quality tables**; ARCHITECTURE (system table regenerated; §4 new commands/reads; a new §9 "Items and jobs" data flow; `snapshot_schema.md` updated); `PERF_LEDGER.md` line; skills (`worldsim-architecture`: "add an item kind", "add a recipe", "add a work type", tables not floats; `worldsim-testing`: ledger-per-kind pattern; pitfalls).
## 11.2 Review (two fresh subagents)
Code: ledger holes across craft transforms; reservation leaks on bill removal; determinism of bill scheduling (order, keys); the seam line count (`git diff --numstat` on `decision/system.rs` must be **net negative** or ≤ +20 with justification); baseline leaks. Numbers: recompute E1, E3, E7, E8 from raw output.
## 11.3 Screen checks (windowed, shots to `$DATA/shots/`)
A knapping spot with a bill list open; the priorities table; an inspector showing a skill tier and a carried blade with quality; the order bar's build menu.
## 11.4 Gate and publish
Gate (now with hard `bench`) green on a clean commit → fast-forward `v8/main` → report commit → docs step → fast-forward.

# 12. Not in M2
Food as items (8.3 only if time allows); animals and hide sources; rooms (M3); hill digging (M3); knowledge/tech (M4); pathfinding; the 64-px view (M-ART2); needs conversion from f32 (M-PERF); neighbours.

# 13. 인게임 확인사항 (오너용)
1. F7로 바위를 채석 지정하면 사람이 가서 돌을 캐고, 돌 더미가 생겨 비축 구역으로 옮겨진다. 강가 부싯돌 노두도 같다.
2. F3 메뉴에서 돌깎이 자리를 놓으면 돌 4개가 배달된 뒤 지어진다.
3. 돌깎이 자리를 클릭해 "부싯돌 날 만들기 6"을 추가하면, 부싯돌이 배달되고 누군가 앉아 깎아 날이 생기고, 날이 비축 구역으로 간다.
4. 목공대에서 "돌도끼 만들기"와 "장작 유지 40"을 걸면 도끼가 생기고, 장작이 40개 근처로 유지되며, 화덕이 그 장작을 쓴다.
5. P 키의 우선순위 표에서 한 사람의 '베기'를 0으로 두면 그 사람은 절대 나무를 베지 않는다.
6. 사람 상세에 '돌 다듬기 3 · 초보'처럼 기술이 보이고, 같은 사람이 날을 만들수록 품질 숫자가 올라간다.
7. 도끼를 든 사람은 나무를 눈에 띄게 빨리 벤다.
8. 3년을 아무 주문 없이 돌려도(장작 유지만) 추위로 죽지 않는다.
9. 저장(F5)·불러오기(F9) 뒤에도 주문·우선순위·기술이 그대로다.

# 14. 구현 완료 보고 (한국어; `docs/v8/m2/M2_REPORT.md`와 최종 메시지)
```
## 구현 완료 보고 — M2
### 구현 의도
### Part 0 기록 (ADR 0018, VISION §6, ROADMAP·HISTORY·archive 갱신 — 커밋)
### 구현 내용 (물건·노드·재료, 작업장·주문·레시피, 우선순위·기술·품질·도구, 장작 일감화, UI — 파일별)
### 구현 방법 (장부 변환 기록, 주문 스케줄링 키, 속도·품질 표의 근거, 재료 정수화, 장작 변환, 결정 틱 제거 범위와 줄 수)
### 평가 결과 (E1–E10 표 + 시드별 부록 경로)
### 벤치마크 (장부 줄, 시스템별 상위 10, 결정 시스템 점유율 전후)
### 보정 기록
### 변경된 파일 목록 (커밋별)
### 리뷰 결과
### 확인된 제한사항 (드롭한 것, 식량 추상 유지, 1,200줄 초과 파일)
### 오너 결정 필요 (다음: M-ART2 vs M3; 우선순위 표의 기본값; 품질 표시 방식)
### Gate 결과
git ls-remote 결과
```
