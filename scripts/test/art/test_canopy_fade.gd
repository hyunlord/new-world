extends SceneTree
const CANOPY = preload("res://scripts/ui/nature_canopy.gd")
class CrossingDriver extends Node:
	var renderer: Node
	var agent_id: int
	var stage: int = 0
	func _process(_delta: float) -> void:
		stage += 1
		var feet: Vector2 = Vector2(-1000,-1010) if stage == 1 else Vector2(-950,-1010)
		var previous: Dictionary = renderer.get("_prev_positions")
		var current: Dictionary = renderer.get("_curr_positions")
		previous[agent_id] = feet
		current[agent_id] = feet
		# Seed the draw integrator, not the sprite: AgentRenderer must still publish
		# these feet before NatureRenderer reads them in this same scene frame.
		var drawn: Dictionary = renderer.get("_drawn_positions")
		drawn[agent_id] = feet

class CrossingObserver extends Node:
	signal checked(passed: bool)
	var driver: CrossingDriver
	var tree: Sprite2D
	var agent: Sprite2D
	func _process(_delta: float) -> void:
		var expected: float = 0.45 if driver.stage == 1 else 1.0
		var expected_feet: Vector2 = Vector2(-1000,-1010) if driver.stage == 1 else Vector2(-950,-1010)
		checked.emit(is_equal_approx(tree.modulate.a,expected) and agent.position == expected_feet)

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	var objects: Node2D = Node2D.new()
	var image: Image = Image.create(32,40,false,Image.FORMAT_RGBA8)
	image.fill(Color.WHITE)
	var tree: Sprite2D = Sprite2D.new()
	tree.texture = ImageTexture.create_from_image(image)
	tree.centered = false
	tree.offset = Vector2(-16,-39)
	tree.position = Vector2(100,100)
	tree.set_meta("art_kind","tree")
	objects.add_child(tree)
	var agent: Sprite2D = Sprite2D.new()
	agent.position = Vector2(100,90)
	agent.set_meta("art_kind","agent")
	objects.add_child(agent)
	assert(is_equal_approx(CANOPY.alpha(tree,CANOPY.buckets(objects)),0.45))
	agent.position.y = 101
	assert(CANOPY.alpha(tree,CANOPY.buckets(objects)) == 1.0)
	agent.position = Vector2(140,90)
	assert(CANOPY.alpha(tree,CANOPY.buckets(objects)) == 1.0)
	agent.position = Vector2(100,90)
	agent.visible = false
	assert(CANOPY.alpha(tree,CANOPY.buckets(objects)) == 1.0)
	agent.visible = true
	tree.modulate = Color(0.86,0.9,1.0,1.0)
	tree.modulate.a = CANOPY.alpha(tree,CANOPY.buckets(objects))
	assert(tree.modulate.is_equal_approx(Color(0.86,0.9,1.0,0.45)))
	for i: int in range(199):
		var extra: Sprite2D = Sprite2D.new()
		extra.position = Vector2((i%20)*48, floori(float(i)/20)*48)
		extra.set_meta("art_kind","agent")
		objects.add_child(extra)
	var trees: Array[Sprite2D] = []
	for i: int in range(400):
		var extra: Sprite2D = Sprite2D.new()
		extra.texture = tree.texture
		extra.centered = false
		extra.offset = tree.offset
		extra.position = Vector2((i%20)*48+8,floori(float(i)/20)*48+8)
		trees.append(extra)
		objects.add_child(extra)
	var start: int = Time.get_ticks_usec()
	for repeat: int in range(30):
		var heads: Dictionary = CANOPY.buckets(objects)
		for candidate: Sprite2D in trees:
			CANOPY.alpha(candidate,heads)
	print("canopy benchmark 200 agents/400 trees mean_us=",float(Time.get_ticks_usec()-start)/30.0)
	objects.free()
	await _scene_order_crossing()

func _scene_order_crossing() -> void:
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var world: Node = main.get_node("WorldSim")
	world.call("set_sim_speed",0.0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(main)
	var agents: Node = main.get_node("AgentRenderer")
	agents.call("_process",0.0)
	var nature: Node = main.get_node("NatureRenderer")
	var sprites: Dictionary = agents.get("_sprites")
	var agent_id: int = int(sprites.keys()[0])
	var agent: Sprite2D = sprites[agent_id] as Sprite2D
	var tree: Sprite2D = Sprite2D.new()
	var image: Image = Image.create(32,40,false,Image.FORMAT_RGBA8)
	image.fill(Color.WHITE)
	tree.texture = ImageTexture.create_from_image(image)
	tree.centered = false
	tree.offset = Vector2(-16,-39)
	tree.position = Vector2(-1000,-1000)
	tree.set_meta("art_kind","tree")
	main.get_node("WorldObjects").add_child(tree)
	var nature_sprites: Dictionary = nature.get("_sprites")
	nature_sprites["crossing_fixture"] = tree
	var driver: CrossingDriver = CrossingDriver.new()
	driver.renderer = agents
	driver.agent_id = agent_id
	driver.process_priority = agents.process_priority-1
	main.add_child(driver)
	var observer: CrossingObserver = CrossingObserver.new()
	observer.driver = driver
	observer.tree = tree
	observer.agent = agent
	observer.process_priority = maxi(nature.process_priority,agents.process_priority)+1
	main.add_child(observer)
	for step: int in range(2):
		var passed: bool = await observer.checked
		if not passed:
			push_error("same-frame canopy entry/exit failed at crossing step %d" % step)
			main.queue_free()
			quit(1)
			return
	observer.set_process(false)
	driver.set_process(false)
	main.queue_free()
	await process_frame
	print("test_canopy_fade: PASS including actual scene-order entry and exit")
	quit()
