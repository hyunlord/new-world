//! sim-bridge: Boundary adapters between external callers and simulation crates.
//!
//! Phase R-3 will expose this through Godot GDExtension.
//! For now, this module provides pure-Rust conversion helpers that can be
//! reused by the future FFI layer.

mod pathfinding_backend;
mod pathfinding_gpu;

use fluent_bundle::types::FluentNumber;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use godot::builtin::VariantType;
use godot::prelude::*;
use pathfinding_backend::{
    get_backend_mode, has_gpu_backend, read_dispatch_counts, record_dispatch,
    reset_dispatch_counts, set_backend_mode, PATHFIND_BACKEND_GPU,
};
use pathfinding_gpu::{
    pathfind_grid_batch_tuple_gpu_bytes, pathfind_grid_batch_vec2_gpu_bytes,
    pathfind_grid_batch_xy_gpu_bytes, pathfind_grid_gpu_bytes,
};
use serde::Deserialize;
use sim_core::{config::GameConfig, GameCalendar, WorldMap};
use sim_engine::{EngineSnapshot, GameEvent, SimEngine, SimResources};
use sim_systems::{
    body,
    pathfinding::{find_path, find_path_with_workspace, GridCostMap, GridPos, PathfindWorkspace},
    runtime::{
        ChildStressProcessorRuntimeSystem, EmotionRuntimeSystem, JobAssignmentRuntimeSystem,
        MentalBreakRuntimeSystem, NeedsRuntimeSystem, ResourceRegenSystem, StatSyncSystem,
        StatThresholdRuntimeSystem, StatsRecorderSystem, StressRuntimeSystem,
        UpperNeedsRuntimeSystem,
    },
    stat_curve,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::{Arc, Mutex, OnceLock};
use unic_langid::LanguageIdentifier;

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
    let Some(parsed) = parse_pathfind_backend(mode) else {
        return false;
    };
    set_backend_mode(parsed);
    true
}

/// Returns configured pathfinding backend mode string.
pub fn get_pathfind_backend_mode() -> &'static str {
    let mode = get_backend_mode();
    backend_mode_to_str(mode)
}

/// Returns resolved pathfinding backend mode string (feature-gated resolution).
pub fn resolve_pathfind_backend_mode() -> &'static str {
    let mode = get_backend_mode();
    resolve_backend_mode(mode)
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
    if from_xy.len() != to_xy.len() || from_xy.len() % 2 != 0 {
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

fn dispatch_pathfind_grid_bytes(
    backend_mode: u8,
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
    let resolved_mode = resolve_backend_mode_code(backend_mode);
    record_dispatch(resolved_mode);
    match resolved_mode {
        PATHFIND_BACKEND_GPU => pathfind_grid_gpu_bytes(
            width, height, walkable, move_cost, from_x, from_y, to_x, to_y, max_steps,
        ),
        _ => pathfind_grid_bytes(
            width, height, walkable, move_cost, from_x, from_y, to_x, to_y, max_steps,
        ),
    }
}

fn dispatch_pathfind_grid_batch_vec2_bytes(
    backend_mode: u8,
    width: i32,
    height: i32,
    walkable: &[u8],
    move_cost: &[f32],
    from_points: &[Vector2],
    to_points: &[Vector2],
    max_steps: usize,
) -> Result<Vec<Vec<GridPos>>, PathfindError> {
    let resolved_mode = resolve_backend_mode_code(backend_mode);
    record_dispatch(resolved_mode);
    match resolved_mode {
        PATHFIND_BACKEND_GPU => pathfind_grid_batch_vec2_gpu_bytes(
            width,
            height,
            walkable,
            move_cost,
            from_points,
            to_points,
            max_steps,
        ),
        _ => pathfind_grid_batch_vec2_bytes(
            width,
            height,
            walkable,
            move_cost,
            from_points,
            to_points,
            max_steps,
        ),
    }
}

fn dispatch_pathfind_grid_batch_bytes(
    backend_mode: u8,
    width: i32,
    height: i32,
    walkable: &[u8],
    move_cost: &[f32],
    from_points: &[(i32, i32)],
    to_points: &[(i32, i32)],
    max_steps: usize,
) -> Result<Vec<Vec<GridPos>>, PathfindError> {
    let resolved_mode = resolve_backend_mode_code(backend_mode);
    record_dispatch(resolved_mode);
    match resolved_mode {
        PATHFIND_BACKEND_GPU => pathfind_grid_batch_tuple_gpu_bytes(
            width,
            height,
            walkable,
            move_cost,
            from_points,
            to_points,
            max_steps,
        ),
        _ => pathfind_grid_batch_bytes(
            width,
            height,
            walkable,
            move_cost,
            from_points,
            to_points,
            max_steps,
        ),
    }
}

fn dispatch_pathfind_grid_batch_xy_bytes(
    backend_mode: u8,
    width: i32,
    height: i32,
    walkable: &[u8],
    move_cost: &[f32],
    from_xy: &[i32],
    to_xy: &[i32],
    max_steps: usize,
) -> Result<Vec<Vec<GridPos>>, PathfindError> {
    let resolved_mode = resolve_backend_mode_code(backend_mode);
    record_dispatch(resolved_mode);
    match resolved_mode {
        PATHFIND_BACKEND_GPU => pathfind_grid_batch_xy_gpu_bytes(
            width, height, walkable, move_cost, from_xy, to_xy, max_steps,
        ),
        _ => pathfind_grid_batch_xy_bytes(
            width, height, walkable, move_cost, from_xy, to_xy, max_steps,
        ),
    }
}

fn packed_i32_to_vec(values: &PackedInt32Array) -> Vec<i32> {
    values.as_slice().to_vec()
}

fn packed_f32_to_vec(values: &PackedFloat32Array) -> Vec<f32> {
    values.as_slice().to_vec()
}

fn packed_u8_to_vec(values: &PackedByteArray) -> Vec<u8> {
    values.as_slice().to_vec()
}

fn vec_i32_to_packed(values: Vec<i32>) -> PackedInt32Array {
    PackedInt32Array::from(values)
}

fn vec_f32_to_packed(values: Vec<f32>) -> PackedFloat32Array {
    PackedFloat32Array::from(values)
}

fn vec_u8_to_packed(values: Vec<u8>) -> PackedByteArray {
    PackedByteArray::from(values)
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

fn encode_path_groups_xy(path_groups: Vec<Vec<GridPos>>) -> Array<PackedInt32Array> {
    let mut output: Array<PackedInt32Array> = Array::new();
    for group in path_groups {
        let mut packed: PackedInt32Array = PackedInt32Array::new();
        packed.resize(group.len() * 2);
        for (idx, p) in group.into_iter().enumerate() {
            let base = idx * 2;
            packed[base] = p.x;
            packed[base + 1] = p.y;
        }
        output.push(&packed);
    }
    output
}

fn encode_path_groups_vec2(path_groups: Vec<Vec<GridPos>>) -> Array<PackedVector2Array> {
    let mut output: Array<PackedVector2Array> = Array::new();
    for group in path_groups {
        let packed = encode_path_vec2(group);
        output.push(&packed);
    }
    output
}

fn encode_path_xy(path: Vec<GridPos>) -> PackedInt32Array {
    let mut packed: PackedInt32Array = PackedInt32Array::new();
    packed.resize(path.len() * 2);
    for (idx, p) in path.into_iter().enumerate() {
        let base = idx * 2;
        packed[base] = p.x;
        packed[base + 1] = p.y;
    }
    packed
}

fn encode_path_vec2(path: Vec<GridPos>) -> PackedVector2Array {
    let mut packed: PackedVector2Array = PackedVector2Array::new();
    packed.resize(path.len());
    for (idx, p) in path.into_iter().enumerate() {
        packed[idx] = Vector2::new(p.x as f32, p.y as f32);
    }
    packed
}

fn parse_pathfind_backend(mode: &str) -> Option<u8> {
    pathfinding_backend::parse_backend_mode(mode)
}

#[inline]
fn normalize_max_steps(max_steps: i32) -> usize {
    if max_steps <= 0 {
        200_usize
    } else {
        max_steps as usize
    }
}

fn resolve_backend_mode_code(mode: u8) -> u8 {
    pathfinding_backend::resolve_backend_mode_code(mode)
}

fn backend_mode_to_str(mode: u8) -> &'static str {
    pathfinding_backend::backend_mode_to_str(mode)
}

fn resolve_backend_mode(mode: u8) -> &'static str {
    pathfinding_backend::resolve_backend_mode_str(mode)
}

const EVENT_TYPE_ID_TICK_COMPLETED: i32 = 1;
const EVENT_TYPE_ID_SIMULATION_PAUSED: i32 = 2;
const EVENT_TYPE_ID_SIMULATION_RESUMED: i32 = 3;
const EVENT_TYPE_ID_SPEED_CHANGED: i32 = 4;
const EVENT_TYPE_ID_GENERIC: i32 = 9000;
const RUNTIME_SYSTEM_KEY_STAT_THRESHOLD: &str = "stat_threshold_system";
const RUNTIME_SYSTEM_KEY_EMOTION: &str = "emotion_system";
const RUNTIME_SYSTEM_KEY_STRESS: &str = "stress_system";
const RUNTIME_SYSTEM_KEY_CHILD_STRESS_PROCESSOR: &str = "child_stress_processor";
const RUNTIME_SYSTEM_KEY_MENTAL_BREAK: &str = "mental_break_system";
const RUNTIME_SYSTEM_KEY_JOB_ASSIGNMENT: &str = "job_assignment_system";
const RUNTIME_SYSTEM_KEY_NEEDS: &str = "needs_system";
const RUNTIME_SYSTEM_KEY_UPPER_NEEDS: &str = "upper_needs_system";
const RUNTIME_SYSTEM_KEY_STAT_SYNC: &str = "stat_sync_system";
const RUNTIME_SYSTEM_KEY_RESOURCE_REGEN: &str = "resource_regen_system";
const RUNTIME_SYSTEM_KEY_STATS_RECORDER: &str = "stats_recorder";
const RUNTIME_SPEED_OPTIONS: [u32; 5] = [1, 2, 3, 5, 10];
const RUNTIME_COMPUTE_DOMAINS: [&str; 5] =
    ["pathfinding", "needs", "stress", "emotion", "orchestration"];
const WS2_MAGIC: [u8; 4] = *b"WS2\0";
const WS2_VERSION: u16 = 1;
const WS2_HEADER_SIZE: usize = 16;
static FLUENT_SOURCES: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

#[derive(Debug, Clone, Deserialize)]
struct RuntimeConfig {
    world_width: Option<u32>,
    world_height: Option<u32>,
    ticks_per_second: Option<u32>,
    max_ticks_per_frame: Option<u32>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            world_width: Some(256),
            world_height: Some(256),
            ticks_per_second: Some(10),
            max_ticks_per_frame: Some(5),
        }
    }
}

struct RuntimeState {
    engine: SimEngine,
    accumulator: f64,
    ticks_per_second: u32,
    max_ticks_per_frame: u32,
    speed_index: i32,
    paused: bool,
    captured_events: Arc<Mutex<Vec<GameEvent>>>,
    registered_systems: Vec<RuntimeSystemEntry>,
    rust_registered_systems: HashSet<String>,
    compute_domain_modes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct RuntimeSystemEntry {
    name: String,
    system_key: String,
    priority: i32,
    tick_interval: i32,
    active: bool,
    registration_index: i32,
    rust_implemented: bool,
    rust_registered: bool,
    exec_backend: String,
}

impl RuntimeState {
    fn from_seed(seed: u64, config: RuntimeConfig) -> Self {
        let game_config = GameConfig::default();
        let world_width = config.world_width.unwrap_or(256).max(1);
        let world_height = config.world_height.unwrap_or(256).max(1);
        let ticks_per_second = config.ticks_per_second.unwrap_or(10).max(1);
        let max_ticks_per_frame = config.max_ticks_per_frame.unwrap_or(5).max(1);
        let calendar = GameCalendar::new(&game_config);
        let map = WorldMap::new(world_width, world_height, seed);
        let captured_events = Arc::new(Mutex::new(Vec::<GameEvent>::with_capacity(256)));
        let mut resources = SimResources::new(calendar, map, seed);
        let event_sink = Arc::clone(&captured_events);
        resources
            .event_bus
            .subscribe(Box::new(move |event: &GameEvent| {
                if let Ok(mut buffer) = event_sink.lock() {
                    buffer.push(event.clone());
                }
            }));
        let engine = SimEngine::new(resources);
        Self {
            engine,
            accumulator: 0.0,
            ticks_per_second,
            max_ticks_per_frame,
            speed_index: 0,
            paused: false,
            captured_events,
            registered_systems: Vec::new(),
            rust_registered_systems: HashSet::new(),
            compute_domain_modes: runtime_default_compute_domain_modes(),
        }
    }
}

fn parse_runtime_config(config_json: &str) -> RuntimeConfig {
    if config_json.trim().is_empty() {
        return RuntimeConfig::default();
    }
    serde_json::from_str::<RuntimeConfig>(config_json).unwrap_or_default()
}

fn clamp_speed_index(index: i32) -> i32 {
    index.clamp(0, (RUNTIME_SPEED_OPTIONS.len() - 1) as i32)
}

fn runtime_speed_multiplier(index: i32) -> f64 {
    let clamped = clamp_speed_index(index) as usize;
    f64::from(RUNTIME_SPEED_OPTIONS[clamped])
}

fn runtime_system_key_from_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let normalized = trimmed.replace('\\', "/").to_lowercase();
    let tail = normalized.rsplit('/').next().unwrap_or_default();
    let key = tail.strip_suffix(".gd").unwrap_or(tail);
    key.to_string()
}

fn runtime_supports_rust_system(system_key: &str) -> bool {
    matches!(
        system_key,
        RUNTIME_SYSTEM_KEY_STATS_RECORDER
            | RUNTIME_SYSTEM_KEY_RESOURCE_REGEN
            | RUNTIME_SYSTEM_KEY_STAT_SYNC
            | RUNTIME_SYSTEM_KEY_UPPER_NEEDS
            | RUNTIME_SYSTEM_KEY_NEEDS
            | RUNTIME_SYSTEM_KEY_STRESS
            | RUNTIME_SYSTEM_KEY_EMOTION
            | RUNTIME_SYSTEM_KEY_CHILD_STRESS_PROCESSOR
            | RUNTIME_SYSTEM_KEY_MENTAL_BREAK
            | RUNTIME_SYSTEM_KEY_JOB_ASSIGNMENT
            | RUNTIME_SYSTEM_KEY_STAT_THRESHOLD
    )
}

