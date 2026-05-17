# P7-β — `SocialInteractionSystem` + 2 CausalEvent variants + `DecisionReason::SocialReason` + `AgentDecisionSystem` takeover

> Lane: `--full` (sim-core `CausalEvent`/`DecisionReason` enum extensions +
> sim-systems new runtime module + agent_decision.rs takeover from Phase 7-α
> inert placeholders + sim-engine `SimResources` schema additions all force
> full lane).
> Scope: Wire the agent-to-agent interaction tick loop. Mutual handshake →
> multi-tick progress → completion + familiarity bump. Full causal chain.
> Governance: v3.3.17. Visual: backend only (no `.gd`/`.gdshader`/`.tscn`/
> `.tres`, no `scripts/` or `scenes/` path) — Pipeline VLM no-godot-scope auto
> credit expected.

---

## Section 1 — Implementation Intent

Phase 7-α (`35fbd501`) landed the data substrate (`Social`, `RelationshipKey`,
`RelationshipState`, `TargetKind::Agent(AgentId)` 5th variant) with two
**explicitly inert** match-arm placeholders in `agent_decision.rs`:

- Line 275: `TargetKind::Agent(_) => false` in the Seeking-branch
  `has_resource` switch.
- Lines 391-395: `TargetKind::Agent(_) => { /* no-op */ }` in the Consuming-
  branch.

Both placeholders document themselves as "β scope — runtime resolution lands in
Phase 7-β". This dispatch is that takeover.

Phase 7-β simultaneously:
1. **Introduces** the new `SocialInteractionSystem` at priority 134 (slot
   immediately after `ConstructionSystem`=133). Its responsibilities:
   - For each agent in `Consuming { TargetKind::Agent(other_id) }`, verify
     mutual + co-location, advance per-pair progress, emit causal chain on
     completion.
   - Handle the **asymmetric-partner fallback** (partner moved off-tile or no
     longer Consuming): revert agent to Idle without panic (Phase 6-β
     absent-site fallback symmetry).
2. **Extends** `CausalEvent` with two new variants:
   - `SocialInteractionStarted { id, parent, agents, position, tick }`
   - `SocialInteractionCompleted { id, parent, agents, position, familiarity_after, tick }`
   - `agents` is the canonical `(smaller AgentId, larger AgentId)` pair.
3. **Extends** `DecisionReason` with the 5th variant `SocialReason`.
4. **Activates** Phase 7-α's two inert placeholders in `agent_decision.rs`:
   - Idle branch gains a **5th cascade step** (after Hunger/Thirst/Fatigue/
     Construction): co-located agent with mutual loneliness breach →
     `Seeking { TargetKind::Agent(other_id) }` + emit
     `AgentDecision { reason: SocialReason }`. Deterministic tie-break:
     lowest AgentId among co-located breached partners.
   - Seeking branch flips the `false` placeholder to a proper "partner is on
     this tile AND in `Seeking { Agent(self.id) }`" check. On match, both
     agents transition to `Consuming { Agent(other) }` and a single
     `SocialInteractionStarted` event is emitted.
   - Consuming branch **stays** a no-op — `SocialInteractionSystem` (134)
     owns Consuming-state exit semantics (same pattern as Construction-system
     ownership of Consuming{ConstructionSite}).
5. **Adds two HashMap fields** to `SimResources`:
   - `relationships: HashMap<RelationshipKey, RelationshipState>` — sparse,
     starts empty.
   - `interaction_progress: HashMap<RelationshipKey, u32>` — sparse, per-pair
     multi-tick progress counter.
6. **Exports four constants** from `agent_decision.rs`:
   - `SOCIAL_THRESHOLD: f64 = 50.0`
   - `SOCIAL_CONSUME_AMOUNT: f64 = 30.0` (mirrors HUNGER/THIRST/FATIGUE)
   - `REQUIRED_INTERACTION_PROGRESS: u32 = 3` (smaller than construction =5)
   - `FAMILIARITY_BUMP: f64 = 0.1` (saturates at 1.0 after 10 interactions)

The completion chain that closes the "왜?" UI walk:

```
AgentDecision { reason: SocialReason, agent: self, ... }
    ↓ (parent ref)
SocialInteractionStarted { parent: Some(AgentDecision.id), agents: (a, b), position }
    ↓ (parent ref)
SocialInteractionCompleted { parent: Some(SocialInteractionStarted.id),
    agents: (a, b), familiarity_after: <new value> }
```

After P7-β:
- `sim_systems::runtime::social::SocialInteractionSystem` exists at priority 134.
- `DecisionReason` has **5 variants** (Hunger/Thirst/Fatigue ThresholdBreach +
  ConstructionReason + SocialReason).
- `CausalEvent` has **8 variants** (BuildingPlaced / StampDirty /
  InfluenceChanged / AgentDecision / ConstructionStarted / ConstructionCompleted /
  SocialInteractionStarted / SocialInteractionCompleted).
- `AgentDecisionSystem` FSM extended with the **5th cascade** (Social is the
  lowest-priority drive — Needs and Construction always win).
- `SimResources` has `relationships` + `interaction_progress` HashMaps.
- A ≥12-assertion harness `harness_p7_beta_social_system.rs` proves the full
  agent FSM mutual handshake cycle, the 3-link causal chain, asymmetric-partner
  fallback, and cascade ordering preservation.

