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
const StatsRecorder = preload("res://scripts/systems/stats_recorder.gd")
const RelationshipManagerScript = preload("res://scripts/core/relationship_manager.gd")
const SocialEventSystem = preload("res://scripts/systems/social_event_system.gd")
const EmotionSystem = preload("res://scripts/systems/emotion_system.gd")
const AgeSystem = preload("res://scripts/systems/age_system.gd")
const FamilySystem = preload("res://scripts/systems/family_system.gd")
const MortalitySystem = preload("res://scripts/systems/mortality_system.gd")
const ChildcareSystem = preload("res://scripts/systems/childcare_system.gd")
const StressSystem = preload("res://scripts/systems/stress_system.gd")
const MentalBreakSystem = preload("res://scripts/systems/mental_break_system.gd")
const TraumaScarSystem = preload("res://scripts/systems/trauma_scar_system.gd")
const TraitViolationSystem = preload("res://scripts/systems/trait_violation_system.gd")
const CopingSystem = preload("res://scripts/systems/phase4/coping_system.gd")
const MoraleSystem = preload("res://scripts/systems/phase4/morale_system.gd")
const ContagionSystem = preload("res://scripts/systems/phase4/contagion_system.gd")
const Phase4CoordinatorScript = preload("res://scripts/systems/phase4/phase4_coordinator.gd")
const ChildStressProcessor = preload("res://scripts/systems/phase5/child_stress_processor.gd")
const IntergenerationalSystem = preload("res://scripts/systems/phase5/intergenerational_system.gd")
const ParentingSystem = preload("res://scripts/systems/phase5/parenting_system.gd")
const PauseMenuClass = preload("res://scripts/ui/pause_menu.gd")

