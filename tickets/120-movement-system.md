# 120 - MovementSystem

## Objective
Create the movement system that moves entities toward their action targets and applies arrival effects.

## Prerequisites
- 060-simulation-system-base
- 090-entity-system
- 070-world-data

## Non-goals
- No pathfinding (A* is Phase 1)
- No collision between entities
- No terrain speed modifiers (Phase 1)

## Files to create
- `scripts/systems/movement_system.gd`

## Implementation Steps
1. Create `scripts/systems/movement_system.gd` extending SimulationSystem
2. ```gdscript
   class_name MovementSystem
   extends SimulationSystem

   var _entity_manager: EntityManager
   var _world_data: WorldData

   func _init() -> void:
       system_name = "movement"
       priority = 30
       tick_interval = 1
   ```
3. `init(entity_manager: EntityManager, world_data: WorldData)`:
   - Store references
4. `execute_tick(tick: int) -> void`:
   - For each alive entity:
     - Decrease action_timer by 1
     - If action_timer <= 0 and current_action != "idle":
       - _apply_arrival_effect(entity, tick)
       - entity.current_action = "idle"
       - continue
     - If current_action == "rest" or current_action == "idle": continue
     - If action_target == entity.position: continue
     - If action_target == Vector2i(-1, -1): continue
     - _move_toward_target(entity, tick)
5. `_move_toward_target(entity: EntityData, tick: int)`:
   - Greedy 8-directional: pick the neighbor closest to target that is walkable
   - dx = sign(target.x - pos.x), dy = sign(target.y - pos.y)
   - Try (pos.x+dx, pos.y+dy) first, then fallback to (pos.x+dx, pos.y) or (pos.x, pos.y+dy)
   - If valid: _entity_manager.move_entity(entity, new_pos)
   - emit_event("entity_moved", {"entity_id": entity.id, "from": old_pos, "to": new_pos, "tick": tick})
6. `_apply_arrival_effect(entity: EntityData, tick: int)`:
   ```
   match entity.current_action:
       "seek_food":
           entity.hunger = min(entity.hunger + 0.4, 1.0)
           emit_event("entity_ate", {...})
       "rest":
           entity.energy = min(entity.energy + 0.5, 1.0)
           emit_event("entity_rested", {...})
       "socialize":
           entity.social = min(entity.social + 0.3, 1.0)
           emit_event("entity_socialized", {...})
       "wander":
           pass  # no effect, just moved
   ```

## Verification
- Gate PASS
- Entities move toward targets
- Arrival effects restore needs

## Acceptance Criteria
- [ ] Greedy 8-direction movement
- [ ] Respects walkability
- [ ] Arrival effects for each action type
- [ ] action_timer countdown
- [ ] Events emitted on move/arrival
- [ ] Gate PASS

## Risk Notes
- Greedy movement can get stuck on concave obstacles
- No diagonal cost multiplier (all moves cost 1 tick)
- Entities may overlap on same tile (no collision)

## Roll-back Plan
- Delete file
