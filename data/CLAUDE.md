# data/ — CLAUDE.md

> Data definitions: species, traits, emotions, stressors, buildings, skills, tech.
> All game content is defined here as JSON files.
> Both Rust (sim-data crate via serde) and GDScript (for UI display) read these files.

---

## Principle: Data-Driven Design

Game content (what exists) is separated from game logic (how it behaves).

```
data/           ← defines WHAT (species, traits, buildings, skills, tech)
rust/           ← defines HOW (Rust systems that process data)
scripts/ui/     ← displays data to the player
```

**Adding new content** (a new trait, building, skill) should NEVER require code changes.

---

## Dual Consumer: Rust + GDScript

Data JSON files are read by TWO consumers:
1. **Rust (sim-data crate)**: Loads via serde_json at startup. Structs must match JSON keys exactly.
2. **GDScript (UI)**: May read certain data files for display (e.g., trait tooltips, building descriptions).

**When modifying JSON format:**
1. Update the JSON file
2. Update the Rust serde struct in `rust/crates/sim-data/`
3. Run `cargo test -p sim-data` to verify parsing
4. Check if any GDScript reads the file directly and update accordingly

---

## JSON Format Rules

### File Structure
```json
{
  "version": "1.0",
  "description": "Brief description",
  "data": [
    {
      "id": "unique_snake_case_id",
      "display_name_key": "LOCALE_KEY_FOR_NAME",
      ...
    }
  ]
}
```

### Rules
- File names: `snake_case.json`
- IDs: `snake_case` (e.g., `"bronze_sword"`, `"crop_farming"`)
- Localization keys: `UPPER_SNAKE_CASE` (e.g., `"TRAIT_BRAVE"`)
- **Never put user-visible text in data files** — only locale keys
- Percentages: 0.0~1.0 (not 0~100)
- All numeric values compatible with f64

### Validation
```bash
# Rust-side validation (serde parsing)
cd rust && cargo test -p sim-data

# Full integration test
cd rust && cargo test --test data_loading_test
```

---

## Do NOT

- Put user-visible text in data files (only locale keys)
- Change JSON key names without updating Rust serde structs
- Use types incompatible with serde_json (keep to strings, numbers, booleans, arrays, objects)
- Add data files without corresponding Rust loader