var sim_engine: RefCounted
var world_data: RefCounted
var world_generator: RefCounted
var entity_manager: RefCounted
var resource_map: RefCounted
var pathfinder: RefCounted
var building_manager: RefCounted
var save_manager: RefCounted
var settlement_manager: RefCounted
var stats_recorder: RefCounted
var relationship_manager: RefCounted
var pause_menu: CanvasLayer
var _last_used_slot: int = 1

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
var social_event_system: RefCounted
var emotion_system: RefCounted
var age_system: RefCounted
var family_system: RefCounted
var mortality_system: RefCounted
var childcare_system: RefCounted
var stress_system: RefCounted
var mental_break_system: RefCounted
var trauma_scar_system: RefCounted
var trait_violation_system: RefCounted
var coping_system: RefCounted
var morale_system: RefCounted
var contagion_system: RefCounted
var phase4_coordinator: Node
var child_stress_processor: RefCounted
var intergenerational_system: RefCounted
var parenting_system: RefCounted
var debug_console = null
var debug_panel = null

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
	save_manager.migrate_legacy_save()

	# Initialize settlement manager + first settlement
	settlement_manager = SettlementManager.new()
	entity_manager._settlement_manager = settlement_manager

	# Initialize relationship manager
	relationship_manager = RelationshipManagerScript.new()

	# ── Create simulation systems ──────────────────────────
	resource_regen_system = ResourceRegenSystem.new()
	resource_regen_system.init(resource_map, world_data)

	job_assignment_system = JobAssignmentSystem.new()
	job_assignment_system.init(entity_manager, building_manager)

	needs_system = NeedsSystem.new()
	needs_system.init(entity_manager, building_manager)

	childcare_system = ChildcareSystem.new()
	childcare_system.init(entity_manager, building_manager, settlement_manager)

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

	emotion_system = EmotionSystem.new()
	emotion_system.init(entity_manager)

	age_system = AgeSystem.new()
	age_system.init(entity_manager, sim_engine.rng)

	mortality_system = MortalitySystem.new()
	mortality_system.init(entity_manager, sim_engine.rng)
	needs_system._mortality_system = mortality_system

	stress_system = StressSystem.new()
	stress_system.init(entity_manager)
	mortality_system._stress_system = stress_system

	mental_break_system = MentalBreakSystem.new()
	mental_break_system.init(entity_manager, sim_engine.rng)

	trauma_scar_system = TraumaScarSystem.new()
	trauma_scar_system.init({"entity_manager": entity_manager})
	mental_break_system.set_trauma_scar_system(trauma_scar_system)
	stress_system.set_trauma_scar_system(trauma_scar_system)

	trait_violation_system = TraitViolationSystem.new()
	trait_violation_system.init({"entity_manager": entity_manager})
	trait_violation_system._stress_system = stress_system
	trait_violation_system._trauma_scar_system = trauma_scar_system
	behavior_system.set_trait_violation_system(trait_violation_system)

	social_event_system = SocialEventSystem.new()
	social_event_system.init(entity_manager, relationship_manager, sim_engine.rng)
	social_event_system._stress_system = stress_system

	family_system = FamilySystem.new()
	family_system.init(entity_manager, relationship_manager, building_manager, settlement_manager, sim_engine.rng, mortality_system)
	family_system._stress_system = stress_system

	stats_recorder = StatsRecorder.new()
	stats_recorder.init(entity_manager, building_manager, settlement_manager)

	# ── Phase 4: Coping / Morale / Contagion ───────────────
	coping_system = CopingSystem.new()
	coping_system.init(entity_manager, sim_engine.rng)

	morale_system = MoraleSystem.new()
	morale_system.init(entity_manager)
	behavior_system.set_morale_system(morale_system)

	contagion_system = ContagionSystem.new()
	contagion_system.init(entity_manager)

	phase4_coordinator = Phase4CoordinatorScript.new()
	add_child(phase4_coordinator)
	phase4_coordinator.init_phase4(coping_system, morale_system, contagion_system, stress_system, entity_manager)

	# ── Phase 5: Childhood / ACE / Parenting ──────────────
	child_stress_processor = ChildStressProcessor.new()
	child_stress_processor.init(entity_manager)
	intergenerational_system = IntergenerationalSystem.new()
	intergenerational_system.init(entity_manager, settlement_manager)
	parenting_system = ParentingSystem.new()
	parenting_system.init(entity_manager)

	# ── Register all systems (auto-sorted by priority) ─────
	sim_engine.register_system(resource_regen_system)     # priority 5
	sim_engine.register_system(childcare_system)          # priority 8 (before needs — feed children first)
	sim_engine.register_system(job_assignment_system)     # priority 8
	sim_engine.register_system(needs_system)              # priority 10
	sim_engine.register_system(building_effect_system)    # priority 15
	sim_engine.register_system(behavior_system)           # priority 20
	sim_engine.register_system(gathering_system)          # priority 25
	sim_engine.register_system(construction_system)       # priority 28
	sim_engine.register_system(movement_system)           # priority 30
	sim_engine.register_system(emotion_system)            # priority 32
	sim_engine.register_system(child_stress_processor)   # priority 32 (Phase 5)
	sim_engine.register_system(stress_system)             # priority 34
	sim_engine.register_system(mental_break_system)       # priority 35
	sim_engine.register_system(trauma_scar_system)        # priority 36
	sim_engine.register_system(trait_violation_system)   # priority 37
	sim_engine.register_system(social_event_system)       # priority 37
	sim_engine.register_system(age_system)                # priority 48
	sim_engine.register_system(mortality_system)          # priority 49
	sim_engine.register_system(population_system)         # priority 50
	sim_engine.register_system(family_system)             # priority 52
	sim_engine.register_system(migration_system)          # priority 60
	sim_engine.register_system(stats_recorder)            # priority 90
	sim_engine.register_system(contagion_system)          # priority 38
	sim_engine.register_system(morale_system)             # priority 40
	sim_engine.register_system(coping_system)             # priority 42
	sim_engine.register_system(intergenerational_system) # priority 45 (Phase 5)
	sim_engine.register_system(parenting_system)         # priority 46 (Phase 5)

	# Render world (with resource tinting)
	world_renderer.render_world(world_data, resource_map)

	# Init renderers with updated references
	entity_renderer.init(entity_manager, building_manager, resource_map)
	building_renderer.init(building_manager, settlement_manager)
	hud.init(sim_engine, entity_manager, building_manager, settlement_manager, world_data, camera, stats_recorder, relationship_manager)
	pause_menu = PauseMenuClass.new()
	pause_menu.set_save_manager(save_manager)
	pause_menu.save_requested.connect(_save_game_slot)
	pause_menu.load_requested.connect(_load_game_slot)
	add_child(pause_menu)

	# Debug Console + Panel (debug build only — self-destructs in release)
	if OS.is_debug_build():
		var _dc_script = load("res://scenes/debug/debug_console.gd")
		if _dc_script != null:
			debug_console = _dc_script.new()
			debug_console._entity_manager = entity_manager
			debug_console._stress_system = stress_system
			debug_console._mental_break_system = mental_break_system
			debug_console._trauma_scar_system = trauma_scar_system
			debug_console._trait_violation_system = trait_violation_system
			debug_console._sim_engine = sim_engine
			debug_console._coping_system = coping_system
			debug_console._morale_system = morale_system
			debug_console._contagion_system = contagion_system
			debug_console._settlement_manager = settlement_manager
			debug_console._child_stress_processor = child_stress_processor
			debug_console._intergenerational_system = intergenerational_system
			debug_console._parenting_system = parenting_system
			debug_console._behavior_system = behavior_system
			add_child(debug_console)
			debug_console.init_phase4_commands()
			debug_console.init_phase5_commands()
			debug_console.init_behavior_commands()
		var _dp_script = load("res://scenes/debug/debug_panel.gd")
		if _dp_script != null:
			debug_panel = _dp_script.new()
			debug_panel._entity_manager = entity_manager
			debug_panel._stress_system = stress_system
			debug_panel._mental_break_system = mental_break_system
			debug_panel._trauma_scar_system = trauma_scar_system
			debug_panel._trait_violation_system = trait_violation_system
			debug_panel._sim_engine = sim_engine
			debug_panel._console = debug_console
			add_child(debug_panel)

	# Initialize name generator with sim RNG and entity manager
	NameGenerator.init(sim_engine.rng, entity_manager)

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
		# Register name in NameGenerator for duplicate prevention
		NameGenerator.register_name(e.entity_name, founding.id)

	# Bootstrap initial stockpile so gatherers can deliver and HUD shows resources
	_bootstrap_stockpile(founding, center)

	# Bootstrap initial relationships for faster couple formation
	_bootstrap_relationships(initial_alive)

	# Initialize chronicle system with entity manager for name lookups
	ChronicleSystem.init(entity_manager)

	# Enable camera entity following
	camera.set_entity_manager(entity_manager)

	# Connect lifecycle signals for chronicle event logging
	SimulationBus.entity_born.connect(_on_entity_born_chronicle)
	SimulationBus.entity_died.connect(_on_entity_died_chronicle)
	SimulationBus.couple_formed.connect(_on_couple_formed_chronicle)

	_print_startup_banner(seed_value)
	hud.show_startup_toast(entity_manager.get_alive_count())


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
		# Weighted age distribution for realistic population pyramid
		var age_years: int = _weighted_random_age(sim_engine.rng)
		var day_offset: int = sim_engine.rng.randi_range(0, 364)
		var initial_age: int = age_years * GameConfig.TICKS_PER_YEAR + day_offset * 12
		entity_manager.spawn_entity(walkable_tiles[i], "", initial_age)

	print("[Main] Spawned %d entities near world center." % count)


