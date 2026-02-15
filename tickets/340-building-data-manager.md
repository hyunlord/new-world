# Ticket 340: BuildingData + BuildingManager [Foundation]

## Objective
Create building data structure and manager for placing/querying buildings.

## Dependencies
- 300 (GameConfig building type definitions)

## Non-goals
- No construction system
- No rendering
- No building effects

## Files to change
- NEW `scripts/core/building_data.gd`
- NEW `scripts/core/building_manager.gd`

## Step-by-step
1. building_data.gd (extends RefCounted):
   - var id: int = 0
   - var building_type: String = ""  # "stockpile", "shelter", "campfire"
   - var tile_x: int = 0
   - var tile_y: int = 0
   - var is_built: bool = false
   - var build_progress: float = 0.0
   - var storage: Dictionary = {"food": 0.0, "wood": 0.0, "stone": 0.0}
   - func to_dict() → Dictionary
   - static func from_dict(data: Dictionary) → RefCounted (use load() pattern)
2. building_manager.gd (extends RefCounted):
   - var _buildings: Dictionary = {}  # id → BuildingData
   - var _tile_map: Dictionary = {}   # "x,y" → building_id (fast lookup)
   - var _next_id: int = 1
   - func place_building(type: String, x: int, y: int) → RefCounted
   - func get_building_at(x: int, y: int) → RefCounted (or null)
   - func get_buildings_by_type(type: String) → Array
   - func get_nearest_building(x: int, y: int, type: String) → RefCounted (or null)
   - func remove_building(id: int) → void
   - func get_all_buildings() → Array
   - func to_save_data() → Array
   - func load_save_data(data: Array) → void
3. Use SimulationBus.emit_event for building_placed, building_completed

## Done Definition
- BuildingData stores building state
- BuildingManager can place, query, remove buildings
- Tile-based lookup works
- Serialization works
- No SCRIPT ERROR in headless
