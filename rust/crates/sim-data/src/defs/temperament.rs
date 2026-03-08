use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Declarative temperament configuration bundle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TemperamentRules {
    /// Polygenic score weight rows by axis.
    #[serde(default)]
    pub prs_weights: Vec<DerivedRuleOverride>,
    /// Initial HEXACO bias rows by axis.
    #[serde(default)]
    pub bias_matrix: Vec<BiasMatrixRow>,
    /// Event-driven temperament shift rules.
    #[serde(default)]
    pub shift_rules: Vec<TemperamentShiftRule>,
}

/// Row of derived rule weights keyed by axis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DerivedRuleOverride {
    /// TCI axis identifier such as `ns` or `ha`.
    pub axis: String,
    /// Weight vector for that axis.
    pub weights: Vec<f64>,
}

/// Row of HEXACO bias values keyed by facet id.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BiasMatrixRow {
    /// TCI axis identifier.
    pub axis: String,
    /// Bias values applied to HEXACO facets.
    pub values: BTreeMap<String, f64>,
}

/// Event-driven temperament shift rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TemperamentShiftRule {
    /// Trigger that activates the rule.
    pub trigger: CauseTrigger,
    /// Conditions that must match before shifting.
    #[serde(default)]
    pub conditions: Vec<ShiftCondition>,
    /// Axis deltas applied when triggered.
    #[serde(default)]
    pub effects: Vec<AxisShift>,
    /// Whether to cascade downstream recalculation.
    pub cascade: bool,
    /// Causal log annotation key.
    pub causal_log: String,
}

/// Trigger for a temperament rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CauseTrigger {
    /// Triggered by a named event id.
    Event(String),
}

/// Condition gate for a temperament shift rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShiftCondition {
    /// Compare a temperament axis against a serialized threshold expression.
    Temperament { axis: String, value: String },
}

/// A single axis delta to apply.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AxisShift {
    /// Axis identifier.
    pub axis: String,
    /// Delta added to the axis.
    pub delta: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_temperament_rules_from_ron() {
        let rules: TemperamentRules = ron::from_str(
            r#"TemperamentRules(
                prs_weights: [
                    DerivedRuleOverride(axis: "ns", weights: [0.1, 0.2]),
                ],
                bias_matrix: [
                    BiasMatrixRow(axis: "ns", values: {"X_SOCIAL_BOLDNESS": 0.3}),
                ],
                shift_rules: [
                    TemperamentShiftRule(
                        trigger: Event("family_death"),
                        conditions: [Temperament(axis: "ha", value: ">0.5")],
                        effects: [
                            AxisShift(axis: "ha", delta: 0.3),
                            AxisShift(axis: "ns", delta: -0.2),
                        ],
                        cascade: true,
                        causal_log: "family_death_temperament_shift",
                    ),
                ],
            )"#,
        )
        .expect("expected temperament rules ron to parse");

        assert_eq!(rules.shift_rules.len(), 1);
        assert_eq!(rules.prs_weights[0].weights.len(), 2);
    }
}
