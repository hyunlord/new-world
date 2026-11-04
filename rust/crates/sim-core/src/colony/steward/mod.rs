//! Steward state and rule schema: content references are data, actions are mechanics.
mod food_plan;
mod rules;
pub use food_plan::{CultivationPlan, FoodPlan};
mod state;
pub use rules::{parse_rules, rules, Action, Condition, FuelPlan, Rule, RuleFile};
pub use state::{DefenceState, OwnedOrder, StewardScope, StewardState, Target};
