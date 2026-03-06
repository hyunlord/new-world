# scripts/core/AGENTS.md

## Purpose

- GDScript-side core infrastructure: locale, buses, runtime wrappers, save/load helpers, config mirror, and legacy/shadow state holders.

## Current Boundary

- Target architecture is Rust-first.
- Current repo reality is hybrid: this tree still contains relay code plus legacy/shadow state and fallback-era managers.
- Unless a ticket is an explicit migration, preserve current Rust/GDScript parity instead of trying to finish the migration opportunistically.

## Must Follow

- Put new authoritative simulation logic in Rust, not here.
- When touching manager/wrapper paths, preserve current bridge and shadow-state parity unless the ticket explicitly migrates ownership.
- Treat `game_config.gd` as a mirror, not the authoritative source of simulation constants.
- Route runtime-facing reads/writes through the existing SimBridge and SimulationBus patterns already used here.
- Keep user-visible strings on `Locale.*`.
- Keep save/load comments aligned with actual runtime behavior, not intended future behavior.

## Do Not

- Do not claim `EntityManager`, `SimulationEngine`, or other legacy pieces are already gone when they still exist in code.
- Do not delete or replace managers without a migration ticket.
- Do not add new authoritative gameplay systems here.
- Do not use `tr()`.

## Verification

- `if rg -n "\\btr\\(" scripts/core; then echo "Unexpected tr() usage in scripts/core"; exit 1; fi`
- `cd rust && cargo test -p sim-bridge` when bridge contracts or runtime wrapper behavior change
