---
name: kanban-workflow
description: |
  Kanban board integration, Codex dispatch rules, autopilot/ultrapilot workflow,
  ticket management, and batch lifecycle. Use when dispatching tickets via
  codex_dispatch.sh, creating batches, running autopilot workflow, or managing
  PROGRESS.md and ticket templates.
---

# Kanban & Dispatch Workflow — SKILL.md

> This skill covers kanban board integration, Codex dispatch rules, autopilot workflow, and common workflow mistakes.
> For code conventions and project architecture → see CLAUDE.md
> For localization and prompt engineering → see .claude/skills/worldsim-code/SKILL.md

---

## Part 1: Kanban Board Integration

### 규칙: 모든 작업은 칸반에 등록한다

칸반 서버: `http://localhost:8800` (Docker: `docker compose -f tools/kanban/docker-compose.yml up -d`)

1. **프롬프트 시작 시** — 배치(Batch)를 생성한다:
   ```bash
   source tools/kanban/scripts/kanban_helpers.sh
   BATCH_ID=$(kanban_create_batch "프롬프트 제목" "prompt-filename.md")
   ```

2. **DISPATCH 티켓 생성 시** — 칸반에 등록한다:
   ```bash
   TICKET_ID=$(kanban_create_ticket "티켓 제목" "$BATCH_ID" "codex" 1 "시스템명" "high")
   ```

3. **DIRECT 작업 시작 시** — 티켓 생성 + 즉시 in_progress:
   ```bash
   TICKET_ID=$(kanban_direct_start "DIRECT: 통합 와이어링" "$BATCH_ID" 8 "시스템명")
   ```

4. **DIRECT 작업 완료 시** — 상태 업데이트:
   ```bash
   kanban_direct_done "$TICKET_ID"
   # 실패 시: kanban_direct_fail "$TICKET_ID" "에러 메시지"
   ```

### Dispatch 경로별 칸반 처리 ⚠️ 반드시 구분할 것

칸반 티켓 등록은 **dispatch 경로에 따라 처리 방법이 다르다.**

#### 경로 A: `codex_dispatch.sh` 사용 시 → 칸반 자동 처리됨
`codex_dispatch.sh`는 내부적으로 칸반 API를 호출한다. BATCH_ID 환경변수만 설정하면 자동으로:
- 칸반 티켓 생성
- Codex 프롬프트에 KANBAN_INSTRUCTIONS 주입 (start/log/done/fail)

```bash
BATCH_ID=$BATCH_ID TICKET_NUMBER=1 SYSTEM_NAME="stress" PRIORITY="high" \
  bash tools/codex_dispatch.sh tickets/t-XXX.md
```

#### 경로 B: `ask_codex` MCP 사용 시 → 칸반 수동 처리 필수 ⚠️

`ask_codex` MCP 도구는 칸반 연동이 없다. 따라서 다음을 **직접** 해야 한다:

**Step 1: 칸반 티켓 생성 (dispatch 전)**
```bash
TICKET_ID=$(kanban_create_ticket "티켓 제목" "$BATCH_ID" "codex" 1 "시스템명" "high")
```

**Step 2: 프롬프트 파일에 KANBAN_INSTRUCTIONS 수동 삽입**
프롬프트 .md 파일 끝에 다음을 추가:
```
--- KANBAN INTEGRATION ---
Run these at the appropriate times (fail silently if server unavailable):
  Start:    source tools/kanban/scripts/kanban_helpers.sh && kanban_start "{TICKET_ID}" "codex-agent"
  Progress: kanban_log "{TICKET_ID}" "info" "description of progress"
  Done:     kanban_done "{TICKET_ID}"
  Failed:   kanban_fail "{TICKET_ID}" "error description"
--- END KANBAN ---
```
{TICKET_ID}는 Step 1에서 받은 실제 ID로 치환한다.

**Step 3: MCP 도구로 dispatch**
```
ask_codex(
  agent_role: "developer",
  prompt_file: ".omc/prompts/ticket-name.md",
  output_file: ".omc/prompts/ticket-name-response.md"
)
```
- prompt_file은 **워킹 디렉토리 기준 상대경로**여야 한다 (strict path policy)
- `/tmp/` 등 워킹 디렉토리 밖은 에러 발생

**Step 4: 결과 확인 후 칸반 업데이트**
- Codex가 칸반 업데이트를 못 했다면, 직접 done/failed 처리:
```bash
kanban_direct_done "$TICKET_ID"   # 또는
kanban_direct_fail "$TICKET_ID" "에러 내용"
```

### 칸반 서버가 꺼져 있어도 작업은 중단하지 않는다
- curl에 `-sf` 플래그를 사용하므로, 칸반 서버가 없어도 에러 무시하고 작업 계속 진행
- 칸반은 모니터링 도구이지 작업 의존성이 아님

---

