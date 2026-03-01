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
    let age_factor = if mother_age < 16.0 || mother_age > 45.0 {
        0.7
    } else if mother_age < 18.0 || mother_age > 40.0 {
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
        action_energy_cost, age_trainability_modifier, age_trainability_modifiers,
        anxious_attachment_stress_delta, calc_realized_values, calc_training_gain,
        calc_training_gains, child_deprivation_damage_step, child_parent_stress_transfer,
        child_parent_transfer_apply_step, child_shrp_step, child_simultaneous_ace_step,
        child_social_buffered_intensity, child_stage_code_from_age_ticks, child_stress_apply_step,
        child_stress_type_code, compute_age_curve, compute_age_curves, critical_severity,
        economic_tendencies_step, erg_frustration_step, job_satisfaction_score,
        job_satisfaction_score_batch, leader_age_respect, leader_score, needs_base_decay_step,
        needs_critical_severity_step, network_social_capital_norm, occupation_best_skill_index,
        occupation_should_switch, reputation_decay_value, reputation_event_delta,
        rest_energy_recovery, revolution_risk_score, stratification_gini,
        stratification_status_score, stratification_wealth_score, stress_injection_apply_step,
        stress_rebound_apply_step, stress_shaken_countdown_step, stress_support_score,
        thirst_decay, upper_needs_best_skill_normalized, upper_needs_job_alignment, upper_needs_step,
        value_plasticity, warmth_decay, family_newborn_health, title_is_elder, title_skill_tier,
        social_attachment_affinity_multiplier, social_proposal_accept_prob,
        tension_scarcity_pressure, tension_next_value, resource_regen_next,
        age_body_speed, age_body_strength, tech_discovery_prob, migration_food_scarce,
        migration_should_attempt, tech_cultural_memory_decay, tech_modifier_stack_clamp,
        movement_should_skip_tick, building_campfire_social_boost, building_add_capped,
        cognition_activity_modifier,
        cognition_ace_fluid_decline_mult,
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
        let out =
            stratification_status_score(0.4, 0.7, 0.6, 50.0, 0.3, 0.3, 0.2, 0.1, 0.2, 0.2);
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
