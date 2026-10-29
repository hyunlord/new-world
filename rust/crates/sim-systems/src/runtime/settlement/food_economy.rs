//! Deterministic renewable food accounting shared by gates and probe reports.

#[path = "food_supply.rs"]
mod food_supply;

use crate::runtime::agent_actions::needs::HUNGER_CONSUME_AMOUNT;
use crate::runtime::resource_regen::{season_at, FOOD_REGEN_AMOUNT, REGEN_INTERVAL};
use hecs::World;
use sim_core::components::{Agent, Cold, Hunger, Inventory, ResourceKind, SettlementId};
use sim_core::rules::BirthRule;
use sim_engine::SimResources;

/// Winter index in `SeasonRule::regen_permille`.
pub const WINTER: usize = 3;

/// Renewable sources inside the formation tile's forage radius.
fn source_count(resources: &SimResources, sid: SettlementId, forage_radius: u32) -> usize {
    let Some(settlement) = resources.settlements.get(&sid) else {
        return 0;
    };
    let center = settlement.formation_tile;
    resources
        .food_source_max
        .keys()
        .filter(|tile| tile.0.abs_diff(center.0).max(tile.1.abs_diff(center.1)) <= forage_radius)
        .count()
}

/// Supply under one season's regrowth permille, or the fixed rate without seasons.
pub fn season_supply_per_day(
    resources: &SimResources,
    sid: SettlementId,
    forage_radius: u32,
    season: usize,
) -> f64 {
    if resources.rules.colony.is_some() {
        return food_supply::seasonal(resources, sid, forage_radius)
            .get(season)
            .copied()
            .unwrap_or(0.0);
    }
    let base = supply_per_day(resources, sid, forage_radius);
    match resources.rules.seasons {
        Some(rule) => base * f64::from(rule.regen_permille[season.min(WINTER)]) / 1000.0,
        None => base,
    }
}

/// Mean daily supply over a year; equals the fixed supply without seasons.
pub fn annual_mean_supply(resources: &SimResources, sid: SettlementId, forage_radius: u32) -> f64 {
    if resources.rules.colony.is_some() {
        let seasons = resources.rules.calendar.seasons_per_year;
        return if seasons == 0 {
            0.0
        } else {
            food_supply::seasonal(resources, sid, forage_radius)
                .iter()
                .sum::<f64>()
                / f64::from(seasons)
        };
    }
    let base = supply_per_day(resources, sid, forage_radius);
    match resources.rules.seasons {
        Some(rule) => {
            let total: u32 = rule.regen_permille.iter().map(|p| u32::from(*p)).sum();
            base * f64::from(total) / 4000.0
        }
        None => base,
    }
}

/// Winter daily supply; equals the fixed supply without seasons.
pub fn winter_supply_per_day(
    resources: &SimResources,
    sid: SettlementId,
    forage_radius: u32,
) -> f64 {
    season_supply_per_day(resources, sid, forage_radius, WINTER)
}

/// Catchment radius used by the store target: sustain radius, else forage radius, else 16.
pub fn catchment_radius(resources: &SimResources) -> u32 {
    match resources.rules.society.birth {
        BirthRule::Provisioned {
            sustain: Some(rule),
            ..
        } => rule.forage_radius,
        _ => resources
            .rules
            .agent
            .forage
            .map_or(16, |rule| rule.forage_radius),
    }
}

/// Stockpile food plus food carried by live members, summed in AgentId order.
pub fn stock_in_hand(world: &World, resources: &SimResources, sid: SettlementId) -> u64 {
    if resources.rules.colony.is_some() {
        return crate::runtime::food::reserves::units(world, resources, sid, true);
    }
    let Some(settlement) = resources.settlements.get(&sid) else {
        return 0;
    };
    let mut carried: Vec<_> = world
        .query::<(&Agent, &Inventory)>()
        .iter()
        .filter(|(_, (agent, _))| settlement.member_agents.contains(&agent.id))
        .map(|(_, (agent, inventory))| (agent.id, inventory.get(ResourceKind::Food)))
        .collect();
    carried.sort_by_key(|(id, _)| *id);
    u64::from(
        settlement
            .stockpile
            .get(&ResourceKind::Food)
            .copied()
            .unwrap_or(0),
    ) + carried
        .iter()
        .map(|(_, food)| u64::from(*food))
        .sum::<u64>()
}

/// Store target: `base_days x need`, plus in summer and autumn (seasons with
/// winter preparation) `days_per_season x max(0, need - winter supply)`.
/// `None` without a stock-target rule.
pub fn stock_target(world: &World, resources: &SimResources, sid: SettlementId) -> Option<f64> {
    let rule = resources.rules.society.stock_target?;
    let need = members_need_per_day(world, resources, sid);
    let mut target = rule.base_days * need;
    if let (Some(_), Some((season, _))) = (resources.rules.seasons, season_at(resources)) {
        if season == 1 || season == 2 {
            let winter = winter_supply_per_day(resources, sid, catchment_radius(resources));
            target += f64::from(resources.rules.season_days()) * (need - winter).max(0.0);
        }
    }
    Some(target)
}

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

