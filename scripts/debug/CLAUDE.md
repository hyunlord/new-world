# scripts/debug/ — Debug Panel System

## Purpose
In-game debug overlay (F3 toggle) and shared data core for simulation inspection.
This is the **sole debug system** — legacy F11/F12 debug has been removed.

## Architecture

```
SimBridge (Rust GDExtension)
  ↓ get_system_perf(), get_debug_summary(), get_config_values(), etc.
DebugDataProvider (shared core)
  ↓ cached data
DebugOverlay (F3 toggle) ←→ 8 Tab Panels
```

**DebugDataProvider** is the single entry point for all debug data.
Panels NEVER call SimBridge directly — always go through the provider.

## Rules

### Data Flow
- **Read-only from SimBridge.** Debug panels never write simulation state directly.
- **Balance tuning uses `push_command()`** — the only write path, going through the standard command queue.
- **Caching is mandatory.** Every provider method has a tick-based cache interval. Never call SimBridge every frame.

### Locale
- **Every visible string must use `Locale.ltr("DEBUG_*")`.**
- Never hardcode Korean or English text. Never use Godot `tr()`.
- Locale keys are in `localization/ko/debug.json` and `localization/en/debug.json`.
- Debug/log-only strings (console prints, variable names) are exempt.

### Performance
- **F3 OFF = zero cost.** No SimBridge calls, no _process logic, no rendering.
- **F3 COMPACT = <0.1ms.** Only reads `get_debug_summary()` every 10 ticks.
- **F3 FULL = <0.5ms.** Reads vary by active tab. Inactive tabs consume nothing.
- `enable_debug_mode(true)` must be called when overlay opens (activates PerfTracker in Rust).
- `enable_debug_mode(false)` must be called when overlay closes.

### Code Style
- All panel classes extend `Control` or `VBoxContainer`.
- Class names: `Debug{Name}Panel` (e.g., `DebugPerfPanel`, `DebugBalanceTuner`).
- File names: `{name}_panel.gd` or `{name}.gd` in `panels/` subdirectory.
- Reusable widgets go in `widgets/` subdirectory.

## File Map

```
scripts/debug/
├── CLAUDE.md                      ← you are here
├── debug_data_provider.gd         ← shared core (SimBridge wrapper + cache)
├── debug_overlay.gd               ← F3 overlay controller (OFF/COMPACT/FULL)
├── panels/
│   ├── perf_panel.gd              ← system ms bars + tick history graph
│   ├── entity_inspector.gd        ← entity ID search + raw Component dump
│   ├── system_panel.gd            ← 74 systems list with priority/R/W
│   ├── event_monitor.gd           ← real-time event stream + category filter
│   ├── balance_tuner.gd           ← config sliders + instant apply
│   ├── world_stats.gd             ← population/climate/settlement dashboard
│   ├── guardrail_monitor.gd       ← 9 guardrails status display
│   └── ffi_monitor.gd             ← FFI bandwidth bars
└── widgets/
    ├── stat_bar.gd                ← horizontal bar (label + value + color)
    ├── mini_graph.gd              ← sparkline from PackedFloat32Array
    ├── collapsible_section.gd     ← toggle arrow + child container
    └── search_filter.gd           ← text input + dropdown filter
```

## SimBridge Debug API (Rust side)

These methods are defined in `rust/crates/sim-bridge/src/debug_api.rs`:

| Method | Returns | Cache Interval |
|--------|---------|:--------------:|
| `enable_debug_mode(bool)` | void | — |
| `get_debug_summary()` | Dictionary | 10 ticks |
| `get_system_perf()` | Dictionary | 60 ticks |
| `get_tick_history()` | PackedFloat32Array | 60 ticks |
| `get_config_values()` | Dictionary | on change |
| `set_config_value(key, value)` | bool | — |
| `get_guardrail_status()` | Array[Dictionary] | 100 ticks |
| `query_entities_by_condition(cond, threshold)` | PackedInt32Array | on demand |
| `dump_entity_raw(id)` | Dictionary | on demand |

## F3 Toggle Modes

```
OFF → (F3) → COMPACT → (F3) → FULL → (F3) → OFF
```

- **OFF**: CanvasLayer hidden. `enable_debug_mode(false)`. Zero cost.
- **COMPACT**: 3-line HUD top-left. Shows FPS, tick ms, population.
- **FULL**: Left panel (30% screen width). 8 tabs. Game continues running.

## Balance Tuner Parameters

14 config keys with min/max/default. All clamped in Rust.
Sliders send `set_config_value(key, value)` → Rust applies immediately.
"Reset All" calls `SimConfig::reset_defaults()` via command.

## Guardrail Monitor

9 guardrails tracked:

| Name | Detect Condition | System |
|------|-----------------|--------|
| Emotion Runaway | 70%+ Fear > 0.9 in region | EmotionRunawayGuard(450) |
| Genetic Collapse | genetic variance < 10% initial | GeneticDiversityGuard(460) |
| Luddite Loop | TechProgress=0, Tradition>0.95 | LudditeLoopGuard(470) |
| Event Flood | EventBus > 1000/type/tick | EventFloodGuard(480) |
| Death Spiral | stress reserve<0.01 + health<0.2 | DeathSpiralGuard(490) |
| Faction Explosion | factions > 5% of population | FactionFormSys internal |
| Permanent Dictatorship | 0 leader changes in 3 generations | LegitimacySys internal |
| Religious War Loop | hostile edges>40% + pop declining | StabilitySys internal |
| Famine Spiral | food_days < 0.5 | ResourceRegenSys+MigrationSys |

## Do NOT

- Do not add game logic here. This is pure read-only inspection + config tuning.
- Do not bypass DebugDataProvider to call SimBridge directly from panels.
- Do not add hardcoded strings. Use `Locale.ltr()`.
- Do not make the overlay pause the game. It must always be non-blocking.
- Do not reference legacy debug code (F11/F12). It has been removed.