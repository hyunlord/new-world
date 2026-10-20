//! Exact non-river coverage quantiles over smooth seeded elevation and moisture.
use super::{
    noise::{h64, value},
    WorldGenConfig,
};
use sim_core::{TerrainType, TileGrid};

pub(super) fn generate(cfg: &WorldGenConfig, grid: &mut TileGrid) {
    let mut elevation = vec![0.0; grid.len()];
    let mut moisture = vec![0.0; grid.len()];
    for y in 0..cfg.height {
        for x in 0..cfg.width {
            let i = grid.idx(x, y);
            let e = 0.6 * value(cfg.seed, 1, x, y, 16)
                + 0.3 * value(cfg.seed, 2, x, y, 8)
                + 0.1 * value(cfg.seed, 3, x, y, 4);
            elevation[i] = e;
            moisture[i] = 0.7 * value(cfg.seed, 4, x, y, 12) + 0.3 * value(cfg.seed, 5, x, y, 6);
            grid.elevation[i] = (e * 255.0) as u8;
        }
    }
    let mut river_x =
        (cfg.width / 4 + (h64(cfg.seed, 6, 0, 0) % u64::from(cfg.width / 2)) as u32) as i64;
    for y in 0..cfg.height {
        if y > 0 {
            let r = h64(cfg.seed, 7, 0, y) % 100;
            river_x += if r < 30 {
                -1
            } else if r >= 70 {
                1
            } else {
                0
            };
            let delta = i64::from(cfg.width / 2) - river_x;
            if delta.abs() > i64::from(cfg.width / 4) {
                river_x += delta.signum();
            }
        }
        river_x = river_x.clamp(3, i64::from(cfg.width) - 5);
        for x in river_x as u32..=river_x as u32 + 1 {
            let i = grid.idx(x, y);
            grid.terrain_type[i] = TerrainType::River;
            grid.elevation[i] = 100;
        }
    }
    let mut land: Vec<usize> = (0..grid.len())
        .filter(|&i| grid.terrain_type[i] != TerrainType::River)
        .collect();
    // Every requested fraction uses the SAME denominator: all non-river tiles.
    let total = land.len() as f64;
    let mountains = (total * cfg.mountain_coverage).round() as usize;
    let hills = (total * cfg.hill_coverage).round() as usize;
    let forests = (total * cfg.forest_coverage).round() as usize;
    // Row-major index is precisely the required (y, x) tie-break.
    land.sort_by(|&a, &b| elevation[b].total_cmp(&elevation[a]).then(a.cmp(&b)));
    for &i in land.iter().take(mountains) {
        grid.terrain_type[i] = TerrainType::Mountain;
    }
    for &i in land.iter().skip(mountains).take(hills) {
        grid.terrain_type[i] = TerrainType::Hill;
    }
    let mut lowland = land[mountains + hills..].to_vec();
    lowland.sort_by(|&a, &b| moisture[b].total_cmp(&moisture[a]).then(a.cmp(&b)));
    for &i in lowland.iter().take(forests) {
        grid.terrain_type[i] = TerrainType::Forest;
    }
}
// A few seeds have too little Forest/Plain boundary near the chosen camp.
// Exchange class labels with distant forest only when the exact recipe cannot
// supply all three clusters. Elevation, river, camp, resources and counts stay fixed.
pub(super) fn repair_berry_edges(
    cfg: &WorldGenConfig,
    resources: &mut sim_engine::SimResources,
    camp: (u32, u32),
) {
    use super::camp::{berry_candidates, cluster_centers, distance};
    for swaps in 0..resources.tile_grid.len() {
        let candidates = berry_candidates(resources, camp);
        let centers = cluster_centers(cfg, &candidates);
        if candidates.len() >= cfg.berry_bushes as usize && centers.len() == 3 {
            // Pass 10: counted for measurement only.
            resources.worldgen_report.berry_repair_swaps = swaps as u32;
            return;
        }
        let grid = &resources.tile_grid;
        let mut targets = Vec::new();
        let mut donors = Vec::new();
        for y in 0..grid.height {
            for x in 0..grid.width {
                let p = (x, y);
                if resources.water_tiles.contains_key(&p)
                    || resources.building_kinds.contains_key(&p)
                {
                    continue;
                }
                let d = distance(p, camp);
                match grid.terrain_type[grid.idx(x, y)] {
                    TerrainType::Plain
                        if (6..=16).contains(&d)
                            && grid.neighbors_4(x, y).iter().any(|&(nx, ny)| {
                                grid.terrain_type[grid.idx(nx, ny)] == TerrainType::Plain
                            })
                            && (centers.len() == 3
                                || centers.iter().all(|&q| distance(p, q) >= 6)) =>
                    {
                        targets.push(p)
                    }
                    TerrainType::Forest if d > 16 => donors.push(p),
                    _ => {}
                }
            }
        }
        let rank = |&(x, y): &(u32, u32)| (h64(cfg.seed, 9, x, y), y, x);
        targets.sort_by_key(rank);
        donors.sort_by_key(rank);
        let target = targets
            .first()
            .copied()
            .expect("berry repair needs a local Plain edge");
        let donor = donors
            .first()
            .copied()
            .expect("berry repair needs a distant Forest donor");
        let grid = &mut resources.tile_grid;
        let a = grid.idx(target.0, target.1);
        let b = grid.idx(donor.0, donor.1);
        grid.terrain_type.swap(a, b);
    }
    panic!("berry edge repair exhausted the finite tile budget");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn river_and_exact_coverage() {
        let cfg = WorldGenConfig::baseline(1);
        let mut grid = TileGrid::new(64, 64);
        generate(&cfg, &mut grid);
        for row in grid.terrain_type.chunks(64) {
            assert_eq!(row.iter().filter(|&&t| t == TerrainType::River).count(), 2);
        }
        for (kind, fraction) in [
            (TerrainType::Mountain, cfg.mountain_coverage),
            (TerrainType::Hill, cfg.hill_coverage),
            (TerrainType::Forest, cfg.forest_coverage),
        ] {
            assert_eq!(
                grid.terrain_type.iter().filter(|&&t| t == kind).count(),
                (3968.0_f64 * fraction).round() as usize
            );
        }
    }
}
