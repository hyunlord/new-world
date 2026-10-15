use std::collections::{BTreeMap, BTreeSet};

use sim_core::components::{ActivityKind, BuildingKind, SettlementId};
use sim_core::material::TerrainType;
use sim_core::rng::{keyed_u64, STREAM_ROUTINE_PLACE};
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
        let owned: Vec<_> = resources
            .building_kinds
            .iter()
            .filter_map(|(tile, kind)| {
                let owned = resources.building_owner.get(tile).map_or_else(
                    || {
                        settlement
                            .member_buildings
                            .iter()
                            .any(|id| resources.building_registry.get(id) == Some(tile))
                    },
                    |owner| *owner == sid,
                );
                owned.then_some((*tile, *kind))
            })
            .collect();
        let mut places: [Vec<Tile>; 7] = std::array::from_fn(|_| Vec::new());
        for y in home.1.saturating_sub(14)
            ..=home
                .1
                .saturating_add(14)
                .min(resources.tile_grid.height - 1)
        {
            for x in home.0.saturating_sub(14)
                ..=home.0.saturating_add(14).min(resources.tile_grid.width - 1)
            {
                let tile = (x, y);
                if occupied.contains(&tile)
                    || matches!(
                        resources.tile_grid.terrain_type[resources.tile_grid.idx(x, y)],
                        TerrainType::River | TerrainType::Mountain,
                    )
                {
                    continue;
                }
                let d = distance(tile, home);
                let eligible = [
                    d == 1,
                    d <= 8
                        && owned.iter().any(|(p, kind)| {
                            *kind == BuildingKind::Stockpile && distance(tile, *p) == 1
                        }),
                    d <= 8
                        && owned.iter().any(|(p, kind)| {
                            *kind == BuildingKind::Shelter && distance(tile, *p) == 1
                        }),
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

pub(super) fn choose(
    candidates: &[Tile],
    occupancy: &Occupancy,
    seed: u64,
    key: u64,
    tick: u64,
    replicate: u32,
) -> Option<Tile> {
    let rank = keyed_u64(seed, STREAM_ROUTINE_PLACE, key, tick, 0, replicate);
    candidates
        .iter()
        .copied()
        .filter(|tile| occupancy.get(tile).copied().unwrap_or(0) < 2)
        .min_by_key(|tile| {
            (
                occupancy.get(tile).copied().unwrap_or(0),
                rank,
                tile.1,
                tile.0,
            )
        })
}
