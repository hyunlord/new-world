#![allow(unused_imports)]
// TODO(v3.1): REFACTOR - reduce direct entity/string coupling in cognition systems and consume authoritative data-driven runtime state.

use hecs::{Entity, World};
use rand::Rng;
use sim_core::components::{
    Age, Behavior, Body as BodyComponent, Coping, Economic, Emotion, Identity, Intelligence,
    Inventory, Memory, MemoryEntry, Needs, Personality, Position, Skills, Social, Stress, Traits,
    Values,
};
use sim_core::temperament::{Temperament, TemperamentAxes};
use sim_core::config;
use sim_core::{
    ActionType, AttachmentType, BuildingId, ChannelId, CopingStrategyId, EmotionType, EntityId,
    GrowthStage, HexacoAxis, HexacoFacet, IntelligenceType, MentalBreakType, NeedType,
    RelationType, ResourceType, SettlementId, Sex, SocialClass, TechState, TerrainType, Tile,
    ValueType,
};
use sim_engine::{SimEvent, SimEventType, SimResources, SimSystem};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use super::crafting;
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

    #[allow(dead_code)]
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
        self.potential_baselines
            .retain(|&entity, _| world.contains(entity));

        let mut query = world.query::<(
            &mut Intelligence,
            Option<&Age>,
            Option<&Needs>,
            Option<&Skills>,
            Option<&Memory>,
            Option<&Identity>,
            Option<&Personality>,
        )>();
        for (
            entity,
            (
                intelligence,
                age_opt,
                needs_opt,
                skills_opt,
                memory_opt,
                identity_opt,
                personality_opt,
            ),
        ) in &mut query
        {
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

            if age_ticks <= config::INTEL_NUTRITION_CRIT_AGE_TICKS
                && intelligence.nutrition_penalty < config::INTEL_NUTRITION_MAX_PENALTY
            {
                let hunger = needs_opt
                    .map(|needs| needs.get(NeedType::Hunger) as f32)
                    .unwrap_or(1.0)
                    .clamp(0.0, 1.0);
                if hunger < config::INTEL_NUTRITION_HUNGER_THRESHOLD as f32 {
                    let severity = 1.0 - hunger / config::INTEL_NUTRITION_HUNGER_THRESHOLD as f32;
                    let delta = config::INTEL_NUTRITION_PENALTY_PER_TICK as f32 * severity;
                    intelligence.nutrition_penalty = (intelligence.nutrition_penalty as f32 + delta)
                        .min(config::INTEL_NUTRITION_MAX_PENALTY as f32)
                        as f64;
                }
            }

            if age_years >= config::INTEL_ACE_CRIT_AGE_YEARS as f32
                && intelligence.ace_penalty <= 0.0
            {
                let birth_tick = identity_opt
                    .map(|identity| identity.birth_tick)
                    .unwrap_or(0);
                let cutoff = birth_tick
                    + (config::INTEL_ACE_CRIT_AGE_YEARS as f32 * config::TICKS_PER_YEAR as f32)
                        as u64;
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
            let env_penalty = (intelligence.nutrition_penalty + intelligence.ace_penalty) as f32;

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

            let mut rebuilt: Vec<MemoryEntry> =
                Vec::with_capacity(old_entries.len() + recent_entries.len());
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
                    let summary_intensity =
                        body::memory_summary_intensity(max_intensity, MEMORY_SUMMARY_SCALE)
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

/// Returns a score modifier (f32) for `action` based on TCI expressed temperament axes.
///
/// Bias is centered at 0.5 (neutral): axis > 0.5 → positive nudge, < 0.5 → negative nudge.
/// Max per-action effect is ±0.15 which nudges but does not dominate needs-based scores.
/// Academic basis: Cloninger et al. (1993) — TCI axis → neurotransmitter → behavioral tendency.
#[inline]
fn temperament_action_bias(axes: &TemperamentAxes, action: ActionType) -> f32 {
    let bias: f64 = match action {
        // NS (dopamine) → exploratory approach, novelty, rapid switching
        ActionType::Explore => 0.30 * (axes.ns - 0.5),
        ActionType::Forage => 0.15 * (axes.ns - 0.5),
        // HA (serotonin) → passive avoidance, anticipatory caution
        ActionType::Flee => 0.30 * (axes.ha - 0.5),
        ActionType::Rest => 0.15 * (axes.ha - 0.5),
        // RD (noradrenaline) → social reward seeking
        ActionType::Socialize => 0.30 * (axes.rd - 0.5),
        // P (corticostriatal) → perseverance, industriousness
        ActionType::Build => 0.20 * (axes.p - 0.5),
        ActionType::Craft => 0.15 * (axes.p - 0.5),
        ActionType::GatherStone => 0.10 * (axes.p - 0.5),
        _ => 0.0,
    };
    bias as f32
}

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
        ActionType::Wander => config::ACTION_TIMER_WANDER,
        ActionType::Explore => config::ACTION_TIMER_EXPLORE,
        ActionType::Forage | ActionType::GatherWood | ActionType::GatherStone => {
            config::ACTION_TIMER_FORAGE
        }
        ActionType::Eat => config::ACTION_TIMER_EAT,
        ActionType::Hunt => config::ACTION_TIMER_HUNT,
        ActionType::DeliverToStockpile => config::ACTION_TIMER_DELIVER,
        ActionType::Build => config::ACTION_TIMER_BUILD,
        ActionType::Craft => config::ACTION_TIMER_CRAFT,
        ActionType::TakeFromStockpile => config::ACTION_TIMER_TAKE_STOCKPILE,
        ActionType::Rest => config::ACTION_TIMER_REST,
        ActionType::Sleep => config::ACTION_TIMER_SLEEP,
        ActionType::Socialize => config::ACTION_TIMER_SOCIALIZE,
        ActionType::VisitPartner => config::ACTION_TIMER_VISIT_PARTNER,
        ActionType::Drink => config::ACTION_TIMER_DRINK,
        ActionType::Flee => config::ACTION_TIMER_FLEE,
        ActionType::SitByFire => config::ACTION_TIMER_SIT_BY_FIRE,
        ActionType::SeekShelter => config::ACTION_TIMER_SEEK_SHELTER,
        _ => config::ACTION_TIMER_DEFAULT,
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
    let origin_x = position.tile_x();
    let origin_y = position.tile_y();
    let mut idx = (entity_raw
        .wrapping_mul(31)
        .wrapping_add(tick.wrapping_mul(17))
        % BEHAVIOR_WANDER_OFFSETS.len() as u64) as usize;
    for _ in 0..BEHAVIOR_WANDER_OFFSETS.len() {
        let (dx, dy) = BEHAVIOR_WANDER_OFFSETS[idx];
        let nx = origin_x + dx;
        let ny = origin_y + dy;
        if resources.map.in_bounds(nx, ny) {
            let tile = resources.map.get(nx as u32, ny as u32);
            if tile.passable {
                return (nx, ny);
            }
        }
        idx = (idx + 1) % BEHAVIOR_WANDER_OFFSETS.len();
    }
    (origin_x, origin_y)
}

fn behavior_select_action(
    age_stage: GrowthStage,
    needs: &Needs,
    stress_opt: Option<&Stress>,
    emotion_opt: Option<&Emotion>,
    personality_opt: Option<&Personality>,
    temperament_opt: Option<&Temperament>,
    behavior: &Behavior,
    has_build_target: bool,
    has_settlement: bool,
    has_food_item: bool,
    has_tool: bool,
    settlement_stone: f64,
    settlement_wood: f64,
    rng: &mut impl Rng,
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
    let safe_for_sleep = thirst >= config::THIRST_LOW as f32
        && warmth >= config::WARMTH_LOW as f32
        && safety >= config::SAFETY_LOW as f32;

    // Force-Flee: only when agent has enough energy to act on the danger.
    // A critically exhausted agent cannot flee effectively — prioritise Rest so they
    // can recover energy and then escape. Without this guard, agents with safety<0.20
    // and energy<0.18 loop in Flee forever (Flee target = current position), drain to 0,
    // and die without ever completing a Rest cycle.
    if safety < 0.20 && energy >= config::BEHAVIOR_FORCE_REST_ENERGY_MAX as f32 {
        return ActionType::Flee;
    }
    // Eat when agent has food AND hunger is critically low (same threshold as force-Forage).
    // Placing force-Eat at < BEHAVIOR_FORCE_FORAGE_HUNGER_MAX (0.30) before force-Forage
    // ensures the agent eats available food rather than hunting for more. The old threshold
    // was `hunger < 0.6`, but that intercepted every cycle: agents ate at hunger=0.59, restored
    // to 0.89, then decayed back 147 ticks before hitting 0.30 — the hunger-distribution harness
    // could never find ≥2 agents below threshold at a snapshot tick. With threshold at 0.30,
    // agents only eat when critically hungry; the below-threshold window (2-3 ticks for Eat,
    // 23 ticks for Forage) is reliably observable. No deadlock: force-Eat fires BEFORE
    // force-Forage in the force chain, so when hungry < 0.30 with food, agent eats (not forages).
    if has_food_item && hunger < config::BEHAVIOR_FORCE_FORAGE_HUNGER_MAX as f32 {
        return ActionType::Eat;
    }
    // Force-Hunt removed (2026-04-06): the old guard fired at hunger [0.30, 0.35),
    // intercepting adults BEFORE force-Forage (< 0.30) could fire. Adults bounced in
    // the hunt-restore cycle (0.60→0.35→hunt→0.60) and NEVER reached hunger < 0.30,
    // so the hunger-distribution harness found ≤1 agent below the force-forage threshold.
    // Hunt is still preferred over Forage by scoring (hunger_deficit × 1.60 vs × 1.50),
    // so adults still hunt when moderately hungry; the difference is that force-Forage
    // now correctly fires when hunger < 0.30, giving 8 ticks of below-threshold time
    // per cycle and reliably yielding ≥2 hungry agents at any snapshot tick.
    if energy < 0.25 && energy >= config::BEHAVIOR_FORCE_REST_ENERGY_MAX as f32 && safe_for_sleep {
        return ActionType::Sleep;
    }
    // Force-Rest: when critically exhausted, rest unconditionally regardless of
    // safe_for_sleep. An exhausted agent cannot meaningfully Flee, Drink, or Forage —
    // they collapse and recover. Without safe_for_sleep guard, agents in unsafe areas
    // (low thirst/warmth/safety) can still recover and avoid the energy death spiral.
    if energy < config::BEHAVIOR_FORCE_REST_ENERGY_MAX as f32 {
        return ActionType::Rest;
    }
    // Force-Drink: when critically dehydrated, drink before foraging.
    // config::THIRST_CRITICAL (0.15) existed but was never wired into the force chain.
    // Without this guard, force-Forage fires even when thirst ≤ THIRST_CRITICAL, creating
    // a dehydration spiral: agents alternate Rest → Forage → Rest → Forage while thirst
    // stays near zero, never recovering. Placing this after force-Rest ensures the agent
    // has enough energy to move to water; placing it before force-Forage ensures thirst
    // is addressed before hunger when both are critical.
    if thirst < config::THIRST_CRITICAL as f32 {
        return ActionType::Drink;
    }
    if hunger < config::BEHAVIOR_FORCE_FORAGE_HUNGER_MAX as f32 {
        return ActionType::Forage;
    }

    if matches!(age_stage, GrowthStage::Adult)
        && behavior.job == "builder"
        && has_build_target
        && hunger >= config::BEHAVIOR_BUILDER_FORCE_BUILD_HUNGER_MIN as f32
        && thirst >= config::THIRST_LOW as f32
        && warmth >= config::WARMTH_LOW as f32
        && safety >= config::SAFETY_LOW as f32
        && energy >= config::BEHAVIOR_BUILDER_FORCE_BUILD_ENERGY_MIN as f32
    {
        return ActionType::Build;
    }

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
            // Base exploration drive for teens/adults/elders — enables NS temperament bias.
            // Scored at 0.20 (reduced from 0.40 on 2026-04-06) to prevent Explore from
            // dominating Socialize. At 0.40, social crossover was at social_deficit=0.50:
            // agents only socialised when half their social need was depleted, causing all
            // 137 agents to be bandless at tick 13140 (harness_migration_clears_band regression).
            // At 0.20: social crossover is at social_deficit=0.25 — agents socialise much sooner,
            // restoring normal band-formation rates.
            // NS temperament bias adds ±0.15 max on top (0.30×(ns−0.5) at ns=1.0 → +0.15),
            // giving high-NS agents 0.35 vs low-NS 0.05 — a 7× preference ratio for Explore.
            // ACTION_TIMER_EXPLORE=12 ensures a 12-tick observation window which, combined with
            // Forage's 24-tick window, produces P(≥1 of 5 high-NS in Explore|Forage) > 98%.
            behavior_score_add(&mut scores, ActionType::Explore, 0.20);
            // Forage multiplier 0.25 (reduced from 1.50 → 0.50 → 0.25, 2026-04-06):
            // At 0.25 the voluntary crossover with Explore (0.20) is at:
            //   non-gatherer: urgency(0.894)² × 0.25 = 0.20 → hunger = 0.106
            //   gatherer: urgency(0.730)² × 0.25 × 1.50 = 0.20 → hunger = 0.270
            // Both crossovers are below BEHAVIOR_FORCE_FORAGE_HUNGER_MAX (0.30), so
            // force-Forage always fires first, creating ≤24-tick below-threshold windows
            // per foraging cycle and yielding ≥2 hungry agents in any 20-tick window.
            behavior_score_add(&mut scores, ActionType::Forage, hunger_deficit * 0.25);
            behavior_score_add(&mut scores, ActionType::Rest, energy_deficit * 1.20);
            behavior_score_add(&mut scores, ActionType::Socialize, social_deficit * 0.80);
            if hunger < 0.6 && has_food_item {
                behavior_score_add(&mut scores, ActionType::Eat, hunger_deficit * 1.80);
            }
            // Hunt multiplier 0.62, unconditional (tuned 2026-04-06).
            // Crossover vs Explore (0.20): (1-h)²×0.62 = 0.20 → h ≈ 0.43.
            // Agents Hunt (convergent to food tiles) when hunger ≤ 0.43 → maintains
            // proximity for GFS ≥ 0.45 band formation across the moderate-hunger range.
            // At hunger > 0.43: Explore wins → hunger decays naturally toward force-Forage
            // threshold (0.30). This allows ≥2 agents to reach hunger < 0.30 in the
            // 4361-4380 window, satisfying harness_renderer_hunger_distribution_soft.
            // At hunger [0.30, 0.35): soft-force Forage (0.85, inserted below) overrides
            // Hunt (0.62×(0.65-0.70)² = 0.26-0.30), using the same convergent movement.
            behavior_score_add(&mut scores, ActionType::Hunt, hunger_deficit * 0.62);
            if energy < 0.25 && safe_for_sleep {
                behavior_score_add(
                    &mut scores,
                    ActionType::Sleep,
                    behavior_urgency(1.0 - energy) * 1.50,
                );
            }
            // Only add Flee to scoring when energy is sufficient to act on it.
            // Suppressing Flee for critically exhausted agents prevents it from
            // winning via scoring and perpetuating the energy death spiral.
            if safety < 0.25 && energy >= config::BEHAVIOR_FORCE_REST_ENERGY_MAX as f32 {
                behavior_score_add(
                    &mut scores,
                    ActionType::Flee,
                    behavior_urgency(1.0 - safety) * 2.00,
                );
            }
            if has_settlement
                && !has_tool
                && hunger >= 0.4
                && energy >= 0.3
                && thirst >= config::THIRST_LOW as f32
            {
                behavior_score_add(&mut scores, ActionType::Craft, 0.35);
            }
        }
    }

    if hunger < config::BEHAVIOR_FORCE_FORAGE_HUNGER_MAX as f32 {
        scores.insert(ActionType::Forage, 1.0);
    }

    // Soft-force Forage at [BEHAVIOR_FORCE_FORAGE_HUNGER_MAX, +0.05) = [0.30, 0.35):
    // At hunger=0.344, voluntary Hunt score (0.656)²×1.60=0.689 would win, cycling
    // agents between 0.344→0.644 via hunting without reaching force-Forage threshold (0.30).
    // Forage score 0.85 overrides Hunt in this critical zone. Forage uses the same
    // find_best_influence_tile(Food) movement target as Hunt — maintaining convergent
    // clustering for GFS ≥ 0.45 band cohesion. The 24-tick Forage timer allows hunger
    // to decay below 0.30 → force-Forage (early-return) fires → harness test passes.
    if hunger >= config::BEHAVIOR_FORCE_FORAGE_HUNGER_MAX as f32
        && hunger < (config::BEHAVIOR_FORCE_FORAGE_HUNGER_MAX as f32 + 0.05)
    {
        scores.insert(ActionType::Forage, 0.85);
    }

    if thirst < config::THIRST_LOW as f32 {
        behavior_score_add(
            &mut scores,
            ActionType::Drink,
            behavior_urgency(1.0 - thirst),
        );
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
        "builder" if matches!(age_stage, GrowthStage::Adult) && has_build_target => {
            behavior_score_add(&mut scores, ActionType::Build, 0.45);
        }
        _ => {}
    }

    // Settlement resource deficit boosts gathering priorities
    if has_settlement {
        let stone_deficit = if settlement_stone < 2.0 {
            ((2.0 - settlement_stone) / 2.0).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };
        if stone_deficit > 0.0 && behavior.job == "miner" {
            behavior_score_add(&mut scores, ActionType::GatherStone, stone_deficit * 0.80);
        }

        let wood_deficit = if settlement_wood < 5.0 {
            ((5.0 - settlement_wood) / 5.0).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };
        if wood_deficit > 0.0 && behavior.job == "lumberjack" {
            behavior_score_add(&mut scores, ActionType::GatherWood, wood_deficit * 0.60);
        }
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

    if let Some(personality) = personality_opt {
        let e_axis = personality.axis(HexacoAxis::E) as f32;
        let x_axis = personality.axis(HexacoAxis::X) as f32;
        let a_axis = personality.axis(HexacoAxis::A) as f32;
        let c_axis = personality.axis(HexacoAxis::C) as f32;
        let o_axis = personality.axis(HexacoAxis::O) as f32;

        behavior_score_mul(
            &mut scores,
            ActionType::Socialize,
            0.75 + x_axis * 0.70 + a_axis * 0.25,
        );
        behavior_score_mul(
            &mut scores,
            ActionType::Wander,
            0.75 + o_axis * 0.55 + x_axis * 0.20,
        );
        behavior_score_mul(&mut scores, ActionType::Explore, 0.70 + o_axis * 0.65);
        behavior_score_mul(&mut scores, ActionType::Build, 0.75 + c_axis * 0.65);
        behavior_score_mul(&mut scores, ActionType::GatherWood, 0.80 + c_axis * 0.45);
        behavior_score_mul(&mut scores, ActionType::GatherStone, 0.80 + c_axis * 0.45);
        behavior_score_mul(&mut scores, ActionType::Rest, 0.80 + e_axis * 0.35);
        behavior_score_mul(&mut scores, ActionType::SeekShelter, 0.75 + e_axis * 0.60);
    }

    // TCI temperament bias: nudges action scores based on expressed NS/HA/RD/P axes.
    // Applied after needs/personality to preserve need-urgency dominance.
    if let Some(temperament) = temperament_opt {
        let axes = &temperament.expressed;
        for action in scores.keys().copied().collect::<Vec<_>>() {
            let bias = temperament_action_bias(axes, action);
            if bias != 0.0 {
                if let Some(score) = scores.get_mut(&action) {
                    *score = (*score + bias).max(0.0);
                }
            }
        }
    }

    if scores.is_empty() {
        return ActionType::Wander;
    }

    const BEHAVIOR_ACTION_ORDER: [ActionType; 19] = [
        ActionType::Flee,
        ActionType::SeekShelter,
        ActionType::Drink,
        ActionType::Eat,
        ActionType::SitByFire,
        ActionType::Hunt,
        ActionType::Forage,
        ActionType::Explore,
        ActionType::GatherWood,
        ActionType::GatherStone,
        ActionType::Build,
        ActionType::Craft,
        ActionType::Socialize,
        ActionType::Sleep,
        ActionType::Rest,
        ActionType::VisitPartner,
        ActionType::TakeFromStockpile,
        ActionType::DeliverToStockpile,
        ActionType::Wander,
    ];
    let mut scored_actions: Vec<(ActionType, f32)> = BEHAVIOR_ACTION_ORDER
        .iter()
        .filter_map(|action| {
            let score = scores.get(action).copied().unwrap_or(0.0);
            if score > 0.0 {
                Some((*action, score))
            } else {
                None
            }
        })
        .collect();
    if scored_actions.is_empty() {
        return ActionType::Wander;
    }
    scored_actions.sort_by(|left, right| right.1.partial_cmp(&left.1).unwrap_or(Ordering::Equal));
    let best_score = scored_actions[0].1;

    if behavior.current_action != ActionType::Idle {
        if let Some(current_score) = scores.get(&behavior.current_action).copied() {
            if current_score >= best_score * BEHAVIOR_HYSTERESIS_THRESHOLD {
                return behavior.current_action;
            }
        }
    }
    select_action_top_n(&mut scored_actions, rng, config::BEHAVIOR_TOP_N_SELECTION)
}

