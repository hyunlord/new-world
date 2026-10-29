//! Authored food demand and preservation policy for ordinary steward commands.
use crate::colony::RecipeKind;
use serde::{Deserialize, Serialize};

/// All planning targets are data, independent of content display names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoodPlan {
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
        self.adult_daily_milli > 0
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
