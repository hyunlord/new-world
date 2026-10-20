# tools/v8/smoke.gd — headless label scan for the V8 smoke (tools/v8/smoke.sh).
# Purpose: boot res://scenes/main.tscn, run --frames N frames (default 900),
#   open the panels a player opens (a click on the first agent through
#   OrderTool.click_world -> inspector; a click on an empty tile ->
#   causal "why?" panel; the chronicle toggled as the C key does), then walk
#   every Label / RichTextLabel / Button and report texts that are
#   raw localization keys (Locale.ltr returns the key itself when it is missing
#   and logs nothing, so a missing key only shows as key-shaped text).
# Bridge methods called: get_agent_snapshot (to find an agent to click); OrderTool's click path
#   (get_agent_detail, get_tile_causal_history, can_place_blueprint).
# Output: one "SMOKE_RAW_KEY <node path> <text>" line per hit, then
#   "SMOKE_DONE frames=N labels=L raw_keys=K".
extends SceneTree

const KEY_RE: String = "^[A-Z][A-Z0-9]*(_[A-Z0-9]+)+$"

var _frames: int = 900


func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size():
		if args[i] == "--frames" and i + 1 < args.size():
			_frames = int(args[i + 1])
	call_deferred("_run")


func _run() -> void:
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	for _i: int in _frames:
		await process_frame
	await _open_panels(main)
	var regex: RegEx = RegEx.new()
	regex.compile(KEY_RE)
	var labels: int = 0
	var hits: int = 0
	var stack: Array[Node] = [main]
	while not stack.is_empty():
		var node: Node = stack.pop_back()
		for child: Node in node.get_children():
			stack.push_back(child)
		var text: String = ""
		if node is Label:
			text = (node as Label).text
		elif node is RichTextLabel:
			text = (node as RichTextLabel).get_parsed_text()
		elif node is Button:
			text = (node as Button).text
		else:
			continue
		labels += 1
		for token: String in text.split(" ", false):
			if regex.search(token.strip_edges().trim_suffix(":")) != null:
				hits += 1
				print("SMOKE_RAW_KEY %s %s" % [main.get_path_to(node), token])
	print("SMOKE_DONE frames=%d labels=%d raw_keys=%d" % [_frames, labels, hits])
	quit(0)


func _open_panels(main: Node) -> void:
	var world: Node = main.get_node_or_null("WorldSim")
	var tool: Node = main.get_node_or_null("ColonyView/OrderTool")
	if world != null and tool != null and tool.has_method("click_world"):
		var snap: Dictionary = world.call("get_agent_snapshot")
		var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
		var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
		if not xs.is_empty():
			var tile: Vector2i = Vector2i(xs[0], ys[0])
			tool.call("click_world", Vector2(tile) * 16.0 + Vector2(8, 8), tile)
			for _i: int in 10:
				await process_frame
		tool.call("click_world", Vector2(2, 2) * 16.0 + Vector2(8, 8), Vector2i(2, 2))
	# M1: show the order bar's hints and toast texts too (raw-key scan).
	var bar: Node = main.get_node_or_null("UI/OrderBar")
	if bar != null and bar.has_method("select_mode"):
		bar.call("select_mode", 3)
		bar.call("show_toast", "x")
	var chronicle: Node = main.get_node_or_null("UI/ChroniclePanel")
	if chronicle != null and chronicle.has_method("toggle_visible"):
		chronicle.call("toggle_visible")
	for _i: int in 30:
		await process_frame
