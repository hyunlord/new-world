#![allow(unused_imports)]

use hecs::{Entity, World};
use rand::Rng;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use sim_core::components::{
    Age, Behavior, Body as BodyComponent, Coping, CopingRebound, Economic, Emotion, Identity,
    Intelligence, Memory, MemoryEntry, Needs, Personality, Position, Skills, Social, Stress, Traits,
    Values,
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
use crate::stat_curve;


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
        let mut query = world.query::<(
            &Needs,
            &mut Stress,
            Option<&Emotion>,
            Option<&Personality>,
            Option<&Social>,
            Option<&Coping>,
            Option<&Memory>,
            Option<&Behavior>,
            Option<&Identity>,
        )>();
        for (_, (needs, stress, emotion_opt, personality_opt, social_opt, coping_opt, memory_opt, behavior_opt, identity_opt)) in &mut query {
            // --- Gather inputs ---

            // Needs (normalized 0-1)
            let hunger = needs.get(NeedType::Hunger) as f32;
            let energy = needs.energy as f32;
            let social = needs.get(NeedType::Belonging) as f32;

            // HEXACO personality axes (0-1)
            let e_axis = personality_opt.map(|p| p.axis(HexacoAxis::E) as f32).unwrap_or(0.5);
            let c_axis = personality_opt.map(|p| p.axis(HexacoAxis::C) as f32).unwrap_or(0.5);
            let x_axis = personality_opt.map(|p| p.axis(HexacoAxis::X) as f32).unwrap_or(0.5);
            let o_axis = personality_opt.map(|p| p.axis(HexacoAxis::O) as f32).unwrap_or(0.5);
            let a_axis = personality_opt.map(|p| p.axis(HexacoAxis::A) as f32).unwrap_or(0.5);
            let h_axis = personality_opt.map(|p| p.axis(HexacoAxis::H) as f32).unwrap_or(0.5);

            // Emotion values (0-1)
            let (fear, anger, sadness, disgust, surprise, joy, trust_val, anticipation) =
                if let Some(emotion) = emotion_opt {
                    (
                        emotion.get(EmotionType::Fear) as f32,
                        emotion.get(EmotionType::Anger) as f32,
                        emotion.get(EmotionType::Sadness) as f32,
                        emotion.get(EmotionType::Disgust) as f32,
                        emotion.get(EmotionType::Surprise) as f32,
                        emotion.get(EmotionType::Joy) as f32,
                        emotion.get(EmotionType::Trust) as f32,
                        emotion.get(EmotionType::Anticipation) as f32,
                    )
                } else {
                    (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
                };

            // Compute valence/arousal from Plutchik primaries
            let pos_avg = (joy + trust_val + anticipation) / 3.0;
            let neg_avg = (fear + anger + sadness + disgust) / 4.0;
            let valence = (pos_avg - neg_avg).clamp(-1.0, 1.0);
            let arousal = ((anger + fear + surprise + anticipation) / 4.0).clamp(0.0, 1.0);

            // Support score from relationship trust values
            let support_score = if let Some(social_comp) = social_opt {
                let strengths: Vec<f32> = social_comp
                    .edges
                    .iter()
                    .map(|e| e.trust as f32)
                    .collect();
                body::stress_support_score(&strengths)
            } else {
                0.3
            };

            // Denial state from Coping
            let denial_active = coping_opt
                .map(|c| c.active_strategy == Some(CopingStrategyId::Denial))
                .unwrap_or(false);

            // Trauma scar resilience modifier (scars reduce resilience)
            let scar_resilience_mod = memory_opt
                .map(|m| {
                    m.trauma_scars
                        .iter()
                        .map(|s| s.severity as f32 * -0.05)
                        .sum::<f32>()
                        .clamp(-0.5, 0.0)
                })
                .unwrap_or(0.0);

            // State flags
            let is_sleeping = behavior_opt
                .map(|b| b.current_action == ActionType::Sleep)
                .unwrap_or(false);
            let is_safe = identity_opt
                .map(|id| id.settlement_id.is_some())
                .unwrap_or(false);

            // Scale to stat_curve native scales (stress 0-2000, reserve/allostatic/resilience 0-100)
            let stress_native = stress.level_native().0;
            let reserve_native = stress.reserve_native().0;
            let allostatic_native = stress.allostatic_native().0;
            let resilience_native = stress.resilience_native().0;
            let reserve_ratio = stress.reserve as f32; // already 0-1

            // Stress trace arrays
            let per_tick: Vec<f32> = stress.stress_traces.iter().map(|t| t.per_tick).collect();
            let decay_rates: Vec<f32> = stress.stress_traces.iter().map(|t| t.decay_rate).collect();

            // --- Call full Lazarus/Folkman stress tick step ---
            let result = stat_curve::stress_tick_step(
                &per_tick,
                &decay_rates,
                0.01, // min_keep
                hunger,
                energy,
                social,
                0.0, // threat (not yet tracked)
                0.0, // conflict (not yet tracked)
                support_score,
                e_axis,
                fear,
                trust_val,
                c_axis,
                o_axis,
                reserve_ratio,
                anger,
                sadness,
                disgust,
                surprise,
                joy,
                anticipation,
                valence,
                arousal,
                stress_native,
                resilience_native,
                reserve_native,
                stress.stress_delta_last,
                stress.gas_stage,
                is_sleeping,
                is_safe,
                allostatic_native,
                1.0,  // ace_stress_mult (default)
                1.0,  // trait_accum_mult (default)
                0.05, // epsilon
                denial_active,
                0.60, // denial_redirect_fraction (Gross 1998)
                stress.hidden_threat_accumulator,
                800.0, // denial_max_accumulator
                1.0,   // avoidant_allostatic_mult (default)
                e_axis,
                c_axis,
                x_axis,
                o_axis,
                a_axis,
                h_axis,
                scar_resilience_mod,
            );

            // --- Write outputs (scale back to 0-1 normalized) ---
            stress.set_level_native(NativeStress(result.stress));
            stress.set_reserve_native(NativePercent(result.reserve));
            stress.set_allostatic_native(NativePercent(result.allostatic));
            stress.set_resilience_native(NativePercent(result.resilience));
            stress.gas_stage = result.gas_stage;
            stress.stress_delta_last = result.delta;
            stress.hidden_threat_accumulator = result.hidden_threat_accumulator;
            stress.recalculate_state();

            // Update stress traces: apply updated per_tick and remove decayed traces
            let mut retained = Vec::new();
            for (i, trace) in stress.stress_traces.iter().enumerate() {
                if i < result.active_mask.len() && result.active_mask[i] != 0 {
                    let mut t = trace.clone();
                    if i < result.updated_per_tick.len() {
                        t.per_tick = result.updated_per_tick[i];
                    }
                    retained.push(t);
                }
            }
            stress.stress_traces = retained;

            // Shaken state countdown (Yerkes-Dodson work efficiency)
            if stress.shaken_remaining > 0 {
                let shaken = body::stress_shaken_countdown_step(stress.shaken_remaining);
                stress.shaken_remaining = shaken[0] as i32;
                if shaken[1] != 0.0 {
                    stress.shaken_work_penalty = 0.0;
                }
            }
        }
    }
}

/// Rust runtime system for mental-break trigger and recovery.
///
/// This performs active writes on `Stress.active_mental_break`,
/// `Stress.mental_break_remaining`, and `Stress.mental_break_count`.
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

