# tools/v8/save_load_check.gd — headless check of the M1.5 quick save / load (F5 / F9).
# Purpose: boot res://scenes/main.tscn, advance the world, call the same functions the F5 and F9 keys call
#   (OrderBar.quick_save / quick_load), and check through the bridge reads that the load restored the saved tick
#   and every agent's id, tile and state, and that two loads of one save tick on identically. Deletes its save
#   file at the end. Prints "SAVE_LOAD_CHECK PASS ..." or "SAVE_LOAD_CHECK FAIL <reason>" and quits 0 / 1.
# Bridge methods called: via SimReads advance_ticks, get_time_snapshot, get_agent_snapshot; through OrderBar
#   save_world, load_world.
# Run: "$GODOT_BIN" --headless --path . --script res://tools/v8/save_load_check.gd
#   Windowed with "-- --shots <abs dir>" it also saves the screen right after the save and after the first load
#   (save_toast.png, load_toast.png: the F5 / F9 toasts; M1.5 goal 10.3).
extends SceneTree

var _shots: String = ""


func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size():
		if args[i] == "--shots" and i + 1 < args.size():
			_shots = args[i + 1]
	call_deferred("_run")


## Save the screen to <shots>/<name> after a few frames (windowed runs only).
func _shot(name: String) -> void:
	if _shots == "":
		return
	DirAccess.make_dir_recursive_absolute(_shots)
	for _i: int in 6:
		await process_frame
	var image: Image = root.get_texture().get_image()
	image.save_png(_shots.path_join(name))


func _fail(reason: String) -> void:
	print("SAVE_LOAD_CHECK FAIL %s" % reason)
	quit(1)


func _tick(world: Node) -> int:
	return int(SimReads.read(world, "get_time_snapshot").get("tick", -1))


## Ids, tiles and states of every agent, as text.
func _agents(world: Node) -> String:
	var snap: Dictionary = SimReads.read(world, "get_agent_snapshot")
	return "%s|%s|%s|%s" % [str(snap.get("agent_ids", [])), str(snap.get("xs", [])), str(snap.get("ys", [])), str(snap.get("states", []))]


func _run() -> void:
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	await process_frame
	var world: Node = main.get_node_or_null("WorldSim")
	var bar: Node = main.get_node_or_null("UI/OrderBar")
	if world == null or bar == null:
		_fail("missing WorldSim or UI/OrderBar")
		return
	# Pause the frame-driven clock as the P key does (a load's world_reset re-applies the speed, not the pause), so
	# only advance_ticks moves the world.
	world.process_mode = Node.PROCESS_MODE_DISABLED
	SimReads.mutate(world, "advance_ticks", [700])
	await process_frame
	var saved_tick: int = _tick(world)
	var saved_agents: String = _agents(world)
	var path: String = str(bar.call("quick_save"))
	if path == "" or not FileAccess.file_exists(path):
		_fail("quick_save wrote no file (%s)" % path)
		return
	await _shot("save_toast.png")
	SimReads.mutate(world, "advance_ticks", [400])
	await process_frame
	if _tick(world) != saved_tick + 400 or _agents(world) == saved_agents:
		_fail("the world did not move after the save")
		return
	if not bool(bar.call("quick_load")):
		_fail("quick_load returned false")
		return
	await process_frame
	await process_frame
	await _shot("load_toast.png")
	var loaded_tick: int = _tick(world)
	var loaded_agents: String = _agents(world)
	if loaded_tick != saved_tick:
		_fail("tick after load %d, saved %d" % [loaded_tick, saved_tick])
		return
	if loaded_agents != saved_agents:
		_fail("agents after load differ from the save")
		return
	SimReads.mutate(world, "advance_ticks", [300])
	await process_frame
	var first_run: String = _agents(world)
	if not bool(bar.call("quick_load")):
		_fail("second quick_load returned false")
		return
	await process_frame
	SimReads.mutate(world, "advance_ticks", [300])
	await process_frame
	var second_run: String = _agents(world)
	if first_run != second_run:
		_fail("two loads of one save ticked on differently")
		return
	DirAccess.remove_absolute(ProjectSettings.globalize_path(path))
	var count: int = len(SimReads.read(world, "get_agent_snapshot").get("agent_ids", []))
	print("SAVE_LOAD_CHECK PASS tick=%d file=%s agents=%d" % [saved_tick, path.get_file(), count])
	quit(0)
