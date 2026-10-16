use hecs::{Entity, World};
use sim_core::components::{
    close_kin, Agent, AgentId, BodyHealth, Child, Elder, FamilyEvent, FamilyObservations, Kin, Sex,
};
use sim_engine::{RuntimeSystem, SimResources};
use std::collections::{BTreeMap, BTreeSet, HashSet};

/// Matches eligible residents after maturation at the family rule's clock minute.
#[derive(Debug, Default)]
pub struct PairingSystem;

impl RuntimeSystem for PairingSystem {
    fn name(&self) -> &str {
        "PairingSystem"
    }
    fn priority(&self) -> u32 {
        144
    }
    fn tick_interval(&self) -> u64 {
        1
    }
    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let Some(rule) = resources.rules.society.family else {
            return;
        };
        let tick = resources.current_tick;
        if tick
            .saturating_add(resources.rules.clock_offset_ticks)
            .checked_rem(resources.ticks_per_day)
            != Some(u64::from(rule.pair_minute))
        {
            return;
        }
        let alive: BTreeSet<_> = world
            .query::<(&Agent, Option<&BodyHealth>)>()
            .iter()
            .filter(|(_, (_, health))| health.is_none_or(|health| !health.is_dead()))
            .map(|(_, (agent, _))| agent.id)
            .collect();
        let mut people: BTreeMap<_, _> = world
            .query::<(&Agent, &Kin, Option<&Sex>, Option<&Child>, Option<&Elder>)>()
            .iter()
            .filter(|(_, (agent, ..))| alive.contains(&agent.id))
            .map(|(entity, (agent, kin, sex, child, elder))| {
                (
                    agent.id,
                    (
                        entity,
                        *kin,
                        sex.copied(),
                        child.is_none() && elder.is_none(),
                    ),
                )
            })
            .collect();
        let mut changes = BTreeMap::new();
        for (&id, (entity, kin, _, _)) in &mut people {
            if kin.partner.is_some_and(|partner| !alive.contains(&partner)) {
                kin.partner = None;
                changes.insert(id, (*entity, *kin));
            }
        }
        let mut relationships: Vec<_> = resources.relationships.iter().collect();
        relationships.sort_by_key(|(key, _)| (key.smaller(), key.larger()));
        let mut known: BTreeMap<AgentId, Vec<(AgentId, f64)>> = BTreeMap::new();
        for (key, relationship) in relationships {
            if relationship.familiarity >= rule.familiarity_threshold() {
                known
                    .entry(key.smaller())
                    .or_default()
                    .push((key.larger(), relationship.familiarity));
                known
                    .entry(key.larger())
                    .or_default()
                    .push((key.smaller(), relationship.familiarity));
            }
        }
        let mut settlements: Vec<_> = resources.settlements.iter().collect();
        settlements.sort_by_key(|(id, _)| **id);
        let mut events = Vec::new();
        // Pass 19 chronicle: (settlement, woman, man, familiarity), pushed
        // after the loop (`resources` stays borrowed through it).
        let mut paired: Vec<(sim_core::components::SettlementId, AgentId, AgentId, f64)> = Vec::new();
        for (sid, settlement) in settlements {
            let females: Vec<_> = people
                .iter()
                .filter(|(id, (_, kin, sex, adult))| {
                    settlement.member_agents.contains(id)
                        && *adult
                        && kin.partner.is_none()
                        && sex.is_some_and(|sex| sex.female)
                })
                .map(|(&id, _)| id)
                .collect();
            for mother in females {
                let Some((_, mother_kin, _, _)) = people.get(&mother) else {
                    continue;
                };
                if mother_kin.partner.is_some() {
                    continue;
                }
                let father = known
                    .get(&mother)
                    .into_iter()
                    .flatten()
                    .filter(|(id, _)| settlement.member_agents.contains(id))
                    .filter(|(id, _)| {
                        people.get(id).is_some_and(|(_, kin, sex, adult)| {
                            *adult
                                && kin.partner.is_none()
                                && sex.is_some_and(|sex| !sex.female)
                                && !close_kin(mother, mother_kin, *id, kin)
                        })
                    })
                    .min_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
                if let Some(&(father, familiarity)) = father {
                    for (id, partner) in [(mother, father), (father, mother)] {
                        if let Some((entity, kin, _, _)) = people.get_mut(&id) {
                            kin.partner = Some(partner);
                            changes.insert(id, (*entity, *kin));
                        }
                    }
                    events.push(FamilyEvent::Paired { mother, father });
                    paired.push((*sid, mother, father, familiarity));
                }
            }
        }
        for (_, (entity, kin)) in changes {
            if let Ok(mut current) = world.get::<&mut Kin>(entity) {
                *current = kin;
            }
        }
        for event in events {
            record_family_event(world, tick, event);
        }
        for (sid, woman, man, familiarity) in paired {
            crate::runtime::chronicle_log::record(
                world,
                resources,
                sim_core::chronicle::ChronicleEntry::new(tick, sim_core::chronicle::ChronicleKind::Paired)
                    .subject(woman)
                    .other(Some(man))
                    .settlement(Some(sid))
                    .value((familiarity * 1000.0).round() as i64),
            );
        }
    }
}

pub(crate) fn record_family_event(world: &mut World, tick: u64, event: FamilyEvent) {
    let entity = world
        .query::<&FamilyObservations>()
        .iter()
        .next()
        .map(|(entity, _)| entity);
    if let Some(entity) = entity {
        if let Ok(mut observations) = world.get::<&mut FamilyObservations>(entity) {
            if observations.tick != tick {
                observations.tick = tick;
                observations.events.clear();
            }
            observations.events.push(event);
        }
    } else {
        world.spawn((FamilyObservations {
            tick,
            events: vec![event],
        },));
    }
}

pub(crate) fn birth_parents(
    world: &World,
    members: &HashSet<AgentId>,
) -> Option<(Entity, AgentId, AgentId)> {
    let people: BTreeMap<_, _> = world
        .query::<(
            &Agent,
            &Kin,
            &Sex,
            Option<&Child>,
            Option<&Elder>,
            Option<&BodyHealth>,
        )>()
        .iter()
        .filter(|(_, (agent, _, _, _, _, health))| {
            members.contains(&agent.id) && health.is_none_or(|health| !health.is_dead())
        })
        .map(|(entity, (agent, kin, sex, child, elder, _))| {
            (
                agent.id,
                (entity, *kin, *sex, child.is_none() && elder.is_none()),
            )
        })
        .collect();
    people
        .iter()
        .filter_map(|(&mother, (entity, kin, sex, adult))| {
            if !sex.female || !adult {
                return None;
            }
            let father = kin.partner?;
            let (_, other, sex, _) = people.get(&father)?;
            (other.partner == Some(mother) && !sex.female).then_some((
                kin.last_birth_tick,
                mother,
                *entity,
                father,
            ))
        })
        .min_by_key(|row| (row.0, row.1))
        .map(|(_, mother, entity, father)| (entity, mother, father))
}
