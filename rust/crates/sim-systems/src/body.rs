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

/// Energy cost multiplier for action execution.
///
/// Mirrors `needs_system` action energy cost formula.
pub fn action_energy_cost(base_cost: f32, end_norm: f32, end_cost_reduction: f32) -> f32 {
    let clamped_end = clamp_f32(end_norm, 0.0, 1.0);
    base_cost * (1.0 - end_cost_reduction * clamped_end)
}

/// Energy recovery amount during rest.
///
/// Mirrors `needs_system` rest recovery formula.
pub fn rest_energy_recovery(base_recovery: f32, rec_norm: f32, rec_recovery_bonus: f32) -> f32 {
    let clamped_rec = clamp_f32(rec_norm, 0.0, 1.0);
    base_recovery * (1.0 + rec_recovery_bonus * clamped_rec)
}

/// Thirst decay step with temperature acceleration.
///
/// Mirrors `needs_system` thirst decay branch.
pub fn thirst_decay(base_decay: f32, tile_temp: f32, temp_neutral: f32) -> f32 {
    if tile_temp > temp_neutral {
        base_decay * (1.0 + (tile_temp - temp_neutral) * 2.0)
    } else {
        base_decay
    }
}

/// Warmth decay step from tile temperature bands.
///
/// Mirrors `needs_system` warmth decay branch.
pub fn warmth_decay(
    base_decay: f32,
    tile_temp: f32,
    has_tile_temp: bool,
    temp_neutral: f32,
    temp_freezing: f32,
    temp_cold: f32,
) -> f32 {
    if !has_tile_temp {
        return base_decay;
    }
    if tile_temp >= temp_neutral {
        return 0.0;
    }
    if tile_temp < temp_freezing {
        return base_decay * 5.0;
    }
    if tile_temp < temp_cold {
        return base_decay * 3.0;
    }
    let denom = (temp_neutral - temp_cold).max(0.000_001);
    let cold_ratio = (temp_neutral - tile_temp) / denom;
    base_decay * (1.0 + cold_ratio * 2.0)
}

/// Combined baseline need decays.
///
/// Returns `[hunger_decay, energy_decay, social_decay, thirst_decay, warmth_decay, safety_decay]`.
pub fn needs_base_decay_step(
    hunger_value: f32,
    hunger_decay_rate: f32,
    hunger_stage_mult: f32,
    hunger_metabolic_min: f32,
    hunger_metabolic_range: f32,
    energy_decay_rate: f32,
    social_decay_rate: f32,
    safety_decay_rate: f32,
    thirst_base_decay: f32,
    warmth_base_decay: f32,
    tile_temp: f32,
    has_tile_temp: bool,
    temp_neutral: f32,
    temp_freezing: f32,
    temp_cold: f32,
    needs_expansion_enabled: bool,
) -> [f32; 6] {
    let metabolic_factor = hunger_metabolic_min + hunger_metabolic_range * hunger_value;
    let hunger_decay = hunger_decay_rate * hunger_stage_mult * metabolic_factor;
    let thirst = if needs_expansion_enabled {
        thirst_decay(thirst_base_decay, tile_temp, temp_neutral)
    } else {
        0.0
    };
    let warmth = if needs_expansion_enabled {
        warmth_decay(
            warmth_base_decay,
            tile_temp,
            has_tile_temp,
            temp_neutral,
            temp_freezing,
            temp_cold,
        )
    } else {
        0.0
    };
    let safety = if needs_expansion_enabled {
        safety_decay_rate
    } else {
        0.0
    };
    [
        hunger_decay,
        energy_decay_rate,
        social_decay_rate,
        thirst,
        warmth,
        safety,
    ]
}

/// Compute normalized severity for a critical threshold crossing.
///
/// Returns `0.0` when the value is above threshold or threshold is invalid.
pub fn critical_severity(value: f32, critical_threshold: f32) -> f32 {
    if critical_threshold <= 0.0 || value >= critical_threshold {
        return 0.0;
    }
    clamp_f32(1.0 - (value / critical_threshold), 0.0, 1.0)
}

