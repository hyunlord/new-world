# Ticket 230: Log Filtering + Meaningful Events [Medium]

## Objective
Suppress high-frequency events (entity_moved) from console while keeping them in EventLogger. Add action_changed event with emoji formatting.

## Non-goals
- No log file output
- No UI log panel

## Files to change
- `scripts/core/event_logger.gd` — add QUIET_EVENTS filter, emoji formatting
- `scripts/ai/behavior_system.gd` — emit action_changed event when action changes

## Steps
1. In event_logger.gd:
   - Add QUIET_EVENTS array: ["entity_moved"]
   - Replace generic print with formatted output based on event type
   - Use emoji prefixes for readability
2. In behavior_system.gd `_assign_action`:
   - Capture old_action before assignment
   - Emit "action_changed" event when action differs

## Done Definition
- entity_moved not printed to console
- entity_spawned, action_changed, entity_starved printed with emoji
- EventLogger still stores ALL events
