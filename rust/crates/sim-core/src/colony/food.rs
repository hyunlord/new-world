//! Persistent colony food ownership, plant growth and bounded meal evidence.
use super::content::PlantKind;
use super::{quality_bucket, ItemKind, ItemSpec, ItemStack};
use crate::material::MaterialId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A tile in the simulation map.
pub type FoodTile = (u32, u32);

/// Stable merge identity independent of a workshop shelf's vector position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FoodStackKey {
    /// Registry item identity.
    pub kind: ItemKind,
    /// Material identity, when applicable.
    pub material: Option<MaterialId>,
    /// Existing ten-point quality bucket.
    pub quality_bucket: u8,
}
impl From<ItemSpec> for FoodStackKey {
    fn from(spec: ItemSpec) -> Self {
        Self {
            kind: spec.kind,
            material: spec.material,
            quality_bucket: quality_bucket(spec.quality),
        }
    }
}

/// The one physical owner of a food stack's age metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FoodOwner {
    /// Stack on a map tile.
    Ground(FoodTile),
    /// Stack in an agent's existing Carry component, keyed by AgentId.
    Carried(u64),
    /// Stack on an existing workshop shelf.
    Shelf {
        /// Workshop identity.
        workshop: u32,
        /// Stable stack identity.
        key: FoodStackKey,
    },
    /// Stack contained in a deployed real pot.
    Pot {
        /// Deployed pot identity.
        id: u32,
        /// Stable stack identity.
        key: FoodStackKey,
    },
}

/// Birth date follows transfers; merging retains the oldest date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoodAge {
    /// Zero-based simulation day of production.
    pub born_day: u32,
}
impl FoodAge {
    /// Conservative date for two compatible stacks.
    pub const fn merged(self, other: Self) -> Self {
        Self {
            born_day: if self.born_day < other.born_day {
                self.born_day
            } else {
                other.born_day
            },
        }
    }
    /// Remaining complete days under the current physical storage class.
    pub const fn remaining(self, day: u32, lifetime: u32) -> u32 {
        lifetime.saturating_sub(day.saturating_sub(self.born_day))
    }
}

/// Per-tile harvest state; calendar phases are resolved from species data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlantState {
    /// Stable species identity.
    pub species: PlantKind,
    /// Items available from this ripe harvest opportunity.
    pub units_left: u16,
    /// Earliest day a depleted plant can regrow.
    pub next_regrow_day: u32,
}

/// A stationary container with one real vessel and conserved contents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PotStore {
    /// Tile at which this vessel is deployed.
    pub tile: FoodTile,
    /// One physical vessel removed from ground/carried ownership.
    pub vessel: ItemSpec,
    /// Contents; each merge key occurs at most once.
    pub contents: Vec<ItemStack>,
}

/// One completed item consumption, never a planned or started meal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MealRecord {
    /// Completion tick.
    pub tick: u64,
    /// Consumed item identity.
    pub item: ItemKind,
    /// Actual whole pieces consumed.
    pub pieces: u16,
    /// Nominal food units consumed.
    pub units: u32,
    /// Whether preparation is cooked according to item data.
    pub cooked: bool,
    /// Actual physical owner from which this piece was consumed.
    pub source_owner: FoodOwner,
    /// Original stack birth date, preserved across storage transfers.
    pub born_day: u32,
    /// Lifetime of the source's actual storage class at consumption.
    pub storage_lifetime_days: u32,
    /// Actual retained choice matching the person's running mind node, if any.
    pub source_choice: Option<crate::causal::EventId>,
}

/// Display meals and independent group timestamps, bounded by registered groups.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MealHistory {
    /// At most eight most recent successful meals, oldest first.
    pub last_meals: Vec<MealRecord>,
    /// Latest successful consumption tick for each nutritional group.
    pub last_group_tick: BTreeMap<String, u64>,
}
impl MealHistory {
    /// Append a successful meal while retaining at most eight display records.
    pub fn record(&mut self, meal: MealRecord, groups: &[String]) {
        for group in groups {
            self.last_group_tick.insert(group.clone(), meal.tick);
        }
        self.last_meals.push(meal);
        if self.last_meals.len() > 8 {
            self.last_meals.drain(..self.last_meals.len() - 8);
        }
    }
    /// Count actual nutritional groups in a half-open trailing interval.
    pub fn groups_in(&self, now: u64, window: u64) -> usize {
        self.last_group_tick
            .values()
            .filter(|tick| **tick <= now && now - **tick < window)
            .count()
    }
    /// Drop expired group timestamps; the display tail is independent.
    pub fn prune(&mut self, now: u64, window: u64) {
        self.last_group_tick
            .retain(|_, tick| *tick <= now && now - *tick < window);
    }
}

