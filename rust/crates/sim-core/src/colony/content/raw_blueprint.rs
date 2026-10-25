//! Raw blueprint data; never deserializes installed content IDs.
use super::defs::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BlueprintRaw {
    pub identity: IdentityDef,
    pub placeable: bool,
    pub menu_order: i32,
    pub shortcut_slot: i32,
    pub footprint: (u32, u32),
    pub footprint_rule: Option<String>,
    pub needs: Vec<BlueprintNeedRaw>,
    pub work: u32,
    pub work_rule: Option<String>,
    pub placement: PlacementCategory,
    pub completion: CompletionCategory,
    pub workshop_tags: Vec<String>,
    pub requires_economy: bool,
    pub requires_knowledge: bool,
    pub requires_space: bool,
    pub planned_nav: PlannedNav,
    pub material: CompletionMaterial,
    pub style_id: String,
    /// Legacy packed wall render category, when applicable.
    pub render_wall_code: Option<u8>,
    pub completion_policy: CompletionPolicy,
    pub initial_bill: Option<InitialBillRaw>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BlueprintNeedRaw {
    pub item: String,
    pub count: u16,
    pub count_rule: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct InitialBillRaw {
    pub recipe: String,
    pub enabled_rule: String,
    pub target_rule: String,
}
