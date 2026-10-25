//! Sparse persisted identity for data-defined finished construction and reusable tools.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{BlueprintKind, ItemKind};
use crate::material::MaterialId;

/// Independent physical slots on a tile; removing a bed must preserve its floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CompletedLayer {
    /// A wall or door.
    Wall,
    /// A floor.
    Floor,
    /// A bed or bedroll.
    Bed,
    /// A dwelling or workshop occupying the tile.
    Structure,
}

/// Authoritative identity retained when appearance alone cannot identify content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletedIdentity {
    /// The completed registered blueprint.
    pub blueprint: BlueprintKind,
    /// The registered construction style.
    pub style_id: String,
}

/// Additional content identity retained independently of physical projections.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentState {
    /// Only authoritative completions; old material projections retain their history.
    pub completed: BTreeMap<((u32, u32), CompletedLayer), CompletedIdentity>,
    /// Non-default reusable cutter selected at each workshop.
    pub workshop_cutters: BTreeMap<u32, ItemKind>,
    /// Novel illness origin retained until recovery or death; default illness stays implicit.
    pub illness_origins: BTreeMap<crate::components::AgentId, super::story::PressureKind>,
    /// Persisted compression default; never reinterpreted using the installed catalog.
    pub tree_material_default: Option<MaterialId>,
    /// Non-default material for each standing tree.
    pub tree_materials: BTreeMap<(u32, u32), MaterialId>,
}

impl ContentState {
    /// Effective persisted material; callers separately establish tree membership.
    pub fn tree_material(&self, tile: (u32, u32)) -> Option<MaterialId> {
        self.tree_materials
            .get(&tile)
            .copied()
            .or(self.tree_material_default)
    }

    /// Record a standing tree against its already selected persisted default.
    pub fn record_tree_material(
        &mut self,
        tile: (u32, u32),
        material: MaterialId,
    ) -> Result<(), &'static str> {
        let default = self
            .tree_material_default
            .ok_or("tree material default is missing")?;
        if material == default {
            self.tree_materials.remove(&tile);
        } else {
            self.tree_materials.insert(tile, material);
        }
        Ok(())
    }

    /// Remove only this tree's override, retaining the world's compression default.
    pub fn forget_tree_material(&mut self, tile: (u32, u32)) {
        self.tree_materials.remove(&tile);
    }

    /// Whether the world still has the historical implicit content representation.
    pub fn is_empty(&self) -> bool {
        self.completed.is_empty()
            && self.workshop_cutters.is_empty()
            && self.illness_origins.is_empty()
            && self.tree_material_default.is_none()
            && self.tree_materials.is_empty()
    }
}

/// Decode a numeric recipe slot without changing its historical Debug/serde spelling.
pub fn deserialize_recipe_code<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let code = u32::deserialize(deserializer)?;
    super::content::ids::RecipeKind::checked(code).map_err(serde::de::Error::custom)?;
    Ok(code)
}

/// Historical per-agent recipe quality accumulators.
pub type CraftQuality = BTreeMap<(crate::components::AgentId, u32), (u64, u64)>;

/// Validate raw numeric recipe keys while retaining the existing tuple-key map format.
pub fn deserialize_craft_quality<'de, D>(deserializer: D) -> Result<CraftQuality, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let values = CraftQuality::deserialize(deserializer)?;
    for (_, code) in values.keys() {
        super::content::ids::RecipeKind::checked(*code).map_err(serde::de::Error::custom)?;
    }
    Ok(values)
}

#[cfg(test)]
mod tree_tests {
    use super::*;
    #[test]
    fn sparse_tree_identity_requires_default_and_removes_only_its_override() {
        let mut state = ContentState::default();
        let oak = MaterialId::from_str_hash("oak");
        let pine = MaterialId::from_str_hash("pine");
        assert!(state.record_tree_material((1, 2), pine).is_err());
        assert!(state.is_empty());
        state.tree_material_default = Some(oak);
        state.record_tree_material((1, 2), pine).expect("default");
        state.record_tree_material((2, 2), oak).expect("default");
        assert_eq!(state.tree_materials.len(), 1);
        assert_eq!(state.tree_material((1, 2)), Some(pine));
        assert_eq!(state.tree_material((2, 2)), Some(oak));
        state.forget_tree_material((1, 2));
        assert!(state.tree_materials.is_empty());
        assert_eq!(state.tree_material_default, Some(oak));
        assert!(!state.is_empty());
    }
}
