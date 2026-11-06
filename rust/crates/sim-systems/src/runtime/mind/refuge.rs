//! Physical refuge facts: real heat/fuel, reachable room tiles and existing seats.
use super::context::{PersonSnapshot, TargetCandidate, TickContext};
use crate::runtime::{settlement::firewood, thermal};
use sim_core::colony::{Carry, JobKind};
use sim_core::components::{ActivityKind, ResourceKind};
use sim_engine::SimResources;

pub(super) fn active(person: &PersonSnapshot) -> bool {
    person
        .activity
        .is_some_and(|a| a.kind == ActivityKind::SitByFire)
}

pub(super) fn here(person: &PersonSnapshot, res: &SimResources) -> bool {
    let Some(thermal) = res.thermal.as_ref() else {
        return false;
    };
    let tile = (person.position.x, person.position.y);
    res.colony
        .room_of_tile
        .get(&tile)
        .and_then(|room| res.colony.rooms.get(room))
        .is_some_and(|room| room.roofed)
        && thermal::felt_temperature(res, person.apparel.as_ref(), tile)
            .is_some_and(|felt| felt >= thermal.rules.cold_threshold_d10)
}

pub(super) fn needs_ready(person: &PersonSnapshot) -> bool {
    let ceiling = crate::runtime::survival::SAFE_NEED_CEILING;
    person.hunger.is_none_or(|hunger| hunger < ceiling)
        && person.thirst.is_none_or(|thirst| thirst < ceiling)
}

pub(super) fn fuel_job(person: &PersonSnapshot, res: &SimResources) -> bool {
    person
        .job
        .is_some_and(|job| fuel_service_job(person.agent, job.kind, person.carry, res))
}

/// The assignment boundary uses the same exact physical fuel exception as mind.
pub(crate) fn fuel_service_job(
    agent: sim_core::components::AgentId,
    kind: JobKind,
    carry: Option<Carry>,
    res: &SimResources,
) -> bool {
    let Some(settlement) = res
        .settlements
        .values()
        .find(|s| s.member_agents.contains(&agent))
    else {
        return false;
    };
    let rate = firewood::current_service_rate(res);
    if rate == 0
        || settlement
            .stockpile
            .get(&ResourceKind::Wood)
            .copied()
            .unwrap_or(0)
            >= rate
    {
        return false;
    }

    match kind {
        JobKind::Gather { .. } => true,
        JobKind::Chop { tree } => res
            .colony
            .workshops
            .values()
            .filter(|shop| res.nav.reachable(tree, shop.work_tile()))
            .any(|shop| {
                shop.bills
                    .iter()
                    .filter(|bill| !bill.suspended)
                    .any(|bill| {
                        bill.recipe.recipe().is_some_and(|recipe| {
                            recipe.output.0.definition().fuel_units.is_some()
                                && recipe.inputs.iter().any(|(kind, _)| {
                                    *kind == sim_core::colony::ItemKind::role("tree_output")
                                })
                        })
                    })
            }),
        JobKind::Craft { workshop, bill, .. } => res
            .colony
            .workshops
            .get(&workshop)
            .and_then(|shop| shop.bill(bill))
            .and_then(|bill| bill.recipe.recipe())
            .is_some_and(|recipe| recipe.output.0.definition().fuel_units.is_some()),
        JobKind::Supply { workshop, from, .. } => {
            res.colony.workshops.get(&workshop).is_some_and(|shop| {
                shop.bills
                    .iter()
                    .filter(|bill| !bill.suspended && shop.active == Some(bill.id))
                    .any(|bill| {
                        bill.recipe.recipe().is_some_and(|recipe| {
                            recipe.output.0.definition().fuel_units.is_some()
                                && carry
                                    .map(|carry| carry.kind)
                                    .or_else(|| {
                                        res.colony.ground_items.get(&from).map(|item| item.kind)
                                    })
                                    .is_some_and(|input| {
                                        recipe.inputs.iter().any(|(kind, _)| *kind == input)
                                    })
                        })
                    })
            })
        }
        JobKind::Haul { from, .. } => {
            carry.is_some_and(|carry| carry.kind.definition().fuel_units.is_some())
                || res
                    .colony
                    .ground_items
                    .get(&from)
                    .is_some_and(|item| item.kind.definition().fuel_units.is_some())
        }
        _ => false,
    }
}

