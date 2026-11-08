//! Authored building templates expand into existing blueprint pieces; never new placeable kinds.
use crate::colony::knowledge::ContentStatus;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod catalogue;
mod geometry;
#[cfg(test)]
mod tests;
pub use catalogue::TemplateCatalog;
pub use geometry::{material_cost, PlannedPiece};

/// Authored automatic-growth zone; only housing has an executable driver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildingZoneKind {
    /// Stable catalogue id, independent of serialized physical ZoneKind.
    pub id: String,
    /// Localisation key.
    pub name_key: String,
    /// Reserved definitions are visible but cannot issue placement commands.
    pub status: ContentStatus,
    /// Missing behavior and planned pass for reserved definitions.
    pub debt: Option<String>,
}

/// Content categories select existing building functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingCategory {
    /// A dwelling, associated with its existing room or hut.
    Dwelling,
    /// A stockpile enclosure.
    Storage,
    /// A sheltered workshop.
    Workshop,
    /// A future civic or utility building.
    Utility,
    /// A future ritual/government building.
    Civic,
    /// A future trade building.
    Trade,
}

/// A selector expands deterministically in row-major tile order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PieceAt {
    /// Entire existing blueprint at the template anchor (the legacy hut).
    Anchor,
    /// Perimeter, excluding explicitly authored edge placements.
    Ring,
    /// Centre tile of the named perimeter edge.
    Edge(Direction),
    /// Every interior tile, or the first count tiles if count is specified.
    Interior,
}

/// Edge selectors rotate with the footprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// Upper edge.
    North,
    /// Right edge.
    East,
    /// Lower edge.
    South,
    /// Left edge.
    West,
}

/// A single authored piece entry, before geometric expansion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplatePiece {
    /// Existing blueprint id; unresolved only when the template is inert.
    pub kind: String,
    /// Geometric selector.
    pub at: PieceAt,
    /// Optional cap on the number of selected tiles (e.g. two bedrolls).
    #[serde(
        default,
        deserialize_with = "deserialize_count",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_count"
    )]
    pub count: Option<u32>,
}

/// Existing shelter semantics, without adding a temperature model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShelterKind {
    /// An enclosure using existing room roof/sleep mechanics.
    Roofed,
    /// An unroofed future utility.
    Open,
}

/// Informative function metadata; only existing capabilities are connected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildingFunction {
    /// Existing beds expected in the template.
    pub beds: u16,
    /// Optional stockpile item filter, using existing stockpile semantics.
    pub storage: Option<String>,
    /// Maximum captive animals; zero leaves the template outside livestock selection.
    #[serde(default)]
    pub livestock_capacity: u16,
    /// Existing station tags.
    pub workshop_tags: Vec<String>,
    /// Existing shelter capability.
    pub warmth: ShelterKind,
}

/// Roof silhouettes distinguish stone-age and metal-age placeholders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoofShape {
    /// Paleolithic/neolithic silhouette.
    Round,
    /// Bronze/iron silhouette.
    Angular,
}

/// Data-only placeholder art; icon refers to the glyph data file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildingArt {
    /// Footprint-sized roof geometry.
    pub roof: RoofShape,
    /// Material registry id used to resolve a colour.
    pub tint_material: String,
    /// One of the authored glyph ids.
    pub icon: String,
}

/// A named arrangement of existing piece blueprints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildingTemplate {
    /// Stable content id; no enum or code branch per building.
    pub id: String,
    /// Localisation key.
    pub name_key: String,
    /// Era registry id.
    pub era: String,
    /// Existing function family.
    pub category: BuildingCategory,
    /// Content ladder tier, 1..=4.
    pub tier: u8,
    /// External perimeter width and height, in tiles.
    pub footprint: (u32, u32),
    /// Blueprint plan in authored order.
    pub pieces: Vec<TemplatePiece>,
    /// Alternative interior stations; first entry is deterministic default.
    pub stations: Vec<String>,
    /// Settlement knowledge required to select this template.
    pub requires_knowledge: Vec<String>,
    /// Informative exact sum for the default expanded plan.
    pub materials: BTreeMap<String, u32>,
    /// Existing functional capabilities.
    pub function: BuildingFunction,
    /// Previous template; absent for an initial stage.
    pub upgrade_from: Option<String>,
    /// Next template; absent for final stages.
    pub upgrade_to: Option<String>,
    /// Automatic growth designation, when connected.
    pub zone_kind: Option<String>,
    /// Grey templates remain visible and unplaceable.
    pub status: ContentStatus,
    /// Missing mechanisms and planned pass for the debt ledger.
    pub debt: Option<String>,
    /// Placeholder display metadata.
    pub art: BuildingArt,
}

fn deserialize_count<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<u32>, D::Error> {
    u32::deserialize(deserializer).map(Some)
}

fn serialize_count<S: serde::Serializer>(
    count: &Option<u32>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    count.unwrap_or_default().serialize(serializer)
}
