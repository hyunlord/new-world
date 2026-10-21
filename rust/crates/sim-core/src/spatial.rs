//! Spatial index (M1.5 goal 6.2, ADR 0016 C3): 16 × 16-tile chunks.
//!
//! [`ChunkIndex<T>`] keeps `(tile, value)` entries in a `BTreeMap` from chunk
//! id `(cy, cx)` to a vector sorted by `(y, x)` then value. Queries visit
//! chunks in rings of Chebyshev chunk distance around the centre's chunk:
//! [`ChunkIndex::ring`] yields one ring's entries (chunks in `(cy, cx)` order,
//! entries in `(y, x, value)` order), and [`ChunkIndex::nearest_where`]
//! returns the entry with the smallest `(Chebyshev distance, y, x, value)`
//! among those within `max_radius` that satisfy a predicate — exactly the
//! choice of a linear `min_by_key((distance, y, x))` over the same set (the
//! job layer's `nearest`). It visits the non-empty chunks in order of their
//! exact Chebyshev distance to the centre and stops at the first chunk
//! farther than the best entry found, so ties at equal distance in another
//! chunk are still compared (M1.5 DECISIONS D15: a ring walk over every chunk
//! coordinate cost more than the linear scan when candidates are few).
//!
//! [`ColonyIndex`] holds the job searches' indexes (ground stacks, zone
//! cells, chop-marked trees), derived from `ColonyState` and rebuilt when its
//! `overlay_version` changes (every change to zones, designations, ground
//! items, sites or trees bumps it). [`SpatialIndex`] adds the agents' index,
//! built on demand at most once per tick ([`SpatialIndex::agents_at`]).
//! Integer only; nothing here is random or hash-ordered.
use std::collections::BTreeMap;

use crate::colony::ColonyState;
use crate::components::AgentId;

/// Chunk side in tiles.
pub const CHUNK: u32 = 16;

/// A chunk id, `(cy, cx)`: ordered by row, then column.
pub type ChunkId = (u32, u32);

type Tile = (u32, u32);

/// Chebyshev distance between two tiles.
pub const fn chebyshev(a: Tile, b: Tile) -> u32 {
    let dx = a.0.abs_diff(b.0);
    let dy = a.1.abs_diff(b.1);
    if dx > dy {
        dx
    } else {
        dy
    }
}

/// The chunk of `tile`.
pub const fn chunk_of(tile: Tile) -> ChunkId {
    (tile.1 / CHUNK, tile.0 / CHUNK)
}

/// Entries by chunk (see the module header).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkIndex<T> {
    chunks: BTreeMap<ChunkId, Vec<(Tile, T)>>,
    len: usize,
}

impl<T> Default for ChunkIndex<T> {
    fn default() -> Self {
        Self {
            chunks: BTreeMap::new(),
            len: 0,
        }
    }
}

impl<T: Copy + Ord> ChunkIndex<T> {
    /// An empty index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the index holds no entry.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Remove every entry.
    pub fn clear(&mut self) {
        self.chunks.clear();
        self.len = 0;
    }

    /// Add `(tile, value)`, keeping the chunk's entries sorted by `(y, x, value)`.
    pub fn insert(&mut self, tile: Tile, value: T) {
        let id = chunk_of(tile);
        let list = self.chunks.entry(id).or_default();
        let key = (tile.1, tile.0, value);
        let at = list.partition_point(|(t, v)| (t.1, t.0, *v) < key);
        list.insert(at, (tile, value));
        self.len += 1;
    }

    /// Remove one `(tile, value)` entry; returns whether it was there.
    pub fn remove(&mut self, tile: Tile, value: T) -> bool {
        let id = chunk_of(tile);
        let Some(list) = self.chunks.get_mut(&id) else {
            return false;
        };
        let Some(at) = list.iter().position(|e| *e == (tile, value)) else {
            return false;
        };
        list.remove(at);
        if list.is_empty() {
            self.chunks.remove(&id);
        }
        self.len -= 1;
        true
    }

