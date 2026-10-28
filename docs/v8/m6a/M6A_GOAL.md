# GOAL: WorldSim — M6a (Codex): the food chain — wild plants as data with seasons, harvested food as items, cooking at the hearth, food variety, spoilage and preservation, and a village that plans for winter — so a colony survives several years on food it gathered, cooked and stored, with every step visible and caused. First of two Neolithic passes (M6b: farming, weaving, room temperature).

Repository `hyunlord/new-world`, local clone `/Users/rexxa/work/new-world-pass4`. Base: **`origin/v8/main` after M-UX2.1 is published** — published 2026-10-29 as `5b0c88812374b87e7854cc9203f144ad0ccd9009` (Part 0 checks). Executed by **Codex (GPT Astra)**, unattended. Target 20 hours, hard stop 30.

## Why
The depth roadmap (`docs/v8/DEPTH_ROADMAP.md`, owner-approved 2026-10-26) puts M6 Neolithic next: "몇 년을 살아남는 마을, 첫 밭, 첫 천 옷". The planner splits it: **M6a food chain** (this goal) then **M6b farming, weaving, room temperature** (Appendix B; a later goal). Food first, because farming needs crops to be food items, and winter needs stores.

Today (verified at `3814828c`):
- Plant food is V7's abstract layer: `food_tiles` / `food_source_max` counters with seasonal regrowth (`runtime/resource_regen`), eaten at the bush or carried as `stock_in_hand` into a settlement store (`runtime/settlement/{food_economy,stockpile_deposit,stockpile_consume}`). The colony has **no plant food items**; `items.ron` has 17 items and only `meat` carries a `FoodDef` (units, spoil days).
- The colony mind (`mind.ron`, 157 nodes) eats through ported V7 behaviours: `eat` (17 nodes), `forage` (6), `gather` (6), `hearth_meal` (4), reading food facts in `runtime/mind/{facts,context,behaviours/*}`.
- No cooking, no nutrition, no preservation buildings. Era progress is capped at 643 ‰ because the farming and weaving markers do not exist (`knowledge/aggregate.rs`).
- Calendar: seasons of 6 days, a year of 24 days; winter is season index 3.

The roadmap principles bind this pass: a new system must take input from ≥ 2 systems and feed ≥ 2; everything visible (inspector, chronicle, "왜?"); every effect points at its cause; bars come from arithmetic; content is data, behaviour is code (ADR 0033); colony only — baseline stays byte-identical.

