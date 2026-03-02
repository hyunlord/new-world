use hecs::World;
use std::collections::HashMap;
use sim_core::components::{
    Behavior, Body as BodyComponent, Emotion, Identity, Memory, Needs, Personality, Position,
    Skills, Social, Stress, Traits, Values,
};
use sim_core::config;
use sim_core::{
    ActionType, AttachmentType, EmotionType, GrowthStage, NeedType, RelationType, SettlementId,
    ValueType,
};
use sim_engine::{SimResources, SimSystem};
use crate::body;

/// First production Rust runtime system migrated from GDScript.
///
/// This system mirrors the scheduler slot of `stats_recorder.gd` and
/// executes inside the Rust engine tick loop.
#[derive(Debug, Clone)]
pub struct StatsRecorderSystem {
    priority: u32,
    tick_interval: u64,
}

impl StatsRecorderSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for StatsRecorderSystem {
    fn name(&self) -> &'static str {
        "stats_recorder"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        // Initial Rust-port baseline: keep this system side-effect free while
        // moving scheduler ownership into Rust. Follow-up phases will port the
        // full history/snapshot behavior from GDScript.
        let _population_count: u32 = world.len();
        let _settlement_count: usize = resources.settlements.len();
        let _queued_events: usize = resources.event_bus.pending_count();
    }
}

/// Rust runtime baseline system for stat-cache synchronization.
///
/// Full parity requires Rust-owned entity stat storage (Phase D). Until then this
/// keeps scheduler ownership migration and registry tracking in Rust.
#[derive(Debug, Clone)]
pub struct StatSyncSystem {
    priority: u32,
    tick_interval: u64,
}

impl StatSyncSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for StatSyncSystem {
    fn name(&self) -> &'static str {
        "stat_sync_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        // Baseline: this system intentionally has no gameplay side effects
        // until entity stat caches are migrated into Rust-owned data.
        let _population_count: u32 = world.len();
        let _map_tile_count: usize = resources.map.tile_count();
    }
}

/// Rust runtime system for tile resource regeneration.
///
/// This is the Rust execution counterpart of `resource_regen_system.gd`.
#[derive(Debug, Clone)]
pub struct ResourceRegenSystem {
    priority: u32,
    tick_interval: u64,
}

impl ResourceRegenSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for ResourceRegenSystem {
    fn name(&self) -> &'static str {
        "resource_regen_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, _world: &mut World, resources: &mut SimResources, _tick: u64) {
        let width: u32 = resources.map.width;
        let height: u32 = resources.map.height;
        for y in 0..height {
            for x in 0..width {
                let tile = resources.map.get_mut(x, y);
                for deposit in &mut tile.resources {
                    if deposit.regen_rate <= 0.0 || deposit.amount >= deposit.max_amount {
                        continue;
                    }
                    let next_amount = deposit.amount + deposit.regen_rate;
                    deposit.amount = next_amount.min(deposit.max_amount);
                }
            }
        }
    }
}

/// Rust runtime system for base-needs decay and energy adjustment.
///
/// This ports the hot-path math from `needs_system.gd` using the existing
/// Rust body kernels (`needs_base_decay_step`, `action_energy_cost`,
/// `rest_energy_recovery`).
#[derive(Debug, Clone)]
pub struct NeedsRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl NeedsRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for NeedsRuntimeSystem {
    fn name(&self) -> &'static str {
        "needs_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(
            &mut Needs,
            Option<&Behavior>,
            Option<&BodyComponent>,
            Option<&Position>,
        )>();
        for (_, (needs, behavior_opt, body_opt, position_opt)) in &mut query {
            let mut tile_temp: f32 = config::WARMTH_TEMP_NEUTRAL as f32;
            let mut has_tile_temp = false;
            if let Some(position) = position_opt {
                let x = position.x;
                let y = position.y;
                if resources.map.in_bounds(x, y) {
                    let tile = resources.map.get(x as u32, y as u32);
                    tile_temp = tile.temperature;
                    has_tile_temp = true;
                }
            }

            let decays = body::needs_base_decay_step(
                needs.get(NeedType::Hunger) as f32,
                config::HUNGER_DECAY_RATE as f32,
                1.0,
                config::HUNGER_METABOLIC_MIN as f32,
                config::HUNGER_METABOLIC_RANGE as f32,
                config::ENERGY_DECAY_RATE as f32,
                config::SOCIAL_DECAY_RATE as f32,
                config::SAFETY_DECAY_RATE as f32,
                config::THIRST_DECAY_RATE as f32,
                config::WARMTH_DECAY_RATE as f32,
                tile_temp,
                has_tile_temp,
                config::WARMTH_TEMP_NEUTRAL as f32,
                config::WARMTH_TEMP_FREEZING as f32,
                config::WARMTH_TEMP_COLD as f32,
                true,
            );

            let mut energy = (needs.energy as f32 - decays[1]).clamp(0.0, 1.0);
            if let Some(behavior) = behavior_opt {
                if behavior.current_action != ActionType::Idle
                    && behavior.current_action != ActionType::Rest
                {
                    let end_norm = body_opt
                        .map(|body_component| {
                            (body_component.end_realized as f32 / config::BODY_REALIZED_MAX as f32)
                                .clamp(0.0, 1.0)
                        })
                        .unwrap_or(0.5);
                    let action_cost = body::action_energy_cost(
                        config::ENERGY_ACTION_COST as f32,
                        end_norm,
                        config::BODY_END_COST_REDUCTION as f32,
                    );
                    energy = (energy - action_cost).clamp(0.0, 1.0);
                } else if behavior.current_action == ActionType::Rest {
                    let rec_norm = body_opt
                        .map(|body_component| {
                            (body_component.rec_realized as f32 / config::BODY_REALIZED_MAX as f32)
                                .clamp(0.0, 1.0)
                        })
                        .unwrap_or(0.5);
                    let recovery = body::rest_energy_recovery(
                        config::BODY_REST_ENERGY_RECOVERY as f32,
                        rec_norm,
                        config::BODY_REC_RECOVERY_BONUS as f32,
                    );
                    energy = (energy + recovery).clamp(0.0, 1.0);
                }
            }

            needs.set(
                NeedType::Hunger,
                (needs.get(NeedType::Hunger) as f32 - decays[0]) as f64,
            );
            needs.set(
                NeedType::Belonging,
                (needs.get(NeedType::Belonging) as f32 - decays[2]) as f64,
            );
            needs.set(
                NeedType::Thirst,
                (needs.get(NeedType::Thirst) as f32 - decays[3]) as f64,
            );
            needs.set(
                NeedType::Warmth,
                (needs.get(NeedType::Warmth) as f32 - decays[4]) as f64,
            );
            needs.set(
                NeedType::Safety,
                (needs.get(NeedType::Safety) as f32 - decays[5]) as f64,
            );
            needs.energy = energy as f64;
            needs.set(NeedType::Sleep, energy as f64);
        }
    }
}

/// Rust runtime baseline system for stress processing.
///
/// Full parity requires Rust-owned event/stressor sources and trace queues.
/// This phase keeps scheduler ownership and registry integration in Rust while
/// remaining side-effect free.
#[derive(Debug, Clone)]
pub struct StressRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl StressRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for StressRuntimeSystem {
    fn name(&self) -> &'static str {
        "stress_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(&Needs, &Stress, Option<&Emotion>)>();
        for (_, (needs, stress, emotion_opt)) in &mut query {
            let critical = body::needs_critical_severity_step(
                needs.get(NeedType::Thirst) as f32,
                needs.get(NeedType::Warmth) as f32,
                needs.get(NeedType::Safety) as f32,
                config::THIRST_CRITICAL as f32,
                config::WARMTH_CRITICAL as f32,
                config::SAFETY_CRITICAL as f32,
            );
            let _critical_total = critical[0] + critical[1] + critical[2];
            let _stress_snapshot = stress.level as f32 + stress.reserve as f32 + stress.allostatic_load as f32;
            if let Some(emotion) = emotion_opt {
                let _negative_valence = emotion.get(EmotionType::Fear) as f32
                    + emotion.get(EmotionType::Anger) as f32
                    + emotion.get(EmotionType::Sadness) as f32;
            }
        }
    }
}

/// Rust runtime baseline system for emotion processing.
///
/// Full parity needs Rust migration of fast/slow/memory-trace emotion state and
/// queued appraisal events. This phase provides Rust scheduler integration.
#[derive(Debug, Clone)]
pub struct EmotionRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl EmotionRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for EmotionRuntimeSystem {
    fn name(&self) -> &'static str {
        "emotion_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(&Emotion, Option<&Stress>, Option<&Personality>)>();
        for (_, (emotion, stress_opt, personality_opt)) in &mut query {
            let z_c = personality_opt
                .map(|personality| ((personality.axis(sim_core::HexacoAxis::C) as f32) - 0.5) * 2.0)
                .unwrap_or(0.0);
            let threshold = body::emotion_break_threshold(z_c, 300.0, 50.0);
            let stress_level = stress_opt.map(|stress| stress.level as f32 * 2000.0).unwrap_or(0.0);
            let _trigger_p = body::emotion_break_trigger_probability(
                stress_level,
                threshold,
                60.0,
                0.01,
            );
            let fear = emotion.get(EmotionType::Fear) as f32;
            let anger = emotion.get(EmotionType::Anger) as f32;
            let sadness = emotion.get(EmotionType::Sadness) as f32;
            let disgust = emotion.get(EmotionType::Disgust) as f32;
            let joy = emotion.get(EmotionType::Joy) as f32;
            let trust = emotion.get(EmotionType::Trust) as f32;
            let outrage = (anger + fear - trust - joy).max(0.0);
            let _break_type = body::emotion_break_type_code(
                outrage,
                fear,
                anger,
                sadness,
                disgust,
                0.7,
            );
        }
    }
}

