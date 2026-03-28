use std::collections::HashMap;

/// Faction identifier for territory ownership.
/// Maps to settlement_id+1 (for settlements) or band_id+1000 (for bands).
pub type FactionId = u16;

/// Per-faction territory influence grid.
/// Each faction has a width×height f32 channel representing territorial influence.
#[derive(Debug, Clone)]
pub struct TerritoryGrid {
    /// Grid width in tiles.
    pub width: u32,
    /// Grid height in tiles.
    pub height: u32,
    /// faction_id → flat array of f32 influence values (width × height).
    channels: HashMap<FactionId, Vec<f32>>,
}

impl TerritoryGrid {
    /// Create a new empty territory grid.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            channels: HashMap::new(),
        }
    }

    /// Ensure a faction channel exists, creating it if needed.
    pub fn ensure_faction(&mut self, faction_id: FactionId) {
        let cell_count = (self.width * self.height) as usize;
        self.channels
            .entry(faction_id)
            .or_insert_with(|| vec![0.0_f32; cell_count]);
    }

    /// Get immutable reference to a faction's territory data.
    pub fn get(&self, faction_id: FactionId) -> Option<&Vec<f32>> {
        self.channels.get(&faction_id)
    }

    /// Stamp Gaussian influence at (cx, cy) for a faction.
    /// Values are additive and clamped to 1.0.
    pub fn stamp_gaussian(
        &mut self,
        faction_id: FactionId,
        cx: u32,
        cy: u32,
        intensity: f32,
        radius: f32,
    ) {
        self.ensure_faction(faction_id);
        let w = self.width as i32;
        let h = self.height as i32;
        let data = self.channels.get_mut(&faction_id).unwrap_or_else(|| unreachable!());
        let sigma = radius / 2.5;
        let sigma_sq_2 = 2.0 * sigma * sigma;
        let search = radius.ceil() as i32;

        for dy in -search..=search {
            let ty = cy as i32 + dy;
            if ty < 0 || ty >= h {
                continue;
            }
            for dx in -search..=search {
                let tx = cx as i32 + dx;
                if tx < 0 || tx >= w {
                    continue;
                }
                let dist_sq = (dx * dx + dy * dy) as f32;
                if dist_sq > radius * radius {
                    continue;
                }
                let value = intensity * (-dist_sq / sigma_sq_2).exp();
                let idx = ty as usize * self.width as usize + tx as usize;
                data[idx] = (data[idx] + value).min(1.0);
            }
        }
    }

    /// Apply decay to all factions. Values below min_threshold are zeroed.
    pub fn decay_all(&mut self, decay_rate: f32, min_threshold: f32) {
        for data in self.channels.values_mut() {
            for cell in data.iter_mut() {
                *cell *= decay_rate;
                if *cell < min_threshold {
                    *cell = 0.0;
                }
            }
        }
    }

    /// Export a faction's territory as u8 bytes (0–255 normalized).
    pub fn export_u8(&self, faction_id: FactionId) -> Vec<u8> {
        let cell_count = (self.width * self.height) as usize;
        match self.channels.get(&faction_id) {
            Some(data) => data
                .iter()
                .map(|&val| (val.clamp(0.0, 1.0) * 255.0) as u8)
                .collect(),
            None => vec![0_u8; cell_count],
        }
    }

    /// Export a combined "dominant faction" texture.
    /// Returns (faction_id_map, max_density_map) both as `Vec<u8>`.
    pub fn export_dominant(&self) -> (Vec<u8>, Vec<u8>) {
        let cell_count = (self.width * self.height) as usize;
        let mut faction_map = vec![0_u8; cell_count];
        let mut density_map = vec![0_u8; cell_count];

        for (&faction_id, data) in &self.channels {
            for (i, &val) in data.iter().enumerate() {
                let quantized = (val.clamp(0.0, 1.0) * 255.0) as u8;
                if quantized > density_map[i] {
                    density_map[i] = quantized;
                    faction_map[i] = faction_id as u8;
                }
            }
        }

        (faction_map, density_map)
    }

    /// Get all active faction IDs.
    pub fn active_factions(&self) -> Vec<FactionId> {
        self.channels.keys().copied().collect()
    }

    /// Remove a faction's channel entirely.
    pub fn remove_faction(&mut self, faction_id: FactionId) {
        self.channels.remove(&faction_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid_is_empty() {
        let grid = TerritoryGrid::new(16, 16);
        assert!(grid.active_factions().is_empty());
    }

    #[test]
    fn stamp_gaussian_creates_faction() {
        let mut grid = TerritoryGrid::new(16, 16);
        grid.stamp_gaussian(1, 8, 8, 0.5, 4.0);
        assert!(grid.get(1).is_some());
        let data = grid.get(1).unwrap();
        assert!(data[8 * 16 + 8] > 0.0);
    }

    #[test]
    fn decay_reduces_values() {
        let mut grid = TerritoryGrid::new(16, 16);
        grid.stamp_gaussian(1, 8, 8, 1.0, 4.0);
        let before = grid.get(1).unwrap()[8 * 16 + 8];
        grid.decay_all(0.5, 0.001);
        let after = grid.get(1).unwrap()[8 * 16 + 8];
        assert!(after < before);
    }

    #[test]
    fn export_dominant_picks_strongest() {
        let mut grid = TerritoryGrid::new(4, 4);
        grid.stamp_gaussian(1, 2, 2, 0.3, 2.0);
        grid.stamp_gaussian(2, 2, 2, 0.8, 2.0);
        let (fmap, dmap) = grid.export_dominant();
        assert_eq!(fmap[2 * 4 + 2], 2);
        assert!(dmap[2 * 4 + 2] > 0);
    }

    #[test]
    fn decay_zeros_below_threshold() {
        let mut grid = TerritoryGrid::new(4, 4);
        grid.stamp_gaussian(1, 2, 2, 0.01, 1.0);
        grid.decay_all(0.1, 0.005);
        let data = grid.get(1).unwrap();
        assert_eq!(data[2 * 4 + 2], 0.0);
    }
}