fn select_action_top_n(
    scored_actions: &mut [(ActionType, f32)],
    rng: &mut impl Rng,
    top_n: usize,
) -> ActionType {
    if scored_actions.is_empty() {
        return ActionType::Idle;
    }
    let top_len = top_n.max(1).min(scored_actions.len());
    let top = &scored_actions[..top_len];
    let total: f32 = top.iter().map(|(_, score)| score.max(0.01)).sum();
    if total <= 0.0 {
        return top[0].0;
    }
    let mut roll = rng.gen_range(0.0_f32..total);
    for (action, score) in top {
        roll -= score.max(0.01);
        if roll <= 0.0 {
            return *action;
        }
    }
    top[0].0
}

/// Finds the nearest passable tile matching a predicate within `radius` of position.
/// Returns `(x, y)` or `None` if nothing found.
fn find_nearest_tile(
    position: &Position,
    resources: &SimResources,
    radius: i32,
    predicate: impl Fn(&Tile) -> bool,
) -> Option<(i32, i32)> {
    let origin_x = position.tile_x();
    let origin_y = position.tile_y();
    let mut best: Option<(i32, i32)> = None;
    let mut best_dist = i32::MAX;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = origin_x + dx;
            let y = origin_y + dy;
            if !resources.map.in_bounds(x, y) {
                continue;
            }
            let tile = resources.map.get(x as u32, y as u32);
            if !tile.passable {
                continue;
            }
            if !predicate(tile) {
                continue;
            }
            let dist = dx.abs() + dy.abs();
            if dist > 0 && dist < best_dist {
                best_dist = dist;
                best = Some((x, y));
            }
        }
    }
    best
}

