//! `MaterialDef` — exactly 7 fields in §3.F order. Add/remove/reorder X.

use serde::{Deserialize, Serialize};

use crate::material::category::MaterialCategory;
use crate::material::id::MaterialId;
use crate::material::properties::MaterialProperties;
use crate::material::terrain::TerrainType;

/// One material definition: identity + classification + physical
/// properties + tier + biome distribution + optional mod source.
///
/// Field order is locked by §3.F of the material schema spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDef {
    /// Content-derived 32-bit id (djb2 of the source string).
    pub id: MaterialId,
    /// Human-readable name (UI fallback; localisation is a separate concern).
    pub name: String,
    /// High-level classification.
    pub category: MaterialCategory,
    /// Physical / ecological / cultural properties.
    pub properties: MaterialProperties,
    /// Tier (0..=255). Higher = more advanced / rarer.
    pub tier: u8,
    /// Terrains where the material naturally appears.
    #[serde(default)]
    pub natural_in: Vec<TerrainType>,
    /// Mod identifier; `None` means base game.
    #[serde(default)]
    pub mod_source: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::properties::test_support::valid_props;

    fn sample() -> MaterialDef {
        MaterialDef {
            id: MaterialId::from_str_hash("iron"),
            name: "Iron".to_string(),
            category: MaterialCategory::Mineral,
            properties: valid_props(),
            tier: 3,
            natural_in: vec![TerrainType::Mountain],
            mod_source: None,
        }
    }

    #[test]
    fn seven_named_fields_destructure_in_locked_order() {
        let def = sample();
        let MaterialDef {
            id: _x0,
            name: _x1,
            category: _x2,
            properties: _x3,
            tier: _x4,
            natural_in: _x5,
            mod_source: _x6,
        } = def;
        let _: MaterialId = _x0;
        let _: String = _x1;
        let _: MaterialCategory = _x2;
        let _: MaterialProperties = _x3;
        let _: u8 = _x4;
        let _: Vec<TerrainType> = _x5;
        let _: Option<String> = _x6;
    }

    #[test]
    fn natural_in_default_empty_vec() {
        // Round-trip through RON without natural_in set — serde default Vec.
        let ron = r#"(
            id: 1,
            name: "x",
            category: stone,
            properties: (
                density: 100.0,
                hardness: 1.0,
                shear_yield: 1000.0,
                impact_yield: 1000.0,
                fracture_toughness: 1000.0,
                melting_point: 0.0,
                flammability: 0.0,
                thermal_conductivity: 0.04,
                cultural_value: 0.0,
                rarity: 0.0,
                work_difficulty: 0.0,
                aesthetic_value: 0.0,
                workability: 0.0,
                preservation: 0.0,
            ),
            tier: 0,
        )"#;
        let def: MaterialDef = ron::from_str(ron).expect("parse default");
        assert!(def.natural_in.is_empty());
        assert!(def.mod_source.is_none());
    }
}
