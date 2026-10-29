//! Rebuildable colony catchment capacity; never simulation or save authority.
use sim_core::components::SettlementId;
use sim_core::rules::CalendarRule;
use std::collections::BTreeMap;

/// Inputs which may change the fixed planted catchment's capacity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoodSupplyKey {
    /// Navigation cost generation.
    pub nav_version: u64,
    /// A cold navigation map admits no assumed reachable plants.
    pub nav_built: bool,
    /// Settlement formation tile and requested catchment radius.
    pub center_radius: ((u32, u32), u32),
    /// Seasonal phase arithmetic.
    pub calendar: CalendarRule,
    /// Active neighbouring catchments when fission sharing is enabled.
    pub sharing: Vec<(SettlementId, (u32, u32))>,
    /// Initial placement cardinality; plant identities are immutable after load.
    pub plant_count: usize,
}

/// One independently invalidated entry per settlement/radius query.
#[derive(Debug, Default)]
pub struct FoodSupplyCache {
    /// Key and four seasonal nominal edible-unit totals in millionths.
    pub entries: BTreeMap<(SettlementId, u32), (FoodSupplyKey, [u64; 4])>,
    /// Diagnostic rebuild count, excluded from persisted state.
    pub rebuilds: u64,
}