## Weighted random age for initial population pyramid
## 0-5: 10%, 5-14: 15%, 15-30: 40%, 30-50: 25%, 50-70: 8%, 70+: 2%
func _weighted_random_age(rng: RandomNumberGenerator) -> int:
	var roll: float = rng.randf()
	if roll < 0.10:
		return rng.randi_range(0, 4)
	elif roll < 0.25:
		return rng.randi_range(5, 14)
	elif roll < 0.65:
		return rng.randi_range(15, 30)
	elif roll < 0.90:
		return rng.randi_range(30, 50)
	elif roll < 0.98:
		return rng.randi_range(50, 69)
	else:
		return rng.randi_range(70, 80)


## Bootstrap initial relationships so couples can form quickly
func _bootstrap_relationships(alive: Array) -> void:
	if alive.size() < 4:
		return
	# Separate by gender for close_friend pairing (opposite gender needed for romance)
	var males: Array = []
	var females: Array = []
	for i in range(alive.size()):
		var e: RefCounted = alive[i]
		if e.gender == "male":
			males.append(e)
		else:
			females.append(e)

	# 3-4 friend pairs (same or mixed gender)
	var friend_count: int = mini(4, alive.size() / 4)
	for i in range(friend_count):
		var idx_a: int = (i * 2) % alive.size()
		var idx_b: int = (i * 2 + 1) % alive.size()
		if idx_a == idx_b:
			continue
		var rel: RefCounted = relationship_manager.get_or_create(alive[idx_a].id, alive[idx_b].id)
		rel.affinity = 40.0
		rel.trust = 55.0
		rel.interaction_count = 12
		rel.type = "friend"

	# 2-3 partner pairs (opposite gender, adults only)
	var partner_count: int = mini(3, mini(males.size(), females.size()))
	var adult_males: Array = []
	var adult_females: Array = []
	for i in range(males.size()):
		if males[i].age_stage == "adult":
			adult_males.append(males[i])
	for i in range(females.size()):
		if females[i].age_stage == "adult":
			adult_females.append(females[i])
	partner_count = mini(partner_count, mini(adult_males.size(), adult_females.size()))
	for i in range(partner_count):
		var m: RefCounted = adult_males[i]
		var f: RefCounted = adult_females[i]
		var rel: RefCounted = relationship_manager.get_or_create(m.id, f.id)
		rel.affinity = 85.0
		rel.trust = 75.0
		rel.romantic_interest = 90.0
		rel.interaction_count = 25
		rel.type = "partner"
		m.partner_id = f.id
		f.partner_id = m.id
		m.emotions["love"] = 0.5
		f.emotions["love"] = 0.5

	print("[Main] Bootstrapped %d friend + %d partner relationships." % [friend_count, partner_count])


