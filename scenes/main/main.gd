extends Node2D

const SimulationEngine = preload("res://scripts/core/simulation_engine.gd")
const WorldData = preload("res://scripts/core/world_data.gd")
const WorldGenerator = preload("res://scripts/core/world_generator.gd")
const EntityManager = preload("res://scripts/core/entity_manager.gd")
const ResourceMap = preload("res://scripts/core/resource_map.gd")
const NeedsSystem = preload("res://scripts/systems/needs_system.gd")
const BehaviorSystem = preload("res://scripts/ai/behavior_system.gd")
const MovementSystem = preload("res://scripts/systems/movement_system.gd")

var sim_engine: RefCounted
var world_data: RefCounted
var world_generator: RefCounted
var entity_manager: RefCounted
var resource_map: RefCounted
var needs_system: RefCounted
var behavior_system: RefCounted
var movement_system: RefCounted

@onready var world_renderer: Sprite2D = $WorldRenderer
@onready var entity_renderer: Node2D = $EntityRenderer
@onready var camera: Camera2D = $Camera
@onready var hud: CanvasLayer = $HUD


func _ready() -> void:
	var seed_value: int = GameConfig.WORLD_SEED

	# Initialize simulation engine
	sim_engine = SimulationEngine.new()
	sim_engine.init_with_seed(seed_value)

	# Generate world
	world_data = WorldData.new()
	world_data.init_world(GameConfig.WORLD_SIZE.x, GameConfig.WORLD_SIZE.y)
	world_generator = WorldGenerator.new()
	world_generator.generate(world_data, seed_value)

	# Initialize resource map
	resource_map = ResourceMap.new()
	resource_map.init_resources(GameConfig.WORLD_SIZE.x, GameConfig.WORLD_SIZE.y)
	var res_rng: RandomNumberGenerator = RandomNumberGenerator.new()
	res_rng.seed = seed_value + 100
	resource_map.populate_from_biomes(world_data, res_rng)

	# Initialize entity manager
	entity_manager = EntityManager.new()
	entity_manager.init(world_data, sim_engine.rng)

	# Create and register simulation systems
	needs_system = NeedsSystem.new()
	needs_system.init(entity_manager)

	behavior_system = BehaviorSystem.new()
	behavior_system.init(entity_manager, world_data, sim_engine.rng)

	movement_system = MovementSystem.new()
	movement_system.init(entity_manager, world_data)

	sim_engine.register_system(needs_system)
	sim_engine.register_system(behavior_system)
	sim_engine.register_system(movement_system)

	# Render world
	world_renderer.render_world(world_data)

	# Init renderers
	entity_renderer.init(entity_manager)
	hud.init(sim_engine, entity_manager)

	# Spawn initial entities
	_spawn_initial_entities()

	_print_startup_banner(seed_value)


func _spawn_initial_entities() -> void:
	var center := GameConfig.WORLD_SIZE / 2
	var spawn_radius: int = 30
	var walkable_tiles: Array[Vector2i] = []

	for dy in range(-spawn_radius, spawn_radius + 1):
		for dx in range(-spawn_radius, spawn_radius + 1):
			var x: int = center.x + dx
			var y: int = center.y + dy
			if world_data.is_walkable(x, y):
				walkable_tiles.append(Vector2i(x, y))

	if walkable_tiles.is_empty():
		push_warning("[Main] No walkable tiles near center!")
		return

	var count: int = mini(GameConfig.INITIAL_SPAWN_COUNT, walkable_tiles.size())
	# Shuffle using engine RNG for determinism
	for i in range(walkable_tiles.size() - 1, 0, -1):
		var j: int = sim_engine.rng.randi() % (i + 1)
		var tmp := walkable_tiles[i]
		walkable_tiles[i] = walkable_tiles[j]
		walkable_tiles[j] = tmp

	for i in range(count):
		entity_manager.spawn_entity(walkable_tiles[i])

	print("[Main] Spawned %d entities near world center." % count)


func _process(delta: float) -> void:
	sim_engine.update(delta)


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		match event.keycode:
			KEY_SPACE:
				sim_engine.toggle_pause()
			KEY_PERIOD:
				sim_engine.increase_speed()
			KEY_COMMA:
				sim_engine.decrease_speed()


func _print_startup_banner(seed_value: int) -> void:
	print("")
	print("╔══════════════════════════════════════╗")
	print("║  WorldSim Phase 0                    ║")
	print("║  Seed: %-30d ║" % seed_value)
	print("║  World: %dx%d  |  Entities: %-8d ║" % [GameConfig.WORLD_SIZE.x, GameConfig.WORLD_SIZE.y, GameConfig.INITIAL_SPAWN_COUNT])
	print("╚══════════════════════════════════════╝")
	print("")
	print("  Controls:")
	print("    WASD / Arrows  = Pan camera")
	print("    Mouse Wheel    = Zoom in/out")
	print("    Trackpad Pinch = Zoom in/out")
	print("    Two-finger Pan = Scroll camera")
	print("    Middle Mouse   = Drag pan")
	print("    Left Click     = Select entity")
	print("    Space          = Pause / Resume")
	print("    . (period)     = Speed up")
	print("    , (comma)      = Speed down")
	print("")