/// Rust runtime baseline system for stat-threshold evaluation.
///
/// Full parity requires Rust-owned modifier/effect state and event emission.
/// This phase ports threshold predicate execution into the Rust scheduler.
#[derive(Debug, Clone)]
pub struct StatThresholdRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl StatThresholdRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn norm_to_stat_i32(value: f64) -> i32 {
    (value.clamp(0.0, 1.0) * 1000.0).round() as i32
}

impl SimSystem for StatThresholdRuntimeSystem {
    fn name(&self) -> &'static str {
        "stat_threshold_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<&Needs>();
        for (_, needs) in &mut query {
            let thirst_stat = norm_to_stat_i32(needs.get(NeedType::Thirst));
            let warmth_stat = norm_to_stat_i32(needs.get(NeedType::Warmth));
            let safety_stat = norm_to_stat_i32(needs.get(NeedType::Safety));
            let hunger_stat = norm_to_stat_i32(needs.get(NeedType::Hunger));

            let _thirst_low = body::stat_threshold_is_active(
                thirst_stat,
                norm_to_stat_i32(config::THIRST_LOW),
                0,
                25,
                false,
            );
            let _warmth_low = body::stat_threshold_is_active(
                warmth_stat,
                norm_to_stat_i32(config::WARMTH_LOW),
                0,
                25,
                false,
            );
            let _safety_low = body::stat_threshold_is_active(
                safety_stat,
                norm_to_stat_i32(config::SAFETY_LOW),
                0,
                25,
                false,
            );
            let _hunger_low = body::stat_threshold_is_active(
                hunger_stat,
                norm_to_stat_i32(config::HUNGER_EAT_THRESHOLD),
                0,
                25,
                false,
            );
        }
    }
}

/// Rust runtime baseline system for job-assignment balancing.
///
/// Full parity requires Rust-owned population/job mutation and event emission.
/// This phase ports ratio/deficit computation into Rust scheduler execution.
#[derive(Debug, Clone)]
pub struct JobAssignmentRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl JobAssignmentRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn job_code_from_name(job: &str) -> Option<usize> {
    match job {
        "gatherer" => Some(0),
        "lumberjack" => Some(1),
        "builder" => Some(2),
        "miner" => Some(3),
        _ => None,
    }
}

#[inline]
fn baseline_job_ratios(alive_count: i32) -> [f32; 4] {
    if alive_count < 10 {
        [0.8, 0.1, 0.1, 0.0]
    } else {
        [0.35, 0.25, 0.2, 0.2]
    }
}

impl SimSystem for JobAssignmentRuntimeSystem {
    fn name(&self) -> &'static str {
        "job_assignment_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut counts = [0_i32; 4];
        let mut alive_count = 0_i32;
        let mut query = world.query::<(&Behavior, Option<&Identity>)>();
        for (_, (behavior, identity_opt)) in &mut query {
            if let Some(identity) = identity_opt {
                if matches!(
                    identity.growth_stage,
                    GrowthStage::Infant | GrowthStage::Toddler
                ) {
                    continue;
                }
                if matches!(identity.growth_stage, GrowthStage::Child | GrowthStage::Teen) {
                    counts[0] += 1;
                    alive_count += 1;
                    continue;
                }
            }
            alive_count += 1;
            if let Some(code) = job_code_from_name(behavior.job.as_str()) {
                counts[code] += 1;
            }
        }
        if alive_count <= 0 {
            return;
        }
        let ratios = baseline_job_ratios(alive_count);
        let _best_job_code = body::job_assignment_best_job_code(&ratios, &counts, alive_count);
        let _rebalance_codes = body::job_assignment_rebalance_codes(&ratios, &counts, alive_count, 1.5);
    }
}

/// Rust runtime baseline system for child stress processing.
///
/// Full parity requires Rust-owned developmental-state metadata and stressor/event queues.
/// This phase ports child-stage appraisal math into Rust scheduler execution.
#[derive(Debug, Clone)]
pub struct ChildStressProcessorRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl ChildStressProcessorRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn child_stage_code_from_growth_stage(stage: GrowthStage) -> i32 {
    match stage {
        GrowthStage::Infant => 0,
        GrowthStage::Toddler => 1,
        GrowthStage::Child => 2,
        GrowthStage::Teen => 3,
        _ => 4,
    }
}

#[inline]
fn child_stage_baseline_params(stage_code: i32) -> (f32, bool, f32, f32, f32, f32) {
    match stage_code {
        0 => (1.0, true, 0.85, 1.0, 1.2, 1.0),
        1 => (0.9, false, 0.85, 1.0, 1.1, 1.0),
        2 => (0.8, false, 0.85, 1.0, 1.0, 1.0),
        3 => (0.7, false, 0.85, 1.0, 1.0, 1.0),
        _ => (0.0, false, 0.85, 1.0, 1.0, 1.0),
    }
}

impl SimSystem for ChildStressProcessorRuntimeSystem {
    fn name(&self) -> &'static str {
        "child_stress_processor"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(&Identity, &Stress, Option<&Needs>)>();
        for (_, (identity, stress, needs_opt)) in &mut query {
            if !identity.growth_stage.is_child_age() {
                continue;
            }
            let stage_code = child_stage_code_from_growth_stage(identity.growth_stage);
            let (buffer_power, shrp_active, shrp_override_threshold, spike_mult, vulnerability_mult, break_threshold_mult) =
                child_stage_baseline_params(stage_code);
            let intensity = (stress.level as f32).clamp(0.0, 1.0);
            let attachment_quality = needs_opt
                .map(|needs| needs.get(NeedType::Belonging) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let caregiver_present = attachment_quality > 0.5;
            let buffered_intensity = body::child_social_buffered_intensity(
                intensity,
                attachment_quality,
                caregiver_present,
                buffer_power,
            );
            let shrp = body::child_shrp_step(
                buffered_intensity,
                shrp_active,
                shrp_override_threshold,
                vulnerability_mult,
            );
            let stress_type_code = body::child_stress_type_code(
                shrp[0],
                caregiver_present,
                attachment_quality,
            );
            let _next = body::child_stress_apply_step(
                0.5,
                (stress.reserve as f32 * 100.0).clamp(0.0, 100.0),
                (stress.level as f32 * 2000.0).clamp(0.0, 2000.0),
                (stress.allostatic_load as f32 * 100.0).clamp(0.0, 100.0),
                shrp[0],
                spike_mult,
                vulnerability_mult,
                break_threshold_mult,
                stress_type_code,
            );
        }
    }
}

/// Rust runtime baseline system for mental-break trigger evaluation.
///
/// Full parity requires Rust-owned break definition tables, stochastic trigger,
/// active-break countdown state, and event emission.
#[derive(Debug, Clone)]
pub struct MentalBreakRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl MentalBreakRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for MentalBreakRuntimeSystem {
    fn name(&self) -> &'static str {
        "mental_break_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(&Stress, Option<&Personality>, Option<&Needs>)>();
        for (_, (stress, personality_opt, needs_opt)) in &mut query {
            let c_axis = personality_opt
                .map(|personality| personality.axis(sim_core::HexacoAxis::C) as f32)
                .unwrap_or(0.5);
            let e_axis = personality_opt
                .map(|personality| personality.axis(sim_core::HexacoAxis::E) as f32)
                .unwrap_or(0.5);
            let energy_norm = needs_opt
                .map(|needs| needs.energy as f32)
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);
            let hunger_norm = needs_opt
                .map(|needs| needs.get(NeedType::Hunger) as f32)
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);
            let reserve = (stress.reserve as f32 * 100.0).clamp(0.0, 100.0);
            let allostatic = (stress.allostatic_load as f32 * 100.0).clamp(0.0, 100.0);
            let threshold = body::mental_break_threshold(
                520.0,
                0.5,
                c_axis,
                e_axis,
                allostatic,
                energy_norm,
                hunger_norm,
                1.0,
                0.0,
                420.0,
                900.0,
                reserve,
                0.0,
            );
            let stress_scaled = (stress.level as f32 * 2000.0).clamp(0.0, 2000.0);
            let _trigger_p =
                body::mental_break_chance(stress_scaled, threshold, reserve, allostatic, 6000.0, 0.25);
        }
    }
}

/// Rust runtime baseline system for occupation evaluation.
///
/// Full parity requires Rust-owned occupation category mapping, occupation/job mutation,
/// and occupation-changed event emission.
#[derive(Debug, Clone)]
pub struct OccupationRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl OccupationRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn occupation_to_skill_id(occupation: &str) -> String {
    format!("SKILL_{}", occupation.to_uppercase())
}

