# Simulation Ownership Cutover Map

| Responsibility | Owner | Evidence |
|---|---|---|
| Registry initialization | Rust | `runtime_init()` + `runtime_register_default_systems()` through `sim-bridge` |
| Simulation tick | Rust | `runtime_tick_frame()` called from `SimulationEngine.update()` |
| Agent state mutation | Rust | Runtime systems in `sim-systems`; Godot only reads snapshots/detail |
| World state mutation | Rust during live runtime | ECS world lives behind `sim-bridge`; Godot setup tools mutate pre-bootstrap editor state only |
| Needs progression | Rust | `NeedsRuntimeSystem` and runtime detail diagnostics |
| Movement resolution | Rust | Runtime tick + snapshot export |
| Resource consumption / gather results | Rust | Runtime systems update stockpiles/needs; Godot reads overlays/detail |
| Social interaction | Rust | Runtime systems and bridge detail/tab queries |
| Danger response | Rust | Runtime systems / snapshots / detail getters |
| Settlement evaluation | Rust | Runtime world summary + bridge getters |
| UI rendering | Godot | `scripts/ui/**`, `scripts/rendering/**` |
| Player commands | Godot -> bridge -> Rust | `runtime_apply_commands_v2` and queue paths |
| Observer/archive debug state | Godot | `ChronicleSystem`, debug overlay, legacy shadow archives |

## Cutover Status
- `simulation_tick_owner = rust_only`
- `registry_owner = rust_only`
- `active_gameplay_mutation_owner = rust_only`
- Remaining non-authoritative residue:
  - setup/editor world mutation before bootstrap
  - shadow managers used as UI fallback
  - archive/debug helper state
