use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// Typed material definition loaded from RON.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MaterialDef {
    /// Stable material identifier.
    pub id: String,
    /// Localization key for the display name.
    pub display_name_key: String,
    /// Coarse material family.
    pub category: MaterialCategory,
    /// Tag selectors used by recipes and content queries.
    pub tags: BTreeSet<String>,
    /// Numeric material properties used for derived stats.
    pub properties: MaterialProperties,
}

/// Core material property bundle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MaterialProperties {
    /// Approximate hardness in the 0.0..=10.0 range.
    pub hardness: f64,
    /// Density in g/cm^3 and always > 0.0.
    pub density: f64,
    /// Melting point in degrees Celsius when applicable.
    pub melting_point: Option<f64>,
    /// Relative rarity in the 0.0..=1.0 range.
    pub rarity: f64,
    /// Base economic value, always >= 0.0.
    pub value: f64,
}

/// Broad material categories for content grouping.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MaterialCategory {
    /// Stone-like materials.
    Stone,
    /// Wood-like materials.
    Wood,
    /// Metal materials.
    Metal,
    /// Animal-derived materials.
    Animal,
    /// Plant-derived materials.
    Plant,
    /// Earthen materials.
    Earth,
    /// Liquid materials.
    Liquid,
    /// Food materials.
    Food,
    /// Fuel materials.
    Fuel,
    /// Mineral materials.
    Mineral,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_material_def_from_ron() {
        let material: MaterialDef = ron::from_str(
            r#"MaterialDef(
                id: "flint",
                display_name_key: "MAT_FLINT",
                category: Stone,
                tags: ["stone", "sharp"],
                properties: MaterialProperties(
                    hardness: 7.0,
                    density: 2.6,
                    melting_point: None,
                    rarity: 0.3,
                    value: 2.5,
                ),
            )"#,
        )
        .expect("expected material ron to parse");

        assert_eq!(material.id, "flint");
        assert!(material.tags.contains("sharp"));
    }
}