/// Combined critical severities for thirst/warmth/safety.
///
/// Returns `[thirst_severity, warmth_severity, safety_severity]`.
pub fn needs_critical_severity_step(
    thirst: f32,
    warmth: f32,
    safety: f32,
    thirst_critical: f32,
    warmth_critical: f32,
    safety_critical: f32,
) -> [f32; 3] {
    [
        critical_severity(thirst, thirst_critical),
        critical_severity(warmth, warmth_critical),
        critical_severity(safety, safety_critical),
    ]
}

/// Combined ERG frustration tick update.
///
/// Returns:
/// `[growth_ticks, relatedness_ticks, growth_regressing, growth_started, relatedness_regressing, relatedness_started]`.
pub fn erg_frustration_step(
    competence: f32,
    autonomy: f32,
    self_actualization: f32,
    belonging: f32,
    intimacy: f32,
    growth_threshold: f32,
    relatedness_threshold: f32,
    frustration_window: i32,
    growth_ticks: i32,
    relatedness_ticks: i32,
    was_regressing_growth: bool,
    was_regressing_relatedness: bool,
) -> [i32; 6] {
    let growth_frustrated = competence < growth_threshold
        && autonomy < growth_threshold
        && self_actualization < growth_threshold;
    let relatedness_frustrated =
        belonging < relatedness_threshold && intimacy < relatedness_threshold;

    let new_growth_ticks = if growth_frustrated {
        growth_ticks + 1
    } else {
        (growth_ticks - 10).max(0)
    };
    let new_relatedness_ticks = if relatedness_frustrated {
        relatedness_ticks + 1
    } else {
        (relatedness_ticks - 10).max(0)
    };

    let window = frustration_window.max(0);
    let growth_regressing = new_growth_ticks >= window;
    let relatedness_regressing = new_relatedness_ticks >= window;
    let growth_started = growth_regressing && !was_regressing_growth;
    let relatedness_started = relatedness_regressing && !was_regressing_relatedness;

    [
        new_growth_ticks,
        new_relatedness_ticks,
        if growth_regressing { 1 } else { 0 },
        if growth_started { 1 } else { 0 },
        if relatedness_regressing { 1 } else { 0 },
        if relatedness_started { 1 } else { 0 },
    ]
}

/// Additional stress delta for anxious attachment state.
///
/// Returns `stress_rate` only when social need is below `social_threshold`.
pub fn anxious_attachment_stress_delta(
    social: f32,
    social_threshold: f32,
    stress_rate: f32,
) -> f32 {
    if social < social_threshold {
        stress_rate
    } else {
        0.0
    }
}

/// Returns normalized best-skill value in `[0.0, 1.0]`.
///
/// `max_level` is clamped to at least `1` to avoid division by zero.
pub fn upper_needs_best_skill_normalized(skill_levels: &[i32], max_level: i32) -> f32 {
    let mut best = 0_i32;
    for level in skill_levels {
        if *level > best {
            best = *level;
        }
    }
    let denom = max_level.max(1) as f32;
    clamp_f32((best as f32) / denom, 0.0, 1.0)
}

/// Computes job-value alignment in `[0.0, 1.0]`.
///
/// `job_code`: `1 = builder/miner`, `2 = gatherer/lumberjack`, others = no alignment bonus.
pub fn upper_needs_job_alignment(
    job_code: i32,
    craftsmanship: f32,
    skill: f32,
    hard_work: f32,
    nature: f32,
    independence: f32,
) -> f32 {
    let alignment = match job_code {
        1 => maxf32(craftsmanship) * 0.5 + maxf32(skill) * 0.3 + maxf32(hard_work) * 0.2,
        2 => maxf32(nature) * 0.5 + maxf32(independence) * 0.3 + maxf32(hard_work) * 0.2,
        _ => 0.0,
    };
    clamp_f32(alignment, 0.0, 1.0)
}

