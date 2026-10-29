//! Select bounded harvest orders for immediate meals and durable winter reserves.
use super::FoodSummary;
use sim_core::colony::content::{installed_or_bundled, FoodStorageClass};
use sim_core::colony::steward::{FoodPlan, Target};
use sim_core::colony::{Command, Designation};
use sim_engine::SimResources;
use std::cmp::Reverse;

type Tile = (u32, u32);

pub(super) fn plan(
    world: &hecs::World,
    res: &SimResources,
    policy: &FoodPlan,
    facts: &FoodSummary,
) -> Vec<Command> {
    let Some(hearth) = crate::runtime::jobs::hearth::hearth_tiles(res)
        .first()
        .copied()
    else {
        return Vec::new();
    };
    let immediate = facts
        .daily_milli
        .saturating_mul(u64::from(policy.reserve_days))
        .div_ceil(1000);
    let reserve = if facts.preparing {
        facts
            .daily_milli
            .saturating_mul(u64::from(facts.winter_days))
            .div_ceil(1000)
            .max(super::winter_need(facts.adults))
    } else {
        0
    };
    let mut immediate_gap = immediate.saturating_sub(facts.usable_units);
    let mut winter_gap = reserve.saturating_sub(facts.winter_units);
    let catalog = installed_or_bundled();
    let mut ingredients = super::dependencies::ingredient_deficits(world, res, policy, facts);
    let mut candidates = Vec::new();
    for (&tile, plant) in &res.food.plants {
        if plant.units_left == 0 || !crate::runtime::food::plants::harvestable(res, tile) {
            continue;
        }
        let Some(def) = catalog.plant(plant.species) else {
            continue;
        };
        let Some(food) = &def.yield_item.definition().food else {
            continue;
        };
        if distance(tile, hearth) > policy.forage_radius
            || !crate::commands::camp_reaches(res, tile)
        {
            continue;
        }
        let units = u64::from(plant.units_left) * u64::from(food.units);
        let durable = food.raw_edible
            && food.lifetime_days(FoodStorageClass::Stockpile)
                > facts.winter_in.saturating_add(policy.winter_keep_days);
        if res.colony.designations.get(&tile) == Some(&Designation::Harvest) {
            if let Some(missing) = ingredients.get_mut(&def.yield_item) {
                *missing = missing.saturating_sub(u64::from(plant.units_left));
            }
            if food.raw_edible {
                immediate_gap = immediate_gap.saturating_sub(units);
            }
            if durable {
                winter_gap = winter_gap.saturating_sub(units);
            }
            continue;
        }
        if res.colony.designations.contains_key(&tile)
            || res
                .steward
                .vetoed(&Target::Tile(tile.0, tile.1), res.current_tick)
        {
            continue;
        }
        let missing_group = food
            .groups
            .iter()
            .any(|group| !facts.groups.contains(group));
        candidates.push((tile, units, food.raw_edible, durable, missing_group));
    }
    let urgent = facts.usable_units.saturating_mul(1000) < facts.daily_milli;
    candidates.sort_by_key(|(tile, _, edible, durable, missing)| {
        (
            urgent && !edible,
            Reverse(*missing),
            Reverse(facts.preparing && *durable),
            distance(*tile, hearth),
            tile.1,
            tile.0,
        )
    });
    let mut out = Vec::new();
    let mut chosen_groups = facts.groups.clone();
    for (tile, units, edible, durable, _) in candidates {
        if u32::try_from(out.len()).unwrap_or(u32::MAX) >= policy.harvest_batch {
            break;
        }
        let Some(plant) = res.food.plants.get(&tile) else {
            continue;
        };
        let Some(def) = catalog.plant(plant.species) else {
            continue;
        };
        let Some(food) = &def.yield_item.definition().food else {
            continue;
        };
        let diversity = !urgent
            && food
                .groups
                .iter()
                .any(|group| !chosen_groups.contains(group));
        let ingredient_needed = ingredients.get(&def.yield_item).copied().unwrap_or(0) > 0;
        if (immediate_gap == 0 || !edible)
            && (winter_gap == 0 || !durable)
            && !diversity
            && !ingredient_needed
        {
            continue;
        }
        if urgent && !edible {
            continue;
        }
        out.push(Command::DesignateHarvest {
            x0: tile.0,
            y0: tile.1,
            x1: tile.0,
            y1: tile.1,
        });
        if edible {
            immediate_gap = immediate_gap.saturating_sub(units);
        }
        if durable {
            winter_gap = winter_gap.saturating_sub(units);
        }
        if let Some(missing) = ingredients.get_mut(&def.yield_item) {
            *missing = missing.saturating_sub(u64::from(plant.units_left));
        }
        chosen_groups.extend(food.groups.iter().cloned());
    }
    out
}
fn distance(a: Tile, b: Tile) -> u32 {
    a.0.abs_diff(b.0).max(a.1.abs_diff(b.1))
}
