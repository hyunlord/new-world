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
    /// A band lifecycle event surfaced in the social narrative.
    BandLifecycle,
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
    /// Social-group dynamics drove the event.
    SocialGroup,
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
            Self::SocialGroup => "social_group",
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
    /// Locale parameters used when the event is summarized into the timeline.
    #[serde(default)]
    pub summary_params: BTreeMap<String, String>,
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

/// Attention category assigned to one chronicle entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ChronicleSignificanceCategory {
    /// The summary should be ignored by the active timeline.
    Ignore,
    /// The summary is too weak for the timeline and is dropped.
    Minor,
    /// The summary is relevant but should stay in the background queue.
    Notable,
    /// The summary is important enough for the visible queue when slots exist.
    Major,
    /// The summary must surface even if another visible item must be displaced.
    Critical,
}

impl ChronicleSignificanceCategory {
    /// Returns a stable identifier for bridge/UI consumers.
    pub fn id(self) -> &'static str {
        match self {
            Self::Ignore => "ignore",
            Self::Minor => "minor",
            Self::Notable => "notable",
            Self::Major => "major",
            Self::Critical => "critical",
        }
    }
}

/// Result of routing one chronicle entry through the attention budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChronicleRouteResult {
    /// Queue that received the entry.
    pub queue: ChronicleQueueBucket,
    /// Whether one queue pruned its oldest entry.
    pub pruned: bool,
    /// Whether a visible entry had to be displaced into recall.
    pub displaced_visible: bool,
    /// Whether this visible entry was promoted from the background queue.
    pub promoted_background: bool,
}

impl ChronicleRouteResult {
    /// Returns a route result for one dropped entry.
    pub fn dropped() -> Self {
        Self {
            queue: ChronicleQueueBucket::Dropped,
            pruned: false,
            displaced_visible: false,
            promoted_background: false,
        }
    }
}

/// Stable non-reused identifier for one chronicle entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct ChronicleEntryId(pub u64);

impl ChronicleEntryId {
    /// Creates the next stable chronicle entry id from one monotonic counter.
    pub fn from_counter(counter: u64) -> Self {
        Self(counter)
    }
}

/// Lifecycle status for one chronicle entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChronicleEntryStatus {
    /// The entry was created but not yet routed through the attention budget.
    Pending,
    /// The entry is active in one runtime queue.
    Published,
    /// The entry was suppressed and is not stored in an active runtime queue.
    Suppressed,
    /// Reserved for future archive/history migration.
    Archived,
}

/// Queue bucket that currently owns one chronicle entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChronicleQueueBucket {
    /// The entry is currently visible to the player.
    Visible,
    /// The entry is stored in the background queue.
    Background,
    /// The entry is stored in the recall queue.
    Recall,
    /// The entry was dropped during routing.
    Dropped,
}

impl ChronicleQueueBucket {
    /// Returns a stable identifier for bridge/UI consumers.
    pub fn id(self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::Background => "background",
            Self::Recall => "recall",
            Self::Dropped => "dropped",
        }
    }
}

/// Backward-compatible alias kept during queue bucket migration.
pub type ChronicleQueueKind = ChronicleQueueBucket;

/// Monotonic revision identifier for chronicle snapshot responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct ChronicleSnapshotRevision(pub u64);

impl ChronicleSnapshotRevision {
    /// Returns the next monotonic chronicle snapshot revision.
    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1).max(1))
    }
}

/// Stable thread identifier, monotonically allocated by `ChronicleTimeline`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChronicleThreadId(pub u64);

/// Thread grouping key. Two entries with the same key belong to the same thread.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChronicleThreadKey {
    /// Stable event-family identifier for grouped entries.
    pub event_family: String,
    /// Entity or spatial scope used to keep threads local and deterministic.
    pub scope: ChronicleThreadScope,
}

/// Discriminator for thread entity scope.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ChronicleThreadScope {
    /// Thread tracks one agent's recurring pattern.
    Agent(EntityId),
    /// Thread tracks recurring events in one spatial bucket.
    Location(i32, i32),
    /// Thread has no subject or location scope.
    Global,
}

impl ChronicleThreadScope {
    /// Returns a stable machine-readable scope identifier for bridge consumers.
    pub fn to_id_string(&self) -> String {
        match self {
            Self::Agent(id) => format!("agent:{}", id.0),
            Self::Location(bucket_x, bucket_y) => format!("location:{bucket_x},{bucket_y}"),
            Self::Global => "global".to_string(),
        }
    }
}

/// Thread lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChronicleThreadState {
    /// Thread has exactly one entry and is not yet an established pattern.
    Emerging,
    /// Thread has received multiple entries and remains active.
    Developing,
    /// Thread has gone stale and is only retained for recall/debug queries.
    Resolved,
}

impl ChronicleThreadState {
    /// Returns a stable machine-readable state identifier.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Emerging => "emerging",
            Self::Developing => "developing",
            Self::Resolved => "resolved",
        }
    }
}

/// Runtime thread object grouping related chronicle entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronicleThread {
    /// Stable thread identifier.
    pub thread_id: ChronicleThreadId,
    /// Compound grouping key.
    pub key: ChronicleThreadKey,
    /// Current lifecycle state.
    pub state: ChronicleThreadState,
    /// Tick when the first entry was attached to this thread.
    pub started_tick: u64,
    /// Tick when the most recent entry was attached.
    pub last_entry_tick: u64,
    /// Ordered entry membership (chronological, oldest first).
    pub entry_ids: Vec<ChronicleEntryId>,
    /// Recency-weighted tension score used for ranking.
    pub tension_score: f64,
    /// Thread headline derived from the most recent entry headline.
    pub headline: ChronicleHeadline,
}

/// Minimal subject reference carried by one chronicle entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChronicleSubjectRefLite {
    /// Primary entity attached to the entry, if any.
    pub entity_id: Option<EntityId>,
    /// Frozen display name used for legacy bridge/UI consumers.
    pub display_name: Option<String>,
    /// Entity alive/dead/unknown state frozen at entry creation time.
    pub ref_state: ChronicleEntityRefState,
}

/// Entity reference state for chronicle tombstone handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ChronicleEntityRefState {
    /// Entity was alive at the time of entry creation.
    Alive,
    /// Entity was dead at the time of entry creation.
    Dead,
    /// Entity state is unknown or not applicable.
    #[default]
    Unknown,
}

impl ChronicleEntityRefState {
    /// Returns a stable machine-readable identifier for bridge/debug serialization.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Alive => "alive",
            Self::Dead => "dead",
            Self::Unknown => "unknown",
        }
    }
}

/// Minimal location reference carried by one chronicle entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChronicleLocationRefLite {
    /// Representative tile-space X location of the entry.
    pub tile_x: i32,
    /// Representative tile-space Y location of the entry.
    pub tile_y: i32,
    /// Human-readable region or settlement label frozen at entry creation time.
    pub region_label: Option<String>,
}

/// Minimal feed render hint returned by chronicle snapshot endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChronicleFeedRenderHint {
    /// Stable icon identifier for UI-side lookup.
    pub icon_id: String,
    /// Stable color token for feed rendering.
    pub color_token: String,
}

/// Locale-keyed fast-read narrative payload for feed-level chronicle rendering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChronicleHeadline {
    /// Locale key for the headline layer.
    pub locale_key: String,
    /// Locale interpolation params for the headline layer.
    pub params: BTreeMap<String, String>,
}

impl ChronicleHeadline {
    /// Validates headline payload invariants.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.locale_key.trim().is_empty() {
            return Err("headline locale_key must not be empty");
        }
        Ok(())
    }
}

/// Locale-keyed contextual narrative payload for expanded chronicle previews.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChronicleCapsule {
    /// Locale key for the capsule layer.
    pub locale_key: String,
    /// Locale interpolation params for the capsule layer.
    pub params: BTreeMap<String, String>,
}

impl ChronicleCapsule {
    /// Validates capsule payload invariants.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.locale_key.trim().is_empty() {
            return Err("capsule locale_key must not be empty");
        }
        Ok(())
    }
}

/// Minimal future-facing deep-read anchor for chronicle dossier expansion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChronicleDossierStub {
    /// Locale key for the dossier stub layer.
    pub locale_key: String,
    /// Locale interpolation params for the dossier stub layer.
    pub params: BTreeMap<String, String>,
    /// Stable tags that identify the kind of deeper chronicle context available later.
    pub detail_tags: Vec<String>,
}

impl ChronicleDossierStub {
    /// Validates dossier stub payload invariants.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.locale_key.trim().is_empty() {
            return Err("dossier_stub locale_key must not be empty");
        }
        Ok(())
    }
}

/// Canonical runtime chronicle entry stored in the bounded chronicle timeline.
///
/// Layered narrative payload is locale-key based and resolved by Godot UI code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChronicleEntryLite {
    /// Stable chronicle entry identifier.
    pub entry_id: ChronicleEntryId,
    /// Inclusive first tick represented by the entry.
    pub start_tick: u64,
    /// Inclusive last tick represented by the entry.
    pub end_tick: u64,
    /// Stable event-family identifier for future thread/archive attachment.
    pub event_family: String,
    /// Dominant low-level event category represented by the entry.
    pub event_type: ChronicleEventType,
    /// Dominant influence or cause represented by the entry.
    pub cause: ChronicleEventCause,
    /// Feed-level fast-read narrative payload.
    pub headline: ChronicleHeadline,
    /// Expanded contextual narrative payload.
    pub capsule: ChronicleCapsule,
    /// Minimal deep-read anchor for future dossier work.
    pub dossier_stub: ChronicleDossierStub,
    /// Primary subject reference for bridge/UI consumers.
    pub entity_ref: ChronicleSubjectRefLite,
    /// Representative location reference.
    pub location_ref: ChronicleLocationRefLite,
    /// Significance score assigned during summarization.
    pub significance: f64,
    /// Attention category assigned during summarization.
    pub significance_category: ChronicleSignificanceCategory,
    /// Decomposed significance scoring for debug and tuning transparency.
    pub significance_meta: ChronicleSignificanceMeta,
    /// Current runtime queue bucket.
    pub queue_bucket: ChronicleQueueBucket,
    /// Current lifecycle status.
    pub status: ChronicleEntryStatus,
    /// Tick when the entry first surfaced in the visible queue.
    pub surfaced_tick: Option<u64>,
    /// Machine-readable reason for the most recent displacement, if any.
    pub displacement_reason: Option<String>,
    /// Ordered history of queue bucket transitions.
    pub queue_transitions: Vec<ChronicleQueueTransition>,
    /// Thread this entry belongs to, if any.
    pub thread_id: Option<ChronicleThreadId>,
}

