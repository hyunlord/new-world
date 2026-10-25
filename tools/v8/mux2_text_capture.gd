# MUX2 real-scene Korean text inventory and rendered capture at both requested scales.
# Bridge reads: get_agent_snapshot/get_agent_detail, get_rooms, get_colony_overlay, get_steward.
# Mutations only through actual UI defence toggle; optional load_world through SimReads.
extends SceneTree
const View = preload("res://scripts/world/colony/view.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
var main: Node
var out: String = "/Users/rexxa/work/mux2-data/ui-text/run-%d" % OS.get_process_id()
var inventory: Array[Dictionary] = []
var causal_histories: Array[Dictionary] = []
var failures: int = 0
func _init() -> void:
	call_deferred("run")
func frames(n: int = 5) -> void:
	for _i: int in n:
		await process_frame
func check(label: String, good: bool) -> void:
	print("MUX2_TEXT_%s %s" % ["PASS" if good else "FAIL",label])
	if not good: failures += 1
func hangul_at(text: String, at: int) -> bool:
	if at < 0 or at >= text.length(): return false
	var code: int = text.unicode_at(at)
	return code >= 0xac00 and code <= 0xd7a3
func shaped_lines(control: Control, text: String, wrap: int) -> Dictionary:
	var font: Font = control.get_theme_font("font")
	var font_size: int = control.get_theme_font_size("font_size")
	var width: float = control.size.x
	if control is Button:
		width -= control.get_theme_stylebox("normal").get_minimum_size().x
	var server: TextServer = TextServerManager.get_primary_interface()
	var shaped: RID = server.create_shaped_text()
	server.shaped_text_add_string(shaped,text,font.get_rids(),font_size,{},"ko")
	var flags: int = TextServer.BREAK_MANDATORY | TextServer.BREAK_WORD_BOUND
	if wrap == TextServer.AUTOWRAP_WORD_SMART: flags |= TextServer.BREAK_ADAPTIVE
	var ranges: PackedInt32Array = server.shaped_text_get_line_breaks(shaped,maxf(1,width),0,flags)
	var lines: PackedStringArray = []
	var intact: bool = true
	var fits: bool = true
	for i: int in range(0,ranges.size(),2):
		var start: int = ranges[i]
		var end: int = ranges[i+1]
		var line: String = text.substr(start,end-start)
		lines.append(line.replace("\u2060",""))
		fits = fits and font.get_string_size(line.strip_edges(),HORIZONTAL_ALIGNMENT_LEFT,-1,font_size).x <= width + 1.0
		var left: int = end - 1
		var right: int = end
		while left >= 0 and text[left] == "\u2060": left -= 1
		while right < text.length() and text[right] == "\u2060": right += 1
		if hangul_at(text,left) and hangul_at(text,right): intact = false
	server.free_rid(shaped)
	return {"lines":lines,"word_boundaries_intact":intact,"line_widths_fit":fits,"width":width}
func scan(node: Node, shot: String) -> void:
	if node is Control and (node as Control).is_visible_in_tree() and (node is Label or node is RichTextLabel or node is Button):
		var text: String = str(node.get("text"))
		if not text.is_empty():
			var control: Control = node as Control
			var row: Dictionary = {"shot":shot,"path":str(node.get_path()),"raw":text.replace("\u2060",""),"display":text,"rect":str(control.get_global_rect()),"minimum":str(control.get_combined_minimum_size()),"tooltip":control.tooltip_text.replace("\u2060","")}
			if node is Label:
				row["lines"] = (node as Label).get_line_count()
				row["visible_lines"] = (node as Label).get_visible_line_count()
				row["wrap"] = (node as Label).autowrap_mode
			if node is Button:
				row["clip_text"] = (node as Button).clip_text
				if str(node.get_path()).contains("/Alerts/"):
					check("alert text unclipped " + str(node.name), not (node as Button).clip_text)
			var wrap: int = int(node.get("autowrap_mode")) if node is Label or node is Button else 0
			if wrap != TextServer.AUTOWRAP_OFF:
				var shaping: Dictionary = shaped_lines(control,text,wrap)
				row["shaping"] = shaping
				check("Korean word boundaries " + str(node.get_path()),bool(shaping["word_boundaries_intact"]))
				check("wrapped widths fit " + str(node.get_path()),bool(shaping["line_widths_fit"]))
			inventory.append(row)
	for child: Node in node.get_children(): scan(child,shot)
func capture(shot: String) -> void:
	await frames(10)
	scan(root,shot)
	if DisplayServer.get_name() != "headless":
		await RenderingServer.frame_post_draw
		check("save " + shot, root.get_texture().get_image().save_png(out.path_join(shot+".png")) == OK)
func click(control: Control) -> void:
	await click_point(control.get_global_rect().get_center())
func click_point(point: Vector2) -> void:
	var motion := InputEventMouseMotion.new()
	motion.position = point
	Input.parse_input_event(motion)
	await frames(1)
	for down: bool in [true,false]:
		var event := InputEventMouseButton.new()
		event.position = point
		event.button_index = MOUSE_BUTTON_LEFT
		event.pressed = down
		Input.parse_input_event(event)
		await frames(2)
func wheel_into_view(scroll: ScrollContainer, label: Control, description: String) -> void:
	for _step: int in 40:
		var viewport_rect: Rect2 = scroll.get_global_rect()
		var target_rect: Rect2 = label.get_global_rect()
		if viewport_rect.encloses(target_rect):
			check(description+" fully visible after wheel scrolling",true)
			return
		var motion := InputEventMouseMotion.new()
		motion.position = viewport_rect.get_center()
		Input.parse_input_event(motion)
		var wheel := InputEventMouseButton.new()
		wheel.position = motion.position
		wheel.button_index = MOUSE_BUTTON_WHEEL_DOWN if target_rect.end.y > viewport_rect.end.y else MOUSE_BUTTON_WHEEL_UP
		wheel.pressed = true
		Input.parse_input_event(wheel)
		await frames(3)
	check(description+" fully visible after wheel scrolling",false)
func verify_explanation(tile: Vector2i, source: String) -> void:
	var causal: Control = main.get_node("UI/CausalPanel")
	var history: Array = main.get_node("WorldSim").get_tile_causal_history(tile.x,tile.y)
	causal_histories.append({"source": source, "tile": [tile.x,tile.y], "history": history})
	for event: Dictionary in history:
		var rendered: String = str(causal.call("_format_event",event)).replace("\u2060", "")
		check(source+" event has readable category "+str(event.get("kind", "")),not rendered.ends_with("] ?") and not rendered.contains("UI_CAUSAL_") and not rendered.ends_with(str(root.get_node("Locale").call("ltr","UI_CAUSAL_EVENT_OTHER"))))
	var has_content: bool = not history.is_empty() or not str(causal.call("wait_text",tile)).is_empty()
	check(source+" why targets exact live tile",causal.get("_anchor_tile") == tile)
	check(source+" why matches real content",causal.visible == has_content)
	if not has_content:
		var toast: Control = main.get_node("UI/OrderBar").get("_toast_panel")
		var label: Label = main.get_node("UI/OrderBar").get("_toast")
		var expected: String = str(root.get_node("Locale").call("ltr","MUX_WHY_EMPTY"))
		check(source+" empty why shows actual toast",toast.visible and label.text.replace("\u2060", "") == expected)
func run() -> void:
	print("MUX2_TEXT_OUTPUT " + out)
	DirAccess.make_dir_recursive_absolute(out)
	main = load("res://scenes/main.tscn").instantiate()
	var hud: Node = main.get_node("UI/MuxHud")
	hud.set("store",Store.new(out.path_join("settings-%d.cfg" % OS.get_process_id())))
	root.add_child(main)
	await frames(25)
	var world: Node = main.get_node("WorldSim")
	main.get_node("Camera2D").call("set_paused",true)
	root.get_node("Locale").call("set_language","ko")
	var arguments: PackedStringArray = OS.get_cmdline_user_args()
	if arguments.size() > 0:
		check("load real save",bool(SimReads.mutate(world,"load_world",[arguments[0]])))
	await frames(15)
	for scale: float in [1.0,1.25]:
		hud.get("store").set_value("ui_scale",scale)
		var suffix: String = str(scale).replace(".","_")
		await capture("hud-"+suffix)
		var hover := InputEventMouseMotion.new()
		hover.position = (hud.get_node("Actions/WhyButton") as Control).get_global_rect().get_center()
		Input.parse_input_event(hover)
		await create_timer(0.8).timeout
		await capture("why-tooltip-"+suffix)
		await click(hud.get_node("Actions/HelpButton"))
		await capture("help-"+suffix)
		hud.get("help").hide()
		await click(hud.get_node("Actions/SettingsButton"))
		await capture("settings-"+suffix)
		var before: Dictionary = SimReads.read(world,"get_steward")
		var toggle: Control = hud.get("settings").find_child("StewardDefence",true,false)
		check("settings defence exists",toggle != null)
		if toggle != null:
			await click(toggle)
			await frames(35)
			var after: Dictionary = SimReads.read(world,"get_steward")
			check("settings defence dispatched toggle", bool(after.get("defence_enabled",true)) != bool(before.get("defence_enabled",true)))
			await click(toggle)
		hud.get("settings").hide()
		var people: Dictionary = SimReads.read(world,"get_agent_snapshot")
		var ids: PackedInt64Array = people.get("ids",PackedInt64Array())
		if not ids.is_empty():
			main.get_node("UI/AgentInspectorPanel").call("display_agent",world.get_agent_detail(ids[0]))
			var initial_inspector: Control = main.get_node("UI/AgentInspectorPanel")
			(initial_inspector.get("_scroll") as ScrollContainer).scroll_vertical = 0
			await frames()
			var look: Control = initial_inspector.get("_look_label") as Control
			check("Look fully visible at "+suffix,(initial_inspector.get("_scroll") as ScrollContainer).get_global_rect().encloses(look.get_global_rect()))
			await capture("person-"+suffix)
			var why: Control = main.get_node("UI/AgentInspectorPanel").find_child("Why",true,false)
			check("person why exists", why != null)
			if why != null:
				await click(why)
				var detail: Dictionary = world.get_agent_detail(ids[0])
				verify_explanation(Vector2i(int(detail.get("x",0)),int(detail.get("y",0))),"person")
				await capture("person-why-"+suffix)
			main.get_node("UI/CausalPanel").hide()
			var inspector: Control = main.get_node("UI/AgentInspectorPanel")
			await wheel_into_view(inspector.get("_scroll") as ScrollContainer,inspector.get("_mood_label") as Control,"Mood "+suffix)
			await capture("person-details-"+suffix)
			inspector.hide()
		var rooms: Dictionary = SimReads.read(world,"get_rooms")
		var room_ids: PackedInt32Array = rooms.get("ids",PackedInt32Array())
		if not room_ids.is_empty():
			var room_panel: Control = main.get_node("UI/RoomPanel")
			room_panel.call("display_room",room_ids[0])
			await capture("room-"+suffix)
			var room_why: Control = room_panel.find_child("Why",true,false)
			check("room why exists",room_why != null)
			if room_why != null:
				await click(room_why)
				var centres_x: PackedInt32Array = rooms.get("centre_xs",PackedInt32Array())
				var centres_y: PackedInt32Array = rooms.get("centre_ys",PackedInt32Array())
				verify_explanation(Vector2i(centres_x[0],centres_y[0]),"room")
				await capture("room-why-"+suffix)
			room_panel.hide()
			main.get_node("UI/CausalPanel").hide()
		else:
			print("MUX2_TEXT_NOT_RUN room capture: no real room in supplied world")
		var items: Dictionary = SimReads.read(world,"get_colony_overlay")
		var item_xs: PackedInt32Array = items.get("item_xs",PackedInt32Array())
		var item_ys: PackedInt32Array = items.get("item_ys",PackedInt32Array())
		var item_tile: Vector2i = Vector2i(-1,-1)
		for item_index: int in mini(item_xs.size(),item_ys.size()):
			var candidate: Vector2i = Vector2i(item_xs[item_index],item_ys[item_index])
			if int(main.get_node("ColonyView/AgentLayer").call("entity_at",View.tile_centre(candidate))) < 0 and int(main.get_node("ColonyView/OrderTool").call("workshop_at",candidate)) < 0:
				item_tile = candidate
				break
		if item_tile.x >= 0:
			var tile: Vector2i = item_tile
			main.get_node("Camera2D").call("center_on_tile",tile)
			await frames(10)
			await click_point(root.get_canvas_transform()*View.tile_centre(tile))
			await frames(10)
			var item_panel: Control = main.get_node_or_null("UI/ItemPanel")
			check("item inspector opens from dispatched tile selection",item_panel != null and item_panel.visible)
			if item_panel != null and item_panel.visible:
				await capture("item-"+suffix)
				var item_why: Control = item_panel.find_child("Why",true,false)
				check("item why exists",item_why != null)
				if item_why != null:
					await click(item_why)
					verify_explanation(tile,"item")
					await capture("item-why-"+suffix)
				item_panel.hide()
				main.get_node("UI/CausalPanel").hide()
		else:
			print("MUX2_TEXT_NOT_RUN item capture: no real items in supplied world")
	var file: FileAccess = FileAccess.open(out.path_join("text-nodes.json"),FileAccess.WRITE)
	check("inventory writable",file != null)
	if file != null:
		file.store_string(JSON.stringify({"nodes":inventory,"causal_histories":causal_histories,"failures":failures,"renderer":DisplayServer.get_name()},"\t"))
	print("MUX2_TEXT_DONE failures=%d nodes=%d" % [failures,inventory.size()])
	quit(1 if failures else 0)
