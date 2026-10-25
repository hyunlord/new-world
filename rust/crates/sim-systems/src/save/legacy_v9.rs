//! Frozen format9 body and historical tree migration; never decode it as current content state.
use serde::Deserialize;
use sim_core::colony::content_state::{CompletedIdentity, CompletedLayer, ContentState};
use sim_core::colony::materials::{embedded_definitions, MaterialStats, OAK};
use sim_core::colony::{story::PressureKind, ItemKind};
use sim_core::components::AgentId;
use sim_core::material::MaterialCategory;
use sim_engine::SimResources;
use std::collections::BTreeMap;

pub(super) const REVISION: &str = "catalog-v1-fnv1a64:10279805eb0560ff:44709";

#[derive(Deserialize)]
pub(super) struct LegacyContentStateV9 {
    completed: BTreeMap<((u32, u32), CompletedLayer), CompletedIdentity>,
    workshop_cutters: BTreeMap<u32, ItemKind>,
    illness_origins: BTreeMap<AgentId, PressureKind>,
}

impl From<LegacyContentStateV9> for ContentState {
    fn from(old: LegacyContentStateV9) -> Self {
        Self {
            completed: old.completed,
            workshop_cutters: old.workshop_cutters,
            illness_origins: old.illness_origins,
            tree_material_default: None,
            tree_materials: BTreeMap::new(),
        }
    }
}

#[derive(Deserialize)]
pub(super) struct LegacyBodyV9 {
    pub current_tick: u64,
    pub resources: SimResources,
    pub world: super::world::WorldSave,
    pub systems: Vec<(String, Vec<u8>)>,
    pub steward: super::legacy_steward::LegacyStewardState,
    pub content_state: LegacyContentStateV9,
}

/// Historical standing trees physically yielded oak regardless of old renderer guesses.
pub(super) fn migrate_trees(resources: &mut SimResources) -> Result<(), String> {
    if resources.rules.colony.is_some() {
        for definition in embedded_definitions()? {
            if definition.category == MaterialCategory::Wood {
                let stats = MaterialStats::from_def(&definition);
                resources
                    .colony
                    .materials
                    .by_id
                    .entry(definition.id)
                    .or_insert_with(|| (definition.name.to_lowercase(), stats));
            }
        }
    }
    if !resources.colony.trees.is_empty() {
        resources.content_state.tree_material_default = Some(OAK);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn historical_trees_are_oak_and_existing_material_stats_are_never_replaced() {
        let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        resources.rules.colony = Some(sim_core::rules::ColonyRule::M1);
        let mut old = sim_core::colony::ColonyMaterials::load()
            .by_id
            .remove(&OAK)
            .expect("oak");
        old.1.hardness = 123;
        resources.colony.materials.by_id.insert(OAK, old.clone());
        resources
            .colony
            .trees
            .insert((2, 3), sim_core::colony::Tree { logs: 4 });
        migrate_trees(&mut resources).expect("migration");
        assert_eq!(resources.colony.materials.by_id.get(&OAK), Some(&old));
        assert!(resources
            .colony
            .materials
            .stats(sim_core::material::MaterialId::from_str_hash("pine"))
            .is_some());
        assert_eq!(resources.content_state.tree_material((2, 3)), Some(OAK));
        assert!(resources.content_state.tree_materials.is_empty());
        assert_eq!(resources.colony.trees[&(2, 3)].logs, 4);
    }
}
