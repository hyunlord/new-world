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

use crate::body;


/// Rust runtime system for age/environment-adjusted intelligence updates.
///
/// This performs active writes on `Intelligence.values`, `Intelligence.g_factor`,
/// `Intelligence.ace_penalty`, and `Intelligence.nutrition_penalty`.
#[derive(Debug, Clone)]
pub struct IntelligenceRuntimeSystem {
    priority: u32,
    tick_interval: u64,
    potential_baselines: HashMap<Entity, [f32; 8]>,
}

impl IntelligenceRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
            potential_baselines: HashMap::new(),
        }
    }

    pub(crate) fn baseline_count(&self) -> usize {
        self.potential_baselines.len()
    }
}

const INTEL_CURVE_FLUID: [(f32, f32); 8] = [
    (0.0, 0.20),
    (5.0, 0.50),
    (15.0, 0.85),
    (22.0, 1.00),
    (35.0, 1.00),
    (55.0, 0.85),
    (75.0, 0.60),
    (100.0, 0.50),
];
const INTEL_CURVE_CRYSTALLIZED: [(f32, f32); 8] = [
    (0.0, 0.15),
    (5.0, 0.30),
    (15.0, 0.55),
    (25.0, 0.75),
    (50.0, 0.95),
    (65.0, 1.00),
    (80.0, 0.85),
    (100.0, 0.75),
];
const INTEL_CURVE_PHYSICAL: [(f32, f32); 9] = [
    (0.0, 0.10),
    (5.0, 0.35),
    (12.0, 0.65),
    (20.0, 0.90),
    (28.0, 1.00),
    (40.0, 0.85),
    (60.0, 0.60),
    (80.0, 0.45),
    (100.0, 0.35),
];

#[inline]
fn interpolate_curve(curve: &[(f32, f32)], age_years: f32) -> f32 {
    if curve.is_empty() {
        return 1.0;
    }
    if age_years <= curve[0].0 {
        return curve[0].1;
    }
    let last_idx = curve.len() - 1;
    if age_years >= curve[last_idx].0 {
        return curve[last_idx].1;
    }
    for idx in 1..curve.len() {
        let prev = curve[idx - 1];
        let next = curve[idx];
        if age_years <= next.0 {
            let span = (next.0 - prev.0).max(0.000_001);
            let t = ((age_years - prev.0) / span).clamp(0.0, 1.0);
            return prev.1 + (next.1 - prev.1) * t;
        }
    }
    curve[last_idx].1
}

#[inline]
fn is_fluid_intelligence(kind: IntelligenceType) -> bool {
    matches!(kind, IntelligenceType::Logical | IntelligenceType::Spatial)
}

#[inline]
fn intelligence_age_modifier(kind: IntelligenceType, age_years: f32) -> f32 {
    if is_fluid_intelligence(kind) {
        return interpolate_curve(&INTEL_CURVE_FLUID, age_years);
    }
    if matches!(kind, IntelligenceType::Kinesthetic) {
        return interpolate_curve(&INTEL_CURVE_PHYSICAL, age_years);
    }
    interpolate_curve(&INTEL_CURVE_CRYSTALLIZED, age_years)
}

impl SimSystem for IntelligenceRuntimeSystem {
    fn name(&self) -> &'static str {
        "intelligence_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        // Clean up baselines for despawned entities to prevent memory leak
        self.potential_baselines.retain(|&entity, _| world.contains(entity));

