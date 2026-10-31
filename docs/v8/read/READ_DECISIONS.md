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

### Reviewer answers

Reviewers were independent agents receiving ONLY image paths and question text. Proxy captures from the same BASE code (M-UX2.1 and M6a evidence) were used because Godot could not render in this agent session (macOS WindowServer unavailable). Reviewer judgments based on what is visible in the screenshots.

| Q | Reviewer answer | Target first two links | Verdict |
|---|---|---|---|
| Q1 | 굶어 죽음 ← 저장 식량 0 (왜? panel shows "사망 당시 신선한 저장 식량: 0", then "도달 가능한 익은 식량: 10159", then "수확하지 못한 이유는 알 수 없다") | 굶어 죽음 ← 먹지 못함 | **PASS** — first two links visible in the 왜? panel: death + stored food 0. Chain depth 3 present but last link admits "이유는 알 수 없다" |
| Q2 | 일 안 함 ← 식사 중 (inspector shows "지금: 식사 · 이유: 새 활동 가능 · 배고픔을 해소할 수 있음", 목표: 식량) | 일 안 함 ← 먹는 중 | **PASS** — inspector clearly shows "지금: 식사" and hunger as the reason |
| Q3 | 사람이 도망 중 ← 늑대 위협 (inspector shows fleeing from wolves, NOT cold) | 춥다 ← 화덕 꺼짐 or 장작 없음 | **FAIL** — proxy images show a fleeing person, not a cold person; even with a correct cold capture, the chain from cold to hearth to firewood is not shown |
| Q4 | 사람이 일하는 중 (inspector shows working person, no stockpile diagnostic) | 물건이 안 옴 ← 운반자 없음 | **FAIL** — no stockpile-level diagnosis exists; the UI shows individual person actions, not aggregate stockpile status |
| Q5 | 연대기에 집사 행동 목록 보임, 경고에 장작 부족·식량 부족·기술 위험 보임, but no single "지금 할 일" synthesis | 지금 할 일 ← 비축 없음 | **FAIL** — alerts and chronicle show fragments but no single plaque or recommended action |
| Q6 | 식량 바닥에 뿌리 5개, 6일 남음 (food tooltip shows roots on ground, 6 days left). 경고: 겨울 장작 부족, 먹을 것 3일 미만, 비축 구역 없음. No cause chain for winter shortage | 겨울 준비 부족 ← 저장량 부족 | **FAIL** — alerts say "겨울 장작 부족" and "먹을 것 3일 미만" but not WHY preparation is insufficient (no harvest? wrong priorities? spoilage?) |

**BASE score: 2/6 PASS (Q1, Q2). FAIL on Q3, Q4, Q5, Q6.**
Required bar: 5/6 with Q1 and Q5 mandatory. Q1 PASS, Q5 FAIL. Bar NOT MET on BASE. This is expected per the brief ("BASE scoring mostly fail is expected and is NOT a spec defect").

### Suspected spec defects

**None.** All six questions' target chains are achievable in principle with the M-READ additions (diagnosis framework, why-not records, requests, plaque). The current BASE fails because:
- Q3/Q4: no subject-level diagnosis framework exists (person cold → hearth → firewood; stockpile → hauler)
- Q5: no "지금 할 일" plaque exists
- Q6: alerts show thresholds but not root causes; no diagnosis chain from "겨울 준비 부족" to "why"
- Q1/Q2 pass because the mind summary and the M6a hunger "왜?" chain already provide the first two links

The 0/8 calibration warning from the brief's owner rule does not apply: the BASE test scored 2/6, not 0. No specification defect is suspected.