impl SimSystem for OccupationRuntimeSystem {
    fn name(&self) -> &'static str {
        "occupation_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(&Behavior, Option<&Skills>, Option<&Identity>)>();
        for (_, (behavior, skills_opt, identity_opt)) in &mut query {
            if let Some(identity) = identity_opt {
                if matches!(
                    identity.growth_stage,
                    GrowthStage::Infant | GrowthStage::Toddler
                ) {
                    continue;
                }
            }

            let mut skill_levels: Vec<i32> = Vec::new();
            if let Some(skills) = skills_opt {
                skill_levels.reserve(skills.entries.len());
                for entry in skills.entries.values() {
                    skill_levels.push(entry.level as i32);
                }
            }
            let best_index = body::occupation_best_skill_index(skill_levels.as_slice());
            let best_skill_level = if best_index < 0 {
                0
            } else {
                skill_levels[best_index as usize]
            };
            if best_skill_level < config::OCCUPATION_MIN_SKILL_LEVEL as i32 {
                continue;
            }

            if behavior.occupation.is_empty()
                || behavior.occupation == "none"
                || behavior.occupation == "laborer"
            {
                continue;
            }
            let current_skill_id = occupation_to_skill_id(behavior.occupation.as_str());
            let current_level = skills_opt
                .map(|skills| skills.get_level(current_skill_id.as_str()) as i32)
                .unwrap_or(0);
            let _should_switch = body::occupation_should_switch(
                best_skill_level,
                current_level,
                config::OCCUPATION_CHANGE_HYSTERESIS as f32,
            );
        }
    }
}

/// Rust runtime baseline system for trauma-scar processing.
///
/// Full parity requires Rust-owned scar definition tables, scar stack mutation,
/// reactivation triggers, and emotion-baseline drift updates.
#[derive(Debug, Clone)]
pub struct TraumaScarRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl TraumaScarRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for TraumaScarRuntimeSystem {
    fn name(&self) -> &'static str {
        "trauma_scar_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(&sim_core::components::Memory, Option<&Stress>)>();
        for (_, (memory, stress_opt)) in &mut query {
            if memory.trauma_scars.is_empty() {
                continue;
            }
            let mut stress_sensitivity_mult = 1.0_f32;
            for scar in &memory.trauma_scars {
                let stacks = (scar.reactivation_count as i32 + 1).max(1);
                let base_chance = (scar.severity as f32 * 0.2).clamp(0.0, 1.0);
                let _acquire_chance =
                    body::trauma_scar_acquire_chance(base_chance, 1.0, stacks, 0.30);

                let base_mult = 1.0 + (scar.severity as f32 * 0.5);
                let factor = body::trauma_scar_sensitivity_factor(base_mult, stacks);
                stress_sensitivity_mult *= factor;
            }
            let _clamped_mult = stress_sensitivity_mult.clamp(0.5, 3.0);
            if let Some(stress) = stress_opt {
                let _stress_probe = stress.level as f32 * stress_sensitivity_mult;
            }
        }
    }
}

/// Rust runtime baseline system for title evaluation.
///
/// Full parity requires Rust-owned title grant/revoke state and settlement leadership linkage.
#[derive(Debug, Clone)]
pub struct TitleRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl TitleRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for TitleRuntimeSystem {
    fn name(&self) -> &'static str {
        "title_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, tick: u64) {
        let mut query = world.query::<(&Identity, Option<&Skills>)>();
        for (_, (identity, skills_opt)) in &mut query {
            let age_ticks = tick.saturating_sub(identity.birth_tick);
            let age_years = age_ticks as f32 / 8760.0;
            let _is_elder =
                body::title_is_elder(age_years, config::TITLE_ELDER_MIN_AGE_YEARS as f32);

            if let Some(skills) = skills_opt {
                for entry in skills.entries.values() {
                    let _tier = body::title_skill_tier(
                        entry.level as i32,
                        config::TITLE_EXPERT_SKILL_LEVEL as i32,
                        config::TITLE_MASTER_SKILL_LEVEL as i32,
                    );
                }
            }
        }
    }
}

/// Rust runtime baseline system for value-system progression.
///
/// Full parity requires Rust-owned peer interaction selection, settlement culture sync,
/// and value mutation/rationalization event flows.
#[derive(Debug, Clone)]
pub struct ValueRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl ValueRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for ValueRuntimeSystem {
    fn name(&self) -> &'static str {
        "value_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, tick: u64) {
        let mut query = world.query::<(&Values, Option<&Identity>, Option<&Personality>)>();
        for (_, (values, identity_opt, personality_opt)) in &mut query {
            let age_years = identity_opt
                .map(|identity| tick.saturating_sub(identity.birth_tick) as f32 / 8760.0)
                .unwrap_or(25.0);
            let plasticity = body::value_plasticity(age_years);

            let mut abs_sum = 0.0_f32;
            for value in values.values {
                abs_sum += (value as f32).abs();
            }
            let mean_abs_value = abs_sum / values.values.len() as f32;

            let openness = personality_opt
                .map(|personality| personality.axes[5] as f32)
                .unwrap_or(0.5);
            let extraversion = personality_opt
                .map(|personality| personality.axes[2] as f32)
                .unwrap_or(0.5);
            let _peer_receptivity =
                ((openness + extraversion) * 0.5 * plasticity * (1.0 - mean_abs_value))
                    .clamp(0.0, 1.0);
        }
    }
}

#[derive(Default, Debug, Clone)]
struct NetworkSettlementAccumulator {
    population: u32,
    unhappiness_sum: f32,
    frustration_sum: f32,
    independence_count: u32,
}

/// Rust runtime baseline system for social-network and revolution-risk evaluation.
///
/// Full parity requires Rust-owned settlement authority transitions, revolution cooldown
/// state mutation, leader replacement, and event emission.
#[derive(Debug, Clone)]
pub struct NetworkRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl NetworkRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for NetworkRuntimeSystem {
    fn name(&self) -> &'static str {
        "network_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut settlement_accumulators: HashMap<SettlementId, NetworkSettlementAccumulator> =
            HashMap::new();
        let mut query = world.query::<(
            &Social,
            Option<&Emotion>,
            Option<&Needs>,
            Option<&Values>,
            Option<&Identity>,
        )>();
        for (_, (social, emotion_opt, needs_opt, values_opt, identity_opt)) in &mut query {
            let mut strong_count = 0.0_f32;
            let mut weak_count = 0.0_f32;
            let mut bridge_count = 0.0_f32;
            for edge in &social.edges {
                let is_strong = matches!(
                    edge.relation_type,
                    RelationType::Friend
                        | RelationType::CloseFriend
                        | RelationType::Intimate
                        | RelationType::Spouse
                        | RelationType::Parent
                        | RelationType::Child
                        | RelationType::Sibling
                );
                if edge.is_bridge {
                    bridge_count += if is_strong { 1.0 } else { 0.5 };
                } else if is_strong {
                    strong_count += 1.0;
                } else {
                    weak_count += 1.0;
                }
            }
            let rep_score = ((social.reputation_local + social.reputation_regional) as f32 * 0.5)
                .clamp(0.0, 1.0);
            let _social_cap = body::network_social_capital_norm(
                strong_count,
                weak_count,
                bridge_count,
                rep_score,
                config::NETWORK_SOCIAL_CAP_STRONG_W as f32,
                config::NETWORK_SOCIAL_CAP_WEAK_W as f32,
                config::NETWORK_SOCIAL_CAP_BRIDGE_W as f32,
                config::NETWORK_SOCIAL_CAP_REP_W as f32,
                config::NETWORK_SOCIAL_CAP_NORM_DIV as f32,
            );

            let Some(identity) = identity_opt else {
                continue;
            };
            let Some(settlement_id) = identity.settlement_id else {
                continue;
            };
            let accumulator = settlement_accumulators.entry(settlement_id).or_default();
            accumulator.population += 1;

            let valence_norm = if let Some(emotion) = emotion_opt {
                let positive = emotion.get(EmotionType::Joy) as f32 + emotion.get(EmotionType::Trust) as f32;
                let negative = emotion.get(EmotionType::Fear) as f32
                    + emotion.get(EmotionType::Anger) as f32
                    + emotion.get(EmotionType::Sadness) as f32
                    + emotion.get(EmotionType::Disgust) as f32;
                ((positive - negative).clamp(-1.0, 1.0) + 1.0) * 0.5
            } else {
                0.5
            };
            accumulator.unhappiness_sum += 1.0 - valence_norm.clamp(0.0, 1.0);

            let frustration = if let Some(needs) = needs_opt {
                let hunger = (1.0 - needs.get(NeedType::Hunger) as f32).max(0.0);
                let energy = (1.0 - needs.energy as f32).max(0.0);
                let safety = (1.0 - needs.get(NeedType::Safety) as f32).max(0.0);
                (hunger + energy + safety) / 3.0
            } else {
                0.5
            };
            accumulator.frustration_sum += frustration.clamp(0.0, 1.0);

            if values_opt
                .map(|values| values.get(ValueType::Independence) as f32 > 0.3)
                .unwrap_or(false)
            {
                accumulator.independence_count += 1;
            }
        }

        for accumulator in settlement_accumulators.values() {
            if accumulator.population == 0 {
                continue;
            }
            let population = accumulator.population as f32;
            let unhappiness = accumulator.unhappiness_sum / population;
            let frustration = accumulator.frustration_sum / population;
            let independence_ratio = accumulator.independence_count as f32 / population;
            let _risk = body::revolution_risk_score(
                unhappiness,
                frustration,
                0.5,
                0.5,
                independence_ratio,
            );
        }
    }
}

/// Rust runtime baseline system for social-event evaluation.
///
/// Full parity requires Rust-owned pair selection, relationship mutation, and social-event emission.
#[derive(Debug, Clone)]
pub struct SocialEventRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl SocialEventRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn attachment_socialize_mult(attachment: AttachmentType) -> f32 {
    match attachment {
        AttachmentType::Secure => config::ATTACHMENT_SOCIALIZE_MULT[0] as f32,
        AttachmentType::Anxious => config::ATTACHMENT_SOCIALIZE_MULT[1] as f32,
        AttachmentType::Avoidant => config::ATTACHMENT_SOCIALIZE_MULT[2] as f32,
        AttachmentType::Fearful => config::ATTACHMENT_SOCIALIZE_MULT[3] as f32,
    }
}

