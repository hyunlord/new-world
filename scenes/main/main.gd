extends Node2D

const SimulationEngine = preload("res://scripts/core/simulation/simulation_engine.gd")
const WorldData = preload("res://scripts/core/world/world_data.gd")
const WorldGenerator = preload("res://scripts/core/world/world_generator.gd")
const EntityManager = preload("res://scripts/core/entity/entity_manager.gd")
const ResourceMap = preload("res://scripts/core/world/resource_map.gd")
const BuildingManager = preload("res://scripts/core/settlement/building_manager.gd")
const SaveManager = preload("res://scripts/core/simulation/save_manager.gd")
const SettlementManager = preload("res://scripts/core/settlement/settlement_manager.gd")
const RelationshipManagerScript = preload("res://scripts/core/social/relationship_manager.gd")
const PauseMenuClass = preload("res://scripts/ui/panels/pause_menu.gd")
const AmbienceManagerClass = preload("res://scripts/ui/ambience_manager.gd")
const ReputationManagerScript = preload("res://scripts/core/social/reputation_manager.gd")
const TechTreeManagerScript = preload("res://scripts/core/tech/tech_tree_manager.gd")
const WorldSetupScript = preload("res://scenes/setup/world_setup.gd")
const OverlayRendererClass = preload("res://scripts/ui/renderers/overlay_renderer.gd")

var sim_engine: RefCounted
var world_data: RefCounted
var world_generator: RefCounted
var entity_manager: RefCounted
var resource_map: RefCounted
var building_manager: RefCounted
var save_manager: RefCounted
var settlement_manager: RefCounted
var relationship_manager: RefCounted
var reputation_manager: RefCounted
var tech_tree_manager: RefCounted
var pause_menu: CanvasLayer
var ambience_manager: Node = null
var _last_used_slot: int = 1
var _world_setup: Node = null
var _loading_overlay: CanvasLayer = null
var _loading_bar: ProgressBar = null
var _loading_label: Label = null
var _loading_count_label: Label = null
var _overlay_renderer: Sprite2D = null

@onready var world_renderer: Sprite2D = $WorldRenderer
@onready var entity_renderer: Node2D = $EntityRenderer
@onready var building_renderer: Node2D = $BuildingRenderer
@onready var camera: Camera2D = $Camera
@onready var hud: CanvasLayer = $HUD


func _ready() -> void:
	RenderingServer.set_default_clear_color(Color(0.024, 0.031, 0.063, 1.0))
	DisplayServer.window_set_mode(DisplayServer.WINDOW_MODE_MAXIMIZED)
	var seed_value: int = GameConfig.WORLD_SEED

	# Initialize simulation engine
	sim_engine = SimulationEngine.new()
	sim_engine.init_with_seed(seed_value)
	var registry_validation: Dictionary = sim_engine.validate_runtime_registry()
	if not bool(registry_validation.get("count_match", false)) or not bool(registry_validation.get("all_rust", false)):
		push_error("[Main] Rust runtime registry authority validation failed; aborting boot.")
		return

	# Generate world
	world_data = WorldData.new()
	world_data.init_world(GameConfig.WORLD_SIZE.x, GameConfig.WORLD_SIZE.y)
	world_generator = WorldGenerator.new()

	# Initialize resource map
	resource_map = ResourceMap.new()
	resource_map.init_resources(GameConfig.WORLD_SIZE.x, GameConfig.WORLD_SIZE.y)

	# Initialize entity manager
	entity_manager = EntityManager.new()
	entity_manager.init(world_data, sim_engine.rng)

	# Initialize building manager
	building_manager = BuildingManager.new()

	# Initialize save manager
	save_manager = SaveManager.new()

	# Initialize settlement manager + first settlement
	settlement_manager = SettlementManager.new()
	entity_manager._settlement_manager = settlement_manager

	# Initialize relationship manager
	relationship_manager = RelationshipManagerScript.new()

	# ── Bug #2 fix: StatQuery needs settlement_manager for skill XP gating ──
	StatQuery.init_settlement_manager(settlement_manager)
	reputation_manager = ReputationManagerScript.new()
	tech_tree_manager = TechTreeManagerScript.new()
	tech_tree_manager.load_all()

	# Init renderers with updated references
	entity_renderer.init(entity_manager, building_manager, resource_map, settlement_manager, sim_engine)
	building_renderer.init(null, null, sim_engine)
	_overlay_renderer = OverlayRendererClass.new()
	_overlay_renderer.name = "OverlayRenderer"
	add_child(_overlay_renderer)
	move_child(_overlay_renderer, world_renderer.get_index() + 1)
	_overlay_renderer.init(sim_engine, world_renderer)
	hud.init(sim_engine, entity_manager, building_manager, settlement_manager, world_data, camera, null, relationship_manager, reputation_manager)
	_ensure_ambience_manager()
	hud.call_deferred("set_tech_tree_manager", tech_tree_manager)
	pause_menu = PauseMenuClass.new()
	pause_menu.set_save_manager(save_manager)
	pause_menu.set_sim_engine(sim_engine)
	pause_menu.save_requested.connect(_save_game_slot)
	pause_menu.load_requested.connect(_load_game_slot)
	add_child(pause_menu)

	## 시뮬레이션 일시정지 후 맵 에디터/프리셋 선택 화면 표시
	sim_engine.is_paused = true
	_build_loading_overlay()
	_enter_setup_mode()


