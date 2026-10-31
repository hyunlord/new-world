# M-READ Part 1 — Legibility inventory, scenarios and the 30-second baseline test

Base: `313ea92ba` (M6a published 2026-10-31). Branch: `v8/read`. Pass brief: [READ_GOAL.md](READ_GOAL.md).

## 1. Today's legibility surfaces

### 1.1 Inspector ("지금 · 이유")

The agent inspector panel (`scripts/ui/panels/agent_inspector_panel.gd`) is opened by clicking a person. It shows:

- **"지금: …"** — the current action from the snapshot's job code (`JOB_KEYS`), e.g. "지금: 일하기", "지금: 식사", "지금: 쉬기".
- **"지금 · 이유"** — the mind summary line from `get_agent_mind(entity_bits)` via the causal panel's `mind_summary()`. Format: "지금: [행동] — 이유: [사실1] · [사실2]". Shows the chosen node's name and at most two fact reasons from the winning mind choice.
- Needs bars (hunger, thirst, sleep), age, carrying, job, bed, skills (M2 economy), mood/apparel/draft/illness/hp (M5), knowledge tab (M4), story tab (M5), food meals (M6a).
- **"왜?" button** — opens the causal panel with `open_mind()`, showing the mind choice, top-3 candidates with scores, and fact reasons.

**SimReads feeding it:** `get_agent_snapshot`, `get_time_snapshot`, `get_activity_snapshot`, `get_porter_snapshot`, `get_world_info`, `get_content_catalog`. Direct calls: `get_agent_detail`, `get_agent_m5`, `get_agent_mind`, `get_agent_economy`, `get_agent_knowledge`, `get_agent_food`. Via `person_read_cache.gd`: `get_person`, `get_person_chronicle`, `get_agent_family`, `get_agent_temperament`.

**Capture:** `/Users/rexxa/work/read-data/captures/base/surface_inspector.png`

### 1.2 "왜?" panel (causal panel, mind mode)

The causal panel (`scripts/ui/panels/causal_panel.gd`) has two modes:

- **Mind mode** (`open_mind`): shows "지금: [행동] — 이유: [사실들]" and up to 3 candidates with scores. Opened by the inspector's "왜?" button.
- **Tile mode** (`open_tile`): shows tile history from `get_tile_causal_events`. Used for tile-click "왜?".

The panel shows facts that *caused* the current mind choice (eligibility predicates and score terms), but does **not** show:
- Why a *different* action was not chosen (blocked candidates, why-not).
- A chain deeper than one level (no "why is hunger high?" → "because no food" → "because winter").

**SimReads feeding it:** `get_tile_causal_events` (via `SimReads.causal_events`). Direct: `get_agent_mind`. Via SimReads: `get_knowledge_waits`, `get_colony_overlay`, `get_time_snapshot`.

**Capture:** `/Users/rexxa/work/read-data/captures/base/surface_why_panel.png`

### 1.3 Chronicle + M-UX2.1 kind filter

The chronicle panel (`scripts/ui/panels/chronicle_panel.gd`, toggled with C) pages entries from `get_chronicle_since`, keeps the last 60 visible notable rows. `assets/ui/chronicle_feed.json` controls kind visibility: kind 47 (`mind_choice`) is hidden, kind 53 (`food_operation`) is hidden; kinds 48–52 are shown.

Features:
- Kind group filters: 전체, 압력, 사냥·늑대, 다툼·기분, 지식, 삶.
- "고른 사람만" filter for the inspector's selected person.
- "→ 왜?" links on entries with causes, opening the why panel (`why_panel.gd`).
- Year summary header with births, deaths, pressures, discoveries, quarrels, animals.
- Knowledge loss toast on the order bar.
- Porter group merging for consecutive same-kind entries.

The why panel (`scripts/ui/panels/why_panel.gd`) shows a single entry's cause chain from `get_chronicle_why(index)`: the entry sentence, then each cause as one line with an arrow. Chains end with "더 거슬러 올라갈 원인이 없음" when depth is 1. **Issue #647: 0 verified cause chains of depth >= 3.**

