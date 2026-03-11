#![allow(unused_imports)]
// TODO(v3.1): REFACTOR - split legacy threshold/chronicle behaviors from reusable observation and telemetry paths.

use hecs::{Entity, World};
use rand::Rng;
use sim_core::components::{
    Age, Behavior, Body as BodyComponent, Coping, Economic, Emotion, Identity, Intelligence,
    Memory, MemoryEntry, Needs, Personality, Position, Skills, Social, Stress, Traits, Values,
};
use sim_core::config;
use sim_core::{
    ActionType, AttachmentType, BuildingId, CopingStrategyId, EmotionType, EntityId, GrowthStage,
    HexacoAxis, HexacoFacet, IntelligenceType, MentalBreakType, NeedType, RelationType,
    ResourceType, SettlementId, Sex, SocialClass, TechState, ValueType,
};
use sim_engine::{
    ChronicleCluster, ChronicleEvent, ChronicleEventCause, ChronicleEventType, ChronicleSummary,
    SimResources, SimSystem,
};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::body;

const STATS_RECORDER_MAX_HISTORY: usize = 200;

/// Rust runtime system for stat-sync derived composite cache refresh.
///
/// This performs active writes on `SimResources.stat_sync_derived`.
#[derive(Debug, Clone)]
pub struct StatSyncRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl StatSyncRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for StatSyncRuntimeSystem {
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
        let mut next_cache: HashMap<EntityId, [f32; 8]> = HashMap::new();

        let mut query = world.query::<(
            &Age,
            Option<&Personality>,
            Option<&Emotion>,
            Option<&BodyComponent>,
            Option<&Values>,
            Option<&Needs>,
            Option<&Intelligence>,
        )>();
        for (
            entity,
            (age, personality_opt, emotion_opt, body_opt, values_opt, needs_opt, intel_opt),
        ) in &mut query
        {
            if !age.alive {
                continue;
            }
            let x = personality_opt
                .map(|personality_component| personality_component.axis(HexacoAxis::X) as f32)
                .unwrap_or(0.5);
            let a = personality_opt
                .map(|personality_component| personality_component.axis(HexacoAxis::A) as f32)
                .unwrap_or(0.5);
            let h = personality_opt
                .map(|personality_component| personality_component.axis(HexacoAxis::H) as f32)
                .unwrap_or(0.5);
            let e = personality_opt
                .map(|personality_component| personality_component.axis(HexacoAxis::E) as f32)
                .unwrap_or(0.5);
            let o = personality_opt
                .map(|personality_component| personality_component.axis(HexacoAxis::O) as f32)
                .unwrap_or(0.5);
            let c = personality_opt
                .map(|personality_component| personality_component.axis(HexacoAxis::C) as f32)
                .unwrap_or(0.5);

            let joy = emotion_opt
                .map(|emotion| emotion.get(EmotionType::Joy) as f32)
                .unwrap_or(0.0);
            let anticipation = emotion_opt
                .map(|emotion| emotion.get(EmotionType::Anticipation) as f32)
                .unwrap_or(0.0);
            let anger = emotion_opt
                .map(|emotion| emotion.get(EmotionType::Anger) as f32)
                .unwrap_or(0.0);

            let str_pot = body_opt
                .map(|body_component| {
                    (body_component.str_potential as f32 / config::BODY_POTENTIAL_MAX as f32)
                        .clamp(0.0, 1.0)
                })
                .unwrap_or(0.5);
            let attractiveness = body_opt
                .map(|body_component| body_component.attractiveness)
                .unwrap_or(0.5);
            let height = body_opt
                .map(|body_component| body_component.height)
                .unwrap_or(0.5);

            let value_norm = |value_type: ValueType| -> f32 {
                values_opt
                    .map(|values| ((values.get(value_type) as f32 + 1.0) * 0.5).clamp(0.0, 1.0))
                    .unwrap_or(0.5)
            };
            let romance = value_norm(ValueType::Romance);
            let truth = value_norm(ValueType::Truth);
            let artwork = value_norm(ValueType::Artwork);
            let knowledge = value_norm(ValueType::Knowledge);
            let merriment = value_norm(ValueType::Merriment);
            let friendship = value_norm(ValueType::Friendship);
            let competition = value_norm(ValueType::Competition);
            let recognition = needs_opt
                .map(|needs| needs.get(NeedType::Recognition) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);

            let i_ling = intel_opt
                .map(|intelligence| intelligence.get(IntelligenceType::Linguistic) as f32)
                .unwrap_or(0.5);
            let i_log = intel_opt
                .map(|intelligence| intelligence.get(IntelligenceType::Logical) as f32)
                .unwrap_or(0.5);
            let i_spa = intel_opt
                .map(|intelligence| intelligence.get(IntelligenceType::Spatial) as f32)
                .unwrap_or(0.5);
            let i_mus = intel_opt
                .map(|intelligence| intelligence.get(IntelligenceType::Musical) as f32)
                .unwrap_or(0.5);
            let i_kin = intel_opt
                .map(|intelligence| intelligence.get(IntelligenceType::Kinesthetic) as f32)
                .unwrap_or(0.5);
            let i_inter = intel_opt
                .map(|intelligence| intelligence.get(IntelligenceType::Interpersonal) as f32)
                .unwrap_or(0.5);
            let i_intra = intel_opt
                .map(|intelligence| intelligence.get(IntelligenceType::Intrapersonal) as f32)
                .unwrap_or(0.5);
            let i_nat = intel_opt
                .map(|intelligence| intelligence.get(IntelligenceType::Naturalistic) as f32)
                .unwrap_or(0.5);

            let inputs = [
                x,
                a,
                h,
                e,
                o,
                c,
                joy,
                anticipation,
                anger,
                str_pot,
                romance,
                truth,
                artwork,
                knowledge,
                merriment,
                friendship,
                competition,
                recognition,
                i_ling,
                i_log,
                i_spa,
                i_mus,
                i_kin,
                i_inter,
                i_intra,
                i_nat,
                attractiveness,
                height,
                age.years as f32,
            ];
            let derived = body::stat_sync_derived_scores(&inputs);
            next_cache.insert(EntityId(entity.id() as u64), derived);
        }
        resources.stat_sync_derived = next_cache;
    }
}

