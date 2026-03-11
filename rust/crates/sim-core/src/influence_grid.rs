use crate::config;
use crate::influence_channel::{ChannelClampPolicy, ChannelId, ChannelMeta};
use crate::wall_mask::WallBlockingMask;
use serde::{Deserialize, Serialize};

/// One active influence emitter stamped into the runtime grid.
#[derive(Debug, Clone, PartialEq)]
pub struct EmitterRecord {
    /// Tile-space x coordinate.
    pub x: u32,
    /// Tile-space y coordinate.
    pub y: u32,
    /// Channel emitted by this source.
    pub channel: ChannelId,
    /// Radius in tile units.
    pub radius: f64,
    /// Raw emission intensity written before normalization/clamp.
    pub base_intensity: f64,
    /// Distance falloff profile.
    pub falloff: FalloffType,
    /// Optional source attenuation override.
    pub decay_rate: Option<f64>,
    /// Optional semantic tags.
    pub tags: Vec<String>,
    /// When true, this emitter is re-applied on the next update.
    pub dirty: bool,
}

/// Distance attenuation profile used during stamping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FalloffType {
    /// `intensity * (1 - dist / radius)`.
    Linear,
    /// `intensity * exp(-dist / radius)`.
    Exponential,
    /// `intensity * exp(-(dist^2) / (2 * sigma^2))`, where `sigma = radius / 2`.
    Gaussian,
    /// Legacy compatibility falloff: `intensity / (1 + dist^2)`.
    InverseSquare,
    /// Constant intensity within the radius.
    Constant,
}

/// Double-buffered spatial influence grid shared by the simulation runtime.
#[derive(Debug, Clone)]
pub struct InfluenceGrid {
    width: u32,
    height: u32,
    channel_count: usize,
    current: Vec<Vec<f64>>,
    pending: Vec<Vec<f64>>,
    wall_blocking: WallBlockingMask,
    channel_meta: Vec<ChannelMeta>,
    active_emitters: Vec<EmitterRecord>,
    stagger_index: usize,
}

impl InfluenceGrid {
    /// Creates a new influence grid for the given dimensions and channel metadata.
    pub fn new(width: u32, height: u32, channels: Vec<ChannelMeta>) -> Self {
        let channel_meta = merge_channel_meta(channels);
        let cell_count = (width * height) as usize;
        let channel_count = ChannelId::count();
        Self {
            width,
            height,
            channel_count,
            current: vec![vec![0.0; cell_count]; channel_count],
            pending: vec![vec![0.0; cell_count]; channel_count],
            wall_blocking: WallBlockingMask::new(width, height),
            channel_meta,
            active_emitters: Vec::new(),
            stagger_index: 0,
        }
    }

    /// Replaces channel metadata in place, preserving current buffer contents.
    pub fn set_channel_meta(&mut self, channels: &[ChannelMeta]) {
        self.channel_meta = merge_channel_meta(channels.to_vec());
    }

    /// Returns the current metadata for one channel.
    pub fn channel_meta(&self, channel: ChannelId) -> &ChannelMeta {
        &self.channel_meta[channel.index()]
    }

    /// Registers a new active emitter and marks it dirty for the next update.
    pub fn register_emitter(&mut self, mut emitter: EmitterRecord) {
        emitter.dirty = true;
        self.active_emitters.push(emitter);
    }

    /// Replaces the full active emitter set.
    pub fn replace_emitters(&mut self, mut emitters: Vec<EmitterRecord>) {
        for emitter in &mut emitters {
            emitter.dirty = true;
        }
        self.active_emitters = emitters;
    }

    /// Clears all active emitters.
    pub fn clear_emitters(&mut self) {
        self.active_emitters.clear();
    }

    /// Removes all emitters that match the given tile and channel.
    pub fn remove_emitter(&mut self, x: u32, y: u32, channel: ChannelId) {
        self.active_emitters
            .retain(|emitter| !(emitter.x == x && emitter.y == y && emitter.channel == channel));
    }

