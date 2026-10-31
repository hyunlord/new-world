# GOAL: WorldSim — M-READ (Claude Code or Codex): make causes readable before adding more depth — a data-driven state diagnosis for people, stockpiles, hearths, rooms and the village; "why not" records from the colony mind; a first version of villagers' requests built from those; one current task with a world marker; a grouped chronicle; and a 30-second diagnosis test that an independent reviewer must pass from screenshots alone.

Repository `hyunlord/new-world`, local clone `/Users/rexxa/work/new-world-pass4`. Base: **`origin/v8/main` = `313ea92bad2455b6abd0924e3c566be8b07bad65`** (M6a published 2026-10-31). Executed by **Claude Code** (in the owner-created worktree, see the launch file) or Codex, unattended. Target 16 hours, hard stop 24. Long builds, gates and probes run in the background; wait on the notification, never poll with sleeps.

## Why
Owner decision 2026-10-29 (roadmap v2 §5, row 2.5): insert a legibility pass between M6a and M6b. Evidence:
- The owner's first playtest verdict was "뭘 해야하는 건지 모르겠네" (M-UX).
- The owner's own Feudal Lord Simulator concluded the root problem was **causal illegibility, not interaction modality**. Its Stage 1 (`hyunlord/feudal-lord-simulator`, `docs/plans/2026-08-07-stage-1-make-cause-visible-design.md`) answered with read-only, ordered cause checks per object ("곡창에 빵이 없습니다 — 방앗간 확인"), a grouped population ledger, one onboarding imperative with a marker on an actually buildable tile, and a 30-second diagnosis acceptance test.
- WorldSim's story-yield probe found **0 chronicle entries with a verified cause chain of depth ≥ 3** (#647): recorded parent links alone do not explain things.
- The M-CORE mind evaluates every node's predicates each choice but records only the winner, so "why is she not eating?" has no answer today.
- Roadmap principle 2: invisible depth does not exist. M7–M9 must land in a game where causes are already readable.
- Related issues: #647, #650 (requests), the GH-IDEAS2 idea issues in milestone "M-READ 읽기" (find them with `gh issue list --milestone "M-READ 읽기"`).

Out of scope: new simulation behaviour. The steward, mind policy (`mind.ron` nodes and scores) and all gameplay stay unchanged. M-READ only reads, records and presents.

# 0. RULES
- **Start check:** `git -C /Users/rexxa/work/new-world-pass4 fetch origin`; `origin/v8/main` must be `313ea92b…` and contain `docs/v8/m6a/M6A_REPORT.md` with the status line "게시". If `v8/main` moved past it, use the new tip as BASE only if it is a fast-forward and record it; otherwise stop and report.
- **Isolation:** your worktree is `/Users/rexxa/work/new-world-read-dev` on branch `v8/read` from BASE (Claude Code: the owner created it before starting you — confirm with `git -C /Users/rexxa/work/new-world-read-dev log -1`; Codex: create it with `git -C /Users/rexxa/work/new-world-pass4 worktree add -b v8/read /Users/rexxa/work/new-world-read-dev <BASE>`). Never edit/build/run in `new-world-pass4`, `new-world-play2-dev`, `new-world-m6a-r2`, `new-world-m6a-dev` or another session's worktree; never kill or signal processes you did not start; never touch the local `.claude/settings.json` or install hooks. Commit this brief first as `docs/v8/read/READ_GOAL.md` (from `~/Downloads/goal_worldsim_read.md`). Push after every commit: `git push origin v8/read:refs/heads/v8/read-review` (the planner reads it there).
- **Git:** commits `v8-read(pN): …`, explicit paths. A red gate means diagnose, fix, re-run the full gate; stop only for a code/data conflict after a rebase, an unfixable identity/determinism failure, or the hard stop. Publish after a full gate with status 0: `git push origin v8/read:v8/wip` then `:v8/main` (fast-forwards; if main moved, rebase — docs conflicts keep both sides, code/data conflicts stop — and re-gate). Never force; never push `lead/main`, `main`, `codex/*`, `v8/art-*`.
- AGENTS.md, ADR 0016 (C2 view independence: diagnosis never changes simulation state), ADR 0033 (checks, requests and text are data), `docs/v8/core/MIND_AUTHORING.md` binding.
- **Behaviour-preserving pass:** baseline identity byte-identical; colony gameplay byte-identical — the colony fixture summaries, with only the new why-not record fields excluded by one committed normalising script that lists exactly those keys. Any gameplay difference is a defect.
- **Measurement:** timing runs start at 1-min load < 8 (record 1/5/15); old/new interleaved in one session; one measuring job on this Mac at a time.
- Issue ledger rows get temporary IDs (READ-1, …) until publish, then the next free ISS numbers; GitHub issues for them; close fixed ones (and the M-READ idea issues you complete) with public commit links after publishing. DECISIONS in `docs/v8/read/READ_DECISIONS.md`. Never ask the owner.
- **Drop order:** map highlight of grouped chronicle subjects (§2.5) → room/hearth diagnosis subjects (keep person, stockpile, settlement) → second world marker. **Never drop:** diagnosis framework + data-only test, why-not records, requests v0, the current-task plaque, chronicle grouping, the 30-second test, behaviour preservation, perf, docs, gate, report.

