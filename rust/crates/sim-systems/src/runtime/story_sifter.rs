use std::collections::{HashMap, HashSet};

use hecs::World;
use sim_core::components::{Identity, Personality, Position, Social, Stress};
use sim_core::config;
use sim_core::{RelationType, StressState};
use sim_engine::{
    EventStore, NotificationTier, SimEvent, SimEventType, SimNotification, SimResources, SimSystem,
};

#[derive(Clone, Debug)]
struct EntityStoryContext {
    name: String,
    position_x: f64,
    position_y: f64,
    stress_state: Option<StressState>,
    personality_axes: [f64; 6],
    relations: Vec<(u32, f64, RelationType)>,
}

type SocialReversalPair<'a> = (Option<&'a SimEvent>, Option<&'a SimEvent>);

/// Narrative pattern matcher that turns persisted simulation events into UI notifications.
#[derive(Debug, Clone)]
pub struct StorySifterRuntimeSystem {
    priority: u32,
    tick_interval: u64,
    last_processed_tick: Option<u64>,
}

impl StorySifterRuntimeSystem {
    /// Creates a story-sifter runtime system with explicit priority and interval.
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
            last_processed_tick: None,
        }
    }
}

impl SimSystem for StorySifterRuntimeSystem {
    fn name(&self) -> &'static str {
        "story_sifter_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        let contexts = build_story_contexts(world);
        let since_tick = self
            .last_processed_tick
            .map(|processed_tick| processed_tick.saturating_add(1))
            .unwrap_or(0);
        let recent_events: Vec<SimEvent> = resources
            .event_store
            .since_tick(since_tick)
            .filter(|event| event.tick <= tick)
            .cloned()
            .collect();

        for event in &recent_events {
            if let Some(notification) =
                immediate_notification_for_event(event, &contexts, &resources.event_store, tick)
            {
                queue_notification(resources, notification, tick);
            }
        }

        for notification in sift_patterns(&resources.event_store, tick, &contexts) {
            queue_notification(resources, notification, tick);
        }

        self.last_processed_tick = Some(tick);
    }
}

fn build_story_contexts(world: &World) -> HashMap<u32, EntityStoryContext> {
    let mut contexts: HashMap<u32, EntityStoryContext> = HashMap::new();
    let mut query = world.query::<(
        Option<&Identity>,
        Option<&Position>,
        Option<&Stress>,
        Option<&Personality>,
        Option<&Social>,
    )>();
    for (entity, (identity_opt, position_opt, stress_opt, personality_opt, social_opt)) in
        &mut query
    {
        let raw_id = entity.id();
        let name = identity_opt
            .map(|identity| identity.name.clone())
            .unwrap_or_else(|| format!("Agent {}", raw_id));
        let (position_x, position_y) = position_opt
            .map(|position| (position.x, position.y))
            .unwrap_or((0.0, 0.0));
        let stress_state = stress_opt.map(|stress| stress.state);
        let personality_axes = personality_opt
            .map(|personality| personality.axes)
            .unwrap_or([0.5; 6]);
        let relations = social_opt
            .map(|social| {
                social
                    .edges
                    .iter()
                    .filter_map(|edge| {
                        u32::try_from(edge.target.0)
                            .ok()
                            .map(|target| (target, edge.affinity, edge.relation_type))
                    })
                    .collect()
            })
            .unwrap_or_default();
        contexts.insert(
            raw_id,
            EntityStoryContext {
                name,
                position_x,
                position_y,
                stress_state,
                personality_axes,
                relations,
            },
        );
    }
    contexts
}

fn queue_notification(
    resources: &mut SimResources,
    notification: SimNotification,
    current_tick: u64,
) {
    let history_cutoff =
        current_tick.saturating_sub(config::STORY_SIFTER_IMPORTANCE_LOOKBACK_TICKS);
    resources
        .notification_history
        .retain(|recent| recent.tick >= history_cutoff);

    if !should_notify(&notification, &resources.notification_history, current_tick) {
        return;
    }

    if notification.tier != NotificationTier::Ambient {
        if resources.pending_notifications.len() >= config::NOTIFICATION_PENDING_LIMIT {
            resources.pending_notifications.remove(0);
        }
        resources.pending_notifications.push(notification.clone());
    }

    resources.notification_history.push(notification);
}

