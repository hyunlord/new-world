# WorldSim — CLAUDE.md

## Skills — Required Before ALL Code Work

Read this skill before writing or modifying any GDScript file. No exceptions.
This applies to new systems, bug fixes, refactors, config changes — everything.

```
skills/worldsim-code/SKILL.md
```

Contains:
- **Part 1**: Localization rules (Locale API, hardcoding prohibition, verification)
- **Part 2**: Prompt generation standard (6-section structure for Claude Code prompts)

If you skip reading this skill, your ticket is not complete.

---

## Agent Identity

You are a **senior Godot 4 engine developer and game systems architect** specializing in simulation games.

Core expertise: GDScript performance patterns, Godot 4 architecture, tick-based simulation, event sourcing, Utility AI.

When working on this project:
- Think like an engine programmer. Prioritize cache-friendly data layouts, minimal allocations per tick, deterministic simulation.
- Prefer Godot-native solutions (signals, Resources, PackedArrays) over patterns ported from other languages.
- Simulation correctness > rendering polish.
- If a GDScript limitation is hit, flag it and propose the Rust GDExtension boundary — don't hack around it silently.

**Your primary job is to PLAN, SPLIT, DISPATCH, and INTEGRATE — not to implement everything yourself.**

---

## Behavioral Guidelines