**Read first:** `AGENTS.md`, `CLAUDE.md` and the skills they list (`worldsim-architecture`, `worldsim-godot`, `worldsim-testing`, `worldsim-pitfalls`, `verification-before-completion`); `docs/v8/DEPTH_ROADMAP.md` §3/§7; `docs/v8/core/MIND_AUTHORING.md` and ADR 0046; `docs/v8/m6a/M6A_REPORT.md` (the food chain you will diagnose: `plants.ron`, food items, `food_economy.rs`, steward winter planning, the alerts it added); `docs/v8/mux2_1/MUX21_REPORT.md` (the chronicle kind filter and the inspector line).

**Lessons from M6a that bind here (owner, 2026-10-31):** a bar that scores 0/8 on the first calibration round is a suspected specification defect — stop and report before freezing or spending seeds (the 30-second test is run on BASE first for exactly this reason); every fixture or evaluation that runs a world checks survival, never only its own feature; keep the pass small — if a part of §2 turns out to need simulation changes, record it as M7 debt instead of growing the scope.

# 1. PART 1 — Inventory and scenarios → `docs/v8/read/READ_DESIGN.md`
1. Today's legibility surfaces, with a capture of each: inspector (incl. "지금 · 이유"), "왜?" panel, chronicle (incl. the M-UX2.1 kind filter), alerts (`assets/ui/alerts.json`), goals (`goals.json`), lessons (`lessons.json`), steward toasts/chronicle, tech map, M6a's food/winter lines. Note which bridge reads (`SimReads`) feed them.
2. **Six owner questions**, each turned into a seeded scenario (calibration seeds 0–7 or a constructed state through existing commands/fixtures; record how):
   Q1 왜 이 사람이 굶어 죽었나 · Q2 왜 이 어른이 일을 안 하나 · Q3 왜 이 사람이 춥나 / 왜 화덕이 꺼졌나 · Q4 왜 비축 구역에 물건이 안 오나 · Q5 지금 내가 뭘 해야 하나 · Q6 왜 겨울 준비가 모자라나.
   For each: what the current screen answers (capture), and the target answer chain written out (e.g. Q1: 굶어 죽음 ← 3일간 먹지 못함 ← 비축 0 ← 겨울이라 열매 없음 ← 가을 저장 4일분 / 필요 9일분).
3. ADR "diagnosis and why-not" (next free number): read-only diagnosis from state, why-not records from the mind, requests as aggregated why-nots, why this complements recorded cause links.

