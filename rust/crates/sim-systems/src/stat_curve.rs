const VALUE_SCALE: f32 = 1000.0;

#[inline]
fn lerpf(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
fn need_deficit(value: f32, threshold: f32) -> f32 {
    if threshold <= 0.0 {
        return 0.0;
    }
    ((threshold - value) / threshold).clamp(0.0, 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContinuousStressInputs {
    pub hunger: f32,
    pub energy_deficit: f32,
    pub social_isolation: f32,
    pub total: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StressPrimaryStep {
    pub appraisal_scale: f32,
    pub hunger: f32,
    pub energy_deficit: f32,
    pub social_isolation: f32,
    pub total: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmotionStressContribution {
    pub fear: f32,
    pub anger: f32,
    pub sadness: f32,
    pub disgust: f32,
    pub surprise: f32,
    pub joy: f32,
    pub trust: f32,
    pub anticipation: f32,
    pub va_composite: f32,
    pub total: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StressReserveStep {
    pub reserve: f32,
    pub gas_stage: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StressStateSnapshot {
    pub stress_state: i32,
    pub stress_mu_sadness: f32,
    pub stress_mu_anger: f32,
    pub stress_mu_fear: f32,
    pub stress_mu_joy: f32,
    pub stress_mu_trust: f32,
    pub stress_neg_gain_mult: f32,
    pub stress_pos_gain_mult: f32,
    pub stress_blunt_mult: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StressTraceBatchStep {
    pub total_contribution: f32,
    pub updated_per_tick: Vec<f32>,
    pub active_mask: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StressDeltaStep {
    pub delta: f32,
    pub hidden_threat_accumulator: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StressPostUpdateStep {
    pub reserve: f32,
    pub gas_stage: i32,
    pub allostatic: f32,
    pub stress_state: i32,
    pub stress_mu_sadness: f32,
    pub stress_mu_anger: f32,
    pub stress_mu_fear: f32,
    pub stress_mu_joy: f32,
    pub stress_mu_trust: f32,
    pub stress_neg_gain_mult: f32,
    pub stress_pos_gain_mult: f32,
    pub stress_blunt_mult: f32,
}

/// LOG_DIMINISHING: XP required for a level step.
pub fn log_xp_required(
    level: i32,
    base_xp: f32,
    exponent: f32,
    level_breakpoints: &[i32],
    breakpoint_multipliers: &[f32],
) -> f32 {
    if level <= 0 {
        return 0.0;
    }

    let mut mult = breakpoint_multipliers.first().copied().unwrap_or(1.0);
    for idx in 0..level_breakpoints.len() {
        if level >= level_breakpoints[idx] && idx + 1 < breakpoint_multipliers.len() {
            mult = breakpoint_multipliers[idx + 1];
        }
    }

    base_xp * (level as f32).powf(exponent) * mult
}

/// Inverse of LOG_DIMINISHING cumulative curve: total xp -> current level.
pub fn xp_to_level(
    xp: f32,
    base_xp: f32,
    exponent: f32,
    level_breakpoints: &[i32],
    breakpoint_multipliers: &[f32],
    max_level: i32,
) -> i32 {
    let mut cumulative = 0.0_f32;
    for lv in 1..=max_level {
        cumulative += log_xp_required(
            lv,
            base_xp,
            exponent,
            level_breakpoints,
            breakpoint_multipliers,
        );
        if cumulative > xp {
            return lv - 1;
        }
    }
    max_level
}

/// SCURVE formation speed by phase.
pub fn scurve_speed(current_value: i32, phase_breakpoints: &[i32], phase_speeds: &[f32]) -> f32 {
    let bp0 = phase_breakpoints.first().copied().unwrap_or(300);
    let bp1 = phase_breakpoints.get(1).copied().unwrap_or(700);
    let sp0 = phase_speeds.first().copied().unwrap_or(1.5);
    let sp1 = phase_speeds.get(1).copied().unwrap_or(1.0);
    let sp2 = phase_speeds.get(2).copied().unwrap_or(0.3);

    if current_value < bp0 {
        sp0
    } else if current_value < bp1 {
        sp1
    } else {
        sp2
    }
}

/// Natural need decay.
pub fn need_decay(
    current: i32,
    decay_per_year: i32,
    ticks_elapsed: i32,
    metabolic_mult: f32,
    ticks_per_year: i32,
) -> i32 {
    let decay_per_tick = decay_per_year as f32 / ticks_per_year as f32;
    let total_decay = decay_per_tick * ticks_elapsed as f32 * metabolic_mult;
    (current - total_decay as i32).clamp(0, 1000)
}

/// Continuous stress contribution from unmet lower needs.
pub fn stress_continuous_inputs(
    hunger: f32,
    energy: f32,
    social: f32,
    appraisal_scale: f32,
) -> ContinuousStressInputs {
    let h_def = need_deficit(hunger, 0.35);
    let e_def = need_deficit(energy, 0.40);
    let soc_def = need_deficit(social, 0.25);

    let s_hunger = (3.0 * h_def + 9.0 * h_def * h_def) * appraisal_scale;
    let s_energy = (2.0 * e_def + 10.0 * e_def * e_def) * appraisal_scale;
    let s_social = 2.0 * soc_def * soc_def * appraisal_scale;

    ContinuousStressInputs {
        hunger: s_hunger,
        energy_deficit: s_energy,
        social_isolation: s_social,
        total: s_hunger + s_energy + s_social,
    }
}

/// Lazarus appraisal-derived stress scale.
pub fn stress_appraisal_scale(
    hunger: f32,
    energy: f32,
    social: f32,
    threat: f32,
    conflict: f32,
    support_score: f32,
    extroversion: f32,
    fear_value: f32,
    trust_value: f32,
    conscientiousness: f32,
    openness: f32,
    reserve_ratio: f32,
) -> f32 {
    let d_dep = 0.45 * (1.0 - hunger) + 0.35 * (1.0 - energy) + 0.20 * (1.0 - social);
    let d = (0.30 * d_dep + 0.40 * threat + 0.20 * conflict).clamp(0.0, 1.0);

    let r_physical = 0.5 * hunger + 0.5 * energy;
    let r_safety = 1.0 - threat;
    let r =
        (0.30 * r_physical + 0.30 * r_safety + 0.25 * support_score + 0.15 * 0.5).clamp(0.0, 1.0);

    let threat_appraisal = d
        * (1.0 + 0.55 * (extroversion - 0.5) * 2.0 + 0.25 * (fear_value / 100.0)
            - 0.15 * (trust_value / 100.0));

    let coping_appraisal = r
        * (1.0
            + 0.35 * (conscientiousness - 0.5) * 2.0
            + 0.20 * (openness - 0.5) * 2.0
            + 0.20 * reserve_ratio);

    let imbalance = f32::max(0.0, threat_appraisal - coping_appraisal);
    (1.0 + 0.8 * imbalance).clamp(0.7, 1.9)
}

/// Combined primary stress step:
/// Lazarus appraisal scale + continuous unmet-needs stress.
pub fn stress_primary_step(
    hunger: f32,
    energy: f32,
    social: f32,
    threat: f32,
    conflict: f32,
    support_score: f32,
    extroversion: f32,
    fear_value: f32,
    trust_value: f32,
    conscientiousness: f32,
    openness: f32,
    reserve_ratio: f32,
) -> StressPrimaryStep {
    let appraisal = stress_appraisal_scale(
        hunger,
        energy,
        social,
        threat,
        conflict,
        support_score,
        extroversion,
        fear_value,
        trust_value,
        conscientiousness,
        openness,
        reserve_ratio,
    );
    let continuous = stress_continuous_inputs(hunger, energy, social, appraisal);

    StressPrimaryStep {
        appraisal_scale: appraisal,
        hunger: continuous.hunger,
        energy_deficit: continuous.energy_deficit,
        social_isolation: continuous.social_isolation,
        total: continuous.total,
    }
}

/// Emotion-to-stress contribution with fixed weights and VA composite term.
pub fn stress_emotion_contribution(
    fear: f32,
    anger: f32,
    sadness: f32,
    disgust: f32,
    surprise: f32,
    joy: f32,
    trust: f32,
    anticipation: f32,
    valence: f32,
    arousal: f32,
) -> EmotionStressContribution {
    const EMOTION_STRESS_THRESHOLD: f32 = 20.0;
    const VA_GAMMA: f32 = 3.0;

    let fear_c = 0.09 * f32::max(0.0, fear - EMOTION_STRESS_THRESHOLD);
    let anger_c = 0.06 * f32::max(0.0, anger - EMOTION_STRESS_THRESHOLD);
    let sadness_c = 0.05 * f32::max(0.0, sadness - EMOTION_STRESS_THRESHOLD);
    let disgust_c = 0.04 * f32::max(0.0, disgust - EMOTION_STRESS_THRESHOLD);
    let surprise_c = 0.03 * f32::max(0.0, surprise - EMOTION_STRESS_THRESHOLD);
    let joy_c = -0.05 * f32::max(0.0, joy - EMOTION_STRESS_THRESHOLD);
    let trust_c = -0.04 * f32::max(0.0, trust - EMOTION_STRESS_THRESHOLD);
    let anticipation_c = -0.02 * f32::max(0.0, anticipation - EMOTION_STRESS_THRESHOLD);

    let neg = (-valence / 100.0).clamp(0.0, 1.0);
    let ar = (arousal / 100.0).clamp(0.0, 1.0);
    let va = VA_GAMMA * ar * neg;

    let total = fear_c
        + anger_c
        + sadness_c
        + disgust_c
        + surprise_c
        + joy_c
        + trust_c
        + anticipation_c
        + va;

    EmotionStressContribution {
        fear: fear_c,
        anger: anger_c,
        sadness: sadness_c,
        disgust: disgust_c,
        surprise: surprise_c,
        joy: joy_c,
        trust: trust_c,
        anticipation: anticipation_c,
        va_composite: va,
        total,
    }
}

/// Stress recovery decay value per tick.
pub fn stress_recovery_value(
    stress: f32,
    support_score: f32,
    resilience: f32,
    reserve: f32,
    is_sleeping: bool,
    is_safe: bool,
) -> f32 {
    const BASE_DECAY_PER_TICK: f32 = 1.2;
    const DECAY_FRAC: f32 = 0.006;
    const SAFE_DECAY_BONUS: f32 = 0.8;
    const SLEEP_DECAY_BONUS: f32 = 1.5;
    const SUPPORT_DECAY_MULT: f32 = 0.12;

    let mut decay = BASE_DECAY_PER_TICK + DECAY_FRAC * stress;
    if is_safe {
        decay += SAFE_DECAY_BONUS;
    }
    if is_sleeping {
        decay += SLEEP_DECAY_BONUS;
    }
    decay *= 1.0 + SUPPORT_DECAY_MULT * support_score;
    decay *= 1.0 + 0.10 * (resilience - 0.5) * 2.0;
    if reserve < 30.0 {
        decay *= 0.85;
    }
    decay
}

/// Final stress delta step with denial redirect handling.
pub fn stress_delta_step(
    continuous_input: f32,
    trace_input: f32,
    emotion_input: f32,
    ace_stress_mult: f32,
    trait_accum_mult: f32,
    recovery: f32,
    epsilon: f32,
    denial_active: bool,
    denial_redirect_fraction: f32,
    hidden_threat_accumulator: f32,
    denial_max_accumulator: f32,
) -> StressDeltaStep {
    let mut delta =
        (continuous_input + trace_input + emotion_input) * ace_stress_mult * trait_accum_mult
            - recovery;
    if delta.abs() < epsilon {
        delta = 0.0;
    }

    let mut hidden = hidden_threat_accumulator;
    if denial_active && delta > 0.0 {
        let redirected = delta * denial_redirect_fraction;
        hidden = (hidden + redirected).min(denial_max_accumulator);
        delta -= redirected;
    }

    StressDeltaStep {
        delta,
        hidden_threat_accumulator: hidden,
    }
}

/// Reserve + GAS stage transition step.
pub fn stress_reserve_step(
    reserve: f32,
    stress: f32,
    resilience: f32,
    stress_delta_last: f32,
    gas_stage: i32,
    is_sleeping: bool,
) -> StressReserveStep {
    let drain = f32::max(0.0, (stress - 150.0) / 350.0) * (0.7 + 0.6 * (1.0 - resilience));
    let recover_base = 0.4 + 0.6 * resilience;
    let recover = recover_base * if is_sleeping { 1.0 } else { 0.15 };
    let new_reserve = (reserve - drain + recover).clamp(0.0, 100.0);

    let mut new_stage = gas_stage;
    if (stress_delta_last > 40.0 || stress > 400.0) && new_stage == 0 {
        new_stage = 1;
    }
    if new_reserve >= 30.0 && stress < 500.0 && new_stage == 1 {
        new_stage = 2;
    }
    if new_reserve < 30.0 {
        new_stage = 3;
    }

    StressReserveStep {
        reserve: new_reserve,
        gas_stage: new_stage,
    }
}

/// Combined post-stress update step:
/// reserve + GAS transition + allostatic update + state snapshot.
pub fn stress_post_update_step(
    reserve: f32,
    stress: f32,
    resilience: f32,
    stress_delta_last: f32,
    gas_stage: i32,
    is_sleeping: bool,
    allostatic: f32,
    avoidant_allostatic_mult: f32,
) -> StressPostUpdateStep {
    let reserve_step = stress_reserve_step(
        reserve,
        stress,
        resilience,
        stress_delta_last,
        gas_stage,
        is_sleeping,
    );
    let next_allostatic = stress_allostatic_step(allostatic, stress, avoidant_allostatic_mult);
    let snapshot = stress_state_snapshot(stress, next_allostatic);

    StressPostUpdateStep {
        reserve: reserve_step.reserve,
        gas_stage: reserve_step.gas_stage,
        allostatic: next_allostatic,
        stress_state: snapshot.stress_state,
        stress_mu_sadness: snapshot.stress_mu_sadness,
        stress_mu_anger: snapshot.stress_mu_anger,
        stress_mu_fear: snapshot.stress_mu_fear,
        stress_mu_joy: snapshot.stress_mu_joy,
        stress_mu_trust: snapshot.stress_mu_trust,
        stress_neg_gain_mult: snapshot.stress_neg_gain_mult,
        stress_pos_gain_mult: snapshot.stress_pos_gain_mult,
        stress_blunt_mult: snapshot.stress_blunt_mult,
    }
}

/// Allostatic load update step.
pub fn stress_allostatic_step(allostatic: f32, stress: f32, avoidant_allostatic_mult: f32) -> f32 {
    const ALLO_RATE: f32 = 0.035;
    const ALLO_STRESS_THRESHOLD: f32 = 250.0;
    const ALLO_RECOVERY_THRESHOLD: f32 = 120.0;
    const ALLO_RECOVERY_RATE: f32 = 0.003;

    let mut next = allostatic;
    if stress > ALLO_STRESS_THRESHOLD {
        let mut allo_inc =
            ALLO_RATE * f32::max(0.0, stress - ALLO_STRESS_THRESHOLD) / ALLO_STRESS_THRESHOLD;
        allo_inc = f32::min(allo_inc, 0.05);
        next = (next + allo_inc * avoidant_allostatic_mult).clamp(0.0, 100.0);
    }
    if stress < ALLO_RECOVERY_THRESHOLD {
        next = (next - ALLO_RECOVERY_RATE).clamp(0.0, 100.0);
    }
    next
}

/// Stress state bucket + stress-driven emotion meta snapshot.
pub fn stress_state_snapshot(stress: f32, allostatic: f32) -> StressStateSnapshot {
    let stress_state = if stress >= 500.0 {
        3
    } else if stress >= 350.0 {
        2
    } else if stress >= 200.0 {
        1
    } else {
        0
    };

    let s1 = ((stress - 100.0) / 400.0).clamp(0.0, 1.0);
    let s2 = ((stress - 300.0) / 400.0).clamp(0.0, 1.0);
    let allo_ratio = allostatic / 100.0;

    let stress_mu_sadness = 6.0 * s1 + 10.0 * allo_ratio;
    let stress_mu_anger = 4.0 * s1 + 8.0 * allo_ratio;
    let stress_mu_fear = 5.0 * s1 + 12.0 * allo_ratio;
    let stress_mu_joy = -(5.0 * s1 + 12.0 * allo_ratio);
    let stress_mu_trust = -(4.0 * s1 + 10.0 * allo_ratio);

    let stress_neg_gain_mult = 1.0 + 0.7 * s2;
    let stress_pos_gain_mult = 1.0 - 0.5 * s2;

    let blunt_denom = 1.0 + 0.9 * allo_ratio * if allo_ratio > 0.6 { 2.0 } else { 1.0 };
    let stress_blunt_mult = 1.0 / blunt_denom;

    StressStateSnapshot {
        stress_state,
        stress_mu_sadness,
        stress_mu_anger,
        stress_mu_fear,
        stress_mu_joy,
        stress_mu_trust,
        stress_neg_gain_mult,
        stress_pos_gain_mult,
        stress_blunt_mult,
    }
}

/// Batch step for stress traces:
/// - sums current per_tick values
/// - applies per-entry decay
/// - marks entries active when decayed value >= min_keep
pub fn stress_trace_batch_step(
    per_tick: &[f32],
    decay_rate: &[f32],
    min_keep: f32,
) -> StressTraceBatchStep {
    let len = per_tick.len().min(decay_rate.len());
    let mut total = 0.0_f32;
    let mut updated: Vec<f32> = Vec::with_capacity(len);
    let mut active: Vec<u8> = Vec::with_capacity(len);

    for idx in 0..len {
        let contribution = per_tick[idx];
        total += contribution;
        let next = contribution * (1.0 - decay_rate[idx]);
        updated.push(next);
        active.push(if next >= min_keep { 1_u8 } else { 0_u8 });
    }

    StressTraceBatchStep {
        total_contribution: total,
        updated_per_tick: updated,
        active_mask: active,
    }
}

/// Resilience value update.
pub fn stress_resilience_value(
    e_axis: f32,
    c_axis: f32,
    x_axis: f32,
    o_axis: f32,
    a_axis: f32,
    h_axis: f32,
    support_score: f32,
    allostatic: f32,
    hunger: f32,
    energy: f32,
    scar_resilience_mod: f32,
) -> f32 {
    let mut r = 0.35 * (1.0 - e_axis)
        + 0.25 * c_axis
        + 0.15 * x_axis
        + 0.10 * o_axis
        + 0.10 * a_axis
        + 0.05 * h_axis
        + 0.25 * support_score
        - 0.30 * (allostatic / 100.0);

    let fatigue_penalty =
        ((0.3 - energy) / 0.3).clamp(0.0, 0.3) + ((0.3 - hunger) / 0.3).clamp(0.0, 0.2);
    r -= 0.20 * fatigue_penalty;
    r += scar_resilience_mod;
    r.clamp(0.05, 1.0)
}

/// SIGMOID_EXTREME influence.
pub fn sigmoid_extreme(
    value: i32,
    flat_zone_lo: i32,
    flat_zone_hi: i32,
    pole_multiplier: f32,
) -> f32 {
    let flat_lo = flat_zone_lo as f32 / VALUE_SCALE;
    let flat_hi = flat_zone_hi as f32 / VALUE_SCALE;
    let norm = value as f32 / VALUE_SCALE;

    if norm >= flat_lo && norm <= flat_hi {
        let t_mid = (norm - flat_lo) / (flat_hi - flat_lo);
        lerpf(0.7, 1.3, t_mid)
    } else if norm < flat_lo {
        let t_lo = 1.0 - (norm / flat_lo);
        let bottom = 1.0 / pole_multiplier;
        f32::max(bottom, lerpf(0.7, bottom, t_lo.powf(1.5)))
    } else {
        let t_hi = (norm - flat_hi) / (1.0 - flat_hi);
        f32::min(pole_multiplier, lerpf(1.3, pole_multiplier, t_hi.powf(1.5)))
    }
}

/// POWER influence.
pub fn power_influence(value: i32, exponent: f32) -> f32 {
    (value as f32 / VALUE_SCALE).powf(exponent)
}

/// THRESHOLD_POWER influence.
pub fn threshold_power(value: i32, threshold: i32, exponent: f32, max_output: f32) -> f32 {
    if value >= threshold {
        return 0.0;
    }
    let deficit = (threshold - value) as f32 / threshold as f32;
    f32::min(max_output, deficit.powf(exponent) * max_output)
}

/// LINEAR influence.
pub fn linear_influence(value: i32) -> f32 {
    value as f32 / VALUE_SCALE
}

/// STEP influence.
pub fn step_influence(value: i32, threshold: i32, above_value: f32, below_value: f32) -> f32 {
    if value >= threshold {
        above_value
    } else {
        below_value
    }
}

/// STEP_LINEAR influence with ordered `(below_threshold, multiply)` entries.
pub fn step_linear(value: i32, steps: &[(i32, f32)]) -> f32 {
    let mut result = 1.0_f32;
    for (below, multiply) in steps {
        if value < *below {
            result = *multiply;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_xp_required_basics() {
        let req = log_xp_required(10, 100.0, 1.8, &[], &[]);
        assert!(req > 0.0);
        assert_eq!(log_xp_required(0, 100.0, 1.8, &[], &[]), 0.0);
    }

    #[test]
    fn xp_to_level_increases_with_xp() {
        let l1 = xp_to_level(100.0, 100.0, 1.8, &[], &[], 100);
        let l2 = xp_to_level(10_000.0, 100.0, 1.8, &[], &[], 100);
        assert!(l2 >= l1);
    }

    #[test]
    fn need_decay_clamps_bounds() {
        assert_eq!(need_decay(5, 4380, 4380, 1.0, 4380), 0);
        assert_eq!(need_decay(1000, 0, 4380, 1.0, 4380), 1000);
    }

    #[test]
    fn threshold_power_zero_above_threshold() {
        assert_eq!(threshold_power(400, 350, 2.0, 12.0), 0.0);
        assert!(threshold_power(200, 350, 2.0, 12.0) > 0.0);
    }

    #[test]
    fn step_linear_applies_last_matching_rule() {
        let steps = vec![(800, 0.9), (600, 0.8), (300, 0.6)];
        assert_eq!(step_linear(900, &steps), 1.0);
        assert_eq!(step_linear(700, &steps), 0.9);
        assert_eq!(step_linear(500, &steps), 0.8);
        assert_eq!(step_linear(200, &steps), 0.6);
    }

    #[test]
    fn stress_continuous_inputs_zero_when_needs_are_high() {
        let out = stress_continuous_inputs(0.9, 0.9, 0.9, 1.2);
        assert_eq!(
            out,
            ContinuousStressInputs {
                hunger: 0.0,
                energy_deficit: 0.0,
                social_isolation: 0.0,
                total: 0.0
            }
        );
    }

    #[test]
    fn stress_continuous_inputs_increase_with_deficit() {
        let mild = stress_continuous_inputs(0.30, 0.35, 0.20, 1.0);
        let severe = stress_continuous_inputs(0.10, 0.10, 0.05, 1.0);
        assert!(severe.hunger > mild.hunger);
        assert!(severe.energy_deficit > mild.energy_deficit);
        assert!(severe.social_isolation > mild.social_isolation);
        assert!(severe.total > mild.total);
    }

    #[test]
    fn stress_appraisal_scale_is_clamped() {
        let low =
            stress_appraisal_scale(1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.5, 0.0, 100.0, 1.0, 1.0, 1.0);
        let high =
            stress_appraisal_scale(0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 100.0, 0.0, 0.0, 0.0, 0.0);
        assert!((0.7..=1.9).contains(&low));
        assert!((0.7..=1.9).contains(&high));
    }

    #[test]
    fn stress_appraisal_scale_rises_with_worse_context() {
        let baseline =
            stress_appraisal_scale(0.8, 0.8, 0.8, 0.1, 0.0, 0.8, 0.5, 10.0, 80.0, 0.7, 0.7, 0.7);
        let stressed =
            stress_appraisal_scale(0.2, 0.3, 0.2, 0.8, 0.4, 0.1, 0.8, 80.0, 20.0, 0.2, 0.2, 0.2);
        assert!(stressed >= baseline);
    }

    #[test]
    fn stress_primary_step_matches_appraisal_plus_continuous() {
        let step = stress_primary_step(
            0.22, 0.38, 0.17, 0.4, 0.2, 0.3, 0.7, 60.0, 40.0, 0.5, 0.6, 0.4,
        );
        let appraisal = stress_appraisal_scale(
            0.22, 0.38, 0.17, 0.4, 0.2, 0.3, 0.7, 60.0, 40.0, 0.5, 0.6, 0.4,
        );
        let cont = stress_continuous_inputs(0.22, 0.38, 0.17, appraisal);

        assert!((step.appraisal_scale - appraisal).abs() < 1e-6);
        assert!((step.hunger - cont.hunger).abs() < 1e-6);
        assert!((step.energy_deficit - cont.energy_deficit).abs() < 1e-6);
        assert!((step.social_isolation - cont.social_isolation).abs() < 1e-6);
        assert!((step.total - cont.total).abs() < 1e-6);
    }

    #[test]
    fn stress_emotion_contribution_is_zero_at_neutral_inputs() {
        let out =
            stress_emotion_contribution(10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 0.0, 0.0);
        assert_eq!(
            out,
            EmotionStressContribution {
                fear: 0.0,
                anger: 0.0,
                sadness: 0.0,
                disgust: 0.0,
                surprise: 0.0,
                joy: -0.0,
                trust: -0.0,
                anticipation: -0.0,
                va_composite: 0.0,
                total: 0.0
            }
        );
    }

    #[test]
    fn stress_emotion_contribution_adds_va_when_negative_high_arousal() {
        let out = stress_emotion_contribution(
            10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, -100.0, 100.0,
        );
        assert_eq!(out.va_composite, 3.0);
        assert_eq!(out.total, 3.0);
    }

    #[test]
    fn stress_recovery_value_higher_when_sleeping_and_safe() {
        let awake_unsafe = stress_recovery_value(300.0, 0.3, 0.5, 50.0, false, false);
        let sleep_safe = stress_recovery_value(300.0, 0.3, 0.5, 50.0, true, true);
        assert!(sleep_safe > awake_unsafe);
    }

    #[test]
    fn stress_recovery_value_reduced_on_low_reserve() {
        let normal = stress_recovery_value(300.0, 0.5, 0.5, 50.0, false, true);
        let low_reserve = stress_recovery_value(300.0, 0.5, 0.5, 10.0, false, true);
        assert!(low_reserve < normal);
    }

    #[test]
    fn stress_delta_step_zeroes_small_delta_by_epsilon() {
        let out = stress_delta_step(1.0, 1.0, 1.0, 1.0, 1.0, 3.01, 0.05, false, 0.6, 0.0, 800.0);
        assert_eq!(out.delta, 0.0);
        assert_eq!(out.hidden_threat_accumulator, 0.0);
    }

    #[test]
    fn stress_delta_step_redirects_and_caps_hidden_when_denial_active() {
        let out = stress_delta_step(10.0, 5.0, 5.0, 1.0, 1.0, 0.0, 0.05, true, 0.6, 790.0, 800.0);
        assert!((out.delta - 8.0).abs() < 1e-6);
        assert_eq!(out.hidden_threat_accumulator, 800.0);
    }

    #[test]
    fn stress_reserve_step_enters_alarm_and_resistance() {
        let alarm = stress_reserve_step(80.0, 450.0, 0.5, 50.0, 0, false);
        assert_eq!(alarm.gas_stage, 2);

        let resistance = stress_reserve_step(80.0, 300.0, 0.6, 0.0, 1, true);
        assert_eq!(resistance.gas_stage, 2);
    }

    #[test]
    fn stress_reserve_step_enters_exhaustion_on_low_reserve() {
        let step = stress_reserve_step(10.0, 600.0, 0.2, 10.0, 1, false);
        assert!(step.reserve < 30.0);
        assert_eq!(step.gas_stage, 3);
    }

    #[test]
    fn stress_post_update_step_matches_component_steps() {
        let reserve_step = stress_reserve_step(80.0, 420.0, 0.6, 25.0, 1, false);
        let allo = stress_allostatic_step(20.0, 420.0, 1.35);
        let snap = stress_state_snapshot(420.0, allo);
        let combined = stress_post_update_step(80.0, 420.0, 0.6, 25.0, 1, false, 20.0, 1.35);

        assert!((combined.reserve - reserve_step.reserve).abs() < 1e-6);
        assert_eq!(combined.gas_stage, reserve_step.gas_stage);
        assert!((combined.allostatic - allo).abs() < 1e-6);
        assert_eq!(combined.stress_state, snap.stress_state);
        assert!((combined.stress_blunt_mult - snap.stress_blunt_mult).abs() < 1e-6);
    }

    #[test]
    fn stress_allostatic_step_increases_under_high_stress() {
        let base = stress_allostatic_step(10.0, 300.0, 1.0);
        let avoidant = stress_allostatic_step(10.0, 300.0, 1.5);
        assert!(base > 10.0);
        assert!(avoidant > base);
    }

    #[test]
    fn stress_allostatic_step_recovers_under_low_stress() {
        let next = stress_allostatic_step(10.0, 80.0, 1.0);
        assert!(next < 10.0);
    }

    #[test]
    fn stress_state_snapshot_buckets_thresholds() {
        assert_eq!(stress_state_snapshot(100.0, 0.0).stress_state, 0);
        assert_eq!(stress_state_snapshot(250.0, 0.0).stress_state, 1);
        assert_eq!(stress_state_snapshot(400.0, 0.0).stress_state, 2);
        assert_eq!(stress_state_snapshot(600.0, 0.0).stress_state, 3);
    }

    #[test]
    fn stress_state_snapshot_blunt_multiplier_drops_with_allostatic_load() {
        let low = stress_state_snapshot(300.0, 20.0);
        let high = stress_state_snapshot(300.0, 80.0);
        assert!(high.stress_blunt_mult < low.stress_blunt_mult);
    }

    #[test]
    fn stress_trace_batch_step_marks_active_and_sums_total() {
        let out = stress_trace_batch_step(&[1.0, 0.5, 0.01], &[0.1, 0.2, 0.5], 0.01);
        assert!((out.total_contribution - 1.51).abs() < 1e-6);
        assert_eq!(out.active_mask, vec![1, 1, 0]);
        assert_eq!(out.updated_per_tick.len(), 3);
    }

    #[test]
    fn stress_trace_batch_step_uses_min_len() {
        let out = stress_trace_batch_step(&[1.0, 2.0], &[0.1], 0.01);
        assert_eq!(out.updated_per_tick.len(), 1);
        assert_eq!(out.active_mask.len(), 1);
        assert!((out.total_contribution - 1.0).abs() < 1e-6);
    }

    #[test]
    fn stress_resilience_value_is_clamped() {
        let low = stress_resilience_value(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 100.0, 0.0, 0.0, -1.0);
        let high = stress_resilience_value(0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0);
        assert!((0.05..=1.0).contains(&low));
        assert!((0.05..=1.0).contains(&high));
    }

    #[test]
    fn stress_resilience_value_drops_with_fatigue() {
        let rested =
            stress_resilience_value(0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.3, 20.0, 1.0, 1.0, 0.0);
        let fatigued =
            stress_resilience_value(0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.3, 20.0, 0.1, 0.1, 0.0);
        assert!(fatigued < rested);
    }
}
