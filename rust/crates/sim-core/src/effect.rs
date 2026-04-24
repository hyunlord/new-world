use serde::{Deserialize, Serialize};

use crate::config;
use crate::influence_channel::ChannelId;
use crate::influence_grid::{EmitterRecord, FalloffType};
use crate::EmotionType;
use crate::EntityId;

/// Shared stat targets addressable by effect primitives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectStat {
    /// Hunger need scalar.
    Hunger,
    /// Warmth need scalar.
    Warmth,
    /// Safety need scalar.
    Safety,
    /// Comfort diagnostic scalar (apply-only; not mapped to any NeedType).
    Comfort,
    /// Energy/rest scalar.
    Energy,
    /// Meaning need scalar (L3 Growth — purpose/significance).
    Meaning,
}

/// Shared boolean flags addressable by effect primitives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectFlag {
    /// Actor is currently sheltered.
    Sheltered,
    /// Actor is currently unsafe.
    Unsafe,
    /// Actor is currently resting or sleeping.
    Resting,
}

/// ECS component describing an entity that emits influence into the grid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InfluenceEmitter {
    /// Influence channel emitted by the entity.
    pub channel: ChannelId,
    /// Influence radius in tiles.
    pub radius: f64,
    /// Raw base intensity before normalization and clamp.
    pub base_intensity: f64,
    /// Falloff profile for propagation.
    pub falloff: FalloffType,
    /// Optional per-emitter source attenuation applied during stamping.
    pub decay_rate: Option<f64>,
    /// Optional semantic tags for downstream rule/debug consumers.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Whether the emitter is active.
    pub enabled: bool,
}

impl InfluenceEmitter {
    /// Converts the component into a runtime grid emitter record.
    pub fn to_record(&self, x: u32, y: u32) -> EmitterRecord {
        EmitterRecord {
            x,
            y,
            channel: self.channel,
            radius: self.radius,
            base_intensity: self.base_intensity,
            falloff: self.falloff,
            decay_rate: self.decay_rate,
            tags: self.tags.clone(),
            dirty: self.enabled,
        }
    }
}

impl Default for InfluenceEmitter {
    fn default() -> Self {
        Self {
            channel: ChannelId::Warmth,
            radius: 0.0,
            base_intensity: 0.0,
            falloff: FalloffType::Linear,
            decay_rate: None,
            tags: Vec::new(),
            enabled: false,
        }
    }
}

/// ECS component describing which influence channels an entity samples.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfluenceReceiver {
    /// Channels sampled by the entity.
    pub channels: Vec<ChannelId>,
}

impl InfluenceReceiver {
    /// Returns true when the receiver listens to the given channel.
    pub fn listens_to(&self, channel: ChannelId) -> bool {
        self.channels.contains(&channel)
    }
}

impl Default for InfluenceReceiver {
    fn default() -> Self {
        Self {
            channels: ChannelId::all().to_vec(),
        }
    }
}

/// Minimal shared effect primitive scaffold for simulation-side integration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EffectPrimitive {
    /// Adds a scalar delta to a target stat.
    AddStat { stat: EffectStat, amount: f64 },
    /// Multiplies a target stat by a scalar factor.
    MulStat { stat: EffectStat, factor: f64 },
    /// Sets a boolean-like flag.
    SetFlag { flag: EffectFlag, active: bool },
    /// Emits an influence signal into the grid.
    EmitInfluence { emitter: InfluenceEmitter },
    /// Requests a named event emission.
    SpawnEvent { event_key: String },
    /// Schedules another effect after a delay.
    Schedule {
        delay_ticks: u64,
        effect: Box<EffectPrimitive>,
    },
    /// Adjusts an emotion intensity by a signed delta (Plutchik 8 emotions).
    AdjustEmotion {
        emotion: EmotionType,
        amount: f64,
    },
}

/// Identifies the origin of an effect for causal logging.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectSource {
    /// System or subsystem that enqueued the effect.
    pub system: String,
    /// Stable kind or event identifier used in debug/causal output.
    pub kind: String,
}

/// One queued effect targeting a specific entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectEntry {
    /// Target entity that should receive the effect.
    pub entity: EntityId,
    /// The effect primitive to apply.
    pub effect: EffectPrimitive,
    /// Causal source metadata for debug and explain output.
    pub source: EffectSource,
}

/// Effect scheduled for future application.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduledEffect {
    /// Tick at which the effect should be promoted into the active buffer.
    pub fire_tick: u64,
    /// The queued effect entry to apply when promoted.
    pub entry: EffectEntry,
}

/// Double-buffered effect queue shared across runtime systems.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectQueue {
    /// Write-side queue populated during the current tick.
    pending: Vec<EffectEntry>,
    /// Read-side queue drained by the effect apply system.
    active: Vec<EffectEntry>,
    /// Delayed effects waiting for their fire tick.
    scheduled: Vec<ScheduledEffect>,
}

