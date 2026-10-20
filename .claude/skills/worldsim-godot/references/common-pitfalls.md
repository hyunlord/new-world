# Godot 4.6 general pitfalls — reference

> **Origin and changes.** Adapted from `.agents/skills/godot/references/common-pitfalls.md` (generic Godot skill).
> Kept: `@onready`/`_ready` order, node paths, resource sharing, input placement, static typing, null checks,
> instancing, tweens. **Fixed for 4.6:** "connecting a signal then emitting at once may miss the callback" is
> wrong — `connect` takes effect immediately and `emit` calls connected callables synchronously; "connections
> leak when a node is freed" is wrong for object-method callables — Godot removes a connection when either object
> is freed; "modifying a shared resource modifies the .tres file" — it changes the shared in-memory instance, and
> only a save writes it; "set position before `add_child` is wrong" — `position` is local and fine before
> `add_child`, only `global_*` needs the node inside the tree; `await tween.finished` never resumes if the tween
> is killed. **Dropped (out of scope: WorldSim is a 2D front end with no physics bodies):** CharacterBody3D
> movement, 3D basis/forward vectors, mouse capture. WorldSim-specific pitfalls are the Lessons table in `SKILL.md`.

## `_ready()` and `@onready` order

- `_ready()` runs **bottom-up**: children finish `_ready()` before their parent's `_ready()` starts. A parent can
  use its children in `_ready()`; a child must not assume its parent is ready.
- `@onready var x = …` is evaluated just before that node's `_ready()`. A value that another node sets up later
  (in its own `_ready()` or a deferred call) is not there yet: read it lazily or after the other node signals.
- To run something after the whole tree is ready: `call_deferred("_late_init")` or `await get_tree().process_frame`.

```gdscript
func _ready() -> void:
	for child: Node in get_children():
		if child.has_method("configure"):
			child.configure(_config)   # children are ready here
	call_deferred("_late_init")        # after every _ready() of this frame
```

## Node references

- Long relative paths (`$"../../UI/HealthBar"`) break when the hierarchy changes. Prefer an exported `NodePath`,
  a group (`get_tree().get_first_node_in_group("x")`), a signal, or — in WorldSim — the fixed paths of
  `scenes/main.tscn` resolved once in `_ready()` (`get_node_or_null("/root/Main/WorldSim")`) with a null check.
- `get_node_or_null` + a null check beats `$Path` when the node may be absent (fixtures, headless tools).

## Signals

- `sig.connect(callable)` is immediate; `sig.emit()` right after it calls the callable synchronously.
- `emit.call_deferred()` / `call_deferred` postpones to the end of the frame — use it to avoid re-entering an
  object that is still mid-call (WorldSim's bridge emits `world_reset` deferred for this reason).
- A connection is removed automatically when the emitter or the target object is freed. Lambdas that capture
  other objects are not covered by that: disconnect them explicitly in `_exit_tree()`.
- `CONNECT_ONE_SHOT` disconnects after the first call.

## Resources are shared

```gdscript
@export var style: StyleBoxFlat
func _ready() -> void:
	style.bg_color = Color.RED                 # changes every user of this resource in memory
	var mine: StyleBoxFlat = style.duplicate() # per-instance copy
	mine.bg_color = Color.RED
```

`load()`/`preload()` return the cached shared instance. Duplicate before mutating, or set
`resource_local_to_scene`. Containers returned by the bridge are a separate case: see the packed-array aliasing
lesson in `SKILL.md`.

## Input

- Gameplay-style input that must not be missed: handle where it is used. UI input: `_gui_input` on the Control,
  or `_unhandled_input` for world clicks that UI did not consume.
- `Input.parse_input_event()` injects an event but does **not** move the OS cursor: code that polls
  `get_viewport().get_mouse_position()` will not see it (WorldSim lesson, `SKILL.md`).

## Static typing and null checks

```gdscript
var panel: PanelContainer = get_node_or_null("Panel") as PanelContainer
if panel == null:
	return
```

Typed code turns "property does not exist" into a parse error that the strict check catches. Untyped `Variant`
values from the bridge need explicit conversion and a type check (`if v is Dictionary`).

## Instancing

```gdscript
var node: Node2D = scene.instantiate()
node.position = Vector2(10, 20)   # local position: fine before add_child
add_child(node)
node.global_position = target     # global_* only once the node is in the tree
```

An instantiated scene that is never `add_child`-ed is not processed or drawn and leaks unless freed.

## Tweens

```gdscript
var _tween: Tween
func animate(target: Vector2) -> void:
	if _tween != null and _tween.is_valid():
		_tween.kill()              # stop the previous animation first
	_tween = create_tween()
	_tween.tween_property(self, "position", target, 0.12)
	await _tween.finished          # never resumes if the tween is killed
```

In tests, wait on `Tween.finished` (or another signal), never on a wall-clock timer: a fixed 0.15 s wait failed at
low frame rates (WorldSim lesson, `SKILL.md`).

## Quick checklist

- [ ] Null from `@onready`? → the value is created later; read lazily or after a signal.
- [ ] Parent method called from a child's `_ready()`? → defer it.
- [ ] Node path breaks after a move? → resolve once with `get_node_or_null` and check.
- [ ] A resource changed everywhere? → `duplicate()` first.
- [ ] `global_position` error? → set it after `add_child`.
- [ ] Animation stuck? → was the tween killed while something awaited it?
- [ ] Injected input ignored? → read the event's position, not the OS cursor.
