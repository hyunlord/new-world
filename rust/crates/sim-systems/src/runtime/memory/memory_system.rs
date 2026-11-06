//! MemorySystem: decays, encodes and reinforces agent memories from this tick's causal events.
//!
//! Priority: 136 · Interval: 1 tick · Gate: none
//! Reads: `Agent`; `SimResources.causal_log` (this tick's events, `MemoryRecalled`)
//! Writes: `Memory`
//! Events: none
//!
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
//! 2. **Encoding pass** — apply entries collected from causal events whose
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
use sim_core::colony::mind_records::{MindChoiceDisplay, MindRecordDictionary};
use sim_core::components::{Agent, AgentId, Memory, MemoryArm, MemoryEntry};
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
        let dictionary = resources
            .mind
            .history
            .as_ref()
            .map(|history| history.dictionary());

        // Resolve actors once; collect both event roles in tile/FIFO order before
        // mutating Memory. Decoding a current MindChoice serves both roles.
        let mut id_to_entity: HashMap<AgentId, Entity> =
            HashMap::with_capacity(world.query::<&Agent>().iter().len());
        for (entity, agent) in world.query::<&Agent>().iter() {
            id_to_entity.insert(agent.id, entity);
        }
        let mut recalled: Vec<(AgentId, EventId)> = Vec::new();
        let mut to_insert: Vec<(Entity, MemoryEntry)> = Vec::new();
        for (_tile_idx, log) in resources.causal_log.iter() {
            for event in log.iter() {
                if event.tick() != current_tick {
                    continue;
                }
                let classified = match event {
                    CausalEvent::MemoryRecalled {
                        agent,
                        recalled_event,
                        ..
                    } => {
                        recalled.push((*agent, *recalled_event));
                        None
                    }
                    CausalEvent::MindChoice { choice } => dictionary
                        .and_then(|dictionary| choice.decode_display(dictionary).ok())
                        .and_then(|decoded| {
                            if let Some(id) = recalled_mind_event(&decoded) {
                                recalled.push((decoded.agent, id));
                            }
                            classify_mind_choice(&decoded)
                        }),
                    _ => classify_event(event, &resources.causal_log, dictionary),
                };
                if let Some((arm, salience, valence, agents)) = classified {
                    for agent_id in agents {
                        if let Some(entity) = id_to_entity.get(&agent_id).copied() {
                            to_insert.push((
                                entity,
                                MemoryEntry::new(event.id(), current_tick, valence, salience, arm),
                            ));
                        }
                    }
                }
            }
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

/// Classify a [`CausalEvent`] to its `(arm, salience, valence,
/// actor_agent_ids)` tuple per the planning §β mapping table.
///
/// The leading [`MemoryArm`] is the cascade-arm classification stored on the
/// resulting [`MemoryEntry`] at encode time (Fix D). It is the SOLE source of
/// an entry's arm thereafter — the cascade-bias sum no longer re-derives it
/// from a `causal_log` lookup, so memory bias is deterministic regardless of
/// `event_id` values or ring-buffer eviction. The arm assigned here mirrors
/// exactly the `(arm, event)` pairs the former `event_id_matches_arm`
/// recognised, so cascade behaviour is preserved for any event still resident
/// in the ring.
///
/// Returns `None` for non-actor events (BuildingPlaced / StampDirty /
/// InfluenceChanged), anti-recursion variants (AgentDecision{MemoryReason}
/// / MemoryRecalled), and Construction events whose parent chain is
/// unreachable (evicted from the ring buffer).
fn classify_event(
    event: &CausalEvent,
    causal_log: &CausalLogStorage,
    dictionary: Option<&MindRecordDictionary>,
) -> Option<(MemoryArm, f64, f64, Vec<AgentId>)> {
    match event {
        CausalEvent::MindChoice { choice } => {
            let decoded = choice.decode_display(dictionary?).ok()?;
            classify_mind_choice(&decoded)
        }
        CausalEvent::AgentDecision { agent, reason, .. } => match reason {
            DecisionReason::HungerThresholdBreach => Some((MemoryArm::Hunger, 0.4, -0.3, vec![*agent])),
            DecisionReason::ThirstThresholdBreach => Some((MemoryArm::Thirst, 0.4, -0.3, vec![*agent])),
            DecisionReason::FatigueThresholdBreach => Some((MemoryArm::Fatigue, 0.3, -0.2, vec![*agent])),
            DecisionReason::ConstructionReason => Some((MemoryArm::Construction, 0.5, 0.1, vec![*agent])),
            DecisionReason::SocialReason => Some((MemoryArm::Social, 0.5, 0.2, vec![*agent])),
            DecisionReason::MemoryReason => None, // anti-recursion
            DecisionReason::CombatReason => None, // Phase 9-β anti-recursion
            DecisionReason::SettlementReason => None, // Phase 10-β anti-recursion
        },
        CausalEvent::ConstructionStarted { parent, .. } => {
            // Parent walk: the originating AgentDecision{ConstructionReason}
            // carries the actor.
            let parent_id = (*parent)?;
            let parent_event = causal_log.lookup(parent_id)?;
            let agent = decision_actor(parent_event, dictionary)?;
            Some((MemoryArm::Construction, 0.6, 0.3, vec![agent]))
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
                    let agent = decision_actor(grandparent_event, dictionary)?;
                    Some((MemoryArm::Construction, 0.8, 0.6, vec![agent]))
                }
                _ => None,
            }
        }
        CausalEvent::SocialInteractionStarted { agents, .. } => {
            Some((MemoryArm::Social, 0.6, 0.4, vec![agents.0, agents.1]))
        }
        CausalEvent::SocialInteractionCompleted { agents, .. } => {
            Some((MemoryArm::Social, 0.8, 0.7, vec![agents.0, agents.1]))
        }
        // Phase 9-β encoding: CombatStarted → attacker only (defender
        // did not initiate); CombatCompleted → both parties (mirrors
        // SocialInteraction pattern, negative valence — hostile memory).
        CausalEvent::CombatStarted { attacker, .. } => {
            Some((MemoryArm::Combat, 0.8, -0.6, vec![*attacker]))
        }
        CausalEvent::CombatCompleted { attacker, defender, .. } => {
            Some((MemoryArm::Combat, 0.9, -0.8, vec![*attacker, *defender]))
        }
        CausalEvent::BuildingPlaced { .. }
        | CausalEvent::StampDirty { .. }
        | CausalEvent::InfluenceChanged { .. }
        | CausalEvent::MemoryRecalled { .. }
        | CausalEvent::AgentBorn { .. }
        | CausalEvent::SettlementFormed { .. }
        | CausalEvent::SettlementDissolved { .. }
        // add-starvation-death: AgentDied is a causal leaf — the dead agent has
        // no Memory to encode into, so it is a non-actor event here.
        | CausalEvent::AgentDied { .. }
        | CausalEvent::PressureCause { .. }
        | CausalEvent::MindFactOrigin { .. } => None,
    }
}

