---
name: worldsim-code
description: |
  Required for ALL WorldSim GDScript work.
  Part 1: Localization rules — enforced on every code change.
  Part 2: Prompt generation standard — 6-section structure for Claude Code prompts.
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