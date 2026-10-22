# GOAL: WorldSim — M4: knowledge and technology — skills are owned by people and die with them; knowledge spreads by talk, watching and teaching; the stone age has a first tech map of ~24 nodes; the settlement's era is observed, not declared; and the first Neolithic markers (ground stone, pottery) can be discovered. Plus the M3 follow-ups: the benchmark ledger rule, river fords, tool wear so skill growth can finally be measured.

Repository `hyunlord/new-world`. Base: **`v8/main` = `5da079d37865d5ee00affc4a517e7f1897cfbfa3`** (M3 report; gate green at `bf16de16`). `v8/wip` is at the same commit.

Executed by **Claude Code**, unattended, on the Mac in `/Users/rexxa/work/new-world-pass4`. No DGX. Headless except 12.3. Target 20 hours, hard stop 24. The art session may run ART-TRIAL5 on `v8/art-trial5` in parallel; do not touch `art/**`, `docs/v8/arttrial*/**`, do not merge art branches.

## Why this pass exists

ROADMAP M4: "토기를 아는 사람이 가르치고, 그 사람이 죽으면 위험 경고가 뜬다." VISION §4 fixes the design and this pass implements its first stage: technology is **owned by individuals and disappears with them** (Henrich 2004, the Tasmania effect; Boyd & Richerson dual inheritance), spreads through channels of different fidelity and speed, decays when unused, is **observed** at settlement level as the union of what people know (practitioner counts → stable / at risk / lost), and the era is a label computed from the coverage of marker technologies — "신석기 전환기 12 %" — never a button.

Today a person's competence is `Skills` (M2: six kinds, level 0–20 from work ticks) and anyone may take any job. Nothing is known or unknown; nothing can be lost; nothing is taught. M4 adds `Knowledge` beside `Skills` and keeps both: knowledge gates *whether* a person can do a thing and sets the quality ceiling; skill level keeps setting speed and the quality roll. This is deliberately the small version of the owner's design (sparse storage ≈ 100 B/person, Warm/Cold cadence, 4 of the 6 channels; records and schools are later eras).

M3 left three debts carried here as Part 0: the benchmark ledger accepted a contended session as the comparison row (the +10 % rule was effectively bypassed); the default world's near forest lies across the river with no ford and no refusal reason; and skill growth (F7) has never been measured because nothing consumes blades — tools now wear out.

---

# 0. RULES
- AGENTS.md and the skills (`worldsim-architecture` first: C1–C5, the index, the seam, "add a command / job type / item kind / blueprint kind / room role"). Commit to `v8/wip` (`v8-m4(pN): …`), `v8/main` by fast-forward after a green gate. Never `lead/main`, `main`, `codex/*`, `v8/art-*`; no force-push; explicit paths; hook and ignore files untouched; leave the untracked `.ignore`, `docs/revitalize/`, `.gitignore` change alone.
- **Never ask the owner** → `docs/v8/m4/M4_DECISIONS.md`.
- C3 ceilings and the regression rule are hard (gate `bench`, with the Part 0 ledger fix). `decision/system.rs`: ≤ +20 lines.
- Parts: 0 = §5 · 1 = §6 (knowledge data + component + gating) · 2 = §7 (channels, forgetting, discovery) · 3 = §8 (settlement aggregate, era, pottery/grinding chains) · 4 = §9 (UI) · 5 = §10 (evaluation) · 6 = §11 (docs, review, gate, report). Push after every Part.
- **Time rule:** past 24 h, finish the step, push, go to Part 6.
- **Drop order:** 8.6 water-pot effect (pots then only count for progress) → 7.4 observation channel → 9.6 tech-map lane polish (keep the list view) → 8.5 ground axe (keep stone grinding as a knowledge with the grinding stone) → knowledge "why?" lines.
- **Never drop:** Part 0, `Knowledge` with gating, oral + apprenticeship channels, forgetting, practitioner aggregate with at-risk/lost warnings and chronicle, era progress in the HUD, the pottery chain to a fired pot, the tech map (list view at minimum), tool wear + the soak with a blade sink, evaluation, identity, docs, report.

# 1. MACHINE
Pass4 checkout + base worktree `../new-world-m4-base` at `5da079d3`. Data `/Users/rexxa/work/m4-data/`. `CARGO_BUILD_JOBS=8`, tests `--test-threads=8`, probe `--workers 6`.

