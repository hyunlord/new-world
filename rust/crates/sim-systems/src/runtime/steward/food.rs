//! Demand-limited food orders and observable physical winter stores.
mod cooking;
mod dependencies;
mod harvest;
mod observe;
mod storage;
mod summary;
use hecs::World;
pub use observe::observe;
use sim_core::colony::steward::FoodPlan;
use sim_core::colony::Command;
use sim_engine::SimResources;
pub use summary::{summary, winter_need, FoodSummary};

/// Plan normal replayable commands from one current food snapshot.
pub fn plan(world: &World, res: &SimResources, policy: &FoodPlan) -> Vec<Command> {
    let facts = summary(world, res, policy);
    let mut commands = storage::plan(res, policy, &facts);
    commands.extend(harvest::plan(world, res, policy, &facts));
    commands.extend(cooking::plan(world, res, policy, &facts));
    let reserved_pots = dependencies::reserved_cooking_pots(res, policy);
    if let Some((&tile, _)) = res.colony.ground_items.iter().find(|(tile, stack)| {
        stack.kind.definition().pot_capacity > 0
            && res.colony.zone_of_tile.contains_key(tile)
            && !reserved_pots.contains(*tile)
            && !res.food.pots.values().any(|pot| pot.tile == **tile)
            && !res.steward.vetoed(
                &sim_core::colony::steward::Target::Tile(tile.0, tile.1),
                res.current_tick,
            )
    }) {
        commands.push(Command::DeployFoodPot { tile });
    }
    commands
}

#[cfg(test)]
mod storage_tests;

#[cfg(test)]
mod cooking_bill_tests;
