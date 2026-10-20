# Godot UI-side architecture patterns — reference

> **Origin and changes.** Adapted from `.agents/skills/godot/references/architecture-patterns.md` (generic Godot
> skill). Kept: the three principles (small components, signals, data separated from logic) and the state-machine
> pattern, rewritten for UI modes. **Dropped:** Pattern 1 (3D interaction components), Pattern 2 (health/mana
> attributes), Pattern 3 (spell effects in `.tres`) and Pattern 4 (inventory) — each keeps gameplay state in
> GDScript, which WorldSim forbids (simulation state lives in Rust and GDScript only reads it through
> `SimReads`); Pattern 3 also drew effect magnitudes with `randf_range` and its `.tres` example used an enum value
> (12) outside its 8-entry enum. The generic patterns stay useful for presentation code only.

## Principles (presentation side)

1. **Small components.** One renderer or panel per concern, each a `Node2D`/`Control` child of `Main` (see the
   node list in `scenes/main.tscn`): `TerrainRenderer`, `BuildingRenderer`, `AgentRenderer`, `HudTopbar`, …
   Small files are easier to review and to check with the strict check.
2. **Signals for UI events.** UI-originated events (selection, panel open/close, camera) are Godot signals.
   Simulation events never originate in GDScript; they arrive through bridge snapshots (and `world_reset`).
3. **Data apart from logic.** Visual data (theme, styleboxes, fonts) lives in `.tres`
   (`assets/ui/worldsim_theme.tres`); text lives in localization files (`localization/<lang>/<category>.json`
   compiled to `localization/compiled/*.json`); game content and rules live in Rust.

```
Main (Node2D)
├─ WorldSim (WorldSimNode, Rust)        ← owns all simulation state
├─ SimReads (Node, sim_reads.gd)        ← one cached read layer per frame
├─ TerrainRenderer / BuildingRenderer / AgentRenderer … (Node2D)
├─ Camera2D
└─ UI (CanvasLayer)
   ├─ HudTopbar (Control)
   ├─ AgentInspectorPanel (Control)
   └─ …
```

## Pattern: a mode state machine for UI

Use for UI modes (inspect / follow / overview), not for agent behaviour (that is Rust).

```gdscript
class_name UiModeMachine
extends Node
# Purpose: switches the active UI mode. Bridge methods called: none.

signal mode_changed(new_mode: StringName)

var _modes: Dictionary = {}      # StringName -> UiMode
var _current: UiMode = null

func _ready() -> void:
	for child: Node in get_children():
		if child is UiMode:
			_modes[child.name] = child
			(child as UiMode).finished.connect(change_mode)

func change_mode(mode: StringName) -> void:
	if _current != null:
		_current.exit()
	_current = _modes.get(mode) as UiMode
	if _current != null:
		_current.enter()
	mode_changed.emit(mode)

func _unhandled_input(event: InputEvent) -> void:
	if _current != null:
		_current.handle_input(event)
```

```gdscript
class_name UiMode
extends Node
# Purpose: base class of one UI mode. Bridge methods called: none.

signal finished(next_mode: StringName)

func enter() -> void:
	pass

func exit() -> void:
	pass

func handle_input(_event: InputEvent) -> void:
	pass
```

Notes: the `_event` underscore keeps the strict check's `unused_parameter` warning quiet; class names are global,
so pick names no other script uses.
