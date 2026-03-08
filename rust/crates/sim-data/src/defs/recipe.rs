use serde::{Deserialize, Serialize};

/// Recipe definition using tag+threshold material matching.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RecipeDef {
    /// Stable recipe identifier.
    pub id: String,
    /// Localization key for the recipe name.
    pub display_name_key: String,
    /// Material inputs resolved by tag selectors.
    pub inputs: Vec<TagRequirement>,
    /// Optional building or technology prerequisites.
    pub requires: Option<RecipeRequires>,
    /// Output template and material forwarding behavior.
    pub output: RecipeOutput,
    /// Craft time in simulation ticks.
    pub time_ticks: u32,
    /// Optional skill tag used to gate access.
    pub skill_tag: Option<String>,
    /// Optional minimum skill level.
    pub min_skill_level: Option<f64>,
}

/// Selector for materials by tag and thresholds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TagRequirement {
    /// Required material tag.
    pub tag: String,
    /// Optional minimum hardness.
    pub min_hardness: Option<f64>,
    /// Optional minimum density.
    pub min_density: Option<f64>,
    /// Optional maximum rarity.
    pub max_rarity: Option<f64>,
    /// Required stack amount.
    pub amount: u32,
}

/// Optional recipe-level prerequisites.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RecipeRequires {
    /// Required building tag.
    pub building_tag: Option<String>,
    /// Required technology id.
    pub tech: Option<String>,
}

/// Recipe output definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RecipeOutput {
    /// Output template id.
    pub template: String,
    /// Index of the input material applied to the result.
    pub material_from_input: usize,
    /// Optional result count, defaulting to 1 when omitted.
    pub count: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_recipe_def_from_ron() {
        let recipe: RecipeDef = ron::from_str(
            r#"RecipeDef(
                id: "stone_knife",
                display_name_key: "RECIPE_STONE_KNIFE",
                inputs: [
                    TagRequirement(
                        tag: "sharp",
                        min_hardness: Some(4.0),
                        min_density: None,
                        max_rarity: None,
                        amount: 1,
                    ),
                ],
                requires: None,
                output: RecipeOutput(
                    template: "knife",
                    material_from_input: 0,
                    count: None,
                ),
                time_ticks: 60,
                skill_tag: Some("knapping"),
                min_skill_level: Some(0.1),
            )"#,
        )
        .expect("expected recipe ron to parse");

        assert_eq!(recipe.output.template, "knife");
        assert_eq!(recipe.inputs[0].tag, "sharp");
    }
}