fn should_notify(
    notification: &SimNotification,
    recent: &[SimNotification],
    current_tick: u64,
) -> bool {
    if notification.tier == NotificationTier::Ambient {
        return false;
    }
    !recent.iter().any(|recent_notification| {
        recent_notification.kind == notification.kind
            && recent_notification.primary_entity == notification.primary_entity
            && current_tick.saturating_sub(recent_notification.tick)
                < config::NOTIFICATION_COOLDOWN_TICKS
    })
}

fn sift_patterns(
    store: &EventStore,
    current_tick: u64,
    contexts: &HashMap<u32, EntityStoryContext>,
) -> Vec<SimNotification> {
    let mut notifications: Vec<SimNotification> = Vec::new();
    let window = current_tick.saturating_sub(config::STORY_SIFTER_LOOKBACK_TICKS);

    if let Some(notification) = sift_first_occurrence(store, window, contexts, current_tick) {
        notifications.push(notification);
    }
    if let Some(notification) = sift_rapid_escalation(store, contexts, current_tick) {
        notifications.push(notification);
    }
    if let Some(notification) = sift_social_reversal(store, window, contexts, current_tick) {
        notifications.push(notification);
    }
    if let Some(notification) = sift_simultaneous_crisis(store, current_tick, contexts) {
        notifications.push(notification);
    }
    if let Some(notification) = sift_deadline_approaching(store, window, contexts, current_tick) {
        notifications.push(notification);
    }
    if let Some(notification) = sift_relationship_triangle(contexts, current_tick) {
        notifications.push(notification);
    }
    if let Some(notification) = sift_personality_clash(store, window, contexts, current_tick) {
        notifications.push(notification);
    }
    if let Some(notification) = sift_recovery_arc(store, current_tick, contexts) {
        notifications.push(notification);
    }

    notifications
}

fn sift_first_occurrence(
    store: &EventStore,
    window_start: u64,
    contexts: &HashMap<u32, EntityStoryContext>,
    current_tick: u64,
) -> Option<SimNotification> {
    let candidates = [
        SimEventType::Birth,
        SimEventType::RelationshipFormed,
        SimEventType::Death,
        SimEventType::MentalBreakStart,
    ];
    // Pre-compute which event types have occurred before the window — O(n) once
    let mut seen_before_window: HashSet<SimEventType> = HashSet::new();
    for event in store.iter() {
        if event.tick >= window_start {
            break;
        }
        if candidates.contains(&event.event_type) {
            seen_before_window.insert(event.event_type.clone());
        }
    }
    for event in store.since_tick(window_start) {
        if !candidates.contains(&event.event_type) {
            continue;
        }
        // O(1) lookup instead of O(total_events) scan
        if seen_before_window.contains(&event.event_type) {
            continue;
        }
        let cause = if event.cause.is_empty() {
            label_for_event_type(&event.event_type).to_string()
        } else {
            event.cause.clone()
        };
        return Some(make_notification(
            event.tick,
            NotificationTier::Milestone,
            "first_occurrence",
            calculate_importance(&SimEventType::FirstOccurrence, store, current_tick),
            event.actor,
            event.target,
            "NOTIF_FIRST",
            format!("A first: {}.", cause),
            contexts,
        ));
    }
    None
}

fn sift_rapid_escalation(
    store: &EventStore,
    contexts: &HashMap<u32, EntityStoryContext>,
    current_tick: u64,
) -> Option<SimNotification> {
    let window_start = current_tick.saturating_sub(config::STORY_SIFTER_RAPID_ESCALATION_TICKS);
    let mut per_actor: HashMap<u32, (u64, u64, i32, i32)> = HashMap::new();
    for event in store.by_type(&SimEventType::StressEscalated, window_start) {
        let stage = event.value.round() as i32;
        let entry = per_actor
            .entry(event.actor)
            .or_insert((event.tick, event.tick, stage, stage));
        entry.0 = entry.0.min(event.tick);
        entry.1 = entry.1.max(event.tick);
        entry.2 = entry.2.min(stage);
        entry.3 = entry.3.max(stage);
    }
    for (actor, (first_tick, last_tick, min_stage, max_stage)) in per_actor {
        if last_tick.saturating_sub(first_tick) > config::STORY_SIFTER_RAPID_ESCALATION_TICKS {
            continue;
        }
        if min_stage > 1 || max_stage < 3 {
            continue;
        }
        let name = context_name(contexts, actor);
        return Some(make_notification(
            current_tick,
            NotificationTier::Drama,
            "rapid_escalation",
            calculate_importance(&SimEventType::StressEscalated, store, current_tick),
            actor,
            None,
            "NOTIF_STRESS_RISING",
            format!("Something is building in {}. Stress is rising.", name),
            contexts,
        ));
    }
    None
}

