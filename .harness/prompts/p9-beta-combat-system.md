# P9-β — CombatSystem + CausalEvent combat chain + AgentDecisionSystem 7th cascade arm

> Lane: `--full` (sim-core `.rs` + sim-systems `.rs` edits — new system module + 2 new
> CausalEvent variants + DecisionReason/MemoryRecallTrigger extensions + decision/memory
> system modifications force full lane by hook detection).
> Scope: Second sub-stage of V7 Phase 9 (Combat System). Lands the full runtime combat
> causal chain: CombatSystem (priority 137), CausalEvent::CombatStarted/CombatCompleted,
> DecisionReason::CombatReason, MemoryRecallTrigger::CombatContext, AgentDecisionSystem
> 7th cascade arm, MemorySystem encoding extension, SimResources combat fields.
> Governance: v3.3.15. Visual: backend only (no `.gd`/`.gdshader`/`.tscn`/`.tres`,
> no `scripts/` or `scenes/` path) — Pipeline VLM no-godot-scope auto credit expected.

---

## Section 1 — Implementation Intent

Phase 9-α closed at `8768904a` (adjusted 105/100, APPROVE). Phase 9-β executes the
full runtime combat causal chain per `.harness/plans/phase9.md §β` locked decisions
P9β-1 through P9β-10.

Phase 9-β has **nine deliverables** across sim-core, sim-engine, sim-systems:

1. `event.rs` — extend `DecisionReason` (+1), `MemoryRecallTrigger` (+1), `CausalEvent`
   (+2), and all four accessor impls.
2. `sim-engine/lib.rs` — add `combat_pairs` + `combat_progress` to `SimResources`.
3. `sim-systems/runtime/mod.rs` — add `pub mod combat;`.
4. `sim-systems/runtime/combat/mod.rs` — new module (constants + re-export).
5. `sim-systems/runtime/combat/combat_system.rs` — new `CombatSystem` (priority 137).
6. `sim-systems/runtime/decision/agent_decision.rs` — add `CascadeArm::Combat` +
   7th cascade arm in `breached` + CombatReason early-exit handler.
7. `sim-systems/runtime/memory/memory_system.rs` — extend `classify_event` for
   CombatStarted/CombatCompleted + anti-recursion for CombatReason.
8. `sim-systems/lib.rs` — `register_combat_systems` + extend `register_default_runtime_systems`.
9. `sim-test/tests/harness_p9_beta_combat_system.rs` — ≥14 assertions (A1–A18).

**CRITICAL locked fact (P9β-3):**
`CombatCompleted` has field `hp_after: f64`. NOT `defender_died: bool`. This is
non-negotiable. Any deviation from `hp_after: f64` will cause harness failures.

---

## Section 2 — What to Build (locked facts)

### P9β-D1: `rust/crates/sim-core/src/causal/event.rs` (MODIFY)

**Current state (confirmed):**
- `DecisionReason`: 6 variants ending with `MemoryReason`
- `MemoryRecallTrigger`: 3 variants: `CascadeBias`, `SimilaritySearch`, `Periodic`
- `CausalEvent`: 9 variants ending with `MemoryRecalled`
- `impl CausalEvent` has 4 methods: `id()`, `parent()`, `tick()`, `channel()`

**Change 1 — `DecisionReason`: add 7th variant after `MemoryReason`:**
```rust
/// Agent's combat cascade arm activated by a negative memory weight
/// delta that strictly exceeded `BIAS_FLIP_THRESHOLD` in the negative
/// direction. A co-located idle peer is the combat target.
/// V7 Phase 9-β / P9β-5.
CombatReason,
```

**Change 2 — `DecisionReason::as_str()`: add arm:**
```rust
DecisionReason::CombatReason => "combat_reason",
```

**Change 3 — `MemoryRecallTrigger`: add 4th variant after `Periodic`:**
```rust
/// Phase 9-β scope: `AgentDecisionSystem` combat cascade arm fired
/// this memory recall. `agent_id` is the enemy agent targeted by
/// the combat transition.
CombatContext {
    /// The enemy agent targeted by the combat cascade.
    agent_id: AgentId,
},
```

