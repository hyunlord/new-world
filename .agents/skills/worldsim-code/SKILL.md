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

## Crate Responsibilities (v3.1)

### sim-core/
- ECS components (18+ types: Identity through Faith + Temperament)
- Influence Grid (influence_grid.rs) — 8-12 typed channel grid, wall blocking mask
- Effect Primitives (effect.rs) — 6-type standard
- Causal Log (causal_log.rs) — per-entity ring buffer, CauseRef
- Tile Grid (tile_grid.rs) — building structural data, wall/floor/roof
- Room System (room.rs) — dirty-flagged BFS, room role auto-detection
- Temperament (temperament.rs) — TCI 4-axis (NS/HA/RD/P), latent+expressed, archetype_label()
- Config constants (config.rs)

### sim-data/
- RON loader + validation pipeline
- MaterialDef — properties (hardness/density/melting/rarity/value), tags
- FurnitureDef — furniture type, required materials, influence emission
- ActionDef — action conditions, effects, duration, tool requirements
- RecipeDef — tag+threshold inputs, era gates, outputs
- StructureDef — building blueprints (walls, roof, required furniture)
- WorldRuleset — 5-slot world rules, composition, priority/merge
- TemperamentRules — PRS weight matrix 4x38, bias matrix 4x24, shift rules
- All content in RON files. Zero .rs changes for new content.

### sim-systems/
- RuntimeSystems (53+ registered, Hot/Warm/Cold)
- Material auto-derivation (property → item stat formulas)
- Room influence aggregation (furniture -> room cache)
- Building construction AI (GOAP + blueprint)
- Tag-based recipe resolution
- Temperament derivation (gene -> TCI) and shift (dramatic event -> axis change)
- World Rules application (settings -> compile -> parameter override)

### sim-engine/
- Tick loop, system scheduling by frequency tier
- Double-buffer swap, damping, Sigmoid
- World Rules lifecycle management

### sim-bridge/
- Rust↔Godot FFI (gdext)
- FrameSnapshot, Influence Grid data texture upload
- MultiMesh buffer (Vec<f32> → PackedFloat32Array)
- Oracle text pipeline boundary (player input -> LLM -> response)

### sim-test/
- Headless test binary
- Material auto-derivation tests
- Room detection tests
- Tag recipe resolution tests
- Temperament derivation + shift tests
- World Rules loading + composition tests

## RON File Standards (v3.1)

### Material (sim-data/materials/*.ron)

```ron
MaterialDef(
    name: "copper",
    category: Metal,
    tags: ["metal", "soft_metal"],
    properties: {
        hardness: 3.0,
        density: 8.96,
        melting_point: 1085,
        rarity: 0.6,
        value: 5.0,
    },
)
```

### Recipe (sim-data/recipes/*.ron)

```ron
RecipeDef(
    name: "bronze_sword",
    inputs: [{ tag: "metal", min_hardness: 3.0, amount: 2 }],
    requires: { building_tag: "forge", tech: "metalworking" },
    output: { template: "sword", material_from_input: 0 },
)
```

### Structure (sim-data/structures/*.ron)

```ron
StructureDef(
    name: "forge",
    min_size: (3, 3),
    required_components: [
        Wall(count: 4, tags: ["stone"]),
        Roof(tags: ["thatch"]),
        Furniture("anvil", 1),
        Furniture("hearth", 1),
        Furniture("water_trough", 1),
    ],
    role_recognition: "auto",
    influence: [(channel: "noise", radius: 40, intensity: 0.6)],
)
```

### World Rules (sim-data/world_rules/*.ron)

```ron
WorldRuleset(
    name: "DungeonEconomy",
    priority: 100,
    resource_modifiers: [(target: "surface_foraging", multiplier: 0.1)],
    special_zones: [(kind: "dungeon_node", count: (3, 7))],
    special_resources: [(name: "magic_stone", tags: ["currency"])],
    agent_modifiers: [(system: "essence", effect: "temperament_shift")],
)
```

### Temperament Rules (sim-data/temperament/*.ron)

