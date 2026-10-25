//! Raw recipe data; never deserializes installed content IDs.
use super::defs::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RecipeRaw {
    pub identity: IdentityDef,
    pub workshop: String,
    pub workshop_tags: Vec<String>,
    pub inputs: Vec<(String, u16)>,
    pub output: (String, u16),
    pub work_ticks: u32,
    pub skill: String,
    pub work: String,
    pub quality_add: i32,
    pub material_term: MaterialTerm,
    pub tool: Option<RecipeToolRaw>,
    pub auto_bill_role: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RecipeToolRaw {
    pub capability: String,
    pub preferred_item: String,
    pub count: u16,
    pub reusable: bool,
}
