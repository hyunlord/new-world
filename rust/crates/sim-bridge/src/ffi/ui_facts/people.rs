//! Stable person facts for bulk UI evaluation; no alert policy or simulation writes.

use sim_core::colony::people::{Drafted, Mood, Sick};
use sim_core::colony::Job;
use sim_core::components::{Agent, AgentState, BodyHealth, Child, Kin, Position, TargetKind};
use sim_engine::SimResources;
use std::collections::{BTreeMap, BTreeSet};

/// A living person's facts, ordered by AgentId by the collector.
#[derive(Clone, Debug, PartialEq)]
pub struct PersonFact {
    /// Stable simulation identity, distinct from the entity bits.
    pub id: u64,
    /// Minimum living member of the authoritative settlement household, or own id outside one.
    pub household_id: u64,
    /// Entity identity used by the existing snapshot selection surface.
    pub entity_bits: u64,
    /// Current tile.
    pub tile: (u32, u32),
    /// Current health; absent when the component is absent.
    pub hp: Option<f64>,
    /// Maximum health, with the same presence as hp.
    pub max_hp: Option<f64>,
    /// Whether this person has the Child life-stage marker.
    pub child: bool,
    /// Whether the configured infant age window applies.
    pub infant: bool,
    /// Birth tick from the Child component, absent for adults.
    pub born_tick: Option<u64>,
    /// Whether the FSM is idle (separate from job or incapacity facts).
    pub idle: bool,
    /// Whether the FSM is currently consuming sleep.
    pub sleeping: bool,
    /// Whether the current tile is inside a roofed room.
    pub roofed: bool,
    /// Assigned bed tile, without inferring ownership from proximity.
    pub bed: Option<(u32, u32)>,
    /// Active job component presence.
    pub has_job: bool,
    /// Draft component presence.
    pub drafted: bool,
    /// Illness component presence, matching existing M5 reads.
    pub sick: bool,
    /// Whether the recorded breakdown includes the current display day.
    pub breakdown: bool,
    /// Last night's and current night's raw mood sleep bits.
    pub nights: u8,
    /// Recorded mother identity.
    pub mother: Option<u64>,
    /// Recorded father identity.
    pub father: Option<u64>,
    /// Recorded partner identity.
    pub partner: Option<u64>,
}

pub(super) fn collect(world: &hecs::World, res: &SimResources) -> Vec<PersonFact> {
    let mut household_of = BTreeMap::new();
    let mut settlements: Vec<_> = res.settlements.iter().collect();
    settlements.sort_by_key(|(id, _)| **id);
    for (_, settlement) in settlements {
        let members: BTreeSet<_> = settlement.member_agents.iter().copied().collect();
        for group in sim_systems::runtime::agent::households::households(world, &members) {
            if let Some(&representative) = group.first() {
                for id in group {
                    household_of.entry(id).or_insert(representative);
                }
            }
        }
    }
    let infant_ticks = res.rules.society.life_stage.and_then(|r| r.infant_ticks);
    let mut rows: Vec<_> = world
        .query::<(
            &Agent,
            &Position,
            Option<&BodyHealth>,
            Option<&Child>,
            Option<&AgentState>,
            Option<&Kin>,
            Option<&Mood>,
            Option<&Sick>,
            Option<&Drafted>,
            Option<&Job>,
        )>()
        .iter()
        .filter_map(
            |(entity, (agent, pos, health, child, state, kin, mood, sick, draft, job))| {
                if health.is_some_and(BodyHealth::is_dead) {
                    return None;
                }
                Some(PersonFact {
                    id: agent.id,
                    household_id: household_of.get(&agent.id).copied().unwrap_or(agent.id),
                    entity_bits: entity.to_bits().get(),
                    tile: (pos.x, pos.y),
                    hp: health.map(|h| h.hp),
                    max_hp: health.map(|h| h.max_hp),
                    child: child.is_some(),
                    infant: child.is_some_and(|c| c.is_infant(res.current_tick, infant_ticks)),
                    born_tick: child.map(|c| c.born_tick),
                    idle: state.is_some_and(|s| *s == AgentState::Idle),
                    sleeping: state.is_some_and(|s| {
                        matches!(
                            s,
                            AgentState::Consuming {
                                target: TargetKind::Sleep
                            }
                        )
                    }),
                    roofed: sim_systems::runtime::rooms::roofed(res, (pos.x, pos.y)),
                    bed: res.bed_of.get(&agent.id).copied(),
                    has_job: job.is_some(),
                    drafted: draft.is_some(),
                    sick: sick.is_some(),
                    breakdown: mood
                        .is_some_and(|m| m.broken(sim_systems::runtime::mood::today(res))),
                    nights: mood.map_or(0, |m| m.nights),
                    mother: kin.and_then(|k| k.mother),
                    father: kin.and_then(|k| k.father),
                    partner: kin.and_then(|k| k.partner),
                })
            },
        )
        .collect();
    rows.sort_by_key(|row| row.id);
    rows
}
