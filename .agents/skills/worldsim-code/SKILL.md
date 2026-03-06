---
name: worldsim-code
description: |
  Required for ALL WorldSim work (Rust AND GDScript).
  Part 1: Localization rules — enforced on every GDScript UI change.
  Part 2: Rust coding standards — enforced on every Rust change.
  Part 3: Prompt generation standard — 6-section structure for Codex prompts.
  Part 4: Prompt engineering lessons — failure patterns and solutions.
  Read all parts. No exceptions.
---

# WorldSim Code Skill

---

# PART 1 — Localization (GDScript UI)

## Core Premise

WorldSim does NOT use `TranslationServer` or `tr()`.
All player-facing text must go through the custom Autoload `Locale` (`scripts/core/locale.gd`).

Localization is a GDScript-side concern. Rust simulation code does NOT generate user-visible text — it uses locale keys that GDScript resolves.

---

## Locale API Reference

| API | When to use | Example |
|-----|-------------|---------|
| `Locale.ltr("KEY")` | Simple text lookup | `Locale.ltr("UI_POPULATION")` |
| `Locale.trf1("KEY", "param", val)` | Text with 1 placeholder | `Locale.trf1("UI_POP_FMT", "n", pop)` |
| `Locale.trf2("KEY", "p1", v1, "p2", v2)` | 2 placeholders | `Locale.trf2("UI_POS_FMT", "x", x, "y", y)` |
| `Locale.trf3(...)` | 3 placeholders | See codebase |
| `Locale.tr_id("PREFIX", id)` | PREFIX_ID pattern | `Locale.tr_id("JOB", entity.job)` → `JOB_GATHERER` |
| `Locale.get_month_name(n)` | Month names 1–12 | `Locale.get_month_name(3)` |
| `Locale.set_locale("en")` | Runtime language switch | Settings screen only |

**NEVER use:** `tr()`, `TranslationServer.translate()`

---

## Localization Rules

### Rule 1: No hardcoded strings — ever
```gdscript
# ❌ FORBIDDEN
label.text = "Population"
_add_notification("Population: %d!" % m, color)
_make_label("WorldSim Controls", "help_title")

# ✅ CORRECT
label.text = Locale.ltr("UI_POPULATION")
_add_notification(Locale.trf1("UI_NOTIF_POP_FMT", "n", m), color, NotifCategory.POPULATION)
_make_label(Locale.ltr("UI_HELP_TITLE"), "help_title")
```

### Rule 2: No .contains() on English text for logic
```gdscript
# ❌ FORBIDDEN — breaks in Korean locale
if text.contains("Population") or text.contains("born"):
    bg_color = green

# ✅ CORRECT — use category enum or locale-key matching
match category:
    NotifCategory.POPULATION: bg_color = green
    NotifCategory.DEATH: bg_color = red
```

### Rule 3: No capitalize() as user-visible fallback
```gdscript
# ❌ FORBIDDEN — shows English in Korean locale
if localized == key:
    label = raw_id.capitalize()

# ✅ CORRECT — ensure all values have locale keys
label = Locale.tr_id("JOB", job)  # JOB_GATHERER → "채집꾼" (ko) / "Gatherer" (en)
```

### Rule 4: Both en/ and ko/ for every new key
When adding keys: update `localization/en/` AND `localization/ko/` simultaneously.

### Rule 5: Key naming convention
```
UI_*        → ui.json
JOB_* STATUS_* ACTION_* → game.json
TRAIT_*     → traits.json
EMOTION_*   → emotions.json
EVENT_*     → events.json
DEATH_*     → deaths.json
BUILDING_*  → buildings.json
```

---

## Localization Verification

```bash
# Hardcoded string scan
grep -rn '\.text\s*=\s*"[A-Z]' scripts/ui/ | grep -v 'Locale\.' | grep -v 'debug_cheat'

# _make_label hardcoded scan
grep -rn '_make_label.*"[A-Za-z]' scripts/ui/ | grep -v 'Locale' | grep -v 'debug_cheat'

# .contains() English scan
grep -rn '\.contains("' scripts/ui/ | grep -v 'Locale'

# "Nameless" hardcoded scan
grep -rn '"Nameless"' scripts/ | grep -v 'Locale'

# en/ko key symmetry
python3 -c "
import json, glob
en = set(); ko = set()
for f in glob.glob('localization/en/*.json'):
    en.update(json.load(open(f)).get('strings', json.load(open(f))).keys())
for f in glob.glob('localization/ko/*.json'):
    ko.update(json.load(open(f)).get('strings', json.load(open(f))).keys())
diff = en.symmetric_difference(ko)
print(f'PASS: {len(en)} keys symmetric' if not diff else f'FAIL: {len(diff)} asymmetric')
"
```

