---
name: worldsim-code
description: |
  Required for ALL WorldSim GDScript work.
  Part 1: Localization rules — enforced on every code change.
  Part 2: Notion documentation update — required after every ticket completion.
  Read both parts. No exceptions.
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

# PART 2 — Notion Documentation Update

## Core Principle

**You are not storing Q&A or work history. You are evolving technical documentation.**

Every ticket changes the system. Notion must reflect the current state of the codebase.

---

## When to Run This

After every ticket completion — no exceptions:
- New system implementation
- Bug fix
- Refactor
- Config/data structure change
- Any code change whatsoever

---

## Forbidden Actions

- ❌ Do NOT create Q&A log pages, Q&A databases, or Q&A archives
- ❌ Do NOT write "Question:", "Q:", "A:", "Answer:" anywhere in a doc
- ❌ Do NOT use conversational tone in technical documentation
- ❌ Do NOT append to the bottom of a page without integrating into sections
- ❌ Do NOT add information that already exists on the page

---

## Step 1: Extract Information

From this ticket's work, extract what applies. Skip categories with nothing relevant.

| Category | What to extract |
|----------|----------------|
| **Implementation intent** | Why was it built this way? What problem does it solve? |
| **Implementation method** | What architecture, pattern, or algorithm was used? |
| **References** | Papers, theories, external models referenced |
| **Data structure** | Schemas, Enums, constants, field definitions |
| **Internal logic** | Execution flow, condition branches, state transitions, formulas |
| **Development history** | What changed vs before? Migrations? |
| **Tradeoffs** | Known limits, intentional omissions, future improvement areas |

---

## Step 2: Target Analysis

1. Determine which system/domain the extracted information belongs to
2. **Read the current Notion page for that system in full** — do not skip this
3. Check the related GitHub code and recent commits directly

Do not skip step 2. Writing without reading the existing page causes duplicates and conflicts.

---

## Step 3: Merge Into Documentation

Integrate new information into the appropriate sections. Never just append.

### Decision rules

| Situation | Action |
|-----------|--------|
| System doc page does not exist | Create new page with skeleton structure (see below) |
| Section exists but is incomplete | Expand and strengthen |
| Existing content is wrong or outdated | Rewrite — code is the source of truth |
| New info duplicates existing | Do NOT add. Keep existing if better; replace if new is more accurate |
| New Enum / constant / data structure | Update page AND register in Data Definitions DB |
| Major change | Update page AND record in Change Log DB |

### Section-level merge criteria

```
Overview section
├── System role description became more accurate → replace existing sentence
├── New system dependency discovered → add to relationship description
└── Mermaid system map needs updating → update diagram

Design Intent section
├── New design philosophy or principle revealed → add
├── New reference (paper, theory) mentioned → add
└── Contradicts existing design intent → correct based on current code

Architecture section
├── New class/node added → update classDiagram
├── Signal/event flow changed → rewrite flow diagram
└── Interface with another system changed → update that system's doc too

Data Structure section
├── New field/property → add row to table
├── Existing field type/default changed → update table row
├── New Enum/constant → add table + register in Data Definitions DB
└── Deleted field/Enum → remove from table + remove from DB

Core Logic section
├── Algorithm/formula changed → replace code block + update explanation
├── New state transition → update stateDiagram
├── Condition branch changed → rewrite branch description
└── Existing code block is deprecated → remove and replace with current

Development History section
└── Add new row: (date | change | reason) + record in Change Log DB

Constraints & Future Plans section
├── Resolved constraint → remove, move to history
├── New limitation discovered → add
└── Next steps changed → update
```

### Cross-reference rules

- Change affects another system → update that system's doc too
- Project overview system map changed → update it
- New page created → add links from related pages

---

## Step 4: Required Report

Include this in every ticket completion report:

```
## Notion Update
| Page | Section | Action | Content |
|------|---------|--------|---------|
| [SystemName] | Architecture | modified | Updated classDiagram with new class |
| [SystemName] | Data Structure | added | New Enum table for XxxType |
| Data Definitions DB | — | added | XxxEnum registered |
| Change Log DB | — | added | [date] XxxSystem refactored — reason |

## Other System Docs Affected
- [SystemName]: which part changed and why
```

If genuinely nothing was doc-worthy, state it explicitly:
```
## Notion Update
- No doc-worthy changes. Reason: [explanation]
```

Silence is not acceptable. The field must always be present.

---

## New Page Skeleton

When creating a new system page, use this structure:

```markdown
# [SystemName]

## Overview
[One paragraph: what this system does and why it exists]

### System Dependencies
[What this system reads from / writes to / signals it emits or listens to]

### Mermaid System Map
[diagram]

## Design Intent
[Why it was designed this way. Academic references if applicable.]

## Architecture
[classDiagram or component breakdown]

### Signal / Event Flow
[flowchart or sequence diagram]

## Data Structure
| Field | Type | Default | Description |
|-------|------|---------|-------------|

## Core Logic
[Key algorithms, formulas, state machines with code examples]

## Development History
| Date | Change | Reason |
|------|--------|--------|

## Constraints & Future Plans
- Current known limitations
- Intentional omissions
- Next steps
```

---

## Notion Quality Bar

A Notion update passes when ALL of the following are true:

- [ ] Page reflects current code state — not a past or hypothetical state
- [ ] No Q&A format, no conversational tone anywhere
- [ ] No duplicate information added
- [ ] All affected system docs updated (not just the primary one)
- [ ] Data Definitions DB updated if new Enum/constant was added
- [ ] Change Log DB updated if this was a major change
- [ ] Notion Update table filled out in ticket completion report

---

# PART 3 — Prompt Generation Standard

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

### Section 6: Verification & Notion
**How to confirm it works, and what to document.**

**Verification:**
- Gate command to run
- Smoke test (specific in-game action or debug command, completes in <30s)
- What to observe in output/logs to confirm correct behavior
- Edge cases to manually test

**Notion update target:**
- Which system doc page to update
- Which sections change (Overview / Architecture / Data Structure / Core Logic / History)
- Whether Data Definitions DB needs new entries
- Whether Change Log DB needs a new entry

Example:
```
## Verification & Notion

### Verification
- Gate: bash scripts/gate.sh → PASS
- Smoke test: spawn 5 entities, set stress > 0.3, run 100 ticks
  → trauma_records should be non-empty for affected entities
  → SimulationBus should have emitted trauma_recorded N times
- Edge case: entity with stress exactly at TRAUMA_THRESHOLD → no record created
- Edge case: trauma record decays to 0 → removed from array, no crash

### Notion Update
- Page: TraumaSystem (create new if not exists)
  - Overview: system role and design intent
  - Architecture: classDiagram with TraumaRecord + TraumaSystem
  - Data Structure: TraumaRecord fields table
  - Core Logic: decay formula + state transition diagram
  - History: new row (date | initial implementation | van der Kolk model)
- Data Definitions DB: TraumaType enum
- Change Log DB: new entry
- Also update: EntityData page (new trauma_records field)
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
- [ ] Notion section names the exact page and sections to update

If Claude Code would need to ask a follow-up question after reading this prompt → rewrite it.