extends SceneTree
const Selection = preload("res://scripts/core/settlement_selection.gd")
const Agents = preload("res://scripts/ui/agent_renderer.gd")
const GodPanel = preload("res://scripts/ui/panels/god_hand_panel.gd")
const Effects = preload("res://scripts/rendering/art/god_hand_effects.gd")
var failures: int = 0
class World extends Node:
	var calls: Array = []
	var rain: PackedInt32Array = PackedInt32Array()
	func get_settlement_snapshot() -> Dictionary:
		return {"settlement_ids": PackedInt32Array([7, 42]), "formation_xs": PackedInt32Array([4, 20]), "formation_ys": PackedInt32Array([5, 21])}
	func get_society_snapshot() -> Dictionary:
		return {"settlement_ids": PackedInt32Array([7, 42]), "parent_ids": PackedInt32Array([-1, 7]), "member_counts": PackedInt32Array([1, 1]), "home_agent_ids": PackedInt64Array([101, 201]), "home_settlement_ids": PackedInt32Array([7, 42])}
	func get_intervention_state() -> Dictionary:
		return {"rain_settlement_ids": rain}
	func apply_intervention(kind: String, id: int, amount: int) -> Dictionary:
		calls.append([kind, id, amount])
		if kind == "rain":
			rain = PackedInt32Array([id])
		return {"ok": true, "tick": 5, "applied": amount}
func _initialize() -> void:
	call_deferred("run")
func check(value: bool, message: String) -> void:
	if not value:
		failures += 1
		push_error(message)
func run() -> void:
	Selection.reset()
	var world := World.new()
	var panel := GodPanel.new()
	root.add_child(panel)
	panel.set_process(false)
	panel.configure(world)
	var image := Image.create(8, 8, false, Image.FORMAT_RGBA8)
	image.fill(Color.WHITE)
	var original := ShaderMaterial.new()
	var a := Sprite2D.new()
	a.texture = ImageTexture.create_from_image(image)
	a.region_enabled = true
	a.region_rect = Rect2(1, 1, 6, 6)
	a.material = original
	var b := Sprite2D.new()
	b.texture = a.texture
	b.region_enabled = true
	b.region_rect = a.region_rect
	Selection.select(42)
	Agents._apply_settlement_outline(a, 101)
	Agents._apply_settlement_outline(b, 201)
	check(not bool(a.get_meta("settlement_outline")) and bool(b.get_meta("settlement_outline")), "only selected roster members outlined")
	check(a.material == original, "outline preserves variant shader")
	check(b.get_node("SettlementOutline").visible, "selected member outline exists")
	panel.cast_kind("rain")
	check(world.calls.back() == ["rain", 42, 13] and world.rain == PackedInt32Array([42]), "selected daughter receives rain")
	var toast: String = panel._toast.text
	Selection.select(7)
	panel.refresh_state()
	check(panel._toast.text == toast, "toast retains actual cast target after selection changes")
	Selection.clear()
	Agents._apply_settlement_outline(b, 201)
	check(not b.get_node("SettlementOutline").visible, "no outline after clearing")
	panel.cast_kind("rain")
	check(world.calls.back() == ["rain", -1, 13], "no selection preserves home command")
	var effects := Effects.new()
	effects.notify_cast("gift_wood", 42)
	Selection.select(7)
	check(effects._notified_target == 42, "gift flash target frozen at cast")
	panel.free()
	world.free()
	effects.free()
	a.free()
	b.free()
	Selection.reset()
	print("SETTLEMENT_SELECT failures=", failures)
	quit(1 if failures else 0)