# 2. CONTEXT (planner's survey at `5da079d3`; paths under `rust/crates/`; verify)
- **Skills:** `sim-core/src/colony/skills.rs` — `Skills { entries: [Skill; 6] }`, `SkillKind { Knap, Woodwork, Chop, Quarry, Build, Mining }` (`workshop.rs`), `XP_TABLE`, `SPEED_PERMILLE`, `QUALITY_BASE 30 + 3·level + bonus`, `AXE_PERMILLE`. `WorkType { Craft, Build, Quarry, Chop, Split, Haul, Mine }`, `WorkPriorities` 0–4 with the M3 skill tie-break.
- **Recipes:** `sim-core/data/colony/recipes.ron` (`knap_blade`, `haft_axe`, `split_firewood`; fields id/workshop/inputs/output/work_ticks/skill/work). Workshops `KnappingSpot, WoodBench, DryingRack`; bills Make/Maintain. `ItemKind { Log, Firewood, Stone, Flint, FlintBlade, StoneAxe, Hide }` with material + quality; ledger invariant per kind.
- **Space (M3):** `BlueprintKind` incl. `LogWall, StoneWall, Door, PackedFloor, Bedroll`; designations `Chop, Quarry, Dig, Deconstruct`; rooms with roles; nav (`sim_core::nav`, regions, memo); `TileGrid.door/dug`.
- **People:** `Temperament { ns, ha, rd, p }` (u16), `Family`, `Lineage`, `Memory` (V7 substrate, 32 entries), `Social.loneliness` (f64, baseline), ages in years (adult 15), deaths from cold/thirst, the evening hearth gathering (hall).
- **Chronicle:** `sim-core/src/chronicle.rs` `ChronicleKind::ALL` (24 kinds) with subject/object fields; chronicle panel in the UI; the causal "why?" panel.
- **Commands:** `Command` enum + log + replay; `ReserveKey`; jobs `assign.rs/execute.rs`; `ChunkIndex`.
- **Bench ledger:** `tools/v8/bench.sh` waits ≤ 45 min for 1-min load < 8 then labels CONTENDED; `bench_ledger.py check` compares with the **latest** pass rows; the M3 rows (`23be0178`) were appended from a CONTENDED session (load 19–25), so the gate's quiet run at `bf16de16` compared against inflated numbers (×0.96–×1.099) and the true M2→M3 change was never measured.
- **UI keys in use:** F1 chop, F2 stockpile zone, F3 build (1–9), F4 clear zone, F6 cancel, F7 quarry, F8 dig, F10 deconstruct, R rooms, P priorities, Space pause, Home/H, F5/F9 save/load. Free: **T** (tech map), **K** (teach order).
- **Calendar:** 1,440 ticks/day, 6-day seasons, 24-day year. Founders 10; fertility 33 ‰/day.
- **Art state to record (§5.1):** ART-TRIAL4 (`9f9c6d6b`) not adopted, people parked (D21); ART-TRIAL5 running on `v8/art-trial5` (whole-figure sheets, owner picks the look in Part 0) — record whatever `docs/v8/arttrial5/` holds at Part 6 time.

# 3. DESIGN

## 3.1 Knowledge data (ADR 0024)
`sim-core/data/colony/knowledge.ron` (`schema_version 1`), loaded like recipes. `KnowledgeId` = index in file order (u8). Fields: `id`, `lane` (Survival | Craft | Build | Mine | Pottery | Social), `era` (Paleolithic | Neolithic), `threshold` ‰ (proficiency at which a person is a practitioner and may do the gated work), `prereqs: [id]`, `difficulty` (1–3: discovery chance class), `discover: [cond]` (see 3.4), `gates: [gate]` (Recipe(id) | Blueprint(kind) | Designation(kind) | Workshop(kind) | Channel(Apprenticeship)), `marker_weight` (0 = not a marker; Neolithic markers have weights), `name_key`.

The first map (24 nodes; the implementer may adjust prereq edges only, in DECISIONS):

