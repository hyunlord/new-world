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
    ChronicleCapsule, ChronicleCluster, ChronicleDossierStub, ChronicleEntryLite,
    ChronicleEntityRefState, ChronicleEntryStatus, ChronicleEvent, ChronicleEventCause,
    ChronicleEventType, ChronicleHeadline, ChronicleLocationRefLite, ChronicleQueueBucket,
    ChronicleQueueTransition, ChronicleSignificanceCategory, ChronicleSignificanceMeta,
    ChronicleSubjectRefLite, SimResources, SimSystem,
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
/// This performs active writes on `SimResources.stat_threshold_flags` only.
/// Action decisions are owned by BehaviorSystem (priority 20) and completed
/// by MovementRuntimeSystem (priority 30). This system must NOT override
/// `Behavior.current_action` — doing so at priority 12 aborts ongoing Rest/Forage
/// actions before MovementSystem can apply the completion bonus (+0.70 energy),
/// creating a permanent energy-sink that keeps all agents at near-zero energy.
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

        let mut query = world.query::<(&Age, &Needs, &Stress)>();
        for (entity, (age, needs, stress)) in &mut query {
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
            // Action override removed: BehaviorSystem (priority 20) and MovementRuntimeSystem
            // (priority 30) own action lifecycle. Overriding here (priority 12) aborted ongoing
            // Rest before the +0.70 completion bonus could be applied, keeping all agents
            // energy-starved indefinitely. Threshold flags above are still emitted for UI/other
            // systems to read via resources.stat_threshold_flags.
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
            summarize_recent_chronicle_events(_world, resources, self.last_summarized_tick, tick);
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
    current_tick: u64,
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
            consumed_social_events.insert((
                event.tick,
                event.entity_id,
                event.tile_x,
                event.tile_y,
            ));
        }
        if let Some(entry) = summary_to_timeline_entry(world, resources, &summary) {
            append_chronicle_entry(resources, entry, summary.end_tick);
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
            if let Some(entry) = cluster_to_timeline_entry(world, resources, &cluster) {
                append_chronicle_entry(resources, entry, cluster.end_tick);
            }
        }
    }

    if let Some(result) = resources
        .chronicle_timeline
        .promote_background_if_starved(current_tick)
    {
        log::debug!(
            "[Chronicle] summary_surfaced cause=background category=notable promoted_background=true pruned={} displaced_visible={}",
            result.pruned,
            result.displaced_visible
        );
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
        grouped.entry(key).or_default().push(event.clone());
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

fn chronicle_events_can_cluster(previous: Option<&ChronicleEvent>, next: &ChronicleEvent) -> bool {
    let Some(previous) = previous else {
        return false;
    };
    if previous.event_type == ChronicleEventType::BandLifecycle
        || next.event_type == ChronicleEventType::BandLifecycle
    {
        return false;
    }
    next.tick.saturating_sub(previous.tick) <= config::CHRONICLE_CLUSTER_WINDOW_TICKS
        && previous.cause == next.cause
        && previous.event_type == next.event_type
}

fn cluster_to_timeline_entry(
    world: &World,
    resources: &mut SimResources,
    cluster: &ChronicleCluster,
) -> Option<ChronicleEntryLite> {
    let dominant = dominant_event(cluster.events.as_slice())?;
    if dominant.event_type == ChronicleEventType::BandLifecycle
        && dominant.cause == ChronicleEventCause::SocialGroup
    {
        return band_lifecycle_to_timeline_entry(world, resources, cluster, dominant);
    }
    let base_score = cluster_base_score(cluster.events.as_slice());
    let cause_bonus = chronicle_cause_bonus(Some(dominant.cause));
    let raw_score = base_score + cause_bonus;
    let score = adjusted_cluster_significance(
        resources,
        dominant.event_type,
        dominant.cause,
        cluster.end_tick,
        raw_score,
    );
    let repeat_penalty = (raw_score - score).max(0.0);
    let category = chronicle_category(score);
    if category < ChronicleSignificanceCategory::Notable {
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

    let tile = cluster
        .events
        .last()
        .map(|event| (event.tile_x, event.tile_y))?;
    let agent_label = cluster
        .entity_id
        .map(|entity_id| chronicle_entity_label(world, entity_id))
        .unwrap_or_default();
    let (headline_key, capsule_key, dossier_stub_key) =
        chronicle_template_keys(dominant.event_type, dominant.cause, false);
    let mut params = BTreeMap::new();
    if !agent_label.is_empty() {
        params.insert("agent".to_string(), agent_label.clone());
    }
    params.insert("count".to_string(), cluster.events.len().to_string());
    params.insert("x".to_string(), tile.0.to_string());
    params.insert("y".to_string(), tile.1.to_string());

    log::debug!(
        "[Chronicle] summary_generated entity={} cause={} score={:.2}",
        cluster
            .entity_id
            .map(|entity_id| entity_id.0.to_string())
            .unwrap_or_else(|| "group".to_string()),
        dominant.cause.id(),
        score
    );

    Some(ChronicleEntryLite {
        entry_id: resources.chronicle_timeline.allocate_entry_id(),
        start_tick: cluster.start_tick,
        end_tick: cluster.end_tick,
        event_family: chronicle_event_family(dominant.event_type, dominant.cause, false),
        event_type: dominant.event_type,
        cause: dominant.cause,
        headline: ChronicleHeadline {
            locale_key: headline_key.to_string(),
            params: params.clone(),
        },
        capsule: ChronicleCapsule {
            locale_key: capsule_key.to_string(),
            params: params.clone(),
        },
        dossier_stub: ChronicleDossierStub {
            locale_key: dossier_stub_key.to_string(),
            params,
            detail_tags: chronicle_dossier_stub_tags(dominant.event_type, dominant.cause, false),
        },
        entity_ref: ChronicleSubjectRefLite {
            entity_id: cluster.entity_id,
            display_name: (!agent_label.is_empty()).then_some(agent_label),
            ref_state: cluster
                .entity_id
                .map(|_| ChronicleEntityRefState::Alive)
                .unwrap_or(ChronicleEntityRefState::Unknown),
        },
        location_ref: ChronicleLocationRefLite {
            tile_x: tile.0,
            tile_y: tile.1,
            region_label: None,
        },
        significance: score,
        significance_category: category,
        significance_meta: ChronicleSignificanceMeta {
            base_score,
            cause_bonus,
            group_bonus: 0.0,
            repeat_penalty,
            final_score: score,
            reason_tags: vec![format!("cause:{}", dominant.cause.id())],
        },
        queue_bucket: ChronicleQueueBucket::Dropped,
        status: ChronicleEntryStatus::Pending,
        surfaced_tick: None,
        displacement_reason: None,
        queue_transitions: Vec::new(),
        thread_id: None,
    })
}

fn band_lifecycle_to_timeline_entry(
    world: &World,
    resources: &mut SimResources,
    cluster: &ChronicleCluster,
    dominant: &ChronicleEvent,
) -> Option<ChronicleEntryLite> {
    let base_score = cluster_base_score(cluster.events.as_slice());
    let cause_bonus = chronicle_cause_bonus(Some(dominant.cause));
    let raw_score = base_score + cause_bonus;
    let score = adjusted_cluster_significance(
        resources,
        dominant.event_type,
        dominant.cause,
        cluster.end_tick,
        raw_score,
    );
    let repeat_penalty = (raw_score - score).max(0.0);
    let category = chronicle_category(score);
    if category < ChronicleSignificanceCategory::Notable {
        return None;
    }

    let tile = cluster
        .events
        .last()
        .map(|event| (event.tile_x, event.tile_y))?;
    let agent_label = cluster
        .entity_id
        .map(|entity_id| chronicle_entity_label(world, entity_id))
        .unwrap_or_default();
    let mut params = dominant.summary_params.clone();
    params
        .entry("x".to_string())
        .or_insert_with(|| tile.0.to_string());
    params
        .entry("y".to_string())
        .or_insert_with(|| tile.1.to_string());
    if !agent_label.is_empty() {
        params
            .entry("agent".to_string())
            .or_insert_with(|| agent_label.clone());
    }

    Some(ChronicleEntryLite {
        entry_id: resources.chronicle_timeline.allocate_entry_id(),
        start_tick: cluster.start_tick,
        end_tick: cluster.end_tick,
        event_family: band_lifecycle_event_family(dominant.summary_key.as_str()),
        event_type: dominant.event_type,
        cause: dominant.cause,
        headline: ChronicleHeadline {
            locale_key: dominant.summary_key.clone(),
            params: params.clone(),
        },
        capsule: ChronicleCapsule {
            locale_key: dominant.summary_key.clone(),
            params: params.clone(),
        },
        dossier_stub: ChronicleDossierStub {
            locale_key: dominant.summary_key.clone(),
            params,
            detail_tags: band_lifecycle_dossier_stub_tags(dominant.summary_key.as_str()),
        },
        entity_ref: ChronicleSubjectRefLite {
            entity_id: cluster.entity_id,
            display_name: (!agent_label.is_empty()).then_some(agent_label),
            ref_state: cluster
                .entity_id
                .map(|_| ChronicleEntityRefState::Alive)
                .unwrap_or(ChronicleEntityRefState::Unknown),
        },
        location_ref: ChronicleLocationRefLite {
            tile_x: tile.0,
            tile_y: tile.1,
            region_label: None,
        },
        significance: score,
        significance_category: category,
        significance_meta: ChronicleSignificanceMeta {
            base_score,
            cause_bonus,
            group_bonus: 0.0,
            repeat_penalty,
            final_score: score,
            reason_tags: vec![
                format!("cause:{}", dominant.cause.id()),
                format!("summary:{}", dominant.summary_key),
            ],
        },
        queue_bucket: ChronicleQueueBucket::Dropped,
        status: ChronicleEntryStatus::Pending,
        surfaced_tick: None,
        displacement_reason: None,
        queue_transitions: Vec::new(),
        thread_id: None,
    })
}

fn summary_to_timeline_entry(
    _world: &World,
    resources: &mut SimResources,
    cluster: &ChronicleCluster,
) -> Option<ChronicleEntryLite> {
    let dominant = dominant_event(cluster.events.as_slice())?;
    let unique_entities = cluster
        .events
        .iter()
        .map(|event| event.entity_id)
        .collect::<BTreeSet<_>>();
    let base_score = cluster_base_score(cluster.events.as_slice());
    let cause_bonus = chronicle_cause_bonus(Some(ChronicleEventCause::Social));
    let group_bonus = unique_entities.len() as f64;
    let raw_score = base_score + cause_bonus + group_bonus;
    let score = adjusted_cluster_significance(
        resources,
        ChronicleEventType::GatheringFormation,
        ChronicleEventCause::Social,
        cluster.end_tick,
        raw_score,
    );
    let repeat_penalty = (raw_score - score).max(0.0);
    let category = chronicle_category(score);
    if category < ChronicleSignificanceCategory::Notable {
        log::debug!(
            "[Chronicle] cluster_rejected entity=group cause={} score={:.2}",
            dominant.cause.id(),
            score
        );
        return None;
    }

    let tile = cluster
        .events
        .last()
        .map(|event| (event.tile_x, event.tile_y))?;
    let (headline_key, capsule_key, dossier_stub_key) =
        chronicle_template_keys(dominant.event_type, dominant.cause, true);
    let mut params = BTreeMap::new();
    params.insert("count".to_string(), unique_entities.len().to_string());
    params.insert("x".to_string(), tile.0.to_string());
    params.insert("y".to_string(), tile.1.to_string());

    log::debug!(
        "[Chronicle] cluster_created entity=group cause={} size={} score={:.2}",
        dominant.cause.id(),
        unique_entities.len(),
        score
    );

    Some(ChronicleEntryLite {
        entry_id: resources.chronicle_timeline.allocate_entry_id(),
        start_tick: cluster.start_tick,
        end_tick: cluster.end_tick,
        event_family: chronicle_event_family(dominant.event_type, dominant.cause, true),
        event_type: ChronicleEventType::GatheringFormation,
        cause: ChronicleEventCause::Social,
        headline: ChronicleHeadline {
            locale_key: headline_key.to_string(),
            params: params.clone(),
        },
        capsule: ChronicleCapsule {
            locale_key: capsule_key.to_string(),
            params: params.clone(),
        },
        dossier_stub: ChronicleDossierStub {
            locale_key: dossier_stub_key.to_string(),
            params,
            detail_tags: chronicle_dossier_stub_tags(dominant.event_type, dominant.cause, true),
        },
        entity_ref: ChronicleSubjectRefLite {
            entity_id: None,
            display_name: None,
            ref_state: ChronicleEntityRefState::Unknown,
        },
        location_ref: ChronicleLocationRefLite {
            tile_x: tile.0,
            tile_y: tile.1,
            region_label: None,
        },
        significance: score,
        significance_category: category,
        significance_meta: ChronicleSignificanceMeta {
            base_score,
            cause_bonus,
            group_bonus,
            repeat_penalty,
            final_score: score,
            reason_tags: vec![
                format!("cause:{}", ChronicleEventCause::Social.id()),
                format!("group_size:{}", unique_entities.len()),
            ],
        },
        queue_bucket: ChronicleQueueBucket::Dropped,
        status: ChronicleEntryStatus::Pending,
        surfaced_tick: None,
        displacement_reason: None,
        queue_transitions: Vec::new(),
        thread_id: None,
    })
}

fn cluster_base_score(events: &[ChronicleEvent]) -> f64 {
    events
        .iter()
        .map(|event| event.magnitude.significance * 2.0)
        .sum::<f64>()
}

fn dominant_event(events: &[ChronicleEvent]) -> Option<&ChronicleEvent> {
    events.iter().max_by(|left, right| {
        left.magnitude
            .significance
            .partial_cmp(&right.magnitude.significance)
            .unwrap_or(Ordering::Equal)
    })
}

fn adjusted_cluster_significance(
    resources: &SimResources,
    event_type: ChronicleEventType,
    cause: ChronicleEventCause,
    tick: u64,
    raw_score: f64,
) -> f64 {
    let repeat_count = resources.chronicle_timeline.recent_family_count(
        event_type,
        cause,
        tick.saturating_sub(config::CHRONICLE_REPEAT_SUPPRESSION_WINDOW_TICKS),
    );
    (raw_score - repeat_count as f64 * config::CHRONICLE_REPEAT_SUPPRESSION_STEP).max(0.0)
}

fn chronicle_category(score: f64) -> ChronicleSignificanceCategory {
    if score >= config::CHRONICLE_SUMMARY_CRITICAL_THRESHOLD {
        return ChronicleSignificanceCategory::Critical;
    }
    if score >= config::CHRONICLE_SUMMARY_SIGNIFICANCE_THRESHOLD {
        return ChronicleSignificanceCategory::Major;
    }
    if score >= config::CHRONICLE_SUMMARY_NOTABLE_THRESHOLD {
        return ChronicleSignificanceCategory::Notable;
    }
    if score > 0.0 {
        return ChronicleSignificanceCategory::Minor;
    }
    ChronicleSignificanceCategory::Ignore
}

fn chronicle_cause_bonus(cause: Option<ChronicleEventCause>) -> f64 {
    match cause.unwrap_or(ChronicleEventCause::Unknown) {
        ChronicleEventCause::Danger => config::CHRONICLE_DANGER_BONUS,
        ChronicleEventCause::Food => config::CHRONICLE_FOOD_BONUS,
        ChronicleEventCause::Warmth => config::CHRONICLE_WARMTH_BONUS,
        ChronicleEventCause::Social => config::CHRONICLE_SOCIAL_BONUS,
        ChronicleEventCause::SocialGroup => config::CHRONICLE_SOCIAL_BONUS,
        _ => 0.0,
    }
}

fn band_lifecycle_dossier_stub_tags(summary_key: &str) -> Vec<String> {
    let mut tags = vec!["band".to_string(), "social".to_string()];
    let kind = match summary_key {
        "CHRONICLE_BAND_FORMED" => "formed",
        "CHRONICLE_BAND_PROMOTED" => "promoted",
        "CHRONICLE_BAND_SPLIT" => "split",
        "CHRONICLE_BAND_DISSOLVED" => "dissolved",
        "CHRONICLE_BAND_LEADER" => "leader",
        "CHRONICLE_LONER_JOINED" => "recruitment",
        _ => "lifecycle",
    };
    tags.push(kind.to_string());
    tags
}

fn band_lifecycle_event_family(summary_key: &str) -> String {
    match summary_key {
        "CHRONICLE_BAND_FORMED" => "band.lifecycle.formed".to_string(),
        "CHRONICLE_BAND_PROMOTED" => "band.lifecycle.promoted".to_string(),
        "CHRONICLE_BAND_SPLIT" => "band.lifecycle.split".to_string(),
        "CHRONICLE_BAND_DISSOLVED" => "band.lifecycle.dissolved".to_string(),
        "CHRONICLE_BAND_LEADER" => "band.lifecycle.leader".to_string(),
        "CHRONICLE_LONER_JOINED" => "band.lifecycle.joined".to_string(),
        _ => "band.lifecycle.unknown".to_string(),
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
    world
        .get::<&Identity>(entity)
        .map(|identity| identity.name.clone())
        .unwrap_or_else(|_| format!("#{}", entity_id.0))
}

fn chronicle_template_keys(
    event_type: ChronicleEventType,
    cause: ChronicleEventCause,
    is_group: bool,
) -> (&'static str, &'static str, &'static str) {
    if is_group && cause == ChronicleEventCause::Social {
        return (
            "CHRONICLE_TITLE_SOCIAL_GATHERING",
            "CHRONICLE_SUMMARY_SOCIAL_GATHERING",
            "CHRONICLE_DOSSIER_STUB_SOCIAL_GATHERING",
        );
    }
    match (event_type, cause) {
        (ChronicleEventType::InfluenceAvoidance, ChronicleEventCause::Danger) => (
            "CHRONICLE_TITLE_ESCAPE_DANGER",
            "CHRONICLE_SUMMARY_ESCAPE_DANGER",
            "CHRONICLE_DOSSIER_STUB_ESCAPE_DANGER",
        ),
        (ChronicleEventType::ShelterSeeking, ChronicleEventCause::Warmth) => (
            "CHRONICLE_TITLE_SHELTER_SEEKING",
            "CHRONICLE_SUMMARY_SHELTER_SEEKING",
            "CHRONICLE_DOSSIER_STUB_SHELTER_SEEKING",
        ),
        (ChronicleEventType::GatheringFormation, ChronicleEventCause::Social) => (
            "CHRONICLE_TITLE_SOCIAL_ATTRACTION",
            "CHRONICLE_SUMMARY_SOCIAL_ATTRACTION",
            "CHRONICLE_DOSSIER_STUB_SOCIAL_ATTRACTION",
        ),
        _ => (
            "CHRONICLE_TITLE_FOOD_ATTRACTION",
            "CHRONICLE_SUMMARY_FOOD_ATTRACTION",
            "CHRONICLE_DOSSIER_STUB_FOOD_ATTRACTION",
        ),
    }
}

fn chronicle_dossier_stub_tags(
    event_type: ChronicleEventType,
    cause: ChronicleEventCause,
    is_group: bool,
) -> Vec<String> {
    if is_group && cause == ChronicleEventCause::Social {
        return vec![
            "social".to_string(),
            "gathering".to_string(),
            "group".to_string(),
        ];
    }
    match (event_type, cause) {
        (ChronicleEventType::InfluenceAvoidance, ChronicleEventCause::Danger) => vec![
            "danger".to_string(),
            "avoidance".to_string(),
            "survival".to_string(),
        ],
        (ChronicleEventType::ShelterSeeking, ChronicleEventCause::Warmth) => vec![
            "warmth".to_string(),
            "shelter".to_string(),
            "survival".to_string(),
        ],
        (ChronicleEventType::GatheringFormation, ChronicleEventCause::Social) => vec![
            "social".to_string(),
            "attraction".to_string(),
            "group".to_string(),
        ],
        _ => vec![
            "food".to_string(),
            "attraction".to_string(),
            "survival".to_string(),
        ],
    }
}

fn chronicle_event_family(
    event_type: ChronicleEventType,
    cause: ChronicleEventCause,
    is_group: bool,
) -> String {
    if is_group && cause == ChronicleEventCause::Social {
        return "social.group_gathering".to_string();
    }
    match (event_type, cause) {
        (ChronicleEventType::InfluenceAttraction, ChronicleEventCause::Food)
        | (ChronicleEventType::ResourceDiscovery, ChronicleEventCause::Food) => {
            "influence.food_attraction".to_string()
        }
        (ChronicleEventType::InfluenceAvoidance, ChronicleEventCause::Danger) => {
            "influence.danger_avoidance".to_string()
        }
        (ChronicleEventType::ShelterSeeking, ChronicleEventCause::Warmth)
        | (ChronicleEventType::InfluenceAttraction, ChronicleEventCause::Warmth) => {
            "influence.shelter_seeking".to_string()
        }
        (ChronicleEventType::GatheringFormation, ChronicleEventCause::Social) => {
            "influence.social_attraction".to_string()
        }
        _ => format!("chronicle.{:?}.{}", event_type, cause.id()).to_lowercase(),
    }
}

fn append_chronicle_entry(
    resources: &mut SimResources,
    entry: ChronicleEntryLite,
    surfaced_tick: u64,
) {
    let cause_id = entry.cause.id();
    let category_id = entry.significance_category.id();
    let result = resources
        .chronicle_timeline
        .route_entry(entry, surfaced_tick);
    match result.queue {
        ChronicleQueueBucket::Visible => log::debug!(
            "[Chronicle] summary_surfaced cause={} category={} promoted_background={} pruned={} displaced_visible={}",
            cause_id,
            category_id,
            result.promoted_background,
            result.pruned,
            result.displaced_visible
        ),
        ChronicleQueueBucket::Background => log::debug!(
            "[Chronicle] summary_backgrounded cause={} category={} pruned={}",
            cause_id,
            category_id,
            result.pruned
        ),
        ChronicleQueueBucket::Recall => log::debug!(
            "[Chronicle] summary_recalled cause={} category={} pruned={}",
            cause_id,
            category_id,
            result.pruned
        ),
        ChronicleQueueBucket::Dropped => log::debug!(
            "[Chronicle] summary_suppressed cause={} category={}",
            cause_id,
            category_id
        ),
    }
    if result.pruned {
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
            summary_params: BTreeMap::new(),
            effect_key: "steering_velocity".to_string(),
        }
    }

    fn make_band_event(
        tick: u64,
        entity_id: EntityId,
        summary_key: &str,
        params: BTreeMap<String, String>,
    ) -> ChronicleEvent {
        ChronicleEvent {
            tick,
            entity_id,
            event_type: ChronicleEventType::BandLifecycle,
            cause: ChronicleEventCause::SocialGroup,
            magnitude: sim_engine::ChronicleEventMagnitude {
                influence: 3.0,
                steering: 0.0,
                significance: 3.0,
            },
            tile_x: 8,
            tile_y: 13,
            summary_key: summary_key.to_string(),
            summary_params: params,
            effect_key: "band:test".to_string(),
        }
    }

    #[test]
    fn low_significance_cluster_is_rejected() {
        let mut world = World::new();
        let mut resources = make_resources();
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

        assert!(cluster_to_timeline_entry(&world, &mut resources, &cluster).is_none());
    }

    #[test]
    fn danger_cluster_generates_escape_summary() {
        let mut world = World::new();
        let mut resources = make_resources();
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

        let entry =
            cluster_to_timeline_entry(&world, &mut resources, &cluster).expect("danger entry");
        assert_eq!(entry.cause, ChronicleEventCause::Danger);
        assert_eq!(entry.capsule.locale_key, "CHRONICLE_SUMMARY_ESCAPE_DANGER");
        assert_eq!(
            entry.dossier_stub.locale_key,
            "CHRONICLE_DOSSIER_STUB_ESCAPE_DANGER"
        );
        assert_eq!(
            entry.capsule.params.get("agent"),
            Some(&"Mina".to_string())
        );
        assert_eq!(
            entry.significance_category,
            ChronicleSignificanceCategory::Major
        );
    }

    #[test]
    fn band_lifecycle_cluster_uses_raw_summary_key_and_params() {
        let mut world = World::new();
        let mut resources = make_resources();
        let entity_id = make_entity(&mut world, "Ari");
        let mut params = BTreeMap::new();
        params.insert("name".to_string(), "band_7".to_string());
        params.insert("leader".to_string(), "Ari".to_string());
        let cluster = ChronicleCluster::new(make_band_event(
            40,
            entity_id,
            "CHRONICLE_BAND_LEADER",
            params,
        ));

        let entry = cluster_to_timeline_entry(&world, &mut resources, &cluster)
            .expect("band lifecycle entry");

        assert_eq!(entry.headline.locale_key, "CHRONICLE_BAND_LEADER");
        assert_eq!(entry.capsule.locale_key, "CHRONICLE_BAND_LEADER");
        assert_eq!(
            entry.headline.params.get("name").map(String::as_str),
            Some("band_7")
        );
        assert_eq!(
            entry.headline.params.get("leader").map(String::as_str),
            Some("Ari")
        );
        assert_eq!(entry.cause, ChronicleEventCause::SocialGroup);
    }

    #[test]
    fn social_group_cluster_requires_multiple_entities() {
        let mut world = World::new();
        let mut resources = make_resources();
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
        let entry =
            summary_to_timeline_entry(&world, &mut resources, &clusters[0]).expect("group entry");
        assert_eq!(entry.entity_ref.entity_id, None);
        assert_eq!(
            entry.capsule.locale_key,
            "CHRONICLE_SUMMARY_SOCIAL_GATHERING"
        );
        assert_eq!(
            entry.dossier_stub.locale_key,
            "CHRONICLE_DOSSIER_STUB_SOCIAL_GATHERING"
        );
        assert_eq!(entry.capsule.params.get("count"), Some(&"3".to_string()));
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

        summarize_recent_chronicle_events(&world, &mut resources, 0, 210);

        assert_eq!(resources.chronicle_timeline.len(), 1);
        let entry = resources
            .chronicle_timeline
            .recent_entries(1)
            .first()
            .copied()
            .expect("one entry");
        assert_eq!(entry.cause, ChronicleEventCause::Danger);
    }

    #[test]
    fn repeat_penalty_downgrades_repeated_cluster_to_background_significance() {
        let mut world = World::new();
        let mut resources = make_resources();
        let entity_id = make_entity(&mut world, "Leto");
        let entry_id = resources.chronicle_timeline.allocate_entry_id();
        append_chronicle_entry(
            &mut resources,
            ChronicleEntryLite {
                entry_id,
                start_tick: 50,
                end_tick: 50,
                event_family: "influence.food_attraction".to_string(),
                event_type: ChronicleEventType::InfluenceAttraction,
                cause: ChronicleEventCause::Food,
                headline: ChronicleHeadline {
                    locale_key: "CHRONICLE_TITLE_FOOD_ATTRACTION".to_string(),
                    params: BTreeMap::new(),
                },
                capsule: ChronicleCapsule {
                    locale_key: "CHRONICLE_SUMMARY_FOOD_ATTRACTION".to_string(),
                    params: BTreeMap::new(),
                },
                dossier_stub: ChronicleDossierStub {
                    locale_key: "CHRONICLE_DOSSIER_STUB_FOOD_ATTRACTION".to_string(),
                    params: BTreeMap::new(),
                    detail_tags: vec!["food".to_string()],
                },
                entity_ref: ChronicleSubjectRefLite {
                    entity_id: Some(entity_id),
                    display_name: None,
                    ref_state: ChronicleEntityRefState::Alive,
                },
                location_ref: ChronicleLocationRefLite {
                    tile_x: 2,
                    tile_y: 2,
                    region_label: None,
                },
                significance: 7.0,
                significance_category: ChronicleSignificanceCategory::Major,
                significance_meta: ChronicleSignificanceMeta {
                    base_score: 7.0,
                    cause_bonus: 0.0,
                    group_bonus: 0.0,
                    repeat_penalty: 0.0,
                    final_score: 7.0,
                    reason_tags: vec!["seed".to_string()],
                },
                queue_bucket: ChronicleQueueBucket::Dropped,
                status: ChronicleEntryStatus::Pending,
                surfaced_tick: None,
                displacement_reason: None,
                queue_transitions: Vec::new(),
                thread_id: None,
            },
            50,
        );

        let cluster = ChronicleCluster {
            start_tick: 100,
            end_tick: 100,
            entity_id: Some(entity_id),
            events: vec![make_event(
                100,
                entity_id,
                ChronicleEventType::InfluenceAttraction,
                ChronicleEventCause::Food,
                2.0,
                3,
                3,
            )],
        };

        let entry = cluster_to_timeline_entry(&world, &mut resources, &cluster).expect("entry");
        assert_eq!(
            entry.significance_category,
            ChronicleSignificanceCategory::Notable
        );
    }

    #[test]
    fn anti_starvation_promotes_background_summary_to_visible_queue() {
        let mut resources = make_resources();
        let entry_id = resources.chronicle_timeline.allocate_entry_id();
        append_chronicle_entry(
            &mut resources,
            ChronicleEntryLite {
                entry_id,
                start_tick: 10,
                end_tick: 10,
                event_family: "social.group_gathering".to_string(),
                event_type: ChronicleEventType::GatheringFormation,
                cause: ChronicleEventCause::Social,
                headline: ChronicleHeadline {
                    locale_key: "CHRONICLE_TITLE_SOCIAL_GATHERING".to_string(),
                    params: BTreeMap::new(),
                },
                capsule: ChronicleCapsule {
                    locale_key: "CHRONICLE_SUMMARY_SOCIAL_GATHERING".to_string(),
                    params: BTreeMap::new(),
                },
                dossier_stub: ChronicleDossierStub {
                    locale_key: "CHRONICLE_DOSSIER_STUB_SOCIAL_GATHERING".to_string(),
                    params: BTreeMap::new(),
                    detail_tags: vec!["social".to_string()],
                },
                entity_ref: ChronicleSubjectRefLite {
                    entity_id: None,
                    display_name: None,
                    ref_state: ChronicleEntityRefState::Unknown,
                },
                location_ref: ChronicleLocationRefLite {
                    tile_x: 4,
                    tile_y: 5,
                    region_label: None,
                },
                significance: 5.0,
                significance_category: ChronicleSignificanceCategory::Notable,
                significance_meta: ChronicleSignificanceMeta {
                    base_score: 5.0,
                    cause_bonus: 0.0,
                    group_bonus: 0.0,
                    repeat_penalty: 0.0,
                    final_score: 5.0,
                    reason_tags: vec!["seed".to_string()],
                },
                queue_bucket: ChronicleQueueBucket::Dropped,
                status: ChronicleEntryStatus::Pending,
                surfaced_tick: None,
                displacement_reason: None,
                queue_transitions: Vec::new(),
                thread_id: None,
            },
            10,
        );

        let result = resources
            .chronicle_timeline
            .promote_background_if_starved(config::CHRONICLE_VISIBLE_STARVATION_TICKS + 1)
            .expect("background summary should be promoted");
        assert_eq!(result.queue, ChronicleQueueBucket::Visible);
        assert!(result.promoted_background);
        assert_eq!(resources.chronicle_timeline.visible_len(), 1);
        assert_eq!(resources.chronicle_timeline.background_len(), 0);
    }
}