## Part 2: Codex Dispatch Rules

### ⚠️ DISPATCH TOOL ROUTING [ABSOLUTE RULE — READ THIS FIRST]

You have multiple tools available. Only specific tools count as "dispatching to Codex":

**✅ VALID Codex dispatch methods:**
- `bash tools/codex_dispatch.sh tickets/<file>.md` — shell script dispatch (GDScript/Godot 전용)
- `ask_codex` MCP tool — MCP Codex dispatch (모든 언어/프로젝트)

**❌ INVALID — these are NOT Codex dispatch:**
- `Task` tool (Claude sub-agent) — sends work to another Claude instance, NOT Codex. Counts as DIRECT.
- Implementing the code yourself — obviously not dispatch.

**Before every dispatch action, check:**
1. Am I about to call `ask_codex` or `codex_dispatch.sh`? → ✅ Proceed
2. Am I about to call `Task` tool? → ❌ STOP. Route to `ask_codex` or `codex_dispatch.sh` instead.
3. Am I about to write the code myself? → Only if classified 🔴 DIRECT with justification in PROGRESS.md.

**Task tool is for lead-internal work only** (research, analysis, codebase exploration).
Task tool must NEVER be used for implementation tickets classified as 🟢 DISPATCH.

---

### ⚠️ CRITICAL RULE: Default is DISPATCH, not implement directly.

When you create tickets, the DEFAULT action is to dispatch them to Codex.
You may only implement directly if **ALL THREE** of the following are true:
1. The change modifies shared interfaces (SimulationBus signals, GameConfig schema, EntityManager API)
2. The change is pure integration wiring (<50 lines, connecting already-implemented pieces)
3. The change cannot be split into any smaller independent unit

If even ONE file in the ticket is a standalone change, split it out and dispatch that part.

**You MUST justify in writing why you are NOT dispatching — BEFORE implementing.**
Write this justification in PROGRESS.md first:
```
[DIRECT] t-XXX: <reason why this cannot be dispatched>
```
If you cannot articulate a clear reason, dispatch it.

---

### How to split "cross-system" work for dispatch

Most "cross-system" features CAN be split. **"This is cross-system" is NOT a valid reason to skip dispatch.**

Example: "Add resource gathering system"
- ❌ WRONG: "This is cross-system, I'll do it all myself"
- ✅ RIGHT:
  - t-301: Add ResourceMap data class (standalone new file) → 🟢 DISPATCH
  - t-302: Add GatheringSystem (standalone new file) → 🟢 DISPATCH
  - t-303: Wire into main.gd, add signals → 🔴 DIRECT (integration wiring)
  - t-304: Add tests → 🟢 DISPATCH

The ONLY parts you implement directly are signal definitions and final wiring (usually <50 lines each).

---

### How to dispatch coupled/balance changes (Config-first, Fan-out)

**"Files overlap so I can't dispatch" is NOT a valid reason for 0% dispatch.**
When files overlap, use **sequential dispatch** instead of parallel.

**Pattern: Config-first, then fan-out**

```
Step 1: 🔴 DIRECT — Shared config changes (game_config.gd etc.) first. Commit.
Step 2: 🟢 DISPATCH (sequential) — Systems that depend on config, one at a time:
  t-501: entity_data.gd changes → dispatch, wait for completion
  t-502: needs_system.gd changes → dispatch (depends on t-501)
  t-503: construction_system.gd → dispatch (parallel with t-502, different file)
Step 3: 🔴 DIRECT — Final integration wiring + verification
```

Key principles:
- **Sequential dispatch is still dispatch.** It counts toward dispatch ratio.
- **"Can't parallelize" ≠ "Can't dispatch."** These are different things.
- Config first → all dependencies flow one direction.

❌ Bad (0% dispatch):
```
| t-500 | 🔴 DIRECT | config + entity + needs 3 files at once   |
| t-510 | 🔴 DIRECT | behavior + job 2 files at once            |
| t-520 | 🔴 DIRECT | config(overlap) + construction + behavior |
Dispatch ratio: 0/3 = 0% ❌
```

✅ Good (86% dispatch, same work):
```
| t-500 | 🔴 DIRECT   | game_config.gd balance constants (shared config)    |
| t-501 | 🟢 DISPATCH | entity_data.gd starving_timer field                 |
| t-502 | 🟢 DISPATCH | needs_system.gd starvation grace (after t-501)      |
| t-503 | 🟢 DISPATCH | construction_system.gd build_ticks (after t-500)    |
| t-504 | 🟢 DISPATCH | population_system.gd birth relaxation (after t-500) |
| t-505 | 🟢 DISPATCH | behavior+job override (after t-500)                 |
| t-506 | 🟢 DISPATCH | movement_system.gd auto-eat (after t-502)           |
Dispatch ratio: 6/7 = 86% ✅
```

