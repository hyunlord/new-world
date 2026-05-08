//! Influence channel definitions.
//!
//! Phase 0 Section 2.3.1 base — 8 channels, each with explicit decay,
//! aggregation, update tier and wall-blocking policy.
//!
//! v0.1.1 fixes wired here:
//! - ISSUE 1: Warmth source = STATIC heat sources only (agent body heat is
//!   handled by a separate component in Phase 4 Agent Core).
//! - ISSUE 2: linear decay (Noise/Danger) and wall blocking are documented
//!   as separate mechanisms (Songs of Syx 2-tile reference).
//! - ISSUE 3: Danger uses linear `alpha = 5` with sight-radius cap (15).
//! - ISSUE 4: Light propagates via recursive shadowcasting, not RayCast.
//! - ISSUE 6: per-channel aggregation policy (`AggKind`).
//! - ISSUE 9: Social skips `LodTier::Far`/`Dormant` agents.

use serde::{Deserialize, Serialize};

/// 8 base influence channels.
///
/// `#[repr(u8)]` so the variants can index `[T; 8]` arrays directly.
///
/// Per-channel decay, aggregation and update-tier policies are encoded as
/// methods on this enum so callers cannot drift from the spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum InfluenceChannel {
    /// Environmental warmth from STATIC heat sources (hearth, stove, sun).
    ///
    /// v0.1.1 ISSUE 1 fix: agent body heat is NOT stamped here; it lives on
    /// a per-agent component handled in Phase 4.
    ///
    /// Decay: exponential, `k = 0.15`. Aggregation: `Additive`.
    Warmth = 0,

    /// Light / line-of-sight intensity.
    ///
    /// v0.1.1 ISSUE 4 fix: propagated via recursive shadowcasting (Adam
    /// Milazzo / Björn Bergström symmetric variant), not 8-octant raycast.
    /// Aggregation: `Max`.
    Light = 1,

    /// Noise.
    ///
    /// Decay: linear, `alpha = 15`. Wall blocking is a separate mechanism
    /// driven by material density (Songs of Syx 2-tile reference) — v0.1.1
    /// ISSUE 2 fix. Aggregation: `Max`.
    Noise = 2,

    /// Food aroma.
    ///
    /// Decay: exponential, `k = 0.10`. Aggregation: `Max`.
    FoodAroma = 3,

    /// Danger (predators, fire, hostile agents).
    ///
    /// v0.1.1 ISSUE 3 fix: linear `alpha = 5` with a sight-radius cap of
    /// 15 tiles to prevent global panic spread. Aggregation: `Max`.
    Danger = 4,

    /// Social density (presence of other agents within ~5 tiles).
    ///
    /// Source: agent presence (+1 per agent in radius). v0.1.1 ISSUE 9
    /// fix: only `LodTier::Full`/`Medium` agents stamp this channel.
    /// Decay: BFS aggregate. Aggregation: `Additive`.
    Social = 5,

    /// Spiritual / ritual influence.
    ///
    /// Decay: exponential, `k = 0.08`. Aggregation: `Max`.
    Spiritual = 6,

    /// Aesthetic / beauty influence.
    ///
    /// Decay: exponential, `k = 0.12`. Aggregation: `Max`.
    Beauty = 7,
}

impl InfluenceChannel {
    /// Total number of base channels (used for fixed-size arrays).
    pub const COUNT: usize = 8;

    /// All channels in canonical order. Iterate this rather than the
    /// numeric range so additions are caught at compile time.
    pub fn all() -> &'static [InfluenceChannel] {
        &[
            Self::Warmth,
            Self::Light,
            Self::Noise,
            Self::FoodAroma,
            Self::Danger,
            Self::Social,
            Self::Spiritual,
            Self::Beauty,
        ]
    }

    /// Per-channel aggregation policy.
    ///
    /// v0.1.1 ISSUE 6 fix — Warmth and Social accumulate from multiple
    /// sources (`Additive`), the rest take the strongest source (`Max`).
    pub fn aggregation(&self) -> AggKind {
        match self {
            Self::Warmth | Self::Social => AggKind::Additive,
            Self::Light
            | Self::Noise
            | Self::FoodAroma
            | Self::Danger
            | Self::Spiritual
            | Self::Beauty => AggKind::Max,
        }
    }

    /// Update-frequency tier classification (Phase 0 Section 2.6).
    pub fn update_tier(&self) -> UpdateTier {
        match self {
            Self::Danger | Self::Noise => UpdateTier::Hot,
            Self::Light | Self::FoodAroma | Self::Social => UpdateTier::Warm,
            Self::Warmth | Self::Spiritual | Self::Beauty => UpdateTier::Cold,
        }
    }
}