pub const STAT_THRESHOLD_FLAG_HUNGER_LOW: u32 = 1 << 0;
pub const STAT_THRESHOLD_FLAG_STRESS_HIGH: u32 = 1 << 1;

/// Rust runtime system for threshold evaluation and effect application.
///
/// This performs active writes on `SimResources.stat_threshold_flags` and
/// entity `Behavior.current_action`.
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

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let previous_flags = resources.stat_threshold_flags.clone();
        let mut next_flags: HashMap<EntityId, u32> = HashMap::new();

        let mut query = world.query::<(&Age, &Needs, &Stress, &mut Behavior)>();
        for (entity, (age, needs, stress, behavior)) in &mut query {
            if !age.alive {
                continue;
            }
            let entity_id = EntityId(entity.id() as u64);
            let current_flags = previous_flags.get(&entity_id).copied().unwrap_or(0);

            let hunger_value = (needs.get(NeedType::Hunger) as f32 * 1000.0).round() as i32;
            let stress_value = (stress.level as f32 * 1000.0).round() as i32;

            let hunger_active = body::stat_threshold_is_active(
                hunger_value,
                200,
                0,
                50,
                (current_flags & STAT_THRESHOLD_FLAG_HUNGER_LOW) != 0,
            );
            let stress_active = body::stat_threshold_is_active(
                stress_value,
                700,
                1,
                40,
                (current_flags & STAT_THRESHOLD_FLAG_STRESS_HIGH) != 0,
            );

            let mut flags = 0_u32;
            if hunger_active {
                flags |= STAT_THRESHOLD_FLAG_HUNGER_LOW;
            }
            if stress_active {
                flags |= STAT_THRESHOLD_FLAG_STRESS_HIGH;
            }
            if flags != 0 {
                next_flags.insert(entity_id, flags);
            }

            if stress_active {
                behavior.current_action = ActionType::Rest;
            } else if hunger_active {
                behavior.current_action = ActionType::Forage;
            } else if matches!(
                behavior.current_action,
                ActionType::Rest | ActionType::Forage
            ) {
                behavior.current_action = ActionType::Idle;
            }
        }

        resources.stat_threshold_flags = next_flags;
    }
}

