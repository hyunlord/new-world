//! Steward state and rule schema: content references are data, actions are mechanics.
mod food_plan;
mod rules;
pub use food_plan::FoodPlan;
mod state;
pub use rules::{parse_rules, rules, Action, Condition, Rule, RuleFile};
pub use state::{DefenceState, OwnedOrder, StewardScope, StewardState, Target};