/// Aggregation policy for a channel.
///
/// v0.1.1 ISSUE 6 fix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggKind {
    /// `saturating_add` — overlapping sources sum (caps at `255`). Used by
    /// Warmth and Social.
    Additive,
    /// `saturating_max` — strongest source dominates. Used by everything
    /// else.
    Max,
}

/// Update-frequency tier (Phase 0 Section 2.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateTier {
    /// Every tick (30 TPS).
    Hot,
    /// Staggered (1 channel per tick).
    Warm,
    /// Event-driven, dirty-region only.
    Cold,
}

/// Decay function kind used by [`ChannelDef`].
///
/// v0.1.1 ISSUE 4 — Light no longer uses raycast; the dedicated
/// `Shadowcasting` variant is the canonical option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecayKind {
    /// `intensity_next = max(0, intensity - alpha)` per step.
    Linear,
    /// `intensity_next = intensity * exp(-k)` per step.
    Exponential,
    /// Recursive shadowcasting (Adam Milazzo). Applies to `Light` only.
    Shadowcasting,
    /// Aggregate from neighbouring sources (BFS sum). Applies to `Social`.
    BFSAggregate,
}

/// How a wall material derives its blocking coefficient for a channel.
///
/// Mods declaring custom channels via [`ChannelDef`] use this enum to
/// describe how blocking should be computed from material properties.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockingDerive {
    /// Derived from `1 - thermal_conductivity / 400`.
    Thermal,
    /// Derived from `density / 25_000`.
    Density,
    /// `1.0` for any solid wall, `0.0` otherwise.
    Binary,
    /// No blocking (e.g. Danger spreads regardless of walls).
    None,
    /// Custom formula expressed as a string (interpreted by the mod system
    /// in a later phase).
    Custom {
        /// Formula source (mod-defined; not parsed by the core crate).
        formula: String,
    },
}

/// Mod-compatible channel definition.
///
/// Phase 0 Section 2.8.1 base. v0.1.1 ISSUE 6 fix introduces the explicit
/// `aggregation` field so mods cannot accidentally collide with built-in
/// per-channel policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelDef {
    /// Stable string id used by data files.
    pub id: String,
    /// Localisation key for the human-readable name.
    pub display_name_key: String,
    /// Decay function used by this channel.
    pub decay: DecayKind,
    /// Decay parameter (`k` for exponential, `alpha` for linear).
    pub decay_param: f32,
    /// Maximum propagation radius in tiles.
    pub max_radius: u32,
    /// How frequently the channel is recomputed.
    pub update_tier: UpdateTier,
    /// Aggregation policy (Additive vs Max).
    pub aggregation: AggKind,
    /// How wall blocking is derived from material properties.
    pub wall_blocking_derive: BlockingDerive,
}

// ----------------- serde for UpdateTier / AggKind -----------------
//
// These are tag-style enums so we serialise/deserialise as plain strings
// rather than relying on the default tagged representation.

impl Serialize for UpdateTier {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let name = match self {
            Self::Hot => "Hot",
            Self::Warm => "Warm",
            Self::Cold => "Cold",
        };
        s.serialize_str(name)
    }
}

impl<'de> Deserialize<'de> for UpdateTier {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        match s.as_str() {
            "Hot" => Ok(Self::Hot),
            "Warm" => Ok(Self::Warm),
            "Cold" => Ok(Self::Cold),
            other => Err(serde::de::Error::custom(format!(
                "invalid UpdateTier: {other}"
            ))),
        }
    }
}