    /// Stamps one emitter into the pending buffer.
    pub fn stamp(&mut self, emitter: &EmitterRecord) {
        if emitter.x >= self.width || emitter.y >= self.height {
            return;
        }
        let channel_index = emitter.channel.index();
        let meta = &self.channel_meta[channel_index];
        let radius = emitter
            .radius
            .max(meta.default_radius)
            .min(f64::from(meta.max_radius.max(1)));
        if radius <= 0.0 {
            return;
        }

        let radius_sq = radius * radius;
        let sweep_limit = radius.ceil() as i32;
        let center_x = emitter.x as i32;
        let center_y = emitter.y as i32;
        let source_decay = emitter.decay_rate.unwrap_or(meta.decay_rate).clamp(0.0, 1.0);
        let source_scale = (1.0 - source_decay).clamp(0.0, 1.0);

        for dy in -sweep_limit..=sweep_limit {
            for dx in -sweep_limit..=sweep_limit {
                let next_x = center_x + dx;
                let next_y = center_y + dy;
                if next_x < 0
                    || next_y < 0
                    || next_x >= self.width as i32
                    || next_y >= self.height as i32
                {
                    continue;
                }

                let dist_sq = (dx * dx + dy * dy) as f64;
                if dist_sq > radius_sq {
                    continue;
                }

                let dist = dist_sq.sqrt();
                let next_x_u32 = next_x as u32;
                let next_y_u32 = next_y as u32;
                let idx = self.index(next_x_u32, next_y_u32);
                let wall_factor = self.wall_factor_along_path(
                    emitter.x,
                    emitter.y,
                    next_x_u32,
                    next_y_u32,
                    channel_index,
                );
                if wall_factor <= 0.0 {
                    continue;
                }
                let raw_value =
                    raw_falloff_value(emitter.falloff, emitter.base_intensity, radius, dist_sq, dist)
                        * source_scale;

                self.pending[channel_index][idx] += raw_value * wall_factor;
            }
        }
    }

    /// Samples one channel value from the current buffer.
    #[inline]
    pub fn sample(&self, x: u32, y: u32, channel: ChannelId) -> f64 {
        if x >= self.width || y >= self.height {
            return 0.0;
        }
        self.current[channel.index()][self.index(x, y)]
    }

    /// Samples the local channel gradient using centered differences.
    #[inline]
    pub fn sample_gradient(&self, channel: ChannelId, x: u32, y: u32) -> (f64, f64) {
        let left = if x > 0 {
            self.sample(x - 1, y, channel)
        } else {
            self.sample(x, y, channel)
        };
        let right = if x + 1 < self.width {
            self.sample(x + 1, y, channel)
        } else {
            self.sample(x, y, channel)
        };
        let up = if y > 0 {
            self.sample(x, y - 1, channel)
        } else {
            self.sample(x, y, channel)
        };
        let down = if y + 1 < self.height {
            self.sample(x, y + 1, channel)
        } else {
            self.sample(x, y, channel)
        };
        ((right - left) * 0.5, (down - up) * 0.5)
    }

    /// Samples a weighted sum of multiple channels without allocating.
    #[inline]
    pub fn sample_weighted_sum(&self, x: u32, y: u32, channel_weights: &[(ChannelId, f64)]) -> f64 {
        channel_weights
            .iter()
            .map(|(channel, weight)| self.sample(x, y, *channel) * *weight)
            .sum()
    }

    /// Samples all channel values for one tile from the current buffer.
    pub fn sample_all(&self, x: u32, y: u32) -> Vec<(ChannelId, f64)> {
        if x >= self.width || y >= self.height {
            return ChannelId::all()
                .into_iter()
                .map(|channel| (channel, 0.0))
                .collect();
        }
        let idx = self.index(x, y);
        self.channel_meta
            .iter()
            .map(|meta| (meta.id, self.current[meta.id.index()][idx]))
            .collect()
    }

