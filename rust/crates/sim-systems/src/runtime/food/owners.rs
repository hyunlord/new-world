//! Physical owner reads, capacity admission and whole-stack mutations.
use super::items::{FoodBatch, FoodError};
use hecs::{Entity, World};
use sim_core::colony::food::{FoodAge, FoodOwner, FoodStackKey};
use sim_core::colony::{Carry, ItemSpec, ItemStack};
use sim_core::components::Agent;
use sim_engine::SimResources;

pub(super) fn entity(world: &World, id: u64) -> Option<Entity> {
    world
        .query::<&Agent>()
        .iter()
        .find_map(|(entity, agent)| (agent.id == id).then_some(entity))
}

pub(super) fn stack(world: &World, res: &SimResources, owner: FoodOwner) -> Option<ItemStack> {
    match owner {
        FoodOwner::Ground(tile) => res.colony.ground_items.get(&tile).copied(),
        FoodOwner::Carried(id) => {
            let entity = entity(world, id)?;
            let carry = world.get::<&Carry>(entity).ok()?;
            Some(ItemStack::of(carry.spec(), carry.count))
        }
        FoodOwner::Shelf { workshop, key } => res
            .colony
            .workshops
            .get(&workshop)?
            .shelf
            .iter()
            .find(|s| FoodStackKey::from(s.spec()) == key)
            .copied(),
        FoodOwner::Pot { id, key } => res
            .food
            .pots
            .get(&id)?
            .contents
            .iter()
            .find(|s| FoodStackKey::from(s.spec()) == key)
            .copied(),
    }
}

pub(super) fn age(
    res: &SimResources,
    owner: FoodOwner,
    spec: ItemSpec,
    day: u32,
) -> Result<FoodAge, FoodError> {
    let Some(food) = &spec.kind.definition().food else {
        return Ok(FoodAge { born_day: day });
    };
    let age = res
        .food
        .ages
        .get(&owner)
        .copied()
        .ok_or(FoodError::MissingAge)?;
    if age.born_day > day {
        return Err(FoodError::Invalid);
    }
    if age.remaining(day, food.lifetime_days(super::storage::class(res, owner))) == 0 {
        return Err(FoodError::Expired);
    }
    Ok(age)
}

pub(super) fn admit(
    world: &World,
    res: &SimResources,
    owner: FoodOwner,
    batch: FoodBatch,
    day: u32,
) -> Result<(), FoodError> {
    if batch.count == 0 || batch.age.born_day > day {
        return Err(FoodError::Invalid);
    }
    if let Some(food) = &batch.spec.kind.definition().food {
        if batch
            .age
            .remaining(day, food.lifetime_days(super::storage::class(res, owner)))
            == 0
        {
            return Err(FoodError::Expired);
        }
    }
    let current = stack(world, res, owner);
    if let Some(current) = current {
        if !current.spec().merges_with(&batch.spec) {
            return Err(FoodError::NoCapacity);
        }
        age(res, owner, current.spec(), day)?;
    }
    let present = current.map_or(0, |s| s.count);
    let room = match owner {
        FoodOwner::Ground(tile) => crate::runtime::jobs::items::room_on(res, tile, batch.spec),
        FoodOwner::Carried(id) => {
            entity(world, id).ok_or(FoodError::MissingOwner)?;
            crate::runtime::jobs::items::stack_max(res, batch.spec.kind).saturating_sub(present)
        }
        FoodOwner::Shelf { workshop, key } => {
            if !res.colony.workshops.contains_key(&workshop) {
                return Err(FoodError::MissingOwner);
            }
            if key != FoodStackKey::from(batch.spec) {
                return Err(FoodError::Invalid);
            }
            u16::MAX - present
        }
        FoodOwner::Pot { id, key } => {
            let pot = res.food.pots.get(&id).ok_or(FoodError::MissingOwner)?;
            if batch.spec.kind.definition().food.is_none() || key != FoodStackKey::from(batch.spec)
            {
                return Err(FoodError::Invalid);
            }
            let capacity = u32::from(pot.vessel.kind.definition().pot_capacity);
            let used: u32 = pot.contents.iter().map(|s| u32::from(s.count)).sum();
            u16::try_from(capacity.saturating_sub(used)).unwrap_or(0)
        }
    };
    if room < batch.count {
        return Err(FoodError::NoCapacity);
    }
    Ok(())
}

pub(super) fn set(
    world: &mut World,
    res: &mut SimResources,
    owner: FoodOwner,
    spec: ItemSpec,
    count: u16,
) -> Result<(), FoodError> {
    match owner {
        FoodOwner::Ground(tile) => {
            if count == 0 {
                res.colony.ground_items.remove(&tile);
            } else {
                res.colony
                    .ground_items
                    .insert(tile, ItemStack::of(spec, count));
            }
        }
        FoodOwner::Carried(id) => {
            let entity = entity(world, id).ok_or(FoodError::MissingOwner)?;
            if count == 0 {
                world
                    .remove_one::<Carry>(entity)
                    .map_err(|_| FoodError::MissingOwner)?;
            } else {
                world
                    .insert_one(entity, Carry::of(spec, count))
                    .map_err(|_| FoodError::MissingOwner)?;
            }
        }
        FoodOwner::Shelf { workshop, key } => {
            let shelf = &mut res
                .colony
                .workshops
                .get_mut(&workshop)
                .ok_or(FoodError::MissingOwner)?
                .shelf;
            set_contents(shelf, key, spec, count);
        }
        FoodOwner::Pot { id, key } => {
            let contents = &mut res
                .food
                .pots
                .get_mut(&id)
                .ok_or(FoodError::MissingOwner)?
                .contents;
            set_contents(contents, key, spec, count);
        }
    }
    res.colony.touch(false);
    Ok(())
}

fn set_contents(contents: &mut Vec<ItemStack>, key: FoodStackKey, spec: ItemSpec, count: u16) {
    if let Some(index) = contents
        .iter()
        .position(|s| FoodStackKey::from(s.spec()) == key)
    {
        if count == 0 {
            contents.remove(index);
        } else {
            contents[index] = ItemStack::of(spec, count);
        }
    } else if count > 0 {
        contents.push(ItemStack::of(spec, count));
    }
}
