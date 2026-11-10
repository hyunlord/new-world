//! Data-authored surface mineral deposits and their discovery policy.
use super::{IdentityDef, ItemArt, SpeciesOrigin};
use crate::colony::ItemKind;
use crate::{
    colony::knowledge::{ContentStatus, KnowledgeId},
    material::MaterialId,
};

/// One resolved ore kind; mechanics never dispatch on its name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutcropDef {
    /// Stable family identity.
    pub identity: IdentityDef,
    /// Resolver fallback category.
    pub category: String,
    /// Authored capability labels.
    pub tags: Vec<String>,
    /// Connected kinds populate new colony worlds.
    pub status: ContentStatus,
    /// Deferred behavior and owning pass.
    pub debt: Option<String>,
    /// Shared C1 physical habitat.
    pub origin: SpeciesOrigin,
    /// Inclusive count of clusters per admitted world.
    pub per_world: (u16, u16),
    /// Inclusive tiles per cluster.
    pub cluster: (u16, u16),
    /// Independent world admission probability, permille.
    pub presence_permille: u16,
    /// Maximum Chebyshev distance from the founding camp.
    pub camp_radius: u32,
    /// Initial extractable units on each occupied tile.
    pub units: u16,
    /// Physical material of extracted ore.
    pub material: MaterialId,
    /// Registered renderer tint identity.
    pub material_name: String,
    /// Resolved output item for the ordinary mining job.
    pub yield_item: ItemKind,
    /// Personal practice admitting walking discovery.
    pub knowledge: KnowledgeId,
    /// Personal practice admitting discovery on quarry completion.
    pub quarry_knowledge: KnowledgeId,
    /// Walking discovery radius in tiles.
    pub reveal_radius: u32,
    /// Completed quarry discovery radius in tiles.
    pub quarry_radius: u32,
    /// Data-authored rock placeholder.
    pub art: ItemArt,
    /// Short visible deposit mark.
    pub glyph: String,
}
