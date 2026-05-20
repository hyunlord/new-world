//! V7 Phase 8-γ — MemorySystem (priority 136).
//!
//! Four-phase tick:
//! 0. **Recalled set** — collect `(AgentId, EventId)` pairs from
//!    `CausalEvent::MemoryRecalled` events whose `tick == current_tick`.
//!    These were emitted by `AgentDecisionSystem` (priority 125) earlier
//!    in the same tick and represent entries that just triggered a cascade
//!    flip.
//! 1. **Selective decay pass** — apply linear decay to every entry whose
//!    `(AgentId, EventId)` pair is NOT in the recalled set. Recalled entries
//!    are held back from decay so that Phase 3 reinforcement applies to the
//!    pre-decay salience, satisfying the locked A9 formula
//!    `(sal_before + REINFORCEMENT_BOOST).min(1.0)` with 1e-9 tolerance.
//! 2. **Encoding pass** — scan every per-tile causal log for events whose
//!    `tick == resources.current_tick`. For each event with an `AgentId`
//!    actor (or two, for social events), resolve the actor's entity via
//!    a single up-front `AgentId → Entity` map, then insert a new
//!    [`MemoryEntry`] per the locked mapping table (see crate docs of
//!    [`MemorySystem`]).
//! 3. **Reinforcement pass** — for every entry in the recalled set, call
//!    `Memory::reinforce(idx, REINFORCEMENT_BOOST)`.
//!
//! Anti-recursion: `AgentDecision { MemoryReason }` events and
//! `MemoryRecalled` events are NOT encoded (planning §β P8β-NEW-2).
//!
//! Eviction-tolerant: parent walks (Construction events) terminate
//! silently when the parent or grandparent has been evicted from the
//! ring buffer — no panic, no fallback actor (Phase 3-β precedent).
//!
//! [`Memory`]: sim_core::components::Memory
//! [`MemoryEntry`]: sim_core::components::MemoryEntry

use std::collections::HashMap;

use hecs::{Entity, World};
use sim_core::causal::{CausalEvent, CausalLogStorage, DecisionReason, EventId};
use sim_core::components::{Agent, AgentId, Memory, MemoryEntry};
use sim_engine::{RuntimeSystem, SimResources};

/// Per-tick salience decay rate. Locked at `0.001` by plan §2 P8β-NEW-2.
/// 1000 ticks of silent decay drops a saturated entry from 1.0 to 0.0.
pub const DECAY_RATE: f64 = 0.001;

/// Salience boost applied on each cascade-bias recall. 10 recalls
/// saturates an entry from floor to ceiling — mirrors the
/// `FAMILIARITY_BUMP = 0.1` shape from Phase 7-β.
pub const REINFORCEMENT_BOOST: f64 = 0.1;

/// Maximum recency horizon for the cascade-bias `recency_factor` linear
/// ramp. Above this elapsed-tick count a memory contributes zero weight.
pub const MAX_RECENCY_TICKS: u64 = 4380;

/// Phase 8-β memory system. Unit struct — all state lives in the per-agent
/// [`Memory`] component.
///
/// [`Memory`]: sim_core::components::Memory
#[derive(Debug, Default)]
pub struct MemorySystem;