        let mut query = world.query::<(
            &mut Intelligence,
            Option<&Age>,
            Option<&Needs>,
            Option<&Skills>,
            Option<&Memory>,
            Option<&Identity>,
            Option<&Personality>,
        )>();
        for (entity, (intelligence, age_opt, needs_opt, skills_opt, memory_opt, identity_opt, personality_opt)) in &mut query {
            let baseline = self.potential_baselines.entry(entity).or_insert_with(|| {
                intelligence
                    .values
                    .map(|value| (value as f32).clamp(0.02, 0.98))
            });
            let age_ticks = age_opt.map(|age| age.ticks).unwrap_or(0);
            let age_years = age_opt
                .map(|age| {
                    if age.years > 0.0 {
                        age.years as f32
                    } else {
                        age.ticks as f32 / config::TICKS_PER_YEAR as f32
                    }
                })
                .unwrap_or(0.0)
                .max(0.0);

            if age_ticks <= config::INTEL_NUTRITION_CRIT_AGE_TICKS as u64
                && intelligence.nutrition_penalty < config::INTEL_NUTRITION_MAX_PENALTY
            {
                let hunger = needs_opt
                    .map(|needs| needs.get(NeedType::Hunger) as f32)
                    .unwrap_or(1.0)
                    .clamp(0.0, 1.0);
                if hunger < config::INTEL_NUTRITION_HUNGER_THRESHOLD as f32 {
                    let severity = 1.0
                        - hunger / config::INTEL_NUTRITION_HUNGER_THRESHOLD as f32;
                    let delta = config::INTEL_NUTRITION_PENALTY_PER_TICK as f32 * severity;
                    intelligence.nutrition_penalty = (intelligence.nutrition_penalty as f32 + delta)
                        .min(config::INTEL_NUTRITION_MAX_PENALTY as f32)
                        as f64;
                }
            }

            if age_years >= config::INTEL_ACE_CRIT_AGE_YEARS as f32
                && intelligence.ace_penalty <= 0.0
            {
                let birth_tick = identity_opt.map(|identity| identity.birth_tick).unwrap_or(0);
                let cutoff = birth_tick + (config::INTEL_ACE_CRIT_AGE_YEARS as f32
                    * config::TICKS_PER_YEAR as f32) as u64;
                let scar_count = memory_opt
                    .map(|memory| {
                        memory
                            .trauma_scars
                            .iter()
                            .filter(|scar| scar.acquired_tick < cutoff)
                            .count() as u32
                    })
                    .unwrap_or(0);
                if scar_count >= config::INTEL_ACE_SCARS_THRESHOLD_MAJOR {
                    intelligence.ace_penalty = config::INTEL_ACE_PENALTY_MAJOR;
                } else if scar_count >= config::INTEL_ACE_SCARS_THRESHOLD_MINOR {
                    intelligence.ace_penalty = config::INTEL_ACE_PENALTY_MINOR;
                }
            }

            let active_skill_count = skills_opt
                .map(|skills| {
                    skills
                        .entries
                        .values()
                        .filter(|entry| {
                            u32::from(entry.level) >= config::INTEL_ACTIVITY_SKILL_THRESHOLD
                        })
                        .count() as i32
                })
                .unwrap_or(0);
            let activity_mod = body::cognition_activity_modifier(
                active_skill_count,
                config::INTEL_ACTIVITY_BUFFER as f32,
                config::INTEL_INACTIVITY_ACCEL as f32,
            );
            let ace_fluid_mult = body::cognition_ace_fluid_decline_mult(
                intelligence.ace_penalty as f32,
                config::INTEL_ACE_PENALTY_MINOR as f32,
                config::INTEL_ACE_FLUID_DECLINE_MULT as f32,
            );
            let env_penalty =
                (intelligence.nutrition_penalty + intelligence.ace_penalty) as f32;

            let intel_order = [
                IntelligenceType::Linguistic,
                IntelligenceType::Logical,
                IntelligenceType::Spatial,
                IntelligenceType::Musical,
                IntelligenceType::Kinesthetic,
                IntelligenceType::Interpersonal,
                IntelligenceType::Intrapersonal,
                IntelligenceType::Naturalistic,
            ];
            for (idx, kind) in intel_order.iter().enumerate() {
                let potential = baseline[idx];
                let base_mod = intelligence_age_modifier(*kind, age_years);
                let effective = body::intelligence_effective_value(
                    potential,
                    base_mod,
                    age_years,
                    is_fluid_intelligence(*kind),
                    activity_mod,
                    ace_fluid_mult,
                    env_penalty,
                    0.02,
                    0.98,
                );
                intelligence.values[idx] = effective as f64;
            }

            let openness = personality_opt
                .map(|personality| personality.axis(HexacoAxis::O) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            intelligence.g_factor = body::intelligence_g_value(
                false,
                0.0,
                0.0,
                config::INTEL_HERITABILITY_G as f32,
                config::INTEL_G_MEAN as f32,
                openness,
                config::INTEL_OPENNESS_G_WEIGHT as f32,
                0.0,
            )
            .clamp(0.0, 1.0) as f64;
        }
    }
}

/// Rust runtime system for memory decay, eviction, compression, and promotion.
///
/// This ports the core working-memory management loop from `memory_system.gd`
/// into Rust ECS state writes.
#[derive(Debug, Clone)]
pub struct MemoryRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl MemoryRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

