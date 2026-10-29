//! Data-defined wild species; runtime state is stored separately by tile.
use super::super::ids::ItemKind;
use super::IdentityDef;
use crate::material::TerrainType;
use serde::Deserialize;

/// Seasonal state interpreted by the plant lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum PlantPhase {
    /// No visible growth or harvest.
    Dormant,
    /// Living growth before harvest.
    Growing,
    /// Harvestable while the ripe window is open and yield remains.
    Ripe,
    /// Exhausted seasonal growth.
    Spent,
}

/// Seeded tile admission and species distribution.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlantPlacement {
    /// Allowed physical terrain categories.
    pub terrains: Vec<TerrainType>,
    /// Inclusive elevation range.
    pub elevation: (u8, u8),
    /// Relative weight among eligible species.
    pub weight: u32,
    /// Occupancy contribution per thousand eligible tiles.
    pub density_permille: u32,
}

/// Validated species with item and knowledge references resolved locally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantDef {
    /// Stable explicit identity and presentation key.
    pub identity: IdentityDef,
    /// Presentation fallback category.
    pub category: String,
    /// Authored capability and presentation tags.
    pub tags: Vec<String>,
    /// Deterministic world generation distribution.
    pub placement: PlantPlacement,
    /// Spring, summer, autumn and winter phases.
    pub phases: [PlantPhase; 4],
    /// First ripe day within each ripe season.
    pub ripe_start_day: u32,
    /// Exclusive ripe day end within each ripe season; none uses season end.
    pub ripe_end_day: Option<u32>,
    /// Produced physical item.
    pub yield_item: ItemKind,
    /// Whole items per successful harvest.
    pub yield_count: u16,
    /// Days between harvest opportunities; zero means once per ripe window.
    pub regrow_days: u32,
    /// Base work ticks before practitioner speed adjustment.
    pub harvest_ticks: u32,
    /// Optional practice bit that speeds harvest.
    pub knowledge: Option<u8>,
    /// Practitioner speed multiplier in permille.
    pub knowledge_speed_permille: u32,
}
