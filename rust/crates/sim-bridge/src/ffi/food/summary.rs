//! Food alert observations use the same summary as the steward, with no UI policy copy.
use sim_core::colony::steward::{rules, Action};
use sim_engine::SimResources;

/// One colony-wide supply observation; colony fission is disabled in this pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FoodSummaryView {
    /// Actual stored edible satiety units.
    pub stored: u64,
    /// Daily demand in thousandths of units.
    pub daily_milli: u64,
    /// Warm-adult owner winter reserve bar.
    pub winter_need: u64,
    /// Days until winter, zero during winter.
    pub winter_in: u32,
    /// Authored preparation window is active.
    pub preparing: bool,
    /// Stored edible units expiring within two days.
    pub expiring: u64,
    /// Real camp tile for alert navigation.
    pub tile: (u32, u32),
}
/// No defaults are invented if the food planning rule is absent or invalid.
pub fn collect_food_summary(world: &hecs::World, res: &SimResources) -> Option<FoodSummaryView> {
    res.rules.colony?;
    let registry = rules().ok()?;
    let policy = registry.rules.iter().find_map(|rule| match &rule.action {
        Action::FoodPlan(policy) => Some(policy),
        _ => None,
    })?;
    let facts = sim_systems::runtime::steward::food::summary(world, res, policy);
    let tile = res
        .settlements
        .iter()
        .min_by_key(|(id, _)| *id)?
        .1
        .formation_tile;
    Some(FoodSummaryView {
        stored: facts.stored_units,
        daily_milli: facts.daily_milli,
        winter_need: sim_systems::runtime::steward::food::winter_need(facts.adults),
        winter_in: facts.winter_in,
        preparing: facts.preparing,
        expiring: facts.expiring_units,
        tile,
    })
}
