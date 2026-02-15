# Ticket 440: HUD Extension + SaveManager + Main Integration [Batch 4]

## Objective
Extend HUD with resource stats, add save/load system, wire everything in main.gd.

## Dependencies
- ALL previous tickets

## Files to change
- `scripts/ui/hud.gd` — resource stats, inventory in entity panel
- NEW `scripts/core/save_manager.gd` — JSON save/load
- `scenes/main/main.gd` — wire all new systems, F5/F9 keybindings
- `scenes/main/main.tscn` — add BuildingRenderer node
- `CLAUDE.md` — update with Phase 1 docs

## Step-by-step
1. HUD top bar: Add Pop count, Food/Wood/Stone totals from stockpiles
2. Entity panel: Add job label, inventory display
3. save_manager.gd: save_game(path), load_game(path) — JSON
4. main.gd:
   - Create and init all new systems (resource_map, pathfinder, building_manager, etc.)
   - Register new systems with sim_engine
   - Add F5 (save) and F9 (load) keybindings
   - Wire building_renderer
5. CLAUDE.md: Document all new files, systems, events

## Done Definition
- HUD shows population, stockpile resources
- Entity panel shows job and inventory
- F5 saves, F9 loads
- All systems wired and running
- Gate PASS