impl SimSystem for SocialEventRuntimeSystem {
    fn name(&self) -> &'static str {
        "social_event_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(&Social, Option<&Personality>)>();
        for (_, (social, personality_opt)) in &mut query {
            let attachment = personality_opt
                .map(|personality| personality.attachment)
                .unwrap_or(AttachmentType::Secure);
            let attach_mult = attachment_socialize_mult(attachment);
            for edge in &social.edges {
                let trust_norm = (edge.trust as f32).clamp(0.0, 1.0);
                let familiarity_norm = (edge.familiarity as f32).clamp(0.0, 1.0);
                let compat_proxy = (0.5 + 0.5 * (trust_norm + familiarity_norm) * 0.5).clamp(0.0, 1.0);
                let _affinity_mult =
                    body::social_attachment_affinity_multiplier(attach_mult, attach_mult);
                let _proposal_prob = body::social_proposal_accept_prob(
                    edge.affinity as f32,
                    compat_proxy,
                );
            }
        }
    }
}

/// Rust runtime baseline system for building-effect evaluation.
///
/// Full parity requires Rust-owned building manager integration and proximity filtering.
#[derive(Debug, Clone)]
pub struct BuildingEffectRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl BuildingEffectRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for BuildingEffectRuntimeSystem {
    fn name(&self) -> &'static str {
        "building_effect_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, tick: u64) {
        let hour = (tick % 24) as i32;
        let is_night = hour >= 20 || hour < 6;
        let social_boost = body::building_campfire_social_boost(is_night, 0.01, 0.02);
        let mut query = world.query::<(&Needs, Option<&Social>)>();
        for (_, (needs, social_opt)) in &mut query {
            let social_capital_norm = social_opt
                .map(|social| social.social_capital as f32)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);
            let _next_social = body::building_add_capped(
                social_capital_norm,
                social_boost,
                1.0,
            );
            let _next_warmth = body::building_add_capped(
                needs.get(NeedType::Warmth) as f32,
                config::WARMTH_FIRE_RESTORE as f32,
                1.0,
            );
            let _next_energy = body::building_add_capped(
                needs.energy as f32,
                0.01,
                1.0,
            );
            let _next_safety = body::building_add_capped(
                needs.get(NeedType::Safety) as f32,
                config::SAFETY_SHELTER_RESTORE as f32,
                1.0,
            );
        }
    }
}

/// Rust runtime baseline system for family-system evaluation.
///
/// Full parity requires Rust-owned partner matching, pregnancy lifecycle state,
/// birth spawning, and parental/event side effects.
#[derive(Debug, Clone)]
pub struct FamilyRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl FamilyRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for FamilyRuntimeSystem {
    fn name(&self) -> &'static str {
        "family_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, tick: u64) {
        let mut query = world.query::<(&Identity, Option<&Needs>, Option<&Social>)>();
        for (_, (identity, needs_opt, social_opt)) in &mut query {
            if identity.sex != sim_core::Sex::Female {
                continue;
            }
            let age_years = tick.saturating_sub(identity.birth_tick) as f32 / 8760.0;
            if !(15.0..=45.0).contains(&age_years) {
                continue;
            }

            let has_partner = social_opt.and_then(|social| social.spouse).is_some();
            if !has_partner {
                continue;
            }

            let mother_nutrition = needs_opt
                .map(|needs| needs.get(NeedType::Hunger) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);

            let gestation_weeks = 40_i32;
            let _newborn_health =
                body::family_newborn_health(gestation_weeks, mother_nutrition, age_years, 1.0, 0.0);
        }
    }
}

/// Rust runtime baseline system for leader election scoring.
///
/// Full parity requires Rust-owned settlement candidate filtering, leader assignment state,
/// and leadership change event emission.
#[derive(Debug, Clone)]
pub struct LeaderRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl LeaderRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for LeaderRuntimeSystem {
    fn name(&self) -> &'static str {
        "leader_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, tick: u64) {
        let mut query = world.query::<(
            &Identity,
            Option<&Personality>,
            Option<&Social>,
            Option<&Values>,
        )>();
        for (_, (identity, personality_opt, social_opt, values_opt)) in &mut query {
            if identity.growth_stage < GrowthStage::Adult {
                continue;
            }
            let age_years = tick.saturating_sub(identity.birth_tick) as f32 / 8760.0;
            let age_respect = body::leader_age_respect(age_years);
            let charisma = personality_opt
                .map(|personality| personality.axis(sim_core::HexacoAxis::X) as f32)
                .unwrap_or(0.5);
            let wisdom = personality_opt
                .map(|personality| personality.axis(sim_core::HexacoAxis::O) as f32)
                .unwrap_or(0.5);
            let trustworthiness = personality_opt
                .map(|personality| personality.axis(sim_core::HexacoAxis::H) as f32)
                .unwrap_or(0.5);
            let intimidation = personality_opt
                .map(|personality| personality.axis(sim_core::HexacoAxis::A) as f32)
                .map(|a| (1.0 - a).clamp(0.0, 1.0))
                .unwrap_or(0.5);
            let social_capital = social_opt
                .map(|social| social.social_capital as f32)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);
            let rep_overall = social_opt
                .map(|social| ((social.reputation_local + social.reputation_regional) * 0.5) as f32)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);
            let _leader_score = body::leader_score(
                charisma,
                wisdom,
                trustworthiness,
                intimidation,
                social_capital,
                age_respect,
                config::LEADER_W_CHARISMA as f32,
                config::LEADER_W_WISDOM as f32,
                config::LEADER_W_TRUSTWORTHINESS as f32,
                config::LEADER_W_INTIMIDATION as f32,
                config::LEADER_W_SOCIAL_CAPITAL as f32,
                config::LEADER_W_AGE_RESPECT as f32,
                rep_overall,
            );

            let _authority_hint = values_opt
                .map(|values| values.get(ValueType::Law) as f32 - values.get(ValueType::Tradition) as f32)
                .unwrap_or(0.0);
        }
    }
}

/// Rust runtime baseline system for age-stage/body recalc evaluation.
///
/// Full parity requires Rust-owned stage transition events, yearly maturation,
/// and body realized-value mutation paths.
#[derive(Debug, Clone)]
pub struct AgeRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl AgeRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for AgeRuntimeSystem {
    fn name(&self) -> &'static str {
        "age_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, tick: u64) {
        let mut query = world.query::<(&Identity, Option<&BodyComponent>)>();
        for (_, (identity, body_opt)) in &mut query {
            let age_ticks = tick.saturating_sub(identity.birth_tick);
            let age_years = age_ticks as f32 / config::TICKS_PER_YEAR as f32;
            let _is_elder = body::title_is_elder(age_years, config::TITLE_ELDER_MIN_AGE_YEARS as f32);
            let _curves = body::compute_age_curves(age_years);

            if let Some(body_component) = body_opt {
                let _speed = body::age_body_speed(
                    body_component.agi_realized,
                    config::BODY_SPEED_SCALE as f32,
                    config::BODY_SPEED_BASE as f32,
                );
                let _strength = body::age_body_strength(body_component.str_realized);
            }
        }
    }
}

/// Rust runtime baseline system for mortality hazard evaluation.
///
/// Full parity requires Rust-owned death roll, cause routing, and bereavement side effects.
#[derive(Debug, Clone)]
pub struct MortalityRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl MortalityRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for MortalityRuntimeSystem {
    fn name(&self) -> &'static str {
        "mortality_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, tick: u64) {
        const A1: f32 = 0.60;
        const B1: f32 = 1.30;
        const A2: f32 = 0.010;
        const A3: f32 = 0.00006;
        const B3: f32 = 0.090;
        const TECH_K1: f32 = 0.30;
        const TECH_K2: f32 = 0.20;
        const TECH_K3: f32 = 0.05;
        const TECH_LEVEL: f32 = 0.0;
        const CARE_HUNGER_MIN: f32 = 0.3;
        const CARE_PROTECTION_FACTOR: f32 = 0.6;
        const SEASON_INFANT_MOD: f32 = 1.0;
        const SEASON_BACKGROUND_MOD: f32 = 1.0;

        let mut query = world.query::<(&Identity, Option<&Needs>, Option<&BodyComponent>)>();
        for (_, (identity, needs_opt, body_opt)) in &mut query {
            let age_ticks = tick.saturating_sub(identity.birth_tick);
            let age_years = age_ticks as f32 / config::TICKS_PER_YEAR as f32;
            let nutrition = needs_opt
                .map(|needs| needs.get(NeedType::Hunger) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let dr_norm = body_opt
                .map(|body_component| {
                    (body_component.dr_realized as f32 / config::BODY_REALIZED_DR_MAX as f32)
                        .clamp(0.0, 1.0)
                })
                .unwrap_or(0.5);
            let _hazards = body::mortality_hazards_and_prob(
                age_years,
                A1,
                B1,
                A2,
                A3,
                B3,
                TECH_K1,
                TECH_K2,
                TECH_K3,
                TECH_LEVEL,
                nutrition,
                CARE_HUNGER_MIN,
                CARE_PROTECTION_FACTOR,
                SEASON_INFANT_MOD,
                SEASON_BACKGROUND_MOD,
                1.0,
                dr_norm,
                config::BODY_DR_MORTALITY_REDUCTION as f32,
                age_years < 1.0,
            );
        }
    }
}

