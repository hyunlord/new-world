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

/// Computes job satisfaction score in `[0.0, 1.0]`.
///
/// Inputs are normalized:
/// - personality actual axes in `[0,1]`
/// - personality ideal axes in `[-1,1]` (sign indicates direction)
/// - value actual in `[0,1]`
/// - value weights in `[0,1]`
pub fn job_satisfaction_score(
    personality_actual: &[f32],
    personality_ideal: &[f32],
    value_actual: &[f32],
    value_weights: &[f32],
    skill_fit: f32,
    autonomy: f32,
    competence: f32,
    meaning: f32,
    autonomy_level: f32,
    prestige: f32,
    w_skill_fit: f32,
    w_value_fit: f32,
    w_personality_fit: f32,
    w_need_fit: f32,
) -> f32 {
    let personality_len = personality_actual.len().min(personality_ideal.len());
    let mut personality_sum = 0.0_f32;
    let mut personality_weight = 0.0_f32;
    for i in 0..personality_len {
        let ideal = personality_ideal[i];
        if ideal.abs() <= f32::EPSILON {
            continue;
        }
        let actual = clamp_f32(personality_actual[i], 0.0, 1.0);
        let weight = ideal.abs();
        if ideal > 0.0 {
            personality_sum += weight * actual;
        } else {
            personality_sum += weight * (1.0 - actual);
        }
        personality_weight += weight;
    }
    let personality_fit = if personality_weight > 0.0 {
        personality_sum / personality_weight
    } else {
        0.5
    };

    let value_len = value_actual.len().min(value_weights.len());
    let mut value_sum = 0.0_f32;
    let mut value_weight = 0.0_f32;
    for i in 0..value_len {
        let weight = value_weights[i];
        let actual = clamp_f32(value_actual[i], 0.0, 1.0);
        value_sum += weight * actual;
        value_weight += weight;
    }
    let value_fit = if value_weight > 0.0 {
        value_sum / value_weight
    } else {
        0.5
    };

    let clamped_skill_fit = clamp_f32(skill_fit, 0.0, 1.0);
    let need_fit = autonomy * autonomy_level * 0.35
        + competence * clamped_skill_fit * 0.35
        + meaning * prestige * 0.30;

    clamp_f32(
        clamped_skill_fit * w_skill_fit
            + value_fit * w_value_fit
            + personality_fit * w_personality_fit
            + need_fit * w_need_fit,
        0.0,
        1.0,
    )
}

