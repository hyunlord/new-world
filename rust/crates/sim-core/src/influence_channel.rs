use crate::config;
use serde::{Deserialize, Serialize};

/// Clamp behavior applied after one propagation pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelClampPolicy {
    /// Compress values with `x / (1 + |x|)` into `[-1.0, 1.0]`.
    Sigmoid,
    /// Clamp values directly into `[0.0, 1.0]`.
    UnitInterval,
}

/// Typed influence channels used by the spatial causality runtime.
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ChannelId {
    Food = 0,
    Danger = 1,
    Warmth = 2,
    Social = 3,
    Authority = 4,
    Noise = 5,
    Disease = 6,
    Light = 7,
    Spiritual = 8,
    Beauty = 9,
}

/// Channel-level propagation metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelMeta {
    /// Stable typed identifier for this channel.
    pub id: ChannelId,
    /// Stable string key used for data/bridge/debug mapping.
    pub name: String,
    /// Per-tick decay multiplier input, clamped to `0.0..=1.0`.
    pub decay_rate: f64,
    /// Default radius used when an emitter omits an explicit radius.
    pub default_radius: f64,
    /// Maximum propagation radius processed for this channel.
    pub max_radius: u32,
    /// Wall attenuation sensitivity in `0.0..=1.0`.
    pub wall_blocking_sensitivity: f64,
    /// Post-propagation clamp/compression policy.
    pub clamp_policy: ChannelClampPolicy,
}

impl ChannelMeta {
    /// Returns a copy with all numeric fields clamped into valid ranges.
    pub fn sanitized(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            decay_rate: self.decay_rate.clamp(0.0, 1.0),
            default_radius: self.default_radius.max(0.0),
            max_radius: self.max_radius.max(1),
            wall_blocking_sensitivity: self.wall_blocking_sensitivity.clamp(0.0, 1.0),
            clamp_policy: self.clamp_policy,
        }
    }
}

impl ChannelId {
    /// Returns the stable zero-based index for this channel.
    pub fn index(self) -> usize {
        self as usize
    }

    /// Returns the number of built-in channels.
    pub fn count() -> usize {
        10
    }

    /// Returns the built-in channel ordering.
    pub fn all() -> [ChannelId; 10] {
        [
            Self::Food,
            Self::Danger,
            Self::Warmth,
            Self::Social,
            Self::Authority,
            Self::Noise,
            Self::Disease,
            Self::Light,
            Self::Spiritual,
            Self::Beauty,
        ]
    }

