# Run the unchanged Why input contract in an explicit locale, with layout diagnostics.
# Bridge: inherited actual causal-history reads and construction fixture only.
extends "res://tools/v8/mux2_why_input_test.gd"

func run() -> void:
	var language: String = "ko"
	for argument: String in OS.get_cmdline_user_args():
		if argument.begins_with("--locale="):
			language = argument.trim_prefix("--locale=")
	root.get_node("Locale").call("set_language", language)
	print("MUX21_WHY_LOCALE " + language)
	await super.run()

func _placement_regression(panel: Control, camera: Camera2D, tile: Vector2i) -> void:
	await super._placement_regression(panel, camera, tile)
	var store: RefCounted = main.get_node("UI/MuxHud").get("store")
	var saved_scale: float = float(store.call("value", "ui_scale"))
	store.call("set_value", "ui_scale", 1.5)
	panel.call("open_tile", tile)
	await frames(12)
	check("placement scale1.5 real history visible", panel.visible)
	var rectangle: Rect2 = panel.get_global_rect()
	check("placement scale1.5 original size limits", rectangle.size.x <= 420.5 and rectangle.size.y <= root.get_visible_rect().size.y * 0.5 + 0.5)
	check("placement scale1.5 remains in viewport", root.get_visible_rect().encloses(rectangle))
	var hits: Array[String] = []
	_popup_intersections(main.get_node("UI"), panel, rectangle, hits)
	print("MUX21_WHY_SCALE15 " + JSON.stringify({"popup": str(rectangle), "intersections": hits}))
	var visible_hits: Array[String] = []
	_visible_intersections(main.get_node("UI"), panel, rectangle, root.get_visible_rect(), visible_hits)
	check("placement scale1.5 clears actual visible controls", visible_hits.is_empty())
	if DisplayServer.get_name() != "headless":
		var directory: String = "/Users/rexxa/work/mux2-data/ui-why/run-%d" % OS.get_process_id()
		DirAccess.make_dir_recursive_absolute(directory)
		await RenderingServer.frame_post_draw
		check("capture scale1.5", root.get_texture().get_image().save_png(directory.path_join("why-1.5.png")) == OK)
	panel.hide()
	store.call("set_value", "ui_scale", saved_scale)
	await frames(12)

# Additional 1.5x coverage accounts for clipped alert rows; the inherited 42 checks stay unchanged.
func _visible_intersections(node: Node, popup: Control, rectangle: Rect2, clip_rect: Rect2, hits: Array[String]) -> void:
	if node == popup or (node is CanvasItem and not (node as CanvasItem).is_visible_in_tree()):
		return
	var child_clip: Rect2 = clip_rect
	if node is Control:
		var control: Control = node as Control
		var actual: Rect2 = control.get_global_transform_with_canvas() * Rect2(Vector2.ZERO, control.size)
		if node is Panel or node is PanelContainer or node is BaseButton:
			var visible_rect: Rect2 = actual.intersection(clip_rect)
			if actual.intersects(rectangle):
				print("MUX21_WHY_CLIP " + JSON.stringify({"node": str(node.get_path()), "actual": str(actual), "clip": str(clip_rect), "visible": str(visible_rect)}))
			if visible_rect.has_area() and visible_rect.intersects(rectangle):
				hits.append(str(node.get_path()))
		if control.clip_contents:
			child_clip = clip_rect.intersection(actual)
	for child: Node in node.get_children():
		_visible_intersections(child, popup, rectangle, child_clip, hits)

func check(label: String, good: bool) -> void:
	if label.contains("original size limits") or label.contains("opens"):
		var popup: Control = main.get_node("UI/CausalPanel")
		var heading: Label = popup.get("_title_label")
		var camera: Camera2D = main.get_node("Camera2D")
		print("MUX21_WHY_STATE " + JSON.stringify({"check": label,
			"locale": root.get_node("Locale").get("_current_lang"),
			"popup_size": str(popup.size), "popup_scale": str(popup.scale),
			"heading_minimum": str(heading.get_combined_minimum_size()),
			"heading_parent_minimum": str(heading.get_parent().get_combined_minimum_size()),
			"panel_minimum": str((popup.get("_panel") as Control).get_combined_minimum_size()),
			"camera_position": str(camera.position),
			"canvas_transform": str(root.get_canvas_transform()),
			"original_point_tile": str(View.world_to_tile(root.get_canvas_transform().affine_inverse() * (root.get_visible_rect().size * Vector2(0.55, 0.55))))}))
	super.check(label, good)
	if label.contains("original size limits"):
		var popup: Control = main.get_node("UI/CausalPanel")
		var heading: Label = popup.get("_title_label")
		var close: Control = popup.find_child("Close", true, false)
		var bounds: Rect2 = popup.get_global_rect()
		super.check(label + " title fully visible", heading.get_visible_line_count() == heading.get_line_count() and bounds.encloses(heading.get_global_rect()) and not heading.clip_text)
		super.check(label + " close inside popup", bounds.encloses(close.get_global_rect()))
