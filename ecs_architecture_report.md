# ECS Architecture Report

## Current structure

- ECS world: `hecs::World` inside `sim-engine`
- Runtime resources: `SimResources`
- Mutation path: runtime systems in `sim-systems`
- Scheduler entrypoint: `SimEngine::tick()`

## Component layout

Shared components live in:
- `sim-core/src/components/**`

Recent core foundations present:
- `InfluenceEmitter`
- `InfluenceReceiver`
- `Temperament`
- `RoomId`

## Scheduling model

- runtime systems implement the existing `SimSystem` pattern
- registered in deterministic scheduler order from `sim-bridge`
- per-system timing captured by `PerfTracker` when debug mode is enabled

## Query hot-path risks

### `scripts/ui` read amplification

- many UI/render paths still query `get_entity_detail()` or snapshot data every frame or every tick
- this is acceptable for debug/probe surfaces, but remains a cost center

### `sim-bridge` debug/detail fanout

- detail/debug paths build large dictionaries for Godot
- this is not simulation authority leakage, but it is an allocation-heavy inspection path

### Legacy manager duplication

- some Godot renderers and panels still keep legacy manager references even when runtime fallbacks exist
- this duplicates read models and complicates caching

## Allocation hotspots

- `PerfTracker` uses `HashMap<String, ...>` for system timing labels
- debug/perf snapshot export allocates many dictionaries/JSON-like structures
- Godot detail panels frequently rebuild strings and arrays from bridge dictionaries

## Relation/unbounded structure risks

- social systems still use sparse relationship structures in Rust and legacy relationship helpers in GDScript
- long-tail social/history/debug surfaces can grow without a fully unified causal/data policy

## Validation verdict

- ECS runtime ownership is structurally sound.
- biggest remaining architecture debt is not the Rust scheduler itself; it is the hybrid UI/shadow layer around it.
