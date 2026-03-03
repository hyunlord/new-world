use serde::{Deserialize, Serialize};
use crate::enums::ActionType;
use crate::ids::EntityId;

/// Current behavior / action state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Behavior {
    pub current_action: ActionType,
    pub action_target_entity: Option<EntityId>,
    pub action_target_x: Option<i32>,
    pub action_target_y: Option<i32>,
    /// Progress toward completing the current action (0.0..=1.0)
    pub action_progress: f64,
    /// Action timer (counts up; action completes when >= action_duration)
    pub action_timer: i32,
    /// Total ticks needed for current action
    pub action_duration: i32,
    /// Carry amount (food/resources currently held)
    pub carry: f32,
    /// Current job assignment
    pub job: String,
    /// Job satisfaction (0.0..=1.0)
    pub job_satisfaction: f32,
    /// Occupation title
    pub occupation: String,
    pub occupation_satisfaction: f32,
}

impl Default for Behavior {
    fn default() -> Self {
        Self {
            current_action: ActionType::Idle,
            action_target_entity: None,
            action_target_x: None,
            action_target_y: None,
            action_progress: 0.0,
            action_timer: 0,
            action_duration: 0,
            carry: 0.0,
            job: "gatherer".to_string(),
            job_satisfaction: 0.5,
            occupation: String::new(),
            occupation_satisfaction: 0.5,
        }
    }
}