**Change 4 — `CausalEvent`: add 2 variants after `MemoryRecalled`:**
```rust
/// Agent transitioned to `Consuming { Agent(defender) }` via the
/// combat cascade arm. Emitted ONCE per pair by `AgentDecisionSystem`
/// from the smaller-`AgentId` agent's evaluation (deduplication,
/// mirrors `SocialInteractionStarted` pattern).
/// `parent` links to the emitting agent's `AgentDecision{CombatReason}`.
/// V7 Phase 9-β / P9β-1.
CombatStarted {
    /// This event's unique id.
    id: EventId,
    /// Parent event id — the attacker's `AgentDecision{CombatReason}`.
    parent: Option<EventId>,
    /// Agent initiating combat (the one whose cascade triggered).
    attacker: AgentId,
    /// Agent being attacked.
    defender: AgentId,
    /// Shared tile coordinate at combat start.
    position: (u32, u32),
    /// Simulation tick at which combat started.
    tick: u64,
},

/// `CombatSystem` applied `DAMAGE_PER_COMBAT_TICK` to the defender.
/// `parent` links to the originating `CombatStarted`. `hp_after`
/// snapshots the defender's HP after damage (saturates at 0.0).
/// NOTE: field is `hp_after: f64`, NOT `defender_died: bool` — P9β-3.
/// V7 Phase 9-β / P9β-1.
CombatCompleted {
    /// This event's unique id.
    id: EventId,
    /// Parent event id — the originating `CombatStarted`.
    parent: Option<EventId>,
    /// Agent that initiated combat.
    attacker: AgentId,
    /// Agent that received damage.
    defender: AgentId,
    /// Shared tile coordinate at completion.
    position: (u32, u32),
    /// Defender HP after `apply_damage(DAMAGE_PER_COMBAT_TICK)`
    /// (saturated at 0.0). P9β-3: this is NOT `defender_died: bool`.
    hp_after: f64,
    /// Simulation tick at which damage was applied.
    tick: u64,
},
```

**Change 5 — extend all 4 `impl CausalEvent` methods:**

In `id()`, `parent()`, `tick()`: add to the existing match arms:
```rust
| CausalEvent::CombatStarted { id, .. }
| CausalEvent::CombatCompleted { id, .. } => *id,
```
(similarly for `parent` and `tick` fields)

In `channel()` — add to the `None` arm:
```rust
| CausalEvent::CombatStarted { .. }
| CausalEvent::CombatCompleted { .. } => None,
```

---

### P9β-D2: `rust/crates/sim-engine/src/lib.rs` (MODIFY)

**Change 1 — imports:** Add `HashSet` to the existing `use std::collections::{...}`.

**Change 2 — `SimResources` struct:** Add two fields after `interaction_progress`
(before `time_of_day`):
```rust
/// Active combat pairs for the current tick. Keys are canonical
/// `(smaller AgentId, larger AgentId)` tuples inserted by
/// `AgentDecisionSystem` when the combat cascade arm fires.
/// `CombatSystem` (priority 137) consumes and removes completed pairs.
/// Phase 9-β / P9β-1.
pub combat_pairs: HashSet<(AgentId, AgentId)>,

/// Per-pair combat progress counter. Keyed by the same canonical
/// `(smaller, larger)` tuple as `combat_pairs`. `CombatSystem`
/// increments each pair's counter until it reaches
/// `REQUIRED_COMBAT_PROGRESS`, at which point `CombatCompleted` fires.
/// Phase 9-β / P9β-1.
pub combat_progress: HashMap<(AgentId, AgentId), u32>,
```

**Change 3 — `SimResources::new()`:** Add:
```rust
combat_pairs: HashSet::new(),
combat_progress: HashMap::new(),
```

---

### P9β-D3: `rust/crates/sim-systems/src/runtime/mod.rs` (MODIFY)

Add `pub mod combat;` in alphabetical order between `agent` and `construction`:
```rust
pub mod agent;
pub mod combat;
pub mod construction;
pub mod decision;
pub mod influence;
pub mod memory;
pub mod needs;
pub mod social;
```

