use serde::{Deserialize, Serialize};

/// Runtime action definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ActionDef {
    /// Stable action identifier.
    pub id: String,
    /// Localization key for the action name.
    pub display_name_key: String,
    /// High-level action category.
    pub category: ActionCategory,
    /// Preconditions that gate action execution.
    pub preconditions: Vec<ActionCondition>,
    /// Effects applied when the action resolves.
    pub effects: Vec<ActionEffect>,
    /// Base duration in simulation ticks.
    pub base_duration_ticks: u32,
    /// Optional tool tag or output template requirement.
    pub tool_tag: Option<String>,
    /// Optional skill tag needed for the action.
    pub skill_tag: Option<String>,
    /// Optional animation key for render integration.
    pub animation_key: Option<String>,
}

/// High-level action categories.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionCategory {
    /// Survival actions.
    Survival,
    /// Crafting actions.
    Craft,
    /// Social actions.
    Social,
    /// Movement actions.
    Movement,
    /// Combat actions.
    Combat,
    /// Ritual actions.
    Ritual,
    /// Rest actions.
    Rest,
}

/// Declarative action precondition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionCondition {
    /// Requires an item with a tag.
    HasItem { tag: String },
    /// Requires an entity with a tag within radius.
    NearEntity { tag: String, radius: f64 },
    /// Requires a need to be below a threshold.
    NeedBelow { need: String, threshold: f64 },
    /// Requires a need to be above a threshold.
    NeedAbove { need: String, threshold: f64 },
    /// Requires a skill level threshold.
    HasSkill { tag: String, min_level: f64 },
    /// Requires being inside a building role.
    InBuilding { role: String },
    /// Requires a time-of-day window.
    TimeOfDay { from: f64, to: f64 },
}

/// Declarative action effect.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionEffect {
    /// Add or subtract a need value.
    AddNeed { need: String, amount: f64 },
    /// Consume an item matching a tag.
    ConsumeItem { tag: String, amount: u32 },
    /// Produce an item template with an optional material tag.
    ProduceItem {
        template: String,
        material_tag: Option<String>,
    },
    /// Award skill experience.
    AddSkillXP { skill: String, amount: f64 },
    /// Emit influence into a channel.
    EmitInfluence {
        channel: String,
        radius: f64,
        intensity: f64,
    },
    /// Spawn a simulation event.
    SpawnEvent { event_id: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_action_def_from_ron() {
        let action: ActionDef = ron::from_str(
            r#"ActionDef(
                id: "knap_stone_knife",
                display_name_key: "ACTION_KNAP_STONE_KNIFE",
                category: Craft,
                preconditions: [
                    HasSkill(tag: "knapping", min_level: 0.1),
                ],
                effects: [
                    ProduceItem(template: "knife", material_tag: Some("sharp")),
                ],
                base_duration_ticks: 60,
                tool_tag: Some("knife"),
                skill_tag: Some("knapping"),
                animation_key: Some("craft"),
            )"#,
        )
        .expect("expected action ron to parse");

        assert_eq!(action.id, "knap_stone_knife");
        assert!(matches!(action.category, ActionCategory::Craft));
    }
}
