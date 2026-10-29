//! Demand-bounded empty stockpile extensions; normal haul jobs move every item.
use super::FoodSummary;
use crate::runtime::jobs::items::{can_hold_items, room_on, stack_max};
use sim_core::colony::steward::{FoodPlan, Target};
use sim_core::colony::{quality_bucket, Command, ItemSpec, ReserveKey};
use sim_engine::SimResources;
use std::collections::BTreeMap;

type Tile = (u32, u32);
fn distance(a: Tile, b: Tile) -> u32 {
    a.0.abs_diff(b.0).max(a.1.abs_diff(b.1))
}

pub(super) fn plan(res: &SimResources, policy: &FoodPlan, facts: &FoodSummary) -> Vec<Command> {
    let Some(hearth) = crate::runtime::jobs::hearth::hearth_tiles(res)
        .first()
        .copied()
    else {
        return Vec::new();
    };
    // Quantities never merge across material or quality buckets. Only local,
    // reachable inventory is a reason to allocate more stockpile cells.
    let mut buckets: BTreeMap<ItemSpec, u64> = BTreeMap::new();
    let mut nonfood_occupied = 0_u64;
    for (&tile, stack) in &res.colony.ground_items {
        if stack.count == 0
            || distance(tile, hearth) > policy.forage_radius
            || !crate::commands::camp_reaches(res, tile)
        {
            continue;
        }
        if let Some(food) = &stack.kind.definition().food {
            let owner = sim_core::colony::food::FoodOwner::Ground(tile);
            let day = crate::runtime::food::day(res);
            if !res.food.ages.get(&owner).is_some_and(|age| {
                age.born_day <= day
                    && age.remaining(
                        day,
                        food.lifetime_days(crate::runtime::food::storage::class(res, owner)),
                    ) > 0
            }) {
                continue;
            }
        }
        let mut spec = stack.spec();
        spec.quality = quality_bucket(spec.quality) * 10;
        *buckets.entry(spec).or_default() += u64::from(stack.count);
        if spec.kind.definition().food.is_none() && res.colony.zone_of_tile.contains_key(&tile) {
            nonfood_occupied += 1;
        }
    }
    let food_buckets: Vec<_> = buckets
        .iter()
        .filter(|(spec, _)| spec.kind.definition().food.is_some())
        .collect();
    if food_buckets.is_empty() {
        return Vec::new();
    }
    let min_food_stack_units = food_buckets
        .iter()
        .filter_map(|(spec, _)| {
            let food = spec.kind.definition().food.as_ref()?;
            food.raw_edible
                .then_some(u64::from(stack_max(res, spec.kind)) * u64::from(food.units.max(1)))
        })
        .min();
    let reserve = facts
        .daily_milli
        .saturating_mul(u64::from(if facts.preparing {
            facts.winter_days.max(policy.reserve_days)
        } else {
            policy.reserve_days
        }))
        .div_ceil(1000)
        .max(if facts.preparing {
            super::winter_need(facts.adults)
        } else {
            0
        });
    // Inedible ingredients still need physical slots, but never receive
    // an invented edible-unit conversion for future reserve capacity.
    let reserve_slots = min_food_stack_units.map_or(0, |units| reserve.div_ceil(units));
    let reserve_gap_slots = min_food_stack_units.map_or(0, |units| {
        reserve.saturating_sub(facts.usable_units).div_ceil(units)
    });
    let nonfood_slots: u64 = buckets
        .iter()
        .filter(|(spec, _)| spec.kind.definition().food.is_none())
        .map(|(spec, count)| count.div_ceil(u64::from(stack_max(res, spec.kind))))
        .sum();
    let food_slots: u64 = food_buckets
        .iter()
        .map(|(spec, count)| count.div_ceil(u64::from(stack_max(res, spec.kind))))
        .sum();
    // An absolute inventory/demand bound, not an additive allowance each firing.
    // Existing food must fit even when it exceeds the minimum winter reserve.
    let limit = nonfood_slots
        .max(nonfood_occupied)
        .saturating_add(food_buckets.len() as u64)
        .saturating_add(food_slots.max(reserve_slots));
    let zones: Vec<_> = res
        .colony
        .zone_of_tile
        .keys()
        .copied()
        .filter(|&tile| {
            distance(tile, hearth) <= policy.forage_radius
                && crate::commands::camp_reaches(res, tile)
        })
        .collect();
    let room_to_expand = limit.saturating_sub(zones.len() as u64);
    if room_to_expand == 0 {
        return Vec::new();
    }
    let empty = zones
        .iter()
        .filter(|tile| {
            !res.colony.ground_items.contains_key(tile)
                && can_hold_items(res, **tile)
                && !res
                    .colony
                    .reservations
                    .contains_key(&ReserveKey::Cell(**tile))
        })
        .count() as u64;
    let mut missing_slots = 0_u64;
    for (&spec, &count) in &buckets {
        let present: u64 = zones
            .iter()
            .filter_map(|tile| res.colony.ground_items.get(tile))
            .filter(|stack| stack.spec().merges_with(&spec))
            .map(|stack| u64::from(stack.count))
            .sum();
        let room: u64 = zones
            .iter()
            .filter(|tile| {
                res.colony.ground_items.contains_key(tile)
                    && !res
                        .colony
                        .reservations
                        .contains_key(&ReserveKey::Cell(**tile))
            })
            .map(|&tile| u64::from(room_on(res, tile, spec)))
            .sum();
        missing_slots = missing_slots.saturating_add(
            count
                .saturating_sub(present)
                .saturating_sub(room)
                .div_ceil(u64::from(stack_max(res, spec.kind))),
        );
    }
    missing_slots = missing_slots
        .saturating_add(reserve_gap_slots)
        .saturating_sub(empty);
    let count = missing_slots
        .min(room_to_expand)
        .min(u64::from(policy.harvest_batch));
    if count == 0 {
        return Vec::new();
    }
    let mut candidates = Vec::new();
    for y in hearth.1.saturating_sub(policy.forage_radius)
        ..=hearth
            .1
            .saturating_add(policy.forage_radius)
            .min(res.tile_grid.height.saturating_sub(1))
    {
        for x in hearth.0.saturating_sub(policy.forage_radius)
            ..=hearth
                .0
                .saturating_add(policy.forage_radius)
                .min(res.tile_grid.width.saturating_sub(1))
        {
            let tile = (x, y);
            if res.colony.zone_of_tile.contains_key(&tile)
                || res.colony.ground_items.contains_key(&tile)
                || res.colony.designations.contains_key(&tile)
                || res
                    .colony
                    .reservations
                    .contains_key(&ReserveKey::Cell(tile))
                || res
                    .colony
                    .reservations
                    .contains_key(&ReserveKey::Stack(tile))
                || !can_hold_items(res, tile)
                || !crate::commands::camp_reaches(res, tile)
                || res.steward.vetoed(&Target::Tile(x, y), res.current_tick)
            {
                continue;
            }
            candidates.push(tile);
        }
    }
    candidates.sort_by_key(|&tile| {
        (
            !zones.iter().any(|&zone| distance(tile, zone) == 1),
            distance(tile, hearth),
            tile.1,
            tile.0,
        )
    });
    candidates
        .into_iter()
        .take(usize::try_from(count).unwrap_or(usize::MAX))
        .map(|(x, y)| Command::AddStockpile {
            x0: x,
            y0: y,
            x1: x,
            y1: y,
        })
        .collect()
}
