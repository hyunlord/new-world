# 060 - SimulationSystem Base Class

## Objective
Create the base class for all simulation systems (NeedsSystem, BehaviorSystem, MovementSystem).

## Prerequisites
- 030-simulation-bus

## Non-goals
- No concrete system implementations (separate tickets)

## Files to create
- `scripts/core/simulation_system.gd`

## Implementation Steps
1. Create `scripts/core/simulation_system.gd` extending RefCounted
2. Fields:
   ```gdscript
   class_name SimulationSystem
   extends RefCounted

   var system_name: String = "base"
   var priority: int = 0        # lower = earlier execution
   var tick_interval: int = 1   # execute every N ticks
   var is_active: bool = true
   ```
3. Virtual method:
   ```gdscript
   func execute_tick(_tick: int) -> void:
       pass  # Override in subclasses
   ```
4. Event helper:
   ```gdscript
   func emit_event(event_type: String, data: Dictionary = {}) -> void:
       SimulationBus.emit_event(event_type, data)
   ```

## Verification
- Gate PASS
- Class can be instantiated

## Acceptance Criteria
- [ ] class_name SimulationSystem declared
- [ ] All fields defined with defaults
- [ ] execute_tick() is overridable
- [ ] emit_event() helper delegates to SimulationBus
- [ ] Gate PASS

## Risk Notes
- class_name is global scope in Godot - ensure no conflicts
- RefCounted lifecycle managed by reference counting

## Roll-back Plan
- Delete file
