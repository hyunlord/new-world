//! Data-authored cultivation inputs, work and physical outputs.
use super::super::ids::ItemKind;
use super::IdentityDef;
use serde::Deserialize;

/// One data-authored seasonal planting interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SowingWindow {
    /// Season index, spring through winter (0..4).
    pub season: u8,
    /// Inclusive day within the season.
    pub start_day: u32,
    /// Exclusive day within the season.
    pub end_day_exclusive: u32,
}
/// One physical component of a crop harvest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CropYield {
    /// Physical output item.
    pub item: ItemKind,
    /// Physical output pieces per harvest.
    pub count: u16,
}
/// Validated crop mechanics resolved against the selected catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CropDef {
    /// Stable selected-content identity.
    pub identity: IdentityDef,
    /// Presentation fallback category.
    pub category: String,
    /// Data-authored capability tags.
    pub tags: Vec<String>,
    /// Wild plant from which physical seed stock can originate.
    pub wild_source: Option<super::super::ids::PlantKind>,
    /// Physical input item consumed by sowing.
    pub seed_item: ItemKind,
    /// Physical input pieces required per sowing.
    pub seeds_per_tile: u16,
    /// Atomic physical output batches per tended harvest.
    pub yields: Vec<CropYield>,
    /// Allowed seasonal planting intervals.
    pub sow_windows: Vec<SowingWindow>,
    /// Whether winter destroys an established crop.
    pub frost_sensitive: bool,
    /// Whole days from sowing to maturity.
    pub growth_days: u32,
    /// Base work ticks for preparing soil.
    pub till_ticks: u32,
    /// Base work ticks after physical seed delivery.
    pub sow_ticks: u32,
    /// Base work ticks for one tending operation.
    pub tend_ticks: u32,
    /// Base work ticks before committing harvest output.
    pub harvest_ticks: u32,
    /// Untended output multiplier, rounded down per output.
    pub untended_yield_permille: u16,
    /// Soil depleted by a committed harvest.
    pub fertility_cost: u16,
    /// Daily soil restoration while fallow.
    pub fallow_recovery_per_day: u16,
    /// Soil fertility when first designated.
    pub initial_fertility: u16,
    /// Minimum soil fertility required to sow.
    pub min_fertility: u16,
    /// Required practitioner knowledge for cultivation work.
    pub knowledge: Option<u8>,
}
