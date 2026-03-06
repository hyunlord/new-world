# data/AGENTS.md

## Purpose

- JSON content definitions consumed by Rust and, in limited cases, GDScript UI.

## Current Boundary

- Files here define content and schema only. They do not contain gameplay logic.
- Rust `sim-data` is the primary parser and schema enforcer.
- Some UI code may read these files for display, so wire-format changes affect both Rust and GDScript.

## Must Follow

- Keep content IDs in `snake_case`.
- Keep locale keys in `UPPER_SNAKE_CASE`.
- Store locale keys, not user-visible English or Korean text.
- When schema changes, update Rust `sim-data` consumers in the same task.
- Check whether any direct GDScript readers need updates before changing file shape.
- Keep numbers and field types compatible with `serde_json` and Rust-side expectations.

## Do Not

- Do not put user-visible prose in JSON files.
- Do not change key names silently.
- Do not add behavior that belongs in Rust systems.
- Do not leave `data/` and `sim-data` out of sync.

## Verification

- `cd rust && cargo test -p sim-data`
- `cd rust && cargo test --test data_loading_test`
