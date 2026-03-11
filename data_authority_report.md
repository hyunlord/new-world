# Data Authority Report

## Expected architecture

startup  
-> load RON registry  
-> build registries  
-> inject into ECS

JSON is compatibility-only.

## Current authority decision

- Authoritative path: `RON DataRegistry`
- Compatibility path: legacy JSON loaders for names/personality/tests

## RON authority map

### Runtime

- `rust/crates/sim-bridge/src/lib.rs`
  - `runtime_init()` loads `DataRegistry::load_from_directory(...)`
- `rust/crates/sim-test/src/main.rs`
  - headless runtime also loads `DataRegistry::load_from_directory(...)`

### Registry features already consumed

- materials
- furniture
- recipes
- structures
- actions
- world rules
- temperament rules

Relevant path:
- `rust/crates/sim-data/src/registry.rs`

## Remaining JSON usage

### Runtime compatibility boot

- `rust/crates/sim-bridge/src/runtime_registry.rs`
  - `load_legacy_runtime_bootstrap()`
  - still loads:
    - `load_personality_distribution()`
    - `load_name_cultures()`

### Tests

- `rust/tests/data_loading_test.rs`
  - still validates `sim_data::load_all(...)`

### Legacy JSON bundle API

- `rust/crates/sim-data/src/lib.rs`
  - `DataBundle`
  - `load_all(base_dir)`

## Mixed authority risks

- boot remains partially dependent on JSON compatibility content
- JSON bundle tests still make legacy data shape look first-class
- `config.rs` still contains a large number of hardcoded constants that should eventually move into RON/world rules

## Config hardcode status

- Runtime registry manifest is typed, but many tick rates and system tunings still come from `sim_core::config`
- world rules are present in the registry, but most tuning is not yet derived from them

## Migration recommendation

1. Keep `DataRegistry` as the only authoritative simulation content path.
2. Keep JSON only for:
   - names
   - personality bootstrap
   - legacy tests explicitly marked compatibility
3. Continue migrating config constants into RON/world-rules over time.
4. Rename/document JSON loaders as compatibility surfaces in future tickets.

## Decision

`RON registry authoritative` is the correct and already-mostly-implemented architecture.