fn sift_social_reversal(
    store: &EventStore,
    window_start: u64,
    contexts: &HashMap<u32, EntityStoryContext>,
    current_tick: u64,
) -> Option<SimNotification> {
    let mut pairs: HashMap<(u32, u32), SocialReversalPair<'_>> = HashMap::new();
    for event in store.since_tick(window_start) {
        let Some(target) = event.target else {
            continue;
        };
        if !matches!(
            event.event_type,
            SimEventType::SocialConflict | SimEventType::SocialCooperation
        ) {
            continue;
        }
        let key = ordered_pair(event.actor, target);
        let entry = pairs.entry(key).or_insert((None, None));
        match event.event_type {
            SimEventType::SocialConflict => entry.0 = Some(event),
            SimEventType::SocialCooperation => entry.1 = Some(event),
            _ => {}
        }
    }
    for ((left, right), (conflict, cooperation)) in pairs {
        let (Some(conflict_event), Some(cooperation_event)) = (conflict, cooperation) else {
            continue;
        };
        if conflict_event.tick == cooperation_event.tick {
            continue;
        }
        let last_was_conflict = conflict_event.tick > cooperation_event.tick;
        let message = if last_was_conflict {
            format!(
                "{} and {} turned from cooperation to conflict.",
                context_name(contexts, left),
                context_name(contexts, right)
            )
        } else {
            format!(
                "{} and {} turned from conflict to cooperation.",
                context_name(contexts, left),
                context_name(contexts, right)
            )
        };
        return Some(make_notification(
            current_tick,
            NotificationTier::Drama,
            "social_reversal",
            calculate_importance(&SimEventType::SocialConflict, store, current_tick),
            left,
            Some(right),
            if last_was_conflict {
                "NOTIF_CONFLICT"
            } else {
                "NOTIF_RELATIONSHIP"
            },
            message,
            contexts,
        ));
    }
    None
}

fn sift_simultaneous_crisis(
    store: &EventStore,
    current_tick: u64,
    contexts: &HashMap<u32, EntityStoryContext>,
) -> Option<SimNotification> {
    let window_start = current_tick.saturating_sub(config::STORY_SIFTER_BREAK_CLUSTER_TICKS);
    let actors: Vec<u32> = store
        .by_type(&SimEventType::MentalBreakStart, window_start)
        .into_iter()
        .map(|event| event.actor)
        .collect();
    let unique = unique_ordered(&actors);
    if unique.len() < 2 {
        return None;
    }
    let first = unique[0];
    let second = unique[1];
    Some(make_notification(
        current_tick,
        NotificationTier::Crisis,
        "simultaneous_crisis",
        calculate_importance(&SimEventType::MentalBreakStart, store, current_tick),
        first,
        Some(second),
        "NOTIF_MENTAL_BREAK",
        format!(
            "{} and {} are both breaking down.",
            context_name(contexts, first),
            context_name(contexts, second)
        ),
        contexts,
    ))
}

