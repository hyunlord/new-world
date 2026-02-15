# 050 - SimulationEngine

## Objective
Create the core simulation engine with fixed timestep tick loop, speed control, deterministic RNG, and system registration.

## Prerequisites
- 020-game-config
- 030-simulation-bus

## Non-goals
- No save/load integration (ticket 090 handles entity serialization)
- No actual systems registered (later tickets)

## Files to create
- `scripts/core/simulation_engine.gd`

## Implementation Steps
1. Create `scripts/core/simulation_engine.gd` extending RefCounted
2. State:
   ```gdscript
   var current_tick: int = 0
   var is_paused: bool = false
   var speed_index: int = 0  # index into GameConfig.SPEED_OPTIONS
   var _accumulator: float = 0.0
   var _systems: Array = []  # Array of SimulationSystem
   var rng: RandomNumberGenerator = RandomNumberGenerator.new()
   var _seed: int = 0
   ```
3. `init_with_seed(seed_value: int)`:
   - Set _seed, rng.seed = seed_value
   - Reset current_tick = 0, _accumulator = 0
4. `register_system(system) -> void`:
   - Add to _systems, sort by priority
5. `update(delta: float) -> void`:
   - If paused, return
   - Calculate tick_duration = 1.0 / GameConfig.TICKS_PER_SECOND
   - speed = GameConfig.SPEED_OPTIONS[speed_index]
   - _accumulator += delta * speed
   - ticks_this_frame = 0
   - While _accumulator >= tick_duration AND ticks_this_frame < GameConfig.MAX_TICKS_PER_FRAME:
     - _process_tick()
     - _accumulator -= tick_duration
     - ticks_this_frame += 1
6. `_process_tick()`:
   - current_tick += 1
   - For each system in _systems (sorted by priority):
     - If system.is_active and current_tick % system.tick_interval == 0:
       - system.execute_tick(current_tick)
   - SimulationBus.tick_completed.emit(current_tick)
7. Speed/pause:
   ```gdscript
   func toggle_pause() -> void
   func set_speed(index: int) -> void
   func increase_speed() -> void
   func decrease_speed() -> void
   ```
8. Game time conversion:
   ```gdscript
   func get_game_time() -> Dictionary:
       var total_hours = current_tick * GameConfig.TICK_HOURS
       var hour = total_hours % GameConfig.HOURS_PER_DAY
       var day = (total_hours / GameConfig.HOURS_PER_DAY) % GameConfig.DAYS_PER_YEAR + 1
       var year = total_hours / (GameConfig.HOURS_PER_DAY * GameConfig.DAYS_PER_YEAR) + 1
       return {"year": year, "day": day, "hour": hour, "tick": current_tick}
   ```

## Verification
- Gate PASS
- Engine can be instantiated and update() called without crash

## Acceptance Criteria
- [ ] Fixed timestep works correctly
- [ ] Speed multiplier affects tick rate
- [ ] Pause stops ticking
- [ ] MAX_TICKS_PER_FRAME prevents frame drops
- [ ] Systems executed in priority order
- [ ] Deterministic RNG with seed
- [ ] Game time conversion correct
- [ ] Gate PASS

## Risk Notes
- RefCounted cannot emit signals directly; use SimulationBus
- Accumulator must be capped to prevent spiral of death

## Roll-back Plan
- Delete file
