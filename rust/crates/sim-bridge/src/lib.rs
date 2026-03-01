//! sim-bridge: Boundary adapters between external callers and simulation crates.
//!
//! Phase R-3 will expose this through Godot GDExtension.
//! For now, this module provides pure-Rust conversion helpers that can be
//! reused by the future FFI layer.

use godot::prelude::*;
use sim_systems::{
    body,
    pathfinding::{find_path, GridCostMap, GridPos},
    stat_curve,
};
use std::sync::atomic::{AtomicU8, Ordering};

const PATHFIND_BACKEND_AUTO: u8 = 0;
const PATHFIND_BACKEND_CPU: u8 = 1;
const PATHFIND_BACKEND_GPU: u8 = 2;
static PATHFIND_BACKEND_MODE: AtomicU8 = AtomicU8::new(PATHFIND_BACKEND_AUTO);

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
    let grid = build_grid_cost_map(width, height, walkable, move_cost)?;
    Ok(find_path(
        &grid,
        GridPos::new(from_x, from_y),
        GridPos::new(to_x, to_y),
        max_steps,
    ))
}

fn build_grid_cost_map(
    width: i32,
    height: i32,
    walkable: &[u8],
    move_cost: &[f32],
) -> Result<GridCostMap, PathfindError> {
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

    let mut grid = GridCostMap::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            grid.set_walkable(x, y, walkable[idx] != 0);
            grid.set_move_cost(x, y, move_cost[idx]);
        }
    }
    Ok(grid)
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

    let grid = build_grid_cost_map(width, height, walkable, move_cost)?;
    let mut out = Vec::with_capacity(from_points.len());
    for idx in 0..from_points.len() {
        let (from_x, from_y) = from_points[idx];
        let (to_x, to_y) = to_points[idx];
        let path = find_path(
            &grid,
            GridPos::new(from_x, from_y),
            GridPos::new(to_x, to_y),
            max_steps,
        );
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

fn decode_xy_pairs(values: &PackedInt32Array) -> Option<Vec<(i32, i32)>> {
    if values.len() % 2 != 0 {
        return None;
    }
    let mut out: Vec<(i32, i32)> = Vec::with_capacity(values.len() / 2);
    let slice = values.as_slice();
    let mut i = 0usize;
    while i + 1 < slice.len() {
        out.push((slice[i], slice[i + 1]));
        i += 2;
    }
    Some(out)
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

fn parse_pathfind_backend(mode: &str) -> Option<u8> {
    match mode.to_ascii_lowercase().as_str() {
        "auto" => Some(PATHFIND_BACKEND_AUTO),
        "cpu" => Some(PATHFIND_BACKEND_CPU),
        "gpu" => Some(PATHFIND_BACKEND_GPU),
        _ => None,
    }
}

fn backend_mode_to_str(mode: u8) -> &'static str {
    match mode {
        PATHFIND_BACKEND_CPU => "cpu",
        PATHFIND_BACKEND_GPU => "gpu",
        _ => "auto",
    }
}

fn resolve_backend_mode(mode: u8) -> &'static str {
    match mode {
        PATHFIND_BACKEND_CPU => "cpu",
        PATHFIND_BACKEND_GPU => {
            if cfg!(feature = "gpu") {
                "gpu"
            } else {
                "cpu"
            }
        }
        _ => {
            if cfg!(feature = "gpu") {
                "gpu"
            } else {
                "cpu"
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
        let Some(parsed) = parse_pathfind_backend(&mode_string) else {
            return false;
        };
        PATHFIND_BACKEND_MODE.store(parsed, Ordering::Relaxed);
        true
    }

    #[func]
    fn get_pathfinding_backend(&self) -> GString {
        let mode = PATHFIND_BACKEND_MODE.load(Ordering::Relaxed);
        backend_mode_to_str(mode).into()
    }

    #[func]
    fn resolve_pathfinding_backend(&self) -> GString {
        let mode = PATHFIND_BACKEND_MODE.load(Ordering::Relaxed);
        resolve_backend_mode(mode).into()
    }

    #[func]
    fn has_gpu_pathfinding(&self) -> bool {
        cfg!(feature = "gpu")
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
        // GPU path is not implemented yet; use CPU pathfinding as fallback.
        self.pathfind_grid(
            width, height, walkable, move_cost, from_x, from_y, to_x, to_y, max_steps,
        )
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
        // GPU path is not implemented yet; use CPU pathfinding as fallback.
        self.pathfind_grid_xy(
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
        let steps = if max_steps <= 0 {
            200_usize
        } else {
            max_steps as usize
        };

        let from_pairs = match decode_xy_pairs(&from_xy) {
            Some(v) => v,
            None => return Array::new(),
        };
        let to_pairs = match decode_xy_pairs(&to_xy) {
            Some(v) => v,
            None => return Array::new(),
        };

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
        // GPU path is not implemented yet; use CPU batch pathfinding as fallback.
        self.pathfind_grid_batch_xy(
            width, height, walkable, move_cost, from_xy, to_xy, max_steps,
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
    use super::{
        parse_pathfind_backend, pathfind_from_flat, pathfind_grid_batch_bytes, pathfind_grid_bytes,
        resolve_backend_mode, PathfindError, PathfindInput, PATHFIND_BACKEND_AUTO,
        PATHFIND_BACKEND_CPU, PATHFIND_BACKEND_GPU,
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
}
