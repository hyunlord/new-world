# Windowed real-scene food observations; root serializes execution.
# Args: NEW_EXTERNAL_OUT [MAX_DAYS=30] [OPTIONAL_REAL_SAVE] [--focus why|pot-food] [--provenance FILE]
# Focus mode omits harvest and proves only the named case; save provenance file is recorded when supplied.
# Inputs: reset seed0 or explicitly supplied save, ordinary full-map harvest designation,
# explicit tick advances, Korean/UI scales. Never inject physical food, meals or chronicle rows.
# Reuses rendered text inventory, line-fit checks and PNG metadata; visual verdict remains pending.
extends "res://tools/v8/mux21_play_capture.gd"

const ALL_CASES: Array[String] = ["phase0", "phase1", "phase2", "phase3", "meal", "food", "cooking", "winter", "winter-chronicle", "chronicle", "why", "pot-food", "workshop-food"]
var focus: String = ""
var provenance: String = ""
var source_save: String = ""
var required_cases: Array[String] = ALL_CASES.duplicate()
var observed: Dictionary = {}
var food_cursor: int = 0
var pot_scan_cursor: int = 0
var pot_scan_tick: int = -60

func run() -> void:
	var args := PackedStringArray()
	var raw_args: PackedStringArray = OS.get_cmdline_user_args()
	var cursor: int = 0
	while cursor < raw_args.size():
		var argument: String = raw_args[cursor]
		if argument in ["--focus", "--provenance"]:
			if cursor + 1 >= raw_args.size():
				quit(1)
				return
			cursor += 1
			if argument == "--focus": focus = raw_args[cursor]
			else: provenance = raw_args[cursor]
		elif argument.begins_with("--"):
			quit(1)
			return
		else: args.append(argument)
		cursor += 1
	if focus not in ["", "why", "pot-food"] or args.size() > 3 or (not provenance.is_empty() and not FileAccess.file_exists(provenance)):
		quit(1)
		return
	if not focus.is_empty(): required_cases = [focus]
	if args.is_empty():
		push_error("M6a capture requires new external output directory")
		quit(1)
		return
	out = args[0].simplify_path().trim_suffix("/")
	var project: String = ProjectSettings.globalize_path("res://").trim_suffix("/")
	if not out.is_absolute_path() or out == project or out.begins_with(project + "/") or DirAccess.dir_exists_absolute(out):
		quit(1)
		return
	if DirAccess.make_dir_recursive_absolute(out) != OK:
		quit(1)
		return
	started = Time.get_ticks_msec()
	create_timer(1800.0).timeout.connect(_watchdog)
	mode = "m6a" if focus.is_empty() else "m6a-focused-" + focus
	evidence["coverage_scope"] = "full" if focus.is_empty() else "focused_only"
	evidence["required_cases"] = required_cases
	evidence["full_suite_complete"] = false
	evidence["commands"] = []
	source_save = args[2] if args.size() > 2 else ""
	inputs = _hashes()
	if not provenance.is_empty():
		evidence["save_provenance"] = {"path": provenance, "sha256": FileAccess.get_sha256(provenance), "content": FileAccess.get_file_as_string(provenance)}
	else:
		evidence["save_provenance"] = {"status": "not_supplied", "note": "A save hash identifies bytes, not its generation history"}
	evidence["fixture_inputs"] = {"seed": 0, "state_injection": false, "advance_step": 60}
	check("windowed capture required", DisplayServer.get_name() != "headless")
	if failures > 0:
		_finish()
		return
	main = load("res://scenes/main.tscn").instantiate()
	hud = main.get_node("UI/MuxHud")
	hud.set("store", Store.new(out.path_join("settings.cfg")))
	root.add_child(main)
	check("main boot", bool(main.get("boot_ok")))
	world = main.get_node("WorldSim")
	camera = main.get_node("Camera2D") as Camera2D
	inspector = main.get_node("UI/AgentInspectorPanel") as Control
	causal = main.get_node("UI/CausalPanel") as Control
	world.connect("world_reset", _pause_fixture)
	_pause_fixture()
	if args.size() > 2:
		evidence["fixture_inputs"]["save"] = args[2]
		evidence["fixture_inputs"]["save_sha256"] = FileAccess.get_sha256(args[2])
		var loaded: Variant = SimReads.mutate(world, "load_world", [args[2]])
		evidence["fixture_inputs"]["load_result"] = loaded
		evidence["commands"].append({"method": "load_world", "args": [args[2]], "result": loaded})
		check("load real save", bool(loaded))
	else:
		SimReads.mutate(world, "reset_world", [0])
		evidence["commands"].append({"method": "reset_world", "args": [0]})
	if failures > 0:
		_finish()
		return
	await frames(8)
	_pause_fixture()
	root.get_node("Locale").call("set_language", "ko")
	hud.get("store").set_value("helper_on", false)
	var info: Dictionary = SimReads.read(world, "get_world_info")
	evidence["fixture_inputs"]["world_info"] = info
	evidence["fixture_inputs"]["seed"] = int(info.get("seed", 0))
	evidence["fixture_inputs"]["initial_tick"] = _tick()
	evidence["fixture_inputs"]["harvest_policy"] = "full_map" if focus.is_empty() else "none_focused_observation"
	if focus.is_empty():
		var harvest_args: Array = [0, 0, int(info["width"]) - 1, int(info["height"]) - 1]
		var harvest_result: Variant = SimReads.mutate(world, "cmd_designate_harvest", harvest_args)
		evidence["fixture_inputs"]["harvest_result"] = harvest_result
		evidence["commands"].append({"tick": _tick(), "method": "cmd_designate_harvest", "args": harvest_args, "result": harvest_result})
	var per_day: int = int(SimReads.read(world, "get_time_snapshot").get("ticks_per_day", 1440))
	var end_tick: int = _tick() + per_day * clampi(args[1].to_int() if args.size() > 1 else 30, 1, 48)
	while _tick() <= end_tick and failures == 0:
		await _observe_food()
		if _tick() == end_tick or (not focus.is_empty() and observed.has(focus)): break
		var advance: int = mini(60, end_tick - _tick())
		evidence["commands"].append({"tick": _tick(), "method": "advance_ticks", "args": [advance]})
		SimReads.mutate(world, "advance_ticks", [advance])
		await frames(1)
	evidence["observed"] = observed
	for required: String in required_cases:
		check("observed natural " + required, observed.has(required))
	evidence["fixture_inputs"]["final_tick"] = _tick()
	evidence["full_suite_complete"] = focus.is_empty() and failures == 0 and inputs == _hashes()
	evidence["coverage_note"] = ("Focused completion covers only " + focus + "; never full-suite acceptance. " if not focus.is_empty() else "") + "Missing cases are missing evidence, never synthesized; use a disclosed real simulation save for focused follow-up."
	_finish()

