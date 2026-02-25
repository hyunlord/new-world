---
name: worldsim-code
description: |
  Required for ALL WorldSim GDScript work.
  Part 1: Localization rules — enforced on every code change.
  Part 2: Prompt generation standard — 6-section structure for Claude Code prompts.
  Part 3: Prompt engineering lessons — failure patterns and solutions from real development.
  Read all parts. No exceptions.
---

# WorldSim Code Skill

---

# PART 1 — Localization

## Core Premise

WorldSim does NOT use `TranslationServer` or `tr()`.
All player-facing text must go through the custom Autoload `Locale` (`scripts/core/locale.gd`).

---

## Locale API Reference

| API | When to use | Example |
|-----|-------------|---------|
| `Locale.ltr("KEY")` | Simple text lookup | `Locale.ltr("UI_POPULATION")` |
| `Locale.trf("KEY", {...})` | Text with placeholders | `Locale.trf("EVENT_BORN", {"name": e.name})` |
| `Locale.tr_id("PREFIX", id)` | PREFIX_ID pattern | `Locale.tr_id("JOB", entity.job)` → `JOB_GATHERER` |
| `Locale.get_month_name(n)` | Month names 1–12 | `Locale.get_month_name(3)` |
| `Locale.set_locale("en")` | Runtime language switch | Settings screen only |
| ~~`tr_data()`~~ | **deprecated** | Do NOT use. Report if found. Do NOT fix. |

**NEVER use:** `tr()`, `TranslationServer.translate()`, `tr_data()`

---

## JSON File Structure

```
localization/
├── en/
│   ├── ui.json        — UI_* keys (panels, buttons, HUD, notifications)
│   ├── game.json      — JOB_*, STATUS_*, ACTION_*, GAME_* keys
│   ├── traits.json    — TRAIT_* keys
│   ├── emotions.json  — EMOTION_* keys
│   ├── events.json    — EVENT_* keys (placeholder templates)
│   ├── deaths.json    — DEATH_* keys
│   ├── buildings.json — BUILDING_* keys
│   ├── tutorial.json  — TUTORIAL_* keys
│   ├── debug.json     — DEBUG_* keys (debug build only)
│   ├── coping.json    — COPING_* keys
│   └── childhood.json — CHILDHOOD_* keys
└── ko/               (identical structure, Korean translations)
```

When adding new keys: **update en/ AND ko/ simultaneously.** One side only = incomplete.

---

## Localization Rules

### Rule 1: No hardcoded strings — ever

```gdscript
# ❌ FORBIDDEN
label.text = "Population"
label.text = "인구"
log_msg("Entity died")

# ✅ CORRECT
label.text = Locale.ltr("UI_POPULATION")
log_msg(Locale.trf("EVENT_ENTITY_DIED", {"name": entity.name}))
```

### Rule 2: Choose the right API for the pattern

```gdscript
# Simple text
title.text = Locale.ltr("UI_PANEL_ENTITY_DETAIL")

# With variables
text = Locale.trf("EVENT_BORN", {"name": e.name, "mother": m.name, "father": f.name})

# ID-based (Job / Status / Action)
label.text = Locale.tr_id("JOB", entity.job)       # → JOB_GATHERER
label.text = Locale.tr_id("STATUS", entity.status)  # → STATUS_HUNGRY
label.text = Locale.tr_id("ACTION", entity.action)  # → ACTION_GATHER_WOOD

# Month name
text = Locale.get_month_name(calendar.month)
```

### Rule 3: Key naming convention

```
UI_*                          → ui.json
JOB_* STATUS_* ACTION_* GAME_* → game.json
TRAIT_*                       → traits.json
EMOTION_*                     → emotions.json
EVENT_*                       → events.json
DEATH_*                       → deaths.json
BUILDING_*                    → buildings.json
COPING_*                      → coping.json
CHILDHOOD_*                   → childhood.json
DEBUG_*                       → debug.json  (must be inside OS.is_debug_build() guard)
```

### Rule 4: DEBUG keys require a debug guard

```gdscript
# ONLY inside this guard
if OS.is_debug_build():
    print(Locale.ltr("DEBUG_TICK_INFO"))
```

---

## Exceptions (Locale NOT required)

