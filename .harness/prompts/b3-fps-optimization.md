# B3: FPS Optimization — _draw() Overlay Streamlining

## Summary
Performance refactoring of entity_renderer.gd. Removes dead legacy per-agent draw loop and adds early-continue to binary snapshot overlay loop. No behavior change — Sprite2D already handles agent body rendering.

## Changes
- File: scripts/ui/renderers/entity_renderer.gd
- `_draw()`: Remove ~80-line legacy per-agent loop (draw_circle, _draw_triangle_outlined, etc.). Legacy fallback now only draws civilization regions, settlement boundaries, band labels, hover tooltip.
- `_draw_binary_snapshots()`: Add `needs_overlay` early-continue — skip per-agent expensive work (pos calculation, job/vis/size lookups) for agents that have no selection, no danger flag, no probe mode, and no name to draw.
- `danger_flags` hoisted before early-continue to avoid redundant re-read inside Z2 block.
- No simulation logic changes. No Rust changes. No new localization keys.
- Helper functions `_draw_triangle_outlined`, `_draw_square_outlined`, `_draw_diamond_outlined` retained — still used by `_draw_probe_survival_badge`.

## Verification
- Agents render via Sprite2D as before (no visual regression)
- Selection ring appears on clicked agent
- Hover tooltip works
- Hunger warning (red dot) appears on hungry agents
- Probe survival badges visible in probe mode
- Z3+ zoom hides sprites, shows settlement labels
- FPS improvement target: 20 → 30+ at 23 agents
