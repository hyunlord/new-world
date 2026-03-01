//! sim-bridge: Boundary adapters between external callers and simulation crates.
//!
//! Phase R-3 will expose this through Godot GDExtension.
//! For now, this module provides pure-Rust conversion helpers that can be
//! reused by the future FFI layer.

use godot::prelude::*;
use sim_systems::{
    pathfinding::{find_path, GridCostMap, GridPos},
    stat_curve,
};

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
            width, height, walkable, move_cost, from_x, from_y, to_x, to_y, max_steps,
        )?;
        out.push(path);
    }
    Ok(out)
}

fn packed_i32_to_vec(values: &PackedInt32Array) -> Vec<i32> {
    values.as_slice().to_vec()
}

fn packed_f32_to_vec(values: &PackedFloat32Array) -> Vec<f32> {
    values.as_slice().to_vec()
}

fn build_step_pairs(
    thresholds: &PackedInt32Array,
    multipliers: &PackedFloat32Array,
) -> Vec<(i32, f32)> {
    let len = thresholds.len().min(multipliers.len());
    let mut pairs: Vec<(i32, f32)> = Vec::with_capacity(len);
    for idx in 0..len {
        pairs.push((thresholds[idx], multipliers[idx]));
    }
    pairs
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
    fn has_gpu_pathfinding(&self) -> bool {
        cfg!(feature = "gpu")
    }

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
            width, height, walkable, move_cost, from_x, from_y, to_x, to_y, max_steps,
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
        let steps = if max_steps <= 0 {
            200_usize
        } else {
            max_steps as usize
        };

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

    #[func]
    fn stat_log_xp_required(
        &self,
        level: i32,
        base_xp: f32,
        exponent: f32,
        level_breakpoints: PackedInt32Array,
        breakpoint_multipliers: PackedFloat32Array,
    ) -> f32 {
        let breakpoints = packed_i32_to_vec(&level_breakpoints);
        let multipliers = packed_f32_to_vec(&breakpoint_multipliers);
        stat_curve::log_xp_required(level, base_xp, exponent, &breakpoints, &multipliers)
    }

    #[func]
    fn stat_xp_to_level(
        &self,
        xp: f32,
        base_xp: f32,
        exponent: f32,
        level_breakpoints: PackedInt32Array,
        breakpoint_multipliers: PackedFloat32Array,
        max_level: i32,
    ) -> i32 {
        let breakpoints = packed_i32_to_vec(&level_breakpoints);
        let multipliers = packed_f32_to_vec(&breakpoint_multipliers);
        stat_curve::xp_to_level(xp, base_xp, exponent, &breakpoints, &multipliers, max_level)
    }

    #[func]
    fn stat_skill_xp_progress(
        &self,
        level: i32,
        xp: f32,
        base_xp: f32,
        exponent: f32,
        level_breakpoints: PackedInt32Array,
        breakpoint_multipliers: PackedFloat32Array,
        max_level: i32,
    ) -> VarDictionary {
        let breakpoints = packed_i32_to_vec(&level_breakpoints);
        let multipliers = packed_f32_to_vec(&breakpoint_multipliers);
        let clamped_max = max_level.max(0);
        let clamped_level = level.clamp(0, clamped_max);

        let mut xp_at_level = 0.0_f32;
        for lv in 1..=clamped_level {
            xp_at_level +=
                stat_curve::log_xp_required(lv, base_xp, exponent, &breakpoints, &multipliers);
        }

        let xp_to_next = if clamped_level < clamped_max {
            stat_curve::log_xp_required(
                clamped_level + 1,
                base_xp,
                exponent,
                &breakpoints,
                &multipliers,
            )
        } else {
            0.0_f32
        };

        let progress = xp - xp_at_level;

        let mut dict = VarDictionary::new();
        dict.set("level", clamped_level);
        dict.set("max_level", clamped_max);
        dict.set("xp_at_level", xp_at_level as f64);
        dict.set("xp_to_next", xp_to_next as f64);
        dict.set("progress_in_level", progress as f64);
        dict
    }

    #[func]
    fn stat_scurve_speed(
        &self,
        current_value: i32,
        phase_breakpoints: PackedInt32Array,
        phase_speeds: PackedFloat32Array,
    ) -> f32 {
        let breakpoints = packed_i32_to_vec(&phase_breakpoints);
        let speeds = packed_f32_to_vec(&phase_speeds);
        stat_curve::scurve_speed(current_value, &breakpoints, &speeds)
    }

    #[func]
    fn stat_need_decay(
        &self,
        current: i32,
        decay_per_year: i32,
        ticks_elapsed: i32,
        metabolic_mult: f32,
        ticks_per_year: i32,
    ) -> i32 {
        stat_curve::need_decay(
            current,
            decay_per_year,
            ticks_elapsed,
            metabolic_mult,
            ticks_per_year,
        )
    }

    #[func]
    fn stat_sigmoid_extreme(
        &self,
        value: i32,
        flat_zone_lo: i32,
        flat_zone_hi: i32,
        pole_multiplier: f32,
    ) -> f32 {
        stat_curve::sigmoid_extreme(value, flat_zone_lo, flat_zone_hi, pole_multiplier)
    }

    #[func]
    fn stat_power_influence(&self, value: i32, exponent: f32) -> f32 {
        stat_curve::power_influence(value, exponent)
    }

    #[func]
    fn stat_threshold_power(
        &self,
        value: i32,
        threshold: i32,
        exponent: f32,
        max_output: f32,
    ) -> f32 {
        stat_curve::threshold_power(value, threshold, exponent, max_output)
    }

    #[func]
    fn stat_linear_influence(&self, value: i32) -> f32 {
        stat_curve::linear_influence(value)
    }

    #[func]
    fn stat_step_influence(
        &self,
        value: i32,
        threshold: i32,
        above_value: f32,
        below_value: f32,
    ) -> f32 {
        stat_curve::step_influence(value, threshold, above_value, below_value)
    }

    #[func]
    fn stat_step_linear(
        &self,
        value: i32,
        below_thresholds: PackedInt32Array,
        multipliers: PackedFloat32Array,
    ) -> f32 {
        let step_pairs = build_step_pairs(&below_thresholds, &multipliers);
        stat_curve::step_linear(value, &step_pairs)
    }
}

struct SimBridgeExtension;

#[gdextension(entry_symbol = worldsim_rust_init)]
unsafe impl ExtensionLibrary for SimBridgeExtension {}

#[cfg(test)]
mod tests {
    use super::{
        pathfind_from_flat, pathfind_grid_batch_bytes, pathfind_grid_bytes, PathfindError,
        PathfindInput,
    };
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