/// Rust runtime baseline system for population gate evaluation.
///
/// Full parity requires Rust-owned building/resource managers and birth/death side effects.
#[derive(Debug, Clone)]
pub struct PopulationRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl PopulationRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for PopulationRuntimeSystem {
    fn name(&self) -> &'static str {
        "population_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        const POP_MIN_FOR_BIRTH: i32 = 5;
        const POP_FREE_HOUSING_CAP: i32 = 25;
        const POP_SHELTER_CAPACITY: i32 = 6;
        const POP_FOOD_PER_ALIVE: f32 = 0.5;

        let alive_count = world.len().min(i32::MAX as u32) as i32;
        let mut total_shelters: i32 = 0;
        let mut total_food: f32 = 0.0;

        for settlement in resources.settlements.values() {
            total_food += settlement.stockpile_food.max(0.0) as f32;
            total_shelters += settlement.buildings.len().min(i32::MAX as usize) as i32;
        }

        let max_entities = (config::MAX_ENTITIES.min(i32::MAX as u32)) as i32;
        let _housing_cap = body::population_housing_cap(
            total_shelters,
            POP_FREE_HOUSING_CAP,
            POP_SHELTER_CAPACITY,
        );
        let _birth_block_code = body::population_birth_block_code(
            alive_count,
            max_entities,
            total_shelters,
            total_food,
            POP_MIN_FOR_BIRTH,
            POP_FREE_HOUSING_CAP,
            POP_SHELTER_CAPACITY,
            POP_FOOD_PER_ALIVE,
        );
    }
}

/// Rust runtime baseline system for migration pressure evaluation.
///
/// Full parity requires Rust-owned candidate selection, settlement creation,
/// and stockpile/member transfer side effects.
#[derive(Debug, Clone)]
pub struct MigrationRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl MigrationRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for MigrationRuntimeSystem {
    fn name(&self) -> &'static str {
        "migration_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, _world: &mut World, resources: &mut SimResources, _tick: u64) {
        const OVERCROWD_PER_SHELTER: i32 = 8;
        const FOOD_PER_CAPITA_THRESHOLD: f32 = 0.3;

        for settlement in resources.settlements.values() {
            let population = settlement.members.len().min(i32::MAX as usize) as i32;
            let shelter_count = settlement.buildings.len().min(i32::MAX as usize) as i32;
            let overcrowded = shelter_count > 0 && population > shelter_count * OVERCROWD_PER_SHELTER;
            let nearby_food = settlement.stockpile_food.max(0.0) as f32;
            let food_scarce =
                body::migration_food_scarce(nearby_food, population, FOOD_PER_CAPITA_THRESHOLD);
            let _should_attempt = body::migration_should_attempt(
                overcrowded,
                food_scarce,
                0.5,
                config::MIGRATION_CHANCE as f32,
            );
        }
    }
}

/// Rust runtime baseline system for trait-violation stress evaluation.
///
/// Full parity requires Rust-owned violation history state, stress injection,
/// and intrusive-thought / scar side-effect handling.
#[derive(Debug, Clone)]
pub struct TraitViolationRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl TraitViolationRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for TraitViolationRuntimeSystem {
    fn name(&self) -> &'static str {
        "trait_violation_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        const INTRUSIVE_BASE_CHANCE: f32 = 0.005;
        const VIOLATION_HISTORY_DECAY_TICKS: i32 = 365 * 12;
        const REPEATED_HABIT_MODIFIER: f32 = 0.0;
        const FORCED_MODIFIER: f32 = 0.5;
        const SURVIVAL_MODIFIER: f32 = 0.4;
        const NO_WITNESS_MODIFIER: f32 = 0.85;
        const FACET_THRESHOLD: f32 = 0.6;

        let mut query = world.query::<(Option<&Traits>, Option<&Stress>, Option<&Memory>)>();
        for (_, (traits_opt, stress_opt, memory_opt)) in &mut query {
            let stress_level = stress_opt.map(|stress| stress.level as f32).unwrap_or(0.5);
            let context = body::trait_violation_context_modifier(
                false,
                stress_level > 0.8,
                stress_level > 0.9,
                true,
                REPEATED_HABIT_MODIFIER,
                FORCED_MODIFIER,
                SURVIVAL_MODIFIER,
                NO_WITNESS_MODIFIER,
            );
            let _facet_scale = body::trait_violation_facet_scale(stress_level, FACET_THRESHOLD);
            let ptsd_mult = stress_opt
                .map(|stress| 1.0 + (stress.allostatic_load as f32).clamp(0.0, 1.0))
                .unwrap_or(1.0);
            let ticks_since = traits_opt
                .map(|traits| (traits.active.len().min(i32::MAX as usize) as i32) * 10)
                .unwrap_or(0);
            let has_trauma_scars = memory_opt
                .map(|memory| !memory.trauma_scars.is_empty())
                .unwrap_or(false);
            let _intrusive_chance = body::trait_violation_intrusive_chance(
                INTRUSIVE_BASE_CHANCE * context.max(0.0),
                ptsd_mult,
                ticks_since,
                VIOLATION_HISTORY_DECAY_TICKS,
                has_trauma_scars,
            );
        }
    }
}

/// Rust runtime system for upper-needs decay/fulfillment.
///
/// The step formula mirrors the `upper_needs_system.gd` Rust-bridge path.
#[derive(Debug, Clone)]
pub struct UpperNeedsRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl UpperNeedsRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn upper_needs_job_code(job: &str) -> i32 {
    match job {
        "builder" | "miner" => 1,
        "gatherer" | "lumberjack" => 2,
        _ => 0,
    }
}

#[inline]
fn upper_need_value(values: &Values, key: ValueType) -> f32 {
    values.get(key) as f32
}

