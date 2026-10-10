//! Deterministic renewable food accounting shared by gates and probe reports.

use crate::runtime::decision::HUNGER_CONSUME_AMOUNT;
use crate::runtime::resource_regen::{FOOD_REGEN_AMOUNT, REGEN_INTERVAL};
use hecs::World;
use sim_core::components::{Agent, Hunger, SettlementId};
use sim_engine::SimResources;

/// Founder food requirement per standard 1,440-tick day.
pub const ADULT_NEED_PER_DAY: f64 = 2.4;

/// Founder food requirement scaled to the configured day length.
pub fn adult_need_per_day(ticks_per_day: u64) -> f64 {
    0.05 * ticks_per_day as f64 / f64::from(HUNGER_CONSUME_AMOUNT)
}

/// Food units required by an agent's actual hunger growth per configured day.
pub fn need_per_day(hunger: &Hunger, ticks_per_day: u64) -> f64 {
    f64::from(hunger.growth_rate) * ticks_per_day as f64 / f64::from(HUNGER_CONSUME_AMOUNT)
}

/// Daily renewable production of one registered food source.
pub fn source_supply_per_day(ticks_per_day: u64) -> f64 {
    f64::from(FOOD_REGEN_AMOUNT) * ticks_per_day as f64 / REGEN_INTERVAL as f64
}

/// Renewable daily food supply inside the formation tile's forage radius.
pub fn supply_per_day(resources: &SimResources, sid: SettlementId, forage_radius: u32) -> f64 {
    let Some(settlement) = resources.settlements.get(&sid) else {
        return 0.0;
    };
    let center = settlement.formation_tile;
    let sources = resources
        .food_source_max
        .keys()
        .filter(|tile| tile.0.abs_diff(center.0).max(tile.1.abs_diff(center.1)) <= forage_radius)
        .count();
    sources as f64 * source_supply_per_day(resources.ticks_per_day)
}

/// Sum actual daily needs of live members in stable agent-id order.
pub fn members_need_per_day(world: &World, resources: &SimResources, sid: SettlementId) -> f64 {
    let Some(settlement) = resources.settlements.get(&sid) else {
        return 0.0;
    };
    let mut members: Vec<_> = world
        .query::<(&Agent, &Hunger)>()
        .iter()
        .filter(|(_, (agent, _))| settlement.member_agents.contains(&agent.id))
        .map(|(_, (agent, hunger))| (agent.id, need_per_day(hunger, resources.ticks_per_day)))
        .collect();
    members.sort_by_key(|(id, _)| *id);
    members.iter().map(|(_, need)| *need).sum()
}

/// Future adults supported by the usable share of renewable supply.
pub fn sustainable_adults(supply: f64, utilization: f64, ticks_per_day: u64) -> u32 {
    let need = adult_need_per_day(ticks_per_day);
    if need <= 0.0 {
        return 0;
    }
    (utilization * supply / need).floor() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::{components::Settlement, material::MaterialRegistry};

    #[test]
    fn ten_bushes_support_forty_two_adults_and_twenty_four_founders_need_fifty_seven() {
        let mut resources = SimResources::new(64, 64, MaterialRegistry::new());
        let mut world = World::new();
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.formation_tile = (20, 20);
        for id in 1..=24 {
            settlement.add_member_agent(id);
            world.spawn((Agent { id }, Hunger::new(0.0, 0.05)));
        }
        resources.settlements.insert(1, settlement);
        for x in 20..30 {
            resources.food_source_max.insert((x, 20), 4);
        }
        resources.food_source_max.insert((37, 20), 4);
        assert_eq!(supply_per_day(&resources, 1, 16), 120.0);
        assert!((members_need_per_day(&world, &resources, 1) - 57.6).abs() < 1e-4);
        assert_eq!(adult_need_per_day(1440), ADULT_NEED_PER_DAY);
        assert_eq!(sustainable_adults(120.0, 0.85, 1440), 42);
        assert_eq!(need_per_day(&Hunger::new(0.0, 0.0), 1440), 0.0);
        assert_eq!(sustainable_adults(120.0, 0.85, 0), 0);
        assert_eq!(supply_per_day(&resources, 99, 16), 0.0);
    }
}
