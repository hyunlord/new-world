//! Authoritative settlement accounting for bulk UI reads.

use sim_engine::SimResources;
use sim_systems::runtime::settlement::{firewood, food_economy};

/// One settlement's stock and needs, sorted by settlement id.
#[derive(Clone, Debug, PartialEq)]
pub struct SettlementFact {
    /// Stable settlement identity.
    pub id: u32,
    /// Formation anchor for camera jumps.
    pub tile: (u32, u32),
    /// Settlement food plus live members' carried food.
    pub food: u64,
    /// Authoritative food consumption per configured day.
    pub food_need: f64,
    /// Settlement wood plus live members' carried wood.
    pub wood: u64,
    /// Authoritative target including seasonal preparation.
    pub wood_target: f64,
    /// Winter daily burn before unknown upcoming severity.
    pub winter_wood_need: f64,
}

pub(super) fn collect(world: &hecs::World, res: &SimResources) -> Vec<SettlementFact> {
    let mut rows: Vec<_> = res
        .settlements
        .iter()
        .map(|(&id, s)| SettlementFact {
            id,
            tile: s.formation_tile,
            food: food_economy::stock_in_hand(world, res, id),
            food_need: food_economy::members_need_per_day(world, res, id),
            wood: firewood::wood_in_hand(world, res, id),
            wood_target: firewood::wood_target(res).unwrap_or(0.0),
            winter_wood_need: res
                .rules
                .society
                .firewood
                .map_or(0.0, |r| firewood::burn_per_day(&r, 3)),
        })
        .collect();
    rows.sort_by_key(|r| r.id);
    rows
}
