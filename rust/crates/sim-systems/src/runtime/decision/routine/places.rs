use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use sim_core::components::{ActivityKind, BuildingId, BuildingKind, SettlementId};
use sim_core::material::TerrainType;
use sim_engine::SimResources;

pub(super) type Tile = (u32, u32);
pub(super) type Places = BTreeMap<SettlementId, [Vec<Tile>; 7]>;
pub(super) type Occupancy = BTreeMap<Tile, u32>;

pub(super) const KINDS: [ActivityKind; 7] = [
    ActivityKind::TendFire,
    ActivityKind::Craft,
    ActivityKind::Repair,
    ActivityKind::HearthCircle,
    ActivityKind::Explore,
    ActivityKind::Play,
    ActivityKind::SitByFire,
];

const fn distance(a: Tile, b: Tile) -> u32 {
    let dx = a.0.abs_diff(b.0);
    let dy = a.1.abs_diff(b.1);
    if dx > dy {
        dx
    } else {
        dy
    }
}

/// Inclusive scan bounds of the 29 × 29 area around `home`.
fn scan_area(home: Tile, width: u32, height: u32) -> (std::ops::RangeInclusive<u32>, std::ops::RangeInclusive<u32>) {
    (
        home.0.saturating_sub(14)..=home.0.saturating_add(14).min(width - 1),
        home.1.saturating_sub(14)..=home.1.saturating_add(14).min(height - 1),
    )
}

fn blocked(resources: &SimResources, x: u32, y: u32) -> bool {
    matches!(
        resources.tile_grid.terrain_type[resources.tile_grid.idx(x, y)],
        TerrainType::River | TerrainType::Mountain,
    )
}

/// Whether one of the eight neighbours of `tile` (Chebyshev distance 1) is in `set`.
fn next_to(set: &BTreeSet<Tile>, tile: Tile) -> bool {
    (-1_i64..=1).any(|dy| {
        (-1_i64..=1).any(|dx| {
            (dx, dy) != (0, 0)
                && u32::try_from(i64::from(tile.0) + dx)
                    .ok()
                    .zip(u32::try_from(i64::from(tile.1) + dy).ok())
                    .is_some_and(|p| set.contains(&p))
        })
    })
}

/// Every settlement's candidate tiles per activity kind, in `(y, x)` order.
pub(super) fn collect(resources: &SimResources) -> Places {
    let occupied: BTreeSet<_> = resources
        .building_registry
        .values()
        .copied()
        .chain(resources.building_kinds.keys().copied())
        .collect();
    let mut result = BTreeMap::new();
    for (&sid, settlement) in &resources.settlements {
        let home = settlement.formation_tile;
        let mut stockpiles = BTreeSet::new();
        let mut shelters = BTreeSet::new();
        for (tile, kind) in &resources.building_kinds {
            let owned = resources.building_owner.get(tile).map_or_else(
                || {
                    settlement
                        .member_buildings
                        .iter()
                        .any(|id| resources.building_registry.get(id) == Some(tile))
                },
                |owner| *owner == sid,
            );
            if owned {
                match kind {
                    BuildingKind::Stockpile => stockpiles.insert(*tile),
                    BuildingKind::Shelter => shelters.insert(*tile),
                    _ => false,
                };
            }
        }
        let mut places: [Vec<Tile>; 7] = std::array::from_fn(|_| Vec::new());
        let (xs, ys) = scan_area(home, resources.tile_grid.width, resources.tile_grid.height);
        for y in ys {
            for x in xs.clone() {
                let tile = (x, y);
                if occupied.contains(&tile) || blocked(resources, x, y) {
                    continue;
                }
                let d = distance(tile, home);
                let eligible = [
                    d == 1,
                    d <= 8 && next_to(&stockpiles, tile),
                    d <= 8 && next_to(&shelters, tile),
                    d == 2,
                    (10..=14).contains(&d),
                    (2..=3).contains(&d),
                    (1..=2).contains(&d),
                ];
                for (index, allowed) in eligible.into_iter().enumerate() {
                    if allowed {
                        places[index].push(tile);
                    }
                }
            }
        }
        result.insert(sid, places);
    }
    result
}