| id | lane | era | thr ‰ | prereqs | gates | discover | w |
|---|---|---|---|---|---|---|---|
| fire_keeping | Survival | P | 300 | — | hearth feeding | seeded | 0 |
| foraging | Survival | P | 300 | — | gather food | seeded | 0 |
| water_finding | Survival | P | 200 | — | — (lore) | seeded | 0 |
| flint_knapping | Craft | P | 300 | — | Recipe knap_blade, Workshop KnappingSpot | seeded (3) | 0 |
| hafting | Craft | P | 300 | flint_knapping | Recipe haft_axe | seeded (2) | 0 |
| firewood_splitting | Craft | P | 200 | — | Recipe split_firewood, Workshop WoodBench | seeded (4) | 0 |
| pressure_flaking | Craft | P | 400 | flint_knapping | Recipe knap_fine_blade (quality +15 floor) | Stockpile FlintBlade ≥ 10 ∧ knapper level ≥ 6 | 0 |
| stone_grinding | Craft | N | 400 | hafting, stone_quarrying | Recipe grind_axe, Workshop GrindingStone | Stockpile Stone ≥ 12 ∧ hafting ≥ 500 ‰ | 3 |
| hut_building | Build | P | 300 | — | Blueprint Hut | seeded | 0 |
| timber_wall | Build | N | 300 | hut_building | Blueprint LogWall | Logs ≥ 8 in stockpile (seeded 2) | 2 |
| doorway | Build | P | 200 | timber_wall | Blueprint Door | seeded (1) | 0 |
| packed_floor | Build | P | 200 | — | Blueprint PackedFloor | seeded (2) | 0 |
| bedding | Build | P | 200 | — | Blueprint Bedroll | seeded (3) | 0 |
| stone_walling | Build | N | 400 | timber_wall, stone_quarrying | Blueprint StoneWall | Stone ≥ 20 ∧ timber_wall ≥ 500 ‰ | 1 |
| stone_quarrying | Mine | P | 300 | — | Designation Quarry | seeded (3) | 0 |
| digging | Mine | P | 300 | stone_quarrying | Designation Dig | seeded (2) | 0 |
| clay_digging | Mine | N | 300 | digging | Designation ClayPit | river bank within 24 tiles of camp ∧ digging ≥ 400 ‰ | 0 |
| clay_prep | Pottery | N | 300 | clay_digging | — | Stockpile Clay ≥ 4 | 0 |
| pot_forming | Pottery | N | 400 | clay_prep | Recipe form_pot, Workshop PotteryBench | clay_prep ≥ 500 ‰ ∧ fire_keeping ≥ 300 ‰ | 0 |
| pot_firing | Pottery | N | 400 | pot_forming, fire_keeping | Recipe fire_pot (at the hearth) | Stockpile RawPot ≥ 2 | 3 |
| instruction | Social | P | 300 | — | Channel Apprenticeship | seeded (2) | 0 |
| storytelling | Social | P | 200 | — | oral channel fidelity +10 % | seeded | 0 |
| tracking | Survival | P | 300 | — | — (M5 hunting) | seeded (2) | 0 |
| hide_scraping | Craft | P | 300 | flint_knapping | Workshop DryingRack (no recipe until M5) | seeded (1) | 0 |

"seeded (n)" = n of the 10 founders start as practitioners (deterministic by seed via `STREAM_KNOWLEDGE`), at proficiency 500–700 ‰; "seeded" without n = all founders at 600 ‰. Everything Neolithic starts at 0 holders. The seeding guarantees at least two technologies with exactly one holder from day 1 (doorway, hide_scraping) so the at-risk warning is real.

## 3.2 Component and gating
```rust
pub struct KnowledgeEntry { id: KnowledgeId, proficiency: u16 /* ‰ 0..=1000 */, last_used: u32 /* tick, u32 */, source: Source /* Seeded|Discovered|Oral|Observed|Apprenticed */, observed: u16 /* ‰ gained by observation only */ }
pub struct Knowledge { entries: SmallVec<[KnowledgeEntry; 12]> }   // sorted by id; ≤ 24
```
≈ 12 × 10 B + header ≤ 160 B; the C3 4 KiB/agent ceiling stands. Added to every colony person at spawn/birth (children: empty). Save format v4.

**Gating** (ADR 0024): `JobAssignSystem` offers a job only to people who are practitioners of every knowledge whose `gates` cover that job's recipe / blueprint / designation / workshop (`knowledge::gate_for(job) -> &[KnowledgeId]`, a precomputed table). Placing an order is never refused (the player may plan ahead); the order sits with reason `NoPractitioner` visible in the "why?" panel and the tech map. Ungated work (haul, chop, gather, build Hut if hut_building is universal) stays open to all. **Quality ceiling:** craft quality = `min(QUALITY_BASE + 3·level + bonus, QUAL_CAP[proficiency / 100])` with `QUAL_CAP = [0, 20, 35, 45, 55, 65, 75, 85, 92, 97, 100]` (the VISION's `0.3 + 0.7p²` shape as an integer table). Using a knowledge (completing a gated job) sets `last_used` and adds **+2 ‰** proficiency (cap 1000).