func _food_shots(name: String, tile: Vector2i, facts: Dictionary, panel: Control = null) -> void:
	camera.position = View.tile_centre(tile)
	camera.zoom = Vector2.ONE
	camera.call("_clamp_position")
	for factor: float in [1.0, 1.25]:
		hud.get("store").set_value("ui_scale", factor)
		await create_timer(0.6).timeout
		await frames(5)
		if panel != null:
			_fit_control(panel, name + " panel")
		await _shot("%s-%s" % [name, factor], {"scale": factor, "facts": facts, "tile": [tile.x, tile.y]})
	observed[name] = {"tick": _tick(), "facts": facts}
	if panel != null: panel.hide()

func _observe_food() -> void:
	var plants: Dictionary = world.get_plants(0, 0, 65535, 65535)
	for phase: int in 4:
		if _wanted("phase%d" % phase) and not observed.has("phase%d" % phase):
			await _plant_shots(plants, phase)
	var agents: Dictionary = SimReads.read(world, "get_agent_snapshot")
	if _wanted("meal") and not observed.has("meal"):
		for bits: int in agents.get("ids", []):
			var food: Dictionary = world.get_agent_food(bits)
			if not food.get("meal_kinds", []).is_empty():
				var detail: Dictionary = world.get_agent_detail(bits)
				inspector.call("display_agent", detail)
				await frames(5)
				var label: Control = inspector.get("_food_label") as Control
				(inspector.get("_scroll") as ScrollContainer).ensure_control_visible(label)
				await frames(5)
				_fit_control(label, "food meal text")
				await _food_shots("meal", Vector2i(detail["x"], detail["y"]), food, inspector)
				break
	var overlay: Dictionary = SimReads.read(world, "get_colony_overlay")
	if _wanted("food") and not observed.has("food"):
		for i: int in overlay.get("item_xs", []).size():
			var tile := Vector2i(overlay["item_xs"][i], overlay["item_ys"][i])
			var food: Dictionary = world.get_tile_food(tile.x, tile.y)
			if food.get("kinds", []).is_empty(): continue
			await _inspect_food_route(tile, "food")
			break
	if _wanted("pot-food") and not observed.has("pot-food") and _tick() - pot_scan_tick >= 60:
		pot_scan_tick = _tick()
		# A deployed vessel has no ground-item row. Inspect actual storage cells,
		# at most32 getters per60ticks, rotating through larger stockpiles.
		var cells: Dictionary = SimReads.read(world, "get_ui_facts").get("stockpile_tiles", {})
		var xs: PackedInt64Array = cells.get("xs", PackedInt64Array())
		var ys: PackedInt64Array = cells.get("ys", PackedInt64Array())
		var count: int = mini(xs.size(), ys.size())
		var reads: int = 0
		for _attempt: int in mini(count, 32):
			var index: int = pot_scan_cursor % count
			pot_scan_cursor = (index + 1) % count
			var tile := Vector2i(xs[index], ys[index])
			var contents: Dictionary = world.get_tile_food(tile.x, tile.y)
			reads += 1
			if "pot" in contents.get("storage", []):
				if not evidence.has("pot_candidates"): evidence["pot_candidates"] = []
				evidence["pot_candidates"].append({"tick": _tick(), "tile": [tile.x, tile.y], "food": contents.duplicate(true)})
				await _inspect_food_route(tile, "pot-food")
				break
		evidence["pot_scan"] = {"tick": _tick(), "stockpile_cells": count, "last_reads": reads, "next_cell": pot_scan_cursor, "max_reads": 32, "interval_ticks": 60}
	if _wanted("workshop-food") and not observed.has("workshop-food"):
		var shops: Dictionary = SimReads.read(world, "get_workshops")
		for i: int in shops.get("ids", []).size():
			var work := Vector2i(shops["work_xs"][i], shops["work_ys"][i])
			var contents: Dictionary = world.get_tile_food(work.x, work.y)
			if not contents.get("kinds", []).is_empty():
				await _inspect_food_route(Vector2i(shops["xs"][i], shops["ys"][i]), "workshop-food", Vector2i(shops["ws"][i], shops["hs"][i]))
				break
	var facts: Dictionary = SimReads.read(world, "get_ui_facts").get("food_chain", {})
	if bool(facts.get("preparing", false)) and int(facts.get("stored_units", 0)) < int(facts.get("winter_need", 0)) and _wanted("winter") and not observed.has("winter"):
		await _food_shots("winter", Vector2i(facts.get("x", 0), facts.get("y", 0)), facts)
	var chronicle: Control = main.get_node("UI/ChroniclePanel") as Control
	for row: Dictionary in _rows(food_cursor):
		food_cursor = maxi(food_cursor, int(row.get("index", -1)) + 1)
		if int(row.get("kind", -1)) < 48: continue
		var why_read: Dictionary = world.get_chronicle_why(int(row["index"]))
		if not evidence.has("food_why_reads"): evidence["food_why_reads"] = []
		evidence["food_why_reads"].append({"index": row["index"], "tick": _tick(), "read": why_read.duplicate(true)})
		check("food why metadata aligns with links", why_read.has("food_daily_milli") and why_read.has("links") and why_read["food_daily_milli"].size() == why_read["links"].size())
		if int(row.get("kind", -1)) == 50:
			check("winter recorded historical denominator present", int(row.get("food_daily_milli", -1)) >= 0)
			if _wanted("winter-chronicle") and not observed.has("winter-chronicle"):
				var sentence: String = ChronicleText.winter_food_sentence(row, root.get_node("Locale"))
				check("winter quantities rendered", sentence.contains(str(row.get("value", -1))) and sentence.contains(str(row.get("cause", -1))))
				await _chronicle_shots("winter-chronicle", row)
		if int(row.get("kind", -1)) == 48 and _wanted("cooking") and not observed.has("cooking"):
			await _chronicle_shots("cooking", row)
		if _wanted("chronicle") and not observed.has("chronicle"):
			await _chronicle_shots("chronicle", row)
		if int(row.get("kind", -1)) == 52 and _wanted("why") and not observed.has("why"):
			if bool(chronicle.call("open_why", row["index"])):
				await _food_shots("why", View.world_to_tile(camera.position), row, main.get_node("UI/WhyPanel") as Control)