/// Bush capacity at which a bush counts one whole source under
/// a sustain rule (Pass 10; v15's capacity).
pub const REFERENCE_BUSH_CAPACITY: u8 = 4;

/// Sources inside the formation tile's forage radius, each weighted
/// `min(capacity, 4) / 4` (Pass 10, `game_v16`). Summed over sorted tiles.
/// Pass 11: under `FissionRule` a bush shared with n − 1 other settlements'
/// catchments counts 1/n.
fn weighted_source_count(resources: &SimResources, sid: SettlementId, forage_radius: u32) -> f64 {
    let Some(settlement) = resources.settlements.get(&sid) else {
        return 0.0;
    };
    let center = settlement.formation_tile;
    let mut weights: Vec<((u32, u32), u8)> = resources
        .food_source_max
        .iter()
        .filter(|(tile, _)| {
            tile.0.abs_diff(center.0).max(tile.1.abs_diff(center.1)) <= forage_radius
        })
        .map(|(tile, max)| (*tile, (*max).min(REFERENCE_BUSH_CAPACITY)))
        .collect();
    weights.sort_unstable();
    // Pass 11 (`game_v17`, under `FissionRule`): a bush inside the forage
    // radius of n settlements (this one and every other with a member) counts
    // 1/n, so overlapping catchments are not planned twice by two birth gates.
    if resources.rules.society.fission.is_some() {
        let mut others: Vec<(u32, u32)> = resources
            .settlements
            .iter()
            .filter(|(other, s)| **other != sid && !s.member_agents.is_empty())
            .map(|(_, s)| s.formation_tile)
            .collect();
        others.sort_unstable();
        return weights
            .iter()
            .map(|(tile, w)| {
                let shared = others
                    .iter()
                    .filter(|c| tile.0.abs_diff(c.0).max(tile.1.abs_diff(c.1)) <= forage_radius)
                    .count();
                f64::from(*w) / (1 + shared) as f64
            })
            .sum::<f64>()
            / f64::from(REFERENCE_BUSH_CAPACITY);
    }
    weights.iter().map(|(_, w)| f64::from(*w)).sum::<f64>() / f64::from(REFERENCE_BUSH_CAPACITY)
}

/// Renewable daily food supply inside the formation tile's forage radius.
pub fn supply_per_day(resources: &SimResources, sid: SettlementId, forage_radius: u32) -> f64 {
    if resources.rules.colony.is_some() {
        let day = crate::runtime::food::day(resources);
        let (_, season, _, _) = resources.rules.calendar.date_of_day(u64::from(day));
        return food_supply::seasonal(resources, sid, forage_radius)
            .get(season as usize)
            .copied()
            .unwrap_or(0.0);
    }
    // M1.5: the colony's `Fertility` rule keeps the weighted count its
    // provisioned rule used, so its store target is unchanged by the switch.
    let weighted = matches!(
        resources.rules.society.birth,
        BirthRule::Provisioned {
            sustain: Some(_),
            ..
        } | BirthRule::Fertility { .. }
    );
    let sources = if weighted {
        weighted_source_count(resources, sid, forage_radius)
    } else {
        source_count(resources, sid, forage_radius) as f64
    };
    sources * source_supply_per_day(resources.ticks_per_day)
}

/// Sum actual daily needs of live members in stable agent-id order.
pub fn members_need_per_day(world: &World, resources: &SimResources, sid: SettlementId) -> f64 {
    let Some(settlement) = resources.settlements.get(&sid) else {
        return 0.0;
    };
    let mut members: Vec<_> = if resources.rules.colony.is_some() {
        let cold_multiplier = resources
            .rules
            .society
            .firewood
            .map(|rule| rule.cold_hunger_permille);
        world
            .query::<(&Agent, &Hunger, Option<&Cold>)>()
            .iter()
            .filter(|(_, (agent, _, _))| settlement.member_agents.contains(&agent.id))
            .map(|(_, (agent, hunger, cold))| {
                // Match HungerDecaySystem's f32 growth before projecting to days.
                let growth = match (cold, cold_multiplier) {
                    (Some(_), Some(permille)) => hunger.growth_rate * permille as f32 / 1000.0,
                    _ => hunger.growth_rate,
                };
                (
                    agent.id,
                    need_per_day(&Hunger::new(0.0, growth), resources.ticks_per_day),
                )
            })
            .collect()
    } else {
        world
            .query::<(&Agent, &Hunger)>()
            .iter()
            .filter(|(_, (agent, _))| settlement.member_agents.contains(&agent.id))
            .map(|(_, (agent, hunger))| (agent.id, need_per_day(hunger, resources.ticks_per_day)))
            .collect()
    };
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