Any result in the first four scans → **VIOLATION. Fix before reporting done.**

---

---

# PART 2 — Rust Coding Standards

## Core Premise

ALL simulation logic is Rust. GDScript is UI/rendering only.

---

## Rust Rules

### Rule 1: f64 everywhere for simulation math
```rust
// ❌ WRONG
let decay: f32 = 0.01;

// ✅ CORRECT
let decay: f64 = config::EMOTION_DECAY_RATE;
```

### Rule 2: No unwrap() in production
```rust
// ❌ WRONG
let val = map.get("key").unwrap();

// ✅ CORRECT
let val = map.get("key").unwrap_or(&default);
let val = map.get("key").ok_or(MyError::MissingKey)?;
```

### Rule 3: Enums over strings in hot paths
```rust
// ❌ WRONG
if job == "gatherer" { ... }

// ✅ CORRECT
if job == Job::Gatherer { ... }
```

### Rule 4: Constants in config, not inline
```rust
// ❌ WRONG
let threshold = 0.8;

// ✅ CORRECT
let threshold = config::STRESS_TRAUMA_THRESHOLD;
```

### Rule 5: Doc comments on all pub items
```rust
/// Calculates Gompertz-Makeham mortality hazard rate.
/// Returns probability of death this tick.
pub fn mortality_hazard(age: f64) -> f64 { ... }
```

### Rule 6: Tests in every file
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mortality_increases_with_age() {
        assert!(mortality_hazard(70.0) > mortality_hazard(30.0));
    }
}
```

### Rule 7: No Godot types outside sim-bridge
```rust
// ❌ WRONG (in sim-core or sim-systems)
use godot::prelude::*;
fn process(dict: Dictionary) { ... }

// ✅ CORRECT (only in sim-bridge)
// sim-core/sim-systems use only Rust-native types
```

---

## Rust Verification

```bash
# Build
cd rust && cargo build --workspace

# Test
cd rust && cargo test --workspace

# Lint
cd rust && cargo clippy --workspace -- -D warnings

# Headless sim test
cd rust && cargo run -p sim-test
```

---

---

# PART 3 — Prompt Generation Standard

## Required Structure

Every Codex prompt MUST contain all 6 sections:

### Section 1: Implementation Intent
**Why does this exist? Why this approach?**
- Problem being solved, academic reference if applicable, tradeoffs

### Section 2: What to Build
**Exactly what gets created or changed.**
- File paths (Rust crate + module, or GDScript path), structs/classes, fields, types, defaults
- Events to emit, config constants to add, locale keys to add
- Explicit scope boundary

### Section 3: How to Implement
**Step-by-step logic with enough detail for zero follow-up.**
- Tick priority/interval (for systems), exact formulas, state transitions, code snippets
- For Rust: which crate, which module, query patterns, event emissions
- For GDScript: which SimBridge method to call, which signals to connect

### Section 4: Dispatch Plan
**How to split into tickets.**
- Table: Ticket | File/Concern | 🟢 DISPATCH or 🔴 DIRECT | Depends on
- Dispatch ratio ≥60%
- DIRECT only for: shared interface changes, integration wiring <50 lines

### Section 5: Localization Checklist
**Every new text key in both languages.**
- Table: Key | JSON file | en value | ko value
- "No new localization keys." if none

### Section 6: Verification & Notion
**How to confirm it works.**
- Gate command, smoke test, expected output
- Notion page to update

---

---

# PART 4 — Prompt Engineering Lessons

> Real failures and solutions from development. See original for Lessons 1-6.
> Key additions for Rust-first workflow:

## Lesson 7: Rust tickets need crate + module specification

**현상:** Codex가 Rust 코드를 잘못된 crate에 배치.
**해결:** 프롬프트에 반드시 `crate: sim-systems, module: runtime/psychology.rs` 명시.

## Lesson 8: SimBridge getter 추가는 별도 티켓

**현상:** Rust 시스템 구현 후 UI에서 데이터를 못 읽음 — SimBridge getter가 없어서.
**해결:** 시스템 구현 티켓 + SimBridge getter 티켓 + UI 연결 티켓을 별도로 분리.

## Lesson 9: cargo test는 Godot 없이 실행 가능

**현상:** Codex가 "Godot가 없어서 테스트 불가"라고 보고.
**해결:** sim-core/sim-systems/sim-engine/sim-data는 Godot 없이 `cargo test` 가능. sim-bridge만 Godot 필요. 프롬프트에 명시.