impl ChronicleEntryLite {
    /// Validates minimal runtime invariants for one chronicle entry.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.entry_id.0 == 0 {
            return Err("entry_id must be non-zero");
        }
        if self.start_tick > self.end_tick {
            return Err("start_tick must be <= end_tick");
        }
        self.headline.validate()?;
        self.capsule.validate()?;
        self.dossier_stub.validate()?;
        if !self.queue_transitions.is_empty() {
            let first_transition = &self.queue_transitions[0];
            if first_transition.to == ChronicleQueueBucket::Dropped {
                return Err("queue_transitions must not transition into dropped");
            }
        }
        Ok(())
    }

    /// Returns `true` when this entry is attached to one entity.
    pub fn matches_entity(&self, entity_id: EntityId) -> bool {
        self.entity_ref.entity_id == Some(entity_id)
    }

    /// Builds the temporary legacy summary adapter for bridge/UI migration.
    pub fn to_legacy_summary(&self) -> ChronicleSummary {
        ChronicleSummary {
            start_tick: self.start_tick,
            end_tick: self.end_tick,
            entity_id: self.entity_ref.entity_id,
            event_type: self.event_type,
            cause: self.cause,
            title: self.headline.locale_key.clone(),
            description: self.capsule.locale_key.clone(),
            params: self.capsule.params.clone(),
            tile_x: self.location_ref.tile_x,
            tile_y: self.location_ref.tile_y,
            significance: self.significance,
            category: self.significance_category,
        }
    }
}

/// Temporary legacy summary adapter kept for bridge/UI migration.
///
/// This is not the canonical runtime chronicle unit anymore. Runtime authority now
/// lives in `ChronicleEntryLite`; `ChronicleSummary` remains only as a compatibility
/// shape while Godot/UI callers migrate off the summary-only contract.
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
    ///
    /// Temporary migration field derived from `ChronicleEntryLite.headline`.
    pub title: String,
    /// Locale key for the summary description.
    ///
    /// Temporary migration field derived from `ChronicleEntryLite.capsule`.
    pub description: String,
    /// Locale parameters for `description`.
    ///
    /// Temporary migration field derived from `ChronicleEntryLite.capsule.params`.
    pub params: BTreeMap<String, String>,
    /// Representative tile-space X location of the summary.
    pub tile_x: i32,
    /// Representative tile-space Y location of the summary.
    pub tile_y: i32,
    /// Significance score assigned during summarization.
    pub significance: f64,
    /// Attention category assigned during summarization.
    pub category: ChronicleSignificanceCategory,
}

/// Decomposed significance scoring for debug and tuning transparency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChronicleSignificanceMeta {
    /// Raw sum of event magnitudes before bonuses.
    pub base_score: f64,
    /// Cause-specific bonus applied on top of the base score.
    pub cause_bonus: f64,
    /// Social group-size bonus (0.0 for non-social entries).
    pub group_bonus: f64,
    /// Penalty applied for repeated event families in the suppression window.
    pub repeat_penalty: f64,
    /// Final computed significance score.
    pub final_score: f64,
    /// Machine-readable tags describing why this score was assigned.
    pub reason_tags: Vec<String>,
}

impl Default for ChronicleSignificanceMeta {
    fn default() -> Self {
        Self {
            base_score: 0.0,
            cause_bonus: 0.0,
            group_bonus: 0.0,
            repeat_penalty: 0.0,
            final_score: 0.0,
            reason_tags: Vec::new(),
        }
    }
}

/// One queue transition event in an entry lifecycle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChronicleQueueTransition {
    /// The queue bucket before this transition.
    pub from: ChronicleQueueBucket,
    /// The queue bucket after this transition.
    pub to: ChronicleQueueBucket,
    /// Tick when this transition occurred.
    pub tick: u64,
    /// Machine-readable reason for the transition.
    pub reason: String,
}

/// Runtime telemetry counters for chronicle routing diagnostics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ChronicleTelemetry {
    /// Total entries routed through `route_entry`.
    pub total_routed: u64,
    /// Entries surfaced to the visible queue.
    pub visible_count: u64,
    /// Entries routed to the background queue.
    pub background_count: u64,
    /// Entries routed to the recall queue.
    pub recall_count: u64,
    /// Entries dropped due to low significance or invalid state.
    pub drop_count: u64,
    /// Entries displaced from visible to recall.
    pub displacement_count: u64,
    /// Entries archived after active-queue eviction.
    pub archive_count: u64,
    /// Background entries promoted to visible after starvation.
    pub promotion_count: u64,
    /// Threads created during entry attachment.
    pub thread_create_count: u64,
    /// Threads evicted after becoming stale.
    pub thread_evict_count: u64,
}

/// Feed-ready chronicle item returned by the bridge snapshot family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChronicleFeedItemSnapshot {
    /// Stable entry identifier.
    pub entry_id: ChronicleEntryId,
    /// Future thread attachment placeholder.
    pub thread_id: Option<u64>,
    /// Dominant event type represented by the feed item.
    pub event_type: ChronicleEventType,
    /// Dominant cause represented by the feed item.
    pub cause: ChronicleEventCause,
    /// Current queue bucket for this entry.
    pub queue_bucket: ChronicleQueueBucket,
    /// Final significance category.
    pub category: ChronicleSignificanceCategory,
    /// Raw significance score retained for legacy consumers and debug surfaces.
    pub significance: f64,
    /// Inclusive first tick represented by the item.
    pub start_tick: u64,
    /// Inclusive last tick represented by the item.
    pub end_tick: u64,
    /// Feed-level headline payload.
    pub headline: ChronicleHeadline,
    /// Feed-level secondary contextual payload.
    pub capsule: ChronicleCapsule,
    /// Representative location of the entry.
    pub location_ref: ChronicleLocationRefLite,
    /// Primary subjects associated with the entry.
    pub primary_subjects: Vec<ChronicleSubjectRefLite>,
    /// Minimal render hint for feed consumers.
    pub render_hint: ChronicleFeedRenderHint,
}

/// Feed response returned by the chronicle snapshot family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChronicleFeedResponse {
    /// Snapshot revision shared by all items in this response.
    pub snapshot_revision: ChronicleSnapshotRevision,
    /// `true` when the requested revision is no longer available.
    pub revision_unavailable: bool,
    /// Feed-ready chronicle items.
    pub items: Vec<ChronicleFeedItemSnapshot>,
    /// Runtime telemetry counters captured in the same snapshot revision.
    pub telemetry: ChronicleTelemetry,
}

/// Entry detail response returned by the chronicle snapshot family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChronicleEntryDetailSnapshot {
    /// Snapshot revision shared by all returned data.
    pub snapshot_revision: ChronicleSnapshotRevision,
    /// `true` when the requested revision is no longer available.
    pub revision_unavailable: bool,
    /// The canonical entry payload, when found.
    pub entry: Option<ChronicleEntryLite>,
}

/// Recall slice item returned by the chronicle snapshot family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChronicleRecallItemSnapshot {
    /// Stable entry identifier.
    pub entry_id: ChronicleEntryId,
    /// Current queue bucket for the recalled entry.
    pub queue_bucket: ChronicleQueueBucket,
    /// Stable suppression reason token.
    pub suppression_reason: String,
    /// Tick at which the entry was suppressed or displaced.
    pub suppressed_tick: u64,
    /// Priority score used for recall ordering.
    pub recall_priority: f64,
    /// Feed-level headline payload.
    pub headline: ChronicleHeadline,
    /// Representative location of the entry.
    pub location_ref: ChronicleLocationRefLite,
    /// Dominant cause retained for filtering/debug.
    pub cause: ChronicleEventCause,
}

/// Recall queue response returned by the chronicle snapshot family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChronicleRecallSliceResponse {
    /// Snapshot revision shared by all returned data.
    pub snapshot_revision: ChronicleSnapshotRevision,
    /// `true` when the requested revision is no longer available.
    pub revision_unavailable: bool,
    /// Recalled chronicle items, newest first.
    pub items: Vec<ChronicleRecallItemSnapshot>,
}

/// Minimal thread snapshot returned by the chronicle snapshot family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChronicleThreadSnapshot {
    /// Stable thread identifier placeholder for future thread migration.
    pub thread_id: u64,
    /// Stable thread state identifier.
    pub state_id: String,
    /// Minimal feed-compatible headline for the thread.
    pub headline: ChronicleHeadline,
    /// Tension score placeholder for future thread ranking.
    pub tension_score: f64,
    /// Entry membership in chronological order.
    pub entry_ids: Vec<ChronicleEntryId>,
    /// Machine-readable scope identifier.
    pub scope: String,
    /// Tick when the thread started.
    pub started_tick: u64,
    /// Tick when the most recent entry was attached.
    pub last_entry_tick: u64,
    /// Number of chronicle entries currently attached to the thread.
    pub entry_count: usize,
}

/// Story-thread list response returned by the chronicle snapshot family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChronicleThreadListResponse {
    /// Snapshot revision shared by all returned data.
    pub snapshot_revision: ChronicleSnapshotRevision,
    /// `true` when the requested revision is no longer available.
    pub revision_unavailable: bool,
    /// Thread snapshots ordered by runtime tension score.
    pub items: Vec<ChronicleThreadSnapshot>,
}

/// History/archive response returned by the chronicle snapshot family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChronicleHistorySliceResponse {
    /// Snapshot revision shared by all returned data.
    pub snapshot_revision: ChronicleSnapshotRevision,
    /// `true` when the requested revision is no longer available.
    pub revision_unavailable: bool,
    /// Archive/history items, newest first.
    pub items: Vec<ChronicleFeedItemSnapshot>,
    /// Pagination cursor for the next request.
    pub next_cursor_before_tick: Option<u64>,
    /// Pagination cursor for stable same-tick pagination.
    pub next_cursor_before_entry_id: Option<ChronicleEntryId>,
    /// Runtime telemetry counters captured in the same snapshot revision.
    pub telemetry: ChronicleTelemetry,
}

/// Bounded recent world-history timeline built from raw chronicle events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronicleTimeline {
    visible_queue: VecDeque<ChronicleEntryLite>,
    background_queue: VecDeque<ChronicleEntryLite>,
    recall_queue: VecDeque<ChronicleEntryLite>,
    archive_queue: VecDeque<ChronicleEntryLite>,
    max_visible: usize,
    max_background: usize,
    max_recall: usize,
    max_archive: usize,
    last_visible_tick: Option<u64>,
    next_entry_id: u64,
    snapshot_revision: ChronicleSnapshotRevision,
    thread_registry: BTreeMap<ChronicleThreadKey, ChronicleThreadId>,
    threads: BTreeMap<ChronicleThreadId, ChronicleThread>,
    next_thread_id: u64,
    telemetry: ChronicleTelemetry,
}

impl ChronicleTimeline {
    /// Creates a new bounded chronicle timeline.
    pub fn new() -> Self {
        Self {
            visible_queue: VecDeque::with_capacity(config::CHRONICLE_VISIBLE_MAX_ENTRIES),
            background_queue: VecDeque::with_capacity(config::CHRONICLE_TIMELINE_MAX_ENTRIES),
            recall_queue: VecDeque::with_capacity(config::CHRONICLE_RECALL_MAX_ENTRIES),
            archive_queue: VecDeque::with_capacity(config::CHRONICLE_ARCHIVE_MAX_ENTRIES),
            max_visible: config::CHRONICLE_VISIBLE_MAX_ENTRIES,
            max_background: config::CHRONICLE_TIMELINE_MAX_ENTRIES,
            max_recall: config::CHRONICLE_RECALL_MAX_ENTRIES,
            max_archive: config::CHRONICLE_ARCHIVE_MAX_ENTRIES,
            last_visible_tick: None,
            next_entry_id: 1,
            snapshot_revision: ChronicleSnapshotRevision(1),
            thread_registry: BTreeMap::new(),
            threads: BTreeMap::new(),
            next_thread_id: 1,
            telemetry: ChronicleTelemetry::default(),
        }
    }

