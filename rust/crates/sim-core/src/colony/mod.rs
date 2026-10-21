//! The M1 colony data: physical items, trees, orders (designations, zones,
//! blueprints), recorded commands and jobs. Data only — commands live in
//! `sim_systems::commands`, job logic in `sim_systems::runtime::jobs`.
//!
//! Everything here is empty in the baseline world: only `WorldRules::colony`
//! (`rules.colony = Some(..)`) and its worldgen ever fill it (ADR 0014).
//! Every collection is a `BTreeMap`/`BTreeSet` or a `Vec` in a documented
//! order, so iteration is deterministic.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod command;
mod items;
mod job;
mod orders;

pub use command::{Command, CommandRecord, CommandResult, RejectReason};
pub use items::{Carry, ItemKind, ItemLedger, ItemStack};
pub use job::{CancelReason, Job, JobKind, JobStage, WorkType};
pub use orders::{
    BlueprintKind, ColonySite, Designation, MaterialNeeds, Occupant, ReserveKey, Tree, Zone,
    ZoneId, ZoneKind,
};

use crate::components::AgentId;

/// Colony counters (M1); measurement and UI only, never read by a decision.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColonyTallies {
    /// Jobs taken.
    pub jobs_taken: u64,
    /// Jobs finished.
    pub jobs_done: u64,
    /// Jobs cancelled, by reason.
    pub cancels: BTreeMap<CancelReason, u64>,
    /// Trees felled.
    pub trees_felled: u64,
    /// Huts finished.
    pub huts_built: u64,
    /// Commands rejected (all reasons, drafting included).
    pub commands_rejected: u64,
    /// Founders the colony worldgen made adults (goal 8.1; 0 when none was needed).
    pub founder_fixes: u32,
}

impl ColonyTallies {
    /// Cancels for `reason` so far.
    pub fn cancels_of(&self, reason: CancelReason) -> u64 {
        self.cancels.get(&reason).copied().unwrap_or(0)
    }
}

/// All colony state kept beside the ECS world (`SimResources::colony`).
#[derive(Debug, Clone, Default)]
pub struct ColonyState {
    /// Standing trees by tile.
    pub trees: BTreeMap<(u32, u32), Tree>,
    /// Bumped whenever a tree is added or removed (renderers rebuild on change).
    pub trees_version: u64,
    /// Ground stacks, one per tile.
    pub ground_items: BTreeMap<(u32, u32), ItemStack>,
    /// Zones by id.
    pub zones: BTreeMap<ZoneId, Zone>,
    /// The zone each zoned tile belongs to.
    pub zone_of_tile: BTreeMap<(u32, u32), ZoneId>,
    /// Next zone id to hand out (starts at 1).
    pub next_zone_id: ZoneId,
    /// Player marks by tile.
    pub designations: BTreeMap<(u32, u32), Designation>,
    /// Footprint tiles of blueprint sites and finished huts.
    pub occupied: BTreeMap<(u32, u32), Occupant>,
    /// Who holds each reserved thing.
    pub reservations: BTreeMap<ReserveKey, AgentId>,
    /// Every command applied, in order.
    pub command_log: Vec<CommandRecord>,
    /// Items created and consumed.
    pub item_ledger: ItemLedger,
    /// Bumped by any change to zones, designations, ground items, sites or trees.
    pub overlay_version: u64,
    /// Counters.
    pub tallies: ColonyTallies,
    /// Measurement only (M1.5 goal 8.2.1): jobs the decision seam released to
    /// a need, counted by [`JobReleaseKey`]. No system reads it, and no
    /// fingerprint or summary includes it.
    pub release_counts: BTreeMap<JobReleaseKey, u64>,
}

/// Measurement-only key of a job released to a need (`ColonyState::release_counts`):
/// `(need, job kind, job stage, need value bin)`. Need: 0 hunger (the choice
/// targets food), 1 thirst (water), 2 sleep, 3 social (another agent),
/// 4 forage, 5 firewood, 6 other. Kind: 0 chop, 1 haul, 2 deliver, 3 build.
/// Stage: 0 walking to the source, 1 working, 2 walking to the destination.
/// Bin: the chosen need's value (hunger, thirst or fatigue) in tens, 0–10;
/// 255 for a need without a value.
pub type JobReleaseKey = (u8, u8, u8, u8);

impl ColonyState {
    /// Mark the overlay (and, with `trees`, the tree layer) as changed.
    pub fn touch(&mut self, trees: bool) {
        self.overlay_version += 1;
        if trees {
            self.trees_version += 1;
        }
    }

    /// Logs (or `kind`) lying on the ground in total.
    pub fn ground_total(&self, kind: ItemKind) -> u64 {
        self.ground_items
            .values()
            .filter(|stack| stack.kind == kind)
            .map(|stack| u64::from(stack.count))
            .sum()
    }
}
