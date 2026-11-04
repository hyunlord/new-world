//! Authored food demand and preservation policy for ordinary steward commands.
use crate::colony::{content::CropKind, RecipeKind};
use serde::{Deserialize, Serialize};

/// Replacement planting remains independent of transient wild-food fullness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CultivationPlan {
    /// Registered crop selected for automatic replacement planting.
    pub crop: CropKind,
    /// Minimum annual food demand share planned as cultivated output.
    pub replacement_permille: u32,
    /// Calibrated edible output after retained seed and losses.
    pub net_units_per_tile: u32,
    /// Hard accepted-area ceiling relative to the replacement target.
    pub area_cap_permille: u32,
    /// Maximum new field tiles ordered in one planning pass.
    pub batch: u32,
}
/// All planning targets are data, independent of content display names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoodPlan {
    /// Optional automatic field replacement policy.
    #[serde(default)]
    pub cultivation: Option<CultivationPlan>,
    /// Adult satiety units per day, in thousandths.
    pub adult_daily_milli: u32,
    /// Child satiety units per day, in thousandths.
    pub child_daily_milli: u32,
    /// Demand multiplier for a currently cold person, in thousandths.
    pub cold_multiplier: u32,
    /// Ordinary immediate supply horizon.
    pub reserve_days: u32,
    /// Zero-based year day when winter reserves begin accumulating.
    pub reserve_start_day: u32,
    /// Winter season index.
    pub winter_season: u32,
    /// Maximum harvest distance from the settlement hearth.
    pub forage_radius: u32,
    /// Maximum newly designated plants per firing.
    pub harvest_batch: u32,
    /// Maximum cooked demand horizon, in thousandths of a day.
    pub cooked_days_milli: u32,
    /// Ordered recipes used for demand-limited cooking and preservation.
    pub recipes: Vec<RecipeKind>,
    /// Prerequisite vessel recipes and physical maintained counts.
    pub vessel_recipes: Vec<(RecipeKind, u16)>,
    /// Stockpile storage lifetime required past winter's start for reserve harvest.
    pub winter_keep_days: u32,
}
impl FoodPlan {
    pub(super) fn valid(&self) -> bool {
        self.cultivation.as_ref().is_none_or(|plan| {
            plan.replacement_permille > 0
                && plan.replacement_permille <= 1000
                && plan.net_units_per_tile > 0
                && plan.area_cap_permille >= 1000
                && plan.batch > 0
                && crate::colony::content::installed_or_bundled()
                    .crop(plan.crop)
                    .is_some()
        }) && self.adult_daily_milli > 0
            && self.child_daily_milli > 0
            && self.cold_multiplier >= 1000
            && self.reserve_days > 0
            && self.winter_season < 4
            && self.forage_radius > 0
            && self.harvest_batch > 0
            && self.cooked_days_milli > 0
            && !self.recipes.is_empty()
            && self.recipes.iter().all(|recipe| recipe.recipe().is_some())
            && self
                .vessel_recipes
                .iter()
                .all(|(recipe, count)| *count > 0 && recipe.recipe().is_some())
    }
}