impl Serialize for AggKind {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let name = match self {
            Self::Additive => "Additive",
            Self::Max => "Max",
        };
        s.serialize_str(name)
    }
}

impl<'de> Deserialize<'de> for AggKind {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        match s.as_str() {
            "Additive" => Ok(Self::Additive),
            "Max" => Ok(Self::Max),
            other => Err(serde::de::Error::custom(format!(
                "invalid AggKind: {other}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_count_8() {
        assert_eq!(InfluenceChannel::COUNT, 8);
        assert_eq!(InfluenceChannel::all().len(), 8);
    }

    #[test]
    fn test_aggregation_warmth_additive() {
        assert_eq!(InfluenceChannel::Warmth.aggregation(), AggKind::Additive);
    }

    #[test]
    fn test_aggregation_social_additive() {
        assert_eq!(InfluenceChannel::Social.aggregation(), AggKind::Additive);
    }

    #[test]
    fn test_aggregation_others_max() {
        for ch in [
            InfluenceChannel::Light,
            InfluenceChannel::Noise,
            InfluenceChannel::FoodAroma,
            InfluenceChannel::Danger,
            InfluenceChannel::Spiritual,
            InfluenceChannel::Beauty,
        ] {
            assert_eq!(ch.aggregation(), AggKind::Max, "{ch:?} should be Max");
        }
    }

    #[test]
    fn test_update_tier_classification() {
        assert_eq!(InfluenceChannel::Danger.update_tier(), UpdateTier::Hot);
        assert_eq!(InfluenceChannel::Noise.update_tier(), UpdateTier::Hot);
        assert_eq!(InfluenceChannel::Light.update_tier(), UpdateTier::Warm);
        assert_eq!(InfluenceChannel::FoodAroma.update_tier(), UpdateTier::Warm);
        assert_eq!(InfluenceChannel::Social.update_tier(), UpdateTier::Warm);
        assert_eq!(InfluenceChannel::Warmth.update_tier(), UpdateTier::Cold);
        assert_eq!(InfluenceChannel::Spiritual.update_tier(), UpdateTier::Cold);
        assert_eq!(InfluenceChannel::Beauty.update_tier(), UpdateTier::Cold);
    }

    #[test]
    fn test_repr_u8_indexing() {
        // The repr(u8) discriminants must match COUNT-sized array indices.
        assert_eq!(InfluenceChannel::Warmth as usize, 0);
        assert_eq!(InfluenceChannel::Beauty as usize, 7);
    }

    #[test]
    fn test_update_tier_serde_roundtrip() {
        let tiers = [UpdateTier::Hot, UpdateTier::Warm, UpdateTier::Cold];
        for t in tiers {
            let s = ron::to_string(&t).expect("serialize");
            let back: UpdateTier = ron::from_str(&s).expect("deserialize");
            assert_eq!(back, t);
        }
    }

    #[test]
    fn test_agg_kind_serde_roundtrip() {
        for k in [AggKind::Additive, AggKind::Max] {
            let s = ron::to_string(&k).expect("serialize");
            let back: AggKind = ron::from_str(&s).expect("deserialize");
            assert_eq!(back, k);
        }
    }

    #[test]
    fn test_channel_def_ron_parse() {
        let ron = r#"(
            id: "warmth",
            display_name_key: "CHANNEL_WARMTH",
            decay: Exponential,
            decay_param: 0.15,
            max_radius: 12,
            update_tier: "Cold",
            aggregation: "Additive",
            wall_blocking_derive: Thermal,
        )"#;
        let def: ChannelDef = ron::from_str(ron).expect("parse ChannelDef");
        assert_eq!(def.id, "warmth");
        assert_eq!(def.aggregation, AggKind::Additive);
        assert_eq!(def.update_tier, UpdateTier::Cold);
        assert_eq!(def.decay, DecayKind::Exponential);
        assert_eq!(def.wall_blocking_derive, BlockingDerive::Thermal);
    }
}
