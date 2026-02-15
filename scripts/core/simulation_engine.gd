extends RefCounted

var current_tick: int = 0
var is_paused: bool = false
var speed_index: int = 0
var rng: RandomNumberGenerator = RandomNumberGenerator.new()

var _accumulator: float = 0.0
var _systems: Array = []
var _seed: int = 0


## Initialize the engine with a deterministic seed
func init_with_seed(seed_value: int) -> void:
	_seed = seed_value
	rng.seed = seed_value
	current_tick = 0
	_accumulator = 0.0


## Register a simulation system (sorted by priority)
func register_system(system: RefCounted) -> void:
	_systems.append(system)
	_systems.sort_custom(func(a, b): return a.priority < b.priority)


## Called every frame from Main._process(delta)
func update(delta: float) -> void:
	if is_paused:
		return
	var tick_duration: float = 1.0 / GameConfig.TICKS_PER_SECOND
	var speed: int = GameConfig.SPEED_OPTIONS[speed_index]
	_accumulator += delta * speed
	var ticks_this_frame: int = 0
	while _accumulator >= tick_duration and ticks_this_frame < GameConfig.MAX_TICKS_PER_FRAME:
		_process_tick()
		_accumulator -= tick_duration
		ticks_this_frame += 1
	# Prevent spiral of death
	if _accumulator > tick_duration * 3.0:
		_accumulator = 0.0


func _process_tick() -> void:
	current_tick += 1
	for i in range(_systems.size()):
		var system = _systems[i]
		if system.is_active and current_tick % system.tick_interval == 0:
			system.execute_tick(current_tick)
	SimulationBus.tick_completed.emit(current_tick)


## Toggle pause state
func toggle_pause() -> void:
	is_paused = not is_paused
	SimulationBus.pause_changed.emit(is_paused)


## Set speed by index
func set_speed(index: int) -> void:
	speed_index = clampi(index, 0, GameConfig.SPEED_OPTIONS.size() - 1)
	SimulationBus.speed_changed.emit(speed_index)


## Increase speed to next level
func increase_speed() -> void:
	set_speed(speed_index + 1)


## Decrease speed to previous level
func decrease_speed() -> void:
	set_speed(speed_index - 1)


## Convert current tick to game time
func get_game_time() -> Dictionary:
	return GameConfig.tick_to_date(current_tick)
