# 040 - EventLogger Autoload

## Objective
Create the EventLogger singleton that subscribes to SimulationBus and records all events in memory with query/stats capabilities.

## Prerequisites
- 030-simulation-bus

## Non-goals
- No file-based persistence (Phase 1)
- No real-time UI viewer (ticket 140-hud handles display)

## Files to create
- `scripts/core/event_logger.gd`

## Implementation Steps
1. Create `scripts/core/event_logger.gd` extending Node
2. Internal storage:
   ```gdscript
   var _events: Array[Dictionary] = []
   var _type_counts: Dictionary = {}
   const MAX_EVENTS: int = 100000
   const PRUNE_AMOUNT: int = 10000
   ```
3. In `_ready()`, connect to `SimulationBus.simulation_event`
4. `_on_simulation_event(event: Dictionary)`:
   - Append to _events
   - Update _type_counts
   - If _events.size() > MAX_EVENTS: prune oldest PRUNE_AMOUNT
   - Print to console: `print("[Event] tick=%d type=%s" % [event.get("tick", -1), event.get("type", "unknown")])`
5. Query methods:
   ```gdscript
   func get_entity_history(entity_id: int, limit: int = 50) -> Array[Dictionary]
   func get_by_type(event_type: String, limit: int = 50) -> Array[Dictionary]
   func get_tick_range(from_tick: int, to_tick: int) -> Array[Dictionary]
   func get_stats() -> Dictionary  # returns _type_counts copy
   func get_total_count() -> int
   func get_recent(count: int = 20) -> Array[Dictionary]
   ```
6. Serialization:
   ```gdscript
   func to_save_data() -> Array[Dictionary]
   func load_save_data(data: Array) -> void
   func clear() -> void
   ```

## Verification
- Gate PASS
- Events logged when emit_event called on SimulationBus

## Acceptance Criteria
- [ ] Subscribes to SimulationBus.simulation_event
- [ ] Stores events in memory
- [ ] Prunes when exceeding MAX_EVENTS
- [ ] Query methods return correct filtered results
- [ ] Console prints each event
- [ ] Gate PASS

## Risk Notes
- Array scanning is O(n) for queries - acceptable for Phase 0
- Pruning removes oldest events (no importance weighting)

## Roll-back Plan
- Delete file, remove autoload from project.godot
