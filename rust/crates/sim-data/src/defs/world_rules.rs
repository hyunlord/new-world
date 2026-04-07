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
    /// Inclusive count range (min, max).
    pub count: (u32, u32),
    /// Cluster radius in tiles.
    #[serde(default = "default_zone_radius")]
    pub radius: u32,
    /// Terrain type to apply to tiles within the zone (matches `TerrainType` variant name).
    #[serde(default)]
    pub terrain_override: Option<String>,
    /// Resource to add or boost within the zone.
    #[serde(default)]
    pub resource_boost: Option<ZoneResourceBoost>,
    /// Temperature delta applied to zone tiles (positive = warmer, negative = colder).
    #[serde(default)]
    pub temperature_mod: Option<f32>,
    /// Moisture delta applied to zone tiles.
    #[serde(default)]
    pub moisture_mod: Option<f32>,
}

fn default_zone_radius() -> u32 {
    3
}

/// Resource modification applied to each tile inside a special zone.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ZoneResourceBoost {
    /// Resource type identifier (matches `ResourceType` variant name, e.g. `"Food"`).
    pub resource: String,
    /// Amount added to the tile resource (or used as initial amount if absent).
    pub amount: f64,
    /// Max-amount cap; an existing resource's cap is raised to this value if lower.
    pub max_amount: f64,
    /// Regen rate; an existing resource's rate is raised to this value if lower.
    pub regen_rate: f64,
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

    #[test]
    fn parses_rule_special_zone_with_all_fields() {
        let zone: RuleSpecialZone = ron::from_str(
            r#"RuleSpecialZone(
                kind: "hot_spring",
                count: (2, 4),
                radius: 3,
                terrain_override: Some("Grassland"),
                resource_boost: Some(ZoneResourceBoost(
                    resource: "Food",
                    amount: 8.0,
                    max_amount: 12.0,
                    regen_rate: 0.5,
                )),
                temperature_mod: Some(0.3),
                moisture_mod: Some(0.2),
            )"#,
        )
        .expect("RuleSpecialZone with all fields should parse");

        assert_eq!(zone.kind, "hot_spring");
        assert_eq!(zone.count, (2, 4));
        assert_eq!(zone.radius, 3);
        assert_eq!(zone.terrain_override.as_deref(), Some("Grassland"));
        let boost = zone.resource_boost.as_ref().expect("resource_boost must be Some");
        assert_eq!(boost.resource, "Food");
        assert!((boost.amount - 8.0).abs() < 1e-6);
        assert_eq!(zone.temperature_mod, Some(0.3));
    }

    #[test]
    fn parses_rule_special_zone_defaults() {
        let zone: RuleSpecialZone = ron::from_str(
            r#"RuleSpecialZone(kind: "dungeon_node", count: (1, 3))"#,
        )
        .expect("RuleSpecialZone with only required fields should parse");

        assert_eq!(zone.radius, 3, "default radius should be 3");
        assert!(zone.terrain_override.is_none());
        assert!(zone.resource_boost.is_none());
        assert!(zone.temperature_mod.is_none());
        assert!(zone.moisture_mod.is_none());
    }
}