fn classify_mind_choice(
    decoded: &MindChoiceDisplay<'_>,
) -> Option<(MemoryArm, f64, f64, Vec<AgentId>)> {
    if decoded.selected.labels.finish.is_some()
        || !decoded
            .selected
            .labels
            .tags
            .iter()
            .any(|tag| tag == "natural")
        || decoded
            .selected
            .labels
            .tags
            .iter()
            .any(|tag| tag == "memory")
    {
        return None;
    }
    let (arm, salience, valence) = mind_memory_mapping(decoded.selected.labels.behaviour)?;
    Some((arm, salience, valence, vec![decoded.agent]))
}

fn decision_actor(
    event: &CausalEvent,
    dictionary: Option<&MindRecordDictionary>,
) -> Option<AgentId> {
    match event {
        CausalEvent::AgentDecision { agent, .. } => Some(*agent),
        CausalEvent::MindChoice { choice } => Some(choice.decode_display(dictionary?).ok()?.agent),
        _ => None,
    }
}

fn mind_memory_mapping(behaviour: &str) -> Option<(MemoryArm, f64, f64)> {
    match behaviour {
        "eat" => Some((MemoryArm::Hunger, 0.4, -0.3)),
        "drink" => Some((MemoryArm::Thirst, 0.4, -0.3)),
        "sleep" => Some((MemoryArm::Fatigue, 0.3, -0.2)),
        "socialise" => Some((MemoryArm::Social, 0.5, 0.2)),
        _ => None,
    }
}

