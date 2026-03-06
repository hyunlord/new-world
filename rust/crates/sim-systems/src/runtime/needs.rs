#![allow(unused_imports)]

use hecs::{Entity, World};
use rand::Rng;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use sim_core::components::{
    Age, Behavior, Body as BodyComponent, Coping, Economic, Emotion, Identity, Intelligence, Memory,
    MemoryEntry, Needs, Personality, Position, Skills, Social, Stress, Traits, Values,
};
use sim_core::config;
use sim_core::{
    ActionType, AttachmentType, EmotionType, GrowthStage, HexacoAxis, HexacoFacet,
    BuildingId, CopingStrategyId, EntityId, IntelligenceType, MentalBreakType, NeedType, RelationType, ResourceType,
    SettlementId, Sex, SocialClass, TechState, ValueType,
};
use sim_engine::{SimResources, SimSystem};
use sim_core::scales::{NativeStress, NativePercent};

use crate::body;


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
                let x = position.tile_x();
                let y = position.tile_y();
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

/// Rust runtime system for child-stage stress processing.
///
/// This performs active writes on `Stress.level/reserve/allostatic_load` for
/// infant/toddler/child/teen entities using child-stress body kernels.
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
fn child_stage_code(stage: GrowthStage) -> Option<i32> {
    match stage {
        GrowthStage::Infant => Some(0),
        GrowthStage::Toddler => Some(1),
        GrowthStage::Child => Some(2),
        GrowthStage::Teen => Some(3),
        _ => None,
    }
}

#[inline]
fn child_stage_multipliers(stage_code: i32) -> (f32, f32, f32) {
    match stage_code {
        0 => (0.90, 1.20, 1.20),
        1 => (1.00, 1.10, 1.10),
        2 => (1.10, 1.00, 1.00),
        3 => (1.20, 1.05, 1.00),
        _ => (1.00, 1.00, 1.00),
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
        let mut query = world.query::<(
            &Age,
            &mut Stress,
            Option<&Needs>,
            Option<&Social>,
            Option<&Personality>,
        )>();
        for (_, (age, stress, needs_opt, social_opt, personality_opt)) in &mut query {
            let Some(stage_code) = child_stage_code(age.stage) else {
                continue;
            };

            let belonging = needs_opt
                .map(|needs| needs.get(NeedType::Belonging) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let safety = needs_opt
                .map(|needs| needs.get(NeedType::Safety) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let energy = needs_opt
                .map(|needs| needs.energy as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let hunger = needs_opt
                .map(|needs| needs.get(NeedType::Hunger) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let base_intensity = (1.0 - (belonging * 0.35 + safety * 0.35 + energy * 0.20 + hunger * 0.10))
                .clamp(0.0, 1.0);
            if base_intensity <= 0.05 {
                continue;
            }

            let caregiver_present = social_opt
                .map(|social| !social.parents.is_empty() || social.spouse.is_some())
                .unwrap_or(false);
            let attachment_quality = social_opt
                .map(|social| social.reputation_local as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let buffered_intensity = body::child_social_buffered_intensity(
                base_intensity,
                attachment_quality,
                caregiver_present,
                0.5,
            )
            .clamp(0.0, 1.0);
            let stress_type = body::child_stress_type_code(
                buffered_intensity,
                caregiver_present,
                attachment_quality,
            );
            let (spike_mult, vulnerability_mult, break_threshold_mult) =
                child_stage_multipliers(stage_code);
            let resilience = personality_opt
                .map(|personality| personality.axis(HexacoAxis::C) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let out = body::child_stress_apply_step(
                resilience,
                stress.reserve_native().0.clamp(0.0, 100.0),
                stress.level_native().0.clamp(0.0, 2000.0),
                stress.allostatic_native().0.clamp(0.0, 100.0),
                buffered_intensity,
                spike_mult,
                vulnerability_mult,
                break_threshold_mult,
                stress_type,
            );
            stress.set_reserve_native(NativePercent(out[1]));
            stress.set_level_native(NativeStress(out[2]));
            stress.set_allostatic_native(NativePercent(out[3]));
            stress.recalculate_state();
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