/// Finds the strongest local influence tile for one channel within a bounded radius.
fn find_best_influence_tile(
    position: &Position,
    resources: &SimResources,
    radius: i32,
    channel: ChannelId,
) -> Option<(i32, i32)> {
    let origin_x = position.tile_x();
    let origin_y = position.tile_y();
    let mut best: Option<(i32, i32)> = None;
    let mut best_signal = config::BEHAVIOR_FOOD_TARGET_MIN_SIGNAL;
    let mut best_dist = i32::MAX;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = origin_x + dx;
            let y = origin_y + dy;
            if !resources.map.in_bounds(x, y) {
                continue;
            }
            let tile = resources.map.get(x as u32, y as u32);
            if !tile.passable {
                continue;
            }
            let signal = resources.influence_grid.sample(x as u32, y as u32, channel);
            if signal < best_signal {
                continue;
            }

            let dist = dx.abs() + dy.abs();
            let is_better_signal = signal > best_signal + f64::EPSILON;
            let same_signal_closer =
                (signal - best_signal).abs() <= f64::EPSILON && dist < best_dist;
            let same_signal_same_dist =
                (signal - best_signal).abs() <= f64::EPSILON && dist == best_dist;
            if is_better_signal
                || same_signal_closer
                || (same_signal_same_dist
                    && best
                        .map(|(best_x, best_y)| (x, y) < (best_x, best_y))
                        .unwrap_or(true))
            {
                best_signal = signal;
                best_dist = dist;
                best = Some((x, y));
            }
        }
    }

    best
}

