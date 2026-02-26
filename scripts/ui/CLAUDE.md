# scripts/ui/ — CLAUDE.md

> UI layer: rendering, panels, HUD, camera.
> **This layer READS game state. It NEVER WRITES simulation state.**

---

## Golden Rule

```
UI ──reads──→ EntityData, WorldData, SimulationBus signals
UI ──NEVER──→ modifies EntityData, calls system functions, changes GameConfig
```

If you need the UI to trigger a game action (e.g., player spawns entity), emit a **request signal** through SimulationBus. The relevant system handles it.

```gdscript
# ✅ CORRECT: UI requests, system executes
SimulationBus.emit_signal("player_spawn_requested", position)

# ❌ WRONG: UI directly modifies game state
EntityManager.spawn_entity(position)
```

---

## Directory Structure

```
ui/
  panels/
    entity_detail_panel.gd     — Selected entity info
    building_detail_panel.gd   — Selected building info
    chronicle_panel.gd         — Event history
    entity_list_panel.gd       — Entity list/filter
    minimap_panel.gd           — World minimap
    stats_panel.gd             — Population/resource stats
    stats_detail_panel.gd      — Detailed statistics
    pause_menu.gd              — Pause/settings
  renderers/
    world_renderer.gd          — Tile rendering
    entity_renderer.gd         — Entity sprites
    building_renderer.gd       — Building sprites
  hud.gd                       — Top-level HUD container
  camera_controller.gd         — Pan/zoom/follow
  popup_manager.gd             — Toast notifications
  trait_tooltip.gd             — Trait hover tooltips
```

---

## Panel Conventions

### Lifecycle
```gdscript
func _ready() -> void:
    # Connect to SimulationBus signals
    SimulationBus.connect("entity_spawned", _on_entity_spawned)
    SimulationBus.connect("emotion_changed", _on_emotion_changed)

func _on_entity_spawned(entity_id: int) -> void:
    if _showing_entity_id == entity_id:
        _refresh()

func _refresh() -> void:
    # Read from EntityData, update UI elements
    var ed = EntityManager.get_entity(_showing_entity_id)
    if ed == null:
        hide()
        return
    _update_labels(ed)
```

### Data Display Rules

- **7-level descriptors** for 0.0~1.0 values (Dwarf Fortress style):
  ```
  0.00~0.14: Locale.ltr("STAT_EXTREMELY_LOW")
  0.15~0.29: Locale.ltr("STAT_VERY_LOW")
  0.30~0.44: Locale.ltr("STAT_SOMEWHAT_LOW")
  0.45~0.55: (hidden — average, not shown)
  0.56~0.70: Locale.ltr("STAT_SOMEWHAT_HIGH")
  0.71~0.85: Locale.ltr("STAT_VERY_HIGH")
  0.86~1.00: Locale.ltr("STAT_EXTREMELY_HIGH")
  ```

- **Never show raw float values** to the player. Always use descriptors or progress bars.
- **Stat display names**: Use `Locale.ltr("STAT_" + stat_key.to_upper())`, never hardcode.
- **Forbidden**: `"H: 0.72"` — Must be `"정직-겸손: ████████░░ (매우 높음)"`

### Toast/Event Notifications
- Major emotion changes → toast via PopupManager
- Major life events → toast + chronicle entry
- Toast format: `Locale.ltr("TOAST_" + event_type)` with parameter substitution

---

## Localization [STRICTLY ENFORCED]

**Every single user-visible string MUST use `Locale.ltr()`.**

```gdscript
# ❌ WRONG
label.text = "Health: " + str(health)
label.text = "정신 붕괴"
label.text = ed.mental_break_type.to_upper()  # Raw enum name shown to user!

# ✅ CORRECT
label.text = Locale.ltr("UI_HEALTH") + ": " + str(health)
label.text = Locale.ltr("STATUS_MENTAL_BREAK")
label.text = Locale.ltr("MENTAL_BREAK_" + ed.mental_break_type.to_upper())
```

**Common mistakes in UI:**
1. Showing raw enum names (`PANIC`, `RAGE`) instead of localized versions
2. Concatenating translated + untranslated strings
3. Forgetting `ko/` JSON entries when adding new keys
4. Using `tr()` instead of `Locale.ltr()`

---

## Renderer Conventions

### WorldRenderer
- Renders tiles from WorldData (PackedArrays)
- Responds to camera viewport changes
- LOD: only render visible chunks

### EntityRenderer
- Reads position from EntityData
- Visual state from emotions/health (color tinting, animations)
- Does NOT store entity state — reads every frame

### BuildingRenderer
- Reads from BuildingManager
- Construction progress visualization

**Renderers are pure display. They never modify the data they read.**

---

## Camera Controller

- Supports: pan (drag/WASD), zoom (scroll/pinch), follow entity
- Emits `camera_moved` for renderers to update visible area
- Respects world bounds from WorldData

---

## Rust Migration Notes

UI stays 100% GDScript. There is no performance need to migrate UI to Rust.
The UI's job is to read data (which may come from Rust-backed systems) and display it.

If Rust-backed systems return data in a different format (e.g., PackedArray instead of Dictionary), write a thin GDScript adapter in the UI layer.

---

## Do NOT

- Write to EntityData, WorldData, or any simulation state
- Import or reference any system directly
- Call `process_tick()` or any simulation function
- Use `await` for data that should be available synchronously
- Show raw internal identifiers (entity IDs, enum names) to players
- Hardcode any user-visible string
- Put game logic in UI code ("if health < 0.2 then entity dies" → belongs in a system)