# 0. RULES
- **Start check:** `git -C /Users/rexxa/work/new-world-pass4 fetch origin`; `origin/v8/main` must contain `docs/v8/mux2_1/MUX21_REPORT.md`; record its SHA as **BASE**. If not, stop and report "M-UX2.1 not published" (it measures FPS on this Mac; two measuring jobs at once invalidate both).
- **Isolation:** own worktree `/Users/rexxa/work/new-world-m6a-dev`, branch `v8/m6a` from BASE (`git -C /Users/rexxa/work/new-world-pass4 worktree add -b v8/m6a /Users/rexxa/work/new-world-m6a-dev <BASE>`; `…-2` if the path exists and is not a registered worktree). Never edit/build/run in `/Users/rexxa/work/new-world-pass4`, `/Users/rexxa/work/new-world-play2-dev`, or another session's worktree; never kill or signal other processes. Commit this brief first as `docs/v8/m6a/M6A_GOAL.md` (from `~/Downloads/codex_goal_worldsim_m6a.md`).
- **Roadmap docs, your first commit after M6A_GOAL.md (docs only, owner-approved 2026-10-28/29):** commit `~/Downloads/DEPTH_ROADMAP_v2.md` as `docs/v8/DEPTH_ROADMAP.md` (replacing v1; keep the v1 text under a final section "v1 (2026-10-26)"). Update `docs/ROADMAP.md` to its §5: M-UX2.1 done with `5b0c8881`, M6a in progress, M-READ 읽기 (row 2.5, after M6a), M6b, M7–M11, "UX·아트 후보", and the "다음" line. Point `docs/VISION.md` §7 at the v2 document. Add one `docs/HISTORY.md` line. Add the new process rules of its §7 (one measuring job on this Mac at a time; temporary pass issue IDs renumbered to the next free ISS number at publish; doc conflicts keep both sides, code/data conflicts stop; a red gate means fix and re-run; the held-out seed register) to `AGENTS.md` and `.claude/skills/worldsim-docs-upkeep/SKILL.md`. After publishing, edit GitHub issue #649 so its document line links `docs/v8/DEPTH_ROADMAP.md` at the published SHA.
- **Issue IDs:** new ledger rows use temporary IDs (M6A-1, …) until you publish, then the next free ISS numbers.
- **Git:** commits `v8-m6a(pN): …`, explicit paths. A red gate means diagnose, fix, re-run. Publish after a full gate with status 0: `git push origin v8/m6a:v8/wip` then `:v8/main` (fast-forwards). If refused because `v8/main` advanced, rebase only without conflicts, re-run the full gate, push; on a conflict, stop and report. Never force; never push `lead/main`, `main`, `codex/*`, `v8/art-*`.
- AGENTS.md, ADR 0016, ADR 0033, `docs/v8/core/MIND_AUTHORING.md` binding. Baseline identity byte-identical in every commit. Behaviour-preserving commits keep colony fixtures byte-identical; behaviour-changing commits are separate, predicted, measured, recorded.
- **Measurement:** timing runs start at 1-min load < 8 (record 1/5/15); old/new interleaved in one session; contended runs are INVALID and repeated (15-minute windows, up to 8).
- **Calibration:** seeds 0–7 only, ≤ 4 logged rounds; freeze policy and data before held-out; no tuning after.
- Records as you go: `docs/HISTORY.md`, `CHANGELOG.md`, `docs/ISSUES.md` + GitHub issues for rows you add (registry-created issues only; close fixed ones with the public commit link after publishing). DECISIONS in `docs/v8/m6a/M6A_DECISIONS.md`. Never ask the owner; questions go in the report.
- **Drop order:** drying rack (§2.5 keeps pot storage) → nut/root species beyond one each → food-group mood thoughts (keep the record and the inspector line) → UI food breakdown tooltip. **Never drop:** plants as data, food items and eating from them, cooking, spoilage, winter planning, held-out evaluation, the data-only tests, baseline identity, perf, docs, gate, report.

# 1. PART 1 — Design with arithmetic (before code) → `docs/v8/m6a/M6A_DESIGN.md`
1. **Inventory** every colony reader/writer of `food_tiles`, `food_source_max`, the settlement food store, `stock_in_hand`, `RESOURCE_SOURCE_INFINITE` for food, and every mind node/fact/behaviour that touches food (file:line, colony vs baseline). Verdict per row: replace / keep for baseline only / delete.
2. **Food arithmetic** from the current rules (cite the constants): hunger gain per tick and per day; units per meal; meals per adult-day. With the §2 data defaults compute: ripe units per plant per season; plants inside the forage radius on seeds 0–7 (mean, min); harvestable units per year; units needed per adult-year; **winter need** = adults × 6 days × units per adult-day; expected spoilage under each storage. Derive each bar in §5 from these numbers, write the derivation, and freeze them before calibration. The defaults below may change only with that written arithmetic.
3. ADR "food chain" (next free number): plants, food items, eating, cooking, spoilage, winter planning; what stays V7-only.

# 2. PART 2 — Build
## 2.1 Wild plants as data
- `sim-core/data/colony/plants.ron` (schema 1), species with: `id`, `name_key`, `placement` (terrain + elevation bands + weight + density ‰ per tile), `phases` per season index (`dormant` / `growing` / `ripe` / `spent`), `yield` (item id, units per ripe plant, regrow days within the ripe season), `harvest_ticks`, `knowledge` (node id that speeds or gates harvest), `tags` (category for fallback). Initial species (defaults; Part 1 may adjust with arithmetic): `berry_bush` (berries, ripe summer+autumn), `hazel` (hazelnuts, ripe autumn, keep long), `wild_grain` (wild_grain, ripe late summer; raw inedible), `wild_root` (roots, diggable spring–autumn), `wild_flax` (flax_stalks, not food; for M6b, placed now).
- Plant state lives in a tile-indexed table (not one ECS entity per plant), updated on season/day boundaries only; worldgen places plants in the colony world from data and the seed. The V7 food layer stays for the baseline world.
- Registry + resolver + category fallback; a **data-only test**: add a species (e.g. `wild_apple`) in a test copy of `plants.ron` and items.ron only, and show it is placed, ripens, is harvested, stored and eaten.

