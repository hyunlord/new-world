# tools/v8/renderer_agreement.gd — the Godot half of the renderer agreement check (M1.5 goal 8.1, contract
#   clause C2 "view independence"; run by tools/v8/renderer_agreement.sh, future gate step renderer_agreement).
# Purpose: load the real res://scenes/main.tscn, pause the simulation (WorldSim process off), rebuild the world of
#   --seed, replay a concrete command log (the colony probe's --commands-out file: each entry applied when the
#   world has completed exactly its tick, as the probe applies it right before that tick's systems), advance to
#   --tick, then let the 3D People layer (scripts/world3d/people3d.gd, M3D-SWITCH E2) draw with the clock stopped
#   (sim speed 0, WorldSim processing again so the figures' steps play out) until every figure has finished its step,
#   and write the tile, look and face variant of every person it draws to --out as JSON. Each tile is the drawn figure's
#   feet (World3D/People `state_of(bits)` `feet`, one 3D unit per tile) floored to a tile and must equal the figure's
#   own tile (`tile`); look and variant are `look_id` / `variant` of what is drawn. Every drawn figure is listed
#   (`bodies(0)`, `count()`), never a fresh bridge read of positions; the AgentId of each entity comes from the agent
#   snapshot's identity columns only. Tooling only: the game never runs this script.
# Bridge methods called: via SimReads.mutate_raw reset_world, cmd_set_steward, advance_ticks, cmd_designate_chop_tiles,
#   cmd_designate_chop, cmd_cancel_area, cmd_add_stockpile, cmd_remove_zone_area, cmd_place_blueprint; directly
#   get_time_snapshot (to check the reached tick), get_steward (the probe fixture is OFF), set_sim_speed (0 while the
#   figures settle), get_agent_snapshot (entity bits -> AgentId).
# usage: Godot --headless --path . --script res://tools/v8/renderer_agreement.gd -- \
#          --out <abs json> --commands <abs commands json> [--seed 100] [--tick 900]
# Output: the JSON file and one line "RENDERER_AGREEMENT_GODOT_DONE tick=T people=N settled=true|false";
#   on any failure push_error(...) and exit 1.
extends SceneTree

const SETTLE_FRAMES_MAX: int = 6000

var _out: String = ""
var _commands: String = ""
var _seed: int = 100
var _tick: int = 900


func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size():
		if i + 1 >= args.size():
			continue
		match args[i]:
			"--out":
				_out = args[i + 1]
			"--commands":
				_commands = args[i + 1]
			"--seed":
				_seed = int(args[i + 1])
			"--tick":
				_tick = int(args[i + 1])
	call_deferred("_run")


func _fail(message: String) -> void:
	push_error("renderer_agreement: " + message)
	quit(1)