---

## Section 2 — Locked facts (from pre-grep + plan §3 §β — must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| P7β-1: `SocialInteractionSystem::priority` | plan §3 §β + grep-verified (133=Construction max) | `priority() = 134`. Strictly after `ConstructionSystem` (133), strictly before `InfluenceVisualizationSystem` (1000). |
| P7β-2: `SocialInteractionSystem::tick_interval` | plan §3 §β | `tick_interval() = 1`. Every tick. |
| P7β-3: `SocialInteractionStarted` struct | plan §3 §β verbatim | `SocialInteractionStarted { id: EventId, parent: Option<EventId>, agents: (AgentId, AgentId), position: (u32, u32), tick: u64 }`. Position is `(u32, u32)` tuple matching existing CausalEvent precedent (verified in P6-β event.rs). `agents` is canonicalised: `(min(a,b), max(a,b))` — mirrors `RelationshipKey::new()` semantics. |
| P7β-4: `SocialInteractionCompleted` struct | plan §3 §β verbatim | `SocialInteractionCompleted { id: EventId, parent: Option<EventId>, agents: (AgentId, AgentId), position: (u32, u32), familiarity_after: f64, tick: u64 }`. `familiarity_after` snapshots the post-bump familiarity value (between 0.0 and 1.0). |
| P7β-5: `DecisionReason::SocialReason` | plan §3 §β | Append `SocialReason` as 5th variant after `ConstructionReason`. `as_str()` returns `"social_reason"`. |
| P7β-6: Idle-branch 5th cascade | plan §3 §β verbatim | After the existing 4 cascade arms (Hunger, Thirst, Fatigue, Construction), append a 5th `else if`: "agent's `Social.loneliness > SOCIAL_THRESHOLD` AND there exists another agent on the same tile whose `Social.loneliness > SOCIAL_THRESHOLD`" → `Some((TargetKind::Agent(other_id), DecisionReason::SocialReason))`. Tie-break: lowest AgentId among co-located breached partners. Social is **lowest priority** — Needs and Construction always win. |
| P7β-7: Mutual handshake (Seeking → Consuming) | plan §3 §β verbatim | The Phase 7-α inert `TargetKind::Agent(_) => false` at `agent_decision.rs:275` becomes "partner is at same tile AND partner is currently in `Seeking { Agent(self.id) }`". When both conditions hold, both agents transition to `Consuming { Agent(other) }` and ONE `SocialInteractionStarted` event is emitted (canonicalised, not double-emitted). |
| P7β-8: SocialInteractionStarted emission point | plan §3 §β | Emitted by `AgentDecisionSystem` (priority 125) the moment a mutual handshake closes. **Emitted exactly once per handshake** even though both agents are observed. Deduplication: only the agent with the **smaller AgentId** emits the event — the other agent observes the partner-Seeking precondition and transitions silently. `parent = Some(<most recent same-tile AgentDecision id where reason==SocialReason AND agent==self.id>)`. |
| P7β-9: Consuming branch unchanged | plan §3 §β | The Phase 7-α `TargetKind::Agent(_) => { /* no-op */ }` placeholder at `agent_decision.rs:391-395` stays. `SocialInteractionSystem` at priority 134 owns all Consuming-state exit semantics. |
| P7β-10: ConstructionSystem precedent for SocialInteractionSystem tick | plan §3 §β verbatim | (a) Snapshot all agents in `Consuming { Agent(_) }` upfront. (b) For each, look up the partner by `AgentId` — confirm partner also in `Consuming { Agent(self.id) }` AND co-located. (c) Asymmetric or non-co-located: agent → `Idle`, no progress change (absent-partner fallback). (d) Mutual + co-located: advance the per-pair `interaction_progress[RelationshipKey]` by 1 (only ONCE per pair per tick — dedup by canonical key). (e) On `progress >= REQUIRED_INTERACTION_PROGRESS`: emit `SocialInteractionCompleted`, bump `RelationshipState::familiarity` by `FAMILIARITY_BUMP`, both agents → `Idle`, both `Social.loneliness -= SOCIAL_CONSUME_AMOUNT` (saturating at 0.0), remove the `interaction_progress` entry. |
| P7β-11: `interaction_progress` dedup | plan §3 §β implicit | `interaction_progress[RelationshipKey]` is incremented at most ONCE per tick per pair. Both agents observe the same pair; the system must dedupe by the canonical `RelationshipKey`. Phase 6-β precedent: ConstructionSystem dedup by Entity. Here: dedup by canonical RelationshipKey. |
| P7β-12: Familiarity bump on completion | plan §3 §β verbatim | On completion: `relationships.entry(RelationshipKey::new(a, b)).or_insert_with(RelationshipState::new).bump(FAMILIARITY_BUMP)`. The `SocialInteractionCompleted` event captures `familiarity_after` as the value POST-bump. |
| P7β-13: Schema additions on `SimResources` | plan §3 §β verbatim | Two new public fields after the existing `*_tiles` HashMaps: `pub relationships: HashMap<RelationshipKey, RelationshipState>` (starts empty) + `pub interaction_progress: HashMap<RelationshipKey, u32>` (starts empty). `SimResources::new` and `Default` both initialize them to `HashMap::new()`. |
| P7β-14: Four constants | plan §3 §β verbatim | Exported from `agent_decision.rs` as `pub const`: `SOCIAL_THRESHOLD: f64 = 50.0`, `SOCIAL_CONSUME_AMOUNT: f64 = 30.0`, `REQUIRED_INTERACTION_PROGRESS: u32 = 3`, `FAMILIARITY_BUMP: f64 = 0.1`. Doc-comments mirror the Hunger/Thirst/Fatigue threshold/consume style. |
| P7β-15: register helper | plan §3 §β + Construction precedent | Add `pub fn register_social_systems(engine: &mut SimEngine)` to `sim-systems/src/lib.rs` mirroring `register_construction_systems`. If `DEFAULT_RUNTIME_SYSTEMS` (or similar central list) exists, append `SocialInteractionSystem` immediately after `ConstructionSystem`. |
| P7β-16: BuildingStampSystem + Construction untouched | plan §3 cross-cutting | No edits to `sim-systems/runtime/influence/building_stamp.rs` or `sim-systems/runtime/construction/construction_system.rs`. Phase 2 + Phase 6 substrate intact. |
| P7β-17: CausalEvent derives | event.rs precedent | `CausalEvent` derives `Debug, Clone, PartialEq` only (NOT `Serialize`/`Deserialize` — pre-existing constraint, see causal/event.rs:22). The two new variants follow the same derives. `DecisionReason::SocialReason` inherits the existing `Debug, Clone, Copy, PartialEq, Eq, Hash` set. |
| Harness assertion count | plan §3 §β + Phase 6-β precedent | **≥12 assertions** in `harness_p7_beta_social_system.rs`. Each locked fact P7β-1 through P7β-17 + regression coverage. |