```gdscript
var job: String = "gatherer"            # Internal logic ID, never shown to player
var path: String = "user://save.json"   # File path
label.text = "⏸" if paused else "▶"    # Unicode symbol, language-independent
lbl.text = " " + text                   # Pure whitespace padding
if OS.is_debug_build(): print("DEBUG")  # Debug-only print
```

---

## Localization Pre-Work Checklist

Before writing or modifying any .gd file:

- [ ] Does this work introduce any new player-facing text?
- [ ] Does existing code in scope have hardcoded strings? (fix them if in scope)
- [ ] Do the required keys already exist in JSON? (avoid duplicates)
- [ ] Is `tr_data()` used anywhere in scope? (report, do NOT fix)

---

## Localization Post-Work Verification

Run against every .gd file changed in this ticket.

### Step 1: Hardcoded string scan

```bash
# Detect .text = "literal" without Locale wrapper (run from repo root)
grep -rn '\.text\s*=\s*"' scripts/ui/ | grep -v 'Locale\.'

# Detect direct string in print() outside debug guard
grep -n 'print\s*(\s*"' <file>.gd | grep -v 'OS.is_debug_build'
```

Any result → **VIOLATION. Fix and re-verify.**

### Step 2: Verify new keys exist in both JSON files

```bash
# Extract all keys referenced in changed file
grep -oh 'ltr("[^"]*")' <file>.gd    | sed 's/ltr("//;s/")//'
grep -oh 'trf("[^"]*",' <file>.gd    | sed 's/trf("//;s/",//'
grep -oh 'tr_id("[^"]*"' <file>.gd   | sed 's/tr_id("//;s/")//'

# Verify each key exists in the appropriate en/ and ko/ JSON
```

Missing key → **add to both en/ and ko/ before reporting done.**

### Step 3: Check deprecated API

```bash
grep -rn 'tr_data(' scripts/
```

If found → **do NOT fix. Report in Out-of-Scope Issues.**

---

## Known Existing Issues (do not fix without a ticket)

| File | Line | Issue | Priority |
|------|------|-------|----------|
| pause_menu.gd | 352 | `"Slot %d %s"` hardcoded → replaceable with `UI_SLOT_FORMAT` | Low |

---

---

# PART 2 — Prompt Generation Standard

## When This Applies

Any time a prompt is written for Claude Code — whether by the lead (Claude Code itself)
or by the user via Claude.ai. Every prompt sent to Claude Code must meet this standard.

---

## Required Structure

Every Claude Code prompt MUST contain all 6 sections below. No exceptions.

---

### Section 1: Implementation Intent
**What problem does this solve? Why now?**

- What user-visible or system-level problem is being solved
- Why this approach was chosen over alternatives
- What academic reference, theory, or prior art informs the design (if applicable)
- What constraints or tradeoffs were accepted

Example:
```
## Implementation Intent
WorldSim agents currently have no memory of emotional states between ticks.
This causes unrealistic behavior where trauma has no lasting effect.
Based on van der Kolk's somatic stress theory and Lazarus's appraisal model,
we implement a persistent TraumaRecord attached to EntityData.
Tradeoff: memory overhead per entity is ~200 bytes — acceptable at Phase 0 scale (~500 entities).
```

---

### Section 2: What to Build
**Exactly what is being implemented.**

- System name, file paths, class names
- Data structures, fields, types, default values
- Signals to emit or listen to
- GameConfig constants to add
- Localization keys to add (en/ and ko/)
- Precise scope — what is IN and what is explicitly OUT

Example:
```
## What to Build
Create `scripts/systems/trauma_system.gd` (class_name TraumaSystem).
Add `trauma_records: Array[TraumaRecord]` to EntityData.
Add signal `trauma_recorded(entity_id, trauma_type, severity)` to SimulationBus.
Add constants to GameConfig: TRAUMA_DECAY_RATE = 0.001, TRAUMA_THRESHOLD = 0.3
Add localization keys:
  - EVENT_TRAUMA_TRIGGERED → en: "{name} experienced trauma", ko: "{name}이(가) 트라우마를 경험했습니다"
NOT in scope: trauma UI panel, therapy mechanics, intergenerational transmission.
```

---

### Section 3: How to Implement
**Step-by-step implementation with exact logic.**

- Execution flow (tick order, priority, interval)
- Algorithms, formulas, state transitions
- Condition branches with exact values
- Code snippets for non-obvious logic
- Integration points (which system calls what, through which signal)