## Bootstrap a pre-built stockpile with starter resources
func _bootstrap_stockpile(settlement: RefCounted, center: Vector2i) -> void:
	# Find a walkable tile near center for the stockpile
	for dy in range(-3, 4):
		for dx in range(-3, 4):
			var sx: int = center.x + dx
			var sy: int = center.y + dy
			if world_data.is_walkable(sx, sy):
				var sp: RefCounted = building_manager.place_building("stockpile", sx, sy)
				if sp != null:
					sp.is_built = true
					sp.build_progress = 1.0
					sp.settlement_id = settlement.id
					sp.storage["food"] = 15.0
					sp.storage["wood"] = 5.0
					sp.storage["stone"] = 2.0
					settlement.building_ids.append(sp.id)
					print("[Main] Bootstrapped stockpile at (%d,%d) with starter resources." % [sx, sy])
					return
	push_warning("[Main] Could not place bootstrap stockpile near center!")


var _last_overlay_tick: int = 0
var _last_minimap_tick: int = 0
var _last_balance_tick: int = 0
var _current_day_color: Color = Color(1.0, 1.0, 1.0)
var _day_night_enabled: bool = true

func _process(delta: float) -> void:
	sim_engine.update(delta)
	var current_tick: int = sim_engine.current_tick

	# Refresh resource overlay every 100 ticks
	if current_tick - _last_overlay_tick >= 100:
		_last_overlay_tick = current_tick
		world_renderer.update_resource_overlay()

	# Refresh minimap every 20 ticks
	if current_tick - _last_minimap_tick >= 20:
		_last_minimap_tick = current_tick
		var minimap = hud.get_minimap()
		if minimap != null:
			minimap.request_update()
			minimap.update_minimap()

	# Balance debug log every 500 ticks
	if current_tick - _last_balance_tick >= 500 and current_tick > 0:
		_last_balance_tick = current_tick
		_log_balance(current_tick)

	# Day/night cycle (smooth lerp, slower at high speed)
	if sim_engine and _day_night_enabled:
		var gt: Dictionary = sim_engine.get_game_time()
		var hour_f: float = float(gt.hour)
		var target_color: Color = _get_daylight_color(hour_f)
		var lerp_speed: float = 0.3 * delta
		if sim_engine.speed_index >= 3:
			lerp_speed = 0.05 * delta
		_current_day_color = _current_day_color.lerp(target_color, minf(lerp_speed, 1.0))
		world_renderer.modulate = _current_day_color