**Determinism rationale** (axiom #2 — verified before assuming):
1. `SocialInteractionSystem::tick` iterates a HashMap-snapshot by AgentId →
   position. HashMap order is not deterministic but per-pair mutations are
   commutative (each pair processes its own RelationshipKey). Cross-pair
   ordering is irrelevant to test outcomes.
2. The Idle-branch 5th cascade tie-break uses **lowest AgentId among
   co-located breached partners** — deterministic.
3. The `SocialInteractionStarted` emission dedup uses **smaller-AgentId
   emits** — deterministic single emission per handshake.
4. `RelationshipKey::new()` canonicalises pair direction — `RelationshipKey(a,
   b) == RelationshipKey(b, a)` always, removing pair-direction ambiguity.
5. `interaction_progress` keyed by `RelationshipKey` ensures dedup is
   structural, not order-dependent.

---

## Section 3 — What to build (5 files added + 4 files modified)

### 3.1 `rust/crates/sim-core/src/causal/event.rs` (MODIFY)

**(a)** `DecisionReason` enum — append `SocialReason`:

```rust
pub enum DecisionReason {
    HungerThresholdBreach,
    ThirstThresholdBreach,
    FatigueThresholdBreach,
    ConstructionReason,
    /// Agent transitioned from Idle to Seeking{Agent(other)} after
    /// detecting a co-located partner whose loneliness also breached
    /// SOCIAL_THRESHOLD. V7 Phase 7-β / P7β-5. Social is the lowest-
    /// priority drive — Needs and Construction always win.
    SocialReason,
}
```

`as_str()` arm: `DecisionReason::SocialReason => "social_reason"`.

**(b)** `CausalEvent` enum — append two variants after `ConstructionCompleted`:

```rust
pub enum CausalEvent {
    BuildingPlaced { ... },
    StampDirty { ... },
    InfluenceChanged { ... },
    AgentDecision { ... },
    ConstructionStarted { ... },
    ConstructionCompleted { ... },

    /// Mutual social handshake closed: both agents transitioned from
    /// Seeking{Agent(other)} to Consuming{Agent(other)} on the same tile.
    /// Emitted ONCE per handshake by AgentDecisionSystem (deduplicated by
    /// emitting only from the smaller-AgentId agent's transition).
    /// parent links to the same-tile AgentDecision{SocialReason, agent: emitter}.
    /// V7 Phase 7-β / P7β-3.
    SocialInteractionStarted {
        id: EventId,
        parent: Option<EventId>,
        agents: (AgentId, AgentId),  // canonical (smaller, larger)
        position: (u32, u32),
        tick: u64,
    },

    /// Mutual social interaction reached REQUIRED_INTERACTION_PROGRESS.
    /// Emitted by SocialInteractionSystem before the agent state transitions
    /// back to Idle. familiarity_after is the post-bump RelationshipState
    /// familiarity value (saturating at 1.0). parent links to the originating
    /// SocialInteractionStarted. V7 Phase 7-β / P7β-4.
    SocialInteractionCompleted {
        id: EventId,
        parent: Option<EventId>,
        agents: (AgentId, AgentId),  // canonical (smaller, larger)
        position: (u32, u32),
        familiarity_after: f64,
        tick: u64,
    },
}
```

**(c)** Accessor methods (`id`, `parent`, `tick`, `channel`) — extend `match`
arms exhaustively for the two new variants. `channel()` returns `None` for
both (channel-agnostic).

**(d)** `#[cfg(test)] mod tests` — extend with:
- `social_interaction_started_records_fields`
- `social_interaction_completed_records_fields`
- `decision_reason_social_as_str` (covers the new `SocialReason` arm)

### 3.2 `rust/crates/sim-systems/src/runtime/social/mod.rs` (NEW)

```rust
//! V7 Phase 7-β / P7β-1 — Agent-to-agent social interaction runtime.
//!
//! Owns the progress-and-completion side of the social interaction loop.
//! `AgentDecisionSystem` (priority 125) handles all AgentState transitions
//! INTO and THROUGH Seeking/Consuming for `TargetKind::Agent(_)`; this
//! system handles the OUT transition (Consuming → Idle) plus per-pair
//! progress, familiarity bump, and absent-partner fallback.

pub mod social_interaction_system;

pub use social_interaction_system::SocialInteractionSystem;
```

### 3.3 `rust/crates/sim-systems/src/runtime/social/social_interaction_system.rs` (NEW)

```rust
//! `SocialInteractionSystem` at priority 134 — the runtime tick of the
//! Phase 7 agent-to-agent interaction loop.
//!
//! Tick ordering context:
//!
//! ```text
//! 125 AgentDecisionSystem        (Idle→Seeking, Seeking→Consuming with mutual handshake, SocialInteractionStarted emit)
//! 130 HungerDecaySystem
//! 131 ThirstDecaySystem
//! 132 SleepDecaySystem
//! 133 ConstructionSystem
//! 134 SocialInteractionSystem    ← this file
//! ```

use std::collections::{HashMap, HashSet};
use hecs::{Entity, World};
use sim_core::causal::CausalEvent;
use sim_core::components::{
    Agent, AgentId, AgentState, Position, RelationshipKey, RelationshipState,
    Social, TargetKind,
};
use sim_engine::{RuntimeSystem, SimResources};

use crate::runtime::decision::agent_decision::{
    FAMILIARITY_BUMP, REQUIRED_INTERACTION_PROGRESS, SOCIAL_CONSUME_AMOUNT,
};

#[derive(Debug, Default)]
pub struct SocialInteractionSystem;

impl SocialInteractionSystem {
    pub fn new() -> Self { Self }
}

impl RuntimeSystem for SocialInteractionSystem {
    fn name(&self) -> &str { "SocialInteractionSystem" }
    fn priority(&self) -> u32 { 134 }
    fn tick_interval(&self) -> u64 { 1 }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let tick = resources.current_tick;

        // (a) Snapshot agents in Consuming { Agent(_) }.
        // Per-agent record: (entity, AgentId, position, partner AgentId).
        struct ConsumingPair {
            entity: Entity,
            agent_id: AgentId,
            position: (u32, u32),
            partner_id: AgentId,
        }
        let mut consuming: Vec<ConsumingPair> = Vec::new();
        {
            let mut q = world.query::<(&Agent, &Position, &AgentState)>();
            for (entity, (a, pos, state)) in q.iter() {
                if let AgentState::Consuming { target: TargetKind::Agent(partner_id) } = *state {
                    consuming.push(ConsumingPair {
                        entity,
                        agent_id: a.id,
                        position: (pos.x, pos.y),
                        partner_id,
                    });
                }
            }
        }

        // (b) Build (AgentId → ConsumingPair index) for partner lookup.
        let id_to_idx: HashMap<AgentId, usize> = consuming.iter().enumerate()
            .map(|(i, p)| (p.agent_id, i)).collect();

        // (c) Classify each into mutual-or-fallback.
        // mutual_pairs: dedup by canonical RelationshipKey; only process once.
        let mut mutual_processed: HashSet<RelationshipKey> = HashSet::new();
        let mut fallback_entities: Vec<Entity> = Vec::new();
        let mut completions: Vec<(RelationshipKey, (u32, u32), Vec<Entity>)> = Vec::new();
        let mut progressed: Vec<RelationshipKey> = Vec::new();

        for pair in &consuming {
            let partner_idx = id_to_idx.get(&pair.partner_id);
            let mutual = partner_idx.map(|&i| {
                let partner = &consuming[i];
                partner.partner_id == pair.agent_id
                    && partner.position == pair.position
            }).unwrap_or(false);

            if !mutual {
                fallback_entities.push(pair.entity);
                continue;
            }

            let key = RelationshipKey::new(pair.agent_id, pair.partner_id);
            if !mutual_processed.insert(key) {
                continue;  // already processed this pair via the smaller agent
            }

            // Advance progress.
            let prog = resources.interaction_progress.entry(key).or_insert(0);
            *prog += 1;
            let new_prog = *prog;

            if new_prog >= REQUIRED_INTERACTION_PROGRESS {
                // Both agents complete.
                let partner_idx_unwrapped = id_to_idx[&pair.partner_id];
                let partner_entity = consuming[partner_idx_unwrapped].entity;
                completions.push((key, pair.position, vec![pair.entity, partner_entity]));
            } else {
                progressed.push(key);
            }
        }

        // (d) Apply fallbacks first.
        for entity in fallback_entities {
            if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
                *state = AgentState::Idle;
            }
        }

        // (e) Apply completions: bump familiarity, emit events, reset state,
        //     decrement Social, remove interaction_progress entry.
        for (key, position, entities) in completions {
            // Bump familiarity.
            let entry = resources.relationships.entry(key).or_default();
            entry.bump(FAMILIARITY_BUMP);
            let familiarity_after = entry.familiarity;

            // Parent: most recent same-tile SocialInteractionStarted with this pair.
            let width = resources.tile_grid.width as usize;
            let tile_idx = (position.1 as usize) * width + (position.0 as usize);
            let parent = resources
                .causal_log
                .get(tile_idx)
                .and_then(|log| log.as_slice().iter().rev().find_map(|ev| match ev {
                    CausalEvent::SocialInteractionStarted { id, agents, .. }
                        if *agents == (key.smaller(), key.larger()) => Some(*id),
                    _ => None,
                }));

            let completed_id = resources.issue_event_id();
            resources.causal_log.push(
                tile_idx,
                CausalEvent::SocialInteractionCompleted {
                    id: completed_id,
                    parent,
                    agents: (key.smaller(), key.larger()),
                    position,
                    familiarity_after,
                    tick,
                },
            );

            // Reset both agent states + decrement Social.
            for entity in entities {
                if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
                    *state = AgentState::Idle;
                }
                if let Ok(mut social) = world.get::<&mut Social>(entity) {
                    social.loneliness = (social.loneliness - SOCIAL_CONSUME_AMOUNT).max(0.0);
                }
            }

            // Remove the progress entry.
            resources.interaction_progress.remove(&key);
        }

        // (f) progressed list is informational only — progress already bumped.
        let _ = progressed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metadata() {
        let s = SocialInteractionSystem::new();
        assert_eq!(s.name(), "SocialInteractionSystem");
        assert_eq!(s.priority(), 134);
        assert_eq!(s.tick_interval(), 1);
    }
}
```

**Note for the Generator**: the pseudocode above shows the structure but the
Generator MUST adapt to compile cleanly under hecs' borrow rules and the
existing `sim_engine::RuntimeSystem` trait shape. Specifically:
- Nested `world.get::<&mut AgentState>` calls inside borrow scopes may need
  restructuring (collect entity references first, drop the query, then mutate).
- The dedup-by-key + "smaller AgentId emits" handshake check is the locked
  semantic; the implementation may use a different data-structure layout as
  long as the observable behavior matches Section 2 P7β-7/P7β-10/P7β-11.

### 3.4 `rust/crates/sim-systems/src/runtime/mod.rs` (MODIFY)

Add `pub mod social;` in alphabetical position. Adjust if the actual existing
ordering differs; preserve alphabetical ordering of the public module list.

### 3.5 `rust/crates/sim-systems/src/lib.rs` (MODIFY)

Add a `register_social_systems` helper next to `register_construction_systems`:

```rust
pub fn register_social_systems(engine: &mut sim_engine::SimEngine) {
    engine.register_runtime_system(Box::new(
        crate::runtime::social::SocialInteractionSystem::new(),
    ));
}
```

If `DEFAULT_RUNTIME_SYSTEMS` or similar central registration list exists, add
`SocialInteractionSystem` to it immediately after `ConstructionSystem` to match
the priority ordering.

### 3.6 `rust/crates/sim-engine/src/lib.rs` (MODIFY — `SimResources` schema)

Two new public fields on `SimResources`, after the existing `sleep_tiles`:

```rust
pub struct SimResources {
    // ... existing fields ...
    pub food_tiles: HashMap<(u32, u32), u8>,
    pub water_tiles: HashMap<(u32, u32), u8>,
    pub sleep_tiles: HashMap<(u32, u32), u8>,
    pub relationships: HashMap<RelationshipKey, RelationshipState>,     // NEW
    pub interaction_progress: HashMap<RelationshipKey, u32>,            // NEW
    // ... rest ...
}
```

`SimResources::new` and any `Default` impl initialize both to `HashMap::new()`.
Imports updated: `use sim_core::components::{RelationshipKey, RelationshipState};`.

### 3.7 `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs` (MODIFY)

Multiple loci of change inside `AgentDecisionSystem::tick` and the file-level
constants block:

**(a)** Add four `pub const` exports near the existing
`FATIGUE_THRESHOLD`/`FATIGUE_CONSUME_AMOUNT` constants:

```rust
pub const SOCIAL_THRESHOLD: f64 = 50.0;
pub const SOCIAL_CONSUME_AMOUNT: f64 = 30.0;
pub const REQUIRED_INTERACTION_PROGRESS: u32 = 3;
pub const FAMILIARITY_BUMP: f64 = 0.1;
```

Doc-comments mirror the existing `HUNGER_THRESHOLD`/`HUNGER_CONSUME_AMOUNT`
doc style.

**(b)** Top of tick — build the co-located-partner map. After the existing
width/height guard, before the agent query. Build:

```rust
let mut by_pos: HashMap<(u32, u32), Vec<(AgentId, f64)>> = HashMap::new();
{
    let mut q = world.query::<(&Agent, &Position, Option<&Social>)>();
    for (_, (a, pos, social_opt)) in q.iter() {
        let lone = social_opt.map(|s| s.loneliness).unwrap_or(0.0);
        by_pos.entry((pos.x, pos.y)).or_default().push((a.id, lone));
    }
}
```

**(c)** Idle-branch — 5th cascade step. Extend the existing `breached =
if/else if/else if/else if/else` ladder with a 5th `else if`:

```rust
} else if let Some((other_id, _)) = social_opt.as_ref()
    .filter(|s| s.loneliness > SOCIAL_THRESHOLD)
    .and_then(|_| {
        by_pos.get(&(pos.x, pos.y))
            .map(|peers| peers.iter()
                .filter(|(id, lone)| *id != agent.id && *lone > SOCIAL_THRESHOLD)
                .min_by_key(|(id, _)| *id)
                .copied())
            .flatten()
    })
{
    Some((TargetKind::Agent(other_id), DecisionReason::SocialReason))
} else {
    None
};
```

The agent query must add `Option<&Social>` to the existing tuple so
`social_opt` is in scope inside the agent loop.

**(d)** Seeking branch — flip the `TargetKind::Agent(_) => false` placeholder
at line 275 to a proper mutual check:

```rust
TargetKind::Agent(partner_id) => {
    // Mutual handshake: partner must be at this tile AND in
    // Seeking { Agent(self.id) }. Both transitions happen in this same
    // tick: the smaller-AgentId agent emits SocialInteractionStarted.
    // (Implementation detail: the actual paired state transition needs
    // a post-loop apply pass since the agent query borrows world mutably
    // for the current agent's state.)
    // For the has_resource boolean here, we check the partner-Seeking
    // precondition. The actual paired transition + emission lives in a
    // post-loop pass.
    by_pos.get(&(pos.x, pos.y))
        .map(|peers| peers.iter().any(|(id, _)| *id == partner_id))
        .unwrap_or(false)
    // (Partner-Seeking-self check happens in the post-loop apply pass.)
}
```

**Note**: the borrow-checker friendly implementation may need to:
- Collect "agent is in Seeking { Agent(X) } at position P" tuples during the
  agent query.
- After the query borrow drops, scan the collected tuples for mutual pairs
  (same position, X→Y and Y→X), and apply paired Consuming transitions +
  ONE `SocialInteractionStarted` emission (smaller-AgentId emits).
- Generator chooses the exact shape; the locked semantic is "mutual handshake
  with smaller-AgentId-emits dedup".

**(e)** Consuming branch — leave the inert `TargetKind::Agent(_) => { /* no-op */ }`
intact (Phase 7-α placeholder at lines 391-395 stays unchanged; the
SocialInteractionSystem (134) owns exit semantics).

**(f)** Existing test extensions: any inline `#[cfg(test)]` tests that
exhaustively match `DecisionReason` or `TargetKind` need to add an arm for
`SocialReason` / `Agent(_)`. Grep for existing match arms before declaring
this section complete.

