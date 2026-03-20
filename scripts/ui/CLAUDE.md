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

---

## GDScript UI Patterns [MANDATORY -- Read before any UI work]

These rules were learned from repeated failures. Every rule prevented at least one multi-hour debugging session.

### Layout: Container-Based, Never Manual Offsets

Use VBoxContainer, HBoxContainer, MarginContainer for all layout.
Manual pixel offsets break on different resolutions and UI scales.
If two elements overlap, the fix is NEVER adjusting offsets -- it's restructuring the container hierarchy.

### Panel Lifecycle: `_ensure_ui()` Pattern

`_build_ui()` in `_ready()` alone is unreliable -- `set_data()` may be called before `_ready()`.
Call `_ensure_ui()` from ALL THREE entry points: `set_XXX_id()`, `_ready()`, `_process()`.

### Panel Lifecycle: Force-Tree Guard

Panels added via `add_child()` in init may not be in the scene tree when `open_XXX()` is called.
Always check: `if not _panel.is_inside_tree(): _parent.add_child(_panel)`

### Panel Lifecycle: Data Before Visibility

Set data FIRST, then make visible. Showing an empty panel gives it size (0,0) that never recovers.

### Refresh: Cache Check Before queue_free

Check if data count changed BEFORE calling `queue_free()` on children.
Destroying then returning early = empty container = 3-second flicker cycle.

### Signals: Prevent Duplicate Connections

Anonymous lambdas in `connect()` accumulate on every refresh. Use a `Dictionary` keyed by `get_instance_id()` to track and disconnect old callables before reconnecting.

### Signals: No Recursive Emit

The function that implements an action must NOT emit the signal that triggers it. `follow_entity()` must not emit `follow_entity_requested`.

### Camera: Probe Mode Bypass

`_probe_observation_mode = true` skips `match current_state:`. Any state (FOLLOW) that should work in probe mode needs explicit handling in the probe block.

### Rendering: Minimum Alpha Values

`alpha = 0.06` is invisible. Minimums: fill >= 0.15, border >= 0.40, text >= 0.50.

### Data Updates: _process vs Event-Driven

Data that changes every tick must be updated in `_process`, not only on events. Label text set once on zoom change shows stale data forever.

### BBCode: Deprecated

All new UI uses Godot native nodes (Label, ProgressBar, PanelContainer). RichTextLabel with BBCode is forbidden for new UI.