/// Batch version of [`job_satisfaction_score`].
///
/// Candidate vectors are flattened in row-major order:
/// - `personality_ideals_flat`: `candidate_count * axis_count`
/// - `value_weights_flat`: `candidate_count * value_count`
pub fn job_satisfaction_score_batch(
    personality_actual: &[f32],
    personality_ideals_flat: &[f32],
    value_actual: &[f32],
    value_weights_flat: &[f32],
    skill_fits: &[f32],
    autonomy: f32,
    competence: f32,
    meaning: f32,
    autonomy_levels: &[f32],
    prestiges: &[f32],
    w_skill_fit: f32,
    w_value_fit: f32,
    w_personality_fit: f32,
    w_need_fit: f32,
) -> Vec<f32> {
    let axis_count = personality_actual.len();
    let value_count = value_actual.len();
    if axis_count == 0 || value_count == 0 {
        return Vec::new();
    }

    let by_personality = personality_ideals_flat.len() / axis_count;
    let by_value = value_weights_flat.len() / value_count;
    let candidate_count = by_personality
        .min(by_value)
        .min(skill_fits.len())
        .min(autonomy_levels.len())
        .min(prestiges.len());
    if candidate_count == 0 {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(candidate_count);
    for idx in 0..candidate_count {
        let pi_start = idx * axis_count;
        let pi_end = pi_start + axis_count;
        let vw_start = idx * value_count;
        let vw_end = vw_start + value_count;
        let score = job_satisfaction_score(
            personality_actual,
            &personality_ideals_flat[pi_start..pi_end],
            value_actual,
            &value_weights_flat[vw_start..vw_end],
            skill_fits[idx],
            autonomy,
            competence,
            meaning,
            autonomy_levels[idx],
            prestiges[idx],
            w_skill_fit,
            w_value_fit,
            w_personality_fit,
            w_need_fit,
        );
        out.push(score);
    }
    out
}

/// Returns the index of the highest skill level.
///
/// When multiple entries share the maximum value, the first index is returned.
/// Returns `-1` for an empty input slice.
pub fn occupation_best_skill_index(skill_levels: &[i32]) -> i32 {
    if skill_levels.is_empty() {
        return -1;
    }
    let mut best_idx: i32 = 0;
    let mut best_level: i32 = skill_levels[0];
    for (idx, level) in skill_levels.iter().enumerate() {
        if *level > best_level {
            best_level = *level;
            best_idx = idx as i32;
        }
    }
    best_idx
}

/// Returns whether occupation should switch based on normalized margin.
///
/// Normalized margin is `(best_skill_level - current_skill_level) / 100.0`.
pub fn occupation_should_switch(
    best_skill_level: i32,
    current_skill_level: i32,
    change_hysteresis: f32,
) -> bool {
    let normalized_margin = (best_skill_level as f32 - current_skill_level as f32) / 100.0;
    normalized_margin >= change_hysteresis
}

/// Returns job index with maximum deficit (`target - current`).
///
/// Falls back to `0` when inputs are empty.
pub fn job_assignment_best_job_code(ratios: &[f32], counts: &[i32], alive_count: i32) -> i32 {
    let len = ratios.len().min(counts.len());
    if len == 0 {
        return 0;
    }
    let mut best_idx: i32 = 0;
    let mut best_deficit: f32 = f32::MIN;
    for idx in 0..len {
        let target = ratios[idx] * alive_count as f32;
        let deficit = target - counts[idx] as f32;
        if deficit > best_deficit {
            best_deficit = deficit;
            best_idx = idx as i32;
        }
    }
    best_idx
}

/// Returns `[worst_surplus_job_idx, worst_deficit_job_idx]` for rebalancing.
///
/// Returns `[-1, -1]` when no rebalance is needed.
pub fn job_assignment_rebalance_codes(
    ratios: &[f32],
    counts: &[i32],
    alive_count: i32,
    threshold: f32,
) -> [i32; 2] {
    let len = ratios.len().min(counts.len());
    if len == 0 {
        return [-1, -1];
    }
    let mut worst_surplus_idx: i32 = -1;
    let mut worst_surplus: f32 = 0.0;
    let mut worst_deficit_idx: i32 = -1;
    let mut worst_deficit: f32 = 0.0;

    for idx in 0..len {
        let target = ratios[idx] * alive_count as f32;
        let current = counts[idx] as f32;
        let surplus = current - target;
        if surplus > worst_surplus {
            worst_surplus = surplus;
            worst_surplus_idx = idx as i32;
        }
        let deficit = target - current;
        if deficit > worst_deficit {
            worst_deficit = deficit;
            worst_deficit_idx = idx as i32;
        }
    }

    if worst_surplus < threshold || worst_deficit < threshold {
        return [-1, -1];
    }
    [worst_surplus_idx, worst_deficit_idx]
}

/// Returns whether a threshold effect should be active.
///
/// `direction_code`: `0=below`, `1=above`.
pub fn stat_threshold_is_active(
    value: i32,
    threshold: i32,
    direction_code: i32,
    hysteresis: i32,
    currently_active: bool,
) -> bool {
    match direction_code {
        0 => {
            if currently_active {
                value < threshold + hysteresis
            } else {
                value < threshold
            }
        }
        1 => {
            if currently_active {
                value > threshold - hysteresis
            } else {
                value > threshold
            }
        }
        _ => false,
    }
}

/// Computes food/wood/stone delta rates per 100 ticks.
///
/// Returns `[food_rate, wood_rate, stone_rate]`.
pub fn stats_resource_deltas_per_100(
    latest_food: f32,
    latest_wood: f32,
    latest_stone: f32,
    older_food: f32,
    older_wood: f32,
    older_stone: f32,
    tick_diff: f32,
) -> [f32; 3] {
    if tick_diff <= 0.0 {
        return [0.0, 0.0, 0.0];
    }
    let scale = 100.0 / tick_diff;
    [
        (latest_food - older_food) * scale,
        (latest_wood - older_wood) * scale,
        (latest_stone - older_stone) * scale,
    ]
}

/// Linear maturation target:
/// `0` below `start_age`, `max_shift` at/after `end_age`, linear in between.
pub fn personality_linear_target(age: i32, max_shift: f32, start_age: i32, end_age: i32) -> f32 {
    if age < start_age {
        return 0.0;
    }
    let span = (end_age - start_age) as f32;
    if span <= 0.0 {
        return max_shift;
    }
    let t = clamp_f32((age - start_age) as f32 / span, 0.0, 1.0);
    max_shift * t
}

/// Computes effective intelligence value with age/activity/ACE modifiers.
pub fn intelligence_effective_value(
    potential: f32,
    base_mod: f32,
    age_years: f32,
    is_fluid: bool,
    activity_mod: f32,
    ace_fluid_mult: f32,
    env_penalty: f32,
    min_val: f32,
    max_val: f32,
) -> f32 {
    let effective_mod = if base_mod < 1.0 && age_years > 25.0 {
        let mut decline_amount = 1.0 - base_mod;
        if is_fluid {
            decline_amount *= activity_mod * ace_fluid_mult;
        } else {
            decline_amount *= activity_mod;
        }
        1.0 - decline_amount
    } else {
        base_mod
    };
    clamp_f32((potential - env_penalty) * effective_mod, min_val, max_val)
}

/// Computes intelligence `g` value from parental blend, openness shift, and noise.
pub fn intelligence_g_value(
    has_parents: bool,
    parent_a_g: f32,
    parent_b_g: f32,
    heritability_g: f32,
    g_mean: f32,
    openness_mean: f32,
    openness_weight: f32,
    noise: f32,
) -> f32 {
    let base = if has_parents {
        let mid = (parent_a_g + parent_b_g) * 0.5;
        mid * heritability_g + g_mean * (1.0 - heritability_g)
    } else {
        g_mean
    };
    base + openness_weight * (openness_mean - 0.5) + noise
}

/// Computes child axis z-score from inheritance, sex shift, and culture shift.
pub fn personality_child_axis_z(
    has_parents: bool,
    parent_a_axis: f32,
    parent_b_axis: f32,
    heritability: f32,
    random_axis_z: f32,
    is_female: bool,
    sex_diff_d: f32,
    culture_shift: f32,
) -> f32 {
    let z_mid = if has_parents {
        0.5 * (parent_a_axis + parent_b_axis)
    } else {
        0.0
    };
    let env_factor = (1.0 - 0.5 * heritability * heritability).sqrt();
    let mut z_child = heritability * z_mid + env_factor * random_axis_z;
    if is_female {
        z_child += sex_diff_d * 0.5;
    } else {
        z_child -= sex_diff_d * 0.5;
    }
    z_child + culture_shift
}

/// Behavior weight multiplier from morale with flourishing/normal/dissatisfied bands.
pub fn morale_behavior_weight_multiplier(
    morale: f32,
    flourishing_threshold: f32,
    flourishing_min: f32,
    flourishing_max: f32,
    normal_min: f32,
    normal_max: f32,
    dissatisfied_min: f32,
    dissatisfied_max: f32,
    languishing_min: f32,
    languishing_max: f32,
) -> f32 {
    if morale >= flourishing_threshold {
        let slope = (flourishing_max - flourishing_min) / (1.0 - flourishing_threshold);
        return clamp_f32(
            flourishing_min + (morale - flourishing_threshold) * slope,
            flourishing_min,
            flourishing_max,
        );
    }
    if morale >= 0.3 {
        let t = (morale - 0.3) / (flourishing_threshold - 0.3);
        return normal_min + (normal_max - normal_min) * t;
    }
    if morale >= 0.0 {
        let t = morale / 0.3;
        return dissatisfied_min + (dissatisfied_max - dissatisfied_min) * t;
    }
    let t = clamp_f32(morale + 1.0, 0.0, 1.0);
    languishing_min + (languishing_max - languishing_min) * t
}

/// Migration probability based on settlement morale and patience resistance.
pub fn morale_migration_probability(
    morale_s: f32,
    k: f32,
    threshold_morale: f32,
    patience: f32,
    patience_resistance: f32,
    max_probability: f32,
) -> f32 {
    let p_base = 1.0 / (1.0 + (-k * (threshold_morale - morale_s)).exp());
    let resistance = patience_resistance * patience;
    clamp_f32(p_base - resistance, 0.0, max_probability)
}

/// Computes StatSync derived scores.
///
/// Input order:
/// `[X, A, H, E, O, C, joy, anticipation, anger, str_pot, romance, truth, artwork,
///   knowledge, merriment, friendship, competition, recognition, i_ling, i_log, i_spa,
///   i_mus, i_kin, i_inter, i_intra, i_nat, attract, ent_height, age_years]`
///
/// Returns:
/// `[charisma, intimidation, allure, trustworthiness, creativity, wisdom, popularity, risk_tolerance]`
pub fn stat_sync_derived_scores(inputs: &[f32]) -> [f32; 8] {
    if inputs.len() < 29 {
        return [0.0; 8];
    }
    let x = inputs[0];
    let a = inputs[1];
    let h = inputs[2];
    let e = inputs[3];
    let o = inputs[4];
    let c = inputs[5];
    let joy = inputs[6];
    let anticipation = inputs[7];
    let anger = inputs[8];
    let str_pot = inputs[9];
    let romance = inputs[10];
    let truth = inputs[11];
    let artwork = inputs[12];
    let knowledge = inputs[13];
    let merriment = inputs[14];
    let friendship = inputs[15];
    let competition = inputs[16];
    let recognition = inputs[17];
    let i_ling = inputs[18];
    let i_log = inputs[19];
    let i_spa = inputs[20];
    let i_mus = inputs[21];
    let i_kin = inputs[22];
    let i_inter = inputs[23];
    let i_intra = inputs[24];
    let i_nat = inputs[25];
    let attract = inputs[26];
    let ent_height = inputs[27];
    let age_years = inputs[28];

    let charisma = clamp_f32(
        i_inter * 0.16
            + i_ling * 0.10
            + x * 0.22
            + a * 0.16
            + h * 0.08
            + joy * 0.08
            + (1.0 - e) * 0.06
            + recognition * 0.04
            + merriment * 0.05
            + friendship * 0.05,
        0.0,
        1.0,
    );
    let intimidation = clamp_f32(
        str_pot * 0.28
            + ent_height * 0.17
            + anger * 0.22
            + (1.0 - e) * 0.12
            + x * 0.10
            + competition * 0.06
            + i_kin * 0.05,
        0.0,
        1.0,
    );
    let allure = clamp_f32(
        attract * 0.35 + charisma * 0.35 + romance * 0.20 + x * 0.10,
        0.0,
        1.0,
    );
    let trustworthiness = clamp_f32(h * 0.40 + a * 0.30 + truth * 0.30, 0.0, 1.0);
    let creativity = clamp_f32(
        i_spa * 0.15
            + i_mus * 0.10
            + i_ling * 0.05
            + o * 0.25
            + anticipation * 0.15
            + artwork * 0.15
            + i_intra * 0.05
            + x * 0.10,
        0.0,
        1.0,
    );
    let age_factor = clamp_f32((age_years - 25.0) / 35.0, 0.0, 1.0);
    let wisdom = clamp_f32(
        i_intra * 0.16
            + i_log * 0.12
            + c * 0.14
            + o * 0.10
            + a * 0.10
            + knowledge * 0.14
            + age_factor * 0.12
            + i_ling * 0.06
            + i_nat * 0.06,
        0.0,
        1.0,
    );
    let popularity = clamp_f32(
        charisma * 0.50 + merriment * 0.25 + friendship * 0.25,
        0.0,
        1.0,
    );
    let risk_tolerance = clamp_f32(
        (1.0 - e) * 0.40 + o * 0.30 + competition * 0.15 + (1.0 - c) * 0.15,
        0.0,
        1.0,
    );

    [
        charisma,
        intimidation,
        allure,
        trustworthiness,
        creativity,
        wisdom,
        popularity,
        risk_tolerance,
    ]
}

/// Total AoE contagion susceptibility from refractory, personality, and crowd dilution.
pub fn contagion_aoe_total_susceptibility(
    donor_count: i32,
    crowd_dilute_divisor: f32,
    refractory_active: bool,
    refractory_susceptibility: f32,
    x_axis: f32,
    e_axis: f32,
) -> f32 {
    let divisor = if crowd_dilute_divisor <= 0.0 {
        1.0
    } else {
        crowd_dilute_divisor
    };
    let donor_ratio = (donor_count.max(0) as f32) / divisor;
    let crowd_factor = 1.0 / donor_ratio.max(1.0).sqrt();
    let susceptibility = if refractory_active {
        refractory_susceptibility
    } else {
        1.0
    };
    let personality_susceptibility = 0.7 + 0.3 * x_axis + 0.2 * (e_axis - 0.5);
    susceptibility * personality_susceptibility * crowd_factor
}

/// Stress contagion delta under a minimum gap threshold.
pub fn contagion_stress_delta(
    stress_gap: f32,
    stress_gap_threshold: f32,
    transfer_rate: f32,
    total_susceptibility: f32,
    max_delta: f32,
) -> f32 {
    if stress_gap <= stress_gap_threshold {
        return 0.0;
    }
    clamp_f32(
        stress_gap * transfer_rate * total_susceptibility,
        0.0,
        max_delta,
    )
}

/// Network contagion valence delta from social susceptibility and crowd dilution.
pub fn contagion_network_delta(
    donor_count: i32,
    crowd_dilute_divisor: f32,
    refractory_active: bool,
    refractory_susceptibility: f32,
    network_decay: f32,
    a_axis: f32,
    valence_gap: f32,
    delta_scale: f32,
    max_abs_delta: f32,
) -> f32 {
    let divisor = if crowd_dilute_divisor <= 0.0 {
        1.0
    } else {
        crowd_dilute_divisor
    };
    let donor_ratio = (donor_count.max(0) as f32) / divisor;
    let crowd_factor = 1.0 / donor_ratio.max(1.0).sqrt();
    let mut base_weight = if refractory_active {
        refractory_susceptibility
    } else {
        1.0
    };
    base_weight *= network_decay;
    base_weight *= 0.8 + 0.4 * a_axis;
    base_weight *= crowd_factor;
    clamp_f32(
        valence_gap * base_weight * delta_scale,
        -max_abs_delta,
        max_abs_delta,
    )
}

/// Spiral amplification increment for high-stress negative-valence states.
pub fn contagion_spiral_increment(
    stress: f32,
    valence: f32,
    stress_threshold: f32,
    valence_threshold: f32,
    stress_divisor: f32,
    valence_divisor: f32,
    intensity_scale: f32,
    max_increment: f32,
) -> f32 {
    if stress < stress_threshold || valence >= valence_threshold {
        return 0.0;
    }
    let stress_norm = (stress - stress_threshold)
        / if stress_divisor <= 0.0 {
            1.0
        } else {
            stress_divisor
        };
    let valence_norm = (valence - valence_threshold).abs()
        / if valence_divisor <= 0.0 {
            1.0
        } else {
            valence_divisor
        };
    clamp_f32(
        intensity_scale * stress_norm * valence_norm,
        0.0,
        max_increment,
    )
}

/// Mental-break threshold from resilience, personality, resources, and trauma modifiers.
pub fn mental_break_threshold(
    base_break_threshold: f32,
    resilience: f32,
    c_axis: f32,
    e_axis: f32,
    allostatic: f32,
    energy_norm: f32,
    hunger_norm: f32,
    ace_break_threshold_mult: f32,
    trait_break_threshold_add: f32,
    threshold_min: f32,
    threshold_max: f32,
    reserve: f32,
    scar_threshold_reduction: f32,
) -> f32 {
    let mut threshold = base_break_threshold;
    threshold *= 1.0 + 0.40 * (resilience - 0.5) * 2.0;
    threshold *= 1.0 + 0.25 * (c_axis - 0.5) * 2.0;
    threshold *= 1.0 - 0.35 * (e_axis - 0.5) * 2.0;
    threshold *= 1.0 - 0.25 * (allostatic / 100.0);
    threshold *= 0.85 + 0.15 * energy_norm;
    threshold *= 0.85 + 0.15 * hunger_norm;
    threshold *= ace_break_threshold_mult;
    threshold += trait_break_threshold_add;
    threshold = clamp_f32(threshold, threshold_min, threshold_max);
    if reserve < 30.0 {
        threshold -= 40.0;
    }
    if reserve < 15.0 {
        threshold -= 80.0;
    }
    threshold -= scar_threshold_reduction;
    threshold.max(threshold_min)
}

/// Mental-break trigger chance from stress over-threshold amount and stress-state multipliers.
pub fn mental_break_chance(
    stress: f32,
    threshold: f32,
    reserve: f32,
    allostatic: f32,
    break_scale: f32,
    break_cap_per_tick: f32,
) -> f32 {
    if stress <= threshold || break_scale <= 0.0 {
        return 0.0;
    }
    let mut p = clamp_f32((stress - threshold) / break_scale, 0.0, break_cap_per_tick);
    if reserve < 30.0 {
        p *= 1.3;
    }
    if allostatic > 60.0 {
        p *= 1.2;
    }
    p
}

/// Trait-violation context multiplier from forced/survival/witness conditions.
pub fn trait_violation_context_modifier(
    is_habit: bool,
    forced_by_authority: bool,
    survival_necessity: bool,
    no_witness: bool,
    repeated_habit_modifier: f32,
    forced_modifier: f32,
    survival_modifier: f32,
    no_witness_modifier: f32,
) -> f32 {
    if is_habit {
        return repeated_habit_modifier;
    }
    let mut mult = 1.0;
    if forced_by_authority {
        mult *= forced_modifier;
    }
    if survival_necessity {
        mult *= survival_modifier;
    }
    if no_witness {
        mult *= no_witness_modifier;
    }
    mult
}

/// Trait-violation facet scale where extreme facets amplify dissonance stress.
pub fn trait_violation_facet_scale(facet_value: f32, threshold: f32) -> f32 {
    if facet_value <= threshold {
        return 1.0;
    }
    let denom = (1.0 - threshold).max(0.000_001);
    (facet_value - threshold) / denom
}

/// Intrusive-thought trigger chance from PTSD multiplier and history decay.
pub fn trait_violation_intrusive_chance(
    base_chance: f32,
    ptsd_mult: f32,
    ticks_since: i32,
    history_decay_ticks: i32,
    has_trauma_scars: bool,
) -> f32 {
    if ptsd_mult < 1.4 {
        return 0.0;
    }
    let decay_div = history_decay_ticks.max(1) as f32;
    let decay_factor = (-(ticks_since.max(0) as f32) / decay_div).exp();
    let mut chance = base_chance * (ptsd_mult - 1.0) * decay_factor;
    if has_trauma_scars {
        chance *= 2.0;
    }
    chance
}

/// Trauma-scar acquisition chance with kindling amplification and global scaling.
pub fn trauma_scar_acquire_chance(
    base_chance: f32,
    chance_scale: f32,
    existing_stacks: i32,
    kindling_factor: f32,
) -> f32 {
    let mut chance = base_chance * chance_scale;
    if existing_stacks > 0 {
        chance *= 1.0 + kindling_factor * existing_stacks as f32;
    }
    clamp_f32(chance, 0.0, 1.0)
}

/// Trauma-scar stress-sensitivity factor for one scar entry.
pub fn trauma_scar_sensitivity_factor(base_mult: f32, stacks: i32) -> f32 {
    let safe_stacks = stacks.max(1) as f32;
    let delta = base_mult - 1.0;
    1.0 + delta * (1.0 + 0.5 * (safe_stacks - 1.0))
}

/// Memory intensity decay step: `intensity * exp(-rate * dt_years)`.
pub fn memory_decay_intensity(intensity: f32, rate: f32, dt_years: f32) -> f32 {
    intensity * (-rate * dt_years).exp()
}

/// Batch memory intensity decay for paired `(intensity, rate)` inputs.
pub fn memory_decay_batch(intensities: &[f32], rates: &[f32], dt_years: f32) -> Vec<f32> {
    let len = intensities.len().min(rates.len());
    let mut out = Vec::with_capacity(len);
    for idx in 0..len {
        out.push(memory_decay_intensity(
            intensities[idx],
            rates[idx],
            dt_years,
        ));
    }
    out
}

/// Summary intensity for compressed memory entries.
pub fn memory_summary_intensity(max_intensity: f32, summary_scale: f32) -> f32 {
    max_intensity * summary_scale
}

/// Attachment type classifier code.
///
/// Returns:
/// `0=secure`, `1=anxious`, `2=avoidant`, `3=disorganized`.
pub fn attachment_type_code(
    sensitivity: f32,
    consistency: f32,
    ace_score: f32,
    abuser_is_caregiver: bool,
    sensitivity_threshold_secure: f32,
    consistency_threshold_secure: f32,
    sensitivity_threshold_anxious: f32,
    consistency_threshold_disorganized: f32,
    abuser_is_caregiver_ace_min: f32,
    avoidant_sensitivity_max: f32,
    avoidant_consistency_min: f32,
) -> i32 {
    if sensitivity >= sensitivity_threshold_secure && consistency >= consistency_threshold_secure {
        return 0;
    }
    if sensitivity >= sensitivity_threshold_anxious
        && consistency < consistency_threshold_disorganized
    {
        return 1;
    }
    if sensitivity < avoidant_sensitivity_max && consistency >= avoidant_consistency_min {
        return 2;
    }
    if ace_score >= abuser_is_caregiver_ace_min && abuser_is_caregiver {
        return 3;
    }
    1
}

/// Parenting raw quality from personality, stress burden, break state, and ACE score.
pub fn attachment_raw_parenting_quality(
    has_personality: bool,
    a_axis: f32,
    e_axis: f32,
    has_emotion_data: bool,
    stress: f32,
    allostatic: f32,
    has_active_break: bool,
    ace_score: f32,
) -> f32 {
    let mut base = 0.50_f32;
    if has_personality {
        base += 0.15 * a_axis;
        base += 0.10 * e_axis;
    }
    if has_emotion_data {
        base -= 0.20 * clamp_f32(stress / 2000.0, 0.0, 1.0);
        base -= 0.15 * clamp_f32(allostatic / 100.0, 0.0, 1.0);
    }
    if has_active_break {
        base -= 0.30;
    }
    base -= 0.10 * clamp_f32(ace_score / 10.0, 0.0, 1.0);
    clamp_f32(base, 0.0, 1.0)
}

/// Substance coping effect step for parenting quality and side-effect accumulators.
///
/// Returns `[new_quality, new_neglect_chance, new_consistency_penalty]`.
pub fn attachment_coping_quality_step(
    base_quality: f32,
    dependency: f32,
    neglect_chance: f32,
    consistency_penalty: f32,
) -> [f32; 3] {
    let dep = clamp_f32(dependency, 0.0, 1.0);
    let quality = clamp_f32(base_quality - (0.10 + 0.15 * dep), 0.0, 1.0);
    let new_neglect = neglect_chance + 0.02 * (1.0 + dep);
    let new_consistency = consistency_penalty + 0.15;
    [quality, new_neglect, new_consistency]
}

/// Attachment protective factor from secure-bond bonus and emotional health term.
pub fn attachment_protective_factor(
    is_secure: bool,
    eh: f32,
    secure_weight: f32,
    eh_weight: f32,
    max_pf: f32,
) -> f32 {
    let mut pf = 0.0_f32;
    if is_secure {
        pf += secure_weight;
    }
    pf += eh_weight * eh;
    clamp_f32(pf, 0.0, max_pf)
}

/// Normalized scar index from scar count.
pub fn intergen_scar_index(scar_count: i32, norm_divisor: f32) -> f32 {
    let divisor = if norm_divisor <= 0.0 {
        1.0
    } else {
        norm_divisor
    };
    clamp_f32((scar_count.max(0) as f32) / divisor, 0.0, 1.0)
}

/// Intergenerational epigenetic load step.
///
/// Input order:
/// `[epi_m, allo_m, scar_m, mw_epi, mw_allo, mw_scar,
///   epi_f, allo_f, scar_f, fw_epi, fw_allo, fw_scar,
///   base_t, max_t, bonus_t, adversity, preg_stress, malnutrition,
///   prenatal_w_stress, prenatal_w_malnutrition, prenatal_max,
///   baseline, maternal_weight, paternal_weight]`
///
/// Returns `[child_epi_load, transmission_rate]`.
pub fn intergen_child_epigenetic_step(inputs: &[f32]) -> [f32; 2] {
    if inputs.len() < 24 {
        return [0.05, 0.30];
    }
    let epi_m = inputs[0];
    let allo_m = inputs[1];
    let scar_m = inputs[2];
    let mw_epi = inputs[3];
    let mw_allo = inputs[4];
    let mw_scar = inputs[5];

    let epi_f = inputs[6];
    let allo_f = inputs[7];
    let scar_f = inputs[8];
    let fw_epi = inputs[9];
    let fw_allo = inputs[10];
    let fw_scar = inputs[11];

    let base_t = inputs[12];
    let max_t = inputs[13];
    let bonus_t = inputs[14];
    let adversity = inputs[15];
    let preg_stress = inputs[16];
    let malnutrition = inputs[17];
    let prenatal_w_stress = inputs[18];
    let prenatal_w_malnutrition = inputs[19];
    let prenatal_max = inputs[20];
    let baseline = inputs[21];
    let maternal_weight = inputs[22];
    let paternal_weight = inputs[23];

    let mother_state = clamp_f32(
        mw_epi * epi_m + mw_allo * allo_m + mw_scar * scar_m,
        0.0,
        1.0,
    );
    let father_state = clamp_f32(
        fw_epi * epi_f + fw_allo * allo_f + fw_scar * scar_f,
        0.0,
        1.0,
    );

    let transmission_rate = clamp_f32(base_t + bonus_t * adversity, base_t, max_t);
    let prenatal = clamp_f32(
        prenatal_w_stress * preg_stress + prenatal_w_malnutrition * malnutrition,
        0.0,
        prenatal_max,
    );
    let child = clamp_f32(
        baseline
            + transmission_rate * (maternal_weight * mother_state + paternal_weight * father_state)
            + prenatal,
        0.0,
        1.0,
    );
    [child, transmission_rate]
}

/// HPA sensitivity multiplier from epigenetic load.
pub fn intergen_hpa_sensitivity(epigenetic_load: f32, hpa_load_weight: f32) -> f32 {
    1.0 + epigenetic_load * hpa_load_weight
}

/// Meaney-style repair update for epigenetic load.
pub fn intergen_meaney_repair_load(
    current_load: f32,
    parenting_quality: f32,
    threshold: f32,
    repair_rate: f32,
    min_load: f32,
) -> f32 {
    if parenting_quality < threshold {
        return current_load;
    }
    (current_load - repair_rate * (parenting_quality - threshold) * 2.0).max(min_load)
}

/// Parenting adulthood stress gain multiplier after epigenetic HPA sensitivity adjustment.
pub fn parenting_hpa_adjusted_stress_gain(
    current_stress_mult: f32,
    epigenetic_load: f32,
    hpa_load_weight: f32,
) -> f32 {
    current_stress_mult * intergen_hpa_sensitivity(epigenetic_load, hpa_load_weight)
}

/// Bandura observational learning base rate for coping familiarity update.
pub fn parenting_bandura_base_rate(
    base_coeff: f32,
    coping_mult: f32,
    observation_strength: f32,
    is_maladaptive: bool,
    maladaptive_multiplier: f32,
) -> f32 {
    let mut rate = base_coeff * coping_mult * observation_strength;
    if is_maladaptive {
        rate *= maladaptive_multiplier;
    }
    rate
}

/// Next ACE partial score after applying one event.
pub fn ace_partial_score_next(current_partial: f32, severity: f32, ace_weight: f32) -> f32 {
    clamp_f32(current_partial + severity.max(0.0) * ace_weight, 0.0, 1.0)
}

/// Aggregate ACE total from all item partial scores.
pub fn ace_score_total_from_partials(partials: &[f32]) -> f32 {
    let mut total = 0.0_f32;
    for partial in partials {
        total += *partial;
    }
    clamp_f32(total, 0.0, 10.0)
}

/// Aggregate threat/deprivation totals from item partial scores.
///
/// `type_codes` uses:
/// - `1`: threat
/// - `2`: deprivation
/// - any other value: ignored
///
/// Returns `[threat_total, deprivation_total]`.
pub fn ace_threat_deprivation_totals(partials: &[f32], type_codes: &[i32]) -> [f32; 2] {
    let len = partials.len().min(type_codes.len());
    let mut threat = 0.0_f32;
    let mut deprivation = 0.0_f32;
    for idx in 0..len {
        match type_codes[idx] {
            1 => threat += partials[idx],
            2 => deprivation += partials[idx],
            _ => {}
        }
    }
    [threat, deprivation]
}

/// ACE adulthood modifier adjustment with protective factor mitigation.
///
/// Returns `[stress_gain_mult, break_threshold_mult, allostatic_base]`.
pub fn ace_adult_modifiers_adjusted(
    base_stress_gain_mult: f32,
    base_break_threshold_mult: f32,
    base_allostatic_base: f32,
    break_floor: f32,
    protective_factor: f32,
) -> [f32; 3] {
    let pf = clamp_f32(protective_factor, 0.0, 1.0);
    let break_mult = base_break_threshold_mult.max(break_floor);
    let stress_gain_mult = 1.0 + (base_stress_gain_mult - 1.0) * (1.0 - pf);
    let break_threshold_mult = 1.0 - (1.0 - break_mult) * (1.0 - pf);
    let allostatic_base = base_allostatic_base * (1.0 - 0.5 * pf);
    [stress_gain_mult, break_threshold_mult, allostatic_base]
}

/// Backfilled ACE score estimate for adults without childhood history.
///
/// `attachment_code`: `0=secure`, `1=anxious`, `2=avoidant`, `3=disorganized`.
pub fn ace_backfill_score(allostatic: f32, trauma_count: i32, attachment_code: i32) -> f32 {
    let disorg_bonus = if attachment_code == 3 { 1.5 } else { 0.0 };
    let insecure_bonus = if attachment_code == 1 || attachment_code == 2 {
        0.7
    } else {
        0.0
    };
    let stress_component = 0.08 * clamp_f32(allostatic, 0.0, 100.0);
    let scar_component = 0.8 * trauma_count.max(0) as f32;
    clamp_f32(
        stress_component + scar_component + disorg_bonus + insecure_bonus,
        0.0,
        10.0,
    )
}

/// Age-based leadership respect score in `[0.0, 1.0]`.
pub fn leader_age_respect(age_years: f32) -> f32 {
    clamp_f32((age_years - 18.0) / 40.0, 0.0, 1.0)
}

/// Weighted leadership score with optional reputation bonus.
pub fn leader_score(
    charisma: f32,
    wisdom: f32,
    trustworthiness: f32,
    intimidation: f32,
    social_capital: f32,
    age_respect: f32,
    w_charisma: f32,
    w_wisdom: f32,
    w_trustworthiness: f32,
    w_intimidation: f32,
    w_social_capital: f32,
    w_age_respect: f32,
    rep_overall: f32,
) -> f32 {
    let base_score = charisma * w_charisma
        + wisdom * w_wisdom
        + trustworthiness * w_trustworthiness
        + intimidation * w_intimidation
        + social_capital * w_social_capital
        + age_respect * w_age_respect;
    base_score * (1.0 + rep_overall * 0.20)
}

/// Social capital score normalized to `[0.0, 1.0+]` before caller-side clamp.
pub fn network_social_capital_norm(
    strong_count: f32,
    weak_count: f32,
    bridge_count: f32,
    rep_score: f32,
    strong_weight: f32,
    weak_weight: f32,
    bridge_weight: f32,
    rep_weight: f32,
    norm_div: f32,
) -> f32 {
    let raw = strong_count * strong_weight
        + weak_count * weak_weight
        + bridge_count * bridge_weight
        + rep_score * rep_weight;
    if norm_div <= 0.0 {
        raw
    } else {
        raw / norm_div
    }
}

/// Composite revolution risk from five equal-weight components.
pub fn revolution_risk_score(
    unhappiness: f32,
    frustration: f32,
    inequality: f32,
    leader_unpopularity: f32,
    independence_ratio: f32,
) -> f32 {
    (unhappiness + frustration + inequality + leader_unpopularity + independence_ratio) / 5.0
}

/// Reputation event delta after valence/magnitude scaling and negativity bias.
pub fn reputation_event_delta(
    valence: f32,
    magnitude: f32,
    delta_scale: f32,
    neg_bias: f32,
) -> f32 {
    valence * magnitude * delta_scale * neg_bias
}

/// Reputation decay step preserving sign-specific retention.
pub fn reputation_decay_value(value: f32, pos_decay: f32, neg_decay: f32) -> f32 {
    let decay = if value < 0.0 { neg_decay } else { pos_decay };
    value * decay
}

/// Economic tendency step:
/// returns `[saving, risk, generosity, materialism]` in `[-1.0, 1.0]`.
#[allow(clippy::too_many_arguments)]
pub fn economic_tendencies_step(
    h: f32,
    e: f32,
    x: f32,
    a: f32,
    c: f32,
    o: f32,
    age_years: f32,
    val_self_control: f32,
    val_law: f32,
    val_commerce: f32,
    val_competition: f32,
    val_martial_prowess: f32,
    val_sacrifice: f32,
    val_cooperation: f32,
    val_family: f32,
    val_power: f32,
    val_fairness: f32,
    belonging: f32,
    wealth_norm: f32,
    culture_gen: f32,
    culture_mat: f32,
    is_male: bool,
    wealth_generosity_penalty: f32,
) -> [f32; 4] {
    let age_factor = 1.0 / (1.0 + (-(age_years - 22.0) / 10.0).exp());

    let saving = clamp_f32(
        bipolar(c) * 0.40
            + val_self_control * 0.20
            + bipolar(e) * 0.15
            + bipolar(age_factor) * 0.10
            + val_law * 0.10
            + (-val_commerce) * 0.05,
        -1.0,
        1.0,
    );

    let mut risk = clamp_f32(
        -bipolar(e) * 0.25
            + bipolar(x) * 0.20
            + -bipolar(c) * 0.20
            + bipolar(o) * 0.15
            + val_competition * 0.10
            + val_martial_prowess * 0.05
            + -bipolar(age_factor) * 0.05,
        -1.0,
        1.0,
    );
    if is_male {
        risk = clamp_f32(risk + 0.06, -1.0, 1.0);
    }

    let mut generosity = clamp_f32(
        bipolar(h) * 0.25
            + bipolar(a) * 0.20
            + val_sacrifice * 0.20
            + val_cooperation * 0.15
            + bipolar(belonging) * 0.10
            + val_family * 0.05
            + culture_gen * 0.05,
        -1.0,
        1.0,
    );
    if wealth_norm > 0.80 {
        generosity *= wealth_generosity_penalty;
    }

    let materialism = clamp_f32(
        -bipolar(h) * 0.30
            + val_commerce * 0.20
            + val_power * 0.15
            + -val_fairness * 0.10
            + bipolar(wealth_norm) * 0.10
            + val_competition * 0.10
            + culture_mat * 0.05,
        -1.0,
        1.0,
    );

    [saving, risk, generosity, materialism]
}

/// Gini coefficient from a slice of wealth-like values.
pub fn stratification_gini(values: &[f32]) -> f32 {
    let n = values.len();
    if n < 2 {
        return 0.0;
    }

    let mut sorted_vals = values.to_vec();
    sorted_vals.sort_by(|a, b| a.total_cmp(b));

    let mut sum_diff = 0.0_f32;
    let mut total = 0.0_f32;
    for (i, value) in sorted_vals.iter().enumerate() {
        total += *value;
        sum_diff += (2.0 * i as f32 - n as f32 + 1.0) * *value;
    }
    if total < 0.001 {
        return 0.0;
    }
    clamp_f32(sum_diff / (n as f32 * total), 0.0, 1.0)
}

/// Composite status score for stratification monitor.
#[allow(clippy::too_many_arguments)]
pub fn stratification_status_score(
    rep_overall: f32,
    wealth_norm: f32,
    leader_bonus: f32,
    age_years: f32,
    rep_competence: f32,
    w_reputation: f32,
    w_wealth: f32,
    w_leader: f32,
    w_age: f32,
    w_competence: f32,
) -> f32 {
    let age_respect = clamp_f32((age_years - 30.0) / 40.0, 0.0, 1.0);
    clamp_f32(
        rep_overall * w_reputation
            + wealth_norm * w_wealth
            + leader_bonus * w_leader
            + age_respect * w_age
            + rep_competence * w_competence,
        -1.0,
        1.0,
    )
}

/// Wealth score used by stratification monitor from normalized resources.
pub fn stratification_wealth_score(
    food_days: f32,
    wood_norm: f32,
    stone_norm: f32,
    w_food: f32,
    w_wood: f32,
    w_stone: f32,
) -> f32 {
    let safe_food = food_days.max(0.0);
    let safe_wood = wood_norm.max(0.0);
    let safe_stone = stone_norm.max(0.0);
    w_food * (1.0 + safe_food).ln()
        + w_wood * (1.0 + 10.0 * safe_wood).ln()
        + w_stone * (1.0 + 10.0 * safe_stone).ln()
}

/// Age-based value plasticity in `[0.10, 1.0]`.
pub fn value_plasticity(age_years: f32) -> f32 {
    if age_years < 7.0 {
        return 1.0;
    }
    if age_years < 15.0 {
        let t = (age_years - 7.0) / 8.0;
        return 1.0 + (0.70 - 1.0) * t;
    }
    if age_years < 25.0 {
        let t = (age_years - 15.0) / 10.0;
        return 0.70 + (0.30 - 0.70) * t;
    }
    if age_years < 50.0 {
        let t = (age_years - 25.0) / 25.0;
        return 0.30 + (0.10 - 0.30) * t;
    }
    0.10
}

/// Newborn health score from gestation, nutrition, maternal age, and genetics.
pub fn family_newborn_health(
    gestation_weeks: i32,
    mother_nutrition: f32,
    mother_age: f32,
    genetics_z: f32,
    tech: f32,
) -> f32 {
    let tech_t = clamp_f32(tech / 10.0, 0.0, 1.0);
    let w50 = 35.0 + (24.0 - 35.0) * tech_t;
    let survival_base = 1.0 / (1.0 + (-(gestation_weeks as f32 - w50) / 2.0).exp());

    let damage = if gestation_weeks < 28 {
        0.9 + (0.3 - 0.9) * tech_t
    } else if gestation_weeks < 32 {
        0.5 + (0.1 - 0.5) * tech_t
    } else if gestation_weeks < 37 {
        0.2 + (0.02 - 0.2) * tech_t
    } else {
        0.01
    };

    let nutrition_factor = 0.6 + (1.1 - 0.6) * clamp_f32(mother_nutrition, 0.0, 1.0);
    let age_factor = if !(16.0..=45.0).contains(&mother_age) {
        0.7
    } else if !(18.0..=40.0).contains(&mother_age) {
        0.85
    } else {
        1.0
    };
    let genetics_factor = clamp_f32(genetics_z, 0.7, 1.3);
    let health = survival_base * (1.0 - damage) * nutrition_factor * age_factor * genetics_factor;
    clamp_f32(health, 0.0, 1.0)
}

/// Returns whether an entity is eligible for elder title by age.
pub fn title_is_elder(age_years: f32, elder_min_age_years: f32) -> bool {
    age_years >= elder_min_age_years
}

/// Returns title tier code from skill level (`0=none`, `1=expert`, `2=master`).
pub fn title_skill_tier(level: i32, expert_level: i32, master_level: i32) -> i32 {
    if level >= master_level {
        2
    } else if level >= expert_level {
        1
    } else {
        0
    }
}

/// Attachment-based affinity multiplier clamped to social bounds.
pub fn social_attachment_affinity_multiplier(a_mult: f32, b_mult: f32) -> f32 {
    clamp_f32(a_mult.min(b_mult), 0.40, 1.60)
}

/// Proposal acceptance probability from romantic interest and compatibility.
pub fn social_proposal_accept_prob(romantic_interest: f32, compatibility: f32) -> f32 {
    clamp_f32((romantic_interest / 100.0) * compatibility, 0.0, 1.0)
}

/// Tension scarcity pressure for settlement pair.
pub fn tension_scarcity_pressure(
    s1_deficit: bool,
    s2_deficit: bool,
    per_shared_resource: f32,
) -> f32 {
    if s1_deficit || s2_deficit {
        per_shared_resource * 2.0
    } else {
        0.0
    }
}

/// Next tension value after scarcity pressure and natural decay.
pub fn tension_next_value(
    current: f32,
    scarcity_pressure: f32,
    decay_per_year: f32,
    dt_years: f32,
) -> f32 {
    let decay = decay_per_year * dt_years;
    clamp_f32(current + scarcity_pressure - decay, 0.0, 1.0)
}

/// Resource regeneration next value: `min(current + rate, cap)` with guards.
pub fn resource_regen_next(current: f32, cap: f32, rate: f32) -> f32 {
    if cap <= 0.0 || rate <= 0.0 || current >= cap {
        return current;
    }
    (current + rate).min(cap)
}

/// Age-system derived movement speed from realized agility.
pub fn age_body_speed(agi_realized: i32, speed_scale: f32, speed_base: f32) -> f32 {
    agi_realized as f32 * speed_scale + speed_base
}

/// Age-system derived strength from realized strength.
pub fn age_body_strength(str_realized: i32) -> f32 {
    str_realized as f32 / 1000.0
}

/// Tech discovery per-check probability from annual components.
#[allow(clippy::too_many_arguments)]
pub fn tech_discovery_prob(
    base: f32,
    pop_bonus: f32,
    knowledge_bonus: f32,
    openness_bonus: f32,
    logical_bonus: f32,
    naturalistic_bonus: f32,
    soft_bonus: f32,
    rediscovery_bonus: f32,
    max_bonus: f32,
    checks_per_year: f32,
) -> f32 {
    let mut annual_total = base
        + pop_bonus
        + knowledge_bonus
        + openness_bonus
        + logical_bonus
        + naturalistic_bonus
        + soft_bonus
        + rediscovery_bonus;
    annual_total = clamp_f32(annual_total, 0.0, base + max_bonus + rediscovery_bonus);
    if checks_per_year <= 1.0 || annual_total >= 1.0 {
        return clamp_f32(annual_total, 0.0, 1.0);
    }
    1.0 - (1.0 - annual_total).powf(1.0 / checks_per_year)
}

/// Migration food-scarcity decision from nearby food and population.
pub fn migration_food_scarce(nearby_food: f32, population: i32, per_capita_threshold: f32) -> bool {
    nearby_food < population as f32 * per_capita_threshold
}

/// Migration attempt gate from pressures and random roll.
pub fn migration_should_attempt(
    overcrowded: bool,
    food_scarce: bool,
    chance_roll: f32,
    migration_chance: f32,
) -> bool {
    overcrowded || food_scarce || chance_roll < migration_chance
}

/// Population-system housing cap from shelter count.
pub fn population_housing_cap(
    total_shelters: i32,
    free_population_cap: i32,
    shelter_capacity_per_building: i32,
) -> i32 {
    if total_shelters <= 0 {
        free_population_cap
    } else {
        total_shelters.max(0) * shelter_capacity_per_building.max(0)
    }
}

/// Population birth blocking rule code.
///
/// Returns:
/// - `0`: birth allowed
/// - `1`: blocked by max entities
/// - `2`: blocked by minimum population gate
/// - `3`: blocked by housing capacity
/// - `4`: blocked by food threshold
pub fn population_birth_block_code(
    alive_count: i32,
    max_entities: i32,
    total_shelters: i32,
    total_food: f32,
    min_population: i32,
    free_population_cap: i32,
    shelter_capacity_per_building: i32,
    food_per_alive: f32,
) -> i32 {
    if alive_count >= max_entities {
        return 1;
    }
    if alive_count < min_population {
        return 2;
    }
    // Births block only when the total housing (free slots + all shelter capacity)
    // is exhausted.  free_population_cap agents need no shelter; each shelter
    // building covers shelter_capacity_per_building additional agents.
    let max_housing =
        free_population_cap + total_shelters.max(0) * shelter_capacity_per_building.max(0);
    if alive_count >= max_housing {
        return 3;
    }
    if total_food < alive_count.max(0) as f32 * food_per_alive {
        return 4;
    }
    0
}

/// Chronicle prune scheduler gate.
pub fn chronicle_should_prune(
    current_year: i32,
    last_prune_year: i32,
    prune_interval_years: i32,
) -> bool {
    current_year - last_prune_year >= prune_interval_years
}

/// Chronicle cutoff tick for `max_age_years`.
pub fn chronicle_cutoff_tick(current_year: i32, max_age_years: i32, ticks_per_year: i32) -> i32 {
    (current_year - max_age_years) * ticks_per_year
}

/// Chronicle world-event retention rule.
pub fn chronicle_keep_world_event(
    event_tick: i32,
    importance: i32,
    low_cutoff_tick: i32,
    med_cutoff_tick: i32,
) -> bool {
    if importance <= 2 && event_tick < low_cutoff_tick {
        return false;
    }
    if importance == 3 && event_tick < med_cutoff_tick {
        return false;
    }
    true
}

/// Chronicle personal-event retention rule.
pub fn chronicle_keep_personal_event(has_valid_world_tick: bool, importance: i32) -> bool {
    has_valid_world_tick || importance >= 4
}

/// Encodes psychology break type string to compact code.
pub fn psychology_break_type_code(break_type: &str) -> i32 {
    match break_type {
        "outrage_violence" => 1,
        "panic" => 2,
        "rage" => 3,
        "shutdown" => 4,
        "purge" => 5,
        _ => 0,
    }
}

/// Decodes psychology break type code to string.
pub fn psychology_break_type_label(code: i32) -> &'static str {
    match code {
        1 => "outrage_violence",
        2 => "panic",
        3 => "rage",
        4 => "shutdown",
        5 => "purge",
        _ => "",
    }
}

