use hecs::World;
use sim_core::components::{
    Behavior, Body as BodyComponent, Emotion, Identity, Needs, Personality, Position, Skills,
    Social, Stress, Values,
};
use sim_core::config;
use sim_core::{ActionType, EmotionType, GrowthStage, HexacoAxis, NeedType, ValueType};
use sim_engine::{SimResources, SimSystem};

use crate::body;

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

/// Rust runtime system for stress-state updates.
///
/// This system performs actual component writes (`Stress.level/reserve/allostatic_load/state`)
/// and is the first step of the strict Phase-5 active-write migration for stress.
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
        let mut query = world.query::<(&Needs, &mut Stress, Option<&Emotion>)>();
        for (_, (needs, stress, emotion_opt)) in &mut query {
            let hunger: f32 = needs.get(NeedType::Hunger) as f32;
            let thirst: f32 = needs.get(NeedType::Thirst) as f32;
            let warmth: f32 = needs.get(NeedType::Warmth) as f32;
            let safety: f32 = needs.get(NeedType::Safety) as f32;
            let social: f32 = needs.get(NeedType::Belonging) as f32;
            let energy: f32 = needs.energy as f32;

            let critical = body::needs_critical_severity_step(
                thirst,
                warmth,
                safety,
                config::THIRST_CRITICAL as f32,
                config::WARMTH_CRITICAL as f32,
                config::SAFETY_CRITICAL as f32,
            );
            let hunger_critical: f32 = ((0.15 - hunger) / 0.15).clamp(0.0, 1.0);

            let (negative_emotion, positive_emotion) = if let Some(emotion) = emotion_opt {
                let neg = (emotion.get(EmotionType::Fear)
                    + emotion.get(EmotionType::Anger)
                    + emotion.get(EmotionType::Sadness)
                    + emotion.get(EmotionType::Disgust)
                    + emotion.get(EmotionType::Surprise))
                    as f32
                    / 5.0;
                let pos = (emotion.get(EmotionType::Joy)
                    + emotion.get(EmotionType::Trust)
                    + emotion.get(EmotionType::Anticipation))
                    as f32
                    / 3.0;
                (neg, pos)
            } else {
                (0.0, 0.0)
            };

            let need_pressure = hunger_critical
                + critical[0]
                + critical[1]
                + critical[2]
                + (1.0 - energy).clamp(0.0, 1.0)
                + (1.0 - social).clamp(0.0, 1.0);
            let stress_delta = need_pressure * 0.02 + negative_emotion * 0.01 - positive_emotion * 0.005;

            let next_level: f32 = (stress.level as f32 + stress_delta).clamp(0.0, 1.0);
            let reserve_delta: f32 = if next_level > 0.6 {
                -0.002 * ((next_level - 0.6) / 0.4)
            } else {
                0.001 * ((0.6 - next_level) / 0.6)
            };
            let next_reserve: f32 = (stress.reserve as f32 + reserve_delta).clamp(0.0, 1.0);
            let allostatic_delta: f32 = if next_level > 0.7 {
                0.0015 * ((next_level - 0.7) / 0.3)
            } else {
                -0.0005
            };
            let next_allostatic: f32 =
                (stress.allostatic_load as f32 + allostatic_delta).clamp(0.0, 1.0);

            stress.level = next_level as f64;
            stress.reserve = next_reserve as f64;
            stress.allostatic_load = next_allostatic as f64;
            stress.recalculate_state();
        }
    }
}

/// Rust runtime system for emotion baseline/primary updates.
///
/// This performs active-write updates on `Emotion.primary` and `Emotion.baseline`
/// using stress/needs context and personality-adjusted decay.
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

#[inline]
fn personality_z(personality_opt: Option<&Personality>, axis: HexacoAxis) -> f32 {
    let axis_val = personality_opt
        .map(|personality| personality.axis(axis) as f32)
        .unwrap_or(0.5);
    ((axis_val - 0.5) / 0.15).clamp(-3.0, 3.0)
}

