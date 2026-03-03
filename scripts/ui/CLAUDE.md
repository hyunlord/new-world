# scripts/ui/ — CLAUDE.md

> UI layer: rendering, panels, HUD, camera.
> **This layer READS game state via SimBridge. It NEVER WRITES simulation state.**
> All simulation logic is in Rust. UI is pure display + input forwarding.

---

## Golden Rule

```
UI ──reads──→ SimBridge.get_*() methods, SimulationBus signals
UI ──sends──→ SimBridge.push_command() for player actions
UI ──NEVER──→ modifies entity data, calls Rust internals, writes to ECS
```

---

## Directory Structure

```
ui/
  panels/
    entity_detail_panel.gd     — Selected entity info (reads SimBridge)
    building_detail_panel.gd   — Selected building info
    chronicle_panel.gd         — Event history
    list_panel.gd              — Entity list/filter
    minimap_panel.gd           — World minimap
    stats_panel.gd             — Population/resource stats
    settlement_detail_panel.gd — Settlement details
    pause_menu.gd              — Pause/settings
    trait_tooltip.gd           — Trait hover tooltips
    settlement_tabs/           — Settlement sub-panels
    world_stats_tabs/          — World stats sub-panels
  renderers/
    world_renderer.gd          — Tile rendering
    entity_renderer.gd         — Entity sprites (reads positions from SimBridge)
    building_renderer.gd       — Building sprites
  map_editor/                  — Map editor UI
  hud.gd                       — Top-level HUD container
  camera_controller.gd         — Pan/zoom/follow
```

---

## Data Access Pattern

### Reading Entity Data
```gdscript
# ✅ CORRECT: Read through SimBridge
var detail: Dictionary = SimBridge.get_entity_detail(entity_id)
var name: String = detail.get("name", "")
var stress: float = detail.get("stress", 0.0)

# ❌ WRONG: Direct entity access (legacy pattern, must not be used)
var entity = EntityManager.get_entity(entity_id)
```

### Player Actions
```gdscript
# ✅ CORRECT: Send command through SimBridge
SimBridge.push_command("spawn_entity", {"x": 10, "y": 20})
SimBridge.push_command("set_speed", {"speed": 2})

# ❌ WRONG: Directly modifying state
EntityManager.spawn_entity(position)
```

### Listening to Events
```gdscript
# ✅ CORRECT: Listen to SimulationBus (receives events relayed from Rust)
SimulationBus.connect("entity_died", _on_entity_died)
SimulationBus.connect("building_constructed", _on_building_constructed)
```

---

## Localization [STRICTLY ENFORCED]

**Every single user-visible string MUST use `Locale.ltr()`.**

```gdscript
# ❌ WRONG
label.text = "Health: " + str(health)
label.text = "정신 붕괴"

# ✅ CORRECT
label.text = Locale.ltr("UI_HEALTH") + ": " + str(health)
label.text = Locale.ltr("STATUS_MENTAL_BREAK")
label.text = Locale.tr_id("JOB", entity_job)
```

**Common mistakes in UI:**
1. Showing raw enum names instead of localized versions
2. Concatenating translated + untranslated strings
3. Forgetting `ko/` JSON entries when adding new keys
4. Using `tr()` instead of `Locale.ltr()`
5. Hardcoded strings in `_make_label()`, `_add_notification()`, `draw_string()`

---

## Renderer Conventions

Renderers read positional/visual data from SimBridge bulk getters:

```gdscript
# Entity positions as PackedFloat64Array
var positions: PackedFloat64Array = SimBridge.get_entity_positions()
# Format: [x0, y0, x1, y1, x2, y2, ...]
```

**Renderers are pure display. They never modify data.**

---

## Do NOT

- Write to any simulation state
- Import or reference simulation systems (they're in Rust)
- Access entity data except through SimBridge
- Put game logic in UI code
- Hardcode any user-visible string
- Use `await` for data that should be available synchronously
- Show raw internal identifiers (entity IDs, enum names) to players