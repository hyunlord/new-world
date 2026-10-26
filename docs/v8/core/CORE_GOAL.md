# GOAL: WorldSim — M-CORE (Codex): give the colony its own brain — a data-driven think tree with scored choices and a recorded reason for every choice — so that every later depth pass (mind, society, body) plugs in as data + facts + behaviours instead of more branches in the V7 cascade; freeze the V7 decision path as baseline-only; measure first, shadow-compare, switch, evaluate, record the depth roadmap.

Repository `hyunlord/new-world`, local clone `/Users/rexxa/work/new-world-pass4`. Base: **`origin/v8/main` after M-UX2 is published** (Part 0 checks it). Executed by **Codex (GPT Astra)**, unattended. Target 16 hours, hard stop 24.

## Why
The game's purpose is a deep internal simulation (DF/RimWorld-level people whose every act has a cause). The owner approved, 2026-10-26, the depth roadmap in **Appendix A**: M-CORE → M6 Neolithic → M7 mind → M8 society → M9 body → M10 stories → M11 scale, and the v1.0 definition in it. M-CORE comes first because every colony person still decides through V7's `AgentDecisionSystem` (`rust/crates/sim-systems/src/runtime/decision/system.rs`, 3,487 lines; its `tick()` is one ~2,900-line function of pass-gated V7 rules, game_v17–v26) with jobs wedged in through `jobs/seam.rs`. Each new system so far added "≤ 20 lines at the decision seam". Mind, society and body would each add dozens more branches; ADR 0039 already rejected splitting the cascade because its arms share mutation order. Last measured cost: **80.0 % of the N = 1,000 tick at M2** (`docs/perf/PERF_LEDGER.md`, 1,371 µs); the current share is **unmeasured** (`docs/v8/review/CODE_REVIEW_2026-10.md` l.173, 295) — do not quote 80 % as today's figure.

Decision (owner-approved roadmap): the colony gets a new system; V7's cascade stays, unchanged, for the `baseline` world only (ADR 0014 identity). No deletion of baseline or of research tags.

