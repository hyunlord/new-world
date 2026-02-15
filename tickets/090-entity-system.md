# 090 - Entity System (EntityData + EntityManager)

## Objective
Create the data-oriented entity system with EntityData (pure data) and EntityManager (lifecycle management).

## Prerequisites
- 020-game-config
- 030-simulation-bus
- 070-world-data

## Non-goals
- No complex AI state (separate ticket)
- No inventory/equipment
- No relationships between entities

## Files to create
- `scripts/core/entity_data.gd`
- `scripts/core/entity_manager.gd`

## Implementation Steps

### EntityData
1. Create `scripts/core/entity_data.gd` extending RefCounted
2. ```gdscript
   class_name EntityData
   extends RefCounted

   var id: int = -1
   var entity_name: String = ""
   var position: Vector2i = Vector2i.ZERO
   var is_alive: bool = true

   # Needs (0.0 = critical, 1.0 = full)
   var hunger: float = 1.0
   var energy: float = 1.0
   var social: float = 1.0

   # Attributes
   var age: int = 0  # in ticks
   var speed: float = 1.0
   var strength: float = 1.0

   # AI State
   var current_action: String = "idle"
   var current_goal: String = ""
   var action_target: Vector2i = Vector2i(-1, -1)
   var action_timer: int = 0
   ```
3. Serialization:
   ```gdscript
   func to_dict() -> Dictionary
   static func from_dict(data: Dictionary) -> EntityData
   ```

### EntityManager
1. Create `scripts/core/entity_manager.gd` extending RefCounted
2. ```gdscript
   class_name EntityManager
   extends RefCounted

   var _entities: Dictionary = {}  # id → EntityData
   var _next_id: int = 1
   var _world_data: WorldData
   var _rng: RandomNumberGenerator
   ```
3. `init(world_data: WorldData, rng: RandomNumberGenerator)`:
   - Store references
4. Name generation:
   ```gdscript
   const FIRST_NAMES = ["Alder", "Bryn", "Cedar", "Dawn", "Elm", "Fern", "Glen", "Heath", "Ivy", "Jade", "Kael", "Luna", "Moss", "Nix", "Oak", "Pine", "Quinn", "Reed", "Sage", "Thorn", "Uma", "Vale", "Wren", "Xara", "Yew", "Zara"]
   func _generate_name() -> String:
       return FIRST_NAMES[_rng.randi() % FIRST_NAMES.size()]
   ```
5. `spawn_entity(pos: Vector2i) -> EntityData`:
   - Create EntityData with _next_id
   - Random attribute variation: speed 0.8~1.2, strength 0.8~1.2
   - Register on world_data entity map
   - Emit entity_spawned event
   - Return entity
6. `move_entity(entity: EntityData, new_pos: Vector2i) -> void`:
   - Update world_data entity map
   - Update entity.position
7. `kill_entity(entity_id: int, cause: String) -> void`:
   - Set is_alive = false
   - Unregister from world_data
   - Emit entity_died event with cause
8. Query:
   ```gdscript
   func get_entity(id: int) -> EntityData
   func get_alive_entities() -> Array[EntityData]
   func get_entities_near(pos: Vector2i, radius: int) -> Array[EntityData]
   func get_alive_count() -> int
   ```

## Verification
- Gate PASS

## Acceptance Criteria
- [ ] EntityData is pure data (RefCounted)
- [ ] Serialization round-trips correctly
- [ ] EntityManager spawns with random variation
- [ ] World map updated on spawn/move/kill
- [ ] Events emitted on spawn/kill
- [ ] Gate PASS

## Risk Notes
- Entity IDs are sequential ints, never reused (prevents stale references)
- get_entities_near is O(n) scan - acceptable for Phase 0

## Roll-back Plan
- Delete files
