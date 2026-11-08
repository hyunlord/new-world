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
    /// Optional domestication policy; absent species remain purely wild.
    #[serde(default)]
    pub tameable: Option<TameableDef<I>>,
    /// Data-selected placeholder geometry and tint, with a category fallback.
    #[serde(default = "animal_art_default")]
    pub art: super::ItemArt,
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

fn animal_art_default() -> super::ItemArt {
    super::ItemArt::for_category("animal")
}

/// Data-only policy interpreted by the capture jobs and single livestock system.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TameableDef<I = ItemKind> {
    /// Connected knowledge needed by the capturing worker.
    pub capture_knowledge: String,
    /// One physical tool consumed only on a successful capture.
    pub capture_tool: Option<I>,
    /// Registered capture mechanic selector.
    pub capture_job: String,
    /// Maximum capture search distance in tiles.
    pub capture_range_tiles: u32,
    /// Successful or failed capture work duration.
    pub capture_work_ticks: u32,
    /// Chance per completed, persisted capture attempt.
    pub capture_success_permille: u16,
    /// Whole days before another completed attempt may be made.
    pub capture_retry_days: u32,
    /// Inclusive keyed biological age at wild appearance.
    pub initial_age_days: (u32, u32),
    /// First adult age; only younger wild animals may be captured.
    pub kid_age_days: u32,
    /// Minimum interval since capture or the mother's last committed birth.
    pub breeding_interval_days: u32,
    /// Natural lifespan in whole days.
    pub max_age_days: u32,
    /// Distinct forage cells consumed per head at each daily feeding.
    pub graze_tiles_per_head: u32,
    /// Consecutive failed feedings causing starvation.
    pub starvation_days: u32,
    /// Whole days between consuming and regenerating a forage cell.
    pub grass_regrow_days: u32,
    /// Work duration for a physical milk yield.
    pub milk_work_ticks: u32,
    /// Work duration for an explicitly ordered slaughter.
    pub slaughter_work_ticks: u32,
    /// Generic catalogue-referenced products.
    pub products: LivestockProducts<I>,
}

/// Outputs of generic livestock mechanics, independent of species names.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LivestockProducts<I = ItemKind> {
    /// One milk item per committed recurring yield.
    pub milk: Option<RecurringProduct<I>>,
    /// Reserved recurring wool policy; absent until its processing pass.
    pub wool: Option<RecurringProduct<I>>,
    /// Food output of a slaughter.
    pub meat: (I, u16),
    /// Durable hide output of a slaughter.
    pub hide: (I, u16),
}

/// A single physical recurring output and its real knowledge requirement.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecurringProduct<I = ItemKind> {
    /// Catalogue item identity.
    pub item: I,
    /// Whole-day interval after a successful output.
    pub every_days: u32,
    /// Connected practitioner knowledge required at commit.
    pub requires_knowledge: String,
}