fn sift_deadline_approaching(
    store: &EventStore,
    window_start: u64,
    contexts: &HashMap<u32, EntityStoryContext>,
    current_tick: u64,
) -> Option<SimNotification> {
    let mut by_actor: HashMap<u32, Vec<&SimEvent>> = HashMap::new();
    for event in store.by_type(&SimEventType::NeedCritical, window_start) {
        by_actor.entry(event.actor).or_default().push(event);
    }
    for (actor, events) in by_actor {
        let Some(first) = events.first() else {
            continue;
        };
        let Some(last) = events.last() else {
            continue;
        };
        if last.tick.saturating_sub(first.tick) < config::STORY_SIFTER_NEED_PERSISTENCE_TICKS
            || events.len() < 2
        {
            continue;
        }
        let cause = if last.cause.is_empty() {
            "critical need".to_string()
        } else {
            last.cause.clone()
        };
        return Some(make_notification(
            current_tick,
            NotificationTier::Drama,
            "deadline_approaching",
            calculate_importance(&SimEventType::NeedCritical, store, current_tick),
            actor,
            None,
            "NOTIF_NEED_CRITICAL",
            format!(
                "{} is in desperate need. {}",
                context_name(contexts, actor),
                cause
            ),
            contexts,
        ));
    }
    None
}

/// Cap entity scan to avoid O(n × r²) explosion at high agent counts.
const TRIANGLE_SCAN_CAP: usize = 50;

fn sift_relationship_triangle(
    contexts: &HashMap<u32, EntityStoryContext>,
    current_tick: u64,
) -> Option<SimNotification> {
    let mut actors: Vec<u32> = contexts.keys().copied().collect();
    actors.sort_unstable();
    // Only scan first TRIANGLE_SCAN_CAP entities to bound worst-case cost
    for actor_a in actors.iter().take(TRIANGLE_SCAN_CAP) {
        let Some(context_a) = contexts.get(actor_a) else {
            continue;
        };
        for (target_b, affinity_ab, relation_ab) in &context_a.relations {
            if !is_positive_relation(*relation_ab, *affinity_ab) {
                continue;
            }
            let Some(context_b) = contexts.get(target_b) else {
                continue;
            };
            for (target_c, affinity_bc, relation_bc) in &context_b.relations {
                if *target_c == *actor_a || !is_positive_relation(*relation_bc, *affinity_bc) {
                    continue;
                }
                let affinity_ac = relation_affinity(contexts, *actor_a, *target_c);
                if affinity_ac >= config::STORY_SIFTER_NEGATIVE_RELATION_AFFINITY {
                    continue;
                }
                return Some(make_notification(
                    current_tick,
                    NotificationTier::Drama,
                    "relationship_triangle",
                    0.75,
                    *actor_a,
                    Some(*target_c),
                    "NOTIF_CONFLICT",
                    format!(
                        "{}, {}, and {} are caught in a tense triangle.",
                        context_name(contexts, *actor_a),
                        context_name(contexts, *target_b),
                        context_name(contexts, *target_c)
                    ),
                    contexts,
                ));
            }
        }
    }
    None
}

fn sift_personality_clash(
    store: &EventStore,
    window_start: u64,
    contexts: &HashMap<u32, EntityStoryContext>,
    current_tick: u64,
) -> Option<SimNotification> {
    let mut pair_counts: HashMap<(u32, u32), usize> = HashMap::new();
    for event in store.by_type(&SimEventType::SocialConflict, window_start) {
        let Some(target) = event.target else {
            continue;
        };
        *pair_counts
            .entry(ordered_pair(event.actor, target))
            .or_insert(0) += 1;
    }
    for ((left, right), count) in pair_counts {
        if count < 2 {
            continue;
        }
        let distance = personality_distance(contexts, left, right);
        if distance < config::STORY_SIFTER_PERSONALITY_CLASH_DISTANCE {
            continue;
        }
        return Some(make_notification(
            current_tick,
            NotificationTier::Drama,
            "personality_clash",
            calculate_importance(&SimEventType::SocialConflict, store, current_tick),
            left,
            Some(right),
            "NOTIF_CONFLICT",
            format!(
                "{} and {} keep clashing.",
                context_name(contexts, left),
                context_name(contexts, right)
            ),
            contexts,
        ));
    }
    None
}

