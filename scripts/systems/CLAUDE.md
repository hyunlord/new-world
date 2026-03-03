# scripts/systems/ — CLAUDE.md

> ⚠️ **LEGACY DIRECTORY** — All simulation systems have migrated to Rust.
> New system logic goes in `rust/crates/sim-systems/`.
> See `rust/crates/sim-systems/CLAUDE.md` for system architecture, priorities, and formulas.

---

## Status

This directory contains **legacy GDScript system files** from before the Rust migration.
They are retained for reference and as GDScript fallback (ComputeBackend GDScript-only mode).

**DO NOT add new systems here.** All new tick-based simulation logic must be implemented in Rust.

---

## What Lives Here (Legacy)

```
systems/
  lifecycle/     — PopulationSystem, MortalitySystem, AgeSystem, FamilySystem
  psychology/    — EmotionSystem, StressSystem, MentalBreakSystem
  economy/       — NeedsSystem, GatheringSystem, ConstructionSystem, JobAssignmentSystem
  social/        — SocialEventSystem, ChronicleSystem, ReputationSystem
  world/         — ResourceRegenSystem, BuildingEffectSystem, MovementSystem, MigrationSystem
  stats/         — StatsRecorderSystem
```

These GDScript systems are used ONLY when:
1. `ComputeBackend` is set to GDScript-only mode (debug/fallback)
2. A specific system has not yet been ported to Rust

---

## Rust Equivalents

| GDScript (here) | Rust (authoritative) |
|------------------|---------------------|
| `lifecycle/*.gd` | `rust/crates/sim-systems/src/runtime/biology.rs` |
| `psychology/*.gd` | `rust/crates/sim-systems/src/runtime/psychology.rs` |
| `economy/*.gd` | `rust/crates/sim-systems/src/runtime/economy.rs`, `needs.rs` |
| `social/*.gd` | `rust/crates/sim-systems/src/runtime/social.rs` |
| `world/*.gd` | `rust/crates/sim-systems/src/runtime/world.rs` |
| `stats/*.gd` | `rust/crates/sim-systems/src/runtime/record.rs` |

---

## Rules

1. **DO NOT add new systems here** — go to `rust/crates/sim-systems/`
2. **DO NOT modify these files** unless fixing a GDScript-fallback-only bug
3. **If modifying**: treat as legacy maintenance, not new development
4. **Any changes here must be mirrored in the Rust equivalent** to maintain parity