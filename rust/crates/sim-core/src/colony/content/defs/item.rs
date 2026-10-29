//! Item capabilities; values select existing integer mechanics.
use super::super::ids::ItemKind;
use super::{FoodDef, IdentityDef};
use serde::Deserialize;

/// Validated ItemDef metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemDef {
    /// Identity.
    pub identity: IdentityDef,
    /// Category.
    pub category: String,
    /// Existing material-dependent localized label, or empty.
    pub material_format_key: String,
    /// Tags.
    pub tags: Vec<String>,
    /// Stack max.
    pub stack_max: u16,
    /// Carry.
    pub carry: u16,
    /// Stack rule.
    pub stack_rule: Option<String>,
    /// Carry rule.
    pub carry_rule: Option<String>,
    /// Value.
    pub value: u32,
    /// Made by.
    pub made_by: Option<String>,
    /// Clue nodes.
    pub clue_nodes: Vec<String>,
    /// Chop.
    pub chop: Option<ChopDef>,
    /// Weapon.
    pub weapon: Option<WeaponDef>,
    /// Cutter.
    pub cutter: Option<CutterDef>,
    /// Food.
    pub food: Option<FoodDef>,
    /// Number of food pieces held by one physical vessel.
    pub pot_capacity: u16,
    /// Fuel units.
    pub fuel_units: Option<u32>,
    /// Apparel.
    pub apparel: Option<ApparelDef>,
}

/// Validated ChopDef metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChopDef {
    /// Quality permille.
    pub quality_permille: [u32; 11],
    /// Wear rule.
    pub wear_rule: Option<String>,
    /// Wear default.
    pub wear_default: u32,
    /// Break return.
    pub break_return: Option<(ItemKind, u16)>,
    /// Preserve return material.
    pub preserve_return_material: bool,
}

/// Validated WeaponDef metadata.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponDef {
    /// Damage.
    pub damage: u16,
    /// Damage rule.
    pub damage_rule: Option<String>,
    /// Wear rule.
    pub wear_rule: Option<String>,
    /// Wear default.
    pub wear_default: u32,
    /// Draft rank.
    pub draft_rank: Option<u16>,
    /// Hunt rank.
    pub hunt_rank: Option<u16>,
    /// Hunt hit bonus.
    pub hunt_hit_bonus: bool,
    /// Break tally.
    pub break_tally: Option<String>,
}

/// Validated CutterDef metadata.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CutterDef {
    /// Wear rule.
    pub wear_rule: Option<String>,
    /// Wear default.
    pub wear_default: u32,
    /// Break tally.
    pub break_tally: Option<String>,
}

/// Validated ApparelDef metadata.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApparelDef {
    /// Legacy top code.
    pub legacy_top_code: u8,
    /// Warmth rank.
    pub warmth_rank: u16,
    /// Cold permille.
    pub cold_permille: u32,
    /// Cold snap work immune.
    pub cold_snap_work_immune: bool,
}
