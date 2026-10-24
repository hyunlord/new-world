# mart21_shots.gd — the M-ART2.1 screens (goal §8): the compare set and the seasons strip, windowed.
# Purpose: `--set <dir with mart2_set.json> --out <dir>` loads every shot's world save written by tools/v8/mart2_shots.gd
#   (the exact moment M-ART2 and the old renderer drew), stops the clock, frames the same tile at the same zoom as
#   mart2_shots.gd `_frame_at`, waits for the ground (¼× retextures four chunks a frame) and saves `<name>.png` with
#   this renderer, plus mart21_set.json {name, tick, tile, zoom, season, blend}. `--seasons <save> --tile x,y --out
#   <dir>` loads one save and advances the clock (advance_ticks; saves are the probe's colony seed 0) to noon on day 3
#   of each season in calendar order, saving `season_<n>_<name>_1x.png` and `_quarter.png` (the tile at 1× and the
#   map centre at ¼×) and seasons.json. `--frames N` overrides the settle frames. Run `--set` and `--seasons` as two
#   processes: after the set's last save (the migrant dialog) the seasons' advance stalled past a 20-minute bound,
#   while the seasons alone take ~3 minutes (MART21_DECISIONS D6). Prints MART21_SHOTS_DONE n=<shots>.
#   `Godot --path . --rendering-driver opengl3 --resolution 1920x1080 --script res://tools/v8/mart21_shots.gd -- ...`
# Bridge methods called: load_world, advance_ticks (direct); via SimReads get_time_snapshot, get_world_info.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")

var _main: Node = null
var _world: Node = null
var _camera: Camera2D = null
var _settle: int = 90


func _init() -> void:
	call_deferred("_run")


func _arg(name: String) -> String:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var i: int = args.find(name)
	return args[i + 1] if i >= 0 and i + 1 < args.size() else ""


func _frames(n: int) -> void:
	for _i: int in n:
		await process_frame


func _time() -> Dictionary:
	return SimReads.read(_world, "get_time_snapshot")


func _frame_at(tile: Vector2, zoom: float) -> void:
	_camera.zoom = Vector2(zoom, zoom)
	_camera.position = tile * View.TILE + Vector2(View.TILE, View.TILE) * 0.5
	_camera.call("_clamp_position")
	await _frames(_settle)
	await RenderingServer.frame_post_draw


func _save(path: String) -> void:
	root.get_viewport().get_texture().get_image().save_png(path)


func _season_note() -> Dictionary:
	var trees: Node = _main.get_node("ColonyView/TreeLayer")
	return {"season": ",".join(trees.call("season_pair")), "blend": float(trees.call("season_blend"))}


func _run() -> void:
	var out: String = _arg("--out")
	if out == "":
		push_error("mart21_shots: --out <dir> is required")
		quit(2)
		return
	DirAccess.make_dir_recursive_absolute(out)
	if _arg("--frames") != "":
		_settle = int(_arg("--frames"))
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	_world = _main.get_node("WorldSim")
	_camera = _main.get_node("Camera2D")
	await _frames(10)
	var n: int = 0
	if _arg("--set") != "":
		var set_json: Variant = JSON.parse_string(FileAccess.get_file_as_string(_arg("--set").path_join("mart2_set.json")))
		if not set_json is Dictionary:
			push_error("mart21_shots: no mart2_set.json in %s" % _arg("--set"))
			quit(1)
			return
		var notes: Array = []
		for shot: Variant in (set_json as Dictionary).get("shots", []):
			var s: Dictionary = shot
			_world.call("load_world", str(s["save"]))
			_world.process_mode = Node.PROCESS_MODE_DISABLED
			await _frames(30)
			var tile: Array = s["tile"]
			await _frame_at(Vector2(float(tile[0]), float(tile[1])), float(s["zoom"]))
			_save(out.path_join("%s.png" % str(s["name"])))
			var note: Dictionary = {"name": s["name"], "tick": int(_time().get("tick", 0)), "tile": tile, "zoom": s["zoom"]}
			note.merge(_season_note())
			notes.append(note)
			print("MART21_SHOT %s" % JSON.stringify(note))
			n += 1
		var f: FileAccess = FileAccess.open(out.path_join("mart21_set.json"), FileAccess.WRITE)
		f.store_string(JSON.stringify({"source": _arg("--set"), "shots": notes}, " "))
		f.close()
	if _arg("--seasons") != "":
		_world.call("load_world", _arg("--seasons"))
		_world.process_mode = Node.PROCESS_MODE_DISABLED
		await _frames(30)
		var parts: PackedStringArray = _arg("--tile").split(",")
		var tile: Vector2 = Vector2(float(parts[0]), float(parts[1])) if parts.size() == 2 else Vector2(96, 96)
		var info: Dictionary = SimReads.read(_world, "get_world_info")
		var centre: Vector2 = Vector2(float(info.get("width", 192)) * 0.5, float(info.get("height", 192)) * 0.5)
		var notes: Array = []
		for k: int in 4:
			var t: Dictionary = _time()
			var per: int = int(t.get("days_per_season", 0))
			var tpd: int = int(t.get("ticks_per_day", 1440))
			if per <= 0:
				break
			# The next noon on day 3 of a season, one season after the previous shot.
			var day: int = int(t.get("day_index", 0))
			var target_day: int = (floori(float(day) / per) + (1 if k > 0 or day % per >= 2 else 0)) * per + 2
			var now: float = float(day) * tpd + float(t.get("time_of_day", 0.0)) / 24.0 * tpd
			var ticks: int = maxi(0, roundi(float(target_day) * tpd + 0.5 * tpd - now))
			_world.call("advance_ticks", ticks)
			await _frames(5)
			var name: String = str(_season_note()["season"]).get_slice(",", 1)
			await _frame_at(tile, 1.0)
			_save(out.path_join("season_%d_%s_1x.png" % [k, name]))
			await _frame_at(centre, 0.25)
			_save(out.path_join("season_%d_%s_quarter.png" % [k, name]))
			var note: Dictionary = {"index": k, "tick": int(_time().get("tick", 0)), "day": int(_time().get("day_index", 0)), "tile": [tile.x, tile.y]}
			note.merge(_season_note())
			notes.append(note)
			print("MART21_SEASON %s" % JSON.stringify(note))
			n += 2
		var f2: FileAccess = FileAccess.open(out.path_join("seasons.json"), FileAccess.WRITE)
		f2.store_string(JSON.stringify({"save": _arg("--seasons"), "shots": notes}, " "))
		f2.close()
	print("MART21_SHOTS_DONE n=%d" % n)
	quit(0)
