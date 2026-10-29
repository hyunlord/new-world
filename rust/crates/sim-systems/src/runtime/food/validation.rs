//! Save/gate validation of physical food ownership and the satiety conservation equation.
use hecs::World;
use sim_core::colony::food::{FoodOwner, FoodStackKey, FoodState};
use sim_core::colony::{Carry, ItemStack};
use sim_core::components::Agent;
use sim_engine::SimResources;
use std::collections::{BTreeMap, BTreeSet};

/// Validate each physical food owner, all metadata and the exact units balance.
pub fn validate(world: &World, res: &SimResources) -> Result<(), String> {
    if res.rules.colony.is_none() {
        return if res.food == FoodState::default() {
            Ok(())
        } else {
            Err("baseline contains colony food state".into())
        };
    }
    for (&index, evidence) in &res.food.winter_plans {
        let entry = res
            .chronicle
            .entries()
            .get(index as usize)
            .ok_or_else(|| format!("winter plan evidence missing entry {index}"))?;
        if entry.kind != sim_core::chronicle::ChronicleKind::WinterFoodPlan
            || entry.tick != evidence.tick
            || entry.value != i64::try_from(evidence.stored_units).unwrap_or(i64::MAX)
            || entry.cause != u32::try_from(evidence.required_units).unwrap_or(u32::MAX)
        {
            return Err(format!("winter plan evidence mismatch at {index}"));
        }
    }
    let mut physical = BTreeMap::new();
    for (&tile, &stack) in &res.colony.ground_items {
        insert(&mut physical, FoodOwner::Ground(tile), stack)?;
    }
    for (_, (carry, agent)) in world.query::<(&Carry, Option<&Agent>)>().iter() {
        if carry.kind.definition().food.is_none() {
            continue;
        }
        let agent = agent.ok_or_else(|| "food carried without a stable agent owner".to_owned())?;
        insert(
            &mut physical,
            FoodOwner::Carried(agent.id),
            ItemStack::of(carry.spec(), carry.count),
        )?;
    }
    for (&workshop, shop) in &res.colony.workshops {
        for &stack in &shop.shelf {
            insert(
                &mut physical,
                FoodOwner::Shelf {
                    workshop,
                    key: FoodStackKey::from(stack.spec()),
                },
                stack,
            )?;
        }
    }
    for (&id, pot) in &res.food.pots {
        let capacity = pot.vessel.kind.definition().pot_capacity;
        let count: u32 = pot.contents.iter().map(|s| u32::from(s.count)).sum();
        if capacity == 0 || count > u32::from(capacity) || id == 0 || id >= res.food.next_pot_id {
            return Err(format!("invalid physical pot {id}: {count}/{capacity}"));
        }
        for &stack in &pot.contents {
            if stack.kind.definition().food.is_none() {
                return Err(format!("nonfood in pot {id}"));
            }
            insert(
                &mut physical,
                FoodOwner::Pot {
                    id,
                    key: FoodStackKey::from(stack.spec()),
                },
                stack,
            )?;
        }
    }
    let owners: BTreeSet<_> = physical.keys().copied().collect();
    let ages: BTreeSet<_> = res.food.ages.keys().copied().collect();
    if owners != ages {
        return Err(format!(
            "food owner/age mismatch: missing {:?}, orphan {:?}",
            owners.difference(&ages).collect::<Vec<_>>(),
            ages.difference(&owners).collect::<Vec<_>>()
        ));
    }
    for batch in &res.food.pending_drying {
        if batch.count == 0 || batch.output.kind.definition().food.is_none() {
            return Err("invalid pending drying output".into());
        }
    }
    let day = super::day(res);
    for (owner, age) in &res.food.ages {
        if age.born_day > day {
            return Err(format!(
                "future food age {owner:?}: {} > {day}",
                age.born_day
            ));
        }
    }
    let mut present = 0u128;
    for stack in physical.values() {
        if let Some(food) = &stack.kind.definition().food {
            present += u128::from(food.units) * u128::from(stack.count);
        }
    }
    let t = &res.food.tallies;
    let incoming = u128::from(t.initial_units)
        + u128::from(t.harvested_units)
        + u128::from(t.craft_output_units)
        + u128::from(t.traded_in_units);
    let outgoing = present
        + u128::from(t.eaten_units)
        + u128::from(t.spoiled_units)
        + u128::from(t.lost_units)
        + u128::from(t.craft_input_units)
        + u128::from(t.traded_out_units);
    if incoming != outgoing {
        return Err(format!("food units ledger: incoming {incoming}, outgoing+present {outgoing}, present {present}"));
    }
    for (&id, meals) in &res.food.meals {
        if meals.last_meals.len() > 8
            || meals
                .last_meals
                .windows(2)
                .any(|pair| pair[0].tick > pair[1].tick)
            || meals
                .last_meals
                .iter()
                .any(|meal| meal.tick > res.current_tick || meal.pieces == 0)
        {
            return Err(format!("invalid bounded meal history for {id}"));
        }
    }
    Ok(())
}

fn insert(
    physical: &mut BTreeMap<FoodOwner, ItemStack>,
    owner: FoodOwner,
    stack: ItemStack,
) -> Result<(), String> {
    if stack.kind.definition().food.is_none() {
        return Ok(());
    }
    if stack.count == 0 {
        return Err(format!("empty food owner {owner:?}"));
    }
    if physical.insert(owner, stack).is_some() {
        return Err(format!("duplicate food owner {owner:?}"));
    }
    Ok(())
}
