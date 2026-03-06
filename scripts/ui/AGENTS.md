# scripts/ui/AGENTS.md

## Purpose

- UI, rendering, camera, HUD, panels, and editor-facing presentation code.

## Current Boundary

- New or rewritten UI should prefer SimBridge/runtime getters.
- Current repo reality is mixed: some panels are bridge-first, while others still read legacy managers directly.
- Existing editor/setup tooling may still touch pre-runtime world data; do not expand that pattern beyond the current editor paths unless the ticket is a migration.

## Must Follow

- Prefer SimBridge/runtime getters for new UI code and major rewrites.
- When touching manager-backed UI, keep the change local and do not introduce broader direct-manager coupling.
- Treat simulation state as read-only from UI, except through existing command paths or legacy editor/setup flows already present in this tree.
- Keep selection, open/close, and camera concerns in UI-local signals; do not turn them into gameplay logic.
- Use `Locale.ltr()`, `Locale.trf*()`, or `Locale.tr_id()` for user-visible text.
- If a UI change depends on a new bridge getter, verify the Rust API exists before wiring the UI.

## Do Not

- Do not add new gameplay logic here.
- Do not mutate live simulation state directly from ordinary UI code.
- Do not add hardcoded UI strings or `tr()`.
- Do not claim every UI path is already fully bridged when editing legacy panels.

## Verification

- `if rg -n "\\btr\\(" scripts/ui; then echo "Unexpected tr() usage in scripts/ui"; exit 1; fi`
- `cd rust && cargo test -p sim-bridge` when bridge contracts or runtime getter usage change