fn register_supported_rust_system(
    state: &mut RuntimeState,
    system_key: &str,
    priority: i32,
    tick_interval: i32,
) -> bool {
    if !runtime_supports_rust_system(system_key) {
        return false;
    }
    if state.rust_registered_systems.contains(system_key) {
        return true;
    }
    let priority_u32 = priority.max(0) as u32;
    let tick_interval_u64 = tick_interval.max(1) as u64;
    match system_key {
        RUNTIME_SYSTEM_KEY_STAT_THRESHOLD => {
            state
                .engine
                .register(StatThresholdRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_EMOTION => {
            state
                .engine
                .register(EmotionRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_STRESS => {
            state
                .engine
                .register(StressRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_CHILD_STRESS_PROCESSOR => {
            state
                .engine
                .register(ChildStressProcessorRuntimeSystem::new(
                    priority_u32,
                    tick_interval_u64,
                ));
        }
        RUNTIME_SYSTEM_KEY_MENTAL_BREAK => {
            state
                .engine
                .register(MentalBreakRuntimeSystem::new(
                    priority_u32,
                    tick_interval_u64,
                ));
        }
        RUNTIME_SYSTEM_KEY_JOB_ASSIGNMENT => {
            state
                .engine
                .register(JobAssignmentRuntimeSystem::new(
                    priority_u32,
                    tick_interval_u64,
                ));
        }
        RUNTIME_SYSTEM_KEY_NEEDS => {
            state
                .engine
                .register(NeedsRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_UPPER_NEEDS => {
            state
                .engine
                .register(UpperNeedsRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_STAT_SYNC => {
            state
                .engine
                .register(StatSyncSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_RESOURCE_REGEN => {
            state
                .engine
                .register(ResourceRegenSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_STATS_RECORDER => {
            state
                .engine
                .register(StatsRecorderSystem::new(priority_u32, tick_interval_u64));
        }
        _ => {
            return false;
        }
    }
    state
        .rust_registered_systems
        .insert(system_key.to_string());
    true
}

fn game_event_type_id(event: &GameEvent) -> i32 {
    match event {
        GameEvent::TickCompleted { .. } => EVENT_TYPE_ID_TICK_COMPLETED,
        GameEvent::SimulationPaused => EVENT_TYPE_ID_SIMULATION_PAUSED,
        GameEvent::SimulationResumed => EVENT_TYPE_ID_SIMULATION_RESUMED,
        GameEvent::SpeedChanged { .. } => EVENT_TYPE_ID_SPEED_CHANGED,
        _ => EVENT_TYPE_ID_GENERIC,
    }
}

fn game_event_tick(event: &GameEvent) -> i64 {
    match event {
        GameEvent::TickCompleted { tick } => *tick as i64,
        _ => -1,
    }
}

fn game_event_payload(event: &GameEvent) -> VarDictionary {
    let mut payload = VarDictionary::new();
    match event {
        GameEvent::TickCompleted { tick } => {
            payload.set("tick", *tick as i64);
        }
        GameEvent::EntityDied { entity_id, cause } => {
            payload.set("entity_id", entity_id.0 as i64);
            payload.set("cause", cause.clone());
        }
        GameEvent::EntitySpawned { entity_id } => {
            payload.set("entity_id", entity_id.0 as i64);
        }
        GameEvent::SpeedChanged { speed_index } => {
            payload.set("speed_index", *speed_index as i64);
        }
        _ => {}
    }
    payload
}

fn game_event_to_v2_dict(event: &GameEvent) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("event_type_id", game_event_type_id(event));
    dict.set("event_name", event.name());
    dict.set("tick", game_event_tick(event));
    dict.set("payload", game_event_payload(event));
    dict
}

fn runtime_default_compute_domain_modes() -> HashMap<String, String> {
    let mut modes = HashMap::<String, String>::new();
    for domain in RUNTIME_COMPUTE_DOMAINS {
        modes.insert(domain.to_string(), "gpu_auto".to_string());
    }
    modes
}

fn is_supported_compute_mode(mode: &str) -> bool {
    matches!(mode, "cpu" | "gpu_auto" | "gpu_force")
}

fn dict_get_string(dict: &VarDictionary, key: &str) -> Option<String> {
    let value = dict.get(key)?;
    Some(value.to::<GString>().to_string())
}

fn dict_get_i32(dict: &VarDictionary, key: &str) -> Option<i32> {
    let value = dict.get(key)?;
    Some(value.to::<i64>() as i32)
}

fn dict_get_bool(dict: &VarDictionary, key: &str) -> Option<bool> {
    let value = dict.get(key)?;
    Some(value.to::<bool>())
}

fn fluent_sources() -> &'static Mutex<HashMap<String, String>> {
    FLUENT_SOURCES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn locale_key(locale: &str) -> String {
    locale.trim().to_lowercase()
}

fn parse_language_identifier(locale: &str) -> LanguageIdentifier {
    locale
        .parse::<LanguageIdentifier>()
        .ok()
        .or_else(|| "en-US".parse::<LanguageIdentifier>().ok())
        .expect("fallback locale should parse")
}

fn store_fluent_source(locale: &str, source: &str) -> bool {
    let key = locale_key(locale);
    if key.is_empty() || source.trim().is_empty() {
        return false;
    }
    let Ok(mut sources) = fluent_sources().lock() else {
        return false;
    };
    sources.insert(key, source.to_string());
    true
}

fn clear_fluent_source(locale: &str) {
    let key = locale_key(locale);
    if key.is_empty() {
        return;
    }
    let Ok(mut sources) = fluent_sources().lock() else {
        return;
    };
    sources.remove(&key);
}

fn lookup_fluent_source(locale: &str) -> Option<String> {
    let key = locale_key(locale);
    let Ok(sources) = fluent_sources().lock() else {
        return None;
    };
    if let Some(source) = sources.get(&key) {
        return Some(source.clone());
    }
    if key.contains('-') {
        let base = key.split('-').next().unwrap_or_default();
        if let Some(source) = sources.get(base) {
            return Some(source.clone());
        }
    }
    sources.get("en").cloned()
}

fn variant_to_fluent_value(value: &Variant) -> FluentValue<'static> {
    match value.get_type() {
        VariantType::INT => FluentValue::Number(FluentNumber::from(value.to::<i64>() as f64)),
        VariantType::FLOAT => FluentValue::Number(FluentNumber::from(value.to::<f64>())),
        VariantType::BOOL => {
            let value_text = if value.to::<bool>() { "true" } else { "false" };
            FluentValue::String(value_text.to_string().into())
        }
        _ => FluentValue::String(value.to::<GString>().to_string().into()),
    }
}

fn build_fluent_args(params: &VarDictionary) -> Option<FluentArgs<'static>> {
    let mut args = FluentArgs::new();
    let mut has_arg = false;
    for (key_var, value_var) in params.iter_shared() {
        let key = key_var.to::<GString>().to_string();
        if key.is_empty() {
            continue;
        }
        args.set(key, variant_to_fluent_value(&value_var));
        has_arg = true;
    }
    if has_arg {
        Some(args)
    } else {
        None
    }
}

fn format_fluent_from_source(
    source: &str,
    locale: &str,
    key: &str,
    params: &VarDictionary,
) -> Option<String> {
    let args = build_fluent_args(params);
    format_fluent_from_source_args(source, locale, key, args)
}

fn format_fluent_from_source_args(
    source: &str,
    locale: &str,
    key: &str,
    args: Option<FluentArgs<'static>>,
) -> Option<String> {
    let resource = FluentResource::try_new(source.to_string()).ok()?;
    let language_id = parse_language_identifier(locale);
    let mut bundle = FluentBundle::new(vec![language_id]);
    bundle.set_use_isolating(false);
    bundle.add_resource(resource).ok()?;
    let message = bundle.get_message(key)?;
    let pattern = message.value()?;
    let mut errors = Vec::new();
    let resolved = bundle.format_pattern(pattern, args.as_ref(), &mut errors);
    Some(resolved.into_owned())
}

fn format_fluent_message(locale: &str, key: &str, params: &VarDictionary) -> Option<String> {
    let source = lookup_fluent_source(locale)?;
    format_fluent_from_source(&source, locale, key, params)
}

fn encode_ws2_blob(snapshot: &EngineSnapshot) -> Option<Vec<u8>> {
    let serialized = bincode::serialize(snapshot).ok()?;
    let compressed = zstd::stream::encode_all(serialized.as_slice(), 3).ok()?;
    let checksum = crc32fast::hash(&compressed);
    let payload_len = compressed.len() as u32;
    let mut out = Vec::with_capacity(WS2_HEADER_SIZE + compressed.len());
    out.extend_from_slice(&WS2_MAGIC);
    out.extend_from_slice(&WS2_VERSION.to_le_bytes());
    out.extend_from_slice(&0_u16.to_le_bytes());
    out.extend_from_slice(&checksum.to_le_bytes());
    out.extend_from_slice(&payload_len.to_le_bytes());
    out.extend_from_slice(compressed.as_slice());
    Some(out)
}

fn decode_ws2_blob(bytes: &[u8]) -> Option<EngineSnapshot> {
    if bytes.len() < WS2_HEADER_SIZE {
        return None;
    }
    if bytes[0..4] != WS2_MAGIC {
        return None;
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != WS2_VERSION {
        return None;
    }
    let checksum = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let payload_len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
    if bytes.len() != WS2_HEADER_SIZE + payload_len {
        return None;
    }
    let payload = &bytes[WS2_HEADER_SIZE..];
    if crc32fast::hash(payload) != checksum {
        return None;
    }
    let decoded = zstd::stream::decode_all(payload).ok()?;
    bincode::deserialize::<EngineSnapshot>(&decoded).ok()
}

#[derive(GodotClass)]
#[class(base=Object)]
pub struct WorldSimRuntime {
    base: Base<Object>,
    state: Option<RuntimeState>,
}

#[godot_api]
impl IObject for WorldSimRuntime {
    fn init(base: Base<Object>) -> Self {
        Self { base, state: None }
    }
}

#[godot_api]
impl WorldSimRuntime {
    #[func]
    fn runtime_init(&mut self, seed: i64, config_json: GString) -> bool {
        let config = parse_runtime_config(&config_json.to_string());
        self.state = Some(RuntimeState::from_seed(seed.max(0) as u64, config));
        true
    }

    #[func]
    fn runtime_is_initialized(&self) -> bool {
        self.state.is_some()
    }

    #[func]
    fn runtime_tick_frame(
        &mut self,
        delta_sec: f64,
        speed_index: i32,
        paused: bool,
    ) -> VarDictionary {
        let mut out = VarDictionary::new();
        let Some(state) = self.state.as_mut() else {
            out.set("initialized", false);
            out.set("current_tick", 0_i64);
            out.set("ticks_processed", 0_i64);
            out.set("speed_index", 0_i64);
            out.set("paused", true);
            out.set("accumulator", 0.0_f64);
            return out;
        };

        if speed_index >= 0 {
            let clamped_speed = clamp_speed_index(speed_index);
            if clamped_speed != state.speed_index {
                state.speed_index = clamped_speed;
                state
                    .engine
                    .resources_mut()
                    .event_bus
                    .emit(GameEvent::SpeedChanged {
                        speed_index: clamped_speed,
                    });
            }
        }
        if paused != state.paused {
            state.paused = paused;
            if paused {
                state
                    .engine
                    .resources_mut()
                    .event_bus
                    .emit(GameEvent::SimulationPaused);
            } else {
                state
                    .engine
                    .resources_mut()
                    .event_bus
                    .emit(GameEvent::SimulationResumed);
            }
        }
        let mut ticks_processed: u32 = 0;

        if !paused {
            let tick_duration = 1.0_f64 / f64::from(state.ticks_per_second);
            state.accumulator += delta_sec.max(0.0) * runtime_speed_multiplier(state.speed_index);
            while state.accumulator >= tick_duration && ticks_processed < state.max_ticks_per_frame
            {
                let emitted_tick = state.engine.current_tick() + 1;
                state
                    .engine
                    .resources_mut()
                    .event_bus
                    .emit(GameEvent::TickCompleted { tick: emitted_tick });
                state.engine.tick();
                state.accumulator -= tick_duration;
                ticks_processed += 1;
            }
            if state.accumulator > tick_duration * 3.0 {
                state.accumulator = 0.0;
            }
        }

        out.set("initialized", true);
        out.set("current_tick", state.engine.current_tick() as i64);
        out.set("ticks_processed", ticks_processed as i64);
        out.set("speed_index", state.speed_index as i64);
        out.set("paused", paused);
        out.set("accumulator", state.accumulator);
        out
    }

    #[func]
    fn runtime_get_snapshot(&self) -> PackedByteArray {
        let Some(state) = self.state.as_ref() else {
            return PackedByteArray::new();
        };
        let snapshot = state.engine.snapshot();
        let bytes = serde_json::to_vec(&snapshot).unwrap_or_default();
        PackedByteArray::from(bytes)
    }

    #[func]
    fn runtime_apply_snapshot(&mut self, snapshot_bytes: PackedByteArray) -> bool {
        let Some(state) = self.state.as_mut() else {
            return false;
        };
        let bytes = snapshot_bytes.as_slice();
        let Ok(snapshot) = serde_json::from_slice::<EngineSnapshot>(bytes) else {
            return false;
        };
        state.engine.restore_from_snapshot(&snapshot);
        true
    }

    #[func]
    fn runtime_save_ws2(&self, path: GString) -> bool {
        let Some(state) = self.state.as_ref() else {
            return false;
        };
        let path_string = path.to_string();
        if path_string.is_empty() {
            return false;
        }
        let snapshot = state.engine.snapshot();
        let Some(blob) = encode_ws2_blob(&snapshot) else {
            return false;
        };
        fs::write(path_string, blob).is_ok()
    }

    #[func]
    fn runtime_load_ws2(&mut self, path: GString) -> bool {
        let Some(state) = self.state.as_mut() else {
            return false;
        };
        let path_string = path.to_string();
        if path_string.is_empty() {
            return false;
        }
        let Ok(bytes) = fs::read(path_string) else {
            return false;
        };
        let Some(snapshot) = decode_ws2_blob(&bytes) else {
            return false;
        };
        state.engine.restore_from_snapshot(&snapshot);
        true
    }

    #[func]
    fn runtime_export_events_v2(&mut self) -> Array<VarDictionary> {
        let mut out: Array<VarDictionary> = Array::new();
        let Some(state) = self.state.as_mut() else {
            return out;
        };
        let mut drained: Vec<GameEvent> = Vec::new();
        if let Ok(mut events) = state.captured_events.lock() {
            drained.extend(events.drain(..));
        }
        for event in drained {
            let dict = game_event_to_v2_dict(&event);
            out.push(&dict);
        }
        out
    }

    #[func]
    fn runtime_get_registry_snapshot(&self) -> Array<VarDictionary> {
        let mut out: Array<VarDictionary> = Array::new();
        let Some(state) = self.state.as_ref() else {
            return out;
        };
        for entry in &state.registered_systems {
            let mut dict = VarDictionary::new();
            dict.set("name", entry.name.clone());
            dict.set("system_key", entry.system_key.clone());
            dict.set("priority", entry.priority);
            dict.set("tick_interval", entry.tick_interval);
            dict.set("active", entry.active);
            dict.set("registration_index", entry.registration_index);
            dict.set("rust_implemented", entry.rust_implemented);
            dict.set("rust_registered", entry.rust_registered);
            dict.set("exec_backend", entry.exec_backend.clone());
            out.push(&dict);
        }
        out
    }

    #[func]
    fn runtime_get_compute_domain_modes(&self) -> VarDictionary {
        let mut out = VarDictionary::new();
        let Some(state) = self.state.as_ref() else {
            return out;
        };
        for (domain, mode) in &state.compute_domain_modes {
            out.set(domain.as_str(), mode.as_str());
        }
        out
    }

    #[func]
    fn runtime_clear_registry(&mut self) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        state.registered_systems.clear();
        state.rust_registered_systems.clear();
        state.engine.clear_systems();
    }

    #[func]
    fn runtime_apply_commands_v2(&mut self, commands: Array<VarDictionary>) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        for command in commands.iter_shared() {
            let Some(command_id_var) = command.get("command_id") else {
                continue;
            };
            let command_id = command_id_var.to::<GString>().to_string();
            if command_id == "set_speed_index" {
                let Some(payload_var) = command.get("payload") else {
                    continue;
                };
                let payload = payload_var.to::<VarDictionary>();
                let Some(speed_var) = payload.get("speed_index") else {
                    continue;
                };
                let speed = speed_var.to::<i64>() as i32;
                state.speed_index = clamp_speed_index(speed);
                continue;
            }
            if command_id == "reset_accumulator" {
                state.accumulator = 0.0;
                continue;
            }
            if command_id == "clear_registry" {
                state.registered_systems.clear();
                state.rust_registered_systems.clear();
                state.engine.clear_systems();
                continue;
            }
            if command_id == "register_system" {
                let Some(payload_var) = command.get("payload") else {
                    continue;
                };
                let payload = payload_var.to::<VarDictionary>();
                let Some(name) = dict_get_string(&payload, "name") else {
                    continue;
                };
                let priority = dict_get_i32(&payload, "priority").unwrap_or(100);
                let tick_interval = dict_get_i32(&payload, "tick_interval").unwrap_or(1);
                let active = dict_get_bool(&payload, "active").unwrap_or(true);
                let registration_index =
                    dict_get_i32(&payload, "registration_index").unwrap_or(i32::MAX);
                let system_key = runtime_system_key_from_name(&name);
                let rust_implemented = runtime_supports_rust_system(system_key.as_str());
                let rust_registered = if rust_implemented {
                    register_supported_rust_system(
                        state,
                        system_key.as_str(),
                        priority,
                        tick_interval,
                    )
                } else {
                    false
                };
                let exec_backend = if rust_registered {
                    "rust".to_string()
                } else {
                    "gdscript".to_string()
                };
                if let Some(existing) = state
                    .registered_systems
                    .iter_mut()
                    .find(|entry| entry.name == name)
                {
                    existing.system_key = system_key;
                    existing.priority = priority;
                    existing.tick_interval = tick_interval;
                    existing.active = active;
                    existing.registration_index = registration_index;
                    existing.rust_implemented = rust_implemented;
                    existing.rust_registered = rust_registered;
                    existing.exec_backend = exec_backend;
                } else {
                    state.registered_systems.push(RuntimeSystemEntry {
                        name,
                        system_key,
                        priority,
                        tick_interval,
                        active,
                        registration_index,
                        rust_implemented,
                        rust_registered,
                        exec_backend,
                    });
                }
                state
                    .registered_systems
                    .sort_by(|a, b| {
                        a.priority
                            .cmp(&b.priority)
                            .then(a.registration_index.cmp(&b.registration_index))
                            .then(a.name.cmp(&b.name))
                    });
                continue;
            }
            if command_id == "set_compute_domain_mode" {
                let Some(payload_var) = command.get("payload") else {
                    continue;
                };
                let payload = payload_var.to::<VarDictionary>();
                let Some(domain) = dict_get_string(&payload, "domain") else {
                    continue;
                };
                let Some(mode) = dict_get_string(&payload, "mode") else {
                    continue;
                };
                if !is_supported_compute_mode(mode.as_str()) {
                    continue;
                }
                if !RUNTIME_COMPUTE_DOMAINS.contains(&domain.as_str()) {
                    continue;
                }
                state.compute_domain_modes.insert(domain, mode);
                continue;
            }
            if command_id == "set_compute_mode_all" {
                let Some(payload_var) = command.get("payload") else {
                    continue;
                };
                let payload = payload_var.to::<VarDictionary>();
                let Some(mode) = dict_get_string(&payload, "mode") else {
                    continue;
                };
                if !is_supported_compute_mode(mode.as_str()) {
                    continue;
                }
                for domain in RUNTIME_COMPUTE_DOMAINS {
                    state
                        .compute_domain_modes
                        .insert(domain.to_string(), mode.clone());
                }
                continue;
            }
        }
    }
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
    fn set_pathfinding_backend(&self, mode: GString) -> bool {
        let mode_string = mode.to_string();
        set_pathfind_backend_mode(&mode_string)
    }

    #[func]
    fn get_pathfinding_backend(&self) -> GString {
        get_pathfind_backend_mode().into()
    }

    #[func]
    fn resolve_pathfinding_backend(&self) -> GString {
        resolve_pathfind_backend_mode().into()
    }

    #[func]
    fn has_gpu_pathfinding(&self) -> bool {
        has_gpu_backend()
    }

    #[func]
    fn get_pathfinding_backend_stats(&self) -> VarDictionary {
        let configured_mode = get_backend_mode();
        let resolved_mode = resolve_backend_mode_code(configured_mode);
        let (cpu_dispatches, gpu_dispatches) = read_dispatch_counts();

        let mut dict = VarDictionary::new();
        dict.set("configured", backend_mode_to_str(configured_mode));
        dict.set("resolved", resolve_backend_mode(resolved_mode));
        dict.set("cpu_dispatches", cpu_dispatches as i64);
        dict.set("gpu_dispatches", gpu_dispatches as i64);
        dict.set("total_dispatches", (cpu_dispatches + gpu_dispatches) as i64);
        dict
    }

    #[func]
    fn reset_pathfinding_backend_stats(&self) {
        reset_dispatch_counts();
    }

    #[func]
    fn locale_load_fluent(&self, locale: GString, source: GString) -> bool {
        store_fluent_source(&locale.to_string(), &source.to_string())
    }

    #[func]
    fn locale_clear_fluent(&self, locale: GString) {
        clear_fluent_source(&locale.to_string());
    }

    #[func]
    fn locale_format_fluent(
        &self,
        locale: GString,
        key: GString,
        params: VarDictionary,
    ) -> GString {
        let key_string = key.to_string();
        if key_string.is_empty() {
            return GString::new();
        }
        let Some(resolved) = format_fluent_message(&locale.to_string(), &key_string, &params)
        else {
            return GString::from(key_string.as_str());
        };
        GString::from(resolved.as_str())
    }

    #[func]
    fn body_compute_age_curve(&self, axis: GString, age_years: f32) -> f32 {
        let axis_string = axis.to_string();
        body::compute_age_curve(axis_string.as_str(), age_years)
    }

    #[func]
    fn body_compute_age_curves(&self, age_years: f32) -> PackedFloat32Array {
        let curves = body::compute_age_curves(age_years);
        vec_f32_to_packed(curves.to_vec())
    }

    #[func]
    fn body_calc_training_gain(
        &self,
        potential: i32,
        trainability: i32,
        xp: f32,
        training_ceiling: f32,
        xp_for_full_progress: f32,
    ) -> i32 {
        body::calc_training_gain(
            potential,
            trainability,
            xp,
            training_ceiling,
            xp_for_full_progress,
        )
    }

    #[func]
    fn body_calc_training_gains(
        &self,
        potentials: PackedInt32Array,
        trainabilities: PackedInt32Array,
        xps: PackedFloat32Array,
        training_ceilings: PackedFloat32Array,
        xp_for_full_progress: f32,
    ) -> PackedInt32Array {
        let gains = body::calc_training_gains(
            potentials.as_slice(),
            trainabilities.as_slice(),
            xps.as_slice(),
            training_ceilings.as_slice(),
            xp_for_full_progress,
        );
        vec_i32_to_packed(gains)
    }

    #[func]
    fn body_calc_realized_values(
        &self,
        potentials: PackedInt32Array,
        trainabilities: PackedInt32Array,
        xps: PackedFloat32Array,
        training_ceilings: PackedFloat32Array,
        age_years: f32,
        xp_for_full_progress: f32,
    ) -> PackedInt32Array {
        let realized = body::calc_realized_values(
            potentials.as_slice(),
            trainabilities.as_slice(),
            xps.as_slice(),
            training_ceilings.as_slice(),
            age_years,
            xp_for_full_progress,
        );
        vec_i32_to_packed(realized)
    }

    #[func]
    fn body_age_trainability_modifier(&self, axis: GString, age_years: f32) -> f32 {
        let axis_string = axis.to_string();
        body::age_trainability_modifier(axis_string.as_str(), age_years)
    }

    #[func]
    fn body_age_trainability_modifier_rec(&self, age_years: f32) -> f32 {
        body::age_trainability_modifier("rec", age_years)
    }

    #[func]
    fn body_age_trainability_modifiers(&self, age_years: f32) -> PackedFloat32Array {
        let modifiers = body::age_trainability_modifiers(age_years);
        vec_f32_to_packed(modifiers.to_vec())
    }

    #[func]
    fn body_action_energy_cost(
        &self,
        base_cost: f32,
        end_norm: f32,
        end_cost_reduction: f32,
    ) -> f32 {
        body::action_energy_cost(base_cost, end_norm, end_cost_reduction)
    }

    #[func]
    fn body_rest_energy_recovery(
        &self,
        base_recovery: f32,
        rec_norm: f32,
        rec_recovery_bonus: f32,
    ) -> f32 {
        body::rest_energy_recovery(base_recovery, rec_norm, rec_recovery_bonus)
    }

    #[func]
    fn body_thirst_decay(&self, base_decay: f32, tile_temp: f32, temp_neutral: f32) -> f32 {
        body::thirst_decay(base_decay, tile_temp, temp_neutral)
    }

    #[func]
    fn body_warmth_decay(
        &self,
        base_decay: f32,
        tile_temp: f32,
        has_tile_temp: bool,
        temp_neutral: f32,
        temp_freezing: f32,
        temp_cold: f32,
    ) -> f32 {
        body::warmth_decay(
            base_decay,
            tile_temp,
            has_tile_temp,
            temp_neutral,
            temp_freezing,
            temp_cold,
        )
    }

    #[func]
    fn body_needs_base_decay_step(
        &self,
        scalar_inputs: PackedFloat32Array,
        flag_inputs: PackedByteArray,
    ) -> PackedFloat32Array {
        let scalars = scalar_inputs.as_slice();
        let hunger_value = *scalars.first().unwrap_or(&0.0);
        let hunger_decay_rate = *scalars.get(1).unwrap_or(&0.0);
        let hunger_stage_mult = *scalars.get(2).unwrap_or(&1.0);
        let hunger_metabolic_min = *scalars.get(3).unwrap_or(&0.0);
        let hunger_metabolic_range = *scalars.get(4).unwrap_or(&0.0);
        let energy_decay_rate = *scalars.get(5).unwrap_or(&0.0);
        let social_decay_rate = *scalars.get(6).unwrap_or(&0.0);
        let safety_decay_rate = *scalars.get(7).unwrap_or(&0.0);
        let thirst_base_decay = *scalars.get(8).unwrap_or(&0.0);
        let warmth_base_decay = *scalars.get(9).unwrap_or(&0.0);
        let tile_temp = *scalars.get(10).unwrap_or(&0.0);
        let temp_neutral = *scalars.get(11).unwrap_or(&0.5);
        let temp_freezing = *scalars.get(12).unwrap_or(&0.0);
        let temp_cold = *scalars.get(13).unwrap_or(&0.25);
        let flags = flag_inputs.as_slice();
        let has_tile_temp = flags.first().copied().unwrap_or(0) != 0;
        let needs_expansion_enabled = flags.get(1).copied().unwrap_or(0) != 0;

        let out = body::needs_base_decay_step(
            hunger_value,
            hunger_decay_rate,
            hunger_stage_mult,
            hunger_metabolic_min,
            hunger_metabolic_range,
            energy_decay_rate,
            social_decay_rate,
            safety_decay_rate,
            thirst_base_decay,
            warmth_base_decay,
            tile_temp,
            has_tile_temp,
            temp_neutral,
            temp_freezing,
            temp_cold,
            needs_expansion_enabled,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_needs_critical_severity_step_packed(
        &self,
        scalar_inputs: PackedFloat32Array,
    ) -> PackedFloat32Array {
        let scalars = scalar_inputs.as_slice();
        let thirst = *scalars.first().unwrap_or(&0.0);
        let warmth = *scalars.get(1).unwrap_or(&0.0);
        let safety = *scalars.get(2).unwrap_or(&0.0);
        let thirst_critical = *scalars.get(3).unwrap_or(&0.0);
        let warmth_critical = *scalars.get(4).unwrap_or(&0.0);
        let safety_critical = *scalars.get(5).unwrap_or(&0.0);
        let out = body::needs_critical_severity_step(
            thirst,
            warmth,
            safety,
            thirst_critical,
            warmth_critical,
            safety_critical,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_needs_critical_severity_step(
        &self,
        thirst: f32,
        warmth: f32,
        safety: f32,
        thirst_critical: f32,
        warmth_critical: f32,
        safety_critical: f32,
    ) -> PackedFloat32Array {
        let scalar_inputs = vec_f32_to_packed(vec![
            thirst,
            warmth,
            safety,
            thirst_critical,
            warmth_critical,
            safety_critical,
        ]);
        self.body_needs_critical_severity_step_packed(scalar_inputs)
    }

    #[func]
    fn body_erg_frustration_step_packed(
        &self,
        scalar_inputs: PackedFloat32Array,
        flag_inputs: PackedByteArray,
    ) -> PackedInt32Array {
        let scalars = scalar_inputs.as_slice();
        let competence = *scalars.first().unwrap_or(&0.0);
        let autonomy = *scalars.get(1).unwrap_or(&0.0);
        let self_actualization = *scalars.get(2).unwrap_or(&0.0);
        let belonging = *scalars.get(3).unwrap_or(&0.0);
        let intimacy = *scalars.get(4).unwrap_or(&0.0);
        let growth_threshold = *scalars.get(5).unwrap_or(&0.0);
        let relatedness_threshold = *scalars.get(6).unwrap_or(&0.0);
        let frustration_window = scalars.get(7).copied().unwrap_or(0.0).round() as i32;
        let growth_ticks = scalars.get(8).copied().unwrap_or(0.0).round() as i32;
        let relatedness_ticks = scalars.get(9).copied().unwrap_or(0.0).round() as i32;
        let flags = flag_inputs.as_slice();
        let was_regressing_growth = flags.first().copied().unwrap_or(0) != 0;
        let was_regressing_relatedness = flags.get(1).copied().unwrap_or(0) != 0;

        let out = body::erg_frustration_step(
            competence,
            autonomy,
            self_actualization,
            belonging,
            intimacy,
            growth_threshold,
            relatedness_threshold,
            frustration_window,
            growth_ticks,
            relatedness_ticks,
            was_regressing_growth,
            was_regressing_relatedness,
        );
        vec_i32_to_packed(out.to_vec())
    }

    #[func]
    fn body_anxious_attachment_stress_delta(
        &self,
        social: f32,
        social_threshold: f32,
        stress_rate: f32,
    ) -> f32 {
        body::anxious_attachment_stress_delta(social, social_threshold, stress_rate)
    }

    #[func]
    fn body_upper_needs_best_skill_normalized(
        &self,
        skill_levels: PackedInt32Array,
        max_level: i32,
    ) -> f32 {
        body::upper_needs_best_skill_normalized(skill_levels.as_slice(), max_level)
    }

    #[func]
    fn body_occupation_best_skill_index(&self, skill_levels: PackedInt32Array) -> i32 {
        body::occupation_best_skill_index(skill_levels.as_slice())
    }

    #[func]
    fn body_occupation_should_switch(
        &self,
        best_skill_level: i32,
        current_skill_level: i32,
        change_hysteresis: f32,
    ) -> bool {
        body::occupation_should_switch(best_skill_level, current_skill_level, change_hysteresis)
    }

    #[func]
    fn body_job_assignment_best_job_code(
        &self,
        ratios: PackedFloat32Array,
        counts: PackedInt32Array,
        alive_count: i32,
    ) -> i32 {
        body::job_assignment_best_job_code(ratios.as_slice(), counts.as_slice(), alive_count)
    }

    #[func]
    fn body_job_assignment_rebalance_codes(
        &self,
        ratios: PackedFloat32Array,
        counts: PackedInt32Array,
        alive_count: i32,
        threshold: f32,
    ) -> PackedInt32Array {
        let out = body::job_assignment_rebalance_codes(
            ratios.as_slice(),
            counts.as_slice(),
            alive_count,
            threshold,
        );
        vec_i32_to_packed(out.to_vec())
    }

    #[func]
    fn body_stat_threshold_is_active(
        &self,
        value: i32,
        threshold: i32,
        direction_code: i32,
        hysteresis: i32,
        currently_active: bool,
    ) -> bool {
        body::stat_threshold_is_active(
            value,
            threshold,
            direction_code,
            hysteresis,
            currently_active,
        )
    }

    #[func]
    fn body_stats_resource_deltas_per_100(
        &self,
        latest_food: f32,
        latest_wood: f32,
        latest_stone: f32,
        older_food: f32,
        older_wood: f32,
        older_stone: f32,
        tick_diff: f32,
    ) -> PackedFloat32Array {
        let out = body::stats_resource_deltas_per_100(
            latest_food,
            latest_wood,
            latest_stone,
            older_food,
            older_wood,
            older_stone,
            tick_diff,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_personality_linear_target(
        &self,
        age: i32,
        max_shift: f32,
        start_age: i32,
        end_age: i32,
    ) -> f32 {
        body::personality_linear_target(age, max_shift, start_age, end_age)
    }

    #[func]
    fn body_intelligence_effective_value(
        &self,
        potential: f32,
        base_mod: f32,
        age_years: f32,
        is_fluid: bool,
        activity_mod: f32,
        ace_fluid_mult: f32,
        env_penalty: f32,
        min_val: f32,
        max_val: f32,
    ) -> f32 {
        body::intelligence_effective_value(
            potential,
            base_mod,
            age_years,
            is_fluid,
            activity_mod,
            ace_fluid_mult,
            env_penalty,
            min_val,
            max_val,
        )
    }

    #[func]
    fn body_intelligence_g_value(
        &self,
        has_parents: bool,
        parent_a_g: f32,
        parent_b_g: f32,
        heritability_g: f32,
        g_mean: f32,
        openness_mean: f32,
        openness_weight: f32,
        noise: f32,
    ) -> f32 {
        body::intelligence_g_value(
            has_parents,
            parent_a_g,
            parent_b_g,
            heritability_g,
            g_mean,
            openness_mean,
            openness_weight,
            noise,
        )
    }

    #[func]
    fn body_personality_child_axis_z(
        &self,
        has_parents: bool,
        parent_a_axis: f32,
        parent_b_axis: f32,
        heritability: f32,
        random_axis_z: f32,
        is_female: bool,
        sex_diff_d: f32,
        culture_shift: f32,
    ) -> f32 {
        body::personality_child_axis_z(
            has_parents,
            parent_a_axis,
            parent_b_axis,
            heritability,
            random_axis_z,
            is_female,
            sex_diff_d,
            culture_shift,
        )
    }

    #[func]
    fn body_morale_behavior_weight_multiplier(
        &self,
        morale: f32,
        flourishing_threshold: f32,
        flourishing_min: f32,
        flourishing_max: f32,
        normal_min: f32,
        normal_max: f32,
        dissatisfied_min: f32,
        dissatisfied_max: f32,
        languishing_min: f32,
        languishing_max: f32,
    ) -> f32 {
        body::morale_behavior_weight_multiplier(
            morale,
            flourishing_threshold,
            flourishing_min,
            flourishing_max,
            normal_min,
            normal_max,
            dissatisfied_min,
            dissatisfied_max,
            languishing_min,
            languishing_max,
        )
    }

    #[func]
    fn body_morale_migration_probability(
        &self,
        morale_s: f32,
        k: f32,
        threshold_morale: f32,
        patience: f32,
        patience_resistance: f32,
        max_probability: f32,
    ) -> f32 {
        body::morale_migration_probability(
            morale_s,
            k,
            threshold_morale,
            patience,
            patience_resistance,
            max_probability,
        )
    }

    #[func]
    fn body_stat_sync_derived_scores(&self, inputs: PackedFloat32Array) -> PackedFloat32Array {
        let v = packed_f32_to_vec(&inputs);
        let out = body::stat_sync_derived_scores(&v);
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_contagion_aoe_total_susceptibility(
        &self,
        donor_count: i32,
        crowd_dilute_divisor: f32,
        refractory_active: bool,
        refractory_susceptibility: f32,
        x_axis: f32,
        e_axis: f32,
    ) -> f32 {
        body::contagion_aoe_total_susceptibility(
            donor_count,
            crowd_dilute_divisor,
            refractory_active,
            refractory_susceptibility,
            x_axis,
            e_axis,
        )
    }

    #[func]
    fn body_contagion_stress_delta(
        &self,
        stress_gap: f32,
        stress_gap_threshold: f32,
        transfer_rate: f32,
        total_susceptibility: f32,
        max_delta: f32,
    ) -> f32 {
        body::contagion_stress_delta(
            stress_gap,
            stress_gap_threshold,
            transfer_rate,
            total_susceptibility,
            max_delta,
        )
    }

    #[func]
    fn body_contagion_network_delta(
        &self,
        donor_count: i32,
        crowd_dilute_divisor: f32,
        refractory_active: bool,
        refractory_susceptibility: f32,
        network_decay: f32,
        a_axis: f32,
        valence_gap: f32,
        delta_scale: f32,
        max_abs_delta: f32,
    ) -> f32 {
        body::contagion_network_delta(
            donor_count,
            crowd_dilute_divisor,
            refractory_active,
            refractory_susceptibility,
            network_decay,
            a_axis,
            valence_gap,
            delta_scale,
            max_abs_delta,
        )
    }

    #[func]
    fn body_contagion_spiral_increment(
        &self,
        stress: f32,
        valence: f32,
        stress_threshold: f32,
        valence_threshold: f32,
        stress_divisor: f32,
        valence_divisor: f32,
        intensity_scale: f32,
        max_increment: f32,
    ) -> f32 {
        body::contagion_spiral_increment(
            stress,
            valence,
            stress_threshold,
            valence_threshold,
            stress_divisor,
            valence_divisor,
            intensity_scale,
            max_increment,
        )
    }

    #[func]
    fn body_mental_break_threshold(
        &self,
        base_break_threshold: f32,
        resilience: f32,
        c_axis: f32,
        e_axis: f32,
        allostatic: f32,
        energy_norm: f32,
        hunger_norm: f32,
        ace_break_threshold_mult: f32,
        trait_break_threshold_add: f32,
        threshold_min: f32,
        threshold_max: f32,
        reserve: f32,
        scar_threshold_reduction: f32,
    ) -> f32 {
        body::mental_break_threshold(
            base_break_threshold,
            resilience,
            c_axis,
            e_axis,
            allostatic,
            energy_norm,
            hunger_norm,
            ace_break_threshold_mult,
            trait_break_threshold_add,
            threshold_min,
            threshold_max,
            reserve,
            scar_threshold_reduction,
        )
    }

    #[func]
    fn body_mental_break_chance(
        &self,
        stress: f32,
        threshold: f32,
        reserve: f32,
        allostatic: f32,
        break_scale: f32,
        break_cap_per_tick: f32,
    ) -> f32 {
        body::mental_break_chance(
            stress,
            threshold,
            reserve,
            allostatic,
            break_scale,
            break_cap_per_tick,
        )
    }

    #[func]
    fn body_trait_violation_context_modifier(
        &self,
        is_habit: bool,
        forced_by_authority: bool,
        survival_necessity: bool,
        no_witness: bool,
        repeated_habit_modifier: f32,
        forced_modifier: f32,
        survival_modifier: f32,
        no_witness_modifier: f32,
    ) -> f32 {
        body::trait_violation_context_modifier(
            is_habit,
            forced_by_authority,
            survival_necessity,
            no_witness,
            repeated_habit_modifier,
            forced_modifier,
            survival_modifier,
            no_witness_modifier,
        )
    }

    #[func]
    fn body_trait_violation_facet_scale(&self, facet_value: f32, threshold: f32) -> f32 {
        body::trait_violation_facet_scale(facet_value, threshold)
    }

    #[func]
    fn body_trait_violation_intrusive_chance(
        &self,
        base_chance: f32,
        ptsd_mult: f32,
        ticks_since: i32,
        history_decay_ticks: i32,
        has_trauma_scars: bool,
    ) -> f32 {
        body::trait_violation_intrusive_chance(
            base_chance,
            ptsd_mult,
            ticks_since,
            history_decay_ticks,
            has_trauma_scars,
        )
    }

    #[func]
    fn body_trauma_scar_acquire_chance(
        &self,
        base_chance: f32,
        chance_scale: f32,
        existing_stacks: i32,
        kindling_factor: f32,
    ) -> f32 {
        body::trauma_scar_acquire_chance(
            base_chance,
            chance_scale,
            existing_stacks,
            kindling_factor,
        )
    }

    #[func]
    fn body_trauma_scar_sensitivity_factor(&self, base_mult: f32, stacks: i32) -> f32 {
        body::trauma_scar_sensitivity_factor(base_mult, stacks)
    }

    #[func]
    fn body_memory_decay_batch(
        &self,
        intensities: PackedFloat32Array,
        rates: PackedFloat32Array,
        dt_years: f32,
    ) -> PackedFloat32Array {
        let intensity_vec = packed_f32_to_vec(&intensities);
        let rate_vec = packed_f32_to_vec(&rates);
        let out = body::memory_decay_batch(&intensity_vec, &rate_vec, dt_years);
        vec_f32_to_packed(out)
    }

    #[func]
    fn body_memory_summary_intensity(&self, max_intensity: f32, summary_scale: f32) -> f32 {
        body::memory_summary_intensity(max_intensity, summary_scale)
    }

    #[func]
    fn body_attachment_type_code(
        &self,
        sensitivity: f32,
        consistency: f32,
        ace_score: f32,
        abuser_is_caregiver: bool,
        sensitivity_threshold_secure: f32,
        consistency_threshold_secure: f32,
        sensitivity_threshold_anxious: f32,
        consistency_threshold_disorganized: f32,
        abuser_is_caregiver_ace_min: f32,
        avoidant_sensitivity_max: f32,
        avoidant_consistency_min: f32,
    ) -> i32 {
        body::attachment_type_code(
            sensitivity,
            consistency,
            ace_score,
            abuser_is_caregiver,
            sensitivity_threshold_secure,
            consistency_threshold_secure,
            sensitivity_threshold_anxious,
            consistency_threshold_disorganized,
            abuser_is_caregiver_ace_min,
            avoidant_sensitivity_max,
            avoidant_consistency_min,
        )
    }

    #[func]
    fn body_attachment_raw_parenting_quality(
        &self,
        has_personality: bool,
        a_axis: f32,
        e_axis: f32,
        has_emotion_data: bool,
        stress: f32,
        allostatic: f32,
        has_active_break: bool,
        ace_score: f32,
    ) -> f32 {
        body::attachment_raw_parenting_quality(
            has_personality,
            a_axis,
            e_axis,
            has_emotion_data,
            stress,
            allostatic,
            has_active_break,
            ace_score,
        )
    }

    #[func]
    fn body_attachment_coping_quality_step(
        &self,
        base_quality: f32,
        dependency: f32,
        neglect_chance: f32,
        consistency_penalty: f32,
    ) -> PackedFloat32Array {
        let out = body::attachment_coping_quality_step(
            base_quality,
            dependency,
            neglect_chance,
            consistency_penalty,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_attachment_protective_factor(
        &self,
        is_secure: bool,
        eh: f32,
        secure_weight: f32,
        eh_weight: f32,
        max_pf: f32,
    ) -> f32 {
        body::attachment_protective_factor(is_secure, eh, secure_weight, eh_weight, max_pf)
    }

    #[func]
    fn body_intergen_scar_index(&self, scar_count: i32, norm_divisor: f32) -> f32 {
        body::intergen_scar_index(scar_count, norm_divisor)
    }

    #[func]
    fn body_intergen_child_epigenetic_step(
        &self,
        inputs: PackedFloat32Array,
    ) -> PackedFloat32Array {
        let v = packed_f32_to_vec(&inputs);
        let out = body::intergen_child_epigenetic_step(&v);
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_intergen_hpa_sensitivity(&self, epigenetic_load: f32, hpa_load_weight: f32) -> f32 {
        body::intergen_hpa_sensitivity(epigenetic_load, hpa_load_weight)
    }

    #[func]
    fn body_intergen_meaney_repair_load(
        &self,
        current_load: f32,
        parenting_quality: f32,
        threshold: f32,
        repair_rate: f32,
        min_load: f32,
    ) -> f32 {
        body::intergen_meaney_repair_load(
            current_load,
            parenting_quality,
            threshold,
            repair_rate,
            min_load,
        )
    }

    #[func]
    fn body_parenting_hpa_adjusted_stress_gain(
        &self,
        current_stress_mult: f32,
        epigenetic_load: f32,
        hpa_load_weight: f32,
    ) -> f32 {
        body::parenting_hpa_adjusted_stress_gain(
            current_stress_mult,
            epigenetic_load,
            hpa_load_weight,
        )
    }

    #[func]
    fn body_parenting_bandura_base_rate(
        &self,
        base_coeff: f32,
        coping_mult: f32,
        observation_strength: f32,
        is_maladaptive: bool,
        maladaptive_multiplier: f32,
    ) -> f32 {
        body::parenting_bandura_base_rate(
            base_coeff,
            coping_mult,
            observation_strength,
            is_maladaptive,
            maladaptive_multiplier,
        )
    }

    #[func]
    fn body_ace_partial_score_next(
        &self,
        current_partial: f32,
        severity: f32,
        ace_weight: f32,
    ) -> f32 {
        body::ace_partial_score_next(current_partial, severity, ace_weight)
    }

    #[func]
    fn body_ace_score_total_from_partials(&self, partials: PackedFloat32Array) -> f32 {
        body::ace_score_total_from_partials(&packed_f32_to_vec(&partials))
    }

    #[func]
    fn body_ace_threat_deprivation_totals(
        &self,
        partials: PackedFloat32Array,
        type_codes: PackedInt32Array,
    ) -> PackedFloat32Array {
        let partial_vec = packed_f32_to_vec(&partials);
        let code_vec = packed_i32_to_vec(&type_codes);
        let out = body::ace_threat_deprivation_totals(&partial_vec, &code_vec);
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_ace_adult_modifiers_adjusted(
        &self,
        base_stress_gain_mult: f32,
        base_break_threshold_mult: f32,
        base_allostatic_base: f32,
        break_floor: f32,
        protective_factor: f32,
    ) -> PackedFloat32Array {
        let out = body::ace_adult_modifiers_adjusted(
            base_stress_gain_mult,
            base_break_threshold_mult,
            base_allostatic_base,
            break_floor,
            protective_factor,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_ace_backfill_score(
        &self,
        allostatic: f32,
        trauma_count: i32,
        attachment_code: i32,
    ) -> f32 {
        body::ace_backfill_score(allostatic, trauma_count, attachment_code)
    }

    #[func]
    fn body_leader_age_respect(&self, age_years: f32) -> f32 {
        body::leader_age_respect(age_years)
    }

    #[func]
    fn body_leader_score(
        &self,
        charisma: f32,
        wisdom: f32,
        trustworthiness: f32,
        intimidation: f32,
        social_capital: f32,
        age_respect: f32,
        w_charisma: f32,
        w_wisdom: f32,
        w_trustworthiness: f32,
        w_intimidation: f32,
        w_social_capital: f32,
        w_age_respect: f32,
        rep_overall: f32,
    ) -> f32 {
        body::leader_score(
            charisma,
            wisdom,
            trustworthiness,
            intimidation,
            social_capital,
            age_respect,
            w_charisma,
            w_wisdom,
            w_trustworthiness,
            w_intimidation,
            w_social_capital,
            w_age_respect,
            rep_overall,
        )
    }

    #[func]
    fn body_network_social_capital_norm(
        &self,
        strong_count: f32,
        weak_count: f32,
        bridge_count: f32,
        rep_score: f32,
        strong_weight: f32,
        weak_weight: f32,
        bridge_weight: f32,
        rep_weight: f32,
        norm_div: f32,
    ) -> f32 {
        body::network_social_capital_norm(
            strong_count,
            weak_count,
            bridge_count,
            rep_score,
            strong_weight,
            weak_weight,
            bridge_weight,
            rep_weight,
            norm_div,
        )
    }

    #[func]
    fn body_revolution_risk_score(
        &self,
        unhappiness: f32,
        frustration: f32,
        inequality: f32,
        leader_unpopularity: f32,
        independence_ratio: f32,
    ) -> f32 {
        body::revolution_risk_score(
            unhappiness,
            frustration,
            inequality,
            leader_unpopularity,
            independence_ratio,
        )
    }

    #[func]
    fn body_reputation_event_delta(
        &self,
        valence: f32,
        magnitude: f32,
        delta_scale: f32,
        neg_bias: f32,
    ) -> f32 {
        body::reputation_event_delta(valence, magnitude, delta_scale, neg_bias)
    }

    #[func]
    fn body_reputation_decay_value(&self, value: f32, pos_decay: f32, neg_decay: f32) -> f32 {
        body::reputation_decay_value(value, pos_decay, neg_decay)
    }

    #[func]
    fn body_economic_tendencies_step(
        &self,
        scalar_inputs: PackedFloat32Array,
        is_male: bool,
        wealth_generosity_penalty: f32,
    ) -> PackedFloat32Array {
        let scalars = scalar_inputs.as_slice();
        let out = body::economic_tendencies_step(
            *scalars.first().unwrap_or(&0.5),
            *scalars.get(1).unwrap_or(&0.5),
            *scalars.get(2).unwrap_or(&0.5),
            *scalars.get(3).unwrap_or(&0.5),
            *scalars.get(4).unwrap_or(&0.5),
            *scalars.get(5).unwrap_or(&0.5),
            *scalars.get(6).unwrap_or(&0.0),
            *scalars.get(7).unwrap_or(&0.0),
            *scalars.get(8).unwrap_or(&0.0),
            *scalars.get(9).unwrap_or(&0.0),
            *scalars.get(10).unwrap_or(&0.0),
            *scalars.get(11).unwrap_or(&0.0),
            *scalars.get(12).unwrap_or(&0.0),
            *scalars.get(13).unwrap_or(&0.0),
            *scalars.get(14).unwrap_or(&0.0),
            *scalars.get(15).unwrap_or(&0.0),
            *scalars.get(16).unwrap_or(&0.0),
            *scalars.get(17).unwrap_or(&0.0),
            *scalars.get(18).unwrap_or(&0.0),
            *scalars.get(19).unwrap_or(&0.0),
            *scalars.get(20).unwrap_or(&0.0),
            is_male,
            wealth_generosity_penalty,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_stratification_gini(&self, values: PackedFloat32Array) -> f32 {
        body::stratification_gini(values.as_slice())
    }

    #[func]
    fn body_stratification_status_score(&self, scalar_inputs: PackedFloat32Array) -> f32 {
        let scalars = scalar_inputs.as_slice();
        body::stratification_status_score(
            *scalars.first().unwrap_or(&0.0),
            *scalars.get(1).unwrap_or(&0.0),
            *scalars.get(2).unwrap_or(&0.0),
            *scalars.get(3).unwrap_or(&0.0),
            *scalars.get(4).unwrap_or(&0.0),
            *scalars.get(5).unwrap_or(&0.0),
            *scalars.get(6).unwrap_or(&0.0),
            *scalars.get(7).unwrap_or(&0.0),
            *scalars.get(8).unwrap_or(&0.0),
            *scalars.get(9).unwrap_or(&0.0),
        )
    }

    #[func]
    fn body_stratification_wealth_score(
        &self,
        food_days: f32,
        wood_norm: f32,
        stone_norm: f32,
        w_food: f32,
        w_wood: f32,
        w_stone: f32,
    ) -> f32 {
        body::stratification_wealth_score(food_days, wood_norm, stone_norm, w_food, w_wood, w_stone)
    }

    #[func]
    fn body_value_plasticity(&self, age_years: f32) -> f32 {
        body::value_plasticity(age_years)
    }

    #[func]
    fn body_family_newborn_health(
        &self,
        gestation_weeks: i32,
        mother_nutrition: f32,
        mother_age: f32,
        genetics_z: f32,
        tech: f32,
    ) -> f32 {
        body::family_newborn_health(
            gestation_weeks,
            mother_nutrition,
            mother_age,
            genetics_z,
            tech,
        )
    }

    #[func]
    fn body_title_is_elder(&self, age_years: f32, elder_min_age_years: f32) -> bool {
        body::title_is_elder(age_years, elder_min_age_years)
    }

    #[func]
    fn body_title_skill_tier(&self, level: i32, expert_level: i32, master_level: i32) -> i32 {
        body::title_skill_tier(level, expert_level, master_level)
    }

    #[func]
    fn body_social_attachment_affinity_multiplier(&self, a_mult: f32, b_mult: f32) -> f32 {
        body::social_attachment_affinity_multiplier(a_mult, b_mult)
    }

    #[func]
    fn body_social_proposal_accept_prob(&self, romantic_interest: f32, compatibility: f32) -> f32 {
        body::social_proposal_accept_prob(romantic_interest, compatibility)
    }

    #[func]
    fn body_tension_scarcity_pressure(
        &self,
        s1_deficit: bool,
        s2_deficit: bool,
        per_shared_resource: f32,
    ) -> f32 {
        body::tension_scarcity_pressure(s1_deficit, s2_deficit, per_shared_resource)
    }

    #[func]
    fn body_tension_next_value(
        &self,
        current: f32,
        scarcity_pressure: f32,
        decay_per_year: f32,
        dt_years: f32,
    ) -> f32 {
        body::tension_next_value(current, scarcity_pressure, decay_per_year, dt_years)
    }

    #[func]
    fn body_resource_regen_next(&self, current: f32, cap: f32, rate: f32) -> f32 {
        body::resource_regen_next(current, cap, rate)
    }

    #[func]
    fn body_age_body_speed(&self, agi_realized: i32, speed_scale: f32, speed_base: f32) -> f32 {
        body::age_body_speed(agi_realized, speed_scale, speed_base)
    }

    #[func]
    fn body_age_body_strength(&self, str_realized: i32) -> f32 {
        body::age_body_strength(str_realized)
    }

    #[func]
    #[allow(clippy::too_many_arguments)]
    fn body_tech_discovery_prob(
        &self,
        base: f32,
        pop_bonus: f32,
        knowledge_bonus: f32,
        openness_bonus: f32,
        logical_bonus: f32,
        naturalistic_bonus: f32,
        soft_bonus: f32,
        rediscovery_bonus: f32,
        max_bonus: f32,
        checks_per_year: f32,
    ) -> f32 {
        body::tech_discovery_prob(
            base,
            pop_bonus,
            knowledge_bonus,
            openness_bonus,
            logical_bonus,
            naturalistic_bonus,
            soft_bonus,
            rediscovery_bonus,
            max_bonus,
            checks_per_year,
        )
    }

    #[func]
    fn body_migration_food_scarce(
        &self,
        nearby_food: f32,
        population: i32,
        per_capita_threshold: f32,
    ) -> bool {
        body::migration_food_scarce(nearby_food, population, per_capita_threshold)
    }

    #[func]
    fn body_migration_should_attempt(
        &self,
        overcrowded: bool,
        food_scarce: bool,
        chance_roll: f32,
        migration_chance: f32,
    ) -> bool {
        body::migration_should_attempt(overcrowded, food_scarce, chance_roll, migration_chance)
    }

    #[func]
    fn body_population_housing_cap(
        &self,
        total_shelters: i32,
        free_population_cap: i32,
        shelter_capacity_per_building: i32,
    ) -> i32 {
        body::population_housing_cap(
            total_shelters,
            free_population_cap,
            shelter_capacity_per_building,
        )
    }

    #[func]
    #[allow(clippy::too_many_arguments)]
    fn body_population_birth_block_code(
        &self,
        alive_count: i32,
        max_entities: i32,
        total_shelters: i32,
        total_food: f32,
        min_population: i32,
        free_population_cap: i32,
        shelter_capacity_per_building: i32,
        food_per_alive: f32,
    ) -> i32 {
        body::population_birth_block_code(
            alive_count,
            max_entities,
            total_shelters,
            total_food,
            min_population,
            free_population_cap,
            shelter_capacity_per_building,
            food_per_alive,
        )
    }

    #[func]
    fn body_chronicle_should_prune(
        &self,
        current_year: i32,
        last_prune_year: i32,
        prune_interval_years: i32,
    ) -> bool {
        body::chronicle_should_prune(current_year, last_prune_year, prune_interval_years)
    }

    #[func]
    fn body_chronicle_cutoff_tick(
        &self,
        current_year: i32,
        max_age_years: i32,
        ticks_per_year: i32,
    ) -> i32 {
        body::chronicle_cutoff_tick(current_year, max_age_years, ticks_per_year)
    }

    #[func]
    fn body_chronicle_keep_world_event(
        &self,
        event_tick: i32,
        importance: i32,
        low_cutoff_tick: i32,
        med_cutoff_tick: i32,
    ) -> bool {
        body::chronicle_keep_world_event(event_tick, importance, low_cutoff_tick, med_cutoff_tick)
    }

    #[func]
    fn body_chronicle_keep_personal_event(
        &self,
        has_valid_world_tick: bool,
        importance: i32,
    ) -> bool {
        body::chronicle_keep_personal_event(has_valid_world_tick, importance)
    }

    #[func]
    fn body_psychology_break_type_code(&self, break_type: GString) -> i32 {
        body::psychology_break_type_code(&break_type.to_string())
    }

    #[func]
    fn body_psychology_break_type_label(&self, code: i32) -> GString {
        GString::from(body::psychology_break_type_label(code))
    }

    #[func]
    fn body_coping_learn_probability(
        &self,
        stress: f32,
        allostatic: f32,
        is_recovery: bool,
        break_count: i32,
        owned_count: i32,
        coping_count_max: f32,
    ) -> f32 {
        body::coping_learn_probability(
            stress,
            allostatic,
            is_recovery,
            break_count,
            owned_count,
            coping_count_max,
        )
    }

    #[func]
    fn body_coping_softmax_index(&self, scores: PackedFloat32Array, roll01: f32) -> i32 {
        body::coping_softmax_index(&packed_f32_to_vec(&scores), roll01)
    }

    #[func]
    fn body_emotion_break_threshold(&self, z_c: f32, base_threshold: f32, z_scale: f32) -> f32 {
        body::emotion_break_threshold(z_c, base_threshold, z_scale)
    }

    #[func]
    fn body_emotion_break_trigger_probability(
        &self,
        stress: f32,
        threshold: f32,
        beta: f32,
        tick_prob: f32,
    ) -> f32 {
        body::emotion_break_trigger_probability(stress, threshold, beta, tick_prob)
    }

    #[func]
    fn body_emotion_break_type_code(
        &self,
        outrage: f32,
        fear: f32,
        anger: f32,
        sadness: f32,
        disgust: f32,
        outrage_threshold: f32,
    ) -> i32 {
        body::emotion_break_type_code(outrage, fear, anger, sadness, disgust, outrage_threshold)
    }

    #[func]
    fn body_emotion_adjusted_half_life(&self, base_half_life: f32, coeff: f32, z: f32) -> f32 {
        body::emotion_adjusted_half_life(base_half_life, coeff, z)
    }

    #[func]
    fn body_emotion_baseline_value(
        &self,
        base_value: f32,
        scale_value: f32,
        z: f32,
        min_value: f32,
        max_value: f32,
    ) -> f32 {
        body::emotion_baseline_value(base_value, scale_value, z, min_value, max_value)
    }

    #[func]
    fn body_emotion_habituation_factor(&self, eta: f32, repeat_count: i32) -> f32 {
        body::emotion_habituation_factor(eta, repeat_count)
    }

    #[func]
    fn body_emotion_contagion_susceptibility(&self, z_e: f32, z_a: f32) -> f32 {
        body::emotion_contagion_susceptibility(z_e, z_a)
    }

    #[func]
    fn body_emotion_contagion_distance_factor(&self, distance: f32, distance_scale: f32) -> f32 {
        body::emotion_contagion_distance_factor(distance, distance_scale)
    }

    #[func]
    fn body_emotion_event_impulse_from_appraisal(
        &self,
        inputs: PackedFloat32Array,
    ) -> PackedFloat32Array {
        let out = body::emotion_event_impulse_from_appraisal(&packed_f32_to_vec(&inputs));
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_emotion_event_impulse_batch(
        &self,
        flat_inputs: PackedFloat32Array,
    ) -> PackedFloat32Array {
        let out = body::emotion_event_impulse_batch(&packed_f32_to_vec(&flat_inputs));
        vec_f32_to_packed(out)
    }

    #[func]
    fn body_tech_cultural_memory_decay(
        &self,
        current_memory: f32,
        base_decay: f32,
        forgotten_long_multiplier: f32,
        memory_floor: f32,
        forgotten_recent: bool,
    ) -> f32 {
        body::tech_cultural_memory_decay(
            current_memory,
            base_decay,
            forgotten_long_multiplier,
            memory_floor,
            forgotten_recent,
        )
    }

    #[func]
    fn body_tech_modifier_stack_clamp(
        &self,
        multiplier_product: f32,
        additive_sum: f32,
        multiplier_cap: f32,
        additive_cap: f32,
    ) -> PackedFloat32Array {
        let out = body::tech_modifier_stack_clamp(
            multiplier_product,
            additive_sum,
            multiplier_cap,
            additive_cap,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_movement_should_skip_tick(&self, skip_mod: i32, tick: i32, entity_id: i32) -> bool {
        body::movement_should_skip_tick(skip_mod, tick, entity_id)
    }

    #[func]
    fn body_building_campfire_social_boost(
        &self,
        is_night: bool,
        day_boost: f32,
        night_boost: f32,
    ) -> f32 {
        body::building_campfire_social_boost(is_night, day_boost, night_boost)
    }

    #[func]
    fn body_building_add_capped(&self, current: f32, delta: f32, cap: f32) -> f32 {
        body::building_add_capped(current, delta, cap)
    }

    #[func]
    fn body_childcare_take_food(&self, available: f32, remaining: f32) -> f32 {
        body::childcare_take_food(available, remaining)
    }

    #[func]
    fn body_childcare_hunger_after(
        &self,
        current_hunger: f32,
        withdrawn: f32,
        food_hunger_restore: f32,
    ) -> f32 {
        body::childcare_hunger_after(current_hunger, withdrawn, food_hunger_restore)
    }

    #[func]
    fn body_tech_propagation_culture_modifier(
        &self,
        knowledge_avg: f32,
        tradition_avg: f32,
        knowledge_weight: f32,
        tradition_weight: f32,
        min_mod: f32,
        max_mod: f32,
    ) -> f32 {
        body::tech_propagation_culture_modifier(
            knowledge_avg,
            tradition_avg,
            knowledge_weight,
            tradition_weight,
            min_mod,
            max_mod,
        )
    }

    #[func]
    fn body_tech_propagation_carrier_bonus(
        &self,
        max_skill: i32,
        skill_divisor: f32,
        weight: f32,
    ) -> f32 {
        body::tech_propagation_carrier_bonus(max_skill, skill_divisor, weight)
    }

    #[func]
    fn body_tech_propagation_final_prob(
        &self,
        base_prob: f32,
        lang_penalty: f32,
        culture_mod: f32,
        carrier_bonus: f32,
        stability_bonus: f32,
        max_prob: f32,
    ) -> f32 {
        body::tech_propagation_final_prob(
            base_prob,
            lang_penalty,
            culture_mod,
            carrier_bonus,
            stability_bonus,
            max_prob,
        )
    }

    #[func]
    fn body_mortality_hazards_and_prob(
        &self,
        model_inputs: PackedFloat32Array,
        env_inputs: PackedFloat32Array,
        is_monthly: bool,
    ) -> PackedFloat32Array {
        let m = packed_f32_to_vec(&model_inputs);
        let e = packed_f32_to_vec(&env_inputs);
        if m.len() < 10 || e.len() < 8 {
            return PackedFloat32Array::new();
        }
        let out = body::mortality_hazards_and_prob(
            m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7], m[8], m[9], e[0], e[1], e[2], e[3],
            e[4], e[5], e[6], e[7], is_monthly,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_cognition_activity_modifier(
        &self,
        active_skill_count: i32,
        activity_buffer: f32,
        inactivity_accel: f32,
    ) -> f32 {
        body::cognition_activity_modifier(active_skill_count, activity_buffer, inactivity_accel)
    }

    #[func]
    fn body_cognition_ace_fluid_decline_mult(
        &self,
        ace_penalty: f32,
        ace_penalty_minor: f32,
        ace_fluid_decline_mult: f32,
    ) -> f32 {
        body::cognition_ace_fluid_decline_mult(
            ace_penalty,
            ace_penalty_minor,
            ace_fluid_decline_mult,
        )
    }

    #[func]
    fn body_upper_needs_job_alignment(
        &self,
        job_code: i32,
        craftsmanship: f32,
        skill: f32,
        hard_work: f32,
        nature: f32,
        independence: f32,
    ) -> f32 {
        body::upper_needs_job_alignment(
            job_code,
            craftsmanship,
            skill,
            hard_work,
            nature,
            independence,
        )
    }

    #[func]
    fn body_job_satisfaction_score(
        &self,
        personality_actual: PackedFloat32Array,
        personality_ideal: PackedFloat32Array,
        value_actual: PackedFloat32Array,
        value_weights: PackedFloat32Array,
        skill_fit: f32,
        autonomy: f32,
        competence: f32,
        meaning: f32,
        autonomy_level: f32,
        prestige: f32,
        w_skill_fit: f32,
        w_value_fit: f32,
        w_personality_fit: f32,
        w_need_fit: f32,
    ) -> f32 {
        body::job_satisfaction_score(
            personality_actual.as_slice(),
            personality_ideal.as_slice(),
            value_actual.as_slice(),
            value_weights.as_slice(),
            skill_fit,
            autonomy,
            competence,
            meaning,
            autonomy_level,
            prestige,
            w_skill_fit,
            w_value_fit,
            w_personality_fit,
            w_need_fit,
        )
    }

    #[func]
    fn body_job_satisfaction_score_batch(
        &self,
        personality_actual: PackedFloat32Array,
        personality_ideals_flat: PackedFloat32Array,
        value_actual: PackedFloat32Array,
        value_weights_flat: PackedFloat32Array,
        skill_fits: PackedFloat32Array,
        autonomy: f32,
        competence: f32,
        meaning: f32,
        autonomy_levels: PackedFloat32Array,
        prestiges: PackedFloat32Array,
        w_skill_fit: f32,
        w_value_fit: f32,
        w_personality_fit: f32,
        w_need_fit: f32,
    ) -> PackedFloat32Array {
        let out = body::job_satisfaction_score_batch(
            personality_actual.as_slice(),
            personality_ideals_flat.as_slice(),
            value_actual.as_slice(),
            value_weights_flat.as_slice(),
            skill_fits.as_slice(),
            autonomy,
            competence,
            meaning,
            autonomy_levels.as_slice(),
            prestiges.as_slice(),
            w_skill_fit,
            w_value_fit,
            w_personality_fit,
            w_need_fit,
        );
        vec_f32_to_packed(out)
    }

    #[func]
    fn body_upper_needs_step_packed(
        &self,
        scalar_inputs: PackedFloat32Array,
        flag_inputs: PackedByteArray,
    ) -> PackedFloat32Array {
        let scalars = scalar_inputs.as_slice();
        let current_values = [
            *scalars.first().unwrap_or(&0.0),
            *scalars.get(1).unwrap_or(&0.0),
            *scalars.get(2).unwrap_or(&0.0),
            *scalars.get(3).unwrap_or(&0.0),
            *scalars.get(4).unwrap_or(&0.0),
            *scalars.get(5).unwrap_or(&0.0),
            *scalars.get(6).unwrap_or(&0.0),
            *scalars.get(7).unwrap_or(&0.0),
        ];
        let decay_values = [
            *scalars.get(8).unwrap_or(&0.0),
            *scalars.get(9).unwrap_or(&0.0),
            *scalars.get(10).unwrap_or(&0.0),
            *scalars.get(11).unwrap_or(&0.0),
            *scalars.get(12).unwrap_or(&0.0),
            *scalars.get(13).unwrap_or(&0.0),
            *scalars.get(14).unwrap_or(&0.0),
            *scalars.get(15).unwrap_or(&0.0),
        ];
        let competence_job_gain = *scalars.get(16).unwrap_or(&0.0);
        let autonomy_job_gain = *scalars.get(17).unwrap_or(&0.0);
        let belonging_settlement_gain = *scalars.get(18).unwrap_or(&0.0);
        let intimacy_partner_gain = *scalars.get(19).unwrap_or(&0.0);
        let recognition_skill_coeff = *scalars.get(20).unwrap_or(&0.0);
        let self_act_skill_coeff = *scalars.get(21).unwrap_or(&0.0);
        let meaning_base_gain = *scalars.get(22).unwrap_or(&0.0);
        let meaning_aligned_gain = *scalars.get(23).unwrap_or(&0.0);
        let transcendence_settlement_gain = *scalars.get(24).unwrap_or(&0.0);
        let transcendence_sacrifice_coeff = *scalars.get(25).unwrap_or(&0.0);
        let best_skill_norm = *scalars.get(26).unwrap_or(&0.0);
        let alignment = *scalars.get(27).unwrap_or(&0.0);
        let sacrifice_value = *scalars.get(28).unwrap_or(&0.0);
        let flags = flag_inputs.as_slice();
        let has_job = flags.first().copied().unwrap_or(0) != 0;
        let has_settlement = flags.get(1).copied().unwrap_or(0) != 0;
        let has_partner = flags.get(2).copied().unwrap_or(0) != 0;

        let out = body::upper_needs_step(
            &current_values,
            &decay_values,
            competence_job_gain,
            autonomy_job_gain,
            belonging_settlement_gain,
            intimacy_partner_gain,
            recognition_skill_coeff,
            self_act_skill_coeff,
            meaning_base_gain,
            meaning_aligned_gain,
            transcendence_settlement_gain,
            transcendence_sacrifice_coeff,
            best_skill_norm,
            alignment,
            sacrifice_value,
            has_job,
            has_settlement,
            has_partner,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_child_parent_stress_transfer(
        &self,
        parent_stress: f32,
        parent_dependency: f32,
        attachment_code: i32,
        caregiver_support_active: bool,
        buffer_power: f32,
        contagion_input: f32,
    ) -> f32 {
        body::child_parent_stress_transfer(
            parent_stress,
            parent_dependency,
            attachment_code,
            caregiver_support_active,
            buffer_power,
            contagion_input,
        )
    }

    #[func]
    fn body_child_simultaneous_ace_step(
        &self,
        prev_residual: f32,
        severities: PackedFloat32Array,
    ) -> PackedFloat32Array {
        let out = body::child_simultaneous_ace_step(severities.as_slice(), prev_residual);
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_child_social_buffered_intensity(
        &self,
        intensity: f32,
        attachment_quality: f32,
        caregiver_present: bool,
        buffer_power: f32,
    ) -> f32 {
        body::child_social_buffered_intensity(
            intensity,
            attachment_quality,
            caregiver_present,
            buffer_power,
        )
    }

    #[func]
    fn body_child_shrp_step(
        &self,
        intensity: f32,
        shrp_active: bool,
        shrp_override_threshold: f32,
        vulnerability_mult: f32,
    ) -> PackedFloat32Array {
        let out = body::child_shrp_step(
            intensity,
            shrp_active,
            shrp_override_threshold,
            vulnerability_mult,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_child_stress_type_code(
        &self,
        intensity: f32,
        attachment_present: bool,
        attachment_quality: f32,
    ) -> i32 {
        body::child_stress_type_code(intensity, attachment_present, attachment_quality)
    }

    #[func]
    fn body_child_stress_apply_step(
        &self,
        resilience: f32,
        reserve: f32,
        stress: f32,
        allostatic: f32,
        intensity: f32,
        spike_mult: f32,
        vulnerability_mult: f32,
        break_threshold_mult: f32,
        stress_type_code: i32,
    ) -> PackedFloat32Array {
        let out = body::child_stress_apply_step(
            resilience,
            reserve,
            stress,
            allostatic,
            intensity,
            spike_mult,
            vulnerability_mult,
            break_threshold_mult,
            stress_type_code,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_child_parent_transfer_apply_step(
        &self,
        current_stress: f32,
        transferred: f32,
        transfer_threshold: f32,
        transfer_scale: f32,
        stress_clamp_max: f32,
    ) -> f32 {
        body::child_parent_transfer_apply_step(
            current_stress,
            transferred,
            transfer_threshold,
            transfer_scale,
            stress_clamp_max,
        )
    }

    #[func]
    fn body_child_deprivation_damage_step(&self, current_damage: f32, damage_rate: f32) -> f32 {
        body::child_deprivation_damage_step(current_damage, damage_rate)
    }

    #[func]
    fn body_child_stage_code_from_age_ticks(
        &self,
        age_ticks: i32,
        infant_max_years: f32,
        toddler_max_years: f32,
        child_max_years: f32,
        teen_max_years: f32,
    ) -> i32 {
        body::child_stage_code_from_age_ticks(
            age_ticks,
            infant_max_years,
            toddler_max_years,
            child_max_years,
            teen_max_years,
        )
    }

    #[func]
    fn body_stress_rebound_apply_step(
        &self,
        stress: f32,
        hidden_threat_accumulator: f32,
        total_rebound: f32,
        stress_clamp_max: f32,
    ) -> PackedFloat32Array {
        let out = body::stress_rebound_apply_step(
            stress,
            hidden_threat_accumulator,
            total_rebound,
            stress_clamp_max,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_stress_injection_apply_step(
        &self,
        stress: f32,
        final_instant: f32,
        final_per_tick: f32,
        trace_threshold: f32,
        stress_clamp_max: f32,
    ) -> PackedFloat32Array {
        let out = body::stress_injection_apply_step(
            stress,
            final_instant,
            final_per_tick,
            trace_threshold,
            stress_clamp_max,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_stress_shaken_countdown_step(&self, shaken_remaining: i32) -> PackedFloat32Array {
        let out = body::stress_shaken_countdown_step(shaken_remaining);
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_stress_support_score(&self, strengths: PackedFloat32Array) -> f32 {
        body::stress_support_score(strengths.as_slice())
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
        let steps = normalize_max_steps(max_steps);
        let backend_mode = get_backend_mode();

        let path = match dispatch_pathfind_grid_bytes(
            backend_mode,
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

        encode_path_vec2(path)
    }

    #[func]
    fn pathfind_grid_xy(
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
    ) -> PackedInt32Array {
        let steps = normalize_max_steps(max_steps);
        let backend_mode = get_backend_mode();

        let path = match dispatch_pathfind_grid_bytes(
            backend_mode,
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
            Err(_) => return PackedInt32Array::new(),
        };

        encode_path_xy(path)
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
        let steps = normalize_max_steps(max_steps);

        let path = match dispatch_pathfind_grid_bytes(
            PATHFIND_BACKEND_GPU,
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

        encode_path_vec2(path)
    }

    #[func]
    fn pathfind_grid_gpu_xy(
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
    ) -> PackedInt32Array {
        let steps = normalize_max_steps(max_steps);

        let path = match dispatch_pathfind_grid_bytes(
            PATHFIND_BACKEND_GPU,
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
            Err(_) => return PackedInt32Array::new(),
        };

        encode_path_xy(path)
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
        let steps = normalize_max_steps(max_steps);
        let backend_mode = get_backend_mode();

        let path_groups = match dispatch_pathfind_grid_batch_vec2_bytes(
            backend_mode,
            width,
            height,
            walkable.as_slice(),
            move_cost.as_slice(),
            from_points.as_slice(),
            to_points.as_slice(),
            steps,
        ) {
            Ok(groups) => groups,
            Err(_) => return Array::new(),
        };

        encode_path_groups_vec2(path_groups)
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
        let steps = normalize_max_steps(max_steps);

        let path_groups = match dispatch_pathfind_grid_batch_vec2_bytes(
            PATHFIND_BACKEND_GPU,
            width,
            height,
            walkable.as_slice(),
            move_cost.as_slice(),
            from_points.as_slice(),
            to_points.as_slice(),
            steps,
        ) {
            Ok(groups) => groups,
            Err(_) => return Array::new(),
        };

        encode_path_groups_vec2(path_groups)
    }

    #[func]
    fn pathfind_grid_batch_xy(
        &self,
        width: i32,
        height: i32,
        walkable: PackedByteArray,
        move_cost: PackedFloat32Array,
        from_xy: PackedInt32Array,
        to_xy: PackedInt32Array,
        max_steps: i32,
    ) -> Array<PackedInt32Array> {
        let steps = normalize_max_steps(max_steps);
        let backend_mode = get_backend_mode();

        let path_groups = match dispatch_pathfind_grid_batch_xy_bytes(
            backend_mode,
            width,
            height,
            walkable.as_slice(),
            move_cost.as_slice(),
            from_xy.as_slice(),
            to_xy.as_slice(),
            steps,
        ) {
            Ok(groups) => groups,
            Err(_) => return Array::new(),
        };

        encode_path_groups_xy(path_groups)
    }

    #[func]
    fn pathfind_grid_gpu_batch_xy(
        &self,
        width: i32,
        height: i32,
        walkable: PackedByteArray,
        move_cost: PackedFloat32Array,
        from_xy: PackedInt32Array,
        to_xy: PackedInt32Array,
        max_steps: i32,
    ) -> Array<PackedInt32Array> {
        let steps = normalize_max_steps(max_steps);

        let path_groups = match dispatch_pathfind_grid_batch_xy_bytes(
            PATHFIND_BACKEND_GPU,
            width,
            height,
            walkable.as_slice(),
            move_cost.as_slice(),
            from_xy.as_slice(),
            to_xy.as_slice(),
            steps,
        ) {
            Ok(groups) => groups,
            Err(_) => return Array::new(),
        };

        encode_path_groups_xy(path_groups)
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
    fn stat_stress_continuous_inputs(
        &self,
        hunger: f32,
        energy: f32,
        social: f32,
        appraisal_scale: f32,
    ) -> VarDictionary {
        let out = stat_curve::stress_continuous_inputs(hunger, energy, social, appraisal_scale);

        let mut dict = VarDictionary::new();
        dict.set("hunger", out.hunger as f64);
        dict.set("energy_deficit", out.energy_deficit as f64);
        dict.set("social_isolation", out.social_isolation as f64);
        dict.set("total", out.total as f64);
        dict
    }

    #[func]
    fn stat_stress_appraisal_scale(
        &self,
        hunger: f32,
        energy: f32,
        social: f32,
        threat: f32,
        conflict: f32,
        support_score: f32,
        extroversion: f32,
        fear_value: f32,
        trust_value: f32,
        conscientiousness: f32,
        openness: f32,
        reserve_ratio: f32,
    ) -> f32 {
        stat_curve::stress_appraisal_scale(
            hunger,
            energy,
            social,
            threat,
            conflict,
            support_score,
            extroversion,
            fear_value,
            trust_value,
            conscientiousness,
            openness,
            reserve_ratio,
        )
    }

    #[func]
    fn stat_stress_primary_step(
        &self,
        hunger: f32,
        energy: f32,
        social: f32,
        threat: f32,
        conflict: f32,
        support_score: f32,
        extroversion: f32,
        fear_value: f32,
        trust_value: f32,
        conscientiousness: f32,
        openness: f32,
        reserve_ratio: f32,
    ) -> VarDictionary {
        let out = stat_curve::stress_primary_step(
            hunger,
            energy,
            social,
            threat,
            conflict,
            support_score,
            extroversion,
            fear_value,
            trust_value,
            conscientiousness,
            openness,
            reserve_ratio,
        );

        let mut dict = VarDictionary::new();
        dict.set("appraisal_scale", out.appraisal_scale as f64);
        dict.set("hunger", out.hunger as f64);
        dict.set("energy_deficit", out.energy_deficit as f64);
        dict.set("social_isolation", out.social_isolation as f64);
        dict.set("total", out.total as f64);
        dict
    }

    #[func]
    fn stat_stress_emotion_contribution(
        &self,
        fear: f32,
        anger: f32,
        sadness: f32,
        disgust: f32,
        surprise: f32,
        joy: f32,
        trust: f32,
        anticipation: f32,
        valence: f32,
        arousal: f32,
    ) -> VarDictionary {
        let out = stat_curve::stress_emotion_contribution(
            fear,
            anger,
            sadness,
            disgust,
            surprise,
            joy,
            trust,
            anticipation,
            valence,
            arousal,
        );

        let mut dict = VarDictionary::new();
        dict.set("fear", out.fear as f64);
        dict.set("anger", out.anger as f64);
        dict.set("sadness", out.sadness as f64);
        dict.set("disgust", out.disgust as f64);
        dict.set("surprise", out.surprise as f64);
        dict.set("joy", out.joy as f64);
        dict.set("trust", out.trust as f64);
        dict.set("anticipation", out.anticipation as f64);
        dict.set("va_composite", out.va_composite as f64);
        dict.set("total", out.total as f64);
        dict
    }

    #[func]
    fn stat_stress_recovery_value(
        &self,
        stress: f32,
        support_score: f32,
        resilience: f32,
        reserve: f32,
        is_sleeping: bool,
        is_safe: bool,
    ) -> f32 {
        stat_curve::stress_recovery_value(
            stress,
            support_score,
            resilience,
            reserve,
            is_sleeping,
            is_safe,
        )
    }

    #[func]
    fn stat_stress_emotion_recovery_delta_step(
        &self,
        emotion_inputs: PackedFloat32Array,
        scalar_inputs: PackedFloat32Array,
        flags: PackedByteArray,
    ) -> VarDictionary {
        let e = emotion_inputs.as_slice();
        let s = scalar_inputs.as_slice();
        let f = flags.as_slice();
        let ef = |idx: usize, fallback: f32| -> f32 { e.get(idx).copied().unwrap_or(fallback) };
        let sf = |idx: usize, fallback: f32| -> f32 { s.get(idx).copied().unwrap_or(fallback) };
        let bf = |idx: usize| -> bool { f.get(idx).copied().unwrap_or(0_u8) != 0_u8 };

        let out = stat_curve::stress_emotion_recovery_delta_step(
            ef(0, 0.0),
            ef(1, 0.0),
            ef(2, 0.0),
            ef(3, 0.0),
            ef(4, 0.0),
            ef(5, 0.0),
            ef(6, 0.0),
            ef(7, 0.0),
            ef(8, 0.0),
            ef(9, 0.0),
            sf(0, 0.0),
            sf(1, 0.3),
            sf(2, 0.5),
            sf(3, 50.0),
            bf(0),
            bf(1),
            sf(4, 0.0),
            sf(5, 0.0),
            sf(6, 1.0),
            sf(7, 1.0),
            sf(8, 0.05),
            bf(2),
            sf(9, 0.6),
            sf(10, 0.0),
            sf(11, 800.0),
        );

        let mut dict = VarDictionary::new();
        dict.set("fear", out.fear as f64);
        dict.set("anger", out.anger as f64);
        dict.set("sadness", out.sadness as f64);
        dict.set("disgust", out.disgust as f64);
        dict.set("surprise", out.surprise as f64);
        dict.set("joy", out.joy as f64);
        dict.set("trust", out.trust as f64);
        dict.set("anticipation", out.anticipation as f64);
        dict.set("va_composite", out.va_composite as f64);
        dict.set("emotion_total", out.emotion_total as f64);
        dict.set("recovery", out.recovery as f64);
        dict.set("delta", out.delta as f64);
        dict.set(
            "hidden_threat_accumulator",
            out.hidden_threat_accumulator as f64,
        );
        dict
    }

    #[func]
    fn stat_stress_trace_emotion_recovery_delta_step(
        &self,
        per_tick: PackedFloat32Array,
        decay_rate: PackedFloat32Array,
        min_keep: f32,
        emotion_inputs: PackedFloat32Array,
        scalar_inputs: PackedFloat32Array,
        flags: PackedByteArray,
    ) -> VarDictionary {
        let e = emotion_inputs.as_slice();
        let s = scalar_inputs.as_slice();
        let f = flags.as_slice();
        let ef = |idx: usize, fallback: f32| -> f32 { e.get(idx).copied().unwrap_or(fallback) };
        let sf = |idx: usize, fallback: f32| -> f32 { s.get(idx).copied().unwrap_or(fallback) };
        let bf = |idx: usize| -> bool { f.get(idx).copied().unwrap_or(0_u8) != 0_u8 };

        let out = stat_curve::stress_trace_emotion_recovery_delta_step(
            per_tick.as_slice(),
            decay_rate.as_slice(),
            min_keep,
            ef(0, 0.0),
            ef(1, 0.0),
            ef(2, 0.0),
            ef(3, 0.0),
            ef(4, 0.0),
            ef(5, 0.0),
            ef(6, 0.0),
            ef(7, 0.0),
            ef(8, 0.0),
            ef(9, 0.0),
            sf(0, 0.0),
            sf(1, 0.3),
            sf(2, 0.5),
            sf(3, 50.0),
            bf(0),
            bf(1),
            sf(4, 0.0),
            sf(5, 1.0),
            sf(6, 1.0),
            sf(7, 0.05),
            bf(2),
            sf(8, 0.6),
            sf(9, 0.0),
            sf(10, 800.0),
        );

        let mut dict = VarDictionary::new();
        dict.set(
            "total_trace_contribution",
            out.total_trace_contribution as f64,
        );
        dict.set("updated_per_tick", vec_f32_to_packed(out.updated_per_tick));
        dict.set("active_mask", vec_u8_to_packed(out.active_mask));
        dict.set("fear", out.fear as f64);
        dict.set("anger", out.anger as f64);
        dict.set("sadness", out.sadness as f64);
        dict.set("disgust", out.disgust as f64);
        dict.set("surprise", out.surprise as f64);
        dict.set("joy", out.joy as f64);
        dict.set("trust", out.trust as f64);
        dict.set("anticipation", out.anticipation as f64);
        dict.set("va_composite", out.va_composite as f64);
        dict.set("emotion_total", out.emotion_total as f64);
        dict.set("recovery", out.recovery as f64);
        dict.set("delta", out.delta as f64);
        dict.set(
            "hidden_threat_accumulator",
            out.hidden_threat_accumulator as f64,
        );
        dict
    }

    #[func]
    fn stat_stress_tick_step(
        &self,
        per_tick: PackedFloat32Array,
        decay_rate: PackedFloat32Array,
        min_keep: f32,
        scalar_inputs: PackedFloat32Array,
        flags: PackedByteArray,
    ) -> VarDictionary {
        let s = scalar_inputs.as_slice();
        let f = flags.as_slice();
        let sf = |idx: usize, fallback: f32| -> f32 { s.get(idx).copied().unwrap_or(fallback) };
        let bf = |idx: usize| -> bool { f.get(idx).copied().unwrap_or(0_u8) != 0_u8 };

        let out = stat_curve::stress_tick_step(
            per_tick.as_slice(),
            decay_rate.as_slice(),
            min_keep,
            sf(0, 0.5),         // hunger
            sf(1, 0.5),         // energy
            sf(2, 0.5),         // social
            sf(3, 0.0),         // threat
            sf(4, 0.0),         // conflict
            sf(5, 0.3),         // support_score
            sf(6, 0.5),         // extroversion
            sf(7, 0.0),         // fear
            sf(8, 0.0),         // trust
            sf(9, 0.5),         // conscientiousness
            sf(10, 0.5),        // openness
            sf(11, 0.5),        // reserve_ratio
            sf(12, 0.0),        // anger
            sf(13, 0.0),        // sadness
            sf(14, 0.0),        // disgust
            sf(15, 0.0),        // surprise
            sf(16, 0.0),        // joy
            sf(17, 0.0),        // anticipation
            sf(18, 0.0),        // valence
            sf(19, 0.0),        // arousal
            sf(20, 0.0),        // stress
            sf(21, 0.5),        // resilience
            sf(22, 50.0),       // reserve
            sf(23, 0.0),        // stress_delta_last
            sf(24, 0.0) as i32, // gas_stage
            bf(0),              // is_sleeping
            bf(1),              // is_safe
            sf(25, 0.0),        // allostatic
            sf(26, 1.0),        // ace_stress_mult
            sf(27, 1.0),        // trait_accum_mult
            sf(28, 0.05),       // epsilon
            bf(2),              // denial_active
            sf(29, 0.6),        // denial_redirect_fraction
            sf(30, 0.0),        // hidden_threat_accumulator
            sf(31, 800.0),      // denial_max_accumulator
            sf(32, 1.0),        // avoidant_allostatic_mult
            sf(33, 0.5),        // e_axis
            sf(34, 0.5),        // c_axis
            sf(35, 0.5),        // x_axis
            sf(36, 0.5),        // o_axis
            sf(37, 0.5),        // a_axis
            sf(38, 0.5),        // h_axis
            sf(39, 0.0),        // scar_resilience_mod
        );

        let mut dict = VarDictionary::new();
        dict.set("appraisal_scale", out.appraisal_scale as f64);
        dict.set("hunger", out.hunger as f64);
        dict.set("energy_deficit", out.energy_deficit as f64);
        dict.set("social_isolation", out.social_isolation as f64);
        dict.set("continuous_total", out.continuous_total as f64);
        dict.set(
            "total_trace_contribution",
            out.total_trace_contribution as f64,
        );
        dict.set("updated_per_tick", vec_f32_to_packed(out.updated_per_tick));
        dict.set("active_mask", vec_u8_to_packed(out.active_mask));
        dict.set("fear", out.fear as f64);
        dict.set("anger", out.anger as f64);
        dict.set("sadness", out.sadness as f64);
        dict.set("disgust", out.disgust as f64);
        dict.set("surprise", out.surprise as f64);
        dict.set("joy", out.joy as f64);
        dict.set("trust", out.trust as f64);
        dict.set("anticipation", out.anticipation as f64);
        dict.set("va_composite", out.va_composite as f64);
        dict.set("emotion_total", out.emotion_total as f64);
        dict.set("recovery", out.recovery as f64);
        dict.set("delta", out.delta as f64);
        dict.set(
            "hidden_threat_accumulator",
            out.hidden_threat_accumulator as f64,
        );
        dict.set("stress", out.stress as f64);
        dict.set("reserve", out.reserve as f64);
        dict.set("gas_stage", out.gas_stage);
        dict.set("allostatic", out.allostatic as f64);
        dict.set("stress_state", out.stress_state);
        dict.set("stress_mu_sadness", out.stress_mu_sadness as f64);
        dict.set("stress_mu_anger", out.stress_mu_anger as f64);
        dict.set("stress_mu_fear", out.stress_mu_fear as f64);
        dict.set("stress_mu_joy", out.stress_mu_joy as f64);
        dict.set("stress_mu_trust", out.stress_mu_trust as f64);
        dict.set("stress_neg_gain_mult", out.stress_neg_gain_mult as f64);
        dict.set("stress_pos_gain_mult", out.stress_pos_gain_mult as f64);
        dict.set("stress_blunt_mult", out.stress_blunt_mult as f64);
        dict.set("resilience", out.resilience as f64);
        dict
    }

    #[func]
    fn stat_stress_tick_step_packed(
        &self,
        per_tick: PackedFloat32Array,
        decay_rate: PackedFloat32Array,
        min_keep: f32,
        scalar_inputs: PackedFloat32Array,
        flags: PackedByteArray,
    ) -> VarDictionary {
        let s = scalar_inputs.as_slice();
        let f = flags.as_slice();
        let sf = |idx: usize, fallback: f32| -> f32 { s.get(idx).copied().unwrap_or(fallback) };
        let bf = |idx: usize| -> bool { f.get(idx).copied().unwrap_or(0_u8) != 0_u8 };

        let out = stat_curve::stress_tick_step(
            per_tick.as_slice(),
            decay_rate.as_slice(),
            min_keep,
            sf(0, 0.5),
            sf(1, 0.5),
            sf(2, 0.5),
            sf(3, 0.0),
            sf(4, 0.0),
            sf(5, 0.3),
            sf(6, 0.5),
            sf(7, 0.0),
            sf(8, 0.0),
            sf(9, 0.5),
            sf(10, 0.5),
            sf(11, 0.5),
            sf(12, 0.0),
            sf(13, 0.0),
            sf(14, 0.0),
            sf(15, 0.0),
            sf(16, 0.0),
            sf(17, 0.0),
            sf(18, 0.0),
            sf(19, 0.0),
            sf(20, 0.0),
            sf(21, 0.5),
            sf(22, 50.0),
            sf(23, 0.0),
            sf(24, 0.0) as i32,
            bf(0),
            bf(1),
            sf(25, 0.0),
            sf(26, 1.0),
            sf(27, 1.0),
            sf(28, 0.05),
            bf(2),
            sf(29, 0.6),
            sf(30, 0.0),
            sf(31, 800.0),
            sf(32, 1.0),
            sf(33, 0.5),
            sf(34, 0.5),
            sf(35, 0.5),
            sf(36, 0.5),
            sf(37, 0.5),
            sf(38, 0.5),
            sf(39, 0.0),
        );

        let scalars: Vec<f32> = vec![
            out.appraisal_scale,
            out.hunger,
            out.energy_deficit,
            out.social_isolation,
            out.total_trace_contribution,
            out.fear,
            out.anger,
            out.sadness,
            out.disgust,
            out.surprise,
            out.joy,
            out.trust,
            out.anticipation,
            out.va_composite,
            out.recovery,
            out.delta,
            out.hidden_threat_accumulator,
            out.stress,
            out.reserve,
            out.allostatic,
            out.resilience,
            out.stress_mu_sadness,
            out.stress_mu_anger,
            out.stress_mu_fear,
            out.stress_mu_joy,
            out.stress_mu_trust,
            out.stress_neg_gain_mult,
            out.stress_pos_gain_mult,
            out.stress_blunt_mult,
            out.continuous_total,
            out.emotion_total,
        ];
        let ints: Vec<i32> = vec![out.gas_stage, out.stress_state];

        let mut dict = VarDictionary::new();
        dict.set("scalars", vec_f32_to_packed(scalars));
        dict.set("ints", vec_i32_to_packed(ints));
        dict.set("updated_per_tick", vec_f32_to_packed(out.updated_per_tick));
        dict.set("active_mask", vec_u8_to_packed(out.active_mask));
        dict
    }

    #[func]
    fn stat_stress_delta_step(
        &self,
        continuous_input: f32,
        trace_input: f32,
        emotion_input: f32,
        ace_stress_mult: f32,
        trait_accum_mult: f32,
        recovery: f32,
        epsilon: f32,
        denial_active: bool,
        denial_redirect_fraction: f32,
        hidden_threat_accumulator: f32,
        denial_max_accumulator: f32,
    ) -> VarDictionary {
        let out = stat_curve::stress_delta_step(
            continuous_input,
            trace_input,
            emotion_input,
            ace_stress_mult,
            trait_accum_mult,
            recovery,
            epsilon,
            denial_active,
            denial_redirect_fraction,
            hidden_threat_accumulator,
            denial_max_accumulator,
        );

        let mut dict = VarDictionary::new();
        dict.set("delta", out.delta as f64);
        dict.set(
            "hidden_threat_accumulator",
            out.hidden_threat_accumulator as f64,
        );
        dict
    }

    #[func]
    fn stat_stress_post_update_step(
        &self,
        reserve: f32,
        stress: f32,
        resilience: f32,
        stress_delta_last: f32,
        gas_stage: i32,
        is_sleeping: bool,
        allostatic: f32,
        avoidant_allostatic_mult: f32,
    ) -> VarDictionary {
        let out = stat_curve::stress_post_update_step(
            reserve,
            stress,
            resilience,
            stress_delta_last,
            gas_stage,
            is_sleeping,
            allostatic,
            avoidant_allostatic_mult,
        );

        let mut dict = VarDictionary::new();
        dict.set("reserve", out.reserve as f64);
        dict.set("gas_stage", out.gas_stage);
        dict.set("allostatic", out.allostatic as f64);
        dict.set("stress_state", out.stress_state);
        dict.set("stress_mu_sadness", out.stress_mu_sadness as f64);
        dict.set("stress_mu_anger", out.stress_mu_anger as f64);
        dict.set("stress_mu_fear", out.stress_mu_fear as f64);
        dict.set("stress_mu_joy", out.stress_mu_joy as f64);
        dict.set("stress_mu_trust", out.stress_mu_trust as f64);
        dict.set("stress_neg_gain_mult", out.stress_neg_gain_mult as f64);
        dict.set("stress_pos_gain_mult", out.stress_pos_gain_mult as f64);
        dict.set("stress_blunt_mult", out.stress_blunt_mult as f64);
        dict
    }

    #[func]
    fn stat_stress_post_update_resilience_step(
        &self,
        scalar_inputs: PackedFloat32Array,
        flags: PackedByteArray,
    ) -> VarDictionary {
        let s = scalar_inputs.as_slice();
        let f = flags.as_slice();
        let sf = |idx: usize, fallback: f32| -> f32 { s.get(idx).copied().unwrap_or(fallback) };
        let bf = |idx: usize| -> bool { f.get(idx).copied().unwrap_or(0_u8) != 0_u8 };

        let out = stat_curve::stress_post_update_resilience_step(
            sf(0, 0.0),        // reserve
            sf(1, 0.0),        // stress
            sf(2, 0.5),        // resilience
            sf(3, 0.0),        // stress_delta_last
            sf(4, 0.0) as i32, // gas_stage
            bf(0),             // is_sleeping
            sf(5, 0.0),        // allostatic
            sf(6, 1.0),        // avoidant_allostatic_mult
            sf(7, 0.5),        // e_axis
            sf(8, 0.5),        // c_axis
            sf(9, 0.5),        // x_axis
            sf(10, 0.5),       // o_axis
            sf(11, 0.5),       // a_axis
            sf(12, 0.5),       // h_axis
            sf(13, 0.3),       // support_score
            sf(14, 0.5),       // hunger
            sf(15, 0.5),       // energy
            sf(16, 0.0),       // scar_resilience_mod
        );

        let mut dict = VarDictionary::new();
        dict.set("reserve", out.reserve as f64);
        dict.set("gas_stage", out.gas_stage);
        dict.set("allostatic", out.allostatic as f64);
        dict.set("stress_state", out.stress_state);
        dict.set("stress_mu_sadness", out.stress_mu_sadness as f64);
        dict.set("stress_mu_anger", out.stress_mu_anger as f64);
        dict.set("stress_mu_fear", out.stress_mu_fear as f64);
        dict.set("stress_mu_joy", out.stress_mu_joy as f64);
        dict.set("stress_mu_trust", out.stress_mu_trust as f64);
        dict.set("stress_neg_gain_mult", out.stress_neg_gain_mult as f64);
        dict.set("stress_pos_gain_mult", out.stress_pos_gain_mult as f64);
        dict.set("stress_blunt_mult", out.stress_blunt_mult as f64);
        dict.set("resilience", out.resilience as f64);
        dict
    }

    #[func]
    fn stat_stress_reserve_step(
        &self,
        reserve: f32,
        stress: f32,
        resilience: f32,
        stress_delta_last: f32,
        gas_stage: i32,
        is_sleeping: bool,
    ) -> VarDictionary {
        let out = stat_curve::stress_reserve_step(
            reserve,
            stress,
            resilience,
            stress_delta_last,
            gas_stage,
            is_sleeping,
        );

        let mut dict = VarDictionary::new();
        dict.set("reserve", out.reserve as f64);
        dict.set("gas_stage", out.gas_stage);
        dict
    }

    #[func]
    fn stat_stress_allostatic_step(
        &self,
        allostatic: f32,
        stress: f32,
        avoidant_allostatic_mult: f32,
    ) -> f32 {
        stat_curve::stress_allostatic_step(allostatic, stress, avoidant_allostatic_mult)
    }

    #[func]
    fn stat_stress_state_snapshot(&self, stress: f32, allostatic: f32) -> VarDictionary {
        let out = stat_curve::stress_state_snapshot(stress, allostatic);
        let mut dict = VarDictionary::new();
        dict.set("stress_state", out.stress_state);
        dict.set("stress_mu_sadness", out.stress_mu_sadness as f64);
        dict.set("stress_mu_anger", out.stress_mu_anger as f64);
        dict.set("stress_mu_fear", out.stress_mu_fear as f64);
        dict.set("stress_mu_joy", out.stress_mu_joy as f64);
        dict.set("stress_mu_trust", out.stress_mu_trust as f64);
        dict.set("stress_neg_gain_mult", out.stress_neg_gain_mult as f64);
        dict.set("stress_pos_gain_mult", out.stress_pos_gain_mult as f64);
        dict.set("stress_blunt_mult", out.stress_blunt_mult as f64);
        dict
    }

    #[func]
    fn stat_stress_trace_batch_step(
        &self,
        per_tick: PackedFloat32Array,
        decay_rate: PackedFloat32Array,
        min_keep: f32,
    ) -> VarDictionary {
        let out = stat_curve::stress_trace_batch_step(
            per_tick.as_slice(),
            decay_rate.as_slice(),
            min_keep,
        );
        let mut dict = VarDictionary::new();
        dict.set("total_contribution", out.total_contribution as f64);
        dict.set("updated_per_tick", vec_f32_to_packed(out.updated_per_tick));
        dict.set("active_mask", vec_u8_to_packed(out.active_mask));
        dict
    }

    #[func]
    fn stat_stress_resilience_value(
        &self,
        e_axis: f32,
        c_axis: f32,
        x_axis: f32,
        o_axis: f32,
        a_axis: f32,
        h_axis: f32,
        support_score: f32,
        allostatic: f32,
        hunger: f32,
        energy: f32,
        scar_resilience_mod: f32,
    ) -> f32 {
        stat_curve::stress_resilience_value(
            e_axis,
            c_axis,
            x_axis,
            o_axis,
            a_axis,
            h_axis,
            support_score,
            allostatic,
            hunger,
            energy,
            scar_resilience_mod,
        )
    }

    #[func]
    fn stat_stress_work_efficiency(&self, stress: f32, shaken_penalty: f32) -> f32 {
        stat_curve::stress_work_efficiency(stress, shaken_penalty)
    }

    #[func]
    fn stat_stress_personality_scale(
        &self,
        values: PackedFloat32Array,
        weights: PackedFloat32Array,
        high_amplifies: PackedByteArray,
        trait_multipliers: PackedFloat32Array,
    ) -> f32 {
        stat_curve::stress_personality_scale(
            &packed_f32_to_vec(&values),
            &packed_f32_to_vec(&weights),
            &packed_u8_to_vec(&high_amplifies),
            &packed_f32_to_vec(&trait_multipliers),
        )
    }

    #[func]
    fn stat_stress_relationship_scale(
        &self,
        method: GString,
        bond_strength: f32,
        min_mult: f32,
        max_mult: f32,
    ) -> f32 {
        let method_string = method.to_string();
        stat_curve::stress_relationship_scale(&method_string, bond_strength, min_mult, max_mult)
    }

    #[func]
    fn stat_stress_context_scale(&self, active_multipliers: PackedFloat32Array) -> f32 {
        stat_curve::stress_context_scale(&packed_f32_to_vec(&active_multipliers))
    }

    #[func]
    fn stat_stress_emotion_inject_step(
        &self,
        fast_current: PackedFloat32Array,
        slow_current: PackedFloat32Array,
        fast_inject: PackedFloat32Array,
        slow_inject: PackedFloat32Array,
        scale: f32,
    ) -> VarDictionary {
        let out = stat_curve::stress_emotion_inject_step(
            &packed_f32_to_vec(&fast_current),
            &packed_f32_to_vec(&slow_current),
            &packed_f32_to_vec(&fast_inject),
            &packed_f32_to_vec(&slow_inject),
            scale,
        );
        let mut dict = VarDictionary::new();
        dict.set("fast", vec_f32_to_packed(out.fast));
        dict.set("slow", vec_f32_to_packed(out.slow));
        dict
    }

    #[func]
    fn stat_stress_rebound_queue_step(
        &self,
        amounts: PackedFloat32Array,
        delays: PackedInt32Array,
        decay_per_tick: f32,
    ) -> VarDictionary {
        let out = stat_curve::stress_rebound_queue_step(
            &packed_f32_to_vec(&amounts),
            &packed_i32_to_vec(&delays),
            decay_per_tick,
        );
        let mut dict = VarDictionary::new();
        dict.set("total_rebound", out.total_rebound as f64);
        dict.set(
            "remaining_amounts",
            vec_f32_to_packed(out.remaining_amounts),
        );
        dict.set("remaining_delays", vec_i32_to_packed(out.remaining_delays));
        dict
    }

    #[func]
    fn stat_stress_event_scale_step(
        &self,
        base_instant: f32,
        base_per_tick: f32,
        is_loss: bool,
        personality_scale: f32,
        appraisal_scale: f32,
        relationship_method: GString,
        bond_strength: f32,
        relationship_min_mult: f32,
        relationship_max_mult: f32,
        context_active_multipliers: PackedFloat32Array,
    ) -> VarDictionary {
        let out = stat_curve::stress_event_scale_step(
            base_instant,
            base_per_tick,
            is_loss,
            personality_scale,
            appraisal_scale,
            &relationship_method.to_string(),
            bond_strength,
            relationship_min_mult,
            relationship_max_mult,
            &packed_f32_to_vec(&context_active_multipliers),
        );
        let mut dict = VarDictionary::new();
        dict.set("relationship_scale", out.relationship_scale as f64);
        dict.set("context_scale", out.context_scale as f64);
        dict.set("total_scale", out.total_scale as f64);
        dict.set("loss_mult", out.loss_mult as f64);
        dict.set("final_instant", out.final_instant as f64);
        dict.set("final_per_tick", out.final_per_tick as f64);
        dict
    }

    #[func]
    fn stat_stress_event_scale_step_code(
        &self,
        base_instant: f32,
        base_per_tick: f32,
        is_loss: bool,
        personality_scale: f32,
        appraisal_scale: f32,
        relationship_method_code: i32,
        bond_strength: f32,
        relationship_min_mult: f32,
        relationship_max_mult: f32,
        context_active_multipliers: PackedFloat32Array,
    ) -> VarDictionary {
        let out = stat_curve::stress_event_scale_step_code(
            base_instant,
            base_per_tick,
            is_loss,
            personality_scale,
            appraisal_scale,
            relationship_method_code,
            bond_strength,
            relationship_min_mult,
            relationship_max_mult,
            &packed_f32_to_vec(&context_active_multipliers),
        );
        let mut dict = VarDictionary::new();
        dict.set("relationship_scale", out.relationship_scale as f64);
        dict.set("context_scale", out.context_scale as f64);
        dict.set("total_scale", out.total_scale as f64);
        dict.set("loss_mult", out.loss_mult as f64);
        dict.set("final_instant", out.final_instant as f64);
        dict.set("final_per_tick", out.final_per_tick as f64);
        dict
    }

    #[func]
    fn stat_stress_event_inject_step(
        &self,
        base_instant: f32,
        base_per_tick: f32,
        is_loss: bool,
        personality_scale: f32,
        appraisal_scale: f32,
        relationship_method: GString,
        bond_strength: f32,
        relationship_min_mult: f32,
        relationship_max_mult: f32,
        context_active_multipliers: PackedFloat32Array,
        fast_current: PackedFloat32Array,
        slow_current: PackedFloat32Array,
        fast_inject: PackedFloat32Array,
        slow_inject: PackedFloat32Array,
    ) -> VarDictionary {
        let out = stat_curve::stress_event_inject_step(
            base_instant,
            base_per_tick,
            is_loss,
            personality_scale,
            appraisal_scale,
            &relationship_method.to_string(),
            bond_strength,
            relationship_min_mult,
            relationship_max_mult,
            &packed_f32_to_vec(&context_active_multipliers),
            &packed_f32_to_vec(&fast_current),
            &packed_f32_to_vec(&slow_current),
            &packed_f32_to_vec(&fast_inject),
            &packed_f32_to_vec(&slow_inject),
        );
        let mut dict = VarDictionary::new();
        dict.set("relationship_scale", out.relationship_scale as f64);
        dict.set("context_scale", out.context_scale as f64);
        dict.set("total_scale", out.total_scale as f64);
        dict.set("loss_mult", out.loss_mult as f64);
        dict.set("final_instant", out.final_instant as f64);
        dict.set("final_per_tick", out.final_per_tick as f64);
        dict.set("fast", vec_f32_to_packed(out.fast));
        dict.set("slow", vec_f32_to_packed(out.slow));
        dict
    }

    #[func]
    fn stat_stress_event_inject_step_code(
        &self,
        base_instant: f32,
        base_per_tick: f32,
        is_loss: bool,
        personality_scale: f32,
        appraisal_scale: f32,
        relationship_method_code: i32,
        bond_strength: f32,
        relationship_min_mult: f32,
        relationship_max_mult: f32,
        context_active_multipliers: PackedFloat32Array,
        fast_current: PackedFloat32Array,
        slow_current: PackedFloat32Array,
        fast_inject: PackedFloat32Array,
        slow_inject: PackedFloat32Array,
    ) -> VarDictionary {
        let out = stat_curve::stress_event_inject_step_code(
            base_instant,
            base_per_tick,
            is_loss,
            personality_scale,
            appraisal_scale,
            relationship_method_code,
            bond_strength,
            relationship_min_mult,
            relationship_max_mult,
            &packed_f32_to_vec(&context_active_multipliers),
            &packed_f32_to_vec(&fast_current),
            &packed_f32_to_vec(&slow_current),
            &packed_f32_to_vec(&fast_inject),
            &packed_f32_to_vec(&slow_inject),
        );
        let mut dict = VarDictionary::new();
        dict.set("relationship_scale", out.relationship_scale as f64);
        dict.set("context_scale", out.context_scale as f64);
        dict.set("total_scale", out.total_scale as f64);
        dict.set("loss_mult", out.loss_mult as f64);
        dict.set("final_instant", out.final_instant as f64);
        dict.set("final_per_tick", out.final_per_tick as f64);
        dict.set("fast", vec_f32_to_packed(out.fast));
        dict.set("slow", vec_f32_to_packed(out.slow));
        dict
    }

    #[func]
    fn stat_stress_event_scaled(
        &self,
        base_instant: f32,
        base_per_tick: f32,
        is_loss: bool,
        personality_scale: f32,
        relationship_scale: f32,
        context_scale: f32,
        appraisal_scale: f32,
    ) -> VarDictionary {
        let out = stat_curve::stress_event_scaled(
            base_instant,
            base_per_tick,
            is_loss,
            personality_scale,
            relationship_scale,
            context_scale,
            appraisal_scale,
        );
        let mut dict = VarDictionary::new();
        dict.set("total_scale", out.total_scale as f64);
        dict.set("loss_mult", out.loss_mult as f64);
        dict.set("final_instant", out.final_instant as f64);
        dict.set("final_per_tick", out.final_per_tick as f64);
        dict
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
    use super::pathfinding_backend::{
        read_dispatch_counts, reset_dispatch_counts, PATHFIND_BACKEND_AUTO, PATHFIND_BACKEND_CPU,
        PATHFIND_BACKEND_GPU,
    };
    use super::{
        decode_ws2_blob, dispatch_pathfind_grid_batch_vec2_bytes,
        dispatch_pathfind_grid_batch_xy_bytes, dispatch_pathfind_grid_bytes, encode_ws2_blob,
        format_fluent_from_source_args, get_pathfind_backend_mode, has_gpu_pathfind_backend,
        parse_pathfind_backend, pathfind_backend_dispatch_counts, pathfind_from_flat,
        pathfind_grid_batch_bytes, pathfind_grid_batch_dispatch_bytes,
        pathfind_grid_batch_vec2_bytes, pathfind_grid_batch_xy_bytes,
        pathfind_grid_batch_xy_dispatch_bytes, pathfind_grid_bytes,
        reset_pathfind_backend_dispatch_counts, resolve_backend_mode,
        resolve_pathfind_backend_mode, runtime_supports_rust_system,
        runtime_system_key_from_name, set_pathfind_backend_mode, PathfindError, PathfindInput,
    };
    use fluent_bundle::types::FluentNumber;
    use fluent_bundle::{FluentArgs, FluentValue};
    use godot::prelude::Vector2;
    use sim_engine::EngineSnapshot;
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
    fn fluent_format_replaces_named_params() {
        let source = "ui-greeting = Hello, { $name }!";
        let mut args = FluentArgs::new();
        args.set("name", FluentValue::String("Aria".into()));
        let value = format_fluent_from_source_args(source, "en-US", "ui-greeting", Some(args))
            .expect("message should be formatted");
        assert_eq!(value, "Hello, Aria!");
    }

    #[test]
    fn fluent_format_supports_plural_rules() {
        let source =
            "ui-item-count = { $count ->\n    [one] One item\n   *[other] { $count } items\n}";
        let mut args = FluentArgs::new();
        args.set("count", FluentValue::Number(FluentNumber::from(3_i64)));
        let value = format_fluent_from_source_args(source, "en-US", "ui-item-count", Some(args))
            .expect("plural message should be formatted");
        assert_eq!(value, "3 items");
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
    fn pathfind_grid_rejects_invalid_dimensions() {
        let walkable = vec![1_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let err = pathfind_grid_bytes(0, 4, &walkable, &move_cost, 0, 0, 3, 3, 200)
            .expect_err("zero width must fail");
        assert_eq!(
            err,
            PathfindError::InvalidDimensions {
                width: 0,
                height: 4
            }
        );
    }

    #[test]
    fn pathfind_grid_returns_singleton_for_stationary_query() {
        let walkable = vec![0_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let path = pathfind_grid_bytes(4, 4, &walkable, &move_cost, 2, 2, 2, 2, 200)
            .expect("stationary query should succeed");
        assert_eq!(path, vec![GridPos::new(2, 2)]);
    }

    #[test]
    fn pathfind_grid_returns_empty_when_start_is_out_of_bounds() {
        let walkable = vec![1_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let path = pathfind_grid_bytes(4, 4, &walkable, &move_cost, -1, 0, 3, 3, 200)
            .expect("out-of-bounds start should be handled");
        assert!(path.is_empty());
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

    #[test]
    fn pathfind_grid_batch_xy_matches_tuple_results() {
        let walkable = vec![1_u8; 36];
        let move_cost = vec![1.0_f32; 36];
        let from = vec![(0, 0), (5, 0), (0, 5), (2, 3)];
        let to = vec![(5, 5), (0, 5), (5, 0), (4, 1)];

        let mut from_xy: Vec<i32> = Vec::with_capacity(from.len() * 2);
        let mut to_xy: Vec<i32> = Vec::with_capacity(to.len() * 2);
        for idx in 0..from.len() {
            from_xy.push(from[idx].0);
            from_xy.push(from[idx].1);
            to_xy.push(to[idx].0);
            to_xy.push(to[idx].1);
        }

        let grouped = pathfind_grid_batch_bytes(6, 6, &walkable, &move_cost, &from, &to, 400)
            .expect("tuple batch should succeed");
        let grouped_xy =
            pathfind_grid_batch_xy_bytes(6, 6, &walkable, &move_cost, &from_xy, &to_xy, 400)
                .expect("xy batch should succeed");

        assert_eq!(grouped_xy, grouped);
    }

    #[test]
    fn pathfind_grid_batch_vec2_matches_tuple_results() {
        let walkable = vec![1_u8; 36];
        let move_cost = vec![1.0_f32; 36];
        let from = vec![(0, 0), (5, 0), (0, 5), (2, 3)];
        let to = vec![(5, 5), (0, 5), (5, 0), (4, 1)];
        let from_vec2 = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(5.0, 0.0),
            Vector2::new(0.0, 5.0),
            Vector2::new(2.0, 3.0),
        ];
        let to_vec2 = vec![
            Vector2::new(5.0, 5.0),
            Vector2::new(0.0, 5.0),
            Vector2::new(5.0, 0.0),
            Vector2::new(4.0, 1.0),
        ];

        let grouped = pathfind_grid_batch_bytes(6, 6, &walkable, &move_cost, &from, &to, 400)
            .expect("tuple batch should succeed");
        let grouped_vec2 =
            pathfind_grid_batch_vec2_bytes(6, 6, &walkable, &move_cost, &from_vec2, &to_vec2, 400)
                .expect("vec2 batch should succeed");

        assert_eq!(grouped_vec2, grouped);
    }

    #[test]
    fn pathfind_grid_batch_returns_singletons_for_stationary_queries() {
        let walkable = vec![0_u8; 25];
        let move_cost = vec![1.0_f32; 25];
        let from = vec![(1, 1), (2, 3), (4, 0)];
        let to = vec![(1, 1), (2, 3), (4, 0)];

        let grouped = pathfind_grid_batch_bytes(5, 5, &walkable, &move_cost, &from, &to, 200)
            .expect("stationary tuple batch should succeed");
        assert_eq!(
            grouped,
            vec![
                vec![GridPos::new(1, 1)],
                vec![GridPos::new(2, 3)],
                vec![GridPos::new(4, 0)]
            ]
        );

        let from_xy = vec![1, 1, 2, 3, 4, 0];
        let to_xy = vec![1, 1, 2, 3, 4, 0];
        let grouped_xy =
            pathfind_grid_batch_xy_bytes(5, 5, &walkable, &move_cost, &from_xy, &to_xy, 200)
                .expect("stationary xy batch should succeed");
        assert_eq!(grouped_xy, grouped);

        let from_vec2 = vec![
            Vector2::new(1.0, 1.0),
            Vector2::new(2.0, 3.0),
            Vector2::new(4.0, 0.0),
        ];
        let to_vec2 = vec![
            Vector2::new(1.0, 1.0),
            Vector2::new(2.0, 3.0),
            Vector2::new(4.0, 0.0),
        ];
        let grouped_vec2 =
            pathfind_grid_batch_vec2_bytes(5, 5, &walkable, &move_cost, &from_vec2, &to_vec2, 200)
                .expect("stationary vec2 batch should succeed");
        assert_eq!(grouped_vec2, grouped);
    }

    #[test]
    fn pathfind_grid_batch_returns_empty_for_out_of_bounds_start() {
        let walkable = vec![1_u8; 25];
        let move_cost = vec![1.0_f32; 25];
        let from = vec![(-1, 0), (0, 0)];
        let to = vec![(4, 4), (4, 4)];

        let grouped = pathfind_grid_batch_bytes(5, 5, &walkable, &move_cost, &from, &to, 200)
            .expect("batch should succeed");
        assert_eq!(grouped.len(), 2);
        assert!(grouped[0].is_empty());
        assert!(!grouped[1].is_empty());

        let from_xy = vec![-1, 0, 0, 0];
        let to_xy = vec![4, 4, 4, 4];
        let grouped_xy =
            pathfind_grid_batch_xy_bytes(5, 5, &walkable, &move_cost, &from_xy, &to_xy, 200)
                .expect("xy batch should succeed");
        assert_eq!(grouped_xy, grouped);
    }

    #[test]
    fn pathfind_grid_batch_xy_rejects_odd_length_inputs() {
        let walkable = vec![1_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let from_xy = vec![0, 0, 1];
        let to_xy = vec![3, 3, 2];

        let err = pathfind_grid_batch_xy_bytes(4, 4, &walkable, &move_cost, &from_xy, &to_xy, 200)
            .expect_err("odd-length xy arrays must fail");
        assert_eq!(
            err,
            PathfindError::MismatchedBatchLength {
                from_len: 3,
                to_len: 3
            }
        );
    }

    #[test]
    fn pathfind_grid_batch_xy_rejects_invalid_dimensions() {
        let walkable = vec![1_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let from_xy = vec![0, 0];
        let to_xy = vec![1, 1];

        let err = pathfind_grid_batch_xy_bytes(4, 0, &walkable, &move_cost, &from_xy, &to_xy, 200)
            .expect_err("zero height must fail");
        assert_eq!(
            err,
            PathfindError::InvalidDimensions {
                width: 4,
                height: 0
            }
        );
    }

    #[test]
    fn backend_dispatch_counters_track_resolved_modes() {
        reset_dispatch_counts();

        let walkable = vec![1_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let (cpu_before, gpu_before) = read_dispatch_counts();
        let _ = dispatch_pathfind_grid_bytes(
            PATHFIND_BACKEND_CPU,
            4,
            4,
            &walkable,
            &move_cost,
            0,
            0,
            3,
            3,
            200,
        )
        .expect("cpu dispatch should succeed");
        let (cpu_after_cpu_call, gpu_after_cpu_call) = read_dispatch_counts();
        let _ = dispatch_pathfind_grid_bytes(
            PATHFIND_BACKEND_GPU,
            4,
            4,
            &walkable,
            &move_cost,
            0,
            0,
            3,
            3,
            200,
        )
        .expect("gpu dispatch should succeed");
        let (cpu_after_gpu_call, gpu_after_gpu_call) = read_dispatch_counts();

        assert!(cpu_after_cpu_call + gpu_after_cpu_call >= cpu_before + gpu_before + 1);
        assert!(
            cpu_after_gpu_call + gpu_after_gpu_call >= cpu_after_cpu_call + gpu_after_cpu_call + 1
        );
        if cfg!(feature = "gpu") {
            assert!(gpu_after_gpu_call >= gpu_after_cpu_call + 1);
        } else {
            assert!(cpu_after_gpu_call >= cpu_after_cpu_call + 1);
        }
    }

    #[test]
    fn parses_pathfinding_backend_modes() {
        assert_eq!(parse_pathfind_backend("auto"), Some(PATHFIND_BACKEND_AUTO));
        assert_eq!(parse_pathfind_backend("cpu"), Some(PATHFIND_BACKEND_CPU));
        assert_eq!(parse_pathfind_backend("gpu"), Some(PATHFIND_BACKEND_GPU));
        assert_eq!(parse_pathfind_backend("GPU"), Some(PATHFIND_BACKEND_GPU));
        assert_eq!(parse_pathfind_backend("unknown"), None);
    }

    #[test]
    fn resolves_pathfinding_backend_with_feature_gate() {
        assert_eq!(resolve_backend_mode(PATHFIND_BACKEND_CPU), "cpu");
        assert_eq!(
            resolve_backend_mode(PATHFIND_BACKEND_GPU),
            if cfg!(feature = "gpu") { "gpu" } else { "cpu" }
        );
        assert_eq!(
            resolve_backend_mode(PATHFIND_BACKEND_AUTO),
            if cfg!(feature = "gpu") { "gpu" } else { "cpu" }
        );
    }

    #[test]
    fn backend_dispatch_single_matches_cpu_path() {
        let walkable = vec![1_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let cpu = dispatch_pathfind_grid_bytes(
            PATHFIND_BACKEND_CPU,
            4,
            4,
            &walkable,
            &move_cost,
            0,
            0,
            3,
            3,
            200,
        )
        .expect("cpu dispatch should succeed");

        let auto = dispatch_pathfind_grid_bytes(
            PATHFIND_BACKEND_AUTO,
            4,
            4,
            &walkable,
            &move_cost,
            0,
            0,
            3,
            3,
            200,
        )
        .expect("auto dispatch should succeed");

        let gpu = dispatch_pathfind_grid_bytes(
            PATHFIND_BACKEND_GPU,
            4,
            4,
            &walkable,
            &move_cost,
            0,
            0,
            3,
            3,
            200,
        )
        .expect("gpu dispatch should succeed");

        assert_eq!(auto, cpu);
        assert_eq!(gpu, cpu);
    }

    #[test]
    fn backend_dispatch_batch_modes_match_cpu_path() {
        let walkable = vec![1_u8; 36];
        let move_cost = vec![1.0_f32; 36];
        let from_vec2 = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(5.0, 0.0),
            Vector2::new(0.0, 5.0),
        ];
        let to_vec2 = vec![
            Vector2::new(5.0, 5.0),
            Vector2::new(0.0, 5.0),
            Vector2::new(5.0, 0.0),
        ];
        let from_xy = vec![0, 0, 5, 0, 0, 5];
        let to_xy = vec![5, 5, 0, 5, 5, 0];

        let cpu_vec2 = dispatch_pathfind_grid_batch_vec2_bytes(
            PATHFIND_BACKEND_CPU,
            6,
            6,
            &walkable,
            &move_cost,
            &from_vec2,
            &to_vec2,
            300,
        )
        .expect("cpu vec2 dispatch should succeed");
        let auto_vec2 = dispatch_pathfind_grid_batch_vec2_bytes(
            PATHFIND_BACKEND_AUTO,
            6,
            6,
            &walkable,
            &move_cost,
            &from_vec2,
            &to_vec2,
            300,
        )
        .expect("auto vec2 dispatch should succeed");
        let gpu_vec2 = dispatch_pathfind_grid_batch_vec2_bytes(
            PATHFIND_BACKEND_GPU,
            6,
            6,
            &walkable,
            &move_cost,
            &from_vec2,
            &to_vec2,
            300,
        )
        .expect("gpu vec2 dispatch should succeed");

        let cpu_xy = dispatch_pathfind_grid_batch_xy_bytes(
            PATHFIND_BACKEND_CPU,
            6,
            6,
            &walkable,
            &move_cost,
            &from_xy,
            &to_xy,
            300,
        )
        .expect("cpu xy dispatch should succeed");
        let auto_xy = dispatch_pathfind_grid_batch_xy_bytes(
            PATHFIND_BACKEND_AUTO,
            6,
            6,
            &walkable,
            &move_cost,
            &from_xy,
            &to_xy,
            300,
        )
        .expect("auto xy dispatch should succeed");
        let gpu_xy = dispatch_pathfind_grid_batch_xy_bytes(
            PATHFIND_BACKEND_GPU,
            6,
            6,
            &walkable,
            &move_cost,
            &from_xy,
            &to_xy,
            300,
        )
        .expect("gpu xy dispatch should succeed");

        assert_eq!(auto_vec2, cpu_vec2);
        assert_eq!(gpu_vec2, cpu_vec2);
        assert_eq!(auto_xy, cpu_xy);
        assert_eq!(gpu_xy, cpu_xy);
    }

    #[test]
    fn public_backend_mode_helpers_roundtrip_and_validate() {
        let previous = get_pathfind_backend_mode().to_string();
        assert_eq!(has_gpu_pathfind_backend(), cfg!(feature = "gpu"));

        assert!(set_pathfind_backend_mode("cpu"));
        assert_eq!(get_pathfind_backend_mode(), "cpu");
        assert_eq!(resolve_pathfind_backend_mode(), "cpu");

        assert!(set_pathfind_backend_mode("auto"));
        assert_eq!(get_pathfind_backend_mode(), "auto");
        assert_eq!(
            resolve_pathfind_backend_mode(),
            if cfg!(feature = "gpu") { "gpu" } else { "cpu" }
        );

        assert!(!set_pathfind_backend_mode("invalid-mode"));
        assert!(set_pathfind_backend_mode(&previous));
    }

    #[test]
    fn public_dispatch_counter_helpers_track_dispatch_paths() {
        let previous = get_pathfind_backend_mode().to_string();
        assert!(set_pathfind_backend_mode("cpu"));
        reset_pathfind_backend_dispatch_counts();

        let walkable = vec![1_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let from = vec![(0, 0), (1, 1)];
        let to = vec![(3, 3), (2, 2)];
        let from_xy = vec![0, 0, 1, 1];
        let to_xy = vec![3, 3, 2, 2];

        let (cpu_before, gpu_before) = pathfind_backend_dispatch_counts();
        let _ = pathfind_grid_batch_dispatch_bytes(4, 4, &walkable, &move_cost, &from, &to, 200)
            .expect("dispatch tuple batch should succeed");
        let _ = pathfind_grid_batch_xy_dispatch_bytes(
            4, 4, &walkable, &move_cost, &from_xy, &to_xy, 200,
        )
        .expect("dispatch xy batch should succeed");
        let (cpu_after, gpu_after) = pathfind_backend_dispatch_counts();

        assert!(cpu_after >= cpu_before + 2);
        assert_eq!(gpu_after, gpu_before);
        assert!(set_pathfind_backend_mode(&previous));
    }

    #[test]
    fn ws2_roundtrip_preserves_snapshot_scalars() {
        let snapshot = EngineSnapshot {
            tick: 42,
            year: 3,
            day_of_year: 12,
            entity_count: 10,
            settlement_count: 2,
            system_count: 7,
            events_dispatched: 99,
        };
        let encoded = encode_ws2_blob(&snapshot).expect("ws2 encode should succeed");
        let decoded = decode_ws2_blob(&encoded).expect("ws2 decode should succeed");
        assert_eq!(decoded.tick, 42);
        assert_eq!(decoded.year, 3);
        assert_eq!(decoded.day_of_year, 12);
        assert_eq!(decoded.entity_count, 10);
        assert_eq!(decoded.settlement_count, 2);
        assert_eq!(decoded.system_count, 7);
        assert_eq!(decoded.events_dispatched, 99);
    }

    #[test]
    fn ws2_decode_rejects_invalid_magic() {
        let mut bytes = vec![0_u8; 16];
        bytes[0] = b'B';
        bytes[1] = b'A';
        bytes[2] = b'D';
        assert!(decode_ws2_blob(&bytes).is_none());
    }

    #[test]
    fn runtime_system_key_normalizes_script_paths() {
        assert_eq!(
            runtime_system_key_from_name("res://scripts/systems/record/stats_recorder.gd"),
            "stats_recorder"
        );
        assert_eq!(
            runtime_system_key_from_name("res:\\scripts\\systems\\record\\stats_recorder.gd"),
            "stats_recorder"
        );
        assert_eq!(runtime_system_key_from_name("stats_recorder"), "stats_recorder");
    }

    #[test]
    fn runtime_supports_expected_ported_systems() {
        assert!(runtime_supports_rust_system("stats_recorder"));
        assert!(runtime_supports_rust_system("resource_regen_system"));
        assert!(runtime_supports_rust_system("stat_sync_system"));
        assert!(runtime_supports_rust_system("stat_threshold_system"));
        assert!(runtime_supports_rust_system("upper_needs_system"));
        assert!(runtime_supports_rust_system("needs_system"));
        assert!(runtime_supports_rust_system("stress_system"));
        assert!(runtime_supports_rust_system("emotion_system"));
        assert!(runtime_supports_rust_system("child_stress_processor"));
        assert!(runtime_supports_rust_system("mental_break_system"));
        assert!(runtime_supports_rust_system("job_assignment_system"));
        assert!(!runtime_supports_rust_system("behavior_system"));
    }
}
