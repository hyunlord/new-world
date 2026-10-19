extends SceneTree
## Synthetic boundary checks only; this does not establish real mouse-input acceptance.
var failures: int = 0

func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)

func _initialize() -> void:
	run.call_deferred()

func run() -> void:
	var main: Node = load("res://scenes/main.tscn").instantiate()
	var world: Node = main.get_node("WorldSim")
	world.set_sim_speed(0.0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(main)
	main.set_process(false)
	SimReads.mutate(world, "reset_world_stress", [15100, 200])
	var agents: Node = main.get_node("AgentRenderer")
	agents.set_process(false)
	var reads: Node = main.get_node("SimReads")
	var snap: Dictionary = SimReads.read(world, "get_agent_snapshot").duplicate(true)
	for key: Variant in snap:
		if typeof(snap[key]) in [TYPE_PACKED_BYTE_ARRAY, TYPE_PACKED_INT32_ARRAY, TYPE_PACKED_INT64_ARRAY, TYPE_PACKED_FLOAT32_ARRAY, TYPE_PACKED_FLOAT64_ARRAY, TYPE_ARRAY]:
			snap[key] = snap[key].slice(0, 1)
	reads.get("_cache")["get_agent_snapshot"] = snap
	agents._process(0.0)
	var agent_id: int = int(snap["agent_ids"][0])
	var sprite: Sprite2D = main.get_node(NodePath("WorldObjects/Agent_%d" % agent_id)) as Sprite2D
	var tooltip: Control = main.get_node("UI/ActivityTooltipP13") as Control
	tooltip.set_process(false)
	var panel: PanelContainer = tooltip.get("_panel") as PanelContainer
	var label: Label = tooltip.get("_label") as Label
	var inspector: Node = main.get_node("UI/AgentInspectorPanel")
	var selection: int = int(inspector.get_selected_entity_bits())
	var pixels: Image = sprite.texture.get_image()
	if sprite.region_enabled:
		pixels = pixels.get_region(Rect2i(sprite.region_rect))
	var opaque: Rect2 = Rect2(pixels.get_used_rect())
	if sprite.flip_h:
		opaque.position.x = sprite.get_rect().size.x - opaque.end.x
	var local_center: Vector2 = sprite.get_rect().position + opaque.get_center()
	var center: Vector2 = root.get_visible_rect().size * 0.5
	sprite.position += sprite.get_canvas_transform().affine_inverse() * center - sprite.to_global(local_center)
	var screen: Vector2 = sprite.get_global_transform_with_canvas() * local_center
	sprite.set_meta("fallback_kind", "activity_icon")
	sprite.set_meta("activity_kind", 0)
	check(int(agents.drawn_entity_at(sprite.to_global(local_center))) == int(snap["ids"][0]), "fixture uses actual drawn hit geometry")
	tooltip._update_hover(0.0, screen, false)
	tooltip._update_hover(0.39, screen, false)
	check(not panel.visible, "hidden before continuous 0.4 seconds")
	tooltip._update_hover(0.011, screen, false)
	check(panel.visible, "shown after continuous 0.4 seconds")
	var locale: Node = root.get_node("Locale")
	var old_language: String = str(locale.get("_current_lang"))
	for language: String in ["en", "ko"]:
		locale.set_language(language)
		check(not panel.visible, "locale clears prior tooltip")
		tooltip._update_hover(0.0, screen, false)
		tooltip._update_hover(0.401, screen, false)
		var expected: String = str(locale.ltr("UI_TOOLTIP_PERSON_ACTIVITY_FMT")) % [PersonNames.display_name(agent_id, language), str(locale.ltr("UI_ACTIVITY_TEND_FIRE"))]
		check(label.text == expected and panel.visible, "localized person and activity " + language)
	var original_bits: int = int(snap["ids"][0])
	snap["ids"][0] = original_bits + 1
	sprite.set_meta("entity_bits", original_bits + 1)
	tooltip._update_hover(1.0, screen, false)
	check(not panel.visible, "entity change restarts the entire hover delay")
	snap["ids"][0] = original_bits
	sprite.set_meta("entity_bits", original_bits)
	tooltip._update_hover(1.0, Vector2(-1, -1), false)
	check(not panel.visible, "mouse-out hides immediately")
	tooltip._update_hover(0.0, screen, false)
	tooltip._update_hover(0.401, screen, false)
	tooltip._update_hover(0.0, screen, true)
	check(not panel.visible, "blocking UI hides")
	tooltip._update_hover(0.0, screen, false)
	tooltip._update_hover(0.401, screen, false)
	sprite.set_meta("activity_kind", -1)
	tooltip._update_hover(0.0, screen, false)
	check(not panel.visible, "no activity hides immediately")
	sprite.set_meta("activity_kind", 0)
	sprite.set_meta("fallback_kind", "")
	tooltip._update_hover(1.0, screen, false)
	check(not panel.visible, "inactive daily row does not expose stale activity")
	sprite.set_meta("fallback_kind", "activity_icon")
	tooltip._update_hover(0.0, screen, false)
	tooltip._update_hover(0.401, screen, false)
	(main.get_node("UI") as CanvasLayer).hide()
	tooltip._update_hover(0.0, screen, false)
	check(not panel.visible, "hidden UI hides tooltip")
	(main.get_node("UI") as CanvasLayer).show()
	tooltip._update_hover(0.0, screen, false)
	tooltip._update_hover(0.401, screen, false)
	tooltip._notification(Node.NOTIFICATION_WM_MOUSE_EXIT)
	check(not panel.visible, "window exit clears tooltip")
	tooltip._update_hover(0.0, screen, false)
	tooltip._update_hover(0.401, screen, false)
	world.emit_signal("world_reset")
	check(not panel.visible and int(tooltip.get("_hovered")) == -1, "world reset clears hover identity")
	check(int(inspector.get_selected_entity_bits()) == selection, "hover never changes selection")
	check(tooltip.mouse_filter == Control.MOUSE_FILTER_IGNORE and panel.mouse_filter == Control.MOUSE_FILTER_IGNORE and label.mouse_filter == Control.MOUSE_FILTER_IGNORE, "tooltip never intercepts mouse input")
	locale.set_language(old_language)
	main.free()
	print("PERSON_ACTIVITY_TOOLTIP_P13 boundary_failures=", failures, " real_input=NOT_VERIFIED")
	quit(1 if failures else 0)