func _hashes() -> Dictionary:
	var hashes: Dictionary = super._hashes()
	for external: String in [source_save, provenance]:
		if not external.is_empty(): hashes[external] = FileAccess.get_sha256(external)
	for path: String in ["res://tools/v8/m6a_ui_capture.gd", "res://assets/ui/food_chronicle.json", "res://assets/ui/alerts.json", "res://assets/v8art/render_params.json", "res://scripts/world/colony/plant_layer.gd", "res://scripts/ui/food_text.gd"]:
		hashes[path] = FileAccess.get_sha256(path)
	return hashes


func _inspect_food_route(tile: Vector2i, name: String, footprint: Vector2i = Vector2i.ONE) -> void:
	for node_name: String in ["ItemPanel", "RoomPanel", "WorkshopPanel", "AgentInspectorPanel", "CausalPanel"]:
		var node: CanvasItem = main.get_node_or_null("UI/" + node_name) as CanvasItem
		if node != null: node.hide()
	camera.position = View.tile_centre(tile)
	camera.call("_clamp_position")
	await frames(5)
	var order: Node = main.get_node("ColonyView/OrderTool")
	var agents: Node = main.get_node("ColonyView/AgentLayer")
	var visitors: Node = main.get_node("ColonyView/VisitorLayer")
	var click_tile: Vector2i = tile
	var point: Vector2 = View.tile_centre(tile)
	var clear_point: bool = false
	# A workshop origin may be covered by a person's drawn body. Search only the
	# real footprint; never suppress the production person/visitor hit test.
	for y: int in footprint.y:
		for x: int in footprint.x:
			for offset: Vector2 in [Vector2(0.2, 0.2), Vector2(0.8, 0.2), Vector2(0.2, 0.8), Vector2(0.8, 0.8), Vector2(0.5, 0.5)]:
				var candidate_tile: Vector2i = tile + Vector2i(x, y)
				var candidate: Vector2 = View.tile_to_world(candidate_tile) + offset * View.TILE
				if int(agents.call("entity_at", candidate, false)) >= 0: continue
				if int(visitors.call("caravan_at", candidate)) >= 0 or int(visitors.call("wave_at", candidate)) >= 0: continue
				click_tile = candidate_tile
				point = candidate
				clear_point = true
				break
			if clear_point: break
		if clear_point: break
	var diagnostic: Dictionary = {"name": name, "tick": _tick(), "origin": [tile.x, tile.y], "tile": [click_tile.x, click_tile.y], "point": [point.x, point.y], "clear": clear_point, "workshop": order.call("workshop_at", click_tile), "agent": agents.call("entity_at", point, false), "caravan": visitors.call("caravan_at", point), "wave": visitors.call("wave_at", point)}
	if not evidence.has("inspection_attempts"): evidence["inspection_attempts"] = []
	evidence["inspection_attempts"].append(diagnostic)
	if not clear_point:
		# No safe natural entry this tick: remain missing evidence and retry later.
		return
	order.call("click_world", point, click_tile)
	await frames(5)
	diagnostic["visible_panels"] = []
	for child: Node in main.get_node("UI").get_children():
		if child is CanvasItem and (child as CanvasItem).visible:
			diagnostic["visible_panels"].append(str(child.name))
	var owner: Control = null
	var button: Button = null
	for node_name: String in ["RoomPanel", "WorkshopPanel", "ItemPanel"]:
		var candidate: Control = main.get_node_or_null("UI/" + node_name) as Control
		if candidate != null and candidate.visible:
			owner = candidate
			button = candidate.find_child("Why" if node_name == "ItemPanel" else "FoodWhy", true, false) as Button
			break
	diagnostic["selected_inspector"] = str(owner.get_path()) if owner != null else ""
	diagnostic["selected_button"] = str(button.get_path()) if button != null else ""
	check(name + " normal click inspection entry", owner != null and button != null)
	if owner == null or button == null:
		await _shot(name + "-entry-failed", diagnostic)
		return
	await click(button)
	await frames(6)
	var snapshot: Array = causal.call("food_snapshot")
	check(name + " why has actual food observations", causal.visible and not snapshot.is_empty())
	if name == "pot-food":
		var has_pot_food: bool = false
		for row: Dictionary in snapshot:
			if "pot" in row["food"].get("storage", []): has_pot_food = true
		check("pot-food explanation includes actual pot contents", has_pot_food)
	for row: Dictionary in snapshot:
		check(name + " why matches current authoritative getter", row["food"] == world.get_tile_food(int(row["x"]), int(row["y"])))
	for factor: float in [1.0, 1.25]:
		hud.get("store").set_value("ui_scale", factor)
		await frames(8)
		check(name + " preserves original inspector", owner.visible)
		var bounds: Rect2 = Rect2(owner.global_position, owner.size * owner.scale)
		check(name + " explanation avoids inspector", not bounds.intersects(Rect2(causal.global_position, causal.size * causal.scale)))
		await _shot("%s-why-%s" % [name, factor], {"scale": factor, "food": snapshot, "inspection_path": str(owner.get_path())})
		if name == "pot-food":
			var scroll: ScrollContainer = causal.get("_scroll") as ScrollContainer
			var history: VBoxContainer = causal.get("_history_container") as VBoxContainer
			var last: Label = history.get_child(history.get_child_count() - 1) as Label if history.get_child_count() > 0 else null
			check("pot-food last explanation label exists", last != null)
			if last == null: return
			var before: int = scroll.scroll_vertical
			var needed: bool = not scroll.get_global_rect().encloses(last.get_global_rect())
			if needed:
				await wheel_into_view(scroll, last, "pot-food final explanation scale%s" % factor)
			var visible_last: bool = scroll.get_global_rect().encloses(last.get_global_rect())
			check("pot-food final explanation fully enclosed scale%s" % factor, visible_last)
			var scroll_proof: Dictionary = {"scale": factor, "wheel_required": needed, "scroll_before": before,
				"scroll_after": scroll.scroll_vertical, "viewport_rect": str(scroll.get_global_rect()),
				"last_label_rect": str(last.get_global_rect()), "last_label_text": last.text,
				"fully_enclosed": visible_last, "input": "mouse_wheel" if needed else "already_visible"}
			if not evidence.has("pot_explanation_scroll"): evidence["pot_explanation_scroll"] = []
			evidence["pot_explanation_scroll"].append(scroll_proof)
			if needed:
				await _shot("pot-food-why-bottom-%s" % factor, {"scale": factor, "food": snapshot, "scroll": scroll_proof})
	observed[name] = {"tick": _tick(), "food": snapshot}
	causal.hide()
	owner.hide()