impl EffectQueue {
    /// Creates an empty queue.
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            active: Vec::new(),
            scheduled: Vec::new(),
        }
    }

    /// Enqueues one effect for application on the next flush.
    pub fn push(&mut self, entry: EffectEntry) {
        if self.pending.len() < config::EFFECT_QUEUE_MAX_PENDING {
            self.pending.push(entry);
        }
    }

    /// Enqueues one delayed effect.
    pub fn push_scheduled(&mut self, effect: ScheduledEffect) {
        if self.scheduled.len() < config::EFFECT_QUEUE_MAX_SCHEDULED {
            self.scheduled.push(effect);
        }
    }

    /// Promotes pending and due scheduled effects into the active buffer.
    pub fn flush(&mut self, current_tick: u64) {
        self.active.clear();
        std::mem::swap(&mut self.active, &mut self.pending);

        let mut remaining = Vec::with_capacity(self.scheduled.len());
        for scheduled in self.scheduled.drain(..) {
            if scheduled.fire_tick <= current_tick {
                self.active.push(scheduled.entry);
            } else {
                remaining.push(scheduled);
            }
        }
        self.scheduled = remaining;
    }

    /// Drains the active buffer for one application pass.
    pub fn drain_active(&mut self) -> Vec<EffectEntry> {
        std::mem::take(&mut self.active)
    }

    /// Returns the number of entries waiting in the pending buffer.
    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    /// Returns a read-only view of the pending buffer for test inspection.
    pub fn pending(&self) -> &[EffectEntry] {
        &self.pending
    }

    /// Returns the number of entries available in the active buffer.
    pub fn active_len(&self) -> usize {
        self.active.len()
    }

    /// Returns the number of delayed scheduled effects.
    pub fn scheduled_len(&self) -> usize {
        self.scheduled.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn influence_emitter_converts_into_grid_record() {
        let emitter = InfluenceEmitter {
            channel: ChannelId::Light,
            radius: 4.0,
            base_intensity: 0.6,
            falloff: FalloffType::Exponential,
            decay_rate: Some(0.1),
            tags: vec!["light_source".to_string()],
            enabled: true,
        };

        let record = emitter.to_record(3, 7);

        assert_eq!(record.x, 3);
        assert_eq!(record.y, 7);
        assert_eq!(record.channel, ChannelId::Light);
        assert!(record.dirty);
        assert_eq!(record.tags, vec!["light_source".to_string()]);
    }

    #[test]
    fn influence_receiver_default_listens_to_all_channels() {
        let receiver = InfluenceReceiver::default();
        assert!(receiver.listens_to(ChannelId::Warmth));
        assert!(receiver.listens_to(ChannelId::Danger));
    }

    fn sample_entry(id: u64) -> EffectEntry {
        EffectEntry {
            entity: EntityId(id),
            effect: EffectPrimitive::AddStat {
                stat: EffectStat::Hunger,
                amount: -0.1,
            },
            source: EffectSource {
                system: "test_system".to_string(),
                kind: "sample_effect".to_string(),
            },
        }
    }

    #[test]
    fn effect_queue_flush_swaps_pending_to_active() {
        let mut queue = EffectQueue::new();
        queue.push(sample_entry(1));
        queue.push(sample_entry(2));
        queue.push(sample_entry(3));

        queue.flush(10);
        let drained = queue.drain_active();

        assert_eq!(drained.len(), 3);
        assert_eq!(queue.pending_len(), 0);
        assert_eq!(queue.active_len(), 0);
    }

    #[test]
    fn effect_queue_scheduled_promotes_on_tick() {
        let mut queue = EffectQueue::new();
        queue.push_scheduled(ScheduledEffect {
            fire_tick: 15,
            entry: sample_entry(7),
        });

        queue.flush(15);
        let drained = queue.drain_active();

        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].entity, EntityId(7));
        assert_eq!(queue.scheduled_len(), 0);
    }

    #[test]
    fn effect_queue_scheduled_retains_future() {
        let mut queue = EffectQueue::new();
        queue.push_scheduled(ScheduledEffect {
            fire_tick: 20,
            entry: sample_entry(9),
        });

        queue.flush(15);

        assert_eq!(queue.drain_active().len(), 0);
        assert_eq!(queue.scheduled_len(), 1);
    }

    #[test]
    fn effect_queue_double_buffer_isolation() {
        let mut queue = EffectQueue::new();
        queue.push(sample_entry(1));
        assert_eq!(queue.active_len(), 0);

        queue.flush(1);
        queue.push(sample_entry(2));

        let drained = queue.drain_active();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].entity, EntityId(1));
        assert_eq!(queue.pending_len(), 1);
        assert_eq!(queue.scheduled_len(), 0);
    }
}