---

### P9β-D4: `rust/crates/sim-systems/src/runtime/combat/mod.rs` (NEW)

```rust
//! Phase 9-β Combat subsystem.
pub mod combat_system;
pub use combat_system::CombatSystem;

/// Damage applied to defender per `CombatSystem` tick (P9Plan-5).
/// Single application resolves combat in one tick alongside
/// `REQUIRED_COMBAT_PROGRESS = 1`.
pub const DAMAGE_PER_COMBAT_TICK: f64 = 10.0;

/// Ticks of `Consuming{Agent}` required before `CombatCompleted` fires.
/// Locked at 1 — immediate resolution in the same tick as `CombatStarted`.
/// P9Plan-5.
pub const REQUIRED_COMBAT_PROGRESS: u32 = 1;
```

---

### P9β-D5: `rust/crates/sim-systems/src/runtime/combat/combat_system.rs` (NEW)

```rust
//! `CombatSystem` (priority 137) — Phase 9-β runtime combat resolution.
//!
//! Responsibility: advance `combat_progress` for every active pair in
//! `SimResources::combat_pairs`. When a pair reaches
//! `REQUIRED_COMBAT_PROGRESS`:
//!   1. Apply `DAMAGE_PER_COMBAT_TICK` to the defender's `BodyHealth`.
//!   2. Emit `CausalEvent::CombatCompleted` (parent = most recent same-pair
//!      `CombatStarted` on the shared tile).
//!   3. Bump `relationships[RelationshipKey::new(attacker, defender)].hostility`
//!      by `HOSTILITY_BUMP`.
//!   4. Remove pair from `combat_pairs` + `combat_progress`.
//!   5. If defender `is_dead()`: despawn immediately + resource-map cleanup
//!      (relationships, interaction_progress, combat_pairs, combat_progress).
//!      Reset attacker to `AgentState::Idle`.
//!   6. If defender alive: reset both attacker and defender to `Idle`.
//!
//! `AgentDecisionSystem` (priority 125) owns the `Idle → Consuming{Agent}`
//! transition and `CombatStarted` emission. `CombatSystem` never emits
//! `CombatStarted` — it only emits `CombatCompleted`.

use std::collections::HashMap;

use hecs::World;
use sim_core::causal::CausalEvent;
use sim_core::components::{
    Agent, AgentId, AgentState, BodyHealth, Position, RelationshipKey, TargetKind, HOSTILITY_BUMP,
};
use sim_engine::{RuntimeSystem, SimResources};

use crate::runtime::combat::{DAMAGE_PER_COMBAT_TICK, REQUIRED_COMBAT_PROGRESS};

/// Phase 9-β combat resolution system.
#[derive(Debug, Default)]
pub struct CombatSystem;

impl CombatSystem {
    /// Construct a fresh instance.
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeSystem for CombatSystem {
    fn name(&self) -> &str {
        "CombatSystem"
    }

    fn priority(&self) -> u32 {
        137
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let tick = resources.current_tick;
        let width = resources.tile_grid.width;

        // Build AgentId → Entity map (pattern from MemorySystem).
        let id_to_entity: HashMap<AgentId, hecs::Entity> = world
            .query::<&Agent>()
            .iter()
            .map(|(e, a)| (a.id, e))
            .collect();

        // Collect active pairs; clone to avoid borrow conflict during
        // progress mutation.
        let active_pairs: Vec<(AgentId, AgentId)> =
            resources.combat_pairs.iter().copied().collect();

        // Advance progress for all active pairs; collect completed.
        let mut completed: Vec<(AgentId, AgentId)> = Vec::new();
        for pair in &active_pairs {
            let prog = resources.combat_progress.entry(*pair).or_insert(0);
            *prog += 1;
            if *prog >= REQUIRED_COMBAT_PROGRESS {
                completed.push(*pair);
            }
        }

        // Process each completed pair.
        for (attacker_id, defender_id) in completed {
            // Remove tracking before any despawn path (idempotent cleanup).
            resources.combat_pairs.remove(&(attacker_id, defender_id));
            resources.combat_progress.remove(&(attacker_id, defender_id));

            let attacker_entity = match id_to_entity.get(&attacker_id).copied() {
                Some(e) => e,
                None => continue, // attacker already despawned
            };
            let defender_entity = match id_to_entity.get(&defender_id).copied() {
                Some(e) => e,
                None => {
                    // Defender already gone — reset attacker.
                    if let Ok(mut s) = world.get::<&mut AgentState>(attacker_entity) {
                        *s = AgentState::Idle;
                    }
                    continue;
                }
            };

            // Derive shared tile from attacker position.
            let position = world
                .get::<&Position>(attacker_entity)
                .map(|p| (p.x, p.y))
                .unwrap_or((0, 0));
            let tile_idx = position.1 * width + position.0;

            // Find parent CombatStarted id on the same tile (most recent
            // matching pair, reverse scan).
            let parent_id = resources.causal_log.get(tile_idx).and_then(|log| {
                log.as_slice().iter().rev().find_map(|ev| {
                    if let CausalEvent::CombatStarted { id, attacker, defender, .. } = ev {
                        if *attacker == attacker_id && *defender == defender_id {
                            return Some(*id);
                        }
                    }
                    None
                })
            });

            // Apply damage to defender.
            let hp_after = if let Ok(mut bh) = world.get::<&mut BodyHealth>(defender_entity) {
                bh.apply_damage(DAMAGE_PER_COMBAT_TICK);
                bh.hp
            } else {
                0.0
            };

            // Emit CombatCompleted.
            let completed_id = resources.issue_event_id();
            resources.causal_log.push(
                tile_idx,
                CausalEvent::CombatCompleted {
                    id: completed_id,
                    parent: parent_id,
                    attacker: attacker_id,
                    defender: defender_id,
                    position,
                    hp_after,
                    tick,
                },
            );

            // Bump hostility (saturating, NaN-safe via `bump_hostility`).
            resources
                .relationships
                .entry(RelationshipKey::new(attacker_id, defender_id))
                .or_default()
                .bump_hostility(HOSTILITY_BUMP);

            // Check death via the post-damage component value.
            let defender_dead = world
                .get::<&BodyHealth>(defender_entity)
                .map(|bh| bh.is_dead())
                .unwrap_or(true);

            if defender_dead {
                let dead_id = defender_id;
                let _ = world.despawn(defender_entity);

                // Cleanup all resource maps referencing the dead agent.
                resources
                    .relationships
                    .retain(|k, _| k.0 != dead_id && k.1 != dead_id);
                resources
                    .interaction_progress
                    .retain(|k, _| k.0 != dead_id && k.1 != dead_id);
                resources
                    .combat_pairs
                    .retain(|(a, d)| *a != dead_id && *d != dead_id);
                resources
                    .combat_progress
                    .retain(|(a, d), _| *a != dead_id && *d != dead_id);

                // Reset attacker to Idle.
                if let Ok(mut s) = world.get::<&mut AgentState>(attacker_entity) {
                    *s = AgentState::Idle;
                }
            } else {
                // Both survive — reset both to Idle.
                if let Ok(mut s) = world.get::<&mut AgentState>(attacker_entity) {
                    *s = AgentState::Idle;
                }
                if let Ok(mut s) = world.get::<&mut AgentState>(defender_entity) {
                    *s = AgentState::Idle;
                }
            }
        }
    }
}
```