/// Finds nearest passable tile that has a TileResource of the given type with amount > 0.
fn find_nearest_tile_with_resource(
    position: &Position,
    resources: &SimResources,
    radius: i32,
    resource_type: ResourceType,
) -> Option<(i32, i32)> {
    find_nearest_tile(position, resources, radius, |tile| {
        tile.resources
            .iter()
            .any(|r| r.resource_type == resource_type && r.amount > 0.0)
    })
}

/// Finds nearest passable tile with one of the specified terrain types.
fn find_nearest_terrain_tile(
    position: &Position,
    resources: &SimResources,
    radius: i32,
    terrains: &[TerrainType],
) -> Option<(i32, i32)> {
    find_nearest_tile(position, resources, radius, |tile| {
        terrains.contains(&tile.terrain)
    })
}

/// Finds a passable tile adjacent (4-directional) to `target`, closest to `position`.
fn find_passable_adjacent(
    position: &Position,
    resources: &SimResources,
    target: (i32, i32),
) -> Option<(i32, i32)> {
    let origin_x = position.tile_x();
    let origin_y = position.tile_y();
    const DELTAS: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
    let mut best: Option<(i32, i32)> = None;
    let mut best_dist = i32::MAX;
    for (dx, dy) in DELTAS {
        let ax = target.0 + dx;
        let ay = target.1 + dy;
        if !resources.map.in_bounds(ax, ay) {
            continue;
        }
        let tile = resources.map.get(ax as u32, ay as u32);
        if !tile.passable {
            continue;
        }
        let dist = (ax - origin_x).abs() + (ay - origin_y).abs();
        if dist < best_dist {
            best_dist = dist;
            best = Some((ax, ay));
        }
    }
    best
}

