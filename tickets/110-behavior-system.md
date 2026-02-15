# 110 - BehaviorSystem (Utility AI)

## Objective
Create the Utility AI behavior decision system that evaluates and selects actions based on need urgency.

## Prerequisites
- 060-simulation-system-base
- 090-entity-system
- 070-world-data

## Non-goals
- No GOAP/BT (Phase 1)
- No social relationship tracking
- No memory of past actions

## Files to create
- `scripts/ai/behavior_system.gd`

## Implementation Steps
1. Create `scripts/ai/behavior_system.gd` extending SimulationSystem
2. ```gdscript
   class_name BehaviorSystem
   extends SimulationSystem

   var _entity_manager: EntityManager
   var _world_data: WorldData
   var _rng: RandomNumberGenerator

   func _init() -> void:
       system_name = "behavior"
       priority = 20
       tick_interval = 5  # decide every 5 ticks
   ```
3. `init(entity_manager: EntityManager, world_data: WorldData, rng: RandomNumberGenerator)`:
   - Store references
4. `execute_tick(tick: int) -> void`:
   - For each alive entity:
     - If entity.action_timer > 0: skip (still executing previous action)
     - scores = _evaluate_actions(entity)
     - Pick highest score action
     - _assign_action(entity, action, tick)
5. `_evaluate_actions(entity: EntityData) -> Dictionary`:
   ```
   "wander":    0.2 + randf() * 0.1  # base low-priority
   "seek_food": _urgency_curve(1.0 - entity.hunger) * 1.5
   "rest":      _urgency_curve(1.0 - entity.energy) * 1.2
   "socialize": _urgency_curve(1.0 - entity.social) * 0.8
   ```
6. `_urgency_curve(deficit: float) -> float`:
   - return pow(deficit, 2.0)  # exponential urgency
   - deficit 0.0 → 0.0 (no need), deficit 1.0 → 1.0 (critical)
7. `_assign_action(entity: EntityData, action: String, tick: int)`:
   - entity.current_action = action
   - Set action_target based on action:
     - "wander": random walkable neighbor within 5 tiles
     - "seek_food": random GRASSLAND/FOREST tile within 10 tiles (or wander if none)
     - "rest": current position (stay in place)
     - "socialize": nearest other entity position (or wander if none nearby)
   - entity.action_timer = action-specific duration (wander=5, seek_food=15, rest=10, socialize=8)
   - emit_event("action_chosen", {"entity_id": entity.id, "action": action, "tick": tick})
8. Helper: `_find_random_walkable_nearby(pos: Vector2i, radius: int) -> Vector2i`:
   - Scan radius×radius area, collect walkable tiles, pick random one

## Verification
- Gate PASS
- Entities choose actions based on needs

## Acceptance Criteria
- [ ] Utility AI scores all actions
- [ ] Exponential urgency curve
- [ ] Hungry entities prefer seek_food
- [ ] Tired entities prefer rest
- [ ] Action targets set correctly
- [ ] Gate PASS

## Risk Notes
- _find_random_walkable_nearby scans area every decision - O(r²) but r is small
- RNG must use engine's RNG for determinism

## Roll-back Plan
- Delete file
