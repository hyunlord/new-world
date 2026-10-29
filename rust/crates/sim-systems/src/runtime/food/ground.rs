//! All-or-nothing ground placement with deterministic same-region spill order.
use super::items::{FoodBatch, FoodError};
use sim_core::colony::food::FoodOwner;
use sim_core::colony::ItemStack;
use sim_engine::SimResources;
use std::collections::BTreeMap;

type Tile = (u32, u32);

pub(super) fn plan(
    res: &SimResources,
    tile: Tile,
    batches: &[FoodBatch],
    day: u32,
) -> Result<Vec<(Tile, FoodBatch)>, FoodError> {
    let mut planned = Vec::new();
    let mut occupied: BTreeMap<Tile, ItemStack> = BTreeMap::new();
    let region = (res.nav.is_built() && res.nav.passable(tile))
        .then(|| res.nav.region_of(tile))
        .filter(|r| *r != 0);
    for batch in batches {
        if batch.count == 0 || batch.age.born_day > day {
            return Err(FoodError::Invalid);
        }
        let mut left = batch.count;
        for restrict in [region, None]
            .into_iter()
            .take(if region.is_some() { 2 } else { 1 })
        {
            for ring in 0..=res.tile_grid.width.max(res.tile_grid.height) {
                let (cx, cy, r) = (i64::from(tile.0), i64::from(tile.1), i64::from(ring));
                for y in (cy - r)..=(cy + r) {
                    for x in (cx - r)..=(cx + r) {
                        if left == 0 {
                            break;
                        }
                        if (x - cx).abs().max((y - cy).abs()) != r {
                            continue;
                        }
                        let (Ok(x), Ok(y)) = (u32::try_from(x), u32::try_from(y)) else {
                            continue;
                        };
                        let at = (x, y);
                        if x >= res.tile_grid.width
                            || y >= res.tile_grid.height
                            || !crate::runtime::jobs::items::can_hold_items(res, at)
                            || restrict.is_some_and(|r| res.nav.region_of(at) != r)
                        {
                            continue;
                        }
                        if let Some(food) = &batch.spec.kind.definition().food {
                            if batch.age.remaining(
                                day,
                                food.lifetime_days(super::storage::class(
                                    res,
                                    FoodOwner::Ground(at),
                                )),
                            ) == 0
                            {
                                continue;
                            }
                        }
                        let current = occupied
                            .get(&at)
                            .or_else(|| res.colony.ground_items.get(&at))
                            .copied();
                        if current.is_some_and(|s| !s.spec().merges_with(&batch.spec)) {
                            continue;
                        }
                        if let Some(existing) = res.colony.ground_items.get(&at) {
                            super::owners::age(res, FoodOwner::Ground(at), existing.spec(), day)?;
                        }
                        let max = crate::runtime::jobs::items::stack_max(res, batch.spec.kind);
                        let count = max.saturating_sub(current.map_or(0, |s| s.count)).min(left);
                        if count == 0 {
                            continue;
                        }
                        let entry = occupied.entry(at).or_insert_with(|| {
                            current.unwrap_or_else(|| ItemStack::of(batch.spec, 0))
                        });
                        entry.absorb(batch.spec, count);
                        planned.push((at, FoodBatch { count, ..*batch }));
                        left -= count;
                    }
                }
                if left == 0 {
                    break;
                }
            }
            if left == 0 {
                break;
            }
        }
        if left > 0 {
            return Err(FoodError::NoCapacity);
        }
    }
    Ok(planned)
}

pub(super) fn apply(res: &mut SimResources, plan: &[(Tile, FoodBatch)]) {
    for &(tile, batch) in plan {
        res.colony
            .ground_items
            .entry(tile)
            .or_insert_with(|| ItemStack::of(batch.spec, 0))
            .absorb(batch.spec, batch.count);
        if batch.spec.kind.definition().food.is_some() {
            res.food
                .ages
                .entry(FoodOwner::Ground(tile))
                .and_modify(|age| *age = age.merged(batch.age))
                .or_insert(batch.age);
        }
        super::events::stored(res, FoodOwner::Ground(tile), batch.spec.kind, batch.count);
    }
    res.colony.touch(false);
    res.food.touch();
}

pub(super) fn put(
    res: &mut SimResources,
    tile: Tile,
    batch: FoodBatch,
    day: u32,
) -> Result<Vec<(Tile, u16)>, (FoodError, FoodBatch)> {
    let plan = plan(res, tile, &[batch], day).map_err(|e| (e, batch))?;
    apply(res, &plan);
    Ok(plan
        .into_iter()
        .map(|(tile, batch)| (tile, batch.count))
        .collect())
}