/// Combined upper-needs decay + fulfillment step.
///
/// Current value order:
/// `[competence, autonomy, self_actualization, meaning, transcendence, recognition, belonging, intimacy]`.
pub fn upper_needs_step(
    current_values: &[f32; 8],
    decay_values: &[f32; 8],
    competence_job_gain: f32,
    autonomy_job_gain: f32,
    belonging_settlement_gain: f32,
    intimacy_partner_gain: f32,
    recognition_skill_coeff: f32,
    self_act_skill_coeff: f32,
    meaning_base_gain: f32,
    meaning_aligned_gain: f32,
    transcendence_settlement_gain: f32,
    transcendence_sacrifice_coeff: f32,
    best_skill_norm: f32,
    alignment: f32,
    sacrifice_value: f32,
    has_job: bool,
    has_settlement: bool,
    has_partner: bool,
) -> [f32; 8] {
    let mut out = [0.0_f32; 8];
    for i in 0..8 {
        out[i] = current_values[i] - decay_values[i];
    }

    if has_job {
        out[0] += competence_job_gain;
        out[1] += autonomy_job_gain;
    }
    if has_settlement {
        out[6] += belonging_settlement_gain;
    }
    if has_partner {
        out[7] += intimacy_partner_gain;
    }

    out[5] += recognition_skill_coeff * best_skill_norm;
    out[2] += self_act_skill_coeff * best_skill_norm;

    out[3] += meaning_base_gain;
    if has_job {
        out[3] += meaning_aligned_gain * alignment;
    }

    if has_settlement {
        out[4] += transcendence_settlement_gain;
    }
    let sacrifice_norm = clamp_f32((sacrifice_value + 1.0) * 0.5, 0.0, 1.0);
    out[4] += transcendence_sacrifice_coeff * sacrifice_norm;

    for i in 0..8 {
        out[i] = clamp_f32(out[i], 0.0, 1.0);
    }
    out
}

#[inline]
fn maxf32(value: f32) -> f32 {
    value.max(0.0)
}

/// Parent-to-child stress transfer under attachment/co-regulation factors.
///
/// `attachment_code`: `0=secure`, `1=anxious`, `2=avoidant`, `3=disorganized`.
pub fn child_parent_stress_transfer(
    parent_stress: f32,
    parent_dependency: f32,
    attachment_code: i32,
    caregiver_support_active: bool,
    buffer_power: f32,
    contagion_input: f32,
) -> f32 {
    let coefficient = match attachment_code {
        0 => 0.15,
        1 => 0.35,
        2 => 0.20,
        3 => 0.45,
        _ => 0.25,
    };

    let mut base_transfer = parent_stress * parent_dependency * coefficient;
    if caregiver_support_active {
        base_transfer *= 1.0 - buffer_power;
    }
    let combined = 1.0 - (1.0 - base_transfer) * (1.0 - contagion_input);
    clamp_f32(combined, 0.0, 1.0)
}

/// Simultaneous ACE burst aggregation.
///
/// Returns `[effective_damage, max_severity_index, kindling_bonus]`.
pub fn child_simultaneous_ace_step(severities: &[f32], prev_residual: f32) -> [f32; 3] {
    if severities.is_empty() {
        return [0.0, -1.0, 0.0];
    }

    let mut burst = 1.0_f32;
    let mut max_severity = -1.0_f32;
    let mut max_index: i32 = -1;
    for (idx, severity_raw) in severities.iter().enumerate() {
        let severity = clamp_f32(*severity_raw, 0.0, 1.0);
        burst *= 1.0 - severity;
        if severity > max_severity {
            max_severity = severity;
            max_index = idx as i32;
        }
    }
    burst = 1.0 - burst;
    let effective_damage = clamp_f32(burst * (1.0 + prev_residual), 0.0, 1.25);
    let kindling_bonus = severities.len().saturating_sub(1) as f32;
    [effective_damage, max_index as f32, kindling_bonus]
}

