# GOAL: WorldSim — M6b (Codex, staged, long-running): the Neolithic turn — first fix the gate so passes stop needing their own gate modes, then farming (fields, sowing, growth, harvest, seed, soil) with the steward's food over-harvest fixed, then weaving (flax → fibre → thread → cloth → clothes), then room temperature and house stages — each stage designed with arithmetic, built, evaluated, gated and published on its own before the next starts.

Repository `hyunlord/new-world`. Executed by **Codex**, started by the owner in `/Users/rexxa/work`, unattended, for as long as the stages take: target 36 hours, hard stop 48. **Work actively and spend the tokens:** read widely before deciding, write thorough tests, run calibrations generously, review your own diffs a second time before every gate, and never sit idle — when one stage waits (a gate, a load window, a measurement), advance design, tests or docs of the next. If your environment offers parallel agents or sessions, use them for independent work items (each in its own worktree, one measuring job at a time); otherwise sequence. Do not stop to ask the owner anything; decide, record, continue.

## Why
- Roadmap v2 (`docs/v8/DEPTH_ROADMAP.md`, owner-approved) puts **M6b 농경·직조·온도** after M-LOOP: "첫 밭, 첫 천 옷, 따뜻한 집". Era progress is capped at 643 ‰ until the farming (weight 3) and weaving (weight 2) markers exist (`sim-systems/src/runtime/knowledge/aggregate.rs`). M-LOOP's chapter 4 "첫 밭과 천 옷" is locked waiting for this pass.
- The owner's 2026-11-01 playtest showed a game where people do too little that is visible or meaningful. Fields that change through the seasons, flax turning into clothes, and warm vs cold houses are the first large, visible Neolithic change.
- Lessons that bind this pass:
  - **Keep stages small.** M6a failed twice because one pass replaced too much.
  - **Every world-running check asserts survival.**
  - **A bar at 0/8 on the first calibration round is a suspected specification defect** (O-2): stop that stage's evaluation, write the mechanism, and fix the measurement.
  - **Bars come from arithmetic.**
  - **Survival rows are judged relative to BASE** (within one world).
  - **The steward caps production** (M-LOOP).
- Open issues to resolve or carry:
  - #674: steward food over-harvest and roofed store; first item of Stage 1.
  - #675: the correct spoilage bar.
  - #665: house stages.
  - #668: era gauges.
  - #652: painting zones and fields.
  - #664: overlays.
  - #679: generic gate (Stage 0).
  - #676: trades.
  - #677: pathfinding cost.
  - #670: plant art.

## 0. RULES (all stages)
- **Setup (you do it all):**
  1. `git -C /Users/rexxa/work/new-world-pass4 fetch origin`.
  2. **Wait for M-LOOP.** M-LOOP may still be running its gate on this Mac. Until `origin/v8/main` contains `docs/v8/loop/LOOP_REPORT.md`, do only reading and writing of design documents. That means no `cargo`, no Godot, no probes, no worktree builds. Re-check every 15 minutes, up to 12 hours. While you wait, write Stage 0's and Stage 1's design documents in a scratch folder `/Users/rexxa/work/m6b-data/design/`, reading the code from `origin/v8/loop-review` with `git show`.
  3. When M-LOOP is published, record that SHA as **BASE0**. Create your worktree: `git -C /Users/rexxa/work/new-world-pass4 worktree add -b v8/m6b /Users/rexxa/work/new-world-m6b-dev origin/v8/main`. Use `…-2` if the path exists and is not a registered worktree.
  4. Remove finished worktrees whose work is published and where no process has its cwd: `new-world-loop-dev`, `new-world-read-dev`, `new-world-m6a-r2`, `new-world-m6a-dev`, `new-world-q4-dev`, `new-world-core-dev`, `new-world-mux2-dev`. Use `git -C /Users/rexxa/work/new-world-pass4 worktree remove --force <path>` and list what you removed.
  5. Run `caffeinate -dims` in the background for the whole run.
  6. Commit this brief as `docs/v8/m6b/M6B_GOAL.md` (from `~/Downloads/codex_goal_worldsim_m6b.md`).
  - Never touch `new-world-pass4`, `new-world-play2-dev` or `/Users/rexxa/work/new-world-play/`.
  - Never kill or signal processes you did not start.
  - Never touch `.claude/settings.json` or install hooks.
