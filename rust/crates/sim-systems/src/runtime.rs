use hecs::World;
use sim_core::components::{
    Behavior, Body as BodyComponent, Emotion, Identity, Needs, Personality, Position, Skills,
    Social, Stress, Values,
};
use sim_core::config;
use sim_core::{ActionType, EmotionType, GrowthStage, NeedType, ValueType};
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
        EmotionRuntimeSystem, JobAssignmentRuntimeSystem, NeedsRuntimeSystem, ResourceRegenSystem,
        StatThresholdRuntimeSystem, StressRuntimeSystem, UpperNeedsRuntimeSystem,
    };
    use crate::body;
    use hecs::World;
    use sim_core::components::{
        Behavior, Body as BodyComponent, Emotion, Identity, Needs, Personality, Position,
        SkillEntry, Skills, Social, Stress, Values,
    };
    use sim_core::{GameCalendar, GrowthStage, NeedType, ResourceType, SettlementId, ValueType, WorldMap, config::GameConfig};
    use sim_core::world::TileResource;
    use sim_core::ids::EntityId;
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
