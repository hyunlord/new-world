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
pub mod materials;
mod orders;
pub mod skills;
pub mod workshop;

pub use command::{Command, CommandRecord, CommandResult, RejectReason};
pub use items::{
    quality_bucket, Carry, ItemKind, ItemLedger, ItemSpec, ItemStack, Tool, RAW_QUALITY,
};
pub use job::{CancelReason, Job, JobKind, JobStage, WorkType};
pub use materials::{ColonyMaterials, MaterialStats};
pub use orders::{
    BlueprintKind, ColonySite, Designation, MaterialNeeds, NeedLine, NodeKind, Occupant,
    ReserveKey, ResourceNode, Tree, Zone, ZoneId, ZoneKind,
};
pub use skills::{Skill, Skills, WorkPriorities};
pub use workshop::{
    recipes, Bill, BillId, BillMode, Recipe, RecipeKind, SkillKind, Workshop, WorkshopId,
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
    /// Quarry batches finished (M2).
    #[serde(default)]
    pub quarry_batches: u64,
    /// Resource nodes quarried to exhaustion (M2).
    #[serde(default)]
    pub nodes_exhausted: u64,
    /// Workshops finished (M2).
    #[serde(default)]
    pub workshops_built: u64,
    /// Crafts finished (M2).
    #[serde(default)]
    pub crafts_done: u64,
    /// Crafts cancelled after their inputs were consumed (M2; the inputs are lost).
    #[serde(default)]
    pub crafts_lost: u64,
    /// Jobs taken, by `(agent, JobKind::code)` (M2; measurement and UI only).
    #[serde(default)]
    pub taken_by: BTreeMap<(AgentId, u8), u64>,
    /// Chop jobs finished with an axe in hand (M2).
    #[serde(default)]
    pub chops_with_axe: u64,
    /// Axes worn out (each became one stone; M2).
    #[serde(default)]
    pub axes_worn_out: u64,
    /// Crafted items' `(count, quality sum)` by `(agent, RecipeKind::code)` (M2; measurement only).
    #[serde(default)]
    pub craft_quality: BTreeMap<(AgentId, u8), (u64, u64)>,
    /// Space sites finished (walls, doors, floors, bedrolls; M3).
    #[serde(default)]
    pub space_built: u64,
    /// Built pieces taken apart (M3).
    #[serde(default)]
    pub space_removed: u64,
    /// Hill tiles dug out (M3).
    #[serde(default)]
    pub tiles_dug: u64,
    /// Hill tiles the colony worldgen opened so the camp lies on open ground
    /// (M3, DECISIONS D17; 0 when none was needed).
    #[serde(default)]
    pub camp_pass_tiles: u32,
}

impl ColonyTallies {
    /// Cancels for `reason` so far.
    pub fn cancels_of(&self, reason: CancelReason) -> u64 {
        self.cancels.get(&reason).copied().unwrap_or(0)
    }
}

/// All colony state kept beside the ECS world (`SimResources::colony`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColonyState {
    /// Standing trees by tile.
    pub trees: BTreeMap<(u32, u32), Tree>,
    /// Resource nodes (boulders, flint outcrops) by tile (M2).
    #[serde(default)]
    pub nodes: BTreeMap<(u32, u32), ResourceNode>,
    /// The colony's material table (M2; built at colony worldgen).
    #[serde(default)]
    pub materials: ColonyMaterials,
    /// Finished workshops by id (M2).
    #[serde(default)]
    pub workshops: BTreeMap<WorkshopId, Workshop>,
    /// Next workshop id (from 1).
    #[serde(default)]
    pub next_workshop_id: WorkshopId,
    /// Next bill id (from 1).
    #[serde(default)]
    pub next_bill_id: BillId,
    /// The future work tile of each workshop blueprint site, by site centre
    /// (M2 review m5: later blueprints may not cover it).
    #[serde(default)]
    pub site_work_tiles: BTreeMap<(u32, u32), (u32, u32)>,
    /// M3 (ADR 0022): the kind of every 1×1 space site (wall, door, floor,
    /// bedroll) by tile, while it is a blueprint; read by the nav map.
    #[serde(default)]
    pub space_sites: BTreeMap<(u32, u32), BlueprintKind>,
    /// M3 (ADR 0023): every finished bedroll by tile, with its material (a bed).
    #[serde(default)]
    pub beds: BTreeMap<(u32, u32), crate::material::MaterialId>,
    /// M3 (ADR 0023): the rooms by id (`RoomSystem`), each tile's room, the
    /// next fresh id, the player's role overrides, the rooms announced in the
    /// chronicle, and whether walls, doors or digging changed since the last
    /// detection.
    #[serde(default)]
    pub rooms: BTreeMap<crate::rooms::RoomId, crate::rooms::Room>,
    /// The room of each room tile.
    #[serde(default)]
    pub room_of_tile: BTreeMap<(u32, u32), crate::rooms::RoomId>,
    /// Next fresh room id (from 1).
    #[serde(default)]
    pub next_room_id: crate::rooms::RoomId,
    /// Roles the player set (`SetRoomRole`).
    #[serde(default)]
    pub room_overrides: BTreeMap<crate::rooms::RoomId, crate::rooms::RoomRole>,
    /// Rooms already logged as formed.
    #[serde(default)]
    pub rooms_announced: std::collections::BTreeSet<crate::rooms::RoomId>,
    /// A wall, door or dug tile changed since the last room detection.
    #[serde(default)]
    pub rooms_dirty: bool,
    /// Bumped whenever the rooms or their roles change (renderers).
    #[serde(default)]
    pub rooms_version: u64,
    /// M3 (ADR 0021): targets a person found unreachable, by `(person, tile)`,
    /// with the tick until which the job assignment skips them for that person.
    #[serde(default)]
    pub unreachable: BTreeMap<(AgentId, (u32, u32)), u64>,
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
/// 4 forage, 5 firewood, 6 other. Kind: `JobKind::code` − 1 (0 chop, 1 haul, 2 deliver, 3 build, 4 quarry, …).
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

    /// Items of `kind` lying on the ground in total.
    pub fn ground_total(&self, kind: ItemKind) -> u64 {
        self.ground_items
            .values()
            .filter(|stack| stack.kind == kind)
            .map(|stack| u64::from(stack.count))
            .sum()
    }
}