func _rows(start: int = 0) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	var cursor: int = start
	for _page: int in 10000:
		var page: Dictionary = world.get_chronicle_since(cursor, 512)
		check("food chronicle metadata aligns with indexes", page.get("food_daily_milli", []).size() == page.get("index", []).size())
		result.append_array(ChronicleText.rows(page))
		var next: int = int(page.get("next", cursor))
		if next >= int(page.get("total", next)): return result
		if next <= cursor:
			check("food chronicle cursor advances", false)
			return result
		cursor = next
	check("food chronicle pagination bound", false)
	return result


func _plant_shots(plants: Dictionary, phase: int) -> void:
	var info: Dictionary = SimReads.read(world, "get_world_info")
	var centre := Vector2(float(info["width"]) / 2.0, float(info["height"]) / 2.0)
	var trees: Node = main.get_node("ColonyView/TreeLayer")
	var choices: Array[int] = []
	for i: int in plants.get("xs", []).size():
		if int(plants["phases"][i]) != phase: continue
		var tile := Vector2i(plants["xs"][i], plants["ys"][i])
		if tile.x < 12 or tile.y < 12 or tile.x >= int(info["width"]) - 12 or tile.y >= int(info["height"]) - 12: continue
		if bool(trees.call("under_canopy", tile)): continue
		choices.append(i)
	if choices.is_empty(): return
	var lighting: Dictionary = _plant_lighting()
	if not bool(lighting["full_daylight"]):
		if not evidence.has("plant_light_deferrals"): evidence["plant_light_deferrals"] = []
		evidence["plant_light_deferrals"].append({"phase": phase, "tick": _tick(), "lighting": lighting})
		return
	choices.sort_custom(func(a: int, b: int) -> bool:
		return Vector2(plants["xs"][a], plants["ys"][a]).distance_squared_to(centre) < Vector2(plants["xs"][b], plants["ys"][b]).distance_squared_to(centre))
	var layer: Node2D = main.get_node("ColonyView/PlantLayer") as Node2D
	for i: int in choices.slice(0, 12):
		var tile := Vector2i(plants["xs"][i], plants["ys"][i])
		camera.position = View.base_point(tile)
		camera.zoom = Vector2(2, 2)
		camera.call("_clamp_position")
		await create_timer(0.7).timeout
		await frames(5)
		var target := Rect2()
		for batch: MultiMeshInstance2D in layer.get("_batches"):
			for instance: int in batch.multimesh.instance_count:
				var transform: Transform2D = batch.multimesh.get_instance_transform_2d(instance)
				# Anchor may differ by asset; the instance centre must belong to this tile.
				if View.world_to_tile(transform.origin) != tile: continue
				target = batch.get_global_transform_with_canvas() * (transform * Rect2(-batch.texture.get_size() / 2.0, batch.texture.get_size()))
		if not target.has_area() or not root.get_visible_rect().encloses(target.grow(32)): continue
		var blocked: bool = false
		for child: Node in main.get_node("ColonyView/World").get_children():
			if child is Sprite2D and (child as Sprite2D).is_visible_in_tree():
				var sprite := child as Sprite2D
				if (sprite.get_global_transform_with_canvas() * sprite.get_rect()).intersects(target):
					blocked = true
		if blocked: continue
		var name: String = "phase%d" % phase
		var facts: Dictionary = {"kind": plants["kinds"][i], "phase": phase, "units": plants["units"][i], "tile": [tile.x, tile.y], "target_screen_rect": [target.position.x, target.position.y, target.size.x, target.size.y], "zoom": 2.0}
		for factor: float in [1.0, 1.25]:
			hud.get("store").set_value("ui_scale", factor)
			await frames(8)
			camera.position = View.base_point(tile)
			camera.zoom = Vector2(2, 2)
			camera.call("_clamp_position")
			await create_timer(0.7).timeout
			await frames(5)
			target = Rect2()
			var texture: Texture2D = null
			for batch: MultiMeshInstance2D in layer.get("_batches"):
				for instance: int in batch.multimesh.instance_count:
					var transform: Transform2D = batch.multimesh.get_instance_transform_2d(instance)
					if View.world_to_tile(transform.origin) != tile: continue
					target = batch.get_global_transform_with_canvas() * (transform * Rect2(-batch.texture.get_size() / 2.0, batch.texture.get_size()))
					texture = batch.texture
			blocked = false
			for child: Node in main.get_node("ColonyView/World").get_children():
				if child is Sprite2D and (child as Sprite2D).is_visible_in_tree():
					var sprite := child as Sprite2D
					if (sprite.get_global_transform_with_canvas() * sprite.get_rect()).intersects(target): blocked = true
			var target_ready: bool = texture != null and target.has_area() and root.get_visible_rect().encloses(target.grow(32)) and not blocked
			check("%s scale%s actual target remains visible" % [name, factor], target_ready)
			if not target_ready: return
			facts = facts.duplicate(true)
			facts["target_screen_rect"] = [target.position.x, target.position.y, target.size.x, target.size.y]
			lighting = _plant_lighting()
			facts["lighting"] = lighting
			check("%s scale%s natural full daylight" % [name, factor], bool(lighting["full_daylight"]))
			if not bool(lighting["full_daylight"]): return
			await _shot("%s-%s" % [name, factor], {"scale": factor, "facts": facts})
			var raw: Image = root.get_texture().get_image()
			var pixels: Dictionary = _plant_pixels(raw, target, texture)
			if not evidence.has("plant_pixel_checks"): evidence["plant_pixel_checks"] = []
			evidence["plant_pixel_checks"].append({"name": name, "scale": factor, "tick": _tick(), "facts": facts.duplicate(true), "pixels": pixels})
			check("%s scale%s actual crop contains plant pixels" % [name, factor], bool(pixels["present"]))
			facts["pixel_evidence"] = pixels
			if not bool(pixels["present"]): return
			var crop: Rect2i = Rect2i(target.grow(32)).intersection(Rect2i(Vector2i.ZERO, raw.get_size()))
			var path: String = out.path_join("%s-%s-crop.png" % [name, factor])
			check(name + " raw viewport crop saved", raw.get_region(crop).save_png(path) == OK)
			evidence["shots"].append({"name": name + " crop", "png": path, "sha256": FileAccess.get_sha256(path), "crop_screen_rect": [crop.position.x, crop.position.y, crop.size.x, crop.size.y], "facts": facts, "scale": factor})
		observed[name] = {"tick": _tick(), "facts": facts, "visual_verdict": "PENDING"}
		return


