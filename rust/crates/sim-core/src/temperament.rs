use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::components::Personality;
use crate::config;

/// Four-axis TCI temperament values used by shared runtime state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TemperamentAxes {
    /// Novelty seeking.
    pub ns: f64,
    /// Harm avoidance.
    pub ha: f64,
    /// Reward dependence.
    pub rd: f64,
    /// Persistence.
    pub p: f64,
}

impl TemperamentAxes {
    /// Returns a copy with all axis values clamped into `[0.0, 1.0]`.
    pub fn clamped(self) -> Self {
        Self {
            ns: self.ns.clamp(0.0, 1.0),
            ha: self.ha.clamp(0.0, 1.0),
            rd: self.rd.clamp(0.0, 1.0),
            p: self.p.clamp(0.0, 1.0),
        }
    }
}

impl Default for TemperamentAxes {
    fn default() -> Self {
        Self {
            ns: config::TEMPERAMENT_DEFAULT_AXIS_VALUE,
            ha: config::TEMPERAMENT_DEFAULT_AXIS_VALUE,
            rd: config::TEMPERAMENT_DEFAULT_AXIS_VALUE,
            p: config::TEMPERAMENT_DEFAULT_AXIS_VALUE,
        }
    }
}

/// Shared PRS weight row used for data-driven temperament derivation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemperamentPrsWeightRow {
    /// TCI axis identifier such as `ns` or `ha`.
    pub axis: String,
    /// Ordered HEXACO axis weights `[H, E, X, A, C, O]`.
    pub weights: Vec<f64>,
}

/// Shared bias-matrix row used for facet-level temperament derivation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemperamentBiasRow {
    /// TCI axis identifier such as `ns` or `ha`.
    pub axis: String,
    /// Bias values keyed by HEXACO facet id.
    pub values: BTreeMap<String, f64>,
}

/// A single axis delta within a shift rule effect list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShiftEffectView {
    /// Axis index: 0=ns, 1=ha, 2=rd, 3=p.
    pub axis_idx: usize,
    /// Delta added to the expressed axis.
    pub delta: f64,
}

/// Condition gate for a data-driven temperament shift rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShiftConditionView {
    /// Expressed axis value must be strictly above `threshold`.
    AxisAbove {
        /// Axis index: 0=ns, 1=ha, 2=rd, 3=p.
        axis_idx: usize,
        /// Threshold value.
        threshold: f64,
    },
    /// Expressed axis value must be strictly below `threshold`.
    AxisBelow {
        /// Axis index: 0=ns, 1=ha, 2=rd, 3=p.
        axis_idx: usize,
        /// Threshold value.
        threshold: f64,
    },
}

/// Shared event-driven shift rule view for runtime temperament integration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemperamentShiftRuleView {
    /// Trigger event key that activates the rule.
    pub trigger_event: String,
    /// Axis deltas applied when the rule fires.
    pub effects: Vec<ShiftEffectView>,
    /// Conditions that must be met before the shift is applied.
    pub conditions: Vec<ShiftConditionView>,
    /// Causal log annotation key.
    pub causal_log: String,
}

/// Shared temperament-rules view used by runtime code without depending on `sim-data`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TemperamentRuleSet {
    /// PRS weight rows by TCI axis.
    pub prs_weights: Vec<TemperamentPrsWeightRow>,
    /// HEXACO facet bias rows by TCI axis.
    pub bias_matrix: Vec<TemperamentBiasRow>,
    /// Event-driven shift rules.
    pub shift_rules: Vec<TemperamentShiftRuleView>,
}

/// Maximum number of dramatic shift events per agent lifetime (Cloninger 1993).
/// CLAUDE.md: "0–3 times per lifetime".
pub const TEMPERAMENT_MAX_SHIFTS_PER_LIFETIME: u8 = 3;

/// Minimum meaningful expressed-vs-latent delta per axis.
/// Shifts that would produce sub-minimum deltas after clamping are reverted
/// on that axis to prevent noise-level deviations.
pub const TEMPERAMENT_MIN_SHIFT_DELTA: f64 = 0.05;

/// Shared ECS temperament component derived from genes/personality inputs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Temperament {
    /// Minimal polygenic core used by the scaffold.
    pub genes: [f64; 4],
    /// Latent temperament axes.
    pub latent: TemperamentAxes,
    /// Expressed temperament axes used by runtime systems.
    pub expressed: TemperamentAxes,
    /// Whether a latent/expressed divergence is currently unlocked.
    pub awakened: bool,
    /// Number of dramatic shift events applied in this agent's lifetime (capped at 3).
    pub shift_count: u8,
}

