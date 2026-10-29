//! Ideal raw-edible harvest capacity, not stock or a promise of completed meals.
use sim_core::colony::content::{installed_or_bundled, PlantDef, PlantPhase};
use sim_core::components::SettlementId;
use sim_core::rules::CalendarRule;
use sim_engine::food_supply_cache::FoodSupplyKey;
use sim_engine::SimResources;
use std::collections::BTreeMap;

const SHARE_SCALE: u64 = 1_000_000;

/// Existing legacy observer API projects integer capacity to units per day once.
pub(super) fn seasonal(res: &SimResources, sid: SettlementId, radius: u32) -> [f64; 4] {
    let totals = seasonal_units(res, sid, radius);
    let denominator = u64::from(res.rules.calendar.days_per_season).saturating_mul(SHARE_SCALE);
    if denominator == 0 {
        return [0.0; 4];
    }
    totals.map(|units| units as f64 / denominator as f64)
}

fn seasonal_units(res: &SimResources, sid: SettlementId, radius: u32) -> [u64; 4] {
    let Some(settlement) = res.settlements.get(&sid) else {
        return [0; 4];
    };
    let key = FoodSupplyKey {
        nav_version: res.nav.version,
        nav_built: res.nav.is_built(),
        center_radius: (settlement.formation_tile, radius),
        calendar: res.rules.calendar,
        sharing: if res.rules.society.fission.is_some() {
            res.settlements
                .iter()
                .filter(|(_, other)| !other.member_agents.is_empty())
                .map(|(id, other)| (*id, other.formation_tile))
                .collect()
        } else {
            Vec::new()
        },
        plant_count: res.food.plants.len(),
    };
    if let Some((old_key, value)) = res.food_supply_cache.borrow().entries.get(&(sid, radius)) {
        if *old_key == key {
            return *value;
        }
    }
    let mut supply = [0_u64; 4];
    let region = res.nav.region_of(settlement.formation_tile);
    if key.nav_built && region != 0 {
        let catalog = installed_or_bundled();
        let mut capacities = BTreeMap::new();
        let center = settlement.formation_tile;
        // Plant placement is immutable after initialization/load. Only this local
        // catchment is read on a geometry/nav cache miss, never the whole map.
        for x in center.0.saturating_sub(radius)
            ..=center
                .0
                .saturating_add(radius)
                .min(res.tile_grid.width.saturating_sub(1))
        {
            for (&tile, plant) in res
                .food
                .plants
                .range((x, center.1.saturating_sub(radius))..=(x, center.1.saturating_add(radius)))
            {
                if !res.nav.reaches(region, tile) {
                    continue;
                }
                let Some(def) = catalog.plant(plant.species) else {
                    continue;
                };
                let capacity = capacities
                    .entry(plant.species)
                    .or_insert_with(|| opportunities(def, &key.calendar));
                let shared = 1 + key
                    .sharing
                    .iter()
                    .filter(|(id, home)| {
                        *id != sid
                            && tile.0.abs_diff(home.0).max(tile.1.abs_diff(home.1)) <= radius
                            && res.nav.reaches(res.nav.region_of(*home), tile)
                    })
                    .count();
                for season in 0..4 {
                    // Divide once per plant, floor at one millionth of a unit.
                    let share = capacity[season].saturating_mul(SHARE_SCALE) / shared as u64;
                    supply[season] = supply[season].saturating_add(share);
                }
            }
        }
    }
    let mut cache = res.food_supply_cache.borrow_mut();
    cache.rebuilds = cache.rebuilds.saturating_add(1);
    cache.entries.insert((sid, radius), (key, supply));
    supply
}

fn opportunities(def: &PlantDef, calendar: &CalendarRule) -> [u64; 4] {
    let mut units = [0_u64; 4];
    let Some(food) = &def.yield_item.definition().food else {
        return units;
    };
    if !food.raw_edible || calendar.days_per_season == 0 {
        return units;
    }
    let mut next = 0;
    let mut previous_ripe = false;
    // Follow continuous ripe windows: seasonal boundaries do not magically
    // replenish a once-only plant when both adjoining phases remain ripe.
    for day in 0..calendar
        .days_per_season
        .saturating_mul(calendar.seasons_per_year.min(4))
    {
        let ripe = crate::runtime::food::plants::phase(def, calendar, day) == PlantPhase::Ripe;
        if ripe && !previous_ripe {
            next = day;
        }
        if ripe && day >= next {
            let season = (day / calendar.days_per_season) as usize;
            units[season] =
                units[season].saturating_add(u64::from(def.yield_count) * u64::from(food.units));
            next = if def.regrow_days == 0 {
                u32::MAX
            } else {
                day.saturating_add(def.regrow_days)
            };
        }
        previous_ripe = ripe;
    }
    units
}
