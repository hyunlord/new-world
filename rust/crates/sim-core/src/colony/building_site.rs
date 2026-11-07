//! Persistent template provenance over the existing physical construction jobs.
use super::BlueprintKind;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Additive stable template-site identity, independent of transient ECS entities.
pub type BuildingSiteId = u32;
/// Coordinates in the existing tile grid.
pub type Tile = (u32, u32);

/// Why a template was ordered; the command-log record is stored on the site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BuildCause {
    /// Direct player command.
    Command,
    /// Housing zone identity.
    Zone(u32),
    /// Steward request identity.
    Steward(u32),
}

/// State of one authored piece. Missing physical sites never imply completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PiecePhase {
    /// Wait for earlier overlapping pieces or temporarily obstructed placement.
    Pending,
    /// Existing construction jobs own this piece.
    Active,
    /// Exact physical completion identity was observed.
    Complete,
    /// Existing deconstruction jobs must remove the previous wall first.
    Replacing,
    /// Player cancelled this piece or the active physical site disappeared.
    Cancelled,
}

/// One persistent piece in authored expansion order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingPiece {
    /// Globally monotonic identity; never an ECS entity number.
    pub id: u32,
    /// Existing blueprint identity.
    pub kind: BlueprintKind,
    /// Physical footprint top-left.
    pub tile: Tile,
    /// Physical footprint dimensions.
    pub size: Tile,
    /// Lifecycle state.
    pub phase: PiecePhase,
}
impl BuildingPiece {
    /// Existing physical site lookup key.
    pub fn centre(&self) -> Tile {
        (self.tile.0 + self.size.0 / 2, self.tile.1 + self.size.1 / 2)
    }
    /// Whether another footprint has any tile in common.
    pub fn overlaps(&self, other: &Self) -> bool {
        self.tile.0 < other.tile.0.saturating_add(other.size.0)
            && other.tile.0 < self.tile.0.saturating_add(self.size.0)
            && self.tile.1 < other.tile.1.saturating_add(other.size.1)
            && other.tile.1 < self.tile.1.saturating_add(self.size.1)
    }
}

/// Lifecycle of the aggregate template.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingStatus {
    /// Some work remains; rejected placement can be retried without changing numbers.
    Building,
    /// Every piece has an exact physical completion witness.
    Complete,
    /// Explicit cancel or lost/replaced physical work prevents completion.
    Cancelled,
}

/// Saved template identity, progress, and provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingSite {
    /// Stable site identity.
    pub id: BuildingSiteId,
    /// Authored template id, including extension-pack ids.
    pub template: String,
    /// Selected authored station; normalized default choice is persisted explicitly.
    #[serde(default)]
    pub station: Option<String>,
    /// Unrotated placement anchor.
    pub anchor: Tile,
    /// Clockwise quarter turns, 0 through 3.
    pub rotation: u8,
    /// Actual rotated footprint.
    pub footprint: Tile,
    /// Authored order; activation follows this order.
    pub pieces: Vec<BuildingPiece>,
    /// Source of this order.
    pub cause: BuildCause,
    /// Top-level command record that created this generation of the site.
    pub source_record: u32,
    /// Tick at which this generation started.
    pub placed_tick: u64,
    /// Aggregate lifecycle.
    pub status: BuildingStatus,
    /// Exactly-once completion chronicle link for this generation.
    pub completion_entry: Option<u32>,
}

/// Additive saved state; housing never enters the stockpile tile index.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingState {
    /// Sites retained after completion and cancellation for provenance.
    pub sites: BTreeMap<BuildingSiteId, BuildingSite>,
    /// Next site identity. Checked allocation rejects exhaustion.
    pub next_site_id: u32,
    /// Next piece identity. Checked allocation rejects exhaustion.
    pub next_piece_id: u32,
    /// Housing zones use ColonyState's shared zone allocator but a separate index.
    pub housing_zones: BTreeMap<u32, BTreeSet<Tile>>,
}

impl BuildingState {
    /// Reject stale restored counters rather than overwrite existing provenance identities.
    pub fn next_piece_after(&self, count: u32) -> Option<u32> {
        if self
            .sites
            .values()
            .flat_map(|site| &site.pieces)
            .any(|piece| piece.id >= self.next_piece_id)
        {
            return None;
        }
        self.next_piece_id.checked_add(count)
    }

    /// Read-only allocation preflight; callers commit these counters only after validation.
    pub fn next_ids(&self, count: u32) -> Option<(u32, u32)> {
        if self.sites.keys().any(|&id| id >= self.next_site_id) {
            return None;
        }
        Some((
            self.next_site_id.checked_add(1)?,
            self.next_piece_after(count)?,
        ))
    }
}
