# 100 - NeedsSystem

## Objective
Create the needs decay system that reduces hunger/energy each tick and handles starvation death.

## Prerequisites
- 060-simulation-system-base
- 090-entity-system

## Non-goals
- No complex need interactions
- No need regeneration (handled by behavior effects in MovementSystem)

## Files to create
- `scripts/systems/needs_system.gd`

## Implementation Steps
1. Create `scripts/systems/needs_system.gd` extending SimulationSystem
2. ```gdscript
   class_name NeedsSystem
   extends SimulationSystem

   var _entity_manager: EntityManager

   func _init() -> void:
       system_name = "needs"
       priority = 10
       tick_interval = 1
   ```
3. `init(entity_manager: EntityManager) -> void`:
   - Store reference
4. `execute_tick(tick: int) -> void`:
   - For each alive entity:
     - Decrease hunger by GameConfig.HUNGER_DECAY_RATE
     - Decrease energy by GameConfig.ENERGY_DECAY_RATE
     - If entity.current_action != "idle" and current_action != "rest":
       - Extra energy cost: GameConfig.ENERGY_ACTION_COST
     - Decrease social by GameConfig.SOCIAL_DECAY_RATE
     - entity.age += 1
     - Clamp all needs to [0.0, 1.0]
     - If hunger <= 0.0:
       - _entity_manager.kill_entity(entity.id, "starvation")
       - emit_event("entity_starved", {"entity_id": entity.id, "entity_name": entity.entity_name, "tick": tick})

## Verification
- Gate PASS
- Entity hunger decreases each tick
- Entity dies when hunger reaches 0

## Acceptance Criteria
- [ ] Extends SimulationSystem
- [ ] priority=10, tick_interval=1
- [ ] Hunger/energy/social decay per tick
- [ ] Extra energy cost during actions
- [ ] Starvation death at hunger=0
- [ ] Age increments
- [ ] Gate PASS

## Risk Notes
- Decay rates must be balanced so entities don't die too fast
- At default rates: ~333 ticks to starve (≈33 seconds at 10 tps)

## Roll-back Plan
- Delete file