# 0. RULES
- **Start check:** `git -C /Users/rexxa/work/new-world-pass4 fetch origin`; `origin/v8/main` must contain `docs/v8/mux2/MUX2_REPORT.md`; record its SHA as **BASE**. If not, stop and report "M-UX2 not published".
- **Isolation:** your own worktree `/Users/rexxa/work/new-world-core-dev`, branch `v8/core` from BASE (`git -C /Users/rexxa/work/new-world-pass4 worktree add -b v8/core /Users/rexxa/work/new-world-core-dev <BASE>`; if the path exists and is not a registered worktree, use `…-2`). Never edit, build or run in `/Users/rexxa/work/new-world-pass4`, `/Users/rexxa/work/new-world-play2-dev` or another session's worktree; never kill or signal other processes. Read this brief from `~/Downloads/codex_goal_worldsim_core.md`, commit it first as `docs/v8/core/CORE_GOAL.md`.
- **Git:** commits `v8-core(pN): …`, explicit paths. Publish only after a gate with status 0: `git push origin v8/core:v8/wip`, then `git push origin v8/core:v8/main` (fast-forwards). If refused because `v8/main` advanced (a renderer pass such as M-UX2.1 may publish meanwhile), rebase onto `origin/v8/main` only if the rebase has no conflicts, re-run the full gate (and re-check that the shadow/held-out numbers are unaffected: the other pass may not touch the simulation), then push; on a conflict or a red gate, stop and report. Never force. Never push to `lead/main`, `main`, `codex/*`, `v8/art-*`.
- AGENTS.md, ADR 0016, ADR 0033 are binding. **Safety net:** baseline identity byte-identical in every commit. Colony OFF fixtures and the steward-ON fixtures byte-identical in every commit **before the switch commit (Part 5)**; the switch commit is the only behaviour change of the colony, predicted, measured, recorded.
- **Bars state their arithmetic** in the report. Calibrate on seeds 0–7 only (≤ 4 logged rounds); freeze the policy before held-out seeds.
- Records as you go: `docs/HISTORY.md`, `CHANGELOG.md`, `docs/ISSUES.md` (rows found/fixed), GitHub issues the registry created (#513 onward) closed with a commit link when fixed; new issues for what you find; never edit issues the registry did not create. DECISIONS in `docs/v8/core/CORE_DECISIONS.md`. Never ask the owner; questions go in the report.
- **Drop order:** crate-level separation (§2.3) → inspector candidate list (§6, keep the one-line reason) → perf work beyond the bar. **Never drop:** inventory (§1), shadow comparison (§4), switch + held-out evaluation (§5), identity, perf measurement (a missed perf bar is reported, not hidden), story-yield instrument (§7), docs (§8), gate, report.

# 1. PART 1 — Measure and inventory (no behaviour change)
1. **Per-system cost:** `worldsim_perf` per-system at N = 200/1,000/2,000 on the colony stress scene, quiet machine (1-min load ≤ 4, record it), 3 interleaved runs. Record `AgentDecisionSystem`'s share **s** (mean µs / tick mean µs) at N = 1,000. Add a feature-gated (`perf_arms`, off by default) timer per cascade arm; report the cost per arm.
2. **Colony inventory** `docs/v8/core/CORE_INVENTORY.md`: every branch/arm of `AgentDecisionSystem` and `jobs/seam.rs` with line range, gate rule, and whether it can fire with `rules.colony = Some`. Prove "fires in colony" with counters (feature-gated) over the 9 colony OFF fixtures, the steward-ON fixtures and 8 seeds × 48 days steward on. Columns: arm · reads · writes · colony fire count · cost · verdict (**port** / **replace** with the new rule stated / **drop** with reason). V7-only arms (porters, fission founders, influence parent links …) are dropped from the colony only if their colony fire count is 0 or the replacement is stated.
3. The seam's yield rules (`yield_margin_permille`, `finish_leg_tiles`, critical preemption 75, craft yields only to critical) are listed with their values: they become data in §3.

# 2. PART 2 — Baseline freeze (no behaviour change)
1. Register systems by world: a table in code (`sim-engine` registration) listing which systems run in `baseline`, `colony`, or both; one test prints it into `docs/v8/core/SYSTEM_WORLDS.md`.
2. After §5's switch, move `AgentDecisionSystem` and its colony-unreachable helpers into `sim-systems/src/runtime/baseline/decision/` by pure move; delete the colony gates inside it that become dead (`rules.colony` checks — dead because it no longer runs in the colony) and `jobs/seam.rs`. Identity byte-identical. Gate step `baseline_isolation`: no colony module imports `runtime::baseline::*` (grep check, like `extensibility_grep`).
3. (Droppable) a `sim-baseline` crate for the frozen V7 systems if the dependency graph allows it without touching identity; else record why not.

# 3. PART 3 — The colony mind (ADR "colony mind")
**Model** (RimWorld's ThinkTree + utility scoring inside a band; ADR 0033: kinds are data, behaviour is code):
- **`sim-core/data/colony/mind.ron`** (schema_version 1): ordered **bands** `critical`, `duty`, `need`, `work`, `leisure`, `idle`, and **nodes**: `id`, `band`, `behaviour` (a registered code key), `when` (a list of predicates over named facts: `fact op value`, ops `<`, `<=`, `>`, `>=`, `==`, `has`, `not`), `score` (integer terms: `fact × weight‰` + constant; permille, i64), `hysteresis` (score margin to switch within a band), `life_stages` (adult/child/elder), `tags`.
- **Facts:** a `PersonFacts` table built once per mind tick (columnar, AgentId order) by **registered fact providers** (code: name → provider). Initial facts cover everything the ported arms read (hunger, thirst, fatigue, social, drafted, threat distance + reachable shelter (M-UX2 flee rule), held job and stage, carry state, distance to job, night window, temperament axes, life stage, sick hp, mood band, cold). A new pass adds a provider + data, never edits the mind core.
- **Behaviours** (code, registry name → impl): at least `eat`, `drink`, `sleep`, `flee`, `fight_drafted`, `do_job` (the jobs layer assigns; the mind decides *whether* to work now), `hearth_meal`, `socialise`, `rest`, `wander`, plus whatever §1 marks **port**. Each behaviour owns start/continue/finish and writes `AgentState`/`SeekTarget` as the current code does.
- **Choice:** per person, AgentId order: walk bands top-down; the first band with ≥ 1 eligible node wins; inside it the highest score wins (ties: node order in data). A running behaviour continues unless a higher band becomes eligible or a node in its band beats it by `hysteresis`. The seam's yield rules become `when`/`hysteresis` data (state the mapping in DECISIONS). Randomness only via keyed streams `(agent, tick, node)`; no hash-map iteration order in results.
- **Decision record:** every switch writes `CausalEvent::MindChoice { agent, node, band, score, top3: [(node, score)], facts: [(fact, value)] (the predicates and score terms that decided it) }` with the parent being the cause record when the fact has one (hunger → last meal, threat → the pack's spawn, job → the order/steward command). Bounded ring per person (last 16) for the UI; chronicle only for band changes into `critical`.
- **Scheduling:** same priority slot (125) and cadence the colony uses now; registered only in the colony world.
- **Data-only test (ADR 0033 family test):** add, by data only, a node `warm_at_hearth` (band `need`, `when cold has; hearth_lit has`, behaviour `rest` at the hearth target) in a test fixture copy of `mind.ron`; show it fires and is recorded. Also a test that an unknown behaviour/fact name fails loading with a clear message.
- `docs/v8/core/MIND_AUTHORING.md`: how to add a fact, a behaviour, a node; with the `warm_at_hearth` example.

# 4. PART 4 — Shadow comparison (no behaviour change)
Run the mind in **shadow** (computes choices, writes nothing, feature `mind_shadow`) beside the old decision on seeds 0–7 × 20 days, steward on and off. Per person-tick compare the chosen **band and behaviour** with the old choice. `docs/v8/core/CORE_SHADOW.md`: agreement table by arm; every disagreement category (≥ 0.1 % of person-ticks) with an example tick and a verdict: *intended* (cite the §1 replace rule) or *fixed* (fix and re-run). **Bar:** behaviour agreement ≥ 97 % on person-ticks where the old choice is a **port** arm (arithmetic: a faithful port disagrees only on hysteresis/tie edges; 3 % allows ~1 in 33 boundary ticks), 0 unexplained categories.

# 5. PART 5 — Switch and evaluate (the one behaviour-changing commit)
- Switch the colony to the mind; re-capture colony fixtures; record old → new digests in DECISIONS. Baseline identity byte-identical.
- **Held-out seeds 1232–1263** (new; M-UX2 used 1200–1231), 48 days, steward on, no player orders: every row of M-UX2's defence-ON evaluation table ≥ its M-UX2 measured value − 1 world (1 world = 3.1 pp of 32), **except the cold-snap firewood row, whose denominator is the worlds that saw a cold snap** (owner decision after M-UX2: a world without a cold snap is neither a pass nor a fail; report exposed/32 and the no-depletion count; bar: no depletion in ≥ (exposed − 1) of them); ledger exact 32/32; story/knowledge/neighbour fixtures pass or their changes are explained. Run the same seeds on the old brain at BASE as the comparison column.
- **Why coverage:** ≥ 99 % of behaviour switches in those runs have a `MindChoice` with ≥ 1 fact; 100 % of `critical` switches.
- **Perf bar:** same session, interleaved, quiet: mind mean ≤ 0.5 × the old decision mean at N = 1,000; total tick mean ≤ (1 − s/2 + 0.05) × old tick mean (arithmetic: halving a share s cuts the tick by s/2; 0.05 headroom for facts building). Report N = 200/2,000 too. Use columnar facts, per-band early exit, cadence for leisure/idle nodes (data `every_ticks`) if needed.

# 6. UI (small, GDScript reads only)
Person inspector: one line "지금: <행동> — 왜: <가장 큰 이유 1–2개>" from the latest `MindChoice` (localised names from data); the existing "왜?" button shows the top-3 candidates with scores. Text keys in the locale data (en + ko); no hardcoded strings.

# 7. Story-yield instrument (the depth roadmap's metric)
`worldsim_probe story-yield`: per world-year, (a) chronicle entries whose cause chain has depth ≥ 3, (b) distinct cause-kind chains of depth ≥ 3, (c) people with ≥ 3 chronicle entries. Record BASE and M-CORE numbers on seeds 1232–1263 in `docs/v8/core/STORY_YIELD.md` (no bar; it is the reference for M6–M11).

# 8. Docs
two ADRs with the **next free numbers** after the published `docs/decisions/` (M-UX2 already used 0045): "colony mind" and "baseline isolation", `docs/ARCHITECTURE.md` decision section rewritten, AGENTS.md pointer to MIND_AUTHORING. **Commit Appendix A** as `docs/v8/DEPTH_ROADMAP.md`; update `docs/ROADMAP.md` (rows M-CORE, M6–M11 from Appendix A; the old "M6 마감" and "M-PERF" rows marked superseded with a link; the "다음" line) and `docs/VISION.md` (the v1.0 definition from Appendix A, replacing the old one, old text kept under "이전 정의").

# 9. Gate, publish, report
Gate (all current steps + `baseline_isolation` + mind data-only test + shadow re-run on seed 0 as a fixture) → publish → remove your worktree. `docs/v8/core/CORE_REPORT.md` (Korean) and the final message:
```
## 구현 완료 보고 — M-CORE
### 측정 (결정 시스템 점유율 s, 갈래별 비용)
### 콜로니 갈래 목록 (옮김 / 바꿈 / 버림, 각각 이유)
### 새 두뇌 구조 (띠·노드·사실·행동, 데이터만으로 노드 추가 시험)
### 그림자 비교 (일치율, 불일치 범주와 판정)
### 전환 평가 (held-out 1232–1263: 새 두뇌 / 옛 두뇌 표)
### "왜?" 기록 범위와 사람 정보창
### 성능 (전후 표, 막대 산수)
### 기준선 분리 (등록표, 동일성)
### 이야기 수확량 (BASE / M-CORE)
### 닫은 GitHub 이슈 / 새로 연 이슈
### 변경된 파일 목록 (커밋별)
### 확인된 제한사항
### 오너 결정 필요
### Gate 결과
### 오너가 할 일 (실행 한 줄: /Users/rexxa/work/new-world-play2-dev/tools/play.sh main)
git ls-remote 결과
```

# 10. 인게임 확인사항 (오너용)
1. 사람을 누르면 정보창에 "지금: 나무 베기 — 왜: 작업 우선순위 1, 배고픔 낮음"처럼 지금 하는 일과 이유가 한 줄로 보인다.
2. "왜?"를 누르면 그 사람이 고민한 후보 3개와 점수가 보인다.
3. 마을 생활은 M-UX2와 비슷하게 돌아간다: 첫날 비축, 열흘 안에 집과 잠자리, 늑대가 오면 방어와 대피.
4. 배고픈 사람은 일을 멈추고 먹으러 가고, 늑대가 가까우면 하던 일을 버리고 피한다. 그때 정보창의 이유가 바뀐다.
5. 화면이 전보다 느려지지 않는다.

---

# Appendix A — 깊이 로드맵 (오너 승인 2026-10-26; `docs/v8/DEPTH_ROADMAP.md`로 커밋)

## v1.0 정의
10명으로 시작해 몇 년 안에 약 100명이 되고, 신석기 시대로 넘어간다. 사람마다 성격·기억·관계가 있고, 그 일생과 마을의 역사를 읽을 수 있다. 1만 명, 청동기 이후, LLM 해설은 v1.0 밖이다.

## 원칙
1. **맞물림으로 깊어진다.** 새 시스템은 기존 시스템 둘 이상에서 입력을 받고 둘 이상에 출력해야 들어온다. 시스템 수를 목표로 삼지 않는다(V7: 59개 시스템, 실제 작동 30 %).
2. **보이지 않으면 없다.** 모든 내부 상태 변화는 "왜?", 연대기, 사람 상세 중 하나에 나타난다.
3. **효과마다 원인 기록.** 모든 효과는 원인 레코드를 가리킨다.
4. **막대는 산수부터.** 기댓값 계산을 설계에 적고 나서 막대를 정한다.
5. **성능과 규모는 설계 단계에서.** 시스템마다 상세 LOD(가까운 사람은 전부 계산, 먼 사람은 요약)를 정한다.
6. **이야기 수확량을 잰다.** 세계 1년당 깊이 3 이상 원인 사슬 수, 서로 다른 사슬 종류 수, 기록이 3개 이상인 사람 수(M-CORE §7)를 매 패스 보고한다.
7. **매 패스 오너 플레이.** 설계 → 산수 → 구현 → 보류 시드 평가 → 오너 플레이.
8. **콘텐츠는 데이터, 행동은 코드**(ADR 0033). 새 기둥은 사실 제공자 + 행동 + `mind.ron` 노드로 꽂는다.

## 다섯 기둥
| 기둥 | 내용 | 맞물림 |
|---|---|---|
| A 두뇌 (M-CORE) | 데이터 판단 트리 + 띠 안 점수, 선택마다 이유 기록 | 모든 기둥의 꽂는 자리 |
| B 마음 (M7) | 사건 → 생각(원인·크기·지속·성격 보정) → 기분 = 생각의 합 → 행동; 의미 있는 생각은 기억, 사람 사이 생각은 호감 | 기분·다툼·관계·기억·연대기 통합 |
| C 개인차 (M7) | 기질 4축 → 성격 6축(HEXACO) 파생 → 선호(음식·재료·일·사람) → 생각의 크기와 일 고르기 | 마음, 일, 관계 |
| D 몸 (M9) | 부위 약 20개, 상처·출혈·감염·치유, 약초 지식, 노화에 따른 능력 저하 | 지식, 물건, 방, 마음, 일 능력 |
| E 사회 (M8) | 친구·연인·배우자·원수 형성, 결혼과 분가, 존경과 지도자, 장례·잔치, 아이의 놀이 학습 | 마음, 지식 전승, 인구 |

## 패스 순서
| 순서 | 패스 | 핵심 | 보여야 하는 것 |
|---|---|---|---|
| 1 | M-CORE | 콜로니 두뇌, 기준선 분리, 이야기 수확량 계기 | 사람마다 "지금 하는 일과 이유"가 보인다 |
| 2 | M6 신석기 | 채집 식물의 계절, 요리, 영양 다양성, 보존, 농경, 직조, 방 온도와 재료 단열 | 몇 년을 살아남는 마을, 첫 밭, 첫 천 옷 |
| 3 | M7 마음 | 생각·기억·호감 엔진, 성격 파생과 선호, 기분 수정자를 데이터로 | "왜 슬픈가"에 원인 있는 생각 목록이 보인다 |
| 4 | M8 사회 | 관계 형성, 결혼·분가, 지도자, 의례, 아이 학습, 인구 상한 40 → 100 | 가계가 이어지고 지도자가 바뀐다 |
| 5 | M9 몸 | 부위·상처·감염·약초·노화 | 늑대 상처가 흉터와 다리 절기로 남는다 |
| 6 | M10 이야기 | 이야기 감지기, 일생 페이지, 연대기 큐레이션, 연말 책 | 한 사람의 일생을 읽을 수 있다 |
| 7 | M11 규모 | 100 → 1,000명, 상세 LOD | 이웃이 실제 정착지가 된다 |
| 사이사이 | UX·아트 | 오너 플레이 뒤 UX 패스, 아트(작물·직조 옷·상의·북쪽 방향·노인) | — |

대체된 줄: 옛 "M6 마감"(아트 본 생산, 사람 상세)은 M10과 아트 패스로, 옛 "M-PERF"는 M-CORE의 성능 막대와 M11로 옮긴다.
