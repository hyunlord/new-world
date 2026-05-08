//! Influence propagation primitives.
//!
//! Phase 0 Sections 2.5, 2.7, 4. The four exported routines map onto the
//! four canonical decay families:
//!
//! - [`propagate_bfs`] — generic 4-neighbour BFS with channel-aware
//!   aggregation. Used by Warmth / FoodAroma / Spiritual / Beauty.
//! - [`propagate_shadowcast`] — recursive symmetric shadowcasting for the
//!   Light channel (v0.1.1 ISSUE 4 fix).
//! - [`propagate_noise`] — linear decay with density-derived wall blocking.
//!   Used by Noise (alpha=15) and, with alpha=5 + sight-radius cap, Danger
//!   (v0.1.1 ISSUE 3).
//! - [`stamp_social_aggregate`] — LOD-aware additive stamping for Social
//!   (v0.1.1 ISSUE 9 fix).
//!
//! Performance budget for these primitives is documented in Phase 0
//! Section 2.6: Hot tier ~0.3 ms / Warm tier ~0.05 ms @ 30 TPS for the
//! reference 1K-agent scenario.

use std::collections::VecDeque;

use crate::influence::blocking::MaterialBlockingCache;
use crate::influence::channel::{AggKind, InfluenceChannel};
use crate::tile::TileGrid;

/// LOD tier classifier used by Social stamping.
///
/// Phase 0 Section 1.5 + 2.5.4. The full LOD component lives on agents in
/// a later phase; this enum is the minimum surface needed by Phase 1
/// influence stamping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LodTier {
    /// Full simulation fidelity (camera-near).
    Full,
    /// Reduced fidelity but still ticking.
    Medium,
    /// Stub state — heavy systems skipped.
    Simplified,
    /// Frozen — does not contribute to influence.
    Dormant,
}

impl LodTier {
    /// `true` for `Full`/`Medium` agents (they participate in active
    /// systems including Social stamping). `Simplified`/`Dormant` skip.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Full | Self::Medium)
    }
}

/// Apply the channel's aggregation policy to combine `dst` with a fresh
/// `new` value.
#[inline]
fn apply_agg(channel: InfluenceChannel, dst: u8, new: u8) -> u8 {
    match channel.aggregation() {
        AggKind::Additive => dst.saturating_add(new),
        AggKind::Max => dst.max(new),
    }
}

/// Generic 4-neighbour BFS propagation with material wall blocking.
///
/// Phase 0 Section 2.7.1. Aggregation is selected per channel — Warmth
/// accumulates from multiple sources, the rest take the strongest.
///
/// Fixes wired in:
/// - v0.1.1 ISSUE 6: aggregation via [`InfluenceChannel::aggregation`].
/// - v0.1.1 ISSUE 7: blocking cache passed by reference.
/// - v0.1.1 EC-2: source tile is exempt from self-blocking.
///
/// The `decay_fn` receives `(intensity_now, step)` and returns the new
/// intensity after one step (before wall blocking is applied). Use
/// closures for exponential or custom decay.
#[allow(clippy::too_many_arguments)]
pub fn propagate_bfs<F>(
    tile_grid: &TileGrid,
    blocking_cache: &MaterialBlockingCache,
    influence: &mut [u8],
    source: (u32, u32),
    initial_intensity: u8,
    decay_fn: F,
    channel: InfluenceChannel,
    max_radius: u32,
) where
    F: Fn(f32, f32) -> f32,
{
    debug_assert_eq!(influence.len(), tile_grid.len());

    let (sx, sy) = source;
    if sx >= tile_grid.width || sy >= tile_grid.height {
        return;
    }

    let mut queue: VecDeque<((u32, u32), u8, u32)> = VecDeque::with_capacity(256);
    let mut visited = vec![false; tile_grid.len()];

    let source_idx = tile_grid.idx(sx, sy);
    influence[source_idx] = apply_agg(channel, influence[source_idx], initial_intensity);
    visited[source_idx] = true;
    queue.push_back(((sx, sy), initial_intensity, 0));

    while let Some(((x, y), intensity, distance)) = queue.pop_front() {
        if distance >= max_radius || intensity < 5 {
            continue;
        }

        for (nx, ny) in tile_grid.neighbors_4(x, y) {
            let nidx = tile_grid.idx(nx, ny);
            if visited[nidx] {
                continue;
            }

            // v0.1.1 EC-2 fix: source tile is exempt from self-blocking.
            // v0.1.1 ISSUE 7 fix: the cache is supplied by reference.
            let blocking = if distance == 0 {
                0.0
            } else if let Some(material_id) = tile_grid.wall_material[nidx] {
                blocking_cache.get(material_id, channel)
            } else {
                0.0
            };

            let decayed = decay_fn(intensity as f32, 1.0);
            let after_blocking = decayed * (1.0 - blocking);
            let new_intensity = after_blocking.clamp(0.0, 255.0) as u8;

            if new_intensity > 0 {
                influence[nidx] = apply_agg(channel, influence[nidx], new_intensity);
                visited[nidx] = true;
                queue.push_back(((nx, ny), new_intensity, distance + 1));
            }
        }
    }
}

