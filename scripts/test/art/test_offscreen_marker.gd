extends SceneTree
const Marker = preload("res://scripts/ui/offscreen_marker.gd")
const Groups = preload("res://scripts/ui/founding_group_renderer.gd")
const Labels = preload("res://scripts/ui/settlement_labels.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
var failures: int = 0
func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)
func _initialize() -> void:
	call_deferred("run")
func run() -> void:
	var viewport := Rect2(0, 0, 960, 540)
	var playable := Rect2(8, 100, 944, 290)
	for direction: Vector2 in [Vector2.RIGHT, Vector2(1, 1).normalized(), Vector2.DOWN, Vector2(-1, 1).normalized(), Vector2.LEFT, Vector2(-1, -1).normalized(), Vector2.UP, Vector2(1, -1).normalized()]:
		var row: Dictionary = Marker.geometry(viewport.get_center() + direction * 2000.0, viewport, playable)
		var p: Array = row["edge_screen_position"]
		var edge := Vector2(p[0], p[1])
		check(bool(row["edge_visible"]) and not bool(row["stake_visible"]), "offscreen exclusive arrow")
		check(playable.grow(-4.5).has_point(edge), "9px glyph inside playable viewport")
		check(absf(viewport.get_center().direction_to(edge).angle_to(direction)) < 0.001, "edge lies on centre target ray")
		check(Marker.hit(row, edge), "actual arrow hit geometry")
	check(not bool(Marker.geometry(Vector2(500, 280), viewport, playable)["edge_visible"]), "onscreen suppresses arrow")
	var camera := Camera2D.new()
	root.add_child(camera)
	Marker.pan(camera, Vector2(901, 703))
	check(camera.position == Vector2(901, 703), "pan reaches destination")
	Selection.reset()
	Selection.observe({"settlement_ids": PackedInt32Array([7]), "home_agent_ids": PackedInt64Array([101]), "home_settlement_ids": PackedInt32Array([7])}, {"settlement_ids": PackedInt32Array([7]), "formation_xs": PackedInt32Array([80]), "formation_ys": PackedInt32Array([80])})
	var groups := Groups.new()
	root.add_child(groups)
	groups.set_process(false)
	groups.refresh_snapshots({"founder_agent_ids": PackedInt64Array([101]), "founder_fission_ids": PackedInt32Array([3]), "founder_leader": PackedByteArray([1]), "founder_site_xs": PackedInt32Array([40]), "founder_site_ys": PackedInt32Array([40])}, {"agent_ids": PackedInt64Array([101]), "xs": PackedInt32Array([10]), "ys": PackedInt32Array([10])})
	check(groups._groups[0]["marker_color"] == Selection.color_for(7), "founding marker parent colour")
	Selection.select(7)
	var labels := Labels.new()
	root.add_child(labels)
	labels.set_process(false)
	labels._update_labels()
	var selected: Dictionary = labels.capture_metadata()["selected_marker"]
	check(selected.get("id") == 7 and not selected.has("displayed_distance_tiles"), "selected hearth marker no distance")
	check(selected.get("marker_color") == Selection.color_for(7).to_html(false), "selected village colour")
	for language: String in ["en", "ko"]:
		root.get_node("Locale").call("set_language", language)
		check(str(root.get_node("Locale").call("ltr", "FOUNDING_SITE_DISTANCE_FMT")) % 24 == ("24 tiles" if language == "en" else "24칸"), "localized distance")
	labels.free()
	groups.free()
	camera.free()
	Selection.reset()
	print("OFFSCREEN_MARKER failures=", failures)
	quit(1 if failures else 0)
