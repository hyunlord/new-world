//! Validated flora membership and integer weighted selection over existing wood materials.
use crate::material::{MaterialId, TerrainType};

/// An admitted existing physical Wood material, in canonical name order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloraSpecies {
    /// Stable physical material identity.
    pub material: MaterialId,
    /// Canonical lowercase physical material name.
    pub name: String,
}
/// Positive authored weight for one admitted member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloraWeight {
    /// Admitted material identity.
    pub material: MaterialId,
    /// Relative integer weight.
    pub weight: u32,
}
/// Validated inclusive elevation band with canonical-name ordered weights.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloraDistribution {
    /// Supported terrain for this band.
    pub terrain: TerrainType,
    /// Inclusive elevation bounds.
    pub elevation: (u8, u8),
    pub(in crate::colony::content) weights: Vec<FloraWeight>,
    pub(in crate::colony::content) cumulative: Vec<u64>,
    pub(in crate::colony::content) total: u64,
}
impl FloraDistribution {
    /// Canonical material-name ordered authored weights.
    pub fn weights(&self) -> &[FloraWeight] {
        &self.weights
    }
    /// Strictly positive checked sum of all weights.
    pub const fn total_weight(&self) -> u64 {
        self.total
    }
    /// Resolve an already bounded draw; out-of-range values are rejected.
    pub fn select(&self, draw: u64) -> Option<MaterialId> {
        self.cumulative
            .iter()
            .position(|&end| draw < end)
            .and_then(|index| self.weights.get(index))
            .map(|weight| weight.material)
    }
}
/// Immutable resolved flora policy; no global registry or random state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloraCatalog {
    pub(in crate::colony::content) default: MaterialId,
    pub(in crate::colony::content) species: Vec<FloraSpecies>,
    pub(in crate::colony::content) distributions: Vec<FloraDistribution>,
}
impl FloraCatalog {
    /// Compression default copied into each newly generated world's persisted state.
    pub const fn default_material(&self) -> MaterialId {
        self.default
    }
    /// Admitted members sorted by canonical name.
    pub fn species(&self) -> &[FloraSpecies] {
        &self.species
    }
    /// Lookup admitted physical identity.
    pub fn species_by_material(&self, material: MaterialId) -> Option<&FloraSpecies> {
        self.species
            .iter()
            .find(|species| species.material == material)
    }
    /// Lookup exact canonical physical name; aliases are not implicit admissions.
    pub fn species_by_name(&self, name: &str) -> Option<&FloraSpecies> {
        self.species.iter().find(|species| species.name == name)
    }
    /// Unique supported band, or None for terrain without a flora policy.
    pub fn distribution(&self, terrain: TerrainType, elevation: u8) -> Option<&FloraDistribution> {
        self.distributions.iter().find(|band| {
            band.terrain == terrain
                && band.elevation.0 <= elevation
                && elevation <= band.elevation.1
        })
    }
}