impl Temperament {
    /// Derives a temperament scaffold from the current HEXACO personality state using
    /// data-driven PRS weights and facet-level bias rows.
    pub fn from_personality_with_rules(
        personality: &Personality,
        rules: &TemperamentRuleSet,
    ) -> Self {
        let mut axes = [0.0_f64; 4];

        for row in &rules.prs_weights {
            let Some(axis_idx) = axis_index(&row.axis) else {
                continue;
            };
            let weighted_sum = row
                .weights
                .iter()
                .enumerate()
                .filter_map(|(i, weight)| personality.axes.get(i).map(|value| value * weight))
                .sum::<f64>();
            axes[axis_idx] = weighted_sum;
        }

        for row in &rules.bias_matrix {
            let Some(axis_idx) = axis_index(&row.axis) else {
                continue;
            };
            for (facet_key, bias_value) in &row.values {
                if let Some(facet_value) = resolve_facet_value(personality, facet_key) {
                    axes[axis_idx] += facet_value * bias_value;
                }
            }
        }

        let latent = TemperamentAxes {
            ns: axes[0],
            ha: axes[1],
            rd: axes[2],
            p: axes[3],
        }
        .clamped();

        Self {
            genes: [latent.ns, latent.ha, latent.rd, latent.p],
            latent,
            expressed: latent,
            awakened: false,
            shift_count: 0,
        }
    }

    /// Derives a temperament scaffold from the current HEXACO personality state.
    pub fn from_personality(personality: &Personality) -> Self {
        let latent = TemperamentAxes {
            ns: ((personality.axes[5] + personality.axes[2]) * 0.5).clamp(0.0, 1.0),
            ha: personality.axes[1].clamp(0.0, 1.0),
            rd: ((personality.axes[2] + personality.axes[3]) * 0.5).clamp(0.0, 1.0),
            p: personality.axes[4].clamp(0.0, 1.0),
        };
        Self {
            genes: [latent.ns, latent.ha, latent.rd, latent.p],
            latent,
            expressed: latent,
            awakened: false,
            shift_count: 0,
        }
    }

    /// Applies a temperament shift for a recognized event key.
    ///
    /// First checks `rules.shift_rules` for a data-driven rule matching the
    /// event key. If found, evaluates conditions and applies serialised
    /// [`ShiftEffectView`] deltas. When no data-driven rule matches, falls
    /// back to a hardcoded mapping (backward compatibility for empty rulesets).
    ///
    /// Each event maps to (ns_delta, ha_delta, rd_delta, p_delta) based on
    /// Cloninger (1993): dramatic life events shift TCI axes by 0.05–0.15,
    /// occurring 0–3 times per lifetime.
    ///
    /// Returns `true` if a shift was applied, `false` if the event_key is
    /// unrecognized, a condition gate blocks it, or the lifetime cap has been
    /// reached.
    pub fn check_shift_rules(&mut self, event_key: &str, rules: &TemperamentRuleSet) -> bool {
        // Enforce 0–3 shifts per lifetime (CLAUDE.md architectural rule)
        if self.shift_count >= TEMPERAMENT_MAX_SHIFTS_PER_LIFETIME {
            return false;
        }

        // --- Data-driven path: look up event_key in rules.shift_rules ---
        if let Some(rule) = rules
            .shift_rules
            .iter()
            .find(|r| r.trigger_event == event_key)
        {
            // Evaluate all conditions — if any fails, reject the shift
            if !self.evaluate_conditions(&rule.conditions) {
                return false;
            }
            // Accumulate per-axis deltas from the rule's effects
            let (mut ns, mut ha, mut rd, mut p) = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
            for effect in &rule.effects {
                match effect.axis_idx {
                    0 => ns += effect.delta,
                    1 => ha += effect.delta,
                    2 => rd += effect.delta,
                    3 => p += effect.delta,
                    _ => {}
                }
            }
            self.apply_shift(ns, ha, rd, p);
            self.shift_count = self.shift_count.saturating_add(1);
            log::debug!(
                "[Temperament] data-driven shift {}/{} applied for '{}': ns={:+.2}, ha={:+.2}, rd={:+.2}, p={:+.2}",
                self.shift_count,
                TEMPERAMENT_MAX_SHIFTS_PER_LIFETIME,
                event_key,
                ns, ha, rd, p
            );
            return true;
        }

        // No data-driven rule matched — event_key is unrecognized or rules
        // are empty (no registry loaded).  Return false rather than falling
        // back to hardcoded values so a missing/broken RON rule path is
        // surfaced immediately in tests.
        false
    }

