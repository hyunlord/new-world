//! Atomic food ownership transitions; age never resets during a move.
pub use super::placement::{book_creation, create_ground, create_ground_batches, relocate_ground};
use hecs::World;
use sim_core::colony::food::{FoodAge, FoodOwner};
use sim_core::colony::ItemSpec;
use sim_engine::SimResources;
use std::fmt;

/// Detached whole items whose original age follows their next owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoodBatch {
    /// Merge identity and quality.
    pub spec: ItemSpec,
    /// Whole items in transit.
    pub count: u16,
    /// Earliest production day; irrelevant for nonfood items.
    pub age: FoodAge,
}

/// A failed operation leaves the physical source unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoodError {
    /// The owner or its item is absent.
    MissingOwner,
    /// A food stack has no birth record.
    MissingAge,
    /// The source is already expired.
    Expired,
    /// Destination identity differs or has insufficient capacity.
    NoCapacity,
    /// A zero count, impossible date or invalid operation was requested.
    Invalid,
    /// A required person component is absent.
    MissingPerson,
    /// The data forbids eating this item without preparation.
    Inedible,
}
impl fmt::Display for FoodError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "food ownership: {self:?}")
    }
}
impl std::error::Error for FoodError {}

/// Why whole items enter the physical world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoodSource {
    /// Explicit debug injection, accounted as initial supply.
    Debug,
    /// Raw plant or animal harvest.
    Harvest,
    /// Recipe output; the craft caller books ItemLedger::craft_end once.
    Craft,
    /// A caravan import.
    Trade,
    /// Explicit conversion of an old save's abstract reserve.
    Migration,
}

/// Construct a detached new batch; creation accounting happens on placement.
pub fn new_batch(spec: ItemSpec, count: u16, born_day: u32) -> Result<FoodBatch, FoodError> {
    if count == 0 {
        return Err(FoodError::Invalid);
    }
    Ok(FoodBatch {
        spec,
        count,
        age: FoodAge { born_day },
    })
}

/// Inspect a physical owner without allocating or mutating it.
pub fn stack(
    world: &World,
    res: &SimResources,
    owner: FoodOwner,
) -> Option<sim_core::colony::ItemStack> {
    super::owners::stack(world, res, owner)
}

/// Take at most `count` items; an expired or untracked food stack cannot move.
pub fn take(
    world: &mut World,
    res: &mut SimResources,
    owner: FoodOwner,
    count: u16,
    day: u32,
) -> Result<FoodBatch, FoodError> {
    let physical = stack(world, res, owner).ok_or(FoodError::MissingOwner)?;
    let age = super::owners::age(res, owner, physical.spec(), day)?;
    let count = count.min(physical.count);
    if count == 0 {
        return Err(FoodError::Invalid);
    }
    super::owners::set(world, res, owner, physical.spec(), physical.count - count)?;
    if physical.count == count {
        res.food.ages.remove(&owner);
    }
    res.food.touch();
    Ok(FoodBatch {
        spec: physical.spec(),
        count,
        age,
    })
}

/// Place the entire batch or return it untouched to the caller.
pub fn put(
    world: &mut World,
    res: &mut SimResources,
    owner: FoodOwner,
    batch: FoodBatch,
    day: u32,
) -> Result<(), (FoodError, FoodBatch)> {
    if let Err(error) = super::owners::admit(world, res, owner, batch, day) {
        return Err((error, batch));
    }
    let current = stack(world, res, owner);
    let count = current.map_or(0, |s| s.count) + batch.count;
    let mut merged = sim_core::colony::ItemStack::of(batch.spec, 0);
    if let Some(current) = current {
        merged = current;
    }
    merged.absorb(batch.spec, batch.count);
    if let Err(error) = super::owners::set(world, res, owner, merged.spec(), count) {
        return Err((error, batch));
    }
    if batch.spec.kind.definition().food.is_some() {
        res.food
            .ages
            .entry(owner)
            .and_modify(|age| *age = age.merged(batch.age))
            .or_insert(batch.age);
    }
    super::events::stored(res, owner, batch.spec.kind, batch.count);
    res.food.touch();
    Ok(())
}

/// Move whole items only after both ownership operations have been admitted.
pub fn transfer(
    world: &mut World,
    res: &mut SimResources,
    from: FoodOwner,
    to: FoodOwner,
    count: u16,
    day: u32,
) -> Result<u16, FoodError> {
    if from == to {
        return Err(FoodError::Invalid);
    }
    let physical = stack(world, res, from).ok_or(FoodError::MissingOwner)?;
    let age = super::owners::age(res, from, physical.spec(), day)?;
    let batch = new_batch(physical.spec(), count.min(physical.count), age.born_day)?;
    super::owners::admit(world, res, to, batch, day)?;
    // World is exclusively borrowed, so admission remains valid between these writes.
    let taken = take(world, res, from, batch.count, day)?;
    put(world, res, to, taken, day).map_err(|(error, _)| error)?;
    Ok(batch.count)
}

/// Destination tiles and whole-item counts produced by ground placement.
pub type GroundPlacements = Vec<((u32, u32), u16)>;

/// Place a complete batch in deterministic nearby rings, retaining it on failure.
pub fn put_ground(
    res: &mut SimResources,
    tile: (u32, u32),
    batch: FoodBatch,
    day: u32,
) -> Result<GroundPlacements, (FoodError, FoodBatch)> {
    super::ground::put(res, tile, batch, day)
}

/// Drop a physical owner atomically; already expired food is explicitly spoiled.
pub fn drop_ground(
    world: &mut World,
    res: &mut SimResources,
    from: FoodOwner,
    tile: (u32, u32),
    count: u16,
    day: u32,
) -> Result<u16, FoodError> {
    let physical = stack(world, res, from).ok_or(FoodError::MissingOwner)?;
    let age = match super::owners::age(res, from, physical.spec(), day) {
        Ok(age) => age,
        Err(FoodError::Expired) => return super::storage::expire_count(world, res, from, count),
        Err(error) => return Err(error),
    };
    let batch = new_batch(physical.spec(), count.min(physical.count), age.born_day)?;
    let plan = super::ground::plan(res, tile, &[batch], day)?;
    take(world, res, from, batch.count, day)?;
    super::ground::apply(res, &plan);
    Ok(batch.count)
}

/// Account for recipe inputs already removed atomically by their caller.
pub fn book_craft_input(res: &mut SimResources, batch: FoodBatch) {
    if let Some(food) = &batch.spec.kind.definition().food {
        res.food.tallies.craft_input_units += u64::from(food.units) * u64::from(batch.count);
    }
}