fn sift_recovery_arc(
    store: &EventStore,
    current_tick: u64,
    contexts: &HashMap<u32, EntityStoryContext>,
) -> Option<SimNotification> {
    let window_start = current_tick.saturating_sub(config::STORY_SIFTER_RECOVERY_TICKS);
    let starts = store.by_type(&SimEventType::MentalBreakStart, window_start);
    for start in starts {
        let is_calm = contexts
            .get(&start.actor)
            .and_then(|context| context.stress_state)
            .map(|state| state == StressState::Calm)
            .unwrap_or(false);
        if !is_calm {
            continue;
        }
        let had_end = store
            .by_actor(start.actor, start.tick)
            .into_iter()
            .any(|event| event.event_type == SimEventType::MentalBreakEnd);
        if !had_end {
            continue;
        }
        return Some(make_notification(
            current_tick,
            NotificationTier::Milestone,
            "recovery_arc",
            0.8,
            start.actor,
            None,
            "NOTIF_STRESS_RISING",
            format!(
                "{} found calm again after breaking down.",
                context_name(contexts, start.actor)
            ),
            contexts,
        ));
    }
    None
}

fn immediate_notification_for_event(
    event: &SimEvent,
    contexts: &HashMap<u32, EntityStoryContext>,
    store: &EventStore,
    current_tick: u64,
) -> Option<SimNotification> {
    let importance = calculate_importance(&event.event_type, store, current_tick);
    match event.event_type {
        SimEventType::MentalBreakStart => Some(make_notification(
            event.tick,
            NotificationTier::Crisis,
            "mental_break",
            importance,
            event.actor,
            event.target,
            "NOTIF_MENTAL_BREAK",
            format!(
                "{} broke down. The stress was too much.",
                context_name(contexts, event.actor)
            ),
            contexts,
        )),
        SimEventType::Death => Some(make_notification(
            event.tick,
            NotificationTier::Crisis,
            "death",
            importance,
            event.actor,
            event.target,
            "NOTIF_DEATH",
            format!(
                "{} has died. {}",
                context_name(contexts, event.actor),
                event.cause
            ),
            contexts,
        )),
        SimEventType::RelationshipFormed => Some(make_notification(
            event.tick,
            NotificationTier::Milestone,
            "relationship",
            importance,
            event.actor,
            event.target,
            "NOTIF_RELATIONSHIP",
            format!(
                "{} and {} have grown closer.",
                context_name(contexts, event.actor),
                context_name(contexts, event.target.unwrap_or(event.actor))
            ),
            contexts,
        )),
        SimEventType::SocialConflict => Some(make_notification(
            event.tick,
            NotificationTier::Drama,
            "conflict",
            importance,
            event.actor,
            event.target,
            "NOTIF_CONFLICT",
            format!(
                "{} and {} are in conflict.",
                context_name(contexts, event.actor),
                context_name(contexts, event.target.unwrap_or(event.actor))
            ),
            contexts,
        )),
        SimEventType::StressEscalated
            if event.value >= config::STORY_SIFTER_STRESS_NOTIFY_STAGE =>
        {
            Some(make_notification(
                event.tick,
                NotificationTier::Drama,
                "stress_rising",
                importance,
                event.actor,
                event.target,
                "NOTIF_STRESS_RISING",
                format!(
                    "Something is building in {}.",
                    context_name(contexts, event.actor)
                ),
                contexts,
            ))
        }
        SimEventType::Birth => Some(make_notification(
            event.tick,
            NotificationTier::Milestone,
            "birth",
            importance,
            event.actor,
            event.target,
            "",
            format!("{} was born.", context_name(contexts, event.actor)),
            contexts,
        )),
        SimEventType::AgeTransition => Some(make_notification(
            event.tick,
            NotificationTier::Milestone,
            "age_transition",
            importance,
            event.actor,
            event.target,
            "",
            format!(
                "{} reached {}.",
                context_name(contexts, event.actor),
                event.cause
            ),
            contexts,
        )),
        _ => None,
    }
}

fn calculate_importance(event_type: &SimEventType, store: &EventStore, current_tick: u64) -> f64 {
    let window = current_tick.saturating_sub(config::STORY_SIFTER_IMPORTANCE_LOOKBACK_TICKS);
    let occurrences = store.by_type(event_type, window).len() as f64;
    let base = match event_type {
        SimEventType::MentalBreakStart | SimEventType::Death => 1.0,
        SimEventType::SocialConflict => 0.7,
        SimEventType::RelationshipFormed => 0.5,
        SimEventType::BandSplit => 0.8,
        SimEventType::BandPromoted | SimEventType::BandFormed => 0.6,
        SimEventType::FirstOccurrence => 0.9,
        _ => 0.3,
    };
    let rarity_bonus = 1.0 / (occurrences + 1.0);
    (base + rarity_bonus * 0.5).min(1.0)
}