    /// Evaluates all shift conditions against current expressed axes.
    /// Returns `true` only if every condition is satisfied.
    fn evaluate_conditions(&self, conditions: &[ShiftConditionView]) -> bool {
        for cond in conditions {
            match cond {
                ShiftConditionView::AxisAbove { axis_idx, threshold } => {
                    if self.expressed_axis(*axis_idx) <= *threshold {
                        return false;
                    }
                }
                ShiftConditionView::AxisBelow { axis_idx, threshold } => {
                    if self.expressed_axis(*axis_idx) >= *threshold {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Returns the expressed value for a given axis index (0=ns, 1=ha, 2=rd, 3=p).
    fn expressed_axis(&self, idx: usize) -> f64 {
        match idx {
            0 => self.expressed.ns,
            1 => self.expressed.ha,
            2 => self.expressed.rd,
            3 => self.expressed.p,
            _ => config::TEMPERAMENT_DEFAULT_AXIS_VALUE,
        }
    }

    /// Applies one axis delta and keeps the component within valid bounds.
    ///
    /// After clamping to [0.0, 1.0], any axis whose absolute delta from latent
    /// is nonzero but below [`TEMPERAMENT_MIN_SHIFT_DELTA`] is snapped back to
    /// the latent value. This prevents clamping near boundaries from creating
    /// noise-level deviations.
    pub fn apply_shift(&mut self, ns: f64, ha: f64, rd: f64, p: f64) {
        let clamped = TemperamentAxes {
            ns: self.expressed.ns + ns,
            ha: self.expressed.ha + ha,
            rd: self.expressed.rd + rd,
            p: self.expressed.p + p,
        }
        .clamped();

        // Snap back axes where clamping created a sub-minimum delta from latent
        self.expressed = TemperamentAxes {
            ns: snap_if_sub_minimum(clamped.ns, self.latent.ns),
            ha: snap_if_sub_minimum(clamped.ha, self.latent.ha),
            rd: snap_if_sub_minimum(clamped.rd, self.latent.rd),
            p: snap_if_sub_minimum(clamped.p, self.latent.p),
        };
        self.awakened = self.expressed != self.latent;
    }

    /// Returns a locale key for the current high-level temperament label.
    pub fn archetype_label_key(&self) -> &'static str {
        let axes = self.expressed;
        if axes.ns >= 0.6 && axes.ha < 0.5 {
            "TEMPERAMENT_SANGUINE"
        } else if axes.ns >= 0.6 && axes.ha >= 0.5 {
            "TEMPERAMENT_CHOLERIC"
        } else if axes.ns < 0.5 && axes.ha >= 0.6 {
            "TEMPERAMENT_MELANCHOLIC"
        } else {
            "TEMPERAMENT_PHLEGMATIC"
        }
    }
}

impl Default for Temperament {
    fn default() -> Self {
        Self {
            genes: [config::TEMPERAMENT_DEFAULT_AXIS_VALUE; 4],
            latent: TemperamentAxes::default(),
            expressed: TemperamentAxes::default(),
            awakened: false,
            shift_count: 0,
        }
    }
}

/// Returns `expressed` unchanged unless it differs from `latent` by a nonzero
/// amount smaller than [`TEMPERAMENT_MIN_SHIFT_DELTA`], in which case `latent`
/// is returned (snap back).
///
/// Uses a small epsilon (1e-9) to prevent floating-point imprecision from
/// snapping back deltas that are exactly at the minimum threshold boundary.
#[inline]
fn snap_if_sub_minimum(expressed: f64, latent: f64) -> f64 {
    let delta = (expressed - latent).abs();
    if delta > 1e-12 && delta < TEMPERAMENT_MIN_SHIFT_DELTA - 1e-9 {
        latent
    } else {
        expressed
    }
}

/// Returns the axis index for a TCI axis name: ns=0, ha=1, rd=2, p=3.
pub fn axis_index(axis: &str) -> Option<usize> {
    match axis {
        "ns" => Some(0),
        "ha" => Some(1),
        "rd" => Some(2),
        "p" => Some(3),
        _ => None,
    }
}

fn resolve_facet_value(personality: &Personality, facet_key: &str) -> Option<f64> {
    let facet_index = match facet_key {
        "H_SINCERITY" => Some(0),
        "H_FAIRNESS" => Some(1),
        "H_GREED_AVOIDANCE" => Some(2),
        "H_MODESTY" => Some(3),
        "E_FEARFULNESS" => Some(4),
        "E_ANXIETY" => Some(5),
        "E_DEPENDENCE" => Some(6),
        "E_SENTIMENTALITY" => Some(7),
        "X_SOCIAL_SELF_ESTEEM" => Some(8),
        "X_SOCIAL_BOLDNESS" => Some(9),
        "X_SOCIABILITY" => Some(10),
        "X_LIVELINESS" => Some(11),
        "A_FORGIVENESS" => Some(12),
        "A_GENTLENESS" => Some(13),
        "A_FLEXIBILITY" => Some(14),
        "A_PATIENCE" => Some(15),
        "C_ORGANIZATION" => Some(16),
        "C_DILIGENCE" => Some(17),
        "C_PERFECTIONISM" => Some(18),
        "C_PRUDENCE" => Some(19),
        "O_AESTHETIC_APPRECIATION" => Some(20),
        "O_INQUISITIVENESS" => Some(21),
        "O_CREATIVITY" => Some(22),
        "O_UNCONVENTIONALITY" => Some(23),
        _ => None,
    }?;

    personality.facets.get(facet_index).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temperament_from_personality_maps_hexaco_axes() {
        let personality = Personality {
            axes: [0.4, 0.8, 0.7, 0.6, 0.9, 0.5],
            facets: [0.5; 24],
        };

        let temperament = Temperament::from_personality(&personality);

        assert!(temperament.expressed.ha > 0.7);
        assert!(temperament.expressed.p > 0.8);
    }

    #[test]
    fn temperament_shift_clamps_and_sets_awakened() {
        let mut temperament = Temperament::default();
        temperament.apply_shift(0.6, 0.0, 0.0, 0.0);
        assert_eq!(temperament.expressed.ns, 1.0);
        assert!(temperament.awakened);
    }

    #[test]
    fn temperament_from_rules_uses_prs_weights() {
        let personality = Personality {
            axes: [0.1, 0.2, 0.3, 0.4, 0.5, 0.6],
            facets: [0.5; 24],
        };
        let rules = TemperamentRuleSet {
            prs_weights: vec![
                TemperamentPrsWeightRow {
                    axis: "ns".to_string(),
                    weights: vec![0.10, 0.20, 0.30, 0.40],
                },
                TemperamentPrsWeightRow {
                    axis: "ha".to_string(),
                    weights: vec![0.05, 0.10, 0.15, 0.20, 0.25, 0.30],
                },
                TemperamentPrsWeightRow {
                    axis: "rd".to_string(),
                    weights: vec![0.50],
                },
                TemperamentPrsWeightRow {
                    axis: "p".to_string(),
                    weights: vec![0.0, 0.0, 0.0, 0.0, 1.0],
                },
            ],
            ..TemperamentRuleSet::default()
        };

        let temperament = Temperament::from_personality_with_rules(&personality, &rules);

        assert!((temperament.latent.ns - 0.30).abs() < 1e-9);
        assert!((temperament.latent.ha - 0.455).abs() < 1e-9);
        assert!((temperament.latent.rd - 0.05).abs() < 1e-9);
        assert!((temperament.latent.p - 0.5).abs() < 1e-9);
    }

    #[test]
    fn temperament_from_rules_applies_bias_matrix() {
        let mut facets = [0.0; 24];
        facets[9] = 0.8;
        facets[21] = 0.4;
        let personality = Personality {
            axes: [0.2; 6],
            facets,
        };
        let mut ns_bias = BTreeMap::new();
        ns_bias.insert("X_SOCIAL_BOLDNESS".to_string(), 0.20);
        ns_bias.insert("O_INQUISITIVENESS".to_string(), 0.25);
        let rules = TemperamentRuleSet {
            bias_matrix: vec![TemperamentBiasRow {
                axis: "ns".to_string(),
                values: ns_bias,
            }],
            ..TemperamentRuleSet::default()
        };

        let temperament = Temperament::from_personality_with_rules(&personality, &rules);
        let expected = 0.8 * 0.20 + 0.4 * 0.25;
        assert!((temperament.latent.ns - expected).abs() < 1e-9);
    }

    #[test]
    fn temperament_from_rules_clamps_to_unit() {
        let personality = Personality {
            axes: [1.0; 6],
            facets: [1.0; 24],
        };
        let mut ns_bias = BTreeMap::new();
        ns_bias.insert("X_SOCIAL_BOLDNESS".to_string(), 1.0);
        let rules = TemperamentRuleSet {
            prs_weights: vec![TemperamentPrsWeightRow {
                axis: "ns".to_string(),
                weights: vec![2.0, 2.0, 2.0, 2.0, 2.0, 2.0],
            }],
            bias_matrix: vec![TemperamentBiasRow {
                axis: "ns".to_string(),
                values: ns_bias,
            }],
            ..TemperamentRuleSet::default()
        };

        let temperament = Temperament::from_personality_with_rules(&personality, &rules);
        assert_eq!(temperament.latent.ns, 1.0);
    }

    #[test]
    fn temperament_from_rules_fallback_path_stays_distinct_from_legacy() {
        let personality = Personality {
            axes: [0.3, 0.7, 0.2, 0.8, 0.6, 0.9],
            facets: [0.4; 24],
        };
        let temperament_from_rules =
            Temperament::from_personality_with_rules(&personality, &TemperamentRuleSet::default());
        let legacy = Temperament::from_personality(&personality);

        assert_eq!(
            temperament_from_rules.latent,
            TemperamentAxes {
                ns: 0.0,
                ha: 0.0,
                rd: 0.0,
                p: 0.0,
            }
        );
        assert_ne!(temperament_from_rules.latent, legacy.latent);
    }
}
