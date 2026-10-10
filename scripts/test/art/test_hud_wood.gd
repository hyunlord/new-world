extends SceneTree

const Topbar = preload("res://scripts/ui/panels/hud_topbar.gd")
const Seek = preload("res://scripts/ui/seek_viz_renderer.gd")
var _failures: int = 0

class SnapshotWorld extends Node:
	var settlements: Dictionary = {}
	var agents: Dictionary = {}
	func get_tallies() -> Dictionary: return {}
	func get_agent_snapshot() -> Dictionary: return agents
	func get_settlement_snapshot() -> Dictionary: return settlements
	func get_construction_snapshot() -> Dictionary: return {}
	func get_building_snapshot() -> Dictionary: return {}
	func get_resource_snapshot() -> Dictionary: return {}
	func get_time_snapshot() -> Dictionary: return {}

func _initialize() -> void:
	call_deferred("_run")

func _check(condition: bool, message: String) -> void:
	if not condition:
		_failures += 1
		push_error(message)

func _run() -> void:
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
	locale.call("set_language", language)
	seek.free()
	topbar.free()
	world.free()
	print("HUD_WOOD failures=", _failures)
	quit(1 if _failures else 0)
