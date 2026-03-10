use godot::prelude::Vector2;
use sim_systems::pathfinding::{
    find_path, find_path_with_workspace, GridCostMap, GridPos, PathfindWorkspace,
};
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};

pub const PATHFIND_BACKEND_AUTO: u8 = 0;
pub const PATHFIND_BACKEND_CPU: u8 = 1;
pub const PATHFIND_BACKEND_GPU: u8 = 2;
const GPU_BACKEND_ACTIVE: bool = false;

static PATHFIND_BACKEND_MODE: AtomicU8 = AtomicU8::new(PATHFIND_BACKEND_AUTO);
static CPU_DISPATCH_COUNT: AtomicU64 = AtomicU64::new(0);
static GPU_DISPATCH_COUNT: AtomicU64 = AtomicU64::new(0);

#[inline]
pub(crate) fn set_backend_mode(mode: u8) {
    PATHFIND_BACKEND_MODE.store(mode, Ordering::Relaxed);
}

#[inline]
pub(crate) fn get_backend_mode() -> u8 {
    PATHFIND_BACKEND_MODE.load(Ordering::Relaxed)
}

#[inline]
pub(crate) fn parse_backend_mode(mode: &str) -> Option<u8> {
    match mode.to_ascii_lowercase().as_str() {
        "auto" => Some(PATHFIND_BACKEND_AUTO),
        "cpu" => Some(PATHFIND_BACKEND_CPU),
        "gpu" => Some(PATHFIND_BACKEND_GPU),
        _ => None,
    }
}

#[inline]
pub(crate) fn backend_mode_to_str_core(mode: u8) -> &'static str {
    match mode {
        PATHFIND_BACKEND_CPU => "cpu",
        PATHFIND_BACKEND_GPU => "gpu",
        _ => "auto",
    }
}

#[inline]
pub(crate) fn resolve_backend_mode_code_core(mode: u8) -> u8 {
    match mode {
        PATHFIND_BACKEND_CPU => PATHFIND_BACKEND_CPU,
        PATHFIND_BACKEND_GPU => {
            if cfg!(feature = "gpu") && GPU_BACKEND_ACTIVE {
                PATHFIND_BACKEND_GPU
            } else {
                PATHFIND_BACKEND_CPU
            }
        }
        _ => PATHFIND_BACKEND_CPU,
    }
}

#[inline]
pub(crate) fn resolve_backend_mode_str_core(mode: u8) -> &'static str {
    match resolve_backend_mode_code_core(mode) {
        PATHFIND_BACKEND_GPU => "gpu",
        _ => "cpu",
    }
}

#[inline]
pub(crate) fn has_gpu_backend() -> bool {
    cfg!(feature = "gpu") && GPU_BACKEND_ACTIVE
}

