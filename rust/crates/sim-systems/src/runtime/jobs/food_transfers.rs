//! Age-preserving food moves at job pickup, delivery and cancellation seams.
use hecs::{Entity, World};
use sim_core::colony::food::{FoodOwner, FoodStackKey};
use sim_core::colony::{Carry, ItemKind, ItemSpec, JobKind};
use sim_core::components::Agent;
use sim_engine::SimResources;

use crate::runtime::food::{items, storage};

fn day(res: &SimResources) -> u32 {
    u32::try_from(crate::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX)
}

/// Pick up food through the ownership seam; ordinary items retain the old path.
pub(super) fn pickup(
    world: &mut World,
    res: &mut SimResources,
    entity: Entity,
    from: (u32, u32),
    kind: ItemKind,
    count: u16,
) -> (u16, ItemSpec) {
    if kind.definition().food.is_none() {
        return super::items::take_items(res, from, kind, count);
    }
    let absent = (0, ItemSpec::raw(kind, None));
    // Only Supply jobs withdraw pot contents; other job sources stay ground-only.
    let supply = world
        .get::<&sim_core::colony::Job>(entity)
        .ok()
        .is_some_and(
            |job| matches!(job.kind, JobKind::Supply { from: source, .. } if source == from),
        );
    let source = if supply {
        super::pot_supply::source(res, from, kind)
    } else {
        res.colony
            .ground_items
            .get(&from)
            .copied()
            .filter(|stack| stack.kind == kind && stack.count > 0)
            .map(|stack| (FoodOwner::Ground(from), stack))
    };
    let Some((source, stack)) = source else {
        return absent;
    };
    let spec = stack.spec();
    let Some(id) = world.get::<&Agent>(entity).ok().map(|a| a.id) else {
        return absent;
    };
    let today = day(res);
    let count =
        items::transfer(world, res, source, FoodOwner::Carried(id), count, today).unwrap_or(0);
    (count, spec)
}

/// Drop food without changing its birth day; failure retains the complete carried stack.
pub(super) fn drop(world: &mut World, res: &mut SimResources, entity: Entity, tile: (u32, u32)) {
    let Some(id) = world.get::<&Agent>(entity).ok().map(|a| a.id) else {
        return;
    };
    let today = day(res);
    let _ = items::drop_ground(world, res, FoodOwner::Carried(id), tile, u16::MAX, today);
}

/// Retry a previously full destination before the carrier accepts unrelated work.
pub(super) fn retry_stranded(world: &mut World, res: &mut SimResources) {
    let mut loads: Vec<_> = world
        .query::<(&Agent, &sim_core::components::Position, &Carry)>()
        .without::<&sim_core::colony::Job>()
        .iter()
        .filter(|(_, (_, _, carry))| carry.kind.definition().food.is_some())
        .map(|(entity, (agent, position, _))| (agent.id, entity, (position.x, position.y)))
        .collect();
    loads.sort_unstable_by_key(|row| row.0);
    for (_, entity, tile) in loads {
        drop(world, res, entity, tile);
    }
}

/// Deliver a food load; physical ownership is retained on any failed destination admission.
pub(super) fn arrive(
    world: &mut World,
    res: &mut SimResources,
    entity: Entity,
    kind: JobKind,
    tile: (u32, u32),
    carry: Carry,
) {
    let Some(id) = world.get::<&Agent>(entity).ok().map(|a| a.id) else {
        return;
    };
    let owner = FoodOwner::Carried(id);
    let today = day(res);
    if storage::remaining_days(world, res, owner, today) == Ok(0) {
        let _ = storage::expire_owner(world, res, owner);
        return;
    }
    match kind {
        JobKind::Supply { workshop, .. } => {
            let target = FoodOwner::Shelf {
                workshop,
                key: FoodStackKey::from(carry.spec()),
            };
            if items::transfer(world, res, owner, target, carry.count, today).is_err() {
                drop(world, res, entity, tile);
            }
        }
        JobKind::Trade { .. } => {
            if let Ok(batch) = items::take(world, res, owner, carry.count, today) {
                res.colony
                    .item_ledger
                    .trade_out(batch.spec.kind, u64::from(batch.count));
                if let Some(food) = &batch.spec.kind.definition().food {
                    res.food.tallies.traded_out_units +=
                        u64::from(food.units) * u64::from(batch.count);
                }
                res.colony.neighbours.version += 1;
            }
        }
        _ => {
            if let Some(age) = res.food.ages.get(&owner).copied() {
                for (target, room) in
                    super::food_storage::clean_owners(res, tile, carry.spec(), age.born_day)
                {
                    let Some(remaining) = items::stack(world, res, owner) else {
                        break;
                    };
                    let count = remaining.count.min(room);
                    let _ = items::transfer(world, res, owner, target, count, today);
                }
            }
            let pots: Vec<_> = res
                .food
                .pots
                .iter()
                .filter(|(_, pot)| pot.tile == tile)
                .map(|(&id, _)| id)
                .collect();
            for pot in pots {
                let Some(remaining) = items::stack(world, res, owner) else {
                    break;
                };
                let Some(store) = res.food.pots.get(&pot) else {
                    continue;
                };
                let used: u32 = store
                    .contents
                    .iter()
                    .map(|stack| u32::from(stack.count))
                    .sum();
                let room =
                    u32::from(store.vessel.kind.definition().pot_capacity).saturating_sub(used);
                let count = remaining.count.min(u16::try_from(room).unwrap_or(u16::MAX));
                if count == 0 {
                    continue;
                }
                let target = FoodOwner::Pot {
                    id: pot,
                    key: FoodStackKey::from(remaining.spec()),
                };
                let _ = items::transfer(world, res, owner, target, count, today);
            }
            drop(world, res, entity, tile);
        }
    }
}

/// Real destination room for a food haul includes vessels deployed on that tile.
pub(super) fn haul_room(res: &SimResources, tile: (u32, u32), spec: ItemSpec) -> u16 {
    let ground = super::items::room_on(res, tile, spec);
    if spec.kind.definition().food.is_none() {
        return ground;
    }
    let pots: u32 = res
        .food
        .pots
        .values()
        .filter(|pot| pot.tile == tile)
        .map(|pot| {
            let used: u32 = pot
                .contents
                .iter()
                .map(|stack| u32::from(stack.count))
                .sum();
            u32::from(pot.vessel.kind.definition().pot_capacity).saturating_sub(used)
        })
        .sum();
    u16::try_from(u32::from(ground).saturating_add(pots)).unwrap_or(u16::MAX)
}
