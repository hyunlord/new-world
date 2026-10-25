//! Construction categories and rule-sensitive blueprint metadata.
use super::super::ids::{ItemKind, RecipeKind};
use super::IdentityDef;
use serde::Deserialize;

/// Supported PlacementCategory mechanics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum PlacementCategory {
    /// Structure mechanics.
    Structure,
    /// Wall mechanics.
    Wall,
    /// Floor mechanics.
    Floor,
    /// Furnishing mechanics.
    Furnishing,
}

/// Supported CompletionCategory mechanics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum CompletionCategory {
    /// Dwelling mechanics.
    Dwelling,
    /// Workshop mechanics.
    Workshop,
    /// Wall mechanics.
    Wall,
    /// Door mechanics.
    Door,
    /// Floor mechanics.
    Floor,
    /// Bed mechanics.
    Bed,
}

/// Supported PlannedNav mechanics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum PlannedNav {
    /// Unchanged mechanics.
    Unchanged,
    /// Blocked mechanics.
    Blocked,
    /// Door mechanics.
    Door,
}

/// Supported CompletionPolicy mechanics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum CompletionPolicy {
    /// HistoricalProjection mechanics.
    HistoricalProjection,
    /// Authoritative mechanics.
    Authoritative,
}

/// Source of authoritative completed material.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CompletionMaterial {
    /// First delivered material, or a validated default.
    FirstDelivered {
        /// Registered fallback material.
        fallback: String,
    },
    /// Fixed registered material.
    Fixed {
        /// Registered material.
        material: String,
    },
}

/// Validated BlueprintNeed metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintNeed {
    /// Item.
    pub item: ItemKind,
    /// Count.
    pub count: u16,
    /// Count rule.
    pub count_rule: Option<String>,
}

/// Validated InitialBillDef metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialBillDef {
    /// Recipe.
    pub recipe: RecipeKind,
    /// Enabled rule.
    pub enabled_rule: String,
    /// Target rule.
    pub target_rule: String,
}

/// Validated BlueprintDef metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintDef {
    /// Identity.
    pub identity: IdentityDef,
    /// Placeable.
    pub placeable: bool,
    /// Menu order.
    pub menu_order: i32,
    /// Shortcut slot.
    pub shortcut_slot: i32,
    /// Footprint.
    pub footprint: (u32, u32),
    /// Footprint rule.
    pub footprint_rule: Option<String>,
    /// Needs.
    pub needs: Vec<BlueprintNeed>,
    /// Work.
    pub work: u32,
    /// Work rule.
    pub work_rule: Option<String>,
    /// Placement.
    pub placement: PlacementCategory,
    /// Completion.
    pub completion: CompletionCategory,
    /// Workshop tags.
    pub workshop_tags: Vec<String>,
    /// Requires economy.
    pub requires_economy: bool,
    /// Requires knowledge.
    pub requires_knowledge: bool,
    /// Requires space.
    pub requires_space: bool,
    /// Planned nav.
    pub planned_nav: PlannedNav,
    /// Material.
    pub material: CompletionMaterial,
    /// Style id.
    pub style_id: String,
    /// Legacy packed wall render category, when applicable.
    pub render_wall_code: Option<u8>,
    /// Completion policy.
    pub completion_policy: CompletionPolicy,
    /// Initial bill.
    pub initial_bill: Option<InitialBillDef>,
}
