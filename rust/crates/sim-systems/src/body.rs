//! Body-related hot-path math extracted from GDScript.
//!
//! `compute_age_curve` mirrors `BodyAttributes.compute_age_curve` in GDScript.

#[inline]
fn clamp_f32(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

/// Compute age curve multiplier for a body axis.
///
/// Axes: `str`, `agi`, `end`, `tou`, `rec`, `dr`.
/// Returns value in `[0.02, 1.0]`.
pub fn compute_age_curve(axis: &str, age_years: f32) -> f32 {
    let (a50, k, t0, r1, t1, r2) = match axis {
        "str" => (16.0_f32, 0.35_f32, 35.0_f32, 0.007_f32, 70.0_f32, 0.030_f32),
        "agi" => (14.0_f32, 0.45_f32, 25.0_f32, 0.009_f32, 65.0_f32, 0.035_f32),
        "end" => (15.0_f32, 0.38_f32, 30.0_f32, 0.008_f32, 70.0_f32, 0.020_f32),
        "tou" => (17.0_f32, 0.32_f32, 40.0_f32, 0.007_f32, 75.0_f32, 0.020_f32),
        "rec" => (12.0_f32, 0.50_f32, 20.0_f32, 0.011_f32, 60.0_f32, 0.030_f32),
        "dr" => (6.0_f32, 0.90_f32, 55.0_f32, 0.010_f32, 75.0_f32, 0.030_f32),
        _ => return 0.5_f32,
    };

    let grow: f32 = 1.0_f32 / (1.0_f32 + (-k * (age_years - a50)).exp());
    let decl1: f32 = (-r1 * (age_years - t0).max(0.0_f32)).exp();
    let decl2: f32 = (-r2 * (age_years - t1).max(0.0_f32)).exp();
    let raw: f32 = clamp_f32(grow * decl1 * decl2, 0.02_f32, 1.0_f32);

    if axis == "dr" {
        let maternal_bonus: f32 = 0.20_f32 * (-age_years / 0.5_f32).exp();
        clamp_f32(raw + maternal_bonus, 0.02_f32, 1.0_f32)
    } else {
        raw
    }
}

/// Compute all body age curves in fixed axis order:
/// `[str, agi, end, tou, rec, dr]`.
pub fn compute_age_curves(age_years: f32) -> [f32; 6] {
    [
        compute_age_curve("str", age_years),
        compute_age_curve("agi", age_years),
        compute_age_curve("end", age_years),
        compute_age_curve("tou", age_years),
        compute_age_curve("rec", age_years),
        compute_age_curve("dr", age_years),
    ]
}

/// Compute training gain for a body axis.
///
/// Mirrors `BodyAttributes.calc_training_gain` math.
pub fn calc_training_gain(
    potential: i32,
    trainability: i32,
    xp: f32,
    training_ceiling: f32,
    xp_for_full_progress: f32,
) -> i32 {
    let max_gain = (potential as f32) * training_ceiling;
    let progress_divisor = if xp_for_full_progress <= 0.0 {
        1.0
    } else {
        xp_for_full_progress
    };
    let xp_progress = clamp_f32(xp / progress_divisor, 0.0, 1.0);
    let xp_factor = 1.0 - (-3.0 * xp_progress).exp();
    let train_factor = clamp_f32((trainability as f32) / 500.0, 0.1, 2.0);
    let gain = max_gain * xp_factor * train_factor;
    let clamped_gain = clamp_f32(gain, 0.0, max_gain * 2.0);
    clamped_gain as i32
}

/// Compute training gains for multiple axes in one pass.
///
/// Uses the shortest input length among the provided slices.
pub fn calc_training_gains(
    potentials: &[i32],
    trainabilities: &[i32],
    xps: &[f32],
    training_ceilings: &[f32],
    xp_for_full_progress: f32,
) -> Vec<i32> {
    let len = potentials
        .len()
        .min(trainabilities.len())
        .min(xps.len())
        .min(training_ceilings.len());
    let mut out = Vec::with_capacity(len);
    for idx in 0..len {
        out.push(calc_training_gain(
            potentials[idx],
            trainabilities[idx],
            xps[idx],
            training_ceilings[idx],
            xp_for_full_progress,
        ));
    }
    out
}

/// Age-based trainability modifier for each body axis.
///
/// Mirrors `BodyAttributes.get_age_trainability_modifier`.
pub fn age_trainability_modifier(axis: &str, age_years: f32) -> f32 {
    match axis {
        "str" => {
            if age_years < 13.0 {
                0.60
            } else if age_years < 18.0 {
                0.85
            } else if age_years < 31.0 {
                1.00
            } else if age_years < 51.0 {
                0.85
            } else if age_years < 66.0 {
                0.65
            } else if age_years < 81.0 {
                0.45
            } else {
                0.30
            }
        }
        "end" => {
            if age_years < 13.0 {
                0.70
            } else if age_years < 18.0 {
                0.90
            } else if age_years < 31.0 {
                1.00
            } else if age_years < 51.0 {
                0.90
            } else if age_years < 66.0 {
                0.75
            } else if age_years < 81.0 {
                0.55
            } else {
                0.35
            }
        }
        "agi" => {
            if age_years < 7.0 {
                0.70
            } else if age_years < 14.0 {
                1.00
            } else if age_years < 18.0 {
                0.85
            } else if age_years < 31.0 {
                0.80
            } else if age_years < 51.0 {
                0.65
            } else {
                0.45
            }
        }
        "tou" => {
            if age_years < 13.0 {
                1.00
            } else if age_years < 26.0 {
                0.90
            } else if age_years < 41.0 {
                0.50
            } else if age_years < 61.0 {
                0.30
            } else {
                0.15
            }
        }
        "rec" => {
            if age_years < 13.0 {
                0.60
            } else if age_years < 18.0 {
                0.85
            } else if age_years < 31.0 {
                1.00
            } else if age_years < 51.0 {
                0.80
            } else if age_years < 66.0 {
                0.60
            } else if age_years < 81.0 {
                0.40
            } else {
                0.25
            }
        }
        _ => 1.00,
    }
}

/// Compute age-based trainability modifiers in fixed axis order:
/// `[str, agi, end, tou, rec]`.
pub fn age_trainability_modifiers(age_years: f32) -> [f32; 5] {
    [
        age_trainability_modifier("str", age_years),
        age_trainability_modifier("agi", age_years),
        age_trainability_modifier("end", age_years),
        age_trainability_modifier("tou", age_years),
        age_trainability_modifier("rec", age_years),
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        age_trainability_modifier, age_trainability_modifiers, calc_training_gain,
        calc_training_gains,
        compute_age_curve, compute_age_curves,
    };

    #[test]
    fn unknown_axis_returns_default_midpoint() {
        assert_eq!(compute_age_curve("unknown", 25.0), 0.5);
    }

    #[test]
    fn curves_remain_in_valid_range() {
        for axis in ["str", "agi", "end", "tou", "rec", "dr"] {
            for age in [0.0_f32, 5.0_f32, 20.0_f32, 45.0_f32, 80.0_f32, 110.0_f32] {
                let v = compute_age_curve(axis, age);
                assert!((0.02..=1.0).contains(&v), "axis={axis} age={age} value={v}");
            }
        }
    }

    #[test]
    fn strength_curve_has_growth_then_decline_shape() {
        let child = compute_age_curve("str", 5.0);
        let adult = compute_age_curve("str", 25.0);
        let elder = compute_age_curve("str", 95.0);
        assert!(adult > child);
        assert!(adult > elder);
    }

    #[test]
    fn dr_gets_early_maternal_bonus() {
        let age = 0.0_f32;
        let grow = 1.0_f32 / (1.0_f32 + (-0.9_f32 * (age - 6.0_f32)).exp());
        let raw = grow.clamp(0.02_f32, 1.0_f32);
        let with_bonus = compute_age_curve("dr", age);
        assert!(with_bonus > raw);
    }

    #[test]
    fn batch_curve_order_matches_single_curve_calls() {
        let age = 29.0_f32;
        let curves = compute_age_curves(age);
        assert_eq!(curves[0], compute_age_curve("str", age));
        assert_eq!(curves[1], compute_age_curve("agi", age));
        assert_eq!(curves[2], compute_age_curve("end", age));
        assert_eq!(curves[3], compute_age_curve("tou", age));
        assert_eq!(curves[4], compute_age_curve("rec", age));
        assert_eq!(curves[5], compute_age_curve("dr", age));
    }

    #[test]
    fn training_gain_is_zero_at_zero_xp() {
        let gain = calc_training_gain(1000, 500, 0.0, 0.5, 10_000.0);
        assert_eq!(gain, 0);
    }

    #[test]
    fn training_gain_scales_with_trainability() {
        let low = calc_training_gain(1000, 300, 10_000.0, 0.5, 10_000.0);
        let high = calc_training_gain(1000, 800, 10_000.0, 0.5, 10_000.0);
        assert!(high > low);
    }

    #[test]
    fn training_gain_clamps_to_twice_max_gain() {
        let gain = calc_training_gain(1000, 5000, 10_000_000.0, 0.5, 1.0);
        assert!(gain <= 1000);
        assert!(gain > 0);
    }

    #[test]
    fn batch_training_gains_match_single_calls() {
        let potentials = [1000, 700, 900];
        let trainabilities = [400, 600, 800];
        let xps = [500.0_f32, 2500.0_f32, 10_000.0_f32];
        let ceilings = [0.5_f32, 0.3_f32, 1.5_f32];
        let batched = calc_training_gains(&potentials, &trainabilities, &xps, &ceilings, 10_000.0);
        assert_eq!(batched.len(), 3);
        for idx in 0..batched.len() {
            let single = calc_training_gain(
                potentials[idx],
                trainabilities[idx],
                xps[idx],
                ceilings[idx],
                10_000.0,
            );
            assert_eq!(batched[idx], single);
        }
    }

    #[test]
    fn trainability_modifier_defaults_to_one_for_unknown_axis() {
        assert_eq!(age_trainability_modifier("unknown", 20.0), 1.0);
    }

    #[test]
    fn trainability_modifier_declines_with_age_for_strength() {
        let adult = age_trainability_modifier("str", 25.0);
        let elder = age_trainability_modifier("str", 85.0);
        assert!(adult > elder);
    }

    #[test]
    fn batch_trainability_order_matches_single_calls() {
        let age = 44.0_f32;
        let mods = age_trainability_modifiers(age);
        assert_eq!(mods[0], age_trainability_modifier("str", age));
        assert_eq!(mods[1], age_trainability_modifier("agi", age));
        assert_eq!(mods[2], age_trainability_modifier("end", age));
        assert_eq!(mods[3], age_trainability_modifier("tou", age));
        assert_eq!(mods[4], age_trainability_modifier("rec", age));
    }
}
