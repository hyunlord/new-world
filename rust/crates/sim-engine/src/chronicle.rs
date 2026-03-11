use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};
use sim_core::{config, ChannelId, EntityId};

/// Structured chronicle event categories emitted by runtime systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChronicleEventType {
    /// A general movement decision was made from one dominant influence.
    MovementDecision,
    /// The entity moved toward an attractive influence field.
    InfluenceAttraction,
    /// The entity moved away from an aversive influence field.
    InfluenceAvoidance,
    /// The entity joined or reinforced a local group cluster.
    GatheringFormation,
    /// The entity biased movement toward shelter or warmth.
    ShelterSeeking,
    /// The entity discovered or biased toward a resource-rich location.
    ResourceDiscovery,
}

/// Typed dominant cause categories for chronicle attribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChronicleEventCause {
    /// Food influence dominated the decision.
    Food,
    /// Danger influence dominated the decision.
    Danger,
    /// Warmth influence dominated the decision.
    Warmth,
    /// Social influence dominated the decision.
    Social,
    /// Authority influence dominated the decision.
    Authority,
    /// Noise influence dominated the decision.
    Noise,
    /// Disease influence dominated the decision.
    Disease,
    /// No typed influence channel was available.
    Unknown,
}

impl ChronicleEventCause {
    /// Returns a stable raw identifier for debug or bridge consumers.
    pub fn id(self) -> &'static str {
        match self {
            Self::Food => "food",
            Self::Danger => "danger",
            Self::Warmth => "warmth",
            Self::Social => "social",
            Self::Authority => "authority",
            Self::Noise => "noise",
            Self::Disease => "disease",
            Self::Unknown => "unknown",
        }
    }
}

impl From<ChannelId> for ChronicleEventCause {
    fn from(value: ChannelId) -> Self {
        match value {
            ChannelId::Food => Self::Food,
            ChannelId::Danger => Self::Danger,
            ChannelId::Warmth => Self::Warmth,
            ChannelId::Social => Self::Social,
            ChannelId::Authority => Self::Authority,
            ChannelId::Noise => Self::Noise,
            ChannelId::Disease => Self::Disease,
            ChannelId::Light | ChannelId::Spiritual | ChannelId::Beauty => Self::Unknown,
        }
    }
}

/// Scalar magnitudes attached to one chronicle event.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChronicleEventMagnitude {
    /// Dominant sampled signal strength.
    pub influence: f64,
    /// Final steering-force magnitude applied this tick.
    pub steering: f64,
    /// Significance score used for bounded logging and pruning.
    pub significance: f64,
}

impl ChronicleEventMagnitude {
    /// Returns `true` when the event should survive low-significance pruning.
    pub fn is_high_significance(self) -> bool {
        self.significance >= config::CHRONICLE_HIGH_SIGNIFICANCE_THRESHOLD
    }

    /// Returns `true` when the event should survive medium-significance pruning.
    pub fn is_medium_significance(self) -> bool {
        self.significance >= config::CHRONICLE_MEDIUM_SIGNIFICANCE_THRESHOLD
    }
}

/// Structured chronicle event stored in the bounded runtime log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChronicleEvent {
    /// Tick at which the event occurred.
    pub tick: u64,
    /// Target entity that experienced the event.
    pub entity_id: EntityId,
    /// High-level event category.
    pub event_type: ChronicleEventType,
    /// Dominant influence or cause category.
    pub cause: ChronicleEventCause,
    /// Scalar magnitudes associated with the decision.
    pub magnitude: ChronicleEventMagnitude,
    /// Tile-space X location of the event.
    pub tile_x: i32,
    /// Tile-space Y location of the event.
    pub tile_y: i32,
    /// Locale key summarizing the event for later UI translation.
    pub summary_key: String,
    /// Stable effect identifier for debugging.
    pub effect_key: String,
}

/// Temporal cluster of related chronicle events for one entity or local group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChronicleCluster {
    /// Inclusive first tick represented by the cluster.
    pub start_tick: u64,
    /// Inclusive last tick represented by the cluster.
    pub end_tick: u64,
    /// Primary entity attached to the cluster, if any.
    pub entity_id: Option<EntityId>,
    /// Ordered raw chronicle events that belong to the cluster.
    pub events: Vec<ChronicleEvent>,
}

impl ChronicleCluster {
    /// Creates a cluster seeded with one raw chronicle event.
    pub fn new(event: ChronicleEvent) -> Self {
        Self {
            start_tick: event.tick,
            end_tick: event.tick,
            entity_id: Some(event.entity_id),
            events: vec![event],
        }
    }

    /// Appends one chronicle event and expands the cluster bounds.
    pub fn push(&mut self, event: ChronicleEvent) {
        self.start_tick = self.start_tick.min(event.tick);
        self.end_tick = self.end_tick.max(event.tick);
        self.events.push(event);
    }
}

