# GOAL: WorldSim — M-LOOP (Claude Code): give the player a loop — decision cards raised by the simulation, a year-by-year chapter of goals, a steward limited to survival with capped production, a chronicle that shows only what matters, and the UI clutter and bugs the owner hit on 2026-11-01 fixed — so that a first-time player knows what the villagers are doing, what to watch, and what they can do.

Repository `hyunlord/new-world`. Base: **`origin/v8/main` = `fbc03bddb8f5b68f43fba1b2dad283982e5cf436`** (M-READ, published 2026-11-01). Executed by **Claude Code**, started in `/Users/rexxa/work`, unattended; it creates its own worktree (§0). Target 16 hours, hard stop 24. Long builds, gates and probes run in the background; wait on the notification, never poll with sleeps.

## Why (owner playtest 2026-11-01 12:23, screenshot in `docs/v8/loop/owner_playtest_2026-11-01.png` — the owner will attach it; if absent, use the description here)
Year 1 spring day 5, 22:01, 10 people, steward on. The owner's verdicts, verbatim in spirit:
1. "연대기는 너무 단순하고 반복적인 내용들만 많은데? 뭘 보라는 거임" — the chronicle lists routine events.
2. "지금 할 일이 어디에 뭐라고 적혀 있다는 거지" — the M-READ plaque (top-left: "시키노: 먹을 게 없어요 — 집사가 하고 있어요: 먹을거리 모으기") has no title and tells the player the steward is handling it, i.e. nothing for the player.
3. Scrolling inside the chronicle or inspector also zooms the map (wheel passes through to the camera).
4. Text too large; panels cover the view: two top rows, the plaque, a stuck item popup ("통나무 ×3 / 왜?"), alerts, the why panel, minimap, speed controls, a toast, two rows of key help — about 40 % of the screen.
5. "얘네들이 직관적으로 뭘 하는지 잘 모르겠네. 전체 목표라든가? 마을은 한 개인 건가?" — no visible purpose; the steward does the first year's basics, so the player has nothing to decide.
6. 304 logs stacked on day 5 ("저렇게 나무 쌓여 있는 게 정상인가? 할 게 저거밖에 없어서?") — steward over-production, the same pattern as M6a's over-harvest (#674). The item popup's "왜?" opens the old tile causal log showing "[4752] 기타 사건 기록" — an unlabeled raw record.
7. "지금 개발의 목표가 뭐야? 너무 원시시대이기만 한 것 같은데" — no visible horizon.

Owner-approved design (2026-11-01 12:42, planner proposal "플레이어는 조종하지 않고 긴장이 생길 때 결정한다"): the simulation raises tension → the player decides on a **card** → the consequence lands on people → the chronicle tells it. Three layers: the long goal (10 → ~100 people, Neolithic; see `docs/VISION.md` §7), **year chapters**, and **cards now**. Free-form tools shrink to five: paint zones, place blueprints, assign roles, teach, draft. The steward keeps people alive; growth is the player's. This pass builds the vessel; M7–M9 fill it.

Roadmap: `docs/v8/DEPTH_ROADMAP.md` (M-LOOP goes between M-READ and M6b; update §5 and add a §8 history line). Related issues: #650 (requests), #663 (one current task), #674 (over-harvest), #651/#665 (building look), #667 (storyteller strength setting), #647.

# 0. RULES
- **Setup (you do all of it; the owner only starts you in `/Users/rexxa/work`):**
  1. `git -C /Users/rexxa/work/new-world-pass4 fetch origin`; `origin/v8/main` must be `fbc03bdd…` (if it moved by fast-forward, use the tip as BASE and record it; otherwise stop and report).
  2. Finished worktrees from earlier passes may be removed if no process has its cwd there (`lsof -a -d cwd +D <path>` empty): `/Users/rexxa/work/new-world-read-dev`, `/Users/rexxa/work/new-world-m6a-r2`, `/Users/rexxa/work/new-world-m6a-dev`, `/Users/rexxa/work/new-world-q4-dev`, `/Users/rexxa/work/new-world-core-dev`, `/Users/rexxa/work/new-world-mux2-dev` — `git -C /Users/rexxa/work/new-world-pass4 worktree remove --force <path>` (their work is published; list what you removed). Never touch `new-world-pass4`, `new-world-play2-dev` or `/Users/rexxa/work/new-world-play/`.
  3. Create your worktree: `git -C /Users/rexxa/work/new-world-pass4 worktree add -b v8/loop /Users/rexxa/work/new-world-loop-dev origin/v8/main` (if the path exists and is not a registered worktree, use `…-2`). `cd` there for everything that follows.
  4. Copy this brief to `docs/v8/loop/LOOP_GOAL.md` and the owner's screenshot `~/Downloads/owner_playtest_2026-11-01.png` to `docs/v8/loop/owner_playtest_2026-11-01.png` (if the file is missing, note it and use the description above); commit both as `v8-loop(p0)`.
  5. Start `caffeinate -dims` in the background for the session and stop it at the end.
