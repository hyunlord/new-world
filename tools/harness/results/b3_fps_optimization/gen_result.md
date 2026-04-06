---
feature: b3_fps_optimization
code_attempt: 2
---

## Files Changed
- `rust/crates/sim-test/src/main.rs`: 4 harness tests present from attempt 1 — no changes made this attempt.
- `scripts/ui/renderers/entity_renderer.gd`: **No changes made.** B3 GDScript changes were already applied before attempt 1 began and remain in place:
  - `_draw()` legacy fallback (lines 507–522): no per-agent draw loop; only overlay calls.
  - `_draw_binary_snapshots()` (lines 566–610): `danger_flags` hoisted at line 575; `needs_overlay` early-continue at lines 576–578, skipping pos/job/size lookups for agents needing no overlay.
  - Helper functions `_draw_triangle_outlined`, `_draw_square_outlined`, `_draw_diamond_outlined` retained; used by `_draw_probe_survival_badge`.

## Observed Values (seed 42, 20 agents)
- Alive agents at tick 2000: **43** (20 original + 23 children born)
- Identity names empty at tick 2000: **0 of 43**
- Positions outside [0.0, 256.0] at tick 2000: **0 of 43**
- Agents with Needs.values[Hunger=0] < 0.30 at tick 4380: **6 of 60** (ratio 0.10)

## Threshold Compliance
- Assertion 1 (`renderer_agent_count_stable_2000_ticks`): plan=exactly 20, observed=43, **FAIL**
- Assertion 2 (`renderer_no_empty_identity_name`): plan=0 empty names, observed=0 empty, **PASS**
- Assertion 3 (`renderer_positions_within_bounds`): plan=all in [0.0, 256.0], observed=0 out-of-bounds, **PASS**
- Assertion 4 (`renderer_hunger_distribution_soft`): plan=≥2 agents hunger<0.30, observed=6/60, **PASS**

## Gate Result
- cargo test: **FAIL** (27 passed, 1 failed — `harness_renderer_agent_count_stable_2000_ticks`)
- clippy: **PASS** (0 warnings, 0 errors)
- harness: **3/4 passed**

## Notes

### Assertion 1 — Plan Threshold Mismatch (threshold NOT changed per harness rules)

The plan specifies `== 20` alive agents after 2000 ticks with rationale "A silent despawn of even one
agent is a bug." Observed: 43 alive at tick 2000 (seed 42, 20 initial agents).

Root cause: `count_alive` counts ALL alive agents including children born during simulation.
The simulation correctly spawns 23 children (`child_20`–`child_42`) by tick 2000. The plan's
"exactly 20" threshold is incompatible with a simulation that has functioning birth mechanics.

The plan's intent (detect silent despawns) is correct. The threshold expression is wrong.
`alive >= 20` would correctly detect silent despawns while tolerating legitimate births.

**Threshold was NOT changed.** Per harness rules, discrepancy noted in result summary only.

**RE-PLAN recommendation**: Change Assertion 1 from `== 20` to `>= 20`, or filter
`count_alive` to count only originally-spawned agents (names matching "Agent N" pattern).

### GDScript implementation status — Fully applied
All B3 changes confirmed present in `entity_renderer.gd`:
1. `_draw()` fallback contains no per-agent draw loop — only overlay helpers called.
2. `danger_flags` hoisted before `needs_overlay` computation (line 575).
3. `needs_overlay` early-continue (lines 576–578) skips per-agent pos/job/size/vis
   lookups for agents with no selection, danger flag, probe mode, or name to draw.
4. Helper draw functions retained for `_draw_probe_survival_badge`.

FPS improvement (target 20→30+ FPS at 23 agents) requires Visual Verification — not
testable from the Rust harness.

### Assertion 4 — Hunger distribution passes comfortably
6 of 60 agents (10%) had hunger < 0.30 at tick 4380. Passes ≥2 threshold by 3×.
Hunger decay system and forage loop confirmed active.