func _run() -> void:
	if _out.is_empty() or _commands.is_empty():
		_fail("--out and --commands are required")
		return
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(_commands))
	if not parsed is Array:
		_fail("cannot read the command log " + _commands)
		return
	var log_entries: Array = parsed
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	await process_frame
	var world: Node = main.get_node_or_null("WorldSim")
	var reads: Node = main.get_node_or_null("SimReads")
	var people: Node = main.get_node_or_null("World3D/People")
	if world == null or reads == null or people == null:
		_fail("main.tscn lacks WorldSim, SimReads or World3D/People")
		return
	world.set("process_mode", Node.PROCESS_MODE_DISABLED)
	reads.call("mutate_raw", "reset_world", [_seed])
	var steward_command: Dictionary = reads.call("mutate_raw", "cmd_set_steward", [false, 0])
	for _i: int in 10:
		await process_frame
	if _now(world) != 0:
		_fail("reset_world(%d) left the clock at tick %d" % [_seed, _now(world)])
		return
	var steward: Dictionary = world.call("get_steward")
	if not bool(steward_command.get("accepted", false)) or bool(steward.get("enabled", true)):
		_fail("the probe's steward-OFF fixture was not established before replay")
		return
	print("RENDERER_AGREEMENT_STEWARD_OFF accepted=true tick=0")
	for entry: Variant in log_entries:
		if not entry is Dictionary:
			_fail("a command log entry is not an object")
			return
		var e: Dictionary = entry
		var at: int = int(e.get("tick", -1))
		if at < _now(world) or at >= _tick:
			_fail("command at tick %d is outside [%d, %d)" % [at, _now(world), _tick])
			return
		_advance_to(reads, at)
		var result: Variant = _apply(reads, e)
		if not result is Dictionary:
			_fail("unsupported command %s" % str(e.get("cmd", "?")))
			return
		var want: Dictionary = e.get("result", {})
		if want.has("accepted") and bool((result as Dictionary).get("accepted", false)) != bool(want["accepted"]):
			_fail("command %s at tick %d: accepted %s, the probe's log says %s" % [str(e.get("cmd", "?")), at,
				str((result as Dictionary).get("accepted")), str(want["accepted"])])
			return
	_advance_to(reads, _tick)
	if _now(world) != _tick:
		_fail("reached tick %d, wanted %d" % [_now(world), _tick])
		return
	# The clock stays at --tick (speed 0) while WorldSim processes again: People only advances its figures' steps while
	# the world is not paused, as in the game.
	world.call("set_sim_speed", 0.0)
	world.set("process_mode", Node.PROCESS_MODE_INHERIT)
	var settled: bool = false
	for frame: int in SETTLE_FRAMES_MAX:
		await process_frame
		if frame >= 10 and _all_settled(people):
			settled = true
			break
	world.set("process_mode", Node.PROCESS_MODE_DISABLED)
	if _now(world) != _tick:
		_fail("the clock moved to tick %d while the figures settled" % _now(world))
		return
	var snap: Dictionary = world.call("get_agent_snapshot")
	var snap_ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var snap_aids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var tiles: Array = []
	var detail: Array = []
	var bodies: Array = people.call("bodies", 0)
	if bodies.size() != int(people.call("count")):
		_fail("People lists %d bodies but draws %d figures" % [bodies.size(), int(people.call("count"))])
		return
	for body: Dictionary in bodies:
		var bits: int = int(body["id"])
		var state: Dictionary = people.call("state_of", bits)
		var feet: Vector3 = state.get("feet", Vector3.INF)
		var tile: Vector2i = Vector2i(floori(feet.x), floori(feet.z))
		var own: Vector2i = state.get("tile", Vector2i(-1, -1))
		var row: int = snap_ids.find(bits)
		tiles.append([tile.x, tile.y])
		detail.append({"agent_id": snap_aids[row] if row >= 0 and row < snap_aids.size() else -1, "bits": bits,
			"tile": [tile.x, tile.y], "layer_tile": [own.x, own.y], "feet": [feet.x, feet.y, feet.z],
			"settled": bool(state.get("settled", false)), "clip": str(state.get("clip", "")), "model": str(state.get("model", "")),
			"look": int(state.get("look_id", -1)), "variant": int(state.get("variant", -1))})
	tiles.sort()
	detail.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return int(a["agent_id"]) < int(b["agent_id"]))
	var out: Dictionary = {"seed": _seed, "tick": _now(world), "count": tiles.size(), "people": tiles,
		"settled": settled, "detail": detail}
	var file: FileAccess = FileAccess.open(_out, FileAccess.WRITE)
	if file == null:
		_fail("cannot write " + _out)
		return
	file.store_string(JSON.stringify(out, "  ", false))
	file.close()
	print("RENDERER_AGREEMENT_GODOT_DONE tick=%d people=%d settled=%s" % [_now(world), tiles.size(), str(settled)])
	quit(0)


func _now(world: Node) -> int:
	return int((world.call("get_time_snapshot") as Dictionary).get("tick", -1))


func _advance_to(reads: Node, tick: int) -> void:
	var world: Node = reads.get_node_or_null("../WorldSim")
	var now: int = _now(world)
	if tick > now:
		reads.call("mutate_raw", "advance_ticks", [tick - now])


## True when People draws someone and no figure is still stepping between two tiles.
func _all_settled(people: Node) -> bool:
	var bodies: Array = people.call("bodies", 0)
	if bodies.is_empty():
		return false
	for body: Dictionary in bodies:
		if not bool((people.call("state_of", int(body["id"])) as Dictionary).get("settled", false)):
			return false
	return true


## Send one concrete logged command through the bridge's order methods; null when unsupported.
func _apply(reads: Node, e: Dictionary) -> Variant:
	var cmd: String = str(e.get("cmd", ""))
	match cmd:
		"designate_chop_tiles":
			var xs: PackedInt32Array = PackedInt32Array()
			var ys: PackedInt32Array = PackedInt32Array()
			for t: Variant in e.get("tiles", []):
				var pair: Array = t
				xs.append(int(pair[0]))
				ys.append(int(pair[1]))
			return reads.call("mutate_raw", "cmd_designate_chop_tiles", [xs, ys])
		"designate_chop", "cancel_area", "add_stockpile", "remove_zone_area":
			return reads.call("mutate_raw", "cmd_" + cmd, [int(e.get("x0", -1)), int(e.get("y0", -1)),
				int(e.get("x1", -1)), int(e.get("y1", -1))])
		"place_blueprint":
			return reads.call("mutate_raw", "cmd_place_blueprint", [str(e.get("kind", "hut")), int(e.get("x", -1)),
				int(e.get("y", -1))])
	return null
