# scripts/debug/AGENTS.md

## Purpose

- Debug overlay, debug data provider, and read-only simulation inspection tools.

## Current Boundary

- `DebugDataProvider` is the single shared access layer for runtime debug data.
- Debug panels are read-only except for config-tuning paths that already go through commands.

## Must Follow

- Read runtime data through `DebugDataProvider`, not direct SimBridge calls from panels/widgets.
- Keep caching for expensive debug reads.
- Keep the overlay non-blocking and zero-cost when disabled.
- Use `Locale.*` for visible strings.
- Use standard command paths for balance/config writes.
- Treat RuleHistory, CausalLog, temperament state, and faith/oracle traces as read-only diagnostic surfaces.

## Do Not

- Do not add gameplay logic here.
- Do not call SimBridge directly from panels or widgets.
- Do not hardcode UI strings or use `tr()`.
- Do not make the overlay pause or own the simulation loop.
- Do not add debug paths that mutate World Rules or oracle outcomes outside approved command/config routes.

## Verification

- `if rg -n "\\btr\\(" scripts/debug; then echo "Unexpected tr() usage in scripts/debug"; exit 1; fi`
- `if rg -n "SimBridge" scripts/debug/panels scripts/debug/widgets; then echo "Panels/widgets must not call SimBridge directly"; exit 1; fi`
