# data/AGENTS.md

## Purpose

- RON-first content definitions and migration-era content assets consumed by Rust `sim-data` and, in limited cases, tooling/UI.
- v3.1 content families include materials, recipes, structures, world rules, and temperament rules.

## Current Boundary

- Files here define content and schema only. They do not contain gameplay logic.
- Target v3.1 content is material/recipe/structure/action/furniture/world-rule/temperament data expressed through RON schemas.
- Rust `sim-data` is the primary parser and schema enforcer.
- Some UI code may read these files for display, so wire-format changes affect both Rust and GDScript.

## Must Follow

- Keep content IDs in `snake_case`.
- Keep tags and recipe selectors data-driven and lowercase.
- Keep locale keys in `UPPER_SNAKE_CASE`.
- Store locale keys, not user-visible English or Korean text.
- When schema changes, update Rust `sim-data` consumers in the same task.
- Check whether any direct GDScript readers need updates before changing file shape.
- Keep numbers and field types compatible with serde-based Rust-side expectations.
- Prefer tag+threshold recipe inputs over direct material-ID coupling when the schema supports both.
- Treat new content as `.ron`-first; legacy JSON is migration baggage, not the v3 target pattern.
- Keep World Rules content declarative: slots, composition, merge priority, and on-action patches are data, not logic.
- Keep temperament shift rules declarative and event-driven; do not encode personality outcomes in prose.

## Do Not

- Do not put user-visible prose in content files.
- Do not change key names silently.
- Do not add behavior that belongs in Rust systems.
- Do not leave `data/` and `sim-data` out of sync.
- Do not reintroduce JSON-only assumptions into new architecture docs or schemas.
- Do not encode direct entity references in recipe or world-rule selectors when a tag or typed selector exists.

## Verification

- `cd rust && cargo test -p sim-data`
- `cd rust && cargo test --test data_loading_test`
