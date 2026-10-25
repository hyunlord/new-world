# MUX2 dispatched input regression, real scene and bridge facts; no simulated UI outcomes.
# Bridge: get_tile_causal_history; normal scene reads. Runtime owned by root.
extends SceneTree
const View = preload("res://scripts/world/colony/view.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
var main: Node
var checks: int = 0
var orders: int = 0
var failures: int = 0
func _init() -> void:
	call_deferred("run")
func frames(n: int = 4) -> void:
	for _i: int in n:
		await process_frame
func check(label: String, good: bool) -> void:
	checks += 1
	print("MUX2_WHY_%s %s" % ["PASS" if good else "FAIL", label])
	if not good:
		failures += 1
func pointer(point: Vector2, pressed: bool, alt: bool = false) -> void:
	var motion := InputEventMouseMotion.new()
	motion.position = point
	Input.parse_input_event(motion)
	await frames(1)
	var event := InputEventMouseButton.new()
	event.position = point
	event.button_index = MOUSE_BUTTON_LEFT
	event.pressed = pressed
	event.alt_pressed = alt
	Input.parse_input_event(event)
	await frames(1)
func run() -> void:
	main = load("res://scenes/main.tscn").instantiate()
	main.get_node("UI/MuxHud").set("store", Store.new("/Users/rexxa/work/mux2-data/why-settings-%d.cfg" % OS.get_process_id()))
	root.add_child(main)
	await frames(20)
	var camera: Camera2D = main.get_node("Camera2D")
	camera.call("set_paused", true)
	camera.call("center_on_tile", Vector2i(96,96))
	await frames()
	var panel: Control = main.get_node("UI/CausalPanel")
	var point: Vector2 = root.get_visible_rect().size * Vector2(0.55,0.55)
	panel.hide()
	await pointer(point, true)
	await pointer(point, false)
	await frames()
	check("ordinary map click does not open causal panel", not panel.visible)
	panel.hide()
	await pointer(point, true, true)
	await pointer(point + Vector2(80,20), false, true)
	await frames()
	check("alt drag does not open causal panel", not panel.visible)
	panel.hide()
	await pointer(point, true, true)
	await pointer(point, false, true)
	await frames()
	var tile: Vector2i = View.world_to_tile(root.get_canvas_transform().affine_inverse() * point)
	var history: Array = main.get_node("WorldSim").get_tile_causal_history(tile.x,tile.y)
	var has_content: bool = not history.is_empty() or str(panel.call("wait_text",tile)) != ""
	check("alt click matches actual tile content", panel.visible == has_content)
	if not has_content:
		var toast: Control = main.get_node("UI/OrderBar").get("_toast_panel")
		var toast_label: Label = main.get_node("UI/OrderBar").get("_toast")
		var empty_text: String = str(root.get_node("Locale").call("ltr","MUX_WHY_EMPTY"))
		var empty_shown: bool = toast != null and toast.visible and toast_label.text.replace("\u2060", "") == empty_text
		check("empty tile uses toast", empty_shown)
		await create_timer(2.1).timeout
		check("empty toast expires after two seconds",empty_shown and (not toast.visible or toast_label.text.replace("\u2060", "") != empty_text))
	if panel.visible:
		check("popup maximum screen width", panel.size.x * panel.scale.x <= 420.5)
		check("popup maximum screen height", panel.size.y * panel.scale.y <= root.get_visible_rect().size.y * 0.5 + 0.5)
	# Positive fixture uses the actual engine event path, never synthetic display_history.
	var world: Node = main.get_node("WorldSim")
	check("fixture accepted by bridge", bool(SimReads.mutate(world,"on_building_placed",[tile.x,tile.y,1])))
	SimReads.mutate(world,"advance_ticks",[1])
	await frames()
	var recorded: Array = world.get_tile_causal_history(tile.x,tile.y)
	check("engine recorded a real causal event", not recorded.is_empty())
	panel.hide()
	await pointer(point,true,true)
	await pointer(point,false,true)
	await frames(10)
	check("alt click opens recorded event",panel.visible)
	if panel.visible:
		check("recorded popup screen width",panel.size.x * panel.scale.x <= 420.5)
		check("recorded popup half screen height",panel.size.y * panel.scale.y <= root.get_visible_rect().size.y * 0.5 + 0.5)
	await _placement_regression(panel, camera, tile)
	panel.call("open_tile", tile)
	await frames()
	var key := InputEventKey.new()
	key.keycode = KEY_ESCAPE
	key.physical_keycode = KEY_ESCAPE
	key.pressed = true
	Input.parse_input_event(key)
	await frames()
	check("Esc closes popup", not panel.visible)
	# The top-row selector deliberately opens the same real tile explanation.
	var why_button: Control = main.get_node_or_null("UI/MuxHud/Actions/WhyButton")
	check("top why tool exists",why_button != null)
	if why_button != null:
		var button_point: Vector2 = why_button.get_global_rect().get_center()
		await pointer(button_point,true)
		await pointer(button_point,false)
		await pointer(point,true)
		await pointer(point,false)
		await frames(10)
		check("top why tool opens real history",panel.visible)
		var close_button: Control = panel.find_child("Close",true,false)
		check("popup close button exists",close_button != null)
		if close_button != null:
			await pointer(close_button.get_global_rect().get_center(),true)
			await pointer(close_button.get_global_rect().get_center(),false)
			check("popup close button works",not panel.visible)
	for factor: float in [1.0,1.25]:
		main.get_node("UI/MuxHud").get("store").set_value("ui_scale",factor)
		await frames(10)
		await pointer(point,true,true)
		await pointer(point,false,true)
		await frames(12)
		check("scale%s opens history" % factor,panel.visible)
		check("scale%s width bound" % factor,panel.size.x * panel.scale.x <= 420.5)
		check("scale%s height bound" % factor,panel.size.y * panel.scale.y <= root.get_visible_rect().size.y*0.5 + 0.5)
		if DisplayServer.get_name() != "headless":
			var directory: String = "/Users/rexxa/work/mux2-data/ui-why/run-%d" % OS.get_process_id()
			print("MUX2_WHY_OUTPUT " + directory)
			DirAccess.make_dir_recursive_absolute(directory)
			await RenderingServer.frame_post_draw
			check("capture scale%s" % factor, root.get_texture().get_image().save_png(directory.path_join("why-%s.png" % factor)) == OK)
		var outside: Vector2 = Vector2(40,root.get_visible_rect().size.y*0.55)
		await pointer(outside,true)
		await pointer(outside,false)
		check("outside click closes",not panel.visible)
	# Crossing the threshold permanently suppresses click even if the pointer returns.
	panel.hide()
	await pointer(point,true,true)
	var away := InputEventMouseMotion.new()
	away.position = point + Vector2(80,20)
	away.relative = Vector2(80,20)
	away.button_mask = MOUSE_BUTTON_MASK_LEFT
	Input.parse_input_event(away)
	await frames(1)
	await pointer(point,false,true)
	await frames()
	check("out-and-back alt drag never opens",not panel.visible)
	if why_button != null:
		await pointer(point,true,true)
		var crossing := InputEventMouseMotion.new()
		crossing.position = why_button.get_global_rect().get_center()
		crossing.relative = crossing.position - point
		crossing.button_mask = MOUSE_BUTTON_MASK_LEFT
		Input.parse_input_event(crossing)
		await frames(1)
		await pointer(crossing.position,false,true)
		await frames()
		check("drag ending over GUI never opens",not panel.visible)
		check("GUI crossing releases camera gesture",int(camera.get("_pointer_button")) == MOUSE_BUTTON_NONE)
	camera.call("center_on_tile",tile)
	await frames(8)
	point = root.get_canvas_transform() * View.tile_centre(tile)
	var tool: Node = main.get_node("ColonyView/OrderTool")
	tool.connect("order_sent",func(_command: String,_result: Dictionary) -> void: orders += 1)
	tool.call("set_mode",1)
	await pointer(point,true,true)
	await pointer(point,false,true)
	await frames(8)
	check("alt click with active tool opens why",panel.visible)
	check("alt click preserves active tool without order",int(tool.get("mode")) == 1 and orders == 0)
	tool.call("set_mode",0)
	print("MUX2_WHY_DONE failures=%d checks=%d" % [failures,checks])
	quit(1 if failures else 0)

# Independent traversal reports actual visible panel/button rectangles, not the placement helper's output.
func _popup_intersections(node: Node, popup: Control, rectangle: Rect2, hits: Array[String]) -> void:
	if node == popup or (node is CanvasItem and not (node as CanvasItem).is_visible_in_tree()):
		return
	if node is Panel or node is PanelContainer or node is BaseButton:
		var control: Control = node as Control
		var actual: Rect2 = control.get_global_transform_with_canvas() * Rect2(Vector2.ZERO, control.size)
		if actual.has_area() and actual.intersects(rectangle):
			hits.append(str(control.get_path()))
	for child: Node in node.get_children():
		_popup_intersections(child, popup, rectangle, hits)


func _placement_regression(panel: Control, camera: Camera2D, tile: Vector2i) -> void:
	var saved_camera: Vector2 = camera.position
	var camera_processing: bool = camera.is_processing()
	camera.set_process(false)
	var store: RefCounted = main.get_node("UI/MuxHud").get("store")
	var saved_scale: float = float(store.call("value", "ui_scale"))
	# MuxHud owns this instance inside MapDock; use its verified public property.
	var minimap: Control = main.get_node("UI/MuxHud").get("minimap") as Control
	check("placement minimap fixture exists and is visible", minimap != null and minimap.is_visible_in_tree())
	if minimap == null or not minimap.is_visible_in_tree():
		camera.set_process(camera_processing)
		return
	for factor: float in [1.0, 1.25]:
		store.call("set_value", "ui_scale", factor)
		panel.hide()
		await frames(12)
		var obstacle: Rect2 = minimap.get_global_transform_with_canvas() * Rect2(Vector2.ZERO, minimap.size)
		# A room/item Why caller can anchor under the minimap: the old flipped popup overlaps its lower half.
		var desired: Vector2 = Vector2(obstacle.end.x - 8.0, obstacle.end.y - 24.0)
		var current: Vector2 = root.get_canvas_transform() * View.tile_centre(tile)
		camera.position += (current - desired) / camera.zoom
		camera.call("_clamp_position")
		await frames(8)
		panel.call("open_tile", tile)
		await frames(12)
		var rectangle: Rect2 = panel.get_global_transform_with_canvas() * Rect2(Vector2.ZERO, panel.size)
		var hits: Array[String] = []
		_popup_intersections(main.get_node("UI"), panel, rectangle, hits)
		print("MUX2_WHY_PLACEMENT " + JSON.stringify({"scale": factor, "popup": str(rectangle), "minimap": str(obstacle), "intersections": hits}))
		check("placement scale%s real history visible" % factor, panel.visible)
		check("placement scale%s original size limits" % factor, rectangle.size.x <= 420.5 and rectangle.size.y <= root.get_visible_rect().size.y * 0.5 + 0.5)
		check("placement scale%s remains in viewport" % factor, root.get_visible_rect().encloses(rectangle))
		check("placement scale%s clears actual visible controls" % factor, hits.is_empty())
		if DisplayServer.get_name() != "headless":
			var directory: String = "/Users/rexxa/work/mux2-data/ui-why-placement/run-%d" % OS.get_process_id()
			DirAccess.make_dir_recursive_absolute(directory)
			await RenderingServer.frame_post_draw
			check("placement capture scale%s" % factor, root.get_texture().get_image().save_png(directory.path_join("why-placement-%s.png" % factor)) == OK)
	panel.hide()
	store.call("set_value", "ui_scale", saved_scale)
	camera.position = saved_camera
	camera.set_process(camera_processing)
	await frames(12)
	# Full-screen visible occluder: no fit is possible, so retain bounded content and expose exhaustion.
	if panel.has_method("_place_popup"):
		var area: Vector2 = root.get_visible_rect().size
		var obstacles: Array[Rect2] = [Rect2(Vector2.ZERO, area)]
		var placed: Vector2 = panel.call("_place_popup", Vector2(20, 20), Vector2(200, 100), area, obstacles)
		check("saturated viewport reports exhausted bounded fallback", bool(panel.get("_placement_exhausted")) and Rect2(Vector2.ZERO, area).encloses(Rect2(placed, Vector2(200, 100))))
	else:
		check("saturated viewport reports exhausted bounded fallback", false)
