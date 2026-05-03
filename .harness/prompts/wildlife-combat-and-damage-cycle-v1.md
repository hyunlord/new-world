# wildlife-combat-and-damage-cycle-v1 — Implementation Prompt

## Feature Summary

Implement Phase A3 of the wildlife system: bidirectional combat damage cycle.
Wildlife attacks adjacent agents (DamagePart via EffectQueue); agents with Fight action
deal damage back (DamageWildlife via EffectQueue). WildlifeAttackSystem priority 23,
cooldown 30 ticks, range 1.5 tiles.

---

## Section 1: Implementation Intent

Wildlife Phase A2 (wildlife-threat-detection-and-flee-v1) established danger emission +
Flee behavior. Phase A3 completes the combat loop:

```
WildlifeAttackSystem (priority 23, interval 1):
  → query wildlife + agents within WILDLIFE_ATTACK_RANGE (1.5 tiles)
  → cooldown guard: tick.saturating_sub(last_attack_tick) < WILDLIFE_ATTACK_COOLDOWN (30)
  → push EffectPrimitive::DamagePart { part_idx: 255, severity: kind.attack_damage(), flags_bits: 0x01, bleed_rate: 2 }
  → source.kind = "{species}_attack" (for causal log)
  → update wildlife.last_attack_tick

EffectApplySystem (priority 50):
  → DamagePart: apply_injury() + push to causal_log with cause.kind = source.kind
  → DamageWildlife: resolve entity, reduce current_hp by damage, clamp to 0.0

BehaviorRuntimeSystem (cognition.rs):
  → Fight scoring: cornered (safety < 0.25 && energy < FIGHT_MIN_ENERGY) → 0.70
  → Fight scoring: aggressive (NS >= 0.60 && safety < 0.30 && energy >= FIGHT_MIN_ENERGY) → 0.50
  → Force-Flee: safety < 0.20 && energy >= BEHAVIOR_FORCE_REST_ENERGY_MAX → immediate Flee
```

Pre-investigation confirmed:
- `EffectPrimitive::DamagePart` and `DamageWildlife` exist in sim-core/src/effect.rs
- `EffectApplySystem` already handles DamagePart; DamageWildlife arm needed
- `Wildlife.last_attack_tick: u32` field exists
- `Wildlife.is_alive()` checks current_hp > 0.0
- `WildlifeAttackSystem` must be new file in sim-systems/src/runtime/

---

## Section 2: What to Build

### 2A. New file: `rust/crates/sim-systems/src/runtime/wildlife_attack.rs`
- `WildlifeAttackSystem` struct (priority: u32, tick_interval: u64)
- Two-pass ECS pattern: read-only snapshot of humans → mutating pass (push effects + update cooldown)
- Query wildlife: `(&Wildlife, &Position)`; query agents: `(&Identity, &Position, &BodyHealth)` where species_id == "human"
- Cooldown: `tick_u32.saturating_sub(wildlife.last_attack_tick) < config::WILDLIFE_ATTACK_COOLDOWN`
- Range: euclidean distance <= `config::WILDLIFE_ATTACK_RANGE`

### 2B. Edit: `rust/crates/sim-systems/src/runtime/effect_apply.rs`
- Add `DamageWildlife` arm: resolve entity ID → get mut Wildlife → current_hp = (current_hp - damage).max(0.0)
- Ensure DamagePart arm pushes to causal_log with `cause.kind = source.kind`

### 2C. Edit: `rust/crates/sim-systems/src/runtime/mod.rs`
- Add `pub mod wildlife_attack;`
- Re-export `WildlifeAttackSystem`

### 2D. Edit: `rust/crates/sim-bridge/src/runtime_system.rs`
- Register `WildlifeAttackSystem::new(23, 1)` in `DEFAULT_RUNTIME_SYSTEMS`

### 2E. Edit: `rust/crates/sim-systems/src/runtime/cognition.rs`
- Fight scoring section: add cornered + aggressive conditions as described above
- wildlife_positions snapshot pre-computed from `(&Wildlife, &Position)` query
- nearest_wildlife_dist: min euclidean distance to any alive wildlife

### 2F. Edit: `rust/crates/sim-core/src/config.rs`
- `WILDLIFE_ATTACK_COOLDOWN: u32 = 30`
- `WILDLIFE_ATTACK_RANGE: f64 = 1.5`
- `FIGHT_MIN_ENERGY: f32 = 0.30`