    /// Applies one full-grid update and swaps pending into current.
    pub fn tick_update(&mut self) {
        for channel in 0..self.channel_count {
            self.prepare_pending_channel(channel);
        }

        let dirty_emitters: Vec<EmitterRecord> = self
            .active_emitters
            .iter()
            .filter(|emitter| emitter.dirty)
            .cloned()
            .collect();
        for emitter in &dirty_emitters {
            self.stamp(emitter);
        }

        for emitter in &mut self.active_emitters {
            emitter.dirty = false;
        }

        for channel in 0..self.channel_count {
            self.normalize_and_clamp_channel(channel);
        }

        std::mem::swap(&mut self.current, &mut self.pending);
    }

    /// Applies one staggered channel update and advances the internal channel cursor.
    pub fn staggered_update(&mut self) {
        let channel = self.stagger_index % self.channel_count;
        self.prepare_pending_channel(channel);

        let target_channel = self.channel_meta[channel].id;
        let dirty_emitters: Vec<EmitterRecord> = self
            .active_emitters
            .iter()
            .filter(|emitter| emitter.dirty && emitter.channel == target_channel)
            .cloned()
            .collect();
        for emitter in &dirty_emitters {
            self.stamp(emitter);
        }
        for emitter in &mut self.active_emitters {
            if emitter.channel == target_channel {
                emitter.dirty = false;
            }
        }
        self.normalize_and_clamp_channel(channel);
        std::mem::swap(&mut self.current[channel], &mut self.pending[channel]);
        self.stagger_index = (self.stagger_index + 1) % self.channel_count;
    }

    /// Sets one tile's wall blocking coefficient.
    pub fn set_wall_blocking(&mut self, x: u32, y: u32, blocking: f64) {
        self.wall_blocking.set(x, y, blocking);
    }

    /// Clears all runtime wall blocking coefficients.
    pub fn clear_wall_blocking(&mut self) {
        self.wall_blocking.clear();
    }

    /// Returns the current wall blocking coefficient for one tile.
    pub fn wall_blocking_at(&self, x: u32, y: u32) -> f64 {
        self.wall_blocking.get(x, y)
    }

    /// Returns an immutable view over one channel's current buffer.
    pub fn get_channel_data(&self, channel: ChannelId) -> &[f64] {
        &self.current[channel.index()]
    }

    /// Returns the grid dimensions.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Returns the number of active emitters currently registered.
    pub fn active_emitter_count(&self) -> usize {
        self.active_emitters.len()
    }

