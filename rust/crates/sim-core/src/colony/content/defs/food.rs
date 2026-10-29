//! Food capabilities and storage lifetime policy, interpreted without content-name branches.
use serde::Deserialize;

/// Physical storage condition of a food stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoodStorageClass {
    /// Unprotected loose food or food carried by a person.
    Ground,
    /// Food on a designated stockpile cell.
    Stockpile,
    /// Food inside an actual roofed storeroom.
    RoofedStore,
    /// Food contained in a finite-capacity physical vessel.
    Pot,
}

/// Authored lifetime in calendar days for each storage condition.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoodStorageDays {
    /// Ground lifetime.
    pub ground: u32,
    /// Stockpile lifetime.
    pub stockpile: u32,
    /// Roofed storeroom lifetime.
    pub roofed_store: u32,
    /// Pot lifetime.
    pub pot: u32,
}

/// Item food capability; legacy selectors remain available to baseline mechanics.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoodDef {
    /// Whole satiety units per item.
    pub units: u32,
    /// Legacy per-world unit selector.
    pub units_rule: Option<String>,
    /// Legacy lifetime when storage-specific data is absent.
    pub spoil_days: u32,
    /// Legacy per-world lifetime selector.
    pub spoil_rule: Option<String>,
    /// Ingredient nutritional groups, independent of preparation category.
    #[serde(default)]
    pub groups: Vec<String>,
    /// Whether this item may be eaten without further processing.
    #[serde(default = "edible_default")]
    pub raw_edible: bool,
    /// Whether consumption counts as a cooked meal.
    #[serde(default)]
    pub cooked: bool,
    /// Authored food-choice preference.
    #[serde(default)]
    pub preference: i32,
    /// Storage-specific lifetimes; absence preserves legacy content loading.
    #[serde(default)]
    pub storage_spoil_days: Option<FoodStorageDays>,
    /// Keyed sickness probability per thousand completed raw meals.
    #[serde(default)]
    pub sick_permille: u32,
}
fn edible_default() -> bool {
    true
}
impl FoodDef {
    /// Lifetime for the physical storage class; moving never resets food age.
    pub fn lifetime_days(&self, class: FoodStorageClass) -> u32 {
        let Some(days) = &self.storage_spoil_days else {
            return self.spoil_days;
        };
        match class {
            FoodStorageClass::Ground => days.ground,
            FoodStorageClass::Stockpile => days.stockpile,
            FoodStorageClass::RoofedStore => days.roofed_store,
            FoodStorageClass::Pot => days.pot,
        }
    }
}