func _get_daylight_color(hour: float) -> Color:
	if hour >= 7.0 and hour < 17.0:
		return Color(1.0, 1.0, 1.0)           # Day
	elif hour >= 17.0 and hour < 19.0:
		return Color(1.0, 0.88, 0.75)          # Sunset
	elif hour >= 19.0 or hour < 5.0:
		return Color(0.55, 0.55, 0.7)          # Night
	else:  # 5~7
		return Color(0.8, 0.8, 0.9)            # Dawn


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		if event.ctrl_pressed:
			match event.keycode:
				KEY_S:
					_save_game()
					get_viewport().set_input_as_handled()
				KEY_L:
					_load_game()
					get_viewport().set_input_as_handled()
				KEY_EQUAL:
					GameConfig.ui_scale = minf(GameConfig.ui_scale + 0.1, GameConfig.UI_SCALE_MAX)
					hud.apply_ui_scale()
					get_viewport().set_input_as_handled()
				KEY_MINUS:
					GameConfig.ui_scale = maxf(GameConfig.ui_scale - 0.1, GameConfig.UI_SCALE_MIN)
					hud.apply_ui_scale()
					get_viewport().set_input_as_handled()
				KEY_0:
					GameConfig.ui_scale = 1.0
					hud.apply_ui_scale()
					get_viewport().set_input_as_handled()
		else:
			match event.keycode:
				KEY_ESCAPE:
					if pause_menu.is_menu_visible():
						pause_menu.hide_menu()
					elif not hud.close_all_popups():
						pause_menu.show_menu()
				KEY_SPACE:
					sim_engine.toggle_pause()
				KEY_PERIOD:
					sim_engine.increase_speed()
				KEY_COMMA:
					sim_engine.decrease_speed()
				KEY_TAB:
					world_renderer.toggle_resource_overlay()
					var overlay_vis: bool = world_renderer.is_resource_overlay_visible()
					hud.set_resource_legend_visible(overlay_vis)
					entity_renderer.resource_overlay_visible = overlay_vis
				KEY_M:
					hud.toggle_minimap()
				KEY_G:
					hud.toggle_stats()
				KEY_H:
					hud.toggle_help()
				KEY_N:
					_day_night_enabled = not _day_night_enabled
					if not _day_night_enabled:
						_current_day_color = Color(1.0, 1.0, 1.0)
						world_renderer.modulate = Color(1.0, 1.0, 1.0)
				KEY_C:
					hud.toggle_chronicle()
				KEY_P:
					hud.toggle_list()
				KEY_E:
					if hud.is_detail_visible():
						hud.close_detail()
					else:
						hud.open_entity_detail()
						hud.open_building_detail()


func _save_game() -> void:
	_save_game_slot(_last_used_slot)


func _load_game() -> void:
	_load_game_slot(_last_used_slot)


func _save_game_slot(slot: int) -> void:
	_last_used_slot = slot
	var path: String = save_manager.get_slot_dir(slot)
	var was_paused: bool = sim_engine.is_paused
	sim_engine.is_paused = true
	var success: bool = save_manager.save_game(path, sim_engine, entity_manager, building_manager, resource_map, settlement_manager, relationship_manager, stats_recorder)
	if success:
		NameGenerator.save_registry(path + "/names.json")
		print("[Main] Game saved to slot %d (tick %d)" % [slot, sim_engine.current_tick])
	else:
		push_warning("[Main] Save failed!")
	sim_engine.is_paused = was_paused


func _load_game_slot(slot: int) -> void:
	_last_used_slot = slot
	var path: String = save_manager.get_slot_dir(slot)
	sim_engine.is_paused = true
	var success: bool = save_manager.load_game(path, sim_engine, entity_manager, building_manager, resource_map, world_data, settlement_manager, relationship_manager, stats_recorder)
	if success:
		# Re-render world with loaded resource data
		world_renderer.render_world(world_data, resource_map)
		# Sync death/birth counters from loaded stats to entity manager
		entity_manager.total_deaths = stats_recorder.total_deaths
		entity_manager.total_births = stats_recorder.total_births
		# Restore name registry
		NameGenerator.load_registry(path + "/names.json")
		print("[Main] Game loaded from slot %d (tick %d)" % [slot, sim_engine.current_tick])
	else:
		push_warning("[Main] Load failed!")
	sim_engine.is_paused = false