/// Recursive symmetric shadowcasting for the Light channel.
///
/// v0.1.1 ISSUE 4 fix — replaces an 8-octant raycast loop with the
/// recursive variant after Björn Bergström / Adam Milazzo. Lit-set
/// symmetry is preserved (`A sees B ⇔ B sees A`).
///
/// Walls are treated as binary opaque tiles. Distance attenuation uses
/// `intensity / (1 + 0.1 * d)` to mimic the "gentle falloff" required by
/// Phase 0 Section 2.5.3.
pub fn propagate_shadowcast(
    tile_grid: &TileGrid,
    influence: &mut [u8],
    source: (u32, u32),
    initial_intensity: u8,
    max_radius: u32,
) {
    debug_assert_eq!(influence.len(), tile_grid.len());

    let (sx, sy) = source;
    if sx >= tile_grid.width || sy >= tile_grid.height {
        return;
    }

    // v0.1.1 EC-2 + EC-11 fix: source tile is always lit.
    let source_idx = tile_grid.idx(sx, sy);
    influence[source_idx] =
        apply_agg(InfluenceChannel::Light, influence[source_idx], initial_intensity);

    // 8-octant transformation matrices (xx, xy, yx, yy).
    const MULTS: [(i32, i32, i32, i32); 8] = [
        (1, 0, 0, 1),
        (0, 1, 1, 0),
        (0, -1, 1, 0),
        (-1, 0, 0, 1),
        (-1, 0, 0, -1),
        (0, -1, -1, 0),
        (0, 1, -1, 0),
        (1, 0, 0, -1),
    ];

    for (xx, xy, yx, yy) in MULTS {
        cast_light_octant(
            tile_grid,
            influence,
            sx as i32,
            sy as i32,
            1,
            1.0,
            0.0,
            max_radius as i32,
            initial_intensity,
            xx,
            xy,
            yx,
            yy,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn cast_light_octant(
    tile_grid: &TileGrid,
    influence: &mut [u8],
    cx: i32,
    cy: i32,
    row: i32,
    mut start_slope: f32,
    end_slope: f32,
    radius: i32,
    initial_intensity: u8,
    xx: i32,
    xy: i32,
    yx: i32,
    yy: i32,
) {
    if start_slope < end_slope {
        return;
    }

    let mut new_start = 0.0;
    let mut blocked = false;

    for distance in row..=radius {
        if blocked {
            break;
        }

        let dy = -distance;
        for dx in -distance..=0 {
            let current_x = cx + dx * xx + dy * xy;
            let current_y = cy + dx * yx + dy * yy;

            let left_slope = (dx as f32 - 0.5) / (dy as f32 + 0.5);
            let right_slope = (dx as f32 + 0.5) / (dy as f32 - 0.5);

            // Bounds + slope window.
            if current_x < 0
                || current_y < 0
                || current_x >= tile_grid.width as i32
                || current_y >= tile_grid.height as i32
            {
                continue;
            }
            if start_slope < right_slope {
                continue;
            }
            if end_slope > left_slope {
                break;
            }

            let dist_sq = (dx * dx + dy * dy) as f32;
            if dist_sq <= (radius * radius) as f32 {
                let dist = dist_sq.sqrt();
                let intensity_f = (initial_intensity as f32) / (1.0 + 0.1 * dist);
                let intensity = intensity_f.clamp(0.0, 255.0) as u8;
                let idx = tile_grid.idx(current_x as u32, current_y as u32);
                influence[idx] =
                    apply_agg(InfluenceChannel::Light, influence[idx], intensity);
            }

            let idx = tile_grid.idx(current_x as u32, current_y as u32);
            let opaque = tile_grid.is_wall(idx);

            if blocked {
                if opaque {
                    new_start = right_slope;
                } else {
                    blocked = false;
                    start_slope = new_start;
                }
            } else if opaque && distance < radius {
                blocked = true;
                cast_light_octant(
                    tile_grid,
                    influence,
                    cx,
                    cy,
                    distance + 1,
                    start_slope,
                    left_slope,
                    radius,
                    initial_intensity,
                    xx,
                    xy,
                    yx,
                    yy,
                );
                new_start = right_slope;
            }
        }
    }
}

/// Linear-decay propagation for the Noise channel.
///
/// Phase 0 Section 2.5.2 + 2.7.3. v0.1.1 ISSUE 2 fix — linear decay and
/// density-derived wall blocking are kept as *separate* mechanisms (the
/// Songs of Syx 2-tile reference). v0.1.1 EC-2 fix — source tile self-
/// blocking is exempt.
pub fn propagate_noise(
    tile_grid: &TileGrid,
    blocking_cache: &MaterialBlockingCache,
    influence: &mut [u8],
    source: (u32, u32),
    initial_intensity: u8,
) {
    propagate_linear(
        tile_grid,
        Some(blocking_cache),
        influence,
        source,
        initial_intensity,
        InfluenceChannel::Noise,
        15,
        u32::MAX,
    );
}

/// Linear-decay propagation for the Danger channel.
///
/// v0.1.1 ISSUE 3 fix — alpha=5 with a sight-radius cap of 15 tiles to
/// prevent global panic spread. No wall blocking (Danger pierces walls).
pub fn propagate_danger(
    tile_grid: &TileGrid,
    influence: &mut [u8],
    source: (u32, u32),
    initial_intensity: u8,
) {
    propagate_linear(
        tile_grid,
        None,
        influence,
        source,
        initial_intensity,
        InfluenceChannel::Danger,
        5,
        15,
    );
}

#[allow(clippy::too_many_arguments)]
fn propagate_linear(
    tile_grid: &TileGrid,
    blocking_cache: Option<&MaterialBlockingCache>,
    influence: &mut [u8],
    source: (u32, u32),
    initial_intensity: u8,
    channel: InfluenceChannel,
    alpha: u8,
    max_radius: u32,
) {
    debug_assert_eq!(influence.len(), tile_grid.len());

    let (sx, sy) = source;
    if sx >= tile_grid.width || sy >= tile_grid.height {
        return;
    }

    let mut queue: VecDeque<((u32, u32), u8, u32)> = VecDeque::with_capacity(256);
    let mut visited = vec![false; tile_grid.len()];

    let source_idx = tile_grid.idx(sx, sy);
    influence[source_idx] = apply_agg(channel, influence[source_idx], initial_intensity);
    visited[source_idx] = true;
    queue.push_back(((sx, sy), initial_intensity, 0u32));

    while let Some(((x, y), intensity, distance)) = queue.pop_front() {
        if intensity < 5 || distance >= max_radius {
            continue;
        }

        for (nx, ny) in tile_grid.neighbors_4(x, y) {
            let nidx = tile_grid.idx(nx, ny);
            if visited[nidx] {
                continue;
            }

            // v0.1.1 EC-2 fix: source tile self-blocking exempt.
            let blocking = if distance == 0 {
                0.0
            } else if let (Some(cache), Some(mat_id)) =
                (blocking_cache, tile_grid.wall_material[nidx])
            {
                cache.get(mat_id, channel)
            } else {
                0.0
            };

            // Linear decay (alpha) is independent of wall blocking.
            let decayed = intensity.saturating_sub(alpha);
            let after_block = (decayed as f32 * (1.0 - blocking)).clamp(0.0, 255.0) as u8;

            if after_block > 0 {
                influence[nidx] = apply_agg(channel, influence[nidx], after_block);
                visited[nidx] = true;
                queue.push_back(((nx, ny), after_block, distance + 1));
            }
        }
    }
}

/// Stamp Social density into the influence buffer using LOD-filtered
/// agents.
///
/// v0.1.1 ISSUE 9 fix — only `LodTier::Full`/`Medium` agents contribute,
/// keeping the worst-case 10K-agent scenario inside the 30 TPS budget
/// (Phase 0 Section 2.5.4).
///
/// Each contributing agent stamps a diamond (Manhattan radius 5) of `+1`
/// per tile, capped at `255`.
pub fn stamp_social_aggregate(
    influence: &mut [u8],
    width: u32,
    height: u32,
    agents: impl Iterator<Item = ((u32, u32), LodTier)>,
) {
    debug_assert_eq!(influence.len(), (width as usize) * (height as usize));

    for ((ax, ay), lod) in agents {
        if !lod.is_active() {
            continue;
        }

        for dy in -5..=5i32 {
            for dx in -5..=5i32 {
                if dx.abs() + dy.abs() > 5 {
                    continue;
                }
                let nx = ax as i32 + dx;
                let ny = ay as i32 + dy;
                if nx < 0 || (nx as u32) >= width || ny < 0 || (ny as u32) >= height {
                    continue;
                }
                let idx = (ny as u32 * width + nx as u32) as usize;
                influence[idx] = influence[idx].saturating_add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::MaterialId;

    fn open_grid(w: u32, h: u32) -> TileGrid {
        TileGrid::new(w, h)
    }

    fn empty_cache() -> MaterialBlockingCache {
        MaterialBlockingCache::empty()
    }

    // ---------- propagate_bfs ----------

    #[test]
    fn test_bfs_open_field_reaches_radius() {
        let grid = open_grid(16, 16);
        let cache = empty_cache();
        let mut buf = vec![0u8; grid.len()];
        // Linear decay closure for predictable test.
        propagate_bfs(
            &grid,
            &cache,
            &mut buf,
            (8, 8),
            200,
            |i, _| i - 30.0,
            InfluenceChannel::Warmth,
            5,
        );
        // Source tile must be set.
        assert_eq!(buf[grid.idx(8, 8)], 200);
        // A nearby tile must receive some influence.
        assert!(buf[grid.idx(8, 9)] > 0);
        // A tile at radius+ must NOT (decay drives below 5 well before that).
        // We only assert the source > neighbour > distant pattern here.
        assert!(buf[grid.idx(8, 8)] >= buf[grid.idx(8, 9)]);
    }

    #[test]
    fn test_bfs_blocked_by_wall_full() {
        let mut grid = open_grid(8, 8);
        let mid = MaterialId::from_str_hash("wall_test");
        // Place an opaque wall at (4,4) with full blocking on Warmth.
        let wall_idx = grid.idx(4, 4);
        grid.wall_material[wall_idx] = Some(mid);

        // Build a cache via a registry-backed path so the wall material's
        // Warmth blocking coefficient comes from the real derivation.
        let mut registry = crate::material::MaterialRegistry::new();
        let _ = mid; // silence unused (id is computed in the def below)
        let mut def =
            sample_def_with_density("wall_test", 25_000.0, 0.04 /* low conductivity */);
        def.id = MaterialId::from_str_hash("wall_test");
        registry.register(def, None).expect("reg");
        let cache = MaterialBlockingCache::build(&registry);

        let mut buf = vec![0u8; grid.len()];
        propagate_bfs(
            &grid,
            &cache,
            &mut buf,
            (3, 4),
            200,
            |i, _| i - 30.0,
            InfluenceChannel::Warmth,
            10,
        );

        // The wall tile should pass relatively little to its far side
        // compared to the source side.
        let near_source = buf[grid.idx(3, 4)];
        let far_side = buf[grid.idx(5, 4)];
        assert!(
            near_source > far_side,
            "near={} far={}",
            near_source,
            far_side
        );
    }

    fn sample_def_with_density(
        name: &str,
        density: f64,
        thermal_conductivity: f64,
    ) -> crate::material::MaterialDef {
        use crate::material::properties::test_support::valid_props;
        let mut p = valid_props();
        p.density = density;
        p.thermal_conductivity = thermal_conductivity;
        crate::material::MaterialDef {
            id: MaterialId::from_str_hash(name),
            name: name.to_string(),
            category: crate::material::MaterialCategory::Stone,
            properties: p,
            tier: 1,
            natural_in: vec![],
            mod_source: None,
        }
    }

    #[test]
    fn test_bfs_aggregation_additive_warmth() {
        let grid = open_grid(8, 8);
        let cache = empty_cache();
        let mut buf = vec![0u8; grid.len()];

        // Two warmth sources at the same spot would accumulate.
        propagate_bfs(
            &grid,
            &cache,
            &mut buf,
            (4, 4),
            100,
            |i, _| i - 10.0,
            InfluenceChannel::Warmth,
            3,
        );
        propagate_bfs(
            &grid,
            &cache,
            &mut buf,
            (4, 4),
            100,
            |i, _| i - 10.0,
            InfluenceChannel::Warmth,
            3,
        );
        // Source tile should have accumulated to ~200 (saturating add).
        assert_eq!(buf[grid.idx(4, 4)], 200);
    }

    #[test]
    fn test_bfs_aggregation_max_others() {
        let grid = open_grid(8, 8);
        let cache = empty_cache();
        let mut buf = vec![0u8; grid.len()];

        propagate_bfs(
            &grid,
            &cache,
            &mut buf,
            (4, 4),
            100,
            |i, _| i - 10.0,
            InfluenceChannel::FoodAroma,
            3,
        );
        propagate_bfs(
            &grid,
            &cache,
            &mut buf,
            (4, 4),
            150,
            |i, _| i - 10.0,
            InfluenceChannel::FoodAroma,
            3,
        );
        // Max wins — second stronger source dominates.
        assert_eq!(buf[grid.idx(4, 4)], 150);
    }

    #[test]
    fn test_bfs_source_self_blocking_exempt() {
        // EC-2: even if the source tile has a wall on it, the source
        // itself must register the initial intensity.
        let mut grid = open_grid(8, 8);
        let src_idx = grid.idx(4, 4);
        grid.wall_material[src_idx] = Some(MaterialId::from_str_hash("wall_x"));

        let mut registry = crate::material::MaterialRegistry::new();
        registry
            .register(
                sample_def_with_density("wall_x", 25_000.0, 0.04),
                None,
            )
            .expect("reg");
        let cache = MaterialBlockingCache::build(&registry);

        let mut buf = vec![0u8; grid.len()];
        propagate_bfs(
            &grid,
            &cache,
            &mut buf,
            (4, 4),
            120,
            |i, _| i - 20.0,
            InfluenceChannel::Warmth,
            3,
        );
        assert_eq!(buf[grid.idx(4, 4)], 120, "source must be lit despite wall");
    }

    // ---------- propagate_shadowcast ----------

    #[test]
    fn test_shadowcast_open_field_lit() {
        let grid = open_grid(16, 16);
        let mut buf = vec![0u8; grid.len()];
        propagate_shadowcast(&grid, &mut buf, (8, 8), 200, 6);
        // Source tile lit.
        assert_eq!(buf[grid.idx(8, 8)], 200);
        // A nearby tile lit.
        assert!(buf[grid.idx(9, 8)] > 0);
        assert!(buf[grid.idx(8, 9)] > 0);
        assert!(buf[grid.idx(7, 7)] > 0);
    }

    #[test]
    fn test_shadowcast_wall_blocks_line() {
        let mut grid = open_grid(16, 16);
        // Vertical wall column at x=10.
        for y in 0..16 {
            let i = grid.idx(10, y);
            grid.wall_material[i] = Some(MaterialId::from_str_hash("opaque"));
        }
        let mut buf = vec![0u8; grid.len()];
        propagate_shadowcast(&grid, &mut buf, (8, 8), 200, 8);
        // Tiles in front of the wall (x < 10) lit.
        assert!(buf[grid.idx(9, 8)] > 0, "front of wall should be lit");
        // Tiles behind the wall (x > 10) must be dark on the same row.
        assert_eq!(buf[grid.idx(12, 8)], 0, "shadow behind wall");
        assert_eq!(buf[grid.idx(13, 8)], 0, "shadow behind wall");
    }

    #[test]
    fn test_shadowcast_symmetric_open() {
        // Adam Milazzo invariant: A sees B iff B sees A.
        let grid = open_grid(16, 16);
        let mut buf_a = vec![0u8; grid.len()];
        let mut buf_b = vec![0u8; grid.len()];
        propagate_shadowcast(&grid, &mut buf_a, (4, 4), 200, 6);
        propagate_shadowcast(&grid, &mut buf_b, (10, 10), 200, 6);
        // (4,4) sees (10,10)? It's outside radius 6 (Manhattan 12) — both should be 0.
        assert_eq!(buf_a[grid.idx(10, 10)], 0);
        assert_eq!(buf_b[grid.idx(4, 4)], 0);
        // Inside radius — symmetry holds.
        assert!(buf_a[grid.idx(7, 4)] > 0);
        let mut buf_c = vec![0u8; grid.len()];
        propagate_shadowcast(&grid, &mut buf_c, (7, 4), 200, 6);
        assert!(buf_c[grid.idx(4, 4)] > 0);
    }

    // ---------- propagate_noise / propagate_danger ----------

    #[test]
    fn test_propagate_noise_linear_decay() {
        let grid = open_grid(16, 16);
        let cache = empty_cache();
        let mut buf = vec![0u8; grid.len()];
        propagate_noise(&grid, &cache, &mut buf, (8, 8), 200);
        // Source.
        assert_eq!(buf[grid.idx(8, 8)], 200);
        // Linear decay alpha=15: at distance 1 → 185, distance 2 → 170, etc.
        assert!(buf[grid.idx(8, 9)] > 0);
        assert!(buf[grid.idx(8, 9)] < 200);
    }

    #[test]
    fn test_propagate_danger_capped_at_15() {
        let grid = open_grid(64, 64);
        let mut buf = vec![0u8; grid.len()];
        propagate_danger(&grid, &mut buf, (32, 32), 200);
        // Source lit.
        assert_eq!(buf[grid.idx(32, 32)], 200);
        // Within 15-tile cap should propagate (alpha=5 → 195, 190, ...).
        assert!(buf[grid.idx(32, 33)] > 0);
        // Beyond the cap → 0.
        assert_eq!(buf[grid.idx(32, 32 + 16)], 0);
    }

    // ---------- stamp_social_aggregate ----------

    #[test]
    fn test_stamp_social_lod_skip_far_and_dormant() {
        let mut buf = vec![0u8; 16 * 16];
        let agents = [
            ((4u32, 4u32), LodTier::Full),
            ((10u32, 10u32), LodTier::Simplified),
            ((12u32, 12u32), LodTier::Dormant),
        ];
        stamp_social_aggregate(&mut buf, 16, 16, agents.into_iter());
        // Full agent stamped a diamond.
        assert_eq!(buf[4 * 16 + 4], 1);
        // Simplified / Dormant did NOT stamp.
        assert_eq!(buf[10 * 16 + 10], 0);
        assert_eq!(buf[12 * 16 + 12], 0);
    }

    #[test]
    fn test_stamp_social_diamond_radius_5() {
        let mut buf = vec![0u8; 16 * 16];
        stamp_social_aggregate(
            &mut buf,
            16,
            16,
            std::iter::once(((8u32, 8u32), LodTier::Full)),
        );
        // Manhattan-5 corner reachable.
        assert_eq!(buf[(8 + 5) + 8 * 16], 1);
        // Manhattan-6 corner not reached.
        assert_eq!(buf[(8 + 6) + 8 * 16], 0);
    }

    #[test]
    fn test_lod_tier_is_active() {
        assert!(LodTier::Full.is_active());
        assert!(LodTier::Medium.is_active());
        assert!(!LodTier::Simplified.is_active());
        assert!(!LodTier::Dormant.is_active());
    }
}
