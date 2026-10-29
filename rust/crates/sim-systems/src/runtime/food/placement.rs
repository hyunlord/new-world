//! Atomic placement and accounting for newly created or displaced ground stacks.
use super::items::{new_batch, put_ground, stack, take, FoodBatch, FoodError, FoodSource};
use hecs::World;
use sim_core::colony::{food::FoodOwner, ItemSpec};
use sim_engine::SimResources;

/// Move a set of blocked ground owners only if every destination fits.
/// Callers block the source footprint before planning, and undo that block on error.
pub fn relocate_ground(
    world: &mut World,
    res: &mut SimResources,
    tiles: &[(u32, u32)],
    destination: (u32, u32),
    day: u32,
) -> Result<(), FoodError> {
    if tiles.is_empty() {
        return Ok(());
    }
    let mut batches = Vec::with_capacity(tiles.len());
    for &tile in tiles {
        let owner = FoodOwner::Ground(tile);
        let physical = stack(world, res, owner).ok_or(FoodError::MissingOwner)?;
        let age = super::owners::age(res, owner, physical.spec(), day)?;
        batches.push(new_batch(physical.spec(), physical.count, age.born_day)?);
    }
    let plan = super::ground::plan(res, destination, &batches, day)?;
    for (&tile, batch) in tiles.iter().zip(&batches) {
        take(world, res, FoodOwner::Ground(tile), batch.count, day)?;
    }
    super::ground::apply(res, &plan);
    Ok(())
}

/// Create a harvested, crafted, imported or migrated batch only when all fits.
pub fn create_ground(
    res: &mut SimResources,
    tile: (u32, u32),
    spec: ItemSpec,
    count: u16,
    day: u32,
    source: FoodSource,
) -> Result<(), FoodError> {
    let batch = new_batch(spec, count, day)?;
    put_ground(res, tile, batch, day).map_err(|(error, _)| error)?;
    book_creation(res, batch, source);
    Ok(())
}

/// Admit several newly created stacks together, then place and account for all of them.
pub fn create_ground_batches(
    res: &mut SimResources,
    tile: (u32, u32),
    batches: &[FoodBatch],
    day: u32,
    source: FoodSource,
) -> Result<(), FoodError> {
    let plan = super::ground::plan(res, tile, batches, day)?;
    super::ground::apply(res, &plan);
    for &batch in batches {
        book_creation(res, batch, source);
    }
    Ok(())
}

/// Book a newly placed batch exactly once; transfers never call this.
pub fn book_creation(res: &mut SimResources, batch: FoodBatch, source: FoodSource) {
    let n = u64::from(batch.count);
    let units = batch
        .spec
        .kind
        .definition()
        .food
        .as_ref()
        .map_or(0, |f| u64::from(f.units) * n);
    let tally = &mut res.food.tallies;
    match source {
        FoodSource::Debug => {
            res.colony.item_ledger.create(batch.spec.kind, n);
            tally.initial_units += units;
        }
        FoodSource::Harvest => {
            res.colony.item_ledger.create(batch.spec.kind, n);
            if units > 0 {
                *tally.harvested.entry(batch.spec.kind).or_default() += n;
            }
            tally.harvested_units += units;
        }
        FoodSource::Craft => tally.craft_output_units += units,
        FoodSource::Trade => {
            res.colony.item_ledger.trade_in(batch.spec.kind, n);
            tally.traded_in_units += units;
        }
        FoodSource::Migration => {
            res.colony.item_ledger.create(batch.spec.kind, n);
            *tally.migrated.entry(batch.spec.kind).or_default() += n;
            tally.initial_units += units;
        }
    }
}
