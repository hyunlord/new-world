# GOAL: WorldSim — M5.5: neighbours and the M5 corrections — migrants who bring their knowledge, seasonal traders whose goods give technical hints and who buy the colony's surplus; drafting that matters (the undrafted flee); pressures that do not stack into a wipe-out; mood and quarrels tuned to be felt; meat carried home; the planner's impossible bars rewritten with their arithmetic; ADR 0016 amended for the memory instrument.

Repository `hyunlord/new-world`. Base: **`v8/main` = `cb58a801d656291a04a5ebfb3be9805845ee6433`** (M5 report; gate green at `ef1bf3b7`). `v8/wip` at the same commit.

Executed by **Claude Code**, unattended, in `/Users/rexxa/work/new-world-pass4`. No DGX. Headless except 12.3. Target 22 hours, hard stop 26. The art session runs ART-STRUCT on `v8/art-struct`; do not touch `art/**`, `docs/v8/art*/**`, do not merge art branches.

## Why this pass exists

VISION §7 item 7 and §10.2: "이웃은 지식의 통로다." Ten founders alone cannot hold a growing technology (Henrich 2004); in Dwarf Fortress migrants arrive in waves, in RimWorld traders arrive on storyteller dates. Neighbours exist off-screen as aggregates, send migrants who carry their own knowledge, and send traders whose artifacts are clues to technologies the colony lacks (Civilization's eureka). No ROADMAP stage held this yet; it belongs before M6.

M5 (`ef1bf3b7`) also showed five design faults, and four of its failed bars were the planner's arithmetic:
- **F8** (skill growth): 8-use knives and "≥ 10 blades consumed" needed 80 knife uses; worlds had 6–22. Third pass in a row the blade sink was mis-sized. Traders that buy blades are the natural sink.
- **F5** (apparel): a wrap needs 2 cured hides, so 4 wraps by day 30 need 8 kills when 48 days yield ~6; and exposure ×700 ‰ cannot make wearers' cold "≤ half" (expected 0.7).
- **F6b** (mood): the common worst day sums to −430; pressure hits bodies, not minds.
- **F7** (quarrels): 7 worlds under 2/year; and the cause-chain check could not fail.
- Real faults: drafting barely matters (undrafted worlds beat all 100 packs, 3 deaths); seeds 804 and 814 nearly wiped out by a cold snap stacked with illness on zero firewood; meat rots at the kill site waiting for haulers (F4 27/32).
- **C3 memory instrument:** D19 moved the ceilings from RSS to the allocator counter without amending ADR 0016, and the change was made by stopping a running gate. Defensible (RSS on macOS reads 83–118 B/tile for one binary), but the contract must say it and RSS must keep an outer bound — fragmentation is real memory.

---

# 0. RULES
- AGENTS.md and the skills (`worldsim-architecture` first). Commit to `v8/wip` (`v8-m55(pN): …`), `v8/main` by fast-forward after a green gate. Never `lead/main`, `main`, `codex/*`, `v8/art-*`; no force-push; explicit paths; hook/ignore files and the untracked `.ignore`, `docs/revitalize/`, `.gitignore` change untouched.
- **Never ask the owner** → `docs/v8/m5_5/M55_DECISIONS.md`. Never change a gate's instrument while a gate runs; stop, record, then change.
- C3 ceilings and the regression rule hard. `decision/system.rs` ≤ +20 lines.
- Parts: 0 = §5 · 1 = §6 (corrections) · 2 = §7 (neighbours, migrants) · 3 = §8 (traders, eureka) · 4 = §9 (UI) · 5 = §10 (evaluation) · 6 = §11 (docs, review, gate, report). Push after every Part.
- **Calibration** on seeds 0–7 only, ≤ 8 rounds, each logged; this pass **may** change the M5 mood modifier table, the breakdown threshold and the quarrel formula's constants (declared in DECISIONS with before/after); held-out seeds 900–931 are measured once at the end and once after review fixes, never used to tune.
- **Time rule:** past 26 h, finish the step, push, go to Part 6.
- **Drop order:** 8.4 gifts → 9.5 neighbour rumours on the tech map → 7.5 migrant decline consequences → trader haggling text → year summary additions.
- **Never drop:** Part 0, all §6 corrections, two neighbours with migrants (accept/decline) and seasonal traders with barter and eureka, the evaluation, identity, docs, report.

# 1. MACHINE
Pass4 checkout + base worktree `../new-world-m55-base` at `cb58a801`. Data `/Users/rexxa/work/m55-data/`. `CARGO_BUILD_JOBS=8`, tests `--test-threads=8`, probe `--workers 6`.

# 2. CONTEXT (planner's survey at `cb58a801`; verify)
- **Story:** `sim-core/src/colony/story.rs` (`PressureKind`), `runtime/story/` (StorytellerSystem 170), mercy/gap rules, illness with immunity (M5 D17).
- **Wildlife:** `sim-core/src/colony/wildlife.rs`, `rules/wildlife.rs` (`knife_uses: 8`), `runtime/wildlife/`, `runtime/jobs/hunt.rs` (butchering wears the knife at kill), drafting commands, `CombatSystem` 137 pairs reused for animals.
- **Recipes:** `scrape_hide` (Hide 1 → CuredHide, knife), `make_hide_wrap` (**CuredHide 2** → HideWrap), `make_fur_cloak` (WolfHide 2); hourly dressing pass.
- **Mood:** `runtime/mood/` (MoodSystem at minute 7, morning hand-over in the 8 o'clock hour), modifier table (M5 §3.4), thresholds 250 / 100 / 750; QuarrelSystem in five slices every 6 ticks, nearest 8.
- **V7 leftovers to inspect, not to revive blindly:** `components/settlement_migrant.rs`, `runtime/settlement/` (V7 migration/fission). Colony code must not depend on them unless they fit; note in DECISIONS.
- **Ledger:** `bench_ledger.py check --pass/--base`, `--justified-scenes`, `append --gate-log`; allocator column; D19 ceilings on the allocator counter.
- **UI keys in use:** A C D H K L M P Q R S T W, arrows, Home, F1–F10, Space. Free: **N** (neighbours).
- **Art state:** route I adopted (ART-PROD3, 34/36); ENV1 PARTIAL (rehearsal compositor, owner: structures unnatural — mixed projection); ART-STRUCT running (one projection, code-drawn walls/edges).

# 3. DESIGN

## 3.1 Corrections (ADR 0027–0030 amendments)
1. **No stacked pressure:** a minor pressure (illness) never starts during a major or within 2 days after one ends; a major never starts while ≥ 2 people are sick. Record in ADR 0027.
2. **Drafting matters** (ADR 0028): `FleeSystem` priority 123 (before decision 125), interval 3: an undrafted person within 8 tiles of a hunting/attacking wolf gets `Fleeing { until }` (30 ticks, renewed while a wolf is within 8) and a path toward the nearest roofed room, else the hearth, else away from the wolf; decision skips fleeing people (seam lines counted). A wolf adjacent to a fleeing person attacks; the person defends at ×500 ‰ damage. Drafted people fight at full damage with weapons as M5. Wolf strength unchanged unless calibration needs it (record).
3. **Meat home:** a successful hunt chains a `Haul` of the meat (and hide) by the hunter to the nearest stockpile cell that accepts food, before any other job (the hunter carries up to 3 meat + 1 hide; extra stays and gets a normal haul job at haul priority + 1).
4. **Hide wrap = 1 cured hide** (recipe change; fur cloak stays 2 wolf hides). Arithmetic: 4 wraps need 4 kills.
5. **Knife = 4 uses** (`knife_uses: 4`).
6. **Mood and quarrels felt** (ADR 0030): calibrate the modifier table and thresholds so that, on seeds 0–7 with the storyteller on, the mean breakdowns per world over 2 years is 0.5–1.5 and quarrels per world-year 3–8, while with the storyteller off breakdowns per world ≤ 0.25. Allowed knobs: pressure-related modifiers (cold night, hungry, sick, pressure active, kin died) up to ×2, breakdown threshold 100 → up to 150, tantrum threshold 250 → up to 300, `QUARREL_PERMILLE` constants. Every change in DECISIONS with the round it came from.
7. **Quarrel cause chains that can fail:** each quarrel stores its cause as a reference to a concrete record — a chronicle entry, a pressure record, or a need-state sample (hunger/thirst/cold value at the quarrel tick); the evaluation checks that the referenced record exists, belongs to the initiator (or the world, for pressures), and lies within the 2 days before the quarrel.

## 3.2 Neighbours (ADR 0031)
- `SimResources.neighbours: Vec<Neighbour>` — exactly **2** per world, generated from the seed (`STREAM_NEIGHBOURS`): `{ id, name_key, edge: N|E|S|W (different), distance_days: 2–5, population: 20–60, attitude: 0–100 (start 40–60), known: bitset of knowledge nodes (all Paleolithic + 2–4 Neolithic nodes, keyed, at least one of the pottery chain in at least one neighbour), last_contact_day }`. Off-screen aggregate: once a day `NeighbourSystem` 174 drifts population ±1 % and nothing else (no simulation of their people).
- **Migrant waves:** wave 1 on day 8–12 (2–4 people), wave 2 on day 20–26 (2–6); then one roll per season on day 2–5: chance ‰ = min(800, 50·food_surplus_days + stockpile_value/50 + attitude·2) where `food_surplus_days` = stored food / (adults × daily need); size 1–4; **population cap 40** (no wave when adults + children ≥ 40). A wave comes from the neighbour with the higher attitude (ties by id). Migrants spawn at that neighbour's map edge, walk to the hearth (nav), and wait 1 day for the player: command `AcceptMigrants { wave }` / `DeclineMigrants { wave }` (recorded). Accepted: they join (households, beds, priorities default, apparel from their own items: 50 % arrive with a HideWrap). Declined or unanswered after 1 day: they walk away; attitude −5 (7.5, droppable consequence).
- **Migrant knowledge:** each migrant gets all Paleolithic nodes their neighbour knows at 450–700 ‰ (keyed), and each of the neighbour's Neolithic nodes with 50 % chance at 420–650 ‰ (≥ the node's threshold 400 in ≥ 50 % of cases → some arrive as practitioners). Source = `Migrant { neighbour }` (new knowledge source; floor as Apprenticed).
- Chronicle: `MigrantsArrived { wave, neighbour, count }`, `MigrantsJoined`, `MigrantsDeclined`.

## 3.3 Traders and eureka (ADR 0032)
- **Caravans:** one candidate date per season (day 2–5, keyed) from a neighbour with attitude ≥ 30 (the higher one; alternate if both); never during a major pressure (shift to its end + 1 day; if past the season, skip); not before day 6. 2 traders + goods; walk from their edge to the hearth; stay **1 day**; leave.
- **Goods:** 6–10 item stacks drawn from a table weighted toward what the colony lacks: items of nodes the neighbour knows and the colony does not (ClayPot, GroundAxe, CuredHide, Spear, HideWrap, FurCloak) weight 3; staples (Meat, Flint, Clay, Hide, Firewood) weight 1; quality 30–70.
- **Value table** (`sim-core/data/colony/values.ron`): base value per `ItemKind` (Log 2, Firewood 1, Stone 1, Flint 3, FlintBlade 6, StoneAxe 14, GroundAxe 24, Spear 16, Hide 5, CuredHide 9, HideWrap 14, FurCloak 30, WolfHide 8, Clay 2, RawPot 5, ClayPot 12, Meat 3) × (500 + 5·quality) / 1000, rounded, min 1. Traders buy any item at 80 % and sell at 120 % of value.
- **Barter:** command `Trade { caravan, give: Vec<ItemId>, take: Vec<(stack, count)> }` (recorded); accepted iff value(give)·0.8 ≥ value(take)·1.2; items move through haul jobs to/from the trader's spot (a temporary trade zone at the hearth), ledger exact (`traded_out`, `traded_in` added to the ledger identity). Attitude +3 per trade; **gift** (give without take): +1 per 10 value (8.4, droppable).
- **Eureka:** while the colony holds an artifact whose producing knowledge no colonist knows, `DiscoverySystem` multiplies the chance for that node by **×3** for adults who meet its prerequisites, and the node's `discover` stockpile condition counts as met. Chronicle `Eureka { knowledge, artifact }` on such a discovery. The tech map node detail shows "이웃 ○○가 앎 (소문)" for nodes known by a neighbour (9.5, droppable).
- Chronicle: `TraderArrived`, `Traded { give_value, take_value }`, `TraderLeft`.

## 3.4 ADR 0016 amendment (Part 0)
C3 memory clause: the per-agent and per-tile ceilings are judged on the **allocator counter** (deterministic) when the run has it; **RSS keeps an outer bound**: the median of 5 repeats ≤ 5,120 B/agent and ≤ 128 B/tile, reported every pass and failing the gate if exceeded. Record why (D19 numbers) and that gates never change instruments mid-run.

---

# 4. PERFORMANCE PLAN
NeighbourSystem daily; migrants are ordinary people after acceptance; caravans ≤ 6 extra entities for one day; FleeSystem interval 3 queries wolves (≤ 4) × people in 8 tiles through the chunk index: ≤ 15 µs/tick at N = 1,000 with a pack present. Budget for all M5.5 systems ≤ 60 µs/tick at N = 1,000 (bench has no wolves; add a `wolves` bench line: N = 1,000 with a 4-wolf pack for 2,880 ticks). alloc/agent ≤ M5 + 32 B.

# 5. PART 0 — Records and amendments
1. Commit this goal as `docs/v8/m5_5/M55_GOAL.md`. ROADMAP gets a row **M5.5 이웃과 보정** before M6.
2. Art record: ROADMAP/HISTORY/ADR 0018 — ENV1 PARTIAL (owner: structures unnatural), ART-STRUCT running; route I adopted.
3. ADR 0016 amendment (§3.4) with the RSS outer bound added to `bench_ledger.py` and the gate summary.
4. M5 owner decisions recorded (D-numbered): no pressure overlap; undrafted flee; hunter carries meat; knife 4 uses; mood/quarrel tuning allowed within §3.1.6 limits; deer art yes, wolf art pending.
5. ADR 0031, 0032 drafted; amendments to 0027, 0028, 0029, 0030 drafted. Base data → `$DATA/base/`.
Push.

# 6. PART 1 — Corrections
§3.1. Tests: no illness starts during or within 2 days after a major (seeds 0–7 × 2 years); a major waits while ≥ 2 sick; fleeing people move away within 30 ticks and toward a roofed room; drafted kill rates as M5; meat reaches a stockpile the same hour as the kill; hide wrap from one cured hide; knife breaks after 4 uses; quarrel cause references resolve (unit + probe); calibration rounds logged.

# 7. PART 2 — Neighbours and migrants
§3.2. Tests: two neighbours per world on different edges, deterministic; waves on their windows; cap 40 respected; accepted migrants join households, get beds, apparel, priorities; their knowledge seeded per §3.2 with `Migrant` source and floors; declined waves leave and attitude drops; save v6 round-trip; identity.

# 8. PART 3 — Traders and eureka
§3.3. Tests: caravan dates never in a major pressure and never before day 6; barter acceptance rule; ledger exact through `traded_out/in` and haul paths; attitude changes; eureka triples the chance (unit) and a scripted world with a ClayPot and no potter discovers pot_forming faster than without (probe, seeds 0–7, median days); chronicle entries.

# 9. PART 4 — UI
1. **Migrant dialog** on arrival (pauses at 1×, not at higher speeds unless a setting says so): each person's name, sex, age, top 3 knowledge with ‰ and "숙련" marks, apparel; 받아들이기 / 돌려보내기; a HUD banner with the countdown.
2. **Trade dialog** when the player clicks the caravan or presses the banner: two columns (우리 비축 / 그들의 물건) with counts and values, a balance bar (주는 값 × 0.8 vs 받는 값 × 1.2), 거래 / 선물 / 닫기; items unknown to the colony marked "처음 보는 물건 — 단서".
3. **N — neighbours panel:** both neighbours: name, direction, distance, population, attitude bar, last contact, rumoured Neolithic knowledge.
4. **Tech map:** "이웃 ○○가 앎 (소문)" in node detail; eureka glyph on nodes with an artifact present.
5. **Flee/draft:** a fleeing person shows a small "!" and the inspector line "도망 중: 늑대"; the drafted list unchanged.
6. Chronicle lines for the new kinds; year summary adds migrants joined, trades, eurekas.
7. Localization keys in `docs/v8/m5_5/M55_LOCALE.md` (≥ 60 expected).

# 10. PART 5 — Evaluation (held-out seeds 900–931; 48 days)
**M55-U script:** M5-U's orders (house, storehouse, teach, spears, hunt zone, wraps), `Maintain 12 FlintBlade`; **at every caravan:** sell all FlintBlades above 4 and take the highest-value items the balance allows, preferring unknown artifacts; **accept every migrant wave**; at every wolf spawn draft 4 adults with spears (as M5). Controls: **no-draft** (same script, never draft), **story-off** (storyteller disabled), **no-neighbours** (neighbours disabled) — each on the same 32 seeds.

Bars (arithmetic stated where a bar depends on rates):

| # | bar | pass |
|---|---|---|
| F1 | no minor pressure during or within 2 days after a major in any world; non-wolf pressure deaths ≤ 2 per world in 2 years; no world loses ≥ 50 % of its people | 32/32 · ≥ 30/32 · 32/32 |
| F2 | drafting matters: draft script 0 wolf deaths in ≥ 28/32; no-draft control ≥ 1 wolf death or a person below 50 hp in ≥ 20/32; ≥ 80 % of undrafted adults within 8 tiles of a wolf start moving away within 30 ticks (pooled) | all three |
| F3 | hunting: ≥ 6 kills and ≥ 12 meat eaten by day 48, meat spoiled before reaching a stockpile ≤ 10 % of meat dropped | ≥ 28/32 |
| F4 | apparel: ≥ 3 wearers by day 30 and ≥ 5 by day 48 (1 hide per wrap; M5 rates ≈ 1 kill / 7 days → ~4 by day 30, ~6–7 by 48, plus traded hides/wraps); wearers' mean night cold exposure ≤ 0.8 × non-wearers' (model 0.7) | ≥ 24/32 |
| F5 | mood: with pressure, ≥ 1 breakdown in ≥ 8/32 worlds and ≤ 3 in every world; story-off ≤ 1 per world in ≥ 28/32 | both |
| F6 | quarrels: ≥ 4 per world over 2 years in ≥ 28/32; fights ≤ quarrels / 4; 0 quarrel deaths; ≥ 90 % of quarrels' cause references resolve per §3.1.7 | all |
| F7 | migrants: waves 1 and 2 arrive in their windows in 32/32; population at day 48 in 16–40 in ≥ 30/32; ≥ 1 Neolithic node arrives with a migrant practitioner in ≥ 20/32 | all |
| F8 | traders: ≥ 6 caravans in 2 years (8 seasons, minus those that collide with majors) in ≥ 28/32; ≥ 1 unknown artifact bought in ≥ 28/32; eureka discoveries reported (no bar) with the no-neighbours control's discovery days for comparison | both bars |
| F9 | skill growth, with the trade sink: blades crafted days 37–48 ≥ 5; top knapper level ≥ 6 (XP 630 ticks ≈ 12 blades at 45–63 ticks); mean quality of blades crafted days 37–48 − days 1–12 ≥ 8 (30 + 3L + 2.5: level ~1 → ~35.5, level ≥ 6 → ≥ 50.5) | ≥ 24/32 |
| F10 | demographics unchanged in story-off + no-neighbours: births/woman-year 0.20–0.30; ledger per kind exact incl. traded_in/out; reservations; soak and story fixtures green | all |
| F11 | M5.5 systems ≤ 60 µs/tick at N = 1,000 (+ the wolves bench line); alloc/agent ≤ M5 + 32 B; RSS outer bound; C3; regression vs M5 quiet rows; identity; replay | all |

# 11. PART 6 — Docs, review, gate, report
- Docs: CHANGELOG; ROADMAP (M5.5 done; 다음 → M-ART2 when ART-STRUCT is approved by the owner, else M6 prep); HISTORY; ADR 0016 amendment, 0027–0030 amendments, 0031–0032 final; ARCHITECTURE (systems 123, 174; §13 neighbours and trade data flow; save v6); snapshot schema v6; PERF_LEDGER; skills (`worldsim-architecture`: "add a neighbour event", the trade ledger, the eureka rule; `worldsim-testing`: controls and the arithmetic-before-bars rule — a bar that depends on rates states its expected value; pitfalls: never change an instrument mid-gate).
- Review: two fresh subagents — code (flee seam and cancellation; migrant joining paths: households, beds, knowledge floors; barter ledger; eureka determinism; caravan scheduling vs pressures); numbers (F2, F4, F5, F6, F9 recomputed from raw).
- 12.3 Screens (windowed): migrant dialog, trade dialog with the balance bar, neighbours panel, a fleeing villager with "!", a eureka chronicle line, tech map with a rumour → `docs/images/m55_contact_sheet.png`.
- Gate (30 steps: + `neighbours` fixture: 2 seeds × 48 days with neighbours on, waves accepted, one trade, ledger exact) green on a clean commit → fast-forward → report → ledger append with the gate log → docs → fast-forward.

# 12. Not in M5.5
Raids, faction relations beyond attitude, expeditions, marriage exchange, migrants' families and histories beyond knowledge and apparel, currency, prices that move, Henrich critical-population formula.

# 13. 인게임 확인사항 (오너용)
1. 8~12일째에 "이주민 3명이 왔다" 창이 뜬다. 각자 아는 기술이 보이고, 받아들이면 마을 사람이 된다. 토기를 아는 이주민이 들어오면 기술 지도의 토기 노드가 바로 밝아진다.
2. 계절마다 교역 무리가 와서 하루 머문다. 클릭하면 거래 창이 열리고, 날을 팔아 토기나 간 도끼를 살 수 있다. 처음 보는 물건을 사면 그 기술을 발견할 확률이 오른다(연대기 "단서").
3. N을 누르면 이웃 둘의 방향·인구·태도와 그들이 안다는 기술(소문)이 보인다.
4. 늑대가 오면 소집 안 한 사람은 "!"를 띄우고 방이나 모닥불로 도망친다. 소집해서 창으로 막지 않으면 다치거나 죽는다.
5. 한파와 병이 겹치지 않는다. 사냥꾼은 고기를 바로 들고 온다.
6. 겨울에 기분이 눈에 띄게 내려가고, 가끔 누군가 실의에 빠진다. 다툼은 1년에 몇 번 있고, "왜?"가 실제 원인을 가리킨다.
7. 날을 팔기 위해 계속 만들어서, 2년째엔 날 품질이 눈에 띄게 오른다.
8. 48일을 돌려도 FPS·틱 시간이 M5와 다르지 않다.

# 14. 구현 완료 보고 (한국어; `docs/v8/m5_5/M55_REPORT.md`와 최종 메시지)
```
## 구현 완료 보고 — M5.5
### 구현 의도
### Part 0 기록 (ADR 0016 개정, 아트 상태, M5 결정 기록)
### 보정 (§3.1 일곱 항목 — 각각 전후 수치)
### 구현 내용 (이웃·이주·교역·유레카·도망, UI — 파일별)
### 구현 방법 (이웃 생성, 파도 확률식, 가치표와 물물교환 규칙, 유레카, 도망 접점 줄 수)
### 성능 계획과 실제
### 평가 결과 (F1–F11 표 + 시드별 부록, 대조군 셋과 비교)
### 보정 기록 (라운드별 바꾼 수치와 이유)
### 변경된 파일 목록 (커밋별)
### 리뷰 결과
### 확인된 제한사항
### 오너 결정 필요 (다음: M-ART2 vs M6; 인구 상한; 교역 빈도; 이주 거절의 대가)
### Gate 결과
git ls-remote 결과
```