/// Rust runtime system for aggregated simulation stats snapshots.
///
/// This performs active writes on `SimResources.stats_history` and
/// `SimResources.stats_peak_population`.
#[derive(Debug, Clone)]
pub struct StatsRecorderRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl StatsRecorderRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for StatsRecorderRuntimeSystem {
    fn name(&self) -> &'static str {
        "stats_recorder"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        let mut pop = 0_usize;
        let mut gatherers = 0_u32;
        let mut lumberjacks = 0_u32;
        let mut builders = 0_u32;
        let mut miners = 0_u32;
        let mut none_job = 0_u32;

        let mut query = world.query::<(&Age, Option<&Behavior>)>();
        for (_, (age, behavior_opt)) in &mut query {
            if !age.alive {
                continue;
            }
            pop += 1;
            let Some(behavior) = behavior_opt else {
                none_job = none_job.saturating_add(1);
                continue;
            };
            match behavior.job.as_str() {
                "gatherer" => gatherers = gatherers.saturating_add(1),
                "lumberjack" => lumberjacks = lumberjacks.saturating_add(1),
                "builder" => builders = builders.saturating_add(1),
                "miner" => miners = miners.saturating_add(1),
                _ => none_job = none_job.saturating_add(1),
            }
        }

        let mut food = 0.0_f64;
        let mut wood = 0.0_f64;
        let mut stone = 0.0_f64;
        for settlement in resources.settlements.values() {
            food += settlement.stockpile_food.max(0.0);
            wood += settlement.stockpile_wood.max(0.0);
            stone += settlement.stockpile_stone.max(0.0);
        }

        if pop > resources.stats_peak_population {
            resources.stats_peak_population = pop;
        }
        resources
            .stats_history
            .push(sim_engine::RuntimeStatsSnapshot {
                tick,
                pop,
                food,
                wood,
                stone,
                gatherers,
                lumberjacks,
                builders,
                miners,
                none_job,
            });
        if resources.stats_history.len() > STATS_RECORDER_MAX_HISTORY {
            let overflow = resources.stats_history.len() - STATS_RECORDER_MAX_HISTORY;
            resources.stats_history.drain(0..overflow);
        }
    }
}

// ── Chronicle System ─────────────────────────────────────────────────

/// Constants matching chronicle_system.gd
const CHRONICLE_PRUNE_INTERVAL_YEARS: i32 = 10;
const CHRONICLE_LOW_IMPORTANCE_MAX_AGE_YEARS: i32 = 20;
const CHRONICLE_MED_IMPORTANCE_MAX_AGE_YEARS: i32 = 50;

/// Rust runtime system for chronicle event pruning.
///
/// Periodically removes old low-significance chronicle entries while keeping
/// bounded high-significance history intact.
#[derive(Debug, Clone)]
// TODO(v3.1): DELETE - replace legacy chronicle pruning path with v3.1 causal log + observation/oracle layers.
pub struct ChronicleRuntimeSystem {
    priority: u32,
    tick_interval: u64,
    last_prune_year: i32,
    last_summarized_tick: u64,
}

impl ChronicleRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
            last_prune_year: 0,
            last_summarized_tick: 0,
        }
    }
}

impl SimSystem for ChronicleRuntimeSystem {
    fn name(&self) -> &'static str {
        "chronicle_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, _world: &mut World, resources: &mut SimResources, tick: u64) {
        if should_summarize_chronicle(tick, self.last_summarized_tick) {
            summarize_recent_chronicle_events(_world, resources, self.last_summarized_tick);
            self.last_summarized_tick = tick;
        }