/// Pass 18: the inputs [`collect`] reads, to tell when a cached result is stale.
#[derive(Debug, Default, PartialEq)]
struct Inputs {
    kinds: BTreeMap<Tile, BuildingKind>,
    owner: BTreeMap<Tile, SettlementId>,
    registry: HashMap<BuildingId, Tile>,
    settlements: BTreeMap<SettlementId, (Tile, HashSet<BuildingId>)>,
    size: (u32, u32),
    /// River/Mountain flags of each settlement's scan area (settlement id
    /// order, then scan order).
    blocked: Vec<bool>,
}

impl Inputs {
    fn of(resources: &SimResources) -> Self {
        let settlements: BTreeMap<_, _> = resources
            .settlements
            .iter()
            .map(|(&sid, s)| (sid, (s.formation_tile, s.member_buildings.clone())))
            .collect();
        let size = (resources.tile_grid.width, resources.tile_grid.height);
        let mut blocked_flags = Vec::new();
        for (home, _) in settlements.values() {
            let (xs, ys) = scan_area(*home, size.0, size.1);
            for y in ys {
                for x in xs.clone() {
                    blocked_flags.push(blocked(resources, x, y));
                }
            }
        }
        Self {
            kinds: resources.building_kinds.clone(),
            owner: resources.building_owner.clone(),
            registry: resources.building_registry.clone(),
            settlements,
            size,
            blocked: blocked_flags,
        }
    }

    /// Equality with [`Self::of`]`(resources)` without building it.
    fn matches(&self, resources: &SimResources) -> bool {
        let size = (resources.tile_grid.width, resources.tile_grid.height);
        if self.size != size
            || self.kinds != resources.building_kinds
            || self.owner != resources.building_owner
            || self.registry != resources.building_registry
            || self.settlements.len() != resources.settlements.len()
            || !resources.settlements.iter().all(|(sid, s)| {
                self.settlements
                    .get(sid)
                    .is_some_and(|(home, buildings)| *home == s.formation_tile && *buildings == s.member_buildings)
            })
        {
            return false;
        }
        let mut flags = self.blocked.iter();
        for (home, _) in self.settlements.values() {
            let (xs, ys) = scan_area(*home, size.0, size.1);
            for y in ys {
                for x in xs.clone() {
                    if flags.next() != Some(&blocked(resources, x, y)) {
                        return false;
                    }
                }
            }
        }
        flags.next().is_none()
    }
}

/// Pass 18: [`collect`]'s result kept across ticks while its inputs are equal.
#[derive(Debug, Default)]
pub(in crate::runtime::decision) struct PlacesCache {
    inputs: Option<Inputs>,
    places: Places,
}

impl PlacesCache {
    /// Recompute the places of `resources` only when an input changed.
    pub(super) fn refresh(&mut self, resources: &SimResources) {
        if !self.inputs.as_ref().is_some_and(|inputs| inputs.matches(resources)) {
            self.places = scope!("routine_places_collect", { collect(resources) });
            self.inputs = Some(Inputs::of(resources));
        }
    }

    /// The places as of the last [`Self::refresh`].
    pub(super) const fn places(&self) -> &Places {
        &self.places
    }
}

/// The free candidate with the fewest occupants, first in `(y, x)` order:
/// the reference [`Cursor::choose`] is tested against.
///
/// Pass 17 minimised `(occupancy, rank, y, x)` with a rank keyed by the
/// person and tick but not the tile, so the rank never decides; candidates
/// are generated in `(y, x)` order, so the result is the first tile with no
/// occupant, else the first with one (a tile with two is full).
#[cfg(test)]
pub(super) fn choose(candidates: &[Tile], occupancy: &Occupancy) -> Option<Tile> {
    let mut single = None;
    for tile in candidates {
        match occupancy.get(tile).copied().unwrap_or(0) {
            0 => return Some(*tile),
            1 if single.is_none() => single = Some(*tile),
            _ => {}
        }
    }
    single
}

/// Pass 18: a per-tick scan position over one candidate list, giving
/// [`choose`]'s answer without rescanning the occupied front of the list.
///
/// Invariants: every tile before `zero` has an occupant; every tile before
/// `one` has two (`one` advances only when no tile is free). Occupancy only
/// grows within a tick except at `end`, which calls [`Cursor::rewind`].
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Cursor {
    zero: usize,
    one: usize,
}