/// Food accounting distinguishes raw harvest from processing and actual meals.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoodTallies {
    /// Harvested pieces by item, counted once at creation.
    pub harvested: BTreeMap<ItemKind, u64>,
    /// Eaten pieces by item.
    pub eaten: BTreeMap<ItemKind, u64>,
    /// Spoiled pieces by item.
    pub spoiled: BTreeMap<ItemKind, u64>,
    /// Pieces lost when a dead carrier has no legal drop destination.
    pub lost: BTreeMap<ItemKind, u64>,
    /// Pieces explicitly converted from an old save's abstract reserve.
    pub migrated: BTreeMap<ItemKind, u64>,
    /// Nominal satiety units at opening/migration.
    pub initial_units: u64,
    /// Nominal raw harvested units, never processing outputs.
    pub harvested_units: u64,
    /// Nominal food units eaten.
    pub eaten_units: u64,
    /// Nominal food units spoiled.
    pub spoiled_units: u64,
    /// Explicit physical losses, distinct from expiry or meals.
    pub lost_units: u64,
    /// Input units consumed by recipes.
    pub craft_input_units: u64,
    /// Output units created by recipes.
    pub craft_output_units: u64,
    /// Units entering through trade.
    pub traded_in_units: u64,
    /// Units leaving through trade.
    pub traded_out_units: u64,
    /// Completed whole-item meal events.
    pub meals: u64,
    /// Completed cooked whole-item meal events.
    pub cooked_meals: u64,
    /// Actual hunger relief in tenths, separate from nominal units.
    pub relief_tenths: u64,
    /// Nominal satiety wasted to hunger clamping in tenths.
    pub wasted_tenths: u64,
}

/// Once-only events retain their producing causal IDs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoodFirsts {
    /// Last calendar day on which a winter preparation plan was chronicled.
    pub last_plan_day: Option<u32>,
    /// Years whose winter began with the physically required edible reserve.
    pub winter_started_with_stores: BTreeSet<u32>,
    /// First successful cooked meal cause.
    pub cooked_meal: Option<u64>,
    /// First dried storage cause.
    pub dried_store: Option<u64>,
    /// Completed winters already chronicled, by year index.
    pub survived_winters: BTreeSet<u32>,
}

/// Explicit format13 sidecar; empty and inert in baseline worlds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DryingBatch {
    /// Workshop responsible for passive preservation.
    pub workshop: u32,
    /// Bill that supplied the input.
    pub bill: u32,
    /// Output drop location.
    pub tile: FoodTile,
    /// Authored result of the completed work.
    pub output: ItemSpec,
    /// Whole output pieces.
    pub count: u16,
    /// Earliest simulation tick at which drying is complete.
    pub due_tick: u64,
}

/// Historical denominator and quantities for one actual winter-plan chronicle entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WinterPlanEvidence {
    /// Tick of the linked WinterFoodPlan entry.
    pub tick: u64,
    /// Actual all-member daily demand in thousandths at that observation.
    pub daily_demand_milli: u64,
    /// Fresh reachable edible stored units before display clamping.
    pub stored_units: u64,
    /// Frozen winter requirement before display clamping.
    pub required_units: u64,
}

/// Explicit format13 sidecar; empty and inert in baseline worlds.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoodState {
    /// Exact admitted food owner retained until its action ends or becomes invalid.
    pub active_targets: BTreeMap<crate::components::AgentId, FoodOwner>,
    /// Tile-indexed plant state.
    pub plants: BTreeMap<FoodTile, PlantState>,
    /// Exactly one age for every physical food owner.
    pub ages: BTreeMap<FoodOwner, FoodAge>,
    /// Real deployed pot vessels and their contents.
    pub pots: BTreeMap<u32, PotStore>,
    /// Next pot ID; allocation starts at one.
    pub next_pot_id: u32,
    /// Bounded per-person meal and group histories.
    pub meals: BTreeMap<u64, MealHistory>,
    /// Day last processed by the daily system.
    pub last_daily_day: Option<u32>,
    /// Conserved food-flow evidence.
    pub tallies: FoodTallies,
    /// Once-only narrative evidence.
    pub firsts: FoodFirsts,
    /// Mutation generation for pure derived readers.
    pub version: u64,
    /// Passive preservation batches; outputs are not yet physical items.
    pub pending_drying: Vec<DryingBatch>,
    /// Plan observations keyed by their actual chronicle entry index.
    pub winter_plans: BTreeMap<u32, WinterPlanEvidence>,
}
impl FoodState {
    /// Notify derived food readers of a physical or ownership change.
    pub fn touch(&mut self) {
        self.version = self.version.wrapping_add(1);
    }
}
