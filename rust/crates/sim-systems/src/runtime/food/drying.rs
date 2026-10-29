//! Complete passive preservation without resetting a due output's age on placement retries.
use super::items::{self, FoodError, FoodSource};
use sim_core::colony::food::{DryingBatch, FoodOwner};
use sim_engine::SimResources;

/// Finish due batches in insertion order; blocked outputs age from their completion day.
pub fn advance(res: &mut SimResources) -> Result<(), FoodError> {
    let day = super::day(res);
    let mut index = 0;
    while index < res.food.pending_drying.len() {
        let batch = res.food.pending_drying[index];
        if batch.due_tick > res.current_tick {
            index += 1;
            continue;
        }
        let born_day = u32::try_from(crate::runtime::resource_regen::day_index_at(
            res,
            batch.due_tick,
        ))
        .map_err(|_| FoodError::Invalid)?;
        let output = items::new_batch(batch.output, batch.count, born_day)?;
        let food = batch
            .output
            .kind
            .definition()
            .food
            .as_ref()
            .ok_or(FoodError::Inedible)?;
        if !res.colony.workshops.contains_key(&batch.workshop) {
            // No completion evidence survives a removed workshop: inputs were
            // already consumed at craft start, so retain cancellation accounting.
            res.colony.tallies.crafts_lost += 1;
        } else if output.age.remaining(
            day,
            food.lifetime_days(super::storage::class(res, FoodOwner::Ground(batch.tile))),
        ) == 0
        {
            discard_expired(res, batch, output);
        } else {
            match items::create_ground_batches(res, batch.tile, &[output], day, FoodSource::Craft) {
                Ok(()) => res
                    .colony
                    .item_ledger
                    .craft_end(&[(batch.output.kind, batch.count)]),
                Err(FoodError::NoCapacity) => {
                    index += 1;
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
        res.food.pending_drying.remove(index);
        res.food.touch();
    }
    Ok(())
}

/// Completed output that cannot survive is recorded as production and an explicit sink.
fn discard_expired(res: &mut SimResources, batch: DryingBatch, output: items::FoodBatch) {
    items::book_creation(res, output, FoodSource::Craft);
    res.colony
        .item_ledger
        .craft_end(&[(batch.output.kind, batch.count)]);
    let count = u64::from(batch.count);
    res.colony.item_ledger.consume(batch.output.kind, count);
    let units = batch
        .output
        .kind
        .definition()
        .food
        .as_ref()
        .map_or(0, |food| count * u64::from(food.units));
    *res.food
        .tallies
        .spoiled
        .entry(batch.output.kind)
        .or_default() += count;
    res.food.tallies.spoiled_units += units;
}
