# scripts/systems/AGENTS.md

## Purpose

- Legacy GDScript systems retained for fallback/reference behavior.

## Current Boundary

- Authoritative simulation systems live in Rust.
- This directory is for fallback maintenance and parity reference only.

## Must Follow

- Treat edits here as legacy maintenance, not new development.
- If you change fallback behavior here, check the Rust equivalent in the same task or report the parity risk.
- Keep changes tightly scoped to the requested fallback issue.
- Preserve parity notes for material, building, temperament, and world-rule behavior when touching fallback code.

## Do Not

- Do not add new systems here.
- Do not extend this tree with new gameplay logic.
- Do not treat these files as the source of truth for simulation behavior.
- Do not implement authoritative building 2-layer, World Rules, or temperament pipelines here.

## Verification

- `cd rust && cargo test -p sim-systems`