fn make_notification(
    tick: u64,
    tier: NotificationTier,
    kind: &str,
    importance: f64,
    primary_entity: u32,
    secondary_entity: Option<u32>,
    message_key: &str,
    message_fallback: String,
    contexts: &HashMap<u32, EntityStoryContext>,
) -> SimNotification {
    let (position_x, position_y) = contexts
        .get(&primary_entity)
        .map(|context| (context.position_x, context.position_y))
        .unwrap_or((0.0, 0.0));
    SimNotification {
        tick,
        tier,
        kind: kind.to_string(),
        importance,
        primary_entity,
        secondary_entity,
        message_key: message_key.to_string(),
        message_fallback,
        position_x,
        position_y,
    }
}

fn context_name(contexts: &HashMap<u32, EntityStoryContext>, entity_id: u32) -> String {
    contexts
        .get(&entity_id)
        .map(|context| context.name.clone())
        .unwrap_or_else(|| format!("Agent {}", entity_id))
}

fn label_for_event_type(event_type: &SimEventType) -> &'static str {
    match event_type {
        SimEventType::NeedCritical => "need critical",
        SimEventType::NeedSatisfied => "need satisfied",
        SimEventType::EmotionShift => "emotion shift",
        SimEventType::MoodChanged => "mood changed",
        SimEventType::StressEscalated => "stress escalation",
        SimEventType::MentalBreakStart => "mental break",
        SimEventType::MentalBreakEnd => "mental break recovery",
        SimEventType::RelationshipFormed => "relationship formed",
        SimEventType::RelationshipBroken => "relationship broken",
        SimEventType::SocialConflict => "social conflict",
        SimEventType::SocialCooperation => "social cooperation",
        SimEventType::BandFormed => "band formed",
        SimEventType::BandPromoted => "band promoted",
        SimEventType::BandSplit => "band split",
        SimEventType::BandDissolved => "band dissolved",
        SimEventType::BandLeaderElected => "band leader elected",
        SimEventType::LonerJoinedBand => "loner joined band",
        SimEventType::ActionChanged => "action changed",
        SimEventType::TaskCompleted => "task completed",
        SimEventType::Birth => "birth",
        SimEventType::Death => "death",
        SimEventType::AgeTransition => "age transition",
        SimEventType::FirstOccurrence => "first occurrence",
        SimEventType::Custom(_) => "custom event",
    }
}

fn ordered_pair(left: u32, right: u32) -> (u32, u32) {
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}

fn unique_ordered(values: &[u32]) -> Vec<u32> {
    let mut out: Vec<u32> = Vec::new();
    for value in values {
        if !out.contains(value) {
            out.push(*value);
        }
    }
    out
}

fn personality_distance(contexts: &HashMap<u32, EntityStoryContext>, left: u32, right: u32) -> f64 {
    let Some(left_context) = contexts.get(&left) else {
        return 0.0;
    };
    let Some(right_context) = contexts.get(&right) else {
        return 0.0;
    };
    left_context
        .personality_axes
        .iter()
        .zip(right_context.personality_axes.iter())
        .map(|(left_axis, right_axis)| (left_axis - right_axis).abs())
        .sum::<f64>()
        / 6.0
}

fn relation_affinity(contexts: &HashMap<u32, EntityStoryContext>, left: u32, right: u32) -> f64 {
    contexts
        .get(&left)
        .and_then(|context| {
            context
                .relations
                .iter()
                .find(|(target, _, _)| *target == right)
                .map(|(_, affinity, _)| *affinity)
        })
        .unwrap_or(0.0)
}

fn is_positive_relation(relation_type: RelationType, affinity: f64) -> bool {
    affinity >= config::STORY_SIFTER_POSITIVE_RELATION_AFFINITY
        || matches!(
            relation_type,
            RelationType::Friend | RelationType::CloseFriend | RelationType::Intimate
        )
}

#[cfg(test)]
mod tests {
    use super::{
        calculate_importance, should_notify, sift_first_occurrence, StorySifterRuntimeSystem,
    };
    use hecs::World;
    use sim_core::{config::GameConfig, GameCalendar, WorldMap};
    use sim_engine::{
        EventStore, NotificationTier, SimEvent, SimEventType, SimNotification, SimResources,
        SimSystem,
    };

