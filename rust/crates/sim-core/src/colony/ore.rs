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
/// C2-b metal-chain record: who mined, what tracked goods arrived by trade, and the
/// typed facts behind the first-ingot and first-tool-use Chronicle lines.
///
/// shortcut: sources are recorded per item kind, not per physical unit; a first-event
/// witness lists every recorded source of its inputs. Upgrade to per-unit lots if a
/// line must name exactly which miner's ore went into one batch.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetalState {
    /// Mined units by (yield item, miner), from completed ordinary mining batches.
    pub mined: BTreeMap<(super::ItemKind, crate::components::AgentId), u32>,
    /// Accepted imports of `ore`/`ingot`-tagged goods, in trade order.
    pub imports: Vec<MetalImport>,
    /// First-event facts keyed by their Chronicle index.
    pub witnesses: BTreeMap<u32, MetalWitness>,
}
impl MetalState {
    /// Whether no metal-chain state exists.
    pub fn is_empty(&self) -> bool {
        self.mined.is_empty() && self.imports.is_empty() && self.witnesses.is_empty()
    }
}
/// One accepted trade's tracked goods.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetalImport {
    /// Index of the trade's `Traded` Chronicle entry.
    pub traded: u32,
    /// The neighbour traded with.
    pub neighbour: u8,
    /// Imported item.
    pub item: super::ItemKind,
    /// Imported count.
    pub count: u16,
    /// Data identity of the card the accepted command came from, if any.
    pub card: Option<String>,
}
/// One recipe input of a first ingot and its recorded sources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetalInput {
    /// Input item.
    pub item: super::ItemKind,
    /// Units the recipe consumed.
    pub count: u16,
    /// Recorded miners of this item with their mined units.
    pub miners: Vec<(crate::components::AgentId, u32)>,
    /// Indices into [`MetalState::imports`] of this item.
    pub imports: Vec<u32>,
}
/// Facts behind a metal-chain Chronicle line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetalWitness {
    /// The first time an `ingot`-tagged item was made.
    FirstIngot {
        /// The crafter.
        worker: crate::components::AgentId,
        /// The workshop's work tile.
        workshop: Tile,
        /// The recipe.
        recipe: super::RecipeKind,
        /// The ingot made.
        item: super::ItemKind,
        /// Recipe inputs with their recorded sources.
        inputs: Vec<MetalInput>,
    },
    /// The first successful chop with a `tool`-tagged item of this kind.
    FirstToolUsed {
        /// The user.
        worker: crate::components::AgentId,
        /// The tool item.
        item: super::ItemKind,
    },
}