func _chronicle_shots(name: String, row: Dictionary) -> void:
	var panel: Control = main.get_node("UI/ChroniclePanel") as Control
	var body: RichTextLabel = panel.get("_body") as RichTextLabel
	var markup := RegEx.new()
	markup.compile("\\[[^\\]]*\\]")
	var expected: String = markup.sub(ChronicleText.sentence(row, -1, root.get_node("Locale"), SimReads.read(world, "get_world_info")), "", true)
	var valid: bool = true
	for factor: float in [1.0, 1.25]:
		hud.get("store").set_value("ui_scale", factor)
		await frames(5)
		if panel.visible: panel.hide()
		for down: bool in [true, false]:
			var event := InputEventKey.new()
			event.keycode = KEY_C
			event.pressed = down
			Input.parse_input_event(event)
			await frames(2)
		await frames(5)
		var actual: String = panel.call("plain_text")
		var matching_row: Dictionary = {}
		for entry: Dictionary in panel.get("_entries"):
			if int(entry.get("index", -1)) == int(row["index"]): matching_row = entry
		var found: int = actual.find(expected) if not expected.is_empty() else -1
		var present: bool = panel.visible and body.is_visible_in_tree() and not matching_row.is_empty() and found >= 0
		check(name + " C opens actual event text", present)
		valid = valid and present
		var paragraph: int = actual.substr(0, maxi(found, 0)).count("\n")
		if present:
			body.scroll_to_paragraph(paragraph)
			await frames(5)
		var diagnostics: Dictionary = {"scale": factor, "event": row, "selected_row": matching_row, "expected_sentence": expected, "rendered_text": actual, "text_found_at": found, "paragraph": paragraph, "body_rect": str(body.get_global_rect()), "scroll": body.get_v_scroll_bar().value, "visible_lines": body.get_visible_line_count(), "cursor": panel.get("_next")}
		check(name + " event body has visible lines", present and body.get_visible_line_count() > 0)
		await _shot("%s-%s" % [name, factor], diagnostics)
	panel.hide()
	if valid: observed[name] = {"tick": _tick(), "event": row, "expected_sentence": expected}


