# T-410: Settlement Data + Manager

## Action: DISPATCH (Codex)
## Files: scripts/core/settlement_data.gd (NEW), scripts/core/settlement_manager.gd (NEW)

## CRITICAL: NO class_name on RefCounted scripts (headless mode fails)

### settlement_data.gd
```
extends RefCounted
var id: int = 0
var center_x: int = 0
var center_y: int = 0
var founding_tick: int = 0
var member_ids: Array[int] = []
var building_ids: Array[int] = []

func to_dict() -> Dictionary
static func from_dict(data: Dictionary) -> RefCounted
```

### settlement_manager.gd
```
extends RefCounted
var _settlements: Dictionary = {}  # id -> settlement
var _next_id: int = 1

func create_settlement(cx: int, cy: int, tick: int) -> RefCounted
func get_settlement(id: int) -> RefCounted
func get_all_settlements() -> Array
func get_nearest_settlement(x: int, y: int) -> RefCounted
func add_member(settlement_id: int, entity_id: int) -> void
func remove_member(settlement_id: int, entity_id: int) -> void
func add_building(settlement_id: int, building_id: int) -> void
func get_settlement_population(id: int) -> int
func to_save_data() -> Array
func load_save_data(data: Array) -> void
```
