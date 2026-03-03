# scripts/ai/ — CLAUDE.md

> ⚠️ **LEGACY DIRECTORY** — AI decision-making has migrated to Rust.
> New AI/behavior logic goes in `rust/crates/sim-systems/`.
> See `rust/crates/sim-systems/CLAUDE.md` for system architecture and priorities.

---

## Status

This directory contains **legacy GDScript AI files** from before the Rust migration.
Retained for reference and GDScript fallback mode.

**DO NOT add new AI logic here.** All behavior, utility AI, and action selection must be implemented in Rust.

---

## What Lives Here (Legacy)

```
ai/
  behavior_system.gd  — Utility AI: action scoring, selection, execution
```

---

## Rust Equivalents

| GDScript (here) | Rust (authoritative) |
|------------------|---------------------|
| `behavior_system.gd` | `rust/crates/sim-systems/src/runtime/cognition.rs` (action selection) |
| Utility AI scoring | `rust/crates/sim-systems/src/runtime/cognition.rs` |
| Pathfinding | `rust/crates/sim-systems/src/pathfinding.rs` |

---

## AI Evolution Roadmap (All in Rust)

```
Phase 0 (current):  Utility AI — weighted scoring (Rust)
Phase 2:            GOAP — goal-oriented action planning (Rust)
Phase 3:            Behavior Trees — complex multi-step behaviors (Rust)
Phase 4:            ML (ONNX) — learned behavior patterns (Rust, ort crate)
Phase 5:            Local LLM — natural language decisions (Rust, llama-cpp-rs)
```

---

## Rules

1. **DO NOT add new AI logic here** — go to Rust `sim-systems`
2. **DO NOT modify** unless fixing GDScript-fallback-only bug
3. **Any changes must be mirrored in Rust** to maintain parity