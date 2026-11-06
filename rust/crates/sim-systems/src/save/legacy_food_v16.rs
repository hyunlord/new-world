//! Frozen format14–16 food ownership, including the old physical item leaves.
use serde::{Deserialize, Serialize};
use sim_core::colony::food::*;
use sim_core::colony::legacy_items_v16::{ItemSpecV16, ItemStackV16};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize)]
pub(super) struct PotStoreV16 {
    /// Tile at which this vessel is deployed.
    pub tile: FoodTile,
    /// One physical vessel removed from ground/carried ownership.
    pub vessel: ItemSpecV16,
    /// Contents; each merge key occurs at most once.
    pub contents: Vec<ItemStackV16>,
}
impl From<PotStoreV16> for PotStore {
    fn from(old: PotStoreV16) -> Self {
        Self {
            tile: old.tile,
            vessel: old.vessel.into(),
            contents: old.contents.into_iter().map(Into::into).collect(),
        }
    }
}
impl TryFrom<PotStore> for PotStoreV16 {
    type Error = &'static str;
    fn try_from(old: PotStore) -> Result<Self, Self::Error> {
        Ok(Self {
            tile: old.tile,
            vessel: old.vessel.try_into()?,
            contents: old
                .contents
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct DryingBatchV16 {
    /// Workshop responsible for passive preservation.
    pub workshop: u32,
    /// Bill that supplied the input.
    pub bill: u32,
    /// Output drop location.
    pub tile: FoodTile,
    /// Authored result of the completed work.
    pub output: ItemSpecV16,
    /// Whole output pieces.
    pub count: u16,
    /// Earliest simulation tick at which drying is complete.
    pub due_tick: u64,
}
impl From<DryingBatchV16> for DryingBatch {
    fn from(old: DryingBatchV16) -> Self {
        Self {
            workshop: old.workshop,
            bill: old.bill,
            tile: old.tile,
            output: old.output.into(),
            count: old.count,
            due_tick: old.due_tick,
        }
    }
}
impl TryFrom<DryingBatch> for DryingBatchV16 {
    type Error = &'static str;
    fn try_from(old: DryingBatch) -> Result<Self, Self::Error> {
        Ok(Self {
            workshop: old.workshop,
            bill: old.bill,
            tile: old.tile,
            output: old.output.try_into()?,
            count: old.count,
            due_tick: old.due_tick,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct FoodStateV16 {
    /// Exact admitted food owner retained until its action ends or becomes invalid.
    pub active_targets: BTreeMap<sim_core::components::AgentId, FoodOwner>,
    /// Tile-indexed plant state.
    pub plants: BTreeMap<FoodTile, PlantState>,
    /// Exactly one age for every physical food owner.
    pub ages: BTreeMap<FoodOwner, FoodAge>,
    /// Real deployed pot vessels and their contents.
    pub pots: BTreeMap<u32, PotStoreV16>,
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
    pub pending_drying: Vec<DryingBatchV16>,
    /// Plan observations keyed by their actual chronicle entry index.
    pub winter_plans: BTreeMap<u32, WinterPlanEvidence>,
    /// Timestamped observations, empty in baseline and legacy migrations.
    pub evidence: sim_core::colony::food_evidence::FoodEvidence,
}
impl From<FoodStateV16> for FoodState {
    fn from(old: FoodStateV16) -> Self {
        Self {
            active_targets: old.active_targets,
            plants: old.plants,
            ages: old.ages,
            pots: old
                .pots
                .into_iter()
                .map(|(id, pot)| (id, pot.into()))
                .collect(),
            next_pot_id: old.next_pot_id,
            meals: old.meals,
            last_daily_day: old.last_daily_day,
            tallies: old.tallies,
            firsts: old.firsts,
            version: old.version,
            pending_drying: old.pending_drying.into_iter().map(Into::into).collect(),
            winter_plans: old.winter_plans,
            evidence: old.evidence,
        }
    }
}
impl TryFrom<FoodState> for FoodStateV16 {
    type Error = &'static str;
    fn try_from(old: FoodState) -> Result<Self, Self::Error> {
        Ok(Self {
            active_targets: old.active_targets,
            plants: old.plants,
            ages: old.ages,
            pots: old
                .pots
                .into_iter()
                .map(|(id, pot)| pot.try_into().map(|pot| (id, pot)))
                .collect::<Result<_, _>>()?,
            next_pot_id: old.next_pot_id,
            meals: old.meals,
            last_daily_day: old.last_daily_day,
            tallies: old.tallies,
            firsts: old.firsts,
            version: old.version,
            pending_drying: old
                .pending_drying
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            winter_plans: old.winter_plans,
            evidence: old.evidence,
        })
    }
}