## true → 맵 에디터를 건너뛰고 바로 샌드박스 모드로 시작 (디버그용)
const SKIP_SETUP: bool = true


## WorldSetup 씬 생성 및 world_data/resource_map 주입
func _enter_setup_mode() -> void:
	if SKIP_SETUP:
		@warning_ignore("integer_division")
		var preset_gen = preload("res://scripts/core/world/preset_map_generator.gd").new()
		preset_gen.generate_preset(world_data, resource_map, "island", GameConfig.PRESET_SEED_ISLAND)
		var suggestions: Array = preset_gen.get_spawn_suggestions(world_data, resource_map)
		var spawn_data: Array = []
		for s in suggestions:
			spawn_data.append({position = s, count = GameConfig.MAP_EDITOR_SPAWN_DEFAULT})
		if spawn_data.is_empty():
			@warning_ignore("integer_division")
			var center: Vector2i = GameConfig.WORLD_SIZE / 2
			spawn_data.append({position = center, count = GameConfig.MAP_EDITOR_SPAWN_DEFAULT})
		_on_setup_confirmed(spawn_data, GameConfig.STARTUP_MODE_SANDBOX)
		return
	hud.visible = false
	_world_setup = WorldSetupScript.new()
	_world_setup.setup(world_data, resource_map)
	add_child(_world_setup)
	_world_setup.setup_confirmed.connect(_on_setup_confirmed)


## 스폰 진행률 표시용 로딩 오버레이 동적 생성 (숨김 상태로 시작)
func _build_loading_overlay() -> void:
	_loading_overlay = CanvasLayer.new()
	_loading_overlay.layer = 10
	_loading_overlay.visible = false
	add_child(_loading_overlay)

	var bg := ColorRect.new()
	bg.color = Color(0.05, 0.05, 0.08, 0.92)
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	_loading_overlay.add_child(bg)

	var vbox := VBoxContainer.new()
	vbox.set_anchors_preset(Control.PRESET_CENTER)
	vbox.custom_minimum_size = Vector2(320, 100)
	vbox.alignment = BoxContainer.ALIGNMENT_CENTER
	bg.add_child(vbox)

	_loading_label = Label.new()
	_loading_label.text = Locale.ltr("UI_LOADING_SPAWNING")
	_loading_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	vbox.add_child(_loading_label)

	_loading_bar = ProgressBar.new()
	_loading_bar.min_value = 0.0
	_loading_bar.max_value = 1.0
	_loading_bar.step = 0.001
	_loading_bar.value = 0.0
	_loading_bar.custom_minimum_size = Vector2(300, 20)
	vbox.add_child(_loading_bar)

	_loading_count_label = Label.new()
	_loading_count_label.text = "0 / 0"
	_loading_count_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	vbox.add_child(_loading_count_label)


func _ensure_ambience_manager() -> void:
	if ambience_manager != null:
		return
	ambience_manager = AmbienceManagerClass.new()
	ambience_manager.name = "AmbienceManager"
	add_child(ambience_manager)