- Never edit/build/run in `new-world-pass4`, `new-world-play2-dev` or another session's worktree; never kill or signal processes you did not start; never touch `.claude/settings.json` or install hooks. Push after every commit to `refs/heads/v8/loop-review`.
- **Read first:** `AGENTS.md`, `CLAUDE.md` and their skills; `docs/v8/DEPTH_ROADMAP.md` §3/§7; `docs/v8/read/READ_REPORT.md` and `DIAGNOSIS_AUTHORING.md` (requests v0, plaque, why-not, chronicle grouping); `docs/v8/mux/MUX_REPORT.md` §steward and ADR 0036/0045 (steward rules in `sim-core/data/colony/steward.ron`); `assets/ui/{goals,alerts,controls,chronicle_feed,chronicle_grouping,lessons}.json`; the bridge command list in `sim-bridge/src` (`cmd_accept_migrants`, `cmd_decline_migrants`, `cmd_trade`, `cmd_order_teach`, `cmd_draft*`, `cmd_set_work_priority`, `cmd_place_blueprint`, `cmd_add_stockpile`, `cmd_designate_harvest`, `cmd_add_bill`, `cmd_set_steward`).
- **Two kinds of commits.** *Presentation* commits (UI, data for UI, bridge reads) keep baseline identity and every colony fixture byte-identical. *Behaviour* commits (only §2.4 steward scope and caps, and §2.5's "idle adults" rule if it touches jobs) are separate, predicted, measured and evaluated in §3; baseline identity stays byte-identical in all of them.
- A red gate means diagnose, fix, re-run from step 1 (the M-READ gate mode and the step-41 stall waiver apply: all other steps pass, each zoom ≤ 2 frames > 50 ms and none > 60 ms, capture complete — record use). Stop only for a code/data conflict after a rebase, an unfixable identity/determinism failure, a calibration bar at 0/8 (report before freezing), or the hard stop.
- **Measurement:** timing at 1-min load < 8 (record 1/5/15), old/new interleaved, one measuring job on this Mac; the owner will not play meanwhile. Append this pass's bench rows to `docs/perf/PERF_LEDGER.md` after the green gate.
- **Content is data, behaviour is code** (ADR 0033): card kinds, chapters, roles, chronicle visibility, text — all data with ko+en keys; no hardcoded strings; Korean word-boundary wrapping at UI scale 1.0 and 1.25.
- Temporary issue IDs (LOOP-1, …) until publish; DECISIONS in `docs/v8/loop/LOOP_DECISIONS.md`; never ask the owner.
- **Drop order:** roles (§2.6; keep the priority table) → quarrel/pressure/storage cards (§2.2 rows marked *optional*) → chapter preview art. **Never drop:** §1 fixes, §2.1 plaque, §2.2 cards for requests/migrants/traders/knowledge-risk/wolves, §2.3 chapters, §2.4 steward scope + caps, §2.5 chronicle curation, §3 evaluation, §4 first-10-minutes test, gate, report.

# 1. PART 1 — Fixes from the playtest (presentation unless noted)
1. **Wheel capture:** any scrollable panel (chronicle, inspector, tech map, priorities, lists) consumes wheel/trackpad scroll; the camera zooms only when the pointer is over the world. UI test: scroll over each panel → zoom unchanged.
2. **UI density:** (a) a text scale setting 0.85 / 1.0 / 1.15 / 1.25 in 설정, default **1.0** at 1920×1080 (today's look is effectively larger — measure and say what it was); (b) the two top status rows become one row with the rarely-changing items (era %, tech risk, births/deaths) moved into the 마을 목표/chapter panel; (c) the bottom key help collapses to one line after the first order or 3 minutes, reopens with `?`; (d) item/tile popups close when clicking elsewhere and never stack — at most one inspector, one card panel, one list panel open; (e) the alerts panel shows at most 3 lines and folds the rest.
3. **Old "왜?" cleanup:** the tile causal panel never shows unlabeled kinds ("기타 사건 기록"); entries without a locale label are filtered by data (`chronicle_feed.json` or a new `causal_feed.json`); if the tile holds a diagnosable subject (person, stockpile, hearth, room), "왜?" opens the M-READ diagnosis first with a "기록 보기" link to the labelled causal entries.
4. **Noise alerts:** "할 일 없는 어른" never fires during the night window or for people in leisure/social/eating; by day it becomes a card (§2.2) instead of an alert. Alert texts state the number and the first cause ("먹을 것 3일 미만 ← 봄이라 열매 없음") using the diagnosis chain.

# 2. PART 2 — The loop
## 2.1 "지금 할 일" plaque (presentation)
Title "지금 할 일" always visible. Content priority: the top open card's title → else the chapter's next unmet condition with its tool hint ("창고 청사진 놓기 — F3") → else "할 일 없음 — 집사가 마을을 돌보고 있어요". When the steward is already handling the matter, the plaque shows the *player's* next step, not the steward's. "보기" centres the camera; the world marker rule from M-READ stays (≤ 2 markers).

## 2.2 Decision cards (presentation + existing commands)
- `assets/ui/cards.json` (schema 1): card kinds with `id`, `trigger` (a chronicle kind, a request id, or a diagnosis check id), `title_key`, `body` (the cause chain from the diagnosis/why-not or the event's cause ref), `options` (2–3; each: `text_key`, `command` from the existing bridge commands with parameters resolved from the trigger, `recommended: bool`), `default` (what happens if unanswered: `steward` / `none`), `expires` (in-game hours), `priority`, `tags`.
- Initial cards:

| id | trigger | options |
|---|---|---|
| need_food | request `need_food` | 사냥 구역 (`cmd_add_hunt_zone` near camp) / 채집 구역 칠하기 (enter zone tool) / 집사에게 맡김 |
| need_warmth | request `need_warmth` | 벌목 지정 (`cmd_designate_chop` nearest trees) / 집사에게 맡김 |
| migrants | `MigrantsArrived` | 받기 (`cmd_accept_migrants`) / 거절 (`cmd_decline_migrants`); body lists each migrant's name, age and the knowledge they bring |
| trader | `TraderArrived` | 교환 (`cmd_trade` with the offered terms) / 거절; body: what they want, what they give, and the eureka hint if any |
| knowledge_risk | `KnowledgeAtRisk` | 가르치기 (`cmd_order_teach`: holder → the recommended learner, the adult with the highest related skill and lowest age) / 나중에 |
| wolves | `StewardDefence` or the approach signal | 집사 추천대로 소집 (`cmd_draft` the steward's 4) / 모두 대피 (undraft all) |
| idle_adults | ≥ 2 adults idle by day for ≥ 2 hours | 역할 주기 (§2.6) / 벌목 지정 / 괜찮아요 |
| quarrel *(optional)* | `Quarrel` | 떨어뜨리기 (different work priorities for the two) / 그대로 |
| pressure *(optional)* | `PressureBegan` (cold snap, drought) | 장작 우선 / 사냥 중단 / 그대로 |
| storage *(optional)* | diagnosis `stockpile_full` or spoilage above need | 창고 청사진 놓기 (enter blueprint tool at a suggested spot) / 비축 구역 넓히기 |

- Card panel on the right, above the alerts: **≤ 3 open cards**, priority order, the rest folded with a count. Each card: title, 1–3 cause lines, options as buttons, the recommended one marked. Answering sends ordinary commands (they enter the command log with source Player, so replay stays valid). Expiry applies the default and chronicles "집사가 대신 정했어요: …" (when `steward`) or nothing.
- Card state (open/answered/expired) is presentation; it is not saved and not hashed. After a load, cards are rebuilt from events of the last in-game day only. State this limit in the report.
- Data-only test: add a card kind by data only (e.g. `sick_person` on `Sick` with options 쉬게 하기 / 그대로) and show it appears and its option sends the command.

## 2.3 Year chapters (presentation, data)
- `assets/ui/chapters.json` replaces `goals.json` groups: chapters in order, each with `title_key`, `year_hint`, `conditions` (existing completion facts; add facts only through bridge reads), `reward_text_key` (what the chapter unlocks, e.g. "이주민이 찾아오기 시작합니다"), `next_preview_key`.
- Initial chapters: 1 "겨울을 난다" (stockpile, house with beds for all, firewood for winter, first winter with no deaths); 2 "모두에게 집과 창고" (roofed store, cooked meals for a week, a taught technique, a second hearth or room role); 3 "첫 토기와 첫 이주민" (pottery discovered or traded, a migrant wave accepted, a trade); 4 "첫 밭과 천 옷" (locked: "M6b에서 열립니다"); 5 "신석기 마을" (era ≥ 600 ‰, population ≥ 40 — the v1.0 horizon, shown as the long goal).
- The 마을 목표 panel shows the current chapter only (title, year, n/m conditions with ✓, the reward, one line of the next chapter). Completing a chapter: chronicle entry + toast + the next chapter opens.

## 2.4 Steward: survival only, with caps (behaviour — separate commits)
- **Scope:** default rules = stockpile (first), wood, food plan + hunt, house (shelter), defence. **Removed from the default set:** production bills (rule 4) and teach (rule 6) — they become cards (knowledge_risk, and a `tools_low` card when axes/blades are below need). Keep them available under a "집사가 더 많이" toggle in 설정 (off by default).
- **Caps (data in `steward.ron`):** every production rule has a `target` = need × days + margin and never designates/bills beyond it: logs ≤ (planned builds' logs + 20), firewood ≤ winter need × 1.25 (the M6a arithmetic), food per #674 (stop proactive harvest when usable stores + reserved production meet the target; build a roofed store before harvesting into the open). Diagnose first why 304 logs existed on day 5 (which rule, which designation count) and write it in DECISIONS.
- Predicted effect, written before measuring: fewer idle-chop hours, logs at day 24 ≤ target × 1.5 in ≥ 28/32 worlds; survival rows unchanged vs BASE within one world.

## 2.5 Chronicle curation (presentation, data)
- Default feed shows only *notable* kinds, by data: Born, Died, Paired, CameOfAge, BecameElder, Widowed, Quarrel, Fight, Breakdown, KnowledgeDiscovered, KnowledgeAtRisk, KnowledgeLost, Eureka, MigrantsArrived/Joined/Declined, TraderArrived/Traded, WolfKilled, Drafted (first per event), PressureBegan/Ended, Sick/Recovered, FirstCookedMeal, FirstDriedStore, WinterFoodPlan, WinterStoredSurvived, StewardDefence, RoomFormed (first of each role), HutBuilt (first), chapter completed, card decisions. Routine kinds (Hunted, FoodOperation, StewardAction, PorterDeparted/Delivered, HungerFoodChain unless a death follows, MindChoice) go behind the "전부" toggle.
- Each line: day, subject name, what, "← cause" when a cause ref exists; grouping from M-READ stays. A "사람별" filter (click a person → their entries).

## 2.6 Roles (presentation + `cmd_set_work_priority`; droppable)
Four roles in `assets/ui/roles.json`: 사냥·채집, 벌목·건축, 제작, 돌봄·운반 — each a preset of work priorities. The people list gets a role column; changing a role sends the priority commands. The priority table stays as "고급". Founders get roles from their highest skill at world start (presentation default, applied as commands on the first tick — record how replay handles it).

# 3. PART 3 — Evaluation
- **Behaviour (steward) held-out:** seeds 1392–1423, 48 days, steward on, no player orders. Bars: the M-UX2 rows (day-1 stockpile 32/32, beds by day 10 ≥ 30, cold deaths 0 ≥ 28, village alive day 48 ≥ 30, wolf deaths ≤ 1 ≥ 28, ledger 32/32) within one world of BASE on the same seeds; logs at day 24 ≤ target × 1.5 ≥ 28/32; firewood never zero in a cold snap among exposed worlds (exposed − 1). Calibration seeds 0–7, log every round; a 0/8 bar stops the pass for a report.
- **Cards scripted run:** seeds 0–7, 48 days, a script that answers every card with its recommended option within 1 hour: survival equal or better than the no-answer run; every answer appears in the command log and the chronicle; no card ever shows a command that fails validation (count = 0).
- Presentation preservation: colony fixtures byte-identical except the behaviour commits' recorded changes.

# 4. PART 4 — The first-10-minutes test (replaces the 30-second test for this pass)
Capture the first 10 in-game days at 1× (seed 0, steward on) as a sequence: every chapter/plaque/card change and one screen per in-game day, KO, UI scale 1.0 and 1.25. An **independent reviewer** (fresh session, images only) answers: (a) 이 사람들은 지금 무엇을 하려고 하는가, (b) 플레이어가 지금 할 수 있는 결정은 무엇인가, (c) 이 마을의 올해 목표는 무엇인가, (d) 가장 최근 중요한 사건과 그 원인은. Bar: 4/4, each answer naming the on-screen chapter/card/chronicle item. Run on BASE too (expect fails) and record both.

# 5. Perf, docs, gate, publish, report
- Bench ≤ ×1.10 per scene vs BASE interleaved; C3 PASS; card/chapter evaluation ≤ 0.5 ms per in-game hour at N = 200; UI: opening a card or chapter adds no frame > 33 ms.
- Docs: an ADR "player loop: cards, chapters, survival steward"; `docs/ARCHITECTURE.md` UI data section; `README.md` "처음 할 일" rewritten around the plaque, cards and chapters; `docs/v8/DEPTH_ROADMAP.md` §5 (M-LOOP row, status), §8 line; `docs/ROADMAP.md`; lessons in skills; `PERF_LEDGER` rows.
- Gate → publish (fast-forward of BASE; rebase rule) → issues: close #650, #663, #674 (if the cap fixed it), #667 (if the storyteller setting landed — optional) with public commit links; open rows for what you deferred; update #649.
- `docs/v8/loop/LOOP_REPORT.md` (Korean) and the final message:
```
## 구현 완료 보고 — M-LOOP
### 오너 지적 1–7과 처리 (전후 캡처)
### 지금 할 일·결정 카드·장(chapter) (데이터, 예시 캡처, 데이터만으로 카드 추가 시험)
### 집사 범위와 상한 (304 통나무의 원인, 바뀐 규칙, held-out 1392–1423 표 — BASE 열 포함)
### 연대기 큐레이션 (기본/전부, 예시)
### 역할 (있으면)
### 첫 10분 시험 (BASE / M-LOOP, 검토자 답)
### 성능
### 닫은 GitHub 이슈 / 새로 연 이슈
### 변경된 파일 목록 (커밋별)
### 확인된 제한사항
### 오너 결정 필요
### Gate 결과
### 오너가 할 일 (실행 한 줄: /Users/rexxa/work/new-world-play2-dev/tools/play.sh main)
git ls-remote 결과
```

# 6. 인게임 확인사항 (오너용)
1. 시작하면 위에 "지금 할 일"이 제목과 함께 있고, 오른쪽 "마을 목표"에 "1장: 겨울을 난다 — 조건 0/4"가 보인다.
2. 며칠 안에 오른쪽에 카드가 뜬다(예: "시키노: 먹을 게 없어요 ← 봄이라 열매 없음 — [사냥 구역] [채집 구역 칠하기] [집사에게 맡김]"). 버튼을 누르면 사람들이 그대로 움직이고 연대기에 "내가 정했어요"가 남는다.
3. 통나무가 수백 개 쌓이지 않는다. 집사는 비축·장작·먹을거리·집·방어만 하고, 도구 만들기와 가르치기는 카드로 묻는다.
4. 연대기에는 출생·죽음·다툼·발견·이주민·늑대·첫 요리 같은 일만 보이고, "전부"를 켜야 일상이 보인다.
5. 패널 안에서 스크롤해도 지도가 줌되지 않고, 화면을 가리는 UI가 전보다 훨씬 적다. 글자 크기는 설정에서 바꿀 수 있다.
6. 1년을 넘기면 "1장 완료" 기록과 함께 2장이 열린다.
