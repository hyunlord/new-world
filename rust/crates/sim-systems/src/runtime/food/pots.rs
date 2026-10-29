//! Real pot vessels with finite contents and atomic deployment/removal.
use super::items::{FoodBatch, FoodError};
use hecs::World;
use sim_core::colony::food::{FoodAge, FoodOwner, FoodStackKey, PotStore};
use sim_engine::SimResources;

/// Deploy one actual vessel from ground or hands without creating another item.
pub fn deploy_pot(
    world: &mut World,
    res: &mut SimResources,
    from: FoodOwner,
    tile: (u32, u32),
    day: u32,
) -> Result<u32, FoodError> {
    if !matches!(from, FoodOwner::Ground(_) | FoodOwner::Carried(_))
        || !crate::runtime::jobs::items::can_hold_items(res, tile)
    {
        return Err(FoodError::Invalid);
    }
    let stack = super::items::stack(world, res, from).ok_or(FoodError::MissingOwner)?;
    if stack.kind.definition().pot_capacity == 0 {
        return Err(FoodError::Invalid);
    }
    let id = res.food.next_pot_id.max(1);
    let next = id.checked_add(1).ok_or(FoodError::NoCapacity)?;
    if res.food.pots.contains_key(&id) {
        return Err(FoodError::Invalid);
    }
    let vessel = super::items::take(world, res, from, 1, day)?;
    res.food.pots.insert(
        id,
        PotStore {
            tile,
            vessel: vessel.spec,
            contents: Vec::new(),
        },
    );
    res.food.next_pot_id = next;
    res.food.touch();
    Ok(id)
}

/// Return the real vessel and all fresh contents to ground; failed space checks keep it deployed.
pub fn remove_pot(
    world: &mut World,
    res: &mut SimResources,
    id: u32,
    day: u32,
) -> Result<(), FoodError> {
    let pot = res.food.pots.get(&id).ok_or(FoodError::MissingOwner)?;
    let tile = pot.tile;
    let mut batches = vec![FoodBatch {
        spec: pot.vessel,
        count: 1,
        age: FoodAge { born_day: day },
    }];
    let mut expired = Vec::new();
    for stack in &pot.contents {
        let owner = FoodOwner::Pot {
            id,
            key: FoodStackKey::from(stack.spec()),
        };
        let age = res
            .food
            .ages
            .get(&owner)
            .copied()
            .ok_or(FoodError::MissingAge)?;
        if super::storage::remaining_days(world, res, owner, day)? == 0 {
            expired.push(owner);
        } else {
            batches.push(FoodBatch {
                spec: stack.spec(),
                count: stack.count,
                age,
            });
        }
    }
    let plan = super::ground::plan(res, tile, &batches, day)?;
    // Admission used an immutable snapshot; no fallible operation follows placement.
    for owner in expired {
        super::storage::expire_owner(world, res, owner)?;
    }
    super::ground::apply(res, &plan);
    res.food.ages.retain(
        |owner, _| !matches!(owner, FoodOwner::Pot { id: owner_id, .. } if *owner_id == id),
    );
    res.food.pots.remove(&id);
    res.food.touch();
    Ok(())
}
