# P6-β — ConstructionSystem + ConstructionStarted/Completed CausalEvent + DecisionReason::ConstructionReason + AgentDecisionSystem extension

> Lane: `--full` (sim-core `CausalEvent` enum extension + sim-systems new runtime
> module + agent_decision.rs takeover from Phase 6-α inert placeholders all
> force full lane).
> Scope: Wire the agent-driven construction tick loop. An idle agent with all
> Needs unbreached and a co-located `ConstructionSite` transitions through the
> full FSM, progress increments per tick of contact, and a 4-link causal chain
> closes on completion.
> Governance: v3.3.17. Visual: backend-leaning (no new HUD surface,
> no `.gd`/`.gdshader`/`.tscn`/`.tres`) — Pipeline VLM no-godot-scope auto
> credit expected.

---

## Section 1 — Implementation Intent

Phase 6-α landed the data substrate (`BuildingBlueprint`, `ConstructionSite`,
`TargetKind::ConstructionSite`) with two **explicitly inert** match-arm
placeholders in `agent_decision.rs`:

- Line 213-221: `TargetKind::ConstructionSite => false` in the Seeking-branch
  `has_resource` check.
- Line 287-291: `TargetKind::ConstructionSite => { /* no-op */ }` in the
  Consuming-branch.

Both placeholders document themselves as "β scope — runtime resolution lands
in Phase 6-β". This dispatch is that takeover.

Phase 6-β simultaneously:
1. **Activates** the two inert placeholders in `agent_decision.rs`:
   - Idle branch gains a 4th cascade step: after Hunger/Thirst/Fatigue checks
     fail, check for a co-located `ConstructionSite` entity — if present and
     not yet complete, transition to `Seeking { TargetKind::ConstructionSite }`
     and emit `CausalEvent::AgentDecision { reason: DecisionReason::ConstructionReason }`.
   - Seeking branch flips the `false` placeholder to a proper co-location
     check (HashMap built from a pre-loop world query).
   - Consuming branch **stays** a no-op — ConstructionSystem at priority 133
     owns all progress mutation and absent-site fallback.
2. **Introduces** the new `ConstructionSystem` at priority 133 (slot
   immediately after `SleepDecaySystem`). Its responsibilities:
   - For each agent in `AgentState::Consuming { target: TargetKind::ConstructionSite }`,
     locate the co-located `ConstructionSite` and either advance progress or
     emit completion+despawn.
   - Emit `CausalEvent::ConstructionCompleted` followed by
     `CausalEvent::BuildingPlaced` on the completion tick, chained as
     `ConstructionCompleted.parent = ConstructionStarted.id`,
     `BuildingPlaced.parent = ConstructionCompleted.id`.
   - Handle the **absent-site fallback** (race condition: agent in
     Consuming{ConstructionSite} whose target entity was already despawned):
     transition agent to `Idle` without panic, without progress increment.
3. **Extends** `CausalEvent` with two new variants:
   - `ConstructionStarted { id, parent, blueprint, position, tick }`
   - `ConstructionCompleted { id, parent, blueprint, position, tick }`
