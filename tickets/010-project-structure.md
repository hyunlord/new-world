# 010 - Project Structure & Configuration

## Objective
Set up the directory structure, update project.godot with autoloads and input mappings, update CLAUDE.md for WorldSim Phase 0.

## Non-goals
- No game logic yet
- No actual script implementations (just empty placeholder files if needed)

## Files to create/modify
- `project.godot` — add autoloads, input map
- `CLAUDE.md` — rewrite for WorldSim Phase 0
- Create directories:
  - `scripts/core/`
  - `scripts/ai/`
  - `scripts/systems/`
  - `scripts/ui/`
  - `scenes/main/`
  - `resources/`
  - `tests/`

## Implementation Steps
1. Create all directories listed above (with .gdkeep or similar if needed for git)
2. Update `project.godot`:
   - Add autoload entries for SimulationBus, GameConfig, EventLogger
   - Add input mappings: camera_zoom_in, camera_zoom_out, camera_drag, pause_toggle, speed_up, speed_down
3. Rewrite `CLAUDE.md` with WorldSim Phase 0 context

## Verification
- All directories exist
- `project.godot` parses without error
- Gate PASS (headless smoke)

## Acceptance Criteria
- [ ] Directory tree matches spec
- [ ] project.godot has autoload entries
- [ ] project.godot has input mappings
- [ ] CLAUDE.md describes WorldSim architecture

## Risk Notes
- Autoload paths must match actual script paths (scripts/core/)
- Input map action names must be consistent across codebase

## Roll-back Plan
- `git revert` the commit