## WorldSetup 완료 시 호출 — 맵 렌더링, 스폰, 시뮬레이션 시작
func _on_setup_confirmed(spawn_data: Array, startup_mode: String) -> void:
	var resolved_startup_mode: String = _normalize_startup_mode(startup_mode)
	if _world_setup != null:
		_world_setup.queue_free()
		_world_setup = null

	world_renderer.render_world(world_data, resource_map)
	if _overlay_renderer != null and _overlay_renderer.has_method("sync_with_world_renderer"):
		_overlay_renderer.call("sync_with_world_renderer", world_renderer)
	_loading_overlay.visible = true
	var bootstrap_agents: Array = await _spawn_at_points_async(spawn_data, resolved_startup_mode)
	_loading_overlay.visible = false

	@warning_ignore("integer_division")
	var center: Vector2i = GameConfig.WORLD_SIZE / 2
	if not bootstrap_agents.is_empty():
		center = Vector2i(int(bootstrap_agents[0].get("x", center.x)), int(bootstrap_agents[0].get("y", center.y)))
	elif not spawn_data.is_empty():
		center = spawn_data[0].position
	var bootstrap_result: Dictionary = sim_engine.bootstrap_world(
		_build_runtime_bootstrap_payload(center, bootstrap_agents, resolved_startup_mode)
	)

	ChronicleSystem.init(entity_manager)
	if camera.has_method("set_sim_engine"):
		camera.call("set_sim_engine", sim_engine)

	SimulationBus.entity_born.connect(_on_entity_born_chronicle)
	SimulationBus.entity_died.connect(_on_entity_died_chronicle)
	SimulationBus.couple_formed.connect(_on_couple_formed_chronicle)

	hud.visible = true
	_apply_startup_mode_presentation(resolved_startup_mode, center)
	_print_startup_banner(GameConfig.WORLD_SEED)
	hud.show_startup_toast(
		int(bootstrap_result.get("entity_count", bootstrap_agents.size())),
		resolved_startup_mode
	)
	sim_engine.is_paused = false


## 스폰 포인트 목록에서 에이전트를 SPAWN_BATCH_SIZE씩 나눠 스폰.
## 각 배치 후 await process_frame으로 화면 업데이트 허용.
func _spawn_at_points_async(spawn_data: Array, startup_mode: String) -> Array:
	if startup_mode == GameConfig.STARTUP_MODE_PROBE:
		return await _spawn_probe_agents_async(spawn_data)

	# 총 스폰 수 미리 계산 (프로그레스 바용)
	var total: int = 0
	if spawn_data.is_empty():
		@warning_ignore("integer_division")
		total = mini(GameConfig.INITIAL_SPAWN_COUNT,
			_count_walkable_near(GameConfig.WORLD_SIZE / 2, 30))
	else:
		for sp in spawn_data:
			total += sp.get("count", GameConfig.MAP_EDITOR_SPAWN_DEFAULT)
	total = maxi(total, 1)

	_loading_bar.value = 0.0
	_loading_count_label.text = Locale.trf("UI_LOADING_COUNT_FMT", {"current": 0, "total": total})

	var spawned: int = 0
	var agents: Array = []

	if spawn_data.is_empty():
		# 폴백: 월드 중심 근처에서 스폰
		@warning_ignore("integer_division")
		var center := GameConfig.WORLD_SIZE / 2
		var walkable: Array = _get_walkable_near(center, 30)
		if walkable.is_empty():
			push_warning("[Main] No walkable tiles near center!")
			return []
		var count: int = mini(GameConfig.INITIAL_SPAWN_COUNT, walkable.size())
		# Fisher-Yates shuffle (기존 로직 유지)
		for i in range(walkable.size() - 1, 0, -1):
			var j: int = sim_engine.rng.randi() % (i + 1)
			var tmp = walkable[i]
			walkable[i] = walkable[j]
			walkable[j] = tmp
		for i in range(count):
			var age_years: int = _weighted_random_age(sim_engine.rng)
			var day_offset: int = sim_engine.rng.randi_range(0, 364)
			var initial_age: int = age_years * GameConfig.TICKS_PER_YEAR + day_offset * 12
			agents.append({"x": walkable[i].x, "y": walkable[i].y, "age_ticks": initial_age})
			spawned += 1
			if spawned % GameConfig.SPAWN_BATCH_SIZE == 0:
				_loading_bar.value = float(spawned) / float(total)
				_loading_count_label.text = Locale.trf("UI_LOADING_COUNT_FMT",
					{"current": spawned, "total": total})
				await get_tree().process_frame
		print("[Main] Spawned %d entities near world center." % spawned)
	else:
		for sp in spawn_data:
			var pos: Vector2i = sp.position
			var count: int = sp.get("count", GameConfig.MAP_EDITOR_SPAWN_DEFAULT)
			var nearby: Array = _get_walkable_near(pos, 30)
			if nearby.is_empty():
				continue
			for i in range(count):
				var tile: Vector2i = nearby[sim_engine.rng.randi() % nearby.size()]
				var age_years: int = _weighted_random_age(sim_engine.rng)
				var day_offset: int = sim_engine.rng.randi_range(0, 364)
				var initial_age: int = age_years * GameConfig.TICKS_PER_YEAR + day_offset * 12
				agents.append({"x": tile.x, "y": tile.y, "age_ticks": initial_age})
				spawned += 1
				if spawned % GameConfig.SPAWN_BATCH_SIZE == 0:
					_loading_bar.value = float(spawned) / float(total)
					_loading_count_label.text = Locale.trf("UI_LOADING_COUNT_FMT",
						{"current": spawned, "total": total})
					await get_tree().process_frame
		print("[Main] Spawned entities at %d spawn point(s)." % spawn_data.size())

	# 마지막 배치가 BATCH_SIZE 미만인 경우 최종 업데이트
	_loading_bar.value = 1.0
	_loading_count_label.text = Locale.trf("UI_LOADING_COUNT_FMT",
		{"current": spawned, "total": total})
	await get_tree().process_frame
	return agents


