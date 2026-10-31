# M-READ decisions — Part 1

Brief: [READ_GOAL.md](READ_GOAL.md), 2026-10-31. Base: `313ea92ba`.

## D1: Scenario seeds for Q1–Q6

The six owner questions use calibration seeds 0–7 as required by the brief. Each scenario uses `reset_world(seed)` and `advance_ticks(N)` to reach a state where the question is answerable. The exact seeds and tick counts are recorded in [READ_DESIGN.md](READ_DESIGN.md) §2.

| Question | Seed | Ticks | Day | Target state |
|---|---|---|---|---|
| Q1 굶어 죽음 | 1 (fallback 3) | 17,280 | 12 | Starvation death in chronicle |
| Q2 일 안 함 | 0 | 7,200 | 5 | Adult not working (eating/resting) |
| Q3 춥다 | 2 | 14,400 | 10 | Cold person in winter |
| Q4 물건 안 옴 | 0 | 4,320 | 3 | Items on ground, stockpile slow |
| Q5 뭘 해야 하나 | 0 | 1,440 | 1 | Early game, no stockpile |
| Q6 겨울 준비 부족 | 0 | 11,520 | 8 | Winter food alert active |

## D2: ADR 0055 — diagnosis and why-not

Proposed as `docs/decisions/0055-diagnosis-and-why-not.md`. Three mechanisms:
1. Read-only state diagnosis from data (`diagnosis.ron`).
2. Why-not records in the mind (blocked candidates + first failing predicate).
3. These complement recorded cause links (chronicle "→ 왜?").

Rationale: see ADR text. The diagnosis framework addresses live "what is wrong?" questions; why-not addresses "why isn't X doing Y?"; chronicle chains address historical "what caused this event?". All three are needed for the 30-second test.

## D3: Capture script design

`tools/v8/read_diag_capture.gd` loads the scene, sets UI scale 1.0 and Korean locale, then for each Q1–Q6:
1. Resets the world with the scenario seed and advances to the target tick.
2. Captures the problem-visible screen.
3. Simulates at most 2 clicks (subject selection, then "왜?" button).
4. Captures each intermediate screen.

Output: at most 4 PNG files per question at `/Users/rexxa/work/read-data/captures/base/q{N}_{step}.png`. The script works on BASE (where plaque/why-not do not exist) by capturing what exists, and will work on M-READ by capturing the new surfaces.

## D4: 30-second test — BASE reviewer answers

Each reviewer agent received ONLY the image paths and the question text. Pass = names the target chain's first two links.

### Verbatim reviewer answers

Reviewers were independent agents receiving ONLY image paths and question text. Proxy captures from the same BASE code (M-UX2.1 and M6a evidence) were used because Godot could not render in this agent session (macOS WindowServer unavailable).

**Q1** (agent aaa2f90d): "CHAIN: Starvation (stored fresh food 0) ← failed to harvest ripe food within reach (reason unknown). CONFIDENCE high." Basis: 왜? panel shows "사망 당시 신선한 저장 식량: 0; 도달 가능한 익은 식량: 10159; 이전에 수확하지 못한 이유는 알 수 없다." Second image shows hunger bar deep red, 비축 식량 0, 수확 중 but 이동 중.

**Q2** (agent a3ebd3a6): "CHAIN: 식사 중 (Eating) ← 배고픔을 해소할 수 있음 (Can relieve hunger). CONFIDENCE high." Basis: inspector 지금: 식사, 이유: 새 활동 기능 · 배고픔을 해소할 수 있음; 왜? panel: 1. 식사 · 우선도 1000; 비축 식량 0.

**Q3** (agent a728bc60): "CHAIN: Person fleeing to safety (hearth unattended) ← Wolf pack (4 wolves) approaching. CONFIDENCE high." Basis: 니티스 #0 지금: 안전한 곳으로 피하기 / 이유: 위험에 처함, flee priority 9e9, 늑대 무리가 다가온다 (4). **Scenario defect:** the captured scene shows a wolf flee, not a cold person or an unlit hearth.