---

### P9β-D6: `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs` (MODIFY)

**Current state (confirmed):**
- `CascadeArm` enum: `Hunger(0)`, `Thirst(1)`, `Fatigue(2)`, `Construction(3)`, `Social(4)`
- `BIAS_FLIP_THRESHOLD = 1.0`
- `all_idle_peers_by_pos: HashMap<(u32,u32), Vec<AgentId>>` exists (bias-only Social map)
- `breached` if-else chain ends with `} else { None };`
- `if let Some((natural_target, natural_reason)) = breached {` block follows
- Inside it: `let natural_arm = match natural_reason { ... };` (6-arm exhaustive match)
- `DecisionReason::MemoryReason => CascadeArm::Construction` is the current unreachable fallback

**Change 1 — `CascadeArm` enum:** Add 6th variant after `Social`:
```rust
/// Memory-driven negative combat trigger. A non-natural cascade arm
/// activated by `memory_weight_delta` strictly below
/// `-BIAS_FLIP_THRESHOLD`. Phase 9-β / P9β-5.
Combat,
```

**Change 2 — `arm_priority_index`:** Add:
```rust
CascadeArm::Combat => 5,
```

**Change 3 — `event_id_matches_arm`:** Extend the `matches!` macro with Combat arms:
```rust
| (CascadeArm::Combat, CausalEvent::CombatStarted { .. })
| (CascadeArm::Combat, CausalEvent::CombatCompleted { .. })
| (CascadeArm::Combat, CausalEvent::AgentDecision {
    reason: DecisionReason::CombatReason,
    ..
})
```

