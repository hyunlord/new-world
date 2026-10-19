extends SceneTree

const KINDS: Array[String] = ["memory_recalled", "combat_started"]
var failures: int = 0

class ObservedRenderer:
	extends "res://scripts/ui/agent_renderer.gd"
	var sequences: Dictionary = {"memory_recalled": [], "combat_started": []}
	var batches: int = 0

	func _process(_delta: float) -> void:
		pass

	func _ingest_memory_recalls(events: Array) -> void:
		batches += 1
		_record(events, "memory_recalled", _seen_recall_event_ids)
		super._ingest_memory_recalls(events)

	func _ingest_combat_events(events: Array) -> void:
		_record(events, "combat_started", _seen_combat_event_ids)
		super._ingest_combat_events(events)

	func _record(events: Array, kind: String, seen: Dictionary) -> void:
		var known: Dictionary = seen.duplicate()
		for event: Dictionary in events:
			var id: int = int(event.get("id", -1))
			if str(event.get("kind", "")) == kind and id >= 0 and not known.has(id):
				known[id] = true
				sequences[kind].append([_last_causal_tick, id])

func _initialize() -> void:
	call_deferred("run")

func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)

# Deliberately retained old per-tile path, whitelisted by the P9 static gate.
func _old_events(world: Node, snap: Dictionary) -> Array:
	var events: Array = []
	var xs: PackedInt32Array = snap["xs"]
	var ys: PackedInt32Array = snap["ys"]
	var visited: Dictionary = {}
	for i: int in range(xs.size()):
		var key: int = (int(xs[i]) << 20) | int(ys[i])
		if visited.has(key):
			continue
		visited[key] = true
		events.append_array(world.get_tile_causal_history(int(xs[i]), int(ys[i])))
	return events

func _old_ingest(events: Array, tick: int, state: Dictionary) -> void:
	for kind: String in KINDS:
		var seen: Dictionary = state["seen"][kind]
		for event: Dictionary in events:
			if str(event.get("kind", "")) != kind:
				continue
			var id: int = int(event.get("id", -1))
			if id < 0 or seen.has(id):
				continue
			seen[id] = true
			state["sequences"][kind].append([tick, id])
		if seen.size() > 512:
			seen.clear()

func _fixture(renderer: ObservedRenderer, snap: Dictionary) -> void:
	renderer._on_world_reset()
	check(renderer._last_causal_tick == -1 and not renderer._causal_rearm, "reset clears tick marker and rearm")
	var events: Array = []
	for id: int in range(513):
		events.append({"id": id, "kind": "memory_recalled", "agent_id": 17})
		events.append({"id": 1000 + id, "kind": "combat_started", "agent_id": 18})
	renderer._last_causal_tick = 123
	renderer.sequences = {"memory_recalled": [], "combat_started": []}
	var old: Dictionary = {"seen": {"memory_recalled": {}, "combat_started": {}}, "sequences": {"memory_recalled": [], "combat_started": []}}
	for scan: int in range(2):
		_old_ingest(events, 123, old)
		renderer._ingest_memory_recalls(events)
		renderer._ingest_combat_events(events)
		for kind: String in KINDS:
			check(renderer.sequences[kind] == old["sequences"][kind], "overflow fixture re-ingests retained events in old order: %s" % kind)
	check(renderer._seen_recall_event_ids.is_empty() and renderer._seen_combat_event_ids.is_empty(), "both seen sets clear above 512")
	check(renderer._causal_rearm, "overflow requests next-frame scan")
	check(renderer.recall_cue_remaining(17) == 36 and renderer.combat_cue_remaining(18) == 36, "both cue kinds ingest")
	var before: int = renderer.batches
	renderer._ingest_causal_events(123, PackedInt32Array(), PackedInt32Array())
	check(renderer.batches == before + 1, "overflow rescans unchanged tick next frame")
	before = renderer.batches
	renderer._ingest_causal_events(123, PackedInt32Array(), PackedInt32Array())
	check(renderer.batches == before, "ordinary unchanged tick skips batch")
	for frame: int in range(36):
		renderer._clear_expired_recalling()
		renderer._clear_expired_combating()
	check(renderer.recall_cue_remaining(17) == 0 and renderer.combat_cue_remaining(18) == 0, "cue timers expire on frames without ticks")
	renderer._on_world_reset()
	before = renderer.batches
	renderer._ingest_causal_events(123, snap["xs"], snap["ys"])
	check(renderer.batches == before + 1, "reset rescans even if tick is unchanged")

func run() -> void:
	var main: Node = load("res://scenes/main.tscn").instantiate()
	root.add_child(main)
	main.process_mode = Node.PROCESS_MODE_DISABLED
	var world: Node = main.get_node("WorldSim")
	check(bool(SimReads.mutate(world, "reset_world_stress", [15100, 500])), "W500 stress reset")
	await process_frame
	await process_frame
	var renderer := ObservedRenderer.new()
	renderer.world_sim = world
	var old: Dictionary = {"seen": {"memory_recalled": {}, "combat_started": {}}, "sequences": {"memory_recalled": [], "combat_started": []}}
	check(int(SimReads.mutate(world, "advance_ticks", [1440])) == 1440, "W500 setup advances 1440 ticks")
	for step: int in range(600):
		check(int(SimReads.mutate(world, "advance_ticks", [1])) == 1441 + step, "explicit single tick advance")
		var snap: Dictionary = SimReads.read(world, "get_agent_snapshot")
		var tick: int = int(SimReads.read(world, "get_time_snapshot")["tick"])
		_old_ingest(_old_events(world, snap), tick, old)
		renderer._clear_expired_recalling()
		renderer._clear_expired_combating()
		renderer._ingest_causal_events(tick, snap["xs"], snap["ys"])
		for kind: String in KINDS:
			check(renderer.sequences[kind] == old["sequences"][kind], "per-kind ingestion sequence matches at tick %d: %s" % [tick, kind])
		await process_frame
	print("CAUSAL_BATCH_P9 ticks=600 recalls=", renderer.sequences["memory_recalled"].size(), " combats=", renderer.sequences["combat_started"].size())
	_fixture(renderer, SimReads.read(world, "get_agent_snapshot"))
	renderer.free()
	main.queue_free()
	await process_frame
	print("CAUSAL_BATCH_P9 failures=", failures)
	quit(1 if failures else 0)
