//! Raw item data; never deserializes installed content IDs.
use super::defs::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ItemRaw {
    pub identity: IdentityDef,
    pub category: String,
    #[serde(default, deserialize_with = "deserialize_art")]
    pub art: Option<ItemArt>,
    #[serde(default)]
    pub status: crate::colony::knowledge::ContentStatus,
    #[serde(default)]
    pub debt: Option<String>,
    #[serde(default)]
    pub era: String,
    #[serde(default)]
    pub lane: String,
    pub material_format_key: String,
    pub tags: Vec<String>,
    pub stack_max: u16,
    pub carry: u16,
    pub stack_rule: Option<String>,
    pub carry_rule: Option<String>,
    pub value: u32,
    pub made_by: Option<String>,
    pub clue_nodes: Vec<String>,
    pub chop: Option<ChopRaw>,
    pub weapon: Option<WeaponDef>,
    pub cutter: Option<CutterDef>,
    #[serde(default)]
    pub tool_capabilities: Vec<String>,
    pub food: Option<FoodDef>,
    #[serde(default)]
    pub pot_capacity: u16,
    pub fuel_units: Option<u32>,
    pub apparel: Option<ApparelDef>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ChopRaw {
    pub quality_permille: [u32; 11],
    pub wear_rule: Option<String>,
    pub wear_default: u32,
    pub break_return: Option<(String, u16)>,
    pub preserve_return_material: bool,
}

// Optional authored metadata is omitted for category defaults, or a plain art record when present.
fn deserialize_art<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<ItemArt>, D::Error> {
    ItemArt::deserialize(deserializer).map(Some)
}