fn recalled_mind_event(choice: &MindChoiceDisplay<'_>) -> Option<EventId> {
    if !choice
        .selected
        .labels
        .tags
        .iter()
        .any(|tag| tag == "memory")
    {
        return None;
    }
    let fact = match choice.selected.labels.behaviour {
        "eat" => "memory_hunger",
        "drink" => "memory_thirst",
        "sleep" => "memory_fatigue",
        "socialise" => "memory_social",
        _ => return None,
    };
    choice
        .facts
        .iter()
        .find(|witness| witness.name == fact)?
        .cause
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collection_preserves_duplicate_recall_and_new_entry_reinforcement() {
        let mut engine =
            sim_engine::SimEngine::new(4, 4, sim_core::material::MaterialRegistry::new());
        let entity = engine.spawn_agent(0, 0);
        let agent = engine.world.get::<&Agent>(entity).expect("agent").id;
        let mut memory = Memory::new();
        memory.insert(MemoryEntry::new(1, 0, -0.3, 0.4, MemoryArm::Hunger));
        memory.insert(MemoryEntry::new(2, 0, -0.3, 0.4, MemoryArm::Hunger));
        engine.world.insert_one(entity, memory).expect("memory");
        engine.resources.current_tick = 5;
        for (id, recalled_event) in [(10, 1), (11, 1), (12, 3)] {
            engine.resources.causal_log.push(
                0,
                CausalEvent::MemoryRecalled {
                    id,
                    parent: None,
                    agent,
                    recalled_event,
                    triggered_by: sim_core::causal::MemoryRecallTrigger::CascadeBias,
                    tick: 5,
                },
            );
        }
        engine.resources.causal_log.push(
            1,
            CausalEvent::AgentDecision {
                id: 3,
                parent: None,
                agent,
                position: (1, 0),
                reason: DecisionReason::HungerThresholdBreach,
                tick: 5,
            },
        );
        MemorySystem.tick(&mut engine.world, &mut engine.resources);
        let actual = engine.world.get::<&Memory>(entity).expect("memory");
        let mut expected = Memory::new();
        expected.insert(MemoryEntry::new(
            1,
            0,
            -0.3,
            (0.4 + 0.1) + 0.1,
            MemoryArm::Hunger,
        ));
        expected.insert(MemoryEntry::new(2, 0, -0.3, 0.4 - 0.001, MemoryArm::Hunger));
        expected.insert(MemoryEntry::new(3, 5, -0.3, 0.4 + 0.1, MemoryArm::Hunger));
        expected.entries[0].reinforcement_count = 2;
        expected.entries[2].reinforcement_count = 1;
        assert_eq!(*actual, expected);
    }

    #[test]
    fn collection_preserves_tile_fifo_eviction_and_nonsuffix_current_events() {
        let mut engine =
            sim_engine::SimEngine::new(4, 4, sim_core::material::MaterialRegistry::new());
        let entity = engine.spawn_agent(0, 0);
        let agent = engine.world.get::<&Agent>(entity).expect("agent").id;
        let mut memory = Memory::new();
        for id in 0..32 {
            memory.insert(MemoryEntry::new(id, 0, 0.0, 0.9, MemoryArm::Social));
        }
        engine.world.insert_one(entity, memory).expect("memory");
        engine.resources.current_tick = 5;
        // Push the later tile first; neither global ID order nor tick suffix is valid.
        for (tile, id, tick) in [(2, 90, 5), (1, 100, 5), (1, 101, 4), (1, 99, 5)] {
            engine.resources.causal_log.push(
                tile,
                CausalEvent::AgentDecision {
                    id,
                    parent: None,
                    agent,
                    position: (tile, 0),
                    reason: DecisionReason::HungerThresholdBreach,
                    tick,
                },
            );
        }
        MemorySystem.tick(&mut engine.world, &mut engine.resources);
        let actual = engine.world.get::<&Memory>(entity).expect("memory");
        assert_eq!(actual.entries.len(), 32);
        // Each fresh low-salience entry evicts its predecessor before the next insertion.
        assert_eq!(
            actual.entries[0],
            MemoryEntry::new(90, 5, -0.3, 0.4, MemoryArm::Hunger)
        );
        for (index, entry) in actual.entries.iter().enumerate().skip(1) {
            assert_eq!(
                *entry,
                MemoryEntry::new(index as u64, 0, 0.0, 0.9 - 0.001, MemoryArm::Social)
            );
        }
    }

    fn mind_fixture(tag: &str) -> (CausalEvent, MindRecordDictionary) {
        use sim_core::colony::mind::{
            compile, BehaviourSpec, FactKind, FactSpec, MindRegistrySchema,
        };
        use sim_core::colony::mind_records::{
            ChoiceCandidate, ChoiceFact, MindChoiceRecord, PackedMindChoice,
        };
        let schema = MindRegistrySchema {
            facts: vec![FactSpec {
                name: "memory_hunger".into(),
                kind: FactKind::Signed,
            }],
            behaviours: vec![BehaviourSpec {
                name: "eat".into(),
                target_bindings: vec![],
            }],
        };
        let source =
            r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
            (id:"idle",band:"idle",behaviour:"eat",when:[],
             score:(constant:0,terms:[(fact:"memory_hunger",weight_permille:1)]),
             hysteresis:0,life_stages:["adult","child","elder"],tags:["TAG"],name_key:"IDLE")])"#
                .replace("TAG", tag);
        let policy = compile(&source, &schema).expect("policy");
        let node = policy.node_id("idle").expect("node");
        let record = MindChoiceRecord {
            revision: policy.revision(),
            id: 10,
            parent: Some(3),
            agent: 7,
            tick: 5,
            node,
            band: policy.node(node).expect("node").band,
            score: 2,
            top3: vec![ChoiceCandidate { node, score: 2 }],
            facts: vec![ChoiceFact {
                fact: policy.fact_id("memory_hunger").expect("fact"),
                value: 2,
                cause: Some(3),
            }],
        };
        (
            CausalEvent::MindChoice {
                choice: PackedMindChoice::new(&policy, &record).expect("choice"),
            },
            MindRecordDictionary::from_policy(&policy).expect("dictionary"),
        )
    }

    #[test]
    fn mind_natural_choices_encode_and_memory_choices_only_recall_real_origin() {
        let log = CausalLogStorage::new();
        let (natural, dictionary) = mind_fixture("natural");
        assert_eq!(
            classify_event(&natural, &log, Some(&dictionary)),
            Some((MemoryArm::Hunger, 0.4, -0.3, vec![7]))
        );
        assert!(classify_event(&natural, &log, None).is_none());
        let (memory, dictionary) = mind_fixture("memory");
        assert!(classify_event(&memory, &log, Some(&dictionary)).is_none());
        if let CausalEvent::MindChoice { choice } = memory {
            let decoded = choice.decode_display(&dictionary).expect("decode");
            assert_eq!(recalled_mind_event(&decoded), Some(3));
        } else {
            panic!("choice fixture");
        }
        let (continuation, dictionary) = mind_fixture("continuation");
        assert!(classify_event(&continuation, &log, Some(&dictionary)).is_none());
        let bridge = CausalEvent::MindFactOrigin {
            id: 11,
            tick: 5,
            source: sim_core::colony::mind_records::MindFactSource::Command(0),
        };
        assert!(classify_event(&bridge, &log, Some(&dictionary)).is_none());
    }

    #[test]
    fn construction_attributes_actual_mind_parent_without_reclassifying_its_behaviour() {
        let (choice, dictionary) = mind_fixture("natural");
        let mut log = CausalLogStorage::new();
        log.push(0, choice);
        let started = CausalEvent::ConstructionStarted {
            id: 11,
            parent: Some(10),
            blueprint: sim_core::components::BuildingBlueprint::new(1, 1, 1, 1),
            position: (0, 0),
            tick: 5,
        };
        assert_eq!(
            classify_event(&started, &log, Some(&dictionary)),
            Some((MemoryArm::Construction, 0.6, 0.3, vec![7]))
        );
    }

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
        assert_eq!(
            classify_event(&hunger, &log, None),
            Some((MemoryArm::Hunger, 0.4, -0.3, vec![7]))
        );

        let thirst = CausalEvent::AgentDecision {
            id: 1,
            parent: None,
            agent: 7,
            position: (0, 0),
            reason: DecisionReason::ThirstThresholdBreach,
            tick: 0,
        };
        assert_eq!(
            classify_event(&thirst, &log, None),
            Some((MemoryArm::Thirst, 0.4, -0.3, vec![7]))
        );

        let fatigue = CausalEvent::AgentDecision {
            id: 1,
            parent: None,
            agent: 7,
            position: (0, 0),
            reason: DecisionReason::FatigueThresholdBreach,
            tick: 0,
        };
        assert_eq!(
            classify_event(&fatigue, &log, None),
            Some((MemoryArm::Fatigue, 0.3, -0.2, vec![7]))
        );

        let con = CausalEvent::AgentDecision {
            id: 1,
            parent: None,
            agent: 7,
            position: (0, 0),
            reason: DecisionReason::ConstructionReason,
            tick: 0,
        };
        assert_eq!(
            classify_event(&con, &log, None),
            Some((MemoryArm::Construction, 0.5, 0.1, vec![7]))
        );

        let social = CausalEvent::AgentDecision {
            id: 1,
            parent: None,
            agent: 7,
            position: (0, 0),
            reason: DecisionReason::SocialReason,
            tick: 0,
        };
        assert_eq!(
            classify_event(&social, &log, None),
            Some((MemoryArm::Social, 0.5, 0.2, vec![7]))
        );

        // SocialInteractionStarted returns BOTH agents.
        let started = CausalEvent::SocialInteractionStarted {
            id: 1,
            parent: None,
            agents: (3, 5),
            position: (0, 0),
            tick: 0,
        };
        assert_eq!(
            classify_event(&started, &log, None),
            Some((MemoryArm::Social, 0.6, 0.4, vec![3, 5]))
        );

        let completed = CausalEvent::SocialInteractionCompleted {
            id: 1,
            parent: None,
            agents: (3, 5),
            position: (0, 0),
            familiarity_after: 0.5,
            tick: 0,
        };
        assert_eq!(
            classify_event(&completed, &log, None),
            Some((MemoryArm::Social, 0.8, 0.7, vec![3, 5]))
        );
    }

    #[test]
    fn classify_event_returns_none_for_non_actor_and_anti_recursion() {
        use sim_core::causal::MemoryRecallTrigger;
        use sim_core::influence::{DirtyRegion, InfluenceChannel};
        let log = CausalLogStorage::new();

        let bp = CausalEvent::BuildingPlaced {
            id: 1,
            parent: None,
            position: (0, 0),
            radius: 1,
            tick: 0,
        };
        assert!(classify_event(&bp, &log, None).is_none());

        let sd = CausalEvent::StampDirty {
            id: 1,
            parent: None,
            channel: InfluenceChannel::Warmth,
            region: DirtyRegion::new(0, 0, 1, 1),
            tick: 0,
        };
        assert!(classify_event(&sd, &log, None).is_none());

        let ic = CausalEvent::InfluenceChanged {
            id: 1,
            parent: None,
            channel: InfluenceChannel::Warmth,
            position: (0, 0),
            old: 0.0,
            new: 0.0,
            tick: 0,
        };
        assert!(classify_event(&ic, &log, None).is_none());

        let mem_reason = CausalEvent::AgentDecision {
            id: 1,
            parent: None,
            agent: 7,
            position: (0, 0),
            reason: DecisionReason::MemoryReason,
            tick: 0,
        };
        assert!(classify_event(&mem_reason, &log, None).is_none());

        let mr = CausalEvent::MemoryRecalled {
            id: 1,
            parent: None,
            agent: 7,
            recalled_event: 0,
            triggered_by: MemoryRecallTrigger::CascadeBias,
            tick: 0,
        };
        assert!(classify_event(&mr, &log, None).is_none());
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
            classify_event(&started, &log, None),
            Some((MemoryArm::Construction, 0.6, 0.3, vec![42]))
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
            classify_event(&completed, &log, None),
            Some((MemoryArm::Construction, 0.8, 0.6, vec![42]))
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
        assert!(classify_event(&started, &log, None).is_none());
    }
}