    /// Returns the stable string key for this channel.
    pub fn key(self) -> &'static str {
        match self {
            Self::Food => "food",
            Self::Danger => "danger",
            Self::Warmth => "warmth",
            Self::Social => "social",
            Self::Authority => "authority",
            Self::Noise => "noise",
            Self::Disease => "disease",
            Self::Light => "light",
            Self::Spiritual => "spiritual",
            Self::Beauty => "beauty",
        }
    }

    /// Returns the typed channel id for a registry/debug string key.
    pub fn from_key(key: &str) -> Option<Self> {
        let normalized = key.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "food" | "food_aroma" | "forage" => Some(Self::Food),
            "danger" | "fear" | "unsafe" => Some(Self::Danger),
            "warmth" | "heat" | "shelter" => Some(Self::Warmth),
            "social" | "craft" => Some(Self::Social),
            "authority" => Some(Self::Authority),
            "noise" => Some(Self::Noise),
            "disease" | "sickness" => Some(Self::Disease),
            "light" => Some(Self::Light),
            "spiritual" | "faith" => Some(Self::Spiritual),
            "beauty" => Some(Self::Beauty),
            _ => None,
        }
    }

    /// Returns the built-in channel metadata.
    pub fn default_meta(self) -> ChannelMeta {
        match self {
            Self::Food => ChannelMeta {
                id: self,
                name: self.key().to_string(),
                decay_rate: config::INFLUENCE_FOOD_DECAY_RATE,
                default_radius: config::INFLUENCE_FOOD_DEFAULT_RADIUS,
                max_radius: config::INFLUENCE_FOOD_MAX_RADIUS,
                wall_blocking_sensitivity: config::INFLUENCE_FOOD_WALL_BLOCK,
                clamp_policy: ChannelClampPolicy::UnitInterval,
            },
            Self::Danger => ChannelMeta {
                id: self,
                name: self.key().to_string(),
                decay_rate: config::INFLUENCE_DANGER_DECAY_RATE,
                default_radius: config::INFLUENCE_DANGER_DEFAULT_RADIUS,
                max_radius: config::INFLUENCE_DANGER_MAX_RADIUS,
                wall_blocking_sensitivity: config::INFLUENCE_DANGER_WALL_BLOCK,
                clamp_policy: ChannelClampPolicy::UnitInterval,
            },
            Self::Warmth => ChannelMeta {
                id: self,
                name: self.key().to_string(),
                decay_rate: config::INFLUENCE_WARMTH_DECAY_RATE,
                default_radius: config::INFLUENCE_WARMTH_DEFAULT_RADIUS,
                max_radius: config::INFLUENCE_WARMTH_MAX_RADIUS,
                wall_blocking_sensitivity: config::INFLUENCE_WARMTH_WALL_BLOCK,
                clamp_policy: ChannelClampPolicy::UnitInterval,
            },
            Self::Social => ChannelMeta {
                id: self,
                name: self.key().to_string(),
                decay_rate: config::INFLUENCE_SOCIAL_DECAY_RATE,
                default_radius: config::INFLUENCE_SOCIAL_DEFAULT_RADIUS,
                max_radius: config::INFLUENCE_SOCIAL_MAX_RADIUS,
                wall_blocking_sensitivity: config::INFLUENCE_SOCIAL_WALL_BLOCK,
                clamp_policy: ChannelClampPolicy::Sigmoid,
            },
            Self::Authority => ChannelMeta {
                id: self,
                name: self.key().to_string(),
                decay_rate: config::INFLUENCE_AUTHORITY_DECAY_RATE,
                default_radius: config::INFLUENCE_AUTHORITY_DEFAULT_RADIUS,
                max_radius: config::INFLUENCE_AUTHORITY_MAX_RADIUS,
                wall_blocking_sensitivity: config::INFLUENCE_AUTHORITY_WALL_BLOCK,
                clamp_policy: ChannelClampPolicy::Sigmoid,
            },
            Self::Noise => ChannelMeta {
                id: self,
                name: self.key().to_string(),
                decay_rate: config::INFLUENCE_NOISE_DECAY_RATE,
                default_radius: config::INFLUENCE_NOISE_DEFAULT_RADIUS,
                max_radius: config::INFLUENCE_NOISE_MAX_RADIUS,
                wall_blocking_sensitivity: config::INFLUENCE_NOISE_WALL_BLOCK,
                clamp_policy: ChannelClampPolicy::Sigmoid,
            },
            Self::Disease => ChannelMeta {
                id: self,
                name: self.key().to_string(),
                decay_rate: config::INFLUENCE_DISEASE_DECAY_RATE,
                default_radius: config::INFLUENCE_DISEASE_DEFAULT_RADIUS,
                max_radius: config::INFLUENCE_DISEASE_MAX_RADIUS,
                wall_blocking_sensitivity: config::INFLUENCE_DISEASE_WALL_BLOCK,
                clamp_policy: ChannelClampPolicy::UnitInterval,
            },
            Self::Light => ChannelMeta {
                id: self,
                name: self.key().to_string(),
                decay_rate: config::INFLUENCE_LIGHT_DECAY_RATE,
                default_radius: config::INFLUENCE_LIGHT_DEFAULT_RADIUS,
                max_radius: config::INFLUENCE_LIGHT_MAX_RADIUS,
                wall_blocking_sensitivity: config::INFLUENCE_LIGHT_WALL_BLOCK,
                clamp_policy: ChannelClampPolicy::UnitInterval,
            },
            Self::Spiritual => ChannelMeta {
                id: self,
                name: self.key().to_string(),
                decay_rate: config::INFLUENCE_SPIRITUAL_DECAY_RATE,
                default_radius: config::INFLUENCE_SPIRITUAL_DEFAULT_RADIUS,
                max_radius: config::INFLUENCE_SPIRITUAL_MAX_RADIUS,
                wall_blocking_sensitivity: config::INFLUENCE_SPIRITUAL_WALL_BLOCK,
                clamp_policy: ChannelClampPolicy::Sigmoid,
            },
            Self::Beauty => ChannelMeta {
                id: self,
                name: self.key().to_string(),
                decay_rate: config::INFLUENCE_BEAUTY_DECAY_RATE,
                default_radius: config::INFLUENCE_BEAUTY_DEFAULT_RADIUS,
                max_radius: config::INFLUENCE_BEAUTY_MAX_RADIUS,
                wall_blocking_sensitivity: config::INFLUENCE_BEAUTY_WALL_BLOCK,
                clamp_policy: ChannelClampPolicy::Sigmoid,
            },
        }
    }

    /// Returns built-in metadata in channel index order.
    pub fn default_channels() -> Vec<ChannelMeta> {
        Self::all()
            .into_iter()
            .map(ChannelId::default_meta)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn influence_channel_indices_are_stable() {
        let indices: Vec<usize> = ChannelId::all().into_iter().map(ChannelId::index).collect();
        assert_eq!(indices, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(ChannelId::count(), 10);
    }

    #[test]
    fn influence_channel_aliases_cover_legacy_keys() {
        assert_eq!(ChannelId::from_key("food_aroma"), Some(ChannelId::Food));
        assert_eq!(ChannelId::from_key("craft"), Some(ChannelId::Social));
        assert_eq!(ChannelId::from_key("shelter"), Some(ChannelId::Warmth));
        assert_eq!(ChannelId::from_key("unknown"), None);
    }

    #[test]
    fn influence_channel_defaults_cover_all_channels() {
        let channels = ChannelId::default_channels();
        assert_eq!(channels.len(), ChannelId::count());
        assert_eq!(channels[ChannelId::Food.index()].id, ChannelId::Food);
        assert_eq!(channels[ChannelId::Warmth.index()].max_radius, 10);
        assert_eq!(
            channels[ChannelId::Danger.index()].clamp_policy,
            ChannelClampPolicy::UnitInterval
        );
    }
}
