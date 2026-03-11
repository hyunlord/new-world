use serde::{Deserialize, Serialize};

use crate::influence_channel::ChannelId;
use crate::influence_grid::{EmitterRecord, FalloffType};

/// Shared stat targets addressable by effect primitives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectStat {
    /// Hunger need scalar.
    Hunger,
    /// Warmth need scalar.
    Warmth,
    /// Safety need scalar.
    Safety,
    /// Comfort diagnostic scalar.
    Comfort,
    /// Energy/rest scalar.
    Energy,
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
}
