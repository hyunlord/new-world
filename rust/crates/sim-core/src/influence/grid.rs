//! `InfluenceGrid` — 8-channel double-buffered SoA grid.
//!
//! Phase 0 Section 2.4 base.
//!
//! v0.1.1 ISSUE 5 fix: there is no `source_templates: HashMap<...>` field
//! on this struct. Sources are written directly into `pending` by the BFS
//! / shadowcasting routines.

use crate::influence::channel::InfluenceChannel;

/// Inclusive 2D bounding box, used to mark Cold-tier dirty regions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtyRegion {
    /// Inclusive minimum X.
    pub min_x: u32,
    /// Inclusive minimum Y.
    pub min_y: u32,
    /// Inclusive maximum X.
    pub max_x: u32,
    /// Inclusive maximum Y.
    pub max_y: u32,
}

impl DirtyRegion {
    /// Build a region from arbitrary coordinates (auto-orders min/max).
    pub fn new(x0: u32, y0: u32, x1: u32, y1: u32) -> Self {
        Self {
            min_x: x0.min(x1),
            min_y: y0.min(y1),
            max_x: x0.max(x1),
            max_y: y0.max(y1),
        }
    }
}

/// Double-buffered 8-channel influence grid stored as Struct of Arrays.
///
/// Each channel owns two parallel buffers (`current`, `pending`) and a
/// list of dirty regions. The simulation writes into `pending` during a
/// tick, then calls [`InfluenceGrid::swap`] at tick end.
pub struct InfluenceGrid {
    /// Width in tiles (matches the owning `TileGrid`).
    pub width: u32,
    /// Height in tiles (matches the owning `TileGrid`).
    pub height: u32,
    /// Read-side buffer per channel — sampled by AI / agents.
    pub current: [Vec<u8>; InfluenceChannel::COUNT],
    /// Write-side buffer per channel — accumulates this tick's stamps.
    pub pending: [Vec<u8>; InfluenceChannel::COUNT],
    /// Pending dirty regions per channel (consumed by Cold-tier passes).
    pub dirty_regions: [Vec<DirtyRegion>; InfluenceChannel::COUNT],
}

impl InfluenceGrid {
    /// Allocate a fresh grid. All buffers start zeroed.
    pub fn new(width: u32, height: u32) -> Self {
        let total = (width as usize) * (height as usize);
        let init = || vec![0u8; total];

        Self {
            width,
            height,
            current: std::array::from_fn(|_| init()),
            pending: std::array::from_fn(|_| init()),
            dirty_regions: std::array::from_fn(|_| Vec::new()),
        }
    }

    /// Linear index from `(x, y)` — must match the owning `TileGrid::idx`.
    #[inline]
    pub fn idx(&self, x: u32, y: u32) -> usize {
        debug_assert!(x < self.width && y < self.height);
        (y as usize) * (self.width as usize) + (x as usize)
    }

    /// Total tiles per buffer (`width * height`).
    #[inline]
    pub fn len(&self) -> usize {
        (self.width as usize) * (self.height as usize)
    }

    /// `true` if either dimension is zero.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Read the current value at `(x, y)` for the given channel.
    pub fn sample(&self, x: u32, y: u32, channel: InfluenceChannel) -> u8 {
        self.current[channel as usize][self.idx(x, y)]
    }

    /// Borrow the read-side buffer for a channel (immutable).
    pub fn current_buf(&self, channel: InfluenceChannel) -> &[u8] {
        &self.current[channel as usize]
    }

    /// Borrow the pending (write-side) buffer for a channel (mutable).
    pub fn pending_buf_mut(&mut self, channel: InfluenceChannel) -> &mut [u8] {
        &mut self.pending[channel as usize]
    }

    /// Swap `current` and `pending` for every channel. Call once per tick.
    pub fn swap(&mut self) {
        for ch in 0..InfluenceChannel::COUNT {
            std::mem::swap(&mut self.current[ch], &mut self.pending[ch]);
        }
    }

    /// Mark a dirty region for a channel (typically used for Cold-tier
    /// channels driven by event sourcing).
    pub fn mark_dirty(&mut self, channel: InfluenceChannel, region: DirtyRegion) {
        self.dirty_regions[channel as usize].push(region);
    }