func _spawn_probe_agents_async(spawn_data: Array) -> Array:
	var center: Vector2i = _probe_spawn_anchor(spawn_data)
	var walkable: Array = _get_sorted_walkable_near(center, GameConfig.PROBE_START_SPAWN_RADIUS)
	if walkable.is_empty():
		push_warning("[Main] No walkable tiles for Probe Start!")
		return []

	var total: int = GameConfig.PROBE_START_POPULATION
	_loading_bar.value = 0.0
	_loading_count_label.text = Locale.trf("UI_LOADING_COUNT_FMT", {"current": 0, "total": total})

	var agents: Array = []
	for i in range(total):
		var tile: Vector2i = walkable[min(i, walkable.size() - 1)]
		var age_years: int = GameConfig.PROBE_START_AGE_YEARS[i % GameConfig.PROBE_START_AGE_YEARS.size()]
		var day_offset: int = GameConfig.PROBE_START_DAY_OFFSETS[i % GameConfig.PROBE_START_DAY_OFFSETS.size()]
		var initial_age: int = age_years * GameConfig.TICKS_PER_YEAR + day_offset * 12
		agents.append({
			"x": tile.x,
			"y": tile.y,
			"age_ticks": initial_age,
			"sex": GameConfig.PROBE_START_SEXES[i % GameConfig.PROBE_START_SEXES.size()],
		})
		_loading_bar.value = float(i + 1) / float(total)
		_loading_count_label.text = Locale.trf(
			"UI_LOADING_COUNT_FMT",
			{"current": i + 1, "total": total})
		await get_tree().process_frame

	return agents


func _normalize_startup_mode(startup_mode: String) -> String:
	if startup_mode == GameConfig.STARTUP_MODE_PROBE:
		return GameConfig.STARTUP_MODE_PROBE
	return GameConfig.STARTUP_MODE_SANDBOX


func _probe_spawn_anchor(spawn_data: Array) -> Vector2i:
	if not spawn_data.is_empty():
		return spawn_data[0].position
	@warning_ignore("integer_division")
	return GameConfig.WORLD_SIZE / 2


func _get_sorted_walkable_near(center: Vector2i, radius: int) -> Array:
	var result: Array = _get_walkable_near(center, radius)
	result.sort_custom(func(left: Vector2i, right: Vector2i) -> bool:
		var left_dist: int = absi(left.x - center.x) + absi(left.y - center.y)
		var right_dist: int = absi(right.x - center.x) + absi(right.y - center.y)
		if left_dist != right_dist:
			return left_dist < right_dist
		if left.y != right.y:
			return left.y < right.y
		return left.x < right.x
	)
	return result