    /// Allocates the next stable chronicle entry identifier.
    pub fn allocate_entry_id(&mut self) -> ChronicleEntryId {
        let id = ChronicleEntryId::from_counter(self.next_entry_id);
        self.next_entry_id = self.next_entry_id.saturating_add(1).max(1);
        id
    }

    fn allocate_thread_id(&mut self) -> ChronicleThreadId {
        let id = ChronicleThreadId(self.next_thread_id);
        self.next_thread_id = self.next_thread_id.saturating_add(1).max(1);
        id
    }

    /// Routes one canonical entry through the attention budget.
    pub fn route_entry(
        &mut self,
        mut entry: ChronicleEntryLite,
        surfaced_tick: u64,
    ) -> ChronicleRouteResult {
        self.telemetry.total_routed = self.telemetry.total_routed.saturating_add(1);
        if let Err(reason) = entry.validate() {
            log::warn!("[Chronicle] invalid entry dropped: {reason}");
            self.telemetry.drop_count = self.telemetry.drop_count.saturating_add(1);
            self.bump_snapshot_revision();
            return ChronicleRouteResult::dropped();
        }

        match entry.significance_category {
            ChronicleSignificanceCategory::Critical => {
                self.bump_snapshot_revision();
                self.telemetry.visible_count = self.telemetry.visible_count.saturating_add(1);
                let thread_key = Self::derive_thread_key(&entry);
                let thread_id = self.attach_entry_to_thread(
                    thread_key,
                    entry.entry_id,
                    entry.end_tick,
                    &entry.headline,
                );
                entry.queue_bucket = ChronicleQueueBucket::Visible;
                entry.status = ChronicleEntryStatus::Published;
                entry.surfaced_tick = Some(surfaced_tick);
                entry.queue_transitions.push(ChronicleQueueTransition {
                    from: ChronicleQueueBucket::Dropped,
                    to: ChronicleQueueBucket::Visible,
                    tick: surfaced_tick,
                    reason: "routed_critical".to_string(),
                });
                entry.thread_id = Some(thread_id);
                let displaced = self.push_visible(entry, surfaced_tick);
                let displaced_visible = displaced.is_some();
                let mut pruned = false;
                if let Some(mut displaced_entry) = displaced {
                    self.telemetry.displacement_count =
                        self.telemetry.displacement_count.saturating_add(1);
                    self.telemetry.recall_count = self.telemetry.recall_count.saturating_add(1);
                    displaced_entry.displacement_reason =
                        Some("displaced_by_critical".to_string());
                    displaced_entry.queue_transitions.push(ChronicleQueueTransition {
                        from: ChronicleQueueBucket::Visible,
                        to: ChronicleQueueBucket::Recall,
                        tick: surfaced_tick,
                        reason: "displaced_by_critical".to_string(),
                    });
                    displaced_entry.queue_bucket = ChronicleQueueBucket::Recall;
                    displaced_entry.status = ChronicleEntryStatus::Published;
                    if let Some(evicted_entry) = self.push_recall(displaced_entry) {
                        let _ = self.archive_evicted_entry(
                            evicted_entry,
                            ChronicleQueueBucket::Recall,
                            surfaced_tick,
                            "evicted_from_recall",
                        );
                        pruned = true;
                    }
                }
                self.update_thread_states(surfaced_tick);
                ChronicleRouteResult {
                    queue: ChronicleQueueBucket::Visible,
                    pruned,
                    displaced_visible,
                    promoted_background: false,
                }
            }
            ChronicleSignificanceCategory::Major => {
                if self.has_visible_capacity() {
                    self.bump_snapshot_revision();
                    self.telemetry.visible_count = self.telemetry.visible_count.saturating_add(1);
                    let thread_key = Self::derive_thread_key(&entry);
                    let thread_id = self.attach_entry_to_thread(
                        thread_key,
                        entry.entry_id,
                        entry.end_tick,
                        &entry.headline,
                    );
                    entry.queue_bucket = ChronicleQueueBucket::Visible;
                    entry.status = ChronicleEntryStatus::Published;
                    entry.surfaced_tick = Some(surfaced_tick);
                    entry.queue_transitions.push(ChronicleQueueTransition {
                        from: ChronicleQueueBucket::Dropped,
                        to: ChronicleQueueBucket::Visible,
                        tick: surfaced_tick,
                        reason: "routed_major".to_string(),
                    });
                    entry.thread_id = Some(thread_id);
                    let displaced = self.push_visible(entry, surfaced_tick);
                    let displaced_visible = displaced.is_some();
                    let mut pruned = false;
                    if let Some(mut displaced_entry) = displaced {
                        self.telemetry.displacement_count =
                            self.telemetry.displacement_count.saturating_add(1);
                        self.telemetry.recall_count =
                            self.telemetry.recall_count.saturating_add(1);
                        displaced_entry.displacement_reason =
                            Some("displaced_by_major".to_string());
                        displaced_entry.queue_transitions.push(ChronicleQueueTransition {
                            from: ChronicleQueueBucket::Visible,
                            to: ChronicleQueueBucket::Recall,
                            tick: surfaced_tick,
                            reason: "displaced_by_major".to_string(),
                        });
                        displaced_entry.queue_bucket = ChronicleQueueBucket::Recall;
                        displaced_entry.status = ChronicleEntryStatus::Published;
                        if let Some(evicted_entry) = self.push_recall(displaced_entry) {
                            let _ = self.archive_evicted_entry(
                                evicted_entry,
                                ChronicleQueueBucket::Recall,
                                surfaced_tick,
                                "evicted_from_recall",
                            );
                            pruned = true;
                        }
                    }
                    self.update_thread_states(surfaced_tick);
                    ChronicleRouteResult {
                        queue: ChronicleQueueBucket::Visible,
                        pruned,
                        displaced_visible,
                        promoted_background: false,
                    }
                } else {
                    self.bump_snapshot_revision();
                    self.telemetry.recall_count = self.telemetry.recall_count.saturating_add(1);
                    let thread_key = Self::derive_thread_key(&entry);
                    let thread_id = self.attach_entry_to_thread(
                        thread_key,
                        entry.entry_id,
                        entry.end_tick,
                        &entry.headline,
                    );
                    entry.queue_bucket = ChronicleQueueBucket::Recall;
                    entry.status = ChronicleEntryStatus::Published;
                    entry.queue_transitions.push(ChronicleQueueTransition {
                        from: ChronicleQueueBucket::Dropped,
                        to: ChronicleQueueBucket::Recall,
                        tick: surfaced_tick,
                        reason: "routed_major_overflow".to_string(),
                    });
                    entry.thread_id = Some(thread_id);
                    let pruned = if let Some(evicted_entry) = self.push_recall(entry) {
                        let _ = self.archive_evicted_entry(
                            evicted_entry,
                            ChronicleQueueBucket::Recall,
                            surfaced_tick,
                            "evicted_from_recall",
                        );
                        true
                    } else {
                        false
                    };
                    self.update_thread_states(surfaced_tick);
                    ChronicleRouteResult {
                        queue: ChronicleQueueBucket::Recall,
                        pruned,
                        displaced_visible: false,
                        promoted_background: false,
                    }
                }
            }
            ChronicleSignificanceCategory::Notable => {
                self.bump_snapshot_revision();
                self.telemetry.background_count =
                    self.telemetry.background_count.saturating_add(1);
                let thread_key = Self::derive_thread_key(&entry);
                let thread_id = self.attach_entry_to_thread(
                    thread_key,
                    entry.entry_id,
                    entry.end_tick,
                    &entry.headline,
                );
                entry.queue_bucket = ChronicleQueueBucket::Background;
                entry.status = ChronicleEntryStatus::Published;
                entry.queue_transitions.push(ChronicleQueueTransition {
                    from: ChronicleQueueBucket::Dropped,
                    to: ChronicleQueueBucket::Background,
                    tick: surfaced_tick,
                    reason: "routed_notable".to_string(),
                });
                entry.thread_id = Some(thread_id);
                let pruned = if let Some(evicted_entry) = self.push_background(entry) {
                    let _ = self.archive_evicted_entry(
                        evicted_entry,
                        ChronicleQueueBucket::Background,
                        surfaced_tick,
                        "evicted_from_background",
                    );
                    true
                } else {
                    false
                };
                self.update_thread_states(surfaced_tick);
                ChronicleRouteResult {
                    queue: ChronicleQueueBucket::Background,
                    pruned,
                    displaced_visible: false,
                    promoted_background: false,
                }
            }
            ChronicleSignificanceCategory::Minor | ChronicleSignificanceCategory::Ignore => {
                self.bump_snapshot_revision();
                self.telemetry.drop_count = self.telemetry.drop_count.saturating_add(1);
                entry.queue_bucket = ChronicleQueueBucket::Dropped;
                entry.status = ChronicleEntryStatus::Suppressed;
                ChronicleRouteResult::dropped()
            }
        }
    }

