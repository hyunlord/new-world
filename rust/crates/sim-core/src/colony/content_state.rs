//! Sparse persisted identity for data-defined finished construction and reusable tools.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{BlueprintKind, ItemKind};

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

/// Additional content state; bundled historical worlds leave both maps empty.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentState {
    /// Only authoritative completions; old material projections retain their history.
    pub completed: BTreeMap<((u32, u32), CompletedLayer), CompletedIdentity>,
    /// Non-default reusable cutter selected at each workshop.
    pub workshop_cutters: BTreeMap<u32, ItemKind>,
}

impl ContentState {
    /// Whether the world still has the historical implicit content representation.
    pub fn is_empty(&self) -> bool {
        self.completed.is_empty() && self.workshop_cutters.is_empty()
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