const MEMORY_FORGET_THRESHOLD: f32 = 0.01;
const MEMORY_SUMMARY_SCALE: f32 = 0.70;

#[inline]
fn memory_is_permanent_event(event_type: &str) -> bool {
    matches!(
        event_type,
        "birth"
            | "marriage"
            | "child_born"
            | "partner_died"
            | "war"
            | "migration"
            | "promotion"
            | "betrayal"
            | "trauma"
            | "achievement"
            | "proposal"
    )
}

#[inline]
fn memory_decay_rate_from_encoding(intensity: f64) -> f32 {
    config::memory_decay_rate(intensity.clamp(0.0, 1.0)) as f32
}

impl SimSystem for MemoryRuntimeSystem {
    fn name(&self) -> &'static str {
        "memory_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, tick: u64) {
        let dt_years = self.tick_interval as f32 / config::TICKS_PER_YEAR as f32;
        let cutoff_tick = tick.saturating_sub(config::MEMORY_COMPRESS_INTERVAL_TICKS);
        let mut query = world.query::<&mut Memory>();
        for (_, memory) in &mut query {
            if !memory.short_term.is_empty() {
                let entries: Vec<MemoryEntry> = memory.short_term.drain(..).collect();
                let mut intensities: Vec<f32> = Vec::with_capacity(entries.len());
                let mut rates: Vec<f32> = Vec::with_capacity(entries.len());
                for entry in &entries {
                    intensities.push(entry.current_intensity as f32);
                    rates.push(memory_decay_rate_from_encoding(entry.intensity));
                }
                let decayed = body::memory_decay_batch(&intensities, &rates, dt_years);
                let mut remaining: Vec<MemoryEntry> = Vec::with_capacity(entries.len());
                for (idx, mut entry) in entries.into_iter().enumerate() {
                    let next_intensity = decayed
                        .get(idx)
                        .copied()
                        .unwrap_or(entry.current_intensity as f32)
                        .clamp(0.0, 1.0);
                    if next_intensity < MEMORY_FORGET_THRESHOLD {
                        continue;
                    }
                    entry.current_intensity = next_intensity as f64;
                    remaining.push(entry);
                }
                memory.short_term = remaining.into_iter().collect();
            }

            if memory.short_term.len() > config::MEMORY_WORKING_MAX {
                let mut entries: Vec<MemoryEntry> = memory.short_term.drain(..).collect();
                entries.sort_by(|a, b| {
                    a.current_intensity
                        .partial_cmp(&b.current_intensity)
                        .unwrap_or(Ordering::Equal)
                });
                let excess = entries.len() - config::MEMORY_WORKING_MAX;
                entries.drain(0..excess);
                memory.short_term = entries.into_iter().collect();
            }

            let mut old_entries: Vec<MemoryEntry> = Vec::new();
            let mut recent_entries: Vec<MemoryEntry> = Vec::new();
            for entry in memory.short_term.drain(..) {
                if entry.tick < cutoff_tick {
                    old_entries.push(entry);
                } else {
                    recent_entries.push(entry);
                }
            }

            let mut rebuilt: Vec<MemoryEntry> = Vec::with_capacity(old_entries.len() + recent_entries.len());
            if old_entries.len() >= config::MEMORY_COMPRESS_MIN_GROUP {
                let mut groups: HashMap<(String, Option<u64>), Vec<MemoryEntry>> = HashMap::new();
                for entry in old_entries {
                    let key = (entry.event_type.clone(), entry.target_id);
                    groups.entry(key).or_default().push(entry);
                }
                let mut keys: Vec<(String, Option<u64>)> = groups.keys().cloned().collect();
                keys.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
                for key in keys {
                    let Some(group) = groups.remove(&key) else {
                        continue;
                    };
                    if group.len() < config::MEMORY_COMPRESS_MIN_GROUP {
                        rebuilt.extend(group.into_iter());
                        continue;
                    }
                    let mut max_intensity = 0.0_f32;
                    let mut oldest_tick = u64::MAX;
                    for entry in &group {
                        max_intensity = max_intensity.max(entry.current_intensity as f32);
                        oldest_tick = oldest_tick.min(entry.tick);
                    }
                    let event_type = group[0].event_type.clone();
                    let target_id = group[0].target_id;
                    let summary_intensity = body::memory_summary_intensity(
                        max_intensity,
                        MEMORY_SUMMARY_SCALE,
                    )
                    .clamp(0.0, 1.0);
                    rebuilt.push(MemoryEntry {
                        event_type: format!("{event_type}_summary"),
                        target_id,
                        tick: oldest_tick,
                        intensity: summary_intensity as f64,
                        current_intensity: summary_intensity as f64,
                        is_permanent: false,
                    });
                }
                memory.last_compression_tick = tick;
            } else {
                rebuilt.extend(old_entries.into_iter());
            }
            rebuilt.extend(recent_entries.into_iter());
            memory.short_term = rebuilt.into_iter().collect();

            let mut permanent_keys: HashSet<(String, u64)> = memory
                .permanent
                .iter()
                .map(|entry| (entry.event_type.clone(), entry.tick))
                .collect();
            let mut promoted: Vec<MemoryEntry> = Vec::new();
            for entry in memory.short_term.iter_mut() {
                if entry.current_intensity < config::MEMORY_PERMANENT_THRESHOLD {
                    continue;
                }
                if !memory_is_permanent_event(entry.event_type.as_str()) {
                    continue;
                }
                let key = (entry.event_type.clone(), entry.tick);
                if !permanent_keys.insert(key) {
                    continue;
                }
                entry.is_permanent = true;
                let mut permanent_entry = entry.clone();
                permanent_entry.is_permanent = true;
                promoted.push(permanent_entry);
            }
            memory.permanent.extend(promoted.into_iter());
        }
    }
}

