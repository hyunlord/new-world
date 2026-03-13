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

/// Shared event-driven shift rule view for runtime temperament integration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemperamentShiftRuleView {
    /// Trigger event key that activates the rule.
    pub trigger_event: String,
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
        }
    }

    /// Placeholder for event-driven shift-rule processing.
    pub fn check_shift_rules(&mut self, event_key: &str, rules: &TemperamentRuleSet) {
        if rules
            .shift_rules
            .iter()
            .any(|rule| rule.trigger_event == event_key)
        {
            log::debug!(
                "[Temperament] shift_rules check stub for event '{}' ({} rule(s) loaded)",
                event_key,
                rules.shift_rules.len()
            );
        }
    }

    /// Applies one axis delta and keeps the component within valid bounds.
    pub fn apply_shift(&mut self, ns: f64, ha: f64, rd: f64, p: f64) {
        self.expressed = TemperamentAxes {
            ns: self.expressed.ns + ns,
            ha: self.expressed.ha + ha,
            rd: self.expressed.rd + rd,
            p: self.expressed.p + p,
        }
        .clamped();
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
        }
    }
}

fn axis_index(axis: &str) -> Option<usize> {
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
