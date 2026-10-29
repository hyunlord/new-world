//! FoodDailySystem: day-boundary plant opportunities, physical spoilage and bounded meal history.
//!
//! Priority: 119. Interval: calendar day, at its first tick. Colony only.
//! Reads: calendar, plant and food data, physical storage owners.
//! Writes: FoodState plants/ages/meals/tallies, physical items and item ledger.
//! Events: exact food expiry accounting; narrative consumers read the resulting tallies.
pub mod choice;
pub mod drying;
pub mod events;
mod ground;
pub mod items;
pub mod loss;
pub mod meals;
pub mod migration;
mod owners;
mod placement;
pub mod plants;
mod pots;
pub mod reserves;
pub mod storage;
mod validation;
use hecs::World;
use sim_engine::{RuntimeSystem, SimResources};
pub use validation::validate;

/// Current zero-based simulation calendar day.
pub fn day(res: &SimResources) -> u32 {
    u32::try_from(crate::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX)
}

/// Calendar-bound daily food maintenance with no per-tick plant or stack scan.
pub struct FoodDailySystem {
    interval: u64,
}
impl FoodDailySystem {
    /// Match this world's calendar; no per-system evolving state is persisted.
    pub fn new(res: &SimResources) -> Self {
        Self {
            interval: res.ticks_per_day.max(1),
        }
    }
}
impl RuntimeSystem for FoodDailySystem {
    fn name(&self) -> &str {
        "FoodDailySystem"
    }
    fn priority(&self) -> u32 {
        119
    }
    fn tick_interval(&self) -> u64 {
        self.interval
    }
    fn tick_offset(&self, res: &SimResources) -> u64 {
        (self.interval - res.rules.clock_offset_ticks % self.interval) % self.interval
    }
    fn tick(&mut self, world: &mut World, res: &mut SimResources) {
        if res.rules.colony.is_none() {
            return;
        }
        let day = day(res);
        if res.food.last_daily_day == Some(day) {
            return;
        }
        if let Err(error) = storage::spoil_day(world, res, day) {
            panic!("food ownership invariant failed during daily expiry: {error}");
        }
        if let Err(error) = drying::advance(res) {
            panic!("food ownership invariant failed during drying: {error}");
        }
        let previous_day = res.food.last_daily_day;
        plants::advance_day(res, day, previous_day);
        crate::runtime::steward::food::observe(world, res);
        let window = res.rules.calendar.days_to_ticks(3);
        for meals in res.food.meals.values_mut() {
            meals.prune(res.current_tick, window);
        }
        res.food.last_daily_day = Some(day);
        res.food.touch();
    }
}
