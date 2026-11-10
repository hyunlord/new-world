//! Durable surface deposits and immutable discovery evidence; mechanics live in sim-systems.
use super::{building_site::Tile, content::OutcropKind, knowledge::KnowledgeId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
/// A finite deposit at one physical tile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OreDeposit {
    /// Data-authored content identity.
    pub kind: OutcropKind,
    /// Extractable units; exhausted entries retain their provenance.
    pub remaining: u16,
    /// Whether a real discovery has revealed this tile.
    pub revealed: bool,
    /// Cluster anchor, preserving geological grouping across saves.
    pub cluster: Tile,
}
/// The actual worker and admission facts when a tile first became visible.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OreWitness {
    /// Discovered kind.
    pub kind: OutcropKind,
    /// Physical deposit coordinate.
    pub tile: Tile,
    /// Discoverer's stable person identity.
    pub worker: crate::components::AgentId,
    /// Actual discoverer position at commit.
    pub worker_tile: Tile,
    /// Committed simulation tick.
    pub tick: u64,
    /// Actual admitting knowledge, prospecting or quarrying.
    pub knowledge: KnowledgeId,
    /// Proficiency observed at commit.
    pub proficiency: u16,
    /// Authored practice threshold observed at commit.
    pub required: u16,
    /// Completed quarry target; absent for a walking discovery.
    pub quarry: Option<Tile>,
}
/// Save-tail state, empty for baseline and historical worlds.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OreState {
    /// Monotonic projection invalidation counter.
    pub version: u64,
    /// All generated deposits, including hidden and exhausted ones.
    pub deposits: BTreeMap<Tile, OreDeposit>,
    /// Immutable evidence keyed by the original Chronicle index.
    pub witnesses: BTreeMap<u32, OreWitness>,
}
impl OreState {
    /// Record a physical or discovery change.
    pub fn touch(&mut self) {
        self.version = self.version.saturating_add(1);
    }
    /// Whether no ore feature state exists.
    pub fn is_empty(&self) -> bool {
        self.deposits.is_empty() && self.witnesses.is_empty() && self.version == 0
    }
}