        let ticks_per_year = config::TICKS_PER_YEAR as i32;
        if ticks_per_year <= 0 {
            return;
        }
        let current_year = tick as i32 / ticks_per_year;

        if !body::chronicle_should_prune(
            current_year,
            self.last_prune_year,
            CHRONICLE_PRUNE_INTERVAL_YEARS,
        ) {
            return;
        }
        self.last_prune_year = current_year;

        let low_cutoff = body::chronicle_cutoff_tick(
            current_year,
            CHRONICLE_LOW_IMPORTANCE_MAX_AGE_YEARS,
            ticks_per_year,
        )
        .max(0) as u64;
        let med_cutoff = body::chronicle_cutoff_tick(
            current_year,
            CHRONICLE_MED_IMPORTANCE_MAX_AGE_YEARS,
            ticks_per_year,
        )
        .max(0) as u64;

        resources
            .chronicle_log
            .prune_by_significance(low_cutoff, med_cutoff);
    }
}

fn should_summarize_chronicle(tick: u64, last_summarized_tick: u64) -> bool {
    tick.saturating_sub(last_summarized_tick) >= config::CHRONICLE_SUMMARY_INTERVAL_TICKS
}

fn summarize_recent_chronicle_events(
    world: &World,
    resources: &mut SimResources,
    last_summarized_tick: u64,
) {
    let recent_events: Vec<ChronicleEvent> = resources
        .chronicle_log
        .events_since(last_summarized_tick)
        .into_iter()
        .cloned()
        .collect();
    if recent_events.is_empty() {
        return;
    }

    let mut consumed_social_events: BTreeSet<(u64, EntityId, i32, i32)> = BTreeSet::new();
    for summary in build_social_gathering_summaries(&recent_events) {
        for event in &summary.events {
            consumed_social_events.insert((event.tick, event.entity_id, event.tile_x, event.tile_y));
        }
        if let Some(entry) = summary_to_timeline_summary(world, &summary) {
            append_chronicle_summary(resources, entry);
        }
    }

    let mut clusters_by_entity: BTreeMap<EntityId, Vec<ChronicleCluster>> = BTreeMap::new();
    for event in &recent_events {
        let event_key = (event.tick, event.entity_id, event.tile_x, event.tile_y);
        if consumed_social_events.contains(&event_key) {
            continue;
        }
        let clusters = clusters_by_entity.entry(event.entity_id).or_default();
        if let Some(last_cluster) = clusters.last_mut() {
            if chronicle_events_can_cluster(last_cluster.events.last(), event) {
                last_cluster.push(event.clone());
                continue;
            }
        }
        clusters.push(ChronicleCluster::new(event.clone()));
    }

    for clusters in clusters_by_entity.into_values() {
        for cluster in clusters {
            if let Some(summary) = cluster_to_timeline_summary(world, &cluster) {
                append_chronicle_summary(resources, summary);
            }
        }
    }
}

fn build_social_gathering_summaries(events: &[ChronicleEvent]) -> Vec<ChronicleCluster> {
    let mut grouped: BTreeMap<(i32, i32), Vec<ChronicleEvent>> = BTreeMap::new();
    let bucket_size = config::CHRONICLE_SOCIAL_BUCKET_SIZE.max(1);

    for event in events {
        if event.cause != ChronicleEventCause::Social {
            continue;
        }
        let key = (
            event.tile_x.div_euclid(bucket_size),
            event.tile_y.div_euclid(bucket_size),
        );
        grouped
            .entry(key)
            .or_default()
            .push(event.clone());
    }

    let mut clusters: Vec<ChronicleCluster> = Vec::new();
    for mut bucket_events in grouped.into_values() {
        bucket_events.sort_by_key(|event| event.tick);
        for event in bucket_events {
            if let Some(last_cluster) = clusters.last_mut() {
                if chronicle_events_can_cluster(last_cluster.events.last(), &event) {
                    last_cluster.push(event);
                    continue;
                }
            }
            clusters.push(ChronicleCluster {
                start_tick: event.tick,
                end_tick: event.tick,
                entity_id: None,
                events: vec![event],
            });
        }
    }

    clusters
        .into_iter()
        .filter(|cluster| {
            cluster
                .events
                .iter()
                .map(|event| event.entity_id)
                .collect::<BTreeSet<_>>()
                .len()
                >= config::CHRONICLE_SOCIAL_GROUP_SIZE_THRESHOLD
        })
        .collect()
}