```ron
TemperamentShiftRule(
    trigger: Event("family_death"),
    conditions: [Temperament(axis: "ha", value: ">0.5")],
    effect: TemperamentShift(axis: "ha", delta: +0.3, paired_axis: "ns", paired_delta: -0.2),
    cascade: true,
    causal_log: "family_death->temperament_shift",
)
```

### Rules
- All content is RON. Never hardcode simulation parameters.
- Material properties auto-derive item stats. Never manually set weapon damage.
- Recipes use tags, never material IDs.
- New material/building/world rule = new `.ron` file only. Zero `.rs` changes.
- Temperament shift rules are data-driven. Never hardcode personality changes.

---

---

# PART 3 — Prompt Generation Standard

## Required Structure

Every Codex prompt MUST contain all 6 sections:

### Section 1: Implementation Intent
**Why does this exist? Why this approach?**
- Problem being solved, academic reference if applicable, tradeoffs
- For architecture-doc sync, state that the change applies to Claude-facing `CLAUDE.md`, Codex-facing `AGENTS.md`, and both checked-in `worldsim-code` skill mirrors.

### Section 2: What to Build
**Exactly what gets created or changed.**
- File paths (Rust crate + module, or GDScript path), structs/classes, fields, types, defaults
- Events to emit, config constants to add, locale keys to add
- Explicit scope boundary
- For doc sync prompts, explicitly scope `CLAUDE.md`, `AGENTS.md`, the `.agents` and `.claude` `worldsim-code` skill mirrors, and `all repo AGENTS.md files whose guidance conflicts with v3.1 architecture`.

### Section 3: How to Implement
**Step-by-step logic with enough detail for zero follow-up.**
- Tick priority/interval (for systems), exact formulas, state transitions, code snippets
- For Rust: which crate, which module, query patterns, event emissions
- For GDScript: which SimBridge method to call, which signals to connect
- For doc sync prompts, add an `AGENTS.md` scan step (`rg --files -g 'AGENTS.md'` plus targeted `rg -n`) and update every scoped `AGENTS.md` that needs v3.1 alignment.

### Section 4: Dispatch Plan
**How to split into tickets.**
- Table: Ticket | File/Concern | 🟢 DISPATCH or 🔴 DIRECT | Depends on
- Dispatch ratio ≥60%
- DIRECT only for: shared interface changes, integration wiring <50 lines
- Shared instruction-doc sync (`CLAUDE.md`, `AGENTS.md`, skill docs) may be `🔴 DIRECT` when the change is pure text coordination.

### Section 5: Localization Checklist
**Every new text key in both languages.**
- Table: Key | JSON file | en value | ko value
- "No new localization keys." if none

### Section 6: Verification & Notion
**How to confirm it works.**
- Gate command, smoke test, expected output
- Notion page to update
- For doc sync prompts, require a stale-guidance scan so no conflicting architecture descriptions remain across `CLAUDE.md`, `AGENTS.md`, and both `SKILL.md` mirrors.

## Documentation Sync Prompt Pattern (v3.1)

When the task is architecture-document synchronization for both Claude and Codex:
- `Implementation Intent` must say the v3.1 decision set is being propagated to `CLAUDE.md`, `AGENTS.md`, and both checked-in `worldsim-code` skill mirrors so Claude and Codex follow the same architecture.
- `What to Build` must list the primary root docs plus the scoped rule: `all repo AGENTS.md files whose guidance conflicts with v3.1 architecture`.
- `How to Implement` must include a repo-wide `AGENTS.md` scan, then update every scoped file that still carries stale boundaries.
- `Dispatch Plan` should include direct tickets for root `AGENTS.md`, root `CLAUDE.md`, both skill mirrors, and scoped `AGENTS.md` cleanup when the work is shared-text maintenance.
- `Verification & Notion` must include grep checks for `14 Day-1`, `World Rules`, `TCI temperament`, and a conflict scan for stale JSON / v3 language.
- Final report templates for these prompts should call out which scoped `AGENTS.md` files were updated.

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
