use crate::config;
use serde::{Deserialize, Serialize};

/// Per-agent steering parameters derived from HEXACO personality.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SteeringParams {
    /// Base movement speed in pixels per tick before conversion into tile units.
    pub base_speed: f64,
    /// Random per-tick speed variation as a fraction of the base speed.
    pub speed_variance: f64,
    /// Wander circle radius in pixels.
    pub wander_radius: f64,
    /// Wander projection distance in pixels.
    pub wander_distance: f64,
    /// Wander heading jitter per tick in degrees.
    pub wander_jitter: f64,
    /// Reduces wander influence while an agent has an explicit task target.
    pub wander_suppression: f64,
    /// Pull toward nearby agents.
    pub cohesion_weight: f64,
    /// Push away from nearby agents.
    pub separation_weight: f64,
    /// Included for future flock alignment; unused in W1-A.
    pub alignment_weight: f64,
    /// Bias toward approaching nearby agents.
    pub social_approach_weight: f64,
    /// Separation distance in pixels.
    pub personal_space_radius: f64,
    /// Preference for straight-line travel toward explicit targets.
    pub path_directness: f64,
    /// How far wander/search logic is willing to explore in pixels.
    pub exploration_radius: f64,
    /// Preference for novel destinations.
    pub destination_variety: f64,
    /// Future flee-scaling multiplier.
    pub flee_multiplier: f64,
    /// Mood-derived speed multiplier.
    pub mood_speed_multiplier: f64,
    /// Stress-derived speed multiplier.
    pub stress_speed_multiplier: f64,
}

impl Default for SteeringParams {
    fn default() -> Self {
        Self {
            base_speed: config::AGENT_BASE_SPEED,
            speed_variance: config::AGENT_SPEED_VARIANCE,
            wander_radius: 30.0,
            wander_distance: 40.0,
            wander_jitter: 15.0,
            wander_suppression: 0.3,
            cohesion_weight: 0.6,
            separation_weight: 1.0,
            alignment_weight: 0.7,
            social_approach_weight: 0.5,
            personal_space_radius: 25.0,
            path_directness: 0.6,
            exploration_radius: 150.0,
            destination_variety: 0.5,
            flee_multiplier: 1.0,
            mood_speed_multiplier: 1.0,
            stress_speed_multiplier: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SteeringParams;
    use crate::config;

    #[test]
    fn default_steering_params_match_ticket_baseline() {
        let params = SteeringParams::default();
        assert_eq!(params.base_speed, config::AGENT_BASE_SPEED);
        assert_eq!(params.speed_variance, config::AGENT_SPEED_VARIANCE);
        assert_eq!(params.wander_radius, 30.0);
        assert_eq!(params.personal_space_radius, 25.0);
        assert_eq!(params.mood_speed_multiplier, 1.0);
        assert_eq!(params.stress_speed_multiplier, 1.0);
    }

    #[test]
    fn stage1_steering_params_has_all_fields() {
        let params = SteeringParams::default();
        assert!(params.base_speed > 0.0);
        assert!(params.speed_variance > 0.0);
        assert!(params.wander_radius > 0.0);
        assert!(params.wander_distance > 0.0);
        assert!(params.wander_jitter > 0.0);
        assert!(params.cohesion_weight >= 0.0);
        assert!(params.separation_weight >= 0.0);
        assert!((0.0..=1.0).contains(&params.path_directness));
        assert!(params.exploration_radius > 0.0);
        assert!(params.flee_multiplier > 0.0);
        assert!(params.personal_space_radius > 0.0);
    }
}