/// Coping learn probability during/after mental-break processing.
pub fn coping_learn_probability(
    stress: f32,
    allostatic: f32,
    is_recovery: bool,
    break_count: i32,
    owned_count: i32,
    coping_count_max: f32,
) -> f32 {
    let stress_norm = clamp_f32(stress / 2000.0, 0.0, 1.0);
    let allostatic_norm = clamp_f32(allostatic / 100.0, 0.0, 1.0);
    let n = break_count.max(0) as f32;
    let k_n = 1.0 - (-0.35 * n).exp();

    let safe_count_max = if coping_count_max <= 0.0 {
        1.0
    } else {
        coping_count_max
    };
    let s_n =
        ((1.0 + owned_count.max(0) as f32).ln() / (1.0 + safe_count_max).ln()).clamp(0.0, 1.0);

    let mut logit = -2.5;
    logit += clamp_f32((stress_norm - 0.6) / 0.4, 0.0, 1.0);
    logit += 0.7 * clamp_f32((allostatic_norm - 0.5) / 0.5, 0.0, 1.0);
    logit += if is_recovery { 1.2 } else { 0.0 };
    logit += 1.4 * k_n;
    logit -= 1.1 * s_n;

    let mut p = 1.0 / (1.0 + (-logit).exp());
    p *= if is_recovery { 1.0 } else { 0.30 };
    clamp_f32(p, 0.0, 1.0)
}