const BEHAVIOR_HYSTERESIS_THRESHOLD: f32 = 0.85;
const BEHAVIOR_WANDER_OFFSETS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

#[inline]
fn behavior_urgency(deficit: f32) -> f32 {
    deficit.clamp(0.0, 1.0).powi(2)
}

#[inline]
fn behavior_score_add(scores: &mut HashMap<ActionType, f32>, action: ActionType, delta: f32) {
    if delta <= 0.0 {
        return;
    }
    let entry = scores.entry(action).or_insert(0.0);
    *entry += delta;
}

#[inline]
fn behavior_score_mul(scores: &mut HashMap<ActionType, f32>, action: ActionType, multiplier: f32) {
    if let Some(score) = scores.get_mut(&action) {
        *score = (*score * multiplier.max(0.0)).max(0.0);
    }
}

#[inline]
fn behavior_base_timer(action: ActionType) -> i32 {
    match action {
        ActionType::Wander => 5,
        ActionType::Forage | ActionType::GatherWood | ActionType::GatherStone => 20,
        ActionType::DeliverToStockpile => 30,
        ActionType::Build => 25,
        ActionType::TakeFromStockpile => 15,
        ActionType::Rest => 10,
        ActionType::Socialize => 8,
        ActionType::VisitPartner => 15,
        ActionType::Drink => 10,
        ActionType::SitByFire => 20,
        ActionType::SeekShelter => 15,
        _ => 10,
    }
}

#[inline]
fn behavior_timer_with_stress(
    base_timer: i32,
    stress_level: f32,
    allostatic_load: f32,
    stress_exempt: bool,
) -> i32 {
    if base_timer <= 0 {
        return 1;
    }
    if stress_exempt {
        return base_timer;
    }
    let level = stress_level.clamp(0.0, 1.0);
    let mut mult = if level < 0.25 {
        0.90
    } else if level < 0.60 {
        1.0
    } else if level < 0.85 {
        1.0 + ((level - 0.60) / 0.25) * 0.30
    } else {
        1.30 + ((level - 0.85) / 0.15) * 0.30
    };
    if allostatic_load > 0.5 {
        mult *= 1.0 + (allostatic_load - 0.5) * 0.2;
    }
    (base_timer as f32 * mult).round().clamp(1.0, 120.0) as i32
}

fn behavior_pick_wander_target(
    position: &Position,
    resources: &SimResources,
    tick: u64,
    entity_raw: u64,
) -> (i32, i32) {
    let mut idx = (entity_raw
        .wrapping_mul(31)
        .wrapping_add(tick.wrapping_mul(17))
        % BEHAVIOR_WANDER_OFFSETS.len() as u64) as usize;
    for _ in 0..BEHAVIOR_WANDER_OFFSETS.len() {
        let (dx, dy) = BEHAVIOR_WANDER_OFFSETS[idx];
        let nx = position.x + dx;
        let ny = position.y + dy;
        if resources.map.in_bounds(nx, ny) {
            let tile = resources.map.get(nx as u32, ny as u32);
            if tile.passable {
                return (nx, ny);
            }
        }
        idx = (idx + 1) % BEHAVIOR_WANDER_OFFSETS.len();
    }
    (position.x, position.y)
}

