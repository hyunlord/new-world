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