    fn index(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    fn prepare_pending_channel(&mut self, channel: usize) {
        self.pending[channel].clone_from(&self.current[channel]);
        let decay = self.channel_meta[channel].decay_rate.clamp(0.0, 1.0);
        for value in &mut self.pending[channel] {
            *value *= 1.0 - decay;
        }
    }

    fn normalize_and_clamp_channel(&mut self, channel: usize) {
        let max_abs = self.pending[channel]
            .iter()
            .map(|value| value.abs())
            .fold(0.0_f64, f64::max);
        if max_abs > 1.0 + config::INFLUENCE_NORMALIZATION_EPSILON {
            let scale = 1.0 / max_abs;
            for value in &mut self.pending[channel] {
                *value *= scale;
            }
        }

        match self.channel_meta[channel].clamp_policy {
            ChannelClampPolicy::Sigmoid => {
                for value in &mut self.pending[channel] {
                    let raw = *value;
                    *value = raw / (1.0 + raw.abs());
                }
            }
            ChannelClampPolicy::UnitInterval => {
                for value in &mut self.pending[channel] {
                    *value = value.clamp(0.0, 1.0);
                }
            }
        }
    }

    fn wall_factor_along_path(
        &self,
        start_x: u32,
        start_y: u32,
        end_x: u32,
        end_y: u32,
        channel_index: usize,
    ) -> f64 {
        let wall_sensitivity = self.channel_meta[channel_index]
            .wall_blocking_sensitivity
            .clamp(0.0, 1.0);
        if wall_sensitivity <= 0.0 {
            return 1.0;
        }

        let mut x = start_x as i32;
        let mut y = start_y as i32;
        let end_x_i32 = end_x as i32;
        let end_y_i32 = end_y as i32;
        let dx = (end_x_i32 - x).abs();
        let sx = if x < end_x_i32 {
            1
        } else if x > end_x_i32 {
            -1
        } else {
            0
        };
        let dy = -(end_y_i32 - y).abs();
        let sy = if y < end_y_i32 {
            1
        } else if y > end_y_i32 {
            -1
        } else {
            0
        };
        let mut err = dx + dy;
        let mut factor = 1.0;

        loop {
            if !(x == start_x as i32 && y == start_y as i32) {
                let tile_block = self.wall_blocking.get(x as u32, y as u32);
                if tile_block > 0.0 {
                    let attenuation = 1.0 - (tile_block * wall_sensitivity).clamp(0.0, 1.0);
                    factor *= attenuation;
                    if factor <= 0.0 {
                        return 0.0;
                    }
                }
            }

            if x == end_x_i32 && y == end_y_i32 {
                break;
            }

            let double_err = err * 2;
            if double_err >= dy {
                err += dy;
                x += sx;
            }
            if double_err <= dx {
                err += dx;
                y += sy;
            }
        }

        factor.clamp(0.0, 1.0)
    }
}

fn merge_channel_meta(channels: Vec<ChannelMeta>) -> Vec<ChannelMeta> {
    let mut channel_meta = ChannelId::default_channels();
    for meta in channels {
        channel_meta[meta.id.index()] = meta.sanitized();
    }
    channel_meta
}

fn raw_falloff_value(
    falloff: FalloffType,
    base_intensity: f64,
    radius: f64,
    dist_sq: f64,
    dist: f64,
) -> f64 {
    match falloff {
        FalloffType::Linear => {
            if radius <= 0.0 {
                0.0
            } else {
                base_intensity * (1.0 - dist / radius).max(0.0)
            }
        }
        FalloffType::Exponential => base_intensity * (-(dist / radius.max(1.0))).exp(),
        FalloffType::Gaussian => {
            let sigma = (radius * 0.5).max(config::INFLUENCE_NORMALIZATION_EPSILON);
            base_intensity * (-(dist_sq) / (2.0 * sigma * sigma)).exp()
        }
        FalloffType::InverseSquare => base_intensity / (1.0 + dist_sq),
        FalloffType::Constant => base_intensity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_influence_grid(width: u32, height: u32) -> InfluenceGrid {
        InfluenceGrid::new(width, height, ChannelId::default_channels())
    }

    #[test]
    fn influence_grid_starts_empty() {
        let grid = make_influence_grid(16, 16);
        assert_eq!(grid.dimensions(), (16, 16));
        assert_eq!(grid.sample(4, 4, ChannelId::Warmth), 0.0);
        assert_eq!(grid.sample_all(4, 4).len(), ChannelId::count());
    }

    #[test]
    fn influence_grid_tick_stamps_and_samples_center_value() {
        let mut grid = make_influence_grid(256, 256);
        grid.register_emitter(EmitterRecord {
            x: 100,
            y: 100,
            channel: ChannelId::Warmth,
            radius: 6.0,
            base_intensity: 0.8,
            falloff: FalloffType::Constant,
            decay_rate: None,
            tags: Vec::new(),
            dirty: false,
        });

        grid.tick_update();

        let center = grid.sample(100, 100, ChannelId::Warmth);
        assert!(center > 0.0);
        assert_eq!(grid.sample(200, 200, ChannelId::Warmth), 0.0);
    }

    #[test]
    fn influence_grid_tick_decay_reduces_value_on_second_update() {
        let mut grid = make_influence_grid(64, 64);
        grid.register_emitter(EmitterRecord {
            x: 32,
            y: 32,
            channel: ChannelId::Warmth,
            radius: 8.0,
            base_intensity: 0.8,
            falloff: FalloffType::Constant,
            decay_rate: None,
            tags: Vec::new(),
            dirty: false,
        });

        grid.tick_update();
        let first = grid.sample(32, 32, ChannelId::Warmth);
        grid.remove_emitter(32, 32, ChannelId::Warmth);
        grid.tick_update();
        let second = grid.sample(32, 32, ChannelId::Warmth);
        assert!(second < first);
    }

    #[test]
    fn influence_grid_wall_blocking_reduces_values() {
        let mut grid = make_influence_grid(32, 32);
        grid.set_wall_blocking(17, 16, 0.9);
        grid.register_emitter(EmitterRecord {
            x: 16,
            y: 16,
            channel: ChannelId::Warmth,
            radius: 4.0,
            base_intensity: 0.1,
            falloff: FalloffType::Constant,
            decay_rate: None,
            tags: Vec::new(),
            dirty: false,
        });

        grid.tick_update();
        let blocked = grid.sample(17, 16, ChannelId::Warmth);

        let mut unblocked = make_influence_grid(32, 32);
        unblocked.register_emitter(EmitterRecord {
            x: 16,
            y: 16,
            channel: ChannelId::Warmth,
            radius: 4.0,
            base_intensity: 0.1,
            falloff: FalloffType::Constant,
            decay_rate: None,
            tags: Vec::new(),
            dirty: false,
        });
        unblocked.tick_update();
        let clear = unblocked.sample(17, 16, ChannelId::Warmth);

        assert!(blocked < clear);
    }

    #[test]
    fn influence_grid_gradient_points_toward_emitter() {
        let mut grid = make_influence_grid(24, 24);
        grid.register_emitter(EmitterRecord {
            x: 12,
            y: 12,
            channel: ChannelId::Food,
            radius: 6.0,
            base_intensity: 0.8,
            falloff: FalloffType::Gaussian,
            decay_rate: None,
            tags: Vec::new(),
            dirty: false,
        });
        grid.tick_update();

        let gradient = grid.sample_gradient(ChannelId::Food, 8, 12);
        assert!(gradient.0 > 0.0);
    }

    #[test]
    fn influence_grid_weighted_sum_combines_channels_without_allocation() {
        let mut grid = make_influence_grid(16, 16);
        grid.register_emitter(EmitterRecord {
            x: 8,
            y: 8,
            channel: ChannelId::Warmth,
            radius: 5.0,
            base_intensity: 0.9,
            falloff: FalloffType::Constant,
            decay_rate: None,
            tags: Vec::new(),
            dirty: false,
        });
        grid.register_emitter(EmitterRecord {
            x: 4,
            y: 8,
            channel: ChannelId::Danger,
            radius: 5.0,
            base_intensity: 0.6,
            falloff: FalloffType::Exponential,
            decay_rate: None,
            tags: Vec::new(),
            dirty: false,
        });
        grid.tick_update();

        let weights = [(ChannelId::Warmth, 1.0), (ChannelId::Danger, -1.0)];
        let score = grid.sample_weighted_sum(8, 8, &weights);
        assert!(score > 0.0);
    }

    #[test]
    fn influence_grid_staggered_update_advances_one_channel() {
        let mut grid = make_influence_grid(16, 16);
        grid.register_emitter(EmitterRecord {
            x: 8,
            y: 8,
            channel: ChannelId::Warmth,
            radius: 5.0,
            base_intensity: 0.8,
            falloff: FalloffType::Constant,
            decay_rate: None,
            tags: Vec::new(),
            dirty: false,
        });
        grid.register_emitter(EmitterRecord {
            x: 8,
            y: 8,
            channel: ChannelId::Light,
            radius: 5.0,
            base_intensity: 0.8,
            falloff: FalloffType::Constant,
            decay_rate: None,
            tags: Vec::new(),
            dirty: false,
        });

        grid.staggered_update();
        let first_food = grid.sample(8, 8, ChannelId::Food);
        let first_warmth = grid.sample(8, 8, ChannelId::Warmth);
        let first_light = grid.sample(8, 8, ChannelId::Light);

        assert_eq!(first_food, 0.0);
        assert!(first_warmth >= 0.0);
        assert_eq!(first_light, 0.0);

        for _ in 0..ChannelId::count() {
            grid.staggered_update();
        }
        assert!(grid.sample(8, 8, ChannelId::Light) > 0.0);
    }
}