impl SimSystem for UpperNeedsRuntimeSystem {
    fn name(&self) -> &'static str {
        "upper_needs_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(
            &mut Needs,
            Option<&Skills>,
            Option<&Values>,
            Option<&Behavior>,
            Option<&Identity>,
            Option<&Social>,
        )>();
        for (_, (needs, skills_opt, values_opt, behavior_opt, identity_opt, social_opt)) in &mut query {
            if let Some(identity) = identity_opt {
                if matches!(identity.growth_stage, GrowthStage::Infant | GrowthStage::Toddler) {
                    continue;
                }
            }

            let has_job = behavior_opt
                .map(|behavior| behavior.job.as_str() != "none")
                .unwrap_or(false);
            let has_settlement = identity_opt
                .and_then(|identity| identity.settlement_id)
                .is_some();
            let has_partner = social_opt.and_then(|social| social.spouse).is_some();

            let foraging = skills_opt
                .map(|skills| skills.get_level("SKILL_FORAGING") as i32)
                .unwrap_or(0);
            let woodcutting = skills_opt
                .map(|skills| skills.get_level("SKILL_WOODCUTTING") as i32)
                .unwrap_or(0);
            let mining = skills_opt
                .map(|skills| skills.get_level("SKILL_MINING") as i32)
                .unwrap_or(0);
            let construction = skills_opt
                .map(|skills| skills.get_level("SKILL_CONSTRUCTION") as i32)
                .unwrap_or(0);
            let hunting = skills_opt
                .map(|skills| skills.get_level("SKILL_HUNTING") as i32)
                .unwrap_or(0);
            let skill_levels = [foraging, woodcutting, mining, construction, hunting];
            let best_skill_norm = body::upper_needs_best_skill_normalized(&skill_levels, 100);

            let job_code = behavior_opt
                .map(|behavior| upper_needs_job_code(behavior.job.as_str()))
                .unwrap_or(0);
            let (craftsmanship, skill, hard_work, nature, independence, sacrifice) =
                if let Some(values) = values_opt {
                    (
                        upper_need_value(values, ValueType::Craftsmanship),
                        upper_need_value(values, ValueType::Skill),
                        upper_need_value(values, ValueType::HardWork),
                        upper_need_value(values, ValueType::Nature),
                        upper_need_value(values, ValueType::Independence),
                        upper_need_value(values, ValueType::Sacrifice),
                    )
                } else {
                    (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
                };
            let alignment = body::upper_needs_job_alignment(
                job_code,
                craftsmanship,
                skill,
                hard_work,
                nature,
                independence,
            );

            let current_values = [
                needs.get(NeedType::Competence) as f32,
                needs.get(NeedType::Autonomy) as f32,
                needs.get(NeedType::SelfActualization) as f32,
                needs.get(NeedType::Meaning) as f32,
                needs.get(NeedType::Transcendence) as f32,
                needs.get(NeedType::Recognition) as f32,
                needs.get(NeedType::Belonging) as f32,
                needs.get(NeedType::Intimacy) as f32,
            ];
            let decay_values = [
                config::UPPER_NEEDS_COMPETENCE_DECAY as f32,
                config::UPPER_NEEDS_AUTONOMY_DECAY as f32,
                config::UPPER_NEEDS_SELF_ACTUATION_DECAY as f32,
                config::UPPER_NEEDS_MEANING_DECAY as f32,
                config::UPPER_NEEDS_TRANSCENDENCE_DECAY as f32,
                config::UPPER_NEEDS_RECOGNITION_DECAY as f32,
                config::UPPER_NEEDS_BELONGING_DECAY as f32,
                config::UPPER_NEEDS_INTIMACY_DECAY as f32,
            ];
            let out = body::upper_needs_step(
                &current_values,
                &decay_values,
                config::UPPER_NEEDS_COMPETENCE_JOB_GAIN as f32,
                config::UPPER_NEEDS_AUTONOMY_JOB_GAIN as f32,
                config::UPPER_NEEDS_BELONGING_SETTLEMENT_GAIN as f32,
                config::UPPER_NEEDS_INTIMACY_PARTNER_GAIN as f32,
                config::UPPER_NEEDS_RECOGNITION_SKILL_COEFF as f32,
                config::UPPER_NEEDS_SELF_ACTUATION_SKILL_COEFF as f32,
                config::UPPER_NEEDS_MEANING_BASE_GAIN as f32,
                config::UPPER_NEEDS_MEANING_ALIGNED_GAIN as f32,
                config::UPPER_NEEDS_TRANSCENDENCE_SETTLEMENT_GAIN as f32,
                config::UPPER_NEEDS_TRANSCENDENCE_SACRIFICE_COEFF as f32,
                best_skill_norm,
                alignment,
                sacrifice,
                has_job,
                has_settlement,
                has_partner,
            );

            needs.set(NeedType::Competence, out[0] as f64);
            needs.set(NeedType::Autonomy, out[1] as f64);
            needs.set(NeedType::SelfActualization, out[2] as f64);
            needs.set(NeedType::Meaning, out[3] as f64);
            needs.set(NeedType::Transcendence, out[4] as f64);
            needs.set(NeedType::Recognition, out[5] as f64);
            needs.set(NeedType::Belonging, out[6] as f64);
            needs.set(NeedType::Intimacy, out[7] as f64);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AgeRuntimeSystem, BuildingEffectRuntimeSystem, ChildStressProcessorRuntimeSystem, EmotionRuntimeSystem,
        FamilyRuntimeSystem, JobAssignmentRuntimeSystem, MentalBreakRuntimeSystem, MigrationRuntimeSystem, MortalityRuntimeSystem, NeedsRuntimeSystem, NetworkRuntimeSystem,
        LeaderRuntimeSystem, OccupationRuntimeSystem, PopulationRuntimeSystem, SocialEventRuntimeSystem,
        ResourceRegenSystem, StatThresholdRuntimeSystem, StressRuntimeSystem,
        TitleRuntimeSystem, TraitViolationRuntimeSystem, TraumaScarRuntimeSystem,
        UpperNeedsRuntimeSystem, ValueRuntimeSystem,
    };
    use crate::body;
    use hecs::World;
    use sim_core::components::{
        Behavior, Body as BodyComponent, Emotion, Identity, Needs, Personality, Position,
        SkillEntry, Skills, Social, Stress, TraumaScar, Traits, Values, Memory, RelationshipEdge,
    };
    use sim_core::{GameCalendar, GrowthStage, NeedType, ResourceType, Settlement, SettlementId, ValueType, WorldMap, config::GameConfig};
    use sim_core::world::TileResource;
    use sim_core::ids::{BuildingId, EntityId};
    use sim_core::ActionType;
    use sim_engine::{SimResources, SimSystem};

    fn make_resources() -> SimResources {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let mut map = WorldMap::new(2, 1, 123);
        map.get_mut(0, 0).resources.push(TileResource {
            resource_type: ResourceType::Food,
            amount: 3.0,
            max_amount: 5.0,
            regen_rate: 0.75,
        });
        map.get_mut(1, 0).resources.push(TileResource {
            resource_type: ResourceType::Wood,
            amount: 4.8,
            max_amount: 5.0,
            regen_rate: 1.0,
        });
        SimResources::new(calendar, map, 999)
    }

    #[test]
    fn resource_regen_system_applies_regen_and_caps_at_max() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut system = ResourceRegenSystem::new(5, 10);
        system.run(&mut world, &mut resources, 10);

        let first = &resources.map.get(0, 0).resources[0];
        assert!((first.amount - 3.75).abs() < 1e-6);

        let second = &resources.map.get(1, 0).resources[0];
        assert!((second.amount - 5.0).abs() < 1e-6);
    }

    #[test]
    fn needs_runtime_system_applies_decay_and_action_energy_cost() {
        let mut world = World::new();
        let mut resources = make_resources();
        resources.map.get_mut(0, 0).temperature = 0.2;

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.6);
        needs.set(NeedType::Belonging, 0.7);
        needs.set(NeedType::Thirst, 0.8);
        needs.set(NeedType::Warmth, 0.9);
        needs.set(NeedType::Safety, 0.85);
        needs.energy = 0.75;
        needs.set(NeedType::Sleep, 0.75);

        let behavior = Behavior {
            current_action: ActionType::GatherWood,
            ..Behavior::default()
        };
        let body_component = BodyComponent {
            end_realized: 6000,
            rec_realized: 4000,
            ..BodyComponent::default()
        };
        let position = Position::new(0, 0);
        let entity = world.spawn((needs, behavior, body_component, position));

        let mut system = NeedsRuntimeSystem::new(10, 4);
        system.run(&mut world, &mut resources, 4);

        let decays = body::needs_base_decay_step(
            0.6,
            sim_core::config::HUNGER_DECAY_RATE as f32,
            1.0,
            sim_core::config::HUNGER_METABOLIC_MIN as f32,
            sim_core::config::HUNGER_METABOLIC_RANGE as f32,
            sim_core::config::ENERGY_DECAY_RATE as f32,
            sim_core::config::SOCIAL_DECAY_RATE as f32,
            sim_core::config::SAFETY_DECAY_RATE as f32,
            sim_core::config::THIRST_DECAY_RATE as f32,
            sim_core::config::WARMTH_DECAY_RATE as f32,
            0.2,
            true,
            sim_core::config::WARMTH_TEMP_NEUTRAL as f32,
            sim_core::config::WARMTH_TEMP_FREEZING as f32,
            sim_core::config::WARMTH_TEMP_COLD as f32,
            true,
        );
        let action_cost = body::action_energy_cost(
            sim_core::config::ENERGY_ACTION_COST as f32,
            (6000.0 / sim_core::config::BODY_REALIZED_MAX as f32).clamp(0.0, 1.0),
            sim_core::config::BODY_END_COST_REDUCTION as f32,
        );
        let expected_energy = (0.75 - decays[1] - action_cost).clamp(0.0, 1.0);

        let updated = world
            .get::<&Needs>(entity)
            .expect("updated needs component should be queryable");
        assert!((updated.get(NeedType::Hunger) as f32 - (0.6 - decays[0]).clamp(0.0, 1.0)).abs() < 1e-6);
        assert!((updated.get(NeedType::Belonging) as f32 - (0.7 - decays[2]).clamp(0.0, 1.0)).abs() < 1e-6);
        assert!((updated.get(NeedType::Thirst) as f32 - (0.8 - decays[3]).clamp(0.0, 1.0)).abs() < 1e-6);
        assert!((updated.get(NeedType::Warmth) as f32 - (0.9 - decays[4]).clamp(0.0, 1.0)).abs() < 1e-6);
        assert!((updated.get(NeedType::Safety) as f32 - (0.85 - decays[5]).clamp(0.0, 1.0)).abs() < 1e-6);
        assert!((updated.energy as f32 - expected_energy).abs() < 1e-6);
        assert!((updated.get(NeedType::Sleep) as f32 - expected_energy).abs() < 1e-6);
    }

    #[test]
    fn stress_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let needs = Needs::default();
        let stress = Stress::default();
        let emotion = Emotion::default();
        let entity = world.spawn((needs, stress, emotion));

        let mut system = StressRuntimeSystem::new(34, 4);
        system.run(&mut world, &mut resources, 4);

        let after = world
            .get::<&Stress>(entity)
            .expect("stress component should remain available");
        assert!((after.level - 0.0).abs() < 1e-9);
        assert!((after.reserve - 1.0).abs() < 1e-9);
        assert!((after.allostatic_load - 0.0).abs() < 1e-9);
    }

    #[test]
    fn emotion_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let emotion = Emotion::default();
        let stress = Stress::default();
        let personality = Personality::default();
        let entity = world.spawn((emotion, stress, personality));

        let mut system = EmotionRuntimeSystem::new(32, 12);
        system.run(&mut world, &mut resources, 12);

        let after = world
            .get::<&Emotion>(entity)
            .expect("emotion component should remain available");
        assert!((after.get(sim_core::EmotionType::Fear) - 0.0).abs() < 1e-9);
        assert!((after.get(sim_core::EmotionType::Joy) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn stat_threshold_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.45);
        needs.set(NeedType::Thirst, 0.30);
        needs.set(NeedType::Warmth, 0.22);
        needs.set(NeedType::Safety, 0.28);
        let entity = world.spawn((needs,));

        let mut system = StatThresholdRuntimeSystem::new(12, 5);
        system.run(&mut world, &mut resources, 5);

        let after = world
            .get::<&Needs>(entity)
            .expect("needs component should remain available");
        assert!((after.get(NeedType::Hunger) - 0.45).abs() < 1e-9);
        assert!((after.get(NeedType::Thirst) - 0.30).abs() < 1e-9);
        assert!((after.get(NeedType::Warmth) - 0.22).abs() < 1e-9);
        assert!((after.get(NeedType::Safety) - 0.28).abs() < 1e-9);
    }

    #[test]
    fn job_assignment_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let behavior = Behavior {
            job: "builder".to_string(),
            ..Behavior::default()
        };
        let identity = Identity {
            growth_stage: GrowthStage::Adult,
            ..Identity::default()
        };
        let entity = world.spawn((behavior, identity));

        let mut system = JobAssignmentRuntimeSystem::new(8, sim_core::config::JOB_ASSIGNMENT_TICK_INTERVAL);
        system.run(&mut world, &mut resources, sim_core::config::JOB_ASSIGNMENT_TICK_INTERVAL);

        let after = world
            .get::<&Behavior>(entity)
            .expect("behavior component should remain available");
        assert_eq!(after.job, "builder");
    }

    #[test]
    fn child_stress_processor_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let identity = Identity {
            growth_stage: GrowthStage::Child,
            ..Identity::default()
        };
        let needs = Needs::default();
        let stress = Stress {
            level: 0.35,
            reserve: 0.85,
            allostatic_load: 0.1,
            ..Stress::default()
        };
        let entity = world.spawn((identity, needs, stress));

        let mut system = ChildStressProcessorRuntimeSystem::new(32, 2);
        system.run(&mut world, &mut resources, 2);

        let after = world
            .get::<&Stress>(entity)
            .expect("stress component should remain available");
        assert!((after.level - 0.35).abs() < 1e-9);
        assert!((after.reserve - 0.85).abs() < 1e-9);
        assert!((after.allostatic_load - 0.1).abs() < 1e-9);
    }

    #[test]
    fn mental_break_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let stress = Stress {
            level: 0.62,
            reserve: 0.55,
            allostatic_load: 0.22,
            ..Stress::default()
        };
        let personality = Personality::default();
        let needs = Needs::default();
        let entity = world.spawn((stress, personality, needs));

        let mut system = MentalBreakRuntimeSystem::new(35, 1);
        system.run(&mut world, &mut resources, 1);

        let after = world
            .get::<&Stress>(entity)
            .expect("stress component should remain available");
        assert!((after.level - 0.62).abs() < 1e-9);
        assert!((after.reserve - 0.55).abs() < 1e-9);
        assert!((after.allostatic_load - 0.22).abs() < 1e-9);
    }

    #[test]
    fn occupation_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let behavior = Behavior {
            occupation: "foraging".to_string(),
            job: "gatherer".to_string(),
            ..Behavior::default()
        };
        let identity = Identity {
            growth_stage: GrowthStage::Adult,
            ..Identity::default()
        };
        let mut skills = Skills::default();
        skills
            .entries
            .insert("SKILL_FORAGING".to_string(), SkillEntry { level: 65, xp: 0.0 });
        skills
            .entries
            .insert("SKILL_MINING".to_string(), SkillEntry { level: 40, xp: 0.0 });
        let entity = world.spawn((behavior, identity, skills));

        let mut system = OccupationRuntimeSystem::new(36, sim_core::config::OCCUPATION_EVAL_INTERVAL);
        system.run(&mut world, &mut resources, sim_core::config::OCCUPATION_EVAL_INTERVAL);

        let after = world
            .get::<&Behavior>(entity)
            .expect("behavior component should remain available");
        assert_eq!(after.occupation, "foraging");
        assert_eq!(after.job, "gatherer");
    }

    #[test]
    fn trauma_scar_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let memory = Memory {
            trauma_scars: vec![TraumaScar {
                scar_id: "betrayal".to_string(),
                acquired_tick: 120,
                severity: 0.6,
                reactivation_count: 2,
            }],
            ..Memory::default()
        };
        let stress = Stress {
            level: 0.4,
            reserve: 0.8,
            allostatic_load: 0.15,
            ..Stress::default()
        };
        let entity = world.spawn((memory, stress));

        let mut system = TraumaScarRuntimeSystem::new(36, 10);
        system.run(&mut world, &mut resources, 10);

        let after = world
            .get::<&Memory>(entity)
            .expect("memory component should remain available");
        assert_eq!(after.trauma_scars.len(), 1);
        assert_eq!(after.trauma_scars[0].scar_id, "betrayal");
        assert_eq!(after.trauma_scars[0].reactivation_count, 2);
    }

    #[test]
    fn title_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let identity = Identity {
            birth_tick: 0,
            growth_stage: GrowthStage::Adult,
            ..Identity::default()
        };
        let mut skills = Skills::default();
        skills
            .entries
            .insert("SKILL_FORAGING".to_string(), SkillEntry { level: 80, xp: 0.0 });
        let entity = world.spawn((identity, skills));

        let mut system = TitleRuntimeSystem::new(37, sim_core::config::TITLE_EVAL_INTERVAL);
        system.run(&mut world, &mut resources, 8760 * 60);

        let after_identity = world
            .get::<&Identity>(entity)
            .expect("identity component should remain available");
        let after_skills = world
            .get::<&Skills>(entity)
            .expect("skills component should remain available");
        assert_eq!(after_identity.birth_tick, 0);
        assert_eq!(
            after_skills.get_level("SKILL_FORAGING"),
            80,
        );
    }

    #[test]
    fn value_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut values = Values::default();
        values.set(ValueType::Tradition, 0.4);
        values.set(ValueType::Nature, -0.2);
        let identity = Identity {
            birth_tick: 0,
            growth_stage: GrowthStage::Adult,
            ..Identity::default()
        };
        let personality = Personality::default();
        let entity = world.spawn((values, identity, personality));

        let mut system = ValueRuntimeSystem::new(55, 200);
        system.run(&mut world, &mut resources, 8760 * 20);

        let after = world
            .get::<&Values>(entity)
            .expect("values component should remain available");
        assert!((after.get(ValueType::Tradition) - 0.4).abs() < 1e-9);
        assert!((after.get(ValueType::Nature) + 0.2).abs() < 1e-9);
    }

    #[test]
    fn network_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut social = Social::default();
        let mut edge = RelationshipEdge::new(EntityId(77));
        edge.relation_type = sim_core::RelationType::Friend;
        social.edges.push(edge);
        social.reputation_local = 0.6;
        social.reputation_regional = 0.4;

        let mut values = Values::default();
        values.set(ValueType::Independence, 0.5);
        let identity = Identity {
            settlement_id: Some(SettlementId(1)),
            ..Identity::default()
        };
        let needs = Needs::default();
        let emotion = Emotion::default();
        let entity = world.spawn((social, values, identity, needs, emotion));

        let mut system = NetworkRuntimeSystem::new(58, sim_core::config::REVOLUTION_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::REVOLUTION_TICK_INTERVAL,
        );

        let after_social = world
            .get::<&Social>(entity)
            .expect("social component should remain available");
        let after_values = world
            .get::<&Values>(entity)
            .expect("values component should remain available");
        assert_eq!(after_social.edges.len(), 1);
        assert!((after_values.get(ValueType::Independence) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn social_event_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut social = Social::default();
        let mut edge = RelationshipEdge::new(EntityId(9));
        edge.affinity = 65.0;
        edge.trust = 0.55;
        edge.familiarity = 0.45;
        social.edges.push(edge);
        let personality = Personality {
            attachment: sim_core::AttachmentType::Anxious,
            ..Personality::default()
        };
        let entity = world.spawn((social, personality));

        let mut system = SocialEventRuntimeSystem::new(37, 30);
        system.run(&mut world, &mut resources, 30);

        let after = world
            .get::<&Social>(entity)
            .expect("social component should remain available");
        assert_eq!(after.edges.len(), 1);
        assert!((after.edges[0].affinity - 65.0).abs() < 1e-9);
        assert!((after.edges[0].trust - 0.55).abs() < 1e-9);
    }

    #[test]
    fn building_effect_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut needs = Needs::default();
        needs.set(NeedType::Warmth, 0.6);
        needs.set(NeedType::Safety, 0.4);
        needs.energy = 0.55;
        let social = Social::default();
        let entity = world.spawn((needs, social));

        let mut system =
            BuildingEffectRuntimeSystem::new(15, sim_core::config::BUILDING_EFFECT_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::BUILDING_EFFECT_TICK_INTERVAL,
        );

        let after = world
            .get::<&Needs>(entity)
            .expect("needs component should remain available");
        assert!((after.get(NeedType::Warmth) - 0.6).abs() < 1e-9);
        assert!((after.get(NeedType::Safety) - 0.4).abs() < 1e-9);
        assert!((after.energy - 0.55).abs() < 1e-9);
    }

    #[test]
    fn family_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let identity = Identity {
            sex: sim_core::Sex::Female,
            birth_tick: 0,
            growth_stage: GrowthStage::Adult,
            settlement_id: Some(SettlementId(1)),
            ..Identity::default()
        };
        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.7);
        let social = Social {
            spouse: Some(EntityId(2)),
            ..Social::default()
        };
        let entity = world.spawn((identity, needs, social));

        let mut system = FamilyRuntimeSystem::new(52, 365);
        system.run(&mut world, &mut resources, 8760 * 25);

        let after_identity = world
            .get::<&Identity>(entity)
            .expect("identity component should remain available");
        let after_needs = world
            .get::<&Needs>(entity)
            .expect("needs component should remain available");
        assert_eq!(after_identity.sex, sim_core::Sex::Female);
        assert!((after_needs.get(NeedType::Hunger) - 0.7).abs() < 1e-9);
    }

    #[test]
    fn leader_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let identity = Identity {
            birth_tick: 0,
            growth_stage: GrowthStage::Adult,
            ..Identity::default()
        };
        let personality = Personality::default();
        let mut social = Social::default();
        social.social_capital = 0.4;
        social.reputation_local = 0.6;
        social.reputation_regional = 0.5;
        let mut values = Values::default();
        values.set(ValueType::Law, 0.3);
        values.set(ValueType::Tradition, 0.2);
        let entity = world.spawn((identity, personality, social, values));

        let mut system = LeaderRuntimeSystem::new(52, sim_core::config::LEADER_CHECK_INTERVAL);
        system.run(&mut world, &mut resources, 8760 * 30);

        let after_identity = world
            .get::<&Identity>(entity)
            .expect("identity component should remain available");
        let after_social = world
            .get::<&Social>(entity)
            .expect("social component should remain available");
        assert_eq!(after_identity.growth_stage, GrowthStage::Adult);
        assert!((after_social.social_capital - 0.4).abs() < 1e-9);
    }

    #[test]
    fn age_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let identity = Identity {
            birth_tick: 0,
            growth_stage: GrowthStage::Adult,
            ..Identity::default()
        };
        let body = BodyComponent::default();
        let entity = world.spawn((identity, body));

        let mut system = AgeRuntimeSystem::new(48, 50);
        system.run(
            &mut world,
            &mut resources,
            (sim_core::config::TICKS_PER_YEAR as u64) * 20,
        );

        let after_identity = world
            .get::<&Identity>(entity)
            .expect("identity component should remain available");
        let after_body = world
            .get::<&BodyComponent>(entity)
            .expect("body component should remain available");
        assert_eq!(after_identity.birth_tick, 0);
        assert_eq!(after_identity.growth_stage, GrowthStage::Adult);
        assert_eq!(after_body.agi_realized, 700);
        assert_eq!(after_body.str_realized, 1000);
    }

    #[test]
    fn mortality_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let identity = Identity {
            birth_tick: 0,
            growth_stage: GrowthStage::Adult,
            ..Identity::default()
        };
        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.75);
        let mut body = BodyComponent::default();
        body.dr_realized = 850;
        let entity = world.spawn((identity, needs, body));

        let mut system = MortalityRuntimeSystem::new(49, 1);
        system.run(
            &mut world,
            &mut resources,
            (sim_core::config::TICKS_PER_YEAR as u64) * 30,
        );

        let after_identity = world
            .get::<&Identity>(entity)
            .expect("identity component should remain available");
        let after_needs = world
            .get::<&Needs>(entity)
            .expect("needs component should remain available");
        let after_body = world
            .get::<&BodyComponent>(entity)
            .expect("body component should remain available");
        assert_eq!(after_identity.growth_stage, GrowthStage::Adult);
        assert!((after_needs.get(NeedType::Hunger) - 0.75).abs() < 1e-9);
        assert_eq!(after_body.dr_realized, 850);
    }

    #[test]
    fn population_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let identity = Identity::default();
        world.spawn((identity,));

        let mut settlement = Settlement::new(
            SettlementId(1),
            "settlement-1".to_string(),
            0,
            0,
            0,
        );
        settlement.stockpile_food = 22.0;
        settlement.buildings = vec![BuildingId(10), BuildingId(11)];
        resources.settlements.insert(settlement.id, settlement);

        let mut system = PopulationRuntimeSystem::new(50, sim_core::config::POPULATION_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::POPULATION_TICK_INTERVAL,
        );

        assert_eq!(world.len(), 1);
        let after = resources
            .settlements
            .get(&SettlementId(1))
            .expect("settlement should remain available");
        assert_eq!(after.buildings.len(), 2);
        assert!((after.stockpile_food - 22.0).abs() < 1e-9);
    }

    #[test]
    fn migration_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut settlement = Settlement::new(
            SettlementId(2),
            "settlement-2".to_string(),
            8,
            5,
            0,
        );
        settlement.members = vec![EntityId(1), EntityId(2), EntityId(3), EntityId(4)];
        settlement.buildings = vec![BuildingId(20)];
        settlement.stockpile_food = 10.0;
        resources.settlements.insert(settlement.id, settlement);

        let mut system = MigrationRuntimeSystem::new(60, sim_core::config::MIGRATION_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::MIGRATION_TICK_INTERVAL,
        );

        assert_eq!(world.len(), 0);
        let after = resources
            .settlements
            .get(&SettlementId(2))
            .expect("settlement should remain available");
        assert_eq!(after.members.len(), 4);
        assert_eq!(after.buildings.len(), 1);
        assert!((after.stockpile_food - 10.0).abs() < 1e-9);
    }

    #[test]
    fn trait_violation_runtime_system_baseline_runs_without_side_effects() {
        let mut world = World::new();
        let mut resources = make_resources();
        let traits = Traits {
            active: vec!["lawful".to_string()],
            salience_scores: vec![("lawful".to_string(), 0.9)],
        };
        let stress = Stress {
            level: 0.85,
            reserve: 0.35,
            allostatic_load: 0.75,
            ..Stress::default()
        };
        let memory = Memory {
            trauma_scars: vec![TraumaScar {
                scar_id: "betrayal".to_string(),
                acquired_tick: 120,
                severity: 0.5,
                reactivation_count: 1,
            }],
            ..Memory::default()
        };
        let entity = world.spawn((traits, stress, memory));

        let mut system = TraitViolationRuntimeSystem::new(37, 1);
        system.run(&mut world, &mut resources, 1);

        let after_traits = world
            .get::<&Traits>(entity)
            .expect("traits component should remain available");
        let after_stress = world
            .get::<&Stress>(entity)
            .expect("stress component should remain available");
        let after_memory = world
            .get::<&Memory>(entity)
            .expect("memory component should remain available");
        assert_eq!(after_traits.active.len(), 1);
        assert!((after_stress.level - 0.85).abs() < 1e-9);
        assert_eq!(after_memory.trauma_scars.len(), 1);
    }

    #[test]
    fn upper_needs_runtime_system_updates_upper_need_buckets() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut needs = Needs::default();
        needs.set(NeedType::Competence, 0.45);
        needs.set(NeedType::Autonomy, 0.52);
        needs.set(NeedType::SelfActualization, 0.33);
        needs.set(NeedType::Meaning, 0.41);
        needs.set(NeedType::Transcendence, 0.29);
        needs.set(NeedType::Recognition, 0.35);
        needs.set(NeedType::Belonging, 0.38);
        needs.set(NeedType::Intimacy, 0.32);

        let mut skills = Skills::default();
        skills
            .entries
            .insert("SKILL_FORAGING".to_string(), SkillEntry { level: 78, xp: 0.0 });
        skills
            .entries
            .insert("SKILL_MINING".to_string(), SkillEntry { level: 40, xp: 0.0 });

        let mut values = Values::default();
        values.set(ValueType::Nature, 0.7);
        values.set(ValueType::Independence, 0.4);
        values.set(ValueType::HardWork, 0.5);
        values.set(ValueType::Sacrifice, 0.2);

        let behavior = Behavior {
            job: "gatherer".to_string(),
            ..Behavior::default()
        };
        let identity = Identity {
            settlement_id: Some(SettlementId(1)),
            growth_stage: GrowthStage::Adult,
            ..Identity::default()
        };
        let social = Social {
            spouse: Some(EntityId(2)),
            ..Social::default()
        };

        let entity = world.spawn((needs, skills, values, behavior, identity, social));
        let mut system = UpperNeedsRuntimeSystem::new(12, 5);
        system.run(&mut world, &mut resources, 5);

        let best_skill_norm = body::upper_needs_best_skill_normalized(&[78, 0, 40, 0, 0], 100);
        let alignment = body::upper_needs_job_alignment(2, 0.0, 0.0, 0.5, 0.7, 0.4);
        let expected = body::upper_needs_step(
            &[0.45, 0.52, 0.33, 0.41, 0.29, 0.35, 0.38, 0.32],
            &[
                sim_core::config::UPPER_NEEDS_COMPETENCE_DECAY as f32,
                sim_core::config::UPPER_NEEDS_AUTONOMY_DECAY as f32,
                sim_core::config::UPPER_NEEDS_SELF_ACTUATION_DECAY as f32,
                sim_core::config::UPPER_NEEDS_MEANING_DECAY as f32,
                sim_core::config::UPPER_NEEDS_TRANSCENDENCE_DECAY as f32,
                sim_core::config::UPPER_NEEDS_RECOGNITION_DECAY as f32,
                sim_core::config::UPPER_NEEDS_BELONGING_DECAY as f32,
                sim_core::config::UPPER_NEEDS_INTIMACY_DECAY as f32,
            ],
            sim_core::config::UPPER_NEEDS_COMPETENCE_JOB_GAIN as f32,
            sim_core::config::UPPER_NEEDS_AUTONOMY_JOB_GAIN as f32,
            sim_core::config::UPPER_NEEDS_BELONGING_SETTLEMENT_GAIN as f32,
            sim_core::config::UPPER_NEEDS_INTIMACY_PARTNER_GAIN as f32,
            sim_core::config::UPPER_NEEDS_RECOGNITION_SKILL_COEFF as f32,
            sim_core::config::UPPER_NEEDS_SELF_ACTUATION_SKILL_COEFF as f32,
            sim_core::config::UPPER_NEEDS_MEANING_BASE_GAIN as f32,
            sim_core::config::UPPER_NEEDS_MEANING_ALIGNED_GAIN as f32,
            sim_core::config::UPPER_NEEDS_TRANSCENDENCE_SETTLEMENT_GAIN as f32,
            sim_core::config::UPPER_NEEDS_TRANSCENDENCE_SACRIFICE_COEFF as f32,
            best_skill_norm,
            alignment,
            0.2,
            true,
            true,
            true,
        );

        let updated = world
            .get::<&Needs>(entity)
            .expect("updated needs component should be queryable");
        assert!((updated.get(NeedType::Competence) as f32 - expected[0]).abs() < 1e-6);
        assert!((updated.get(NeedType::Recognition) as f32 - expected[5]).abs() < 1e-6);
        assert!((updated.get(NeedType::Belonging) as f32 - expected[6]).abs() < 1e-6);
        assert!((updated.get(NeedType::Intimacy) as f32 - expected[7]).abs() < 1e-6);
    }
}
