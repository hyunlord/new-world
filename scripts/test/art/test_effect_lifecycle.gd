extends SceneTree

const Effects = preload("res://scripts/ui/world_effects.gd")
const Structures = preload("res://scripts/ui/building_renderer.gd")

func _initialize() -> void:
	call_deferred("_run")

func _run() -> void:
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var world: Node = main.get_node("WorldSim")
	world.call("set_sim_speed",0.0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(main)
	await process_frame
	var effects: Node = main.get_node("WorldEffects")
	effects.set_process(false)
	var transient := Sprite2D.new()
	var persistent := Sprite2D.new()
	main.get_node("WorldObjects").add_child(transient)
	main.get_node("WorldObjects").add_child(persistent)
	effects._effects.append({"sprite":transient})
	effects._persistent["test"] = persistent
	effects._elapsed = 999.0
	effects._next_birds = 998.0
	effects._next_fish = 997.0
	effects._next_insect = 996.0
	effects._reset()
	assert(not is_instance_valid(transient) and not is_instance_valid(persistent))
	assert(effects._effects.is_empty() and effects._persistent.is_empty())
	assert(effects._elapsed == 0.0 and effects._next_birds == 15.0 and effects._next_fish == 25.0 and effects._next_insect == 8.0)
	var first: int = effects._rng.randi()
	effects._reset()
	assert(effects._rng.randi() == first)
	var agents: Node = main.get_node("AgentRenderer")
	agents.call("refresh_snapshot")
	var retired := Sprite2D.new()
	retired.set_meta("art_kind","agent")
	retired.set_meta("art_id",999999)
	retired.set_meta("activity","drink")
	main.get_node("WorldObjects").add_child(retired)
	effects._process(0.0)
	assert(not effects._previous.has(999999))
	var active: Array = agents.call("get_active_agent_sprites")
	assert(not active.is_empty())
	var actor: Sprite2D = active[0]
	agents.call("set_capture_visible",false)
	actor.set_meta("activity","asleep")
	actor.set_meta("sheltered_sleep",false)
	var frames: Array[String] = ["direction_01_campfire"]
	effects._spawn(frames,actor.position,1.0,Vector2.ZERO,true)
	assert(not effects._effects.is_empty())
	effects._process(0.0)
	for effect: Dictionary in effects._effects:
		assert(not bool(effect.get("agent_owned",false)))
	assert(not effects._persistent.has("bed_%d" % int(actor.get_meta("art_id"))))
	assert(not bool(actor.get_meta("sheltered_sleep")))
	var daylight: Node = main.get_node("DaylightController")
	assert(daylight._sheltered_sleep_tiles().is_empty())
	agents.call("set_capture_visible",true)
	actor.set_meta("sheltered_sleep",true)
	actor.set_meta("tile_x",10)
	actor.set_meta("tile_y",10)
	retired.set_meta("sheltered_sleep",true)
	retired.set_meta("tile_x",999)
	retired.set_meta("tile_y",999)
	var sleeper_tiles: Dictionary = daylight._sheltered_sleep_tiles()
	assert(sleeper_tiles.has(Vector2i(10,10)))
	assert(not sleeper_tiles.has(Vector2i(999,999)))
	var host := Vector2i(10,10)
	var occupied: Dictionary = {host:true,Vector2i(11,10):true,Vector2i(10,11):true}
	assert(Structures.prop_footprint_allowed(host,Vector2.ZERO,8,4,occupied))
	assert(not Structures.prop_footprint_allowed(host,Vector2(16,0),8,4,occupied))
	assert(not Structures.prop_footprint_allowed(host,Vector2(0,9),8,4,occupied))
	assert(Structures.prop_footprint_allowed(host,Vector2(-16,0),8,4,occupied))
	main.queue_free()
	await process_frame
	print("ART EFFECT LIFECYCLE PASS: reset frees nodes, seeded replay, active-agent API, capture suppression, occupied-neighbor footprints")
	quit(0)
