# 030 - SimulationBus Autoload

## Objective
Create the global event bus singleton for decoupled communication between simulation systems and UI.

## Prerequisites
- 010-project-structure

## Non-goals
- No event filtering/routing (all subscribers get all events)
- No persistence (EventLogger handles that)

## Files to create
- `scripts/core/simulation_bus.gd`

## Implementation Steps
1. Create `scripts/core/simulation_bus.gd` extending Node
2. Define signals:
   ```gdscript
   signal simulation_event(event: Dictionary)
   signal ui_notification(message: String, type: String)
   signal entity_selected(entity_id: int)
   signal entity_deselected()
   signal tick_completed(tick: int)
   signal speed_changed(speed_index: int)
   signal pause_changed(paused: bool)
   ```
3. Add helper method:
   ```gdscript
   func emit_event(event_type: String, data: Dictionary = {}) -> void:
       var event := {
           "type": event_type,
           "tick": data.get("tick", -1),
           "timestamp": Time.get_ticks_msec(),
       }
       event.merge(data)
       simulation_event.emit(event)
   ```
4. Add UI notification helper:
   ```gdscript
   func notify(message: String, type: String = "info") -> void:
       ui_notification.emit(message, type)
   ```

## Verification
- File parses without error
- Signals can be connected in tests
- Gate PASS

## Acceptance Criteria
- [ ] SimulationBus.gd extends Node
- [ ] All signals defined with proper types
- [ ] emit_event() helper works
- [ ] Gate PASS

## Risk Notes
- Signal parameter types must match across all emitters/receivers
- Dictionary events are flexible but not type-safe (acceptable for Phase 0)

## Roll-back Plan
- Delete file, remove autoload from project.godot