    fn make_resources() -> SimResources {
        let config = GameConfig::default();
        SimResources::new(GameCalendar::new(&config), WorldMap::new(8, 8, 7), 7)
    }

    fn make_event(tick: u64, actor: u32, event_type: SimEventType) -> SimEvent {
        SimEvent {
            tick,
            event_type,
            actor,
            target: None,
            tags: vec!["test".to_string()],
            cause: "test".to_string(),
            value: 0.0,
        }
    }

    #[test]
    fn first_occurrence_pattern_emits_milestone() {
        let mut store = EventStore::new(32);
        store.push(make_event(10, 1, SimEventType::Birth));
        let notification = sift_first_occurrence(&store, 0, &std::collections::HashMap::new(), 10)
            .expect("first occurrence should produce a milestone");
        assert_eq!(notification.tier, NotificationTier::Milestone);
        assert_eq!(notification.kind, "first_occurrence");
    }

    #[test]
    fn cooldown_blocks_duplicate_kind_for_same_actor() {
        let recent = vec![SimNotification {
            tick: 100,
            tier: NotificationTier::Drama,
            kind: "conflict".to_string(),
            importance: 0.5,
            primary_entity: 10,
            secondary_entity: None,
            message_key: String::new(),
            message_fallback: "x".to_string(),
            position_x: 0.0,
            position_y: 0.0,
        }];
        let next = SimNotification {
            tick: 120,
            tier: NotificationTier::Drama,
            kind: "conflict".to_string(),
            importance: 0.7,
            primary_entity: 10,
            secondary_entity: None,
            message_key: String::new(),
            message_fallback: "y".to_string(),
            position_x: 0.0,
            position_y: 0.0,
        };
        assert!(!should_notify(&next, &recent, 120));
    }

    #[test]
    fn runtime_system_promotes_mental_break_events() {
        let mut world = World::new();
        let mut resources = make_resources();
        resources
            .event_store
            .push(make_event(10, 7, SimEventType::MentalBreakStart));
        let mut system = StorySifterRuntimeSystem::new(sim_core::config::STORY_SIFTER_PRIORITY, 10);
        system.run(&mut world, &mut resources, 10);
        assert!(resources
            .pending_notifications
            .iter()
            .any(|notification| notification.kind == "mental_break"));
    }

    #[test]
    fn rarity_importance_decreases_with_frequency() {
        let mut store = EventStore::new(32);
        store.push(make_event(1, 1, SimEventType::Death));
        let first = calculate_importance(&SimEventType::Death, &store, 1);
        store.push(make_event(2, 2, SimEventType::Death));
        let second = calculate_importance(&SimEventType::Death, &store, 2);
        assert!(first >= second);
    }

    #[test]
    fn stage1_notification_cooldown_allows_different_actor() {
        let recent = vec![SimNotification {
            tick: 80,
            tier: NotificationTier::Drama,
            kind: "conflict".to_string(),
            importance: 0.5,
            primary_entity: 42,
            secondary_entity: None,
            message_key: String::new(),
            message_fallback: "older".to_string(),
            position_x: 0.0,
            position_y: 0.0,
        }];
        let next = SimNotification {
            tick: 100,
            tier: NotificationTier::Drama,
            kind: "conflict".to_string(),
            importance: 0.7,
            primary_entity: 99,
            secondary_entity: None,
            message_key: String::new(),
            message_fallback: "next".to_string(),
            position_x: 0.0,
            position_y: 0.0,
        };
        assert!(should_notify(&next, &recent, 100));
    }

    #[test]
    fn stage1_importance_scoring_rare_events_higher() {
        let mut store = EventStore::new(64);
        for tick in 0..10 {
            store.push(make_event(tick, 1, SimEventType::ActionChanged));
        }
        store.push(make_event(11, 1, SimEventType::MentalBreakStart));
        let common = calculate_importance(&SimEventType::ActionChanged, &store, 12);
        let rare = calculate_importance(&SimEventType::MentalBreakStart, &store, 12);
        assert!(rare > common);
    }
}
