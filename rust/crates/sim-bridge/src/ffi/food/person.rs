//! Successful meal history and current hunger for one selected live person.
use sim_core::colony::food::MealRecord;
use sim_core::components::{Agent, Hunger};
use sim_engine::SimEngine;

/// Read-only person food detail, independent of the original nine-key detail getter.
#[derive(Debug, Clone, PartialEq)]
pub struct PersonFoodView {
    /// Stable agent identity.
    pub agent: u64,
    /// Current hunger points.
    pub hunger: f32,
    /// Last completed meals, newest first, at most eight.
    pub meals: Vec<MealRecord>,
    /// Actual ingredient groups eaten in the trailing three calendar days.
    pub groups: Vec<String>,
}
/// Missing entities or baseline worlds have no colony food detail.
pub fn collect_person_food(engine: &SimEngine, bits: u64) -> Option<PersonFoodView> {
    engine.resources.rules.colony?;
    let entity = hecs::Entity::from_bits(bits)?;
    let agent = engine.world.get::<&Agent>(entity).ok()?;
    let hunger = engine.world.get::<&Hunger>(entity).ok()?.value;
    let mut row = PersonFoodView {
        agent: agent.id,
        hunger,
        meals: Vec::new(),
        groups: Vec::new(),
    };
    if let Some(history) = engine.resources.food.meals.get(&agent.id) {
        let now = engine.resources.current_tick;
        let window = engine.resources.rules.calendar.days_to_ticks(3);
        row.meals = history.last_meals.iter().rev().cloned().collect();
        row.groups = history
            .last_group_tick
            .iter()
            .filter(|(_, tick)| **tick <= now && now - **tick < window)
            .map(|(group, _)| group.clone())
            .collect();
    }
    Some(row)
}