fn behavior_select_action(
    age_stage: GrowthStage,
    needs: &Needs,
    stress_opt: Option<&Stress>,
    emotion_opt: Option<&Emotion>,
    behavior: &Behavior,
) -> ActionType {
    let hunger = needs.get(NeedType::Hunger) as f32;
    let thirst = needs.get(NeedType::Thirst) as f32;
    let warmth = needs.get(NeedType::Warmth) as f32;
    let safety = needs.get(NeedType::Safety) as f32;
    let social = needs.get(NeedType::Belonging) as f32;
    let energy = needs.energy as f32;

    let hunger_deficit = behavior_urgency(1.0 - hunger);
    let energy_deficit = behavior_urgency(1.0 - energy);
    let social_deficit = behavior_urgency(1.0 - social);

    let mut scores: HashMap<ActionType, f32> = HashMap::new();
    match age_stage {
        GrowthStage::Infant | GrowthStage::Toddler => {
            behavior_score_add(&mut scores, ActionType::Wander, 0.30);
            behavior_score_add(&mut scores, ActionType::Rest, energy_deficit * 1.20);
            behavior_score_add(&mut scores, ActionType::Socialize, social_deficit * 0.80);
        }
        GrowthStage::Child => {
            behavior_score_add(&mut scores, ActionType::Wander, 0.30);
            behavior_score_add(&mut scores, ActionType::Rest, energy_deficit * 1.20);
            behavior_score_add(&mut scores, ActionType::Socialize, social_deficit * 0.80);
            behavior_score_add(&mut scores, ActionType::Forage, hunger_deficit * 0.60);
        }
        GrowthStage::Teen | GrowthStage::Adult | GrowthStage::Elder => {
            behavior_score_add(&mut scores, ActionType::Wander, 0.20);
            behavior_score_add(&mut scores, ActionType::Forage, hunger_deficit * 1.50);
            behavior_score_add(&mut scores, ActionType::Rest, energy_deficit * 1.20);
            behavior_score_add(&mut scores, ActionType::Socialize, social_deficit * 0.80);
        }
    }

    if hunger < 0.30 {
        scores.insert(ActionType::Forage, 1.0);
    }

    if thirst < config::THIRST_LOW as f32 {
        behavior_score_add(&mut scores, ActionType::Drink, behavior_urgency(1.0 - thirst));
    }
    if warmth < config::WARMTH_LOW as f32 {
        behavior_score_add(
            &mut scores,
            ActionType::SitByFire,
            behavior_urgency(1.0 - warmth) * 0.90,
        );
    }
    if warmth < config::WARMTH_LOW as f32 || safety < config::SAFETY_LOW as f32 {
        let shelter_score =
            behavior_urgency(1.0 - warmth) * 0.60 + behavior_urgency(1.0 - safety) * 0.40;
        behavior_score_add(&mut scores, ActionType::SeekShelter, shelter_score);
    }

    match behavior.job.as_str() {
        "gatherer" => behavior_score_mul(&mut scores, ActionType::Forage, 1.50),
        "lumberjack" => behavior_score_add(&mut scores, ActionType::GatherWood, 0.45),
        "miner" => behavior_score_add(&mut scores, ActionType::GatherStone, 0.40),
        "builder" if matches!(age_stage, GrowthStage::Adult) => {
            behavior_score_add(&mut scores, ActionType::Build, 0.45);
        }
        _ => {}
    }
    if matches!(age_stage, GrowthStage::Teen) {
        scores.remove(&ActionType::GatherStone);
        behavior_score_mul(&mut scores, ActionType::GatherWood, 0.70);
    }

    let stress_level = stress_opt
        .map(|stress| stress.level as f32)
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    if stress_level > 0.65 {
        behavior_score_mul(&mut scores, ActionType::Rest, 1.0 + stress_level * 0.50);
        behavior_score_add(
            &mut scores,
            ActionType::SeekShelter,
            (stress_level - 0.60).max(0.0) * 0.80,
        );
    }

    if let Some(emotion) = emotion_opt {
        let fear = emotion.get(EmotionType::Fear) as f32;
        let sadness = emotion.get(EmotionType::Sadness) as f32;
        let joy = emotion.get(EmotionType::Joy) as f32;
        if fear > 0.40 {
            behavior_score_add(&mut scores, ActionType::SeekShelter, fear * 1.20);
        }
        if sadness > 0.40 {
            behavior_score_add(&mut scores, ActionType::Rest, sadness * 0.60);
        }
        if joy > 0.60 {
            behavior_score_add(&mut scores, ActionType::Socialize, joy * 0.50);
        }
    }

    if scores.is_empty() {
        return ActionType::Wander;
    }

    const BEHAVIOR_ACTION_ORDER: [ActionType; 13] = [
        ActionType::SeekShelter,
        ActionType::Drink,
        ActionType::SitByFire,
        ActionType::Forage,
        ActionType::GatherWood,
        ActionType::GatherStone,
        ActionType::Build,
        ActionType::Socialize,
        ActionType::Rest,
        ActionType::VisitPartner,
        ActionType::TakeFromStockpile,
        ActionType::DeliverToStockpile,
        ActionType::Wander,
    ];
    let mut best_action = ActionType::Wander;
    let mut best_score = -1.0_f32;
    for action in BEHAVIOR_ACTION_ORDER {
        let score = scores.get(&action).copied().unwrap_or(0.0);
        if score > best_score + 1e-6 {
            best_score = score;
            best_action = action;
        }
    }

    if behavior.current_action != ActionType::Idle {
        if let Some(current_score) = scores.get(&behavior.current_action).copied() {
            if current_score >= best_score * BEHAVIOR_HYSTERESIS_THRESHOLD {
                return behavior.current_action;
            }
        }
    }
    best_action
}