const MENTAL_BREAK_BASE_THRESHOLD: f32 = 520.0;
const MENTAL_BREAK_THRESHOLD_MIN: f32 = 420.0;
const MENTAL_BREAK_THRESHOLD_MAX: f32 = 900.0;
const MENTAL_BREAK_SCALE: f32 = 6000.0;
const MENTAL_BREAK_CAP_PER_TICK: f32 = 0.25;
const MENTAL_BREAK_OUTRAGE_THRESHOLD: f32 = 60.0;

#[inline]
fn mental_break_type_from_code(code: i32) -> MentalBreakType {
    match code {
        1 => MentalBreakType::OutrageViolence,
        2 => MentalBreakType::Panic,
        3 => MentalBreakType::Rage,
        5 => MentalBreakType::Purge,
        _ => MentalBreakType::Shutdown,
    }
}

#[inline]
fn mental_break_duration_ticks(kind: MentalBreakType) -> u32 {
    match kind {
        MentalBreakType::OutrageViolence => 24,
        MentalBreakType::Panic => 16,
        MentalBreakType::Rage => 20,
        MentalBreakType::Purge => 18,
        _ => 30,
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

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(
            &mut Stress,
            Option<&Needs>,
            Option<&Personality>,
            Option<&Emotion>,
        )>();
        for (_, (stress, needs_opt, personality_opt, emotion_opt)) in &mut query {
            if stress.active_mental_break.is_some() {
                let dec = self.tick_interval.min(u32::MAX as u64) as u32;
                if stress.mental_break_remaining > dec {
                    stress.mental_break_remaining -= dec;
                } else {
                    stress.mental_break_remaining = 0;
                    stress.active_mental_break = None;
                    stress.level = (stress.level as f32 * 0.80).clamp(0.0, 1.0) as f64;
                    // Enter shaken state (Yerkes-Dodson work efficiency penalty)
                    stress.shaken_remaining = 48;
                    stress.shaken_work_penalty = -0.25;
                    stress.recalculate_state();
                }
                continue;
            }

            if stress.level < 0.70 {
                continue;
            }

            let c_axis = personality_opt
                .map(|personality| personality.axis(HexacoAxis::C) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let e_axis = personality_opt
                .map(|personality| personality.axis(HexacoAxis::E) as f32)
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
            let resilience = c_axis;
            let reserve_scaled = stress.reserve_native().0.clamp(0.0, 100.0);
            let allostatic_scaled = stress.allostatic_native().0.clamp(0.0, 100.0);
            let stress_scaled = stress.level_native().0.clamp(0.0, 4000.0);

            let threshold = body::mental_break_threshold(
                MENTAL_BREAK_BASE_THRESHOLD,
                resilience,
                c_axis,
                e_axis,
                allostatic_scaled,
                energy,
                hunger,
                1.0,
                0.0,
                MENTAL_BREAK_THRESHOLD_MIN,
                MENTAL_BREAK_THRESHOLD_MAX,
                reserve_scaled,
                0.0,
            );
            let trigger_p = body::mental_break_chance(
                stress_scaled,
                threshold,
                reserve_scaled,
                allostatic_scaled,
                MENTAL_BREAK_SCALE,
                MENTAL_BREAK_CAP_PER_TICK,
            )
            .clamp(0.0, 1.0);
            if trigger_p <= 0.0 {
                continue;
            }

            let roll: f32 = resources.rng.gen_range(0.0..1.0);
            if roll >= trigger_p {
                continue;
            }

            let (fear, anger, sadness, disgust) = if let Some(emotion) = emotion_opt {
                (
                    emotion.get(EmotionType::Fear) as f32 * 100.0,
                    emotion.get(EmotionType::Anger) as f32 * 100.0,
                    emotion.get(EmotionType::Sadness) as f32 * 100.0,
                    emotion.get(EmotionType::Disgust) as f32 * 100.0,
                )
            } else {
                (20.0, 20.0, 30.0, 15.0)
            };
            let outrage = (anger * 0.6 + fear * 0.4).clamp(0.0, 100.0);
            let break_code = body::emotion_break_type_code(
                outrage,
                fear,
                anger,
                sadness,
                disgust,
                MENTAL_BREAK_OUTRAGE_THRESHOLD,
            );
            let break_type = mental_break_type_from_code(break_code);
            stress.active_mental_break = Some(break_type);
            stress.mental_break_count = stress.mental_break_count.saturating_add(1);
            stress.mental_break_remaining = mental_break_duration_ticks(break_type);
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TraitViolationHistory {
    desensitize_mult: f32,
    ptsd_mult: f32,
    repeats: u32,
    last_violation_tick: u64,
}

/// Rust runtime system for trait-violation stress updates.
///
/// This performs active writes on `Stress.level/reserve/allostatic_load` using
/// trait-context mismatch checks and persistent desensitization/PTSD history.
#[derive(Debug, Clone)]
pub struct TraitViolationRuntimeSystem {
    priority: u32,
    tick_interval: u64,
    history: HashMap<Entity, TraitViolationHistory>,
}

impl TraitViolationRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
            history: HashMap::new(),
        }
    }
}

const TRAIT_VIOLATION_DESENSITIZE_DECAY: f32 = 0.85;
const TRAIT_VIOLATION_DESENSITIZE_MIN: f32 = 0.30;
const TRAIT_VIOLATION_PTSD_INCREASE: f32 = 1.10;
const TRAIT_VIOLATION_PTSD_MAX: f32 = 2.0;
const TRAIT_VIOLATION_ALLOSTATIC_THRESHOLD: f32 = 0.50;
const TRAIT_VIOLATION_BASE_INTRUSIVE_CHANCE: f32 = 0.005;
const TRAIT_VIOLATION_HISTORY_DECAY_TICKS: i32 = 365 * 12;

#[inline]
fn trait_violation_action_base(action: ActionType) -> Option<(&'static str, f32)> {
    match action {
        ActionType::TakeFromStockpile => Some(("steal", 22.0)),
        ActionType::Fight => Some(("harm_innocent", 20.0)),
        ActionType::MentalBreak => Some(("panic", 10.0)),
        ActionType::Flee => Some(("retreat", 12.0)),
        _ => None,
    }
}

#[inline]
fn trait_violation_matching_facet(
    action_key: &str,
    traits_opt: Option<&Traits>,
    personality_opt: Option<&Personality>,
) -> Option<f32> {
    let traits = traits_opt?;
    let personality = personality_opt?;
    let match_trait = |id: &str| traits.has_trait(id);
    match action_key {
        "steal" if match_trait("f_fair_minded") => Some(personality.facet(HexacoFacet::Fairness) as f32),
        "steal" if match_trait("f_sincere") => Some(personality.facet(HexacoFacet::Sincerity) as f32),
        "harm_innocent" if match_trait("f_sentimental") => {
            Some(personality.facet(HexacoFacet::Sentimentality) as f32)
        }
        "harm_innocent" if match_trait("f_gentle") => Some(personality.facet(HexacoFacet::Gentleness) as f32),
        "panic" if match_trait("f_calm") => Some(personality.facet(HexacoFacet::Anxiety) as f32),
        "retreat" if match_trait("f_fearless") => Some(personality.facet(HexacoFacet::Fearfulness) as f32),
        _ => None,
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

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        let mut query = world.query::<(
            &mut Stress,
            Option<&Behavior>,
            Option<&Needs>,
            Option<&Social>,
            Option<&Traits>,
            Option<&Personality>,
        )>();
        for (entity, (stress, behavior_opt, needs_opt, social_opt, traits_opt, personality_opt)) in
            &mut query
        {
            let mut history = self.history.get(&entity).copied().unwrap_or(TraitViolationHistory {
                desensitize_mult: 1.0,
                ptsd_mult: 1.0,
                repeats: 0,
                last_violation_tick: tick,
            });

            if let Some(behavior) = behavior_opt {
                if let Some((action_key, base_stress)) =
                    trait_violation_action_base(behavior.current_action)
                {
                    if let Some(facet_value) =
                        trait_violation_matching_facet(action_key, traits_opt, personality_opt)
                    {
                        let survival_necessity = matches!(action_key, "steal")
                            && needs_opt
                                .map(|needs| (needs.get(NeedType::Hunger) as f32) < 0.2)
                                .unwrap_or(false);
                        let no_witness = social_opt
                            .map(|social| (social.social_capital as f32) < 0.2)
                            .unwrap_or(true);
                        let is_habit = history.repeats >= 3
                            && tick.saturating_sub(history.last_violation_tick) < 180;
                        let context_mult = body::trait_violation_context_modifier(
                            is_habit,
                            false,
                            survival_necessity,
                            no_witness,
                            0.0,
                            0.5,
                            0.4,
                            0.85,
                        );
                        if context_mult > 0.0 {
                            let facet_scale =
                                body::trait_violation_facet_scale(facet_value.clamp(0.0, 1.0), 0.6);
                            let stress_delta = (base_stress
                                * facet_scale
                                * history.desensitize_mult
                                * history.ptsd_mult
                                * context_mult
                                / 400.0)
                                .clamp(0.0, 0.25);
                            if stress_delta > 0.0 {
                                let next_level = (stress.level as f32 + stress_delta).clamp(0.0, 1.0);
                                let next_allostatic = (stress.allostatic_load as f32 + stress_delta * 0.45)
                                    .clamp(0.0, 1.0);
                                let next_reserve =
                                    (stress.reserve as f32 - stress_delta * 0.35).clamp(0.0, 1.0);
                                stress.level = next_level as f64;
                                stress.allostatic_load = next_allostatic as f64;
                                stress.reserve = next_reserve as f64;
                                stress.recalculate_state();

                                if next_allostatic < TRAIT_VIOLATION_ALLOSTATIC_THRESHOLD {
                                    history.desensitize_mult = (history.desensitize_mult
                                        * TRAIT_VIOLATION_DESENSITIZE_DECAY)
                                        .max(TRAIT_VIOLATION_DESENSITIZE_MIN);
                                } else {
                                    history.ptsd_mult = (history.ptsd_mult
                                        * TRAIT_VIOLATION_PTSD_INCREASE)
                                        .min(TRAIT_VIOLATION_PTSD_MAX);
                                }
                                history.repeats = history.repeats.saturating_add(1);
                                history.last_violation_tick = tick;
                            }
                        } else {
                            history.repeats = history.repeats.saturating_add(1);
                            history.last_violation_tick = tick;
                        }
                    }
                }
            }

            let ticks_since = tick.saturating_sub(history.last_violation_tick) as i32;
            let intrusive_p = body::trait_violation_intrusive_chance(
                TRAIT_VIOLATION_BASE_INTRUSIVE_CHANCE,
                history.ptsd_mult,
                ticks_since,
                TRAIT_VIOLATION_HISTORY_DECAY_TICKS,
                false,
            )
            .clamp(0.0, 1.0);
            if intrusive_p > 0.0 {
                let roll: f32 = resources.rng.gen_range(0.0..1.0);
                if roll < intrusive_p {
                    let intrusive_stress = (0.004 * history.ptsd_mult).clamp(0.0, 0.03);
                    stress.level = (stress.level as f32 + intrusive_stress).clamp(0.0, 1.0) as f64;
                    stress.allostatic_load = (stress.allostatic_load as f32 + intrusive_stress * 0.20)
                        .clamp(0.0, 1.0) as f64;
                    stress.recalculate_state();
                }
            }

            if ticks_since > TRAIT_VIOLATION_HISTORY_DECAY_TICKS {
                history.repeats = history.repeats.saturating_sub(1);
                history.desensitize_mult =
                    (history.desensitize_mult + 0.01).clamp(TRAIT_VIOLATION_DESENSITIZE_MIN, 1.0);
                history.ptsd_mult = (history.ptsd_mult - 0.01).clamp(1.0, TRAIT_VIOLATION_PTSD_MAX);
            }
            self.history.insert(entity, history);
        }
    }
}

/// Rust runtime system for trauma-scar baseline drift.
///
/// This performs active writes on `Emotion.baseline` from persistent
/// `Memory.trauma_scars` entries.
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

const TRAUMA_SCAR_BASELINE_SCALE: f32 = 0.001;

#[inline]
fn trauma_apply_baseline_delta(emotion: &mut Emotion, emotion_type: EmotionType, delta: f32) {
    let idx = emotion_type as usize;
    let next = (emotion.baseline[idx] as f32 + delta).clamp(0.0, 1.0);
    emotion.baseline[idx] = next as f64;
}

fn trauma_scar_apply_baseline_shifts(emotion: &mut Emotion, scar_id: &str, scale: f32) {
    match scar_id {
        "hypervigilance" => {
            trauma_apply_baseline_delta(emotion, EmotionType::Fear, 0.10 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Joy, -0.05 * scale);
        }
        "anger_dysregulation" => {
            trauma_apply_baseline_delta(emotion, EmotionType::Anger, 0.08 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Joy, -0.05 * scale);
        }
        "violence_imprint" => {
            trauma_apply_baseline_delta(emotion, EmotionType::Fear, 0.08 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Joy, -0.10 * scale);
        }
        "emotional_numbness" => {
            trauma_apply_baseline_delta(emotion, EmotionType::Sadness, 0.10 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Trust, -0.10 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Joy, -0.15 * scale);
        }
        "compulsive_consumption" => {
            trauma_apply_baseline_delta(emotion, EmotionType::Anticipation, 0.02 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Joy, -0.05 * scale);
        }
        "complicated_grief" => {
            trauma_apply_baseline_delta(emotion, EmotionType::Sadness, 0.15 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Joy, -0.15 * scale);
        }
        "dissociative_tendency" => {
            trauma_apply_baseline_delta(emotion, EmotionType::Sadness, 0.08 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Joy, -0.08 * scale);
        }
        "chronic_paranoia" => {
            trauma_apply_baseline_delta(emotion, EmotionType::Fear, 0.12 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Sadness, 0.15 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Joy, -0.10 * scale);
        }
        "anxious_attachment" => {
            trauma_apply_baseline_delta(emotion, EmotionType::Sadness, 0.12 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Joy, -0.05 * scale);
        }
        _ => {
            trauma_apply_baseline_delta(emotion, EmotionType::Fear, 0.05 * scale);
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
        let mut query = world.query::<(&Memory, &mut Emotion)>();
        for (_, (memory, emotion)) in &mut query {
            if memory.trauma_scars.is_empty() {
                continue;
            }
            for scar in &memory.trauma_scars {
                let severity = (scar.severity as f32).clamp(0.0, 1.0);
                let base_mult = 1.0 + severity * 0.25;
                let stacks = (scar.reactivation_count as i32).max(1);
                let sensitivity = body::trauma_scar_sensitivity_factor(base_mult, stacks)
                    .clamp(0.5, 3.0);
                let shift_scale = TRAUMA_SCAR_BASELINE_SCALE * sensitivity;
                trauma_scar_apply_baseline_shifts(emotion, scar.scar_id.as_str(), shift_scale);
            }
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

/// Rust runtime system for morale-driven behavior/need adjustment.
///
/// This computes a per-entity morale signal and applies active writes to
/// `Behavior.job_satisfaction`, `Behavior.occupation_satisfaction`,
/// `Needs.meaning`, and `Needs.transcendence`.
#[derive(Debug, Clone)]
pub struct MoraleRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl MoraleRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn maslow_morale_multiplier(hunger: f32, energy: f32, safety: f32, belonging: f32) -> f32 {
    if hunger < 0.3 || energy < 0.3 {
        0.0
    } else if hunger < 0.6 || energy < 0.6 {
        0.4
    } else if safety < 0.3 {
        0.2
    } else if safety < 0.6 {
        0.6
    } else if belonging < 0.3 {
        0.7
    } else {
        1.0
    }
}

impl SimSystem for MoraleRuntimeSystem {
    fn name(&self) -> &'static str {
        "morale_system"
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
            &mut Behavior,
            Option<&Emotion>,
            Option<&Stress>,
            Option<&Personality>,
            Option<&Social>,
        )>();
        for (_, (needs, behavior, emotion_opt, stress_opt, personality_opt, social_opt)) in &mut query {
            let hunger = needs.get(NeedType::Hunger) as f32;
            let energy = needs.energy as f32;
            let safety = needs.get(NeedType::Safety) as f32;
            let belonging = needs.get(NeedType::Belonging) as f32;

            let pa = emotion_opt
                .map(|emotion| {
                    ((emotion.get(EmotionType::Joy)
                        + emotion.get(EmotionType::Trust)
                        + emotion.get(EmotionType::Anticipation)) as f32
                        / 3.0)
                        .clamp(0.0, 1.0)
                })
                .unwrap_or(0.5);
            let na = emotion_opt
                .map(|emotion| {
                    ((emotion.get(EmotionType::Fear)
                        + emotion.get(EmotionType::Anger)
                        + emotion.get(EmotionType::Sadness)
                        + emotion.get(EmotionType::Disgust)
                        + emotion.get(EmotionType::Surprise)) as f32
                        / 5.0)
                        .clamp(0.0, 1.0)
                })
                .unwrap_or(0.5);
            let ls = ((hunger + energy + belonging) / 3.0).clamp(0.0, 1.0);
            let maslow_mult = maslow_morale_multiplier(hunger, energy, safety, belonging);

            let mut morale = (0.40 * pa - 0.30 * na + 0.30 * ls) * maslow_mult;
            let hygiene_threshold = 0.5;
            let hygiene_penalty_rate = 0.8;
            if safety < hygiene_threshold {
                morale -= (hygiene_threshold - safety) * hygiene_penalty_rate * 0.3;
            }
            if belonging < hygiene_threshold {
                morale -= (hygiene_threshold - belonging) * hygiene_penalty_rate * 0.2;
            }
            if hunger < hygiene_threshold {
                morale -= (hygiene_threshold - hunger) * hygiene_penalty_rate * 0.25;
            }
            if energy < hygiene_threshold {
                morale -= (hygiene_threshold - energy) * hygiene_penalty_rate * 0.25;
            }

            let extraversion = personality_opt
                .map(|personality| personality.axis(HexacoAxis::X) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let autonomy = needs.get(NeedType::Autonomy) as f32;
            let warr_autonomy = -1.5 * (autonomy - 0.6).powi(2) + 0.15;
            let warr_social = -2.0 * (belonging - 0.5).powi(2) + 0.12;
            let warr_info = (extraversion / 0.7).min(1.0) * 0.10;
            morale = (morale + (warr_autonomy + warr_social + warr_info).clamp(-0.3, 0.3))
                .clamp(-1.0, 1.0);

            let stress_level = stress_opt
                .map(|stress| stress.level as f32)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);
            let settlement_morale = if let Some(social) = social_opt {
                ((morale + social.social_capital as f32) * 0.5).clamp(-1.0, 1.0)
            } else {
                morale
            };
            let patience = personality_opt
                .map(|personality| personality.axis(HexacoAxis::C) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let migration_p = body::morale_migration_probability(
                settlement_morale,
                10.0,
                0.35,
                patience,
                0.3,
                0.95,
            );

            let behavior_weight = body::morale_behavior_weight_multiplier(
                morale,
                0.6,
                1.2,
                1.55,
                0.85,
                1.2,
                0.55,
                0.85,
                0.30,
                0.55,
            );
            let stress_penalty = (stress_level * 0.12).clamp(0.0, 0.12);
            behavior.job_satisfaction =
                (behavior.job_satisfaction + (behavior_weight - 1.0) * 0.12 - stress_penalty)
                    .clamp(0.0, 1.0);
            behavior.occupation_satisfaction = (behavior.occupation_satisfaction
                + (behavior_weight - 1.0) * 0.10
                - stress_penalty * 0.8)
                .clamp(0.0, 1.0);

            let morale_unit = ((morale + 1.0) * 0.5).clamp(0.0, 1.0);
            let next_meaning =
                (needs.get(NeedType::Meaning) as f32 * 0.85 + morale_unit * 0.15 - migration_p * 0.05)
                    .clamp(0.0, 1.0);
            let next_transcendence = (needs.get(NeedType::Transcendence) as f32 * 0.90
                + morale_unit * 0.10
                - migration_p * 0.03)
                .clamp(0.0, 1.0);
            needs.set(NeedType::Meaning, next_meaning as f64);
            needs.set(NeedType::Transcendence, next_transcendence as f64);
        }
    }
}

/// Rust runtime system for emotion/stress contagion propagation.
///
/// This performs active writes to `Emotion.primary` and `Stress.level` using
/// AoE and same-settlement network spread kernels.
#[derive(Debug, Clone)]
pub struct ContagionRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl ContagionRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}


#[derive(Debug, Clone)]
struct ContagionSnapshot {
    entity: Entity,
    x: i32,
    y: i32,
    settlement_id: Option<u64>,
    emotions: [f32; 8],
    stress_2000: f32,
    valence: f32,
    x_axis: f32,
    e_axis: f32,
    a_axis: f32,
}

#[inline]
fn contagion_valence(emotions: &[f32; 8]) -> f32 {
    let joy = emotions[EmotionType::Joy as usize];
    let trust = emotions[EmotionType::Trust as usize];
    let anticipation = emotions[EmotionType::Anticipation as usize];
    let fear = emotions[EmotionType::Fear as usize];
    let sadness = emotions[EmotionType::Sadness as usize];
    let anger = emotions[EmotionType::Anger as usize];
    let disgust = emotions[EmotionType::Disgust as usize];
    let surprise = emotions[EmotionType::Surprise as usize];
    let positive = (joy + trust + anticipation) / 3.0;
    let negative = (fear + sadness + anger + disgust + surprise) / 5.0;
    ((positive - negative) * 100.0).clamp(-100.0, 100.0)
}

impl SimSystem for ContagionRuntimeSystem {
    fn name(&self) -> &'static str {
        "contagion_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        const AOE_RADIUS: i32 = 3;
        const NETWORK_HOP_RADIUS: i32 = 15;
        const CROWD_DILUTE_DIVISOR: f32 = 6.0;
        const REFRACTORY_SUSCEPTIBILITY: f32 = 0.25;
        const BASE_MIMICRY_WEIGHT: f32 = 0.08;
        const MAX_EMOTION_CONTAGION_DELTA: f32 = 0.08;
        const MAX_STRESS_CONTAGION_DELTA: f32 = 30.0;
        const STRESS_SCALE: f32 = 2000.0;
        const NETWORK_DECAY: f32 = 0.5;

        let mut snapshots: Vec<ContagionSnapshot> = Vec::new();
        let mut read_query =
            world.query::<(&Position, &Emotion, &Stress, Option<&Personality>, Option<&Identity>)>();
        for (entity, (position, emotion, stress, personality_opt, identity_opt)) in &mut read_query {
            let mut emotions = [0.0_f32; 8];
            for (idx, value) in emotion.primary.iter().enumerate() {
                emotions[idx] = *value as f32;
            }
            let x_axis = personality_opt
                .map(|personality| personality.axis(HexacoAxis::X) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let e_axis = personality_opt
                .map(|personality| personality.axis(HexacoAxis::E) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let a_axis = personality_opt
                .map(|personality| personality.axis(HexacoAxis::A) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            snapshots.push(ContagionSnapshot {
                entity,
                x: position.x,
                y: position.y,
                settlement_id: identity_opt.and_then(|identity| identity.settlement_id.map(|id| id.0)),
                emotions,
                stress_2000: stress.level_native().0.clamp(0.0, STRESS_SCALE),
                valence: contagion_valence(&emotions),
                x_axis,
                e_axis,
                a_axis,
            });
        }
        drop(read_query);

        if snapshots.len() < 2 {
            return;
        }

        let mut emotion_deltas: HashMap<Entity, [f32; 8]> = HashMap::new();
        let mut stress_deltas: HashMap<Entity, f32> = HashMap::new();
        let emotion_keys: [EmotionType; 5] = [
            EmotionType::Joy,
            EmotionType::Sadness,
            EmotionType::Fear,
            EmotionType::Anger,
            EmotionType::Trust,
        ];

        for recipient in &snapshots {
            let mut donor_count = 0_i32;
            let mut donor_emotion_sums = [0.0_f32; 8];
            let mut donor_stress_sum = 0.0_f32;
            for donor in &snapshots {
                if donor.entity == recipient.entity {
                    continue;
                }
                let dist = (donor.x - recipient.x).abs() + (donor.y - recipient.y).abs();
                if dist > AOE_RADIUS {
                    continue;
                }
                donor_count += 1;
                donor_stress_sum += donor.stress_2000;
                for (idx, sum) in donor_emotion_sums.iter_mut().enumerate() {
                    *sum += donor.emotions[idx];
                }
            }
            if donor_count <= 0 {
                continue;
            }

            let total_susceptibility = body::contagion_aoe_total_susceptibility(
                donor_count,
                CROWD_DILUTE_DIVISOR,
                false,
                REFRACTORY_SUSCEPTIBILITY,
                recipient.x_axis,
                recipient.e_axis,
            );
            let mut entry = [0.0_f32; 8];
            for emotion_key in emotion_keys {
                let idx = emotion_key as usize;
                let donor_avg = donor_emotion_sums[idx] / donor_count as f32;
                let gap = donor_avg - recipient.emotions[idx];
                entry[idx] = (gap * BASE_MIMICRY_WEIGHT * total_susceptibility)
                    .clamp(-MAX_EMOTION_CONTAGION_DELTA, MAX_EMOTION_CONTAGION_DELTA);
            }
            emotion_deltas.insert(recipient.entity, entry);

            let avg_stress = donor_stress_sum / donor_count as f32;
            let stress_gap = avg_stress - recipient.stress_2000;
            let stress_delta = body::contagion_stress_delta(
                stress_gap,
                10.0,
                0.04,
                total_susceptibility,
                MAX_STRESS_CONTAGION_DELTA,
            ) / STRESS_SCALE;
            if stress_delta > 0.0 {
                stress_deltas
                    .entry(recipient.entity)
                    .and_modify(|value| *value += stress_delta)
                    .or_insert(stress_delta);
            }
        }

        for recipient in &snapshots {
            let Some(settlement_id) = recipient.settlement_id else {
                continue;
            };
            let mut donors: Vec<&ContagionSnapshot> = Vec::new();
            for donor in &snapshots {
                if donor.entity == recipient.entity {
                    continue;
                }
                if donor.settlement_id != Some(settlement_id) {
                    continue;
                }
                let dist = (donor.x - recipient.x).abs() + (donor.y - recipient.y).abs();
                if dist <= NETWORK_HOP_RADIUS {
                    donors.push(donor);
                }
            }
            if donors.is_empty() {
                continue;
            }
            let donor_count = donors.len() as i32;
            let avg_valence = donors.iter().map(|donor| donor.valence).sum::<f32>() / donor_count as f32;
            let valence_gap = avg_valence - recipient.valence;
            let valence_delta = body::contagion_network_delta(
                donor_count,
                CROWD_DILUTE_DIVISOR,
                false,
                REFRACTORY_SUSCEPTIBILITY,
                NETWORK_DECAY,
                recipient.a_axis,
                valence_gap,
                0.04,
                4.0,
            );
            if valence_delta.abs() <= 0.01 {
                continue;
            }
            let joy_delta = (valence_delta / 100.0).clamp(-0.04, 0.04);
            let sadness_delta = (-valence_delta / 100.0).clamp(-0.04, 0.04);
            let entry = emotion_deltas.entry(recipient.entity).or_insert([0.0_f32; 8]);
            entry[EmotionType::Joy as usize] += joy_delta;
            entry[EmotionType::Trust as usize] += joy_delta * 0.5;
            entry[EmotionType::Sadness as usize] += sadness_delta;
            entry[EmotionType::Fear as usize] += sadness_delta * 0.5;
        }

        for recipient in &snapshots {
            let spiral_increment = body::contagion_spiral_increment(
                recipient.stress_2000,
                recipient.valence,
                500.0,
                -40.0,
                1500.0,
                60.0,
                3.0,
                12.0,
            ) / STRESS_SCALE;
            if spiral_increment > 0.0 {
                stress_deltas
                    .entry(recipient.entity)
                    .and_modify(|value| *value += spiral_increment)
                    .or_insert(spiral_increment);
            }
        }

        let mut write_query = world.query::<(&mut Emotion, &mut Stress)>();
        for (entity, (emotion, stress)) in &mut write_query {
            if let Some(delta_emotions) = emotion_deltas.get(&entity) {
                for (idx, delta) in delta_emotions.iter().enumerate() {
                    let next = (emotion.primary[idx] as f32 + *delta).clamp(0.0, 1.0);
                    emotion.primary[idx] = next as f64;
                }
            }
            if let Some(stress_delta) = stress_deltas.get(&entity) {
                let next_level = (stress.level as f32 + *stress_delta).clamp(0.0, 1.0);
                stress.level = next_level as f64;
                if *stress_delta > 0.0 {
                    let next_allostatic = (stress.allostatic_load as f32 + *stress_delta * 0.25)
                        .clamp(0.0, 1.0);
                    stress.allostatic_load = next_allostatic as f64;
                }
                stress.recalculate_state();
            }
        }
    }
}


/// Rust runtime system for coping strategy cooldown/selection updates.
///
/// This performs active writes on `Coping.active_strategy`,
/// `Coping.strategy_cooldowns`, and `Coping.usage_counts`.
#[derive(Debug, Clone)]
pub struct CopingRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl CopingRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

const COPING_COUNT_MAX: f32 = 15.0;
const COPING_CLEAR_STRESS_MAX: f32 = 0.20;
const COPING_CLEAR_ALLOSTATIC_MAX: f32 = 0.20;
const COPING_RECOVERY_STRESS_MAX: f32 = 0.55;
const COPING_RECOVERY_ALLOSTATIC_MAX: f32 = 0.60;

const COPING_STRATEGY_ORDER: [CopingStrategyId; 15] = [
    CopingStrategyId::StrategicPlanning,
    CopingStrategyId::InstrumentalSupport,
    CopingStrategyId::EmotionalSupport,
    CopingStrategyId::PositiveReframing,
    CopingStrategyId::Denial,
    CopingStrategyId::Acceptance,
    CopingStrategyId::Humor,
    CopingStrategyId::ReligiousCoping,
    CopingStrategyId::Venting,
    CopingStrategyId::ActiveDistraction,
    CopingStrategyId::BehavioralDisengagement,
    CopingStrategyId::SelfBlame,
    CopingStrategyId::SubstanceUse,
    CopingStrategyId::Rumination,
    CopingStrategyId::ProblemSolving,
];

#[inline]
fn coping_strategy_from_index(index: i32) -> Option<CopingStrategyId> {
    if index < 0 || (index as usize) >= COPING_STRATEGY_ORDER.len() {
        return None;
    }
    Some(COPING_STRATEGY_ORDER[index as usize])
}

#[inline]
fn coping_strategy_cooldown_ticks(strategy: CopingStrategyId) -> u32 {
    match strategy {
        CopingStrategyId::Denial => 96,
        CopingStrategyId::SubstanceUse => 120,
        CopingStrategyId::Rumination => 72,
        CopingStrategyId::BehavioralDisengagement => 72,
        _ => 36,
    }
}

#[inline]
fn coping_axis(personality_opt: Option<&Personality>, axis: HexacoAxis) -> f32 {
    personality_opt
        .map(|personality| personality.axis(axis) as f32)
        .unwrap_or(0.5)
        .clamp(0.0, 1.0)
}

#[inline]
fn coping_need(needs_opt: Option<&Needs>, need_type: NeedType, fallback: f32) -> f32 {
    needs_opt
        .map(|needs| needs.get(need_type) as f32)
        .unwrap_or(fallback)
        .clamp(0.0, 1.0)
}

fn coping_utility_scores(
    personality_opt: Option<&Personality>,
    needs_opt: Option<&Needs>,
    stress_norm: f32,
    allostatic_norm: f32,
) -> [f32; 15] {
    let x = coping_axis(personality_opt, HexacoAxis::X);
    let a = coping_axis(personality_opt, HexacoAxis::A);
    let h = coping_axis(personality_opt, HexacoAxis::H);
    let e = coping_axis(personality_opt, HexacoAxis::E);
    let o = coping_axis(personality_opt, HexacoAxis::O);
    let c = coping_axis(personality_opt, HexacoAxis::C);

    let belonging = coping_need(needs_opt, NeedType::Belonging, 0.5);
    let safety = coping_need(needs_opt, NeedType::Safety, 0.5);
    let energy = needs_opt
        .map(|needs| needs.energy as f32)
        .unwrap_or(0.5)
        .clamp(0.0, 1.0);

    let overwhelm = ((stress_norm + allostatic_norm) * 0.5).clamp(0.0, 1.0);
    let social_buffer = ((belonging + safety) * 0.5).clamp(0.0, 1.0);

    let mut scores = [0.0_f32; 15];
    scores[0] = c * 0.80 + (1.0 - overwhelm) * 0.60 + safety * 0.20;
    scores[1] = x * 0.40 + a * 0.40 + social_buffer * 0.30 + overwhelm * 0.20;
    scores[2] = x * 0.50 + a * 0.40 + (1.0 - belonging) * 0.20;
    scores[3] = o * 0.50 + c * 0.20 + (1.0 - overwhelm) * 0.30;
    scores[4] = (1.0 - c) * 0.50 + overwhelm * 0.70;
    scores[5] = c * 0.40 + a * 0.20 + overwhelm * 0.30;
    scores[6] = x * 0.50 + o * 0.30 + (1.0 - overwhelm) * 0.20;
    scores[7] = (1.0 - o) * 0.20 + a * 0.40 + overwhelm * 0.40;
    scores[8] = x * 0.20 + (1.0 - a) * 0.60 + overwhelm * 0.60;
    scores[9] = x * 0.30 + o * 0.30 + overwhelm * 0.40;
    scores[10] = (1.0 - c) * 0.60 + overwhelm * 0.70;
    scores[11] = h * 0.20 + (1.0 - e) * 0.50 + overwhelm * 0.60;
    scores[12] = (1.0 - c) * 0.50 + (1.0 - h) * 0.40 + overwhelm * 0.80;
    scores[13] = (1.0 - x) * 0.30 + (1.0 - e) * 0.40 + overwhelm * 0.70;
    scores[14] = c * 0.70 + o * 0.30 + (1.0 - overwhelm) * 0.40 + energy * 0.20;
    for score in &mut scores {
        *score = (*score).clamp(0.001, 5.0);
    }
    scores
}

/// Apply coping strategy execution effects.
///
/// Scale note: ECS stores stress.level on 0-1, GDScript uses 0-2000.
/// Reserve: ECS 0-1, GDScript 0-100. Allostatic: ECS 0-1, GDScript 0-100.
/// Denial accumulator is stored on raw GDScript scale (0-800).
fn coping_apply_strategy_effects(
    strategy: CopingStrategyId,
    prof: f32,
    coping: &mut Coping,
    stress: &mut Stress,
) {
    match strategy {
        // C05 Denial (Compas 2001): immediate stress relief but hidden debt accumulates.
        // 72 ticks later (or allostatic > 0.8) the debt explodes at 1.5x.
        CopingStrategyId::Denial => {
            let relief = 150.0 * (0.5 + 0.5 * prof);
            stress.set_level_native(NativeStress(stress.level_native().0 - relief));
            // Hidden debt: 80% of current perceived threat redirected to accumulator
            let real_threat = stress.level_native().0 * 0.4;
            coping.denial_accumulator += real_threat * 0.8;
            coping.denial_timer = 72;
            // Permanent allostatic penalty
            stress.set_allostatic_native(NativePercent(stress.allostatic_native().0 + 2.0));
        }
        // C11 Behavioral Disengagement (Seligman 1975): GAS reserve recovery
        // but helplessness accumulates → permanent control_appraisal_cap damage.
        CopingStrategyId::BehavioralDisengagement => {
            stress.set_reserve_native(NativePercent(stress.reserve_native().0 + 10.0));
            coping.helplessness_score = (coping.helplessness_score + 0.10).min(1.0);
            stress.set_allostatic_native(NativePercent(stress.allostatic_native().0 + 3.0));
            if coping.helplessness_score > 0.8 {
                coping.control_appraisal_cap = 0.3;
            }
        }
        // C12/C14 Rumination (Treynor 2003): two sub-types determined by proficiency.
        // Low prof → Reflection (adaptive, reserve gain).
        // High prof → Brooding (maladaptive, allostatic penalty).
        // Sigmoid crossover at prof=0.5 for smooth transition.
        CopingStrategyId::Rumination => {
            let brooding_weight = 1.0 / (1.0 + (-10.0_f32 * (prof - 0.5)).exp());
            let reflection_weight = 1.0 - brooding_weight;
            if reflection_weight > 0.3 {
                stress.set_reserve_native(NativePercent(
                    stress.reserve_native().0 + 5.0 * reflection_weight,
                ));
            }
            if brooding_weight > 0.3 {
                stress.set_allostatic_native(NativePercent(
                    stress.allostatic_native().0 + 4.0 * brooding_weight,
                ));
            }
        }
        // C13 Substance Use (Cooper 1995): strong immediate relief but
        // delayed rebound (12 ticks) + dependency accumulation.
        // Dependency > 0.6 triggers withdrawal stress when substance_recent_timer expires.
        CopingStrategyId::SubstanceUse => {
            let relief = 300.0 * (0.4 + 0.6 * prof);
            stress.set_level_native(NativeStress(stress.level_native().0 - relief));
            coping.rebound_queue.push(CopingRebound {
                delay: 12,
                stress_rebound: 0.15 * prof,
                allostatic_add: 0.08,
            });
            coping.dependency_score = (coping.dependency_score + 0.05).min(1.0);
            coping.substance_recent_timer = 24;
        }
        // Generic adaptive coping: moderate stress relief scaled by proficiency.
        _ => {
            let relief = 80.0 * (0.5 + 0.5 * prof);
            stress.set_level_native(NativeStress(stress.level_native().0 - relief));
        }
    }
    stress.recalculate_state();
}

impl SimSystem for CopingRuntimeSystem {
    fn name(&self) -> &'static str {
        "coping_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let dec = self.tick_interval.min(u32::MAX as u64) as u32;
        let mut query = world.query::<(
            &mut Coping,
            Option<&mut Stress>,
            Option<&Personality>,
            Option<&Needs>,
        )>();
        for (_, (coping, stress_opt, personality_opt, needs_opt)) in &mut query {
            // === Phase 1: Maintenance (runs every tick interval) ===

            // Cooldown countdown
            for cooldown in coping.strategy_cooldowns.values_mut() {
                *cooldown = cooldown.saturating_sub(dec);
            }

            let Some(stress) = stress_opt else {
                continue;
            };

            // C05 Denial timer countdown + explosion (Compas 2001)
            if coping.denial_timer > 0 {
                coping.denial_timer = coping.denial_timer.saturating_sub(dec);
                if coping.denial_timer == 0 && coping.denial_accumulator > 0.0 {
                    let debt = coping.denial_accumulator;
                    stress.set_level_native(NativeStress(stress.level_native().0 + debt * 1.5));
                    coping.denial_accumulator = 0.0;
                    stress.recalculate_state();
                }
            }

            // Allostatic overload triggers early denial explosion
            if coping.denial_accumulator > 0.01 && stress.allostatic_load > 0.8 {
                let debt = coping.denial_accumulator;
                stress.set_level_native(NativeStress(stress.level_native().0 + debt * 1.5));
                coping.denial_accumulator = 0.0;
                coping.denial_timer = 0;
                stress.recalculate_state();
            }

            // C13 Rebound queue processing (Cooper 1995)
            if !coping.rebound_queue.is_empty() {
                let mut remaining = Vec::new();
                for mut rb in coping.rebound_queue.drain(..) {
                    rb.delay -= dec as i32;
                    if rb.delay <= 0 {
                        // Rebound fires: stress += rebound * 300 (native scale)
                        stress.set_level_native(NativeStress(
                            stress.level_native().0 + rb.stress_rebound * 300.0,
                        ));
                        // Allostatic add is already normalized 0-1
                        stress.allostatic_load =
                            (stress.allostatic_load + rb.allostatic_add as f64).clamp(0.0, 1.0);
                    } else {
                        remaining.push(rb);
                    }
                }
                coping.rebound_queue = remaining;
                stress.recalculate_state();
            }

            // Substance withdrawal timer
            if coping.substance_recent_timer > 0 {
                coping.substance_recent_timer = coping.substance_recent_timer.saturating_sub(dec);
            }

            // Withdrawal stress injection (dependency > 0.6 + timer expired)
            if coping.dependency_score > 0.6 && coping.substance_recent_timer == 0 {
                // 90 stress on 0-2000 native scale
                stress.set_level_native(NativeStress(stress.level_native().0 + 90.0));
                coping.substance_recent_timer = 24;
                stress.recalculate_state();
            }

            // === Phase 2: Strategy Selection (Carver COPE Scale) ===

            let stress_norm = (stress.level as f32).clamp(0.0, 1.0);
            let allostatic_norm = (stress.allostatic_load as f32).clamp(0.0, 1.0);

            // Low stress → clear active strategy
            if stress_norm <= COPING_CLEAR_STRESS_MAX
                && allostatic_norm <= COPING_CLEAR_ALLOSTATIC_MAX
            {
                coping.active_strategy = None;
                continue;
            }

            let is_recovery = stress_norm <= COPING_RECOVERY_STRESS_MAX
                && allostatic_norm <= COPING_RECOVERY_ALLOSTATIC_MAX;
            let break_count = coping.break_count.min(i32::MAX as u32) as i32;
            let owned_count = coping.usage_counts.len().min(i32::MAX as usize) as i32;

            // Lazarus sigmoid learn probability with K(n)/S(N) saturation
            let learn_p = body::coping_learn_probability(
                stress.level_native().0,
                stress.allostatic_native().0,
                is_recovery,
                break_count,
                owned_count,
                COPING_COUNT_MAX,
            )
            .clamp(0.0, 1.0);
            if learn_p <= 0.0 {
                continue;
            }

            let learn_roll: f32 = resources.rng.gen_range(0.0..1.0);
            if learn_roll >= learn_p {
                continue;
            }

            // COPE Scale utility scoring with HEXACO personality weights
            let scores =
                coping_utility_scores(personality_opt, needs_opt, stress_norm, allostatic_norm);
            let strategy_roll: f32 = resources.rng.gen_range(0.0..1.0);
            let strategy_idx = body::coping_softmax_index(&scores, strategy_roll);
            let Some(strategy) = coping_strategy_from_index(strategy_idx) else {
                continue;
            };
            if coping.is_on_cooldown(strategy) {
                continue;
            }

            coping.active_strategy = Some(strategy);
            coping
                .usage_counts
                .entry(strategy)
                .and_modify(|count| *count = count.saturating_add(1))
                .or_insert(1);
            coping
                .strategy_cooldowns
                .insert(strategy, coping_strategy_cooldown_ticks(strategy));

            // Update proficiency (increases with use)
            let prof = coping.proficiency.entry(strategy).or_insert(0.0);
            *prof = (*prof + 0.02).clamp(0.0, 1.0);
            let prof_val = *prof;

            // === Phase 3: Execute Strategy Effects ===
            coping_apply_strategy_effects(strategy, prof_val, coping, stress);
        }
    }
}

// ── Personality Maturation ───────────────────────────────────────────

/// OU process constants matching personality_maturation.gd
const MATURATION_THETA: f32 = 0.03;
const MATURATION_SIGMA: f32 = 0.03;

/// Default maturation targets (Ashton & Lee 2016):
/// H: +1.0 SD from 18→60, E/X: mild increase
const MATURATION_TARGETS: [(usize, f32, i32, i32); 3] = [
    (0, 1.0, 18, 60),  // H: +1.0 shift, ages 18-60
    (1, 0.3, 18, 60),  // E: +0.3 shift, ages 18-60
    (2, 0.2, 20, 50),  // X: +0.2 shift, ages 20-50
];

/// Rust runtime system for age-based personality maturation.
///
/// Ports `personality_maturation.gd`: applies an Ornstein-Uhlenbeck (OU) process
/// to each facet z-score, drifting toward age-appropriate targets. Called once
/// per game year per entity. [Ashton & Lee 2016]
#[derive(Debug, Clone)]
pub struct PersonalityMaturationRuntimeSystem {
    priority: u32,
    tick_interval: u64,
    last_year: i32,
}

impl PersonalityMaturationRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
            last_year: -1,
        }
    }
}

/// Convert facet value (0..1) to z-score
#[inline]
fn facet_to_zscore(v: f64) -> f64 {
    (v - 0.5) * 4.0 // maps 0..1 to -2..+2
}

/// Convert z-score back to facet value (0..1)
#[inline]
fn zscore_to_facet(z: f64) -> f64 {
    (z / 4.0 + 0.5).clamp(0.0, 1.0)
}

impl SimSystem for PersonalityMaturationRuntimeSystem {
    fn name(&self) -> &'static str {
        "personality_maturation_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        use sim_core::components::personality::AXIS_COUNT;

        let ticks_per_year = config::TICKS_PER_YEAR as i32;
        if ticks_per_year <= 0 {
            return;
        }
        let current_year = tick as i32 / ticks_per_year;
        if current_year == self.last_year {
            return;
        }
        self.last_year = current_year;

        let mut query = world.query::<(&Age, &mut Personality)>();
        for (_, (age, personality)) in &mut query {
            let age_years = age.years as i32;

            // For each axis, compute target then drift each of its 4 facets
            for axis_idx in 0..AXIS_COUNT {
                // Look up maturation target for this axis
                let target = MATURATION_TARGETS
                    .iter()
                    .find(|(idx, _, _, _)| *idx == axis_idx)
                    .map(|(_, max_shift, start, end)| {
                        body::personality_linear_target(age_years, *max_shift, *start, *end)
                    })
                    .unwrap_or(0.0);

                // Drift each of the 4 facets for this axis
                let facet_base = axis_idx * 4;
                for f in 0..4 {
                    let fi = facet_base + f;
                    let current_z = facet_to_zscore(personality.facets[fi]);
                    // OU: dz = theta * (target - current) + N(0, sigma)
                    let u1 = resources.rng.gen_range(0.0001_f32..1.0_f32);
                    let u2 = resources.rng.gen_range(0.0_f32..1.0_f32);
                    let noise =
                        MATURATION_SIGMA * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
                    let dz = MATURATION_THETA * (target - current_z as f32) + noise;
                    personality.facets[fi] = zscore_to_facet(current_z + dz as f64);
                }
            }
            personality.recalculate_axes();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TraitRuntimeSystem — v3 Binary Trait Evaluation
// ═══════════════════════════════════════════════════════════════════════

/// Source type for a trait condition evaluation.
#[derive(Debug, Clone)]
pub enum TraitConditionSource {
    /// HEXACO axis threshold check
    HexacoAxis(HexacoAxis),
    /// HEXACO facet threshold check
    HexacoFacet(HexacoFacet),
    /// Value threshold check
    Value(ValueType),
}

/// Direction of threshold comparison.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TraitDirection {
    /// Value must be >= threshold
    High,
    /// Value must be <= threshold
    Low,
}

/// A single condition that must be met for a trait to activate.
#[derive(Debug, Clone)]
pub struct TraitCondition {
    pub source: TraitConditionSource,
    pub direction: TraitDirection,
    pub threshold: f64,
}

/// Definition of a binary trait with activation conditions.
#[derive(Debug, Clone)]
pub struct TraitDefinition {
    pub id: String,
    pub conditions: Vec<TraitCondition>,
    /// Traits that are incompatible (mutually exclusive).
    pub incompatibles: Vec<String>,
}

/// Data-driven binary trait evaluator.
///
/// Evaluates personality/value conditions to determine which binary traits
/// should be active on each entity. Supports HEXACO axis, HEXACO facet,
/// and value-based conditions with high/low thresholds.
///
/// Academic basis: v3 binary trait evaluation from trait_system.gd
/// (threshold-gated personality → trait mapping).
#[derive(Debug, Clone)]
pub struct TraitRuntimeSystem {
    priority: u32,
    tick_interval: u64,
    definitions: Vec<TraitDefinition>,
}

impl TraitRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
            definitions: Vec::new(),
        }
    }

    pub fn with_definitions(mut self, defs: Vec<TraitDefinition>) -> Self {
        self.definitions = defs;
        self
    }

    /// Evaluate all conditions for a single trait definition.
    fn evaluate_conditions(
        def: &TraitDefinition,
        personality: &Personality,
        values: &Values,
    ) -> bool {
        if def.conditions.is_empty() {
            return false;
        }
        def.conditions.iter().all(|cond| {
            let val = match &cond.source {
                TraitConditionSource::HexacoAxis(axis) => personality.axes[*axis as usize],
                TraitConditionSource::HexacoFacet(facet) => personality.facets[*facet as usize],
                TraitConditionSource::Value(vtype) => values.values[*vtype as usize],
            };
            match cond.direction {
                TraitDirection::High => val >= cond.threshold,
                TraitDirection::Low => val <= cond.threshold,
            }
        })
    }
}

