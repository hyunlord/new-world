//! Exact published format-15 body before cultivation and origin accounting.
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub(super) struct LegacyBodyV15 {
    pub(super) current_tick: u64,
    pub(super) resources: sim_engine::legacy_v7::LegacySimResourcesV16,
    pub(super) world: super::world::WorldSave,
    pub(super) systems: Vec<(String, Vec<u8>)>,
    pub(super) steward: sim_core::colony::steward::StewardState,
    pub(super) content_state: sim_core::colony::content_state::ContentState,
    pub(super) mind: sim_core::colony::mind_state::MindState,
    pub(super) food: super::legacy_food_v16::FoodStateV16,
}
pub(super) fn decode_body(payload: &[u8]) -> Result<super::Body, super::SaveError> {
    let old: LegacyBodyV15 = super::decode(payload).map_err(super::SaveError::Decode)?;
    Ok(super::Body {
        current_tick: old.current_tick,
        resources: old.resources.into(),
        world: old.world,
        systems: old.systems,
        steward: old.steward,
        content_state: old.content_state,
        mind: old.mind,
        food: old.food.into(),
        cultivation: Default::default(),
        processing: Default::default(),
        buildings: Default::default(),
    })
}
