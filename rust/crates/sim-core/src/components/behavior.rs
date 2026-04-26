// TODO(v3.1): Convert remaining f32 fields to f64 for determinism.
use crate::enums::ActionType;
use crate::ids::EntityId;
use serde::{Deserialize, Serialize};

fn default_one() -> f64 {
    1.0
}

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
    /// Recipe ID being crafted for the current craft action.
    #[serde(default)]
    pub craft_recipe_id: Option<String>,
    /// Primary material selected for the current craft action.
    #[serde(default)]
    pub craft_material_id: Option<String>,
    /// Band center X coordinate used by steering cohesion.
    #[serde(default)]
    pub band_center_x: Option<f64>,
    /// Band center Y coordinate used by steering cohesion.
    #[serde(default)]
    pub band_center_y: Option<f64>,
    /// Multiplier applied when separating from non-band outsiders.
    #[serde(default = "default_one")]
    pub outsider_separation_mult: f64,
    /// Tick when Mourn last completed near a cairn. Used to enforce MOURN_COOLDOWN_TICKS.
    #[serde(default)]
    pub mourn_last_tick: Option<u64>,
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
            job: "none".to_string(),
            job_satisfaction: 0.5,
            occupation: String::new(),
            occupation_satisfaction: 0.5,
            craft_recipe_id: None,
            craft_material_id: None,
            band_center_x: None,
            band_center_y: None,
            outsider_separation_mult: 1.0,
            mourn_last_tick: None,
        }
    }
}
