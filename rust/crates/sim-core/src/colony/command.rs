//! Player orders as recorded commands (M1, ADR 0012): the command, its result and the log entry.
use serde::{Deserialize, Serialize};

use super::BlueprintKind;
use crate::components::AgentId;

/// One player order. Applied only through `sim_systems::commands::apply_command`
/// and recorded with its tick and result, so a run replays exactly.
/// Rectangles are inclusive tile corners in any order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    /// Mark every tree in the rectangle for felling.
    DesignateChop {
        /// Corner x.
        x0: u32,
        /// Corner y.
        y0: u32,
        /// Opposite corner x.
        x1: u32,
        /// Opposite corner y.
        y1: u32,
    },
    /// Mark the trees on exactly these tiles (scripts and tools); same effect
    /// per listed tree tile as [`Command::DesignateChop`].
    DesignateChopTiles {
        /// Tiles `(x, y)`, in the order given.
        tiles: Vec<(u32, u32)>,
    },
    /// Remove chop marks and blueprints in the rectangle.
    CancelArea {
        /// Corner x.
        x0: u32,
        /// Corner y.
        y0: u32,
        /// Opposite corner x.
        x1: u32,
        /// Opposite corner y.
        y1: u32,
    },
    /// Add the rectangle's valid tiles as one stockpile zone.
    AddStockpile {
        /// Corner x.
        x0: u32,
        /// Corner y.
        y0: u32,
        /// Opposite corner x.
        x1: u32,
        /// Opposite corner y.
        y1: u32,
    },
    /// Remove the rectangle's tiles from every zone.
    RemoveZoneArea {
        /// Corner x.
        x0: u32,
        /// Corner y.
        y0: u32,
        /// Opposite corner x.
        x1: u32,
        /// Opposite corner y.
        y1: u32,
    },
    /// Place a blueprint with its footprint's top-left at `(x, y)`.
    PlaceBlueprint {
        /// The blueprint.
        kind: BlueprintKind,
        /// Footprint top-left x.
        x: u32,
        /// Footprint top-left y.
        y: u32,
    },
    /// Draft or release a person (reserved for M5; rejected in M1).
    SetDrafted {
        /// The person.
        agent: AgentId,
        /// Drafted or not.
        drafted: bool,
    },
}

impl Command {
    /// Stable snake-case name used by command scripts and logs.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::DesignateChop { .. } => "designate_chop",
            Self::DesignateChopTiles { .. } => "designate_chop_tiles",
            Self::CancelArea { .. } => "cancel_area",
            Self::AddStockpile { .. } => "add_stockpile",
            Self::RemoveZoneArea { .. } => "remove_zone_area",
            Self::PlaceBlueprint { .. } => "place_blueprint",
            Self::SetDrafted { .. } => "set_drafted",
        }
    }
}

/// Why a command did nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RejectReason {
    /// The world has no colony rule (the baseline world takes no orders).
    NoColony,
    /// Part or all of the area is outside the map.
    OutOfMap,
    /// A footprint tile is water.
    Water,
    /// A footprint tile is mountain, a tree, a building, a site, a zone or reserved.
    Blocked,
    /// Valid, but there was nothing to change.
    Nothing,
    /// The command exists but is not part of this milestone (drafting, M5).
    NotYet,
}

impl RejectReason {
    /// Stable snake-case name for the bridge and logs.
    pub const fn name(self) -> &'static str {
        match self {
            Self::NoColony => "no_colony",
            Self::OutOfMap => "out_of_map",
            Self::Water => "water",
            Self::Blocked => "blocked",
            Self::Nothing => "nothing",
            Self::NotYet => "not_yet",
        }
    }
}

/// What a command did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandResult {
    /// Whether it changed anything.
    pub accepted: bool,
    /// Trees designated, tiles zoned or removed, things cancelled, sites placed.
    pub count: u32,
    /// Why not, when not accepted.
    pub reason: Option<RejectReason>,
}

impl CommandResult {
    /// Accepted with `count` changes.
    pub const fn ok(count: u32) -> Self {
        Self {
            accepted: true,
            count,
            reason: None,
        }
    }

    /// Rejected for `reason`.
    pub const fn rejected(reason: RejectReason) -> Self {
        Self {
            accepted: false,
            count: 0,
            reason: Some(reason),
        }
    }
}

/// One entry of the command log: applied before the systems of `tick` ran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandRecord {
    /// The tick whose systems ran after the command (the engine's next tick).
    pub tick: u64,
    /// The command.
    pub command: Command,
    /// What it did.
    pub result: CommandResult,
}
