//! Surface minerals use their own keyed draws after legacy colony generation.
//! Reads: selected origins, physical blockers and a local navigation map. Writes: only ore state.
use sim_core::{
    colony::{
        content::{installed_or_bundled, Catalog},
        knowledge::ContentStatus,
        ore::OreDeposit,
    },
    nav::NavState,
    rng::{keyed_below, keyed_u64, STREAM_ORE_ORIGIN},
    spatial::chebyshev,
};
use sim_engine::SimResources;
/// Populate a new colony using its selected data.
pub fn populate(res: &mut SimResources) {
    populate_with_catalog(res, installed_or_bundled());
}
/// Pure catalogue selection seam used by the data-only extension proof.
/// Existing nonempty state is left untouched. Save loading never calls this entrypoint.
pub fn populate_with_catalog(res: &mut SimResources, catalog: &Catalog) {
    if res.rules.colony.is_none() || !res.ore.is_empty() {
        return;
    }
    let Some(camp) = super::colony::hearth(res) else {
        return;
    };
    let climate = res
        .world_variety
        .unwrap_or_else(|| sim_core::variety::WorldVariety::draw(res.world_seed))
        .climate;
    let mut nav = NavState::default();
    nav.install(crate::runtime::nav::build(res));
    for kind in catalog.outcrop_ids() {
        let Some(def) = catalog
            .outcrop(kind)
            .filter(|d| d.status == ContentStatus::Connected)
        else {
            continue;
        };
        let draw = |slot, upper| {
            keyed_below(
                res.world_seed,
                STREAM_ORE_ORIGIN,
                u64::from(kind.code()),
                0,
                slot,
                0,
                upper,
            )
        };
        if draw(0, 1000) >= u64::from(def.presence_permille) {
            continue;
        }
        let clusters =
            u64::from(def.per_world.0) + draw(1, u64::from(def.per_world.1 - def.per_world.0) + 1);
        let mut candidates = Vec::new();
        for y in camp.1.saturating_sub(def.camp_radius)
            ..=camp
                .1
                .saturating_add(def.camp_radius)
                .min(res.tile_grid.height.saturating_sub(1))
        {
            for x in camp.0.saturating_sub(def.camp_radius)
                ..=camp
                    .0
                    .saturating_add(def.camp_radius)
                    .min(res.tile_grid.width.saturating_sub(1))
            {
                let tile = (x, y);
                let at = res.tile_grid.idx(x, y);
                if def.origin.admits(
                    climate,
                    res.tile_grid.terrain_type[at],
                    res.tile_grid.elevation[at],
                ) && !res.colony.trees.contains_key(&tile)
                    && !res.colony.nodes.contains_key(&tile)
                    && !res.colony.ground_items.contains_key(&tile)
                    && !res.colony.space_sites.contains_key(&tile)
                    && !res.building_kinds.contains_key(&tile)
                    && !res.food.plants.contains_key(&tile)
                    && res.tile_grid.wall_material[at].is_none()
                    && nav.reachable(camp, tile)
                {
                    candidates.push(tile);
                }
            }
        }
        candidates.sort_by_key(|&(x, y)| {
            (
                keyed_u64(
                    res.world_seed,
                    STREAM_ORE_ORIGIN,
                    u64::from(kind.code()),
                    (u64::from(x) << 32) | u64::from(y),
                    2,
                    0,
                ),
                y,
                x,
            )
        });
        let mut made = 0;
        for &anchor in &candidates {
            if made >= clusters {
                break;
            }
            if res.ore.deposits.contains_key(&anchor) {
                continue;
            }
            let size = u64::from(def.cluster.0)
                + keyed_below(
                    res.world_seed,
                    STREAM_ORE_ORIGIN,
                    u64::from(kind.code()),
                    (u64::from(anchor.0) << 32) | u64::from(anchor.1),
                    3,
                    0,
                    u64::from(def.cluster.1 - def.cluster.0) + 1,
                );
            let mut cells: Vec<_> = candidates
                .iter()
                .copied()
                .filter(|tile| {
                    chebyshev(anchor, *tile) <= 1 && !res.ore.deposits.contains_key(tile)
                })
                .collect();
            if cells.len() < usize::from(def.cluster.0) {
                continue;
            }
            cells.sort_by_key(|tile| (chebyshev(anchor, *tile), tile.1, tile.0));
            for tile in cells
                .into_iter()
                .take(usize::try_from(size).unwrap_or(usize::MAX))
            {
                res.ore.deposits.insert(
                    tile,
                    OreDeposit {
                        kind,
                        remaining: def.units,
                        revealed: false,
                        cluster: anchor,
                    },
                );
            }
            made += 1;
        }
    }
    if !res.ore.deposits.is_empty() {
        res.ore.touch();
    }
}