    /// Drop all queued dirty regions for the channel.
    pub fn clear_dirty(&mut self, channel: InfluenceChannel) {
        self.dirty_regions[channel as usize].clear();
    }

    /// Zero the pending buffer for a single channel (used before a
    /// fresh stamp pass for non-additive channels).
    pub fn clear_pending(&mut self, channel: InfluenceChannel) {
        self.pending[channel as usize].fill(0);
    }

    /// Zero the pending buffer for every channel.
    pub fn clear_all_pending(&mut self) {
        for ch in 0..InfluenceChannel::COUNT {
            self.pending[ch].fill(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_new_initializes_zero() {
        let g = InfluenceGrid::new(16, 16);
        for ch in InfluenceChannel::all() {
            for v in g.current_buf(*ch) {
                assert_eq!(*v, 0);
            }
        }
    }

    #[test]
    fn test_idx_matches_tile_grid_layout() {
        let g = InfluenceGrid::new(8, 8);
        assert_eq!(g.idx(0, 0), 0);
        assert_eq!(g.idx(7, 0), 7);
        assert_eq!(g.idx(0, 1), 8);
        assert_eq!(g.idx(7, 7), 63);
    }

    #[test]
    fn test_len_and_empty() {
        let g = InfluenceGrid::new(4, 4);
        assert_eq!(g.len(), 16);
        assert!(!g.is_empty());

        let z = InfluenceGrid::new(4, 0);
        assert!(z.is_empty());
    }

    #[test]
    fn test_sample_after_swap() {
        let mut g = InfluenceGrid::new(8, 8);
        let i = g.idx(3, 3);
        g.pending[InfluenceChannel::Warmth as usize][i] = 200;
        // Before swap, current is still zero.
        assert_eq!(g.sample(3, 3, InfluenceChannel::Warmth), 0);
        g.swap();
        assert_eq!(g.sample(3, 3, InfluenceChannel::Warmth), 200);
    }

    #[test]
    fn test_swap_idempotent_two_swaps() {
        let mut g = InfluenceGrid::new(4, 4);
        let i = g.idx(1, 1);
        g.pending[InfluenceChannel::Light as usize][i] = 50;
        g.swap();
        g.swap();
        // After two swaps, the original pending is back as pending.
        assert_eq!(g.pending[InfluenceChannel::Light as usize][i], 50);
        assert_eq!(g.current[InfluenceChannel::Light as usize][i], 0);
    }

    #[test]
    fn test_clear_pending() {
        let mut g = InfluenceGrid::new(4, 4);
        g.pending[InfluenceChannel::Noise as usize].fill(99);
        g.clear_pending(InfluenceChannel::Noise);
        for v in &g.pending[InfluenceChannel::Noise as usize] {
            assert_eq!(*v, 0);
        }
        // Other channels unaffected.
        g.pending[InfluenceChannel::Light as usize].fill(50);
        g.clear_pending(InfluenceChannel::Noise);
        for v in &g.pending[InfluenceChannel::Light as usize] {
            assert_eq!(*v, 50);
        }
    }

    #[test]
    fn test_dirty_region_tracking() {
        let mut g = InfluenceGrid::new(16, 16);
        g.mark_dirty(InfluenceChannel::Warmth, DirtyRegion::new(0, 0, 4, 4));
        g.mark_dirty(InfluenceChannel::Warmth, DirtyRegion::new(8, 8, 10, 10));
        assert_eq!(g.dirty_regions[InfluenceChannel::Warmth as usize].len(), 2);
        g.clear_dirty(InfluenceChannel::Warmth);
        assert_eq!(g.dirty_regions[InfluenceChannel::Warmth as usize].len(), 0);
    }

    #[test]
    fn test_dirty_region_normalises_corners() {
        let r = DirtyRegion::new(5, 6, 1, 2);
        assert_eq!(r.min_x, 1);
        assert_eq!(r.min_y, 2);
        assert_eq!(r.max_x, 5);
        assert_eq!(r.max_y, 6);
    }
}
