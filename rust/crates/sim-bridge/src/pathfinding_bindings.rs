use godot::prelude::{Array, PackedInt32Array, PackedVector2Array, Vector2};
use sim_systems::pathfinding::GridPos;

use crate::pathfinding_core;

pub(crate) fn encode_path_groups_xy(path_groups: Vec<Vec<GridPos>>) -> Array<PackedInt32Array> {
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

pub(crate) fn encode_path_groups_vec2(path_groups: Vec<Vec<GridPos>>) -> Array<PackedVector2Array> {
    let mut output: Array<PackedVector2Array> = Array::new();
    for group in path_groups {
        let packed = encode_path_vec2(group);
        output.push(&packed);
    }
    output
}

pub(crate) fn encode_path_xy(path: Vec<GridPos>) -> PackedInt32Array {
    let mut packed: PackedInt32Array = PackedInt32Array::new();
    packed.resize(path.len() * 2);
    for (idx, p) in path.into_iter().enumerate() {
        let base = idx * 2;
        packed[base] = p.x;
        packed[base + 1] = p.y;
    }
    packed
}

pub(crate) fn encode_path_vec2(path: Vec<GridPos>) -> PackedVector2Array {
    let mut packed: PackedVector2Array = PackedVector2Array::new();
    packed.resize(path.len());
    for (idx, p) in path.into_iter().enumerate() {
        packed[idx] = Vector2::new(p.x as f32, p.y as f32);
    }
    packed
}

pub(crate) fn parse_pathfind_backend(mode: &str) -> Option<u8> {
    pathfinding_core::parse_backend_mode(mode)
}

#[inline]
pub(crate) fn normalize_max_steps(max_steps: i32) -> usize {
    if max_steps <= 0 {
        200_usize
    } else {
        max_steps as usize
    }
}

pub(crate) fn resolve_backend_mode_code(mode: u8) -> u8 {
    pathfinding_core::resolve_backend_mode_code_core(mode)
}

pub(crate) fn backend_mode_to_str(mode: u8) -> &'static str {
    pathfinding_core::backend_mode_to_str_core(mode)
}

pub(crate) fn resolve_backend_mode(mode: u8) -> &'static str {
    pathfinding_core::resolve_backend_mode_str_core(mode)
}
