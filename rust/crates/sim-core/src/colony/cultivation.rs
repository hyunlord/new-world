//! Persistent physical fields, soil and bounded food-origin accounting.
use super::content::{CropKind, ItemKind};
use super::food::FoodOwner;
use crate::components::SettlementId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Calendar-driven crop lifecycle; work transitions are committed by jobs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldPhase {
    /// Designated but not prepared.
    Untilled,
    /// Soil prepared for physical seed delivery.
    Tilled,
    /// Seed consumed; growth begins on the next lifecycle update.
    Sown {
        /// Calendar day of committed sowing.
        planted_day: u32,
    },
    /// Established crop accumulating daily growth.
    Growing {
        /// Calendar day of committed sowing.
        planted_day: u32,
        /// Accumulated whole growth days.
        growth_days: u32,
    },
    /// Ready for a harvest job.
    Ripe {
        /// Calendar day the crop matured.
        ripe_day: u32,
    },
    /// Physical output committed; stubble remains until daily update.
    Harvested {
        /// Calendar day of the committed harvest.
        harvest_day: u32,
    },
    /// Resting soil without an established crop.
    Fallow,
}
/// Scheduling preference, never permission to bypass physical prerequisites.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldWork {
    /// Ordinary autonomous scheduling.
    #[default]
    Auto,
    /// One explicitly requested feasible job before returning to Auto.
    Now,
    /// No field work until resumed.
    Paused,
}
/// Erasing a designation leaves both the established crop and soil intact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldTile {
    /// Settlement that owns the field.
    pub owner: SettlementId,
    /// Selected stable crop identity.
    pub crop: CropKind,
    /// Current physical lifecycle phase.
    pub phase: FieldPhase,
    /// Whether field work is designated.
    pub active: bool,
    /// Accepted scheduling preference.
    pub work: FieldWork,
    /// Whether this cycle completed its tending job.
    pub tended: bool,
}
/// Outward-rounded fraction of physical food attributable to cultivated crops.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OriginShare {
    /// Conservative lower share, in millionths.
    pub lower: u32,
    /// Conservative upper share, in millionths.
    pub upper: u32,
}
impl OriginShare {
    /// One whole share in millionths.
    pub const SCALE: u32 = 1_000_000;
    /// Pure cultivated origin.
    pub const FIELD: Self = Self {
        lower: Self::SCALE,
        upper: Self::SCALE,
    };
}
/// Actual completed meals by calendar year; bounds retain mixing uncertainty.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EatenTotals {
    /// Total nominal food units actually eaten.
    pub units: u64,
    /// Lower attributed eaten units multiplied by SCALE.
    pub field_lower: u64,
    /// Upper attributed eaten units multiplied by SCALE.
    pub field_upper: u64,
}
/// Cumulative committed work and physical seed/output accounting.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CultivationCounters {
    /// Completed tilling jobs.
    pub tiles_tilled: u64,
    /// Completed sowing jobs.
    pub sowings: u64,
    /// Physical seed pieces consumed by crop.
    pub seed_units_sown: BTreeMap<CropKind, u64>,
    /// Nominal food units consumed as seeds, never meals.
    pub seeded_food_units: u64,
    /// Physical seed pieces lost when a dead carrier cannot drop them.
    pub seed_lost: u64,
    /// Completed harvest jobs.
    pub harvests: u64,
    /// Physical output pieces committed by item.
    pub harvested_units: BTreeMap<ItemKind, u64>,
}
/// Explicit version-16 save tail; empty for pre-cultivation worlds.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CultivationState {
    /// Established fields, including erased designations.
    pub fields: BTreeMap<(u32, u32), FieldTile>,
    /// Persistent soil fertility in permille, including erased tiles.
    pub fertility: BTreeMap<(u32, u32), u16>,
    /// Cumulative committed cultivation accounting.
    pub counters: CultivationCounters,
    /// Last completed daily lifecycle update.
    pub last_update_day: Option<u32>,
    /// Shares attached to physical food owners.
    pub food_origins: BTreeMap<FoodOwner, OriginShare>,
    /// Shares held by active crafting agents after ingredient debit.
    pub craft_origins: BTreeMap<u64, OriginShare>,
    /// Shares aligned with FoodState pending drying batches.
    pub drying_origins: Vec<OriginShare>,
    /// Qualified expired food units by calendar day and settlement.
    pub expiry_hunger_days: BTreeMap<(u32, u32), u64>,
    /// Actual meal totals by calendar year.
    pub eaten_by_year: BTreeMap<u32, EatenTotals>,
}
