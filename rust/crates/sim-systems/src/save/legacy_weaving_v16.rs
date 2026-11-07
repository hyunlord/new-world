//! Exact published format16 body and narrowly admitted predecessor catalog.
use super::{legacy_food_v16::FoodStateV16, world::WorldSave, Body, SaveError};
use serde::Deserialize;
use sim_core::colony::content::{Catalog, IdentityDef, RawSources};

const REVISION: &str = "catalog-v1-fnv1a64:6e0fe353840b9796:67358";

#[derive(Deserialize)]
struct LegacyBodyV16 {
    current_tick: u64,
    resources: sim_engine::legacy_v7::LegacySimResourcesV16,
    world: WorldSave,
    systems: Vec<(String, Vec<u8>)>,
    steward: sim_core::colony::steward::StewardState,
    content_state: sim_core::colony::content_state::ContentState,
    mind: sim_core::colony::mind_state::MindState,
    food: FoodStateV16,
    cultivation: sim_core::colony::cultivation::CultivationState,
}

pub(super) fn decode_body(payload: &[u8]) -> Result<Body, SaveError> {
    let old: LegacyBodyV16 = super::decode(payload).map_err(SaveError::Decode)?;
    Ok(Body {
        current_tick: old.current_tick,
        resources: old.resources.into(),
        world: old.world,
        systems: old.systems,
        steward: old.steward,
        content_state: old.content_state,
        mind: old.mind,
        food: old.food.into(),
        cultivation: old.cultivation,
        processing: Default::default(),
        buildings: Default::default(),
    })
}

fn identity(definition: Option<&IdentityDef>, code: u32, name: &str, serde_name: &str) -> bool {
    definition.is_some_and(|id| {
        id.code == code
            && id.id == name
            && id.serde_name == serde_name
            && id.serde_aliases.is_empty()
            && id.resolver_aliases.is_empty()
    })
}

pub(super) fn accepts_catalog(revision: &str, catalog: &Catalog) -> Result<bool, SaveError> {
    if revision != REVISION {
        return Ok(false);
    }
    let bundled = Catalog::from_sources(RawSources::bundled(), None)
        .map_err(|error| SaveError::Decode(error.to_string()))?;
    if catalog.revision() != bundled.revision() {
        return Ok(false);
    }
    let same = super::cultivation_catalog::identities_preserved(catalog)
        && [
            (30, "flax_seed", "FlaxSeed"),
            (31, "emmer_seed", "EmmerSeed"),
        ]
        .into_iter()
        .all(|(code, name, serde_name)| {
            identity(
                catalog.item_by_code(code).map(|d| &d.identity),
                code,
                name,
                serde_name,
            )
        })
        && [
            (18, "extract_flax_seed", "ExtractFlaxSeed"),
            (19, "extract_emmer_seed", "ExtractEmmerSeed"),
        ]
        .into_iter()
        .all(|(code, name, serde_name)| {
            identity(
                catalog.recipe_by_code(code).map(|d| &d.identity),
                code,
                name,
                serde_name,
            )
        })
        && [(0, "emmer", "Emmer"), (1, "flax", "Flax")]
            .into_iter()
            .all(|(code, name, serde_name)| {
                identity(
                    catalog.crop_by_code(code).map(|d| &d.identity),
                    code,
                    name,
                    serde_name,
                )
            });
    if !same {
        return Err(SaveError::Decode(
            "format16 catalog identities changed without a migration map".into(),
        ));
    }
    Ok(true)
}