fn chronicle_events_can_cluster(
    previous: Option<&ChronicleEvent>,
    next: &ChronicleEvent,
) -> bool {
    let Some(previous) = previous else {
        return false;
    };
    next.tick.saturating_sub(previous.tick) <= config::CHRONICLE_CLUSTER_WINDOW_TICKS
        && previous.cause == next.cause
        && previous.event_type == next.event_type
}

fn cluster_to_timeline_summary(
    world: &World,
    cluster: &ChronicleCluster,
) -> Option<ChronicleSummary> {
    let dominant = dominant_event(cluster.events.as_slice())?;
    let score = cluster_significance(cluster.events.as_slice());
    if score < config::CHRONICLE_SUMMARY_SIGNIFICANCE_THRESHOLD {
        log::debug!(
            "[Chronicle] cluster_rejected entity={} cause={} score={:.2}",
            cluster
                .entity_id
                .map(|entity_id| entity_id.0.to_string())
                .unwrap_or_else(|| "group".to_string()),
            dominant.cause.id(),
            score
        );
        return None;
    }

    let tile = cluster.events.last().map(|event| (event.tile_x, event.tile_y))?;
    let agent_label = cluster
        .entity_id
        .map(|entity_id| chronicle_entity_label(world, entity_id))
        .unwrap_or_default();
    let (title_key, description_key) =
        chronicle_template_keys(dominant.event_type, dominant.cause, false);
    let mut params = BTreeMap::new();
    if !agent_label.is_empty() {
        params.insert("agent".to_string(), agent_label);
    }
    params.insert("count".to_string(), cluster.events.len().to_string());

    log::debug!(
        "[Chronicle] summary_generated entity={} cause={} score={:.2}",
        cluster
            .entity_id
            .map(|entity_id| entity_id.0.to_string())
            .unwrap_or_else(|| "group".to_string()),
        dominant.cause.id(),
        score
    );

    Some(ChronicleSummary {
        start_tick: cluster.start_tick,
        end_tick: cluster.end_tick,
        entity_id: cluster.entity_id,
        event_type: dominant.event_type,
        cause: dominant.cause,
        title: title_key.to_string(),
        description: description_key.to_string(),
        params,
        tile_x: tile.0,
        tile_y: tile.1,
        significance: score,
    })
}

fn summary_to_timeline_summary(
    _world: &World,
    cluster: &ChronicleCluster,
) -> Option<ChronicleSummary> {
    let dominant = dominant_event(cluster.events.as_slice())?;
    let unique_entities = cluster
        .events
        .iter()
        .map(|event| event.entity_id)
        .collect::<BTreeSet<_>>();
    let score = social_cluster_significance(cluster.events.as_slice(), unique_entities.len());
    if score < config::CHRONICLE_SUMMARY_SIGNIFICANCE_THRESHOLD {
        log::debug!(
            "[Chronicle] cluster_rejected entity=group cause={} score={:.2}",
            dominant.cause.id(),
            score
        );
        return None;
    }

    let tile = cluster.events.last().map(|event| (event.tile_x, event.tile_y))?;
    let (title_key, description_key) =
        chronicle_template_keys(dominant.event_type, dominant.cause, true);
    let mut params = BTreeMap::new();
    params.insert("count".to_string(), unique_entities.len().to_string());

    log::debug!(
        "[Chronicle] cluster_created entity=group cause={} size={} score={:.2}",
        dominant.cause.id(),
        unique_entities.len(),
        score
    );

    Some(ChronicleSummary {
        start_tick: cluster.start_tick,
        end_tick: cluster.end_tick,
        entity_id: None,
        event_type: ChronicleEventType::GatheringFormation,
        cause: ChronicleEventCause::Social,
        title: title_key.to_string(),
        description: description_key.to_string(),
        params,
        tile_x: tile.0,
        tile_y: tile.1,
        significance: score,
    })
}

