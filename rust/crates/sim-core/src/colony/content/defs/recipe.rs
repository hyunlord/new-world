//! Recipe identity, compatibility and existing quality mechanics.
use super::super::ids::{BlueprintKind, ItemKind, RecipeKind};
use super::IdentityDef;
use crate::colony::{SkillKind, WorkType};
use serde::Deserialize;

/// Existing integer material adjustment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MaterialTerm {
    /// No material quality adjustment.
    None,
    /// Truncating integer (hardness-offset)/divisor.
    Hardness {
        /// Hardness zero point.
        offset: i32,
        /// Positive divisor.
        divisor: i32,
    },
}

/// Validated RecipeToolDef metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeToolDef {
    /// Capability.
    pub capability: String,
    /// Preferred item.
    pub preferred_item: ItemKind,
    /// Count.
    pub count: u16,
    /// Reusable.
    pub reusable: bool,
}

/// Validated RecipeDef metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeDef {
    /// Identity.
    pub identity: IdentityDef,
    /// Id.
    pub id: RecipeKind,
    /// Workshop.
    pub workshop: BlueprintKind,
    /// Compatible workshops.
    pub compatible_workshops: Vec<BlueprintKind>,
    /// Workshop tags.
    pub workshop_tags: Vec<String>,
    /// Inputs.
    pub inputs: Vec<(ItemKind, u16)>,
    /// Output.
    pub output: (ItemKind, u16),
    /// Work ticks.
    pub work_ticks: u32,
    /// Skill.
    pub skill: SkillKind,
    /// Work.
    pub work: WorkType,
    /// Quality add.
    pub quality_add: i32,
    /// Material term.
    pub material_term: MaterialTerm,
    /// Tool.
    pub tool: Option<RecipeToolDef>,
    /// Auto bill role.
    pub auto_bill_role: Option<String>,
}
