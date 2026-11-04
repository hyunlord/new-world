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
    let (immediate, reserve) = targets(policy, facts);
    let (pending_immediate, pending_winter) = pending_output(world, res, policy, facts);
    let mut immediate_gap = immediate
        .saturating_sub(facts.usable_units)
        .saturating_sub(pending_immediate)
        .saturating_sub(super::cooking::convertible_immediate(
            world, res, policy, facts,
        ));
    let mut winter_gap = reserve
        .saturating_sub(facts.winter_units)
        .saturating_sub(pending_winter);
    let catalog = installed_or_bundled();
    let mut ingredients = super::dependencies::ingredient_deficits(world, res, policy, facts);
    let mut seed_need = super::dependencies::seed_inputs(world, res, policy, facts);
    let mut seed_available = super::cooking::free_inputs(world, res);
    let seed_recipe = super::dependencies::seed_recipe(policy).and_then(|kind| kind.recipe());
    for (kind, missing) in &mut seed_need {
        let physical = if kind.definition().food.is_some() {
            if let Some(recipe) = seed_recipe {
                super::cooking::take_recipe_input(
                    world,
                    res,
                    &mut seed_available,
                    recipe,
                    *kind,
                    *missing,
                )
            } else {
                seed_available.get(&(None, *kind)).copied().unwrap_or(0)
            }
        } else {
            super::dependencies::physical_pieces(world, res, *kind)
        };
        *missing = missing.saturating_sub(physical);
    }
    let mut candidates = Vec::new();
    for (&tile, plant) in &res.food.plants {
        if plant.units_left == 0 || !crate::runtime::food::plants::harvestable(res, tile) {
            continue;
        }
        let Some(def) = catalog.plant(plant.species) else {
            continue;
        };
        let food = def.yield_item.definition().food.as_ref();
        if distance(tile, hearth) > policy.forage_radius
            || !crate::commands::camp_reaches(res, tile)
        {
            continue;
        }
        let units = u64::from(plant.units_left) * food.map_or(0, |food| u64::from(food.units));
        let edible = food.is_some_and(|food| food.raw_edible);
        let durable = food.is_some_and(|food| {
            food.raw_edible
                && super::summary::covers_winter(
                    food.lifetime_days(FoodStorageClass::Stockpile),
                    facts.winter_in,
                    policy.winter_keep_days,
                )
        });
        if res.colony.designations.get(&tile) == Some(&Designation::Harvest) {
            if let Some(missing) = seed_need.get_mut(&def.yield_item) {
                *missing = missing.saturating_sub(u64::from(plant.units_left));
            }
            if let Some(missing) = ingredients.get_mut(&def.yield_item) {
                *missing = missing.saturating_sub(u64::from(plant.units_left));
            }
            if edible {
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
        let missing_group = food.is_some_and(|food| {
            food.groups
                .iter()
                .any(|group| !facts.groups.contains(group))
        });
        candidates.push((tile, units, edible, durable, missing_group));
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
        let food = def.yield_item.definition().food.as_ref();
        let seed_needed = seed_need.get(&def.yield_item).copied().unwrap_or(0) > 0;
        if immediate_gap == 0 && winter_gap == 0 && !seed_needed {
            continue;
        }
        let diversity = immediate_gap > 0
            && !urgent
            && food.is_some_and(|food| {
                food.groups
                    .iter()
                    .any(|group| !chosen_groups.contains(group))
            });
        let ingredient_needed = ingredients.get(&def.yield_item).copied().unwrap_or(0) > 0;
        if (immediate_gap == 0 || !edible)
            && (winter_gap == 0 || !durable)
            && !diversity
            && !ingredient_needed
            && !seed_needed
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
        if let Some(missing) = seed_need.get_mut(&def.yield_item) {
            *missing = missing.saturating_sub(u64::from(plant.units_left));
        }
        if edible {
            immediate_gap = immediate_gap.saturating_sub(units);
        }
        if durable {
            winter_gap = winter_gap.saturating_sub(units);
        }
        if let Some(missing) = ingredients.get_mut(&def.yield_item) {
            *missing = missing.saturating_sub(u64::from(plant.units_left));
        }
        if let Some(food) = food {
            chosen_groups.extend(food.groups.iter().cloned());
        }
    }
    out
}
pub(super) fn pending_output(
    world: &hecs::World,
    res: &SimResources,
    policy: &FoodPlan,
    facts: &FoodSummary,
) -> (u64, u64) {
    use sim_core::colony::{Job, JobKind, JobStage};
    let day = crate::runtime::food::day(res);
    let winter_start = day.saturating_add(facts.winter_in);
    let mut immediate = 0u64;
    let mut winter = 0u64;
    let mut count = |kind: sim_core::colony::ItemKind, pieces: u16, due: u64, tile: Tile| {
        if !crate::commands::camp_reaches(res, tile) {
            return;
        }
        let Some(food) = &kind.definition().food else {
            return;
        };
        if !food.raw_edible {
            return;
        }
        let Ok(born) = u32::try_from(crate::runtime::resource_regen::day_index_at(res, due)) else {
            return;
        };
        let lifetime = food.lifetime_days(crate::runtime::food::storage::class(
            res,
            sim_core::colony::food::FoodOwner::Ground(tile),
        ));
        if born.saturating_add(lifetime) <= day {
            return;
        }
        let units = u64::from(pieces).saturating_mul(u64::from(food.units));
        if born <= day.saturating_add(policy.reserve_days) {
            immediate = immediate.saturating_add(units);
        }
        if facts.preparing
            && born <= winter_start
            && born.saturating_add(food.lifetime_days(FoodStorageClass::Stockpile))
                >= winter_start.saturating_add(policy.winter_keep_days)
        {
            winter = winter.saturating_add(units);
        }
    };
    for batch in &res.food.pending_drying {
        if res.colony.workshops.contains_key(&batch.workshop) {
            count(batch.output.kind, batch.count, batch.due_tick, batch.tile);
        }
    }
    for (_, job) in world.query::<&Job>().iter() {
        let (JobKind::Craft { workshop, bill, .. }, JobStage::Working { ticks_left }) =
            (job.kind, job.stage)
        else {
            continue;
        };
        let Some(shop) = res.colony.workshops.get(&workshop) else {
            continue;
        };
        let Some(recipe) = shop.bill(bill).and_then(|bill| bill.recipe.recipe()) else {
            continue;
        };
        let due = res
            .current_tick
            .saturating_add(u64::from(ticks_left))
            .saturating_add(
                u64::from(recipe.food_requirements.duration_days).saturating_mul(res.ticks_per_day),
            );
        count(recipe.output.0, recipe.output.1, due, shop.work_tile());
    }
    (immediate, winter)
}

fn distance(a: Tile, b: Tile) -> u32 {
    a.0.abs_diff(b.0).max(a.1.abs_diff(b.1))
}

pub(super) fn targets(policy: &FoodPlan, facts: &FoodSummary) -> (u64, u64) {
    let immediate = facts
        .daily_milli
        .saturating_mul(u64::from(policy.reserve_days))
        .div_ceil(1000);
    let reserve = if facts.preparing {
        super::summary::winter_target(policy, facts)
    } else {
        0
    };
    (immediate, reserve)
}
