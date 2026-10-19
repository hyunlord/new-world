extends SceneTree
const Names = preload("res://scripts/ui/person_names.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
var failures: int = 0

func _initialize() -> void:
	_run.call_deferred()

func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)

func _run() -> void:
	root.size = Vector2i(1920, 1080)
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	await process_frame
	await process_frame
	main.process_mode = Node.PROCESS_MODE_DISABLED
	var agents: Node = main.get_node("AgentRenderer")
	if not agents.has_method("set_hovered_person_p12"):
		check(false, "person links require a hover presentation route")
		main.free()
		quit(1)
		return
	var sprites: Dictionary = agents.get("_sprites")
	var id: int = int(sprites.keys()[0])
	var sprite: Sprite2D = sprites[id]
	var label := RichTextLabel.new()
	main.get_node("UI").add_child(label)
	Names.prepare_rich_text(label)
	Selection.selected_id = -1
	label.meta_hover_started.emit("person:%d" % id)
	check(int(agents.get("_hovered_person_p12")) == id, "person metadata enters hover")
	var outline: Sprite2D = sprite.get_node_or_null("SettlementOutline") as Sprite2D
	check(outline != null and outline.visible, "hover reuses real settlement outline shader")
	if outline != null:
		check((outline.material as ShaderMaterial).get_shader_parameter("outline_color") == Color("D7BD77"), "hover uses ring color")
	var camera: Camera2D = main.get_node("Camera2D") as Camera2D
	camera.position_smoothing_enabled = false
	camera.zoom = Vector2(4, 4)
	camera.position = sprite.global_position
	camera.force_update_scroll()
	await _capture("on-screen")
	camera.position = sprite.global_position + Vector2(10000, 10000)
	camera.force_update_scroll()
	agents.call("_update_link_hover_p12")
	check(bool((agents.get("_hover_marker_p12") as Dictionary).get("edge_visible", false)), "offscreen hovered person uses existing edge marker")
	await _capture("off-screen")
	label.meta_hover_ended.emit("person:%d" % id)
	check(int(agents.get("_hovered_person_p12")) == -1, "hover ends with link")
	check(outline != null and not outline.visible, "deselected hover outline restores")
	label.meta_hover_started.emit("person:%d" % id)
	label.hide()
	check(int(agents.get("_hovered_person_p12")) == -1, "hidden panel cannot leave a stale hover")
	label.show()
	label.meta_hover_started.emit("person:99999999")
	check(int(agents.get("_hovered_person_p12")) == -1, "dead or missing sprite does not invent a target")
	main.free()
	print("LINK_HOVER_P12 failures=", failures)
	quit(1 if failures else 0)


func _capture(stem: String) -> void:
	var output: String = OS.get_environment("P12_HOVER_OUT")
	if output.is_empty():
		return
	check(DirAccess.make_dir_recursive_absolute(output) == OK, "hover evidence directory")
	await process_frame
	await RenderingServer.frame_post_draw
	check(root.get_texture().get_image().save_png(output.path_join(stem + ".png")) == OK, "save hover image")
