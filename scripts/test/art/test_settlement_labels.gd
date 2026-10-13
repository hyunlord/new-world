extends SceneTree
const Selection = preload("res://scripts/core/settlement_selection.gd")
const Labels = preload("res://scripts/ui/settlement_labels.gd")
const Topbar = preload("res://scripts/ui/panels/hud_topbar.gd")
var failures: int = 0
class World extends Node:
	var ids: PackedInt64Array = PackedInt64Array([7])
	func get_settlement_snapshot() -> Dictionary: return {"ids": ids}
	func get_agent_snapshot() -> Dictionary: return {}
	func get_construction_snapshot() -> Dictionary: return {}
	func get_building_snapshot() -> Dictionary: return {}
	func get_time_snapshot() -> Dictionary: return {}
	func get_resource_snapshot() -> Dictionary: return {}
	func get_tallies() -> Dictionary: return {}

func check(value: bool, message: String) -> void:
	if not value:
		failures += 1
		push_error(message)
func _initialize() -> void:
	call_deferred("run")
func run() -> void:
	Selection.reset()
	var locale: Node = root.get_node("Locale")
	var settlements: Dictionary = {"settlement_ids": PackedInt32Array([7, 42]), "formation_xs": PackedInt32Array([4, 20]), "formation_ys": PackedInt32Array([5, 21])}
	var society: Dictionary = {"settlement_ids": PackedInt32Array([7, 42]), "parent_ids": PackedInt32Array([-1, 7]), "member_counts": PackedInt32Array([10, 4]), "home_agent_ids": PackedInt64Array([101, 102, 201]), "home_settlement_ids": PackedInt32Array([7, 7, 42]), "founder_agent_ids": PackedInt64Array([102]), "fission_daughters": PackedInt32Array([42]), "fission_parents": PackedInt32Array([7]), "fission_site_xs": PackedInt32Array([19]), "fission_site_ys": PackedInt32Array([19])}
	Selection.observe(society, settlements)
	check(Selection.number_for(7) == 1 and Selection.number_for(42) == 2, "sorted stable UI numbers")
	check(Selection.formation_for(42) == Vector2i(20, 21), "actual formation wins over intended fission site")
	check(Selection.displayed_members(7) == 9, "walking founder subtracted from home roster")
	check(Selection.color_for(7) == Color("E0B060") and Selection.color_for(42) == Color("7FB0D8"), "number palette")
	for language: String in ["en", "ko"]:
		locale.call("set_language", language)
		check(Selection.name_for(42, locale) == str(locale.call("ltr", "SETTLEMENT_LABEL_DAUGHTER_FMT")) % [2, 1], "localized daughter parent numbers")
	var layer := Labels.new()
	root.add_child(layer)
	layer.set_process(false)
	layer._update_labels()
	check(layer.capture_metadata()["labels"].size() == 2, "one label per alive settlement")
	for row: Dictionary in layer.capture_metadata()["labels"]:
		check(int(row["screen_font_px"]) == 10, "screen-size label font")
	Selection.observe({"settlement_ids": PackedInt32Array([42]), "parent_ids": PackedInt32Array([7]), "member_counts": PackedInt32Array([4])}, {"settlement_ids": PackedInt32Array([42]), "formation_xs": PackedInt32Array([20]), "formation_ys": PackedInt32Array([21])})
	check(Selection.number_for(42) == 2 and Selection.number_for(7) == 1, "dissolution preserves both numbers")
	Selection.observe({"settlement_ids": PackedInt32Array([42, 50])})
	check(Selection.number_for(50) == 3, "later settlement does not reuse dissolved number")
	var world := World.new()
	var topbar := Topbar.new()
	root.add_child(topbar)
	topbar.set_process(false)
	topbar._world_sim = world
	topbar._refresh()
	check(not topbar._settlements_label.visible, "actual topbar one village hidden")
	world.ids = PackedInt64Array([7, 42])
	topbar._refresh()
	check(topbar._settlements_label.visible and topbar._settlements_label.text == str(locale.call("ltr", "HUD_SETTLEMENTS_FMT")) % 2, "actual topbar two villages count localized")
	topbar.free()
	world.free()
	Selection.observe(society, settlements)
	layer._update_labels()
	var clicked: Array = layer.capture_metadata()["labels"][1]["screen_rect"]
	check(layer.select_at(Vector2(clicked[0] + clicked[2] / 2.0, clicked[1] + clicked[3] / 2.0)), "actual label rectangle accepts click")
	check(Selection.selected_id == 42, "daughter label selects daughter ID")
	var escape := InputEventKey.new()
	escape.pressed = true
	escape.keycode = KEY_ESCAPE
	layer._input(escape)
	check(Selection.selected_id == -1, "Escape clears settlement")
	Selection.select(42)
	var ground := InputEventMouseButton.new()
	ground.pressed = true
	ground.button_index = MOUSE_BUTTON_LEFT
	ground.position = Vector2(-10000, -10000)
	layer._unhandled_input(ground)
	check(Selection.selected_id == -1, "empty ground clears settlement")
	Selection.reset()
	Selection.observe({"settlement_ids": PackedInt32Array([50])})
	check(Selection.number_for(50) == 1 and Selection.number_for(7) == 0, "world reset clears historical numbering")
	layer.free()
	Selection.reset()
	print("SETTLEMENT_LABELS failures=", failures)
	quit(1 if failures else 0)
