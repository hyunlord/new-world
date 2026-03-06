# scripts/ai/AGENTS.md

## Purpose

- Legacy GDScript AI/fallback reference code.

## Current Boundary

- Authoritative AI behavior lives in Rust.
- This directory exists for fallback/reference work only.

## Must Follow

- Treat changes here as legacy maintenance only.
- If you fix fallback behavior here, check the Rust equivalent in the same task or report the parity risk.
- Keep edits narrowly scoped to the fallback issue being addressed.

## Do Not

- Do not add new AI features here.
- Do not make this directory authoritative again.
- Do not do broad rewrites when the ticket is not explicitly about fallback maintenance.

## Verification

- `cd rust && cargo test -p sim-systems`
