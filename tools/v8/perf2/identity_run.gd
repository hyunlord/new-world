# identity_run.gd — PERF2-1 A identity proof (tools only, never in the game).
# Purpose: load `--load <wsave>` into a bare WorldSimNode (the library .gdextension macos.debug names), advance
#   `--days N` whole days of ticks (get_world_info ticks_per_day, else 1440) and save after each day; prints one
#   `PERF2_IDENTITY day <d> tick <t> sha256 <hex>` line per day and writes `--out <json>` with the library's sha256,
#   the per-day save hashes and the end-save hash. Run twice (game and --dev library) and compare the JSON rows.
# Bridge methods called: load_world, advance_ticks, save_world, get_time_snapshot, get_world_info (direct).
extends SceneTree


func _init() -> void:
	call_deferred("_run")


func _arg(name: String, fallback: String) -> String:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var i: int = args.find(name)
	return args[i + 1] if i >= 0 and i + 1 < args.size() else fallback


func _run() -> void:
	var scene: String = _arg("--load", "")
	var days: int = int(_arg("--days", "30"))
	var out: String = _arg("--out", "")
	var work: String = _arg("--work", OS.get_user_data_dir().path_join("perf2_identity"))
	DirAccess.make_dir_recursive_absolute(work)
	var config: ConfigFile = ConfigFile.new()
	config.load("res://sim_bridge.gdextension")
	var library: String = str(config.get_value("libraries", "macos.debug", ""))
	var world: Object = ClassDB.instantiate("WorldSimNode")
	if world == null or scene.is_empty() or not bool(world.call("load_world", scene)):
		push_error("PERF2_IDENTITY cannot load " + scene)
		quit(1)
		return
	var info: Dictionary = world.call("get_world_info")
	var per_day: int = int(info.get("ticks_per_day", 1440))
	var rows: Array = []
	var started: int = Time.get_ticks_msec()
	for day: int in range(1, days + 1):
		world.call("advance_ticks", per_day)
		var path: String = work.path_join("day.wsave")
		if not bool(world.call("save_world", path)):
			push_error("PERF2_IDENTITY save failed")
			quit(1)
			return
		var tick: int = int((world.call("get_time_snapshot") as Dictionary).get("tick", -1))
		var sha: String = FileAccess.get_sha256(path)
		rows.append({"day": day, "tick": tick, "sha256": sha})
		print("PERF2_IDENTITY day %d tick %d sha256 %s" % [day, tick, sha])
	var result: Dictionary = {"scene": scene, "days": days, "ticks_per_day": per_day, "library": library,
		"library_sha256": FileAccess.get_sha256(library), "wall_s": float(Time.get_ticks_msec() - started) / 1000.0,
		"days_rows": rows, "end_save_sha256": rows[-1]["sha256"] if not rows.is_empty() else ""}
	if not out.is_empty():
		var file: FileAccess = FileAccess.open(out, FileAccess.WRITE)
		file.store_string(JSON.stringify(result, "  "))
		file.close()
	print("PERF2_IDENTITY_DONE " + JSON.stringify({"library": library, "end": result["end_save_sha256"], "wall_s": result["wall_s"]}))
	(world as Node).free()
	quit(0)
