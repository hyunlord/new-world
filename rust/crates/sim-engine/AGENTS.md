# rust/crates/sim-engine/AGENTS.md

## Purpose

- Tick orchestration, event delivery, deferred commands, and lightweight engine snapshots.

## Current Boundary

- This crate owns engine flow and shared runtime plumbing.
- Gameplay rules belong in `sim-systems`, not here.
- The current snapshot type is `EngineSnapshot`, a lightweight diagnostic/save snapshot, not a full world-state dump.

## Must Follow

- Keep tick ordering, event flow, and deferred-mutation semantics explicit.
- Treat `GameEvent`, `CommandQueue`, and `EngineSnapshot` changes as interface changes.
- If a UI-visible event shape changes, update `sim-bridge` in the same task.
- Keep docs and comments aligned with actual `EngineSnapshot` behavior.
- Preserve determinism in engine flow.

## Do Not

- Do not add domain/business logic here.
- Do not import from `sim-bridge`.
- Do not reintroduce stale `FrameSnapshot` or old player-command documentation.
- Do not change event or command ordering casually.

## Verification

- `cd rust && cargo test -p sim-engine`
- `cd rust && cargo clippy -p sim-engine -- -D warnings`