impl SimSystem for TraitRuntimeSystem {
    fn name(&self) -> &'static str {
        "trait_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        if self.definitions.is_empty() {
            return;
        }

        // Pass 1: Evaluate traits for all entities
        struct TraitUpdate {
            entity: Entity,
            new_active: Vec<String>,
        }

        let mut updates: Vec<TraitUpdate> = Vec::new();

        {
            let mut query = world.query::<(&Personality, &Values, &Traits)>();
            for (entity, (personality, values, current_traits)) in &mut query {
                let mut new_active: Vec<String> = Vec::new();
                let mut activated_set: HashSet<String> = HashSet::new();

                // Evaluate each definition
                for def in &self.definitions {
                    if Self::evaluate_conditions(def, personality, values) {
                        // Check incompatibles — first activated wins
                        let blocked = def
                            .incompatibles
                            .iter()
                            .any(|inc| activated_set.contains(inc));
                        if !blocked {
                            activated_set.insert(def.id.clone());
                            new_active.push(def.id.clone());
                        }
                    }
                }

                // Preserve event-granted traits (those not in any definition)
                let defined_ids: HashSet<&str> =
                    self.definitions.iter().map(|d| d.id.as_str()).collect();
                for existing in &current_traits.active {
                    if !defined_ids.contains(existing.as_str())
                        && !activated_set.contains(existing)
                    {
                        new_active.push(existing.clone());
                    }
                }

                updates.push(TraitUpdate { entity, new_active });
            }
        }

        // Pass 2: Apply trait updates
        for update in updates {
            if let Ok(mut traits) = world.get::<&mut Traits>(update.entity) {
                traits.active = update.new_active;
            }
        }
    }
}
