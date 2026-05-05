//! `MaterialCategory` — exactly 6 variants, last carries a mod-defined byte.

use serde::{Deserialize, Serialize};

/// High-level material classification. The order is locked by §3.B of the
/// material schema spec — do not reorder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialCategory {
    /// Stone-like inorganic (granite, basalt, ...).
    Stone,
    /// Wood from trees (oak, pine, ...).
    Wood,
    /// Animal-derived (bone, leather, ...).
    Animal,
    /// Refined mineral / metal (iron, copper, ...).
    Mineral,
    /// Plant-derived non-wood (cotton, flax, ...).
    Plant,
    /// Mod-defined category id (0..=255).
    Mod(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_variants_match_exhaustively() {
        let values = [
            MaterialCategory::Stone,
            MaterialCategory::Wood,
            MaterialCategory::Animal,
            MaterialCategory::Mineral,
            MaterialCategory::Plant,
            MaterialCategory::Mod(0),
        ];
        for v in values {
            // Exhaustive match — adding a 7th variant breaks compilation.
            match v {
                MaterialCategory::Stone => {}
                MaterialCategory::Wood => {}
                MaterialCategory::Animal => {}
                MaterialCategory::Mineral => {}
                MaterialCategory::Plant => {}
                MaterialCategory::Mod(_) => {}
            }
        }
    }

    #[test]
    fn mod_carries_u8() {
        let _: MaterialCategory = MaterialCategory::Mod(0u8);
        let _: MaterialCategory = MaterialCategory::Mod(255u8);
    }

    #[test]
    fn mod_255_ron_roundtrip() {
        let original = MaterialCategory::Mod(255u8);
        let ser = ron::ser::to_string(&original).expect("serialize Mod(255)");
        let back: MaterialCategory = ron::de::from_str(&ser).expect("deserialize Mod(255)");
        assert_eq!(back, original);
        match back {
            MaterialCategory::Mod(n) => {
                let _: u8 = n;
                assert_eq!(n, 255u8);
            }
            other => panic!("expected Mod(255), got {other:?}"),
        }
    }

    #[test]
    fn mod_0_ron_roundtrip() {
        let original = MaterialCategory::Mod(0u8);
        let ser = ron::ser::to_string(&original).expect("serialize Mod(0)");
        let back: MaterialCategory = ron::de::from_str(&ser).expect("deserialize Mod(0)");
        assert_eq!(back, original);
    }
}
