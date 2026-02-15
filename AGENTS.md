## Professional Standard
Act as a production-level Godot 4 developer.
Before modifying scenes or scripts:
- Check for signal connections.
- Check for NodePath dependencies.
- Ensure scene inheritance is preserved.
- Do not break existing exports.
- Do not refactor unrelated scenes.

## Goal
Implement one ticket at a time with minimal scope and produce code that passes Gate.

## Non-negotiables
- One ticket = one PR/branch (or one commit if working locally).
- Keep diffs minimal; do NOT refactor unrelated code.
- Do NOT touch secrets or add tokens.
- Do NOT introduce breaking changes without migration notes.

## What to do for each ticket
1. Read the ticket file in `tickets/###-*.md`.
2. Implement exactly what the ticket asks (no extras).
3. Run the ticket’s verification commands.
4. Report:
   - summary of changes
   - files changed
   - commands run and outputs (PASS/FAIL)
   - risks/edge cases found

## Godot-specific conventions
- Avoid renaming scenes/resources unless required.
- Keep node paths stable when possible.
- Prefer deterministic logic for simulation/gameplay.
- If multiplayer/netcode exists, avoid nondeterminism and frame-dependent bugs.

## Gate
If running Gate on Mac:
- Use: `./scripts/gate.sh` (project root)
If running Gate on Windows:
- Use: `.\scripts\gate.ps1` if provided
