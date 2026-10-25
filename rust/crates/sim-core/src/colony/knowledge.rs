//! Knowledge (M4, ADR 0024–0026): the knowledge map, a person's `Knowledge`,
//! the gate masks and the settlement's technology aggregate. Data only — the
//! channels, forgetting, discovery and the aggregate live in
//! `sim_systems::runtime::knowledge`, the gating in the job layer.
//!
//! The map is data (`data/colony/knowledge.ron`), resolved once by the selected
//! immutable catalog; a node's [`KnowledgeId`] is its index in file order. A person is
//! a **practitioner** of a node when their proficiency (‰) is at or above the
//! node's threshold; [`Knowledge::practice`] caches that as a bitset (bit `i` =
//! node `i`), refreshed by every write. A job is gated by the nodes whose gates
//! cover its recipe, blueprint or designation ([`gates`], precomputed masks).
//!
//! Integers only (ADR 0016 C4): proficiencies in permille, ticks as `u32`.

mod aggregate;
mod component;
mod gate_masks;
mod parse;
mod types;

pub use aggregate::*;
pub use component::*;
pub(crate) use gate_masks::gates_with_catalog;
pub use gate_masks::Gates;
pub(crate) use parse::merge_sources;
pub(crate) use parse::parse_with_catalog;
pub use types::*;

/// A node of the knowledge map: its index in `knowledge.ron` (file order).
pub type KnowledgeId = u8;

/// Most nodes the map may hold (the practitioner bitset is a `u32`).
pub const MAX_NODES: usize = 32;

/// The quality ceiling of a craft by proficiency decile (goal 3.2): the VISION's
/// `0.3 + 0.7·p²` shape as an integer table, index `proficiency / 100`.
pub const QUAL_CAP: [u8; 11] = [0, 20, 35, 45, 55, 65, 75, 85, 92, 97, 100];

/// The quality cap of a craft made at `proficiency` ‰.
pub const fn quality_cap(proficiency: u16) -> u8 {
    let i = (proficiency / 100) as usize;
    QUAL_CAP[if i > 10 { 10 } else { i }]
}

/// Parse using the selected catalog; invalid input is rejected.
pub fn parse_defs(text: &str) -> Option<KnowledgeDefs> {
    parse_with_catalog(text, super::content::installed_or_bundled()).ok()
}

/// The selected immutable catalog's knowledge definitions.
pub fn defs() -> &'static KnowledgeDefs {
    super::content::installed_or_bundled().knowledge_defs()
}

/// The node named `name` in the selected catalog.
pub fn id(name: &str) -> Option<KnowledgeId> {
    defs().id_of(name)
}

/// The bit of node `id` in a practitioner bitset.
pub const fn bit(id: KnowledgeId) -> u32 {
    1u32 << (id as u32 & 31)
}

/// The selected immutable catalog's derived gate masks.
pub fn gates() -> &'static Gates {
    super::content::installed_or_bundled().knowledge_gates()
}

/// The ids set in `mask`, ascending.
pub fn ids_of(mask: u32) -> impl Iterator<Item = KnowledgeId> {
    (0..32u8).filter(move |i| mask & bit(*i) != 0)
}

/// Ticks per day of the knowledge clock (1 tick = 1 minute in every world).
pub const DAY_TICKS: u64 = 1_440;

/// The day index of `tick` as an entry stores it (saturating).
pub fn day_of(tick: u64) -> u16 {
    u16::try_from(tick / DAY_TICKS).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod registry_tests;
#[cfg(test)]
mod tests;