- **Read first:**
  - `AGENTS.md` and its skills (`worldsim-architecture`, `worldsim-godot`, `worldsim-testing`, `worldsim-pitfalls`, `systematic-debugging`, `verification-before-completion`, `worldsim-docs-upkeep`).
  - `docs/v8/DEPTH_ROADMAP.md` §3/§5/§7.
  - `docs/v8/m6a/M6A_REPORT.md`, `M6A_DESIGN.md` and `M6A_DECISIONS.md`: plants, food items, the R1/R2 lessons, the O-1 withdrawn bar.
  - `docs/v8/loop/LOOP_REPORT.md` and `LOOP_DECISIONS.md`: cards, chapters, steward scope B′ and caps, the gate mode, seeds spent.
  - `docs/v8/read/DIAGNOSIS_AUTHORING.md` and `docs/v8/core/MIND_AUTHORING.md`.
  - `docs/perf/PERF_LEDGER.md`.
  - The data in `rust/crates/sim-core/data/colony/` (`plants.ron`, `items.ron`, `recipes.ron`, `knowledge.ron`, `steward.ron`, `mind.ron`, `diagnosis.ron`, `requests.ron`), plus `assets/ui/{cards,chapters,chronicle_feed}.json` and `assets/v8art/materials/*.json`.
- **Git:**
  - One branch `v8/m6b`. Commit names are `v8-m6b(sN-pM): …`, with explicit paths only.
  - After every commit, push to `refs/heads/v8/m6b-review`.
  - **Each stage publishes on its own** after a full gate with status 0: `git push origin v8/m6b:v8/wip`, then `:v8/main`, both fast-forwards.
  - If `v8/main` moved, rebase. Docs conflicts keep both sides; code or data conflicts stop that stage. Re-run the gate.
  - Never force-push. Never push `lead/main`, `main`, `codex/*` or `v8/art-*`.
- **A red gate means diagnose, fix, re-run from step 1.** Stop a stage only for:
  - an unfixable identity or determinism failure,
  - a code conflict,
  - a first-round 0/8 bar (report it, fix the measurement, continue),
  - the hard stop.
  - When a stage is truly blocked, record it and move to the next stage's work that does not depend on it.
- **Baseline identity is byte-identical in every commit.** Colony behaviour changes are separate commits: predicted, measured and recorded.
- **Measurement:**
  - Timing repeats start at a 1-min load < 8; record 1/5/15.
  - Interleave old and new binaries in one session.
  - Run one measuring job at a time.
  - Append bench rows to `PERF_LEDGER.md` after each green gate.
- **Held-out seeds (register):** 1392–1519 are spent by M-LOOP. Use **1520–1551 for Stage 1, 1552–1583 for Stage 2, 1584–1615 for Stage 3** and 1616+ for re-runs. Each set is evaluated once after a freeze. Calibrate on seeds 0–7 with no round limit, logging every round.
- **Content is data, behaviour is code** (ADR 0033). Every new family gets a registry, a resolver, a category fallback and a data-only test. Every text string is a locale key in ko and en.
- **Issue IDs:** use temporary IDs (M6B-1, …) until each stage publishes, then the next free ISS numbers with GitHub issues. Close fixed issues with public commit links.
- **Records:** decisions go in `docs/v8/m6b/M6B_DECISIONS.md`. One `docs/v8/m6b/M6B_REPORT.md` (Korean) gets a section per stage.

