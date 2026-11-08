//! Physical habitat admission shared by data-authored wild plants and animals.
use crate::material::TerrainType;
use serde::Deserialize;

/// The three world-variety climate bands, independent of content identities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ClimateBand {
    /// World variety climate zero.
    Mild,
    /// World variety climate one.
    Neutral,
    /// World variety climate two.
    Harsh,
}

impl ClimateBand {
    /// Decode the existing world variety climate index.
    pub const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::Mild),
            1 => Some(Self::Neutral),
            2 => Some(Self::Harsh),
            _ => None,
        }
    }
}

/// Authored habitat; omission retains historical species admission.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpeciesOrigin {
    /// Allowed world climates.
    pub climate: Vec<ClimateBand>,
    /// Allowed current ground classes.
    pub terrain: Vec<TerrainType>,
    /// Inclusive height bounds; no bound admits every height.
    pub elevation: Option<(u8, u8)>,
}

impl Default for SpeciesOrigin {
    fn default() -> Self {
        Self {
            climate: vec![ClimateBand::Mild, ClimateBand::Neutral, ClimateBand::Harsh],
            terrain: vec![
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
            ],
            elevation: None,
        }
    }
}

impl SpeciesOrigin {
    /// Admit a tile without drawing randomness or altering simulation state.
    pub fn admits(&self, climate: u8, terrain: TerrainType, elevation: u8) -> bool {
        ClimateBand::from_index(climate).is_some_and(|band| self.climate.contains(&band))
            && self.terrain.contains(&terrain)
            && self
                .elevation
                .is_none_or(|(lo, hi)| (lo..=hi).contains(&elevation))
    }

    /// Reject empty, duplicated or inverted habitat declarations.
    pub fn valid(&self) -> bool {
        !self.climate.is_empty()
            && !self.terrain.is_empty()
            && self
                .climate
                .iter()
                .enumerate()
                .all(|(i, value)| !self.climate[..i].contains(value))
            && self
                .terrain
                .iter()
                .enumerate()
                .all(|(i, value)| !self.terrain[..i].contains(value))
            && self.elevation.is_none_or(|(lo, hi)| lo <= hi)
    }
}
