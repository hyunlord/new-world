use godot::prelude::Vector2;
use sim_systems::pathfinding::GridPos;

use crate::PathfindError;

pub(crate) fn pathfind_grid_gpu_bytes(
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
    // Placeholder GPU entrypoint: keep behavior stable until compute path is integrated.
    crate::pathfind_grid_bytes(
        width, height, walkable, move_cost, from_x, from_y, to_x, to_y, max_steps,
    )
}

pub(crate) fn pathfind_grid_batch_vec2_gpu_bytes(
    width: i32,
    height: i32,
    walkable: &[u8],
    move_cost: &[f32],
    from_points: &[Vector2],
    to_points: &[Vector2],
    max_steps: usize,
) -> Result<Vec<Vec<GridPos>>, PathfindError> {
    // Placeholder GPU entrypoint: keep behavior stable until compute path is integrated.
    crate::pathfind_grid_batch_vec2_bytes(
        width,
        height,
        walkable,
        move_cost,
        from_points,
        to_points,
        max_steps,
    )
}

pub(crate) fn pathfind_grid_batch_xy_gpu_bytes(
    width: i32,
    height: i32,
    walkable: &[u8],
    move_cost: &[f32],
    from_xy: &[i32],
    to_xy: &[i32],
    max_steps: usize,
) -> Result<Vec<Vec<GridPos>>, PathfindError> {
    // Placeholder GPU entrypoint: keep behavior stable until compute path is integrated.
    crate::pathfind_grid_batch_xy_bytes(width, height, walkable, move_cost, from_xy, to_xy, max_steps)
}
