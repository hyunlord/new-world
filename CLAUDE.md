# WorldSim — Phase 0

## Project Vision
AI-driven god simulation (WorldBox + Dwarf Fortress + CK3).
Player observes/intervenes as god; AI agents autonomously develop civilization.

## Tech Stack
- Engine: Godot 4.3+ (currently 4.6, Mobile renderer)
- Language: GDScript (Phase 0-1), Rust GDExtension later
- Architecture: Simulation (tick) ≠ Rendering (frame) fully separated
- Events: Event Sourcing — all state changes recorded as events
- AI: Utility AI (Phase 0) → GOAP/BT → ML ONNX → Local LLM
- Data: In-memory (Phase 0) → SQLite → SQLite + DuckDB

## Directory Structure
```
scripts/core/       — SimulationEngine, WorldData, EntityManager, EventLogger, SimulationBus, GameConfig
scripts/ai/         — BehaviorSystem (Utility AI)
scripts/systems/    — NeedsSystem, MovementSystem
scripts/ui/         — WorldRenderer, EntityRenderer, CameraController, HUD
scenes/main/        — Main scene (main.tscn + main.gd)
resources/          — Assets
tests/              — Test scripts
tickets/            — Ticket files (010-150)
scripts/            — gate.ps1, gate.sh (build verification)
```

## Autoloads
- `GameConfig` — constants, biome definitions, simulation parameters
- `SimulationBus` — global signal hub for decoupled communication
- `EventLogger` — subscribes to SimulationBus, stores events in memory

## Coding Conventions
- `class_name` at top of file
- PascalCase classes, snake_case variables/functions
- Signal names: past tense (entity_spawned, tick_completed)
- Type hints required: `var speed: float = 1.0`
- System-to-system communication via SimulationBus (no direct references)
- Use PackedArray for bulk data (performance)
- No magic numbers → use GameConfig constants
- Public functions get `##` doc comments

## Architecture
```
Main._process(delta) → sim_engine.update(delta)
  ├ NeedsSystem   (prio=10, every tick)  — decay hunger/energy/social, starvation
  ├ BehaviorSystem (prio=20, every 5 ticks) — Utility AI action selection
  └ MovementSystem (prio=30, every tick)  — greedy 8-dir movement, arrival effects

SimulationBus (signals) ← all events flow here
EventLogger ← records all events from SimulationBus

WorldData (PackedArrays) — 256×256 tile grid
EntityManager (Dictionary) — entity lifecycle
```

## Phase 0 Checklist
- [x] Fixed timestep tick loop (SimulationEngine)
- [x] Entity data structure (EntityData, EntityManager)
- [x] Utility AI behavior system (BehaviorSystem)
- [x] Event logging (EventLogger + SimulationBus)
- [x] World generation (WorldGenerator + WorldData)
- [x] Rendering (WorldRenderer + EntityRenderer)
- [x] Camera (CameraController)
- [x] HUD (status bar + entity info panel)
- [x] Main scene (wires everything together)
- [x] Gate scripts (gate.ps1, gate.sh)
- [x] Tickets (010-150)

## Known Limitations (Phase 0)
- In-memory only (no persistence to disk beyond JSON)
- Greedy movement (no A* pathfinding)
- O(n) entity queries (no spatial indexing)
- No save/load UI (data structures support it)
- No multiplayer
- Entity cap ~500 before performance concerns
- No diagonal movement cost multiplier
