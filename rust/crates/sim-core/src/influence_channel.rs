use crate::config;
use serde::{Deserialize, Serialize};

/// Influence channels available in the initial A-2 grid.
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelId {
    Warmth = 0,
    Light = 1,
    Noise = 2,
    Danger = 3,
    FoodAroma = 4,
    Spiritual = 5,
    Authority = 6,
    Beauty = 7,
}

/// Channel-level propagation and decay metadata.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChannelMeta {
    /// Stable channel identifier.
    pub id: ChannelId,
    /// Per-tick decay multiplier input, clamped to `0.0..=1.0`.
    pub decay_rate: f64,
    /// Maximum radius sweep depth processed per tick update.
    pub propagation_speed: u32,
    /// Default wall sensitivity for this channel.
    pub default_wall_block: f64,
}

impl ChannelId {
    /// Returns the stable zero-based index for this channel.
    pub fn index(self) -> usize {
        self as usize
    }

    /// Returns the number of built-in channels.
    pub fn count() -> usize {
        8
    }

    /// Returns the built-in channel ordering.
    pub fn all() -> [ChannelId; 8] {
        [
            Self::Warmth,
            Self::Light,
            Self::Noise,
            Self::Danger,
            Self::FoodAroma,
            Self::Spiritual,
            Self::Authority,
            Self::Beauty,
        ]
    }

    /// Returns the default metadata for this channel.
    pub fn default_meta(self) -> ChannelMeta {
        match self {
            Self::Warmth => ChannelMeta {
                id: self,
                decay_rate: config::INFLUENCE_WARMTH_DECAY_RATE,
                propagation_speed: config::INFLUENCE_WARMTH_PROPAGATION_SPEED,
                default_wall_block: config::INFLUENCE_WARMTH_WALL_BLOCK,
            },
            Self::Light => ChannelMeta {
                id: self,
                decay_rate: config::INFLUENCE_LIGHT_DECAY_RATE,
                propagation_speed: config::INFLUENCE_LIGHT_PROPAGATION_SPEED,
                default_wall_block: config::INFLUENCE_LIGHT_WALL_BLOCK,
            },
            Self::Noise => ChannelMeta {
                id: self,
                decay_rate: config::INFLUENCE_NOISE_DECAY_RATE,
                propagation_speed: config::INFLUENCE_NOISE_PROPAGATION_SPEED,
                default_wall_block: config::INFLUENCE_NOISE_WALL_BLOCK,
            },
            Self::Danger => ChannelMeta {
                id: self,
                decay_rate: config::INFLUENCE_DANGER_DECAY_RATE,
                propagation_speed: config::INFLUENCE_DANGER_PROPAGATION_SPEED,
                default_wall_block: config::INFLUENCE_DANGER_WALL_BLOCK,
            },
            Self::FoodAroma => ChannelMeta {
                id: self,
                decay_rate: config::INFLUENCE_FOOD_AROMA_DECAY_RATE,
                propagation_speed: config::INFLUENCE_FOOD_AROMA_PROPAGATION_SPEED,
                default_wall_block: config::INFLUENCE_FOOD_AROMA_WALL_BLOCK,
            },
            Self::Spiritual => ChannelMeta {
                id: self,
                decay_rate: config::INFLUENCE_SPIRITUAL_DECAY_RATE,
                propagation_speed: config::INFLUENCE_SPIRITUAL_PROPAGATION_SPEED,
                default_wall_block: config::INFLUENCE_SPIRITUAL_WALL_BLOCK,
            },
            Self::Authority => ChannelMeta {
                id: self,
                decay_rate: config::INFLUENCE_AUTHORITY_DECAY_RATE,
                propagation_speed: config::INFLUENCE_AUTHORITY_PROPAGATION_SPEED,
                default_wall_block: config::INFLUENCE_AUTHORITY_WALL_BLOCK,
            },
            Self::Beauty => ChannelMeta {
                id: self,
                decay_rate: config::INFLUENCE_BEAUTY_DECAY_RATE,
                propagation_speed: config::INFLUENCE_BEAUTY_PROPAGATION_SPEED,
                default_wall_block: config::INFLUENCE_BEAUTY_WALL_BLOCK,
            },
        }
    }

    /// Returns a `Vec` of built-in channel metadata in index order.
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
        assert_eq!(indices, vec![0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(ChannelId::count(), 8);
    }

    #[test]
    fn influence_channel_defaults_cover_all_channels() {
        let channels = ChannelId::default_channels();
        assert_eq!(channels.len(), ChannelId::count());
        assert_eq!(channels[ChannelId::Warmth.index()].id, ChannelId::Warmth);
        assert_eq!(channels[ChannelId::Light.index()].propagation_speed, 5);
        assert!(
            (channels[ChannelId::Danger.index()].default_wall_block - 0.0).abs() < f64::EPSILON
        );
    }
}