func _print_startup_banner(seed_value: int) -> void:
	print("")
	print("======================================")
	print("  WorldSim Phase 2-A1")
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
	print("    Left Click     = Select entity/building")
	print("    Space          = Pause / Resume")
	print("    . (period)     = Speed up")
	print("    , (comma)      = Speed down")
	print("    Tab            = Toggle resource overlay")
	print("    M              = Toggle minimap")
	print("    G              = Statistics detail")
	print("    E              = Entity/Building detail")
	print("    C              = Chronicle")
	print("    P              = Entity/Building list")
	print("    H              = Help overlay (pauses)")
	print("    N              = Toggle day/night")
	print("    Cmd+S          = Quick Save")
	print("    Cmd+L          = Quick Load")
	print("    Cmd+=/-/0      = UI Scale (%.1f)" % GameConfig.ui_scale)
	print("    Double-click   = Open detail popup")
	print("")


func _log_balance(tick: int) -> void:
	var alive: Array = entity_manager.get_alive_entities()
	var pop: int = alive.size()
	if pop == 0:
		print("[Balance] tick=%d pop=0 ALL DEAD" % tick)
		return
	var total_hunger: float = 0.0
	var total_food_inv: float = 0.0
	var gatherers: int = 0
	for i in range(alive.size()):
		var e: RefCounted = alive[i]
		total_hunger += e.hunger
		total_food_inv += e.inventory.get("food", 0.0)
		if e.current_action == "gather_food":
			gatherers += 1
	var avg_hunger: float = total_hunger / float(pop)
	var food_in_stockpiles: float = 0.0
	if building_manager != null:
		var stockpiles: Array = building_manager.get_buildings_by_type("stockpile")
		for i in range(stockpiles.size()):
			if stockpiles[i].is_built:
				food_in_stockpiles += stockpiles[i].storage.get("food", 0.0)
	print("[Balance] tick=%d pop=%d avg_hunger=%.2f food_inv=%.1f food_stockpile=%.1f gatherers=%d" % [
		tick, pop, avg_hunger, total_food_inv, food_in_stockpiles, gatherers])


## Chronicle event handlers — bridge lifecycle signals to ChronicleSystem
func _on_entity_born_chronicle(entity_id: int, entity_name: String, parent_ids: Array, tick: int) -> void:
	var parent_names: String = ""
	for i in range(parent_ids.size()):
		var pid: int = parent_ids[i]
		var pe: RefCounted = entity_manager.get_entity(pid)
		if pe != null:
			parent_names += pe.entity_name
		else:
			parent_names += "?"
		if i < parent_ids.size() - 1:
			parent_names += ", "
	var desc: String = "%s born" % entity_name
	var l10n_key: String = "EVT_BORN"
	if parent_names != "":
		desc += " (parents: %s)" % parent_names
		l10n_key = "EVT_BORN_PARENTS"
	ChronicleSystem.log_event(ChronicleSystem.EVENT_BIRTH, entity_id, desc, 4, parent_ids, tick,
		{"key": l10n_key, "params": {"name": entity_name, "parents": parent_names}})


func _on_entity_died_chronicle(entity_id: int, entity_name: String, cause: String, age_years: float, tick: int) -> void:
	var desc: String = "%s died (%s, age %d)" % [entity_name, cause, int(age_years)]
	ChronicleSystem.log_event(ChronicleSystem.EVENT_DEATH, entity_id, desc, 4, [], tick,
		{"key": "EVT_DIED", "params": {"name": entity_name, "cause_id": cause, "age": str(int(age_years))}})


func _on_couple_formed_chronicle(entity_a_id: int, entity_a_name: String, entity_b_id: int, entity_b_name: String, tick: int) -> void:
	var desc: String = "%s and %s coupled" % [entity_a_name, entity_b_name]
	ChronicleSystem.log_event(ChronicleSystem.EVENT_MARRIAGE, entity_a_id, desc, 3, [entity_b_id], tick,
		{"key": "EVT_MARRIED", "params": {"name": entity_a_name, "partner": entity_b_name}})
