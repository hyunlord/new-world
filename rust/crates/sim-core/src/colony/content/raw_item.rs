//! Raw item data; never deserializes installed content IDs.
use super::defs::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ItemRaw {
    pub identity: IdentityDef,
    pub category: String,
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
    pub food: Option<FoodDef>,
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
