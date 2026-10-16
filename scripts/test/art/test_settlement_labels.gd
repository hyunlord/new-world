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
		check(int(row["screen_font_px"]) == 12, "label uses allowed maximum screen font")
		var actual: Label = layer._labels[int(row["id"])]
		check(actual.get_theme_font_size("font_size") == 12, "actual label font matches metadata")
		var rectangle: Array = row["screen_rect"]
		check(is_equal_approx(float(rectangle[2]), actual.size.x + 10.0) and is_equal_approx(float(rectangle[3]), actual.size.y + 6.0), "click rectangle contains padded backplate")
	for zoom: float in [0.5, 1.0, 2.0, 4.0, 8.0]:
		layer.scale = Vector2.ONE * zoom
		layer._update_labels()
		var label: Label = layer._labels[7]
		check(is_equal_approx(label.get_global_transform_with_canvas().get_scale().x, 1.0), "font and padding retain screen size at every zoom")
	layer.scale = Vector2.ONE
	layer._update_labels()
	var objects := Node2D.new()
	objects.name = "WorldObjects"
	root.add_child(objects)
	var obstacle := Sprite2D.new()
	var texture := Image.create(40, 40, false, Image.FORMAT_RGBA8)
	texture.fill(Color.WHITE)
	obstacle.texture = ImageTexture.create_from_image(texture)
	obstacle.position = Vector2(60, 60)
	obstacle.set_meta("art_kind", "building")
	objects.add_child(obstacle)
	var placed: Rect2 = layer._clear_sprite_bounds(Rect2(45, 45, 60, 24))
	check(placed.end.y <= 40.0 - Labels.LABEL_SPRITE_GAP_PX, "placement clears actual roof sprite with screen gap")
	check(placed.position.x == 45.0, "collision avoidance retains formation horizontal anchor")
	obstacle.position = Vector2(60, 20)
	var blocked: Rect2 = layer._clear_sprite_bounds(Rect2(45, -20, 60, 24))
	check(blocked.position.y >= 6.0, "stacked roof near viewport top cannot push label offscreen")
	obstacle.set_meta("art_kind", "agent")
	obstacle.position = Vector2(60, 60)
	var walking: Rect2 = layer._clear_sprite_bounds(Rect2(45, 45, 60, 24))
	check(walking.position == Vector2(45, 45), "moving agents never move label geometry")
	var ui := Node.new()
	ui.name = "UI"
	root.add_child(ui)
	var hud := Control.new()
	hud.name = "HudTopbar"
	ui.add_child(hud)
	var hud_panel := PanelContainer.new()
	hud_panel.position = Vector2(12, 12)
	hud_panel.size = Vector2(300, 100)
	hud.add_child(hud_panel)
	obstacle.set_meta("art_kind", "building")
	obstacle.position = Vector2(60, 120)
	var below_hud: Rect2 = layer._clear_sprite_bounds(Rect2(45, 80, 60, 24))
	check(below_hud.position.y >= 118.0, "blocked roof retains bounded label below actual HUD bottom plus gap")
	ui.free()
	objects.free()
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
	check(not topbar._settlements_label.is_visible_in_tree() and not topbar._settlements_row.is_visible_in_tree(), "actual topbar one village label and icon hidden")
	check(topbar._clock_label.is_visible_in_tree() and topbar._agents_label.is_visible_in_tree() and topbar._sites_label.is_visible_in_tree(), "one village keeps clock population and construction visible")
	world.ids = PackedInt64Array([7, 42])
	topbar._refresh()
	check(topbar._settlements_label.visible and topbar._settlements_row.is_visible_in_tree() and topbar._settlements_label.text == str(locale.call("ltr", "HUD_SETTLEMENTS_FMT")) % 2, "actual topbar two villages count localized")
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
