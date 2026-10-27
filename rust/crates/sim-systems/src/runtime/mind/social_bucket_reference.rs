//! Frozen original complete social snapshot preparation.
use super::*;
pub(super) fn prepare_social_snapshot(
    context: &mut TickContext,
    world: &hecs::World,
    resources: &SimResources,
) -> Result<(), FactError> {
    use sim_core::components::ConstructionSite;
    use std::collections::BTreeSet;
    let sites: BTreeSet<_> = world
        .query::<&ConstructionSite>()
        .iter()
        .filter(|(entity, site)| {
            !site.is_complete()
                && crate::runtime::knowledge::open_to_any_builder(world, resources, *entity)
        })
        .map(|(_, site)| (site.position.x, site.position.y))
        .collect();
    let mut natural: BTreeMap<(u32, u32), Vec<AgentId>> = BTreeMap::new();
    let mut relaxed: BTreeMap<(u32, u32), Vec<AgentId>> = BTreeMap::new();
    let mut idle: BTreeMap<(u32, u32), Vec<AgentId>> = BTreeMap::new();
    let routine = resources.rules.agent.routine.is_some();
    for person in context.rows() {
        let available = person.life_stage != LifeStage::Child
            && crate::runtime::agent_actions::routine::idle_equivalent(person.state, routine)
            && (resources.rules.agent.social_seek.is_none()
                || resources
                    .social_retry_after
                    .get(&person.agent)
                    .is_none_or(|until| *until <= resources.current_tick));
        if !available {
            continue;
        }
        let tile = (person.position.x, person.position.y);
        let threshold = fatigue_threshold(person, resources);
        let lonely = person.social.is_some_and(|v| v > needs::SOCIAL_THRESHOLD);
        let hungry = person
            .hunger
            .is_some_and(|v| v > f64::from(needs::HUNGER_THRESHOLD));
        let thirsty = person.thirst.is_some_and(|v| v > needs::THIRST_THRESHOLD);
        let tired = person.fatigue.is_some_and(|v| v > threshold);
        if lonely && !hungry && !thirsty && !tired && !sites.contains(&tile) {
            natural.entry(tile).or_default().push(person.agent);
        }
        let day = (resources.ticks_per_day != 0).then(|| {
            resources
                .current_tick
                .saturating_add(resources.rules.clock_offset_ticks)
                % resources.ticks_per_day
        });
        let night = needs::person_night_rest(
            resources.rules.agent.night_rest,
            resources.rules.agent.temperament,
            person.temperament.as_ref(),
            day,
        )
        .is_some();
        if !night || !tired {
            idle.entry(tile).or_default().push(person.agent);
            if lonely {
                relaxed.entry(tile).or_default().push(person.agent);
            }
        }
    }
    let rows: Vec<_> = context
        .rows()
        .iter()
        .map(|person| {
            let tile = (person.position.x, person.position.y);
            let peer = |map: &BTreeMap<(u32, u32), Vec<AgentId>>| {
                map.get(&tile)
                    .and_then(|ids| ids.iter().find(|id| **id != person.agent))
                    .copied()
            };
            let natural_peer = peer(&natural);
            let memory_peer = if person.social.is_some_and(|v| v > needs::SOCIAL_THRESHOLD) {
                peer(&relaxed)
            } else {
                None
            }
            .or_else(|| person.memory_causes[4].and_then(|_| peer(&idle)));
            let handshake = match person.state {
                AgentState::Seeking {
                    target: TargetKind::Agent(partner),
                } => context.row(partner).is_ok_and(|other| {
                    other.position == person.position
                        && other.state
                            == AgentState::Seeking {
                                target: TargetKind::Agent(person.agent),
                            }
                }),
                AgentState::Idle | AgentState::Consuming { .. } | AgentState::Seeking { .. } => {
                    false
                }
            };
            (person.agent, tile, natural_peer, memory_peer, handshake)
        })
        .collect();
    context.install_social_snapshot(rows)?;
    Ok(())
}
