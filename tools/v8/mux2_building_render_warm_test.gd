# Windowed offscreen warmup oracle: actual per-texture pixels, identical main-view pixels, and temporary cleanup.
# Bridge methods: reset_world/advance_ticks via SimReads; get_time_snapshot/get_building_snapshot diagnostic reads; awaits world_reset.
# Real seed0 scene supplies light resources, then is removed. Pixel isolation uses a static scene; actual-game same-tick proof remains separate.
extends SceneTree

const Building = preload("res://scripts/world/colony/building_layer.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const LitWarm = preload("res://scripts/world/colony/lit_warm_representatives.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
var _failures: Array[String] = []
var _light_template: PointLight2D
var _custom_collection: Dictionary = {}
var _evidence: String

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	if DisplayServer.get_name() == "headless":
		push_error("MUX2_RENDER_WARM requires actual windowed renderer")
		quit(1)
		return
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var settings: String = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
	var project: String = ProjectSettings.globalize_path("res://").simplify_path().trim_suffix("/")
	if args.size() != 1 or not args[0].is_absolute_path() or args[0].simplify_path().begins_with(project + "/") or DirAccess.dir_exists_absolute(args[0]) or not settings.is_absolute_path() or settings.simplify_path().begins_with(project + "/"):
		push_error("lit warm fixture requires new external output and external settings directory")
		quit(1)
		return
	_evidence = args[0]
	if DirAccess.make_dir_recursive_absolute(_evidence) != OK or DirAccess.make_dir_recursive_absolute(settings) != OK:
		quit(1)
		return
	_light_template = await _actual_light_template(settings)
	var collection_report: Dictionary = _custom_collection.get("report", {})
	if _light_template == null or not bool(collection_report.get("complete", false)):
		push_error("actual light/custom collection unavailable: %s" % [_failures])
		_release_templates()
		quit(1)
		return
	_parent_traversal_controls()
	var ids: Array[String] = Building.texture_variant_ids(ArtData.bindings())
	if ids.is_empty():
		_release_templates()
		quit(1)
		return
	var background: ColorRect = ColorRect.new()
	background.color = Color(0.13, 0.21, 0.29, 1.0)
	background.size = root.get_visible_rect().size
	root.add_child(background)
	var reference: Sprite2D = Sprite2D.new()
	reference.texture = ArtData.texture(ids[0])
	reference.centered = false
	reference.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	reference.position = Vector2(40, 40)
	root.add_child(reference)
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	var before: Image = root.get_texture().get_image()
	var children_before: int = root.get_child_count()
	var viewport: SubViewport = Building.make_warm_viewport(ids)
	_check(viewport != null, "valid bounded viewport")
	if viewport == null:
		_release_templates()
		quit(1)
		return
	root.add_child(viewport)
	var warm_texture: ViewportTexture = viewport.get_texture()
	var requested_mode: int = viewport.render_target_update_mode
	_log_viewport("scheduled_before_render", viewport)
	await RenderingServer.frame_post_draw
	_log_viewport("first_post_draw_before_readback", viewport)
	var image: Image = warm_texture.get_image()
	var after: Image = root.get_texture().get_image()
	_log_viewport("first_post_draw_after_readback", viewport)
	var draws: int = viewport.get_render_info(Viewport.RENDER_INFO_TYPE_CANVAS, Viewport.RENDER_INFO_DRAW_CALLS_IN_FRAME)
	_check(draws > 0 and requested_mode == SubViewport.UPDATE_ONCE, "requested one-shot produced a real canvas render")
	_check(before.get_size() == after.get_size() and before.get_data() == after.get_data(), "main-view bytes unchanged while warming")
	_check(viewport.get_child_count() == ids.size(), "one sprite per resolved variant")
	for index: int in ids.size():
		var sprite: Sprite2D = viewport.get_child(index) as Sprite2D
		var original: Texture2D = ArtData.texture(ids[index])
		_check(sprite.texture.get_rid() == original.get_rid() and sprite.material == null and sprite.texture_filter == CanvasItem.TEXTURE_FILTER_NEAREST, "same RID material and filter " + ids[index])
		var rect: Rect2i = Rect2i(Vector2i(sprite.position), Vector2i(original.get_width(), original.get_height()))
		_check(Rect2i(Vector2i.ZERO, image.get_size()).encloses(rect), "unclipped texture " + ids[index])
		var visible_pixels: int = 0
		for y: int in range(rect.position.y, rect.end.y):
			for x: int in range(rect.position.x, rect.end.x):
				if image.get_pixel(x, y).a > 0.0: visible_pixels += 1
		_check(visible_pixels > 0, "actual drawn pixels " + ids[index])
	# Changing content must not redraw a consumed one-shot request, even when the node property still says ONCE.
	var changed_sprite: Sprite2D = viewport.get_child(0) as Sprite2D
	changed_sprite.texture = null
	await process_frame
	_log_viewport("next_process_frame", viewport)
	await RenderingServer.frame_post_draw
	_log_viewport("next_post_draw", viewport)
	var not_rearmed: Image = warm_texture.get_image()
	_check(not_rearmed.get_data() == image.get_data(), "content change does not redraw without rearm")
	viewport.render_target_update_mode = SubViewport.UPDATE_ONCE
	_log_viewport("explicitly_rearmed", viewport)
	await RenderingServer.frame_post_draw
	_log_viewport("rearmed_post_draw", viewport)
	var rearmed: Image = warm_texture.get_image()
	_check(rearmed.get_data() != image.get_data(), "explicit rearm renders changed content")
	viewport.queue_free()
	warm_texture = null
	await process_frame
	await process_frame
	await RenderingServer.frame_post_draw
	_check(get_nodes_in_group("mux2_render_warmup").is_empty() and root.get_child_count() == children_before, "temporary topology freed")
	var cleaned: Image = root.get_texture().get_image()
	_check(before.get_data() == cleaned.get_data(), "main-view bytes unchanged after cleanup")
	var missing: Array[String] = ["fixture_missing_variant"]
	_check(Building.make_warm_viewport(missing) == null, "missing variant rejected")
	var oversized: Array[String] = []
	for index: int in 65: oversized.append(ids[0])
	_check(Building.make_warm_viewport(oversized) == null, "texture count bound")
	await _lit_controls(ids, before)
	_release_templates()
	print("MUX2_RENDER_WARM_DONE variants=%d canvas_draws=%d failures=%s" % [ids.size(), draws, _failures])
	quit(0 if _failures.is_empty() else 1)

func _log_viewport(stage: String, viewport: SubViewport) -> void:
	print("MUX2_RENDER_WARM_SEQUENCE %s" % [JSON.stringify({"stage": stage, "frame": Engine.get_process_frames(),
		"usec": Time.get_ticks_usec(), "update_mode": viewport.render_target_update_mode,
		"canvas_draw_calls": viewport.get_render_info(Viewport.RENDER_INFO_TYPE_CANVAS, Viewport.RENDER_INFO_DRAW_CALLS_IN_FRAME),
		"queued_for_deletion": viewport.is_queued_for_deletion()})])

func _check(ok: bool, label: String) -> void:
	if not ok: _failures.append(label)

# Build the real seed0 scene only to obtain its data-resolved runtime light settings/resources.
# The source light is never toggled. Then remove this setup scene before the static pixel-isolation fixture.
func _actual_light_template(settings: String) -> PointLight2D:
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	main.get_node("UI/MuxHud").set("store", Store.new(settings.path_join("lit-warm-%d.cfg" % OS.get_process_id())))
	root.add_child(main)
	main.get_node("Camera2D").call("set_paused", true)
	var world: Node = main.get_node("WorldSim")
	var reset_receipt: Signal = Signal(world, "world_reset")
	var reset_ok: bool = bool(main.get("boot_ok")) and bool(SimReads.mutate(world, "reset_world", [0]))
	_check(reset_ok, "real scene seed0 bootstrap")
	_describe_light_sources(main, world, "immediate_after_reset_return")
	if reset_ok:
		# Native reset_world emits this signal deferred; DaylightController refreshes its nodes on that receipt.
		await reset_receipt
		_describe_light_sources(main, world, "after_deferred_world_reset")
		# BuildingStampSystem drains worldgen's queued buildings on the first native tick, not on reset.
		var tick_before: Dictionary = world.call("get_time_snapshot")
		var advanced: Variant = SimReads.mutate(world, "advance_ticks", [1])
		var tick_after: Dictionary = world.call("get_time_snapshot")
		print("MUX2_LIT_MANUAL_SETUP_TICK %s" % JSON.stringify({"before": tick_before, "after": tick_after, "requested": 1, "return": advanced}))
		_check(int(tick_before.get("tick", -1)) == 0 and int(tick_after.get("tick", -1)) == 1, "declared first native stamp tick")
		_describe_light_sources(main, world, "after_first_native_tick_before_render")
		# A real frame runs the existing Daylight process: empty lights reconcile immediately.
		await RenderingServer.frame_post_draw
		_describe_light_sources(main, world, "after_first_tick_normal_daylight_process")
	var template: PointLight2D = null
	for child: Node in main.get_node("DaylightController").get_children():
		if child is PointLight2D:
			var source: PointLight2D = child as PointLight2D
			var eligibility: Dictionary = _light_eligibility(source)
			print("MUX2_LIT_ELIGIBILITY %s" % JSON.stringify(eligibility))
			if not bool(eligibility["eligible"]): continue
			var before: Dictionary = _light_fields(source)
			template = Building.clone_warm_light(source)
			_check(_light_fields(source) == before, "actual source light untouched")
			if template != null:
				for field: String in Building.WARM_LIGHT_FIELDS:
					_check(template.get(field) == source.get(field), "actual cloned light field " + field)
			print("MUX2_LIT_SOURCE %s" % JSON.stringify(before))
			if template != null:
				_check(is_equal_approx((main.get_node("Camera2D") as Camera2D).zoom.x, 1.0), "custom source inventory zoom1")
				var collected: Variant = LitWarm.collect(main.get_node("ColonyView"), source)
				_custom_collection = collected if collected is Dictionary else {}
				var raw_report: Variant = _custom_collection.get("report")
				var collection_report: Dictionary = raw_report if raw_report is Dictionary else {"complete": false, "errors": ["collector returned no report"]}
				_custom_collection["report"] = collection_report
				print("MUX2_CUSTOM_COLLECTION %s" % JSON.stringify(collection_report))
				_check(bool(collection_report.get("complete", false)), "bounded actual custom receiver collection")
			break
	main.queue_free()
	await process_frame
	await process_frame
	return template

func _light_fields(light: PointLight2D) -> Dictionary:
	var fields: Dictionary = {"enabled": light.enabled, "energy": light.energy, "position": str(light.position)}
	for field: String in Building.WARM_LIGHT_FIELDS: fields[field] = str(light.get(field))
	fields["texture_rid"] = light.texture.get_rid().get_id() if light.texture != null else 0
	return fields

func _lit_controls(ids: Array[String], main_before: Image) -> void:
	var before: Dictionary = _light_fields(_light_template)
	var lit: SubViewport = Building.make_lit_warm_viewport(ids, _light_template)
	var unlit: SubViewport = Building.make_lit_warm_viewport(ids, _light_template)
	_check(lit != null and unlit != null, "bounded actual light viewports")
	if lit == null or unlit == null:
		if lit != null: lit.free()
		if unlit != null: unlit.free()
		return
	_check(ids.size() >= 3, "three independent pixel controls available")
	if ids.size() < 3:
		lit.free()
		unlit.free()
		return
	var custom_lit: Dictionary = LitWarm.append_to(lit, _custom_collection)
	var custom_unlit: Dictionary = LitWarm.append_to(unlit, _custom_collection)
	_check(bool(custom_lit.get("complete", false)) and bool(custom_unlit.get("complete", false)), "bounded custom viewport packing")
	print("MUX2_CUSTOM_PACKING %s" % JSON.stringify(custom_lit))
	if not bool(custom_lit.get("complete", false)) or not bool(custom_unlit.get("complete", false)):
		lit.free()
		unlit.free()
		return
	(unlit.get_node("Mux2WarmLight") as PointLight2D).enabled = false
	# These are private fixture receivers only: disjoint-mask and unshaded controls.
	for viewport: SubViewport in [lit, unlit]:
		(viewport.get_child(1) as Sprite2D).light_mask = 0
		var unshaded: CanvasItemMaterial = CanvasItemMaterial.new()
		unshaded.light_mode = CanvasItemMaterial.LIGHT_MODE_UNSHADED
		(viewport.get_child(2) as Sprite2D).material = unshaded
		root.add_child(viewport)
	var lit_texture: ViewportTexture = lit.get_texture()
	var unlit_texture: ViewportTexture = unlit.get_texture()
	await RenderingServer.frame_post_draw
	var lit_image: Image = lit_texture.get_image()
	var unlit_image: Image = unlit_texture.get_image()
	_check(lit.get_render_info(Viewport.RENDER_INFO_TYPE_CANVAS, Viewport.RENDER_INFO_DRAW_CALLS_IN_FRAME) > 0, "actual lit canvas draws")
	_check(root.get_texture().get_image().get_data() == main_before.get_data(), "main frame unchanged by actual lit warm")
	for index: int in 3:
		var sprite: Sprite2D = lit.get_child(index) as Sprite2D
		var rect: Rect2i = Rect2i(Vector2i(sprite.position), Vector2i(sprite.texture.get_size()))
		var left: Image = lit_image.get_region(rect)
		var right: Image = unlit_image.get_region(rect)
		if index == 0:
			var changed_opaque_rgb: int = 0
			for y: int in left.get_height():
				for x: int in left.get_width():
					var a: Color = left.get_pixel(x, y)
					var b: Color = right.get_pixel(x, y)
					if a.a > 0.0 and b.a > 0.0 and Vector3(a.r, a.g, a.b) != Vector3(b.r, b.g, b.b): changed_opaque_rgb += 1
			_check(changed_opaque_rgb > 0, "actual light changes opaque RGB")
			print("MUX2_LIT_CHANGED_PIXELS %d" % changed_opaque_rgb)
		else:
			_check(left.get_data() == right.get_data(), "mask/unshaded negative control %d" % index)
	for proxy: Dictionary in custom_lit.get("proxies", []):
		var values: Array = proxy["pixel_rect"]
		var rect: Rect2i = Rect2i(int(values[0]), int(values[1]), int(values[2]), int(values[3]))
		_check(Rect2i(Vector2i.ZERO, lit_image.get_size()).encloses(rect), "custom proxy fully in viewport")
		var sprite: Sprite2D = lit.get_node(str(proxy["node"])) as Sprite2D
		var receipt: Dictionary = proxy["source"]
		_check(sprite.texture.get_rid().get_id() == int(receipt["texture_rid"]) and sprite.material.get_rid().get_id() == int(receipt["material_rid"]), "custom same actual texture/material RIDs")
		_check(sprite.light_mask == int(receipt["light_mask"]) and sprite.z_index == int(receipt["absolute_z"]) and sprite.texture_filter == int(receipt["filter"]), "custom actual receiver state")
		var left: Image = lit_image.get_region(rect)
		var right: Image = unlit_image.get_region(rect)
		var opaque: int = 0
		var changed: int = 0
		for y: int in left.get_height():
			for x: int in left.get_width():
				var a: Color = left.get_pixel(x, y)
				var b: Color = right.get_pixel(x, y)
				if a.a > 0.0 and b.a > 0.0:
					opaque += 1
					if Vector3(a.r, a.g, a.b) != Vector3(b.r, b.g, b.b): changed += 1
		_check(opaque > 0 and changed > 0, "actual custom lit pixels " + str(receipt["shader_path"]))
		print("MUX2_CUSTOM_PIXELS %s" % JSON.stringify({"shader": receipt["shader_path"], "opaque": opaque, "changed_rgb": changed, "receipt": receipt}))
		var material: ShaderMaterial = sprite.material as ShaderMaterial
		for name: String in receipt["uniforms"]:
			var value: Variant = material.get_shader_parameter(name)
			var after: Variant = {"texture_rid": (value as Texture2D).get_rid().get_id()} if value is Texture2D else str(value)
			_check(after == (receipt["uniforms"] as Dictionary)[name], "shared custom uniform unchanged " + name)
	_check(lit_image.save_png(_evidence.path_join("lit.png")) == OK, "lit PNG retained")
	_check(unlit_image.save_png(_evidence.path_join("unlit.png")) == OK, "unlit PNG retained")
	_check(root.get_texture().get_image().save_png(_evidence.path_join("main.png")) == OK, "main PNG retained")
	var receipt_file: FileAccess = FileAccess.open(_evidence.path_join("custom-material-receipt.json"), FileAccess.WRITE)
	if receipt_file != null:
		receipt_file.store_string(JSON.stringify(custom_lit, "\t"))
		receipt_file.close()
	else:
		_check(false, "custom receipt write")
	_check(_light_fields(_light_template) == before, "retained actual template unchanged")
	lit.render_target_update_mode = SubViewport.UPDATE_DISABLED
	unlit.render_target_update_mode = SubViewport.UPDATE_DISABLED
	lit.queue_free()
	unlit.queue_free()
	lit_texture = null
	unlit_texture = null
	await process_frame
	await process_frame
	await RenderingServer.frame_post_draw
	_check(get_nodes_in_group("mux2_render_warmup").is_empty(), "lit warm topology freed")
	_check(root.get_texture().get_image().get_data() == main_before.get_data(), "main frame unchanged after lit cleanup")
	_check(Building.make_lit_warm_viewport(ids, null) == null, "missing actual light rejected")
	var no_texture: PointLight2D = PointLight2D.new()
	_check(Building.make_lit_warm_viewport(ids, no_texture) == null, "missing light texture rejected")
	no_texture.free()

func _light_eligibility(source: PointLight2D) -> Dictionary:
	var names: Array[String] = []
	for property: Dictionary in source.get_property_list(): names.append(str(property["name"]))
	var reasons: Array[String] = []
	if source.is_queued_for_deletion(): reasons.append("queued_for_deletion")
	if source.texture == null: reasons.append("texture_missing")
	if source.shadow_enabled: reasons.append("shadow_unsupported")
	for field: String in Building.WARM_LIGHT_FIELDS:
		if not names.has(field): reasons.append("property_missing:" + field)
	return {"path": str(source.get_path()), "instance_id": source.get_instance_id(), "eligible": reasons.is_empty(), "reasons": reasons, "property_names": names}

func _describe_light_sources(main: Node, world: Node, stage: String) -> void:
	var children: Array[Dictionary] = []
	for child: Node in main.get_node("DaylightController").get_children():
		var row: Dictionary = {"path": str(child.get_path()), "class": child.get_class(), "queued_for_deletion": child.is_queued_for_deletion()}
		if child is PointLight2D: row["eligibility"] = _light_eligibility(child as PointLight2D)
		children.append(row)
	var buildings: Dictionary = world.call("get_building_snapshot")
	var clock: Dictionary = world.call("get_time_snapshot")
	print("MUX2_LIT_SOURCE_SEQUENCE %s" % JSON.stringify({"stage": stage, "frame": Engine.get_process_frames(), "usec": Time.get_ticks_usec(), "clock": clock, "building_count": (buildings.get("ids", PackedInt64Array()) as PackedInt64Array).size(), "children": children}))

func _release_templates() -> void:
	if is_instance_valid(_light_template): _light_template.free()
	_light_template = null
	_custom_collection.clear()

func _parent_traversal_controls() -> void:
	var parent: Node2D = Node2D.new()
	parent.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	parent.modulate = Color(0.5, 0.75, 1.0, 1.0)
	var direct: Sprite2D = Sprite2D.new()
	parent.add_child(direct)
	_check(LitWarm._canvas_parent(direct) == parent, "public immediate canvas parent")
	_check(LitWarm._effective_property(direct, "texture_filter") == CanvasItem.TEXTURE_FILTER_NEAREST, "direct canvas filter inherited")
	_check(LitWarm._inherited_modulate(direct) == parent.modulate, "direct canvas modulate inherited")
	direct.top_level = true
	_check(LitWarm._canvas_parent(direct) == null, "top_level stops canvas parent chain")
	_check(LitWarm._effective_property(direct, "texture_filter") == 0 and LitWarm._inherited_modulate(direct) == Color.WHITE, "top_level stops inherited state")
	var gap: Node = Node.new()
	parent.add_child(gap)
	var indirect: Sprite2D = Sprite2D.new()
	gap.add_child(indirect)
	_check(LitWarm._canvas_parent(indirect) == null, "non-canvas node stops parent chain")
	_check(LitWarm._effective_property(indirect, "texture_filter") == 0 and LitWarm._inherited_modulate(indirect) == Color.WHITE, "non-canvas gap stops inherited state")
	parent.free()
