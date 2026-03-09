/// Per-tile wall blocking coefficients used by the influence grid.
#[derive(Debug, Clone)]
pub struct WallBlockingMask {
    width: u32,
    height: u32,
    data: Vec<f64>,
}

impl WallBlockingMask {
    /// Creates a new wall mask initialized to no blocking.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0.0; (width * height) as usize],
        }
    }

    /// Returns the configured blocking coefficient for one tile.
    pub fn get(&self, x: u32, y: u32) -> f64 {
        if x >= self.width || y >= self.height {
            return 0.0;
        }
        self.data[(y * self.width + x) as usize]
    }

    /// Sets the blocking coefficient for one tile.
    pub fn set(&mut self, x: u32, y: u32, blocking: f64) {
        if x >= self.width || y >= self.height {
            return;
        }
        self.data[(y * self.width + x) as usize] = blocking.clamp(0.0, 1.0);
    }

    /// Derives a wall-blocking coefficient from material density.
    pub fn set_from_material_density(&mut self, x: u32, y: u32, density: f64) {
        let blocking = (density * 0.15).clamp(0.0, 1.0);
        self.set(x, y, blocking);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_mask_clamps_values_and_bounds() {
        let mut mask = WallBlockingMask::new(4, 4);
        mask.set(1, 1, 2.0);
        mask.set(3, 3, -1.0);
        mask.set(9, 9, 0.5);
        assert!((mask.get(1, 1) - 1.0).abs() < f64::EPSILON);
        assert!((mask.get(3, 3) - 0.0).abs() < f64::EPSILON);
        assert!((mask.get(9, 9) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn wall_mask_density_conversion_matches_influence_expectations() {
        let mut mask = WallBlockingMask::new(2, 2);
        mask.set_from_material_density(0, 0, 6.0);
        mask.set_from_material_density(1, 0, 3.3);
        mask.set_from_material_density(0, 1, 1.34);
        assert!((mask.get(0, 0) - 0.9).abs() < 1e-6);
        assert!((mask.get(1, 0) - 0.495).abs() < 1e-6);
        assert!((mask.get(0, 1) - 0.201).abs() < 1e-6);
    }
}