### 3.8 `rust/crates/sim-test/tests/harness_p7_beta_social_system.rs` (NEW)

≥12 numbered assertions. Same `// A1: ...` header pattern as Phase 6-β.

Mandatory coverage:

1. **A1** `SocialInteractionSystem::priority() == 134` and `tick_interval() == 1`.
2. **A2** `DecisionReason::SocialReason.as_str() == "social_reason"`.
3. **A3** `DecisionReason` has exactly 5 variants (exhaustive `match` check).
4. **A4** `CausalEvent::SocialInteractionStarted` and `SocialInteractionCompleted`
   are constructible with the locked field shapes; `id()` / `parent()` /
   `tick()` accessors round-trip; `channel()` returns `None` for both.
5. **A5** Idle agent + co-located partner agent + both with
   `Social.loneliness > SOCIAL_THRESHOLD` + no Needs/Construction breach →
   after one `AgentDecisionSystem::tick`, both agents are in
   `Seeking { Agent(other_id) }` AND both have an
   `AgentDecision { reason: SocialReason }` pushed to the tile log.
6. **A6** Same setup as A5 but with `Hunger.value > HUNGER_THRESHOLD` on one
   agent: Hunger wins on that agent (state = `Seeking { Food }`, reason =
   `HungerThresholdBreach`); the partner stays Idle since the Hunger-breached
   agent is no longer a valid social partner. (Proves Social is lowest
   priority — Needs win.)