impl MemorySystem {
    /// Construct a fresh instance.
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeSystem for MemorySystem {
    fn name(&self) -> &str {
        "MemorySystem"
    }

    fn priority(&self) -> u32 {
        136
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let current_tick = resources.current_tick;

        // Phase 0: Collect (AgentId, EventId) pairs for MemoryRecalled
        // events emitted in this tick by AgentDecisionSystem (priority 125).
        // These entries are exempted from decay below so the reinforcement
        // formula `(sal_before + REINFORCEMENT_BOOST).min(1.0)` is exact.
        let mut recalled: Vec<(AgentId, EventId)> = Vec::new();
        for (_tile_idx, log) in resources.causal_log.iter() {
            for event in log.iter() {
                if event.tick() != current_tick {
                    continue;
                }
                if let CausalEvent::MemoryRecalled { agent, recalled_event, .. } = event {
                    recalled.push((*agent, *recalled_event));
                }
            }
        }

        // Build AgentId → Entity map once — shared by phases 1, 2, 3.
        let mut id_to_entity: HashMap<AgentId, Entity> = HashMap::new();
        for (entity, agent) in world.query::<&Agent>().iter() {
            id_to_entity.insert(agent.id, entity);
        }

        // Phase 1: Selective decay — skip recalled entries. Per spec
        // §γ: decay runs BEFORE encoding within the same tick, so a newly
        // encoded entry on tick T carries its unmodified mapping-table
        // salience.
        for (_, (agent, memory)) in world.query_mut::<(&Agent, &mut Memory)>() {
            for entry in &mut memory.entries {
                let is_recalled = recalled
                    .iter()
                    .any(|(aid, eid)| *aid == agent.id && *eid == entry.event_id);
                if !is_recalled {
                    entry.salience = (entry.salience - DECAY_RATE).max(0.0);
                }
            }
        }

        // Phase 2: Encoding pass. Collect (entity, MemoryEntry) tuples
        // first so the borrow on causal_log is released before mutating
        // Memory components.
        let mut to_insert: Vec<(Entity, MemoryEntry)> = Vec::new();
        for (_tile_idx, log) in resources.causal_log.iter() {
            for event in log.iter() {
                if event.tick() != current_tick {
                    continue;
                }
                if let Some((salience, valence, agents)) =
                    classify_event(event, &resources.causal_log)
                {
                    for agent_id in agents {
                        if let Some(entity) = id_to_entity.get(&agent_id).copied() {
                            to_insert.push((
                                entity,
                                MemoryEntry::new(event.id(), current_tick, valence, salience),
                            ));
                        }
                    }
                }
            }
        }

        // Apply collected entries. Agents without a `Memory` component
        // are silently skipped. Idempotency guard: skip if event_id
        // already present (plan §β A14 lock).
        for (entity, entry) in to_insert {
            if let Ok(mut mem) = world.get::<&mut Memory>(entity) {
                if mem.find_by_event_id(entry.event_id).is_none() {
                    mem.insert(entry);
                }
            }
        }

        // Phase 3: Reinforce recalled entries.
        for (agent_id, recalled_event_id) in &recalled {
            if let Some(entity) = id_to_entity.get(agent_id).copied() {
                if let Ok(mut mem) = world.get::<&mut Memory>(entity) {
                    if let Some(idx) = mem.find_by_event_id(*recalled_event_id) {
                        mem.reinforce(idx, REINFORCEMENT_BOOST);
                    }
                }
            }
        }
    }
}

/// Classify a [`CausalEvent`] to its `(salience, valence, actor_agent_ids)`
/// tuple per the planning §β mapping table.
///
/// Returns `None` for non-actor events (BuildingPlaced / StampDirty /
/// InfluenceChanged), anti-recursion variants (AgentDecision{MemoryReason}
/// / MemoryRecalled), and Construction events whose parent chain is
/// unreachable (evicted from the ring buffer).
fn classify_event(
    event: &CausalEvent,
    causal_log: &CausalLogStorage,
) -> Option<(f64, f64, Vec<AgentId>)> {
    match event {
        CausalEvent::AgentDecision { agent, reason, .. } => match reason {
            DecisionReason::HungerThresholdBreach => Some((0.4, -0.3, vec![*agent])),
            DecisionReason::ThirstThresholdBreach => Some((0.4, -0.3, vec![*agent])),
            DecisionReason::FatigueThresholdBreach => Some((0.3, -0.2, vec![*agent])),
            DecisionReason::ConstructionReason => Some((0.5, 0.1, vec![*agent])),
            DecisionReason::SocialReason => Some((0.5, 0.2, vec![*agent])),
            DecisionReason::MemoryReason => None, // anti-recursion
            DecisionReason::CombatReason => None, // Phase 9-β anti-recursion
        },
        CausalEvent::ConstructionStarted { parent, .. } => {
            // Parent walk: the originating AgentDecision{ConstructionReason}
            // carries the actor.
            let parent_id = (*parent)?;
            let parent_event = causal_log.lookup(parent_id)?;
            if let CausalEvent::AgentDecision { agent, .. } = parent_event {
                Some((0.6, 0.3, vec![*agent]))
            } else {
                None
            }
        }
        CausalEvent::ConstructionCompleted { parent, .. } => {
            // Strict two-hop walk only: ConstructionCompleted → ConstructionStarted
            // → AgentDecision{ConstructionReason}. One-hop (Completed → Decision
            // directly) returns None — that path violates the plan contract.
            let parent_id = (*parent)?;
            let parent_event = causal_log.lookup(parent_id)?;
            match parent_event {
                CausalEvent::ConstructionStarted { parent: gp, .. } => {
                    let grandparent_id = (*gp)?;
                    let grandparent_event = causal_log.lookup(grandparent_id)?;
                    if let CausalEvent::AgentDecision { agent, .. } = grandparent_event {
                        Some((0.8, 0.6, vec![*agent]))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        CausalEvent::SocialInteractionStarted { agents, .. } => {
            Some((0.6, 0.4, vec![agents.0, agents.1]))
        }
        CausalEvent::SocialInteractionCompleted { agents, .. } => {
            Some((0.8, 0.7, vec![agents.0, agents.1]))
        }
        // Phase 9-β encoding: CombatStarted → attacker only (defender
        // did not initiate); CombatCompleted → both parties (mirrors
        // SocialInteraction pattern, negative valence — hostile memory).
        CausalEvent::CombatStarted { attacker, .. } => {
            Some((0.8, -0.6, vec![*attacker]))
        }
        CausalEvent::CombatCompleted { attacker, defender, .. } => {
            Some((0.9, -0.8, vec![*attacker, *defender]))
        }
        CausalEvent::BuildingPlaced { .. }
        | CausalEvent::StampDirty { .. }
        | CausalEvent::InfluenceChanged { .. }
        | CausalEvent::MemoryRecalled { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_is_136_interval_is_1() {
        let s = MemorySystem::new();
        assert_eq!(s.priority(), 136);
        assert_eq!(s.tick_interval(), 1);
        assert_eq!(s.name(), "MemorySystem");
    }

    #[test]
    fn classify_event_returns_correct_mapping_for_each_actor_variant() {
        let log = CausalLogStorage::new();

        let hunger = CausalEvent::AgentDecision {
            id: 1,
            parent: None,
            agent: 7,
            position: (0, 0),
            reason: DecisionReason::HungerThresholdBreach,
            tick: 0,
        };
        assert_eq!(classify_event(&hunger, &log), Some((0.4, -0.3, vec![7])));

        let thirst = CausalEvent::AgentDecision {
            id: 1,
            parent: None,
            agent: 7,
            position: (0, 0),
            reason: DecisionReason::ThirstThresholdBreach,
            tick: 0,
        };
        assert_eq!(classify_event(&thirst, &log), Some((0.4, -0.3, vec![7])));

        let fatigue = CausalEvent::AgentDecision {
            id: 1,
            parent: None,
            agent: 7,
            position: (0, 0),
            reason: DecisionReason::FatigueThresholdBreach,
            tick: 0,
        };
        assert_eq!(classify_event(&fatigue, &log), Some((0.3, -0.2, vec![7])));

        let con = CausalEvent::AgentDecision {
            id: 1,
            parent: None,
            agent: 7,
            position: (0, 0),
            reason: DecisionReason::ConstructionReason,
            tick: 0,
        };
        assert_eq!(classify_event(&con, &log), Some((0.5, 0.1, vec![7])));

        let social = CausalEvent::AgentDecision {
            id: 1,
            parent: None,
            agent: 7,
            position: (0, 0),
            reason: DecisionReason::SocialReason,
            tick: 0,
        };
        assert_eq!(classify_event(&social, &log), Some((0.5, 0.2, vec![7])));

        // SocialInteractionStarted returns BOTH agents.
        let started = CausalEvent::SocialInteractionStarted {
            id: 1,
            parent: None,
            agents: (3, 5),
            position: (0, 0),
            tick: 0,
        };
        assert_eq!(classify_event(&started, &log), Some((0.6, 0.4, vec![3, 5])));

        let completed = CausalEvent::SocialInteractionCompleted {
            id: 1,
            parent: None,
            agents: (3, 5),
            position: (0, 0),
            familiarity_after: 0.5,
            tick: 0,
        };
        assert_eq!(
            classify_event(&completed, &log),
            Some((0.8, 0.7, vec![3, 5]))
        );
    }

    #[test]
    fn classify_event_returns_none_for_non_actor_and_anti_recursion() {
        use sim_core::influence::{DirtyRegion, InfluenceChannel};
        use sim_core::causal::MemoryRecallTrigger;
        let log = CausalLogStorage::new();

        let bp = CausalEvent::BuildingPlaced {
            id: 1,
            parent: None,
            position: (0, 0),
            radius: 1,
            tick: 0,
        };
        assert!(classify_event(&bp, &log).is_none());

        let sd = CausalEvent::StampDirty {
            id: 1,
            parent: None,
            channel: InfluenceChannel::Warmth,
            region: DirtyRegion::new(0, 0, 1, 1),
            tick: 0,
        };
        assert!(classify_event(&sd, &log).is_none());

        let ic = CausalEvent::InfluenceChanged {
            id: 1,
            parent: None,
            channel: InfluenceChannel::Warmth,
            position: (0, 0),
            old: 0.0,
            new: 0.0,
            tick: 0,
        };
        assert!(classify_event(&ic, &log).is_none());

        let mem_reason = CausalEvent::AgentDecision {
            id: 1,
            parent: None,
            agent: 7,
            position: (0, 0),
            reason: DecisionReason::MemoryReason,
            tick: 0,
        };
        assert!(classify_event(&mem_reason, &log).is_none());

        let mr = CausalEvent::MemoryRecalled {
            id: 1,
            parent: None,
            agent: 7,
            recalled_event: 0,
            triggered_by: MemoryRecallTrigger::CascadeBias,
            tick: 0,
        };
        assert!(classify_event(&mr, &log).is_none());
    }

    #[test]
    fn classify_construction_started_walks_parent() {
        let mut log = CausalLogStorage::new();
        // AgentDecision{Construction} at tile 0
        let decision = CausalEvent::AgentDecision {
            id: 10,
            parent: None,
            agent: 42,
            position: (0, 0),
            reason: DecisionReason::ConstructionReason,
            tick: 0,
        };
        log.push(0, decision);
        let started = CausalEvent::ConstructionStarted {
            id: 11,
            parent: Some(10),
            blueprint: sim_core::components::BuildingBlueprint::new(1, 1, 1, 1),
            position: (0, 0),
            tick: 0,
        };
        assert_eq!(
            classify_event(&started, &log),
            Some((0.6, 0.3, vec![42]))
        );
    }

    #[test]
    fn classify_construction_completed_two_hop_walk() {
        let mut log = CausalLogStorage::new();
        let decision = CausalEvent::AgentDecision {
            id: 10,
            parent: None,
            agent: 42,
            position: (0, 0),
            reason: DecisionReason::ConstructionReason,
            tick: 0,
        };
        let started = CausalEvent::ConstructionStarted {
            id: 11,
            parent: Some(10),
            blueprint: sim_core::components::BuildingBlueprint::new(1, 1, 1, 1),
            position: (0, 0),
            tick: 0,
        };
        log.push(0, decision);
        log.push(0, started);
        let completed = CausalEvent::ConstructionCompleted {
            id: 12,
            parent: Some(11),
            blueprint: sim_core::components::BuildingBlueprint::new(1, 1, 1, 1),
            position: (0, 0),
            tick: 0,
        };
        assert_eq!(
            classify_event(&completed, &log),
            Some((0.8, 0.6, vec![42]))
        );
    }

    #[test]
    fn classify_construction_with_evicted_parent_returns_none() {
        let log = CausalLogStorage::new();
        // Parent id 999 never recorded.
        let started = CausalEvent::ConstructionStarted {
            id: 11,
            parent: Some(999),
            blueprint: sim_core::components::BuildingBlueprint::new(1, 1, 1, 1),
            position: (0, 0),
            tick: 0,
        };
        assert!(classify_event(&started, &log).is_none());
    }
}