#[inline]
fn needs_value(needs_opt: Option<&Needs>, need_type: NeedType, fallback: f32) -> f32 {
    needs_opt
        .map(|needs| needs.get(need_type) as f32)
        .unwrap_or(fallback)
        .clamp(0.0, 1.0)
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
        let mut query = world.query::<(
            &mut Emotion,
            Option<&Stress>,
            Option<&Needs>,
            Option<&Personality>,
        )>();
        for (_, (emotion, stress_opt, needs_opt, personality_opt)) in &mut query {
            let stress_level = stress_opt
                .map(|stress| stress.level as f32)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);

            let hunger = needs_value(needs_opt, NeedType::Hunger, 1.0);
            let thirst = needs_value(needs_opt, NeedType::Thirst, 1.0);
            let warmth = needs_value(needs_opt, NeedType::Warmth, 1.0);
            let safety = needs_value(needs_opt, NeedType::Safety, 1.0);
            let social = needs_value(needs_opt, NeedType::Belonging, 1.0);
            let energy = needs_opt
                .map(|needs| needs.energy as f32)
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);

            let deficit_hunger = (1.0 - hunger).clamp(0.0, 1.0);
            let deficit_thirst = (1.0 - thirst).clamp(0.0, 1.0);
            let deficit_warmth = (1.0 - warmth).clamp(0.0, 1.0);
            let deficit_safety = (1.0 - safety).clamp(0.0, 1.0);
            let deficit_social = (1.0 - social).clamp(0.0, 1.0);
            let deficit_energy = (1.0 - energy).clamp(0.0, 1.0);

            let z_e = personality_z(personality_opt, HexacoAxis::E);
            let z_a = personality_z(personality_opt, HexacoAxis::A);
            let z_c = personality_z(personality_opt, HexacoAxis::C);

            let baseline_fear = body::emotion_baseline_value(0.10, 0.02, z_e, 0.0, 1.0);
            let baseline_anger = body::emotion_baseline_value(0.08, -0.02, z_a, 0.0, 1.0);
            let baseline_sadness = body::emotion_baseline_value(0.08, 0.015, z_e, 0.0, 1.0);
            let baseline_disgust = body::emotion_baseline_value(0.06, -0.01, z_a, 0.0, 1.0);
            let baseline_surprise = body::emotion_baseline_value(0.10, 0.01, z_e, 0.0, 1.0);
            let baseline_joy = body::emotion_baseline_value(0.25, -0.02, z_e, 0.0, 1.0);
            let baseline_trust = body::emotion_baseline_value(0.22, 0.02, z_a, 0.0, 1.0);
            let baseline_anticipation = body::emotion_baseline_value(0.20, 0.01, z_c, 0.0, 1.0);

            let fear_target = (baseline_fear + stress_level * 0.50 + deficit_safety * 0.35).clamp(0.0, 1.0);
            let anger_target = (baseline_anger + stress_level * 0.35 + deficit_hunger * 0.20).clamp(0.0, 1.0);
            let sadness_target = (baseline_sadness + stress_level * 0.30 + deficit_social * 0.30).clamp(0.0, 1.0);
            let disgust_target = (baseline_disgust
                + stress_level * 0.18
                + (deficit_thirst * 0.10 + deficit_warmth * 0.10))
                .clamp(0.0, 1.0);
            let surprise_target = (baseline_surprise + stress_level * 0.15).clamp(0.0, 1.0);
            let joy_target = (baseline_joy
                - stress_level * 0.35
                + social * 0.20
                + energy * 0.15
                - deficit_energy * 0.10)
                .clamp(0.0, 1.0);
            let trust_target = (baseline_trust - stress_level * 0.25 + social * 0.20 + safety * 0.20)
                .clamp(0.0, 1.0);
            let anticipation_target =
                (baseline_anticipation - stress_level * 0.20 + energy * 0.20 + safety * 0.15)
                    .clamp(0.0, 1.0);

            let hl = body::emotion_adjusted_half_life(4.0, 0.25, z_c).max(0.001);
            let k = 0.693_147 / hl;
            let decay = (-k).exp();
            let blend = (1.0 - decay).clamp(0.0, 1.0);

            let update = |current: f64, target: f32| -> f64 {
                ((current as f32) * decay + target * blend).clamp(0.0, 1.0) as f64
            };

            emotion.baseline[EmotionType::Fear as usize] = baseline_fear as f64;
            emotion.baseline[EmotionType::Anger as usize] = baseline_anger as f64;
            emotion.baseline[EmotionType::Sadness as usize] = baseline_sadness as f64;
            emotion.baseline[EmotionType::Disgust as usize] = baseline_disgust as f64;
            emotion.baseline[EmotionType::Surprise as usize] = baseline_surprise as f64;
            emotion.baseline[EmotionType::Joy as usize] = baseline_joy as f64;
            emotion.baseline[EmotionType::Trust as usize] = baseline_trust as f64;
            emotion.baseline[EmotionType::Anticipation as usize] = baseline_anticipation as f64;

            *emotion.get_mut(EmotionType::Fear) = update(emotion.get(EmotionType::Fear), fear_target);
            *emotion.get_mut(EmotionType::Anger) = update(emotion.get(EmotionType::Anger), anger_target);
            *emotion.get_mut(EmotionType::Sadness) =
                update(emotion.get(EmotionType::Sadness), sadness_target);
            *emotion.get_mut(EmotionType::Disgust) =
                update(emotion.get(EmotionType::Disgust), disgust_target);
            *emotion.get_mut(EmotionType::Surprise) =
                update(emotion.get(EmotionType::Surprise), surprise_target);
            *emotion.get_mut(EmotionType::Joy) = update(emotion.get(EmotionType::Joy), joy_target);
            *emotion.get_mut(EmotionType::Trust) =
                update(emotion.get(EmotionType::Trust), trust_target);
            *emotion.get_mut(EmotionType::Anticipation) =
                update(emotion.get(EmotionType::Anticipation), anticipation_target);
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
        for (_, (needs, skills_opt, values_opt, behavior_opt, identity_opt, social_opt)) in
            &mut query
        {
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
        EmotionRuntimeSystem, NeedsRuntimeSystem, ResourceRegenSystem, StressRuntimeSystem,
        UpperNeedsRuntimeSystem,
    };
    use crate::body;
    use hecs::World;
    use sim_core::components::{
        Behavior, Body as BodyComponent, Emotion, Identity, Needs, Personality, Position,
        SkillEntry, Skills, Social, Stress, Values,
    };
    use sim_core::ids::EntityId;
    use sim_core::world::TileResource;
    use sim_core::{
        config::GameConfig, ActionType, EmotionType, GameCalendar, GrowthStage, NeedType,
        ResourceType, SettlementId, ValueType, WorldMap,
    };
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
    fn stress_runtime_system_updates_stress_state_and_components() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.05);
        needs.set(NeedType::Thirst, 0.10);
        needs.set(NeedType::Warmth, 0.08);
        needs.set(NeedType::Safety, 0.10);
        needs.set(NeedType::Belonging, 0.20);
        needs.energy = 0.15;

        let mut emotion = Emotion::default();
        emotion.add(EmotionType::Fear, 0.7);
        emotion.add(EmotionType::Anger, 0.6);
        emotion.add(EmotionType::Sadness, 0.5);

        let stress = Stress::default();
        let entity = world.spawn((needs, stress, emotion));

        let mut system = StressRuntimeSystem::new(34, 4);
        system.run(&mut world, &mut resources, 4);

        let updated = world
            .get::<&Stress>(entity)
            .expect("updated stress component should be queryable");
        assert!(updated.level > 0.0);
        assert!(updated.reserve <= 1.0);
        assert!(updated.allostatic_load >= 0.0);
    }

    #[test]
    fn emotion_runtime_system_updates_primary_and_baseline_values() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut emotion = Emotion::default();
        *emotion.get_mut(EmotionType::Fear) = 0.1;
        *emotion.get_mut(EmotionType::Joy) = 0.2;

        let mut needs = Needs::default();
        needs.set(NeedType::Safety, 0.2);
        needs.set(NeedType::Belonging, 0.3);
        needs.set(NeedType::Hunger, 0.25);
        needs.energy = 0.3;

        let stress = Stress {
            level: 0.8,
            reserve: 0.5,
            allostatic_load: 0.2,
            ..Stress::default()
        };

        let personality = Personality::default();
        let entity = world.spawn((emotion, needs, stress, personality));
        let mut system = EmotionRuntimeSystem::new(32, 12);
        system.run(&mut world, &mut resources, 12);

        let updated = world
            .get::<&Emotion>(entity)
            .expect("updated emotion component should be queryable");
        assert!(updated.get(EmotionType::Fear) > 0.1);
        assert!(updated.get(EmotionType::Joy) < 0.2);
        assert!(updated.baseline(EmotionType::Trust) >= 0.0);
        assert!(updated.baseline(EmotionType::Trust) <= 1.0);
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
        skills.entries.insert(
            "SKILL_FORAGING".to_string(),
            SkillEntry { level: 78, xp: 0.0 },
        );
        skills.entries.insert(
            "SKILL_MINING".to_string(),
            SkillEntry { level: 40, xp: 0.0 },
        );

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