**Change 4 — `breached` if-else chain:** Replace the final `} else { None };` with the
combat arm (7th cascade step). The social arm currently looks like:
```rust
} else if social_opt.is_some_and(|s| s.loneliness > SOCIAL_THRESHOLD) {
    social_eligible_by_pos
        .get(&(pos.x, pos.y))
        .and_then(...)
        .map(|other_id| { ... })
} else {
    None
};
```
Replace `} else { None };` with:
```rust
} else {
    // 7th cascade: combat arm. Fires when no higher-priority drive
    // is active AND memory_weight_delta for the Combat arm is
    // strictly below -BIAS_FLIP_THRESHOLD (negative direction).
    // Requires a co-located idle peer (lowest-AgentId wins).
    // Phase 9-β / P9β-2 / P9β-5.
    if let Some(memory) = memory_opt {
        let combat_delta = memory_weight_delta(
            memory,
            CascadeArm::Combat,
            tick,
            &resources.causal_log,
        );
        if combat_delta < -BIAS_FLIP_THRESHOLD {
            all_idle_peers_by_pos
                .get(&(pos.x, pos.y))
                .and_then(|peers| {
                    peers
                        .iter()
                        .filter(|id| **id != agent.id)
                        .min()
                        .copied()
                })
                .map(|enemy_id| (TargetKind::Agent(enemy_id), DecisionReason::CombatReason))
        } else {
            None
        }
    } else {
        None
    }
};
```

**Change 5 — `if let Some((natural_target, natural_reason)) = breached {` block:**
Insert BEFORE the `let natural_arm = match natural_reason {` line:

```rust
// Phase 9-β / P9β-1: combat arm — special handling, bypasses the
// memory-bias flip mechanism. Emits MemoryRecalled{CombatContext},
// AgentDecision{CombatReason}, and (from smaller-id agent only)
// CombatStarted. Transitions DIRECTLY to Consuming{Agent} (no Seeking).
if natural_reason == DecisionReason::CombatReason {
    if let TargetKind::Agent(enemy_id) = natural_target {
        let Some(memory) = memory_opt else {
            continue; // defensive — combat_delta check above required memory
        };
        let Some(recalled_event) =
            top_contributor_entry(memory, CascadeArm::Combat, tick, &resources.causal_log)
        else {
            continue; // top contributor evicted between scoring and emission
        };
        let recall_parent = resources
            .causal_log
            .lookup(recalled_event)
            .and_then(|e| e.parent());
        let recall_id = resources.issue_event_id();
        resources.causal_log.push(
            tile_idx,
            CausalEvent::MemoryRecalled {
                id: recall_id,
                parent: recall_parent,
                agent: agent.id,
                recalled_event,
                triggered_by: MemoryRecallTrigger::CombatContext {
                    agent_id: enemy_id,
                },
                tick,
            },
        );
        let decision_id = resources.issue_event_id();
        resources.causal_log.push(
            tile_idx,
            CausalEvent::AgentDecision {
                id: decision_id,
                parent: Some(recall_id),
                agent: agent.id,
                position: (pos.x, pos.y),
                reason: DecisionReason::CombatReason,
                tick,
            },
        );
        // Emit CombatStarted once per pair — only from smaller AgentId.
        if agent.id < enemy_id {
            let started_id = resources.issue_event_id();
            resources.causal_log.push(
                tile_idx,
                CausalEvent::CombatStarted {
                    id: started_id,
                    parent: Some(decision_id),
                    attacker: agent.id,
                    defender: enemy_id,
                    position: (pos.x, pos.y),
                    tick,
                },
            );
        }
        // Direct transition: Idle → Consuming{Agent} (no Seeking step).
        *state = AgentState::Consuming {
            target: TargetKind::Agent(enemy_id),
        };
        let canonical = if agent.id < enemy_id {
            (agent.id, enemy_id)
        } else {
            (enemy_id, agent.id)
        };
        resources.combat_pairs.insert(canonical);
    }
    continue;
}
```