pub(super) fn target(
    person: &PersonSnapshot,
    context: &TickContext,
    res: &SimResources,
) -> Option<TargetCandidate> {
    let threshold = res.thermal.as_ref()?.rules.cold_threshold_d10;
    let origin = (person.position.x, person.position.y);
    let scope = context.target_scope(person.agent).ok()?;
    let service = firewood::current_service_rate(res);
    let heatable = scope.settlement.and_then(|sid| {
        let settlement = res.settlements.get(&sid)?;
        (service > 0
            && settlement
                .stockpile
                .get(&ResourceKind::Wood)
                .copied()
                .unwrap_or(0)
                >= service)
            .then(|| firewood::primary_hearth_room(res, sid))
            .flatten()
            .map(|room| room.id)
    });
    res.colony
        .rooms
        .values()
        .filter(|room| room.roofed)
        .flat_map(|room| room.tiles.iter().copied().map(move |tile| (room.id, tile)))
        .filter(|(room, tile)| {
            Some(*room) == heatable
                || thermal::felt_temperature(res, person.apparel.as_ref(), *tile)
                    .is_some_and(|t| t >= threshold)
        })
        .filter(|(_, tile)| res.nav.reachable(origin, *tile))
        .filter(|(_, tile)| {
            context
                .rows()
                .iter()
                .filter(|other| {
                    other.agent != person.agent && other.activity.is_some_and(|a| a.place == *tile)
                })
                .count()
                < 2
        })
        .min_by_key(|(_, tile)| {
            (
                person.activity.is_none_or(|a| a.place != *tile),
                origin.0.abs_diff(tile.0) + origin.1.abs_diff(tile.1),
                *tile,
            )
        })
        .map(|(_, tile)| TargetCandidate {
            tile,
            agent: None,
            food: None,
        })
}

/// A real injured recovery owner must survive assignment before routine cleanup.
pub(crate) fn protects_assignment(
    world: &hecs::World,
    res: &SimResources,
    entity: hecs::Entity,
    agent: sim_core::components::AgentId,
) -> bool {
    use sim_core::colony::people::{Apparel, Drafted, Fleeing};
    use sim_core::components::{
        Activity, AgentState, BodyHealth, Hunger, Position, TargetKind, Thirst,
    };
    let Some(thermal) = res.thermal.as_ref() else {
        return false;
    };
    let Some(choice) = res
        .mind
        .running
        .get(&agent)
        .filter(|choice| !choice.finished)
    else {
        return false;
    };
    if res
        .mind
        .history
        .as_ref()
        .and_then(|history| history.dictionary().node_label(choice.node.index()))
        .is_none_or(|node| node.behaviour != "warm_refuge")
        || world.get::<&Drafted>(entity).is_ok()
        || world.get::<&Fleeing>(entity).is_ok()
        || !world
            .get::<&BodyHealth>(entity)
            .is_ok_and(|health| health.hp < health.max_hp)
        || !world.get::<&AgentState>(entity).is_ok_and(|state| {
            matches!(
                *state,
                AgentState::Seeking {
                    target: TargetKind::Activity
                } | AgentState::Consuming {
                    target: TargetKind::Activity
                }
            )
        })
    {
        return false;
    }
    let urgent = res
        .rules
        .agent
        .critical_preemption
        .unwrap_or(crate::runtime::agent_actions::needs::CRITICAL_NEED);
    if world
        .get::<&Hunger>(entity)
        .is_ok_and(|h| f64::from(h.value) >= urgent)
        || world
            .get::<&Thirst>(entity)
            .is_ok_and(|t| t.value >= urgent)
    {
        return false;
    }
    let Ok(activity) = world.get::<&Activity>(entity) else {
        return false;
    };
    let Ok(position) = world.get::<&Position>(entity) else {
        return false;
    };
    if activity.kind != ActivityKind::SitByFire
        || !res.nav.reachable((position.x, position.y), activity.place)
    {
        return false;
    }
    let Some(room) = res
        .colony
        .room_of_tile
        .get(&activity.place)
        .and_then(|id| res.colony.rooms.get(id))
        .filter(|room| room.roofed)
    else {
        return false;
    };
    let apparel = world.get::<&Apparel>(entity).ok();
    if thermal::felt_temperature(res, apparel.as_deref(), activity.place)
        .is_some_and(|felt| felt >= thermal.rules.cold_threshold_d10)
    {
        return true;
    }
    let rate = firewood::current_service_rate(res);
    res.settlements.iter().any(|(&sid, settlement)| {
        settlement.member_agents.contains(&agent)
            && rate > 0
            && settlement
                .stockpile
                .get(&ResourceKind::Wood)
                .copied()
                .unwrap_or(0)
                >= rate
            && firewood::primary_hearth_room(res, sid).is_some_and(|primary| primary.id == room.id)
    })
}
