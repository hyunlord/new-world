use serde::{Deserialize, Serialize};

/// Global simulation constant overrides applied at world-load time.
///
/// Each field is `Option<_>` — `None` means "use the hardcoded config default".
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GlobalConstants {
    /// Season mode override: "default", "eternal_winter", "eternal_summer", "no_seasons".
    #[serde(default)]
    pub season_mode: Option<String>,
    /// Hunger decay rate multiplier (1.0 = default, 2.0 = twice as fast).
    #[serde(default)]
    pub hunger_decay_mul: Option<f64>,
    /// Warmth decay rate multiplier (1.0 = default).
    #[serde(default)]
    pub warmth_decay_mul: Option<f64>,
    /// Food regeneration multiplier (0.0 = no food regrows, 1.0 = default).
    #[serde(default)]
    pub food_regen_mul: Option<f64>,
    /// Wood regeneration multiplier.
    #[serde(default)]
    pub wood_regen_mul: Option<f64>,
    /// Whether farming/agriculture is enabled.
    #[serde(default)]
    pub farming_enabled: Option<bool>,
    /// Base temperature bias (-1.0 = frigid, 0.0 = default, 1.0 = scorching).
    #[serde(default)]
    pub temperature_bias: Option<f64>,
    /// Disaster frequency multiplier (0.0 = no disasters, 1.0 = default, 2.0 = double).
    #[serde(default)]
    pub disaster_frequency_mul: Option<f64>,
}

/// Declarative world ruleset definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorldRuleset {
    /// Ruleset name or id.
    pub name: String,
    /// Merge priority used during composition.
    pub priority: i32,
    /// Resource economy modifiers.
    #[serde(default)]
    pub resource_modifiers: Vec<RuleResourceModifier>,
    /// Special zones spawned by the ruleset.
    #[serde(default)]
    pub special_zones: Vec<RuleSpecialZone>,
    /// Special resources introduced by the ruleset.
    #[serde(default)]
    pub special_resources: Vec<RuleResourceSpawn>,
    /// Agent-facing modifiers.
    #[serde(default)]
    pub agent_modifiers: Vec<RuleAgentModifier>,
    /// Optional influence-channel metadata overrides.
    #[serde(default)]
    pub influence_channels: Vec<InfluenceChannelRule>,
    /// Global simulation constant overrides applied at world-load time.
    #[serde(default)]
    pub global_constants: Option<GlobalConstants>,
}

/// Resource-economy modifier rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RuleResourceModifier {
    /// Target resource or system id.
    pub target: String,
    /// Scalar multiplier applied at compile time.
    pub multiplier: f64,
}

/// Special zone spawn definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RuleSpecialZone {
    /// Zone type identifier.
    pub kind: String,
    /// Inclusive count range.
    pub count: (u32, u32),
}

/// Special resource definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RuleResourceSpawn {
    /// Resource identifier.
    pub name: String,
    /// Tags associated with the special resource.
    pub tags: Vec<String>,
}

/// Agent-facing world rule modifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RuleAgentModifier {
    /// Target subsystem identifier.
    pub system: String,
    /// Effect identifier applied by the rule.
    pub effect: String,
}

/// Declarative override for one influence-channel metadata entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InfluenceChannelRule {
    /// Stable influence channel id.
    pub channel: String,
    /// Optional decay-rate override.
    pub decay_rate: Option<f64>,
    /// Optional default-radius override.
    pub default_radius: Option<f64>,
    /// Optional maximum-radius override.
    pub max_radius: Option<u32>,
    /// Optional wall-blocking sensitivity override.
    pub wall_blocking_sensitivity: Option<f64>,
    /// Optional clamp policy override.
    pub clamp_policy: Option<InfluenceClampPolicyDef>,
}

/// Declarative influence clamp policy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InfluenceClampPolicyDef {
    /// Compress values with a sigmoid policy.
    Sigmoid,
    /// Clamp values directly into `[0.0, 1.0]`.
    UnitInterval,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_world_ruleset_from_ron() {
        let ruleset: WorldRuleset = ron::from_str(
            r#"WorldRuleset(
                name: "BaseRules",
                priority: 0,
                resource_modifiers: [
                    RuleResourceModifier(target: "surface_foraging", multiplier: 1.0),
                ],
                special_zones: [],
                special_resources: [],
                agent_modifiers: [],
                influence_channels: [
                    InfluenceChannelRule(
                        channel: "food",
                        decay_rate: Some(0.18),
                        default_radius: Some(7.0),
                        max_radius: Some(14),
                        wall_blocking_sensitivity: Some(0.2),
                        clamp_policy: Some(UnitInterval),
                    ),
                ],
            )"#,
        )
        .expect("expected world rules ron to parse");

        assert_eq!(ruleset.name, "BaseRules");
        assert_eq!(ruleset.resource_modifiers.len(), 1);
        assert_eq!(ruleset.influence_channels.len(), 1);
    }
}