## STAGE 0 — Generic gate (#679), behaviour-preserving
Three passes in a row (M-READ, M-LOOP, M6a) needed their own gate modes. Steps 44/45 were pinned to M-CORE's source, the bench base went stale, and fixture comparisons broke on every change. Make the gate compare the candidate against **the published `v8/main` it was built from (its merge base)**, with no per-pass mode:
- **Bench:** build the base binaries at the merge base, run interleaved, ≤ ×1.10 per scene, plus the C3 ceilings. Keep the ledger append. A `--justified` remainder still needs a CHANGELOG entry and ≤ ×1.15.
- **Fixture comparisons** (steps 44, 45, 50, and the colony fixtures):
  - Compare the merge base with the candidate after one committed normaliser with an explicit key list.
  - A pass declares its intentional fixture changes in `docs/<pass>/CHANGED_FIXTURES.json`, with a reason and the commit. The gate then requires survival and 0 KnowledgeLost on those fixtures instead of byte equality.
- **Stall waiver:** M-UX2.1's rule (≤ 2 frames > 50 ms, none > 60 ms, capture complete, all other steps pass) becomes a built-in, recorded result.
- **Survival everywhere:** every fixture that runs a world asserts survival relative to the merge base (founders − 2 or the base value) and 0 KnowledgeLost.
- **Tests and proof:**
  - Unit tests for each mechanism.
  - Run the new gate on BASE0 against itself: it must pass with zero declared changes.
  - Run it on a deliberately broken candidate (one gameplay constant changed): it must fail with a readable reason.
- Publish Stage 0 alone. Update `AGENTS.md` (the gate section) and `worldsim-testing`.

## STAGE 1 — Farming, and the steward stops over-harvesting
**Part 1 — Design with arithmetic** → `docs/v8/m6b/S1_DESIGN.md`, before any code:
- Food per adult-day (M6a: 2.4 units).
- Wild yield per year on seeds 0–7: measure it from BASE0 runs, do not estimate.
- Field yield per tile per season, labour hours per tile (till, sow, tend, harvest).
- The break-even field size for 10 / 20 / 40 people, and seed retention.
- From these, derive every bar of Part 3 and freeze them.