## 3.3 Channels (ADR 0025)
- **Oral** (`OralTransmissionSystem`, priority 160, runs once per evening at the hearth gathering, tick 1,260 of the day): pair each gathered person with a keyed random other gathered person; for each knowledge the teller holds ≥ threshold and the listener lacks, the listener gains an entry at **350 ‰** (`Oral`) with probability 50 % (keyed, `STREAM_KNOWLEDGE`), +10 points if the teller knows storytelling; at most one knowledge per pair per evening. Oral entries cannot reach threshold without use or teaching: cap 350 ‰ unless the knowledge's threshold ≤ 300 (then it is a practitioner at once — talk suffices for simple things).
- **Observation** (7.4, droppable; `ObservationSystem` 161, interval 30): a person working within Chebyshev 3 tiles of a practitioner performing gated work gains +5 ‰ (`Observed`), tracked in `observed`, capped at `threshold − 50`.
- **Apprenticeship** (`Teach` jobs): command `OrderTeach { teacher, learner, knowledge }` (recorded). Requires: teacher practitioner, teacher knows `instruction`, learner adult or child ≥ 10 years, learner below threshold. Creates a paired job: `JobKind::Teach { learner, knowledge }` for the teacher (`WorkType::Craft` priority rules; new `SkillKind` not needed) and `JobKind::Learn` for the learner; they meet at the gated workshop (or the hearth for Survival/Social lanes). A session = 60 ticks; per session the learner gains `+15 ‰ × fidelity`, fidelity = 950 ‰ (× 1,100 ‰ if the teacher's proficiency ≥ 800); ≤ 4 sessions per day; the order completes when the learner reaches the threshold (`Apprenticed`), cancels if either dies, leaves, or the teacher falls below threshold. Chronicle `KnowledgeLearned { learner, teacher, knowledge, channel }`.
- **Forgetting** (`KnowledgeDecaySystem` 162, interval 60, Cold): for entries with `tick − last_used > 6 days (8,640 ticks)`: proficiency −= `DECAY[source]` per interval — Seeded 1, Discovered 1, Apprenticed 1, Oral 2, Observed 2 (‰); floors: Seeded/Discovered 100, others 0; an entry at 0 is removed. (An unused oral skill at 350 ‰ is gone 6 + 7.3 days after its last use; a seeded 600 ‰ skill falls below a 300 threshold after 6 + 12.5 days.) Numbers are calibration candidates (§10).
- **Discovery** (`DiscoverySystem` 163, interval 240): candidates = adults with `Temperament.ns ≥ 600` (innovation potential; top ≈ 20 %); for each knowledge not held by **this person**, with all prereqs held ≥ threshold by this person and `discover` conditions true for the settlement: chance per check by difficulty `CHANCE = [1/40, 1/90, 1/200]`; on success the person gains it at 500 ‰ (`Discovered`); chronicle `KnowledgeDiscovered`. Conditions evaluated once per interval into a bitset (stockpile counts from the ledger; "river bank within 24 tiles" from worldgen; skill levels from `Skills`).

## 3.4 Settlement aggregate and era (ADR 0026)
- `TechAggregateSystem` 164, interval 240, writes `SimResources.tech: Vec<TechState>` per knowledge: `practitioners` (adults ≥ threshold **and** `last_used` within 6 days, or seeded/learned within the last 6 days), `holders` (≥ threshold regardless of use), `state ∈ { Unknown, Known { stable: practitioners ≥ 3 }, AtRisk (1–2 practitioners), Lost (was Known, now 0 holders) }`, `last_change_tick`, `names of practitioners (≤ 5)`.
- Events on transitions: chronicle `KnowledgeAtRisk { knowledge, holder }` when practitioners drop to 1, `KnowledgeLost { knowledge, last_holder }` when holders reach 0 after having been Known; HUD warning line "⚠ 문: Fox만 앎" (`M4_WARN_AT_RISK`), toast on loss.
- **Era progress:** markers = knowledge with `marker_weight > 0` (Neolithic set in M4: stone_grinding 3, pot_firing 3, timber_wall 2, stone_walling 1; farming 3 and weaving 2 are **listed with weight but absent** so the ceiling is 9/14 ≈ 64 %). `coverage_m = min(1000, 1000 · practitioners_m / adults)`; `progress‰ = Σ w·coverage / Σ w`. Label: < 250 "구석기", 250–749 "신석기 전환기", ≥ 750 "신석기 시대" (unreachable in M4 by design). Shown in the HUD as "신석기 전환기 31 %"; recomputed with the aggregate; it can go **down**.

## 3.5 Chains that make the markers real (ADR 0024 data)
- **Clay:** worldgen places `ClayBank` ground resource on river-bank tiles (like flint outcrops, density such that ≥ 3 banks lie within 24 tiles of the camp in ≥ 30/32 seeds); designation `ClayPit` (`WorkType::Mine`, 60 ticks, yields `Clay 3`, bank depletes after 6 digs); `ItemKind::Clay`.
- **Pottery:** workshop `PotteryBench` 1×1 (Log 1, progress 20); recipe `form_pot`: Clay 2 → `RawPot 1`, 60 ticks, skill `Pottery` (new 7th `SkillKind`; the priorities table gets a column); recipe `fire_pot` at the **hearth** (the hearth becomes a workshop target for this recipe only): RawPot 1 + Firewood 2 → `ClayPot 1`, 90 ticks, skill Pottery. `ItemKind::{Clay, RawPot, ClayPot}`.
- **8.6 Water pot (droppable):** a `ClayPot` stored in a room's stockpile can be **filled**: `JobKind::FillPot` (Haul type) at a river tile adjacent to passable ground, 30 ticks, sets the item's `filled: bool` (an item flag field, new); `ThirstSystem`: a thirsty person whose current room (or the hall) holds a filled pot drinks there (−1 fill = 4 drinks), instead of walking to the river. Effect visible as fewer thirst walks (D-recorded metric).
- **Ground stone (8.5, droppable):** workshop `GrindingStone` 1×1 (Stone 2, progress 30); recipe `grind_axe`: StoneAxe 1 + Stone 1 → `GroundAxe 1`, 120 ticks, skill Woodwork; `AXE_PERMILLE` for GroundAxe = 450 − 10·bucket (min 350); durability 120.
- **Tool wear (Part 0):** `StoneAxe` durability **40 chops** (`Item.durability: u16`, decremented per Chop job; at 0 the axe breaks: item consumed, ledger `consumed`, chronicle `ToolBroke`), `GroundAxe` 120, `FlintBlade` as a blade is consumed by hafting already. The soak and M4-U scripts carry `Maintain 2 StoneAxe` so blades are made all along → F8 measurable.

## 3.6 M3 follow-ups (Part 0)
- **Ledger rule:** `bench_ledger.py check` compares against the latest **quiet** pass rows (a CONTENDED row may be appended only with `--note` and is never a comparison base); re-measure M3 at `bf16de16` in a quiet session (wait rule as is; if the host never gets quiet within 45 min, say so and compare anyway, labelled) against M2's quiet rows (`f6a89e72`) and append it as the M3 pass row; the contended rows stay in the file, re-labelled "contended, not a base". Report the true M2→M3 ratios.
- **Fords:** worldgen guarantees ≥ 2 `Ford` tiles (river tiles with cost 30, passable) within 40 tiles of the camp where the river separates the camp region from the largest forest region; `Ford` drawn distinctly. Unreachable designation → toast "닿을 수 없음: 강 건너" (reason from the region check). Default seed unchanged.
- **M3 owner decisions recorded** (D-numbered): room role auto + manual lock is the default; roof = enclosure until further notice; rivers get fords, default seed stays; F7 re-measured in M4 with tool wear.

---

# 4. PERFORMANCE PLAN (before coding; in DECISIONS)
Knowledge systems are Warm/Cold: decay 60, observation 30 (neighbours via `ChunkIndex`, only people at work), discovery 240, aggregate 240, oral once a day. Budget at N = 1,000 with the M4 soak: **all knowledge systems ≤ 150 µs per tick mean**, `JobAssignSystem` growth from gating ≤ +40 µs (the gate table is a precomputed bitset per job kind; a person's practitioner bitset `u32` is cached on the component and refreshed by the decay/learn paths). Memory ≤ 160 B per person added. `worldsim_perf` gets a `knowledge` line in the per-system table. If over budget: (1) cache the practitioner bitset (should already be there), (2) observation every 60, (3) aggregate every 480.

---

# 5. PART 0 — Records and follow-ups
1. Commit this goal as `docs/v8/m4/M4_GOAL.md`.
2. Art record: ROADMAP/HISTORY/ADR 0018 note "ART-TRIAL5 running (whole-figure sheets; owner look pick)"; final state updated in Part 6 from `docs/v8/arttrial5/` if a report exists.
3. ADR 0024 (knowledge data, component, gating), 0025 (channels, forgetting, discovery), 0026 (aggregate and era) drafted; final in Part 6.
4. §3.6: ledger rule + quiet re-measure; fords + refusal toast; tool wear + `Maintain 2 StoneAxe` in `m3_soak_script.json` (the soak fixture now has a blade sink; rename to `soak_script.json`, keep the gate step name); M3 decisions recorded.
5. Base data: identity base, M3 colony fixtures, quiet ledger rows, soak first run → `$DATA/base/`.
Push.

# 6. PART 1 — Data, component, gating
§3.1–3.2. `sim-core/src/colony/knowledge.rs` (defs loader, `Knowledge`, `gate_for`, `QUAL_CAP`, practitioner bitset), seeding in `worldgen/colony.rs`, `JobAssignSystem` gate, quality cap in craft completion, `Skills` gains a 7th kind `Pottery` (priorities table column; saved priorities keep their codes). Tests: defs load and prereq graph is a DAG with ≤ 24 nodes; seeding counts per seed (3 knappers, 1 doorway, …) across seeds 0–7; a non-practitioner is never assigned a gated job (fixture: remove hafting from everyone → haft bills wait with `NoPractitioner`); quality cap applies; save/load v4 round-trip; baseline identity.

# 7. PART 2 — Channels, forgetting, discovery
§3.3. Tests: oral — two founders, one knows knapping, after 8 evenings the other holds it at 350 ‰ with probability consistent with 50 %/evening (keyed; assert the deterministic outcome for seeds 0–7 and that ≥ 5/8 learned by evening 8); apprenticeship — `OrderTeach` completes in ≤ 4 days of sessions and the learner is a practitioner (`Apprenticed`), cancels on the teacher's death; forgetting — a seeded 600 ‰ skill unused for 20 days is below 300 ‰ and the person is no longer offered the job; discovery — with Stone ≥ 12 and a hafting practitioner at ns ≥ 600, stone_grinding is discovered within 60 days on ≥ 6/8 seeds (probe, no orders beyond quarrying); chronicle entries present; identity.

# 8. PART 3 — Aggregate, era, chains
§3.4–3.5. Tests: aggregate states for a seeded colony (doorway AtRisk at tick 0, knapping Known stable); losing the only doorway holder (test-only `debug_kill` command available in the probe/tests, **never** in the game bridge) → `KnowledgeLost` within one aggregate interval and the Door blueprint waits with `NoPractitioner`; era progress is 0 at start and > 0 after a ClayPot and a GroundAxe exist and are used; clay banks placed on ≥ 30/32 seeds; the pottery chain from ClayPit to ClayPot completes in a scripted colony (ledger exact for Clay/RawPot/ClayPot); water pot drinks counted (if kept); tool wear breaks an axe after 40 chops and the ledger stays exact.

# 9. PART 4 — UI
1. **HUD:** era line "신석기 전환기 31 %" beside the calendar; a warning badge "⚠ 기술 위험 2" that opens the tech map.
2. **T — tech map panel:** lanes as rows (생존·제작·건설·채굴·토기·사회), eras as columns (구석기 | 신석기), nodes as cards: name, state colour (미지 회색 / 위험 빨강 / 알려짐 노랑 / 안정 초록 / 소실 검정 테두리), practitioner count, marker weight glyph; prereq edges as lines (9.6 droppable: lane polish; the list view with the same cards grouped by lane is the floor). Click a node → detail: practitioners with proficiency bars, who could learn it, discovery conditions and which are met, gated recipes/blueprints; a **가르치기** button → K order dialog pre-filled.
3. **K — teach order:** teacher dropdown (practitioners with `instruction`), learner dropdown (eligible), knowledge dropdown; creates `OrderTeach`; the active teaching list with progress and a cancel.
4. **Inspector:** 지식 tab — each entry: name, proficiency bar with the threshold mark, source, last used "3일 전", state words (숙련 / 배우는 중 / 잊어 감); the person's current teach/learn job.
5. **Priorities table:** the Pottery column. **Build menu:** PotteryBench, GrindingStone; **designations:** F7 submenu or key `F7` then `2` for ClayPit (keep quarry as `F7 1`); ClayBank and Ford tiles drawn (temporary sprites in `gen_tiles.py`).
6. **Chronicle:** the four new kinds with Korean lines; "why?" on a waiting order: "기다리는 중: 토기 굽기를 아는 사람이 없음".
7. Localization keys in `docs/v8/m4/M4_LOCALE.md` (≥ 70 expected: 24 knowledge names + lanes, states, panel labels, orders, reasons, chronicle lines, era labels). `Locale.trf` for formats.

# 10. PART 5 — Evaluation (fresh seeds 700–731; calibration seeds 0–7, ≤ 8 rounds, each logged)
**M4-U script** (resolved per seed at tick 360; renewals): the soak economy (with `Maintain 2 StoneAxe`, `Maintain 10 FlintBlade`, `Make 4 GroundAxe` once stone_grinding is known, chop renewals, quarry) **plus**: day 1 — M3's house + storehouse orders; day 2 — `OrderTeach` doorway from its single holder to one other adult; day 3 — ClayPit designations on all banks within 24 tiles once clay_digging is held by anyone (script polls each day), then `Make 6 ClayPot` once pot_forming is held; days 1–24 — nothing else. Run **36 days** (1.5 years).

| # | bar | pass |
|---|---|---|
| F1 | seeding: every seed starts with ≥ 2 technologies at exactly 1 practitioner and the HUD badge shows them | 32/32 |
| F2 | oral spread: knapping holders among adults ≥ 6 by day 12 (from 3) | ≥ 28/32 |
| F3 | apprenticeship: the doorway teach order completes by day 8 and the learner is a practitioner | ≥ 28/32 |
| F4 | at-risk → stable: doorway practitioners ≥ 2 by day 10 (F3) and ≥ 3 by day 24 via oral/teach | ≥ 24/32 |
| F5 | forgetting: at least one seeded skill in each world drops below threshold by day 36 (something unused) **and** no gated chain the script needs stalls for lack of a practitioner for more than 3 consecutive days | ≥ 28/32 |
| F6 | discovery: stone_grinding discovered by day 30 | ≥ 20/32 |
| F7 | pottery: clay_digging, clay_prep, pot_forming, pot_firing all discovered and ≥ 2 ClayPot fired by day 36 | ≥ 16/32 (report the chain's stall point per seed) |
| F8 | skill growth (the M2/M3 debt): top knapper level ≥ 6 at day 36 and blade quality mean days 25–36 − days 1–6 ≥ 8 (blades keep being made because axes wear) | ≥ 24/32 |
| F9 | era progress > 0 by day 36 in every seed where F6 or F7 passed; it never exceeds 64 % | all |
| F10 | loss drama (probe fixture, seeds 0–7 with `debug_kill` of the doorway holder at day 2 before any teaching): `KnowledgeLost` fires within 240 ticks, the HUD/why lines say it, and a Door order waits with `NoPractitioner` | 8/8 |
| F11 | ledger per kind exact (incl. Clay/RawPot/ClayPot, broken axes); 0 deaths in 36 days; soak green | all |
| F12 | knowledge systems ≤ 150 µs/tick at N = 1,000; C3 ceilings; regression vs the quiet M3 row ≤ +10 % (bench) ; baseline identity; replay byte-identical | all |

# 11. PART 6 — Docs, review, gate, report
- Docs: CHANGELOG; ROADMAP (M4 done; 다음 → M5 or M-ART2 per the art state; recommend); HISTORY; ADR 0024–0026 final; ARCHITECTURE (system table, §11 knowledge: data flow from defs → component → gate → channels → aggregate → HUD; schema v4); `snapshot_schema.md`; `PERF_LEDGER.md` (quiet rows); skills (`worldsim-architecture`: "add a knowledge node", "add a gate", channel rules, the practitioner bitset; `worldsim-testing`: knowledge fixtures and `debug_kill`; pitfalls: the contended-ledger lesson).
- Review: two fresh subagents — code (gating completeness: every recipe/blueprint/designation either gated or listed as universal in DECISIONS; determinism of oral pairing and discovery under keyed RNG; decay/learn paths refresh the bitset; teach job cancellation paths; ledger across clay/pot/axe-break; seam ≤ +20 on `decision/system.rs`); numbers (F2, F3, F6, F7, F8 recomputed from raw).
- 12.3 Screen checks (windowed): the tech map with at least one node in each of 미지/위험/안정; the teach dialog; the inspector 지식 tab; the HUD era line; a fired pot in a storehouse; a ford. Shots to `$DATA/shots/`.
- Gate (28 steps: the ledger rule change, and a `knowledge` assertion in the soak: 0 `KnowledgeLost` in 20 days) green on a clean commit → fast-forward → report commit → docs step → fast-forward.

# 12. Not in M4
Records/writing, schools, Henrich critical-population formula (M5+), knowledge distortion in oral transmission, farming, weaving, hunting (tracking is lore only), trade of knowledge with neighbours, the 169-node V7 tech list (only this 24-node map), knowledge heatmap overlay, children's play learning.

# 13. 인게임 확인사항 (오너용)
1. 시작 직후 상단에 "구석기 0 %"와 "⚠ 기술 위험 2"가 보인다. T를 누르면 기술 지도가 열리고, 문 만들기·가죽 손질이 빨강(1명만 앎)이다.
2. 빨간 노드를 눌러 "가르치기" → 스승·제자를 고르면 두 사람이 작업장에서 만나 가르치고, 며칠 뒤 제자의 지식 탭에 "문 만들기 · 숙련"이 생기며 노드가 노랑/초록으로 바뀐다.
3. 저녁 모닥불에 모인 뒤 아침에 몇 사람의 지식 탭에 "돌 깨기 · 배우는 중 (구전)"이 새로 보인다.
4. 돌을 많이 쌓아 두면 어느 날 연대기에 "Fox가 간석기를 발견했다!"가 뜨고, 갈돌 작업장을 지을 수 있게 된다. 간 도끼로 베면 더 빠르다.
5. 강가에서 점토를 파고(F7-2), 토기대에서 빚고, 모닥불에서 구우면 창고에 토기가 쌓이고 상단 진척도가 "신석기 전환기 n %"로 바뀐다.
6. 도끼는 40번 베면 부러진다(연대기 "도끼가 부러졌다"). 날을 계속 만들어야 하고, 같은 사람이 계속 만들면 품질이 오른다.
7. 오래 안 쓴 기술은 지식 탭에서 "잊어 감"으로 내려가고, 결국 그 일을 못 받는다. 유일한 보유자가 죽으면 "기술 소실" 토스트와 검은 테두리 노드가 나온다.
8. 강에 여울이 생겨 건너편 숲을 벨 수 있고, 닿지 않는 곳을 표시하면 "닿을 수 없음" 토스트가 뜬다.
9. 36일을 돌려도 FPS·틱 시간이 M3와 다르지 않다.

# 14. 구현 완료 보고 (한국어; `docs/v8/m4/M4_REPORT.md`와 최종 메시지)
```
## 구현 완료 보고 — M4
### 구현 의도
### Part 0 기록 (장부 규칙과 조용한 재측정 결과 M2→M3 실제 비율, 여울, 도구 마모, M3 결정 기록, 아트 상태)
### 구현 내용 (지식 데이터 24노드, 컴포넌트·게이팅, 채널 3~4종, 망각, 발견, 집계·시대, 점토·토기·간석기 사슬, UI — 파일별)
### 구현 방법 (비트셋 게이트, 키 난수 스트림, 감쇠 표, 발견 확률, 진척도 식, 결정 틱 접점 줄 수)
### 성능 계획과 실제 (지식 시스템 µs, JobAssign 증가분, 바이트/사람, 벤치 장부)
### 평가 결과 (F1–F12 표 + 시드별 부록, 토기 사슬 정지 지점 분포)
### 보정 기록 (라운드별 바꾼 수치와 이유)
### 변경된 파일 목록 (커밋별)
### 리뷰 결과
### 확인된 제한사항
### 오너 결정 필요 (다음: M5 vs M-ART2; 망각 속도 체감; 발견 빈도; 가르치기 UI 흐름)
### Gate 결과
git ls-remote 결과
```