## 2.2 Food as items
- Extend `FoodDef` (data): `units` (satiety), `groups` (tags: fruit, nut, grain, root, meat, meal), `raw_edible` (bool), `spoil_days` per storage class (`ground`, `stockpile`, `roofed_store`, `pot`), `sick_permille` when eaten raw (meat only by default). New items: `berries`, `hazelnuts`, `wild_grain`, `roots`, `flax_stalks` (no food), `roast_meat`, `porridge`, `stew`, `dried_meat`, `dried_berries`.
- **Harvest is a job** (`jobs` layer): a `harvest` designation per plant (the player's order "채집", and the steward's), yields items on the ground, hauled by the existing haul; knowledge `foraging` speeds it.
- **Eating:** the `eat` / `hearth_meal` behaviours take food items from stockpiles or hands; choice by a data score (cooked > raw, a group the person has not eaten in the last 3 days, then nearest; inedible raw items never). Mind changes are `mind.ron` nodes + fact providers + behaviour target code per MIND_AUTHORING; record each new fact/behaviour in DECISIONS.
- Retire the colony's settlement food counter: the "days of food" alert and steward read the sum of food items' units.

## 2.3 Cooking
Recipes in `recipes.ron` at the hearth (a bill, data): `roast_meat` (1 meat), `porridge` (2 wild_grain; needs knowledge `grain_parching` or a clay pot), `stew` (1 meat + 1 roots, needs a clay pot), batch sizes and work ticks in data. New knowledge nodes in `knowledge.ron`: `cooking` (paleolithic, prereq fire_keeping), `grain_parching` (paleolithic, prereq foraging), `food_drying` (paleolithic). Cooked items: more units, no raw sickness. The steward keeps a standing cooking bill when raw food exists and a hearth is lit.

## 2.4 Variety
Each person keeps the groups eaten over the last 3 days (bounded, integer). Mood: two thoughts "여러 가지를 먹었다" (≥ 3 groups) / "같은 것만 먹었다" (1 group) as mood modifiers (the modifier table is still code — list the two variants as M7 debt in DECISIONS) and raw-meat sickness through the existing `Sick`. The inspector shows the last meals and groups.

## 2.5 Spoilage and preservation
Spoilage per stack by storage class each day (integer days remaining). A roofed storeroom (existing room role) and items stored in a clay pot (pot capacity in data) use their longer spoil days. **Drying rack** (blueprint in data, needs `food_drying`): turns meat/berries into dried items over days in data. Spoiled food becomes nothing (or `rot` if Part 1 chooses; record).

