extends Node2D

const SimulationEngine = preload("res://scripts/core/simulation_engine.gd")
const WorldData = preload("res://scripts/core/world_data.gd")
const WorldGenerator = preload("res://scripts/core/world_generator.gd")
const EntityManager = preload("res://scripts/core/entity_manager.gd")
const ResourceMap = preload("res://scripts/core/resource_map.gd")
const Pathfinder = preload("res://scripts/core/pathfinder.gd")
const BuildingManager = preload("res://scripts/core/building_manager.gd")
const SaveManager = preload("res://scripts/core/save_manager.gd")
const NeedsSystem = preload("res://scripts/systems/needs_system.gd")
const BehaviorSystem = preload("res://scripts/ai/behavior_system.gd")
const MovementSystem = preload("res://scripts/systems/movement_system.gd")
const ResourceRegenSystem = preload("res://scripts/systems/resource_regen_system.gd")
const GatheringSystem = preload("res://scripts/systems/gathering_system.gd")
const ConstructionSystem = preload("res://scripts/systems/construction_system.gd")
const BuildingEffectSystem = preload("res://scripts/systems/building_effect_system.gd")
const JobAssignmentSystem = preload("res://scripts/systems/job_assignment_system.gd")
const PopulationSystem = preload("res://scripts/systems/population_system.gd")
const SettlementManager = preload("res://scripts/core/settlement_manager.gd")
const MigrationSystem = preload("res://scripts/systems/migration_system.gd")

var sim_engine: RefCounted
var world_data: RefCounted
var world_generator: RefCounted
var entity_manager: RefCounted
var resource_map: RefCounted
var pathfinder: RefCounted
var building_manager: RefCounted
var save_manager: RefCounted
var settlement_manager: RefCounted

var needs_system: RefCounted
var behavior_system: RefCounted
var movement_system: RefCounted
var resource_regen_system: RefCounted
var gathering_system: RefCounted
var construction_system: RefCounted
var building_effect_system: RefCounted
var job_assignment_system: RefCounted
var population_system: RefCounted
var migration_system: RefCounted

@onready var world_renderer: Sprite2D = $WorldRenderer
@onready var entity_renderer: Node2D = $EntityRenderer
@onready var building_renderer: Node2D = $BuildingRenderer
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

	# Initialize pathfinder
	pathfinder = Pathfinder.new()

	# Initialize building manager
	building_manager = BuildingManager.new()

	# Initialize save manager
	save_manager = SaveManager.new()

	# Initialize settlement manager + first settlement
	settlement_manager = SettlementManager.new()

	# ── Create simulation systems ──────────────────────────
	resource_regen_system = ResourceRegenSystem.new()
	resource_regen_system.init(resource_map, world_data)

	job_assignment_system = JobAssignmentSystem.new()
	job_assignment_system.init(entity_manager, building_manager)

	needs_system = NeedsSystem.new()
	needs_system.init(entity_manager)

	building_effect_system = BuildingEffectSystem.new()
	building_effect_system.init(entity_manager, building_manager, sim_engine)

	behavior_system = BehaviorSystem.new()
	behavior_system.init(entity_manager, world_data, sim_engine.rng, resource_map, building_manager, settlement_manager)

	gathering_system = GatheringSystem.new()
	gathering_system.init(entity_manager, resource_map)

	construction_system = ConstructionSystem.new()
	construction_system.init(entity_manager, building_manager)

	movement_system = MovementSystem.new()
	movement_system.init(entity_manager, world_data, pathfinder, building_manager)

	population_system = PopulationSystem.new()
	population_system.init(entity_manager, building_manager, world_data, sim_engine.rng, settlement_manager)

	migration_system = MigrationSystem.new()
	migration_system.init(entity_manager, building_manager, settlement_manager, world_data, resource_map, sim_engine.rng)

	# ── Register all systems (auto-sorted by priority) ─────
	sim_engine.register_system(resource_regen_system)     # priority 5
	sim_engine.register_system(job_assignment_system)     # priority 8
	sim_engine.register_system(needs_system)              # priority 10
	sim_engine.register_system(building_effect_system)    # priority 15
	sim_engine.register_system(behavior_system)           # priority 20
	sim_engine.register_system(gathering_system)          # priority 25
	sim_engine.register_system(construction_system)       # priority 28
	sim_engine.register_system(movement_system)           # priority 30
	sim_engine.register_system(population_system)         # priority 50
	sim_engine.register_system(migration_system)          # priority 60

	# Render world (with resource tinting)
	world_renderer.render_world(world_data, resource_map)

	# Init renderers
	entity_renderer.init(entity_manager)
	building_renderer.init(building_manager)
	hud.init(sim_engine, entity_manager, building_manager, settlement_manager)

	# Spawn initial entities + create first settlement
	_spawn_initial_entities()

	# Create founding settlement at world center and assign all entities
	var center := GameConfig.WORLD_SIZE / 2
	var founding: RefCounted = settlement_manager.create_settlement(center.x, center.y, 0)
	var initial_alive: Array = entity_manager.get_alive_entities()
	for i in range(initial_alive.size()):
		var e: RefCounted = initial_alive[i]
		e.settlement_id = founding.id
		settlement_manager.add_member(founding.id, e.id)

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


var _last_overlay_tick: int = 0

func _process(delta: float) -> void:
	sim_engine.update(delta)
	# Refresh resource overlay every 100 ticks
	var current_tick: int = sim_engine.current_tick
	if current_tick - _last_overlay_tick >= 100:
		_last_overlay_tick = current_tick
		world_renderer.update_resource_overlay()


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		match event.keycode:
			KEY_SPACE:
				sim_engine.toggle_pause()
			KEY_PERIOD:
				sim_engine.increase_speed()
			KEY_COMMA:
				sim_engine.decrease_speed()
			KEY_TAB:
				world_renderer.toggle_resource_overlay()
			KEY_F5:
				_save_game()
			KEY_F9:
				_load_game()


func _save_game() -> void:
	var path: String = "user://quicksave.json"
	var was_paused: bool = sim_engine.is_paused
	sim_engine.is_paused = true
	var success: bool = save_manager.save_game(path, sim_engine, entity_manager, building_manager, resource_map, settlement_manager)
	if success:
		print("[Main] Game saved to %s (tick %d)" % [path, sim_engine.current_tick])
	else:
		push_warning("[Main] Save failed!")
	sim_engine.is_paused = was_paused


func _load_game() -> void:
	var path: String = "user://quicksave.json"
	sim_engine.is_paused = true
	var success: bool = save_manager.load_game(path, sim_engine, entity_manager, building_manager, resource_map, world_data, settlement_manager)
	if success:
		# Re-render world with loaded resource data
		world_renderer.render_world(world_data, resource_map)
		print("[Main] Game loaded from %s (tick %d)" % [path, sim_engine.current_tick])
	else:
		push_warning("[Main] Load failed!")
	sim_engine.is_paused = false


func _print_startup_banner(seed_value: int) -> void:
	print("")
	print("======================================")
	print("  WorldSim Phase 1")
	print("  Seed: %d" % seed_value)
	print("  World: %dx%d  |  Entities: %d" % [GameConfig.WORLD_SIZE.x, GameConfig.WORLD_SIZE.y, GameConfig.INITIAL_SPAWN_COUNT])
	print("  Systems: %d registered" % sim_engine._systems.size())
	print("======================================")
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
	print("    Tab            = Toggle resource overlay")
	print("    F5             = Quick Save")
	print("    F9             = Quick Load")
	print("")