/// Narrative-ready summary entry stored in the bounded chronicle timeline.
///
/// `title` and `description` are locale keys resolved by Godot UI code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChronicleSummary {
    /// Inclusive first tick represented by the summary.
    pub start_tick: u64,
    /// Inclusive last tick represented by the summary.
    pub end_tick: u64,
    /// Primary entity attached to the summary, if any.
    pub entity_id: Option<EntityId>,
    /// Dominant low-level event category represented by the summary.
    pub event_type: ChronicleEventType,
    /// Dominant influence or cause represented by the summary.
    pub cause: ChronicleEventCause,
    /// Locale key for the summary title.
    pub title: String,
    /// Locale key for the summary description.
    pub description: String,
    /// Locale parameters for `title` and `description`.
    pub params: BTreeMap<String, String>,
    /// Representative tile-space X location of the summary.
    pub tile_x: i32,
    /// Representative tile-space Y location of the summary.
    pub tile_y: i32,
    /// Significance score assigned during summarization.
    pub significance: f64,
}

/// Bounded recent world-history timeline built from raw chronicle events.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChronicleTimeline {
    summaries: VecDeque<ChronicleSummary>,
    max_summaries: usize,
}

impl ChronicleTimeline {
    /// Creates a new bounded chronicle timeline.
    pub fn new() -> Self {
        Self {
            summaries: VecDeque::with_capacity(config::CHRONICLE_TIMELINE_MAX_ENTRIES),
            max_summaries: config::CHRONICLE_TIMELINE_MAX_ENTRIES,
        }
    }

    /// Appends one summary and prunes the oldest entry when capacity is exceeded.
    pub fn append_summary(&mut self, summary: ChronicleSummary) -> bool {
        let mut pruned = false;
        if self.summaries.len() >= self.max_summaries {
            self.summaries.pop_front();
            pruned = true;
        }
        self.summaries.push_back(summary);
        pruned
    }

    /// Returns recent summaries, newest first.
    pub fn recent_summaries(&self, count: usize) -> Vec<&ChronicleSummary> {
        self.summaries.iter().rev().take(count).collect()
    }

    /// Returns recent summaries for one entity, newest first.
    pub fn query_by_entity(&self, entity_id: EntityId, count: usize) -> Vec<&ChronicleSummary> {
        self.summaries
            .iter()
            .rev()
            .filter(|summary| summary.entity_id == Some(entity_id))
            .take(count)
            .collect()
    }

    /// Clears all stored summaries.
    pub fn clear(&mut self) {
        self.summaries.clear();
    }

    /// Returns the number of stored summaries.
    pub fn len(&self) -> usize {
        self.summaries.len()
    }

    /// Returns `true` when no summaries are currently stored.
    pub fn is_empty(&self) -> bool {
        self.summaries.is_empty()
    }
}

/// Bounded world and per-entity chronicle storage.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChronicleLog {
    world_events: VecDeque<ChronicleEvent>,
    personal_events: BTreeMap<EntityId, VecDeque<ChronicleEvent>>,
    max_world_events: usize,
    max_per_entity_events: usize,
}

impl ChronicleLog {
    /// Creates a new bounded chronicle log.
    pub fn new() -> Self {
        Self {
            world_events: VecDeque::with_capacity(config::CHRONICLE_LOG_MAX_EVENTS),
            personal_events: BTreeMap::new(),
            max_world_events: config::CHRONICLE_LOG_MAX_EVENTS,
            max_per_entity_events: config::CHRONICLE_LOG_MAX_PER_ENTITY,
        }
    }

    /// Appends one event to the world log and entity-local ring buffer.
    pub fn append_event(&mut self, event: ChronicleEvent) {
        if self.world_events.len() >= self.max_world_events {
            self.world_events.pop_front();
        }
        self.world_events.push_back(event.clone());

        let personal = self
            .personal_events
            .entry(event.entity_id)
            .or_insert_with(|| VecDeque::with_capacity(self.max_per_entity_events));
        if personal.len() >= self.max_per_entity_events {
            personal.pop_front();
        }
        personal.push_back(event);
    }

    /// Returns recent world events, newest first.
    pub fn recent_events(&self, count: usize) -> Vec<&ChronicleEvent> {
        self.world_events.iter().rev().take(count).collect()
    }

    /// Returns recent events for one entity, newest first.
    pub fn query_by_entity(&self, entity_id: EntityId, count: usize) -> Vec<&ChronicleEvent> {
        self.personal_events
            .get(&entity_id)
            .map(|events| events.iter().rev().take(count).collect())
            .unwrap_or_default()
    }

    /// Returns the latest event for one entity, if any.
    pub fn latest_for_entity(&self, entity_id: EntityId) -> Option<&ChronicleEvent> {
        self.personal_events
            .get(&entity_id)
            .and_then(VecDeque::back)
    }

    /// Clears all world and personal events.
    pub fn clear(&mut self) {
        self.world_events.clear();
        self.personal_events.clear();
    }

    /// Returns the current world-event count.
    pub fn world_len(&self) -> usize {
        self.world_events.len()
    }

    /// Returns world events newer than `tick_exclusive`, oldest first.
    pub fn events_since(&self, tick_exclusive: u64) -> Vec<&ChronicleEvent> {
        self.world_events
            .iter()
            .filter(|event| event.tick > tick_exclusive)
            .collect()
    }

