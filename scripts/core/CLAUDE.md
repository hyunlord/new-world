# scripts/core/ — CLAUDE.md

> GDScript-side core infrastructure. Now primarily serves as the **UI relay layer**.
> All simulation state and logic has migrated to Rust (see `rust/` CLAUDE.md files).
> This layer provides: Locale (i18n), SimulationBus (UI event relay), GDScript-side config mirror.

---

## Role Change: Before vs After Rust Migration

| Component | Before (GDScript) | After (Rust-first) |
|-----------|-------------------|---------------------|
| EntityData | Authoritative state | **Deleted** — state in Rust ECS |
| EntityManager | Entity lifecycle | **Thin wrapper** — delegates to SimBridge |
| SimulationEngine | Tick loop | **Deleted** — Rust sim-engine owns tick |
| SimulationBus | System-to-system comms | **UI relay only** — receives from SimBridge |
| GameConfig | Authoritative constants | **Mirror** — Rust config is authoritative |
| Locale | i18n | **Unchanged** — still GDScript Autoload |
| SaveManager | Save/Load | **Delegates to Rust** — Rust serializes state |

---

## What Still Lives Here

```
core/
  simulation/
    simulation_bus.gd       — UI event relay (receives events from SimBridge)
    simulation_bus_v2.gd    — Extended UI event relay
    game_config.gd          — Constants mirror for UI display
    game_calendar.gd        — Date formatting for UI
  locale.gd                 — Locale Autoload (i18n) — UNCHANGED
  save_manager.gd           — Delegates to Rust for serialization
  event_logger.gd           — Subscribes to SimulationBus for event history
  compute_backend.gd        — Runtime mode routing (Rust/GDScript fallback)
```

---

## SimulationBus: UI Event Relay

SimulationBus is **no longer the primary event system**. The Rust EventBus handles simulation events.

SimulationBus now:
1. **Receives** events from SimBridge (Rust → GDScript)
2. **Re-emits** as Godot signals for UI panels to listen to
3. **Emits** UI-only events (camera, selection, panel state)

```gdscript
# ✅ CORRECT: UI listens to SimulationBus
SimulationBus.connect("entity_died", _on_entity_died)

# ❌ WRONG: UI emits simulation events
SimulationBus.emit_signal("entity_spawned", id)  # This should come from Rust only
```

**Exception**: UI-only signals (not simulation state) can still originate from GDScript:
- `camera_moved`, `entity_selected`, `panel_opened`, `speed_changed`

---

## GameConfig: Mirror Only

`game_config.gd` now mirrors a subset of Rust `sim-core/src/config.rs` constants.

**Authoritative values live in Rust.** GDScript GameConfig is for:
- UI display (e.g., "Max carry: 10" shown in tooltip)
- UI calculations (e.g., font sizes, panel dimensions — not simulation)
- Constants that only UI needs

When changing a simulation constant:
1. Change `rust/crates/sim-core/src/config.rs` first
2. If UI needs the value, update `game_config.gd` to match
3. If UI doesn't need it, don't add it to GameConfig

---

## Locale: Unchanged

Locale remains the authoritative i18n system for all user-visible text.

- `Locale.ltr("KEY")` — simple text lookup
- `Locale.trf1("KEY", "param", value)` — formatted text
- `Locale.tr_id("PREFIX", id)` — dynamic key composition

**Rule**: Every user-visible string in GDScript uses Locale. No exceptions.

See `.claude/skills/worldsim-code/SKILL.md` for complete localization protocol.

---

## Do NOT

- Put simulation logic in GDScript (all sim logic is Rust)
- Emit simulation events from SimulationBus (only Rust EventBus → SimBridge → SimulationBus)
- Treat GameConfig as authoritative (Rust config.rs is source of truth)
- Directly access Rust entity data (go through SimBridge)
- Modify entity state from GDScript