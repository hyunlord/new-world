//! Exact format17 body and component columns before data-driven era state.
use super::{world::WorldSave, Body, SaveError};
use serde::{Deserialize, Serialize};
use sim_core::colony::content::{Catalog, IdentityDef, RawSources};

const REVISION: &str = "catalog-v1-fnv1a64:69307027cd9a9f8e:76374";

#[derive(Deserialize)]
struct LegacyBodyV17 {
    current_tick: u64,
    resources: sim_engine::legacy_v7::LegacySimResourcesV17,
    world: WorldSave,
    systems: Vec<(String, Vec<u8>)>,
    steward: sim_core::colony::steward::StewardState,
    content_state: sim_core::colony::content_state::ContentState,
    mind: sim_core::colony::mind_state::MindState,
    food: sim_core::colony::food::FoodState,
    cultivation: sim_core::colony::cultivation::CultivationState,
    processing: sim_core::colony::processing::ProcessingState,
}

pub(super) fn decode_body(payload: &[u8]) -> Result<Body, SaveError> {
    let old: LegacyBodyV17 = super::decode(payload).map_err(SaveError::Decode)?;
    Ok(Body {
        current_tick: old.current_tick,
        resources: old.resources.into(),
        world: old.world,
        systems: old.systems,
        steward: old.steward,
        content_state: old.content_state,
        mind: old.mind,
        food: old.food,
        cultivation: old.cultivation,
        processing: old.processing,
        buildings: Default::default(),
        livestock: Default::default(),
    })
}

#[derive(Serialize, Deserialize)]
struct EntryV17 {
    last_used: u16,
    proficiency: u16,
    tag: u8,
    observed5: u8,
}

#[derive(Serialize, Deserialize)]
struct KnowledgeV17 {
    entries: Vec<EntryV17>,
    practice: u32,
}

/// Re-encode the old opaque column, preserving observation debt and source bits.
pub(super) fn knowledge_column(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let old: Vec<KnowledgeV17> = super::decode(bytes)?;
    let upgraded: Vec<_> = old
        .into_iter()
        .map(|knowledge| {
            let entries: Vec<_> = knowledge
                .entries
                .into_iter()
                .map(|entry| {
                    (
                        entry.last_used,
                        entry.proficiency,
                        entry.tag,
                        entry.observed5,
                        0u8,
                    )
                })
                .collect();
            (entries, u64::from(knowledge.practice))
        })
        .collect();
    super::encode(&upgraded)
}

pub(super) fn skills_column(bytes: &[u8]) -> Result<Vec<u8>, String> {
    use sim_core::colony::{skills::Skill, JobKind, SkillKind};
    #[derive(Deserialize)]
    struct SkillsV17 {
        entries: [Skill; SkillKind::ALL.len()],
    }
    let old: Vec<SkillsV17> = super::decode(bytes)?;
    let upgraded: Vec<_> = old
        .into_iter()
        .map(|skills| (skills.entries, [0u32; JobKind::NAMES.len()]))
        .collect();
    super::encode(&upgraded)
}

/// Historical test writer: old columns explicitly omit the new counters/id high bits.
#[cfg(test)]
pub(super) fn old_knowledge_column(bytes: &[u8]) -> Result<Vec<u8>, String> {
    use sim_core::colony::knowledge::Knowledge;
    let current: Vec<Knowledge> = super::decode(bytes)?;
    let old = current
        .into_iter()
        .map(|knowledge| {
            let practice = u32::try_from(knowledge.practice())
                .map_err(|_| "new knowledge mask cannot fit format17".to_owned())?;
            let entries = knowledge
                .entries()
                .iter()
                .map(|entry| {
                    if entry.id() > 31 {
                        return Err("new knowledge id cannot fit format17".to_owned());
                    }
                    Ok(EntryV17 {
                        last_used: entry.last_used,
                        proficiency: entry.proficiency,
                        tag: entry.id() | (entry.source().code() << 5),
                        observed5: u8::try_from(entry.observed() / 5).map_err(|e| e.to_string())?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(KnowledgeV17 { entries, practice })
        })
        .collect::<Result<Vec<_>, String>>()?;
    super::encode(&old)
}

#[cfg(test)]
pub(super) fn old_skills_column(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let current: Vec<sim_core::colony::Skills> = super::decode(bytes)?;
    super::encode(
        &current
            .into_iter()
            .map(|skills| (skills.entries,))
            .collect::<Vec<_>>(),
    )
}

fn identity(definition: Option<&IdentityDef>, row: (u32, &str, &str)) -> bool {
    definition.is_some_and(|id| {
        id.code == row.0
            && id.id == row.1
            && id.serde_name == row.2
            && id.serde_aliases.is_empty()
            && id.resolver_aliases.is_empty()
    })
}

/// Admit only the exact published bundled predecessor into the current bundle.
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
            (32, "flax_fibre", "FlaxFibre"),
            (33, "linen_thread", "LinenThread"),
            (34, "linen_cloth", "LinenCloth"),
            (35, "hand_spindle", "HandSpindle"),
            (36, "linen_tunic", "LinenTunic"),
            (37, "linen_wrap", "LinenWrap"),
        ]
        .into_iter()
        .all(|row| identity(catalog.item_by_code(row.0).map(|d| &d.identity), row))
        && [
            (18, "extract_flax_seed", "ExtractFlaxSeed"),
            (19, "extract_emmer_seed", "ExtractEmmerSeed"),
            (20, "ret_flax", "RetFlax"),
            (21, "spin_thread", "SpinThread"),
            (22, "weave_cloth", "WeaveCloth"),
            (23, "make_linen_tunic", "MakeLinenTunic"),
            (24, "make_linen_wrap", "MakeLinenWrap"),
            (25, "make_hand_spindle", "MakeHandSpindle"),
        ]
        .into_iter()
        .all(|row| identity(catalog.recipe_by_code(row.0).map(|d| &d.identity), row))
        && [(0, "emmer", "Emmer"), (1, "flax", "Flax")]
            .into_iter()
            .all(|row| identity(catalog.crop_by_code(row.0).map(|d| &d.identity), row))
        && [
            (27, "cultivation"),
            (28, "spinning"),
            (29, "weaving"),
            (30, "tailoring"),
        ]
        .into_iter()
        .all(|(code, name)| {
            catalog
                .knowledge_defs()
                .nodes
                .iter()
                .any(|node| node.id == code && node.name == name)
        });
    if !same {
        return Err(SaveError::Decode(
            "format17 catalog identities changed without a migration map".into(),
        ));
    }
    Ok(true)
}

#[cfg(test)]
#[path = "legacy_spine_tests.rs"]
mod tests;
