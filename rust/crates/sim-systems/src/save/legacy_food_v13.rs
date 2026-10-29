//! Exact format13 wire layout; only the new food evidence starts empty.
use super::*;
use sim_core::colony::food::*;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize)]
pub(super) struct LegacyBodyV13 {
    pub(super) current_tick: u64,
    pub(super) resources: SimResources,
    pub(super) world: world::WorldSave,
    pub(super) systems: Vec<(String, Vec<u8>)>,
    pub(super) steward: sim_core::colony::steward::StewardState,
    pub(super) content_state: sim_core::colony::content_state::ContentState,
    pub(super) mind: sim_core::colony::mind_state::MindState,
    pub(super) food: LegacyFoodStateV13,
}

#[derive(Serialize, Deserialize)]
pub(super) struct LegacyFoodStateV13 {
    /// Exact admitted food owner retained until its action ends or becomes invalid.
    pub(super) active_targets: BTreeMap<sim_core::components::AgentId, FoodOwner>,
    /// Tile-indexed plant state.
    pub(super) plants: BTreeMap<FoodTile, PlantState>,
    /// Exactly one age for every physical food owner.
    pub(super) ages: BTreeMap<FoodOwner, FoodAge>,
    /// Real deployed pot vessels and their contents.
    pub(super) pots: BTreeMap<u32, PotStore>,
    /// Next pot ID; allocation starts at one.
    pub(super) next_pot_id: u32,
    /// Bounded per-person meal and group histories.
    pub(super) meals: BTreeMap<u64, MealHistory>,
    /// Day last processed by the daily system.
    pub(super) last_daily_day: Option<u32>,
    /// Conserved food-flow evidence.
    pub(super) tallies: FoodTallies,
    /// Once-only narrative evidence.
    pub(super) firsts: FoodFirsts,
    /// Mutation generation for pure derived readers.
    pub(super) version: u64,
    /// Passive preservation batches; outputs are not yet physical items.
    pub(super) pending_drying: Vec<DryingBatch>,
    /// Plan observations keyed by their actual chronicle entry index.
    pub(super) winter_plans: BTreeMap<u32, WinterPlanEvidence>,
}

impl From<LegacyFoodStateV13> for FoodState {
    fn from(old: LegacyFoodStateV13) -> Self {
        Self {
            active_targets: old.active_targets,
            plants: old.plants,
            ages: old.ages,
            pots: old.pots,
            next_pot_id: old.next_pot_id,
            meals: old.meals,
            last_daily_day: old.last_daily_day,
            tallies: old.tallies,
            firsts: old.firsts,
            version: old.version,
            pending_drying: old.pending_drying,
            winter_plans: old.winter_plans,
            evidence: Default::default(),
        }
    }
}

impl From<LegacyBodyV13> for Body {
    fn from(old: LegacyBodyV13) -> Self {
        Self {
            current_tick: old.current_tick,
            resources: old.resources,
            world: old.world,
            systems: old.systems,
            steward: old.steward,
            content_state: old.content_state,
            mind: old.mind,
            food: old.food.into(),
        }
    }
}