fn dominant_event(events: &[ChronicleEvent]) -> Option<&ChronicleEvent> {
    events.iter().max_by(|left, right| {
        left.magnitude
            .significance
            .partial_cmp(&right.magnitude.significance)
            .unwrap_or(Ordering::Equal)
    })
}

fn cluster_significance(events: &[ChronicleEvent]) -> f64 {
    let base = events
        .iter()
        .map(|event| event.magnitude.significance * 2.0)
        .sum::<f64>();
    base + chronicle_cause_bonus(dominant_event(events).map(|event| event.cause))
}

fn social_cluster_significance(events: &[ChronicleEvent], unique_entities: usize) -> f64 {
    cluster_significance(events) + unique_entities as f64
}

fn chronicle_cause_bonus(cause: Option<ChronicleEventCause>) -> f64 {
    match cause.unwrap_or(ChronicleEventCause::Unknown) {
        ChronicleEventCause::Danger => config::CHRONICLE_DANGER_BONUS,
        ChronicleEventCause::Food => config::CHRONICLE_FOOD_BONUS,
        ChronicleEventCause::Warmth => config::CHRONICLE_WARMTH_BONUS,
        ChronicleEventCause::Social => config::CHRONICLE_SOCIAL_BONUS,
        _ => 0.0,
    }
}

fn chronicle_entity_label(world: &World, entity_id: EntityId) -> String {
    let Some(entity) = world
        .query::<&Identity>()
        .iter()
        .find_map(|(entity, _)| (entity.id() as u64 == entity_id.0).then_some(entity))
    else {
        return format!("#{}", entity_id.0);
    };
    world.get::<&Identity>(entity)
        .map(|identity| identity.name.clone())
        .unwrap_or_else(|_| format!("#{}", entity_id.0))
}

fn chronicle_template_keys(
    event_type: ChronicleEventType,
    cause: ChronicleEventCause,
    is_group: bool,
) -> (&'static str, &'static str) {
    if is_group && cause == ChronicleEventCause::Social {
        return (
            "CHRONICLE_TITLE_SOCIAL_GATHERING",
            "CHRONICLE_SUMMARY_SOCIAL_GATHERING",
        );
    }
    match (event_type, cause) {
        (ChronicleEventType::InfluenceAvoidance, ChronicleEventCause::Danger) => (
            "CHRONICLE_TITLE_ESCAPE_DANGER",
            "CHRONICLE_SUMMARY_ESCAPE_DANGER",
        ),
        (ChronicleEventType::ShelterSeeking, ChronicleEventCause::Warmth) => (
            "CHRONICLE_TITLE_SHELTER_SEEKING",
            "CHRONICLE_SUMMARY_SHELTER_SEEKING",
        ),
        (ChronicleEventType::GatheringFormation, ChronicleEventCause::Social) => (
            "CHRONICLE_TITLE_SOCIAL_ATTRACTION",
            "CHRONICLE_SUMMARY_SOCIAL_ATTRACTION",
        ),
        _ => (
            "CHRONICLE_TITLE_FOOD_ATTRACTION",
            "CHRONICLE_SUMMARY_FOOD_ATTRACTION",
        ),
    }
}

