//! Attach conservative old meat dates and explicitly expire untraceable pieces.
use hecs::World;
use sim_core::colony::food::{FoodAge, FoodOwner, FoodStackKey};
use sim_core::colony::{content, Carry, ItemStack};
use sim_core::components::Agent;
use sim_engine::SimResources;
use std::collections::{BTreeMap, VecDeque};

pub(super) fn attach(world: &mut World, res: &mut SimResources) -> Result<(), String> {
    let owners = collect(world, res)?;
    let meat = content::installed_or_bundled()
        .item_role("grazer_meat")
        .ok_or("legacy meat role is absent")?;
    let mut lots = res.colony.wildlife.meat_lots.clone();
    lots.sort_by_key(|(day, _)| *day);
    let day = super::super::day(res);
    if lots.iter().any(|(born, _)| *born > day) {
        return Err("legacy meat lot has a future birth date".into());
    }
    let mut lots: VecDeque<_> = lots.into_iter().filter(|(_, count)| *count > 0).collect();
    let recorded: u64 = lots.iter().map(|(_, count)| u64::from(*count)).sum();
    let present: u64 = owners
        .values()
        .filter(|stack| stack.kind == meat)
        .map(|stack| u64::from(stack.count))
        .sum();
    if recorded > present {
        return Err(format!(
            "legacy meat lots claim {recorded} pieces but only {present} exist"
        ));
    }
    for (owner, stack) in owners {
        let units = stack
            .kind
            .definition()
            .food
            .as_ref()
            .map_or(0, |food| food.units);
        let initial = u64::from(units) * u64::from(stack.count);
        res.food.tallies.initial_units = res
            .food
            .tallies
            .initial_units
            .checked_add(initial)
            .ok_or("legacy food opening units overflow")?;
        let (known, oldest) = if stack.kind == meat {
            assign(&mut lots, stack.count)
        } else {
            (0, None)
        };
        if let Some(born_day) = oldest {
            res.food.ages.insert(owner, FoodAge { born_day });
        }
        let unmatched = stack.count - known;
        if unmatched > 0 {
            super::super::storage::expire_count(world, res, owner, unmatched)
                .map_err(|error| format!("legacy unknown-age expiry: {error}"))?;
        }
    }
    res.colony.wildlife.meat_lots.clear();
    super::super::storage::spoil_day(world, res, day)
        .map_err(|error| format!("legacy known-age expiry: {error}"))?;
    Ok(())
}

fn assign(lots: &mut VecDeque<(u32, u32)>, count: u16) -> (u16, Option<u32>) {
    let mut left = u32::from(count);
    let mut oldest = None;
    while left > 0 {
        let Some((born, available)) = lots.front_mut() else {
            break;
        };
        if oldest.is_none() {
            oldest = Some(*born);
        }
        let taken = left.min(*available);
        left -= taken;
        *available -= taken;
        if *available == 0 {
            lots.pop_front();
        }
    }
    // `left` never exceeds the original u16 count.
    let known = u16::try_from(u32::from(count) - left).unwrap_or(count);
    (known, oldest)
}

fn collect(world: &World, res: &SimResources) -> Result<BTreeMap<FoodOwner, ItemStack>, String> {
    let mut owners = BTreeMap::new();
    for (tile, stack) in &res.colony.ground_items {
        if stack.kind.definition().food.is_some() {
            owners.insert(FoodOwner::Ground(*tile), *stack);
        }
    }
    for (entity, carry) in world.query::<&Carry>().iter() {
        if carry.kind.definition().food.is_none() {
            continue;
        }
        let agent = world
            .get::<&Agent>(entity)
            .map_err(|_| "legacy food Carry lacks an Agent identity")?;
        let owner = FoodOwner::Carried(agent.id);
        if owners
            .insert(owner, ItemStack::of(carry.spec(), carry.count))
            .is_some()
        {
            return Err("duplicate legacy carried food identity".into());
        }
    }
    for (workshop, value) in &res.colony.workshops {
        for stack in &value.shelf {
            if stack.kind.definition().food.is_none() {
                continue;
            }
            let owner = FoodOwner::Shelf {
                workshop: *workshop,
                key: FoodStackKey::from(stack.spec()),
            };
            if owners.insert(owner, *stack).is_some() {
                return Err("duplicate legacy food shelf merge key".into());
            }
        }
    }
    Ok(owners)
}
