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

#[cfg(test)]
mod tests {
    use super::compute_age_curve;

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
}
