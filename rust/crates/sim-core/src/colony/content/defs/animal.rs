//! Species definitions sharing explicit grazer and predator mechanisms.
use super::{IdentityDef, RuleBool, RuleU16, RuleU32};
use crate::colony::content::ItemKind;
/// Existing animal behavior categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum AnimalMechanic {
    /// Wander, graze, flee and rest.
    Grazer,
    /// Hunt, bite and retreat as a pack.
    Predator,
}
/// Validated AnimalDef metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimalDef<I = ItemKind> {
    /// Identity.
    pub identity: IdentityDef,
    /// Category.
    pub category: String,
    /// Mechanic.
    pub mechanic: AnimalMechanic,
    /// Huntable.
    pub huntable: bool,
    /// Hp.
    pub hp: RuleU16,
    /// Grazer.
    pub grazer: Option<GrazerDef>,
    /// Predator.
    pub predator: Option<PredatorDef>,
    /// Loot.
    pub loot: AnimalLoot<I>,
}
/// Validated GrazerDef metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrazerDef {
    /// Flee.
    pub flee: RuleBool,
    /// Flee tiles.
    pub flee_tiles: RuleU32,
    /// Flee speed.
    pub flee_speed: RuleU32,
    /// Flee steps.
    pub flee_steps: RuleU32,
    /// Rest steps.
    pub rest_steps: RuleU32,
    /// Home tiles.
    pub home_tiles: RuleU32,
    /// Min hearth distance.
    pub min_hearth_distance: RuleU32,
}
/// Validated PredatorDef metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredatorDef {
    /// Sight.
    pub sight: RuleU32,
    /// Damage.
    pub damage: RuleU16,
    /// Chase tiles.
    pub chase_tiles: RuleU32,
    /// Hit ticks.
    pub hit_ticks: RuleU32,
}
/// Validated AnimalLoot metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimalLoot<I = ItemKind> {
    /// Meat.
    pub meat: Option<MeatLoot<I>>,
    /// Hide.
    pub hide: Option<HideLoot<I>>,
}
/// Validated MeatLoot metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeatLoot<I = ItemKind> {
    /// Item.
    pub item: I,
    /// With cutter.
    pub with_cutter: RuleU16,
    /// Without cutter.
    pub without_cutter: RuleU16,
}
/// Validated HideLoot metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HideLoot<I = ItemKind> {
    /// Item.
    pub item: I,
    /// Count.
    pub count: u16,
    /// Enabled.
    pub enabled: RuleBool,
}
