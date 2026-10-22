//! Job data (M1): what a worker is doing for the colony. Logic lives in
//! `sim_systems::runtime::jobs`.
use serde::{Deserialize, Serialize};

use super::{BillId, ItemKind, ReserveKey, WorkshopId};
use crate::material::MaterialId;

/// The kinds of work (M2 goal 3.6). The declaration order is the order among
/// equal priorities: `JobAssignSystem` takes the lowest priority number first,
/// then this order, then the nearest target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WorkType {
    /// Bring a bill's inputs to a workshop, then make the item.
    Craft,
    /// Bring materials to a blueprint, then build it.
    Build,
    /// Quarry a designated boulder or outcrop.
    Quarry,
    /// Fell a designated tree.
    Chop,
    /// Split firewood (the wood bench's firewood bill; before a bench, gather deadwood).
    Split,
    /// Carry loose items into a stockpile.
    Haul,
}

impl WorkType {
    /// Every work type in tie-break order.
    pub const ALL: [Self; 6] = [
        Self::Craft,
        Self::Build,
        Self::Quarry,
        Self::Chop,
        Self::Split,
        Self::Haul,
    ];

    /// Stable code for the bridge (the tie-break order, 0 = craft).
    pub const fn code(self) -> u8 {
        match self {
            Self::Craft => 0,
            Self::Build => 1,
            Self::Quarry => 2,
            Self::Chop => 3,
            Self::Split => 4,
            Self::Haul => 5,
        }
    }

    /// Stable lowercase name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Craft => "craft",
            Self::Build => "build",
            Self::Quarry => "quarry",
            Self::Chop => "chop",
            Self::Split => "split",
            Self::Haul => "haul",
        }
    }

    /// Parse [`Self::name`].
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|w| w.name() == name)
    }

    /// The type of the bridge's `code`.
    pub fn from_code(code: u8) -> Option<Self> {
        Self::ALL.into_iter().find(|w| w.code() == code)
    }
}

/// One job's target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobKind {
    /// Fell the tree at `tree`.
    Chop {
        /// The tree tile.
        tree: (u32, u32),
    },
    /// Carry logs from the stack at `from` to the stockpile cell `to`.
    Haul {
        /// The source stack tile.
        from: (u32, u32),
        /// The destination cell.
        to: (u32, u32),
    },
    /// Carry logs from the stack at `from` to the site whose centre is `site`.
    Deliver {
        /// The source stack tile.
        from: (u32, u32),
        /// The site centre.
        site: (u32, u32),
    },
    /// Build the site whose centre is `site`.
    Build {
        /// The site centre.
        site: (u32, u32),
    },
    /// Quarry one batch from the resource node at `node` (M2).
    Quarry {
        /// The node tile.
        node: (u32, u32),
    },
    /// Carry a bill input from the stack at `from` to the input shelf of
    /// `workshop`, delivered at its work tile `to` (M2: `Deliver` for a shelf).
    Supply {
        /// The source stack tile.
        from: (u32, u32),
        /// The workshop.
        workshop: WorkshopId,
        /// Its work tile.
        to: (u32, u32),
    },
    /// Gather deadwood at the spot `spot` into firewood items and carry them to
    /// `to`: a stockpile cell, or the hearth pile when no cell has room (M2
    /// goal 3.8, `WorkType::Split`'s fallback).
    Gather {
        /// The deadwood spot (`SimResources.wood_tiles`).
        spot: (u32, u32),
        /// Where the firewood goes.
        to: (u32, u32),
    },
    /// Make one item of `bill` at `workshop`, working at its work tile `at` (M2).
    Craft {
        /// The workshop.
        workshop: WorkshopId,
        /// The bill.
        bill: BillId,
        /// Its work tile.
        at: (u32, u32),
    },
}

impl JobKind {
    /// Stable code for the bridge's job column (0 is "no job").
    pub const fn code(self) -> u8 {
        match self {
            Self::Chop { .. } => 1,
            Self::Haul { .. } => 2,
            Self::Deliver { .. } => 3,
            Self::Build { .. } => 4,
            Self::Quarry { .. } => 5,
            Self::Supply { .. } => 6,
            Self::Craft { .. } => 7,
            Self::Gather { .. } => 8,
        }
    }

    /// Stable name for logs and the bridge.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Chop { .. } => "chop",
            Self::Haul { .. } => "haul",
            Self::Deliver { .. } => "deliver",
            Self::Build { .. } => "build",
            Self::Quarry { .. } => "quarry",
            Self::Supply { .. } => "supply",
            Self::Craft { .. } => "craft",
            Self::Gather { .. } => "gather",
        }
    }

    /// The work type the job counts as.
    pub const fn work(self) -> WorkType {
        match self {
            Self::Chop { .. } => WorkType::Chop,
            Self::Haul { .. } => WorkType::Haul,
            Self::Deliver { .. } | Self::Build { .. } => WorkType::Build,
            Self::Quarry { .. } => WorkType::Quarry,
            Self::Supply { .. } | Self::Craft { .. } => WorkType::Craft,
            Self::Gather { .. } => WorkType::Split,
        }
    }

    /// The tile the worker walks to first.
    pub const fn source(self) -> (u32, u32) {
        match self {
            Self::Chop { tree } => tree,
            Self::Haul { from, .. } | Self::Deliver { from, .. } => from,
            Self::Build { site } => site,
            Self::Quarry { node } => node,
            Self::Supply { from, .. } => from,
            Self::Craft { at, .. } => at,
            Self::Gather { spot, .. } => spot,
        }
    }

    /// The tile the load goes to (Haul and Deliver).
    pub const fn dest(self) -> Option<(u32, u32)> {
        match self {
            Self::Haul { to, .. } | Self::Supply { to, .. } | Self::Gather { to, .. } => Some(to),
            Self::Deliver { site, .. } => Some(site),
            Self::Chop { .. } | Self::Build { .. } | Self::Quarry { .. } | Self::Craft { .. } => {
                None
            }
        }
    }
}