    /// The chunk ids at chunk-Chebyshev distance exactly `r` from `centre`'s
    /// chunk, in `(cy, cx)` order (only ids with entries).
    fn ring_ids(&self, centre: Tile, r: u32) -> Vec<ChunkId> {
        let (cy, cx) = chunk_of(centre);
        let (cy, cx, r) = (i64::from(cy), i64::from(cx), i64::from(r));
        let mut out = Vec::new();
        for y in (cy - r)..=(cy + r) {
            if y < 0 || y > i64::from(u32::MAX) {
                continue;
            }
            let full = y == cy - r || y == cy + r;
            let xs: Vec<i64> = if full {
                ((cx - r)..=(cx + r)).collect()
            } else if r == 0 {
                vec![cx]
            } else {
                vec![cx - r, cx + r]
            };
            for x in xs {
                if x < 0 || x > i64::from(u32::MAX) {
                    continue;
                }
                let id = (y as u32, x as u32);
                if self.chunks.contains_key(&id) {
                    out.push(id);
                }
            }
        }
        out
    }

    /// The entries of chunk ring `radius_chunks` around `centre` (chunks in
    /// `(cy, cx)` order, entries in `(y, x, value)` order).
    pub fn ring(&self, centre: Tile, radius_chunks: u32) -> impl Iterator<Item = &(Tile, T)> {
        self.ring_ids(centre, radius_chunks)
            .into_iter()
            .filter_map(move |id| self.chunks.get(&id))
            .flatten()
    }

    /// Every entry in chunk order (rows of chunks, then `(y, x, value)`).
    pub fn iter(&self) -> impl Iterator<Item = &(Tile, T)> {
        self.chunks.values().flatten()
    }

    /// The Chebyshev distance from `centre` to the nearest tile of chunk `id`.
    fn chunk_distance(centre: Tile, id: ChunkId) -> u32 {
        let axis = |c: u32, lo: u32| lo.saturating_sub(c).max(c.saturating_sub(lo + CHUNK - 1));
        axis(centre.0, id.1 * CHUNK).max(axis(centre.1, id.0 * CHUNK))
    }

    /// The entry with the smallest `(Chebyshev distance to centre, y, x,
    /// value)` among those within `max_radius` for which `pred` holds (see
    /// the module header). The non-empty chunks are visited by their exact
    /// distance to `centre`; the search stops at the first chunk farther than
    /// the best entry found, so a chunk at the best entry's distance is still
    /// compared. `pred` is called only for entries that would beat the best.
    pub fn nearest_where(
        &self,
        centre: Tile,
        max_radius: u32,
        mut pred: impl FnMut(Tile, &T) -> bool,
    ) -> Option<(Tile, T)> {
        if self.len == 0 {
            return None;
        }
        let mut order: Vec<(u32, ChunkId)> = self
            .chunks
            .keys()
            .map(|id| (Self::chunk_distance(centre, *id), *id))
            .filter(|(d, _)| *d <= max_radius)
            .collect();
        order.sort_unstable();
        let mut best: Option<(u32, u32, u32, T)> = None;
        for (lower, id) in order {
            if best.is_some_and(|b| lower > b.0) {
                break;
            }
            let Some(list) = self.chunks.get(&id) else {
                continue;
            };
            for &(tile, value) in list {
                let d = chebyshev(centre, tile);
                if d > max_radius {
                    continue;
                }
                let key = (d, tile.1, tile.0, value);
                if best.is_none_or(|b| key < b) && pred(tile, &value) {
                    best = Some(key);
                }
            }
        }
        best.map(|(_, y, x, value)| ((x, y), value))
    }
}

/// The job searches' indexes (ground stacks, zone cells, chop-marked trees),
/// derived from [`ColonyState`] (see the module header).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ColonyIndex {
    /// The `overlay_version` the indexes were built from (`None` = never).
    pub version: Option<u64>,
    /// Every tile with a ground stack.
    pub ground: ChunkIndex<()>,
    /// Every zoned (stockpile) cell.
    pub cells: ChunkIndex<()>,
    /// Every tile with a chop designation.
    pub marked: ChunkIndex<()>,
}

