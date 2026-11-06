//! Physical refuge facts: real heat/fuel, reachable room tiles and existing seats.
use super::context::{PersonSnapshot, TargetCandidate, TickContext};
use crate::runtime::{settlement::firewood, thermal};
use sim_core::colony::JobKind;
use sim_core::components::{ActivityKind, ResourceKind};
use sim_engine::SimResources;

pub(super) fn active(person: &PersonSnapshot) -> bool {
    person
        .activity
        .is_some_and(|a| a.kind == ActivityKind::SitByFire)
}

pub(super) fn fuel_job(person: &PersonSnapshot, res: &SimResources) -> bool {
    let Some(job) = person.job else {
        return false;
    };
    let Some(settlement) = res
        .settlements
        .values()
        .find(|s| s.member_agents.contains(&person.agent))
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

    match job.kind {
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
                                && person
                                    .carry
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
            person
                .carry
                .is_some_and(|carry| carry.kind.definition().fuel_units.is_some())
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
