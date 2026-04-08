# A-9 Phase 4: Agent Constants Runtime Application

## Goal

Replace the abstract `RuleAgentModifier` struct with a concrete `AgentConstants` struct (same pattern as `GlobalConstants`). After this feature, rulesets can override mortality rate, skill XP gain, body stat potential, fertility, lifespan, and movement speed multipliers.

## Current State

- `world_rules.rs`: `WorldRuleset` has `agent_modifiers: Vec<RuleAgentModifier>` (abstract, unused)
- `RuleAgentModifier` struct: `{ system: String, effect: String }` — no runtime consumer
- `SimResources` (engine.rs): has `hunger_decay_rate`, `warmth_decay_rate`, `food_regen_mul`, etc. from GlobalConstants, but NO agent constant fields
- `apply_world_rules()`: handles `global_constants`, ignores `agent_modifiers`
- RON files: `base_rules.ron` has `agent_modifiers: []`, `eternal_winter.ron` has `agent_modifiers: []`
- References to fix: `sim-data/src/lib.rs:31`, `sim-data/src/defs/mod.rs:21`, `sim-engine/src/engine.rs:1108`, `sim-test/src/main.rs:3045`

## Changes Required

### 1. `rust/crates/sim-data/src/defs/world_rules.rs`

Replace `RuleAgentModifier` struct with `AgentConstants`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AgentConstants {
    #[serde(default)]
    pub mortality_mul: Option<f64>,
    #[serde(default)]
    pub skill_xp_mul: Option<f64>,
    #[serde(default)]
    pub body_potential_mul: Option<f64>,
    #[serde(default)]
    pub fertility_mul: Option<f64>,
    #[serde(default)]
    pub lifespan_mul: Option<f64>,
    #[serde(default)]
    pub move_speed_mul: Option<f64>,
}
```

Change `WorldRuleset` field:
- REMOVE: `pub agent_modifiers: Vec<RuleAgentModifier>,`
- ADD: `#[serde(default)] pub agent_constants: Option<AgentConstants>,`

Update the unit test RON string (`parses_world_ruleset_from_ron`): replace `agent_modifiers: []` with `agent_constants: None`.

### 2. `rust/crates/sim-data/src/defs/mod.rs`

Replace `RuleAgentModifier` export with `AgentConstants`:
- REMOVE: `RuleAgentModifier` from the use/re-export list
- ADD: `AgentConstants`

### 3. `rust/crates/sim-data/src/lib.rs`

Replace `RuleAgentModifier` export with `AgentConstants` in re-exports (line ~31).

### 4. `rust/crates/sim-engine/src/engine.rs`

**Part A — SimResources fields** (add after existing GlobalConstants fields `season_mode`):
```rust
/// Mortality rate multiplier (default 1.0).
pub mortality_mul: f64,
/// Global skill XP gain multiplier (default 1.0).
pub skill_xp_mul: f64,
/// Body stat potential multiplier (default 1.0).
pub body_potential_mul: f64,
/// Fertility/birth rate multiplier (default 1.0).
pub fertility_mul: f64,
/// Lifespan multiplier for Siler model (default 1.0).
pub lifespan_mul: f64,
/// Movement speed multiplier (default 1.0).
pub move_speed_mul: f64,
```

**Part B — SimResources::new() initialization** (add after `season_mode: "default".to_string()`):
```rust
mortality_mul: 1.0,
skill_xp_mul: 1.0,
body_potential_mul: 1.0,
fertility_mul: 1.0,
lifespan_mul: 1.0,
move_speed_mul: 1.0,
```

**Part C — apply_world_rules()** (add after the `global_constants` block):
```rust
if let Some(ref agent) = rules.agent_constants {
    if let Some(mul) = agent.mortality_mul {
        self.mortality_mul = mul.max(0.0);
    }
    if let Some(mul) = agent.skill_xp_mul {
        self.skill_xp_mul = mul.max(0.0);
    }
    if let Some(mul) = agent.body_potential_mul {
        self.body_potential_mul = mul.max(0.0);
    }
    if let Some(mul) = agent.fertility_mul {
        self.fertility_mul = mul.clamp(0.0, 10.0);
    }
    if let Some(mul) = agent.lifespan_mul {
        self.lifespan_mul = mul.max(0.1);
    }
    if let Some(mul) = agent.move_speed_mul {
        self.move_speed_mul = mul.clamp(0.1, 5.0);
    }
    info!(
        "[WorldRules] agent constants: mortality={:.2}, skill_xp={:.2}, lifespan={:.2}, fertility={:.2}",
        self.mortality_mul, self.skill_xp_mul, self.lifespan_mul, self.fertility_mul
    );
}
```

**Part D — fix broken references in engine.rs** (line ~1108):
- Change `agent_modifiers: Vec::new()` → `agent_constants: None`

### 5. `rust/crates/sim-test/src/main.rs`

Line ~3045: change `agent_modifiers: vec![]` → `agent_constants: None`

### 6. RON files

**`rust/crates/sim-data/data/world_rules/base_rules.ron`**:
- Change `agent_modifiers: [],` → `agent_constants: None,`

**`rust/crates/sim-data/data/world_rules/scenarios/eternal_winter.ron`**:
- Change `agent_modifiers: [],` to:
```ron
agent_constants: Some(AgentConstants(
    mortality_mul: Some(1.3),
    skill_xp_mul: Some(1.5),
    body_potential_mul: None,
    fertility_mul: Some(0.7),
    lifespan_mul: Some(0.8),
    move_speed_mul: None,
)),
```

## Harness Test

Add to `rust/crates/sim-test/src/main.rs`:

```rust
#[test]
fn harness_agent_constants_defaults() {
    let engine = make_stage1_engine(42, 20);
    let resources = engine.resources();

    assert!((resources.mortality_mul - 1.0).abs() < 1e-9, "mortality_mul default should be 1.0");
    assert!((resources.skill_xp_mul - 1.0).abs() < 1e-9, "skill_xp_mul default should be 1.0");
    assert!((resources.body_potential_mul - 1.0).abs() < 1e-9, "body_potential_mul default should be 1.0");
    assert!((resources.fertility_mul - 1.0).abs() < 1e-9, "fertility_mul default should be 1.0");
    assert!((resources.lifespan_mul - 1.0).abs() < 1e-9, "lifespan_mul default should be 1.0");
    assert!((resources.move_speed_mul - 1.0).abs() < 1e-9, "move_speed_mul default should be 1.0");
}
```

## Gate

```bash
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings
```

## Scope Limits

- `move_speed_mul`: stored in SimResources only — actual steering integration is out of scope
- `body_potential_mul`: stored only — entity spawner integration is out of scope  
- `fertility_mul`: stored only — birth system integration is out of scope if the integration point is not obvious
- `mortality_mul` and `skill_xp_mul`: stored only — actual system integration is out of scope for this ticket

All 6 fields must default to 1.0 and be stored in SimResources. RON loading and apply_world_rules() application is required. System-level usage is out of scope.