4. **Extends** `DecisionReason` with the 4th variant `ConstructionReason`.
5. **Emits** `CausalEvent::ConstructionStarted` from `AgentDecisionSystem` on
   the `Seeking { ConstructionSite } → Consuming { ConstructionSite }`
   transition (the natural emission point — the agent has the position
   context and the parent AgentDecision id is fresh in the same tile's log).

The completion chain that closes the "왜?" UI walk:

```
AgentDecision { reason: ConstructionReason, parent: <InfluenceChanged?> }
    ↓ (parent ref)
ConstructionStarted { parent: Some(AgentDecision.id), blueprint, position }
    ↓ (parent ref)
ConstructionCompleted { parent: Some(ConstructionStarted.id), blueprint, position }
    ↓ (parent ref)
BuildingPlaced { parent: Some(ConstructionCompleted.id), position, radius }
```

After P6-β:
- `sim_systems::runtime::construction::ConstructionSystem` exists at priority 133.
- `sim_core::components::DecisionReason` has **four** variants
  (Hunger/Thirst/Fatigue/Construction).
- `sim_core::components::CausalEvent` has **six** variants
  (BuildingPlaced/StampDirty/InfluenceChanged/AgentDecision/
  ConstructionStarted/ConstructionCompleted).
- `AgentDecisionSystem` FSM extended with the 4th cascade (Construction is the
  lowest-priority drive — Needs always win).
- A ≥12-assertion harness `harness_p6_beta_construction_system.rs` proves the
  full agent FSM cycle, the 4-link causal chain, and the absent-site fallback.

---

## Section 2 — Locked facts (from pre-grep — must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| P6β-1: ConstructionSystem priority | Planning §2.2 + system priority chain | `priority() = 133`. Strictly after `SleepDecaySystem` (132), strictly before `InfluenceVisualizationSystem` (1000). |
| P6β-2: ConstructionSystem tick_interval | Planning §2.2 + Phase 5 needs precedent | `tick_interval() = 1`. Every tick. |
| P6β-3: ConstructionStarted struct | Planning §2.2 + CausalEvent precedent (BuildingPlaced/AgentDecision use `position: (u32, u32)`) | `ConstructionStarted { id: EventId, parent: Option<EventId>, blueprint: BuildingBlueprint, position: (u32, u32), tick: u64 }`. **Precedent-driven amendment**: `position` is `(u32, u32)` tuple (not the `Position` struct) to match existing CausalEvent variants. `blueprint` is the full embedded snapshot (BuildingBlueprint is Copy). |
| P6β-4: ConstructionCompleted struct | Planning §2.2 + same precedent + Entity-lifetime rationale | `ConstructionCompleted { id: EventId, parent: Option<EventId>, blueprint: BuildingBlueprint, position: (u32, u32), tick: u64 }`. **Precedent-driven amendment**: dropped `building_entity: hecs::Entity` from the planning spec — the entity is despawned in the same call site, so storing its ID would be misleading (recycled hecs handle). Position is the durable identifier; the embedded blueprint snapshot mirrors ConstructionStarted exactly. |
| P6β-5: DecisionReason::ConstructionReason | Planning §2.2 + existing pattern | Append `ConstructionReason` as the 4th `DecisionReason` variant after `FatigueThresholdBreach`. `as_str()` returns `"construction_reason"` (matches snake_case discriminator style: `"hunger_threshold_breach"` etc.). |
| P6β-6: AgentDecisionSystem Idle-branch cascade | Planning §2.2 + agent_decision.rs lines 147-194 existing structure | Extend the existing `breached = if/else if/else if/else` ladder with a 4th `else if` that checks co-located ConstructionSite presence (via a HashMap built from a pre-loop world query). If true and no Need has breached: `Some((TargetKind::ConstructionSite, DecisionReason::ConstructionReason))`. Construction is **lowest priority** — needs always win. |
| P6β-7: Co-location detection | Planning §2.2 + Phase 5 same-tile precedent | Build `construction_sites: HashMap<(u32, u32), Entity>` at the top of `AgentDecisionSystem::tick` via `world.query::<&ConstructionSite>()` — one entry per active site, keyed by `(site.position.x, site.position.y)`. The agent loop reads from this map. ConstructionSystem MAY rebuild the same map locally (or use a separate strategy) — the maps are NOT shared; each system owns its own snapshot per tick. |
| P6β-8: Agent FSM Seeking → Consuming + ConstructionStarted emission | Planning §2.2 + parent-chain requirement | On the `Seeking { ConstructionSite } → Consuming { ConstructionSite }` transition (when the agent's tile contains an active ConstructionSite per the same pre-built HashMap), emit `CausalEvent::ConstructionStarted` BEFORE flipping the state. `parent = Some(<most recent same-tile AgentDecision id matching this agent>)` or `None` if not found. Position is the agent's current tile. |
| P6β-9: Agent FSM Consuming branch | Planning §2.2 + Phase 6-α inert preservation contract | Leave the existing `TargetKind::ConstructionSite => { /* no-op */ }` no-op in `AgentDecisionSystem` untouched. ConstructionSystem at priority 133 (next system in the same tick) owns Consuming-state mutation. |
| P6β-10: ConstructionSystem tick logic | Planning §2.2 verbatim | (a) Build `sites: HashMap<(u32,u32), Entity>` from `world.query::<&ConstructionSite>()`. (b) For each agent in `Consuming { ConstructionSite }`: look up site by agent's `Position`. (c) If absent: `state = Idle` (no panic, no progress change). (d) If present and not complete: `site.advance()` (returns true exactly when completion edge reached). (e) On completion edge: emit `ConstructionCompleted` + `BuildingPlaced` (in that order, parent chain), record the site Entity in a deferred-despawn list, `state = Idle`. (f) After the agent loop: despawn all collected entities. |
| P6β-11: BuildingPlaced from ConstructionSystem | Planning §2.2 "spawn BuildingPlaced event" | ConstructionSystem pushes `CausalEvent::BuildingPlaced` directly to `resources.causal_log` (same `causal_log.push(tile_idx, ...)` path AgentDecisionSystem uses). The existing FFI-driven BuildingStampSystem (priority 90) path is **NOT** changed — both paths coexist. The Phase 6-β-emitted BuildingPlaced uses `radius = 0` (agent-construction has no influence radius in Phase 6 minimal scope; influence stamping is left for the BSS-driven path or future phases). |
| P6β-12: register helper | Phase 5 needs precedent | Add `pub fn register_construction_systems(engine: &mut SimEngine)` to `sim-systems/src/lib.rs` that calls `engine.register_runtime_system(Box::new(ConstructionSystem::new()))`. Mirror the form of `register_needs_systems` if present, or follow `AgentDecisionSystem` registration if `register_needs_systems` does not exist. |
| P6β-13: BuildingStampSystem priority 90 untouched | Planning §2.2 cross-cutting concerns | `BuildingStampSystem` is not modified. Phase 2 substrate intact. No new file edits to `sim-systems/src/runtime/influence/building_stamp.rs`. |
| P6β-14: serde discipline | Existing CausalEvent precedent | `CausalEvent` derives `Debug, Clone, PartialEq` only (NOT `Serialize`/`Deserialize` — pre-existing constraint, see causal/event.rs:22 "Copy is intentionally NOT derived"). The two new variants follow the same derives. The new `DecisionReason::ConstructionReason` variant inherits the existing `Debug, Clone, Copy, PartialEq, Eq, Hash` set. Test extensions cover all six final CausalEvent variants and all four final DecisionReason variants. |
| Harness assertion count | Planning §2.2 + Phase 6-α precedent | **≥12 assertions** in `harness_p6_beta_construction_system.rs`. Each locked fact above gets at least one corresponding assertion. |

**Determinism rationale** (axiom #2 — verified before assuming):
1. `ConstructionSystem::tick` iterates a HashMap by hecs Entity → position
   snapshot. HashMap order is not deterministic but the **mutations** are
   commutative (each agent processes its own site; sites are unique per
   position). Cross-agent ordering is irrelevant to test outcomes.
2. `CausalEvent::ConstructionStarted/Completed` parent linkage uses the same
   "most recent same-tile log entry" pattern as `AgentDecision.parent` lookup.
3. `BuildingPlaced` direct emission from ConstructionSystem uses the existing
   `causal_log.push` infrastructure — same path tested by Phase 3-α/β.
4. The 4th cascade step (`else if construction_site_at(pos)`) is added
   AFTER the three existing Need checks — no reordering of existing
   deterministic ordering.

---

## Section 3 — What to build (5-file scope + 1 test file)

### 3.1 `rust/crates/sim-core/src/causal/event.rs` (MODIFY)

Two locus changes:

**(a) `DecisionReason` enum** — append `ConstructionReason` variant after
`FatigueThresholdBreach`:

```rust
pub enum DecisionReason {
    HungerThresholdBreach,
    ThirstThresholdBreach,
    FatigueThresholdBreach,
    /// Agent transitioned from Idle to Seeking{ConstructionSite} after
    /// detecting a co-located active ConstructionSite while no Need was
    /// breached. V7 Phase 6-β / P6β-5. Construction is the lowest-priority
    /// drive — Hunger/Thirst/Fatigue always win.
    ConstructionReason,
}
```

Add the `as_str()` arm:

```rust
DecisionReason::ConstructionReason => "construction_reason",
```

**(b) `CausalEvent` enum** — append two variants after `AgentDecision`:

```rust
pub enum CausalEvent {
    BuildingPlaced { ... },
    StampDirty { ... },
    InfluenceChanged { ... },
    AgentDecision { ... },

    /// Agent transitioned from Seeking{ConstructionSite} to
    /// Consuming{ConstructionSite}. Emitted by AgentDecisionSystem the moment
    /// an agent reaches an active construction site. parent links to the
    /// originating AgentDecision{ConstructionReason} on the same tile.
    /// V7 Phase 6-β / P6β-3.
    ConstructionStarted {
        id: EventId,
        parent: Option<EventId>,
        blueprint: crate::components::BuildingBlueprint,
        position: (u32, u32),
        tick: u64,
    },

    /// Construction progress reached required_progress on this tick. Emitted
    /// by ConstructionSystem before BuildingPlaced (which is the chain's
    /// final closing event). parent links to the originating ConstructionStarted.
    /// V7 Phase 6-β / P6β-4.
    ConstructionCompleted {
        id: EventId,
        parent: Option<EventId>,
        blueprint: crate::components::BuildingBlueprint,
        position: (u32, u32),
        tick: u64,
    },
}
```

**(c) Accessor methods** (`id()`, `parent()`, `tick()`, `channel()`) — extend
all four `match` arms to handle the two new variants. `channel()` returns
`None` for both (channel-agnostic, like BuildingPlaced and AgentDecision).

**(d) `#[cfg(test)] mod tests`** — extend with at minimum:
- `construction_started_records_fields`
- `construction_completed_records_fields`
- `decision_reason_construction_as_str` (covers the new arm).

### 3.2 `rust/crates/sim-systems/src/runtime/construction/mod.rs` (NEW)

```rust
//! V7 Phase 6-β / P6β-1 — Agent-driven construction runtime.
//!
//! Owns the progress-and-completion side of the construction loop.
//! `AgentDecisionSystem` (priority 125) handles all AgentState transitions
//! INTO and THROUGH Seeking/Consuming for `TargetKind::ConstructionSite`;
//! this system handles the OUT transition (Consuming → Idle) plus all
//! ConstructionSite progress mutation.

pub mod construction_system;

pub use construction_system::ConstructionSystem;
```

### 3.3 `rust/crates/sim-systems/src/runtime/construction/construction_system.rs` (NEW)

```rust
//! `ConstructionSystem` at priority 133 — the runtime tick of the
//! Phase 6 agent-driven construction loop.
//!
//! Tick ordering context:
//!
//! ```text
//! 125 AgentDecisionSystem        (Idle→Seeking, Seeking→Consuming, ConstructionStarted emit)
//! 130 HungerDecaySystem
//! 131 ThirstDecaySystem
//! 132 SleepDecaySystem
//! 133 ConstructionSystem         ← this file
//! ```

use std::collections::HashMap;
use hecs::{Entity, World};
use sim_core::causal::CausalEvent;
use sim_core::components::{
    AgentState, ConstructionSite, Position, TargetKind,
};
use sim_engine::{RuntimeSystem, SimResources};

#[derive(Debug, Default)]
pub struct ConstructionSystem;

impl ConstructionSystem {
    pub fn new() -> Self { Self }
}

impl RuntimeSystem for ConstructionSystem {
    fn name(&self) -> &str { "ConstructionSystem" }
    fn priority(&self) -> u32 { 133 }
    fn tick_interval(&self) -> u64 { 1 }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let width = resources.tile_grid.width;
        if width == 0 || resources.tile_grid.height == 0 { return; }
        let tick = resources.current_tick;

        // (a) Snapshot site positions for fast agent-side lookup.
        let mut sites_by_pos: HashMap<(u32, u32), Entity> = HashMap::new();
        {
            let mut q = world.query::<&ConstructionSite>();
            for (e, site) in q.iter() {
                sites_by_pos.insert((site.position.x, site.position.y), e);
            }
        }

        // (b) Collect agent decisions for deferred application.
        struct Pending {
            agent_entity: Entity,
            agent_position: (u32, u32),
            site_entity: Option<Entity>,    // None = absent fallback
            completion: Option<(crate::CompletionEdge)>,  // see below
        }
        // Actually use a simpler vector of operations.

        let mut completions: Vec<(Entity, (u32, u32), sim_core::components::BuildingBlueprint, Option<u64>)> = Vec::new();
        // (site_entity_to_despawn, position, blueprint_snapshot, parent_construction_started_id)
        let mut agent_idle_resets: Vec<Entity> = Vec::new();
        let mut progress_updates: Vec<Entity> = Vec::new();

        {
            let mut q = world.query::<(&Position, &mut AgentState)>();
            for (agent_entity, (pos, state)) in q.iter() {
                if !matches!(*state, AgentState::Consuming { target: TargetKind::ConstructionSite }) {
                    continue;
                }
                let key = (pos.x, pos.y);
                match sites_by_pos.get(&key).copied() {
                    None => {
                        // (c) absent-site fallback
                        *state = AgentState::Idle;
                        agent_idle_resets.push(agent_entity);
                    }
                    Some(site_entity) => {
                        progress_updates.push(site_entity);
                        // We'll detect completion outside the agent borrow.
                    }
                }
            }
        }

        // (d) Apply progress + collect completion edges.
        let mut sites_to_despawn: Vec<Entity> = Vec::new();
        for site_entity in progress_updates {
            let (just_completed, blueprint_snapshot, position) = {
                let mut site = world
                    .get::<&mut ConstructionSite>(site_entity)
                    .expect("site exists (gathered same tick)");
                let edge = site.advance();
                (edge, site.blueprint, (site.position.x, site.position.y))
            };
            if just_completed {
                // (e) Emit ConstructionCompleted then BuildingPlaced.
                let tile_idx = (position.1 as usize) * (width as usize) + (position.0 as usize);

                let parent_started = resources
                    .causal_log
                    .get(tile_idx)
                    .and_then(|log| log.as_slice().iter().rev().find_map(|ev| match ev {
                        CausalEvent::ConstructionStarted { id, .. } => Some(*id),
                        _ => None,
                    }));

                let completed_id = resources.issue_event_id();
                resources.causal_log.push(
                    tile_idx,
                    CausalEvent::ConstructionCompleted {
                        id: completed_id,
                        parent: parent_started,
                        blueprint: blueprint_snapshot,
                        position,
                        tick,
                    },
                );
                let placed_id = resources.issue_event_id();
                resources.causal_log.push(
                    tile_idx,
                    CausalEvent::BuildingPlaced {
                        id: placed_id,
                        parent: Some(completed_id),
                        position,
                        radius: 0,
                        tick,
                    },
                );
                sites_to_despawn.push(site_entity);

                // Reset the agent at this position to Idle.
                // (Agent FSM might have multiple agents on the same site; reset all.)
                let mut q = world.query::<(&Position, &mut AgentState)>();
                for (e, (p, s)) in q.iter() {
                    if p.x == position.0 && p.y == position.1
                        && matches!(*s, AgentState::Consuming { target: TargetKind::ConstructionSite })
                    {
                        *s = AgentState::Idle;
                        agent_idle_resets.push(e);
                    }
                }
            }
        }

        // (f) Despawn completed sites after agent state updates.
        for site_entity in sites_to_despawn {
            let _ = world.despawn(site_entity);
        }

        // `agent_idle_resets` is no longer needed beyond local tracking; kept
        // for potential test introspection but the state mutation already
        // happened in-place above.
        let _ = agent_idle_resets;
    }
}

#[cfg(test)]
mod tests { /* priority + tick_interval metadata test */ }
```

**Note for the Generator**: the pseudocode above shows the structure but the
Generator MUST adapt it to compile cleanly under hecs' borrow rules and the
existing `sim_engine::RuntimeSystem` trait shape. Specifically:
- The `world.get::<&mut ConstructionSite>(entity)` pattern requires hecs
  ≥ 0.10. Verify the actual API at write time.
- If the borrow checker rejects the nested `world.query` inside the
  completion loop, restructure to collect (entity, blueprint, position)
  tuples first, drop the query, then iterate the collected vec.
- Test the metadata-only `priority()` and `tick_interval()` assertions in
  the inline `#[cfg(test)] mod tests`.

### 3.4 `rust/crates/sim-systems/src/runtime/mod.rs` (MODIFY)

Add `pub mod construction;` to the module list, alphabetically positioned
after `agent` and before `decision`:

```rust
pub mod agent;
pub mod construction;   // ← NEW
pub mod decision;
pub mod influence;
pub mod needs;
```

(Adjust if the actual existing ordering differs; preserve alphabetical
ordering of the public module list.)

### 3.5 `rust/crates/sim-systems/src/lib.rs` (MODIFY)

Add a `register_construction_systems` helper next to the existing
needs/decision registration helpers (find the existing pattern at write time
— it follows the form `pub fn register_X_systems(engine: &mut SimEngine)`).

```rust
pub fn register_construction_systems(engine: &mut sim_engine::SimEngine) {
    engine.register_runtime_system(Box::new(
        crate::runtime::construction::ConstructionSystem::new(),
    ));
}
```

If `DEFAULT_RUNTIME_SYSTEMS` or a similar central registration list exists,
add `ConstructionSystem` to it as well, immediately after `SleepDecaySystem`
to match the priority ordering.

### 3.6 `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs` (MODIFY)

Three loci of change inside `AgentDecisionSystem::tick`:

**(a) Top of tick — build co-location map** (after the width/height guard,
before the existing agent query). Mirror the snapshot pattern documented in
3.3 above:

```rust
let construction_sites: HashMap<(u32, u32), hecs::Entity> = {
    let mut sites = HashMap::new();
    let mut q = world.query::<&ConstructionSite>();
    for (e, site) in q.iter() {
        sites.insert((site.position.x, site.position.y), e);
    }
    sites
};
```

**(b) Idle branch — 4th cascade step.** Extend the existing
`breached = if/else if/else if/else` ladder with a 4th `else if`:

```rust
let breached = if /* hunger */ { ... }
    else if /* thirst */ { ... }
    else if /* sleep */ { ... }
    else if construction_sites.contains_key(&(pos.x, pos.y)) {
        Some((TargetKind::ConstructionSite, DecisionReason::ConstructionReason))
    } else {
        None
    };
```

The rest of the Idle-branch emission code (parent lookup, AgentDecision
push, state flip) requires no changes — it already handles the
`Some((target, reason))` shape generically.

**(c) Seeking branch — flip the Phase 6-α `false` placeholder + add
ConstructionStarted emission on the transition.** Replace lines 213-221:

```rust
TargetKind::ConstructionSite => {
    construction_sites.contains_key(&(pos.x, pos.y))
}
```

When `has_resource == true` for the ConstructionSite case, the existing
code transitions `state = Consuming { target }`. ADD an emission
immediately before the state flip (inside the `if has_resource` block,
conditional on `target == ConstructionSite`):

```rust
if has_resource {
    if matches!(target, TargetKind::ConstructionSite) {
        let tile_idx = (pos.y as usize) * (width as usize) + (pos.x as usize);
        let blueprint_snapshot = construction_sites
            .get(&(pos.x, pos.y))
            .and_then(|&site_entity| {
                world.get::<&ConstructionSite>(site_entity).ok().map(|s| s.blueprint)
            });
        // If we successfully snapshotted the blueprint, emit the chain start.
        if let Some(blueprint) = blueprint_snapshot {
            // Parent: most recent same-tile AgentDecision for this agent.
            let parent = resources
                .causal_log
                .get(tile_idx)
                .and_then(|log| log.as_slice().iter().rev().find_map(|ev| match ev {
                    CausalEvent::AgentDecision { id, agent: a, .. } if *a == agent.id => Some(*id),
                    _ => None,
                }));
            let id = resources.issue_event_id();
            resources.causal_log.push(
                tile_idx,
                CausalEvent::ConstructionStarted {
                    id,
                    parent,
                    blueprint,
                    position: (pos.x, pos.y),
                    tick,
                },
            );
        }
    }
    *state = AgentState::Consuming { target };
}
```

The borrow checker may require restructuring (`world.get` inside the
agent-query iteration is a nested borrow). If so, the Generator should
either:
- Hoist the blueprint lookup BEFORE entering the Seeking branch's
  has_resource check by pre-snapshotting blueprints alongside entities in
  step (a), e.g.,
  `sites: HashMap<(u32,u32), (Entity, BuildingBlueprint)>`, OR
- Defer the ConstructionStarted emission to a post-loop pass over
  collected transitions.

Either path is acceptable as long as the parent linkage to the
same-tile, same-agent AgentDecision is preserved.

**(d) Consuming branch — leave the no-op intact.** Lines 287-291 stay
exactly as Phase 6-α set them. ConstructionSystem owns Consuming-state
exit semantics.

### 3.7 `rust/crates/sim-test/tests/harness_p6_beta_construction_system.rs` (NEW)

≥12 numbered assertions. Same `// A1: ...` header pattern as Phase 6-α.

Mandatory assertion coverage:

1. **A1** `ConstructionSystem::priority() == 133` and `tick_interval() == 1`.
2. **A2** `DecisionReason::ConstructionReason.as_str() == "construction_reason"`.
3. **A3** `DecisionReason` has exactly 4 variants (exhaustive match check).
4. **A4** `CausalEvent::ConstructionStarted` and `ConstructionCompleted`
   are constructible with the locked field shapes; `id()` / `parent()` /
   `tick()` accessors round-trip; `channel()` returns `None`.
5. **A5** Agent in `Idle` state, no Needs breached, with a co-located
   `ConstructionSite` → after one `AgentDecisionSystem::tick`,
   `state == Seeking { ConstructionSite }` and a
   `CausalEvent::AgentDecision { reason: ConstructionReason }` was pushed
   to the tile log.
6. **A6** Same setup as A5 but with `Hunger.value > HUNGER_THRESHOLD`:
   Hunger wins; state is `Seeking { Food }`; the AgentDecision reason is
   `HungerThresholdBreach`. (Proves Construction is lowest-priority.)
7. **A7** Agent in `Seeking { ConstructionSite }` on the site's tile →
   after one `AgentDecisionSystem::tick`,
   `state == Consuming { ConstructionSite }` AND
   `CausalEvent::ConstructionStarted` was pushed to the tile log with
   `parent == Some(<AgentDecision.id>)` and `blueprint == site.blueprint`.
8. **A8** Agent in `Consuming { ConstructionSite }` on the site's tile,
   site `required_progress == 3`: after three `ConstructionSystem::tick`s,
   `site.progress == 3`, `state == Idle`, two new events appear in the
   tile log on the completion tick:
   `CausalEvent::ConstructionCompleted` with
   `parent == Some(<ConstructionStarted.id>)` followed by
   `CausalEvent::BuildingPlaced` with
   `parent == Some(<ConstructionCompleted.id>)`. The site entity is
   despawned (no longer queryable in the world).
9. **A9** Causal chain integrity end-to-end: walking
   `BuildingPlaced.parent` backwards yields the exact chain
   `BuildingPlaced → ConstructionCompleted → ConstructionStarted →
   AgentDecision{ConstructionReason}` (four events, parent ids stitched).
10. **A10** Absent-site fallback: agent in
    `Consuming { ConstructionSite }` whose site entity is despawned via
    `world.despawn(site_entity)` before `ConstructionSystem::tick` runs.
    After one tick: agent state is `Idle`, no panic, no new events
    emitted (or only the agent-state change with no progress/completion
    chain).
11. **A11** Construction does not advance when no agent is
    `Consuming { ConstructionSite }`. Site with `required_progress == 5`,
    no agent anywhere near it: after 10 `ConstructionSystem::tick`s,
    `site.progress == 0`, no `ConstructionCompleted` event.
12. **A12** Phase 5 regression: spawn an agent with full Need set
    (`Hunger { value: 60, growth_rate: 0 }`, all others at 0) and a
    food_tile + a co-located ConstructionSite. After one
    `AgentDecisionSystem::tick`, Hunger wins: `state == Seeking { Food }`,
    AgentDecision reason is `HungerThresholdBreach`. The ConstructionSite
    is untouched. **Phase 5 cascade ordering preserved.**
13. **A13** Phase 6-α regression: a fresh `ConstructionSite` has
    `progress == 0`, `is_complete() == false`. After one
    `ConstructionSystem::tick` with no co-located agent in Consuming
    state, the site is unchanged. (Proves the new system does not
    spuriously advance idle sites.)

---

## Section 4 — Locale

**No new locale keys required.** Phase 6-β is backend only — no new HUD
surface. Locale work for construction-related UI is deferred to Phase 6-δ
(optional, gated on user mandate per planning §2.4).

---

## Section 5 — Verification

```bash
cd rust && cargo build --workspace 2>&1 | tail -20
cd rust && cargo test --workspace 2>&1 | grep -E "test result|FAILED" | tail -20
cd rust && cargo test -p sim-test harness_p6_beta -- --nocapture 2>&1 | tail -50
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -20
```

Expected:
- `cargo build`: clean (new module + two new CausalEvent variants + one new
  DecisionReason variant + agent_decision.rs takeover).
- `cargo test --workspace`: zero new failures. **All Phase 2-5 + Phase 6-α
  harness tests still pass** (T7.10.A-F + harness_phase2_substantial +
  harness_p3_* + harness_p4_* + harness_p5_alpha/beta/gamma +
  harness_p6_alpha_construction_components all CLEAN).
- `cargo test -p sim-test harness_p6_beta`: ≥12 assertions pass.
- `cargo clippy`: zero new warnings.

---

## Section 6 — Lane

`--full`. Forced by: `sim-core/src/causal/event.rs` enum extension +
`sim-systems/src/runtime/` new module + `agent_decision.rs` takeover.
Planning debate, Visual Verify (no-godot-scope auto credit expected), FFI
Chain check, Regression Guard, and Evaluator all run.

---

## Section 7 — 인게임 확인사항

**None.** Phase 6-β adds no runtime FFI methods, no rendering changes, no
HUD surface. Pipeline VLM is expected to issue **no-godot-scope auto
credit** per v3.3.7 §2 (no `.gd`, `.gdshader`, `.tscn`, `.tres`, no
`scripts/` or `scenes/` path edits).

The agent-driven construction visual milestone is reserved for Phase 6-δ
(user-mandated optional sub-stage). Phase 6-β closes the backend
construction loop; Phase 6-γ (next dispatch) adds the chronicle harness
that walks one agent through a full build cycle.

---

## Self-check before dispatching the Generator

- [x] `ConstructionSystem` priority is `133`. Not 134, not 132.
- [x] `DecisionReason` enum gains exactly one variant (`ConstructionReason`).
- [x] `CausalEvent` enum gains exactly two variants
      (`ConstructionStarted`, `ConstructionCompleted`).
- [x] `BuildingPlaced` variant body is **byte-for-byte unchanged**. Only
      its emission site grows (ConstructionSystem now also emits it).
- [x] `AgentState` enum body is **byte-for-byte unchanged**. (Phase 6-α
      already added the `TargetKind::ConstructionSite` variant — β touches
      only the FSM logic, not the state shape.)
- [x] `BuildingStampSystem` (priority 90) and all `sim-systems/runtime/influence/`
      files are **untouched**. Phase 2 substrate intact.
- [x] No FFI change. No `sim-bridge` change.
- [x] No Locale change. No GDScript change.
- [x] `agent_decision.rs` Consuming-branch ConstructionSite no-op stays
      intact (lines 287-291 of the post-Phase-6-α version). ConstructionSystem
      owns Consuming-state exit.
- [x] Harness has at least 13 numbered assertions mapped to P6β-1 through
      P6β-14 + regression coverage.
- [x] Causal chain: `AgentDecision{ConstructionReason} → ConstructionStarted
      → ConstructionCompleted → BuildingPlaced` — exactly four links, parent
      ids stitched.