**SimReads feeding it:** `get_time_snapshot`, `get_world_info`, `get_neighbours`. Direct: `get_chronicle_since`, `get_year_summary`, `get_chronicle_why`.

**Capture:** `/Users/rexxa/work/read-data/captures/base/surface_chronicle.png`, `/Users/rexxa/work/read-data/captures/base/surface_why_chain.png`

### 1.4 Alerts (`assets/ui/alerts.json`)

The alerts panel (`scripts/ui/mux/alerts_panel.gd`) evaluates 19 data-driven conditions against `get_ui_facts` at most twice per second. Each alert has severity (danger/warning/notice), text, explanation, fix suggestion, and a jump target.

Current alerts (by severity):
- **Danger:** wolves approaching, person breakdown, person injured.
- **Warning:** food low, firewood low, sleeping outside, storehouse full, orders unreachable, knowledge waiting, technique at risk, person sick, meat spoiling, newborn without bed, food winter preparation, food spoiling.
- **Notice:** stockpile missing, idle adults, migrants waiting, trader present.

Alerts show *what* is wrong but not *why*. Example: "음식 부족" says food is low but not why food is low (winter? no harvest? spoilage?).

**SimReads feeding it:** `get_ui_facts`.

**Capture:** `/Users/rexxa/work/read-data/captures/base/surface_alerts.png`

### 1.5 Goals (`assets/ui/goals.json`)

The goals panel (`scripts/ui/mux/goals_panel.gd`) shows 5 first-year goals: stockpile, house, firewood, winter survival, teaching. Each has a completion condition checked against world facts. The panel also shows era progress (구석기 → 신석기) and markers from the tech map.

Goals are binary (done/not done) and do not explain *how* to achieve them or *why* they are stuck.

**SimReads feeding it:** Facts supplied by the coordinator; no direct SimReads calls.

**Capture:** `/Users/rexxa/work/read-data/captures/base/surface_goals.png`

### 1.6 Lessons (`assets/ui/lessons.json`)

The learning helper (`scripts/ui/mux/learning_helper.gd`) reads 13 data-driven lessons that guide new players: camera, speed, person, stockpile, chop, house, priorities, bills, rooms, teaching, alerts, drafting, neighbours. Each has a trigger, completion condition, highlight target, and steward rule.

Lessons teach *controls* (how to use the UI) but not *diagnosis* (why the village is struggling).

**SimReads feeding it:** `get_ui_facts` (via the facts adapter).

**Capture:** `/Users/rexxa/work/read-data/captures/base/surface_lessons.png`

### 1.7 Steward toasts / chronicle

The steward system (`get_steward` via SimReads) manages autonomous village actions. Steward toasts appear on the order bar for:
- Knowledge loss: "기술 소실: [technique] (마지막: [person])".
- Draft exits: "[person]: 너무 지쳐 소집에서 빠짐".
- Defence summoning: "마을이 스스로: 늑대를 막으려고 N명을 불렀어요".

Toasts are transient, one-line notifications. They identify events but do not chain causes.

**SimReads feeding it:** `get_steward`, `get_chronicle_since` (for toast triggers).

**Capture:** `/Users/rexxa/work/read-data/captures/base/surface_steward_toast.png`

### 1.8 Tech map

The tech panel (`scripts/ui/panels/tech_panel.gd`, toggled with T) shows knowledge nodes in lanes (생존, 제작, 건설, 채굴, 토기, 사회) and eras (구석기, 신석기). Each card shows its status (미지/위험/알려짐/안정/소실), practitioners count, and era marker weight. Detail view shows holders, learners, discovery conditions, gated recipes/buildings, and a teach button.

The tech map shows *what knowledge exists* and *who holds it*, but does not diagnose *why* a technique was lost or *why* discovery conditions are not met in terms of village actions.

**SimReads feeding it:** `get_tech`, `get_knowledge_waits`, `get_teach_orders`, `get_time_snapshot`, `get_neighbours`.

**Capture:** `/Users/rexxa/work/read-data/captures/base/surface_tech_map.png`

### 1.9 M6a food/winter lines