/// Softmax categorical pick from score list using `roll01 in [0,1]`.
///
/// Returns selected index or `-1` when input is empty.
pub fn coping_softmax_index(scores: &[f32], roll01: f32) -> i32 {
    if scores.is_empty() {
        return -1;
    }
    let mut max_score = f32::NEG_INFINITY;
    for score in scores {
        max_score = max_score.max(*score);
    }
    let mut exp_scores: Vec<f32> = Vec::with_capacity(scores.len());
    let mut sum = 0.0_f32;
    for score in scores {
        let ev = (*score - max_score).exp();
        exp_scores.push(ev);
        sum += ev;
    }
    if sum <= 0.0 {
        return 0;
    }
    let mut target = clamp_f32(roll01, 0.0, 1.0) * sum;
    for (idx, ev) in exp_scores.iter().enumerate() {
        if target <= *ev {
            return idx as i32;
        }
        target -= *ev;
    }
    (exp_scores.len() - 1) as i32
}

/// Emotion mental-break threshold from conscientiousness z-score.
pub fn emotion_break_threshold(z_c: f32, base_threshold: f32, z_scale: f32) -> f32 {
    base_threshold + z_scale * z_c
}

/// Emotion mental-break trigger probability for one tick.
pub fn emotion_break_trigger_probability(
    stress: f32,
    threshold: f32,
    beta: f32,
    tick_prob: f32,
) -> f32 {
    if beta <= 0.0 {
        return 0.0;
    }
    let p = 1.0 / (1.0 + (-(stress - threshold) / beta).exp());
    clamp_f32(p * tick_prob, 0.0, 1.0)
}

/// Emotion mental-break type code from dominant negative emotion.
///
/// Returns codes compatible with `psychology_break_type_label`:
/// `1=outrage_violence`, `2=panic`, `3=rage`, `4=shutdown`, `5=purge`.
pub fn emotion_break_type_code(
    outrage: f32,
    fear: f32,
    anger: f32,
    sadness: f32,
    disgust: f32,
    outrage_threshold: f32,
) -> i32 {
    if outrage > outrage_threshold {
        return 1;
    }
    let mut code = 4;
    let mut max_val = sadness;
    if fear > max_val {
        max_val = fear;
        code = 2;
    }
    if anger > max_val {
        max_val = anger;
        code = 3;
    }
    if disgust > max_val {
        code = 5;
    }
    code
}

/// Emotion half-life adjustment from trait z-score.
pub fn emotion_adjusted_half_life(base_half_life: f32, coeff: f32, z: f32) -> f32 {
    let base = if base_half_life <= 0.000_001 {
        0.000_001
    } else {
        base_half_life
    };
    (base * (coeff * z).exp()).max(0.000_001)
}

/// Emotion baseline value from linear trait term and clamp band.
pub fn emotion_baseline_value(
    base_value: f32,
    scale_value: f32,
    z: f32,
    min_value: f32,
    max_value: f32,
) -> f32 {
    clamp_f32(base_value + scale_value * z, min_value, max_value)
}

/// Emotion habituation factor from repeated count.
pub fn emotion_habituation_factor(eta: f32, repeat_count: i32) -> f32 {
    (-eta * repeat_count.max(0) as f32).exp()
}

/// Emotion contagion susceptibility from HEXACO E/A z-scores.
pub fn emotion_contagion_susceptibility(z_e: f32, z_a: f32) -> f32 {
    (0.2 * z_e + 0.1 * z_a).exp()
}

/// Emotion contagion distance attenuation factor.
pub fn emotion_contagion_distance_factor(distance: f32, distance_scale: f32) -> f32 {
    if distance_scale <= 0.000_001 {
        return 0.0;
    }
    (-distance / distance_scale).exp()
}

/// Emotion impulse vector from appraisal dimensions and per-emotion sensitivities.
///
/// Input order:
/// `[g, n, c, a, m, p, b, fr, base_intensity,
///   sens_joy, sens_sadness, sens_anger, sens_fear,
///   sens_disgust, sens_surprise, sens_trust, sens_anticipation]`
///
/// Returns `[joy, sadness, anger, fear, disgust, surprise, trust, anticipation]`.
pub fn emotion_event_impulse_from_appraisal(inputs: &[f32]) -> [f32; 8] {
    if inputs.len() < 17 {
        return [0.0; 8];
    }
    let g = inputs[0];
    let n = inputs[1];
    let c = inputs[2];
    let a = inputs[3];
    let m = inputs[4];
    let p = inputs[5];
    let b = inputs[6];
    let fr = inputs[7];
    let base_intensity = inputs[8];
    let sens_joy = inputs[9];
    let sens_sadness = inputs[10];
    let sens_anger = inputs[11];
    let sens_fear = inputs[12];
    let sens_disgust = inputs[13];
    let sens_surprise = inputs[14];
    let sens_trust = inputs[15];
    let sens_anticipation = inputs[16];

    let joy = base_intensity * g.max(0.0) * (1.0 + 0.5 * n) * sens_joy;
    let sadness = base_intensity * (-g).max(0.0) * (1.0 - c) * sens_sadness;
    let anger = base_intensity * (-g).max(0.0) * c * (-a + m).max(0.0) * sens_anger;
    let fear = base_intensity * (-g).max(0.0) * (1.0 - c) * (0.5 + 0.5 * n) * sens_fear;
    let disgust = base_intensity * (p + 0.7 * m) * (0.5 + 0.5 * (-g).max(0.0)) * sens_disgust;
    let surprise = base_intensity * n * sens_surprise;
    let trust = base_intensity * b.max(0.0) * (1.0 - p) * (1.0 - m) * sens_trust;
    let anticipation = base_intensity * fr * (0.5 + 0.5 * g.max(0.0)) * sens_anticipation;

    [
        joy,
        sadness,
        anger,
        fear,
        disgust,
        surprise,
        trust,
        anticipation,
    ]
}

/// Batch variant of `emotion_event_impulse_from_appraisal`.
///
/// Input is a flat buffer with stride 17 per event.
/// Output is a flat buffer with stride 8 per event.
pub fn emotion_event_impulse_batch(flat_inputs: &[f32]) -> Vec<f32> {
    const IN_STRIDE: usize = 17;
    let mut out: Vec<f32> = Vec::new();
    if flat_inputs.len() < IN_STRIDE {
        return out;
    }
    let count = flat_inputs.len() / IN_STRIDE;
    out.reserve(count * 8);
    for idx in 0..count {
        let start = idx * IN_STRIDE;
        let impulse = emotion_event_impulse_from_appraisal(&flat_inputs[start..start + IN_STRIDE]);
        out.extend_from_slice(&impulse);
    }
    out
}

/// Cultural-memory decay step for technology forgetting.
pub fn tech_cultural_memory_decay(
    current_memory: f32,
    base_decay: f32,
    forgotten_long_multiplier: f32,
    memory_floor: f32,
    forgotten_recent: bool,
) -> f32 {
    let decay_rate = if forgotten_recent {
        base_decay
    } else {
        base_decay * forgotten_long_multiplier
    };
    (current_memory - decay_rate).max(memory_floor)
}

/// Clamp stacked tech modifiers (multiplier/additive) within configured caps.
pub fn tech_modifier_stack_clamp(
    multiplier_product: f32,
    additive_sum: f32,
    multiplier_cap: f32,
    additive_cap: f32,
) -> [f32; 2] {
    let mul = clamp_f32(multiplier_product, 0.01, multiplier_cap);
    let add = clamp_f32(additive_sum, -additive_cap, additive_cap);
    [mul, add]
}

/// Age-stage movement skip gate used in movement tick loop.
pub fn movement_should_skip_tick(skip_mod: i32, tick: i32, entity_id: i32) -> bool {
    skip_mod > 0 && (tick + entity_id) % skip_mod == 0
}

/// Campfire social boost selector by day/night.
pub fn building_campfire_social_boost(is_night: bool, day_boost: f32, night_boost: f32) -> f32 {
    if is_night {
        night_boost
    } else {
        day_boost
    }
}

/// Additive capped value update.
pub fn building_add_capped(current: f32, delta: f32, cap: f32) -> f32 {
    (current + delta).min(cap)
}

/// Childcare stockpile withdrawal amount for one stockpile step.
pub fn childcare_take_food(available: f32, remaining: f32) -> f32 {
    if available <= 0.0 || remaining <= 0.0 {
        return 0.0;
    }
    available.min(remaining)
}

/// Childcare hunger update after food withdrawal.
pub fn childcare_hunger_after(
    current_hunger: f32,
    withdrawn: f32,
    food_hunger_restore: f32,
) -> f32 {
    clamp_f32(current_hunger + withdrawn * food_hunger_restore, 0.0, 1.0)
}

/// Culture receptivity modifier for cross-settlement tech propagation.
pub fn tech_propagation_culture_modifier(
    knowledge_avg: f32,
    tradition_avg: f32,
    knowledge_weight: f32,
    tradition_weight: f32,
    min_mod: f32,
    max_mod: f32,
) -> f32 {
    let mut culture_mod = 1.0_f32;
    culture_mod += (knowledge_avg + 1.0) * 0.5 * knowledge_weight;
    culture_mod -= (tradition_avg + 1.0) * 0.5 * tradition_weight;
    clamp_f32(culture_mod, min_mod, max_mod)
}