7. **A7** Same setup as A5 but with an active co-located `ConstructionSite`
   that the agent could service: `ConstructionReason` wins (state =
   `Seeking { ConstructionSite }`). (Proves Construction wins over Social.)
8. **A8** Two agents in `Seeking { Agent(each other) }` co-located at same
   tile → after one `AgentDecisionSystem::tick`, both agents are in
   `Consuming { Agent(other) }` AND **exactly one** `SocialInteractionStarted`
   event with `agents == (smaller, larger)` is pushed to the tile log.
   `parent == Some(<AgentDecision.id of the smaller-AgentId agent's SocialReason
   decision>)`.
9. **A9** Two agents in `Consuming { Agent(each other) }` on same tile, fresh
   `interaction_progress`: after three `SocialInteractionSystem::tick`s,
   `interaction_progress[RelationshipKey(a,b)]` is removed (or zero), both
   agents are in `Idle`, both `Social.loneliness` decreased by
   `SOCIAL_CONSUME_AMOUNT`, `relationships[RelationshipKey(a,b)].familiarity ==
   FAMILIARITY_BUMP`, one `SocialInteractionCompleted` event with
   `familiarity_after == FAMILIARITY_BUMP`, `parent ==
   Some(<SocialInteractionStarted.id>)`.
10. **A10** Causal chain integrity walk: from `SocialInteractionCompleted`,
    trace `.parent()` → `SocialInteractionStarted`; trace its `.parent()` →
    `AgentDecision { reason: SocialReason }`. Three-link chain stitched.