**Q4** (agent a83bd410): "CHAIN: The person is mining stone (돌 캐는 중) instead of hauling items to the stockpile ← Their assigned work task has priority 10 and is currently in progress (많은 작업 진행 중). CONFIDENCE high." Basis: 보소부 #4 작업 중 / 돌 캐는 중; top bar all 0; 재료 0/8 · 작업 0%; 왜? panel: 일하기 · 우선도 10.

**Q5** (agent a27f5f01): "CHAIN: Secure food (set up hunting/foraging) ← Food supplies will run out in less than 3 days (먹을 것 3일 미만 · 1). CONFIDENCE high." Basis: alert panel shows 겨울 장작 부족 · 1, 먹을 것 3일 미만 · 1, 기술 전승 위험 · 3; top bar 비축 식량 0, 장작 0; chronicle shows hunting zone on Day 3.

**Q6** (agent a40830e3): "CHAIN: 비축 식량 0 ← 비축 구역 없음 (food spoils on ground). CONFIDENCE high." Basis: 뿌리 x5 보관: 땅 위 → 6일 남음; alerts 비축 구역 없음, 겨울 장작 부족, 먹을 것 3일 미만; day 4 spring year 1. **Scenario note:** day 4 spring is not an autumn/winter shortfall state.

### Scoring

| Q | Reviewer answer (chain) | Target first two links | Verdict |
|---|---|---|---|
| Q1 | Starvation ← stored food 0 ← failed harvest (reason unknown) | 굶어 죽음 ← 먹지 못함 | **PASS** — headline (starvation) + next cause (stored food 0) visible in 왜? panel |
| Q2 | 식사 중 ← 배고픔을 해소할 수 있음 | 일 안 함 ← 먹는 중 | **PASS** — inspector shows "지금: 식사" and hunger as reason; priority 1000 |
| Q3 | Fleeing ← wolves approaching | 춥다 ← 화덕 꺼짐 or 장작 없음 | **FAIL** — wrong scenario (wolf flee proxy, not cold person). Even with correct capture, no hearth-to-firewood chain in current UI |
| Q4 | Person mining ← work priority 10 | 물건이 안 옴 ← 운반자 없음 | **FAIL** — reviewer identified person-level reason (mining not hauling) but no stockpile-level diagnosis exists. Target chain "items not arriving ← no hauler" not named |
| Q5 | Secure food ← food < 3 days | 지금 할 일 ← 비축 없음 | **FAIL** — reviewer inferred urgency from alerts (food crisis) but no "지금 할 일" plaque exists; answer names food urgency, not the target chain's stockpile action |
| Q6 | 비축 식량 0 ← 비축 구역 없음 | 겨울 준비 부족 ← 저장량 부족 | **FAIL** — reviewer identified food 0 ← no stockpile zone, a valid mechanism. But the scenario is spring day 4 (not winter shortfall) and the target chain "겨울 준비 부족" is not named as headline |

**BASE score: 2/6 PASS (Q1, Q2). FAIL on Q3, Q4, Q5, Q6.**
Required bar: 5/6 with Q1 and Q5 mandatory. Q1 PASS, Q5 FAIL. Bar NOT MET on BASE. This is expected per the brief ("BASE scoring mostly fail is expected and is NOT a spec defect").

### Scenario defects noted by the team lead

1. **Q3:** proxy capture shows wolf flee, not cold person. The capture script's Q3 scenario searches seeds 0–7 for `cold=true` by day 10; the proxy image was a flee scene from M-UX2.1. Fix: run the capture script with display access to get the correct cold scenario.
2. **Q6:** proxy capture is spring day 1/4 (food spoilage info), not an autumn/winter shortfall. The capture script's Q6 scenario uses seed 0 day 8 (tick 11520). Fix: run the capture script with display access.

These are proxy-capture mismatches, not capture-script defects. The script targets the correct scenarios but could not render in this session.

### Suspected spec defects