func _apply_startup_mode_presentation(startup_mode: String, center: Vector2i) -> void:
	var is_probe: bool = startup_mode == GameConfig.STARTUP_MODE_PROBE
	if hud.has_method("set_startup_mode"):
		hud.call("set_startup_mode", startup_mode)
	if hud.has_method("set_probe_observation_mode"):
		hud.call("set_probe_observation_mode", is_probe)
	if entity_renderer.has_method("set_probe_observation_mode"):
		entity_renderer.call("set_probe_observation_mode", is_probe)
	if camera.has_method("set_probe_observation_mode"):
		# Keep the camera in manual observation mode across startup presets.
		camera.call("set_probe_observation_mode", true)
	if is_probe:
		if world_renderer.has_method("set_resource_overlay_visible"):
			world_renderer.call("set_resource_overlay_visible", false)
		hud.set_resource_legend_visible(false)
		entity_renderer.resource_overlay_visible = false
	else:
		if world_renderer.has_method("is_resource_overlay_visible"):
			entity_renderer.resource_overlay_visible = bool(world_renderer.call("is_resource_overlay_visible"))
		return

	var focus_entity_id: int = _probe_focus_entity_id()
	if focus_entity_id >= 0:
		SimulationBus.entity_selected.emit(focus_entity_id)
		if camera.has_method("focus_entity"):
			camera.call("focus_entity", focus_entity_id, "startup_focus")
		return

	if camera.has_method("focus_world_tile"):
		camera.call("focus_world_tile", Vector2(center.x, center.y), "startup_focus")


func _probe_focus_entity_id() -> int:
	var summary: Dictionary = sim_engine.get_world_summary()
	if summary.is_empty():
		return -1
	var settlement_summaries: Array = summary.get("settlement_summaries", [])
	if settlement_summaries.is_empty():
		return -1
	var first_summary_raw: Variant = settlement_summaries[0]
	if not (first_summary_raw is Dictionary):
		return -1
	var first_summary: Dictionary = first_summary_raw
	var settlement_detail_raw: Variant = first_summary.get("settlement", {})
	if not (settlement_detail_raw is Dictionary):
		return -1
	var settlement_detail: Dictionary = settlement_detail_raw
	var member_ids: Variant = settlement_detail.get("member_ids", [])
	if not (member_ids is Array) or member_ids.is_empty():
		return -1
	return int(member_ids[0])


func _build_runtime_bootstrap_payload(center: Vector2i, agents: Array, startup_mode: String) -> Dictionary:
	var width: int = world_data.width
	var height: int = world_data.height
	var tile_count: int = width * height
	var biomes: Array = []
	var elevation: Array = []
	var moisture: Array = []
	var temperature: Array = []
	var food: Array = []
	var wood: Array = []
	var stone: Array = []
	biomes.resize(tile_count)
	elevation.resize(tile_count)
	moisture.resize(tile_count)
	temperature.resize(tile_count)
	food.resize(tile_count)
	wood.resize(tile_count)
	stone.resize(tile_count)
	var idx: int = 0
	for y in range(height):
		for x in range(width):
			biomes[idx] = world_data.get_biome(x, y)
			elevation[idx] = world_data.get_elevation(x, y)
			moisture[idx] = world_data.get_moisture(x, y)
			temperature[idx] = world_data.get_temperature(x, y)
			food[idx] = resource_map.get_food(x, y)
			wood[idx] = resource_map.get_wood(x, y)
			stone[idx] = resource_map.get_stone(x, y)
			idx += 1
	var stockpile: Dictionary = _startup_stockpile_for_mode(startup_mode)
	return {
		"startup_mode": startup_mode,
		"world": {
			"width": width,
			"height": height,
			"biomes": biomes,
			"elevation": elevation,
			"moisture": moisture,
			"temperature": temperature,
			"food": food,
			"wood": wood,
			"stone": stone,
		},
		"founding_settlement": {
			"id": 1,
			"name": "Settlement 1",
			"x": center.x,
			"y": center.y,
			"stockpile_food": float(stockpile.get("food", 0.0)),
			"stockpile_wood": float(stockpile.get("wood", 0.0)),
			"stockpile_stone": float(stockpile.get("stone", 0.0)),
		},
		"agents": agents,
	}


