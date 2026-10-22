//! M4 knowledge (ADR 0024–0026): the systems that move, fade, find and count
//! what people know, and the helpers the job layer uses for gating.
//!
//! - [`oral::OralTransmissionSystem`] (160, once a day at the evening gathering)
//! - [`observe::ObservationSystem`] (161, every 30 ticks)
//! - [`decay::KnowledgeDecaySystem`] (162, every 60 ticks)
//! - [`discover::DiscoverySystem`] (163, every 240 ticks)
//! - [`aggregate::TechAggregateSystem`] (164, every 240 ticks)
//! - [`seed`]: the founders' knowledge at worldgen (`STREAM_KNOWLEDGE`).
//!
//! Apprenticeship is a pair of jobs in the job layer (`runtime::jobs::teach`).
//! Every system returns at once without `ColonyRule::knowledge`, so the
//! baseline world and the M1–M3 colony rules never run any of it.
use hecs::{Entity, World};
use sim_core::colony::knowledge::{Knowledge, Source};
use sim_core::components::{Agent, AgentId, Child, Life, Lineage, Position, Temperament};
use sim_core::rules::KnowledgeRule;
use sim_engine::SimResources;

pub mod aggregate;
pub mod decay;
pub mod discover;
pub mod observe;
pub mod oral;
pub mod seed;

pub use aggregate::{TechAggregateSystem, TECH_AGGREGATE_PRIORITY};
pub use decay::{KnowledgeDecaySystem, KNOWLEDGE_DECAY_PRIORITY};
pub use discover::{DiscoverySystem, DISCOVERY_PRIORITY};
pub use observe::{ObservationSystem, OBSERVATION_PRIORITY};
pub use oral::{OralTransmissionSystem, ORAL_PRIORITY};

/// The knowledge rule in force (`None` outside the M4 colony).
pub fn rule(res: &SimResources) -> Option<KnowledgeRule> {
    res.rules.colony.and_then(|c| c.knowledge)
}

/// One living person as the knowledge systems see them.
#[derive(Debug, Clone, Copy)]
pub struct Person {
    /// The person.
    pub id: AgentId,
    /// Their entity.
    pub entity: Entity,
    /// The subject of their keyed draws (lineage key, else the id).
    pub key: u64,
    /// Their tile.
    pub pos: (u32, u32),
    /// An adult or elder (no `Child` marker).
    pub adult: bool,
    /// Age in whole years (0 without `Life`).
    pub years: u64,
    /// Novelty seeking ‰ (500 without a temperament).
    pub ns: u16,
}

/// Every living person, in `AgentId` order.
pub fn people(world: &World, res: &SimResources) -> Vec<Person> {
    let tick = res.current_tick;
    let year = res.rules.calendar.years_to_ticks(1).max(1);
    let mut out: Vec<Person> = world
        .query::<(
            &Agent,
            &Position,
            Option<&Child>,
            Option<&Lineage>,
            Option<&Life>,
            Option<&Temperament>,
        )>()
        .iter()
        .map(
            |(entity, (agent, pos, child, lineage, life, temperament))| Person {
                id: agent.id,
                entity,
                key: lineage.map_or(agent.id, |l| l.key),
                pos: (pos.x, pos.y),
                adult: child.is_none(),
                years: life.map_or(0, |l| u64::try_from(l.age_ticks(tick)).unwrap_or(0) / year),
                ns: temperament.map_or(500, |t| t.ns),
            },
        )
        .collect();
    out.sort_by_key(|p| p.id);
    out
}

/// A copy of `entity`'s knowledge (empty without the component).
pub fn knowledge_of(world: &World, entity: Entity) -> Knowledge {
    world
        .get::<&Knowledge>(entity)
        .map(|k| (*k).clone())
        .unwrap_or_default()
}

/// Write `knowledge` back to `entity`, adding the component when missing (a
/// structural insert: callers apply writes in `AgentId` order after their query).
pub fn store(world: &mut World, entity: Entity, knowledge: Knowledge) {
    if let Ok(mut held) = world.get::<&mut Knowledge>(entity) {
        *held = knowledge;
        return;
    }
    let _ = world.insert_one(entity, knowledge);
}

/// The practitioner bitset of `entity` (0 without the component).
pub fn practice_of(world: &World, entity: Entity) -> u32 {
    world.get::<&Knowledge>(entity).map_or(0, |k| k.practice())
}

/// A completed gated job (goal 3.2): every node of `mask` the worker holds
/// gains the rule's `use_gain` ‰ and is stamped used. Nothing without the rule.
pub fn used(world: &mut World, res: &SimResources, entity: Entity, mask: u32) {
    let Some(rule) = rule(res) else {
        return;
    };
    if mask == 0 {
        return;
    }
    if let Ok(mut k) = world.get::<&mut Knowledge>(entity) {
        k.use_mask(mask, rule.use_gain, res.current_tick);
    }
}

/// The lowest proficiency among the nodes of `mask` the worker holds (the
/// craft's quality cap reads it; 1000 for an empty mask).
pub fn proficiency_of(world: &World, entity: Entity, mask: u32) -> u16 {
    let k = world.get::<&Knowledge>(entity).ok();
    sim_core::colony::knowledge::ids_of(mask)
        .map(|id| k.as_ref().map_or(0, |k| k.proficiency(id)))
        .min()
        .unwrap_or(1000)
}

/// Whether some living adult practises every node of `mask` (a gated order can run).
pub fn anyone_practises(world: &World, mask: u32) -> bool {
    if mask == 0 {
        return true;
    }
    world
        .query::<&Knowledge>()
        .without::<&Child>()
        .iter()
        .any(|(_, k)| k.practises_all(mask))
}

/// The source code a chronicle `KnowledgeLearned` carries for a channel.
pub const fn channel_code(source: Source) -> u8 {
    source.code()
}