    /// Prunes aged low-significance events while preserving bounded storage.
    pub fn prune_by_significance(&mut self, low_cutoff_tick: u64, medium_cutoff_tick: u64) {
        self.world_events.retain(|event| {
            if event.magnitude.is_high_significance() {
                return true;
            }
            if event.magnitude.is_medium_significance() {
                return event.tick >= medium_cutoff_tick;
            }
            event.tick >= low_cutoff_tick
        });

        while self.world_events.len() > self.max_world_events {
            self.world_events.pop_front();
        }

        let valid_ticks: BTreeSet<u64> = self.world_events.iter().map(|event| event.tick).collect();
        self.personal_events.retain(|_, events| {
            events.retain(|event| {
                if event.magnitude.is_high_significance() {
                    return true;
                }
                valid_ticks.contains(&event.tick)
            });
            !events.is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event(tick: u64, entity_id: EntityId, significance: f64) -> ChronicleEvent {
        ChronicleEvent {
            tick,
            entity_id,
            event_type: ChronicleEventType::InfluenceAttraction,
            cause: ChronicleEventCause::Food,
            magnitude: ChronicleEventMagnitude {
                influence: significance,
                steering: significance,
                significance,
            },
            tile_x: 4,
            tile_y: 9,
            summary_key: "CAUSE_INFLUENCE_FOOD_GRADIENT".to_string(),
            effect_key: "steering_velocity".to_string(),
        }
    }

    #[test]
    fn chronicle_log_keeps_recent_world_and_personal_entries_bounded() {
        let mut log = ChronicleLog::new();
        let entity = EntityId(7);
        for tick in 0..(config::CHRONICLE_LOG_MAX_EVENTS + 5) as u64 {
            log.append_event(sample_event(tick, entity, 0.8));
        }

        assert_eq!(log.world_len(), config::CHRONICLE_LOG_MAX_EVENTS);
        assert_eq!(
            log.query_by_entity(entity, 64).len(),
            config::CHRONICLE_LOG_MAX_PER_ENTITY
        );
        assert_eq!(
            log.latest_for_entity(entity).map(|event| event.tick),
            Some((config::CHRONICLE_LOG_MAX_EVENTS + 4) as u64)
        );
    }

    #[test]
    fn chronicle_log_prunes_old_low_significance_events() {
        let mut log = ChronicleLog::new();
        let low_entity = EntityId(1);
        let medium_entity = EntityId(2);
        let high_entity = EntityId(3);
        log.append_event(sample_event(10, low_entity, 0.10));
        log.append_event(sample_event(20, medium_entity, 0.40));
        log.append_event(sample_event(30, high_entity, 0.90));

        log.prune_by_significance(15, 25);

        assert!(log.latest_for_entity(low_entity).is_none());
        assert!(log.latest_for_entity(medium_entity).is_none());
        assert_eq!(log.latest_for_entity(high_entity).map(|event| event.tick), Some(30));
    }

    #[test]
    fn chronicle_timeline_keeps_recent_entries_bounded() {
        let mut timeline = ChronicleTimeline::new();
        for index in 0..(config::CHRONICLE_TIMELINE_MAX_ENTRIES + 5) {
            timeline.append_summary(ChronicleSummary {
                start_tick: index as u64,
                end_tick: index as u64,
                entity_id: Some(EntityId(index as u64)),
                event_type: ChronicleEventType::InfluenceAttraction,
                cause: ChronicleEventCause::Food,
                title: "CHRONICLE_TITLE_FOOD_ATTRACTION".to_string(),
                description: "CHRONICLE_SUMMARY_FOOD_ATTRACTION".to_string(),
                params: BTreeMap::new(),
                tile_x: 0,
                tile_y: 0,
                significance: 8.0,
            });
        }

        assert_eq!(timeline.len(), config::CHRONICLE_TIMELINE_MAX_ENTRIES);
        assert_eq!(
            timeline
                .recent_summaries(1)
                .first()
                .and_then(|summary| summary.entity_id),
            Some(EntityId((config::CHRONICLE_TIMELINE_MAX_ENTRIES + 4) as u64))
        );
    }

    #[test]
    fn chronicle_timeline_queries_recent_entries_by_entity() {
        let mut timeline = ChronicleTimeline::new();
        let entity_id = EntityId(44);
        for tick in 0..3_u64 {
            timeline.append_summary(ChronicleSummary {
                start_tick: tick,
                end_tick: tick,
                entity_id: Some(entity_id),
                event_type: ChronicleEventType::ShelterSeeking,
                cause: ChronicleEventCause::Warmth,
                title: "CHRONICLE_TITLE_SHELTER_SEEKING".to_string(),
                description: "CHRONICLE_SUMMARY_SHELTER_SEEKING".to_string(),
                params: BTreeMap::new(),
                tile_x: 1,
                tile_y: 2,
                significance: 7.5,
            });
        }

        let summaries = timeline.query_by_entity(entity_id, 2);
        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[0].start_tick, 2);
        assert_eq!(summaries[1].start_tick, 1);
    }
}