#[inline]
pub(crate) fn record_dispatch(resolved_mode: u8) {
    match resolved_mode {
        PATHFIND_BACKEND_GPU => {
            GPU_DISPATCH_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        _ => {
            CPU_DISPATCH_COUNT.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[inline]
pub(crate) fn read_dispatch_counts() -> (u64, u64) {
    (
        CPU_DISPATCH_COUNT.load(Ordering::Relaxed),
        GPU_DISPATCH_COUNT.load(Ordering::Relaxed),
    )
}

#[inline]
pub(crate) fn reset_dispatch_counts() {
    CPU_DISPATCH_COUNT.store(0, Ordering::Relaxed);
    GPU_DISPATCH_COUNT.store(0, Ordering::Relaxed);
}

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
    let PathfindInput {
        width,
        height,
        walkable,
        move_cost,
        from,
        to,
        max_steps,
    } = input;

    let grid = GridCostMap::from_flat_owned_unchecked(width, height, walkable, move_cost);
    Ok(find_path(&grid, from, to, max_steps))
}

/// Returns accumulated resolved backend dispatch counters `(cpu, gpu)`.
pub fn pathfind_backend_dispatch_counts() -> (u64, u64) {
    read_dispatch_counts()
}

/// Resets accumulated resolved backend dispatch counters.
pub fn reset_pathfind_backend_dispatch_counts() {
    reset_dispatch_counts();
}

/// Sets configured pathfinding backend mode from string (`auto`, `cpu`, `gpu`).
pub fn set_pathfind_backend_mode(mode: &str) -> bool {
    let Some(parsed) = parse_backend_mode(mode) else {
        return false;
    };
    set_backend_mode(parsed);
    true
}

/// Returns configured pathfinding backend mode string.
pub fn get_pathfind_backend_mode() -> &'static str {
    let mode = get_backend_mode();
    backend_mode_to_str_core(mode)
}

/// Returns resolved pathfinding backend mode string (feature-gated resolution).
pub fn resolve_pathfind_backend_mode() -> &'static str {
    let mode = get_backend_mode();
    resolve_backend_mode_str_core(mode)
}

/// Returns whether GPU backend is available in this build.
pub fn has_gpu_pathfind_backend() -> bool {
    has_gpu_backend()
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
    validate_grid_inputs(width, height, walkable, move_cost)?;
    if from_x == to_x && from_y == to_y {
        return Ok(vec![GridPos::new(from_x, from_y)]);
    }
    let grid = build_grid_cost_map_unchecked(width, height, walkable, move_cost);
    Ok(find_path(
        &grid,
        GridPos::new(from_x, from_y),
        GridPos::new(to_x, to_y),
        max_steps,
    ))
}

fn validate_grid_inputs(
    width: i32,
    height: i32,
    walkable: &[u8],
    move_cost: &[f32],
) -> Result<(), PathfindError> {
    if width <= 0 || height <= 0 {
        return Err(PathfindError::InvalidDimensions { width, height });
    }
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
    Ok(())
}

fn build_grid_cost_map_unchecked(
    width: i32,
    height: i32,
    walkable: &[u8],
    move_cost: &[f32],
) -> GridCostMap {
    GridCostMap::from_flat_bytes_unchecked(width, height, walkable, move_cost)
}

fn build_grid_cost_map(
    width: i32,
    height: i32,
    walkable: &[u8],
    move_cost: &[f32],
) -> Result<GridCostMap, PathfindError> {
    validate_grid_inputs(width, height, walkable, move_cost)?;
    Ok(build_grid_cost_map_unchecked(
        width, height, walkable, move_cost,
    ))
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

    if from_points
        .iter()
        .zip(to_points.iter())
        .all(|(from, to)| from == to)
    {
        validate_grid_inputs(width, height, walkable, move_cost)?;
        let mut out = Vec::with_capacity(from_points.len());
        for &(x, y) in from_points {
            out.push(vec![GridPos::new(x, y)]);
        }
        return Ok(out);
    }

    let grid = build_grid_cost_map(width, height, walkable, move_cost)?;
    let mut workspace = PathfindWorkspace::new((width * height) as usize);
    let mut out = Vec::with_capacity(from_points.len());
    for idx in 0..from_points.len() {
        let (from_x, from_y) = from_points[idx];
        let (to_x, to_y) = to_points[idx];
        if from_x == to_x && from_y == to_y {
            out.push(vec![GridPos::new(from_x, from_y)]);
            continue;
        }
        let path = find_path_with_workspace(
            &grid,
            GridPos::new(from_x, from_y),
            GridPos::new(to_x, to_y),
            max_steps,
            &mut workspace,
        );
        out.push(path);
    }
    Ok(out)
}

pub fn pathfind_grid_batch_xy_bytes(
    width: i32,
    height: i32,
    walkable: &[u8],
    move_cost: &[f32],
    from_xy: &[i32],
    to_xy: &[i32],
    max_steps: usize,
) -> Result<Vec<Vec<GridPos>>, PathfindError> {
    if from_xy.len() != to_xy.len() || !from_xy.len().is_multiple_of(2) {
        return Err(PathfindError::MismatchedBatchLength {
            from_len: from_xy.len(),
            to_len: to_xy.len(),
        });
    }

    let mut all_stationary = true;
    let mut idx = 0usize;
    while idx + 1 < from_xy.len() {
        if from_xy[idx] != to_xy[idx] || from_xy[idx + 1] != to_xy[idx + 1] {
            all_stationary = false;
            break;
        }
        idx += 2;
    }
    if all_stationary {
        validate_grid_inputs(width, height, walkable, move_cost)?;
        let pair_count = from_xy.len() / 2;
        let mut out = Vec::with_capacity(pair_count);
        let mut i = 0usize;
        while i + 1 < from_xy.len() {
            out.push(vec![GridPos::new(from_xy[i], from_xy[i + 1])]);
            i += 2;
        }
        return Ok(out);
    }

    let grid = build_grid_cost_map(width, height, walkable, move_cost)?;
    let mut workspace = PathfindWorkspace::new((width * height) as usize);
    let pair_count = from_xy.len() / 2;
    let mut out = Vec::with_capacity(pair_count);
    let mut cursor = 0usize;
    while cursor + 1 < from_xy.len() {
        let from_x = from_xy[cursor];
        let from_y = from_xy[cursor + 1];
        let to_x = to_xy[cursor];
        let to_y = to_xy[cursor + 1];
        if from_x == to_x && from_y == to_y {
            out.push(vec![GridPos::new(from_x, from_y)]);
            cursor += 2;
            continue;
        }
        let path = find_path_with_workspace(
            &grid,
            GridPos::new(from_x, from_y),
            GridPos::new(to_x, to_y),
            max_steps,
            &mut workspace,
        );
        out.push(path);
        cursor += 2;
    }
    Ok(out)
}

pub fn pathfind_grid_batch_vec2_bytes(
    width: i32,
    height: i32,
    walkable: &[u8],
    move_cost: &[f32],
    from_points: &[Vector2],
    to_points: &[Vector2],
    max_steps: usize,
) -> Result<Vec<Vec<GridPos>>, PathfindError> {
    if from_points.len() != to_points.len() {
        return Err(PathfindError::MismatchedBatchLength {
            from_len: from_points.len(),
            to_len: to_points.len(),
        });
    }

    if from_points.iter().zip(to_points.iter()).all(|(from, to)| {
        from.x.round() as i32 == to.x.round() as i32 && from.y.round() as i32 == to.y.round() as i32
    }) {
        validate_grid_inputs(width, height, walkable, move_cost)?;
        let mut out = Vec::with_capacity(from_points.len());
        for from in from_points {
            out.push(vec![GridPos::new(
                from.x.round() as i32,
                from.y.round() as i32,
            )]);
        }
        return Ok(out);
    }

    let grid = build_grid_cost_map(width, height, walkable, move_cost)?;
    let mut workspace = PathfindWorkspace::new((width * height) as usize);
    let mut out = Vec::with_capacity(from_points.len());
    for idx in 0..from_points.len() {
        let from = from_points[idx];
        let to = to_points[idx];
        let from_x = from.x.round() as i32;
        let from_y = from.y.round() as i32;
        let to_x = to.x.round() as i32;
        let to_y = to.y.round() as i32;
        if from_x == to_x && from_y == to_y {
            out.push(vec![GridPos::new(from_x, from_y)]);
            continue;
        }
        let path = find_path_with_workspace(
            &grid,
            GridPos::new(from_x, from_y),
            GridPos::new(to_x, to_y),
            max_steps,
            &mut workspace,
        );
        out.push(path);
    }
    Ok(out)
}

/// Dispatches tuple batch pathfinding according to current backend mode.
pub fn pathfind_grid_batch_dispatch_bytes(
    width: i32,
    height: i32,
    walkable: &[u8],
    move_cost: &[f32],
    from_points: &[(i32, i32)],
    to_points: &[(i32, i32)],
    max_steps: usize,
) -> Result<Vec<Vec<GridPos>>, PathfindError> {
    let backend_mode = get_backend_mode();
    dispatch_pathfind_grid_batch_bytes(
        backend_mode,
        width,
        height,
        walkable,
        move_cost,
        from_points,
        to_points,
        max_steps,
    )
}

/// Dispatches packed-xy batch pathfinding according to current backend mode.
pub fn pathfind_grid_batch_xy_dispatch_bytes(
    width: i32,
    height: i32,
    walkable: &[u8],
    move_cost: &[f32],
    from_xy: &[i32],
    to_xy: &[i32],
    max_steps: usize,
) -> Result<Vec<Vec<GridPos>>, PathfindError> {
    let backend_mode = get_backend_mode();
    dispatch_pathfind_grid_batch_xy_bytes(
        backend_mode,
        width,
        height,
        walkable,
        move_cost,
        from_xy,
        to_xy,
        max_steps,
    )
}

pub(crate) fn dispatch_pathfind_grid_bytes(
    _backend_mode: u8,
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
    record_dispatch(PATHFIND_BACKEND_CPU);
    pathfind_grid_bytes(
        width, height, walkable, move_cost, from_x, from_y, to_x, to_y, max_steps,
    )
}

pub(crate) fn dispatch_pathfind_grid_batch_vec2_bytes(
    _backend_mode: u8,
    width: i32,
    height: i32,
    walkable: &[u8],
    move_cost: &[f32],
    from_points: &[Vector2],
    to_points: &[Vector2],
    max_steps: usize,
) -> Result<Vec<Vec<GridPos>>, PathfindError> {
    record_dispatch(PATHFIND_BACKEND_CPU);
    pathfind_grid_batch_vec2_bytes(
        width,
        height,
        walkable,
        move_cost,
        from_points,
        to_points,
        max_steps,
    )
}

pub(crate) fn dispatch_pathfind_grid_batch_bytes(
    _backend_mode: u8,
    width: i32,
    height: i32,
    walkable: &[u8],
    move_cost: &[f32],
    from_points: &[(i32, i32)],
    to_points: &[(i32, i32)],
    max_steps: usize,
) -> Result<Vec<Vec<GridPos>>, PathfindError> {
    record_dispatch(PATHFIND_BACKEND_CPU);
    pathfind_grid_batch_bytes(
        width,
        height,
        walkable,
        move_cost,
        from_points,
        to_points,
        max_steps,
    )
}

pub(crate) fn dispatch_pathfind_grid_batch_xy_bytes(
    _backend_mode: u8,
    width: i32,
    height: i32,
    walkable: &[u8],
    move_cost: &[f32],
    from_xy: &[i32],
    to_xy: &[i32],
    max_steps: usize,
) -> Result<Vec<Vec<GridPos>>, PathfindError> {
    record_dispatch(PATHFIND_BACKEND_CPU);
    pathfind_grid_batch_xy_bytes(
        width, height, walkable, move_cost, from_xy, to_xy, max_steps,
    )
}