Example:
```
## How to Implement
TraumaSystem runs at priority=45, every 30 ticks.
On each tick:
  for each entity:
    if entity.stress > TRAUMA_THRESHOLD:
      severity = (entity.stress - TRAUMA_THRESHOLD) / (1.0 - TRAUMA_THRESHOLD)
      record = TraumaRecord.new(type, severity, current_tick)
      entity.trauma_records.append(record)
      SimulationBus.emit_signal("trauma_recorded", entity.id, type, severity)
    for record in entity.trauma_records:
      record.severity -= TRAUMA_DECAY_RATE  # decay over time
      if record.severity <= 0:
        entity.trauma_records.erase(record)
```

---

### Section 4: Dispatch Plan
**How to split and dispatch this work.**

- Ticket breakdown (one ticket = one file or one concern)
- Classification: 🟢 DISPATCH or 🔴 DIRECT for each ticket
- Dependency order (which must complete before which)
- Dispatch method: `ask_codex` or `codex_dispatch.sh`
- Target dispatch ratio (must be ≥60%)

Example:
```
## Dispatch Plan
| Ticket | File | Action | Depends on |
|--------|------|--------|------------|
| t-701 | TraumaRecord data class | 🟢 DISPATCH | — |
| t-702 | EntityData: add trauma_records field | 🟢 DISPATCH | t-701 |
| t-703 | TraumaSystem logic | 🟢 DISPATCH | t-702 |
| t-704 | GameConfig: TRAUMA_* constants | 🔴 DIRECT | — |
| t-705 | SimulationBus: trauma_recorded signal | 🔴 DIRECT | — |
| t-706 | Wire TraumaSystem into SimulationEngine | 🔴 DIRECT | t-703,705 |
| t-707 | Tests | 🟢 DISPATCH | t-703 |

Dispatch ratio: 4/7 = 57% → re-split t-706 if possible to hit ≥60%
Order: t-704,705 (DIRECT, parallel) → t-701 → t-702 → t-703,707 (parallel) → t-706
```

---

### Section 5: Localization Checklist
**Every key that must be added.**

- List every new localization key
- Specify which JSON file each belongs to
- Provide both en/ and ko/ values
- Confirm no existing key is duplicated

Example:
```
## Localization Checklist
| Key | File | en | ko |
|-----|------|----|----|
| EVENT_TRAUMA_TRIGGERED | events.json | "{name} experienced trauma" | "{name}이(가) 트라우마를 경험했습니다" |
| STATUS_TRAUMATIZED | game.json | "Traumatized" | "트라우마 상태" |
| DEBUG_TRAUMA_DECAY | debug.json | "Trauma decay: {value}" | "트라우마 감쇠: {value}" |
```

If no new keys: write "No new localization keys."

---

### Section 6: Verification
**How to confirm it works.**

- Gate command to run
- Smoke test (specific in-game action or debug command, completes in <30s)
- What to observe in output/logs to confirm correct behavior
- Edge cases to manually test

Example:
```
## Verification
- Gate: bash scripts/gate.sh → PASS
- Smoke test: spawn 5 entities, set stress > 0.3, run 100 ticks
  → trauma_records should be non-empty for affected entities
  → SimulationBus should have emitted trauma_recorded N times
- Edge case: entity with stress exactly at TRAUMA_THRESHOLD → no record created
- Edge case: trauma record decays to 0 → removed from array, no crash
```

---

## Quality Bar for Prompts

A prompt is ready to send to Claude Code when:

- [ ] Implementation Intent explains WHY, not just WHAT
- [ ] What to Build lists exact file paths, class names, field types
- [ ] How to Implement has actual formulas/logic, not vague descriptions
- [ ] Dispatch Plan has ≥60% DISPATCH ratio with dependency order
- [ ] Localization Checklist has both en/ and ko/ for every new key
- [ ] Verification has a concrete smoke test command (not "check if it works")

If Claude Code would need to ask a follow-up question after reading this prompt → rewrite it.

---

---

# PART 3 — Prompt Engineering Lessons (실패에서 배운 규칙들)

> 이 섹션은 실제 개발 중 발견한 문제와 해결 패턴을 기록한다.
> "무엇을 하라"는 Part 1-2에 있고, "왜 이렇게 됐는가"는 여기에 있다.
> 새로운 실패 패턴이 발견되면 이 섹션에 추가한다.

---

