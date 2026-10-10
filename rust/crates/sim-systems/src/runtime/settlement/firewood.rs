//! Revitalize Pass 5 — firewood accounting shared by decisions, the fire and probes.
//!
//! Pure reads: none of these helpers mutates simulation state or draws RNG.
//! Without `SocietyRules::firewood` every helper reports nothing (`None` / 0).

use crate::runtime::decision::agent_decision::clock_in_window;
use crate::runtime::resource_regen::season_at;
use hecs::World;
use sim_core::components::{Agent, Inventory, ResourceKind, SettlementId};
use sim_core::rules::FirewoodRule;
use sim_engine::SimResources;

/// Minutes in a day; windows are minute-of-day.
const MINUTES_PER_DAY: u32 = 1440;

/// Whole hours in the half-open, possibly midnight-wrapping burn window
/// (13 for 18:00–07:00).
pub fn burn_hours(rule: &FirewoodRule) -> u32 {
    let start = rule.burn_start_minute % MINUTES_PER_DAY;
    let end = rule.burn_end_minute % MINUTES_PER_DAY;
    (end + MINUTES_PER_DAY - start) % MINUTES_PER_DAY / 60
}

/// Index into `burn_per_hour`: the current season, or 0 without seasons.
pub fn burn_season(resources: &SimResources) -> usize {
    season_at(resources).map_or(0, |(season, _)| season.min(3))
}

/// Wood burned per night in season `season`: `burn_per_hour[season] x burn hours`.
pub fn burn_per_day(rule: &FirewoodRule, season: usize) -> f64 {
    f64::from(rule.burn_per_hour[season.min(3)]) * f64::from(burn_hours(rule))
}

/// Wood target of every settlement: `base_days x burn_per_day(current)`, plus a
/// winter's burn (`days_per_season x burn_per_day(winter)`) in summer and
/// autumn when seasons are on and winter preparation is enabled. `None`
/// without the firewood rule.
pub fn wood_target(resources: &SimResources) -> Option<f64> {
    let rule = resources.rules.society.firewood?;
    let season = burn_season(resources);
    let mut target = rule.base_days * burn_per_day(&rule, season);
    if let (true, Some(seasons), Some((current, _))) =
        (rule.winter_prep, resources.rules.seasons, season_at(resources))
    {
        if current == 1 || current == 2 {
            target += f64::from(seasons.days_per_season) * burn_per_day(&rule, 3);
        }
    }
    Some(target)
}

/// Stockpile wood plus wood carried by live members, summed in AgentId order.
pub fn wood_in_hand(world: &World, resources: &SimResources, sid: SettlementId) -> u64 {
    let Some(settlement) = resources.settlements.get(&sid) else {
        return 0;
    };
    let mut carried: Vec<_> = world
        .query::<(&Agent, &Inventory)>()
        .iter()
        .filter(|(_, (agent, _))| settlement.member_agents.contains(&agent.id))
        .map(|(_, (agent, inventory))| (agent.id, inventory.get(ResourceKind::Wood)))
        .collect();
    carried.sort_by_key(|(id, _)| *id);
    u64::from(settlement.stockpile.get(&ResourceKind::Wood).copied().unwrap_or(0))
        + carried.iter().map(|(_, wood)| u64::from(*wood)).sum::<u64>()
}

/// Whether the clock lies inside the firewood gathering window.
pub fn in_gather_window(resources: &SimResources) -> bool {
    resources.rules.society.firewood.is_some_and(|rule| {
        clock_in_window(resources, rule.gather_start_minute, rule.gather_end_minute)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::components::Settlement;
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::{SeasonRule, WorldRules};

    fn rule() -> FirewoodRule {
        FirewoodRule {
            gather_start_minute: 600,
            gather_end_minute: 1020,
            wood_radius: 18,
            haul_threshold: 2,
            max_gatherers_per_spot: 1,
            regen_permille: 500,
            base_days: 1.5,
            winter_prep: true,
            burn_start_minute: 1080,
            burn_end_minute: 420,
            burn_per_hour: [2, 1, 2, 3],
            cold_start_minute: 1320,
            cold_end_minute: 360,
            cold_hunger_permille: 1500,
        }
    }

    #[test]
    fn burn_window_is_thirteen_hours_and_burn_per_day_scales() {
        let r = rule();
        assert_eq!(burn_hours(&r), 13);
        assert_eq!(burn_per_day(&r, 0), 26.0);
        assert_eq!(burn_per_day(&r, 3), 39.0);
    }

    #[test]
    fn target_adds_a_winter_of_burn_in_summer_and_autumn_only() {
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        assert_eq!(wood_target(&resources), None);
        resources.rules = WorldRules::game_v10();
        resources.rules.society.firewood = Some(rule());
        resources.rules.seasons = Some(SeasonRule { days_per_season: 7, regen_permille: [750, 1250, 1000, 500] });
        // Display day d at 08:00: tick = d x 1440 (480-tick clock offset).
        let at_day = |r: &mut SimResources, d: u64| r.current_tick = d * 1440;
        at_day(&mut resources, 0);
        assert_eq!(wood_target(&resources), Some(1.5 * 26.0));
        at_day(&mut resources, 7);
        assert_eq!(wood_target(&resources), Some(1.5 * 13.0 + 7.0 * 39.0));
        at_day(&mut resources, 14);
        assert_eq!(wood_target(&resources), Some(1.5 * 26.0 + 7.0 * 39.0));
        at_day(&mut resources, 21);
        assert_eq!(wood_target(&resources), Some(1.5 * 39.0));
        resources.rules.seasons = None;
        assert_eq!(wood_target(&resources), Some(1.5 * 26.0));
    }

    #[test]
    fn wood_in_hand_counts_stockpile_and_members_only() {
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        let mut world = World::new();
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.add_member_agent(1);
        settlement.store(ResourceKind::Wood, 5);
        resources.settlements.insert(1, settlement);
        for (id, wood) in [(1u64, 2u32), (2, 7)] {
            let mut inventory = Inventory::default();
            inventory.add(ResourceKind::Wood, wood);
            world.spawn((Agent { id }, inventory));
        }
        assert_eq!(wood_in_hand(&world, &resources, 1), 7);
        assert_eq!(wood_in_hand(&world, &resources, 9), 0);
    }
}
