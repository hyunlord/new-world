# Performance Baseline

## Available instrumentation

- `sim-engine::PerfTracker`
  - per-system last tick microseconds
  - rolling tick history
- `sim-bridge::debug_api`
  - exposes current tick cost and per-system timing when debug mode is enabled

## Current baseline status

There is no dedicated repository-wide benchmark harness yet. The current baseline is defined by:

- successful workspace build/test on the existing runtime path
- per-system timing availability in debug mode
- code-structure inspection of hot paths

## Likely hot paths

### Rust runtime

- `sim-engine::tick()`
- dense runtime registration manifest in `sim-bridge`
- social / psychology / economy systems with broad ECS queries

### Bridge/debug

- `get_entity_detail()`
- debug snapshot export
- per-frame UI polling of detail/snapshot dictionaries

### Godot shell

- renderer redraw every frame
- HUD and detail panel dictionary/string churn
- minimap regeneration path

## Baseline recommendation

- Use `PerfTracker` as the authoritative runtime cost surface.
- Add a dedicated headless benchmark harness in a follow-up ticket instead of guessing from debug UI usage.

## Current conclusion

- the repository already contains the right low-level instrumentation hooks
- the biggest immediate performance risk is hybrid presentation/state duplication, not missing timing instrumentation