    /// Promotes the highest-significance background entry when attention has been starved.
    pub fn promote_background_if_starved(
        &mut self,
        current_tick: u64,
    ) -> Option<ChronicleRouteResult> {
        let last_visible_tick = self.last_visible_tick.unwrap_or(0);
        if current_tick.saturating_sub(last_visible_tick)
            < config::CHRONICLE_VISIBLE_STARVATION_TICKS
            || self.background_queue.is_empty()
        {
            return None;
        }

        let best_index = self
            .background_queue
            .iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| {
                left.significance
                    .partial_cmp(&right.significance)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| left.end_tick.cmp(&right.end_tick))
            })
            .map(|(index, _)| index)?;
        let mut entry = self.background_queue.remove(best_index)?;
        self.bump_snapshot_revision();
        self.telemetry.promotion_count = self.telemetry.promotion_count.saturating_add(1);
        self.telemetry.visible_count = self.telemetry.visible_count.saturating_add(1);
        entry.queue_bucket = ChronicleQueueBucket::Visible;
        entry.status = ChronicleEntryStatus::Published;
        if entry.surfaced_tick.is_none() {
            entry.surfaced_tick = Some(current_tick);
        }
        entry.queue_transitions.push(ChronicleQueueTransition {
            from: ChronicleQueueBucket::Background,
            to: ChronicleQueueBucket::Visible,
            tick: current_tick,
            reason: "starvation_promotion".to_string(),
        });
        let displaced = self.push_visible(entry, current_tick);
        let displaced_visible = displaced.is_some();
        let mut pruned = false;
        if let Some(mut displaced_entry) = displaced {
            self.telemetry.displacement_count = self.telemetry.displacement_count.saturating_add(1);
            self.telemetry.recall_count = self.telemetry.recall_count.saturating_add(1);
            displaced_entry.displacement_reason =
                Some("displaced_by_background_promotion".to_string());
            displaced_entry.queue_transitions.push(ChronicleQueueTransition {
                from: ChronicleQueueBucket::Visible,
                to: ChronicleQueueBucket::Recall,
                tick: current_tick,
                reason: "displaced_by_background_promotion".to_string(),
            });
            displaced_entry.queue_bucket = ChronicleQueueBucket::Recall;
            displaced_entry.status = ChronicleEntryStatus::Published;
            if let Some(evicted_entry) = self.push_recall(displaced_entry) {
                let _ = self.archive_evicted_entry(
                    evicted_entry,
                    ChronicleQueueBucket::Recall,
                    current_tick,
                    "evicted_from_recall",
                );
                pruned = true;
            }
        }
        Some(ChronicleRouteResult {
            queue: ChronicleQueueBucket::Visible,
            pruned,
            displaced_visible,
            promoted_background: true,
        })
    }

    /// Returns recent canonical entries, newest first.
    pub fn recent_entries(&self, count: usize) -> Vec<&ChronicleEntryLite> {
        self.visible_queue.iter().rev().take(count).collect()
    }

    /// Returns a snapshot of runtime routing telemetry.
    pub fn telemetry_snapshot(&self) -> &ChronicleTelemetry {
        &self.telemetry
    }

    /// Returns the current monotonic snapshot revision for chronicle read endpoints.
    pub fn snapshot_revision(&self) -> ChronicleSnapshotRevision {
        self.snapshot_revision
    }

    /// Returns one feed snapshot response from the visible queue, newest first.
    pub fn feed_snapshot(
        &self,
        count: usize,
        requested_revision: Option<ChronicleSnapshotRevision>,
    ) -> ChronicleFeedResponse {
        if !self.is_revision_available(requested_revision) {
            return ChronicleFeedResponse {
                snapshot_revision: self.snapshot_revision,
                revision_unavailable: true,
                items: Vec::new(),
                telemetry: self.telemetry.clone(),
            };
        }

        ChronicleFeedResponse {
            snapshot_revision: self.snapshot_revision,
            revision_unavailable: false,
            items: self
                .visible_queue
                .iter()
                .rev()
                .take(count)
                .map(Self::feed_item_from_entry)
                .collect(),
            telemetry: self.telemetry.clone(),
        }
    }

    /// Returns one full chronicle entry detail snapshot when present.
    pub fn entry_detail_snapshot(
        &self,
        entry_id: ChronicleEntryId,
        requested_revision: Option<ChronicleSnapshotRevision>,
    ) -> ChronicleEntryDetailSnapshot {
        if !self.is_revision_available(requested_revision) {
            return ChronicleEntryDetailSnapshot {
                snapshot_revision: self.snapshot_revision,
                revision_unavailable: true,
                entry: None,
            };
        }

        ChronicleEntryDetailSnapshot {
            snapshot_revision: self.snapshot_revision,
            revision_unavailable: false,
            entry: self.find_entry(entry_id).cloned(),
        }
    }

    /// Returns the current recall slice, newest first.
    pub fn recall_slice_snapshot(
        &self,
        count: usize,
        requested_revision: Option<ChronicleSnapshotRevision>,
    ) -> ChronicleRecallSliceResponse {
        if !self.is_revision_available(requested_revision) {
            return ChronicleRecallSliceResponse {
                snapshot_revision: self.snapshot_revision,
                revision_unavailable: true,
                items: Vec::new(),
            };
        }

        ChronicleRecallSliceResponse {
            snapshot_revision: self.snapshot_revision,
            revision_unavailable: false,
            items: self
                .recall_queue
                .iter()
                .rev()
                .take(count)
                .map(Self::recall_item_from_entry)
                .collect(),
        }
    }

    /// Returns the current history slice from the cold archive queue.
    pub fn history_slice_snapshot(
        &self,
        count: usize,
        cursor_before_tick: Option<u64>,
        cursor_before_entry_id: Option<ChronicleEntryId>,
        requested_revision: Option<ChronicleSnapshotRevision>,
    ) -> ChronicleHistorySliceResponse {
        if !self.is_revision_available(requested_revision) {
            return ChronicleHistorySliceResponse {
                snapshot_revision: self.snapshot_revision,
                revision_unavailable: true,
                items: Vec::new(),
                next_cursor_before_tick: None,
                next_cursor_before_entry_id: None,
                telemetry: self.telemetry.clone(),
            };
        }

        let items: Vec<ChronicleFeedItemSnapshot> = self
            .archive_queue
            .iter()
            .rev()
            .filter(|entry| {
                if let Some(before_tick) = cursor_before_tick {
                    if entry.end_tick > before_tick {
                        return false;
                    }
                    if entry.end_tick == before_tick {
                        if let Some(before_id) = cursor_before_entry_id {
                            return entry.entry_id < before_id;
                        }
                    }
                }
                true
            })
            .take(count)
            .map(Self::feed_item_from_entry)
            .collect();

        let next_cursor_before_tick = items.last().map(|item| item.end_tick);
        let next_cursor_before_entry_id = items.last().map(|item| item.entry_id);

        ChronicleHistorySliceResponse {
            snapshot_revision: self.snapshot_revision,
            revision_unavailable: false,
            items,
            next_cursor_before_tick,
            next_cursor_before_entry_id,
            telemetry: self.telemetry.clone(),
        }
    }

    /// Returns the current story-thread list.
    ///
    /// Thread snapshots are intentionally empty until Chronicle thread migration lands.
    pub fn story_threads_snapshot(
        &self,
        count: usize,
        requested_revision: Option<ChronicleSnapshotRevision>,
    ) -> ChronicleThreadListResponse {
        if !self.is_revision_available(requested_revision) {
            return ChronicleThreadListResponse {
                snapshot_revision: self.snapshot_revision,
                revision_unavailable: true,
                items: Vec::new(),
            };
        }

        let mut sorted_threads: Vec<&ChronicleThread> = self.threads.values().collect();
        sorted_threads.sort_by(|left, right| {
            right
                .tension_score
                .partial_cmp(&left.tension_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        ChronicleThreadListResponse {
            snapshot_revision: self.snapshot_revision,
            revision_unavailable: false,
            items: sorted_threads
                .into_iter()
                .take(count)
                .map(|thread| ChronicleThreadSnapshot {
                    thread_id: thread.thread_id.0,
                    state_id: thread.state.id().to_string(),
                    headline: thread.headline.clone(),
                    tension_score: thread.tension_score,
                    entry_ids: thread.entry_ids.clone(),
                    scope: thread.key.scope.to_id_string(),
                    started_tick: thread.started_tick,
                    last_entry_tick: thread.last_entry_tick,
                    entry_count: thread.entry_ids.len(),
                })
                .collect(),
        }
    }

    /// Returns recent canonical entries for one entity across all queues, newest first.
    pub fn query_entries_by_entity(
        &self,
        entity_id: EntityId,
        count: usize,
    ) -> Vec<&ChronicleEntryLite> {
        let mut entries: Vec<&ChronicleEntryLite> = self
            .visible_queue
            .iter()
            .chain(self.background_queue.iter())
            .chain(self.recall_queue.iter())
            .chain(self.archive_queue.iter())
            .filter(|entry| entry.matches_entity(entity_id))
            .collect();
        entries.sort_by(|left, right| right.end_tick.cmp(&left.end_tick));
        entries.truncate(count);
        entries
    }

    /// Returns the current number of visible entries.
    pub fn visible_len(&self) -> usize {
        self.visible_queue.len()
    }

    /// Returns the current number of background entries.
    pub fn background_len(&self) -> usize {
        self.background_queue.len()
    }

    /// Returns the current number of recall entries.
    pub fn recall_len(&self) -> usize {
        self.recall_queue.len()
    }

    /// Returns the current number of archived entries.
    pub fn archive_len(&self) -> usize {
        self.archive_queue.len()
    }

    /// Returns `true` when another entry can be surfaced immediately.
    pub fn has_visible_capacity(&self) -> bool {
        self.visible_queue.len() < self.max_visible
    }

    /// Returns the tick at which the most recent visible entry was surfaced.
    pub fn last_visible_tick(&self) -> Option<u64> {
        self.last_visible_tick
    }

    /// Returns how many entries of one event family were seen since `since_tick`.
    pub fn recent_family_count(
        &self,
        event_type: ChronicleEventType,
        cause: ChronicleEventCause,
        since_tick: u64,
    ) -> usize {
        self.visible_queue
            .iter()
            .chain(self.background_queue.iter())
            .chain(self.recall_queue.iter())
            .chain(self.archive_queue.iter())
            .filter(|entry| {
                entry.end_tick >= since_tick
                    && entry.event_type == event_type
                    && entry.cause == cause
            })
            .count()
    }

    /// Clears all stored chronicle entries.
    pub fn clear(&mut self) {
        self.bump_snapshot_revision();
        self.visible_queue.clear();
        self.background_queue.clear();
        self.recall_queue.clear();
        self.archive_queue.clear();
        self.last_visible_tick = None;
        self.next_entry_id = 1;
        self.thread_registry.clear();
        self.threads.clear();
        self.next_thread_id = 1;
        self.telemetry = ChronicleTelemetry::default();
    }

    /// Returns the total number of stored entries across all queues.
    pub fn len(&self) -> usize {
        self.visible_queue.len()
            + self.background_queue.len()
            + self.recall_queue.len()
            + self.archive_queue.len()
    }

    /// Returns `true` when no entries are currently stored.
    pub fn is_empty(&self) -> bool {
        self.visible_queue.is_empty()
            && self.background_queue.is_empty()
            && self.recall_queue.is_empty()
            && self.archive_queue.is_empty()
    }

    fn push_visible(
        &mut self,
        entry: ChronicleEntryLite,
        surfaced_tick: u64,
    ) -> Option<ChronicleEntryLite> {
        let displaced = if self.visible_queue.len() >= self.max_visible {
            self.visible_queue.pop_front()
        } else {
            None
        };
        self.last_visible_tick = Some(surfaced_tick);
        self.visible_queue.push_back(entry);
        displaced
    }

    fn push_background(&mut self, entry: ChronicleEntryLite) -> Option<ChronicleEntryLite> {
        let pruned = if self.background_queue.len() >= self.max_background {
            self.background_queue.pop_front()
        } else {
            None
        };
        self.background_queue.push_back(entry);
        pruned
    }

    fn push_recall(&mut self, entry: ChronicleEntryLite) -> Option<ChronicleEntryLite> {
        let pruned = if self.recall_queue.len() >= self.max_recall {
            self.recall_queue.pop_front()
        } else {
            None
        };
        self.recall_queue.push_back(entry);
        pruned
    }

    fn push_archive(&mut self, mut entry: ChronicleEntryLite) -> Option<ChronicleEntryLite> {
        entry.status = ChronicleEntryStatus::Archived;
        entry.queue_bucket = ChronicleQueueBucket::Dropped;
        self.telemetry.archive_count = self.telemetry.archive_count.saturating_add(1);
        let pruned = if self.archive_queue.len() >= self.max_archive {
            self.archive_queue.pop_front()
        } else {
            None
        };
        self.archive_queue.push_back(entry);
        pruned
    }

    fn archive_evicted_entry(
        &mut self,
        mut entry: ChronicleEntryLite,
        from: ChronicleQueueBucket,
        tick: u64,
        reason: &str,
    ) -> bool {
        entry.displacement_reason = Some(reason.to_string());
        entry.queue_transitions.push(ChronicleQueueTransition {
            from,
            to: ChronicleQueueBucket::Dropped,
            tick,
            reason: reason.to_string(),
        });
        self.push_archive(entry).is_some()
    }

    fn is_revision_available(&self, requested_revision: Option<ChronicleSnapshotRevision>) -> bool {
        requested_revision
            .map(|revision| revision == self.snapshot_revision)
            .unwrap_or(true)
    }

    fn bump_snapshot_revision(&mut self) {
        self.snapshot_revision = self.snapshot_revision.next();
    }

    fn find_entry(&self, entry_id: ChronicleEntryId) -> Option<&ChronicleEntryLite> {
        self.visible_queue
            .iter()
            .chain(self.background_queue.iter())
            .chain(self.recall_queue.iter())
            .chain(self.archive_queue.iter())
            .find(|entry| entry.entry_id == entry_id)
    }

    /// Validates internal queue/thread integrity and returns human-readable issues.
    pub fn validate_integrity(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if self.visible_queue.len() > self.max_visible {
            issues.push(format!(
                "visible_queue overflow: {} > {}",
                self.visible_queue.len(),
                self.max_visible
            ));
        }
        if self.background_queue.len() > self.max_background {
            issues.push(format!(
                "background_queue overflow: {} > {}",
                self.background_queue.len(),
                self.max_background
            ));
        }
        if self.recall_queue.len() > self.max_recall {
            issues.push(format!(
                "recall_queue overflow: {} > {}",
                self.recall_queue.len(),
                self.max_recall
            ));
        }
        if self.archive_queue.len() > self.max_archive {
            issues.push(format!(
                "archive_queue overflow: {} > {}",
                self.archive_queue.len(),
                self.max_archive
            ));
        }

        let mut seen_ids = BTreeSet::new();
        for entry in self
            .visible_queue
            .iter()
            .chain(self.background_queue.iter())
            .chain(self.recall_queue.iter())
            .chain(self.archive_queue.iter())
        {
            if !seen_ids.insert(entry.entry_id) {
                issues.push(format!("duplicate entry_id: {:?}", entry.entry_id));
            }
        }

        for (thread_id, thread) in &self.threads {
            for &entry_id in &thread.entry_ids {
                if !seen_ids.contains(&entry_id) {
                    issues.push(format!(
                        "thread {:?} references orphaned entry_id {:?}",
                        thread_id, entry_id
                    ));
                }
            }
        }

        for (key, thread_id) in &self.thread_registry {
            if !self.threads.contains_key(thread_id) {
                issues.push(format!(
                    "thread_registry references missing thread {:?} for key {}",
                    thread_id, key.event_family
                ));
            }
        }

        issues
    }

    /// Returns aggregate significance statistics grouped by category for debug tuning.
    pub fn significance_debug_summary(&self) -> BTreeMap<String, (usize, f64, f64, f64)> {
        let mut buckets: BTreeMap<String, (usize, f64, f64, f64)> = BTreeMap::new();
        for entry in self
            .visible_queue
            .iter()
            .chain(self.background_queue.iter())
            .chain(self.recall_queue.iter())
            .chain(self.archive_queue.iter())
        {
            let key = entry.significance_category.id().to_string();
            let bucket = buckets
                .entry(key)
                .or_insert((0, f64::INFINITY, f64::NEG_INFINITY, 0.0));
            bucket.0 += 1;
            bucket.1 = bucket.1.min(entry.significance);
            bucket.2 = bucket.2.max(entry.significance);
            bucket.3 += entry.significance;
        }

        for bucket in buckets.values_mut() {
            if bucket.0 > 0 {
                bucket.3 /= bucket.0 as f64;
            } else {
                bucket.1 = 0.0;
                bucket.2 = 0.0;
            }
        }

        buckets
    }

    fn feed_item_from_entry(entry: &ChronicleEntryLite) -> ChronicleFeedItemSnapshot {
        ChronicleFeedItemSnapshot {
            entry_id: entry.entry_id,
            thread_id: entry.thread_id.map(|thread_id| thread_id.0),
            event_type: entry.event_type,
            cause: entry.cause,
            queue_bucket: entry.queue_bucket,
            category: entry.significance_category,
            significance: entry.significance,
            start_tick: entry.start_tick,
            end_tick: entry.end_tick,
            headline: entry.headline.clone(),
            capsule: entry.capsule.clone(),
            location_ref: entry.location_ref.clone(),
            primary_subjects: vec![entry.entity_ref.clone()],
            render_hint: ChronicleFeedRenderHint {
                icon_id: entry.cause.id().to_string(),
                color_token: entry.cause.id().to_string(),
            },
        }
    }

    fn recall_item_from_entry(entry: &ChronicleEntryLite) -> ChronicleRecallItemSnapshot {
        ChronicleRecallItemSnapshot {
            entry_id: entry.entry_id,
            queue_bucket: entry.queue_bucket,
            suppression_reason: entry
                .displacement_reason
                .clone()
                .unwrap_or_else(|| "attention_budget".to_string()),
            suppressed_tick: entry.surfaced_tick.unwrap_or(entry.end_tick),
            recall_priority: entry.significance,
            headline: entry.headline.clone(),
            location_ref: entry.location_ref.clone(),
            cause: entry.cause,
        }
    }

    fn derive_thread_key(entry: &ChronicleEntryLite) -> ChronicleThreadKey {
        let scope = if let Some(entity_id) = entry.entity_ref.entity_id {
            ChronicleThreadScope::Agent(entity_id)
        } else if entry.location_ref.tile_x != 0 || entry.location_ref.tile_y != 0 {
            ChronicleThreadScope::Location(
                entry.location_ref.tile_x / config::CHRONICLE_THREAD_LOCATION_BUCKET_SIZE,
                entry.location_ref.tile_y / config::CHRONICLE_THREAD_LOCATION_BUCKET_SIZE,
            )
        } else {
            ChronicleThreadScope::Global
        };
        ChronicleThreadKey {
            event_family: entry.event_family.clone(),
            scope,
        }
    }

    fn attach_entry_to_thread(
        &mut self,
        key: ChronicleThreadKey,
        entry_id: ChronicleEntryId,
        tick: u64,
        headline: &ChronicleHeadline,
    ) -> ChronicleThreadId {
        if let Some(&thread_id) = self.thread_registry.get(&key) {
            let entry_ids = if let Some(thread) = self.threads.get_mut(&thread_id) {
                thread.entry_ids.push(entry_id);
                thread.last_entry_tick = tick;
                thread.headline = headline.clone();
                thread.state = if thread.entry_ids.len() >= 2 {
                    ChronicleThreadState::Developing
                } else {
                    ChronicleThreadState::Emerging
                };
                thread.entry_ids.clone()
            } else {
                Vec::new()
            };
            let mut tension_score = self.compute_tension_for_thread_entries(&entry_ids, tick);
            if self.find_entry(entry_id).is_none() {
                tension_score += 1.0;
            }
            if let Some(thread) = self.threads.get_mut(&thread_id) {
                thread.tension_score = tension_score;
            }
            return thread_id;
        }

        let thread_id = self.allocate_thread_id();
        let thread = ChronicleThread {
            thread_id,
            key: key.clone(),
            state: ChronicleThreadState::Emerging,
            started_tick: tick,
            last_entry_tick: tick,
            entry_ids: vec![entry_id],
            tension_score: 1.0,
            headline: headline.clone(),
        };
        self.thread_registry.insert(key, thread_id);
        self.threads.insert(thread_id, thread);
        self.telemetry.thread_create_count = self.telemetry.thread_create_count.saturating_add(1);
        self.evict_stale_threads_if_needed(tick);
        thread_id
    }

    fn compute_tension_for_thread_entries(
        &self,
        entry_ids: &[ChronicleEntryId],
        current_tick: u64,
    ) -> f64 {
        let half_life = config::CHRONICLE_THREAD_TENSION_HALF_LIFE_TICKS;
        if half_life <= 0.0 {
            return entry_ids.len() as f64;
        }

        let mut tension = 0.0_f64;
        for &entry_id in entry_ids {
            if let Some(entry) = self.find_entry(entry_id) {
                let age = current_tick.saturating_sub(entry.end_tick) as f64;
                tension += 0.5_f64.powf(age / half_life);
            }
        }
        tension
    }

    fn update_thread_states(&mut self, current_tick: u64) {
        let updates: Vec<(ChronicleThreadId, f64, ChronicleThreadState)> = self
            .threads
            .iter()
            .map(|(&thread_id, thread)| {
                let tension_score =
                    self.compute_tension_for_thread_entries(&thread.entry_ids, current_tick);
                let state = match thread.state {
                    ChronicleThreadState::Developing | ChronicleThreadState::Emerging => {
                        if current_tick.saturating_sub(thread.last_entry_tick)
                            >= config::CHRONICLE_THREAD_RESOLVE_WINDOW_TICKS
                        {
                            ChronicleThreadState::Resolved
                        } else if thread.entry_ids.len() >= 2 {
                            ChronicleThreadState::Developing
                        } else {
                            ChronicleThreadState::Emerging
                        }
                    }
                    ChronicleThreadState::Resolved => ChronicleThreadState::Resolved,
                };
                (thread_id, tension_score, state)
            })
            .collect();

        for (thread_id, tension_score, state) in updates {
            if let Some(thread) = self.threads.get_mut(&thread_id) {
                thread.tension_score = tension_score;
                thread.state = state;
            }
        }
    }

    fn evict_stale_threads_if_needed(&mut self, current_tick: u64) {
        while self.threads.len() > config::CHRONICLE_THREAD_MAX_ACTIVE {
            let evict_candidate = self
                .threads
                .iter()
                .filter(|(_, thread)| thread.state == ChronicleThreadState::Resolved)
                .map(|(&thread_id, thread)| {
                    (
                        thread_id,
                        self.compute_tension_for_thread_entries(&thread.entry_ids, current_tick),
                        thread.key.clone(),
                    )
                })
                .min_by(|left, right| {
                    left.1
                        .partial_cmp(&right.1)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

            let Some((thread_id, tension_score, key)) = evict_candidate else {
                break;
            };
            if tension_score >= config::CHRONICLE_THREAD_EVICTION_TENSION_THRESHOLD {
                break;
            }

            self.threads.remove(&thread_id);
            self.thread_registry.remove(&key);
            self.clear_thread_id_on_entries(thread_id);
            self.telemetry.thread_evict_count = self.telemetry.thread_evict_count.saturating_add(1);
        }
    }

    fn clear_thread_id_on_entries(&mut self, thread_id: ChronicleThreadId) {
        for entry in self
            .visible_queue
            .iter_mut()
            .chain(self.background_queue.iter_mut())
            .chain(self.recall_queue.iter_mut())
            .chain(self.archive_queue.iter_mut())
        {
            if entry.thread_id == Some(thread_id) {
                entry.thread_id = None;
            }
        }
    }
}

impl Default for ChronicleTimeline {
    fn default() -> Self {
        Self::new()
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
            summary_params: BTreeMap::new(),
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
        assert_eq!(
            log.latest_for_entity(high_entity).map(|event| event.tick),
            Some(30)
        );
    }

    fn sample_entry(
        timeline: &mut ChronicleTimeline,
        tick: u64,
        entity_id: Option<EntityId>,
        event_type: ChronicleEventType,
        cause: ChronicleEventCause,
        significance: f64,
        significance_category: ChronicleSignificanceCategory,
    ) -> ChronicleEntryLite {
        ChronicleEntryLite {
            entry_id: timeline.allocate_entry_id(),
            start_tick: tick,
            end_tick: tick,
            event_family: format!("chronicle.{}.{}", event_type as u8, cause.id()),
            event_type,
            cause,
            headline: ChronicleHeadline {
                locale_key: "CHRONICLE_TITLE".to_string(),
                params: BTreeMap::new(),
            },
            capsule: ChronicleCapsule {
                locale_key: "CHRONICLE_CAPSULE".to_string(),
                params: BTreeMap::new(),
            },
            dossier_stub: ChronicleDossierStub {
                locale_key: "CHRONICLE_DOSSIER_STUB".to_string(),
                params: BTreeMap::new(),
                detail_tags: vec!["debug".to_string()],
            },
            entity_ref: ChronicleSubjectRefLite {
                entity_id,
                display_name: None,
                ref_state: entity_id
                    .map(|_| ChronicleEntityRefState::Alive)
                    .unwrap_or(ChronicleEntityRefState::Unknown),
            },
            location_ref: ChronicleLocationRefLite {
                tile_x: 0,
                tile_y: 0,
                region_label: None,
            },
            significance,
            significance_category,
            significance_meta: ChronicleSignificanceMeta {
                base_score: significance,
                cause_bonus: 0.0,
                group_bonus: 0.0,
                repeat_penalty: 0.0,
                final_score: significance,
                reason_tags: vec!["test".to_string()],
            },
            queue_bucket: ChronicleQueueBucket::Dropped,
            status: ChronicleEntryStatus::Pending,
            surfaced_tick: None,
            displacement_reason: None,
            queue_transitions: Vec::new(),
            thread_id: None,
        }
    }

    #[test]
    fn chronicle_entry_lite_validation_rejects_invalid_ranges() {
        let mut timeline = ChronicleTimeline::new();
        let mut entry = sample_entry(
            &mut timeline,
            10,
            Some(EntityId(1)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        entry.start_tick = 11;

        assert_eq!(entry.validate(), Err("start_tick must be <= end_tick"));
    }

    #[test]
    fn chronicle_entry_lite_validation_rejects_empty_dossier_stub_locale_key() {
        let mut timeline = ChronicleTimeline::new();
        let mut entry = sample_entry(
            &mut timeline,
            10,
            Some(EntityId(1)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        entry.dossier_stub.locale_key.clear();

        assert_eq!(
            entry.validate(),
            Err("dossier_stub locale_key must not be empty")
        );
    }

    #[test]
    fn chronicle_timeline_keeps_recent_entries_bounded() {
        let mut timeline = ChronicleTimeline::new();
        for index in 0..(config::CHRONICLE_VISIBLE_MAX_ENTRIES + 5) {
            let entry = sample_entry(
                &mut timeline,
                index as u64,
                Some(EntityId(index as u64)),
                ChronicleEventType::InfluenceAttraction,
                ChronicleEventCause::Food,
                8.0,
                ChronicleSignificanceCategory::Critical,
            );
            let _ = timeline.route_entry(entry, index as u64);
        }

        assert_eq!(
            timeline.visible_len(),
            config::CHRONICLE_VISIBLE_MAX_ENTRIES
        );
        assert_eq!(
            timeline
                .recent_entries(1)
                .first()
                .and_then(|entry| entry.entity_ref.entity_id),
            Some(EntityId((config::CHRONICLE_VISIBLE_MAX_ENTRIES + 4) as u64))
        );
    }

    #[test]
    fn chronicle_timeline_queries_recent_entries_by_entity() {
        let mut timeline = ChronicleTimeline::new();
        let entity_id = EntityId(44);
        for tick in 0..3_u64 {
            let entry = ChronicleEntryLite {
                location_ref: ChronicleLocationRefLite {
                    tile_x: 1,
                    tile_y: 2,
                    region_label: None,
                },
                ..sample_entry(
                    &mut timeline,
                    tick,
                    Some(entity_id),
                    ChronicleEventType::ShelterSeeking,
                    ChronicleEventCause::Warmth,
                    7.5,
                    ChronicleSignificanceCategory::Major,
                )
            };
            let _ = timeline.route_entry(entry, tick);
        }

        let entries = timeline.query_entries_by_entity(entity_id, 2);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].start_tick, 2);
        assert_eq!(entries[1].start_tick, 1);
    }

    #[test]
    fn chronicle_timeline_routes_notable_to_background_and_major_to_recall_when_full() {
        let mut timeline = ChronicleTimeline::new();
        for index in 0..config::CHRONICLE_VISIBLE_MAX_ENTRIES {
            let entry = sample_entry(
                &mut timeline,
                index as u64,
                Some(EntityId(index as u64)),
                ChronicleEventType::InfluenceAttraction,
                ChronicleEventCause::Food,
                7.0,
                ChronicleSignificanceCategory::Major,
            );
            let _ = timeline.route_entry(entry, index as u64);
        }

        let notable_entry = sample_entry(
            &mut timeline,
            99,
            Some(EntityId(99)),
            ChronicleEventType::GatheringFormation,
            ChronicleEventCause::Social,
            5.0,
            ChronicleSignificanceCategory::Notable,
        );
        let notable = timeline.route_entry(notable_entry, 99);
        let major_entry = ChronicleEntryLite {
            location_ref: ChronicleLocationRefLite {
                tile_x: 1,
                tile_y: 1,
                region_label: None,
            },
            ..sample_entry(
                &mut timeline,
                100,
                Some(EntityId(100)),
                ChronicleEventType::ShelterSeeking,
                ChronicleEventCause::Warmth,
                8.0,
                ChronicleSignificanceCategory::Major,
            )
        };
        let major = timeline.route_entry(major_entry, 100);

        assert_eq!(notable.queue, ChronicleQueueBucket::Background);
        assert_eq!(major.queue, ChronicleQueueBucket::Recall);
        assert_eq!(
            timeline.visible_len(),
            config::CHRONICLE_VISIBLE_MAX_ENTRIES
        );
        assert_eq!(timeline.background_len(), 1);
        assert_eq!(timeline.recall_len(), 1);
    }

    #[test]
    fn chronicle_timeline_snapshot_revision_advances_on_mutation() {
        let mut timeline = ChronicleTimeline::new();
        let initial_revision = timeline.snapshot_revision();
        let entry = sample_entry(
            &mut timeline,
            1,
            Some(EntityId(1)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        let _ = timeline.route_entry(entry, 1);

        assert!(timeline.snapshot_revision().0 > initial_revision.0);
    }

    #[test]
    fn chronicle_timeline_feed_snapshot_returns_newest_visible_entries() {
        let mut timeline = ChronicleTimeline::new();
        for tick in 0..3_u64 {
            let entry = sample_entry(
                &mut timeline,
                tick,
                Some(EntityId(10 + tick)),
                ChronicleEventType::InfluenceAttraction,
                ChronicleEventCause::Food,
                8.0,
                ChronicleSignificanceCategory::Major,
            );
            let _ = timeline.route_entry(entry, tick);
        }

        let response = timeline.feed_snapshot(2, Some(timeline.snapshot_revision()));
        assert!(!response.revision_unavailable);
        assert_eq!(response.items.len(), 2);
        assert_eq!(response.items[0].end_tick, 2);
        assert_eq!(response.items[1].end_tick, 1);
    }

    #[test]
    fn chronicle_timeline_entry_detail_snapshot_returns_matching_entry() {
        let mut timeline = ChronicleTimeline::new();
        let entry = sample_entry(
            &mut timeline,
            12,
            Some(EntityId(2)),
            ChronicleEventType::ShelterSeeking,
            ChronicleEventCause::Warmth,
            7.0,
            ChronicleSignificanceCategory::Major,
        );
        let entry_id = entry.entry_id;
        let _ = timeline.route_entry(entry, 12);

        let detail = timeline.entry_detail_snapshot(entry_id, Some(timeline.snapshot_revision()));
        assert!(!detail.revision_unavailable);
        assert_eq!(
            detail.entry.as_ref().map(|entry| entry.entry_id),
            Some(entry_id)
        );
    }

    #[test]
    fn chronicle_timeline_recall_slice_reports_recalled_entries() {
        let mut timeline = ChronicleTimeline::new();
        for index in 0..config::CHRONICLE_VISIBLE_MAX_ENTRIES {
            let entry = sample_entry(
                &mut timeline,
                index as u64,
                Some(EntityId(index as u64)),
                ChronicleEventType::InfluenceAttraction,
                ChronicleEventCause::Food,
                8.0,
                ChronicleSignificanceCategory::Major,
            );
            let _ = timeline.route_entry(entry, index as u64);
        }
        let recall_entry = sample_entry(
            &mut timeline,
            99,
            Some(EntityId(999)),
            ChronicleEventType::InfluenceAvoidance,
            ChronicleEventCause::Danger,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        let _ = timeline.route_entry(recall_entry, 99);

        let recall = timeline.recall_slice_snapshot(4, Some(timeline.snapshot_revision()));
        assert!(!recall.revision_unavailable);
        assert_eq!(recall.items.len(), 1);
        assert_eq!(recall.items[0].queue_bucket, ChronicleQueueBucket::Recall);
    }

    #[test]
    fn chronicle_archive_captures_evicted_recall_entries() {
        let mut timeline = ChronicleTimeline::new();
        timeline.max_visible = 1;
        timeline.max_recall = 1;

        let first = sample_entry(
            &mut timeline,
            1,
            Some(EntityId(1)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            7.0,
            ChronicleSignificanceCategory::Major,
        );
        let _ = timeline.route_entry(first, 1);

        let second = sample_entry(
            &mut timeline,
            2,
            Some(EntityId(2)),
            ChronicleEventType::InfluenceAvoidance,
            ChronicleEventCause::Danger,
            7.0,
            ChronicleSignificanceCategory::Major,
        );
        let second_id = second.entry_id;
        let _ = timeline.route_entry(second, 2);

        let third = sample_entry(
            &mut timeline,
            3,
            Some(EntityId(3)),
            ChronicleEventType::ShelterSeeking,
            ChronicleEventCause::Warmth,
            7.0,
            ChronicleSignificanceCategory::Major,
        );
        let _ = timeline.route_entry(third, 3);

        assert_eq!(timeline.recall_len(), 1);
        assert_eq!(timeline.archive_len(), 1);
        let archived = timeline.find_entry(second_id).expect("archived entry");
        assert_eq!(archived.status, ChronicleEntryStatus::Archived);
        assert_eq!(
            archived.displacement_reason.as_deref(),
            Some("evicted_from_recall")
        );
    }

    #[test]
    fn chronicle_archive_captures_evicted_background_entries() {
        let mut timeline = ChronicleTimeline::new();
        timeline.max_background = 1;

        let first = sample_entry(
            &mut timeline,
            1,
            Some(EntityId(10)),
            ChronicleEventType::GatheringFormation,
            ChronicleEventCause::Social,
            5.0,
            ChronicleSignificanceCategory::Notable,
        );
        let first_id = first.entry_id;
        let _ = timeline.route_entry(first, 1);

        let second = sample_entry(
            &mut timeline,
            2,
            Some(EntityId(11)),
            ChronicleEventType::GatheringFormation,
            ChronicleEventCause::Social,
            5.0,
            ChronicleSignificanceCategory::Notable,
        );
        let _ = timeline.route_entry(second, 2);

        assert_eq!(timeline.background_len(), 1);
        assert_eq!(timeline.archive_len(), 1);
        let archived = timeline.find_entry(first_id).expect("archived entry");
        assert_eq!(archived.status, ChronicleEntryStatus::Archived);
        assert_eq!(
            archived.displacement_reason.as_deref(),
            Some("evicted_from_background")
        );
    }

    #[test]
    fn chronicle_archive_bounded_by_max() {
        let mut timeline = ChronicleTimeline::new();
        timeline.max_background = 1;
        timeline.max_archive = 2;

        for tick in 1..=5_u64 {
            let entry = sample_entry(
                &mut timeline,
                tick,
                Some(EntityId(tick)),
                ChronicleEventType::GatheringFormation,
                ChronicleEventCause::Social,
                5.0,
                ChronicleSignificanceCategory::Notable,
            );
            let _ = timeline.route_entry(entry, tick);
        }

        assert_eq!(timeline.archive_len(), 2);
    }

    #[test]
    fn chronicle_archive_entries_have_archived_status() {
        let mut timeline = ChronicleTimeline::new();
        timeline.max_background = 1;

        let first = sample_entry(
            &mut timeline,
            1,
            Some(EntityId(20)),
            ChronicleEventType::GatheringFormation,
            ChronicleEventCause::Social,
            5.0,
            ChronicleSignificanceCategory::Notable,
        );
        let _ = timeline.route_entry(first, 1);

        let second = sample_entry(
            &mut timeline,
            2,
            Some(EntityId(21)),
            ChronicleEventType::GatheringFormation,
            ChronicleEventCause::Social,
            5.0,
            ChronicleSignificanceCategory::Notable,
        );
        let _ = timeline.route_entry(second, 2);

        assert_eq!(
            timeline.archive_queue.back().map(|entry| entry.status),
            Some(ChronicleEntryStatus::Archived)
        );
    }

    #[test]
    fn chronicle_history_slice_returns_archived_entries() {
        let mut timeline = ChronicleTimeline::new();
        timeline.max_background = 1;

        for tick in 1..=3_u64 {
            let entry = sample_entry(
                &mut timeline,
                tick,
                Some(EntityId(30 + tick)),
                ChronicleEventType::GatheringFormation,
                ChronicleEventCause::Social,
                5.0,
                ChronicleSignificanceCategory::Notable,
            );
            let _ = timeline.route_entry(entry, tick);
        }

        let history =
            timeline.history_slice_snapshot(10, None, None, Some(timeline.snapshot_revision()));
        assert!(!history.revision_unavailable);
        assert_eq!(history.items.len(), 2);
        assert_eq!(history.items[0].end_tick, 2);
        assert_eq!(history.items[1].end_tick, 1);
        assert_eq!(history.telemetry.archive_count, 2);
    }

    #[test]
    fn chronicle_history_slice_paginates_by_cursor() {
        let mut timeline = ChronicleTimeline::new();
        timeline.max_background = 1;

        for tick in 1..=4_u64 {
            let entry = sample_entry(
                &mut timeline,
                tick,
                Some(EntityId(40 + tick)),
                ChronicleEventType::GatheringFormation,
                ChronicleEventCause::Social,
                5.0,
                ChronicleSignificanceCategory::Notable,
            );
            let _ = timeline.route_entry(entry, tick);
        }

        let first_page =
            timeline.history_slice_snapshot(2, None, None, Some(timeline.snapshot_revision()));
        assert_eq!(first_page.items.len(), 2);
        assert_eq!(first_page.items[0].end_tick, 3);
        assert_eq!(first_page.items[1].end_tick, 2);

        let second_page = timeline.history_slice_snapshot(
            2,
            first_page.next_cursor_before_tick,
            first_page.next_cursor_before_entry_id,
            Some(timeline.snapshot_revision()),
        );
        assert_eq!(second_page.items.len(), 1);
        assert_eq!(second_page.items[0].end_tick, 1);
    }

    #[test]
    fn chronicle_telemetry_counts_routing_outcomes() {
        let mut timeline = ChronicleTimeline::new();
        timeline.max_visible = 1;
        timeline.max_recall = 1;

        let major_visible = sample_entry(
            &mut timeline,
            1,
            Some(EntityId(100)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            7.0,
            ChronicleSignificanceCategory::Major,
        );
        let _ = timeline.route_entry(major_visible, 1);

        let notable = sample_entry(
            &mut timeline,
            2,
            Some(EntityId(101)),
            ChronicleEventType::GatheringFormation,
            ChronicleEventCause::Social,
            5.0,
            ChronicleSignificanceCategory::Notable,
        );
        let _ = timeline.route_entry(notable, 2);

        let major_recall = sample_entry(
            &mut timeline,
            3,
            Some(EntityId(102)),
            ChronicleEventType::ShelterSeeking,
            ChronicleEventCause::Warmth,
            7.0,
            ChronicleSignificanceCategory::Major,
        );
        let _ = timeline.route_entry(major_recall, 3);

        let critical = sample_entry(
            &mut timeline,
            4,
            Some(EntityId(103)),
            ChronicleEventType::InfluenceAvoidance,
            ChronicleEventCause::Danger,
            9.0,
            ChronicleSignificanceCategory::Critical,
        );
        let _ = timeline.route_entry(critical, 4);

        let minor = sample_entry(
            &mut timeline,
            5,
            Some(EntityId(104)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            0.5,
            ChronicleSignificanceCategory::Minor,
        );
        let _ = timeline.route_entry(minor, 5);

        let telemetry = timeline.telemetry_snapshot();
        assert_eq!(telemetry.total_routed, 5);
        assert_eq!(telemetry.visible_count, 2);
        assert_eq!(telemetry.background_count, 1);
        assert_eq!(telemetry.recall_count, 2);
        assert_eq!(telemetry.drop_count, 1);
        assert_eq!(telemetry.displacement_count, 1);
        assert_eq!(telemetry.archive_count, 1);
        assert!(telemetry.thread_create_count >= 4);
    }

    #[test]
    fn chronicle_integrity_detects_no_issues_on_healthy_timeline() {
        let mut timeline = ChronicleTimeline::new();
        let first = sample_entry(
            &mut timeline,
            1,
            Some(EntityId(200)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            7.0,
            ChronicleSignificanceCategory::Major,
        );
        let _ = timeline.route_entry(first, 1);

        let second = sample_entry(
            &mut timeline,
            2,
            Some(EntityId(201)),
            ChronicleEventType::GatheringFormation,
            ChronicleEventCause::Social,
            5.0,
            ChronicleSignificanceCategory::Notable,
        );
        let _ = timeline.route_entry(second, 2);

        assert!(timeline.validate_integrity().is_empty());
    }

    #[test]
    fn chronicle_timeline_rejects_unavailable_snapshot_revisions() {
        let timeline = ChronicleTimeline::new();
        let feed = timeline.feed_snapshot(5, Some(ChronicleSnapshotRevision(999)));
        let detail = timeline
            .entry_detail_snapshot(ChronicleEntryId(1), Some(ChronicleSnapshotRevision(999)));
        let recall = timeline.recall_slice_snapshot(5, Some(ChronicleSnapshotRevision(999)));
        let threads = timeline.story_threads_snapshot(5, Some(ChronicleSnapshotRevision(999)));
        let history =
            timeline.history_slice_snapshot(5, None, None, Some(ChronicleSnapshotRevision(999)));

        assert!(feed.revision_unavailable);
        assert!(detail.revision_unavailable);
        assert!(recall.revision_unavailable);
        assert!(threads.revision_unavailable);
        assert!(history.revision_unavailable);
    }

    #[test]
    fn chronicle_thread_attaches_entries_with_same_key() {
        let mut timeline = ChronicleTimeline::new();
        let first = sample_entry(
            &mut timeline,
            10,
            Some(EntityId(7)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        let first_id = first.entry_id;
        let _ = timeline.route_entry(first, 10);

        let second = sample_entry(
            &mut timeline,
            20,
            Some(EntityId(7)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            8.5,
            ChronicleSignificanceCategory::Major,
        );
        let second_id = second.entry_id;
        let _ = timeline.route_entry(second, 20);

        let first_thread_id = timeline.find_entry(first_id).and_then(|entry| entry.thread_id);
        let second_thread_id = timeline.find_entry(second_id).and_then(|entry| entry.thread_id);
        assert_eq!(first_thread_id, second_thread_id);
        let thread = timeline.threads.get(&second_thread_id.unwrap()).unwrap();
        assert_eq!(thread.state, ChronicleThreadState::Developing);
        assert_eq!(thread.entry_ids, vec![first_id, second_id]);
    }

    #[test]
    fn chronicle_thread_separates_different_event_families() {
        let mut timeline = ChronicleTimeline::new();
        let first = sample_entry(
            &mut timeline,
            10,
            Some(EntityId(7)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        let first_id = first.entry_id;
        let _ = timeline.route_entry(first, 10);

        let second = sample_entry(
            &mut timeline,
            20,
            Some(EntityId(7)),
            ChronicleEventType::InfluenceAvoidance,
            ChronicleEventCause::Danger,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        let second_id = second.entry_id;
        let _ = timeline.route_entry(second, 20);

        let first_thread_id = timeline.find_entry(first_id).and_then(|entry| entry.thread_id);
        let second_thread_id = timeline.find_entry(second_id).and_then(|entry| entry.thread_id);
        assert_ne!(first_thread_id, second_thread_id);
    }

    #[test]
    fn chronicle_thread_separates_different_entities() {
        let mut timeline = ChronicleTimeline::new();
        let first = sample_entry(
            &mut timeline,
            10,
            Some(EntityId(7)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        let first_id = first.entry_id;
        let _ = timeline.route_entry(first, 10);

        let second = sample_entry(
            &mut timeline,
            20,
            Some(EntityId(8)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        let second_id = second.entry_id;
        let _ = timeline.route_entry(second, 20);

        let first_thread_id = timeline.find_entry(first_id).and_then(|entry| entry.thread_id);
        let second_thread_id = timeline.find_entry(second_id).and_then(|entry| entry.thread_id);
        assert_ne!(first_thread_id, second_thread_id);
    }

    #[test]
    fn chronicle_thread_resolves_after_window() {
        let mut timeline = ChronicleTimeline::new();
        let first = sample_entry(
            &mut timeline,
            10,
            Some(EntityId(9)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        let first_id = first.entry_id;
        let _ = timeline.route_entry(first, 10);

        let second = sample_entry(
            &mut timeline,
            11,
            Some(EntityId(9)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        let _ = timeline.route_entry(second, 11);

        let thread_id = timeline.find_entry(first_id).and_then(|entry| entry.thread_id).unwrap();
        timeline.update_thread_states(11 + config::CHRONICLE_THREAD_RESOLVE_WINDOW_TICKS);
        assert_eq!(
            timeline.threads.get(&thread_id).map(|thread| thread.state),
            Some(ChronicleThreadState::Resolved)
        );
    }

    #[test]
    fn chronicle_thread_reactivates_on_new_entry() {
        let mut timeline = ChronicleTimeline::new();
        let first = sample_entry(
            &mut timeline,
            10,
            Some(EntityId(9)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        let first_id = first.entry_id;
        let _ = timeline.route_entry(first, 10);

        let second = sample_entry(
            &mut timeline,
            11,
            Some(EntityId(9)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        let _ = timeline.route_entry(second, 11);
        let thread_id = timeline.find_entry(first_id).and_then(|entry| entry.thread_id).unwrap();

        timeline.update_thread_states(11 + config::CHRONICLE_THREAD_RESOLVE_WINDOW_TICKS);
        assert_eq!(
            timeline.threads.get(&thread_id).map(|thread| thread.state),
            Some(ChronicleThreadState::Resolved)
        );

        let third = sample_entry(
            &mut timeline,
            12 + config::CHRONICLE_THREAD_RESOLVE_WINDOW_TICKS,
            Some(EntityId(9)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        let third_id = third.entry_id;
        let _ = timeline.route_entry(third, 12 + config::CHRONICLE_THREAD_RESOLVE_WINDOW_TICKS);

        assert_eq!(
            timeline.find_entry(third_id).and_then(|entry| entry.thread_id),
            Some(thread_id)
        );
        assert_eq!(
            timeline.threads.get(&thread_id).map(|thread| thread.state),
            Some(ChronicleThreadState::Developing)
        );
    }

    #[test]
    fn chronicle_thread_evicts_lowest_tension_resolved() {
        let mut timeline = ChronicleTimeline::new();
        let mut first_entry_id = None;
        let mut first_thread_id = None;

        for index in 0..config::CHRONICLE_THREAD_MAX_ACTIVE {
            let entry = sample_entry(
                &mut timeline,
                index as u64,
                Some(EntityId(index as u64 + 1)),
                ChronicleEventType::InfluenceAttraction,
                ChronicleEventCause::Food,
                8.0,
                ChronicleSignificanceCategory::Major,
            );
            let entry_id = entry.entry_id;
            let _ = timeline.route_entry(entry, index as u64);
            if index == 0 {
                first_entry_id = Some(entry_id);
                first_thread_id = timeline.find_entry(entry_id).and_then(|item| item.thread_id);
            }
        }

        let current_tick = config::CHRONICLE_THREAD_RESOLVE_WINDOW_TICKS + 1000;
        timeline.update_thread_states(current_tick);

        let overflow_entry = sample_entry(
            &mut timeline,
            current_tick,
            Some(EntityId(9999)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        let _ = timeline.route_entry(overflow_entry, current_tick);

        let first_thread_id = first_thread_id.unwrap();
        let first_entry_id = first_entry_id.unwrap();
        assert!(timeline.threads.len() <= config::CHRONICLE_THREAD_MAX_ACTIVE);
        assert!(!timeline.threads.contains_key(&first_thread_id));
        assert_eq!(
            timeline.find_entry(first_entry_id).and_then(|entry| entry.thread_id),
            None
        );
    }

    #[test]
    fn chronicle_thread_tension_decays_with_age() {
        let mut timeline = ChronicleTimeline::new();
        let entry = sample_entry(
            &mut timeline,
            0,
            Some(EntityId(55)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        let entry_id = entry.entry_id;
        let _ = timeline.route_entry(entry, 0);

        let thread_id = timeline.find_entry(entry_id).and_then(|item| item.thread_id).unwrap();
        let initial_tension = timeline.threads.get(&thread_id).unwrap().tension_score;
        timeline.update_thread_states(config::CHRONICLE_THREAD_TENSION_HALF_LIFE_TICKS as u64);
        let decayed_tension = timeline.threads.get(&thread_id).unwrap().tension_score;

        assert!(decayed_tension < initial_tension);
    }

    #[test]
    fn chronicle_thread_story_threads_snapshot_returns_by_tension() {
        let mut timeline = ChronicleTimeline::new();
        let old_entry = sample_entry(
            &mut timeline,
            0,
            Some(EntityId(1)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        let old_id = old_entry.entry_id;
        let _ = timeline.route_entry(old_entry, 0);

        timeline.update_thread_states(800);

        let new_entry = sample_entry(
            &mut timeline,
            800,
            Some(EntityId(2)),
            ChronicleEventType::InfluenceAvoidance,
            ChronicleEventCause::Danger,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        let new_id = new_entry.entry_id;
        let _ = timeline.route_entry(new_entry, 800);

        let old_thread_id = timeline.find_entry(old_id).and_then(|entry| entry.thread_id).unwrap();
        let new_thread_id = timeline.find_entry(new_id).and_then(|entry| entry.thread_id).unwrap();
        let threads = timeline.story_threads_snapshot(2, Some(timeline.snapshot_revision()));

        assert_eq!(threads.items.len(), 2);
        assert_eq!(threads.items[0].thread_id, new_thread_id.0);
        assert_eq!(threads.items[1].thread_id, old_thread_id.0);
        assert!(threads.items[0].tension_score >= threads.items[1].tension_score);
    }

    #[test]
    fn chronicle_thread_feed_item_propagates_thread_id() {
        let mut timeline = ChronicleTimeline::new();
        let entry = sample_entry(
            &mut timeline,
            10,
            Some(EntityId(7)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            8.0,
            ChronicleSignificanceCategory::Major,
        );
        let entry_id = entry.entry_id;
        let _ = timeline.route_entry(entry, 10);

        let thread_id = timeline.find_entry(entry_id).and_then(|item| item.thread_id).unwrap();
        let response = timeline.feed_snapshot(1, Some(timeline.snapshot_revision()));
        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].thread_id, Some(thread_id.0));
    }

    #[test]
    fn chronicle_subject_ref_state_defaults_to_unknown() {
        assert_eq!(ChronicleEntityRefState::default(), ChronicleEntityRefState::Unknown);
    }

    #[test]
    fn chronicle_subject_ref_preserves_frozen_state() {
        let mut timeline = ChronicleTimeline::new();
        let mut entry = sample_entry(
            &mut timeline,
            5,
            Some(EntityId(77)),
            ChronicleEventType::InfluenceAvoidance,
            ChronicleEventCause::Danger,
            9.0,
            ChronicleSignificanceCategory::Critical,
        );
        entry.entity_ref.ref_state = ChronicleEntityRefState::Dead;
        let entry_id = entry.entry_id;
        let _ = timeline.route_entry(entry, 5);

        assert_eq!(
            timeline
                .entry_detail_snapshot(entry_id, Some(timeline.snapshot_revision()))
                .entry
                .as_ref()
                .map(|item| item.entity_ref.ref_state),
            Some(ChronicleEntityRefState::Dead)
        );
    }

    #[test]
    fn chronicle_significance_meta_decomposes_score() {
        let meta = ChronicleSignificanceMeta {
            base_score: 4.0,
            cause_bonus: 3.0,
            group_bonus: 2.0,
            repeat_penalty: 1.5,
            final_score: 7.5,
            reason_tags: vec!["cause:danger".to_string()],
        };

        let recomposed = meta.base_score + meta.cause_bonus + meta.group_bonus - meta.repeat_penalty;
        assert!((recomposed - meta.final_score).abs() < f64::EPSILON);
    }

    #[test]
    fn chronicle_significance_meta_captures_repeat_penalty() {
        let mut timeline = ChronicleTimeline::new();
        for tick in 0..config::CHRONICLE_VISIBLE_MAX_ENTRIES as u64 {
            let mut entry = sample_entry(
                &mut timeline,
                tick,
                Some(EntityId(88)),
                ChronicleEventType::InfluenceAttraction,
                ChronicleEventCause::Food,
                7.0,
                ChronicleSignificanceCategory::Major,
            );
            entry.significance_meta.repeat_penalty = 1.5;
            let _ = timeline.route_entry(entry, tick);
        }

        let latest = timeline.recent_entries(1).first().copied().expect("latest entry");
        assert!(latest.significance_meta.repeat_penalty > 0.0);
    }

    #[test]
    fn chronicle_lifecycle_records_initial_routing_transition() {
        let mut timeline = ChronicleTimeline::new();
        let entry = sample_entry(
            &mut timeline,
            15,
            Some(EntityId(9)),
            ChronicleEventType::InfluenceAttraction,
            ChronicleEventCause::Food,
            7.0,
            ChronicleSignificanceCategory::Major,
        );
        let entry_id = entry.entry_id;
        let _ = timeline.route_entry(entry, 15);
        let entry = timeline.find_entry(entry_id).expect("entry");

        assert_eq!(entry.queue_transitions.len(), 1);
        assert_eq!(entry.queue_transitions[0].from, ChronicleQueueBucket::Dropped);
        assert_eq!(entry.queue_transitions[0].to, ChronicleQueueBucket::Visible);
    }

    #[test]
    fn chronicle_lifecycle_records_displacement() {
        let mut timeline = ChronicleTimeline::new();
        for index in 0..config::CHRONICLE_VISIBLE_MAX_ENTRIES {
            let entry = sample_entry(
                &mut timeline,
                index as u64,
                Some(EntityId(index as u64 + 1)),
                ChronicleEventType::InfluenceAttraction,
                ChronicleEventCause::Food,
                7.0,
                ChronicleSignificanceCategory::Major,
            );
            let _ = timeline.route_entry(entry, index as u64);
        }

        let critical = sample_entry(
            &mut timeline,
            999,
            Some(EntityId(999)),
            ChronicleEventType::InfluenceAvoidance,
            ChronicleEventCause::Danger,
            9.0,
            ChronicleSignificanceCategory::Critical,
        );
        let _ = timeline.route_entry(critical, 999);

        let recalled = timeline
            .recall_slice_snapshot(1, Some(timeline.snapshot_revision()))
            .items
            .into_iter()
            .next()
            .expect("recalled entry");
        let detail = timeline
            .entry_detail_snapshot(recalled.entry_id, Some(timeline.snapshot_revision()))
            .entry
            .expect("detail");

        assert_eq!(
            detail.displacement_reason.as_deref(),
            Some("displaced_by_critical")
        );
        assert!(detail.queue_transitions.len() >= 2);
        assert_eq!(
            detail.queue_transitions.last().map(|transition| transition.to),
            Some(ChronicleQueueBucket::Recall)
        );
    }
}