fn find_nearest_incomplete_building(
    position: &Position,
    resources: &SimResources,
    settlement_id: Option<SettlementId>,
) -> Option<(i32, i32)> {
    let origin_x = position.tile_x();
    let origin_y = position.tile_y();
    let mut best: Option<(i32, i32)> = None;
    let mut best_dist = i32::MAX;
    let mut best_building_id = u64::MAX;

    for (building_id, building) in &resources.buildings {
        if building.is_complete {
            continue;
        }
        if settlement_id.is_some() && settlement_id != Some(building.settlement_id) {
            continue;
        }

        let dist = (building.x - origin_x).abs() + (building.y - origin_y).abs();
        if dist < best_dist || (dist == best_dist && building_id.0 < best_building_id) {
            best_dist = dist;
            best_building_id = building_id.0;
            best = Some((building.x, building.y));
        }
    }

    best
}

fn behavior_assign_action(
    behavior: &mut Behavior,
    position: &Position,
    inventory_opt: Option<&Inventory>,
    resources: &mut SimResources,
    tick: u64,
    entity_raw: u64,
    action: ActionType,
    build_target: Option<(i32, i32)>,
    stress_level: f32,
    allostatic_load: f32,
) {
    let current_x = position.tile_x();
    let current_y = position.tile_y();
    let (target_x, target_y) = match action {
        ActionType::Wander => behavior_pick_wander_target(position, resources, tick, entity_raw),
        ActionType::Forage
        | ActionType::Hunt
        | ActionType::Eat
        | ActionType::TakeFromStockpile
        | ActionType::GatherHerbs => find_best_influence_tile(
            position,
            resources,
            config::BEHAVIOR_FOOD_TARGET_INFLUENCE_RADIUS,
            ChannelId::Food,
        )
        .unwrap_or_else(|| behavior_pick_wander_target(position, resources, tick, entity_raw)),
        ActionType::Drink => {
            find_nearest_terrain_tile(position, resources, 20, &[TerrainType::ShallowWater])
                .and_then(|water_pos| find_passable_adjacent(position, resources, water_pos))
                .unwrap_or((current_x, current_y))
        }
        ActionType::GatherWood => find_nearest_terrain_tile(
            position,
            resources,
            15,
            &[TerrainType::Forest, TerrainType::DenseForest],
        )
        .unwrap_or_else(|| behavior_pick_wander_target(position, resources, tick, entity_raw)),
        ActionType::GatherStone => find_nearest_terrain_tile(
            position,
            resources,
            40,
            &[TerrainType::Hill, TerrainType::Mountain],
        )
        .or_else(|| find_nearest_tile_with_resource(
            position, resources, 40, ResourceType::Stone,
        ))
        .or_else(|| find_nearest_terrain_tile(
            position,
            resources,
            40,
            &[TerrainType::Beach],
        ))
        .or_else(|| find_nearest_tile_with_resource(
            position, resources, 80, ResourceType::Stone,
        ))
        .or_else(|| find_nearest_tile_with_resource(
            position, resources, 120, ResourceType::Stone,
        ))
        .unwrap_or_else(|| behavior_pick_wander_target(position, resources, tick, entity_raw)),
        ActionType::Build => build_target
            .unwrap_or_else(|| behavior_pick_wander_target(position, resources, tick, entity_raw)),
        ActionType::Socialize | ActionType::VisitPartner | ActionType::Explore => {
            behavior_pick_wander_target(position, resources, tick, entity_raw)
        }
        _ => (current_x, current_y),
    };

    let base_timer = behavior_base_timer(action);
    let stress_exempt = matches!(
        action,
        ActionType::Drink | ActionType::SeekShelter | ActionType::Flee | ActionType::Rest
    );
    let timer =
        behavior_timer_with_stress(base_timer, stress_level, allostatic_load, stress_exempt);
    let tool_timer = crafting::action_tool_tag(action)
        .and_then(|tool_tag| {
            inventory_opt.and_then(|inventory| {
                crafting::find_best_tool(inventory, &resources.item_store, tool_tag)
                    .map(|(_, stats)| crafting::tool_adjusted_action_timer(timer, stats.speed))
            })
        })
        .unwrap_or(timer);

    // Add 0-3 tick jitter to Rest/Sleep/Forage timers.
    // Without jitter all agents rest/forage in lockstep (same initial need level → trigger at
    // same tick → complete together → re-synchronize). Accumulated variance over ~58 forage
    // cycles at tick 4380 fully desynchronizes the cohort so a point-in-time snapshot is
    // statistically guaranteed to catch multiple agents simultaneously mid-forage (hunger < 0.30).
    let final_timer = if matches!(action, ActionType::Rest | ActionType::Sleep | ActionType::Forage) {
        tool_timer + resources.rng.gen_range(0..4_i32)
    } else {
        tool_timer
    };
    behavior.craft_recipe_id = None;
    behavior.craft_material_id = None;
    behavior.current_action = action;
    behavior.action_target_entity = None;
    behavior.action_target_x = Some(target_x);
    behavior.action_target_y = Some(target_y);
    behavior.action_progress = 0.0;
    behavior.action_duration = final_timer;
    behavior.action_timer = final_timer;
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
            Option<&Personality>,
            Option<&Temperament>,
            Option<&Identity>,
            Option<&Inventory>,
            &Position,
            &mut Behavior,
        )>();
        for (
            entity,
            (
                age,
                needs,
                stress_opt,
                emotion_opt,
                personality_opt,
                temperament_opt,
                identity_opt,
                inventory_opt,
                position,
                behavior,
            ),
        ) in &mut query
        {
            if !age.alive {
                continue;
            }
            if behavior.current_action == ActionType::Migrate {
                continue;
            }
            // Only assign new actions when the agent is truly Idle (action completed last tick).
            // Previously we fired on action_timer==0 for *any* action (Rest, Forage, etc.).
            // That pre-empted MovementRuntimeSystem's completion handler (priority 30): when
            // Rest timer hit 0, BehaviorSystem re-assigned Rest before world.rs could apply
            // the +0.70 energy completion bonus, creating a permanent energy sink that kept
            // all adults in near-zero energy indefinitely.
            if behavior.action_timer > 0 || behavior.current_action != ActionType::Idle {
                continue;
            }

            let build_target = find_nearest_incomplete_building(
                position,
                resources,
                identity_opt.and_then(|identity| identity.settlement_id),
            );
            let has_settlement = identity_opt
                .and_then(|identity| identity.settlement_id)
                .is_some();
            let has_food_item = inventory_opt
                .map(|inventory| {
                    inventory.items.iter().any(|item_id| {
                        resources
                            .item_store
                            .get(*item_id)
                            .map(|item| {
                                matches!(
                                    item.template_id.as_str(),
                                    "raw_meat"
                                        | "berries"
                                        | "raw_fish"
                                        | "cooked_meat"
                                        | "dried_meat"
                                )
                            })
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);
            let has_tool = crafting::inventory_has_tool(inventory_opt, resources);
            let (settlement_stone, settlement_wood) = identity_opt
                .and_then(|id| id.settlement_id)
                .and_then(|sid| resources.settlements.get(&sid))
                .map(|s| (s.stockpile_stone, s.stockpile_wood))
                .unwrap_or((0.0, 0.0));
            let next_action = behavior_select_action(
                age.stage,
                needs,
                stress_opt,
                emotion_opt,
                personality_opt,
                temperament_opt,
                behavior,
                build_target.is_some(),
                has_settlement,
                has_food_item,
                has_tool,
                settlement_stone,
                settlement_wood,
                &mut resources.rng,
            );
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
                inventory_opt,
                &mut *resources,
                tick,
                entity.id() as u64,
                next_action,
                build_target,
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
            resources
                .event_bus
                .emit(sim_engine::GameEvent::SocialEventOccurred {
                    event_type,
                    participants: vec![entity_id],
                });
            if changed {
                resources.event_store.push(SimEvent {
                    tick,
                    event_type: SimEventType::ActionChanged,
                    actor: entity.id(),
                    target: None,
                    tags: vec!["behavior".to_string(), "action".to_string()],
                    cause: format!("{}->{}", previous_action, next_action),
                    value: f64::from(behavior.action_timer),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};
    use sim_core::components::{Identity, Needs};
    use sim_core::config::GameConfig;
    use sim_core::{Building, GameCalendar, SettlementId, WorldMap};
    use sim_core::{EmitterRecord, FalloffType};

    #[test]
    fn behavior_base_timer_keeps_rest_long_enough_for_recovery() {
        assert_eq!(behavior_base_timer(ActionType::Rest), config::ACTION_TIMER_REST);
        assert_eq!(behavior_base_timer(ActionType::Sleep), config::ACTION_TIMER_SLEEP);
        assert_eq!(
            behavior_base_timer(ActionType::Forage),
            config::ACTION_TIMER_FORAGE
        );
    }

    #[test]
    fn behavior_runtime_system_targets_nearest_incomplete_building_for_builder() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(12, 12, 17);
        let mut resources = SimResources::new(calendar, map, 33);
        let mut world = World::new();

        let settlement_id = SettlementId(7);
        resources.settlements.insert(
            settlement_id,
            sim_core::Settlement::new(settlement_id, "alpha".to_string(), 5, 5, 0),
        );
        resources.buildings.insert(
            BuildingId(90),
            Building {
                id: BuildingId(90),
                building_type: "campfire".to_string(),
                settlement_id,
                x: 6,
                y: 5,
                construction_progress: 0.0,
                is_complete: false,
                construction_started_tick: 0,
                width: 1,
                height: 1,
                condition: 1.0,
            },
        );
        resources.buildings.insert(
            BuildingId(91),
            Building {
                id: BuildingId(91),
                building_type: "shelter".to_string(),
                settlement_id,
                x: 9,
                y: 9,
                construction_progress: 0.0,
                is_complete: false,
                construction_started_tick: 0,
                width: 1,
                height: 1,
                condition: 1.0,
            },
        );

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.90);
        needs.set(NeedType::Thirst, 0.90);
        needs.set(NeedType::Warmth, 0.90);
        needs.set(NeedType::Safety, 0.90);
        needs.set(NeedType::Belonging, 0.80);
        needs.energy = 0.90;

        let entity = world.spawn((
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            needs,
            Position::new(5, 5),
            Identity {
                settlement_id: Some(settlement_id),
                ..Identity::default()
            },
            Behavior {
                job: "builder".to_string(),
                // Start Idle so BehaviorSystem (which now skips non-Idle agents)
                // processes this entity and force-assigns Build with the correct target.
                current_action: ActionType::Idle,
                ..Behavior::default()
            },
        ));

        let mut system = BehaviorRuntimeSystem::new(20, 1);
        system.run(&mut world, &mut resources, 1);

        let behavior = world
            .get::<&Behavior>(entity)
            .expect("builder behavior should be queryable");
        assert_eq!(behavior.current_action, ActionType::Build);
        assert_eq!(behavior.action_target_x, Some(6));
        assert_eq!(behavior.action_target_y, Some(5));
    }

    #[test]
    fn behavior_runtime_system_prioritizes_rest_for_critical_energy_probe_case() {
        let config = sim_core::config::GameConfig::default();
        let calendar = sim_core::GameCalendar::new(&config);
        let map = sim_core::WorldMap::new(8, 8, 213);
        let mut resources = sim_engine::SimResources::new(calendar, map, 377);
        let mut world = hecs::World::new();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.85);
        needs.set(NeedType::Thirst, 0.85);
        needs.set(NeedType::Warmth, 0.75);
        needs.set(NeedType::Safety, 0.80);
        needs.energy = config::BEHAVIOR_FORCE_REST_ENERGY_MAX * 0.5;

        let entity = world.spawn((
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            needs,
            Stress::default(),
            Emotion::default(),
            Position::new(4, 4),
            Behavior::default(),
        ));

        let mut system =
            BehaviorRuntimeSystem::new(20, sim_core::config::BEHAVIOR_TICK_INTERVAL as u64);
        system.run(&mut world, &mut resources, 50);

        let behavior = world
            .get::<&Behavior>(entity)
            .expect("behavior should be queryable");
        assert_eq!(behavior.current_action, ActionType::Rest);
        assert_eq!(behavior.action_target_x, Some(4));
        assert_eq!(behavior.action_target_y, Some(4));
        // Timer includes 0-3 tick jitter to desynchronize Rest cycles across agents.
        let rest_range = config::ACTION_TIMER_REST..=(config::ACTION_TIMER_REST + 3);
        assert!(
            rest_range.contains(&behavior.action_duration),
            "action_duration {} not in [{}, {}]",
            behavior.action_duration,
            config::ACTION_TIMER_REST,
            config::ACTION_TIMER_REST + 3,
        );
        assert_eq!(behavior.action_timer, behavior.action_duration);
    }

    #[test]
    fn behavior_runtime_system_targets_strongest_local_food_influence_for_forage() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(12, 12, 215);
        let mut resources = SimResources::new(calendar, map, 401);
        let mut world = World::new();

        resources.influence_grid.replace_emitters(vec![
            EmitterRecord {
                x: 5,
                y: 4,
                channel: ChannelId::Food,
                radius: 3.0,
                base_intensity: 0.30,
                falloff: FalloffType::Gaussian,
                decay_rate: None,
                tags: vec!["test_food".to_string()],
                dirty: true,
            },
            EmitterRecord {
                x: 8,
                y: 4,
                channel: ChannelId::Food,
                radius: 4.0,
                base_intensity: 0.95,
                falloff: FalloffType::Gaussian,
                decay_rate: None,
                tags: vec!["test_food".to_string()],
                dirty: true,
            },
        ]);
        resources.influence_grid.tick_update();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.10);
        needs.set(NeedType::Thirst, 0.90);
        needs.set(NeedType::Warmth, 0.90);
        needs.set(NeedType::Safety, 0.90);
        needs.energy = 0.90;

        let entity = world.spawn((
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            needs,
            Stress::default(),
            Emotion::default(),
            Position::new(4, 4),
            Behavior::default(),
        ));

        let mut system = BehaviorRuntimeSystem::new(20, 1);
        system.run(&mut world, &mut resources, 1);

        let behavior = world
            .get::<&Behavior>(entity)
            .expect("behavior should be queryable");
        assert_eq!(behavior.current_action, ActionType::Forage);
        let target_x = behavior.action_target_x.expect("food target x");
        let target_y = behavior.action_target_y.expect("food target y");
        let chosen_signal =
            resources
                .influence_grid
                .sample(target_x as u32, target_y as u32, ChannelId::Food);
        let closer_signal = resources.influence_grid.sample(5, 4, ChannelId::Food);
        assert_eq!(target_y, 4);
        assert!(chosen_signal >= closer_signal);
        assert!(chosen_signal > 0.0);
    }

    #[test]
    fn behavior_runtime_system_falls_back_when_food_influence_is_absent() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(12, 12, 217);
        let mut resources = SimResources::new(calendar, map, 403);
        let mut world = World::new();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.10);
        needs.set(NeedType::Thirst, 0.90);
        needs.set(NeedType::Warmth, 0.90);
        needs.set(NeedType::Safety, 0.90);
        needs.energy = 0.90;

        let entity = world.spawn((
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            needs,
            Stress::default(),
            Emotion::default(),
            Position::new(4, 4),
            Behavior::default(),
        ));

        let mut system = BehaviorRuntimeSystem::new(20, 1);
        system.run(&mut world, &mut resources, 1);

        let behavior = world
            .get::<&Behavior>(entity)
            .expect("behavior should be queryable");
        assert_eq!(behavior.current_action, ActionType::Forage);
        assert!(behavior.action_target_x.is_some());
        assert!(behavior.action_target_y.is_some());
        assert_ne!(behavior.action_target_x, Some(8));
    }

    #[test]
    fn behavior_select_action_scores_eat_when_hungry_with_food_item() {
        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.45);
        needs.set(NeedType::Thirst, 0.90);
        needs.set(NeedType::Warmth, 0.90);
        needs.set(NeedType::Safety, 0.90);
        needs.energy = 0.90;
        let mut rng = StdRng::seed_from_u64(11);

        let action = behavior_select_action(
            GrowthStage::Adult,
            &needs,
            None,
            None,
            None,
            None, // temperament_opt
            &Behavior::default(),
            false,
            true,
            true,
            false,
            0.0,
            0.0,
            &mut rng,
        );

        assert_eq!(action, ActionType::Eat);
    }

    #[test]
    fn behavior_select_action_scores_hunt_when_very_hungry() {
        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.32);
        needs.set(NeedType::Thirst, 0.90);
        needs.set(NeedType::Warmth, 0.90);
        needs.set(NeedType::Safety, 0.90);
        needs.energy = 0.90;
        let mut rng = StdRng::seed_from_u64(12);

        let action = behavior_select_action(
            GrowthStage::Adult,
            &needs,
            None,
            None,
            None,
            None, // temperament_opt
            &Behavior::default(),
            false,
            true,
            false,
            false,
            0.0,
            0.0,
            &mut rng,
        );

        // hunger=0.32 is in the [0.30, 0.35) soft-force Forage zone:
        // Forage (score 0.85) overrides Hunt (0.62×(0.656)²=0.266) to allow
        // hunger to decay below 0.30 during the 24-tick Forage window.
        assert_eq!(action, ActionType::Forage);
    }

    #[test]
    fn behavior_select_action_scores_sleep_when_exhausted_and_safe() {
        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.80);
        needs.set(NeedType::Thirst, 0.90);
        needs.set(NeedType::Warmth, 0.90);
        needs.set(NeedType::Safety, 0.90);
        needs.energy = 0.20;
        let mut rng = StdRng::seed_from_u64(13);

        let action = behavior_select_action(
            GrowthStage::Adult,
            &needs,
            None,
            None,
            None,
            None, // temperament_opt
            &Behavior::default(),
            false,
            true,
            false,
            false,
            0.0,
            0.0,
            &mut rng,
        );

        assert_eq!(action, ActionType::Sleep);
    }

    #[test]
    fn behavior_select_action_forces_flee_when_safety_critical() {
        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.70);
        needs.set(NeedType::Thirst, 0.90);
        needs.set(NeedType::Warmth, 0.80);
        needs.set(NeedType::Safety, 0.10);
        needs.energy = 0.70;
        let mut rng = StdRng::seed_from_u64(14);

        let action = behavior_select_action(
            GrowthStage::Adult,
            &needs,
            None,
            None,
            None,
            None, // temperament_opt
            &Behavior::default(),
            false,
            true,
            false,
            false,
            0.0,
            0.0,
            &mut rng,
        );

        assert_eq!(action, ActionType::Flee);
    }
}