impl Cursor {
    /// The same tile as [`choose`]`(candidates, occupancy)`.
    pub(super) fn choose(&mut self, candidates: &[Tile], occupancy: &Occupancy) -> Option<Tile> {
        let count = |tile: &Tile| occupancy.get(tile).copied().unwrap_or(0);
        while candidates.get(self.zero).is_some_and(|tile| count(tile) != 0) {
            self.zero += 1;
        }
        if let Some(tile) = candidates.get(self.zero) {
            return Some(*tile);
        }
        while candidates.get(self.one).is_some_and(|tile| count(tile) != 1) {
            self.one += 1;
        }
        candidates.get(self.one).copied()
    }

    /// `tile`'s occupancy fell: rescan from it (lists are in `(y, x)` order).
    pub(super) fn rewind(&mut self, candidates: &[Tile], tile: Tile) {
        if let Ok(index) = candidates.binary_search_by_key(&(tile.1, tile.0), |t| (t.1, t.0)) {
            self.zero = self.zero.min(index);
            self.one = self.one.min(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::rng::{keyed_u64, STREAM_ROUTINE_PLACE};

    /// Pass 17's `choose`, kept to prove the short-circuit equal to it.
    fn choose_p17(candidates: &[Tile], occupancy: &Occupancy, seed: u64, key: u64, tick: u64) -> Option<Tile> {
        let rank = keyed_u64(seed, STREAM_ROUTINE_PLACE, key, tick, 0, 0);
        candidates
            .iter()
            .copied()
            .filter(|tile| occupancy.get(tile).copied().unwrap_or(0) < 2)
            .min_by_key(|tile| (occupancy.get(tile).copied().unwrap_or(0), rank, tile.1, tile.0))
    }

    #[test]
    fn short_circuit_choice_equals_the_pass17_minimum() {
        let candidates: Vec<Tile> = (0..6).flat_map(|y| (0..6).map(move |x| (x, y))).collect();
        for pattern in 0_u64..2_000 {
            let mut occupancy = Occupancy::new();
            let mut bits = pattern.wrapping_mul(0x9e37_79b9_7f4a_7c15);
            for tile in &candidates {
                let count = (bits % 3) as u32;
                bits = bits.rotate_left(7) ^ pattern;
                if count > 0 || bits % 5 == 0 {
                    occupancy.insert(*tile, count);
                }
            }
            for key in [0, 7, u64::MAX] {
                assert_eq!(choose(&candidates, &occupancy), choose_p17(&candidates, &occupancy, 3, key, pattern));
            }
        }
    }

    #[test]
    fn cursor_matches_choose_under_starts_and_ends() {
        let candidates: Vec<Tile> = (0..5).flat_map(|y| (0..5).map(move |x| (x * 2, y))).collect();
        for run in 0_u64..300 {
            let mut occupancy = Occupancy::new();
            let mut cursor = Cursor::default();
            let mut state = run.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
            let mut held: Vec<Tile> = Vec::new();
            for _ in 0..200 {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                assert_eq!(cursor.choose(&candidates, &occupancy), choose(&candidates, &occupancy), "run {run}");
                if state % 4 == 0 && !held.is_empty() {
                    let tile = held.swap_remove((state as usize / 4) % held.len());
                    if let Some(count) = occupancy.get_mut(&tile) {
                        *count = count.saturating_sub(1);
                    }
                    cursor.rewind(&candidates, tile);
                } else if let Some(tile) = choose(&candidates, &occupancy) {
                    *occupancy.entry(tile).or_default() += 1;
                    held.push(tile);
                } else if state % 3 == 0 {
                    // A start of another kind on a tile outside this list.
                    *occupancy.entry((99, 99)).or_default() += 1;
                }
            }
        }
    }

    #[test]
    fn next_to_is_chebyshev_distance_one() {
        let set: BTreeSet<Tile> = [(0, 0), (5, 5), (9, 0)].into_iter().collect();
        for y in 0..12 {
            for x in 0..12 {
                let expected = set.iter().any(|p| distance((x, y), *p) == 1);
                assert_eq!(next_to(&set, (x, y)), expected, "{x},{y}");
            }
        }
    }
}