**Change 6 — `natural_arm` match:** Add exhaustiveness arm (unreachable in practice
because CombatReason hits `continue` above):
```rust
DecisionReason::CombatReason => CascadeArm::Combat,
```

---

### P9β-D7: `rust/crates/sim-systems/src/runtime/memory/memory_system.rs` (MODIFY)

**Current state (confirmed):**
`classify_event` match ends:
```rust
CausalEvent::SocialInteractionCompleted { agents, .. } => {
    Some((0.8, 0.7, vec![agents.0, agents.1]))
}
CausalEvent::BuildingPlaced { .. }
| CausalEvent::StampDirty { .. }
| CausalEvent::InfluenceChanged { .. }
| CausalEvent::MemoryRecalled { .. } => None,
```

**Change 1 — add Combat arms to `classify_event` BEFORE the catch-all None arm:**
```rust
// Phase 9-β encoding: CombatStarted → attacker only (defender did not
// initiate); CombatCompleted → both parties (mirrors SocialInteraction
// pattern, negative valence — hostile memory).
CausalEvent::CombatStarted { attacker, .. } => {
    Some((0.8, -0.6, vec![*attacker]))
}
CausalEvent::CombatCompleted { attacker, defender, .. } => {
    Some((0.9, -0.8, vec![*attacker, *defender]))
}
```

**Change 2 — extend anti-recursion in `AgentDecision` arm of `classify_event`:**
```rust
DecisionReason::MemoryReason => None,     // existing
DecisionReason::CombatReason => None,     // Phase 9-β anti-recursion
```

**Change 3 — extend catch-all None arm to include new CombatStarted/CombatCompleted
as handled above.** The catch-all `BuildingPlaced|StampDirty|InfluenceChanged|MemoryRecalled => None`
stays unchanged; Rust exhaustiveness is satisfied because CombatStarted and CombatCompleted
are handled in the new arms above.

---

### P9β-D8: `rust/crates/sim-systems/src/lib.rs` (MODIFY)

**Change 1 — add `register_combat_systems` function (mirrors `register_memory_systems`):**
```rust
/// Register the Phase 9-β combat stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 137 : [`runtime::combat::CombatSystem`]
///
/// Slots strictly after `MemorySystem` (priority 136). Owns agent
/// `Consuming { Agent(_) }` exit semantics for combat pairs tracked
/// in `SimResources::combat_pairs`. `AgentDecisionSystem` (priority 125)
/// owns the Idle → Consuming transition and `CombatStarted` emission.
pub fn register_combat_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::combat::CombatSystem::new()));
}
```

**Change 2 — extend `register_default_runtime_systems`:** Add call after
`register_memory_systems(engine);`:
```rust
register_combat_systems(engine);
```

**Change 3 — update doc comment priority list** to add:
```
/// - 137  CombatSystem (Phase 9-β)
```

---

### P9β-D9: `rust/crates/sim-test/tests/harness_p9_beta_combat_system.rs` (NEW)

Assertion map (≥14 required, provide 18):

