use std::collections::BTreeMap;

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
    /// Base construction time in ticks.
    #[serde(default = "default_build_ticks")]
    pub build_ticks: u64,
    /// Flat resource costs keyed by resource tag (for example `wood`, `stone`).
    #[serde(default)]
    pub resource_costs: BTreeMap<String, f64>,
    /// Influence emitted when the structure is complete.
    #[serde(default)]
    pub influence_when_complete: Vec<InfluenceEmission>,
    /// Layout blueprint: relative tile positions for walls, floors, furniture.
    /// If None, the structure uses legacy hardcoded layout.
    #[serde(default)]
    pub blueprint: Option<Blueprint>,
}

/// Data-driven building layout blueprint. Specifies relative positions
/// for walls, floors, furniture, and doors from the building center (0,0).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Blueprint {
    /// Wall positions relative to center (0,0).
    pub walls: Vec<BlueprintTile>,
    /// Floor positions relative to center.
    #[serde(default)]
    pub floors: Vec<BlueprintTile>,
    /// Furniture placements relative to center.
    #[serde(default)]
    pub furniture: Vec<BlueprintFurniture>,
    /// Door positions relative to center (gaps in wall ring).
    #[serde(default)]
    pub doors: Vec<(i32, i32)>,
}

/// A single tile entry in a blueprint (wall or floor).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlueprintTile {
    /// Relative position from building center.
    pub offset: (i32, i32),
    /// Material tag — resolved at runtime based on available resources.
    #[serde(default = "default_material_tag")]
    pub material_tag: String,
}

/// A furniture placement entry in a blueprint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlueprintFurniture {
    /// Relative position from building center.
    pub offset: (i32, i32),
    /// Furniture definition id (must match a FurnitureDef.id).
    pub furniture_id: String,
}

fn default_material_tag() -> String {
    "building_material".to_string()
}

fn default_build_ticks() -> u64 {
    60
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
                id: "shelter",
                display_name_key: "BUILDING_TYPE_SHELTER",
                min_size: (2, 2),
                required_components: [
                    Wall(count: 8, tags: ["building_material"]),
                    Roof(tags: ["roof_material"]),
                    Furniture(id: "fire_pit", count: 1),
                ],
                optional_components: [],
                role_recognition: Auto,
                build_ticks: 60,
                resource_costs: {
                    "wood": 4.0,
                    "stone": 1.0,
                },
                influence_when_complete: [],
            )"#,
        )
        .expect("expected structure ron to parse");

        assert_eq!(structure.id, "shelter");
        assert_eq!(structure.required_components.len(), 3);
        assert_eq!(structure.build_ticks, 60);
        assert_eq!(structure.resource_costs.get("wood"), Some(&4.0));
        // blueprint field should default to None when omitted
        assert!(structure.blueprint.is_none());
    }

    #[test]
    fn parses_structure_def_with_blueprint() {
        let structure: StructureDef = ron::from_str(
            r#"StructureDef(
                id: "test_shelter",
                display_name_key: "BUILDING_TYPE_SHELTER",
                min_size: (5, 5),
                required_components: [
                    Wall(count: 8, tags: ["building_material"]),
                ],
                optional_components: [],
                role_recognition: Auto,
                build_ticks: 60,
                resource_costs: {},
                influence_when_complete: [],
                blueprint: Some(Blueprint(
                    walls: [
                        BlueprintTile(offset: (-2, -2), material_tag: "building_material"),
                        BlueprintTile(offset: (-1, -2), material_tag: "building_material"),
                        BlueprintTile(offset: (0, -2), material_tag: "building_material"),
                    ],
                    floors: [
                        BlueprintTile(offset: (0, 0), material_tag: "packed_earth"),
                        BlueprintTile(offset: (1, 0), material_tag: "packed_earth"),
                    ],
                    furniture: [
                        BlueprintFurniture(offset: (0, 0), furniture_id: "fire_pit"),
                    ],
                    doors: [(0, 2)],
                )),
            )"#,
        )
        .expect("expected structure with blueprint to parse");

        assert_eq!(structure.id, "test_shelter");
        assert!(structure.blueprint.is_some());

        let bp = structure.blueprint.as_ref().unwrap();
        assert_eq!(bp.walls.len(), 3);
        assert_eq!(bp.floors.len(), 2);
        assert_eq!(bp.furniture.len(), 1);
        assert_eq!(bp.doors.len(), 1);
        assert_eq!(bp.doors[0], (0, 2));
        assert_eq!(bp.furniture[0].furniture_id, "fire_pit");
        assert_eq!(bp.furniture[0].offset, (0, 0));
        assert_eq!(bp.floors[0].material_tag, "packed_earth");
    }
}
