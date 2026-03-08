use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::TagRequirement;

/// Furniture or equipment content definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FurnitureDef {
    /// Stable furniture identifier.
    pub id: String,
    /// Localization key for the furniture name.
    pub display_name_key: String,
    /// Material requirements for construction.
    pub required_materials: Vec<TagRequirement>,
    /// Tile footprint in the world.
    pub size: (u32, u32),
    /// Influence emissions contributed by this furniture.
    #[serde(default)]
    pub influence_emissions: Vec<InfluenceEmission>,
    /// Optional room role contribution tag.
    pub role_contribution: Option<String>,
    /// Flexible stat map for content tuning.
    #[serde(default)]
    pub stats: BTreeMap<String, f64>,
}

/// Static influence emission descriptor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InfluenceEmission {
    /// Influence channel id.
    pub channel: String,
    /// Influence radius in tiles.
    pub radius: f64,
    /// Emission intensity scalar.
    pub intensity: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_furniture_def_from_ron() {
        let furniture: FurnitureDef = ron::from_str(
            r#"FurnitureDef(
                id: "fire_pit",
                display_name_key: "FURN_FIRE_PIT",
                required_materials: [],
                size: (1, 1),
                influence_emissions: [
                    InfluenceEmission(channel: "warmth", radius: 6.0, intensity: 0.8),
                ],
                role_contribution: Some("hearth"),
                stats: {
                    "durability": 10.0,
                },
            )"#,
        )
        .expect("expected furniture ron to parse");

        assert_eq!(furniture.id, "fire_pit");
        assert_eq!(furniture.size, (1, 1));
    }
}