## 2.6 Winter planning (steward + alerts)
- Before winter the steward targets **stores ≥ winter need** (Part 1's formula) and raises harvest, drying and cooking priorities when below; the player can override; chronicle "겨울 준비: 저장 n일분 / 필요 m일분".
- Alerts: "겨울까지 n일, 저장 m일분" when stores < need; "음식이 썩고 있어요" when > 20 % of stored units will spoil within 2 days.

## 2.7 Visible and caused
- Inspector: food line (last meal, groups, hunger), "왜?" for eating choices (MindChoice already) and for spoilage (stack → storage class → days).
- Chronicle (player-facing kinds only, no mind_choice flood): first cooked meal, first dried store, first winter survived with stores, a death by hunger with its cause chain (no stores ← harvest shortfall ← reason).
- Renderer: plants per phase (ripe/dormant/spent) from art data if the ART manifest has them; otherwise a code-drawn placeholder with a tint per phase and an issue for the art line (no art generation in this goal).

# 3. PART 3 — Calibrate, freeze, evaluate
- Calibration seeds 0–7, 72 days (3 years), steward on, no player orders; ≤ 4 rounds; freeze.
- **Held-out seeds 1296–1327**, 72 days, steward on, no player orders. Bars (defaults; Part 1 replaces each number with its arithmetic, frozen before held-out):
  - village alive at day 72: ≥ 30/32;
  - no hunger death in winters 2 and 3: ≥ 28/32;
  - stores at the start of winters 2 and 3 ≥ winter need: ≥ 26/32 (denominator: worlds alive at that day);
  - share of meals eaten cooked from year 2 on: ≥ 50 % in ≥ 26/32;
  - median adult eats ≥ 3 groups per 3 days in summer of year 2: ≥ 26/32;
  - spoiled units ≤ 30 % of harvested units over 72 days: ≥ 26/32;
  - M-UX2/M-CORE survival rows (day-1 stockpile, beds by day 10, wolf deaths ≤ 1, ledger exact 32/32) do not drop by more than one world vs. BASE on the same seeds (run BASE as the comparison column; it lacks the food chain).
- Story yield (`worldsim_probe story-yield`) BASE vs M6a, reference only.

# 4. Perf, docs, gate, publish, report
- **Perf:** standing bench ≤ +10 % on every scene (no justification unless the scene's cause is written and the owner rule in the bench is met), C3 ceilings PASS. Plant updates only on day/season boundaries; spoilage daily; no per-tick plant scans.
- **Docs:** the ADR, `docs/ARCHITECTURE.md` (food chain section, system table), MIND_AUTHORING example for a food node, README "처음 할 일" (채집, 요리, 겨울 준비), `docs/ROADMAP.md` row M6a and a row M6b (Appendix B), lessons in skills.
- Gate (all current steps + the new data-only tests + a food fixture: 2 worlds × 72 days, ledger exact including food units, no negative stock) → publish → remove worktree if `git worktree remove` succeeds without `--force`.
- `docs/v8/m6a/M6A_REPORT.md` (Korean) and the final message:
```
## 구현 완료 보고 — M6a
### 설계 산수 (먹을 양, 겨울 필요량, 막대 유도)
### 바뀐 것과 남긴 것 (V7 먹이 층: 콜로니에서 대체, 기준선 유지)
### 식물 (종, 계절, 배치, 데이터만으로 종 추가 시험)
### 먹을 것 물건과 먹기 (고르는 규칙, 두뇌 노드·사실·행동 추가)
### 요리·다양성·썩음·보존
### 겨울 준비 (집사, 경고, 연대기)
### held-out 1296–1327 (M6a / BASE 표)
### 성능 (장면별 비율)
### 이야기 수확량
### 닫은 GitHub 이슈 / 새로 연 이슈
### 변경된 파일 목록 (커밋별)
### 확인된 제한사항
### 오너 결정 필요
### Gate 결과
### 오너가 할 일 (실행 한 줄: /Users/rexxa/work/new-world-play2-dev/tools/play.sh main)
git ls-remote 결과
```

# 5. 인게임 확인사항 (오너용)
1. 여름에 덤불에 열매가 달리고, 겨울에는 앙상해진다. 사람들이 따서 비축 구역으로 나른다.
2. 화덕에서 고기를 굽고 죽을 끓인다. 사람 정보창에 "마지막 식사: 구운 고기, 열매"가 보인다.
3. 겨울이 오기 전에 "겨울 준비: 저장 n일분 / 필요 m일분" 기록과 경고가 뜨고, 집사가 채집과 말리기를 늘린다.
4. 비축 구역에 오래 둔 열매는 썩고, 지붕 있는 창고나 토기에 넣은 것은 오래 간다.
5. 3년을 지나도 굶어 죽는 사람이 없다(또는 죽으면 연대기에서 "저장 부족 ← 채집 부족 ← 이유"로 따라갈 수 있다).

---

# Appendix B — M6b outline (next goal, not this one)
- **Farming:** wild_grain and later species domesticated by knowledge (`cultivation`, neolithic): a field zone, tilling, sowing seed items, growth by season in data, harvest, seed saving, simple soil fertility; era marker **farming** (weight 3).
- **Weaving:** flax_stalks → retting → spinning → loom (blueprint) → cloth → clothes (apparel data); era marker **weaving** (weight 2); clothes art goes to an art pass.
- **Room temperature:** outdoor temperature by season and pressure, per-room temperature from material insulation (the material system's data), hearth heat and doors; replaces the binary `Cold` marker; mood and exposure read temperature.
- Era progress ceiling lifts from 643 ‰ to 1,000 ‰.