## Lesson 1: Conversation Compaction은 미래 지침을 삭제한다

**발견 시점:** 2025-02-25, Kanban Extension Part 1

**현상:** Claude Code가 배치는 만들었지만 개별 티켓을 하나도 생성하지 않음. 13개 파일 1490줄을 전부 DIRECT로 처리.

**원인:** 
- Step 0에서 배치 생성 → 실행됨 (이미 완료된 bash 결과라 compaction에서 보존)
- "각 티켓 시작 시 curl POST" 지침 → 아직 실행 안 된 미래 지침이라 compaction에서 요약/삭제됨
- Compaction 후 Claude Code는 "5개 기능 구현해야 함"만 기억하고 칸반 절차는 잊음

**해결 패턴: 사전 생성 (Pre-creation)**
```
❌ BAD:  "각 티켓 작업 시작 시 칸반에 등록해라" (미래 지침 → compaction에 취약)
✅ GOOD: "지금 당장 이 스크립트를 실행해서 12개 티켓을 전부 만들어라" (즉시 실행 → compaction 전 완료)
```

**적용 규칙:**
- 칸반 티켓은 **프롬프트 시작 시 한 번에 전부 생성**한다
- 생성 스크립트는 복사-붙여넣기로 한 번에 실행할 수 있는 형태로 작성한다
- 생성 결과(ID 목록)를 echo로 출력하여 compaction 후에도 변수가 살아있게 한다

---

## Lesson 2: "ask_codex로 dispatch해라"는 실행 불가능한 지시다

**발견 시점:** 2025-02-25, Kanban Extension Part 1

**현상:** Claude Code가 dispatch를 0% 수행. 프롬프트에 "ask_codex로 직접 dispatch해라"라고 명시했음에도.

**원인:**
- `ask_codex`는 shell 명령이 아님 → `which ask_codex` = NOT FOUND
- 실제 이름: `mcp__plugin_oh-my-claudecode_x__ask_codex` (oh-my-claudecode MCP 플러그인 도구)
- Claude Code: "ask_codex가 뭐지? 없는데? → 그냥 내가 직접 하자" → 100% DIRECT

**해결 패턴: 정확한 호출 절차를 3단계로 명시**
```
❌ BAD:  "ask_codex로 dispatch해라"
✅ GOOD: 
  1. 프롬프트 파일을 .omc/prompts/ 에 저장
  2. MCP tool ask_codex 호출 (agent_role, prompt_file, output_file)
  3. 응답 파일을 읽고 결과 확인
```

**적용 규칙:**
- 프롬프트에서 도구를 언급할 때는 **정확한 호출 방법**을 포함한다
- "X를 사용해라"만으로는 부족. "X를 이렇게 호출해라"까지 써야 한다
- CLAUDE.md의 "Dispatch Commands" 섹션에 두 경로(codex_dispatch.sh / ask_codex MCP)를 모두 명시

---

## Lesson 3: codex_dispatch.sh와 ask_codex MCP의 칸반 연동 차이

**발견 시점:** 2025-02-25, 코드 분석 중

**현상:** `codex_dispatch.sh` 경유 dispatch는 칸반에 잘 찍히는데, `ask_codex` 경유 dispatch는 칸반에 아무것도 안 남음.

**원인:**
- `codex_dispatch.sh` (line 34~56): BATCH_ID 환경변수가 있으면 자동으로 칸반 API 호출 + KANBAN_INSTRUCTIONS를 Codex 프롬프트에 주입
- `ask_codex` MCP: 프롬프트 파일만 전달. 칸반 관련 코드 없음.

**해결 패턴: 경로별 체크리스트**
```
codex_dispatch.sh → BATCH_ID만 설정하면 칸반 자동
ask_codex MCP     → 4단계 수동 처리:
  1. kanban_create_ticket으로 티켓 생성
  2. 프롬프트 파일에 KANBAN_INSTRUCTIONS 수동 삽입
  3. ask_codex MCP 호출
  4. 완료 후 칸반 상태 수동 업데이트
```

**적용 규칙:**
- CLAUDE.md에 두 경로를 "경로 A / 경로 B"로 명확히 분리 (2025-02-25 반영 완료)
- 프롬프트에서 dispatch 방법을 지정할 때 어떤 경로를 사용하는지 명시한다

---

## Lesson 4: 하드게이트 vs 지침 — Claude Code는 지침을 무시할 수 있다

