//! `TerrainType` — exactly 10 variants in §3.C order. Forbidden extras are
//! enumerated in the spec, not here, to avoid contaminating source grep checks.

use serde::{Deserialize, Serialize};

/// Terrain biome classification used by `MaterialDef::natural_in` and
/// `MaterialProperties::distribution`. Order locked by §3.C of the spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainType {
    /// Open plains.
    Plain,
    /// Forest cover.
    Forest,
    /// High-altitude rocky terrain.
    Mountain,
    /// Rolling lower-altitude relief.
    Hill,
    /// River bank / freshwater corridor.
    River,
    /// Sea coast.
    Coast,
    /// Arid sand / rock desert.
    Desert,
    /// Cold treeless tundra.
    Tundra,
    /// Wet lowland swamp.
    Swamp,
    /// Underground cave network.
    Cave,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_variants_match_exhaustively() {
        let all = [
            TerrainType::Plain,
            TerrainType::Forest,
            TerrainType::Mountain,
            TerrainType::Hill,
            TerrainType::River,
            TerrainType::Coast,
            TerrainType::Desert,
            TerrainType::Tundra,
            TerrainType::Swamp,
            TerrainType::Cave,
        ];
        for t in all {
            match t {
                TerrainType::Plain => {}
                TerrainType::Forest => {}
                TerrainType::Mountain => {}
                TerrainType::Hill => {}
                TerrainType::River => {}
                TerrainType::Coast => {}
                TerrainType::Desert => {}
                TerrainType::Tundra => {}
                TerrainType::Swamp => {}
                TerrainType::Cave => {}
            }
        }
    }
}
