//! sim-bridge: Boundary adapters between external callers and simulation crates.
//!
//! Phase R-3 will expose this through Godot GDExtension.
//! For now, this module provides pure-Rust conversion helpers that can be
//! reused by the future FFI layer.

use sim_systems::pathfinding::{find_path, GridCostMap, GridPos};
use godot::prelude::*;

/// Flat-grid input for pathfinding requests crossing the bridge boundary.
///
/// - `walkable` and `move_cost` must be exactly `width * height` in length.
#[derive(Debug, Clone)]
pub struct PathfindInput {
    pub width: i32,
    pub height: i32,
    pub walkable: Vec<bool>,
    pub move_cost: Vec<f32>,
    pub from: GridPos,
    pub to: GridPos,
    pub max_steps: usize,
}

impl PathfindInput {
    fn expected_len(&self) -> usize {
        (self.width * self.height) as usize
    }

    fn validate(&self) -> Result<(), PathfindError> {
        if self.width <= 0 || self.height <= 0 {
            return Err(PathfindError::InvalidDimensions {
                width: self.width,
                height: self.height,
            });
        }

        let expected = self.expected_len();
        if self.walkable.len() != expected {
            return Err(PathfindError::InvalidWalkableLength {
                expected,
                got: self.walkable.len(),
            });
        }
        if self.move_cost.len() != expected {
            return Err(PathfindError::InvalidMoveCostLength {
                expected,
                got: self.move_cost.len(),
            });
        }

        Ok(())
    }
}

/// Bridge-level errors for pathfinding request validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathfindError {
    InvalidDimensions { width: i32, height: i32 },
    InvalidWalkableLength { expected: usize, got: usize },
    InvalidMoveCostLength { expected: usize, got: usize },
    MismatchedBatchLength { from_len: usize, to_len: usize },
}

/// Executes pathfinding from bridge-friendly flat buffers.
pub fn pathfind_from_flat(input: PathfindInput) -> Result<Vec<GridPos>, PathfindError> {
    input.validate()?;

    let mut grid = GridCostMap::new(input.width, input.height);
    for y in 0..input.height {
        for x in 0..input.width {
            let idx = (y * input.width + x) as usize;
            grid.set_walkable(x, y, input.walkable[idx]);
            grid.set_move_cost(x, y, input.move_cost[idx]);
        }
    }

    Ok(find_path(&grid, input.from, input.to, input.max_steps))
}

/// Pathfinding entry shape intended for future Godot bridge exposure.
///
/// `walkable` uses byte flags where 0 = blocked, non-zero = walkable.
pub fn pathfind_grid_bytes(
    width: i32,
    height: i32,
    walkable: &[u8],
    move_cost: &[f32],
    from_x: i32,
    from_y: i32,
    to_x: i32,
    to_y: i32,
    max_steps: usize,
) -> Result<Vec<GridPos>, PathfindError> {
    let expected = (width * height) as usize;
    if walkable.len() != expected {
        return Err(PathfindError::InvalidWalkableLength {
            expected,
            got: walkable.len(),
        });
    }
    if move_cost.len() != expected {
        return Err(PathfindError::InvalidMoveCostLength {
            expected,
            got: move_cost.len(),
        });
    }

    let input = PathfindInput {
        width,
        height,
        walkable: walkable.iter().map(|v| *v != 0).collect(),
        move_cost: move_cost.to_vec(),
        from: GridPos::new(from_x, from_y),
        to: GridPos::new(to_x, to_y),
        max_steps,
    };
    pathfind_from_flat(input)
}

pub fn pathfind_grid_batch_bytes(
    width: i32,
    height: i32,
    walkable: &[u8],
    move_cost: &[f32],
    from_points: &[(i32, i32)],
    to_points: &[(i32, i32)],
    max_steps: usize,
) -> Result<Vec<Vec<GridPos>>, PathfindError> {
    if from_points.len() != to_points.len() {
        return Err(PathfindError::MismatchedBatchLength {
            from_len: from_points.len(),
            to_len: to_points.len(),
        });
    }

    let mut out = Vec::with_capacity(from_points.len());
    for idx in 0..from_points.len() {
        let (from_x, from_y) = from_points[idx];
        let (to_x, to_y) = to_points[idx];
        let path = pathfind_grid_bytes(
            width,
            height,
            walkable,
            move_cost,
            from_x,
            from_y,
            to_x,
            to_y,
            max_steps,
        )?;
        out.push(path);
    }
    Ok(out)
}

#[derive(GodotClass)]
#[class(base=Object, singleton)]
pub struct WorldSimBridge {
    base: Base<Object>,
}