11. **A11** Asymmetric-partner fallback: agent A in `Consuming { Agent(B) }`,
    agent B moved or in `Idle` state. After one `SocialInteractionSystem::tick`,
    agent A is in `Idle`, no panic, no new `SocialInteractionCompleted`,
    `interaction_progress` for `RelationshipKey(A,B)` is unchanged (or absent
    if never started).
12. **A12** `interaction_progress` dedup: two agents on same tile in mutual
    `Consuming`, after one tick, `interaction_progress[RelationshipKey]` is
    `1` (not `2`). Proves single-increment per pair per tick (P7β-11).
13. **A13** Familiarity saturation: pre-set
    `relationships[RelationshipKey(a,b)].familiarity = 0.95`. After one
    completion, `familiarity == 1.0` (saturated, not 1.05).
14. **A14** Phase 6-β regression: spawn an agent with `Hunger=60` and a
    co-located `ConstructionSite` AND a co-located partner with breached
    `Social.loneliness`. After one `AgentDecisionSystem::tick`, Hunger wins
    (state = `Seeking { Food }`). All other prior-phase regressions
    (Construction system, Sleep cycle) untouched.
15. **A15** Workspace test count grows by ≥12 in `harness_p7_beta`.

---

## Section 4 — Locale

**No new locale keys required.** Phase 7-β is backend only. Locale work
deferred to Phase 7-δ (optional, user mandate).