/// Carrier skill-to-propagation bonus.
pub fn tech_propagation_carrier_bonus(max_skill: i32, skill_divisor: f32, weight: f32) -> f32 {
    if skill_divisor <= 0.0 {
        return 1.0;
    }
    1.0 + (max_skill as f32 / skill_divisor) * weight
}

/// Final cross-settlement propagation probability with upper cap.
pub fn tech_propagation_final_prob(
    base_prob: f32,
    lang_penalty: f32,
    culture_mod: f32,
    carrier_bonus: f32,
    stability_bonus: f32,
    max_prob: f32,
) -> f32 {
    let raw = base_prob * lang_penalty * culture_mod * carrier_bonus * stability_bonus;
    clamp_f32(raw, 0.0, max_prob)
}

/// Siler-model hazard decomposition and check-period death probability.
///
/// Returns `[h_infant, h_background, h_senescence, mu_total, q_annual, q_check]`.
pub fn mortality_hazards_and_prob(
    age_years: f32,
    a1: f32,
    b1: f32,
    a2: f32,
    a3: f32,
    b3: f32,
    tech_k1: f32,
    tech_k2: f32,
    tech_k3: f32,
    tech_level: f32,
    nutrition: f32,
    care_hunger_min: f32,
    care_protection_factor: f32,
    season_infant_mod: f32,
    season_background_mod: f32,
    frailty: f32,
    dr_norm: f32,
    dr_mortality_reduction: f32,
    is_monthly: bool,
) -> [f32; 6] {
    let mu_infant = a1 * (-b1 * age_years).exp();
    let mu_background = a2;
    let mu_senescence = a3 * (b3 * age_years).exp();

    let mut m1 = (-tech_k1 * tech_level).exp();
    let mut m2 = (-tech_k2 * tech_level).exp();
    let m3 = (-tech_k3 * tech_level).exp();

    m1 *= 2.0 + (0.8 - 2.0) * nutrition;
    m2 *= 1.5 + (0.9 - 1.5) * nutrition;
    if age_years <= 2.0 && nutrition > care_hunger_min {
        m1 *= care_protection_factor;
    }

    m1 *= season_infant_mod;
    m2 *= season_background_mod;

    let h_infant = m1 * mu_infant;
    let h_background = m2 * mu_background;
    let h_senescence = m3 * mu_senescence;

    let mut mu_total = h_infant + h_background + h_senescence;
    mu_total *= frailty;
    mu_total *= 1.0 - dr_mortality_reduction * dr_norm;

    let q_annual = clamp_f32(1.0 - (-mu_total).exp(), 0.0, 0.999);
    let q_check = if is_monthly {
        1.0 - (1.0 - q_annual).powf(1.0 / 12.0)
    } else {
        q_annual
    };

    [
        h_infant,
        h_background,
        h_senescence,
        mu_total,
        q_annual,
        q_check,
    ]
}

/// Intelligence activity modifier ("use it or lose it").
pub fn cognition_activity_modifier(
    active_skill_count: i32,
    activity_buffer: f32,
    inactivity_accel: f32,
) -> f32 {
    if active_skill_count >= 1 {
        activity_buffer
    } else {
        inactivity_accel
    }
}

/// ACE-based fluid decline multiplier gate.
pub fn cognition_ace_fluid_decline_mult(
    ace_penalty: f32,
    ace_penalty_minor: f32,
    ace_fluid_decline_mult: f32,
) -> f32 {
    if ace_penalty >= ace_penalty_minor {
        ace_fluid_decline_mult
    } else {
        1.0
    }
}

#[inline]
fn maxf32(value: f32) -> f32 {
    value.max(0.0)
}