impl ColonyIndex {
    /// Rebuild from `colony` when its `overlay_version` changed; returns
    /// whether it rebuilt.
    pub fn refresh(&mut self, colony: &ColonyState) -> bool {
        if self.version == Some(colony.overlay_version) {
            return false;
        }
        self.ground.clear();
        self.cells.clear();
        self.marked.clear();
        for tile in colony.ground_items.keys() {
            self.ground.insert(*tile, ());
        }
        for tile in colony.zone_of_tile.keys() {
            self.cells.insert(*tile, ());
        }
        for tile in colony.designations.keys() {
            self.marked.insert(*tile, ());
        }
        self.version = Some(colony.overlay_version);
        true
    }
}

/// All spatial indexes kept in `SimResources::spatial` (derived state:
/// rebuilt from the world, never saved).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpatialIndex {
    /// The colony job searches' indexes.
    pub colony: ColonyIndex,
    /// Agents by tile (`AgentId`); read it through [`Self::agents_at`].
    pub agents: ChunkIndex<AgentId>,
    /// The tick `agents` was built at (`None` = never).
    pub agents_tick: Option<u64>,
}

impl SpatialIndex {
    /// The agents' index for `tick`, rebuilt from `positions` (`(tile,
    /// AgentId)` of every agent) only when it was last built at another
    /// tick: a system that needs agents by place pays O(N) once per tick, and
    /// a tick in which nobody asks pays nothing (DECISIONS D15).
    pub fn agents_at(
        &mut self,
        tick: u64,
        positions: impl FnOnce() -> Vec<(Tile, AgentId)>,
    ) -> &ChunkIndex<AgentId> {
        if self.agents_tick != Some(tick) {
            self.agents.clear();
            for (tile, id) in positions() {
                self.agents.insert(tile, id);
            }
            self.agents_tick = Some(tick);
        }
        &self.agents
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::{keyed_below, STREAM_COLONY_BUSH};

    /// The linear choice the index must reproduce: `min_by_key((d, y, x, v))`.
    fn brute(
        items: &[(Tile, u64)],
        centre: Tile,
        radius: u32,
        pred: impl Fn(Tile, &u64) -> bool,
    ) -> Option<(Tile, u64)> {
        items
            .iter()
            .copied()
            .filter(|(t, v)| chebyshev(centre, *t) <= radius && pred(*t, v))
            .min_by_key(|(t, v)| (chebyshev(centre, *t), t.1, t.0, *v))
    }

    #[test]
    fn nearest_where_matches_the_linear_scan_on_random_sets() {
        let mut checked = 0;
        for set in 0..40u64 {
            let n = 1 + keyed_below(9, STREAM_COLONY_BUSH, set, 0, 0, 0, 300) as usize;
            let side = 32 + keyed_below(9, STREAM_COLONY_BUSH, set, 1, 0, 0, 200) as u32;
            let mut items = Vec::new();
            let mut index = ChunkIndex::new();
            for i in 0..n as u64 {
                let x =
                    keyed_below(9, STREAM_COLONY_BUSH, set, 2, i as u32, 0, u64::from(side)) as u32;
                let y =
                    keyed_below(9, STREAM_COLONY_BUSH, set, 3, i as u32, 0, u64::from(side)) as u32;
                // Several values on one tile now and then (the agents index).
                let v = keyed_below(9, STREAM_COLONY_BUSH, set, 4, i as u32, 0, 4);
                items.push(((x, y), v));
                index.insert((x, y), v);
            }
            assert_eq!(index.len(), n);
            for q in 0..25u32 {
                let cx =
                    keyed_below(9, STREAM_COLONY_BUSH, set, 5, q, 0, u64::from(side) + 20) as u32;
                let cy =
                    keyed_below(9, STREAM_COLONY_BUSH, set, 6, q, 0, u64::from(side) + 20) as u32;
                let radius = [0, 1, 7, 16, 17, 33, 96, u32::MAX][q as usize % 8];
                let parity = q % 3;
                let pred = |t: Tile, v: &u64| match parity {
                    0 => true,
                    1 => (t.0 + t.1).is_multiple_of(2),
                    _ => *v != 1,
                };
                assert_eq!(
                    index.nearest_where((cx, cy), radius, pred),
                    brute(&items, (cx, cy), radius, pred),
                    "set {set} query {q} centre ({cx}, {cy}) radius {radius}"
                );
                checked += 1;
            }
        }
        assert_eq!(checked, 1_000);
    }

    #[test]
    fn equal_distances_break_ties_by_row_then_column_then_value() {
        let mut index = ChunkIndex::new();
        // Four tiles at Chebyshev 16 from (16, 16), in four chunks.
        for t in [(32, 16), (0, 16), (16, 32), (16, 0)] {
            index.insert(t, 0u8);
        }
        assert_eq!(
            index.nearest_where((16, 16), 99, |_, _| true),
            Some(((16, 0), 0))
        );
        // Same tile, two values: the smaller wins.
        index.insert((16, 0), 0);
        let mut agents = ChunkIndex::new();
        agents.insert((5, 5), 9u64);
        agents.insert((5, 5), 3u64);
        assert_eq!(
            agents.nearest_where((0, 0), 10, |_, _| true),
            Some(((5, 5), 3))
        );
        assert!(agents.remove((5, 5), 3));
        assert!(!agents.remove((5, 5), 3));
        assert_eq!(
            agents.nearest_where((0, 0), 10, |_, _| true),
            Some(((5, 5), 9))
        );
        // A tile in the next chunk ring at the same distance as the best is
        // still compared: (17, 0) and (15, 0) are both 1 from (16, 1) and 16/15 lie
        // in chunks 1 and 0; (15, 0) wins on x.
        let mut edge = ChunkIndex::new();
        edge.insert((17, 0), ());
        edge.insert((15, 0), ());
        assert_eq!(
            edge.nearest_where((16, 1), 5, |_, _| true),
            Some(((15, 0), ()))
        );
    }

    #[test]
    fn the_agents_index_is_built_once_per_tick_on_demand() {
        let mut index = SpatialIndex::default();
        let mut builds = 0;
        let positions = |builds: &mut u32| {
            *builds += 1;
            vec![((3, 3), 7u64), ((40, 2), 2)]
        };
        assert_eq!(index.agents_at(5, || positions(&mut builds)).len(), 2);
        assert_eq!(index.agents_at(5, || positions(&mut builds)).len(), 2);
        assert_eq!(builds, 1);
        let nearest =
            index
                .agents_at(6, || positions(&mut builds))
                .nearest_where((39, 0), 10, |_, _| true);
        assert_eq!((nearest, builds), (Some(((40, 2), 2)), 2));
    }

    #[test]
    fn rings_list_chunks_in_row_order_and_respect_the_radius() {
        let mut index = ChunkIndex::new();
        for t in [(1, 1), (20, 1), (1, 20), (50, 50), (20, 20)] {
            index.insert(t, ());
        }
        let ring0: Vec<_> = index.ring((18, 18), 0).map(|e| e.0).collect();
        assert_eq!(ring0, vec![(20, 20)]);
        let ring1: Vec<_> = index.ring((18, 18), 1).map(|e| e.0).collect();
        assert_eq!(ring1, vec![(1, 1), (20, 1), (1, 20)]);
        let ring2: Vec<_> = index.ring((18, 18), 2).map(|e| e.0).collect();
        assert_eq!(ring2, vec![(50, 50)]);
        assert_eq!(
            index.nearest_where((50, 50), 0, |_, _| true),
            Some(((50, 50), ()))
        );
        assert_eq!(index.nearest_where((49, 49), 0, |_, _| true), None);
        assert_eq!(
            index.nearest_where((0, 0), 1, |_, _| true),
            Some(((1, 1), ()))
        );
        assert_eq!(
            ChunkIndex::<()>::new().nearest_where((0, 0), 9, |_, _| true),
            None
        );
    }
}