---

### Dispatch Decision Tree

```
New ticket created
  │
  ├─ Pure new file? (new system, new data class, new test)
  │   └─ ALWAYS DISPATCH. No exceptions.
  │
  ├─ Single-file modification? (tuning, bug fix, config change)
  │   └─ ALWAYS DISPATCH. No exceptions.
  │
  ├─ Modifies ONLY shared interfaces? (signals, schemas, base APIs)
  │   └─ DIRECT. Log reason in PROGRESS.md.
  │
  ├─ Modifies shared interfaces AND implementation files?
  │   └─ SPLIT: shared interface → DIRECT, implementation → DISPATCH
  │
  ├─ Multiple files overlapping with other tickets?
  │   └─ DON'T skip dispatch. Use Config-first fan-out.
  │       1. DIRECT the shared config
  │       2. Sequential DISPATCH the rest
  │
  └─ Integration wiring? (<50 lines, connecting dispatched work)
      └─ DIRECT. This is your core job.
```

---

### Dispatch Commands

#### 방법 1: `codex_dispatch.sh` (GDScript/Godot 전용)

```bash
# Single ticket
bash tools/codex_dispatch.sh tickets/t-010-fix-input.md

# Parallel dispatch (no file overlap, max 3)
bash tools/codex_dispatch.sh tickets/t-301-resource-map.md &
bash tools/codex_dispatch.sh tickets/t-302-gathering-system.md &
bash tools/codex_dispatch.sh tickets/t-304-gathering-tests.md &
wait

# Sequential dispatch (config-first pattern)
bash tools/codex_dispatch.sh tickets/t-501-entity-data.md
# wait for completion...
bash tools/codex_dispatch.sh tickets/t-502-needs-system.md &
bash tools/codex_dispatch.sh tickets/t-503-construction.md &
wait

# Check status
bash tools/codex_status.sh

# Apply + gate verify
bash tools/codex_apply.sh
```

#### 방법 2: `ask_codex` MCP (모든 언어/프로젝트)

`ask_codex`는 shell 명령이 아니라 oh-my-claudecode MCP 플러그인의 도구다.

```bash
# Step 1: 프롬프트 파일을 워킹 디렉토리 내에 생성
mkdir -p .omc/prompts
cat > .omc/prompts/ticket-name.md << 'PROMPT_EOF'
(프롬프트 내용 — 티켓 상세, 구현 지침)

--- KANBAN INTEGRATION ---
Run these at the appropriate times (fail silently if server unavailable):
  Start:    source tools/kanban/scripts/kanban_helpers.sh && kanban_start "TICKET_ID_HERE" "codex-agent"
  Progress: kanban_log "TICKET_ID_HERE" "info" "description of progress"
  Done:     kanban_done "TICKET_ID_HERE"
  Failed:   kanban_fail "TICKET_ID_HERE" "error description"
--- END KANBAN ---
PROMPT_EOF

# Step 2: MCP 도구 호출
# ask_codex(
#   agent_role: "developer",
#   prompt_file: ".omc/prompts/ticket-name.md",
#   output_file: ".omc/prompts/ticket-name-response.md"
# )

# Step 3: 응답 확인
# cat .omc/prompts/ticket-name-response.md
```

**⚠️ 중요:**
- `prompt_file`은 반드시 **워킹 디렉토리 기준 상대경로** (strict path policy)
- `/tmp/` 등 워킹 디렉토리 밖 경로는 에러 발생
- `codex_dispatch.sh`와 달리 칸반 자동 처리 없음 → Part 1의 "경로 B" 참조

---

### Dispatch 경로 선택 기준

```
GDScript/Godot 파일?
  ├─ YES → codex_dispatch.sh 사용 (칸반 자동, AGENTS.md 자동 참조)
  └─ NO  → ask_codex MCP 사용 (Python, JS, Rust 등)
            ⚠️ 칸반 티켓 수동 생성 + KANBAN_INSTRUCTIONS 수동 삽입 필수
```

---

## Part 3: Autopilot Workflow

When the user gives a feature request:

1. **Plan** — Split into 5–10 tickets. Each ticket targets 1–2 files max. If 3+ files, split further.
   **Before writing a single line of code, count your tickets.
   If you have fewer than 3 tickets for any non-trivial feature, you have not split enough. Re-split.**

2. **Sequence** — Order by dependency. Identify parallel vs sequential.

3. **Classify each ticket:**
   - 🟢 DISPATCH: New file, single system change, test, config change, bug fix
   - 🔴 DIRECT: Shared interface, signal schema, integration wiring (<50 lines)
   - **If >40% are DIRECT → split is wrong. Re-split until ≥60% dispatch.**
   - **If files overlap → Config-first fan-out. Do NOT mark all as DIRECT.**

4. **Write PROGRESS.md FIRST** — classification table before any code.