---

## Section 5 — Verification

```bash
cd rust && cargo build --workspace 2>&1 | tail -20
cd rust && cargo test --workspace 2>&1 | grep -E "test result|FAILED" | tail -20
cd rust && cargo test -p sim-test harness_p7_beta -- --nocapture 2>&1 | tail -50
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -20
```

Expected:
- `cargo build`: clean (new module + 2 new CausalEvent variants + 1 new
  DecisionReason variant + 2 new SimResources fields + 4 constants +
  agent_decision.rs takeover).
- `cargo test --workspace`: zero new failures. **All Phase 2-7-α harnesses
  still pass**.
- `cargo test -p sim-test harness_p7_beta`: ≥12 assertions pass.
- `cargo clippy`: zero new warnings.

---

## Section 6 — Lane

`--full`. Forced by: `sim-core/causal/event.rs` enum extensions +
`sim-systems/runtime/` new module + `agent_decision.rs` takeover +
`sim-engine/lib.rs` schema additions. Planning debate, Visual Verify
(no-godot-scope auto credit expected), FFI Chain check, Regression Guard,
and Evaluator all run.

---

## Section 7 — 인게임 확인사항

**None.** Phase 7-β adds no FFI, no rendering, no GDScript, no HUD, no
Locale keys. Pipeline VLM is expected to issue **no-godot-scope auto credit**
per v3.3.7 §2 (no `.gd`, `.gdshader`, `.tscn`, `.tres`, no `scripts/` or
`scenes/` path edits).

The agent-driven social interaction visual milestone is reserved for Phase 7-δ
(user-mandated optional sub-stage). Phase 7-β closes the backend interaction
loop; Phase 7-γ (next dispatch) adds the two-agent chronicle harness that
walks a full social cycle end-to-end.

---

## Self-check before dispatching the Generator

- [x] `SocialInteractionSystem` priority is `134`. Not 135, not 133.
- [x] `DecisionReason` gains exactly one variant (`SocialReason`).
- [x] `CausalEvent` gains exactly two variants
      (`SocialInteractionStarted`, `SocialInteractionCompleted`).
- [x] `SimResources` gains exactly two fields (`relationships`,
      `interaction_progress`). Both `HashMap`, both initialize empty.
- [x] Four constants exported from `agent_decision.rs`.
- [x] `BuildingPlaced` / `ConstructionStarted` / `ConstructionCompleted` /
      `AgentDecision` / `StampDirty` / `InfluenceChanged` variant bodies are
      **byte-for-byte unchanged**. Only accessor `match` arms grow.
- [x] `AgentState` enum body **byte-for-byte unchanged** (Phase 7-α already
      added `TargetKind::Agent(AgentId)`; β touches only FSM logic).
- [x] `BuildingStampSystem` (90) + `ConstructionSystem` (133) + all
      `sim-systems/runtime/influence/` + `sim-systems/runtime/construction/`
      files **untouched**.
- [x] No FFI change. No `sim-bridge` change.
- [x] No Locale change. No GDScript change.
- [x] `agent_decision.rs` Consuming-branch `Agent(_)` no-op stays intact.
      SocialInteractionSystem owns Consuming-state exit.
- [x] Harness has at least 14 numbered assertions mapped to P7β-1 through
      P7β-17 + regression coverage.
- [x] Causal chain: `AgentDecision{SocialReason} → SocialInteractionStarted
      → SocialInteractionCompleted` — exactly three links, parent ids stitched.
