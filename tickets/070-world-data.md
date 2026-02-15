# 070 - WorldData

## Objective
Create the tile-based world data structure using PackedArrays for efficient storage and access.

## Prerequisites
- 020-game-config

## Non-goals
- No world generation logic (ticket 080)
- No rendering (ticket 130)

## Files to create
- `scripts/core/world_data.gd`

## Implementation Steps
1. Create `scripts/core/world_data.gd` extending RefCounted
2. Storage (1D PackedArrays, index = y * width + x):
   ```gdscript
   class_name WorldData
   extends RefCounted

   var width: int
   var height: int
   var biomes: PackedInt32Array      # Biome enum values
   var elevation: PackedFloat32Array  # 0.0 to 1.0
   var moisture: PackedFloat32Array   # 0.0 to 1.0
   var temperature: PackedFloat32Array # 0.0 to 1.0
   var _entity_map: Dictionary = {}   # Vector2i → Array[int] (entity IDs)
   ```
3. Init:
   ```gdscript
   func init_world(w: int, h: int) -> void:
       width = w; height = h
       var size = w * h
       biomes.resize(size)
       elevation.resize(size)
       moisture.resize(size)
       temperature.resize(size)
   ```
4. Index helpers:
   ```gdscript
   func _idx(x: int, y: int) -> int: return y * width + x
   func is_valid(x: int, y: int) -> bool: return x >= 0 and x < width and y >= 0 and y < height
   func get_biome(x: int, y: int) -> int: return biomes[_idx(x, y)]
   func set_biome(x: int, y: int, b: int) -> void: biomes[_idx(x, y)] = b
   func get_elevation(x: int, y: int) -> float: return elevation[_idx(x, y)]
   func set_tile(x: int, y: int, b: int, e: float, m: float, t: float) -> void
   ```
5. Movement helpers:
   ```gdscript
   func is_walkable(x: int, y: int) -> bool:
       if not is_valid(x, y): return false
       var biome = biomes[_idx(x, y)]
       return GameConfig.BIOME_MOVE_COST.get(biome, 0.0) > 0.0

   func get_move_cost(x: int, y: int) -> float:
       return GameConfig.BIOME_MOVE_COST.get(biomes[_idx(x, y)], 0.0)

   func get_walkable_neighbors(x: int, y: int) -> Array[Vector2i]:
       var neighbors: Array[Vector2i] = []
       for dy in range(-1, 2):
           for dx in range(-1, 2):
               if dx == 0 and dy == 0: continue
               var nx = x + dx; var ny = y + dy
               if is_walkable(nx, ny):
                   neighbors.append(Vector2i(nx, ny))
       return neighbors
   ```
6. Entity map:
   ```gdscript
   func register_entity(pos: Vector2i, entity_id: int) -> void
   func unregister_entity(pos: Vector2i, entity_id: int) -> void
   func move_entity(from: Vector2i, to: Vector2i, entity_id: int) -> void
   func get_entities_at(pos: Vector2i) -> Array
   ```

## Verification
- Gate PASS

## Acceptance Criteria
- [ ] PackedArray storage for biome/elevation/moisture/temperature
- [ ] Index conversion correct (1D ↔ 2D)
- [ ] is_walkable respects biome move costs
- [ ] get_walkable_neighbors returns valid 8-direction neighbors
- [ ] Entity map tracks entity positions
- [ ] Gate PASS

## Risk Notes
- PackedInt32Array for biomes (enum stored as int)
- Entity map uses Dictionary (not PackedArray) for flexibility
- Diagonal movement cost could be √2 × cost in future

## Roll-back Plan
- Delete file
