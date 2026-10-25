//! Data-defined pressure identities and existing pressure mechanics.
use super::{IdentityDef, RuleSpan, RuleU32};
use crate::colony::content::AnimalKind;
/// Existing pressure mechanisms, independent of content identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum PressureMechanic {
    /// Cold exposure and extra fuel use.
    Cold,
    /// Reduced regrowth and increased thirst.
    Drought,
    /// A predator pack.
    Pack,
    /// Sickness and contagion.
    Illness,
}
/// Validated PressureDef metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PressureDef<A = AnimalKind> {
    /// Identity.
    pub identity: IdentityDef,
    /// Category.
    pub category: String,
    /// Mechanic.
    pub mechanic: PressureMechanic,
    /// Major.
    pub major: bool,
    /// Enabled rule.
    pub enabled_rule: Option<String>,
    /// Duration.
    pub duration: RuleSpan,
    /// Duration draw slot.
    pub duration_draw_slot: Option<u32>,
    /// Mercy.
    pub mercy: PressureMercy,
    /// Due days.
    pub due_days: Option<RuleU32>,
    /// Due seasons.
    pub due_seasons: Vec<u32>,
    /// Intensity.
    pub intensity: PressureIntensity,
    /// Cold.
    pub cold: Option<ColdEffects>,
    /// Drought.
    pub drought: Option<DroughtEffects>,
    /// Illness.
    pub illness: Option<IllnessEffects>,
    /// Pack.
    pub pack: Option<PackEffects<A>>,
}
/// Validated PressureMercy metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PressureMercy {
    /// First day.
    pub first_day: Option<RuleU32>,
    /// First winter days.
    pub first_winter_days: Option<RuleU32>,
    /// Repeat gap days.
    pub repeat_gap_days: Option<RuleU32>,
    /// Winter end days.
    pub winter_end_days: Option<RuleU32>,
}
/// Existing intensity calculations, selected by data.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PressureIntensity {
    /// No additional intensity.
    None,
    /// Extend a weather pressure for a large colony.
    ExtendForAdults {
        /// Adult threshold.
        threshold: RuleU32,
        /// Days added when threshold is met.
        extra_days: u32,
    },
    /// Derive pack size from stockpiled wealth.
    PackByWealth {
        /// Minimum pack size.
        base: RuleU32,
        /// Items per additional animal.
        per_items: RuleU32,
        /// Maximum pack size.
        max: RuleU32,
    },
}
/// Validated ColdEffects metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColdEffects {
    /// Burn.
    pub burn: RuleU32,
    /// Exposure.
    pub exposure: RuleU32,
    /// Outdoor work.
    pub outdoor_work: RuleU32,
}
/// Validated DroughtEffects metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DroughtEffects {
    /// Regrowth.
    pub regrowth: RuleU32,
    /// Thirst.
    pub thirst: RuleU32,
}
/// Validated IllnessEffects metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IllnessEffects {
    /// Work.
    pub work: RuleU32,
    /// Sleep.
    pub sleep: RuleU32,
    /// Spread.
    pub spread: RuleU32,
}
/// Validated PackEffects metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackEffects<A = AnimalKind> {
    /// Animal.
    pub animal: A,
}
/// One cumulative seasonal draw boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PressureDraw {
    /// Exclusive upper boundary of the draw in 0..1000.
    pub upper_exclusive: u16,
    /// Quiet draw when absent.
    pub kind: Option<crate::colony::content::PressureKind>,
}
/// Frozen seasonal scheduling policy; new identities do not rewrite base policy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PressureSchedule {
    /// Draw rows keyed by mechanical season index.
    pub seasons: std::collections::BTreeMap<u32, Vec<PressureDraw>>,
    /// Due pressure precedence.
    pub due_order: Vec<crate::colony::content::PressureKind>,
    /// Optional minor pressure identity.
    pub minor: Option<crate::colony::content::PressureKind>,
    /// Seasons allowing the minor draw.
    pub minor_seasons: Vec<u32>,
    /// Existing minor keyed draw slot.
    pub minor_draw_slot: u32,
}