**발견 시점:** 2025-02-25, 여러 프롬프트 반복 실행 중

**현상:** "각 티켓 시작 시 칸반에 등록해라", "dispatch ratio ≥60%를 유지해라" 등의 지침을 Claude Code가 반복적으로 무시.

**원인:**
- **지침 (instruction)**: "~해라" → Claude Code가 효율성 판단으로 스킵 가능
- **하드게이트 (gate)**: "이것이 완료되지 않으면 다음 단계 진행 금지" → 스킵하면 다음 단계를 못 함

**해결 패턴: 지침을 게이트로 변환**
```
❌ 지침:   "각 티켓 시작 시 칸반에 등록해라"
✅ 게이트: "12개 티켓 ID가 모두 출력될 때까지 코드 작성 금지. 아래 스크립트를 먼저 실행해라."

❌ 지침:   "PROGRESS.md에 기록해라"  
✅ 게이트: "PROGRESS.md에 classification table을 먼저 쓰고, 그 내용을 출력해라. 출력 없이는 dispatch 금지."
```

**적용 규칙:**
- 반드시 지켜야 하는 절차는 **Step N: 제목 (다음 Step 진행 조건: XXX)** 형태로 작성
- 검증 가능한 출력물을 요구 (echo, cat, curl 결과 등)
- "~해라"보다 "~하고, 그 결과를 보여라. 결과가 없으면 다음 진행 금지" 패턴 사용

---

## Lesson 5: 프롬프트 길이와 Claude Code의 선택적 주의력

**발견 시점:** 2025-02-25, CLAUDE.md 칸반 규칙 무시 관찰

**현상:** CLAUDE.md가 ~500줄. 칸반 규칙이 36번째 줄에 있는데도 무시됨.

**원인:**
- Claude Code는 컨텍스트 윈도우 내에서 **최근에 읽은 것 + 자주 참조되는 것**에 주의력이 집중
- CLAUDE.md 전체를 읽지만, 500줄 중 칸반 섹션은 상대적으로 적은 비중
- 특히 compaction 후에는 CLAUDE.md 원문이 아닌 요약본을 참조

**해결 패턴: 프롬프트 내 중복 강화**
```
❌ BAD:  CLAUDE.md에만 규칙을 넣고 프롬프트에서 "CLAUDE.md를 따라라"
✅ GOOD: CLAUDE.md에 규칙 + 프롬프트 Step 0에 동일 규칙의 실행 가능 버전 + gate.sh에 검증
```

**적용 규칙:**
- 중요 규칙은 **3곳에 존재**해야 한다: (1) CLAUDE.md (영구 규칙), (2) 프롬프트 Step 0 (이번 작업 강제), (3) gate.sh (사후 검증)
- 프롬프트에서 CLAUDE.md 참조만으로는 부족. 핵심 규칙은 프롬프트에 인라인으로 반복한다

---

## Lesson 6: GDScript 외 프로젝트의 dispatch 경로

**발견 시점:** 2025-02-25, Kanban 확장 작업 (Python + React)

**현상:** 칸반 보드(Python/React)를 `codex_dispatch.sh`로 dispatch하면 AGENTS.md(GDScript 규칙)를 참조하게 되어 부적절.

**원인:**
- `codex_dispatch.sh`는 GDScript/Godot 전용으로 설계됨 (AGENTS.md 참조, gate.sh 실행)
- Python/React 프로젝트에는 다른 규칙이 필요

**해결 패턴:**
```
GDScript/Godot → codex_dispatch.sh (AGENTS.md 자동 참조, gate.sh 실행)
Python/JS/Rust → ask_codex MCP (프롬프트에 직접 규칙 포함)
```

**적용 규칙:**
- CLAUDE.md에 "Dispatch 경로 선택 기준" 의사결정 트리 추가 (2025-02-25 반영 완료)
- 비-GDScript 프롬프트에는 해당 언어/프레임워크의 규칙을 프롬프트 내에 직접 포함

---

## 교훈 추가 템플릿

새로운 실패 패턴 발견 시 아래 형식으로 추가:

```markdown
## Lesson N: 제목

**발견 시점:** 날짜, 작업명
**현상:** 무엇이 잘못되었나
**원인:** 왜 발생했나
**해결 패턴:** BAD vs GOOD 비교
**적용 규칙:** 어디에 어떻게 반영했나
```