/// Child stress social-buffer attenuation.
pub fn child_social_buffered_intensity(
    intensity: f32,
    attachment_quality: f32,
    caregiver_present: bool,
    buffer_power: f32,
) -> f32 {
    if !caregiver_present {
        return intensity;
    }
    let social_buffer = attachment_quality * buffer_power;
    intensity * (1.0 - social_buffer)
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
        if trainabilities[idx] < 0 {
            out.push(0);
            continue;
        }
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

/// Compute realized body values in fixed order:
/// `[str, agi, end, tou, rec, dr]`.
///
/// - `potentials` expects at least 6 values (`dr` at index 5).
/// - `trainabilities`, `xps`, `training_ceilings` expect 5 values.
/// - `trainability < 0` means "axis disabled", producing zero training gain.
pub fn calc_realized_values(
    potentials: &[i32],
    trainabilities: &[i32],
    xps: &[f32],
    training_ceilings: &[f32],
    age_years: f32,
    xp_for_full_progress: f32,
) -> Vec<i32> {
    let curves = compute_age_curves(age_years);
    let mut realized = vec![0_i32; 6];
    let len = 5_usize
        .min(potentials.len())
        .min(trainabilities.len())
        .min(xps.len())
        .min(training_ceilings.len());

    for idx in 0..len {
        let gain = if trainabilities[idx] < 0 {
            0
        } else {
            calc_training_gain(
                potentials[idx],
                trainabilities[idx],
                xps[idx],
                training_ceilings[idx],
                xp_for_full_progress,
            )
        };
        let value = ((potentials[idx] + gain) as f32 * curves[idx]) as i32;
        realized[idx] = value.clamp(0, 15_000);
    }

    if potentials.len() > 5 {
        let dr = ((potentials[5] as f32) * curves[5]) as i32;
        realized[5] = dr.clamp(0, 10_000);
    }

    realized
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
        action_energy_cost, age_trainability_modifier, age_trainability_modifiers,
        anxious_attachment_stress_delta, calc_realized_values, calc_training_gain,
        calc_training_gains, child_parent_stress_transfer, child_simultaneous_ace_step,
        child_social_buffered_intensity, compute_age_curve, compute_age_curves, critical_severity,
        erg_frustration_step, needs_base_decay_step, needs_critical_severity_step,
        rest_energy_recovery, thirst_decay, upper_needs_best_skill_normalized,
        upper_needs_job_alignment, upper_needs_step, warmth_decay,
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
    fn action_energy_cost_decreases_with_endurance() {
        let low = action_energy_cost(0.01, 0.1, 0.5);
        let high = action_energy_cost(0.01, 0.9, 0.5);
        assert!(high < low);
    }

    #[test]
    fn rest_energy_recovery_increases_with_recovery_stat() {
        let low = rest_energy_recovery(0.02, 0.1, 0.5);
        let high = rest_energy_recovery(0.02, 0.9, 0.5);
        assert!(high > low);
    }

    #[test]
    fn thirst_decay_accelerates_above_neutral_temp() {
        let neutral = thirst_decay(0.01, 0.5, 0.5);
        let hot = thirst_decay(0.01, 0.8, 0.5);
        assert!(hot > neutral);
    }

    #[test]
    fn warmth_decay_uses_expected_temperature_bands() {
        let base = 0.01_f32;
        let no_temp = warmth_decay(base, 0.2, false, 0.5, 0.1, 0.3);
        assert_eq!(no_temp, base);
        let warm = warmth_decay(base, 0.6, true, 0.5, 0.1, 0.3);
        assert_eq!(warm, 0.0);
        let freezing = warmth_decay(base, 0.05, true, 0.5, 0.1, 0.3);
        assert_eq!(freezing, base * 5.0);
        let cold = warmth_decay(base, 0.2, true, 0.5, 0.1, 0.3);
        assert_eq!(cold, base * 3.0);
    }

    #[test]
    fn base_decay_step_matches_manual_formula() {
        let out = needs_base_decay_step(
            0.7, 0.004, 0.8, 0.5, 0.5, 0.003, 0.002, 0.001, 0.005, 0.01, 0.2, true, 0.5, 0.1, 0.3,
            true,
        );
        let expected_hunger = 0.004 * 0.8 * (0.5 + 0.5 * 0.7);
        assert_eq!(out[0], expected_hunger);
        assert_eq!(out[1], 0.003);
        assert_eq!(out[2], 0.002);
        assert_eq!(out[3], thirst_decay(0.005, 0.2, 0.5));
        assert_eq!(out[4], warmth_decay(0.01, 0.2, true, 0.5, 0.1, 0.3));
        assert_eq!(out[5], 0.001);
    }

    #[test]
    fn critical_severity_returns_zero_above_threshold() {
        assert_eq!(critical_severity(0.9, 0.5), 0.0);
        assert_eq!(critical_severity(0.9, 0.0), 0.0);
    }

    #[test]
    fn critical_severity_step_matches_individual_calculations() {
        let out = needs_critical_severity_step(0.1, 0.2, 0.3, 0.4, 0.5, 0.6);
        assert_eq!(out[0], critical_severity(0.1, 0.4));
        assert_eq!(out[1], critical_severity(0.2, 0.5));
        assert_eq!(out[2], critical_severity(0.3, 0.6));
    }

    #[test]
    fn erg_frustration_step_increments_and_starts_regression() {
        let out =
            erg_frustration_step(0.1, 0.2, 0.3, 0.2, 0.2, 0.5, 0.4, 100, 99, 99, false, false);
        assert_eq!(out[0], 100);
        assert_eq!(out[1], 100);
        assert_eq!(out[2], 1);
        assert_eq!(out[3], 1);
        assert_eq!(out[4], 1);
        assert_eq!(out[5], 1);
    }

    #[test]
    fn erg_frustration_step_recovers_ticks_when_not_frustrated() {
        let out = erg_frustration_step(0.9, 0.8, 0.9, 0.9, 0.8, 0.5, 0.4, 100, 7, 3, true, true);
        assert_eq!(out[0], 0);
        assert_eq!(out[1], 0);
        assert_eq!(out[2], 0);
        assert_eq!(out[3], 0);
        assert_eq!(out[4], 0);
        assert_eq!(out[5], 0);
    }

    #[test]
    fn anxious_attachment_stress_delta_applies_below_threshold() {
        let delta = anxious_attachment_stress_delta(0.2, 0.3, 0.15);
        assert_eq!(delta, 0.15);
    }

    #[test]
    fn anxious_attachment_stress_delta_is_zero_at_or_above_threshold() {
        let at_threshold = anxious_attachment_stress_delta(0.3, 0.3, 0.15);
        let above = anxious_attachment_stress_delta(0.5, 0.3, 0.15);
        assert_eq!(at_threshold, 0.0);
        assert_eq!(above, 0.0);
    }

    #[test]
    fn upper_needs_best_skill_normalized_uses_max_level() {
        let norm = upper_needs_best_skill_normalized(&[12, 45, 31, 20, 44], 100);
        assert_eq!(norm, 0.45);
    }

    #[test]
    fn upper_needs_job_alignment_matches_builder_weights() {
        let alignment = upper_needs_job_alignment(1, 0.8, 0.6, 0.5, 0.1, 0.2);
        assert_eq!(alignment, 0.8 * 0.5 + 0.6 * 0.3 + 0.5 * 0.2);
    }

    #[test]
    fn upper_needs_step_applies_decay_gain_and_clamp() {
        let out = upper_needs_step(
            &[0.1, 0.2, 0.3, 0.95, 0.9, 0.5, 0.2, 0.1],
            &[0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01],
            0.05,
            0.04,
            0.03,
            0.02,
            0.10,
            0.12,
            0.02,
            0.08,
            0.03,
            0.04,
            0.5,
            0.8,
            0.6,
            true,
            true,
            true,
        );
        assert!(out.iter().all(|v| (0.0..=1.0).contains(v)));
        assert!(out[0] > 0.1);
        assert!(out[4] > 0.9 - 0.01);
        assert!(out[3] <= 1.0);
    }

    #[test]
    fn child_parent_stress_transfer_respects_attachment_profile() {
        let secure = child_parent_stress_transfer(0.7, 0.8, 0, false, 0.0, 0.0);
        let disorganized = child_parent_stress_transfer(0.7, 0.8, 3, false, 0.0, 0.0);
        assert!(disorganized > secure);
    }

    #[test]
    fn child_parent_stress_transfer_applies_social_buffer() {
        let no_buffer = child_parent_stress_transfer(0.8, 0.7, 1, false, 0.4, 0.0);
        let with_buffer = child_parent_stress_transfer(0.8, 0.7, 1, true, 0.4, 0.0);
        assert!(with_buffer < no_buffer);
    }

    #[test]
    fn child_simultaneous_ace_step_matches_expected_shape() {
        let out = child_simultaneous_ace_step(&[0.2, 0.8, 0.1], 0.1);
        assert!(out[0] > 0.0);
        assert_eq!(out[1], 1.0);
        assert_eq!(out[2], 2.0);
    }

    #[test]
    fn child_simultaneous_ace_step_empty_input_returns_zero() {
        let out = child_simultaneous_ace_step(&[], 0.4);
        assert_eq!(out[0], 0.0);
        assert_eq!(out[1], -1.0);
        assert_eq!(out[2], 0.0);
    }

    #[test]
    fn child_social_buffered_intensity_reduces_when_caregiver_present() {
        let no_support = child_social_buffered_intensity(0.8, 0.7, false, 0.5);
        let with_support = child_social_buffered_intensity(0.8, 0.7, true, 0.5);
        assert!(with_support < no_support);
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
    fn batch_training_gains_skip_negative_trainability() {
        let potentials = [1000, 700];
        let trainabilities = [-1, 600];
        let xps = [10_000.0_f32, 10_000.0_f32];
        let ceilings = [0.5_f32, 0.5_f32];
        let batched = calc_training_gains(&potentials, &trainabilities, &xps, &ceilings, 10_000.0);
        assert_eq!(batched[0], 0);
        assert!(batched[1] > 0);
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

    #[test]
    fn realized_values_match_manual_formula() {
        let potentials = [1000, 900, 800, 700, 600, 500];
        let trainabilities = [500, 600, 700, 800, 900];
        let xps = [
            2_000.0_f32,
            3_000.0_f32,
            4_000.0_f32,
            5_000.0_f32,
            6_000.0_f32,
        ];
        let ceilings = [0.5_f32, 0.3_f32, 1.5_f32, 0.2_f32, 0.6_f32];
        let age = 30.0_f32;
        let out =
            calc_realized_values(&potentials, &trainabilities, &xps, &ceilings, age, 10_000.0);
        assert_eq!(out.len(), 6);
        let curves = compute_age_curves(age);
        for idx in 0..5 {
            let gain = calc_training_gain(
                potentials[idx],
                trainabilities[idx],
                xps[idx],
                ceilings[idx],
                10_000.0,
            );
            let expected = ((potentials[idx] + gain) as f32 * curves[idx]) as i32;
            assert_eq!(out[idx], expected.clamp(0, 15_000));
        }
        let expected_dr = ((potentials[5] as f32) * curves[5]) as i32;
        assert_eq!(out[5], expected_dr.clamp(0, 10_000));
    }
}