func _startup_stockpile_for_mode(startup_mode: String) -> Dictionary:
	if startup_mode == GameConfig.STARTUP_MODE_PROBE:
		return {
			"food": GameConfig.PROBE_START_SETTLEMENT_FOOD,
			"wood": GameConfig.PROBE_START_SETTLEMENT_WOOD,
			"stone": GameConfig.PROBE_START_SETTLEMENT_STONE,
		}
	return {
		"food": GameConfig.SANDBOX_START_SETTLEMENT_FOOD,
		"wood": GameConfig.SANDBOX_START_SETTLEMENT_WOOD,
		"stone": GameConfig.SANDBOX_START_SETTLEMENT_STONE,
	}


## 중심 타일 근처의 walkable 타일 목록 반환
func _get_walkable_near(center: Vector2i, radius: int) -> Array:
	var result: Array = []
	for dy in range(-radius, radius + 1):
		for dx in range(-radius, radius + 1):
			var x: int = center.x + dx
			var y: int = center.y + dy
			if world_data.is_walkable(x, y):
				result.append(Vector2i(x, y))
	return result


## 총 스폰 수 사전 계산용 — 타일 목록 대신 카운트만 반환
func _count_walkable_near(center: Vector2i, radius: int) -> int:
	var count: int = 0
	for dy in range(-radius, radius + 1):
		for dx in range(-radius, radius + 1):
			if world_data.is_walkable(center.x + dx, center.y + dy):
				count += 1
	return count


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


var _last_overlay_tick: int = 0
var _last_minimap_tick: int = 0
var _last_balance_tick: int = 0
var _current_day_color: Color = Color(1.0, 1.0, 1.0)
var _day_night_enabled: bool = true


func _process(delta: float) -> void:
	if _world_setup != null:
		return
	if sim_engine == null:
		return
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
		var hour_f: float = float(gt.get("hour", 0))
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
					if ambience_manager != null and ambience_manager.has_method("toggle_mute"):
						var muted: bool = bool(ambience_manager.call("toggle_mute"))
						if hud != null and hud.has_method("show_sound_status"):
							hud.call("show_sound_status", muted)
					get_viewport().set_input_as_handled()
				KEY_B:
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
				KEY_F3:
					hud.toggle_debug_overlay()
					get_viewport().set_input_as_handled()
				KEY_F12:
					if GameConfig.DEBUG_PANEL_ENABLED:
						hud.toggle_debug_panel()
						get_viewport().set_input_as_handled()


func _save_game() -> void:
	_save_game_slot(_last_used_slot)


func _load_game() -> void:
	_load_game_slot(_last_used_slot)


func _save_game_slot(slot: int) -> void:
	_last_used_slot = slot
	var path: String = save_manager.get_slot_dir(slot)
	var was_paused: bool = sim_engine.is_paused
	sim_engine.is_paused = true
	var success: bool = save_manager.save_game(path, sim_engine, entity_manager, building_manager, resource_map, settlement_manager, relationship_manager, null)
	if not success:
		push_warning("[Main] Save failed!")
	sim_engine.is_paused = was_paused


func _load_game_slot(slot: int) -> void:
	_last_used_slot = slot
	var path: String = save_manager.get_slot_dir(slot)
	sim_engine.is_paused = true
	var success: bool = save_manager.load_game(path, sim_engine, entity_manager, building_manager, resource_map, world_data, settlement_manager, relationship_manager, null)
	if success:
		# Re-render world with loaded resource data
		world_renderer.render_world(world_data, resource_map)
		if _overlay_renderer != null and _overlay_renderer.has_method("sync_with_world_renderer"):
			_overlay_renderer.call("sync_with_world_renderer", world_renderer)
	else:
		push_warning("[Main] Load failed!")
	sim_engine.is_paused = false


func _print_startup_banner(seed_value: int) -> void:
	print("")
	print("======================================")
	print("  WorldSim Phase 2-A1")
	print("  Seed: %d" % seed_value)
	print("  World: %dx%d  |  Entities: %d" % [GameConfig.WORLD_SIZE.x, GameConfig.WORLD_SIZE.y, GameConfig.INITIAL_SPAWN_COUNT])
	print("  Systems: %d registered" % sim_engine.get_registered_system_count())
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
	print("    B              = Cycle minimap size")
	print("    M              = Toggle ambient sound")
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
	if not GameConfig.DEBUG_BALANCE_LOG:
		return
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
