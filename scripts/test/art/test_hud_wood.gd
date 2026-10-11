extends SceneTree

const Topbar = preload("res://scripts/ui/panels/hud_topbar.gd")
const Buildings = preload("res://scripts/ui/building_renderer.gd")
const Seek = preload("res://scripts/ui/seek_viz_renderer.gd")
var _failures: int = 0

class SnapshotWorld extends Node:
	var settlements: Dictionary = {}
	var agents: Dictionary = {}
	var clock: Dictionary = {}
	func get_tallies() -> Dictionary: return {}
	func get_agent_snapshot() -> Dictionary: return agents
	func get_settlement_snapshot() -> Dictionary: return settlements
	func get_construction_snapshot() -> Dictionary: return {}
	func get_building_snapshot() -> Dictionary: return {}
	func get_resource_snapshot() -> Dictionary: return {}
	func get_time_snapshot() -> Dictionary: return clock

func _initialize() -> void:
	call_deferred("_run")

func _check(condition: bool, message: String) -> void:
	if not condition:
		_failures += 1
		push_error(message)

func _run() -> void:
	_check(Buildings.wood_caption("Firewood", 205, 273) == "Firewood 205/273", "target caption")
	_check(Buildings.wood_caption("Firewood", 205, 0) == "Firewood 205", "legacy caption")
	_check(Buildings.wood_tint(205, 273) == Buildings.WOOD_SHORT_TINT, "only shortage is orange")
	_check(Buildings.wood_tint(273, 273) == Color.WHITE and Buildings.wood_tint(0, 0) == Color.WHITE, "target reached or absent white")
	var target_snapshot: Dictionary = {"formation_xs": PackedInt32Array([1, 9]), "formation_ys": PackedInt32Array([1, 9]), "wood_targets": PackedInt32Array([273, 273])}
	_check(Buildings.wood_target(target_snapshot, Vector2i(1, 1)) == 273, "targets never summed")
	target_snapshot.erase("wood_targets")
	_check(Buildings.wood_target(target_snapshot, Vector2i(1, 1)) == 0, "missing target legacy")
	var topbar: Control = Topbar.new()
	root.add_child(topbar)
	topbar.set_process(false)
	var world: SnapshotWorld = SnapshotWorld.new()
	topbar._world_sim = world
	_check(not topbar._wood_row.visible, "legacy HUD hides wood and its spacing")
	var locale: Node = root.get_node("Locale")
	var language: String = str(locale.get("_current_lang"))
	world.settlements = {"wood_stocks": PackedInt32Array([28, 287, 311, 158])}
	for lang: String in ["en", "ko"]:
		locale.call("set_language", lang)
		topbar._refresh()
		_check(topbar._wood_label.text == ("Firewood 784" if lang == "en" else "장작 784"), "wood sum localized")
		_check(topbar._wood_row.visible, "typed wood shown")
	_check(topbar._wood_row.get_index() == topbar._resource_labels[0].get_index() + 2, "wood directly after food pair gap")
	for invalid: Variant in [null, [2, 3], PackedByteArray([2, 3]), "wood"]:
		world.settlements["wood_stocks"] = invalid
		topbar._refresh()
		_check(not topbar._wood_row.visible, "wrong wood type keeps legacy layout")
	world.settlements["wood_stocks"] = PackedInt32Array()
	topbar._refresh()
	_check(topbar._wood_row.visible and topbar._wood_label.text.ends_with(" 0"), "empty typed stock sums zero")
	world.settlements.erase("wood_stocks")
	topbar._refresh()
	_check(not topbar._wood_row.visible, "removed key hides previous counter")
	var seek: Node2D = Seek.new()
	root.add_child(seek)
	seek.set_process(false)
	seek._world_sim = world
	world.agents = {"xs": PackedInt32Array([1, 2]), "ys": PackedInt32Array([1, 2]), "states": PackedByteArray([1, 1]), "seek_kinds": PackedByteArray([0, 1]), "target_xs": PackedInt32Array([3, 4]), "target_ys": PackedInt32Array([3, 4]), "target_kinds": PackedByteArray([7])}
	seek._process(0.0)
	_check(seek._target_kinds == PackedByteArray([7]), "short optional targets retained per row")
	_check(seek._kind_color(0, 1.0, 7) == seek.COLOR_WOOD, "wood uses target kind despite seek zero")
	_check(seek._kind_color(7, 1.0) == seek.COLOR_GRAY, "seek seven does not imply wood")
	_check(seek._kind_color(1, 1.0) == seek.COLOR_FOOD, "legacy seek food unchanged")
	world.agents["target_kinds"] = [7]
	seek._process(0.0)
	_check(seek._target_kinds.is_empty(), "wrong target type clears optional colors")
	world.agents.erase("target_kinds")
	seek._process(0.0)
	_check(seek._target_kinds.is_empty(), "missing target key legacy")
	world.agents = {"colds": PackedByteArray([1, 0, 1])}
	for lang: String in ["en", "ko"]:
		locale.call("set_language", lang)
		topbar._refresh()
		_check(topbar._cold_label.visible and topbar._cold_label.text == ("Cold 2" if lang == "en" else "추위 2"), "localized cold count")
		_check(topbar._cold_icon.visible, "cold count snowflake is visible")
		_check(topbar._cold_label.get_theme_color("font_color") == Color("a8c0d0"), "cold count uses frost color")
		for severity: int in range(3):
			world.clock = {"winter_severity": severity}
			topbar._refresh()
			var key: String = ["HUD_WINTER_MILD", "HUD_WINTER_NORMAL", "HUD_WINTER_HARSH"][severity]
			_check(topbar._winter_label.visible and topbar._winter_label.text == locale.ltr(key), "winter severity localized")
		world.clock = {"winter_severity": -1}
		topbar._refresh()
		_check(not topbar._winter_label.visible, "severity absent outside booked winter")
	_check(topbar._cold_label.get_index() == topbar._wood_row.get_index() + 1, "cold after wood")
	for value: Variant in [null, [1], PackedByteArray([0])]:
		world.agents["colds"] = value
		topbar._refresh()
		_check(not topbar._cold_label.visible, "no cold count hidden")
		_check(not topbar._cold_icon.visible, "no cold snowflake hidden")
	locale.call("set_language", language)
	seek.free()
	topbar.free()
	world.free()
	print("HUD_WOOD failures=", _failures)
	quit(1 if _failures else 0)