fn behavior_assign_action(
    behavior: &mut Behavior,
    position: &Position,
    resources: &SimResources,
    tick: u64,
    entity_raw: u64,
    action: ActionType,
    stress_level: f32,
    allostatic_load: f32,
) {
    let mut target_x = position.x;
    let mut target_y = position.y;
    if action == ActionType::Wander {
        let target = behavior_pick_wander_target(position, resources, tick, entity_raw);
        target_x = target.0;
        target_y = target.1;
    }

    let base_timer = behavior_base_timer(action);
    let stress_exempt = matches!(
        action,
        ActionType::Drink | ActionType::SeekShelter | ActionType::Flee
    );
    let timer = behavior_timer_with_stress(base_timer, stress_level, allostatic_load, stress_exempt);

    behavior.current_action = action;
    behavior.action_target_entity = None;
    behavior.action_target_x = Some(target_x);
    behavior.action_target_y = Some(target_y);
    behavior.action_progress = 0.0;
    behavior.action_duration = timer;
    behavior.action_timer = timer;
}

/// Rust runtime system for utility-style behavior selection.
///
/// This performs active writes on `Behavior.current_action`,
/// `Behavior.action_target_*`, `Behavior.action_timer`, and `Behavior.action_duration`,
/// then emits action-selection events through the runtime event bus.
#[derive(Debug, Clone)]
pub struct BehaviorRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl BehaviorRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for BehaviorRuntimeSystem {
    fn name(&self) -> &'static str {
        "behavior_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        let mut query = world.query::<(
            &Age,
            &Needs,
            Option<&Stress>,
            Option<&Emotion>,
            &Position,
            &mut Behavior,
        )>();
        for (entity, (age, needs, stress_opt, emotion_opt, position, behavior)) in &mut query {
            if !age.alive {
                continue;
            }
            if behavior.current_action == ActionType::Migrate {
                continue;
            }
            if behavior.action_timer > 0 {
                continue;
            }

            let next_action = behavior_select_action(age.stage, needs, stress_opt, emotion_opt, behavior);
            let previous_action = behavior.current_action;
            let stress_level = stress_opt
                .map(|stress| stress.level as f32)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);
            let allostatic_load = stress_opt
                .map(|stress| stress.allostatic_load as f32)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);
            behavior_assign_action(
                behavior,
                position,
                resources,
                tick,
                entity.id() as u64,
                next_action,
                stress_level,
                allostatic_load,
            );

            let changed = previous_action != next_action;
            let entity_id = EntityId(entity.id() as u64);
            let event_type = if changed {
                format!("action_changed:{}->{}", previous_action, next_action)
            } else {
                format!("action_chosen:{}", next_action)
            };
            resources.event_bus.emit(sim_engine::GameEvent::SocialEventOccurred {
                event_type,
                participants: vec![entity_id],
            });
        }
    }
}