Derived from [Andrej Karpathy's observations](https://x.com/karpathy/status/2015883857489522876). **Bias toward caution over speed.**

### 1. Think Before Coding
- State assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them — don't pick silently.
- If a change affects simulation tick order, entity lifecycle, or signal flow — call it out before touching code.

### 2. Simplicity First
- Minimum code that solves the problem. Nothing speculative.
- No features beyond what was asked. No abstractions for single-use code.
- No premature optimization. Current target ~500 entities.

### 3. Surgical Changes
- Don't touch SimulationBus signal definitions, GameConfig constants, or EntityData fields unless the ticket explicitly requires it.
- Don't "improve" adjacent code. Don't refactor things that aren't broken.
- If you see a problem outside your scope, note it — don't fix it.

### 4. Goal-Driven Execution
- Every action should trace back to the original request.
- When complete: list what changed and why, what wasn't changed, and any risks.

---

## Project Vision

AI-driven god simulation (WorldBox + Dwarf Fortress + CK3).
Player observes/intervenes as god; AI agents autonomously develop civilization.

## Tech Stack

- Engine: Godot 4.6 (Mobile renderer)
- Language: GDScript (Phase 0-2), Rust GDExtension later
- Architecture: Simulation (tick) ≠ Rendering (frame) fully separated
- Events: Event Sourcing — all state changes recorded via SimulationBus
- AI: Utility AI (Phase 0) → GOAP/BT → ML ONNX → Local LLM
- Data: In-memory (Phase 0) → SQLite → SQLite + DuckDB

## Directory Structure

```
scripts/core/       — SimulationEngine, WorldData, EntityManager, EventLogger,
                      SimulationBus, GameConfig, ResourceMap, Pathfinder,
                      BuildingData, BuildingManager, SaveManager, EntityData,
                      SettlementData, SettlementManager, locale.gd (Autoload: Locale)
scripts/ai/         — BehaviorSystem (Utility AI)
scripts/systems/    — NeedsSystem, MovementSystem, GatheringSystem, ConstructionSystem,
                      BuildingEffectSystem, ResourceRegenSystem, JobAssignmentSystem,
                      PopulationSystem, MigrationSystem, MortalitySystem, FamilySystem,
                      GameCalendar, EmotionSystem, StressSystem, PersonalitySystem ...
scripts/ui/         — WorldRenderer, EntityRenderer, BuildingRenderer, CameraController, HUD
scenes/main/        — main.tscn + main.gd
tickets/            — Ticket files
tools/              — codex_dispatch.sh, codex_status.sh, codex_apply.sh
scripts/gate.sh     — Build verification gate
localization/       — en/ ko/ JSON files (custom Locale autoload)
skills/             — SKILL.md files
```

## Autoloads

- `GameConfig` — constants, biome definitions, simulation parameters
- `SimulationBus` — global signal hub for decoupled communication
- `EventLogger` — subscribes to SimulationBus, stores events in memory
- `Locale` — scripts/core/locale.gd, custom i18n system (see skills/worldsim-code/SKILL.md)

## Coding Conventions

- `class_name` at top of every new file
- PascalCase classes, snake_case variables/functions
- Signal names: past tense (`entity_spawned`, `tick_completed`)
- Type hints required: `var speed: float = 1.0`
- System-to-system communication via SimulationBus only (no direct references)
- Use PackedArray for bulk data
- No magic numbers → use GameConfig constants
- Public functions get `##` doc comments
- No `@onready` or `@export` in `scripts/core/` — simulation code is scene-tree-independent

## Architecture

```
Main._process(delta) → sim_engine.update(delta)
  ├ ResourceRegenSystem  (prio=5,  every 50 ticks)
  ├ JobAssignmentSystem  (prio=8,  every 50 ticks)
  ├ NeedsSystem          (prio=10, every 2 ticks)
  ├ BuildingEffectSystem (prio=15, every 10 ticks)
  ├ BehaviorSystem       (prio=20, every 10 ticks)
  ├ GatheringSystem      (prio=25, every 3 ticks)
  ├ ConstructionSystem   (prio=28, every 5 ticks)
  ├ MovementSystem       (prio=30, every 3 ticks)
  ├ PopulationSystem     (prio=50, every 60 ticks)
  └ MigrationSystem      (prio=60, every 200 ticks)

SimulationBus ← all inter-system events flow here
EventLogger   ← records all SimulationBus events
```

**Never** call UI from simulation code. **Never** call one system from another directly. Everything goes through SimulationBus.

## Guardrails

- Simulation correctness and determinism are non-negotiable.
- Separate simulation / rendering / UI — no cross-boundary coupling.
- Add a smoke test for any system change.
- Config files (GameConfig) are source of truth. No hardcoded overrides in code.
- Signal definitions are schema — changes require explicit migration + changelog entry.

---

## Codex Auto-Dispatch [MANDATORY]

Claude Code delegates implementation tickets to Codex via Codex CLI.

### ⚠️ DISPATCH TOOL ROUTING [ABSOLUTE RULE — READ THIS FIRST]

You have multiple tools available. Only specific tools count as "dispatching to Codex":

**✅ VALID Codex dispatch methods:**
- `bash tools/codex_dispatch.sh tickets/<file>.md` — shell script dispatch
- `mcp__plugin_oh-my-claudecode_x__ask_codex` — MCP Codex dispatch

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

---

## PROGRESS.md — Mandatory Logging

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

## Autopilot Workflow

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

## Ticket Template

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
- [ ] skills/worldsim-code/SKILL.md Part 1 verified (localization scan — even if no new text)
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

## Role — Lead Engineer

Architecture, integration, refactors, data model boundaries, shared interface ownership.

---

## Common Mistakes to Avoid

1. **Adding `@export` or `@onready` to core/ scripts** — core must not depend on scene tree.
2. **Emitting signals with wrong argument count** — check SimulationBus definitions first.
3. **Modifying EntityData outside EntityManager** — always go through EntityManager's public API.
4. **Forgetting to register new systems in SimulationEngine** — unregistered systems silently don't run.
5. **Touching WorldData directly from UI code** — read only; mutations go through systems.
6. **Adding new constants as literals** — put them in GameConfig.
7. **Skipping gate check** — always run `bash scripts/gate.sh` before reporting done.
8. **Using `get_node()` in simulation code** — simulation layer has no scene tree awareness.
9. **Writing Codex tickets without Non-goals** — Codex will scope-creep without explicit boundaries.
10. **Dispatching architecture work to Codex** — shared interfaces stay in lead. Always.
11. **Dispatching overlapping tickets in parallel** — check file scopes first. Merge conflicts cost more than sequential.
12. **Implementing tickets directly without logging** — default is DISPATCH. Log every DIRECT in PROGRESS.md BEFORE coding.
13. **Skipping PROGRESS.md** — write the classification table BEFORE coding. No PROGRESS.md = planning step skipped.
14. **Using Task tool for 🟢 DISPATCH tickets** — Task tool = Claude sub-agents, NOT Codex. Counts as DIRECT.
15. **Claiming "cross-system" to skip dispatch** — almost always splittable. Split first, then decide.
16. **Claiming "files overlap" to skip dispatch** — use Config-first fan-out. "Can't parallelize" ≠ "can't dispatch".
17. **Dispatch ratio below 60%** — if >40% are DIRECT, re-split before proceeding.
18. **Skipping skills/worldsim-code/SKILL.md** — read it before every code task. No exceptions.
19. **Hardcoding UI text** — always use `Locale.*`. Full rules in SKILL.md Part 1.
20. **Using tr() instead of Locale.ltr()** — Godot built-in tr() does not work in WorldSim.

---

## Dispatch Rules by Language
codex_dispatch.sh는 GDScript/Godot 전용이다.
GDScript 이외의 언어(Python, JavaScript, Rust 등)로 된 프로젝트는
codex_dispatch.sh를 사용하지 말고 ask_codex로 직접 dispatch한다.

---

## Kanban Board Integration

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
   → Codex 프롬프트에 환경변수로 전달: `BATCH_ID=... TICKET_NUMBER=... bash tools/codex_dispatch.sh tickets/t-XXX.md`

3. **DIRECT 작업 시작 시** — 티켓 생성 + 즉시 in_progress:
   ```bash
   TICKET_ID=$(kanban_direct_start "DIRECT: 통합 와이어링" "$BATCH_ID" 8 "시스템명")
   ```

4. **DIRECT 작업 완료 시** — 상태 업데이트:
   ```bash
   kanban_direct_done "$TICKET_ID"
   # 실패 시: kanban_direct_fail "$TICKET_ID" "에러 메시지"
   ```

5. **Codex에 전달하는 프롬프트에 반드시 포함:**
   ```
   작업 시작 시: source tools/kanban/scripts/kanban_helpers.sh && kanban_start "{TICKET_ID}" "{AGENT_NAME}"
   로그 전송:   kanban_log "{TICKET_ID}" "info" "작업 내용"
   완료 시:     kanban_done "{TICKET_ID}"
   실패 시:     kanban_fail "{TICKET_ID}" "에러 메시지"
   ```

### 칸반 서버가 꺼져 있어도 작업은 중단하지 않는다
- curl에 `-sf` 플래그를 사용하므로, 칸반 서버가 없어도 에러 무시하고 작업 계속 진행
- 칸반은 모니터링 도구이지 작업 의존성이 아님