# Compare opaque source texels with the raw rendered viewport at their projected
# centres. Geometry alone cannot accept a crop containing only terrain.
func _plant_pixels(raw: Image, target: Rect2, texture: Texture2D) -> Dictionary:
	var source: Image = texture.get_image()
	var daylight: CanvasModulate = main.get_node("DaylightController/CanvasModulate") as CanvasModulate
	var tint: Color = daylight.color
	var tested: int = 0
	var matched: int = 0
	for y: int in source.get_height():
		for x: int in source.get_width():
			var expected: Color = source.get_pixel(x, y)
			if expected.a < 0.99: continue
			# Exclude the dark outline: crown/stem colour must also be present.
			if maxf(expected.r, maxf(expected.g, expected.b)) < 0.2: continue
			expected *= tint
			var point: Vector2i = Vector2i(target.position + Vector2((float(x) + 0.5) / source.get_width(), (float(y) + 0.5) / source.get_height()) * target.size)
			if not Rect2i(Vector2i.ZERO, raw.get_size()).has_point(point): continue
			var actual: Color = raw.get_pixelv(point)
			tested += 1
			if maxf(absf(actual.r - expected.r), maxf(absf(actual.g - expected.g), absf(actual.b - expected.b))) <= 0.06: matched += 1
	return {"tested": tested, "matched": matched, "canvas_modulate": [tint.r, tint.g, tint.b, tint.a], "tolerance": 0.06, "minimum_fraction": 0.8, "present": tested >= 4 and matched * 5 >= tested * 4}


func _plant_lighting() -> Dictionary:
	var controller: Node = main.get_node("DaylightController")
	var modulation: CanvasModulate = controller.get_node("CanvasModulate") as CanvasModulate
	var tint: Color = modulation.color
	var lights: Array = controller.call("lights")
	var active_energy: bool = false
	for light: Dictionary in lights:
		if bool(light["enabled"]) and float(light["energy"]) > 0.0:
			active_energy = true
	return {"canvas_modulate": [tint.r, tint.g, tint.b, tint.a], "lights": lights,
		"full_daylight": tint == Color.WHITE and not active_energy,
		"requirement": "Natural full daylight: white CanvasModulate and no enabled light with positive energy"}


func _wanted(case_name: String) -> bool:
	return case_name in required_cases