- [x] Single `SocialInteractionStarted` per handshake (smaller-AgentId emits,
      dedup verified by A8).
- [x] Single `interaction_progress` increment per pair per tick
      (dedup verified by A12).

---

## §β Re-plan — Option A: SocialDecaySystem priority 135 + 22-assertion enforcement

**Provenance**: 4 dispatches × max attempts stalled. Codex Evaluator iterated
demands toward the **already-locked 22-assertion plan** that Generator
under-implemented at 17. Plan §β re-plan adds the runtime path required for
canonical social emergence (`SocialDecaySystem`) and re-affirms the original
22-assertion contract.

See `.harness/plans/phase7.md` §β re-plan section for full details.

### Re-plan additions (Option A locked)

**New runtime system**: `SocialDecaySystem` priority **135**, tick_interval 1,
in `rust/crates/sim-systems/src/runtime/needs/social_decay.rs`. Direct
structural mirror of `SleepDecaySystem` (priority 132). Walks all `&mut Social`
and calls `Social::tick()`. Lands as the 4th need-decay system in the
register_needs_systems helper.

**Registration order**:
```
90  BSS → 100 IUS → 110 AIS → 120 Move → 125 Decision →
130 HungerDecay → 131 ThirstDecay → 132 SleepDecay →
133 Construction → 134 SocialInteraction → 135 SocialDecay →
1000 Visualization
```

The 135 slot keeps `SocialInteractionSystem` reading **pre-decay** `loneliness`
for handshake/progress/completion (consistent with how AgentDecisionSystem at
125 reads pre-decay needs for all four), and the decay applies afterward.

**`make_stage1_engine` extension**: Stage-1 agent archetype gains a positive
`Social` growth_rate (suggested `Social::new(0.0, 0.04)`) so canonical
4380-tick runs produce loneliness breach naturally. Math: 4380 × 0.04 =
175.2 → saturation by ~tick 2500.

**Locked 22-assertion contract**:
- A1-A17: unchanged.
- A18: canonical 4380-tick emergent run on `make_stage1_engine(42, 20)` →
  ≥1 `SocialInteractionStarted` + ≥1 `SocialInteractionCompleted` + ≥1
  non-zero `relationships[k].familiarity`. **No manual lonely-pair
  preconditioning permitted**.
- A19: 3-seed sweep (42, 100, 200) — each seed independently hits the A18
  floor.
- A20: same-seed determinism — `(tick, SocialInteractionStarted.id)`
  sequence + `relationships` HashMap snapshot identical across two runs of
  the same seed.
- A21: `SocialInteractionSystem` registered through
  `register_default_runtime_systems` (production path, not just via the
  social-only helper).
- A22: real partner despawn mid-interaction — agent A in
  `Consuming { Agent(B) }`, despawn B mid-progress; A returns to Idle on
  next `SocialInteractionSystem::tick` without panic; no
  `SocialInteractionCompleted` emitted for the despawned pair.

**Deterministic ordering**: `SocialInteractionSystem::tick` must process
candidate pairs in **sorted canonical `RelationshipKey` order** before any
side effect (event push, familiarity bump, state transition). HashMap
iteration becomes a snapshot; ordering is applied at the side-effect loop.

### Re-plan harness extensions

`harness_p7_beta_social_system.rs` final form:
- 17 existing assertions tightened per Evaluator feedback (A2, A4, A7, A8,
  A10, A12, A13, A15 as flagged in r4 attempt 1 review).
- 5 new assertions A18-A22 added with the locks above.
- Drop the `total_stone` / `total_wood` fabrications in A14/A17 — use the
  plan's locked DecisionReason-count ±15% thresholds across pre-social
  needs.
- Diagnostic `--nocapture` counter for canonical social-path execution
  (`agents_doing_social_interaction=N`) with `N > 0`.

### Files modified (re-plan extension)

In addition to the original Phase 7-β file list:
- `rust/crates/sim-systems/src/runtime/needs/social_decay.rs` (NEW, ~108 LoC)
- `rust/crates/sim-systems/src/runtime/needs/mod.rs` (add `pub mod social_decay;
  pub use social_decay::SocialDecaySystem;`)
- `rust/crates/sim-systems/src/lib.rs` (`register_needs_systems` registers
  `SocialDecaySystem`; `register_default_runtime_systems` doc updated for the
  4-needs sequence)
- `rust/crates/sim-systems/src/runtime/social/social_interaction_system.rs`
  (sorted-key processing before side effects)
- `rust/crates/sim-test/tests/harness_p7_beta_social_system.rs`
  (`make_stage1_engine` Social growth_rate + 5 new assertions + tightened
  existing assertions + diagnostic counter)

### Verification (re-plan)

```bash
cd rust && cargo test --workspace 2>&1 | grep -E "test result|FAILED" | tail -10
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -3
cd rust && cargo test -p sim-test harness_p7_beta -- --nocapture 2>&1 | tail -30
```

Expected:
- Workspace tests pass + clippy clean.
- `harness_p7_beta_social_system` 22-assertion harness passes.
- `SocialDecaySystem` metadata assertion ( `priority == 135`, `tick_interval == 1`).
- `make_stage1_engine(42, 20)` 4380-tick run produces ≥1 social interaction
  cycle naturally.
- Phase 2-7-α regressions remain CLEAN.
