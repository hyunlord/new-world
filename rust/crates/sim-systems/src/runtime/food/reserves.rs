//! Physical edible reserves for shared settlement observers, without legacy counters.
use hecs::World;
use sim_core::colony::content::FoodStorageClass;
use sim_core::colony::food::FoodOwner;
use sim_core::components::SettlementId;
use sim_engine::SimResources;

/// Fresh stored units reachable from this settlement, plus its carried food when requested.
pub fn units(world: &World, res: &SimResources, sid: SettlementId, include_carried: bool) -> u64 {
    let Some(settlement) = res.settlements.get(&sid) else {
        return 0;
    };
    let day = super::day(res);
    res.food
        .ages
        .keys()
        .copied()
        .filter_map(|owner| {
            let reachable = match owner {
                FoodOwner::Carried(id) => include_carried && settlement.member_agents.contains(&id),
                FoodOwner::Ground(tile) => owner_at(res, tile) == Some(sid),
                FoodOwner::Shelf { workshop, .. } => res
                    .colony
                    .workshops
                    .get(&workshop)
                    .is_some_and(|shop| owner_at(res, shop.work_tile()) == Some(sid)),
                FoodOwner::Pot { id, .. } => res
                    .food
                    .pots
                    .get(&id)
                    .is_some_and(|pot| owner_at(res, pot.tile) == Some(sid)),
            };
            if !reachable {
                return None;
            }
            if !matches!(owner, FoodOwner::Carried(_))
                && super::storage::class(res, owner) == FoodStorageClass::Ground
            {
                return None;
            }
            let stack = super::items::stack(world, res, owner)?;
            let food = stack.kind.definition().food.as_ref()?;
            if !food.raw_edible || super::storage::remaining_days(world, res, owner, day).ok()? == 0
            {
                return None;
            }
            Some(u64::from(food.units) * u64::from(stack.count))
        })
        .sum()
}

fn owner_at(res: &SimResources, tile: (u32, u32)) -> Option<SettlementId> {
    let region = res.nav.region_of(tile);
    if region == 0 {
        return None;
    }
    res.settlements
        .iter()
        .filter(|(_, settlement)| res.nav.region_of(settlement.formation_tile) == region)
        .min_by_key(|(id, settlement)| {
            let home = settlement.formation_tile;
            (home.0.abs_diff(tile.0).max(home.1.abs_diff(tile.1)), **id)
        })
        .map(|(id, _)| *id)
}