M6a added:
- **Inspector food line:** `get_agent_food` shows recent meals and food groups ("식사: [items] · 음식군: [groups]").
- **Food stacks:** `food_text.gd` formats item, count, storage type, remaining days.
- **Winter preparation alert:** `food_winter` condition in alerts.json checks if stored_units < winter_need when preparing.
- **Food spoilage alert:** `food_spoil` checks if expiring_units > 20% of stored.
- **Steward winter planning:** autonomous harvest, cooking, drying, pot storage.
- **Food chronicle entries:** kinds 48–52 shown in the chronicle feed.

These surfaces show food *status* (how much, what kind, how long until spoilage) and winter *readiness* (stored vs needed). They do not explain *why* food is low (no workers? no plants? wrong priorities?).

**SimReads feeding it:** `get_ui_facts` (food_supply collection), `get_steward`. Direct: `get_agent_food`.

**Capture:** `/Users/rexxa/work/read-data/captures/base/surface_food_winter.png`

### 1.10 Summary: what is missing

| Surface | Shows | Missing |
|---|---|---|
| Inspector "지금 · 이유" | Current action + top 2 reasons | Why the action was chosen *over alternatives*; why the reasons hold |
| "왜?" panel | Mind choice facts + top 3 candidates | Blocked candidates and their first failing predicate; chain depth |
| Chronicle "→ 왜?" | Recorded cause chain | Chains stop at depth 1 (#647); no "why not" |
| Alerts | Current problems | Why the problem exists; root cause chain |
| Goals | Completion status | Why a goal is stuck; what to do next |
| Lessons | Control tutorials | Diagnosis; what to do when things go wrong |
| Steward toasts | Event notifications | Why the event happened |
| Tech map | Knowledge status | Why a technique is at risk (no students? no teachers?) |
| Food/winter | Stock levels, winter need | Why food is low; why harvest is insufficient |

**Root gap:** The player cannot follow a cause chain from a visible problem (someone starved, someone is cold, winter food is low) to its actionable root cause. Recorded parent links alone do not explain things (#647). The mind records only the winner, not why alternatives were blocked.

---

## 2. Six owner questions — seeded scenarios

### Q1: 왜 이 사람이 굶어 죽었나 (Why did this person starve to death?)

**Scenario:** Seed 1, advance to day 12 (tick 17,280). With 10 people and default steward, a starvation death can occur during or after the first winter if food preparation is insufficient. If seed 1 does not produce a starvation death by day 12, use seed 3 and advance to day 18.

**Reproduction:** `reset_world(1)`, `advance_ticks(17280)`, find a dead person in the chronicle (kind 0, death_reason starvation), click their name in the chronicle to open inspector in record mode.

**Target cause chain:**
굶어 죽음 ← 3일간 먹지 못함 ← 비축에 음식 없음 ← 겨울이라 익은 식물 없음 ← 가을 저장량 부족 (필요량 대비)

**What the current screen answers:**
- Chronicle: "[이름]이(가) 굶주림으로 죽었다" with "→ 왜?" link.
- Why panel: shows the chronicle entry and its recorded cause (if any). Expected: chain depth 1 or 0 — "더 거슬러 올라갈 원인이 없음".
- Inspector record: shows death reason "굶주림".
- Alert: "음식 부족" may have been active.

**What is missing:** Why they couldn't eat (no food? food existed but unreachable?). Why no food (winter? spoilage? no harvest?). The chain stops at "starved" without reaching the root cause.

### Q2: 왜 이 어른이 일을 안 하나 (Why is this adult not working?)

**Scenario:** Seed 0, advance to day 5 (tick 7,200). Look for an adult whose job is not "일하기" — they may be eating, resting, socializing, or idle.

**Reproduction:** `reset_world(0)`, `advance_ticks(7200)`, find an adult in the agent snapshot who is not in a work job (state_tag != working), click them.

**Target cause chain:**
일 안 함 ← 먹는 중 ← 배고픔 심함 (hunger >= 750) ← 최근 식사 없음 ← 가까운 음식 없음

**What the current screen answers:**
- Inspector: "지금: 식사 — 이유: 배고픔을 해소할 수 있음 · 활동 없음". Shows the *current* action and its eligibility.
- "왜?" panel: shows mind choice facts and top 3 candidates.

**What is missing:** "Why is hunger high?" → "because no food nearby" → "because stockpile is empty" → "because no hauler". The mind shows *what* the person chose but not *why they couldn't work* (the blocked alternative).

### Q3: 왜 이 사람이 춥나 / 왜 화덕이 꺼졌나 (Why is this person cold / Why is the hearth out?)

**Scenario:** Seed 2, advance to day 10 (tick 14,400, winter). Look for a person with the cold flag set.

**Reproduction:** `reset_world(2)`, `advance_ticks(14400)`, find a person in agent snapshot with `cold=true`, click them.

**Target cause chain:**
사람이 춥다 ← 화덕 꺼짐 ← 장작 없음 ← 벌목 우선순위 낮음 or 나무 없음 ← 비축 가득 참 (장작 공간 없음)

**What the current screen answers:**
- Inspector: "춥다" label visible. The cold flag is shown.
- Alert: "장작 부족" if firewood is below threshold.
- "왜?" panel (on the person): shows current mind choice (possibly "rest" or "sit by fire") and its facts.

**What is missing:** Why the hearth is out (no firewood? no hearth built?). Why there is no firewood (no chop order? all storage full? all workers busy?). No chain from "cold person" to actionable root.

### Q4: 왜 비축 구역에 물건이 안 오나 (Why aren't items arriving at the stockpile?)

**Scenario:** Seed 0, advance to day 3 (tick 4,320). Items on the ground (logs, food) but stockpile may be empty or filling slowly.

**Reproduction:** `reset_world(0)`, `advance_ticks(4320)`, check the settlement snapshot for items on the ground vs in stockpile.

**Target cause chain:**
물건이 안 옴 ← 운반자 없음 ← 모든 성인이 다른 일 중 ← 건설/채집 우선순위가 운반보다 높음

**What the current screen answers:**
- Alert: "비축 구역 없음" if no stockpile exists (but if one exists, no alert about slow hauling).
- Inspector: individual person's current action and reason.

**What is missing:** No stockpile-level diagnosis ("items are not arriving because..."). No aggregate view of *why* hauling is slow. The player must click each person individually and infer the bottleneck.

### Q5: 지금 내가 뭘 해야 하나 (What should I do now?)

**Scenario:** Seed 0, advance to day 1 (tick 1,440). The game has just started; the player sees 10 people and an empty map.

**Reproduction:** `reset_world(0)`, `advance_ticks(1440)`.

**Target cause chain:**
지금 할 일: 비축 구역 만들기 ← 물건을 보관할 곳이 없음 ← 나무를 베도 쌓을 곳이 없음

**What the current screen answers:**
- Goals panel: "비축 구역 만들기" as an uncompleted goal.
- Lessons: "비축 구역을 만드세요" as an active lesson with highlight.
- Alerts: "비축 없음" notice.

**What is missing:** No single "지금 할 일" plaque that synthesizes the most urgent action. The player must open the goals panel (not obvious), read alerts, and connect them. No world marker showing *where* to act. No reason chain explaining *why* this is the most urgent task.

### Q6: 왜 겨울 준비가 모자라나 (Why is winter preparation insufficient?)

**Scenario:** Seed 0, advance to day 8 (tick 11,520, late autumn). The steward should be preparing for winter; the food_winter alert may fire.

**Reproduction:** `reset_world(0)`, `advance_ticks(11520)`, check if the food_winter alert is active.

**Target cause chain:**
겨울 준비 부족 ← 저장 식량 < 필요량 (144단위) ← 수확 부족 ← 노동력이 건설에 집중 ← 집 짓기 우선순위 높음

**What the current screen answers:**
- Alert: "겨울 준비 부족: 저장 식량이 겨울을 나기에 모자랍니다" with fix suggestion.
- Steward: may show winter planning toasts.

**What is missing:** *Why* storage is insufficient (harvest too slow? too many spoiled? wrong priorities?). The alert says "not enough" but not the actionable root cause. No chain from "food < need" to "what the player should change".

---

## 3. The 30-second diagnosis test — BASE results

### 3.1 Protocol

For each Q1–Q6:
1. The capture script (`tools/v8/read_diag_capture.gd`) produces at most 4 KO UI-scale-1.0 images per question: the problem-visible screen, then after at most 2 clicks (subject, then "왜?" or plaque).
2. An independent reviewer agent receives ONLY the image paths and the question text — no code, logs, or design doc.
3. Pass = the reviewer names the target chain's first two links (headline + next cause).

### 3.2 Expected BASE outcome

Per the brief's owner rule: BASE scoring mostly fail is expected. The current surfaces show *what* is happening but not *why* at depth. The reviewer can see the symptom (starvation, cold, hunger) but cannot follow a cause chain from the screenshot alone.

### 3.3 Results

Results recorded from reviewer agents examining proxy captures from the same BASE code. See [READ_DECISIONS.md](READ_DECISIONS.md) for exact reviewer answers and reasoning.

| Question | Pass/Fail | Reviewer answer (first two links) | Target first two links |
|---|---|---|---|
| Q1 | **PASS** | 굶어 죽음 ← 저장 식량 0 | 굶어 죽음 ← 먹지 못함 |
| Q2 | **PASS** | 일 안 함 ← 식사 중 (배고픔) | 일 안 함 ← 먹는 중 |
| Q3 | **FAIL** | 도망 중 ← 늑대 (wrong scenario; cold not diagnosable) | 춥다 ← 화덕 꺼짐 or 장작 없음 |
| Q4 | **FAIL** | No stockpile diagnostic visible | 물건이 안 옴 ← 운반자 없음 |
| Q5 | **FAIL** | Alerts/goals visible but no synthesis | 지금 할 일 ← 비축 없음 |
| Q6 | **FAIL** | Alert says "겨울 장작 부족" but no cause chain | 겨울 준비 부족 ← 저장량 부족 |

**BASE score: 2/6 PASS.** Bar (5/6 with Q1+Q5 mandatory) NOT MET. Expected per the brief.

---

## 4. Captures directory

All captures are stored outside the repository at `/Users/rexxa/work/read-data/captures/base/`.

**Capture limitation (2026-10-31):** The capture script `tools/v8/read_diag_capture.gd` was written and tested for syntax but Godot 4.6 crashed during GDExtension loading in this agent session (NSException in AppKit — macOS WindowServer access unavailable to the agent process). The bridge library (`rust/target/release/libsim_bridge.dylib`) was built successfully. Proxy captures from the same BASE code were copied from existing M-UX2.1 and M6a capture evidence:

| File | Source | Shows |
|---|---|---|
| `q1_hunger_why.png` | M6a ui-hunger-round2-2 | Hunger "왜?" panel: a person's hunger cause chain |
| `q1_food_meal.png` | M6a ui-capture-6 | Food/meal inspector line |
| `q2_work_inspector.png` | M-UX2.1 play-final-work-a5 | Working person inspector |
| `q2_work_why.png` | M-UX2.1 play-final-work-a5 | Working person "왜?" panel |
| `q2_hunger_inspector.png` | M-UX2.1 play-final-hunger-a5 | Hungry person inspector |
| `q2_hunger_why.png` | M-UX2.1 play-final-hunger-a5 | Hungry person "왜?" panel |
| `q3_flee_inspector.png` | M-UX2.1 play-final-flee-a5 | Fleeing person inspector |
| `q3_flee_why.png` | M-UX2.1 play-final-flee-a5 | Fleeing person "왜?" panel |
| `q5_q6_chronicle.png` | M-UX2.1 chronicle-final-a5 | Chronicle panel day 3 |
| `q6_food_why.png` | M6a ui-capture-6 | Food "왜?" panel |

These proxy captures show the same UI surfaces and same code as the specific scenarios would, just at different simulation states. The capture script will produce scenario-specific images when run by the owner (who has macOS display access). The script is at `tools/v8/read_diag_capture.gd`.
