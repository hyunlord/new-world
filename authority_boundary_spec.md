# Authority Boundary Spec

## Required Architecture

```text
Godot
  ↓
sim-bridge
  ↓
Rust ECS simulation
```

## Allowed Godot Responsibilities
- load scenes and UI shell
- gather startup/setup input
- render snapshots and detail queries
- send player commands through `SimulationBusV2` / `sim_bridge`
- observe runtime events and history feeds
- maintain non-authoritative presentation caches

## Forbidden Godot Responsibilities
- registering authoritative simulation systems
- clearing and rebuilding the runtime registry
- owning the simulation tick loop
- mutating authoritative ECS state directly
- applying needs, movement, resource, social, or behavior updates as source of truth

## Required Rust Responsibilities
- load authoritative RON registry during runtime init
- build the runtime system manifest
- own scheduler order and frame stepping
- own ECS world bootstrap
- own simulation state mutation

## Boot Sequence Contract
1. Godot shell starts.
2. `sim_bridge` resolves the native runtime.
3. Rust runtime initializes and loads authoritative data.
4. Rust registers the default runtime system manifest.
5. Godot validates registry truth.
6. Godot submits bootstrap payload.
7. Rust boots ECS world.
8. Godot only requests state and renders it.

## Residual Hybrid Reality
The repository still contains legacy shadow/bootstrap managers in GDScript.

These are allowed temporarily only if they are:
- not part of the active runtime scheduler
- not the source of truth for per-frame simulation state
- treated as observers, caches, or bootstrap helpers

They remain cleanup targets for later tickets, but they do not redefine the authority boundary for WS-REF-004A.