# 2. PART 2 — Build
## 2.1 Diagnosis framework (read-only)
- `sim-core/data/colony/diagnosis.ron` (schema 1): **subjects** `person`, `stockpile`, `hearth`, `room`, `settlement`; each with an ordered list of **checks**: `id`, `when` (predicates over named facts, the mind's operator set), `text_key` with named parameters, `severity` (`danger` / `warning` / `notice` / `ok`), optional `next` (subject kind + how to pick it, e.g. "the stockpile nearest this person's home") so checks form a chain, and `tags`.
- **Fact providers:** reuse the mind's fact providers for person facts; add read-only providers for stockpile (reachable from homes, accepts kind, free room, reserved), hearth (lit, fuel, fuel stock), room (roofed, beds, temperature or cold flag), settlement (stores vs winter need from M6a, adults, idle adults, tools). A provider may read the world and resources; it never writes, reserves or draws randomness.
- **Evaluation:** on demand for a selected subject (bridge read), and once per in-game hour for the settlement (cached in a presentation resource that is not saved and not hashed). Result: the ordered failing checks, each with parameters and the chain to depth ≤ 5; first failing check is the headline.
- **Data-only test:** add a check by data only in a test copy of `diagnosis.ron` and show it appears with its text and chain.
- `docs/v8/read/DIAGNOSIS_AUTHORING.md`: how to add a check, a fact, a subject.

## 2.2 Why-not records (mind)
- For each MindChoice, also record up to **2 blocked candidates**: nodes in a higher band than the chosen node, or in the same band with a higher score bound, whose `when` failed — `node`, `band`, and the **first failing predicate** (`fact`, `op`, `value`, actual value) with that fact's cause when it has one. Record only when such a candidate exists. Bounded ring per person as MindChoice (state the bytes per person; C3 memory ceilings must pass).
- The "왜?" view shows "하려던 것: 먹기 — 막힘: 먹을 것 없음(비축 0)" under the current action.

## 2.3 Requests v0
- `sim-core/data/colony/requests.ron`: maps (blocked node tag or failing fact) → request kind, `text_key`, suggested action (an existing order/tool), and priority. Once per in-game hour, per settlement, count each person's latest why-not against the table; a request is active when ≥ 1 adult has been blocked by it for ≥ 2 of the last 4 hours (data threshold). Voice: the most affected adult by name.
- Status per request: "집사가 하고 있어요" if a steward command addressing it exists (by tag), else "할 일". At most 3 shown, by priority then count. Requests are presentation state: not saved, not hashed, no effect on the simulation.
- Example target: "카야: 먹을 게 없어요 ← 비축 0 ← 겨울이라 열매 없음 — 할 일: 저장 늘리기".

## 2.4 One current task + world marker
- A single plaque "지금 할 일": the top active request; else the next unfinished goal (`goals.json`); else nothing. One line, the reason chain on hover, and a "보기" button that centres the camera.
- **World marker:** at most 1 marker (2 for a two-place task) on a tile where the suggested action is actually valid (same validity the order tool uses); never more than 2 markers on screen. (FLS Phase 11 lesson: 1,901 markers covered the map.)
- Lessons and alerts stay; the plaque must not duplicate an alert line word for word.

## 2.5 Chronicle grouping
- In the player-facing chronicle list, consecutive entries with the same kind and the same cause kind within one in-game day group into one line with a count ("3명이 추위로 다침 ← 한파"). Clicking a group lists its entries and highlights their subjects on the map (people ringed, tiles outlined) until closed. The data filter from M-UX2.1 stays; grouping rules live in UI data.

## 2.6 Text
All new strings as locale keys (ko and en) in the existing locale data; no hardcoded strings; Korean word-boundary wrapping (M-UX2) at UI scale 1.0 and 1.25.

# 3. PART 3 — The 30-second diagnosis test
- For each of Q1–Q6, a capture script produces: the screen at the moment the problem is visible, and the screens after at most 2 clicks (subject, then "왜?" or the plaque) — at most 4 images per question, KO, UI scale 1.0.
- **Independent reviewer:** a separate agent session that receives only the images and the question (no code, no logs, no design doc) and must state the cause chain. Pass when its answer names the target chain's first two links (headline + next cause). Record the reviewer's exact answers.
- **Bar: ≥ 5/6 questions pass, and Q1 and Q5 must pass.** Run it on BASE first (expected: most fail) and on M-READ; report both. If the reviewer's answer is ambiguous, a second independent reviewer decides; record both.

# 4. Perf, docs, gate, publish, report
- **Perf:** standing bench ≤ +10 % on every scene, C3 ceilings PASS (why-not records are the main risk; measure bytes/person). UI: selecting a subject and opening "왜?" adds no frame > 33 ms; the hourly settlement diagnosis + requests cost ≤ 1 ms per in-game hour at N = 200 (bench it).
- **Docs:** the ADR, DIAGNOSIS_AUTHORING, a MIND_AUTHORING section on why-not, `docs/ARCHITECTURE.md` (read-model section), README "처음 할 일" pointing at the plaque, `docs/ROADMAP.md` M-READ row, `docs/v8/DEPTH_ROADMAP.md` §5 status and one §6 row added verbatim from `~/Downloads/DEPTH_ROADMAP_v2.md` (the "살아 있는 마을" row, Manor Lords; also its 2026-10-29/30 history lines if missing), lessons in skills.
- Gate (all current steps + the diagnosis data-only test + a gameplay-preservation step with the normalising script + the 30-second test as a recorded check; the step-41 stall waiver of M-UX2.1/M6a applies: all other steps pass, each zoom ≤ 2 frames > 50 ms and none > 60 ms, capture complete — record use) → publish → leave the worktree for the owner to remove (your session runs inside it).
- `docs/v8/read/READ_REPORT.md` (Korean) and the final message:
```
## 구현 완료 보고 — M-READ
### 오너 질문 여섯 개 (지금 화면의 답 → 목표 사슬)
### 상태 진단 (대상·검사·사실, 데이터만으로 검사 추가 시험)
### 왜 안 하나 (기록 방식, 사람당 바이트, 예시)
### 요청 첫 판 (규칙, 예시 3개, 집사 상태 표시)
### 지금 할 일 하나와 지도 표시
### 연대기 묶기
### 30초 진단 시험 (BASE / M-READ, 질문별 판정과 검토자 답)
### 행동 보존 (기준선 동일성, 콜로니 요약 정규화 비교)
### 성능
### 닫은 GitHub 이슈 / 새로 연 이슈
### 변경된 파일 목록 (커밋별)
### 확인된 제한사항
### 오너 결정 필요
### Gate 결과
### 오너가 할 일 (실행 한 줄: /Users/rexxa/work/new-world-play2-dev/tools/play.sh main)
git ls-remote 결과
```

# 5. 인게임 확인사항 (오너용)
1. 화면 위쪽에 "지금 할 일" 한 줄이 있고, 마을 사람 이름과 이유가 붙어 있다(예: "카야: 먹을 게 없어요 ← 비축 0"). "보기"를 누르면 그 자리로 화면이 가고 지도에 표시가 하나 뜬다.
2. 사람을 누르고 "왜?"를 누르면 지금 하는 일 아래에 "하려던 것: 먹기 — 막힘: 먹을 것 없음"이 보인다.
3. 비축 구역·화덕·방을 누르면 문제가 있을 때 원인 한 줄과 그 다음 원인이 보인다(예: "물건이 안 와요 ← 길이 막힘").
4. 연대기에서 같은 일이 여러 번 일어나면 한 줄로 묶이고, 누르면 관련된 사람들이 지도에 표시된다.
5. 굶어 죽은 사람이 있으면 연대기와 정보창만 보고 30초 안에 왜 죽었는지 알 수 있다.