### 2G. Edit: `rust/crates/sim-test/src/main.rs`
- Module `mod harness_a3_wildlife_combat` with 8 tests (see Section 5)

---

## Section 3: How to Implement

### Step 1: Config constants (sim-core/src/config.rs)
Add at end of config:
```rust
pub const WILDLIFE_ATTACK_COOLDOWN: u32 = 30;
pub const WILDLIFE_ATTACK_RANGE: f64 = 1.5;
pub const FIGHT_MIN_ENERGY: f32 = 0.30;
```

### Step 2: WildlifeAttackSystem (new file)
```rust
// Two-pass pattern to avoid ECS borrow conflicts:
// Pass 1 (read-only): snapshot (wildlife_entity, target_entity_id, damage, kind_str)
// Pass 2 (mutating): push DamagePart to effect_queue, update last_attack_tick
```

### Step 3: EffectApplySystem DamageWildlife arm
```rust
EffectPrimitive::DamageWildlife { entity_id, damage } => {
    if let Some(e) = world.find_entity(entity_id) {
        if let Ok(mut w) = world.get::<&mut Wildlife>(e) {
            w.current_hp = (w.current_hp - damage as f64).max(0.0);
        }
    }
    // push causal log entry
}
```

### Step 4: cognition.rs Fight scoring
```rust
// Pre-compute nearest wildlife distance before agent loop
let wildlife_positions: Vec<(f64, f64)> = world
    .query::<(&Wildlife, &Position)>()
    .iter()
    .filter(|(_, (w, _))| w.is_alive())
    .map(|(_, (_, p))| (p.x, p.y))
    .collect();

// In agent scoring loop:
let nearest_dist = wildlife_positions.iter()
    .map(|(wx, wy)| ((wx - px).powi(2) + (wy - py).powi(2)).sqrt())
    .fold(f64::INFINITY, f64::min);

if nearest_dist <= 2.0 {
    let cornered = safety < 0.25 && energy < config::FIGHT_MIN_ENERGY;
    let aggressive = ns >= 0.60 && safety < 0.30 && energy >= config::FIGHT_MIN_ENERGY;
    if cornered { behavior_score_add(&mut scores, ActionType::Fight, 0.70); }
    else if aggressive { behavior_score_add(&mut scores, ActionType::Fight, 0.50); }
}
```

### Step 5: Register in DEFAULT_RUNTIME_SYSTEMS (sim-bridge)
Priority 23, tick_interval 1.

---

## Section 4: Harness Tests (8 tests)

All in `mod harness_a3_wildlife_combat` in sim-test/src/main.rs.

Helper: `make_combat_engine()` — registers WildlifeAttackSystem(23,1) + EffectApplySystem(50,1), no movement system.
Helper: `spawn_minimal_human(engine, x, y)` — spawns Identity::default(), Position::new(x,y), BodyHealth::default(), Age::default().

1. **A3-1** (Type B): `wolf.attack_damage() > 0 && bear.attack_damage() > wolf.attack_damage()`
2. **A3-2** (Type B): wolf+human at (32,32), run 50 ticks, agent `aggregate_hp < 1.0`
3. **A3-3** (Type A): `run_ticks(31)` → `last_attack_tick == 30`; `run_ticks(1)` → unchanged
4. **A3-4** (Type A): `ActionType::Fight as u8 == 13`; `wildlife_attack_system` in registry
5. **A3-5** (Type B): agent safety=0.15, energy=0.20, wolf at same tile → Flee (Force-Flee triggers)
6. **A3-6** (Type A): push DamageWildlife to effect_queue, run 1 tick → bear.current_hp < max_hp
7. **A3-7** (Type A): `wolf.is_alive()` true at full HP, false at 0 HP
8. **A3-8** (Type C): make_combat_engine, wolf+human at (32,32), run 50 ticks → causal_log has ≥1 "_attack" entry

---

## Section 5: Verification

```bash
cd rust && cargo test -p sim-test harness_a3 -- --nocapture
# Expected: 8 tests pass

cd rust && cargo test --workspace
cd rust && cargo clippy --workspace -- -D warnings
```

Gate: all A3 harness tests pass, clippy clean.
