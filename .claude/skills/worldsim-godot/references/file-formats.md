# Godot 4.6 text file formats (.gd, .tscn, .tres) — reference

> **Origin and changes.** Adapted from (at tag `v7-final-art`) `.agents/skills/godot/references/file-formats.md` (generic Godot skill).
> Kept: the GDScript / scene / resource syntax rules and the instance-override explanation.
> **Fixed for 4.6:** typed-array rule (Godot writes `Array[T]([...])` only for *typed* properties; untyped
> `Array` properties are written `[...]` and that is valid); `load_steps` is described as a loading-progress hint,
> not a validity rule; `uid` handling now covers the per-script `.uid` sidecar files of Godot 4.4+ and this repo's
> `.gitignore`. **Dropped:** `Color("#ff0000")` as a `.tres` value (the text-resource parser writes and reads
> `Color(r, g, b, a)`); the 3D RPG examples (spells, pickups, doors) were shortened to neutral examples.
> WorldSim note: most UI nodes are built in code (`scripts/ui/panels/*.gd`); `scenes/main.tscn` is a flat list of
> nodes with scripts and is the only scene edited by hand.

## GDScript files (.gd)

Full language. In this repo every `.gd` starts with a comment block (purpose, bridge methods called), uses
static types (`var speed: float = 1.0`), and never holds simulation state.

```gdscript
extends Control
# Purpose: … Bridge methods called: get_tallies (via SimReads).

signal closed

@export var margin_px: int = 12
var _label: Label

func _ready() -> void:
	_label = Label.new()
	add_child(_label)
```

`class_name` registers a **global** name: two scripts with the same `class_name` collide, and removing one breaks
every user (see the Lessons table in `SKILL.md`).

## Scene files (.tscn)

Text serialization of a node tree. Strict format.

```
[gd_scene load_steps=4 format=3 uid="uid://…"]

[ext_resource type="Script" path="res://scripts/core/sim_reads.gd" id="21_sim_reads"]
[ext_resource type="Texture2D" uid="uid://…" path="res://assets/x.png" id="2_tex"]

[sub_resource type="StyleBoxFlat" id="StyleBoxFlat_panel"]
bg_color = Color(0.063, 0.043, 0.012, 0.92)

[node name="Main" type="Node2D"]

[node name="SimReads" type="Node" parent="."]
script = ExtResource("21_sim_reads")

[node name="Label" type="Label" parent="UI/Panel"]
text = "KEY_NAME"
```

### Header
- `format=3` for every Godot 4 file.
- `load_steps` (count of ext + sub resources) is a loading-progress hint. A wrong value does not stop the scene
  loading, and newer editors may stop writing it — but **V7 static tests pinned the `load_steps` and
  `ext_resource` counts of `main.tscn`** (`v7-final-art:docs/art/production7/DECISIONS.md` L13). V8-0 deleted
  those source-text tests; after an edit, run the import, the strict check and the smoke instead.
- `uid="uid://…"`: the scene's own UID. Do not invent or change it by hand; the editor/import owns it.

### ext_resource
- Declare before use; reference with `ExtResource("id")` — never `preload()` (that is GDScript).
- `id` is any unique string (`"21_sim_reads"`); keep existing ids when editing.
- An optional `uid="uid://…"` attribute points at the target's UID. When the UID is stale (script moved, `.uid`
  regenerated) Godot falls back to `path` with a warning; fix the reference rather than ignore the warning.

### sub_resource
Inline resources, declared before use, referenced with `SubResource("id")`.

### Node hierarchy
- Root node: no `parent`.
- Direct children: `parent="."`; deeper: `parent="UI/Panel"` (path from the root, without the root's name).
- Order of `[node]` sections = child order = draw/process order among siblings. In `main.tscn` that order
  matters: `SimReads` sits right after `WorldSim`; renderers are ordered for drawing.

### Instanced scenes and property overrides
`[node name="X" parent="." instance=ExtResource("5_scene")]` instantiates another scene **with that scene's
default values**. Properties set directly under that section override the instance root. Overriding a property
of a *child* of the instance needs a separate section naming the child with `parent="X"` (the editor writes these
sections and may add `index="N"`); a forgotten override silently keeps the default (often `null`).
Prefer making such changes in the editor or in code; WorldSim builds most UI in code, so this is rare here.

### Safe vs risky hand edits
- Safe: add an `ext_resource`; add a new node section; change a simple property value.
- Risky: editing overrides of instanced children; moving nodes between parents; reordering sections (changes
  sibling order).
- Never: change UIDs by hand; delete an `ext_resource` that is still referenced.

## Resource files (.tres)

Serialized data. **No GDScript syntax.**

```
[gd_resource type="Theme" load_steps=5 format=3]

[ext_resource type="FontFile" path="res://assets/fonts/galmuri/Galmuri11.ttf" id="1_body"]

[sub_resource type="StyleBoxFlat" id="StyleBoxFlat_panel"]
bg_color = Color(0.063, 0.043, 0.012, 0.92)
content_margin_left = 12.0

[resource]
default_font = ExtResource("1_body")
default_font_size = 24
Label/font_sizes/font_size = 24
```

(The example is `assets/ui/worldsim_theme.tres`.) Rules:
1. No `var`, `const`, `func`, `preload()` — only `key = value` lines.
2. External references through declared `ExtResource("id")`; nested ones through `SubResource("id")`.
3. Arrays follow the property's declared type: a typed property (`Array[int]`, `Array[MyRes]`) is written
   `Array[int]([1, 2, 3])` or `Array[ExtResource("2_effect")]([SubResource("E1")])`; an untyped `Array` property
   is written `[1, 2, 3]`. A mismatch fails to assign the property.
4. Value syntax: `42`, `3.14`, `true`, `"text"`, `Vector2(10, 20)`, `Vector2i(1, 2)`, `Color(1, 0, 0, 1)`,
   `PackedInt32Array(1, 2, 3)`, `PackedStringArray("a", "b")`.

## Sidecar files Godot writes

| file | what | in git here? |
|---|---|---|
| `*.gd.uid`, `*.gdshader.uid`, `sim_bridge.gdextension.uid` | the resource UID of a script/shader (Godot 4.4+) | **yes** since M1 (`.gitignore` no longer ignores `*.uid`; gate step 20 checks every Godot source has one) |
| `*.import` next to an asset | import settings + remap into `.godot/imported/` | **no** (`*.import` ignored), except the three Galmuri `*.ttf.import` files, which are tracked because they carry the pixel-font settings (antialiasing 0, hinting 0, subpixel positioning 0) |
| `.godot/` | class cache, `extension_list.cfg`, imported data | **no**; created by the first import |

Since M1 the `.uid` files are committed, so a script's UID is the same in every checkout: `git mv` the `.uid`
with the script, commit both, and re-import (procedure in `SKILL.md`). Before M1 they were ignored and
checkout-local, which is how stale UIDs entered scenes.

## Common mistakes

| mistake | correct form |
|---|---|
| `script = preload("res://x.gd")` in .tscn/.tres | declare `[ext_resource type="Script" path="res://x.gd" id="1"]`, then `script = ExtResource("1")` |
| `var speed = 5.0` in .tres | `speed = 5.0` |
| `ExtResource("1_player")` without a declaration | add the `[ext_resource … id="1_player"]` line first |
| `parent="NonExistentNode"` | the parent path must name an existing node section |
| instanced scene whose child needs a value, no override | add the child override section, or set it in code |
| typed property written as untyped array (or the reverse) | match the property's declared type |
