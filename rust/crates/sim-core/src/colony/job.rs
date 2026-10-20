//! Job data (M1): what a worker is doing for the colony. Logic lives in
//! `sim_systems::runtime::jobs`.
use serde::{Deserialize, Serialize};

use super::ReserveKey;

/// The kinds of work, in M1's fixed priority order (Build > Chop > Haul).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WorkType {
    /// Bring logs to a blueprint, then build it.
    Build,
    /// Fell a designated tree.
    Chop,
    /// Carry loose logs into a stockpile.
    Haul,
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
}

impl JobKind {
    /// Stable code for the bridge's job column (0 is "no job").
    pub const fn code(self) -> u8 {
        match self {
            Self::Chop { .. } => 1,
            Self::Haul { .. } => 2,
            Self::Deliver { .. } => 3,
            Self::Build { .. } => 4,
        }
    }

    /// Stable name for logs and the bridge.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Chop { .. } => "chop",
            Self::Haul { .. } => "haul",
            Self::Deliver { .. } => "deliver",
            Self::Build { .. } => "build",
        }
    }

    /// The tile the worker walks to first.
    pub const fn source(self) -> (u32, u32) {
        match self {
            Self::Chop { tree } => tree,
            Self::Haul { from, .. } | Self::Deliver { from, .. } => from,
            Self::Build { site } => site,
        }
    }

    /// The tile the load goes to (Haul and Deliver).
    pub const fn dest(self) -> Option<(u32, u32)> {
        match self {
            Self::Haul { to, .. } => Some(to),
            Self::Deliver { site, .. } => Some(site),
            Self::Chop { .. } | Self::Build { .. } => None,
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
    /// Logs this job moves (Haul / Deliver): reserved at the stack, then carried.
    pub amount: u16,
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
            (JobKind::Chop { tree }, _) => vec![ReserveKey::Tree(tree)],
            (JobKind::Haul { from, to }, JobStage::ToSource) => {
                vec![ReserveKey::Stack(from), ReserveKey::Cell(to)]
            }
            (JobKind::Haul { to, .. }, _) => vec![ReserveKey::Cell(to)],
            (JobKind::Deliver { from, .. }, JobStage::ToSource) => vec![ReserveKey::Stack(from)],
            (JobKind::Deliver { .. }, _) => Vec::new(),
            (JobKind::Build { site }, _) => vec![ReserveKey::Site(site)],
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