fn append_chronicle_summary(resources: &mut SimResources, summary: ChronicleSummary) {
    if resources.chronicle_timeline.append_summary(summary) {
        log::debug!("[Chronicle] timeline_pruned");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::config::GameConfig;
    use sim_core::{GameCalendar, WorldMap};

    fn make_resources() -> SimResources {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(32, 32, 77);
        SimResources::new(calendar, map, 77)
    }

    fn make_entity(world: &mut World, name: &str) -> EntityId {
        let entity = world.spawn((Identity {
            name: name.to_string(),
            ..Identity::default()
        },));
        EntityId(entity.id() as u64)
    }

    fn make_event(
        tick: u64,
        entity_id: EntityId,
        event_type: ChronicleEventType,
        cause: ChronicleEventCause,
        significance: f64,
        tile_x: i32,
        tile_y: i32,
    ) -> ChronicleEvent {
        ChronicleEvent {
            tick,
            entity_id,
            event_type,
            cause,
            magnitude: sim_engine::ChronicleEventMagnitude {
                influence: significance,
                steering: significance,
                significance,
            },
            tile_x,
            tile_y,
            summary_key: "CAUSE_TEST".to_string(),
            effect_key: "steering_velocity".to_string(),
        }
    }

    #[test]
    fn low_significance_cluster_is_rejected() {
        let mut world = World::new();
        let entity_id = make_entity(&mut world, "Ari");
        let cluster = ChronicleCluster::new(make_event(
            10,
            entity_id,
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            0.20,
            2,
            3,
        ));

        assert!(cluster_to_timeline_summary(&world, &cluster).is_none());
    }

    #[test]
    fn danger_cluster_generates_escape_summary() {
        let mut world = World::new();
        let entity_id = make_entity(&mut world, "Mina");
        let mut cluster = ChronicleCluster::new(make_event(
            10,
            entity_id,
            ChronicleEventType::InfluenceAvoidance,
            ChronicleEventCause::Danger,
            0.65,
            5,
            7,
        ));
        cluster.push(make_event(
            20,
            entity_id,
            ChronicleEventType::InfluenceAvoidance,
            ChronicleEventCause::Danger,
            0.40,
            6,
            7,
        ));

        let summary = cluster_to_timeline_summary(&world, &cluster).expect("danger summary");
        assert_eq!(summary.cause, ChronicleEventCause::Danger);
        assert_eq!(summary.description, "CHRONICLE_SUMMARY_ESCAPE_DANGER");
        assert_eq!(summary.params.get("agent"), Some(&"Mina".to_string()));
    }

    #[test]
    fn social_group_cluster_requires_multiple_entities() {
        let mut world = World::new();
        let entity_a = make_entity(&mut world, "A");
        let entity_b = make_entity(&mut world, "B");
        let entity_c = make_entity(&mut world, "C");
        let events = vec![
            make_event(
                200,
                entity_a,
                ChronicleEventType::GatheringFormation,
                ChronicleEventCause::Social,
                0.50,
                10,
                10,
            ),
            make_event(
                205,
                entity_b,
                ChronicleEventType::GatheringFormation,
                ChronicleEventCause::Social,
                0.45,
                11,
                10,
            ),
            make_event(
                210,
                entity_c,
                ChronicleEventType::GatheringFormation,
                ChronicleEventCause::Social,
                0.48,
                10,
                11,
            ),
        ];

        let clusters = build_social_gathering_summaries(&events);
        assert_eq!(clusters.len(), 1);
        let summary = summary_to_timeline_summary(&world, &clusters[0]).expect("group summary");
        assert_eq!(summary.entity_id, None);
        assert_eq!(summary.description, "CHRONICLE_SUMMARY_SOCIAL_GATHERING");
        assert_eq!(summary.params.get("count"), Some(&"3".to_string()));
    }

    #[test]
    fn summarization_appends_recent_timeline_entries() {
        let mut world = World::new();
        let entity_id = make_entity(&mut world, "Daro");
        let mut resources = make_resources();
        resources.chronicle_log.append_event(make_event(
            210,
            entity_id,
            ChronicleEventType::InfluenceAvoidance,
            ChronicleEventCause::Danger,
            0.70,
            3,
            4,
        ));

        summarize_recent_chronicle_events(&world, &mut resources, 0);

        assert_eq!(resources.chronicle_timeline.len(), 1);
        let summary = resources
            .chronicle_timeline
            .recent_summaries(1)
            .first()
            .copied()
            .expect("one summary");
        assert_eq!(summary.cause, ChronicleEventCause::Danger);
    }
}
