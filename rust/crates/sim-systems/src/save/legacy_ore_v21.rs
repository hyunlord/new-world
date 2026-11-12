//! Genuine C1-b format21 body; no retrospective ore generation or upper-mask bits.
use super::{world, Body, SaveError};
use serde::{Deserialize, Serialize};
use sim_core::colony::content::{Catalog, RawSources};
use sim_engine::SimResources;

// Recorded from the unmodified 4a44336e7 probe's retained colony21.save envelope.
const REVISION: &str = "catalog-v1-fnv1a64:d06d8982b4e80005:170575";

#[derive(Serialize, Deserialize)]
struct LegacyBodyV21 {
    current_tick: u64,
    resources: SimResources,
    world: world::WorldSave,
    systems: Vec<(String, Vec<u8>)>,
    steward: sim_core::colony::steward::StewardState,
    content_state: sim_core::colony::content_state::ContentState,
    mind: sim_core::colony::mind_state::MindState,
    food: sim_core::colony::food::FoodState,
    cultivation: sim_core::colony::cultivation::CultivationState,
    processing: sim_core::colony::processing::ProcessingState,
    buildings: sim_core::colony::building_site::BuildingState,
    livestock: sim_core::colony::livestock::LivestockState,
}

pub(super) fn decode_body(payload: &[u8]) -> Result<Body, SaveError> {
    let old: LegacyBodyV21 = super::decode(payload).map_err(SaveError::Decode)?;
    Ok(Body {
        current_tick: old.current_tick,
        resources: old.resources,
        world: old.world,
        systems: old.systems,
        steward: old.steward,
        content_state: old.content_state,
        mind: old.mind,
        food: old.food,
        cultivation: old.cultivation,
        processing: old.processing,
        buildings: old.buildings,
        livestock: old.livestock,
        neighbour_upper: Default::default(),
        ore: Default::default(),
        metal: Default::default(),
    })
}

pub(super) fn accepts_catalog(revision: &str, catalog: &Catalog) -> Result<bool, SaveError> {
    if revision != REVISION {
        return Ok(false);
    }
    let bundled = Catalog::from_sources(RawSources::bundled(), None)
        .map_err(|error| SaveError::Decode(error.to_string()))?;
    if bundled.revision() != catalog.revision() {
        return Ok(false);
    }
    if !super::legacy_livestock_v20::identities_preserved(catalog) {
        return Err(SaveError::Decode(
            "format21 identities changed without a migration map".into(),
        ));
    }
    Ok(true)
}

#[cfg(test)]
#[path = "ore_save_tests.rs"]
mod tests;
