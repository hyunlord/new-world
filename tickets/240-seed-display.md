# Ticket 240: Seed Display + Entity Selection Verification [Low]

## Objective
Print a prominent startup banner with seed/world info. Verify entity click selection works correctly.

## Non-goals
- No seed input UI
- No entity selection rework

## Files to change
- `scenes/main/main.gd` — replace startup print with banner box

## Steps
1. Replace simple print with formatted banner:
   - Box drawing characters
   - Seed value, world size, entity count
2. Verify entity_renderer click detection uses canvas_transform correctly

## Done Definition
- Startup console shows formatted banner with seed, world size, entity count
- Entity click selection works (already implemented, just verify)