#[inline]
fn bipolar(value: f32) -> f32 {
    (value - 0.5) * 2.0
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

/// SHRP adjusted intensity and override flag.
///
/// Returns `[adjusted_intensity, override_flag]`.
pub fn child_shrp_step(
    intensity: f32,
    shrp_active: bool,
    shrp_override_threshold: f32,
    vulnerability_mult: f32,
) -> [f32; 2] {
    if !shrp_active {
        return [intensity, 0.0];
    }
    if intensity < shrp_override_threshold {
        return [0.0, 0.0];
    }
    [intensity * vulnerability_mult, 1.0]
}

/// Child stress type classification code.
///
/// Returns `0=positive`, `1=tolerable`, `2=toxic`.
pub fn child_stress_type_code(
    intensity: f32,
    attachment_present: bool,
    attachment_quality: f32,
) -> i32 {
    if intensity < 0.30 {
        0
    } else if attachment_present && attachment_quality > 0.50 {
        1
    } else {
        2
    }
}

/// Applies child stress state update for the classified stress type.
///
/// Returns `[next_resilience, next_reserve, next_stress, next_allostatic, developmental_damage_delta]`.
pub fn child_stress_apply_step(
    resilience: f32,
    reserve: f32,
    stress: f32,
    allostatic: f32,
    intensity: f32,
    spike_mult: f32,
    vulnerability_mult: f32,
    break_threshold_mult: f32,
    stress_type_code: i32,
) -> [f32; 5] {
    let mut next_resilience = resilience;
    let mut next_reserve = reserve;
    let mut next_stress = stress;
    let mut next_allostatic = allostatic;
    let mut developmental_damage_delta = 0.0_f32;

    match stress_type_code {
        0 => {
            next_resilience = clamp_f32(next_resilience + 0.01 * intensity, 0.0, 1.0);
            next_reserve = clamp_f32(next_reserve + 0.5 * intensity, 0.0, 100.0);
        }
        1 => {
            let gas_cost = intensity * spike_mult * 6.0;
            next_reserve = clamp_f32(next_reserve - gas_cost, 0.0, 100.0);
            next_stress = clamp_f32(next_stress + intensity * spike_mult * 8.0, 0.0, 2000.0);
        }
        _ => {
            next_stress = clamp_f32(
                next_stress + intensity * spike_mult * vulnerability_mult * 16.0,
                0.0,
                2000.0,
            );
            next_allostatic = clamp_f32(
                next_allostatic + intensity * vulnerability_mult * 1.5,
                0.0,
                100.0,
            );
            developmental_damage_delta = intensity * break_threshold_mult * 0.02;
        }
    }

    [
        next_resilience,
        next_reserve,
        next_stress,
        next_allostatic,
        developmental_damage_delta,
    ]
}

/// Applies parent-transfer stress to child stress gauge when transfer exceeds threshold.
pub fn child_parent_transfer_apply_step(
    current_stress: f32,
    transferred: f32,
    transfer_threshold: f32,
    transfer_scale: f32,
    stress_clamp_max: f32,
) -> f32 {
    if transferred <= transfer_threshold {
        return current_stress;
    }
    clamp_f32(
        current_stress + transferred * transfer_scale,
        0.0,
        stress_clamp_max,
    )
}

/// Applies deprivation developmental-damage accumulation step.
pub fn child_deprivation_damage_step(current_damage: f32, damage_rate: f32) -> f32 {
    current_damage + damage_rate
}

/// Child developmental stage classification from age ticks.
///
/// Returns `0=infant`, `1=toddler`, `2=child`, `3=teen`, `4=adult`.
pub fn child_stage_code_from_age_ticks(
    age_ticks: i32,
    infant_max_years: f32,
    toddler_max_years: f32,
    child_max_years: f32,
    teen_max_years: f32,
) -> i32 {
    let years = (age_ticks as f32) / 8760.0;
    if years < infant_max_years {
        0
    } else if years < toddler_max_years {
        1
    } else if years < child_max_years {
        2
    } else if years < teen_max_years {
        3
    } else {
        4
    }
}

/// Applies rebound stress and hidden-threat accumulator release.
///
/// Returns `[next_stress, next_hidden_threat]`.
pub fn stress_rebound_apply_step(
    stress: f32,
    hidden_threat_accumulator: f32,
    total_rebound: f32,
    stress_clamp_max: f32,
) -> [f32; 2] {
    if total_rebound <= 0.0 {
        return [stress, hidden_threat_accumulator];
    }
    [
        clamp_f32(stress + total_rebound, 0.0, stress_clamp_max),
        maxf32(hidden_threat_accumulator - total_rebound),
    ]
}

/// Applies event stress injection outputs to current stress gauge.
///
/// Returns `[next_stress, append_trace_flag]`.
pub fn stress_injection_apply_step(
    stress: f32,
    final_instant: f32,
    final_per_tick: f32,
    trace_threshold: f32,
    stress_clamp_max: f32,
) -> [f32; 2] {
    let next_stress = clamp_f32(stress + final_instant, 0.0, stress_clamp_max);
    let append_flag = if final_per_tick.abs() > trace_threshold {
        1.0
    } else {
        0.0
    };
    [next_stress, append_flag]
}

/// Updates shaken countdown and returns whether work penalty should be cleared.
///
/// Returns `[next_shaken_remaining, clear_penalty_flag]`.
pub fn stress_shaken_countdown_step(shaken_remaining: i32) -> [f32; 2] {
    if shaken_remaining <= 0 {
        return [0.0, 0.0];
    }
    let next_remaining = (shaken_remaining - 1).max(0);
    let clear_penalty = if next_remaining <= 0 { 1.0 } else { 0.0 };
    [next_remaining as f32, clear_penalty]
}

/// Computes stress support score from relationship strength samples.
///
/// Strength samples are expected in `[0, 1]` and clamped defensively.
/// Empty input returns baseline support `0.3`.
pub fn stress_support_score(strengths: &[f32]) -> f32 {
    if strengths.is_empty() {
        return 0.3;
    }

    let mut strong = 0.0_f32;
    let mut weak_sum = 0.0_f32;

    for raw in strengths {
        let strength = clamp_f32(*raw, 0.0, 1.0);
        if strength > strong {
            weak_sum += strong;
            strong = strength;
        } else {
            weak_sum += strength;
        }
    }

    clamp_f32(
        0.65 * strong + 0.35 * (1.0 - (-weak_sum / 1.5).exp()),
        0.0,
        1.0,
    )
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
        ace_adult_modifiers_adjusted, ace_backfill_score, ace_partial_score_next,
        ace_score_total_from_partials, ace_threat_deprivation_totals, action_energy_cost,
        age_body_speed, age_body_strength, age_trainability_modifier, age_trainability_modifiers,
        anxious_attachment_stress_delta, attachment_coping_quality_step,
        attachment_protective_factor, attachment_raw_parenting_quality, attachment_type_code,
        building_add_capped, building_campfire_social_boost, calc_realized_values,
        calc_training_gain, calc_training_gains, child_deprivation_damage_step,
        child_parent_stress_transfer, child_parent_transfer_apply_step, child_shrp_step,
        child_simultaneous_ace_step, child_social_buffered_intensity,
        child_stage_code_from_age_ticks, child_stress_apply_step, child_stress_type_code,
        childcare_hunger_after, childcare_take_food, chronicle_cutoff_tick,
        chronicle_keep_personal_event, chronicle_keep_world_event, chronicle_should_prune,
        cognition_ace_fluid_decline_mult, cognition_activity_modifier, compute_age_curve,
        compute_age_curves, contagion_aoe_total_susceptibility, contagion_network_delta,
        contagion_spiral_increment, contagion_stress_delta, coping_learn_probability,
        coping_softmax_index, critical_severity, economic_tendencies_step,
        emotion_adjusted_half_life, emotion_baseline_value, emotion_break_threshold,
        emotion_break_trigger_probability, emotion_break_type_code,
        emotion_contagion_distance_factor, emotion_contagion_susceptibility,
        emotion_event_impulse_batch, emotion_event_impulse_from_appraisal,
        emotion_habituation_factor, erg_frustration_step, family_newborn_health,
        intelligence_effective_value, intelligence_g_value, intergen_child_epigenetic_step,
        intergen_hpa_sensitivity, intergen_meaney_repair_load, intergen_scar_index,
        job_assignment_best_job_code, job_assignment_rebalance_codes, job_satisfaction_score,
        job_satisfaction_score_batch, leader_age_respect, leader_score, memory_decay_batch,
        memory_decay_intensity, memory_summary_intensity, mental_break_chance,
        mental_break_threshold, migration_food_scarce, migration_should_attempt,
        morale_behavior_weight_multiplier, morale_migration_probability,
        mortality_hazards_and_prob, movement_should_skip_tick, needs_base_decay_step,
        needs_critical_severity_step, network_social_capital_norm, occupation_best_skill_index,
        occupation_should_switch, parenting_bandura_base_rate, parenting_hpa_adjusted_stress_gain,
        personality_child_axis_z, personality_linear_target, population_birth_block_code,
        population_housing_cap, psychology_break_type_code, psychology_break_type_label,
        reputation_decay_value, reputation_event_delta, resource_regen_next, rest_energy_recovery,
        revolution_risk_score, social_attachment_affinity_multiplier, social_proposal_accept_prob,
        stat_sync_derived_scores, stat_threshold_is_active, stats_resource_deltas_per_100,
        stratification_gini, stratification_status_score, stratification_wealth_score,
        stress_injection_apply_step, stress_rebound_apply_step, stress_shaken_countdown_step,
        stress_support_score, tech_cultural_memory_decay, tech_discovery_prob,
        tech_modifier_stack_clamp, tech_propagation_carrier_bonus,
        tech_propagation_culture_modifier, tech_propagation_final_prob, tension_next_value,
        tension_scarcity_pressure, thirst_decay, title_is_elder, title_skill_tier,
        trait_violation_context_modifier, trait_violation_facet_scale,
        trait_violation_intrusive_chance, trauma_scar_acquire_chance,
        trauma_scar_sensitivity_factor, upper_needs_best_skill_normalized,
        upper_needs_job_alignment, upper_needs_step, value_plasticity, warmth_decay,
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
    fn job_satisfaction_score_matches_weighted_formula_shape() {
        let score = job_satisfaction_score(
            &[0.7, 0.4, 0.5, 0.6, 0.8, 0.3],
            &[0.2, -0.1, 0.0, 0.15, 0.3, 0.1],
            &[0.8, 0.6, 0.4],
            &[0.5, 0.3, 0.2],
            0.75,
            0.6,
            0.7,
            0.5,
            0.4,
            0.45,
            0.35,
            0.25,
            0.2,
            0.2,
        );
        assert!((0.0..=1.0).contains(&score));
        assert!(score > 0.5);
    }

    #[test]
    fn job_satisfaction_score_defaults_midpoint_without_weights() {
        let score = job_satisfaction_score(
            &[0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            &[],
            &[],
            0.5,
            0.5,
            0.5,
            0.5,
            0.5,
            0.5,
            0.25,
            0.25,
            0.25,
            0.25,
        );
        assert_eq!(score, 0.4375);
    }

    #[test]
    fn job_satisfaction_score_batch_matches_single_calls() {
        let personality_actual = [0.7, 0.4, 0.5, 0.6, 0.8, 0.3];
        let personality_ideals_flat = [
            0.2, -0.1, 0.0, 0.15, 0.3, 0.1, 0.1, 0.0, -0.2, 0.1, 0.25, -0.1,
        ];
        let value_actual = [0.8, 0.6, 0.4];
        let value_weights_flat = [0.5, 0.3, 0.2, 0.2, 0.4, 0.4];
        let skill_fits = [0.75, 0.35];
        let autonomy_levels = [0.4, 0.25];
        let prestiges = [0.45, 0.3];

        let batch = job_satisfaction_score_batch(
            &personality_actual,
            &personality_ideals_flat,
            &value_actual,
            &value_weights_flat,
            &skill_fits,
            0.6,
            0.7,
            0.5,
            &autonomy_levels,
            &prestiges,
            0.35,
            0.25,
            0.2,
            0.2,
        );
        assert_eq!(batch.len(), 2);

        let single0 = job_satisfaction_score(
            &personality_actual,
            &personality_ideals_flat[0..6],
            &value_actual,
            &value_weights_flat[0..3],
            skill_fits[0],
            0.6,
            0.7,
            0.5,
            autonomy_levels[0],
            prestiges[0],
            0.35,
            0.25,
            0.2,
            0.2,
        );
        let single1 = job_satisfaction_score(
            &personality_actual,
            &personality_ideals_flat[6..12],
            &value_actual,
            &value_weights_flat[3..6],
            skill_fits[1],
            0.6,
            0.7,
            0.5,
            autonomy_levels[1],
            prestiges[1],
            0.35,
            0.25,
            0.2,
            0.2,
        );

        assert_eq!(batch[0], single0);
        assert_eq!(batch[1], single1);
    }

    #[test]
    fn occupation_best_skill_index_returns_first_max() {
        assert_eq!(occupation_best_skill_index(&[]), -1);
        assert_eq!(occupation_best_skill_index(&[4, 9, 7, 9]), 1);
    }

    #[test]
    fn occupation_should_switch_uses_hysteresis_margin() {
        assert!(occupation_should_switch(65, 50, 0.1));
        assert!(!occupation_should_switch(56, 50, 0.1));
    }

    #[test]
    fn job_assignment_best_job_code_picks_largest_deficit() {
        let ratios = [0.5, 0.2, 0.2, 0.1];
        let counts = [10, 2, 2, 1];
        assert_eq!(job_assignment_best_job_code(&ratios, &counts, 20), 1);
    }

    #[test]
    fn job_assignment_rebalance_codes_respects_threshold() {
        let ratios = [0.5, 0.2, 0.2, 0.1];
        let balanced_counts = [10, 4, 4, 2];
        assert_eq!(
            job_assignment_rebalance_codes(&ratios, &balanced_counts, 20, 1.5),
            [-1, -1]
        );
        let skewed_counts = [14, 2, 2, 2];
        assert_eq!(
            job_assignment_rebalance_codes(&ratios, &skewed_counts, 20, 1.5),
            [0, 1]
        );
    }

    #[test]
    fn stat_threshold_is_active_handles_below_and_hysteresis() {
        assert!(stat_threshold_is_active(4, 5, 0, 2, false));
        assert!(!stat_threshold_is_active(5, 5, 0, 2, false));
        assert!(stat_threshold_is_active(6, 5, 0, 2, true));
        assert!(!stat_threshold_is_active(7, 5, 0, 2, true));
    }

    #[test]
    fn stat_threshold_is_active_handles_above_and_hysteresis() {
        assert!(stat_threshold_is_active(6, 5, 1, 2, false));
        assert!(!stat_threshold_is_active(5, 5, 1, 2, false));
        assert!(stat_threshold_is_active(4, 5, 1, 2, true));
        assert!(!stat_threshold_is_active(3, 5, 1, 2, true));
    }

    #[test]
    fn stats_resource_deltas_per_100_scales_by_tick_diff() {
        let out = stats_resource_deltas_per_100(120.0, 55.0, 20.0, 100.0, 40.0, 10.0, 200.0);
        assert!((out[0] - 10.0).abs() < 1e-6);
        assert!((out[1] - 7.5).abs() < 1e-6);
        assert!((out[2] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn stats_resource_deltas_per_100_returns_zero_when_tick_diff_invalid() {
        let out = stats_resource_deltas_per_100(10.0, 10.0, 10.0, 5.0, 5.0, 5.0, 0.0);
        assert_eq!(out, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn personality_linear_target_clamps_by_age_window() {
        assert!((personality_linear_target(10, 1.0, 18, 60) - 0.0).abs() < 1e-6);
        assert!((personality_linear_target(60, 1.0, 18, 60) - 1.0).abs() < 1e-6);
        assert!((personality_linear_target(80, 1.0, 18, 60) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn personality_linear_target_interpolates_linearly() {
        let mid = personality_linear_target(39, 0.8, 18, 60);
        assert!((mid - 0.4).abs() < 1e-6);
    }

    #[test]
    fn intelligence_effective_value_applies_fluid_decline_modifiers() {
        let fluid = intelligence_effective_value(0.8, 0.7, 40.0, true, 0.9, 1.2, 0.1, 0.02, 0.98);
        let cryst = intelligence_effective_value(0.8, 0.7, 40.0, false, 0.9, 1.2, 0.1, 0.02, 0.98);
        assert!(fluid < cryst);
        assert!((0.02..=0.98).contains(&fluid));
    }

    #[test]
    fn intelligence_effective_value_uses_base_when_not_declining() {
        let out = intelligence_effective_value(0.6, 1.05, 20.0, true, 0.9, 1.2, 0.1, 0.02, 0.98);
        assert!((out - 0.525).abs() < 1e-6);
    }

    #[test]
    fn intelligence_g_value_uses_parental_blend_when_available() {
        let with_parents = intelligence_g_value(true, 0.7, 0.5, 0.6, 0.5, 0.6, 0.2, 0.01);
        let without_parents = intelligence_g_value(false, 0.7, 0.5, 0.6, 0.5, 0.6, 0.2, 0.01);
        assert!(with_parents > without_parents);
    }

    #[test]
    fn intelligence_g_value_applies_openness_shift_and_noise() {
        let base = intelligence_g_value(false, 0.0, 0.0, 0.6, 0.5, 0.5, 0.2, 0.0);
        let shifted = intelligence_g_value(false, 0.0, 0.0, 0.6, 0.5, 0.7, 0.2, 0.01);
        assert!((base - 0.5).abs() < 1e-6);
        assert!((shifted - 0.55).abs() < 1e-6);
    }

    #[test]
    fn personality_child_axis_z_applies_inheritance_and_sex_shift() {
        let female = personality_child_axis_z(true, 0.6, 0.2, 0.5, 0.1, true, 0.4, 0.0);
        let male = personality_child_axis_z(true, 0.6, 0.2, 0.5, 0.1, false, 0.4, 0.0);
        assert!(female > male);
    }

    #[test]
    fn personality_child_axis_z_applies_culture_shift() {
        let neutral = personality_child_axis_z(false, 0.0, 0.0, 0.5, 0.2, true, 0.0, 0.0);
        let shifted = personality_child_axis_z(false, 0.0, 0.0, 0.5, 0.2, true, 0.0, 0.3);
        assert!((shifted - neutral - 0.3).abs() < 1e-6);
    }

    #[test]
    fn morale_behavior_weight_multiplier_follows_band_rules() {
        let flourishing = morale_behavior_weight_multiplier(
            0.8, 0.6, 1.2, 1.55, 0.85, 1.2, 0.55, 0.85, 0.30, 0.55,
        );
        let normal = morale_behavior_weight_multiplier(
            0.45, 0.6, 1.2, 1.55, 0.85, 1.2, 0.55, 0.85, 0.30, 0.55,
        );
        let languishing = morale_behavior_weight_multiplier(
            -0.6, 0.6, 1.2, 1.55, 0.85, 1.2, 0.55, 0.85, 0.30, 0.55,
        );
        assert!(flourishing > normal);
        assert!(languishing < normal);
    }

    #[test]
    fn morale_migration_probability_increases_when_morale_drops() {
        let high_morale = morale_migration_probability(0.8, 10.0, 0.35, 0.5, 0.3, 0.95);
        let low_morale = morale_migration_probability(0.2, 10.0, 0.35, 0.5, 0.3, 0.95);
        assert!(low_morale > high_morale);
        assert!((0.0..=0.95).contains(&low_morale));
    }

    #[test]
    fn stat_sync_derived_scores_returns_expected_shape() {
        let inputs = [
            0.6, 0.5, 0.5, 0.4, 0.7, 0.6, 0.6, 0.5, 0.3, 0.7, 0.4, 0.8, 0.5, 0.7, 0.5, 0.6, 0.4,
            0.5, 0.6, 0.7, 0.5, 0.4, 0.6, 0.7, 0.6, 0.5, 0.5, 0.6, 35.0,
        ];
        let out = stat_sync_derived_scores(&inputs);
        assert_eq!(out.len(), 8);
        assert!(out.iter().all(|v| (0.0..=1.0).contains(v)));
    }

    #[test]
    fn stat_sync_derived_scores_handles_short_input() {
        let out = stat_sync_derived_scores(&[0.1, 0.2]);
        assert_eq!(out, [0.0; 8]);
    }

    #[test]
    fn contagion_aoe_total_susceptibility_drops_with_larger_crowd() {
        let small_group = contagion_aoe_total_susceptibility(2, 6.0, false, 0.25, 0.6, 0.5);
        let larger_group = contagion_aoe_total_susceptibility(18, 6.0, false, 0.25, 0.6, 0.5);
        assert!(small_group > larger_group);
    }

    #[test]
    fn contagion_stress_delta_applies_threshold_and_cap() {
        let below_threshold = contagion_stress_delta(8.0, 10.0, 0.04, 1.0, 30.0);
        let capped = contagion_stress_delta(5000.0, 10.0, 0.04, 1.0, 30.0);
        assert_eq!(below_threshold, 0.0);
        assert_eq!(capped, 30.0);
    }

    #[test]
    fn contagion_network_delta_is_clamped_to_band() {
        let delta = contagion_network_delta(20, 6.0, false, 0.25, 0.5, 0.7, 500.0, 0.04, 4.0);
        assert_eq!(delta, 4.0);
    }

    #[test]
    fn contagion_spiral_increment_respects_conditions() {
        let none = contagion_spiral_increment(300.0, -50.0, 500.0, -40.0, 1500.0, 60.0, 3.0, 12.0);
        let active =
            contagion_spiral_increment(900.0, -70.0, 500.0, -40.0, 1500.0, 60.0, 3.0, 12.0);
        assert_eq!(none, 0.0);
        assert!(active > 0.0);
    }

    #[test]
    fn mental_break_threshold_applies_reserve_and_scar_reductions() {
        let baseline = mental_break_threshold(
            520.0, 0.5, 0.5, 0.5, 40.0, 1.0, 1.0, 1.0, 0.0, 420.0, 900.0, 40.0, 0.0,
        );
        let depleted = mental_break_threshold(
            520.0, 0.5, 0.5, 0.5, 40.0, 1.0, 1.0, 1.0, 0.0, 420.0, 900.0, 10.0, 25.0,
        );
        assert!(depleted < baseline);
    }

    #[test]
    fn mental_break_chance_respects_threshold_and_modifiers() {
        let none = mental_break_chance(400.0, 500.0, 100.0, 10.0, 6000.0, 0.25);
        let base = mental_break_chance(900.0, 500.0, 100.0, 10.0, 6000.0, 0.25);
        let amplified = mental_break_chance(900.0, 500.0, 20.0, 70.0, 6000.0, 0.25);
        assert_eq!(none, 0.0);
        assert!(amplified > base);
    }

    #[test]
    fn trait_violation_context_modifier_applies_expected_multipliers() {
        let habitual =
            trait_violation_context_modifier(true, true, true, true, 0.0, 0.5, 0.4, 0.85);
        let contextual =
            trait_violation_context_modifier(false, true, true, true, 0.0, 0.5, 0.4, 0.85);
        assert_eq!(habitual, 0.0);
        assert!((contextual - (0.5 * 0.4 * 0.85)).abs() < 1e-6);
    }

    #[test]
    fn trait_violation_facet_scale_increases_above_threshold() {
        assert_eq!(trait_violation_facet_scale(0.4, 0.6), 1.0);
        assert!((trait_violation_facet_scale(0.9, 0.6) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn trait_violation_intrusive_chance_requires_ptsd_and_decays_over_time() {
        let below = trait_violation_intrusive_chance(0.005, 1.2, 10, 100, false);
        let recent = trait_violation_intrusive_chance(0.005, 1.8, 10, 100, false);
        let old = trait_violation_intrusive_chance(0.005, 1.8, 400, 100, false);
        let scar_boosted = trait_violation_intrusive_chance(0.005, 1.8, 10, 100, true);
        assert_eq!(below, 0.0);
        assert!(recent > old);
        assert!(scar_boosted > recent);
    }

    #[test]
    fn trauma_scar_acquire_chance_scales_and_clamps() {
        let base = trauma_scar_acquire_chance(0.2, 1.0, 0, 0.3);
        let kindled = trauma_scar_acquire_chance(0.2, 1.0, 2, 0.3);
        let capped = trauma_scar_acquire_chance(0.9, 2.0, 5, 0.3);
        assert!(kindled > base);
        assert_eq!(capped, 1.0);
    }

    #[test]
    fn trauma_scar_sensitivity_factor_applies_diminishing_stack_bonus() {
        let one_stack = trauma_scar_sensitivity_factor(1.4, 1);
        let three_stacks = trauma_scar_sensitivity_factor(1.4, 3);
        assert!(three_stacks > one_stack);
        assert!((one_stack - 1.4).abs() < 1e-6);
    }

    #[test]
    fn memory_decay_intensity_matches_exponential_formula() {
        let out = memory_decay_intensity(0.8, 0.5, 1.0);
        assert!((out - (0.8 * (-0.5_f32).exp())).abs() < 1e-6);
    }

    #[test]
    fn memory_decay_batch_uses_pairwise_min_length() {
        let out = memory_decay_batch(&[1.0, 0.5, 0.25], &[0.1, 0.2], 0.5);
        assert_eq!(out.len(), 2);
        assert!(out[0] < 1.0);
        assert!(out[1] < 0.5);
    }

    #[test]
    fn memory_summary_intensity_scales_max_value() {
        assert!((memory_summary_intensity(0.9, 0.7) - 0.63).abs() < 1e-6);
    }

    #[test]
    fn attachment_type_code_follows_threshold_ordering() {
        let secure = attachment_type_code(
            0.8, 0.8, 2.0, false, 0.65, 0.60, 0.40, 0.35, 4.0, 0.35, 0.50,
        );
        let anxious = attachment_type_code(
            0.5, 0.2, 2.0, false, 0.65, 0.60, 0.40, 0.35, 4.0, 0.35, 0.50,
        );
        let avoidant = attachment_type_code(
            0.2, 0.8, 2.0, false, 0.65, 0.60, 0.40, 0.35, 4.0, 0.35, 0.50,
        );
        let disorganized =
            attachment_type_code(0.2, 0.2, 5.0, true, 0.65, 0.60, 0.40, 0.35, 4.0, 0.35, 0.50);
        assert_eq!(secure, 0);
        assert_eq!(anxious, 1);
        assert_eq!(avoidant, 2);
        assert_eq!(disorganized, 3);
    }

    #[test]
    fn attachment_raw_parenting_quality_decreases_with_burden() {
        let low_burden =
            attachment_raw_parenting_quality(true, 0.7, 0.7, true, 200.0, 20.0, false, 1.0);
        let high_burden =
            attachment_raw_parenting_quality(true, 0.7, 0.7, true, 1800.0, 90.0, true, 9.0);
        assert!(low_burden > high_burden);
    }

    #[test]
    fn attachment_coping_quality_step_returns_adjusted_triplet() {
        let out = attachment_coping_quality_step(0.8, 0.5, 0.1, 0.2);
        assert!(out[0] < 0.8);
        assert!(out[1] > 0.1);
        assert!(out[2] > 0.2);
    }

    #[test]
    fn attachment_protective_factor_is_clamped() {
        let secure = attachment_protective_factor(true, 1.0, 0.30, 0.15, 0.45);
        let insecure = attachment_protective_factor(false, 1.0, 0.30, 0.15, 0.45);
        let clamped = attachment_protective_factor(true, 10.0, 0.30, 0.15, 0.45);
        assert!(secure > insecure);
        assert_eq!(clamped, 0.45);
    }

    #[test]
    fn intergen_scar_index_is_normalized() {
        assert_eq!(intergen_scar_index(0, 5.0), 0.0);
        assert_eq!(intergen_scar_index(5, 5.0), 1.0);
        assert_eq!(intergen_scar_index(7, 5.0), 1.0);
    }

    #[test]
    fn intergen_child_epigenetic_step_returns_load_and_t() {
        let inputs = [
            0.2, 0.3, 0.1, 0.5, 0.3, 0.2, 0.1, 0.2, 0.1, 0.6, 0.25, 0.15, 0.3, 0.4, 0.1, 0.5, 0.4,
            0.2, 0.25, 0.10, 0.35, 0.05, 0.65, 0.35,
        ];
        let out = intergen_child_epigenetic_step(&inputs);
        assert_eq!(out.len(), 2);
        assert!((0.0..=1.0).contains(&out[0]));
        assert!((0.3..=0.4).contains(&out[1]));
    }

    #[test]
    fn intergen_hpa_sensitivity_scales_with_load() {
        let low = intergen_hpa_sensitivity(0.1, 0.6);
        let high = intergen_hpa_sensitivity(0.8, 0.6);
        assert!(high > low);
    }

    #[test]
    fn intergen_meaney_repair_load_applies_threshold_and_floor() {
        let unchanged = intergen_meaney_repair_load(0.4, 0.5, 0.7, 0.002, 0.05);
        let repaired = intergen_meaney_repair_load(0.4, 0.9, 0.7, 0.002, 0.05);
        let floored = intergen_meaney_repair_load(0.051, 1.0, 0.7, 10.0, 0.05);
        assert_eq!(unchanged, 0.4);
        assert!(repaired < unchanged);
        assert_eq!(floored, 0.05);
    }

    #[test]
    fn parenting_hpa_adjusted_stress_gain_tracks_epigenetic_load() {
        let low = parenting_hpa_adjusted_stress_gain(1.2, 0.1, 0.6);
        let high = parenting_hpa_adjusted_stress_gain(1.2, 0.8, 0.6);
        assert!(high > low);
    }

    #[test]
    fn parenting_bandura_base_rate_applies_maladaptive_multiplier() {
        let adaptive = parenting_bandura_base_rate(0.002, 1.1, 0.7, false, 1.5);
        let maladaptive = parenting_bandura_base_rate(0.002, 1.1, 0.7, true, 1.5);
        assert!(maladaptive > adaptive);
    }

    #[test]
    fn ace_partial_score_next_clamps_to_unit_interval() {
        let next = ace_partial_score_next(0.9, 0.8, 0.5);
        let unchanged = ace_partial_score_next(0.4, -1.0, 0.7);
        assert_eq!(next, 1.0);
        assert_eq!(unchanged, 0.4);
    }

    #[test]
    fn ace_score_total_from_partials_clamps_to_ten() {
        let total = ace_score_total_from_partials(&[0.5, 0.8, 0.9, 0.3]);
        let capped = ace_score_total_from_partials(&[2.0, 3.0, 4.0, 5.0]);
        assert!((total - 2.5).abs() < 1e-6);
        assert_eq!(capped, 10.0);
    }

    #[test]
    fn ace_threat_deprivation_totals_routes_by_type_code() {
        let out = ace_threat_deprivation_totals(&[0.5, 0.8, 0.2, 1.0, 0.1], &[1, 2, 1, 0, 2]);
        assert!((out[0] - 0.7).abs() < 1e-6);
        assert!((out[1] - 0.9).abs() < 1e-6);
    }

    #[test]
    fn ace_adult_modifiers_adjusted_applies_floor_and_protective_factor() {
        let out = ace_adult_modifiers_adjusted(1.6, 0.3, 20.0, 0.5, 0.4);
        assert!((out[0] - 1.36).abs() < 1e-6);
        assert!((out[1] - 0.7).abs() < 1e-6);
        assert!((out[2] - 16.0).abs() < 1e-6);
    }

    #[test]
    fn ace_backfill_score_accounts_for_attachment_and_scars() {
        let secure = ace_backfill_score(20.0, 1, 0);
        let disorganized = ace_backfill_score(20.0, 1, 3);
        let capped = ace_backfill_score(500.0, 20, 3);
        assert!(disorganized > secure);
        assert_eq!(capped, 10.0);
    }

    #[test]
    fn leader_age_respect_is_clamped() {
        assert_eq!(leader_age_respect(10.0), 0.0);
        assert_eq!(leader_age_respect(18.0), 0.0);
        assert_eq!(leader_age_respect(58.0), 1.0);
        assert_eq!(leader_age_respect(90.0), 1.0);
    }

    #[test]
    fn leader_score_applies_reputation_bonus() {
        let no_bonus = leader_score(
            0.6, 0.5, 0.7, 0.2, 0.4, 0.8, 0.25, 0.15, 0.15, 0.15, 0.15, 0.15, 0.0,
        );
        let with_bonus = leader_score(
            0.6, 0.5, 0.7, 0.2, 0.4, 0.8, 0.25, 0.15, 0.15, 0.15, 0.15, 0.15, 0.4,
        );
        assert!(with_bonus > no_bonus);
    }

    #[test]
    fn network_social_capital_norm_scales_by_weights_and_divisor() {
        let score = network_social_capital_norm(2.0, 3.0, 1.0, 0.5, 3.0, 1.0, 5.0, 10.0, 20.0);
        assert_eq!(
            score,
            (2.0 * 3.0 + 3.0 * 1.0 + 1.0 * 5.0 + 0.5 * 10.0) / 20.0
        );
    }

    #[test]
    fn revolution_risk_score_is_component_average() {
        let risk = revolution_risk_score(0.2, 0.4, 0.6, 0.8, 0.0);
        assert_eq!(risk, 0.4);
    }

    #[test]
    fn reputation_event_delta_applies_negativity_bias() {
        let positive = reputation_event_delta(1.0, 0.5, 0.2, 1.0);
        let negative = reputation_event_delta(-1.0, 0.5, 0.2, 2.0);
        assert_eq!(positive, 0.1);
        assert_eq!(negative, -0.2);
    }

    #[test]
    fn reputation_decay_value_uses_sign_specific_decay() {
        assert_eq!(reputation_decay_value(0.5, 0.9, 0.8), 0.45);
        assert_eq!(reputation_decay_value(-0.5, 0.9, 0.8), -0.4);
    }

    #[test]
    fn economic_tendencies_step_returns_bounded_values() {
        let out = economic_tendencies_step(
            0.6, 0.4, 0.7, 0.5, 0.8, 0.65, 28.0, 0.2, 0.3, 0.1, 0.25, 0.05, 0.4, 0.35, 0.2, 0.3,
            0.1, 0.7, 0.85, 0.2, 0.15, true, 0.8,
        );
        assert!(out.iter().all(|v| (-1.0..=1.0).contains(v)));
    }

    #[test]
    fn economic_tendencies_step_applies_gender_and_wealth_adjustments() {
        let female = economic_tendencies_step(
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 30.0, 0.0, 0.0, 0.0, 0.2, 0.0, 0.4, 0.0, 0.0, 0.0, 0.0,
            0.5, 0.9, 0.0, 0.0, false, 0.5,
        );
        let male = economic_tendencies_step(
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 30.0, 0.0, 0.0, 0.0, 0.2, 0.0, 0.4, 0.0, 0.0, 0.0, 0.0,
            0.5, 0.9, 0.0, 0.0, true, 0.5,
        );
        assert!(male[1] > female[1]);
        assert!(male[2] <= female[2]); // wealth penalty reduces generosity
    }

    #[test]
    fn stratification_gini_detects_inequality() {
        let equal = stratification_gini(&[1.0, 1.0, 1.0, 1.0]);
        let skewed = stratification_gini(&[0.0, 0.0, 0.0, 10.0]);
        assert!(equal <= 0.0001);
        assert!(skewed > 0.7);
    }

    #[test]
    fn stratification_status_score_matches_weighted_formula() {
        let out = stratification_status_score(0.4, 0.7, 0.6, 50.0, 0.3, 0.3, 0.2, 0.1, 0.2, 0.2);
        assert!((out - 0.48).abs() < 1e-6);

        let clipped =
            stratification_status_score(2.0, 2.0, 2.0, 90.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0);
        assert!(clipped <= 1.0);
    }

    #[test]
    fn value_plasticity_decreases_by_age_stage() {
        assert!((value_plasticity(5.0) - 1.0).abs() < 1e-6);
        assert!((value_plasticity(15.0) - 0.70).abs() < 1e-6);
        assert!((value_plasticity(25.0) - 0.30).abs() < 1e-6);
        assert!((value_plasticity(55.0) - 0.10).abs() < 1e-6);
    }

    #[test]
    fn stratification_wealth_score_increases_with_resources() {
        let low = stratification_wealth_score(0.0, 0.0, 0.0, 0.4, 0.3, 0.3);
        let high = stratification_wealth_score(2.0, 1.0, 1.0, 0.4, 0.3, 0.3);
        assert!(low.abs() < 1e-6);
        assert!(high > low);
    }

    #[test]
    fn family_newborn_health_penalizes_preterm_birth() {
        let preterm = family_newborn_health(30, 0.8, 28.0, 1.0, 0.0);
        let term = family_newborn_health(39, 0.8, 28.0, 1.0, 0.0);
        assert!((0.0..=1.0).contains(&preterm));
        assert!((0.0..=1.0).contains(&term));
        assert!(term > preterm);
    }

    #[test]
    fn title_is_elder_uses_min_age_threshold() {
        assert!(title_is_elder(60.0, 55.0));
        assert!(!title_is_elder(40.0, 55.0));
    }

    #[test]
    fn title_skill_tier_prioritizes_master_over_expert() {
        assert_eq!(title_skill_tier(95, 60, 90), 2);
        assert_eq!(title_skill_tier(70, 60, 90), 1);
        assert_eq!(title_skill_tier(40, 60, 90), 0);
    }

    #[test]
    fn social_attachment_affinity_multiplier_clamps_to_bounds() {
        assert!((social_attachment_affinity_multiplier(0.2, 0.3) - 0.4).abs() < 1e-6);
        assert!((social_attachment_affinity_multiplier(2.0, 1.8) - 1.6).abs() < 1e-6);
        assert!((social_attachment_affinity_multiplier(1.2, 0.9) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn social_proposal_accept_prob_is_bounded() {
        assert!((social_proposal_accept_prob(80.0, 0.75) - 0.6).abs() < 1e-6);
        assert!((social_proposal_accept_prob(200.0, 2.0) - 1.0).abs() < 1e-6);
        assert!((social_proposal_accept_prob(-20.0, 1.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn tension_scarcity_pressure_activates_when_any_side_deficit() {
        assert!((tension_scarcity_pressure(false, false, 0.2) - 0.0).abs() < 1e-6);
        assert!((tension_scarcity_pressure(true, false, 0.2) - 0.4).abs() < 1e-6);
        assert!((tension_scarcity_pressure(false, true, 0.2) - 0.4).abs() < 1e-6);
    }

    #[test]
    fn tension_next_value_applies_decay_and_clamp() {
        assert!((tension_next_value(0.5, 0.1, 0.2, 0.5) - 0.5).abs() < 1e-6);
        assert!((tension_next_value(0.95, 0.3, 0.0, 0.0) - 1.0).abs() < 1e-6);
        assert!((tension_next_value(0.1, 0.0, 1.0, 0.5) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn resource_regen_next_applies_cap_and_guards() {
        assert!((resource_regen_next(2.0, 10.0, 0.5) - 2.5).abs() < 1e-6);
        assert!((resource_regen_next(9.9, 10.0, 0.5) - 10.0).abs() < 1e-6);
        assert!((resource_regen_next(5.0, 0.0, 0.5) - 5.0).abs() < 1e-6);
        assert!((resource_regen_next(5.0, 10.0, 0.0) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn age_body_speed_and_strength_follow_scaling() {
        assert!((age_body_speed(800, 0.001, 0.2) - 1.0).abs() < 1e-6);
        assert!((age_body_strength(750) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn tech_discovery_prob_converts_annual_to_per_check() {
        let annual_only = tech_discovery_prob(0.24, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 12.0);
        assert!(annual_only > 0.0 && annual_only < 0.24);
        let clamped = tech_discovery_prob(2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 1.0);
        assert!((clamped - 1.0).abs() < 1e-6);
    }

    #[test]
    fn migration_food_scarce_checks_threshold() {
        assert!(migration_food_scarce(9.0, 40, 0.3));
        assert!(!migration_food_scarce(15.0, 40, 0.3));
    }

    #[test]
    fn migration_should_attempt_when_any_pressure_or_roll_hits() {
        assert!(migration_should_attempt(true, false, 1.0, 0.0));
        assert!(migration_should_attempt(false, true, 1.0, 0.0));
        assert!(migration_should_attempt(false, false, 0.1, 0.2));
        assert!(!migration_should_attempt(false, false, 0.3, 0.2));
    }

    #[test]
    fn population_housing_cap_uses_free_cap_without_shelters() {
        assert_eq!(population_housing_cap(0, 25, 6), 25);
        assert_eq!(population_housing_cap(3, 25, 6), 18);
    }

    #[test]
    fn population_birth_block_code_follows_gate_order() {
        let maxed = population_birth_block_code(100, 100, 10, 1000.0, 5, 25, 6, 0.5);
        let too_few = population_birth_block_code(3, 100, 10, 1000.0, 5, 25, 6, 0.5);
        // alive=65, free_cap=25, shelters=6, capacity=6 → max_housing=25+36=61 < 65 → block
        let housing = population_birth_block_code(65, 100, 6, 1000.0, 5, 25, 6, 0.5);
        let food = population_birth_block_code(40, 100, 10, 5.0, 5, 25, 6, 0.5);
        let allow = population_birth_block_code(40, 100, 10, 1000.0, 5, 25, 6, 0.5);
        assert_eq!(maxed, 1);
        assert_eq!(too_few, 2);
        assert_eq!(housing, 3);
        assert_eq!(food, 4);
        assert_eq!(allow, 0);
    }

    #[test]
    fn chronicle_should_prune_respects_interval() {
        assert!(!chronicle_should_prune(12, 5, 10));
        assert!(chronicle_should_prune(15, 5, 10));
    }

    #[test]
    fn chronicle_cutoff_tick_calculates_age_window() {
        assert_eq!(chronicle_cutoff_tick(100, 20, 4380), 350400);
    }

    #[test]
    fn chronicle_keep_world_event_applies_importance_bands() {
        let keep_low_old = chronicle_keep_world_event(100, 2, 200, 50);
        let keep_med_old = chronicle_keep_world_event(100, 3, 50, 150);
        let keep_high = chronicle_keep_world_event(10, 4, 200, 150);
        assert!(!keep_low_old);
        assert!(!keep_med_old);
        assert!(keep_high);
    }

    #[test]
    fn chronicle_keep_personal_event_keeps_high_importance_or_valid_world_tick() {
        assert!(chronicle_keep_personal_event(true, 1));
        assert!(chronicle_keep_personal_event(false, 4));
        assert!(!chronicle_keep_personal_event(false, 3));
    }

    #[test]
    fn psychology_break_type_code_roundtrips_known_types() {
        for kind in ["outrage_violence", "panic", "rage", "shutdown", "purge"] {
            let code = psychology_break_type_code(kind);
            let out = psychology_break_type_label(code);
            assert_eq!(out, kind);
        }
        assert_eq!(psychology_break_type_code("unknown"), 0);
        assert_eq!(psychology_break_type_label(0), "");
    }

    #[test]
    fn coping_learn_probability_rises_with_stress_and_recovery() {
        let low = coping_learn_probability(400.0, 20.0, false, 0, 2, 15.0);
        let high = coping_learn_probability(1800.0, 80.0, true, 5, 2, 15.0);
        assert!(high > low);
        assert!((0.0..=1.0).contains(&low));
        assert!((0.0..=1.0).contains(&high));
    }

    #[test]
    fn coping_softmax_index_selects_valid_index() {
        let scores = [1.0_f32, 3.0_f32, 2.0_f32];
        let idx_low = coping_softmax_index(&scores, 0.0);
        let idx_high = coping_softmax_index(&scores, 1.0);
        assert!((0..=2).contains(&idx_low));
        assert!((0..=2).contains(&idx_high));
        assert_eq!(coping_softmax_index(&[], 0.5), -1);
    }

    #[test]
    fn emotion_break_threshold_and_probability_are_bounded() {
        let threshold = emotion_break_threshold(1.0, 300.0, 50.0);
        let low = emotion_break_trigger_probability(200.0, threshold, 60.0, 0.01);
        let high = emotion_break_trigger_probability(1200.0, threshold, 60.0, 0.01);
        assert!((threshold - 350.0).abs() < 1e-6);
        assert!(high > low);
        assert!((0.0..=1.0).contains(&high));
    }

    #[test]
    fn emotion_break_type_code_prioritizes_outrage_then_dominant_emotion() {
        assert_eq!(
            emotion_break_type_code(80.0, 10.0, 20.0, 30.0, 40.0, 60.0),
            1
        );
        assert_eq!(
            emotion_break_type_code(20.0, 50.0, 30.0, 10.0, 20.0, 60.0),
            2
        );
        assert_eq!(
            emotion_break_type_code(20.0, 10.0, 55.0, 10.0, 20.0, 60.0),
            3
        );
        assert_eq!(
            emotion_break_type_code(20.0, 10.0, 10.0, 15.0, 20.0, 60.0),
            5
        );
    }

    #[test]
    fn emotion_adjusted_half_life_and_baseline_follow_formula() {
        let hl = emotion_adjusted_half_life(2.0, 0.5, 1.0);
        let baseline = emotion_baseline_value(10.0, 4.0, 0.5, 0.0, 12.0);
        assert!(hl > 2.0);
        assert!((baseline - 12.0).abs() < 1e-6);
    }

    #[test]
    fn emotion_habituation_and_contagion_factors_are_sensible() {
        let hab0 = emotion_habituation_factor(0.2, 0);
        let hab5 = emotion_habituation_factor(0.2, 5);
        let sus = emotion_contagion_susceptibility(1.0, 1.0);
        let near = emotion_contagion_distance_factor(1.0, 5.0);
        let far = emotion_contagion_distance_factor(20.0, 5.0);
        assert_eq!(hab0, 1.0);
        assert!(hab5 < hab0);
        assert!(sus > 1.0);
        assert!(near > far);
    }

    #[test]
    fn emotion_event_impulse_from_appraisal_matches_expected_shape() {
        let out = emotion_event_impulse_from_appraisal(&[
            0.8, 0.4, 0.6, 0.2, 0.1, 0.0, 0.7, 0.5, 30.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
        ]);
        assert_eq!(out.len(), 8);
        assert!(out[0] > 0.0);
        assert_eq!(out[1], 0.0);
        assert!(out[6] > 0.0);
        assert!(out[7] > 0.0);
    }

    #[test]
    fn emotion_event_impulse_batch_emits_expected_flat_length() {
        let flat = [
            0.8, 0.4, 0.6, 0.2, 0.1, 0.0, 0.7, 0.5, 30.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
            -0.6, 0.2, 0.3, 0.1, 0.2, 0.3, 0.4, 0.2, 20.0, 0.9, 1.1, 1.2, 1.0, 1.0, 0.8, 1.0, 1.0,
        ];
        let out = emotion_event_impulse_batch(&flat);
        assert_eq!(out.len(), 16);
    }

    #[test]
    fn tech_cultural_memory_decay_respects_state_and_floor() {
        let recent = tech_cultural_memory_decay(1.0, 0.05, 0.5, 0.1, true);
        let long = tech_cultural_memory_decay(1.0, 0.05, 0.5, 0.1, false);
        assert!((recent - 0.95).abs() < 1e-6);
        assert!((long - 0.975).abs() < 1e-6);
        let floored = tech_cultural_memory_decay(0.12, 0.5, 1.0, 0.1, true);
        assert!((floored - 0.1).abs() < 1e-6);
    }

    #[test]
    fn tech_modifier_stack_clamp_applies_caps() {
        let out = tech_modifier_stack_clamp(2.5, 1.2, 1.8, 0.7);
        assert!((out[0] - 1.8).abs() < 1e-6);
        assert!((out[1] - 0.7).abs() < 1e-6);
        let out2 = tech_modifier_stack_clamp(0.0, -2.0, 2.0, 0.5);
        assert!((out2[0] - 0.01).abs() < 1e-6);
        assert!((out2[1] + 0.5).abs() < 1e-6);
    }

    #[test]
    fn movement_should_skip_tick_matches_mod_logic() {
        assert!(movement_should_skip_tick(3, 6, 0));
        assert!(movement_should_skip_tick(3, 5, 1));
        assert!(!movement_should_skip_tick(3, 5, 2));
        assert!(!movement_should_skip_tick(0, 6, 0));
    }

    #[test]
    fn building_campfire_social_boost_selects_by_time() {
        assert!((building_campfire_social_boost(true, 0.01, 0.02) - 0.02).abs() < 1e-6);
        assert!((building_campfire_social_boost(false, 0.01, 0.02) - 0.01).abs() < 1e-6);
    }

    #[test]
    fn building_add_capped_limits_to_cap() {
        assert!((building_add_capped(0.8, 0.1, 1.0) - 0.9).abs() < 1e-6);
        assert!((building_add_capped(0.95, 0.2, 1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn childcare_take_food_clamps_to_remaining_and_zero_bounds() {
        assert!((childcare_take_food(10.0, 3.5) - 3.5).abs() < 1e-6);
        assert!((childcare_take_food(2.0, 5.0) - 2.0).abs() < 1e-6);
        assert_eq!(childcare_take_food(0.0, 2.0), 0.0);
        assert_eq!(childcare_take_food(2.0, 0.0), 0.0);
    }

    #[test]
    fn childcare_hunger_after_applies_restore_and_clamp() {
        assert!((childcare_hunger_after(0.2, 0.3, 1.5) - 0.65).abs() < 1e-6);
        assert_eq!(childcare_hunger_after(0.9, 1.0, 0.5), 1.0);
    }

    #[test]
    fn tech_propagation_culture_modifier_applies_weights_and_clamp() {
        let neutral = tech_propagation_culture_modifier(0.0, 0.0, 0.3, 0.4, 0.1, 2.0);
        assert!((neutral - 0.95).abs() < 1e-6);
        let floored = tech_propagation_culture_modifier(-1.0, 1.0, 0.3, 0.4, 0.1, 2.0);
        assert!((floored - 0.6).abs() < 1e-6);
    }

    #[test]
    fn tech_propagation_carrier_bonus_scales_by_skill() {
        assert!((tech_propagation_carrier_bonus(0, 20.0, 0.5) - 1.0).abs() < 1e-6);
        assert!((tech_propagation_carrier_bonus(20, 20.0, 0.5) - 1.5).abs() < 1e-6);
    }

    #[test]
    fn tech_propagation_final_prob_multiplies_and_clamps() {
        let base = tech_propagation_final_prob(0.2, 1.0, 1.5, 1.4, 1.3, 0.95);
        assert!((base - 0.546).abs() < 1e-6);
        let capped = tech_propagation_final_prob(1.0, 2.0, 2.0, 2.0, 2.0, 0.95);
        assert!((capped - 0.95).abs() < 1e-6);
    }

    #[test]
    fn mortality_hazards_and_prob_monthly_check_is_lower_than_annual() {
        let annual = mortality_hazards_and_prob(
            35.0, 0.60, 1.30, 0.010, 0.00006, 0.090, 0.30, 0.20, 0.05, 0.0, 0.7, 0.3, 0.6, 1.0,
            1.0, 1.0, 0.2, 0.35, false,
        );
        let monthly = mortality_hazards_and_prob(
            35.0, 0.60, 1.30, 0.010, 0.00006, 0.090, 0.30, 0.20, 0.05, 0.0, 0.7, 0.3, 0.6, 1.0,
            1.0, 1.0, 0.2, 0.35, true,
        );
        assert!((0.0..=0.999).contains(&annual[4]));
        assert!(monthly[5] <= annual[4]);
    }

    #[test]
    fn mortality_hazards_and_prob_applies_care_protection_and_dr_reduction() {
        let no_protection = mortality_hazards_and_prob(
            1.0, 0.60, 1.30, 0.010, 0.00006, 0.090, 0.30, 0.20, 0.05, 0.0, 1.0, 2.0, 0.6, 1.0, 1.0,
            1.0, 0.0, 0.35, false,
        );
        let with_protection = mortality_hazards_and_prob(
            1.0, 0.60, 1.30, 0.010, 0.00006, 0.090, 0.30, 0.20, 0.05, 0.0, 1.0, 0.3, 0.6, 1.0, 1.0,
            1.0, 1.0, 0.35, false,
        );
        assert!(with_protection[0] < no_protection[0]);
        assert!(with_protection[3] < no_protection[3]);
    }

    #[test]
    fn cognition_activity_modifier_switches_by_active_skill_count() {
        assert!((cognition_activity_modifier(0, 0.9, 1.2) - 1.2).abs() < 1e-6);
        assert!((cognition_activity_modifier(2, 0.9, 1.2) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn cognition_ace_fluid_decline_mult_applies_threshold_gate() {
        assert!((cognition_ace_fluid_decline_mult(0.3, 0.4, 1.5) - 1.0).abs() < 1e-6);
        assert!((cognition_ace_fluid_decline_mult(0.5, 0.4, 1.5) - 1.5).abs() < 1e-6);
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
    fn child_shrp_step_blocks_below_threshold() {
        let out = child_shrp_step(0.6, true, 0.85, 1.3);
        assert_eq!(out[0], 0.0);
        assert_eq!(out[1], 0.0);
    }

    #[test]
    fn child_shrp_step_applies_multiplier_and_flag() {
        let out = child_shrp_step(0.9, true, 0.85, 1.2);
        assert_eq!(out[0], 1.08);
        assert_eq!(out[1], 1.0);
    }

    #[test]
    fn child_stress_type_code_uses_expected_thresholds() {
        assert_eq!(child_stress_type_code(0.2, false, 0.0), 0);
        assert_eq!(child_stress_type_code(0.6, true, 0.8), 1);
        assert_eq!(child_stress_type_code(0.6, false, 0.8), 2);
    }

    #[test]
    fn child_stress_apply_step_matches_tolerable_shape() {
        let out = child_stress_apply_step(0.5, 80.0, 200.0, 10.0, 0.7, 1.2, 1.4, 1.0, 1);
        assert_eq!(out[0], 0.5);
        assert!(out[1] < 80.0);
        assert!(out[2] > 200.0);
        assert_eq!(out[3], 10.0);
        assert_eq!(out[4], 0.0);
    }

    #[test]
    fn child_stress_apply_step_toxic_produces_damage_delta() {
        let out = child_stress_apply_step(0.5, 80.0, 200.0, 10.0, 0.8, 1.1, 1.3, 1.2, 2);
        assert!(out[2] > 200.0);
        assert!(out[3] > 10.0);
        assert!(out[4] > 0.0);
    }

    #[test]
    fn child_parent_transfer_apply_step_respects_threshold() {
        let unchanged = child_parent_transfer_apply_step(120.0, 0.04, 0.05, 20.0, 2000.0);
        let applied = child_parent_transfer_apply_step(120.0, 0.10, 0.05, 20.0, 2000.0);
        assert_eq!(unchanged, 120.0);
        assert!(applied > unchanged);
    }

    #[test]
    fn child_deprivation_damage_step_accumulates_linearly() {
        let next = child_deprivation_damage_step(0.3, 0.02);
        assert!((next - 0.32).abs() < 1e-6);
    }

    #[test]
    fn child_stage_code_from_age_ticks_uses_cutoffs() {
        assert_eq!(child_stage_code_from_age_ticks(0, 2.0, 5.0, 12.0, 18.0), 0);
        assert_eq!(
            child_stage_code_from_age_ticks(8760 * 3, 2.0, 5.0, 12.0, 18.0),
            1
        );
        assert_eq!(
            child_stage_code_from_age_ticks(8760 * 8, 2.0, 5.0, 12.0, 18.0),
            2
        );
        assert_eq!(
            child_stage_code_from_age_ticks(8760 * 15, 2.0, 5.0, 12.0, 18.0),
            3
        );
        assert_eq!(
            child_stage_code_from_age_ticks(8760 * 20, 2.0, 5.0, 12.0, 18.0),
            4
        );
    }

    #[test]
    fn stress_rebound_apply_step_updates_stress_and_hidden() {
        let out = stress_rebound_apply_step(150.0, 80.0, 20.0, 2000.0);
        assert_eq!(out[0], 170.0);
        assert_eq!(out[1], 60.0);
    }

    #[test]
    fn stress_shaken_countdown_step_reports_clear_at_zero() {
        let out = stress_shaken_countdown_step(1);
        assert_eq!(out[0], 0.0);
        assert_eq!(out[1], 1.0);
    }

    #[test]
    fn stress_injection_apply_step_sets_trace_flag_when_over_threshold() {
        let out = stress_injection_apply_step(120.0, 30.0, 0.02, 0.01, 2000.0);
        assert_eq!(out[0], 150.0);
        assert_eq!(out[1], 1.0);
    }

    #[test]
    fn stress_injection_apply_step_clears_trace_flag_when_under_threshold() {
        let out = stress_injection_apply_step(120.0, 30.0, 0.005, 0.01, 2000.0);
        assert_eq!(out[1], 0.0);
    }

    #[test]
    fn stress_support_score_returns_default_when_empty() {
        assert_eq!(stress_support_score(&[]), 0.3);
    }

    #[test]
    fn stress_support_score_prefers_stronger_primary_tie() {
        let weak_only = stress_support_score(&[0.25, 0.20, 0.15]);
        let one_strong = stress_support_score(&[0.80, 0.20, 0.15]);
        assert!(one_strong > weak_only);
        assert!((0.0..=1.0).contains(&one_strong));
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