**Part 2 — Build:**
- **#674 first:**
  - The steward's food plan stops proactive harvest once usable stores plus reserved production reach the target.
  - It builds a roofed store before harvesting into the open (a blueprint via the existing house/store rule path).
  - The M-LOOP cap pattern applies to food.
  - Diagnose with numbers before changing (how much is harvested above target, which rule does it).
  - Register the **correct spoilage bar (#675)**: "days on which a member is awake, above the seek threshold, with no reachable edible food, while food of the settlement expires". First check it on a calibration run (BASE0 and candidate), and adopt it only if it discriminates.
- **Cultivation:**
  - Knowledge `cultivation`: neolithic; prereqs `foraging` and `grain_parching`. Discovery comes through the existing channels: eureka, migrants, observation.
  - New data `crops.ron`, or `plants.ron` extended with `cultivated: true`. Species: `emmer` (from wild_grain) and `flax` (from wild_flax).
  - Phases per season with planting windows, growth days, yield, seed per harvest, and soil draw.
- **Field zone:**
  - A paintable zone, through the M-LOOP zone tool and #652's paint input if not already there.
  - The tile gains a field state: untilled, tilled, sown(day), growing, ripe, harvested, fallow.
  - Soil fertility is an integer per tile. It falls per harvest and recovers when fallow.
- **Jobs:** till, sow (consumes seed items: `emmer_seed`, `flax_seed`, or the grain itself with a seed share), harvest. All go through the jobs layer and the mind's existing `do_job`, with new nodes only if needed (MIND_AUTHORING). Seed saving is a steward reserve (data).
- **Steward:**
  - A farming rule: plan the field size from the arithmetic, sow in window, harvest when ripe.
  - Cap: field area ≤ the planned need × 1.25.
  - Two new cards in `cards.json`: `sowing_window` (sow now / later / let the steward) and `harvest_ready`.
- **Diagnosis:** add field checks to `diagnosis.ron` (not sown in window, no seed, soil exhausted, no farmer). Requests: `need_seed`.
- **Era marker `farming` (weight 3)** counts practitioners of cultivation.
- **Chapters and chronicle:**
  - Chapter 4 "첫 밭과 천 옷" unlocks its farming conditions: first field sown, first harvest.
  - Chronicle notable kinds: first field, first harvest, crop failure (with cause).
- **Visuals:**
  - Code-drawn field tiles per state: furrows, sprouts, ripe tint, stubble; season-tinted.
  - Crop sprites are code-drawn placeholders. Open an art issue, like #670.

**Part 3 — Evaluate:**
- Calibrate on seeds 0–7 for 96 days (4 years), steward on, no orders. Freeze.
- **Held-out 1520–1551, 96 days.** Bars come from the arithmetic. Defaults, which the design replaces:
  - survival BASE-relative within one world;
  - no winter hunger deaths in years 2–4 ≥ 28/32;
  - a field sown by year 2 ≥ 28/32;
  - year-3 food from fields ≥ 30 % of eaten units in ≥ 24/32;
  - stores above target never exceed target × 1.5 for > 6 consecutive days in ≥ 28/32 (#674);
  - the #675 bar (if adopted);
  - ledger exact 32/32;
  - era progress ceiling now reachable above 643 ‰ in data (a test).
- Gate, publish Stage 1, then a Korean section in the report (template below).

## STAGE 2 — Weaving and clothes
- **Arithmetic first:** flax per tunic, labour per step, clothes needed per adult per year, and warmth value.
- **Chain** (recipes, knowledge and blueprints all as data):
  - `retting`: flax_stalks lie in water or on wet ground for N days and become `flax_fibre`. This is a placed "retting pit" or a water-adjacent zone.
  - `spinning` (knowledge `spinning`): fibre becomes `thread`, using a hand spindle item.
  - `loom` blueprint (knowledge `weaving`): thread becomes `linen_cloth`.
  - `tailoring`: cloth becomes `linen_tunic` and `linen_wrap`. These are apparel with `cold_permille`, a comfort value and wear, through the existing apparel system (ADR 0029).
- **Era marker `weaving` (weight 2).**
- **Cards:** `cloth_short` (the steward's dress order vs the player's).
- **Diagnosis:** "옷이 해짐 ← 천 없음 ← 실 없음 ← 아마 없음".
- **Chronicle:** first cloth, first tunic.
- **Chapter 4:** its clothing condition completes.
- **Evaluate:**
  - Held-out 1552–1583.
  - Survival BASE-relative.
  - Share of adults wearing linen by year 3 (bar from arithmetic).
  - Cold-hours per adult in winter not worse than BASE.
  - Ledger exact.
- Gate, publish Stage 2.
- **Art:** clothes for the route I people need new tops. Do not generate art. Open an art issue with exact needs (colours, layers, which looks), and render a tint/marker placeholder.

## STAGE 3 — Room temperature and house stages
- **Temperature model** (integers, deci-degrees). This replaces the binary `Cold` marker for the colony; the baseline keeps its path.
  - Outdoor temperature = season base + day/night swing + pressure (cold snap) offset, all in data.
  - Each roofed room's temperature moves toward outdoor + hearth heat (fuel-limited) + body heat, at a rate set by the room's insulation.
  - Insulation comes from the walls' sim material: add `insulation` to the sim-side material data. Wall materials are persisted identities (ADR 0044). Doors are open or closed.
  - A person's felt temperature = place temperature + apparel insulation (Stage 2 clothes count).
  - `Cold` becomes derived (felt < threshold). Mood, exposure and the mind read the derived value.
  - Update per in-game hour per room, not per tick.
- **Arithmetic first:** what hearth keeps a 3×3 log room above threshold in winter; what open-air sleeping costs.
- **House stages (#665):**
  - A dwelling's stage comes from beds, floor, hearth, warmth and stores nearby. It upgrades immediately and downgrades after N days, in data.
  - A thought or mood modifier per stage. Modifiers are still code — record it as M7 debt.
  - A render marker per stage (presentation; feeds #651 later).
- **Era gauges (#668):** the tech map shows the era markers (farming, weaving, …) as gauges with their condition.
- **Overlays (#664):** a temperature overlay computed with the same per-room geometry as the simulation.
- **Evaluate:**
  - Held-out 1584–1615.
  - Survival BASE-relative.
  - Cold deaths ≤ BASE.
  - In houses with a lit hearth, felt temperature ≥ threshold ≥ 95 % of winter night-hours.
  - Ledger exact.
  - Bench ≤ ×1.10.
- Gate, publish Stage 3.

## Every stage also delivers
- **Behaviour preservation:** baseline identity, and declared fixture changes with reasons (Stage 0 mechanism).
- **Captures instead of an owner playtest:** for each stage, a sequence of KO captures at UI scale 1.0 (start, the new feature appearing, the card or chronicle line, the inspector or diagnosis), plus one independent reviewer (a fresh session, images only) answering "이 단계에서 무엇이 새로 생겼고 사람들이 무엇을 하고 있나". Pass when the answer names the new feature.
- **Docs:** an ADR per stage; `docs/ARCHITECTURE.md`; README "처음 할 일"; `docs/ROADMAP.md`; DEPTH_ROADMAP §5/§8; the seed register in DEPTH_ROADMAP §7; lessons in skills; PERF_LEDGER rows.

## Report
`docs/v8/m6b/M6B_REPORT.md` (Korean) gets one section per stage. After each publish, send a short Korean message, then the final one with this template:
```
## 구현 완료 보고 — M6b (단계 0–3)
### 단계 0 게이트 일반화 (무엇이 바뀌었나, BASE 자기 비교·고의 결함 시험)
### 단계 1 농경 (설계 산수, 집사 과잉 수확 원인과 수정, 밭·씨앗·토양, held-out 1520–1551 표 — BASE 열)
### 단계 2 직조 (사슬, held-out 1552–1583 표)
### 단계 3 온도와 집 단계 (모델, 산수, held-out 1584–1615 표)
### 시대 진척 (643‰ 상한 해제, 표지 게이지)
### 플레이 대신 확인한 화면 (단계별 캡처와 검토자 답)
### 성능 (단계별 장면 비율)
### 닫은 GitHub 이슈 / 새로 연 이슈
### 변경된 파일 목록 (단계·커밋별)
### 확인된 제한사항
### 오너 결정 필요
### Gate 결과 (단계별)
### 오너가 할 일 (실행 한 줄: /Users/rexxa/work/new-world-play2-dev/tools/play.sh main)
git ls-remote 결과
```

## 인게임 확인사항 (오너용)
1. 2년차 봄에 "씨 뿌릴 때예요" 카드가 뜨고, 밭 구역을 칠하면 사람들이 갈고 씨를 뿌린다. 밭이 계절 따라 싹 → 익음 → 그루터기로 바뀐다.
2. 집사는 저장이 목표에 닿으면 더 거두지 않고, 지붕 있는 창고를 먼저 짓는다. 음식이 쌓여 썩지 않는다.
3. 아마를 물가에 담갔다가 실을 잣고 베틀에서 천을 짜서 옷을 입는다. 정보창에 "아마 옷"이 보인다.
4. 겨울밤에 화덕이 켜진 집은 따뜻하고, 꺼진 집이나 바깥은 춥다. 온도 오버레이로 보인다.
5. 집이 잠자리·바닥·화덕·따뜻함을 갖출수록 단계가 오르고, 기술 지도에서 농경·직조 게이지가 차면서 시대 진척이 64 %를 넘는다.
