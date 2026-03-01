const VALUE_SCALE: f32 = 1000.0;

#[inline]
fn lerpf(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
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
}
