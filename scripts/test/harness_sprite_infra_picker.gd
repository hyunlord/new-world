## Harness test for sprite-infra G1/G2/G3 — executes the production GDScript
## picker + emoji-fallback functions and emits PASS/FAIL lines on stdout.
##
## Usage:
##   godot --headless --path . --script scripts/test/harness_sprite_infra_picker.gd
##
## Output contract (parsed by rust/crates/sim-test):
##   [G1] PASS (or [G1] FAIL: <reason>)
##   [G2] PASS (or [G2] FAIL: <reason>)
##   [G3] PASS (or [G3] FAIL: <reason>)
##   [SUMMARY] ALL_PASS (emitted iff all three asserted)
##
## Exit code: 0 on ALL_PASS, 1 on any FAIL / load error.

extends SceneTree

var _renderer: Node = null
var _g1_ok: bool = false
var _g2_ok: bool = false
var _g3_ok: bool = false


func _initialize() -> void:
	var renderer_script: Script = load("res://scripts/ui/renderers/building_renderer.gd")
	if renderer_script == null:
		print("[LOAD] FAIL: could not load building_renderer.gd")
		quit(1)
		return
	_renderer = renderer_script.new()
	if _renderer == null:
		print("[LOAD] FAIL: renderer_script.new() returned null")
		quit(1)
		return

	_check_g1()
	_check_g2()
	_check_g3()

	if _g1_ok and _g2_ok and _g3_ok:
		print("[SUMMARY] ALL_PASS")
		quit(0)
	else:
		print("[SUMMARY] FAIL g1=%s g2=%s g3=%s" % [_g1_ok, _g2_ok, _g3_ok])
		quit(1)


## G1 — _pick_variant_for_entity is deterministic and returns [0, 4] for count=5.
func _check_g1() -> void:
	var results: Array[int] = []
	for i in 10:
		results.append(_renderer._pick_variant_for_entity(42, 5))
	var first: int = results[0]
	for v in results:
		if v != first:
			print("[G1] FAIL: non-deterministic results=%s" % [results])
			return
	if first < 0 or first > 4:
		print("[G1] FAIL: value %d not in [0, 4]" % first)
		return
	print("[G1] PASS value=%d samples=%d" % [first, results.size()])
	_g1_ok = true


## G2 — _pick_variant_for_entity distributes across >= 3 unique indices over
## 100 distinct entity_ids (rules out constant-bypass regressions).
func _check_g2() -> void:
	var unique: Dictionary = {}
	for i in 100:
		var v: int = _renderer._pick_variant_for_entity(i, 5)
		if v < 0 or v > 4:
			print("[G2] FAIL: picker returned %d for entity_id=%d (out of [0, 4])" % [v, i])
			return
		unique[v] = true
	if unique.size() < 3:
		print("[G2] FAIL: only %d unique indices over 100 ids: %s" % [unique.size(), unique.keys()])
		return
	print("[G2] PASS unique=%d" % unique.size())
	_g2_ok = true


## G3 — emoji fallback preserved for furniture ids with no sprite on disk.
## totem → 🗿, hearth → 🔥. The renderer function under test is
## _tile_furniture_icon(), which the draw path invokes when _load_furniture_texture()
## returns null (i.e. no sprite present on disk).
func _check_g3() -> void:
	var totem_icon: String = _renderer._tile_furniture_icon("totem")
	var hearth_icon: String = _renderer._tile_furniture_icon("hearth")
	# No sprite folder ships for either id in Feature 1 — the load function must
	# therefore return null so the draw path falls through to the emoji.
	# We also confirm emoji glyphs themselves are the exact spec literals.
	var totem_tex: Texture2D = _renderer._load_furniture_texture("totem", 12345)
	var hearth_tex: Texture2D = _renderer._load_furniture_texture("hearth", 67890)
	if totem_tex != null:
		print("[G3] FAIL: totem sprite unexpectedly loaded — fallback will be skipped")
		return
	if hearth_tex != null:
		print("[G3] FAIL: hearth sprite unexpectedly loaded — fallback will be skipped")
		return
	if totem_icon != "🗿":
		print("[G3] FAIL: totem icon expected 🗿 got %s" % totem_icon)
		return
	if hearth_icon != "🔥":
		print("[G3] FAIL: hearth icon expected 🔥 got %s" % hearth_icon)
		return
	print("[G3] PASS totem=%s hearth=%s" % [totem_icon, hearth_icon])
	_g3_ok = true