```
A1  : DAMAGE_PER_COMBAT_TICK == 10.0
A2  : REQUIRED_COMBAT_PROGRESS == 1
A3  : CombatSystem::priority() == 137
A4  : CombatSystem::tick_interval() == 1
A5  : DecisionReason::CombatReason.as_str() == "combat_reason"
A6  : MemoryRecallTrigger::CombatContext { agent_id: 42 } variant is constructible
A7  : CausalEvent::CombatStarted has fields id,parent,attacker,defender,position,tick
      (no hp_after field — that belongs to CombatCompleted only)
A8  : CausalEvent::CombatCompleted has hp_after: f64 field (NOT defender_died: bool)
A9  : Memory cascade flip → CombatStarted emitted in 1 tick
      Setup: 10×10 grid; agent A (id=1) + agent B (id=2) at tile (0,0);
      both Idle, no Hunger/Thirst/Sleep/Social components;
      push 2 CombatCompleted events into causal_log at tile 0
      (attacker=1, defender=2; both with id=1 and id=2 respectively);
      give agent A two MemoryEntries pointing to those event ids
      (valence=-0.8, salience=0.9, encoded_tick=current_tick);
      run 1 tick; assert CombatStarted exists in causal_log with
      attacker=1, defender=2.
A10 : CombatCompleted emitted in same tick as CombatStarted (REQUIRED_COMBAT_PROGRESS=1);
      run 1 more tick after A9 setup (total 2 ticks) — assert CombatCompleted
      in causal log.
A11 : HP reduced by DAMAGE_PER_COMBAT_TICK (100.0 → 90.0);
      assert defender's BodyHealth.hp == 90.0 after combat.
A12 : Entity despawned when hp <= 0.0;
      setup: give defender BodyHealth::new_with_max(10.0) (hp=10.0);
      run combat; assert world.contains(defender_entity) == false after.
A13 : hostility bumped by HOSTILITY_BUMP on CombatCompleted;
      assert resources.relationships[RelationshipKey::new(1,2)].hostility == HOSTILITY_BUMP.
A14 : MemoryRecalled{CombatContext} event precedes AgentDecision{CombatReason}
      in causal_log (tick order / push order within the tick).
A15 : AgentDecision{CombatReason} precedes CombatStarted in causal_log.
A16 : Non-hostile agent (no negative combat memory) does NOT trigger combat;
      agent A with no Memory component or zero-entry Memory → no CombatStarted.
A17 : Phase 9-α regression — BodyHealth::new().hp == 100.0; DEFAULT_MAX_HP == 100.0;
      RelationshipState has hostility field starting at 0.0; HOSTILITY_BUMP == 0.1.
A18 : Phase 8-α regression — Memory, MemoryEntry, MEMORY_CAP, SALIENCE_FLOOR
      re-exports intact from sim_core::components.
```

**Harness setup notes for A9 (runtime integration test):**

Use `make_stage1_engine(seed, num_agents)` or construct a minimal engine manually:
- 10×10 tile grid
- Two agents: id=1 at (0,0), id=2 at (0,0), both `AgentState::Idle`
- Both have `Memory` component
- Manually push 2 `CausalEvent::CombatCompleted` events into
  `resources.causal_log` at tile_idx=0 with ids `EVENT_ID_A` and `EVENT_ID_B`
  (using `resources.issue_event_id()` or fixed values like 100/101)
- Give agent 1 two `MemoryEntry` values:
  `MemoryEntry::new(EVENT_ID_A, current_tick, -0.8, 0.9)` and
  `MemoryEntry::new(EVENT_ID_B, current_tick, -0.8, 0.9)`
- Combined combat delta = (-0.8 × 0.9 × ~1.0) × 2 ≈ -1.44 < -1.0 (BIAS_FLIP_THRESHOLD) ✓
- After 1 `engine.run_ticks(1)`, scan causal_log for `CombatStarted` with attacker=1

---

## Section 3 — How to Implement

Follow Phase 8-β / 7-β / 9-α component+system precedent:

