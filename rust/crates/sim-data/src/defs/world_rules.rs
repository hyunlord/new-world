use serde::{Deserialize, Serialize};

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
            )"#,
        )
        .expect("expected world rules ron to parse");

        assert_eq!(ruleset.name, "BaseRules");
        assert_eq!(ruleset.resource_modifiers.len(), 1);
    }
}
