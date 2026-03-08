use serde::{Deserialize, Serialize};

use super::InfluenceEmission;

/// Building blueprint definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StructureDef {
    /// Stable structure identifier.
    pub id: String,
    /// Localization key for the structure name.
    pub display_name_key: String,
    /// Minimum footprint required for the structure.
    pub min_size: (u32, u32),
    /// Required structural or furniture components.
    pub required_components: Vec<StructureRequirement>,
    /// Optional structural or furniture components.
    #[serde(default)]
    pub optional_components: Vec<StructureRequirement>,
    /// Role recognition policy for the structure.
    pub role_recognition: RoleRecognition,
    /// Influence emitted when the structure is complete.
    #[serde(default)]
    pub influence_when_complete: Vec<InfluenceEmission>,
}

/// Requirement entry for a structure blueprint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StructureRequirement {
    /// Wall requirement with tag selectors.
    Wall { count: u32, tags: Vec<String> },
    /// Roof requirement with tag selectors.
    Roof { tags: Vec<String> },
    /// Furniture requirement by id.
    Furniture { id: String, count: u32 },
}

/// Role recognition configuration for a structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoleRecognition {
    /// Infer the role from matching furniture and components.
    Auto,
    /// Force a fixed structure role.
    Manual { role: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_structure_def_from_ron() {
        let structure: StructureDef = ron::from_str(
            r#"StructureDef(
                id: "lean_to_structure",
                display_name_key: "STRUCT_LEAN_TO",
                min_size: (2, 2),
                required_components: [
                    Roof(tags: ["wood", "plant_fiber"]),
                    Furniture(id: "fire_pit", count: 1),
                ],
                optional_components: [],
                role_recognition: Auto,
                influence_when_complete: [],
            )"#,
        )
        .expect("expected structure ron to parse");

        assert_eq!(structure.id, "lean_to_structure");
        assert_eq!(structure.required_components.len(), 2);
    }
}