/// Where a job is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStage {
    /// Walking to the source.
    ToSource,
    /// Working at the source (chopping).
    Working {
        /// Work ticks still to do.
        ticks_left: u32,
    },
    /// Walking the load to its destination.
    ToDest,
}

/// The job a worker holds (an ECS component).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    /// What and where.
    pub kind: JobKind,
    /// How far along.
    pub stage: JobStage,
    /// Tick the job was taken.
    pub taken_at: u64,
    /// Items this job moves (Haul / Deliver): reserved at the stack, then carried.
    pub amount: u16,
    /// The kind of item it moves (M2; `Log` for jobs that move none).
    pub item: ItemKind,
    /// A craft's main input material, set when the craft starts (M2; carried to the output).
    pub material: Option<MaterialId>,
    /// A chop job's tool stop: the stockpile stack where the worker picks up
    /// an axe on the way to the tree (M2 goal 3.7); `None` once picked up.
    pub fetch: Option<(u32, u32)>,
    /// Smallest Chebyshev distance to the current walking goal so far.
    pub best_distance: u32,
    /// Tick `best_distance` last shrank (the stuck rule, 7.5).
    pub progress_tick: u64,
}

impl Job {
    /// The reservation keys this job holds in its current stage (7.6: a job
    /// holds exactly these). Haul and Deliver hold their stack only until
    /// pickup; Haul holds its cell until the drop.
    pub fn keys(&self) -> Vec<ReserveKey> {
        match (self.kind, self.stage) {
            (JobKind::Chop { tree }, JobStage::ToSource) => match self.fetch {
                Some(axe) => vec![ReserveKey::Tree(tree), ReserveKey::Stack(axe)],
                None => vec![ReserveKey::Tree(tree)],
            },
            (JobKind::Chop { tree }, _) => vec![ReserveKey::Tree(tree)],
            (JobKind::Haul { from, to }, JobStage::ToSource) => {
                vec![ReserveKey::Stack(from), ReserveKey::Cell(to)]
            }
            (JobKind::Haul { to, .. }, _) => vec![ReserveKey::Cell(to)],
            (JobKind::Deliver { from, .. }, JobStage::ToSource) => vec![ReserveKey::Stack(from)],
            (JobKind::Deliver { .. }, _) => Vec::new(),
            (JobKind::Build { site }, _) => vec![ReserveKey::Site(site)],
            (JobKind::Quarry { node }, _) => vec![ReserveKey::Node(node)],
            (JobKind::Supply { from, .. }, JobStage::ToSource) => vec![ReserveKey::Stack(from)],
            (JobKind::Supply { .. }, _) => Vec::new(),
            (JobKind::Craft { at, .. }, _) => vec![ReserveKey::Bench(at)],
            (JobKind::Gather { spot, .. }, JobStage::ToSource | JobStage::Working { .. }) => {
                vec![ReserveKey::Wood(spot)]
            }
            (JobKind::Gather { .. }, JobStage::ToDest) => Vec::new(),
        }
    }
}

/// Why a job ended without finishing (counted in `ColonyTallies`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CancelReason {
    /// The decision system chose survival work (food, water, sleep, foraging,
    /// firewood, social, fire, flight).
    Needs,
    /// The tree, stack, cell or site is gone.
    TargetGone,
    /// A command removed the target.
    Command,
    /// No Chebyshev progress for `stuck_ticks`.
    Stuck,
    /// Another system changed the worker's state (fission, combat, a finished site).
    Interrupted,
    /// The worker died.
    Died,
    /// The goal can no longer be reached (M3, ADR 0021: a wall or a site cut
    /// the way); the worker avoids the target for a day.
    Unreachable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_follow_the_stage() {
        let mut job = Job {
            kind: JobKind::Haul {
                from: (1, 1),
                to: (5, 5),
            },
            stage: JobStage::ToSource,
            taken_at: 0,
            amount: 5,
            item: ItemKind::Log,
            material: None,
            fetch: None,
            best_distance: 4,
            progress_tick: 0,
        };
        assert_eq!(
            job.keys(),
            vec![ReserveKey::Stack((1, 1)), ReserveKey::Cell((5, 5))]
        );
        job.stage = JobStage::ToDest;
        assert_eq!(job.keys(), vec![ReserveKey::Cell((5, 5))]);
        job.kind = JobKind::Deliver {
            from: (1, 1),
            site: (9, 9),
        };
        assert!(job.keys().is_empty());
        assert_eq!(job.kind.dest(), Some((9, 9)));
        assert_eq!(
            [
                JobKind::Chop { tree: (0, 0) }.code(),
                job.kind.code(),
                JobKind::Build { site: (0, 0) }.code()
            ],
            [1, 3, 4]
        );
    }
}