5. **Dispatch first, then direct** — ALL 🟢 tickets to Codex before starting 🔴 work.
   Use `ask_codex` or `codex_dispatch.sh` — **NEVER Task tool for 🟢 tickets.**

6. **Gate** — `bash scripts/gate.sh` after each integration.

7. **Fix failures** — gate fails → analyze and fix. Codex caused it → re-dispatch with clearer ticket.

8. **Do not ask** the user for additional commands. Make reasonable defaults.

9. **Update PROGRESS.md** with results.

10. **Summarize** — dispatch ratio, tool used, DIRECT reasons, files changed.

---

## Part 4: PROGRESS.md — Mandatory Logging

PROGRESS.md lives at the project root. Append-only — never delete past entries.

### When to write

- **Before starting any batch**: Log the classification table
- **Before each DIRECT implementation**: Log the `[DIRECT]` justification
- **After completing a batch**: Log results

### Format

```markdown
## [Feature Name] — [Ticket Range]

### Context
[1-2 sentences: what problem this batch solves]

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| t-XXX | ... | 🟢 DISPATCH | ask_codex         | standalone new file |
| t-XXX | ... | 🟢 DISPATCH | codex_dispatch.sh | single system |
| t-XXX | ... | 🔴 DIRECT   | —                 | shared config |
| t-XXX | ... | 🔴 DIRECT   | —                 | integration wiring <50 lines |

### Dispatch ratio: X/Y = ZZ% ✅/❌ (target: ≥60%)

### Dispatch strategy
[parallel / sequential / config-first-fan-out — explain order and dependencies]

### Localization Verification
- Hardcoded scan: PASS / FAIL
- New keys added: [list or none]
- ko/ updated: YES / NO

### Results
- Gate: PASS / FAIL
- Dispatch ratio: X/Y = ZZ%
- Files changed: [count]
- Dispatch tool used: ask_codex (N tickets)
```

### Rules
- **Never delete past entries.** Append-only.
- **Always log BEFORE implementing**, not after. This forces planning before coding.
- **If dispatch ratio is <60%, STOP and re-split** before proceeding.
- **Log which dispatch tool was used** — makes it auditable that Codex, not Task tool, was used.

---

## Part 5: Ticket Template

Every ticket in `tickets/` must include:

```markdown
## Objective
[One sentence: what this ticket delivers]

## Non-goals
[What this ticket explicitly does NOT do — required, prevents scope creep]

## Scope
Files to create/modify:
- path/to/file.gd — [what changes]
- path/to/test.gd — [what test to add]

## Acceptance Criteria
- [ ] worldsim-code SKILL.md Part 1 verified (localization scan — even if no new text)
- [ ] Dispatch ratio confirmed ≥60% in PROGRESS.md
- [ ] Smoke test: [command that completes in <30s]
- [ ] Gate passes: bash scripts/gate.sh

## Risk Notes
- Perf: [expected impact on tick time]
- Signals: [any signal changes — if yes, lead must review]
- Data: [any EntityData/WorldData schema changes]

## Context
[Links to relevant code, prior tickets, or architecture docs]
```

**Quality bar:** If Codex needs to ask a follow-up question, the ticket was underspecified. Rewrite it.

---

## Part 6: Common Workflow Mistakes

These are dispatch/kanban/workflow mistakes. For code mistakes → see CLAUDE.md.

1. **Writing Codex tickets without Non-goals** — Codex will scope-creep without explicit boundaries.
2. **Dispatching architecture work to Codex** — shared interfaces stay in lead. Always.
3. **Dispatching overlapping tickets in parallel** — check file scopes first. Merge conflicts cost more than sequential.
4. **Implementing tickets directly without logging** — default is DISPATCH. Log every DIRECT in PROGRESS.md BEFORE coding.
5. **Skipping PROGRESS.md** — write the classification table BEFORE coding. No PROGRESS.md = planning step skipped.
6. **Using Task tool for 🟢 DISPATCH tickets** — Task tool = Claude sub-agents, NOT Codex. Counts as DIRECT.
7. **Claiming "cross-system" to skip dispatch** — almost always splittable. Split first, then decide.
8. **Claiming "files overlap" to skip dispatch** — use Config-first fan-out. "Can't parallelize" ≠ "can't dispatch".
9. **Dispatch ratio below 60%** — if >40% are DIRECT, re-split before proceeding.
10. **Using ask_codex without kanban registration** — ask_codex has NO built-in kanban. Must manually create ticket + insert KANBAN_INSTRUCTIONS into prompt file.
11. **Putting ask_codex prompt files outside workdir** — strict path policy. Use `.omc/prompts/` relative path only.
12. **Using `kanban_direct_start` in pre-creation scripts** — creates ticket AND sets in_progress immediately, violating dependency order. Use `kanban_create_ticket` for pre-creation, then manually start later.