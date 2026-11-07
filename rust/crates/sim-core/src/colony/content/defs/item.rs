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
    /// Data-authored shape and material tint for code-drawn placeholders.
    pub art: ItemArt,
    /// Whether authored behaviour is wired into the game.
    pub status: crate::colony::knowledge::ContentStatus,
    /// Missing behaviour and planned pass for inert content.
    pub debt: Option<String>,
    /// Era metadata for the generated content-debt ledger.
    pub era: String,
    /// Lane metadata for the generated content-debt ledger.
    pub lane: String,
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
    /// Reusable workshop capabilities, independent of cutting and knife wear.
    pub tool_capabilities: Vec<String>,
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
    /// Worn simulation days before expiry; absent preserves legacy durability.
    #[serde(default)]
    pub wear_days: Option<u32>,
    /// Additive mood target contribution while worn; legacy clothing is inert.
    #[serde(default)]
    pub comfort_permille: i32,
}

/// Eight renderer shapes shared by all data-authored item kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ItemShape {
    /// An irregular resource lump.
    Lump,
    /// A long narrow rod.
    Rod,
    /// A pointed blade.
    Blade,
    /// An open container.
    Vessel,
    /// A folded sheet or fibre.
    Cloth,
    /// A cluster of small grains.
    Grain,
    /// A mineral cluster.
    Ore,
    /// A rectangular metal bar.
    Ingot,
}

impl ItemShape {
    /// Renderer-facing stable selector, independent of item names.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Lump => "lump",
            Self::Rod => "rod",
            Self::Blade => "blade",
            Self::Vessel => "vessel",
            Self::Cloth => "cloth",
            Self::Grain => "grain",
            Self::Ore => "ore",
            Self::Ingot => "ingot",
        }
    }
}

/// Shape and material identity for asset-free item drawing.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemArt {
    /// Geometry selected from the eight supported category shapes.
    pub shape: ItemShape,
    /// Material registry id used to resolve a display colour.
    pub tint_material: String,
}

impl ItemArt {
    /// Category fallback keeps unknown data-authored kinds visible.
    pub fn for_category(category: &str) -> Self {
        let shape = match category {
            "tool" | "weapon" => ItemShape::Blade,
            "apparel" | "textile" => ItemShape::Cloth,
            "food" | "seed" => ItemShape::Grain,
            "vessel" => ItemShape::Vessel,
            "ore" => ItemShape::Ore,
            "ingot" => ItemShape::Ingot,
            _ => ItemShape::Lump,
        };
        Self {
            shape,
            tint_material: "oak".into(),
        }
    }
}
