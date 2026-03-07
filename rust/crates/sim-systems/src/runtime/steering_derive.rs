use sim_core::components::{Personality, SteeringParams};

/// Derives steering parameters from HEXACO axis values.
pub fn derive_steering_params(personality: &Personality) -> SteeringParams {
    let h = personality.axes[0];
    let e = personality.axes[1];
    let x = personality.axes[2];
    let a = personality.axes[3];
    let c = personality.axes[4];
    let o = personality.axes[5];

    SteeringParams {
        base_speed: lerp(40.0, 80.0, x) + lerp(-5.0, 10.0, c),
        speed_variance: lerp(0.02, 0.20, e),
        wander_radius: lerp(15.0, 50.0, x),
        wander_distance: lerp(20.0, 80.0, o),
        wander_jitter: lerp(5.0, 30.0, x) + lerp(0.0, 15.0, e),
        wander_suppression: lerp(0.0, 0.8, c),
        cohesion_weight: lerp(0.2, 1.2, a),
        separation_weight: lerp(1.5, 0.5, a),
        alignment_weight: lerp(0.3, 1.2, a),
        social_approach_weight: lerp(0.1, 1.0, x),
        personal_space_radius: lerp(15.0, 40.0, h),
        path_directness: lerp(0.3, 0.95, c),
        exploration_radius: lerp(50.0, 400.0, o),
        destination_variety: lerp(0.2, 1.0, o),
        flee_multiplier: lerp(0.5, 2.0, e),
        mood_speed_multiplier: 1.0,
        stress_speed_multiplier: 1.0,
    }
}

fn lerp(low: f64, high: f64, t: f64) -> f64 {
    low + (high - low) * t.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use sim_core::components::SteeringParams;
    use sim_core::components::Personality;

    use super::derive_steering_params;

    #[test]
    fn derive_steering_params_maps_axes_into_expected_ranges() {
        let mut personality = Personality::default();
        personality.axes = [0.0, 1.0, 1.0, 0.0, 1.0, 0.0];
        let params = derive_steering_params(&personality);
        assert!(params.base_speed > 80.0);
        assert_eq!(params.separation_weight, 1.5);
        assert_eq!(params.path_directness, 0.95);
        assert_eq!(params.exploration_radius, 50.0);
    }

    #[test]
    fn stage1_derive_steering_extreme_extraversion() {
        let mut personality = Personality::default();
        personality.axes[2] = 1.0;
        let params = derive_steering_params(&personality);
        assert!(params.base_speed >= 75.0);
        assert!(params.wander_radius >= 45.0);
        assert!(params.social_approach_weight >= 0.9);
    }

    #[test]
    fn stage1_derive_steering_extreme_conscientiousness() {
        let mut personality = Personality::default();
        personality.axes[4] = 1.0;
        let params = derive_steering_params(&personality);
        assert!(params.path_directness >= 0.9);
        assert!(params.wander_suppression >= 0.7);
    }

    #[test]
    fn stage1_derive_steering_extreme_emotionality() {
        let mut personality = Personality::default();
        personality.axes[1] = 1.0;
        let params = derive_steering_params(&personality);
        assert!(params.speed_variance >= 0.18);
        assert!(params.flee_multiplier >= 1.8);
    }

    #[test]
    fn stage1_derive_steering_neutral_personality() {
        let personality = Personality::default();
        let params: SteeringParams = derive_steering_params(&personality);
        assert!(params.base_speed > 50.0 && params.base_speed < 70.0);
        assert!(params.wander_radius > 25.0 && params.wander_radius < 35.0);
    }
}