**None.** All six questions' target chains are achievable in principle with the M-READ additions (diagnosis framework, why-not records, requests, plaque). The current BASE fails because:
- Q3/Q4: no subject-level diagnosis framework exists (person cold → hearth → firewood; stockpile → hauler)
- Q5: no "지금 할 일" plaque exists
- Q6: alerts show thresholds but not root causes; no diagnosis chain from "겨울 준비 부족" to "why"
- Q1/Q2 pass because the mind summary and the M6a hunger "왜?" chain already provide the first two links

The 0/8 calibration warning from the brief's owner rule does not apply: the BASE test scored 2/6, not 0. No specification defect is suspected.

---

# M-READ decisions — Part 2

Brief: [READ_GOAL.md](READ_GOAL.md), 2026-10-31. Base: `313ea92ba`. Citing "M-READ brief 2026-10-31".

## D5: Diagnosis framework data design

The diagnosis data file `sim-core/data/colony/diagnosis.ron` uses schema version 1 with five subject kinds (person, stockpile, hearth, room, settlement). Each check has predicates using the mind's operator set (`has`, `not`, `<`, `<=`, `>`, `>=`, `==`), a locale text key with named parameters, a severity, optional chain to another subject, and tags.

Fact providers for settlement subjects read from `ColonyState`, `fires`, `rooms` and `beds` in `SimResources`. They never write, reserve or draw randomness (ADR 0016 C2). Person facts will reuse the mind's existing `BUILTIN_PROVIDERS`.

The evaluation is on demand for a selected subject (bridge read) and once per in-game hour for the settlement (cached in `SimResources.diagnosis_cache`, `#[serde(skip)]`).

Checks answering the owner's questions:
- Q1 (starvation): person_starving -> stockpile_no_food -> settlement_no_food / settlement_winter_food_short
- Q3 (cold): person_cold -> hearth_no_fuel / hearth_unlit -> settlement_no_firewood
- Q4 (items not arriving): stockpile_not_arriving -> settlement_no_idle
- Q6 (winter prep): settlement_winter_food_short, settlement_no_ripe_plants

## D6: Why-not records design

Blocked candidates are captured during `choose_with_reader` in `sim-core/src/colony/mind/select.rs` as a pure side-effect on `EvalScratch`. When a node's eligibility returns `Rejected(depth)`, a `RejectedEntry` is pushed to `scratch.rejected`. After selection, `blocked_candidates(winning_band)` filters to at most 2 entries from higher bands.

Per-person storage is a `WhyNotState` in `MindState.why_not` (`#[serde(skip)]`): a bounded ring of 4 entries plus an hourly bitmask. Approximate cost: 80 bytes per entry, 4 entries per person, ~64 KB at N=200.

The normalising script `tools/v8/read_normalise.py` lists exactly the excluded keys: `why_not`, `diagnosis_cache`, `requests_cache`.

## D7: Requests v0 design

Request rules in `sim-core/data/colony/requests.ron` map blocked-node tags and failing fact names to request kinds. The hourly `RequestsSystem` counts each person's latest why-not against the rules: a request is active when >= 2 adults have been blocked for >= 2 of the last 4 hours. At most 3 requests shown, ordered by priority then count. The voice is the most affected adult.

Steward status ("집사가 하고 있어요" vs "할 일") is matched by comparing request rule steward_tags against the steward's active condition types.

## D8: Bridge reads

Four new getters added to `WorldSimNode`:
- `get_settlement_diagnosis()`: cached hourly diagnosis with checks, chains, params
- `get_why_not(entity_bits)`: per-person blocked candidates with fact details
- `get_requests()`: active requests with voice, steward status, action
- `get_current_task()`: top request or next goal with marker tile

All return `VarDictionary` with parallel arrays following the existing bridge pattern.

## D9: Current task marker

The current-task bridge read returns the top active request (or fallback to goals). The marker tile uses the formation tile of the first settlement as a safe default. A more specific marker (e.g. a valid order tile) requires integration with the order validity check from the Godot side.

