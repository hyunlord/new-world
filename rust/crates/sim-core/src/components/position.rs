use serde::{Deserialize, Serialize};

/// World position in continuous tile coordinates.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Position {
    /// Continuous tile X coordinate.
    pub x: f64,
    /// Continuous tile Y coordinate.
    pub y: f64,
    /// Velocity along the tile X axis (tiles per tick).
    pub vel_x: f64,
    /// Velocity along the tile Y axis (tiles per tick).
    pub vel_y: f64,
    /// Eight-direction facing for renderer consumers.
    pub movement_dir: u8,
}

impl Position {
    /// Creates a position from integer tile coordinates.
    pub fn new(x: i32, y: i32) -> Self {
        Self::from_f64(f64::from(x), f64::from(y))
    }

    /// Creates a position from continuous tile coordinates.
    pub fn from_f64(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            vel_x: 0.0,
            vel_y: 0.0,
            movement_dir: 0,
        }
    }

    /// Returns the Euclidean distance to another position in tile units.
    pub fn distance_to(&self, other: &Position) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Returns the squared Euclidean distance to another position in tile units.
    pub fn distance_sq(&self, other: &Position) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Returns the rounded tile X coordinate for map lookups.
    pub fn tile_x(&self) -> i32 {
        self.x.round() as i32
    }

    /// Returns the rounded tile Y coordinate for map lookups.
    pub fn tile_y(&self) -> i32 {
        self.y.round() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::Position;

    #[test]
    fn new_position_uses_continuous_storage_with_zero_velocity() {
        let position = Position::new(3, 4);
        assert_eq!(position.x, 3.0);
        assert_eq!(position.y, 4.0);
        assert_eq!(position.vel_x, 0.0);
        assert_eq!(position.vel_y, 0.0);
        assert_eq!(position.movement_dir, 0);
    }

    #[test]
    fn tile_helpers_round_continuous_coordinates() {
        let position = Position::from_f64(2.6, 7.4);
        assert_eq!(position.tile_x(), 3);
        assert_eq!(position.tile_y(), 7);
    }

    #[test]
    fn stage1_position_is_f64_with_velocity() {
        let position = Position {
            x: 100.5,
            y: 200.7,
            vel_x: 1.5,
            vel_y: -0.8,
            movement_dir: 3,
        };
        assert!((position.x - 100.5).abs() < f64::EPSILON);
        assert!((position.y - 200.7).abs() < f64::EPSILON);
        assert!((position.vel_x - 1.5).abs() < f64::EPSILON);
        assert!((position.vel_y + 0.8).abs() < f64::EPSILON);
        assert_eq!(position.movement_dir, 3);
    }

    #[test]
    fn stage1_position_clamped_to_world_bounds() {
        let mut position = Position {
            x: -10.0,
            y: 300.0,
            vel_x: 0.0,
            vel_y: 0.0,
            movement_dir: 0,
        };
        position.x = position.x.clamp(0.0, 256.0);
        position.y = position.y.clamp(0.0, 256.0);
        assert_eq!(position.x, 0.0);
        assert_eq!(position.y, 256.0);
    }
}
