# rust/crates/sim-data/ — CLAUDE.md

> JSON data loading and definition structs. Uses serde_json for type-safe parsing.
> Read-only at runtime — data is loaded once at startup.

---

## Module Map

```
sim-data/src/
  lib.rs                   — DataStore struct, re-exports
  loader.rs                — File I/O, JSON loading orchestration
  error.rs                 — DataError type
  species.rs               — SpeciesDef (lifespan, fertility, body template)
  trait_defs.rs            — TraitDef (187+ traits, activation conditions)
  stressor_events.rs       — StressorDef (severity, type, triggers)
  emotion_presets.rs       — EmotionPreset (base emotion configs)
  mental_breaks.rs         — MentalBreakDef (10 types, thresholds)
  mortality.rs             — MortalityCurve (Siler model parameters)
  coping.rs                — CopingStrategyDef (15 types)
  developmental_stages.rs  — DevelopmentalStageDef (child development)
  attachment_config.rs     — AttachmentTypeDef (secure/anxious/avoidant/fearful)
  tech.rs                  — TechDef (169 technologies, prerequisites, effects)
  occupations.rs           — OccupationDef (jobs, skill requirements)
  value_events.rs          — ValueEventDef (value-shifting events)
```

---

## DataStore

```rust
pub struct DataStore {
    pub species: Vec<SpeciesDef>,
    pub traits: Vec<TraitDef>,
    pub stressors: Vec<StressorDef>,
    pub emotions: Vec<EmotionPreset>,
    pub mental_breaks: Vec<MentalBreakDef>,
    pub mortality: Vec<MortalityCurve>,
    pub coping: Vec<CopingStrategyDef>,
    pub tech: Vec<TechDef>,
    pub occupations: Vec<OccupationDef>,
    // ...
}

impl DataStore {
    pub fn load_from_dir(data_path: &Path) -> Result<Self, DataError> { ... }
}
```

Systems receive `&DataStore` (read-only reference). Data never changes after loading.

---

## Serde Pattern

Every data struct uses serde for JSON deserialization:

```rust
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct TraitDef {
    pub id: String,
    pub display_name_key: String,   // Localization key (e.g., "TRAIT_BRAVE")
    pub category: String,
    pub source_facets: Vec<FacetCondition>,
    pub effects: HashMap<String, f64>,
    #[serde(default)]
    pub conflicts_with: Vec<String>,
}
```

### Rules
1. **All fields have serde defaults** where reasonable (`#[serde(default)]`)
2. **display_name_key is a locale key**, not actual text
3. **`id` is always `String`, lowercase snake_case**
4. **Numeric values: f64** (0.0~1.0 for percentages)
5. **No Godot types** — pure Rust/serde

---

## JSON Schema Parity

Data files live in `data/` at the project root. Rust serde structs must exactly match JSON key names.

**When JSON schema changes:**
1. Update the Rust struct
2. Run `cargo test -p sim-data` to verify all JSON files still parse
3. Update `data/CLAUDE.md` if format changed

---

## Testing

```bash
# Verify all JSON files parse correctly
cd rust && cargo test -p sim-data

# Integration test (loads from actual data/ directory)
cd rust && cargo test --test data_loading_test
```

Every data module should have:
- Unit test with inline JSON
- Integration test against actual `data/` files

---

## Do NOT

- Modify data at runtime — DataStore is immutable after loading
- Put user-visible text in data structs — only locale keys
- Use Godot types (this crate is pure Rust)
- Skip `#[serde(default)]` on optional fields
- Add data loading that doesn't go through serde