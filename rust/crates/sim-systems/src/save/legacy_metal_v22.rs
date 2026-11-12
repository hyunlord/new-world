//! Genuine C2-a format22 body: no metal tail and 26 completed-job counters.
use super::{world, Body, SaveError};
use serde::{Deserialize, Serialize};
use sim_core::colony::content::{Catalog, RawSources};
use sim_core::colony::skills::{Skill, Skills};
use sim_core::colony::JobKind;
use sim_engine::SimResources;

// Recorded from a save written by the unmodified C2-a candidate e91cc8df6 (seed0 walk fixture).
const REVISION: &str = "catalog-v1-fnv1a64:230d5c2e322d576e:231930";

#[derive(Serialize, Deserialize)]
struct LegacyBodyV22 {
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
    #[serde(deserialize_with = "super::knowledge_upper::deserialize")]
    neighbour_upper: super::knowledge_upper::UpperWords,
    ore: sim_core::colony::ore::OreState,
}

pub(super) fn decode_body(payload: &[u8]) -> Result<Body, SaveError> {
    let old: LegacyBodyV22 = super::decode(payload).map_err(SaveError::Decode)?;
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
        neighbour_upper: old.neighbour_upper,
        ore: old.ore,
        metal: Default::default(),
    })
}

#[derive(Serialize, Deserialize)]
struct SkillsV22 {
    entries: [Skill; 8],
    completed: [u32; 26],
}

/// Format21/22 Skills: copy the 26 historical counters; the appended mining counter starts at 0.
pub(super) fn skills_column(bytes: &[u8], expected_entities: usize) -> Result<Vec<u8>, String> {
    let old: Vec<SkillsV22> = super::decode(bytes)?;
    if old.len() != expected_entities {
        return Err("format22 Skills column entity count mismatch".into());
    }
    let current = old
        .into_iter()
        .map(|old| {
            let mut completed = [0; JobKind::NAMES.len()];
            completed[..26].copy_from_slice(&old.completed);
            Skills {
                entries: old.entries,
                completed,
            }
        })
        .collect::<Vec<_>>();
    super::encode(&current)
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
            "format22 identities changed without a migration map".into(),
        ));
    }
    Ok(true)
}
