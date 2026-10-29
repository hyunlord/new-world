//! Daily expiry and storage class derived from the stack's real physical owner.
use super::items::FoodError;
pub use super::pots::{deploy_pot, remove_pot};
use hecs::World;
use sim_core::colony::content::FoodStorageClass;
use sim_core::colony::food::FoodOwner;
use sim_core::rooms::RoomRole;
use sim_engine::SimResources;

/// Physical storage determines lifetime; carrying has the ground lifetime.
pub fn class(res: &SimResources, owner: FoodOwner) -> FoodStorageClass {
    let tile = match owner {
        FoodOwner::Pot { .. } => return FoodStorageClass::Pot,
        FoodOwner::Carried(_) => return FoodStorageClass::Ground,
        FoodOwner::Ground(tile) => tile,
        FoodOwner::Shelf { workshop, .. } => {
            let Some(workshop) = res.colony.workshops.get(&workshop) else {
                return FoodStorageClass::Ground;
            };
            workshop.work_tile()
        }
    };
    if res
        .colony
        .rooms
        .values()
        .any(|room| room.roofed && room.role == RoomRole::Storehouse && room.tiles.contains(&tile))
    {
        FoodStorageClass::RoofedStore
    } else if res
        .colony
        .zones
        .values()
        .any(|zone| zone.tiles.contains(&tile))
    {
        FoodStorageClass::Stockpile
    } else {
        FoodStorageClass::Ground
    }
}

/// Complete days remaining, requiring the original age rather than inventing freshness.
pub fn remaining_days(
    world: &World,
    res: &SimResources,
    owner: FoodOwner,
    day: u32,
) -> Result<u32, FoodError> {
    let stack = super::items::stack(world, res, owner).ok_or(FoodError::MissingOwner)?;
    let food = stack
        .kind
        .definition()
        .food
        .as_ref()
        .ok_or(FoodError::Inedible)?;
    let age = res.food.ages.get(&owner).ok_or(FoodError::MissingAge)?;
    if age.born_day > day {
        return Err(FoodError::Invalid);
    }
    Ok(age.remaining(day, food.lifetime_days(class(res, owner))))
}

/// Remove a measured count without a freshness guard, for expiry and legacy migration.
pub fn expire_count(
    world: &mut World,
    res: &mut SimResources,
    owner: FoodOwner,
    count: u16,
) -> Result<u16, FoodError> {
    let stack = super::items::stack(world, res, owner).ok_or(FoodError::MissingOwner)?;
    let food = stack
        .kind
        .definition()
        .food
        .as_ref()
        .ok_or(FoodError::Inedible)?;
    let count = count.min(stack.count);
    if count == 0 {
        return Err(FoodError::Invalid);
    }
    super::owners::set(world, res, owner, stack.spec(), stack.count - count)?;
    if count == stack.count {
        res.food.ages.remove(&owner);
    }
    res.colony.item_ledger.consume(stack.kind, u64::from(count));
    *res.food.tallies.spoiled.entry(stack.kind).or_default() += u64::from(count);
    res.food.tallies.spoiled_units += u64::from(food.units) * u64::from(count);
    res.food.touch();
    Ok(count)
}

/// Expire the whole physical stack, even when migration has no trustworthy age.
pub fn expire_owner(
    world: &mut World,
    res: &mut SimResources,
    owner: FoodOwner,
) -> Result<u16, FoodError> {
    expire_count(world, res, owner, u16::MAX)
}

/// Daily expiry visits tracked food owners once, in stable owner order.
pub fn spoil_day(world: &mut World, res: &mut SimResources, day: u32) -> Result<u64, FoodError> {
    if res.rules.colony.is_none() {
        return Ok(0);
    }
    let owners: Vec<_> = res.food.ages.keys().copied().collect();
    let mut pieces = 0;
    for owner in owners {
        if remaining_days(world, res, owner, day)? == 0 {
            pieces += u64::from(expire_owner(world, res, owner)?);
        }
    }
    Ok(pieces)
}
