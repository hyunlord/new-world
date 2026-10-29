//! Explicit food losses when a dying carrier cannot place a physical stack.
use hecs::{Entity, World};
use sim_core::colony::food::FoodOwner;
use sim_core::colony::Carry;
use sim_core::components::Agent;
use sim_engine::SimResources;

/// Record only food still carried after the ordinary death-drop attempt failed.
pub fn lose_carried(world: &mut World, res: &mut SimResources, entity: Entity) {
    let Some((id, carry)) = world
        .get::<&Agent>(entity)
        .ok()
        .map(|agent| agent.id)
        .zip(world.get::<&Carry>(entity).ok().map(|carry| *carry))
    else {
        return;
    };
    let Some(food) = &carry.kind.definition().food else {
        return;
    };
    if world.remove_one::<Carry>(entity).is_err() {
        return;
    }
    res.food.ages.remove(&FoodOwner::Carried(id));
    let count = u64::from(carry.count);
    res.colony.item_ledger.consume(carry.kind, count);
    *res.food.tallies.lost.entry(carry.kind).or_default() += count;
    res.food.tallies.lost_units += count * u64::from(food.units);
    res.food.touch();
}