1. **`event.rs` first** — all downstream code depends on the new variants.
   Edit the 4 accessor impls carefully: `id()`, `parent()`, `tick()` use
   `| CausalEvent::CombatStarted { id, .. } | CausalEvent::CombatCompleted { id, .. } => *id`
   pattern (one extra `| arm` each). `channel()` adds them to the `None` arm.

2. **`sim-engine/lib.rs`** — add `HashSet` to imports, add 2 fields + init.
   `HashSet<(AgentId, AgentId)>` not `HashSet<RelationshipKey>` — raw canonical tuples.

3. **`runtime/mod.rs`** — one-line addition.

4. **`combat/mod.rs`** — constants + re-export. Keep it minimal.

5. **`combat_system.rs`** — use `id_to_entity` map (build once at tick start).
   The `completed` vec is collected in a separate pass to avoid borrow conflict
   between `combat_pairs.iter()` and `combat_progress.entry()`.
   For dead-agent cleanup: `retain(|k, _| k.0 != dead_id && k.1 != dead_id)` on
   `RelationshipKey`-keyed maps; `retain(|(a, d)| ...)` for raw-tuple maps.

6. **`agent_decision.rs`** — five changes. The most critical:
   - The combat arm replaces the final `} else { None };` in the `breached` chain.
   - The `CombatReason` early-exit block goes BEFORE `let natural_arm = match ...`.
   - The `DecisionReason::CombatReason => CascadeArm::Combat` arm is added LAST
     in the `natural_arm` match for exhaustiveness — it is unreachable in practice.
   - Do NOT add Combat to the `eligible` vec in the bias-flip section — the combat
     arm is a standalone trigger, not part of the P8β bias-flip mechanism.

7. **`memory_system.rs`** — two changes. The CombatStarted/CombatCompleted arms go
   AFTER `SocialInteractionCompleted` and BEFORE the catch-all `BuildingPlaced|...`.
   The anti-recursion `CombatReason => None` goes after `MemoryReason => None` in
   the `AgentDecision` arm's inner match.

8. **`sim-systems/lib.rs`** — add function + 1-line call. Update the doc comment
   priority list.

9. **Harness test** — 18 assertions. Use direct construction for type/constant checks
   (A1–A8). For runtime checks (A9–A16): build a minimal engine, pre-populate the
   causal_log with CombatCompleted events, pre-populate Memory entries on agent 1,
   then run ticks and scan the log.

**Verify after each file:**
```bash
cd rust && cargo build --workspace 2>&1 | tail -5
```

**Final gate:**
```bash
cd rust && cargo test -p sim-test --test harness_p9_beta_combat_system -- --nocapture
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail -10
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | grep "^error" | head -5
```

---

## Section 4 — Locale

No new locale keys. Phase 9-β is backend-only. No `.gd`, `.tscn`, `.tres`, or
`localization/` files modified.

---

## Section 5 — Verification Gate

```bash
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
```

Both must be clean. `harness_p9_beta_combat_system` must pass with ≥14 assertions.
All Phase 9-α / 8-α / 7-α / 6-α harnesses must remain CLEAN (regression baseline).

---

## Section 6 — Lane

`--full` (sim-core `.rs` changes: new CausalEvent variants, DecisionReason variant,
MemoryRecallTrigger variant; sim-systems `.rs` changes: new system module +
agent_decision modification + memory_system modification; sim-engine `.rs` changes:
new SimResources fields).

Cold-tier auto credit expected (no GDScript/scene changes).

---

## Section 7 — In-game verification

Backend only. No Godot scope. Pipeline VLM will produce VISUAL_WARNING (no game
running) — hook applies +8 env cost adjustment per CLAUDE.md §7 adjusted score
formula. Expected adjusted score: 84+8 = 92 or higher at attempt 1.

Score decomposition target:
```
Code Quality:   15/15  (attempt 1)
Visual Verify:   0/20  → +8 env cost adjustment (VLM SKIP/WARNING expected)
Tests:          20/20
Regression:     15/15
Evaluator:      15/15
Gate:           10/10
Raw:            75/95 → Adjusted: 83+8 = 91 or 84+8 = 92
```
