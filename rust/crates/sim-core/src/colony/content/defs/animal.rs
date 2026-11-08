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
    /// Whether natural population behavior is enabled.
    #[serde(default)]
    pub status: crate::colony::knowledge::ContentStatus,
    /// Disabled behavior and owning follow-up pass.
    #[serde(default)]
    pub debt: Option<String>,
    /// Category.
    pub category: String,
    /// Climate and terrain in which the wild species originates.
    #[serde(default)]
    pub origin: super::SpeciesOrigin,
    /// Optional independent wild population; absent retains legacy role seeding.
    #[serde(default)]
    pub population: Option<WildPopulation>,
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
/// Data-only wild population policy interpreted by the existing animal system.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WildPopulation {
    /// Inclusive initial population range; also the stable per-world cap.
    pub initial: (u32, u32),
    /// Days between one-head replenishments below that cap.
    pub respawn_days: u32,
    /// Radius around the selected herd centre for initial placement and births.
    pub herd_tiles: u32,
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