#[godot_api]
impl IObject for WorldSimBridge {
    fn init(base: Base<Object>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl WorldSimBridge {
    #[func]
    fn pathfind_grid(
        &self,
        width: i32,
        height: i32,
        walkable: PackedByteArray,
        move_cost: PackedFloat32Array,
        from_x: i32,
        from_y: i32,
        to_x: i32,
        to_y: i32,
        max_steps: i32,
    ) -> PackedVector2Array {
        let steps = if max_steps <= 0 {
            200_usize
        } else {
            max_steps as usize
        };

        let path = match pathfind_grid_bytes(
            width,
            height,
            walkable.as_slice(),
            move_cost.as_slice(),
            from_x,
            from_y,
            to_x,
            to_y,
            steps,
        ) {
            Ok(path) => path,
            Err(_) => return PackedVector2Array::new(),
        };

        let points: Vec<Vector2> = path
            .into_iter()
            .map(|p| Vector2::new(p.x as f32, p.y as f32))
            .collect();
        PackedVector2Array::from(points)
    }

    #[func]
    fn pathfind_grid_gpu(
        &self,
        width: i32,
        height: i32,
        walkable: PackedByteArray,
        move_cost: PackedFloat32Array,
        from_x: i32,
        from_y: i32,
        to_x: i32,
        to_y: i32,
        max_steps: i32,
    ) -> PackedVector2Array {
        // GPU path is not implemented yet; use CPU pathfinding as fallback.
        self.pathfind_grid(
            width,
            height,
            walkable,
            move_cost,
            from_x,
            from_y,
            to_x,
            to_y,
            max_steps,
        )
    }

    #[func]
    fn pathfind_grid_batch(
        &self,
        width: i32,
        height: i32,
        walkable: PackedByteArray,
        move_cost: PackedFloat32Array,
        from_points: PackedVector2Array,
        to_points: PackedVector2Array,
        max_steps: i32,
    ) -> Array<PackedVector2Array> {
        let steps = if max_steps <= 0 { 200_usize } else { max_steps as usize };

        let from_pairs: Vec<(i32, i32)> = from_points
            .as_slice()
            .iter()
            .map(|p| (p.x.round() as i32, p.y.round() as i32))
            .collect();
        let to_pairs: Vec<(i32, i32)> = to_points
            .as_slice()
            .iter()
            .map(|p| (p.x.round() as i32, p.y.round() as i32))
            .collect();

        let path_groups = match pathfind_grid_batch_bytes(
            width,
            height,
            walkable.as_slice(),
            move_cost.as_slice(),
            &from_pairs,
            &to_pairs,
            steps,
        ) {
            Ok(groups) => groups,
            Err(_) => return Array::new(),
        };

        let mut output: Array<PackedVector2Array> = Array::new();
        for group in path_groups {
            let points: Vec<Vector2> = group
                .into_iter()
                .map(|p| Vector2::new(p.x as f32, p.y as f32))
                .collect();
            let packed = PackedVector2Array::from(points);
            output.push(&packed);
        }
        output
    }

    #[func]
    fn pathfind_grid_gpu_batch(
        &self,
        width: i32,
        height: i32,
        walkable: PackedByteArray,
        move_cost: PackedFloat32Array,
        from_points: PackedVector2Array,
        to_points: PackedVector2Array,
        max_steps: i32,
    ) -> Array<PackedVector2Array> {
        // GPU path is not implemented yet; use CPU batch pathfinding as fallback.
        self.pathfind_grid_batch(
            width,
            height,
            walkable,
            move_cost,
            from_points,
            to_points,
            max_steps,
        )
    }
}

struct SimBridgeExtension;

#[gdextension(entry_symbol = worldsim_rust_init)]
unsafe impl ExtensionLibrary for SimBridgeExtension {}

#[cfg(test)]
mod tests {
    use super::{pathfind_from_flat, pathfind_grid_batch_bytes, pathfind_grid_bytes, PathfindError, PathfindInput};
    use sim_systems::pathfinding::GridPos;

    fn base_input() -> PathfindInput {
        PathfindInput {
            width: 4,
            height: 4,
            walkable: vec![true; 16],
            move_cost: vec![1.0; 16],
            from: GridPos::new(0, 0),
            to: GridPos::new(3, 3),
            max_steps: 200,
        }
    }

    #[test]
    fn validates_walkable_length() {
        let mut input = base_input();
        input.walkable.pop();
        let err = pathfind_from_flat(input).unwrap_err();
        assert_eq!(
            err,
            PathfindError::InvalidWalkableLength {
                expected: 16,
                got: 15
            }
        );
    }

    #[test]
    fn validates_move_cost_length() {
        let mut input = base_input();
        input.move_cost.pop();
        let err = pathfind_from_flat(input).unwrap_err();
        assert_eq!(
            err,
            PathfindError::InvalidMoveCostLength {
                expected: 16,
                got: 15
            }
        );
    }

    #[test]
    fn returns_path_on_valid_input() {
        let input = base_input();
        let path = pathfind_from_flat(input).expect("pathfinding should succeed");
        assert_eq!(path.first().copied(), Some(GridPos::new(0, 0)));
        assert_eq!(path.last().copied(), Some(GridPos::new(3, 3)));
    }

    #[test]
    fn pathfind_grid_accepts_byte_walkable_flags() {
        let walkable = vec![1_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let path = pathfind_grid_bytes(4, 4, &walkable, &move_cost, 0, 0, 3, 3, 200)
            .expect("pathfinding should succeed");
        assert_eq!(path.first().copied(), Some(GridPos::new(0, 0)));
        assert_eq!(path.last().copied(), Some(GridPos::new(3, 3)));
    }

    #[test]
    fn pathfind_grid_batch_processes_multiple_queries() {
        let walkable = vec![1_u8; 25];
        let move_cost = vec![1.0_f32; 25];
        let from = vec![(0, 0), (4, 0), (0, 4)];
        let to = vec![(4, 4), (0, 4), (4, 0)];

        let groups = pathfind_grid_batch_bytes(5, 5, &walkable, &move_cost, &from, &to, 200)
            .expect("batch pathfinding should succeed");
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].first().copied(), Some(GridPos::new(0, 0)));
        assert_eq!(groups[0].last().copied(), Some(GridPos::new(4, 4)));
    }

    #[test]
    fn pathfind_grid_batch_rejects_mismatched_input_lengths() {
        let walkable = vec![1_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let from = vec![(0, 0), (1, 1)];
        let to = vec![(3, 3)];

        let err = pathfind_grid_batch_bytes(4, 4, &walkable, &move_cost, &from, &to, 200)
            .expect_err("mismatched input lengths must fail");
        assert_eq!(
            err,
            PathfindError::MismatchedBatchLength {
                from_len: 2,
                to_len: 1
            }
        );
    }
}
